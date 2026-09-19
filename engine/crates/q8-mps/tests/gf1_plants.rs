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
//!
//! **Amendment 1's plants** (`conformance/crystal/GF1_AMENDMENT_1.md`, "Plants added"): P3′, P7,
//! P8, P9, one test each below, on the non-local minimiser `sre2_nonlocal_min`. Tolerances are
//! the amendment's (P3′: 1e-9; P8: 1e-6; P7: the parent's 1e-12). Two of the amendment's rows say
//! something a theorem contradicts, and the tests say so where they assert the right thing:
//! P3′'s "`M₂` itself is `0.415 N`" holds under random CLIFFORDS and not under random angles, and
//! §A2's "a GHZ state's `M₂^nl` is not zero" contradicts P7's own row. P8's two convergence
//! clauses fail at the amendment's numbers (the descent converges linearly and three sweeps is
//! not a criterion); it asserts them as written and is `#[ignore]`d with the measured values in
//! its doc. The correction is in the amendment's `Correction on building` section. Beside the
//! plants: the amendment's literal golden-section line search refereed against the exact one;
//! the parity theorem (`magic.rs`, finding (0)) measured on a Schwinger vacuum — the frame
//! removes nothing there, by symmetry; and `#[ignore]`d cost measurements and P8's twelve-sweep
//! convergence study.

