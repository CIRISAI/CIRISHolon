//! The measured report for the multiscale sandbox: the ladder, the ledger, and what one
//! throw actually costs at each tier that runs.
//!
//! ```sh
//! cargo run -p holon-sandbox --example sandbox_report --release
//! ```

use holon_sandbox::incremental::{certify_incremental, Budget, PriorityAdapter, Workspace};
use holon_sandbox::scene::{root_scene, QuadrantMaterializer, ResolutionModel};
use holon_sandbox::sim::{Session, Verdict};
use holon_sandbox::tier::{tiers, Evaluator, Ledger, TierId};
use ciris_sim_core::runtime::certify_runtime_adaptive;

fn main() {
    println!("== the ladder ==");
    println!(
        "{:<11}{:>12}{:>12}{:>10}{:>20}{:>14}  {}",
        "tier", "g0 (m)", "domain (m)", "root", "constituents", "l_ch/10 (m)", "evaluator"
    );
    for tier in tiers() {
        let required = tier
            .required_spacing_m()
            .map(|value| format!("{value:.3e}"))
            .unwrap_or_else(|| "-".into());
        let evaluator = match tier.evaluator {
            Evaluator::GaugePlaquette => "exact plaquette".to_string(),
            Evaluator::GranularContact => "granular contact".to_string(),
            Evaluator::Cohesive => "cohesive".to_string(),
            Evaluator::GeodesicChart { tier_eps_max } => {
                format!("geodesic (stake {tier_eps_max:.0e})")
            }
            Evaluator::Unavailable(refusal) => format!("REFUSES: {refusal:?}"),
        };
        println!(
            "{:<11}{:>12.4e}{:>12.4e}{:>10}{:>20}{:>14}  {}",
            tier.name,
            tier.g0_m,
            tier.domain_m,
            tier.root_grain_units,
            tier.constituents,
            required,
            evaluator
        );
    }

    println!("\n== the ledger, and what it refuses ==");
    for tier in tiers() {
        if !tier.domain_m.is_finite() {
            continue;
        }
        let declared = match tier.ledger() {
            Ledger::Fits { constituents, binding, .. } => {
                format!("fits: {constituents:e} at g0 (binding lane {binding:?})")
            }
            other => format!("{other:?}"),
        };
        let atoms = match tier.ledger_in_atoms() {
            Ledger::Fits { constituents, .. } => format!("fits: {constituents:e} atoms"),
            Ledger::Overflows { lane, factor } => {
                format!("REFUSED: {factor:.3e}x over the {lane:?} lane")
            }
            other => format!("{other:?}"),
        };
        println!("  {:<11} {:<52} in atoms: {}", tier.name, declared, atoms);
    }

    println!("\n== one throw, per tier ==");
    for id in TierId::ALL {
        let mut session = Session::new(id);
        let started = std::time::Instant::now();
        session.throw(0.5, 0.4, 0.6);
        let certify = started.elapsed().as_secs_f64();

        let stepped = std::time::Instant::now();
        for _ in 0..60 {
            session.step(1.0 / 60.0);
        }
        let frames = stepped.elapsed().as_secs_f64();

        println!(
            "  {:<11} {:<22} holons {:>6}  active {:>5}  bonds {:>5}  mats {:>5}  \
             certify {:>8.2} ms  60 frames {:>7.2} ms ({:.2} ms/frame)",
            session.tier.name,
            format!("{:?}", session.verdict()),
            session.arena().len(),
            session.nodes().len(),
            session.relations().len(),
            session.materializations(),
            1e3 * certify,
            1e3 * frames,
            1e3 * frames / 60.0,
        );
        if session.verdict() == Verdict::Certified {
            println!(
                "              impulse {:.4e} N*s   peak contact {:.4e} N   dt {:.3e} s   \
                 slow motion {:.4}x real   contact softened {:.1}x",
                session.impulse_n_s(),
                session.peak_contact_n(),
                session.dt_s(),
                session.slow_motion(),
                session.softening_factor(),
            );
            println!(
                "              contact k {:.4e} N/m (physical {:.4e})  projectile r {:.3e} m \
                 at ({:.3e}, {:.3e})",
                session.contact_stiffness_n_m(),
                session.physical_stiffness_n_m(),
                session.projectile().radius_m,
                session.projectile().position[0],
                session.projectile().position[1],
            );
        }
        if let Some(refusal) = session.law_refusal() {
            println!("              relation law refused: {refusal:?} (contact only)");
        }
    }

    println!("\n== incremental vs the shipped certifier, same model ==");
    speedup();
}

