# The tier ladder — scaffold, referees, and benchmark targets

*Every tier: its status, the referee that certifies it, the benchmark target
it is measured against, and the next milestone. A tier without a referee and
a target is not scaffolded — it's a wish. Statuses are honest: ASSUMED means
proven and no longer under test; BUILT means certified against its referee;
SCAFFOLD means the object shape exists and the target is named; NAMED means
only the plan exists.*

| tier | status | referee | benchmark target | next milestone |
|---|---|---|---|---|
| 0 — classical bit-planes | **ASSUMED (core)** — NOT DONE until the front-end exceeds OpenQASM (see front-end row) | exact statevector | word-parallel batching (64 shots per word op, free) | batch mode |
| 1 — stabilizer (packed Pauli planes) | **ASSUMED (core)** — NOT DONE until the front-end exceeds OpenQASM (see front-end row) | certified unpacked tableau + qiskit + stim | **stim ≤ 1× — REACHED AT EVERY n** (quiet-runner bake-off, 7/7 sizes ahead: 0.69× at n=64 down to 0.437× at n=4096 — 1.4–2.3× faster, margin growing with n; `conformance/BENCHMARKS.md` entry six, reproducible via the `bakeoff` workflow). Stack: transposed flat column engine + fused AVX2/WASM-SIMD128/scalar rowsum kernel (bit-identical by gate) + one-pass canonical terminal sampler | roofline analysis: is the margin maximal? |
| 2 — magic (exact Z[ω] branch sums) | **BUILT, and the exponent moved**: Magic5FromCat LANDED (magic5.rs, exact-equality-gated against both prior paths and the frozen referee, planted defects caught) — realized α 0.4111 at t=28, 0.4027 at t=64 (74.8× fewer branches), asymptote 0.3963 never quoted as a measurement; AND branch-slicing LANDED (sliced.rs, 64 branches/word on a proved structural sharing theorem, bit-identical at every lane) — 14–26× where t ≤ n. **Positions against the lake's limits (Limits.lean): the slicing factor is an L2 (word width, BEGGING) position at 22–41% of the 64-wide bound, the residual being L4-flavored (exact γ coefficients are not bits — ring fraction 3–6% at n=64); the exponent is an L3 (BEGGING) position above the open floor.** Pruned×sliced are ALTERNATIVES (dedup destroys shared structure; measured both ways); magic5×sliced is UNSWEPT and the tuner refuses to guess | frozen holon-qasm + qiskit/Aer | the published exact-exponent ladder (numerically verified in `conformance/srank/`): Bravyi–Smith–Smolin 2016 6→7 α≤0.4679 (the simulator on it is Bravyi–Gosset 2016) → **Qassim–Pashayan–Gosset 2021 α≤0.3963** (Quantum 5, 606; explicit closed-form cat construction — realized α at t=64 is 0.4027, quote it with the finite-t caveat) → **Magic5FromCat** (Kissinger–van de Wetering–Vilmart, TQC 2022: 4-to-3 partial rule, 0.3963 CONCRETELY at finite t, Apache-2.0 Rust in quizx over our exact ring — 74.8× at t=64) → opportunistic cat₄/cat₆ 0.25–0.264 where circuit structure allows → T-count preprocessing (PyZX full_reduce) multiplicative on top. Trap defused: Labib–Russo 2026's χ=3 at 4 copies is the FACE state, not π/8 — do not import | port Magic5FromCat — and the rule being RECURSIVE (N(t)=3·N(t−4), each term keeps a T) is native, not a risk: a branch IS a child holon, so the decomposition interface should be recursive like the object itself |
| 2.5 — exact shots | **BUILT** | brute-force overlaps + certified branch sums | Aer ext-stab shot throughput, at exactness Aer cannot match at any speed | O(branches²) → orbit-aware Gram |
| mesh (CPU shards / GPU / cluster) | **BUILT** intra-node (shard-invariant CPU; 4090 at 336–396×, struct-determinism); the merge law is now a THEOREM, not a test result (`lean/CIRISHolon/MergeLaw.lean`: `shardedFold_invariant`, `digest_convicts` — zero-false-positive corruption conviction) | the merge law's Lean proof + its Rust tests | **near-linear to 1024 shards multi-node**; quiet-machine efficiency curves owed | inter-machine transport via the one transport square |
| bulk — MPS/DMRG | **SCAFFOLD** (MpsHolon shape; python DMRG upstream, ED-certified) | exact ED ≤ 20 sites; Schwinger closed forms | **ITensor/TeNPy** sweep-time parity on Schwinger-class Hamiltonians | port DMRG onto the holon object + merge law |
| crystal — gauge-coupled matter | **NAMED** — entry gated on SCHWINGER-2's verdict (running); recursion is the OPPORTUNITY: RG/coarse-graining IS the holon's recursive chart, so the tier is built ON internal recursion under the one merge law | the staked five-rung ladder (gauge sweep, verified anchors): R0 ED+mass-shift `m_lat = m − N_F g²a/8` (Dempsey PRR 4, 043133 — an exact discrete chiral symmetry, not a fit; 0.06% on ≤16 sites); R1 M_V/g → 1/√π = 0.5641895835 (parity 0.56419(4) Byrnes PRD 66; 0.56421(9) Bañuls JHEP 11 (2013) 158); R2 condensate → e^γ/2π^{3/2} = 0.1599288349 (parity 0.159928(1) Buyens arXiv:1411.0020 — 7 figures, the programme's tightest); R3 M_S/g → 2/√π staked at 3–4 digits ONLY (published scalar errors are 10–30× vector); R4 (m/g)_c at θ=π = 0.333561(4) / 0.333556(5) (two independent methods, 0.78σ — carries its own internal refuter). Compute parity line: N=1000 sites machine-precision on one machine (Arguello Cruz arXiv:2412.01902 App. C — whose speed comes from DISCARDING the random initial MPS: determinism and speed aligned, citable). **TRAP staked: do NOT stake 2-flavor until the factor-2.19 Hosotani-convention gap is closed** (Schwägerl PRD 112 vs Itou arXiv:2307.16655) | SCHWINGER-2 verdict → port to bulk tier |
| physics/graphics — grain → cosmic | grandfathered engine (battery-at-touch) | the conformance battery per tier | **browser: 60 fps at 10⁶ grains in WASM** (the sandbox viewer builds and is gated on every commit; the hosted tab was retired from the thesis page with the spin-out — hosting it HERE is owed); native: Rapier/PhysX-class rigid-body throughput, measured not claimed | per-tier battery certification |
| deployment range | WASM **builds and is gated today** (hosted demo owed here since the thesis page's tab retired) → laptop GPU (4090, measured) → clusters (mesh law, multi-node owed) | determinism at every rung | identical certified results at every scale — the range IS the product | multi-node demo |

| front-end — the OpenQASM surface | **COMPLETION DEBT for tiers 0/1** (they are not done until the surface EXCEEDS OpenQASM) — today: OpenQASM 2 subset (fixed Clifford+T+CCX gate enum, one q/c register, terminal measurement only; no parameterized gates, custom gate defs, reset, mid-circuit measurement, or classical conditionals — no adaptive circuits) | the spec + public corpora | **MQT Bench** (>70k circuits, OpenQASM 2/3) and the **ABSTRACTS** Clifford+T suite (arXiv:2608.24370) parse-and-run coverage | the route, in order: adaptive Clifford (mid-circuit measurement + feed-forward stays efficient, Aaronson–Gottesman §III); full OpenQASM 2 surface (registers, custom gates, reset, conditionals); arbitrary angles by Ross–Selinger synthesis — approximate ONCE at the front door as an exact Clifford+T word, T-count O(log(1/ε)), so the approximation is explicit, isolated, and priced in the same T currency |
| open systems — noise and channels | **NAMED** — no density matrices, Kraus channels, or noise models anywhere yet | exact small-system channel truth; Aer noise simulations | match Aer's noise-model results exactly where the channel is Clifford-representable; refuse elsewhere | design: the ledger's channel form (mixed stabilizer / CH-form prior art first) |
| verification scope — the Lean layer | kernel theorems PROVED (closure walls, tier structure, one-rebit stabilizer kernel); **the running Rust engine is NOT verified** and the claim discipline forbids saying otherwise | Lean CI on the theorems; the engine's own referees for the engine | **VOQC/SQIR's extraction discipline** (arXiv:1912.02250) — a verified path from proof to running kernel; nearest structural precedent to cite: **Lean-QEC** (arXiv:2605.16523) | state-and-prove stabilizer closure ABOUT the tableau implementation (axis found vacant by the sweep) |

## The honest boundary — envelopes, caps, exemptions, refusals

*The goal is to find the EDGES of what the holon can do by robust simulation of
reality — so the edges are named, enforced, and part of the record, never
discovered by a reviewer first. A refusal is a result.*

| boundary | state | enforcement |
|---|---|---|
| **arithmetic envelope** | exactness holds while every Z[ω] coefficient fits i128; coefficients grow like 2^{O(n+t)} (Quist–Coopmans–Laarman, arXiv:2602.17775), so the envelope is reachable | **ENFORCED 2026-08-27**: `Cyc` add/mul/alignment REFUSE (panic) on overflow in all three rings — holon ledger, holon-qasm referee, GPU host ring (`envelope_tests` pin it); the GPU fold was already magnitude-pre-guarded. Next: per-run envelope line in the certificate; differential oracle vs Selinger's bignum `ZOmega` (newsynth). **And the envelope is now ROUTED AROUND, not just enforced**: `residue.rs` carries the fold in CRT prime children — each prime a child holon under the one merge law, the certificate's own digests as the carrier (`digests_jointly_faithful` is its faithfulness proof) — so `amplitude_auto` scales the RING to the circuit and no coefficient envelope exists on that path at all. Not a workaround: the object recursing. Refusal remains the direct path's backstop |
| **statevector cap** | reference tier refuses above N_MAX = 24 qubits by name | router assertion, `holon-qasm/src/lib.rs` |
| **router magic cap** | branch-sum route taken at t ≤ 12 (and no Toffoli) by default | router condition, `holon-qasm/src/lib.rs:185` |
| **sampler working scope** | t ≤ 8 comfortable, t ≈ 10 the edge (documented in-source) | `holon/src/sample.rs` header; O(branches²) Gram is the next lever |
| **adaptivity** | none — terminal measurement only; a mid-circuit measurement is REFUSED at parse, not approximated | parser; lifted by the front-end row's milestone |
| **CI exemptions** | DMRG (python, upstream referee), the Hubbard reference, and holon-gpu run outside the default gate set — allowlisted WITH owner and exit criteria, not hidden | `ci-gates.sh` CRATE_ALLOW |
| **claim discipline** (from the adversarial sweep) | the exact five-integer Z[ω] representation is **SliQSim's** (Tsai–Jiang–Jhang, DAC 2021) and **quizx** already pairs the ring with stabilizer decomposition (QST 2022) — we cite, we do not claim the ring; shard-invariant merging is a FREE CONSEQUENCE of exact arithmetic (exactness buys it; we never claim it as a technique — ReproBLAS solved the hard float version); certificates have ABFT ancestry. The surviving claim is the **conjunction** — certified exactness + refusal boundary + distribution + kernel theorems — plus the vacant grounds the sweep found (no machine-checked simulator kernel, no certificate-carrying distributed simulation standard) | `PRIOR_ART.md` (in progress — six-lane sweep) |

### The boundaries, challenged (2026-08-27) — which bend, which break, which stand

Each boundary audited one by one, with the absoluteness claim mechanized where
a theorem exists to state (`lean/CIRISHolon/Boundaries.lean`):

| boundary | verdict | machine witness |
|---|---|---|
| arithmetic envelope | **BREAKABLE** — absolute in KIND (no fixed width carries ℤ: `no_fixed_width_carrier`), pure engineering in LOCATION (128 bits). Removal routes: bignum, or CRT residue arithmetic — and `digests_jointly_faithful` proves the certificate's own mod-p digests jointly separate distinct values, so the corruption detector and the envelope-removal mechanism are ONE mechanism (`MergeLaw.digest_commutes` already carries each residue through the whole fold) | `Boundaries.lean`, `MergeLaw.lean` |
| statevector cap (24q) | number **IGNORABLE** (routing default; exascale statevector reaches ~50q); law **ABSOLUTE for generic states** — `generic_state_table_absolute`: distinguishing the 2^(2^n) support patterns needs 2^n bits, pigeonhole, no cleverness exempted. The ladder exists because reality is not generic: structure is what the router detects, and hitting this wall MEANS no structure was found | `Boundaries.lean` |
| router magic cap (t≤12) | **IGNORABLE as stated** (latency default; the mesh already ran t=28 exact). The law behind it is NOT information-theoretically absolute: the WALL is proved (`magic_wall`, re-exporting `Object.pullback_not_pauli`), but the exponential PRICE is open — lower bound linear (PSV 2022) vs upper 2^{0.3963t} (QPG 2021). The one boundary our instrument might genuinely BEND: certified exact decomposition search attacks the open problem itself | `Boundaries.lean`, `Stabilizer.lean` |
| sampler scope (t≤8..10) | **BENDABLE** — the O(branches²) Gram is an algorithm choice, not a law; orbit-aware Gram is the milestone | — (no law of its own) |
| no-adaptivity | **REMOVABLE, and now COMPLETION DEBT** — adaptive Clifford stays efficient (Aaronson–Gottesman §III; measurement-update closure mechanization OWED, named in `Boundaries.lean`); arbitrary angles enter exactly via Ross–Selinger synthesis at the front door | `Boundaries.lean` header (debt named) |
| CI exemptions | **NO LAW AT ALL** — process debt; write the gates. Listed apart so a chore is never laundered into a limit | — |

## TODO — Ossicle integrations (trust and entropy; none of these buy speed)

Parked until the speed program lands; each is additive and none touches the
exact tiers' semantics.

- **Certificate-logged measurement entropy**: CIRISOssicle's on-GPU TRNG
  (timing LSBs, 7.99 bits/byte) feeds terminal-sample outcome selection;
  the draw is logged in the certificate so runs are unpredictable in
  advance and replayable after. Requires SP 800-90B-style conditioning and
  health tests before the certificate may cite the source. The fence,
  stated once: entropy buys ZERO magic — no TRNG pays a T-gate's price.
- **Shard sole-tenancy attestation**: Ossicle's VALIDATED workload detector
  (100% TP / 0% FP, 2.5 ms latency, 1% floor) runs beside each GPU shard
  and appends a "sole-tenant during the fold window" attestation to the
  shard certificate — the algebraic digest proves the arithmetic, the
  strain gauge attests the environment. The engine's own fold must be
  whitelisted by signature (the shard thereby proves it ran ONLY the
  engine).
- **Certified-randomness audit demo**: an Ossicle-sourced stream whose
  expansion is audited by the exact engine — the sharpest form of the
  "who referees the referee" critique of float-certified randomness
  pipelines (CAMPAIGNS.md side-bets).
- **Critical-ridge share measurement on the timing stream**: the
  edge-of-chaos jitter is a wild near-critical substrate; measure its
  whole-only share on the 4090 with the full timeseries discipline
  (phase-randomization null, clip artifacts, 0.227/N floor) and the
  classical cap expected to bind — a stance instrument, not an engine
  feature.

## The tuning module — organic degradation under a declared policy

`engine/crates/holon/src/tune.rs` + `lean/CIRISHolon/Tune.lean`. The DX
declares what is HELD (exactness, or a latency/frame budget) and what may
DEGRADE, in order, to declared degrees; the certificate records what
degraded; refusal is the total fallback. The law is machine-checked:
`select_sound` (the hold is held), `select_complete` (refusal only when
nothing lawful remains), `exact_never_degraded`, and `frame_budget_held` —
the referee face and the graphics face are one selector with the hold
swapped. **This gate is what real-time browser rendering rides on**: the
graphics tier holds the frame budget and degrades detail organically
(level-of-detail generalized), and every tier's banked speedup widens what
fits inside the frame. The selector's v1 routing encodes only MEASURED
rules (t>n → pruned; t≤n → sliced; t≥5 beyond n → magic5); unswept
interactions are named (`Unswept::Magic5TimesSliced`) and never guessed.
WHY a held configuration is ideal on given hardware is Limits.lean's half:
sweeps stop where the HARD floors (L1, L4) say there is nothing left, and
keep finding wins exactly on the BEGGING axes (L2, L3). Calibration is
rented: sweep tables carry host fingerprint + epoch, foreign tables are
ignored.
