//! GF1's plants — `conformance/crystal/GF1_PREREG.md` §4, P1–P6, one test each. Every plant
//! must fire before a vacuum is read (§5 (e)): a failing plant here means the instrument is not
//! admitted. Tolerances are the prereg's where it names one (P4: 1e-9; P6: 0.05) and 1e-9 for
//! exact identities where it says "to the floor".
//!
//! THE CARRIER FOR P1, STATED. `TensorSite` is REAL (`Vec<f64>`), so the prereg's `|T⟩ =
//! (|0⟩ + e^{iπ/4}|1⟩)/√2` cannot be carried. The state used is the real H-type magic state
//! `|ψ⟩ = cos(π/8)|0⟩ + sin(π/8)|1⟩`, Clifford-equivalent to `|T⟩` (a `π/2` rotation about `X`,
//! a Clifford, takes its Bloch vector `(1, 0, 1)/√2` to `|T⟩`'s `(1, 1, 0)/√2`, and `M₂` is
//! Clifford-invariant): `⟨X⟩ = sin(π/4)`, `⟨Y⟩ = 0`, `⟨Z⟩ = cos(π/4)`,
//! so `Σ_a ⟨σ_a⟩⁴ = 1 + ¼ + 0 + ¼ = 3/2` and `M₂ = −log₂(3/4) = log₂(4/3)` per site — the same
//! constant `|T⟩` gives (`⟨X⟩ = ⟨Y⟩ = 1/√2`, `⟨Z⟩ = 0`). Each test computes the target from
//! the definition rather than pasting the number.
//!
//! P3's Cliffords are drawn from the EIGHT single-qubit Cliffords whose matrix is real up to
//! phase (`I`, `X`, `Y`, `Z`, `H`, and the `±π/2` rotations about `Y` with their reflection — the
//! dihedral stabiliser of the `Y` axis): a real tensor cannot carry `S`. The minimiser sees all 24 through
//! their Pauli action; what P3 checks of it is that it returns the unrotated state's own
//! minimum, and zero on a rotated stabilizer state.

use q8_mps::magic::{
    apply_real_clifford, ghz_mps, product_state_mps, random_mps, real_clifford_indices, sre2, sre2_brute,
    sre2_local_min, sre2_local_min_trace,
};
use q8_mps::mps::TensorSite;
use std::f64::consts::PI;
use std::time::Instant;

/// `M₂` per site of the real H-type state, FROM THE DEFINITION: `−log₂( Σ_a ⟨σ_a⟩⁴ / 2 )`.
fn h_type_magic_per_site() -> f64 {
    let (c, s) = ((PI / 8.0).cos(), (PI / 8.0).sin());
    let (x, y, z) = (2.0 * c * s, 0.0f64, c * c - s * s);
    let sum4 = 1.0 + x.powi(4) + y.powi(4) + z.powi(4);
    -(sum4 / 2.0).log2()
}

fn h_type_product(n: usize) -> Vec<TensorSite> {
    let (c, s) = ((PI / 8.0).cos(), (PI / 8.0).sin());
    product_state_mps(&vec![[c, s]; n])
}

/// The crate's LCG, for the plants' random draws.
fn lcg(seed: u64) -> impl FnMut() -> u64 {
    let mut st = seed;
    move || {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        st >> 11
    }
}

fn rotate_by_random_real_cliffords(tensors: &[TensorSite], seed: u64) -> (Vec<TensorSite>, Vec<usize>) {
    let real = real_clifford_indices();
    let mut rnd = lcg(seed);
    let picks: Vec<usize> = (0..tensors.len()).map(|_| real[(rnd() % real.len() as u64) as usize]).collect();
    let rotated = tensors.iter().zip(&picks).map(|(t, &k)| apply_real_clifford(t, k).unwrap()).collect();
    (rotated, picks)
}

/// P1 — additivity and the constant: `M₂(|ψ⟩^{⊗N}) = N · log₂(4/3)` at N ∈ {1, 4, 10}.
#[test]
fn p1_a_product_of_h_type_magic_states_reads_n_log2_four_thirds() {
    let per_site = h_type_magic_per_site();
    assert!((per_site - (4.0f64 / 3.0).log2()).abs() < 1e-15, "the definition gives log2(4/3): {per_site}");
    for n in [1usize, 4, 10] {
        let m2 = sre2(&h_type_product(n)).expect("a chi=1 product state is admitted");
        let target = n as f64 * per_site;
        println!("P1 N={n}: M2 = {m2:.15}  target {target:.15}  diff {:.3e}", m2 - target);
        assert!((m2 - target).abs() <= 1e-9, "N={n}: M2 {m2} vs N·log2(4/3) = {target}");
    }
}

