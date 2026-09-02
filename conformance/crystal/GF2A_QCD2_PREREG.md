# Pre-registration — GF2a: a baryon out of the SU(3) vacuum, on the solver we already have

*Frozen 2026-09-02, committed ALONE, before the integral builder exists. The rung
UNDER GF2 of the fold below the atom (`OBJECT.md`): in one space dimension with open
boundaries the axial gauge eliminates the SU(3) gauge field exactly, as it did the
U(1) field for the Schwinger model, and what remains is a fermion Hamiltonian with a
non-local colour-Coulomb term — an electronic-structure problem in (site × colour)
orbitals with a closed-form two-body tensor. This engine's exact determinant solver
and its DMRG take exactly that object, so a hadron of a NON-ABELIAN gauge theory can
be derived here today with no fitted constant. The physics is not new (credits below);
what is on trial is Fold I's "forced, not fitted" where this stack can pay for it.*

misfits: contacts M-VOLUME-SCALE (the chain lengths are staked per coupling on the
SCHWINGER-3 standard N ≳ 20√x and the mass is extrapolated in 1/N with residuals
printed; a lattice claim without that discharge is the defect the standard names),
M-NULL-MISSTAKE (the gates are staked on sector ENERGY DIFFERENCES, the quantities a
mass gap controls, never on absolute energies), M-ONE-MODEL-DELTA (no external number
at N = 3 is fitted or assumed: the referees are two independent solvers on one tensor
and a dense diagonalisation, and every extrapolation prints its residual),
M-STALE-INSTRUMENT (the builder, this freeze and the results document are committed
together with the binary's hash), M-PLANT-OBS and M-PLANT-SECTOR (three plants below,
each with its carrier asserted nonzero in the sector the plant acts on),
M-BARE-CHARGE (every sector is a fixed total quark number, and the Coulomb term makes
any coloured total state cost energy growing with N; the baryon's colour-singlet
nature is MEASURED as convergence with N, never assumed), M-HOMOG (the chain is
homogeneous by construction; no locality claim is staked), M-CHEAPER-THAN-ITS-PRICE
(the cost model is stated: determinant counts per sector are printed by the same
arithmetic the periodic table uses), M-DEVICE-CLASS (CPU only, one device class),
M-PROVENANCE-OVERREACH (the results document carries the binary's sha256 beside the
crate commit and rustc), M-MAINTENANCE-LENS (no repair claim), M-PARITY-PROTECT (no
parity claim). Not contacted: M-GAUGE-LAUNDER, M-LOOP-BLIND, M-COND-PROBE,
M-ELECTRIC-BASIS, M-RING-MIXING, M-GAUGE-UNIFORM-MOMENTUM, M-KINEMATIC-NONLOCAL,
M-FIXED-POINT-TRAJECTORY, M-NONBIJECTIVE-STEP, M-FINAL-VIEW-COLLISIONS,
M-PROBE-EIGENSTATE, M-IDLE-CALIBRATED-TIMEOUT, M-PLACEMENT-LOTTERY.

## Model

Massless one-flavour QCD in 1+1 dimensions, staggered fermions on an open chain of
`N` sites (`N` even), axial gauge, in the Hamer–Kogut W-units of the Schwinger
instrument (`x = 1/(g a)²`):

```
W = x Σ_{n} Σ_c (ψ†_{n,c} ψ_{n+1,c} + h.c.)  +  Σ_{n<N−1} Σ_a ( Σ_{k≤n} q^a_k )²
q^a_k = Σ_{c,c'} ψ†_{k,c} T^a_{cc'} ψ_{k,c'}          (T^a = λ^a/2, a = 1..8)
```

Credits: Banks–Kogut–Susskind and Hamer et al. for the staggered spin form; the
axial-gauge 1+1D QCD Hamiltonian as used for quantum simulation by Atas, Haase,
Zhang, Kühn, Muschik and others (2023) and by Farrell, Chernyshev, Powell, Zemlevskiy,
Illa and Savage (2023); 't Hooft (1974) for the large-N meson spectrum, cited as
context and NOT staked at N = 3.

**The reduction to electronic-structure form, which is the whole of the instrument.**
With `Σ_a T^a_{cc'} T^a_{dd'} = ½ δ_{cd'} δ_{c'd} − ⅙ δ_{cc'} δ_{dd'}` (the SU(3) Fierz
identity) and `Σ_{n<N−1} Σ_{k,k'≤n} = Σ_{k,k'} (N−1−max(k,k'))`, the Coulomb term is
`Σ_{k,k'} w_{kk'} Σ_{cc'dd'} F_{cc'dd'} E_{(k,c)(k,c')} E_{(k',d)(k',d')}` with
`w_{kk'} = N−1−max(k,k')`, `F = ½ δ_{cd'} δ_{c'd} − ⅙ δ_{cc'} δ_{dd'}`, and
`E_pq = ψ†_p ψ_q`. In the solver's convention
`H = Σ h_pq E_pq + ½ Σ (pq|rs) (E_pq E_rs − δ_qr E_ps)` this is EXACTLY
`(pq|rs) = 2 w_{kk'} F_{cc'dd'}` for `p = (k,c), q = (k,c'), r = (k',d), s = (k',d')`
(zero unless `p, q` share a site and `r, s` share a site) and the one-body correction
`h_{(k,c),(k,c)} = +(4/3)(N−1−k)` for `k ≤ N−2` — the Casimir `C_F = 4/3` of one quark per
link it sits to the left of — plus the hopping `h_{(k,c),(k+1,c)} = x`. No other term.

