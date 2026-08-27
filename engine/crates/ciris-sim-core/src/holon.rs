//! Recursive holons and certified adaptive refinement.
//!
//! A ball, a storyline, a quark, a person, and an NPC are not variants of an enum here.
//! They are the same recursively compositional holon. Typed realizations are executable
//! charts over that common maximal choice object, never replacement ontologies. The core
//! stores identity-independent structure, additive REG+ gross state, irreducible whole-state,
//! active channels, and a boundary flag. Realization-specific update, readout, and
//! certification mathematics lives in implementations of [`BoundaryModel`].
//!
//! The arena is fixed-capacity and allocator-free. It is a resident refinement window:
//! a [`Decomposition::Latent`] holon represents recursively implied children that are not
//! currently materialised; [`Decomposition::Expanded`] means its immediate children are
//! resident and their REG+ gross states must compose exactly to the parent.

use crate::regplus::GrossState;

pub const NO_HOLON: usize = usize::MAX;

/// Executable realizations available on a holon. Multiple channels may be active
/// simultaneously. They select update/readout/certification machinery over the common
/// holon; they are not mutually exclusive kinds or independent ontologies.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Channels(u32);

impl Channels {
    pub const NONE: Self = Self(0);
    pub const REG_PLUS: Self = Self(1 << 0);
    pub const MECHANICAL: Self = Self(1 << 1);
    pub const QUANTUM: Self = Self(1 << 2);
    pub const NARRATIVE: Self = Self(1 << 3);
    pub const AGENTIC: Self = Self(1 << 4);

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Preserve all channel bits, including channels defined by a newer host.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u32 {
        self.0
    }
}

/// Whether this holon's recursively implied children are resident.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Decomposition {
    /// Terminal at the grain constant.
    Leaf,
    /// Children exist in the model but are outside the resident refinement window.
    #[default]
    Latent,
    /// Immediate children are resident in the arena.
    Expanded,
}

/// One recursively compositional holon: simultaneously a whole and a part.
///
/// `whole` is deliberately not constrained to equal any function of the children. It is
/// storage for the whole-only state required by the ontology's non-factoring results.
/// Eviction rule (amendment A2/M20): the slot holds ONLY what provably fails to factor
/// through the declared partial views. Anything computable from resident children by
/// running a chart — specimen strength, percolation state, ensemble summaries — is a
/// chart output or a descriptor field, never `whole` storage.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Holon<const W: usize> {
    pub parent: usize,
    pub depth: u16,
    /// Current grain diameter in integer multiples of the terminal grain constant.
    pub grain_units: u32,
    pub gross: GrossState,
    pub whole: [f64; W],
    pub channels: Channels,
    /// Whether finer dynamics may be required at this holon's interaction boundary.
    pub boundary: bool,
    pub decomposition: Decomposition,
}

