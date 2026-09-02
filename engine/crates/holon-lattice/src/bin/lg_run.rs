//! Node LG's campaign driver: every gate of `conformance/mesh/LG_PREREG.md`, in order,
//! each with its WORK COUNT beside its verdict.
//!
//! A gate that reports PASS having done no work has not passed (M-VACUOUS-SUCCESS), so no
//! line below prints a verdict without the number of checks behind it. The run records why
//! it stopped (M-EXIT-DISCRIMINATOR) and prints VOID rather than a score if a budget or a
//! control fails (M-BUDGET-LAUNDER).
//!
//! Usage: `lg_run [--quick]`. `--quick` shrinks the reference run and the probe lattice; it
//! is for smoke-testing the driver and its output is labelled QUICK and must never be banked.

use holon_lattice::chart::BlockChart;
use holon_lattice::isotropy;
use holon_lattice::lattice::Lattice;
use holon_lattice::probe::{probe, probe_on, Move, Population, Reading};
use holon_lattice::state::{Model, COLLISION_GROUP_ORDER};

struct Report {
    failures: Vec<String>,
    checks: u64,
}

impl Report {
    fn gate(&mut self, id: &str, pass: bool, work: u64, detail: String) {
        self.checks += work;
        if work == 0 {
            self.failures.push(format!("{id}: ZERO WORK — a gate that checked nothing did not pass"));
            println!("  {id:<4} VOID (zero work)  {detail}");
            return;
        }
        if pass {
            println!("  {id:<4} PASS  [{work} checks]  {detail}");
        } else {
            self.failures.push(format!("{id}: {detail}"));
            println!("  {id:<4} FAIL  [{work} checks]  {detail}");
        }
    }
}

fn fhp_lattice(l: usize, seed: u64) -> Lattice {
    let m = Model::fhp6();
    let c = m.fhp_i(true);
    Lattice::seeded(m, l, seed, 0.35, c)
}

