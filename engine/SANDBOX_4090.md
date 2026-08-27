# SANDBOX_4090 — an atomically-accurate sandbox as many small holon engines on one GPU

Status: design study, and its recommendation is **negative for the headline use case**.
Hardware numbers are **measured on the local device**, not published specs. Prototype numbers are
measured. GPU throughput is measured on a *structural* kernel, not a granular contact solver.
Tier table, caps, resident sets and solver timings are the holon-sandbox lane's, measured at
`708b67d`, relayed 2026-08-23, and carry that lane's HARD/SOFT/PENDING labels.
Date: 2026-08-23.

Frame: `INTEGRATION_FRAME.md` — one holon, values only. A shard is an **arena with a declared g0 and
root**; a boundary relation is a **pair of indices**. No new entity class appears anywhere below.

---

## 0. The recommendation: **do not build a GPU backend to make the sandbox real-time. It cannot.**

The sandbox tier buys **2.76% of real time per frame** (HARD, measured: ~9 ms/frame at 110–200 nodes,
wasm, explicit integrator against real quartz stiffness). Real-time therefore needs **36.2×**. That
is the number a GPU would have to deliver, and the measurements say it cannot deliver it *on this
workload*:

| | |
|---|---|
| measured default resident set, ball impact | **149 holons** (grading 2) |
| that, in GPU units | **4.7 warps** — against the card's **304 warp slots** |
| measured GPU rate, one engine | 4.145e8 holon-steps/s |
| implied wasm rate on the same scene | 9.853e7 holon-steps/s |
| **speedup available on one narrow scene** | **4.2×** |
| **speedup needed for real-time** | **36.2×** |

A 149-holon frontier is **~65× too narrow to fill the card**. The step count is enormous (CFL at
g0 = 0.5 mm and a 5,990 m/s quartz wave speed gives dt = 4.17e-8 s, ≈5,950 steps per 9 ms frame) and
the per-step width is five warps. **GPUs need wide steps, not many narrow ones** — and at 3.0 µs of
launch overhead, a non-persistent kernel would spend 18 ms of pure launch per frame, twice the whole
budget. The 36× has to come from a wider step or a bigger dt (an implicit integrator), not from this
hardware.

**What a GPU can honestly buy is throughput, not latency:** ~1.0e6 concurrent 149-holon scenes fit in
memory and 1.49e8 holons per step is amply wide for the card — but every one of those scenes still
runs at 2.76% of real. That is a **batch-simulation machine**, and it is a real thing to want; it is
just not a faster sandbox.

