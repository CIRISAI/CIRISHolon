# RUNG 1 — THE H-BOND NETWORK TIER — prereg

*Frozen 2026-09-02, before any rung-1 instrument existed and before any network reading
was taken on any trajectory. This document stakes the charts, the window, the budgets,
the controls, the plants and the meaning of every possible answer. Nothing below was
written after seeing a network reading; the instrument is written after this file and the
git history is the check.*

**misfits:** M-VACUOUS-SUCCESS, M-FIXED-POINT-TRAJECTORY, M-FINAL-VIEW-COLLISIONS,
M-BASE-RATE-OMITTED, M-PLANT-OBS, M-PLANT-SECTOR, M-TAG-AS-PROPERTY, M-MAINTENANCE-LENS,
M-STALE-INSTRUMENT, M-CHEAPER-THAN-ITS-PRICE, M-PLACEMENT-LOTTERY, M-NULL-MISSTAKE,
M-EXIT-DISCRIMINATOR, M-VOLUME-SCALE, M-HOMOG, M-POPULATION-CHOICE, M-SORTS-NOT-SEPARATES,
M-MAX-OVER-SUCCESSES, M-CONJUNCTION-MONOTONE, M-PRESENTATION-VERDICT, M-NONBIJECTIVE-STEP,
M-UNTESTED-GAP, M-PROVENANCE-OVERREACH, M-ONE-MODEL-DELTA, M-LOOP-BLIND,
M-DEVICE-CLASS, M-IDLE-CALIBRATED-TIMEOUT.

---

## 0. Why this exists

`docs/workbench/app.js` carries a four-band scale ladder. The molecular band (~nm) is
LIVE and holds a resolving certificate: the closure census's `CERTIFIED-STRICT` row, an
OH₂ held 893.8 fs past a pre-staked 834 fs window. The next band up — **H-bond network,
~10 nm** — is FENCED, owner "GANTT node G, rung 1", and its stated exit is

> *"a promoted molecular chart admitted by measured closure — the H₂O quotient and a
> derived water–water interaction, per the FSD's own build chain (§7)."*

This campaign is the measurement that either produces that certificate or replaces the
fence's owner-and-exit with a **measured verdict and a named boundary**.

The band flip is not this lane's to perform. `docs/workbench/smoke.mjs` gates it in both
directions: a LIVE band must cite a `certificate` at `path:line` whose line carries the
string `CERTIFIED`, and a FENCED band whose certificate already resolves fails the gate as
"the rung has landed and the flip is owed". **So the only thing that can flip this band is
a banked log line carrying a verdict.** This prereg stakes what would have to be true for
such a line to exist, and stakes equally clearly what happens when it does not.

**This campaign can only wound or harden the network tier. It cannot manufacture it.** If
every chart fails, the correct report is that the H-bond network is not a certified tier
of this engine's dynamics, said at full volume, with the failing chart and the failing
gate named.

---

## 1. THE TIER, THE VIEW, THE MOTION, AND THREE LEGS

From `lean/CIRISHolon/Object.lean` and `lean/CIRISHolon/Tiers.lean`:

```
Closed v T  ≔  ∃ h, v ∘ T = h ∘ v                 witness: `Closed`
Held   v T  ≔  v ∘ T = v                          witness: `Held`
Closed v T ↔ ∀ x y, v x = v y → v (T x) = v (T y) witness: `closed_iff_fiber_invariant`
Held v T → Closed v T                             witness: `held_imp_closed`
(v∘T = h∘v) → (u∘h = g∘u) → (u∘v)∘T = g∘(u∘v)     witness: `closed_comp`
NonFactoring ↔ ¬ Closed                           witness: `nonfactoring_iff_not_closed`
```

* **X** — the micro-state: positions and velocities of all 12 atoms plus the species
  assignment, exactly as the census reads it.
* **T** — ONE GRAIN BOUNDARY, `Sim::step_frame(SUBSTEPS)` with `SUBSTEPS = 64`, the
  engine's own coarse clock. **Inherited from the census unchanged.** This campaign does
  not re-verify that `T` is bijective; that is an inherited and named gap
  (M-NONBIJECTIVE-STEP, M-UNTESTED-GAP), and it is the same `T` under which the molecular
  tier's own certificate was taken, so no comparison in this document turns on it.
* **v_mol** — the MOLECULAR tier's chart: the partition of arena indices induced by the
  engine's own bonded-pair graph (`partition::labels_from_bonds` over the dump's `bonded`
  bits). This is the tier BELOW. It is read, never recomputed from geometry.
* **v_net** — a NETWORK tier chart. §2 defines seven of them precisely.

Three legs, because "there is a persistent network object", "the network chart has its own
law", and "the network tier sits on top of the molecular tier" are three different claims
and the programme has conflated them before.

### Leg A-N — HELD (is there a persistent network object?)

For a candidate structure `C ⊆ O` (a set of oxygen ARENA indices, `|C| ≥ 2`), the view is

```
v_C : X → {0,1},   v_C(x) = 1  iff  C is EXACTLY a component of the undirected
                                    H-bond graph at x
```

"Exactly a component" means every oxygen in `C` is H-bond-connected within `C`, and no
oxygen outside `C` is in `C`'s H-bond component. `Held v_C T` over a window is `v_C ≡ 1`
across it. **By `held_imp_closed` a Held view is a Closed one**, so this leg — and only
this leg — can produce a positive closure verdict with a theorem behind it. It is the same
leg that certified the molecular tier, run at the tier above with no threshold changed.

### Leg B-N — CLOSED (does the network chart carry its own dynamics?)

`closed_iff_fiber_invariant` makes closure fiber-invariance. On a trajectory only the
observed fibers can be tested: collect every pair of frames `(s,t)` with `v_net(s) =
v_net(t)` and ask whether `v_net(s+1) = v_net(t+1)`. A pair where they differ IS a witness
pair in the exact sense of `nonfactoring_iff_not_closed`, exhibited rather than argued.

