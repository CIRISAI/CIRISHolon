//! FLUID-1's carrier: the orientation lattice — a particle that carries a direction, a bond
//! that is a RECORDED PAIR of slots, a closure that moves as one, and the rent clause that
//! releases it.
//!
//! `conformance/mesh/FLUID1_PREREG.md` is the freeze. FLUID-0 read branch (c): over all 4,608
//! REG+ laws of the single-species lattice gas the Schmidt number stays under `0.45` against
//! water's `435`, so a carrier with no closure in it cannot carry the liquid's two
//! coefficients. This module puts a closure in the carrier and keeps every conserved integer
//! exact while doing it.
//!
//! # What is a bond here
//!
//! A bond is the ordered pair of SLOTS `(donor, acceptor)`, a slot being `cell · 6 + dir`.
//! It is never a flag on a particle (M-TAG-AS-PROPERTY): the two role tables
//! [`OrientationLattice::donor_of`] and [`OrientationLattice::acceptor_of`] are an INDEX into
//! the pair list, and the pair list is the object. A particle holds at most one donor role
//! and at most one acceptor role.
//!
//! # The step, and the four rules that are this module's own
//!
//! One step is (1) collision, (2) formation, (3) breaking, (4) streaming — the freeze's
//! order. Four rules the freeze leaves to the instrument are DECLARED here and nowhere else:
//!
//! 1. **The collision's slot remap.** `s → t` by [`Lattice::collision_image`] — the same
//!    table and the same chirality hash as [`Lattice::advance`]. The cell's incoming
//!    particles are re-seated on the outgoing slots: the BONDED ones first, in ascending
//!    incoming-direction order onto the ascending outgoing slots, each keeping its
//!    orientation and both of its roles; the free ones take the remaining outgoing slots with
//!    their orientations shuffled among themselves by a partial Fisher–Yates off the counter
//!    hash. The orientation multiset of the cell is therefore preserved EXACTLY, so the
//!    per-orientation census is conserved and not merely its total.
//!
//! 2. **A bond pays no rent on the step it forms.** The break test of phase (3) applies only
//!    to bonds that existed at the START of the step. Without this clause, formation at
//!    `p_form = 1` re-makes every broken bond inside the same step and the held fraction is
//!    exactly `1 − p_break` at every chart, so the freeze's own `1/(1 + p_break)` (G0's
//!    retention and G3's detailed balance) could not be met by any value of `E₀/kT_lat`. With
//!    it, the free state lasts exactly one step and the stationary held fraction is
//!    `1/(1 + p_break)`, which is what the freeze sets the chart from.
//!
//! 3. **Bonded streaming keeps the LABELS and moves the POSITIONS.** The mover's direction
//!    `m ∈ {a, b}` is drawn per step by the counter hash between the two particles' own
//!    directions; both particles are DISPLACED by `dirs[m]`, and both keep their own
//!    direction label. Momentum is `Σ dirs[label]`, so the joint move leaves both components
//!    integer-exact. Giving the two particles the label `m` instead would change the momentum
//!    by `dirs[m] − dirs[c]` per bond per step and there would be no shear readout left to
//!    take. Exactly one particle is then displaced off its own plane's shift (the one whose
//!    label is not `m`; when `a = b` there is none), and that hop is executed only into a
//!    VACANCY of that plane: if the target slot is already occupied after the shift, the
//!    pair's joint move is REFUSED, both particles stream normally, and the bond is released
//!    as `blocked` — counted apart from a rent break, never folded into it. Every plane's map
//!    is then a shift composed with moves into vacancies, which is injective, so mass and both
//!    momenta are exact under either branch. [`tests::momentum_is_exact_under_bonded_streaming`]
//!    measures that rather than asserting it.
//!
//!    **The pass is over bonds, and a particle is displaced at most once (amended).** A
//!    particle holds two roles, so it can sit in two bonds at once and a bond can therefore
//!    be a link of a CHAIN, not only of an isolated pair. The rule above is stated for a
//!    pair and says nothing about a chain, so the pass makes it well-defined and does so in
//!    the branch the rule already has. Bonds are walked in ascending POST-SHIFT donor slot,
//!    an order computed ONCE before any hop (walking the role table live would revisit a
//!    bond whose donor a hop had just moved forward, and process it twice). Each joint move
//!    CLAIMS its two particles at its own `m`. A later bond whose particle is already claimed
//!    at that same `m` needs no second hop for it — so a chain whose bonds draw the same
//!    mover moves as one whole — and a later bond whose particle is claimed at a DIFFERENT
//!    `m` is REFUSED and released as `blocked`, exactly as a missing vacancy refuses it.
//!    Every particle is therefore displaced by exactly one direction per step, which is what
//!    keeps each plane's map injective. The mover's draw is keyed on the PRE-shift donor
//!    slot, so it does not depend on which pass order the bond is walked in. A hop carries
//!    BOTH of the moved particle's roles and rewrites BOTH bonds' recorded slots; rewriting
//!    only the bond being processed is what left a stale slot in the role table.
//!
//! 4. **Formation is a greedy pass in ascending `(cell, slot)` order**, and the acceptor is
//!    the lowest occupied slot of the target cell with a free acceptor role. Unlike
//!    [`Lattice::advance`], whose result depends on no traversal order, this one does: several
//!    donors may point into one cell and the roles are finite. The order is fixed and the run
//!    is reproducible bitwise, but the rule is sequential and is declared as such.
//!
//! # The colour rides the particle, and the collision is still colour-blind
//!
//! FLUID-0's tracer readout is reused UNCHANGED IN RULE: at a firing collision the red bits
//! are redistributed among the outgoing particles by [`crate::lattice::recolour`], and
//! streaming carries a colour bit with its particle — including the anomalous hop of a bonded
//! pair. So `D` is the self-diffusion of a particle that is sometimes trapped, and with bonds
//! forbidden the whole colour plane is bit-identical to [`Lattice::advance_with_colour`].

use crate::chart::BlockChart;
use crate::lattice::{mix64, recolour, ColourRule, Lattice};
use crate::state::Model;
use crate::transport::{fit_with, FitRule, TransportReading};
use core::f64::consts::PI;

/// The orientation of an unoccupied slot. Not a direction; a hole.
pub const NO_ORIENT: u8 = 0xFF;
/// "This role is free" in [`OrientationLattice::donor_of`] / `acceptor_of`.
pub const NO_BOND: u32 = u32::MAX;
/// The six directions of FHP-6; this module is defined on that chart alone.
pub const N_DIRS: usize = 6;

/// The counter-hash streams, held apart from `lattice.rs`'s three so that a particle's
/// orientation, its colour and its bond's fate are independent draws.
const ORIENT_SEED_KEY: u64 = 0x4F72_6965_6E74_5364;
const ORIENT_MIX_KEY: u64 = 0x4F72_6965_6E74_4D78;
const BREAK_KEY: u64 = 0x426F_6E64_4272_6B21;
const MOVER_KEY: u64 = 0x426F_6E64_4D76_7221;
const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

/// The crate's convention for a uniform `[0,1)` out of a counter hash.
#[inline]
fn unit(h: u64) -> f64 {
    (h >> 11) as f64 / ((1u64 << 53) as f64)
}

/// One bond: the ordered pair of slots, and nothing else. The link direction and the acceptor
/// angle are DERIVED from the state ([`OrientationLattice::bond_geometry`]) rather than stored,
/// so a stale copy of either cannot exist.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bond {
    pub donor: u32,
    pub acceptor: u32,
}

/// The bond rule's parameters — the amplitude table READ from CT-2 and the chart's one number.
#[derive(Clone, Copy, Debug)]
pub struct OrientationRules {
    /// Bonds may form. `false` is the no-bond control, which must reproduce FLUID-0.
    pub bonds_enabled: bool,
    /// `A(φ)` at the six lattice angles, indexed by `φ / 60°`. READ from CT-2's tilt family;
    /// never typed into this crate.
    pub amplitude: [f64; N_DIRS],
    /// `E₀ / kT_lat`, set from ONE measured retention by [`OrientationRules::rent_from_retention`].
    pub rent: f64,
    /// `p_break ≡ 0`: the freeze's `T → 0` limit, run as the cold control.
    pub cold: bool,
    /// Streaming on. `false` is G3's held-geometry scene.
    pub stream: bool,
}

impl OrientationRules {
    /// `p_break(φ) = exp(−A(φ)·E₀/kT_lat)`, and exactly zero in the cold control.
    #[inline]
    pub fn p_break(&self, phi_index: usize) -> f64 {
        if self.cold {
            0.0
        } else {
            (-self.amplitude[phi_index] * self.rent).exp()
        }
    }

    /// The chart, from ONE measured number: the value of `E₀/kT_lat` at which a linear bond
    /// (`φ = 0°`, where `A = 1`) has stationary held fraction `1/(1 + p_break) = f`.
    ///
    /// `1/(1 + exp(−E₀/kT_lat)) = f` ⟹ `E₀/kT_lat = ln(f / (1 − f))`.
    pub fn rent_from_retention(f: f64) -> f64 {
        assert!(f > 0.0 && f < 1.0, "a retention outside (0,1) sets no chart");
        (f / (1.0 - f)).ln()
    }

    /// The stationary held fraction of the two-state chain the rent clause is: formation at
    /// `p_form = 1` out of the free state, breaking at `p_break` out of the held state, and a
    /// bond paying no rent on the step it forms.
    #[inline]
    pub fn retention(p_break: f64) -> f64 {
        1.0 / (1.0 + p_break)
    }
}

