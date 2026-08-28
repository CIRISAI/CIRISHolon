# Benchmark manifest — measured ratios or it didn't happen

Rule: a performance claim without a moved, recorded ratio does not merge.

| tier | workload | ours | reference | ratio | date |
|---|---|---|---|---|---|
| tableau (packed, holon crate) | n=256 d=5120 + full measure | **0.0078 s** | qiskit StabilizerState 0.024 s | **3.1× faster** | 2026-08-27 |
| tableau (unpacked, holon-qasm) | n=256 d=5120 sample | 0.052 s | qiskit 0.024 s | 0.45× | 2026-08-27 |
| statevector (holon-qasm) | n=20 d=160 | 0.096 s | qiskit Statevector 1.44 s | **15× faster** | 2026-08-27 |
| magic (holon-qasm, exact Z[ω]) | slope in T | 1.005 log2-s/T | no exact reference exists | — | 2026-08-27 |
| DMRG (python, upstream) | Schwinger N=64 | unbenchmarked | ITensor/TeNPy | owed | — |

Named SOTA targets: Stim (Clifford; est. 1–2 orders beyond us — SIMD,
transposed layouts), Aer/qsim (statevector; est. 10–50×), Bravyi–Gosset
extent (magic; 2^t → 2^{0.48t}), ITensor (bulk).

## 2026-08-27, second entry: the full stack, measured against the field

Battle-rig (scratchpad/qasm/BATTLERIG.md upstream; stim 1.16.0, qiskit 2.5.1,
Aer 0.17.2, all on this machine, medians of 3):

| lane | point | ours | best other | reading |
|---|---|---|---|---|
| Clifford | n=64 d=1280 | 4.3 ms (unpacked) | stim 0.2 ms | stim leads 28× (their SIMD lane); qiskit StabilizerState 1,093× behind us and TIMEOUT ≥ n=256 |
| Clifford | n=1024 d=20480 | 8.3 s (unpacked) | stim 26 ms | stim leads 318×; packed planes close ~6.7× of it (next re-run) |
| statevector | n=24 d=192 | 9.1 s (scalar) | Aer 1.7 s (C++/threads) | Aer leads 5.5×; we lead qiskit-numpy 9.6× |
| hidden shift | n=40 t=14 | **0.086 s exact p=1.0** | Aer ext-stab 47.4 s, top outcome 1% (fails) | NARROWED 2026-08-27 (external audit): the two tools answer DIFFERENT questions — one exact selected amplitude vs approximate sampling with an accuracy/runtime dial Aer documents. The honest claim is the exact-amplitude niche (exact where the approximate sampler's dial fails at this setting), not a 550× like-for-like victory |
| hidden shift | n=60 t=28 | **1.09 s exact p=1.0** | nothing finishes | **a column no other tool populates** |
| corrupted control | n=20 t=14 | p = 0.000000 | — | two-sided: the exactness is not a constant-1 artifact |
| GPU fold | 10⁶ branches, n=32 | **6.7 ms** | CPU 32-shard 197 ms (loadavg 29–37) | 336–396× vs serial; struct-level determinism across launch shapes |

## The foundation assumption (directive, 2026-08-27)

Tiers 0 and 1 are PROVEN — exact conformance to external referees, the
boundary measured on both sides, the category demonstrated — and the holon
now ASSUMES them from day one: bit-planes and Pauli-plane tableaus are
settled ground, not hypotheses under test. The development ladder builds
UPWARD on that ground: the magic tier (exponent 0.500, rank-7 slot open),
the entangled bulk (MPS at scale), and then the CRYSTAL tiers — gauge-
coupled matter on the same object, where SCHWINGER-2's verdict is the next
rung's referee. Optimization of the assumed tiers (stim-gap closure via
SIMD/transposed layouts) is ordinary engineering with a named target and
never blocks the climb.

## 2026-08-27, third entry: the packed tableau vs stim, honest triangulation

Three methodologies measured, two rejected with their failure modes named:
per-Python-call driving charges stim ~15 µs/gate of interpreter overhead
(rejected, unfair to stim); compile-inclusive timing charges stim its one-
time ~130 ms circuit compilation (rejected, wrong denominator for long-run
use). The rig's per-call engine timings stand as the stim reference. Against
them, the packed Pauli-plane tableau (holon-run clifford-sample, medians of
3):

