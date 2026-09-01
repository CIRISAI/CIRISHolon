//! Shipped three-body surfaces, and the door they come through.
//!
//! The sandbox generates the H3 surface in the browser — nine determinants a node, an
//! affordable load. It cannot generate the heteronuclear ones: (H,H,Cl) is 605
//! determinants a node and (Cl,Cl,Cl) is 9,477, over 14,157 nodes. Those are computed on
//! the mesh and SHIPPED, which is why this module exists and why it is a door rather than
//! a reader.
//!
//! # The standard this meets, and where it came from
//!
//! `bank.rs`'s pair door was retrofitted on 2026-08-30 after a shipped table carried
//! `converged: false` through three commits and every gate passed it — not because the
//! gate was lax about convergence but because nothing on this side read convergence at
//! any magnitude. The correction was to read the NUMBER rather than the producer's flag.
//! This door is built to that standard from birth rather than retrofitted to it, and the
//! schema it implements (`SATURATION3/trimer-table/v1`, written by `saturation3-mesh`)
//! goes one step further: it carries no top-level `converged` field at all, because for
//! these systems every solve exits STAGNATED at the f64 expansion floor and a boolean
//! derived from that threshold would be false everywhere and mean nothing. So a
//! `converged` field ARRIVING is itself a refusal here — see
//! [`TrimerRefusal::ConvergedFieldPresent`].
//!
//! # What a door can and cannot check
//!
//! It can check that a claim is present, that it is weighed against the feature it
//! describes, and that the artifact's own certificate reproduces. It cannot check that
//! the numbers are right; that is `tests/` and the referee's job. The line matters
//! because a gate that reports success on a shape it cannot check is worse than no gate —
//! it is the pair gate's own history, and the reason this one refuses an unfamiliar grid
//! rule instead of interpolating on it.


/// Surfaces the bank will hold. SATURATION-3 ships four; the bound is stated rather than
/// grown on demand because a fifth arriving should be a decision, not an allocation.
pub const MAX_TRIMER_SURFACES: usize = 4;

/// The largest uncertainty a shipped surface may declare and still be loaded, hartree.
///
/// DERIVED, and deliberately the SAME constant the pair door derives its bound from:
/// `holon_chem::pair::WELL_MIN_DEPTH`, the depth below which the schema on the other side
/// declines to call a dip a well. The three-body term enters the same total energy, the
/// same ledger and the same drift bound as the pair term, so "too small for this app to
/// read" has to mean one thing in both doors. A surface whose declared uncertainty
/// reaches it cannot resolve the shallowest feature this schema recognises anywhere, and
/// the force loop would integrate it anyway.
///
/// Pinned to the pair constant on purpose. If the two are ever unpinned the trimer door's
/// meaning silently drifts from the pair door's, so `plant_resolution_pinning` asserts the
/// identity and the change gets announced rather than discovered.
pub const RESOLVABLE_TRIMER_UNCERTAINTY: f64 = holon_chem::pair::WELL_MIN_DEPTH;

/// How the artifact says its grid is spaced.
///
/// # Why a rule and not a pair of corners
///
/// Spans plus counts do NOT determine node positions: they say nothing about the interior
/// spacing. This build's H3 grid is not uniform in `r` — `trimer::r_of_tau` places nodes
/// through a stretch with `STRETCH_A = 2.0` — so a consumer handed only corners and
/// counts, guessing uniform, would interpolate a stretched table on uniform axes and be
/// smoothly, plausibly wrong everywhere except the boundary.
///
/// That is the failure `load_water_table` already refuses by construction, and it is why
/// this enum exists rather than three `f64` pairs. An unrecognised rule is REFUSED. The
/// question of which rules the shipped artifacts will declare is open with
/// `saturation3-mesh` as of 2026-08-30; until one is agreed, the only admissible rule is
/// this build's own, and a table that does not declare it does not load.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AxisRule {
    /// Nothing was declared. Never admissible: an undeclared spacing cannot be checked,
    /// and "unstated" must not read as "the same as ours".
    Undeclared,
    /// Equally spaced between the declared endpoints. What `holon-tables`' emitter
    /// actually produces (`TableGrid::geometry`), confirmed against a real artifact.
    UniformLinear,
    /// This build's H3 spacing: `trimer::r_of_tau` with `STRETCH_A`, and `trimer::node_c`
    /// on `u`. NOT what the shipped artifacts use — recorded because the in-browser
    /// surface is on it, and because assuming the two agreed is the mistake this whole
    /// enum exists to prevent.
    TauStretchH3,
    // THIS ENUM NAMES SPACING, AND ONLY SPACING. Do not add a traversal variant here.
    //
    // `de4-table`'s grid vocabulary (54821bd) keeps the two strictly apart, and the
    // reason is not tidiness: traversal is a property of how a REGION is walked, spacing
    // is a property of WHERE the nodes are, and a field that means both cannot answer
    // either question. The artifact carries its traversal separately as `warm_policy`
    // ("CanonicalChain" on the emitted H3), and this door deliberately does not read it —
    // an unread field being a decision here rather than an oversight, because nothing the
    // door decides depends on the order the producer visited its nodes in.
    //
    // That independence is now measured rather than assumed. `de4-table` falsified the
    // sum-parity serpentine's documented adjacency invariant for even interior axis
    // extents, and production `[2, 2, 2]` — the region shape of the very artifact in
    // `tests/data/` — is one of the failing cases. A consumer that had reconstructed node
    // positions from a traversal rule would have been wrong on the first real table it
    // met. This door reads the shipped COORDINATES instead, which is why the finding
    // costs it nothing; that was luck as much as design when the choice was made, and it
    // is written down here so the next change keeps it on purpose.
}

