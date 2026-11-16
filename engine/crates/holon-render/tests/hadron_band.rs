//! The sub-atom band's gates: the door IS the campaign's exact arm, the density it draws
//! conserves quark number, and every refusal fires by name rather than hanging the page.

use holon_render::hadron::*;
use std::sync::Mutex;

/// THE BANDS ARE ONE PROCESS-GLOBAL OBJECT (`hadron.rs::BANDS`, three slots), and five of
/// the gates below solve, step and grab slot 1. Rust runs a file's tests on parallel
/// threads, so without this lock two gates step one band at once and the energy-drift
/// gate reads the other's steps as its own drift — it fired exactly that way under a
/// loaded full-suite run (2026-09-05, `CHANNEL_LEDGER.md`) and passed alone. The gates
/// therefore take the band in turn. A poisoned lock (a gate that panicked while holding
/// it) is taken anyway: the next gate re-solves its own band before reading it.
static SERIAL: Mutex<()> = Mutex::new(());
fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

#[test]
fn the_band_is_the_exact_colour_lane_arm_to_the_bit() {
    let _band = serial();
    for (n, x) in [(4usize, 4.0f64), (6, 4.0), (6, 9.0)] {
        for b in 0..=2 {
            let q = holon_chem::qcd2::Qcd2::new(n, x);
            let n_q = q.quarks(b);
            if n_q % 3 != 0 || n_q / 3 > n {
                continue;
            }
            let referee = q.ground_lanes(b, 1);
            let code = holon_hadron_solve(n as u32, x, b);
            // 0 converged, 3 trivial — a one-determinant sector (N=4, B=2 is C(4,4)^3 = 1)
            // has nothing to iterate, and reporting that honestly is the exit's job
            assert!(code == 0 || code == 3, "N={n} x={x} B={b}: the door failed (code {code})");
            let e = holon_hadron_energy(b);
            assert_eq!(
                e.to_bits(),
                referee.energy.to_bits(),
                "N={n} x={x} B={b}: door {e:.12} vs the exact arm {:.12}",
                referee.energy
            );
            assert!(holon_hadron_margin(b) >= 0.0, "N={n} B={b}: negative variational margin");
            assert_eq!(holon_hadron_n_det(b) as usize, q.lane_space(b).n_det);
            assert_eq!(holon_hadron_dim_for(n as u32, b), holon_hadron_n_det(b), "the priced dimension is the real one");
            // the density it draws conserves quark number
            let total: f64 = (0..n).map(|k| holon_hadron_occ(b, k as u32)).sum();
            assert!(
                (total - n_q as f64).abs() <= 1e-9,
                "N={n} B={b}: the drawn density sums to {total:.9}, not the sector's {n_q} quarks"
            );
            for k in 0..n {
                let o = holon_hadron_occ(b, k as u32);
                assert!((-1e-12..=3.0 + 1e-12).contains(&o), "N={n} B={b} site {k}: density {o} outside [0, 3]");
            }
            assert!(holon_hadron_occ(b, n as u32).is_nan(), "off the end of the chain must be NaN, not a number");
        }
        // the mass needs BOTH sectors from the SAME box, and says nothing otherwise
        holon_hadron_solve(n as u32, x, 0);
        holon_hadron_solve(n as u32, x, 1);
        let m = holon_hadron_baryon_mass();
        let q = holon_chem::qcd2::Qcd2::new(n, x);
        let want = holon_chem::qcd2::Qcd2::baryon_mass(q.ground_lanes(0, 1).energy, q.ground_lanes(1, 1).energy, x);
        assert!((m - want).abs() <= 1e-12, "N={n} x={x}: baryon mass {m:.12} vs {want:.12}");
        println!("N={n} x={x}: E0 {:.9}  E1 {:.9}  baryon mass {m:.9}  ({} and {} determinants)",
            holon_hadron_energy(0), holon_hadron_energy(1), holon_hadron_n_det(0), holon_hadron_n_det(1));
    }
}

