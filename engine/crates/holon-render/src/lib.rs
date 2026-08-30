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
//! Every force and energy comes from the potential table (see `table.rs` for the
//! contract), and there are two ways to fill it.
//!
//! The DEFAULT is `holon_table_generate`: `holon-chem` solves H2 in the STO-3G basis
//! exactly (full CI) from closed-form Gaussian integrals, differentiates it analytically
//! for forces and curvature, and finds its own equilibrium and dissociation asymptote —
//! in the browser, at load, in tens of milliseconds. The sandbox therefore does not
//! *play* a curve somebody computed; it *solves* the one it is showing, and the residual
//! against a pinned 50-digit referee is on the banner rather than in a footnote.
//!
//! The FALLBACK is `h2_potential.json` pushed through `holon_table_begin`/`knot`/
//! `finish`. It is still a supported mode: a host that cannot run the generator, or an
//! A/B against a different curve, wants it. Both routes end in the same interpolator and
//! the same validation, so nothing downstream can tell which one filled the table.

// The JSON reader is NATIVE ONLY. The browser has a JSON parser already and pushes
// knots through the ABI below; shipping a second one inside the wasm would be pure
// weight. This cfg is what makes the module header's claim true rather than aspirational.
pub mod bank;
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
    u32::from(sim().table_mut().begin(count as usize))
}

#[no_mangle]
pub extern "C" fn holon_table_knot(index: u32, r: f64, e: f64, f: f64) -> u32 {
    u32::from(sim().table_mut().knot(index as usize, r, e, f))
}

// ------------------------------------------------- the engine-computed curve
//
// The other way to fill the table, and the default one: rather than parsing somebody's
// file, compute the curve here. `holon-chem` solves H2 in the STO-3G basis exactly (full
// CI) from closed-form Gaussian integrals, differentiates it analytically for the forces
// and the curvature, and finds its own equilibrium and dissociation asymptote. The knots
// go straight into the same interpolator the file path fills, through the same
// validation, so nothing downstream can tell which route filled it — which is the point:
// the physics is one implementation, and the source of the numbers is a mode.

/// Compute the curve and load it, in one call. Returns the `LoadStatus` discriminant
/// (1 = Ok); 6 means the generator itself refused the request.
///
/// Allocation-free: `holon_chem::stream_table` hands over one knot at a time and each is
/// pushed as it arrives, so no copy of the curve exists anywhere but in the table. That
/// is what keeps this path out of the wasm's (absent) allocator.
#[no_mangle]
pub extern "C" fn holon_table_generate(r_min: f64, r_max: f64, count: u32) -> u32 {
    generate_table(&mut sim(), r_min, r_max, count as usize)
}

/// The Rust-side entry point behind [`holon_table_generate`], for a caller that owns its
/// own [`Sim`] rather than driving the static one through the ABI — the 3D shell in
/// `holon-render-3d` is that caller.
///
/// Defined once and called by the export above rather than the other way round, because
/// two copies of "generate, validate, adopt the clocks" is exactly how one of them ends
/// up forgetting to adopt the clocks. Returns the same status code the ABI returns.
pub fn generate_table(s: &mut Sim, r_min: f64, r_max: f64, count: usize) -> u32 {
    // H2 goes into the H-H slot BY NAME rather than into "the table", because in a bank
    // there is no such thing as the table. `Sim::empty` seeds hydrogen as species 0, so
    // for every scene this function has ever been called on, the slot it targets is the
    // one the single-table sandbox filled.
    let Some(slot) = s.bank.slot_of_z(1, 1) else {
        return GENERATOR_REFUSED;
    };
    if !s.bank.table_slot_mut(slot).begin(count) {
        return status_code(s.bank.table_slot(slot).status);
    }
    let meta = holon_chem::stream_table(r_min, r_max, count, |i, r, e, f, e2| {
        let t = s.bank.table_slot_mut(slot);
        t.knot(i, r, e, f) && t.knot_curvature(i, e2)
    });
    let Some(meta) = meta else {
        return GENERATOR_REFUSED;
    };
    let status = s
        .bank
        .table_slot_mut(slot)
        .finish(meta.r_e, meta.d_e, meta.e_asymptote);
    if status == table::LoadStatus::Ok {
        // H2 in the STO-3G minimal basis is FOUR determinants on the determinant route.
        // The provenance is stamped from those facts rather than assumed, so the H2 curve
        // is graded by the same gate every other curve is.
        if let Err(r) = s.bank.commit(
            slot,
            bank::TableProvenance::solved_exact(4, 0.0),
            &bank::D1_RECORD,
            bank::Host::Browser,
        ) {
            return refusal_code(r);
        }
        // Every clock is a function of the curve, so a new curve re-derives all of them
        // here rather than leaving the previous table's timestep in place.
        s.adopt_table_timescale();
    }
    status_code(status)
}

