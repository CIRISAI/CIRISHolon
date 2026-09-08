//! THE THERMOSTAT, AS A CHOICE: Berendsen, or stochastic velocity rescaling.
//!
//! # What the second review said, and it is right
//!
//! > *the thermostat is Berendsen velocity rescaling — suppressed fluctuations are not
//! > canonical sampling; a stochastic-rescaling thermostat (Bussi–Donadio–Parrinello 2007)
//! > is the named replacement.*
//!
//! Berendsen velocity rescaling (Berendsen, Postma, van Gunsteren, DiNola and Haak,
//! *J. Chem. Phys.* **81** (1984) 3684) drives the instantaneous temperature towards its
//! target by a deterministic factor. It reaches the right MEAN kinetic energy and it does
//! not sample the canonical ensemble: it has no stochastic term at all, so the kinetic
//! energy's own fluctuation is damped by the same time constant that damps its error, and
//! the ensemble it produces has a kinetic-energy variance smaller than the canonical one by
//! a factor set by `Δt/τ`. Every number a run reports that is a FLUCTUATION — a heat
//! capacity, a compressibility, any variance-based response — is then wrong by that factor,
//! and the ones that are means are only right because the mean is what it was tuned to.
//!
//! **Stochastic velocity rescaling** (Bussi, Donadio and Parrinello, *J. Chem. Phys.*
//! **126** (2007) 014101) is Berendsen's rescaling with the missing noise put back, chosen
//! so that the canonical distribution of the kinetic energy is the exact stationary
//! distribution of the update. It costs one Gaussian and one chi-squared deviate per step
//! and it is otherwise the same operation on the same velocities — which is why it can wear
//! the same ledger column with no change to what that column MEANS.
//!
//! # The canonical fact this is tested against
//!
//! For `N_f` momentum degrees of freedom the canonical kinetic energy is
//! `K ~ Gamma(N_f/2, k_B T)`, hence
//!
//! ```text
//! <K> = (N_f/2) k_B T        Var(K)/<K>^2 = 2/N_f
//! ```
//!
//! In three dimensions with `N` atoms and no constraints `N_f = 3N`, so the relative
//! variance is `2/(3N)`. `holon-render/tests/thermostat.rs` measures it for both
//! thermostats on a harmonic test system and prints both.
//!
//! # What does not change
//!
//! The ledger. Both thermostats rescale every velocity by one factor `λ`, so both post the
//! kinetic energy they moved to `w_ext` and `ExternalWork::thermostat`, and both post the
//! momentum the rescaling moved to `j_ext` — the same code, in `Sim::apply_thermostat`, on
//! whichever `λ` was chosen. The receipt is exact either way and says nothing different.
//!
//! **Berendsen stays the default and every existing record still selects it.** The
//! stochastic branch is the only place this module's generator is touched, so a Berendsen
//! run draws nothing and is bit-for-bit what it always was; `Sim::thermostat_draws` is `0`
//! after one and is the check.
//!
//! # The fence on `N_f`, stated rather than left to be discovered
//!
//! `Sim::apply_thermostat` hands the stochastic rule `N_f = dims.dof() · n` — the SAME
//! degree-of-freedom count `Sim::temperature` divides by, so the target the thermostat aims
//! at and the temperature every gate reads are one quantity and cannot drift apart. That
//! count does NOT subtract the three centre-of-mass degrees of freedom that a wall-less box
//! (`Boundary::Open`, `Boundary::Periodic`) conserves. It is the engine's existing
//! convention and it is not changed here — changing it would move every temperature this
//! tree has ever recorded — but it means that in a wall-less box the canonical relative
//! variance the rule reproduces is `2/(dims·n)` and the physically exact one is
//! `2/(dims·n − 3)`, a difference of `0.26 %` at 384 atoms. Anyone who needs the
//! fluctuation to that precision must fix the convention in `Sim::temperature` first, and
//! this note is here so that is a decision and not a surprise.