| n | packed (ours) | stim (rig reference) | stim leads |
|---|---:|---:|---:|
| 64 | 0.54 ms | 0.2 ms | 2.7× |
| 256 | 8.9 ms | 1.6 ms | 5.6× |
| 1024 | 93.6 ms | 26 ms | 3.6× |

**The gap closed from 28–318× (unpacked) to 2.7–5.6× (packed).** The
remainder is SIMD width and layout transposition — named, mechanical, and
not a blocker for anything above tier 1.

## 2026-08-27, fourth entry: the transpose lands — crossover at n=1024

The mechanical remainder named in the third entry is built: `coltableau.rs`,
the column-major tableau (stim's layout, credited: Gidney, Quantum 5, 497),
every unitary gate ~2n/64 word operations with word-parallel sign masks,
conformance-gated BIT-IDENTICAL to the certified row-major reference on
random circuits to n=130 (planes and signs both; measurement flows through
the reference after one transpose). Same rig methodology as entry three
(engine-only timing both sides, medians of 3, all n qubits measured), same
session, same machine, concurrent background load on both sides
(`conformance/qasm/rerun_stim_h2h.py`, `h2h_col_results.json`):

| n | ours (column) | stim (same-session) | ratio |
|---|---:|---:|---:|
| 64 | 0.178 ms | 0.095 ms | 1.88× behind |
| 256 | 1.995 ms | 0.834 ms | 2.39× behind |
| 1024 | 35.18 ms | 42.10 ms | **0.84× — we lead** |

The word-parallel scaling wins as n grows; the small-n remainder is a
per-gate constant (nested-Vec column indirection, bounds checks — flatten to
one allocation, then re-measure). Entry three's absolute stim numbers differ
from today's because circuits, seeds, and machine load differ; the paired
same-session ratio is the honest quantity, and it now brackets 1× —
the tier-1 target (stim ≤ 1×) is REACHED at n=1024 and remains open below it.

## 2026-08-27, fifth entry: the flag-plant — the category, demonstrated at n=1000

The demonstration the category claim was waiting for
(`conformance/qasm/flagplant.qasm`, results `flagplant_results.txt`):
**n = 1000 qubits, t = 28, 7663 gates, exact amplitude** — a scale no
statevector simulator reaches at any cost (2^1000 amplitudes), computed as
an exact algebraic number:

| shards | amplitude | wall |
|---:|---|---:|
| 1 | exactly 1 (re 1.000000000000, im 0.000000000000) | 2106.6 s |
| 8 | identical | 2056.6 s |
| 32 | identical | 2062.4 s |
| corrupted control | **p = 0.000000000000** | 2528.8 s |

Three shard counts, one answer, bit-identical — the merge law doing in
practice what `MergeLaw.lean` proves in principle. The corrupted control
(one tampered gate) flips the exact amplitude from 1 to 0: the corruption
is not estimated to be probable, it is exhibited. A float engine at this
scale reports ≈1 to some tolerance; the exact engine reports 1, the
rational number.

Honest notes: wall time is flat across shard counts on this run — the
machine was carrying two unrelated computations (SCHWINGER-2's DMRG and
this run's own siblings), so shard speedup is not measurable here and is
not claimed; the demonstrated property is invariance, not scaling. Scaling
curves belong to a quiet-machine session, already owed by entry four.

## 2026-08-27, sixth entry: the bake-off — ahead of stim at every n

Run on a quiet CI runner (the `bakeoff` workflow, run 33110694325; AVX2,
stim 1.16.0, medians of 5, engine-only timing both sides, same circuits,
full terminal measurement both sides):

| n | ours | stim | ours/stim |
|---:|---:|---:|---:|
| 64 | 0.076 ms | 0.110 ms | 0.689 |
| 128 | 0.208 ms | 0.300 ms | 0.694 |
| 256 | 0.655 ms | 1.151 ms | 0.569 |
| 512 | 2.487 ms | 5.301 ms | 0.469 |
| 1024 | 13.615 ms | 25.678 ms | 0.530 |
| 2048 | 73.631 ms | 156.465 ms | 0.471 |
| 4096 | 481.711 ms | 1102.582 ms | **0.437** |

**Ahead at 7/7 sizes: 1.44× faster at the smallest, 2.29× at the largest,
margin growing with n.** The stack that did it: the transposed flat column
engine (gates ~2n/64 word ops), the fused AVX2 rowsum kernel (bit-identical
to scalar by gate), and the one-pass canonical terminal sampler on flat
planes. Whether the margin is the MAXIMUM theoretical amount is a separate
roofline question, deliberately not claimed here; the reached claim is
"faster than stim at every measured n on a quiet machine, reproducible by
workflow dispatch."

## 2026-08-27, seventh entry: the speedups stack — the flag-plant at 850×

All three optimization lanes verified (each rerun and recounted before
acceptance), merged, and cross-certified — the sliced lane's
lane-vs-scalar bit-identity tests now run against the rewritten affine
engine, so the lanes referee each other. The merge resolution took both
independent improvements at the three conflicted phase sites (the new flat
structure AND the pinned `mul_i_pow` unit multiply). 110 tests + the
frozen referee, all green.

**The measured stack, on the banked artifact itself:** the flag-plant
(n=1000, t=28, 7663 gates, exact amplitude) ran in **2.47 s** against the
banked 2106.6 s — **≈850×, and not one bit of the answer moved**
(p = 1.000000000000 exactly, as before). Per the lanes' own analyses the
factor decomposes into the affine engine's algorithmic pass (factored
elimination reuse, early-exit dependence answers, flat bit-matrices:
330–907× on this circuit family), with Magic5 (7.5× fewer branches at
t=28) and slicing (14–26× where t ≤ n) available on the routes the tuner
selects. Positions, not laws: every factor is a position against
Limits.lean's ledger (L2/L3 begging, L1/L4 floors), and the loaded-machine
caveat from the lane reports carries — quiet-runner confirmation of the
full sweep remains the citable form.

## 2026-08-27, eighth entry: the quiet runner confirms the stack — and falsifies a routing rule

Post-merge quiet-runner sweep (bakeoff run 33119952357, loadavg 0.75):

**Tier 1 vs stim, re-confirmed after all three lane merges — ahead at 7/7,
better than entry six:** 0.685 at n=64 down to **0.431** at n=4096. No
regression from the merges; slight improvement.

**The flag-plant on neutral hardware:** 4.82 s (median of 3, identical
exact answer) on a stock CI runner vs the banked 2106.6 s on the (loaded)
production box — **437× on hardware we don't own**, alongside 850× locally.

**And the sweep did referee work on our own tuner:** the CI sliced-surface
table shows the PRUNED path dominating every swept case at t ≤ 12 — v1's
"t ≤ n → sliced" routing rule, measured against the pre-rewrite engine,
is FALSIFIED on the post-rewrite engine (the 330–907× affine pass
accelerated the substrate both alternatives ride on). The tuner is
corrected to v2: certified default everywhere, three named `Unswept`
interactions instead of one, alternatives callable explicitly and
exactness-pinned (`tests/tuned.rs`) so a future sweep can promote them on
speed alone. A routing rule died by measurement within hours of being
written — that is the tuner working exactly as designed.

## 2026-08-27, ninth entry: audit-driven corrections and the Born upgrade

An external audit (accepted as a fix list) qualified two headlines and
named a gap; all three are actioned:

1. **The stim comparison was not semantically equivalent** — our
   `clifford-sample` returned the deterministic canonical witness (free
   bits false) while stim Born-samples. Fixed at the same one-pass cost:
   `sample_born_flat` draws the canonical frame's free bits from a seeded
   stream (which IS the Born distribution for full computational-basis
   measurement), folds their contribution exactly into the constraint
   right-hand sides, logs the seed for replay, and replays through the
   sequential reference in tests. The CLI now Born-samples by default;
   entries six/eight's ratios are re-measured under Born-vs-Born below
   (entry ten, CI). Until then the banked ratios read as "canonical
   support witness vs Born sampler" — established, but narrower.
2. **The Aer line is narrowed in place** (entry one): different questions,
   exact-amplitude niche, no like-for-like 550×.
3. **The 437×/850× numbers are self-relative** — improvements over this
   engine's own banked artifact, never a claimed victory over an external
   solver; entries five and seven now say so explicitly by this entry.

**Owed and named: a QuiZX head-to-head** on structured instances
(hidden-shift class, where graphical simplification reports large exact
results with related decompositions) — the one serious structured exact
solver not yet in the manifest. Until it runs, no claim ranks us against
QuiZX-style simplification.

The audit's summary formula is adopted as the standing claim: unusually
fast exact amplitudes for large-n low-effective-magic Clifford+T, a highly
competitive Clifford kernel, and the novelty in the exact certified
COMPOSITION of structural reductions — not a universally faster solver.

## 2026-08-27, tenth entry: Born-vs-Born — the honest stim comparison, and we still lead at every n

Quiet CI runner, both engines performing BORN-RANDOM terminal measurement
(ours: `sample_born_flat`, seeded free bits folded exactly into the
constraint RHS, seed in the output JSON; stim: its native sampler):

| n | ours | stim | ours/stim |
|---:|---:|---:|---:|
| 64 | 0.083 ms | 0.123 ms | 0.677 |
| 128 | 0.220 ms | 0.348 ms | 0.632 |
| 256 | 0.615 ms | 1.095 ms | 0.562 |
| 512 | 2.340 ms | 4.695 ms | 0.498 |
| 1024 | 14.234 ms | 22.987 ms | 0.619 |
| 2048 | 73.473 ms | 137.539 ms | 0.534 |
| 4096 | 588.342 ms | 1150.893 ms | 0.511 |

**Ahead at 7/7 sizes, 1.48–2.01× faster, semantics matched.** Entry nine's
qualification is discharged: the citable claim is now "faster random
stabilizer sampler at every measured n," with entries six/eight kept as
the narrower canonical-witness measurements they were.

## 2026-08-27, eleventh entry: Magic5 vs pruned — the unswept interaction, swept

`holon-magic-h2h` (exact agreement asserted on every row, medians of 5,
box carrying the DMRG referee — ratios far exceed load noise):

| n | t | pruned | magic5 | magic5 branches | pruned leads |
|---:|---:|---:|---:|---:|---:|
| 16 | 12 | 0.25 ms | 3.6 ms | 36 | 15× |
| 64 | 16 | 1.1 ms | 115 ms | 108 | 103× |
| 128 | 20 | 5.2 ms | 1.58 s | 324 | 306× |
| 256 | 24 | 78 ms | 36.7 s | 972 | 470× |

**Verdict: the tuner's Pruned default stands confirmed by measurement —
but the CAUSE I first wrote was wrong, and the probe corrected it.**

I first attributed the gap to Magic5's per-branch constant (~1000×
behind). A diagnostic probe (`examples/magic5_probe.rs`) measured both
arms directly and refutes that:

| n | t | pruned branches AFTER DEDUP | magic5 branches | per-branch µs (pruned / magic5) |
|---:|---:|---:|---:|---:|
| 64 | 16 | **1** | 108 | 8.8 / 18.5 |
| 128 | 20 | **1** | 324 | 33.2 / 64.8 |
| 256 | 24 | **2** | 972 | 118 / 268 |

Magic5's per-branch cost is only **2.0–2.3×** pruned's — both ride the
rewritten affine engine. The entire 15–500× gap is **DEDUPLICATION**: on
these random circuits the pruned path's canonical merge collapses a
2^{t/2} branch space to ONE or TWO surviving branches, while Magic5
evaluates all 108–972 of its (genuinely smaller-than-naive) branches.
An exponent advantage cannot beat a collapse to O(1).

Named optimization, corrected: give Magic5's branch space the SAME
canonical dedup (its recursion produces many branches that are equal up
to global scalar — exactly what `prune.rs` already merges exactly).
Until then Magic5's regime is circuits whose branch space does NOT
collapse. `Unswept::Magic5VersusPruned` retires to this measured note;
`Magic5TimesSliced` and `SlicedOnRewrittenEngine` remain open.


## 2026-08-27, twelfth entry: the reframed optimization, implemented and REFUTED

Entry eleven's corrected diagnosis named the fix: give Magic5's recursion
the same exact canonical dedup that collapses the pruned path's branch
space. Implemented (`prune::dedup_branches` exposed — one canonical merge,
no second mechanism; `Magic5Source::deduped()`), exactness-gated (every
basis state of random Clifford+T circuits, multiple shard counts, exact
`Cyc` equality), and measured:

| n | t | magic5 branches | after exact dedup | collapse |
|---:|---:|---:|---:|---:|
| 64 | 16 | 108 | **108** | **1.0×** |
| 128 | 20 | 324 | **324** | **1.0×** |
| 256 | 24 | 972 | **972** | **1.0×** |

**Nothing collapses. The hypothesis is refuted, and the reason is a law
worth having: branch-space redundancy and decomposition efficiency are in
TENSION.** The pruned path's naive 2^{t/2} expansion collapses to 1–2
branches precisely because it is redundant — many of its branches evolve
to the same canonical stabilizer state. Magic5's cat-state decomposition
is *designed* to be an efficient near-minimal spanning set, so its
branches are pairwise distinct by construction and there is nothing left
to merge. **An exponent advantage and a dedup advantage compete for the
same redundancy; you cannot bank both.** That is why the pruned default
wins here, and it is not fixable by adding dedup to Magic5.

Residual win, kept: canonicalized branches query ~2× faster
(11.3 vs 19.3 µs at n=64; 127 vs 260 µs at n=256), so `deduped()` pays for
itself when one source answers many queries (sampling, mesh sweeps) —
build cost is one pass over the branch space.

Magic5's honest regime is now precisely stated: circuits whose naive
branch space does NOT collapse under canonical merge. Finding that regime
(structured/hidden-shift families are the candidates) is the next sweep,
not another optimization.