#[test]
fn every_refusal_fires_by_name_and_changes_nothing() {
    let _band = serial();
    // a good solve to have something to protect
    assert_eq!(holon_hadron_solve(4, 4.0, 1), 0);
    let good = holon_hadron_energy(1);
    // bad parameters: odd chain, too small, baryon number out of range, non-positive coupling
    for (n, x, b, why) in [(5u32, 4.0f64, 1i32, "odd chain"), (0, 4.0, 1, "empty chain"), (4, 4.0, 7, "no such sector"), (4, 0.0, 1, "zero coupling"), (4, -1.0, 1, "negative coupling")] {
        assert_eq!(holon_hadron_solve(n, x, b), 5, "{why} must be refused with code 5");
    }
    // too big: N=10 B=1 is 9.3 million determinants, far over the cap
    assert_eq!(holon_hadron_solve(10, 4.0, 1), 6, "a sector over the cap must be refused with code 6, not attempted");
    assert!(holon_hadron_dim_for(10, 1) > holon_hadron_max_det(), "the page can price the refusal before asking for it");
    // and none of that disturbed the good solve
    assert_eq!(holon_hadron_energy(1).to_bits(), good.to_bits(), "a refusal changed the stored band");
    // an unsolved sector reads as unsolved, never as zero
    assert_eq!(holon_hadron_exit(9), 4);
    assert!(holon_hadron_energy(9).is_nan(), "an out-of-range sector must read NaN, not 0.0");
    println!("refusals: bad parameters -> 5, over the {:.0}-determinant cap -> 6, unsolved -> NaN and exit 4", holon_hadron_max_det());
}

// ------------------------------------------------------------------ the grab and the dynamics

/// The propagator is UNITARY and the evolution CONSERVES ENERGY. Those two are what separate
/// real dynamics from a moving picture, and they are checked over many steps rather than one.
#[test]
fn the_evolution_is_unitary_and_conserves_energy() {
    let _band = serial();
    assert!(holon_hadron_solve(6, 4.0, 1) == 0);
    let e0 = holon_hadron_live_energy(1);
    assert!((e0 - holon_hadron_energy(1)).abs() <= 1e-9, "the live energy of the ground state is its energy");
    let (mut worst_n, mut worst_e) = (0.0f64, 0.0f64);
    for _ in 0..40 {
        assert_eq!(holon_hadron_step(1, 0.05), 0);
        worst_n = worst_n.max((holon_hadron_norm(1) - 1.0).abs());
        worst_e = worst_e.max((holon_hadron_live_energy(1) - e0).abs() / e0.abs());
    }
    println!("40 steps of dt=0.05: worst norm drift {worst_n:.2e}, worst relative energy drift {worst_e:.2e}");
    assert!(worst_n <= 1e-10, "the propagator is not unitary: norm drifted {worst_n:.3e}");
    assert!(worst_e <= 1e-10, "the evolution does not conserve energy: drifted {worst_e:.3e} relative");
    assert!((holon_hadron_time(1) - 2.0).abs() <= 1e-12, "elapsed model time is the sum of the steps");
}

/// AN EIGENSTATE DOES NOT MOVE. Evolving the ground state under its OWN Hamiltonian changes
/// only a phase, so the density this band draws must be unchanged — the control that says a
/// moving picture is not being mistaken for physics.
#[test]
fn the_ground_state_evolves_without_moving() {
    let _band = serial();
    assert_eq!(holon_hadron_solve(6, 4.0, 1), 0);
    let before: Vec<f64> = (0..6).map(|k| holon_hadron_occ(1, k)).collect();
    for _ in 0..20 {
        assert_eq!(holon_hadron_step(1, 0.1), 0);
    }
    let after: Vec<f64> = (0..6).map(|k| holon_hadron_occ(1, k)).collect();
    let worst = before.iter().zip(&after).map(|(a, b)| (a - b).abs()).fold(0.0f64, f64::max);
    println!("eigenstate after 20 steps: worst density change {worst:.2e} (a phase moves nothing)");
    assert!(worst <= 1e-10, "the ground state's density moved by {worst:.3e} under its own Hamiltonian");
}

