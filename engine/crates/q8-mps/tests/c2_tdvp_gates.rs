//! G-C2 — the staked correctness battery for `q8_mps::tdvp`, the carrier tower's C2 node
//! (`conformance/water_observatory/C2_TDVP_RESULTS.md`, WB-8.3).
//!
//! The claim under test is narrow and checkable: **single-site TDVP on this crate's MPO
//! machinery integrates the Schrodinger equation — exactly when the bond cap makes the MPS
//! manifold the whole Hilbert space, and to second order in the step when it does not.**
//! Everything else in this file exists to make that claim falsifiable.
//!
//! THE REFEREE IS A DIFFERENT CODE PATH ALL THE WAY DOWN. `Mpo::dense` builds `H` by
//! Kronecker products, `jacobi_eigen` diagonalises it, and `exact_propagate` applies one
//! phase per eigenvalue. Not one line of that is shared with `Tdvp::step`, which contracts
//! environments and Krylov-exponentiates effective Hamiltonians. Two implementations
//! agreeing is evidence; one implementation agreeing with itself is not.
//!
//! THE STAKE is the `const` block below. It was frozen before the first run and the first
//! run FALSIFIED PART OF IT, which is recorded here rather than quietly edited: the
//! original G-C2-2 staked a second-order convergence band at the full bond cap, and the
//! measured error was flat at `~1e-13` across a factor of eight in step size. The
//! integrator is not second order there, it is EXACT there — the projector-splitting
//! exactness property (Lubich–Oseledets 2014; Lubich–Vandereycken–Walach and Haegeman et
//! al. 2016): when the manifold contains the exact trajectory, the splitting reproduces
//! it with no step-size error. The gate was rewritten to test the true statement, which
//! is strictly stronger, and the order band moved to `G-C2-2b` where the manifold is a
//! genuine submanifold and an order is a meaningful thing to read. Bands are two-sided
//! where the quantity can err in both directions: an order of 2.6 is as much a failure as
//! an order of 1.2, because it means the error is not the error we think it is.
//!
//! EVERY GATE IS DEMONSTRATED FIRING (`g_c2_3_planted_mutations_fire`). Four planted
//! defects, each one a bug a person actually writes when implementing this integrator, and
//! each is required to break a NAMED prong — an OR-gate that only ever fires on one
//! disjunct is a one-prong gate wearing a disguise, so the firing prong is asserted and
//! printed per mutation.

use q8_mps::mpo::{dense_from_mpo, Mpo};
use q8_mps::tdvp::{
    exact_propagate, relative_distance, to_dense, CTensorSite, Cx, Mutation, Tdvp,
};

// ----------------------------------------------------------------- THE STAKE

/// Chain sites; `2 * SITES` Jordan-Wigner spin-orbitals, Hilbert dimension `2^(2*SITES)`.
const SITES: usize = 3;
const T_HOP: f64 = 1.0;
const U_INT: f64 = 4.0;
const MU: f64 = 0.0;
/// The NATURAL cap for a 6-site chain: `min(2^k, 2^(6-k))` peaks at 8. At this cap the MPS
/// manifold is the whole Hilbert space and the integrator is EXACT (`G-C2-2a`).
const CHI_FULL: usize = 8;
/// A genuine SUBmanifold. Here the flow is the TDVP flow rather than the Schrodinger flow,
/// so the integrator's own convergence order is a meaningful quantity (`G-C2-2b`) — and it
/// is the only regime in which the symmetric sweep is distinguishable from the
/// one-directional one, because at the cap both are exact.
const CHI_SUB: usize = 4;
/// Pinned. `tdvp::deterministic_state` is an LCG, not an RNG: no gate in this crate is
/// allowed to depend on a seed the runner chooses.
const SEED: u64 = 20260901;

