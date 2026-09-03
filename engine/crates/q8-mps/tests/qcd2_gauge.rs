//! GF2a gate G0's engine half: the accumulator MPO is the same operator as the
//! determinant solver's integral tensor — dense spectrum against the FCI referee at N = 4
//! (numbers from `holon-chem`'s `examples/qcd2` on 2026-09-02, plant (i) passed there), and
//! the sweep reaching them.

use q8_mps::qcd2::Qcd2;

/// FCI referees at x = 4, N = 4: E0(B=0) with 6 quarks, E0(B=1) with 9 quarks.
const REFEREE: [(i32, usize, f64); 2] = [(0, 6, -24.5391166860), (1, 9, -17.5847761876)];

/// Lowest eigenvalue of the dense MPO restricted to `n_q` occupied modes, by power
/// iteration on the shifted operator (dim 924 and 220 here).
fn lowest_in_sector(h: &[f64], dim_full: usize, n_q: usize, diagonal_shift: f64) -> f64 {
    let states: Vec<usize> = (0..dim_full).filter(|b| b.count_ones() as usize == n_q).collect();
    let dim = states.len();
    let mut sub = vec![0.0; dim * dim];
    for (i, &bi) in states.iter().enumerate() {
        for (j, &bj) in states.iter().enumerate() {
            sub[i * dim + j] = h[bi * dim_full + bj] + if i == j { diagonal_shift } else { 0.0 };
        }
    }
    // symmetry check: the MPO must be Hermitian to roundoff
    let mut asym = 0.0f64;
    for i in 0..dim {
        for j in 0..dim {
            asym = asym.max((sub[i * dim + j] - sub[j * dim + i]).abs());
        }
    }
    assert!(asym <= 1e-12, "the dense MPO is not symmetric: {asym:e}");
    let shift = sub.iter().map(|v| v.abs()).sum::<f64>() / dim as f64 * 4.0 + 1.0;
    let mut v = vec![1.0 / (dim as f64).sqrt(); dim];
    let mut lam = 0.0;
    for _ in 0..50000 {
        let mut w = vec![0.0; dim];
        for i in 0..dim {
            let mut acc = 0.0;
            for j in 0..dim {
                acc += (if i == j { shift } else { 0.0 } - sub[i * dim + j]) * v[j];
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
fn the_accumulator_mpo_is_the_fci_tensor_at_four_sites() {
    let q = Qcd2::new(4, 4.0);
    let dim = 1usize << q.sites();
    for (b, n_q, e_ref) in REFEREE {
        let dense = q.mpo(n_q).dense();
        // the MPO carries the penalty without its constant; inside the sector it is exactly
        // −λ n_q² on the diagonal, put back BEFORE the power iteration (a constant of that size
        // would flatten its convergence rate to nothing)
        let e = lowest_in_sector(&dense, dim, n_q, q.lam * (n_q as f64) * (n_q as f64));
        assert!((e - e_ref).abs() <= 1e-8, "B={b}: dense MPO {e:.10} vs FCI {e_ref:.10}");
    }
}

// ---------------------------------------------------------------- E7: the symmetric arm (amendment A1)

#[test]
fn the_symmetric_sweep_reaches_the_referees_at_four_sites_with_no_penalty() {
    // plants (i)–(iii) re-run on the successor at N = 4 (A1.5): the labelled sweep on the
    // UNPENALISED Hamiltonian lands on the frozen referees, and its energy carries no λ n_q²
    let q = Qcd2::new(4, 4.0);
    for (b, _, e_ref) in REFEREE {
        let (r, labels) = q.ground_energy_sym(b, 64, 40, false).expect("symmetric sweep");
        assert!((r.energy - e_ref).abs() <= 1e-8, "B={b}: symmetric {:.10} vs FCI {e_ref:.10}", r.energy);
        assert!(r.converged, "B={b}: not converged by the amendment's test after {} sweeps", r.sweeps_used);
        // every bond state carries a definite label and the boundary carries the sector
        let sector = q.sector(q.quarks(b)).unwrap();
        assert_eq!(labels[labels.len() - 1], vec![sector.total]);
        assert!(labels.iter().all(|l| l.iter().all(|c| *c != q8_mps::symmetric::NO_CHARGE)));
    }
}

#[test]
fn plant_v_a_start_outside_the_sector_is_refused_by_name() {
    let q = Qcd2::new(4, 4.0);
    let n_q = q.quarks(0);
    // six quarks, but four red and one each green and blue: the same count, the wrong block
    let mut occ = vec![false; q.sites()];
    for (k, o) in occ.iter_mut().enumerate() {
        *o = matches!(k, 0 | 3 | 6 | 9 | 1 | 2);
    }
    let cfg = q8_mps::symmetric::SymConfig::amendment(64, 10);
    match q.ground_energy_sym_from(&occ, n_q, &cfg, None) {
        Err(q8_mps::symmetric::SymRefusal::StartOutsideSector { start, total }) => {
            assert_eq!(start[0], 4);
            assert_eq!(total, [2, 2, 2, 0]);
        }
        other => panic!("a start outside the sector was not refused by name: {:?}", other.map(|r| r.0.energy)),
    }
}

#[test]
fn the_symmetric_sweep_at_six_sites_lands_on_the_exact_referee() {
    // B = 2 at N = 6 is 216 determinants on the lanes (exact −19.1570928549 at x = 4); the
    // labelled sweep at χ = 64 holds the whole sector, so it must reproduce the exact energy
    let q = Qcd2::new(6, 4.0);
    let (r, _) = q.ground_energy_sym(2, 64, 40, false).expect("symmetric sweep");
    assert!((r.energy - (-19.1570928549)).abs() <= 1e-8, "N=6 B=2: symmetric {:.10} vs exact -19.1570928549", r.energy);
    // B = 0 at N = 6 is 8,000 determinants: the χ = 64 truncation is real here, and the
    // amendment's gate is staked at N = 8 on the ladder, not here. This prints the miss and
    // requires only that the labelled sweep is not the retired arm's 0.62 at χ = 40.
    let (r0, _) = q.ground_energy_sym(0, 64, 8, false).expect("symmetric sweep");
    let miss = r0.energy - (-38.2128396646);
    println!("N=6 B=0 x=4: symmetric chi=64 E={:.10} (exact -38.2128396646, miss {:+.3e}, converged {}, sweeps {})", r0.energy, miss, r0.converged, r0.sweeps_used);
    assert!(miss.abs() < 1e-2, "the labelled sweep misses the exact energy by {miss:+.3e} at chi=64");
}

#[test]
fn the_sweep_reaches_the_fci_ground_state_at_four_sites() {
    let q = Qcd2::new(4, 4.0);
    for (b, _, e_ref) in REFEREE {
        let res = q.ground_energy(b, 64, 120, 1e-12).expect("sweep");
        assert!((res.energy - e_ref).abs() <= 1e-8, "B={b}: engine {:.10} vs FCI {e_ref:.10}", res.energy);
        // the local residual is absolute in an operator whose norm carries the penalty scale
        assert!(res.worst_lanczos_residual <= 1e-9 * (1.0 + q.lam * 81.0), "residual {:e}", res.worst_lanczos_residual);
    }
}

// ---------------------------------------------------------------- E14 item 1: the block-sparse operator

/// The block-sparse two-site operator is the dense one TO THE BIT on every bond of every
/// N = 6 sector and on the N = 8 random start, on random label-consistent vectors — and it
/// omits most of the work: the live fraction is printed beside each bond.
#[test]
fn the_block_sparse_operator_is_the_dense_operator_to_the_bit() {
    use q8_mps::blocks::BlockPlan;
    use q8_mps::mps;
    use q8_mps::symmetric::{charge_add, random_start};
    let mut worst_live = 1.0f64;
    let mut checked = 0usize;
    for (n, x, bs) in [(6usize, 4.0f64, vec![0i32, 1, 2]), (8, 4.0, vec![0, 1, 2])] {
        let q = Qcd2::new(n, x);
        for b in bs {
            let n_q = q.quarks(b);
            let sector = q.sector(n_q).unwrap();
            let mpo = { let mut u = Qcd2::new(n, x); u.lam = 0.0; u.mpo(n_q) };
            let (tensors, labels) = random_start(&sector, 256, 11);
            let l = tensors.len();
            // every left and right environment, as the sweep would hold them
            let mut lefts = vec![mps::trivial_left_env_mpo(mpo.sites[0].d_l)];
            for j in 0..l {
                let g = mps::grow_left_mpo(&lefts[j], &mpo.sites[j], &tensors[j]);
                lefts.push(g);
            }
            let mut rights: Vec<mps::Env> = vec![mps::trivial_right_env_mpo(mpo.sites[l - 1].d_r); l + 1];
            for k in (0..l).rev() {
                rights[k] = mps::grow_right_mpo(&rights[k + 1], &mpo.sites[k], &tensors[k]);
            }
            let mut st = 0x1234_5678_9abc_def0u64 ^ (n as u64) << 8 ^ (b as u64);
            let mut rnd = || {
                st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                ((st >> 11) as f64) / ((1u64 << 53) as f64) - 0.5
            };
            for j in 0..(l - 1) {
                let (q_l, q_r) = (&labels[j], &labels[j + 2]);
                let (e1, e2) = (sector.site_charge[j], sector.site_charge[j + 1]);
                let (chi_l, chi_r) = (q_l.len(), q_r.len());
                // a random vector on the live entries only
                let mut psi = vec![0.0; chi_l * 4 * chi_r];
                for (li, &ql) in q_l.iter().enumerate() {
                    for a in 0..2 {
                        let qa = if a == 1 { charge_add(ql, e1) } else { ql };
                        for bb in 0..2 {
                            let qab = if bb == 1 { charge_add(qa, e2) } else { qa };
                            for (ri, &qr) in q_r.iter().enumerate() {
                                if qr == qab {
                                    psi[((li * 2 + a) * 2 + bb) * chi_r + ri] = rnd();
                                }
                            }
                        }
                    }
                }
                let (left, right) = (&lefts[j], &rights[j + 2]);
                let plan = BlockPlan::build(q_l, q_r, e1, e2, left, right).expect("the QCD2 MPO conserves colour, so every channel has one shift");
                let dense = mps::apply_effective_h_mpo(left, &mpo.sites[j], &mpo.sites[j + 1], right, &psi, chi_l, chi_r);
                let sparse = plan.apply(left, &mpo.sites[j], &mpo.sites[j + 1], right, &psi);
                let mismatched = dense.iter().zip(&sparse).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
                assert_eq!(mismatched, 0, "N={n} B={b} bond {j}: block-sparse differs from dense on {mismatched} of {} entries", dense.len());
                assert_eq!(plan.live_left(), mps::live_channels(left), "N={n} B={b} bond {j}: live left channels");
                assert_eq!(plan.live_right(), mps::live_channels(right), "N={n} B={b} bond {j}: live right channels");
                let live = plan.live_entries() as f64 / psi.len() as f64;
                worst_live = worst_live.min(live);
                checked += 1;
            }
            println!("N={n} B={b}: {} bonds bit-identical; worst live fraction so far {:.2}%", l - 1, 100.0 * worst_live);
        }
    }
    assert!(checked >= 40, "only {checked} bonds checked");
    assert!(worst_live < 0.05, "the live fraction never fell below 5% — the sparsity this operator exists for is not being exercised");
}

// ---------------------------------------------------------------- E14 item 2: what moved

/// The change instrumentation is a MEASUREMENT of the sweep, not a change to it: with
/// `skip_unmoved` off every gate above is untouched (they ran on this build), and the kept
/// block masses plus the discarded weight of every bond sum to the state's norm. With
/// `skip_unmoved` ON, on a sector the arm solves exactly, bonds ARE skipped once they stop
/// moving and the energy still lands on the exact referee.
#[test]
fn the_change_instrumentation_measures_and_skipping_unmoved_bonds_keeps_the_referee() {
    let q = Qcd2::new(6, 4.0);
    let (r, _) = q.ground_energy_sym(2, 64, 40, false).expect("symmetric sweep");
    assert_eq!(r.bond_energy.len(), q.sites() - 1);
    assert_eq!(r.block_mass.len(), q.sites() - 1);
    for (j, (mass, dw)) in r.block_mass.iter().zip(&r.discarded_weight).enumerate() {
        let kept: f64 = mass.iter().map(|m| m.1).sum();
        assert!((kept + dw - 1.0).abs() <= 1e-9, "bond {j}: kept block mass {kept} + discarded {dw} != 1");
        assert!(!mass.is_empty(), "bond {j}: no block carries mass");
    }
    assert!(r.bond_energy_delta.iter().all(|d| d.is_finite()), "after two or more sweeps every bond has a measured delta");
    assert!(r.site_delta.iter().any(|d| d.is_finite()), "some site kept its shape across the last sweep, so its delta is a number");
    assert_eq!(r.bonds_skipped, 0, "skip_unmoved is off by default and skipped nothing");

    // SKIPPING ON, gated as the mechanism it is: a skipped bond lags its neighbours by one
    // sweep, so on a sector that CREEPS (N = 6, B = 0 at χ = 64 truncates and drifts ~2e-6 per
    // sweep for sixty sweeps; under the amendment's test every sector here either converges
    // in the minimum four sweeps or never, measured on all four (b, χ) candidates), the
    // skipping arm after k sweeps must sit no higher than the full arm after k − 1 — one
    // sweep of lag and not more — while both stay variational, and it must have skipped, or
    // the option is decoration. The saving is printed, not asserted: it is what this buys.
    let n_q = q.quarks(0);
    let sector = q.sector(n_q).unwrap();
    let k = 16;
    let full_cfg = q8_mps::symmetric::SymConfig::amendment(64, k);
    let (rf, _) = q.ground_energy_sym_from(&[], n_q, &full_cfg, Some(q8_mps::symmetric::random_start(&sector, 256, 7))).expect("full sweep");
    let mut cfg = full_cfg.clone();
    cfg.skip_unmoved = true;
    let (rs, _) = q.ground_energy_sym_from(&[], n_q, &cfg, Some(q8_mps::symmetric::random_start(&sector, 256, 7))).expect("symmetric sweep, skipping");
    assert_eq!(rf.sweeps_used, k, "the creeping sector converged, which it was not supposed to; pick another instance");
    assert_eq!(rs.sweeps_used, k);
    let tol = 10.0 * cfg.rtol * rf.energy.abs().max(1.0);
    // THE MEASUREMENT, pinned: skipping stays variational (never below the full arm's answer),
    // stays BOUNDED (its excursion above the full arm is under 1e-5), and actually skips — and
    // the excursion is printed, because it is the reason the option ships OFF.
    let below = rs.energy_history.iter().cloned().fold(f64::INFINITY, f64::min);
    assert!(below >= rf.energy - tol, "the skipping arm went BELOW the full arm's converged energy: {below:.10} vs {:.10} — a non-variational step", rf.energy);
    // the excursion is skipping's cost AGAINST THE FULL ARM AT THE SAME SWEEP, once both are
    // past the amendment's minimum — not against the final answer from sweep one
    let excursion = rs.energy_history.iter().zip(&rf.energy_history).skip(cfg.min_sweeps).map(|(a, b)| a - b).fold(0.0f64, f64::max);
    assert!(excursion < 1e-5, "skipping's excursion above the full arm is {excursion:.2e}: no longer the bounded 3.7e-6 that was measured");
    let rises = rs.energy_history.windows(2).filter(|w| w[1] > w[0] + tol).count();
    assert!(rs.bonds_skipped > 0, "skip_unmoved never skipped a bond on a creeping sector — the instrument measured nothing");
    println!(
        "N=6 B=0 chi=64, {k} sweeps: full E {:.10} ({} Lanczos its); skipping ({} its, {} of {} bond visits skipped over the run) sits at most {excursion:.2e} above it and rose between sweeps {rises} times — the truncation fixed point perturbed; Lanczos saved {:.0}%, and the option stays OFF",
        rf.energy, rf.lanczos_iterations_total, rs.lanczos_iterations_total, rs.bonds_skipped, 2 * (q.sites() - 1) * k,
        100.0 * (1.0 - rs.lanczos_iterations_total as f64 / rf.lanczos_iterations_total as f64)
    );
}

// ---------------------------------------------------------------- E14 item 4: the variance

fn mps_to_dense(tensors: &[q8_mps::mps::TensorSite]) -> Vec<f64> {
    // amplitude of every basis state |s_0 ... s_{L-1}>, bit j of the index = s_j
    let l = tensors.len();
    let mut out = vec![0.0; 1 << l];
    for idx in 0..(1usize << l) {
        let mut v = vec![1.0f64];
        for (j, t) in tensors.iter().enumerate() {
            let s = (idx >> j) & 1;
            let mut nv = vec![0.0; t.chi_r];
            for (li, &vl) in v.iter().enumerate() {
                if vl == 0.0 { continue; }
                for r in 0..t.chi_r {
                    nv[r] += vl * t.get(s, li, r);
                }
            }
            v = nv;
        }
        out[idx] = v[0];
    }
    out
}

/// The exact variance through the squared MPO equals the dense `⟨H²⟩ − ⟨H⟩²` on a random
/// labelled state (the operator identity), is at the residual's own scale on a converged
/// state (an eigenstate's variance is zero), and is REFUSED by name above its lease.
#[test]
fn the_energy_variance_is_the_dense_one_and_vanishes_on_the_eigenstate() {
    use q8_mps::symmetric::random_start;
    use q8_mps::variance::{energy_variance, expectation_mpo, price_bytes, square};
    let q = Qcd2::new(4, 4.0);
    let (b, n_q, e_ref) = REFEREE[0];
    let sector = q.sector(n_q).unwrap();
    let mpo = { let mut u = Qcd2::new(4, 4.0); u.lam = 0.0; u.mpo(n_q) };
    let dim = 1usize << q.sites();
    let h = mpo.dense();
    // (i) a random labelled state: the MPO route is the dense route
    let (t, _) = random_start(&sector, 256, 5);
    let psi = mps_to_dense(&t);
    let norm: f64 = psi.iter().map(|x| x * x).sum();
    let hpsi: Vec<f64> = (0..dim).map(|i| (0..dim).map(|j| h[i * dim + j] * psi[j]).sum()).collect();
    let e_dense: f64 = psi.iter().zip(&hpsi).map(|(a, b)| a * b).sum::<f64>() / norm;
    let h2_dense: f64 = hpsi.iter().map(|x| x * x).sum::<f64>() / norm;
    let (e_mpo, h2_mpo, var_mpo) = energy_variance(&t, &mpo).expect("within the lease at N=4");
    assert!((e_mpo - e_dense).abs() <= 1e-9 * e_dense.abs().max(1.0), "<H>: mpo {e_mpo} vs dense {e_dense}");
    assert!((h2_mpo - h2_dense).abs() <= 1e-9 * h2_dense.abs().max(1.0), "<H^2>: mpo {h2_mpo} vs dense {h2_dense}");
    let var_dense = h2_dense - e_dense * e_dense;
    assert!((var_mpo - var_dense).abs() <= 1e-9 * var_dense.abs().max(1.0), "variance: mpo {var_mpo} vs dense {var_dense}");
    assert!(var_mpo > 1.0, "a random state's variance is order one, not {var_mpo}");
    // the squared MPO is H² on the dense side too
    let h2 = square(&mpo);
    assert_eq!(h2.sites[0].d_l, mpo.sites[0].d_l * mpo.sites[0].d_l);
    let raw = expectation_mpo(&t, &h2);
    assert!((raw / norm - h2_dense).abs() <= 1e-9 * h2_dense.abs().max(1.0));
    // (ii) the converged state: variance at the residual's scale, energy on the referee
    let (r, _) = q.ground_energy_sym(b, 64, 40, false).expect("symmetric sweep");
    let (e, _, var) = energy_variance(&r.tensors, &mpo).expect("within the lease");
    assert!((e - e_ref).abs() <= 1e-8, "converged <H> {e:.10} vs referee {e_ref:.10}");
    assert!(var.abs() <= 1e-12, "the converged state's variance is {var:.3e}, not at the residual's scale");
    println!("N=4 B={b}: random state variance {var_mpo:.6} (dense {var_dense:.6}); converged state variance {var:.3e} at energy {e:.10}");
    // (iii) the price is refused by name above the lease
    let price = price_bytes(&t, &mpo);
    assert!(price > 0);
    std::env::set_var("Q8_VARIANCE_LEASE_BYTES", (price / 2).to_string());
    let refused = energy_variance(&t, &mpo);
    std::env::remove_var("Q8_VARIANCE_LEASE_BYTES");
    match refused {
        Err(e) => assert!(e.price_bytes == price && e.lease_bytes == price / 2, "{e}"),
        Ok(_) => panic!("a price above the lease was not refused"),
    }
}
