# NODE LG — RESULTS

**Every gate of `LG_PREREG.md` PASSED. The tier certifies as its own object, and the
certificate contains an honest negative: this fluid tier's coarse charts are NOT closed
views, and the only exactly-closed chart is the global one, which closes by conservation
alone.**

The defect is not a residual and not a fit. At one step the closure defect of the block
chart `v_b` is **exactly the block's boundary fraction**,

> **W(b) = 1 − max(0, b−2)² / b²   for b < L,   and W(L) = 0**

measured to exact equality at every chart, and identical for **every one of the 4608 REG+
collision laws** the census admits — the identity collision included. Hydrodynamics on this
tier is a measured approximation, not a closed view, and the failure belongs to the lattice
rather than to any choice of collision rule.

| | |
|---|---|
| prereg | `conformance/mesh/LG_PREREG.md`, ADMITTED, frozen at `ce392f3` |
| instrument | `engine/crates/holon-lattice/`, committed at `c61ddbd`, clean tree |
| campaign log | `lg_full.log` — its header carries the instrument commit, the uncommitted-path count, and the toolchain (M-STALE-INSTRUMENT) |
| test log | `lg_tests.log` — 32 tests, 32 passed, 0 failed |
| log sha256 | `lg_full.log` `7180b50a…aaacd` · `lg_tests.log` `9dccd645…f7694` — naming what was measured and nothing inferred beside it (M-PROVENANCE-OVERREACH) |
| termination | `STEPS_COMPLETED` — no gate tripped early, no budget exhausted |

---

## 0. THE FIRST LAW, AND A CONCRETE REASON IT IS NEEDED

**Nothing in this node is composed through `closed_comp`.** This tier is not a view of the
molecular dynamics. The molecular-to-lattice **seam is a separate claim and takes no status
from this node, in any branch.**

That fence is not abstract, and this campaign found the instance that makes it sharp.
`holon-lens::field_lg` — rung 2's amendment A2, owned by `rung2-continuum`, frozen in
`RUNG2_PREREG_A2.md` — reads **the molecular scene as FHP-6 mode occupancy**. Its header
quotes the same operator instruction this node was given, and answers it the other way:

| | `holon-lens::field_lg` (rung 2, A2) | `holon-lattice` (node LG) |
|---|---|---|
| what it is | a **chart** on the molecular tier | a **tier** with its own dynamics |
| the motion | the molecular dynamics, unchanged | `T = S ∘ C` on a hex torus, this node's own |
| what FHP-6 supplies | the partition of velocities into modes | the state space AND the law that moves it |

The two agree about the **label** — both use `regplus::sector`, the one implementation, and
`holon-mesh/tests/rung2_lg_pin.rs` pins that agreement. **Agreeing about a label is not a
seam.** Neither node has established that one tier is a coarsening of the other, and this
document establishes nothing about it.

One vocabulary trap, named so nobody walks into it: **"collision" means two different things
in the two crates.** Here it is the local FHP transition, an element of a 4608-member group.
In `field_lg` it is a census view-collision — two frames sharing a coarse view. The word is
the same and the objects are unrelated.

---

## 1. WHAT EXISTED BEFORE, CONFIRMED

The inventory in `LG_PREREG.md` §1 stands after the campaign: **no lattice-gas dynamics
existed in either tree.** `field_lg` does not change that — it has no `step`, no collide and
no stream; its exported functions are `cartesian`, `mode_of`, `local_words`,
`readings_from_words`, `phase_defect`. The state space existed twice, the motion nowhere,
and both modules that would have carried it said so in their own headers.

---

## 2. GATE BY GATE

Every verdict below carries its work count, because a gate that reports PASS on work not
done has not passed (M-VACUOUS-SUCCESS). Total checks performed: **1,454,433**. Test phase: **32 tests, 32 passed, 0 failed** (28 lib, 2 FCHC control, 2 divisor sweep).

### 2.1 Instrument control, run first

| gate | verdict | reading |
|---|---|---|
| **G5** bijectivity | PASS, 294,912 checks | all **4608** enumerated laws are **distinct** conserving bijections on 64 states |
| **G11** census | PASS, 64 checks | FHP-6: **53** sectors, histogram **44 / 7 / 2** — `Core/Lattice.lean` reproduced |
| **G11** FCHC leg | PASS, `tests/fchc_control.rs` | 16,777,216 states → **72,047** sectors, largest **11,740**, through `holon-mesh::fchc` |
| **G12** isotropy | PASS, 4 checks | FHP-6 residual **4.441e-16** (T⁴ₓₓₓₓ 2.2500, T⁴ₓₓyy 0.7500); HPP-4 residual **0.6667** |

