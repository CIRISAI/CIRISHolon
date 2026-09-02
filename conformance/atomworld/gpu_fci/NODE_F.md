# Node F — the device class becomes part of what a table IS

*GANTT node F, gpu-prod. Increment F.1: the identity axis, the refusal, and the
manifest, wired through the CPU path end to end. F.2 is the GPU-class launcher.*

## What the receipt asked for, and where each part stands

| receipt | state |
|---|---|
| same-table bit-identity WITHIN class | **GREEN** — `g1_gate` (7) and `nd_bit_identity` (8) pass unchanged; a new test asserts 1-worker and 4-worker runs of one CPU-class spec agree bit-for-bit with equal digests, carrier asserted |
| device class in every manifest | **GREEN for the generators** (`s3_tables`, `de4_table`) and for pair curves; the SHIPPED tables predate the axis — see the gap below |
| throughput registered as the ROUND-TRIP quantity with between-invocation spread | **DONE at `d42768d`** — 61.619 ± 4.927 over twelve invocations at loadavg 78–110 |

## The three-axis law, as implemented

A table's identity is three things, and they are one identity rather than three
diagnostics. Each was learned the same way: a number moved and nothing recorded
which regime made it.

| axis | what it fixes | how it was learned |
|---|---|---|
| **device class** | which arithmetic produced the bits | G2: the classes agree to 3.033e-15 and **91.0% of 207,025 entries differ BITWISE** |
| **solver budget** | which regime the solves ran under | a silent 1200 → 4000 default change put artifacts either side under different regimes |
| **subtraction basis** | what the stored number is a residual OF | a four-body residual read as a total is wrong by the whole of MBE3 |

`Surface::basis()` is **required, not defaulted**. A default of "total" would let
a surface that DOES subtract inherit a manifest line saying it does not — and
`OhhhSurface` is the only surface here that subtracts, which is exactly the case
the method exists for. Making it required turned the compiler into the audit:
it named both implementors and the test fixture the moment the method landed.

## The refusal, and where it moved to

`--device gpu` is REFUSED by name with its exit, never downgraded:

    REFUSED: --device gpu: this binary cannot generate a GPU-class table. It links
    holon-tables, which does not link CUDA by design. Use holon-gpu's device-class
    launcher, which supplies the GPU provider. REFUSED rather than run on the CPU:
    a table stamped `gpu` that a CPU produced would pass every gate and be wrong.

An unknown class is refused rather than defaulted, because defaulting would stamp
this build's class onto an artifact the caller meant for another one.

**The refusal moved during construction, and the reason is worth keeping.** It
first lived at the solve site, where it fired correctly and uselessly: N workers
each raised the right message inside a scoped thread, and what reached the caller
was `a scoped thread panicked`. A loud refusal that names what was asked, what
was found and the exit is worth nothing if the naming is swallowed by the thread
that did it. It is now refused once, at the generator's entry, before a worker
exists. The per-node check is KEPT — but it now checks the class on the
`Solution` that came BACK, which is a different failure: a provider that lied.

## The gap this increment does NOT close

**The shipped pair tables in `docs/atoms/tables/` declare no device class.** They
carry `solver_budget_iterations` (the sibling lane regenerated them when that
axis landed) and no `device_class`, because they predate this one. Every one of
them is CPU-produced — there was no GPU path when they were made — so the answer
is knowable with certainty, and the artifact still does not say it.

They are NOT hand-annotated, and that refusal is the point: an artifact edited by
hand is no longer the output of the code that claims to produce it, which is what
`water.rs`'s `the_committed_table_is_this_build_s_own_output` exists to catch.
Closing it means a deliberate regeneration, which is its own act with its own
review — the same way the budget axis was closed.

## What F.2 has to face, measured rather than assumed

GPU-class table generation will not be a drop-in speedup, and the two reasons are
already measured:

* **the sigma is 4% of a Davidson iteration** — 14.7 ms against 410 ms at the
  `(O,O,O)` scale; the host driver is the other 96%. So Amdahl caps a whole-table
  speedup near **3%** even with the device free, and 30 workers sharing one device
  serialise that 4% among themselves, which can make it negative.

**CORRECTION, and it removes the second reason entirely.** This section first
said worker count was bounded by VRAM at "roughly 2–3 workers on this card
against 32 CPU workers". **That was arithmetic run backwards** — 16 GB over
0.5 GB is 32, not 2 — and it was repeated into two lanes' inboxes and this
document before anything measured it. The measurement:

    (O,O,O)-shaped, 207,025 det:  480.5 MiB per worker
    card:                          15,683 MiB free
    with a 1 GiB reserve:          30 workers fit

**VRAM is not what stops GPU-class table generation.** The conclusion survives —
the device arm helps a SINGLE large solve and does not help a table — but on ONE
leg instead of two, and the leg that failed is the one I had stated most
confidently. It is now pinned by
`gpu_lease.rs::the_gpu_worker_bound_is_derived_from_vram_and_bounds_in_both_directions`,
which derives the bound from the live card rather than from a sentence.

Saying this here, before F.2 is built, is still cheaper than discovering it
after — and the correction is the better argument for saying it early.
