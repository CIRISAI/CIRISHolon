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

use holon_chem::trimer::TrimerTable;

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
    /// This build's H3 spacing exactly: `trimer::r_of_tau` on both distance axes with the
    /// crate's own `R_LO`, `R_HI` and `STRETCH_A`, and `trimer::node_c` on `u`. A table
    /// declaring this is interpolable by the existing evaluator with no new code.
    TauStretchH3,
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
    /// The merge digest. Zero means absent.
    pub digest: u64,
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
            digest: 0,
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
    pub fn admit(&self, loaded: bool) -> Result<(), TrimerRefusal> {
        if self.route == crate::bank::Route::Undeclared {
            return Err(TrimerRefusal::RouteUndeclared);
        }
        if self.axis_rule == AxisRule::Undeclared {
            return Err(TrimerRefusal::GridRuleUnsupported);
        }
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
        if self.digest == 0 {
            return Err(TrimerRefusal::DigestMissing);
        }
        if !loaded {
            return Err(TrimerRefusal::SurfaceNotLoaded);
        }
        Ok(())
    }
}

/// One admitted surface: the nodes, and what the artifact said about them.
pub struct TrimerSurface {
    pub table: TrimerTable,
    pub prov: TrimerProvenance,
}

/// The shipped surfaces this sandbox has admitted.
///
/// HEAP-BACKED, and that is a size decision rather than a style one. A `TrimerTable` is
/// 14,157 `f64` — 113 KB — and `Sim` is already 331 KB with the pair bank in it. Four
/// surfaces held inline would take `Sim` past three quarters of a megabyte, which
/// `holon-render-3d` has already been bitten by once (its `AtomWorld` had to be boxed
/// after the pair bank landed, or the debug profile overflowed its stack building one).
/// A `Vec` that is empty until a surface is actually loaded costs 24 bytes in the common
/// case, which is the same choice `water` made and for the same reason.
pub struct TrimerBank {
    pub surfaces: Vec<TrimerSurface>,
    /// The last refusal, so a host that got a code can ask what it meant.
    pub last_refusal: Option<TrimerRefusal>,
    /// The surface currently being pushed, node by node, over the ABI.
    ///
    /// Boxed and absent until `begin`, for the size reason in this struct's header: a
    /// staging table held inline would put 113 KB in every `Sim` whether or not a host
    /// ever loads a surface.
    staging: Option<Box<TrimerTable>>,
}

impl TrimerBank {
    pub const fn empty() -> Self {
        Self {
            surfaces: Vec::new(),
            last_refusal: None,
            staging: None,
        }
    }

    pub fn clear(&mut self) {
        self.surfaces.clear();
        self.last_refusal = None;
        self.staging = None;
    }

    /// Open a surface for filling. Discards any half-pushed one: an interrupted load must
    /// not be able to contribute its nodes to the next.
    pub fn begin(&mut self) {
        let mut t = Box::new(TrimerTable::empty());
        t.begin();
        self.staging = Some(t);
    }

    /// Push one node value. False if there is no open surface or the node is rejected.
    pub fn knot(&mut self, index: usize, value: f64) -> bool {
        match self.staging.as_mut() {
            Some(t) => t.knot(index, value),
            None => false,
        }
    }

    /// Close the staged surface and put it to the door.
    ///
    /// Takes the staging table whatever happens, so a refused artifact cannot be finished
    /// twice or leave nodes behind for the next one to inherit.
    pub fn finish(
        &mut self,
        meta: holon_chem::trimer::TrimerMeta,
        prov: TrimerProvenance,
    ) -> Result<usize, TrimerRefusal> {
        let Some(mut t) = self.staging.take() else {
            self.last_refusal = Some(TrimerRefusal::SurfaceNotLoaded);
            return Err(TrimerRefusal::SurfaceNotLoaded);
        };
        // `finish` is false when nodes are missing; the table then reports `loaded =
        // false` and the door refuses it on that leg rather than here, so there is one
        // place that decides admissions.
        t.finish(meta);
        self.commit(*t, prov)
    }

    /// Admit a surface, or refuse it and say why.
    ///
    /// The surface is moved in only on success: a refused artifact leaves NOTHING behind,
    /// so a host that ignores the return code gets a sandbox with no surface rather than a
    /// sandbox quietly integrating a refused one.
    pub fn commit(
        &mut self,
        table: TrimerTable,
        prov: TrimerProvenance,
    ) -> Result<usize, TrimerRefusal> {
        if let Err(r) = prov.admit(table.loaded) {
            self.last_refusal = Some(r);
            return Err(r);
        }
        if self.surfaces.len() >= MAX_TRIMER_SURFACES {
            // Not a provenance refusal: a full bank and a bad surface are different
            // problems, the same distinction `BANK_FULL` draws on the pair side.
            self.surfaces.remove(0);
        }
        self.surfaces.push(TrimerSurface { table, prov });
        self.last_refusal = None;
        Ok(self.surfaces.len() - 1)
    }

    /// Is any admitted surface one the browser could not have generated?
    ///
    /// This is the fence, and it is a question about what is LOADED rather than a constant.
    /// `holon_trimer_h_only` is its negation, and the viewer prints its disclaimer from
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