/// Which rule sets the rescaling factor.
///
/// `Default` is [`ThermostatKind::Berendsen`], deliberately: every record in this tree was
/// taken under it, and a default that changed would silently reinterpret them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThermostatKind {
    /// Berendsen velocity rescaling (Berendsen et al. 1984). Deterministic. Reaches the
    /// target mean; does NOT sample the canonical ensemble.
    #[default]
    Berendsen,
    /// Stochastic velocity rescaling (Bussi, Donadio and Parrinello 2007). Canonical: the
    /// kinetic energy's equilibrium distribution is the exact canonical one.
    StochasticRescaling,
}

impl ThermostatKind {
    pub fn name(self) -> &'static str {
        match self {
            ThermostatKind::Berendsen => "berendsen",
            ThermostatKind::StochasticRescaling => "stochastic-rescaling",
        }
    }
    /// The paper the rule is, so a record cites it without the runner having to.
    pub fn credit(self) -> &'static str {
        match self {
            ThermostatKind::Berendsen => {
                "Berendsen, Postma, van Gunsteren, DiNola and Haak, J. Chem. Phys. 81 (1984) 3684"
            }
            ThermostatKind::StochasticRescaling => {
                "Bussi, Donadio and Parrinello, J. Chem. Phys. 126 (2007) 014101"
            }
        }
    }
    /// Does this rule sample the canonical ensemble?
    pub fn is_canonical(self) -> bool {
        matches!(self, ThermostatKind::StochasticRescaling)
    }
}

/// The thermostat's own generator: splitmix64, so a run is a function of its stated seed
/// and of nothing about the host.
///
/// It is SEPARATE from anything else in the engine on purpose. A generator shared with the
/// scene builder would make a thermostatted trajectory depend on how many atoms were
/// placed, and the replay fingerprints would stop meaning what they say.
#[derive(Clone, Copy, Debug)]
pub struct ThermostatRng {
    state: u64,
    /// Every deviate this generator has produced. `0` on a run that never asked for one,
    /// which is how a Berendsen run proves it drew nothing.
    pub draws: u64,
    /// The second Box–Muller deviate of a pair, held rather than thrown away.
    spare: Option<f64>,
}

