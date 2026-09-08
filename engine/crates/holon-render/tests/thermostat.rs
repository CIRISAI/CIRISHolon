//! THE THERMOSTAT COMPARISON, and the Berendsen guarantee.
//!
//! The second review's fifth source claim was that this engine's thermostat is Berendsen
//! velocity rescaling and that suppressed fluctuations are not canonical sampling. Verified
//! and answered: `crate::thermostat` adds stochastic velocity rescaling (Bussi, Donadio and
//! Parrinello, *J. Chem. Phys.* **126** (2007) 014101) as a SELECTABLE alternative, wearing
//! the same ledger column.
//!
//! Three things are measured here.
//!
//! 1. **On a harmonic test system** — `N` three-dimensional oscillators with incommensurate
//!    frequencies, integrated by velocity–Verlet in this file, initialised from the
//!    canonical distribution — the kinetic energy's relative variance is the canonical
//!    `2/N_f = 2/(3N)` under stochastic rescaling and MEASURABLY SMALLER under Berendsen.
//!    Both are printed; the test asserts the canonical one on its own value and asserts of
//!    the other only that it is suppressed, which is what the claim is.
//! 2. **The engine selects Berendsen by default and draws nothing under it.** Two Berendsen
//!    runs of the same scene are bit-for-bit identical and leave `thermostat_draws()` at
//!    exactly `0`; the stochastic run draws and moves.
//! 3. **The ledger is one ledger.** `work_columns_ok` and the posted thermostat column hold
//!    under both rules, because both are the same rescaling with a different `λ`.
//!
//! Why a harmonic system for (1): the canonical kinetic-energy distribution
//! `K ~ Gamma(N_f/2, k_B T)` is a statement about the momenta alone and holds for ANY
//! potential, so the cheapest exactly-integrable one is the honest choice — nothing about
//! the answer depends on the force law, and the test therefore measures the thermostat and
//! not the water.

use holon_chem::elements::HYDROGEN;
use holon_chem::pair::{generate_pair_table, PairTable};
use holon_render::bank::Host;
use holon_render::sim::{Boundary, Dims, Sim, K_B};
use holon_render::thermostat::{bdp_lambda, berendsen_lambda, ThermostatKind, ThermostatRng};
use holon_render::{load_pair_table, TABLE_OK};
use std::sync::OnceLock;

// ------------------------------------------------------------ the harmonic test system

