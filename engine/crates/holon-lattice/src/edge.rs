//! EDGE-0's carrier: the closure that can WAIT — a lattice gas with a rest particle, two
//! donor arms, a bonded pair that stays put when it cannot move as one, and gravity as a
//! posted momentum injection.
//!
//! `conformance/mesh/EDGE0_PREREG.md` is the freeze. FLUID-1 read branch (c) and left two
//! INSTRUMENT findings for the next carrier, in its own words: "the closure must be able to
//! wait" (on FHP-6 there is no rest state, so a bonded pair refused its joint move has to
//! break — 36 % of the chart's releases and 100 % of the cold control's), and "the count is
//! capped by the arm, not the rent" (one donor arm puts the each-bond-once ceiling at 1
//! where the liquid's is 2). This module is those two findings built, and nothing else is
//! changed: `crate::orientation` is untouched, and where the two rules coincide this carrier
//! reproduces it BIT FOR BIT ([`tests::the_one_arm_no_rest_carrier_is_fluid_ones`]).
//!
//! # The three rules this module declares, and why each conserves
//!
//! 1. **The rest particle, and the collisions that make and unmake it (FHP-II/III).** The
//!    chart is [`Model::fhp7`]: FHP-6's six unit directions and a SEVENTH slot of zero
//!    velocity. A rest particle adds `(1, 0, 0)` to the conserved label, so on the hexagon —
//!    where `c_{d−1} + c_{d+1} = c_d` in the axial integers — the configurations `{rest, d}`
//!    and `{d−1, d+1}` carry the SAME `(N, P)` and a collision may exchange them. That is
//!    FHP-II's rest collision, and here it is not a typed table: it is one of the twelve
//!    dimension-2 fibers of the seven-slot census, which `state.rs` computes.
//!
//!    The law is [`fiber_cycle_laws`] — call it **E-I**: `{identity, fiber successor, fiber
//!    predecessor}`, one drawn per cell per step by the counter hash. Each table permutes
//!    within fibers, so each is a sector-preserving bijection and the draw is doubly
//!    stochastic on every fiber (semi-detailed balance holds by construction, which FHP-I's
//!    deterministic chirality does not give).
//!
//!    **On the two fibers FHP-I acts on, E-I's successor and predecessor ARE FHP-I's two
//!    chiralities** — the 3-cycle on the head-on fiber `{9,18,36}` and the swap on `{21,42}`,
//!    measured in [`tests::the_fiber_cycle_law_contains_fhp_i`] and not asserted. E-I is
//!    STRICTLY MORE COLLISIONAL than FHP-I and the freeze must say so: it also turns the
//!    seven other dimension-2 fibers of FHP-6, which FHP-I leaves fixed, so it acts on all
//!    twenty movable six-direction states where FHP-I acts on five. It is a named element
//!    set of the same 4,608-element group `state.rs` enumerates, not a new kind of law, and
//!    NO transport coefficient of it has been measured — FLUID-0's census is about FHP-I's
//!    chart and none of its numbers transfer. Credit: Frisch, Hasslacher & Pomeau 1986
//!    (FHP-I); Frisch, d'Humières, Hasslacher, Lallemand, Pomeau & Rivet 1987 (FHP-II/III,
//!    the rest particle).
//!
//! 2. **Two donor arms, and the WAIT.** A particle carries ONE orientation `o`, and its arms
//!    point along `o` and `o + 2` (120° apart on the hexagon — the lattice's nearest
//!    approach to the water's 104.5°, and the Mercedes-Benz model's angle: Ben-Naim 1971,
//!    Silverstein, Haymet & Dill 1998). Two arms and two acceptor roles put the
//!    each-bond-once ceiling at 2 (the liquid's) and the degree ceiling at 4.
//!
//!    **A bonded pair's mover set is `{a, b, REST}`**, where `a` and `b` are the two
//!    particles' own direction labels. FLUID-1's rule stands unchanged as the first
//!    candidate: `m` is drawn from `{a, b}` by the counter hash and BOTH particles are
//!    displaced by `dirs[m]`. What is new is the second candidate: when that move is refused
//!    — for want of a vacancy, or because a particle is already claimed at another `m` — the
//!    pair tries `m = REST`, whose displacement is `dirs[rest] = [0,0]`, so **both particles
//!    stay in their own cells keeping their own labels**. That is the wait, and it is the
//!    SAME rule with the rest direction admitted as a mover, not a second rule beside it.
//!
//!    **Why it conserves.** Momentum is `Σ dirs[label]` over occupied slots and a
//!    displacement never touches a label, so any displacement pattern whatever leaves both
//!    components integer-exact; mass needs only that the pattern be injective, and every
//!    displacement here is a move within one plane into a slot CHECKED vacant, with each
//!    particle displaced at most once (the claim). A rest state is how a particle can wait
//!    *without carrying momentum* — but the wait itself does not put anything at rest, and
//!    that is the point: the collision is the only thing that changes a label, it is
//!    sector-preserving, and so the ledger never needs the wait's permission.
//!    [`tests::the_wait_holds_a_pair_the_old_rule_would_have_released`] exhibits a scene
//!    where FLUID-1's rule releases the bond and this one holds it.
//!
//!    If the wait is refused too, the bond is released as `blocked` — FLUID-1's branch,
//!    kept, counted, with its two prongs still apart.
//!
//! 2b. **THE CLUSTER MOVES AS ONE** ([`MoverRule::Cluster`], selectable beside rule 2 and
//!    never instead of it). Rule 2 draws a mover per BOND, and EDGE-0's first screen measured
//!    what that costs: of 6,281,540 bonds formed, 5,936,201 were released because a particle
//!    was already claimed by another bond at a DIFFERENT mover. With two arms the bond graph
//!    is dense, so the bonds of one cluster fight each other, and the rule is anti-cohesive
//!    exactly where a dense phase would form. That is a finding about the pair rule and not
//!    about closure, so the closure grammar's own statement is offered as a second rule:
//!
//!    * **The mover is drawn per CONNECTED COMPONENT of the bond graph**, not per bond. The
//!      components are found each step with `holon_closure::ComponentFinder` — the same
//!      union–find `holon_closure::phase` reports the largest component with, kept in one
//!      place and fenced by that crate's own agreement test — over the PRE-shift slots, and
//!      the draw picks one member uniformly by the counter hash and takes ITS label. The draw
//!      is keyed on the component's lowest pre-shift slot, so it depends on no traversal
//!      order.
//!    * **Every member is displaced by the one drawn mover, keeping its own label.** Momentum
//!      is exact for the reason rule 2 gives: a displacement never touches a label.
//!    * **The joint move of the whole component is admitted only if every target slot is
//!      vacant OR vacated by the same component.** A shift of a set onto itself is injective,
//!      and the targets are pairwise distinct by construction (two members sharing a target
//!      would have to share a cell and a label, i.e. be one particle), so the whole map stays
//!      injective and mass is exact. The members are read out and cleared BEFORE any is
//!      written, because the source and target sets overlap.
//!    * **Otherwise the whole component WAITS at rest** — the same second candidate as rule
//!      2, `m = rest`, every member back in its own cell keeping its own label — **and is
//!      released only by the rent clause, per bond.**
//!    * **`blocked` counts the components refused BOTH**, with the prongs apart
//!      (`component_move_refused`, `component_wait_refused`), and releases that component's
//!      bonds so the ledger's balance still closes in bonds.
//!
//!    Each particle belongs to exactly one component, so no particle can be displaced twice
//!    and rule 2's claim table is not needed at all under this rule.
//!
//! 3. **Gravity as a momentum bias at collisions, POSTED.** After the collision and before
//!    formation, each cell is drawn against `rate`; where the draw fires and the slot along
//!    `g` is empty, a particle moving against `g` is turned to move along it (posting
//!    `dirs[g] − dirs[−g] = 2·dirs[g]`), or failing that a REST particle is set moving along
//!    `g` (posting `dirs[g]`). The two prongs are counted apart, so a branch that never
//!    fires is visible rather than hidden in their sum. The particle carries its
//!    orientation, its colour and every one of its roles, and the bonds' recorded slots are
//!    rewritten, so a gravity kick never leaves a stale slot behind.
//!
//!    **The ledger is exact UP TO THE POSTED INJECTION and the gate says so**:
//!    `momentum(t) = momentum(0) + Σ posted(t)`, an integer identity checked at every step
//!    ([`EdgeAudit::px_exact`]). Mass is untouched by gravity and is gated separately — one
//!    gate per conservation law, `lattice.rs`'s own rule for the wall. Credit:
//!    Rothman & Zaleski, *Lattice-Gas Cellular Automata* (1997), §5 — a body force on a
//!    lattice gas is a momentum bias applied at a fraction of the cells.
//!
//! # Two temperatures, never conflated
//!
//! `rules.rent` is `E₀/kT_rent`, FLUID-1's DIMENSIONLESS chart parameter for the bond's
//! break probability. It is not a mechanical temperature and nothing here treats it as one.
//! The capillary spectrum needs a mechanical `kT` and takes the carrier's OWN measured
//! ideal-gas coefficient (`crate::surface::ideal_gas_kt`), which is a different number with
//! a different meaning. Both names are spelled out wherever either appears.
//!
//! # What is inherited unchanged
//!
//! The occupation seeding, the neighbour table, the ledger sum and the colour rule are
//! `lattice.rs`'s; the orientation seeding, the collision's slot remap, the "a bond pays no
//! rent on the step it forms" clause, the fixed-order greedy formation pass, the break rule
//! and the claim-based joint-move pass are `orientation.rs`'s, down to the counter-hash keys
//! (imported from that module, never respelled).

// The index IS the meaning here: a loop over `0..n` walks the cell's direction slots and the
// parallel role arrays are indexed by that same slot, so `enumerate` on one of them would name
// one array as the loop's subject and leave the others reading as incidental.
// `transport.rs` waives the same lint for the same reason.
// `!(x > bar)` is DELIBERATE where `x` may be `NaN`: a refused fit must fall on the failing
// side, and `NaN <= bar` is false while `!(NaN > bar)` is true. `holon-campaign` waives the
// same lint at the same two places for the same reason.
#![allow(
    clippy::needless_range_loop,
    clippy::manual_is_multiple_of,
    clippy::neg_cmp_op_on_partial_ord
)]

use crate::chart::BlockChart;
use crate::lattice::{mix64, recolour, ColourRule, Lattice};
use crate::orientation::{
    unit, BREAK_KEY, GOLDEN, MOVER_KEY, NO_BOND, NO_ORIENT, ORIENT_MIX_KEY, ORIENT_SEED_KEY,
};
use crate::state::Model;

/// Orientations are always one of the SIX lattice arms, whether or not the particle that
/// carries them is moving. A rest particle points somewhere; it just does not go there.
pub const N_ORIENT: usize = 6;
/// Donor arms per particle. Two is the water's two hydrogens and the each-bond-once ceiling.
pub const MAX_ARMS: usize = 2;
/// Acceptor roles per particle. Two is the water's two lone pairs; with two arms the degree
/// ceiling is four, which is the liquid's coordination.
pub const MAX_ACCEPTORS: usize = 2;

/// This module's own counter-hash stream, held apart from `orientation.rs`'s four so that a
/// cell's gravity draw is independent of its collision, its orientations and its bonds.
pub const GRAVITY_KEY: u64 = 0x4772_6176_6974_7921;
/// The collision law's table draw, when the law has more than one table.
pub const TABLE_KEY: u64 = 0x5461_626C_6553_656C;

/// One bond: the ordered pair of slots and WHICH ARM donates it. The link direction and the
/// acceptor angle are derived from the state ([`EdgeLattice::bond_geometry`]), never stored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bond {
    pub donor: u32,
    pub acceptor: u32,
    /// `0` or `1`: the arm at `o` or the arm at `o + 2`.
    pub arm: u8,
    /// Which of the acceptor's roles holds this bond, `0` or `1`.
    ///
    /// Recorded rather than searched for, and the reason is a measured one: without it every
    /// pass that changes the list has to CLEAR both whole role tables and rebuild them, which
    /// is `2 · slots · roles` writes per step and was most of a screen's cost. With it, a
    /// broken or released bond clears exactly the two entries it owns and the rebuild is
    /// write-only, which is `orientation.rs`'s own arrangement carried to two roles. It is
    /// still not a flag on a particle: the pair list is the object and this is the pair's own
    /// record of where its index lives.
    pub acc: u8,
}

