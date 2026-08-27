//! N-e — the composed fracture pipeline on the Lac du Bonnet specimen record:
//! descriptor-built wall, derived (k_n, k_t) laws, a thrown projectile, adaptive
//! crack-tip materialization, and fragment contact — certified end to end.
//!
//! The prize gate made visible: `CohesiveLaw::from_continuum` and the
//! homogenization constructor REFUSE this specimen at coarse spacing, and the
//! adaptive certifier earns the right to run it by refining the impact corridor
//! inside the constructor's validity domain — or returns GrainFloor honestly with
//! required-vs-available spacing on the record.
//!
//! Run with:
//! ```sh
//! cargo run -p ciris-sim-core --example impact_composed --features alloc --release
//! ```

use ciris_sim_core::descriptor::DrawParams;
use ciris_sim_core::fracture::{characteristic_length_m, required_spacing_m, WallGeometry};
use ciris_sim_core::homogenization::{derive_bilinear_cohesive_law, max_bilinear_spacing_m};
use ciris_sim_core::impact::{ImpactConfig, ImpactResidual, ImpactRun, ImpactScene};
use ciris_sim_core::holon::CertificationStatus;
use ciris_sim_core::material::IsotropicMaterial;

const MODAL: [u64; 3] = [30, 60, 10];
const SEED: u64 = 0xC1F1_00E1_0000_A97C;

fn law() -> DrawParams {
    DrawParams {
        grain_mu_ln_m: (7.5e-4_f64).ln(),
        grain_sigma_ln: 0.5,
        weibull_m: 10.0,
        weibull_sigma0_pa: 2.0e8,
        flaw_density_per_m3: 1.0e10,
    }
}

fn config(tol: f64) -> ImpactConfig {
    ImpactConfig {
        geometry: WallGeometry {
            side_m: 0.125,
            thickness_m: 0.1,
            notch_m: 0.0,
        },
        aim_y_m: 0.0625,
        ball_mass_kg: 0.4,
        ball_radius_m: 0.012,
        ball_speed_m_s: 9.0,
        macro_tolerance: tol,
        grading: 2.0,
        duration_s: 1.6e-3,
        cfl: 0.25,
        solver_zeta: 0.15,
        contact_stiffness_fraction: 0.02,
        guard_fraction: 0.5,
        residual: ImpactResidual::Correct,
    }
}

fn scene(tol: f64, root_grain: u32, seed: u64) -> ImpactScene {
    ImpactScene::new(
        config(tol),
        IsotropicMaterial::LAC_DU_BONNET,
        law(),
        &MODAL,
        1_000_000,
        root_grain,
        seed,
    )
    .unwrap()
}

fn report(label: &str, run: &ImpactRun, seconds: f64) {
    let c = &run.result.certificate;
    println!(
        "[{label}] status {:?}; materializations {} ({} corridor / {} far); solves {}",
        c.status, run.result.materializations, run.materialized_near, run.materialized_far,
        run.solves_run
    );
    println!(
        "  finest active {:.2} mm (required {:.2} mm, constructor h_max {:.2} mm, floor {:.2} mm)",
        1e3 * run.finest_active_m,
        1e3 * run.required_spacing_m,
        1e3 * run.constructor_h_max_m,
        1e3 * run.grain_floor_m
    );
    println!(
        "  observables: impulse {:.4} N*s; crack area {:.6} m2; detached mass {:.3} kg",
        c.observables[0], c.observables[1], c.observables[2]
    );
    println!(
        "  bound {:.4}; conservation {:.3e}; guarded bonds {} (worst load {:.3}); [{:.1} s]",
        c.macro_error_bound, c.conservation_residual, run.guarded_bonds,
        run.guarded_worst_load, seconds
    );
}

fn main() {
    let material = IsotropicMaterial::LAC_DU_BONNET;
    let l_ch = characteristic_length_m(&material);
    let required = required_spacing_m(&material);
    let h_max = max_bilinear_spacing_m(&material).unwrap();
    println!("LAC_DU_BONNET: l_ch = {:.2} mm; required corridor spacing l_ch/10 = {:.2} mm", 1e3 * l_ch, 1e3 * required);
    println!("constructor validity (PLANE STRESS: lambda = E*nu/(1-nu^2)): h_max = 2 G_F (lambda+mu)/f_t^2 = {:.2} mm", 1e3 * h_max);
    let coarse_mass = material.density_kg_m3 * 0.125 * 0.125 * 0.1;
    println!(
        "constructor at the initial 125 mm frontier: {:?}",
        derive_bilinear_cohesive_law(&material, coarse_mass, 0.125)
    );

    // Convergence legs (the calibration record for the staked test tolerances).
    for (label, tol, grain) in [
        ("coarse tol 4.0", 4.0, 64),
        ("target tol 1.0", 1.0, 64),
        ("fine   tol 0.5", 0.5, 64),
    ] {
        let start = std::time::Instant::now();
        let mut wall = scene(tol, grain, SEED);
        let run = wall.certify().unwrap();
        report(label, &run, start.elapsed().as_secs_f64());
        assert!(run.result.certificate.passed(), "{label} failed");
        if tol <= 1.0 {
            println!(
                "  brittleness at the certified corridor: h_max/h = {:.1}",
                run.constructor_h_max_m / run.finest_active_m
            );
        }
    }

    // Replay: bit-identical adaptive run on the same quenched Record seed.
    let mut first = scene(1.0, 64, SEED);
    let a = first.certify().unwrap();
    let mut second = scene(1.0, 64, SEED);
    let b = second.certify().unwrap();
    let identical = first.arena().holons() == second.arena().holons()
        && (0..3).all(|k| {
            a.result.certificate.observables[k].to_bits()
                == b.result.certificate.observables[k].to_bits()
        });
    println!("\nreplay with the persisted seed is bit-identical: {identical}");
    assert!(identical);

    // The honest refusal, with the numbers displayed: a grain floor coarser than
    // the requirement cannot certify this specimen.
    let start = std::time::Instant::now();
    let mut floored = scene(1.0, 8, SEED);
    let refusal = floored.certify().unwrap();
    println!(
        "\nroot grain 8: floor {:.2} mm vs required {:.2} mm -> {:?} after {} materializations [{:.1} s]",
        1e3 * refusal.grain_floor_m,
        1e3 * refusal.required_spacing_m,
        refusal.result.certificate.status,
        refusal.result.materializations,
        start.elapsed().as_secs_f64()
    );
    assert_eq!(refusal.result.certificate.status, CertificationStatus::GrainFloor);
}
