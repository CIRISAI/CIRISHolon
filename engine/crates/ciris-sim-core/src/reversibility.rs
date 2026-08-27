//! **The float production floor: entropy this simulator produces because it is made of
//! floats, measured rather than argued.**
//!
//! # Why this quantity and not another
//!
//! `CIRISOntology/Core/Habit.lean` pins the second law in this object's vocabulary, and
//! the load-bearing clause for a *simulator* is that a deterministic step map produces
//! entropy exactly where it fails to be injective —
//! `production_eq_zero_iff_rate_injective`. Velocity Verlet is time-reversible **in exact
//! arithmetic**: negate the velocities, run the same map forward, negate again, and you
//! are back where you started. It is therefore injective, and its entropy production is
//! exactly zero — *as mathematics*.
//!
//! In floats it is not injective, because rounding merges states. **That merging is the
//! only entropy a deterministic simulator has**, and this module counts it: step forward
//! `n`, reverse `n`, and count the initial states that do not come back. The count is a
//! floor on production, in states — the units the theorem is stated in — rather than a
//! proxy in joules or an argument from principle.
//!
//! **Units: nats, with bits in parentheses.** `Core/StochasticHabit.lean` proves
//! `frameEntropy` and Shannon entropy are the same quantity on the uniform-on-fiber state,
//! using `Real.log` — natural log — so nats is the lake's currency and this floor is quoted
//! in it. One merge destroys exactly one bit, and **1 bit = ln 2 = 0.693147… nats**. The
//! measured floor is **0.0514 nats (0.0742 bits)** per ULP-adjacent pair.
//!
//! # Two tolerances, because they are two different measurements
//!
//! Stated explicitly rather than chosen silently:
//!
//! * **exact** — every `f64` of position and velocity returns *bit-identical*
//!   (`to_bits()` equality). This is the one the theorem wants: two states that differ in
//!   any bit are distinguishable, so a step that merges them has destroyed information.
//! * **epsilon** — every component returns within a declared absolute tolerance. This is
//!   the engineering question ("did the trajectory come back?") and it is a *weaker* count:
//!   `not_returned_eps ≤ not_returned_exact` always, since bit-identity implies
//!   within-epsilon.
//!
//! The exact count is the floor. The epsilon count is reported beside it so the gap
//! between "information destroyed" and "visibly wrong" is visible instead of conflated.
//!
//! # What this is NOT
//!
//! A floor, not the production. It counts states that fail to return under **one specific
//! reversal protocol** at one `dt`; a state that returns may still have passed through a
//! merge and come back by luck, so the true production is at least this. It also says
//! nothing about the noisy half of the object — `Habit.lean`'s header is emphatic that
//! `production_nonneg_of_closed` is **false** for stochastic maps, and nothing here is
//! evidence about one.
//!
//! Scope of the reversal: `damping` must be `1.0`. A damped step is not time-reversible
//! even in exact arithmetic — it is a contraction, and measuring its failure to reverse
//! would measure the damping, not the floats. [`float_production_floor`] refuses a damped
//! `Params` rather than returning a number that looks like a floor and is not.

use crate::dynamics::{step, Params, State};
use crate::structure::Structure;

/// One reading of the floor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloorReading {
    /// Initial states drawn.
    pub ensemble: usize,
    /// Forward steps before reversal (and reverse steps after).
    pub steps: usize,
    /// States that did not return **bit-identically**. This is the floor.
    pub not_returned_exact: usize,
    /// States that did not return within [`FloorReading::epsilon`].
    pub not_returned_eps: usize,
    /// The tolerance the epsilon count used.
    pub epsilon: f64,
    /// Largest absolute component drift observed over the whole ensemble.
    pub worst_drift: f64,
}

impl FloorReading {
    /// The floor as a fraction of the ensemble — the number to quote.
    pub fn exact_fraction(&self) -> f64 {
        if self.ensemble == 0 {
            return 0.0;
        }
        self.not_returned_exact as f64 / self.ensemble as f64
    }

    pub fn eps_fraction(&self) -> f64 {
        if self.ensemble == 0 {
            return 0.0;
        }
        self.not_returned_eps as f64 / self.ensemble as f64
    }
}

/// Why a floor could not be measured. Refusals, not silent zeros.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorError {
    /// `damping != 1.0`. A damped step is a contraction and is not time-reversible even
    /// in exact arithmetic, so its failure to return measures the damping rather than the
    /// float. Refused rather than answered.
    Damped,
    /// `dt` is not finite, or is zero. A zero step is trivially reversible and would
    /// report a floor of zero that says nothing about the arithmetic.
    DegenerateStep,
}

