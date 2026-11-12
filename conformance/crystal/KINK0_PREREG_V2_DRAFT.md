# KINK-0 — a unit charge is cheap and mobile in the bulk at half a unit of background field, and glued to the end at none. PREREGISTRATION (second draft, after review)

*DRAFT of 2026-11-11, second version. NOT FROZEN until a second reader has attacked THIS text.
It replaces `KINK0_PREREG_DRAFT.md`, which the review of the same day
(`KINK0_ANOMALY0_REVIEW.md`) found wrong in two stakes and two plants, and whose parameter
point has been seen. What exists at this draft: the instrument `kink0/kink0.py`, its gate G0,
and a DECLARED PILOT at `x = 5`, `m/g ∈ {1.2, 0.1}` (`kink0/pilot.log`), which is not the
campaign's point and is used only to source the bars below. Nothing has been computed at the
campaign's point by anyone.*

## Why

As the first draft: FERMION-0 read that a fermion need not be an input on a `ℤ/2` gauge
substrate. This is the same question for a `U(1)` field in one dimension, where a charge is
a step in the electric field. Coleman (*Ann. Phys.* 101, 1976): with no background field the
two sides of the step differ in energy density and a lone charge exists only at an end or
bound to a partner; with half a unit of background and a heavy enough fermion the two sides
are degenerate and the step moves freely.

## What is already known (credited; none of it is claimed)

Schwinger (1962); Coleman, Jackiw and Susskind (1975); Coleman (1976), including the
"half-asymptotic" particle; the lattice spin form of Banks, Susskind and Kogut (1976); the
background field and string tension on the lattice, Hamer, Kogut, Crewther and Mazzolini
(1982); the transition at half a unit of background near `m/g ≈ 0.33`, Byrnes, Sriganesh,
Bursill and Hamer (2002). Every stake is a direction those papers give. A re-finding.

## Carrier

- **Hamiltonian** (`kink0/kink0.py`, the engine's spin form in its units):
  `W = x Σ (σ⁺_n σ⁻_{n+1} + h.c.) + (μ/2) Σ (−1)ⁿ σᶻ_n + Σ_{n<N−1} (L_n + α)²`,
  `L_n = Σ_{k≤n} q_k`, `q_k = (σᶻ_k + (−1)ᵏ)/2`, `μ = 2 (m/g) √x`. Open chain, `N` even.
- **Method.** Exact diagonalisation in a sector of total charge `Q ∈ {−1, 0, +1}`; the two
  lowest energies and the ground vector. ARPACK's own exit is the discriminator: a
  non-converged solve raises and that state is VOID.
- **The campaign's point.** `x = 3`; `m/g = 0.8` ("heavy", staked to be above the
  transition) and `m/g = 0.15` ("light", staked to be below it); `α ∈ {0, ½}`;
  `N ∈ {12, 14, 16, 18}`. At `x = 3` the lattice is coarse (the registry's
  `N ≳ 20√x = 35` for a continuum reading is NOT met): every statement is about this
  lattice at these sizes (M-VOLUME-SCALE, committed, not merely contacted).
- **Readings.** `ΔE± = E₀(Q = ±1) − E₀(Q = 0)`. The gap inside the `−1` sector,
  `δ = E₁ − E₀`. The charge's mean position `X = Σ n (q_n − q_n⁰)/Q` with `q⁰` the `Q = 0`
  state's profile, and its distance from the right end, `(N − 1) − X`. The step of the
  field it carries, `−(L_n − L_n⁰)`, as a two-link mean. The two-link mean of `⟨L + α⟩` at
  the centre of the `Q = 0` state, `E_c`.
- **Which sector.** At `α = +½` the `−1` charge takes the field to `−½` on its right, the
  degenerate side; the `+1` charge takes it to `3/2`. The build prints
  `E₀(−1; ½) − E₀(+1; −½)`, which is zero by the reflection identity
  `E₀(Q; α) = E₀(−Q; α + Q)`.

## The pilot, and what it is allowed to do

`x = 5`, `m/g ∈ {1.2, 0.1}`, the same sizes. It read (`kink0/pilot.log`): at `α = 0` heavy,
distance from the end `0.547` at every size and `ΔE₊ − ΔE₋ = 2.29`; at `α = ½` heavy,
`δ(18)/δ(12) = 0.518`, `ΔE₋ > 0` with successive differences shrinking, `E_c = 0.486`; at
`α = ½` light, `δ(18)/δ(12) = 0.781`, `ΔE₋ = −0.89`, `E_c = 0.256`; pinning the left half by
`±0.05` moved `X` by `−3.37` and `+1.21`. Each bar below is the pilot's reading loosened by
a stated margin. The pilot's point is not read again and carries no stake.

## Stakes

- **G0 (the instrument).** `N = 12`, `x = 4`, massless, `Q = 0`, `α = 0`: the lowest energy
  agrees with the engine's typed referee `−26.1743157050`
  (`engine/crates/q8-mps/tests/schwinger_gauge.rs`) to `10⁻⁶`. Met in the build: `2.6 × 10⁻¹¹`.
- **S1 (Gauss; EXACT, a check of the reader).** In every state, `Σ q_n = Q` to `10⁻¹⁰`.
- **S2 (glued to the end at no background).** `α = 0`, heavy. The `−1` charge's distance
  from the right end differs by less than `0.05` sites between `N = 12` and `N = 18`; and
  the `+1` charge, which cannot sit on the last site, costs more: `ΔE₊ − ΔE₋ ≥ 1.0` at
  `N = 18`. *Kill:* the distance growing by more than `1` site from `N = 12` to `N = 18`.