G11's FHP leg is a genuine control and not self-confirmation: this crate's label routine is
checked against `regplus::sector` on all 64 states, and `regplus`'s table is the one the Lean
pins. `fchc_control.rs` then compares **two crates'** answers about FHP-6 rather than each
against itself, and P5 confirms the enumerator moves when its mode set does (23 directions →
8,388,608 states, sector count ≠ 72,047).

### 2.2 Conservation — one gate per law, no epsilon anywhere

| gate | verdict | reading |
|---|---|---|
| **G1** mass | PASS, 20,000 checks | EXACT at every step of `L = 256`; total 137,820 particles |
| **G2** momentum-x | PASS, 20,000 checks | EXACT at every step; total −203 |
| **G3** momentum-y | PASS, 20,000 checks | EXACT at every step; total −203 |
| **G4** wall ledger | PASS, 6,000 checks | with a **32-cell** wall (counted from the scene, declared 32): mass EXACT, and `P(t) = P(0) + impulse(t)` EXACT; cumulative impulse **[84, −20]**, nonzero, so the gate did work |
| **G13** carrier moves | PASS, 3 counters | occupancy distance **at step 100** = **178,938** (> 19,661 = 0.30·L², the staked instant), 179,164 at end of run; **116,887,360** collisions fired, minimum **5,551 per step**; minimum per-step churn 177,846 (> 6,554 = 0.10·L²) |

**G2 and G3 read the same total, −203, and that is a coincidence — checked, not assumed.**
`examples/ledger_check.rs` recomputes both from per-direction counts at four configurations:
the two routes agree everywhere, and every other `(L, seed)` gives distinct components
(L=64: 66 / −127; L=128: −66 / 56; L=256 seed 0xBEEF: −44 / 22). At `L = 256` seed 0xC1A5
the two linear combinations of the six direction counts happen to coincide.

### 2.3 Leg A — HELD, gauged in both directions

| gate | verdict | reading |
|---|---|---|
| **G6** | PASS, 2,100 checks | **HPP-4 holds its per-line momentum chart EXACTLY** over 2,000 steps — 128 lines, a chart 64× finer than global. **FHP-6 breaks the same chart at step 0.** |

This is the gate that makes Leg A mean something. `v_L` is Held by G1–G3, but it is
**VACUOUS BY CONSERVATION** and is labelled so rather than counted — it is rung 2's flag,
an existence theorem in field-chart clothes. HPP-4 supplies what a global chart cannot: a
genuinely fine chart that genuinely holds, on the same instrument, at the same granularity
where FHP-6 fails. Historically this spurious invariant is the whole reason FHP exists
(Hardy–Pomeau–de Pazzis 1973 → Frisch–Hasslacher–Pomeau 1986), and it is why HPP is a bad
fluid; here it is the positive control.

**HPP-4's census is the sharper half of the contrast**: 15 sectors with exactly **one**
fiber above dimension 1, so HPP admits a collision group of order **2** where FHP-6 admits
**4608**. The square lattice has almost no dynamics to choose from.

### 2.4 Leg B — CLOSED, probed by construction

**The census's observed-fiber pairing could not have been used here, and using it would have
returned a vacuous pass.** On a moving lattice gas the coarse view essentially never repeats
between frames, so collecting frame pairs `(s,t)` with `v_s = v_t` finds nothing, and
"no witness found" would have been reported where the correct reading is "the instrument
cannot see". M-FIXED-POINT-TRAJECTORY makes the same point from the other side and instructs
staking closure over configurations. So the fiber is **built**:

> given `x`, replace one cell's state by its cyclic successor within its own `(N,P)` fiber.
> The label is unchanged, so `v_b(y) = v_b(x)` for **every** `b` at once.

One identical perturbation therefore serves the whole chart family, and no confound enters
the curve from the probe changing with `b`.

**Independent corroboration that this is the right KIND of chart**, from a lane that tried
the other kind: `RUNG1_RESULTS.md`'s Leg F measured that **geometric-predicate** charts —
H-bond networks built from distance and angle criteria — factored through **nothing**. This
node's fibers are conserved-label classes, closed by construction for sector-preserving
collisions. Their lesson, written to this node by name: a chart meant to compose into a
conserved-label fiber census should be built from conserved labels, not from geometry. It is
corroboration and not a stake — it arrived after this freeze, and nothing in §2 depends on
it — but it means the `(N,P)` chart choice does not rest on taste.