**And residency is not the blocker anywhere it looked like one.** Z0 GRAIN's crack claim demands
ℓ_ch/10 = 4.222e-7 m of a tier whose floor is 1e-6 m — **2.37× short, always, by the tier's own
arithmetic.** The card can hold 1.19 grains of sand at 1 µm resolution (1.49e8 resident capacity
against Z0's 1.25e8 subgrain ledger), and doing so **does not lift the GrainFloor by one bit**.
`GrainFloor` is a property of *the claim against g0*, not of how many holons you can afford.

Build the backend only if the answer to "what for" is **batch throughput across many scenes**. For
anything else, the money is in the certifier (§7, G8: a measured **13,935×** already in hand on the
CPU) and in the integrator.

---

## 1. Hardware truth — the brief's device premise is wrong

`nvidia-smi` and `cudaGetDeviceProperties` agree:

| | brief assumed (AD102) | **measured (this device, AD103)** |
|---|---|---|
| part | RTX 4090 | **RTX 4090 *Laptop* GPU**, CC 8.9 |
| SMs / cores | 128 / 16,384 | **76 / 9,728** |
| memory | 24 GB, ~1 TB/s | **15,944 MiB, 576.1 GB/s theoretical / 435.9 measured** |
| f32 | ~83 TFLOPS | **27.40 TFLOP/s sustained** (96.8% of peak at a locked 1455 MHz) |
| f64 | ~1.3 TFLOPS (1/64) | **0.372 TFLOP/s** — ratio **1/63.8**, the 1/64 rate confirmed |
| L2 | — | **64 MiB** |
| power | — | 80 W default cap (175 W max); clock pinned at base, no throttle over 6 sustained rounds |

The card is **~40% of the compute and ~66% of the memory** the brief budgeted.

Other measured primitives: **i64** 4.38 TIOP/s mul-add, 3.49 TIOP/s add-with-dependency. **Kernel
launch 3.0 µs flat**, independent of grid size. **u64 `atomicAdd`** 0.002 T/s on one slot vs 0.167
T/s across 4,096 — an **83× contention penalty**. **Warp slots: 76 SMs × 4 schedulers = 304.**

---

## 2. The ledger cap: **0.59 grains**, not 3.54

The lane's derivation and mine converged, and the answer is tighter than either first said. One
0.5 mm quartz grain = **5.2116e18 atoms**. `GrossState` has four integer lanes, and **which one binds
is a property of the chart, not of the ledger**:

| what the chart writes per leaf | binding lane | cap (constituents) | **grains** |
|---|---|---:|---:|
| occupancy 0, momentum 0 — *the sandbox chart* | `constituents` u64 | 1.8447e19 | **3.54** |
| occupancy 2 — the `aggregate(n, 2n, …)` demo idiom | `occupancy` u64 | 9.2234e18 | 1.77 |
| occupancy 3 (mean of 0..6) | `occupancy` u64 | 6.1489e18 | 1.18 |
| **occupancy 6 (REG+ max, all six FHP directions)** | `occupancy` u64 | 3.0745e18 | **0.59** |
| **momentum 3 (FHP max per component)** | `momentum` **i64** | 3.0745e18 | **0.59** |

Two things worth carrying:

- **The momentum lane is `[i64; 2]`, not u64** — half the range before the sign bit. At the FHP
  maximum of 3 per component it lands in exactly the same place as saturated occupancy, so a chart
  that writes momentum has no more headroom than one that writes occupancy. **If a single number is
  quoted, 0.59 grains is the defensible general cap; 3.54 is the geometric-chart special case.**
- `checked_combine` refuses **per lane**, so an overflow in any lane is a refused materialization,
  not a wrong number.

The sandbox is safe at 3.54 because it writes occupancy `0` everywhere — now *gated*, not assumed
(`scene::tests::the_sandbox_chart_writes_no_occupancy` grows 12 levels and asserts every holon's
occupancy and both momentum components are zero). The general bound is executable as of `708b67d`:
`tier::Lane`, `tier::LeafWrites`, and `ledger_for(domain, g0, packing, writes)` returning
`Ledger::Fits{binding}` or `Ledger::Overflows{lane, factor}` — **it reports which lane binds even
when nothing overflows.** That closes what this study had open as G6.

The cap is not theoretical; it **bit twice during the lane's own sizing**: g0 = 1 m on Earth gave
1.08e21 (59× over, forced 10 m); g0 = 10 kpc on the observable universe gave 2.32e19 cells (1.26×
over, forced 40 kpc).

---

## 3. The tier table, and where the floor bites

**Naming caution:** these are the sandbox lane's **Z zoom tiers**, which are *not*
`DESCRIPTOR_CHAIN`'s T0–T5. They overlap without corresponding one-to-one — Z−1 sits at T2's grain,
Z0 at T3's, Z2 at T4's. Anything in this document written as T*n* cites `DESCRIPTOR_CHAIN` §3; the
table below is Z.

| Z | tier | g0 | domain | constituents | certifies | **verdict today** |
|---|---|---:|---:|---:|---|---|
| −2 | gauge | none | none | 4 links | Gauss closure, exact | Certified |
| −1 | crystal | 4.9134e-10 m | 1e-6 m | 8.43e9 | — | **NoEvaluator** |
| 0 | grain | 1e-6 m | 5e-4 m | 1.25e8 | crack path | **GrainFloor** |
| **1** | **sandbox** | **5e-4 m** | **0.6 m** | **6.2208e8** | contact impulse | **Certified** |
| 2 | landscape | 1e-2 m | 2e3 m | 4.8e15 | crack path + impulse | Certified |
| 3 | planet | 10 m | 1.2742e7 m | 1.083e18 | — | **NoGravityChart** |
| 4 | galactic | 3.0857e16 m | 9.46e20 m | 1e11 | — | **NoGravityChart** |
| 5 | cosmic | 1.2e21 m | 8.8e26 m | 2e12 | — | **NoGravityChart** |

**Four of eight tiers refuse.** That is the design working, not a gap.

**Where `GrainFloor` bites, and why** (ℓ_ch = E·G_F/f_t², demand = ℓ_ch/10):

- **Z0 grain**: ℓ_ch = 4.222e-6 m → demands 4.222e-7 m of a tier whose floor is 1e-6 m. **Floors by
  2.37×, always.** Computed from the tier's own values.
- **Z2 landscape**: ℓ_ch = 0.1375 m → demands 1.375e-2 m. Meetable, certifies.
- **Z1 sandbox does not ask the crack question at all** — a granular claim demands `g0` (5e-4 m), not
  ℓ_ch/10. Asking it the cohesive question would demand 4.222e-7 m of a tier whose terminal holon is
  5e-4 m: GrainFloor forever, which is the right answer to the wrong question.

**The consequence for sharding, and it is the load-bearing one: the demand a tier makes is a function
of what it CLAIMS, so a shard's work depends on the claim, not just the geometry.** Two shards of
identical size and identical grain can carry wildly different certification cost because they claim
different things. Any static, geometry-based load balancer is therefore wrong by construction.

### Note on scene dimensions

The lane's committed scene is a **square 2D domain** — 0.6 m square, **6.2208e8 grains** — not the
brief's 0.6 × 0.4 × 0.3 m box, whose 6.60e8 at 60% packing is separately correct. Two different
declared boxes, not a discrepancy. (SOFT: the fill fraction is a stage value; note that fill 0.45
against a 1200³ ratio implies 7.776e8, while the declared 6.2208e8 corresponds to 0.36 — that
particular pair is still worth an assertion, gap G7.)

---

## 4. Residency — measured, and far smaller than any estimate

**MEASURED** (wasm, ball impact, default resolution demand):

| grading | resident holons | active cells | certify | frame @ 9 ms budget |
|---:|---:|---:|---:|---:|
| 4 | 117 | 86 | 0.06 ms | 9.0 ms |
| **2 (default)** | **149** | **110** | **0.17 ms** | **9.0 ms** |
| 1 | 233 | 169 | 0.08 ms | 8.9 ms |
| 0.5 | 645 | 466 | 0.23 ms | 8.6 ms |
| 0.25 | 1,865 | 1,365 | 0.68 ms | 9.9 ms |
| 0.125 | 5,605 | 4,138 | 2.21 ms | — |

**Settled pile and pour: PENDING.** The only interaction implemented is a single ball impact; there
is no pour and no settling run. The lane's own honest guess is that a pour is **5–50× the impact's
resident set — a guess, not a measurement, and it is theirs, not mine.** It is not used in any
arithmetic in this document. The reason extrapolation is unsafe is structural: the impact frontier is
**corridor-local** (fine only near contact), while a settled pile or pour resolves over a much larger
fraction of the domain.

**Capacity, for contrast** — 56 B is the header, not the holon:

| what you actually pay per resident holon | B | holons in 15,944 MiB |
|---|---:|---:|
| `RuntimeHolon` header alone (verified `sizeof` = 56) | 56 | 2.99e8 |
| + `first_child`/`next_sibling` index (2×u32) | 64 | 2.61e8 |
| + live ledger overlay (`GrossState`, 32 B — the prototype proved this **mandatory**, §6) | 96 | 1.74e8 |
| + whole-state pool at W=2 | **112** | **1.49e8** |

**149 resident against 1.49e8 capacity is a factor of 1.0e6.** Memory is not the constraint at any
size the lane reached, and it is not the constraint for a GPU either. This study's first pass
estimated residency *geometrically* — every grain in a disturbed volume, 1.9e7–6.5e7 for a crater —
and that was wrong by **five orders of magnitude**, because the certificate keeps only the contact
frontier resident, never the disturbed volume.

---

## 5. The four design decisions (for the batch-throughput architecture, the only one that survives §0)

### D1 — Arithmetic: **native f64**. The 1/64 penalty is real and irrelevant.

Measured: native f64 **0.372 TFLOP/s**; df64 **1.369 Tdf-FLOP/s** (**3.68×** in pure FMA, **2.57×**
in the engine kernel); f32 a further 1.62× over df64.

At the measured workload the binding constraint is *width and step count*, not arithmetic
throughput — so buying 2.57× of arithmetic buys nothing, while spending it costs the thing that
matters. df64 carries ~48 mantissa bits against f64's 53 and is a **different number system**: it can
never reproduce f64 bit patterns, so adopting it drops the GPU out of the tri-target bit-identity
claim (`PORTABILITY.md`) permanently. Native f64 *can* stay in it — IEEE-754 add/mul/fma are
correctly rounded on both hosts, so with `__dadd_rn`/`__dmul_rn`/`__fma_rn` (contraction disabled),
fixed operation order, and no transcendentals in the stepping path, the GPU becomes a **fourth
bit-exact target**.

**Decision: native f64, strict intrinsics, contraction off.** Keep df64 implemented but unused,
behind an explicit declaration and its own error certificate. Its kill: *the df64 trajectory leaves
the certified band against the f64 reference.*

**Integer stays integer.** The REG+ ledger is 100% integer end-to-end and must remain so: integer
addition is exactly associative, so the ledger is order-independent **by construction** — the one
part of the system whose determinism needs no discipline at all.

### D2 — Layout: **SoA for the hot ledger**; the 56 B AoS header stays host-side

Measured on 16 M holons sweeping one field: **AoS 56 B = 1.20 ms; SoA u64 = 0.30 ms — 4.0×.** The
header drags 56 B of traffic for 8 B of payload. On the device, split hot lanes (`constituents`,
`occupancy`, `momentum[2]`, position/velocity) into separate SoA arrays; park cold topology in a side
array touched only at materialization. The host `RuntimeArena` keeps its 56 B `repr(C)` header — a
device-side *layout*, not an ontology change.

### D3 — Boundary protocol: **snapshot-then-apply over disjoint pairs**, certified by re-planning

From the CPU prototype (§6), which proved it exactly conserving and bit-identical. Two properties:
**snapshot-then-apply** (every transfer is a pure function of a port snapshot published after all
local steps and before any write — giving order-independence and per-shard bit-identity) and
**disjoint pairs** (each link owns its own port holon, so pair writes never overlap — making the
apply phase lock-free, with **no atomics on the exchange path at all**).

**What certificate covers a shard boundary: the re-planned transfer.** This is counter-intuitive and
it is the prototype's most important finding — **a global-sum gate does not work.** Doubling *both*
sides of a transfer, or dropping *both*, is perfectly conserved and perfectly antisymmetric; it
passes global sum, pair antisymmetry, apply-consistency, local conservation and composition. Only
re-deriving the intended transfer from the published snapshot catches it. The boundary certificate
must assert **plan-conformance**, not balance.

> **The gap that matters more than the protocol: nothing certifies a join across tiers.**
> The re-root rule, as the lane states it: *a tier declares its own g0 and domain; zooming builds a
> NEW `RuntimeArena`; nothing is shared between the two arenas — not ids, not the frontier, not the
> certificate.* Re-declared are g0, domain, root `grain_units`, root `GrossState`, and the material
> chart; **not** re-declared are the holon type, arena type, certifier, solver, relation type. Tiers
> differ in values.
>
> And: *"the certificate covers ONE tier, ONE throw… **It says nothing across a re-root.** There is
> no composed certificate spanning two tiers."* A **re-root ledger gate** (parent cell constituents
> == child root through the g0 ratio, mutation-tested) is **PENDING, in flight now**.
>
> **If shards live at different tiers, nothing in the current engine certifies the join.** Until that
> gate lands, a multi-tier swarm is uncertified by construction — so the first shipping swarm must be
> **single-tier**, where §6's exchange gate is sufficient.

