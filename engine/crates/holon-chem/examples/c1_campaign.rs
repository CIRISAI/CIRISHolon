//! C1 GATE CAMPAIGN — the run staked by `conformance/water_observatory/C1_GATE_PREREG.md`.
//!
//! Stages are separate so a long one can be detached and resumed:
//!   c1_campaign dvr        G0, G2, G6 and the spectral references
//!   c1_campaign ladder     G3, the P-convergence law
//!   c1_campaign production G1, G4, G7 — the headline and the isotope shift
//!   c1_campaign square     G5 and the classical limit (prereg gate (c)/(d))
//!
//! Every number printed here is either read from the engine or computed from it. No
//! constant in this file is transcribed from a table.
use holon_chem::elements::Species;
use holon_chem::h2::{equilibrium, h2_point};
use holon_chem::rpmd::*;
use holon_chem::tower::{ClassicalState, RingPolymerState};

const TEMPERATURE_K: f64 = 300.0;
const DT: f64 = 4.0;
const N_KNOTS: usize = 4096;
const DVR_N: usize = 601;
const DVR_LO: f64 = 0.50;
const DVR_HI: f64 = 9.00;
const N_LEVELS: usize = 6;
const CHAINS: usize = 8;
const TOL: f64 = 1e-9;

fn beta() -> f64 {
    1.0 / (K_B_HARTREE_PER_KELVIN * TEMPERATURE_K)
}

struct Setup {
    r_e: f64,
    d_e: f64,
    v_min: f64,
    curv: f64,
    omega: f64,
    mu_h2: f64,
    mu_d2: f64,
    r_floor: f64,
}

fn setup() -> Setup {
    let (r_e, d_e, v_min) = equilibrium();
    let curv = h2_point(r_e).e2;
    let mu_h2 = Vib1D::reduced_mass_me(Species::HYDROGEN.mass_u, Species::HYDROGEN.mass_u);
    let mu_d2 = Vib1D::reduced_mass_me(MASS_U_DEUTERIUM, MASS_U_DEUTERIUM);
    let (r_floor, _) = banked_range();
    Setup { r_e, d_e, v_min, curv, omega: (curv / mu_h2).sqrt(), mu_h2, mu_d2, r_floor }
}

fn header(s: &Setup) {
    println!("# C1 GATE CAMPAIGN  (prereg conformance/water_observatory/C1_GATE_PREREG.md)");
    println!("# curve: STO-3G FCI H-H, read from h2::h2_point / h2::equilibrium");
    println!("R_e {:.12} bohr   D_e {:.12} Ha   V(R_e) {:.12} Ha", s.r_e, s.d_e, s.v_min);
    println!("E''(R_e) {:.12} Ha/bohr^2   omega_harm {:.12} a.u. ({:.3} cm^-1)",
        s.curv, s.omega, s.omega * HARTREE_TO_CM_INV);
    println!("mu(H2) {:.9} m_e   mu(D2) {:.9} m_e   T {TEMPERATURE_K} K   beta {:.6} Ha^-1",
        s.mu_h2, s.mu_d2, beta());
    println!("beta*omega(H2) {:.4}   banked inner knot (r_floor) {:.9} bohr", beta()*s.omega, s.r_floor);
}

