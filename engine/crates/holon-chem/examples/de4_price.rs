//! What does dE4 actually COST, and is the six-distance canonical form actually canonical?
//!
//! Two questions this answers with measurements rather than recollection, because the
//! brief that ordered the dE4 table carried a stale price and a stale symbol name
//! (M-STALE-INSTRUMENT):
//!
//! 1. **The price.** `M-CHEAPER-THAN-ITS-PRICE` says the banked cost model is a
//!    falsifying check, so the model has to be banked from a measurement on THIS machine
//!    at a recorded load — not quoted. Two prices are separable and are reported
//!    separately, because they buy different things:
//!      * the VALUE price, one `de4_ohhh_fci` call: what a table NODE costs to solve;
//!      * the GRADIENT price, the path `holon-render`'s `accumulate_four_body` actually
//!        runs: one base call plus three forward-difference calls, four solves per
//!        quadruple per force evaluation. That is what a table would REPLACE, so it is
//!        the price the tabulation decision has to be made against.
//!    The value call is itself split, because `de4_ohhh_fci` is a difference of two
//!    solves and the cheaper-looking half is not free: `ohhh_fci_energy` (the 4-centre
//!    FCI) and `ohhh_mbe3_energy` (six `pair_point` two-centre solves plus four table
//!    lookups) are timed apart.
//!
//! 2. **The canonical form.** `quaternary::sort_ohhh_internals` sorts the three O-H
//!    distances and the three H-H distances INDEPENDENTLY. Independent sorting is
//!    invariant under S3 x S3 (order 36), not under the S3 (order 6) that actually acts
//!    on a geometry — the hydrogen relabelling that moves R2 also moves which pair R23
//!    belongs to. If that is a real collision then two DIFFERENT geometries share one
//!    address, and a table indexed on it stores one value where two are owed. This
//!    exhibits the collision, or fails to, by construction rather than by argument:
//!    build a pair of geometries that agree on both sorted triples, and read dE4 at each.
//!    The proposed repair — the lexicographic minimum of the 6-tuple over the six
//!    relabellings — is scored on the same pair.
//!
//! 3. **The hole.** A box in the six distances is not a box of geometries: three unit
//!    vectors from O exist in R^3 only where the Gram determinant
//!    `G = 1 + 2 u12 u23 u31 - u12^2 - u23^2 - u31^2` is non-negative. This measures
//!    what fraction of a candidate box is not a geometry at all, which is the number
//!    that decides whether the table can be built on distances directly or needs a
//!    continuation rule for its stencil.
//!
//! Run: `cargo run --release -p holon-chem --example de4_price -- [n_price_samples]`

use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point};
use holon_chem::quaternary::{de4_ohhh_fci, ohhh_fci_energy, ohhh_fci_grad, ohhh_mbe3_energy, sort_ohhh_internals};
use holon_chem::trimer::{self, TrimerTable};
use holon_chem::water::{self, WaterTable};
use std::time::Instant;

// The water monomer geometry that seeds the witnesses now lives with the witnesses,
// in `quaternary_table::staked_witnesses`.
const PI: f64 = std::f64::consts::PI;

fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// The 40 staked witness geometries.
///
/// The construction moved into the library as `quaternary_table::staked_witnesses` so that
/// this pricing probe, `examples/de4_certify.rs`'s held-out gates and `tests/quaternary.rs`
/// cannot drift apart: a held-out set that exists in three copies is a held-out set that
/// can silently stop being the same one.
fn witnesses() -> Vec<[[f64; 3]; 4]> {
    holon_chem::quaternary_table::staked_witnesses()
}

/// The proposed repair: the lexicographic least of the six relabelled 6-tuples.
///
/// Comparisons only, so it is exact in f64 and the same geometry presented under any of
/// the six labellings produces the identical array bit-for-bit.
fn canonical6(r: [f64; 3], rhh: [f64; 3]) -> [f64; 6] {
    // rhh is indexed [r12, r23, r31]; under sigma the pair {i,j} maps to {s(i),s(j)}.
    const PERMS: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    // hh_of[i][j] gives the index into rhh of the H_i-H_j distance.
    let hh = |i: usize, j: usize| -> f64 {
        match (i.min(j), i.max(j)) {
            (0, 1) => rhh[0],
            (1, 2) => rhh[1],
            (0, 2) => rhh[2],
            _ => unreachable!(),
        }
    };
    let mut best: Option<[f64; 6]> = None;
    for p in PERMS.iter() {
        let cand = [
            r[p[0]],
            r[p[1]],
            r[p[2]],
            hh(p[0], p[1]),
            hh(p[1], p[2]),
            hh(p[2], p[0]),
        ];
        best = Some(match best {
            None => cand,
            Some(b) => {
                let mut take = false;
                for k in 0..6 {
                    if cand[k] < b[k] {
                        take = true;
                        break;
                    }
                    if cand[k] > b[k] {
                        break;
                    }
                }
                if take {
                    cand
                } else {
                    b
                }
            }
        });
    }
    best.unwrap()
}