/// The `LoadStatus`-space code a provenance refusal reports through the scalar ABI.
///
/// Distinct from every `LoadStatus` discriminant and from [`GENERATOR_REFUSED`], so a host
/// can tell "the curve would not parse" from "the curve parsed and was not allowed in".
/// The two are different problems and a single failure code would merge them.
pub const PROVENANCE_REFUSED: u32 = 16;

/// The refusal's own code, offset above [`PROVENANCE_REFUSED`] so the REASON survives the
/// trip through a `u32`. `holon_bank_refusal_reason` reads it back.
pub fn refusal_code(r: bank::Refusal) -> u32 {
    PROVENANCE_REFUSED
        + match r {
            bank::Refusal::RouteUndeclared => 0,
            bank::Refusal::DmrgClaimedExact => 1,
            bank::Refusal::DmrgUnvalidated => 2,
            bank::Refusal::UncertaintyMissing => 3,
            bank::Refusal::DmrgUncertaintyMissing => 4,
            bank::Refusal::SplitViolated => 5,
            bank::Refusal::CurveNotLoaded => 6,
        }
}

/// Load a pre-computed [`holon_chem::pair::PairTable`] into ITS OWN SLOT in the bank.
///
/// The species come from the table's own metadata rather than from a separate argument:
/// a curve knows which pair it is, and a caller passing the species alongside it is a
/// second statement of the same fact that can disagree with the first. Plant (i) is
/// exactly that disagreement — serving the (A,A) curve where (A,B) belongs — and the way
/// to make it a plant rather than an ordinary hazard is to leave only one place it can be
/// introduced.
///
/// `host` decides whether the browser's in-browser/shipped split is enforced; see
/// [`bank::Host`].
pub fn load_pair_table(s: &mut Sim, pt: &holon_chem::pair::PairTable, host: bank::Host) -> u32 {
    let (za, zb) = (pt.meta.z_a, pt.meta.z_b);
    if s.bank.register(za).is_none() || s.bank.register(zb).is_none() {
        return BANK_FULL;
    }
    let Some(slot) = s.bank.slot_of_z(za, zb) else {
        return BANK_FULL;
    };
    let n = pt.r.len();
    if !s.bank.table_slot_mut(slot).begin(n) {
        return status_code(s.bank.table_slot(slot).status);
    }
    for i in 0..n {
        let t = s.bank.table_slot_mut(slot);
        if !t.knot(i, pt.r[i], pt.e[i], pt.f[i]) {
            return status_code(s.bank.table_slot(slot).status);
        }
        if i < pt.e2.len() {
            t.knot_curvature(i, pt.e2[i]);
        }
    }
    let (r_e, d_e) = match pt.meta.well {
        Some(w) => (w.r_e, w.d_e),
        None => (0.0, 0.0),
    };
    let status = s
        .bank
        .table_slot_mut(slot)
        .finish(r_e, d_e, pt.meta.e_asymptote);
    if status == table::LoadStatus::Ok {
        // THE PROVENANCE COMES OFF THE CURVE, not off a constant. `PairMeta::route` is
        // what the solver actually did — including the size threshold in `fci::solve`
        // that switches to DMRG — so a DMRG curve arriving here is labelled DMRG and is
        // graded as one.
        let prov = bank::TableProvenance {
            route: match pt.meta.route {
                holon_chem::fci::SolverRoute::Determinant => bank::Route::Determinant,
                holon_chem::fci::SolverRoute::Dmrg => bank::Route::Dmrg,
            },
            source: bank::Source::Solved,
            n_det: pt.meta.n_det as u64,
            uncertainty_ha: pt.meta.worst_residual,
            claimed_exact: pt.meta.route.is_exact_in_model(),
        };
        if let Err(r) = s.bank.commit(slot, prov, &bank::D1_RECORD, host) {
            return refusal_code(r);
        }
        s.adopt_table_timescale();
    }
    status_code(status)
}

/// The bank has no room for another species. Distinct from every other code for the same
/// reason [`PROVENANCE_REFUSED`] is: a full bank and a bad curve are different problems.
pub const BANK_FULL: u32 = 15;

/// No route in this engine produces this pair's curve. See
/// [`holon_chem::pair::MPS_MAX_ORBITALS`] for the measurement behind it.
///
/// Distinct from `GENERATOR_REFUSED` (which means the grid request was malformed) and from
/// `BANK_FULL`: this one says the chemistry is out of reach, which is a fact about the
/// engine's solvers and not about the caller's arguments.
pub const CURVE_INFEASIBLE: u32 = 14;