/// Both spectral references on both surfaces, with the interpolation systematic.
fn stage_dvr(s: &Setup) {
    header(s);
    let banked = BankedPes::h2(N_KNOTS);
    let (rmin_i, vmin_i) = banked.minimum();
    let (herm_e, herm_f) = banked.table().hermite_error(4);
    println!("\n## banked table ({N_KNOTS} knots)");
    println!("hermite max|dE| {herm_e:.4e} Ha   max|dF| {herm_f:.4e} Ha/bohr");
    println!("interpolant minimum R {rmin_i:.12} (d {:+.3e})  V {vmin_i:.12} (d {:+.3e})",
        rmin_i - s.r_e, vmin_i - s.v_min);

    for (iso, mu) in [("H2", s.mu_h2), ("D2", s.mu_d2)] {
        for (sname, pes) in [("exact", &ExactPes as &dyn Pes), ("banked", &banked as &dyn Pes)] {
            let sys = Vib1D { mu, pes, name: "iso" };
            let vmin = if sname == "exact" { s.v_min } else { vmin_i };
            match dvr_reference(&sys, DVR_LO, DVR_HI, s.r_floor, DVR_N, N_LEVELS, TOL) {
                Ok(r) => {
                    println!("\n## DVR {iso} / {sname}");
                    println!("  residuals: ritz {:.3e}  grid {:.3e}  box {:.3e}  numerov {:.3e}",
                        r.ritz_residual, r.grid_shift, r.box_shift, r.numerov_gap);
                    println!("  WORK: solves {}  potential_calls {}", r.solves, r.potential_calls);
                    for (n, e) in r.levels.iter().enumerate() {
                        println!("  E{n} {:.12}   (E{n}-Vmin) {:.12}", e, e - vmin);
                    }
                    let zpe = r.zpe(vmin);
                    let eth = r.thermal_energy(beta(), vmin);
                    println!("  ZPE {zpe:.12} Ha   E_thermal(300K) {eth:.12} Ha   thermal-ZPE {:+.3e} Ha",
                        eth - zpe);
                    println!("  omega_e (E1-E0) {:.9} a.u. = {:.4} cm^-1",
                        r.levels[1]-r.levels[0], (r.levels[1]-r.levels[0])*HARTREE_TO_CM_INV);
                    let we = r.levels[1]-r.levels[0];
                    let wexe = 0.5*(2.0*r.levels[1]-r.levels[0]-r.levels[2]);
                    println!("  omega_e x_e (from E0,E1,E2) {:.9} a.u. = {:.4} cm^-1",
                        wexe, wexe*HARTREE_TO_CM_INV);
                    println!("  ZPE/(omega_harm/2) {:.9}   ZPE - omega_harm/2 {:+.6e} Ha ({:+.4}%)",
                        zpe/(0.5*s.omega), zpe-0.5*s.omega, 100.0*(zpe-0.5*s.omega)/(0.5*s.omega));
                    let _ = we;
                }
                Err(e) => println!("\n## DVR {iso} / {sname}  {e}"),
            }
        }
    }
}

fn banked_factory() -> Box<dyn Pes> {
    Box::new(BankedPes::h2(N_KNOTS))
}

fn zpe_of(mu: f64, banked: &BankedPes, vmin: f64) -> (f64, DvrReference) {
    let s = setup();
    let sys = Vib1D { mu, pes: banked, name: "x" };
    let r = dvr_reference(&sys, DVR_LO, DVR_HI, s.r_floor, DVR_N, N_LEVELS, TOL)
        .expect("G0: the banked reference must certify itself");
    (r.zpe(vmin), r)
}

