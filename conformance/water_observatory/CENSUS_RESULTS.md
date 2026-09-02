# THE CLOSURE CENSUS — results

*Stakes: `CENSUS_PREREG.md`, frozen and committed (`2f389e7`) before a line of the
instrument was written. Every threshold quoted here is one of its `PREREG_*` constants.
No stake was moved after seeing a number; where a stake turned out to be wrong that is
written up as a finding (§4) rather than repaired in place.*

---

## 0. THE HEADLINE

*This section was rewritten by its owner on 2026-09-01, folding in the lead's correction
(`c7dd879`) and superseding both it and the original. The original headline said the OH₂
did not exist; that was true of the artifact it examined and false as a general claim, and
what replaced it is not a retraction but a stronger measurement.*

### ROAD ITEM 5 IS MET: a certified-strict OH₂ quotient, on conservation-clean physics

**Seed `0x53415425`, fenced arm, block `0x0a08` — atoms [3, 9, 11], `Z = [8, 1, 1]`.**

| | |
|---|---|
| longest STRICT held run | **893.8 fs** — clears the staked 834 fs window unbroken |
| share of the whole run it was a block | **72.3%** (14,459 of 20,000 frames, 17.5 ps) |
| internal RMS displacement | 0.779 bohr (floor 0.100) — a moving carrier |
| intra-block separation excursion | 0.199 bohr (floor 0.050) — a vibrating O–H, not a passing contact |
| control rate | **0.000** — of 111 other (1 O, 2 H) atom sets, none reaches the window |
| momentum, this seed | \|p\|/bound = **5.70e-5** |
| verdict | **CERTIFIED-STRICT**, and named at the final frame too |

That is `CENSUS_PREREG.md`'s **branch (a)**: G3 strict, G5 moving carrier, G8 control floor,
all met. The product is one persistent three-member QUOTIENT — the thing the road asked
for — and not a component whose formula happens to read like water. Verified twice: by the
census, and by `block_probe` reading the block's held series directly (793 held runs, max
894.7 fs, and the window the budget accepted spelled out frame by frame).

### The OH₂ exists in TWO arms, and only one of them conserves momentum

| arm | OH₂ carrier | \|p\| / bound, all seeds | gate |
|---|---|---|---|
| dE₄ (`p2_de4_seeds/`, banked `1ae2c1b`) | seed `0x53415422` | **9.8e3 – 4.2e5** | **FIRED, 4–5 orders over** |
| MBE3 fenced (this lane, pin `a3b3d4b`) | seed `0x53415425` | **4.7e-5 – 7.9e-5** | passes, nine orders inside |

The lead's correction is confirmed: seed `0x53415422`'s dE₄ line reads
`molecules [H2 H2 OH2 O3H2]`, modal H₂O, 1,118 dE₄ solves — a molecule line, not the header
collision. **And it is under a fired gate.** All six banked dE₄ seeds breach the momentum
bound by four to five orders with energy IN bound, which is the specific signature of a
force that is not equal-and-opposite: a many-body gradient that does not sum to zero. No
molecule downstream of that dynamics is a physics result yet (OBJECT.md rule 7).

**Refinement, from the water lane's diagnosis of the defect (relayed by the lead,
2026-09-01, after this section was first written).** The momentum breach was a DOUBLE MASS
DIVISION, and it did not only break momentum — it also weakened the dE₄ forces themselves
by three to four orders. So the six banked dE₄ seeds were not "MBE3 plus a four-body term";
they were **near-MBE3 physics carrying a biased nudge**. That sharpens the reading in a
direction worth stating: the dE₄ arm's OH₂ is not independent evidence alongside mine, it
is a second observation from substantially the same physics. **The first true four-body
experiment has not been run yet.**

**The conservation-clean OH₂ is the one measured above, and it does not need the four-body
term at all.** It arises in MBE3 with the four (O,O,O) triples honestly fenced, on a build
whose momentum residual sits nearly ten orders of magnitude below the dE₄ arm's. So the
water claim's adjudication does not have to wait on the dE₄ repair: it has a
conservation-clean carrier now, and the repair becomes a separate and narrower question:
**does the four-body term CHANGE the answer.** The fenced arm banked here is the baseline
that question is measured against, which makes it a one-variable comparison rather than an
inference — the same shape §10.5 specifies for the OOO fence, and the second such
experiment this lane leaves specified and not run.

**Two one-variable experiments are therefore owed and named, neither inferred:**

| question | held fixed | the variable | status |
|---|---|---|---|
| does the OOO fence move the endpoint? | everything at `45a513a` | served vs fenced | specified §10.5, not run |
| does the four-body term change the answer? | this fenced arm as baseline | repaired full-strength dE₄ | water lane owns the rerun; this census adjudicates its trajectories when they exist |

### Scoped, so nothing here over-reaches

* **On the MBE3 banked log (`p2_waterquench.log`) there is genuinely no water**, and the
  header-collision analysis below is the correct reading of THAT artifact: `OH2` occurs on
  one line, the surface list, and every molecule line reads H2/OH/O2H/O3H3/O4H2/O4H4. The
  parser gate that separates the two stands unchanged and is worth keeping regardless.
* **One seed of eight** produced a certified-strict OH₂ in the fenced arm; two more seeds
  produced OH₂ blocks that certify only at BUDGET (18.0% and 11.3% of their runs). That is
  1 of 8 strict, not a formation rate, and this document does not offer one.
* **The dE₄ observation is not cashed** and this lane does not cash it.

