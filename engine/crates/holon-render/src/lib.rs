//! Interactive atom renderer: push hydrogen atoms together and watch H2 form, or fail to.
//!
//! The browser owns input and pixels and nothing else. This crate owns the potential,
//! the integrator, the ledger, and the bond predicate. The ABI below is raw
//! `extern "C"` scalars over a shared static — the same shape `holon-ball-game` and
//! `holon-sandbox` use, and for the same reason: no wasm-bindgen, no glue generator,
//! no dependency the size profile has to pay for.
//!
//! # What the app is for
//!
//! Two hydrogen atoms approaching from far apart CANNOT bond, however hard they are
//! pushed. Their relative energy starts at or above the dissociation asymptote and the
//! dynamics conserve it, so they always climb the repulsive wall and come back out.
//! Making H2 requires taking energy away — a third atom to carry it off, a thermostat,
//! or the user's own spring used as a brake — and the ledger reports exactly how much
//! left and by which route. The conservation gates are what make that claim checkable
//! rather than decorative.
//!
//! # The curve
//!
//! Every force and energy comes from `h2_potential.json` (see `table.rs` for the
//! contract). The file is data: replacing it with a different curve changes the physics
//! and touches no code. Until the exact table lands, the shipped file is a Morse
//! placeholder, labelled as one in its own `provenance` field and surfaced as a banner
//! in the viewer.

// The JSON reader is NATIVE ONLY. The browser has a JSON parser already and pushes
// knots through the ABI below; shipping a second one inside the wasm would be pure
// weight. This cfg is what makes the module header's claim true rather than aspirational.
pub mod clock;
pub mod holon;
#[cfg(not(target_arch = "wasm32"))]
pub mod json;
pub mod sim;
pub mod table;

use sim::{Boundary, Sim};
use std::sync::{Mutex, MutexGuard};

static SIM: Mutex<Sim> = Mutex::new(Sim::empty());

fn sim() -> MutexGuard<'static, Sim> {
    SIM.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

// ------------------------------------------------------------------ table loading
//
// The host parses the JSON and pushes knots one at a time. Three calls, in order:
// begin(n), knot(i, r, e, f) for each i, finish(r_e, d_e, e_asymptote).

#[no_mangle]
pub extern "C" fn holon_table_begin(count: u32) -> u32 {
    u32::from(sim().table.begin(count as usize))
}

#[no_mangle]
pub extern "C" fn holon_table_knot(index: u32, r: f64, e: f64, f: f64) -> u32 {
    u32::from(sim().table.knot(index as usize, r, e, f))
}

/// Returns the `LoadStatus` discriminant: 1 = Ok, anything else is a refusal.
#[no_mangle]
pub extern "C" fn holon_table_finish(r_e: f64, d_e: f64, e_asymptote: f64) -> u32 {
    let mut s = sim();
    let status = s.table.finish(r_e, d_e, e_asymptote);
    if status == table::LoadStatus::Ok {
        // Every clock is a function of the curve, so a new curve re-derives all of them
        // here rather than leaving the previous table's timestep in place.
        s.adopt_table_timescale();
    }
    status_code(status)
}

/// Push the optional `d2E_hartree_per_bohr2` column for a knot. Call between `knot` and
/// `finish`; entirely optional (see `table.rs` on why the envelope does not need it).
#[no_mangle]
pub extern "C" fn holon_table_knot_curvature(index: u32, d2: f64) -> u32 {
    u32::from(sim().table.knot_curvature(index as usize, d2))
}

#[no_mangle]
pub extern "C" fn holon_table_has_curvature() -> u32 {
    u32::from(sim().table.has_supplied_curvature())
}

/// Worst relative disagreement between a supplied d2 column and the interpolant's own
/// curvature at the knots. Reported, never enforced: cubic Hermite is C1, so its
/// curvature is discontinuous at knots and a mismatch is expected structure.
#[no_mangle]
pub extern "C" fn holon_table_d2_mismatch() -> f64 {
    sim().table.d2_mismatch
}

