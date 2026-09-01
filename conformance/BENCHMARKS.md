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

## 2026-08-27, eighteenth entry: WASM size — 65 KB vs 390 KB, and why

Identical build settings both sides (`opt-level = "z"`, LTO, one codegen
unit, `panic = "abort"`, stripped), `wasm32-unknown-unknown`, each exposing
one entry point that exercises its own engine end to end:

| build | raw | gzipped | what it contains |
|---|---:|---:|---|
| **holon alone** | **64.8 KB** | **35.5 KB** | QASM superset parser, both simplifier passes, affine engine, exact ring, job API, merge law |
| quizx alone | 390.1 KB | 138.3 KB | graph rewriter + decomposer + its dependency tree |
| **composed** (quizx simplifies, holon evaluates) | **335.6 KB** | **118.1 KB** | quizx's simplifier + all of holon |

**Two facts worth separating.**

FIRST — **the composed build is SMALLER than quizx alone (336 vs 390 KB).**
Not a paradox: the composition needs only quizx's *simplifier*, because
holon does the evaluation, so quizx's decomposer never links. That is the
composition working as designed — take the layer they do better, keep ours
for the rest — and it shows up as bytes rather than as an argument.

SECOND — **holon is 6× smaller than quizx, and the reason is the
dependency tree, not cleverness.** `holon` has **zero external
dependencies**; quizx pulls `num`, `rustc-hash`, `rayon`, `ndarray`,
`approx`, `regex`, `rand`, `itertools`, `openqasm`, `rstest`, `serde`,
`serde_json`, `derive_more`. Ours is small because of choices the exactness
discipline forced anyway: exact integer arithmetic needs no float or bignum
library, bit-packed representations are code-light, and the QASM parser was
written rather than pulled in (which is also why its lowering rules could
each be certified individually).

**One honest qualifier, stated because it flatters us otherwise:** 64.8 KB
is what is REACHABLE from that entry point after dead-code elimination —
the residue carrier, the cyclotomic tower, the sliced evaluator, the ZX
graph and the GPU path are not linked by this particular export. The
engine's whole surface is larger; what this measures is the cost of
*shipping a working exact simulator to a browser*, which is the number that
matters for the WASM tier and is the honest comparison against quizx's
equivalent export.

## 2026-08-27, nineteenth entry: the QuiZX ladder, completed — final table

The sweep finished. Its holon column was measured BEFORE the simplifier
passes landed, so the honest table pairs quizx against BOTH: the engine as
the sweep found it, and the engine as it stands with both passes wired.

| qubits | T-count | quizx | holon (sweep, pre-passes) | holon (both passes) | quizx leads (current) |
|---:|---:|---:|---:|---:|---:|
| 12 | 56 | 1.7 ms | 25 ms | **2.0 ms** | 1.2× |
| 16 | 84 | 3.2 ms | 230 ms | **41 ms** | 13× |
| 20 | 112 | 4.3 ms | 866 ms | **272 ms** | 63× |
| 24 | 140 | 5.7 ms | 7.29 s | **0.66 s** | 116× |
| 30 | 196 | 6.2 ms | 49.9 s | **4.35 s** | 700× |
| 40 | 350 | 21.5 ms | TIMEOUT >900 s | **TIMEOUT >900 s** | **≥42,000×** |

**The passes bought 3–11× and did not change the shape.** At q=40 the
engine still times out where quizx answers in 21 ms — a gap of at least
four orders, and the honest reading is the one entries 16–17 established:
this is the tier's missing canonical form, not a constant factor. The
Clifford tier has its canonicalizer (the tableau, now explicit) and leads
stim; the magic tier lacks its own (the ZX graph) and loses here. One
structural fact explains both columns.

Recorded as the campaign's clearest measured loss, alongside its clearest
measured win, in the same document.

## 2026-08-27, twentieth entry: the canonicalizer, actually integrated

Entry seventeen RECOMMENDED composition and never did it — the composed
build measured in entry eighteen was a size experiment in /tmp, not an
integration. It is integrated now: `crates/holon-zx`, a SEPARATE crate
behind a `zx` feature so `holon`'s core stays zero-dependency and 65 KB
while callers opt into the canonicalizer here. quizx canonicalizes, holon
evaluates exactly. (quizx: Apache-2.0, one-way compatible with AGPL-3.0;
credit Kissinger–van de Wetering and the DKPvdW rewrite rules.)

**The certificate, and the bug it caught.** The gate is amplitude equality
on every basis state — the same bar our own passes carry, applied to a
third-party simplifier we do not control — and it FAILED on first run.
Diagnosis: probabilities matched exactly and the amplitude ratio was a
CONSISTENT global phase (ω^{−1}, ω^{+1} on different trials), i.e. quizx's
circuit extraction returns the state up to the graph's scalar, which it
does not re-insert. Benign, expected, and fatal for us — **amplitudes are
the product**. Fixed exactly, not tolerated: the scalar is read from
quizx's own `exact_phase_and_sqrt2_pow`, converted to a ζ8 exponent, and
REFUSED if it is not one ("outside Z[ω]; refusing rather than rounding").
With the phase carried, amplitude equality passes.

**What it buys on the ladder (T-count, and the whole pass costs ms):**

| q | T before | T after | gates | pass time |
|---:|---:|---:|---:|---:|
| 12 | 56 | **16** | 2061 → 154 | 20 ms |
| 16 | 84 | **40** | 2884 → 373 | 28 ms |
| 24 | 140 | **64** | 4530 → 634 | 49 ms |
| 30 | 196 | **84** | 6164 → 905 | 71 ms |
| 40 | 350 | **186** | 10636 → 2164 | **154 ms** |