impl AxisRule {
    /// Does a coordinate array actually look like this rule?
    ///
    /// The artifact declares a rule AND ships the coordinates, and says the coordinates
    /// win where they disagree. That makes the declared name checkable rather than
    /// merely informative, and a disagreement is a REFUSAL: one of the two is wrong and
    /// nothing on this side can tell which. It is the same principle that keeps
    /// `claimed_exact` apart from `route` in the pair door — a file's self-assessment is
    /// exactly what a file can get wrong.
    pub fn matches(self, nodes: &[f64]) -> bool {
        match self {
            AxisRule::Undeclared => false,
            AxisRule::TauStretchH3 => true, // not cross-checkable here; see `admit`.
            AxisRule::UniformLinear => {
                if nodes.len() < 3 {
                    // Two points are uniform by construction; there is nothing to check.
                    return true;
                }
                let step = nodes[1] - nodes[0];
                // A relative tolerance, because the endpoints are decimal literals and
                // the interior is arithmetic on them: exact equality would refuse a
                // correctly-emitted grid for its last bit.
                let tol = 1e-9 * step.abs().max(1.0);
                nodes.windows(2).all(|w| ((w[1] - w[0]) - step).abs() <= tol)
            }
        }
    }
}

/// What the artifact says about the seams in its domain.
///
/// A seam is a locus where `dE3` stops being smooth — a reactive channel. SATURATION-3's
/// own results record the consequence: refinement "cannot beat a corner", so grid
/// refinement past the seam buys nothing and the interpolation error near it is set by the
/// seam rather than by the spacing. A table that neither locates its seams nor bounds the
/// error it is accepting has an interpolation error near them that nobody has weighed —
/// and the force loop would integrate it anyway, which is the same shape as an unweighed
/// uncertainty.
///
/// So the artifact must say one or the other. Both are acceptable answers; silence is not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SeamRecord {
    /// Neither a locus nor a floor. Refused.
    Absent,
    /// The seams were located and the artifact carries where they run.
    Locus,
    /// The seams were NOT located, and the artifact declares the interpolation error it
    /// is accepting near them instead. An honest answer, and the campaign's own position
    /// while the locus is still owed.
    AcceptedFloor,
}

