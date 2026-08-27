//! Runtime-sized storage and refinement for Rust, browser WASM, and component hosts.
//!
//! [`crate::holon::HolonArena`] is the fully stack-resident path: its capacity and
//! whole-state width are compile-time constants. This module is the complementary
//! `no_std + alloc` path used when scenes arrive at runtime. Holon headers remain contiguous,
//! but variable-width whole-state is kept in one flat scalar pool rather than one heap
//! allocation per holon. The hot holon header therefore has a fixed representation and
//! never contains a Rust pointer, `Vec`, trait object, or recursive child allocation.
//!
//! Hosts should link this crate directly into a Rust or browser-WASM application for the
//! stepping path. A WIT/component adapter can translate typed records into this arena at
//! a trust boundary without making canonical-ABI copies part of each solver iteration.

use alloc::vec;
use alloc::vec::Vec;
use core::ops::Range;

use crate::holon::{CertificationStatus, Channels, Decomposition, Evaluation, HolonError};
use crate::regplus::GrossState;

/// Sentinel used by the packed runtime header for a holon with no parent.
pub const NO_RUNTIME_HOLON: u32 = u32::MAX;

/// Fixed-width holon header. Variable-width whole-state lives in the arena's scalar
/// pool at `whole_offset..whole_offset + whole_len`.
///
/// `repr(C)` makes snapshots layout-auditable. It is not permission to exchange raw
/// Rust memory across a WebAssembly Component boundary; WIT adapters map its fields.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeHolon {
    pub gross: GrossState,
    pub parent: u32,
    pub grain_units: u32,
    pub whole_offset: u32,
    pub whole_len: u32,
    pub channels: Channels,
    pub depth: u16,
    pub decomposition: Decomposition,
    boundary: u8,
}

impl RuntimeHolon {
    pub const fn is_boundary(&self) -> bool {
        self.boundary != 0
    }
}

/// Borrowed input accepted by [`RuntimeArenaBuilder::push`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeHolonSpec<'a> {
    pub parent: u32,
    pub depth: u16,
    pub grain_units: u32,
    pub gross: GrossState,
    pub whole: &'a [f64],
    pub channels: Channels,
    pub boundary: bool,
    pub decomposition: Decomposition,
}

/// Builder permits holons to be streamed from a file, JS holarchy, or WIT list while
/// retaining two contiguous allocations: the fixed-width headers and the scalar pool.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeArenaBuilder {
    holons: Vec<RuntimeHolon>,
    whole: Vec<f64>,
}

impl RuntimeArenaBuilder {
    pub fn with_capacity(holons: usize, whole_scalars: usize) -> Self {
        Self {
            holons: Vec::with_capacity(holons),
            whole: Vec::with_capacity(whole_scalars),
        }
    }

    pub fn push(&mut self, spec: RuntimeHolonSpec<'_>) -> Result<u32, HolonError> {
        let id = u32::try_from(self.holons.len()).map_err(|_| HolonError::Capacity)?;
        let whole_offset = u32::try_from(self.whole.len()).map_err(|_| HolonError::Capacity)?;
        let whole_len = u32::try_from(spec.whole.len()).map_err(|_| HolonError::Capacity)?;
        self.whole
            .len()
            .checked_add(spec.whole.len())
            .and_then(|len| u32::try_from(len).ok())
            .ok_or(HolonError::Capacity)?;

        self.whole.extend_from_slice(spec.whole);
        self.holons.push(RuntimeHolon {
            gross: spec.gross,
            parent: spec.parent,
            grain_units: spec.grain_units,
            whole_offset,
            whole_len,
            channels: spec.channels,
            depth: spec.depth,
            decomposition: spec.decomposition,
            boundary: u8::from(spec.boundary),
        });
        Ok(id)
    }

    pub fn build(self, root: u32) -> Result<RuntimeArena, HolonError> {
        let len = self.holons.len();
        let mut arena = RuntimeArena {
            holons: self.holons,
            whole: self.whole,
            root,
            first_child: vec![NO_RUNTIME_HOLON; len],
            next_sibling: vec![NO_RUNTIME_HOLON; len],
        };
        arena.validate()?;
        // Reverse pass with prepend yields each chain in ascending id order, matching
        // the scan order the frontier historically activated children in.
        for i in (0..len).rev() {
            let parent = arena.holons[i].parent;
            if parent != NO_RUNTIME_HOLON {
                arena.next_sibling[i] = arena.first_child[parent as usize];
                arena.first_child[parent as usize] = i as u32;
            }
        }
        debug_assert!(arena.child_index_matches_headers());
        Ok(arena)
    }
}

/// Runtime-sized resident window over one recursively defined root holon.
///
/// Alongside the flat holon headers the arena keeps a first-child/next-sibling index
/// (two `u32` per holon) so that enumerating a holon's children is `O(children)`
/// instead of an arena-wide scan. For builder-supplied holons the index is derived in
/// one pass at [`RuntimeArenaBuilder::build`]; for materialized children it is exactly
/// the contiguous id range [`RuntimeArena::materialize`] already returns. The index is
/// bookkeeping over the same parent pointers — [`RuntimeArena::validate`] deliberately
/// keeps recomputing child relations from the headers so a corrupted index cannot
/// vouch for itself, and frontier validation catches a wrong index at the first
/// refinement (see the mutation tests).
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeArena {
    holons: Vec<RuntimeHolon>,
    whole: Vec<f64>,
    root: u32,
    first_child: Vec<u32>,
    next_sibling: Vec<u32>,
}

