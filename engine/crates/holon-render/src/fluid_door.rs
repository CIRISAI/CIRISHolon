//! THE FLUID ELEMENT BAND'S DOOR — FLUID-1's orientation lattice, driven from the page.
//!
//! The band was a fence carrying a finding. What it lacked was not a better sentence: it was
//! the instrument. `holon-lattice`'s [`OrientationLattice`] is that instrument — the carrier
//! `conformance/mesh/FLUID1_RESULTS.md` was read on — and this module is the ABI over it. It
//! adds no rule and changes none: `step` here is `OrientationLattice::step` there, and the
//! two clauses that module declares (a bond pays no rent on the step it forms; a joint move
//! needs a vacancy and a common mover) are its own, not this file's.
//!
//! # What the page gets, and what it may not conclude from it
//!
//! It gets the SAME MOTION at a smaller box. FLUID-1's readings were taken at `L = 256` over
//! millions of member-steps with a transport fit on each; a browser at `L = 128` for a few
//! thousand steps reads the LATTICE, not the transport coefficients, and the band says so.
//! What is live here is the structure — the bond count, the largest component, the conserved
//! integers — and the page prints FLUID-1's transport numbers beside it as the RECORD they
//! are, cited to their own lines. Nothing here re-derives a Schmidt number and nothing here
//! is entitled to.
//!
//! # THE NUMBERS ARE READ, NEVER TYPED
//!
//! The bond rule's amplitude table and the retention that sets the rent are CT-1's and
//! CT-2's, and they arrive the way every other shipped artifact on this page arrives: as a
//! committed record the host parses and pushes through a door
//! ([`holon_fluid_amplitude`], [`holon_fluid_rent_from_retention`]). There is no default
//! table in this file and none in the crate — [`holon_fluid_begin`] REFUSES until all six
//! angles and the retention have been pushed. That refusal is the whole point: a table typed
//! into Rust or into JavaScript would be a third copy of a measurement that already has two.
//!
//! # The ledger is the honesty
//!
//! `OrientationLattice` audits every step: mass, both momentum components, the red count, the
//! per-orientation census (not merely its total) and the bond balance
//! `formed − broken_rent − blocked = held`. [`holon_fluid_ledger_exact`] is that audit, and
//! the page draws it as a tick beside the integers rather than asserting exactness in prose.
//!
//! # Refusals
//!
//! Every door refuses the way the seam door does: `0` on success, [`FLUID_REFUSED`] `+ k`
//! with `k` naming the reason. No door panics — `OrientationRules::rent_from_retention`
//! asserts on a retention outside `(0,1)` and `Lattice::seeded` asserts on a bad `L` or
//! density, so both are guarded HERE, before the call, rather than by editing the instrument.

use holon_lattice::lattice::Lattice;
use holon_lattice::orientation::{
    bond_graph, BondGraph, OrientationLattice, OrientationLedger, OrientationRules, N_DIRS,
    NO_ORIENT,
};
use holon_lattice::state::Model;

use std::sync::Mutex;

/// The base of this door's refusal codes, above `SEAM_REFUSED`'s block so a host can tell a
/// fluid refusal from a seam one by the number alone.
pub const FLUID_REFUSED: u32 = 220;

/// `L` outside `[MIN_L, MAX_L]`. The floor is the block chart's smallest divisor of interest
/// and the ceiling is what a page can step at an interactive rate; both are refusals with a
/// number rather than a hang.
pub const REFUSE_BAD_L: u32 = FLUID_REFUSED + 1;
/// A density outside `(0, 1)`, or not finite.
pub const REFUSE_BAD_DENSITY: u32 = FLUID_REFUSED + 2;
/// No lattice has been begun, so there is nothing to act on.
pub const REFUSE_NO_LATTICE: u32 = FLUID_REFUSED + 3;
/// An angle index outside `0..6`, or an amplitude that is not finite and positive.
pub const REFUSE_BAD_AMPLITUDE: u32 = FLUID_REFUSED + 4;
/// A retention outside `(0, 1)`: it sets no chart, and `rent_from_retention` would assert.
pub const REFUSE_BAD_RETENTION: u32 = FLUID_REFUSED + 5;
/// The rule is incomplete — one of the six angles, or the retention, was never pushed.
pub const REFUSE_RULE_INCOMPLETE: u32 = FLUID_REFUSED + 6;

