//! Finite-horizon locality bound for certified holon refinement.
//!
//! The instantaneous boundary state is not, in general, a Markov state for a coarse
//! holon.  Two fine states can agree on gross state and current boundary values while a
//! hidden interior mode reaches the boundary later.  The repair is to certify an
//! *interaction horizon*, not to pretend the missing mode does not exist.
//!
//! For a linear local generator `A`, an observable `P` and latent-state injection `Q`,
//! graph locality makes the first `d` terms of `P exp(tA) Q` vanish when the source and
//! readout are `d` generator hops apart.  With any subordinate norm,
//!
//! ```text
//! ||P exp(tA) Q||
//!   <= ||P|| ||Q|| sum_{n=d}^inf (||A|| t)^n / n!
//!   <= ||P|| ||Q|| exp(z) z^d / d!,  z = ||A|| t.
//! ```
//!
//! The second inequality is deliberately conservative and cheap.  A realization may
//! use a sharper Lieb-Robinson/Mori-Zwanzig bound, but it may not use a weaker claim
//! without naming it.  The quantity below is therefore a sufficient macro-error term
//! to ADD to a [`crate::holon::BoundaryModel`] certificate whenever latent modes are
//! omitted over a finite horizon.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HorizonLocality {
    /// Upper bound on the chosen norm of the local first-order generator.
    pub generator_norm_per_s: f64,
    /// Time interval for which the coarse prediction is being certified.
    pub horizon_s: f64,
    /// Minimum number of generator applications needed for the latent source to reach
    /// the observable.  This is distance in the generator graph, not necessarily the
    /// material graph: a second-order oscillator written as `(x,v)` spends hops moving
    /// between position and velocity components.
    pub min_generator_hops: u32,
    /// Norm bound on the omitted latent-state difference allowed by the descriptor.
    pub latent_state_norm: f64,
    /// Operator norm of the requested readout.  `1` for a normalized scalar readout.
    pub observable_norm: f64,
}

impl HorizonLocality {
    /// Conservative influence bound for the declared horizon.  Invalid declarations
    /// return infinity so they can never accidentally certify a frontier.
    pub fn influence_bound(&self) -> f64 {
        if !self.generator_norm_per_s.is_finite()
            || !self.horizon_s.is_finite()
            || !self.latent_state_norm.is_finite()
            || !self.observable_norm.is_finite()
            || self.generator_norm_per_s < 0.0
            || self.horizon_s < 0.0
            || self.latent_state_norm < 0.0
            || self.observable_norm < 0.0
        {
            return f64::INFINITY;
        }
        if self.latent_state_norm == 0.0 || self.observable_norm == 0.0 {
            return 0.0;
        }
        let z = self.generator_norm_per_s * self.horizon_s;
        self.observable_norm
            * self.latent_state_norm
            * exponential_tail_bound(z, self.min_generator_hops)
    }

    /// Smallest source/readout distance whose conservative latent influence is no more
    /// than `tolerance`.  A frontier may keep omitted state aggregated only if its
    /// nearest latent source is at least this many generator hops from the readout.
    ///
    /// Returns `None` for an invalid declaration, negative/NaN tolerance, or when the
    /// requested tolerance is not reached within `max_hops`.  The bounded search keeps
    /// this helper deterministic and usable in `no_std` certification code.
    pub fn required_hops_for_tolerance(&self, tolerance: f64, max_hops: u32) -> Option<u32> {
        if !tolerance.is_finite() || tolerance < 0.0 {
            return None;
        }
        let mut probe = *self;
        probe.min_generator_hops = 0;
        if !probe.influence_bound().is_finite() {
            return None;
        }
        let mut hops = 0u32;
        while hops <= max_hops {
            probe.min_generator_hops = hops;
            if probe.influence_bound() <= tolerance {
                return Some(hops);
            }
            hops += 1;
        }
        None
    }
}

