//! AUTOCORRELATION-AWARE UNCERTAINTY, and the equilibration rule that comes with it.
//!
//! # Why a campaign needs this and a seed spread is not it
//!
//! A molecular arm's readouts are a TIME SERIES, and consecutive samples of a liquid's
//! bond count or temperature are not independent draws. Dividing the sample standard
//! deviation by `sqrt(N)` over `N` correlated samples reports an error bar smaller than the
//! measurement by whatever factor the correlation time is — which is exactly the arithmetic
//! that makes a wrong number look precise.
//!
//! The correction is one number, the **statistical inefficiency** `g`: the number of
//! consecutive samples that carry one sample's worth of information.
//!
//! ```text
//! g = 1 + 2 Σ_{t=1..N-1} (1 − t/N) C(t)      truncated at the first t with C(t) ≤ 0
//! N_eff = N / g
//! SEM = sd / sqrt(N_eff)
//! ```
//!
//! `C(t)` is the normalised autocovariance of the series. The initial-positive truncation
//! is Geyer's; the `(1 − t/N)` weighting and the `g` name are Chodera's and, before him,
//! Friedberg–Cameron 1970, Swope–Andersen 1982 and Flyvbjerg–Petersen 1989. **The seed
//! spread and this are different quantities and a campaign reports BOTH**: the spread over
//! independent trajectories measures how much the answer depends on where the box started,
//! and `SEM` measures how much of one trajectory's own scatter is real sampling.
//!
//! # The equilibration rule, credited
//!
//! [`equilibration_start`] is **John D. Chodera, "A simple method for automated
//! equilibration detection in molecular simulations", *J. Chem. Theory Comput.* **12**
//! (2016) 1799–1805**, implemented here and not adapted: over every candidate discard
//! point `t0`, compute the statistical inefficiency of what remains and keep the `t0` that
//! **maximises the number of effectively uncorrelated samples** `N_eff(t0) = (N − t0)/g(t0)`.
//!
//! What makes it the right rule for this programme is what it does NOT need: no threshold,
//! no window length, no band, no reference plateau, and no look at whether the readout
//! agrees with experiment. It is an argmax over a quantity computed from the series alone,
//! so an equilibration length chosen this way cannot have been chosen for the answer it
//! gives — which is the failure LIQUID-1's 52 fs of settling and LIQUID-2's first settling
//! criterion were both exposed to.
//!
//! **This module holds no constant and no band.** Like the rest of the harness, every
//! number a campaign gates on stays the campaign's.

/// One series' worth of autocorrelation-aware uncertainty.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ineff {
    /// Samples in the series.
    pub n: usize,
    pub mean: f64,
    /// The sample standard deviation, `N − 1` in the denominator.
    pub sd: f64,
    /// The statistical inefficiency: consecutive samples per independent sample. `≥ 1`.
    pub g: f64,
    /// `n / g`.
    pub n_eff: f64,
    /// `sd / sqrt(n_eff)` — the standard error the correlation admits.
    pub sem: f64,
    /// The lag the autocorrelation sum was truncated at (the first non-positive `C(t)`,
    /// or `n` if the sum ran to the end).
    pub cut_lag: usize,
}

impl Ineff {
    /// The degenerate reading of a series with nothing in it to correlate.
    fn trivial(a: &[f64]) -> Ineff {
        let n = a.len();
        let mean = if n == 0 { f64::NAN } else { a.iter().sum::<f64>() / n as f64 };
        Ineff { n, mean, sd: 0.0, g: 1.0, n_eff: n as f64, sem: 0.0, cut_lag: 0 }
    }

    /// `mean ± k·sem` as a pair — the interval a record prints.
    pub fn interval(&self, k: f64) -> (f64, f64) {
        (self.mean - k * self.sem, self.mean + k * self.sem)
    }
}

/// The statistical inefficiency of a series, with the mean, the spread and the standard
/// error the correlation admits.
///
/// Conventions, stated because they are the ones that differ between implementations:
/// the autocovariance is computed about the series' OWN mean; the normalisation is
/// `C(0) = 1`; the sum carries the `(1 − t/N)` bias weight; and it is truncated at the
/// first lag whose `C(t)` is not positive (Geyer's initial-positive rule), which is what
/// stops the estimator from accumulating the noise of the long-lag tail.
///
/// A series shorter than three samples, or one with no variance at all, reads `g = 1` —
/// the honest floor, and the same convention Chodera's own implementation takes.
pub fn inefficiency(a: &[f64]) -> Ineff {
    let n = a.len();
    if n < 3 {
        return Ineff::trivial(a);
    }
    let nf = n as f64;
    let mean = a.iter().sum::<f64>() / nf;
    let c0 = a.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / nf;
    if !(c0 > 0.0) {
        return Ineff::trivial(a);
    }
    let mut tau = 0.0f64;
    let mut cut = n;
    for t in 1..n {
        let mut acc = 0.0f64;
        for k in 0..(n - t) {
            acc += (a[k] - mean) * (a[k + t] - mean);
        }
        let c = acc / ((n - t) as f64) / c0;
        if c <= 0.0 {
            cut = t;
            break;
        }
        tau += (1.0 - t as f64 / nf) * c;
    }
    let g = (1.0 + 2.0 * tau).max(1.0);
    let sd = (a.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (nf - 1.0)).sqrt();
    let n_eff = (nf / g).max(1.0);
    Ineff { n, mean, sd, g, n_eff, sem: sd / n_eff.sqrt(), cut_lag: cut }
}

