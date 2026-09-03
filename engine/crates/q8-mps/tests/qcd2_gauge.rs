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
