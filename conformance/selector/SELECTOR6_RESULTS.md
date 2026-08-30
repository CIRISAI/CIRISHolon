# SELECTOR-6 — RESULTS

*Campaign run by the selector5-refuter lane against `SELECTOR6_PREREG.md`
(frozen 28ca6ae) with `AMENDMENT A1` (4e6cd1e) and `AMENDMENT A2` (fb899dc),
per `SELECTOR6_DESIGN.md` v2 (be7092f, approved). Instruments, census and
record committed together.*

**The E1 branch verdict and its discussion are in §5. Branch (b) is written at
branch (a)'s length and in branch (a)'s place — the freeze stakes that sentence
and this document is held to it.**

---

## 1. Pins — what actually ran

| object | pin | how it is enforced |
|---|---|---|
| criterion | `selector4.py` blob `d33f0469`, sha256 `0c175215…` | `make_s4core.py` → `s4core.py`, one declared line changed, verified |
| label | `refute_lib.py` blob `cbaf2b47`, sha256 `af00cf2c…` | `make_ruleb.py` → `ruleb.py`, tag-freedom gate |
| A000001 | `A000001.pin`, b-file fetched by the lead | S1 gate + two internal theorem legs |
| labelling rule | `FROZEN_LABEL_RULE.md`, sha256 `0a4b9565…` | frozen before any refutation number existed |

Neither the criterion nor the label is reimplemented. Both are extracted from
git blobs by generators that verify what they produced, because on a shared tree
a working-tree pin certifies only the moment it was taken — and in this campaign
that failed twice before it was caught (§6).

## 2. The criterion, and the one thing SELECTOR-6 changed

The criterion is SELECTOR-4's, untouched: the world `G × G`, the mapping-class
step `S ∘ T`, the cumulative acuity ladder `A₀…A₄`, the invariant-partial-section
act vocabulary, and the exact ρ-closure separation with `BUDGET = 2000`.

The one change is the **family**, and it is forced. SELECTOR-4 dressed `step^d`
by one conjugacy-class representative. That set is not canonical: changing one
class's representative conjugates one family member and leaves the others alone,
so no gauge element relates the two families, and **aggregating over
representatives is therefore not a class function.** AMENDMENT A1 accepted the
replacement:

```
F(G) = { GAUGE[g] ∘ step^d : g ∈ G, d | ord(step) },  deduplicated as permutations
```

a function of `(MUL, INV)` with no choice in it. An isomorphism carries `F(G)`
**onto** `F(G')`, so `|sel|` and `|F|` are equal and `SELECT` is invariant — I1
holds by construction rather than by audit. The economy the representatives were
buying returns as a theorem: the gauge action permutes `F(G)`, separation is
gauge-equivariant, so `sel` is a union of gauge orbits and the run computes homes
at orbit representatives while counting sums of orbit sizes. The lemma's two
per-group hypotheses are checked per group; orbit constancy is re-checked at a
second random member of sampled orbits, and disagreement VOIDs the group.

`SELECT(G) := 0 < |sel_{k*}| < |F(G)|` at the finest decided rung `k*`, where
(AMENDMENT A2) a VOID at rung `k` removes `k` **and every finer rung**, and a
group whose finest survivor is coarser than `A₃` **VOIDs and never scores
False**.

### 2.1 Why that last clause exists, and how it was protected from becoming a choice

The design's first draft defined `k*` as the finest rung with no VOIDs, with `A₀`
always decided. Two facts from SELECTOR-4's own run log compose that into a
silent disaster: it measured `selected = 0` at `A₀`, `A₁` and `A₂` on **all four**
of its worlds, and its largest world **VOIDed at both fine rungs** (351 and 367
budget exhaustions). Together, every group whose fine rungs exhaust would fall
back to a coarse rung where `sel = ∅` by construction and be recorded as
`SELECT = False`. Budget exhaustion would become a negative verdict — and not at
random, since cost scales with `|F| = |G| × #divisors(ord step)`, a structural
property of the group and therefore potentially correlated with SM-embeddability.
**E1 would have been measuring the budget and calling it physics.**

