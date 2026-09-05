# Pre-registration — FIELD-4: the seam served channel by channel — the density field's penetration and induction on the hydrogen-bond contact, the wall over the density field, dispersion from what remains, a placement decided by the flipped dimer, and the hydrogen bond re-asked

*Frozen 2026-09-05, committed ALONE, before the runners existed. Built by the lead with a
delegate on the harvest. FIELD-3 landed the closure identity and the seam rule and read its
harvest (c): the exact dimer's residual over the point-charge field is a wall inside 2.8 Å
and a MISSING ATTRACTION beyond it — `−1.35`, `−1.49`, `−0.84`, `−0.36` mHa at 2.9, 3.1, 3.4,
3.7 Å, a third to a half of the field — decaying faster than any power. No wall could be
harvested over a two-node prefix and the arms did not run. This freeze does what rule 10 of
OBJECT.md says the ledger does with a residual: transfer it into the channel whose shape it
has, one channel at a time, measuring what each leaves. The instruments exist: EMBED-2's
Coulomb-only frozen-density embedding (`holon_chem::density_embed`, the fixed point of two
monomers each solved in the other's frozen density — electrostatics WITH penetration and the
induction of each fragment, WITHOUT exchange between fragments and without inter-fragment
correlation), FIELD-3's six exact linear dimers (`field3/linear_R*.json`, reused as the
referee — the same geometry, the same solver), and FIELD-3's engine (`seam.rs`).*

misfits: contacts **M-EMPTY-SECTOR** (the expectation rule has its EMPTY branch — a start
with fewer units than staked or a binding under `1e-4` in magnitude VOIDs its arms; a
harvest with a positive prefix under three VOIDs the arms before they run); **M-PLANT-OBS**
and **M-PLANT-SECTOR** (two plants, carriers asserted nonzero in the sector the plant acts on
— §5); **M-CHEAPER-THAN-ITS-PRICE** (the one new exact solve is priced by FIELD-3's record,
16,557–52,739 core-seconds per node; a node returning under a tenth of the floor is refused;
the density embedding's cost model is EMBED-3's, seconds per node, and is recorded);
**M-EXIT-DISCRIMINATOR** (the exact solve records its Davidson iteration count and exit; an
iteration cap is VOID; every density fixed point records its sweep count and convergence);
**M-STALE-INSTRUMENT** (this freeze alone; runners, JSON and results together);
**M-VACUOUS-SUCCESS** (every arm reports its frame count and the seam's running drop totals;
an arm whose totals are zero is VOID); **M-NULL-MISSTAKE** (retention on the rung-1 lens,
unchanged); **M-FIXED-POINT-TRAJECTORY** (the OFF arms are FIELD-2's, same seed, reproduced
bit for bit); **M-UNTESTED-GAP** (the terms are fit on one orientation; S2 is the one
measurement across that gap); **M-FORMAT-FLOOR** (FIELD-3's node records are read at their
printed precision, 12 significant digits on quantities of `1e-4` to `1e-2` hartree — eight
digits of headroom over every floor below); **M-FLOOR-UNSTAKED** (the reading floor on every
harvested residual is `1e-6` hartree; no ratio is read whose denominator is under it);
**M-BARE-CHARGE**, **M-HOMOG**, **M-COND-PROBE**, **M-DEVICE-CLASS** (as FIELD-3). Not contacted:
the rest of the registry.

## 0. What is built and measured

**C1 — the density field on the six geometries.** For each of FIELD-3's linear nodes
(`R_OO = 2.5, 2.7, 2.9, 3.1, 3.4, 3.7` Å): `ΔE_ρ(R) = E_A[ρ_B] + E_B[ρ_A] − E_es(A,B) − E_A0 −
E_B0` (EMBED-3's `de_rho`, the fixed point from `DensityStart::Zero`, sweeps and convergence
recorded), and the engine's point-charge field `E_q(R)` on the same geometry (FIELD-3's
`wall.json`, `e_field`). The PENETRATION-AND-INDUCTION residual:

    p(R) = ΔE_ρ(R) − E_q(R)

**C2 — the residual over the density field.** With `ΔE_exact(R)` from FIELD-3's records:

    r_ρ(R) = ΔE_exact(R) − ΔE_ρ(R)

which is exchange between the fragments plus their mutual correlation (dispersion at this
basis level), and nothing electrostatic.

