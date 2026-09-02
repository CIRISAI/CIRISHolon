//! GF2a plants (i), (ii), (iii): the axial-gauge QCD₂ integrals against an INDEPENDENT dense
//! Hamiltonian built from the operator definition over Fock states — no integral tensor
//! anywhere in the referee — and the two mutations the freeze names.

use holon_chem::qcd2::{Mutation, Qcd2, COLOURS};

/// `Σ_a (Σ_{k≤n} q^a_k)²` and the hopping, applied to Fock states `|b⟩` (bit `3k+c` set iff
/// quark of colour `c` on site `k`), with `q^a_k = Σ_{cc'} T^a_{cc'} ψ†_{k,c} ψ_{k,c'}` written
/// out through the Fierz identity ONLY at the level of operator products — every fermionic
/// sign comes from the Jordan–Wigner string, never from the solver's conventions.
fn dense(q: &Qcd2, n_q: usize) -> (Vec<f64>, Vec<usize>) {
    let m = q.n_orb();
    let states: Vec<usize> = (0..1usize << m).filter(|b| b.count_ones() as usize == n_q).collect();
    let index = |b: usize| states.binary_search(&b).expect("state in sector");
    let dim = states.len();
    let mut h = vec![0.0; dim * dim];
    // ψ†_p ψ_q on |b>: (sign, b')
    let hop = |b: usize, p: usize, qq: usize| -> Option<(f64, usize)> {
        if (b >> qq) & 1 == 0 {
            return None;
        }
        let sign_q = if ((b & ((1usize << qq) - 1)).count_ones()) % 2 == 0 { 1.0 } else { -1.0 };
        let b1 = b ^ (1 << qq);
        if (b1 >> p) & 1 == 1 {
            return None;
        }
        let sign_p = if ((b1 & ((1usize << p) - 1)).count_ones()) % 2 == 0 { 1.0 } else { -1.0 };
        Some((sign_p * sign_q, b1 | (1 << p)))
    };
    for (i, &b) in states.iter().enumerate() {
        // hopping
        for k in 0..q.n - 1 {
            for c in 0..COLOURS {
                let (p, s) = (COLOURS * k + c, COLOURS * (k + 1) + c);
                for (from, to) in [(s, p), (p, s)] {
                    if let Some((sg, b2)) = hop(b, to, from) {
                        h[index(b2) * dim + i] += q.x * sg;
                    }
                }
            }
        }
        // colour Coulomb: Σ_{k,k'} w_{kk'} Σ_a q^a_k q^a_{k'} with q^a_k q^a_{k'} = Σ F E_{kc,kc'} E_{k'd,k'd'}
        for k in 0..q.n {
            for kp in 0..q.n {
                let w = (q.n - 1 - k.max(kp)) as f64;
                if w == 0.0 {
                    continue;
                }
                for c in 0..COLOURS {
                    for cp in 0..COLOURS {
                        for d in 0..COLOURS {
                            for dp in 0..COLOURS {
                                let f = holon_chem::qcd2::fierz(c, cp, d, dp, q.mutation);
                                if f == 0.0 {
                                    continue;
                                }
                                // E_{k'd,k'd'} first, then E_{kc,kc'}
                                if let Some((s1, b1)) = hop(b, COLOURS * kp + d, COLOURS * kp + dp) {
                                    if let Some((s2, b2)) = hop(b1, COLOURS * k + c, COLOURS * k + cp) {
                                        h[index(b2) * dim + i] += w * f * s1 * s2;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    (h, states)
}

fn lowest(h: &[f64], dim: usize) -> f64 {
    // power iteration on (shift − H) for the lowest eigenvalue: dim ≤ 924 here, so plain
    // dense arithmetic is fine and needs no library
    let shift = h.iter().map(|v| v.abs()).sum::<f64>() / dim as f64 * 4.0 + 1.0;
    let mut v = vec![1.0 / (dim as f64).sqrt(); dim];
    let mut lam = 0.0;
    for _ in 0..20000 {
        let mut w = vec![0.0; dim];
        for i in 0..dim {
            let mut acc = 0.0;
            for j in 0..dim {
                acc += (if i == j { shift } else { 0.0 } - h[i * dim + j]) * v[j];
            }
            w[i] = acc;
        }
        let norm = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        for x in w.iter_mut() {
            *x /= norm;
        }
        let new_lam = shift - norm;
        let delta = (new_lam - lam).abs();
        lam = new_lam;
        v = w;
        if delta < 1e-13 {
            break;
        }
    }
    lam
}

#[test]
fn plant_i_the_dense_referee_at_four_sites() {
    let q = Qcd2::new(4, 4.0);
    let mut es = Vec::new();
    for b in [0, 1] {
        let n_q = q.quarks(b);
        let (h, states) = dense(&q, n_q);
        let e_dense = lowest(&h, states.len());
        let e_fci = q.ground(b).e.v;
        assert!((e_dense - e_fci).abs() <= 1e-9, "B={b}: dense {e_dense:.12} vs FCI {e_fci:.12}");
        es.push(e_fci);
    }
    assert!(es[1] - es[0] > 0.1, "carrier: a baryon costs energy; E1-E0 = {}", es[1] - es[0]);
}

#[test]
fn plant_ii_the_fierz_trace_mutation_moves_the_baryon_mass() {
    let q = Qcd2::new(4, 4.0);
    let m = q.ground(1).e.v - q.ground(0).e.v;
    let qm = Qcd2::new(4, 4.0).with_mutation(Mutation::FierzTraceOff);
    let mm = qm.ground(1).e.v - qm.ground(0).e.v;
    assert!((m - mm).abs() > 1e-3, "the planted Fierz defect did not move E1-E0: {m} vs {mm}");
    // and the dense referee sees the mutated tensor as the mutated operator (the plant is a
    // defect of the PHYSICS, and both builds agree it is what it is)
    let (h, states) = dense(&qm, qm.quarks(1));
    let e_dense = lowest(&h, states.len());
    assert!((e_dense - qm.ground(1).e.v).abs() <= 1e-9);
}

#[test]
fn plant_iii_three_quarks_on_one_site_are_a_singlet_at_zero_hopping() {
    // x = 0: no hopping; the B = 1 ground state puts three quarks on one site as a colour
    // singlet, whose Coulomb energy is EXACTLY the sea's — E0(B=1) = E0(B=0).
    let q = Qcd2::new(4, 0.0);
    let e0 = q.ground(0).e.v;
    let e1 = q.ground(1).e.v;
    assert!((e1 - e0).abs() <= 1e-12, "E1-E0 at x=0 = {:e}", e1 - e0);
}