**47–71% of the T-count, for milliseconds** — against our own passes' 2–8%
on random circuits. At q=40, where the engine timed out past 900 s, the
canonicalizer alone cuts t from 350 to 186 in 154 ms; at the magic tier's
cost model that is a factor of 2^{0.4·164} ≈ 2^65 off the exponent.

The tier now has all three things the per-tier analysis requires — the
capability, the adaptivity, and the canonical form — with the honest note
that the third is COMPOSED rather than ours, our own attempt having reached
the Clifford half only (entry seventeen, gap located at gadgetization).

## 2026-08-28, twenty-first entry: the composed ladder — the owed experiment, and it refutes the assumption behind it

Entry twenty measured the canonicalizer's T-count reduction and stopped
there; an external review correctly noted the end-to-end runtime was never
measured. It is now (`holon-zx/examples/composed_ladder.rs`, canonicalize →
evaluate, exact amplitudes throughout):

| q | T raw | T after canon | canon time | eval time | composed total | our own passes (entry 15) |
|---:|---:|---:|---:|---:|---:|---:|
| 12 | 56 | **16** | 5 ms | 5 ms | 9 ms | **2.0 ms** |
| 16 | 84 | **40** | 5 ms | 60 ms | 65 ms | **41 ms** |
| 20 | 112 | 48 | 7 ms | 262 ms | 269 ms | — |
| 24 | 140 | **64** | 8 ms | 1.77 s | 1.78 s | **0.66 s** |
| 30 | 196 | **84** | 12 ms | 10.16 s | 10.17 s | **4.35 s** |

**Two findings, and the first one refutes what I expected.**

1. **On this family our own phase-polynomial pass already matches quizx's
   full ZX rewriting on T-count, exactly** — 16, 40, 64, 84 from both,
   every rung. The composed pipeline is nonetheless **2–2.6× SLOWER**,
   because quizx's circuit EXTRACTION emits a larger circuit (154–2164
   gates) than our simplified form, and the evaluator pays for gates. The
   canonicalizer is not the bottleneck (5–12 ms); extraction's gate
   inflation is.

2. **Entry sixteen still stands and is the real dividing line.** On RANDOM
   circuits quizx cuts 63–79% where we cut 2–8%; on this STRUCTURED family
   we tie. So the honest statement is family-dependent, and neither
   "quizx dominates" nor "our passes suffice" survives alone: **ZX wins
   where Hadamards fragment the blocks; phase-polynomial normalization ties
   where long CNOT+diagonal blocks exist.**

Consequence for the tuner, and it is the tuner's own discipline: the
canonicalizer is NOT an unconditional win, so it must be a MEASURED route
like every other — `Unswept::ComposedCanonicalizer` is the honest state
until a sweep says which family gets which pass. The q=40 row (where the
engine alone times out past 900 s) is still running and will be appended
either way.

## 2026-08-28, twenty-second entry: the NATIVE ZX canonical form — the tier-2 gap erased where it was measured

*By the zx-native opus track; verified in the lead session (7 new exactness
tests green, 180 total; examples re-run here). Committed with this entry.*

**The original defect, diagnosed exactly** (correcting entry seventeen's
location): gadgets are not manufactured — they are the RESIDUE of
`gen_pivot`, which unfuses a non-Pauli phase onto a pendant spider AND
THEN pivots the host vertex away. A gadgetizer that does not pivot the
host out of the graph produces nothing, which is exactly what entry
seventeen measured.

**What ships**: a certified **T-count oracle** and a certified **exact
scalar** — the reduced diagram's value is carried in the engine's own
`Cyc` ring (the √2 bookkeeping falls out of the bit-packed sweep as a
popcount), and for plugged diagrams `eval` before reduction, after
reduction, and `run::amplitude` agree EXACTLY as ring elements. NO
circuit extractor yet — stated plainly; the reduced diagram cannot hand a
shorter circuit to the runner.

**T-count oracle, honest table** (raw → local → phase-poly → native ZX
vs quizx's full_simp):

| file | raw | ours-ZX | quizx |
|---|---:|---:|---:|
| q20 d200 | 25 | **15** | 0 |
| q20 d400 | 58 | **40** | 19 |
| q30 d400 | 67 | **51** | 26 |
| q40 d600 | 90 | **44** | 19 |
| q50 d800 | 111 | **71** | 41 |

Gadget fusion is alive (36–51% reduction, from 0%), and the remaining
distance to quizx's 63–79% is the named residue — the oracle does NOT yet
meet the quizx-agreement gate, and tier 2's T-count leg stays honestly
second.

**The headline is the EVALUATOR, and it closes the ledger's worst loss.**
Entry nineteen recorded the hidden-shift ladder at ≥42,000×: engine
TIMEOUT >900 s at q=40 vs quizx 21.5 ms. The native diagram evaluator
(plug inputs/outputs, full_reduce, read the exact scalar) computes
⟨shift|C|0⟩ = 1 EXACTLY at every rung of the ladder — **q=40 in 0.03 s**
— with the flipped-bit control reading exactly 0. Same task, same
circuits, exact ring arithmetic end to end. For strong simulation
(amplitudes), the four-order gap on the family we lost on is GONE; what
remains behind quizx is T-count minimization for circuit handoff, which
is the extractor's job and is named as owed.

## 2026-08-28, twenty-third entry: the extractor lands — tier 2's capability set is complete, and the residual gap is located

*By the zx-native opus track; verified here (full test suite green;
extractor examples re-run).*

**Extraction, certified at the UNITARY level** — stronger than amplitude
sampling: the composite C·extracted† is verified to be the IDENTITY (with
the global phase tracked through the certified scalar) on all 12 benchmark
circuits and a 252-circuit random sweep to n=50, depth=1250. Zero wrong,
zero refused; extraction cost 0.1–3.6 ms per circuit.

**The gap's location, settled by measurement**: extracted T-count equals
the oracle's exactly (extraction creates no T's), re-simplifying the
extracted circuit finds nothing (`round2` stable), and **quizx extracting
OUR reduced diagrams returns the same counts** (`q-extract` column) — so
the remaining distance to quizx's 0/19/26/19/41 lives entirely in
REDUCTION DEPTH (their full_simp's teleportation-era interplay reaches
lower T before extraction), not in extraction. Ours: 15/40/51/44/71.

