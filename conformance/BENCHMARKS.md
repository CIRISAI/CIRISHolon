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