/// G3: the P ladder.
fn stage_ladder(s: &Setup, steps: u64) {
    header(s);
    let banked = BankedPes::h2(N_KNOTS);
    let (_, vmin_i) = banked.minimum();
    let (zpe_ref, rref) = zpe_of(s.mu_h2, &banked, vmin_i);
    let e_ref = rref.thermal_energy(beta(), vmin_i);
    println!("\n## G3 ladder (H2, banked, dt={DT}, {CHAINS} chains, {steps} sampled steps/chain)");
    println!("reference E_thermal {e_ref:.12} Ha   ZPE {zpe_ref:.12} Ha");
    // G3's prediction, computed BEFORE the ladder runs: the exact P-bead energy of a
    // harmonic oscillator at this curve's own omega_harm, with no free parameters.
    let ladder = [1usize, 2, 4, 8, 16, 32, 64, 128, 256, 512];
    let e256 = harmonic_ring_energy(s.omega, beta(), 256);
    println!("closed-form prediction (harmonic_ring_energy at omega_harm), E_P - E_256:");
    for &p in &ladder {
        println!("   P={p:5}  E_P-Vmin {:.9}   E_P-E_256 {:+.6e}",
            harmonic_ring_energy(s.omega, beta(), p), harmonic_ring_energy(s.omega, beta(), p) - e256);
    }
    println!("     P    E_cv-Vmin        err         E_prim-Vmin      err         E_cv-E_ref      pred(E_P-E_256)  meas(E_cv-E_cv256)  Rg        tau     Vcalls        s");
    let mut measured: Vec<(usize, f64, f64)> = Vec::new();
    for &p in &ladder {
        let cfg = PimdConfig { p, temperature_k: TEMPERATURE_K, dt: DT,
            gamma_centroid: s.omega, steps_equil: steps / 10, steps_sample: steps,
            seed: 0xC1_0001 };
        let t0 = std::time::Instant::now();
        let rep = run_pimd_chains(s.mu_h2, "H2", &cfg, CHAINS, s.r_e, &banked_factory);
        println!("{p:6}  {:.9} {:.2e}  {:.9} {:.2e}  {:+.4e}  {:+.6e}  (filled below)  {:.6}  {:6.1}  {:>10}  {:6.1}",
            rep.e_virial - vmin_i, rep.e_virial_err,
            rep.e_primitive - vmin_i, rep.e_primitive_err,
            (rep.e_virial - vmin_i) - e_ref,
            harmonic_ring_energy(s.omega, beta(), p) - e256,
            rep.radius_of_gyration, rep.tau_int,
            rep.potential_calls, t0.elapsed().as_secs_f64());
        assert_eq!(rep.excursions, 0, "banked surface was extrapolated: the run is VOID");
        measured.push((p, rep.e_virial, rep.e_virial_err));
    }

    // G3(a): the whole shape, against a prediction with no free parameters.
    let m256 = measured.iter().find(|r| r.0 == 256).map(|r| r.1).unwrap();
    let s256 = measured.iter().find(|r| r.0 == 256).map(|r| r.2).unwrap();
    println!("\n## G3(a) shape: measured (E_cv(P) - E_cv(256)) vs closed form");
    println!("     P    predicted        measured         err          d           d/pred      3sig/pred");
    for &(p, e, err) in measured.iter() {
        if p == 256 || p == 512 { continue; }
        let pred = harmonic_ring_energy(s.omega, beta(), p) - e256;
        let meas = e - m256;
        let sig = (err * err + s256 * s256).sqrt();
        println!("{p:6}  {pred:+.6e}  {meas:+.6e}  {sig:.2e}  {:+.3e}  {:+8.4}%  {:8.4}%",
            meas - pred, 100.0 * (meas - pred) / pred, 100.0 * 3.0 * sig / pred.abs());
    }
    // G3(b): the exponent, on the window the noise floor allows.
    let win = [16usize, 32, 64];
    let xs: Vec<f64> = win.iter().map(|&p| (p as f64).ln()).collect();
    let ys: Vec<f64> = win.iter()
        .map(|&p| (measured.iter().find(|r| r.0 == p).unwrap().1 - m256).abs().ln())
        .collect();
    let n = xs.len() as f64;
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let num: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = xs.iter().map(|x| (x - mx) * (x - mx)).sum();
    let cx: Vec<f64> = win.iter()
        .map(|&p| (harmonic_ring_energy(s.omega, beta(), p) - e256).abs().ln())
        .collect();
    let mcy = cx.iter().sum::<f64>() / n;
    let cnum: f64 = xs.iter().zip(&cx).map(|(x, y)| (x - mx) * (y - mcy)).sum();
    println!("\n## G3(b) exponent on P in {win:?}");
    println!("  measured  x = {:.4}", -num / den);
    println!("  closed form x = {:.4}   (the asymptotic 2 is NOT reached in this window)", -cnum / den);
}