fn status_code(status: table::LoadStatus) -> u32 {
    match status {
        table::LoadStatus::Empty => 0,
        table::LoadStatus::Ok => 1,
        table::LoadStatus::TooManyKnots => 2,
        table::LoadStatus::TooFewKnots => 3,
        table::LoadStatus::NotIncreasing => 4,
        table::LoadStatus::NotFinite => 5,
    }
}

#[no_mangle]
pub extern "C" fn holon_table_status() -> u32 {
    status_code(sim().table.status)
}

#[no_mangle]
pub extern "C" fn holon_table_knots() -> u32 {
    sim().table.knots() as u32
}

/// RMS mismatch between the file's derivatives and the secant slopes of its values,
/// under the assumed convention `dE/dR = -F`. Near zero for a consistent table.
#[no_mangle]
pub extern "C" fn holon_table_residual() -> f64 {
    sim().table.residual
}

/// The same statistic under `dE/dR = +F`. Should be LARGE; if it is the smaller of the
/// two, the file uses the opposite sign convention and the viewer says so.
#[no_mangle]
pub extern "C" fn holon_table_residual_alt() -> f64 {
    sim().table.residual_alt
}

#[no_mangle]
pub extern "C" fn holon_table_r_e() -> f64 {
    sim().table.r_e
}

#[no_mangle]
pub extern "C" fn holon_table_d_e() -> f64 {
    sim().table.d_e
}

#[no_mangle]
pub extern "C" fn holon_table_asymptote() -> f64 {
    sim().table.e_asymptote
}

#[no_mangle]
pub extern "C" fn holon_table_r_min() -> f64 {
    sim().table.r_min()
}

#[no_mangle]
pub extern "C" fn holon_table_r_max() -> f64 {
    sim().table.r_max()
}

/// Asymptote-zeroed pair energy at separation `r`. Exposed so the viewer can draw the
/// very curve the integrator is using, rather than a second copy of it in JS.
#[no_mangle]
pub extern "C" fn holon_curve_u(r: f64) -> f64 {
    sim().table.u(r)
}

#[no_mangle]
pub extern "C" fn holon_curve_force(r: f64) -> f64 {
    sim().table.force(r)
}

// ------------------------------------------------------------------ scene

/// Reset the scene, CLAMPED to what this device was measured to sustain. An atom count
/// the device cannot carry would be delivered as time dilation, which is a worse answer
/// than saying so.
#[no_mangle]
pub extern "C" fn holon_reset(n: u32) {
    let cap = holon_n_max() as usize;
    let mut s = sim();
    s.reset((n as usize).min(cap.max(2)));
}

#[no_mangle]
pub extern "C" fn holon_rebase() {
    sim().rebase();
}

// ------------------------------------------------------------------ the three clocks
//
// dt is DERIVED from the curve and is not settable. What the host may move is the
// sim-speed (clock 3) and, only behind the explicit toggle, the dt multiplier.

#[no_mangle]
pub extern "C" fn holon_dt() -> f64 {
    sim().dt()
}

#[no_mangle]
pub extern "C" fn holon_dt_reference() -> f64 {
    sim().timescale.dt_reference
}

/// Harmonic angular frequency at the well minimum, atomic units.
#[no_mangle]
pub extern "C" fn holon_omega_e() -> f64 {
    sim().timescale.omega_e
}

/// The envelope frequency the drift bound is actually derived from (fence 3).
#[no_mangle]
pub extern "C" fn holon_omega_env() -> f64 {
    sim().timescale.omega_env
}

#[no_mangle]
pub extern "C" fn holon_k_env() -> f64 {
    sim().timescale.k_env
}

/// Innermost separation reachable at the largest pair energy seen.
#[no_mangle]
pub extern "C" fn holon_r_inner() -> f64 {
    sim().timescale.r_inner
}

#[no_mangle]
pub extern "C" fn holon_period() -> f64 {
    sim().timescale.period
}

