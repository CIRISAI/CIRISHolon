# ANOMALY-1 — adversarial review of the draft, 2026-11-23: REDESIGN rung I; rung 0 may freeze as a gate after named fixes

*An independent reader, given `ANOMALY1_PREREG_DRAFT.md` at `7ef029e`, told to break it, and
held to the draft's reserved region (declined checks are listed at the end). It wrote nothing
in the repository; this is its report as the lead's agent recorded it, shortened. Its
numerical checks are at `x ≤ 1` or `x ≥ 40`, `N ≤ 10`, outside the reserve.*

## 1. The three findings, in order

1. **It is still not an anomaly reading, and its "forced number" is the published staggered
   mass shift.** An exact identity, verified as matrices to `4 × 10⁻¹⁵` (`N = 8`, `x = 0.7`):
   `W(μ=0, α) = x ΣK + Σ(S_n + α + ¼)² + (1/8) Σ(−1)ⁿ σᶻ_n + const`. The "massless" chain
   carries a hidden mass, `m_eff/g = 1/(8√x)`: Dempsey, Klebanov, Pufu and Zan (arXiv
   2206.05308), `m_lat = m − e²a/8`. So the unscreened fraction `w₀` is the bulk field slope
   of the MASSIVE model at that mass. It is a bulk property (`0.799, 0.800, 0.801` at
   `N = 6, 8, 10`, `x = 0.3`) and it vanishes like `1/√x` in the continuum limit. Angelides,
   Funcke, Jansen and Kühn (arXiv 2211.12169, 2303.11016) locate the shift from the field's
   dependence on theta, so a nonzero `w₀` at zero lattice mass is in print. And the stake is
   a single-pole linear-response statement: at `x = 0.3`, where the hop is a perturbation,
   the saturation defect reads `0.0007` to `0.0016`; at `x ≤ 1`, at most `0.028`; at
   `x = 40, 100`, at most `0.077`. A chain with no anomaly physics meets S1. What is
   specific to the anomaly (complete screening; a coupling-independent coefficient) is what
   the draft concedes fails on this lattice, and nothing staked reads it.
2. **The referee is biased, and bars sit on table cells.** The banked `M_∞` is a linear fit
   in `1/N` (`instrument/dmrg_schwinger.py`, lines 296–301 as read); the banked box gaps
   follow `1/N²`, as an open-box level must, with residuals about ten times smaller. The
   box law gives `0.67305, 0.63726, 0.61921` at `x = 4, 9, 16` against the banked
   `0.664187, 0.627240, 0.608620`: low by 1.3 to 1.7 %. That moves the draft's bounds on
   `w₀` outside its own first range at every `x` and biases the defect by about `+0.03`.
   The "3.3 %" spread the bars lean on has no traceable source (the results file's
   `3.3e-3` is an absolute difference, about 0.5 %). S1's kill at `0.22` sits beside a cell
   at `0.225`; S2's at `0.16` beside `0.171`; the `0.40` window is the maximum of the row
   that is a kill only there.
3. **The reserved region is already breached by this campaign.** By the identity in 1, its
   own points are massive-model points at `m_eff/g = 0.031` to `0.088`: S1, S2 and the
   banked gap are the three reserved items at small mass. To first order one reading of
   `w₀` fixes the whole small-mass, small-background surface. The rule does not say whether
   "mass" is the lattice `μ` or the shifted mass, and it is honour-only. Rung IV can be
   blind only as a fixed-`x` lattice number where first-order theory fails: a mass of order
   `g`, a background that is not small, a nonlinear step.

## 2. What held

Every operator identity, the sum rule, the static identity and the inequality are correct
(residuals `≤ 5 × 10⁻¹⁵`); the bound is a theorem at each `N` and assumes no saturation.
Every row of the draft's synthetic tables reproduces to `0.001`. The draft is right that
`MATTER_SWEEP.md`'s "fully screened" sentence is false on this lattice at finite `x`. Not
noticed by the draft: in a state invariant under two-site translation the hopping terms of
the source cancel exactly over an even window, so the "remainder that shrinks with `x`" is
zero in the bulk and, printed, is a free meter of boundary contamination.

## 3. A possible correction to SCHWINGER-3, reported and NOT made

By the box law the continuum value of SCHWINGER-3's gap becomes `0.5654`, 0.2 % from
`1/√π`, where the record has `0.553116`, 2.0 % away. Its branch is unchanged. This rests on
three sizes a column and on the reviewer's fit; it needs the record's own instrument re-read
and, if it stands, a dated CORRECTION. It would be a correction in the record's favour.

## 4. The redesign, as tasks

- Credit rung I as a re-finding of the staggered mass shift read through a sum rule, and
  rename it.
- Why not both and compare: arm A at `μ = 0` (the draft) and arm B at `μ = −¼`, the
  chirally symmetric lattice point. Stake the anomaly's own content on B (the naive form
  with no offset; the gap from the hop alone) with A as the contrast that must fail by its
  `w₀`. Whether arm B may be computed under rung IV's reserve is the lead's decision.
- The referee: re-extrapolate under a dated CORRECTION to SCHWINGER-3, or drop it and stake
  `w₀` against the closure prediction written as a formula.
- Bars calibrated on the `3ω₁` row (bulk weight at `2ω₁` is forbidden by the lattice's
  charge conjugation), off cell edges, with the defect shown over a range of windows.
- Steps of `0.02` and `0.01`; the light-cone table fixed (`0.37` at `N = 20`, `0.42` at
  `N = 22`, at `x = 4`); branches per `x`; a branch for a failed linearity gate; the naive
  plant's void routed to the refuted branch.
- The reserve rewritten: the massless chain is an interior small-mass line; say which mass
  is meant; say the first-order surface is known.
- The instrument committed before the freeze.

Misfits contacted and not named: M-ONE-MODEL-DELTA, M-JOINT-PASS-REGION, M-FLOOR-UNSTAKED,
M-KINEMATIC-NONLOCAL, M-FOREIGN-DOMAIN-CORROBORATION, M-STALE-INSTRUMENT;
M-EXTRAPOLATED-HOLE is named but aimed at the wrong defect (the hole is the fit form).

## Not verified, and declined because of the reserve

Not verified: anything at `1.5 ≤ x ≤ 25` beyond arithmetic on banked gaps; that window
statics at `N = 22` reach the bulk; the papers beyond their abstracts. Declined: `w₀` or
either defect on the `μ = 0` chain within the window; `w₀` at `μ = −¼` within the window;
the massive gap or field at any background; any real-time run. Disclosed: the reviewer
evaluated the first-order continuum formula at `m/g = 1/(8√x)` from memory, which is a
numerical look within the window in letter; it types no number the draft does not hold.
