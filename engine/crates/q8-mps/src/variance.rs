//! THE ENERGY VARIANCE `⟨H²⟩ − ⟨H⟩²` — E14 item 4, the arm's own error bar.
//!
//! `Q10_PREREG.md` §5 says what the discarded weight is NOT (production, never error), and
//! E7 measured it: across twelve rungs the ratio (energy miss ÷ discarded weight) drifted
//! four orders, and the local Lanczos residual sat at its own stopping tolerance on the right
//! answer and on one wrong by 5.9e-3 alike (`GF2A_QCD2_RESULTS.md`, the misfit reading).
//! The variance is the quantity that IS an error bar: zero for an eigenstate, and for a
//! variational state it bounds the energy error from above through the gap. It is the only
//! number the arm can compute about itself at a volume where no exact referee exists.
//!
//! Computed EXACTLY, by the MPO of `H²` contracted through the same environment machinery
//! the sweep uses (`mps::grow_left_mpo`): the squared MPO's site tensor is
//! `W²[(a₁a₂),(b₁b₂),s,s''] = Σ_{s'} W[a₁,b₁,s,s'] · W[a₂,b₂,s',s'']`, `d²` channels, with
//! `START = (START,START)` at pair index 0 and `FINISH = (FINISH,FINISH)` at the last pair
//! index — the same corners the single MPO's boundary vectors select, so the boundary
//! conventions carry over untouched. Nothing here is approximated.
//!
//! PRICED, not assumed: the doubled environment is `d²·χ²` doubles and the growth's scratch
//! twice that, so at `d = 42` (the QCD₂ accumulator) and `χ = 256` the peak is ~3.7 GB, and
//! at `χ = 1024` it is ~60 GB. The price is computed before a byte is allocated and refused
//! by name above the lease (`Q8_VARIANCE_LEASE_BYTES`, default 8 GiB). The named exit when
//! the price bites is the two-site variance of Hubig, Haegeman and Schollwöck (Phys. Rev. B
//! 97, 045125, 2018), `O(D·χ³)` with no doubled environment — cited, not built.

use crate::mpo::{Mpo, MpoSite};
use crate::mps::{self, TensorSite};

/// The MPO of `H²`, `d²` channels per bond.
pub fn square(mpo: &Mpo) -> Mpo {
    let sites = mpo
        .sites
        .iter()
        .map(|w| {
            let (dl, dr) = (w.d_l, w.d_r);
            let mut data = vec![0.0; dl * dl * dr * dr * 4];
            for a1 in 0..dl {
                for b1 in 0..dr {
                    for a2 in 0..dl {
                        for b2 in 0..dr {
                            for s in 0..2 {
                                for spp in 0..2 {
                                    let mut acc = 0.0;
                                    for sp in 0..2 {
                                        acc += w.get(a1, b1, s, sp) * w.get(a2, b2, sp, spp);
                                    }
                                    if acc != 0.0 {
                                        let cl = a1 * dl + a2;
                                        let cr = b1 * dr + b2;
                                        data[((cl * dr * dr + cr) * 2 + s) * 2 + spp] = acc;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            MpoSite::new(dl * dl, dr * dr, data)
        })
        .collect();
    Mpo { sites }
}

/// `⟨ψ|O|ψ⟩` for any MPO, RAW (not divided by `⟨ψ|ψ⟩`): the left environment grown across
/// every site, read at the FINISH channel on the trivial right boundary.
pub fn expectation_mpo(tensors: &[TensorSite], mpo: &Mpo) -> f64 {
    assert_eq!(tensors.len(), mpo.sites.len());
    let mut env = mps::trivial_left_env_mpo(mpo.sites[0].d_l);
    for (t, w) in tensors.iter().zip(&mpo.sites) {
        env = mps::grow_left_mpo(&env, w, t);
    }
    let last = &mpo.sites[mpo.sites.len() - 1];
    assert_eq!(tensors[tensors.len() - 1].chi_r, 1, "the last site must close on a trivial bond");
    env[last.d_r - 1][0]
}

/// The peak bytes the exact variance will allocate: the doubled environment plus the
/// growth's two scratch tensors, at the largest bond of the state.
pub fn price_bytes(tensors: &[TensorSite], mpo: &Mpo) -> u64 {
    let chi = tensors.iter().map(|t| t.chi_l.max(t.chi_r)).max().unwrap_or(1) as u64;
    let d = mpo.sites.iter().map(|w| w.d_l.max(w.d_r)).max().unwrap_or(1) as u64;
    let d2 = d * d;
    // env: d² channels of χ² doubles; grow_left_mpo scratch: tmp_a and tmp_b, each d²·2·χ² doubles
    (d2 * chi * chi + 2 * d2 * 2 * chi * chi) * 8
}

/// The lease the exact variance may spend, bytes: `Q8_VARIANCE_LEASE_BYTES` or 8 GiB.
pub fn lease_bytes() -> u64 {
    std::env::var("Q8_VARIANCE_LEASE_BYTES").ok().and_then(|v| v.parse().ok()).unwrap_or(8u64 << 30)
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarianceRefused {
    pub price_bytes: u64,
    pub lease_bytes: u64,
}

impl std::fmt::Display for VarianceRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the exact variance would allocate {:.2} GB against a lease of {:.2} GB: refused, not \
             estimated — the two-site variance (Hubig–Haegeman–Schollwöck 2018) is the named exit",
            self.price_bytes as f64 / 1e9,
            self.lease_bytes as f64 / 1e9
        )
    }
}

/// `⟨H⟩`, `⟨H²⟩` and the variance `⟨H²⟩ − ⟨H⟩²` of a normalised-or-not MPS under `mpo`,
/// exact, or a refusal naming the price.
pub fn energy_variance(tensors: &[TensorSite], mpo: &Mpo) -> Result<(f64, f64, f64), VarianceRefused> {
    let price = price_bytes(tensors, mpo);
    let lease = lease_bytes();
    if price > lease {
        return Err(VarianceRefused { price_bytes: price, lease_bytes: lease });
    }
    let norm = crate::observables::norm_squared(tensors);
    let h = expectation_mpo(tensors, mpo) / norm;
    let h2 = expectation_mpo(tensors, &square(mpo)) / norm;
    Ok((h, h2, h2 - h * h))
}