/// Place four atoms realising a target set of six distances, or report that the six are
/// not a geometry. O at origin; H1 on +x; H2 in the xy-plane; H3 out of plane.
fn embed(r: [f64; 3], rhh: [f64; 3]) -> Option<[[f64; 3]; 4]> {
    let (r1, r2, r3) = (r[0], r[1], r[2]);
    let (r12, r23, r31) = (rhh[0], rhh[1], rhh[2]);
    let u12 = (r1 * r1 + r2 * r2 - r12 * r12) / (2.0 * r1 * r2);
    let u23 = (r2 * r2 + r3 * r3 - r23 * r23) / (2.0 * r2 * r3);
    let u31 = (r3 * r3 + r1 * r1 - r31 * r31) / (2.0 * r3 * r1);
    if !(u12.abs() <= 1.0) || !(u23.abs() <= 1.0) || !(u31.abs() <= 1.0) {
        return None;
    }
    let g = 1.0 + 2.0 * u12 * u23 * u31 - u12 * u12 - u23 * u23 - u31 * u31;
    if !(g >= 0.0) {
        return None;
    }
    let o = [0.0, 0.0, 0.0];
    let e1 = [1.0, 0.0, 0.0];
    let s12 = (1.0 - u12 * u12).max(0.0).sqrt();
    let e2 = [u12, s12, 0.0];
    // e3 . e1 = u31, e3 . e2 = u23
    let a = u31;
    let b = if s12 > 1e-14 {
        (u23 - u12 * u31) / s12
    } else {
        return None;
    };
    let c2 = 1.0 - a * a - b * b;
    if !(c2 >= 0.0) {
        return None;
    }
    let e3 = [a, b, c2.sqrt()];
    Some([
        o,
        [e1[0] * r1, e1[1] * r1, e1[2] * r1],
        [e2[0] * r2, e2[1] * r2, e2[2] * r2],
        [e3[0] * r3, e3[1] * r3, e3[2] * r3],
    ])
}

fn internals(g: &[[f64; 3]; 4]) -> ([f64; 3], [f64; 3]) {
    (
        [dist(&g[0], &g[1]), dist(&g[0], &g[2]), dist(&g[0], &g[3])],
        [dist(&g[1], &g[2]), dist(&g[2], &g[3]), dist(&g[3], &g[1])],
    )
}

/// The gradient path exactly as `sim.rs::accumulate_four_body` runs it: one base solve
/// plus three FORWARD radial finite differences. Reproduced here rather than called
/// because `sim.rs` is in another crate and behind a Sim; the count of solves, which is
/// what is being priced, is what has to match, and it does: 1 + 3.
fn gradient_path(g: &[[f64; 3]; 4], w: &WaterTable, t: &TrimerTable) -> (f64, [f64; 3]) {
    const H_FD: f64 = 1e-4;
    let base = de4_ohhh_fci(g, w, t);
    let mut fmag = [0.0f64; 3];
    for hid in 0..3 {
        let li = hid + 1;
        let r = dist(&g[0], &g[li]).max(1e-12);
        let u = [
            (g[li][0] - g[0][0]) / r,
            (g[li][1] - g[0][1]) / r,
            (g[li][2] - g[0][2]) / r,
        ];
        let mut m = *g;
        m[li][0] += H_FD * u[0];
        m[li][1] += H_FD * u[1];
        m[li][2] += H_FD * u[2];
        let ep = de4_ohhh_fci(&m, w, t);
        fmag[hid] = -(ep - base) / H_FD;
    }
    (base, fmag)
}