/// The payoff, measured on the sandbox's own scene rather than a synthetic one.
///
/// The sweep is over GRADING — how fast the resolution demand relaxes with distance
/// from the throw — because that is the knob that actually sets how many holons a
/// throw materializes. Sweeping the root grain instead shrinks the domain along with
/// it and the frontier never grows, which is why the first version of this table
/// reported a 4x speedup on a scene of sixty holons.
fn speedup() {
    let tier = holon_sandbox::tier::tier(TierId::Sandbox);
    let matter_line = tier.fill * tier.domain_m;
    let required = tier.required_spacing_m().unwrap();
    let focus = [0.5 * tier.domain_m, 0.4 * tier.domain_m];

    println!(
        "{:>9}{:>10}{:>9}{:>15}{:>18}{:>10}",
        "grading", "holons", "active", "shipped (ms)", "incremental (ms)", "speedup"
    );
    let mut reference_alive = true;
    for grading in [4.0_f64, 2.0, 1.0, 0.5, 0.25, 0.125] {
        let mut arena = root_scene(&tier).unwrap();
        let mut model = ResolutionModel::new(tier.domain_m, required, tier.acuity_m(), grading, focus);
        let mut materializer =
            QuadrantMaterializer::new(tier.domain_m, matter_line, required.min(tier.g0_m));
        let mut workspace = Workspace::new();
        let budget = Budget {
            macro_tolerance: 0.0,
            conservation_tolerance: 1.0e-9,
            max_rounds: 20_000_000,
            max_holons: 4_000_000,
        };
        let started = std::time::Instant::now();
        let fast =
            certify_incremental(&mut arena, &mut model, &mut materializer, &mut workspace, budget)
                .unwrap();
        let fast_s = started.elapsed().as_secs_f64();

        if !reference_alive {
            println!(
                "{grading:>9.3}{:>10}{:>9}{:>15}{:>18.2}{:>10}",
                arena.len(),
                fast.active.len(),
                "not run",
                1e3 * fast_s,
                "-"
            );
            continue;
        }

        let mut reference_arena = root_scene(&tier).unwrap();
        let mut reference = PriorityAdapter::new(
            ResolutionModel::new(tier.domain_m, required, tier.acuity_m(), grading, focus),
            0.0,
        );
        let mut reference_materializer =
            QuadrantMaterializer::new(tier.domain_m, matter_line, required.min(tier.g0_m));
        let started = std::time::Instant::now();
        let slow = certify_runtime_adaptive(
            &mut reference_arena,
            &mut reference,
            &mut reference_materializer,
            0.0,
            1.0e-9,
        )
        .unwrap();
        let slow_s = started.elapsed().as_secs_f64();
        assert_eq!(
            fast.status, slow.certificate.status,
            "the two certifiers disagreed at grading {grading}"
        );
        assert_eq!(
            fast.active.len(),
            slow.certificate.frontier.active_count(),
            "the two certifiers reached different frontiers at grading {grading}"
        );
        println!(
            "{grading:>9.3}{:>10}{:>9}{:>15.2}{:>18.2}{:>9.0}x",
            arena.len(),
            fast.active.len(),
            1e3 * slow_s,
            1e3 * fast_s,
            slow_s / fast_s.max(1e-9)
        );
        // Past a minute the shipped path stops being informative and starts being a
        // wait. The rows after this one run the incremental path alone.
        if slow_s > 60.0 {
            reference_alive = false;
        }
    }
}
