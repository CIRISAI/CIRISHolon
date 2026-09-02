//! THE NEAR/FAR RE-SPLIT — G-SPLIT and G-ROUTE.
//!
//! ```text
//!   cargo run --release --example resplit -- [--arm=split|route|both]
//! ```
//!
//! Frozen design: `conformance/water_observatory/RESPLIT_PREREG.md`, ADMITTED and committed
//! before this file existed. Where this file disagrees with that one, this file is the defect.
//!
//! G-SPLIT is run first and alone because it can REFUTE the design: if no candidate handover
//! radius reproduces the banked curve over the band the far sector would take over, there is
//! nothing to build and branch (c) is the answer.

use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{generate_pair_table, PairTable};
use holon_render::bank::Host;
use holon_render::longrange::CurveTail;
use holon_render::sim::{Boundary, Dims, Sim};
use holon_render::{load_pair_table, TABLE_OK};

/// The freeze's candidate handover radii, bohr. Unchanged from `RESPLIT_PREREG.md` §2.
const LADDER: [f64; 9] = [10.24, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 18.0, 20.0];
/// The band's far end: the banked O–O curve's own support.
const BAND_END: f64 = 20.0;
/// Sampled radii across the band, per candidate.
const SAMPLES: usize = 200;
/// G-SPLIT's admissibility budget, hartree — one tenth of carrier-v2's `PAIR_FLOOR`.
const GAP_BUDGET: f64 = 1.0e-7;
/// The radius the freeze derives from carrier-v2's N = 800 bar.
const TARGET_R_MAX: f64 = 10.59;
/// carrier-v2's constants, quoted so the derivation in the log is checkable in place.
const RHO_ATOMS: f64 = 0.014_860;
const CURVE_KNOTS: usize = 96;
const SWITCH_WIDTH: f64 = 2.0;

/// `N = rho * (3 * (r_max + W))^3`, the freeze's §0 arithmetic.
fn derived_threshold(r_max: f64) -> f64 {
    RHO_ATOMS * (3.0 * (r_max + SWITCH_WIDTH)).powi(3)
}

/// A `CurveTail` over the knots at or below `r_s`, with the exponential index recomputed at
/// the NEW last knot — the PREFIX form of §1, in memory. Every retained knot is the banked
/// curve's own, so this measures the prefix design rather than a re-sampled stand-in.
fn truncated_tail(t: &holon_render::table::PotentialTable, r_s: f64, meta: (&'static str, u64, f64)) -> Option<CurveTail> {
    let mut r = Vec::new();
    let mut u = Vec::new();
    let mut last = 0usize;
    for k in 0..t.knots() {
        if t.knot_r(k) <= r_s + 1e-12 {
            r.push(t.knot_r(k));
            u.push(t.knot_u(k));
            last = k;
        }
    }
    if r.len() < 4 {
        return None;
    }
    let (an, dn) = (t.knot_u(last), t.knot_d(last));
    let hi_b = if an.abs() > 0.0 { -dn / an } else { 0.0 };
    Some(CurveTail { r, u, hi_b, solver_exit: meta.0, solver_budget_iterations: meta.1, uncertainty_hartree: meta.2 })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arm = args
        .iter()
        .find_map(|a| a.strip_prefix("--arm="))
        .unwrap_or("both")
        .to_string();
    println!("# THE NEAR/FAR RE-SPLIT — freeze conformance/water_observatory/RESPLIT_PREREG.md");
    println!("# instrument = engine/crates/holon-render/examples/resplit.rs  arm = {arm}");

    // The curves, through the same door every other instrument uses.
    let mut sim = Box::new(Sim::empty());
    sim.dims = Dims::Three;
    sim.boundary = Boundary::Walls;
    sim.width = 400.0;
    sim.height = 400.0;
    sim.depth = 400.0;
    let mut tabs: Vec<PairTable> = Vec::new();
    let mut oo_meta = ("Unrecorded", 0u64, f64::NAN);
    for (a, b) in [(HYDROGEN, HYDROGEN), (OXYGEN, HYDROGEN), (OXYGEN, OXYGEN)] {
        let pt = generate_pair_table(a, b, CURVE_KNOTS);
        assert_eq!(load_pair_table(&mut sim, &pt, Host::Native), TABLE_OK);
        let exit: &'static str = match format!("{:?}", pt.meta.exit).as_str() {
            "Converged" => "Converged",
            "IterationCap" => "IterationCap",
            "Stagnated" => "Stagnated",
            _ => "Other",
        };
        // G-SOLVE: the disclosure travels with every constant derived from this curve.
        println!(
            "# G-SOLVE {}-{}: route {:?} exit {exit} n_det {} n_basis {} solver_budget {} \
             worst_residual {:.3e}",
            a.symbol, b.symbol, pt.meta.route, pt.meta.n_det, pt.meta.n_basis,
            pt.meta.solver_budget, pt.meta.worst_residual
        );
        if a == OXYGEN && b == OXYGEN {
            oo_meta = (exit, pt.meta.solver_budget as u64, pt.meta.worst_residual);
        }
        tabs.push(pt);
    }
    let slot = sim.bank.slot_of_z(8, 8).expect("O-O registered");
    let full = sim.bank.table_slot(slot);
    println!(
        "# banked O-O curve: {} knots, r_min {:.4} to r_max {:.4} bohr, |u(r_max)| {:.6e} Ha",
        full.knots(), full.r_min(), full.r_max(), full.u(full.r_max()).abs()
    );
    println!(
        "# derived thresholds (freeze §0): banked r_max {:.2} -> N {:.0}; target r_max <= \
         {TARGET_R_MAX} -> N <= {:.0}",
        full.r_max(), derived_threshold(full.r_max()), derived_threshold(TARGET_R_MAX)
    );

    if arm == "split" || arm == "both" {
        gate_split(full, oo_meta);
    }
    if arm == "route" || arm == "both" {
        gate_route(&tabs);
    }
}