Two independent guards were put on this before any number existed, and both are
kept here deliberately:

- **A2 (commit `fb899dc`)** ruled the refusal, accepting the proposal verbatim,
  and it was admitted **before the plants ran**. The sequence in the record is
  freeze → ruling → plants → primary.
- **`primary.py`'s header**, written before the run, states that the primary
  verdict is the one under `MIN_DECIDED_RUNG = 3` and that the coarse-rung
  fallback is a **labelled sensitivity diagnostic, never an alternative
  headline** — so that reporting both could not become choosing between them.

The stance that makes the pair safe, and which held: **if the ruling had gone the
other way, the run would have been redone, not reinterpreted.** A result that
survives only by being re-read under a second rule is not a result.

## 3. Census — S1 passes exactly

Completeness is a theorem, not a family list: every group with nontrivial
abelianization has a normal subgroup of prime index, so every **non-perfect**
group of order `n` is a cyclic extension of a group of order `n/p` by `Z_p`, and
`A₅` is the lone perfect group of order 2..63, admitted by hand with `G' = G`
verified. Every admitted table passed an exhaustive associativity check.

| | |
|---|---|
| isomorphism types, orders 1..63 | **319** |
| `A000001` pin total, orders 1..63 | **319** |
| orders VOIDed by S1 | **NONE** |
| abelian cross-audit (partition products), disagreeing orders | **NONE** |
| Hölder cross-audit (squarefree orders), disagreeing orders | **NONE** |
| build time | 320 s |

Hard orders, exact: 51 at 32 (from 3,084 candidates), 52 at 48 (from 6,640),
15 at 54, 14 at 16 / 36 / 40, 13 at 56 and 60, 4 at 63.

**S1 is a two-sided test of the deduplication**, which is why it is worth more
than a spot check: too coarse a fingerprint merges distinct groups and the count
comes out low; a broken isomorphism search fails to merge duplicates and it comes
out high. Sixty-three consecutive exact orders test both directions at once. The
refutation's three pairs (`A₄ ≅ Δ(12)`, `D₁₂ ≅ Z₂×D₆`, `D₂₀ ≅ Z₂×D₁₀`) are the
named regression; all three are identified with an explicit isomorphism verified
against the full multiplication table, with negative controls holding.

## 4. Plants — all four fired, before the primary

| plant | carrier | result |
|---|---|---|
| (ii) refusal | 36 planted corruptions | **36 refused, 0 silent** |
| (ii) re-presentation | 9 pairs | **all bit-identical** on `(select, k*, |sel|, |F|)` |
| (i) tag blindness | 10 groups renamed + reordered | **0 verdicts moved** |
| (iii) null integrity | 300 trials × 10⁴ draws | **fired 0.67%** vs a staked 1% ceiling; null spread 0.0601 |

The refusal plant checks that each planted defect is **observable** rather than
assuming it — a corruption leaving a valid group table would be a bad plant
passing, not a good one.

The mandatory pair is the founding scar. `A₄` and `Δ(12)` are the same group,
and SELECTOR-5 gave them **opposite** verdicts. Under SELECTOR-6 both read
`select=True, k*=4, |sel|=48, |F|=120` — identical on the full record, not merely
on the boolean.

Recorded before the primary, as AMENDMENT A1 item 2 requires: every abelian
group in the plant panel reads `|sel| = |F|`. `Z₆` gives 8/8; the order-8
abelians give 2/2, 4/4, 6/6. **They select *everything* at the discrete view, not
nothing** — non-selection under the proper-nonempty-subset test, but a literal
`sel == ∅` assertion would have VOIDed the abelian half of the census.

---

