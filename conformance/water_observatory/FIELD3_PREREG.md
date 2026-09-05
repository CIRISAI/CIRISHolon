# Pre-registration — FIELD-3: the unit as a closure, and the seam served by the channels — the strongest-bond identity, closure surfaces confined within units, the exchange wall harvested from the exact dimer, and the hydrogen bond re-asked

*Frozen 2026-09-05, committed ALONE, before the runners existed. Built by the lead. FIELD-2
read (c) by letter and VOID in substance: the engine's two-body bond verdict bonds the donor
hydrogen to the acceptor oxygen at the hydrogen-bond distance, the unit rule then assigns no
water, and the field never acted; and, unstaked, the bare force law REPELS at the
hydrogen-bond geometry by +21.0 mHa because the water monomer's (O,H,H) surface and the O–H
radical curve are served across the seam between two molecules (FIELD2_RESULTS.md). This
freeze does what those two findings say, in the order the evidence puts them: the identity
becomes a closure reading; the closure surfaces serve only within the closure; the contact
between closures is served by the channel ledger (OBJECT.md rule 10) — channel 1 as built,
channel 5 as a value TRANSFERRED from the exact dimer's residual — and the hydrogen bond is
asked again from FIELD-2's own bonded starts.*

misfits: contacts **M-EMPTY-SECTOR** (the expectation rule has an EMPTY branch: a start whose
assignment yields fewer units than staked, or whose binding at the start reads under `1e-4`
hartree in magnitude, VOIDs its arms before they run); **M-PLANT-OBS** and **M-PLANT-SECTOR**
(three plants, each with its carrier asserted nonzero in the sector the plant acts on, each
re-derived for this instrument — §5); **M-CHEAPER-THAN-ITS-PRICE** (the harvest's cost model
is EMBED-3's record — `1,002,001` determinants, 574–950 wall-seconds per node on 32 threads,
about 6.5 core-hours per node — and a node that returns in under a tenth of it is refused
as not that solve); **M-EXIT-DISCRIMINATOR** (every exact solve records its Davidson
iteration count and residual, and a node stopped by the iteration cap is VOID and says so);
**M-STALE-INSTRUMENT** (this freeze alone; runners, JSON and the results document
together); **M-VACUOUS-SUCCESS** (every arm reports its frame count and the seam's work
counters; a seam arm whose seam rule dropped zero cross-unit terms is VOID); **M-NULL-MISSTAKE**
(retention is staked on the rung-1 lens, unchanged); **M-FIXED-POINT-TRAJECTORY** (the OFF
arms are FIELD-2's arms, same seed, and must reproduce FIELD-2's numbers bit for bit);
**M-UNTESTED-GAP** (the wall is harvested on the LINEAR geometry and predicted forward on a
geometry it was not fit to — §4); **M-BARE-CHARGE**, **M-HOMOG** (the words "charge",
"local" appear; classical charges, nothing homogeneous); **M-COND-PROBE** ("inside the"
appears; force terms, not post-step operators); **M-DEVICE-CLASS** (native `f64`, one class);
**M-FLOOR-UNSTAKED** (the harvest's residual has a staked floor: `1e-6` hartree, the
supermolecule solve's residual bound times the determinant count's square root, and no
clause reads a ratio whose denominator is under it). Not contacted: the rest of the registry.

## 0. What is built (the instrument), and what is measured

**A — the unit as a closure reading.** Each hydrogen is assigned to the oxygen it is MOST
BOUND to by the engine's own O–H pair curve (`u_OH(r)` lowest, among oxygens inside the
curve's reach); a WATER UNIT is an oxygen with exactly two hydrogens so assigned. Every
other atom is FREE and keeps the atom-level law with everything. This replaces FIELD-1
AMENDMENT 1's rule (exactly two pair-bonded hydrogens, none shared) for the field's
assignment and is the unit the seam rule reads. Where no hydrogen is contended the two
rules agree by construction, and G-A2 checks that on FIELD-1's own scene.

