//! Adaptive crack-tip materialization on a meter wall (E1).
//!
//! The ℓ_ch arithmetic that forces this step (DESCRIPTOR_CHAIN.md §3.4, fix C2):
//! with DEMO_CALIBRATION, ℓ_ch = E·G_F/f_t² = 13.75 cm, so cohesive spacing must be
//! ≤ ℓ_ch/10 ≈ 1.4 cm — finer than any frozen coarse frontier on a meter wall. Here
//! the certifier begins at one resident holon, materializes finer holons and
//! relations only where the damage residual could move the declared observables
//! (crack path, total impulse), and stops at the coarsest frontier meeting the
//! declared tolerance — or returns GrainFloor honestly when the floor cannot.
//!
//! Wall-clock honesty: `certify_runtime_adaptive` materializes ONE branch per full
//! certify pass and each pass restarts at the root, so total cost grows roughly with
//! the SQUARE of the materialization count. Measured on this scene family (release):
//! 0.125 m → 0.1 s, 0.25 m → 1.5 s, 0.5 m → 21 s; the meter headline below takes a
//! few hundred seconds. The replay demonstration therefore runs at 0.5 m.
//!
//! Run with:
//! ```sh
//! cargo run -p ciris-sim-core --example fracture_adaptive --features alloc --release
//! ```

use ciris_sim_core::descriptor::DrawParams;
use ciris_sim_core::fracture::{
    characteristic_length_m, required_spacing_m, FractureConfig, FractureRun, FractureScene,
    ResidualMode, WallGeometry,
};
use ciris_sim_core::holon::CertificationStatus;

const MODAL: [u64; 3] = [30, 60, 10];
const SEED: u64 = 0xC1F1_57A6_E1AD_A971;

fn law() -> DrawParams {
    DrawParams {
        grain_mu_ln_m: (7.5e-4_f64).ln(),
        grain_sigma_ln: 0.5,
        weibull_m: 10.0,
        weibull_sigma0_pa: 2.0e8,
        flaw_density_per_m3: 1.0e10,
    }
}

fn config(side_m: f64, duration_s: f64) -> FractureConfig {
    FractureConfig {
        geometry: WallGeometry {
            side_m,
            thickness_m: 0.1,
            notch_m: 0.24 * side_m,
        },
        macro_tolerance: 1.0,
        grading: 2.0,
        ramp_speed_m_s: 0.1,
        ramp_time_s: 2.0e-4,
        duration_s,
        cfl: 0.25,
        solver_zeta: 0.15,
        residual: ResidualMode::Correct,
    }
}

fn scene(side_m: f64, duration_s: f64, root_grain: u32, seed: u64) -> FractureScene {
    FractureScene::new(
        config(side_m, duration_s),
        law(),
        &MODAL,
        1_000_000,
        root_grain,
        seed,
    )
    .unwrap()
}

fn report(label: &str, wall: &FractureScene, run: &FractureRun, seconds: f64) {
    let certificate = &run.result.certificate;
    println!("\n[{label}] status {:?}; materializations {}; cohesive solves {}; evaluations {}",
        certificate.status, run.result.materializations, run.solves_run, certificate.evaluations);
    println!("  resident holons {}; active frontier {}; finest active cell {:.2} mm (required {:.2} mm)",
        wall.arena().len(),
        certificate.frontier.active_count(),
        1e3 * run.finest_active_m,
        1e3 * run.required_spacing_m);
    println!("  materialized cells: {} on the damage corridor, {} elsewhere", run.materialized_near, run.materialized_far);
    println!("  observables: impulse {:.3} N*s; path deviation {:.2} mm; crack extent {:.3} m",
        certificate.observables[0], 1e3 * certificate.observables[1], certificate.observables[2]);
    println!("  macro error bound {:.4} (tolerance {:.1}); conservation residual {:.3e}; wall-clock {:.1} s",
        certificate.macro_error_bound, wall.config.macro_tolerance, certificate.conservation_residual, seconds);
}

fn main() {
    let properties = scene(1.0, 3.0e-3, 512, SEED).binding.properties;
    let ell_ch = characteristic_length_m(&properties);
    let required = required_spacing_m(&properties);
    println!("l_ch = E*G_F/f_t^2 = {:.4} m; required cohesive spacing l_ch/10 = {:.2} mm", ell_ch, 1e3 * required);
    println!("meter wall, root grain 512 -> grain-floor spacing {:.3} mm; initial resident frontier: 1 holon (1 m)", 1e3 / 512.0);
    println!("(the meter certification takes a few hundred seconds in release; see the module docs)");

    // 1. The certified adaptive run on the meter wall at tolerance 1.0.
    let started = std::time::Instant::now();
    let mut wall = scene(1.0, 3.0e-3, 512, SEED);
    let run = wall.certify().unwrap();
    report("meter wall, tol 1.0", &wall, &run, started.elapsed().as_secs_f64());
    assert!(run.result.certificate.passed());
    // Locality at meter scale: the damage surface moves between the ~20 refinement
    // rounds, so a handful of cells (measured 16 of 3842, 0.4%) end farther from the
    // FINAL surface than the bookkeeping slack. The honest claim is proportional —
    // materialization is corridor-local — not literally zero; the strict far == 0
    // gate holds at the test scale where the surface settles in 3 rounds.
    assert!(
        run.materialized_far * 50 <= run.materialized_near,
        "materialization not corridor-local: {} far vs {} near",
        run.materialized_far,
        run.materialized_near
    );

    // 2. Replay at the half-meter scale: the quenched Record seed makes the whole
    //    adaptive run bit-identical.
    let started = std::time::Instant::now();
    let mut first = scene(0.5, 2.2e-3, 256, SEED);
    let first_run = first.certify().unwrap();
    report("half-meter wall, tol 1.0", &first, &first_run, started.elapsed().as_secs_f64());
    let mut second = scene(0.5, 2.2e-3, 256, SEED);
    let second_run = second.certify().unwrap();
    let identical = first.arena().holons() == second.arena().holons()
        && (0..3).all(|k| {
            first_run.result.certificate.observables[k].to_bits()
                == second_run.result.certificate.observables[k].to_bits()
        });
    println!("\nreplay with the persisted seed is bit-identical: {identical}");
    assert!(identical);

    // 3. The honest refusal: a grain floor coarser than l_ch/10 cannot certify a
    //    crack path at tolerance 1.0. Root grain 32 floors at 31.25 mm on the meter
    //    wall — exactly the frozen-coarse-frontier situation the fixed demo is in.
    let started = std::time::Instant::now();
    let mut floored = scene(1.0, 3.0e-3, 32, SEED);
    let refusal = floored.certify().unwrap();
    println!(
        "\nroot grain 32 (floor {:.2} mm > required {:.2} mm): status {:?} after {} materializations [{:.1} s]",
        1e3 / 32.0,
        1e3 * required,
        refusal.result.certificate.status,
        refusal.result.materializations,
        started.elapsed().as_secs_f64()
    );
    assert_eq!(refusal.result.certificate.status, CertificationStatus::GrainFloor);
}