Atomics drive the reduction design: per-engine private accumulators reduced hierarchically in fixed
order; never a global ledger atomic (83× contention penalty).

### D4 — Determinism: parallel **across** engines, sequential **within**

| | status |
|---|---|
| integer ledger composition and exchange | **provably deterministic** — integer + is exactly associative; `checked_add` refuses overflow rather than wrapping |
| boundary exchange ordering | **provably deterministic** — snapshot-then-apply; measured bit-identical across 1–32 threads × 4 repeats, on every holon |
| engine interior | **deterministic by construction** — one block, fixed loop order |
| f64 stepping, strict intrinsics + fixed order | **deterministic, conditionally** — only while contraction is off and no transcendental enters the step |
| cross-lane float reductions, float atomics | **not deterministic** — banned on the certified path |
| **seeded materialization** | **NOT portable to device today.** SplitMix64 is integer and fine, but the draw calls `log`/`cos`/`pow` and CUDA's libdevice is not bit-identical to `libm`. **Materialization stays host-side** (G2). |

---

## 6. CPU prototype: measured (`crates/holon-swarm`, 3,141 lines, 34 tests passing in release)

Standalone crate, one dependency (`ciris-sim-core`, `alloc`), no new external deps, no `unsafe`.
A shard *is* a `RuntimeArena`; a port *is* an ordinary holon wearing the `boundary` flag it already
has; a link *is* a pair of indices. No new entity class.

