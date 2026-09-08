# Pre-registration (DRAFT) — COMPARE-0: the programme's water law priced against MB-pol and TIP4P/2005 on the SAME geometries — total interaction energies and forces beside the channel errors, the interpolant's error measured apart from the Hamiltonian's, and the honest expectation written first: at a minimal basis with no transferred dispersion this law does not beat MB-pol on accuracy, and what the comparison measures is stated before it is run

*Frozen 2026-09-08, DRAFT. The first external review's third ask (`GANTT2.md` "The external
review", item 3) and the second review's fourth build step (`GANTT2.md` "The second review",
step 4). CT-3 read: the map beats the family on a held-out bond, the whole served law is
bounded, and `ct3/wall_ct3.json` is the term LIQUID-2 would run. What no record in this tree
carries is the served law's TOTAL interaction energy node by node beside the exact one — C1
reported a weighted residual and a within-count, never the totals — and no record carries any
number from a model outside this programme. This freeze fixes both, on the sixty-five geometries
the programme has already solved exactly, and it commits IN ADVANCE to what each outcome
chooses among. Its phase 1 (this document's gate) runs on records; its phase 2 (the liquid,
under matched conditions, and an NPT density test) is NAMED and NOT RUN.*

misfits: contacts **M-STALE-INSTRUMENT** (every geometry is rebuilt by the map's own builders
and every exact number is read from its own record with the record's path printed beside it;
no number is taken from a results document, and the instrument's commit is recorded);
**M-CHEAPER-THAN-ITS-PRICE** (the cost model this freeze reports is measured, not estimated:
the per-node price of the truth is the records' own `cpu_seconds`, the reference models' price
is their own wall time, and no core-hour figure is quoted that the run did not measure);
**M-PLACEMENT-LOTTERY** (cores 18–23 under `taskset`; no gate here is a wall clock, and every
cost number is reported as measured on a shared machine and is not a per-evaluation cost model
for the liquid); **M-VACUOUS-SUCCESS** (G-X0 exists precisely because a runner that agrees with
itself proves nothing: it checks this lane's served numbers against CT-3's OWN filed numbers for
the one geometry both compute, and it would fail if this lane had rebuilt the law differently);
**M-FORMAT-FLOOR** (every record validates as JSON, twelve significant digits from the engine
side, never a leading `+` on a mantissa); **M-FLOOR-UNSTAKED** (the force gate's bar is
ABSOLUTE, `1e-8` hartree per bohr, because a force component passes through zero and a relative
reading explodes there; the relative reading is kept beside it scaled by the node's own largest
component); **M-TRUNCATION-AS-ERRORBAR** (the interpolant's in-sample residual is an exact zero
by construction and is never quoted as its error; the gauge is leave-one-out on the map and the
direct miss on the node held out from the table); **M-PLANT-OBS** and **M-PLANT-SECTOR** (two
plants, §5, each naming the sector it acts on and asserting its carrier nonzero in that sector);
**M-EXTRAPOLATED-HOLE** (the served law carries an attractive contact term and the walk that
admits it is CT-3's, unchanged and not re-argued here); **M-FIRST-VIOLATION-ONLY** (no
boundedness gate is re-run in this freeze and none is claimed; CT-3's G-B0W is cited as the
record it is, every class and every leg named there); **M-ONE-MODEL-DELTA** (the comparison is
three models against one truth on one set of geometries, not one model against one rival, and
the two reference models are reported side by side and never merged into a single "reference");
**M-POPULATION-CHOICE** (the sixty-five geometries are the ENTIRE set the programme has solved
exactly — nothing is selected, and the one node held out from every fit is reported separately
by name); **M-BASE-RATE-OMITTED** (every error is reported beside the DEPTH of the bond it is an
error of, so a small error on a shallow bond cannot read as accuracy); **M-UNTESTED-GAP** (§4:
what a dimer set cannot decide is named before the branches are committed);
**M-INEQUALITY-READ-AS-EQUALITY** (the TIP4P/2005 placement rule gives a family of possible
mappings and two are run; the spread between them is reported as the placement's own cost and
is never collapsed to one number); **M-EMPTY-SECTOR** (a reference that does not install is a
VOID reading and is recorded as such with its exact failure, never replaced by a number from a
paper's table without that table's citation); **M-EXIT-DISCRIMINATOR** (no new solve runs here,
so no iteration cap can fire; the records' own exits are carried);
**M-FOREIGN-DOMAIN-CORROBORATION** (agreement with MB-pol is not evidence that the programme's
law is right, and disagreement is not evidence that it is wrong — both are readings against a
model, and the only truth on this record is the exact solve); **M-MAINTENANCE-LENS** (no rent
clause is claimed here); **M-BARE-CHARGE**, **M-HOMOG**, **M-COND-PROBE**, **M-DEVICE-CLASS**,
**M-VOLUME-SCALE**, **M-PROVENANCE-OVERREACH**, **M-GAUGE-LAUNDER**, **M-PARITY-PROTECT**,
**M-LOOP-BLIND**, **M-IDLE-CALIBRATED-TIMEOUT**, **M-PROBE-THE-RESOURCE**, **M-NULL-MISSTAKE**,
**M-FIXED-POINT-TRAJECTORY**, **M-IMPORT-EXECUTES**. Not contacted: the rest of the registry.

## 0. What is compared, and what is NOT a prediction

**The geometries.** Sixty-five, and they are every one the programme has solved exactly: CT-2's
sixty-four-node map (fifty-one in `ct2/node_*.json`, thirteen of record in `ct2/gd0_*.json`
with their closed-sector partners in `ct1/sector_*.json`) and CT-3's held-out node
(`ct3/node_twistbent_R2.7_tw20_t85_d0.json`). Nothing is selected. Every geometry is rebuilt
from the record's own centers and the coordinates are printed in the scoring record so a reader
can re-run any model on them.

**The truth.** `ΔE_exact = E_exact(dimer) − E_A0 − E_B0`, the records' own rule, from a full CI
solve in the minimal (STO-3G) basis. This is exact FOR THAT HAMILTONIAN and it is not water:
the review's sentence is kept in the vocabulary and it governs every reading below.

**The three models, on the same sixty-five.**

| model | what it is | where its numbers come from |
|---|---|---|
| the served law | CT-3's: FIELD-9's three walls and the point charge from `ct2/wall_ct2.json`, C1's re-fit contact terms from `ct3/wall_ct3.json`, the transfer from the table rebuilt out of the map's own records, dispersion an exact `0` | the engine, through `holon-render`'s `compare0_harvest` example; `E(g) − E(acceptor 40 bohr along x)` on the engine's own rows |
| MB-pol | the Paesani group's many-body potential | MBX, the group's own C++ library (`github.com/paesanilab/MBX`), through its python plugin; the library's energies and gradients, converted by the library's own constants. No MB-pol number is typed by hand |
| TIP4P/2005 | Abascal & Vega, J. Chem. Phys. **123**, 234505 (2005) | the paper's model-parameter table (`d(OH) = 0.9572` Å, `HOH = 104.52°`, `d(OM) = 0.1546` Å, `q(H) = 0.5564` e, `σ = 3.1589` Å, `ε/k_B = 93.2` K), implemented in `compare0/references.py`; every unit conversion from `scipy.constants` (CODATA), none from memory |

**The rigid model's placement rule, stated and PRICED.** The map's monomer is the minimal-basis
relaxed water (`O–H = 1.943574` bohr, `H–O–H = 96.76°`); TIP4P/2005's is its own
(`0.9572` Å, `104.52°`). The two cannot both be honoured, so two rules are run and both are
reported:

* **remap (PRIMARY).** The oxygen, the H–O–H bisector and the molecular plane are kept exactly;
  the two hydrogens are placed at the MODEL's own bond length and angle in that plane, each on
  the side it was on; `M` sits on the bisector at `d(OM)`. The model then runs on the geometry
  it was parameterised for, and the price is the hydrogens' movement, which the record measures.
* **as_is (SENSITIVITY).** The record's own hydrogens are used unmoved and only `M` is placed.
  The model runs on a monomer it was never fitted to.

The spread between the two is the placement's own cost, MEASURED. Neither rule is chosen by its
answer, and the primary is named here, before either is read.

**What is NOT a forward prediction, said plainly.** Every one of these sixty-five exact numbers
was in the tree before this freeze was written, and this lane's instrument computed the served
law's totals on them BEFORE this document existed — the instrument had to exist for the freeze
to be able to state a derived stake at all. So rule 6 gives phase 1 no support weight: it is a
SCORING of records, a diagnostic, and it is labelled that way in the results. What this freeze
puts forward is (a) the decision rule in §3, written before the numbers were read into it, and
(b) phase 2's stakes in §2, whose data do not exist.

**Phase 2, named and NOT run.** The liquid under matched conditions (structure, pressure,
diffusion, on LIQUID-2's box against the same two references) and an NPT density test before
any density claim. Both wait on LIQUID-2, which waits on the partition-of-unity serving rule
(`liquid2/DRIFT_NOTE.md`); neither is run here and neither is claimed.

## 1. The expectation, written before the scoring is read (M-EMPTY-SECTOR discharged)

**The programme's law is not expected to beat MB-pol on accuracy against experiment, and it is
not expected to beat it against the exact solve either.** MB-pol is fitted to CCSD(T) two-body
energies in a large basis and carries explicit dispersion and explicit polarisation; the served
law is fitted to sixty-four FCI/STO-3G points, has `C₆ = 0` by FIELD-6's rule, and has no
polarisation term at all. Writing that down first is the point: a comparison whose losing
outcome would be spun is not a measurement.

What the comparison MEASURES is three separable things, and each is reported apart:

1. **The law against its own Hamiltonian.** `served − ΔE_exact` on the sixty-five. This is the
   only place the programme's law can be right or wrong, because the exact solve is the truth
   the law was fitted to.
2. **The interpolant against the Hamiltonian's `E_CT`.** In sample this is an exact zero — an
   interpolant interpolates — so the gauge is leave-one-out on the map and the DIRECT miss on
   the one node the table never saw.
3. **The Hamiltonian against reality's best available proxy.** `MB-pol − ΔE_exact`. No model on
   this record is charged with this number: it is the minimal basis's own error and it is what
   the basis lane I-5 exists to close.

**The empty branch.** If MB-pol cannot be installed on this machine the reading is VOID for
every quantity that needs it (the Hamiltonian gap, every force comparison, branch D of §3), the
exact failure is recorded verbatim, and the freeze does NOT substitute a remembered number.
Published MB-pol energies may be cited only where a record carries the paper and its table, and
only for a geometry that paper published. TIP4P/2005 and the served law are still scored: that
half of the comparison does not depend on the install.

## 2. Gates

The gate phase is CHEAP and runs on records. `served` is the engine's example
(`engine/crates/holon-render/examples/compare0_harvest.rs`), `references.py` and `score.py` the
python beside the records (`conformance/water_observatory/compare0/`).

- **G-T0 — the interpolant is an interpolant.** `|E(x_i) − v_i| ≤ 1e-12` hartree at every one of
  the table's sixty sites. CT-3's own bar, carried, because this lane rebuilds the table rather
  than reading it back and a rebuild that disagreed would be silent.
  witness: none (arithmetic on a linear solve at sixty points)
- **G-C1 — the engine serves what this runner sums.** `|e_seam(engine) − the runner's own
  channel sum| ≤ 1e-10` hartree on every one of the sixty-five geometries. The channel sum is
  the decomposition the results table prints, so a wrong decomposition cannot pass.
  witness: none (a residual per geometry, two independent evaluations)
- **G-X0 — this lane serves what CT-3 served.** This lane's served total, field row, seam row
  and table reading for the held-out node against `ct3/gate.json`'s OWN filed numbers,
  `≤ 1e-12` hartree; and the rebuilt geometry against the record's centers, `≤ 1e-9` bohr. This
  is the gate that makes the rest of the comparison meaningful: a re-implementation that agreed
  only with itself would be a new law wearing CT-3's name.
  witness: none (four differences against a filed record, and a geometry check)
- **G-F0 — the served law's force is the derivative of the served law's energy.** On the
  HELD-OUT node, which is not a knot of the table: worst ABSOLUTE
  `|analytic interaction force − central difference of the interaction energy|` at `h = 1e-5`,
  over 6 atoms × 3 coordinates, bar `1e-8` hartree per bohr. The relative reading is kept beside
  it scaled by the node's OWN largest component. A second leg differences the two
  deepest-transfer MAP nodes, which ARE knots: a central difference at a knot of an `r³`
  polyharmonic kernel carries that kernel's own third-derivative discontinuity, so that leg is
  REPORTED and does not gate. The translation sum of the analytic forces is reported beside both.
  witness: none (a finite difference on 54 coordinates, and a translation identity)
- **G-R0 — the reference installs, or its failure is the record.** MB-pol is scored only if
  MBX imports and its shared library loads; otherwise `available: false` carries the exact
  failure string and every MB-pol quantity reads VOID. TIP4P/2005 is scored from the paper's
  table with its citation in the record; a missing citation is a refusal, not a warning.
  0 numbers may enter from memory.
  witness: none (an import, a load, and a citation)
- **G-P0 — the placement rule is priced.** The maximum hydrogen movement the remap rule imposes
  is reported per geometry in bohr, and the interaction energy under both rules is reported. If
  the spread between the two rules exceeds `0.25` of TIP4P/2005's own error against the exact on
  the same geometry, the classical reference's number is reported WITH that spread as its bar
  and no sharper claim is made about it.
  witness: none (two evaluations and a difference)

## 3. What each outcome chooses among — the branches, committed before the numbers are read

The reading chooses ONE of four next steps for the molecular tier. The criteria are stated in
quantities the gate phase measures, and they are mutually exclusive by construction: they are
tested in the order written and the first that fires is the choice.

| order | branch | fires when | what it reads |
|---|---|---|---|
| 1 | **a larger basis** (the I-5 lane) | `RMS(MB-pol − ΔE_exact) > RMS(served − ΔE_exact)` over the sixty-five | the law is already nearer its own Hamiltonian than that Hamiltonian is to the best available proxy for reality, so more work on the law buys nothing a reader outside the programme would credit, and the basis is the binding constraint. This is the branch §1's expectation names as likely, and it is written FIRST so its firing cannot be read as a surprise |
| 2 | **more table coverage** | branch 1 does not fire AND either `RMS(leave-one-out) ≥ 0.5 · RMS(served − ΔE_exact)`, or the worst served error sits at a geometry whose nearest site in the scaled box is outside the map's measured nearest-neighbour band `0.1211`–`0.4030` (`ct3/ct_table.json`) | the map has no knot where the law is worst, and CT-3's results named the two places (between the azimuth sheets at high tilt; a pair donating both ways at once). Nodes, not shapes |
| 3 | **a better angular representation** | branches 1 and 2 do not fire AND the served error is spread rather than localised: no single family carries more than `0.5` of the total squared error while holding less than `0.5` of the geometries | the error is in the functional form the table serves, not in where its knots are |
| 4 | **many-body polarisation** | branches 1, 2 and 3 do not fire | a REFERRAL, never a result: a dimer set cannot see a three-body term at all (§4), so this branch commits the next campaign to a trimer or small-cluster leg with MB-pol's own many-body decomposition (`E1b, E2b, E3b`) as referee, and may not be claimed on dimer evidence |

Whichever fires, the reading is entered as it falls and the losing outcome is reported as
plainly as a winning one.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

* **A dimer set cannot decide the many-body question.** Every geometry here is two waters, and
  MB-pol's three-body term is identically zero on all of them. Branch C above is therefore a
  referral and never a finding, and no sentence in the results may say that polarisation is or
  is not needed on this evidence.
* **The truth is a minimal-basis truth.** `ΔE_exact` is exact for the STO-3G Hamiltonian and no
  more. Both reference models are fitted to (or built for) real water, so the Hamiltonian gap
  is expected to be the largest number on the page. It is reported, and it is never used to
  score the programme's law.
* **The rigid model runs on a remapped monomer.** TIP4P/2005 cannot be evaluated on the map's
  own monomer without breaking the model's own geometry, so its number carries the placement
  cost measured in G-P0 and is reported with it.
* **MB-pol is being asked slightly off its manifold.** The monomers here are minimal-basis
  relaxed, not MB-pol's own equilibrium; the one-body term absorbs that and cancels in the
  interaction difference, but the two-body term is evaluated at monomer geometries outside the
  set it was fitted on. Named, not corrected.
* **No exact force exists in this tree.** No record in `ct1/`, `ct2/` or `ct3/` carries a force;
  the exact solves stored energies only. So NOTHING here is scored against a true force. The
  served law's force is reported against MB-pol's, per atom and as the net force on the
  acceptor, and that is a model-against-model reading. Exact forces on at least the held-out
  node and one node per family are OWED and named as owed.

## 5. Plants

Two, each naming the sector it acts on, and each with its carrier asserted nonzero in that
sector before the plant is read.

- **plant (i) — the classical reference's electrostatics removed.** TIP4P/2005's site charges
  are set to an exact `0` and the interaction is re-evaluated with the Lennard-Jones term alone.
  The energy must move by exactly the Coulomb sum, computed independently by the same routine.
  Sector: the electrostatic sector of the classical reference. Carrier: that Coulomb sum, which
  must be nonzero in that sector (bar `1e-9` hartree) or the plant is VOID.
- **plant (ii) — MB-pol's two-body sector emptied.** The acceptor is translated `40` bohr along
  x — the engine's own far reference — and MB-pol's interaction energy must fall below `1e-9`
  hartree. Sector: MB-pol's two-body sector. Carrier: `E2b` at the node, which must be nonzero
  in that sector (bar `1e-9` hartree) or the plant is VOID.

## 6. Discipline

* Cores 18–23 under `taskset`; no gate is a wall clock and every cost number says what it
  measured.
* No new exact solve runs in this campaign. Every exact number is a record's, with the record's
  path printed beside it in `scores.json`.
* No number enters from memory. The engine's numbers are the engine's, MB-pol's are MBX's,
  TIP4P/2005's five parameters are the 2005 paper's and are cited in the record itself, and
  every unit conversion is `scipy.constants`'.
* The interpolant's error and the Hamiltonian's error are reported apart, in every table, and
  are never summed into one headline.
* Every reference model's failure to install is a recorded VOID with its exact failure string,
  never a substituted number.
* The results document names phase 1 as a scoring of records and not as a forward prediction,
  and says which branch of §3 fired and why.
