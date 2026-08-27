//! Q6: does the exact state's beyond-pair structure predict WHERE the chart fails?
//!
//! `Q_SEAM_PREREG.md` §6 with amendment A1/H2 (the error target is E5, `D_bool` excluded, because
//! `D_bool`'s chart prediction is a structural zero and correlating against it would correlate one
//! correlation functional of the exact state against another). N = 2 is out per A1/P4.

use q_seam::chart::Chart;
use q_seam::hubbard::Hubbard;
use q_seam::lanczos::ground_state;
use q_seam::observables::ExactObservables;
use q_seam::share::measure;
use q_seam::TAU;
use std::io::Write;

const Q6_SITES: [usize; 4] = [4, 6, 8, 10];
const DRAWS: usize = 10_000;

struct Row { n: usize, u: f64, b4: f64, db4: f64, e5: f64, honest: bool, ic3: f64 }

fn main() {
    let mut rows: Vec<Row> = Vec::new();
    let mut ic3_worst = 0.0f64;
    let mut crosscheck_worst = 0.0f64;
    let mut voids = 0usize;

    for &n in &Q6_SITES {
        let mut base = 0.0;
        for (k, &u) in q_seam::SWEEP_U.iter().enumerate() {
            let h = Hubbard::new(n, 1.0, u);
            let g = ground_state(&h).expect("gated in Q5");
            let o = ExactObservables::measure(&h, &g.vector);
            let c = Chart::best(n, 1.0, u).expect("gated in Q5");

            let e_exact = o_energy(g.energy, n);
            let cd = c.density();
            let cm = c.magnetization();
            let cb = c.bond();
            let e5 = [
                (c.energy_per_site() - e_exact).abs() / TAU[0],
                (c.double_occ_mean() - o.double_occ_mean).abs() / TAU[1],
                (0..n).map(|i| (cd[i] - o.density[i]).abs()).fold(0.0, f64::max) / TAU[2],
                (0..n).map(|i| (cm[i] - o.magnetization[i]).abs()).fold(0.0, f64::max) / TAU[3],
                (0..n - 1).map(|i| (cb[i] - o.bond[i]).abs()).fold(0.0, f64::max) / TAU[4],
            ]
            .iter()
            .copied()
            .fold(0.0, f64::max);

            let r = measure(&h, &g.vector);
            ic3_worst = ic3_worst.max(r.ic3_max);
            crosscheck_worst = crosscheck_worst.max(r.worst_crosscheck);
            if r.is_void() {
                voids += 1;
                eprintln!("VOID N={n} U={u}: failed={} crosscheck={:e}", r.failed, r.worst_crosscheck);
                continue;
            }
            if k == 0 { base = r.b4_mean; }
            rows.push(Row { n, u, b4: r.b4_mean, db4: r.b4_mean - base, e5, honest: e5 <= 1.0, ic3: r.ic3_max });
            eprintln!("done N={n} U={u}");
        }
    }

    println!("=== G-Q6-PLUMB (derived: I_C^(3) is EXACTLY zero on this family) ===");
    println!("worst |I_C^(3)| over every triple of every configuration: {ic3_worst:.4e}  (gate 1e-12)");
    println!("verdict: {}", if ic3_worst <= 1e-12 { "PASS - the estimator is validated" } else { "FAIL - Q6 is VOID, not falsified" });
    println!("worst Newton-vs-IPF disagreement: {crosscheck_worst:.4e}  (gate 1e-10)");
    println!("VOID configurations: {voids}\n");

    println!("{:>3} {:>6} {:>13} {:>13} {:>9} {:>7}", "N", "U", "B4_mean", "dB4", "E5", "honest");
    for r in &rows {
        println!("{:>3} {:>6} {:>13.5e} {:>13.5e} {:>9.3} {:>7}", r.n, r.u, r.b4, r.db4, r.e5,
            if r.honest { "yes" } else { "no" });
    }

    // P-Q6-A: partial Spearman of dB4 vs E5 controlling for U, with a within-U-column null.
    let db4: Vec<f64> = rows.iter().map(|r| r.db4).collect();
    let e5: Vec<f64> = rows.iter().map(|r| r.e5).collect();
    let uu: Vec<f64> = rows.iter().map(|r| r.u).collect();
    let observed = partial_spearman(&db4, &e5, &uu);

    let mut cols: std::collections::BTreeMap<u64, Vec<usize>> = Default::default();
    for (i, r) in rows.iter().enumerate() {
        cols.entry(r.u.to_bits()).or_default().push(i);
    }
    let mut rng = 0x5EA0_0000_0000_0006u64;
    let mut ge = 0usize;
    let mut null_vals = Vec::with_capacity(DRAWS);
    for _ in 0..DRAWS {
        let mut permuted = db4.clone();
        for idx in cols.values() {
            let mut pool: Vec<f64> = idx.iter().map(|&i| db4[i]).collect();
            for j in (1..pool.len()).rev() {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let k = (rng >> 33) as usize % (j + 1);
                pool.swap(j, k);
            }
            for (slot, &i) in idx.iter().enumerate() { permuted[i] = pool[slot]; }
        }
        let v = partial_spearman(&permuted, &e5, &uu);
        if v >= observed { ge += 1; }
        null_vals.push(v);
    }
    let p = (ge + 1) as f64 / (DRAWS + 1) as f64;
    null_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let null_med = null_vals[DRAWS / 2];
    let null_hi = null_vals[(DRAWS as f64 * 0.95) as usize];

    println!("\n=== P-Q6-A: partial Spearman rho(dB4, E5 | U) ===");
    println!("null shape first: median {null_med:.4}, 95th pct {null_hi:.4}, min {:.4}, max {:.4}",
        null_vals[0], null_vals[DRAWS - 1]);
    println!("observed rho = {observed:.4}   p = {p:.5}   (staked: rho >= 0.50 and p < 0.01)");
    println!("verdict: {}", if observed >= 0.50 && p < 0.01 { "CONFIRMED" } else { "NOT CONFIRMED" });

    // P-Q6-B: isotonic collapse, same estimator class both sides.
    let res_b4 = isotonic_median_residual(&db4, &e5);
    let res_u = isotonic_median_residual(&uu, &e5);
    let ratio = res_b4 / res_u;
    println!("\n=== P-Q6-B: isotonic collapse ===");
    println!("median |residual| of E5 ~ dB4 : {res_b4:.5}");
    println!("median |residual| of E5 ~ U/t : {res_u:.5}");
    println!("ratio = {ratio:.4}   (staked: <= 0.70)");
    println!("verdict: {}", if ratio <= 0.70 { "CONFIRMED" } else { "NOT CONFIRMED" });

    // P-Q6-C: dB4 at each N's LAST E5-honest configuration.
    println!("\n=== P-Q6-C: dB4 at the honest boundary (instrument vs thermometer) ===");
    let mut vals = Vec::new();
    for &n in &Q6_SITES {
        if let Some(r) = rows.iter().filter(|r| r.n == n && r.honest).max_by(|a, b| a.u.partial_cmp(&b.u).unwrap()) {
            println!("  N={n}: last honest U={} -> dB4 = {:.5e}", r.u, r.db4);
            vals.push(r.db4);
        }
    }
    let mean = vals.iter().sum::<f64>() / vals.len() as f64;
    let sd = (vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (vals.len() - 1) as f64).sqrt();
    let cv = sd / mean;
    println!("CV = {cv:.4}   (staked: <= 0.35)");
    println!("verdict: {}", if cv <= 0.35 { "CONFIRMED" } else { "NOT CONFIRMED" });

    // Kill clauses.
    println!("\n=== Q6 KILL CLAUSES (section 6.5, A1/H2: E5) ===");
    let honest_db4: Vec<f64> = rows.iter().filter(|r| r.honest).map(|r| r.db4).collect();
    let mut hm = honest_db4.clone();
    hm.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let honest_median = hm[hm.len() / 2];
    let a_fires = observed < 0.20 || p > 0.05;
    let b_fires: Vec<_> = rows.iter().filter(|r| r.e5 >= 3.0 && r.db4 <= 1e-10).map(|r| (r.n, r.u)).collect();
    let c_fires: Vec<_> = rows.iter().filter(|r| r.e5 <= 0.5 && r.db4 >= honest_median).map(|r| (r.n, r.u)).collect();
    println!("(a) rho<0.20 or p>0.05                     : {}", if a_fires { "FIRES" } else { "no" });
    println!("(b) dB4 at floor where E5>=3               : {} {:?}", if b_fires.is_empty() { "no" } else { "FIRES" }, b_fires);
    println!("(c) dB4 >= honest median where E5<=0.5     : {} {:?}", if c_fires.is_empty() { "no" } else { "FIRES" }, c_fires);
    println!("honest-median dB4 = {honest_median:.5e}");
    println!("\nQ6 KILL: {}", if a_fires || !b_fires.is_empty() || !c_fires.is_empty() { "FIRES" } else { "does not fire" });

    // ROBUSTNESS CLAUSE (team-lead ruling 2, clause 1): re-adjudicate under the FROZEN G-E4b,
    // which VOIDs N=6 and only N=6. If the Q6 kill verdict differs between readings it is
    // UNADJUDICATED — an amendment must never be the thing that decides a kill.
    println!("\n=== ROBUSTNESS: the FROZEN G-E4b reading (N=6 VOID) ===");
    let keep: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].n != 6).collect();
    let fb: Vec<f64> = keep.iter().map(|&i| rows[i].db4).collect();
    let fe: Vec<f64> = keep.iter().map(|&i| rows[i].e5).collect();
    let fu: Vec<f64> = keep.iter().map(|&i| rows[i].u).collect();
    let fobs = partial_spearman(&fb, &fe, &fu);
    let mut fcols: std::collections::BTreeMap<u64, Vec<usize>> = Default::default();
    for (j, &i) in keep.iter().enumerate() { fcols.entry(rows[i].u.to_bits()).or_default().push(j); }
    let mut frng = 0x5EA0_0000_0000_0007u64;
    let mut fge = 0usize;
    for _ in 0..DRAWS {
        let mut perm = fb.clone();
        for idx in fcols.values() {
            let mut pool: Vec<f64> = idx.iter().map(|&j| fb[j]).collect();
            for j in (1..pool.len()).rev() {
                frng = frng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let k = (frng >> 33) as usize % (j + 1);
                pool.swap(j, k);
            }
            for (slot, &j) in idx.iter().enumerate() { perm[j] = pool[slot]; }
        }
        if partial_spearman(&perm, &fe, &fu) >= fobs { fge += 1; }
    }
    let fp_ = (fge + 1) as f64 / (DRAWS + 1) as f64;
    let fiso = isotonic_median_residual(&fb, &fe) / isotonic_median_residual(&fu, &fe);
    let mut fvals = Vec::new();
    for &n in &Q6_SITES {
        if n == 6 { continue; }
        if let Some(r) = rows.iter().filter(|r| r.n == n && r.honest).max_by(|a, b| a.u.partial_cmp(&b.u).unwrap()) {
            fvals.push(r.db4);
        }
    }
    let fmean = fvals.iter().sum::<f64>() / fvals.len() as f64;
    let fsd = (fvals.iter().map(|v| (v - fmean).powi(2)).sum::<f64>() / (fvals.len() - 1) as f64).sqrt();
    let fcv = fsd / fmean;
    let mut fhm: Vec<f64> = keep.iter().filter(|&&i| rows[i].honest).map(|&i| rows[i].db4).collect();
    fhm.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let fmed = fhm[fhm.len() / 2];
    let fa = fobs < 0.20 || fp_ > 0.05;
    let fb_fire = keep.iter().any(|&i| rows[i].e5 >= 3.0 && rows[i].db4 <= 1e-10);
    let fc = keep.iter().any(|&i| rows[i].e5 <= 0.5 && rows[i].db4 >= fmed);
    let frozen_kill = fa || fb_fire || fc;
    println!("rho = {fobs:.4}, p = {fp_:.5}  (staked >= 0.50, p < 0.01)");
    println!("isotonic ratio = {fiso:.4}  (staked <= 0.70)");
    println!("boundary CV = {fcv:.4}  (staked <= 0.35)");
    println!("clauses: (a) {} (b) {} (c) {}",
        if fa { "FIRES" } else { "no" }, if fb_fire { "FIRES" } else { "no" }, if fc { "FIRES" } else { "no" });
    let full_kill = a_fires || !b_fires.is_empty() || !c_fires.is_empty();
    println!("\nfull reading   : Q6 kill fires = {full_kill}");
    println!("frozen reading : Q6 kill fires = {frozen_kill}");
    if full_kill != frozen_kill {
        println!("ADJUDICATIONS DIFFER -> the Q6 kill is UNADJUDICATED.");
    } else {
        println!("ADJUDICATIONS AGREE -> the Q6 kill verdict does not depend on amendment A2.");
    }

    let dir = "/home/emoore/CIRISOntology/sim_engine/output/q_seam";
    let mut f = std::fs::File::create(format!("{dir}/q6.json")).unwrap();
    writeln!(f, "{{\"rho\":{observed},\"p\":{p},\"iso_ratio\":{ratio},\"cv\":{cv},\"ic3_worst\":{ic3_worst},\"rows\":[").unwrap();
    for (k, r) in rows.iter().enumerate() {
        writeln!(f, "{}{{\"N\":{},\"U\":{},\"b4\":{},\"db4\":{},\"e5\":{},\"honest\":{},\"ic3\":{}}}",
            if k == 0 { "" } else { "," }, r.n, r.u, r.b4, r.db4, r.e5, r.honest, r.ic3).unwrap();
    }
    writeln!(f, "]}}").unwrap();
    println!("\nwrote {dir}/q6.json");
}