**Conservation: EXACT.** 80 rounds at the strictest gate, global `GrossState` bit-identical in all
four integer lanes while 8 of 8 shard balances moved off their opening values. No epsilon anywhere.

**Determinism: BIT-IDENTICAL.** Threaded == sequential over threads ∈ {1,2,3,4,8,16,32} × 4 repeats,
compared on every holon's ledger plus every whole-state f64 by `.to_bits()`, and under natural /
reversed / strided visit order. The sequential reference is written independently of the threaded
path, so agreement is evidence rather than tautology.

**Scaling** (release; `nproc` = 32 on a **heavily loaded shared machine**, load average 18–43, so
wall-clock speedups are not defensible and CPU-time-normalised efficiency is): **≥90% efficiency
against the cores it is actually given through 8 threads**, dropping to 62% at 16 — three `Barrier`s
per round make the round as slow as its slowest shard. **The barrier is the scaling limit, not the
exchange** (86.5 ns per boundary pair per round). Sharding alone gives a **1.64× single-threaded
speedup** at fixed total work, purely from working-set locality — independent support for the
many-small-engines premise, and it costs nothing to take.

**Mutation tests: 9 mutations × 3 gate levels × 2 execution paths, all caught at `Full`.**

**Two weaknesses found and pinned by tests, both design inputs rather than defects:**