## 2026-08-27, thirteenth entry: QuiZX head-to-head — WE LOSE, and the reason matters more than the ratio

The manifest's one owed comparison against a serious structured exact
solver. Identical circuits (quizx's own Bravyi–Gosset hidden-shift
generator), identical task (one exact amplitude ⟨shift|C|0⟩), both arms
exact, medians of 3, same machine:

| qubits | T-count | quizx | holon | quizx leads |
|---:|---:|---:|---:|---:|
| 12 | 56 | 1.7 ms | 25 ms | 15× |
| 16 | 84 | 3.2 ms | 230 ms | 73× |
| 20 | 112 | 4.3 ms | 866 ms | 202× |
| 24 | 140 | 5.7 ms | 7.29 s | **1285×** |

**The gap is not a constant — it is a complexity-class gap on this family,
and the diagnosis is one number: quizx's reported term count is 1.** Its
ZX-calculus simplification rewrites the whole hidden-shift diagram to a
SINGLE stabilizer term before any decomposition happens. There is no
exponential left to price. Our times grow ~3× per +28 T gates (an
exponential we then fight with dedup); quizx's grow linearly in circuit
size, because it is not doing the same kind of work at all.

**What this says about our engine, stated plainly: we have no
diagram-simplification layer.** Every gain banked today — the affine
rewrite, branch slicing, Magic5, the rank factorization — makes the
*branch sum* faster or smaller. None of them can beat an algorithm that
deletes the branch sum. On structured families (hidden-shift, and by
extension the CCZ-rich circuits ZX rewriting was designed for), a
simplification pass is not an optimization but a prerequisite.

**Named import, now the magic tier's top priority** (it was already in
`PRIOR_ART.md` as multiplicative preprocessing and this measurement
promotes it): ZX/graph simplification as a front-end pass — at minimum
Clifford simplification and phase teleportation (Kissinger–van de Wetering,
arXiv:1903.10477), which reduce T-count *before* any exponent applies. Our
exponent work then multiplies against a smaller t, which is exactly the
composition the sweep predicted and this benchmark forces.

**What the loss does NOT touch:** the stim comparison (tier 1, ahead 7/7,
Born-vs-Born), the exact-ring tower and generic-angle carrier (quizx is
Clifford+T-only; it has no face-angle or symbolic-θ capability), the
distributed certificates, and the refusal discipline. Different axes —
which is precisely why the head-to-head was owed rather than assumed, and
why it is recorded here in full.

## 2026-08-27, fourteenth entry: the simplification pass — 4× faster, and it names its own ceiling

Entry thirteen's forced fix, built and measured. `simplify.rs`: three exact
rewrites at the surface level (diagonal-run cancellation — all diagonal
gates commute so the multiset is all that matters; involution cancellation;
magic cancellation, the only one that lowers the exponent), exactness-gated
on every basis state (`tests/simplify.rs`), on by default in the CLI.

Same circuits as entry thirteen, simplification ON:

| q | gates | magic (T-equiv) | holon before | holon after | our speedup |
|---:|---:|---:|---:|---:|---:|
| 12 | 2061 → **79** | 56 → **28** | 25 ms | 5.6 ms | 4.5× |
| 16 | 2884 → **114** | 84 → 84 | 230 ms | 73 ms | 3.2× |
| 20 | 2496 → **150** | 70 → 70 | 866 ms | 293 ms | 3.0× |
| 24 | 4530 → **202** | 140 → **112** | 7.29 s | 1.40 s | 5.2× |
| 30 | 6164 → **272** | 196 → 196 | 49.9 s | 12.4 s | 4.0× |

