//! Where are (O,H,H,H)'s corners, and are any of them BORROWED?
//!
//! The seam law is per table: a grid does not freeze until its own reactive channels
//! have been scanned, because a cubic interpolant across a state crossing has an error
//! floor set by the jump and uniform refinement cannot beat it. This is that scan for the
//! four-body surface, and it carries one question the three-body scans did not have to
//! ask.
//!
//! # The borrowed corner
//!
//! `dE4 = E_FCI(OH3) - E_MBE3(OH3)`, and `E_MBE3` is not a smooth background: it contains
//! three evaluations of the (O,H,H) surface, one of the (H,H,H) surface, and six pair
//! curves. The (O,H,H) surface has two located state crossings of its own (theta ~ 174.9
//! deg and theta ~ 36 deg, `SATURATION3_RESULTS.md`). Wherever a subterm has a corner and
//! the four-centre FCI does not, the DIFFERENCE has one — so the four-body surface can
//! inherit a seam from a table it merely subtracts. A scan that only looked for crossings
//! of the OH3 ground state would walk straight past those.
//!
//! Every slice therefore reports the divided differences of BOTH `E_FCI` and `dE4`, and
//! the pair of readings is the discriminator:
//!
//! | `E_FCI` kinks | `dE4` kinks | reading |
//! |---|---|---|
//! | yes | yes | an OH3 state crossing; the corner is the four-body surface's own |
//! | no  | yes | INHERITED from a subterm; the corner is real for THIS table and a grid line belongs on it, but it is the (O,H,H) or (H,H,H) table's physics |
//! | yes | no  | the crossing is common to FCI and MBE3 and cancels in the difference |
//! | no  | no  | smooth here |
//!
//! # The warm/cold discriminator
//!
//! A kink in a computed ground state has two readings that call for opposite responses: a
//! real crossing of two states (the ground state is the LOWER ENVELOPE, no interpolant
//! removes the corner, place a grid line on it), or Davidson converging onto the upper
//! branch on one side of a near-degeneracy (the corner is ours and the fix is the solver).
//! The discriminator is a warm start across the corner, and it rests on the variational
//! bound rather than on any residual: both solves bound the same Hamiltonian from above,
//! so if a warm start finds a LOWER energy at the same geometry, the cold solve was on
//! the wrong root and nothing else needs to be argued. A residual cannot do this job —
//! `s3_variational_guard` measured a wrong-eigenvector solve at residual 5.98e-11 against
//! the correct solve's 5.24e-11, with the identical exit reason.
//!
//! The walk carries the BETTER of the two vectors forward, so a lower branch found
//! anywhere is transported to every later geometry on the slice. That is what "from
//! either side" buys with one forward pass; it is not a second reversed sweep, and this
//! scan does not claim to be one.
//!
//! Run: `cargo run --release -p holon-chem --example ohhh_seam_scan [-- n_points]`

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::fci::{
    ci_ints, davidson_eigh_from, Order, DAVIDSON_MAX_ITER, DAVIDSON_REQUESTED_TOLERANCE,
};
use holon_chem::pair::geometry_problem;
use holon_chem::quaternary::de4_ohhh_fci;
use holon_chem::trimer::{self, TrimerTable};
use holon_chem::water::{self, WaterTable};
use std::io::Write;
use std::sync::atomic::Ordering;
use std::time::Instant;

const PROGRESS_PATH: &str = "output/ohhh_seam_scan_progress.log";