**The measured curve, at `L = 64`, 81,920 probes per chart** (every cell × every one of the
20 movable states). `Exhaustive` has no sampling in it, which is what makes an exact equality
meetable; the as-configured column reads the lattice's own movable cells once each and carries
finite-position scatter, as it should.

| `b` | measured | derived `W(b)` | probes | as-configured | |
|---:|---|---|---:|---|---|
| 1 | 1.0000000000 | 1.0000000000 | 81,920 | 1.000000 | |
| 2 | 1.0000000000 | 1.0000000000 | 81,920 | 1.000000 | |
| 4 | 0.7500000000 | 0.7500000000 | 81,920 | 0.744301 | |
| 8 | 0.4375000000 | 0.4375000000 | 81,920 | 0.444995 | |
| 16 | 0.2343750000 | 0.2343750000 | 81,920 | 0.243806 | |
| 32 | 0.1210937500 | 0.1210937500 | 81,920 | 0.126858 | |
| 64 | 0.0000000000 | 0.0000000000 | 81,920 | 0.000000 | **VACUOUS BY CONSERVATION** |

**The light cone**, which is what distinguishes a boundary effect from a chart that is
actually closed: at `b = 16`, advancing `k = 1, 2, 4, 8, 16` steps gives 0.2438, 0.4014,
0.6660, 0.9861, **1.0000** — the defect saturates exactly when the cone has crossed the
block. A closed chart would have stayed at zero.

| gate | verdict | reading |
|---|---|---|
| **G7** the defect law | PASS, 573,440 probes | the measured `k = 1` defect **is** `1 − max(0,b−2)²/b²` at all 7 charts, **exactly** |
| **G8** witnesses exhibited | PASS, 6 exhibits | 6 of 6 non-vacuous charts exhibit a witness pair. At `b = 4`: cell 0, state 9 → 18, **both labelled `(2,0,0)`**; the agreeing view has 256 blocks and the stepped views differ in **3** of them |
| **G9** independent probe sets | PASS, 7 sets | each `b` probed independently with its own fiber moves; no `b`'s verdict inferred from another's (M-FINAL-VIEW-COLLISIONS) |
| **G10** two-sided probe gauge | PASS, 163,840 checks | negative control (`y = x`) rate **0.0**; positive control (cross-fiber) rate **1.000** |

### 2.5 The inhomogeneity discharge

A periodic torus is spatially homogeneous, so a locality-shaped result on one may hold for a
homogeneity reason instead (M-HOMOG). A bounce-back wall breaks translation invariance:

| `b` | wall-free rate | derived | probes | wall-touching cells (reported, not averaged in) |
|---:|---|---|---:|---:|
| 2 | 1.0000000000 | 1.0000000000 | 80,640 | 64 |
| 4 | 0.7500000000 | 0.7500000000 | 79,360 | 128 |
| 8 | 0.4375000000 | 0.4375000000 | 76,800 | 256 |
| 16 | 0.2343750000 | 0.2343750000 | 71,680 | 512 |
| 32 | 0.1210937500 | 0.1210937500 | 40,960 | 2,048 |

**G14 PASS, 349,440 probes.** The law survives a structurally inhomogeneous graph. Blocks
touching the wall are reported and **never averaged into the curve** — averaging them would be
the aggregate this node's door shape refuses, one level down.

### 2.6 The collision-law sweep

**G15 PASS, 4,608 laws.** All 4608 sector-preserving collision laws give **one** distinct
`k = 1` defect rate at `b = 4`, `L = 8`: **0.75**, identical to the derived value. **The
identity collision is in that sweep**, so the reading survives removing the collision
entirely.

This is what turns M-ONE-MODEL-DELTA's "worse than the one model you chose" into a statement
about the lattice. The closure failure of the coarse charts is a property of streaming on a
lattice, and **no choice of REG+ collision law removes it.**

### 2.7 Plants

Carrier state **9** — the head-on pair — has a population of **98 cells**, nonzero in the
`(N=2, P=0)` sector the plants act on, asserted before any plant was read (M-PLANT-SECTOR).
Every plant fired its own gate and no other.

