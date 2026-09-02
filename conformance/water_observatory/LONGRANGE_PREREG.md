# THE LONG-RANGE RESIDUAL AUDIT — prereg (GANTT node B1)

*Frozen 2026-09-01, before `longrange_audit` existed and before any residual was
computed. Every number below is either a committed constant (with its file and
line), a figure printed in an already-committed arm log, or a threshold staked
here. Nothing in this file was written after seeing a residual. The git history
of `lane/longrange-audit` is the check: this document lands in its own commit,
ahead of the instrument's.*

**misfits:** M-BUDGET-LAUNDER, M-STALE-INSTRUMENT, M-PROVENANCE-OVERREACH,
M-HOMOG, M-VOLUME-SCALE, M-VACUOUS-SUCCESS, M-MAX-OVER-SUCCESSES,
M-PLANT-OBS, M-PLANT-SECTOR, M-CHEAPER-THAN-ITS-PRICE, M-UNTESTED-GAP,
M-KINEMATIC-NONLOCAL, M-EXIT-DISCRIMINATOR.

Why each is contacted, one line apiece, so the citation is a claim and not
decoration:

| id | the contact |
|---|---|
| M-BUDGET-LAUNDER | a class whose trajectory arm is absent VOIDs loudly and is never scored; the VOID structure is printed beside the scored classes |
| M-STALE-INSTRUMENT | the parked artifacts were written by an OLDER commit than HEAD's runner (§1.4); the instrument's own commit and every artifact path are pinned |
| M-PROVENANCE-OVERREACH | the sha256 refusal names the FILE it hashed and nothing it infers about which run produced it |
| M-HOMOG | this is a locality audit; the scene is a 12-atom box with no bulk, so "distant" here is bounded by a box diagonal and not by a thermodynamic limit |
| M-VOLUME-SCALE | the discarded energy is a sum over pairs and the pair count grows as N²; the N-scaling is NOT measured here and is recorded as owed |
| M-VACUOUS-SUCCESS | the instrument asserts its WORK COUNT (frames scored, pairs beyond the cutoff) and a reading with an empty beyond-cutoff population is VOID, not a pass |
| M-MAX-OVER-SUCCESSES | the verdict is a max over frames, so a refused or unreadable frame may never be silently dropped out of the max |
| M-PLANT-OBS | both plants are re-derived for THIS instrument and pre-checked to fire (§9) |
| M-PLANT-SECTOR | each plant's carrier is stated to be nonzero in the sector the plant acts on (§9) |
| M-CHEAPER-THAN-ITS-PRICE | the pair curves are regenerated, and their cost model is taken from the committed arm logs; a setup that finishes far under it is refused |
| M-UNTESTED-GAP | the staked cutoffs are the radii this engine actually uses, not points interpolated into a hole |
| M-KINEMATIC-NONLOCAL | a locality stake must separate propagation from constraint; here the separation is trivial and stated, because the scene carries no constraint that correlates regions |
| M-EXIT-DISCRIMINATOR | every refusal prints WHY it refused, and the reason is a field of the results table, not a log aside |

---

## 0. The question, in one sentence

The engine's interaction sectors are cutoff-local. **How much pair energy does
that locality discard, per scene class, and is it negligible against the
scene's own energy-drift bound?**

B1 decides nothing about whether an Ewald-class subsystem gets built. The
measurement decides it, through the branches in §7.

---

## 1. WHAT THE ENGINE ACTUALLY DOES — read from committed source, before measuring

This section is the part of the audit that could have been got wrong by
assuming, so it is written down first with its citations.

### 1.1 The pair sector of the census scenes is COMPLETE, not truncated

`engine/crates/holon-render/examples/waterquench_traj.rs` never calls
`Sim::set_pair_cutoff`. So `Sim::pair_switch` is `None`, and
`engine/crates/holon-render/src/sim.rs:2382` takes the first of its two routes:

> `no pair cutoff declared -> the complete N²/2 sum [...] a scene that has not
> declared a truncation budget does not get truncated behind its back`