**What IS measured**, on all eight regenerated trajectories of the hydrogen control arm:

1. **The instrument separates a molecule from an encounter by an order of magnitude, on
   real data.** Across 48 H₂ blocks the longest held run is **666–3367 fs** (median 1415,
   mean 1511) against a staked window of 834. Across 32 H₃/H₄ blocks it is **23–78 fs**.
   The shortest H₂ outlives the longest H₄ by 8.5×, with no threshold tuned between them.
2. **NOTHING CERTIFIES — including all 48 H₂.** They are voided by my own control floor,
   at a pool rate of exactly 0.077 on every single one, and the 0.077 is arithmetic rather
   than physics (§4). This is the pre-committed control finding a defect in the prereg.
3. **The membership view is NOT CLOSED, and its leak EXPANDS.** Leg B exhibits hundreds of
   witness pairs per seed at defects of 0.157–0.247 over ~20,000 informative transitions,
   and **OBJECT.md rule 1's non-expansion budget is BREACHED on six of eight seeds**.
4. **A window staked in time was being measured in frames, and on real data those differ**
   (§6). Caught before it reached a verdict, by cross-checking a printed number against
   the artifact.

---

## 1. What ran

`waterquench_traj`, the frozen P2 protocol with a trajectory dump at every grain boundary,
built in an isolated worktree at a pinned commit because the shared tree did not compile
when this lane started. Hydrogen control arm, `--ozone=fenced`, all eight staked seeds,
20,000 frames each, 16.68 or 33.35 ps of simulated time depending on whether that seed's
timestep halved.

The instrument-identity gate (`holon-render/tests/protocol_identity.rs`) passes: the
frozen-protocol block — the eight seeds, the box, both temperatures, the thermostat
coupling, the frame and substep counts, the jitter, the knot count, the RNG, the placement
and the whole measurement rule — is byte-identical between `waterquench.rs` and
`waterquench_traj.rs`.

## 2. LEG A — is it a persistent quotient?

Every seed forms **exactly six H₂** by the formula reader, which is what `waterquench`
prints and what SATURATION-1 banked.

| population | n | longest held run |
|---|---|---|
| H₂ blocks | 48 | **666.2 – 3366.9 fs** (median 1415.0, mean 1510.7) |
| H₃/H₄ blocks | 32 | **23.3 – 78.4 fs** |
| staked window | | 834.0 fs |

43 of the 48 H₂ clear the window on their longest STRICT run; the other five reach it only
under the budget clause, which is the budget doing exactly the job it was written for.
Every H₄ is an order of magnitude short — the longest, 78.4 fs, is 9% of the window.

**Every carrier moves.** Internal RMS displacements run 0.121–0.847 bohr and intra-block
separation excursions 0.079–4.047 bohr, against staked floors of 0.100 and 0.050. Not one
H₂ is the frozen carrier G5 exists to catch, so none of these readings is the vacuous kind.

**And all 48 read VOID (no separation).** See §4 — the reason is arithmetic about the
eligible pool, not anything about the molecules.

**Totals over the arm: 0 certified-strict, 0 certified-budgeted, 343 transient, 48 void.**

## 3. LEG B — does the membership view carry its own dynamics?

`closed_iff_fiber_invariant` makes closure the statement `∀ x y, v x = v y → v (T x) =
v (T y)`. The instrument collects the observed fibers and exhibits the pairs that break it.

| seed | witness pairs | defect | 1st half | 2nd half | non-expansion (≤1.05×) |
|---|---|---|---|---|---|
| `0x…5421` | 262 | 0.1566 | 0.1372 | 0.1760 | **BREACHED** (1.28×) |
| `0x…5422` | 465 | 0.2470 | 0.2318 | 0.2622 | **BREACHED** (1.13×) |
| `0x…5423` | 370 | 0.1956 | 0.2383 | 0.1529 | ok (0.64×) |
| `0x…5424` | 303 | 0.1706 | 0.1651 | 0.1762 | **BREACHED** (1.07×) |
| `0x…5425` | 441 | 0.2142 | 0.1840 | 0.2443 | **BREACHED** (1.33×) |
| `0x…5426` | 291 | 0.1653 | 0.1465 | 0.1842 | **BREACHED** (1.26×) |
| `0x…5427` | 347 | 0.1820 | 0.2060 | 0.1581 | ok (0.77×) |
| `0x…5428` | 328 | 0.1776 | 0.1541 | 0.2010 | **BREACHED** (1.30×) |

Work count: ~19,984 informative transitions per seed against a staked minimum of 200, so
none of this is the vacuous kind either. Witness pairs are exhibited by index — e.g. on
seed `0x…5421`, frames 7177 and 7185 read the same partition while frames 7178 and 7186 do
not.

**The reading, stated carefully.** The bonded partition is not a Closed view of the atomic
tier: 16–25% of its informative transitions cannot be predicted from the reading alone.
That much is expected — OBJECT.md rule 2 says the claim is never zero leak. What is a
FINDING is the second clause: rule 1 asks for **non-expanding** leak within a budget of
1.05×, and the leak EXPANDS on six of the eight seeds, by 1.07× to 1.33×. A tier built on
this view would certify on its first half and be out of budget by its second. The two seeds
that pass (`0x…5423`, `0x…5427`) pass because their defect happens to FALL across the run,
which is a different thing from a bounded leak and is reported as such.

**And the coarser views fail too**, which is the interesting part — a coarser view is
usually easier to close:

| macro view | bins | defect across the arm | reading |
|---|---|---|---|
| largest domain (atoms) | 13 | 0.143 – 0.229 | NOT CLOSED |
| bonded pair count | 67 | 0.175 – 0.274 | NOT CLOSED |
| mean hexatic ψ6 | 10 | 0.226 – 0.403 | NOT CLOSED |
| H-bond count | 20 | 0.000 | VACUOUS (one reading; no oxygen in this arm) |

The H-bond row is labelled VACUOUS rather than closed on purpose: a constant view is closed
by `h = id` and has said nothing (M-FIXED-POINT-TRAJECTORY).

## 4. THE CONTROL FLOOR IS MIS-STAKED, and its own control found it

All 48 H₂ are VOID at a control rate of exactly **0.077** — the same number on every block
of every seed — and the arithmetic is not subtle.
The eligible pool of (2 H) blocks in a 12-hydrogen box is C(12,2) = 66; excluding the block
under test leaves 65; the scene contains **six** genuine H₂, so any one of them sees the
other **five** pass. 5/65 = 0.0769 against a staked ceiling of 0.05.

The floor as staked can therefore be satisfied only by a scene holding **at most four**
molecules of the composition under test (3/65 = 0.046 passes, 4/65 = 0.062 does not). That
is not a test of whether the criterion discriminates. It is a cap on how many molecules the
scene may contain, which is not what it was written to be — and it is a cap that gets
TIGHTER exactly as a scene becomes more chemically interesting.

**The stake is NOT being moved.** The verdicts above stand as VOID. What is recorded beside
them is the underlying Leg A measurement, which is clean, strong and on moving carriers.

**The sharpest demonstration, from the mixed arm.** The same instrument gives H₂ OPPOSITE
verdicts in the two arms, and the thing that decides it is how many hydrogens are in the
box:

| arm | H atoms | pool C(n,2) − 1 | H₂ molecules | peers passing | control rate | H₂ verdict |
|---|---|---|---|---|---|---|
| hydrogen | 12 | 65 | 6 | 5 | **0.077** | VOID (no separation) |
| mixed | 8 | 27 | 2 | 1 | **0.037** | CERTIFIED-STRICT |

Both numbers are exactly the arithmetic — 5/65 and 1/27 — and both appear in the census
output verbatim. So a molecule that certifies in a box of eight hydrogens is voided in a
box of twelve, with identical physics, identical thresholds, and no measurement
distinguishing them. **A floor whose verdict flips on the composition of the box is not
measuring the block.**

**The defect, named:** the prereg compared a pool rate against a flat constant. This
programme's own discipline rule 5 says to control estimator bias with a **shuffle or
permutation floor**, and that rule was not applied. Staked now for the next freeze and not
retroactively: the pool pass rate must be compared against the pass rate among
same-composition blocks in a surrogate whose bond graph is time-shuffled within each pair.
A flat 5% encodes a fixed selection strength; a shuffle floor encodes the question actually
being asked, which is whether these blocks hold more than chance blocks hold.

## 5. THE TWO ARMS ARE TWO EXPERIMENTS, AND ONLY ONE IS BANKABLE

| | Arm A — banked | Arm B — the running processes |
|---|---|---|
| physics | MBE3, all four surfaces served, fence 0 | MBE3 + `dE4(O,H,H,H)`, fence 4 |
| source | committed, `45a513a` | sim.rs dispatch **in no commit** |
| evidence | `p2_waterquench.log`, 8 seeds, 0/8 water | not yet printed |
| my build | same class: **0** `quaternary::de4_ohhh_fci` symbols | **1** in theirs |

**My build's provenance, verified by symbol table rather than by mtime**, which is the
method the water lane established today: `nm -C` on my `waterquench_traj` finds zero
`quaternary::de4_ohhh_fci` symbols; on the running `waterquench` it finds one. Neither
binary carries any T3 marker (`DEFAULT_SCENE_ATOMS`, `complete_pairs`, `ExternalWork`,
`Periodic` all absent), so both predate that refactor. My worktree is pinned at `a3b3d4b`.

The consequence is clean rather than awkward: **my arms cannot reproduce Arm B, and a
protocol-equality gate between them would fail for a reason that has nothing to do with
trajectories.** They are not two builds of one experiment. `quaternary.rs` itself IS
committed and IS gated (`forty_witnesses_ab_initio_sign_structure_and_bounds`, all 40
staked witnesses with exactly 11 attractive); it is only the sim.rs DISPATCH of it that is
working-tree-only.

Two things were done about that rather than only reported:

* **The bytes are preserved, twice.** The water lane's `rescue/de4-sim-worktree`
  (`7480437`, a `git stash create` that touched neither working tree nor index) and this
  lane's `refs/rescue/de4-2026-09-01` (`11549dd`, 21 files, 6,245 insertions including the
  untracked 738-line `cells.rs`). The reconstruction target is that tree MINUS the T3
  hunks — a live, owned diff, not archaeology.
* **Recoverability was MEASURED.** That content in a throwaway worktree at HEAD fails
  `cargo check -p holon-render` with 14 errors, all T3's own in-flight edits.

Arm B is therefore reported as **provenance partially reconstructed**, and nothing from it
banks until its dispatch is committed and one completed seed reproduces its per-seed line.
`CENSUS_RESUME.md` holds the recipe.

**And the served arm is prep-only.** `--ozone=served` loads a surface that is mid-generation
and uncertified, and whose predecessor was convicted by M-CHEAPER-THAN-ITS-PRICE. That arm
exists to test whether my runner reproduces the committed log — an INSTRUMENT gate — and
nothing from it banks as physics under any outcome.