/// P2 — a stabilizer state reads zero: `|0⟩^N`, `|+⟩^N` (vacuumConfig-like product states) and
/// the GHZ state at N = 6.
#[test]
fn p2_stabilizer_product_states_and_ghz_read_zero() {
    let h = std::f64::consts::FRAC_1_SQRT_2;
    let carriers: Vec<(&str, Vec<TensorSite>)> = vec![
        ("|0>^6", product_state_mps(&[[1.0, 0.0]; 6])),
        ("|+>^6", product_state_mps(&[[h, h]; 6])),
        ("|0>^12", product_state_mps(&[[1.0, 0.0]; 12])),
        ("GHZ_6", ghz_mps(6)),
    ];
    for (name, t) in carriers {
        let m2 = sre2(&t).expect("admitted");
        println!("P2 {name}: M2 = {m2:.3e}");
        assert!(m2.abs() <= 1e-9, "{name}: a stabilizer state must read zero, got {m2}");
    }
    // The plant can fail: a non-stabilizer product state at the same size does not read zero.
    let m2 = sre2(&h_type_product(6)).unwrap();
    assert!(m2 > 1.0, "the control is not vacuous: {m2}");
}

/// P3 — Clifford invariance and the local minimiser. P1's state under a random real Clifford
/// per site: `M₂` unchanged to 1e-9, and `sre2_local_min(…, 3)` equals the UNROTATED state's
/// own local minimum to 1e-9 (which, by invariance, is `N log₂(4/3)` — no local Clifford frame
/// removes the magic of a magic-state product, and the minimiser must not pretend one does). A
/// stabilizer product state under random Cliffords: the minimiser returns zero.
#[test]
fn p3_random_local_cliffords_leave_m2_fixed_and_the_minimiser_finds_the_frame() {
    let n = 10;
    let base = h_type_product(n);
    let (rotated, picks) = rotate_by_random_real_cliffords(&base, 0x5eed_0003);
    assert!(picks.iter().any(|&k| k != 0), "the draw rotated at least one site: {picks:?}");
    let m2_base = sre2(&base).unwrap();
    let m2_rot = sre2(&rotated).unwrap();
    println!("P3 magic product: M2 unrotated {m2_base:.15}  rotated {m2_rot:.15}  picks {picks:?}");
    assert!((m2_base - m2_rot).abs() <= 1e-9, "Clifford invariance: {m2_base} vs {m2_rot}");

    let (loc_base, _) = sre2_local_min(&base, 3).unwrap();
    let (loc_rot, frame) = sre2_local_min(&rotated, 3).unwrap();
    println!("P3 local min: unrotated {loc_base:.15}  rotated {loc_rot:.15}  frame {frame:?}");
    assert!((loc_base - loc_rot).abs() <= 1e-9, "the minimiser must reach the unrotated minimum: {loc_base} vs {loc_rot}");
    assert!((loc_rot - n as f64 * h_type_magic_per_site()).abs() <= 1e-9, "no local Clifford frame absorbs magic-state magic: {loc_rot}");

    // The rotated stabilizer product: |0>, |+> alternating, then random real Cliffords.
    let h = std::f64::consts::FRAC_1_SQRT_2;
    let stab: Vec<[f64; 2]> = (0..n).map(|j| if j % 2 == 0 { [1.0, 0.0] } else { [h, h] }).collect();
    let (rot_stab, picks) = rotate_by_random_real_cliffords(&product_state_mps(&stab), 0x5eed_0033);
    let (loc_stab, frame) = sre2_local_min(&rot_stab, 3).unwrap();
    println!("P3 stabilizer product: local min {loc_stab:.3e}  picks {picks:?}  frame {frame:?}");
    assert!(loc_stab.abs() <= 1e-9, "a rotated stabilizer product must minimise to zero: {loc_stab}");
}

/// P4 — the Pauli-replica `M₂` equals brute enumeration over `4^10` strings to 1e-9, on a
/// random real MPS at χ = 8, N = 10 (seed fixed).
#[test]
fn p4_the_replica_contraction_equals_brute_enumeration_at_n10_chi8() {
    let t = random_mps(10, 8, 0x0004_2424);
    let t0 = Instant::now();
    let replica = sre2(&t).expect("chi=8 is under the lease");
    let t_replica = t0.elapsed().as_secs_f64();
    let t0 = Instant::now();
    let brute = sre2_brute(&t).expect("N=10 is admitted");
    let t_brute = t0.elapsed().as_secs_f64();
    println!("P4 N=10 chi=8: replica {replica:.15} ({t_replica:.1}s)  brute {brute:.15} ({t_brute:.1}s)  diff {:.3e}", replica - brute);
    assert!((replica - brute).abs() <= 1e-9, "replica {replica} vs brute {brute}");
    // It can fail: a different state reads differently.
    let other = sre2(&random_mps(10, 8, 0x0004_2425)).unwrap();
    assert!((other - replica).abs() > 1e-6, "the referee is not vacuous: {other} vs {replica}");
}