**B — the seam rule: closure surfaces serve only within the closure.** With the seam ON:
the pair tables serve a pair only when both atoms are in ONE unit or either is free; the
three-body surfaces serve a triple only when all three are in one unit or any is free. A
cross-unit contact is served by the ledger: channel 1 (the field, FIELD-1's term, on the
new assignment) and channel 5 (the wall, below). Channels 2 and 4 are NOT served across the
seam in this freeze and the results document says what they were measured to be worth
(§3, branch (b)). A change of unit membership is a transition: the seam's energy jump at
fixed positions, old assignment to new, is posted to `w_ext` and to its own receipt column
`work.seam`, exactly as the field's transitions are (FIELD-1). The seam's cross-unit drops
are counted (`seam_work.pairs_dropped`, `triples_dropped`) and the fence count excludes
them. The wall is a ledger row `Row::Seam`, appended at the END of `Row::ALL`, reading an
exact `0.0` when the seam is off. Checkpoint format v7 carries the seam state.

**C — the wall, harvested.** Channel 5's shape is declared by the ledger (exponential,
`Kind::Identity`); its coefficients are transferred from the exact dimer's residual, never
chosen. On the seam programme's exact solver (`holon_chem::embed::supermolecule`, the
1,002,001-determinant water dimer of EMBED-3 System B, `water_dimer_linear` at EMBED-1's
pins), six LINEAR nodes:

    R_OO ∈ {2.5, 2.7, 2.9, 3.1, 3.4, 3.7} Å  (4.72, 5.10, 5.48, 5.86, 6.43, 6.99 bohr)

