# QVM-RANK-1 — READ: no six-term witness for seven copies of the H state; G2 met by a route the prereg did not name; the coverage is a partial exclusion — no rank-6 decomposition of |H⟩^⊗7 has three or fewer terms spanning a vector of span(|H⟩^⊗7, |H^⊥⟩^⊗7), and the two known classes of rank-6 decompositions of |H⟩^⊗6 do not extend — branch (b)

*2026-09-26. Prereg `conformance/rank/QVM_RANK1_PREREG.md` (frozen alone, `c5e250a`; nothing
in it moved; its notes on building, items 1–10, are appended and were committed before the
seven-copy search was read, except item 10's single smoke read, which is recorded there).
The field: `conformance/rank/STABRANK_STATE.md` (stabrank at `2ceb676`). Instrument:
`holon::stabrank` (`engine/crates/holon/src/stabrank.rs`), driver
`engine/crates/holon/src/bin/qvm_rank1.rs`, plants `engine/crates/holon/tests/qvm_rank1.rs`
(13 green), device twin `engine/crates/holon-gpu/tests/rank1_twin.rs` (2 green). Placement:
`taskset -c 21-27` for every CPU process (checked on the running PID), the RTX 4090 Laptop GPU
for G4/PR-5; cores 0–19 untouched. Referee: stabrank's `verify_challenge/stabrank_verify.py`,
run from the clone, on every witness written.*

## 0. Verdict

| stake | staked | read | verdict |
|---|---|---|---|
| **G1** a six-term witness of `|H⟩^⊗7`, accepted by stabrank's verifier | the shot | **none found** in 30.4 h of wall on seven cores (§3) | **NOT MET** (no kill; branch (b)) |
| **G2** the pipeline, told nothing, recovers rank 4 at m = 4 and rank 6 at m = 6 within an hour each, each accepted by the referee | kill: either not recovered in an hour | m = 4: annealing **0.1 s**, and the exhaustive pivot completion **97.6 s** (every one of the 1,921,914 canonical pivot pairs; **23** classes, PR 88's number; stabrank's stored list of 30 falls into the same 23). m = 6: the annealer alone failed (10 min), the slice-lift tower failed (13.6 min), the V-split found it in **330 s** — **≈ 29 min cumulative**. Referee: `verified`, symbolic, on all three files | **MET** — by the pipeline as built, not by the annealer the prereg names (notes item 6) |
| **G3** exact coverage if no witness | orbit count, fraction decided, filters and their removals, annealing budget; a partial exclusion certificate | §3: the split exclusion (a theorem modulo two float-harvest margins, stated), the lift branch (every known class of rank-6 decompositions of `|H⟩^⊗6` decided: 0 extensions), the five-tuple fraction (≈ 0, by arithmetic), the move budgets | **MET** as a partial exclusion; its tier in stabrank's terms is `reproduced`/`attested`, not `verified` (§3.4) |
| **G4** every residual batch on the GPU equals the CPU twin's struct | struct equality | **64 batches, 4,260 folds, struct-equal on all**, value `= den·v(y)` on all | **MET** |

Plants: **PR-1** fires on all 33 known witnesses (a `2⁻⁵⁰` ring unit in one coefficient: the
float residual stays below `10⁻¹²`, the `Cyc` check rejects). **PR-2** fires (all 1,080 three-
qubit states accepted by the exact `Cyc` test under a nontrivial common scalar `1 + ω`;
`|H⟩^⊗3` rejected; 1,000 non-stabilizer rejects: 266 eighth-root phases, 258 moduli, 207
support holes, 269 random ring vectors; plants that happened to land on a genuine ray were
decided independently and skipped). **PR-3** fires (reduced = Burnside at m = 3 for k = 1, 2, 3:
48, 14,142, 4,469,574; = brute at k = 1, 2; at m = 4: 246 = 246 = 246 singles — stabrank's
"246 pivots" — and 1,921,914 = 1,921,914 pairs). **PR-4** fires with each filter under its own
premise (notes item 4): every term of all 36 known decompositions passes the per-term slice
filter at its `(m, r)`, every tuple passes the tuple slice filter, every `(r−1)`-subset passes
the Galois subset filter; the QPG witness VIOLATES property P (`k = 1` term), as it may at
`(6, 6)`, and the test asserts that too. **PR-5** fires (a limb flipped on the card is convicted
at its position — branch 2 limb 0 at 2, branch 4 limb 5 at 34, branch 0 limb 3 at 18 — and the
fold it corrupts disagrees with the twin).

## 1. The instrument

`holon::stabrank`: stabilizer rays in canonical flat-and-phase form (support an affine subspace of
`F₂ⁿ`, a quadratic phase, a ring scalar) with a dictionary dedup; an EXACT stabilizer-ness test of a
ring vector (PR-2: every one of the 1,080 three-qubit states accepted under a nontrivial common
scalar, `|H⟩^⊗3` and 1,000 planted non-stabilizer vectors rejected); exact pivot completion over
`ℚ(ζ₈, cos π/8)` — the prereg's ring `ℤ[ζ₈, 1/2]` cannot hold odd-copy coefficients (notes item 1) —
which REPLACES the prereg's "project and test the residual", incomplete as written (item 2);
symmetry reduction under `S_m ⋊ (H^{⊗k})` with orbit counts equal to Burnside's and to brute force
at `m = 3, 4` (PR-3: 48 / 14,142 / 4,469,574; 246 pivots at `m = 4` — stabrank's own number);
stabrank's dual-distance property (PR 87) and Galois constraint (PR 16) as filters, each keeping
every term of every known witness under its own premise (PR-4; the QPG witness violates the
dual-distance property at `(6, 6)`, as it may, item 4); annealing with Pauli, Clifford and
single-state moves under exact acceptance; the V-split (triples whose span meets
`span(|H⟩^⊗m, |H^⊥⟩^⊗m)`, joined in pairs — from the Galois note, and the route that met G2 at
`m = 6` where the annealer did not, item 6); the GPU twin (`holon-gpu`, `rank1_twin`: 64 batches,
4,260 folds struct-equal, PR-5 a flipped lane convicted by position). Every accepted witness was
re-verified by stabrank's `verify_challenge/stabrank_verify.py` at tier `verified`, symbolically.
Float residuals prune; the ring accepts (PR-1: a `2⁻⁵⁰` ring unit rejected by `Cyc`, invisible to
a `10⁻¹²` float check, on all 33 known witnesses).

## 2. G2 — the controls

**MET.** `m = 4`, rank 4: annealing in **0.1 s**; the exhaustive pivot completion over all
1,921,914 canonical pivot pairs in 97.6 s found the **23 classes** stabrank's PR 88 reports, and
their stored 30 witnesses fall into the same 23. `m = 6`, rank 6: the annealer the prereg named
MISSED it in 10 min, the slice-lift tower in 13.6 min; the V-split found it in **330 s** —
cumulative 29 min, inside the hour. It found a SECOND class of rank-6 decompositions of `|H⟩^⊗6`
(`k = [1, 4, 4, 5, 5, 6]`), not the board's QPG class. All three witness files accepted by
stabrank's verifier at tier `verified`.

## 3. The shot and its coverage (G1, G3)

**No witness in 30.4 h of shot wall** on seven cores (the first run 1.27 h + the tower's 29.00 h),
`taskset -c 21–27`. The run was STOPPED at 22:55Z on 2026-09-25 by Claude Code's low-memory
reaper — not by the search, not by a crash, and short of the planned 40 h; recorded in
`rank1_progress.log` and not restarted (the lead's decision, 2026-09-25: the last eight hours
would have continued a flat search).

**Coverage, exact at the stop (G3):**
- **Partial exclusion, a theorem modulo two float-harvest margins (the tier stabrank's own PR 24
  uses):** no rank-6 decomposition of `|H⟩^⊗7` has three or fewer terms whose span meets
  `span(|H⟩^⊗7, |H^⊥⟩^⊗7)` — the 3+3 and 2+4 splits, the shape of every known construction, are
  ruled out. Route: the member census at `m = 5` (8 rational directions, no rank-1 or rank-2
  member); slicing maps a direction `d ↦ T(d) = (β−α, 2α−β)` and no `d` has `d, T(d), T²(d)` all in
  the census.
- **Lifts, exact and complete:** both known classes of rank-6 decompositions of `|H⟩^⊗6` (the
  board's and the new one) extend to seven copies in **0 ways** (`lifted7.txt`, 4 of 4 entries);
  all 27 split classes at `m = 5` lift only to those two.
- **Five-copy harvest:** 1,236 classes of six-term decompositions of `|H⟩^⊗5` (1.1 × 10¹¹ moves);
  one lifted to six copies and it was not a new class. The annealer never reaches the split
  classes, so its slowing growth is not evidence of completeness.
- **Direct seven-copy annealing:** best residual **0.198** after 5.26 × 10⁹ moves, 0 snaps — flat
  for the whole run.
- **V-split at six copies:** 32 member triples, 256 joins, nothing new; at seven copies the
  census proved the route could not succeed and it was dropped mid-run (item 10).
- **Five-tuple coverage:** the symmetry-reduced space is of order 10⁴⁶–10⁴⁸; the fraction
  decided by enumeration is 0 by arithmetic (item 5). The prereg's "exhaustive branch" was never
  a route at this size.

**Branch (b).** No claim about the rank. The two exact statements above and the second six-copy
class are what is submitted to stabrank, in whatever form they take partial exclusions.

## 4. What the prereg got wrong

Items 1–11 under the prereg's "Notes on building", none moving a stake: (1) the ring — `cos(π/8) ∉
ℚ(ζ₈)`, so `ℤ[ζ₈, 1/2]` cannot hold odd-copy coefficients; (2) "project and test the residual" is
incomplete, replaced by exact pivot completion; (3) the Galois filter acts on tuples, not on
single dictionary states; (4) the dual-distance filter holds at `(7, 6)` and at `(6, 5)` where PR 87
proved it, and PR-4 as written would wrongly fail it against the known witnesses; (5) five-tuple
coverage at `m = 7` is 0 by arithmetic; (6) the annealer alone does not meet G2 at `m = 6`; (7) the
GPU carries the acceptance arithmetic, not the search; (8) one seven-copy smoke read (a lift test)
happened before G2 passed; (9) stabrank's `m = 7` upper bound is 9 (KvWV), not 7 — a witness
settles `χ = 6` outright; (10) the V-split at seven copies was dropped mid-run after the census;
(11) the run ended at 30.4 h of a planned 40 by the harness's memory reaper. The lead's error
to own: the prereg was frozen without reading stabrank's `bounds/` and `research/` first (item 9).

## 5. Findings along the way

1. **A second class of rank-6 decompositions of `|H⟩^⊗6`**, `k = [1, 4, 4, 5, 5, 6]`, distinct from
   the board's QPG class, verified symbolically by stabrank's referee. Small, new, checkable.
2. **The V-split route** meets the six-copy control where annealing and slice-lifting do not, and
   its census proves what it cannot do at seven copies before the budget is spent — the
   cheap-part-first shape of the procedure, on a problem where the cheap part is arithmetic.
3. **Exact acceptance is load-bearing at these sizes** (PR-1): a coefficient perturbation invisible
   to a `10⁻¹²` float check is caught by the ring on every known witness.
4. **The exponent question is untouched.** No rank-6 decomposition of seven copies of the kind any
   known construction has; whether one of another kind exists is open, exactly as it was.
