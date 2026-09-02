# Pre-registration — SCHWINGER-4: the residual interaction between two screened pairs decays at the banked meson mass

*Frozen 2026-09-02, committed ALONE, before the instrument extension exists.
This is node GF0 of the fold below the atom (`OBJECT.md` "The fold below the
atom", LOCKED the same day): Fold II says the mass gap makes the far field
free, so the closure defect between two colour-singlet objects is DERIVED
from the gap rather than fitted. The cheapest place to measure that is the
one gauge theory this engine has already solved to the continuum, and the
referee is our own banked number: SCHWINGER-3's per-column vector-meson
mass. Nothing here is a three-dimensional claim; branch (a) licenses one
measurement of Fold II in one space dimension and nothing more.*

misfits: contacts M-VOLUME-SCALE (the lattice columns are SCHWINGER-3's own
(x, N) pairs, which discharged it: N = ceil(28√x) rounded even, and the
two-pair span never exceeds N/3 so the pairs sit inside the bulk), M-NULL-MISSTAKE
(the gate is staked on the DECAY RATE, the quantity the gap controls, never
on the prefactor, which the screening cloud sets), M-ONE-MODEL-DELTA (the
verdict compares the fitted rate to a BANKED measurement, never to a fitted
model), M-STALE-INSTRUMENT (the extension is a NEW file beside the mirrored
SCHWINGER-3 driver, whose bytes and sha256 provenance stay untouched; the
runner, this prereg and the results document are committed together with the
extension's own hash), M-PLANT-OBS and M-PLANT-SECTOR (two plants below, each
with its carrier asserted nonzero in the sector the plant acts on),
M-BARE-CHARGE (the external charges are STATIC sources inserted through the
Gauss law's background field, i.e. as the shift of every link's electric flux
they sit to the left of — the dressed form, never a bare pair), M-HOMOG (the
two-pair configuration is spatially resolved by construction: the observable
IS a function of separation), M-PROVENANCE-OVERREACH (the results document's
launch header carries the extension's sha256 beside the build it ran under),
M-IDLE-CALIBRATED-TIMEOUT and M-PLACEMENT-LOTTERY (no wall-clock figure is a
gate; per-point checkpoints make the run resumable and the record prints
loadavg beside every point), M-CHEAPER-THAN-ITS-PRICE (the cost model is
stated: one DMRG ground state per point, no excited state, so the run is
cheaper than SCHWINGER-3 per point by construction), M-MAINTENANCE-LENS (no
repair claim; the word "rent" below names Fold I's reading and is not a
maintenance measurement). Not contacted: M-GAUGE-LAUNDER, M-LOOP-BLIND,
M-COND-PROBE, M-ELECTRIC-BASIS, M-RING-MIXING, M-GAUGE-UNIFORM-MOMENTUM,
M-KINEMATIC-NONLOCAL, M-FIXED-POINT-TRAJECTORY, M-NONBIJECTIVE-STEP,
M-FINAL-VIEW-COLLISIONS, M-PROBE-EIGENSTATE, M-DEVICE-CLASS (CPU only, one
device class, stated), M-PARITY-PROTECT (no parity claim).

## Model

The massless Schwinger model in the Hamer–Kogut spin form on an open staggered
chain, exactly SCHWINGER-1/2/3's Hamiltonian (W-units, `x = 1/(g a)²`):

```
W = x Σ_n (σ⁺_n σ⁻_{n+1} + h.c.) + Σ_{n<N−1} (L_n + ε_n)²
L_n = ½ Σ_{k≤n} (σ^z_k + (−1)^k)          (the dynamical flux)
ε_n = Σ_{k≤n} Q^ext_k                      (the STATIC background flux)
```

with `Q^ext` a configuration of static unit charges. Expanding the square, the
static term is a SITE-DIAGONAL potential `Σ_k c_k q_k` with
`c_k = 2 Σ_{n=k}^{N−2} ε_n`, plus a constant: the extension is one additional
diagonal channel entry in the existing bond-dimension-6 MPO and nothing else.
The total-charge penalty `λ Q²` of the mirrored driver is retained unchanged
(the dynamical sector stays `Q = 0`).

Configurations, each a separate DMRG ground state:

| label | static charges | what it is |
|---|---|---|
| `E0` | none | the vacuum |
| `E1(p)` | `+1` at site `p`, `−1` at site `p+s` | one screened pair of size `s = 2` |
| `E2(d)` | pair at `(p, p+s)` and pair at `(p+s+d, p+2s+d)` | two pairs at separation `d` |

The observable is the four-point combination that cancels every self-energy
and every boundary term at the same positions:

```
V(d) = E2(d) − E1(p) − E1(p+s+d) + E0
```

Both single-pair energies are measured at the positions they occupy in the
two-pair configuration; the first pair's position `p` is chosen so the two-pair
span is centred on the chain.

## Grid

| x | N (SCHWINGER-3's k = 28 column) | s | d | χ |
|---|---|---|---|---|
| 9.0 | 84 | 2 | 2, 3, 4, 6, 8, 10, 12, 14, 16, 20, 24 | 40; 64 at d ∈ {8, 12, 24} |
| 4.0 | 56 | 2 | 2, 3, 4, 6, 8, 10, 12, 14, 16 | 40; 64 at d ∈ {6, 10, 16} |

Checkpointed per configuration, resumable; the driver, warm-start ladder and
sweep schedule are the mirrored SCHWINGER-3 driver's, copied verbatim into the
extension with the MPO the one difference (the copy is hashed against the
mirror by the extension's own gauge, see plant (iii)).

## The referee: our own banked mass

`κ_pred(x, N) = M(x, N, χ = 64) / √x` per site, with `M` read from the
SCHWINGER-3 checkpoints committed at `conformance/crystal/` (values, so the
freeze cannot drift: `M(9, 84) = 0.646645`, `M(4, 56) = 0.681667`; hence
`κ_pred(9, 84) = 0.215548`, `κ_pred(4, 56) = 0.340834` per site). The
physical content: in the massless Schwinger model the vector meson is the
LIGHTEST state, so the exchange between two screened pairs is carried by it
and the residual interaction decays as `e^{−M d}`.

## Frozen premises and gates

- **χ-premise, per checked point** (EXACT band): `|V(χ=64) − V(χ=40)| ≤
  max(1e-4, 0.05 · |V(χ=40)|)`, else that point VOIDs; a column with any
  VOID point among its three checks VOIDs. witness: none (measured premise;
  the vacuum and single-pair energies enter every `V(d)` and are checked at
  χ = 64 once per column)
- **Screening premise, per column**: the single pair is SCREENED — `E1 − E0`
  saturates: `|(E1(s=3) − E0) − (E1(s=2) − E0)| < 0.5 · (E1(s=2) − E0)` at
  the column's `(x, N)`, measured once. Else the "pair" is a string, not a
  screened object, and the column VOIDs (the fold's object was not
  constructed). witness: none (measured premise)
- **Noise floor, stated**: a point with `|V(d)| < 10 · 1e-4 = 1e-3` is
  EXCLUDED from the fit (it is inside ten times the χ-band's absolute floor);
  a column with fewer than 5 fit points in the window VOIDs. witness: none
- **Fit window**: `d ≥ 8` on both columns (beyond two screening lengths at
  x = 9 and nearly three at x = 4), through the largest `d` above the floor.
  Ordinary least squares of `ln|V(d)|` on `d`; slope `−κ_fit`; `R²` reported.
- **G1 — the physics gate, two-sided, per column**: with `R² ≥ 0.99`,
  **branch (a)** iff `κ_fit ∈ [0.8, 1.2] · κ_pred`; **branch (b)** iff
  `κ_fit < 0.8 · κ_pred`; **branch (c)** iff `κ_fit > 1.2 · κ_pred`;
  `R² < 0.99` ⇒ that column VOIDs (the decay is not exponential inside the
  window, which is itself reported with the residuals). The campaign's verdict
  is (a) only if BOTH columns read (a); one (b) anywhere is (b); otherwise (c)
  or VOID as the columns read. Never a pass by shrinkage. witness:
  `closure_determines_dynamics` (the law being measured is the forced one;
  the theorem covers the derivation, not the number — the number's referee is
  SCHWINGER-3's checkpoint)
- **G2 — the sign reading** (recorded, not a gate): flipping the second pair's
  orientation must flip the sign of `V(d)` at every `d ≥ 8` while leaving
  `κ_fit` inside `±0.05 · κ_pred` of the unflipped fit at x = 9; reported
  as a check that the residual interaction is the dipole-like exchange and
  not a boundary artefact. Digit-bearing: 0.05. witness: none

## Plants (M-PLANT-OBS, M-PLANT-SECTOR discharged here)

- **(i) The exact referee at small N.** At `N = 12, x = 4`, exact
  diagonalisation with the same static charges (the SCHWINGER-1 referee's
  `build()` extended by the same site-diagonal term, mirrored into the
  instrument directory with its provenance) must agree with the extension's
  DMRG: `|E_DMRG − E_ED| ≤ 1e-6` for `E0`, `E1(p=4)` and `E2(d=2)`.
  Carrier asserted nonzero in the sector the plant acts on: `E1 − E0 > 0.1`
  (the static pair costs energy; a plant on a zero would be vacuous).
- **(ii) The gate must FIRE on a planted defect.** MPO mutation
  `coulomb-off`: the dynamical Coulomb channel is zeroed (the model becomes
  free staggered fermions in the static site potential), everything else
  unchanged. Run the x = 4 column under the mutation. The residual
  interaction of free fermions is a power law with Friedel oscillations, so
  G1 must NOT return branch (a): either `R² < 0.99` or `κ_fit` outside the
  band. Carrier asserted nonzero in the sector: `|V(2)| > 1e-3` under the
  mutation (the observable is live where the plant acts).
- **(iii) The driver is the banked one.** The extension's copied DMRG sweep
  must hash-equal the mirrored SCHWINGER-3 driver's `dmrg()` source with the
  single MPO-construction line substituted; the extension refuses to run if
  it does not (M-STALE-INSTRUMENT in its constructive form). EXACT.

## Meaning

(a) ⇒ Fold II has its first measurement below the atom: the residual
interaction between two screened, gauge-invariant objects decays at the gap,
so the hadron tier's far field is free at this scope (one space dimension,
U(1), the vector meson the lightest exchange). It licenses GF2's use of an
exponentially convergent expansion over hadrons in the 1+1D toy and nothing in
three dimensions. (b) ⇒ Fold II dies at first scope and is kept dead; the
hadron ladder is re-examined before any three-dimensional cycle is spent.
(c) ⇒ the identification of the exchanged state with the vector meson dies,
locality survives, and the heavier rate is reported as the measured exchange.
VOID ⇒ the screening premise, the χ-band or the fit window was wrong, which
the per-point record localises.

## Cost model, stated

One DMRG ground state per configuration, no excited state and no penalty
projection: 15 configurations at x = 9 (`E0`, two `E1` per `d` where positions
differ, `E2` per `d`) plus 13 at x = 4, plus 6 χ = 64 checks and the two
screening-premise points — under 40 ground states, each cheaper than a
SCHWINGER-3 point at the same `(x, N, χ)`. Run detached with per-point
checkpoints and a DONE marker; the record prints loadavg beside every point.

## Amendment A1 — 2026-09-02, PRE-DATA, instrument-schedule-only

*Filed before any staked configuration had completed on either driver (zero
checkpoints in `conformance/crystal/` at filing). No gate, band, grid, referee
or meaning clause above changes. What changes is WHICH DRIVER may compute a
staked point.*

The operator's standing order is a single engine, DRY. The Python driver above is
the banked SCHWINGER-3 referee, single-process, and its first N = 56 ground state
had not finished after thirty minutes; the engine's own DMRG (`q8-mps`, zero
dependencies, general-MPO two-site sweep with Lanczos residual gate 1e-10 and a
fixed Néel start, no RNG) carries the SAME six-channel tensor (`q8-mps/src/schwinger.rs`)
and fans one process per configuration over the machine. So:

- **The engine driver is ADMITTED for staked points provided** it passes plant (i)
  on itself — the three N = 12 referees at the 1e-6 bar (`schwinger4 --gauge`,
  and `tests/schwinger_gauge.rs`, which also grades its MPO against an
  independent dense Hamiltonian spectrum to spectrum) — AND three cross-check
  points per column (`E0` and two `E2(d)` at χ = 40) agree with the Python
  driver within the χ-premise band `max(1e-4, 0.05·|V|)` on the energies'
  contribution to `V`. A cross-check that misses VOIDs the engine arm for that
  column; it does not touch the Python arm.
- **Plant (iii) for the engine arm** reads: the results document carries the
  `schwinger4` binary's sha256, the crate commit, and the rustc version; the
  Python arm keeps its own plant (iii) unchanged.
- **The two arms are both reported.** Whichever arm completes a column first is the arm
  the gates are read on, the other is the cross-check; if both complete, both
  fits are printed and G1 is read on the engine arm with the Python arm's
  `κ_fit` beside it. The `E1` energies are per POSITION on both arms.
- Checkpoints of the engine arm are `ckpt4_rs_*.npz`; its column outputs
  `schwinger4_rs_x<x>.json`; `analyze rs` reads them.

Digit-bearing: 1e-6, 1e-4, 0.05, 3. witness: none (an amendment about which
implementation of one tensor runs; the theorem content is unchanged).