/// Where a series stops being a transient, by Chodera 2016's rule.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Equilibration {
    /// The chosen discard point, as an index into the series.
    pub t0: usize,
    /// The effective sample count of `a[t0..]` — the quantity that was maximised.
    pub n_eff: f64,
    /// The statistical inefficiency of `a[t0..]`.
    pub g: f64,
    /// The whole series' length.
    pub n: usize,
    /// How many candidate discard points were tried.
    pub scanned: usize,
    /// The effective sample count with NOTHING discarded, so the record shows what the
    /// rule bought. `n_eff / n_eff_at_zero` is the reason to believe the discard.
    pub n_eff_at_zero: f64,
}

/// **Chodera 2016**: the discard point that maximises the number of effectively
/// uncorrelated samples in what remains.
///
/// The scan runs over every candidate `t0` that leaves at least three samples, so the rule
/// has no window parameter of its own. It is `O(N²)` in the series length, which is why a
/// campaign runs it on BLOCK means (tens to hundreds of points) and not on raw frames.
///
/// Returns `t0 = 0` on a series too short to scan — an honest "nothing to discard" rather
/// than a refusal, because the caller's own floor is the thing that should refuse.
pub fn equilibration_start(a: &[f64]) -> Equilibration {
    let n = a.len();
    let zero = inefficiency(a);
    if n < 6 {
        return Equilibration {
            t0: 0,
            n_eff: zero.n_eff,
            g: zero.g,
            n,
            scanned: usize::from(n >= 3),
            n_eff_at_zero: zero.n_eff,
        };
    }
    let (mut best_t0, mut best_neff, mut best_g, mut scanned) = (0usize, -1.0f64, 1.0f64, 0usize);
    for t0 in 0..(n - 3) {
        let r = inefficiency(&a[t0..]);
        scanned += 1;
        if r.n_eff > best_neff {
            best_neff = r.n_eff;
            best_t0 = t0;
            best_g = r.g;
        }
    }
    Equilibration {
        t0: best_t0,
        n_eff: best_neff,
        g: best_g,
        n,
        scanned,
        n_eff_at_zero: zero.n_eff,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fixtures' generator: stated seed, nothing else.
    struct Lcg(u64);
    impl Lcg {
        fn unit(&mut self) -> f64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((self.0 >> 11) as f64) / ((1u64 << 53) as f64)
        }
        fn gauss(&mut self) -> f64 {
            let u = self.unit().max(1e-12);
            let v = self.unit();
            (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
        }
    }

    #[test]
    fn white_noise_is_its_own_sample_count() {
        let mut r = Lcg(0x5EED_0001);
        let a: Vec<f64> = (0..4000).map(|_| r.gauss()).collect();
        let s = inefficiency(&a);
        assert!((s.g - 1.0).abs() < 0.35, "white noise must read g near 1, read {}", s.g);
        assert!(s.n_eff > 2500.0, "n_eff {}", s.n_eff);
    }

    /// An AR(1) process has an EXACT answer: `C(t) = phi^t`, so
    /// `g = 1 + 2 phi/(1 − phi)= (1 + phi)/(1 − phi)`. This is the test that the estimator
    /// measures correlation and not scatter.
    #[test]
    fn an_ar1_process_reads_its_analytic_inefficiency() {
        for (phi, want) in [(0.5f64, 3.0f64), (0.8, 9.0)] {
            let mut r = Lcg(0x5EED_0002);
            let mut x = 0.0f64;
            let mut a = Vec::with_capacity(20_000);
            for _ in 0..20_000 {
                x = phi * x + (1.0 - phi * phi).sqrt() * r.gauss();
                a.push(x);
            }
            let s = inefficiency(&a);
            assert!(
                (s.g - want).abs() / want < 0.25,
                "AR(1) phi={phi} must read g near {want}, read {}",
                s.g
            );
            assert!(s.sem > s.sd / (a.len() as f64).sqrt(), "the correction must WIDEN the bar");
        }
    }

    #[test]
    fn a_constant_series_has_no_error_bar_and_no_correlation() {
        let s = inefficiency(&[2.5; 100]);
        assert_eq!(s.g, 1.0);
        assert_eq!(s.sem, 0.0);
        assert_eq!(s.mean, 2.5);
    }

    /// The equilibration rule on a series with a PLANTED transient: an exponential decay
    /// of length `tau` onto a noisy plateau. The rule must discard the transient and must
    /// not discard much more than it.
    #[test]
    fn chodera_finds_a_planted_transient() {
        let mut r = Lcg(0x5EED_0003);
        let tau = 12.0f64;
        let a: Vec<f64> = (0..400)
            .map(|t| 1.0 + 5.0 * (-(t as f64) / tau).exp() + 0.05 * r.gauss())
            .collect();
        let e = equilibration_start(&a);
        assert!(
            (24..=140).contains(&e.t0),
            "the rule must discard the transient (a few tau) and not the arm: t0 = {}",
            e.t0
        );
        assert!(e.n_eff > e.n_eff_at_zero, "discarding must BUY effective samples");
    }

    /// And on a series with NO transient it must not invent one.
    #[test]
    fn chodera_discards_almost_nothing_from_a_flat_series() {
        let mut r = Lcg(0x5EED_0004);
        let a: Vec<f64> = (0..400).map(|_| 1.0 + 0.05 * r.gauss()).collect();
        let e = equilibration_start(&a);
        assert!(e.t0 < 40, "a flat series must keep almost all of itself: t0 = {}", e.t0);
    }
}
