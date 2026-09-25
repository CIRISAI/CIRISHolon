# QVM-RANK-1 — six stabilizer terms for seven copies of the H state? PREREGISTRATION

*Frozen 2026-09-24, committed alone before any code. The open cell, as the Unitary
Foundation's `stabrank` project states it after closing `χ(|H⟩^{⊗6}) = 6` on 2026-09-22 (PR 89,
a rank-5 exclusion over 5,939,465 covers, 73 CPU-hours, attested): **is there a six-term
stabilizer decomposition of `|H⟩^{⊗7}`?** A witness would give `χ(|H⟩^{⊗7k}) ≤ 6^k`, a per-copy
exponent `log₂6 / 7 ≈ 0.369`, below the published `log₂3 / 4 ≈ 0.396` (Kissinger–van de
Wetering–Vilmart 2022; Qassim–Pashayan–Gosset 2021; Bravyi–Gosset 2016) that this engine's
magic tier ports. This is a SHOT, not a symmetric test: a witness is decisive and
checkable by others; its absence within budget proves nothing, since a rank-6 exclusion at
seven copies is far beyond the six-copy exclusion's cost. The stakes are written so that
either outcome is banked honestly.*

## 1. The object and the referee

`|H⟩ = cos(π/8)|0⟩ + sin(π/8)|1⟩`, the `+1` eigenstate of the Hadamard. A witness is a set of six
seven-qubit stabilizer states `{s₁…s₆}` and coefficients `c_i` in the ring `ℤ[ζ₈, 1/2]` (the
engine's `Cyc`, with `√2 = ζ₈ + ζ₈⁻¹`) such that `Σ c_i s_i = |H⟩^{⊗7}` EXACTLY, with each `s_i`
given by its stabilizer tableau (or its affine-form amplitude description). **The referee is
stabrank's own verifier** (numerical at machine precision, exact rational in SymPy, and its
Lean 4 route where its format admits the witness) — not ours. A witness that our exact ring
accepts and their verifier does not is NOT a witness; the disagreement is the finding.

## 2. Mechanisms, declared (the engine's best, and the field's)

- **Exact residuals only.** Every candidate span is tested in `Cyc`; no floating-point
  acceptance anywhere on the accepting path. Floats may PRUNE (a numerically far residual
  is discarded), never accept.