/// THE GRAB DOES SOMETHING, AND THE RIGHT SOMETHING. Reach in, pull one site down, and the
/// quarks flow TOWARD it — the density at the grabbed site rises, and it is the ground state
/// of the pinned Hamiltonian that it is heading for. Releasing puts it back.
#[test]
fn grabbing_a_site_pulls_the_quarks_to_it() {
    let _band = serial();
    assert_eq!(holon_hadron_solve(6, 4.0, 1), 0);
    let site = 1u32;
    let before = holon_hadron_occ(1, site);
    assert_eq!(holon_hadron_pin_at(1, site), 0.0, "nothing is grabbed until someone grabs");
    // pull site 1 down hard: a one-body potential well, the same for every colour
    assert_eq!(holon_hadron_grab(1, site, -6.0), 0);
    assert_eq!(holon_hadron_pin_at(1, site), -6.0);
    // the state has not moved yet — the Hamiltonian changed under it, which is the quench
    assert!((holon_hadron_occ(1, site) - before).abs() <= 1e-12, "a grab alone must not move the state");
    let mut peak = before;
    for _ in 0..60 {
        assert_eq!(holon_hadron_step(1, 0.02), 0);
        peak = peak.max(holon_hadron_occ(1, site));
    }
    println!("grabbed site {site}: density {before:.4} -> peak {peak:.4} over 60 steps of dt=0.02");
    assert!(peak > before + 0.05, "the grab moved nothing: {before:.4} -> {peak:.4}");
    assert!((holon_hadron_norm(1) - 1.0).abs() <= 1e-10, "the grabbed evolution stayed unitary");
    // and letting go puts the ground state back
    assert_eq!(holon_hadron_release(1), 0);
    assert_eq!(holon_hadron_pin_at(1, site), 0.0);
    assert!((holon_hadron_occ(1, site) - before).abs() <= 1e-9, "release did not restore the ground state");
    assert_eq!(holon_hadron_time(1), 0.0, "release resets the clock");
}

/// The grab's refusals fire by name too.
#[test]
fn the_grab_refuses_by_name() {
    let _band = serial();
    assert_eq!(holon_hadron_solve(4, 4.0, 1), 0);
    assert_eq!(holon_hadron_grab(1, 99, -1.0), 5, "no such site");
    assert_eq!(holon_hadron_grab(9, 0, -1.0), 5, "no such sector");
    assert_eq!(holon_hadron_step(9, 0.1), 5, "no such sector");
    assert_eq!(holon_hadron_step(1, f64::NAN), 5, "a non-finite step is refused");
    assert!(holon_hadron_pin_at(1, 99).is_nan(), "off the chain reads NaN");
}

// ------------------------------------------------------------------ the flux string

use holon_chem::lanes::LaneSigma;
use holon_chem::qcd2::Qcd2;
use holon_chem::sigma_op::SigmaOp;

/// A lane operator as a dense matrix on the sector (row-major), by applying it to each basis
/// vector. For the 64- and 216-determinant sectors of the 4-site chain only.
fn dense(space: &holon_chem::lanes::LaneSpace, ham: &holon_chem::lanes::LaneHamiltonian<f64>) -> Vec<f64> {
    let d = space.n_det;
    let mut sig = LaneSigma::new(space, ham, 1);
    let mut m = vec![0.0; d * d];
    let (mut e, mut w) = (vec![0.0; d], vec![0.0; d]);
    for j in 0..d {
        e[j] = 1.0;
        sig.apply(&e, &mut w);
        e[j] = 0.0;
        for i in 0..d {
            m[i * d + j] = w[i];
        }
    }
    m
}

