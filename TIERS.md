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
| 2 — magic (exact Z[ω] branch sums) | **BUILT** — measured exponent 0.500, orbit-bound dedup, exact sampling; **behind published SOTA and stated so** | frozen holon-qasm + qiskit/Aer | the published exact-exponent ladder: Bravyi–Gosset 2016 α≤0.463 → **Qassim–Pashayan–Gosset 2021 α≤0.3963** (exact; constructive; ~37× at t=50) → quizx cat-state decompositions (~0.25 effective on benchmarks, arXiv:2109.01076) — interface is rank-agnostic, paste and go | import QPG-2021 decompositions |
| 2.5 — exact shots | **BUILT** | brute-force overlaps + certified branch sums | Aer ext-stab shot throughput, at exactness Aer cannot match at any speed | O(branches²) → orbit-aware Gram |
| mesh (CPU shards / GPU / cluster) | **BUILT** intra-node (shard-invariant CPU; 4090 at 336–396×, struct-determinism) | the merge law's own tests | **near-linear to 1024 shards multi-node**; quiet-machine efficiency curves owed | inter-machine transport via the one transport square |
| bulk — MPS/DMRG | **SCAFFOLD** (MpsHolon shape; python DMRG upstream, ED-certified) | exact ED ≤ 20 sites; Schwinger closed forms | **ITensor/TeNPy** sweep-time parity on Schwinger-class Hamiltonians | port DMRG onto the holon object + merge law |
| crystal — gauge-coupled matter | **NAMED** — entry gated on SCHWINGER-2's verdict (running) | Schwinger 1962 exact values (M_V/g = 1/√π; condensate) | known continuum values, then 2-flavor spectra | SCHWINGER-2 verdict → port to bulk tier |
| physics/graphics — grain → cosmic | grandfathered engine (battery-at-touch) | the conformance battery per tier | **browser: 60 fps at 10⁶ grains in WASM** (the sandbox viewer builds and is gated on every commit; the hosted tab was retired from the thesis page with the spin-out — hosting it HERE is owed); native: Rapier/PhysX-class rigid-body throughput, measured not claimed | per-tier battery certification |
| deployment range | WASM **builds and is gated today** (hosted demo owed here since the thesis page's tab retired) → laptop GPU (4090, measured) → clusters (mesh law, multi-node owed) | determinism at every rung | identical certified results at every scale — the range IS the product | multi-node demo |

| front-end — the OpenQASM surface | **NAMED** — today: OpenQASM 2 subset (fixed Clifford+T+CCX gate enum, one q/c register, terminal measurement only; no parameterized gates, custom gate defs, reset, mid-circuit measurement, or classical conditionals — no adaptive circuits) | the spec + public corpora | **MQT Bench** (>70k circuits, OpenQASM 2/3) and the **ABSTRACTS** Clifford+T suite (arXiv:2608.24370) parse-and-run coverage | adaptive Clifford first (feed-forward stays inside the tableau tier) |
| open systems — noise and channels | **NAMED** — no density matrices, Kraus channels, or noise models anywhere yet | exact small-system channel truth; Aer noise simulations | match Aer's noise-model results exactly where the channel is Clifford-representable; refuse elsewhere | design: the ledger's channel form (mixed stabilizer / CH-form prior art first) |
| verification scope — the Lean layer | kernel theorems PROVED (closure walls, tier structure, one-rebit stabilizer kernel); **the running Rust engine is NOT verified** and the claim discipline forbids saying otherwise | Lean CI on the theorems; the engine's own referees for the engine | **VOQC/SQIR's extraction discipline** (arXiv:1912.02250) — a verified path from proof to running kernel; nearest structural precedent to cite: **Lean-QEC** (arXiv:2605.16523) | state-and-prove stabilizer closure ABOUT the tableau implementation (axis found vacant by the sweep) |

## The honest boundary — envelopes, caps, exemptions, refusals

*The goal is to find the EDGES of what the holon can do by robust simulation of
reality — so the edges are named, enforced, and part of the record, never
discovered by a reviewer first. A refusal is a result.*

| boundary | state | enforcement |
|---|---|---|
| **arithmetic envelope** | exactness holds while every Z[ω] coefficient fits i128; coefficients grow like 2^{O(n+t)} (Quist–Coopmans–Laarman, arXiv:2602.17775), so the envelope is reachable | **ENFORCED 2026-08-27**: `Cyc` add/mul/alignment REFUSE (panic) on overflow in all three rings — holon ledger, holon-qasm referee, GPU host ring (`envelope_tests` pin it); the GPU fold was already magnitude-pre-guarded. Next: per-run envelope line in the certificate; differential oracle vs Selinger's bignum `ZOmega` (newsynth) |
| **statevector cap** | reference tier refuses above N_MAX = 24 qubits by name | router assertion, `holon-qasm/src/lib.rs` |
| **router magic cap** | branch-sum route taken at t ≤ 12 (and no Toffoli) by default | router condition, `holon-qasm/src/lib.rs:185` |
| **sampler working scope** | t ≤ 8 comfortable, t ≈ 10 the edge (documented in-source) | `holon/src/sample.rs` header; O(branches²) Gram is the next lever |
| **adaptivity** | none — terminal measurement only; a mid-circuit measurement is REFUSED at parse, not approximated | parser; lifted by the front-end row's milestone |
| **CI exemptions** | DMRG (python, upstream referee), the Hubbard reference, and holon-gpu run outside the default gate set — allowlisted WITH owner and exit criteria, not hidden | `ci-gates.sh` CRATE_ALLOW |
| **claim discipline** (from the adversarial sweep) | the exact five-integer Z[ω] representation is **SliQSim's** (Tsai–Jiang–Jhang, DAC 2021) and **quizx** already pairs the ring with stabilizer decomposition (QST 2022) — we cite, we do not claim the ring; shard-invariant merging is a FREE CONSEQUENCE of exact arithmetic (exactness buys it; we never claim it as a technique — ReproBLAS solved the hard float version); certificates have ABFT ancestry. The surviving claim is the **conjunction** — certified exactness + refusal boundary + distribution + kernel theorems — plus the vacant grounds the sweep found (no machine-checked simulator kernel, no certificate-carrying distributed simulation standard) | `PRIOR_ART.md` (in progress — six-lane sweep) |
