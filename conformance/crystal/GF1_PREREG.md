# GF1 — the magic price of a gauge vacuum: PREREGISTRATION

> **CORRECTION, 2026-09-19, on building the instrument — two errors in this prereg, both the
> lead's, both found by the plants before any vacuum was read.** (1) **The cost is wrong by
> `χ⁴`.** §2 prices the Pauli-replica contraction at `O(N · 4 · χ⁴)` and calls `χ ≤ 64`
> feasible. `4χ⁴` per site is the SIZE of the Pauli MPS; its norm is `Σ_P ⟨P⟩² = 2^N`, the
> purity, identically. The fourth powers need four replicas: `χ⁸` memory, `64 χ⁹` time per
> site. Built exactly, the instrument admits `χ ≤ 11` under an 8 GiB lease and refuses
> above by name; the staked ladder at `χ = 40, 64` cannot be read exactly. (2) **`M₂^loc` as
> staked is identically `M₂`.** The stabilizer Rényi entropy is invariant under the whole
> Clifford group, including products of single-site Cliffords — each site's 24 candidates
> permute the same four channel weights, and the fourth power kills the signs. So S3, the
> stake this prereg called "Fold III's own sentence made falsifiable," is as instrumented
> the stake §0 itself says known physics falsifies. The non-local magic that prices a
> stabilizer-rank simulation is the minimum over CONTINUOUS local unitaries, not local
> Cliffords, and needs a different minimiser. (3) No variance-gate threshold existed in the
> repo; plant P6 declared `≤ 10⁻³` per site and passed at `|ΔM₂| = 6.9 × 10⁻⁴` across χ. The
> repair is `GF1_AMENDMENT_1.md`; this document is kept as written beneath this note. The
> lesson is the same one as `LIQUID2_AMENDMENT_2.md`'s: a scaling remembered rather than
> derived, and a quantity assumed to vary that a theorem holds fixed.

*Written 2026-09-19, committed alone, BEFORE the instrument is built and before any state is
read under it. Discharges "GF1 — OPEN · prereg owed" (`GANTT.md`, `OBJECT.md` Fold III). The
form is rung 2's: what is staked, what would kill it, the plants that must fire, the branches
named before the numbers, and the cost priced before the kernel.*

## 0. What Fold III claims, and the sentence in it that can be false

`OBJECT.md`: *"Both confined fixed points are stabilizer states (`vacuumConfig`, the toric
code), so the interacting vacuum in a hadron-sized box is low-magic and area-law, priced
exactly by the strata this engine owns."* Kill as written: *"if the log-price grows with
volume, Fold III dies."*

**Stated honestly before the run: the naive reading of that kill would fire on known physics.**
The stabilizer Rényi entropy of a generic gapped ground state is EXTENSIVE — the magic density
is a finite constant away from the stabilizer points (Haug & Piroli, PRB 2023; Lami &
Collura, PRL 2023, on the Ising chain and others). Total magic growing linearly with `N` is
not a discovery about gauge vacua; it is what any non-stabilizer product structure does, and a
prereg that staked Fold III on it would be staking it on a coin already flipped. What Fold
III needs is narrower and is the thing the strata's PRICE actually depends on:

1. the magic DENSITY `c(x) = lim M₂/N` is small in the confined regime and vanishes at the
   strong-coupling fixed point, and
2. the NON-LOCAL magic — what is left after the best per-site Clifford frame is removed,
   the part entanglement carries and no local rotation can absorb — is BOUNDED in `N`
   (area-law in 1+1D means constant), because that is the part that prices a stabilizer-rank
   simulation beyond a product-state prefactor.

Both are falsifiable, neither is settled by the literature for a lattice gauge vacuum, and
the second is the one Fold III's "priced exactly by the strata" rests on.

## 1. The state