**Tier-2 status after this entry**: the magic tier now EVALUATES exactly
(entry twenty-two: the q=40 timeout became 0.03 s) and EXTRACTS exactly
(this entry) — the capability set is complete; what remains against quizx
is a reduction-depth performance delta with a named location, not a
missing capability.

## 2026-08-30, twenty-fourth entry: the Clifford tier's reach, measured — and the adaptive path found stranded 343× behind it

*By the bigqvm-demo lane. Probe: `engine/crates/holon/examples/bigscale.rs`.
Reproduce: `cargo run --release --example bigscale -- 1024 4096 8192 16384
32768 65536 131072`. Every allocation is guarded against `MemAvailable`
with a 2 GB reserve and refuses loudly rather than OOM-killing a sibling —
this box is shared.*

**STAKED BEFORE MEASURING:** (1) the bake-off's n ≤ 4096 ceiling is a
harness limit, not an engine limit, and the engine reaches the memory
ceiling this box can hold; (2) the dense working set is `(2n)²` bits and
n ≈ 100k–150k is the ceiling at 31 GB RAM. Both stand. A third thing was
NOT staked and is the entry's real content.

**Scaling to n = 131072 — nothing breaks above 4096.** No `u32`/`usize`
indexing defect, no accidental O(n²) temporary, and the memory model is
exact: **8.590 GB predicted, 8.617 GB measured peak RSS.**

| n | column engine | row-major reference | row/col | tableau | peak RSS |
|---:|---:|---:|---:|---:|---:|
| 1024 | 0.035 µs/gate | 5.07 µs/gate | 145× | 0.001 GB | 0.003 GB |
| 4096 | 0.127 | 32.42 | 255× | 0.008 | 0.011 |
| 8192 | 0.617 | 187.5 | 304× | 0.034 | 0.037 |
| 16384 | 0.761 | 368.0 | 484× | 0.134 | 0.171 |
| 32768 | 4.126 | 1212.8 | 294× | 0.537 | 0.545 |
| 65536 | 9.611 | 3333.5 | 347× | 2.147 | 2.163 |
| 131072 | 21.083 | 7230.9 | 343× | 8.590 | 8.617 |

**THE FINDING, which was not the question asked: `adaptive.rs` runs on
`PackedTableau` — the row-major reference — and NOT on `coltableau.rs`, the
transposed engine that won entries six/eight/ten.** Mid-circuit measurement
was built against the certified reference (correctly — that is where the
rowsum path lives) and never moved. The cost of that is the table's third
column: 145× at n=1024, **343× at n=131072**, the gap widening with n
because row-major pays one cache miss per row for a one-bit-per-row update
while the column engine pays `2n/64` contiguous words.

The adaptive measurement path at the same sizes, on a short-range-entangled
state (what QEC actually produces):

| n | peek, random exit | collapse | peek, deterministic |
|---:|---:|---:|---:|
| 65536 | 0.003 ms | 1.141 ms | 1.064 ms |
| 131072 | 0.007 ms | 7.822 ms | 3.751 ms |

**What it costs the flagship, priced against stim rather than asserted.**
stim 1.16.0 installed locally (it was CI-only before) and measured on the
commissioned workload itself — `stim.Circuit.generated("surface_code:
rotated_memory_z")`, 3 rounds, `TableauSimulator`, full exact adaptive
semantics:

| d | n | measurements | stim | peak RSS |
|---:|---:|---:|---:|---:|
| 141 | 40184 | 79521 | 7.26 s | 1.03 GB |
| 181 | 66064 | 131041 | 26.87 s | 2.69 GB |
| 221 | 98344 | 195361 | **65.63 s** | 5.87 GB |

So the flagship is reachable — stim reaches it — and the honest bar is
65.6 s at d=221. Our row-major adaptive path projected on that same
workload is ~750k gate applications at ~5.3 ms each: **about 46 days.**
That is not a tuning gap, it is a layout gap, and it is the whole reason
the next entry exists. Recorded here first, with the engine unchanged, so
the finding has its evidence before its fix.


## 2026-08-30 — SATURATION-3 G2: the FCI sigma kernel at the (O,O,O) scale, GPU vs CPU

The dominant kernel of an all-electron STO-3G FCI table point: the sparse
Hamiltonian matvec at 207,025 determinants (15 orbitals, 24 electrons, 455×455
alpha/beta strings), f64. Measured on the REAL problem — index structures and
integrals exported from `pair::geometry_problem`, and the GPU answer checked
against `fci::sigma_direct`'s own output rather than against a re-derivation.

| arm | sigma/s | GFLOP/s FP64 | ratio |
|---|---:|---:|---:|
| **GPU, RTX 4090 Laptop (sm_89)** | **65.7** | 318.4 | **3.2× the CPU** |
| GPU incl. host round trip | 69.8 | — | PCIe is 0.5 ms against 15 ms of compute |
| CPU, `sigma_direct`, 32 threads | 20.8 | ~97 | the baseline (loadavg 32) |
| CPU, `sigma_direct`, 32 threads | 17.2 | ~80 | same, loadavg 18 — the spread is the machine |