/// `n` particles in three dimensions, each in an isotropic harmonic well of its own
/// frequency, thermostatted every step by a factor the caller supplies.
///
/// Returns `(mean K, Var(K)/<K>^2)` over the sampled steps.
fn harmonic_run(
    n: usize,
    steps: usize,
    burn: usize,
    kt: f64,
    dt: f64,
    mut lambda: impl FnMut(f64, f64, usize) -> f64,
) -> (f64, f64) {
    let n_dof = 3 * n;
    let k_target = 0.5 * n_dof as f64 * kt;
    // Frequencies spread over an incommensurate range so no two oscillators share a phase.
    let w: Vec<f64> = (0..n).map(|i| 0.8 + 0.5 * (i as f64 + 0.5) / n as f64).collect();
    // CANONICAL initial conditions: x ~ N(0, sqrt(kT)/w), v ~ N(0, sqrt(kT)) at unit mass.
    let mut rng = ThermostatRng::new(0x48_41_52_4d_4f_4e_49_43);
    let mut x = vec![[0.0f64; 3]; n];
    let mut v = vec![[0.0f64; 3]; n];
    for i in 0..n {
        for c in 0..3 {
            x[i][c] = kt.sqrt() / w[i] * rng.gauss();
            v[i][c] = kt.sqrt() * rng.gauss();
        }
    }
    let kinetic = |v: &[[f64; 3]]| -> f64 {
        0.5 * v.iter().map(|u| u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sum::<f64>()
    };
    let mut sample: Vec<f64> = Vec::with_capacity(steps.saturating_sub(burn));
    for s in 0..steps {
        // velocity-Verlet on a = -w^2 x
        for i in 0..n {
            let a = -w[i] * w[i];
            for c in 0..3 {
                v[i][c] += 0.5 * dt * a * x[i][c];
                x[i][c] += dt * v[i][c];
                v[i][c] += 0.5 * dt * a * x[i][c];
            }
        }
        let k_now = kinetic(&v);
        let l = lambda(k_now, k_target, n_dof);
        if l.is_finite() && l > 0.0 {
            for u in v.iter_mut() {
                for c in 0..3 {
                    u[c] *= l;
                }
            }
        }
        if s >= burn {
            sample.push(kinetic(&v));
        }
    }
    let m = sample.iter().sum::<f64>() / sample.len() as f64;
    let var = sample.iter().map(|k| (k - m) * (k - m)).sum::<f64>() / (sample.len() - 1) as f64;
    (m, var / (m * m))
}

/// THE COMPARISON. `Var(K)/<K>^2` must be the canonical `2/(3N)` under stochastic
/// rescaling and measurably below it under Berendsen. Both are printed.
#[test]
fn stochastic_rescaling_is_canonical_where_berendsen_is_suppressed() {
    let (n, steps, burn, kt, dt) = (32usize, 600_000usize, 50_000usize, 1.0f64, 0.02f64);
    let n_dof = 3 * n;
    let canonical = 2.0 / n_dof as f64;
    let dt_over_tau = 0.05f64;

    let mut rng = ThermostatRng::new(0xBD_50_32_07);
    let (k_sto, rel_sto) = harmonic_run(n, steps, burn, kt, dt, |k, kt_target, dof| {
        bdp_lambda(k, kt_target, dof, dt_over_tau, &mut rng).unwrap_or(1.0)
    });
    let (k_ber, rel_ber) = harmonic_run(n, steps, burn, kt, dt, |k, kt_target, dof| {
        // Berendsen speaks in temperatures; on this system T is K/(dof/2 k_B) with the
        // same dof, so the two rules are being asked for the same target.
        let t_now = k / (0.5 * dof as f64);
        let t_target = kt_target / (0.5 * dof as f64);
        berendsen_lambda(t_now, t_target, dt_over_tau).unwrap_or(1.0)
    });

    let k_target = 0.5 * n_dof as f64 * kt;
    println!(
        "harmonic test system, N = {n} ({n_dof} dof), kT = {kt}, dt/tau = {dt_over_tau}\n  \
         canonical relative variance 2/N_f            = {canonical:.6e}\n  \
         stochastic rescaling (Bussi et al. 2007)      = {rel_sto:.6e}  ({:.3} x canonical), <K> = {k_sto:.6} against {k_target:.6}\n  \
         Berendsen (Berendsen et al. 1984)             = {rel_ber:.6e}  ({:.3} x canonical), <K> = {k_ber:.6} against {k_target:.6}",
        rel_sto / canonical,
        rel_ber / canonical
    );

    // both must reach the right MEAN — that is what Berendsen is good at and it is not the
    // thing in question
    assert!((k_sto - k_target).abs() / k_target < 0.02, "stochastic mean {k_sto}");
    assert!((k_ber - k_target).abs() / k_target < 0.02, "Berendsen mean {k_ber}");

    // the CANONICAL claim, on its own value
    assert!(
        (rel_sto - canonical).abs() / canonical < 0.15,
        "stochastic rescaling must read the canonical 2/N_f = {canonical:.6e}, read {rel_sto:.6e}"
    );
    // and the suppression, which is the review's point
    assert!(
        rel_ber < 0.5 * canonical,
        "Berendsen's kinetic fluctuation must be measurably suppressed: {rel_ber:.6e} against \
         the canonical {canonical:.6e}"
    );
}

// ------------------------------------------------------- the engine's own selection

/// Knots per fixture curve — small for the reason `saturation2.rs` gives: the pair
/// interpolant's accuracy is another campaign's gate and every knot is a full solve.
const FIXTURE_KNOTS: usize = 24;

fn hh_table() -> &'static PairTable {
    static B: OnceLock<PairTable> = OnceLock::new();
    B.get_or_init(|| generate_pair_table(HYDROGEN, HYDROGEN, FIXTURE_KNOTS))
}

/// A small three-dimensional scene with the thermostat engaged: hydrogen on a ring in a
/// walled box, the same shape `saturation2.rs`'s thermostatted fixture uses.
fn thermostatted_scene() -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    assert_eq!(load_pair_table(&mut s, hh_table(), Host::Native), TABLE_OK);
    s.boundary = Boundary::Walls;
    s.dims = Dims::Three;
    s.reset(8);
    let (cx, cy, cz) = (0.5 * s.width, 0.5 * s.height, 0.5 * s.depth);
    for i in 0..8 {
        let th = i as f64 * core::f64::consts::TAU / 8.0;
        s.set_position_3d(i, cx + 8.0 * th.cos(), cy + 8.0 * th.sin(), cz + 0.4 * i as f64);
        let m = s.atoms[i].mass();
        let v = (K_B * 900.0 / m).sqrt();
        s.set_velocity_3d(i, -v * th.sin(), v * th.cos(), 0.0);
    }
    s.rebase();
    s.thermostat_on = true;
    s.target_temperature = 600.0;
    s.thermostat_tau = 2000.0;
    s
}