**Leg B-N cannot prove closure.** Absence of a witness pair on a sampled trajectory is a
failure to refute at the resolution sampled, and is reported as such — never as "Closed".
The positive form of this leg is therefore always BUDGETED (OBJECT.md rule 2: "exact
closure is not expected; budgets are"), and the budget is §3's `G-N4` and `G-N5`.

### Leg F — FACTORS (is this a tier ABOVE the molecular tier, or a second chart of the atoms?)

This leg exists because the band's stated exit says *"a promoted MOLECULAR chart"*, and
`closed_comp` — the theorem that makes a tower a tower — requires the upper chart to be a
function of the lower one: `v_net = u ∘ v_mol`. **The H-bond criterion is geometric.** It
reads continuous positions and angles that the bonded partition has already discarded. So
whether `v_net` factors through `v_mol` is an open empirical question, not a definition,
and it decides what a positive Leg B-N would even mean.

The test has the same normal form as the closure test: collect every pair of frames `(s,t)`
with `v_mol(s) = v_mol(t)` and ask whether `v_net(s) = v_net(t)`. A pair where they differ
is a FACTORISATION WITNESS, exhibited by index. The defect `F` is the fraction of
informative `v_mol`-collisions carrying one.

`F > 0` does not kill a chart. It says the chart is a view of the ATOMIC tier that happens
to be about molecules, not a view of the molecular tier — so `closed_comp` does not apply
to it, the tower does not stack through it, and the band's exit as written is not met by
it. That is a separable finding with its own kill.

---

## 2. THE CHARTS — seven, defined bit-exactly

**The population rule, declared (M-POPULATION-CHOICE).** The chart set is not
hand-assembled. It is: (i) every reading already implemented in a committed engine
artifact that names the network tier's variable — `holon_lens::lens::hbonds` (the
Luzar–Chandler census) and `holon_lens::partition` (the molecular partition); closed under
(ii) the two coarsening operations the tier's own vocabulary supplies — *project out the
mediating hydrogen*, and *take connected components* — and (iii) the identity-forgetting
map from arena masks to formulas. **No chart is added after any reading is taken.** The
set is fixed at seven by this rule and is listed in full below; a chart invented later is
a new prereg, not a rescue.

**The edge set is the lens's, and that is a declared difference from the census.** The
census's rule 1 was "the edge set is the ENGINE's" — its bond bits came from
`Sim::refresh_pairs` and nothing recomputed them. The H-bond edge set has no engine
counterpart: it is computed here by `holon_lens::lens::hbonds`, called UNMODIFIED, with
its three frozen constants `HB_R_OO_BOHR = 6.6140`, `HB_R_OH_BOHR = 4.6298`,
`HB_ANGLE_DEG = 30.0`. No second implementation of it is written anywhere in this
campaign. Its donor assignment (each hydrogen's NEAREST oxygen, ties resolved by
`Iterator::min_by`, which returns the first minimum, over `oxygens` in ascending arena
order) is deterministic and is named here because determinism of the chart is what makes
two implementers agree.

Let `O` = the oxygen arena indices in ascending order (`|O| = 4` on every mixed arm), and
`H` = the hydrogen arena indices in ascending order (`|H| = 8`).

| chart | reading, canonically |
|---|---|
| **C1H — HB-FULL** | the vector of `(donor_o, hydrogen, acceptor_o)` ARENA-index triples returned by `hbonds`, deduplicated, sorted lexicographically ascending |
| **C1 — HB-ADJ** | C1H with the mediating hydrogen projected out: the deduplicated, ascending-sorted vector of `(donor_o, acceptor_o)` arena pairs |
| **C2 — HB-PART** | the canonical partition of `O` under the UNDIRECTED graph `{a,b}` present iff `(a,b) ∈ C1` or `(b,a) ∈ C1`, computed by `partition::labels_from_bonds` over the `|O|`-node index space with pair bits in the engine's own `traj::pair_index(|O|, ·, ·)` enumeration. Reading = the canonical label vector |
| **C3 — HB-COUNT** | the integer `|C1H|` — the number of H-bond records, counting per mediating hydrogen |
| **C4 — MOL-NET-ID** | the pair `(P, E)`: `P` = the ascending-sorted vector of block masks of `v_mol` with `popcount ≥ 1`; `E` = the deduplicated, ascending-sorted vector of unordered oxygen pairs `(a,b)`, `a < b`, such that some H-bond record has `{donor_o, acceptor_o} = {a,b}` **and** `v_mol` puts `a` and `b` in DIFFERENT blocks |
| **C5 — MOL-NET-FORMULA** | C4 with arena identity forgotten: `(F, K)` where `F` = the ascending-sorted multiset of `partition::formula` strings of the blocks of `v_mol`, and `K` = the ascending-sorted multiset of unordered formula pairs `(formula(block(a)), formula(block(b)))` over the pairs in C4's `E` |
| **C6 — MOL-PART** | the molecular chart itself: `partition::key(labels_from_bonds(n, bonded))`. **The tier below.** Reference and negative control, not a candidate |

**C4 and C5 are the charts the band's exit actually names**: a network of promoted
molecules. They exclude INTRA-molecular H-bonds by construction, because a hydrogen bond
between two atoms of the same molecule is not an edge of a network of molecules.

**Keys.** `holon_lens::census::closure_leg` compares readings as `u64`. Readings are
therefore mapped to DENSE SEQUENTIAL IDS in order of first appearance within a
trajectory — injective by construction, so no hash collision can merge two distinct
readings and manufacture a witness pair. The id values carry no meaning; the closure test
uses only equality. `G-N0` gates the injectivity.

**The refinement ladder, and what it forces.** `refinement_removes_collisions` says that
if `w = f ∘ v'` then every collision of the finer view `v'` is a collision of the coarser
`w` — so a coarser chart has AT LEAST AS MANY collisions and is strictly harder to close.
The ladder here is, by construction,

```
C1H  ⊐  C1  ⊐  C2            (C1 = drop the hydrogen; C2 = take components of C1)
C4   ⊐  C5                   (C5 = forget arena identity)
C3                           NOT nested with C1 (C3 counts per hydrogen, C1 does not),
                             stated so no monotonicity is claimed for it
```