- **The cheap part first (OBJECT.md's procedure).** A six-term witness is a five-term
  partial span plus a residual that is itself a stabilizer state. The search therefore
  enumerates five-tuples (and four-tuples with a rank-≤2 residual) and applies an EXACT
  stabilizer-ness test to the projected residual (affine support with a quadratic phase,
  read off the amplitudes in the ring) — a test the tableau tier makes cheap. The
  combinatorial part is over fewer states; the arithmetic part is exact.
- **Symmetry reduction, declared.** The stabilizer of `|H⟩^{⊗7}` in the Clifford group
  contains `S₇` (qubit permutations) and `H^{⊗k}` on any subset (`|H⟩` is `H`-fixed): a group
  of order `7! · 2⁷ = 645,120`. Candidate tuples are enumerated up to this group; the orbit
  representatives are counted and the count is printed and checked against a brute count
  at `m = 3, 4`.
- **The field's pruning, credited.** stabrank PR 87's property (every term's direction space
  has dual distance `≥ 3`) and PR 16's Galois constraint on decomposition spans are applied
  as filters on the dictionary, each with a plant showing it keeps every known witness.
- **Seeds.** The magic tier's own decomposition (`magic5.rs`), the known rank-4 witness of
  `|H⟩^{⊗4}` and stabrank's rank-6 witness of `|H⟩^{⊗6}` are seeds for a local search
  (annealing with Pauli, Clifford and single-state moves, as stabrank does) with EXACT
  acceptance; the exhaustive branch runs beside it.
- **The GPU fold and the mesh** evaluate residual batches bit-identically to a CPU twin
  (`holon-gpu`'s i128 lanes; `QVM_GPUFOLD1_RESULTS.md`); the device class is declared.

## 3. Stakes

- **G1 — a witness.** Six seven-qubit stabilizer states and ring coefficients whose exact
  sum is `|H⟩^{⊗7}`, accepted by stabrank's verifier. **This is the shot; there is no kill on
  its absence.**
- **G2 — the controls, before any seven-copy search is read.** The same pipeline, told
  nothing, recovers a rank-4 witness of `|H⟩^{⊗4}` and a rank-6 witness of `|H⟩^{⊗6}` (any
  member of their symmetry classes), each accepted by the referee. **Kill:** either control
  not recovered within an hour — the pipeline cannot find what exists, and nothing at
  seven copies is read.
- **G3 — coverage, the honest alternative result.** If no witness lands in the budget, the
  campaign reports EXACTLY what was exhausted: the enumeration's orbit count, the
  fraction of the symmetry-reduced five-tuple space decided, the filters applied and the
  count each removed, and the annealing's move budget — a partial exclusion certificate
  in stabrank's terms, submitted to them as such if it is a form they take.
- **G4 — bit-identity of the device path.** Every residual batch evaluated on the GPU equals
  the CPU twin's struct; a corrupted lane is convicted.

## 4. Plants

| plant | must |
|---|---|
| **PR-1** | a witness perturbed by one ring unit in one coefficient is REJECTED by the exact residual and ACCEPTED by a float check at `10⁻¹²` — the exact path is load-bearing |
| **PR-2** | the exact stabilizer-ness test accepts every state of the three-qubit dictionary (1,080 states) and rejects `|H⟩^{⊗3}` and 1,000 random non-stabilizer states |
| **PR-3** | the symmetry-reduced enumeration's orbit counts at `m = 3, 4` equal the brute counts over the full group |
| **PR-4** | each filter (dual distance, Galois) keeps every term of every known witness at `m = 4, 6` |
| **PR-5** | the GPU twin: a flipped lane convicted by position |

## 5. Budget and placement

Two days of wall: cores 21–27 and the RTX 4090. Cores 0–19 carry running campaigns and are
not touched. The exhaustive branch is checkpointed so the coverage in G3 is exact whenever
it is stopped.

## 6. Branches

- **(a)** G1 met → the witness is submitted to `stabrank` with its exact certificate; the
  exponent `0.369` is theirs to attest; banked here only after their verifier accepts it.
- **(b)** G1 not met in budget → G3's coverage is the result; banked as a partial exclusion,
  no claim made about the rank.
- **(c)** G2 killed → the pipeline is wrong; nothing at seven copies is read.
- **(d)** a witness our ring accepts and the referee rejects → the disagreement is the
  finding and is resolved before anything is claimed.

## 7. What this does not test

Any other magic state or orbit; the qutrit cells; a lower bound at seven copies.

---
witness: `lanes_shardedFold_invariant` (the fold's schedule-independence, `holon-gpu`) for G4; the stabilizer-ness test's correctness is `tableau_closed_under_hadamard`'s engineering face (Stabilizer.lean) — the amplitude form of a stabilizer state; G1–G3 are computational
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-CHEAPER-THAN-ITS-PRICE, M-DEVICE-CLASS, M-PLACEMENT-LOTTERY, M-VACUOUS-SUCCESS, M-VALIDATED-NOT-WIRED, M-PARITY-PROTECT, M-HOMOG, M-FLOOR-UNSTAKED, M-COND-PROBE, M-STALE-INSTRUMENT, M-IDLE-CALIBRATED-TIMEOUT, M-MAINTENANCE-LENS — contacted by keyword, cited.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier (a perturbed witness, the three-qubit dictionary, the m = 3, 4 enumerations, the known witnesses, a flipped lane), and the sector the plant acts on is nonzero in that carrier by construction.

### Notes on building

*Appended 2026-09-24 by the QVM-RANK-1 run, after reading the field (`STABRANK_STATE.md`) and
before the instrument existed. No stake is moved; these are the places where the frozen text
and the mathematics disagree, named before any search is read.*

1. **The ring in §1 is too small for odd m.** `|H⟩^{⊗m} = cos^m(π/8)·(|0⟩+(√2−1)|1⟩)^{⊗m}` and
   `cos(π/8) = √(2+√2)/2 ∉ Q(ζ₈)`; for m = 7 no decomposition has all `c_i ∈ ℤ[ζ₈, 1/2]`. Nor
   is `1/2` the only denominator the rescaled problem needs: the coefficients of the rescaled
   target `a + √2 b` live in `Q(ζ₈)` with arbitrary odd denominators in general. The
   instrument therefore works on the rescaled target (entries in `ℤ[√2]`, stabilizer states
   unnormalised over `ℤ[i]`) and accepts by an integer identity `Σ α_i s_i = D·(a+√2 b)` with
   `α_i ∈ ℤ[ζ₈]` (the `Cyc` ring, `m = 0`) and `D ∈ ℤ`; the emitted witness carries
   `cos⁷(π/8)·α_i/D·2^{k_i/2}` as its SymPy coefficients. Exactness is unchanged; the
   ring's name was wrong.
2. **"Project and test the residual for stabilizer-ness" is not the complete test.** If
   `ψ = Σ_{i≤5} c_i s_i + c_6 s_6`, the residual of `ψ` after orthogonal projection onto
   `span(s_1..s_5)` is `c_6 (I−P) s_6`, which is a stabilizer state only when `s_6 ⊥` that
   span. The complete statement is: a sixth state exists iff `U = span(s_1..s_5, ψ)` contains
   a stabilizer state outside `span(s_1..s_5)` — a dictionary query, not a single test.
   The instrument implements the complete form (pivot tuples, then a scan of the dictionary
   for states lying in the pivot span plus the target — PR #24's quotient form), keeps the
   residual test as the cheap special case, and reports both.
3. **The Galois filter for qubit H is a tuple filter.** PR 16's argument for qubits (every
   unnormalised stabilizer state is a `ℤ[i]` vector; `√2 ↦ −√2` fixes them and maps the
   target to `|H^⊥⟩^{⊗m}`) says the span of any decomposition contains
   `V_m = span(|H⟩^{⊗m}, |H^⊥⟩^{⊗m})`. It removes no single state from the dictionary. As a
   tuple filter it is strong: for rank 6, every five of the six terms satisfy
   `rank(s_1..s_5, a, b) ≤ 6`, and a four-tuple pivot fixes the whole 6-dimensional span
   `span(s_1..s_4, a, b)` in which the remaining two terms must lie.
4. **The dual-distance filter has a premise, and PR-4 as written would fail it for the right
   reason.** "Every term's direction space has dual distance ≥ 3" is PR 87's property at
   `(m, r) = (6, 5)` and, since `χ(H⁵) = 6` (filed 2026-09-24), at `(7, 6)`. The known rank-6
   witness of `|H⟩^6` has a `k = 1` and a `k = 2` term and violates it, as it may — a pair slice
   of a rank-6 decomposition of `|H⟩^6` need only see `χ(H⁴) = 4` terms. The instrument
   implements the general slice-visibility filter (at a `j`-qubit slice point at least
   `χ(H^{m−j})` terms are visible, board values), which reduces to dual distance ≥ 3 per term
   at `(7, 6)`; PR-4 is run with each filter under its own premise at `m = 4` and `m = 6`.
5. **Five-tuple coverage at m = 7 over the full dictionary is ≈ 0 by arithmetic, not by
   effort.** The seven-qubit dictionary has 81,284,860,800 states; the full-support states
   alone (all pass the dual-distance filter) number `2^35 ≈ 3.4·10¹⁰`, so the
   symmetry-reduced five-tuple space is of order `10⁴⁸`. G3's "fraction decided" over that
   space is reported exactly and will be of order `10⁻⁴⁰`. The exhaustive branch is
   therefore ALSO run over declared invariant subclasses that it can decide completely
   (term sets invariant under a subgroup `K` of `S₇ × H^{⊗k}` — the ansatz stabrank lists as
   "not done" for subgroups other than `S_m`), each reported with its own exact coverage.

### Notes on building (the instrument and G2)

*Appended 2026-09-24 ~11:30 CDT, after G2 was read and before the seven-copy shot was read. No
stake is moved.*

6. **"The pipeline" at m = 6 is not the annealer the prereg names.** The declared local search
   (Pauli, Clifford and single-state moves on six terms, float residual, exact acceptance, plus
   the pivot-completion snap) found a rank-4 witness of `|H⟩^4` in 0.1 s and did NOT find a
   rank-6 witness of `|H⟩^6` in 10 minutes on seven cores (4.7·10⁸ moves, best residual
   1.67·10⁻²) — stabrank's own annealer never produced that cell either (their witness is the
   QPG construction). A second route, the slice-lift tower (harvest rank-6 decompositions of
   `|H⟩^5` by annealing — 146 classes in 13.6 min — and lift each one qubit up, completely and
   exactly), also found nothing at m = 6 in its 13.6 minutes. What recovered it is a third
   mechanism the prereg does not list, built from its own Galois note (item 3): **the V-split**.
   Anneal triples toward `span ∩ V ≠ 0` (a smooth objective, `1 − λ_max(B†P_S B)`), complete
   every good pair exactly (the triple's third state lies in `span(pair, a, b)`, found by
   `complete_pivot`), and join two triples whose members of `V` are independent. It found a
   rank-6 witness of `|H⟩^6` in 330 s, accepted by stabrank's verifier (`verified`, symbolic).
   Cumulative wall to the m = 6 control, all three routes counted: ≈ 29 minutes, inside the
   hour. G2's letter ("the same pipeline, told nothing") holds for the pipeline as built; it
   does not hold for the annealer alone, and that is recorded rather than smoothed.
7. **Every member line is rational.** A decomposition's span is `σ`-stable (`√2 ↦ −√2`), so a
   sub-tuple's span meets `V` in a `σ`-stable subspace; a line in `V` is `σ`-stable iff its
   direction `α a + β b` has `β/α ∈ ℚ(i)`. The QPG witness is two triples meeting `V` along
   `5a + 7b` and `7a + 10b` — consecutive convergents of `√2` (`qpg_witness_is_a_v_split`).
8. **The exhaustive branch is the slice lift, not five-tuples.** At `(m, r) = (7, 6)` every term
   is visible on both slices of every qubit (Fact 1, `χ(H⁶) = 6`), the slice terms are
   independent, so the coefficients are forced, and the other slice is `Σ c_j μ_j P_j a_j`
   (stabrank PR 87's structure lemma). `lift` decides EVERY extension of a given rank-6
   decomposition of `|H⟩^6` to `|H⟩^7` exactly (meet-in-the-middle over `256³` half-sums, two
   functionals, exact confirmation) in ~30 s; its completeness is planted on the QPG witness
   (`lift_recovers_the_qpg_witness_from_its_slice`: every one of its six slices lifts back to
   it, uniquely). G3's coverage is therefore stated as "these rank-6 classes of `|H⟩^6` do not
   extend", which is exact for each class and makes no claim about the classes not found.
   The prereg's five-tuple orbit enumeration is built and planted (PR-3) but is not the
   search at m = 7 (item 5). The pivot completion that replaces the residual test (item 2) is
   exact and complete per pivot set with `rank(P, a, b) = r`, and every witness has such a
   subset (its terms' images span `span/V`); at m = 4 it enumerates every canonical pivot
   pair (1,921,914) in 97.6 s and finds **23** rank-4 classes of `|H⟩^4` — exactly PR 88's
   count of "the 23 symmetry classes of the stored rank-4 list".
9. **The device carries acceptance, not search.** Neither the float least-squares objective
   nor the completion scan is a branch fold, so the GPU does what `holon-gpu`'s fold does: the
   exact residual `Σ α_j s_j − den·v` of every decomposition the campaign holds (the 36 known
   ones and every one the campaign writes), at every `y`, against `cpu::fold_packed`. The
   search runs on the CPU (`std::thread`, cores 21–27), not on `holon::mesh` — its fold
   is for amplitudes, not for this shape.
10. **A seven-copy read happened before G2 passed**, once: the QPG witness's lift to `|H⟩^7`
    was run as a smoke test of `lift` at 10:56 CDT (0 lifts, 30.5 s). It is re-run inside the
    shot and reported there; nothing was tuned on it.
11. **Mid-run reallocation, recorded (no stake moved).** The shot started 11:28 CDT with
    v6 2 + l7 2 + v7 2 + a7 1. At shot-wall 1.27 h the member census of `V_5` (rank ≤ 3: eight
    directions, no rank-1 or rank-2 member, no chain `d, T d, T² d`) proved the v7 branch's
    joins can never succeed (every split of `|H⟩^⊗7` is excluded), so the tower was restarted
    from its checkpoints with v7 dropped: h5 5 + v6 1 + l7 1 + a7 1 (`shot/run3.log`,
    `rank1_progress.log` marks both restarts). The m = 5 harvest (h5) never reaches the slices
    of the known m = 6 classes (they are splits; the annealer finds only non-split classes),
    which is why its saturation is NOT evidence of completeness.