## 6. A DEFECT IN THIS INSTRUMENT, found before it reached a verdict

The first version of the census converted the staked window into a frame count once, from
the header's `dt`. **The engine's timestep adapts.** On hydrogen seed `0x53415421` the
header records `dt = 1.0772` at placement, the timestep halves after eleven frames, and
19,988 of the 20,000 frames run at 0.5386 — a delta histogram of 19,988 at 34.4707 a.u.
and 11 at 68.9414. So the window was 500 frames of 1.6676 fs on paper and 500 frames of
0.8338 fs in fact: **417 fs against a staked 834, making certification twice as easy as it
was staked to be.**

Found by cross-checking a printed number against the artifact — `waterquench` reports
`dt 0.5386` for a seed whose dump header says `1.0772`. Every window is now measured
against `Frame::time`. `Header::frame_fs` and `Header::frames_in` were REMOVED rather than
documented around, because a helper that is right on synthetic data and wrong on real data
is worse than no helper: every test passes. The diffusion lens had the same defect in its
lag axis and now uses mean elapsed time. Reports carry the number of distinct frame
durations a run used, so "frames" is never silently a unit again.

## 7. THE LENS STACK, and its refusals

On the hydrogen arm, four of six lenses refuse, each naming the gate that would lift it:

| lens | reading |
|---|---|
| q-tetrahedral | REFUSED — `dims == 3`; a tetrahedral parameter on a plane does not contain the variable it names (M-MAINTENANCE-LENS) |
| Steinhardt q6 | REFUSED — `dims == 3`; every neighbour sits at θ = π/2 and the sum is a hexatic in disguise |
| hexatic ψ6 | 0.397 |
| diffusion | REFUSED at every lag — see below |
| H-bond census | REFUSED — 0 oxygens; a zero here would read as a measured absence |
| largest domain (bonded-pair graph) | 2 atoms over 6 edges |

Gated against exact references: simple cubic `q6 = sqrt(1/8)` analytically, FCC 0.5745,
BCC 0.5107 at fourteen neighbours, a perfect tetrahedron 1, a planar square exactly 1/2, a
triangular-lattice `ψ6` 1.

**And the diffusion lens grew a second refusal, because the first was not enough.** Its
original gate was wall saturation alone, and under that gate it reported a diffusion
constant at every lag from 2 to 200 — a constant that GREW MONOTONICALLY with the fit
window, 0.000814 to 0.018238 bohr²/fs, a factor of 22. That is the signature of fitting a
line to a curve. Measuring the exponent says so directly:

| max lag | MSD exponent | verdict |
|---|---|---|
| 10 – 200 | 1.58, 1.68, 1.74, 1.74, 1.73 | REFUSED — no diffusive regime |
| 500 – 2000 | 1.67, 1.64, 1.54 | REFUSED — wall-saturated as well |

`MSD = 2 d D τ` is a fit to a LINE; the Einstein relation IS the statement that the
exponent is 1. On these trajectories it is 1.54–1.74 at every window, so **the lens now
refuses at every lag** and names which gate refused. The band `[0.85, 1.15]` is not tuned:
it is the tolerance on a log-log slope of ten points around the exponent the relation
asserts.

That refusal is checked in both directions, so it is not a branch that only ever says no:
a synthetic random walk in a large box reads `τ^1.00` and IS reported; synthetic ballistic
motion reads `τ^2` and is refused. **The reading is that a twelve-atom walled box over 16.7
ps has no diffusive regime at all** — one more measurement pointing at the T3 scale-up.

## 8. THE BLIND CLASSIFIER

`classify` takes a `&Trajectory`, and `Trajectory` has no launch label in it — blindness by
signature, not by discipline (M-TAG-AS-PROPERTY).

**On the hydrogen arm: LIQUID, 8 of 8.** Free fraction 0.000–0.065 (bonded), mobility
2.36–4.56 against an ICE bar of 0.10 (flowing). Interior atoms 0–2, which is the honest
reason the ICE branch has nothing to weigh on a twelve-atom scene.

**P-5 fired three times during construction** and named a different defect each time,
before any real data was read:

| firing | rate | defect it named | derived repair |
|---|---|---|---|
| 1 | 4.0% (8/200) | no finite-N floor: `E[ψ6²] = E[q_l²] = 1/N` for random neighbours, and at N = 6 that floor is 0.408 against a 0.45 bar | report `sqrt(max(0, raw² − 1/N)) / sqrt(1 − 1/N)` |
| 2 | 1.6% (16/1000) | six neighbours is not a first shell; a chance gas cluster passes a ratio test while being ragged | require the shell COMPLETE and TIGHT, average over interior atoms only |
| 3 | 0.2% (4/2000) | one atom is not a bulk — every remaining firing had exactly ONE distinct interior atom, its environment counted across dozens of correlated frames | require ≥ 2 distinct interior atoms; report `interior_atoms` beside `interior_samples` |

Every repair is derived from the failure it repairs and **the 0.45 stake never moved**. The
published false-crystal rate is MEASURED — 0.2%, 4 of 2000 — not inferred from zero events,
and it sits under the 1.5% bound the prereg staked. P-1 passes: a liquid trajectory
launched under an `ice` label classifies LIQUID.

## 9. What this census does NOT claim

* It does not claim water formed. On the banked artifact it did not: 0 of 8, verified here
  against the file. The hydrogen arm contains no oxygen at all.
* It does not claim anything about the running dE₄ arm, which had printed no seed line when
  this was written.