**CORE PLACEMENT, declared 2026-08-31 (M-PLACEMENT-LOTTERY, M-IDLE-CALIBRATED-TIMEOUT).**
This host is an i9-13900HX hybrid part. The CPU arm above ran **32 unpinned threads spanning
both P and E cores** — a specific configuration, not a neutral one, and the E/P penalty is
measured at 1.16–1.22× and is duration-independent. So "measured on this box" is not a
specification: a citable ratio is a function of core class, exactly as a bit-gated artifact's
trailing bits are a function of device class.

The 3.2× ratio has margin enough that a ~20% placement effect cannot flip it, so this is an
annotation and not a retraction. Any repeat should be **pinned AND quiet, with both core types
reported** — bigqvm's standard, which their own d=101 verdict flip (0.822 unpinned "we lead"
to 1.201 pinned "they lead 20%") is the case for.
| CPU, same GEMM reformulation, OpenBLAS, 1 thread | 1.40 | 6.8 | **slower than the hand-written kernel** |
| CPU, `sigma_direct`, 1 thread | 2.20 | 10.7 | — |

Device ceilings measured on the same card, for reading the above: FP64 FMA
**416.1 GFLOP/s**, FP64 DAXPY **462.7 GB/s** (38.6 GFLOP/s, memory-bound), PCIe
round trip for one 1.58 MiB CI vector **0.50 ms**. Consumer Ada runs FP64 at 1/64
of FP32, so 318 GFLOP/s is 77% of this card's FP64 FMA peak.

**The baseline was checked, not assumed.** The GPU arm gets a reformulation
(sigma as three GEMMs) *and* a tuned library, so quoting it against a hand-written
loop would attribute the whole difference to the device — `holon-gpu/src/cpu.rs`'s
own warning. Running the identical reformulation on the CPU through OpenBLAS is
**slower** than `sigma_direct`: materialising the intermediate `T` costs 372 MB of
bandwidth per sigma, which the GPU has and the CPU does not. `sigma_direct` is a
good cache-blocked algorithm and is the honest CPU arm, so the 3.2× stands.

**Determinism, which is an adoption condition and not a nicety.** There is not one
atomic in the kernel: both scatters invert into gathers, because the excitation
maps are invertible (`a⁺_p a_q |jb⟩ = s|ib⟩` inverts, so the source string is
unique). Every sum is over a fixed range in a fixed order. Five repeat runs
**bit-identical**.

**Agreement, and why the kernel is measured but NOT adopted.** Against
`sigma_direct`: max absolute difference 4.547e−13, **3.033e−15 relative** to the
scale of |sigma| — and **188,363 of 207,025 entries (91.0%) differ BITWISE**. Both
are correct answers; they are not the same bits. So a GPU-built table is a
different artifact from a CPU-built one, the two can never be mixed inside one
table, and adopting the GPU idles 32 cores rather than adding to them. Adoption is
deferred until P2's fence counter rules the `(O,O,O)` table necessary; see
`SATURATION3_RESULTS.md` G2 and `engine/RESOURCE_DESIGN.md` D0, where this
measurement becomes the general rule that device class belongs to the artifact
rather than to the schedule.

Instruments: `holon-chem/examples/s3_sigma_cost.rs` (CPU),
`holon-chem/examples/s3_sigma_export.rs` (the real problem),
`scratchpad/s3gpu/{probe,sigma,cpu_fair}.{cu,py}` (device ceilings, GPU kernel,
fair CPU arm).

## 2026-08-30, twenty-fifth entry: the adaptive port lands — 30× per measurement, a 65,521-qubit surface code verified, and stim still ahead on the total

*By the bigqvm-demo lane. Engine: `engine/crates/holon/src/coladaptive.rs`.
Demo: `cargo run --release --example surface_flagship -- --d 181 --seed 1`.
Head-to-head: `conformance/qasm/surface_h2h.py`. Every run guards its
allocation against `MemAvailable` and refuses rather than OOM a sibling.*

**STAKED BEFORE MEASURING:** (1) moving the adaptive path onto the column
engine closes most of entry twenty-four's 343× layout gap; (2) a rotated
surface code at d ≥ 141 executes full adaptive syndrome-extraction cycles
with every verification passing; (3) against stim on the identical circuit we
land within a small factor either way, and whichever way it lands gets banked.
All three stand. (3) landed on stim's side.

### The port, measured on the same workload and the same box

| quantity, d=101 | before | after | |
|---|---:|---:|---|
| steady-state round | 0.834 s | **0.065 s** | 13× |
| one measurement in a steady-state batch | 47.19 µs | **1.55 µs** | 30× |
| fallback (row-major) scans in a 4-round run | 10199 | **0** | — |
| whole 4-round QEC demo | 10.6 s | **4.3 s** | 2.5× |

Three things did it, and only the first was designed — the other two were
forced by `examples/measure_profile.rs` after the cost model and the clock
disagreed by 16×:

1. **The determinism scan moves column-side.** "Does any stabilizer
   anticommute with Z_q?" costs the row-major reference `n` scalar bit-gets
   across `2n` separate allocations; it is `⌈2n/64⌉` contiguous words here.
2. **The single-term shortcut.** The destabilizer product has |H| = 1 in the
   steady state (mean 1.85 over a whole run, max 5, measured), and a product
   of one row is that row's SIGN BIT. 71.4% of deterministic measurements —
   at every d — read one bit and touch no row at all. So the row-major
   reference is materialized LAZILY and a clean round transposes zero times.
   The profile that forced this: scan 0.90 µs, hit set 0.97 µs, product
   2.90 µs — against 47.19 µs for the engine call. The gap was never the
   measurement; it was the 0.6 s per-round transpose the measurement was
   dragging behind it.
3. **The mirror patch.** A collapse used to invalidate the column mirror and
   send the rest of the batch back to row-major scanning. The pivot's X-weight
   is 1.5 mean / **2 max, independent of d**, so the mirror is PATCHED
   through the collapse for a couple of column XORs instead. Fallback scans
   went 10199 → 0.