/// Eigenvalues (ascending) and eigenvectors (columns of the returned row-major matrix) of a
/// symmetric matrix, by cyclic Jacobi rotations.
fn eigen(mut a: Vec<f64>, d: usize) -> (Vec<f64>, Vec<f64>) {
    let mut v = vec![0.0; d * d];
    for i in 0..d {
        v[i * d + i] = 1.0;
    }
    for _ in 0..100 {
        let off: f64 = (0..d).flat_map(|i| (i + 1..d).map(move |j| (i, j))).map(|(i, j)| a[i * d + j] * a[i * d + j]).sum();
        if off < 1e-26 {
            break;
        }
        for p in 0..d {
            for q in p + 1..d {
                let apq = a[p * d + q];
                if apq.abs() < 1e-300 {
                    continue;
                }
                let theta = 0.5 * (a[q * d + q] - a[p * d + p]) / apq;
                let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
                let t = if theta == 0.0 { 1.0 } else { t };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                for k in 0..d {
                    let (akp, akq) = (a[k * d + p], a[k * d + q]);
                    a[k * d + p] = c * akp - s * akq;
                    a[k * d + q] = s * akp + c * akq;
                }
                for k in 0..d {
                    let (apk, aqk) = (a[p * d + k], a[q * d + k]);
                    a[p * d + k] = c * apk - s * aqk;
                    a[q * d + k] = s * apk + c * aqk;
                }
                for k in 0..d {
                    let (vkp, vkq) = (v[k * d + p], v[k * d + q]);
                    v[k * d + p] = c * vkp - s * vkq;
                    v[k * d + q] = s * vkp + c * vkq;
                }
            }
        }
    }
    let mut order: Vec<usize> = (0..d).collect();
    order.sort_by(|&i, &j| a[i * d + i].total_cmp(&a[j * d + j]));
    let vals = order.iter().map(|&i| a[i * d + i]).collect();
    let mut vecs = vec![0.0; d * d];
    for (new, &old) in order.iter().enumerate() {
        for k in 0..d {
            vecs[k * d + new] = v[k * d + old];
        }
    }
    (vals, vecs)
}

#[test]
fn the_links_energies_are_the_hamiltonians_gauge_part_exactly() {
    let _band = serial();
    let (n, x) = (4usize, 4.0f64);
    let q = Qcd2::new(n, x);
    // the operator: hopping plus the links is the Hamiltonian, entry for entry
    let (full, hop) = (q.lane_hamiltonian(), q.lane_hopping());
    let links: Vec<_> = (0..n - 1).map(|l| q.lane_link(l)).collect();
    for (i, v) in full.h.iter().enumerate() {
        for (j, a) in v.iter().enumerate() {
            assert!((a - hop.h[i][j] - links.iter().map(|l| l.h[i][j]).sum::<f64>()).abs() < 1e-13);
        }
    }
    for (i, v) in full.g.iter().enumerate() {
        for (j, a) in v.iter().enumerate() {
            assert!((a - hop.g[i][j] - links.iter().map(|l| l.g[i][j]).sum::<f64>()).abs() < 1e-13);
            assert_eq!(hop.g[i][j], 0.0);
        }
    }
    for b in 0..=1 {
        assert_eq!(holon_hadron_solve(n as u32, x, b), 0);
        // the lane form is the determinant route's operator (the electronic-structure integrals)
        assert!((holon_hadron_energy(b) - q.ground(b).e.v).abs() < 1e-8);
        assert_eq!(holon_hadron_links(b), 3);
        let l: Vec<f64> = (0..3).map(|k| holon_hadron_link_energy(b, k)).collect();
        let hop = holon_hadron_hop_energy(b);
        println!("N = 4, x = 4, B = {b}: links {l:?}, hopping {hop}, energy {}", holon_hadron_energy(b));
        assert!(l.iter().all(|v| *v >= 0.0), "a Casimir is not negative: {l:?}");
        assert!((l.iter().sum::<f64>() + hop - holon_hadron_energy(b)).abs() < 1e-9, "links {l:?} + hopping {hop} against {}", holon_hadron_energy(b));
        assert!(holon_hadron_link_energy(b, 3).is_nan(), "no fourth link");
        // the whole chain is a colour singlet, so the charge to a link's left is the conjugate
        // of the charge to its right and the two have one Casimir: the string is mirror-symmetric
        assert!((l[0] - l[2]).abs() < 1e-8, "{l:?}");
    }
    // after a grab the state moves and the same identity holds with the grab's own term
    holon_hadron_solve(n as u32, x, 1);
    let before: Vec<f64> = (0..3).map(|k| holon_hadron_link_energy(1, k)).collect();
    holon_hadron_grab(1, 1, -1.0);
    let mut moved = 0.0f64;
    for _ in 0..100 {
        holon_hadron_step(1, 0.03);
        let l: Vec<f64> = (0..3).map(|k| holon_hadron_link_energy(1, k)).collect();
        assert!(l.iter().all(|v| *v >= 0.0));
        moved = moved.max(l.iter().zip(&before).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max));
        let pin: f64 = (0..n as u32).map(|k| holon_hadron_pin_at(1, k) * holon_hadron_occ(1, k)).sum();
        assert!((l.iter().sum::<f64>() + holon_hadron_hop_energy(1) + pin - holon_hadron_live_energy(1)).abs() < 1e-8);
    }
    assert!(moved > 0.01, "the string moves under the grab: {moved}");
    holon_hadron_release(1);
}