**A real, exact, ~4× win for 0.1–0.4 ms of work — and an honest ceiling.**
The gate count collapses 20–26× every time, but the MAGIC weight only drops
when CCZ triples happen to repeat inside one diagonal run (q=12 and q=24);
at q=16, 20, 30 it does not move at all. That is exactly what local
cancellation can do and no more: **we shrank the Clifford bulk, not the
exponent.**

So the standing against quizx improves from 15–8000× to roughly 4–2000×,
and the diagnosis is unchanged: their term count is still 1. The remaining
gap is precisely the non-local rewriting this pass does not attempt —
spider fusion, local complementation, pivoting, and phase teleportation
(Kissinger–van de Wetering, arXiv:1903.10477), which move phases between
distant gadgets and can cancel magic that never meets in a diagonal run.
That is now the named next rung, with this pass as its provable floor.

### Output format: Qiskit's Result schema, with our extras under `metadata`

INTERFACE.md found no spec with a type for our answers. The pragmatic
resolution, now shipping: emit the **Qiskit `Result` shape** (`backend_name`
/ `success` / `results[]` / `data` / `metadata`) so standard tooling reads
us without adaptation, and carry everything no spec has a field for under
`metadata` — `exact`, `ring`, `residual_zeta16`, and the full simplify
record (gates and magic before/after, pass time). Nothing is dropped
silently, and nothing is smuggled into a field that means something else.

## 2026-08-27, fifteenth entry: the non-local pass breaks entry fourteen's ceiling

`phasepoly.rs` — the phase-polynomial normalization entry fourteen named as
its own ceiling (credited: Amy–Maslov–Mosca). Inside a maximal CNOT+diagonal
block every qubit carries an F₂ linear form and every diagonal gate is a
phase on one of them, so **terms on the same form merge no matter how far
apart the gates were**: `CZ` contributes three terms via
`a·b = (a+b−(a⊕b))/2`, `CCZ` seven via the cubic identity (which IS the 7-T
decomposition), and a form whose total power is even costs no T at all.
Exactness gated on every basis state, plus two tests that pin genuine
distance-cancellation the local pass provably cannot reach.

Same circuits, both passes composed:

| q | magic: raw | after LOCAL | after PHASE-POLY | time (entry 13 → 14 → now) |
|---:|---:|---:|---:|---|
| 12 | 56 | 28 | **16** | 25 ms → 5.6 ms → **2.0 ms** |
| 16 | 84 | **84 (local blind)** | **40** | 230 ms → 73 ms → **41 ms** |
| 20 | 70 | **70 (local blind)** | **50** | 866 ms → 293 ms → **272 ms** |
| 24 | 140 | 112 | **64** | 7.29 s → 1.40 s → **0.66 s** |
| 30 | 196 | **196 (local blind)** | **84** | 49.9 s → 12.4 s → **4.35 s** |

**The ceiling is broken exactly where it was named.** At q=16, 20 and 30 the
local pass moved the magic weight by ZERO; the non-local pass cuts it by
52%, 29% and 57%. Cumulative against entry thirteen: **11–12× faster**, all
amplitudes still exactly 1.000000, and the standing against quizx improves
from 15–8000× to roughly 1.2–700×.

Honest reading of what remains: quizx's term count is still 1 and ours is
not. Phase-polynomial normalization is exact for CNOT+diagonal blocks and
Hadamards END a block — full ZX rewriting (spider fusion, local
complementation, pivoting) acts ACROSS Hadamards, which is the rest of the
gap. Named, not implied.

## The job API and output schema

`job.rs`: a job is a circuit plus an OPTIONAL config beside it. Config keys
follow the shapes standard runners already use (`shots`, `seed_simulator`,
`method`, `target`) so a Qiskit/Cirq-literate caller needs no new
vocabulary, with a `holon` section for what no standard has (which passes
run, exactness policy). Unknown keys are IGNORED (forward compatibility with
runners' extra fields); malformed values REFUSE (a silent default is a wrong
answer wearing a right one); `exact: false` refuses by name and points at
the Policy that does lawful degradation. Output is the Qiskit `Result`
schema with everything no spec has a field for under `metadata` — ring,
exactness, ζ16 residual, per-pass magic and gate counts, timings.