Also: the transpose loop nests are now blocked (TILE = 8), worth ~1.7× on the
inverse direction; the unblocked gather used 8 bytes of every 64-byte line it
pulled and dropped the line before the next iteration wanted the next word.

### The flagship: full adaptive syndrome extraction, verified

Rotated surface code, four-step conflict-free schedule, real mid-circuit
measurement, injected X errors, a decoder reading the mid-circuit outcomes,
feed-forward correction, seeded and replayable:

| d | n | data + ancilla | measurements | wall | peak RSS | verifications |
|---:|---:|---|---:|---:|---:|---|
| 141 | 39761 | 19881 + 19880 | 79520 | 25.8 s | 2.39 GB | **7/7 PASS** |
| 181 | **65521** | 32761 + 32760 | **131040** | 49.5 s | 6.47 GB | **7/7 PASS** |

The seven are not restatements of one check: Z syndromes silent on |0…0⟩;
the logical Z determined and +1; a noiseless round reproducing the previous
one EXACTLY; injected errors lighting exactly the plaquettes that contain
them and no others; the decoder explaining every fired plaquette with none
left over; syndromes returning to their pre-error values after correction;
and the LOGICAL observable still determined and unchanged — the difference
between "the syndromes look right" and "the encoded bit survived".

### Against stim, on the identical circuit — WE LOSE, and where is located

`--stim` emits the same circuit the engine runs, so this compares engines and
not circuit generators. Both arms do full exact adaptive simulation (state
collapsing, not Pauli-frame sampling). Minimum of 7, arms interleaved:

| d | n | measurements | ours | stim | ours/stim |
|---:|---:|---:|---:|---:|---:|
| 21 | 881 | 1320 | 0.002 s | 0.002 s | 0.766 |
| 45 | 4049 | 6072 | 0.051 s | 0.024 s | 2.093 |
| 101 | 20401 | 30600 | 2.211 s | 1.549 s | 1.427 |
| 141 | 39761 | 59640 | 10.191 s | 5.243 s | 1.944 |

**stim leads at 3 of 4 sizes, 1.4–2.1×.** Two disclosures that matter more
than the numbers. First, an earlier median-of-3 pass reported us AHEAD at
d=101 and d=181; taking the minimum instead — the right estimator under
contention, since interference can only add time — reversed it. The
flattering number was noise. Second, this table predates the transpose
blocking and so understates the current engine; the re-run is owed and queued
(`conformance/bigqvm/RESUME.md`).

**Where it goes**, by differencing round counts at d=101 (minimum of 5):

| | fixed cost | marginal per round |
|---|---:|---:|
| ours | 1.054 s | 0.328 s |
| stim | 1.167 s | 0.119 s |

Our fixed cost is slightly BETTER and our marginal round is ~2.75× worse —
and a clean round is 0.065 s, five times cheaper than that average. The
average is dragged up by the rounds that are not clean: a random outcome or a
multi-term product forces the O(n²/64) transpose. Checked rather than
assumed: at d=101 there are ~2550 multi-term measurements per round, and
serving those by extracting rows from the column engine instead would cost
~8 s against the transpose's ~0.35 s, so the current policy is right and the
remaining gap is the transpose's own constant. Closing it needs a faster
transpose or a measurement layout that needs none — named, not attempted.

### Honest limits on everything above

**d = 221 (n = 97681), the commissioned headline, IS NOT IN THIS ENTRY.** Its
working set is 9.54 GB and this box's MemAvailable swung between 2.9 GB and
20 GB during the session as siblings grew; the run has never had its window.
A detached waiter is polling for one (`conformance/bigqvm/run_when_memory.sh`)
and will write `d221.DONE` only on a real success. The largest VERIFIED run
is d = 181 at 65,521 qubits, and that is what is claimed.

**Machine caveat, standing:** every number here was taken at load 33–54 with
siblings competing for memory and CPU. Repetitions of the SAME measurement
spread 2–4× at the large distances. None of these ratios is a quiet-machine
number, and the banked bake-offs (entries six/eight/ten) used a quiet CI
runner — this comparison deserves the same before it is quoted.


### Twenty-fifth entry, UPDATE (same day): the head-to-head re-run — the gap was mine, and it mostly closed

The re-run owed above has run, with the transpose blocking in (min of 5, arms
interleaved, identical circuits, same loaded box at load 42–49):

| d | n | measurements | ours | stim | ours/stim | previous |
|---:|---:|---:|---:|---:|---:|---:|
| 21 | 881 | 1320 | 0.003 s | 0.002 s | 1.130 | 0.766 |
| 45 | 4049 | 6072 | 0.052 s | 0.041 s | 1.264 | 2.093 |
| 101 | 20401 | 30600 | 1.319 s | 1.605 s | **0.822** | 1.427 |
| 141 | 39761 | 59640 | 8.264 s | 7.174 s | 1.152 | 1.944 |

**The honest verdict moves from "stim leads 1.4–2.1×" to "parity, 0.82–1.26×,
with one size where we lead" — and it is still not a clean win for anyone.**
The 1.4–2.1× banked an hour earlier was measuring an engine that no longer
exists; the table above is the current one and the earlier table stays put,
marked, because a record is a history and the warrant for the blocking is in
the numbers it beat.

**Why these minima are usable where the earlier medians were not:** the
per-repetition distributions are bimodal — clean runs cluster tightly at the
bottom, then contention outliers. At d=101 ours reads 1.319, 1.349, 1.908,
2.039, 8.457 and stim 1.605, 1.615, 1.623, 4.015, 4.398: both minima are
confirmed by a near-identical second sample (2% and 0.6% apart), so those two
numbers are well determined even though the medians are not. At d=141 ours is
weaker — min 8.264 with the next at 10.556, 28% away — so that row's ratio is
the softest in the table and is not the one to quote.

