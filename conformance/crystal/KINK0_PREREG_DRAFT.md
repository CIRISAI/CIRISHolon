# KINK-0 — a charge that exists in the bulk only where the vacuum is doubled: the massive Schwinger model at half a unit of background field. PREREGISTRATION (DRAFT)

**REVIEWED 2026-11-11: DO NOT FREEZE AS WRITTEN. See `KINK0_ANOMALY0_REVIEW.md`. The parameter point below (`x = 4`, `m/g ∈ {1, 0.1}`) has been SEEN at `N = 8, 10, 12` and cannot be staked blind; S3, S4 and two plants are wrong as worded. This text is kept as drafted, for the record of what was reviewed.**

*DRAFT of 2026-11-11. NOT FROZEN, and not to be frozen on the author's word alone: FERMION-0's
freeze carried two definitional errors of the author's, both caught before a circuit existed
(`FERMION0_AMENDMENT_1.md`, `FERMION0_AMENDMENT_2.md`). This draft has had one pass. §"Open
design questions" lists what a second reader should check. Nothing here has been computed.*

## Why

FERMION-0 read that on a `ℤ/2` gauge substrate a fermion is a closed view of a vacuum holding
none. The next rung of the question in `LEPTON_LADDER.md` is the same statement where the
gauge field is the one the chemistry tier's electron actually couples to, a `U(1)` field, in
the one dimension where this engine has a banked instrument: the Schwinger model
(`SCHWINGER3_RESULTS.md`, mass from the vacuum to 2 %).

There the literature's answer is sharp and depends on one knob. A charge in 1+1 dimensions
is, by Gauss's law, a STEP in the electric field: the field to its right differs from the
field to its left by one unit. With no background field the two sides have different energy
density, a lone charge costs energy in proportion to the length of fluxed line behind it,
and it exists only glued to a boundary or to a partner. With half a unit of background field
the two sides (`+½` and `−½`) are degenerate for a heavy enough fermion, and the step is a
free particle of finite mass: Coleman's "half-asymptotic" particle, the fermion as a kink
between two vacua (Coleman, *Ann. Phys.* 101 (1976)).

In this record's vocabulary: is a charge an OBJECT in the bulk? At zero background, no; at
half a unit, yes. The world-model theory says a charge is its outflow (`Ladder.outflow_commutes`,
`SceneCharge.no_lone_charge`); this campaign reads where that outflow is free.

## What is already known (credited; none of it is claimed)

- The model and its bosonisation: Schwinger (1962); Coleman, Jackiw and Susskind (1975);
  Coleman (1976). Charge is the winding of the boson field between vacua.
- The lattice spin form used here: Hamer and Kogut; Banks, Kogut and Susskind (1976).
- The transition at half a unit of background field at `m/g ≈ 0.33`: Byrnes, Sriganesh,
  Bursill and Hamer (2002), and the tensor-network literature after it.
- Every stake below is a direction those papers give. The campaign re-finds.

## Carrier