const T_FINAL: f64 = 0.25;
const STEP_COUNTS: [usize; 4] = [4, 8, 16, 32];
/// Step counts for the order fit below the cap, and the fine-step reference the fit is
/// measured against. `N_REF` is 8x the finest, so the reference's own error contributes
/// `(64/512)^2 = 1.6%` of it and biases the measured order by `< 0.02` — small against the
/// band, and stated rather than hoped.
const STEP_COUNTS_SUB: [usize; 4] = [8, 16, 32, 64];
const N_REF_SUB: usize = 512;

/// G-C2-0: the MPS energy instrument and the dense one must agree. Pure instrument
/// agreement, no dynamics; if this fails nothing downstream means anything.
const G0_INSTRUMENT_TOL: f64 = 1e-11;
/// G-C2-1a: the norm is preserved EXACTLY by the integrator (unitary substeps on the
/// orthogonality centre), so this is a round-off band, not a physics band.
const G1_NORM_TOL: f64 = 1e-12;
/// G-C2-1b: likewise the energy. Drift here is a BUG, not a discretisation error.
const G1_ENERGY_TOL: f64 = 1e-10;
/// G-C2-2a: EXACTNESS at the natural cap, at EVERY step size. Not a convergence band —
/// a flat ceiling the integrator must sit under however coarsely it is stepped.
const G2A_EXACT_TOL: f64 = 1e-11;
/// G-C2-2b: the measured convergence order below the cap, two-sided.
const G2_ORDER_LO: f64 = 1.70;
const G2_ORDER_HI: f64 = 2.30;
/// G-C2-2b: the submanifold trajectory must actually be converging, not sitting on a floor.
const G2B_FINEST_ERR: f64 = 1e-4;
/// G-C2-3: a planted defect must miss a band by at least this factor, so "it fired" is
/// never a round-off coin flip.
const G3_MARGIN: f64 = 10.0;

// ----------------------------------------------------------------- helpers

fn build_system(chi: usize) -> (Mpo, Vec<CTensorSite>, Vec<f64>, usize) {
    let mpo = Mpo::from_hubbard(SITES, T_HOP, U_INT, MU);
    let l = 2 * SITES;
    let dim = 1usize << l;
    let h = mpo.dense();
    let init = q8_mps::tdvp::deterministic_state(l, chi, SEED);
    (mpo, init, h, dim)
}

/// `Re <psi|H|psi> / <psi|psi>` on the dense path.
fn dense_energy(h: &[f64], dim: usize, psi: &[Cx]) -> f64 {
    let mut num = 0.0;
    for i in 0..dim {
        let mut acc = Cx::ZERO;
        for j in 0..dim {
            let hij = h[i * dim + j];
            if hij != 0.0 {
                acc = acc.add(psi[j].scale(hij));
            }
        }
        num += psi[i].conj().mul(acc).re;
    }
    let den: f64 = psi.iter().map(|x| x.norm_sq()).sum();
    num / den
}

#[derive(Debug)]
struct Run {
    /// Relative distance to the exact propagator at `T_FINAL`, phase included.
    err: f64,
    /// Worst `| ||psi|| - 1 |` over the trajectory.
    norm_dev: f64,
    /// Worst relative energy drift over the trajectory.
    energy_dev: f64,
    /// Worst Krylov a-posteriori estimate, so the local solver reports its own accuracy.
    krylov_worst: f64,
    krylov_truncations: usize,
    /// The trajectory's own final state, so a run can be compared against another RUN and
    /// not only against the dense referee — which is what `G-C2-2b` needs.
    final_state: Vec<Cx>,
}