/// Why the trimer door refused a surface. One variant per reason, for the reason
/// `bank::Refusal` gives: a single boolean tells a user something is wrong and not which
/// thing. Ordered as the schema's own refusal table orders them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrimerRefusal {
    /// The surface declares no solver route.
    RouteUndeclared,
    /// The grid rule is undeclared or is not one this build can reproduce. Refused rather
    /// than interpolated — see [`AxisRule`].
    GridRuleUnsupported,
    /// `grid.region` absent. The region shape fixes the warm chains and therefore the
    /// trailing bits, so it is part of the artifact's identity: without it two different
    /// surfaces could collide on one digest.
    RegionShapeMissing,
    /// `domain.cited_curves` is empty. A domain derived from pair curves that names none
    /// of them is a physics claim nobody can re-check.
    DomainUncited,
    /// No uncertainty, or a declared zero. An absent bound must never read as a perfect
    /// one.
    UncertaintyMissing,
    /// The declared uncertainty reaches [`RESOLVABLE_TRIMER_UNCERTAINTY`].
    UncertaintyExceedsResolution,
    /// The declared uncertainty is not smaller than the surface's own peak `|dE3|` — the
    /// feature it would describe. The three-body term is inside its own error bar.
    UncertaintyExceedsFeature,
    /// A top-level `converged` arrived. This schema has no such field on purpose; one
    /// appearing means somebody derived an outcome from a threshold.
    ConvergedFieldPresent,
    /// `voided.count > 0` with no nodes named. A VOID that is counted but not named has
    /// been averaged away.
    VoidsCountedNotNamed,
    /// Neither a seam locus nor an accepted-floor note. See [`SeamRecord`].
    SeamRecordMissing,
    /// No merge digest. The certificate is the point: without it a consumer trusts rather
    /// than verifies.
    DigestMissing,
    /// The node array is not filled, whatever the provenance says about it.
    SurfaceNotLoaded,
    /// An axis shipped no coordinates. Spans and counts do not determine spacing, so a
    /// surface without its coordinates cannot be interpolated at all.
    CoordinatesMissing,
    /// A coordinate array's length disagrees with the grid count it belongs to. One of
    /// the two is wrong and the artifact does not say which.
    CoordinateCountMismatch,
    /// A coordinate array is not strictly increasing. A non-monotone axis has no
    /// well-defined cell to locate a point in, and a search on it would silently return
    /// whichever bracket it happened to find first.
    CoordinatesNotMonotone,
    /// The declared axis rule and the shipped coordinates disagree. The artifact says the
    /// coordinates win, but a disagreement means one of its own statements is false and
    /// nothing here can tell which.
    AxisRuleContradictsCoordinates,
    /// The energy array's length is not `nx * ny * nu`.
    EnergyCountMismatch,
    /// A `u` node lies outside `[-1, 1]`. `u` is the COSINE of the apex angle, so a value
    /// past 1 is not a geometry at all — see [`TrimerRefusal::plain`] for the specific
    /// way this is expected to arrive.
    AngleCosineOutOfRange,
    /// An `x` or `y` node is not a positive length. Both are sides of a triangle measured
    /// from the apex, so zero or negative is not a degenerate geometry, it is not one.
    SideLengthNotPositive,
}

impl TrimerRefusal {
    pub fn plain(self) -> &'static str {
        match self {
            TrimerRefusal::RouteUndeclared => {
                "this surface does not say which solver produced it, so nothing can grade it"
            }
            TrimerRefusal::GridRuleUnsupported => {
                "this surface's grid spacing is undeclared or is not one this build can \
                 reproduce; interpolating it would be smoothly and plausibly wrong \
                 everywhere except the boundary, so it is refused instead"
            }
            TrimerRefusal::RegionShapeMissing => {
                "this surface declares no region shape; the shape fixes the warm chains and \
                 therefore the trailing bits, so without it two different surfaces could \
                 collide on one digest"
            }
            TrimerRefusal::DomainUncited => {
                "this surface's domain cites no pair curve; a domain derived from curves \
                 that names none of them is a claim nobody can re-check"
            }
            TrimerRefusal::UncertaintyMissing => {
                "this shipped surface declares no uncertainty; an absent bound must not \
                 read as a zero one"
            }
            TrimerRefusal::UncertaintyExceedsResolution => {
                "this surface's declared uncertainty is larger than the shallowest feature \
                 the schema recognises, so nothing it says is above its own noise"
            }
            TrimerRefusal::UncertaintyExceedsFeature => {
                "this surface's declared uncertainty is larger than the largest three-body \
                 term it found, so the whole surface is inside its own error bar"
            }
            TrimerRefusal::ConvergedFieldPresent => {
                "this surface declares a top-level `converged`; this schema has none, \
                 because every solve of these systems exits stagnated at the expansion \
                 floor and a boolean derived from that threshold would mean nothing"
            }
            TrimerRefusal::VoidsCountedNotNamed => {
                "this surface counts voided nodes but names none of them; a VOID that is \
                 counted and not named has been averaged away"
            }
            TrimerRefusal::SeamRecordMissing => {
                "this surface neither locates its seams nor declares the interpolation \
                 error it accepts near them, so the error where dE3 stops being smooth is \
                 unweighed and would be integrated anyway"
            }
            TrimerRefusal::DigestMissing => {
                "this surface ships no merge digest, so a consumer can only trust it"
            }
            TrimerRefusal::SurfaceNotLoaded => {
                "there are no node values in this surface; its provenance describes nothing"
            }
            TrimerRefusal::CoordinatesMissing => {
                "this surface ships no node coordinates on one or more axes; spans and \
                 counts do not determine spacing, so there is nothing here to interpolate \
                 on"
            }
            TrimerRefusal::CoordinateCountMismatch => {
                "a coordinate array's length disagrees with the grid count it belongs to, \
                 and the artifact does not say which of the two is right"
            }
            TrimerRefusal::CoordinatesNotMonotone => {
                "a coordinate axis is not strictly increasing, so a point has no \
                 well-defined cell and a bracket search would return whichever one it \
                 met first"
            }
            TrimerRefusal::AxisRuleContradictsCoordinates => {
                "this surface's declared axis rule and its own shipped coordinates \
                 disagree; the coordinates are authoritative, but a disagreement means \
                 one of the artifact's statements about itself is false"
            }
            TrimerRefusal::EnergyCountMismatch => {
                "the energy array is not nx*ny*nu long, so the grid it claims and the \
                 values it ships are not the same object"
            }
            TrimerRefusal::AngleCosineOutOfRange => {
                "a u node is outside [-1, 1]; u is the COSINE of the apex angle, and a \
                 value past 1 is most likely an axis parameterised as sqrt(1 - cos) \
                 handed over as if it were the cosine itself"
            }
            TrimerRefusal::SideLengthNotPositive => {
                "an x or y node is not a positive length, and both are triangle sides \
                 measured from the apex"
            }
        }
    }
}

