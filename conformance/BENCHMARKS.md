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
| hidden shift | n=40 t=14 | **0.086 s exact p=1.0** | Aer ext-stab 47.4 s, top outcome 1% (fails) | **550× faster AND exact where approximate sampling fails** |
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