No claim of a trend across d is made: four points, three of them with a
neighbour more than 20% away, on a box at load 42–49. The defensible statement
is parity at these sizes, the quiet-runner measurement still owed.


### Twenty-fifth entry, seed sweep: "every seed" checked rather than assumed

The commission's correctness bar is "the code returns to the codespace, every
seed", and one seed at scale does not establish that — the random half of the
outcomes is exactly what a seed changes, so a sign or ordering defect could
hide in the seeds not run.

| d | seeds | result |
|---:|---|---|
| 45 | 1–12 (12 seeds × 7 checks) | **84/84 PASS, 0 FAIL** |
| 141 | 1, 2, 3, 4 | **28/28 PASS, 0 FAIL** |
| **221** | 1, 2, 3 | **21/21 PASS, 0 FAIL** (added after the memory fix; peak RSS 9.571 GB on every seed, wall 74.0 / 74.3 / 99.6 s) |

Every one of the seven verifications holds on every seed at both distances,
including the two that depend on the coin outcomes: the noiseless round
reproducing the previous one exactly (which fixes the round-1 random
X-syndromes and requires every later round to agree with them), and the
logical observable staying determined and unchanged.


## 2026-08-30, twenty-sixth entry: d = 221 — 97,681 qubits, 195,360 adaptive measurements, verified

The commissioned headline, run and verified. `conformance/bigqvm/
flagship_d221.json`; reproduce with `cargo run --release --example
surface_flagship -- --d 221 --seed 1`.

| | |
|---|---|
| distance | **221** |
| qubits | **97,681** (48,841 data + 48,840 ancilla) |
| stabilizers | 48,840 |
| rounds | 4 full adaptive syndrome-extraction cycles |
| mid-circuit measurements | **195,360** |
| wall (rounds) | **111.25 s** |
| verifications | **7 / 7 PASS** |

Per round: establish 36.39 s, noiseless repeat 2.13 s, errors injected
12.94 s, corrected 4.17 s. Errors were injected on three data qubits, lit
exactly their plaquettes and no others, the decoder read the mid-circuit
outcomes and explained every fired plaquette with none left over, the
correction was fed forward, the syndromes returned to their pre-error values,
and the LOGICAL observable came through determined and unchanged.

**The design constants held at full scale**, which is the part worth keeping:
destabilizer product mean **1.8533** terms and pivot X-weight **max 2** at
d=221 — the same numbers measured at d=11, d=45 and d=101. Fallback scans:
**0 of 195,360**. Transposes: **2** for the entire run. The two shortcuts the
profiler forced were structural, not small-n luck.

**A DEFECT THE RUN EXPOSED, recorded because the guard is a safety claim.**
Peak RSS was **14.36 GB against the 9.54 GB working-set model the memory guard
enforces** — the guard UNDERSTATES, so it could admit a run that then OOMs a
sibling, which is the one failure mode this lane was told to avoid. Cause:
`z_string_value`, the logical-observable reader, called `col.to_packed()` and
allocated a SECOND full row-major tableau beside the lazily-held one — 4.77 GB
extra, twice per demo. Fixed by reusing the single buffer and answering the
common case with none at all (the anticommutation test is a XOR of the
string's X columns, so "is this observable even determined?" costs
`|string|·2n/64` words and no allocation). The 111.25 s and 7/7 above are the
run AS IT HAPPENED and are not restated; the memory figure is the one the fix
moves.

**Also measured, and it is not simulation:** `SurfaceCode::new` spends
**61.8 s** of startup in `verify_commuting`, which is O(stabilizers²) —
1.19e9 pairs at this distance. That is the price of checking the layout
rather than trusting it, and it is worth paying; but it re-runs on every
invocation and at d=221 it exceeds half the wall time, which is why the
head-to-head at this size is slow. A spatial index makes it linear. Named.

**Scheduling, since it is part of the result:** the run needed 11.5 GB free
and the box offered 3.4–6.4 GB for twenty-five minutes. The detached waiter
(`conformance/bigqvm/run_when_memory.sh`) was refused six times, then took a
17.9 GB window unattended. No sibling was disturbed, and no done-marker was
written for a refused attempt — the binary's exit code is the only authority
there, after a first version of the waiter lost that race in shell.


### Twenty-sixth entry, the head-to-head at d = 221: WE LEAD 1.33×, and the ranges do not overlap

Same discipline as the rest: identical circuit (ours, emitted to stim format),
engine-only timing both sides, full exact adaptive simulation on both, arms
interleaved, 3 rounds, n = 97,681, 146,520 mid-circuit measurements.

| | run 1 | run 2 | run 3 | min | median |
|---|---:|---:|---:|---:|---:|
| **ours** | 59.90 s | 63.23 s | 68.99 s | **59.90** | 63.23 |
| **stim** | 79.42 s | 83.72 s | 102.11 s | 79.42 | 83.72 |

**ours/stim = 0.754 on minima, 0.755 on medians — the two estimators agree to
one part in a thousand, and OUR SLOWEST RUN (68.99 s) BEAT STIM'S FASTEST
(79.42 s).** The distributions are disjoint, so no choice of estimator can
flip this one, which is exactly what could not be said of the smaller sizes.
Taken on the same contended box as everything else (load 44–46); the runs are
long enough here that contention averages out instead of dominating, and the
spreads show it — 1.15× ours, 1.29× stim, against 2–6× at d ≤ 141.

The full picture across distance, minima, current engine:

| d | n | ours/stim | |
|---:|---:|---:|---|
| 21 | 881 | 1.130 | stim leads |
| 45 | 4049 | 1.264 | stim leads |
| 101 | 20401 | 0.822 | we lead |
| 141 | 39761 | 1.152 | stim leads |
| **221** | **97681** | **0.754** | **we lead, and it is the reliable one** |

**The honest reading: stim is ahead at the small sizes, we are ahead at the
flagship size, and the middle is inside the noise of this box.** The d=221 row
is the only one whose two estimators agree and whose distributions separate,
so it is the one to quote; the small-d rows are stable but tiny (milliseconds,
where our per-gate constant shows); d=101 and d=141 disagree in direction and
neither should be cited alone. No smooth trend is claimed from five points.

**And the memory diagnosis is confirmed by this run rather than argued:** peak
RSS in `--mode bench` was **9.57 GB against the 9.54 GB model** — the guard is
accurate when `z_string_value` is not called. The 14.36 GB seen in the QEC
demo was that function's second tableau, exactly as the twenty-sixth entry
above says, and the fix targets the right thing.


### Twenty-sixth entry, the memory fix verified: the guard is honest again

`z_string_value` rewritten to reuse the single reference buffer, and to answer
the undetermined case with no buffer at all (the anticommutation test is a XOR
of the string's X columns). Re-run at d=221, same seed, same box:

| | before | after | model |
|---|---:|---:|---:|
| peak RSS | 14.358 GB | **9.571 GB** | 9.545 GB |
| wall | 111.25 s | **73.98 s** | — |
| verifications | 7/7 | 7/7 | — |

**The guard now predicts peak RSS to 0.3%**, which is the property that
matters: a memory guard that understates is a licence to OOM a sibling, and
this one was understating by 50%. The 33% wall-time drop is a side effect —
the second tableau was also being TRANSPOSED into, twice per demo.

Gated by a new conformance test that runs the column-side path and the
row-major reference implementation (`z_string_value_of`) on random states and
random strings and requires agreement, including on the `None` verdict — which
is now decided without materializing a tableau and so has no shared code left
to protect it. Both branches asserted non-vacuous. 12 gates in the file, 101
lib tests, both profiles.


### Twenty-sixth entry, CORRECTION: core placement is a third axis, and it flips one banked row

*Prompted by counter-evidence from the mesh lane, verified here against this
box's own hardware. Their datum: a 26.7 ms job of theirs reads 3.5× its record
at loadavg 46 — SHORTER than my calibration probe and far MORE sensitive,
which falsifies the mechanism I had proposed ("a short job gets an idle core").*

**The box is a hybrid part and I never controlled for it.** 13th Gen Intel
i9-13900HX: P-cores are CPUs 0–15 (8 physical, SMT), E-cores are 16–31, and
`lscpu` reports scaling at 57% of a 5400 MHz max. Every ratio banked above was
taken with the scheduler free to place either arm on either core type, run to
run. That is not a controlled comparison, and it is not fixed by waiting for a
quiet machine.

**Both arms pinned to the SAME core, 3 rounds, minima:**

| d | placement | ours | stim | ours/stim | banked (unpinned) |
|---:|---|---:|---:|---:|---:|
| 101 | P-core | 1.452 s | 1.209 s | **1.201** | 0.822 |
| 101 | E-core | 1.776 s | 1.795 s | **0.989** | 0.822 |
| 221 | P-core | 30.24 s | 33.77 s | **0.895** | 0.754 |
| 221 | E-core | 47.29 s | 65.38 s | **0.723** | 0.754 |

**RETRACTED: "d=101, we lead 0.822".** On a P-core the same comparison reads
1.201 — stim ahead by 20%. The direction of that row depends on where the
scheduler put the two arms, so it was never a result. It is withdrawn, not
re-stated with a caveat.

**STANDS, with its margin corrected: d=221.** We lead on every placement
tested — 0.895 on a P-core, 0.723 on an E-core, 0.754 unpinned. The direction
is robust; the MARGIN is not, and the honest headline number is the
adversarial one, **0.895 (we lead ~12%)**, not the 0.754 banked above. The
"our slowest beat stim's fastest" framing stands only for the unpinned
sampling it described, and that sampling is now known to be confounded, so it
should not be quoted as the strength of the result.

**And the estimator I argued for was wrong too.** I banked "the minimum is the
robust estimator under contention, since interference can only add time." On a
hybrid part it recovers the luckiest PLACEMENT, not the uncontended time — and
the demonstration is direct: d=141's banked "record" of 8.264 s (min of 5,
unpinned) is beaten by **6.274 s pinned to a P-core at loadavg 44.6**. A
minimum taken over a placement lottery is not a floor.

**Pinning beats waiting.** Pinned repetition spreads are 1.1–1.7% against
15–29% unpinned, and at d=221 pinning roughly HALVED both arms (ours 59.90 →
30.24 s, stim 79.42 → 33.77 s), so the unpinned runs were losing more to
migration and placement than to the neighbours. **A quiet window fixes
contention; it does not fix heterogeneity.** The citable table therefore needs
`taskset`, not just an idle box — the waiter's quiet gate is necessary and was
never sufficient.

**My proposed mechanism is withdrawn.** I claimed a short probe cannot see
load because it gets an idle core. Three things say otherwise: the mesh lane's
26.7 ms job is shorter and 3.5× more affected; the E-core penalty on MY
workload is duration-INDEPENDENT (1.16× at d=45, 1.22× at d=101, 1.16× at
d=141), so duration is not the axis it varies along; and the probe's CPU time
tracks its wall time (user 0.07 of wall 0.07–0.11), so it was never waiting
for a core in the first place. The surviving statement is the mesh lane's:
**a probe is blind to contention it does not itself experience** — match the
probe to the measurement in working set and core placement, not merely in
duration. Lengthening my probe worked here only because d=101's 416 MB working
set competes for bandwidth that d=45's 16.5 MB does not.


