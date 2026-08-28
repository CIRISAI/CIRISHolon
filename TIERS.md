# The tier ladder — scaffold, referees, and benchmark targets

*Every tier: its status, the referee that certifies it, the benchmark target
it is measured against, and the next milestone. A tier without a referee and
a target is not scaffolded — it's a wish. Statuses are honest: ASSUMED means
proven and no longer under test; BUILT means certified against its referee;
SCAFFOLD means the object shape exists and the target is named; NAMED means
only the plan exists.*

| tier | status | referee | benchmark target | next milestone |
|---|---|---|---|---|
| 0 — classical bit-planes | **DONE** — core assumed, front-end superset certified, and ADAPTIVE circuits landed (`adaptive.rs`: mid-circuit measurement, reset, feed-forward on a real classical register, seeded and replayable; teleportation verified across 32 seeds and a repetition-code syndrome cycle corrected) | exact statevector | word-parallel batching (64 shots per word op, free) | batch mode |
| 1 — stabilizer (packed Pauli planes) | **DONE** — ahead of stim at 7/7 sizes, adaptive circuits landed, and the tier's CANONICAL FORM now explicit (`PackedTableau::canonicalize`/`canon_key`: RREF over F₂ with the symplectic partner mirrored on the destabilizer half, so different gate orders reaching one state give one key — presentation stripped, content kept) | certified unpacked tableau + qiskit + stim | **stim ≤ 1× — REACHED AT EVERY n** (quiet-runner bake-off, 7/7 sizes ahead: 0.69× at n=64 down to 0.437× at n=4096 — 1.4–2.3× faster, margin growing with n; `conformance/BENCHMARKS.md` entry six, reproducible via the `bakeoff` workflow). Stack: transposed flat column engine + fused AVX2/WASM-SIMD128/scalar rowsum kernel (bit-identical by gate) + one-pass canonical terminal sampler | roofline analysis: is the margin maximal? — and the Born-vs-Born re-measurement (entry nine) plus the OWED QuiZX head-to-head on structured instances |
| 2 — magic (exact Z[ω] branch sums) | **BUILT, and the exponent moved**: Magic5FromCat LANDED (magic5.rs, exact-equality-gated against both prior paths and the frozen referee, planted defects caught) — realized α 0.4111 at t=28, 0.4027 at t=64 (74.8× fewer branches), asymptote 0.3963 never quoted as a measurement; AND branch-slicing LANDED (sliced.rs, 64 branches/word on a proved structural sharing theorem, bit-identical at every lane) — 14–26× where t ≤ n. **Positions against the lake's limits (Limits.lean): the slicing factor is an L2 (word width, BEGGING) position at 22–41% of the 64-wide bound, the residual being L4-flavored (exact γ coefficients are not bits — ring fraction 3–6% at n=64); the exponent is an L3 (BEGGING) position above the open floor.** Pruned×sliced are ALTERNATIVES (dedup destroys shared structure; measured both ways); magic5×sliced is UNSWEPT and the tuner refuses to guess | frozen holon-qasm + qiskit/Aer | the published exact-exponent ladder (numerically verified in `conformance/srank/`): Bravyi–Smith–Smolin 2016 6→7 α≤0.4679 (the simulator on it is Bravyi–Gosset 2016) → **Qassim–Pashayan–Gosset 2021 α≤0.3963** (Quantum 5, 606; explicit closed-form cat construction — realized α at t=64 is 0.4027, quote it with the finite-t caveat) → **Magic5FromCat** (Kissinger–van de Wetering–Vilmart, TQC 2022: 4-to-3 partial rule, 0.3963 CONCRETELY at finite t, Apache-2.0 Rust in quizx over our exact ring — 74.8× at t=64) → opportunistic cat₄/cat₆ 0.25–0.264 where circuit structure allows → T-count preprocessing (PyZX full_reduce) multiplicative on top. Trap defused: Labib–Russo 2026's χ=3 at 4 copies is the FACE state, not π/8 — do not import | port Magic5FromCat — and the rule being RECURSIVE (N(t)=3·N(t−4), each term keeps a T) is native, not a risk: a branch IS a child holon, so the decomposition interface should be recursive like the object itself |
| 2.5 — exact shots | **BUILT** | brute-force overlaps + certified branch sums | Aer ext-stab shot throughput, at exactness Aer cannot match at any speed | O(branches²) → orbit-aware Gram |
| mesh (CPU shards / GPU / cluster) | **BUILT** intra-node (shard-invariant CPU; 4090 at 336–396×, struct-determinism); the merge law is now a THEOREM, not a test result (`lean/CIRISHolon/MergeLaw.lean`: `shardedFold_invariant`, `digest_convicts` — zero-false-positive corruption conviction) | the merge law's Lean proof + its Rust tests | **near-linear to 1024 shards multi-node**; quiet-machine efficiency curves owed | inter-machine transport via the one transport square |
| bulk — MPS/DMRG | **SCAFFOLD** (MpsHolon shape; python DMRG upstream, ED-certified) | exact ED ≤ 20 sites; Schwinger closed forms | **ITensor/TeNPy** sweep-time parity on Schwinger-class Hamiltonians | port DMRG onto the holon object + merge law |
| crystal — gauge-coupled matter | **ENTRY EARNED, referee VOID — successor staked** (SCHWINGER-2 voided by its own N-convergence premise at x=9; cause diagnosed and avoidable: the grid used one N range for every x, where the finite-volume standard N ≳ 20√x — recorded in our OWN prior-art sweep — demands N scale with x, so x=16 was under-resolved before it ran. The x=4 column survives fully posable; the instrument is untouched and certified two-sided. SCHWINGER-3 stakes per-column N.)  Previously:: the bridge campaigns (`conformance/gravity`, five frozen preregs, two VOID by their own gates) closed rung 0 of the curvature module on exact unitary gauge dynamics with charged matter — matter→geometry (ω=0,1,0,1), the closure rung (δ=0,½,0,½, thrice replicated), geometry→matter (silent/flip/silent/inert), all two-routed on two triangulations with plants firing; the DMRG referee (SCHWINGER-2) is running its staked grid with checkpoint 1 converged to 7e-10, far inside its 1e-3 premise; recursion is the OPPORTUNITY: RG/coarse-graining IS the holon's recursive chart, so the tier is built ON internal recursion under the one merge law | the staked five-rung ladder (gauge sweep, verified anchors): R0 ED+mass-shift `m_lat = m − N_F g²a/8` (Dempsey PRR 4, 043133 — an exact discrete chiral symmetry, not a fit; 0.06% on ≤16 sites); R1 M_V/g → 1/√π = 0.5641895835 (parity 0.56419(4) Byrnes PRD 66; 0.56421(9) Bañuls JHEP 11 (2013) 158); R2 condensate → e^γ/2π^{3/2} = 0.1599288349 (parity 0.159928(1) Buyens arXiv:1411.0020 — 7 figures, the programme's tightest); R3 M_S/g → 2/√π staked at 3–4 digits ONLY (published scalar errors are 10–30× vector); R4 (m/g)_c at θ=π = 0.333561(4) / 0.333556(5) (two independent methods, 0.78σ — carries its own internal refuter). Compute parity line: N=1000 sites machine-precision on one machine (Arguello Cruz arXiv:2412.01902 App. C — whose speed comes from DISCARDING the random initial MPS: determinism and speed aligned, citable). **TRAP staked: do NOT stake 2-flavor until the factor-2.19 Hosotani-convention gap is closed** (Schwägerl PRD 112 vs Itou arXiv:2307.16655) | SCHWINGER-2 verdict → port to bulk tier |
| physics/graphics — grain → cosmic | grandfathered engine (battery-at-touch) | the conformance battery per tier | **browser: 60 fps at 10⁶ grains in WASM** (the sandbox viewer builds and is gated on every commit; the hosted tab was retired from the thesis page with the spin-out — hosting it HERE is owed); native: Rapier/PhysX-class rigid-body throughput, measured not claimed | per-tier battery certification |
| deployment range | WASM **builds and is gated today** (hosted demo owed here since the thesis page's tab retired) → laptop GPU (4090, measured) → clusters (mesh law, multi-node owed) | determinism at every rung | identical certified results at every scale — the range IS the product | multi-node demo |

| front-end — the OpenQASM surface | **SURFACE LANDED, adaptivity still owed**: `qasm.rs` accepts the superset quizx and qiskit/OpenQASM circuits actually use (y, sx, sxdg, cz, swap, ccx, ccz, rz/p at π/4 multiples with EXACT ζ16 global-phase ledger — the odd residual declared, never dropped) — built holonically: lowering rules are DATA applied by one recursive rewriter, every rule carries its own transport certificate against an independent dense Z[ζ16] oracle (`tests/qasm_oracle.rs`), scalars live in the ledger not the circuit, and non-π/4 angles REFUSE naming their routes (measured live: the IBM tracker's fstate refuses at its exact line; quizx's hidden-shift qasm runs natively). Previously: today: OpenQASM 2 subset (fixed Clifford+T+CCX gate enum, one q/c register, terminal measurement only; no parameterized gates, custom gate defs, reset, mid-circuit measurement, or classical conditionals — no adaptive circuits) | the spec + public corpora | **MQT Bench** (>70k circuits, OpenQASM 2/3) and the **ABSTRACTS** Clifford+T suite (arXiv:2608.24370) parse-and-run coverage | the route, in order: adaptive Clifford (mid-circuit measurement + feed-forward stays efficient, Aaronson–Gottesman §III); full OpenQASM 2 surface (registers, custom gates, reset, conditionals); arbitrary angles by Ross–Selinger synthesis — approximate ONCE at the front door as an exact Clifford+T word, T-count O(log(1/ε)), so the approximation is explicit, isolated, and priced in the same T currency |
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

## The grain: closure-aligned scheduling (the curvature module, banked)

The bridge campaigns gave the engine a LAW, not just a result:
`lean/CIRISHolon/Grain.lean` machine-checks its kernel — for an involution
R and imaginary unit i, (1+iR)² = 2iR (deterministic toggle) and
(1+iR)⁴ = −4 (revival up to global phase) — and
`engine/crates/holon/src/grain.rs` carries it as a scheduling primitive:

- **A `Grain` is a measured closure schedule**, not a clock. It names the
  steps where a coarse view is EXACT (zero defect), and it REFUSES to be
  constructed without provenance. `Grain::from_bridge_family()` is the one
  named constant (period 4, θ=π/4) and cites its frozen preregs; the
  period belongs to that coupling and never to nature.
- **The tuner consumes it**: a `Policy` may carry a grain, and every
  `Choice` reports `steps_to_close` — so coarse tiers (level-of-detail
  under a frame budget) refresh ON closure boundaries where the view is
  exact, instead of every frame, with a stated bound between them. Same
  law, two faces: the quantum face is the closure rung, the graphics face
  is when a level-of-detail refresh is free.
- **The measured constants are pinned in CI** (`BRIDGE_OMEGA`,
  `BRIDGE_DEFECT_HALVES`, `BRIDGE_BACKREACTION`): the engine holds the
  gravity record's numbers as a test, so editing them without a new frozen
  campaign fails the build.

## The exact-ring tower — complete, classified, and named where it ends

The front-end's question "which rotations can be exact?" has a COMPLETE
theoretical answer (Kronecker–Weber): `diag(1,z)` is exactly representable
iff `z` lies in an abelian extension of ℚ, i.e. `z ∈ ℚ(ζ_n)` for some n.
The wild families sort into four rungs, and `cyclo.rs::ring_for` classifies
any angle into them:

| family | phase | ring | status |
|---|---|---|---|
| Clifford+T (T, S, Z, CCZ, Toffoli) | ζ8 | `Z[ζ8]` = `ledger::Cyc` | ✓ the base |
| Clifford hierarchy: rz(π/8), rz(π/16), … | ζ16, ζ32, ζ64 | `Z[ζ_{2^k}]` = `cyclo::Cyclo` | ✓ **the general 2-power tower**, with `Cyc` proved to embed into every rung (arithmetic commutes with the inclusion) |
| face / T-type magic, arccos(1/√3) | NOT a root of unity, but `(1+i√2)/√3 ∈ ℚ(ζ24)` | `Z[ζ8][√3]` = `face::R3` | ✓ **the door to the IBM tracker's instances** |
| qutrit ζ3 and every 24th root | ζ3, ζ6, ζ12, ζ24 | ALSO `face::R3` — it already carries i, √3, ½, so ζ3 = (−1+i√3)/2 is in it | ✓ free, verified |
| ζ9-class qutrit magic (Strange state) | ζ9 | `Z[ζ9]`, degree 6, Φ₉ = x⁶+x³+1 | ✓ **BUILT** (`cyclon.rs`) — and generalized: the ring is `Z[ζ_n]` for ANY n, with Φ_n derived from scratch by exact recursive division (no tables, no trusted constants), agreeing with the 2-power tower where they overlap. **The classification now has no unimplemen
## What quizx is for, per tier — and the universal need underneath it

The direct answer first, because it is narrower than the question implies:
**quizx itself is usable at exactly one tier.** It is Clifford+T-specific —
it has no face ring, no symbolic angle, no qudits, no distributed
certificates — so it is the magic tier's canonicalizer and nothing else.
At every other tier it would be a category error.

But the PRINCIPLE it embodies is needed at every tier, and naming it
correctly is what makes the rest of the ladder cheaper:

> **A circuit's gate order is PRESENTATION, not content.** ZX rewriting
> wins because moving to a graph dissolves the sequence: Hadamards stop
> being barriers because there is no "before" left for them to bar. The
> redundancy that ordering hid becomes visible, and cancels.

That is the engine's own vocabulary from the predecessor lake:
`gauge_sector_is_order_degeneracy` (`Core/FrameOrder.lean`) — the gauge
sector is the corpus's own presentation. **The universal need at every tier
is a canonicalization pass that strips the presentation and leaves the
invariant.** Where the engine already has one, that tier is fast; where it
does not, that tier is where we lose.

| tier | the presentation | the invariant | canonicalizer | status |
|---|---|---|---|---|
| 0/1 Clifford | gate order | the stabilizer group | the tableau itself — a tableau IS the canonical form, which is WHY Clifford is easy | ✓ have it (and it beat stim) |
| 2 magic | gate order + branch order | the ZX graph / the phase-polynomial content | graph rewriting | ✗ **missing — and this is precisely the measured loss** (entries 13–17) |
| 2 magic, branch layer | which branch subset | the canonical affine state | `Affine::canonicalize` + `canon_key` | ✓ have it — it is why dedup collapses 2^t to O(1) on some families |
| bulk / MPS | the bond-index gauge | the Schmidt spectrum | MPS canonical form (left/right orthogonalization) | ✓ standard, ours inherits it |
| crystal / gauge | the lattice's gate schedule | holonomy and the constraint sector | the BRIDGE campaigns' own reading: the Floquet SEQUENCE is presentation, the closure/holonomy structure is content | ✓ that is what rung 0 measured |
| physics / graphics | draw and update order | the spatial structure | BVH + the grain schedule (`grain.rs`) — a LOD refresh on a closure boundary is exactly this move | ✓ have the law, tier not built |

So the composition recommendation sharpens: **use quizx at tier 2 only, and
read it as the tier's missing canonical form rather than as a faster
competitor.** Every tier that already has its canonicalizer is a tier we do
well on; the one that lacks it is the one we lose on, and the loss was
measurable to three orders of magnitude precisely because canonicalization
is not an optimization — it is the thing that decides which quantities are
even computable.

## Tier 0/1: complete

The completion debt named when the tier ladder was first written is paid,
and the tier now has all three things the per-tier analysis says a tier
needs:

1. **The capability** — ahead of stim at 7/7 sizes (Born-vs-Born, quiet
   runner), with a certified OpenQASM-superset front end whose every
   lowering rule is proved against an independent oracle.
2. **Adaptivity** (`adaptive.rs`) — mid-circuit measurement, reset and
   feed-forward on a real classical register. Free by Aaronson–Gottesman:
   a computational-basis measurement of a stabilizer state is either
   DETERMINED (the tableau knows it) or a fair coin, and either way the
   state stays a stabilizer state, so the tier's cost model does not move.
   Coins come from a seeded stream and the seed is part of the result, so
   an adaptive run is unpredictable in advance and replayable after —
   the same contract terminal sampling already had. Gated by teleportation
   across 32 seeds (correct only if measurement, feed-forward and the
   classical register all behave) and a repetition-code syndrome cycle
   that must return the data qubits to the codespace.
3. **The canonical form** (`PackedTableau::canonicalize` / `canon_key`) —
   RREF over F₂ on the stabilizer half with every operation MIRRORED by its
   symplectic partner on the destabilizer half (a swap pairs with a swap;
   `stab[i] *= stab[p]` pairs with `destab[p] *= destab[i]`). Without the
   mirror the pairing that `measure_peek` relies on is destroyed — caught
   by the state-preservation test, which is why that test exists. Contract:
   equal keys ⟺ equal states, idempotent, state-preserving, and distinct
   states do not collide.

This is what "complete" means for a tier under the per-tier analysis: the
capability, the adaptivity the use case needs, and the canonicalizer that
strips presentation from content. The magic tier has the first two and
lacks the third, which is exactly where it loses.

## Requirement-3 structure: complete at toy scale (2026-08-28)

`ClosureDerives.lean` + CLOSURE-2B + **EINSTEIN-ADM-1C** (re-adjudicated
after external re-review: the original E1 was VACUOUS on a stationary
carrier — claim withdrawn, M-FIXED-POINT-TRAJECTORY registered; the
corrected E1' checks closure UNIVERSALLY, by exhaustion over all
configurations, and passes: the quantum step descends exactly to a
nontrivial classical dynamics on the discrete ADM phase space, preserving
the 2+1 Einstein constraint at the quotient level, with inheritance
demonstrated on a provably moving trajectory). Precise remainder: the
dynamics is INPUT and its closure is the theorem — deriving the dynamics
FROM closure is the open rung. Successors: SU(2)-via-2T, the 3+1
continuum.

## Requirement 1 status after LOCAL-1E (2026-08-28)

Endogenous ✓ (BRIDGE-6/7B R1) · charged-sourcing-observable ✓ (WILSON-2B)
· **local ✓ (LOCAL-1E: the response-function cone, all gates, plants
firing)**. Remaining for requirement 1: ONE model carrying all properties
simultaneously (LOCAL-2: the reciprocal pump inside the cone-verified
dynamics), then the non-abelian instance (FROBENIUS-1).

## CLOSURE-2B adjudicated (2026-08-28): the missing memory is not one-body

On a validated instrument (both plants firing): the configuration
channel's minimax defect is positive and exact (C1); inheritance holds
exactly (C3); and the (Wilson, 't Hooft) channel — the honest discrete
(h, π) — is STILL blind at every firing collision (C2 dead). Requirement
3's toy statement sharpens: closure derives the dynamics where it holds
(EINSTEIN-ADM-1), and where it fails, the missing memory is provably not
one-body phase-space data — it is correlational. Successor: the
pair-correlator channel, outcome open.

## CLOSURE-3 adjudicated (2026-08-28): the memory ladder terminates at second order

v_pair separates every firing collision on a validated instrument (the
pair-sensitivity plant's states are one-body-equal, pair-distinct). The
coarse bookkeeping that closes this model is (one-body + pair); 2B's
whole-pattern reading is BOUNDED accordingly. Sixth green campaign.
