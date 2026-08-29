# SELECTOR-4 — the gauntlet on gauge worlds, scale-indexed by acuity

*2026-08-28. Prereg frozen ALONE before the instrument existed (commit
7cde5c2); instrument and its run log committed together (commit 25eae60,
`selector4.py` + `selector4.log`). Exit 2: a valid run with a staked gate
killed. **G1 branch (a), F1 branch (a), Z1 branch (b), all three plants
FIRED.** Two cells refuse rather than guess and are excluded from every
gate. The bootstrap selection principle REACHES gauge worlds and
DISCRIMINATES between them; the zoom law is false as staked, for a reason
that was derivable before the run.*

## The instrumented family

Four group-torus worlds, state space G×G, dynamics the mapping-class step
S∘T, bijectivity verified before any reading.

| G | \|G\| | states | ord(step) | family | acuity blocks A0..A4 | non-trivial fiber-injective step-cycles A0..A4 |
|---|---|---|---|---|---|---|
| D4 | 8 | 64 | 6 | 16 | 1, 2, 2, 25, 28 | 0, 0, 0, 9, 11 |
| 2T | 24 | 576 | 48 | 60 | 1, 2, 3, 66, 76 | 0, 0, 0, 9, 15 |
| 2O | 48 | 2304 | 144 | 105 | 1, 2, 5, 89, 136 | 0, 0, 0, 25, 199 |
| 2I | 120 | 14400 | 8400 | 480 | 1, 2, 9, 153, 296 | 0, 0, 0, 61, 451 |

### The three constructions the freeze delegated, and the measurement that forced one

**The acuity ladder is CUMULATIVE.** Rung k's view is the join of readings
0..k. Read non-cumulatively the ladder is not a chain at all — (class a,
class b) does not determine the class of [a,b] — so the join is not a
convenience, it is what makes "coarsening" true. B0 verifies the chain
rather than assuming it.

**The act vocabulary had to be generalized, and the forcing fact was
measured before anything was chosen.** SELECTOR-3's rule — a dynamics-invariant
transversal, exactly one state per acuity fiber — admits **ZERO sections at
every rung above A0 on all four groups**. The obstruction is arithmetic: at A1
a transversal needs two fixed points in different fibers or a 2-cycle spanning
them, and the step has exactly one fixed point and no 2-cycles. A vocabulary
empty everywhere cannot be scored, so "exactly once" was relaxed to "at most
once": **invariant PARTIAL sections**, whose atomic generators are the
fiber-injective cycles of the dynamics. This is a strict generalization, not a
replacement — for disjoint invariant P, Q one has `a_{P∪Q} = a_P ∘ a_Q`, so
every full-section act is a word in these — and it makes plants (i) and (iii)
fire by construction rather than by luck.

**That generalization was checked backwards, because it could have flattered
the earlier result.** Applied to SELECTOR-3's own model it selects **exactly
the same set**: 8/48 at n=3 and 60/384 at n=4, the same sets and not merely the
same counts. SELECTOR-3's verdict is robust to the vocabulary that SELECTOR-4
required.

**The family is declared and restricted**, the freeze's escape clause being
needed because |G|·ord(step) reaches 1,008,000 at 2I:
`F(G) = { gauge_r ∘ step^d : d | ord(step), r a conjugacy-class rep }` — one
canonical generator per cyclic subgroup of ⟨step⟩, dressed by one
representative per gauge orbit. 20/70/120/540 labels, collapsing to
16/60/105/480 distinct permutations. It contains the identity (plant (i)'s
carrier) and the step itself, and carries no word-length or size knob.

### The VOID discipline

A cell whose verdict could not be decided at the declared node budget carries
**no verdict and is excluded from EVERY gate**, G1, Z1, F1 and FR1 alike. Its
printed selected count is reported for information only and is a **lower
bound**, because an undecided pair could only ever ADD to a world's identity
set.

---

## Gate by gate

### B0 — **PASS**

Step and gauge action verified bijective on all four worlds; the acuity ladder
verified a chain of coarsenings at every rung; identity sets by the exact
ρ-closure with **no word-length cut** (the pair-BFS reorganized: reachable
pairs from the diagonal are exactly the graphs of `ρ_w = w[T'] ∘ w[T]⁻¹`).

**718 budget exhaustions**, all inside 2I's fine rungs: **351 at 2I A3** and
**367 at 2I A4**. Those two cells — and only those two — are VOIDED.

### G1 — group discrimination reaches gauge worlds · **branch (a)**

**6 of 20 (G, Ak) cells select a proper nonempty subset:**

| cell | selected |
|---|---|
| D4 A3 | 4/16 |
| D4 A4 | 4/16 |
| 2T A3 | 35/60 |
| 2T A4 | 35/60 |
| 2O A3 | 44/105 |
| 2O A4 | 48/105 |

The principle reaches gauge worlds. Every coarse rung (A0, A1, A2) selects
nothing in every group; selection switches on at A3 and only there.

### Z1 — THE ZOOM LAW · **branch (b), KILLED**

Selected-set sizes A0→A4:

| G | sizes | monotone non-increasing (A1..A4) | nested |
|---|---|---|---|
| D4 | 0, 0, 0, 4, 4 | **False** | False |
| 2T | 0, 0, 0, 35, 35 | **False** | False |
| 2O | 0, 0, 0, 44, 48 | **False** | False |
| 2I | 0, 0, 0, 189, 191 | **False** | False |