/// Solve and load any pair potential table (e.g. LiH, HF, Li2, etc.) dynamically.
///
/// NATIVE host: this is the entry point the desktop shells use, and they have no page
/// load budget. The browser's own entry point is [`holon_bank_generate_pair`].
pub fn generate_pair_table(
    s: &mut Sim,
    a: holon_chem::elements::Species,
    b: holon_chem::elements::Species,
    count: usize,
) -> u32 {
    if holon_chem::pair::feasibility(a, b).is_infeasible() {
        return CURVE_INFEASIBLE;
    }
    let pt = holon_chem::pair::generate_pair_table(a, b, count);
    load_pair_table(s, &pt, bank::Host::Native)
}

// ------------------------------------------------- the three-body surface
//
// SATURATION-1's product. `holon-chem` solves H3 at every node of its own grid, subtracts
// the pair sum, and streams the residue in; the interpolator here differentiates it
// analytically for the forces. Same shape as the pair route above and for the same
// reasons: allocation-free, one implementation, and the source of the numbers is a mode
// rather than a fork in the physics.

/// Compute the three-body table and load it, in one call. Returns 1 on success, 0 if the
/// generator refused.
///
/// The table is a fixed array inside the `Sim`, so this fills storage that already exists
/// rather than allocating; and until it is called the three-body term contributes an
/// EXACT zero, so a host that never calls it gets the pairwise sandbox unchanged.
#[no_mangle]
pub extern "C" fn holon_trimer_generate() -> u32 {
    generate_trimer_table(&mut sim())
}

/// The Rust-side entry point behind [`holon_trimer_generate`], for a caller that owns its
/// own [`Sim`] — the 3D shell in `holon-render-3d` is that caller. Defined once and called
/// by the export, for the same reason [`generate_table`] is.
pub fn generate_trimer_table(s: &mut Sim) -> u32 {
    s.trimer.begin();
    let meta = holon_chem::trimer::stream_trimer_table(|i, _x, _y, _u, v| s.trimer.knot(i, v));
    let Some(meta) = meta else {
        return 0;
    };
    if !s.trimer.finish(meta) {
        return 0;
    }
    // The ledger's origin was frozen without the three-body term in it; adopting a surface
    // changes the potential, so the run has to be re-based or the drift would be measured
    // against the wrong zero and would read a JUMP that is not integration error. Doing it
    // here rather than asking every caller to remember is the same reasoning that puts
    // `adopt_table_timescale` inside `generate_table`.
    s.rebase();
    1
}

/// Is a three-body surface loaded?
#[no_mangle]
pub extern "C" fn holon_trimer_loaded() -> u32 {
    u32::from(sim().trimer.loaded)
}

/// Nodes in the three-body table.
#[no_mangle]
pub extern "C" fn holon_trimer_nodes() -> u32 {
    holon_chem::trimer::N_NODES as u32
}

/// Largest `|dE3|` anywhere on the table's grid, hartree — the compact corner's value.
#[no_mangle]
pub extern "C" fn holon_trimer_peak() -> f64 {
    sim().trimer.meta.peak
}

/// The table's measured second-derivative envelope, hartree/bohr^2. The drift bound's
/// three-body term is built from it.
#[no_mangle]
pub extern "C" fn holon_trimer_curvature_envelope() -> f64 {
    sim().trimer.curvature_envelope
}

/// The truncation radius on the two shortest sides, bohr.
#[no_mangle]
pub extern "C" fn holon_trimer_r_max() -> f64 {
    holon_chem::trimer::R_HI
}

/// `LoadStatus::Ok`'s discriminant, so a caller of [`generate_table`] can name the
/// success code rather than repeating the number.
pub const TABLE_OK: u32 = 1;

/// Status code for "the generator would not produce a curve for this request" — a range
/// or a knot count that is not a grid. Distinct from the table's own refusals so the
/// viewer can say which half declined, and public so the tests and `app.js` name it
/// rather than repeating the number.
pub const GENERATOR_REFUSED: u32 = 6;

/// The worst disagreement, in hartree, between this engine's f64 curve and the pinned
/// 50-digit referee, measured over all 492 of the referee's separations and enforced by
/// `holon-chem`'s `tests/referee.rs` on every build. Baked in as a constant so the
/// viewer's banner states a measured number rather than an adjective.
#[no_mangle]
pub extern "C" fn holon_chem_referee_residual() -> f64 {
    holon_chem::REFEREE_MEASURED_E
}