**In the parked census scenes the pair sector discards exactly nothing.** That
is a fact about these scenes and not about the engine: `set_pair_cutoff` exists,
derives its window from the curves at a declared budget
(`sim.rs:1550 derive_pair_cutoff`), and is the O(N) route any scaled-up scene
must take. B1's question is therefore necessarily a COUNTERFACTUAL one, and it
is stated as such in §2: what WOULD the engine's own truncation machinery
discard on these configurations, at the radii this engine actually uses.

Stating it as a counterfactual is not a weakening. It is the only form in which
the question has an answer, and it is the form B2 needs: B2 would be built to
pay a bill that the complete sum is currently paying in O(N²) work.

### 1.2 The radius the engine is already local at

`Sim::list_cutoff` (`sim.rs:1521`) is `max(three_body_cutoff, four_body_cutoff,
r_cut)`. At the commit that produced the parked artifacts (§1.4) the loaded
surfaces are the H₃ trimer, the (O,H,H) water table and (O,O,H):

| radius | constant | value, bohr |
|---|---|---|
| trimer | `holon_chem::trimer::R_HI` | 9.0 |
| (O,H,H) water | `holon_chem::water::R_HI` | 15.0 |
| (O,O,H) | `holon_chem::ooh::R_HI` | 14.0 |
| (O,O,O), served arm only | `holon_chem::ozone::R_HI` | 14.0 |
| four-body switch | `holon_render::sim::DE4_R_CUT` | 6.0 (not loaded at that commit) |

so `three_body_cutoff() = 15.0` and, with no pair cutoff and no four-body term,
`list_cutoff() = 15.0` bohr **for both scored classes** — the trimer/water/ooh
tables are loaded regardless of which nuclei are in the box, because
`three_body_cutoff` reads `.loaded` flags and not the scene's composition.

**15.0 bohr is therefore the radius this engine's cell decomposition is already
built at, and it is the primary staked cutoff `c*`.**

The many-body surfaces are *exactly* zero outside their domains — `sim.rs:1481`
says so in as many words ("Not a tuning parameter and not a truncation") — so
they are `DependsWithinExact` and discard nothing. witness: `DependsWithinExact`
The pair curve is the only sector with a tail, because a table's edge is an
exponential and never an exact zero (`sim.rs:532`). This audit measures the pair
sector and says nothing about whether the many-body DOMAINS are wide enough;
that is a different question with a different instrument.

### 1.3 The boundary is WALLS, so the minimum image is the identity here

`waterquench_traj.rs:507` sets `Boundary::Walls`, not `Boundary::Periodic`. The
scene has no images, so `Geom::delta` is `b − a` and the minimum-image
convention degenerates. The estimator in §3 is nevertheless written
minimum-image-general, so that it transfers unchanged to a periodic scene; on
these artifacts the two agree bit for bit because they are the same arithmetic.

Recorded because it matters for reading the answer: a walled box of
34.6 × 20.8 bohr has a maximum pair separation of
√(34.6² + 20.8²) = 40.37 bohr, and `pbc_ok` would REFUSE `c* = 15.0` on a
periodic box of this size (`0.5 · 20.8 = 10.4 < 15.0`). A periodic version of
this scene could not legally declare the cutoff this one is already local at.
That is the reason `10.4` is on the staked ladder in §4.

### 1.4 The parked artifacts are OLDER than HEAD's runner (M-STALE-INSTRUMENT)

The committed arm logs' header line reads

```
# arm = hydrogen control (12 H)   ozone = fenced   seeds = 8   12 atoms in 34.6 x 20.8 bohr
```

with no `dE4 = ` field. HEAD's `waterquench_traj.rs` prints one. The producing
commit is therefore `4bec9e2` ("T3: the sixteen-atom cap is gone…"), whose
version of the file loads trimer + water + ooh and has no four-body term at all.
`b455cd9` added the `--de4` argument afterwards.

Consequences, both carried into the results document:

* the parked trajectories were integrated with `four_body_cutoff() = 0`, which
  is why §1.2 reads `list_cutoff() = 15.0` and not `max(15.0, 6.0)`;