/// G-SPLIT — the gate that can refute the design.
fn gate_split(full: &holon_render::table::PotentialTable, meta: (&'static str, u64, f64)) {
    println!("# --- G-SPLIT: does the tail model reproduce the banked curve over the band? ---");
    println!(
        "COLUMNS GSPLIT r_s p_fit fit_residual exp_index band n_samples worst_gap at_r \
         derived_N admissible"
    );
    let mut best: Option<(f64, f64)> = None;
    let mut probed = 0usize;
    for &r_s in LADDER.iter() {
        let Some(tail) = truncated_tail(full, r_s, meta) else {
            println!("GSPLIT {r_s:.2} REFUSED — fewer than 4 knots at or below this radius");
            continue;
        };
        let fit = tail.fit();
        // The tail model the far sector would build at this handover: exponent refitted on
        // the SHORTENED curve's own last knots, constant matched at the seam. Both change
        // with `r_s`, which is the whole reason this is a sweep and not one reading.
        let c_p = -tail.u_at(r_s) * r_s.powf(fit.p_fit);
        let (mut worst, mut at_r) = (0.0f64, 0.0f64);
        let mut samples = 0usize;
        for k in 0..SAMPLES {
            let r = r_s + (BAND_END - r_s) * (k as f64 + 1.0) / SAMPLES as f64;
            if r > BAND_END {
                continue;
            }
            let model = -c_p * r.powf(-fit.p_fit);
            let table = full.u(r);
            let gap = (model - table).abs();
            if gap > worst {
                worst = gap;
                at_r = r;
            }
            samples += 1;
        }
        probed += 1;
        let admissible = samples > 0 && worst < GAP_BUDGET;
        if admissible && best.is_none() {
            best = Some((r_s, worst));
        }
        println!(
            "GSPLIT {r_s:.2} {:.4} {:.4e} {:.4} {:?} {samples} {worst:.6e} {at_r:.3} {:.0} {admissible}",
            fit.p_fit, fit.residual, fit.exp_index, fit.band, derived_threshold(r_s)
        );
    }
    println!("# G-WORK (split): {probed} candidate radii probed, {SAMPLES} samples each");
    match best {
        Some((r_s, gap)) if r_s <= TARGET_R_MAX => println!(
            "# GATE G-SPLIT: BRANCH (a) — smallest admissible handover {r_s:.2} bohr \
             (worst gap {gap:.6e} < {GAP_BUDGET:.0e}), at or under the {TARGET_R_MAX} target. \
             Derived threshold {:.0}.",
            derived_threshold(r_s)
        ),
        Some((r_s, gap)) => println!(
            "# GATE G-SPLIT: BRANCH (b) PARTIAL — smallest admissible handover is {r_s:.2} \
             bohr (worst gap {gap:.6e}), ABOVE the {TARGET_R_MAX} target. The table can be \
             shortened but not far enough for the N = 800 bar with this far model. \
             Achievable threshold {:.0}.",
            derived_threshold(r_s)
        ),
        None => println!(
            "# GATE G-SPLIT: BRANCH (c) REFUTED — NO candidate radius reproduces the banked \
             curve to {GAP_BUDGET:.0e} Ha over the band it would hand over. The power-law \
             tail cannot carry any of it, and the near/far split as built cannot buy the \
             route. The near table is NOT banked."
        ),
    }
    // P-GAP, against G-SPLIT: a 1% exponent error must move the worst gap.
    if let Some(tail) = truncated_tail(full, LADDER[0], meta) {
        let fit = tail.fit();
        let clean = band_gap(full, LADDER[0], fit.p_fit, &tail);
        let planted = band_gap(full, LADDER[0], fit.p_fit * 1.01, &tail);
        println!(
            "P-GAP exponent x1.01 at r_s {:.2} | carrier |gap(planted) - gap(clean)| = \
             {:.6e} Ha (sector: the tail model) | clean {clean:.6e} planted {planted:.6e} | \
             verdict {}",
            LADDER[0],
            (planted - clean).abs(),
            if (planted - clean).abs() == 0.0 {
                "REFUSED — carrier reads 0.0"
            } else {
                "FIRED"
            }
        );
    }
}

fn band_gap(full: &holon_render::table::PotentialTable, r_s: f64, p: f64, tail: &CurveTail) -> f64 {
    let c_p = -tail.u_at(r_s) * r_s.powf(p);
    let mut worst = 0.0f64;
    for k in 0..SAMPLES {
        let r = r_s + (BAND_END - r_s) * (k as f64 + 1.0) / SAMPLES as f64;
        worst = worst.max((-c_p * r.powf(-p) - full.u(r)).abs());
    }
    worst
}

/// G-ROUTE — the deliverable. Bisect for the smallest N at which the cell route engages.
fn gate_route(tabs: &[PairTable]) {
    println!("# --- G-ROUTE: the measured threshold, at the banked radius ---");
    println!("COLUMNS GROUTE n edge_bohr cutoff cells_per_axis route");
    let probe = |n: usize| -> (bool, f64, f64, [usize; 3]) {
        let edge = (n as f64 / RHO_ATOMS).cbrt();
        let mut s = Box::new(Sim::empty());
        s.dims = Dims::Three;
        s.boundary = Boundary::Walls;
        s.width = edge;
        s.height = edge;
        s.depth = edge;
        for pt in tabs {
            assert_eq!(load_pair_table(&mut s, pt, Host::Native), TABLE_OK);
        }
        s.reset(n);
        // A CUBIC lattice filling the box, so the atoms' bounding box — which is what the
        // decomposition measures in a walled box — actually spans it. A placement that
        // clusters would report a threshold about the placement (M-VACUOUS-SUCCESS).
        let side = (n as f64).cbrt().ceil() as usize;
        for i in 0..n {
            let (x, y, z) = (i % side, (i / side) % side, i / (side * side));
            s.atoms[i].x = (x as f64 + 0.5) / side as f64 * edge;
            s.atoms[i].y = (y as f64 + 0.5) / side as f64 * edge;
            s.atoms[i].z = (z as f64 + 0.5) / side as f64 * edge;
            s.atoms[i].species = if i % 3 == 0 { OXYGEN } else { HYDROGEN };
        }
        assert!(s.sync_species());
        let ok = s.set_pair_cutoff(1.0e-6);
        s.recompute();
        let cut = s.list_cutoff();
        (
            ok && s.route() == holon_render::cells::Route::Cells,
            edge,
            cut,
            // The cell arithmetic the freeze asks to be printed at every probe. `cells`
            // is private, so this is the same division `CellList::rebuild` performs,
            // recomputed here from public numbers rather than reached for.
            {
                let e = edge;
                let c = cut.max(1.0e-30);
                let n = (e / c).floor().max(1.0) as usize;
                [n, n, n]
            },
        )
    };
    // Bisect between a size that does not engage and one that does.
    let (mut lo, mut hi) = (64usize, 64usize);
    while hi < 200_000 {
        let (engaged, edge, cut, nc) = probe(hi);
        println!("GROUTE {hi} {edge:.2} {cut:.4} {nc:?} {}", if engaged { "Cells" } else { "Complete" });
        if engaged {
            break;
        }
        lo = hi;
        hi *= 2;
    }
    if hi >= 200_000 {
        println!("# GATE G-ROUTE: VOID (V5) — the route never engaged up to N = {hi}");
        return;
    }
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if probe(mid).0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let (_, edge, cut, nc) = probe(hi);
    let derived = derived_threshold(cut - SWITCH_WIDTH);
    println!(
        "# GATE G-ROUTE: measured threshold N = {hi} at edge {edge:.2} bohr, cutoff \
         {cut:.4}, cells {nc:?}. Derived expectation {derived:.0} — ratio {:.3}.",
        hi as f64 / derived
    );
    // P-ROUTE: forcing the complete route must make the search find nothing.
    let edge = (hi as f64 / RHO_ATOMS).cbrt();
    let mut s = Box::new(Sim::empty());
    s.dims = Dims::Three;
    s.boundary = Boundary::Walls;
    s.width = edge;
    s.height = edge;
    s.depth = edge;
    for pt in tabs {
        assert_eq!(load_pair_table(&mut s, pt, Host::Native), TABLE_OK);
    }
    s.reset(hi);
    s.set_pair_cutoff(1.0e-6);
    s.force_complete_route();
    println!(
        "P-ROUTE forced complete at the measured threshold | carrier = the route itself | \
         route {:?} | verdict {}",
        s.route(),
        if s.route() == holon_render::cells::Route::Cells {
            "REFUSED — the policy did not take"
        } else {
            "FIRED — the search cannot report a threshold on a scene that cannot take the route"
        }
    );
}
