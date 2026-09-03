//! Price ONE two-site matvec on a real state: dense vs tiled block-sparse, and the tiled
//! stages apart, at the middle bond of N = 8 after two sweeps at the given chi.
//!   apply_bench [N] [x] [B] [chi] [repeats]
use q8_mps::blocks::BlockPlan;
use q8_mps::mps::{self, Env};
use q8_mps::qcd2::Qcd2;
use q8_mps::symmetric::{charge_add, dmrg_sweep_sym, random_start, SymConfig};
use std::time::Instant;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let n: usize = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(8);
    let x: f64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(4.0);
    let b: i32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(2);
    let chi: usize = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(128);
    let reps: usize = a.get(5).and_then(|s| s.parse().ok()).unwrap_or(20);
    let q = Qcd2::new(n, x);
    let n_q = q.quarks(b);
    let sector = q.sector(n_q).unwrap();
    let mpo = { let mut u = Qcd2::new(n, x); u.lam = 0.0; u.mpo(n_q) };
    let (t0, l0) = random_start(&sector, 256, 7);
    let (r, labels) = dmrg_sweep_sym(&mpo, t0, l0, &sector, &SymConfig::amendment(chi, 2)).unwrap();
    let t = r.tensors;
    let l = t.len();
    let j = l / 2 - 1;
    let mut lefts: Vec<Env> = vec![mps::trivial_left_env_mpo(mpo.sites[0].d_l)];
    for k in 0..j { let g = mps::grow_left_mpo(&lefts[k], &mpo.sites[k], &t[k]); lefts.push(g); }
    let mut rights: Vec<Env> = vec![mps::trivial_right_env_mpo(mpo.sites[l - 1].d_r); l + 1];
    for k in (0..l).rev() { rights[k] = mps::grow_right_mpo(&rights[k + 1], &mpo.sites[k], &t[k]); }
    let (left, right) = (&lefts[j], &rights[j + 2]);
    let (q_l, q_r) = (&labels[j], &labels[j + 2]);
    let (e1, e2) = (sector.site_charge[j], sector.site_charge[j + 1]);
    let (chi_l, chi_r) = (q_l.len(), q_r.len());
    // a label-consistent random vector
    let mut st = 99u64;
    let mut rnd = || { st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((st >> 11) as f64) / ((1u64 << 53) as f64) - 0.5 };
    let mut psi = vec![0.0; chi_l * 4 * chi_r];
    for (li, &ql) in q_l.iter().enumerate() { for aa in 0..2 { let qa = if aa == 1 { charge_add(ql, e1) } else { ql }; for bb in 0..2 { let qab = if bb == 1 { charge_add(qa, e2) } else { qa }; for (ri, &qr) in q_r.iter().enumerate() { if qr == qab { psi[((li * 2 + aa) * 2 + bb) * chi_r + ri] = rnd(); } } } } }
    let tp = Instant::now();
    let plan = BlockPlan::build(q_l, q_r, e1, e2, left, right).unwrap();
    let plan_s = tp.elapsed().as_secs_f64();
    let (w1, w2) = (&mpo.sites[j], &mpo.sites[j + 1]);
    let td = Instant::now();
    let mut dense = Vec::new();
    for _ in 0..reps { dense = mps::apply_effective_h_mpo(left, w1, w2, right, &psi, chi_l, chi_r); }
    let dense_ms = 1e3 * td.elapsed().as_secs_f64() / reps as f64;
    let tb = Instant::now();
    let mut stages = [0.0f64; 3];
    let mut sparse = Vec::new();
    for _ in 0..reps { let (o, s) = plan.apply_timed(left, w1, w2, right, &psi); sparse = o; for i in 0..3 { stages[i] += s[i]; } }
    let sparse_ms = 1e3 * tb.elapsed().as_secs_f64() / reps as f64;
    let mism = dense.iter().zip(&sparse).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
    println!(
        "N={n} x={x} B={b} chi={chi} (bond {j}: chi_l {chi_l} chi_r {chi_r}, live {:.2}%, plan {:.1} ms, threads {}): dense {dense_ms:.2} ms/apply, tiled {sparse_ms:.2} ms/apply ({:.1}x), stages s1 {:.2} ms | W {:.2} ms | s4 {:.2} ms; mismatched bits {mism}",
        100.0 * plan.live_entries() as f64 / psi.len() as f64, 1e3 * plan_s, mps::threads(), dense_ms / sparse_ms,
        1e3 * stages[0] / reps as f64, 1e3 * stages[1] / reps as f64, 1e3 * stages[2] / reps as f64
    );
}