/// The smallest box this door builds. Below it the block chart has no divisor worth the name
/// and the bond graph is all boundary.
pub const MIN_L: usize = 8;
/// The largest. `L = 256` is FLUID-1's own box and the door will build it, so the page can
/// MEASURE the price of the record's box rather than assert it is out of reach; what the page
/// runs at is its own choice, taken from that measurement.
pub const MAX_L: usize = 256;

/// The rule, staged one push at a time before a lattice exists.
///
/// Held apart from the lattice because the pushes come from a fetched record and the box is
/// the page's zoom: re-beginning at another `L` must not require re-reading the artifact, and
/// re-reading the artifact must not silently keep half of a previous table.
struct Staged {
    amplitude: [f64; N_DIRS],
    have: [bool; N_DIRS],
    rent: f64,
    have_rent: bool,
}

impl Staged {
    const fn empty() -> Self {
        Self { amplitude: [f64::NAN; N_DIRS], have: [false; N_DIRS], rent: f64::NAN, have_rent: false }
    }
    fn complete(&self) -> bool {
        self.have_rent && self.have.iter().all(|&x| x)
    }
}

/// What [`holon_fluid_begin`] was given, kept verbatim so [`holon_fluid_no_bond`] can rebuild
/// the SAME scene with the bond rule switched. The control has to be the same seed, the same
/// box and the same law, or it is a different experiment rather than a control.
#[derive(Clone, Copy)]
struct Begin {
    l: usize,
    density: f64,
    seed: u64,
    chirality: bool,
    bonds_on: bool,
}

struct Fluid {
    begun: Begin,
    g: OrientationLattice,
    start: OrientationLedger,
    steps: u64,
    /// RGBA, `l * l * 4`, allocated once at begin and never resized, so the pointer a host
    /// took last frame is the pointer this frame writes.
    image: Vec<u8>,
    /// `(donor slot, acceptor slot)` pairs, capacity fixed at begin to the particle count —
    /// a particle holds one donor role, so the bond count can never exceed it.
    bonds: Vec<u32>,
    /// How many pairs of `bonds` the last [`holon_fluid_bonds_fill`] wrote, served by
    /// [`holon_fluid_bonds_live`]. Kept so a host that took the pointer last frame can ask
    /// how much of the buffer is current without filling it again — reading the whole
    /// capacity would draw bonds that were released three frames ago.
    bond_count: usize,
    graph: Option<BondGraph>,
}

static STAGED: Mutex<Staged> = Mutex::new(Staged::empty());
static FLUID: Mutex<Option<Fluid>> = Mutex::new(None);

fn staged() -> std::sync::MutexGuard<'static, Staged> {
    STAGED.lock().unwrap_or_else(|p| p.into_inner())
}

fn fluid() -> std::sync::MutexGuard<'static, Option<Fluid>> {
    FLUID.lock().unwrap_or_else(|p| p.into_inner())
}

/// Read a scalar off the live lattice, or a stated absence. One helper so that "no lattice"
/// is the SAME value everywhere and a host cannot meet three spellings of it.
fn read<T>(f: impl FnOnce(&Fluid) -> T, absent: T) -> T {
    match fluid().as_ref() {
        Some(x) => f(x),
        None => absent,
    }
}

/// Build the lattice `begun` describes under `rules`. Every input has been checked by the
/// caller; this is the one place the two crates meet.
fn build(begun: Begin, rules: OrientationRules) -> Fluid {
    let m = Model::fhp6();
    let law = m.fhp_i(begun.chirality);
    let lat = Lattice::seeded(m, begun.l, begun.seed, begun.density, law);
    // FLUID-0's own tracer plane, on FLUID-0's own routine. It is what makes the red count a
    // carrier with work in it rather than a zero that conserves trivially, and it is what the
    // no-bond identity is checked on: with bonds forbidden the occupation AND the colour are
    // `Lattice::advance_with_colour`'s, bit for bit.
    let colour = lat.seed_colour_wave(begun.seed, 1.0, 1);
    let g = OrientationLattice::from_lattice(lat, begun.seed, rules).with_colour(colour);
    let start = g.ledger();
    let n = begun.l * begun.l;
    let cap = start.mass.max(0) as usize;
    Fluid {
        begun,
        g,
        start,
        steps: 0,
        image: vec![0u8; n * 4],
        bonds: vec![0u32; 2 * cap],
        bond_count: 0,
        graph: None,
    }
}

