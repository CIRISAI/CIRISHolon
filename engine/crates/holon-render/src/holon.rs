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

use crate::sim::PairReading;

/// Room for composites larger than a pair, which is what SELECTOR-1's subsystem rows
/// will need. The molecule kind uses two.
pub const MAX_MEMBERS: usize = 8;

/// How many rows the layer starts with. NOT a cap: `free_row` grows the vector when every
/// row is live. It used to be `MAX_PAIRS` — a hard `C(16,2) = 120` — which was a cap on
/// how many molecules a scene could hold, derived from a cap on how many atoms it could
/// hold, and both are gone (T3).
const INITIAL_ROWS: usize = 64;

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
    /// Atom indices. `u32`, not `u8`: the scene is no longer capped at sixteen atoms, and
    /// a member index that silently wrapped at 256 would name the wrong atom rather than
    /// fail.
    pub members: [u32; MAX_MEMBERS],
    pub member_count: u8,
    pub formed_at_frame: u64,
    pub formed_at_time: f64,
    /// The pair this row was built from, BY ATOM INDEX rather than by position in the pair
    /// list.
    ///
    /// The list used to be every pair in `(i, j)` order, so a pair's position in it was a
    /// stable name for the pair. With a cutoff-local list the position moves as neighbours
    /// come and go, and a row holding a position would silently start describing a
    /// different pair. The atoms are the identity; the index never was.
    pub pair: (u32, u32),

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
            pair: (0, 0),
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
///
/// Carries its OWN identity. Dwell counters and the previous boundary's bond energy are
/// state that has to follow a pair across frames, and the pair's position in the reading
/// list is no longer a name that survives a frame (see [`HolonRow::pair`]). So the
/// candidate list is kept sorted by `(i, j)` and merged against the new reading list at
/// every boundary — a pair present in both keeps its history, a pair that has left loses
/// it, and neither depends on how the list happened to be ordered.
#[derive(Clone, Copy)]
pub struct Candidate {
    pub i: u32,
    pub j: u32,
    pub dwell_bonded: u8,
    pub dwell_unbound: u8,
    /// Bond-sector energy at the previous grain boundary, for the closure defect.
    pub e_bond_prev: f64,
    pub has_prev: bool,
    pub closure_defect: f64,
    /// Row index when this pair carries a live molecule.
    pub row: i32,
}

impl Candidate {
    pub const fn empty() -> Self {
        Self {
            i: 0,
            j: 0,
            dwell_bonded: 0,
            dwell_unbound: 0,
            e_bond_prev: 0.0,
            has_prev: false,
            closure_defect: f64::INFINITY,
            row: -1,
        }
    }