## 2026-08-27, sixteenth entry: the simplifier head-to-head — the gap is general, and it is large

Entry fifteen's remaining question was whether quizx's collapse-to-1 was a
general capability or a property of the hidden-shift family. Measured, on
RANDOM Clifford+T circuits (quizx's own generator, identical circuits to
both simplifiers, T-count as the metric):

| circuit | raw T | quizx `full_simp` | our local | our local+phase-poly |
|---|---:|---:|---:|---:|
| q20 d200 | 25 | **0** | 21 | 21 |
| q20 d400 | 58 | **19** | 58 | 56 |
| q30 d400 | 67 | **26** | 65 | 63 |
| q40 d600 | 90 | **19** | 90 | 86 |
| q50 d800 | 111 | **41** | 107 | 107 |

**Two findings, both important, and the second is the harder one to say.**

FIRST: the collapse-to-1 is NOT hidden-shift-specific. On random circuits
ZX rewriting removes **63–79%** of the T-count (and on one instance, all of
it). That is a general capability, not a family artifact.

SECOND: **on random circuits our passes are nearly useless — 2–8%.** Their
29–57% on hidden-shift was a property of THAT family (long diagonal runs of
repeated CZ/CCZ, which is exactly what phase-polynomial normalization
merges). Random circuits interleave Hadamards constantly, so the
CNOT+diagonal blocks are short and almost nothing merges within them. The
honest statement of the ceiling is therefore stronger than entry fifteen's:
**Hadamards do not merely end a block — on realistic circuits they are so
frequent that block-local methods have almost nothing to work with.** ZX
wins because it rewrites the graph where Hadamards are edge decorations
rather than barriers.

Consequence, recorded for the tier ledger: a T-count reduction of 63–79%
is worth 2^{0.4·(t_before−t_after)} in our cost model — 2^15 to 2^28 on
these instances. No branch-summing improvement reaches that. **A real
graph-rewriting layer is not an optimization for the magic tier; it is the
tier's prerequisite**, and every exponent gain we have banked multiplies
against a t that only graph rewriting can lower on realistic input.

## 2026-08-27, seventeenth entry: the graph-rewriting layer — half built, and the half that works taught the lesson

`zx.rs`: our own implementation of the published graph-theoretic
simplification (Duncan–Kissinger–Perdrix–van de Wetering, Quantum 4, 279
(2020); quizx, Apache-2.0, is the reference and the benchmark). Graph-like
form, identity removal, local complementation, pivoting — with the
CIRIS-native twist that adjacency is BIT-PACKED, so a local complementation
is a word-parallel XOR sweep rather than a set-of-sets update.

**What works, measured:** the Clifford layer reduces spiders **5–6×**
(1310 → 209 on q50; 351 → 65 on q20), which is the published behaviour.

**What does not, measured and diagnosed:** T-count is **unchanged** on
every instance, against quizx's 63–79%. Two distinct reasons, and the first
is a theorem worth stating:

1. **Clifford simplification CANNOT reduce T-count, by construction.**
   Local complementation shifts neighbour phases by ±π/2 and pivoting by
   0 or π — both EVEN in units of π/4 — so no Clifford rewrite can change a
   phase's parity. T-count is invariant under `clifford_simp`. The
   reduction lives entirely in gadget fusion, which was not obvious to me
   before building it and is now recorded so it is obvious to the next
   reader.
2. **Our gadget layer produces zero gadgets.** Implemented gadgetization
   (pivot a T-spider into carrier+hub form) and fusion (group by support —
   a bitmask hash, which is where the bit-packing pays), and the diagnostic
   reports **0 gadgets** after Clifford simplification on every instance:
   the pivot precondition never fires on the split spiders. That is an
   implementation defect in our gadgetization, not a property of the
   circuits, and it is the whole distance between 0% and 63–79%.

**Standing recommendation, now evidence-backed rather than argued:** the
composition route is the honest path. quizx is Apache-2.0 (one-way
compatible with our AGPL-3.0), it solves this problem well, and our
distinguishing value — exact rings beyond Z[ζ8], the symbolic-angle
carrier, distributed certificates, the refusal discipline — sits ON TOP of
a simplified circuit, not in competition with the simplifier. Building our
own remains open with the gap now precisely located (gadgetization), which
is worth far more to a future attempt than the two passes it took to find.