fn rules_from(s: &Staged, bonds_on: bool) -> OrientationRules {
    OrientationRules {
        bonds_enabled: bonds_on,
        amplitude: s.amplitude,
        rent: s.rent,
        cold: false,
        stream: true,
    }
}

// ---------------------------------------------------------------- the rule, pushed in

/// Push `A(φ)` at one of the six lattice angles, `phi_index = φ / 60°`.
///
/// READ by the host out of `conformance/mesh/fluid1/amplitude_table.json`, which records for
/// every angle the CT record and field it came from. Nothing in this crate carries a default:
/// [`holon_fluid_begin`] refuses until all six are here.
#[no_mangle]
pub extern "C" fn holon_fluid_amplitude(phi_index: u32, a: f64) -> u32 {
    let i = phi_index as usize;
    if i >= N_DIRS || !a.is_finite() || a <= 0.0 {
        return REFUSE_BAD_AMPLITUDE;
    }
    let mut s = staged();
    s.amplitude[i] = a;
    s.have[i] = true;
    0
}

/// Set the chart from ONE measured retention: `E₀/kT_lat = ln(f / (1 − f))`, the value at
/// which a linear bond's stationary held fraction `1/(1 + p_break)` is `f`.
///
/// The arithmetic is [`OrientationRules::rent_from_retention`]'s, called rather than
/// reproduced. It ASSERTS outside `(0,1)`, and a panic in a wasm built with `panic = abort`
/// takes the page down, so the interval is checked here first and refused with a code.
#[no_mangle]
pub extern "C" fn holon_fluid_rent_from_retention(f: f64) -> u32 {
    if !f.is_finite() || f <= 0.0 || f >= 1.0 {
        return REFUSE_BAD_RETENTION;
    }
    let mut s = staged();
    s.rent = OrientationRules::rent_from_retention(f);
    s.have_rent = true;
    0
}

/// The staged `E₀/kT_lat`, or NaN before a retention was pushed.
#[no_mangle]
pub extern "C" fn holon_fluid_rent() -> f64 {
    let s = staged();
    if s.have_rent {
        s.rent
    } else {
        f64::NAN
    }
}

/// The staged `A(φ)` at one angle, or NaN. Read back so the page displays the number the
/// ENGINE holds rather than the number it parsed.
#[no_mangle]
pub extern "C" fn holon_fluid_amplitude_at(phi_index: u32) -> f64 {
    let i = phi_index as usize;
    let s = staged();
    if i < N_DIRS && s.have[i] {
        s.amplitude[i]
    } else {
        f64::NAN
    }
}

/// `p_break(φ) = exp(−A(φ)·E₀/kT_lat)` on the staged rule — the chart's own break
/// probability, from the crate's own `p_break`, never recomputed on the page.
#[no_mangle]
pub extern "C" fn holon_fluid_p_break(phi_index: u32) -> f64 {
    let i = phi_index as usize;
    let s = staged();
    if i >= N_DIRS || !s.complete() {
        return f64::NAN;
    }
    rules_from(&s, true).p_break(i)
}

/// 1 when all six angles and the retention have been pushed.
#[no_mangle]
pub extern "C" fn holon_fluid_rule_ready() -> u32 {
    u32::from(staged().complete())
}

// ---------------------------------------------------------------- the lattice