fn fingerprint(s: &Sim) -> Vec<u64> {
    let mut f = Vec::with_capacity(6 * s.n + 3);
    for i in 0..s.n {
        for x in [
            s.atoms[i].x,
            s.atoms[i].y,
            s.atoms[i].z,
            s.atoms[i].vx,
            s.atoms[i].vy,
            s.atoms[i].vz,
        ] {
            f.push(x.to_bits());
        }
    }
    f.push(s.e_kin.to_bits());
    f.push(s.w_ext.to_bits());
    f.push(s.work.thermostat.to_bits());
    f
}

/// **EVERY EXISTING RECORD STILL SELECTS BERENDSEN, BIT FOR BIT.** The default is
/// Berendsen; a run under the default and a run that selects Berendsen explicitly are the
/// same trajectory to the bit; and neither draws a single deviate, so the stochastic branch
/// cannot have perturbed them.
#[test]
fn berendsen_is_the_default_and_it_draws_nothing() {
    let mut a = thermostatted_scene();
    assert_eq!(a.thermostat_kind, ThermostatKind::Berendsen, "the default is Berendsen");
    for _ in 0..200 {
        a.step_frame(1);
    }
    assert_eq!(a.thermostat_draws(), 0, "a Berendsen run must draw nothing");

    let mut b = thermostatted_scene();
    b.set_thermostat_kind(ThermostatKind::Berendsen, 12345);
    for _ in 0..200 {
        b.step_frame(1);
    }
    assert_eq!(
        fingerprint(&a),
        fingerprint(&b),
        "selecting Berendsen explicitly must be the default trajectory to the bit"
    );
    assert_eq!(b.thermostat_draws(), 0, "selecting it with a seed must still draw nothing");
}

/// The stochastic thermostat MOVES the trajectory and is reproducible from its seed.
#[test]
fn stochastic_rescaling_moves_the_trajectory_and_repeats_from_its_seed() {
    let mut a = thermostatted_scene();
    a.set_thermostat_kind(ThermostatKind::StochasticRescaling, 0xABCDEF);
    for _ in 0..200 {
        a.step_frame(1);
    }
    assert!(a.thermostat_draws() > 0, "the stochastic rule must draw");

    let mut b = thermostatted_scene();
    b.set_thermostat_kind(ThermostatKind::StochasticRescaling, 0xABCDEF);
    for _ in 0..200 {
        b.step_frame(1);
    }
    assert_eq!(fingerprint(&a), fingerprint(&b), "the same seed must give the same trajectory");

    let mut c = thermostatted_scene();
    c.set_thermostat_kind(ThermostatKind::StochasticRescaling, 0x123456);
    for _ in 0..200 {
        c.step_frame(1);
    }
    assert_ne!(fingerprint(&a), fingerprint(&c), "a different seed must give a different one");

    let mut d = thermostatted_scene();
    for _ in 0..200 {
        d.step_frame(1);
    }
    assert_ne!(
        fingerprint(&a),
        fingerprint(&d),
        "and the stochastic rule must differ from Berendsen, or it is not doing anything"
    );
}

/// THE LEDGER IS ONE LEDGER. Both rules post their kinetic change to `w_ext` and to the
/// thermostat column, and the receipt closes under both — the column semantics are
/// unchanged, which is what lets a record be read without knowing which rule ran.
#[test]
fn both_thermostats_wear_the_same_ledger_column() {
    for kind in [ThermostatKind::Berendsen, ThermostatKind::StochasticRescaling] {
        let mut s = thermostatted_scene();
        s.set_thermostat_kind(kind, 0x5EED);
        for _ in 0..400 {
            s.step_frame(1);
        }
        assert!(
            s.work_columns_ok(),
            "{}: the receipt columns must sum to w_ext (residual {:.3e} against {:.3e})",
            kind.name(),
            (s.w_ext - s.work.total()).abs(),
            s.work_columns_bound()
        );
        assert!(
            s.work.thermostat != 0.0,
            "{}: the thermostat must have posted work",
            kind.name()
        );
        println!(
            "{:>22}: w_ext {:.6e}, thermostat column {:.6e}, draws {}, T {:.1} K",
            kind.name(),
            s.w_ext,
            s.work.thermostat,
            s.thermostat_draws(),
            s.temperature()
        );
    }
}