fn run_trajectory(
    mpo: &Mpo,
    init: &[CTensorSite],
    h: &[f64],
    dim: usize,
    n_steps: usize,
    mutation: Mutation,
) -> Run {
    let mut tdvp = Tdvp::new(mpo, init.to_vec());
    tdvp.mutation = mutation;
    let e0 = tdvp.energy();
    let dt = T_FINAL / n_steps as f64;

    let mut norm_dev: f64 = 0.0;
    let mut energy_dev: f64 = 0.0;
    for _ in 0..n_steps {
        tdvp.step(dt);
        norm_dev = norm_dev.max((tdvp.norm_squared().sqrt() - 1.0).abs());
        energy_dev = energy_dev.max((tdvp.energy() - e0).abs() / e0.abs().max(1.0));
    }

    let psi0 = to_dense(init);
    let exact = exact_propagate(h, dim, &psi0, T_FINAL);
    let got = tdvp.to_dense();
    Run {
        err: relative_distance(&got, &exact),
        norm_dev,
        energy_dev,
        krylov_worst: tdvp.worst_krylov_estimate,
        krylov_truncations: tdvp.krylov_truncations,
        final_state: got,
    }
}

// ----------------------------------------------------------------- G-C2-0

/// The instruments agree before anything moves. Four separate checks, because four
/// separate things could be wrong: the MPO builder against the crate's independently
/// gated `dense_from_mpo` (G1's object), the MPS-to-dense basis convention, the energy
/// contraction through the environments, and the referee itself.
#[test]
fn g_c2_0_instruments_agree() {
    let (mpo, init, h, dim) = build_system(CHI_FULL);

    // (a) the builder's MPO is the same operator G1 already gated.
    let reference = dense_from_mpo(SITES, T_HOP, U_INT, MU);
    let mut worst = 0.0f64;
    for (a, b) in h.iter().zip(reference.iter()) {
        worst = worst.max((a - b).abs());
    }
    assert!(
        worst <= 1e-13,
        "G-C2-0a: Mpo::from_hubbard().dense() differs from dense_from_mpo by {worst:e}"
    );

    // (b) the MPS contracts to a normalised vector in the expected basis.
    let psi0 = to_dense(&init);
    assert_eq!(psi0.len(), dim);
    let n: f64 = psi0.iter().map(|x| x.norm_sq()).sum();
    assert!(
        (n - 1.0).abs() <= 1e-12,
        "G-C2-0b: the canonicalised start is not normalised: <psi|psi> = {n:.17}"
    );

    // (c) the environment-contracted energy is the dense energy.
    let tdvp = Tdvp::new(&mpo, init.clone());
    let e_mps = tdvp.energy();
    let e_dense = dense_energy(&h, dim, &psi0);
    let rel = (e_mps - e_dense).abs() / e_dense.abs().max(1.0);
    assert!(
        rel <= G0_INSTRUMENT_TOL,
        "G-C2-0c: MPS energy {e_mps:.15} vs dense {e_dense:.15}, relative {rel:e} > {G0_INSTRUMENT_TOL:e}"
    );

    // (d) the referee is a propagator: the identity at t = 0, and unitary after that.
    let same = exact_propagate(&h, dim, &psi0, 0.0);
    assert!(
        relative_distance(&same, &psi0) <= 1e-12,
        "G-C2-0d: the exact propagator is not the identity at t = 0"
    );
    let moved = exact_propagate(&h, dim, &psi0, T_FINAL);
    let nm: f64 = moved.iter().map(|x| x.norm_sq()).sum();
    assert!(
        (nm - 1.0).abs() <= 1e-11,
        "G-C2-0d: the exact propagator is not unitary: ||psi(T)||^2 = {nm:.17}"
    );
}

// ----------------------------------------------------------------- G-C2-1, G-C2-2a