#[no_mangle]
pub extern "C" fn holon_period_fs() -> f64 {
    let s = sim();
    s.timescale.period * clock::AU_TO_FS
}

/// `omega_env * dt`: how much accuracy the current timestep buys, and how far it is from
/// the `omega*dt = 2` stability limit.
#[no_mangle]
pub extern "C" fn holon_omega_dt() -> f64 {
    sim().timescale.omega_dt()
}

/// `(omega_env * dt)^2 / 4` — recomputed live, never cached, so it cannot go stale when
/// dt moves.
#[no_mangle]
pub extern "C" fn holon_relative_drift_bound() -> f64 {
    sim().timescale.relative_drift_bound()
}

#[no_mangle]
pub extern "C" fn holon_sim_speed() -> f64 {
    sim().timescale.sim_speed_fs_per_wallsec
}

#[no_mangle]
pub extern "C" fn holon_set_sim_speed(fs_per_wall_second: f64) {
    if fs_per_wall_second.is_finite() && fs_per_wall_second > 0.0 {
        sim().timescale.sim_speed_fs_per_wallsec = fs_per_wall_second;
    }
}

/// The explicit user toggle for rung (ii). Off means accuracy is held and time dilates.
#[no_mangle]
pub extern "C" fn holon_set_allow_dt_growth(on: u32) {
    let mut s = sim();
    s.timescale.allow_dt_growth = on != 0;
    if on == 0 {
        // Returning to the exactness hold re-derives dt from the envelope immediately,
        // rather than leaving the enlarged step in place until something else moves.
        let e = s.timescale.e_rel_max;
        s.timescale.e_rel_max = f64::NEG_INFINITY;
        s.timescale.k_env = 0.0;
        let table = core::mem::replace(&mut s.table, table::PotentialTable::empty());
        s.timescale.refresh_envelope(&table, e);
        s.table = table;
    }
}

#[no_mangle]
pub extern "C" fn holon_allow_dt_growth() -> u32 {
    u32::from(sim().timescale.allow_dt_growth)
}

/// Grow dt to this multiple of the derived reference. Ignored unless the toggle is on.
#[no_mangle]
pub extern "C" fn holon_set_dt_multiplier(multiplier: f64) {
    sim().timescale.set_dt_multiplier(multiplier);
}

/// 0 Exact, 1 TimeDilated, 2 AccuracyDeclared, 3 Refused.
#[no_mangle]
pub extern "C" fn holon_rung() -> u32 {
    match sim().timescale.rung {
        clock::Rung::Exact => 0,
        clock::Rung::TimeDilated => 1,
        clock::Rung::AccuracyDeclared => 2,
        clock::Rung::Refused => 3,
    }
}

/// Delivered sim-speed over requested. 1.0 when nothing gave.
#[no_mangle]
pub extern "C" fn holon_dilation() -> f64 {
    sim().timescale.dilation
}

/// Does the current policy construct lawfully? 1 yes, 0 refused by `Policy::new`.
///
/// Exposed because the refusal IS the mechanism: `Degrade::Accuracy` under
/// `Hold::Exactness` is `PolicyError::AccuracyUnderExactness`, so the ladder cannot be
/// misconfigured into degrading accuracy silently.
#[no_mangle]
pub extern "C" fn holon_policy_ok(frame_budget_ms: f64) -> u32 {
    u32::from(sim().timescale.policy(frame_budget_ms).is_ok())
}

// ------------------------------------------------------------------ frame advance

/// Advance one frame of `wall_dt` MEASURED wall-seconds and close the grain.
///
/// The host passes the interval it actually observed between frames. Nothing here
/// assumes 60 Hz, or any Hz. Returns the substeps taken.
#[no_mangle]
pub extern "C" fn holon_advance_frame(wall_dt_seconds: f64) -> u32 {
    let mut s = sim();
    let budget = s.timescale.substep_budget(wall_dt_seconds);
    let plan = s.timescale.plan_frame(wall_dt_seconds, budget);
    s.step_frame(plan.substeps);
    plan.substeps
}