/// Reverse `n` velocity-Verlet steps in place.
///
/// The reversal is the standard one and it is exact in real arithmetic: negate every
/// velocity, run the **same** forward map `n` times, negate again. Nothing here uses a
/// negative `dt`, so the reversed leg executes bit-for-bit the same code path as the
/// forward leg — which matters, because a differently-rounded reverse map would measure
/// the difference between two implementations rather than the irreversibility of one.
pub fn reverse<const N: usize>(
    state: &mut State<N>,
    st: &Structure<N>,
    params: &Params,
    symmetrised: bool,
    n: usize,
) {
    negate_velocities(state);
    for _ in 0..n {
        step(state, st, params, symmetrised);
    }
    negate_velocities(state);
}

fn negate_velocities<const N: usize>(state: &mut State<N>) {
    for i in 0..N {
        for k in 0..3 {
            // Negation is exact in IEEE-754 — it flips the sign bit and touches nothing
            // else — so the reversal protocol itself contributes no rounding.
            state.vel[i][k] = -state.vel[i][k];
        }
    }
}

/// **The measurement.** Draw `ensemble` initial states, run each forward `steps` and back,
/// and count those that do not return.
///
/// The draw is a pure function of `seed`, so a reading is reproducible bit-for-bit on any
/// host; nothing here consults a clock or an allocator.
pub fn float_production_floor<const N: usize>(
    st: &Structure<N>,
    params: &Params,
    symmetrised: bool,
    steps: usize,
    ensemble: usize,
    seed: u64,
    epsilon: f64,
) -> Result<FloorReading, FloorError> {
    if params.damping != 1.0 {
        return Err(FloorError::Damped);
    }
    if !params.dt.is_finite() || params.dt == 0.0 {
        return Err(FloorError::DegenerateStep);
    }

    let mut not_returned_exact = 0usize;
    let mut not_returned_eps = 0usize;
    let mut worst_drift = 0.0f64;

    for member in 0..ensemble {
        let start = draw_state::<N>(seed, member as u64);
        let mut s = start;
        for _ in 0..steps {
            step(&mut s, st, params, symmetrised);
        }
        reverse(&mut s, st, params, symmetrised, steps);

        let mut exact = true;
        let mut drift = 0.0f64;
        for i in 0..N {
            for k in 0..3 {
                if s.pos[i][k].to_bits() != start.pos[i][k].to_bits()
                    || s.vel[i][k].to_bits() != start.vel[i][k].to_bits()
                {
                    exact = false;
                }
                let dp = abs(s.pos[i][k] - start.pos[i][k]);
                let dv = abs(s.vel[i][k] - start.vel[i][k]);
                if dp > drift {
                    drift = dp;
                }
                if dv > drift {
                    drift = dv;
                }
            }
        }
        if !exact {
            not_returned_exact += 1;
        }
        if drift > epsilon {
            not_returned_eps += 1;
        }
        if drift > worst_drift {
            worst_drift = drift;
        }
    }

    Ok(FloorReading {
        ensemble,
        steps,
        not_returned_exact,
        not_returned_eps,
        epsilon,
        worst_drift,
    })
}

#[inline]
fn abs(x: f64) -> f64 {
    if x < 0.0 {
        -x
    } else {
        x
    }
}

/// SplitMix64. Local and tiny so the instrument carries no dependency and no global state;
/// the same `(seed, member)` gives the same state on every host.
fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut x = z;
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// A state in `[-1, 1]^{3N} × [-1, 1]^{3N}`, scaled small so the harmonic regime holds and
/// the trajectory does not run off to where the forces are meaningless.
fn draw_state<const N: usize>(seed: u64, member: u64) -> State<N> {
    let mut s = State::<N>::ZERO;
    let mut counter = mix64(seed ^ member.wrapping_mul(0x1234_5678_9ABC_DEF));
    let mut next = || -> f64 {
        counter = mix64(counter);
        // 53 bits of mantissa into [-1, 1). Exact: the numerator is an integer below 2^53
        // and the divisor is a power of two, so the conversion introduces no rounding of
        // its own and the drawn state is exactly representable.
        let unit = (counter >> 11) as f64 / (1u64 << 53) as f64;
        2.0 * unit - 1.0
    };
    for i in 0..N {
        for k in 0..3 {
            s.pos[i][k] = 0.5 * next();
            s.vel[i][k] = 0.1 * next();
        }
    }
    s
}

