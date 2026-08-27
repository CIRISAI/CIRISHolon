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
