# KINK-0 — a unit charge is cheap and mobile in the bulk at half a unit of background field, and glued to the end at none. PREREGISTRATION

*Frozen 2026-11-11, committed alone, after two adversarial reviews by a second reader
(`KINK0_ANOMALY0_REVIEW.md` for the first; the second is summarised in §"What the second review
changed") and at the lead's standing word of the same day not to wait for his review. It
replaces `KINK0_PREREG_DRAFT.md` and `KINK0_PREREG_V2_DRAFT.md`, both kept. The title of this
file is the second draft's; branch (a)'s sentence below is narrower than the title and is what
the campaign may say.*

*What exists at this freeze: the complete instrument `kink0/kink0.py` with its `campaign`
mode, committed before this file; its gates G0 and G0b, both met; and a declared pilot at
`x = 5` (`kink0/pilot.log`). **What has and has not been seen.** Nothing has been computed at
the campaign's point (`x = 3`, `m/g ∈ {0.8, 0.15}`) by anyone. But the second review, answering
the author's own question about where the transition sits, computed a grid at
`x ∈ {2.5, 3.5, 6}` and seven masses that BRACKETS the campaign's point on both axes, and every
reading is smooth across it. So the campaign's numbers are unseen and can be interpolated to
a few per cent. **This run is a confirmation of a smooth surface, not a blind read**, and is
to be described as that wherever it is cited.*

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
- **Readings.** `ΔE± = E₀(Q = ±1) − E₀(Q = 0)`. The gap within the `−1` sector,
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
a stated margin. The pilot's point is not read again and carries no stake. The second review
found the bars transfer: the gap ratio, the field at the centre and the mass at which `ΔE₋`
changes sign barely depend on `x` between `2.5` and `6`.

**The light mass stays at `0.15`.** The review suggested `0.05`, which gives about twice the
contrast on every reading. It was not adopted: choosing the point with the larger effect
after seeing the grid would be choosing for the result. `0.15` is on the light side of every
clause by the review's interpolation, and `0.07` to `0.11` in `m/g` from the finite-size
crossover: safe for the stakes, not deep in the regime.

## Stakes

- **G0 (the instrument, massless).** `N = 12`, `x = 4`, massless, `Q = 0`, `α = 0`: the lowest
  energy agrees with the engine's typed referee `−26.1743157050`
  (`engine/crates/q8-mps/tests/schwinger_gauge.rs`) to `10⁻⁶`. Met in the build: `2.6 × 10⁻¹¹`.
- **G0b (the new terms).** `N = 8`, `x = 6`, `m/g = 1`, a random background per link,
  `Q ∈ {−1, 0, +1}`: the full sector spectrum equals that of a dense `W` built by Kronecker
  products and restricted to the sector, to `10⁻¹⁰`; with the mass sign reversed it differs by
  more than `1`. Met in the build: `8.5 × 10⁻¹⁴` and `2.05`.
- **S1 (EXACT; a constant of the sector, staked only as a check of the reader).** In every
  state, `Σ q_n = Q` to `10⁻¹⁰`. Every basis state of a sector has the same total, so this
  cannot fail for a normalised vector.
- **S2 (glued at no background, loose at half a unit).** Heavy. At `α = 0` the `−1` charge's
  distance from the right end differs by less than `0.05` sites between `N = 12` and
  `N = 18`; at `α = ½` it grows by at least `1.5` sites. `ΔE₊ − ΔE₋` at `α = 0` is reported,
  not staked (it exceeds one link's energy on any lattice). *Kill:* growth at `α = ½` below
  `0.5` sites.
- **S3 (a mobile object at half a unit).** `α = ½`. Heavy: `δ(18)/δ(12) ≤ 0.65`. Contrast: the
  light ratio exceeds the heavy by at least `0.15`. Direction: with
  `p(N₁, N₂) = ln(δ(N₁)/δ(N₂)) / ln(N₂/N₁)`, `p(16, 18) > p(12, 14)` at the heavy mass (the
  gap closes faster as the chain grows, as a level in a box does) and `p(16, 18) < p(12, 14)`
  at the light (it closes more slowly, as a gapped state does). *Kill:* the heavy ratio
  `≥ 0.80`, or the two ratios within `0.05`, or `p` falling at the heavy mass.
- **S4 (a finite, positive cost at the heavy mass).** `α = ½`. Heavy: `ΔE₋(18) > 0` and
  `|ΔE₋(18) − ΔE₋(16)| < |ΔE₋(16) − ΔE₋(14)|`. Light: `ΔE₋(18) < 0`, staked as a reading of
  this open chain's right end and NOT of the vacuum: in the undoubled regime `ΔE₋` tends to
  the difference of the end's energy between two exits, a lattice constant. No fit is
  staked; `M + b/N` and `M + b/N²` are both printed on the four sizes. *Kill:* `ΔE₋(18) < 0`
  at the heavy mass, or `ΔE₋(18) > 0` at the light one.
- **S5 (the field at the centre).** `α = ½`, `N = 18`, the `Q = 0` state. Heavy: `E_c`
  within `0.05` of `½`. Light: `E_c` lower than the heavy value by at least `0.10`. *Kill:*
  the two within `0.03`.

## What would make a stake vacuous

- S2's `α = 0` clause is close to trivial (the last site holds a `−1` at no field cost); the
  content is its contrast with `α = ½` at the same `x`.
- S3's ratio is a smooth function of mass with no feature; the DIRECTION clause is the one
  that changes sign across the crossover, and it is a sign, not a transferred bar.
- S4 has no fit to pass by construction; its "differences shrinking" clause held at every
  point the review computed, light or heavy, and is kept only as a sanity clause.
- The whole campaign is a confirmation, as the header says.

## Plants

- **P-MASS-SIGN** (carrier: `N = 8`, heavy, `α = 0`, `Q = 0`, with the mass term's staggering
  reversed; sector: the lowest energy). It must move by more than `10⁻³`.
- **P-BACKGROUND** (carrier: `N = 12`, heavy; sector: the charge sectors' labels).
  `E₀(Q = −1; α = 1) = E₀(Q = +1; α = 0)` to `10⁻⁹`, AND `E₀(Q = −1; α = 1) − E₀(Q = 0; α = 0)`
  differs from zero by more than `1`. The second half shows a mislabelled sector would be
  seen.
- **P-PINNED** (carrier: `N = 18`, heavy, `α = ½`, the `−1` sector; sector: the degeneracy of
  the two sides). Background raised to `0.55` on links `0` to `8` only: `X` moves LEFT by at
  least `1.0` site. Lowered to `0.45`: `X` moves RIGHT by at least `0.3`. Signed, so a
  reader blind to the sign cannot pass.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier: the mass term is nonzero at `μ = 2 × 0.8 × √3`; the background enters all 11
links' energies at `N = 12`; the pinning shift acts on 9 of 17 links. The plants cannot be
pre-checked on the carrier of record without seeing the campaign's point, so they run first
at that point and decide branch (e).

## Branches (in this order)

- **(e)** G0, G0b or a plant fails. Nothing is read.
- **(v)** Any solve raises. The stakes that read that state are VOID and the branch is (b)
  with the VOID named.
- **(m)** Every light-mass reading lands on the heavy side of the heavy bars: light ratio
  `≤ 0.65`, `ΔE₋(18) > 0`, `E_c` within `0.05` of `½`. Then the light point is on the heavy
  side of this lattice's crossover, no statement about the knob is made, and the contrast is
  owed in a new freeze. Any other pattern of light-mass failure is (b).
- **(a)** S1–S5 met. On this lattice, at half a unit of background and `m/g = 0.8`, the `−1`
  sector's lowest state is a step of the field that moves freely, at finite positive cost;
  at the light mass the same sector lies below the neutral one and its gap closes more
  slowly; and at zero background a unit charge stays at the end at every size. Two masses do
  not locate the transition. A re-finding of Coleman (1976), read as a confirmation.
- **(b)** Otherwise, stated with each clause that failed.

## What the second review changed

The instrument was confirmed line by line against an independent reader (`2 × 10⁻¹³` on every
field) and its referee was added as G0b; a fixed start vector and an independently computed
residual were added; the draft's S2 could not fail on either clause and now stakes the
contrast between the two backgrounds; S3 gained the direction clause; S4's light clause was
re-titled as a reading of the chain's end; "the 9 left links" became links `0` to `8`;
branch (m) was tightened so it is not an escape hatch; branch (v) was added; branch (a) lost
the sentence "at `m/g = 0.15` it is not", which the stakes do not deliver (a light charge
still moves into the bulk as the chain grows, only less). The review also reported that one
of its batches briefly drove this machine's load average above 300; no timing is staked here.

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
- M-VACUOUS-SUCCESS: the run prints the number of states solved and readings checked; the first draft's S4, which passed by construction, is the instance.
- M-NULL-MISSTAKE: S1's null is staked on the total within one sector, the quantity the conservation constrains, and is labelled EXACT for that reason.
- M-EXIT-DISCRIMINATOR: each solve either raises (VOID) or returns, and the largest independently computed residual `|Hv − Ev|` is printed.
- M-PARITY-PROTECT: S1 reads a constant within a conserved sector; the reflection identity is printed as the check on which sector is which.