/// Build the orientation lattice: FHP-6 at `L × L`, seeded at `density` from `seed`, on
/// FHP-I's collision table at the given chirality, with the bond rule on or off.
///
/// The seed crosses the ABI as two `u32` halves because the scalar ABI this page uses carries
/// no `u64`; `seed = (hi << 32) | lo`.
///
/// `0` on success. Refuses `REFUSE_BAD_L`, `REFUSE_BAD_DENSITY`, `REFUSE_RULE_INCOMPLETE` —
/// and a refusal leaves the previous lattice, if any, exactly where it was.
#[no_mangle]
pub extern "C" fn holon_fluid_begin(
    l: u32,
    density: f64,
    seed_lo: u32,
    seed_hi: u32,
    chirality: u32,
    bonds_on: u32,
) -> u32 {
    let l = l as usize;
    if !(MIN_L..=MAX_L).contains(&l) {
        return REFUSE_BAD_L;
    }
    if !density.is_finite() || density <= 0.0 || density >= 1.0 {
        return REFUSE_BAD_DENSITY;
    }
    let s = staged();
    if !s.complete() {
        return REFUSE_RULE_INCOMPLETE;
    }
    let begun = Begin {
        l,
        density,
        seed: ((seed_hi as u64) << 32) | seed_lo as u64,
        chirality: chirality != 0,
        bonds_on: bonds_on != 0,
    };
    let rules = rules_from(&s, begun.bonds_on);
    drop(s);
    *fluid() = Some(build(begun, rules));
    0
}

/// THE CONTROL. `on != 0` forbids bonds; `on == 0` restores them.
///
/// It REBUILDS the scene from the same `(L, density, seed, chirality)` rather than flipping a
/// flag under a running configuration, because the control FLUID-1 reports is a run with
/// bonds forbidden from the first step — with bonds forbidden the occupation and the colour
/// plane are `Lattice::advance_with_colour`'s bit for bit, which a lattice carrying bonds
/// formed before the flip would not be. Flipping the flag live would produce a third thing
/// that is neither the chart nor the control, and the page would have no name for it.
#[no_mangle]
pub extern "C" fn holon_fluid_no_bond(on: u32) -> u32 {
    let s = staged();
    if !s.complete() {
        return REFUSE_RULE_INCOMPLETE;
    }
    let mut g = fluid();
    let Some(cur) = g.as_ref() else {
        return REFUSE_NO_LATTICE;
    };
    let begun = Begin { bonds_on: on == 0, ..cur.begun };
    let rules = rules_from(&s, begun.bonds_on);
    drop(s);
    *g = Some(build(begun, rules));
    0
}

/// 1 while the bond rule is on, 0 in the no-bond control, and 0 with no lattice.
#[no_mangle]
pub extern "C" fn holon_fluid_bonds_enabled() -> u32 {
    read(|x| u32::from(x.begun.bonds_on), 0)
}

/// Advance `n` steps. Returns the number of steps TAKEN — `0` when no lattice has been begun,
/// which is how a host distinguishes "refused" from "stepped and nothing happened".
#[no_mangle]
pub extern "C" fn holon_fluid_step(n: u32) -> u32 {
    let mut g = fluid();
    let Some(x) = g.as_mut() else { return 0 };
    for _ in 0..n {
        x.g.step();
    }
    x.steps += n as u64;
    x.graph = None;
    n
}

#[no_mangle]
pub extern "C" fn holon_fluid_l() -> u32 {
    read(|x| x.begun.l as u32, 0)
}

#[no_mangle]
pub extern "C" fn holon_fluid_density() -> f64 {
    read(|x| x.begun.density, f64::NAN)
}

#[no_mangle]
pub extern "C" fn holon_fluid_steps() -> f64 {
    read(|x| x.steps as f64, f64::NAN)
}

// ---------------------------------------------------------------- the ledger, exact
//
// Every one of these is an INTEGER the instrument's own audit checks at every step. They
// cross as `f64` because that is the only numeric type this page's ABI carries, and every one
// of them is far inside `2^53` at any `L` this door builds — so the page prints them as
// digits, not as approximations, and `holon_fluid_ledger_exact` is the audit that says it may.

#[no_mangle]
pub extern "C" fn holon_fluid_ledger_mass() -> f64 {
    read(|x| x.g.ledger().mass as f64, f64::NAN)
}

#[no_mangle]
pub extern "C" fn holon_fluid_ledger_px() -> f64 {
    read(|x| x.g.ledger().momentum[0] as f64, f64::NAN)
}

#[no_mangle]
pub extern "C" fn holon_fluid_ledger_py() -> f64 {
    read(|x| x.g.ledger().momentum[1] as f64, f64::NAN)
}

