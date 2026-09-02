//! ACUITY-B, the reading (conformance/water_observatory/ACUITY_B_PREREG.md B.5):
//! the observer's frame around the centre of a periodic hydrogen lattice, the unobserved
//! region carried coarse, measured against the full dynamics from the SAME checkpoint.
//! Prints G1/G2/G4 readings, the observed atoms' deviation, whole-only deltas, and the
//! work saved; then names the B.4 branch the numbers land in. Priced in the work counter.

use holon_render::acuity::AcuityFrame;
use holon_render::sim::{Boundary, Dims, Sim};

const N_SIDE: usize = 4;
/// The freeze's scene (B.5): 10 bohr spacing, thermalised. Override with the first
/// argument to read a different density — the gate tests use 3.0 to make their plants
/// observable, which is a different question from the reading.
const SPACING_DEFAULT: f64 = 10.0;
const SUBSTEPS: u32 = 64;
const FRAMES: usize = 32;
const RMS_BUDGET_BOHR: f64 = 0.5;
const DEFECT_BUDGET_OF_WELL: f64 = 0.10;

fn lattice(spacing: f64) -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    holon_render::json::load_into(
        s.table_mut(),
        &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/viewer/h2_potential.json"))
            .expect("placeholder curve readable"),
    )
    .expect("table loads");
    s.adopt_table_timescale();
    s.dims = Dims::Three;
    s.boundary = Boundary::Periodic;
    let edge = N_SIDE as f64 * spacing;
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
        s.atoms[i].x = (ix as f64 + 0.5) * spacing;
        s.atoms[i].y = (iy as f64 + 0.5) * spacing;
        s.atoms[i].z = (iz as f64 + 0.5) * spacing;
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

fn run(s: &mut Sim) {
    for _ in 0..FRAMES {
        s.step_frame(SUBSTEPS);
    }
}

fn min_image(d: f64, l: f64) -> f64 {
    d - l * (d / l).round()
}

fn main() {
    let spacing: f64 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(SPACING_DEFAULT);
    let edge = N_SIDE as f64 * spacing;
    let c = 0.5 * edge;
    let frame = AcuityFrame { center: [c, c, c], half: spacing };

    let mut full = lattice(spacing);
    let start = full.checkpoint();
    let observed: Vec<usize> = (0..full.n)
        .filter(|&i| frame.contains([full.atoms[i].x, full.atoms[i].y, full.atoms[i].z]))
        .collect();
    let d_e = full.active_d_e().abs().max(f64::MIN_POSITIVE);
    println!("# ACUITY-B reading: N = {}, box {edge:.1} bohr periodic, frame half {:.1} bohr at centre", full.n, frame.half);
    println!("# observed at start: {} atoms; well depth {d_e:.6e} Ha; {} substeps x {} frames", observed.len(), SUBSTEPS, FRAMES);

    run(&mut full);
    let full_defect = full
        .holons
        .live_rows()
        .map(|(_, r)| r.closure_defect_peak)
        .fold(0.0f64, f64::max);
    let (t_full, p_full) = (full.temperature(), full.pressure());

    let mut framed = lattice(spacing);
    framed.restore(&start).expect("the same start");
    framed.set_acuity(Some(frame));
    run(&mut framed);
    let framed_defect = framed
        .holons
        .live_rows()
        .map(|(_, r)| r.closure_defect_peak)
        .fold(0.0f64, f64::max);
    let (t_fr, p_fr) = (framed.temperature(), framed.pressure());

    // G3: the observed atoms against the full dynamics, minimum-imaged.
    let mut sq = 0.0;
    for &i in &observed {
        let dx = min_image(framed.atoms[i].x - full.atoms[i].x, edge);
        let dy = min_image(framed.atoms[i].y - full.atoms[i].y, edge);
        let dz = min_image(framed.atoms[i].z - full.atoms[i].z, edge);
        sq += dx * dx + dy * dy + dz * dz;
    }
    let rms = (sq / observed.len().max(1) as f64).sqrt();
    let defect_diff = (framed_defect - full_defect).abs() / d_e;
    let w = framed.acuity_work;

    println!("G1 momentum: residual {:.3e} vs bound {:.3e} -> {}", framed.momentum_residual_peak, framed.momentum_bound(), if framed.momentum_gate() { "PASS" } else { "FAIL" });
    println!("G2 ledger:   drift {:.3e} vs bound {:.3e} -> {}; work.acuity {:+.6e} Ha; columns_ok {}", framed.drift_peak, framed.drift_bound(), if framed.energy_gate() { "PASS" } else { "FAIL" }, framed.work.acuity, framed.work_columns_ok());
    println!("G3 observed: rms deviation {rms:.4} bohr (budget {RMS_BUDGET_BOHR}); row defect peak full {full_defect:.4e} framed {framed_defect:.4e} -> |diff|/D_e {defect_diff:.4} (budget {DEFECT_BUDGET_OF_WELL})");
    println!("G4 work:     pairs fine {} skipped {} examined {} -> saving {:.1}%; triples skipped {} quads skipped {}; transitions {}", w.pairs_fine, w.pairs_skipped, w.pairs_examined(), 100.0 * w.pair_saving(), w.triples_skipped, w.quads_skipped, w.transitions);
    println!("whole-only:  T full {t_full:.2} K framed {t_fr:.2} K; P full {p_full:.4e} framed {p_fr:.4e} (defined {})", framed.pressure_defined());
    let g3 = rms <= RMS_BUDGET_BOHR && defect_diff <= DEFECT_BUDGET_OF_WELL;
    let instrument_ok = framed.momentum_gate() && framed.energy_gate() && framed.work_columns_ok();
    let branch = if !instrument_ok {
        "(c) instrument defective — nothing learned"
    } else if g3 && w.pairs_skipped > 0 {
        "(a) inside budget with saving > 0: B's missing piece is the ALLOCATION law"
    } else {
        "(b) over budget: the unobserved region is LOAD-BEARING for the observed thing"
    };
    println!("BRANCH {branch}");
}