* It does not claim the bonded partition is Closed. Leg B can only exhibit witness pairs or
  fail to find them, and here it found 262–465 per seed.
* It does not claim H₂ is certified. Under the staked floor H₂ is VOID; the strong Leg A
  reading behind that VOID is reported beside it and never instead of it.
* It does not claim these 2D twelve-atom scenes are liquid water. Four of six lenses refuse
  them mechanically, and the classifier's LIQUID is a statement about a bonded mobile
  twelve-atom scene, nothing more.

---

## 10. G2 MECHANIZED, AND A FORWARD PREDICTION STAKED BEFORE ITS DATA

*Added after the hydrogen arm and BEFORE the mixed arms finished. The prediction in §10.3
is staked against seeds that had not printed when this was committed; git is the check.*

### 10.1 The gate

Comparing two runs "by eye" is how they come to be called the same without anyone
checking, so G2 is now a flag on the census runner: `--reference=<quench log>` parses a
banked log with `quenchlog` and diffs each trajectory's FINAL-FRAME molecule multiset
against that seed's row. It reports per seed and in aggregate.

### 10.2 The result on the hydrogen arm, and why it is 6 of 8

Against `conformance/atomworld/s2_runs/p1_hydrogen.log`: **6 of 8 seeds reproduce.** Seeds
`0x…5427` and `0x…5428` differ — the reference reads `[H2 H2 H2 H2 H4]` and my run reads
`[H2 H2 H2 H2 H2 H2]`.

**The cause is identified and it is not the instrument.** The H–H curve changed between
that reference and my pin:

| run | H–H worst residual |
|---|---|
| banked P1, Aug 30 (`p1_hydrogen.log`, `p1_mixed.log`) | **1.2e-12** |
| committed P2, Aug 31 (`p2_waterquench.log`) | **8.7e-11** |
| this lane, pinned at `a3b3d4b` | **8.7e-11** |

Same `R_e = 1.3887` and `D_e = 0.204142` to six digits, different solver convergence — so
the potential differs at the 1e-11 level. Over 20,000 frames of contact dynamics that is a
Lyapunov divergence, not a bug, and on two of eight seeds it lands the final frame on a
different side of the bond criterion: one H₄ or two H₂.

**The finding, stated as its own claim: the final-frame molecule census is not reproducible
across builds whose potential differs at 1e-11.** The AGGREGATE survives it — six H₂ per
seed, no heavier molecule anywhere, on both builds — and the PER-SEED DETAIL does not.

That is an argument for this census and against formula-matching, and it is worth saying
plainly: a persistence statistic integrates a block's membership over thousands of frames,
while a final-frame formula reads one instant at the end of a chaotic trajectory. The two
seeds that disagree about H₄ agree completely about every H₂ that held.

My pin sits in the SAME potential class as the committed P2 run (8.7e-11 both), which is
the relevant fact for the mixed arms.

### 10.3 THE PREDICTION, staked before the data

`ozone.rs` changed between `45a513a` (the commit behind `p2_waterquench.log`) and my pin
`a3b3d4b` — 19 insertions, 53 deletions. So **the `--ozone=served` arm cannot be a clean G2
at this pin**: it would be comparing two different ozone surfaces, and a mismatch would say
nothing about my runner. That arm is reported as what it is and no G2 verdict is taken from
it.

The `--ozone=fenced` arm is the informative comparison, because it differs from
`p2_waterquench.log` in exactly ONE stated way: the four (O,O,O) triples per seed are
fenced rather than served, on an otherwise same-class potential.

> **STAKED:** the fenced arm reproduces `p2_waterquench.log`'s molecule multiset on a
> MAJORITY of seeds — **≥ 5 of 8**. Reasoning: the OOO term touches 4 triples of the
> C(12,3) = 220 in the box, and the H–H curve is the same class, so most seeds' endpoints
> should be unmoved.
>
> **What each answer means.** ≥ 5/8 — the OOO surface is a small correction at this scale
> and the fence is cheap; the census's aggregate readings transfer between the two
> configurations. ≤ 4/8 — the OOO term materially steers the endpoint even at four triples,
> and no reading taken with it fenced may be compared to one taken with it served. Either
> way the per-seed detail is chaotic (§10.2) and only the aggregate is worth quoting.

Seed `0x…5421` had printed when this was staked and it MATCHES (`[H2 H2 O4H4]` both, fence
4 against fence 0). Seeds 2–8 had not printed. The remaining seven are the test.

### 10.4 THE SERVED ARM REFUSED TO RUN, and that settles §10.3 more strongly than predicted

Minutes after §10.3 was committed, the served arm finished its O–O curve and **panicked**:

```
thread 'main' panicked at waterquench_traj.rs:514:56: the Ozone table generates
```

At `a3b3d4b`, `holon_chem::ozone::generate()` is a hard stub — `pub fn generate() ->
Option<OzoneTable> { None }`. The convicted surface was not patched, it was WITHDRAWN, and
that is the whole of the 19+/53− diff §10.3 cited. So `--ozone=served` cannot run at this
pin at all.

Two things this validates, and one it costs:

* **The runner refused rather than invented.** `--ozone` is a required argument with no
  default and the table load is an `expect`, so the arm died loudly at the moment the
  surface was not there instead of quietly running with an empty one and reporting a
  molecule census. A silent empty-table fallback would have produced numbers indexed to a
  configuration nobody chose.
* **§10.3's reasoning was right and understated it.** The served arm is not merely "not a
  clean G2"; it is impossible at this pin.