/// G1, G4, G7: the headline and the isotope shift.
fn stage_production(s: &Setup, steps: u64) {
    header(s);
    let banked = BankedPes::h2(N_KNOTS);
    let (_, vmin_i) = banked.minimum();
    println!("\n## G1/G4 production (banked, {CHAINS} chains, {steps} sampled steps/chain)");
    let mut zpes = Vec::new();
    for (iso, mu) in [("H2", s.mu_h2), ("D2", s.mu_d2)] {
        let (zpe_ref, rref) = zpe_of(mu, &banked, vmin_i);
        let e_ref = rref.thermal_energy(beta(), vmin_i);
        // Headline plus the dt convergence check. The P axis is the ladder stage's job.
        for &(p, dt) in &[(256usize, DT), (256, DT / 2.0)] {
            let cfg = PimdConfig { p, temperature_k: TEMPERATURE_K, dt,
                gamma_centroid: s.omega, steps_equil: steps / 10, steps_sample: steps,
                seed: 0xC1_0001 };
            let t0 = std::time::Instant::now();
            let rep = run_pimd_chains(mu, iso, &cfg, CHAINS, s.r_e, &banked_factory);
            let zpe = rep.e_virial - vmin_i;
            // Both sides relative to the SAME zero. `thermal_energy` is already measured
            // from V_min; subtracting it from an absolute energy is how the first ladder
            // printed a -1.15 Ha "residual" on a 1e-5 Ha quantity.
            let d = zpe - e_ref;
            println!("{iso} P={p} dt={dt}: ZPE_RPMD {zpe:.10} +- {:.2e}   ZPE_DVR {zpe_ref:.10}",
                rep.e_virial_err);
            println!("    E_cv-E_ref {d:+.4e} Ha = {:+.4}% of ZPE_DVR   stat {:.4}%   prim-virial {:+.3e}",
                100.0*d/zpe_ref, 100.0*rep.e_virial_err/zpe_ref, rep.e_primitive - rep.e_virial);
            println!("    Rg {:.6} bohr   tau {:.1}   samples {}   Vcalls {}   excursions {}   wall {:.1}s",
                rep.radius_of_gyration, rep.tau_int, rep.samples, rep.potential_calls,
                rep.excursions, t0.elapsed().as_secs_f64());
            if p == 256 && dt == DT {
                zpes.push((zpe, rep.e_virial_err, zpe_ref));
            }
            assert_eq!(rep.excursions, 0, "banked surface was extrapolated: the run is VOID");
        }
    }
    if zpes.len() == 2 {
        let (zh, eh, rh) = zpes[0];
        let (zd, ed, rd) = zpes[1];
        let harm = (s.mu_h2 / s.mu_d2).sqrt();
        println!("\n## G4 isotope shift");
        println!("  harmonic ratio sqrt(mu_H2/mu_D2)      {harm:.9}");
        println!("  RPMD   ZPE(D2)/ZPE(H2) = {:.9} +- {:.2e}", zd/zh,
            (zd/zh)*((ed/zd).powi(2)+(eh/zh).powi(2)).sqrt());
        println!("  DVR    ZPE(D2)/ZPE(H2) = {:.9}", rd/rh);
        println!("  RPMD - harmonic  {:+.4e}   DVR - harmonic  {:+.4e}", zd/zh - harm, rd/rh - harm);
    }
}

