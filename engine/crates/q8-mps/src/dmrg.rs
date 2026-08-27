//! The two-site DMRG sweep — `Q8_MPS_PREREG.md` §2, §4 (G7: fixed schedule, no post-hoc
//! extension).
//!
//! Environments are precomputed ONCE per pass, not per bond. Within a left-to-right pass, a
//! bond-`j` update never touches sites `j+2` onward — no earlier bond in the same pass can have
//! either — so the FULL set of right environments computed at the pass's start stays valid for
//! every bond, and the left environment only needs one `grow_left` call per bond (using the
//! JUST-UPDATED tensor) rather than a fresh `O(j)` rebuild. `O(L)` environment work per pass,
//! not `O(L^2)`. This replaced an earlier from-scratch-every-bond version once the target sizes
//! (chi up to 256, L up to 24) made the quadratic version impractically slow — a correctness-
//! neutral change, not a re-derivation: same contractions (`mps::grow_left`/`grow_right`), same
//! per-bond two-site solve, just not re-run redundantly.

use crate::lanczos;
use crate::mpo;
use crate::mps::{self, Env, TensorSite};

pub struct Params {
    pub sites: usize,
    pub t: f64,
    pub u: f64,
    /// The bond ledger — every truncation in this run is capped here (§2's single uniform
    /// scalar; the physical near-boundary cap `min(chi, 2^b, 2^(L-b))` needs no separate code,
    /// it falls out of the SVD's own economy rank at each bond).
    pub chi_max: usize,
    pub max_sweeps: usize,
    pub sweep_tol: f64,
}

pub struct SweepResult {
    pub tensors: Vec<TensorSite>,
    /// `<H'>` — the WORKING (shifted) Hamiltonian's energy, from the last two-site solve of the
    /// last sweep. Unshift with the integer `N_target`, never a measured expectation, per
    /// `Q8_MPS_PREREG.md` §2's pinned convention — this module does not do that unshift itself,
    /// callers do, so the convention lives in exactly one place (the prereg's own wording).
    pub energy_shifted: f64,
    pub sweeps_used: usize,
    pub converged: bool,
    /// Per-bond discarded weight from the run's LAST full sweep only — earlier sweeps' values
    /// are corrected by later optimization and are not representative (`Q8_MPS_PREREG.md` §5).
    pub discarded_weight: Vec<f64>,
    /// THE FENCE (`Q10_PREREG.md` §3a): per-bond kept-spectrum floor `s_min / s_max`, from the
    /// run's LAST full sweep only — same convention and same reason as `discarded_weight`. The
    /// campaign quantity is the MINIMUM over bonds; this vector is kept per-bond so the fence
    /// can be localized rather than only scored. Never an error estimate (§2).
    pub spectrum_floor: Vec<f64>,
    /// `<H'>` (shifted) at the end of EVERY full sweep, in order — G3-primary's own data
    /// (`Q8_MPS_PREREG.md` §4: the floor and monotone-non-increase clauses both need the
    /// per-sweep trajectory, not just the final value; research-manager's Defect 2 — this field
    /// did not exist before, so G3-primary was staked but never actually checkable).
    pub energy_history: Vec<f64>,
    /// The two invariants a correct sweep relies on but that G3/G6/G2 never gated directly
    /// (team-lead/chief-of-staff-2 finding, N=8 U=16: `discarded_max=0` with a `2.55e-2`
    /// non-monotone energy rise — a correct update with zero truncation CANNOT raise the
    /// energy, so either canonical form or the local solve itself is suspect, not the physics).
    /// Worst value across every two-site update in the run. `mps::identity_defect` on the LEFT
    /// environment's START channel / RIGHT environment's FINISH channel.
    pub worst_left_canonical_defect: f64,
    pub worst_right_canonical_defect: f64,
    /// Worst `lanczos::TwoSiteGroundState::residual` across every local solve — computed
    /// unconditionally by `lanczos::ground_state` already, but never previously read past the
    /// `.expect(...)` that only checks the function returned `Some`, not that the residual was
    /// actually small.
    pub worst_lanczos_residual: f64,
}

