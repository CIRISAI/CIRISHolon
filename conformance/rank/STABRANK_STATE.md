# The field's state for QVM-RANK-1 — stabrank, read 2026-09-24

*STEP 0 of the QVM-RANK-1 run, written before any code. Source: a clone of
`https://github.com/unitaryfoundation/stabrank` at `2ceb676` (2026-09-24 11:25 −0400, PR #143),
its `bounds/`, `docs/notes/`, `research/`, `verify_challenge/`, the project page's state-of-the-art
section (`docs/index.html#soa`, built from `bounds/`), and PRs #16, #23, #24, #87, #88, #89 read
through `gh pr view`. The clone lives in the run's scratch area and its
`verify_challenge/stabrank_verify.py` is the referee (§4). Nothing here is ours; it is what we
are shooting at and what we are graded by.*

## 1. The qubit H-type orbit on the board

`|H⟩ = cos(π/8)|0⟩ + sin(π/8)|1⟩`. Every cell with its best bound file (`bounds/qubit_H-m*-*.json`):

| m | lower | how | upper | how | γ of the upper |
|---|---|---|---|---|---|
| 2 | 2 | two amplitude moduli (verified) | 2 | BSS 2015, annealed witness | 0.500 |
| 3 | 3 | all pairs, quotient by the target (PR #18) | 3 | BSS 2015, annealed witness | 0.528 |
| 4 | 4 | all triples of the 36,720 states (PR #24) | 4 | Labib–Russo 2026, Lean witness | 0.500 |
| 5 | **6** | two-qubit base slice, `research/h5_rank5` (dated **2026-09-24**, attested) | 6 | BSS 2015 | 0.517 |
| 6 | **6** | rank-5 exclusion, 5,939,465 full 5-covers of `|H⟩^3`, 73.4 CPU-h (PR #89, attested) | 6 | QPG 2021 Eq. 7 (cat states), Lean witness | 0.431 |
| **7** | **6** | projection of the m = 6 exclusion | **9** | KvWV 2022 §4.2 (cat₆ + the m = 3 witness), Lean witness | 0.453 |
| 8 | 6 | projection | 12 | KvWV | 0.448 |
| 9 | — | — | 18 | structured construction (QPG × KvWV) | 0.463 |
| 10 | 6 | projection | 18 | KvWV | 0.417 |

The board's best finite-cell exponent for H-type is **0.4170** (m = 10). The published
asymptotic exponent `log₂3/4 ≈ 0.396` (KvWV's `χ(T^t) ≤ 3χ(T^{t−4})`) is not a finite cell of
this orbit; `0.3962` on the board belongs to the BK face orbit (`χ(T_BK^4) = 3`).

**The current bound at m = 7 is `6 ≤ χ(|H⟩^{⊗7}) ≤ 9`.** PR #89's body: *"this gives
χ(H^6) = 6, so the m=6 route below the published exponent log_2(3)/4 is closed and the first
open cell below it is 6 terms at m=7."* Only rank 6 at m = 7 beats `0.396`
(`log₂6/7 = 0.369`; rank 7 gives `0.401`). **The open cell is confirmed as the prereg states
it: is there a six-term decomposition of `|H⟩^{⊗7}`?** Since the lower bound is 6, a witness
would also settle the cell, `χ(|H⟩^{⊗7}) = 6`.

Their own costing of the opposite direction (`docs/notes/next_exclusion_feasibility_2.md`,
2026-09-24, candidate A): a rank-6 *exclusion* at m = 7 is "dead on every slicing route" — the
6-cover census of `|H⟩^3` is 1.6–1.9·10⁹, the 5-cover census of `|H⟩^4` is 1.9 million pivot
pairs at 16–137 s each. Nobody has costed the *search* for a witness at m = 7 beyond the
annealer (`autoresearch/runs.jsonl` has no qubit_H m = 7 run; the m = 6 rank-5 anneals
plateau at residual 0.1516).

## 2. What the field already knows about a six-term witness at m = 7

From `next_exclusion_feasibility_2.md` §2.1 (every fact rests on board bounds only):

- **Fact 1** — every term is full along every single qubit (a one-qubit slice sees ≥ χ(H⁶) = 6
  terms, so all six).
- **Fact 2** — every term is full along every qubit PAIR (a two-qubit slice sees
  ≥ χ(H⁵) = 6). Equivalently: **the direction space of every term has dual distance ≥ 3** —
  PR #87's property P, which for m = 7, rank 6 now follows from `χ(H⁵) = 6` (filed today).
- **Fact 3** — at any three-qubit point at most two terms are absent (`χ(H⁴) = 4`); along a
  triple every term is an 8-point term or a 4-point term on the even (E) or odd (O) coset, at
  most two on each.
- Along a qubit quadruple directions have dimension 3 or 4; at most three absent at any
  four-qubit point (`χ(H³) = 3`).

PR #87 (two-qubit slicing, `research/constructions/two_qubit_slice.py`): no rank-5 decomposition
of `|H⟩^6` has a four-qubit slice with exactly four terms, which is where property P came from;
it also left `|H⟩^7` at rank 8 open.

PR #88 (the argument and kernels for the m = 6 rank-5 exclusion): property P gives a triple
and a base point where all five terms are visible, so the base slice is a full 5-cover of
`|H⟩^3` from the 1,080-state dictionary; enumeration and matching run mod 65521 as a
superset, every hit re-decided mod 2013265921 and numerically; compiled kernels
(`cpp/src/cover5.cpp`, `slice_match.cpp`); two positive controls — m = 4 rank 4 recovers
exactly the 23 symmetry classes of the stored rank-4 list, and the board's rank-6 witness
is recovered from each of its four all-visible bases.

PR #89 (the run): 190 batches, 73.4 CPU-hours; stage A 5,939,465 full 5-covers with distinct
independent base states, stage B 12,390 dependent covers, stage C 13,852 covers with a
repeated state; the two moduli (65521 superset, 2013265921 re-decision) plus a numerical
re-decision; attested with `research/h6_rank5/batch_manifest.json`, two seeded batches
re-run from scratch in the certificate.

PR #16 (the Galois constraint, qutrit `T₃`): the span of any exact decomposition is defined
over the field of the stabilizer states' coordinates and is therefore stable under the Galois
group of the target's field over it. **The qubit instance** (Lovitz–Steffan Prop. 3.7 over
`Q(i)`, as PR #16 credits): unnormalised qubit stabilizer states are vectors over `ℤ[i]`;
`|H⟩^{⊗m} ∝ (|0⟩ + (√2−1)|1⟩)^{⊗m} = a + √2·b` with `a, b ∈ ℤ^{2^m}`; the automorphism
`√2 ↦ −√2` of `Q(ζ₈)/Q(i)` fixes every stabilizer state and sends the target to
`|H^⊥⟩^{⊗m}` (up to scale). **So the span of every decomposition contains the 2-dimensional
`V_m = span(|H⟩^{⊗m}, |H^⊥⟩^{⊗m}) = span(a, b)`.** This is a constraint on tuples, not on
single states (every single stabilizer state passes it).

PR #23 (verified witnesses): every cited upper cell got an exact witness; the qubit_H m = 6
correction (rank 6 is QPG 2106.07740 Eq. 7, not Bravyi–Gosset; the rank-7 annealed witness
stays on the ledger as history). PR #24 (exhaustive rank-2/3 exclusion, `rank_exclusion.py`):
pivot + quotient by `span(ψ, s_i)` + parallel-pair sort with a random functional, every
candidate retested; `χ(H⁴) ≥ 4` first proved there (8.69 million collinear candidates, all
dependent triples); `exact: true` certificates earn `verified`.

## 3. The known witnesses (seeds and controls)

| cell | where | form |
|---|---|---|
| `|H⟩^4`, rank 4 | `bounds/qubit_H-m4-upper-4.json` (one); `research/constructions/data/qubit_H_m4_rank4.json` (**all 30** up to the 384-element symmetry group, from `slice_lift.py`; PR #88 counts 23 classes under its matcher's convention) | flat-and-phase terms |
| `|H⟩^6`, rank 6 | `bounds/qubit_H-m6-upper-6.json` (QPG cat construction; coefficients `−√2/4, 1/2, 1/2, −√2/4, 1/2, 1/2`; terms with `k = 6, 1, 5, 5, 6, 2`) | flat-and-phase terms |
| `|H⟩^7`, rank 9 | `bounds/qubit_H-m7-upper-9.json` (KvWV) | flat-and-phase terms, coefficients `±sqrt(1/8 ± √2/16)`… |
| `|H⟩^{2,3}` | `research/constructions/data/qubit_H_m{2,3}_rank{2,3}.json` | flat-and-phase terms |

Note the m = 6 witness has a `k = 1` term and a `k = 2` term: at m = 6, rank 6 the analogue of
Fact 2 does not hold (a pair slice there need only see `χ(H⁴) = 4` terms), so property P is a
fact about (m, r) = (6, 5) and (7, 6), **not** about the known witnesses.

## 4. The referee's interface

`python3 verify_challenge/stabrank_verify.py <submission.json>` prints `PASS`/`FAIL`, the tier,
and the implied γ. An upper-bound submission is a JSON object (`schema/bound.schema.json`):

```json
{"schema_version": "0.1", "orbit": "qubit_H", "m": 7, "direction": "upper", "rank": 6,
 "provenance": {"author": ..., "reference": ..., "method": ..., "date": ..., "github": [...],
                "compute": {...}},
 "witness": {"terms": [{"k": k, "x0": [m bits], "W": [k rows of m bits],
                         "Q": [[k×k, upper triangle read]], "l": [k values mod 4]}, ...],
             "coeffs": ["<sympy expression>", ...]}}
```

Term `j` is `2^{−k/2} Σ_{y∈F₂^k} i^{l·y} (−1)^{Q(y)} |x0 + W y⟩` (l·y summed over the integers
mod 4, `Q` upper triangle incl. diagonal, strings read most-significant-bit first — qubit 1 is
the leftmost bit). The target is `sp.Matrix([cos(π/8), sin(π/8)])^{⊗m}`. The verifier rebuilds
every term (it cannot express a non-stabilizer state), sums `Σ c_j s_j` in SymPy, and
requires every entry of the difference to vanish — symbolically, or to 60 significant digits
below `1e−45` as a fallback; a zero coefficient is refused ("resubmit at the smaller rank").
Pass → tier `verified`. **Caveat for controls:** a file with a `lean` key and a build receipt
in `certs/` short-circuits to tier `lean` without the exact check, so the controls here strip
`lean` before submitting (a stripped m = 7 rank-9 board witness verifies symbolically in
7.5 s here). Lower bounds need a certificate script (`reproduced`, `verified` if declared
exact, `attested` with a batch manifest). The three routes the prereg names map to: the
numerical check (their annealer's residual), the SymPy exact check (this verifier), Lean
(`lean_proofs/`, a per-witness module proving `stabRankP`; out of reach for a new witness here
without a mathlib build).

The m = 7 coefficients need `cos(π/8)` (odd m): `|H⟩^{⊗7} = cos⁷(π/8)·(a + √2 b)` and
`cos(π/8) ∉ Q(ζ₈)`. A witness is therefore written with coefficients
`cos(pi/8)**7 * (rational + rational*sqrt(2) + i(...)) * 2**(k/2)` — see the prereg's notes on
building.

## 5. What this changes for QVM-RANK-1 (recorded in the prereg's notes, no stake moved)

1. The ring in §1 of the prereg is too small for odd m (item above).
2. The Galois filter for qubit H is the `V_m ⊂ span` constraint of §2, a tuple filter.
3. The dual-distance filter's premise is `(m, r) = (7, 6)` (from `χ(H⁵) = 6`); at m = 4 and
   m = 6 the known witnesses violate the per-term form, as they are allowed to, so PR-4 must
   test each filter under its own premise at each m.
4. The board already records every symmetry-free route at m = 7 as out of reach for an
   exclusion; an S₇-invariant ansatz was run only at m = 6, rank 5 (`perm_symmetric_qubit.py`:
   no S₆-invariant decomposition with ≤ 5 terms); **subgroups of S_m and subgroups involving
   the local Cliffords are listed as "not done"** (`constructions_2026_09.md` §2) — that is
   where an exhaustive branch at m = 7 can say something new.