* the pair CURVES are generated by `holon_chem::pair::generate_pair_table`,
  which is not touched by either of those commits — so regenerating them at
  HEAD is expected to reproduce the logged curve, and G2 tests exactly that
  rather than assuming it.

---

## 2. THE QUESTION, PER SCENE CLASS

| class | arm directory | composition | seeds parked | status |
|---|---|---|---|---|
| **CLASS-H** — pure-hydrogen gas | `census-traj/hydrogen/` | 12 H | 8 | scored |
| **CLASS-MIX-FENCED** — mixed quench, (O,O,O) fenced | `census-traj/fenced/` | 4 O + 8 H | 8 | scored |
| **CLASS-MIX-SERVED** — mixed quench, (O,O,O) served | `census-traj/served/` | 4 O + 8 H | **0** | **VOID (V1)** |
| **CLASS-O** — oxygen control | not parked | 12 O | **0** | **VOID (V1)** |

For each scored class: *at the radius the engine is already local at, what pair
energy would a declared truncation discard, over the parked window?*

`CLASS-MIX-SERVED` is VOID before the instrument is written, not after: the
directory `/home/emoore/holon-artifacts/census-traj/served/` exists and holds
zero `.traj` files, while `served.log` is in the manifest. That is a missing arm
and it is never scored, never estimated from its sibling, and never quietly
folded into `CLASS-MIX-FENCED` (M-BUDGET-LAUNDER: an absent case reports as absent).

---

## 3. THE ESTIMATOR

For a frame `f` of a trajectory whose atoms carry nuclear charges `z_i`, and for
a cutoff `c`:

```
r_ij         = |x_i − x_j|          minimum image under the scene's own boundary
u_ab(r)      = PairTable::u(r) for the species pair (a,b) = (z_i, z_j),
               from the SAME curves the sim served (§5), asymptote-zeroed
rmax_ab      = PairTable::r_max() for that pair — the last knot

E_band(c)    = Σ over pairs with  c < r_ij ≤ rmax_ab   of  u_ab(r_ij)
E_tail(c)    = Σ over pairs with      r_ij > rmax_ab   of  u_ab(r_ij)
E_hard(c)    = E_band(c) + E_tail(c)

E_switch(c)  = Σ over pairs with r_ij > c − W  of  (1 − S₂(r_ij; c−W, c))·u_ab(r_ij)
               W = holon_render::sim::PAIR_SWITCH_WIDTH = 2.0 bohr
               S₂ = holon_render::cells::switch_c2, the engine's own C² switch
```

`E_hard` is the brief's estimator and is what G1 scores. `E_switch` is what
`set_pair_cutoff` would ACTUALLY drop — the engine truncates with a C² switch
rather than a step, so that the truncated potential is still a potential and the
energy gate does not become a detector of its own truncation (`sim.rs:2219`).
Both are reported for every frame. Where they disagree, `E_hard` is the larger
and is the one gated, which is the conservative direction.

`E_band` and `E_tail` are reported SEPARATELY and never only as their sum,
because they have different epistemic status: `E_band` is the committed table
read inside its own knots, and `E_tail` is the table's edge extrapolation. §4 is
about that difference.

The sums are over every unordered pair of the frame, in arena-index order
`i < j` — the engine's own enumeration order (`traj::pair_index`), so the
floating-point sum is the order the engine would have produced.

---

## 4. **THE TABLE-EDGE FENCE — THE NUMBER IS A LOWER BOUND**

`holon_render::table` matches `U = a·exp(−b·(R − r_edge))` in value and slope at
the last knot (`table.rs:315`). Past `rmax_ab` the table is therefore an
**exponential**.

**The true long-range interaction is not exponential. It is a power law:
dispersion falls as r⁻⁶, and a dipole–dipole term as r⁻³. An exponential decays
faster than every power law, so beyond `rmax_ab` the table UNDERSTATES the
interaction, and `E_hard` — which is built from that table — is a LOWER BOUND on
the energy a real truncation would discard. Every verdict sentence in the
results document carries that clause. A NEGLIGIBLE verdict means "negligible,
and the true value is at least this", never "negligible, and this is the true
value".**