**The engine's seam law, extended** (`SeamModel` gains three declared shapes, each an exact
`0.0` off, so FIELD-3's engine is the identity when they are): (1) the penetration term
`−P·exp(−c·r)` on every CROSS-UNIT hydrogen–oxygen pair — placed on the hydrogen-bond
contact because that is where the densities overlap; (2) the wall `A·exp(−b·r)` on cross-unit
O–O pairs (FIELD-3's); (3) dispersion `−C₆/r⁶` on cross-unit O–O pairs (channel 3's declared
rate). All three enter `Row::Seam`, which now carries `(Exchange, Whole)`, `(Field, Folded)`,
`(Induction, Folded)` and `(PairDispersion, Whole)` in the ledger's table; their forces are
analytic and their virials in the engine's convention.

**The harvest, in the ledger's order**, every coefficient transferred and none chosen:
- `(P, c)` from `p(R)`: for each `c` on a declared grid (`0.5` to `4.0` per bohr, step 0.01)
  the linear least-squares `P` over the six nodes of the engine's own sum over cross-unit
  H–O pairs, weights `1/ΔE_exact²`; the `c` of least weighted residual. Nodes with
  `|p| < 1e-6` are not readings.
- `(A, b)` from `r_ρ(R)`: FIELD-3's rule — the largest contiguous set of the SHORTEST nodes
  (at least three) with `r_ρ > 1e-6` on which the weighted log-linear fit lies within
  `0.10·|ΔE_exact|` at every node; `R_x` the last of them.
- `C₆` from the remainder beyond `R_x`: `r_ρ − wall`, one coefficient by weighted least
  squares on `−C₆/R_OO⁶`, its nodes' log-log slope reported.

## 1. The expectation, written before the arms (M-EMPTY-SECTOR discharged)

