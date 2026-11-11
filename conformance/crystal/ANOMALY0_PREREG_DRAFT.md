# ANOMALY-0 — the continuity law with a source: the axial anomaly of the Schwinger model read in real time on the engine's own evolver. PREREGISTRATION (DRAFT)

*DRAFT of 2026-11-11. NOT FROZEN, and not to be frozen before a second reader has attacked
it (the author's FERMION-0 freeze carried two definitional errors). Nothing here has been
computed. §"Open design questions" is written for that reader.*

## Why

`LEPTON_LADDER.md` names the axial anomaly in 1+1 dimensions (route R6) as the first
measured statement about handedness this ladder can make. And the CEWP twin has asked the
world model for quantities indexed by time (who is where, by hour). The two are one law:

> what a cell holds changes only by what crosses its faces, plus a declared source.

For people, and for ordinary electric charge, the declared source is zero. For the HANDED
charge of a massless fermion in a gauge field it is not: it is the electric field times a
fixed number, `1/π` in the continuum, whatever the coupling. This campaign reads both on one
carrier with one checker (`holon-scene`'s continuity check, built for the twin's rhythm
rows): the ordinary charge with source zero, exactly; the handed charge with its source.

## What is already known (credited; none of it is claimed)

- The anomaly in two dimensions and the mass it gives the photon: Schwinger (1962); Adler;
  Bell and Jackiw (1969); Johnson (1963).
- Its lattice form for staggered fermions: Banks, Kogut and Susskind (1976); Ambjørn,
  Greensite and Peterson (1983) for the real-time picture of level crossing.
- That in two dimensions the axial charge density IS the spatial vector current, so the
  axial charge is the total current.
- Real-time tensor-network evolution of the Schwinger model after a field quench: Buyens et
  al. (2014, 2017) and others. The campaign re-finds.

## Carrier

- **Model.** `engine/crates/q8-mps/src/schwinger.rs`: `W = x Σ (σ⁺_n σ⁻_{n+1} + h.c.) +
  Σ (L_n + ε_n)² + λQ²`, `L_n = Σ_{k≤n} q_k`, static charges entering as the background `ε_n`.
- **Operators on link `n`.** The kinetic density `K_n = σ⁺_n σ⁻_{n+1} + h.c.`; the current
  `J_n = i (σ⁺_n σ⁻_{n+1} − h.c.)`, with the sign fixed in the build so that
  `dL_n/dτ = −x J_n` holds as an operator identity; the field `E_n = L_n + ε_n`.
- **Protocol.** The vacuum of the chain with no static charge is found by the engine's DMRG.
  At `τ = 0` a static `+1` is placed at site `a = N/4` and a `−1` at `b = 3N/4`, so one unit
  of background field appears between them, and the state is evolved under the new `W` by
  `tdvp.rs` (single-site, complex) to `τ_max`.
- **Sizes.** `x ∈ {4, 9, 16}`, `N` from SCHWINGER-3's smallest rung at each `x` (40, 60, 80),
  bond dimension 40 and 64 (the same premise SCHWINGER-3 staked: the two must agree).
- The **readings, at every step.** `⟨q_n⟩`, `⟨J_n⟩`, `⟨K_n⟩`, `⟨E_n⟩` on every site and link, and
  `⟨E_n K_n + K_n E_n⟩/2` on every link.
- **Cost.** Not measured. One step at the smallest size is timed and the extrapolation
  printed before anything else runs; if the whole ladder exceeds a stated ceiling the
  campaign is cut to `x = 4` and says so.

## Stakes

- **G0 (the evolver on this model).** With no charge placed, the vacuum is stationary: over
  `τ_max` the energy drifts by less than `10⁻⁸` relative and every `⟨J_n⟩` stays below `10⁻⁶`.
- **G1 (the bond-dimension premise).** Every staked quantity agrees between bond dimensions
  40 and 64 to within a tenth of its own budget. Otherwise VOID, not killed.
- **S1 (ordinary charge: the source is zero).** At every step and every site,
  `q_n(τ+δ) − q_n(τ)` equals the time-integrated current in minus current out, to within the
  integrator's own error, staked as a residual below `10⁻⁴` of the largest term. *Kill:* a
  residual above `10⁻²`. Witness: `Ladder.outflow_commutes`, in time.
- **S2 (handed charge: the source is the field times the kinetic density, exactly).** The
  axial charge on the interior `Q₅ = Σ_{n=a}^{b−1} J_n` obeys
  `dQ₅/dτ = c · Σ ⟨E_n K_n + K_n E_n⟩/2 + (boundary terms at a and b)`, an operator identity
  with `c` and the boundary terms derived in the build and stated before the read. Residual
  below `10⁻⁴` of the largest term. *Kill:* above `10⁻²`.
- **S3 (the anomaly coefficient).** At `τ = 0⁺` the identity of S2 reduces to
  `d⟨J_n⟩/dτ = c · ε_n · ⟨K_n⟩_vac`. The continuum value corresponds to `⟨K_n⟩_vac = −2/π`
  per link. Stake: the bulk average of `−π ⟨K_n⟩_vac / 2` is within `0.10` of 1 at `x = 16`,
  closer to 1 at each larger `x`, and its three values extrapolate in `1/√x` to within
  `0.03` of 1. *Kill:* the sequence moving away from 1 with `x`.
- **S4 (the photon's mass, in real time).** The field at the midpoint oscillates. Its
  frequency, as `M/g = ω/(2√x)`, agrees with SCHWINGER-3's `M(x, N)` at the same `x` and `N`
  to within `3 %`. *Kill:* a disagreement above `10 %`.

## Plants

- **P-FIELD-OFF** (carrier: the same run with no charge placed; sector: the source). `Q₅`
  must not move: this is G0, restated as the plant that the source vanishes with the field.
- **P-COULOMB-OFF** (carrier: the run with the engine's `Mutation::CoulombOff`; sector: the
  dynamical field energy). The source term of S2 must vanish and `Q₅` must then follow free
  fermions in a static potential; S4's oscillation must be absent.
- **P-CONTINUITY** (carrier: S1's series with one site's charge series shifted by one step;
  sector: the checker). The continuity check must refuse it and name the site.
- **P-SIGN** (carrier: S2 evaluated with `J_n` replaced by `−J_n`; sector: the identity).
  The residual must exceed 1.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier: the placed charges put one unit of field on `N/2` links; the Coulomb channel is
nonzero at every `x` used; the shifted series differs from the original at every step where
the current is nonzero, checked first; `J_n` is nonzero after `τ = 0`.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(v)** G1 fails. VOID at these bond dimensions.
- **(a)** S1–S4 met. The same continuity law holds for both charges; the handed one has a
  source; the source is the field times the vacuum's own kinetic density; and that density
  tends to the value that makes the coefficient `1/π`. A re-finding.
- **(b)** S1, S2 met, S3 or S4 not. The identity holds and the continuum number is not
  reached at these sizes; stated with the size.
- **(k)** S1 or S2 killed. The operators or the evolver are wrong; the failure is the finding.

## Open design questions (for the second reader)

1. **Is the dynamics needed at all for S3?** At `τ = 0⁺` the coefficient is a STATIC vacuum
   expectation, `⟨K_n⟩`, which DMRG gives without any evolution. Then the anomaly reading is
   one more observable on the banked ground states, and the real-time run earns only S1, S2
   and S4. Should S3 be split off as a static campaign on SCHWINGER-3's full ladder?
2. **Is S3 trivial?** As `x → ∞` the hopping dominates and `⟨K⟩` must tend to the free
   value. If so the stake is a statement about lattice artefacts, not about the anomaly, and
   should be worded as one.
3. **The operator identity of S2:** signs, the factor `c`, the anticommutator ordering, and
   the boundary terms where the interior sum is cut at `a` and `b`. These must be derived on
   paper, checked on a 4-site exact diagonalisation, and committed before any run.
4. **Unit of field.** One whole unit of background is a strong quench, not linear response.
   Is the continuum comparison in S4 still the free photon's frequency, or does the strong
   field pair-produce and shift it? A half-unit background would need the model to accept a
   non-integer `ε`.
5. **Single-site evolution at fixed bond dimension** cannot grow entanglement. Is G1's
   two-rung comparison enough, or is a two-site or subspace-expanded step owed first?
6. **The bars** (`0.10`, `0.03`, `3 %`) are the author's guesses. A pilot at a parameter
   point NOT in the campaign, declared as a pilot, could source them.

## What this does not claim

Not a lepton, not chirality in the sense of the weak interaction (the model is vector-like;
the anomaly here is of a global symmetry), not 3+1 dimensions, not a discovery.

---
witness: `CIRISHolon.Ladder.outflow_commutes` (S1, the law at one time; its form in time is owed in `SceneRhythm.lean`)
witness: none (S2–S4 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-VOLUME-SCALE, M-BAR-FROM-THE-READ, M-TRUNCATION-AS-ERRORBAR, M-EXIT-DISCRIMINATOR, M-NULL-MISSTAKE, M-CHEAPER-THAN-ITS-PRICE, M-HOMOG. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant is built for this reader.
- M-VOLUME-SCALE: three lattice spacings at one size each; the continuum statement of S3 is an extrapolation and is labelled one.
- M-BAR-FROM-THE-READ: question 6 says the bars are not yet sourced.
- M-TRUNCATION-AS-ERRORBAR: no discarded weight is read as an error bar; G1 compares two bond dimensions instead.
- M-EXIT-DISCRIMINATOR: the evolver's Krylov exit is recorded per step, and a step that hits its iteration cap voids the run.
- M-NULL-MISSTAKE: P-FIELD-OFF and P-COULOMB-OFF are the two nulls, and question 2 asks whether S3 is a null in disguise.
- M-CHEAPER-THAN-ITS-PRICE: no cost is claimed before one step is timed.
- M-HOMOG: "bulk" and "interior" mean the links between the two placed charges.