/// The G5 refusal threshold (`Q8_MPS_PREREG.md` §6) — STAKED, and deliberately independent of
/// G4's discarded-weight-to-error calibration (a chosen safety trigger, not a derived one; a
/// later pass could unify them, not built speculatively here).
pub const REFUSAL_THRESHOLD: f64 = 1e-4;

/// A bond's declared ledger was exceeded — FLOOR-type (`GrainFloor.lean`'s taxonomy): a larger
/// `chi_max` serves this request, it is not that nothing finer exists.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Refusal {
    pub bond: usize,
    pub weight: f64,
}

/// `Typed` is the real, default policy — it names the worst bond and refuses rather than
/// silently returning a state whose truncation error is unbounded. `Silent` shares every
/// numeric routine with `Typed`, differing solely in whether the same discarded-weight check
/// gates the return value; its primary purpose is the G5 mutation test (`tests/g5_refusal.rs`),
/// proving the gate discriminates rather than merely asserting it does, but it is also the
/// correct tool to isolate the TRUNCATION MATH from the (also correct) refusal firing in a
/// smoke test that is not testing refusal itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RefusalPolicy {
    Typed,
    Silent,
}

/// The pinned Néel product state, `chi=1` — `Q8_MPS_PREREG.md` §2's fixed initial state.
pub fn run(p: &Params, policy: RefusalPolicy) -> Result<SweepResult, Refusal> {
    run_from(p, policy, mps::initial_state(p.sites))
}

/// Same sweep, starting from caller-supplied tensors instead of the pinned product state — the
/// chi-warm-start entry point (`Q9`'s probe and, pending its own prereg/gates, its remedy).
/// NOT part of any staked gate yet; `run` is what every staked gate calls.
pub fn run_from(
    p: &Params,
    policy: RefusalPolicy,
    initial_tensors: Vec<TensorSite>,
) -> Result<SweepResult, Refusal> {
    let l = 2 * p.sites;
    let mu = p.u / 2.0;
    let mut tensors = initial_tensors;
    let w: Vec<Vec<f64>> =
        (0..l).map(|j| mpo::w_dense(mps::is_up_orbital(j), p.t, p.u, mu)).collect();

    let mut prev_energy = f64::INFINITY;
    let mut last_energy = 0.0;
    let mut converged = false;
    let mut sweeps_used = 0;
    let mut discarded = vec![0.0; l - 1];
    let mut spectrum_floor = vec![0.0; l - 1];
    let mut energy_history: Vec<f64> = Vec::with_capacity(p.max_sweeps);
    let mut worst_left_canonical_defect = 0.0f64;
    let mut worst_right_canonical_defect = 0.0f64;
    let mut worst_lanczos_residual = 0.0f64;

    for sweep in 0..p.max_sweeps {
        sweeps_used = sweep + 1;

        // Left-to-right: S absorbed into the RIGHT tensor of each pair, moving the
        // orthogonality center rightward. Right environments for every bond are valid from the
        // state at the pass's start (untouched until this pass reaches them); the left
        // environment is carried forward incrementally.
        let right_envs = all_right_envs(&tensors, &w);
        let mut left_env = mps::trivial_left_env();
        for j in 0..(l - 1) {
            worst_left_canonical_defect =
                worst_left_canonical_defect.max(mps::identity_defect(&left_env, mpo::START));
            worst_right_canonical_defect = worst_right_canonical_defect
                .max(mps::identity_defect(&right_envs[j + 2], mpo::FINISH));
            let (e, dw, resid, sf) =
                two_site_update(&mut tensors, &w, j, p.chi_max, false, &left_env, &right_envs[j + 2]);
            last_energy = e;
            discarded[j] = dw;
            spectrum_floor[j] = sf;
            worst_lanczos_residual = worst_lanczos_residual.max(resid);
            if policy == RefusalPolicy::Typed && dw > REFUSAL_THRESHOLD {
                return Err(Refusal { bond: j, weight: dw });
            }
            left_env = mps::grow_left(&left_env, &w[j], &tensors[j]);
        }

        // Right-to-left: mirror.
        let left_envs = all_left_envs(&tensors, &w);
        let mut right_env = mps::trivial_right_env();
        for j in (0..(l - 1)).rev() {
            worst_left_canonical_defect =
                worst_left_canonical_defect.max(mps::identity_defect(&left_envs[j], mpo::START));
            worst_right_canonical_defect =
                worst_right_canonical_defect.max(mps::identity_defect(&right_env, mpo::FINISH));
            let (e, dw, resid, sf) =
                two_site_update(&mut tensors, &w, j, p.chi_max, true, &left_envs[j], &right_env);
            last_energy = e;
            discarded[j] = dw;
            spectrum_floor[j] = sf;
            worst_lanczos_residual = worst_lanczos_residual.max(resid);
            if policy == RefusalPolicy::Typed && dw > REFUSAL_THRESHOLD {
                return Err(Refusal { bond: j, weight: dw });
            }
            right_env = mps::grow_right(&right_env, &w[j + 1], &tensors[j + 1]);
        }

        energy_history.push(last_energy);

        if (last_energy - prev_energy).abs() <= p.sweep_tol {
            converged = true;
            break;
        }
        prev_energy = last_energy;
    }

    Ok(SweepResult {
        tensors,
        energy_shifted: last_energy,
        sweeps_used,
        converged,
        discarded_weight: discarded,
        spectrum_floor,
        energy_history,
        worst_left_canonical_defect,
        worst_right_canonical_defect,
        worst_lanczos_residual,
    })
}