/// Process CPU seconds (user+sys) from /proc/self/stat, so a price can say whether it is
/// the code's or the machine's.
fn cpu_seconds() -> f64 {
    let s = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    // field 14 (utime) and 15 (stime), 1-based, after the comm field in parentheses.
    let after = match s.rfind(')') { Some(i) => &s[i + 1..], None => return 0.0 };
    let f: Vec<&str> = after.split_whitespace().collect();
    let hz = 100.0; // USER_HZ is 100 on every Linux this runs on
    let ut: f64 = f.get(11).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    let st: f64 = f.get(12).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    (ut + st) / hz
}

/// Place four atoms from three O-H distances and the three H-O-H cosines. Returns None
/// exactly when the three unit vectors do not exist in R^3 (Gram determinant negative).
fn embed_from_cosines(r: [f64; 3], u: [f64; 3]) -> Option<[[f64; 3]; 4]> {
    let (u12, u23, u31) = (u[0], u[1], u[2]);
    let g = 1.0 + 2.0 * u12 * u23 * u31 - u12 * u12 - u23 * u23 - u31 * u31;
    if !(g >= 0.0) {
        return None;
    }
    let s12 = (1.0 - u12 * u12).max(0.0).sqrt();
    if s12 < 1e-14 {
        return None;
    }
    let e1 = [1.0, 0.0, 0.0];
    let e2 = [u12, s12, 0.0];
    let a = u31;
    let b = (u23 - u12 * u31) / s12;
    let c2 = 1.0 - a * a - b * b;
    if !(c2 >= 0.0) {
        return None;
    }
    let e3 = [a, b, c2.sqrt()];
    Some([
        [0.0, 0.0, 0.0],
        [e1[0] * r[0], e1[1] * r[0], e1[2] * r[0]],
        [e2[0] * r[1], e2[1] * r[1], e2[2] * r[1]],
        [e3[0] * r[2], e3[1] * r[2], e3[2] * r[2]],
    ])
}