impl ThermostatRng {
    pub const fn new(seed: u64) -> Self {
        ThermostatRng { state: seed ^ 0x9E37_79B9_7F4A_7C15, draws: 0, spare: None }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform on `(0, 1)`. Never exactly zero, so a logarithm of it is finite.
    #[inline]
    pub fn unit(&mut self) -> f64 {
        let x = ((self.next_u64() >> 11) as f64 + 0.5) / ((1u64 << 53) as f64);
        x
    }

    /// A standard normal deviate, Box–Muller with the pair kept.
    pub fn gauss(&mut self) -> f64 {
        self.draws += 1;
        if let Some(s) = self.spare.take() {
            return s;
        }
        let u = self.unit();
        let v = self.unit();
        let r = (-2.0 * u.ln()).sqrt();
        let (s, c) = (std::f64::consts::TAU * v).sin_cos();
        self.spare = Some(r * s);
        r * c
    }

    /// A Gamma(`shape`, 1) deviate, Marsaglia and Tsang, *ACM TOMS* **26** (2000) 363.
    ///
    /// `O(1)` in the shape, which is what makes the chi-squared below affordable: the
    /// alternative — summing `N_f − 1` squared normals — is `O(N_f)` per substep and would
    /// put the thermostat's cost on the same curve as the force pass's.
    pub fn gamma(&mut self, shape: f64) -> f64 {
        if shape <= 0.0 {
            return 0.0;
        }
        if shape < 1.0 {
            let g = self.gamma(shape + 1.0);
            let u = self.unit();
            return g * u.powf(1.0 / shape);
        }
        let d = shape - 1.0 / 3.0;
        let c = 1.0 / (9.0 * d).sqrt();
        loop {
            let x = self.gauss();
            let t = 1.0 + c * x;
            if t <= 0.0 {
                continue;
            }
            let v = t * t * t;
            let u = self.unit();
            let x2 = x * x;
            if u < 1.0 - 0.0331 * x2 * x2 {
                return d * v;
            }
            if u.ln() < 0.5 * x2 + d * (1.0 - v + v.ln()) {
                return d * v;
            }
        }
    }

    /// A chi-squared deviate on `k` degrees of freedom: `chi2(k) = 2 · Gamma(k/2, 1)`.
    pub fn chi2(&mut self, k: usize) -> f64 {
        if k == 0 {
            return 0.0;
        }
        2.0 * self.gamma(k as f64 / 2.0)
    }
}

/// BERENDSEN's rescaling factor, exactly as this engine has always computed it.
///
/// `λ² = 1 + (Δt/τ)(T_target/T_now − 1)`. Returns `None` when there is nothing to rescale
/// (no kinetic energy) or when `λ²` has gone non-positive, which is the case the engine
/// has always declined rather than take a square root of.
pub fn berendsen_lambda(t_now: f64, t_target: f64, dt_over_tau: f64) -> Option<f64> {
    if t_now <= 0.0 {
        return None;
    }
    let lambda_sq = 1.0 + dt_over_tau * (t_target / t_now - 1.0);
    if lambda_sq <= 0.0 {
        return None;
    }
    Some(lambda_sq.sqrt())
}

/// BUSSI–DONADIO–PARRINELLO's new kinetic energy, their equation (A7):
///
/// ```text
/// K' = K + (1−c)( K̄ (R₁² + S)/N_f − K ) + 2 R₁ sqrt( K K̄ (1−c) c / N_f )
/// ```
///
/// with `c = exp(−Δt/τ)`, `R₁` a standard normal, `S` a chi-squared on `N_f − 1` degrees of
/// freedom, and `K̄` the target kinetic energy. The rescaling factor is `sqrt(K'/K)`.
///
/// The three limits are the ones to check and they are checked in this module's tests:
/// at `Δt/τ → 0` nothing moves; at `Δt/τ → ∞` the new kinetic energy is a pure canonical
/// draw `K̄ · chi²_{N_f} / N_f` with no memory of `K`; and at every `Δt/τ` between, the
/// canonical distribution is stationary.
pub fn bdp_new_kinetic(
    k_now: f64,
    k_target: f64,
    n_dof: usize,
    dt_over_tau: f64,
    rng: &mut ThermostatRng,
) -> f64 {
    if n_dof == 0 || k_now <= 0.0 || k_target <= 0.0 {
        return k_now;
    }
    let nf = n_dof as f64;
    let c = (-dt_over_tau).exp();
    let r1 = rng.gauss();
    let s = rng.chi2(n_dof - 1);
    let k_new = k_now
        + (1.0 - c) * (k_target * (r1 * r1 + s) / nf - k_now)
        + 2.0 * r1 * (k_now * k_target / nf * (1.0 - c) * c).sqrt();
    // The update is positive by construction in exact arithmetic; the floor is against
    // roundoff at a kinetic energy of essentially zero, and it is a floor and not a clamp
    // to the target, so a thermostat that has nothing to work with still does nothing.
    if k_new.is_finite() && k_new > 0.0 {
        k_new
    } else {
        k_now
    }
}

/// The stochastic-rescaling factor. `None` on a scene with no kinetic energy to rescale,
/// matching [`berendsen_lambda`] so the caller's ledger path is one path.
pub fn bdp_lambda(
    k_now: f64,
    k_target: f64,
    n_dof: usize,
    dt_over_tau: f64,
    rng: &mut ThermostatRng,
) -> Option<f64> {
    if k_now <= 0.0 {
        return None;
    }
    Some((bdp_new_kinetic(k_now, k_target, n_dof, dt_over_tau, rng) / k_now).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mean_var(a: &[f64]) -> (f64, f64) {
        let n = a.len() as f64;
        let m = a.iter().sum::<f64>() / n;
        (m, a.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0))
    }

    #[test]
    fn the_generator_produces_standard_normals() {
        let mut r = ThermostatRng::new(0xC0FFEE);
        let a: Vec<f64> = (0..200_000).map(|_| r.gauss()).collect();
        let (m, v) = mean_var(&a);
        assert!(m.abs() < 0.01, "mean {m}");
        assert!((v - 1.0).abs() < 0.02, "variance {v}");
        assert_eq!(r.draws, 200_000);
    }

    /// `chi2(k)` has mean `k` and variance `2k`. This is the deviate the whole BDP update
    /// rests on, so it is checked against its analytic moments and not against a picture.
    #[test]
    fn the_chi_squared_deviate_has_its_analytic_moments() {
        for k in [1usize, 2, 7, 64, 1151] {
            let mut r = ThermostatRng::new(0x5EED ^ k as u64);
            let a: Vec<f64> = (0..60_000).map(|_| r.chi2(k)).collect();
            let (m, v) = mean_var(&a);
            assert!((m - k as f64).abs() / (k as f64) < 0.02, "k={k} mean {m}");
            assert!((v - 2.0 * (k as f64)).abs() / (2.0 * (k as f64)) < 0.06, "k={k} var {v}");
        }
    }

    /// THE STATIONARITY TEST, and it is the whole claim: iterate the kinetic-energy update
    /// alone and the distribution it settles on is the canonical one,
    /// `<K> = (N_f/2) k_B T` and `Var(K)/<K>² = 2/N_f`.
    #[test]
    fn the_stochastic_update_is_canonical_in_the_kinetic_energy() {
        let n_dof = 96usize;
        let k_target = 0.5 * n_dof as f64; // k_B T = 1 in the units of this test
        let mut rng = ThermostatRng::new(0xBD9_2007);
        let mut k = k_target;
        let mut sample = Vec::with_capacity(400_000);
        for step in 0..400_000 {
            k = bdp_new_kinetic(k, k_target, n_dof, 0.1, &mut rng);
            if step > 2_000 {
                sample.push(k);
            }
        }
        let (m, v) = mean_var(&sample);
        assert!((m - k_target).abs() / k_target < 0.01, "mean {m} against {k_target}");
        let rel = v / (m * m);
        let want = 2.0 / n_dof as f64;
        assert!(
            (rel - want).abs() / want < 0.10,
            "relative variance {rel:.6} against the canonical {want:.6}"
        );
    }

    /// The two limits, so the coupling constant is known to mean what it says.
    #[test]
    fn the_coupling_limits_are_no_move_and_a_fresh_draw() {
        let n_dof = 60usize;
        let k_target = 0.5 * n_dof as f64;
        let mut rng = ThermostatRng::new(7);
        // dt/tau = 0: c = 1, the update is the identity whatever the deviates are
        let k = 3.0 * k_target;
        assert!((bdp_new_kinetic(k, k_target, n_dof, 0.0, &mut rng) - k).abs() < 1e-9);
        // dt/tau -> infinity: c = 0, K' = k_target * chi2(N_f) / N_f, with no memory of K
        let far: Vec<f64> =
            (0..40_000).map(|_| bdp_new_kinetic(k, k_target, n_dof, 60.0, &mut rng)).collect();
        let (m, v) = mean_var(&far);
        assert!((m - k_target).abs() / k_target < 0.02, "mean {m}");
        assert!((v / (m * m) - 2.0 / n_dof as f64).abs() / (2.0 / n_dof as f64) < 0.10);
    }

    /// BERENDSEN, on the same test, is measurably NOT canonical — its kinetic energy has no
    /// stationary fluctuation at all, because its update has no noise in it.
    #[test]
    fn berendsen_has_no_fluctuation_of_its_own() {
        let (t_target, dt_over_tau) = (300.0f64, 0.1f64);
        let mut t = 900.0f64;
        for _ in 0..2_000 {
            let l = berendsen_lambda(t, t_target, dt_over_tau).unwrap();
            t *= l * l;
        }
        assert!((t - t_target).abs() < 1e-6, "Berendsen must reach the target exactly: {t}");
        // and it STAYS there: the next hundred steps move it by nothing at all
        for _ in 0..100 {
            t *= berendsen_lambda(t, t_target, dt_over_tau).unwrap().powi(2);
        }
        assert!((t - t_target).abs() < 1e-9, "and never leave it: {t}");
    }

    #[test]
    fn a_thermostat_kind_carries_its_credit_and_its_honest_flag() {
        assert!(!ThermostatKind::default().is_canonical());
        assert_eq!(ThermostatKind::default(), ThermostatKind::Berendsen);
        assert!(ThermostatKind::StochasticRescaling.is_canonical());
        assert!(ThermostatKind::StochasticRescaling.credit().contains("2007"));
    }
}