fn o_energy(e: f64, n: usize) -> f64 { e / n as f64 }

fn ranks(v: &[f64]) -> Vec<f64> {
    let mut idx: Vec<usize> = (0..v.len()).collect();
    idx.sort_by(|&a, &b| v[a].partial_cmp(&v[b]).unwrap());
    let mut r = vec![0.0; v.len()];
    let mut i = 0;
    while i < idx.len() {
        let mut j = i;
        while j + 1 < idx.len() && v[idx[j + 1]] == v[idx[i]] { j += 1; }
        let avg = ((i + j) as f64) / 2.0 + 1.0;
        for k in i..=j { r[idx[k]] = avg; }
        i = j + 1;
    }
    r
}

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
    let num: f64 = a.iter().zip(b).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let da: f64 = a.iter().map(|x| (x - ma).powi(2)).sum::<f64>().sqrt();
    let db: f64 = b.iter().map(|y| (y - mb).powi(2)).sum::<f64>().sqrt();
    if da == 0.0 || db == 0.0 { 0.0 } else { num / (da * db) }
}

fn partial_spearman(x: &[f64], y: &[f64], z: &[f64]) -> f64 {
    let (rx, ry, rz) = (ranks(x), ranks(y), ranks(z));
    let (xy, xz, yz) = (pearson(&rx, &ry), pearson(&rx, &rz), pearson(&ry, &rz));
    let den = ((1.0 - xz * xz) * (1.0 - yz * yz)).sqrt();
    if den <= 0.0 { 0.0 } else { (xy - xz * yz) / den }
}

