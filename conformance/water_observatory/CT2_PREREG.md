# Pre-registration — CT-2: the angle in the transfer, and the contacts fit under the gate — the charge-transfer term given the bond's alignment on both sides and harvested on an orientation map of exact and closed-sector solves; the contact terms fit INSIDE the boundedness gate; a twisted, tilted and bent bond predicted forward; the first law to pass every gate runs the arms, and the liquid follows

*Frozen 2026-09-06, committed ALONE, before any new solve. Built by the lead (the engine's
angular term and its forces, the constrained fit's rule) with delegates on the harvest and
the arms. CT-1 measured charge transfer by itself (88 % of the hydrogen bond's remainder) and
read two things about carrying it: one exponential on cross-unit H–O distances over-counts
the twisted dimer by 60 %, because charge transfer has an angle and the term has none; and
the contact terms fit on what remains served the bond minimum for the first time by two
terms that were holes below their data, hidden by a gate that stopped at its first violation.
This freeze answers both. The transfer term gets the bond's alignment on both sides — the
donor's O–H pointing at the acceptor oxygen, and the acceptor's lone pair pointing back —
as a DECLARED family with its parameters on grids, harvested on an orientation map of
exact-minus-closed-sector readings that the map's own solver makes affordable (CT-1's
plant (i) solved the exact dimer from the undeformed product in 15 iterations where the
stock start took 139–222). And the contact terms are fit INSIDE the boundedness gate: for
every candidate exponent the amplitude is clamped at the largest value the gate admits, so
a law that fails G-B0 cannot be produced, and the price of that clamp is measured as a
residual ratio instead of discovered as a hole. If the law passes every gate, the arms run;
if the arms read (a), LIQUID-1's condition is met and the liquid runs on it.*

misfits: contacts **M-EMPTY-SECTOR** (the expectation rule keeps its EMPTY branch: a node
whose closure reading finds fewer than two units is not solved and is named; an arm whose
unit count leaves the staked value is VOID); **M-PLANT-OBS** and **M-PLANT-SECTOR** (two
plants, carriers asserted nonzero in the sector each acts on, §5); **M-CHEAPER-THAN-ITS-PRICE**
(every solve priced at CT-1's measured costs — exact from the product start `1,806` core-s at
15 iterations, sector `1,033–2,515` core-s at 7–14 iterations; a solve returning under a tenth
of its own iteration count times the 55 core-second floor is refused); **M-EXIT-DISCRIMINATOR**
(every solve records its exit and iteration count; a cap is VOID); **M-STALE-INSTRUMENT**
(every geometry of record is REBUILT and matched to its record's centers before its numbers
are used); **M-VACUOUS-SUCCESS** (every sector solve asserts its dimension and residual before
its energy is read; every arm reports its units on every pass; G-B0 is RUN on the constrained
law even though it passes by construction, because a gate that is assumed is not a gate);
**M-NULL-MISSTAKE**; **M-FIXED-POINT-TRAJECTORY**; **M-UNTESTED-GAP** (§4); **M-FORMAT-FLOOR**
(every record validates as JSON); **M-FLOOR-UNSTAKED** (the charge-transfer floor `1e-6`
hartree; the reading floor `1e-6`); **M-EXTRAPOLATED-HOLE** (the contact term is fit under the
boundedness gate; the attractive exponential of the transfer term is bounded by the wall's
because its exponent `c_CT ≤ 1.5` per bohr at CT-1's reading is softer than every wall's
and its amplitude is clamped by the same rule); **M-FIRST-VIOLATION-ONLY** (the boundedness
gate names every class and both legs; every fitted parameter at a grid EDGE is reported as
the grid's number beside the fit); **M-BARE-CHARGE**, **M-HOMOG**, **M-COND-PROBE**,
**M-DEVICE-CLASS**; **M-VOLUME-SCALE** (contacted by keyword: the parameter grids are grids of
the fit, not of space; every count is a count). Not contacted: the rest of the registry.

## 0. What is built and measured

**The solver for new exact nodes.** `holon-chem::heitler_london::fci_full_from_product` —
the Davidson on the full determinant space started from the undeformed Heitler–London product,
residual bar `1e-8`, cap 300 — is this campaign's exact solver for every NEW geometry. Its
gate is G-D0 below: on the thirteen exact nodes of record it must reproduce the stock
solver's energies to `1e-8`.

**The orientation map.** At every geometry below the exact solve and the block-localised
solve (`fci_block_localised`, CT-1's instrument as amended), so that `E_CT = E_exact −
E_noCT` is read per geometry. The builders are those of record (`field9_harvest.rs`,
`ct1_harvest.rs`: `linear`, `tilted(R, θ)` = the acceptor turned `θ` about the x-axis through
its own oxygen, `twisted(R, φ, θ)` = the acceptor turned `φ` about the O···O axis then `θ`
about the x-axis, `bent_donor(R, β)` = the donor turned `β` about the x-axis through its own
oxygen, `double_bent(R, β, θ)`); the monomer is EMBED-1's pin. A geometry at which the
engine's closure reading finds fewer than two units is OUTSIDE the identity, is NOT solved,
and is named in the record (FIELD-9's rule).

| family | geometries | count | new solves |
|---|---|---|---|
| of record | CT-1's twelve and its S2 node (`twistbent_R3.2`) | 13 | 0 |
| tilt | `tilted(R, θ)`, `R ∈ {2.7, 2.9, 3.1, 3.4}` Å, `θ ∈ {30, 60, 90, 120, 150, 180}°` | 24 | 22 (2.9 Å/30° is of record; 3.4 Å/180° IS `flipped_R3.4` of record, atom for atom — the acceptor turned by π about x through its own oxygen is the flipped acceptor) |
| twist | `twisted(R, 90°, θ)`, `R ∈ {2.7, 3.0, 3.4}`, `θ ∈ {0, 30, 60, 90, 120, 180}°` | 18 | 17 (3.0 Å/60° is of record) |
| donor | `bent_donor(R, β)`, `R ∈ {2.9, 3.1}`, `β ∈ {15, 45, 60, 90}°` | 8 | 8 |
| doubly varied | `double_bent(2.9, 15, 60)`, `double_bent(2.9, 45, 45)`, `double_bent(2.9, 30, 90)`, and `twisted(3.0, 90, 30)` with the donor bent 20° | 4 | 4 |
| **total** | | **64 distinct** | **51** |

The count column of the families sums to 67 with three geometries shared with the record;
the DISTINCT set is 64. `map` compares every candidate's rebuilt centers against every
record's before solving (M-STALE-INSTRUMENT); a candidate that reproduces a record to `1e-9`
bohr is not solved, is written as `duplicate_<name>.json` naming the record, and is counted
ONCE in every fit and gate under the record's name.

**The term** (the family, DECLARED). For each cross-unit hydrogen–oxygen pair, with `H` the
hydrogen, `O_d` the oxygen of `H`'s own unit, `O_a` the other unit's oxygen, `h₁, h₂` the
hydrogens of `O_a`'s unit:

```
r    = |H − O_a|
θ_d  = the angle O_d–H···O_a               (180° when the donor's O–H points at O_a)
u    = (H − O_a)/r                          (from the acceptor oxygen toward the hydrogen)
b    = unit((h₁ − O_a) + (h₂ − O_a))        (the acceptor's bisector, toward its hydrogens)
n    = unit((h₁ − O_a) × (h₂ − O_a))        (the acceptor's plane normal)
l±   = −cos λ · b ± sin λ · n               (the two lone-pair directions, λ the lone-pair angle)

f_d(θ_d) = ((1 − cos θ_d)/2)^m
g_a(u)   = [ ((1 + u·l₊)/2)^k + ((1 + u·l₋)/2)^k ] / [ 2·((1 + cos λ)/2)^k ]

CT(g) = −P · Σ_{cross-unit H–O pairs} exp(−c·r) · f_d(θ_d) · g_a(u)
```

On the linear dimer of record `θ_d = 180°` and `u·l± = cos λ`, so `f_d = g_a = 1` there and
`P, c` mean what CT-1's meant; at `m = k = 0` the family IS CT-1's term. The grids: `c` on
`0.50–4.00` step `0.02` (176), `m ∈ {0, 1, 2, 4}`, `k ∈ {0, 1, 2, 4}`, `λ ∈ {0, 30, 45, 55,
70}°` (the tetrahedral lone pair sits near 55°); `P` by weighted least squares in closed form
per grid point, weights `1/max(|ΔE_exact|, 5e-3)²` (CT-1's rule), `P ≥ 0`. The fit is the
grid point of least weighted residual over ALL 64 nodes; any of `m, k, λ` at an EDGE of its
grid is reported as the grid's number (`k = 4` or `m = 4`: the family lacks steepness; `λ =
0°` or `70°`: the lone-pair angle unresolved), and the fit is then read as (b) at best.

**The contact terms, fit INSIDE the boundedness gate.** With the wall (FIELD-9's `wall9.json`)
and the transfer term held, the remainder `ΔE_exact − [E_q(g) − E_q(40)]_engine − wall(g) −
CT(g)` on every node is fit by the two-class (H–O, H–H) exponentials on the same grid and
weights — but for each grid exponent the amplitude is CLAMPED at `P_max(c)`, the largest
amplitude (bisection on `[0, P_ls]` to `1e-6` relative) for which `SeamModel::bounded(q_H,
r_min, kT)` returns `None` with every other term of the law held (`r_min` FIELD-9's; the
transfer term at its linear value `f_d = g_a = 1`, the deepest it can be). The chosen point
is the admitted grid point of least weighted residual; the UNCONSTRAINED fit is recorded
beside it with the ratio of the two residuals — a ratio above 2 is the reading that the data
want a shape the gate forbids, named, not a failure. The clamp runs with `C₆ = 0`, because
FIELD-6's rule fits dispersion AFTER both contacts; G-B0 on the full law (dispersion in) is
the gate, and a `C₆` that breaks it is reported as such. Dispersion by FIELD-6's rule after both.

**The engine.** `SeamModel` gains `m_ct: u8, k_ct: u8, lambda_ct: f64` beside `p_ct, c_ct`
(checkpoint v12; the door); `accumulate_seam` evaluates the term on every cross-unit H–O pair
with the acceptor's frame taken from ITS unit's two hydrogens (`unit_of`), and posts analytic
forces on all five atoms; `SeamModel::bounded` and `hole` take the transfer term at its
linear value. The plant `SeamPlant::FlipChargeTransfer` is unchanged. Gates: the seam suite
(FIELD-3's G-B0…B4 unchanged in letter) and G-B3 below on the angular term.

**The arms.** FIELD-9 §2 S3's letter (the dimer and the ring, 293 K and 150 K, 2,000 counted
frames after settling; VOID if the drift peak exceeds `100×` the OFF arm's, the mean
temperature is more than `30 %` from target, or the unit count leaves the staked value) by
`field3_hbonds.rs` reading `wall_ct2.json` and refusing a record `bounded` names.

**The liquid.** If S3 reads (a) and G-B0 admits, LIQUID-1's condition is met; its counted arm
runs under its own freeze and amendment and writes its own results.

## 1. The expectation, written before the arms (M-EMPTY-SECTOR discharged)

As FIELD-4 §1 with every part written separately; the arms' expectation per FIELD-9. The
map's own expectation, written before any new solve: charge transfer is attractive
(`E_CT < −1e-6`) at every node whose shortest cross-unit H···O is at or under `5.0` bohr; a
node beyond that may read null, and reads it as a null, not a miss.

## 2. Gates

- **G-D0 — the product-start solver is the stock solver's equal.** On the thirteen exact
  nodes of record `|E_full − E_record| ≤ 1e-8`, EXACT count 13, exit `Converged`, residual
  `≤ 1e-8`; its price per node recorded beside the stock solver's from the record.
  witness: none (thirteen differences and a count)
- **T0 — the sector is what it says (CT-1's rule, amended).** On every new node: residual
  `≤ 1e-8`, `Converged`, dimension `194,481` EXACT, `E_exact ≤ E_noCT ≤ E_HL(undeformed)`
  within `1e-10`.
  witness: none (a residual, a count, an order)
- **A1 — the map.** `E_CT < −1e-6` at every node whose shortest cross-unit H···O `≤ 5.0`
  bohr (the expectation of §1); the two families' shapes — `E_CT` against tilt at each `R`,
  against twist-tilt at each `R`, against donor bend — written as tables.
  witness: none (a sign per node and four tables)
- **P1 — the family contains CT-1.** The fit restricted to `m = k = 0` on CT-1's twelve
  reproduces CT-1's `P_CT = 1.474879879, c_CT = 1.46` to `1e-9` (EXACT arithmetic).
  witness: none (arithmetic)
- **S1 — the angular term.** Within `max(0.25·|ΔE_exact|, 5e-4)` of the measured `E_CT` on
  every node ⇒ **(a)**; on at least 80 % (52 of 64) ⇒ **(b)**, misses named; fewer ⇒ **(c)**.
  Named beside the branch: the twisted dimer of record (`twisted_R3.0`, CT-1's 60 % miss) —
  within or not, with its number; and whether any of `m, k, λ` sits at a grid edge.
  witness: none (a fit against a stated tolerance)
- **C1 — the contacts under the gate.** Within the same tolerance on at least 80 % of all
  nodes AND the line at 2.7, 2.9, 3.1 Å within; the constrained/unconstrained residual ratio
  reported; a class whose exponent sits at a grid edge reported as such.
  witness: none (a fit against a stated tolerance, and a ratio)
- **G-B0 — bounded, every class named.** `bounded(q_H, r_min, kT)` on the full law returns
  `None` (by construction; RUN regardless); if it names anything, the constrained fit is
  broken and the campaign stops there.
  witness: none (an order on a grid, with a depth, every class)
- **G-B3 — the force is the derivative of the angular term.** On the dimer of record with
  the full law, the central difference at `h = 1e-5` bohr of the seam energy against the
  posted seam force on EVERY atom (both units' oxygens and all four hydrogens): worst relative
  `≤ 1e-8`; momentum and the books closed over 2,000 steps (the seam suite's G-B1/G-B2 letter).
  witness: none (a finite difference on six atoms; conservation gates)
- **G-C1 — the engine's arithmetic, one reference.** `1e-10` on every node with the full law,
  each served with two units (EXACT count).
  witness: none (arithmetic)
- **S2 — a twisted, tilted and bent bond, predicted forward.** The held-out geometry:
  `twisted(3.1, 45°, 45°)` with the DONOR bent `15°` about its own x-axis (`R_OO = 3.1` Å) — a
  twist, a tilt and a bend at once, at a distance and angles no fit point has.
  `prediction.json` BEFORE the solve with every part; the exact solve (G-D0's solver) and the
  sector solve; **(a)** `|E_pred − ΔE_exact| ≤ max(0.25·|ΔE_exact|, 5e-4)`; **(b)** the total
  misses but the transfer term is within `max(0.25·|E_CT|, 2e-4)` of the measured `E_CT`;
  **(c)** both miss.
  witness: none (a prediction filed before its measurement)
- **S3 — retention, read beside the drift.** FIELD-9 §2 S3's letter: **(a)** at 293 K
  `f_SEAM ≥ 0.5` on both the dimer and the ring; **(b)** at 150 K only; **(c)** neither;
  **VOID** if a needed arm is.
  witness: none (a measured population against a frozen instrument, with its own drift)
- **L — the liquid.** If S3 (a) and G-B0 `None`: LIQUID-1 runs; its gates are its own.
  witness: none (a condition met, recorded)

## 3. What each outcome means

S1 (a)/(b) with C1 passing, G-B0 `None` and S3 (a) is the six-channel seam law the liquid was
waiting on, every constant derived, and LIQUID-1 runs. S1 (c) says the declared family does
not carry charge transfer across orientations at this basis, and names by its grid edges
which direction it lacks. C1 with a residual ratio above 2 says the remainder after charge
transfer wants a shape a bounded exponential cannot take — the reading that names the next
shape. S3 (b)/(c) with every fit passing says a law right on 64 dimers is not yet a law for
a bond at temperature, and names how far.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

64 distinct dimer geometries of five families at one basis; one bond predicted once; two arms; and
then a liquid — 128 waters asked about a law fit on two.

## 5. Plants

- **(i) The angle removed.** The family at `m = k = 0` (CT-1's term) must MISS the twisted
  dimer of record by at least `2e-3` hartree where the fitted family's miss is reported
  beside it. Carrier: `|E_CT(twisted_R3.0) − CT_{(0,0)}(twisted_R3.0)| ≥ 1e-3`, asserted
  nonzero in the sector the plant acts on (the twisted node's transfer energy).
- **(ii) The sign of the transfer term.** `P → −P` in the engine (`FlipChargeTransfer`): G-C1
  must fail at the linear 2.9 Å node by `2·|CT(2.9 Å)|` to `1e-10`; carrier `|CT(2.9 Å)| ≥
  1e-4`, asserted nonzero in the sector the plant acts on.

## 6. Discipline

Engine: `seam.rs` (the angular factors, `m_ct, k_ct, lambda_ct`, `bounded`/`hole` at the linear
value), `sim.rs` (the term and its five-atom forces in `accumulate_seam`), `checkpoint.rs`
(v12), `lib.rs` (the door); gates in `tests/seam.rs` (G-B3 on the angular term). Runner
`holon-render/examples/ct2_harvest.rs`: `map` (G-D0's thirteen checks, the duplicate check against every record, then the 51 new
exact and sector solves, detached, a per-node checkpoint between the exact and the sector
solve so a death between them repeats neither, each node's JSON valid), `fit` (T0, A1, P1, S1, the constrained
C1 with the unconstrained beside it, dispersion, G-B0, G-C1, plants, `wall_ct2.json`,
`prediction.json` BEFORE the S2 solve), `predict` (S2); the arms by `field3_hbonds.rs`
reading `wall_ct2.json`; JSON under `conformance/water_observatory/ct2/`; results
`CT2_RESULTS.md` with the instruments. No number enters from outside the engine and its own
solver.