- **S3 (a mobile object at half a unit).** `α = ½`. Heavy: `δ(18)/δ(12) ≤ 0.65`, and the
  step is monotone at every size. And the contrast with the light mass:
  `δ(18)/δ(12)` is larger at the light mass than at the heavy by at least `0.15`.
  *Kill:* the heavy ratio `≥ 0.80` (the lowest state is not a particle in a box), or the two
  ratios within `0.05` of each other (the knob does nothing).
- **S4 (cheap where the vacuum is doubled, and not a vacuum where it is not).** `α = ½`.
  Heavy: `ΔE₋(18) > 0` and `|ΔE₋(18) − ΔE₋(16)| < |ΔE₋(16) − ΔE₋(14)|`. Light:
  `ΔE₋(18) < 0`, so the neutral state is not the lowest. No fit is staked; `M + b/N` and
  `M + b/N²` are both printed on the four sizes. *Kill:* `ΔE₋(18) < 0` at the heavy mass, or
  `ΔE₋(18) > 0` at the light one.
- **S5 (the field at the centre).** `α = ½`, `N = 18`, the `Q = 0` state. Heavy: `E_c`
  within `0.05` of `½`. Light: `E_c` lower than the heavy value by at least `0.10`. *Kill:*
  the two within `0.03`.

## What would make a stake vacuous

- S2's first clause is close to trivial, as the review said: the last site can hold a `−1`
  at no field cost. Its second clause is the content.
- S3's heavy bar alone would pass for any box level; the contrast clause is what the knob
  earns. If the light mass at `x = 3` turns out to lie ABOVE the lattice's transition, S3's
  contrast and S4's and S5's light clauses all fail together, and that is read as "the
  light point was mis-chosen", branch (m), not as three kills.
- S4 has no fit to pass by construction.

## Plants

- **P-MASS-SIGN** (carrier: `N = 8`, heavy, `α = 0`, `Q = 0`, with the mass term's staggering
  reversed; sector: the lowest energy). It must move by more than `10⁻³`.
- **P-BACKGROUND** (carrier: `N = 12`, heavy; sector: the charge sectors' labels).
  `E₀(Q = −1; α = 1) = E₀(Q = +1; α = 0)` to `10⁻⁹`, AND `E₀(Q = −1; α = 1) − E₀(Q = 0; α = 0)`
  differs from zero by more than `1`. The second half shows a mislabelled sector would be
  seen.
- **P-PINNED** (carrier: `N = 18`, heavy, `α = ½`, the `−1` sector; sector: the degeneracy of
  the two sides). Background raised to `0.55` on the 9 left links only: `X` moves LEFT by at
  least `1.0` site. Lowered to `0.45`: `X` moves RIGHT by at least `0.3`. Signed, so a
  reader blind to the sign cannot pass.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier: the mass term is nonzero at `μ = 2 × 0.8 × √3`; the background enters all 11
links' energies at `N = 12`; the pinning shift acts on 9 of 17 links.

## Branches (in this order)

- **(e)** G0 or a plant fails. Nothing is read.
- **(m)** Every light-mass clause fails together (S3's contrast, S4's light sign, S5's light
  bar). The light point is on the wrong side of this lattice's transition; the heavy-mass
  readings stand and the contrast is owed at a lighter mass in a new freeze.
- **(a)** S1–S5 met. On this lattice, at half a unit of background and `m/g = 0.8`, the `−1`
  sector's lowest state is a mobile step of the field at finite, positive cost; at
  `m/g = 0.15` it is not; and at zero background a unit charge is glued to the end. A
  re-finding of Coleman (1976) at four sizes on one coarse lattice.
- **(b)** Otherwise, stated with each clause that failed.

## What this does not claim

Not a lepton; not chirality (the model is vector-like); not the location of the transition
(two masses do not locate it); not a continuum statement; not 3+1 dimensions; not a
discovery.

---
witness: `CIRISHolon.SceneCharge.gpop_total` (S1: the total charge read coarsely or finely)
witness: none (S2–S5 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-VOLUME-SCALE, M-BAR-FROM-THE-READ, M-VACUOUS-SUCCESS, M-NULL-MISSTAKE, M-EXIT-DISCRIMINATOR, M-PARITY-PROTECT. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant is built for this reader, and two were rebuilt after the review found them unable to fire.
- M-VOLUME-SCALE: committed. `N ≤ 18` at `x = 3` is below the registry's own floor for a continuum reading, and no continuum reading is made.
- M-BAR-FROM-THE-READ: every bar comes from a declared pilot at a different point, with its margin stated; none comes from the campaign's own numbers.
- M-VACUOUS-SUCCESS: the first draft's S4 passed by construction; this one stakes no fit, and §"What would make a stake vacuous" names what remains.
- M-NULL-MISSTAKE: P-PINNED is signed and is the control that S3 reads a degeneracy.
- M-EXIT-DISCRIMINATOR: the solver's exit decides VOID, not a residual.
- M-PARITY-PROTECT: the sectors are fixed by the number of up spins, a conserved count; the reflection identity is printed as the check on which sector is which.