#[test]
fn the_string_oscillates_at_the_pulled_chains_energy_gaps() {
    let _band = serial();
    // the ladder's scene: N = 4, x = 4, B = 1, site 1 pulled by a well of depth 1
    let (n, x, site, pull) = (4usize, 4.0f64, 1usize, -1.0f64);
    let q = Qcd2::new(n, x);
    let space = q.lane_space(1);
    let d = space.n_det;
    let mut pulled = q.lane_hamiltonian();
    for c in 0..3 {
        pulled.one_body(c, site, site, pull);
    }
    let (vals, vecs) = eigen(dense(&space, &pulled), d);
    // the state at the grab is the unpulled ground state; its weights on the pulled chain's states
    let ground = q.ground_lanes(1, 1).vector;
    let c: Vec<f64> = (0..d).map(|j| (0..d).map(|k| ground[k] * vecs[k * d + j]).sum()).collect();
    assert!((c.iter().map(|v| v * v).sum::<f64>() - 1.0).abs() < 1e-10);
    // each link's operator between those states
    let links: Vec<Vec<f64>> = (0..n - 1)
        .map(|l| {
            let m = dense(&space, &q.lane_link(l));
            let mut out = vec![0.0; d * d];
            for i in 0..d {
                for j in 0..d {
                    let mut s = 0.0;
                    for a in 0..d {
                        let mut t = 0.0;
                        for b2 in 0..d {
                            t += m[a * d + b2] * vecs[b2 * d + j];
                        }
                        s += vecs[a * d + i] * t;
                    }
                    out[i * d + j] = s;
                }
            }
            out
        })
        .collect();
    // L_n(t) = sum_ij c_i c_j <i|L_n|j> cos((E_i - E_j) t): the lines, strongest first
    let predict = |l: usize, t: f64| (0..d).flat_map(|i| (0..d).map(move |j| (i, j))).map(|(i, j)| c[i] * c[j] * links[l][i * d + j] * ((vals[i] - vals[j]) * t).cos()).sum::<f64>();
    let mut lines: Vec<(f64, f64, usize, usize)> = Vec::new();
    for i in 0..d {
        for j in i + 1..d {
            let amp = (0..n - 1).map(|l| (2.0 * c[i] * c[j] * links[l][i * d + j]).abs()).fold(0.0, f64::max);
            if amp > 1e-4 {
                lines.push((vals[j] - vals[i], amp, i, j));
            }
        }
    }
    lines.sort_by(|a, b| b.1.total_cmp(&a.1));
    println!("the pulled chain's lowest levels: {:?}", &vals[..6]);
    println!("weights of the starting state on them: {:?}", c[..6].iter().map(|v| v * v).collect::<Vec<_>>());
    for (gap, amp, i, j) in lines.iter().take(6) {
        println!("line: gap E{j} - E{i} = {gap:.6} (period {:.4} model time, {:.2} s at 0.6 a second), largest amplitude on a link {amp:.5}", std::f64::consts::TAU / gap, std::f64::consts::TAU / gap / 0.6);
    }
    assert!(!lines.is_empty() && lines[0].1 > 0.01);
    // and the band's own evolution follows that sum of lines
    assert_eq!(holon_hadron_solve(n as u32, x, 1), 0);
    holon_hadron_grab(1, site as u32, pull);
    let mut worst = 0.0f64;
    for step in 1..=200 {
        holon_hadron_step(1, 0.02);
        for l in 0..n - 1 {
            worst = worst.max((holon_hadron_link_energy(1, l as u32) - predict(l, 0.02 * step as f64)).abs());
        }
    }
    println!("the stepped string against its lines over 4 units of model time: largest difference {worst:.2e}");
    assert!(worst < 1e-7, "{worst}");
    holon_hadron_release(1);
}