Sectors: `n_q` quarks in `3N` orbitals; the Dirac sea is half filling `n_q = 3N/2`,
baryon number `B = (n_q − 3N/2)/3`. One string carries every quark (no spin, one
flavour); the second string is empty.

## Grid

| x | N (FCI, exact) | N (DMRG, q8-mps, χ = 40 / 64) | sectors |
|---|---|---|---|
| 4.0 | 4, 6, 8 | 8, 16, 24, 40 | B = 0, 1, 2 |
| 9.0 | 4, 6, 8 | 8, 24, 40, 60 | B = 0, 1, 2 |

FCI determinant counts (printed by the run): N = 8 is 2,704,156 (B = 0), 1,307,504
(B = 1), 134,596 (B = 2) — inside the determinant working set this machine's door
admits at these sizes; N = 10 (155 million at B = 0) is DMRG's.

## Observables

- `M_B(x, N) = [E₀(B=1) − E₀(B=0)] / (2√x)` — the baryon mass in units of `g`, the
  same normalisation as the Schwinger meson.
- `U_BB(x, N) = E₀(B=2) − 2 E₀(B=1) + E₀(B=0)` — the two-baryon interaction energy in
  the finite volume (the pair is confined to the chain; its separation is the volume).

## Frozen gates

- **G0 — two solvers, one tensor** (EXACT): at every FCI-and-DMRG point (N = 8, both x,
  all three sectors) `|E₀(FCI) − E₀(DMRG, χ=64)| ≤ 1e-6`, and the dense gauge of plant
  (i) passes. Else the instrument is wrong and nothing is read. witness: none (a
  cross-solver identity; no Lean object covers the number)
- **G1 — the baryon is a screened object, not a string**: `M_B(x, N)` CONVERGES with
  `N` — for each x, `|M_B(N₄) − M_B(N₃)| < 0.5 · |M_B(N₃) − M_B(N₂)|` over the three
  largest DMRG N, and the extrapolated `M_B(x, ∞)` is finite and positive, fitted in
  `1/N` with the residual printed. A mass growing with N (the colour of three quarks
  NOT screened into a singlet, so the state is a string across the chain) kills the
  reading that the B = 1 ground state is a hadron. witness: `closure_determines_dynamics`
  (Fold I's "the coarse law is forced": the hadron's mass is derived, and the theorem
  covers the derivation, not the number)
- **G2 — the two-baryon interaction is SHORT-RANGED**: `|U_BB(x, N)|` DECREASES with N
  over the DMRG ladder and its ratio between successive N is at most 0.75. A
  finite-volume energy that does not fall with the volume means the two baryons are
  not separable objects. Recorded beside it, not gated: the SIGN of `U_BB` at the
  largest N (attractive or repulsive). Digit-bearing: 0.75. witness: none
- **G3 — the continuum reads the same shape**: `M_B(x=4, ∞)` and `M_B(x=9, ∞)` agree
  within 15% after the `1/√x` correction the Schwinger ladder used (two columns give
  a line, not an extrapolation; the third column is the successor's). A larger spread
  is reported, not a kill: it prices the successor's grid. Digit-bearing: 15.
  witness: none

## Plants (M-PLANT-OBS, M-PLANT-SECTOR)

- **(i) The dense referee at N = 4.** An INDEPENDENT dense Hamiltonian over the
  `2^12` Fock states built from the operator definition (no integral tensor
  anywhere), projected onto `n_q ∈ {6, 9}`, must agree with the FCI on the same
  sectors to `1e-9`. Carrier nonzero in the sector: `E₀(B=1) − E₀(B=0) > 0.1` at
  N = 4, x = 4 (a baryon costs energy; a zero would be vacuous).
- **(ii) The gate must FIRE on a planted defect.** Mutate the Fierz coefficient
  `−⅙ → 0` (drops the trace subtraction, so the interaction is no longer that of a
  traceless generator): the dense referee and the mutated tensor must DISAGREE at N = 4
  by more than `1e-3` in `E₀(B=1) − E₀(B=0)`. Carrier: the same nonzero difference.
- **(iii) The colour is where it says.** The one-body Casimir term alone (`x = 0`,
  hopping off) must give, at N = 4, `B = 1`, EXACTLY the energy of three quarks on one
  site forming a singlet: the Coulomb term's expectation in that state is zero, so
  `E₀(B=1, x=0) = E₀(B=0, x=0)` to `1e-12`. A nonzero value means the tensor's
  Fierz or normal-ordering is wrong. EXACT.

## Meaning

G0–G2 pass ⇒ **a baryon is derived from the SU(3) vacuum on this engine**: a finite,
volume-converged colour-singlet three-quark bound state, and a two-baryon interaction
that falls with separation — Fold I's "forced, not fitted" measured in the dimension
this stack can pay for, with no external number in the loop. The derived `U_BB` is
the seed of the NN table the fold promised; its separation dependence is GF2b's
(static colour sources, SCHWINGER-4's design with colour). G1 fails ⇒ the B = 1 state
is a string at these couplings and the reading dies at this scope, kept. G2 fails ⇒
the baryons are not separable at these volumes; the volume standard is re-examined.
None of this is a statement about three dimensions.

## Cost model, stated

FCI: 9 sector solves per x at N ≤ 8 on the determinant route (the largest 2.7 million
determinants, the door's price printed). DMRG: 3 sectors × 4 N × 2 χ per x through
`q8-mps::Mpo::from_electronic_integrals`, one process per point.
