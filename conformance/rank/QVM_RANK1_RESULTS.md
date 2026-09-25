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
| **G1** a six-term witness of `|H⟩^⊗7`, accepted by stabrank's verifier | the shot | **none found** in {{SHOT_WALL}} h of wall on seven cores (§3) | **NOT MET** (no kill; branch (b)) |
| **G2** the pipeline, told nothing, recovers rank 4 at m = 4 and rank 6 at m = 6 within an hour each, each accepted by the referee | kill: either not recovered in an hour | m = 4: annealing **0.1 s**, and the exhaustive pivot completion **97.6 s** (every one of the 1,921,914 canonical pivot pairs; **23** classes, PR 88's number; stabrank's stored list of 30 falls into the same 23). m = 6: the annealer alone failed (10 min), the slice-lift tower failed (13.6 min), the V-split found it in **330 s** — **≈ 29 min cumulative**. Referee: `verified`, symbolic, on all three files | **MET** — by the pipeline as built, not by the annealer the prereg names (notes item 6) |
| **G3** exact coverage if no witness | orbit count, fraction decided, filters and their removals, annealing budget; a partial exclusion certificate | §3: the split exclusion (a theorem modulo two float-harvest margins, stated), the lift branch (every known class of rank-6 decompositions of `|H⟩^⊗6` decided: 0 extensions), the five-tuple fraction (≈ 0, by arithmetic), the move budgets | **MET** as a partial exclusion; its tier in stabrank's terms is `reproduced`/`attested`, not `verified` (§3.4) |
| **G4** every residual batch on the GPU equals the CPU twin's struct | struct equality | **{{G4_BATCHES}} batches, {{G4_FOLDS}} folds, struct-equal on all**, value `= den·v(y)` on all | **MET** |

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

{{INSTRUMENT}}

## 2. G2 — the controls

{{G2}}

## 3. The shot and its coverage (G1, G3)

{{SHOT}}

## 4. What the prereg got wrong

{{WRONG}}

## 5. Findings along the way

{{FINDINGS}}