/// Conservation and EXACTNESS at the natural bond cap.
///
/// Norm and energy are conserved by construction (every substep is `exp(-i theta H_eff)`
/// with `H_eff` Hermitian, applied to the orthogonality centre of a canonical MPS), so a
/// drift here is a bug and never a step-size effect. The trajectory gate is a flat ceiling
/// rather than a convergence band, and the coarsest step is the interesting one: at
/// `n = 4` the step is a quarter of the whole interval and the answer is still right to
/// `1e-13`.
#[test]
fn g_c2_1_and_2a_conservation_and_exactness_at_the_cap() {
    let (mpo, init, h, dim) = build_system(CHI_FULL);

    for &n in STEP_COUNTS.iter() {
        let r = run_trajectory(&mpo, &init, &h, dim, n, Mutation::None);
        println!(
            "G-C2-2a  n={n:3}  dt={:.6}  err={:.4e}  norm_dev={:.3e}  energy_dev={:.3e}  krylov={:.3e} trunc={}",
            T_FINAL / n as f64,
            r.err,
            r.norm_dev,
            r.energy_dev,
            r.krylov_worst,
            r.krylov_truncations
        );
        assert!(
            r.norm_dev <= G1_NORM_TOL,
            "G-C2-1a: n={n}: norm deviation {:.3e} > {G1_NORM_TOL:e}",
            r.norm_dev
        );
        assert!(
            r.energy_dev <= G1_ENERGY_TOL,
            "G-C2-1b: n={n}: energy drift {:.3e} > {G1_ENERGY_TOL:e} — the substeps are \
             unitary on the orthogonality centre, so this is a BUG, not a step-size error",
            r.energy_dev
        );
        assert_eq!(
            r.krylov_truncations, 0,
            "G-C2-1c: n={n}: {} Krylov solves hit the subspace cap without converging, so \
             the local solver's error would be an unmeasured confound in everything above",
            r.krylov_truncations
        );
        assert!(
            r.err <= G2A_EXACT_TOL,
            "G-C2-2a: n={n}: distance to the exact propagator {:.4e} > {G2A_EXACT_TOL:e}. At \
             the natural cap the manifold contains the exact trajectory, so the \
             projector-splitting integrator must reproduce it with NO step-size error",
            r.err
        );
    }
}

// ----------------------------------------------------------------- G-C2-2b

/// SECOND ORDER below the cap.
///
/// On a genuine submanifold the integrator follows the TDVP flow, not the Schrodinger
/// flow, so the exact propagator is the wrong referee and self-convergence against a
/// fine-step reference is the right one. This is also the only regime that can tell the
/// symmetric sweep from the one-directional one: at the cap both are exact, so the
/// palindrome looks free there and its planted removal is invisible.
#[test]
fn g_c2_2b_second_order_below_the_cap() {
    let (mpo, init, h, dim) = build_system(CHI_SUB);

    let reference = run_trajectory(&mpo, &init, &h, dim, N_REF_SUB, Mutation::None);
    println!(
        "G-C2-2b  reference n={N_REF_SUB}  distance-to-exact={:.4e}  (the PROJECTION error: \
         what a chi={CHI_SUB} manifold cannot represent, and not a step-size error)",
        reference.err
    );

    let mut errs = Vec::new();
    for &n in STEP_COUNTS_SUB.iter() {
        let r = run_trajectory(&mpo, &init, &h, dim, n, Mutation::None);
        let e = relative_distance(&r.final_state, &reference.final_state);
        println!(
            "G-C2-2b  n={n:3}  dt={:.6}  self-err={e:.4e}  energy_dev={:.3e}",
            T_FINAL / n as f64,
            r.energy_dev
        );
        assert!(
            r.energy_dev <= G1_ENERGY_TOL,
            "G-C2-1b (below the cap): n={n}: energy drift {:.3e} > {G1_ENERGY_TOL:e}",
            r.energy_dev
        );
        errs.push(e);
    }

    let finest = *errs.last().unwrap();
    assert!(
        finest <= G2B_FINEST_ERR,
        "G-C2-2b: finest-step self-error {finest:.4e} > {G2B_FINEST_ERR:e}"
    );
    for w in 0..errs.len() - 1 {
        let order = (errs[w] / errs[w + 1]).log2();
        println!(
            "G-C2-2b  order({} -> {}) = {order:.4}",
            STEP_COUNTS_SUB[w], STEP_COUNTS_SUB[w + 1]
        );
        assert!(
            (G2_ORDER_LO..=G2_ORDER_HI).contains(&order),
            "G-C2-2b: measured order {order:.4} between n={} and n={} is outside the staked \
             band [{G2_ORDER_LO}, {G2_ORDER_HI}] (errors {:.4e} and {:.4e})",
            STEP_COUNTS_SUB[w],
            STEP_COUNTS_SUB[w + 1],
            errs[w],
            errs[w + 1]
        );
    }
}

