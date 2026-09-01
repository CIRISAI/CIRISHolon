//! C1 GAUGING PROBE — run BEFORE the bands are frozen, on PLANTS ONLY.
//!
//! Every number here is either an instrument-setup quantity (grid range, the price of a
//! potential call, the interpolant's error) or a plant whose answer is known in closed
//! form. The anharmonic zero-point energy the gate reads is NOT computed here: gauging a
//! ruler against planted values is design, reading the target before staking is not.
use holon_chem::elements::{Species, M_E_PER_U};
use holon_chem::h2::{asymptote, equilibrium, h2_point};
use holon_chem::rpmd::*;

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_else(|| "all".into());
    let (r_e, d_e, e_at_r_e) = equilibrium();
    let mu_h2 = Vib1D::reduced_mass_me(Species::HYDROGEN.mass_u, Species::HYDROGEN.mass_u);
    let curv = h2_point(r_e).e2;
    let omega = (curv / mu_h2).sqrt();
    let beta300 = 1.0 / (K_B_HARTREE_PER_KELVIN * 300.0);

    println!("== setup ==");
    println!("R_e {r_e:.9} bohr   D_e {d_e:.9} Ha   V(R_e) {e_at_r_e:.9} Ha   asym {:.9}", asymptote());
    println!("E''(R_e) {curv:.9} Ha/bohr^2   mu(H2) {mu_h2:.6} m_e   mu(D2) {:.6} m_e",
        Vib1D::reduced_mass_me(MASS_U_DEUTERIUM, MASS_U_DEUTERIUM));
    println!("omega_harm {omega:.9} a.u. ({:.2} cm^-1)   beta(300K) {beta300:.4}   beta*omega {:.3}",
        omega * HARTREE_TO_CM_INV, beta300 * omega);
    let (bmin, bmax) = banked_range();
    println!("banked range derived from WALL_CEILING/TAIL_TOLERANCE: [{bmin:.6}, {bmax:.6}] bohr");

    if arg == "setup" { return; }

    // ---- plant 1: harmonic. DVR against omega (n + 1/2), exactly.
    println!("\n== plant 1: harmonic (DVR vs closed form) ==");
    let hp = HarmonicPes { k: curv, r0: r_e, v0: e_at_r_e };
    let sysh = Vib1D { mu: mu_h2, pes: &hp, name: "harmonic" };
    // A closed-form plant is defined on the whole line, so the box is symmetric about the
    // minimum and the floor is -infinity: the plant has no repulsive wall to lean on.
    match dvr_reference(&sysh, r_e - 4.4, r_e + 4.4, f64::NEG_INFINITY, 701, 6, 1e-9) {
        Ok(rf) => {
            println!("  ritz {:.2e} grid {:.2e} box {:.2e} numerov {:.2e} solves {} Vcalls {}",
                rf.ritz_residual, rf.grid_shift, rf.box_shift, rf.numerov_gap, rf.solves, rf.potential_calls);
            for n in 0..6 {
                let exact = e_at_r_e + omega * (n as f64 + 0.5);
                println!("  n={n}  DVR {:.12}  exact {:.12}  d {:+.3e}", rf.levels[n], exact, rf.levels[n]-exact);
            }
        }
        Err(e) => println!("  {e}"),
    }

    // ---- plant 2: Morse. ANHARMONIC, closed form -- the sector the gate reads.
    println!("\n== plant 2: Morse (DVR vs closed form) ==");
    let a_morse = omega * (mu_h2 / (2.0 * d_e)).sqrt();
    let mp = MorsePes { d_e, a: a_morse, r_e, v0: e_at_r_e };
    println!("  a = {a_morse:.9} 1/bohr (from omega and D_e, not fitted)");
    let sysm = Vib1D { mu: mu_h2, pes: &mp, name: "morse" };
    match dvr_reference(&sysm, r_e - 1.2, 14.0, f64::NEG_INFINITY, 1101, 6, 1e-9) {
        Ok(rf) => {
            println!("  ritz {:.2e} grid {:.2e} box {:.2e} numerov {:.2e}",
                rf.ritz_residual, rf.grid_shift, rf.box_shift, rf.numerov_gap);
            let ex = morse_levels(&mp, mu_h2, 6);
            for n in 0..6 {
                println!("  n={n}  DVR {:.12}  exact {:.12}  d {:+.3e}", rf.levels[n], e_at_r_e+ex[n], rf.levels[n]-e_at_r_e-ex[n]);
            }
            println!("  planted anharmonicity: ZPE_morse - ZPE_harm = {:+.3e} Ha ({:+.3} %)",
                ex[0]-0.5*omega, 100.0*(ex[0]-0.5*omega)/(0.5*omega));
        }
        Err(e) => println!("  {e}"),
    }

    if arg == "dvr" { return; }
    if arg == "pimd" { gauge_pimd(mu_h2, curv, r_e, e_at_r_e, omega, beta300); return; }

    gauge_pimd(mu_h2, curv, r_e, e_at_r_e, omega, beta300);

    // ---- the banked interpolant's departure from the model, in knots.
    println!("\n== banked table: Hermite error vs knot count ==");
    for &nk in &[512usize, 1024, 2048, 4096, 8192] {
        let t = holon_chem::table::generate_table(bmin, bmax, nk).unwrap();
        let (de, df) = t.hermite_error(4);
        let bp = BankedPes::from_table(t);
        let (rmin_i, vmin_i) = bp.minimum();
        println!("  n={nk:5}  max|dE| {de:.3e} Ha  max|dF| {df:.3e}  interp minimum R {rmin_i:.9} (d {:+.2e})  V {vmin_i:.12} (d {:+.2e})",
            rmin_i - r_e, vmin_i - e_at_r_e);
    }

    // ---- price of a potential call on each surface.
    println!("\n== price ==");
    for (name, f) in [("exact", 0usize), ("banked-4096", 1usize)] {
        let bp = BankedPes::h2(4096);
        let n = if f == 0 { 20_000 } else { 4_000_000 };
        let t0 = std::time::Instant::now();
        let mut acc = 0.0;
        for i in 0..n {
            let r = 1.0 + 0.6 * ((i % 997) as f64) / 997.0;
            let (v, d) = if f == 0 { ExactPes.eval(r) } else { bp.eval(r) };
            acc += v + d;
        }
        let el = t0.elapsed().as_secs_f64();
        println!("  {name:12} {:.0} ns/call   (acc {acc:.3})", el / n as f64 * 1e9);
    }
    let _ = M_E_PER_U;
}