    #[inline]
    fn key(&self) -> (u32, u32) {
        (self.i, self.j)
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
    pub rows: Vec<HolonRow>,
    /// One entry per pair of the CURRENT reading list, in the same order, kept aligned by
    /// the merge in [`HolonLayer::step_boundary`].
    pub candidates: Vec<Candidate>,
    pub census: Census,
    /// Live row count, maintained incrementally so the census does not re-count 120 rows
    /// at every boundary just to report a number it already knows.
    live: usize,
    /// Scratch: eligible pair indices for this boundary. A field rather than a local so
    /// the layer stays allocation-free once a scene has settled at a size.
    eligible: Vec<u32>,
    /// Scratch: which atoms already belong to a live row this boundary.
    taken: Vec<bool>,
    /// Scratch: the merged candidate list under construction.
    merged: Vec<Candidate>,
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
            rows: Vec::new(),
            candidates: Vec::new(),
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
            eligible: Vec::new(),
            taken: Vec::new(),
            merged: Vec::new(),
            enabled: true,
            dissolutions_without_defect_rise: 0,
            last_dissolution_defect: 0.0,
            last_dissolution_defect_at_formation: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.rows.clear();
        self.rows.resize(INITIAL_ROWS, HolonRow::empty());
        self.candidates.clear();
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

    /// A free row, GROWING the table when every row is live.
    ///
    /// `None` was the old answer at 120 rows and it meant a formation was silently
    /// dropped: the molecule count would stop rising and nothing would say why. A row is
    /// 96 bytes, so the table doubling is cheaper than the misreading.
    fn free_row(&mut self) -> usize {
        if let Some(k) = self.rows.iter().position(|r| !r.alive) {
            return k;
        }
        let k = self.rows.len();
        self.rows
            .resize((self.rows.len() * 2).max(INITIAL_ROWS), HolonRow::empty());
        k
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

        // --- 0. carry the candidate state onto THIS boundary's reading list ---
        //
        // Both lists are sorted by `(i, j)` — the reading list by construction, the
        // candidate list because the previous merge left it that way — so this is a linear
        // merge with no search and no allocation past the first boundary.
        //
        // A pair that has LEFT the list has left it for one of two reasons: it moved past
        // the neighbour cutoff, or the scene was resized. Either way it is not a bonded
        // pair any more, so a row it was carrying is dissolved here rather than left alive
        // with nothing refreshing it. That dissolution is counted like any other; what it
        // is NOT is a defect-rise event, because there was no boundary at which to measure
        // the rise — so it does not touch `dissolutions_without_defect_rise`, which would
        // otherwise report a departure as a physics anomaly.
        let mut merged = core::mem::take(&mut self.merged);
        merged.clear();
        merged.reserve(pairs.len());
        {
            let old = &self.candidates;
            let mut a = 0usize;
            for p in pairs.iter() {
                let key = (p.i as u32, p.j as u32);
                while a < old.len() && old[a].key() < key {
                    // Departed. Release its row.
                    if old[a].row >= 0 {
                        let r = old[a].row as usize;
                        if self.rows[r].alive {
                            self.rows[r].alive = false;
                            self.live -= 1;
                            self.census.dissolutions += 1;
                        }
                    }
                    a += 1;
                }
                if a < old.len() && old[a].key() == key {
                    merged.push(old[a]);
                    a += 1;
                } else {
                    merged.push(Candidate {
                        i: key.0,
                        j: key.1,
                        ..Candidate::empty()
                    });
                }
            }
            while a < old.len() {
                if old[a].row >= 0 {
                    let r = old[a].row as usize;
                    if self.rows[r].alive {
                        self.rows[r].alive = false;
                        self.live -= 1;
                        self.census.dissolutions += 1;
                    }
                }
                a += 1;
            }
        }
        self.merged = core::mem::take(&mut self.candidates);
        self.candidates = merged;

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
        let mut taken = core::mem::take(&mut self.taken);
        taken.clear();
        taken.resize(n_atoms, false);
        for (k, p) in pairs.iter().enumerate() {
            if self.candidates[k].row >= 0 {
                taken[p.i] = true;
                taken[p.j] = true;
            }
        }
        self.eligible.clear();
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
            self.eligible.push(k as u32);
        }
        let eligible = self.eligible.len();
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
            let slot = self.free_row();
            let mut members = [0u32; MAX_MEMBERS];
            members[0] = p.i as u32;
            members[1] = p.j as u32;
            self.rows[slot] = HolonRow {
                kind: HolonKind::Molecule,
                members,
                member_count: 2,
                formed_at_frame: frame,
                formed_at_time: time,
                pair: (p.i as u32, p.j as u32),
                e_bond: p.e_bond(),
                e_bond_at_formation: p.e_bond(),
                closure_defect_at_formation: self.candidates[k].closure_defect,
                closure_defect: self.candidates[k].closure_defect,
                closure_defect_peak: self.candidates[k].closure_defect,
                alive: true,
            };
            self.candidates[k].row = slot as i32;
            taken[p.i] = true;
            taken[p.j] = true;
            self.live += 1;
            self.census.formations += 1;
        }
        self.taken = taken;

        self.census.molecules = self.live;
    }
}