/// What one step did. Every field is a COUNT: a gate reporting PASS on zero work has not
/// passed (M-VACUOUS-SUCCESS).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepCounts {
    pub collisions_fired: u64,
    pub formed: u64,
    pub broken_rent: u64,
    /// Bonds released because the pair's joint move was REFUSED — for want of a vacancy in
    /// the non-mover's plane, or because one of the two particles was already claimed by an
    /// earlier joint move at a different mover direction. One count, one branch.
    pub blocked: u64,
    /// The two prongs of `blocked`, reported apart so a branch that never fires is visible
    /// rather than hidden inside their sum. `blocked_no_vacancy + blocked_claimed + the
    /// geometry refusal of the rent pass = blocked`.
    pub blocked_no_vacancy: u64,
    pub blocked_claimed: u64,
    /// A held bond whose recorded link was not a link when the rent pass looked at it. Zero
    /// by construction; carried as a count so a defect shows up in the record.
    pub blocked_no_geometry: u64,
    /// Bonds exposed to the break rule this step, by acceptor angle.
    pub break_tests: [u64; N_DIRS],
    pub broken_by_phi: [u64; N_DIRS],
    /// Joint moves attempted, and the subset needing a hop off the plane's own shift.
    pub joint_moves: u64,
    pub anomalous_hops: u64,
}

impl StepCounts {
    fn add(&mut self, o: &StepCounts) {
        self.collisions_fired += o.collisions_fired;
        self.formed += o.formed;
        self.broken_rent += o.broken_rent;
        self.blocked += o.blocked;
        self.blocked_no_vacancy += o.blocked_no_vacancy;
        self.blocked_claimed += o.blocked_claimed;
        self.blocked_no_geometry += o.blocked_no_geometry;
        self.joint_moves += o.joint_moves;
        self.anomalous_hops += o.anomalous_hops;
        for k in 0..N_DIRS {
            self.break_tests[k] += o.break_tests[k];
            self.broken_by_phi[k] += o.broken_by_phi[k];
        }
    }
}

/// The conserved integers of the orientation lattice at one instant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OrientationLedger {
    pub mass: i64,
    pub momentum: [i64; 2],
    /// The count of each orientation over occupied slots. Streaming permutes positions and the
    /// collision permutes the cell's own multiset, so every entry is conserved — the freeze
    /// gates the TOTAL and this carries the stronger reading beside it.
    pub census: [i64; N_DIRS],
    pub census_total: i64,
    pub red: i64,
    pub bonds: i64,
}

/// G2's carrier: every integer, checked at EVERY step, plus the bond balance.
#[derive(Clone, Copy, Debug)]
pub struct OrientationAudit {
    pub mass_exact: bool,
    pub px_exact: bool,
    pub py_exact: bool,
    pub red_exact: bool,
    pub orientation_total_exact: bool,
    pub orientation_census_exact: bool,
    /// `held(t) = held(t−1) + formed − broken_rent − blocked`, integer-identical at every step.
    pub bond_balance_exact: bool,
    pub steps_checked: u64,
    pub mass: i64,
    pub px: i64,
    pub py: i64,
    pub red: i64,
    pub orientation_total: i64,
    pub bonds_final: i64,
    pub totals: StepCounts,
}

impl OrientationAudit {
    pub fn all_exact(&self) -> bool {
        self.mass_exact
            && self.px_exact
            && self.py_exact
            && self.red_exact
            && self.orientation_total_exact
            && self.orientation_census_exact
            && self.bond_balance_exact
    }

    fn fresh(l0: &OrientationLedger) -> Self {
        Self {
            mass_exact: true,
            px_exact: true,
            py_exact: true,
            red_exact: true,
            orientation_total_exact: true,
            orientation_census_exact: true,
            bond_balance_exact: true,
            steps_checked: 0,
            mass: l0.mass,
            px: l0.momentum[0],
            py: l0.momentum[1],
            red: l0.red,
            orientation_total: l0.census_total,
            bonds_final: l0.bonds,
            totals: StepCounts::default(),
        }
    }

    /// Fold another member's audit into this one: the flags are ANDed, the counts summed.
    /// Public because a runner that advances several members in lockstep reports ONE audit.
    pub fn merge_public(&mut self, o: &OrientationAudit) {
        self.merge(o)
    }

    fn merge(&mut self, o: &OrientationAudit) {
        self.mass_exact &= o.mass_exact;
        self.px_exact &= o.px_exact;
        self.py_exact &= o.py_exact;
        self.red_exact &= o.red_exact;
        self.orientation_total_exact &= o.orientation_total_exact;
        self.orientation_census_exact &= o.orientation_census_exact;
        self.bond_balance_exact &= o.bond_balance_exact;
        self.steps_checked += o.steps_checked;
        self.totals.add(&o.totals);
    }
}

/// The whole mutable state, for a probe that must restore a base configuration without
/// rebuilding the neighbour table (rebuilding it per probe is what `probe.rs` warns about).
#[derive(Clone, Debug)]
pub struct OrientationState {
    pub cells: Vec<u8>,
    pub orient: Vec<u8>,
    pub donor_of: Vec<u32>,
    pub acceptor_of: Vec<u32>,
    pub bonds: Vec<Bond>,
    pub colour: Option<Vec<u8>>,
    pub step_index: u64,
}

/// The orientation-carrying lattice: FHP-6's occupation beside a per-slot orientation and a
/// bond table.
#[derive(Clone, Debug)]
pub struct OrientationLattice {
    /// Tables only — the model, the torus wrap, the collision table and the chirality hash.
    /// Its own `cells` are the SEEDING and are not advanced; the live occupation is
    /// [`OrientationLattice::cells`].
    tables: Lattice,
    pub l: usize,
    pub cells: Vec<u8>,
    /// `orient[cell · 6 + dir]`, [`NO_ORIENT`] where the slot is empty.
    pub orient: Vec<u8>,
    /// `donor_of[slot]` — the index in [`OrientationLattice::bonds`] of the bond this
    /// particle DONATES, or [`NO_BOND`].
    pub donor_of: Vec<u32>,
    /// `acceptor_of[slot]` — the bond this particle ACCEPTS, or [`NO_BOND`].
    pub acceptor_of: Vec<u32>,
    /// The bonds themselves. The pair list is the object; the two tables above are its index.
    pub bonds: Vec<Bond>,
    /// The tracer's colour plane, when one is carried.
    pub colour: Option<Vec<u8>>,
    pub rules: OrientationRules,
    pub step_index: u64,
    pub counts: StepCounts,
    pub audit: OrientationAudit,
    /// Auditing every step costs one full ledger per step. The W probe advances millions of
    /// single steps and turns it off; every reading leaves it on.
    pub audit_every_step: bool,
    initial: OrientationLedger,
    // scratch, allocated once
    out_cells: Vec<u8>,
    out_orient: Vec<u8>,
    out_donor: Vec<u32>,
    out_acceptor: Vec<u32>,
    out_colour: Vec<u8>,
    pre_slots: Vec<(u32, u32)>,
    keep: Vec<Bond>,
    /// The joint-move pass's claim on a particle, indexed by its PRE-shift slot: the
    /// direction it is displaced by once a joint move has taken it, [`NO_ORIENT`] while it
    /// is still on its own plane's shift. Cleared through `claimed`, so the pass costs the
    /// bonds and not the lattice.
    claim: Vec<u8>,
    claimed: Vec<u32>,
    order: Vec<u32>,
    blocked_idx: Vec<u32>,
}

impl OrientationLattice {
    /// Wrap an already-seeded [`Lattice`]: the occupation is that lattice's, bit for bit, so
    /// FLUID-0's own seeding routines are the only ones in the path and the no-bond control
    /// can be an IDENTITY rather than an agreement.
    ///
    /// Orientations are drawn uniformly over the six directions from a counter hash of
    /// `(seed, cell, dir)` on this module's own key.
    pub fn from_lattice(tables: Lattice, orient_seed: u64, rules: OrientationRules) -> Self {
        assert_eq!(tables.model.n_dirs(), N_DIRS, "the orientation lattice is defined on FHP-6");
        assert!(
            tables.solid.iter().all(|&s| !s),
            "the orientation lattice carries no wall: the bond rule and the bounce-back have \
             not been defined together and a silent wall would be a channel outside the ledger"
        );
        let l = tables.l;
        let n = l * l;
        let cells = tables.cells.clone();
        let mut orient = vec![NO_ORIENT; n * N_DIRS];
        for (c, &s) in cells.iter().enumerate() {
            for d in 0..N_DIRS {
                if s >> d & 1 == 1 {
                    let h = mix64(
                        ORIENT_SEED_KEY
                            ^ orient_seed
                            ^ (c as u64).wrapping_mul(GOLDEN)
                            ^ ((d as u64) << 56),
                    );
                    orient[c * N_DIRS + d] = (h % N_DIRS as u64) as u8;
                }
            }
        }
        let mut me = Self {
            tables,
            l,
            cells,
            orient,
            donor_of: vec![NO_BOND; n * N_DIRS],
            acceptor_of: vec![NO_BOND; n * N_DIRS],
            bonds: Vec::new(),
            colour: None,
            rules,
            step_index: 0,
            counts: StepCounts::default(),
            audit: OrientationAudit {
                mass_exact: true,
                px_exact: true,
                py_exact: true,
                red_exact: true,
                orientation_total_exact: true,
                orientation_census_exact: true,
                bond_balance_exact: true,
                steps_checked: 0,
                mass: 0,
                px: 0,
                py: 0,
                red: 0,
                orientation_total: 0,
                bonds_final: 0,
                totals: StepCounts::default(),
            },
            audit_every_step: true,
            initial: OrientationLedger {
                mass: 0,
                momentum: [0, 0],
                census: [0; N_DIRS],
                census_total: 0,
                red: 0,
                bonds: 0,
            },
            out_cells: vec![0u8; n],
            out_orient: vec![NO_ORIENT; n * N_DIRS],
            out_donor: vec![NO_BOND; n * N_DIRS],
            out_acceptor: vec![NO_BOND; n * N_DIRS],
            out_colour: vec![0u8; n],
            pre_slots: Vec::new(),
            keep: Vec::new(),
            claim: vec![NO_ORIENT; n * N_DIRS],
            claimed: Vec::new(),
            order: Vec::new(),
            blocked_idx: Vec::new(),
        };
        me.initial = me.ledger();
        me.audit = OrientationAudit::fresh(&me.initial);
        me
    }

