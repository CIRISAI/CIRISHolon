# SATURATION-1 — results

*The record for `SATURATION1_PREREG.md` (frozen 2026-08-28, commit 7f47d76) as
amended by AMENDMENT A1 (commit 482da68). Gates R1, T1, T2, C1, D1 and plants
(i), (ii), (iii) are the engine lane's; F1 is not.*

---

## D1 — THE PROTOCOL, FROZEN

*Written and committed BEFORE the MBE3 arm was run or its output looked at, as
the prereg requires. Everything below is a `const` in
`engine/crates/holon-render/examples/quench.rs`, not a flag, so a reported run
re-runs byte for byte.*

### The scene

| | |
|---|---|
| atoms | 16 (`MAX_ATOMS`) |
| dimensions | 2 — the `z = depth/2` slice, the scene the field screenshots were taken in |
| box | 40 × 24 bohr, soft quadratic walls (`Boundary::Walls`), wall inset 0.6 bohr |
| opening positions | a 4 × 4 lattice at `(w(col+½)/4, h(row+½)/4)` with a per-seed uniform jitter of ±0.8 bohr — every opening separation is then outside the repulsive wall |
| opening velocities | Box–Muller Gaussians from the same seeded stream at `T_init = 3000 K`, with the net momentum removed (the box has walls; a drifting scene would heat itself against them) |
| thermostat | ON from the first step, Berendsen, `T_target = 300 K`, `tau = 2000` atomic time units |
| integration | 20,000 grain boundaries × 64 substeps = 1,280,000 substeps; `dt = 1.0769` a.u. derived from the curve, so 33.4 ps of sim time |
| RNG | one LCG (`x = 6364136223846793005 x + 1442695040888963407`, top 53 bits) seeded per run; nothing else is random |

### The eight staked seeds

```
0x0000000053415401  0x0000000053415402  0x0000000053415403  0x0000000053415404
0x0000000053415405  0x0000000053415406  0x0000000053415407  0x0000000053415408
```

Plant (iii)'s spot check uses the first two.

### The measurement rule

Taken at the final grain boundary, from `Sim::cluster_sizes` — connected
components of the bonded-pair graph, one union-find over the one edge set the
headline `Sim::cluster_count` already reads. No new criterion and no distance
cutoff: an edge exists exactly where the pair layer says `bonded`.

* a component of ONE atom is a **free atom**, not a cluster;
* **largest** = the size of the biggest component of size ≥ 2, or 0 if there is none;
* **modal** = the most common size among components of size ≥ 2, ties broken
  toward the SMALLER size;
* the full histogram is published either way.

### The two criteria, and what each decides

* **CONTROL (pair-only arm)**: `largest ≥ 8` in ≥ 6 of 8 seeds. If it fails the
  gate is VOID — protocol, not physics, per the detector-not-verdict rule — and
  the protocol is not re-tuned afterwards.
* **BRANCH (a) (MBE3 arm)**: `modal == 2` AND `largest ≤ 4`, in ≥ 6 of 8 seeds.
* **BRANCH (b)**: anything else, reported and investigated as a finding about
  the in-model three-body surface, not massaged.

Both arms also report the energy-drift and momentum-residual ratios against
their own derived bounds, per seed.

### Plant (iii)

`dE3` is zeroed at every table node whose triangle has perimeter below **4.0
bohr** (`TrimerTable::zero_inside_perimeter`), and the MBE3 arm is re-run on the
two staked seeds. The plant is scored on the D1 outcome shifting back toward the
droplet.

### DISCLOSED: what was seen before this freeze

Three protocol variants were run on the PAIR-ONLY control while sizing it. The
MBE3 arm was not run and its output was not looked at. The variants and their
control readings:

| frames × substeps | tau | control (largest ≥ 8) |
|---|---|---|
| 3,000 × 24 | 2000 | 2 / 8 |
| 4,500 × 64 | 500 | 0 / 8 |
| **20,000 × 64** | **2000** | 3 / 3 on a three-seed spot check → frozen |

What the first two showed, and why the third is the one: sixteen atoms in this
box need ~10⁴ substeps to diffuse one nearest-neighbour spacing, so 72,000
substeps is a partly-coalesced gas rather than a quench, and a fast thermostat
(`tau = 500`) makes it worse — it freezes the atoms into whatever local cluster
they are in before those clusters can find each other. Neither reading is about
the three-body term; both are the protocol failing to reach an endpoint, which
is exactly what a control is for.