The fence is SIZED rather than left as prose. Two parameter-free comparison
tails are computed and reported beside `E_tail`, each matched to the table's own
last value so that nothing is fitted:

```
E_tail_pow6(c) = Σ over pairs with r_ij > rmax_ab  of  u_ab(rmax_ab) · (rmax_ab / r_ij)^6
E_tail_pow3(c) = Σ over pairs with r_ij > rmax_ab  of  u_ab(rmax_ab) · (rmax_ab / r_ij)^3
```

These are not claims about the physics of these species — the curves are STO-3G
FCI on a minimal basis and a minimal basis does not carry dispersion correctly.
They are the size of the gap the fence covers, stated as a number so a reader
can see whether the lower bound is nearly the answer or nowhere near it. They
are REPORTED, never gated.

---

## 5. THE TABLES: the same curves the sim served

The protocol does not ship curve files; it generates them at run time with
`holon_chem::pair::generate_pair_table(a, b, 96)`, whose knot grid, range
derivation and solver route are committed code. The instrument calls the SAME
function with the SAME knot count, and G2 checks the result against the figures
the arm log printed for that curve.

The check is on the log's PRINTED DIGITS, not on a digest, and the reason is
recorded so it does not read as laxity: an unannounced change to floating-point
summation order downstream can move every pinned digest without moving any
physics, and a digest gate would then fire on a curve that is correct. `R_e` to
1e-4 bohr and `D_e` to 1e-6 Ha are four and six digits of the physics; a curve
that reproduced those and was nevertheless a different curve is not a failure
mode anyone has exhibited.

Curve figures the logs already carry, quoted here so G2 has a target that was
written down before it was tested:

| curve | `R_e`, bohr | `D_e`, Ha | worst residual | log |
|---|---|---|---|---|
| H–H | 1.3887 | 0.204142 | 8.7e-11 | `hydrogen.log:1`, `fenced.log:1` |
| O–H | 1.9909 | 0.122901 | 9.9e-11 | `fenced.log:2` |
| O–O | 2.4421 | 0.147621 | 2.7e-6 | `fenced.log:4` |

The O–O curve's residual exceeds `pair::CONVERGED_RESIDUAL` (1e-9) and the arm
log says so with a `# WARNING` line. That is inherited, not introduced here, and
it is carried into the results document as a stated limitation of
`CLASS-MIX-FENCED` rather than corrected here.

---

## 6. THE STAKED CRITERION AND THE GATES

The reference quantity is the scene's own energy-drift bound,
`Sim::drift_bound()` (`sim.rs:1228`) — derived from the integrator, not tuned:
`DRIFT_SAFETY · 0.25 · ω² · dt² · E_ref`. It is the engine's own statement of how
much energy non-closure it is entitled to. An energy the engine discards that is
small against the energy it already cannot account for is not the thing to build
a subsystem for; one that is large against it is.

`B_s` is the value printed for seed `s` in that arm's committed log (the second
field of `drift <peak>/<bound>`); `D_s` is the peak in the same field.

The staked cutoff ladder — every entry is a committed constant, no interpolation
(M-UNTESTED-GAP):

| `c`, bohr | provenance |
|---|---|
| 6.0 | `sim::DE4_R_CUT` — the tightest radius any sector in this engine uses |
| 9.0 | `trimer::R_HI` |
| 10.4 | `0.5 · BOX_H` — the largest cutoff `Sim::pbc_ok` would admit on a periodic box of this size |
| 14.0 | `ooh::R_HI` = `ozone::R_HI` |
| **15.0 = `c*`** | `water::R_HI` = `three_body_cutoff()` = `list_cutoff()` — **the primary** |
| 41.0 | above the 40.37 bohr box diagonal — the zero control (G6) |