    /// [`OrientationLattice::from_lattice`] with a colour plane attached — the tracer readout's
    /// state. The plane is FLUID-0's own [`Lattice::seed_colour_wave`] output.
    pub fn with_colour(mut self, colour: Vec<u8>) -> Self {
        assert_eq!(colour.len(), self.cells.len());
        assert!(
            colour.iter().zip(&self.cells).all(|(&q, &s)| q & !s == 0),
            "a colour bit without a particle under it"
        );
        self.colour = Some(colour);
        self.initial = self.ledger();
        self.audit = OrientationAudit::fresh(&self.initial);
        self
    }

    /// Re-baseline the ledger to the CURRENT state and clear the audit.
    ///
    /// Two callers need it and neither is a shortcut: a scene built by writing cells and
    /// orientations directly (G3's held geometry) has no ledger until it is built, and a
    /// reading taken after a warm-up wants the warm-up's counts out of its sample. The
    /// conserved integers are conserved from whatever instant the baseline is taken at, so
    /// re-baselining weakens nothing — it moves the start of the window, not the check.
    pub fn reset_initial(&mut self) {
        self.initial = self.ledger();
        self.audit = OrientationAudit::fresh(&self.initial);
        self.counts = StepCounts::default();
    }

    pub fn model(&self) -> &Model {
        &self.tables.model
    }

    /// The table-holding lattice, for the readouts that already exist on it
    /// ([`Lattice::line_momenta_of`], [`Lattice::ledger_of`]).
    pub fn tables(&self) -> &Lattice {
        &self.tables
    }

    #[inline]
    fn neighbour(&self, cell: usize, dir: usize) -> usize {
        self.tables.neighbour_of(cell, dir)
    }

    #[inline]
    fn opposite(&self, dir: usize) -> usize {
        (dir + N_DIRS / 2) % N_DIRS
    }

    /// Every conserved integer right now.
    pub fn ledger(&self) -> OrientationLedger {
        let led = self.tables.ledger_of(&self.cells);
        let mut census = [0i64; N_DIRS];
        for &o in &self.orient {
            if (o as usize) < N_DIRS {
                census[o as usize] += 1;
            }
        }
        let red = match &self.colour {
            Some(q) => q.iter().map(|&b| b.count_ones() as i64).sum(),
            None => 0,
        };
        OrientationLedger {
            mass: led.mass,
            momentum: led.momentum,
            census,
            census_total: census.iter().sum(),
            red,
            bonds: self.bonds.len() as i64,
        }
    }

    /// The bond's link direction and the acceptor's angle index, DERIVED from the state.
    ///
    /// `δ` is the donor's own orientation — the donor arm points along the link by the
    /// formation rule and a bonded particle keeps its orientation, so the two cannot drift
    /// apart. `Some((δ, φ/60°))` when the recorded link is intact, `None` when it is not,
    /// which is how the streaming phase detects a pair it could not move as one.
    pub fn bond_geometry(&self, b: &Bond) -> Option<(usize, usize)> {
        let (dc, dd) = (b.donor as usize / N_DIRS, b.donor as usize % N_DIRS);
        let (ac, ad) = (b.acceptor as usize / N_DIRS, b.acceptor as usize % N_DIRS);
        if self.cells[dc] >> dd & 1 == 0 || self.cells[ac] >> ad & 1 == 0 {
            return None;
        }
        let delta = self.orient[b.donor as usize] as usize;
        if delta >= N_DIRS || self.neighbour(dc, delta) != ac {
            return None;
        }
        let oa = self.orient[b.acceptor as usize] as usize;
        if oa >= N_DIRS {
            return None;
        }
        let phi = (oa + N_DIRS - self.opposite(delta)) % N_DIRS;
        Some((delta, phi))
    }

    /// Rebuild the two role tables from the pair list. Called after every pass that changes
    /// the list, so an index can never point at a bond that has moved.
    fn reindex(&mut self) {
        for (i, b) in self.bonds.iter().enumerate() {
            self.donor_of[b.donor as usize] = i as u32;
            self.acceptor_of[b.acceptor as usize] = i as u32;
        }
    }

    /// One step: collide, form, break, stream. The freeze's order, with the two clauses this
    /// module declares (a bond pays no rent on the step it forms; a joint move needs a
    /// vacancy).
    pub fn step(&mut self) {
        self.counts = StepCounts::default();
        let held_before = self.bonds.len() as i64;

        self.collide();
        let n_start = self.bonds.len();
        if self.rules.bonds_enabled {
            self.form();
        }
        self.break_bonds(n_start);
        if self.rules.stream {
            self.stream();
        }
        self.step_index += 1;

        let counts = self.counts;
        self.audit.totals.add(&counts);
        let held_after = self.bonds.len() as i64;
        let balance = held_before + counts.formed as i64
            - counts.broken_rent as i64
            - counts.blocked as i64;
        self.audit.bond_balance_exact &= balance == held_after;
        self.audit.bonds_final = held_after;
        if self.audit_every_step {
            let led = self.ledger();
            self.audit.mass_exact &= led.mass == self.initial.mass;
            self.audit.px_exact &= led.momentum[0] == self.initial.momentum[0];
            self.audit.py_exact &= led.momentum[1] == self.initial.momentum[1];
            self.audit.red_exact &= led.red == self.initial.red;
            self.audit.orientation_total_exact &= led.census_total == self.initial.census_total;
            self.audit.orientation_census_exact &= led.census == self.initial.census;
            self.audit.orientation_total = led.census_total;
            self.audit.steps_checked += 1;
        }
    }

    // ------------------------------------------------------------------ (1) collision
    fn collide(&mut self) {
        let step = self.step_index;
        let mut fired = 0u64;
        for c in 0..self.cells.len() {
            let s = self.cells[c];
            let t = self.tables.collision_image(c, s, step);
            if t == s {
                continue;
            }
            fired += 1;
            if let Some(q) = &mut self.colour {
                q[c] = recolour(s, t, q[c], N_DIRS, ColourRule::Blind, c, step);
            }
            self.remap_cell(c, s, t, step);
            self.cells[c] = t;
        }
        self.counts.collisions_fired = fired;
    }

    /// Re-seat one cell's particles from the occupied slots of `s` onto those of `t`:
    /// bonded first, ascending onto ascending, keeping orientation and both roles; the free
    /// ones onto the rest, their orientations shuffled among themselves.
    fn remap_cell(&mut self, c: usize, s: u8, t: u8, step: u64) {
        let base = c * N_DIRS;
        let mut ins = [0usize; N_DIRS];
        let mut n_in = 0usize;
        for d in 0..N_DIRS {
            if s >> d & 1 == 1 {
                ins[n_in] = d;
                n_in += 1;
            }
        }
        let mut outs = [0usize; N_DIRS];
        let mut n_out = 0usize;
        for d in 0..N_DIRS {
            if t >> d & 1 == 1 {
                outs[n_out] = d;
                n_out += 1;
            }
        }
        debug_assert_eq!(n_in, n_out, "a sector-preserving law changed the occupancy");

        // Read the cell out, then clear it, then write the new seating: the source and the
        // destination slots overlap, so an in-place walk would overwrite its own input.
        let mut o_in = [NO_ORIENT; N_DIRS];
        let mut d_in = [NO_BOND; N_DIRS];
        let mut a_in = [NO_BOND; N_DIRS];
        for k in 0..n_in {
            let sl = base + ins[k];
            o_in[k] = self.orient[sl];
            d_in[k] = self.donor_of[sl];
            a_in[k] = self.acceptor_of[sl];
            self.orient[sl] = NO_ORIENT;
            self.donor_of[sl] = NO_BOND;
            self.acceptor_of[sl] = NO_BOND;
        }

        let mut seat = [0usize; N_DIRS]; // which incoming index sits on outs[j]
        let mut free_o = [NO_ORIENT; N_DIRS];
        let mut n_free = 0usize;
        let mut j = 0usize;
        for k in 0..n_in {
            if d_in[k] != NO_BOND || a_in[k] != NO_BOND {
                seat[j] = k;
                j += 1;
            } else {
                free_o[n_free] = o_in[k];
                n_free += 1;
            }
        }
        let n_bonded = j;
        // The free orientations, shuffled among themselves: a partial Fisher–Yates off a
        // counter hash of (cell, step) alone, so the result depends on no traversal order.
        if n_free > 1 {
            let seed = mix64(ORIENT_MIX_KEY ^ (c as u64).wrapping_mul(GOLDEN) ^ step.rotate_left(32));
            for k in 0..n_free - 1 {
                let h = mix64(seed ^ ((k as u64) << 40) ^ 0x1234_5678_9ABC_DEF0);
                let pick = k + (h % (n_free - k) as u64) as usize;
                free_o.swap(k, pick);
            }
        }
        let mut f = 0usize;
        for jj in 0..n_out {
            let sl = base + outs[jj];
            if jj < n_bonded {
                let k = seat[jj];
                self.orient[sl] = o_in[k];
                if d_in[k] != NO_BOND {
                    self.donor_of[sl] = d_in[k];
                    self.bonds[d_in[k] as usize].donor = sl as u32;
                }
                if a_in[k] != NO_BOND {
                    self.acceptor_of[sl] = a_in[k];
                    self.bonds[a_in[k] as usize].acceptor = sl as u32;
                }
            } else {
                self.orient[sl] = free_o[f];
                f += 1;
            }
        }
    }

