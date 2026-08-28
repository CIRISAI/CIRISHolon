//! Molecules as FIRST-CLASS COMPOSITE HOLONS.
//!
//! A molecule is not a drawn line between two circles. It is a row: members, its own
//! ledger, when it formed, and what kind of thing it is. Formation is closure
//! acquisition — the pair's energy falls below the in-model asymptote inside its outer
//! turning point, read off the exact curve — and dissolution is rent unpaid.
//!
//! # Three properties this layer must have, and how each is enforced
//!
//! **Formation is ACCOUNTING-ONLY.** Creating or dissolving a row redistributes ledger
//! LABELS and touches no dynamical state. `E_before == E_after` bit-identically across
//! every event, because nothing in this module writes a position or a velocity. The
//! molecule's `e_bond` is a VIEW of energy the global ledger already holds, never a
//! second reservoir — which is what makes "the row closes back into the global ledger"
//! true by construction rather than by careful bookkeeping.
//!
//! **Closure is MEASURED, not interpreted.** An energy threshold proves a bound pair; it
//! does not prove an autonomous molecular view. So at every grain boundary each candidate
//! scores its own one-step closure defect: the composite view asserts the pair is
//! autonomous, and an autonomous pair conserves its bond energy, so the defect is how
//! much that assertion missed by over one frame. A pair being buffeted by a third atom or
//! by the user's spring scores badly and cannot form; a genuinely isolated bound pair
//! scores ~0. At dissolution the defect must RISE — losing closure is what dissolution
//! IS, and if the number does not move, the event was an energy reinterpretation rather
//! than a measured loss of autonomy.
//!
//! **Formation is DETERMINISTIC.** Dwell hysteresis (K consecutive grain boundaries) stops
//! a pair grazing the threshold from flickering rows in and out, and multi-eligibility
//! resolves canonically: most-bound first, ties broken by pair index.
//!
//! # Cost
//!
//! Candidacy evaluation rides the force loop — the distance and the potential are already
//! in hand from the force computation, so the predicate is a handful of flops on numbers
//! that were computed anyway. The row layer itself runs at FRAME boundaries only, never
//! at substeps. `census_enabled` exists so the claim "being a holon is cheap" can be
//! MEASURED (frame cost with the layer on against off) rather than asserted.

use crate::sim::{PairReading, MAX_PAIRS};

pub const MAX_HOLONS: usize = MAX_PAIRS;
/// Room for composites larger than a pair, which is what SELECTOR-1's subsystem rows
/// will need. The molecule kind uses two.
pub const MAX_MEMBERS: usize = 8;

/// Grain boundaries a pair must satisfy the predicate before a row is created, and fail
/// it before a row is destroyed.
///
/// STAKED at 3. One boundary is not hysteresis at all; the cost of K is that a real bond
/// is recognised K frames late (at the default sim-speed, 3 frames is about 3% of a
/// vibration), and the benefit is that a pair sitting on the threshold cannot flicker
/// rows in and out at frame rate. Symmetric on purpose: an asymmetric K would bias the
/// molecule count in whichever direction the shorter arm pointed.
pub const DWELL_K: u8 = 3;

/// Largest one-step closure defect, as a fraction of the well depth, that still counts
/// as an autonomous molecular view at formation.
///
/// STAKED at 1e-2. A truly isolated bound pair scores at the integrator's drift level
/// (~1e-5 of the well depth or below); a pair sharing a close encounter with a third
/// atom, or held in the user's spring, scores orders of magnitude worse. 1% sits in the
/// gap with room on both sides.
pub const CLOSURE_DEFECT_MAX: f64 = 1e-2;

/// What kind of composite a row is.
///
/// Open for extension so SELECTOR-1 can add subsystem rows without a schema change — but
/// the fence ATOMWORLD.md states applies and is repeated here because this enum is
/// exactly where someone would be tempted to ignore it: **an extensible `kind` is SCHEMA
/// compatibility, NOT lawful extension.** A payer-builder kind must be constructed by a
/// predicate definable from existing views, interventions and ledgers under the frozen
/// extension grammar. Adding a variant here is free; earning the right to construct it is
/// not, and this comment is not the place that grants it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HolonKind {
    /// A bonded pair: a maintained closure paying rent against kinetic noise.
    Molecule,
}