impl<const W: usize> Holon<W> {
    pub const EMPTY: Self = Self {
        parent: NO_HOLON,
        depth: 0,
        grain_units: 0,
        gross: GrossState::ZERO,
        whole: [0.0; W],
        channels: Channels::NONE,
        boundary: false,
        decomposition: Decomposition::Latent,
    };

    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        parent: usize,
        depth: u16,
        grain_units: u32,
        gross: GrossState,
        whole: [f64; W],
        channels: Channels,
        boundary: bool,
        decomposition: Decomposition,
    ) -> Self {
        Self {
            parent,
            depth,
            grain_units,
            gross,
            whole,
            channels,
            boundary,
            decomposition,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HolonError {
    Empty,
    Capacity,
    InvalidRoot,
    MultipleRoots,
    InvalidParent,
    InvalidDepth,
    InvalidGrain,
    InvalidDecomposition,
    GrossStateDoesNotCompose,
    FrontierDoesNotCoverRoot,
}

/// Fixed-capacity resident window over one recursively defined root holon.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HolonArena<const CAP: usize, const W: usize> {
    holons: [Holon<W>; CAP],
    len: usize,
    root: usize,
}

impl<const CAP: usize, const W: usize> HolonArena<CAP, W> {
    pub fn from_holons(
        holons: [Holon<W>; CAP],
        len: usize,
        root: usize,
    ) -> Result<Self, HolonError> {
        let arena = Self { holons, len, root };
        arena.validate()?;
        Ok(arena)
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn root(&self) -> usize {
        self.root
    }

    pub fn holon(&self, index: usize) -> Option<&Holon<W>> {
        if index < self.len {
            Some(&self.holons[index])
        } else {
            None
        }
    }

    pub fn is_descendant_or_self(&self, holon: usize, ancestor: usize) -> bool {
        if holon >= self.len || ancestor >= self.len {
            return false;
        }
        let mut current = holon;
        let mut remaining = self.len + 1;
        while remaining > 0 {
            if current == ancestor {
                return true;
            }
            let parent = self.holons[current].parent;
            if parent == NO_HOLON || parent >= self.len {
                return false;
            }
            current = parent;
            remaining -= 1;
        }
        false
    }

    fn child_count(&self, parent: usize) -> usize {
        let mut count = 0;
        for holon in &self.holons[..self.len] {
            if holon.parent == parent {
                count += 1;
            }
        }
        count
    }

    pub fn validate(&self) -> Result<(), HolonError> {
        if self.len == 0 || self.len > CAP {
            return Err(HolonError::Empty);
        }
        if self.root >= self.len || self.holons[self.root].parent != NO_HOLON {
            return Err(HolonError::InvalidRoot);
        }

        let mut roots = 0;
        for i in 0..self.len {
            let holon = self.holons[i];
            if holon.grain_units == 0 {
                return Err(HolonError::InvalidGrain);
            }
            if holon.parent == NO_HOLON {
                roots += 1;
                if i != self.root || holon.depth != 0 {
                    return Err(HolonError::InvalidRoot);
                }
            } else {
                if holon.parent >= self.len || holon.parent == i {
                    return Err(HolonError::InvalidParent);
                }
                let parent = self.holons[holon.parent];
                if holon.depth
                    != parent
                        .depth
                        .checked_add(1)
                        .ok_or(HolonError::InvalidDepth)?
                {
                    return Err(HolonError::InvalidDepth);
                }
                if holon.grain_units >= parent.grain_units {
                    return Err(HolonError::InvalidGrain);
                }
                if !self.is_descendant_or_self(holon.parent, self.root) {
                    return Err(HolonError::InvalidParent);
                }
            }

            let child_count = self.child_count(i);
            match holon.decomposition {
                Decomposition::Leaf if child_count != 0 || holon.grain_units != 1 => {
                    return Err(HolonError::InvalidDecomposition);
                }
                Decomposition::Latent if child_count != 0 || holon.grain_units == 1 => {
                    return Err(HolonError::InvalidDecomposition);
                }
                Decomposition::Expanded if child_count == 0 => {
                    return Err(HolonError::InvalidDecomposition);
                }
                _ => {}
            }

            if holon.decomposition == Decomposition::Expanded {
                let mut composed = GrossState::ZERO;
                for child in &self.holons[..self.len] {
                    if child.parent == i {
                        composed = composed
                            .checked_combine(child.gross)
                            .ok_or(HolonError::GrossStateDoesNotCompose)?;
                    }
                }
                if composed != holon.gross {
                    return Err(HolonError::GrossStateDoesNotCompose);
                }
            }
        }
        if roots != 1 {
            return Err(HolonError::MultipleRoots);
        }
        Ok(())
    }
}

/// A non-overlapping set of resident holons whose gross states cover the root.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frontier<const CAP: usize> {
    active: [bool; CAP],
}

impl<const CAP: usize> Frontier<CAP> {
    pub fn root<const W: usize>(arena: &HolonArena<CAP, W>) -> Self {
        let mut active = [false; CAP];
        active[arena.root] = true;
        Self { active }
    }

    pub fn is_active(&self, holon: usize) -> bool {
        holon < CAP && self.active[holon]
    }

    pub fn active_count<const W: usize>(&self, arena: &HolonArena<CAP, W>) -> usize {
        self.active[..arena.len]
            .iter()
            .filter(|active| **active)
            .count()
    }

    pub fn finest_grain<const W: usize>(&self, arena: &HolonArena<CAP, W>) -> u32 {
        let mut finest = u32::MAX;
        for (i, active) in self.active[..arena.len].iter().enumerate() {
            if *active {
                finest = finest.min(arena.holons[i].grain_units);
            }
        }
        finest
    }

    pub fn represented_grain<const W: usize>(
        &self,
        arena: &HolonArena<CAP, W>,
        holon: usize,
    ) -> u32 {
        let mut grain = 0;
        for i in 0..arena.len {
            if !self.active[i] {
                continue;
            }
            if arena.is_descendant_or_self(holon, i)
                || (arena.is_descendant_or_self(i, holon) && arena.holons[i].boundary)
            {
                grain = grain.max(arena.holons[i].grain_units);
            }
        }
        if grain == 0 {
            arena.holons[holon].grain_units
        } else {
            grain
        }
    }

    pub fn refine<const W: usize>(
        &mut self,
        arena: &HolonArena<CAP, W>,
        holon: usize,
    ) -> Result<(), HolonError> {
        if holon >= arena.len || !self.active[holon] {
            return Err(HolonError::FrontierDoesNotCoverRoot);
        }
        if arena.holons[holon].decomposition != Decomposition::Expanded {
            return Err(HolonError::InvalidDecomposition);
        }
        self.active[holon] = false;
        for i in 0..arena.len {
            if arena.holons[i].parent == holon {
                self.active[i] = true;
            }
        }
        debug_assert!(self.validate(arena).is_ok());
        Ok(())
    }

    pub fn validate<const W: usize>(&self, arena: &HolonArena<CAP, W>) -> Result<(), HolonError> {
        let mut gross = GrossState::ZERO;
        for i in 0..arena.len {
            if !self.active[i] {
                continue;
            }
            for j in 0..arena.len {
                if i != j && self.active[j] && arena.is_descendant_or_self(i, j) {
                    return Err(HolonError::FrontierDoesNotCoverRoot);
                }
            }
            gross = gross
                .checked_combine(arena.holons[i].gross)
                .ok_or(HolonError::FrontierDoesNotCoverRoot)?;
        }
        if gross != arena.holons[arena.root].gross {
            return Err(HolonError::FrontierDoesNotCoverRoot);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Evaluation<const O: usize> {
    pub observables: [f64; O],
    pub macro_error_bound: f64,
    pub conservation_residual: f64,
}

/// Realization-specific gross-plus-boundary mathematics over the common holon.
pub trait BoundaryModel<const CAP: usize, const W: usize, const O: usize> {
    fn evaluate(&mut self, arena: &HolonArena<CAP, W>, frontier: &Frontier<CAP>) -> Evaluation<O>;

    /// Larger values are refined first. Return a non-positive value to keep an active
    /// holon aggregated for this interaction.
    fn refinement_priority(
        &self,
        arena: &HolonArena<CAP, W>,
        frontier: &Frontier<CAP>,
        holon: usize,
    ) -> f64;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CertificationStatus {
    Certified,
    GrainFloor,
    RefinementUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolutionCertificate<const CAP: usize, const O: usize> {
    pub status: CertificationStatus,
    pub frontier: Frontier<CAP>,
    pub observables: [f64; O],
    pub macro_error_bound: f64,
    pub conservation_residual: f64,
    pub evaluations: usize,
}

impl<const CAP: usize, const O: usize> ResolutionCertificate<CAP, O> {
    pub const fn passed(&self) -> bool {
        matches!(self.status, CertificationStatus::Certified)
    }
}

/// Select the coarsest resident refinement frontier certified by the supplied model.
pub fn certify<const CAP: usize, const W: usize, const O: usize>(
    arena: &HolonArena<CAP, W>,
    model: &mut impl BoundaryModel<CAP, W, O>,
    macro_tolerance: f64,
    conservation_tolerance: f64,
) -> ResolutionCertificate<CAP, O> {
    assert!(macro_tolerance >= 0.0 && conservation_tolerance >= 0.0);
    let mut frontier = Frontier::root(arena);
    let mut evaluations = 0;

    loop {
        let evaluation = model.evaluate(arena, &frontier);
        evaluations += 1;
        if evaluation.macro_error_bound <= macro_tolerance
            && evaluation.conservation_residual <= conservation_tolerance
        {
            return ResolutionCertificate {
                status: CertificationStatus::Certified,
                frontier,
                observables: evaluation.observables,
                macro_error_bound: evaluation.macro_error_bound,
                conservation_residual: evaluation.conservation_residual,
                evaluations,
            };
        }

        let mut candidate = NO_HOLON;
        let mut priority = 0.0;
        let mut boundary_at_floor = false;
        for holon in 0..arena.len {
            if !frontier.active[holon] || !arena.holons[holon].boundary {
                continue;
            }
            if arena.holons[holon].grain_units == 1 {
                boundary_at_floor = true;
            }
            if arena.holons[holon].decomposition != Decomposition::Expanded {
                continue;
            }
            let score = model.refinement_priority(arena, &frontier, holon);
            if score > priority {
                priority = score;
                candidate = holon;
            }
        }

        if candidate == NO_HOLON {
            return ResolutionCertificate {
                status: if boundary_at_floor {
                    CertificationStatus::GrainFloor
                } else {
                    CertificationStatus::RefinementUnavailable
                },
                frontier,
                observables: evaluation.observables,
                macro_error_bound: evaluation.macro_error_bound,
                conservation_residual: evaluation.conservation_residual,
                evaluations,
            };
        }
        frontier
            .refine(arena, candidate)
            .expect("validated expanded holon must refine into a valid frontier");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAP: usize = 7;
    const W: usize = 2;

    fn gross(elements: u64) -> GrossState {
        GrossState::aggregate(elements, elements * 2, [elements as i64, 0])
    }

    fn arena() -> HolonArena<CAP, W> {
        let channels = Channels::REG_PLUS
            .union(Channels::MECHANICAL)
            .union(Channels::NARRATIVE);
        let holons = [
            Holon::new(
                NO_HOLON,
                0,
                4,
                gross(8),
                [1.0, 0.25],
                channels,
                true,
                Decomposition::Expanded,
            ),
            Holon::new(
                0,
                1,
                2,
                gross(4),
                [0.4, 0.0],
                channels,
                true,
                Decomposition::Expanded,
            ),
            Holon::new(
                0,
                1,
                2,
                gross(4),
                [0.6, 0.0],
                channels,
                false,
                Decomposition::Expanded,
            ),
            Holon::new(
                1,
                2,
                1,
                gross(2),
                [0.0; W],
                channels,
                true,
                Decomposition::Leaf,
            ),
            Holon::new(
                1,
                2,
                1,
                gross(2),
                [0.0; W],
                channels,
                false,
                Decomposition::Leaf,
            ),
            Holon::new(
                2,
                2,
                1,
                gross(2),
                [0.0; W],
                channels,
                false,
                Decomposition::Leaf,
            ),
            Holon::new(
                2,
                2,
                1,
                gross(2),
                [0.0; W],
                channels,
                false,
                Decomposition::Leaf,
            ),
        ];
        HolonArena::from_holons(holons, CAP, 0).unwrap()
    }

    #[test]
    fn recursive_holon_is_one_type_with_multiple_realizations() {
        let arena = arena();
        let root = arena.holon(arena.root()).unwrap();
        assert!(root.channels.contains(Channels::REG_PLUS));
        assert!(root.channels.contains(Channels::MECHANICAL));
        assert!(root.channels.contains(Channels::NARRATIVE));
        assert_eq!(root.whole, [1.0, 0.25]);
    }

    #[test]
    fn frontier_refinement_preserves_the_gross_holon() {
        let arena = arena();
        let mut frontier = Frontier::root(&arena);
        assert_eq!(frontier.active_count(&arena), 1);
        frontier.refine(&arena, 0).unwrap();
        assert_eq!(frontier.active_count(&arena), 2);
        frontier.refine(&arena, 1).unwrap();
        assert_eq!(frontier.active_count(&arena), 3);
        assert_eq!(frontier.finest_grain(&arena), 1);
    }

    struct TestBoundaryModel;

    impl BoundaryModel<CAP, W, 1> for TestBoundaryModel {
        fn evaluate(
            &mut self,
            arena: &HolonArena<CAP, W>,
            frontier: &Frontier<CAP>,
        ) -> Evaluation<1> {
            let grain = frontier.represented_grain(arena, 1) as f64;
            Evaluation {
                observables: [1.0 - 0.01 * grain],
                macro_error_bound: 0.0002 * grain * grain,
                conservation_residual: 0.0,
            }
        }

        fn refinement_priority(
            &self,
            arena: &HolonArena<CAP, W>,
            _frontier: &Frontier<CAP>,
            holon: usize,
        ) -> f64 {
            arena.holons[holon].grain_units as f64
        }
    }

    #[test]
    fn selector_returns_the_coarsest_passing_boundary_frontier() {
        let arena = arena();
        let certificate = certify(&arena, &mut TestBoundaryModel, 0.001, 1.0e-12);
        assert!(certificate.passed());
        assert_eq!(certificate.frontier.represented_grain(&arena, 1), 2);
        assert_eq!(certificate.evaluations, 2);
        assert!(certificate.macro_error_bound <= 0.001);
    }
}