### Twenty-sixth entry, SECOND CORRECTION: "pin to a P-core" is not a condition, it is a lottery

*Prompted by saturation3-mesh retracting a clause of theirs I had accepted (their
"slower because it landed on an E-core" is falsified in sign on their own
workload: E/P = 0.83, the E-core was FASTER for them). Their retraction sent me
to check whether my own "adversarial P-core" framing was any better founded. It
was not.*

**Within-P spread is larger than the P-vs-E gap, and it is not stable in time.**
Same workload (d=101), same minutes, single-CPU pins:

| | cpu0 (P) | cpu2 (P) | cpu6 (P) | cpu16 (E) | cpu20 (E) | cpu24 (E) |
|---|---:|---:|---:|---:|---:|---:|
| sample 1 | **1.121** | 1.535 | 1.527 | 1.786 | 1.727 | 1.766 |
| sample 2 | 1.576 | 1.459 | **1.119** | — | — | — |

The fastest P-core in sample 1 is the slowest in sample 2, minutes later. The
cause is visible in `/proc/stat`: at sample time cpu0 was 17.1% busy while its
SMT sibling cpu1 was **85.4%**, and cpu2 was 2.7% busy while cpu3 was **100%**.
A P-core's throughput is a function of its sibling's load, which no lane
controls and which changes minute to minute. E-cores on this part have **no SMT
sibling** (`thread_siblings_list` for cpu16 is just `16`) and are correspondingly
stable: 1.727–1.786 across three different E-cores, and cpu20 reads 1.727 /
1.776 / 1.776 across three separate sessions — **1.03× spread against P's
1.41×**.

**So the framing banked in the first correction is withdrawn.** "The honest
headline is the adversarial P-core number, 0.895" assumed "P-core" names a
reproducible condition. It does not. The defensible statement at d=221 is a
RANGE over measured placements — **0.72–0.90, direction consistent (we lead in
all three)** — with each individual number confounded by a sibling state nobody
recorded. Three conditions all favouring us is real evidence for the DIRECTION;
none of the three is a reproducible number.

**And `taskset` restricts, it does not reserve.** I labelled a `taskset -c 0,1`
run "whole physical core, both siblings reserved". That is wrong: taskset
constrains OUR process and does nothing to keep other processes off those CPUs.
Those runs (1.094–1.575 across three physical cores) are not isolation and
should not be read as such. Real isolation needs `isolcpus` or a cpuset cgroup,
neither of which this lane can set up on a shared box.

**The actionable consequence, and it is counterintuitive: for a reproducible
timing on this part, pin to an E-CORE, not a P-core.** E-cores are slower and
have no SMT lottery, so they give a condition that repeats; P-cores give a
faster number that does not. A citable table wants repeatability more than it
wants the best clock.

**What still stands from the first correction:** the d=101 retraction (now
firmer — the verdict flips across placements AND the placements themselves are
not stable), the d=221 direction, and the estimator lesson, which this
strengthens: the minimum over repetitions is a lottery over placement *and*
sibling state, not a floor.


## 2026-08-31, twenty-seventh entry: THE CITABLE TABLE — quiet machine, pinned, both core types

The waiter took its window unattended: **loadavg 6.4 → 6.8, and it held across
both sweeps** (the run refuses to label itself quiet otherwise). Both arms
pinned to the same CPU, identical circuits, 3 rounds, 5 reps, minima with
medians shown for agreement. `conformance/bigqvm/h2h_quiet_{pcore,ecore}.json`.

| d | n | P-core ours/stim | E-core ours/stim | |
|---:|---:|---:|---:|---|
| 21 | 881 | 1.375 | 1.324 | stim leads (see caveat) |
| 45 | 4049 | 1.458 | 1.488 | **stim leads clearly** |
| 101 | 20401 | 0.991 | 0.942 | parity |
| 141 | 39761 | 1.238 | 0.956 | **placements disagree** |
| 221 | 97681 | **1.005** | **0.866** | parity to we-lead-15% |

Spreads 1.01–1.22×, min and median agreeing to ~1% on every row except d=21,
where both arms run in ~1–2 ms and timer resolution dominates (min ratio 1.375
against a median of 2.138) — **that row should not be quoted at all.**

**THE HONEST VERDICT, and it is weaker than everything this lane banked
before.** stim leads clearly at small n (1.46–1.49× at d=45). The arms cross
somewhere around n ≈ 20k. At the flagship size we are between **parity
(1.005 on a P-core) and a 15% lead (0.866 on an E-core)** — the direction at
d=221 is itself placement-dependent even on a quiet box, so the defensible
claim is "parity-to-modestly-ahead at the largest size, clearly behind at small
sizes, with the crossover near n ≈ 20k."

**Every contended measurement in this lane flattered us**, and the size of that
is worth recording: d=221 read 0.754 unpinned-and-loaded, 0.895 pinned-and-
loaded, and **1.005 pinned-and-quiet on the same core class.** Contention was
not neutral noise around a true value — it was a bias in our favour, because
the two engines lose different amounts to it. That is the strongest argument in
this lane for why a contended A/B is not a measurement at all, and it retires
the last of the earlier headlines: 1.33×, then 0.895, now parity-to-0.866.

Contention was also costing a factor of ~3.5 in absolute terms (d=221 P-core:
17.04 s quiet against 30.24 s pinned-but-loaded and 59.90 s unpinned-and-loaded).

**What survives from the whole comparison programme:** our margin GROWS with n
(clearly behind at d=45, parity at d=101, ahead on the reproducible core class
at d=221), which is the shape the column-major engine's word-parallel scaling
predicts, and which every earlier table also showed — the trend was real even
while its magnitudes were not.