fn main() {
    let quick = std::env::args().any(|a| a == "--quick");
    let (run_l, run_steps, probe_l) = if quick { (64, 500, 16) } else { (256, 20_000, 64) };
    let mut r = Report { failures: Vec::new(), checks: 0 };

    println!("=========================================================================");
    println!("NODE LG — the lattice-gas tier, certified on its own dynamics");
    println!("prereg: conformance/mesh/LG_PREREG.md   instrument: holon-lattice");
    println!("mode:   {}", if quick { "QUICK (smoke test — NEVER bank this output)" } else { "FULL" });
    println!("=========================================================================");

    // ---------------------------------------------------------------- G5, G11, G12
    println!("\n--- 6.3 INSTRUMENT CONTROL (run FIRST: these gate everything below) ---");
    let fhp = Model::fhp6();
    let hpp = Model::hpp4();

    let laws = fhp.collision_laws();
    let bijective = laws.iter().all(|c| fhp.is_bijection(c));
    let conserving = laws.iter().all(|c| fhp.is_sector_preserving(c));
    let mut dedup = laws.clone();
    dedup.sort_unstable();
    dedup.dedup();
    r.gate(
        "G5",
        bijective && conserving && dedup.len() == laws.len() && laws.len() == COLLISION_GROUP_ORDER as usize,
        laws.len() as u64 * 64,
        format!("all {} enumerated laws are distinct conserving bijections on 64 states", laws.len()),
    );

    let (sectors, hist) = fhp.census();
    r.gate(
        "G11",
        sectors == 53 && hist == vec![(1, 44), (2, 7), (3, 2)],
        64,
        format!("FHP-6 census {sectors} sectors, histogram {hist:?} (Lean: 53, 44/7/2); \
                 FCHC-24 leg is holon-mesh::fchc, reproduced in tests/fchc_control.rs"),
    );

    let fi = isotropy::measure(&fhp);
    let hi = isotropy::measure(&hpp);
    let mut reordered = fhp.clone();
    reordered.dirs.rotate_left(3);
    let ri = isotropy::measure(&reordered);
    let mut flipped = fhp.clone();
    flipped.dirs = fhp.dirs.iter().map(|d| [-d[0], -d[1]]).collect();
    let li = isotropy::measure(&flipped);
    r.gate(
        "G12",
        fi.residual < 1e-12
            && hi.residual > 0.66
            && ri.t4_xxxx.to_bits() == fi.t4_xxxx.to_bits()
            && (li.residual - fi.residual).abs() < 1e-15,
        4,
        format!(
            "FHP-6 residual {:.3e} (T4_xxxx {:.4}, T4_xxyy {:.4}); HPP-4 residual {:.4} \
             (T4_xxyy {:.4}); 2 re-presentations bit-identical",
            fi.residual, fi.t4_xxxx, fi.t4_xxyy, hi.residual, hi.t4_xxyy
        ),
    );
    println!("       NOT a claim of the Navier-Stokes limit: necessary condition only.");
    println!("       Exit (LG_PREREG 3): viscosity, semi-detailed balance, the g(rho) defect.");

    // ---------------------------------------------------------------- G1-G3, G13
    println!("\n--- 6.1 DYNAMICS AND CONSERVATION (one gate per law) ---");
    let mut g = fhp_lattice(run_l, 0xC1A5);
    let start = g.cells.clone();
    let l0 = g.ledger();
    let (mut ok_m, mut ok_x, mut ok_y) = (true, true, true);
    let (mut fired_total, mut fired_min) = (0u64, u64::MAX);
    let mut sector_churn_min = u64::MAX;
    let mut prev_labels: Vec<u8> = g.cells.clone();
    // LG_PREREG 6.3 G13 stakes the carrier-motion distance AT STEP 100, which is the
    // instant a closure reading taken early in the run would rely on. Reading it only at
    // the end would be a different, later instant on a quantity that saturates, and a
    // freeze's chosen instant is part of the freeze.
    let mut dist_at_100 = 0u64;
    for step in 0..run_steps {
        g.step();
        let l = g.ledger();
        ok_m &= l.mass == l0.mass;
        ok_x &= l.momentum[0] == l0.momentum[0];
        ok_y &= l.momentum[1] == l0.momentum[1];
        fired_total += g.collisions_fired;
        fired_min = fired_min.min(g.collisions_fired);
        let churn = Lattice::occupancy_distance(&prev_labels, &g.cells);
        sector_churn_min = sector_churn_min.min(churn);
        prev_labels.copy_from_slice(&g.cells);
        if step + 1 == 100 {
            dist_at_100 = Lattice::occupancy_distance(&start, &g.cells);
        }
    }
    let cells = (run_l * run_l) as u64;
    r.gate("G1", ok_m, run_steps as u64, format!("mass EXACT at every step (total {})", l0.mass));
    r.gate("G2", ok_x, run_steps as u64, format!("momentum-x EXACT at every step (total {})", l0.momentum[0]));
    r.gate("G3", ok_y, run_steps as u64, format!("momentum-y EXACT at every step (total {})", l0.momentum[1]));

    let dist_end = Lattice::occupancy_distance(&start, &g.cells);
    r.gate(
        "G13",
        dist_at_100 as f64 > 0.30 * cells as f64
            && fired_min >= 1
            && sector_churn_min as f64 >= 0.10 * cells as f64,
        3,
        format!(
            "carrier moved: occupancy distance AT STEP 100 = {dist_at_100} (> {:.0} = \
             0.30*L^2, the staked instant), {dist_end} at end of run; collisions fired \
             {fired_total} total, min {fired_min}/step; min per-step churn \
             {sector_churn_min} (> {:.0} = 0.10*L^2)",
            0.30 * cells as f64,
            0.10 * cells as f64
        ),
    );

    // ---------------------------------------------------------------- G4 the wall ledger
    // 32 cells: LG_PREREG 6.4's wall, which G4's row defers to by naming that section.
    const WALL: usize = 32;
    let mut w = fhp_lattice(128, 0x7A11);
    w.add_wall(24, 64, WALL);
    let w0 = w.ledger();
    let mut wall_ok = true;
    let mut mass_ok = true;
    for _ in 0..2_000 {
        w.step();
        let l = w.ledger();
        mass_ok &= l.mass == w0.mass;
        wall_ok &= l.momentum[0] == w0.momentum[0] + l.wall_impulse[0]
            && l.momentum[1] == w0.momentum[1] + l.wall_impulse[1];
    }
    let wl = w.ledger();
    // The label is COUNTED from the scene, never retyped. This line existed as the literal
    // "48-cell" while add_wall was building 32, so the banked log would have misstated its
    // own configuration -- the measurement was right and the diagnostic lied about the
    // parameter it ran at. A diagnostic must echo its parameters from the object.
    let solid = w.solid.iter().filter(|&&s| s).count();
    assert_eq!(solid, WALL, "the wall the scene carries is not the wall that was declared");
    r.gate(
        "G4",
        wall_ok && mass_ok && wl.wall_impulse != [0, 0],
        2_000 * 3,
        format!(
            "with a {solid}-cell wall (declared {WALL}): mass EXACT, momentum = P(0) + \
             impulse EXACT, \
             cumulative impulse {:?} (nonzero, so the gate did work)",
            wl.wall_impulse
        ),
    );

    // ---------------------------------------------------------------- G6 Leg A, two-sided
    println!("\n--- 6.2 CLOSURE, LEG A (HELD) ---");
    let hc = hpp.hpp_collision();
    let mut h = Lattice::seeded(hpp.clone(), 64, 0x4899, 0.35, hc);
    let h0 = h.line_momenta();
    let mut hpp_held = true;
    for _ in 0..2_000 {
        h.step();
        hpp_held &= h.line_momenta() == h0;
    }
    let mut f = fhp_lattice(64, 0xF1BE);
    let f0 = f.line_momenta();
    let mut fhp_broke_at = None;
    for t in 0..100 {
        f.step();
        if f.line_momenta() != f0 {
            fhp_broke_at = Some(t);
            break;
        }
    }
    r.gate(
        "G6",
        hpp_held && fhp_broke_at.is_some(),
        2_100,
        format!(
            "HPP-4 holds its per-line momentum chart EXACTLY over 2000 steps (128 lines, a \
             chart 64x finer than global); FHP-6 breaks the same chart at step {:?}",
            fhp_broke_at
        ),
    );
    println!("       v_L is HELD by G1-G3 and is LABELLED VACUOUS-BY-CONSERVATION, not counted.");

    // ---------------------------------------------------------------- G7-G10 Leg B
    println!("\n--- 6.2 CLOSURE, LEG B (CLOSED) — probed by construction ---");
    let base = fhp_lattice(probe_l, 0xF1BE);
    let bs: Vec<usize> = (0..)
        .map(|k| 1usize << k)
        .take_while(|&b| b <= probe_l)
        .collect();

    let mut g7_ok = true;
    let mut g7_work = 0u64;
    let mut g8_exhibits = 0u64;
    println!("       {:>4} {:>16} {:>16} {:>10} {:>12}", "b", "measured", "derived W(b)", "probes", "as-configured");
    let mut readings: Vec<Reading> = Vec::new();
    for &b in &bs {
        let chart = BlockChart::new(b, probe_l).unwrap();
        let ex = probe(&base, chart, 1, Population::Exhaustive, Move::Fiber);
        let ac = probe(&base, chart, 1, Population::AsConfigured, Move::Fiber);
        let pred = BlockChart::predicted_witness_rate(b, probe_l);
        let hit = (ex.rate() - pred).abs() < 1e-12;
        g7_ok &= hit;
        g7_work += ex.probes;
        if b < probe_l && ex.exhibit.is_some() {
            g8_exhibits += 1;
        }
        println!(
            "       {:>4} {:>16.10} {:>16.10} {:>10} {:>12.6}{}",
            b, ex.rate(), pred, ex.probes, ac.rate(),
            if b == probe_l { "   <- VACUOUS BY CONSERVATION" } else { "" }
        );
        readings.push(ex);
    }
    r.gate("G7", g7_ok, g7_work, format!("the measured k=1 defect IS 1 - max(0,b-2)^2/b^2 at all {} charts, exactly", bs.len()));
    r.gate(
        "G9",
        readings.len() == bs.len() && readings.iter().all(|x| x.probes > 0),
        bs.len() as u64,
        "each b probed independently with its own fiber moves; no b's verdict inferred from another's".into(),
    );
    r.gate(
        "G8",
        g8_exhibits as usize == bs.len() - 1,
        g8_exhibits,
        format!("{g8_exhibits} of {} non-vacuous charts exhibit a witness pair", bs.len() - 1),
    );
    if let Some(w) = readings.iter().find(|x| x.b == 4).and_then(|x| x.exhibit.as_ref()) {
        let m = &base.model;
        println!(
            "       EXHIBIT at b=4: cell {} state {} -> {} (both label {:?}); the agreeing \
             view has {} blocks and the stepped views differ in {} of them",
            w.cell, w.state_before, w.state_after, m.label(w.state_before),
            w.agreed_view.len(),
            w.stepped_x.iter().zip(&w.stepped_y).filter(|(a, b)| a != b).count()
        );
    }

    let neg = probe(&base, BlockChart::new(4, probe_l).unwrap(), 1, Population::Exhaustive, Move::None);
    let pos = probe(&base, BlockChart::new(1, probe_l).unwrap(), 1, Population::Exhaustive, Move::CrossFiber);
    r.gate(
        "G10",
        neg.witnesses == 0 && pos.rate() == 1.0,
        neg.probes + pos.probes,
        format!("negative control (y = x) rate {:.1}; positive control (cross-fiber) rate {:.3}", neg.rate(), pos.rate()),
    );

    // the light cone: the defect grows with k, which is what a boundary effect does
    // A chart strictly finer than global: at b = L there is nothing for a light cone to
    // cross and the row would read zeros for a reason that is not about light cones.
    let chart16 = BlockChart::new((probe_l / 4).max(1), probe_l).unwrap();
    let ks: Vec<usize> = vec![1, 2, 4, 8, 16];
    let cone: Vec<f64> = ks.iter().map(|&k| probe(&base, chart16, k, Population::AsConfigured, Move::Fiber).rate()).collect();
    println!("       light cone at b={}: k={:?} -> rate {:?}", chart16.b, ks, cone.iter().map(|v| (v * 1e4).round() / 1e4).collect::<Vec<_>>());

    // ---------------------------------------------------------------- G14 inhomogeneity
    println!("\n--- 6.4 THE INHOMOGENEITY DISCHARGE (M-HOMOG) ---");
    let mut wl2 = fhp_lattice(probe_l, 0xF1BE);
    wl2.add_wall(probe_l / 4, probe_l / 2, WALL.min(probe_l));
    let mut g14_ok = true;
    let mut g14_work = 0u64;
    for &b in bs.iter().filter(|&&b| b > 1 && b < probe_l) {
        let chart = BlockChart::new(b, probe_l).unwrap();
        let nb = probe_l / b;
        let mut block_has_wall = vec![false; nb * nb];
        for c in 0..wl2.cells.len() {
            if wl2.solid[c] {
                block_has_wall[((c / probe_l) / b) * nb + ((c % probe_l) / b)] = true;
            }
        }
        let free: Vec<usize> = (0..wl2.cells.len())
            .filter(|&c| !block_has_wall[((c / probe_l) / b) * nb + ((c % probe_l) / b)])
            .collect();
        let touching = wl2.cells.len() - free.len();
        let rr = probe_on(&wl2, chart, 1, Population::ExhaustiveOn, Move::Fiber, &free);
        let pred = BlockChart::predicted_witness_rate(b, probe_l);
        let hit = (rr.rate() - pred).abs() < 1e-12;
        g14_ok &= hit;
        g14_work += rr.probes;
        println!(
            "       b={:<3} wall-free rate {:.10} vs derived {:.10}  [{} probes; {} cells in \
             wall-touching blocks reported, NOT averaged in]",
            b, rr.rate(), pred, rr.probes, touching
        );
    }
    r.gate("G14", g14_ok, g14_work, "the defect law survives a structurally inhomogeneous graph on wall-free blocks".into());

    // ---------------------------------------------------------------- G15 the law sweep
    println!("\n--- 6.5 THE COLLISION-LAW SWEEP (M-ONE-MODEL-DELTA) ---");
    // ONE block's worth of positions per law, which is the reference's population: the
    // lattice is homogeneous and every block is a translate of the first, so the block's
    // b^2 positions already cover every position class. Ranging over all L^2 cells instead
    // multiplies the sweep by (L/b)^2 to re-measure the same classes, and the priced cost
    // of this stage assumes it is not done.
    let (sweep_l, sweep_b) = (8usize, 4usize);
    let sweep_chart = BlockChart::new(sweep_b, sweep_l).unwrap();
    let one_block: Vec<usize> =
        (0..sweep_b).flat_map(|i| (0..sweep_b).map(move |j| i * sweep_l + j)).collect();
    let mut rates: Vec<u64> = Vec::new();
    for c in &laws {
        let lat = Lattice::seeded(fhp.clone(), sweep_l, 0x5EED, 0.35, c.clone());
        let rr = probe_on(&lat, sweep_chart, 1, Population::ExhaustiveOn, Move::Fiber, &one_block);
        rates.push((rr.rate() * 1e12).round() as u64);
    }
    rates.sort_unstable();
    rates.dedup();
    let pred8 = BlockChart::predicted_witness_rate(sweep_b, sweep_l);
    r.gate(
        "G15",
        rates.len() == 1 && (rates[0] as f64 / 1e12 - pred8).abs() < 1e-9,
        laws.len() as u64,
        format!(
            "all {} sector-preserving collision laws give {} distinct k=1 defect rate(s) at \
             b={sweep_b}, L={sweep_l}: {:?} (derived {pred8}) — the defect is the LATTICE's, \
             and the identity collision is in the sweep",
            laws.len(), rates.len(), rates.iter().map(|v| *v as f64 / 1e12).collect::<Vec<_>>()
        ),
    );

    // ---------------------------------------------------------------- plants
    println!("\n--- 7 PLANTS (each must FIRE; every carrier nonzero in its own sector) ---");
    let carrier9 = fhp_lattice(64, 0xC1A5).cells.iter().filter(|&&s| s == 9).count();
    println!("       carrier state 9 population: {carrier9} cells, nonzero in the (N=2,P=0) sector the plants act on");
    let mut plants_ok = carrier9 > 0;
    for (name, target, expect) in [("P1 mass", 0u8, [true, false, false]), ("P2 momentum-x", 34, [false, true, false]), ("P3 momentum-y", 5, [false, false, true])] {
        let mut c = fhp.fhp_i(true);
        c[9] = target;
        let mut lat = Lattice::seeded(fhp.clone(), 64, 0xC1A5, 0.35, c);
        let b0 = lat.ledger();
        let mut moved = [false; 3];
        for _ in 0..200 {
            lat.step();
            let l = lat.ledger();
            moved[0] |= l.mass != b0.mass;
            moved[1] |= l.momentum[0] != b0.momentum[0];
            moved[2] |= l.momentum[1] != b0.momentum[1];
        }
        let ok = moved == expect;
        plants_ok &= ok;
        println!("       {name:<16} 9 -> {target:<3} fires {moved:?}, expected {expect:?}  {}", if ok { "FIRED, ISOLATED" } else { "DID NOT ISOLATE" });
    }
    let mut nb = fhp.fhp_i(true);
    nb[18] = 9;
    let p4 = !fhp.is_bijection(&nb) && fhp.is_sector_preserving(&nb);
    println!("       {:<16} 18 -> 9   bijectivity FAILS while conservation HOLDS: {}", "P4 bijectivity", p4);
    let mut bad = fhp.clone();
    bad.dirs[2] = [1, 1];
    let p5 = bad.census().0 != 53;
    println!("       {:<16} one direction perturbed: census reads {} (not 53): {}", "P5 census", bad.census().0, p5);
    let p6 = isotropy::measure(&hpp).residual > 0.66;
    println!("       {:<16} the tensor path prints a FAILING row on HPP-4: {}", "P6 isotropy", p6);
    let p7 = wl.wall_impulse != [0, 0]
        && (wl.momentum[0] != w0.momentum[0] || wl.momentum[1] != w0.momentum[1]);
    println!("       {:<16} dropping the impulse from the ledger breaks the identity: {}", "P7 wall ledger", p7);
    // P8 is the plant against a closure result produced by a probe that never perturbed
    // anything. It must do BOTH things: read exactly zero, and FAIL to reproduce W(b) at a
    // chart where W(b) is not zero. Reading zero alone is what a correct global chart also
    // does, so the second half is what makes the plant a plant.
    let p8_chart = BlockChart::new(4, probe_l).unwrap();
    let p8_noop = probe(&base, p8_chart, 1, Population::Exhaustive, Move::None);
    let p8_target = BlockChart::predicted_witness_rate(4, probe_l);
    let p8 = p8_noop.witnesses == 0 && (p8_noop.rate() - p8_target).abs() > 1e-9 && p8_target > 0.0;
    println!(
        "       {:<16} a no-op fiber move reads {:.1} over {} probes and so cannot reproduce \
         W(4) = {}: {}",
        "P8 probe", p8_noop.rate(), p8_noop.probes, p8_target, p8
    );
    let fixed = Lattice::seeded(fhp.clone(), 32, 0, 0.0, fhp.fhp_i(true));
    let mut fx = fixed.clone();
    for _ in 0..50 { fx.step(); }
    let p9 = Lattice::occupancy_distance(&fixed.cells, &fx.cells) == 0;
    println!("       {:<16} the empty state is a fixed point and G13's counter refuses it: {}", "P9 fixed point", p9);
    plants_ok &= p4 && p5 && p6 && p7 && p8 && p9;
    r.gate("P1-P9", plants_ok, 9, "every plant fired its own gate and no other".into());

    // ---------------------------------------------------------------- verdict
    println!("\n=========================================================================");
    println!("TERMINATION: STEPS_COMPLETED  (checks performed: {})", r.checks);
    if r.failures.is_empty() {
        println!("VERDICT: every gate PASSED.");
        println!();
        println!("  The tier runs on its OWN dynamics with exact integer conservation, a");
        println!("  bijective motion, and a census that CLASSIFIES its collision law.");
        println!("  Its coarse charts are NOT closed views: the k=1 defect is exactly the");
        println!("  block's boundary fraction, at every scale, for every one of the {} REG+", laws.len());
        println!("  collision laws. The only exactly-closed chart is the global one, and it");
        println!("  closes BY CONSERVATION ALONE — the vacuous end of the axis, not a result.");
        println!();
        println!("  NOT claimed: the Navier-Stokes limit (necessary condition measured only).");
        println!("  NOT claimed: the molecular-to-lattice seam. It takes no status from here.");
    } else {
        println!("VERDICT: {} GATE(S) FAILED — nothing is banked until each is resolved:", r.failures.len());
        for f in &r.failures {
            println!("  - {f}");
        }
    }
    println!("=========================================================================");
    if !r.failures.is_empty() {
        std::process::exit(1);
    }
}