/// A geometry is only ever built from three O-H distances and the three H-O-H cosines,
/// because those are the coordinates the table will be built on and a seam has to be
/// located in the coordinate a grid line could be placed on.
fn geom_from(r: [f64; 3], u: [f64; 3]) -> Option<[[f64; 3]; 4]> {
    let (u12, u23, u31) = (u[0], u[1], u[2]);
    // The Gram determinant and the out-of-plane component are each exactly zero on the
    // planar configurations -- the equilateral C3v point among them -- and a bare
    // `>= 0.0` refuses them on rounding alone: 1 - 0.25 - 0.75 evaluates to -2.2e-16
    // once b has been through a divide. A degenerate geometry is not an unrealisable
    // one, so the fence is a tolerance and the value is clamped, not rejected.
    const EMBED_TOL: f64 = -1e-12;
    let g = 1.0 + 2.0 * u12 * u23 * u31 - u12 * u12 - u23 * u23 - u31 * u31;
    if !(g >= EMBED_TOL) {
        return None;
    }
    let s12 = (1.0 - u12 * u12).max(0.0).sqrt();
    if s12 < 1e-14 {
        return None;
    }
    let a = u31;
    let b = (u23 - u12 * u31) / s12;
    let c2raw = 1.0 - a * a - b * b;
    if !(c2raw >= EMBED_TOL) {
        return None;
    }
    let c2 = c2raw.max(0.0);
    Some([
        [0.0, 0.0, 0.0],
        [r[0], 0.0, 0.0],
        [r[1] * u12, r[1] * s12, 0.0],
        [r[2] * a, r[2] * b, r[2] * c2.sqrt()],
    ])
}

fn to_d2(c: &[[f64; 3]; 4]) -> Vec<[D2; 3]> {
    c.iter()
        .map(|p| [D2::c(p[0]), D2::c(p[1]), D2::c(p[2])])
        .collect()
}

/// One four-centre FCI solve. Returns the TOTAL energy, the converged vector, the
/// iteration count and the residual.
fn solve_fci(c: &[[f64; 3]; 4], start: Option<&[f64]>) -> (f64, Vec<f64>, usize, f64) {
    let (space, mo, nuc) = geometry_problem(
        &[OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN],
        to_d2(c),
    );
    let ci0 = ci_ints(&mo, Order::Value);
    let diag = space.diagonal(&ci0);
    let (e, v, iters, resid, _exit) = davidson_eigh_from(
        &space,
        &ci0,
        &diag,
        DAVIDSON_REQUESTED_TOLERANCE,
        DAVIDSON_MAX_ITER.load(Ordering::Relaxed),
        start,
    );
    (e + nuc.v, v, iters, resid)
}

fn hh(r_i: f64, r_j: f64, u: f64) -> f64 {
    (r_i * r_i + r_j * r_j - 2.0 * r_i * r_j * u).max(0.0).sqrt()
}

/// Third divided difference on a uniform grid, scaled to be a curvature-of-slope: a
/// smooth function gives O(h), a slope discontinuity of size J gives J/h^2 and so BLOWS
/// UP as the slice refines. That divergence, not the raw size, is the corner's signature.
fn dd3(v: &[f64], i: usize, h: f64) -> Option<f64> {
    if i < 1 || i + 2 >= v.len() {
        return None;
    }
    Some((v[i + 2] - 3.0 * v[i + 1] + 3.0 * v[i] - v[i - 1]) / (h * h * h))
}

struct Slice {
    id: usize,
    name: &'static str,
    /// What physically opens along this coordinate, in one line, for the seam record.
    channel: &'static str,
    /// The scan coordinate's name and range.
    axis: &'static str,
    lo: f64,
    hi: f64,
    /// Build (r, u) from the scan parameter.
    build: fn(f64) -> ([f64; 3], [f64; 3]),
}

