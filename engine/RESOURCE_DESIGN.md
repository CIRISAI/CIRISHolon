# RESOURCE_DESIGN — an allocation is a child holon, and its lifecycle is the rent clause

Status: **design, no code**. Numbers marked **HARD** are measured on this machine and named
with their instrument; **PENDING** are owed and are used in no argument below.
Scope: in-process resource discovery, leasing and dispatch — cores, RAM, VRAM, disk, worker
pools, table-generation shards. No transport, no cross-machine scheduling, no consent layer.
Frame: `INTEGRATION_FRAME.md` — one holon, values only.
Banked machinery this rests on: `CIRISOntology/Core/Maintenance.lean` (`rent_holds`,
`underpaid_shrinks`, `unpaid_decays`), `lean/CIRISHolon/MergeLaw.lean` (`shardedFold_invariant`,
`digest_convicts`), the tuner's Hold/Degrade contract, `MESH_DESIGN.md`.
Date: 2026-08-30. Revised the same day with the lead's lease ruling (D3), the
precision-as-resource case (§2.3, D3b), and the misfit contacts (§7b).

*The self-similarity is the design's argument: this is already how the team runs its own
lanes. A lane is commissioned when there is work, probed against the tree it is about to touch,
carries its commits as receipts, and is stood down on closure with the record as the release.
A lane nobody stood down and nobody heard from is exactly the leaked lease this document is
about.*

---

## 0. The recommendation, and the hazard that shapes it

**Every allocation is a child holon: probed at birth, ledgered while it lives, released when the
need ends, and reclaimed if the rent stops.** Discovery is a hint and never a licence.

But the headline requirement — *dispatch prefers GPU wherever a registered, determinism-gated
kernel wins at that size* — carries a hazard that has to be designed for rather than discovered,
and it is measured, not hypothetical:

> **AUTOMATIC DISPATCH MAKES NUMERICAL OUTPUT A FUNCTION OF A PERFORMANCE MEASUREMENT.**
>
> G2 measured the `(O,O,O)` sigma kernel on both devices. They agree to **3.033e-15** relative
> — and **91.0% of the 207,025 entries differ BITWISE** (HARD, `scratchpad/s3gpu/sigma.cu`
> against `sigma_direct`). Both answers are correct. They are not the same bits.
>
> So a dispatcher that sends a workload to the GPU *above* a size crossover and to the CPU
> *below* it makes the last bits of every result depend on where that crossover sits — and the
> crossover is a **measured** quantity that moves with load, driver version, and cuBLAS kernel
> selection. A table generated on a busy afternoon would differ from the same table generated
> on a quiet one, and every bit-identity gate in the engine (G1's, `holon-mesh`'s,
> SATURATION-2's committed 105,105-node table) would fail for a reason that is not a defect.

The resolution is a rule, stated here before any code:

**D0 — DEVICE CLASS IS PART OF THE ARTIFACT, NOT OF THE SCHEDULE.** A workload whose output is
gated on bit-identity declares a device class, and dispatch may choose freely only *within* it.
Dispatch may never silently change the device class of a bit-gated workload. Where a workload is
not bit-gated (a benchmark, a search heuristic, a render), dispatch is free and the chosen device
is logged with the result.

This is the same shape as G1's own resolution — chunking is part of the table's *definition*,
assignment is part of the *run* — applied one level up. Anything that reaches the numbers belongs
to the artifact; anything that only reaches the wall-clock belongs to the schedule.

---

## 1. The object: a lease is a child holon

```
    parent holon
        |
        +-- lease(VRAM, 512 MB)     <- child, probed at birth, ledgered, released
        |       |
        |       +-- lease(worker, 1)  <- recursion: a sharding spawns child leases
        |       +-- lease(worker, 1)
        |
        +-- lease(RAM, 2 GB)
```

A lease carries, from birth: **what** (kind and amount), **who** (the owning holon), **the probe
result that admitted it**, **a ledger entry**, and **a rent record**. It is not a handle; it is an
entry in the parent's books, and the parent's books are what the audit reads.

### 1.1 States

```
   PROBING ──pass──> LEASED ──in use──> ACTIVE ──need ends──> RELEASED
      │                 │                  │
      │                 │                  └──rent stops──> IDLE ──reaped──> RELEASED
      └──fail──> REFUSED                   │
                                           └──resource vanishes──> CONVICTED
```

`REFUSED` and `CONVICTED` are distinct and the distinction is load-bearing. **REFUSED** is *we
asked and the answer was no* — a normal, cheap, frequent outcome. **CONVICTED** is *we held a
valid lease and the resource went away underneath it* — a violation, and evidence of a bug or a
competing process, which the audit must see.

### 1.2 The lifecycle IS the rent clause

`Maintenance.lean` proves, on the model: pay the decay and an entry holds (`rent_holds`),
underpay and it strictly loses (`underpaid_shrinks`), pay nothing and it tends to zero
(`unpaid_decays`). That is the reclamation law, and the mapping is exact rather than decorative:

| rent clause | lease |
|---|---|
| the entry | the lease |
| paying the decay | the holder's continued *use*, which refreshes the lease |
| underpaying | partial use — the lease shrinks toward its working set |
| paying nothing | no use, no refresh: the lease decays to reclaimed |

**Scope, stated honestly:** `Maintenance.lean` is proved ABOUT THE MODEL and this is an
*instantiation of its shape*, not an inheritance of its guarantees. Nothing here is
machine-checked. That must not be advertised otherwise.

---

## 2. Probe before allocating, always

**D1 — DISCOVERY IS A HINT; THE PROBE IS THE AUTHORITY.** Init-time discovery may populate a
registry. It may never admit an allocation. Every allocation probes at the moment of use.

The warrant is one afternoon on this box, and it is worth stating because it is the whole reason
this rule is absolute: the 4090 was reported idle with 16,376 MiB free **and the root filesystem
was simultaneously at 100%, 60 MB free** (HARD, 2026-08-30). Init-time discovery would have
recorded a healthy machine. Every disk write on it was failing.

### 2.1 A probe tests the RESOURCE, not the holder

This is the subtle half and it is where a naive design goes wrong. During that disk-full window:

* a probe asking *"is the holder's process alive and scheduling?"* would have passed for every
  writer on the machine — they were all healthy, and all failing;
* a probe asking *"can I actually write a byte and remove it?"* would have failed correctly.

**D2 — A PROBE MUST ATTEMPT THE THING, or measure the headroom for the thing, and never infer
availability from the holder's liveness.** Free-VRAM before an upload; a real allocation-and-free
for RAM; a write-and-unlink for disk. Cheap, because it is about to be done for real anyway.

### 2.2 What a lease guarantees, and for how long

A probe establishes availability at time *T*. At *T + ε* it may be false — another process fills
the disk. The lead's ruling settles what to do about that, and the answer is not a better probe:

**D3 — A LEASE IS A RECEIPT FOR RENT PAID, NOT A PROMISE ABOUT THE FUTURE.** Four parts, all
binding:

1. **The probe at lease time buys validity NOW, and nothing after.**
2. **Every USE is itself a probe.** The write that fails is the authoritative reading — more
   authoritative than any check that preceded it. The ledger records the failure as a receipt,
   and the holder's response is **refuse-and-release, never retry-forever**.
3. **A lease carries a declared HORIZON** (the rent interval), after which it is stale *by
   definition* and must re-probe. This is the disk case's real answer: the probe passed and was
   false milliseconds later, and the fix is not a sharper probe but **a shorter thing the probe
   is claimed to mean**.
4. **What a lease guarantees forever is only the LEDGER ENTRY** — the record of what was granted
   and when.

The same shape as the hardware campaigns' lesson that calibrations are rented and a band frozen
from a prior epoch decays. A lease does not reserve; it records that we checked, and against what.

### 2.3 PRECISION IS A RESOURCE, and the Davidson floor is a tier boundary

The second live case study, and it is the one that shows a lease has a **quantitative** guarantee
boundary rather than only a temporal one.

An `f64` Davidson solve that exits `Stagnated` at its expansion floor has **EXHAUSTED ITS TIER**.
This is measured, not analogical: every heavy solve in SATURATION-3 exits stagnated, because
`davidson_eigh` accepts a new expansion direction only when its norm clears a hardcoded `1e-10`
after Gram-Schmidt — a **scale-free** floor, which is why residuals cluster just under `1e-10`
regardless of energy or space size (the tables lane's discrimination: an `eps·|E|·√n_det` floor
predicts an 8.6× spread across the staked combos, measured spread 1.21×).

Read in this document's vocabulary: **the lease on f64 arithmetic guarantees residuals down to
roughly the named floor and nothing below it.** So:

* a request for a deeper residual is **not a retry** and **not a constant edit** — hammering the
  same holon past its declared boundary is the arithmetic version of retry-forever, which D3(2)
  already forbids;
* it is an **overflow**, and it must probe and lease the **next-tier holon** — the high-precision
  referee route (ELEMENTS-3's route C, whose declared availability is already recorded as owed).

**D3b — A LEASE STATES ITS QUANTITATIVE BOUNDARY, THE RECEIPT RECORDS WHERE IT STOPPED, AND THE
ESCALATION PATH IS LEASED RATHER THAN IMPROVISED.** The ledger entry for an arithmetic lease
carries *down-to-the-floor, no further*; a solve that stops there has paid its rent in full and
its receipt says so; and a caller needing more routes to a different tier instead of editing a
constant.

This also explains why the floor must NOT be moved, which is otherwise a puzzling refusal: the
eigenvalue error at that floor is `~resid²/gap ≈ 1e−20` Ha, so the solves are ACCURATE and only
the label is wrong — and moving the threshold would shift every energy's trailing bits and cost
SATURATION-2's committed 105,105-node table a full regeneration. A tier boundary is part of the
artifact, exactly as D0 says a device class is.

### 2.3 Refusal is loud, degradation is stated

**D4 — NO SILENT FALLBACK.** A failed probe produces either a LOUD refusal or a *stated* Degrade
under the tuner's Hold/Degrade contract, naming what was asked, what was found, and what the
degraded path is. A silent fallback to a slower path is the vacuous-success shape: the run
completes, the number looks fine, and nothing records that the fast path was never taken.

**D5 — HALF-VISIBLE HARDWARE REFUSES.** Driver present and CUDA broken must REFUSE, not fall back
to CPU. `holon-gpu` already has the shape (`GpuError` names its failures rather than swallowing
them); the resource layer must not undo it.

---

## 3. Spawn only when required or requested

**D6 — NO SPECULATIVE POOLS, NO WARM RESERVES.** The canonical anti-pattern is banked: **63 BLAS
spin-threads** in a process pool, each a resource holon nobody requested, fighting the real work.
A pool that exists because it might be needed is a lease with no owner and no rent.

**D7 — RECURSION CARRIES A DECLARED DEPTH CAP, and exhaustion VOIDs loudly.** A workload that
shards spawns child leases, each probed at its own birth. Without a cap that is a fork bomb in our
own vocabulary. Proposed cap **4** (scene → shard → worker → kernel-allocation), which covers
every structure the engine currently has; exceeding it VOIDs the request with the lease chain in
the message, and raising it is a deliberate edit with a case attached.

---

## 4. Composition, and the one place the merge law genuinely applies

Receipts compose up the recursion, and a failed child is convicted by digest rather than silently
absorbed. **But the merge law's theorems are about an `AddCommMonoid` and they are exact** — G1
had to make exactly this distinction and it applies again here:

| quantity | composes under the merge law? |
|---|---|
| counts, bytes, integer lane digests | **yes** — an exact additive monoid, `shardedFold_invariant` applies literally |
| wall-clock seconds, throughput ratios, float utilisations | **no** — float addition is not associative; these aggregate to a *reported* number, never to a certificate |

**D8 — A RECEIPT THAT IS PART OF A CERTIFICATE MUST BE AN INTEGER.** Bytes leased, leases opened,
leases released, reapings. Anything float-valued is reported and never certified, and the ledger
must not blur the two — that would launder an integer theorem into a claim about floats.

Then `digest_convicts` gives the real guarantee: **leases opened must equal leases
released-plus-convicted**, as an exact identity over integers. A leak is a non-zero residual, and
it is a *proof* of a leak rather than a heuristic.

**D9 — DE-ALLOCATION PROPAGATES LEAF-TO-ROOT.** A parent releasing reclaims its children first;
a child convicted or reaped surfaces in the parent's ledger rather than vanishing.

---

## 5. The reaper, and the one thing it must not do

Idle reaping is the backstop for the explicit path, not a replacement for it.

**D10 — THE REAPER DISTINGUISHES IDLE FROM SLOW, AND CONVICTION REQUIRES EVIDENCE.** A missed
heartbeat is a signal to LOOK, never a verdict. Reaping something still needed turns the resource
layer into a saboteur — a strictly worse failure than the leak it is preventing.

The disk-full window is the case study again and it is the reason this rule has teeth: **every
writer on this machine would have looked dead to a timeout-based monitor**, because they were all
blocked on a full filesystem. A reaper trusting a timeout would have killed the entire machine's
work, correctly identifying that nothing was progressing and completely misdiagnosing why.

The ladder, in order, all three required before reclamation:

1. **grace period** — a multiple of the holder's own declared step time, not a global constant;
2. **second probe** — is the holder's process alive and scheduling?
3. **the discriminator** — *is anything else on this machine progressing?* If nothing is, the
   problem is the machine, not the holder, and the reaper must **stand down and say so** rather
   than reclaim. This is the check that would have saved the disk-full afternoon, and it is the
   one a naive design omits.

Every reaping is logged with all three answers. A lease with no release, no rent and no reaping
is a ledger violation the audit must surface.

---

## 6. Dispatch: three steps, each of which can refuse

**D11 — CONSULT, THEN PROBE, THEN LEASE.** Never fewer than three, each separately logged:

1. **consult** the registered measurement for this workload AT THIS SIZE — crossovers are
   per-size facts, never global ones; the tuner's Choice pattern owns the decision;
2. **probe** the device — registration is a memory, and calibrations are rented;
3. **lease** — and the lease can still be refused.

The dispatch decision is logged per workload with the measured entry it cited, so a wrong choice
is traceable to the entry that caused it rather than to a mood.

### 6.1 The first two citizens

| kernel | registration | determinism gate | status |
|---|---|---|---|
| `(O,O,O)` sigma, GPU | **65.7 sigma/s**, 318 GFLOP/s FP64 (HARD, `s3gpu/sigma.cu`) | fixed reduction order, atomics-free, 5/5 runs bit-identical | measured, **adoption deferred** |
| `(O,O,O)` sigma, CPU | **20.8 sigma/s** aggregate, 32 threads at loadavg 32 (HARD, `s3_sigma_cost.rs`) | serial per node | in use |
| table-generation shards | — | bit-identical at 1/4/8 workers (HARD, G1) | in use |

The GPU entry is the pattern the lead asked for — adoption by measurement, determinism gate
declared, refusal recorded — and it is also the first case of **D0**: it wins 3.2×, its gate
passes, and it is *not* adopted, because the workload is bit-gated and the device class is part
of the artifact.

### 6.2 The registry must not be trusted about itself

**D12 — A RUNTIME SPOT-CHECK VERIFIES REGISTRATION.** A deliberately mis-registered throughput
entry must be caught. Trust-but-verify on the engine's own bookkeeping: periodically, and on any
dispatch whose measured time misses its registered entry by more than a declared factor, re-time
the workload and CONVICT the entry.

**The plant, and it must fire before the mechanism is trusted:** register the GPU sigma at 10×
its measured throughput. Dispatch must send work to the GPU, observe ~65.7 sigma/s against a
claimed 657, and convict the entry. The carrier is asserted non-empty first — the two devices'
throughputs must differ by more than the spot-check's factor, or the plant sits in an empty sector
and VOIDs rather than passes (M-PLANT-SECTOR).

---

## 7. What this design does NOT claim

* **Nothing here is machine-checked.** The rent clause and the merge law are banked results whose
  *shape* is instantiated; the guarantees are not inherited. `Gate.mechanized` discipline applies:
  these are recorded commitments until something checks them.
* **No cross-machine scheduling**, no transport, no consent. In-process, one machine.
* **No claim that a lease reserves anything** — see D3.
* **No claim that GPU dispatch is safe for bit-gated workloads** — see D0; it is explicitly not.
* Recursion cap 4 is a **declared choice**, not a derived bound.

## 7b. Misfit contacts

*The registry is grep-armed: a freeze whose text contacts a registered misfit's keywords without
citing it is refused. Contacted here, with what each one binds:*

* **M-DEVICE-CLASS** *(born of this lane's G2, 2026-08-30)* — the whole of **D0**. A determinism
  gate proven within one device class does not compose across classes.
* **M-VACUOUS-SUCCESS** — **D4**'s no-silent-fallback rule, and the reason a failed probe must
  produce a refusal or a *stated* Degrade: a run that completes on the slow path while reporting
  nothing about the fast one is success reported for work not done.
* **M-BUDGET-LAUNDER** — recursion-cap exhaustion (**D7**) VOIDs loudly and never falls back to a
  scorable outcome.
* **M-PLANT-SECTOR** — the mis-registration plant in **§6.2** asserts its carrier non-empty (the
  two devices' throughputs must differ by more than the spot-check factor) before it is scored.
* **M-STALE-INSTRUMENT** — a registered throughput entry IS an instrument, and **D12** exists
  because a registration is a memory of a measurement rather than the measurement.
* **M-SORTS-NOT-SEPARATES / M-EXIT-DISCRIMINATOR** — **§2.3**: `Stagnated` is the normal exit for
  every heavy solve, so it ranks rather than separates, and the informative discriminator is the
  exit reason *plus* the residual against the floor *plus* the variational margin.
* **PROBE THE RESOURCE, NOT THE HOLDER** (**D2**) is proposed for registration in its own right —
  during the 2026-08-30 disk-full window every writer on this machine was healthy and failing, so
  a liveness-based probe passes exactly when it is most wrong. Offered to the registry as a
  candidate rather than self-registered.

## 8. Open questions, owed before code

1. **What refreshes a lease?** D3 settles what a lease MEANS; it does not settle the mechanism.
   "Continued use" needs a definition per resource kind, and it must be cheap enough that paying
   rent is not itself a cost worth avoiding — a heartbeat that costs more than the work is a
   resource holon nobody requested (D6).
2. **Does the reaper's step-3 discriminator have a cheap implementation?** "Is anything else
   progressing" is the right question and it is not obviously cheap to answer. **PENDING.**
3. **Where does the ledger live** so the audit can read it after a crash, without becoming a
   write amplifier on the very disk whose exhaustion started this?
4. **Is the spot-check's declared factor a constant or per-kernel?** A GPU under thermal throttle
   can legitimately miss its entry by 2×; a mis-registration by 10× must still be caught.