/// `envs[k]` summarizes sites `k..L-1`; `envs[L]` is the trivial boundary. One `O(L)` backward
/// pass, computed once per left-to-right sweep.
fn all_right_envs(tensors: &[TensorSite], w: &[Vec<f64>]) -> Vec<Env> {
    let l = tensors.len();
    let mut envs: Vec<Env> = vec![mps::trivial_right_env(); l + 1];
    for k in (0..l).rev() {
        envs[k] = mps::grow_right(&envs[k + 1], &w[k], &tensors[k]);
    }
    envs
}

/// `envs[k]` summarizes sites `0..k-1`; `envs[0]` is the trivial boundary. Mirror of
/// `all_right_envs`, computed once per right-to-left sweep.
fn all_left_envs(tensors: &[TensorSite], w: &[Vec<f64>]) -> Vec<Env> {
    let l = tensors.len();
    let mut envs: Vec<Env> = vec![mps::trivial_left_env(); l + 1];
    for k in 0..l {
        envs[k + 1] = mps::grow_left(&envs[k], &w[k], &tensors[k]);
    }
    envs
}

fn two_site_update(
    tensors: &mut [TensorSite],
    w: &[Vec<f64>],
    j: usize,
    chi_max: usize,
    absorb_s_left: bool,
    left_env: &Env,
    right_env: &Env,
) -> (f64, f64, f64, f64) {
    let chi_l = tensors[j].chi_l;
    let chi_r = tensors[j + 1].chi_r;
    let mid = tensors[j].chi_r;
    debug_assert_eq!(mid, tensors[j + 1].chi_l, "adjacent bond dimensions out of sync");

    // seed[l,a,b,r] = sum_m A_j[a,l,m] . A_{j+1}[b,m,r] — the current two-site tensor,
    // contracted through its shared bond, as the deterministic Lanczos warm start.
    let mut seed = vec![0.0; chi_l * 2 * 2 * chi_r];
    for lft in 0..chi_l {
        for a in 0..2 {
            for m in 0..mid {
                let av = tensors[j].get(a, lft, m);
                if av == 0.0 {
                    continue;
                }
                let base = (lft * 2 + a) * 2 * chi_r;
                for b in 0..2 {
                    for r in 0..chi_r {
                        seed[base + b * chi_r + r] += av * tensors[j + 1].get(b, m, r);
                    }
                }
            }
        }
    }

    let dim = chi_l * 2 * 2 * chi_r;
    let w1 = &w[j];
    let w2 = &w[j + 1];
    let gs = lanczos::ground_state(
        |psi| mps::apply_effective_h(left_env, w1, w2, right_env, psi, chi_l, chi_r),
        &seed,
        dim,
    )
    .expect("local Lanczos failed to converge");

    let (a_left, a_right, discarded, spectrum_floor) =
        mps::split_two_site(&gs.vector, chi_l, chi_r, chi_max, absorb_s_left);
    tensors[j] = a_left;
    tensors[j + 1] = a_right;

    (gs.energy, discarded, gs.residual, spectrum_floor)
}
