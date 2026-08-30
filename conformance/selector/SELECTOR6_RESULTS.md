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

*(§5 E1, §6 incidents, and §7 H1 are written when the primary lands.)*