| plant | edit | fired | expected | |
|---|---|---|---|---|
| **P1** mass | `9 → 0` | `[mass]` | `[mass]` | FIRED, ISOLATED |
| **P2** momentum-x | `9 → 34` | `[Px]` | `[Px]` | FIRED, ISOLATED |
| **P3** momentum-y | `9 → 5` | `[Py]` | `[Py]` | FIRED, ISOLATED |
| **P4** bijectivity | `18 → 9` | bijectivity fails **while conservation holds** | — | FIRED, isolated from G1–G3 |
| **P5** census | one direction perturbed | census reads **55**, not 53 | — | FIRED |
| **P6** isotropy | HPP-4 through the tensor path | a **failing** row printed | — | FIRED |
| **P7** wall ledger | impulse dropped from the ledger | the identity breaks | — | FIRED |
| **P8** probe | fiber move → no-op | reads **0.0** over 81,920 probes, cannot reproduce `W(4) = 0.75` | — | FIRED |
| **P9** fixed point | a state the dynamics fixes | G13's counter refuses it | — | FIRED |

P8 is the one that matters most: it is the plant against a closure result produced by a probe
that never perturbed anything, and it has to do **both** things — read exactly zero, and fail
to reproduce `W(b)` at a chart where `W(b)` is not zero. Reading zero alone is what a correct
global chart also does.

---

## 3. THE DEFECT LAW BEYOND THE STAKED GRID

`LG_PREREG.md` §5.3's table lists powers of two at `L = 64`, because that was the campaign's
grid — and a law checked only where it was staked is a law checked only where it was
convenient. It is not restricted to them. Two independent sweeps, over every divisor:

| sweep | lattice sizes | charts | result |
|---|---|---|---|
| `ref_defect_allmodels.py` (the frozen reference) | 12, 24, 30, 36 | 31 | exact at all 31 |
| `tests/defect_law_divisors.rs` (gated in CI) | 12, 18, 24, 30 | 28 | exact at all 28 |

`W(3) = 8/9`, `W(5) = 0.64`, `W(9) = 0.3950617284`, `W(15) = 0.2488888889`, `W(18) =
0.2098765432` — odd blocks, non-power-of-two blocks, and lattice sides that are not powers of
two, all reproduced to better than `1e-12`.

This is an **extension of the staked claim, obtained after the freeze**, and is reported as
one. The freeze's own table remains the pre-registered part, and nothing in §2 depends on
this section.

---

## 4. WHAT THIS DOES AND DOES NOT CERTIFY

**Certified, integer-exact, no tolerance:**

1. The tier **runs on its own dynamics**, with mass and each momentum component conserved
   as an integer identity at every step, and the wall impulse a term in the ledger rather
   than a tolerance.
2. The motion is a **bijection**, verified on the 64-state table for every law in the group
   and on the full micro-state by round-trip.
3. **The census classifies the dynamics.** A sector-preserving collision permutes within
   `(N,P)` fibers and can do nothing else, so `sector_dims` 44/7/2 states exactly where a
   law may act: the identity on all 44 fibers of dimension 1, and the whole space of REG+
   collision laws on FHP-6 is `S₃ × (S₂)⁷ × S₃`, order **4608**. FHP-I is one named element:
   the 3-cycle on `{9,18,36}` — the Lean's own `three_route_sector` — and the swap on
   `{21,42}`. The other dimension-3 fiber, `{27,45,54}`, is its particle-hole dual.
4. **The coarse charts are not closed views**, with the defect an exact boundary fraction
   and the only closed chart the vacuous conserved one.

**NOT claimed — the Navier–Stokes limit.** Only the *necessary* lattice condition
(fourth-rank isotropy) is measured. The exit is named and unmoved: a measured kinematic
viscosity against the model's own prediction, semi-detailed balance of the collision table,
and the `g(ρ) ≠ 1` Galilean defect. **No document in this programme may say this tier has a
Navier–Stokes limit until those three are run.**

**NOT claimed — the seam.** §0.

**NOT run — FCHC-24.** Priced in `LG_PREREG.md` §8 and gated on FHP-6 banking, which this
document does. Its ledger needs momentum arity 2 → 4 first, which `fchc.rs` already flags.

---

## 5. WHAT THE INSTRUMENT FOUND ABOUT ITS OWN DESIGN

Reported as plainly as the survivals, per rule 7. Four defects, each found because the
derived law refused to be reproduced, and each now written into the code as the reason a
line reads the way it does.

1. **`line_momenta` summed each momentum component along its own axis** — a quantity nothing
   conserves. It read "not held" on HPP-4 as loudly as on FHP-6, so the Leg-A gauge had no
   sides at all and would have passed as a one-sided check. The line a component is summed
   along is the one that component's movers do not leave: `Px` over constant `j`, `Py` over
   constant `i`.