and it forces `collisions(C1H) ≤ collisions(C1) ≤ collisions(C2)` and `collisions(C4) ≤
collisions(C5)` on EVERY trajectory. That is a machine-checked prediction about this
instrument's own arithmetic and it is gated as `G-N2`.

**Blindness (M-TAG-AS-PROPERTY).** Every chart function's signature is
`(pos: &[[f64;3]], z: &[u32], bonded: u128) -> Reading`. It cannot reach the seed, the arm
label, the ozone setting, the dE₄ flag, or the launch header. Blindness is enforced by the
type, not by discipline.

---

## 3. THE STAKES

### 3.1 The window `W` — INHERITED, NOT RE-CHOSEN

> **W = 834 fs**, `β = 0.02`, `L_flick = 8.4 fs`.

These are `PREREG_WINDOW_FS`, `PREREG_BETA` and `PREREG_FLICKER_FS` from the census,
byte-for-byte, and they are inherited rather than re-derived **because the census is the
referee and moving its window at the tier above is what a rescue looks like.**

The network tier's own natural reference is recorded here so the reading is interpretable
and so nobody has to invent it later: the Luzar–Chandler hydrogen-bond correlation time in
liquid water is of order 0.5–1 ps, which puts `W = 834 fs` **at** the H-bond lifetime
rather than far below it. A strict pass at this tier is therefore expected to be HARD in a
way it was not at the molecular tier, where `W` bought 91 O–H stretch periods. That is
stated in advance so that a TRANSIENT verdict is read as a measurement and not as a bar
set too high after the fact. **No window is changed by this campaign under any outcome**;
§4's branch (D) pre-commits what happens instead.

### 3.2 The closure budget `δ*`

OBJECT.md rule 2: "exact closure is not expected; budgets are." The census staked
non-expansion and no absolute bound, so this campaign must stake one.

> **δ\* = 0.01** — at most 1% of informative transitions may violate functionality.

**Provenance, and it is a JUDGMENT rather than a derivation.** `holon-render/src/holon.rs`
freezes `CLOSURE_DEFECT_MAX = 1e-2` as the engine's own bar for calling a composite view
autonomous, with the stated reasoning that an isolated bound pair scores at the
integrator's drift level (~1e-5) while a buffeted one scores orders worse, so 1% "sits in
the gap with room on both sides". That constant is in ENERGY-fraction units and this one is
in FUNCTIONAL units; transposing it is an act of judgment, it is declared as one here, and
it is staked before data so that it is a bar rather than a description. The measured defect
is printed for every chart and every seed whatever the verdict, so the distance to the bar
is always visible (M-SORTS-NOT-SEPARATES: if no chart clears δ\*, the report says none
did — it does not promote the smallest to "closest").

### 3.3 Non-expansion