    // ------------------------------------------------------------------ (2) formation
    fn form(&mut self) {
        let mut formed = 0u64;
        for c in 0..self.cells.len() {
            let s = self.cells[c];
            if s == 0 {
                continue;
            }
            for a in 0..N_DIRS {
                if s >> a & 1 == 0 {
                    continue;
                }
                let ds = c * N_DIRS + a;
                if self.donor_of[ds] != NO_BOND {
                    continue;
                }
                let delta = self.orient[ds] as usize;
                debug_assert!(delta < N_DIRS, "an occupied slot without an orientation");
                let j = self.neighbour(c, delta);
                let sj = self.cells[j];
                if sj == 0 {
                    continue;
                }
                for b in 0..N_DIRS {
                    if sj >> b & 1 == 0 {
                        continue;
                    }
                    let as_ = j * N_DIRS + b;
                    if self.acceptor_of[as_] != NO_BOND {
                        continue;
                    }
                    let idx = self.bonds.len() as u32;
                    self.bonds.push(Bond { donor: ds as u32, acceptor: as_ as u32 });
                    self.donor_of[ds] = idx;
                    self.acceptor_of[as_] = idx;
                    formed += 1;
                    break;
                }
            }
        }
        self.counts.formed = formed;
    }

    // ------------------------------------------------------------------ (3) the rent
    /// Only the first `n_start` bonds — those that existed at the start of the step — pay.
    fn break_bonds(&mut self, n_start: usize) {
        if self.bonds.is_empty() {
            return;
        }
        let step = self.step_index;
        let mut keep = core::mem::take(&mut self.keep);
        keep.clear();
        keep.reserve(self.bonds.len());
        let bonds = core::mem::take(&mut self.bonds);
        for (i, b) in bonds.iter().enumerate() {
            if i >= n_start {
                keep.push(*b);
                continue;
            }
            let phi = match self.bond_geometry(b) {
                Some((_, phi)) => phi,
                // Unreachable by construction: a held bond's geometry is invariant. Kept as a
                // branch rather than an assert so a defect shows up as a released bond in the
                // balance instead of a panic in a twelve-hour run.
                None => {
                    self.donor_of[b.donor as usize] = NO_BOND;
                    self.acceptor_of[b.acceptor as usize] = NO_BOND;
                    self.counts.blocked += 1;
                    self.counts.blocked_no_geometry += 1;
                    continue;
                }
            };
            self.counts.break_tests[phi] += 1;
            let p = self.rules.p_break(phi);
            let h = mix64(
                BREAK_KEY ^ (b.donor as u64).wrapping_mul(GOLDEN) ^ step.rotate_left(32),
            );
            if unit(h) < p {
                self.donor_of[b.donor as usize] = NO_BOND;
                self.acceptor_of[b.acceptor as usize] = NO_BOND;
                self.counts.broken_rent += 1;
                self.counts.broken_by_phi[phi] += 1;
            } else {
                keep.push(*b);
            }
        }
        self.bonds = keep;
        self.keep = bonds;
        self.reindex();
    }

    // ------------------------------------------------------------------ (4) streaming
    fn stream(&mut self) {
        let n = self.cells.len();
        self.out_cells.fill(0);
        self.out_orient.fill(NO_ORIENT);
        self.out_donor.fill(NO_BOND);
        self.out_acceptor.fill(NO_BOND);
        let carry_colour = self.colour.is_some();
        if carry_colour {
            self.out_colour.fill(0);
        }

        // Pass 1 — the plain shift. A bijection on every plane, so mass and momentum are
        // already exact before any bond is consulted.
        for c in 0..n {
            let s = self.cells[c];
            if s == 0 {
                continue;
            }
            let q = self.colour.as_ref().map(|q| q[c]).unwrap_or(0);
            for d in 0..N_DIRS {
                if s >> d & 1 == 0 {
                    continue;
                }
                let dst = self.neighbour(c, d);
                self.out_cells[dst] |= 1 << d;
                let from = c * N_DIRS + d;
                let to = dst * N_DIRS + d;
                self.out_orient[to] = self.orient[from];
                self.out_donor[to] = self.donor_of[from];
                self.out_acceptor[to] = self.acceptor_of[from];
                if carry_colour && q >> d & 1 == 1 {
                    self.out_colour[dst] |= 1 << d;
                }
            }
        }

        // Every bond's slots, before and after the shift. The pre-shift pair is what the
        // anomalous target is computed from.
        let mut pre = core::mem::take(&mut self.pre_slots);
        pre.clear();
        pre.reserve(self.bonds.len());
        for b in &self.bonds {
            pre.push((b.donor, b.acceptor));
        }
        for b in self.bonds.iter_mut() {
            let (dc, dd) = (b.donor as usize / N_DIRS, b.donor as usize % N_DIRS);
            let (ac, ad) = (b.acceptor as usize / N_DIRS, b.acceptor as usize % N_DIRS);
            b.donor = (self.tables.neighbour_of(dc, dd) * N_DIRS + dd) as u32;
            b.acceptor = (self.tables.neighbour_of(ac, ad) * N_DIRS + ad) as u32;
        }

        // Pass 2 — the joint moves, over the BONDS in ascending post-shift donor slot. The
        // order is computed once, before any hop: walking the role table live would revisit
        // a bond whose donor an earlier hop had moved forward in the table and process it a
        // second time.
        let step = self.step_index;
        let mut order = core::mem::take(&mut self.order);
        order.clear();
        order.extend(0..self.bonds.len() as u32);
        order.sort_unstable_by_key(|&bi| self.bonds[bi as usize].donor);
        let mut blocked_idx = core::mem::take(&mut self.blocked_idx);
        blocked_idx.clear();
        let mut claimed = core::mem::take(&mut self.claimed);
        claimed.clear();

        for &bi in order.iter() {
            let bi = bi as usize;
            let (pre_d, pre_a) = pre[bi];
            let (i, a) = (pre_d as usize / N_DIRS, pre_d as usize % N_DIRS);
            let (j, bd) = (pre_a as usize / N_DIRS, pre_a as usize % N_DIRS);
            self.counts.joint_moves += 1;
            // The mover, keyed on the PRE-shift donor slot so the draw does not depend on the
            // pass order. When both labels agree the plain shift already moved the pair as
            // one and there is nothing to draw.
            let m = if a == bd {
                a
            } else {
                let h =
                    mix64(MOVER_KEY ^ (pre_d as u64).wrapping_mul(GOLDEN) ^ step.rotate_left(32));
                if h & 1 == 0 {
                    a
                } else {
                    bd
                }
            };
            // Where each of the two must end up is `its pre-shift cell + dirs[m]`. A particle
            // already CLAIMED at `m` by an earlier joint move is there; one claimed at any
            // other direction is a conflict and refuses this pair; an unclaimed one whose own
            // label is not `m` has to leave its own plane's shift. At most one hop follows,
            // because `m` is one of the two labels.
            let mut conflict = false;
            let mut hop: Option<(usize, usize, usize)> = None;
            for (p, cellp, lp) in [(pre_d as usize, i, a), (pre_a as usize, j, bd)] {
                let c = self.claim[p];
                if c == m as u8 {
                    continue;
                }
                if c != NO_ORIENT {
                    conflict = true;
                    break;
                }
                if lp != m {
                    debug_assert!(hop.is_none(), "two hops for one joint move");
                    hop = Some((self.neighbour(cellp, lp), self.neighbour(cellp, m), lp));
                }
            }
            if conflict {
                blocked_idx.push(bi as u32);
                self.counts.blocked_claimed += 1;
                continue;
            }
            if let Some((src_cell, tgt_cell, lane)) = hop {
                if self.out_cells[tgt_cell] >> lane & 1 == 1 {
                    // No vacancy: the pair cannot move as one, both stream normally, the bond
                    // is released as `blocked` — counted apart from a rent break, never inside
                    // it.
                    blocked_idx.push(bi as u32);
                    self.counts.blocked_no_vacancy += 1;
                    continue;
                }
                debug_assert_eq!(
                    self.out_cells[src_cell] >> lane & 1,
                    1,
                    "the non-mover is not where the shift put it"
                );
                self.counts.anomalous_hops += 1;
                let from = src_cell * N_DIRS + lane;
                let to = tgt_cell * N_DIRS + lane;
                self.out_cells[tgt_cell] |= 1 << lane;
                self.out_cells[src_cell] &= !(1 << lane);
                self.out_orient[to] = self.out_orient[from];
                // BOTH roles travel with the particle, and BOTH bonds' recorded slots are
                // rewritten: a hop that rewrote only the bond it was processing left the other
                // bond pointing at the slot the particle had left.
                let moved_d = self.out_donor[from];
                let moved_a = self.out_acceptor[from];
                self.out_donor[to] = moved_d;
                self.out_acceptor[to] = moved_a;
                self.out_orient[from] = NO_ORIENT;
                self.out_donor[from] = NO_BOND;
                self.out_acceptor[from] = NO_BOND;
                if moved_d != NO_BOND {
                    self.bonds[moved_d as usize].donor = to as u32;
                }
                if moved_a != NO_BOND {
                    self.bonds[moved_a as usize].acceptor = to as u32;
                }
                if carry_colour {
                    let bit = self.out_colour[src_cell] >> lane & 1;
                    self.out_colour[src_cell] &= !(1 << lane);
                    self.out_colour[tgt_cell] |= bit << lane;
                }
            }
            for p in [pre_d as usize, pre_a as usize] {
                if self.claim[p] == NO_ORIENT {
                    self.claim[p] = m as u8;
                    claimed.push(p as u32);
                }
            }
        }
        for &p in claimed.iter() {
            self.claim[p as usize] = NO_ORIENT;
        }
        self.claimed = claimed;
        self.order = order;
        self.pre_slots = pre;

        // Release the refused pairs. Their two particles are exactly where the shift (or an
        // earlier joint move that claimed one of them) put them, which is what "both stream
        // normally" means. Each bond is visited exactly once, so this list carries no
        // duplicate and the balance counts each release once.
        if !blocked_idx.is_empty() {
            self.counts.blocked += blocked_idx.len() as u64;
            for &bi in blocked_idx.iter() {
                let b = self.bonds[bi as usize];
                self.out_donor[b.donor as usize] = NO_BOND;
                self.out_acceptor[b.acceptor as usize] = NO_BOND;
            }
            let mut dead = vec![false; self.bonds.len()];
            for &bi in blocked_idx.iter() {
                dead[bi as usize] = true;
            }
            let mut kept = Vec::with_capacity(self.bonds.len() - blocked_idx.len());
            for (i, b) in self.bonds.iter().enumerate() {
                if !dead[i] {
                    kept.push(*b);
                }
            }
            self.bonds = kept;
        }
        self.blocked_idx = blocked_idx;

        core::mem::swap(&mut self.cells, &mut self.out_cells);
        core::mem::swap(&mut self.orient, &mut self.out_orient);
        core::mem::swap(&mut self.donor_of, &mut self.out_donor);
        core::mem::swap(&mut self.acceptor_of, &mut self.out_acceptor);
        if carry_colour {
            let mut q = self.colour.take().unwrap();
            core::mem::swap(&mut q, &mut self.out_colour);
            self.colour = Some(q);
        }
        self.reindex();
    }