1. **The obvious gate does not work** — doubling or dropping *both* sides passes every balance-based
   leg. Only re-planning from the published snapshot sees it. (→ D3.)
2. **`RuntimeArena::validate()` is structurally vacuous over a stepping shard.** The core exposes
   `holons()` as `&[RuntimeHolon]` with no mutable ledger accessor, so a stepping shard must carry an
   index-aligned `Vec<GrossState>` overlay. `validate()` reads the arena's *stored* gross states,
   which never change, so it cannot see ledger corruption at all — a gate built literally from "sum
   the roots + call `validate()`" is **blind to a broken composition**. Pinned by
   `weakness_a_ledger_only_gate_is_blind_to_a_broken_composition`, which fails if the escape ever
   stops happening.

That overlay is not a workaround — **it is exactly the flat device buffer the GPU wants** (32 B/holon
SoA), which is why it appears in §4's memory table as mandatory.

---

## 7. What "atomically accurate" honestly means

All-atoms-resident is impossible by eight orders of magnitude and is not the claim. The claim is the
**certificate chain**, and `DESCRIPTOR_CHAIN.md` is stricter about it than the brief:

- The chain **bottoms out in measurement, not in a deeper simulation** (§3, `:68`); the base
  certificate is **statistical, not deductive** (`:97`).