/// What a shipped surface says about itself. Every field is a DECLARATION the artifact
/// makes; none is a fact this process observed, which is exactly why each one is weighed.
#[derive(Clone, Copy, Debug)]
pub struct TrimerProvenance {
    /// Reused from the pair door rather than restated: "which solver produced this" means
    /// the same thing for a surface as for a curve, and two enums would drift.
    pub route: crate::bank::Route,
    /// The three species, by atomic number, in the artifact's own order.
    pub z: [u8; 3],
    pub n_det: u64,
    /// Declared absolute uncertainty on the energy column, hartree.
    pub uncertainty_ha: f64,
    /// The surface's own largest `|dE3|` — the feature the uncertainty is weighed against,
    /// and the trimer analogue of the pair door's `well_depth_ha`.
    pub peak_ha: f64,
    pub axis_rule: AxisRule,
    /// `grid.region`. A zero on any axis means the artifact did not declare it.
    pub region: [u16; 3],
    /// How many pair curves the domain cites. Zero is a refusal.
    pub cited_curves: u32,
    /// Whether a top-level `converged` was present. Its PRESENCE is the defect.
    pub declares_converged: bool,
    pub void_count: u32,
    pub void_named: u32,
    pub seam: SeamRecord,
    /// The merge digest, as eight 32-bit words — the artifact ships a SHA-256 hex string,
    /// which is 256 bits and does not survive a trip through a `u64` or a JavaScript
    /// number. All-zero means absent.
    ///
    /// Checked for PRESENCE only. Reproducing it would mean recomputing a merge digest
    /// over index, energy, BOTH derivatives and status, and this loader takes neither the
    /// derivatives nor the status — so verification is owed and is named as owed rather
    /// than implied by carrying the field.
    pub digest: [u32; 8],
}

impl TrimerProvenance {
    /// A provenance that declares nothing. Every field here is chosen so that an artifact
    /// which simply fails to set it is REFUSED rather than admitted by default — the
    /// property `Route::Undeclared` has in the pair door, applied to every leg.
    pub const fn undeclared() -> Self {
        Self {
            route: crate::bank::Route::Undeclared,
            z: [0, 0, 0],
            n_det: 0,
            uncertainty_ha: 0.0,
            peak_ha: 0.0,
            axis_rule: AxisRule::Undeclared,
            region: [0, 0, 0],
            cited_curves: 0,
            declares_converged: false,
            void_count: 0,
            void_named: 0,
            seam: SeamRecord::Absent,
            digest: [0; 8],
        }
    }