    // ------------------------------------------------------------------ readouts
    /// The bond graph as a number: bonds, particles, the largest connected component and
    /// whether it winds around the torus.
    pub fn bond_graph(&self) -> BondGraph {
        bond_graph(self)
    }

    /// The whole mutable state, for a probe that restores a base configuration.
    pub fn snapshot(&self) -> OrientationState {
        OrientationState {
            cells: self.cells.clone(),
            orient: self.orient.clone(),
            donor_of: self.donor_of.clone(),
            acceptor_of: self.acceptor_of.clone(),
            bonds: self.bonds.clone(),
            colour: self.colour.clone(),
            step_index: self.step_index,
        }
    }

    /// Restore a snapshot into the buffers this lattice already holds.
    pub fn restore(&mut self, s: &OrientationState) {
        self.cells.copy_from_slice(&s.cells);
        self.orient.copy_from_slice(&s.orient);
        self.donor_of.copy_from_slice(&s.donor_of);
        self.acceptor_of.copy_from_slice(&s.acceptor_of);
        self.bonds.clear();
        self.bonds.extend_from_slice(&s.bonds);
        match (&mut self.colour, &s.colour) {
            (Some(a), Some(b)) => a.copy_from_slice(b),
            (a, b) => *a = b.clone(),
        }
        self.step_index = s.step_index;
        self.counts = StepCounts::default();
    }

    /// The fiber move of `probe.rs`, on the orientation state: the cell's local state goes to
    /// its cyclic successor within its own `(N,P)` fiber, and the cell's particles are re-seated
    /// on the new slots in ascending order, each keeping its orientation and both of its roles.
    ///
    /// The label is unchanged, so `v_b` is unchanged for EVERY `b` at once; the orientation
    /// multiset and the whole bond table are unchanged too, so the perturbation is the
    /// occupation byte and nothing else.
    pub fn fiber_perturb(&mut self, cell: usize) -> bool {
        let s = self.cells[cell];
        let t = match self.tables.model.fiber_successor(s) {
            Some(t) => t,
            None => return false,
        };
        let base = cell * N_DIRS;
        let mut o_in = [NO_ORIENT; N_DIRS];
        let mut d_in = [NO_BOND; N_DIRS];
        let mut a_in = [NO_BOND; N_DIRS];
        let mut k = 0usize;
        for d in 0..N_DIRS {
            if s >> d & 1 == 1 {
                o_in[k] = self.orient[base + d];
                d_in[k] = self.donor_of[base + d];
                a_in[k] = self.acceptor_of[base + d];
                self.orient[base + d] = NO_ORIENT;
                self.donor_of[base + d] = NO_BOND;
                self.acceptor_of[base + d] = NO_BOND;
                k += 1;
            }
        }
        let mut colour_bits = 0u8;
        let mut j = 0usize;
        for d in 0..N_DIRS {
            if t >> d & 1 == 1 {
                self.orient[base + d] = o_in[j];
                if d_in[j] != NO_BOND {
                    self.donor_of[base + d] = d_in[j];
                    self.bonds[d_in[j] as usize].donor = (base + d) as u32;
                }
                if a_in[j] != NO_BOND {
                    self.acceptor_of[base + d] = a_in[j];
                    self.bonds[a_in[j] as usize].acceptor = (base + d) as u32;
                }
                colour_bits |= 1 << d;
                j += 1;
            }
        }
        if let Some(q) = &mut self.colour {
            // Colour is a subset of occupancy; the perturbation moves the particles, so it
            // moves their colour with them rather than leaving a bit over a hole.
            let red = q[cell].count_ones();
            let mut r = 0u8;
            let mut left = red;
            for d in 0..N_DIRS {
                if left == 0 {
                    break;
                }
                if colour_bits >> d & 1 == 1 {
                    r |= 1 << d;
                    left -= 1;
                }
            }
            q[cell] = r;
        }
        self.cells[cell] = t;
        true
    }
}

// ---------------------------------------------------------------- the bond graph
/// The bond graph's reading: its largest connected component and whether that component winds
/// around the torus.
///
/// **Spanning, defined.** A component spans when it carries a NON-CONTRACTIBLE cycle: walking
/// its bonds around a loop returns to the starting cell with a net lattice displacement of
/// `L` times a nonzero integer vector. That is measured with a union–find carrying, per
/// particle, its position relative to its component's root; an edge closing a cycle whose two
/// endpoints disagree by `w ≠ 0` exhibits the winding, and `w` is always `L·(m, n)`. The
/// weaker "touches both edges of the box" reading is reported beside as `touches_all_rows` /
/// `touches_all_columns`, because it is not invariant under translating the configuration and
/// a gate that moves when the scene is shifted is not a gate.
#[derive(Clone, Debug)]
pub struct BondGraph {
    pub particles: usize,
    pub bonds: usize,
    pub largest: usize,
    pub largest_fraction: f64,
    /// The largest component winds in the first / second axial direction.
    pub largest_winds: [bool; 2],
    pub largest_spans: bool,
    /// ANY component winds — reported beside, since the largest is the one the freeze asks for.
    pub any_spans: bool,
    /// The weaker, translation-dependent reading, reported and never gated.
    pub largest_touches_all_rows: bool,
    pub largest_touches_all_columns: bool,
    /// Mean degree `2B/N`, the freeze's "bonds per particle" in `[0, 2]`.
    pub bonds_per_particle_degree: f64,
    /// `B/N`, the convention LIQUID-1's `1.1843` is quoted in (each bond once, at its donor).
    /// Structurally capped at `1.0` here, because a lattice particle has ONE donor arm where a
    /// water has two.
    pub bonds_per_particle_donor: f64,
}

fn uf_find(parent: &mut [u32], pot: &mut [[i32; 2]], x: u32) -> (u32, [i32; 2]) {
    let mut root = x;
    let mut acc = [0i32; 2];
    while parent[root as usize] != root {
        acc[0] += pot[root as usize][0];
        acc[1] += pot[root as usize][1];
        root = parent[root as usize];
    }
    let mut cur = x;
    let mut cacc = acc;
    while parent[cur as usize] != cur {
        let next = parent[cur as usize];
        let p = pot[cur as usize];
        parent[cur as usize] = root;
        pot[cur as usize] = cacc;
        cacc[0] -= p[0];
        cacc[1] -= p[1];
        cur = next;
    }
    (root, acc)
}