/// `exp(z) z^d / d!`, an upper bound on the exponential-series tail beginning at
/// degree `d` for `z >= 0`.  Computed by recurrence to avoid integer factorial overflow.
pub fn exponential_tail_bound(z: f64, degree: u32) -> f64 {
    if !z.is_finite() || z < 0.0 {
        return f64::INFINITY;
    }
    if z == 0.0 {
        return if degree == 0 { 1.0 } else { 0.0 };
    }
    let mut leading = 1.0f64;
    let mut n = 1u32;
    while n <= degree {
        leading *= z / n as f64;
        n += 1;
    }
    libm::exp(z) * leading
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_horizon_blocks_every_nonlocal_latent_mode() {
        assert_eq!(exponential_tail_bound(0.0, 1), 0.0);
        assert_eq!(exponential_tail_bound(0.0, 9), 0.0);
        assert_eq!(exponential_tail_bound(0.0, 0), 1.0);
    }

    #[test]
    fn distance_tightens_the_short_horizon_bound() {
        let z = 0.4;
        let near = exponential_tail_bound(z, 1);
        let middle = exponential_tail_bound(z, 3);
        let far = exponential_tail_bound(z, 5);
        assert!(near > middle && middle > far, "{near} {middle} {far}");
    }

    #[test]
    fn invalid_declaration_refuses_to_certify() {
        let bad = HorizonLocality {
            generator_norm_per_s: -1.0,
            horizon_s: 1.0,
            min_generator_hops: 3,
            latent_state_norm: 1.0,
            observable_norm: 1.0,
        };
        assert!(bad.influence_bound().is_infinite());
        assert_eq!(bad.required_hops_for_tolerance(1.0e-3, 20), None);
    }

    #[test]
    fn short_horizon_produces_a_small_finite_refinement_radius() {
        // A unit oscillator chain written first-order has ||A||_inf <= 4.  Over one
        // 600 Hz game substep z=1/150: latent modes only a few generator hops away are
        // already below a 1e-6 normalized-observable tolerance.
        let cert = HorizonLocality {
            generator_norm_per_s: 4.0,
            horizon_s: 1.0 / 600.0,
            min_generator_hops: 0,
            latent_state_norm: 1.0,
            observable_norm: 1.0,
        };
        let required = cert.required_hops_for_tolerance(1.0e-6, 20).unwrap();
        assert!(required <= 4, "required {required} hops");
        let mut at_required = cert;
        at_required.min_generator_hops = required;
        assert!(at_required.influence_bound() <= 1.0e-6);
        if required > 0 {
            at_required.min_generator_hops = required - 1;
            assert!(at_required.influence_bound() > 1.0e-6);
        }
    }

    /// The counterexample that forced this module: a six-mass unit spring chain has two
    /// initial states with identical coarse momentum and identical current boundary
    /// position/velocity, yet an internal `(+1,-1)` velocity mode later moves the
    /// boundary.  The horizon term is nonzero, so a certificate can no longer call
    /// those states equivalent merely because the boundary agrees at t=0.
    #[test]
    fn hidden_zero_gross_mode_has_nonzero_future_influence_budget() {
        const N: usize = 6;
        let mut x = [0.0f64; N];
        let mut v = [0.0f64; N];
        v[0] = 1.0;
        v[1] = -1.0; // zero gross momentum in the left coarse holon

        let force = |x: &[f64; N]| {
            let mut f = [0.0f64; N];
            for i in 0..(N - 1) {
                let d = x[i + 1] - x[i];
                f[i] += d;
                f[i + 1] -= d;
            }
            f
        };

        let dt = 1.0e-3;
        for _ in 0..500 {
            let a0 = force(&x);
            for i in 0..N {
                v[i] += 0.5 * dt * a0[i];
                x[i] += dt * v[i];
            }
            let a1 = force(&x);
            for i in 0..N {
                v[i] += 0.5 * dt * a1[i];
            }
        }

        // Node 2 was initially at rest and is the interaction-side boundary of the
        // left three-node coarse holon.  It has moved because the hidden mode arrived.
        assert!(x[2].abs() > 1.0e-4, "hidden mode did not reach boundary: {}", x[2]);

        // Infinity norm of the first-order chain generator is <= 4.  The nearest
        // omitted velocity component (v1) is three generator hops from x2.
        let cert = HorizonLocality {
            generator_norm_per_s: 4.0,
            horizon_s: 0.5,
            min_generator_hops: 3,
            latent_state_norm: 2.0f64.sqrt(),
            observable_norm: 1.0,
        };
        let bound = cert.influence_bound();
        assert!(bound > x[2].abs(), "bound {bound} missed actual {}", x[2].abs());
    }
}