impl RuntimeArena {
    pub fn from_specs(specs: &[RuntimeHolonSpec<'_>], root: u32) -> Result<Self, HolonError> {
        let whole_scalars = specs
            .iter()
            .try_fold(0_usize, |total, spec| total.checked_add(spec.whole.len()));
        let mut builder = RuntimeArenaBuilder::with_capacity(
            specs.len(),
            whole_scalars.ok_or(HolonError::Capacity)?,
        );
        for spec in specs {
            builder.push(*spec)?;
        }
        builder.build(root)
    }

    pub fn len(&self) -> usize {
        self.holons.len()
    }

    pub fn is_empty(&self) -> bool {
        self.holons.is_empty()
    }

    pub const fn root(&self) -> u32 {
        self.root
    }

    pub fn holons(&self) -> &[RuntimeHolon] {
        &self.holons
    }

    pub fn whole_scalars(&self) -> &[f64] {
        &self.whole
    }

    pub fn holon(&self, index: usize) -> Option<&RuntimeHolon> {
        self.holons.get(index)
    }

    pub fn whole_state(&self, index: usize) -> Option<&[f64]> {
        let holon = self.holons.get(index)?;
        let start = holon.whole_offset as usize;
        let end = start.checked_add(holon.whole_len as usize)?;
        self.whole.get(start..end)
    }

    /// Iterate the immediate children of `holon` in ascending id order, in
    /// `O(children)` via the arena's child index — never an arena-wide scan.
    pub fn children(&self, holon: usize) -> impl Iterator<Item = usize> + '_ {
        let mut next = self
            .first_child
            .get(holon)
            .copied()
            .unwrap_or(NO_RUNTIME_HOLON);
        core::iter::from_fn(move || {
            if next == NO_RUNTIME_HOLON {
                return None;
            }
            let current = next as usize;
            next = self.next_sibling[current];
            Some(current)
        })
    }

    /// Independent recomputation: does the child index name exactly the children the
    /// holon headers declare? Debug-assert insurance at construction sites; the
    /// runtime fence is frontier validation, which fails on any frontier a wrong
    /// index produces.
    fn child_index_matches_headers(&self) -> bool {
        let mut linked_parent = vec![NO_RUNTIME_HOLON; self.len()];
        for parent in 0..self.len() {
            let mut previous = NO_RUNTIME_HOLON;
            for child in self.children(parent) {
                if child >= self.len() || linked_parent[child] != NO_RUNTIME_HOLON {
                    return false;
                }
                // Ascending id order is part of the contract.
                if previous != NO_RUNTIME_HOLON && child as u32 <= previous {
                    return false;
                }
                previous = child as u32;
                linked_parent[child] = parent as u32;
            }
        }
        self.holons
            .iter()
            .enumerate()
            .all(|(i, holon)| linked_parent[i] == holon.parent)
    }