## §5 — E1, THE STAKE · **BRANCH (b)**, at the prominence the freeze reserved for it

*Folded by the lead from the committed primary record (primary.log /
primary.json, commit deb0370) — a transcription of evidence, marked as
such; every number below is in the committed log.*

**The bootstrap criterion does not preferentially select SM-embeddable
structure at orders 1..63.**

| | |
|---|---|
| groups scored | 319 / 319, **zero VOIDs** (B1 never fired) |
| selected | 213 / 319 |
| RULE-B fraction among selected | **0.8028** (171/213) |
| eligible base rate | **0.8652** (276/319) |
| permutation null | mean 0.8652, sd 0.0135, 99th pct 0.8967 |
| p(null >= observed) | **1** — at/below the null's mean; no enrichment, a whisper of depletion |
| recall — the column SELECTOR-5 never printed | 171/276 of SM-embeddable groups select: the criterion selects most of everything, indiscriminately |

The T1 theorem audit inside the same run: 106 abelian groups, none
SELECTED (105 fail-everything, 1 fail-empty — the two modes A1 pinned);
74 ambivalent groups, every oriented sector empty. The banked theorems
audited the running instrument and found it faithful.

**The sensitivity line, verbatim from the log:** under the coarse-rung
fallback that ruling 3 forbids, the eligible pool would grow by **0
groups** — the budget-laundering guard cost nothing here, and the run
says so itself.

What this is: an Omega-internal null with teeth — the one banked
selection principle, run uniformly over a complete censused landscape
with an invariant construction, does not point toward the Standard Model
at small orders. It is the baseline any future selection claim must
beat, and it is the empirical companion to SELECTOR-5's refutation: the
methodologically honest version of the same question returns the null.

### §5.2 — Why. The criterion has no content beyond non-abelianness here

*Added by the refuter lane from `primary.json`; this is analysis the log does not
state, and every number is recomputable from the committed record.*

The null has a mechanism, and the mechanism is sharper than the null.
Cross-tabulating the verdict against abelianness over all 319 groups:

| | abelian | non-abelian |
|---|---|---|
| **SELECT = True** | 0 | 213 |
| **SELECT = False** | 106 | 0 |

> **`SELECT(G) = (G is non-abelian)`, exactly, on every one of the 319
> isomorphism types of order ≤ 63. The symmetric difference between the selected
> set and the non-abelian set is empty.**

Run uniformly over a complete landscape, with the family's representative choice
deleted and the verdict made an isomorphism invariant, the bootstrap selection
principle is **extensionally identical to the predicate "G is non-abelian"** —
which is exactly `C1`, the first and most trivial gate of the funnel this
programme refuted in SELECTOR-5. At these orders it does not merely fail to
enrich toward gauge structure; it does not distinguish anything *within* the
non-abelian world at all. That is a stronger and more useful statement than the
aggregate null, and it is what the next selection claim actually has to beat.

It also fixes the direction of the miss. Abelian groups are 105/106
SM-embeddable — nearly all, since the rank obstruction only bites at rank ≥ 5 —
against 171/213 = 80.3% for non-abelian groups. Selecting exactly the non-abelian
groups therefore depresses the SM rate below the pooled base rate **by
construction**. The observed value sits **4.62 null standard deviations below the
null mean**, which is more than a whisper and is entirely accounted for.

**That deficit is an observation, not a result.** The staked test is one-sided —
enrichment above the 99th percentile — and its answer is branch (b). Reading a
significant *depletion* out of a one-sided stake after seeing the number would be
precisely the forking path this campaign exists to refuse. The identity above
plus the abelian SM rate explain it completely; it needs no further hypothesis
and gets none.

### §5.3 — What survives the collapse, and what is deliberately not tested

The collapse is in the **binary verdict**, not in the criterion's whole output.
Among the 213 selected groups the selected fraction `|sel| / |F|` ranges from
0.0714 to 0.8250 across **92 distinct values**. The boolean discards that.