- **Hamiltonian** (the instrument's spin form, `instrument/dmrg_schwinger.py`, in its units):
  hopping `x Σ (σ⁺σ⁻ + h.c.)`; mass `(μ/2) Σ (−1)ⁿ σᶻ`; field energy `Σ (L_n + α)²`, with
  `L_n = Σ_{k ≤ n} q_k` and `q_n = (σᶻ_n + (−1)ⁿ)/2`. Open chain, the background `α` entering
  at the left end. `x = 1/(ga)²`, `μ = 2m/(g²a)`.
- **Method.** Exact diagonalisation in a fixed sector of total charge `Q`, sparse, lowest
  state only. `N ∈ {8, 10, 12, 14, 16, 18}` sites. No DMRG and no long run: the largest
  sector is `C(18, 9) = 48,620` states.
- **Parameters.** `x = 4`; `m/g = 1` (well above the transition) and, as a contrast,
  `m/g = 0.1` (well below it); `α ∈ {0, ½}`.
- **Observables.** `ΔE(N) = E₀(Q = −1) − E₀(Q = 0)`. The field profile `⟨L_n + α⟩`. The
  charge's mean position `X = Σ n ⟨q_n − q_n^{vac}⟩ / Q`, in units of `N`, and its spread.

## Stakes

- **G0 (the instrument).** At `μ = 0`, `α = 0`, the lowest `Q = 0` energy at `N = 8, 12, 16`
  agrees with the banked exact diagonalisation of SCHWINGER-1 to `10⁻⁶`. EXACT in that sense.
  If not, nothing runs.
- **S1 (Gauss, exact).** In every state computed, `⟨L_{N−1}⟩ = Q` and
  `⟨L_n⟩ − ⟨L_{n−1}⟩ = ⟨q_n⟩` to `10⁻¹⁰`. This is an identity of the spin form and is staked
  only as a check on the reader. *Kill:* 1 violation.
- **S2 (no bulk charge at zero background).** At `α = 0`, `m/g = 1`: the charge of the
  `Q = −1` state sits at the right end, `X/N ≥ 0.85` at `N = 18`, and the distance of `X`
  from the end changes by less than 1 site between `N = 12` and `N = 18`. *Kill:*
  `X/N ≤ 0.65` at `N = 18`.
- **S3 (a bulk charge at half a unit).** At `α = ½`, `m/g = 1`: the charge is spread over
  the chain, `0.40 ≤ X/N ≤ 0.60` at every `N`, with a spread that grows with `N` by at least
  a factor `1.3` from `N = 12` to `N = 18`. *Kill:* `X/N ≥ 0.85` at `N = 18`.
- **S4 (a finite mass).** At `α = ½`, `m/g = 1`, `ΔE(N)` falls with `N` and its last three
  values fit `M + b/N²` with a residual under 5 % of `M`. *Kill:* `ΔE` rising with `N` over
  the last three sizes.
- **S5 (the knob is the vacuum's doubling, not the mass alone).** At `α = ½`, `m/g = 0.1`,
  below the transition, the two field values are not degenerate vacua and S3's signature is
  absent: the field profile of the `Q = 0` state is not flat at `+½`, departing from it by
  at least `0.15` at the centre of the chain at `N = 18`. No kill: the contrast is reported.

## Plants

- **P-MASS-SIGN** (carrier: the `N = 8` Hamiltonian with the mass term's staggering
  reversed; sector: the lowest `Q = 0` energy). The energy must move by more than `10⁻³`.
  Fires only if it does, so that a sign error in the new term cannot hide.
- **P-BACKGROUND** (carrier: `α = 1` at `N = 12`; sector: the spectrum). A whole unit of
  background is the same physics as none with the charge sectors relabelled by 1: the
  lowest energy of sector `Q − 1` at `α = 1` must equal that of sector `Q` at `α = 0` up to
  the boundary term, stated exactly in the build before it is read.
- **P-PINNED** (carrier: S3's state with a field energy of `+0.5` added on the left half of
  the chain; sector: the degeneracy of the two sides). The charge must then pin: `X/N`
  moves outside `[0.40, 0.60]`. It shows S3 reads a degeneracy and not the instrument.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier: the mass term is nonzero at `μ = 2`; the background enters every link's energy;
P-PINNED's added energy acts on 9 links of 18.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** S1–S4 met. A `U(1)` charge is an object in the bulk exactly where the vacuum is
  doubled, and is its own field's step everywhere. A re-finding of Coleman (1976).
- **(b)** S2 met, S3 or S4 killed. At these sizes the kink is not resolved; the sizes are
  the finding and DMRG at the banked ladder is the next freeze.
- **(c)** S2 killed. The reader's picture of confinement on an open chain is wrong; examine
  the boundary before anything else.

## Open design questions (for a second reader, before any freeze)

1. **Boundary conditions.** On an open chain the `Q = −1` sector forces one unit of flux out
   of the right end. S2 stakes that the charge hugs that end. Is the right statement instead
   about two opposite charges and the energy against their separation? That needs a pinning
   term or a constrained sector and is closer to the textbook string tension.
2. **Sizes.** `N ≤ 18` at `x = 4` is a coarse lattice with a correlation length of a few
   sites. Are S3's and S4's numbers resolvable there, or are they bars set by feel
   (M-BAR-FROM-THE-READ in the other direction)? A pilot on a DIFFERENT parameter point,
   declared as such, could set them honestly.
3. **The vacuum subtraction in `X`.** `q_n^{vac}` is the `Q = 0` state's charge at the same
   `α`. At `α = ½` below the transition that state is itself polarised.
4. **Which `Q`.** With staggered sites the sign conventions make `Q = +1` and `Q = −1`
   inequivalent at `α = ½`: one of them moves the field to `−½`, the other to `+3/2`. The
   draft assumes `−1` is the degenerate one; the build must state it from the Hamiltonian.

## The anomaly, a note and not a stake

`LEPTON_LADDER.md` names the 1+1-dimensional axial anomaly (route R6) as the cheapest
measured statement about chirality. In two dimensions the axial charge density is the
spatial component of the vector current, and the anomaly equation together with Maxwell's
gives the field a mass with `M²/g²` equal to the anomaly coefficient, `1/π`. So the banked
`M_V/g = 0.553116` is already an INDIRECT reading of the coefficient: `(M_V/g)² = 0.3059`
against `1/π = 0.3183`, 3.9 % low, inheriting SCHWINGER-3's 2 % on the mass. It is indirect
because it assumes the continuum operator identity. A direct reading needs real-time
evolution of the current under a field, which this instrument does not have.

## What this does not claim

Not a lepton, not chirality (the model is vector-like), not 3+1 dimensions, not a
continuum limit, not a discovery.

---
witness: `CIRISHolon.Ladder.outflow_commutes` (S1, on counts; the charge form is the model's Gauss law)
witness: `CIRISHolon.SceneCharge.no_lone_charge` (S2: a lone charge in the bulk)
witness: none (S2–S5 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BARE-CHARGE, M-VOLUME-SCALE, M-BAR-FROM-THE-READ, M-HOMOG, M-NULL-MISSTAKE, M-EXIT-DISCRIMINATOR. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant is built for this reader.
- M-BARE-CHARGE: S2 is the bare charge's fate with no partner in reach.
- M-VOLUME-SCALE: six sizes at one lattice spacing; nothing is claimed about the continuum.
- M-BAR-FROM-THE-READ: open question 2 says the bars are not yet honestly sourced.
- M-HOMOG: "bulk" means away from the two ends of this chain.
- M-NULL-MISSTAKE: P-PINNED is the control that S3 reads a degeneracy.
- M-EXIT-DISCRIMINATOR: the sparse solver's exit (converged or iteration cap) is recorded per state, and a state that hits the cap is VOID.
