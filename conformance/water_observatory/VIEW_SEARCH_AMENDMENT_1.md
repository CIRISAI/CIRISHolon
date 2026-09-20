# VIEW-SEARCH-1 — AMENDMENT 1: what building the reader found, and S4 re-staked as a separation

*Written 2026-09-20 on building the reader (`view_search.py`) and AFTER its first pass over
the walks — so this is not an amendment before data; it is a correction with the wrong text
kept (`VIEW_SEARCH_PREREG.md` unchanged) and the read reported under both the stakes as
written and the stakes as corrected here, labelled. Six things, in order of weight.*

1. **S4's premise was wrong.** A position-blind relabelling of the cells is NOT a partition
   with no closed structure: each blind "cell momentum" is the velocity sum of a fixed random
   subset of molecules, and single-particle velocity memory (the VACF at 100 fs) is a real
   closed direction that has nothing to do with space. The placebo scored `0.28–0.42` of the
   bound, PV-2 (time-shuffled) `0.04`. The reader is not convicted — it found what is there.
   **S4 re-staked as the separation the fluid instrument already uses:** the spatial `2×2×2`
   view's held-out fraction exceeds the blind view's by `≥ 0.05` at `τ = 100` fs; kill if not.
   The 0.1-of-bound clause is kept for the TIME-shuffled placebo (PV-2), where it belongs.
2. **S2 cannot be read from the in-sample singular values.** With `d = 55` features and
   `~600` training frames the whitened Koopman's top `σ` are biased toward `1` (`0.996`, a
   "20 ps" timescale, where the same mode's autocorrelation at 100 fs is `0.89`); a ridge of
   `10⁻³` does not remove it. **S2 is read from the mode's own autocorrelation** — an
   exponential fit to the transverse-current autocorrelation over the lags where it exceeds
   `0.2`, `Γ_s` and `η = ρΓ_s/k²` per seed — the plain Green–Kubo-type read; the VAMP
   timescale is printed and not graded.
3. **The held-out score is Wu & Noé's validation score** (the train subspaces evaluated with
   the test data's own covariances, bounded by `k`), not train whitening applied to test
   covariances, which read `3–5 × k` whenever a test seed's variance in a low-variance train
   direction was larger.
4. **Features are standardised** before whitening (occupancies vary by `~1`, momenta by
   `~10⁻⁴` au; a trace-relative ridge otherwise swallowed every momentum direction, PV-3).
5. **Per-seed centring.** Each seed's density modes carry a static offset of `1–2` counts over
   its 5–6 ps against a within-seed sd of `1.4–2.7` — a frozen long-wavelength density
   pattern (this model does not diffuse on that time; LIQUID-1 refused `D`). Centring by seed
   reads the dynamics; the offsets are printed as what was removed.
6. **PV-1 at `N = 3,000` had a sampling error over its own tolerance** (`σ` to `0.02`, `t₁` to
   `10 %`); the plant runs at `N = 30,000`. PV-3's `10⁻⁹` assumed no ridge; it is `10⁻⁴`, the
   ridge's own size.