/// One reading of the **merge rate** — the quantity `Habit.lean`'s production is actually
/// about.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MergeReading {
    /// ULP-adjacent pairs tested.
    pub pairs: usize,
    pub steps: usize,
    /// Pairs that became bit-identical: two distinguishable states became one. Each is an
    /// injectivity failure, and destroys exactly one bit.
    pub merged: usize,
    /// Pairs still exactly one ULP apart — neither merged nor separated.
    pub still_adjacent: usize,
}

impl MergeReading {
    /// **Nats destroyed per pair — the primary reading.**
    ///
    /// Nats, not bits, because nats are the currency the theorems speak:
    /// `Core/StochasticHabit.lean` proves `frameEntropy` and Shannon entropy agree on the
    /// uniform-on-fiber state using `Real.log`, i.e. natural log throughout. Reporting this
    /// floor in bits and the lake's fiber counts in nats would leave two currencies to
    /// reconcile at closeout for no reason.
    ///
    /// A merged pair took two distinguishable states to one, destroying exactly one bit —
    /// `ln 2` nats. The measured floor is **0.0514 nats (0.0742 bits)** per ULP-adjacent
    /// pair; both figures are stated here, at the definition, and nowhere is one quietly
    /// substituted for the other.
    ///
    /// Computed here rather than converted from [`Self::bits_per_pair`]: the primary reading
    /// should not be a derived quantity of the secondary one, or the convention is only in
    /// the prose.
    pub fn nats_per_pair(&self) -> f64 {
        if self.pairs == 0 {
            return 0.0;
        }
        self.merged as f64 / self.pairs as f64 * core::f64::consts::LN_2
    }

    /// The same quantity in bits, kept because "one merge destroys one bit" is the clearer
    /// statement of the mechanism. **1 bit = ln 2 = 0.693147… nats**; the measured floor is
    /// 0.0514 nats = 0.0742 bits per ULP-adjacent pair.
    pub fn bits_per_pair(&self) -> f64 {
        self.nats_per_pair() / core::f64::consts::LN_2
    }
}

/// The next representable `f64` away from zero — a one-ULP perturbation.
#[inline]
pub fn next_ulp(x: f64) -> f64 {
    let bits = x.to_bits();
    if x >= 0.0 {
        f64::from_bits(bits + 1)
    } else {
        f64::from_bits(bits - 1)
    }
}

/// Do these two states become one after `steps`? The detector under everything below,
/// exposed so it can be aimed at states a caller constructs rather than only at the ones
/// [`ulp_merge_rate`] draws — which is what makes its controls possible.
pub fn merges<const N: usize>(
    a0: &State<N>,
    b0: &State<N>,
    st: &Structure<N>,
    params: &Params,
    symmetrised: bool,
    steps: usize,
) -> bool {
    let (mut a, mut b) = (*a0, *b0);
    for _ in 0..steps {
        step(&mut a, st, params, symmetrised);
        step(&mut b, st, params, symmetrised);
    }
    for i in 0..N {
        for k in 0..3 {
            if a.pos[i][k].to_bits() != b.pos[i][k].to_bits()
                || a.vel[i][k].to_bits() != b.vel[i][k].to_bits()
            {
                return false;
            }
        }
    }
    true
}