    /// Is this a surface the in-browser generator could not have produced?
    ///
    /// The fence's whole question. H3 is generated in the browser and its being loaded
    /// from a file changes nothing a viewer should announce; a surface with any
    /// non-hydrogen centre is the successor the fence names.
    pub fn is_heteronuclear(&self) -> bool {
        self.z.iter().any(|&z| z != 1)
    }

    /// The door. Legs run in the schema's own order, so a surface with several faults is
    /// reported against the first one the schema lists rather than whichever happens to be
    /// checked first.
    ///
    /// `grid` is the surface's own shape and coordinates; it is passed in rather than held
    /// on the provenance because it is a fact about the ARRAYS, and the point of this
    /// method is to weigh what the artifact SAYS against what it SHIPPED.
    pub fn admit(&self, grid: &SurfaceGrid) -> Result<(), TrimerRefusal> {
        if self.route == crate::bank::Route::Undeclared {
            return Err(TrimerRefusal::RouteUndeclared);
        }
        if self.axis_rule == AxisRule::Undeclared {
            return Err(TrimerRefusal::GridRuleUnsupported);
        }
        // ---- what the artifact SHIPPED, before anything it says about it -------------
        //
        // These run before the declaration legs because a surface whose arrays do not
        // describe a grid cannot have its declarations weighed against anything. A door
        // that checked the paperwork first would report a missing uncertainty on an
        // artifact whose real fault is that it has no coordinates.
        if grid.x.is_empty() || grid.y.is_empty() || grid.u.is_empty() {
            return Err(TrimerRefusal::CoordinatesMissing);
        }
        if grid.x.len() != grid.nx || grid.y.len() != grid.ny || grid.u.len() != grid.nu {
            return Err(TrimerRefusal::CoordinateCountMismatch);
        }
        for axis in [&grid.x, &grid.y, &grid.u] {
            if !axis.iter().all(|v| v.is_finite()) || !axis.windows(2).all(|w| w[1] > w[0]) {
                return Err(TrimerRefusal::CoordinatesNotMonotone);
            }
        }
        // THE AXES MEAN SOMETHING, and the door checks that they could.
        //
        // `saturation3-mesh` stated the convention at 7dff58c: centres are
        // `[0,0,0]`, `[x,0,0]`, `[y*u, y*s, 0]` with `s = sqrt(1 - u^2)`. So species[0] is
        // the apex at the origin, `x` and `y` are the two sides measured from it, and `u`
        // is the COSINE of the angle between them. The third side is not stored; it is
        // `sqrt(x^2 + y^2 - 2*x*y*u)` by the law of cosines, which those centres satisfy
        // exactly.
        //
        // These two legs exist because of a live near-miss they found while writing that
        // down: a neighbouring lane parameterises the SAME axis as `c = sqrt(1 - cos)`
        // over [0.05, 1.4142]. Handed over in `c` and consumed as `u`, the grid runs past
        // 1, where `s = sqrt(1 - u^2)` is imaginary and clamps to zero — a silent band of
        // degenerate collinear geometries along the top of the table, smooth and
        // plausible and wrong. They caught it on the producing side before any handoff.
        // This catches it on the consuming side, which is where a door belongs: the two
        // checks are independent, and a hazard that is invisible in the numbers deserves
        // to be caught twice rather than once.
        if !grid.u.iter().all(|&u| (-1.0..=1.0).contains(&u)) {
            return Err(TrimerRefusal::AngleCosineOutOfRange);
        }
        if !grid.x.iter().chain(grid.y.iter()).all(|&r| r > 0.0) {
            return Err(TrimerRefusal::SideLengthNotPositive);
        }
        if grid.energy.len() != grid.nx * grid.ny * grid.nu {
            return Err(TrimerRefusal::EnergyCountMismatch);
        }
        // The declared rule, weighed against the coordinates it ships. The artifact says
        // the coordinates win; this leg is what makes that a checkable statement instead
        // of a licence for the name to be wrong.
        if !self.axis_rule.matches(&grid.x)
            || !self.axis_rule.matches(&grid.y)
            || !self.axis_rule.matches(&grid.u)
        {
            return Err(TrimerRefusal::AxisRuleContradictsCoordinates);
        }
        // ---- what the artifact SAYS ---------------------------------------------------
        if self.region.iter().any(|&r| r == 0) {
            return Err(TrimerRefusal::RegionShapeMissing);
        }
        if self.cited_curves == 0 {
            return Err(TrimerRefusal::DomainUncited);
        }
        // The uncertainty, in three questions: does it exist, is it below what this app
        // can read at all, and is it below the feature THIS surface claims to have found.
        // The pair door asks the same three and the middle one is the one it was missing.
        if !(self.uncertainty_ha > 0.0) {
            return Err(TrimerRefusal::UncertaintyMissing);
        }
        if !(self.uncertainty_ha < RESOLVABLE_TRIMER_UNCERTAINTY) {
            return Err(TrimerRefusal::UncertaintyExceedsResolution);
        }
        if self.peak_ha > 0.0 && !(self.uncertainty_ha < self.peak_ha) {
            return Err(TrimerRefusal::UncertaintyExceedsFeature);
        }
        if self.declares_converged {
            return Err(TrimerRefusal::ConvergedFieldPresent);
        }
        if self.void_count > 0 && self.void_named == 0 {
            return Err(TrimerRefusal::VoidsCountedNotNamed);
        }
        if self.seam == SeamRecord::Absent {
            return Err(TrimerRefusal::SeamRecordMissing);
        }
        if self.digest.iter().all(|&w| w == 0) {
            return Err(TrimerRefusal::DigestMissing);
        }
        Ok(())
    }
}

