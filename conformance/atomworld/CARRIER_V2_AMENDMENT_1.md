# CARRIER v2 — AMENDMENT 1: the density stake fired, and the controlled quantity was the wrong one

*Written 2026-09-02, **POST-DATA and labelled so in its first line**. It does NOT edit
`CARRIER_V2_PREREG.md` and does not move any threshold in it. `CARRIER_V2_PREREG.md` §4.2's
density stake FIRED; this amendment records what fired it, states what the frozen protocol's
controlled quantity actually is, and freezes a second arm BEFORE that arm's instrument
exists. Frozen 2026-09-02.*

**misfits:** M-VACUOUS-SUCCESS, M-EXIT-DISCRIMINATOR, M-PLANT-OBS, M-PLANT-SECTOR,
M-PROVENANCE-OVERREACH, M-CHEAPER-THAN-ITS-PRICE, M-PLACEMENT-LOTTERY, M-DEVICE-CLASS,
M-STALE-INSTRUMENT, M-VOLUME-SCALE, M-HOMOG, M-UNTESTED-GAP, M-TAG-AS-PROPERTY,
M-NONBIJECTIVE-STEP, M-FIXED-POINT-TRAJECTORY, M-BUDGET-LAUNDER, M-CONJUNCTION-MONOTONE,
M-MAX-OVER-SUCCESSES, M-PRESENTATION-VERDICT, M-BASE-RATE-OMITTED, M-SORTS-NOT-SEPARATES,
M-LOOP-BLIND, M-BARE-CHARGE, M-COND-PROBE, M-MAINTENANCE-LENS, M-PROBE-THE-RESOURCE,
M-IDLE-CALIBRATED-TIMEOUT, M-CACHE-KIND, M-IMPORT-EXECUTES, M-ONE-MODEL-DELTA,
M-PARITY-PROTECT, M-NULL-MISSTAKE, M-POPULATION-CHOICE, M-FINAL-VIEW-COLLISIONS.

---

## A1.0 THE FIRED STAKE, REPORTED AS PLAINLY AS A SURVIVAL

`CARRIER_V2_PREREG.md` §4.2 fixed the scene at **liquid water's atom number density,
0.014860 atoms/bohr³**, and placed the atoms on a uniform cubic lattice.

**On the converged 96-knot curves, at N = 402, that scene reaches T = 241,001 K by frame
500 of a run whose thermostat target is 300 K.** `carrier3d_produce_armA.log`.

It is not an integration failure and it is not a curve artifact, and both alternatives were
checked rather than assumed:

* the energy ledger is **inside its bound** — drift 2.47e-3 against 5.17e0;
* the momentum ledger **closes** — residual 1.70e-10 against a bound of 1.78e-6;
* the discriminator on the curve: an earlier smoke run reached 83,087 K on a deliberately
  cheap 12-knot NOT-CONVERGED O–O curve, which alone proves nothing. **This run used the
  96-knot curve whose worst residual (4.81e-6) is the same order as the one under every
  banked census trajectory (2.68e-6).** Same regime, same result. The temperature is the
  physics of the scene, not the arithmetic of the solver.

## A1.1 WHY — and the controlled quantity the frozen protocol actually holds

At N = 402 the freeze's density gives a 30.02 bohr box and a lattice of
`side = ceil(402^(1/3)) = 8`, so the **nearest-neighbour spacing is 30.02 / 8 = 3.75 bohr**.

The census protocol places 12 atoms on a 4×3 lattice in a 34.6 × 20.8 bohr box, so ITS
nearest-neighbour spacing is **20.8 / 3 = 6.93 bohr**.

> **3.75 against 6.93 — the ladder's scene is 1.85× tighter, and 6.3× denser in the
> neighbourhood that decides whether a pair is bonded.**

Every atom therefore opens inside its neighbours' wells: the pair minima are H–H 1.39,
O–H 1.99 and O–O 2.44 bohr with depths 0.12–0.20 Ha, and 3.75 bohr is comfortably inside
the attractive region of all three. **A uniform ATOM lattice at liquid ATOM density is not a
liquid. It is a supersaturated covalent solid**, and its binding energy arrives as heat the
moment it is released. The order of magnitude checks: ~3 bonds per atom at ~0.1 Ha gives
~0.3 Ha per atom, or `2 × 0.3 / (3 k_B) ≈ 6e4` K, against 2.4e5 K measured after the
release has run for 500 frames.

**The mistake the freeze made is nameable, and it is not "the number was too big".** It
fixed the DENSITY, and the certified protocol's controlled quantity is the **lattice
spacing**. Those coincide in a fixed dimension; they do not coincide across a change from
2D to 3D, which is precisely the change this campaign makes. Holding density fixed while
moving from a plane to a volume tightens the nearest-neighbour distance, and nothing in the
freeze noticed.

