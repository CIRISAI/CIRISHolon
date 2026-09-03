//! THE DECISIVE TEST. Hubig–Haegeman–Schollwöck: the two-site variance is EXACT for a
//! nearest-neighbour Hamiltonian. `Mpo::from_hubbard` is nearest-neighbour (hopping between
//! adjacent orbitals, on-site U and mu), so whichever implementation reproduces the exact
//! variance on it is the correct one.
use q8_mps::dmrg::{dmrg_sweep, DmrgConfig, RefusalPolicy};
use q8_mps::mpo::Mpo;
use q8_mps::mps::TensorSite;

fn main() {
    for (sites, chi) in [(6usize, 2usize), (6, 4), (8, 4), (8, 8)] {
        // a TRUE chain-local Hamiltonian: hopping between ADJACENT MPS sites and an on-site
        // potential, nothing further than one bond. This is the regime where the paper says
        // the two-site variance is EXACT, so it is the test that decides the implementation.
        let l = sites;
        let mut bld = q8_mps::mpo::MpoBuilder::new(l);
        for i in 0..l {
            bld.add_term_factors(&[(i, true), (i, false)], 0.5 + 0.1 * i as f64);
        }
        for i in 0..(l - 1) {
            bld.add_term_factors(&[(i, true), (i + 1, false)], -1.0);
            bld.add_term_factors(&[(i + 1, true), (i, false)], -1.0);
            bld.add_term_factors(&[(i, true), (i, false), (i + 1, true), (i + 1, false)], 2.0);
        }
        let mpo = bld.build();
        let _ = Mpo::from_hubbard;
        // a truncated state: sweep at small chi so the variance is genuinely nonzero
        let mut st = 12345u64;
        let mut rnd = || { st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((st >> 11) as f64) / ((1u64 << 53) as f64) - 0.5 };
        let mut tensors: Vec<TensorSite> = Vec::new();
        let mut prev = 1usize;
        for j in 0..l {
            let next = if j + 1 == l { 1 } else { chi };
            let mut t = TensorSite::zeros(prev, next);
            for s in 0..2 { for l in 0..prev { for r in 0..next { t.set(s, l, r, rnd()); } } }
            tensors.push(t);
            prev = next;
        }
        let cfg = DmrgConfig { chi_max: chi, max_sweeps: 30, sweep_tol: 1e-12, policy: RefusalPolicy::Silent };
        let r = dmrg_sweep(&mpo, tensors, &cfg).expect("sweep");
        let norm = q8_mps::observables::norm_squared(&r.tensors);
        let t: Vec<TensorSite> = r.tensors.iter().map(|x| { let mut y = x.clone(); if std::ptr::eq(x, &r.tensors[0]) { for v in y.data.iter_mut() { *v /= norm.sqrt(); } } y }).collect();
        let (e, h2, exact) = q8_mps::variance::energy_variance(&t, &mpo).expect("small enough");
        let (d2s, one, two) = q8_mps::variance2::two_site_variance(&t, &mpo);
        println!("sites {sites} (L={l}) chi {chi}: E {e:.9} <H2> {h2:.6}  exact var {exact:.9e}  two-site {d2s:.9e} (1s {one:.3e} + 2s {two:.3e})  ratio {:.6}", d2s / exact);
    }
}