fn slices() -> Vec<Slice> {
    vec![
        // ---- 1. H2 ELIMINATION: OH3 -> OH + H2. H2 and H3 close to the H2 bond length
        // while both stretch away from O. This is the four-body surface's own reactive
        // channel and it has no three-body analogue.
        Slice {
            id: 1,
            name: "H2 elimination (OH3 -> OH + H2)",
            channel: "H2-H3 closing to the H2 bond length while both leave O",
            axis: "s (reaction coordinate)",
            lo: 0.0,
            hi: 1.0,
            build: |s| {
                // H1 stays water-like; H2,H3 stretch 1.94 -> 4.2 while their mutual
                // distance closes 3.0 -> 1.40 (H2 equilibrium in this model).
                let r_far = 1.9435740105 + s * (4.2 - 1.9435740105);
                let d_hh = 3.0 + s * (1.40 - 3.0);
                // u23 from the law of cosines on the isoceles pair.
                let u23 = ((2.0 * r_far * r_far - d_hh * d_hh) / (2.0 * r_far * r_far))
                    .clamp(-1.0, 1.0);
                // Keep H1 roughly tetrahedral to both.
                let u12 = -0.33f64;
                let u31 = -0.33f64;
                ([1.9435740105, r_far, r_far], [u12, u23, u31])
            },
        },
        // ---- 2. THE INHERITED (O,H,H) SEAM AT theta ~ 174.9 deg. H1-O-H2 swept through
        // the located crossing with H3 a spectator. If dE4 kinks here and E_FCI does not,
        // the corner is borrowed from the water table this surface subtracts.
        Slice {
            id: 2,
            name: "inherited (O,H,H) near-collinear seam (theta ~ 174.9 deg)",
            channel: "H1-O-H2 through the located (O,H,H) state crossing, H3 spectator",
            axis: "theta_12 (deg)",
            lo: 168.0,
            hi: 180.0,
            build: |th_deg| {
                let u12 = (th_deg * std::f64::consts::PI / 180.0).cos();
                // H3 parked ON THE NORMAL to the H1-O-H2 plane. Near-antipodal H1 and H2
                // force u31 ~ -u23, so a fixed pair of equal negative cosines is not a
                // geometry at any angle in this range; the normal is consistent for every
                // u12 (u23 = u31 = 0) and keeps H3 a spectator, which is the point.
                ([1.766, 2.576, 4.5], [u12, 0.0, 0.0])
            },
        },
        // ---- 3. THE INHERITED (O,H,H) CLOSED-ANGLE SEAM AT theta ~ 36 deg.
        Slice {
            id: 3,
            name: "inherited (O,H,H) closed-angle seam (theta ~ 36 deg)",
            channel: "H1-O-H2 through the second located (O,H,H) crossing",
            axis: "theta_12 (deg)",
            lo: 28.0,
            hi: 46.0,
            build: |th_deg| {
                let u12 = (th_deg * std::f64::consts::PI / 180.0).cos();
                ([2.621, 2.703, 4.5], [u12, -0.25, -0.25])
            },
        },
        // ---- 4. C3v SYMMETRIC APPROACH. All three H equivalent, walking in together:
        // the most symmetric compact geometry, where near-degeneracies are most likely
        // and where a wrong root would be least visible.
        Slice {
            id: 4,
            name: "C3v symmetric approach",
            channel: "three equivalent H walking in together on the C3 axis",
            axis: "R_OH (bohr)",
            lo: 1.3,
            hi: 4.5,
            build: |r| ([r, r, r], [-0.5, -0.5, -0.5]),
        },
        // ---- 5. THE INHERITED (H,H,H) SEAM. Three H closing into an H3 triangle while O
        // recedes: this is the trimer table's own domain arriving inside the four-body
        // term, and the trimer surface is what MBE3 subtracts there.
        Slice {
            id: 5,
            name: "inherited (H,H,H) channel (H3 forms, O recedes)",
            channel: "three H closing to an equilateral H3 while O withdraws",
            axis: "s (reaction coordinate)",
            lo: 0.0,
            hi: 1.0,
            build: |s| {
                let r_o = 2.2 + s * (5.6 - 2.2);
                // Equilateral H3 of side d: the H-O-H cosines follow from the isoceles law.
                let d = 3.4 + s * (1.7 - 3.4);
                let u = ((2.0 * r_o * r_o - d * d) / (2.0 * r_o * r_o)).clamp(-1.0, 1.0);
                ([r_o, r_o, r_o], [u, u, u])
            },
        },
        // ---- 6. THE CLOSED-ANGLE FENCE the (O,H,H) table carries (C_LO = 0.05, i.e.
        // u -> 1). A fence in a subterm is a place the subterm STOPS being the surface;
        // the four-body term inherits whatever the fence does.
        Slice {
            id: 6,
            name: "closed-angle fence approach (u_12 -> 1)",
            channel: "H1 and H2 collapsing onto one direction; the (O,H,H) table's U_FENCE",
            axis: "u_12",
            lo: 0.90,
            hi: 0.9975,
            build: |u12| ([2.0, 2.4, 4.0], [u12, -0.2, -0.2]),
        },
    ]
}