/// The union–find of the note on [`BondGraph`], written here rather than imported: the lens's
/// `largest_domain` is a size-only routine over an abstract edge set, and this crate's
/// dependency profile is deliberately ONE runtime crate (`Cargo.toml` says why), so pulling
/// `holon-lens` in for twenty lines would change the isolation argument. The winding is not in
/// the lens's routine at all.
pub fn bond_graph(g: &OrientationLattice) -> BondGraph {
    let l = g.l as i32;
    let n_slots = g.cells.len() * N_DIRS;
    let mut parent: Vec<u32> = (0..n_slots as u32).collect();
    let mut pot = vec![[0i32; 2]; n_slots];
    let dirs: Vec<[i32; 2]> = g.model().dirs.iter().map(|d| [d[0] as i32, d[1] as i32]).collect();

    let edges: Vec<(u32, u32, [i32; 2])> = g
        .bonds
        .iter()
        .filter_map(|b| {
            let (delta, _) = g.bond_geometry(b)?;
            Some((b.donor, b.acceptor, dirs[delta]))
        })
        .collect();

    for &(u, v, disp) in &edges {
        let (ru, au) = uf_find(&mut parent, &mut pot, u);
        let (rv, av) = uf_find(&mut parent, &mut pot, v);
        if ru != rv {
            parent[rv as usize] = ru;
            pot[rv as usize] = [disp[0] + au[0] - av[0], disp[1] + au[1] - av[1]];
        }
    }

    let mut size = vec![0u32; n_slots];
    let mut occupied = 0usize;
    for c in 0..g.cells.len() {
        let s = g.cells[c];
        for d in 0..N_DIRS {
            if s >> d & 1 == 1 {
                occupied += 1;
                let (r, _) = uf_find(&mut parent, &mut pot, (c * N_DIRS + d) as u32);
                size[r as usize] += 1;
            }
        }
    }
    let mut best_root = 0u32;
    let mut best = 0u32;
    for (r, &sz) in size.iter().enumerate() {
        if sz > best {
            best = sz;
            best_root = r as u32;
        }
    }

    // Second pass for the windings, once every root is final.
    let mut winds_root: Vec<[bool; 2]> = vec![[false; 2]; n_slots];
    let mut any = false;
    for &(u, v, disp) in &edges {
        let (ru, au) = uf_find(&mut parent, &mut pot, u);
        let (rv, av) = uf_find(&mut parent, &mut pot, v);
        debug_assert_eq!(ru, rv, "an edge whose endpoints are in two components");
        let w = [au[0] + disp[0] - av[0], au[1] + disp[1] - av[1]];
        if w != [0, 0] {
            debug_assert_eq!([w[0] % l, w[1] % l], [0, 0], "a winding that is not a multiple of L");
            any = true;
            winds_root[ru as usize][0] |= w[0] != 0;
            winds_root[ru as usize][1] |= w[1] != 0;
        }
        let _ = rv;
    }

    // The translation-dependent reading, reported beside.
    let mut rows = vec![false; g.l];
    let mut cols = vec![false; g.l];
    if best > 1 {
        for c in 0..g.cells.len() {
            let s = g.cells[c];
            for d in 0..N_DIRS {
                if s >> d & 1 == 1 {
                    let (r, _) = uf_find(&mut parent, &mut pot, (c * N_DIRS + d) as u32);
                    if r == best_root {
                        cols[c / g.l] = true;
                        rows[c % g.l] = true;
                    }
                }
            }
        }
    }

    let winds = if best > 1 { winds_root[best_root as usize] } else { [false, false] };
    let particles = occupied;
    BondGraph {
        particles,
        bonds: edges.len(),
        largest: best as usize,
        largest_fraction: if particles > 0 { best as f64 / particles as f64 } else { f64::NAN },
        largest_winds: winds,
        largest_spans: winds[0] || winds[1],
        any_spans: any,
        largest_touches_all_rows: rows.iter().all(|&x| x),
        largest_touches_all_columns: cols.iter().all(|&x| x),
        bonds_per_particle_degree: if particles > 0 {
            2.0 * edges.len() as f64 / particles as f64
        } else {
            f64::NAN
        },
        bonds_per_particle_donor: if particles > 0 {
            edges.len() as f64 / particles as f64
        } else {
            f64::NAN
        },
    }
}

// ---------------------------------------------------------------- the transport readouts
fn sine_table(l: usize, k_index: usize) -> Vec<f64> {
    let w = 2.0 * PI * k_index as f64 / l as f64;
    (0..l).map(|m| (w * m as f64).sin()).collect()
}

fn project_momenta(g: &OrientationLattice, sine: &[f64], along_columns: bool) -> f64 {
    let lm = g.tables.line_momenta_of(&g.cells);
    let off = if along_columns { g.l } else { 0 };
    (0..g.l).map(|m| lm[off + m] as f64 * sine[m]).sum()
}

fn project_colour_excess(g: &OrientationLattice, sine: &[f64]) -> f64 {
    let q = g.colour.as_ref().expect("a colour projection on a lattice with no colour plane");
    let mut rows = vec![0.0f64; g.l];
    for (c, (&s, &qq)) in g.cells.iter().zip(q).enumerate() {
        rows[c % g.l] += qq.count_ones() as f64 - 0.5 * s.count_ones() as f64;
    }
    (0..g.l).map(|m| rows[m] * sine[m]).sum()
}

/// Advance an ensemble in lockstep, projecting the members' mean, and fit it under the given
/// rule — [`crate::transport::shear_ensemble`]'s shape, on this module's motion.
fn ensemble_run(
    members: &mut [OrientationLattice],
    cap: usize,
    rule: &FitRule,
    k: f64,
    project: impl Fn(&OrientationLattice) -> f64,
) -> (TransportReading, OrientationAudit) {
    assert!(!members.is_empty(), "an ensemble needs at least one member");
    let mean = |ms: &[OrientationLattice]| -> f64 {
        ms.iter().map(&project).sum::<f64>() / ms.len() as f64
    };
    let mut series = Vec::with_capacity(cap + 1);
    series.push(mean(members));
    let mut threshold = f64::NAN;
    for t in 0..cap {
        for m in members.iter_mut() {
            m.step();
        }
        series.push(mean(members));
        if t + 1 == rule.start {
            threshold = (rule.end_fraction * series[rule.start].abs()).max(rule.floor);
        }
        if t + 1 > rule.start && series[t + 1].abs() < threshold {
            break;
        }
    }
    let mut audit = members[0].audit;
    for m in members.iter().skip(1) {
        audit.merge(&m.audit);
    }
    let fired = audit.totals.collisions_fired;
    (fit_with(&series, rule, k, fired), audit)
}

/// The shear viscosity of the orientation lattice: FLUID-0's seeding, FLUID-0's readout,
/// FLUID-0's fit rule, this module's motion.
#[allow(clippy::too_many_arguments)]
pub fn shear_ensemble_oriented(
    model: &Model,
    law: &[u8],
    l: usize,
    d: f64,
    a: f64,
    k_index: usize,
    cap: usize,
    seed: u64,
    seeds: usize,
    along_columns: bool,
    drive: [f64; 2],
    rule: &FitRule,
    rules: &OrientationRules,
) -> (TransportReading, OrientationAudit) {
    let mut members: Vec<OrientationLattice> = (0..seeds)
        .map(|s| {
            let sd = crate::transport::ensemble_seed(seed, s);
            let lat = Lattice::seeded_shear_drive(
                model.clone(),
                l,
                sd,
                d,
                a,
                k_index,
                law.to_vec(),
                drive,
                along_columns,
            );
            OrientationLattice::from_lattice(lat, sd, *rules)
        })
        .collect();
    let sine = sine_table(l, k_index);
    let k = crate::transport::wavenumber(l, k_index);
    ensemble_run(&mut members, cap, rule, k, |g| project_momenta(g, &sine, along_columns))
}

/// The tracer diffusivity of the orientation lattice, as above.
#[allow(clippy::too_many_arguments)]
pub fn colour_ensemble_oriented(
    model: &Model,
    law: &[u8],
    l: usize,
    d: f64,
    a: f64,
    k_index: usize,
    cap: usize,
    seed: u64,
    seeds: usize,
    rule: &FitRule,
    rules: &OrientationRules,
) -> (TransportReading, OrientationAudit) {
    let mut members: Vec<OrientationLattice> = (0..seeds)
        .map(|s| {
            let sd = crate::transport::ensemble_seed(seed, s);
            let lat = Lattice::seeded(model.clone(), l, sd, d, law.to_vec());
            let colour = lat.seed_colour_wave(sd, a, k_index);
            OrientationLattice::from_lattice(lat, sd, *rules).with_colour(colour)
        })
        .collect();
    let sine = sine_table(l, k_index);
    let k = crate::transport::wavenumber(l, k_index);
    ensemble_run(&mut members, cap, rule, k, |g| project_colour_excess(g, &sine))
}

// ---------------------------------------------------------------- node LG's probe, here
/// Node LG's block-chart witness rate, re-measured on the orientation lattice's OCCUPATION
/// bytes: one fiber move per movable cell, one step of THIS motion, the chart compared.
///
/// The population is `probe.rs`'s `AsConfigured` — every movable cell exactly once, keeping
/// its own state, enumerated and never sampled (an enumerated count is not an effective count,
/// and a forward-scanning sampler biases the position inside the block, which is the one
/// quantity the defect law is about).
#[derive(Clone, Debug)]
pub struct WitnessReading {
    pub b: usize,
    pub probes: u64,
    pub witnesses: u64,
    pub predicted: f64,
}

impl WitnessReading {
    pub fn rate(&self) -> f64 {
        if self.probes == 0 {
            f64::NAN
        } else {
            self.witnesses as f64 / self.probes as f64
        }
    }
    pub fn stderr(&self) -> f64 {
        if self.probes == 0 {
            return f64::NAN;
        }
        let p = self.rate();
        (p * (1.0 - p) / self.probes as f64).sqrt().max(1.0 / self.probes as f64)
    }
}