/// FNV-1a digest of the pinned referee curve the residual above was measured against.
/// Displayed with the residual: a residual without the identity of what it is a residual
/// FROM is not a claim about anything.
#[no_mangle]
pub extern "C" fn holon_chem_referee_digest() -> u32 {
    holon_chem::REFEREE_DIGEST
}

/// Number of separations in that referee curve.
#[no_mangle]
pub extern "C" fn holon_chem_referee_points() -> u32 {
    holon_chem::REFEREE_GRID_POINTS as u32
}

/// The MODEL at `r`, bypassing the table and its interpolant: total energy in hartree.
///
/// The table is a sampled, interpolated view of this, and the two answer different
/// questions. Asking the table how well the browser reproduces the referee measures the
/// GRID (a few times 1e-10); asking this measures the ARITHMETIC (a few times 1e-15),
/// which is what the banner's residual is about. Without this export the browser-side
/// half of the referee gate could not be measured at all — only inferred from the native
/// build, whose libm is a different one.
#[no_mangle]
pub extern "C" fn holon_chem_energy(r: f64) -> f64 {
    holon_chem::h2_energy(r)
}

/// The model's force at `r`, `-dE/dR`, differentiated analytically rather than sampled.
#[no_mangle]
pub extern "C" fn holon_chem_force(r: f64) -> f64 {
    holon_chem::h2_point(r).f
}

/// Returns the `LoadStatus` discriminant: 1 = Ok, anything else is a refusal.
#[no_mangle]
pub extern "C" fn holon_table_finish(r_e: f64, d_e: f64, e_asymptote: f64) -> u32 {
    let mut s = sim();
    let status = s.table_mut().finish(r_e, d_e, e_asymptote);
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
    u32::from(sim().table_mut().knot_curvature(index as usize, d2))
}

#[no_mangle]
pub extern "C" fn holon_table_has_curvature() -> u32 {
    u32::from(sim().table().has_supplied_curvature())
}

/// Worst relative disagreement between a supplied d2 column and the interpolant's own
/// curvature at the knots. Reported, never enforced: cubic Hermite is C1, so its
/// curvature is discontinuous at knots and a mismatch is expected structure.
#[no_mangle]
pub extern "C" fn holon_table_d2_mismatch() -> f64 {
    sim().table().d2_mismatch
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
    status_code(sim().table().status)
}

#[no_mangle]
pub extern "C" fn holon_table_knots() -> u32 {
    sim().table().knots() as u32
}

/// RMS mismatch between the file's derivatives and the secant slopes of its values,
/// under the assumed convention `dE/dR = -F`. Near zero for a consistent table.
#[no_mangle]
pub extern "C" fn holon_table_residual() -> f64 {
    sim().table().residual
}

/// The same statistic under `dE/dR = +F`. Should be LARGE; if it is the smaller of the
/// two, the file uses the opposite sign convention and the viewer says so.
#[no_mangle]
pub extern "C" fn holon_table_residual_alt() -> f64 {
    sim().table().residual_alt
}

#[no_mangle]
pub extern "C" fn holon_table_r_e() -> f64 {
    sim().table().r_e
}

#[no_mangle]
pub extern "C" fn holon_table_d_e() -> f64 {
    sim().table().d_e
}

#[no_mangle]
pub extern "C" fn holon_table_asymptote() -> f64 {
    sim().table().e_asymptote
}

#[no_mangle]
pub extern "C" fn holon_table_r_min() -> f64 {
    sim().table().r_min()
}

#[no_mangle]
pub extern "C" fn holon_table_r_max() -> f64 {
    sim().table().r_max()
}

/// Asymptote-zeroed pair energy at separation `r`. Exposed so the viewer can draw the
/// very curve the integrator is using, rather than a second copy of it in JS.
#[no_mangle]
pub extern "C" fn holon_curve_u(r: f64) -> f64 {
    sim().table().u(r)
}

