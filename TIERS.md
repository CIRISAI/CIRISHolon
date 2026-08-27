# The tier ladder — scaffold, referees, and benchmark targets

*Every tier: its status, the referee that certifies it, the benchmark target
it is measured against, and the next milestone. A tier without a referee and
a target is not scaffolded — it's a wish. Statuses are honest: ASSUMED means
proven and no longer under test; BUILT means certified against its referee;
SCAFFOLD means the object shape exists and the target is named; NAMED means
only the plan exists.*

| tier | status | referee | benchmark target | next milestone |
|---|---|---|---|---|
| 0 — classical bit-planes | **ASSUMED** | exact statevector | word-parallel batching (64 shots per word op, free) | batch mode |
| 1 — stabilizer (packed Pauli planes) | **ASSUMED** | certified unpacked tableau + qiskit + stim | **stim ≤ 1×** (current: 2.7–5.6×; remainder is SIMD width + layout transpose, mechanical) | SIMD kernels |
| 2 — magic (exact Z[ω] branch sums) | **BUILT** — exponent 0.500, orbit-bound dedup, exact sampling | frozen holon-qasm + qiskit/Aer | **BG rank-7 table** (→ 0.468; interface is rank-agnostic, paste and go) then **quizx** on effective-T instances | rank-7 verification |
| 2.5 — exact shots | **BUILT** | brute-force overlaps + certified branch sums | Aer ext-stab shot throughput, at exactness Aer cannot match at any speed | O(branches²) → orbit-aware Gram |
| mesh (CPU shards / GPU / cluster) | **BUILT** intra-node (shard-invariant CPU; 4090 at 336–396×, struct-determinism) | the merge law's own tests | **near-linear to 1024 shards multi-node**; quiet-machine efficiency curves owed | inter-machine transport via the one transport square |
| bulk — MPS/DMRG | **SCAFFOLD** (MpsHolon shape; python DMRG upstream, ED-certified) | exact ED ≤ 20 sites; Schwinger closed forms | **ITensor/TeNPy** sweep-time parity on Schwinger-class Hamiltonians | port DMRG onto the holon object + merge law |
| crystal — gauge-coupled matter | **NAMED** — entry gated on SCHWINGER-2's verdict (running) | Schwinger 1962 exact values (M_V/g = 1/√π; condensate) | known continuum values, then 2-flavor spectra | SCHWINGER-2 verdict → port to bulk tier |
| physics/graphics — grain → cosmic | grandfathered engine (battery-at-touch) | the conformance battery per tier | **browser: 60 fps at 10⁶ grains in WASM** (sandbox wasm ships today); native: Rapier/PhysX-class rigid-body throughput, measured not claimed | per-tier battery certification |
| deployment range | WASM **ships today** (the sandbox tab) → laptop GPU (4090, measured) → clusters (mesh law, multi-node owed) | determinism at every rung | identical certified results at every scale — the range IS the product | multi-node demo |