/// G5 and the classical limit.
fn stage_square(s: &Setup) {
    header(s);
    let banked = BankedPes::h2(N_KNOTS);
    let b = beta();

    // --- prereg gate (c): P = 1 recovers the classical trajectory.
    // M-FIXED-POINT-TRAJECTORY: launched DISPLACED with nonzero velocity, and the path
    // length is asserted, so the square cannot be closed by nothing happening.
    println!("\n## gate (c): P=1 is the classical trajectory");
    let start = || ClassicalState {
        positions: vec![[0.0, 0.0, 0.0], [0.0, 0.0, s.r_e + 0.35]],
        velocities: vec![[0.0, 0.0, -3.0e-4], [1.0e-4, 0.0, 3.0e-4]],
        masses: vec![Species::HYDROGEN.mass_u, Species::HYDROGEN.mass_u],
    };
    let mut cl = start();
    let mut rp = RingPolymerState {
        beads_pos: vec![start().positions],
        beads_vel: vec![start().velocities],
        masses: start().masses,
    };
    let e0 = classical_energy_3d(&cl, &banked);
    let mut path = 0.0f64;
    let mut worst_pos = 0.0f64;
    let mut worst_vel = 0.0f64;
    let mut half_a = 0.0f64;
    let mut half_b = 0.0f64;
    let steps = 5000;
    for st in 0..steps {
        let before = cl.positions[1][2];
        classical_step_3d(&mut cl, DT, &banked);
        ring_step_3d(&mut rp, DT, b, &banked);
        path += (cl.positions[1][2] - before).abs();
        let de = (classical_energy_3d(&cl, &banked) - e0).abs();
        if st < steps / 2 {
            half_a = half_a.max(de);
        } else {
            half_b = half_b.max(de);
        }
        for i in 0..2 {
            for a in 0..3 {
                worst_pos = worst_pos.max((rp.beads_pos[0][i][a] - cl.positions[i][a]).abs());
                worst_vel = worst_vel.max((rp.beads_vel[0][i][a] - cl.velocities[i][a]).abs());
            }
        }
    }
    let e1 = classical_energy_3d(&cl, &banked);
    println!("  {steps} steps at dt={DT}: worst |dR| {worst_pos:.3e} bohr   worst |dV| {worst_vel:.3e}");
    println!("  path length {path:.6} bohr (must exceed 0.1 or the gate is VOID)");
    println!("  classical energy: E0 {e0:.9}  end-of-run |dE| {:.3e} Ha", (e1 - e0).abs());
    println!("  worst |dE| over the run, first half {half_a:.3e} / second half {half_b:.3e} Ha");
    println!("  (velocity Verlet's energy error is BOUNDED and oscillatory, not secular;");
    println!("   the two halves are printed so a secular drift would be visible as growth)");

    // --- the mutation control: at P=2 the same comparison MUST fail, or gate (c) is blind.
    let mut rp2 = RingPolymerState {
        beads_pos: vec![start().positions, start().positions],
        beads_vel: vec![start().velocities, start().velocities],
        masses: start().masses,
    };
    // Spread the ring by hand so it is a ring and not a replicated point.
    rp2.beads_pos[0][1][2] += 0.05;
    rp2.beads_pos[1][1][2] -= 0.05;
    let mut cl2 = start();
    let mut worst2 = 0.0f64;
    for _ in 0..steps {
        classical_step_3d(&mut cl2, DT, &banked);
        ring_step_3d(&mut rp2, DT, b, &banked);
        let c = centroid_state(&rp2);
        for i in 0..2 {
            for a in 0..3 {
                worst2 = worst2.max((c.positions[i][a] - cl2.positions[i][a]).abs());
            }
        }
    }
    println!("  MUTATION CONTROL P=2 (spread ring): worst |dR| {worst2:.3e} bohr — must be LARGE");

    // --- gate (d): the bead-forgetting square, with its budget.
    //
    // AVERAGED over 32 independent free-ring-polymer draws per P. One draw per row is a
    // sample, not a measurement: the first version of this table read R_g of 0.52, 0.49,
    // 0.48, 0.28, 0.36, 0.31 down the P column and the shape it appeared to show was the
    // RNG's, not the ring's.
    const DRAWS: usize = 32;
    println!("\n## gate (d)/G5: the bead-forgetting commuting square");
    println!("  states are free-ring-polymer draws at 300 K (the shape a LIFTED state has");
    println!("  before the potential has acted on it); {DRAWS} draws per row, mean +- spread");
    println!("     P   Rg(bohr)          force_gap         defect_pos        defect_vel        H_P drift/|H_P|");
    for &p in &[1usize, 2, 4, 8, 16, 32, 64] {
        let mut rg = Vec::new();
        let mut fg = Vec::new();
        let mut dp = Vec::new();
        let mut dv = Vec::new();
        let mut drift = Vec::new();
        for d in 0..DRAWS {
            let mut st = thermal_ring(p, b, s, 0xC1_5A17 + 0x9E37_79B9 * d as u64);
            let bud = commuting_budget(&st, DT, b, &banked);
            rg.push(bud.radius_of_gyration);
            fg.push(bud.force_gap);
            dp.push(bud.defect_pos);
            dv.push(bud.defect_vel);
            let h0 = ring_energy_3d(&st, b, &banked);
            for _ in 0..200 {
                ring_step_3d(&mut st, DT, b, &banked);
            }
            drift.push((ring_energy_3d(&st, b, &banked) - h0) / h0.abs());
        }
        let ms = |v: &Vec<f64>| {
            let m = v.iter().sum::<f64>() / v.len() as f64;
            let sd = (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / v.len() as f64).sqrt();
            (m, sd)
        };
        let (a1, b1) = ms(&rg);
        let (a2, b2) = ms(&fg);
        let (a3, b3) = ms(&dp);
        let (a4, b4) = ms(&dv);
        let (a5, _) = ms(&drift);
        println!("{p:6}  {a1:.4}+-{b1:.4}  {a2:.3e}+-{b2:.1e}  {a3:.3e}+-{b3:.1e}  {a4:.3e}+-{b4:.1e}  {a5:+.3e}");
    }

    // The two scaling laws, on a DETERMINISTIC ring so the law is read and not the noise.
    println!("\n  dt scaling of defect_pos (deterministic ring, P=16, spread 0.08 bohr):");
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for &dt in &[8.0f64, 4.0, 2.0, 1.0, 0.5] {
        let st = cosine_ring(16, s.r_e, 0.08);
        let bud = commuting_budget(&st, dt, b, &banked);
        println!("   dt {dt:5.2}  defect_pos {:.6e}  force_gap {:.6e}", bud.defect_pos, bud.force_gap);
        xs.push(dt.ln());
        ys.push(bud.defect_pos.ln());
    }
    println!("   fitted exponent {:.4}  (staked window [1.7, 2.3])", fit_slope(&xs, &ys));

    println!("\n  Rg scaling of force_gap (deterministic ring, P=32, dt={DT}):");
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for &spread in &[0.02f64, 0.04, 0.08, 0.16, 0.32, 0.64] {
        let st = cosine_ring(32, s.r_e, spread);
        let bud = commuting_budget(&st, DT, b, &banked);
        println!("   spread {spread:5.3}  Rg {:.6e}  force_gap {:.6e}  defect_pos {:.6e}",
            bud.radius_of_gyration, bud.force_gap, bud.defect_pos);
        if spread <= 0.16 {
            xs.push(bud.radius_of_gyration.ln());
            ys.push(bud.force_gap.ln());
        }
    }
    println!("   fitted exponent over spread <= 0.16 bohr: {:.4}  (staked window [1.6, 2.4])",
        fit_slope(&xs, &ys));
    println!("   the two widest rings are OUTSIDE the quadratic region and are reported as");
    println!("   the law's measured domain edge, not folded into the fit");
}