#[no_mangle]
pub extern "C" fn holon_curve_force(r: f64) -> f64 {
    sim().table().force(r)
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
        // Over ALL active curves, not just the primary: the envelope that comes back has
        // to be the same one `Sim::refresh_envelope` would have produced, or returning to
        // the exactness hold would quietly narrow the bound in a mixed scene.
        s.reseed_envelope(e);
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
pub extern "C" fn holon_depth() -> f64 {
    sim().depth
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

/// The third component. Every 2D host can ignore it: in a [`sim::Dims::Two`] scene it
/// reads `depth / 2` for every atom, forever, because nothing can move an atom off the
/// mid-plane (see `sim.rs`'s header).
#[no_mangle]
pub extern "C" fn holon_atom_z(i: u32) -> f64 {
    let s = sim();
    let i = i as usize;
    if i < s.n {
        s.atoms[i].z
    } else {
        0.0
    }
}

#[no_mangle]
pub extern "C" fn holon_atom_speed(i: u32) -> f64 {
    let s = sim();
    let i = i as usize;
    if i < s.n {
        let a = &s.atoms[i];
        (a.vx * a.vx + a.vy * a.vy + a.vz * a.vz).sqrt()
    } else {
        0.0
    }
}

/// The atomic number (nuclear charge Z) of atom `i`.
#[no_mangle]
pub extern "C" fn holon_atom_species_z(i: u32) -> u32 {
    let s = sim();
    let i = i as usize;
    if i < s.n {
        s.atoms[i].species.z
    } else {
        1
    }
}

/// Set the species of atom `i` by nuclear charge Z.
#[no_mangle]
pub extern "C" fn holon_set_atom_species(i: u32, z: u32) -> u32 {
    let Some(sp) = holon_chem::elements::by_z(z) else {
        return 0;
    };
    u32::from(sim().set_species(i as usize, sp))
}

#[no_mangle]
pub extern "C" fn holon_set_velocity(i: u32, vx: f64, vy: f64) {
    sim().set_velocity(i as usize, vx, vy);
}

#[no_mangle]
pub extern "C" fn holon_set_velocity_3d(i: u32, vx: f64, vy: f64, vz: f64) {
    sim().set_velocity_3d(i as usize, vx, vy, vz);
}

#[no_mangle]
pub extern "C" fn holon_set_position(i: u32, x: f64, y: f64) {
    sim().set_position(i as usize, x, y);
}

#[no_mangle]
pub extern "C" fn holon_set_position_3d(i: u32, x: f64, y: f64, z: f64) {
    sim().set_position_3d(i as usize, x, y, z);
}

// ------------------------------------------------------------------ dimensionality
//
// A MODE, not a second physics. The integrator carries three components either way;
// this says how many the scene moves in, and it is read by exactly two things (the
// equipartition denominator and the opening scene). See `sim.rs`'s header.

/// 0 = the mid-plane (2D, the default and what the canvas shell draws), 1 = the full
/// box. Takes effect at the next `holon_reset`, which is what places the atoms.
#[no_mangle]
pub extern "C" fn holon_set_dims(three: u32) {
    let mut s = sim();
    s.dims = if three == 0 {
        sim::Dims::Two
    } else {
        sim::Dims::Three
    };
}

#[no_mangle]
pub extern "C" fn holon_dims() -> u32 {
    match sim().dims {
        sim::Dims::Two => 2,
        sim::Dims::Three => 3,
    }
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

/// Number of CLUSTERS: connected components (size >= 2) of the bonded-pair graph.
/// The headline object — `holon_bonded_count` counts edges of this graph, and on a
/// collapsed droplet the edge count is C(n,2) while the cluster count is 1. See
/// [`Sim::cluster_count`] for why both are true and only one belongs in a headline.
#[no_mangle]
pub extern "C" fn holon_cluster_count() -> u32 {
    sim().cluster_count().0 as u32
}

/// Number of atoms that belong to some cluster (the rest are free atoms).
#[no_mangle]
pub extern "C" fn holon_cluster_atoms() -> u32 {
    sim().cluster_count().1 as u32
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

/// The many-body sector. Its own reader, never folded into `E_pair`: a combined number
/// could not say which sector moved.
#[no_mangle]
pub extern "C" fn holon_e_three() -> f64 {
    sim().e_three
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
pub extern "C" fn holon_momentum_z() -> f64 {
    sim().momentum().2
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
pub extern "C" fn holon_move_anchor_3d(x: f64, y: f64, z: f64) {
    sim().move_anchor_3d(x, y, z);
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

#[no_mangle]
pub extern "C" fn holon_anchor_z() -> f64 {
    sim().anchor.2
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

// ================================================================== the pair-table bank
//
// MIXTURES-1's engine product, exposed. Everything above that reads "the table" is the
// SINGLE-CURVE view — correct for a pure scene and a display convenience in a mixed one.
// These read the bank itself: which species the scene holds, which curve serves which
// pair, and what each curve says about where it came from.
//
// The host displays the strings (producer, grid rule); the engine holds the parts a gate
// acts on. See `bank.rs` for why the split falls there.

/// How many distinct species the bank can hold before it REFUSES a new one.
#[no_mangle]
pub extern "C" fn holon_bank_max_species() -> u32 {
    bank::MAX_SPECIES as u32
}

/// How many slots the bank has, which is the unordered pairs over `max_species`.
#[no_mangle]
pub extern "C" fn holon_bank_slot_count() -> u32 {
    bank::MAX_TABLES as u32
}

/// Determinant count at or above which a pair must arrive as a shipped table rather than
/// being solved at page load. Returned as `f64` because the count can exceed `u32`.
#[no_mangle]
pub extern "C" fn holon_bank_in_browser_det_limit() -> f64 {
    bank::IN_BROWSER_DET_LIMIT as f64
}

#[no_mangle]
pub extern "C" fn holon_bank_species_count() -> u32 {
    sim().bank.species_count() as u32
}

#[no_mangle]
pub extern "C" fn holon_bank_species_z(i: u32) -> u32 {
    sim().bank.species_z(i as usize)
}

/// Register a species with the bank. `1` on success, `0` if the bank is full or `z` is not
/// an element this engine knows.
#[no_mangle]
pub extern "C" fn holon_bank_register(z: u32) -> u32 {
    if holon_chem::elements::by_z(z).is_none() {
        return 0;
    }
    u32::from(sim().bank.register(z).is_some())
}

/// Forget every species and every curve. The scene must be rebuilt afterwards.
#[no_mangle]
pub extern "C" fn holon_bank_clear() {
    sim().bank.clear();
}

/// The slot serving the pair `(za, zb)`, or `-1` if either species is unregistered.
#[no_mangle]
pub extern "C" fn holon_bank_slot(za: u32, zb: u32) -> i32 {
    sim()
        .bank
        .slot_of_z(za, zb)
        .map_or(-1, |s| s as i32)
}

#[no_mangle]
pub extern "C" fn holon_bank_filled(slot: u32) -> u32 {
    let s = sim();
    let slot = slot as usize;
    u32::from(slot < bank::MAX_TABLES && s.bank.is_filled(slot))
}

#[no_mangle]
pub extern "C" fn holon_bank_filled_count() -> u32 {
    sim().bank.filled_count() as u32
}

/// Whether every pair the scene contains has a curve to be evaluated on. The bank's
/// version of `holon_table_status() == 1`, and what `holon_step_frame` now gates on.
#[no_mangle]
pub extern "C" fn holon_pairs_ready() -> u32 {
    u32::from(sim().pairs_ready())
}

/// Solve a pair's curve IN THE BROWSER and bank it.
///
/// Refuses a pair past `holon_bank_in_browser_det_limit` — that is the split, and it is
/// enforced here rather than left to the host to remember. Returns `LoadStatus` (1 = Ok),
/// `BANK_FULL`, `GENERATOR_REFUSED`, or a `PROVENANCE_REFUSED` code.
#[no_mangle]
pub extern "C" fn holon_bank_generate_pair(za: u32, zb: u32, knots: u32) -> u32 {
    let (Some(a), Some(b)) = (
        holon_chem::elements::by_z(za),
        holon_chem::elements::by_z(zb),
    ) else {
        return GENERATOR_REFUSED;
    };
    // Ask before spending. `generate_pair_table` refuses an infeasible pair by panicking,
    // which in a browser is a trap with no message a user can act on; the host gets a code
    // instead, and the ABI never reaches the assert.
    if holon_chem::pair::feasibility(a, b).is_infeasible() {
        return CURVE_INFEASIBLE;
    }
    let pt = holon_chem::pair::generate_pair_table(a, b, knots as usize);
    load_pair_table(&mut sim(), &pt, bank::Host::Browser)
}

/// Which route a pair's curve would take, without computing it: `0` infeasible,
/// `1` determinant/FCI, `2` MPS/DMRG.
///
/// Lets a viewer grey out a species pair it cannot have rather than offering it and
/// hanging. The determinant count behind the answer is `holon_bank_pair_n_det`.
#[no_mangle]
pub extern "C" fn holon_bank_pair_route(za: u32, zb: u32) -> u32 {
    let (Some(a), Some(b)) = (
        holon_chem::elements::by_z(za),
        holon_chem::elements::by_z(zb),
    ) else {
        return 0;
    };
    match holon_chem::pair::feasibility(a, b) {
        holon_chem::pair::Feasibility::Infeasible { .. } => 0,
        holon_chem::pair::Feasibility::Determinant { .. } => 1,
        holon_chem::pair::Feasibility::Mps { .. } => 2,
    }
}

/// The determinant count a pair's solve would face. `f64` because it can exceed `u32` —
/// Na2 is 1.0e9.
#[no_mangle]
pub extern "C" fn holon_bank_pair_n_det(za: u32, zb: u32) -> f64 {
    let (Some(a), Some(b)) = (
        holon_chem::elements::by_z(za),
        holon_chem::elements::by_z(zb),
    ) else {
        return 0.0;
    };
    holon_chem::pair::feasibility(a, b).n_det() as f64
}

// ---- pushing a SHIPPED table into a slot -------------------------------------------
//
// Same three-call shape as the legacy `holon_table_begin/knot/finish`, with the slot named
// and the provenance mandatory. There is no way to push a shipped curve without saying
// what produced it: `finish` takes the route, the determinant count and the uncertainty,
// and refuses the lot if they do not add up.

#[no_mangle]
pub extern "C" fn holon_bank_table_begin(slot: u32, count: u32) -> u32 {
    let mut s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0;
    }
    u32::from(s.bank.table_slot_mut(slot).begin(count as usize))
}

#[no_mangle]
pub extern "C" fn holon_bank_table_knot(slot: u32, index: u32, r: f64, e: f64, f: f64) -> u32 {
    let mut s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0;
    }
    u32::from(s.bank.table_slot_mut(slot).knot(index as usize, r, e, f))
}

#[no_mangle]
pub extern "C" fn holon_bank_table_knot_curvature(slot: u32, index: u32, d2: f64) -> u32 {
    let mut s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0;
    }
    u32::from(
        s.bank
            .table_slot_mut(slot)
            .knot_curvature(index as usize, d2),
    )
}

/// Finish a shipped table AND declare its provenance, in one call.
///
/// `route`: 1 = determinant/FCI, 2 = DMRG. Anything else is `Route::Undeclared` and is
/// refused — which is the point: a host that does not know what it is loading cannot load
/// it.
///
/// `claimed_exact` is what the FILE says about itself, kept apart from what `route`
/// implies. A DMRG table arriving with `claimed_exact = 1` is the plant (iii) defect, and
/// the refusal it earns is `PROVENANCE_REFUSED + 1`.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn holon_bank_table_finish(
    slot: u32,
    r_e: f64,
    d_e: f64,
    e_asymptote: f64,
    route: u32,
    n_det: f64,
    uncertainty_ha: f64,
    claimed_exact: u32,
) -> u32 {
    let mut s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0;
    }
    let status = s
        .bank
        .table_slot_mut(slot)
        .finish(r_e, d_e, e_asymptote);
    if status != table::LoadStatus::Ok {
        return status_code(status);
    }
    let prov = bank::TableProvenance {
        route: match route {
            1 => bank::Route::Determinant,
            2 => bank::Route::Dmrg,
            _ => bank::Route::Undeclared,
        },
        source: bank::Source::Shipped,
        n_det: if n_det.is_finite() && n_det >= 0.0 {
            n_det as u64
        } else {
            0
        },
        uncertainty_ha,
        claimed_exact: claimed_exact != 0,
    };
    if let Err(r) = s.bank.commit(slot, prov, &bank::D1_RECORD, bank::Host::Browser) {
        return refusal_code(r);
    }
    s.adopt_table_timescale();
    status_code(status)
}

// ---- per-slot readouts ---------------------------------------------------------------

macro_rules! slot_scalar {
    ($name:ident, $field:ident) => {
        #[no_mangle]
        pub extern "C" fn $name(slot: u32) -> f64 {
            let s = sim();
            let slot = slot as usize;
            if slot >= bank::MAX_TABLES {
                return 0.0;
            }
            s.bank.table_slot(slot).$field
        }
    };
}

slot_scalar!(holon_bank_r_e, r_e);
slot_scalar!(holon_bank_d_e, d_e);
slot_scalar!(holon_bank_asymptote, e_asymptote);
slot_scalar!(holon_bank_residual, residual);
slot_scalar!(holon_bank_residual_alt, residual_alt);

#[no_mangle]
pub extern "C" fn holon_bank_knots(slot: u32) -> u32 {
    let s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0;
    }
    s.bank.table_slot(slot).knots() as u32
}

#[no_mangle]
pub extern "C" fn holon_bank_r_min(slot: u32) -> f64 {
    let s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0.0;
    }
    s.bank.table_slot(slot).r_min()
}

#[no_mangle]
pub extern "C" fn holon_bank_r_max(slot: u32) -> f64 {
    let s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0.0;
    }
    s.bank.table_slot(slot).r_max()
}