use q8_mps::magic::{
    apply_real_clifford, apply_ry, ghz_mps, product_state_mps, random_mps, real_clifford_indices, sre2,
    sre2_brute, sre2_local_min, sre2_local_min_trace, sre2_nonlocal_min, sre2_nonlocal_min_golden,
    sre2_nonlocal_min_ordered, MagicError, RY_PERIOD,
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

// ------------------------------------------------------------------ Amendment 1's plants

/// `M₂` of `R_y(α)|H⟩` FROM THE DEFINITION: the Bloch vector turns to
/// `(sin(π/4 + 2α), 0, cos(π/4 + 2α))`, so `Σ_a ⟨σ_a⟩⁴ = 1 + sin⁴ + cos⁴` and
/// `M₂ = −log₂(Σ/2) = −log₂(1 − ¼ cos² 4α)`: `log₂(4/3)` at `α = 0`, zero at `α = π/8` (`|+⟩`).
fn rotated_h_type_magic_per_site(alpha: f64) -> f64 {
    let phi = PI / 4.0 + 2.0 * alpha;
    let (x, z) = (phi.sin(), phi.cos());
    -((1.0 + x.powi(4) + z.powi(4)) / 2.0).log2()
}

/// P3′ — amendment row: *"a product of H-type states rotated by random single-site unitaries:
/// `M₂^nl = 0` to `10⁻⁹`, where `M₂` itself is `0.415 N` — the new minimiser removes what the
/// Clifford one could not."*
///
/// Two draws at N = 8, because the row's two clauses hold under DIFFERENT draws. (a) P3's own
/// draw, a random real CLIFFORD per site: `M₂ = N log₂(4/3)` to 1e-9 (Clifford invariance; the
/// exact per-site value is `log₂(4/3) = 0.41503749927884…`, the row's `0.415`), the Clifford
/// minimiser reads the same to 1e-9 — it removes nothing — and `M₂^nl ≤ 1e-9`. (b) A different
/// random ANGLE `α_j ∈ [0, π)` per site, the continuous unitary the row names: `M₂` is
/// `Σ_j −log₂(1 − ¼ cos² 4α_j)` to 1e-9 by additivity, which is NOT `0.415 N` (this draw reads
/// more than `0.5` below it, asserted, so the row's second clause as written would fail), and
/// `M₂^nl ≤ 1e-9` still, because every real single-site state is `R_y(θ)|0⟩` for some `θ` —
/// `SO(2)` is transitive on the real unit circle — so the product is a local rotation of `|0⟩^N`.
#[test]
fn p3_prime_a_product_of_h_type_states_under_random_frames_minimises_to_zero() {
    let n = 8;
    let per_site = h_type_magic_per_site();
    println!("P3' exact M2 of |H> per site: log2(4/3) = {per_site:.17}");
    let base = h_type_product(n);

    // (a) random real Cliffords — P3's draw, the row's "0.415 N" clause.
    let (rot_c, picks) = rotate_by_random_real_cliffords(&base, 0x5eed_0303);
    assert!(picks.iter().any(|&k| k != 0), "the draw rotated at least one site: {picks:?}");
    let m2_c = sre2(&rot_c).unwrap();
    let (loc_c, _) = sre2_local_min(&rot_c, 3).unwrap();
    let nl_c = sre2_nonlocal_min(&rot_c, 3).unwrap();
    println!(
        "P3' (a) Cliffords {picks:?}: M2 {m2_c:.15}  N·log2(4/3) {:.15}  local-Clifford min {loc_c:.15}  M2_nl {:.3e}  angles {:?}  evaluations {}  per sweep {:?}",
        n as f64 * per_site,
        nl_c.m2,
        nl_c.angles.iter().map(|a| format!("{a:.6}")).collect::<Vec<_>>(),
        nl_c.evaluations,
        nl_c.per_sweep
    );
    assert!((m2_c - n as f64 * per_site).abs() <= 1e-9, "M2 under random Cliffords is N·log2(4/3): {m2_c}");
    assert!((loc_c - m2_c).abs() <= 1e-9, "the Clifford minimiser removes nothing: {loc_c} vs {m2_c}");
    assert!(nl_c.m2.abs() <= 1e-9, "M2_nl of a product state must be zero: {}", nl_c.m2);
    assert_eq!(nl_c.m2_identity, m2_c);

    // (b) random continuous angles — the row's "random single-site unitaries".
    let mut rnd = lcg(0x5eed_0313);
    let alphas: Vec<f64> = (0..n).map(|_| PI * (rnd() as f64) / ((1u64 << 53) as f64)).collect();
    let rot_a: Vec<TensorSite> = base.iter().zip(&alphas).map(|(t, &a)| apply_ry(t, a)).collect();
    let m2_a = sre2(&rot_a).unwrap();
    let target_a: f64 = alphas.iter().map(|&a| rotated_h_type_magic_per_site(a)).sum();
    let nl_a = sre2_nonlocal_min(&rot_a, 3).unwrap();
    println!(
        "P3' (b) angles {:?}: M2 {m2_a:.15}  Σ_j m(α_j) {target_a:.15}  diff {:.3e}  vs N·log2(4/3): {:+.6}  M2_nl {:.3e}  angles {:?}  evaluations {}  per sweep {:?}",
        alphas.iter().map(|a| format!("{a:.6}")).collect::<Vec<_>>(),
        m2_a - target_a,
        m2_a - n as f64 * per_site,
        nl_a.m2,
        nl_a.angles.iter().map(|a| format!("{a:.6}")).collect::<Vec<_>>(),
        nl_a.evaluations,
        nl_a.per_sweep
    );
    assert!((m2_a - target_a).abs() <= 1e-9, "additivity under random angles: {m2_a} vs {target_a}");
    assert!(n as f64 * per_site - m2_a > 0.5, "the row's '0.415 N' does not hold under random angles: {m2_a} vs {}", n as f64 * per_site);
    assert!(nl_a.m2.abs() <= 1e-9, "M2_nl of a product state must be zero: {}", nl_a.m2);
    // The frame found is the one the theorem names: R_y(θ_j) R_y(α_j) |H⟩ is a stabilizer state
    // iff π/8 + α_j + θ_j ≡ 0 (mod π/4).
    for (j, (&a, &th)) in alphas.iter().zip(&nl_a.angles).enumerate() {
        let off = (PI / 8.0 + a + th).rem_euclid(RY_PERIOD);
        let off = off.min(RY_PERIOD - off);
        assert!(off < 1e-7, "site {j}: the found angle is not a stabilizer frame, off by {off:.3e}");
    }
}

/// P7 — amendment row: *"a GHZ state: `M₂^nl = M₂ = 0` (stabilizer) — no false magic manufactured
/// by the search."* At N = 8: `|M₂| ≤ 1e-12` and `|M₂^nl| ≤ 1e-12` after three sweeps.
///
/// The row contradicts §A2's own sentence, *"a GHZ state's is not zero: its magic is in the
/// entanglement"*: the row is right, GHZ is a stabilizer state and `M₂^nl ≤ M₂ = 0`. The state
/// that sentence reaches for is this plant's CONTROL: `cos(π/8)|0…0⟩ + sin(π/8)|1…1⟩`, whose
/// Schmidt coefficients across every cut are `(cos π/8, sin π/8)`, which no local frame changes
/// and no stabilizer state has (theirs are `(1, 0)` or `(1, 1)/√2`). Its `M₂^nl` stays clearly
/// above zero: the plant is not vacuous, and the search does not destroy what the entanglement
/// carries either.
#[test]
fn p7_ghz_reads_zero_and_the_search_manufactures_no_magic() {
    let n = 8;
    let ghz = ghz_mps(n);
    let m2 = sre2(&ghz).unwrap();
    let nl = sre2_nonlocal_min(&ghz, 3).unwrap();
    println!(
        "P7 GHZ_8: M2 {m2:.3e}  M2_nl {:.3e}  per sweep {:?}  angles {:?}  evaluations {}",
        nl.m2,
        nl.per_sweep.iter().map(|v| format!("{v:.3e}")).collect::<Vec<_>>(),
        nl.angles.iter().map(|a| format!("{a:.3e}")).collect::<Vec<_>>(),
        nl.evaluations
    );
    assert!(m2.abs() <= 1e-12, "GHZ is a stabilizer state: M2 = {m2}");
    assert!(nl.m2.abs() <= 1e-12, "the search must manufacture no magic on GHZ: {}", nl.m2);
    assert!(nl.m2 <= m2, "M2_nl ≤ M2: {} vs {m2}", nl.m2);

    // The control: entanglement that is not a stabilizer state's.
    let (c, s) = ((PI / 8.0).cos(), (PI / 8.0).sin());
    let mut tilted = ghz_mps(n);
    tilted[0].set(0, 0, 0, c);
    tilted[0].set(1, 0, 1, s);
    let m2_t = sre2(&tilted).unwrap();
    let nl_t = sre2_nonlocal_min(&tilted, 3).unwrap();
    println!("P7 control cos(π/8)|0..0> + sin(π/8)|1..1>: M2 {m2_t:.12}  M2_nl {:.12}  per sweep {:?}", nl_t.m2, nl_t.per_sweep);
    assert!(nl_t.m2 <= m2_t + 1e-12);
    assert!(nl_t.m2 > 0.1, "magic the entanglement carries survives every local frame: {}", nl_t.m2);
}

/// P8 — amendment row: *"a random MPS at `N = 12, χ = 6`: `M₂^nl ≤ M₂`, the fourth sweep improves
/// the third by under `10⁻⁶`, and two different site orders agree to `10⁻⁶` (the descent is not
/// order-trapped at this size)."* Forward order for four sweeps (the three-sweep reading is its
/// prefix, by determinism — asserted in the module's own tests); the reversed order for three.
///
/// **`#[ignore]`d: the amendment's two convergence numbers are not met at this size, and the
/// tolerances are not loosened.** Measured (seed `0x0008_2424`, `M₂ = 6.350414296403`): the
/// forward sweeps read `5.072057011403, 5.058341329348, 5.057632708377, 5.057402824264` — the
/// fourth buys `2.30 × 10⁻⁴`, not under `10⁻⁶`; the reversed order reads `5.068521596680` after
/// three sweeps, `1.09 × 10⁻²` from the forward, not `10⁻⁶`. `M₂^nl ≤ M₂` holds, and is asserted
/// wherever the minimiser runs. The twelve-sweep study below says why: the coordinate descent
/// converges linearly at a ratio of `≈ 0.65` per sweep in BOTH orders (forward
/// `5.057087819338` after twelve, still buying `5 × 10⁻⁶`; reversed `5.057338453899`, buying
/// `1.4 × 10⁻⁴`), and the two geometric tails extrapolate to the same limit within `10⁻⁵` —
/// the descent is not order-trapped, it is slow, and "three sweeps" is not a convergence
/// criterion. The correction is `GF1_AMENDMENT_1.md`, `Correction on building`, C3.
#[test]
#[ignore = "P8's convergence clauses fail at the amendment's numbers: the fourth sweep buys 2.3e-4 and the reversed order differs by 1.1e-2 after three sweeps; see the doc comment and the amendment's correction C3"]
fn p8_a_random_mps_at_n12_chi6_descends_monotonically_converges_by_three_sweeps_and_is_not_order_trapped() {
    let t = random_mps(12, 6, 0x0008_2424);
    let t0 = Instant::now();
    let m2 = sre2(&t).unwrap();
    let t_m2 = t0.elapsed().as_secs_f64();
    let t0 = Instant::now();
    let fwd = sre2_nonlocal_min(&t, 4).unwrap();
    let t_fwd = t0.elapsed().as_secs_f64();
    let rev_order: Vec<usize> = (0..12).rev().collect();
    let t0 = Instant::now();
    let rev = sre2_nonlocal_min_ordered(&t, 3, &rev_order).unwrap();
    let t_rev = t0.elapsed().as_secs_f64();
    let (three, four) = (fwd.per_sweep[3], fwd.per_sweep[4]);
    println!(
        "P8 N=12 chi=6: M2 {m2:.12} ({t_m2:.2}s)  forward per sweep {:?} ({} evaluations, {t_fwd:.1}s)  reversed per sweep {:?} ({} evaluations, {t_rev:.1}s)",
        fwd.per_sweep, fwd.evaluations, rev.per_sweep, rev.evaluations
    );
    println!(
        "P8 forward angles {:?}\nP8 reversed angles {:?}",
        fwd.angles.iter().map(|a| format!("{a:.6}")).collect::<Vec<_>>(),
        rev.angles.iter().map(|a| format!("{a:.6}")).collect::<Vec<_>>()
    );
    println!("P8 three sweeps {three:.12}  fourth bought {:.3e}  |forward − reversed| after three {:.3e}", three - four, (three - rev.m2).abs());
    assert_eq!(fwd.m2_identity, m2);
    assert!(three <= m2, "M2_nl {three} must not exceed M2 {m2}");
    assert!(four <= three, "the running minimum cannot rise");
    assert!(three - four < 1e-6, "a fourth sweep bought {:.3e}", three - four);
    assert!((three - rev.m2).abs() <= 1e-6, "forward {three} vs reversed {} after three sweeps", rev.m2);
    assert_eq!(fwd.evaluations, 1 + 3 * 12 * 4);
    assert_eq!(rev.evaluations, 1 + 3 * 12 * 3);
    assert_eq!(rev.order, rev_order);
}

/// P9 — amendment row: *"`χ = 12` requested: REFUSED by name with the lease it would need."*
/// The non-local minimiser refuses exactly as the exact reader does — the same `Price` error
/// with the same bytes (`4 · 12⁸ · 8 = 13.76 GB`) against the same lease (8 GiB by default),
/// before any evaluation — and the message names both.
#[test]
fn p9_chi_12_is_refused_by_the_nonlocal_minimiser_exactly_as_by_the_exact_reader() {
    let t = random_mps(12, 12, 0x0009_2424);
    assert_eq!(t.iter().map(|s| s.chi_l.max(s.chi_r)).max(), Some(12));
    let exact = sre2(&t).expect_err("chi = 12 is above the lease");
    let nl = sre2_nonlocal_min(&t, 3).expect_err("chi = 12 is above the lease");
    println!("P9 exact reader: {exact}\nP9 non-local minimiser: {nl}");
    assert_eq!(exact, nl, "the same refusal");
    match nl {
        MagicError::Price { bytes, lease_bytes } => {
            assert_eq!(bytes, 4 * 12u64.pow(8) * 8);
            assert!(bytes > lease_bytes, "{bytes} against {lease_bytes}");
            let msg = nl.to_string();
            assert!(msg.contains("13.76 GB") && msg.contains("lease") && msg.contains("refused"), "{msg}");
        }
        other => panic!("expected the price refusal, got {other:?}"),
    }
    // Under the lease at chi = 11, the same chain is admitted (priced only, not run: 6.9 GB).
    let eleven = random_mps(12, 11, 0x0009_2424);
    assert!(q8_mps::magic::price_bytes(&eleven) <= q8_mps::magic::lease_bytes());
}

/// The amendment's line search AS WRITTEN — golden section on `θ_j ∈ [0, π)` — refereed against
/// the exact three-reading search on a random MPS at N = 8, χ = 4, three sweeps, at two bracket
/// tolerances. What is asserted is only what must hold (each descends from `M₂`, and the counts
/// are what the methods cost); what it READS beside the exact minimum, and how many `M₂`
/// evaluations it spends, are printed for the record. The exact search's count is
/// `1 + 3·N·sweeps = 73`; golden section to a bracket of `tol` costs `2 + ⌈log(tol/π)/log 0.618⌉`
/// per site per sweep.
#[test]
fn the_amendments_golden_section_on_0_pi_refereed_against_the_exact_line_search() {
    let t = random_mps(8, 4, 0x0a0a_2424);
    let m2 = sre2(&t).unwrap();
    let t0 = Instant::now();
    let exact = sre2_nonlocal_min(&t, 3).unwrap();
    let t_exact = t0.elapsed().as_secs_f64();
    println!("referee N=8 chi=4: M2 {m2:.12}  exact M2_nl {:.12}  per sweep {:?}  {} evaluations  {t_exact:.1}s", exact.m2, exact.per_sweep, exact.evaluations);
    assert_eq!(exact.evaluations, 1 + 3 * 8 * 3);
    for tol in [1e-3f64, 1e-6] {
        let t0 = Instant::now();
        let g = sre2_nonlocal_min_golden(&t, 3, tol).unwrap();
        let t_g = t0.elapsed().as_secs_f64();
        let per_search = (g.evaluations - 1) as f64 / (8 * 3) as f64;
        println!(
            "referee golden tol {tol:.0e}: M2_nl {:.12}  per sweep {:?}  {} evaluations ({per_search:.1} per line search)  {t_g:.1}s  vs exact {:+.3e}",
            g.m2, g.per_sweep, g.evaluations, g.m2 - exact.m2
        );
        assert_eq!(g.m2_identity, m2);
        assert!(g.m2 <= m2);
        let expected = 2 + ((tol / PI).ln() / (0.5 * (5.0f64.sqrt() - 1.0)).ln()).ceil() as usize;
        assert_eq!((g.evaluations - 1) / (8 * 3), expected, "golden section's count per line search");
    }
}

/// What the ladder will meet — the parity theorem on a Schwinger vacuum (`magic.rs`, module
/// doc, finding (0)): the Jordan–Wigner vacuum has definite charge, hence definite parity
/// `Π_j Z_j`, and on such a real state the identity frame is every site's own minimum, so the
/// amendment's descent from the identity never moves and `M₂^nl = M₂`. At `x = 4, N = 12, χ = 6`
/// (variance `8.3e-4` per site, under A3's gate): the parity is MEASURED at `±1` to 1e-8 first,
/// then `|M₂^nl − M₂| ≤ 1e-12` after one sweep and every angle within 1e-8 of the identity. The
/// re-staked S3 reads the same as S1 at every point of the ladder, by symmetry.
#[test]
fn a_schwinger_vacuum_has_definite_parity_and_the_real_local_frame_removes_none_of_its_magic() {
    use q8_mps::observables::{expectation, norm_squared};
    use q8_mps::schwinger::Schwinger;
    use q8_mps::variance::energy_variance;
    let (n, x, chi) = (12usize, 4.0, 6usize);
    let s = Schwinger::new(n, x, vec![]);
    let (e, res) = s.ground_energy(chi, 60, 1e-11).expect("the sweep runs");
    assert!(res.converged);
    let (_, _, var) = energy_variance(&res.tensors, &s.mpo()).expect("within the variance lease");
    let z_all: Vec<(usize, q8_mps::ops::Op2)> = (0..n).map(|j| (j, q8_mps::magic::PAULI_REAL[q8_mps::magic::PAULI_Z])).collect();
    let parity = expectation(&res.tensors, &z_all) / norm_squared(&res.tensors);
    let m2 = sre2(&res.tensors).unwrap();
    let t0 = Instant::now();
    let nl = sre2_nonlocal_min(&res.tensors, 1).unwrap();
    println!(
        "vacuum x={x} N={n} chi={chi}: E {e:.10}  variance/site {:.3e}  <ΠZ> {parity:+.12}  M2 {m2:.12}  M2_nl {:.12}  per sweep {:?}  angles {:?}  ({} evaluations, {:.1}s)",
        var / n as f64,
        nl.m2,
        nl.per_sweep,
        nl.angles.iter().map(|a| format!("{a:.2e}")).collect::<Vec<_>>(),
        nl.evaluations,
        t0.elapsed().as_secs_f64()
    );
    assert!(var / n as f64 <= 1e-3, "A3's gate");
    assert!((parity.abs() - 1.0).abs() < 1e-8, "the vacuum has definite parity: {parity}");
    assert!((nl.m2 - m2).abs() <= 1e-12, "the frame removes nothing on a parity-symmetric state: {} vs {m2}", nl.m2);
    assert!(nl.angles.iter().all(|&a| a.min(RY_PERIOD - a) < 1e-8), "{:?}", nl.angles);
}

/// P8's convergence study: the same state, twelve sweeps in each order, every sweep's minimum
/// printed — whether the descent is slow or order-trapped is read from where the two orders end.
#[test]
#[ignore = "P8's convergence study: twelve sweeps in each order, ~10 minutes each"]
fn p8_convergence_study_forward_twelve_sweeps() {
    let t = random_mps(12, 6, 0x0008_2424);
    let r = sre2_nonlocal_min(&t, 12).unwrap();
    println!("P8 STUDY forward: per sweep {:?}\nP8 STUDY forward angles {:?}  evaluations {}", r.per_sweep, r.angles, r.evaluations);
}

#[test]
#[ignore = "P8's convergence study: twelve sweeps in each order, ~10 minutes each"]
fn p8_convergence_study_reversed_twelve_sweeps() {
    let t = random_mps(12, 6, 0x0008_2424);
    let order: Vec<usize> = (0..12).rev().collect();
    let r = sre2_nonlocal_min_ordered(&t, 12, &order).unwrap();
    println!("P8 STUDY reversed: per sweep {:?}\nP8 STUDY reversed angles {:?}  evaluations {}", r.per_sweep, r.angles, r.evaluations);
}

/// The cost, measured: `sre2_nonlocal_min` with three sweeps on a random MPS at N = 12, χ = 6,
/// single thread, release. Printed: `sre2`'s own wall time, the minimiser's, and the count of
/// evaluations (`1 + 3·N·sweeps = 109`). `#[ignore]`d because a timing is not a correctness gate.
#[test]
#[ignore = "timing, not correctness: run deliberately to re-bank the M2_nl price at N=12, chi=6"]
fn cost_of_the_nonlocal_minimiser_at_n12_chi6() {
    report_cost(12, 6, 0x0008_2424);
}

/// The cost, measured, at N = 16, χ = 8 (`1 + 3·16·3 = 145` evaluations).
#[test]
#[ignore = "timing, not correctness: run deliberately to re-bank the M2_nl price at N=16, chi=8"]
fn cost_of_the_nonlocal_minimiser_at_n16_chi8() {
    report_cost(16, 8, 0x0010_2424);
}

fn report_cost(n: usize, chi: usize, seed: u64) {
    let t = random_mps(n, chi, seed);
    let t0 = Instant::now();
    let m2 = sre2(&t).unwrap();
    let t_m2 = t0.elapsed().as_secs_f64();
    let t0 = Instant::now();
    let nl = sre2_nonlocal_min(&t, 3).unwrap();
    let t_nl = t0.elapsed().as_secs_f64();
    println!(
        "COST N={n} chi={chi} threads={}: sre2 {t_m2:.2}s (M2 {m2:.12}); sre2_nonlocal_min 3 sweeps {t_nl:.1}s, {} evaluations ({:.2}s each, {:.1} per line search), M2_nl {:.12}, per sweep {:?}",
        q8_mps::mps::threads(),
        nl.evaluations,
        t_nl / nl.evaluations as f64,
        (nl.evaluations - 1) as f64 / (n * 3) as f64,
        nl.m2,
        nl.per_sweep
    );
}