- **G1 — NEGLIGIBILITY, the criterion B1 is staked on.** A class is NEGLIGIBLE iff for every admitted seed `s` and every scored frame `f`, `|E_hard(c*)|(s,f) < 0.10 · B_s`. The max over frames, the seed and the frame index at which it occurs are printed whether the gate passes or fails. witness: `DependsWithinUpTo`
- **G2 — CURVE IDENTITY.** Every regenerated curve reproduces its arm log's printed `R_e` to 1e-4 bohr, `D_e` to 1e-6 Ha, and worst residual to within a factor of 2. A class whose curves fail G2 is VOID, not failed. witness: `none (an equality-of-generation gate has no theorem; it is a measured reproduction against a committed log)`
- **G3 — MANIFEST REFUSAL.** Every `.traj` read is hashed and compared to `conformance/water_observatory/census_traj_manifest.sha256`; a mismatch, or a path the manifest does not list, is REFUSED with the reason and both digests printed, and the file contributes 0 frames. The refusal is demonstrated on a deliberately corrupted copy (plant P1, §9). witness: `none (a provenance gate on an artifact; M-PROVENANCE-OVERREACH is its warrant)`
- **G4 — WORK COUNT.** At least 50 frames scored per scored class, and the count printed for every class including the VOID ones (where it is 0). Below 50 the class is VOID. witness: `none (an anti-vacuity assertion; M-VACUOUS-SUCCESS is its warrant)`
- **G5 — LADDER MONOTONICITY.** For every frame, `|E_hard(c)|` is non-increasing along the staked ladder. 0 violations permitted; a violation means the estimator is defective and VOIDs the whole reading. witness: `dependsWithinUpTo_mono_radius`
- **G6 — ZERO CONTROL.** At `c = 41.0` bohr, above the box diagonal, `E_hard` is EXACT 0.0 in every frame of every class — the instrument reads zero where the scene has nothing beyond the cutoff. A nonzero reading is an index or sign defect and VOIDs the reading. witness: `none (a control zero that is a fact about the SCENE — no pair separation can exceed the box diagonal — not about the instrument's coverage)`
- **G7 — THE BEYOND-CUTOFF POPULATION.** The fraction of pairs with `r > c*` is printed per frame, and at least 1% of scored frames must have a nonempty beyond-cutoff population. A class where nothing was ever excluded has not been tested and is VOID. witness: `none (the eligible-pool rate the reading is drawn from; M-VACUOUS-SUCCESS and M-BASE-RATE-OMITTED are its warrant)`
- **G8 — PLANT P2 FIRES.** The injected-pair plant (§9) moves `E_hard(c*)` by the predicted `u_ab(16.0)` to within 1e-12 relative. witness: `none (a plant-observability check on the estimator; M-PLANT-OBS is its warrant)`

### 6.1 The anti-vacuity companion, and a disclosure

G1's denominator is a BOUND, and a bound can be loose. The ratio `B_s / D_s`
(bound over measured drift peak) is printed for every seed beside every verdict,
and branch (e) of §7 fires where it exceeds 10³.

**Disclosed, because a threshold chosen after looking at data must say so:** the
`drift <peak>/<bound>` fields are already in the committed arm logs and were
read before this freeze. They give `B/D ≈ 84` and `≈ 187` on the first two
`CLASS-H` seeds and `≈ 1.4e5` and `≈ 1.8e5` on the first two `CLASS-MIX-FENCED`
seeds. The 10³ threshold was chosen to separate those two populations. It is a
LABELLING rule only: branch (e) cannot turn a NEGLIGIBLE into a NON-NEGLIGIBLE
or the reverse, and G1's verdict is computed identically with or without it.
What it prevents is a mixed-class pass against a denominator five orders of
magnitude above the energy the run actually failed to conserve being reported as
though it meant something.

---

## 7. THE BRANCHES — every answer's meaning, staked in advance

* **BRANCH (a) — NEGLIGIBLE, BOTH SCORED CLASSES.** G1 passes for `CLASS-H` and
  `CLASS-MIX-FENCED`. → Cutoff-locality at `c* = 15.0` bohr discards less than a
  tenth of the drift bound on these scenes. **B2 is NOT required by this
  measurement.** The sentence carries the lower-bound clause of §4 and the scope
  fence of §10 in the same breath, every time.