* **And the cost, stated plainly: `p2_waterquench.log` is NOT REPRODUCIBLE from any current
  commit.** The surface it ran on exists in no tree that still generates it. Combined with
  §10.2's H–H curve change, neither the P1 nor the P2 banked logs can be re-run today. They
  remain valid RECORDS; they are no longer reproducible EXPERIMENTS, and anything that
  needs to stand on them needs to say which of those two it is relying on.

**Not taken, with the reason:** the water lane has a verified worktree at `45a513a`
(`/home/emoore/holon-wt/p2pin`) where the surface still generates, and re-running the
served arm there would give a bit-level G2 against the banked log. I did not spend it. That
run would validate the instrument against CONVICTED physics on a box already at load 66–85
with the ozone tabulation on the critical path, and the instrument already has a G2 reading
from the hydrogen arm (§10.2) whose one discrepancy is fully explained. The option is
recorded rather than quietly dropped; if anyone wants the bit-level check, that is where it
is and it costs about an hour of contended machine.

**The fenced arm is unaffected and still running.** It is the arm the §10.3 prediction is
staked on.

### 10.5 AMENDMENT, before the data: §10.3's premise is FALSE and its stake is confounded

*Written with two of eight fenced seeds printed. The stake is NOT moved; what follows is a
correction to its REASONING, recorded before the result so that git can tell which came
first.*

§10.3 said the fenced arm differs from `p2_waterquench.log` "in exactly ONE stated way:
the four (O,O,O) triples per seed are fenced rather than served, on an otherwise same-class
potential". **That is wrong. There are two differences, and I missed the larger one.**

| curve | banked P2 (`45a513a`) | this lane (`a3b3d4b`) |
|---|---|---|
| H–H worst residual | 8.7e-11 | 8.7e-11 — same class, as claimed |
| O–H worst residual | 9.9e-11 | 9.9e-11 — same class |
| **O–O worst residual** | **6.7e-6** | **2.7e-6** |

`R_e = 2.4421` and `D_e = 0.147621` agree to six digits, but the O–O curve MOVED. The cause
is in `holon-chem/src/tier.rs`, which gained a sparsity optimisation between the two
commits — it tracks touched `kl` indices instead of zeroing the whole accumulator, which
changes the floating-point accumulation ORDER in the CI solve, which changes where the
solve converges.

**Why this matters more than it looks.** §10.2 measured what a 1e-11 change to the H–H
curve does: it flipped the final-frame molecule census on 2 of 8 hydrogen seeds. The O–O
change is at 1e-6 — five orders of magnitude larger — and it sits on the curve that governs
the oxygen aggregation dominating every mixed-arm scene (the O₄H₄ droplets). So I now
expect the ≥ 5/8 stake to FAIL, and I am saying so before the seeds print rather than after.

**And the deeper defect: the ≤ 4/8 branch's stated meaning is now CONFOUNDED.** §10.3 said
a low match rate would mean "the OOO term materially steers the endpoint even at four
triples". It can no longer mean that, because a moved O–O pair curve would produce the same
low match rate on its own. The experiment as designed cannot separate the two causes, and
no rescue after the fact can separate them either.

**What would.** Run the fenced arm at `45a513a`, where the O–O curve is the banked one and
the ONLY difference is the fence. That is a clean one-variable comparison and it is the
experiment §10.3 should have specified. It is not run here; it is specified so that whoever
wants the OOO question answered has the design rather than an inference.

**The stake stands as staked** and will be scored against ≥ 5/8 when the arm finishes. What
this amendment changes is what the number is allowed to MEAN, and the answer is: much less
than §10.3 claimed.

---

## 11. THE FENCED MIXED ARM — the arm the water result comes from

Eight seeds, 20,000 frames each, `--ozone=fenced` (the four OOO triples honestly fenced),
pin `a3b3d4b`, momentum residual 4.7e-5 to 7.9e-5 of bound on every seed.

### 11.1 What certified

| verdict | count | compositions |
|---|---|---|
| CERTIFIED-STRICT | 15 | 6 H₂, 2 OH, 2 O₄H₂, 2 O₂H, **1 OH₂**, 1 O₃H₂, 1 O₃H |
| CERTIFIED-BUDGETED | 14 | 4 O₂H, **2 OH₂**, 2 OH, 1 O₄H₄, 1 O₄H₂, 1 O₃H₃, 1 O₃H₂, 1 O₂H₃, 1 O₂ |
| TRANSIENT | 262 | |
| VOID | 15 | all "no separation" — the control floor of §4 again |

**Three OH₂ blocks reach a window across the arm: one strict (§0) and two at budget** (18.0%
and 11.3% of their runs). One of eight seeds carries a strict one. That is the honest
denominator, and this document does not convert it into a formation rate.

The census certifies things the final-frame reader never names, and declines things it
does. On seed `0x…5421` the formula reader prints three molecules (`O4H4 H2 H2`) while the
census certifies six — including an OH and an OH₂ it never mentions — and the O₄H₄ it does
print was a block for 9.0% of the run. **A final-frame formula and a persistence statistic
are different readings of the same trajectory, and where they disagree the trajectory says
the persistence one is carrying more.**

### 11.2 Leg B: the leak is smaller here, and it does NOT expand

| seed | defect | non-expansion (≤ 1.05×) |
|---|---|---|
| 5421 | 0.1128 | ok |
| 5422 | 0.1328 | ok |
| 5423 | 0.1339 | ok |
| 5424 | 0.1453 | ok |
| 5425 | 0.1410 | ok |
| 5426 | 0.0815 | ok |
| 5427 | 0.1460 | ok |
| 5428 | 0.1287 | ok |