    /// Transactionally replace one latent holon with resident immediate children.
    ///
    /// Child IDs are append-only, so existing handles held by a Bevy ECS or WASM host
    /// remain stable. The operation commits only if depth, grain, terminal state, and
    /// exact REG+ composition all validate. It returns the contiguous range of new IDs.
    pub fn materialize(
        &mut self,
        parent: usize,
        children: &[RuntimeHolonSpec<'_>],
    ) -> Result<Range<u32>, HolonError> {
        let parent_record = self
            .holons
            .get(parent)
            .copied()
            .ok_or(HolonError::InvalidParent)?;
        if parent_record.decomposition != Decomposition::Latent
            || parent_record.grain_units == 1
            || children.is_empty()
        {
            return Err(HolonError::InvalidDecomposition);
        }
        let parent_id = u32::try_from(parent).map_err(|_| HolonError::Capacity)?;
        let child_depth = parent_record
            .depth
            .checked_add(1)
            .ok_or(HolonError::InvalidDepth)?;
        let start = u32::try_from(self.holons.len()).map_err(|_| HolonError::Capacity)?;
        let final_node_len = self
            .holons
            .len()
            .checked_add(children.len())
            .and_then(|len| u32::try_from(len).ok())
            .ok_or(HolonError::Capacity)?;
        let additional_whole = children
            .iter()
            .try_fold(0_usize, |total, child| total.checked_add(child.whole.len()))
            .ok_or(HolonError::Capacity)?;
        let final_whole_len = self
            .whole
            .len()
            .checked_add(additional_whole)
            .and_then(|len| u32::try_from(len).ok())
            .ok_or(HolonError::Capacity)?;

        let mut composed = GrossState::ZERO;
        for child in children {
            if child.parent != parent_id {
                return Err(HolonError::InvalidParent);
            }
            if child.depth != child_depth {
                return Err(HolonError::InvalidDepth);
            }
            if child.grain_units == 0 || child.grain_units >= parent_record.grain_units {
                return Err(HolonError::InvalidGrain);
            }
            match child.decomposition {
                Decomposition::Leaf if child.grain_units != 1 => {
                    return Err(HolonError::InvalidDecomposition);
                }
                Decomposition::Latent if child.grain_units == 1 => {
                    return Err(HolonError::InvalidDecomposition);
                }
                Decomposition::Expanded => return Err(HolonError::InvalidDecomposition),
                _ => {}
            }
            composed = composed
                .checked_combine(child.gross)
                .ok_or(HolonError::GrossStateDoesNotCompose)?;
        }
        if composed != parent_record.gross {
            return Err(HolonError::GrossStateDoesNotCompose);
        }

        self.holons
            .try_reserve(children.len())
            .map_err(|_| HolonError::Capacity)?;
        self.whole
            .try_reserve(additional_whole)
            .map_err(|_| HolonError::Capacity)?;
        for child in children {
            let whole_offset = self.whole.len() as u32;
            self.whole.extend_from_slice(child.whole);
            self.holons.push(RuntimeHolon {
                gross: child.gross,
                parent: child.parent,
                grain_units: child.grain_units,
                whole_offset,
                whole_len: child.whole.len() as u32,
                channels: child.channels,
                depth: child.depth,
                decomposition: child.decomposition,
                boundary: u8::from(child.boundary),
            });
        }
        self.holons[parent].decomposition = Decomposition::Expanded;
        // The child index for the new children IS the returned contiguous range: the
        // parent was Latent, so it had no children before, and the chain is the range
        // in ascending id order.
        self.first_child
            .resize(final_node_len as usize, NO_RUNTIME_HOLON);
        self.next_sibling
            .resize(final_node_len as usize, NO_RUNTIME_HOLON);
        self.first_child[parent] = start;
        for id in start..final_node_len - 1 {
            self.next_sibling[id as usize] = id + 1;
        }
        debug_assert_eq!(self.holons.len(), final_node_len as usize);
        debug_assert_eq!(self.whole.len(), final_whole_len as usize);
        debug_assert!(self.validate().is_ok());
        debug_assert!(self.child_index_matches_headers());
        Ok(start..final_node_len)
    }

    pub fn is_descendant_or_self(&self, holon: usize, ancestor: usize) -> bool {
        if holon >= self.len() || ancestor >= self.len() {
            return false;
        }
        let mut current = holon;
        let mut remaining = self.len() + 1;
        while remaining > 0 {
            if current == ancestor {
                return true;
            }
            let parent = self.holons[current].parent;
            if parent == NO_RUNTIME_HOLON || parent as usize >= self.len() {
                return false;
            }
            current = parent as usize;
            remaining -= 1;
        }
        false
    }

    pub fn validate(&self) -> Result<(), HolonError> {
        if self.holons.is_empty() {
            return Err(HolonError::Empty);
        }
        if self.holons.len() > u32::MAX as usize {
            return Err(HolonError::Capacity);
        }
        let root = self.root as usize;
        if root >= self.len() || self.holons[root].parent != NO_RUNTIME_HOLON {
            return Err(HolonError::InvalidRoot);
        }

        let mut roots = 0;
        let mut child_counts = vec![0_u32; self.len()];
        let mut composed = vec![GrossState::ZERO; self.len()];
        for (i, holon) in self.holons.iter().copied().enumerate() {
            let whole_start = holon.whole_offset as usize;
            let whole_end = whole_start
                .checked_add(holon.whole_len as usize)
                .ok_or(HolonError::Capacity)?;
            if whole_end > self.whole.len() {
                return Err(HolonError::Capacity);
            }
            if holon.grain_units == 0 {
                return Err(HolonError::InvalidGrain);
            }
            if holon.parent == NO_RUNTIME_HOLON {
                roots += 1;
                if i != root || holon.depth != 0 {
                    return Err(HolonError::InvalidRoot);
                }
            } else {
                let parent_index = holon.parent as usize;
                if parent_index >= self.len() || parent_index == i {
                    return Err(HolonError::InvalidParent);
                }
                let parent = self.holons[parent_index];
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
                child_counts[parent_index] = child_counts[parent_index]
                    .checked_add(1)
                    .ok_or(HolonError::Capacity)?;
                composed[parent_index] = composed[parent_index]
                    .checked_combine(holon.gross)
                    .ok_or(HolonError::GrossStateDoesNotCompose)?;
            }
        }

        for (i, holon) in self.holons.iter().copied().enumerate() {
            let child_count = child_counts[i];
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

            if holon.decomposition == Decomposition::Expanded && composed[i] != holon.gross {
                return Err(HolonError::GrossStateDoesNotCompose);
            }
        }
        if roots != 1 {
            return Err(HolonError::MultipleRoots);
        }
        Ok(())
    }
}

/// Compact runtime-sized active frontier. One allocation stores 64 holons per word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFrontier {
    bits: Vec<u64>,
    len: usize,
}

struct ActiveIndices<'a> {
    bits: &'a [u64],
    next_word: usize,
    current_word: u64,
    current_base: usize,
    len: usize,
}

impl Iterator for ActiveIndices<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.current_word != 0 {
                let bit = self.current_word.trailing_zeros() as usize;
                self.current_word &= self.current_word - 1;
                let holon = self.current_base + bit;
                if holon < self.len {
                    return Some(holon);
                }
            } else if self.next_word < self.bits.len() {
                self.current_base = self.next_word * 64;
                self.current_word = self.bits[self.next_word];
                self.next_word += 1;
            } else {
                return None;
            }
        }
    }
}

impl RuntimeFrontier {
    pub fn root(arena: &RuntimeArena) -> Self {
        let mut frontier = Self {
            bits: vec![0; arena.len().div_ceil(64)],
            len: arena.len(),
        };
        frontier.set(arena.root as usize, true);
        frontier
    }