/// Advance a fixed number of substeps and close the grain. For tests and for the
/// calibration burst; the interactive path uses `holon_advance_frame`.
#[no_mangle]
pub extern "C" fn holon_step_frame(substeps: u32) {
    sim().step_frame(substeps);
}

// ------------------------------------------------------------------ device calibration

/// Run `substeps` of PURE PHYSICS at the calibration scene (N = 16, walls off, no grain
/// closure, no rendering), then restore the caller's scene.
///
/// The host times this with its own clock and calls `holon_set_calibration` with the
/// result, which is why no timer lives in here: `std::time` is not available on
/// wasm32-unknown-unknown, and a second timing path for native would be a second thing to
/// keep true.
#[no_mangle]
pub extern "C" fn holon_calibration_burst(substeps: u32) -> u32 {
    let mut s = sim();
    let restore_n = s.n;
    let restore_boundary = s.boundary;
    s.boundary = sim::Boundary::Open;
    s.reset(sim::MAX_ATOMS);
    for _ in 0..substeps {
        s.step();
    }
    s.boundary = restore_boundary;
    s.reset(restore_n);
    sim::MAX_ATOMS as u32
}

/// Record the measured throughput. `substeps_per_second` is what the host observed
/// during the burst at N = `holon_calibration_atoms()`.
#[no_mangle]
pub extern "C" fn holon_set_calibration(substeps_per_second: f64) {
    if !(substeps_per_second.is_finite() && substeps_per_second > 0.0) {
        return;
    }
    let mut s = sim();
    s.timescale.substeps_per_second = substeps_per_second;
    s.timescale.calibrated = true;
}

#[no_mangle]
pub extern "C" fn holon_calibration_atoms() -> u32 {
    sim::MAX_ATOMS as u32
}

#[no_mangle]
pub extern "C" fn holon_calibrated() -> u32 {
    u32::from(sim().timescale.calibrated)
}

#[no_mangle]
pub extern "C" fn holon_substeps_per_second() -> f64 {
    sim().timescale.substeps_per_second
}

/// Pair evaluations per second on this device: the calibration rate times the pair count
/// of the calibration scene. This is the quantity the O(N^2) force loop actually spends.
#[no_mangle]
pub extern "C" fn holon_pairs_per_second() -> f64 {
    let s = sim();
    let pairs = (sim::MAX_ATOMS * (sim::MAX_ATOMS - 1) / 2) as f64;
    s.timescale.substeps_per_second * pairs
}

#[no_mangle]
pub extern "C" fn holon_required_substeps_per_second() -> f64 {
    sim().timescale.required_substeps_per_second()
}

/// Largest atom count this device sustains at the current sim-speed and accuracy.
#[no_mangle]
pub extern "C" fn holon_n_max() -> f64 {
    let s = sim();
    if !s.timescale.calibrated {
        return sim::MAX_ATOMS as f64;
    }
    let pairs = (sim::MAX_ATOMS * (sim::MAX_ATOMS - 1) / 2) as f64;
    clock::n_max(
        s.timescale.substeps_per_second * pairs,
        s.timescale.required_substeps_per_second(),
    )
}

/// 0 = soft walls, 1 = open (no walls; exact translation invariance).
#[no_mangle]
pub extern "C" fn holon_set_boundary(mode: u32) {
    let mut s = sim();
    s.boundary = if mode == 0 {
        Boundary::Walls
    } else {
        Boundary::Open
    };
}

#[no_mangle]
pub extern "C" fn holon_width() -> f64 {
    sim().width
}

#[no_mangle]
pub extern "C" fn holon_height() -> f64 {
    sim().height
}

#[no_mangle]
pub extern "C" fn holon_wall_inset() -> f64 {
    sim().wall_inset
}

#[no_mangle]
pub extern "C" fn holon_atom_count() -> u32 {
    sim().n as u32
}

#[no_mangle]
pub extern "C" fn holon_atom_x(i: u32) -> f64 {
    let s = sim();
    let i = i as usize;
    if i < s.n {
        s.atoms[i].x
    } else {
        0.0
    }
}