/// The orientation census TOTAL. The per-orientation census is conserved too and the audit
/// checks it entry by entry; the total is what the panel shows and
/// [`holon_fluid_ledger_census`] serves the entries.
#[no_mangle]
pub extern "C" fn holon_fluid_ledger_census_total() -> f64 {
    read(|x| x.g.ledger().census_total as f64, f64::NAN)
}

/// The count of particles carrying orientation `d`, `d` in `0..6`.
#[no_mangle]
pub extern "C" fn holon_fluid_ledger_census(d: u32) -> f64 {
    let d = d as usize;
    if d >= N_DIRS {
        return f64::NAN;
    }
    read(|x| x.g.ledger().census[d] as f64, f64::NAN)
}

#[no_mangle]
pub extern "C" fn holon_fluid_ledger_red() -> f64 {
    read(|x| x.g.ledger().red as f64, f64::NAN)
}

/// Bonds HELD right now.
#[no_mangle]
pub extern "C" fn holon_fluid_ledger_bonds() -> f64 {
    read(|x| x.g.ledger().bonds as f64, f64::NAN)
}

/// Bonds FORMED since the lattice was begun.
#[no_mangle]
pub extern "C" fn holon_fluid_ledger_formed() -> f64 {
    read(|x| x.g.audit.totals.formed as f64, f64::NAN)
}

/// Bonds released by the RENT CLAUSE since the lattice was begun.
#[no_mangle]
pub extern "C" fn holon_fluid_ledger_broken_rent() -> f64 {
    read(|x| x.g.audit.totals.broken_rent as f64, f64::NAN)
}

/// Bonds released because the pair's joint move was REFUSED — no vacancy in the non-mover's
/// plane, or a particle already claimed at another mover direction. Counted APART from a rent
/// break and never folded into one: FLUID-1 §5 is the finding that this is 36 % of the
/// releases at the chart and 100 % of them in the cold control.
#[no_mangle]
pub extern "C" fn holon_fluid_ledger_blocked() -> f64 {
    read(|x| x.g.audit.totals.blocked as f64, f64::NAN)
}

#[no_mangle]
pub extern "C" fn holon_fluid_ledger_collisions() -> f64 {
    read(|x| x.g.audit.totals.collisions_fired as f64, f64::NAN)
}

/// Steps the running audit has actually checked. A gate reporting PASS on zero work has not
/// passed, and the page shows this beside the tick for exactly that reason.
#[no_mangle]
pub extern "C" fn holon_fluid_ledger_steps_checked() -> f64 {
    read(|x| x.g.audit.steps_checked as f64, f64::NAN)
}

/// THE TICK. 1 when every conserved integer has held at every audited step — mass, both
/// momenta, the red count, the orientation TOTAL and the per-orientation census, and the bond
/// balance `formed − broken_rent − blocked = held`. 0 when any of them moved, and 0 with no
/// lattice.
#[no_mangle]
pub extern "C" fn holon_fluid_ledger_exact() -> u32 {
    read(|x| u32::from(x.g.audit.all_exact()), 0)
}

/// The ledger's value at the FIRST step, for the arm of the page that shows the integer is
/// the same one it started as rather than merely internally consistent. `which`:
/// 0 mass, 1 px, 2 py, 3 census total, 4 red.
#[no_mangle]
pub extern "C" fn holon_fluid_ledger_initial(which: u32) -> f64 {
    read(
        |x| match which {
            0 => x.start.mass as f64,
            1 => x.start.momentum[0] as f64,
            2 => x.start.momentum[1] as f64,
            3 => x.start.census_total as f64,
            4 => x.start.red as f64,
            _ => f64::NAN,
        },
        f64::NAN,
    )
}

// ---------------------------------------------------------------- the bond graph

fn ensure_graph(x: &mut Fluid) -> &BondGraph {
    if x.graph.is_none() {
        x.graph = Some(bond_graph(&x.g));
    }
    x.graph.as_ref().expect("just filled")
}

/// `B/N` — bonds counted ONCE, at their donor. FLUID-1 reports `0.289` in this convention and
/// notes its ceiling is 1 on the lattice (one donor arm per particle) against the liquid's 2.
/// The convention is the crate's, not this file's.
#[no_mangle]
pub extern "C" fn holon_fluid_bonds_per_particle() -> f64 {
    let mut g = fluid();
    match g.as_mut() {
        Some(x) => ensure_graph(x).bonds_per_particle_donor,
        None => f64::NAN,
    }
}