/// The shape and the coordinates a surface actually shipped.
///
/// ARBITRARY, not this build's 33 x 33 x 13. The emitter takes `--grid` and `--x/--y/--u`
/// on the command line, so a shipped surface can be any box at any resolution, and the
/// first real artifact is 4 x 4 x 2. An earlier draft of this module stored surfaces in
/// `holon_chem::trimer::TrimerTable`, whose grid is fixed at compile time; that could not
/// have held the artifact the emitter actually writes, and the mistake came from reading
/// matching node COUNTS in a schema example as a matching grid RULE.
#[derive(Clone, Debug, Default)]
pub struct SurfaceGrid {
    pub nx: usize,
    pub ny: usize,
    pub nu: usize,
    /// The node coordinates, in the convention `saturation3-mesh` stated at 7dff58c:
    /// `species[0]` is the APEX at the origin, `x` is the apex-to-`species[1]` side in
    /// bohr, `y` is the apex-to-`species[2]` side, and `u` is the COSINE of the angle
    /// between them, dimensionless on `[-1, 1]`. The third side is deliberately NOT
    /// stored — it is `sqrt(x^2 + y^2 - 2*x*y*u)` — because two copies of one number are
    /// two numbers that can disagree.
    ///
    /// AUTHORITATIVE where they disagree with `axis_rule` — the
    /// artifact's own statement of precedence, so that an emitter changing its spacing
    /// and forgetting to rename its rule still ships the truth.
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub u: Vec<f64>,
    /// The energy column, `nx * ny * nu` long in the artifact's canonical node order.
    pub energy: Vec<f64>,
}

/// One admitted surface: what it shipped, and what it said about it.
pub struct TrimerSurface {
    pub grid: SurfaceGrid,
    pub prov: TrimerProvenance,
}

/// The shipped surfaces this sandbox has admitted.
///
/// HEAP-BACKED throughout, and that is a size decision rather than a style one. `Sim` is
/// already 331 KB with the pair bank in it, and `holon-render-3d` has been bitten once by
/// that (its `AtomWorld` had to be boxed after the pair bank landed, or the debug profile
/// overflowed its stack building one). Surfaces are `Vec`s that are empty until something
/// is actually loaded, so a `Sim` that never sees one grows by a few words.
pub struct TrimerBank {
    pub surfaces: Vec<TrimerSurface>,
    /// The last refusal, so a host that got a code can ask what it meant.
    pub last_refusal: Option<TrimerRefusal>,
    /// The surface currently being pushed over the ABI.
    staging: Option<SurfaceGrid>,
    /// The digest words pushed for the staged surface. Held here rather than passed to
    /// `finish` because a SHA-256 is eight words and `finish` already takes sixteen
    /// declarations; an argument list nobody can read is its own kind of defect.
    pub staging_digest: [u32; 8],
}

impl TrimerBank {
    pub const fn empty() -> Self {
        Self {
            surfaces: Vec::new(),
            last_refusal: None,
            staging: None,
            staging_digest: [0; 8],
        }
    }

    pub fn clear(&mut self) {
        self.surfaces.clear();
        self.last_refusal = None;
        self.staging = None;
        self.staging_digest = [0; 8];
    }