/// Which rule moves a bonded closure: FLUID-1's pair, or EDGE-0's cluster.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoverRule {
    /// Rule 2: the mover is drawn per BOND and a chain moves as one only where its bonds
    /// happen to draw the same mover. FLUID-1's rule, unchanged, and the control arm.
    Pair,
    /// Rule 2b: the mover is drawn per CONNECTED COMPONENT and the whole component moves or
    /// the whole component waits.
    Cluster,
}

/// Gravity: a direction and the per-cell rate at which the bias is applied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gravity {
    /// A MOVING direction, `0..6`. The rest slot is not a direction gravity can point in.
    pub dir: usize,
    /// The per-cell, per-step probability that the bias is attempted.
    pub rate: f64,
}

/// The carrier's rules — every knob, in one place.
#[derive(Clone, Copy, Debug)]
pub struct EdgeRules {
    /// Bonds may form. `false` is the no-bond control.
    pub bonds_enabled: bool,
    /// Donor arms per particle, `1` or `2`. `1` is FLUID-1's.
    pub arms: usize,
    /// Acceptor roles per particle, `1` or `2`. `1` is FLUID-1's.
    pub acceptors: usize,
    /// `A(φ)` at the six lattice angles, READ from CT-2's tilt family; never typed here.
    pub amplitude: [f64; N_ORIENT],
    /// `E₀ / kT_rent` — FLUID-1's DIMENSIONLESS chart parameter. Not a mechanical kT.
    pub rent: f64,
    /// `p_break ≡ 0`: the `T → 0` limit.
    pub cold: bool,
    /// Streaming on. `false` is a held-geometry scene.
    pub stream: bool,
    /// The waiting rule. Needs a rest slot in the model; without one it can never fire and
    /// the carrier falls back to FLUID-1's `blocked` release exactly.
    pub wait_enabled: bool,
    /// Which rule moves a bonded closure — the pair (rule 2) or the cluster (rule 2b).
    pub mover: MoverRule,
    /// Gravity, or none.
    pub gravity: Option<Gravity>,
}

impl EdgeRules {
    /// FLUID-1's rules, so the identity control is a configuration and not a second carrier.
    pub fn fluid1(amplitude: [f64; N_ORIENT], rent: f64) -> Self {
        Self {
            bonds_enabled: true,
            arms: 1,
            acceptors: 1,
            amplitude,
            rent,
            cold: false,
            stream: true,
            wait_enabled: false,
            mover: MoverRule::Pair,
            gravity: None,
        }
    }

    /// EDGE-0's PAIR arm: two arms, two acceptor roles, the wait on, rule 2. The freeze's
    /// pre-committed control.
    pub fn edge0(amplitude: [f64; N_ORIENT], rent: f64) -> Self {
        Self {
            arms: MAX_ARMS,
            acceptors: MAX_ACCEPTORS,
            wait_enabled: true,
            ..Self::fluid1(amplitude, rent)
        }
    }

    /// EDGE-0's CLUSTER arm: [`EdgeRules::edge0`] under rule 2b.
    pub fn edge0_cluster(amplitude: [f64; N_ORIENT], rent: f64) -> Self {
        Self { mover: MoverRule::Cluster, ..Self::edge0(amplitude, rent) }
    }

    /// `p_break(φ) = exp(−A(φ)·E₀/kT_rent)`, and exactly zero in the cold limit.
    #[inline]
    pub fn p_break(&self, phi_index: usize) -> f64 {
        if self.cold {
            0.0
        } else {
            (-self.amplitude[phi_index] * self.rent).exp()
        }
    }

    /// `E₀/kT_rent` from ONE measured retention: `1/(1 + e^{−E₀/kT}) = f`.
    pub fn rent_from_retention(f: f64) -> f64 {
        assert!(f > 0.0 && f < 1.0, "a retention outside (0,1) sets no chart");
        (f / (1.0 - f)).ln()
    }

    /// The stationary held fraction of the rent clause's two-state chain.
    #[inline]
    pub fn retention(p_break: f64) -> f64 {
        1.0 / (1.0 + p_break)
    }
}

/// What one step did. Every field is a COUNT: a gate reporting PASS on zero work has not
/// passed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EdgeCounts {
    pub collisions_fired: u64,
    pub formed: u64,
    pub broken_rent: u64,
    /// Bonds released because BOTH the drawn joint move and the wait were refused.
    pub blocked: u64,
    pub blocked_no_vacancy: u64,
    pub blocked_claimed: u64,
    /// A held bond whose recorded link was not a link when the rent pass looked at it. Zero
    /// by construction; carried so a defect shows in the record instead of silently.
    pub blocked_no_geometry: u64,
    pub break_tests: [u64; N_ORIENT],
    pub broken_by_phi: [u64; N_ORIENT],
    pub joint_moves: u64,
    pub anomalous_hops: u64,
    /// The wait was TRIED (the drawn move was refused) and the wait was TAKEN. Under rule 2
    /// these count PAIRS; under rule 2b they count COMPONENTS, and the rule is in the record.
    pub wait_attempts: u64,
    pub waited: u64,
    /// Rule 2b only: components with at least one bond, walked this step; those whose drawn
    /// move was admitted; those refused the drawn move; those refused the wait as well (and
    /// therefore released). The last two are the two prongs of a cluster release, apart.
    pub components: u64,
    pub component_moves: u64,
    pub component_move_refused: u64,
    pub component_wait_refused: u64,
    /// The size of the largest component the mover pass saw this step — a cluster rule that
    /// never built a cluster has not been shown able to.
    pub largest_component: u64,
    /// Rest particles at the end of the step, and the collision's own net creation.
    pub rest_created: u64,
    pub rest_destroyed: u64,
    /// Gravity's two prongs and its refusals, apart.
    pub gravity_flip: u64,
    pub gravity_rest: u64,
    pub gravity_refused: u64,
}

impl EdgeCounts {
    /// Fold another step's (or another member's) counts into these. `pub` because a runner
    /// that advances several configurations reports ONE set of totals.
    pub fn add(&mut self, o: &EdgeCounts) {
        self.collisions_fired += o.collisions_fired;
        self.formed += o.formed;
        self.broken_rent += o.broken_rent;
        self.blocked += o.blocked;
        self.blocked_no_vacancy += o.blocked_no_vacancy;
        self.blocked_claimed += o.blocked_claimed;
        self.blocked_no_geometry += o.blocked_no_geometry;
        self.joint_moves += o.joint_moves;
        self.anomalous_hops += o.anomalous_hops;
        self.wait_attempts += o.wait_attempts;
        self.waited += o.waited;
        self.components += o.components;
        self.component_moves += o.component_moves;
        self.component_move_refused += o.component_move_refused;
        self.component_wait_refused += o.component_wait_refused;
        self.largest_component = self.largest_component.max(o.largest_component);
        self.rest_created += o.rest_created;
        self.rest_destroyed += o.rest_destroyed;
        self.gravity_flip += o.gravity_flip;
        self.gravity_rest += o.gravity_rest;
        self.gravity_refused += o.gravity_refused;
        for k in 0..N_ORIENT {
            self.break_tests[k] += o.break_tests[k];
            self.broken_by_phi[k] += o.broken_by_phi[k];
        }
    }
}

/// The conserved integers at one instant, and the cumulative posted injection beside them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeLedger {
    pub mass: i64,
    pub momentum: [i64; 2],
    /// Cumulative momentum PUT IN by gravity, in the same integer units, so that
    /// `momentum(t) − injected(t) = momentum(0)` is an identity and not a tolerance.
    pub injected: [i64; 2],
    pub census: [i64; N_ORIENT],
    pub census_total: i64,
    pub red: i64,
    pub bonds: i64,
    pub rest: i64,
}

/// The ledger gate's carrier: every integer, checked at EVERY step.
#[derive(Clone, Copy, Debug)]
pub struct EdgeAudit {
    pub mass_exact: bool,
    /// `momentum(t) − Σ posted(t) == momentum(0)`, integer-identical.
    pub px_exact: bool,
    pub py_exact: bool,
    pub red_exact: bool,
    pub orientation_total_exact: bool,
    pub orientation_census_exact: bool,
    /// `held(t) = held(t−1) + formed − broken_rent − blocked`, integer-identical.
    pub bond_balance_exact: bool,
    pub steps_checked: u64,
    pub mass: i64,
    pub px: i64,
    pub py: i64,
    pub injected: [i64; 2],
    pub red: i64,
    pub orientation_total: i64,
    pub bonds_final: i64,
    pub rest_final: i64,
    pub totals: EdgeCounts,
}

impl EdgeAudit {
    pub fn all_exact(&self) -> bool {
        self.mass_exact
            && self.px_exact
            && self.py_exact
            && self.red_exact
            && self.orientation_total_exact
            && self.orientation_census_exact
            && self.bond_balance_exact
    }

    /// Every failing leg, named — never the first one only.
    pub fn failing_legs(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if !self.mass_exact {
            out.push("mass");
        }
        if !self.px_exact {
            out.push("Px - posted injection");
        }
        if !self.py_exact {
            out.push("Py - posted injection");
        }
        if !self.red_exact {
            out.push("red");
        }
        if !self.orientation_total_exact {
            out.push("orientation total");
        }
        if !self.orientation_census_exact {
            out.push("orientation census");
        }
        if !self.bond_balance_exact {
            out.push("bond balance");
        }
        out
    }