/// `2B/N` — the mean degree, the same bonds in the ceiling-of-2 convention.
#[no_mangle]
pub extern "C" fn holon_fluid_bonds_per_particle_degree() -> f64 {
    let mut g = fluid();
    match g.as_mut() {
        Some(x) => ensure_graph(x).bonds_per_particle_degree,
        None => f64::NAN,
    }
}

/// The largest connected component of the bond graph, as a fraction of the particles.
#[no_mangle]
pub extern "C" fn holon_fluid_largest_fraction() -> f64 {
    let mut g = fluid();
    match g.as_mut() {
        Some(x) => ensure_graph(x).largest_fraction,
        None => f64::NAN,
    }
}

/// 1 when the largest component SPANS — carries a non-contractible cycle around the torus,
/// the translation-invariant reading the freeze asks for, never the "touches both edges" one.
#[no_mangle]
pub extern "C" fn holon_fluid_spans() -> u32 {
    let mut g = fluid();
    match g.as_mut() {
        Some(x) => u32::from(ensure_graph(x).largest_spans),
        None => 0,
    }
}

/// Particles in the largest component.
#[no_mangle]
pub extern "C" fn holon_fluid_largest() -> f64 {
    let mut g = fluid();
    match g.as_mut() {
        Some(x) => ensure_graph(x).largest as f64,
        None => f64::NAN,
    }
}

// ---------------------------------------------------------------- what the page draws