2. **The probe's first sampler biased position within the block.** It picked a random index
   and scanned forward to the next movable cell, which over-samples cells following a run of
   unmovable ones — and position within the block is the one quantity the defect law is
   about. It read 0.7025 against a derived 0.75, and *exceeded* the geometric bound at
   another size. A sampler defect wearing a physics result's clothes.
3. **The second sampler double-counted its own precision.** 20,000 draws with replacement
   from ~300 distinct movable cells, with a binomial band quoted on 20,000 — which turned a
   0.65σ agreement into a 5σ disagreement. An enumerated count is not an effective count.
   The populations are now explicit and named: `Exhaustive` has no sampling in it at all,
   `AsConfigured` enumerates each cell once and bands on that number.
4. **A test asserted HPP-4's census at a guessed 12.** It is 15. The correction sharpened the
   contrast rather than blunting it (§2.3).

**A gap in the freeze itself, reported rather than quietly resolved.** `LG_PREREG.md`'s G7
says the measured rate "equals `W(b)`" without naming the probe's **population**, and the
two available populations answer different questions. G7 is nonetheless pinned, by its own
second clause — agreement with the frozen Python reference, whose population is every
`(position, movable state)` pair — so `Exhaustive` is what the freeze stakes and the
ambiguity changed no verdict. It should have been written down in the freeze.

**Two places the driver did not run what the freeze staked**, found by reading the freeze
against the code rather than by any gate failing. Both were corrected and the campaign was
**re-run from scratch** on the corrected instrument (`f933c51`); the killed run's partial
output is not banked.

* **G13 stakes the carrier-motion distance at step 100.** The driver read it at the end of a
  20,000-step run — a different, later instant on a quantity that saturates, so it would have
  passed on a carrier that took 10,000 steps to start moving. Both instants are now recorded
  and the gate turns on the staked one.
* **The wall is 32 cells.** §6.4 says so and G4's row defers to §6.4 by naming it; the driver
  used 48 in one configuration and `L/4` in the other, so the two obstacle gates were not run
  on the same obstacle.

Neither changed a verdict, which is why they are worth recording rather than explaining: a
freeze's chosen instant and chosen size are part of the freeze, and "the answer came out the
same" is not available as a defence before the answer is known.

**And then the G4 line printed `48-cell wall` while `add_wall` was building 32**, caught by
reading a live log rather than by any gate. The tell was the impulse: the wall had changed and
the cumulative impulse moved `[162,−166] → [84,−20]` exactly as it should, while a literal in
the message stayed behind. **The measurement was right the whole time and the diagnostic lied
about the parameter it ran at** — and `lg_full.log` is the banked artifact, so a reader has
nothing else to go on. The fix was not to retype 32: the label is now **counted from the scene
the run holds** and asserted against the declared constant, so a wall length and the sentence
describing it cannot drift apart again. Third launch; the two killed runs are not banked.

**The same defect then appeared in this document**, which is the part worth admitting. §2.2's
G4 row was drafted while the second run was live and carried its 48-cell wall and its
`[162,−166]` impulse — figures from a run that was killed and never banked. Caught by
cross-checking every quoted figure against the banked log before committing, which is now
what was done: **every number in §2 appears verbatim in `lg_full.log`**, and no figure from a
killed run survives anywhere in this file. A results document drafted alongside a running
campaign will pick up the campaign it was drafted beside, not the one it reports.

**A pricing miss.** `LG_PREREG.md` §8 priced the probe stage at ~9.1e8 cell-updates, assuming
4,096 probes per chart. `Exhaustive` at `L = 64` is 81,920 probes per chart, so the stage ran
about 5× its price. Over-running a price is not the refusal condition M-CHEAPER-THAN-ITS-PRICE
names — that one is a result arriving *cheaper* than its model — but the miss is recorded.

---

## 6. THE LIVE READOUT FOR THE WORKBENCH

Named in `LG_PREREG.md` §12 before the page existed, and unchanged by the result:

> **DOOR SHAPE: DEFECT-AGAINST-VIEW.** `W(b)` against `b` — the closure defect as a function
> of coarse-graining scale, the derived closed form as the curve and the measured points on
> it.