/// Probe `v_b` for fiber invariance under the orientation lattice's own motion.
pub fn witness_rate(base: &OrientationLattice, b: usize, steps: usize) -> WitnessReading {
    let chart = BlockChart::new(b, base.l).expect("b must divide L");
    let snap = base.snapshot();
    let movable = base.model().movable();
    let cells: Vec<usize> =
        (0..base.cells.len()).filter(|&c| movable.contains(&base.cells[c])).collect();
    let mut x = base.clone();
    let mut y = base.clone();
    x.audit_every_step = false;
    y.audit_every_step = false;
    let mut out = WitnessReading {
        b,
        probes: 0,
        witnesses: 0,
        predicted: BlockChart::predicted_witness_rate(b, base.l),
    };
    let model = base.model().clone();
    for &cell in &cells {
        x.restore(&snap);
        y.restore(&snap);
        let moved = y.fiber_perturb(cell);
        debug_assert!(moved, "a cell chosen movable had no fiber successor");
        let _ = moved;
        debug_assert_eq!(
            chart.apply(&model, &x.cells),
            chart.apply(&model, &y.cells),
            "the fiber move left the fiber"
        );
        for _ in 0..steps {
            x.step();
            y.step();
        }
        out.probes += 1;
        if chart.apply(&model, &x.cells) != chart.apply(&model, &y.cells) {
            out.witnesses += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_rules() -> OrientationRules {
        OrientationRules {
            bonds_enabled: true,
            amplitude: [1.0, 1.3, 1.28, 1.42, 1.28, 1.3],
            rent: OrientationRules::rent_from_retention(0.5526),
            cold: false,
            stream: true,
        }
    }

    fn scene(l: usize, seed: u64, rules: OrientationRules) -> OrientationLattice {
        let m = Model::fhp6();
        let law = m.fhp_i(true);
        let lat = Lattice::seeded(m, l, seed, 0.2, law);
        OrientationLattice::from_lattice(lat, seed, rules)
    }

    /// The same scene with FLUID-0's own tracer plane on it, so the red count is a carrier
    /// with work in it rather than a zero that conserves trivially.
    fn coloured_scene(l: usize, seed: u64, rules: OrientationRules) -> OrientationLattice {
        let m = Model::fhp6();
        let law = m.fhp_i(true);
        let lat = Lattice::seeded(m, l, seed, 0.2, law);
        let colour = lat.seed_colour_wave(seed, 1.0, 1);
        OrientationLattice::from_lattice(lat, seed, rules).with_colour(colour)
    }

    /// G2, on this module's own carriers: mass, both momenta and the orientation census over
    /// 500 steps, with the work asserted — a run whose bonds never form conserves trivially
    /// (M-VACUOUS-SUCCESS).
    #[test]
    fn mass_momentum_and_the_orientation_census_are_exactly_invariant() {
        let mut g = coloured_scene(64, 0xF1D1, flat_rules());
        let l0 = g.ledger();
        assert!(l0.census_total > 0, "no particle carries an orientation");
        assert!(l0.red > 0, "no particle carries a colour bit");
        let mut held = 0i64;
        for t in 0..500 {
            g.step();
            held += g.counts.formed as i64 - g.counts.broken_rent as i64 - g.counts.blocked as i64;
            let l = g.ledger();
            assert_eq!(l.mass, l0.mass, "mass moved at step {t}");
            assert_eq!(l.momentum[0], l0.momentum[0], "momentum-x moved at step {t}");
            assert_eq!(l.momentum[1], l0.momentum[1], "momentum-y moved at step {t}");
            assert_eq!(l.red, l0.red, "the red count moved at step {t}");
            assert_eq!(l.census_total, l0.census_total, "the orientation total moved at step {t}");
            assert_eq!(l.census, l0.census, "the per-orientation census moved at step {t}");
            assert_eq!(held, l.bonds, "the bond balance broke at step {t}");
        }
        assert!(g.audit.all_exact(), "the running audit disagrees with the explicit ledger");
        assert!(g.audit.totals.formed > 0, "no bond ever formed: the gates passed on no work");
        assert!(g.audit.totals.broken_rent > 0, "no bond ever paid rent");
        assert!(g.audit.totals.collisions_fired > 0, "no collision fired");
    }

    /// The same statement isolated on the streaming rule that could break it: momentum is exact
    /// while joint moves and anomalous hops are actually happening.
    #[test]
    fn momentum_is_exact_under_bonded_streaming() {
        let mut g = scene(40, 0x51EA, flat_rules());
        let l0 = g.ledger();
        for t in 0..300 {
            g.step();
            let l = g.ledger();
            assert_eq!([l.mass, l.momentum[0], l.momentum[1]], [l0.mass, l0.momentum[0], l0.momentum[1]],
                "the joint move moved a conserved integer at step {t}");
        }
        assert!(g.audit.totals.anomalous_hops > 0, "no pair ever moved off its own shift");
        assert!(g.audit.totals.blocked > 0, "no joint move was ever refused: the branch is untested");
        // Both prongs of the refusal, named apart (a two-prong branch that fires on one prong
        // only is a branch with an untested half), and the geometry refusal at exactly zero.
        let t = g.audit.totals;
        assert!(t.blocked_no_vacancy > 0, "the no-vacancy prong never fired");
        assert!(t.blocked_claimed > 0, "the already-claimed prong never fired");
        assert_eq!(t.blocked_no_geometry, 0, "a held bond's recorded link was not a link");
        assert_eq!(
            t.blocked,
            t.blocked_no_vacancy + t.blocked_claimed + t.blocked_no_geometry,
            "the refusal prongs do not sum to the refusals"
        );
    }

    /// A chain is not a pair, and the pass says so in one direction and the other: three
    /// particles in a line, all pointing along it, bonded `0→1→2`. The middle particle sits in
    /// two bonds at once. With every label equal the whole chain streams as one and no bond is
    /// refused; the count of live bonds and the chain's shape are unchanged after the step.
    #[test]
    fn a_chain_of_three_moves_as_one_when_every_label_agrees() {
        let m = Model::fhp6();
        let law = m.identity_collision();
        let l = 12usize;
        let lat = Lattice::seeded(m, l, 0x0, 0.0, law);
        let mut g = OrientationLattice::from_lattice(
            lat,
            0x0,
            OrientationRules { cold: true, ..flat_rules() },
        );
        let start = 3 * l + 3;
        let mut c = start;
        for _ in 0..3 {
            g.cells[c] = 1 << 0;
            g.orient[c * N_DIRS] = 0;
            c = g.neighbour(c, 0);
        }
        g.initial = g.ledger();
        g.audit = OrientationAudit::fresh(&g.initial);
        for t in 0..25 {
            g.step();
            assert_eq!(g.bonds.len(), 2, "the chain lost a bond at step {t}");
            for b in &g.bonds {
                assert!(g.bond_geometry(b).is_some(), "the chain broke a link at step {t}");
            }
            let bg = g.bond_graph();
            assert_eq!(bg.largest, 3, "the chain fell apart at step {t}");
        }
        let t = g.audit.totals;
        assert_eq!(t.blocked, 0, "a chain with one label refused a joint move");
        assert_eq!(t.anomalous_hops, 0, "a chain with one label needed a hop");
        assert!(t.joint_moves >= 50, "the joint-move pass never ran on the chain");
        assert!(g.audit.all_exact(), "the chain's ledger is not exact");
    }

    /// G2's bond leg: `formed − broken − blocked = held`, integer-identical at every step.
    #[test]
    fn the_bond_balance_is_exact_at_every_step() {
        let mut g = scene(64, 0xB0AD, flat_rules());
        let mut held = 0i64;
        for t in 0..500 {
            g.step();
            held += g.counts.formed as i64 - g.counts.broken_rent as i64 - g.counts.blocked as i64;
            assert_eq!(held, g.bonds.len() as i64, "the bond balance broke at step {t}");
        }
        assert!(g.audit.bond_balance_exact);
        assert!(held > 0, "the balance was checked on an empty bond table");
    }

    /// Every live bond's recorded pair is a real link: the donor's arm points along it and the
    /// acceptor sits at the other end. A bond is a PAIR, and this is the check that it stays one.
    #[test]
    fn every_live_bond_is_a_real_link_with_its_roles_indexed() {
        let mut g = scene(32, 0x11FE, flat_rules());
        for t in 0..200 {
            g.step();
            for (i, b) in g.bonds.iter().enumerate() {
                assert!(g.bond_geometry(b).is_some(), "bond {i} is not a link at step {t}");
                assert_eq!(g.donor_of[b.donor as usize], i as u32, "donor index at step {t}");
                assert_eq!(g.acceptor_of[b.acceptor as usize], i as u32, "acceptor index at step {t}");
            }
            let donors = g.donor_of.iter().filter(|&&x| x != NO_BOND).count();
            let acceptors = g.acceptor_of.iter().filter(|&&x| x != NO_BOND).count();
            assert_eq!(donors, g.bonds.len(), "a stale donor role at step {t}");
            assert_eq!(acceptors, g.bonds.len(), "a stale acceptor role at step {t}");
        }
    }

    /// The no-bond path IS FLUID-0's colour step: with bonds forbidden the occupation bytes
    /// and the colour plane are bit-identical to `Lattice::advance_with_colour` over 500 steps,
    /// with and without the chirality hash. Measured, not asserted.
    #[test]
    fn the_no_bond_path_is_advance_with_colour() {
        let m = Model::fhp6();
        let rules = OrientationRules { bonds_enabled: false, ..flat_rules() };
        for chiral in [false, true] {
            let l = 32usize;
            let mut lat = Lattice::seeded(m.clone(), l, 0xC1A5, 0.3, m.fhp_i(true));
            if chiral {
                lat.randomize_chirality(0x5EED);
            }
            let colour = lat.seed_colour_wave(0xC1A5, 0.4, 1);
            let mut g = OrientationLattice::from_lattice(lat.clone(), 0xC1A5, rules).with_colour(colour.clone());

            let mut cells = lat.cells.clone();
            let mut col = colour;
            let mut out = vec![0u8; l * l];
            let mut out_col = vec![0u8; l * l];
            for t in 0..500u64 {
                lat.advance_with_colour(&mut cells, &mut col, &mut out, &mut out_col, t);
                core::mem::swap(&mut cells, &mut out);
                core::mem::swap(&mut col, &mut out_col);
                g.step();
                assert_eq!(g.cells, cells, "chiral={chiral}: occupation diverged at step {t}");
                assert_eq!(
                    g.colour.as_ref().unwrap(),
                    &col,
                    "chiral={chiral}: colour diverged at step {t}"
                );
                assert!(g.bonds.is_empty(), "a bond formed with bonds forbidden");
            }
        }
    }

    /// G3, in the unit suite: on a held geometry with streaming off, the measured held fraction
    /// is the rent clause's own `1/(1 + p_break)`, at `φ = 0°` and `φ = 120°`.
    #[test]
    fn detailed_balance_of_the_bond_on_a_held_geometry() {
        for phi in [0usize, 2] {
            let (held, expected, tests, links) = held_fraction_scene(phi, 100_000, flat_rules());
            assert!(tests > 1_000, "phi={phi}: the scene never exposed a bond to the rule");
            assert_eq!(links, if phi == 0 { 2 } else { 1 }, "phi={phi}: unexpected link count");
            let rel = (held - expected).abs() / expected;
            assert!(rel <= 0.02, "phi={phi}: held {held} vs 1/(1+p_break) {expected}, rel {rel}");
        }
    }

    /// The chart is set from ONE number and reproduces it: at `E₀/kT_lat = ln(f/(1−f))` the
    /// linear bond's held fraction is `f`.
    #[test]
    fn the_chart_reproduces_the_retention_it_was_set_from() {
        let f = 0.5526;
        let rules = OrientationRules { rent: OrientationRules::rent_from_retention(f), ..flat_rules() };
        let (held, _, _, _) = held_fraction_scene(0, 200_000, rules);
        assert!((held - f).abs() / f <= 0.02, "held {held} against the read retention {f}");
    }

    /// A two-particle scene with streaming off: a donor pointing along the link and an acceptor
    /// at angle `φ`. Returns (measured held fraction of THE link, `1/(1+p_break)`, break tests,
    /// links ever seen in the scene).
    ///
    /// **The held fraction is the fraction of steps THE tracked link is held, not the bond
    /// count.** At `φ = 0°` the acceptor's arm points back along the link by the definition of
    /// the angle, so the geometry also satisfies the formation rule in the reverse direction
    /// and the scene carries TWO links — `i → j` and `j → i` — on the one pair. They are two
    /// bonds and each pays its own rent, so `bonds.len()` averages `2/(1 + p_break)` and reads
    /// exactly double the freeze's `1/(1 + p_break)`. The freeze's G3 is a statement about ONE
    /// link at a held geometry, so the reading is the donor role of the tracked donor slot,
    /// counted once per step. The second link is reported, never subtracted.
    fn held_fraction_scene(
        phi: usize,
        steps: usize,
        rules: OrientationRules,
    ) -> (f64, f64, u64, usize) {
        let m = Model::fhp6();
        let law = m.fhp_i(true);
        let l = 8usize;
        let lat = Lattice::seeded(m.clone(), l, 0x0, 0.0, law);
        assert!(lat.cells.iter().all(|&s| s == 0), "the scene is not empty");
        let mut g = OrientationLattice::from_lattice(lat, 0x0, OrientationRules { stream: false, ..rules });
        let delta = 0usize; // the link direction
        let i = 2 * l + 2;
        let j = g.neighbour(i, delta);
        // one particle per cell, so the collision is the identity on both (dimension-1 fibers)
        g.cells[i] = 1 << 1;
        g.orient[i * N_DIRS + 1] = delta as u8;
        g.cells[j] = 1 << 4;
        let opp = (delta + 3) % N_DIRS;
        g.orient[j * N_DIRS + 4] = ((opp + phi) % N_DIRS) as u8;
        g.initial = g.ledger();
        g.audit = OrientationAudit::fresh(&g.initial);

        let tracked = i * N_DIRS + 1; // the donor slot of the link i -> j
        let mut held = 0u64;
        let mut links_seen = 0usize;
        for _ in 0..steps {
            g.step();
            if g.donor_of[tracked] != NO_BOND {
                held += 1;
            }
            links_seen = links_seen.max(g.bonds.len());
        }
        let p = g.rules.p_break(phi);
        (
            held as f64 / steps as f64,
            OrientationRules::retention(p),
            g.audit.totals.break_tests[phi],
            links_seen,
        )
    }

    /// The bond graph's spanning reading is invariant under translating the configuration,
    /// which the "touches both edges" reading is not — the reason the winding is the gate.
    #[test]
    fn spanning_is_a_winding_and_a_ring_exhibits_one() {
        let m = Model::fhp6();
        let law = m.identity_collision();
        let l = 6usize;
        let lat = Lattice::seeded(m, l, 0x0, 0.0, law);
        let mut g = OrientationLattice::from_lattice(lat, 0x0, OrientationRules { stream: false, cold: true, ..flat_rules() });
        // A ring of particles all pointing along direction 0, one per cell of one column: the
        // chain closes on itself through the wrap, so it winds.
        for k in 0..l {
            let c = k * l;
            g.cells[c] = 1 << 0;
            g.orient[c * N_DIRS] = 0;
        }
        g.initial = g.ledger();
        g.step();
        let bg = g.bond_graph();
        assert_eq!(bg.bonds, l, "the ring did not close: {} bonds", bg.bonds);
        assert_eq!(bg.largest, l);
        assert!(bg.largest_spans, "a closed ring around the torus does not read as spanning");
        assert_eq!(bg.largest_winds, [true, false]);
        assert!((bg.bonds_per_particle_donor - 1.0).abs() < 1e-12);
        assert!((bg.bonds_per_particle_degree - 2.0).abs() < 1e-12);
    }

    /// And an open chain does NOT span, so the reading has both answers on one instrument.
    #[test]
    fn an_open_chain_does_not_span() {
        let m = Model::fhp6();
        let law = m.identity_collision();
        let l = 8usize;
        let lat = Lattice::seeded(m, l, 0x0, 0.0, law);
        let mut g = OrientationLattice::from_lattice(lat, 0x0, OrientationRules { stream: false, cold: true, ..flat_rules() });
        for k in 0..4 {
            let c = k * l;
            g.cells[c] = 1 << 0;
            g.orient[c * N_DIRS] = 0;
        }
        g.initial = g.ledger();
        g.step();
        let bg = g.bond_graph();
        assert_eq!(bg.bonds, 3, "an open chain of four has three bonds, not {}", bg.bonds);
        assert!(!bg.largest_spans, "an open chain read as spanning");
        assert_eq!(bg.largest, 4);
    }

    /// The cold control holds every bond it makes, and the no-bond control makes none: the two
    /// ends of the rent clause, on one instrument.
    #[test]
    fn the_cold_control_holds_and_the_no_bond_control_makes_none() {
        let mut cold = scene(24, 0xC01D, OrientationRules { cold: true, ..flat_rules() });
        for _ in 0..50 {
            cold.step();
        }
        assert_eq!(cold.audit.totals.broken_rent, 0, "the cold control paid rent");
        assert!(!cold.bonds.is_empty(), "the cold control formed nothing");
        let mut off = scene(24, 0xC01D, OrientationRules { bonds_enabled: false, ..flat_rules() });
        for _ in 0..50 {
            off.step();
        }
        assert_eq!(off.audit.totals.formed, 0, "a bond formed with bonds forbidden");
        assert!(off.bonds.is_empty());
    }

    /// The amplitude table is a lever: a bigger `A(φ)` is a smaller `p_break` is a longer bond.
    /// The plant's carrier, checked on the arithmetic before any run reads it.
    #[test]
    fn the_amplitude_table_moves_the_break_probability_the_way_the_map_says() {
        let r = flat_rules();
        for phi in 1..N_DIRS {
            assert!(r.amplitude[phi] > r.amplitude[0], "the test table is not the map's shape");
            assert!(r.p_break(phi) < r.p_break(0), "a higher amplitude did not hold the bond longer");
        }
        // and the chart's own arithmetic, in both directions
        let f = 0.5526;
        let rent = OrientationRules::rent_from_retention(f);
        let p = (-rent).exp();
        assert!((OrientationRules::retention(p) - f).abs() < 1e-12);
    }

    /// The fiber perturbation moves the occupation byte and NOTHING else: same label, same
    /// orientation multiset, same bond table.
    #[test]
    fn the_fiber_perturbation_moves_only_the_occupation_byte() {
        let mut g = scene(24, 0x1BEE, flat_rules());
        for _ in 0..20 {
            g.step();
        }
        let before = g.ledger();
        let movable = g.model().movable();
        let cell = (0..g.cells.len())
            .find(|&c| movable.contains(&g.cells[c]))
            .expect("no movable cell in the configuration");
        let s = g.cells[cell];
        let bonds_before = g.bonds.clone();
        assert!(g.fiber_perturb(cell));
        assert_ne!(g.cells[cell], s);
        let after = g.ledger();
        assert_eq!(after.mass, before.mass);
        assert_eq!(after.momentum, before.momentum);
        assert_eq!(after.census, before.census);
        assert_eq!(after.bonds, before.bonds);
        assert_eq!(g.bonds.len(), bonds_before.len());
        for b in &g.bonds {
            assert!(g.bond_geometry(b).is_some(), "the perturbation broke a bond's link");
        }
    }
}
