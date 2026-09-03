//! MEASURE the label-block sparsity the symmetric arm leaves on the table (E14 item 1):
//! per MPO channel, is the environment block-sparse in the bond labels with ONE charge
//! shift per channel? What fraction of environment entries and of the two-site
//! wavefunction is structurally nonzero? On the random labelled start and after two
//! sweeps at chi=64, N=8, every sector.
//!
//!   sym_sparsity [N] [x] [sweeps] [chi]
use q8_mps::mps::{self, Env, TensorSite};
use q8_mps::qcd2::Qcd2;
use q8_mps::symmetric::{charge_sub, dmrg_sweep_sym, random_start, Charge, Labels, SymConfig};

fn all_right(t: &[TensorSite], mpo: &q8_mps::mpo::Mpo) -> Vec<Env> {
    let l = t.len();
    let mut envs: Vec<Env> = vec![mps::trivial_right_env_mpo(mpo.sites[l - 1].d_r); l + 1];
    for k in (0..l).rev() {
        envs[k] = mps::grow_right_mpo(&envs[k + 1], &mpo.sites[k], &t[k]);
    }
    envs
}
fn all_left(t: &[TensorSite], mpo: &q8_mps::mpo::Mpo) -> Vec<Env> {
    let mut envs: Vec<Env> = vec![mps::trivial_left_env_mpo(mpo.sites[0].d_l)];
    for (j, tj) in t.iter().enumerate() {
        let g = mps::grow_left_mpo(&envs[j], &mpo.sites[j], tj);
        envs.push(g);
    }
    envs
}

/// Per channel: (live entries, total entries, distinct charge shifts q_row − q_col seen).
fn env_report(env: &Env, q: &[Charge]) -> Vec<(usize, usize, usize, Vec<Charge>)> {
    let chi = q.len();
    let mut out = Vec::new();
    for (c, m) in env.iter().enumerate() {
        if m.iter().all(|&v| v == 0.0) {
            continue;
        }
        let mut live = 0usize;
        let mut shifts: Vec<Charge> = Vec::new();
        for i in 0..chi {
            for j in 0..chi {
                if m[i * chi + j] != 0.0 {
                    live += 1;
                    let d = charge_sub(q[i], q[j]);
                    if !shifts.contains(&d) {
                        shifts.push(d);
                    }
                }
            }
        }
        out.push((c, live, chi * chi, shifts));
    }
    out
}

fn report(tag: &str, tensors: &[TensorSite], labels: &Labels, mpo: &q8_mps::mpo::Mpo, sector: &q8_mps::symmetric::Sector) {
    let l = tensors.len();
    let j = l / 2 - 1; // the middle two-site update (j, j+1)
    let lefts = all_left(tensors, mpo);
    let rights = all_right(tensors, mpo);
    let (le, re) = (&lefts[j], &rights[j + 2]);
    let (ql, qr) = (&labels[j], &labels[j + 2]);
    let mut env_live = 0usize;
    let mut env_tot = 0usize;
    let mut multi_shift = 0usize;
    let mut chans = 0usize;
    for (env, q, side) in [(le, ql, "L"), (re, qr, "R")] {
        for (c, live, tot, shifts) in env_report(env, q) {
            env_live += live;
            env_tot += tot;
            chans += 1;
            if shifts.len() > 1 {
                multi_shift += 1;
                println!("   {side} channel {c}: {} DISTINCT SHIFTS {:?}", shifts.len(), &shifts[..shifts.len().min(4)]);
            }
        }
    }
    // the two-site wavefunction's live fraction by the label rule
    let (e1, e2) = (sector.site_charge[j], sector.site_charge[j + 1]);
    let mut psi_live = 0usize;
    for &a in ql {
        for s1 in 0..2 {
            for s2 in 0..2 {
                let mut qab = a;
                if s1 == 1 { qab = q8_mps::symmetric::charge_add(qab, e1); }
                if s2 == 1 { qab = q8_mps::symmetric::charge_add(qab, e2); }
                psi_live += qr.iter().filter(|&&b| b == qab).count();
            }
        }
    }
    let psi_tot = ql.len() * 4 * qr.len();
    println!(
        "{tag}: bond {j}: chi_l {} ({} labels) chi_r {} ({} labels) | live env channels {chans}, channels with >1 shift {multi_shift} | env live {env_live}/{env_tot} = {:.2}% | psi live {psi_live}/{psi_tot} = {:.2}% (waste {:.1}x)",
        ql.len(), { let mut u = ql.clone(); u.sort(); u.dedup(); u.len() },
        qr.len(), { let mut u = qr.clone(); u.sort(); u.dedup(); u.len() },
        100.0 * env_live as f64 / env_tot as f64,
        100.0 * psi_live as f64 / psi_tot as f64,
        psi_tot as f64 / psi_live.max(1) as f64,
    );
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let n: usize = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(8);
    let x: f64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(4.0);
    let sweeps: usize = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(2);
    let chi: usize = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(64);
    let q = Qcd2::new(n, x);
    for b in [2, 1, 0] {
        let n_q = q.quarks(b);
        let sector = q.sector(n_q).expect("sector");
        let unpen = Qcd2::new(n, x);
        let mpo = { let mut u = unpen; u.lam = 0.0; u.mpo(n_q) };
        let (t0, l0) = random_start(&sector, 256, 7);
        report(&format!("N={n} x={x} B={b} random start   "), &t0, &l0, &mpo, &sector);
        if sweeps > 0 {
            let cfg = SymConfig::amendment(chi, sweeps);
            let t = std::time::Instant::now();
            let (r, labels) = dmrg_sweep_sym(&mpo, t0, l0, &sector, &cfg).expect("sweep");
            report(&format!("N={n} x={x} B={b} after {sweeps} sw chi={chi} ({:.0}s, E {:.6})", t.elapsed().as_secs_f64(), r.energy), &r.tensors, &labels, &mpo, &sector);
        }
    }
}
