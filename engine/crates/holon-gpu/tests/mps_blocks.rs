//! E14 item 5b's gate: the device two-site operator is the host reference TO THE BIT on
//! every bond of every N = 6 sector and the N = 8 random start, reproduces itself run to
//! run, and a whole labelled sweep on the device backend lands on the host sweep's energy
//! bit for bit — the standard E13 met for the determinant solver, unchanged.

use holon_gpu::mps_blocks::GpuTwoSite;
use q8_mps::blocks::{BlockPlan, CompactPlan};
use q8_mps::mps::{self, Env};
use q8_mps::qcd2::Qcd2;
use q8_mps::symmetric::{charge_add, random_start, SymConfig};
use std::sync::Arc;

fn envs(t: &[mps::TensorSite], mpo: &q8_mps::mpo::Mpo) -> (Vec<Env>, Vec<Env>) {
    let l = t.len();
    let mut lefts = vec![mps::trivial_left_env_mpo(mpo.sites[0].d_l)];
    for j in 0..l {
        let g = mps::grow_left_mpo(&lefts[j], &mpo.sites[j], &t[j]);
        lefts.push(g);
    }
    let mut rights: Vec<Env> = vec![mps::trivial_right_env_mpo(mpo.sites[l - 1].d_r); l + 1];
    for k in (0..l).rev() {
        rights[k] = mps::grow_right_mpo(&rights[k + 1], &mpo.sites[k], &t[k]);
    }
    (lefts, rights)
}

#[test]
fn the_device_two_site_operator_is_the_host_reference_to_the_bit() {
    let gpu = GpuTwoSite::new(0, 512).expect("no CUDA device");
    let mut bonds = 0usize;
    for (n, x, bs) in [(6usize, 4.0f64, vec![0i32, 1, 2]), (8, 4.0, vec![0, 1, 2])] {
        let q = Qcd2::new(n, x);
        for b in bs {
            let n_q = q.quarks(b);
            let sector = q.sector(n_q).unwrap();
            let mpo = { let mut u = Qcd2::new(n, x); u.lam = 0.0; u.mpo(n_q) };
            let (tensors, labels) = random_start(&sector, 256, 11);
            let (lefts, rights) = envs(&tensors, &mpo);
            let mut st = 0x5eed_0000u64 ^ (n as u64) << 8 ^ (b as u64);
            let mut rnd = || { st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((st >> 11) as f64) / ((1u64 << 53) as f64) - 0.5 };
            for j in 0..(tensors.len() - 1) {
                let (q_l, q_r) = (&labels[j], &labels[j + 2]);
                let (e1, e2) = (sector.site_charge[j], sector.site_charge[j + 1]);
                let (chi_l, chi_r) = (q_l.len(), q_r.len());
                let mut psi = vec![0.0; chi_l * 4 * chi_r];
                for (li, &ql) in q_l.iter().enumerate() { for a in 0..2 { let qa = if a == 1 { charge_add(ql, e1) } else { ql }; for bb in 0..2 { let qab = if bb == 1 { charge_add(qa, e2) } else { qa }; for (ri, &qr) in q_r.iter().enumerate() { if qr == qab { psi[((li * 2 + a) * 2 + bb) * chi_r + ri] = rnd(); } } } } }
                let (w1, w2) = (&mpo.sites[j], &mpo.sites[j + 1]);
                let plan = BlockPlan::build(q_l, q_r, e1, e2, &lefts[j], &rights[j + 2]).unwrap();
                let compact = CompactPlan::build(&plan, w1, w2).unwrap();
                let host = compact.apply_reference(&psi);
                let dense = mps::apply_effective_h_mpo(&lefts[j], w1, w2, &rights[j + 2], &psi, chi_l, chi_r);
                let mut dp = gpu.upload(&compact).expect("upload");
                let dev = gpu.apply(&mut dp, &psi).expect("launch");
                let dev2 = gpu.apply(&mut dp, &psi).expect("launch");
                let m_host = host.iter().zip(&dev).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
                let m_dense = dense.iter().zip(&dev).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
                let m_rerun = dev.iter().zip(&dev2).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
                assert_eq!(m_host, 0, "N={n} B={b} bond {j}: device differs from the host reference on {m_host} of {} entries", host.len());
                assert_eq!(m_dense, 0, "N={n} B={b} bond {j}: device differs from the DENSE operator on {m_dense} entries");
                assert_eq!(m_rerun, 0, "N={n} B={b} bond {j}: the device does not reproduce itself ({m_rerun} entries)");
                bonds += 1;
            }
            println!("N={n} B={b}: every bond bit-identical, device vs host reference vs dense");
        }
    }
    assert!(bonds >= 100, "only {bonds} bonds checked");
}

#[test]
fn a_whole_sweep_on_the_device_backend_lands_on_the_host_sweep_to_the_bit() {
    let gpu = Arc::new(GpuTwoSite::new(0, 512).expect("no CUDA device"));
    for (n, b, chi, e_ref) in [(4usize, 0i32, 64usize, -24.5391166860f64), (6, 2, 64, -19.1570928549)] {
        let q = Qcd2::new(n, 4.0);
        let n_q = q.quarks(b);
        let sector = q.sector(n_q).unwrap();
        let host_cfg = SymConfig::amendment(chi, 40);
        let (rh, _) = q.ground_energy_sym_from(&[], n_q, &host_cfg, Some(random_start(&sector, 256, 7))).expect("host sweep");
        let mut dev_cfg = host_cfg.clone();
        dev_cfg.backend = Some(gpu.clone());
        let (rd, _) = q.ground_energy_sym_from(&[], n_q, &dev_cfg, Some(random_start(&sector, 256, 7))).expect("device sweep");
        assert_eq!(rh.energy.to_bits(), rd.energy.to_bits(), "N={n} B={b}: host {:.15} vs device {:.15}", rh.energy, rd.energy);
        assert_eq!(rh.lanczos_iterations_total, rd.lanczos_iterations_total, "N={n} B={b}: a different Lanczos count means different bits somewhere");
        assert!((rd.energy - e_ref).abs() <= 1e-8, "N={n} B={b}: device sweep {:.10} vs exact {e_ref:.10}", rd.energy);
        let refusals = gpu.refusals.lock().unwrap().len();
        assert_eq!(refusals, 0, "the device refused {refusals} plans and fell back to the host — the test proved nothing about the device there");
        println!("N={n} B={b} chi={chi}: host and device sweeps identical to the bit at {:.12} ({} Lanczos its, 0 refusals)", rd.energy, rd.lanczos_iterations_total);
    }
}