**This node's certificate confers NO band state.** The band-flip law (FSD `b374773`) is that
a band goes live only on a **node-G closure certificate** — a certified coarse view of the
dynamics beneath it. This tier is certified on **its own** dynamics, which is a different
thing; running it under a band would be running physics that is not the certified coarse
truth of that scene. §0's first law already forbade this from the other direction. What
follows are therefore requirements on **research content the page may cite**, never on a band.

Three requirements on whatever `workbench-engine` builds, which are part of the freeze and
not of the page's design freedom:

1. **The `b = L` point is drawn as the VACUOUS end of the axis, labelled as closing by
   conservation alone** — never as the curve's success.
2. **No aggregate.** A single scalar "defect" collapses the only axis this tier's reading
   has, and would let `b = L`'s exact zero be averaged into a pass.
3. **The band may not say the fluid tier is certified closed, under any phrasing**, and may
   say nothing at all about the molecular seam.

---

## 7. KILLS

None fired. Each remains live and separable, as staked in `LG_PREREG.md` §10: K1 the
classification, K2 the defect law, K3 its lattice-locality, K4 the vacuity reading, K5 the
isotropy warrant, K6 the Leg-A gauge. K4 is the one worth watching — a `b < L` with witness
rate exactly zero would be a genuinely closed non-vacuous fluid chart, and would be checked
against the HPP-4 spurious-invariant pattern first, because a chart that closes for a
spurious-invariant reason is a defect of the model rather than a tier.

---

## 8. THE INVARIANT QUESTION — SOLVED RATHER THAN STAKED

The lead suggested staking in advance that the **staggered (Zanetti) momentum invariants**
would show up as extra closed views at a staggered chart, and said to verify the literature
first. Staking a remembered formula would have made the answer depend on the recall, and the
question is sharper than that: **any** spurious invariant is an extra exactly-closed view, and
§4's "the only exactly-closed chart is the global one" is false if one exists.

So the space was **solved**, not sampled. `ref_invariants.py` finds every linear invariant of
`T = S ∘ C`, so a spurious one of any form must show up in a dimension count.

**The derivation, which costs nothing and removes most of the unknowns.** Write
`L(x) = Σ_{c,d} w[c][d]·x_d(c)`. Then `L(Tx) = Σ_{c,d} w[c+DIR[d]][d]·C(x)_d(c)`. Every
single-particle state is alone in its `(N,P)` fiber, so any sector-preserving `C` fixes all
six of them — and those six states **alone** force `w[c+DIR[d]][d] = w[c][d]`: the weight is
constant along the lines in direction `d`. That collapses `6L²` unknowns to one per
(direction, line), and what remains is a per-cell condition imposed for every local state the
collision moves.

### 8.1 The instrument is gauged on two systems whose answers are known independently

Without this, "FHP-I has none" would be a statement about the solver.

| system | invariants found | expected | |
|---|---|---|---|
| identity collision (streaming alone), `L` = 4, 6, 8 | 24, 36, 48 | `6L` — one per (direction, line) | exact |
| **HPP-4**, `L` = 4, 6, 8, 12 | 9, 13, 17, 25 | `2L+1` — its textbook per-line momenta | exact |

HPP-4 is the one that matters: its spurious invariants are real, known, and the historical
reason FHP exists, and the solver finds **exactly** them — `2L−2` spurious quantities beyond
mass and the two momenta. A solver that can find HPP's can find FHP's.

### 8.2 The reading

> **FHP-I on the hex torus has exactly THREE linear invariants — mass and the two momentum
> components — at every size tested: `L` = 4, 6, 8, 10, 12, 16. Zero spurious.**

So on this configuration there is no staggered linear invariant to find, and §4's statement
stands: the only exactly-closed chart in the family is the global one, and it is vacuous.

### 8.3 What this does NOT say

**It does not contradict Zanetti, and it must not be read as doing so.** This session's
web-search budget was exhausted before the citation could be read, and the one page reachable
by direct fetch does not discuss spurious invariants at all. So this node has **not read**
Zanetti's exact statement, its model variant, or its scope — FHP-I here is one specific
element of a 4608-member group, without rest particles, on a periodic hex torus, and any of
those could be the difference.

The claim is therefore exactly this and no more: **on this configuration, gauged on two
systems whose invariant spaces are known independently, the complete space of linear
invariants is three-dimensional.** Whether that agrees with, refines, or is orthogonal to
Zanetti's result is **owed work**, named here as owed rather than quietly dropped, and it is
the one loose end this node hands on.

It also says nothing about **non-linear** invariants. The solver searches linear functionals
of the occupation numbers; that is the class staggered momenta belong to, and it is not
everything.
