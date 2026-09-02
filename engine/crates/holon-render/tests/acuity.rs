//! ACUITY-B (conformance/water_observatory/ACUITY_B_PREREG.md): the observer's frame as
//! an allocation law. G0 identity, G1 momentum, G2 ledger, G4 work partition, each with
//! the plant that must fire before the gate is believed.

use holon_render::acuity::{AcuityFrame, AcuityPlant};
use holon_render::sim::{Boundary, Dims, Sim};

const N_SIDE: usize = 4;
const SPACING: f64 = 3.0;
/// Substeps per grain boundary: the momentum and angular residuals are SAMPLED at
/// boundaries (`close_grain`), so a runner that never closes one reads a residual of
/// zero and its momentum gate is vacuous — the plant behind it stayed silent until this
/// runner closed grains.
const SUBSTEPS: u32 = 64;
const FRAMES: usize = 32;
const STEPS: usize = SUBSTEPS as usize * FRAMES;

fn loaded_sim() -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    holon_render::json::load_into(
        s.table_mut(),
        &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/viewer/h2_potential.json"))
            .expect("placeholder curve readable"),
    )
    .expect("table loads");
    s.adopt_table_timescale();
    s
}

/// A periodic 3D hydrogen lattice with a seeded, momentum-free velocity field. No RNG
/// from the standard library: an LCG, so the scene re-runs byte for byte.
fn lattice() -> Box<Sim> {
    let mut s = loaded_sim();
    s.dims = Dims::Three;
    s.boundary = Boundary::Periodic;
    let edge = N_SIDE as f64 * SPACING;
    s.width = edge;
    s.height = edge;
    s.depth = edge;
    let n = N_SIDE * N_SIDE * N_SIDE;
    s.resize_storage(n);
    let mut st: u64 = 0x5341_5421;
    let mut lcg = || {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((st >> 11) as f64) / ((1u64 << 53) as f64)
    };
    let (mut px, mut py, mut pz) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let ix = i % N_SIDE;
        let iy = (i / N_SIDE) % N_SIDE;
        let iz = i / (N_SIDE * N_SIDE);
        s.atoms[i].x = (ix as f64 + 0.5) * SPACING;
        s.atoms[i].y = (iy as f64 + 0.5) * SPACING;
        s.atoms[i].z = (iz as f64 + 0.5) * SPACING;
        s.atoms[i].vx = 2e-3 * (2.0 * lcg() - 1.0);
        s.atoms[i].vy = 2e-3 * (2.0 * lcg() - 1.0);
        s.atoms[i].vz = 2e-3 * (2.0 * lcg() - 1.0);
        px += s.atoms[i].vx;
        py += s.atoms[i].vy;
        pz += s.atoms[i].vz;
    }
    for i in 0..n {
        s.atoms[i].vx -= px / n as f64;
        s.atoms[i].vy -= py / n as f64;
        s.atoms[i].vz -= pz / n as f64;
    }
    s.sync_species();
    s.rebase();
    s
}

fn frame() -> AcuityFrame {
    let c = 0.5 * N_SIDE as f64 * SPACING;
    AcuityFrame { center: [c, c, c], half: SPACING }
}

fn run(s: &mut Sim, steps: usize) {
    assert_eq!(steps % SUBSTEPS as usize, 0);
    for _ in 0..steps / SUBSTEPS as usize {
        s.step_frame(SUBSTEPS);
    }
}

#[test]
fn g0_a_frame_covering_everything_is_the_identity_in_checkpoint_bytes() {
    let mut a = lattice();
    let mut b = lattice();
    b.set_acuity(Some(AcuityFrame::everything()));
    run(&mut a, STEPS);
    run(&mut b, STEPS);
    assert_eq!(b.acuity_work.pairs_skipped, 0, "nothing is coarse under the everything frame");
    assert!(b.acuity_work.pairs_fine > 0, "the counter saw the pairs");
    assert_eq!(
        a.checkpoint().bytes,
        b.checkpoint().bytes,
        "G0: the framed step with every atom fine must be byte-identical to the classical step"
    );

    // P-1 (sector: the coarse flag): a frame that leaves one corner atom coarse must move
    // the bytes, or G0 has not compared bookkeeping.
    let mut c = lattice();
    let edge = N_SIDE as f64 * SPACING;
    c.set_acuity(Some(AcuityFrame {
        center: [edge, edge, edge],
        half: edge - 0.5 * SPACING - 0.1,
    }));
    run(&mut c, STEPS);
    assert!(c.acuity_work.pairs_skipped > 0, "P-1 precondition: something was coarse");
    assert_ne!(a.checkpoint().bytes, c.checkpoint().bytes, "P-1: a coarse atom moves the digest");
}

#[test]
fn g1_momentum_is_exact_under_a_frame_and_the_reaction_plant_fires() {
    let mut s = lattice();
    s.set_acuity(Some(frame()));
    run(&mut s, STEPS);
    let coarse_now = s.coarse.iter().filter(|&&c| c).count();
    assert!(coarse_now * 4 >= s.n, "precondition: at least a quarter of the atoms are coarse");
    assert!(
        s.momentum_gate(),
        "G1: momentum residual {:e} exceeds bound {:e} under the frame",
        s.momentum_residual_peak,
        s.momentum_bound()
    );

    // P-2 (sector: momentum): drop the reaction on the coarse side; the gate must fire.
    let mut p = lattice();
    p.set_acuity(Some(frame()));
    p.acuity_plant = AcuityPlant::DropReaction;
    run(&mut p, STEPS);
    assert!(!p.momentum_gate(), "P-2: dropping the reaction must open the momentum gate");
}

#[test]
fn g2_the_ledger_closes_with_the_observer_column_and_the_skip_plant_fires() {
    let mut s = lattice();
    s.set_acuity(Some(frame()));
    run(&mut s, STEPS);
    assert!(s.acuity_work.transitions > 0, "precondition: atoms crossed the frame");
    assert!(
        s.energy_gate(),
        "G2: drift {:e} exceeds bound {:e} with transitions posted",
        s.drift_peak,
        s.drift_bound()
    );
    assert!(s.work_columns_ok(), "the receipt columns sum to w_ext, acuity included");
    assert!(s.work.acuity != 0.0, "the observer's column carries the transition energy");

    // P-3 (sector: the ledger): apply transitions without posting them.
    let mut p = lattice();
    p.set_acuity(Some(frame()));
    p.acuity_plant = AcuityPlant::SkipLedger;
    run(&mut p, STEPS);
    assert!(p.acuity_work.transitions > 0, "P-3 precondition: transitions happened");
    assert!(!p.energy_gate(), "P-3: an unposted transition must open the drift gate");
}

#[test]
fn g4_the_work_counter_is_a_partition_and_the_miscount_plant_fires() {
    let mut s = lattice();
    s.set_acuity(Some(frame()));
    run(&mut s, STEPS);
    let n = s.n as u64;
    let examined = STEPS as u64 * n * (n - 1) / 2;
    assert_eq!(
        s.acuity_work.pairs_examined(),
        examined,
        "G4: fine + skipped is the pairs the complete route examined"
    );
    assert!(s.acuity_work.pairs_skipped > 0, "G4: the frame saved work");
    assert!(s.acuity_work.pair_saving() > 0.1, "G4: the saving is a real fraction");

    let mut p = lattice();
    p.set_acuity(Some(frame()));
    p.acuity_plant = AcuityPlant::Miscount;
    run(&mut p, STEPS);
    assert_ne!(p.acuity_work.pairs_examined(), examined, "P-4: losing the skipped count breaks the partition");
}