    /// Open a surface of the declared shape. Discards any half-pushed one: an interrupted
    /// load must not be able to contribute its nodes to the next.
    pub fn begin(&mut self, nx: usize, ny: usize, nu: usize) {
        self.staging_digest = [0; 8];
        self.staging = Some(SurfaceGrid {
            nx,
            ny,
            nu,
            x: vec![f64::NAN; nx],
            y: vec![f64::NAN; ny],
            u: vec![f64::NAN; nu],
            energy: vec![f64::NAN; nx.saturating_mul(ny).saturating_mul(nu)],
        });
    }

    /// Push one coordinate. `axis` is 0 = x, 1 = y, 2 = u.
    pub fn axis_node(&mut self, axis: usize, index: usize, value: f64) -> bool {
        let Some(g) = self.staging.as_mut() else {
            return false;
        };
        let target = match axis {
            0 => &mut g.x,
            1 => &mut g.y,
            2 => &mut g.u,
            _ => return false,
        };
        match target.get_mut(index) {
            Some(slot) => {
                *slot = value;
                true
            }
            None => false,
        }
    }

    /// Push one 32-bit word of the artifact's SHA-256 merge digest.
    pub fn digest_word(&mut self, word: usize, value: u32) -> bool {
        match self.staging_digest.get_mut(word) {
            Some(slot) => {
                *slot = value;
                true
            }
            None => false,
        }
    }

    /// Push one energy value at the artifact's canonical node index.
    pub fn energy_node(&mut self, index: usize, value: f64) -> bool {
        let Some(g) = self.staging.as_mut() else {
            return false;
        };
        match g.energy.get_mut(index) {
            Some(slot) => {
                *slot = value;
                true
            }
            None => false,
        }
    }

    /// Close the staged surface and put it to the door.
    ///
    /// Takes the staging grid whatever happens, so a refused artifact cannot be finished
    /// twice or leave nodes behind for the next one to inherit.
    pub fn finish(&mut self, prov: TrimerProvenance) -> Result<usize, TrimerRefusal> {
        let Some(grid) = self.staging.take() else {
            self.last_refusal = Some(TrimerRefusal::SurfaceNotLoaded);
            return Err(TrimerRefusal::SurfaceNotLoaded);
        };
        self.commit(grid, prov)
    }

    /// Admit a surface, or refuse it and say why.
    ///
    /// The surface is stored only on success: a refused artifact leaves NOTHING behind, so
    /// a host that ignores the return code gets a sandbox with no surface rather than a
    /// sandbox quietly integrating a refused one. That is also what keeps the fence
    /// honest — see [`TrimerBank::any_heteronuclear`].
    pub fn commit(
        &mut self,
        grid: SurfaceGrid,
        prov: TrimerProvenance,
    ) -> Result<usize, TrimerRefusal> {
        if let Err(r) = prov.admit(&grid) {
            self.last_refusal = Some(r);
            return Err(r);
        }
        if self.surfaces.len() >= MAX_TRIMER_SURFACES {
            // Not a provenance refusal: a full bank and a bad surface are different
            // problems, the same distinction `BANK_FULL` draws on the pair side.
            self.surfaces.remove(0);
        }
        self.surfaces.push(TrimerSurface { grid, prov });
        self.last_refusal = None;
        Ok(self.surfaces.len() - 1)
    }

    /// Is any admitted surface one the browser could not have generated?
    ///
    /// This is the fence, and it is a question about what is LOADED rather than a constant.
    /// `holon_trimer_h_only` is its negation, and both viewers print their disclaimer from
    /// that export precisely so no sentence has to be hand-edited the day a surface lands.
    pub fn any_heteronuclear(&self) -> bool {
        self.surfaces.iter().any(|s| s.prov.is_heteronuclear())
    }

    /// The surface for an unordered species triple, if one is admitted.
    ///
    /// Unordered: `(H, H, Cl)` and `(Cl, H, H)` are the same physics, and an artifact's
    /// own ordering is its business rather than the force loop's.
    pub fn find(&self, z: [u8; 3]) -> Option<&TrimerSurface> {
        let want = sorted(z);
        self.surfaces.iter().find(|s| sorted(s.prov.z) == want)
    }

    pub fn len(&self) -> usize {
        self.surfaces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.surfaces.is_empty()
    }
}

/// The three atomic numbers in ascending order — the key an unordered triple hashes to.
fn sorted(mut z: [u8; 3]) -> [u8; 3] {
    z.sort_unstable();
    z
}
