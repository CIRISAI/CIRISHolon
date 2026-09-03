//! One two-site matvec, host tiled vs device, on a real N=8 state at the middle bond.
//!   two_site_bench [B] [chi] [repeats]
use holon_gpu::mps_blocks::GpuTwoSite;
use q8_mps::blocks::{BlockPlan, CompactPlan};
use q8_mps::mps::{self, Env};
use q8_mps::qcd2::Qcd2;
use q8_mps::symmetric::{charge_add, dmrg_sweep_sym, random_start, SymConfig};
use std::time::Instant;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let b: i32 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(2);
    let chi: usize = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(256);
    let reps: usize = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(50);
    let q = Qcd2::new(8, 4.0);
    let n_q = q.quarks(b);
    let sector = q.sector(n_q).unwrap();
    let mpo = { let mut u = Qcd2::new(8, 4.0); u.lam = 0.0; u.mpo(n_q) };
    let (t0, l0) = random_start(&sector, 256, 7);
    let (r, labels) = dmrg_sweep_sym(&mpo, t0, l0, &sector, &SymConfig::amendment(chi, 2)).unwrap();
    let t = r.tensors;
    let l = t.len();
    let j = l / 2 - 1;
    let mut lefts: Vec<Env> = vec![mps::trivial_left_env_mpo(mpo.sites[0].d_l)];
    for k in 0..j { let g = mps::grow_left_mpo(&lefts[k], &mpo.sites[k], &t[k]); lefts.push(g); }
    let mut rights: Vec<Env> = vec![mps::trivial_right_env_mpo(mpo.sites[l - 1].d_r); l + 1];
    for k in (0..l).rev() { rights[k] = mps::grow_right_mpo(&rights[k + 1], &mpo.sites[k], &t[k]); }
    let (q_l, q_r) = (&labels[j], &labels[j + 2]);
    let (e1, e2) = (sector.site_charge[j], sector.site_charge[j + 1]);
    let (chi_l, chi_r) = (q_l.len(), q_r.len());
    let mut st = 99u64;
    let mut rnd = || { st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((st >> 11) as f64) / ((1u64 << 53) as f64) - 0.5 };
    let mut psi = vec![0.0; chi_l * 4 * chi_r];
    for (li, &ql) in q_l.iter().enumerate() { for aa in 0..2 { let qa = if aa == 1 { charge_add(ql, e1) } else { ql }; for bb in 0..2 { let qab = if bb == 1 { charge_add(qa, e2) } else { qa }; for (ri, &qr) in q_r.iter().enumerate() { if qr == qab { psi[((li * 2 + aa) * 2 + bb) * chi_r + ri] = rnd(); } } } } }
    let (w1, w2) = (&mpo.sites[j], &mpo.sites[j + 1]);
    let plan = BlockPlan::build(q_l, q_r, e1, e2, &lefts[j], &rights[j + 2]).unwrap();
    let compact = CompactPlan::build(&plan, w1, w2).unwrap();
    let th = Instant::now();
    let mut host = Vec::new();
    for _ in 0..reps { host = plan.apply(&lefts[j], w1, w2, &rights[j + 2], &psi); }
    let host_ms = 1e3 * th.elapsed().as_secs_f64() / reps as f64;
    let gpu = GpuTwoSite::new(0, 512).expect("no CUDA device");
    let tu = Instant::now();
    let mut dp = gpu.upload(&compact).expect("upload");
    let up_ms = 1e3 * tu.elapsed().as_secs_f64();
    let _ = gpu.apply(&mut dp, &psi).unwrap();
    let td = Instant::now();
    let mut dev = Vec::new();
    for _ in 0..reps { dev = gpu.apply(&mut dp, &psi).unwrap(); }
    let dev_ms = 1e3 * td.elapsed().as_secs_f64() / reps as f64;
    let mism = host.iter().zip(&dev).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
    println!(
        "N=8 B={b} chi={chi} (bond {j}: chi_l {chi_l} chi_r {chi_r}, live {:.2}%, tables {:.1} MB, upload {up_ms:.1} ms once per bond, host threads {}): host tiled {host_ms:.3} ms/apply, device {dev_ms:.3} ms/apply ({:.1}x); mismatched bits {mism}",
        100.0 * plan.live_entries() as f64 / psi.len() as f64, compact.bytes() as f64 / 1e6, mps::threads(), host_ms / dev_ms
    );
}