    /// Reuse this bitset for another certification. No allocation occurs when the
    /// arena length is unchanged; a larger or smaller scene resizes the workspace.
    pub fn reset_root(&mut self, arena: &RuntimeArena) {
        self.bits.resize(arena.len().div_ceil(64), 0);
        self.bits.fill(0);
        self.len = arena.len();
        self.set(arena.root as usize, true);
    }

    /// Deliberately private: a raw single-bit write can silently violate the
    /// invariant every certificate relies on — the active set exactly covers the
    /// root, non-overlapping. The public frontier moves are [`Self::refine`] (down),
    /// [`Self::coarsen`] (up), and [`Self::grow_to`] (track an append-only arena),
    /// each `O(children)` and invariant-preserving; every valid frontier is reachable
    /// from [`Self::root`] through them, so external certifiers need no raw access.
    fn set(&mut self, holon: usize, active: bool) {
        if holon >= self.len {
            return;
        }
        let mask = 1_u64 << (holon % 64);
        if active {
            self.bits[holon / 64] |= mask;
        } else {
            self.bits[holon / 64] &= !mask;
        }
    }

    pub fn is_active(&self, holon: usize) -> bool {
        holon < self.len && self.bits[holon / 64] & (1_u64 << (holon % 64)) != 0
    }

    pub fn active_count(&self) -> usize {
        self.bits
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum()
    }