fn gauge_pimd(mu: f64, curv: f64, r_e: f64, v0: f64, omega: f64, beta: f64) {
    // ---- plant 3: PIMD on the harmonic plant vs the EXACT P-bead ring energy.
    // dt ladder at fixed P: the integrator's own systematic, measured against arithmetic.
    println!("\n== plant 3a: PIMD dt ladder at P=64 vs exact E_P (harmonic) ==");
    println!("    dt      E_cv            err        exact E_P        (E_cv-E_P)     tau    s");
    for &dt in &[8.0f64, 4.0, 2.0, 1.0] {
        let steps = (600_000.0 * (4.0 / dt)) as u64;
        let cfg = PimdConfig { p: 64, temperature_k: 300.0, dt, gamma_centroid: omega,
            steps_equil: steps/10, steps_sample: steps, seed: 0xC1_0002 };
        let t0 = std::time::Instant::now();
        let rep = run_pimd_chains(mu, "harmonic", &cfg, 8, r_e,
            &|| Box::new(HarmonicPes { k: curv, r0: r_e, v0 }));
        let ex = v0 + harmonic_ring_energy(omega, beta, 64);
        println!("{dt:6.2}  {:.9} {:.2e}  {:.9}  {:+.3e}  {:6.1} {:5.1}",
            rep.e_virial, rep.e_virial_err, ex, rep.e_virial - ex, rep.tau_int, t0.elapsed().as_secs_f64());
    }

    println!("\n== plant 3: PIMD vs exact ring-polymer energy E_P (harmonic) ==");
    println!("   P     E_cv            err        E_prim          err        exact E_P        (E_cv-E_P)   tau  s");
    for &p in &[1usize, 8, 32, 64, 128, 256] {
        let steps: u64 = if p >= 128 { 300_000 } else { 600_000 };
        let cfg = PimdConfig { p, temperature_k: 300.0, dt: 4.0, gamma_centroid: omega,
            steps_equil: steps/10, steps_sample: steps, seed: 0xC1_0001 };
        let t0 = std::time::Instant::now();
        let rep = run_pimd_chains(mu, "harmonic", &cfg, 8, r_e,
            &|| Box::new(HarmonicPes { k: curv, r0: r_e, v0: v0 }));
        let ex = v0 + harmonic_ring_energy(omega, beta, p);
        println!("{p:6}  {:.9} {:.2e}  {:.9} {:.2e}  {:.9}  {:+.3e}  {:6.1} {:5.1}",
            rep.e_virial, rep.e_virial_err, rep.e_primitive, rep.e_primitive_err, ex,
            rep.e_virial - ex, rep.tau_int, t0.elapsed().as_secs_f64());
    }

}