/// The slot's curve at `r`, asymptote-zeroed. What the viewer plots per pair.
#[no_mangle]
pub extern "C" fn holon_bank_u(slot: u32, r: f64) -> f64 {
    let s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0.0;
    }
    s.bank.table_slot(slot).u(r)
}

#[no_mangle]
pub extern "C" fn holon_bank_force(slot: u32, r: f64) -> f64 {
    let s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0.0;
    }
    s.bank.table_slot(slot).force(r)
}

// ---- per-slot provenance -------------------------------------------------------------

/// `0` undeclared, `1` determinant/FCI, `2` DMRG.
#[no_mangle]
pub extern "C" fn holon_bank_route(slot: u32) -> u32 {
    let s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0;
    }
    match s.bank.provenance_slot(slot).route {
        bank::Route::Undeclared => 0,
        bank::Route::Determinant => 1,
        bank::Route::Dmrg => 2,
    }
}

/// `0` solved by this process, `1` loaded from a shipped table.
#[no_mangle]
pub extern "C" fn holon_bank_source(slot: u32) -> u32 {
    let s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0;
    }
    match s.bank.provenance_slot(slot).source {
        bank::Source::Solved => 0,
        bank::Source::Shipped => 1,
    }
}

#[no_mangle]
pub extern "C" fn holon_bank_n_det(slot: u32) -> f64 {
    let s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0.0;
    }
    s.bank.provenance_slot(slot).n_det as f64
}