* **BRANCH (b) — NON-NEGLIGIBLE IN AT LEAST ONE SCORED CLASS.** G1 fails there.
  → **The B2 Ewald-class requirement FIRES for that class**, the failing seed
  and frame are named, and the measured ratio is the size of the bill B2 has to
  pay.
* **BRANCH (c) — MIXED VERDICT.** One class passes, the other fails. → Recorded
  per class; B2 fires for the failing class only, and the two verdicts are never
  averaged into one. A per-class result is the answer B1 was asked for, not a
  failure to get one.
* **BRANCH (d) — VOID FOR A CLASS.** Any of V1–V5 (§8) fires. → No verdict about
  that class in either direction. The failing condition is named, the class
  appears in the results table with `VOID` in its verdict column and the
  condition in its reason column, and it is counted in the VOID structure at the
  head of the document (M-BUDGET-LAUNDER).
* **BRANCH (e) — PASSES-BUT-UNINFORMATIVE.** G1 passes for a class but
  `B_s / D_s > 10³` on any admitted seed. → The pass is reported as
  `NEGLIGIBLE (uninformative bound)`, with `|E_hard(c*)| / D_s` printed beside
  it. It counts as a pass for B2's purposes — the pre-staked criterion is the
  pre-staked criterion — and the label travels with it wherever the verdict is
  quoted.

**Pre-committed follow-up, designed in rather than rescued in.** If any class
lands in branch (b), the SAME instrument is run at the smaller ladder radii
without changing a threshold, and the radius at which the class first crosses
0.10·B_s is reported. That radius is B2's design input: it is the distance
beyond which this engine cannot afford to be local. No new estimator, no new
denominator, no re-chosen frame set.

---

## 8. VOID CONDITIONS — never scored, never estimated

- **V1 — MISSING ARM.** A class with zero parked `.traj` files is VOID. Known
  before the instrument existed: `CLASS-MIX-SERVED` and `CLASS-O`.
- **V2 — MANIFEST MISMATCH.** A `.traj` whose sha256 differs from the manifest,
  or that the manifest does not list, is REFUSED (G3). A class left with fewer
  than 50 scorable frames by refusals is VOID.
- **V3 — CURVE IDENTITY.** G2 fails for any curve the class needs.
- **V4 — ESTIMATOR DEFECT.** G5 or G6 fails anywhere.
- **V5 — EMPTY POPULATION.** G7 fails: nothing was ever beyond the cutoff, so
  nothing was tested.

A VOID class is never scored, never inferred from a sibling class, and never
reported with a number in its verdict column. The count of VOID classes and the
reason for each appear at the head of the results document, so a pattern of
refusals is visible rather than inferred from an absence.

---

## 9. PLANTS

Both are re-derived for THIS instrument and pre-checked to fire before the real
reading is taken (M-PLANT-OBS), and each names the sector its carrier must be
nonzero in (M-PLANT-SECTOR).

**P1 — the corruption plant, against G3.** A byte-level copy of an admitted
`.traj` is made in scratch and one byte of its atom block is flipped. The
instrument is pointed at the copy and must REFUSE it by name, printing both
digests.
*Carrier:* the file's sha256 digest.
*The sector the plant acts on:* the artifact-identity sector, i.e. the digest
the manifest gate compares. The carrier is **nonzero in** that sector by
construction — a single flipped byte changes sha256 with probability 1 for a
fixed input, so the plant cannot fail to be observable. It is nonzero in NO
other sector the audit reads, which is the point: it must fire on identity and
not on physics.

**P2 — the injected-pair plant, against G8 and the estimator itself.** In an
admitted frame, one atom is displaced so that its separation from a chosen
partner is exactly 16.0 bohr — just outside `c* = 15.0` — with every other atom
untouched. `E_hard(c*)` must change by exactly `u_ab(16.0)` minus that pair's
prior contribution to the sum, to within 1e-12 relative.
*Carrier:* the single pair term `u_ab(16.0)`.
*The sector the plant acts on:* the beyond-cutoff pair sector. The carrier is
**nonzero in** that sector because a table's edge extrapolation is an
exponential and never an exact zero (`sim.rs:532`); the instrument prints
`u_ab(16.0)` and REFUSES to run the plant if it reads 0.0, rather than passing a
plant that could not have fired.