> **`D`(second half) ≤ 1.05 × `D`(first half)`**, on every seed.

OBJECT.md rule 1's own number, inherited from census `G9`. A tier that certifies on its
first half and is out of budget by its second has not certified.

### 3.4 Anti-vacuity — two clauses, because one is not enough

A constant chart is Closed by `h = id` and has said nothing. The census's own H-bond-count
row read defect 0.000 on the hydrogen arm and was labelled **VACUOUS**, not closed; this
stake is that finding turned into a gate (M-FIXED-POINT-TRAJECTORY, M-VACUOUS-SUCCESS).

> **work count ≥ 200 informative transitions** (the census's `PREREG_MIN_INFORMATIVE`,
> unchanged), AND
> **dynamism: ≥ 200 frames at which the reading CHANGES, and ≥ 4 distinct readings.**

The two clauses pull against each other on purpose: a chart that never changes has many
collisions and no dynamism; a chart that changes every frame has dynamism and no
collisions. Only a chart that is both dynamic and revisited is being tested at all. The 200
matches the work count's own number rather than inventing a second; the 4 is the smallest
count at which a reading is not a coin flip, and it is a judgment, declared.

### 3.5 Contamination — does the chart contain its own variable?

In liquid water there are no covalent O–O bonds, so every Luzar–Chandler edge is by
construction between distinct molecules. In THESE scenes the O–O curve has `R_e = 2.4421
bohr` and oxygens aggregate into covalently bonded clusters (`O4H4`, `O3H2`), all of whose
O–O separations sit well within the criterion's `6.6140 bohr` O···O cut. So the criterion
can fire on covalently bonded oxygen pairs, and a chart built from those edges would be a
relabelled covalent skeleton wearing a hydrogen bond's name — exactly M-MAINTENANCE-LENS's
shape, a lens that does not contain the variable it claims to measure.

Define, per trajectory: an H-bond record `(d, h, a)` at frame `t` is **covalent-contaminated**
iff the engine's own bond bit for arena pair `(d, a)` is set at frame `t`.

```
contamination = (Σ_t #contaminated records at t) / (Σ_t #records at t)
```

> **contamination ≤ 0.20**, else the chart is **VOID (chart does not contain its variable)**.
> If the denominator is 0 the chart is **VOID (empty chart)** and is reported separately.

The 0.20 is a judgment, declared: a chart measuring hydrogen bonding should read near zero,
and one edge in five is the most coincidence a small box is granted before the reading is
refused. Both numerator and denominator are printed for every trajectory, pass or fail.

### 3.6 The control floor — the SHUFFLE floor the census staked and did not get to run

`CENSUS_RESULTS.md` §4 convicted its own control floor: a flat 5% pool rate is satisfiable
only by a scene holding at most four molecules of the composition under test, and it gave
H₂ opposite verdicts in two arms of identical physics. Its staked repair was explicit and
forward-looking:

> *"Staked now for the next freeze and not retroactively: the pool pass rate must be
> compared against the pass rate among same-composition blocks in a surrogate whose bond
> graph is time-shuffled within each pair."*

**This is the next freeze on that instrument, so the repair is implemented here rather than
inherited broken** (M-BASE-RATE-OMITTED, and discipline rule 5's permutation floor).

> **The surrogate.** For each of the `|O|·(|O|−1) = 12` DIRECTED oxygen-pair edge series,
> independently draw a uniform circular shift `k ∈ [1, n_frames − 1]` and rotate that
> edge's boolean time series by `k`. This preserves each edge's marginal occupancy and its
> own run-length distribution EXACTLY, and destroys the inter-edge coincidence that makes a
> particular oxygen set a persistent cluster. The one artificial junction a circular shift
> creates is named here and is negligible against 20,000 frames.
>
> **The RNG is deterministic**: splitmix64 seeded with `header.seed ^ 0x52554E473100`, so
> the floor reproduces exactly. `S = 200` surrogates.
>
> **`q`** = the fraction of the 200 surrogates in which ANY oxygen subset of the same size
> as `C` reaches the window (strict or budgeted) under the identical criterion.
>
> **`q ≤ 0.05`** is required, else **VOID (no separation)**.

The census's flat pool rate is computed and printed BESIDE `q` for comparability with the
banked molecular-tier numbers, and is labelled **SUPERSEDED** wherever it appears.

### 3.6a AMENDMENT-1 — before any real reading, because the instrument's own positive control fired

*Written 2026-09-02 with the plants built and run and with NO trajectory read. Git is the
check on the order. The stake is not moved; an internal inconsistency in §3.6 is repaired,
and the repair is toward the census's own words rather than away from them.*

**What fired.** P-7 — the "must certify" control, a two-oxygen H-bond cluster present in
every one of 1400 frames — came back **VOID (no separation)**. The cause is arithmetic, not
physics: the fixture's single edge has occupancy **1.0000**, and a circular shift of an
all-true series is that series. The surrogate IS the data, so the null has zero degrees of
freedom. **A null that conditions on the statistic under test cannot reject, and it cannot
accept either.**

**The inconsistency it exposed, in this document.** §3.6 quotes `CENSUS_RESULTS.md` §4's
staked repair —

> *"the pool pass rate must be compared against the pass rate among same-composition blocks
> in a surrogate whose bond graph is time-shuffled within each pair"*

— and then defines `q` as something the quotation does not say: *the fraction of surrogates
in which ANY structure of the same size reaches the window*. The census's construction is a
**surrogate-referenced POOL RATE**; the paraphrase was an any-structure rate. They are
different statistics and only the first survives an occupancy of 1.

**The repair, which is a return to the quotation and not a new stake.**

> `p_data` = the fraction of the eligible pool — every same-size subset of the oxygen set
> other than the target, exactly enumerated — that reaches the window in the DATA. This is
> the census's own statistic, the one that read 0.000 of 111 on the molecular certificate.
>
> `p_null_p95` = the 95th percentile, over the `S = 200` circular-shift surrogates, of that
> same pool rate recomputed on the shifted edges.
>
> **The bar is `max(p_null_p95, 0.05)`, and the verdict is VOID (no separation) iff
> `p_data > bar`.**

**The staked 0.05 does not move. Its ROLE changes, and that change is the census's own
diagnosis applied.** §4 convicted the flat 5% for being a CEILING — "a cap on how many
molecules the scene may contain ... that gets TIGHTER exactly as a scene becomes more
chemically interesting". Here it is a FLOOR on the bar: the surrogate can raise the bar when
the scene's own edge marginals would make peers pass by chance, and can never lower it below
what the census demanded. On the census's own worked example this is exactly right — six H₂
in twelve hydrogens gave `p_data = 5/65 = 0.077` against a flat 0.05 and were VOIDed, while a
marginal-preserving surrogate produces peer passes at about the same rate and lifts the bar
to meet it.

**Both diagnostics are kept and printed on every row, binding nothing:**

* `max_edge_occupancy` — the largest per-edge occupancy among the target's edges. **This is
  the field that says whether the surrogate had any power.** As it approaches 1 the shift
  stops moving anything; a reader must never take a surrogate-referenced verdict at high
  occupancy as a statement about the structure.
* `q_any` — the any-structure rate §3.6 originally named, reported so the superseded
  statistic is visible beside the one that replaced it rather than quietly dropped.

**What this costs, stated plainly.** The surrogate leg is powerless against a permanently
occupied edge, and no amendment can give it power there — every marginal-preserving null is
degenerate at occupancy 1. So at high occupancy the discriminating work is done entirely by
the POOL rate, which is a comparison against peers rather than against chance. That is a
real weakening of the control relative to what §3.6 promised, it is named here rather than
discovered later, and every certified row must be read with `max_edge_occupancy` beside it.

### 3.7 Moving carrier

Inherited unchanged from census `G5`: internal RMS displacement ≥ **0.1 bohr** in the
structure's own centroid frame, and at least one intra-structure oxygen separation varying
by ≥ **0.05 bohr** across the window. Else **VOID (frozen carrier)**. The same
`census::carrier_motion` function computes it — one implementation, at both tiers.

### 3.8 Stacking value

> **`D`(network chart) ≤ `D`(C6, the molecular chart)** on the same seed.

A coarse tier that predicts its own successor WORSE than the tier beneath it has bought
autonomy with nothing. This is a design judgment, declared, and its warrant is bounded:
`refinement_removes_collisions` licenses the monotonicity claim only where the charts are
genuinely nested (§2's two ladders), and the network charts are NOT coarsenings of C6, so
this gate is a COMPARISON and not an application of that theorem. Stated so the theorem is
not laundered into a place it does not reach.

---

## 4. THE GATES

- **G-N0 — key injectivity.** The reading-to-`u64` map is injective on the observed set for
  every chart and every trajectory; **EXACT**, checked by construction (dense sequential ids
  by first appearance) and asserted. A collision voids the chart, because a merged reading
  manufactures a witness pair. witness: `none (an instrument-internal identity assertion; M-FINAL-VIEW-COLLISIONS is its warrant)`
- **G-ID — instrument identity, EXACT.** This campaign's Leg B-N engine, applied to chart
  C6 on the eight fenced trajectories, must reproduce `CENSUS_RESULTS.md` §11.2's eight
  defects to the 4 decimal places printed there — 0.1128, 0.1328, 0.1339, 0.1453, 0.1410,
  0.0815, 0.1460, 0.1287 — and their eight non-expansion verdicts. Any mismatch means this
  instrument is not the census's and the campaign is VOID until the difference is explained.
  witness: `closed_iff_fiber_invariant`
- **G-N1 — factorisation.** `F`, the fraction of informative `v_mol`-collisions at which the
  network chart disagrees, reported for every chart with witnesses exhibited by frame index.
  `F = 0` is reported as "no factorisation witness at this resolution", never as "factors".
  `F > 0` means `closed_comp` does not apply and the tower does not stack through that chart.
  witness: `closed_comp`
- **G-N2 — ladder monotonicity, EXACT.** `collisions(C1H) ≤ collisions(C1) ≤ collisions(C2)`
  and `collisions(C4) ≤ collisions(C5)` on every trajectory, with no tolerance. A violation
  convicts the instrument, not the physics. witness: `refinement_removes_collisions`
- **G-N3 — contamination.** `contamination ≤ 0.20`, numerator and denominator printed;
  a zero denominator is VOID (empty chart) and is a distinct outcome from a failed ratio.
  witness: `none (a lens-scope gate on a measurement; M-MAINTENANCE-LENS is its warrant)`
- **G-N4 — closure budget.** `D ≤ 0.01` on every seed. witness: `closed_iff_fiber_invariant`
- **G-N5 — non-expansion.** `D`(2nd half) ≤ 1.05 × `D`(1st half) on every seed. witness: `Closed`
- **G-N6 — stacking value.** `D`(network) ≤ `D`(C6) on the same seed. witness: `none (a design comparison, not a theorem; §3.8 states why refinement_removes_collisions does not reach it)`
- **G-N7 — anti-vacuity.** ≥ 200 informative transitions AND ≥ 200 reading changes AND ≥ 4
  distinct readings, else VOID. All three printed whether or not they pass.
  witness: `none (an anti-vacuity gate on a measurement; M-VACUOUS-SUCCESS and M-FIXED-POINT-TRAJECTORY are its warrant)`
- **G-N8 — Leg A-N strict.** A structure is CERTIFIED-STRICT iff `v_C ≡ 1` over ≥ 1 window
  of `W = 834` fs of simulated time. witness: `Held`
- **G-N9 — Leg A-N budgeted.** CERTIFIED-BUDGETED iff `v_C = 1` on ≥ 98% of a window's
  frames and every breach run ≤ 8.4 fs, with the window held at both endpoints.
  witness: `Held`
- **G-N10 — moving carrier.** Internal RMS ≥ 0.1 bohr and ≥ 1 intra-structure separation
  varying ≥ 0.05 bohr, else VOID (frozen carrier).
  witness: `none (an anti-vacuity gate on a measurement; M-FIXED-POINT-TRAJECTORY is its warrant)`
- **G-N11 — shuffle control floor.** `q ≤ 0.05` over `S = 200` circular-shift surrogates,
  else VOID (no separation). The superseded flat pool rate is printed beside it.
  witness: `none (a permutation floor; M-BASE-RATE-OMITTED is its warrant, and CENSUS_RESULTS.md §4 is the defect it repairs)`
- **G-N12 — refusal scope.** A trajectory whose scene holds no oxygen, or fewer than
  2 oxygens, is REFUSED by name — not passed and not failed (OBJECT.md rule 9).
  witness: `none (a scope refusal; OBJECT.md rule 9 is its warrant)`
- **G-N13 — exit discriminator.** Every VOID and every REFUSAL prints WHICH gate produced
  it, in the verdict tag itself, and the discriminating field is read rather than merely
  carried (M-EXIT-DISCRIMINATOR). **EXACT**: no verdict may print without its cause.
  witness: `none (a reporting obligation; M-EXIT-DISCRIMINATOR is its warrant)`

---

## 5. THE BRANCHES — every answer's meaning, staked in advance

A CERTIFIED verdict is a conjunction of eleven gates. **M-CONJUNCTION-MONOTONE applies and
is answered here: the fraction of charts surviving a nested conjunction is non-increasing
for any predicates whatsoever, so "only k of 6 charts survived" is a theorem about
conjunctions and is never quoted in this campaign as evidence about the world.** The report
gives each chart's reading at each gate, not a survival count.

* **BRANCH (A) — CERTIFIED AS A STACKED TIER.** Some chart passes G-N3, G-N4, G-N5, G-N6,
  G-N7 on **all eight** fenced seeds, and shows no factorisation witness (G-N1, `F = 0`).
  → The H-bond network is a closed view of the molecular tier at the stated budget, and
  `closed_comp` stacks it. A certificate line is banked and the flip is owed to
  workbench-engine, **with the scale fence of §7 in the same sentence**.
* **BRANCH (B) — CERTIFIED AS AN ATOMIC-TIER CHART ONLY.** Passes the same gates but G-N1
  exhibits factorisation witnesses. → The chart is closed at budget but is NOT a view of the
  molecular tier; `closed_comp` does not reach it and the band's exit as written — "a
  promoted MOLECULAR chart" — is not met. **The fence stays**, with the measured verdict,
  the chart named, its budget named, and the missing rung named as the factorisation.
  A certificate line is NOT banked, because the band's own exit is not what was measured.
* **BRANCH (C) — A HELD NETWORK OBJECT.** Some structure passes G-N8 (or G-N9) with G-N10
  and G-N11. → A persistent network object exists, and by `held_imp_closed` it is a Closed
  view. Reported exactly as the census reported the OH₂: which structure, which window, how
  many, on how many seeds, with no formation rate. This branch is INDEPENDENT of (A)/(B) and
  may co-occur with any of them; it is the branch that would carry a certificate on the
  strongest available warrant.
* **BRANCH (D) — NOT CLOSED.** Every chart fails G-N4 or G-N5 while passing anti-vacuity and
  contamination. → The network tier is not certified. **The fence stays with a measured
  verdict**: every chart's defect, its non-expansion ratio, its longest held run in fs, and
  the best-performing chart named with its number, so the distance to the bar is visible.
  No window and no budget is moved. *Pre-committed follow-up, not a rescue:* the measured
  longest held runs are published so that a FUTURE prereg can stake a tier-appropriate
  window with the numbers in hand — and that would be a new freeze with its own kills, not
  an amendment to this one.
* **BRANCH (E) — VOID: the scene does not carry the variable.** Contamination > 0.20 on the
  H-bond charts, or anti-vacuity fails on every chart. → No verdict about the network tier
  either way. **The fence stays and its boundary is NAMED**: the certified molecular
  dynamics available contains no hydrogen-bond network to chart, and the named unblocker is
  the T3 scale-up to a scene holding enough water molecules to have one.
  *Pre-committed follow-up:* the same instrument is run on C4/C5, whose inter-molecular edge
  set excludes covalent pairs by construction. If C4/C5 then read constant or empty, THAT is
  the measurement — the count of frames carrying at least one inter-molecular H-bond is
  reported as the quantity, at full volume, and it is a stronger statement than the
  contamination ratio alone.
* **BRANCH (F) — REFUSED.** G-N12 fires (no oxygen, or fewer than two). → Not a verdict.
  The gate whose passing would lift the refusal is named. The hydrogen arm is expected here
  and is run precisely so the instrument is seen to refuse.

**Cross-branch commitments.**

* If a chart certifies on `de4_on` and not on `de4_off` or vice versa, that is ONE seed and
  **no causal sentence is available in either direction** — the census's §12.3 discipline,
  inherited verbatim. It is reported as a one-variable observation and nothing else.
* Any branch that fires on some seeds and not others reports the per-seed table in full.
  A per-seed split is not converted into a rate.
* **A fired kill is reported as plainly as a survival, and stays fired.**

---

## 6. PLANTS

Every plant names its carrier and **the sector it must be nonzero in** (M-PLANT-SECTOR),
and each is checked to FIRE on this instrument before any real reading is trusted
(M-PLANT-OBS).

* **P-1 — must read defect EXACTLY 0.** Carrier: a synthetic 12-atom trajectory (4 O, 8 H)
  whose H-bond adjacency follows a fixed deterministic period-7 cycle of 7 distinct graphs,
  so the successor reading is a function of the reading by construction. Sector the plant
  acts on: the CHART sector — the planted signal is nonzero in the chart's own time series
  (7 distinct readings, changing every frame) and exactly zero in the DEFECT sector.
  Expected: informative ≥ 200, distinct = 7, changes = `n_frames − 1`, defect `0.000000`.
  An instrument that cannot read zero here cannot read a defect.
* **P-2 — must FIRE (planted defect).** Carrier: P-1's trajectory with exactly **13** frames'
  successors overwritten so that one reading acquires a second successor on 13 occasions.
  Sector: the DEFECT sector, nonzero by construction while the chart sector is otherwise
  P-1's. Expected: violation count EXACTLY 13, witness pairs exhibited, and the exhibited
  indices intersect the planted frames. **This is the planted-defect control the rung
  requires and it must be seen to fire before any real number is read.**
* **P-3 — must VOID by work count.** Carrier: a synthetic whose chart reading is a strictly
  increasing counter, every frame a distinct reading. Sector: the READING-MULTIPLICITY
  sector — zero repeats by construction. Expected: informative = 0 → VOID(work count),
  and **not** "closed".
* **P-4 — must VOID by dynamism (frozen chart).** Carrier: a synthetic whose H-bond graph is
  constant across all frames. Sector: the CHANGE sector — exactly zero changes by
  construction, while the chart reads a legal value at every frame. Expected: distinct = 1,
  changes = 0 → VOID(frozen chart), and **not** "closed by `h = id`".
* **P-5 — must VOID by contamination.** Carrier: a synthetic 4-oxygen scene with every
  oxygen pair at 2.4 bohr (the O–O curve's own `R_e`), the engine bond bit SET on every O–O
  pair, and hydrogens placed so the Luzar–Chandler criterion fires. Sector: the CONTAMINATION
  sector — nonzero by construction, every H-bond edge sitting on a covalently bonded pair.
  Expected: contamination `1.000` → VOID(contaminated). This plant is the one that proves
  §3.5's gate is not decoration.
* **P-6 — must REFUSE.** Carrier: the REAL hydrogen-arm trajectory
  `hydrogen/seed_0x0000000053415421.traj`, 12 hydrogens and no oxygen. Sector: the SPECIES
  sector — zero oxygens by construction. Expected: REFUSAL naming the gate, not a pass and
  not a fail.
* **P-7 — must CERTIFY Leg A-N.** Carrier: a synthetic in which a fixed 2-oxygen H-bond
  cluster is present in every frame, with thermal internal motion above the G-N10 floors.
  Sector: the MEMBERSHIP sector — `v_C ≡ 1` by construction. Expected: CERTIFIED-STRICT.
  A Leg A-N that cannot certify this cannot certify anything.
* **P-8 — must REJECT Leg A-N (budget abuse).** Carrier: P-7 with a single breach run of
  `L_flick + 1` frames placed mid-window, total breach fraction below `β`. Sector:
  MEMBERSHIP — the breach run is the plant. Expected: FAIL G-N8 (any breach) and FAIL G-N9
  (run too long) even though the 2% budget alone would pass it.
* **P-9 — the NEGATIVE CONTROL, on real data.** Carrier: chart C6 — the bonded partition —
  on the eight real fenced trajectories, a chart the census MEASURED as NOT CLOSED at
  defects 0.0815–0.1460. Sector: the DEFECT sector, nonzero **by prior measurement** rather
  than by construction, which is what makes it a control on real physics rather than on a
  fixture. Expected: this instrument reproduces those eight numbers (G-ID) and returns NOT
  CLOSED. **This is the chart known not to close, so the instrument can be seen to say no.**

---

## 7. ADMISSIBILITY, SCOPE, AND THE INHERITED CAVEATS

### 7.1 The trajectory set, named by digest

Every trajectory is an EXISTING certified-arm artifact. **No new dynamics is generated by
this campaign**, so no new physics enters through it and the comparison to the molecular
tier's certificate is on the identical data.

All 23 entries of `conformance/water_observatory/census_traj_manifest.sha256` were verified
`OK` against `/home/emoore/holon-artifacts/census-traj/` before this freeze
(M-PROVENANCE-OVERREACH: the digest is a statement about bytes and nothing more — it does
not certify the physics that produced them, which §7.3 handles separately).

| arm | seeds | scene | role |
|---|---|---|---|
| `fenced/` | 8 (`0x…5421`–`0x…5428`) | 8 H + 4 O, 12 atoms, 34.6 × 20.8 bohr, 2D, 20,000 frames | **primary** — the arm the molecular certificate came from, pin `a3b3d4b`, momentum 4.7e-5–7.9e-5 of bound |
| `de4_off/` | 1 (`0x…5422`) | same | one-variable control at commit `21e6be3`, `dE4_evals 0` by the counter |
| `de4_on/` | 1 (`0x…5422`) | same | its matched arm, `dE4_evals 891` by the counter, `\|p\| 4.87e-12/1.04e-7` |
| `hydrogen/` | 8 | 12 H, 0 O | **P-6, the refusal control** |

### 7.2 THE SCALE FENCE — stated before any verdict, because no verdict lifts it

The band is nominally **~10 nm**. These scenes are **34.6 × 20.8 bohr = 1.83 × 1.10 nm**,
two-dimensional, and hold four oxygens. **No outcome of this campaign licenses a claim at
the ~10 nm scale** (M-VOLUME-SCALE, M-HOMOG). What is measurable here is whether the
network CHART is admitted by measured closure on the physics that exists — which is the
band's exit as `app.js` writes it — and any certificate this campaign produces carries the
scene's dimensions in the same sentence as the verdict. The scale axis is a separate and
named obligation (T3 scale-up), owed and not met.

### 7.3 THE INHERITED CAVEATS, named as the brief requires

Any chart built on bond membership inherits these BY NAME, and this one is built on two
edge sets, so it inherits both plus one of its own:

1. **The engine's `bonded` criterion is evaluated on an unconverged curve.**
   `Sim::refresh_pairs` decides `bonded` from `e_rel < 0 && r < r_outer`, the outer turning
   point, which lives in the dissociation tail past ~6 bohr — exactly where the O–O curve's
   solve caps. `CENSUS_RESULTS.md` §13.4 and `PROVENANCE_de4_arms.md` establish that the
   residual there is **not monotone in budget** (2.683e-6 at budget 4000, 4.81e-6 at budget
   5000 — larger effort, larger worst residual, by 1.8× the wrong way), so **the residual
   may not be quoted as an error bar and must always be quoted with its budget**. The
   magnitude: 4.3e-6 Ha in the tail, 0.45% of kT at 300 K, shifting the outer turning point
   by ~3.7e-4 bohr against intra-block separations of 2–6 bohr. Small, real, and within the
   claims rather than beside them. C4, C5 and C6 all read this criterion.
2. **The `(O,H,H)` surface's error concentrates in the near-collinear band.**
   `DE5_RESULTS.md` §B-1 measures it: at `theta ≥ 150°` the median |diff| is 1.4–1.8e-4 Ha
   against 1.7e-6 at `theta < 150°`, two orders apart, and §7b's own post-data correction
   identifies the mechanism as a state crossing rather than SCF convergence or grid distance.
3. **And one this campaign adds, which the census did not carry.** The H-bond charts are the
   FIRST readings in this chain whose edge set is geometric rather than the engine's — a
   30° ANGULAR cut, living in `(O, H, O)` geometry, which is the `(O,O,H)` surface's domain
   rather than `(O,H,H)`'s. That surface is certified and seam-scanned per
   `WORKBENCH_FSD.md`, and **this campaign does not re-audit it**; the dependence is named
   here so that a network reading is never mistaken for one taken on the engine's own bits.
   This is the specific sense in which the network tier's edge set is weaker-provenanced
   than the molecular tier's, and it is stated in advance rather than discovered later.

### 7.4 Work-unit pricing (M-CHEAPER-THAN-ITS-PRICE, M-PLACEMENT-LOTTERY, M-DEVICE-CLASS)

**This campaign makes no cost claim, so no wall clock number appears as a price.** The box
is loaded and shared; any elapsed time printed is a machine observation under contention on
one core class and is labelled as such, never a cost model, never a per-node price, and
never compared across runs. No timeout, grace period or loadavg heuristic gates anything
here (M-IDLE-CALIBRATED-TIMEOUT). The work is priced in work units instead, which are
device-independent:

| unit | count |
|---|---|
| H-bond donor–acceptor tests | 20,000 frames × 8 H × 4 O = **6.4e5** per trajectory; **1.08e7** over 17 admitted trajectories |
| chart evaluations | 7 charts × 20,000 frames × 17 = **2.38e6** |
| shuffle-floor edge rotations | 200 surrogates × 12 edges × 20,000 frames × 10 mixed trajectories = **4.8e8** |

The shuffle floor dominates by three orders and is the only part of this campaign that
needs a detached run; §8 stakes its run discipline.

### 7.5 What this campaign does NOT claim

* It does not claim a hydrogen-bond network formed. It claims a network chart is or is not
  Closed, at a stated budget, on regenerated trajectories of a frozen protocol.
* It does not claim the Luzar–Chandler criterion is correct for these scenes. It inherits
  the committed lens unmodified and measures, in `G-N3`, whether the criterion contains its
  own variable here — which is a question the criterion's authors never had to ask, because
  liquid water has no covalent O–O bonds.
* It does not claim the ~10 nm band. §7.2.
* It does not claim closure. Leg B-N can exhibit witness pairs or fail to find them, and the
  second is reported as a failure to refute at the resolution sampled. Only Leg A-N, through
  `held_imp_closed`, can produce a positive with a theorem behind it.
* It does not re-verify `T`. §1.

---

## 8. RUN DISCIPLINE

The instrument is built and run in an isolated worktree at a pinned commit; the results
document records the instrument's own commit so a later reader can tell which instrument
produced which number (M-STALE-INSTRUMENT). Long runs use `setsid` with a done-marker and a
`RUNG1_RESUME.md`, so a session ending kills narration and never computation. Run-state
markers stay untracked; every log a results document cites is committed record.

No `Markov` or memoryless assumption is placed on the coarse dynamics anywhere in this
design (M-ONE-MODEL-DELTA): `closed_iff_fiber_invariant` tests functionality of the observed
one-step map directly, which is a property of the trajectory and not of a fitted model. No
chart-loop or holonomy claim is made (M-LOOP-BLIND); the re-root loop is not exercised.

**The instrument reuses the census's own functions** — `census::closure_leg`,
`census::strict_window`, `census::budgeted_window`, `census::longest_true_run_fs`,
`census::carrier_motion`, `partition::labels_from_bonds`, `partition::key`,
`partition::formula`, `lens::hbonds` — rather than reimplementing them. `carrier_motion` is
made `pub` with no change to its body; that visibility change is the entire diff to the
census crate's logic, and `G-ID` is the check that nothing else moved.

---

## ADDENDUM-1 — the live readout this certificate implies

### THIS ADDENDUM IS POST-DATA AND SAYS SO IN ITS FIRST LINE

*Written 2026-09-02, **after** the instrument ran and after `RUNG1_RESULTS.md` was banked
(`a32202c`). The requirement it answers reached this lane after the run, with the stated
condition that it be named "before your instrument runs". **That condition cannot now be
met, and backdating it into the frozen text above would be the exact stake-move this
document's whole discipline forbids** — git would show it, and correctly. So it is written
here, dated, labelled, and separated from everything frozen at `683a339`.*

**What this costs, stated rather than glossed.** The requirement's purpose is that "the page
must not shape the claim": name the quantity first, then let the door serve exactly that. An
answer written after the data cannot carry that guarantee, and this one does not claim it.
**For the guarantee to be real it must be named in the NEXT freeze, before that instrument
exists** — rung 1's next attempt, or rung 2's.

**Why it is nonetheless safe to add here, and the reason is specific:** this names a PAGE
READOUT, not a gate. It enters no verdict, moves no threshold, and cannot change a single
reading in `RUNG1_RESULTS.md`. Had it named a gate, the honest move would have been to
refuse to add it at all.

### The answer: door (b), and it is NOT a scalar

Of the three door shapes offered, this campaign's measurements name **(b), a defect against a
specific coarse VIEW** — with a second field that is mandatory rather than decorative.

**(a) — an AGGREGATE closure defect over the band's rows — SHOULD NOT BE BUILT, and this
campaign is the evidence.** Of 70 chart readings, **36 sit within the closure budget
(`D ≤ 0.01`, some as low as 0.0004) precisely because they barely move** — 0–43 reading
changes across 20,000 frames, 1–6 distinct readings. An aggregate defect over this band would
have displayed "essentially closed" as a live number while measuring nothing whatever. That
is the vacuous-success shape rendered on a page, which is what `WORKBENCH_FSD.md` WB-7.1
exists to forbid. The recommendation against (a) is the single most useful thing this rung
can hand workbench-engine, and it is an argument from measurement rather than from taste.

**(b) — a defect against a named coarse view — BUILD THIS, as a PAIR.** A closure defect
alone is half a readout. On real data the two halves are **exactly disjoint**: every one of
the 36 readings within the budget failed anti-vacuity, and every one of the 32 readings that
cleared anti-vacuity sat outside the budget; **zero readings were both**. A door serving the
defect without the dynamism beside it can therefore only ever show the misleading half. The
readout is:

> **`(closure defect D, reading changes over the window, distinct readings over the window)`**
> for one NAMED view — three numbers, displayed together, never the first alone.

**The view to serve first is the MOLECULAR chart (C6/MOL-PART), not a network chart.** It is
the only chart in the frozen set with a banked, externally cross-validated defect: G-ID
reproduces nine census readings exact to four decimals, including `de4_off` on all three
quantities `CENSUS_RESULTS.md` §13.1 records. **The band that should get a live door is the
one that is already LIVE.** No network-tier door is owed, because this rung banked no
certificate and the band stays fenced.

**(c) — a grain-schedule readout — NOT NAMED, and the reason is a fence, not a preference.**
`grain.rs` states it in its own words: "a `Grain` must be constructed from a period that a
caller measured or derived", and `Grain::from_bridge_family()` is the one named constant,
carrying its provenance. **This campaign measured no cadence at which any network view is
exact**, so naming (c) would be asking workbench-engine to build a door for a schedule
nobody has measured. That is available later if a rung measures one; it is not available now.

### The seam to rung 2, recorded here because it is a design constraint and not a result

Rung 2's composition target is the lattice-gas chart (`Core/Lattice.lean`'s 53 `(N, P)`
sectors, `sector_count`/`sector_dims`; engine side `ciris-sim-core/src/regplus.rs::sector`,
pinned to the Lean theorem by its own test). That chart's fibers are **conserved-label**
classes, and it is closed BY CONSTRUCTION for sector-preserving collisions
(`SectorPreserving.n_eq`/`.p_eq`, with `conserved_descends` carrying a conserved coarse
reading down).

This rung's chart family is defined by a **geometric predicate** — Luzar–Chandler distance
and angle — which is not a conserved label and carries no such guarantee. **Leg F measured
the consequence: it factors through nothing**, with witnesses on 32 of 40 readings against
the molecular chart. So the constraint for whoever takes rung 1 next, stated as evidence
rather than as advice:

> **A network-tier chart intended to compose upward into a conserved-label fiber census
> should itself be built from conserved or near-conserved labels, not from a geometric
> predicate.** This campaign is a measured instance of the failure mode.

**Seam (1) — "avoid design choices needing a bespoke continuum view above you" — is satisfied
VACUOUSLY here, and that is said plainly rather than claimed as a virtue.** This rung
certified nothing, so it composes upward into nothing; there is no bespoke view to avoid
because there is no view.

**Seam (2) — reuse the sector-census instrument — required no change, and the reason is
structural.** This instrument's equivalence classes are of two kinds, neither of which is a
conserved-label fiber census over a fixed local state space: (i) connected components of a
graph, computed by `partition::labels_from_bonds` — **the census's own union-find, reused
verbatim, with no second implementation written anywhere in this campaign**; and (ii)
`k`-subsets of the four-element oxygen set, directly enumerated for the control floor's base
rate. There was no site at which `sector()` was the right instrument and a new one was
written instead.