fn pct(v: &mut Vec<f64>, q: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let i = ((v.len() - 1) as f64 * q).round() as usize;
    v[i]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n_price: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(20);

    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("=== dE4 PRICE AND CANONICAL-FORM PROBE ===");
    println!("host loadavg      {}", load.trim());
    println!("threads           1 (this probe is single-threaded on purpose: a price");
    println!("                  measured against contention is the machine's, not the code's)");

    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/data/s2/s2_water_table.txt"
    ))
    .expect("the committed (O,H,H) table");
    let w = water::from_text(&src).expect("water table parses");
    let t0 = Instant::now();
    let t = trimer::generate().expect("the H3 table");
    println!("trimer::generate  {:.3} s (once, in-process; the H3 table is not an artifact)", t0.elapsed().as_secs_f64());

    let wit = witnesses();
    println!("witnesses         {}", wit.len());

    // ---------------------------------------------------------------- 1. the price
    println!("\n--- 1. PRICE (wall AND process CPU; if they agree, contention is not the story) ---");
    let cpu0 = cpu_seconds();
    let wall0 = Instant::now();
    let mut v_value = Vec::new();
    let mut v_fci = Vec::new();
    let mut iters_seen: Vec<f64> = Vec::new();
    for g in wit.iter() {
        let a = Instant::now();
        let _ = de4_ohhh_fci(g, &w, &t);
        v_value.push(a.elapsed().as_secs_f64() * 1e3);
        let a = Instant::now();
        let _ = ohhh_fci_energy(g);
        v_fci.push(a.elapsed().as_secs_f64() * 1e3);
        iters_seen.push(0.0);
    }
    let wall_all = wall0.elapsed().as_secs_f64();
    let cpu_all = cpu_seconds() - cpu0;
    let vmed = pct(&mut v_value, 0.5);
    println!("VALUE de4_ohhh_fci over ALL {} witnesses:", wit.len());
    println!("  min {:8.1}  p10 {:8.1}  median {:8.1}  p90 {:8.1}  max {:8.1} ms",
        pct(&mut v_value, 0.0), pct(&mut v_value, 0.1), vmed,
        pct(&mut v_value, 0.9), pct(&mut v_value, 1.0));
    let mean_v: f64 = v_value.iter().sum::<f64>() / v_value.len() as f64;
    println!("  MEAN {:8.1} ms   <- the number a grid cost model must use, not the median", mean_v);
    println!("  of which the 4-centre FCI: median {:8.1} ms", pct(&mut v_fci, 0.5));
    println!("  section wall {:.2} s, process CPU {:.2} s, CPU/wall {:.3}",
        wall_all, cpu_all, cpu_all / wall_all);

    let mut v_grad = Vec::new();
    let cpu1 = cpu_seconds();
    let wall1 = Instant::now();
    for g in wit.iter().take(10) {
        let a = Instant::now();
        let _ = gradient_path(g, &w, &t);
        v_grad.push(a.elapsed().as_secs_f64() * 1e3);
    }
    let gw = wall1.elapsed().as_secs_f64();
    let gc = cpu_seconds() - cpu1;
    let gmed = pct(&mut v_grad, 0.5);
    let gmean: f64 = v_grad.iter().sum::<f64>() / v_grad.len() as f64;
    println!("GRADIENT (the path that runs: 1 base + 3 forward FD = 4 solves), n={}:", v_grad.len());
    println!("  median {:8.1} ms   MEAN {:8.1} ms   ratio mean/value-mean {:.2}x",
        gmed, gmean, gmean / mean_v);
    println!("  section wall {:.2} s, process CPU {:.2} s, CPU/wall {:.3}", gw, gc, gc / gw);
    println!("  NOTE the running path is FORWARD difference and yields ONLY the three radial");
    println!("       O-H force components; sim.rs's doc comment says 'central', and nothing");
    println!("       in it generates a force along any H-H coordinate.");

    // Grid-like geometries: the table has to solve the whole domain, not the witnesses,
    // and stretched/compressed geometries are where Davidson is slowest. Price a sample.
    println!("\n--- 1b. PRICE ON GRID-LIKE GEOMETRIES (what a table actually pays) ---");
    let mut v_grid = Vec::new();
    let mut n_try = 0usize;
    let mut n_ok = 0usize;
    let rs = [1.4f64, 2.2, 3.4, 5.0];
    let us = [-0.9f64, -0.5, 0.0, 0.5];
    'outer: for &a in rs.iter() {
        for &b in rs.iter() {
            for &c in rs.iter() {
                for &u1 in us.iter() {
                    for &u2 in us.iter() {
                        for &u3 in us.iter() {
                            n_try += 1;
                            if let Some(g) = embed_from_cosines([a, b, c], [u1, u2, u3]) {
                                n_ok += 1;
                                if n_ok % 7 != 1 { continue; }
                                let t0 = Instant::now();
                                let _ = de4_ohhh_fci(&g, &w, &t);
                                v_grid.push(t0.elapsed().as_secs_f64() * 1e3);
                                if v_grid.len() >= 40 { break 'outer; }
                            }
                        }
                    }
                }
            }
        }
    }
    if !v_grid.is_empty() {
        let gm: f64 = v_grid.iter().sum::<f64>() / v_grid.len() as f64;
        println!("  sampled {} of {} embeddable of {} attempted box points", v_grid.len(), n_ok, n_try);
        println!("  min {:8.1}  median {:8.1}  MEAN {:8.1}  max {:8.1} ms",
            pct(&mut v_grid, 0.0), pct(&mut v_grid, 0.5), gm, pct(&mut v_grid, 1.0));
        println!("  grid-mean / witness-mean = {:.2}x", gm / mean_v);
    }

    // ---------------------------------------- 1c. THE PATH THAT ACTUALLY RUNS, TODAY
    //
    // The trajectory loop was rewritten mid-campaign (commit 21e6be3): the four-body force
    // is no longer one value solve plus three forward radial differences. It is now
    // `ohhh_fci_grad`, NINE seeded dual solves -- three moving atoms times three axes --
    // with the oxygen row imposed by translation invariance and a CI vector warm-started
    // per oxygen hub. That is a different price and a different quantity (the full
    // Cartesian gradient, where the old scheme delivered only the radial projection), so
    // the earlier reading is stale on its own terms and this section replaces it.
    println!("\n--- 1c. THE CURRENT GRADIENT PATH: ohhh_fci_grad, 9 seeded dual solves ---");
    let mut v_grad9 = Vec::new();
    let cpu2 = cpu_seconds();
    let wall2 = Instant::now();
    let mut warm: Option<Vec<f64>> = None;
    for g in wit.iter().take(8) {
        let a = Instant::now();
        let r = ohhh_fci_grad(g, warm.as_deref());
        warm = Some(r.ci);
        v_grad9.push(a.elapsed().as_secs_f64() * 1e3);
    }
    let g9w = wall2.elapsed().as_secs_f64();
    let g9c = cpu_seconds() - cpu2;
    let g9mean: f64 = v_grad9.iter().sum::<f64>() / v_grad9.len() as f64;
    println!("  n={}  median {:8.1} ms   MEAN {:8.1} ms", v_grad9.len(),
        pct(&mut v_grad9, 0.5), g9mean);
    println!("  section wall {:.2} s, process CPU {:.2} s, CPU/wall {:.3}", g9w, g9c, g9c / g9w);
    println!("  MEAN CPU per force evaluation {:8.1} ms", g9mean * (g9c / g9w));
    println!("  This is what a table read replaces. A 4^6 = 4096-node Catmull-Rom stencil");
    println!("  contracted seven times is ~40k flops, tens of microseconds -- so the");
    println!("  tabulation decision is not close, and it is not close under EITHER price.");

    // ---------------------------------------------- 2. the canonical-form collision
    println!("\n--- 2. IS sort_ohhh_internals A CANONICAL FORM? ---");
    // The orbit of a labelled geometry under hydrogen relabelling has SIX members when
    // the three O-H distances are distinct (the stabiliser is trivial). Independent
    // sorting identifies all THIRTY-SIX combinations of (a permutation of the O-H triple)
    // with (a permutation of the H-H triple). Thirty-six over six is six: generically,
    // six distinct geometries are handed one address. Exhibited on exact inputs, so no
    // round-trip rounding can mask it.
    let r = [1.9f64, 2.4, 3.0];
    let hh_a = [2.6f64, 3.3, 4.1];
    let hh_b = [2.6f64, 4.1, 3.3]; // NOT in A's relabelling orbit: r's entries are distinct
    let sa = sort_ohhh_internals(r[0], r[1], r[2], hh_a[0], hh_a[1], hh_a[2]);
    let sb = sort_ohhh_internals(r[0], r[1], r[2], hh_b[0], hh_b[1], hh_b[2]);
    let ca = canonical6(r, hh_a);
    let cb = canonical6(r, hh_b);
    println!("A: R_OH {:?}  R_HH(12,23,31) {:?}", r, hh_a);
    println!("B: R_OH {:?}  R_HH(12,23,31) {:?}", r, hh_b);
    println!("sort_ohhh_internals(A) = {:?}", sa);
    println!("sort_ohhh_internals(B) = {:?}", sb);
    println!("  SAME ADDRESS under sort_ohhh_internals: {}", sa == sb);
    println!("canonical6(A) = {:?}", ca);
    println!("canonical6(B) = {:?}", cb);
    println!("  SAME ADDRESS under canonical6:          {}", ca == cb);
    match (embed(r, hh_a), embed(r, hh_b)) {
        (Some(ga), Some(gb)) => {
            let ea = de4_ohhh_fci(&ga, &w, &t);
            let eb = de4_ohhh_fci(&gb, &w, &t);
            println!("dE4(A) = {:.9} Ha", ea);
            println!("dE4(B) = {:.9} Ha", eb);
            println!("|dE4(A) - dE4(B)| = {:.6e} Ha", (ea - eb).abs());
            if sa == sb && (ea - eb).abs() > 1e-6 {
                println!(">>> COLLISION CONFIRMED: sort_ohhh_internals gives ONE address to two");
                println!("    geometries whose dE4 differ by {:.3e} Ha. It is not a canonical form.", (ea - eb).abs());
            }
            if ca != cb {
                println!(">>> canonical6 SEPARATES them.");
            }
        }
        _ => println!("(one candidate is not embeddable; the address collision stands regardless)"),
    }
    // canonical6 must still be exactly S3-invariant.
    {
        let base = canonical6(r, hh_a);
        let c1 = canonical6([r[1], r[2], r[0]], [hh_a[1], hh_a[2], hh_a[0]]);
        let c2 = canonical6([r[1], r[0], r[2]], [hh_a[0], hh_a[2], hh_a[1]]);
        println!("canonical6 S3-invariance (bit-exact): cyc {}  transposition {}",
            c1 == base, c2 == base);
    }

    // ------------------------------------------------------------------ 3. the hole
    println!("\n--- 3. HOW MUCH OF A DISTANCE BOX IS NOT A GEOMETRY? ---");
    println!("(Gram determinant G = 1 + 2 u12 u23 u31 - u12^2 - u23^2 - u31^2 >= 0)");
    for &(rlo, rhi, n) in &[(1.2f64, 6.0f64, 21usize)] {
        // Sample the box in the six distances uniformly on a grid and count.
        let mut total = 0usize;
        let mut bad_cos = 0usize;
        let mut bad_gram = 0usize;
        let hlo = 0.9f64;
        let hhi = 2.0 * rhi;
        for i in 0..n {
            let r1 = rlo + (rhi - rlo) * i as f64 / (n - 1) as f64;
            for j in 0..n {
                let r2 = rlo + (rhi - rlo) * j as f64 / (n - 1) as f64;
                for k in 0..n {
                    let r3 = rlo + (rhi - rlo) * k as f64 / (n - 1) as f64;
                    for a in 0..n {
                        let d12 = hlo + (hhi - hlo) * a as f64 / (n - 1) as f64;
                        for b in 0..n {
                            let d23 = hlo + (hhi - hlo) * b as f64 / (n - 1) as f64;
                            for c in 0..n {
                                let d31 = hlo + (hhi - hlo) * c as f64 / (n - 1) as f64;
                                total += 1;
                                let u12 = (r1 * r1 + r2 * r2 - d12 * d12) / (2.0 * r1 * r2);
                                let u23 = (r2 * r2 + r3 * r3 - d23 * d23) / (2.0 * r2 * r3);
                                let u31 = (r3 * r3 + r1 * r1 - d31 * d31) / (2.0 * r3 * r1);
                                if u12.abs() > 1.0 || u23.abs() > 1.0 || u31.abs() > 1.0 {
                                    bad_cos += 1;
                                    continue;
                                }
                                let g = 1.0 + 2.0 * u12 * u23 * u31
                                    - u12 * u12
                                    - u23 * u23
                                    - u31 * u31;
                                if g < 0.0 {
                                    bad_gram += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        let good = total - bad_cos - bad_gram;
        println!("box R_OH [{:.1},{:.1}]^3 x R_HH [{:.1},{:.1}]^3, {}^6 = {} points",
            rlo, rhi, hlo, hhi, n, total);
        println!("  not a triangle through O (|cos|>1): {:>10}  ({:5.2}%)", bad_cos, 100.0 * bad_cos as f64 / total as f64);
        println!("  triangles but not embeddable (G<0): {:>10}  ({:5.2}%)", bad_gram, 100.0 * bad_gram as f64 / total as f64);
        println!("  REAL GEOMETRIES:                    {:>10}  ({:5.2}%)", good, 100.0 * good as f64 / total as f64);
    }

    // The cosine box on its own, which is the shape a (R,R,R,u,u,u) grid would use.
    {
        let n = 101usize;
        let mut total = 0usize;
        let mut bad = 0usize;
        for a in 0..n {
            let u12 = -1.0 + 2.0 * a as f64 / (n - 1) as f64;
            for b in 0..n {
                let u23 = -1.0 + 2.0 * b as f64 / (n - 1) as f64;
                for c in 0..n {
                    let u31 = -1.0 + 2.0 * c as f64 / (n - 1) as f64;
                    total += 1;
                    let g = 1.0 + 2.0 * u12 * u23 * u31 - u12 * u12 - u23 * u23 - u31 * u31;
                    if g < 0.0 {
                        bad += 1;
                    }
                }
            }
        }
        println!("cosine box [-1,1]^3, {}^3 points: not embeddable {:5.2}%  (analytic: 1 - pi^2/16 = {:5.2}%)",
            n, 100.0 * bad as f64 / total as f64, 100.0 * (1.0 - PI * PI / 16.0));
    }

    // ------------------------------------------------- 4. what the atom energies are
    println!("\n--- 4. reference scalars ---");
    println!("E(O) = {:.12} Ha   E(H) = {:.12} Ha", atom_energy(OXYGEN), atom_energy(HYDROGEN));
    let p = pair_point(OXYGEN, HYDROGEN, 1.9435740105);
    println!("pair_point(O,H,1.94357) e = {:.12} Ha", p.e);
}