- The mechanical channel **terminates at `GrainFloor` at grain/crystal scale by design**; the atomic
  tiers exist to *certify the crystal descriptor's values*, "**not to be materialized in a game
  frame**" (`INTEGRATION_FRAME.md:81-84`). Z−1 CRYSTAL's `NoEvaluator` is that sentence executing.
- Some quantities are **minted at a tier and provably do not transport**: fracture energy's 30–100×
  gap from 2γ ≈ 1–4 J/m² to 110 J/m² is process-zone dissipation created at T4 (`:348`). An
  "atomically accurate G_F" is false *by theorem* — the number has no atomic ancestor.
- The chain is **experiment-pinned where it matters most**: feldspar (~60% of granite) has no
  fracture-grade certified potential (`:164`).

**What the atomic tier actually buys, measured:** the completed T2 DFT reference carries a **signed
bias of −10 to −14% on stiffness, shape sub-percent**. That is the honest content of "atomically
accurate" — not exactness, but a *certified, signed, quantified* error band on named observables.

**A 4090 sandbox could certify:** exact integer conservation of all four ledger lanes across every
shard and refinement, bit-for-bit; bit-exact replay of a given scene on a given device; contact
impulse at Z1 and crack path at Z2 within their validity domains with signed error bands; and
statistical composition of a materialized ensemble against its descriptor's declared law at
`CERT_Z = 5.0`.

**It must refuse:** everything in the four refusing tiers; Z0's crack path (**GrainFloor by 2.37×,
always — and no amount of GPU memory changes that**); sub-d50 crack paths and nucleation sites
generally ("`not_computable_from` in this tier's clothes", `:195`); fields within ~1 nm of a crack
tip; bond rearrangement, excitation or coordination change; damping ratio and restitution as material
constants (falsified as such, `:284`); **any claim spanning a re-root, until the re-root ledger gate
lands**; and **any claim that a df64 run reproduces the f64 reference bit-for-bit**. The refusals are
the product.

One refusal is currently *silent* and should not be. At the shipped fracture fanout of 4, **every**
statistical check sits below its resolution floor (`CERT_FLOOR_MEAN = 8`, `CERT_FLOOR_SPREAD = 32`)
and rides along unresolved — today's passing descriptor certificate is *silent*, not *supporting*. A
materializer drawing thousands of children is the first configuration where it has teeth. At Z0 the
quenched Weibull draw (3.8 / 0.42 / 0.11 µm for weak/median/strong) is what decides
certify-vs-`GrainFloor` at grain tier, so the draw is the certificate's deciding variable, not a
detail.

---

## 8. Gap list