Inversions at (D4, A2→A3), (2T, A2→A3), (2O, A2→A3), (2O, A3→A4),
(2I, A2→A3), (2I, A3→A4). The kill is stated at full strength in its own
section below.

### F1 — group discrimination · **branch (a)**

Selected fraction per rung, decided cells only:

| rung | D4 | 2T | 2O | 2I |
|---|---|---|---|---|
| A0 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| A1 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| A2 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| A3 | 0.2500 | 0.5833 | 0.4190 | *0.3937* **VOID** |
| A4 | 0.2500 | 0.5833 | 0.4571 | *0.3979* **VOID** |

The verdict rests on the decided cells alone: at A3, 0.2500 vs 0.5833 vs
0.4190 across three different gauge groups. **This is the first measured
instance of an Ω-internal criterion distinguishing gauge structures** — the
door that "particle content from selection" would eventually have to go
through. It is a door, not a passage: see what no outcome claims.

### FR1 — the fraction across |G| · **measured, three points**

| \|G\| | selected | fraction |
|---|---|---|
| 8 | 4/16 | 0.2500 |
| 24 | 35/60 | 0.5833 |
| 48 | 48/105 | 0.4571 |
| 120 | *191/480* | *0.3979* — **VOID**, lower bound only, no verdict |

**The sequence is complete only for |G| = 8, 24, 48, and it is not
monotonic** — it rises then falls. The |G| = 120 entry is undecided at the
declared budget and **is not a data point**. Per the freeze's own honesty
clause, no law is fitted to these numbers and none is suggested; none was
derived in advance, and four points (let alone three) would make any fit
post-hoc by construction.

### Plants — **all three FIRED**

| plant | outcome | carrier · sector |
|---|---|---|
| (i) stasis | **FIRES** | the identity world · its generated act set. It fails at every rung of every group, and every act its subsystems generate is the identity map — it dies on emptiness of **ACTION**, never of structure. |
| (ii) T3 control | **FIRES** | any closed world · its view-aligned identity set. The view-aligned vocabulary separates nothing on closed views: **20 carriers, 115 candidate pairs scored**. Break.lean's `vcov_preservesGauge_of_closed` is the proof; this is the live control. |
| (iii) blind rung | **FIRES** | any world · A0's act set. At A0 every act is the identity map for every world in every group, and **nothing is selected anywhere** — selection requires acuity, and the zoom axis has a floor. |

---

## The kill, reported as plainly as the survivals

**Z1 died as staked, and it was forced to.** The freeze staked that the
selected set is monotone NON-INCREASING along A1→A4 — finer observers exclude
more worlds, coarser acuity forgives. The measurement is the opposite in every
one of the four groups, and the reason is structural rather than empirical:

> Refining the acuity refines the view AND refines the fiber-injectivity test
> that defines the act vocabulary. So both the view and the vocabulary grow
> with acuity; separation is therefore monotone; therefore **|Ident| is
> monotone non-increasing** and the **SELECTED set is monotone
> non-DECREASING**.

This is derivable from the declared construction without running anything.
The stake could not have come out any other way, and it did not.

**Both halves stand, and neither is softened.** The acuity intuition
SURVIVES — something does nest along the ladder, exactly as the principle
says, and it is **indistinguishability**: what a finer observer inherits is a
smaller set of worlds it cannot tell apart. The STAKE as written DIES — the
quantity it named, the selected set, moves the other way. Selection is not the
thing that nests. A re-stake of the zoom law on |Ident| is available and is
not claimed here; that would be a new freeze, not a rescue of this one.

**A structural rhyme, recorded for the process and not for the physics.**
This is the second stake today whose direction was wrong BY CONSTRUCTION and
provable before the run. SATURATION-1's T2 staked the truncation systematic on
the LONGEST side of the triangle, when the quantity that actually governs the
decay is the SECOND-SMALLEST side — a near-collinear chain's longest side is
the sum of two short ones and is not a distance anything decays over. There,
as here, the intuition was sound and the quantity it was attached to was not.
**The lesson is a process one: a stake-direction proof belongs in the freeze
review.** Before a monotonicity or decay direction is frozen, someone should
be required to derive the direction from the declared construction, because
where that derivation is cheap it is also decisive, and it costs a campaign to
skip it.

---

## What NO outcome here claims

Leptons. Groups-of-nature. Kinematics. Constants. The chain's rungs 3–4 remain
open regardless of anything on this page.

G1(a) means the criterion reaches gauge worlds; F1(a) means it tells four
gauge structures apart on the family instrumented here. Neither says which
gauge structure is nature's, and nothing here selects our universe.

---

## Reproduction

`python3 conformance/omega/selector4.py` — exact throughout (integer
permutation arithmetic; no float enters any decision), self-contained but for
`conformance/gravity/{binary_groups,bridge1}.py`. Exit 0 = all gates passed
and plants fired; **2 = valid run, a staked gate killed** (this run); 1 =
instrument invalid (VOID, internal inconsistency, or a missed plant). Total
runtime is dominated by 2I at 1137 s; the other three worlds total under 22 s.
The run log committed beside it is the reading this document reports.