---

## 10. WHAT THIS MEASUREMENT CANNOT SAY

Written before the answer, so that a convenient answer cannot be widened later.

1. **It is the PAIR sector only.** The three- and four-body surfaces return
   exact zeros outside their domains and discard nothing by truncation
   (`DependsWithinExact`, §1.2). Whether those DOMAINS are wide enough is a
   different question, unasked here.
2. **There are no ionic species in these scenes.** Every nucleus is neutral H or
   O and every curve decays exponentially. The r⁻¹ case — the one GANTT says
   makes B2 near-certain once node C ships ionic scenes — is NOT touched by this
   measurement. **A branch (a) verdict is not a statement that B2 is
   unnecessary; it is a statement about neutral scenes at this size.** Any use of
   B1's verdict to defer B2 for ionic scenes is a misuse of it, and this
   paragraph exists to be quoted back.
3. **It is 12 atoms in 34.6 × 20.8 bohr, in two dimensions, walled.** The
   discarded energy is a sum over pairs; the pair count grows as N² while the
   drift bound does not, so the ratio G1 gates is N-dependent and its N-scaling
   is NOT measured here (M-VOLUME-SCALE). Owed, and named as owed.
4. **`E_hard` is a LOWER BOUND** past the table edge (§4).
5. **The scene carries no constraint that correlates separated regions**
   (M-KINEMATIC-NONLOCAL): there is no gauge sector and no projector here, so
   the locality being measured is dynamical propagation through the pair
   potential and nothing else. Stated because the separation is trivial in this
   scene and would not be in an ionic one.
6. **`CLASS-MIX-FENCED`'s O–O curve carries a residual 2.7e-6, above the
   programme's own converged bar** (§5). That is inherited from the parked run
   and it bounds how finely that class's answer can be read.

---

## 11. THE PRICE (M-CHEAPER-THAN-ITS-PRICE)

The cost model is taken from the committed arm logs, which printed each curve's
generation time at the same 96 knots this instrument asks for:

| class | curves needed | priced setup |
|---|---|---|
| CLASS-H | H–H | 3.5–7.0 s (`hydrogen.log:1`, `fenced.log:1`) |
| CLASS-MIX-FENCED | H–H, O–H, O–O | 7.0 + 19.8 + 2596.2 s ≈ 2623 s ≈ 0.73 core-hours (`fenced.log:1–4`) |

Scoring is cheap by comparison: 8 seeds × 20,000 frames × 66 pairs × 6 ladder
entries ≈ 6.3e7 table evaluations per class, which is seconds.

**The refusal:** a `CLASS-MIX-FENCED` run whose curve setup completes in under
1200 s — under half the priced O–O time — is REFUSED as not having generated
that curve, and its reading is void with it. The arithmetic that cannot close is
the falsifying check, and it is stated here so it cannot be waived later.

---

## 12. FRAMES SCORED, AND THE PUBLISHED TABLE

Every frame of every admitted seed is scored: 20,000 frames × 8 seeds =
160,000 frames per class, which exceeds the ≥50-per-class requirement by 3200×.
Scoring all of them removes any question of frame selection.

The results document publishes:

* a per-seed row: seed, frames scored, `max |E_hard(c*)|`, the frame index where
  it occurs, `B_s`, the ratio to `0.10·B_s`, `D_s`, `B_s/D_s`, and the G1 verdict;
* the residual table proper — for a staked stride of every 400th frame (50 frames
  per seed, 400 per class), the columns `frame, class, seed, E_band, E_tail,
  E_hard, E_switch, bound, ratio`;
* the ladder: mean and max `|E_hard(c)|` at each of the six staked radii;
* the fence: `E_tail`, `E_tail_pow6`, `E_tail_pow3` at `c*`;
* the VOID structure, at the head, before any verdict.

The worst frame of each class is named by seed and index, so the claim can be
reproduced on one frame without rerunning the class.