| # | gap | scope |
|---|---|---|
| **G0** | **The use case must be named before any code.** Real-time sandbox: **refuted** (§0, 4.2× available against 36.2× needed). Batch throughput across ~1e6 scenes: viable. Deeper single scene: does not lift `GrainFloor`, which is a claim property. | decision, and §0 answers two of three |
| **G1** | **No contact solver in the GPU throughput model.** §0's 4.2× uses the measured single-engine rate against the implied wasm rate; a real solver changes both sides. The width argument (4.7 warps vs 304 slots) is **independent of solver cost** and is what carries the negative recommendation. | measure only if G0 says batch |
| **G2** | **Materialization cannot run on device** — the draw's `log`/`cos`/`pow` are not bit-identical between `libm` and libdevice. Keep host-side, or supply bit-exact software transcendentals. | host-side is the cheap answer |
| **G3** | **No GPU backend exists.** If built: a separate crate `crates/holon-gpu` **outside the workspace** (the `engine-compare` precedent) so the core's `no_std`/zero-allocation/isolation gates stay valid. CUDA via `cust`, not wgpu: **WGSL has no f64 at all**, which forecloses D1 outright. | new crate, no core deps |
| **G4** | **Nothing certifies a join across a re-root.** The re-root ledger gate is **PENDING, in flight**. Until it lands the first swarm must be single-tier. | blocks multi-tier sharding |
| **G5** | `FractureModel`/`ImpactModel` hold `Rc<RefCell<WallChart>>` — **`!Send`**, so they cannot cross a thread today, let alone a device. Every solve also allocates fresh `Vec`s; `incremental.rs::Workspace` is the only reusable-workspace precedent. | refactor to a passed workspace |
| **G6** | **CLOSED** at `708b67d` — `ledger_for` now takes `LeafWrites`, returns the binding `Lane`, and reports which lane binds even when nothing overflows. | done |
| **G7** | `tier.rs`'s declared 6.2208e8 corresponds to fill 0.36 while the tier declares fill 0.45 (implying 7.776e8); no test asserts they agree. (The 6.60e8-vs-6.2208e8 difference is *not* a defect — different declared boxes.) Also `three_tiers_refuse_and_each_names_why` asserts `== 4`. | reported to that lane |
| **G8** | **`certify_runtime_adaptive` is cubic**, for three named reasons: one refinement per O(active) evaluation (quadratic per descent), a full restart per materialization (the third power), and `RuntimeFrontier::refine` scanning all `arena.len()` though `materialize` already returns the child range. The lane's in-crate replacement, equivalence asserted, measures **30,742 ms → 2.21 ms at 4,204 active cells — 13,935×**. **Do not use the shipped entry point's scaling in any GPU budget; it dominates everything.** | the real constraint, and largely solved |
| **G9** | **PENDING measurements owed:** settled-pile and pour resident sets. The 5–50× pour guess is the lane's and is used nowhere here. | unblocks a real residency budget |

---

## 9. The design in one paragraph

Do not build a GPU backend to make the sandbox real-time; the measured frontier is 149 holons — 4.7
warps against 304 warp slots — so the card offers ~4.2× where 36.2× is needed, and the shortfall is
width and step count, not arithmetic. Build one only for **batch throughput**: ~1.0e6 concurrent
149-holon scenes, wide enough at 1.49e8 holons per step, each still at 2.76% of real. If built:
shard into arenas of a few hundred holons, each with its own declared g0 and root, related by pairs
of boundary indices — values, not a new class; **single-tier only until the re-root ledger gate
lands, because nothing today certifies a join across a re-root**. One persistent kernel, one block
per engine, holons SoA (4.0× over the 56 B AoS header). Step interiors in **native f64** with strict
intrinsics — the 1/64 penalty is irrelevant here and it buys back a fourth bit-exact target. Keep the
REG+ ledger in integers, where associativity makes determinism free, and remember the cap is
**0.59 grains** in general and 3.54 only for a chart that writes neither occupancy nor momentum.
Exchange by snapshot-then-apply over disjoint pairs with no atomics, and certify the boundary by
**re-planning the transfer**, because balance-based gates provably miss transfers doubled or dropped
on both sides. Balance shards by **claim, not geometry** — the demand a tier makes is a function of
what it claims. Then refuse, loudly, every question below d50, and every question spanning a re-root.
Before any of it, spend the effort on the certifier: 13,935× is already measured on the CPU, and it
is worth more than this entire card.
