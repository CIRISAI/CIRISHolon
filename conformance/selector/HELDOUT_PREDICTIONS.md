# SELECTOR-6 · H1 — HELD-OUT PREDICTIONS

*Committed by the selector5-refuter lane BEFORE any held-out group is built,
scored, or counted. No held-out multiplication table exists at the time of this
commit. The only held-out quantity read so far is `A000001(65..71)` from the
pinned b-file, which is the external completeness target and not a measurement.*

The primary (E1 branch (b), commit `deb0370`) found more than an aggregate null:
on all 319 isomorphism types of order ≤ 63, **`SELECT(G) = (G is non-abelian)`
exactly**, symmetric difference empty. A post-hoc identity is worth little. Staked
forward it is worth something, and it is sharp enough to be killed by a single
group.

---

## 1. The held-out population, declared

**(a) Complete orders 65..71**, under the same S1 discipline as the primary: the
census count per order must equal the pinned `A000001` value or that order VOIDs
for every claim.

| order | 65 | 66 | 67 | 68 | 69 | 70 | 71 | total |
|---|---|---|---|---|---|---|---|---|
| `A000001` | 1 | 4 | 1 | 5 | 1 | 4 | 1 | **17** |

**(b) The named 2-swamp panel** — the groups the commission named as the ones
SELECTOR-5's population was protecting itself from.

**One item in the commission's list does not exist, and the correction is made
here rather than discovered mid-run.** Extraspecial 2-groups have order
`2^(1+2n)` — that is 8, 32, and 128. `64 = 2^6` and `1 + 2n = 6` has no integer
solution, so **there is no extraspecial group of order 64.** The slot is filled at
the next order where the objects do exist, keeping the panel at six:

| # | group | order | why it is in the panel |
|---|---|---|---|
| 1 | `ES32+` = D₈∘D₈ | 32 | extraspecial, plus type |
| 2 | `ES32−` = D₈∘Q₈ | 32 | extraspecial, minus type |
| 3 | `Q64` = Dic₁₆ | 64 | generalized quaternion; C3 killed the whole series in SELECTOR-5 |
| 4 | `Q128` = Dic₃₂ | 128 | generalized quaternion, the largest world here |
| 5 | `ES128+` = D₈∘D₈∘D₈ | 128 | extraspecial, plus type — **substitute for the non-existent order-64 pair** |
| 6 | `ES128−` = D₈∘D₈∘Q₈ | 128 | extraspecial, minus type — **substitute** |

Each panel member is verified to be what it is named: `|Z(G)| = 2`, `G' = Z(G)`,
`G/Z(G)` elementary abelian for the extraspecials, with the two types
distinguished by involution count; a unique involution for the generalized
quaternions. A member failing its own definition VOIDs and is reported as VOID.

---

## 2. The predictions

Every one of these is falsifiable by the held-out run, and each is scored
independently.

**P1 — THE SHARP ONE, per group.** For every held-out group,
`SELECT(G) = True` if and only if `G` is non-abelian. **Zero exceptions.**
Concretely: of the 17 groups at orders 65..71, the abelian ones select nothing
(in the `|sel| = |F|` mode, except a group with `|F| = 1`) and every non-abelian
one is selected; all six panel members are non-abelian and are therefore all
predicted `SELECT = True`.
*Killed by:* a single held-out group whose verdict disagrees with its
abelianness.

**P2 — the aggregate null extends.** The RULE-B fraction among the selected
held-out groups does **not** exceed the permutation null's 99th percentile on the
held-out eligible pool. E1's branch (b) holds on a population this census never
touched.
*Killed by:* observed fraction above the 99th percentile.

**P3 — the direction of the miss.** Because abelian groups are almost all
SM-embeddable and P1 makes the non-abelian set exactly the selected set, the
held-out RULE-B fraction among selected will be **at or below** the held-out base
rate.
*Killed by:* selected fraction above the base rate.

**P4 — the panel is not SM-embeddable.** All four extraspecial members (`ES32±`,
`ES128±`) are predicted `RULE-B = False`: an extraspecial group of order `2^(1+2n)`
has a faithful irreducible representation of degree `2^n` and every non-faithful
irreducible kills the centre, so no faithful representation of degree ≤ 3 exists
for `n ≥ 2`. `Q64` and `Q128` are predicted `RULE-B = True` (generalized
quaternion groups have a faithful 2-dimensional representation, hence embed in
U(2) ⊂ SU(3)).
*Killed by:* any panel label disagreeing.

**P5 — budgets.** Orders 65..71 complete with **zero VOIDs**. The three
order-128 panel members (`Q128`, `ES128±`) have `N = |G|² = 16384`, larger than
the `N = 14400` world where SELECTOR-4 recorded 351 and 367 budget exhaustions,
so **a VOID on any of those three is expected and is not a failure of P1–P4** —
it is B1 working. `Q64` (`N = 4096`) is predicted to complete.
*Not killed by* a VOID on an order-128 panel member; VOIDs are reported by group,
with `|F|`, and excluded from P2's pool.

---

## 3. Scoring rule, fixed now

- P1 is scored over every held-out group with a decided verdict. It is
  **all-or-nothing**: one disagreement kills it. The count of disagreements is
  reported whatever it is.
- P2 uses the same instrument as the primary: 10⁵ draws, **seed 20260830**,
  one-sided at the 99th percentile, on the held-out eligible pool alone (not
  pooled with the primary — pooling would let the primary's 319 drown the
  held-out 17 + 6).
- P3 and P4 are reported as stated; P4 is scored per panel member.
- Any order failing S1 VOIDs and is excluded from every prediction, named.
- **No prediction is revised after any held-out number is seen.** If P1 fails,
  the failure is the finding and it is reported at this document's prominence.

## 4. What a confirmation would and would not mean

A confirmed P1 makes "the bootstrap criterion, run invariantly over a complete
landscape, is extensionally non-abelianness" a **forward-predicted** statement
rather than a description of one dataset — rule-6 support for the null itself,
which the programme's discipline treats as scarcer than support for a positive.

It would **not** mean the criterion is trivial at all orders, that the graded
quantity `|sel|/|F|` is empty (that is untested and deliberately so), or anything
about the orders this census does not cover. The held-out population is 17 groups
plus a 6-group panel; it is a test of a sharp prediction, not a second landscape.