// ----------------------------------------------------------------- G-C2-3

/// Which staked prong a planted defect broke. A gate that has never failed has never
/// gated, and an OR-gate that only ever fires on one disjunct is a one-prong gate — so
/// the prong is named, per mutation, and printed.
fn firing_prongs(r: &Run, sub_order: Option<f64>) -> Vec<&'static str> {
    let mut v = Vec::new();
    if r.norm_dev > G1_NORM_TOL * G3_MARGIN {
        v.push("G-C2-1a norm");
    }
    if r.energy_dev > G1_ENERGY_TOL * G3_MARGIN {
        v.push("G-C2-1b energy");
    }
    if r.err > G2A_EXACT_TOL * G3_MARGIN {
        v.push("G-C2-2a exactness at the cap");
    }
    if let Some(o) = sub_order {
        if !(G2_ORDER_LO..=G2_ORDER_HI).contains(&o) {
            v.push("G-C2-2b order below the cap");
        }
    }
    v
}

/// The measured self-convergence order below the cap for one mutation, using the CLEAN
/// integrator's fine-step run as the reference. That choice is deliberate: a mutated
/// integrator's own fine-step limit is a different flow, so self-referencing it would let
/// a consistent-but-wrong scheme pass by converging neatly to the wrong answer.
fn sub_manifold_order(mpo: &Mpo, init: &[CTensorSite], h: &[f64], dim: usize, m: Mutation) -> f64 {
    let reference = run_trajectory(mpo, init, h, dim, N_REF_SUB, Mutation::None);
    let coarse = run_trajectory(mpo, init, h, dim, 16, m);
    let fine = run_trajectory(mpo, init, h, dim, 32, m);
    let ec = relative_distance(&coarse.final_state, &reference.final_state);
    let ef = relative_distance(&fine.final_state, &reference.final_state);
    (ec / ef).log2()
}

#[test]
fn g_c2_3_planted_mutations_fire() {
    let (mpo, init, h, dim) = build_system(CHI_FULL);
    let (mpo_s, init_s, h_s, dim_s) = build_system(CHI_SUB);
    let n = *STEP_COUNTS.last().unwrap();

    let clean = run_trajectory(&mpo, &init, &h, dim, n, Mutation::None);
    let clean_order = sub_manifold_order(&mpo_s, &init_s, &h_s, dim_s, Mutation::None);
    println!("G-C2-3  control: err={:.4e} sub-order={clean_order:.4}", clean.err);
    assert!(
        firing_prongs(&clean, Some(clean_order)).is_empty(),
        "G-C2-3 control: the UNMUTATED run fires a prong, so the battery is measuring the \
         instrument rather than the mutation. err={:.4e} norm_dev={:.3e} energy_dev={:.3e} \
         sub-order={clean_order:.4}",
        clean.err,
        clean.norm_dev,
        clean.energy_dev
    );

    // Each mutation is paired with the prong it MUST break, and the pairing is the point.
    // Two of the four are wrong GENERATORS and show up as a finite error at the cap where
    // the correct integrator has none. ForwardSweepOnly is a wrong ORDER, not a wrong
    // generator: it is exact at the cap too (the exactness property does not care about
    // the sweep direction), so only the submanifold order sees it — which is exactly why
    // that gate had to exist. The fourth breaks Hermiticity, which the energy sees.
    let cases: [(Mutation, &str, bool); 4] = [
        (Mutation::NoBondBackstep, "G-C2-2a exactness at the cap", false),
        (Mutation::BondBackstepWrongSign, "G-C2-2a exactness at the cap", false),
        (Mutation::ForwardSweepOnly, "G-C2-2b order below the cap", true),
        (Mutation::LeftEnvNoConjugate, "G-C2-1b energy", false),
    ];

    for (mutation, required, needs_order) in cases {
        let r = run_trajectory(&mpo, &init, &h, dim, n, mutation);
        let order = if needs_order {
            Some(sub_manifold_order(&mpo_s, &init_s, &h_s, dim_s, mutation))
        } else {
            None
        };
        let fired = firing_prongs(&r, order);
        println!(
            "G-C2-3  {mutation:?}: err={:.4e} norm_dev={:.3e} energy_dev={:.3e} order={order:?} \
             -> fired {fired:?}",
            r.err, r.norm_dev, r.energy_dev
        );
        assert!(
            fired.contains(&required),
            "G-C2-3: {mutation:?} did not fire its named prong `{required}`. Fired: {fired:?}. \
             A planted defect the intended sensor cannot see means the sensor is dead, not \
             that the defect is harmless. err={:.4e} norm_dev={:.3e} energy_dev={:.3e} \
             order={order:?}",
            r.err,
            r.norm_dev,
            r.energy_dev
        );
    }
}