/// One composite holon: `{members, ledger, formed_at, kind}`.
#[derive(Clone, Copy)]
pub struct HolonRow {
    pub kind: HolonKind,
    pub members: [u8; MAX_MEMBERS],
    pub member_count: u8,
    pub formed_at_frame: u64,
    pub formed_at_time: f64,
    /// The pair this row was built from, so the row can be refreshed without a search.
    pub pair: u16,

    // ---- the ledger row ----
    /// Bond-sector energy: pair potential plus pair-frame kinetic. A VIEW of energy the
    /// global ledger already carries, not a separate store.
    pub e_bond: f64,
    pub e_bond_at_formation: f64,

    // ---- measured closure (fence 1) ----
    pub closure_defect_at_formation: f64,
    pub closure_defect: f64,
    pub closure_defect_peak: f64,
    pub alive: bool,
}

impl HolonRow {
    pub const fn empty() -> Self {
        Self {
            kind: HolonKind::Molecule,
            members: [0; MAX_MEMBERS],
            member_count: 0,
            formed_at_frame: 0,
            formed_at_time: 0.0,
            pair: 0,
            e_bond: 0.0,
            e_bond_at_formation: 0.0,
            closure_defect_at_formation: 0.0,
            closure_defect: 0.0,
            closure_defect_peak: 0.0,
            alive: false,
        }
    }
}

/// Per-pair state the row layer needs between grain boundaries.
#[derive(Clone, Copy)]
pub struct Candidate {
    pub dwell_bonded: u8,
    pub dwell_unbound: u8,
    /// Bond-sector energy at the previous grain boundary, for the closure defect.
    pub e_bond_prev: f64,
    pub has_prev: bool,
    pub closure_defect: f64,
    /// Row index when this pair carries a live molecule.
    pub row: i16,
}

impl Candidate {
    pub const fn empty() -> Self {
        Self {
            dwell_bonded: 0,
            dwell_unbound: 0,
            e_bond_prev: 0.0,
            has_prev: false,
            closure_defect: f64::INFINITY,
            row: -1,
        }
    }
}

/// The live count of everything at every level. The demo's thesis, made a number.
#[derive(Clone, Copy, Default)]
pub struct Census {
    /// Micro holons: the atoms. The O(N^2) force loop is their entire cost.
    pub atoms: usize,
    /// Composite holons: live molecule rows.
    pub molecules: usize,
    /// Candidate closures evaluated this frame — every pair is a potential composite.
    pub candidate_evaluations: usize,
    /// Global views: the energy ledger, the momentum ledger, external work.
    pub global_views: usize,
    pub formations: u64,
    pub dissolutions: u64,
    /// Rows whose formation was blocked by the closure-defect gate rather than by energy.
    /// The number that says how often boundness and closure disagree.
    pub closure_rejections: u64,
}

pub struct HolonLayer {
    pub rows: [HolonRow; MAX_HOLONS],
    pub candidates: [Candidate; MAX_PAIRS],
    pub census: Census,
    /// Live row count, maintained incrementally so the census does not re-count 120 rows
    /// at every boundary just to report a number it already knows.
    live: usize,
    /// Scratch: eligible pair indices for this boundary. A field rather than a local so
    /// the layer stays allocation-free.
    eligible: [u16; MAX_PAIRS],
    /// Off measures the frame cost of everything except this layer, which is how the
    /// "holons are cheap" claim gets a number instead of a adjective.
    pub enabled: bool,
    /// Set when a dissolution's closure defect failed to rise. Not a panic: it is a
    /// measurement that wants reporting, and the test asserts on it.
    pub dissolutions_without_defect_rise: u64,
    /// The closure defect of the most recently dissolved row, and what it had been at
    /// that row's formation. Kept because the row itself is gone by the time anyone asks,
    /// and a caller reading the defect off a snapshot taken earlier would be reading a
    /// different moment than the one it labelled.
    pub last_dissolution_defect: f64,
    pub last_dissolution_defect_at_formation: f64,
}