#[no_mangle]
pub extern "C" fn holon_bank_uncertainty(slot: u32) -> f64 {
    let s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0.0;
    }
    s.bank.provenance_slot(slot).uncertainty_ha
}

#[no_mangle]
pub extern "C" fn holon_bank_claimed_exact(slot: u32) -> u32 {
    let s = sim();
    let slot = slot as usize;
    if slot >= bank::MAX_TABLES {
        return 0;
    }
    u32::from(s.bank.provenance_slot(slot).claimed_exact)
}

/// Whether every loaded curve's provenance was admitted by the gate.
#[no_mangle]
pub extern "C" fn holon_bank_provenance_ok() -> u32 {
    u32::from(sim().provenance_ok(bank::Host::Browser))
}

/// The first refused slot, or `-1`.
#[no_mangle]
pub extern "C" fn holon_bank_refusal_slot() -> i32 {
    sim()
        .provenance_refusal(bank::Host::Browser)
        .map_or(-1, |(s, _)| s as i32)
}

/// The first refusal's reason as a `PROVENANCE_REFUSED` code, or `0` if there is none.
#[no_mangle]
pub extern "C" fn holon_bank_refusal_reason() -> u32 {
    sim()
        .provenance_refusal(bank::Host::Browser)
        .map_or(0, |(_, r)| refusal_code(r))
}