#[no_mangle]
pub extern "C" fn holon_atom_y(i: u32) -> f64 {
    let s = sim();
    let i = i as usize;
    if i < s.n {
        s.atoms[i].y
    } else {
        0.0
    }
}

#[no_mangle]
pub extern "C" fn holon_atom_speed(i: u32) -> f64 {
    let s = sim();
    let i = i as usize;
    if i < s.n {
        (s.atoms[i].vx * s.atoms[i].vx + s.atoms[i].vy * s.atoms[i].vy).sqrt()
    } else {
        0.0
    }
}

#[no_mangle]
pub extern "C" fn holon_set_velocity(i: u32, vx: f64, vy: f64) {
    sim().set_velocity(i as usize, vx, vy);
}

#[no_mangle]
pub extern "C" fn holon_set_position(i: u32, x: f64, y: f64) {
    sim().set_position(i as usize, x, y);
}

// ------------------------------------------------------------------ pairs and bonds

#[no_mangle]
pub extern "C" fn holon_pair_count() -> u32 {
    sim().pair_count as u32
}

#[no_mangle]
pub extern "C" fn holon_pair_i(k: u32) -> u32 {
    let s = sim();
    let k = k as usize;
    if k < s.pair_count {
        s.pairs[k].i as u32
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn holon_pair_j(k: u32) -> u32 {
    let s = sim();
    let k = k as usize;
    if k < s.pair_count {
        s.pairs[k].j as u32
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn holon_pair_r(k: u32) -> f64 {
    let s = sim();
    let k = k as usize;
    if k < s.pair_count {
        s.pairs[k].r
    } else {
        0.0
    }
}

#[no_mangle]
pub extern "C" fn holon_pair_e_rel(k: u32) -> f64 {
    let s = sim();
    let k = k as usize;
    if k < s.pair_count {
        s.pairs[k].e_rel
    } else {
        0.0
    }
}

#[no_mangle]
pub extern "C" fn holon_pair_r_outer(k: u32) -> f64 {
    let s = sim();
    let k = k as usize;
    if k < s.pair_count {
        s.pairs[k].r_outer
    } else {
        0.0
    }
}

#[no_mangle]
pub extern "C" fn holon_pair_bonded(k: u32) -> u32 {
    let s = sim();
    let k = k as usize;
    if k < s.pair_count {
        u32::from(s.pairs[k].bonded)
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn holon_bonded_count() -> u32 {
    sim().bonded_count() as u32
}

// ------------------------------------------------------------------ THE LEDGER
//
// One reader per term. The viewer prints them separately because a single combined
// number cannot say which conservation law moved.

#[no_mangle]
pub extern "C" fn holon_e_kin() -> f64 {
    sim().e_kin
}

#[no_mangle]
pub extern "C" fn holon_e_pair() -> f64 {
    sim().e_pair
}

#[no_mangle]
pub extern "C" fn holon_e_wall() -> f64 {
    sim().e_wall
}

#[no_mangle]
pub extern "C" fn holon_e_spring() -> f64 {
    sim().e_spring
}

#[no_mangle]
pub extern "C" fn holon_w_ext() -> f64 {
    sim().w_ext
}

#[no_mangle]
pub extern "C" fn holon_energy() -> f64 {
    sim().energy()
}

/// `E - W_ext`: the quantity that must not move.
#[no_mangle]
pub extern "C" fn holon_ledger() -> f64 {
    sim().ledger()
}

#[no_mangle]
pub extern "C" fn holon_ledger_origin() -> f64 {
    sim().l0
}

#[no_mangle]
pub extern "C" fn holon_drift() -> f64 {
    sim().drift()
}

#[no_mangle]
pub extern "C" fn holon_drift_peak() -> f64 {
    sim().drift_peak
}

#[no_mangle]
pub extern "C" fn holon_drift_bound() -> f64 {
    sim().drift_bound()
}

#[no_mangle]
pub extern "C" fn holon_energy_gate() -> u32 {
    u32::from(sim().energy_gate())
}

// --- momentum, gated separately: one gate per conservation law, never combined ---

#[no_mangle]
pub extern "C" fn holon_momentum_x() -> f64 {
    sim().momentum().0
}

#[no_mangle]
pub extern "C" fn holon_momentum_y() -> f64 {
    sim().momentum().1
}

#[no_mangle]
pub extern "C" fn holon_momentum_residual() -> f64 {
    sim().momentum_residual()
}

#[no_mangle]
pub extern "C" fn holon_momentum_residual_peak() -> f64 {
    sim().momentum_residual_peak
}

#[no_mangle]
pub extern "C" fn holon_momentum_bound() -> f64 {
    sim().momentum_bound()
}

#[no_mangle]
pub extern "C" fn holon_momentum_gate() -> u32 {
    u32::from(sim().momentum_gate())
}

#[no_mangle]
pub extern "C" fn holon_time() -> f64 {
    sim().time
}

#[no_mangle]
pub extern "C" fn holon_steps() -> f64 {
    sim().steps as f64
}

#[no_mangle]
pub extern "C" fn holon_temperature() -> f64 {
    sim().temperature()
}

// ------------------------------------------------------------------ the hand

/// Index of the atom nearest to `(x, y)` within `radius`, or -1.
#[no_mangle]
pub extern "C" fn holon_nearest_atom(x: f64, y: f64, radius: f64) -> i32 {
    let s = sim();
    let mut best = -1i32;
    let mut best_d2 = radius * radius;
    for i in 0..s.n {
        let dx = s.atoms[i].x - x;
        let dy = s.atoms[i].y - y;
        let d2 = dx * dx + dy * dy;
        if d2 <= best_d2 {
            best_d2 = d2;
            best = i as i32;
        }
    }
    best
}

#[no_mangle]
pub extern "C" fn holon_grab(i: u32) {
    sim().grab(i as usize);
}

#[no_mangle]
pub extern "C" fn holon_move_anchor(x: f64, y: f64) {
    sim().move_anchor(x, y);
}

#[no_mangle]
pub extern "C" fn holon_release() {
    sim().release();
}

#[no_mangle]
pub extern "C" fn holon_grabbed() -> i32 {
    match sim().grabbed {
        Some(i) => i as i32,
        None => -1,
    }
}

#[no_mangle]
pub extern "C" fn holon_anchor_x() -> f64 {
    sim().anchor.0
}

#[no_mangle]
pub extern "C" fn holon_anchor_y() -> f64 {
    sim().anchor.1
}

// ------------------------------------------------------------------ thermostat

/// Weak Berendsen thermostat. OFF by default; the default scene is pure NVE, and the
/// energy it moves is posted to `w_ext` so the ledger closes with it on too.
#[no_mangle]
pub extern "C" fn holon_set_thermostat(on: u32, target_kelvin: f64) {
    let mut s = sim();
    s.thermostat_on = on != 0;
    if target_kelvin.is_finite() && target_kelvin >= 0.0 {
        s.target_temperature = target_kelvin;
    }
}

#[no_mangle]
pub extern "C" fn holon_thermostat_on() -> u32 {
    u32::from(sim().thermostat_on)
}

// ------------------------------------------------------------------ the holon census
//
// The recursive architecture, priced. Micro holons (atoms) cost the entire O(N^2) force
// loop; composite holons (molecules) cost a pass over pairs at grain boundaries only;
// candidate closures ride the force loop on numbers already in hand.

#[no_mangle]
pub extern "C" fn holon_census_atoms() -> u32 {
    sim().holons.census.atoms as u32
}

#[no_mangle]
pub extern "C" fn holon_census_molecules() -> u32 {
    sim().holons.census.molecules as u32
}

#[no_mangle]
pub extern "C" fn holon_census_candidates() -> u32 {
    sim().holons.census.candidate_evaluations as u32
}

#[no_mangle]
pub extern "C" fn holon_census_global_views() -> u32 {
    sim().holons.census.global_views as u32
}

#[no_mangle]
pub extern "C" fn holon_census_formations() -> f64 {
    sim().holons.census.formations as f64
}

#[no_mangle]
pub extern "C" fn holon_census_dissolutions() -> f64 {
    sim().holons.census.dissolutions as f64
}

/// Candidates that were BOUND by energy but refused a row because their measured closure
/// defect was too large. The number that says how often boundness and closure disagree.
#[no_mangle]
pub extern "C" fn holon_census_closure_rejections() -> f64 {
    sim().holons.census.closure_rejections as f64
}

/// Turn the composite layer off, so its frame cost can be MEASURED by difference rather
/// than asserted.
#[no_mangle]
pub extern "C" fn holon_set_census_enabled(on: u32) {
    sim().holons.enabled = on != 0;
}

#[no_mangle]
pub extern "C" fn holon_census_enabled() -> u32 {
    u32::from(sim().holons.enabled)
}

/// Total bond-sector energy across live rows. A VIEW of energy the global ledger already
/// holds — adding it to the global total would be double counting.
#[no_mangle]
pub extern "C" fn holon_bond_sector_energy() -> f64 {
    sim().holons.bond_sector_energy()
}

// --- individual composite rows: {members, ledger, formed_at, kind} ---

/// Number of LIVE rows. Row indices below are dense over live rows only.
#[no_mangle]
pub extern "C" fn holon_row_count() -> u32 {
    sim().holons.molecule_count() as u32
}

fn nth_live(s: &sim::Sim, k: usize) -> Option<holon::HolonRow> {
    s.holons.rows.iter().filter(|r| r.alive).nth(k).copied()
}

#[no_mangle]
pub extern "C" fn holon_row_member(k: u32, which: u32) -> u32 {
    let s = sim();
    match nth_live(&s, k as usize) {
        Some(r) if (which as usize) < r.member_count as usize => r.members[which as usize] as u32,
        _ => 0,
    }
}

#[no_mangle]
pub extern "C" fn holon_row_member_count(k: u32) -> u32 {
    let s = sim();
    nth_live(&s, k as usize).map_or(0, |r| r.member_count as u32)
}

/// 0 = Molecule. Extensible for SELECTOR-1 subsystem rows — see `holon.rs` on why an
/// extensible kind is schema compatibility and NOT lawful extension.
#[no_mangle]
pub extern "C" fn holon_row_kind(k: u32) -> u32 {
    let s = sim();
    match nth_live(&s, k as usize).map(|r| r.kind) {
        Some(holon::HolonKind::Molecule) => 0,
        None => u32::MAX,
    }
}

#[no_mangle]
pub extern "C" fn holon_row_e_bond(k: u32) -> f64 {
    let s = sim();
    nth_live(&s, k as usize).map_or(0.0, |r| r.e_bond)
}

#[no_mangle]
pub extern "C" fn holon_row_formed_at_time(k: u32) -> f64 {
    let s = sim();
    nth_live(&s, k as usize).map_or(0.0, |r| r.formed_at_time)
}

#[no_mangle]
pub extern "C" fn holon_row_formed_at_frame(k: u32) -> f64 {
    let s = sim();
    nth_live(&s, k as usize).map_or(0.0, |r| r.formed_at_frame as f64)
}

/// The row's measured one-step closure defect, as a fraction of the well depth.
#[no_mangle]
pub extern "C" fn holon_row_closure_defect(k: u32) -> f64 {
    let s = sim();
    nth_live(&s, k as usize).map_or(0.0, |r| r.closure_defect)
}

#[no_mangle]
pub extern "C" fn holon_row_closure_defect_at_formation(k: u32) -> f64 {
    let s = sim();
    nth_live(&s, k as usize).map_or(0.0, |r| r.closure_defect_at_formation)
}

#[no_mangle]
pub extern "C" fn holon_frame() -> f64 {
    sim().frame as f64
}