/// **The merge rate: how often the step map takes two distinguishable states to one.**
///
/// This is the measurement [`float_production_floor`] is a proxy for, and it is the one in
/// the theorem's units. `Core/Habit.lean`'s `production_eq_zero_iff_rate_injective` says
/// production is the failure of INJECTIVITY, and injectivity fails exactly when two
/// distinct inputs give one output. A round trip that fails to return is *evidence* of
/// rounding but is not itself a merge: the state may simply have landed elsewhere, with
/// nothing else landing on top of it.
///
/// So: take a state, perturb ONE component by one ULP — the smallest possible distinction
/// the representation can carry — step both forward `steps`, and ask whether they are now
/// the same state. Each collision destroys exactly one bit.
pub fn ulp_merge_rate<const N: usize>(
    st: &Structure<N>,
    params: &Params,
    symmetrised: bool,
    steps: usize,
    pairs: usize,
    seed: u64,
) -> Result<MergeReading, FloorError> {
    if !params.dt.is_finite() || params.dt == 0.0 {
        return Err(FloorError::DegenerateStep);
    }

    let mut merged = 0usize;
    let mut still_adjacent = 0usize;

    for member in 0..pairs {
        let a0 = draw_state::<N>(seed, member as u64);
        let mut b0 = a0;
        // Perturb one component, chosen deterministically from the member index so the
        // sweep covers every coordinate rather than always probing the same one.
        let which = (member * 7 + 3) % (N * 3);
        let (i, k) = (which / 3, which % 3);
        b0.pos[i][k] = next_ulp(b0.pos[i][k]);
        if b0.pos[i][k].to_bits() == a0.pos[i][k].to_bits() {
            continue; // perturbation did nothing (non-finite); skip rather than miscount
        }

        if merges(&a0, &b0, st, params, symmetrised, steps) {
            merged += 1;
        } else {
            let (mut a, mut b) = (a0, b0);
            for _ in 0..steps {
                step(&mut a, st, params, symmetrised);
                step(&mut b, st, params, symmetrised);
            }
            if abs(a.pos[i][k] - b.pos[i][k]) <= abs(next_ulp(a.pos[i][k]) - a.pos[i][k]) {
                still_adjacent += 1;
            }
        }
    }

    Ok(MergeReading {
        pairs,
        steps,
        merged,
        still_adjacent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::K11;

    fn undamped() -> Params {
        Params {
            damping: 1.0,
            ..Params::default()
        }
    }

    // ------------------------------------------------------------------ instrument controls
    //
    // Every instrument in this lane gets a control that must report the OPPOSITE outcome,
    // run before the instrument is trusted. A merge detector that never fires and one that
    // always fires are both useless, and both look like a clean result.

    /// **MUST FIRE.** Two identical states are one state; the detector must say so at every
    /// depth. If this fails, every zero below is a zero because the detector is broken.
    #[test]
    fn the_detector_fires_on_states_that_are_already_one() {
        let p = undamped();
        let a = draw_state::<{ crate::data::N }>(0xC0FFEE, 1);
        for &n in [0usize, 1, 8, 64].iter() {
            assert!(
                merges(&a, &a, &K11, &p, false, n),
                "detector failed to see a merge at n={n}; every merge count below is junk"
            );
        }
    }

    /// **MUST NOT FIRE.** Two clearly different states must not be reported as one. Guards
    /// the other direction: a detector that always fires would report a spectacular floor.
    #[test]
    fn the_detector_does_not_fire_on_states_that_are_far_apart() {
        let p = undamped();
        let a = draw_state::<{ crate::data::N }>(0xC0FFEE, 1);
        let b = draw_state::<{ crate::data::N }>(0xC0FFEE, 2);
        for &n in [0usize, 1, 8, 64].iter() {
            assert!(
                !merges(&a, &b, &K11, &p, false, n),
                "detector reported two distinct states as merged at n={n}"
            );
        }
    }

    /// At zero steps nothing has happened, so nothing can have merged. The plumbing control
    /// for the ensemble path rather than for the detector.
    #[test]
    fn nothing_merges_before_the_first_step() {
        let m = ulp_merge_rate(&K11, &undamped(), false, 0, 256, 0xC1_1250).unwrap();
        assert_eq!(m.merged, 0);
        assert_eq!(m.still_adjacent, m.pairs, "untouched pairs must still be adjacent");
    }

    // ------------------------------------------------------------------------- refusals

    /// A damped step is a contraction, not a reversible map. Refused rather than answered
    /// with a number that would look like a floor and be the damping.
    #[test]
    fn a_damped_step_is_refused_a_floor() {
        let p = Params {
            damping: 0.9,
            ..Params::default()
        };
        assert_eq!(
            float_production_floor(&K11, &p, false, 8, 4, 1, 1.0e-12),
            Err(FloorError::Damped)
        );
    }

    #[test]
    fn a_zero_step_is_refused() {
        let p = Params {
            dt: 0.0,
            damping: 1.0,
            ..Params::default()
        };
        assert_eq!(
            float_production_floor(&K11, &p, false, 8, 4, 1, 1.0e-12),
            Err(FloorError::DegenerateStep)
        );
        assert_eq!(
            ulp_merge_rate(&K11, &p, false, 8, 4, 1),
            Err(FloorError::DegenerateStep)
        );
    }

    // --------------------------------------------------------------------- the measurement

    /// **THE FLOOR.** Merges grow with `n` and then saturate.
    ///
    /// Measured on `K11` at `dt = 5e-3`, 256 ULP-adjacent pairs, undamped:
    ///
    /// ```text
    ///   n:      0    1    2    4    8   16   32   64  128  256  512 1024
    ///   merged: 0    0    0    2    3    6    7   11   17   19   19   19
    /// ```
    ///
    /// Saturating at **19/256 = 0.0514 nats (0.0742 bits) per ULP-adjacent pair**. The growth is the
    /// design requirement — a floor that did not grow with `n` would indict the instrument,
    /// not the arithmetic — and the saturation is physical: pairs that have not merged by
    /// `n ≈ 256` have diverged past ULP scale and can no longer collide.
    #[test]
    fn the_merge_rate_grows_with_n_and_then_saturates() {
        let p = undamped();
        let ns = [0usize, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];
        let mut merged = [0usize; 12];
        for (slot, &n) in ns.iter().enumerate() {
            merged[slot] = ulp_merge_rate(&K11, &p, false, n, 256, 0xC1_1250)
                .unwrap()
                .merged;
        }

        for slot in 1..ns.len() {
            assert!(
                merged[slot] >= merged[slot - 1],
                "merges fell from {} at n={} to {} at n={}; the floor must be monotone in n \
                 or the instrument is wrong",
                merged[slot - 1],
                ns[slot - 1],
                merged[slot],
                ns[slot]
            );
        }
        // The floor is quoted in NATS, and the accessor that quotes it is exercised here
        // rather than left as an unrun claim: both readings were dead code, so nothing
        // checked that the primary currency agreed with the count it is derived from.
        let saturated = ulp_merge_rate(&K11, &p, false, 1024, 256, 0xC1_1250).unwrap();
        let nats = saturated.nats_per_pair();
        assert!(
            (nats - 0.0514).abs() < 5.0e-4,
            "the floor is 0.0514 nats (0.0742 bits) per ULP-adjacent pair; read {nats:.6}"
        );
        assert!(
            (saturated.bits_per_pair() - nats / core::f64::consts::LN_2).abs() < 1.0e-15,
            "the two currencies disagree, which is the thing keeping one primary prevents"
        );

        assert_eq!(merged[0], 0, "n=0 must merge nothing");
        assert!(
            merged[3] > 0,
            "no merge had happened by n=4; the floor would be unmeasurable"
        );
        assert!(
            merged[9] >= 10,
            "only {} merges by n=256; the measured value was 19",
            merged[9]
        );
        assert_eq!(
            merged[11], merged[9],
            "the floor should have saturated by n=256; it moved between 256 and 1024"
        );
    }

    /// **THE FINDING, pinned so it is not re-proposed.** The originally specified quantity —
    /// step forward `n`, reverse `n`, count states that do not return — **saturates at 100%
    /// by n = 2** and therefore carries no information about `n` at all.
    ///
    /// It is not wrong, it is uninformative: with 66 independent `f64` components, the
    /// chance that every one returns bit-identically collapses almost immediately. The
    /// round-trip count is a yes/no that is always yes; the merge rate above is the graded
    /// quantity, and it is also the one `production_eq_zero_iff_rate_injective` is about,
    /// since a round trip that fails to return is not the same event as two states becoming
    /// one.
    #[test]
    fn the_roundtrip_count_saturates_by_two_steps_and_is_uninformative() {
        let p = undamped();
        for &n in [2usize, 8, 64, 1024].iter() {
            let r = float_production_floor(&K11, &p, false, n, 256, 0xC1_1250, 1.0e-12).unwrap();
            assert_eq!(
                r.not_returned_exact, r.ensemble,
                "round-trip count was expected to be saturated at n={n}"
            );
        }
    }

    /// What the round trip *does* carry is the drift, which grows cleanly — measured
    /// 5.55e-17 at n=1 to 6.40e-13 at n=1024, roughly `n^1.35`. Reported beside the count so
    /// the useful half of the original specification is not lost with the useless half.
    #[test]
    fn the_roundtrip_drift_grows_with_n() {
        let p = undamped();
        let small = float_production_floor(&K11, &p, false, 1, 256, 0xC1_1250, 1.0e-12).unwrap();
        let large = float_production_floor(&K11, &p, false, 1024, 256, 0xC1_1250, 1.0e-12).unwrap();
        assert!(small.worst_drift > 0.0, "n=1 produced no drift at all");
        assert!(
            large.worst_drift > 100.0 * small.worst_drift,
            "drift barely grew: {:e} at n=1 vs {:e} at n=1024",
            small.worst_drift,
            large.worst_drift
        );
        // And the epsilon count stays zero at 1e-12: information is destroyed long before
        // the trajectory is visibly wrong. That gap is the point of reporting both.
        assert_eq!(large.not_returned_eps, 0);
    }
}