**Eight of eight pass**, against the hydrogen arm's six of eight breaching (§3), and the
defects are lower throughout (0.082–0.146 against 0.157–0.247). The membership view of the
mixed box is a better-behaved coarse view than the pure-hydrogen box's: its budget holds
where the other's did not. A plausible reading, offered as one — the mixed box forms heavy
clusters (O₄H₂, O₃H₂) that pin the partition, while twelve hydrogens keep re-pairing — but
the measurement is the eight-of-eight, not the explanation.

### 11.3 THE STAKED PREDICTION IS SCORED, AND IT FAILED

§10.3 staked **≥ 5 of 8** seeds reproducing `p2_waterquench.log`'s molecule multisets.

> **MEASURED: 2 of 8.** Seeds `0x…5421` (`[H2 H2 O4H4]`) and `0x…5424`
> (`[H2 H2 H2 O2H O2H]`) match; the other six do not. **The stake FAILED.**

§10.5, committed at `d2533b0` with two of eight seeds printed, said it would — and said
why the failure could not be read as an answer about the OOO term, because the O–O curve
had also moved (6.7e-6 → 2.7e-6). That amendment stands: **2 of 8 is confounded and is not
evidence that the fence matters.** The clean experiment is the fenced arm at `45a513a`,
where the only difference IS the fence; it is specified in §10.5 and was not run here.

What 2 of 8 DOES support is §10.2's finding, now on a second arm: a final-frame molecule
census is not stable across builds whose curves move at 1e-6, while the aggregate — no
water on the MBE3 banked log, water on one fenced seed, H₂ everywhere — is what survives.

---

## 12. THE dE₄ ADJUDICATION — design staked before the data

*Written while both arms were still generating their pair curves; neither had printed a
seed line. Git is the check.*

### 12.1 What is being compared, and why it is not the obvious comparison

The lead asked for the full-strength OH₂ to be judged "against your fenced-arm baseline as
the one-variable comparison". **It would not have been one.** The fenced arm of §11 is
pinned at `a3b3d4b`; the full-strength dE₄ run is at `21e6be3`; between them sit the whole
T3 refactor and the `tier.rs` solver change that already moved the O–O curve from 6.7e-6 to
2.7e-6. That is the confound that killed §10.3's prediction, named in §10.5, and repeating
it here would be repeating it knowingly.

So the control is generated fresh **at the same commit**:

| arm | commit | seed | ozone | dE₄ |
|---|---|---|---|---|
| A | `21e6be3` | `0x53415422` | fenced | **on** |
| B | `21e6be3` | `0x53415422` | fenced | **off** |

Everything else identical, including the binary. One variable.

This is possible only because `--de4=on\|off` became a required argument today — see §12.4,
which is the near miss that forced it.

### 12.2 Admissibility gates, checked before any verdict is read

- **G-dE4-1 — the term must have fired.** Arm A must report `dE4_evals > 0` and arm B must
  report exactly `0`. The count comes from `Sim::de4_eval_count`, incremented by the
  physics itself. If either fails, the arms are not what they claim and the comparison is
  **VOID** — no verdict either way. *(A symbol-table check cannot stand in for this; §12.4.)*
- **G-dE4-2 — arm A must reproduce the banked run.** Its final-frame molecule multiset must
  equal `engine/output/p2_de4_full/seed_0x53415422.log`'s `[H2 H2 OH2 O3H2]`. If it does
  not, my regeneration is not their run and the census speaks only about mine.
  *Grounded before the run, by reading the reference runner at `21e6be3` rather than
  assuming: `waterquench.rs` there sets `base.ozone = OzoneTable::empty()` and
  `base.de4_enabled = true`, with the same trimer, water and OOH tables. That is exactly
  arm A's `--ozone=fenced --de4=on`, so G-dE4-2 is comparing like with like. It also has no
  method-style scene setup that the knob gate's `base.<field>` scan would miss.*
- **G-dE4-3 — conservation.** Both arms must hold \\|p\\|/bound below 1, as every
  conservation-clean arm in this document has. A breach voids that arm exactly as it voided
  the broken-dE₄ logs.

### 12.3 THE BRANCHES, with what each would mean

Let **W** = the staked 834 fs window, and the verdict be the census's on the (1 O, 2 H)
block of each arm.

* **(a) A certifies, B does not.** The four-body term is what makes water *on this seed*,
  and the product is a persistent quotient rather than a final-frame formula. The strongest
  available result, and the one the three-arm story predicts.
* **(b) BOTH certify.** The four-body term is NOT what makes water here. This would not be
  surprising: §0 already reports a certified-strict OH₂ from MBE3 physics on seed
  `0x53415425`, so water forming without dE₄ is established — this would establish it on
  *this* seed too, and would mean the dE₄ arm's OH₂ is over-attributed.
* **(c) NEITHER certifies.** The full-strength OH₂ is a final-frame formula that does not
  hold a window. Road item 5's dE₄ leg is not cashed, and §0's MBE3 certification stands
  alone as the water result.
* **(d) B certifies, A does not.** Adding an exact four-body term destroys a quotient that
  MBE3 sustains. Reported as-is and investigated, not explained away.

**What NO branch licenses:** a formation rate, or "dE₄ makes water" as a general claim.
This is ONE seed. §11 already measured 1 strict OH₂ in 8 MBE3 seeds; a single dE₄ seed
cannot be compared against that without the other seven, which are not run here.