/// P5 — on a random MPS at N = 24, χ = 8: `M₂^loc ≤ M₂ + 1e-12` after three sweeps, and a
/// fourth sweep improves the three-sweep minimum by less than 1e-6.
#[test]
fn p5_the_three_sweep_local_minimum_is_under_m2_and_a_fourth_sweep_adds_nothing() {
    let t = random_mps(24, 8, 0x0005_2424);
    let t0 = Instant::now();
    let m2 = sre2(&t).expect("under the lease");
    let t_m2 = t0.elapsed().as_secs_f64();
    let t0 = Instant::now();
    let (trace, frame) = sre2_local_min_trace(&t, 4).expect("under the lease");
    let t_loc = t0.elapsed().as_secs_f64();
    let (three, four) = (trace[3], trace[4]);
    println!("P5 N=24 chi=8: M2 {m2:.15} ({t_m2:.1}s)  trace {trace:?} ({t_loc:.1}s for 4 sweeps)  frame {frame:?}");
    assert!(three <= m2 + 1e-12, "M2_loc {three} must not exceed M2 {m2}");
    assert!(three - four < 1e-6, "a fourth sweep bought {:.3e}", three - four);
    assert!(four <= three, "the running minimum cannot rise: {four} after {three}");
    assert_eq!(frame.len(), 24);
    let (three_alone, _) = sre2_local_min(&random_mps(6, 4, 1), 3).unwrap();
    assert!(three_alone.is_finite());
}

/// P6 — a Schwinger vacuum at x = 4, N = 16, at two χ that both pass the variance gate:
/// `|ΔM₂| ≤ 0.05`.
///
/// THE GATE, STATED: the prereg names "the energy variance gate of `variance.rs`", and that
/// module computes the variance without a threshold — no threshold exists anywhere in the
/// repository. The gate applied here is the variance DENSITY `(⟨H²⟩ − ⟨H⟩²)/N ≤ 1e-3` in the
/// Hamiltonian's own units, SCHWINGER-3's 1e-3 χ-premise band read per site (the variance is
/// extensive). Measured at N = 16, x = 4: χ = 6 reads 1.07e-3 (refused, by a hair), χ = 8 reads
/// 1.9e-4, χ = 10 reads 4.3e-6 — so the two χ are 8 and 10. Under a stricter TOTAL-variance
/// gate at 1e-3 χ = 8 would also be refused and the plant would need χ ≥ 12, which the exact
/// instrument prices at 13.8 GB and refuses; that is reported, not hidden.
#[test]
fn p6_a_schwinger_vacuum_at_two_admitted_chi_reads_the_same_m2_to_0_05() {
    use q8_mps::schwinger::Schwinger;
    use q8_mps::variance::energy_variance;
    const VARIANCE_DENSITY_GATE: f64 = 1e-3;
    let (n, x) = (16usize, 4.0);
    let s = Schwinger::new(n, x, vec![]);
    let mpo = s.mpo();
    let mut readings = Vec::new();
    for chi in [6usize, 8, 10] {
        let t0 = Instant::now();
        let (e, res) = s.ground_energy(chi, 60, 1e-11).expect("the sweep runs");
        let t_dmrg = t0.elapsed().as_secs_f64();
        assert!(res.converged, "chi={chi}: the sweep converged");
        let (_, _, var) = energy_variance(&res.tensors, &mpo).expect("within the variance lease at N=16");
        let density = var / n as f64;
        let admitted = density <= VARIANCE_DENSITY_GATE;
        println!("P6 chi={chi}: E {e:.10}  variance {var:.3e}  per site {density:.3e}  {}  (dmrg {t_dmrg:.1}s)", if admitted { "ADMITTED" } else { "REFUSED" });
        if chi == 6 {
            assert!(!admitted, "chi=6 is the control that the gate refuses: {density:.3e}");
            continue;
        }
        assert!(admitted, "chi={chi} must pass the variance gate: {density:.3e} > {VARIANCE_DENSITY_GATE:.0e}");
        let t0 = Instant::now();
        let m2 = sre2(&res.tensors).expect("chi<=10 is under the lease");
        println!("P6 chi={chi}: M2 = {m2:.12}  M2/N = {:.6}  ({:.1}s)", m2 / n as f64, t0.elapsed().as_secs_f64());
        readings.push((chi, m2));
    }
    assert_eq!(readings.len(), 2, "two admitted chi");
    let delta = (readings[0].1 - readings[1].1).abs();
    println!("P6 |M2(chi=8) - M2(chi=10)| = {delta:.3e}");
    assert!(delta <= 0.05, "the reading is the truncation's, not the state's: {delta}");
}