Per start (FIELD-2's dimer and ring), the extended seam law's binding `E(start) −
E(separated)` (units 40·k bohr apart) with its four cross-unit parts written separately; the
rule: fewer units than staked (2, 4) or `|binding| < 1e-4` ⇒ VOID; `≤ −2 kT` ⇒ expected to
hold; `> −kT` ⇒ expected to break; between ⇒ no expectation. `kT` = `9.28e-4` (293 K),
`4.75e-4` (150 K).

## 2. Gates

- **G-D0 — the identity.** With all three coefficients `0.0`, FIELD-3's seam gates run
  unchanged: checkpoint BYTES identical over 2,000 steps on FIELD-1's scene with the seam
  enabled-then-disabled before the first step (EXACT), and the channel receipt unchanged
  line for line.
  witness: none (bytes; the receipt)
- **G-D1 — the terms are the derivative.** Central differences at `h = 1e-4` bohr on every
  atom of the dimer start with all three terms on (the harvested coefficients, or declared
  test coefficients if the harvest voided, named in the output): `|F − (−∂E_seam)| / |F| ≤
  1e-8` on every component where `|F| > 1e-10`.
  witness: none (finite difference)
- **G-D2 — momentum and the books.** Dimer and ring at 293 K, 2,000 steps, all terms on:
  internal forces sum to `≤ 1e-12` of the largest; residual under its bound; receipt columns
  sum to `w_ext`; honest drift peak under a tenth of the largest posted transition.
  witness: none (conservation gates)
- **C1 — the density field binds at least as much as the charges.** `p(R) ≤ 0` at every node
  where it is a reading, and `|p|` non-increasing from its largest node outward (the
  overlap decays); the fixed points converged. 6 nodes.
  witness: none (a measured sign and order)
- **S1 — what the residual over the density field is.** **(a)** `r_ρ > 1e-6` at all six nodes
  and the wall fits all six within `0.10·|ΔE_exact|` ⇒ exchange is the whole remainder at this
  level; `C₆ = 0` is recorded and dispersion is absent at this basis. **(b)** a positive prefix
  of at least three fits and the remainder beyond `R_x` has a log-log slope in `[−8, −4]` ⇒
  the wall from the prefix and `C₆` from the remainder are both transferred (channel 3 at
  this basis level); a remainder outside that band is reported and NOT transferred (`C₆ = 0`),
  and named. **(c)** a positive prefix under three ⇒ VOID: no wall, the arms do not run, and
  the results document reads the shape of `r_ρ` for what FIELD-5 stakes.
  witness: none (fits against frozen tolerances)
- **G-C1 — the harvest is the engine's arithmetic.** With the harvested coefficients loaded,
  the engine's seam-law interaction on each linear node equals `E_q + p_HO + wall + disp`
  evaluated from the formulas to `1e-10` hartree, 6 nodes.
  witness: none (arithmetic)
- **S2 — the placement, decided forward (rule 6).** BEFORE the flipped node (the acceptor
  rotated by π about the x-axis through its oxygen, `R_OO = 3.4` Å, FIELD-3's geometry) is
  solved, `prediction.json` files TWO numbers from the linear harvest: the staked prediction
  with the penetration term on the cross-unit H–O contacts, and the named alternative with the
  same `p(R)` re-fit on the O–O distance instead. Then the exact solve (`1,002,001`
  determinants; `1,450 ≤ cpu_seconds ≤ 57,600`; residual `≤ 1e-9`; `Converged`). **(a)** the
  H–O placement within `max(0.25·|ΔE_exact|, 5e-4)` hartree ⇒ the penetration term lives on
  the contact; the seam law transfers across orientation at this level. **(b)** it misses and
  the O–O placement is within tolerance ⇒ the placement reading flips and the results say so
  (the arms run with the O–O placement). **(c)** both miss ⇒ the seam law is
  orientation-dependent beyond either placement; the miss is reported in hartree and as a
  fraction; the arms run with the staked placement and S3 is read with that caveat.
  witness: none (a prediction filed before its measurement)
- **S3 — retention under the extended seam law.** FIELD-3's arms, as frozen there: dimer and
  ring, 293 K and 150 K, OFF (FIELD-2's arm, reproduced EXACTLY) and SEAM (field, penetration,
  wall, dispersion), 2,000 settling and 20,000 counted frames, the rung-1 lens. **(a)** at 293 K
  `f_SEAM ≥ 0.5` on both ⇒ the seam law holds water's hydrogen bond at room temperature. **(b)**
  (a) fails at 293 K, holds at 150 K on both ⇒ the seam law binds and `kT` unbinds; the
  binding at the start says by how much. **(c)** fails at both ⇒ the results read the
  dynamics' diagnostic. 2 systems × 2 temperatures.
  witness: none (a measured population against a frozen instrument)

## 3. What each outcome means

S1 (a) or (b) with S2 (a) is the ledger closed at this basis level on the linear dimer and
transferred across orientation: every cross-unit constant derived, none fitted to the
liquid. S3 (a) is the payoff — water bonding to water for reasons the engine derived — and
opens the periodic liquid (EWALD-1 landed its electrostatics). S1 (c) says the exchange wall
is not separable from correlation at this basis, and FIELD-5 stakes a larger basis in the
seam programme's solver.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

Three terms fit on one orientation, one prediction across it, staked once. A seam law that
passes S2 and S3 is still a law fit at six points on one line, and the results say so.

## 5. Plants

- **(i) The sign of the penetration term.** `P → −P`. G-C1 must fail at the 2.9 Å node by
  `2·|p_HO(2.9 Å)|` to `1e-10`. Carrier: `|p_HO(2.9 Å)| ≥ 1e-4` hartree, asserted nonzero in the
  sector the plant acts on (the penetration term at the contact).
- **(ii) The reaction dropped on the new terms.** `F_j` not applied for the penetration and
  dispersion terms. G-D2's momentum gate must fail: the internal force sum exceeds `1e-6` of
  the largest force. Carrier: `|F_pen + F_disp| ≥ 1e-6` hartree/bohr at the dimer start,
  asserted nonzero in the sector the plant acts on (the new terms' forces).

## 6. Discipline

Runners: `holon-render/examples/field4_harvest.rs` (`density` — C1/C2 and the three fits,
`wall4.json`, `prediction.json` filed BEFORE the flipped node; `predict` — refuses without
`prediction.json`, then the flipped solve and `prediction_check.json`), the arms by
`field3_hbonds.rs` reading `wall4.json` (all four coefficients); gates in `tests/seam.rs`
(G-D0–G-D2, plant (ii)); JSON under `conformance/water_observatory/field4/`; results
`FIELD4_RESULTS.md` with the runners. The exact solve runs detached on the declared thread
count. No number enters from outside the engine and its own exact solver.