/// Median absolute residual of an isotonic (monotone non-decreasing) fit of `y` on `x`, by PAVA.
fn isotonic_median_residual(x: &[f64], y: &[f64]) -> f64 {
    let mut idx: Vec<usize> = (0..x.len()).collect();
    idx.sort_by(|&a, &b| x[a].partial_cmp(&x[b]).unwrap());
    let ys: Vec<f64> = idx.iter().map(|&i| y[i]).collect();
    let mut val = ys.clone();
    let mut wt = vec![1.0; ys.len()];
    let mut k = 0;
    while k + 1 < val.len() {
        if val[k] <= val[k + 1] { k += 1; continue; }
        let w = wt[k] + wt[k + 1];
        let v = (val[k] * wt[k] + val[k + 1] * wt[k + 1]) / w;
        val[k] = v; wt[k] = w;
        val.remove(k + 1); wt.remove(k + 1);
        // Re-pool backwards.
        while k > 0 && val[k - 1] > val[k] {
            let w2 = wt[k - 1] + wt[k];
            let v2 = (val[k - 1] * wt[k - 1] + val[k] * wt[k]) / w2;
            val[k - 1] = v2; wt[k - 1] = w2;
            val.remove(k); wt.remove(k);
            k -= 1;
        }
    }
    // Expand the pooled blocks back to per-point fitted values.
    let mut fitted = Vec::with_capacity(ys.len());
    for (v, w) in val.iter().zip(wt.iter()) {
        for _ in 0..(*w as usize) { fitted.push(*v); }
    }
    let mut res: Vec<f64> = ys.iter().zip(fitted.iter()).map(|(a, b)| (a - b).abs()).collect();
    res.sort_by(|a, b| a.partial_cmp(b).unwrap());
    res[res.len() / 2]
}