// ---- gate D1's record ----------------------------------------------------------------
//
// Read by both viewers so the DMRG bridge's admission is on the page rather than in a
// results file. While `validated` is 0, every DMRG curve is refused and the viewer says so.

#[no_mangle]
pub extern "C" fn holon_d1_validated() -> u32 {
    u32::from(bank::D1_RECORD.admits())
}

#[no_mangle]
pub extern "C" fn holon_d1_worst_overlap() -> f64 {
    bank::D1_RECORD.worst_overlap_ha
}

#[no_mangle]
pub extern "C" fn holon_d1_stake() -> f64 {
    bank::D1_RECORD.stake_ha
}

#[no_mangle]
pub extern "C" fn holon_d1_overlap_species() -> u32 {
    bank::D1_RECORD.overlap_species as u32
}

// ---- the three-body fence, DECLARED by the engine ------------------------------------

/// `1` while the tabulated three-body term covers H3 ONLY.
///
/// The force loop already skips any triple containing a non-hydrogen atom. This makes the
/// fact readable, so both viewers can DISPLAY the fence rather than each hardcoding a
/// sentence that would go stale the day a heteronuclear trimer surface lands. MIXTURES-1
/// requires the fence to be shown; a viewer asserting it independently of the engine is a
/// caption, not a fence.
#[no_mangle]
pub extern "C" fn holon_trimer_h_only() -> u32 {
    1
}

/// Which bank slot pair reading `k` was evaluated on, or `-1`.
///
/// Lets the viewer name the curve behind each pair row — the reason a mixed scene's two
/// rows can honestly report different `bonded` verdicts at the same separation.
#[no_mangle]
pub extern "C" fn holon_pair_slot(k: u32) -> i32 {
    let s = sim();
    let k = k as usize;
    if k >= s.pair_count {
        return -1;
    }
    let p = s.pairs[k];
    let slots = s.species_slots();
    s.bank.slot(slots[p.i], slots[p.j]) as i32
}