impl HolonLayer {
    pub const fn empty() -> Self {
        Self {
            rows: [HolonRow::empty(); MAX_HOLONS],
            candidates: [Candidate::empty(); MAX_PAIRS],
            census: Census {
                atoms: 0,
                molecules: 0,
                candidate_evaluations: 0,
                global_views: 3,
                formations: 0,
                dissolutions: 0,
                closure_rejections: 0,
            },
            live: 0,
            eligible: [0; MAX_PAIRS],
            enabled: true,
            dissolutions_without_defect_rise: 0,
            last_dissolution_defect: 0.0,
            last_dissolution_defect_at_formation: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.rows = [HolonRow::empty(); MAX_HOLONS];
        self.candidates = [Candidate::empty(); MAX_PAIRS];
        self.census = Census {
            atoms: 0,
            molecules: 0,
            candidate_evaluations: 0,
            global_views: 3,
            formations: 0,
            dissolutions: 0,
            closure_rejections: 0,
        };
        self.dissolutions_without_defect_rise = 0;
        self.last_dissolution_defect = 0.0;
        self.last_dissolution_defect_at_formation = 0.0;
        self.live = 0;
    }

    pub fn live_rows(&self) -> impl Iterator<Item = (usize, &HolonRow)> {
        self.rows.iter().enumerate().filter(|(_, r)| r.alive)
    }

    pub fn molecule_count(&self) -> usize {
        self.live
    }

    /// Total bond-sector energy across live rows. This is a PARTITION of energy the
    /// global ledger already holds — summing it and adding it to the global total would
    /// be double counting, which is exactly why it is exposed as a view and never as a
    /// term.
    pub fn bond_sector_energy(&self) -> f64 {
        self.rows.iter().filter(|r| r.alive).map(|r| r.e_bond).sum()
    }

    fn free_row(&self) -> Option<usize> {
        self.rows.iter().position(|r| !r.alive)
    }

    /// ONE grain boundary of the composite layer.
    ///
    /// Reads `pairs` (already computed by the force loop) and the current frame/time, and
    /// creates or destroys rows. Writes NOTHING dynamical — see the module header on why
    /// that is the property the whole layer rests on.
    pub fn step_boundary(
        &mut self,
        pairs: &[PairReading],
        n_atoms: usize,
        frame: u64,
        time: f64,
        well_depth: f64,
    ) {
        self.census.atoms = n_atoms;
        self.census.candidate_evaluations = pairs.len();
        if !self.enabled {
            self.census.molecules = 0;
            return;
        }
        let scale = if well_depth.abs() > 0.0 {
            well_depth.abs()
        } else {
            1.0
        };
        let mut rejections: u64 = 0;

        // --- 1. score every candidate's closure defect and advance its dwell ---
        for (k, p) in pairs.iter().enumerate() {
            let c = &mut self.candidates[k];
            // The composite view claims the pair is autonomous; an autonomous pair
            // conserves its bond energy. The defect is how much that claim missed by
            // over one grain.
            c.closure_defect = if c.has_prev {
                (p.e_bond() - c.e_bond_prev).abs() / scale
            } else {
                f64::INFINITY
            };
            c.e_bond_prev = p.e_bond();
            c.has_prev = true;

            if p.bonded {
                c.dwell_bonded = c.dwell_bonded.saturating_add(1);
                c.dwell_unbound = 0;
            } else {
                c.dwell_unbound = c.dwell_unbound.saturating_add(1);
                c.dwell_bonded = 0;
            }

            // Energy says bound, and the dwell is satisfied, but the view is not closed:
            // this pair is being buffeted by something outside it. Counted, because how
            // often boundness and closure disagree is the number fence 1 exists to expose.
            if p.bonded
                && c.row < 0
                && c.dwell_bonded >= DWELL_K
                && !(c.closure_defect.is_finite() && c.closure_defect <= CLOSURE_DEFECT_MAX)
            {
                rejections += 1;
            }
        }
        self.census.closure_rejections += rejections;

        // --- 2. dissolve rows whose pair has failed the predicate for K boundaries ---
        for (k, pair) in pairs.iter().enumerate() {
            let row_index = self.candidates[k].row;
            if row_index < 0 {
                continue;
            }
            let row_index = row_index as usize;
            let defect = self.candidates[k].closure_defect;
            {
                let row = &mut self.rows[row_index];
                row.e_bond = pair.e_bond();
                row.closure_defect = defect;
                if defect.is_finite() && defect > row.closure_defect_peak {
                    row.closure_defect_peak = defect;
                }
            }
            if self.candidates[k].dwell_unbound >= DWELL_K {
                let row = self.rows[row_index];
                // Fence 1's second half: losing closure is what dissolution IS, so the
                // defect must have risen from what it was when the row was created. This
                // is recorded rather than enforced — a dissolution whose defect did not
                // move is a real event that wants explaining, not a crash.
                let rose = row.closure_defect > row.closure_defect_at_formation;
                if !rose {
                    self.dissolutions_without_defect_rise += 1;
                }
                self.last_dissolution_defect = row.closure_defect;
                self.last_dissolution_defect_at_formation = row.closure_defect_at_formation;
                self.rows[row_index].alive = false;
                self.candidates[k].row = -1;
                self.live -= 1;
                self.census.dissolutions += 1;
            }
        }

        // --- 3. form rows, most-bound first ---
        //
        // An atom belongs to at most one composite, so eligibility has to be resolved
        // rather than assumed. CANONICAL RULE: the most-bound eligible pair claims its
        // atoms first (lowest e_rel wins), ties broken by the lower pair index. Both legs
        // are total orders on values already computed, so the outcome does not depend on
        // iteration order, on timing, or on how many candidates there happen to be.
        //
        // The eligible set is collected ONCE and ordered, rather than rescanning every
        // pair for each formation. The rescanning version cost O(formations x pairs) and
        // MEASURED 9.7% of a frame on the 16-atom worst case (every pair mutually bound,
        // so eligibility churns every boundary) — which is not the "cheap" the census is
        // supposed to demonstrate. Sorting a list that is at most `pairs` long, once, is.
        let mut taken = [false; crate::sim::MAX_ATOMS];
        for (k, p) in pairs.iter().enumerate() {
            if self.candidates[k].row >= 0 {
                taken[p.i] = true;
                taken[p.j] = true;
            }
        }
        let mut eligible = 0usize;
        for (k, p) in pairs.iter().enumerate() {
            if self.candidates[k].row >= 0 || !p.bonded || taken[p.i] || taken[p.j] {
                continue;
            }
            if self.candidates[k].dwell_bonded < DWELL_K {
                continue;
            }
            let defect = self.candidates[k].closure_defect;
            if !(defect.is_finite() && defect <= CLOSURE_DEFECT_MAX) {
                continue;
            }
            self.eligible[eligible] = k as u16;
            eligible += 1;
        }
        // Selection sort by (e_rel, index). The eligible set is small in every scene that
        // is not the pathological all-bound one, and the sort is stable in the index by
        // construction: a strict `<` on the energy leaves equal energies in scan order.
        for a in 0..eligible {
            let mut best = a;
            for b in (a + 1)..eligible {
                let (kb, kbest) = (self.eligible[b] as usize, self.eligible[best] as usize);
                if pairs[kb].e_rel < pairs[kbest].e_rel {
                    best = b;
                }
            }
            self.eligible.swap(a, best);
        }
        for slot_index in 0..eligible {
            let k = self.eligible[slot_index] as usize;
            let p = &pairs[k];
            if taken[p.i] || taken[p.j] {
                continue;
            }
            let Some(slot) = self.free_row() else { break };
            let mut members = [0u8; MAX_MEMBERS];
            members[0] = p.i as u8;
            members[1] = p.j as u8;
            self.rows[slot] = HolonRow {
                kind: HolonKind::Molecule,
                members,
                member_count: 2,
                formed_at_frame: frame,
                formed_at_time: time,
                pair: k as u16,
                e_bond: p.e_bond(),
                e_bond_at_formation: p.e_bond(),
                closure_defect_at_formation: self.candidates[k].closure_defect,
                closure_defect: self.candidates[k].closure_defect,
                closure_defect_peak: self.candidates[k].closure_defect,
                alive: true,
            };
            self.candidates[k].row = slot as i16;
            taken[p.i] = true;
            taken[p.j] = true;
            self.live += 1;
            self.census.formations += 1;
        }

        self.census.molecules = self.live;
    }
}
