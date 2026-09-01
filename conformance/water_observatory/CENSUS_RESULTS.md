# THE CLOSURE CENSUS — results

*Stakes: `CENSUS_PREREG.md`, frozen and committed (`2f389e7`) before a line of the
instrument was written. Every threshold quoted here is one of its `PREREG_*` constants.
No stake was moved after seeing a number; where a stake turned out to be wrong that is
written up as a finding (§4) rather than repaired in place.*

---

## 0. THE HEADLINE

**THE OH₂ THIS LANE WAS BRIEFED TO ADJUDICATE DOES NOT EXIST.** The brief said seed 2
produced the programme's first emergent OH₂. It did not. `OH2` occurs on exactly ONE line
of `conformance/atomworld/p2_waterquench.log`, and that line is the header:

```
# Physics Path: Pairs (H-H, O-H, O-O) + Complete MBE3 Triples (H3, OH2, O2H, O3)
```

`OH2` there is the NAME OF A TABLE — the (O,H,H) three-body surface, listed beside H3, O2H
and O3. Every molecule line in that file reads `H2`, `OH`, `O2H`, `O3H3`, `O4H2` or `O4H4`;
the run's own census is `19xH2 1xOH 4xO2H 1xO3H3 1xO4H2 4xO4H4`; and its own headline is
**0 of 8 seeds with H₂O as the modal O-containing molecule**, with all four surfaces served
and the fence at zero. Seed `0x…5422` — "seed 2" — produced **OH**, not OH₂.

The correction came from the saturation2-water lane and is verified here against the
primary artifact rather than taken on trust. It is now a gate:
`holon_lens::quenchlog` parses header surfaces apart from census molecules, and its plant
asserts BOTH halves — that a grep of the real file DOES hit `OH2`, and that the parsed
molecule count for `OH2` is zero. A gate that passed because the string was absent would
be no gate at all; the string is present, and the parser is what separates them.

**So there was never a water molecule for the closure census to promote or reject on the
banked artifact.** That is the result, not a failure of the instrument. The census keyed on
a (1 O, 2 H) block correctly finds none, because none formed.

**The still-running dE₄ arm is a DIFFERENT experiment and is still unverified** (§5). Its
binary carries `quaternary::de4_ohhh_fci`; mine does not. Whatever it prints will be
four-body physics, not another sample of the banked run, and its sim.rs dispatch is in no
commit.

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