fn scan_slice(
    sl: &Slice,
    n: usize,
    w: &WaterTable,
    t: &TrimerTable,
    log: &mut std::fs::File,
) -> (f64, f64, f64, usize) {
    println!("\n================================================================================");
    println!("SLICE {}: {}", sl.id, sl.name);
    println!("  channel: {}", sl.channel);
    println!("  axis:    {} in [{}, {}], {} points", sl.axis, sl.lo, sl.hi, n);
    println!("================================================================================");
    println!(
        "  {:>10} {:>9} {:>9} {:>9} {:>16} {:>14} {:>12} {:>12} {:>11}",
        sl.axis, "R1", "R2", "R3", "E_FCI (Ha)", "dE4 (Ha)", "d3[E_FCI]", "d3[dE4]", "warm-cold"
    );

    let h = (sl.hi - sl.lo) / (n - 1) as f64;
    let mut e_fci = Vec::with_capacity(n);
    let mut de4 = Vec::with_capacity(n);
    let mut params = Vec::with_capacity(n);
    let mut geoms = Vec::with_capacity(n);
    let mut warm_diffs: Vec<f64> = Vec::with_capacity(n);
    let mut skipped = 0usize;

    let t0 = Instant::now();
    let mut carrier: Option<Vec<f64>> = None;

    for i in 0..n {
        let p = sl.lo + h * i as f64;
        let (r, u) = (sl.build)(p);
        let g = match geom_from(r, u) {
            Some(g) => g,
            None => {
                skipped += 1;
                continue;
            }
        };

        let (e_cold, v_cold, _it, _rs) = solve_fci(&g, None);
        let (e_warm, v_warm) = match carrier.as_ref() {
            Some(c) => {
                let (ew, vw, _, _) = solve_fci(&g, Some(c));
                (ew, vw)
            }
            None => (e_cold, v_cold.clone()),
        };
        // Carry the BETTER of the two forward, so the walk never loses a branch it found.
        carrier = Some(if e_warm <= e_cold { v_warm } else { v_cold });
        let d_warm = e_warm - e_cold;
        warm_diffs.push(d_warm);

        let d = de4_ohhh_fci(&g, w, t);
        e_fci.push(e_cold);
        de4.push(d);
        params.push(p);
        geoms.push((r, u));

        let _ = writeln!(
            log,
            "POINT: slice={} i={} p={:.9} r1={:.9} r2={:.9} r3={:.9} u12={:.9} u23={:.9} u31={:.9} \
             d12={:.9} d23={:.9} d31={:.9} e_fci={:.16e} de4={:.16e} d_warm={:.16e}",
            sl.id, i, p, r[0], r[1], r[2], u[0], u[1], u[2],
            hh(r[0], r[1], u[0]), hh(r[1], r[2], u[1]), hh(r[2], r[0], u[2]),
            e_cold, d, d_warm
        );
        let _ = log.flush();
    }

    let m = e_fci.len();
    let mut max_d3_fci = 0.0f64;
    let mut max_d3_de4 = 0.0f64;
    let mut at_de4 = f64::NAN;
    for i in 0..m {
        let a = dd3(&e_fci, i, h);
        let b = dd3(&de4, i, h);
        if let Some(x) = a {
            if x.abs() > max_d3_fci {
                max_d3_fci = x.abs();
            }
        }
        if let Some(x) = b {
            if x.abs() > max_d3_de4 {
                max_d3_de4 = x.abs();
                at_de4 = params[i];
            }
        }
        if i % 2 == 0 || i == m - 1 {
            let (r, _u) = geoms[i];
            println!(
                "  {:>10.5} {:>9.4} {:>9.4} {:>9.4} {:>16.9} {:>14.9} {:>12.3e} {:>12.3e} {:>11.2e}",
                params[i], r[0], r[1], r[2], e_fci[i], de4[i],
                a.unwrap_or(f64::NAN), b.unwrap_or(f64::NAN),
                warm_diffs.get(i).copied().unwrap_or(f64::NAN)
            );
        }
    }

    let min_warm = warm_diffs.iter().copied().fold(f64::INFINITY, f64::min);
    println!("  --");
    println!("  points solved {} (skipped, not a geometry: {})", m, skipped);
    println!("  wall {:.1} s", t0.elapsed().as_secs_f64());
    println!("  max |d3[E_FCI]| {:.4e}", max_d3_fci);
    println!("  max |d3[dE4]|   {:.4e}   at {} = {:.5}", max_d3_de4, sl.axis, at_de4);
    println!("  min (E_warm - E_cold) {:.4e} Ha", min_warm);
    if min_warm < -1e-9 {
        println!("  >>> BRANCH CROSSING: a warm start found a LOWER energy than the cold solve.");
        println!("      The cold answers on this slice are on the upper branch somewhere; the");
        println!("      corner is the SOLVER's, and the fix is the solver, not the grid.");
    } else {
        println!("  >>> VARIATIONALLY STABLE: no warm start beat its cold solve anywhere on this");
        println!("      slice. The cold answers are the lower envelope, so any corner here");
        println!("      belongs to the surface and a grid line has to go on it.");
    }
    // The borrowed-corner reading.
    let ratio = if max_d3_fci > 0.0 { max_d3_de4 / max_d3_fci } else { f64::INFINITY };
    println!("  d3 ratio dE4/E_FCI {:.2}", ratio);
    if ratio > 3.0 {
        println!("  >>> BORROWED CORNER: dE4's third difference is {:.1}x E_FCI's. The kink is", ratio);
        println!("      NOT in the four-centre ground state; it enters through a subterm MBE3");
        println!("      subtracts. Real for this table, and a grid line still belongs on it.");
    }
    (max_d3_fci, max_d3_de4, min_warm, m)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(25);
    let only: Option<Vec<usize>> = args.get(2).map(|s| {
        s.split(',').filter_map(|x| x.trim().parse().ok()).collect()
    });

    println!("=== (O,H,H,H) SEAM SCAN ===");
    println!(
        "host loadavg      {}",
        std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim()
    );
    println!("points per slice  {}", n);

    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/s2/s2_water_table.txt"
    ))
    .expect("the committed (O,H,H) table");
    let w = water::from_text(&src).expect("water table parses");
    let t = trimer::generate().expect("the H3 table");

    let _ = std::fs::create_dir_all("output");
    let mut log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(PROGRESS_PATH)
        .expect("progress log");

    let sls = slices();
    let mut summary = Vec::new();
    for sl in sls.iter() {
        if let Some(ids) = only.as_ref() {
            if !ids.contains(&sl.id) {
                continue;
            }
        }
        let (a, b, c, m) = scan_slice(sl, n, &w, &t, &mut log);
        summary.push((sl.id, sl.name, sl.axis, a, b, c, m));
    }

    println!("\n================================================================================");
    println!("SUMMARY — the seam record's raw material");
    println!("================================================================================");
    println!(
        "  {:>3} {:<46} {:>12} {:>12} {:>12} {:>6}",
        "id", "slice", "d3[E_FCI]", "d3[dE4]", "min warm-cold", "n"
    );
    let mut any_solver_corner = false;
    for (id, name, _ax, a, b, c, m) in summary.iter() {
        println!(
            "  {:>3} {:<46} {:>12.3e} {:>12.3e} {:>12.2e} {:>6}",
            id,
            if name.len() > 46 { &name[..46] } else { name },
            a,
            b,
            c,
            m
        );
        if *c < -1e-9 {
            any_solver_corner = true;
        }
    }
    println!();
    if any_solver_corner {
        println!("VERDICT: at least one slice found a lower branch on a warm start. The scan has");
        println!("         located a SOLVER defect, not only a surface feature; the grid must not");
        println!("         freeze on these energies.");
    } else {
        println!("VERDICT: no warm start beat its cold solve on any slice. Every corner located");
        println!("         here belongs to the surface, and the grid design owes each one either a");
        println!("         grid line placed on it or an accepted error floor with a written reason.");
    }
    println!("\nprogress log: {}", PROGRESS_PATH);
}