fn fit_slope(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let num: f64 = xs.iter().zip(ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let den: f64 = xs.iter().map(|x| (x - mx) * (x - mx)).sum();
    num / den
}

/// A ring whose beads sit on a cosine along the bond: a DETERMINISTIC shape, so the
/// scaling laws are read off the geometry rather than off a random draw.
fn cosine_ring(p: usize, r_e: f64, spread: f64) -> RingPolymerState {
    let mut beads_pos = vec![vec![[0.0f64; 3]; 2]; p];
    let beads_vel = vec![vec![[0.0f64; 3]; 2]; p];
    for k in 0..p {
        let phase = 2.0 * core::f64::consts::PI * (k as f64) / (p as f64);
        beads_pos[k][1] = [0.0, 0.0, r_e + spread * phase.cos()];
    }
    RingPolymerState {
        beads_pos,
        beads_vel,
        masses: vec![Species::HYDROGEN.mass_u; 2],
    }
}

/// A ring drawn from the free-ring-polymer distribution about the equilibrium geometry:
/// a state a trajectory actually reaches, not a replicated point.
fn thermal_ring(p: usize, beta: f64, s: &Setup, seed: u64) -> RingPolymerState {
    scaled_ring(p, beta, s, seed, 1.0)
}

fn scaled_ring(p: usize, beta: f64, s: &Setup, seed: u64, scale: f64) -> RingPolymerState {
    let mut rng = Rng::new(seed);
    let nm = NormalModes::new(p, beta);
    let m = Species::HYDROGEN.mass_u * holon_chem::elements::M_E_PER_U;
    let beta_n = beta / p as f64;
    let mut beads_pos = vec![vec![[0.0f64; 3]; 2]; p];
    let mut beads_vel = vec![vec![[0.0f64; 3]; 2]; p];
    let mut qt = vec![0.0f64; p];
    let mut q = vec![0.0f64; p];
    for i in 0..2 {
        for a in 0..3 {
            let centre = if i == 1 && a == 2 { s.r_e } else { 0.0 };
            qt[0] = centre * (p as f64).sqrt();
            for k in 1..p {
                qt[k] = scale * rng.normal() / (beta_n * m * nm.omega[k] * nm.omega[k]).sqrt();
            }
            nm.from_modes(&qt, &mut q);
            for k in 0..p {
                beads_pos[k][i][a] = q[k];
                // Velocities are ZERO by choice, not by omission: with the ring at rest
                // the whole of the square's defect is attributable to the force gap, which
                // is the mechanism gate (iii) measures. Thermal momenta would add a term
                // that is common to both paths and would only dilute the reading.
                beads_vel[k][i][a] = 0.0;
            }
        }
    }
    RingPolymerState { beads_pos, beads_vel, masses: vec![Species::HYDROGEN.mass_u; 2] }
}

/// G7: the price closes.
///
/// M-CHEAPER-THAN-ITS-PRICE made into a gate. The two unit costs are measured here, the
/// wall time of a real single-chain run is predicted from them, and the prediction is
/// compared to the observation. Run PINNED (`taskset`) on a declared core, because this
/// box is heterogeneous and an unpinned wall clock is an undeclared variable
/// (M-PLACEMENT-LOTTERY).
fn stage_price(s: &Setup) {
    header(s);
    let banked = BankedPes::h2(N_KNOTS);
    println!("\n## G7 price model");

    // (1) the banked surface, per call.
    let n = 8_000_000usize;
    let t0 = std::time::Instant::now();
    let mut acc = 0.0f64;
    for i in 0..n {
        let r = 1.0 + 0.6 * ((i % 997) as f64) / 997.0;
        let (v, d) = banked.eval(r);
        acc += v + d;
    }
    let ns_pes = t0.elapsed().as_secs_f64() / n as f64 * 1e9;
    println!("  banked PES        {ns_pes:8.2} ns/call     (acc {acc:.3})");

    // (2) the exact solver, for the record: this is why the sampler runs on the table.
    let ne = 20_000usize;
    let t0 = std::time::Instant::now();
    let mut acc = 0.0f64;
    for i in 0..ne {
        let r = 1.0 + 0.6 * ((i % 997) as f64) / 997.0;
        let (v, d) = ExactPes.eval(r);
        acc += v + d;
    }
    let ns_exact = t0.elapsed().as_secs_f64() / ne as f64 * 1e9;
    println!("  exact STO-3G FCI  {ns_exact:8.2} ns/call     ({:.0}x the table)", ns_exact / ns_pes);

    // (3) the normal-mode transform, per matrix element.
    let pp = 256usize;
    let nm = NormalModes::new(pp, beta());
    let mut x = vec![0.0f64; pp];
    let mut y = vec![0.0f64; pp];
    for (i, v) in x.iter_mut().enumerate() {
        *v = (i as f64) * 1e-3;
    }
    let reps = 40_000usize;
    let t0 = std::time::Instant::now();
    for _ in 0..reps {
        nm.to_modes(&x, &mut y);
        nm.from_modes(&y, &mut x);
    }
    let ns_elem = t0.elapsed().as_secs_f64() / (2.0 * reps as f64 * (pp * pp) as f64) * 1e9;
    println!("  normal-mode xform {ns_elem:8.4} ns/element  (P={pp}, {:.1} ns per transform)",
        ns_elem * (pp * pp) as f64);

    // (4) the prediction, and the observation.
    for &pv in &[64usize, 256] {
        let steps: u64 = 100_000;
        let predicted = (steps as f64)
            * ((pv as f64) * ns_pes + 2.0 * (pv * pv) as f64 * ns_elem)
            * 1e-9;
        let cfg = PimdConfig { p: pv, temperature_k: TEMPERATURE_K, dt: DT,
            gamma_centroid: s.omega, steps_equil: 0, steps_sample: steps, seed: 0xC1_0007 };
        let t0 = std::time::Instant::now();
        let rep = run_pimd_chains(s.mu_h2, "H2", &cfg, 1, s.r_e, &banked_factory);
        let observed = t0.elapsed().as_secs_f64();
        println!("  P={pv:4} 1 chain x {steps} steps: predicted {predicted:7.2} s  observed {observed:7.2} s  \
ratio {:.3}  (staked: within a factor 3)", observed / predicted);
        println!("       Vcalls {} (expected {})  ->  observed/predicted per call {:.3}",
            rep.potential_calls, steps * pv as u64,
            rep.potential_calls as f64 / (steps * pv as u64) as f64);
    }
}

fn main() {
    let s = setup();
    let stage = std::env::args().nth(1).unwrap_or_else(|| "dvr".into());
    let steps: u64 = std::env::args().nth(2).and_then(|x| x.parse().ok()).unwrap_or(1_000_000);
    match stage.as_str() {
        "dvr" => stage_dvr(&s),
        "ladder" => stage_ladder(&s, steps),
        "production" => stage_production(&s, steps),
        "square" => stage_square(&s),
        "price" => stage_price(&s),
        other => println!("unknown stage {other}"),
    }
}