Per node: `ΔE_exact(R) = E_super − E_A0 − E_B0` (monomers at the pin in their own basis;
the referee is the basis's own dimer and its superposition error is an UNPAID DEBT of the
referee, named, not corrected). The field's value on the same geometry is the ENGINE's
(`Sim::field_energy_of` on the new assignment with the pin charge `q_H = 0.231380372`;
FIELD-1's G4 ties it to the record). The residual:

    r(R) = ΔE_exact(R) − E_field(R)

The wall on each cross-unit O–O pair is `A·exp(−b·R)`, with `ln A` and `b` the weighted
linear regression of `ln r(R)` on `R` over the nodes where `r(R) > 0` (weights
`1/|ΔE_exact(R)|²`; a closed form, no iterative optimiser). Floor: a node whose `|r| < 1e-6`
hartree is not a reading.

**D — the hydrogen bond, re-asked.** FIELD-2's dimer and cyclic tetramer starts, seeds,
thermostat and lens, at 293 K and 150 K: OFF (the bare law — FIELD-2's arms again) and
SEAM (assignment A, seam rule B, field, wall). 2,000 settling frames, 20,000 counted.

## 1. The expectation, written before the arms (M-EMPTY-SECTOR discharged)

For each start the SEAM law's binding, `E(start) − E(separated)` with the molecules moved
40 bohr apart: by G-B4 this is the field plus the wall and nothing else. The rule, with its
EMPTY branch: the assignment yields fewer units than staked (2 on the dimer, 4 on the ring)
or `|binding| < 1e-4` ⇒ **VOID**, the arms for that start do not run; binding `≤ −2 kT` ⇒
expected to hold; `> −kT` ⇒ expected to break; between ⇒ no expectation. `kT` is
`9.28e-4` (293 K), `4.75e-4` (150 K).

## 2. Gates

- **G-A1 — the units exist where FIELD-2 found none.** The new assignment on FIELD-2's
  dimer, ring and square starts yields `2`, `4`, `4` units (FIELD-2's rule: 0, 0, 4), and the
  field's binding at the start on the dimer and the ring is NEGATIVE with magnitude `≥ 1e-4`
  hartree. Else VOID for the seam, before anything else runs.
  witness: none (the assignment read on the staked starts)
- **G-A2 — the identity where nothing is contended.** On FIELD-1's four-water walled scene
  after 2,000 steps, the new assignment equals the old one atom for atom and `e_field` is
  bit-identical (EXACT).
  witness: none (bytes)
- **G-B0 — the seam off is the identity.** Checkpoint BYTES identical over 2,000 steps on
  the FIELD-1 scene with the seam enabled-then-disabled before the first step; the channel
  receipt (`tests/data/channel_ledger.receipt`) unchanged line for line with `Row::Seam`
  reading `0.0` (EXACT). The receipt is re-banked only if the appended row changes a line,
  under this freeze's cause line, and the results document says whether it did.
  witness: none (bytes; the receipt file)
- **G-B1 — the books close with the seam on.** Dimer and ring, 293 K, 2,000 steps: receipt
  columns (`hand`, `thermostat`, `barostat`, `acuity`, `field`, `seam`) sum to `w_ext`, and
  the honest drift peak is under a tenth of the largest posted transition, or under `1e-5`
  hartree when no transition was posted. 2 scenes.
  witness: none (engine ledger)
- **G-B2 — momentum.** Internal forces with the seam on (tables, field, wall) sum to under
  `1e-12` of the largest force magnitude; residual under the engine's bound over 2,000 steps.
  witness: none (conservation gate)
- **G-B3 — the wall is the derivative of its energy.** Central difference of `E_seam` at
  `h = 1e-4` bohr against the analytic force on every atom of the dimer start:
  `|F − (−∂E)| / |F| ≤ 1e-8` on every component where `|F| > 1e-10`.
  witness: none (finite difference)
- **G-B4 — the closures contribute exactly nothing across the seam.** With the seam on and
  the wall's coefficients set to zero, `e_pair + e_three` on the dimer at the FIELD-2 start
  equals `e_pair + e_three` on the same dimer at 40 bohr to the last bit (EXACT): every
  cross-unit table term is dropped, not attenuated. FIELD-2 measured `+21.0` mHa here.
  witness: none (bytes)
- **G-C0 — the price.** The 2.9 Å node: `1,002,001` determinants (EXACT, the basis's count),
  wall time `≤ 1800` s on the declared thread count, else the harvest is REFUSED. Every node
  reports its Davidson iterations and residual (`≤ 1e-9`, EMBED-3's bar) and a node stopped
  by its iteration cap is VOID.
  witness: none (a price, recorded)
- **G-C1 — the harvest is the engine's own arithmetic.** With `(A, b)` loaded, the engine's
  seam-law interaction on each linear node (`Sim` at the node's geometry, `E(R) − E(40 bohr)`)
  equals `E_field(R) + A·exp(−b·R)` to `1e-10` hartree — the same formula, evaluated by the
  force law, on all 6 nodes.
  witness: none (arithmetic)
- **S1 — what the residual is.** Over the six linear nodes: **(a)** `r(R) > 0` at every node
  and the two-parameter wall reproduces `r(R)` within `0.10 · |ΔE_exact(R)|` at every node ⇒
  channel 5 carries the whole cross-unit remainder at this level; the wall is transferred in
  full. **(b)** `r(R) > 0` and the fit within `0.10 · |ΔE_exact|` on a contiguous set of the
  shortest nodes (at least three, ending at a named `R_x`), and beyond `R_x` either `r ≤ 0` or
  the fit misses ⇒ the wall is harvested from the nodes it fits, and the remainder beyond
  `R_x` is MEASURED — its log-log slope between consecutive nodes reported, read as
  channel 2 (induction) if it lies in `[−7, −3]`, as charge penetration (the density field
  at the seam, EMBED-2's instrument) if it decays faster than any power — and named as the
  next transfer, not served here. **(c)** `r(2.5 Å) ≤ 0`, or fewer than three positive nodes
  ⇒ the point-charge field is not the seam's electrostatics at contact; no wall is
  harvested, the seam arms do not run, and FIELD-4 stakes the density field at the seam.
  witness: none (a fit against a frozen tolerance)
- **S2 — the forward prediction (rule 6).** Under (a) or (b), BEFORE the held-out node is
  solved, the runner writes `prediction.json`: the seam law's interaction on the FLIPPED dimer
  (the acceptor rotated by π about the x-axis through its oxygen, hydrogens toward the donor)
  at `R_OO = 3.4` Å, from the linear fit. Then the exact solve. **(a)** `|E_pred − ΔE_exact| ≤
  max(0.25 · |ΔE_exact|, 5e-4)` hartree ⇒ the wall transfers across orientation: the O–O
  contact carries the seam's exchange at this level. **(b)** it misses ⇒ the wall is
  orientation-dependent at this level; the miss is reported in hartree and as a fraction, and
  the H···H contact is named for the next harvest. 1 node, staked once.
  witness: none (a prediction filed before its measurement)
- **S3 — retention under the seam law.** Per system and temperature, `f_SEAM` and `f_OFF`
  over 20,000 counted frames (the rung-1 lens): **(a)** at 293 K `f_SEAM ≥ 0.5` on the dimer
  and on the ring ⇒ the seam law holds water's hydrogen bond at room temperature; the
  H-bond network carrier is run next under it. **(b)** (a) fails at 293 K but `f_SEAM ≥ 0.5`
  at 150 K on both ⇒ the seam law binds and `kT` unbinds it at this level; FIELD-4 stakes
  channel 2. **(c)** fails at both ⇒ the seam law as built does not hold the bond; the
  binding at the start (§1) says whether that was expected, and the results document reads
  the dynamics' own diagnostic (the probe of FIELD-2, under the seam law). `f_OFF` must
  reproduce FIELD-2's `0.0000` on every arm (EXACT), else the OFF arm is not FIELD-2's and
  the comparison is VOID. 2 systems × 2 temperatures.
  witness: none (a measured population against a frozen instrument)

## 3. What each outcome means

S1 (a)/(b) is the ledger doing what rule 10 says: the residual of the exact solve over the
field is carried across the matrix into the channel whose shape it has, as a value, and
what does not fit that shape is measured and NAMED rather than absorbed. S1 (c) says the
charge is the wrong seam electrostatics at contact and the density field is next. S2 is the
first rule-6 support this programme can claim for a transferred coefficient: a number
filed before the measurement it predicts. S3 (a) is the payoff FIELD-2 could not reach —
water bonding to water for a reason the engine derived — and (b)/(c) are temperature facts
with a named next channel.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

The wall is fit on one orientation and one contact type (the O–O pair), and the dynamics
sample every orientation. S2 is the one measurement across that gap, staked once. A wall
that passes S2 and S3 is still a wall fit at six points on one line, and the results
document says so.

## 5. Plants

- **(i) The sign.** `A → −A` in the wall. G-C1 must fail at the 2.5 Å node by
  `2·A·exp(−b·4.72)` to within `1e-10`. Carrier: `A·exp(−b·4.72) ≥ 1e-3` hartree, asserted
  nonzero in the sector the plant acts on (the wall at contact).
- **(ii) The triples served across the seam.** Seam rule for pairs only; the three-body
  surfaces served to cross-unit triples. G-B4 must fail by the cross-unit three-body sum at
  the dimer start, and that sum must equal FIELD-2's `+0.041914` hartree to `1e-9`. Carrier:
  the cross-unit three-body sum `≥ 1e-3` hartree, asserted nonzero in the sector the plant
  acts on (the three-body row).
- **(iii) The reaction dropped on the wall.** `F_j` not applied. G-B2 must fail: the
  internal force sum over 2,000 steps exceeds `1e-6` of the largest force. Carrier:
  `|F_wall| ≥ 1e-6` hartree/bohr at the dimer start, asserted nonzero in the sector the plant
  acts on (the wall's force).

## 6. Discipline

Runners: `holon-render/examples/field3_harvest.rs` (C: six nodes, the fit, `prediction.json`
BEFORE the flipped node, then the flipped node; per-node JSON under
`conformance/water_observatory/field3/`, `wall.json` with `(A, b)` and the fit's residuals);
`holon-render/examples/field3_hbonds.rs` (§1 expectation first, then the arms); gates in
`holon-render/tests/seam.rs`. Results `FIELD3_RESULTS.md` committed with the runners. The
harvest runs detached (`setsid nohup`, done-markers) on the declared thread count with the
lead's builds pinned away from it. No number enters from outside the engine and its own
exact solver.
