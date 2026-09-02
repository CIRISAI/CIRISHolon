# Pre-registration — GF1: the magic price of gauge vacua

*Frozen 2026-09-02, committed ALONE, before its instrument exists. Node GF1 of
the fold below the atom (`OBJECT.md` "The fold below the atom", LOCKED
2026-09-02). Fold III says the interacting gauge vacuum in a hadron-sized box
is a LOW-MAGIC, area-law state because both fixed points of the confined phase
are stabilizer states, and that the magic stratum can therefore PRICE it. This
campaign measures that price on the simplest gauge theory with both fixed
points, exactly, and stakes what a bounded price looks like before any number
is seen. Nothing here is a claim about SU(3); it is the price card Fold III
must carry into GF2.*

misfits: contacts M-PLANT-OBS and M-PLANT-SECTOR (three plants below, each
with its carrier asserted nonzero in the sector the plant acts on),
M-VOLUME-SCALE (the lattice sizes are staked per rung with their finite-size
premise, and a density is read, never a total, so the volume enters
explicitly), M-NULL-MISSTAKE (the gates are staked on the quantity the fold
controls — the magic DENSITY per link and its convergence — never on a total
that grows with the box by construction), M-ONE-MODEL-DELTA (every verdict
compares to an EXACT value: a stabilizer state's zero, a known single-qubit
magic value, or the instrument's own smaller lattice; no fitted model
anywhere), M-STALE-INSTRUMENT (the instrument, this freeze and the results
document are committed together with the instrument's own hash),
M-CHEAPER-THAN-ITS-PRICE (the cost model is stated per rung: the exact Pauli
sum is 4^n over the links, which is why rung 1 stops at twelve links and rung
1b names its own instrument), M-HOMOG (the local Hilbert space is one qubit
per link everywhere; no spatial inhomogeneity is claimed or hidden),
M-MAINTENANCE-LENS (no repair or maintenance claim; the word "rent" does not
occur), M-BARE-CHARGE (the ground state is constructed INSIDE the Gauss-law
sector, never projected from a bare state), M-DEVICE-CLASS (CPU only, one
device class; the exact sums are integers-in-disguise and the results document
prints the numpy build). Not contacted: M-GAUGE-LAUNDER, M-LOOP-BLIND,
M-COND-PROBE, M-ELECTRIC-BASIS, M-RING-MIXING, M-GAUGE-UNIFORM-MOMENTUM,
M-KINEMATIC-NONLOCAL, M-FIXED-POINT-TRAJECTORY, M-NONBIJECTIVE-STEP,
M-FINAL-VIEW-COLLISIONS, M-PROBE-EIGENSTATE, M-PARITY-PROTECT,
M-IDLE-CALIBRATED-TIMEOUT, M-PLACEMENT-LOTTERY, M-PROVENANCE-OVERREACH (rung 1
runs in seconds and carries no launch header; rung 1b's results document will).

## Model

The Z₂ lattice gauge theory (Wegner 1971; Kogut 1979; Kitaev 2003 for the
λ = 0 point, all credited) on an `L_x × L_y` periodic square lattice with one
qubit per link, `n = 2 L_x L_y` links:

```
H(λ) = − Σ_p B_p − λ Σ_l X_l,     B_p = ∏_{l ∈ ∂p} Z_l
Gauss:  A_v = ∏_{l ∋ v} X_l = +1 at every vertex  (the physical sector)
```

The physical sector has dimension `2^{L_x L_y + 1}` (one Gauss constraint is
redundant on the torus): 32 states at 2×2, 128 at 3×2, 512 at 4×2, 1,024 at
3×3. The ground state is found by exact diagonalisation INSIDE that sector.
Both fixed points are stabilizer states: at λ = 0 the toric-code ground state,
at λ → ∞ the product `|+⟩^⊗n`. The confinement transition sits at
`λ_c = 0.3285` by the duality to the 3D Ising model (Wegner; Blöte–Deng 2002
for the number), cited as the referee for the location of the price's maximum
and for nothing else.

## The monotone

The stabilizer 2-Rényi entropy (Leone, Oliviero, Hamma, PRL 128, 050402, 2022,
credited): for a pure state on `n` qubits,

```
M₂(ψ) = − log₂ ( 2^{−n} Σ_{P ∈ 𝒫_n} ⟨ψ|P|ψ⟩⁴ )
```

with the sum over all `4^n` Pauli strings. It is zero exactly on stabilizer
states, additive on products, invariant under Clifford motions, and a magic
monotone for pure states. It is the price PROXY this rung can compute EXACTLY;
the magic stratum's operational cost (stabilizer extent) is not computed here
and no inequality between the two is asserted. The reading is the DENSITY
`m₂ = M₂ / n` per link.

## Rungs, grid and cost model

| rung | lattices (links) | λ | instrument | cost model, stated |
|---|---|---|---|---|
| 1 | 2×2 (8), 3×2 (12) | 0, 0.1, 0.2, 0.25, 0.3, 0.33, 0.36, 0.4, 0.5, 0.7, 1.0, 2.0, ∞ | python: ED in the Gauss sector; the full Pauli sum, `4^8` and `4^12` terms | seconds and minutes respectively |
| 1b | 4×2 (16), 3×3 (18) | the same | the exact tiers: Pauli expectations restricted to the Gauss centralizer (`4^n / 2^{L_x L_y − 1}` strings), `holon-qasm`'s tableau tier as the Pauli engine | `3.4e7` and `2.7e8` strings against 65k and 262k amplitudes; rung 1b freezes its gates here and delivers with its own instrument's hash |

`λ = ∞` is read as the product state directly, not as a limit.

## Frozen gates

- **G0 — both fixed points read EXACTLY zero, at every lattice**: `M₂ ≤ 1e-10`
  at λ = 0 and at λ = ∞ (the states are exact stabilizer states; the bound is
  the ED's own floating residual, stated). A nonzero here is an instrument
  defect, never physics. witness: none (the zero is definitional for
  stabilizer states; the theorem that MAGIC is the wall this price stands
  behind is `tableau_not_closed_under_rotation`, cited for the mechanism, not
  the number)
- **G1 — the maximum of the density sits at the transition**: on every
  lattice the λ maximising `m₂` lies in `[0.2, 0.5]`, the band containing
  `λ_c = 0.3285` with the finite-size shift a 2×2 torus can carry. Digit-bearing.
  witness: none (the referee is the duality's transition point, cited)
- **G2 — the confined phase is CHEAPER than the transition, at every lattice**:
  `m₂(λ = 1.0) < 0.75 · m₂(λ_max)` and `m₂(λ = 2.0) < m₂(λ = 1.0)`. The fold's
  content is that the physical, confined side of the transition is closer to a
  stabilizer state than the critical point is. witness: none
- **G3 — rung 1b's convergence stake, the KILL**: in the confined phase,
  `λ ∈ {0.5, 1.0}`, the density converges with the box — the two largest
  lattices agree better than the two smallest:
  `|m₂(3×3) − m₂(4×2)| ≤ 0.5 · |m₂(3×2) − m₂(2×2)|`, and separately
  `|m₂(4×2) − m₂(3×2)| ≤ 0.75 · |m₂(3×2) − m₂(2×2)|`. A gapped vacuum's
  magic is extensive and its density converges exponentially; a density that
  does NOT converge means no grain-sized box has a bounded price, and **Fold
  III dies outright**. At the transition (`λ = 0.33`) convergence is NOT
  staked (criticality is where the density is allowed to drift with L) and
  the drift is reported. witness: none (measured premise)
- **G4 — the price card**: the numbers `m₂(λ, L)` for every point, the
  extrapolated confined-phase density `m₂(λ = 1.0, L → ∞)` from the three
  largest lattices in `1/n`, and the implied `M₂` of a `4×4×4` three-dimensional
  grain (`192` links) at that density, printed as the card GF2 inherits. The
  card is a MEASUREMENT of a proxy, stated as one; the stratum decision it
  informs (magic tier vs DMRG for the hadron box) is GF2's prereg's business
  and is not made here. Digit-bearing: 192.

## Plants (M-PLANT-OBS, M-PLANT-SECTOR discharged here)

- **(i) A known magic value, EXACT and additive.** `k` copies of
  `|T⟩ = (|0⟩ + e^{iπ/4}|1⟩)/√2` on `k` links and `|0⟩` elsewhere must read
  `M₂ = k · log₂(4/3)` to `1e-12` for `k ∈ {1, 2, 3}` (single-qubit value:
  `⟨X⟩ = ⟨Y⟩ = 1/√2`, `⟨Z⟩ = 0`, so `Σ⟨P⟩⁴ = 3/2`). Carrier asserted nonzero in
  the sector the plant acts on: `|⟨X⟩| > 0.7` on each planted link.
- **(ii) The instrument must be able to be BLIND, and the plant must see it.**
  Mutate the exponent in the Pauli sum from 4 to 2: then `Σ⟨P⟩² = 2^n` for
  every pure state and the mutated instrument reads `M₂ = 0` for everything —
  a monotone that cannot fire. Under the mutation the `|T⟩` plant must read
  `< 1e-12` where the true value is `0.415`: the gauge fires on the blindness.
  Carrier: the same nonzero `⟨X⟩` as (i).
- **(iii) Gauss is enforced, not assumed.** A state prepared OUTSIDE the
  sector (one link flipped from the λ = ∞ product state, so exactly two
  vertices carry `A_v = −1`) must be REFUSED by the sector constructor with
  the two vertices named; and the toric-code ground state constructed by
  the instrument must satisfy `⟨B_p⟩ = 1` on every plaquette to `1e-12`
  (nonzero carrier: `⟨B_p⟩` itself). EXACT.

## Meaning

G0–G2 pass and G3 passes ⇒ the magic price of the Z₂ gauge vacuum is bounded
per link in the confined phase, largest at the transition, exactly zero at
both fixed points: Fold III has its price card at the smallest gauge theory
with both fixed points, and GF2 inherits the card. G3 fails ⇒ Fold III dies
and is kept dead: the interacting vacuum's magic has no bounded density and
no grain-sized box has a bounded price on the magic stratum. G1 or G2 failing
with G3 passing ⇒ the price is bounded but its SHAPE is not the fold's
(reported; the card still transfers). G0 failing ⇒ the instrument is wrong
and nothing is read.