The Schwinger-model vacuum on staggered fermions after Jordan–Wigner — qubits, local
dimension 2, Paulis natural — as `q8-mps::schwinger` produces it at the banked couplings
`x ∈ {0.25, 1, 4, 16}` (SCHWINGER-3's ladder; `x = 16` is the continuum-most point banked)
and volumes `N ∈ {8, 12, 16, 24, 32, 48}` at the χ the SCHWINGER-3 gate admitted, with the
energy variance gate of `variance.rs` required to pass at every point before magic is read.
A state that fails its own variance gate is not a vacuum and is REFUSED, not measured.

The strong-coupling fixed point `x → 0` is `vacuumConfig` (`Vacuum.lean`): every link in
the trivial representation, electric energy zero, Gauss's law exactly zero — a stabilizer
state, `M₂ = 0` by theorem. It is control C1.

## 2. The instrument, named and not yet built

**M₂, the stabilizer Rényi entropy** (Leone, Oliviero, Hamma 2022):
`M₂ = −log₂ ( Σ_P ⟨P⟩⁴ / 2^N )` over all `4^N` Pauli strings. Computed EXACTLY on the MPS
by the Pauli-replica contraction (the "Pauli MPS" of bond dimension `χ²`, whose norm is the
sum of fourth powers) — `O(N · 4 · χ⁴)` per state, feasible at `χ ≤ 64`. Cross-checked at
`N ≤ 10` against brute enumeration of all `4^N` expectations through
`observables::expectation`, which exists.

**M₂^loc, the non-local magic:** `M₂` minimised over products of single-site Clifford
unitaries (24 per site, the group is finite; coordinate descent from the identity frame,
three sweeps, the minimum over sweeps). Reported beside `M₂`, never instead of it.

**The price:** for a box of `L` sites the stabilizer fidelity obeys `F ≤ 2^{−M₂(L)/2}`, so
the stabilizer extent — the magic tier's cost — is at least `2^{M₂(L)}` and the log-price
is `M₂(L)` up to a constant. Reported for `L = 10` (a hadron-sized box on this lattice) at
each coupling, in the same units the magic tier prices T-gates.

Both quantities carry a declared numerical floor: `Σ⟨P⟩⁴` accumulated in `f64` over `4^N`
terms has relative error `≲ 4^N · ε`; at `N = 32` that is `2 × 10^{−3}` on the sum, `3 × 10^{−3}`
on `M₂`. Any difference under `0.01` in `M₂` is reported as "within the floor".

## 3. The stakes, each with its kill

- **S1 — the density.** At each `x`, `M₂/N` converges: the successive differences
  `M₂(N) − M₂(N−Δ)` over `Δ` are constant to `10 %` from `N = 16` up. The limit `c(x)` is
  reported. **Kill:** none — this is the measurement that names `c(x)`; it cannot fail, only
  refuse (a non-converging density means the volumes are too small, and that is reported).
- **S2 — the density at the fixed point.** `c(x) → 0` as `x → 0`: at `x = 0.25`,
  `M₂/N < 0.05`. **Kill:** `M₂/N ≥ 0.05` at `x = 0.25`, which would say the confined vacuum
  is not close to its stabilizer fixed point even at strong coupling.
- **S3 — Fold III's own sentence, made falsifiable.** At each `x`, the non-local magic
  `M₂^loc(N)` is BOUNDED: over `N ∈ {16, 24, 32, 48}` its least-squares slope in `N` is
  under `0.01` per site (an order below any plausible extensive density). **Kill:** a slope
  `≥ 0.01` per site at any coupling — the non-local magic grows with volume, the price is
  not area-law, and **Fold III dies as written.** Separable: it kills the fold and touches
  neither GF0 (read) nor the object's proved core.
- **S4 — the price of a hadron.** `M₂(L = 10)` at every coupling is under `10` — a
  stabilizer extent under `2^{10}`, which the magic tier prices in seconds. **Kill:** a
  coupling at which a ten-site box costs more than `2^{10}`; Fold III's "priced exactly" then
  reads "priced, and here is the price" and the fold's claim weakens to that.

## 4. The plants, each of which must fire before a vacuum is read

| plant | carrier | must |
|---|---|---|
| **P1** | a product of `N` `|T⟩` states as an MPS (`χ = 1`) | `M₂ = N · log₂(4/3) = 0.41504 N` to the floor — additivity and the constant, both |
| **P2** | `vacuumConfig` and a GHZ state | `M₂ = 0` to the floor — a stabilizer state reads zero |
| **P3** | P1 with a random single-site Clifford applied per site | `M₂` unchanged to the floor (Clifford invariance), and `M₂^loc = 0` — the local minimiser finds the frame |
| **P4** | a random MPS at `χ = 8`, `N = 10` | the Pauli-replica `M₂` equals brute enumeration over `4^{10}` to `10^{−9}` |
| **P5** | a random MPS at `N = 24` | `M₂^loc ≤ M₂`, and the three-sweep minimum is not improved by a fourth sweep by more than the floor |
| **P6** | a Schwinger vacuum at `x = 4`, `N = 16`, two χ values that both pass the variance gate | `M₂` agrees between them to `0.05` — the reading is the state's, not the truncation's |

## 5. The branches

- **(a)** S2, S3, S4 all met → Fold III's magic clause is MEASURED at the couplings and
  volumes run; `c(x)` and `M₂^loc` are banked; GF2's price has a number.
- **(b)** S3 fails at any coupling → Fold III dies as written, kept and marked dead; the
  fold's other two legs stand on GF0 and GF2 alone. The record says at which coupling and
  with what slope.
- **(c)** S2 fails → the strong-coupling vacuum is not near its fixed point at `x = 0.25`;
  the ladder is extended to `x = 0.0625` before any verdict, and reported.
- **(d)** S1 refuses (density not converged by `N = 48`) → VOID, the volumes are extended
  once (to `N = 64, 96`) at the cost stated below; a second refusal is the finding.
- **(e)** any plant fails to fire → the instrument is not admitted and nothing is read.

## 6. The cost, before the kernel

Producing the vacua: SCHWINGER-3's own ladder, re-run at the six volumes and four couplings
— its `N = 144, χ = 64` point took `22` hours; the largest here is `N = 48` at the same χ,
`~2` hours, `24` points, **under 20 core-hours** on the DMRG lane. The Pauli-replica
contraction at `χ = 64`: `χ⁴ = 1.7 × 10^7` per site per Pauli, `4 N` Paulis, `N = 48` → `~10^{10}`
flops per state, seconds. The local-Clifford minimiser: `24 × N` evaluations per sweep, three
sweeps, `~10^{13}` flops at `N = 48` — minutes to an hour per state on one core. **Total under
40 core-hours.** Building the instrument — the Pauli-replica contraction and the minimiser
in `q8-mps`, with P1–P6 as its tests — is one to two days of engineering and is the item
this prereg makes the next step.

## 7. What this does not test

The toric-code fixed point (2+1D; no carrier here), the QCD₂ colour lanes (local dimension
above 2 needs generalised Paulis — Howard et al.'s odd-prime mana — and a second prereg),
and whether the magic tier's cost is the RIGHT price for a hadron box (that is GF2's
question). It tests one sentence of Fold III at the strength the sentence is written, and
says in advance which reading of that sentence is already known to be false.
