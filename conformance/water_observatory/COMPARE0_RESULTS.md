# COMPARE-0 — results (phase 1: the dimer set)

*Freeze `COMPARE0_PREREG.md` (DRAFT, audit ADMITS; committed alone at `eb60c81`, corrected once
before either plant ran at `cd83305`). Instrument: `engine/crates/holon-render/examples/compare0_harvest.rs`
(a NEW example — nothing in the engine's `src/` is touched), `compare0/references.py`,
`compare0/score.py`. Records: `compare0/served.json`, `compare0/references.json`,
`compare0/scores.json`, `compare0/gate.json`, `compare0/REFERENCE_INSTALL.md`. Cores 18–23. No
new exact solve ran; every exact number is a record's, with its path in `scores.json`.*

## The verdict, first

**On the sixty-five geometries the programme has solved exactly, the served law is nearer the
exact answer than TIP4P/2005 is — by 7.1× in RMS over all sixty-five and 4.8× over the
thirty-seven the minimal-basis Hamiltonian actually binds. That comparison is IN-SAMPLE against
OUT-OF-SAMPLE and is not an accuracy claim. The one honest out-of-sample number on this record
is the single held-out node, and there the margin is 1.36×, not 7×. The three errors the freeze
required to be kept apart are kept apart below, and the largest of them is not the law's.**

**And the largest error on the page is the Hamiltonian's.** MB-pol — the Paesani group's own
potential, through the group's own OpenMM plugin — differs from the exact minimal-basis answer
by `7.059e-3` hartree RMS over all sixty-five and `1.599e-3` over the thirty-seven bound ones.
Both numbers are larger than the served law's own error against that same Hamiltonian
(`2.096e-3` and `6.740e-4`). By the freeze's §3, tested in the freeze's own order, **branch 1
fires: a larger basis, the I-5 lane.** The programme's law is already nearer its own Hamiltonian
than that Hamiltonian is to the best available proxy for real water, so more work on the law
buys nothing a reader outside the programme would credit. That is the branch §1 of the freeze
named as likely and wrote FIRST so its firing could not be read as a surprise.

## 1. The gates

| gate | verdict | the number |
|---|---|---|
| G-T0 — the interpolant is an interpolant | **PASS** | worst `9.992e-16` hartree at 60 sites (bar `1e-12`); the table is REBUILT here from the map's own records rather than read back from `ct_table.json` |
| G-C1 — the engine serves what this runner sums | **PASS** | worst `9.021e-16` hartree at `linear_R2.3` (bar `1e-10`), on all 65 geometries; the channel sum is the decomposition §3 prints, so a wrong decomposition could not have passed |
| G-X0 — this lane serves what CT-3 served | **PASS** | worst `2.992e-15` hartree over the held-out node's total, field row, seam row and table reading against `ct3/gate.json`'s own filed numbers (bar `1e-12`); the rebuilt geometry `3.313e-11` bohr from the record's centers (bar `1e-9`) |
| G-F0 — the served force is the served energy's derivative | **PASS** | held-out node (NOT a knot): worst ABSOLUTE `2.343e-11` hartree/bohr (bar `1e-8`), relative to that node's own largest component `1.808e-9`; knot leg (reported, not gating) `7.639e-11` at `linear_R2.5`; the analytic forces' translation sum `1.952e-17` |
| G-R0 — the reference installs, or its failure is the record | **PASS for both, by two routes and after four recorded failures** | MB-pol runs through `paesanilab/mbpol 1.1.2` on `openmm 7.2.2` — the group's own plugin. MBX, the group's C++ library, got to 102 of 253 translation units before its three-body degree-4 units exhausted a shared 31 GiB machine and the build was stopped; the MBX cross-check is OWED, not refused. `compare0/REFERENCE_INSTALL.md` carries every command and its result, the four failures included |
| G-P1 — the classical reference's force is its own derivative | **PASS** | worst ABSOLUTE `1.906e-11` hartree/bohr at `linear_R2.3` (bar `1e-8`), over 3 components × 65 geometries. It is here because a Coulomb sign error is invisible in the energy — and the first draft of that routine HAD one, found by writing the check before running it |
| G-P0 — the placement rule is priced | **MEASURED, and it bites** | the remap rule moves a hydrogen by up to `0.1851` bohr; the two placement rules differ by a median `3.038e-3` hartree and up to `2.28e-2`; on **53 of 65** geometries that spread exceeds a quarter of TIP4P/2005's own error, so the classical reference's number carries the spread as its bar and no sharper claim is made about it |
| plant (i) — the classical reference's electrostatics removed | **FIRES** | carrier `−1.0996e-2` hartree (the Coulomb sum) nonzero in the electrostatic sector; the energy moved by exactly that, miss `0.0` |
| plant (ii) — MB-pol's short-range two-body polynomial emptied | **FIRES** | carrier `2.176e-2` hartree (`E2b` at the held-out node) nonzero in the two-body sector; `E2b` exactly `0` with the acceptor 40 bohr away. The far TOTAL is `1.068e-6` hartree — the long-range electrostatic tail, reported beside the plant and NOT inside its floor, which is exactly the correction the freeze made before the plant ran: a plant on the total would have failed at `1.07e-6` against a `1e-9` floor, for a reason that is physics |

**A cross-check the freeze did not ask for, and that could have failed.** Noticed after the
freeze and reported as a check rather than promoted to a gate: CT-3's C1 counted how many of
these same sixty-four map nodes its re-fit law lands inside CT-2's S1 tolerance, and named the
ten it misses and the worst of them. This lane rebuilt that law independently from the records,
so the count, the miss LIST and the worst miss are all falsifiable against `ct3/gate.json`.
**54 of 64, the same ten misses in the same order, and the worst miss `1.1637652059480e-2`
against CT-3's `1.163765205946e-2` — a relative difference of `1.7e-12`.** G-X0 anchors one
geometry; this anchors sixty-four.

## 2. The score, by family

Every number in hartree. `depth` is the family's mean `ΔE_exact`, printed beside the errors
because a small error on a shallow bond is not accuracy (M-BASE-RATE-OMITTED). `served` is the
CT-3 law as `ct3/wall_ct3.json` serves it; `TIP4P remap` is the primary placement rule and
`TIP4P as-is` the sensitivity leg; `interpolant` is the leave-one-out gauge, which is the
INTERPOLANT's error alone and is a component of `served`, not a rival to it. **`MB-pol RMS` is
not a model's error: it is the HAMILTONIAN's**, MB-pol against the exact minimal-basis answer,
and it belongs in this table only so the reader can see it dwarf the rest. It is unpacked in §5.

| family | n | mean depth | served RMS | served max | MB-pol RMS | TIP4P remap RMS | TIP4P as-is RMS | interpolant LOO RMS |
|---|---|---|---|---|---|---|---|---|
| linear | 7 | `+8.4e-4` | `4.457e-3` | `1.164e-2` (`linear_R2.3`) | `1.917e-3` | `4.631e-3` | `9.546e-3` | `6.384e-4` |
| tilt | 25 | `+4.75e-3` | `1.501e-3` | `5.481e-3` (`tilt_R2.7_t180`) | `4.810e-3` | `1.121e-2` | `1.017e-2` | `1.475e-4` |
| twist | 18 | `+1.76e-2` | `2.035e-3` | `5.500e-3` (`twist_R3.0_t120`) | `1.206e-2` | `2.472e-2` | `2.052e-2` | `6.822e-4` |
| donor | 9 | `−1.39e-3` | `8.947e-4` | `1.577e-3` (`donor_R2.9_b90`) | `1.128e-3` | `2.460e-3` | `4.110e-3` | `2.20e-5` |
| dbent | 3 | `−1.80e-3` | `7.567e-4` | `1.131e-3` | `6.59e-4` | `8.747e-4` | `2.224e-3` | `6.47e-5` |
| twistbent | 3 | `−2.67e-3` | `1.501e-3` | `2.583e-3` | `1.340e-3` | `3.247e-3` | `6.775e-3` | `9.99e-4` |
| **ALL** | **65** | `+6.39e-3` | **`2.096e-3`** | `1.164e-2` | **`7.059e-3`** | **`1.487e-2`** | `1.307e-2` | **`4.769e-4`** |
| **BOUND** (`ΔE_exact < 0`) | **37** | `−3.70e-3` | **`6.740e-4`** | `2.583e-3` | **`1.599e-3`** | **`3.216e-3`** | `6.308e-3` | `2.948e-4` |
| UNBOUND (`ΔE_exact ≥ 0`) | 28 | `+1.97e-2` | `3.099e-3` | `1.164e-2` | `1.060e-2` | `2.235e-2` | `1.855e-2` | `6.428e-4` |
| **HELD OUT** (1 node) | **1** | `−2.75e-3` | **`2.583e-3`** | | **`3.76e-5`** | **`3.510e-3`** | `8.841e-3` | `1.723e-3` |

The served law's mean SIGNED error over all sixty-five is `+6.3e-6` hartree — it is centred,
which is what a weighted least-squares residual on its own fit set is. TIP4P/2005's is
`−6.96e-3`: on this Hamiltonian's geometries it systematically OVER-binds, which is exactly
what a model fitted to real liquid water should do against a minimal basis.

**The fences, before the ratio is quoted anywhere.**

1. **In-sample against out-of-sample.** Sixty-four of the sixty-five are the served law's own
   fit set: the table is EXACT at every one of them by construction, and C1's two contact
   amplitudes were fitted on their residual. TIP4P/2005 saw none of them and was fitted to
   liquid water. The all-65 ratio is therefore not a like-for-like accuracy statement. The one
   honest out-of-sample number is the held-out node: served `2.583e-3` against TIP4P/2005's
   `3.510e-3`, a margin of **1.36×**, on ONE geometry.
2. **The truth is STO-3G.** Every error above is against a full CI solve in the MINIMAL basis.
   Both reference models are built for real water, so neither is being scored on what it was
   made for. TIP4P/2005 losing here is not evidence that TIP4P/2005 is a worse model of water.
3. **The Hamiltonian barely binds.** Of the sixty-five, the minimal-basis Hamiltonian binds
   **37** and leaves **28** net repulsive, the most repulsive by `+1.388e-1` hartree
   (`twist_R2.7_t120`, the two hydrogens jammed). The `BOUND` row is the physically meaningful
   one and it is the row a reader should quote.

## 3. Where the served law's error lives

The five worst served errors are all on strongly REPULSIVE geometries, where every channel is
large and the error is a few percent of the largest of them (mHa):

| node | exact | served | error | charges | wall O–O | wall H–O | wall H–H | contact H–O | contact H–H | table CT |
|---|---|---|---|---|---|---|---|---|---|---|
| `linear_R2.3` | `+23.310` | `+34.948` | `+11.638` | `−9.66` | `27.96` | `114.84` | `3.00` | `−60.98` | `−0.14` | `−40.07` |
| `twist_R3.0_t120` | `+44.969` | `+50.470` | `+5.500` | `6.86` | `1.17` | `11.72` | `59.52` | `−5.02` | `−9.55` | `−14.23` |
| `tilt_R2.7_t180` | `+53.275` | `+47.794` | `−5.481` | `5.26` | `4.56` | `27.48` | `49.90` | `−12.57` | `−5.93` | `−20.91` |
| `twist_R2.7_t180` | `+51.469` | `+46.676` | `−4.793` | `4.68` | `4.56` | `27.48` | `49.32` | `−12.57` | `−5.89` | `−20.91` |
| `tilt_R2.7_t150` | `+35.906` | `+32.153` | `−3.753` | `2.98` | `4.56` | `25.36` | `33.93` | `−11.66` | `−3.54` | `−19.48` |

`linear_R2.3` is the innermost node of the map and it carries 0.487 of the whole squared error
on 0.108 of the geometries: at `2.3` Å the H–O wall is `114.84` mHa and the H–O contact term
`−60.98`, so an `11.6` mHa miss is a 10 % disagreement between two terms that nearly cancel.
This is CT-3's C1 worst-miss node, unchanged, and it is where the law is being asked to work
furthest inside its own data.

## 4. The interpolant's error, apart

An interpolant interpolates: the CT-3 table's in-sample residual against `E_CT` is an exact
zero at all sixty-four map nodes, and quoting that as its error would be
M-TRUNCATION-AS-ERRORBAR. The gauges are:

| gauge | RMS | worst | at |
|---|---|---|---|
| leave-one-out over the map's 60 sites | `4.769e-4` | `2.277e-3` | `twist_R2.7_t120` |
| the same over the BOUND subset | `2.948e-4` | `1.723e-3` | the held-out node |
| the ONE node held out from the table, direct | — | `1.723e-3` | `twistbent_R2.7_tw20_t85_d0` |
| CT-2's fitted FAMILY on the same geometries, direct | `2.489e-3` | `9.031e-3` | — |

Two readings follow. **The table still beats the family it replaced**, by 5.2× in RMS across the
map and by 2.49× on the held-out node (`1.723e-3` against `4.294e-3`) — CT-3's S1, carried here
on all sixty-five rather than on one. The 5.2× is understated, not flattered: CT-2's family was
fitted on all sixty-four of these nodes, so `2.489e-3` is its IN-sample error, while `4.769e-4`
is the table's OUT-of-sample leave-one-out. The comparison is handicapped in the family's favour
and the table wins it anyway. And **the interpolant is not the binding term**:
its leave-one-out RMS is `0.23` of the served law's own RMS, so branch 2 of the freeze's §3
(`RMS(LOO) ≥ 0.5 · RMS(served)`) does **not** fire. Where the law is worst is not where the map
is thinnest — `linear_R2.3`'s leave-one-out miss is `1.680e-3` against a served error of
`1.164e-2`, one seventh of it.

## 5. The Hamiltonian's error, apart

MB-pol against the exact FCI/STO-3G interaction energy on the same geometry. **No model on this
record is charged with this number**: it is the minimal basis's own error, and it is what the
basis lane (I-5) exists to close. MB-pol here is the Paesani group's own OpenMM plugin
(`paesanilab/mbpol 1.1.2` on `openmm 7.2.2`), and the reading is *MB-pol as a stand-in for
reality*, which is what a potential fitted to CCSD(T) two-body energies in a large basis is
best available for — not a claim that MB-pol is right.

| subset | n | Hamiltonian gap RMS | worst | at | mean signed |
|---|---|---|---|---|---|
| all | 65 | `7.059e-3` | `4.082e-2` | `twist_R2.7_t120` | `−3.267e-3` |
| **bound** | 37 | **`1.599e-3`** | `2.607e-3` | `twist_R3.4_t0` | `−1.127e-3` |
| unbound | 28 | `1.060e-2` | `4.082e-2` | `twist_R2.7_t120` | `−6.094e-3` |
| held out | 1 | `3.76e-5` | | | `−3.76e-5` |

Three readings.

**The gap is bigger than the law's error, and that is the whole finding.** `7.059e-3` against
the served law's `2.096e-3` over all sixty-five; `1.599e-3` against `6.740e-4` over the bound
ones. The ratio is `3.4×` and `2.4×`. The programme's law is nearer its own Hamiltonian than
that Hamiltonian is to real water — by a factor of two to three — which is exactly the
situation in which the next useful thing to change is the Hamiltonian.

**The gap is a SIGNED bias, not a scatter.** The mean is `−3.267e-3` over all sixty-five and
`−1.127e-3` over the bound ones: MB-pol binds MORE than the minimal basis does, everywhere.
That is the missing dispersion and the missing basis-set flexibility, and it is the shape a
minimal basis is expected to show.

**It concentrates where the minimal basis is worst.** On the bound geometries the gap is
`1.6` mHa, about `1` kcal/mol; on the repulsive ones it is `10.6` mHa and reaches `40.8` mHa
where two hydrogens are jammed (`twist_R2.7_t120`, exact `+138.8` mHa). And on the held-out
node it is `3.76e-5` hartree — MB-pol and a full CI in a minimal basis agree there to
`0.02` kcal/mol, which is a coincidence at one geometry and is reported as one, not as
evidence that the basis is adequate.

## 6. Forces

**No record in `ct1/`, `ct2/` or `ct3/` carries a force.** The exact solves stored energies
only. So nothing on this record is scored against a true force, and exact (FCI/STO-3G) forces
on at least the held-out node and one node per family are **OWED** and named as owed.

What IS on the record: the served law's ANALYTIC interaction force on all six atoms of every
geometry (`scores.json`, `served_force`, hartree/bohr), gated by G-F0 as the derivative of the
served energy, with the analytic forces' translation sum at `1.95e-17`. TIP4P/2005 is rigid and
runs on a remapped monomer, so an atom-wise comparison to it is meaningless; the NET FORCE ON
THE ACCEPTOR molecule is the quantity a rigid model and a flexible one can both be asked for,
and it is recorded for both:

| | RMS over the 65 geometries (hartree/bohr) |
|---|---|
| the served law's net force on the acceptor | `4.188e-2` |
| TIP4P/2005 (remap)'s | `1.910e-2` |
| the difference between them | `2.691e-2` |

The served law pushes about twice as hard as the classical reference and the two disagree by
about as much as the classical one pushes at all. That is a model-against-model reading with no
truth behind it, and it stays one until the owed exact forces exist.

Against MB-pol, which is a flexible model on the same six atoms and so can be compared
atom-wise:

| | RMS over the 65 geometries (hartree/bohr) |
|---|---|
| MB-pol's own interaction force, per atom | `8.126e-3` |
| the served law minus MB-pol, per atom | `6.519e-3` (worst `4.355e-2` at `twist_R2.7_t120`) |
| MB-pol's net force on the acceptor | `2.891e-2` |
| the served law minus MB-pol, net on the acceptor | `1.383e-2` |
| TIP4P/2005 minus MB-pol, net on the acceptor | `1.693e-2` |

The served law's per-atom disagreement with MB-pol is `0.80` of MB-pol's own force magnitude,
and on the net molecular force the served law is nearer MB-pol than TIP4P/2005 is (`1.383e-2`
against `1.693e-2`) — a `1.22×` margin, far smaller than the energy margin and, again, a
model-against-model reading with no truth behind it. MB-pol's force reading carries its own
checks: the six interaction forces sum to `9.1e-12` kJ/mol/nm and they reproduce a central
difference of the interaction energy to `6.7e-4` kJ/mol/nm.

## 7. The cost of one reading

| | measured |
|---|---|
| ONE point of the truth (the exact solve, `E_exact` + the closed sector) | `2,653`–`5,151` core-seconds, mean `3,471`, from the records' own `cpu_seconds` |
| the served law on all 65 geometries, with 65×2 scene constructions, the far reference on each, and 3 finite-difference nodes at 36 coordinates each | `397` seconds wall, `499` core-seconds |
| TIP4P/2005 on all 65 geometries, both placement rules, plus the 65×3 finite-difference force check | `0.070` seconds |
| MB-pol (the OpenMM plugin) on all 65 geometries, plus the plant's two extra evaluations and a 36-coordinate finite-difference force check | `4.94` seconds |

The served law's wall time here is dominated by the engine's SCENE CONSTRUCTION — one `Sim` per
evaluation, twice per geometry for the far reference — and not by the law's arithmetic. It is
reported as measured and it is **not** a per-evaluation cost model for the liquid; the cost lane
owns that number and the second review's step 2 is exactly the instrumentation that would give
it. What the row is good for is the ratio that motivates the whole programme: one point of
truth costs about `3.5e3` core-seconds and one point of a classical reference costs about
`1e-3` — a factor near `3e6`.

## 8. Which branch fired

**Branch 1 — a larger basis (the I-5 lane).** It is the first test in the freeze's order and it
fires: `RMS(MB-pol − ΔE_exact) = 7.059e-3 > RMS(served − ΔE_exact) = 2.096e-3`. On the bound
subset it fires the same way, `1.599e-3 > 6.740e-4`. The other three were computed anyway and
are on the record, because a branch that is never evaluated cannot be said not to have fired:

| branch | criterion | measured | fires |
|---|---|---|---|
| 1 — a larger basis | `RMS(MB-pol − exact) > RMS(served − exact)` | `7.059e-3` vs `2.096e-3` | **YES** |
| 2 — more table coverage | `RMS(LOO) ≥ 0.5 · RMS(served)` | `4.769e-4` vs `1.048e-3` | no |
| 3 — a better angular representation | no family over `0.5` of the squared error while under `0.5` of the geometries | worst is `linear` at `0.487` of the error on `0.108` of the geometries | (would have) |
| 4 — many-body polarisation | if none of the above | — | a REFERRAL only |

Two of those deserve a sentence. **Branch 2 does not fire and it is not close**: the
interpolant's leave-one-out RMS is `0.23` of the served law's error, so the map's coverage is
not what is holding the law back — which is worth saying because CT-3's own results named more
nodes as the next move, and this measurement says they would be a second-order improvement to a
law whose first-order limit is its Hamiltonian. **Branch 3's criterion sits at `0.487` against a
`0.5` threshold** — one node, `linear_R2.3`, is `0.487` of the whole squared error on its own.
That is a knife-edge and it is reported as one; had branch 1 not fired first, this campaign
would have had to say that its decision rested on a threshold it nearly crossed.

What this does NOT license: any statement about many-body polarisation. Branch 4 is a referral
by the freeze's own letter and a dimer set cannot see a three-body term at all — MB-pol's
own three-body sector is exactly `0.0` on all sixty-five geometries here, which is the
measurement that proves the point rather than an argument for it.

## 9. Bookkeeping, declared

* The freeze was committed ALONE before the reference models were run, and corrected once —
  before either plant ran — with the correction written into the freeze itself rather than into
  a postmortem: plant (ii) gates MB-pol's short-range two-body POLYNOMIAL, not its total,
  because MB-pol's electrostatics are long-ranged and a dipole–dipole tail at 40 bohr is real.
* This lane's instrument computed the served law's totals BEFORE the freeze was written — the
  instrument had to exist for the freeze to be able to state a derived stake at all — and the
  freeze says so in §0 in those words. Phase 1 is therefore a SCORING OF RECORDS and rule 6
  gives it no support weight. What was committed in advance is the decision rule of §3.
* G-F0's first cut read a relative `5.104e-6` against a `1e-6` bar and was recorded as a FAIL.
  The bar was wrong, not the force: the scale was the component being differenced, and a force
  component passes through zero. Re-stated as an ABSOLUTE bar with the relative reading scaled
  by the node's own largest component, it reads `2.343e-11` and passes with four orders to
  spare. The first reading is kept here rather than deleted.
* `references.py`'s Coulomb force had a sign error in its first draft, caught by writing G-P1
  before running it. The gate stays in the freeze precisely because that error is invisible in
  the energy and would have been invisible in every table above.
* The thirteen geometries of record take their exact total from `ct2/gd0_*.json` and their
  closed-sector total from `ct1/sector_*.json`, CT-3's own rule, never a third source.
* **MB-pol came with four traps and every one was caught by a check rather than by reading.**
  (i) The plugin's residue template needs the massless `M` site AND the two O–H bonds, or
  `createSystem` refuses. (ii) `Context.computeVirtualSites()` must be called or every energy
  reads `nan` — a failure that announces itself. (iii) The state's force rows leave a STALE,
  non-zero value on the `M` site; distributing it onto the parents was tried FIRST, on the
  reasonable belief that OpenMM had not done so, and it broke the translation sum by exactly
  the M rows. The six real atoms already sum to zero, and the settling evidence is a central
  difference of the interaction energy, not a belief about what OpenMM does. (iv) The plugin's
  four forces arrive as base-class `Force` proxies: the class name, `isinstance` against the
  plugin's own SWIG classes, and `XmlSerializer` ALL fail to tell them apart. The first version
  keyed on the class name, so all four collapsed into one group, every MB-pol sector read zero,
  and **plant (ii) failed silently for want of a carrier** — an absence-shaped defect that
  passed its own check. They are now labelled by `mbpol.xml`'s declaration order and the
  labelling is verified by two facts that must hold and are measured: the three-body sector is
  exactly `0.0` on a dimer, and the one-body sector cancels in the interaction to `3.6e-15`.
* **The MBX cross-check is OWED.** MBX is the group's current C++ library and it is the
  implementation this campaign would rather have used; it reached 102 of 253 translation units
  before its three-body degree-4 units exhausted a shared 31 GiB machine, and the build was
  stopped rather than left to invite the OOM killer onto another lane. The two implementations
  agreeing (or not) is a real check and it has not been made. `REFERENCE_INSTALL.md` is the
  reproduction route.
* No number enters from memory. The engine's numbers are the engine's; TIP4P/2005's five
  parameters and two geometry values are the 2005 paper's and the citation travels with them
  into `references.json`; every unit conversion is `scipy.constants`' (CODATA); MB-pol's
  numbers, where present, are MBX's own, converted by MBX's own constants.
* **A candidate for the misfit registry, not registered here** (the registry is a shared file
  and this lane does not own it): *a rigid reference model evaluated on geometries relaxed
  under a different Hamiltonian carries a PLACEMENT error that can exceed the error being
  measured.* Here it does so on 53 of 65 geometries. The shape is general — it will recur
  every time this programme scores a rigid model against a flexible record — and G-P0 is the
  form of the gate that catches it.