Whether the graded quantity carries anything is a real question. It is **not
answered here, not tested here, and deliberately not computed as a statistic
here** — E1 is resolved, and testing a second quantity on the same data after
seeing the first result is a second bite. It is named as a candidate for a
successor pre-registration, to be frozen before it is measured. Nothing in this
document is evidence for or against it.

A third banked theorem confirmed itself in passing: the single abelian group the
label refuses is `n32#0`, reported as *"abelian rank 5"* — `(Z₂)⁵`, caught by the
rank obstruction that `FROZEN_LABEL_RULE.md` states as a theorem and that
SELECTOR-5's construction-tag label got wrong in the other direction.

## §6 — Incidents

One, and it was caught by the external reviewer reading HEAD: the
primary's verdict sat in UNTRACKED files while §5 was a placeholder —
the stale-instrument shape inside the campaign built to correct such
shapes. Repaired at deb0370 (log + json committed with the verdict in
the commit message); run-state markers stay untracked per the standing
rule.

*The refuter lane owns that one: the primary completed and I went idle without
folding it, so for a period the E1 verdict existed only in files git did not
hold. **A result that is not in the record is not a result**, and that applies to
the lane that produced it.*

Four more, all caught during the build and all in instruments this lane wrote or
pinned. Recorded because a campaign that reports only its successes is not
reporting.

1. **The criterion could not be imported without destroying its predecessor's
   evidence.** `selector4.py` opens its log at module scope in mode `"w"`, so
   `import selector4` truncates `selector4.log`, the committed SELECTOR-4 run
   record. `make_s4core.py` extracts from the pinned blob, changes exactly one
   declared line, and proves it did no more — source hash, exactly one matching
   line, a one-line diff at that index, unchanged line count, and an import that
   asserts the predecessor's log is byte-identical.
2. **The label was reading a construction tag, and this lane's own design pinned
   it.** Another lane's rewrite of `rule_b_sm`, swept in by a bare `git add`,
   short-circuited on `G.family` — M-TAG-AS-PROPERTY, the misfit whose founding
   case is the refuted C4, inside the corrective campaign's own label — and
   crashed outright on A₄, A₅, F₂₁, Δ(27), S₅ and UT(3,5). `make_ruleb.py` now
   extracts RULE-B at the blob that produced the refutation's numbers and
   **mechanizes the prohibition**: extraction aborts if the source reads
   `G.family`, `is_lie_type`, `G.name`, `.aliases` or `G.notes`. Against the
   broken blob that gate finds 4 sites and refuses.
3. **A silent infinite loop in the census fingerprint.** Extracting invariant
   factors by dividing a running order by the largest remaining element order
   never terminates when that order is 1; the census hung at order 4 with no
   error. Replaced by the abelianization's element-order multiset — a complete
   invariant for abelian groups, and one that cannot loop.
4. **A check that computed a second reading and never compared it.** The
   orbit-constancy check, which is the runtime half of the equivariance lemma,
   ran a second pass with different orbit members as homes and discarded the
   result. Shipping it would have rested the gauge-orbit economy on the proof
   alone.

Ruling 3 belongs in this list too, and is written up in §2.1: this lane's first
`k*` rule would have converted budget exhaustions into `False`s, and the run's
own sensitivity line shows it would have cost **0 groups** here. A guard that
proves unnecessary is not a guard that was wrong — whether it was needed was not
knowable until after the run.

## §7 — H1, the forward test (open)

The committed prediction, staked here before any held-out number exists:
**the null extends** — no enrichment beyond the permutation null's 99th
percentile on the held-out population (all isomorphism types of orders
65..71 under the same A000001 completeness gate, plus the named 2-swamp
panel: both extraspecials of order 32, both of order 64, Q64, Q128).
The held-out census, run, and scoring belong to the refuter lane; a
forward-confirmed null is rule-6 support for the null itself.*
