# SELECTOR-5 — ADVERSARIAL REFUTATION

*Refuter lane, 2026-08-30. Instruments: `refute_lib.py`, `refute_baserate.py`,
`refute_c4.py`, `refute_c4_provenance.py`, `refute_dedup.py`, `refute_funnels.py`,
`refute_population.py`, `refute_extended.py`. Labelling rule frozen BEFORE any
refutation number was computed, in `FROZEN_LABEL_RULE.md`
(sha256 `0a4b9565…3bad0d913`).*

---

## VERDICT FIRST

| # | Claim in `SELECTOR5_RESULTS.md` | Verdict |
|---|---|---|
| 1 | "44 of the 45 survivors (97.8%) are authentic Lie-type / Standard Model gauge subgroups" | **REFUTED AS EVIDENCE.** The number reproduces. The base rate of the same label in the unfiltered population is **94.4%** — a figure the sweep's own script prints on line 5 of its output and the results file omits. The funnel buys 3.4 percentage points. |
| 2 | "the frozen SELECTOR-4 bootstrap gauntlet" | **REFUTED.** C1–C4 appear nowhere in `SELECTOR4_PREREG.md`, which stakes a different criterion on a different object. No SELECTOR-5 pre-registration exists. |
| 3 | Gate C4 measures a "Schur multiplier / projective obstruction" | **REFUTED.** C4 is reproduced exactly, on all 430 groups, by a lookup on the construction tag, the group's name, and `|G|`. **Zero of the 45 survivors** owe C4 to a property computed from the group. |
| 4 | The 97.8% measures selection toward gauge structure | **REFUTED — CIRCULAR.** Inside the pool C4 acts on, the ground-truth label is a **deterministic function of the construction tag** (0 of 10 families carry a mixed label; a classifier reading only the tag scores 138/138). C4 is another function of that same tag. |
| 5 | "430 non-isomorphic finite groups … the entire finite group landscape \|G\| ≤ 128" | **REFUTED, twice.** The landscape has **at least 3,014** isomorphism types, so the sweep covers under 15% of it. And the 430 are not non-isomorphic: three duplicate pairs are confirmed isomorphic by explicit isomorphism search. |
| 5b | The population choice is incidental | **REFUTED — it is load-bearing.** Adding every split metacyclic group (which strictly generalises five of the census's own eleven families) gives 261 new isomorphism types. Precision falls to **63/68 = 0.9265**, false positives go **1 → 5**, and **all 23 new survivors have \|G\| ∈ {24, 48, 120}** — off its hard-coded family list, the gauntlet is a filter on the value of \|G\|. |
| 6 | The gauntlet is a criterion on groups | **REFUTED.** It is not an isomorphism invariant. A₄ and Δ(12) **are the same group**; A₄ passes the gauntlet and Δ(12) fails it. |
| 7 | Gate-by-gate metrics (§2), scale-indexed thinning (§3), family table (§4) | **REFUTED — NOT REPRODUCIBLE.** Every cell disagrees with the committed instrument's own printed output. |
| 8 | "Systematic / monotone thinning … proving selection" | **REFUTED.** Monotonicity of a nested conjunction of filters is a theorem about conjunctions, not a result — C1 ⊇ C1∧C2 ⊇ C1∧C2∧C3 ⊇ C1..C4 holds for any four predicates whatsoever. And the per-scale passing fractions are **not** monotone in either version: 0.049 / 0.145 / 0.136 / 0.084 in the script, 0.0385 / 0.1724 / 0.1318 / 0.0838 in the results file. |
| 9 | Gates C1 and C3 as "physical extinction mechanisms" | **SURVIVES AS A RESTATED DEFINITION.** C1 is literally "G is nonabelian"; C3 is literally "G has a non-real conjugacy class". That dihedral and symmetric groups fail C3 is a classical theorem, not a measurement. C3's ancestry (FROB-ORIENT-1) is genuine. |
| 10 | One pretty funnel is evidence | **REFUTED BY CONSTRUCTION.** Two alternative funnels of equal or lesser complexity select two *other* famous physics families from the same 430 at **perfect precision and perfect recall**: the Pauli / Weyl–Heisenberg tower (4/4, 4/4) and the crystallographic point groups (9/9, 9/9). SELECTOR-5 gets 44/45 precision at 44/406 recall against a target that is 94% of its population. |

**Nothing in SELECTOR-5 should be banked.** The reproducible content is: a
census of 427 isomorphism types drawn from eleven hand-chosen families, and the
observation that nonabelian groups with a non-real conjugacy class, intersected
with a hard-coded family list, are mostly groups from that hard-coded family list.

---

## 0. Reproduction

`python3 landscape_sweep.py` runs in 7 s and yields **45 survivors**, the same 45
named in `SELECTOR5_RESULTS.md` §5. Everything below starts from that.

---

## 1. The base-rate / tautology test

### 1.1 The frozen labelling rule

Two rules, both frozen in `FROZEN_LABEL_RULE.md` before any number below existed.

**RULE-A** is what the code does: `is_lie_type` is a **construction tag** set per
builder, never computed from the group. Cyclic → True. Dihedral → True. Dicyclic →
True. Binary polyhedral → True. Δ(3n²) → True. A₃/A₄/A₅, S₂/S₃/S₄, GL(2,3), D₈ →
True. Semidihedral, modular, Frobenius (except F₆ = S₃), Heisenberg (except D₈) →
False. Direct products inherit by `and`. Deduplication promotes to True on a
collision and never demotes.

**RULE-B** is the defensible reading of the English sentence:

> SM(G) := G is isomorphic to a subgroup of SU(3) × SU(2) × U(1),

decided exactly, from the character table, as: *there exist a degree-3
representation A with det A = 1, a degree-2 representation B with det B = 1, and a
linear character χ, whose kernels intersect in the identity.* That is literally the
statement that some homomorphism into the product group is injective.

RULE-B is implemented in `refute_lib.py` by Burnside–Dixon character tables
(validated by degree-sum, integrality and column orthogonality on every group) plus
Frobenius-style determinant characters recovered from power maps. It returns the
right answer on every independently checkable case: A₄, S₄, Q₈, D₈, 2T, 2I, A₅,
UT(3,3), SD₁₆, M₁₆, F₂₁ and Z₁₂₈ all embed; **S₅, UT(3,5) and (Z₂)⁵ do not**.

### 1.2 The two facts RULE-B fixes in advance, both theorems

1. **Generosity.** SU(3) contains U(2) via `A ↦ diag(A, det A⁻¹)`. So *every* finite
   group with a faithful complex representation of degree ≤ 2 is an SU(3) subgroup —
   every cyclic, dihedral, dicyclic, generalized quaternion, semidihedral, modular
   and binary polyhedral group, and every subgroup of any of them.
2. **Rank obstruction.** An abelian subgroup of SU(3) × SU(2) × U(1) has rank
   ≤ 2 + 1 + 1 = 4. So (Z₂)⁵, (Z₂)⁶ and (Z₂)⁷ are **not** SM subgroups — and RULE-A
   labels all three True, by the product-inheritance rule.

### 1.3 The measurement

| pool | n | RULE-A SM | rate | RULE-B SM | rate |
|---|---|---|---|---|---|
| P0 — all groups | 430 | 406 | **0.9442** | 402 | **0.9349** |
| P1 — post-C1 | 256 | 232 | 0.9062 | 231 | 0.9023 |
| P13 — post-C1+C3 | 138 | 115 | 0.8333 | 125 | 0.9058 |
| P123 — the pool C4 acts on | 138 | 115 | 0.8333 | 125 | 0.9058 |
| **S — the 45 survivors** | 45 | 44 | **0.9778** | 44 | **0.9778** |

**The headline 97.8% is measured against a base rate of 93.5–94.4%.** Drawing 45
groups at random from the population, with no funnel at all, is expected to read
about 94%. The funnel's entire yield over the population is **+3.4 percentage
points (RULE-A)** or **+4.3 (RULE-B)** — one to two groups.

`landscape_sweep.py` prints this base rate itself:

```
Standard Model / Lie-type Finite Subgroups: 406 (94.4%)
Generic / Non-Gauge Finite Groups: 24 (5.6%)
```

`SELECTOR5_RESULTS.md` does not contain the figure 406, the figure 94.4%, or any
base rate.

### 1.4 The published label is wrong on 30 of the 430 groups

RULE-A and RULE-B disagree on 30 entries, and in every case RULE-B is right:

**RULE-A says gauge subgroup, RULE-B says no (17):** (Z₂)⁵, (Z₂)⁶, (Z₂)⁷ — barred
by the rank obstruction, rank 5, 6 and 7 against a maximum of 4 — and the fourteen
products of two nonabelian factors D₆×D₆, D₆×D₈, D₆×D₁₀, D₆×D₁₂, D₆×D₁₆, D₈×D₈,
D₈×D₁₀, D₈×D₁₂, D₈×D₁₆, D₁₀×D₁₀, D₁₀×D₁₂, D₆×A₄, D₈×A₄, D₁₀×A₄. These fail because
SU(2) has no dihedral, symmetric or alternating subgroup — by McKay's classification
its finite subgroups are exactly the cyclic, dicyclic and binary polyhedral ones —
so once one nonabelian factor occupies SU(3), the other has nowhere faithful to go.
All fourteen are labelled True purely by `G1.is_lie_type and G2.is_lie_type`.

**RULE-A says no, RULE-B says gauge subgroup (13):** F₂₁, F₃₉, F₅₇, F₉₃, F₁₁₁ (all
genuine SU(3) subgroups — F₂₁ = Z₇⋊Z₃ is a classical one), and SD₁₆, SD₃₂, SD₆₄,
SD₁₂₈, M₁₆, M₃₂, M₆₄, M₁₂₈, every one of which has a faithful 2-dimensional
representation and hence sits inside U(2) ⊂ SU(3).

The label is not a measurement of anything; it is a per-builder annotation with a
7% error rate, and its errors run in both directions.

### 1.5 Permutation control

200,000 draws of a uniform random 45-subset of P123, scored by each rule:

| rule | observed precision on S | base rate in P123 | permutation mean ± sd | max | p |
|---|---|---|---|---|---|
| RULE-A | 0.9778 (44/45) | 0.8333 | 0.8334 ± 0.0458 | 1.0000 | 6.95 × 10⁻⁴ |
| RULE-B | 0.9778 (44/45) | 0.9058 | 0.9058 ± 0.0358 | 1.0000 | 3.63 × 10⁻² |

**Stated honestly: there is a real, small, statistically detectable enrichment of
the survivor set over the C1∧C2∧C3 pool.** It is +14.4 pp under the published
label and +7.2 pp under the defensible one, and a random 45-subset reaches it
about 1 time in 28 under RULE-B. That enrichment is produced entirely by gate C4 —
and §3 shows what C4 is.

Recall, the other half of the ledger and absent from the results file: the funnel
finds **44 of 406** RULE-A gauge subgroups (recall 0.108) and 44 of 402 under
RULE-B. It discards 89% of the very objects it claims to be selecting for,
including every generalized quaternion group Q₂ᵏ — canonical binary-dihedral
SU(2) subgroups, killed by C3 for being ambivalent.

---

## 2. The circularity, stated exactly

Two measurements, both on the pool C4 acts on.

**The label is a function of the construction tag.** Cross-tabulating
`is_lie_type` against `family` inside P123 (n = 138):

| family | SM-labelled | not | label constant within family? |
|---|---|---|---|
| Alternating | 1 | 0 | yes |
| BinaryPolyhedral | 1 | 0 | yes |
| Delta3n2 | 4 | 0 | yes |
| Dicyclic | 15 | 0 | yes |
| DirectProduct | 93 | 0 | yes |
| Frobenius | 0 | 14 | yes |
| Heisenberg | 0 | 1 | yes |
| LieType | 1 | 0 | yes |
| ModularGroup | 0 | 4 | yes |
| Semidihedral | 0 | 4 | yes |

**0 of 10 families carry a mixed label.** A classifier that reads nothing but the
family string reproduces 138/138 = 100% of the ground truth.

**C4 is also a function of that tag.** The predicate

```
family ∈ {BinaryPolyhedral, Dicyclic}
  ∨ name ∈ {2T, 2O, 2I, SL(2,3), SL(2,5), GL(2,3)}
  ∨ hard-coded Schur lookup keyed on family is nontrivial
  ∨ (|Z(G)| ≥ 2 ∧ |G|/2 ∈ {12, 24, 60})
```

reproduces `gate_c4` on **430 of 430** groups — exactly, no exceptions.

The classifier and the ground truth are therefore two readings of one variable.
The measured precision is a property of that variable, not evidence about finite
groups or about the Standard Model. This is the same failure mode already banked in
this programme's record as *nuisance annihilates the rival*: when the label's
defining variable is inside the classifier, the comparison is not a test.

---

## 3. Criteria provenance, scored one by one

| gate | as implemented | banked ancestry | verdict |
|---|---|---|---|
| **C1** commutator defect | `len(G') > 1 and 1 − k(G)/\|G\| > 0`. Both conjuncts are equivalent to "G is nonabelian". The continuous "defect" is reported in every table and used nowhere. | Yes — the stasis argument (abelian ⇒ frozen dynamics), SELECTOR-1/3. | **Sound but decorative.** It is the predicate "nonabelian" wearing a number. |
| **C2** order-spectrum entropy | Docstring says `H_ord ≥ 1.0`. **Code says `> 0.0`.** It excludes exactly one group in 430 — the trivial group. | **None found.** Repo-wide grep for `order_entropy` outside `conformance/selector/` returns nothing. | **NULL GATE with a free, undeclared threshold.** At the documented threshold of 1.0 it would kill Δ(27) (H = 0.2285) and UT(3,5) (H = 0.0672), and Δ(108) (H = 1.0091) would clear it by 0.9%. The implemented threshold is the one that keeps the SU(3) subgroups carrying the headline. A threshold documented at one value, implemented at another, and load-bearing at the difference is a fitted parameter. |
| **C3** orientation index | fraction of non-ambivalent classes > 0. | **Yes, genuine** — FROB-ORIENT-1, `lean/CIRISHolon/FrobOrient.lean`, BRIDGE-1, misfit M-GAUGE-LAUNDER. | **Sound.** But it is the classical fact that dihedral and symmetric groups are ambivalent, so §1's "Ambivalence Laundering Extinction … eliminates 100% of these groups" is a theorem restated, not a finding. |
| **C4** Schur multiplier / spin cover | a lookup table (see below). | **None found.** Repo-wide grep for `schur` outside `conformance/selector/` returns nothing. | **REFUTED.** It is the gate that does all the work (138 → 45) and the only one with no derivation. |

### 3.1 What C4 actually is

`compute_schur_multiplier` branches on `G.family` and returns a hard-coded value
per family; for any family not on its list — including every `DirectProduct` — it
returns the empty tuple. `check_spin_cover` returns True if the family string is
`"BinaryPolyhedral"` or `"Dicyclic"`, or the group's *name* is in a six-element
list, or `|Z(G)| ≥ 2 and |G|//2 ∈ (12, 24, 60)`.

Tracing every survivor (`refute_c4_provenance.py`):

| why C4 fired | survivors |
|---|---|
| family-string override (`"Dicyclic"`, `"BinaryPolyhedral"`) | 16 |
| magic-order rule (`\|Z\| ≥ 2` and `\|G\|/2 ∈ {12,24,60}`) | 25 |
| hard-coded Schur value keyed on family | 5 |
| group-name override | 2 |
| **a property computed from the group** | **0** |

Three consequences:

1. **The 15 dicyclic survivors — one third of the whole result — pass the
   "projective obstruction" gate while having no projective obstruction.** The
   file's own `compute_schur_multiplier` returns `()` for every one of them, which
   is mathematically correct: M(Dic_n) is trivial. They pass because the string
   `"Dicyclic"` is in a list, and they are labelled gauge subgroups because the same
   string is in another list.

2. **The magic-order rule tests the order of a quotient that does not exist.** The
   code computes `q_order = |G| // 2` and comments "quotient could be A4, S4, A5",
   assuming `|Z(G)| = 2`. For **21 of the 25** survivors it fires on, `|Z(G)| ≠ 2`,
   so `|G|/2` is not `|G/Z(G)|`:

   | survivor | \|G\| | \|Z(G)\| | `\|G\|/2` (tested) | `\|G/Z(G)\|` (real) |
   |---|---|---|---|---|
   | Z₆×Q₈ | 48 | 12 | 24 | **4** |
   | Z₆×D₈ | 48 | 12 | 24 | **4** |
   | Z₃×D₈ | 24 | 6 | 12 | **4** |
   | Z₃×Q₈ | 24 | 6 | 12 | **4** |
   | Z₁₀×D₁₂ | 120 | 20 | 60 | **6** |
   | Z₁₀×Dic₃ | 120 | 20 | 60 | **6** |
   | Z₄×D₁₂ | 48 | 8 | 24 | **6** |
   | Z₅×2T | 120 | 10 | 60 | **12** |
   | Z₁₀×A₄ | 120 | 10 | 60 | **12** |
   | … | | | | (21 in total) |

   No spin cover of A₄, S₄ or A₅ has a central quotient of order 4.

3. **C4 is undefined off the family list.** For any group not built by one of the
   eleven builders, `compute_schur_multiplier` returns `()`, so C4 degenerates to
   the magic-order rule alone. The sweep therefore cannot be extended to the actual
   landscape without rewriting the gate.

---

## 4. The gauntlet is not an isomorphism invariant

`structural_signature()` — the deduplication key — includes `schur_multiplier`,
which is computed by branching on the construction tag. So two isomorphic groups
built under different tags get different keys, survive deduplication as separate
entries, and can receive **different verdicts**.

`refute_dedup.py` finds three tagless-signature collisions in the 430 and confirms
all three by explicit generator-image isomorphism search with full multiplication-
table verification:

| \|G\| | entries | families | isomorphic? | Schur values assigned | C4 | full gauntlet |
|---|---|---|---|---|---|---|
| 12 | A₄, Δ(12) | Alternating, Delta3n2 | **yes** | (2,) vs () | True vs False | **PASS vs FAIL** |
| 12 | D₁₂, Z₂×D₆ | Dihedral, DirectProduct | **yes** | (2,) vs () | True vs False | fail, fail |
| 20 | D₂₀, Z₂×D₁₀ | Dihedral, DirectProduct | **yes** | (2,) vs () | True vs False | fail, fail |

**A₄ and Δ(12) are the same group.** One of them is in the survivor table as
"$A_4$ — YES (Gauge) — Tetrahedral subgroup in $SO(3)$". The other is killed by C4.
A criterion whose verdict depends on which name you built the group under is not a
criterion on groups.

It also follows that the census contains **427 isomorphism types, not 430**.

---

## 5. What population was actually swept

The results file says "the entire finite group landscape $|G| \le 128$" and
"430 non-isomorphic finite groups enumerated across all standard families".

The number of isomorphism classes of groups of order ≤ 128 is dominated by the
2-groups. From the standard classification (Besche–Eick–O'Brien; these are the
published counts, not computed here): **2,328** groups of order 128, **267** of
order 64, **231** of order 96, **51** of order 32, **14** of order 16. Those five
orders alone give 2,891; adding the trivial ≥ 1 for each of the other 123 orders,
the landscape contains **at least 3,014** isomorphism types. The conclusion does
not depend on the exact total: the sweep's 427 is **under 15%** of even this lower
bound, and the shortfall is concentrated exactly where the sweep is thinnest:

| \|G\| | true count | entries in the sweep | coverage |
|---|---|---|---|
| 16 | 14 | 11 | 79% |
| 32 | 51 | 15 | 29% |
| 64 | 267 | 18 | 6.7% |
| 96 | 231 | 25 | 10.8% |
| 128 | 2,328 | 15 | **0.64%** |

The 2-groups of order ≤ 128 number 2,669; the sweep contains 68 entries of
2-power order, 2.5% of them. **The population is a choice, and it was not
declared.** It is also the choice that matters, because 2-groups are where
nilpotent, non-ambivalent, nontrivial-multiplier groups that nobody would call
Standard Model subgroups live.

Two named absences and the criterion that killed them, checked:

- **Generalized quaternion Q₂ᵏ = Dic₂ᵏ⁻²** (k = 4,5,6,7). All pass C1, C2 and C4;
  all are killed by **C3**, orientation index exactly 0. Mathematically legitimate —
  generalized quaternion groups are ambivalent — but they are the even-index half of
  the McKay binary-dihedral series, i.e. genuine SU(2) subgroups. C3 discards them
  while the odd-index half survives and is counted as evidence for gauge structure.
- **Extraspecial 2-groups.** The census contains them at order 8 only (D₈ and Q₈,
  both killed by C3) and nowhere above. The extraspecial groups of order 32 and 128
  are absent from the population entirely.
- **Extraspecial groups do survive, and are counted as gauge subgroups.** The entry
  billed as "$SU(3)$ finite subgroup $\Delta(27)$" carries the alias `UT(3,3)` in
  the run's own output: Δ(27) **is** the extraspecial group of order 27 and exponent
  3, the qutrit Weyl–Heisenberg group. So is UT(3,5), the one survivor the results
  file scores as a false positive. The two survivors that are not dicyclic,
  dihedral-product or alternating are both members of the Pauli tower.

### 5.1 What happens when the omitted families are added

The census's own eleven families include five that are special cases of one
construction: cyclic, dihedral, Frobenius, semidihedral and modular groups are all
split metacyclic groups Z_m ⋊ Z_k. So the least arbitrary way to widen the
population is to add *all* of them — every Z_m ⋊_t Z_k with mk ≤ 128 — together
with the (Z_p)^d ⋊ Z_k affine groups and the extraspecial 2-groups of order 32 and
128. That is a principled extension, not a cherry-pick: it strictly generalises
five of the eleven families already in the census.

`refute_population.py` + `refute_extended.py` do this. Result: **261 genuinely new
isomorphism types**, a merged population of **691**, and:

| | census (430) | merged (691) |
|---|---|---|
| survivors | 45 | **68** |
| RULE-B precision on survivors | 44/45 = **0.9778** | 63/68 = **0.9265** |
| non-gauge survivors | 1 | **5** |
| RULE-B base rate in the population | 0.9349 | 0.8741 |
| RULE-B base rate in the *added* groups alone | — | **0.7739** |

Four new false positives appear immediately — Z₁₀⋊₃Z₁₂, Z₁₅⋊₂Z₈, Z₃₀⋊₁₇Z₄,
Z₅⋊₂Z₂₄ — quintupling the false-positive count. The base rate among the added
groups is 0.774 against the curated census's 0.935, so **the population choice
inflates the headline's own denominator by 16 points.**

And the mechanism is visible in one line. Every one of the 23 new survivors has

> **|G| ∈ {24, 48, 120} — 14 at order 120, 8 at order 48, 1 at order 24, and
> nothing else.**

That is forced: `compute_schur_multiplier` returns the empty tuple for any family
not on its hard-coded list, so for every group in the extension C4 collapses to the
magic-order rule alone. Applied outside the eleven families it was written around,
the "projective obstruction" gate is a test on the value of |G|.

*(This extension does not settle the 2-group question. It adds 261 types, mostly
metacyclic; the 2,595 groups of order 64 and 128 remain untouched. That is what the
forward test in §9 is for.)*

---

## 6. Forking paths: two alternative funnels

If a four-criterion funnel that lands on a famous family is evidence, then the
question is how cheap such funnels are. Two were built, each criterion no more
complex than C1–C4, with no threshold tuning and no criterion adjusted after seeing
a survivor list. Same 430 groups.

### Funnel X — the Pauli / Weyl–Heisenberg tower

> X1 nonabelian · X2 nilpotent · X3 |Z(G)| is prime · X4 exponent(G) ≤ |Z(G)|²

Target: *G is extraspecial* (the discrete phase-space / stabilizer groups of
quantum information — the Pauli group at every prime).

**Survivors: 4. On target: 4. Precision 1.0000.** Base rate of the target in the
post-X1 pool: 4/256 = 0.0156. **Enrichment +98.4 percentage points.**
Survivors: D₈, Q₈, Δ(27) = UT(3,3), UT(3,5) — the single-qubit Pauli group and its
qutrit and 5-level analogues.

### Funnel Y — the crystallographic point groups

> Y1 nonabelian · Y2 every element order lies in {1,2,3,4,6} · Y3 G has a faithful
> representation of degree ≤ 3 that is realisable over ℝ (Frobenius–Schur +1)

Target: *G is isomorphic to one of the 32 crystallographic point groups* — the
symmetry classification underlying all of solid-state physics.

**Survivors: 11 entries — 9 isomorphism classes, since the census's duplicates
A₄ ≅ Δ(12) and D₁₂ ≅ Z₂×D₆ are both in the list. Precision 9/9 = 1.000 and recall
9/9 = 1.000, verified by explicit isomorphism search.** The nine classes are
S₃ (= D₆), D₈, D₁₂, A₄, S₄, Z₂×D₈, Z₂×D₁₂, Z₂×A₄, Z₂×S₄ — **exactly the nine
nonabelian abstract types among the 32 crystallographic point groups, all of them,
and nothing else.** Base rate in the post-Y1 pool: 10/256 = 0.0391.

### Comparison

| funnel | criteria | survivors | precision | recall | base rate of its target (P0 / post-first-gate) | enrichment vs P0 |
|---|---|---|---|---|---|---|
| **X** — Pauli tower | 4 | 4 | **1.000** | **1.000** (4/4) | 0.0093 / 0.0156 | **+99.1 pp** |
| **Y** — point groups | 3 | 9 classes | **1.000** | **1.000** (9/9) | 0.0233 / 0.0391 | **+97.7 pp** |
| **C** — SELECTOR-5 | 4 | 45 | 0.9778 | **0.108** (44/406) | 0.9442 / 0.9062 | **+3.4 pp** |

Two funnels built in an afternoon, one of them with three criteria rather than
four, achieve **perfect precision and perfect recall** on two famous families of
physics — the discrete phase-space groups of quantum information and the point
groups of crystallography — where SELECTOR-5 achieves 97.8% precision at 10.8%
recall against a target that is already 94% of its population.

Pretty funnels are cheap. One pretty funnel is not evidence. Overlaps with
SELECTOR-5's survivors: X ∩ C = {Δ(27), UT(3,5)} — *both* of SELECTOR-5's
non-dicyclic, non-product survivors are members of the Pauli tower, and one of them
is the flagship "SU(3) finite subgroup Δ(27)"; Y ∩ C = {A₄, Z₂×A₄}.

A funnel that lands on a famous family is not a discovery when the famous family
is 94% of the population and the criteria were chosen after the population was in
hand.

---

## 7. The published tables are not reproducible from the published instrument

`SELECTOR5_RESULTS.md` §2, §3 and §4 disagree with the committed script's own
printed output in **every** cell. Both were produced by running
`python3 conformance/selector/landscape_sweep.py`, the command the results file
gives for reproduction.

| gate | RESULTS.md pass | script pass | RESULTS.md TP | script TP | RESULTS.md precision | script precision |
|---|---|---|---|---|---|---|
| C1 | 301 | **256** | 119 | **232** | 0.3953 | **0.9062** |
| C2 | 429 | 429 | 186 | **405** | 0.4336 | **0.9441** |
| C3 | 277 | **304** | 158 | **281** | 0.5704 | **0.9243** |
| C4 | 77 | **148** | 63 | **146** | 0.8182 | **0.9865** |
| C1+C2 | 301 | **256** | 119 | **232** | 0.3953 | **0.9062** |
| C1+C2+C3 | 149 | **138** | 91 | **115** | 0.6107 | **0.8333** |
| Full | 45 | 45 | 44 | 44 | 0.9778 | 0.9778 |
| Full F1 | 0.3793 | **0.1951** | | | | |

Scale bins (§3), group counts: RESULTS.md 52 / 58 / 129 / 191; script
**41 / 62 / 125 / 202**.

Family census (§4): RESULTS.md gives Frobenius 22, Heisenberg 3, DirectProduct 159,
Symmetric 4, Alternating 3, Delta 5; the script gives Frobenius **14**, Heisenberg
**1**, DirectProduct **173**, Symmetric **2**, Alternating **2**, Delta **4**. Both
sum to 430.

The one row that agrees is the last one — the survivor count, the survivor names
and the 97.8%. The published gate table implies a base rate of 186/430 = 43.3%,
against which the funnel would look powerfully selective. The instrument's real
base rate is 94.4%. **The tables that make the result look like a discovery are
the tables that do not come from the instrument.**

### 7.1 The test suite is green through all of it

`python3 -m unittest test_landscape_sweep` gives **15/15 OK** in 12 s, as the
results file says. It is green because of what it does not test:

- Every `gate_c2` assertion is `assertTrue` on a group with H_ord well above 1.
  No test exercises a group with 0 < H_ord < 1, so the gap between the documented
  threshold (≥ 1.0) and the implemented one (> 0) is invisible to it — and that gap
  is where Δ(27) and UT(3,5) live.
- The Schur assertions (`assertEqual(g.schur_multiplier, (2,))`,
  `(3,3)`) test the hard-coded lookup table against the hard-coded lookup table.
- No test asserts that any gate is an isomorphism invariant. That is why the
  A₄ / Δ(12) split verdict of §4 survives a green suite.

The missing mutation is one line: assert that two isomorphic groups built by
different builders receive the same verdict. It fails today.

---

## 8. What survives

One observation, correctly stated and worth nothing more than its size:

> Among the 138 groups in this hand-built census that are nonabelian and have a
> non-real conjugacy class, a hard-coded family list selects 45, of which 44 have a
> faithful representation small enough to sit inside SU(3) × SU(2) × U(1) — against
> 125 of 138 (90.6%) in the pool it selected from.

That is a **SURVIVES-AS-POST-HOC-OBSERVATION** at +7.2 percentage points, p = 0.036,
produced by a gate that reads a construction tag. It is not support for any claim
about selection, gauge structure, or the Standard Model.

---

## 9. The forward test that could make this rule-6 support

The programme's rule 6 — *a residual is never support; support comes only from
confirmed advance predictions* — has an available instrument here. Specification,
in the order it must happen:

**Step 1 — repair the criterion so it is a criterion.** C4 must be replaced by a
function of the isomorphism class. Two acceptable forms: (a) M(G) computed by an
actual algorithm (Hopf formula from a presentation, or a computer-algebra
Schur-multiplier routine), or (b) the criterion restated as something already
computable here, e.g. "G has an irreducible projective representation that is not
projectively equivalent to a linear one". Either way the gate must be verified to
agree on all three isomorphic-duplicate pairs of §4, which is a mutation test the
current gate fails.

**Step 2 — declare C2's threshold.** Either `> 0` or `≥ 1.0`, chosen and written
down before Step 4, with the acknowledgement that at `≥ 1.0` the SU(3) subgroups
Δ(27) and UT(3,5) leave the survivor set.

**Step 3 — freeze the label.** RULE-B as stated in `FROZEN_LABEL_RULE.md`, already
implemented and validated in `refute_lib.py`. RULE-A cannot be used: it is a
construction tag, and it is demonstrably wrong in both directions. It calls
(Z₂)⁵, (Z₂)⁶ and (Z₂)⁷ gauge subgroups, which the rank obstruction forbids; it
calls fourteen products of two nonabelian factors gauge subgroups (D₆×D₆, D₆×D₈,
D₈×D₈, D₆×A₄, D₈×A₄, D₁₀×A₄, …), which fail because SU(2) contains no dihedral,
symmetric or alternating group — by McKay its finite subgroups are only cyclic,
dicyclic and binary polyhedral — so the second nonabelian factor has nowhere
faithful to go. And it calls F₂₁, F₃₉, F₅₇, F₉₃, F₁₁₁ and every semidihedral and
modular group *not* gauge subgroups, when all thirteen do embed. Seventeen errors
one way, thirteen the other, out of 430.

**Step 4 — stake the prediction on a held-out population, in writing, before
computing it.** The right held-out population is **all 267 groups of order 64 and
all 231 groups of order 96** — 498 groups, none of them in the current census
beyond 43 entries, and a population where RULE-B's base rate is *low* rather than
94%, so precision has room to be informative. Stake, in advance: the number of
survivors, the precision band, and the direction of the difference from the
held-out base rate. Confirmation of a pre-stated band is rule-6 support.
Anything computed first and interpreted after is not.

**Why this test has teeth.** In the current census the base rate is 94%, so
precision cannot discriminate. Among 2-groups of order 64 and 128, most groups have
no faithful representation of degree ≤ 3 and therefore fail RULE-B, so the base
rate collapses and a funnel that still reads 97% would be saying something. That is
the measurement SELECTOR-5 has not made, and the reason its population choice is
load-bearing rather than incidental.

---

## 10. Reproduction

```bash
cd conformance/selector
python3 landscape_sweep.py            # their 45, 7 s
python3 refute_baserate.py            # base rates + permutation control, ~90 s
python3 refute_c4.py                  # C4 dissection
python3 refute_c4_provenance.py       # why C4 fired, per survivor
python3 refute_dedup.py               # isomorphic duplicates, split verdicts
python3 refute_funnels.py             # the two alternative funnels, ~13 s
python3 refute_extended.py            # extended population, ~20 min on a loaded box
```

`refute_extended.py` writes `refute_extended.json`; the run reported in §5.1 was
made by an earlier revision of the same script whose serialised output is
`refute_extended.pkl` (names, RULE-B labels, survivor set) — every §5.1 number is
recomputable from that file plus `landscape_sweep.generate_landscape()`.