    pub fn active_indices(&self) -> impl Iterator<Item = usize> + '_ {
        ActiveIndices {
            bits: &self.bits,
            next_word: 0,
            current_word: 0,
            current_base: 0,
            len: self.len,
        }
    }

    pub fn finest_grain(&self, arena: &RuntimeArena) -> u32 {
        self.active_indices()
            .map(|holon| arena.holons[holon].grain_units)
            .min()
            .unwrap_or(u32::MAX)
    }

    pub fn represented_grain(&self, arena: &RuntimeArena, holon: usize) -> u32 {
        let mut grain = 0;
        for i in self.active_indices() {
            if arena.is_descendant_or_self(holon, i)
                || (arena.is_descendant_or_self(i, holon) && arena.holons[i].is_boundary())
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

    pub fn refine(&mut self, arena: &RuntimeArena, holon: usize) -> Result<(), HolonError> {
        if holon >= arena.len() || !self.is_active(holon) {
            return Err(HolonError::FrontierDoesNotCoverRoot);
        }
        if arena.holons[holon].decomposition != Decomposition::Expanded {
            return Err(HolonError::InvalidDecomposition);
        }
        self.activate_children(arena, holon);
        debug_assert!(self.validate(arena).is_ok());
        Ok(())
    }

    /// The frontier move refine commits: swap `holon` for its children via the
    /// arena's child index, `O(children)`. Split out (without the validity
    /// debug-assert) so the mutation tests can prove a corrupted child index is
    /// caught by [`Self::validate`].
    fn activate_children(&mut self, arena: &RuntimeArena, holon: usize) {
        self.set(holon, false);
        for child in arena.children(holon) {
            self.set(child, true);
        }
    }

    /// Inverse of [`Self::refine`]: swap the children of `holon` — every one of which
    /// must currently be active — for `holon` itself. `O(children)`.
    ///
    /// Invariant-preserving by the same argument that makes refine sound: an
    /// [`Decomposition::Expanded`] holon's children compose exactly to it, so the
    /// exact cover of the root is unchanged. Requiring ALL children active is what
    /// rules out coarsening over a frontier that is deeper on one branch — that
    /// frontier must be coarsened bottom-up.
    pub fn coarsen(&mut self, arena: &RuntimeArena, holon: usize) -> Result<(), HolonError> {
        if holon >= arena.len() || self.is_active(holon) {
            return Err(HolonError::FrontierDoesNotCoverRoot);
        }
        if arena.holons[holon].decomposition != Decomposition::Expanded {
            return Err(HolonError::InvalidDecomposition);
        }
        for child in arena.children(holon) {
            if !self.is_active(child) {
                return Err(HolonError::FrontierDoesNotCoverRoot);
            }
        }
        for child in arena.children(holon) {
            self.set(child, false);
        }
        self.set(holon, true);
        debug_assert!(self.validate(arena).is_ok());
        Ok(())
    }

    /// Track an arena that has grown by materialization WITHOUT resetting to the
    /// root: newly appended holons start inactive, every current active bit is kept,
    /// and the frontier stays a valid exact cover (materialization never changes the
    /// root's gross state). This is the move a persistent external certifier needs
    /// between materializations; [`Self::reset_root`] remains the start-over form.
    pub fn grow_to(&mut self, arena: &RuntimeArena) -> Result<(), HolonError> {
        if arena.len() < self.len {
            return Err(HolonError::FrontierDoesNotCoverRoot);
        }
        self.bits.resize(arena.len().div_ceil(64), 0);
        self.len = arena.len();
        debug_assert!(self.validate(arena).is_ok());
        Ok(())
    }

    pub fn validate(&self, arena: &RuntimeArena) -> Result<(), HolonError> {
        if self.len != arena.len() {
            return Err(HolonError::FrontierDoesNotCoverRoot);
        }
        let mut gross = GrossState::ZERO;
        for i in self.active_indices() {
            for j in self.active_indices() {
                if i != j && arena.is_descendant_or_self(i, j) {
                    return Err(HolonError::FrontierDoesNotCoverRoot);
                }
            }
            gross = gross
                .checked_combine(arena.holons[i].gross)
                .ok_or(HolonError::FrontierDoesNotCoverRoot)?;
        }
        if gross != arena.holons[arena.root as usize].gross {
            return Err(HolonError::FrontierDoesNotCoverRoot);
        }
        Ok(())
    }
}

/// Realization-specific mathematics over a runtime-sized holarchy.
pub trait RuntimeBoundaryModel<const O: usize> {
    fn evaluate(&mut self, arena: &RuntimeArena, frontier: &RuntimeFrontier) -> Evaluation<O>;

    fn refinement_priority(
        &self,
        arena: &RuntimeArena,
        frontier: &RuntimeFrontier,
        holon: usize,
    ) -> f64;
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeResolutionCertificate<const O: usize> {
    pub status: CertificationStatus,
    pub frontier: RuntimeFrontier,
    pub observables: [f64; O],
    pub macro_error_bound: f64,
    pub conservation_residual: f64,
    pub evaluations: usize,
}

impl<const O: usize> RuntimeResolutionCertificate<O> {
    pub const fn passed(&self) -> bool {
        matches!(self.status, CertificationStatus::Certified)
    }
}

/// Certificate fields returned by the zero-allocation workspace API. The caller-owned
/// [`RuntimeFrontier`] is left at the certified (or finest available) resolution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeCertification<const O: usize> {
    pub status: CertificationStatus,
    pub observables: [f64; O],
    pub macro_error_bound: f64,
    pub conservation_residual: f64,
    pub evaluations: usize,
}

impl<const O: usize> RuntimeCertification<O> {
    pub const fn passed(&self) -> bool {
        matches!(self.status, CertificationStatus::Certified)
    }
}

/// Select the coarsest runtime frontier certified by the supplied realization.
/// Allocation occurs only when the frontier bitset is created.
pub fn certify_runtime<const O: usize>(
    arena: &RuntimeArena,
    model: &mut impl RuntimeBoundaryModel<O>,
    macro_tolerance: f64,
    conservation_tolerance: f64,
) -> RuntimeResolutionCertificate<O> {
    let mut frontier = RuntimeFrontier::root(arena);
    let certificate = certify_runtime_in(
        arena,
        model,
        &mut frontier,
        macro_tolerance,
        conservation_tolerance,
    );
    RuntimeResolutionCertificate {
        status: certificate.status,
        frontier,
        observables: certificate.observables,
        macro_error_bound: certificate.macro_error_bound,
        conservation_residual: certificate.conservation_residual,
        evaluations: certificate.evaluations,
    }
}

/// Workspace form of [`certify_runtime`]. Reusing `frontier` for the same arena avoids
/// all allocation in subsequent certifications.
pub fn certify_runtime_in<const O: usize>(
    arena: &RuntimeArena,
    model: &mut impl RuntimeBoundaryModel<O>,
    frontier: &mut RuntimeFrontier,
    macro_tolerance: f64,
    conservation_tolerance: f64,
) -> RuntimeCertification<O> {
    assert!(macro_tolerance >= 0.0 && conservation_tolerance >= 0.0);
    frontier.reset_root(arena);
    let mut evaluations = 0;

    loop {
        let evaluation = model.evaluate(arena, frontier);
        evaluations += 1;
        if evaluation.macro_error_bound <= macro_tolerance
            && evaluation.conservation_residual <= conservation_tolerance
        {
            return RuntimeCertification {
                status: CertificationStatus::Certified,
                observables: evaluation.observables,
                macro_error_bound: evaluation.macro_error_bound,
                conservation_residual: evaluation.conservation_residual,
                evaluations,
            };
        }

        let mut candidate = None;
        let mut priority = 0.0;
        let mut boundary_at_floor = false;
        for holon in frontier.active_indices() {
            let record = arena.holons[holon];
            if !record.is_boundary() {
                continue;
            }
            if record.grain_units == 1 {
                boundary_at_floor = true;
            }
            if record.decomposition != Decomposition::Expanded {
                continue;
            }
            let score = model.refinement_priority(arena, frontier, holon);
            if score > priority {
                priority = score;
                candidate = Some(holon);
            }
        }

        let Some(candidate) = candidate else {
            return RuntimeCertification {
                status: if boundary_at_floor {
                    CertificationStatus::GrainFloor
                } else {
                    CertificationStatus::RefinementUnavailable
                },
                observables: evaluation.observables,
                macro_error_bound: evaluation.macro_error_bound,
                conservation_residual: evaluation.conservation_residual,
                evaluations,
            };
        };
        frontier
            .refine(arena, candidate)
            .expect("validated expanded holon must refine into a valid frontier");
    }
}

/// Host-supplied procedural decomposition. Implementations normally call
/// [`RuntimeArena::materialize`] with children derived from the holon's definition.
/// Returning `Ok(false)` declines refinement and leaves the current certificate final.
pub trait RuntimeMaterializer {
    fn materialize(&mut self, arena: &mut RuntimeArena, holon: usize) -> Result<bool, HolonError>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdaptiveRuntimeCertificate<const O: usize> {
    pub certificate: RuntimeResolutionCertificate<O>,
    pub materializations: usize,
}

/// Certify while materializing only unresolved boundary branches requested by the model.
///
/// Each materialization restarts selection at the encounter root because the newly
/// resident children may change the globally coarsest valid frontier. Existing holon IDs
/// remain stable: runtime materialization is append-only.
pub fn certify_runtime_adaptive<const O: usize>(
    arena: &mut RuntimeArena,
    model: &mut impl RuntimeBoundaryModel<O>,
    materializer: &mut impl RuntimeMaterializer,
    macro_tolerance: f64,
    conservation_tolerance: f64,
) -> Result<AdaptiveRuntimeCertificate<O>, HolonError> {
    let mut materializations = 0;
    loop {
        let certificate = certify_runtime(arena, model, macro_tolerance, conservation_tolerance);
        if certificate.status != CertificationStatus::RefinementUnavailable {
            return Ok(AdaptiveRuntimeCertificate {
                certificate,
                materializations,
            });
        }

        let mut candidate = None;
        let mut priority = 0.0;
        for holon in certificate.frontier.active_indices() {
            let record = arena.holons[holon];
            if !record.is_boundary() || record.decomposition != Decomposition::Latent {
                continue;
            }
            let score = model.refinement_priority(arena, &certificate.frontier, holon);
            if score > priority {
                priority = score;
                candidate = Some(holon);
            }
        }
        let Some(candidate) = candidate else {
            return Ok(AdaptiveRuntimeCertificate {
                certificate,
                materializations,
            });
        };

        let previous_len = arena.len();
        if !materializer.materialize(arena, candidate)? {
            return Ok(AdaptiveRuntimeCertificate {
                certificate,
                materializations,
            });
        }
        if arena.len() <= previous_len
            || arena.holons[candidate].decomposition != Decomposition::Expanded
        {
            return Err(HolonError::InvalidDecomposition);
        }
        materializations += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gross(elements: u64) -> GrossState {
        GrossState::aggregate(elements, 2 * elements, [elements as i64, 0])
    }

    fn arena() -> RuntimeArena {
        let channels = Channels::REG_PLUS
            .union(Channels::MECHANICAL)
            .union(Channels::NARRATIVE);
        let root_whole = [1.0, 0.5, -0.25];
        let right_whole = [3.0, 5.0];
        let specs = [
            RuntimeHolonSpec {
                parent: NO_RUNTIME_HOLON,
                depth: 0,
                grain_units: 4,
                gross: gross(8),
                whole: &root_whole,
                channels,
                boundary: true,
                decomposition: Decomposition::Expanded,
            },
            RuntimeHolonSpec {
                parent: 0,
                depth: 1,
                grain_units: 2,
                gross: gross(4),
                whole: &[],
                channels,
                boundary: true,
                decomposition: Decomposition::Expanded,
            },
            RuntimeHolonSpec {
                parent: 0,
                depth: 1,
                grain_units: 2,
                gross: gross(4),
                whole: &right_whole,
                channels,
                boundary: false,
                decomposition: Decomposition::Latent,
            },
            RuntimeHolonSpec {
                parent: 1,
                depth: 2,
                grain_units: 1,
                gross: gross(2),
                whole: &[],
                channels,
                boundary: true,
                decomposition: Decomposition::Leaf,
            },
            RuntimeHolonSpec {
                parent: 1,
                depth: 2,
                grain_units: 1,
                gross: gross(2),
                whole: &[],
                channels,
                boundary: false,
                decomposition: Decomposition::Leaf,
            },
        ];
        RuntimeArena::from_specs(&specs, 0).unwrap()
    }

    #[test]
    fn variable_whole_state_is_flat_and_addressable() {
        let arena = arena();
        assert_eq!(arena.whole_scalars(), &[1.0, 0.5, -0.25, 3.0, 5.0]);
        assert_eq!(arena.whole_state(0), Some(&[1.0, 0.5, -0.25][..]));
        assert_eq!(arena.whole_state(1), Some(&[][..]));
        assert_eq!(arena.whole_state(2), Some(&[3.0, 5.0][..]));
    }

    #[test]
    fn frontier_bitset_iterates_across_word_boundaries() {
        let mut frontier = RuntimeFrontier {
            bits: vec![0; 3],
            len: 130,
        };
        for holon in [0, 63, 64, 127, 129] {
            frontier.set(holon, true);
        }
        assert_eq!(frontier.active_count(), 5);
        assert_eq!(
            frontier.active_indices().collect::<Vec<_>>(),
            vec![0, 63, 64, 127, 129]
        );
    }

    #[test]
    fn runtime_frontier_refines_without_changing_gross_state() {
        let arena = arena();
        let mut frontier = RuntimeFrontier::root(&arena);
        frontier.refine(&arena, 0).unwrap();
        frontier.refine(&arena, 1).unwrap();
        assert_eq!(frontier.active_indices().collect::<Vec<_>>(), vec![2, 3, 4]);
        assert_eq!(frontier.finest_grain(&arena), 1);
        frontier.validate(&arena).unwrap();
    }

    struct TestModel;

    impl RuntimeBoundaryModel<1> for TestModel {
        fn evaluate(&mut self, arena: &RuntimeArena, frontier: &RuntimeFrontier) -> Evaluation<1> {
            let grain = frontier.represented_grain(arena, 1) as f64;
            Evaluation {
                observables: [grain],
                macro_error_bound: grain * grain * 0.0002,
                conservation_residual: 0.0,
            }
        }

        fn refinement_priority(
            &self,
            arena: &RuntimeArena,
            _frontier: &RuntimeFrontier,
            holon: usize,
        ) -> f64 {
            arena.holons[holon].grain_units as f64
        }
    }

    #[test]
    fn runtime_selector_returns_coarsest_certified_frontier() {
        let arena = arena();
        let certificate = certify_runtime(&arena, &mut TestModel, 0.001, 1.0e-12);
        assert!(certificate.passed());
        assert_eq!(certificate.evaluations, 2);
        assert_eq!(
            certificate.frontier.active_indices().collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    fn latent_arena() -> RuntimeArena {
        let channels = Channels::REG_PLUS.union(Channels::MECHANICAL);
        let specs = [
            RuntimeHolonSpec {
                parent: NO_RUNTIME_HOLON,
                depth: 0,
                grain_units: 4,
                gross: gross(8),
                whole: &[],
                channels,
                boundary: true,
                decomposition: Decomposition::Expanded,
            },
            RuntimeHolonSpec {
                parent: 0,
                depth: 1,
                grain_units: 2,
                gross: gross(4),
                whole: &[],
                channels,
                boundary: true,
                decomposition: Decomposition::Latent,
            },
            RuntimeHolonSpec {
                parent: 0,
                depth: 1,
                grain_units: 2,
                gross: gross(4),
                whole: &[],
                channels,
                boundary: false,
                decomposition: Decomposition::Latent,
            },
        ];
        RuntimeArena::from_specs(&specs, 0).unwrap()
    }

    #[test]
    fn failed_materialization_is_transactional() {
        let mut arena = latent_arena();
        let before = arena.clone();
        let channels = Channels::REG_PLUS.union(Channels::MECHANICAL);
        let invalid = [RuntimeHolonSpec {
            parent: 1,
            depth: 2,
            grain_units: 1,
            gross: gross(3),
            whole: &[],
            channels,
            boundary: true,
            decomposition: Decomposition::Leaf,
        }];
        assert_eq!(
            arena.materialize(1, &invalid),
            Err(HolonError::GrossStateDoesNotCompose)
        );
        assert_eq!(arena, before);
    }

    struct SplitBoundary(bool);

    impl RuntimeMaterializer for SplitBoundary {
        fn materialize(
            &mut self,
            arena: &mut RuntimeArena,
            holon: usize,
        ) -> Result<bool, HolonError> {
            if self.0 || holon != 1 {
                return Ok(false);
            }
            let channels = Channels::REG_PLUS.union(Channels::MECHANICAL);
            let children = [
                RuntimeHolonSpec {
                    parent: holon as u32,
                    depth: 2,
                    grain_units: 1,
                    gross: gross(2),
                    whole: &[],
                    channels,
                    boundary: true,
                    decomposition: Decomposition::Leaf,
                },
                RuntimeHolonSpec {
                    parent: holon as u32,
                    depth: 2,
                    grain_units: 1,
                    gross: gross(2),
                    whole: &[],
                    channels,
                    boundary: false,
                    decomposition: Decomposition::Leaf,
                },
            ];
            arena.materialize(holon, &children)?;
            self.0 = true;
            Ok(true)
        }
    }

    fn scanned_children(arena: &RuntimeArena, parent: usize) -> Vec<usize> {
        (0..arena.len())
            .filter(|i| arena.holons()[*i].parent as usize == parent)
            .collect()
    }

    #[test]
    fn child_index_matches_a_header_scan_on_both_construction_paths() {
        // Builder-built arena.
        let arena = arena();
        for parent in 0..arena.len() {
            assert_eq!(
                arena.children(parent).collect::<Vec<_>>(),
                scanned_children(&arena, parent)
            );
        }
        // Materialized arena: the index is the returned range.
        let mut latent = latent_arena();
        let channels = Channels::REG_PLUS.union(Channels::MECHANICAL);
        let children = [
            RuntimeHolonSpec {
                parent: 1,
                depth: 2,
                grain_units: 1,
                gross: gross(2),
                whole: &[],
                channels,
                boundary: true,
                decomposition: Decomposition::Leaf,
            },
            RuntimeHolonSpec {
                parent: 1,
                depth: 2,
                grain_units: 1,
                gross: gross(2),
                whole: &[],
                channels,
                boundary: false,
                decomposition: Decomposition::Leaf,
            },
        ];
        let range = latent.materialize(1, &children).unwrap();
        assert_eq!(
            latent.children(1).collect::<Vec<_>>(),
            (range.start as usize..range.end as usize).collect::<Vec<_>>()
        );
        for parent in 0..latent.len() {
            assert_eq!(
                latent.children(parent).collect::<Vec<_>>(),
                scanned_children(&latent, parent)
            );
        }
    }

    #[test]
    fn refine_activates_exactly_the_scanned_children() {
        // No-behavior-change fence for the O(children) refine: the frontier after
        // refining must equal the historical scan-based activation, bit for bit.
        let arena = arena();
        let mut frontier = RuntimeFrontier::root(&arena);
        frontier.refine(&arena, 0).unwrap();
        let mut expected = RuntimeFrontier::root(&arena);
        expected.set(0, false);
        for child in scanned_children(&arena, 0) {
            expected.set(child, true);
        }
        assert_eq!(frontier, expected);
    }

    #[test]
    fn coarsen_is_the_exact_inverse_of_refine() {
        let arena = arena();
        let mut frontier = RuntimeFrontier::root(&arena);
        frontier.refine(&arena, 0).unwrap();
        frontier.refine(&arena, 1).unwrap();
        let refined = frontier.clone();

        // A deeper branch blocks coarsening at the top: holon 1's children are
        // active, holon 0's are not all active.
        assert_eq!(
            frontier.coarsen(&arena, 0),
            Err(HolonError::FrontierDoesNotCoverRoot)
        );
        assert_eq!(frontier, refined);

        // Bottom-up coarsening returns exactly the root frontier.
        frontier.coarsen(&arena, 1).unwrap();
        frontier.coarsen(&arena, 0).unwrap();
        assert_eq!(frontier, RuntimeFrontier::root(&arena));

        // A non-expanded holon cannot be coarsened onto.
        let mut fresh = RuntimeFrontier::root(&arena);
        fresh.refine(&arena, 0).unwrap();
        assert_eq!(
            fresh.coarsen(&arena, 3),
            Err(HolonError::InvalidDecomposition)
        );
    }

    #[test]
    fn grow_to_keeps_a_persistent_frontier_across_materialization() {
        let mut arena = latent_arena();
        let mut frontier = RuntimeFrontier::root(&arena);
        frontier.refine(&arena, 0).unwrap();
        frontier.validate(&arena).unwrap();

        let channels = Channels::REG_PLUS.union(Channels::MECHANICAL);
        let children = [
            RuntimeHolonSpec {
                parent: 1,
                depth: 2,
                grain_units: 1,
                gross: gross(2),
                whole: &[],
                channels,
                boundary: true,
                decomposition: Decomposition::Leaf,
            },
            RuntimeHolonSpec {
                parent: 1,
                depth: 2,
                grain_units: 1,
                gross: gross(2),
                whole: &[],
                channels,
                boundary: false,
                decomposition: Decomposition::Leaf,
            },
        ];
        arena.materialize(1, &children).unwrap();

        // Without grow_to the stale frontier no longer validates (length mismatch);
        // with it, the active set survives and the new children are refinable.
        assert!(frontier.validate(&arena).is_err());
        frontier.grow_to(&arena).unwrap();
        frontier.validate(&arena).unwrap();
        assert_eq!(frontier.active_indices().collect::<Vec<_>>(), vec![1, 2]);
        frontier.refine(&arena, 1).unwrap();
        assert_eq!(frontier.active_indices().collect::<Vec<_>>(), vec![2, 3, 4]);
    }

    #[test]
    fn corrupted_child_index_is_caught_by_frontier_validation() {
        let mut arena = latent_arena();
        let channels = Channels::REG_PLUS.union(Channels::MECHANICAL);
        let children = [
            RuntimeHolonSpec {
                parent: 1,
                depth: 2,
                grain_units: 1,
                gross: gross(2),
                whole: &[],
                channels,
                boundary: true,
                decomposition: Decomposition::Leaf,
            },
            RuntimeHolonSpec {
                parent: 1,
                depth: 2,
                grain_units: 1,
                gross: gross(2),
                whole: &[],
                channels,
                boundary: false,
                decomposition: Decomposition::Leaf,
            },
        ];
        let range = arena.materialize(1, &children).unwrap();
        let first = range.start as usize;

        // MUTATION 1: drop the second child from the chain. The frontier the walk
        // produces under-covers the parent's gross state and must fail validation.
        let pristine_sibling = arena.next_sibling[first];
        arena.next_sibling[first] = NO_RUNTIME_HOLON;
        assert!(!arena.child_index_matches_headers());
        let mut frontier = RuntimeFrontier::root(&arena);
        frontier.refine(&arena, 0).unwrap();
        frontier.activate_children(&arena, 1);
        assert_eq!(
            frontier.validate(&arena),
            Err(HolonError::FrontierDoesNotCoverRoot)
        );

        // MUTATION 2: cross-link to a non-child (the sibling subtree). The produced
        // frontier overlaps/miscomposes and must fail validation.
        arena.next_sibling[first] = 2;
        assert!(!arena.child_index_matches_headers());
        let mut frontier = RuntimeFrontier::root(&arena);
        frontier.refine(&arena, 0).unwrap();
        frontier.activate_children(&arena, 1);
        assert!(frontier.validate(&arena).is_err());

        // Restored index passes both the independent checker and validation.
        arena.next_sibling[first] = pristine_sibling;
        assert!(arena.child_index_matches_headers());
        let mut frontier = RuntimeFrontier::root(&arena);
        frontier.refine(&arena, 0).unwrap();
        frontier.refine(&arena, 1).unwrap();
        frontier.validate(&arena).unwrap();
    }

    #[test]
    fn adaptive_certification_materializes_only_the_unresolved_boundary() {
        let mut arena = latent_arena();
        let result = certify_runtime_adaptive(
            &mut arena,
            &mut TestModel,
            &mut SplitBoundary(false),
            0.00025,
            1.0e-12,
        )
        .unwrap();
        assert!(result.certificate.passed(), "{result:?}");
        assert_eq!(result.materializations, 1);
        assert_eq!(arena.len(), 5);
        assert_eq!(
            arena.holon(1).unwrap().decomposition,
            Decomposition::Expanded
        );
        assert_eq!(arena.holon(2).unwrap().decomposition, Decomposition::Latent);
        assert_eq!(
            result
                .certificate
                .frontier
                .active_indices()
                .collect::<Vec<_>>(),
            vec![2, 3, 4]
        );
        assert_eq!(result.certificate.frontier.represented_grain(&arena, 1), 1);
    }
}
