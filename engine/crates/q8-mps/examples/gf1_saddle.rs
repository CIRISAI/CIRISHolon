//! Is the identity frame a MINIMUM of M₂ in the joint angle space, or only along each axis?
//! The parity theorem (GF1_AMENDMENT_1 correction C1) shows each single-site rotation from
//! the identity raises M₂ on a parity-definite real state, so coordinate descent never
//! moves. A joint move of several sites breaks the parity the theorem needs, so the
//! identity could still be a saddle. Measured here: random joint perturbations of every
//! angle at scale ε on the Schwinger vacuum; and the Hessian's smallest eigenvalue by finite
//! differences over site pairs (N=12, χ=6 keeps one M₂ at ~1.5 s).
use q8_mps::magic::{apply_ry, sre2};
use q8_mps::schwinger::Schwinger;
use std::time::Instant;

fn lcg(seed: u64) -> impl FnMut() -> f64 {
    let mut s = seed;
    move || { s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((s >> 11) as f64) / ((1u64 << 53) as f64) - 0.5 }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let n: usize = args.first().and_then(|v| v.parse().ok()).unwrap_or(12);
    let chi: usize = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(6);
    let x: f64 = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(4.0);
    let draws: usize = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(8);
    let s = Schwinger::new(n, x, vec![]);
    let (e, res) = s.ground_energy(chi, 60, 1e-11).expect("the sweep runs");
    let t = res.tensors.clone();
    let m2 = sre2(&t).unwrap();
    println!("vacuum x={x} N={n} chi={chi}: E {e:.10}  M2 {m2:.12}");
    let rotated = |angles: &[f64]| -> Vec<_> { t.iter().zip(angles).map(|(site, &a)| apply_ry(site, a)).collect() };
    let mut next = lcg(0x4746_3153_4144);
    for &eps in &[0.02f64, 0.05, 0.1, 0.2] {
        let t0 = Instant::now();
        let mut worst = f64::MAX; let mut best_angles = vec![];
        for _ in 0..draws {
            let angles: Vec<f64> = (0..n).map(|_| eps * 2.0 * next()).collect();
            let v = sre2(&rotated(&angles)).unwrap() - m2;
            if v < worst { worst = v; best_angles = angles.clone(); }
        }
        println!("  eps {eps:.2}: {draws} random joint perturbations; min ΔM2 = {worst:+.3e}{}  ({:.0}s)", if worst < -1e-9 { "  <-- LOWER: the identity is not the joint minimum" } else { "  (all higher)" }, t0.elapsed().as_secs_f64());
        if worst < -1e-9 { println!("     angles: {:?}", best_angles.iter().map(|a| format!("{a:+.3}")).collect::<Vec<_>>()); }
    }
    // the Hessian by central differences (h = 0.02): diagonal and every pair
    let h = 0.02f64;
    let t0 = Instant::now();
    let mut hess = vec![vec![0.0f64; n]; n];
    let f = |angles: &[f64]| sre2(&rotated(angles)).unwrap();
    let zero = vec![0.0f64; n];
    let mut plus = vec![0.0; n]; let mut minus = vec![0.0; n];
    for i in 0..n {
        let mut a = zero.clone(); a[i] = h; plus[i] = f(&a);
        a[i] = -h; minus[i] = f(&a);
        hess[i][i] = (plus[i] - 2.0 * m2 + minus[i]) / (h * h);
    }
    for i in 0..n { for j in (i + 1)..n {
        let mut a = zero.clone(); a[i] = h; a[j] = h; let pp = f(&a);
        a[i] = -h; a[j] = -h; let mm = f(&a);
        // f(+,+) + f(-,-) - f(+,0) - f(-,0) - f(0,+) - f(0,-) + 2 f(0,0) = 2 h² H_ij
        let v = (pp + mm - plus[i] - minus[i] - plus[j] - minus[j] + 2.0 * m2) / (2.0 * h * h);
        hess[i][j] = v; hess[j][i] = v;
    } }
    // smallest eigenvalue by inverse power iteration is overkill: Gershgorin bounds plus a
    // Jacobi sweep on a 12x12 is enough — use a simple Jacobi eigenvalue routine
    let eig = jacobi_eigs(&hess);
    let mut e = eig.clone(); e.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("  Hessian at the identity ({:.0}s): diagonal {:?}", t0.elapsed().as_secs_f64(), (0..n).map(|i| format!("{:+.3}", hess[i][i])).collect::<Vec<_>>());
    println!("  eigenvalues: {:?}", e.iter().map(|v| format!("{v:+.3e}")).collect::<Vec<_>>());
    println!("  verdict: {}", if e[0] > 1e-6 { "positive definite — the identity is a strict local minimum in the joint space" } else if e[0] < -1e-6 { "INDEFINITE — a saddle: a joint move lowers M2" } else { "singular to this resolution" });
}

fn jacobi_eigs(a: &[Vec<f64>]) -> Vec<f64> {
    let n = a.len();
    let mut m: Vec<Vec<f64>> = a.to_vec();
    for _ in 0..100 {
        let mut off = 0.0;
        for i in 0..n { for j in 0..n { if i != j { off += m[i][j] * m[i][j]; } } }
        if off < 1e-24 { break; }
        for p in 0..n { for q in (p + 1)..n {
            if m[p][q].abs() < 1e-300 { continue; }
            let theta = (m[q][q] - m[p][p]) / (2.0 * m[p][q]);
            let tt = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
            let c = 1.0 / (tt * tt + 1.0).sqrt(); let s = tt * c;
            for k in 0..n { let (mkp, mkq) = (m[k][p], m[k][q]); m[k][p] = c * mkp - s * mkq; m[k][q] = s * mkp + c * mkq; }
            for k in 0..n { let (mpk, mqk) = (m[p][k], m[q][k]); m[p][k] = c * mpk - s * mqk; m[q][k] = s * mpk + c * mqk; }
        } }
    }
    (0..n).map(|i| m[i][i]).collect()
}