    fn fresh(l0: &EdgeLedger) -> Self {
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
            injected: l0.injected,
            red: l0.red,
            orientation_total: l0.census_total,
            bonds_final: l0.bonds,
            rest_final: l0.rest,
            totals: EdgeCounts::default(),
        }
    }

    /// Fold another member's audit into this one: flags ANDed, counts summed.
    pub fn merge(&mut self, o: &EdgeAudit) {
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

/// The whole mutable state, for a probe that restores a base configuration.
#[derive(Clone, Debug)]
pub struct EdgeState {
    pub cells: Vec<u8>,
    pub orient: Vec<u8>,
    pub donor_of: Vec<u32>,
    pub acceptor_of: Vec<u32>,
    pub bonds: Vec<Bond>,
    pub colour: Option<Vec<u8>>,
    pub step_index: u64,
    pub injected: [i64; 2],
}

/// The Euclidean embedding of a hexagonal direction set — `M = [[1, 1/2], [0, √3/2]]`,
/// applied to every direction including a zero one.
///
/// Written here rather than taken from [`crate::isotropy::embed`] because that routine keys
/// the hexagonal branch on `n_dirs() == 6` and a seven-slot chart would silently take the
/// SQUARE branch. [`tests::the_embedding_agrees_with_isotropys_on_six_directions`] measures
/// the agreement on FHP-6 rather than asserting it.
pub fn hex_embed(model: &Model) -> Vec<[f64; 2]> {
    model
        .dirs
        .iter()
        .map(|d| {
            let (a, b) = (d[0] as f64, d[1] as f64);
            [a + 0.5 * b, (3.0_f64).sqrt() / 2.0 * b]
        })
        .collect()
}

/// The index of the zero-velocity slot, if the model has one.
pub fn rest_index(model: &Model) -> Option<usize> {
    model.dirs.iter().position(|&d| d == [0, 0])
}

/// The law E-I: `{identity, fiber successor, fiber predecessor}`, one drawn per cell per step.
///
/// Every table permutes within `(N, P)` fibers, so each is a sector-preserving bijection by
/// construction — and [`EdgeLattice::from_lattice`] verifies that with `Model::is_bijection`
/// and `Model::is_sector_preserving` rather than trusting how it was built. On the two
/// fibers FHP-I acts on it agrees with FHP-I's two chiralities; on the seven FHP-I leaves
/// fixed it does not, and the module header says so.
pub fn fiber_cycle_laws(model: &Model) -> Vec<Vec<u8>> {
    let id: Vec<u8> = (0..model.n_states() as u8).collect();
    let mut succ = id.clone();
    let mut pred = id.clone();
    for f in model.fibers() {
        if f.len() < 2 {
            continue;
        }
        for (i, &s) in f.iter().enumerate() {
            let t = f[(i + 1) % f.len()];
            succ[s as usize] = t;
            pred[t as usize] = s;
        }
    }
    vec![id, succ, pred]
}

/// The carrier.
#[derive(Clone, Debug)]
pub struct EdgeLattice {
    /// Tables only — the model, the torus wrap and the ledger sum. Its own `cells` are the
    /// SEEDING and are not advanced.
    tables: Lattice,
    /// The collision law's tables and the key its per-cell draw is made on. One table is a
    /// deterministic law and the draw is not made at all.
    laws: Vec<Vec<u8>>,
    law_key: u64,
    pub l: usize,
    pub n: usize,
    pub rest: Option<usize>,
    pub cells: Vec<u8>,
    /// `orient[cell · n + dir]`, [`NO_ORIENT`] where the slot is empty. Always in `0..6`.
    pub orient: Vec<u8>,
    /// `donor_of[slot · MAX_ARMS + arm]` — the bond that arm donates, or [`NO_BOND`].
    pub donor_of: Vec<u32>,
    /// `acceptor_of[slot · MAX_ACCEPTORS + k]` — the bond that role accepts, or [`NO_BOND`].
    pub acceptor_of: Vec<u32>,
    /// The pair list is the object; the two tables above are its index.
    pub bonds: Vec<Bond>,
    pub colour: Option<Vec<u8>>,
    pub rules: EdgeRules,
    pub step_index: u64,
    pub counts: EdgeCounts,
    pub audit: EdgeAudit,
    pub audit_every_step: bool,
    /// Cumulative momentum posted into the fluid by gravity.
    pub injected: [i64; 2],
    /// Per-cell momentum-flux accumulator `[Πxx, Πxy, Πyx, Πyy]`, when one is switched on.
    pub flux: Option<Vec<[f64; 4]>>,
    pub flux_steps: u64,
    embed: Vec<[f64; 2]>,
    initial: EdgeLedger,
    // scratch, allocated once
    out_cells: Vec<u8>,
    out_orient: Vec<u8>,
    out_donor: Vec<u32>,
    out_acceptor: Vec<u32>,
    out_colour: Vec<u8>,
    pre_slots: Vec<(u32, u32)>,
    keep: Vec<Bond>,
    claim: Vec<u8>,
    claimed: Vec<u32>,
    order: Vec<u32>,
    blocked_idx: Vec<u32>,
    dead: Vec<bool>,
    // rule 2b's scratch, allocated once: the shared union-find, the members and the bonds
    // grouped by component, and a per-slot mark for "this slot is vacated by this component".
    finder: holon_closure::ComponentFinder,
    comp_slots: Vec<(u32, u32)>,
    comp_bonds: Vec<(u32, u32)>,
    mine: Vec<bool>,
    from_slots: Vec<u32>,
    to_slots: Vec<u32>,
    payload: Vec<(u8, [u32; MAX_ARMS], [u32; MAX_ACCEPTORS], u8)>,
}

impl EdgeLattice {
    /// Wrap an already-seeded [`Lattice`] with a named law.
    ///
    /// `laws` is the collision law's table set; one table is deterministic, several are drawn
    /// per cell per step. Orientations are drawn uniformly over the six arms from a counter
    /// hash of `(seed, cell, dir)` on `orientation.rs`'s OWN key, so a one-arm, no-rest
    /// carrier is bit-identical to FLUID-1's from the first step.
    pub fn from_lattice(
        tables: Lattice,
        laws: Vec<Vec<u8>>,
        law_key: u64,
        orient_seed: u64,
        rules: EdgeRules,
    ) -> Self {
        assert!(!laws.is_empty(), "a carrier needs at least one collision table");
        for law in &laws {
            assert!(tables.model.is_bijection(law), "a collision table that is not a bijection");
            assert!(
                tables.model.is_sector_preserving(law),
                "a collision table that does not fix the conserved label"
            );
        }
        assert!(
            tables.solid.iter().all(|&s| !s),
            "the edge carrier carries no wall: the bond rule, the wait and the bounce-back \
             have not been defined together and a silent wall would be a channel outside the \
             ledger"
        );
        assert!((1..=MAX_ARMS).contains(&rules.arms), "arms must be 1 or 2");
        assert!((1..=MAX_ACCEPTORS).contains(&rules.acceptors), "acceptor roles must be 1 or 2");
        let n = tables.model.n_dirs();
        let rest = rest_index(&tables.model);
        assert!(n >= N_ORIENT, "the edge carrier is defined on the six lattice arms and up");
        if let Some(g) = rules.gravity {
            assert!(g.dir < N_ORIENT, "gravity points along a MOVING direction");
            assert!((0.0..=1.0).contains(&g.rate), "a gravity rate outside [0,1]");
        }
        let l = tables.l;
        let n_cells = l * l;
        let n_slots = n_cells * n;
        let cells = tables.cells.clone();
        let mut orient = vec![NO_ORIENT; n_slots];
        for (c, &s) in cells.iter().enumerate() {
            for d in 0..n {
                if s >> d & 1 == 1 {
                    let h = mix64(
                        ORIENT_SEED_KEY
                            ^ orient_seed
                            ^ (c as u64).wrapping_mul(GOLDEN)
                            ^ ((d as u64) << 56),
                    );
                    orient[c * n + d] = (h % N_ORIENT as u64) as u8;
                }
            }
        }
        let embed = hex_embed(&tables.model);
        let mut me = Self {
            tables,
            laws,
            law_key,
            l,
            n,
            rest,
            cells,
            orient,
            donor_of: vec![NO_BOND; n_slots * MAX_ARMS],
            acceptor_of: vec![NO_BOND; n_slots * MAX_ACCEPTORS],
            bonds: Vec::new(),
            colour: None,
            rules,
            step_index: 0,
            counts: EdgeCounts::default(),
            audit: EdgeAudit::fresh(&EdgeLedger {
                mass: 0,
                momentum: [0, 0],
                injected: [0, 0],
                census: [0; N_ORIENT],
                census_total: 0,
                red: 0,
                bonds: 0,
                rest: 0,
            }),
            audit_every_step: true,
            injected: [0, 0],
            flux: None,
            flux_steps: 0,
            embed,
            initial: EdgeLedger {
                mass: 0,
                momentum: [0, 0],
                injected: [0, 0],
                census: [0; N_ORIENT],
                census_total: 0,
                red: 0,
                bonds: 0,
                rest: 0,
            },
            out_cells: vec![0u8; n_cells],
            out_orient: vec![NO_ORIENT; n_slots],
            out_donor: vec![NO_BOND; n_slots * MAX_ARMS],
            out_acceptor: vec![NO_BOND; n_slots * MAX_ACCEPTORS],
            out_colour: vec![0u8; n_cells],
            pre_slots: Vec::new(),
            keep: Vec::new(),
            claim: vec![NO_ORIENT; n_slots],
            claimed: Vec::new(),
            order: Vec::new(),
            blocked_idx: Vec::new(),
            dead: Vec::new(),
            finder: holon_closure::ComponentFinder::new(),
            comp_slots: Vec::new(),
            comp_bonds: Vec::new(),
            mine: vec![false; n_slots],
            from_slots: Vec::new(),
            to_slots: Vec::new(),
            payload: Vec::new(),
        };
        me.initial = me.ledger();
        me.audit = EdgeAudit::fresh(&me.initial);
        me
    }

    /// The seven-slot carrier at EDGE-0's law, on an occupation seeded by the caller.
    pub fn seeded(l: usize, seed: u64, density: f64, rules: EdgeRules) -> Self {
        let m = Model::fhp7();
        let laws = fiber_cycle_laws(&m);
        let lat = Lattice::seeded(m, l, seed, density, laws[0].clone());
        Self::from_lattice(lat, laws, TABLE_KEY ^ seed, seed, rules)
    }

    /// The carrier on an occupation the caller supplies cell by cell — the droplet, the slab
    /// and the two-phase scenes. `density(cell) -> per-slot occupation probability`.
    pub fn seeded_by(
        l: usize,
        seed: u64,
        rules: EdgeRules,
        density: impl Fn(usize, usize) -> f64,
    ) -> Self {
        let m = Model::fhp7();
        let n = m.n_dirs();
        let laws = fiber_cycle_laws(&m);
        let mut lat = Lattice::seeded(m, l, seed, 0.0, laws[0].clone());
        let cells: Vec<u8> = (0..l * l)
            .map(|c| {
                let p = density(c / l, c % l).clamp(0.0, 1.0);
                let mut s = 0u8;
                for d in 0..n {
                    let h = mix64(seed ^ (c as u64).wrapping_mul(GOLDEN) ^ ((d as u64) << 56));
                    if unit(h) < p {
                        s |= 1 << d;
                    }
                }
                s
            })
            .collect();
        lat.cells = cells;
        Self::from_lattice(lat, laws, TABLE_KEY ^ seed, seed, rules)
    }

    /// A colour plane attached, for a tracer readout.
    pub fn with_colour(mut self, colour: Vec<u8>) -> Self {
        assert_eq!(colour.len(), self.cells.len());
        assert!(
            colour.iter().zip(&self.cells).all(|(&q, &s)| q & !s == 0),
            "a colour bit without a particle under it"
        );
        self.colour = Some(colour);
        self.initial = self.ledger();
        self.audit = EdgeAudit::fresh(&self.initial);
        self
    }

    /// Switch the per-cell momentum-flux accumulator on and clear it.
    pub fn with_flux(mut self) -> Self {
        self.flux = Some(vec![[0.0; 4]; self.cells.len()]);
        self.flux_steps = 0;
        self
    }

    pub fn clear_flux(&mut self) {
        if let Some(f) = &mut self.flux {
            f.fill([0.0; 4]);
        }
        self.flux_steps = 0;
    }

    /// Re-baseline the ledger to the CURRENT state and clear the audit and the counts. The
    /// conserved integers are conserved from whatever instant the baseline is taken at, so
    /// this moves the start of the window and never the check.
    pub fn reset_initial(&mut self) {
        self.injected = [0, 0];
        self.initial = self.ledger();
        self.audit = EdgeAudit::fresh(&self.initial);
        self.counts = EdgeCounts::default();
    }

    pub fn model(&self) -> &Model {
        &self.tables.model
    }

    pub fn tables(&self) -> &Lattice {
        &self.tables
    }

    pub fn embedding(&self) -> &[[f64; 2]] {
        &self.embed
    }

    #[inline]
    fn neighbour(&self, cell: usize, dir: usize) -> usize {
        self.tables.neighbour_of(cell, dir)
    }

    /// The opposite of a MOVING direction. Defined on the six arms only; the rest slot has
    /// no opposite and never reaches here.
    #[inline]
    fn opposite6(dir: usize) -> usize {
        (dir + N_ORIENT / 2) % N_ORIENT
    }

    /// The direction arm `k` of a particle whose orientation is `o` points along.
    #[inline]
    pub fn arm_dir(o: usize, k: usize) -> usize {
        (o + 2 * k) % N_ORIENT
    }

    /// Every conserved integer right now, and the posted injection beside them.
    pub fn ledger(&self) -> EdgeLedger {
        let led = self.tables.ledger_of(&self.cells);
        let mut census = [0i64; N_ORIENT];
        for &o in &self.orient {
            if (o as usize) < N_ORIENT {
                census[o as usize] += 1;
            }
        }
        let red = match &self.colour {
            Some(q) => q.iter().map(|&b| b.count_ones() as i64).sum(),
            None => 0,
        };
        EdgeLedger {
            mass: led.mass,
            momentum: led.momentum,
            injected: self.injected,
            census,
            census_total: census.iter().sum(),
            red,
            bonds: self.bonds.len() as i64,
            rest: self.rest_count(),
        }
    }

    /// The momentum the ledger was baselined at — what `momentum(t) − Σ posted(t)` must equal.
    pub fn audit_initial_momentum(&self) -> [i64; 2] {
        self.initial.momentum
    }

    /// Occupied rest slots.
    pub fn rest_count(&self) -> i64 {
        match self.rest {
            None => 0,
            Some(r) => self.cells.iter().filter(|&&s| s >> r & 1 == 1).count() as i64,
        }
    }

    /// Occupied slots.
    pub fn particles(&self) -> usize {
        self.cells.iter().map(|&s| s.count_ones() as usize).sum()
    }

    /// The bond's link direction and the acceptor's angle index, DERIVED from the state.
    /// `None` when the recorded link is not a link, which is how a defect shows up as a
    /// released bond in the balance instead of a wrong reading.
    pub fn bond_geometry(&self, b: &Bond) -> Option<(usize, usize)> {
        let n = self.n;
        let (dc, dd) = (b.donor as usize / n, b.donor as usize % n);
        let (ac, ad) = (b.acceptor as usize / n, b.acceptor as usize % n);
        if self.cells[dc] >> dd & 1 == 0 || self.cells[ac] >> ad & 1 == 0 {
            return None;
        }
        let o = self.orient[b.donor as usize] as usize;
        if o >= N_ORIENT {
            return None;
        }
        let delta = Self::arm_dir(o, b.arm as usize);
        if self.neighbour(dc, delta) != ac {
            return None;
        }
        let oa = self.orient[b.acceptor as usize] as usize;
        if oa >= N_ORIENT {
            return None;
        }
        let phi = (oa + N_ORIENT - Self::opposite6(delta)) % N_ORIENT;
        Some((delta, phi))
    }

    /// Point both role tables at the pair list — WRITE ONLY, because every bond that left the
    /// list has already had its own two entries cleared by the pass that removed it. Called
    /// after any pass that changes the list, so an index can never point at a bond that has
    /// moved within it.
    fn reindex(&mut self) {
        for (i, b) in self.bonds.iter().enumerate() {
            self.donor_of[b.donor as usize * MAX_ARMS + b.arm as usize] = i as u32;
            self.acceptor_of[b.acceptor as usize * MAX_ACCEPTORS + b.acc as usize] = i as u32;
        }
    }

    /// Clear the two role entries one bond owns.
    #[inline]
    fn release_roles(&mut self, b: &Bond) {
        self.donor_of[b.donor as usize * MAX_ARMS + b.arm as usize] = NO_BOND;
        self.acceptor_of[b.acceptor as usize * MAX_ACCEPTORS + b.acc as usize] = NO_BOND;
    }

    /// One step: collide, gravity, form, break, stream.
    pub fn step(&mut self) {
        self.counts = EdgeCounts::default();
        let held_before = self.bonds.len() as i64;
        // The rest census costs one pass over the cells on each side of the collision, so it
        // is taken only where the audit is on. The W probe advances millions of single steps
        // with the audit off and must not pay for a count it never reads.
        let rest_before = if self.audit_every_step { self.rest_count() } else { 0 };

        self.collide();
        if self.audit_every_step {
            let after = self.rest_count();
            if after > rest_before {
                self.counts.rest_created = (after - rest_before) as u64;
            } else {
                self.counts.rest_destroyed = (rest_before - after) as u64;
            }
        }
        self.gravity_pass();
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
            self.audit.px_exact &=
                led.momentum[0] - led.injected[0] == self.initial.momentum[0];
            self.audit.py_exact &=
                led.momentum[1] - led.injected[1] == self.initial.momentum[1];
            self.audit.red_exact &= led.red == self.initial.red;
            self.audit.orientation_total_exact &= led.census_total == self.initial.census_total;
            self.audit.orientation_census_exact &= led.census == self.initial.census;
            self.audit.orientation_total = led.census_total;
            self.audit.injected = led.injected;
            self.audit.rest_final = led.rest;
            self.audit.steps_checked += 1;
        }
    }

    // ------------------------------------------------------------------ (1) collision
    /// The image the collision writes at `cell` — one table lookup for a deterministic law,
    /// and a per-cell, per-step draw over the table set otherwise. With ONE table this is
    /// `Lattice::collision_image`'s deterministic branch verbatim.
    #[inline]
    fn image(&self, cell: usize, s: u8, step: u64) -> u8 {
        if self.laws.len() == 1 {
            return self.laws[0][s as usize];
        }
        let h = mix64(self.law_key ^ (cell as u64).wrapping_mul(GOLDEN) ^ step.rotate_left(32));
        self.laws[(h % self.laws.len() as u64) as usize][s as usize]
    }

    fn collide(&mut self) {
        let step = self.step_index;
        let mut fired = 0u64;
        for c in 0..self.cells.len() {
            let s = self.cells[c];
            let t = self.image(c, s, step);
            if t == s {
                continue;
            }
            fired += 1;
            if let Some(q) = &mut self.colour {
                q[c] = recolour(s, t, q[c], self.n, ColourRule::Blind, c, step);
            }
            self.remap_cell(c, s, t, step);
            self.cells[c] = t;
        }
        self.counts.collisions_fired = fired;
    }

    /// Re-seat one cell's particles from the occupied slots of `s` onto those of `t`: BONDED
    /// first, ascending onto ascending, each keeping its orientation and every one of its
    /// roles; the free ones onto the rest, their orientations shuffled among themselves by a
    /// partial Fisher–Yates off a counter hash of `(cell, step)` alone.
    ///
    /// `orientation.rs`'s rule, extended from two roles to `MAX_ARMS + MAX_ACCEPTORS`.
    fn remap_cell(&mut self, c: usize, s: u8, t: u8, step: u64) {
        let n = self.n;
        let base = c * n;
        let mut ins = [0usize; 8];
        let mut n_in = 0usize;
        for d in 0..n {
            if s >> d & 1 == 1 {
                ins[n_in] = d;
                n_in += 1;
            }
        }
        let mut outs = [0usize; 8];
        let mut n_out = 0usize;
        for d in 0..n {
            if t >> d & 1 == 1 {
                outs[n_out] = d;
                n_out += 1;
            }
        }
        debug_assert_eq!(n_in, n_out, "a sector-preserving law changed the occupancy");

        let mut o_in = [NO_ORIENT; 8];
        let mut d_in = [[NO_BOND; MAX_ARMS]; 8];
        let mut a_in = [[NO_BOND; MAX_ACCEPTORS]; 8];
        for k in 0..n_in {
            let sl = base + ins[k];
            o_in[k] = self.orient[sl];
            for r in 0..MAX_ARMS {
                d_in[k][r] = self.donor_of[sl * MAX_ARMS + r];
                self.donor_of[sl * MAX_ARMS + r] = NO_BOND;
            }
            for r in 0..MAX_ACCEPTORS {
                a_in[k][r] = self.acceptor_of[sl * MAX_ACCEPTORS + r];
                self.acceptor_of[sl * MAX_ACCEPTORS + r] = NO_BOND;
            }
            self.orient[sl] = NO_ORIENT;
        }

        let bonded = |k: usize| -> bool {
            d_in[k].iter().any(|&x| x != NO_BOND) || a_in[k].iter().any(|&x| x != NO_BOND)
        };
        let mut seat = [0usize; 8];
        let mut free_o = [NO_ORIENT; 8];
        let mut n_free = 0usize;
        let mut j = 0usize;
        for k in 0..n_in {
            if bonded(k) {
                seat[j] = k;
                j += 1;
            } else {
                free_o[n_free] = o_in[k];
                n_free += 1;
            }
        }
        let n_bonded = j;
        if n_free > 1 {
            let seed =
                mix64(ORIENT_MIX_KEY ^ (c as u64).wrapping_mul(GOLDEN) ^ step.rotate_left(32));
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
                for r in 0..MAX_ARMS {
                    if d_in[k][r] != NO_BOND {
                        self.donor_of[sl * MAX_ARMS + r] = d_in[k][r];
                        self.bonds[d_in[k][r] as usize].donor = sl as u32;
                    }
                }
                for r in 0..MAX_ACCEPTORS {
                    if a_in[k][r] != NO_BOND {
                        self.acceptor_of[sl * MAX_ACCEPTORS + r] = a_in[k][r];
                        self.bonds[a_in[k][r] as usize].acceptor = sl as u32;
                    }
                }
            } else {
                self.orient[sl] = free_o[f];
                f += 1;
            }
        }
    }

    // ------------------------------------------------------------------ (2) gravity
    /// Move one particle within a cell from slot `from` to slot `to`, carrying its
    /// orientation, its colour bit and EVERY one of its roles, and rewriting the recorded
    /// slot of every bond that touches it. The caller has checked `to` is empty.
    fn move_slot(&mut self, cell: usize, from: usize, to: usize) {
        let n = self.n;
        let (a, b) = (cell * n + from, cell * n + to);
        self.orient[b] = self.orient[a];
        self.orient[a] = NO_ORIENT;
        for r in 0..MAX_ARMS {
            let idx = self.donor_of[a * MAX_ARMS + r];
            self.donor_of[b * MAX_ARMS + r] = idx;
            self.donor_of[a * MAX_ARMS + r] = NO_BOND;
            if idx != NO_BOND {
                self.bonds[idx as usize].donor = b as u32;
            }
        }
        for r in 0..MAX_ACCEPTORS {
            let idx = self.acceptor_of[a * MAX_ACCEPTORS + r];
            self.acceptor_of[b * MAX_ACCEPTORS + r] = idx;
            self.acceptor_of[a * MAX_ACCEPTORS + r] = NO_BOND;
            if idx != NO_BOND {
                self.bonds[idx as usize].acceptor = b as u32;
            }
        }
        self.cells[cell] = (self.cells[cell] & !(1 << from)) | (1 << to);
        if let Some(q) = &mut self.colour {
            let bit = q[cell] >> from & 1;
            q[cell] &= !(1 << from);
            q[cell] |= bit << to;
        }
    }

    fn gravity_pass(&mut self) {
        let g = match self.rules.gravity {
            Some(g) => g,
            None => return,
        };
        let step = self.step_index;
        let anti = Self::opposite6(g.dir);
        let dg = self.tables.model.dirs[g.dir];
        let mut flip = 0u64;
        let mut from_rest = 0u64;
        let mut refused = 0u64;
        let mut posted = [0i64; 2];
        for c in 0..self.cells.len() {
            let h = mix64(GRAVITY_KEY ^ (c as u64).wrapping_mul(GOLDEN) ^ step.rotate_left(32));
            if unit(h) >= g.rate {
                continue;
            }
            let s = self.cells[c];
            if s >> g.dir & 1 == 1 {
                refused += 1;
                continue;
            }
            if s >> anti & 1 == 1 {
                self.move_slot(c, anti, g.dir);
                posted[0] += 2 * dg[0];
                posted[1] += 2 * dg[1];
                flip += 1;
            } else if let Some(r) = self.rest.filter(|&r| s >> r & 1 == 1) {
                self.move_slot(c, r, g.dir);
                posted[0] += dg[0];
                posted[1] += dg[1];
                from_rest += 1;
            } else {
                refused += 1;
            }
        }
        self.injected[0] += posted[0];
        self.injected[1] += posted[1];
        self.counts.gravity_flip = flip;
        self.counts.gravity_rest = from_rest;
        self.counts.gravity_refused = refused;
    }

    // ------------------------------------------------------------------ (3) formation
    /// A greedy pass in ascending `(cell, slot, arm)` order; the acceptor is the lowest
    /// occupied slot of the target cell with a free acceptor role. The order is fixed and
    /// the run is reproducible bitwise, but the rule is sequential and is declared as such.
    fn form(&mut self) {
        let n = self.n;
        let mut formed = 0u64;
        for c in 0..self.cells.len() {
            let s = self.cells[c];
            if s == 0 {
                continue;
            }
            for a in 0..n {
                if s >> a & 1 == 0 {
                    continue;
                }
                let ds = c * n + a;
                let o = self.orient[ds] as usize;
                debug_assert!(o < N_ORIENT, "an occupied slot without an orientation");
                for arm in 0..self.rules.arms {
                    if self.donor_of[ds * MAX_ARMS + arm] != NO_BOND {
                        continue;
                    }
                    let delta = Self::arm_dir(o, arm);
                    let j = self.neighbour(c, delta);
                    let sj = self.cells[j];
                    if sj == 0 {
                        continue;
                    }
                    for b in 0..n {
                        if sj >> b & 1 == 0 {
                            continue;
                        }
                        let as_ = j * n + b;
                        let free = (0..self.rules.acceptors)
                            .find(|&k| self.acceptor_of[as_ * MAX_ACCEPTORS + k] == NO_BOND);
                        let k = match free {
                            Some(k) => k,
                            None => continue,
                        };
                        let idx = self.bonds.len() as u32;
                        self.bonds.push(Bond {
                            donor: ds as u32,
                            acceptor: as_ as u32,
                            arm: arm as u8,
                            acc: k as u8,
                        });
                        self.donor_of[ds * MAX_ARMS + arm] = idx;
                        self.acceptor_of[as_ * MAX_ACCEPTORS + k] = idx;
                        formed += 1;
                        break;
                    }
                }
            }
        }
        self.counts.formed = formed;
    }

    // ------------------------------------------------------------------ (4) the rent
    /// Only the first `n_start` bonds — those that existed at the start of the step — pay.
    /// A bond pays no rent on the step it forms, which is what makes the stationary held
    /// fraction `1/(1 + p_break)`.
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
                None => {
                    self.release_roles(b);
                    self.counts.blocked += 1;
                    self.counts.blocked_no_geometry += 1;
                    continue;
                }
            };
            self.counts.break_tests[phi] += 1;
            let p = self.rules.p_break(phi);
            // Keyed on the donor slot XOR the ARM, so the two bonds of a two-armed particle
            // draw independently; with `arm = 0` the key is FLUID-1's donor-slot key, bit for
            // bit, which is what keeps the one-arm carrier an identity against it.
            let key = (b.donor as u64) ^ ((b.arm as u64) << 56);
            let h = mix64(BREAK_KEY ^ key.wrapping_mul(GOLDEN) ^ step.rotate_left(32));
            if unit(h) < p {
                self.release_roles(b);
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

    // ------------------------------------------------------------------ (5) streaming
    fn stream(&mut self) {
        let n = self.n;
        let n_cells = self.cells.len();
        self.out_cells.fill(0);
        self.out_orient.fill(NO_ORIENT);
        self.out_donor.fill(NO_BOND);
        self.out_acceptor.fill(NO_BOND);
        let carry_colour = self.colour.is_some();
        if carry_colour {
            self.out_colour.fill(0);
        }

        // Pass 1 — the plain shift. A bijection on every plane (the rest plane's shift is the
        // identity), so mass and both momenta are already exact before any bond is consulted.
        for c in 0..n_cells {
            let s = self.cells[c];
            if s == 0 {
                continue;
            }
            let q = self.colour.as_ref().map(|q| q[c]).unwrap_or(0);
            for d in 0..n {
                if s >> d & 1 == 0 {
                    continue;
                }
                let dst = self.neighbour(c, d);
                self.out_cells[dst] |= 1 << d;
                let from = c * n + d;
                let to = dst * n + d;
                self.out_orient[to] = self.orient[from];
                for r in 0..MAX_ARMS {
                    self.out_donor[to * MAX_ARMS + r] = self.donor_of[from * MAX_ARMS + r];
                }
                for r in 0..MAX_ACCEPTORS {
                    self.out_acceptor[to * MAX_ACCEPTORS + r] =
                        self.acceptor_of[from * MAX_ACCEPTORS + r];
                }
                if carry_colour && q >> d & 1 == 1 {
                    self.out_colour[dst] |= 1 << d;
                }
            }
        }

        let mut pre = core::mem::take(&mut self.pre_slots);
        pre.clear();
        pre.reserve(self.bonds.len());
        for b in &self.bonds {
            pre.push((b.donor, b.acceptor));
        }
        for b in self.bonds.iter_mut() {
            let (dc, dd) = (b.donor as usize / n, b.donor as usize % n);
            let (ac, ad) = (b.acceptor as usize / n, b.acceptor as usize % n);
            b.donor = (self.tables.neighbour_of(dc, dd) * n + dd) as u32;
            b.acceptor = (self.tables.neighbour_of(ac, ad) * n + ad) as u32;
        }

        // Pass 2 — the joint moves. Rule 2 walks BONDS; rule 2b walks COMPONENTS.
        if self.rules.mover == MoverRule::Cluster {
            self.cluster_pass(&pre, carry_colour);
            self.pre_slots = pre;
            self.finish_stream(carry_colour);
            return;
        }


        // Rule 2 — over the BONDS in ascending post-shift donor slot, an order computed ONCE
        // before any hop.
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
            let arm = self.bonds[bi].arm;
            let (pre_d, pre_a) = pre[bi];
            let (i, a) = (pre_d as usize / n, pre_d as usize % n);
            let (j, bd) = (pre_a as usize / n, pre_a as usize % n);
            self.counts.joint_moves += 1;

            // The mover, keyed on the PRE-shift donor slot XOR the arm, so the draw does not
            // depend on the pass order and the two bonds of a two-armed particle draw apart.
            // With `arm = 0` the key is FLUID-1's, bit for bit. When both labels agree the
            // plain shift already moved the pair as one and there is nothing to draw.
            let drawn = if a == bd {
                a
            } else {
                let key = (pre_d as u64) ^ ((arm as u64) << 56);
                let h = mix64(MOVER_KEY ^ key.wrapping_mul(GOLDEN) ^ step.rotate_left(32));
                if h & 1 == 0 {
                    a
                } else {
                    bd
                }
            };
            // The mover SET: the drawn one first, then the wait — the same rule with the rest
            // direction admitted as a mover, whose displacement is [0,0].
            let mut candidates = [drawn, usize::MAX];
            let mut n_cand = 1usize;
            if self.rules.wait_enabled && a != bd {
                if let Some(r) = self.rest.filter(|&r| r != drawn) {
                    candidates[1] = r;
                    n_cand = 2;
                }
            }

            let mut taken: Option<(usize, usize, bool)> = None; // (mover, hops, was the wait)
            let mut last_conflict = false;
            for ci in 0..n_cand {
                let m = candidates[ci];
                if ci == 1 {
                    self.counts.wait_attempts += 1;
                }
                let mut conflict = false;
                let mut hops: [Option<(usize, usize, usize)>; 2] = [None, None];
                for (slot, (p, cellp, lp)) in
                    [(pre_d as usize, i, a), (pre_a as usize, j, bd)].iter().enumerate()
                {
                    let cl = self.claim[*p];
                    if cl == m as u8 {
                        continue;
                    }
                    if cl != NO_ORIENT {
                        conflict = true;
                        break;
                    }
                    if *lp != m {
                        hops[slot] =
                            Some((self.neighbour(*cellp, *lp), self.neighbour(*cellp, m), *lp));
                    }
                }
                if conflict {
                    last_conflict = true;
                    continue;
                }
                // Every hop's target must be vacant BEFORE any of them is executed: a wait
                // moves both particles and a half-executed pair would not be a joint move.
                let mut ok = true;
                for h in hops.iter().flatten() {
                    if self.out_cells[h.1] >> h.2 & 1 == 1 {
                        ok = false;
                        break;
                    }
                }
                if !ok {
                    last_conflict = false;
                    continue;
                }
                let mut executed = 0usize;
                for h in hops.iter().flatten() {
                    let (src_cell, tgt_cell, lane) = *h;
                    debug_assert_eq!(
                        self.out_cells[src_cell] >> lane & 1,
                        1,
                        "the non-mover is not where the shift put it"
                    );
                    let from = src_cell * n + lane;
                    let to = tgt_cell * n + lane;
                    self.out_cells[tgt_cell] |= 1 << lane;
                    self.out_cells[src_cell] &= !(1 << lane);
                    self.out_orient[to] = self.out_orient[from];
                    self.out_orient[from] = NO_ORIENT;
                    for r in 0..MAX_ARMS {
                        let idx = self.out_donor[from * MAX_ARMS + r];
                        self.out_donor[to * MAX_ARMS + r] = idx;
                        self.out_donor[from * MAX_ARMS + r] = NO_BOND;
                        if idx != NO_BOND {
                            self.bonds[idx as usize].donor = to as u32;
                        }
                    }
                    for r in 0..MAX_ACCEPTORS {
                        let idx = self.out_acceptor[from * MAX_ACCEPTORS + r];
                        self.out_acceptor[to * MAX_ACCEPTORS + r] = idx;
                        self.out_acceptor[from * MAX_ACCEPTORS + r] = NO_BOND;
                        if idx != NO_BOND {
                            self.bonds[idx as usize].acceptor = to as u32;
                        }
                    }
                    if carry_colour {
                        let bit = self.out_colour[src_cell] >> lane & 1;
                        self.out_colour[src_cell] &= !(1 << lane);
                        self.out_colour[tgt_cell] |= bit << lane;
                    }
                    executed += 1;
                }
                taken = Some((m, executed, ci == 1));
                break;
            }

            match taken {
                None => {
                    blocked_idx.push(bi as u32);
                    if last_conflict {
                        self.counts.blocked_claimed += 1;
                    } else {
                        self.counts.blocked_no_vacancy += 1;
                    }
                }
                Some((m, executed, was_wait)) => {
                    self.counts.anomalous_hops += executed as u64;
                    if was_wait {
                        self.counts.waited += 1;
                    }
                    for p in [pre_d as usize, pre_a as usize] {
                        if self.claim[p] == NO_ORIENT {
                            self.claim[p] = m as u8;
                            claimed.push(p as u32);
                        }
                    }
                }
            }
        }

        self.claimed = claimed;
        self.order = order;
        self.pre_slots = pre;
        self.blocked_idx = blocked_idx;
        self.finish_stream(carry_colour);
    }

    // ------------------------------------------------------------ (5b) the cluster pass
    /// Rule 2b: the mover is drawn per CONNECTED COMPONENT, and the whole component moves,
    /// or the whole component waits, or the whole component is released.
    ///
    /// `pre[b]` is bond `b`'s pair of PRE-shift slots; `self.bonds` already carries the
    /// post-shift slots (the plain shift rewrote them). Every position below is computed from
    /// the pre-shift slot, because the component's move is a displacement of the ORIGINAL
    /// configuration and not a correction to the shifted one.
    fn cluster_pass(&mut self, pre: &[(u32, u32)], carry_colour: bool) {
        let n = self.n;
        let step = self.step_index;
        if self.bonds.is_empty() {
            return;
        }

        // (i) the components, over PRE-shift slots, on the shared union-find.
        self.finder.reset(self.cells.len() * n);
        for &(d, a) in pre.iter() {
            self.finder.union(d, a);
        }

        // (ii) group the members and the bonds by component root. Both lists are sorted by
        // (root, id), so the pass order depends on no hash and no traversal.
        let mut comp_slots = core::mem::take(&mut self.comp_slots);
        comp_slots.clear();
        for i in 0..self.finder.touched().len() {
            let s = self.finder.touched()[i];
            let r = self.finder.find(s);
            comp_slots.push((r, s));
        }
        comp_slots.sort_unstable();
        let mut comp_bonds = core::mem::take(&mut self.comp_bonds);
        comp_bonds.clear();
        for (bi, &(d, _)) in pre.iter().enumerate() {
            let r = self.finder.find(d);
            comp_bonds.push((r, bi as u32));
        }
        comp_bonds.sort_unstable();

        let mut blocked_idx = core::mem::take(&mut self.blocked_idx);
        blocked_idx.clear();
        let mut claimed = core::mem::take(&mut self.claimed);
        claimed.clear();
        let mut from_slots = core::mem::take(&mut self.from_slots);
        let mut to_slots = core::mem::take(&mut self.to_slots);
        let mut payload = core::mem::take(&mut self.payload);

        let mut si = 0usize;
        let mut bi = 0usize;
        let mut components = 0u64;
        let mut moves = 0u64;
        let mut waited = 0u64;
        let mut wait_attempts = 0u64;
        let mut move_refused = 0u64;
        let mut wait_refused = 0u64;
        let mut hops = 0u64;
        let mut largest = 0u64;

        while si < comp_slots.len() {
            let root = comp_slots[si].0;
            let s0 = si;
            while si < comp_slots.len() && comp_slots[si].0 == root {
                si += 1;
            }
            let members = &comp_slots[s0..si];
            while bi < comp_bonds.len() && comp_bonds[bi].0 < root {
                bi += 1;
            }
            let b0 = bi;
            while bi < comp_bonds.len() && comp_bonds[bi].0 == root {
                bi += 1;
            }
            components += 1;
            largest = largest.max(members.len() as u64);

            // (iii) the mover: one member picked uniformly by the counter hash, and ITS
            // label. Keyed on the component's LOWEST pre-shift slot — `members` is sorted, so
            // that is `members[0]` — which depends on no traversal order.
            let key = members[0].1 as u64;
            let h = mix64(MOVER_KEY ^ key.wrapping_mul(GOLDEN) ^ step.rotate_left(32));
            let pick = (h % members.len() as u64) as usize;
            let drawn = members[pick].1 as usize % n;

            // (iv) the two candidates: the drawn mover, then the wait.
            let mut taken: Option<(usize, bool)> = None;
            for ci in 0..2usize {
                let m = if ci == 0 {
                    drawn
                } else {
                    match self.rest.filter(|&r| self.rules.wait_enabled && r != drawn) {
                        Some(r) => {
                            wait_attempts += 1;
                            r
                        }
                        None => break,
                    }
                };
                from_slots.clear();
                to_slots.clear();
                for &(_, s) in members.iter() {
                    let (c, d) = (s as usize / n, s as usize % n);
                    from_slots.push((self.neighbour(c, d) * n + d) as u32);
                    to_slots.push((self.neighbour(c, m) * n + d) as u32);
                }
                // A target is admissible when it is vacant, or when it is a slot this same
                // component is vacating. A shift of a set onto itself is injective, and two
                // members cannot share a target (that would need one cell and one label), so
                // admitting on those two conditions keeps the whole map injective.
                for &f in from_slots.iter() {
                    self.mine[f as usize] = true;
                }
                let mut ok = true;
                for &to in to_slots.iter() {
                    let (c, d) = (to as usize / n, to as usize % n);
                    if self.out_cells[c] >> d & 1 == 1 && !self.mine[to as usize] {
                        ok = false;
                        break;
                    }
                }
                for &f in from_slots.iter() {
                    self.mine[f as usize] = false;
                }
                if !ok {
                    if ci == 0 {
                        move_refused += 1;
                    } else {
                        wait_refused += 1;
                    }
                    continue;
                }

                // (v) read every member out, then clear, then write: the source and target
                // sets overlap and an in-place walk would overwrite its own input.
                payload.clear();
                for &f in from_slots.iter() {
                    let f = f as usize;
                    let (c, d) = (f / n, f % n);
                    debug_assert_eq!(
                        self.out_cells[c] >> d & 1,
                        1,
                        "a member is not where the plain shift put it"
                    );
                    let mut dr = [NO_BOND; MAX_ARMS];
                    let mut ar = [NO_BOND; MAX_ACCEPTORS];
                    for r in 0..MAX_ARMS {
                        dr[r] = self.out_donor[f * MAX_ARMS + r];
                        self.out_donor[f * MAX_ARMS + r] = NO_BOND;
                    }
                    for r in 0..MAX_ACCEPTORS {
                        ar[r] = self.out_acceptor[f * MAX_ACCEPTORS + r];
                        self.out_acceptor[f * MAX_ACCEPTORS + r] = NO_BOND;
                    }
                    let col = if carry_colour { self.out_colour[c] >> d & 1 } else { 0 };
                    payload.push((self.out_orient[f], dr, ar, col));
                    self.out_orient[f] = NO_ORIENT;
                    self.out_cells[c] &= !(1 << d);
                    if carry_colour {
                        self.out_colour[c] &= !(1 << d);
                    }
                }
                for (k, &to) in to_slots.iter().enumerate() {
                    let to = to as usize;
                    let (c, d) = (to / n, to % n);
                    let (o, dr, ar, col) = payload[k];
                    self.out_cells[c] |= 1 << d;
                    self.out_orient[to] = o;
                    for r in 0..MAX_ARMS {
                        self.out_donor[to * MAX_ARMS + r] = dr[r];
                        if dr[r] != NO_BOND {
                            self.bonds[dr[r] as usize].donor = to as u32;
                        }
                    }
                    for r in 0..MAX_ACCEPTORS {
                        self.out_acceptor[to * MAX_ACCEPTORS + r] = ar[r];
                        if ar[r] != NO_BOND {
                            self.bonds[ar[r] as usize].acceptor = to as u32;
                        }
                    }
                    if carry_colour && col == 1 {
                        self.out_colour[c] |= 1 << d;
                    }
                    if to_slots[k] != from_slots[k] {
                        hops += 1;
                    }
                }
                // The claim is written for the flux read-off and the clear-down; under this
                // rule it is never a guard, because a particle is in exactly one component.
                for &(_, s) in members.iter() {
                    self.claim[s as usize] = m as u8;
                    claimed.push(s);
                }
                taken = Some((m, ci == 1));
                break;
            }

            match taken {
                Some((_, was_wait)) => {
                    if was_wait {
                        waited += 1;
                    } else {
                        moves += 1;
                    }
                }
                None => {
                    for &(_, b) in comp_bonds[b0..bi].iter() {
                        blocked_idx.push(b);
                    }
                }
            }
        }

        self.counts.joint_moves += components;
        self.counts.components = components;
        self.counts.component_moves = moves;
        self.counts.component_move_refused = move_refused;
        self.counts.component_wait_refused = wait_refused;
        self.counts.largest_component = largest;
        self.counts.wait_attempts += wait_attempts;
        self.counts.waited += waited;
        self.counts.anomalous_hops += hops;
        // A component refused both is released, and its two prongs are `component_move_refused`
        // and `component_wait_refused` above. Rule 2's own prongs (`blocked_no_vacancy`,
        // `blocked_claimed`) are LEFT AT ZERO here rather than reused: they mean "this bond had
        // no vacancy" and "this bond's particle was claimed by another bond", and neither
        // sentence is true under a rule that has no per-bond claim at all.

        self.comp_slots = comp_slots;
        self.comp_bonds = comp_bonds;
        self.blocked_idx = blocked_idx;
        self.claimed = claimed;
        self.from_slots = from_slots;
        self.to_slots = to_slots;
        self.payload = payload;
    }

    /// Release the bonds the mover pass refused, in either rule: their particles are exactly
    /// where the shift (or an admitted move) put them, their two role entries are cleared,
    /// and `blocked` counts them so the ledger's bond balance still closes.
    fn release_blocked(&mut self) {
        let blocked_idx = core::mem::take(&mut self.blocked_idx);
        if !blocked_idx.is_empty() {
            self.counts.blocked += blocked_idx.len() as u64;
            self.dead.clear();
            self.dead.resize(self.bonds.len(), false);
            for &bi in blocked_idx.iter() {
                let b = self.bonds[bi as usize];
                self.out_donor[b.donor as usize * MAX_ARMS + b.arm as usize] = NO_BOND;
                self.out_acceptor[b.acceptor as usize * MAX_ACCEPTORS + b.acc as usize] = NO_BOND;
                self.dead[bi as usize] = true;
            }
            let mut kept = core::mem::take(&mut self.keep);
            kept.clear();
            for (i, b) in self.bonds.iter().enumerate() {
                if !self.dead[i] {
                    kept.push(*b);
                }
            }
            self.keep = core::mem::replace(&mut self.bonds, kept);
        }
        self.blocked_idx = blocked_idx;
    }

    /// The flux read-off, the claim clear-down, the refused releases and the buffer swap —
    /// the tail both mover rules share, so there is ONE place where a step ends.
    ///
    /// Both rules record a displaced particle's direction in `claim`, indexed by its
    /// PRE-shift slot: rule 2 needs it to keep a particle from being displaced twice, rule 2b
    /// does not (a particle is in one component) and writes it for this read-off alone. Where
    /// no claim was written the particle took its own label's shift.
    fn finish_stream(&mut self, carry_colour: bool) {
        let n = self.n;
        if self.flux.is_some() {
            let mut flux = self.flux.take().unwrap();
            for c in 0..self.cells.len() {
                let s = self.cells[c];
                if s == 0 {
                    continue;
                }
                let acc = &mut flux[c];
                for d in 0..n {
                    if s >> d & 1 == 0 {
                        continue;
                    }
                    let cl = self.claim[c * n + d];
                    let disp = if cl == NO_ORIENT { d } else { cl as usize };
                    let dx = self.embed[disp];
                    let mom = self.embed[d];
                    acc[0] += dx[0] * mom[0];
                    acc[1] += dx[0] * mom[1];
                    acc[2] += dx[1] * mom[0];
                    acc[3] += dx[1] * mom[1];
                }
            }
            self.flux = Some(flux);
            self.flux_steps += 1;
        }
        let claimed = core::mem::take(&mut self.claimed);
        for &p in claimed.iter() {
            self.claim[p as usize] = NO_ORIENT;
        }
        self.claimed = claimed;
        self.release_blocked();
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
    /// The whole mutable state, for a probe that restores a base configuration.
    pub fn snapshot(&self) -> EdgeState {
        EdgeState {
            cells: self.cells.clone(),
            orient: self.orient.clone(),
            donor_of: self.donor_of.clone(),
            acceptor_of: self.acceptor_of.clone(),
            bonds: self.bonds.clone(),
            colour: self.colour.clone(),
            step_index: self.step_index,
            injected: self.injected,
        }
    }

    pub fn restore(&mut self, s: &EdgeState) {
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
        self.injected = s.injected;
        self.counts = EdgeCounts::default();
    }

    /// Clear both role tables and point them at the current pair list. The only caller is a
    /// scene built by writing `cells`, `orient` and `bonds` directly; every pass inside the
    /// step keeps the tables in step incrementally and must not use this.
    pub fn rebuild_index(&mut self) {
        self.donor_of.fill(NO_BOND);
        self.acceptor_of.fill(NO_BOND);
        self.reindex();
    }

    /// The fiber move on the occupation byte: the cell's local state goes to its cyclic
    /// successor within its own `(N,P)` fiber and the particles are re-seated in ascending
    /// order, each keeping its orientation and every role. The label is unchanged, so `v_b`
    /// is unchanged for EVERY `b` at once.
    pub fn fiber_perturb(&mut self, cell: usize) -> bool {
        let n = self.n;
        let s = self.cells[cell];
        let t = match self.tables.model.fiber_successor(s) {
            Some(t) => t,
            None => return false,
        };
        let base = cell * n;
        let mut o_in = [NO_ORIENT; 8];
        let mut d_in = [[NO_BOND; MAX_ARMS]; 8];
        let mut a_in = [[NO_BOND; MAX_ACCEPTORS]; 8];
        let mut k = 0usize;
        for d in 0..n {
            if s >> d & 1 == 1 {
                let sl = base + d;
                o_in[k] = self.orient[sl];
                for r in 0..MAX_ARMS {
                    d_in[k][r] = self.donor_of[sl * MAX_ARMS + r];
                    self.donor_of[sl * MAX_ARMS + r] = NO_BOND;
                }
                for r in 0..MAX_ACCEPTORS {
                    a_in[k][r] = self.acceptor_of[sl * MAX_ACCEPTORS + r];
                    self.acceptor_of[sl * MAX_ACCEPTORS + r] = NO_BOND;
                }
                self.orient[sl] = NO_ORIENT;
                k += 1;
            }
        }
        let mut colour_bits = 0u8;
        let mut j = 0usize;
        for d in 0..n {
            if t >> d & 1 == 1 {
                let sl = base + d;
                self.orient[sl] = o_in[j];
                for r in 0..MAX_ARMS {
                    if d_in[j][r] != NO_BOND {
                        self.donor_of[sl * MAX_ARMS + r] = d_in[j][r];
                        self.bonds[d_in[j][r] as usize].donor = sl as u32;
                    }
                }
                for r in 0..MAX_ACCEPTORS {
                    if a_in[j][r] != NO_BOND {
                        self.acceptor_of[sl * MAX_ACCEPTORS + r] = a_in[j][r];
                        self.bonds[a_in[j][r] as usize].acceptor = sl as u32;
                    }
                }
                colour_bits |= 1 << d;
                j += 1;
            }
        }
        if let Some(q) = &mut self.colour {
            let red = q[cell].count_ones();
            let mut r = 0u8;
            let mut left = red;
            for d in 0..n {
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

    /// The coarse chart at block size `b`, on this carrier's occupation bytes.
    pub fn chart(&self, b: usize) -> Option<crate::chart::Field> {
        BlockChart::new(b, self.l).map(|c| c.apply(&self.tables.model, &self.cells))
    }

    /// The bond graph's reading: bonds, particles, the largest component, and whether that
    /// component carries a non-contractible cycle. The union–find with potentials is
    /// `orientation.rs`'s rule, lifted to this carrier's bonds through `holon-closure`.
    pub fn phase(&self) -> holon_closure::Phase<2> {
        let index = self.occupied_slots();
        let id = |slot: u32| -> Option<u32> { index.binary_search(&slot).ok().map(|i| i as u32) };
        let dirs: Vec<[i32; 2]> =
            self.model().dirs.iter().map(|d| [d[0] as i32, d[1] as i32]).collect();
        let edges: Vec<holon_closure::PhaseEdge<2>> = self
            .bonds
            .iter()
            .filter_map(|b| {
                let (delta, _) = self.bond_geometry(b)?;
                Some(holon_closure::PhaseEdge {
                    a: id(b.donor)?,
                    b: id(b.acceptor)?,
                    displacement: dirs[delta],
                })
            })
            .collect();
        holon_closure::phase(index.len(), &edges)
    }

    /// The occupied slots, ascending — the tier's carrier index.
    pub fn occupied_slots(&self) -> Vec<u32> {
        let mut out = Vec::new();
        for c in 0..self.cells.len() {
            let s = self.cells[c];
            for d in 0..self.n {
                if s >> d & 1 == 1 {
                    out.push((c * self.n + d) as u32);
                }
            }
        }
        out
    }

    /// Bonds whose recorded geometry is intact — the same filter the phase uses.
    pub fn live_bonds(&self) -> usize {
        self.bonds.iter().filter(|b| self.bond_geometry(b).is_some()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orientation::{OrientationLattice, OrientationRules};

    fn amp() -> [f64; N_ORIENT] {
        [1.0, 1.337178, 1.288496, 1.420097, 1.288496, 1.337178]
    }

    fn edge_scene(l: usize, seed: u64, d: f64, rules: EdgeRules) -> EdgeLattice {
        EdgeLattice::seeded(l, seed, d, rules)
    }

    #[test]
    fn the_embedding_agrees_with_isotropys_on_six_directions() {
        let m = Model::fhp6();
        let a = hex_embed(&m);
        let b = crate::isotropy::embed(&m);
        assert_eq!(a, b, "two spellings of the hexagonal embedding disagree");
        let seven = hex_embed(&Model::fhp7());
        assert_eq!(seven[6], [0.0, 0.0], "the rest slot must embed at the origin");
        assert_eq!(&seven[..6], &a[..]);
    }

    /// E-I CONTAINS FHP-I and is strictly more collisional than it — measured on the tables,
    /// so the freeze's "the law is FHP-I's fiber move on a chart with one more slot, and it
    /// turns seven more fibers besides" is a checkable statement and not a claim.
    #[test]
    fn the_fiber_cycle_law_contains_fhp_i() {
        let m = Model::fhp6();
        let laws = fiber_cycle_laws(&m);
        assert_eq!(laws.len(), 3);
        assert_eq!(laws[0], m.identity_collision());
        // FHP-I's own five acting states: the two chiralities are E-I's successor and
        // predecessor there, exactly.
        for s in [9u8, 18, 21, 36, 42] {
            assert_eq!(laws[1][s as usize], m.fhp_i(true)[s as usize], "successor at {s}");
            assert_eq!(laws[2][s as usize], m.fhp_i(false)[s as usize], "predecessor at {s}");
        }
        let e_acts: Vec<u8> = (0..64u8).filter(|&s| laws[1][s as usize] != s).collect();
        let f_acts: Vec<u8> = (0..64u8).filter(|&s| m.fhp_i(true)[s as usize] != s).collect();
        assert_eq!(f_acts, vec![9, 18, 21, 36, 42]);
        assert_eq!(e_acts.len(), 20, "E-I must act on every movable six-direction state");
        assert!(f_acts.iter().all(|s| e_acts.contains(s)));
        // Every table of both charts is a conserving bijection, verified rather than assumed.
        for model in [Model::fhp6(), Model::fhp7()] {
            for law in fiber_cycle_laws(&model) {
                assert!(model.is_bijection(&law) && model.is_sector_preserving(&law));
            }
        }
    }

    /// G1's carrier: with ONE arm, ONE acceptor role, no rest slot, no wait and no gravity,
    /// this carrier and `orientation.rs`'s are the same object — cells, orientations, bonds
    /// and colour, bit for bit, over 300 steps of a bonded run.
    #[test]
    fn the_one_arm_no_rest_carrier_is_fluid_ones() {
        let l = 32;
        let seed = 0x464c_5549_4430;
        let m = Model::fhp6();
        let law = m.fhp_i(true);
        let o_rules = OrientationRules {
            bonds_enabled: true,
            amplitude: amp(),
            rent: OrientationRules::rent_from_retention(0.5526),
            cold: false,
            stream: true,
        };
        let e_rules = EdgeRules::fluid1(amp(), EdgeRules::rent_from_retention(0.5526));
        let base = Lattice::seeded(m.clone(), l, seed, 0.2, law.clone());
        let colour = base.seed_colour_wave(seed, 1.0, 1);
        let mut a = OrientationLattice::from_lattice(base.clone(), seed, o_rules)
            .with_colour(colour.clone());
        let mut b = EdgeLattice::from_lattice(base, vec![law], 0, seed, e_rules)
            .with_colour(colour);
        assert_eq!(a.cells, b.cells);
        assert_eq!(a.orient, b.orient);
        let mut bonded_steps = 0;
        for t in 0..300 {
            a.step();
            b.step();
            assert_eq!(a.cells, b.cells, "occupation diverged at step {t}");
            assert_eq!(a.orient, b.orient, "orientation diverged at step {t}");
            assert_eq!(a.colour, b.colour, "colour diverged at step {t}");
            assert_eq!(a.bonds.len(), b.bonds.len(), "bond count diverged at step {t}");
            for (x, y) in a.bonds.iter().zip(&b.bonds) {
                assert_eq!((x.donor, x.acceptor), (y.donor, y.acceptor), "a bond moved at {t}");
                assert_eq!(y.arm, 0, "a one-arm carrier grew a second arm");
            }
            if !a.bonds.is_empty() {
                bonded_steps += 1;
            }
        }
        assert!(bonded_steps > 250, "the identity was measured on a scene with no bonds in it");
        assert_eq!(a.audit.totals.formed, b.audit.totals.formed);
        assert_eq!(a.audit.totals.broken_rent, b.audit.totals.broken_rent);
        assert_eq!(a.audit.totals.blocked, b.audit.totals.blocked);
        assert!(a.audit.all_exact() && b.audit.all_exact());
        assert_eq!(b.audit.totals.waited, 0, "a carrier with no rest slot waited");
    }

    /// The ledger, on the full carrier: mass, both momenta up to the posted injection, the
    /// per-orientation census and the bond balance, at every step of a run with bonds, waits,
    /// rest particles and gravity all live.
    #[test]
    fn the_ledger_is_exact_up_to_the_posted_injection() {
        let mut rules = EdgeRules::edge0(amp(), EdgeRules::rent_from_retention(0.5526));
        rules.gravity = Some(Gravity { dir: 4, rate: 0.05 });
        let mut g = edge_scene(32, 0x0045_4447_4530, 0.25, rules);
        for _ in 0..400 {
            g.step();
        }
        assert!(
            g.audit.all_exact(),
            "the ledger is not exact: {:?}",
            g.audit.failing_legs()
        );
        assert_eq!(g.audit.steps_checked, 400);
        let t = g.audit.totals;
        assert!(t.formed > 0 && t.collisions_fired > 0, "the ledger passed on no work");
        assert!(t.gravity_flip > 0, "gravity's flip prong never fired");
        assert!(t.rest_created > 0, "the rest slot was never created");
        assert!(t.waited > 0, "the wait never fired on the scene that gates it");
        assert_ne!(g.injected, [0, 0], "gravity posted nothing");
        // The posted injection is what the ledger is exact UP TO, so it must be the whole of
        // the difference and not a fitted remainder.
        let led = g.ledger();
        assert_eq!(led.momentum[0] - led.injected[0], g.audit.px);
        assert_eq!(led.momentum[1] - led.injected[1], g.audit.py);
        assert_ne!(led.momentum, [g.audit.px, g.audit.py], "gravity moved no momentum at all");
    }

    /// Gravity with no gravity posts nothing and moves nothing — the control that says the
    /// injection is the body force and not the carrier leaking.
    #[test]
    fn no_gravity_posts_nothing() {
        let rules = EdgeRules::edge0(amp(), EdgeRules::rent_from_retention(0.5526));
        let mut g = edge_scene(24, 7, 0.3, rules);
        for _ in 0..120 {
            g.step();
        }
        assert_eq!(g.injected, [0, 0]);
        assert_eq!(g.audit.totals.gravity_flip + g.audit.totals.gravity_rest, 0);
        assert!(g.audit.all_exact());
    }

    /// The wait, exhibited on a KNOWN SCENE: one bonded pair whose two members carry
    /// different labels, with both of the drawn mover's hop targets blocked, so FLUID-1's
    /// rule has no vacancy under either draw and releases the bond. This carrier holds it.
    ///
    /// The scene is built by hand so the answer is known before the step runs: five
    /// particles, each alone in its cell (a one-particle state is alone in its `(N,P)` fiber,
    /// so the collision is the identity and cannot move anything), the pair at cells `36`
    /// and `44`, and one blocker for each of the two possible movers.
    #[test]
    fn the_wait_holds_a_pair_the_old_rule_would_have_released() {
        let scene = |wait: bool| -> (EdgeLattice, EdgeCounts) {
            let mut rules = EdgeRules::edge0(amp(), 40.0); // a rent this deep never breaks
            rules.wait_enabled = wait;
            let mut g = EdgeLattice::seeded_by(8, 1, rules, |_, _| 0.0);
            let n = g.n;
            let put = |g: &mut EdgeLattice, cell: usize, dir: usize, o: u8| {
                g.cells[cell] |= 1 << dir;
                g.orient[cell * n + dir] = o;
            };
            // The pair: the donor at cell 36 with LABEL 0 and its first arm along direction
            // 3, which points at cell 28; the acceptor there carries LABEL 1, so the two
            // labels differ and a joint move needs a hop. Every particle's second arm
            // (`o + 2`) is aimed at an empty cell, so the scene forms exactly ONE bond.
            put(&mut g, 36, 0, 3); // donor:    arms along 3 -> cell 28, and 5 -> cell 43
            put(&mut g, 28, 1, 2); // acceptor: arms along 2 -> cell 21, and 4 -> cell 27
            put(&mut g, 35, 1, 3); // blocks the m = 0 hop's target (cell 36, lane 1)
            put(&mut g, 29, 0, 0); // blocks the m = 1 hop's target (cell 37, lane 0)
            g.reset_initial();
            g.step();
            let c = g.counts;
            (g, c)
        };
        let (off, c_off) = scene(false);
        assert_eq!(c_off.formed, 1, "the known scene did not form its one bond");
        assert_eq!(c_off.joint_moves, 1);
        assert_eq!(c_off.blocked, 1, "FLUID-1's rule should release this pair");
        assert_eq!(c_off.waited, 0);
        assert_eq!(c_off.wait_attempts, 0, "a carrier with the wait off tried to wait");
        assert!(off.bonds.is_empty());
        assert!(off.audit.all_exact(), "{:?}", off.audit.failing_legs());

        let (on, c_on) = scene(true);
        assert_eq!(c_on.formed, 1);
        assert_eq!(c_on.joint_moves, 1);
        assert_eq!(c_on.wait_attempts, 1, "the wait was not tried on a refused joint move");
        assert_eq!(c_on.waited, 1, "the wait did not hold the pair");
        assert_eq!(c_on.blocked, 0, "the pair was released with the wait on");
        assert_eq!(c_on.anomalous_hops, 2, "a wait displaces BOTH members");
        assert_eq!(on.bonds.len(), 1);
        assert!(on.bond_geometry(&on.bonds[0]).is_some(), "the held bond is not a link");
        assert!(on.audit.all_exact(), "{:?}", on.audit.failing_legs());
        // Waiting means STAYING: both members are back in the cells they started in.
        assert_eq!(on.cells[36] & 1, 1, "the donor did not stay");
        assert_eq!(on.cells[28] >> 1 & 1, 1, "the acceptor did not stay");
        assert_eq!(on.ledger().momentum, off.ledger().momentum, "the wait moved momentum");
    }

    /// The wait's effect on a seeded run, as a RATE rather than as a difference between two
    /// trajectories: of the joint moves FLUID-1's rule would have released, the wait holds a
    /// measured fraction, and the bond count per particle rises with it.
    #[test]
    fn the_wait_saves_a_measured_fraction_of_the_releases() {
        let run = |wait: bool| -> (EdgeCounts, f64) {
            let mut rules = EdgeRules::edge0(amp(), EdgeRules::rent_from_retention(0.5526));
            rules.wait_enabled = wait;
            let mut g = EdgeLattice::seeded(32, 0xB0C5, 0.5, rules);
            for _ in 0..200 {
                g.step();
            }
            g.reset_initial();
            let mut acc = 0.0;
            for _ in 0..100 {
                g.step();
                acc += g.live_bonds() as f64 / g.particles() as f64;
            }
            assert!(g.audit.all_exact(), "{:?}", g.audit.failing_legs());
            (g.audit.totals, acc / 100.0)
        };
        let (off, count_off) = run(false);
        let (on, count_on) = run(true);
        assert_eq!(off.waited + off.wait_attempts, 0);
        assert!(on.wait_attempts > 0, "the wait was never tried");
        let saved = on.waited as f64 / on.wait_attempts as f64;
        assert!(saved > 0.0 && saved <= 1.0, "the wait saved {saved} of its attempts");
        assert!(
            count_on > count_off,
            "the wait did not raise the bond count: {count_on} against {count_off}"
        );
    }

    /// Two arms raise the ceiling: the each-bond-once count exceeds one, which the one-arm
    /// carrier cannot do by construction (FLUID-1's own finding, turned into a check).
    #[test]
    fn two_arms_lift_the_bond_count_over_the_one_arm_ceiling() {
        let count = |arms: usize, acceptors: usize| -> f64 {
            let mut rules = EdgeRules::edge0(amp(), EdgeRules::rent_from_retention(0.5526));
            rules.arms = arms;
            rules.acceptors = acceptors;
            let mut g = EdgeLattice::seeded(32, 0xA2, 0.35, rules);
            for _ in 0..200 {
                g.step();
            }
            let mut acc = 0.0;
            for _ in 0..50 {
                g.step();
                acc += g.live_bonds() as f64 / g.particles() as f64;
            }
            acc / 50.0
        };
        let one = count(1, 1);
        let two = count(2, 2);
        assert!(one <= 1.0, "a one-arm carrier broke its own ceiling: {one}");
        assert!(two > one, "two arms did not raise the count: {two} against {one}");
    }

    /// A held bond is a real link with its roles indexed, at every step — the invariant the
    /// whole pass list rests on.
    #[test]
    fn every_live_bond_is_a_real_link_with_its_roles_indexed() {
        let mut rules = EdgeRules::edge0(amp(), EdgeRules::rent_from_retention(0.5526));
        rules.gravity = Some(Gravity { dir: 1, rate: 0.1 });
        let mut g = edge_scene(24, 99, 0.3, rules);
        for t in 0..150 {
            g.step();
            for (i, b) in g.bonds.iter().enumerate() {
                assert!(g.bond_geometry(b).is_some(), "a dead bond in the list at step {t}");
                assert_eq!(
                    g.donor_of[b.donor as usize * MAX_ARMS + b.arm as usize],
                    i as u32,
                    "a donor role that does not index its bond at step {t}"
                );
                let base = b.acceptor as usize * MAX_ACCEPTORS;
                assert!(
                    (0..MAX_ACCEPTORS).any(|k| g.acceptor_of[base + k] == i as u32),
                    "an acceptor role that does not index its bond at step {t}"
                );
                assert_ne!(b.donor, b.acceptor);
            }
        }
    }

    /// Rule 2b, exhibited on a KNOWN SCENE: a chain of three bonded particles carrying THREE
    /// DIFFERENT labels, alone on the lattice so every target is vacant. Under rule 2 the
    /// three bonds draw their movers apart and at most one of them can be honoured; under
    /// rule 2b the component draws ONE mover and all three are displaced by it.
    #[test]
    fn a_component_moves_as_one_under_rule_2b() {
        let build = |mover: MoverRule| -> (EdgeLattice, EdgeCounts) {
            let mut rules = EdgeRules::edge0_cluster(amp(), 40.0); // a rent this deep never breaks
            rules.mover = mover;
            let mut g = EdgeLattice::seeded_by(8, 1, rules, |_, _| 0.0);
            let n = g.n;
            let put = |g: &mut EdgeLattice, cell: usize, dir: usize, o: u8| {
                g.cells[cell] |= 1 << dir;
                g.orient[cell * n + dir] = o;
            };
            // 36 -> 44 -> 52 along direction 0, with three different LABELS (1, 2, 3) so a
            // joint move needs every member displaced off its own plane's shift but one.
            put(&mut g, 36, 1, 0);
            put(&mut g, 44, 2, 0);
            put(&mut g, 52, 3, 4);
            g.reset_initial();
            g.step();
            let c = g.counts;
            (g, c)
        };

        let (g, c) = build(MoverRule::Cluster);
        assert_eq!(c.formed, 2, "the known scene did not form its two bonds");
        assert_eq!(c.components, 1, "the three particles are not one component");
        assert_eq!(c.largest_component, 3);
        assert_eq!(c.component_moves, 1, "the component did not move");
        assert_eq!(c.waited, 0, "the component waited where every target was vacant");
        assert_eq!(c.blocked, 0);
        assert_eq!(g.bonds.len(), 2, "the component's bonds were released");
        assert!(g.audit.all_exact(), "{:?}", g.audit.failing_legs());

        // Every member displaced by the SAME lattice vector, keeping its own label.
        let l = g.l as i64;
        let find = |g: &EdgeLattice, lab: usize| -> usize {
            (0..g.cells.len()).find(|&c| g.cells[c] >> lab & 1 == 1).expect("the label survived")
        };
        let delta = |from: usize, to: usize| -> (i64, i64) {
            (
                ((to / 8) as i64 - (from / 8) as i64).rem_euclid(l),
                ((to % 8) as i64 - (from % 8) as i64).rem_euclid(l),
            )
        };
        let d0 = delta(36, find(&g, 1));
        let d1 = delta(44, find(&g, 2));
        let d2 = delta(52, find(&g, 3));
        assert_eq!(d0, d1, "two members of one component moved apart");
        assert_eq!(d1, d2, "two members of one component moved apart");
        assert_eq!(g.ledger().momentum, g.audit_initial_momentum());

        // The control: rule 2 on the same scene cannot hold all three, and says so.
        let (_, c2) = build(MoverRule::Pair);
        assert_eq!(c2.formed, 2);
        assert_eq!(c2.components, 0, "rule 2 walks bonds and reports no components");
        assert!(
            c2.blocked > 0 || c2.waited > 0,
            "rule 2 held a three-label chain with no wait and no release"
        );
    }

    /// The ledger under rule 2b, with bonds, waits, rest particles and gravity all live — and
    /// the counts that say the rule did the thing it is for.
    #[test]
    fn rule_2b_keeps_the_ledger_and_builds_clusters() {
        let mut rules = EdgeRules::edge0_cluster(amp(), EdgeRules::rent_from_retention(0.999));
        rules.gravity = Some(Gravity { dir: 4, rate: 0.05 });
        let mut g = EdgeLattice::seeded(32, 0x0045_4447_4531, 0.3, rules);
        for _ in 0..300 {
            g.step();
        }
        assert!(g.audit.all_exact(), "{:?}", g.audit.failing_legs());
        let t = g.audit.totals;
        assert!(t.components > 0 && t.component_moves > 0);
        assert!(t.largest_component > 2, "the cluster rule never built a cluster");
        assert!(t.waited > 0, "the wait never fired under rule 2b");
        assert!(t.component_move_refused > 0, "no component was ever refused its drawn move");
        // Rule 2's per-bond prongs mean nothing here and are left at zero rather than reused.
        assert_eq!(t.blocked_no_vacancy, 0);
        assert_eq!(t.blocked_claimed, 0);
        assert_eq!(t.blocked_no_geometry, 0);
        // Every held bond is still a real link with its roles indexed.
        for (i, b) in g.bonds.iter().enumerate() {
            assert!(g.bond_geometry(b).is_some(), "a dead bond in the list");
            assert_eq!(g.donor_of[b.donor as usize * MAX_ARMS + b.arm as usize], i as u32);
            assert_eq!(
                g.acceptor_of[b.acceptor as usize * MAX_ACCEPTORS + b.acc as usize],
                i as u32
            );
        }
    }

    /// The cluster rule is ALL-OR-NOTHING, and this measures both ends of that on the same
    /// carrier at two densities.
    ///
    /// A component moves only if EVERY target is vacant or vacated by itself, so the rule's
    /// behaviour is set by how big the components are against how much free space there is.
    /// **Dilute (`d = 0.05`)**: the bond graph gels into one component that holds every bond —
    /// the release rate is zero and the bond count per particle is more than twice the pair
    /// rule's. **Dense (`d = 0.30`)**: the same giant component is refused both the move and
    /// the wait every step, so every bond is released and the count collapses BELOW the pair
    /// rule's. Neither end is coexistence, and the screen is what looks for a chart between
    /// them.
    #[test]
    fn the_cluster_rule_is_all_or_nothing_and_the_density_decides_which() {
        let run = |mover: MoverRule, d: f64| -> (EdgeCounts, f64, f64) {
            let mut rules = EdgeRules::edge0_cluster(amp(), EdgeRules::rent_from_retention(0.999));
            rules.mover = mover;
            let mut g = EdgeLattice::seeded(48, 0x0045_4447_4532, d, rules);
            for _ in 0..400 {
                g.step();
            }
            g.reset_initial();
            let (mut bonds, mut largest) = (0.0, 0.0);
            for _ in 0..40 {
                g.step();
                bonds += g.live_bonds() as f64 / g.particles() as f64;
                largest += g.phase().largest_fraction;
            }
            assert!(g.audit.all_exact(), "{:?}", g.audit.failing_legs());
            (g.audit.totals, bonds / 40.0, largest / 40.0)
        };
        let rate = |t: &EdgeCounts| t.blocked as f64 / t.formed.max(1) as f64;

        // Dilute: the cluster rule holds everything the pair rule throws away.
        let (p_lo, bonds_p_lo, _) = run(MoverRule::Pair, 0.05);
        let (c_lo, bonds_c_lo, largest_c_lo) = run(MoverRule::Cluster, 0.05);
        assert!(rate(&p_lo) > 0.9, "the pair rule held bonds at low density: {}", rate(&p_lo));
        assert!(rate(&c_lo) < 0.05, "the cluster rule released at low density: {}", rate(&c_lo));
        assert!(
            bonds_c_lo > 2.0 * bonds_p_lo,
            "the cluster rule held no more: {bonds_c_lo} against {bonds_p_lo}"
        );
        assert!(
            largest_c_lo > 0.5,
            "the dilute cluster rule did not gel into one component: {largest_c_lo}"
        );
        assert!(c_lo.largest_component > 2);

        // Dense: the giant component is refused every step and the count collapses.
        let (_, bonds_p_hi, _) = run(MoverRule::Pair, 0.30);
        let (c_hi, bonds_c_hi, _) = run(MoverRule::Cluster, 0.30);
        assert!(rate(&c_hi) > 0.9, "the dense cluster rule held bonds: {}", rate(&c_hi));
        assert!(
            bonds_c_hi < bonds_p_hi,
            "the dense cluster rule did not fall below the pair rule: {bonds_c_hi} against \
             {bonds_p_hi}"
        );
        assert!(c_hi.component_wait_refused > 0, "no component was ever refused its wait");
    }

    /// The rent clause reproduced on this carrier: on a HELD GEOMETRY (streaming off, one
    /// donor pointing at one acceptor and nothing else on the lattice) the tracked link is
    /// held on `1/(1 + p_break)` of the steps — FLUID-1's G3, measured on this carrier.
    ///
    /// The scene is built by hand rather than seeded, because the statistic is about ONE
    /// link and a seeded scene's link count moves under the rent. Both particles sit in the
    /// REST slot, whose one-particle state is alone in its fiber, so the collision is the
    /// identity and the geometry cannot drift under the measurement.
    #[test]
    fn the_chart_reproduces_the_retention_it_was_set_from() {
        let steps = 200_000usize;
        for (f, phi) in [(0.5526f64, 0usize), (0.5526, 2), (0.75, 2)] {
            let mut rules = EdgeRules::edge0(amp(), EdgeRules::rent_from_retention(f));
            rules.arms = 1;
            rules.acceptors = 1;
            rules.stream = false;
            let mut g = EdgeLattice::seeded_by(4, 0x5EED, rules, |_, _| 0.0);
            let n = g.n;
            let rest = g.rest.expect("the seven-slot chart has a rest slot");
            // The donor sits at cell 0 pointing along direction 0; the acceptor sits in the
            // cell that arm reaches, oriented so the angle is `phi`.
            let a_cell = 0usize;
            let delta = 0usize;
            let b_cell = g.tables().neighbour_of(a_cell, delta);
            g.cells[a_cell] = 1 << rest;
            g.cells[b_cell] = 1 << rest;
            g.orient[a_cell * n + rest] = delta as u8;
            let o_b = (phi + EdgeLattice::opposite6(delta)) % N_ORIENT;
            g.orient[b_cell * n + rest] = o_b as u8;
            g.reset_initial();
            let tracked = a_cell * n + rest;
            let mut held = 0u64;
            for _ in 0..steps {
                g.step();
                if g.donor_of[tracked * MAX_ARMS] != NO_BOND {
                    held += 1;
                }
            }
            assert!(g.audit.all_exact(), "{:?}", g.audit.failing_legs());
            let measured = held as f64 / steps as f64;
            let want = EdgeRules::retention(rules.p_break(phi));
            assert!(
                (measured - want).abs() / want < 0.02,
                "held fraction {measured} against 1/(1+p) = {want} at f={f}, phi={phi}"
            );
            assert!(
                g.audit.totals.break_tests[phi] > steps as u64 / 4,
                "the retention was measured on too few rent tests"
            );
        }
    }
}