// ----------------------------------------------------------------- G-C2-4

/// The tower's own claim about this carrier, checked rather than asserted: the C2 state is
/// a genuinely complex object that genuinely moves. If the imaginary weight stayed at
/// round-off, every complex-only defect above would be untestable; if the state barely
/// moved, every trajectory band would be measuring nothing.
#[test]
fn g_c2_4_the_dynamics_are_actually_complex_and_actually_move() {
    let (mpo, init, h, dim) = build_system(CHI_FULL);
    let n = *STEP_COUNTS.last().unwrap();
    let mut tdvp = Tdvp::new(&mpo, init.clone());
    for _ in 0..n {
        tdvp.step(T_FINAL / n as f64);
    }
    let psi = tdvp.to_dense();
    let im: f64 = psi.iter().map(|x| x.im * x.im).sum::<f64>().sqrt();
    assert!(
        im > 1e-3,
        "G-C2-4: the evolved state carries imaginary weight {im:.3e}; a real trajectory \
         would make every complex-only planted defect unobservable"
    );
    let psi0 = to_dense(&init);
    let exact = exact_propagate(&h, dim, &psi0, T_FINAL);
    let moved = relative_distance(&psi0, &exact);
    assert!(
        moved > 0.1,
        "G-C2-4: the state barely moved over the whole trajectory (relative distance \
         {moved:.3e}); a gate on a static state measures nothing"
    );
}

// ----------------------------------------------------------------- the price

/// MEASURED price per TDVP substep, banked so the C2 carrier's `price_per_substep` is a
/// measurement and not a guess. `#[ignore]`d because a timing is not a correctness gate
/// and because a wall-clock number inside a gate is a placement lottery
/// (`M-PLACEMENT-LOTTERY`); run it deliberately:
///
/// ```text
/// cargo test --release -p q8-mps --test c2_tdvp_gates -- --ignored --nocapture c2_price
/// ```
#[test]
#[ignore = "timing, not correctness: run deliberately to re-bank the C2 price"]
fn c2_price_measurement() {
    for (sites, chi) in [(3usize, 8usize), (4, 16), (5, 32), (6, 64)] {
        let mpo = Mpo::from_hubbard(sites, T_HOP, U_INT, MU);
        let l = 2 * sites;
        let init = q8_mps::tdvp::deterministic_state(l, chi, SEED);
        let mut tdvp = Tdvp::new(&mpo, init);
        let n = 8usize;
        let t0 = std::time::Instant::now();
        for _ in 0..n {
            tdvp.step(0.01);
        }
        let per = t0.elapsed().as_secs_f64() / n as f64;
        println!(
            "C2 price  L={l:2} chi={chi:3} bond_dims={:?}  {per:.6} s/step",
            mpo.bond_dims()
        );
    }
}