/// The occupation bytes, one per cell, `cells[i * L + j]` with bit `d` set when direction `d`
/// of that cell is occupied. The buffer is the lattice's own and is allocated once at begin,
/// so the pointer is stable between frames.
#[no_mangle]
pub extern "C" fn holon_fluid_cells_ptr() -> *const u8 {
    match fluid().as_ref() {
        Some(x) => x.g.cells.as_ptr(),
        None => std::ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn holon_fluid_cells_len() -> u32 {
    read(|x| x.g.cells.len() as u32, 0)
}

/// The orientation of every slot, `orient[cell * 6 + dir]`, `0xFF` where the slot is empty.
#[no_mangle]
pub extern "C" fn holon_fluid_orient_ptr() -> *const u8 {
    match fluid().as_ref() {
        Some(x) => x.g.orient.as_ptr(),
        None => std::ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn holon_fluid_orient_len() -> u32 {
    read(|x| x.g.orient.len() as u32, 0)
}

/// Fill the bond buffer with the LIVE bonds and return how many there are: `2n` `u32`s,
/// `(donor slot, acceptor slot)` per bond, a slot being `cell * 6 + dir`.
///
/// Only bonds whose recorded link IS a link right now are written — `bond_geometry` is the
/// crate's own test of that, and drawing a bond whose link the dynamics has broken would be
/// drawing a stale index rather than the state.
#[no_mangle]
pub extern "C" fn holon_fluid_bonds_fill() -> u32 {
    let mut g = fluid();
    let Some(x) = g.as_mut() else { return 0 };
    let cap = x.bonds.len() / 2;
    let mut n = 0usize;
    for b in x.g.bonds.iter() {
        if n >= cap {
            break;
        }
        if x.g.bond_geometry(b).is_none() {
            continue;
        }
        x.bonds[2 * n] = b.donor;
        x.bonds[2 * n + 1] = b.acceptor;
        n += 1;
    }
    x.bond_count = n;
    n as u32
}

#[no_mangle]
pub extern "C" fn holon_fluid_bonds_ptr() -> *const u32 {
    match fluid().as_ref() {
        Some(x) => x.bonds.as_ptr(),
        None => std::ptr::null(),
    }
}

/// The bond buffer's CAPACITY in pairs — what a host may read without going past the end.
/// [`holon_fluid_bonds_fill`] returns how many of them are live this frame.
#[no_mangle]
pub extern "C" fn holon_fluid_bonds_capacity() -> u32 {
    read(|x| (x.bonds.len() / 2) as u32, 0)
}

/// How many pairs the last [`holon_fluid_bonds_fill`] wrote — the part of the buffer that is
/// CURRENT. Reading past it draws bonds that were released steps ago, which is why this is
/// served rather than left to a host to remember.
#[no_mangle]
pub extern "C" fn holon_fluid_bonds_live() -> u32 {
    read(|x| x.bond_count as u32, 0)
}

/// The axial integer offset of direction `d`, `which = 0` for the first coordinate and `1`
/// for the second. `Model::fhp6`'s own directions, which are `regplus::DIRECTIONS` — the set
/// `CIRISOntology/Core/Lattice.lean` pins — so the page draws the lattice the theorem is
/// about rather than a hexagon someone typed into JavaScript.
#[no_mangle]
pub extern "C" fn holon_fluid_dir_axial(d: u32, which: u32) -> f64 {
    let (d, w) = (d as usize, which as usize);
    if d >= N_DIRS || w >= 2 {
        return f64::NAN;
    }
    read(|x| x.g.model().dirs[d][w] as f64, f64::NAN)
}

/// The EUCLIDEAN embedding of direction `d` — `[a + b/2, (√3/2)·b]` on the hexagon, which is
/// `isotropy::embed`'s and is the shear that turns the axial array into the triangular
/// lattice on screen. `which = 0` for x, `1` for y.
#[no_mangle]
pub extern "C" fn holon_fluid_dir_euclidean(d: u32, which: u32) -> f64 {
    let (d, w) = (d as usize, which as usize);
    if d >= N_DIRS || w >= 2 {
        return f64::NAN;
    }
    read(
        |x| {
            let e = holon_lattice::isotropy::embed(x.g.model());
            e[d][w]
        },
        f64::NAN,
    )
}

/// Paint the lattice into an RGBA buffer of `L × L` pixels, one pixel per CELL, and return
/// the byte count.
///
/// The picture is two facts and no more: how full the cell is, and how much of what is in it
/// is BONDED. Opacity is the occupancy `n/6`; the hue runs from the free colour to the bonded
/// one with the bonded fraction. A cell with nothing in it is fully transparent, so the page's
/// own background shows through and an empty region reads as empty rather than as black.
///
/// One pixel per cell and not per slot: a slot picture would need six sub-pixels and would say
/// nothing the two numbers here do not, and the orientation of a single particle is not
/// legible at a pixel anyway. The orientation is served exactly, per slot, by
/// [`holon_fluid_orient_ptr`] for a host that wants to draw it.
#[no_mangle]
pub extern "C" fn holon_fluid_image_fill() -> u32 {
    let mut g = fluid();
    let Some(x) = g.as_mut() else { return 0 };
    let n = x.begun.l * x.begun.l;
    for c in 0..n {
        let s = x.g.cells[c];
        let occ = s.count_ones();
        let px = 4 * c;
        if occ == 0 {
            x.image[px] = 0;
            x.image[px + 1] = 0;
            x.image[px + 2] = 0;
            x.image[px + 3] = 0;
            continue;
        }
        let mut bonded = 0u32;
        for d in 0..N_DIRS {
            if s >> d & 1 == 0 {
                continue;
            }
            let slot = c * N_DIRS + d;
            if x.g.donor_of[slot] != holon_lattice::orientation::NO_BOND
                || x.g.acceptor_of[slot] != holon_lattice::orientation::NO_BOND
            {
                bonded += 1;
            }
        }
        let f = bonded as f64 / occ as f64;
        x.image[px] = (56.0 + 190.0 * f) as u8;
        x.image[px + 1] = (150.0 - 60.0 * f) as u8;
        x.image[px + 2] = (214.0 - 150.0 * f) as u8;
        x.image[px + 3] = (48.0 + 207.0 * (occ as f64 / N_DIRS as f64)) as u8;
    }
    (n * 4) as u32
}

#[no_mangle]
pub extern "C" fn holon_fluid_image_ptr() -> *const u8 {
    match fluid().as_ref() {
        Some(x) => x.image.as_ptr(),
        None => std::ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn holon_fluid_image_len() -> u32 {
    read(|x| x.image.len() as u32, 0)
}

/// The sentinel `orient` carries where a slot is EMPTY, exported so a host reading the
/// orientation plane does not have to know the number. It is not a direction; it is a hole.
#[no_mangle]
pub extern "C" fn holon_fluid_no_orient() -> u32 {
    NO_ORIENT as u32
}