## A1.2 ARM B — frozen here, before its instrument exists

Arm A (the frozen density) is **kept, marked FIRED, and its log is banked.** Its cost
readings stand: `W_pair` counts depend on the box and the cutoff, not on whether the scene
is physically sensible, so `CARRIER_V2_RESULTS.md` §6's ladder is not withdrawn.

Arm B changes exactly one thing:

> **The lattice spacing is the census protocol's own, `20.8 / 3 = 6.9333… bohr`, written as
> that quotient rather than as a rounded decimal so its provenance is visible in the source.
> The box edge is `side × spacing`, and the density is whatever that implies and is
> REPORTED rather than declared.**

Nothing else moves: same seeds, same T_init/T_target/tau, same substeps, same jitter, same
curves, same 2:1 stoichiometry, same dE₄ ON, same pair floor.

### A1.2.1 The gates, restated for arm B

- **A1 — the scene does not explode.** At the largest produced N, the temperature at frame
  500 is below **5000 K**. This is a deliberately loose bar: the point is to separate "a
  hot quench that will settle" from "a released covalent solid", and the two differ by
  nearly two orders of magnitude, so a bar anywhere between them decides the same way.
  Arm A read 241,001. witness: `none (a temperature reading on a model scene; no theorem in
  lean/CIRISHolon states it and none is invented for it)`
- **A2 — the spacing is the census's, EXACTLY.** The placed nearest-neighbour spacing
  equals 20.8/3 bohr to the bit at every N. This is the one-variable claim and it is
  checkable from the source. witness: `none (a placement identity, checked in the runner)`
- **A3 — the ladder is re-priced at the new spacing before anything is produced**, and
  production runs only at a rung the new ladder priced — the same rule
  `CARRIER_V2_PREREG.md` §5 imposed and this amendment does not relax.
  witness: `none (a process rule, not a proposition)`
- **A4 — the route threshold is REPORTED, not predicted.** At the census spacing the box
  grows as `side × 6.9333`, so 3 cells per axis needs `side ≥ 66/6.9333 = 9.52`, i.e.
  `side ≥ 10`, i.e. **N > 729**. That is a prediction made here, before the run; the ladder
  measures it and both outcomes are reported. witness: `none (arithmetic on this engine's
  own cutoff, stated in advance)`

### A1.2.2 THE PLANT — A1's bar must be shown to discriminate, not merely to pass

A bar written at 5000 K passes trivially on any scene that happens to be well behaved, and a
gate that has never been seen to fail is a fence. So A1 is run as a PAIR, on the same
instrument, in the same process, differing in one argument:

| plant | carrier | sector it must be nonzero in | must |
|---|---|---|---|
| **P-A** the fired placement | arm A's own scene — the frozen density, `--rho=0.014860`, at the same N and seed as arm B | the KINETIC energy, i.e. the reported temperature at frame 500 | **EXCEED 5000 K**, reproducing the fired reading |
| **P-B** the census placement | arm B's scene, spacing 20.8/3 | the same sector | fall BELOW 5000 K |

P-A is the must-fire control and P-B is the measurement. If P-A comes back below the bar
the bar is not measuring what it claims and **A1 is VOID rather than passed** — the
temperature would then be insensitive to the placement, and the whole diagnosis in A1.1
would be wrong. If both land on the same side of the bar, that is the same VOID.

Reported as a pair, always, with both numbers, so no reader has to take the bar's word for
which side the physics is on.

### A1.2.3 What arm B is expected to buy, staked before it is measured

If A4 holds, the census spacing puts the `O(N)` route threshold at **N ≈ 730** instead of
arm A's 4,273 — because at fixed N a sparser scene has a BIGGER box, and the route cares
about the box against a fixed 22 bohr radius. `RUNG2_RESULTS.md`'s scissor bar is N ≥ 800
for a 2×2×2 chart grid at 100 atoms/cell. **800 > 730**, so at the census's own spacing the
scissor-meeting size and the route-engaging size are the same regime, and a single carrier
can satisfy both.

That is a prediction, not a result. If the measured threshold lands above 800 the
convenience disappears and the amendment says so.

## A1.3 WHAT THIS AMENDMENT DOES NOT DO

* It does not withdraw arm A, edit the freeze, or move a threshold in it. Arm A is kept and
  marked FIRED, which is what the discipline requires of a dead stake.
* It does not claim arm B produces water, or any chemistry at all. It claims arm B produces
  a scene that is a QUENCH rather than a detonation, which is a precondition for the
  successor's question and not an answer to it.
* It does not re-open `CARRIER_V2_RESULTS.md` §6's F1/F2 verdicts. Those are readings about
  the engine's cutoff arithmetic and they are true of both arms.
* It carries no timing claim. The host is shared and loaded; `M-PLACEMENT-LOTTERY` and
  `M-DEVICE-CLASS` are contacted and not discharged.