### 12.4 The near miss that forced §12.1's design

`waterquench.rs` sets `de4_enabled = true` in its own `main`, BELOW the frozen-protocol
block that `tests/protocol_identity.rs` byte-compares. When the four-body work landed, the
block was updated in both runners (a `MAX_ATOMS` → `DEFAULT_SCENE_ATOMS` rename), **the gate
passed**, and `waterquench_traj` silently kept `Sim::empty()`'s `de4_enabled: false`.

Regenerating "the dE₄ seed" with it would have run the four-body term **switched off**,
written a trajectory of different physics under the right filename, and let this census
report a confident failure to certify. Every number would have looked reasonable. It was
caught by reading the runner one command before launching it.

Two repairs, both committed before the arms launched: `--de4=on|off` is required with no
default, and the gate now inventories every `base.<field>` assignment in both runners and
requires the stand-in to name every knob its reference names — verified to FIRE against the
runner at `21e6be3` (`missing = [de4_enabled]`) and to pass against the fixed one.

**And the method correction.** Declaring a build by symbol table is sound on PRESENCE and
worthless on ABSENCE: `nm -C` finds zero `quaternary::de4_ohhh_fci` symbols in a build that
calls it, because the call is inlined. This document's earlier pinning conclusions stand —
they rested on presence in one binary against a source that could not call it at all — but
the method needed the caveat before someone read an absence as a verdict. Hence G-dE4-1
being a counter and not a symbol.

---

## 13. THE dE₄ ADJUDICATION — PARTIAL: arm B has landed, arm A has not

*Written with the control complete and the matched arm still running. The verdict of §12.3
is NOT made here. This section records what arm B measured, because a result held only in a
session's memory is a result one interruption from being lost.*

### 13.1 Arm B (`--de4=off`) — MBE3 by measurement, and it makes water

```
seed 0x0000000053415422  dt 0.5386  modal-O OH2  molecules [H2 H2 OH2 O3H2]
                         fenced 4  dE4_evals 0  |p| 3.99e-12/1.04e-7  T 290 K
```

**`dE4_evals 0`** is the functional proof G-dE4-1 asks for: the counter is incremented by
the physics, so the four-body term is absent by measurement rather than by flag.
\|p\|/bound = 3.84e-5 — conservation-clean, G-dE4-3 met.

Census:

| formula | block | held run | held | of run | rms | sep var | ctrl | verdict |
|---|---|---|---|---|---|---|---|---|
| H₂ | `0x0300` | 5366 | 4473.4 fs | 97.7% | 0.299 | 0.270 | 0.037 | CERTIFIED-STRICT |
| H₂ | `0x0c00` | 3066 | 2558.1 fs | 95.4% | 0.655 | 2.588 | 0.037 | CERTIFIED-STRICT |
| O₃H₂ | `0x009d` | 1974 | 1645.1 fs | 38.4% | 0.220 | 0.394 | 0.000 | CERTIFIED-STRICT |
| **OH₂** | **`0x0062`** | **1109** | **923.9 fs** | **85.8%** | **1.103** | **0.257** | **0.000** | **CERTIFIED-STRICT** |
| OH | `0x0011` | 929 | 773.8 fs | 38.5% | 1.000 | 0.209 | 0.000 | CERTIFIED-BUDGETED |

Leg B: 43 distinct partition readings, 124 witness pairs, defect 0.1365, non-expansion ok,
**NOT CLOSED**.

**So a certified-strict water quotient forms with the four-body term measurably switched
off.** That is a second one, independent of §0's seed `0x…5425`, and this one has its
control's provenance recorded (`PROVENANCE_de4_arms.md`).

### 13.2 What is NOT concluded here

Arm A is still running, and until it lands **no causal sentence is available in either
direction**. §12.3's branches stand as staked. The pairing matters: it is arm A that tests
whether the four-body term CHANGES anything, and a control without its treatment is half an
experiment.

### 13.3 A cross-check that the arms are what they claim

`dE4` is cutoff-gated — it solves only for compact quadruples — and in the banked reference
it reports zero solves through frame 4000, first firing at frame 5000 (59 solves), where its
wall time also jumps from 19 s to 309 s. That gives a check worth recording:

| frame | banked reference | arm A (`--de4=on`) | arm B (`--de4=off`) |
|---|---|---|---|
| 2000 | 287 K, drift 1.14e-5 | 287 K, drift 1.14e-5 | 287 K, drift 1.14e-5 |
| 4000 | 325 K, drift 1.73e-5 | 325 K, drift 1.73e-5 | 325 K, drift 1.73e-5 |

**All three are the same trajectory until the one variable engages** — which is what a
controlled comparison looks like from the inside, and evidence that my build reproduces the
banked run before G-dE4-2 is formally scored.

### 13.4 The caveat that applies to every certification in this document

Both arms report the O–O pair curve at **worst residual 4.81e-6 against the code's own
`CONVERGED_RESIDUAL` of 1e-9** — 3.7 orders past its threshold, on the curve governing the
oxygen aggregation these scenes are made of. It is IDENTICAL in the two arms, so it does not
differentiate them and the relative comparison is unaffected; what it qualifies is the
ABSOLUTE claim. A certified water molecule here is certified under an unconverged O–O curve,
and that belongs beside the certification rather than only in the run log.

Raised by `workbench-engine`, who stopped before captioning a page with it and asked. The
provenance gap they found in the same read — the control's log recording no commit, no
binary hash and no gate state, when "same commit" is the entire content of that control — is
answered in `PROVENANCE_de4_arms.md`.
