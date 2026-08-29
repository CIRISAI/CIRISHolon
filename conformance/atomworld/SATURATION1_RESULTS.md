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

---

## The instrument

The three-body surface `dE3(s1, s2, s3)` is computed by STO-3G full CI at every
node of its own grid and interpolated; forces come from differentiating the
interpolant analytically, so the dynamics' energy function IS the tabulated
surface and conservation holds against it rather than against the model the
table approximates. `engine/crates/holon-chem/src/trimer.rs`.

| | |
|---|---|
| coordinates | `(x, y, c)` — the two SHORTEST sorted sides and `c = sqrt(1 − u)`, `u` the cosine of the angle between them |
| grid | 33 × 33 × 13 = 14,157 nodes, 7,293 unique electronic-structure solves (the `x ↔ y` symmetry is exact, so the mirror node is the SAME float) |
| side axis | `r = 0.7 + 8.3 (e^{2τ} − 1)/(e² − 1)`, τ uniform |
| domain | `a ≥ 0.9`, `b ≤ 9.0` on the sorted sides, `c ≤ a + b` — AMENDMENT A1's shape |
| interpolant | tensor-product Catmull-Rom, C1 by construction, node values only |
| peak `|dE3|` | 1.500881 Ha, at the compact corner |
| curvature envelope | 46.934 Ha/bohr² absolute, 51.039 /bohr local, both widened 4× |
| sort kink | 4.018e-2 Ha/bohr (see C1) |

### Why those coordinates, measured

The third coordinate is the one that mattered, and the sweep
(`examples/mbe3_grid_sweep.rs`) is the record. Held-out max error at 41×41×13:

| third coordinate | max error |
|---|---|
| `c = sqrt(1 − u)` | **3.8e-5 Ha** |
| raw cosine `u` | 1.9e-4 Ha |
| `w = (z − |x−y|)/(2 min(x,y))`, linear in `z` | 4.4e-4 Ha, and it does not improve with refinement |

At `x = y` the third side is exactly `z = x √2 c`, so a uniform `c` grid is a
uniform `z` grid where the surface is steepest, while a uniform `u` grid is
uniform in `z²` — coarsest precisely there. The `w` coordinate is linear in `z`
but kinks on the `x = y` diagonal, and that kink is a floor no grid clears.

The side axis's stretch was measured the same way, at 41 knots: `a = 2` gives
3.4e-5 Ha, `a = 3` gives 4.7e-5, uniform-in-`r` gives 5.3e-4. It is exponential
rather than a power law because `dτ/dr` must stay finite at the lower edge — a
coordinate singularity there would be a force singularity.

### The second implementation, and what pays for it

`pair::solve_geometry` costs ~5 ms per H3 point on this machine: its Hermite `E`
tables and `R` tensors are sized for p functions and carried in second-order dual
numbers. A table needs 7,293 of them at load, in a browser.
`trimer::hydrogen_energy` is a value-only, s-only, allocation-free path — closed
forms transcribed from `sto3g.rs` into three dimensions, a fixed 3×3 Cholesky
orthonormaliser, and a nine-determinant CI built by applying ladder operators so
there is no Slater–Condon case analysis to get wrong. It is ~25× faster.

It is a SECOND implementation of one model, which is a cost, and it is paid for
by `tests/trimer.rs::the_fast_path_agrees_with_the_general_n_centre_route`:
**9.77e-15 Ha** worst over a set spanning the domain, against a 1e-12 stake — so
the 50-digit referee chain reaches this path through a gate rather than around
it. Its pair restriction also agrees with the banked `h2_point` route to
2.22e-15 Ha.

---

## R1 — the 50-digit trimer referee · **HOLDS**

`tests/trimer.rs::r1_the_trimer_matches_the_fifty_digit_referee`, live (not
`#[ignore]`d), against `conformance/atomworld/h3_referee.json` pinned by FNV-1a
digest `0xd5b107ba`.

| | measured | stake |
|---|---|---|
| max `|E(H3) − referee|` | **8.99e-15 Ha** at (1.6, 1.6, 1.6) | 1e-10 |
| max `|dE3 − referee|` | **8.11e-15 Ha** at (7.0, 0.9, 7.0) | 1e-10 |

75 geometries, 11,118× inside the stake. The referee shares no code, no language
and no arithmetic with the engine — only the model definition.

The prereg's three DISCLOSED priors are reproduced by the table-building path:
equilateral at r_e **+0.858071**, linear **+0.354728**, H2+H at 2 bohr
**+0.216860**. The far field reads −3.55e-15 Ha at 20 bohr and −8.88e-16 at 40 —
asserted as an f64 FLOOR, never as a literal zero, per AMENDMENT A1's precision
fence: the true equilateral tail is spin frustration at 3J/2 ≈ +4.4e-29 Ha,
thirteen decades below anything f64 carries through this cancellation.

---

## T1 — interpolant fidelity, held out · **HOLDS**

256 pseudo-random geometries, staked seed `0x5341545552415431` written in the
test, drawn inside the domain, none on grid nodes (checked explicitly before the
gate, as the prereg's VOID condition).

| | |
|---|---|
| max `|interpolant − direct FCI|` | **4.6005e-5 Ha** |
| kill | 1e-3 Ha → **21.7× margin** |
| rms | 7.0390e-6 Ha |
| worst at | (s1, s2, s3) = (1.258, 1.725, 1.900) |
| exact-zero errors | 0 of 256 — the two-sided condition is satisfied |

4.6005e-5 Ha is the successor's stake.

---

## T2 — the truncation systematic · **FIRED as staked, HOLDS as amended**

Both readings are on the record, and both are tests.

**As the freeze staked it** (`t2_the_originally_staked_longest_side_shell_fires`)
— max `|dE3|` on the "any side at 7.0 bohr" shell: **1.7720e-2 Ha** at
(3.54, 3.54, 7.00), against a 1e-5 kill. It fires by **1772×**. That test asserts
its own firing, so a later change that quietly made it pass would fail there and
be looked at.

The diagnosis, which AMENDMENT A1 records and this lane measured independently:
`dE3` vanishes only when one atom is far from BOTH others, which for sorted sides
is a statement about the SECOND-smallest side, not the longest. A near-collinear
chain's longest side is the sum of two short ones and is not a distance anything
decays over. Measured on the b-shell here: 6.4e-5 Ha at b = 7, 6.0e-6 at b = 8,
4.7e-7 at b = 9.

**As amended** (`t2_the_shipped_truncation_systematic`) — max `|dE3|` on the
`b = 9.0` shell, which is where this table actually stops reading:
**4.6758e-7 Ha** at (0.90, 9.00, 9.00), against the unchanged 1e-5 kill — a
**21× margin**. The named worst-case instrument, the collinear probe (b, b, 2b),
reads 1.3772e-7 Ha.

---

## C1 — conservation with the third body paying · **HOLDS**

One gate per conservation law, never combined, each against its own derived
bound. `tests/saturation.rs`.

| scene | energy drift / bound | momentum residual / bound |
|---|---|---|
| 3 atoms, open box, pure NVE, 400 × 64 | **4.1e-4** | **3e-4** |
| 8 atoms, walls + thermostat, 400 × 64 | **4.2e-3** | **5e-4** |
| 16 atoms, the full D1 quench (below) | see the D1 tables | see the D1 tables |

The NVE scene carries a peak `|E_three|` of 0.363458 Eh and injects exactly zero
external work; the thermostatted scene's `W_ext` is −0.53 Eh and on the ledger.
`k_three` measured 59.99 and 30.59 Ha/bohr² respectively.

The precondition, measured rather than assumed: the triple force is minus the
gradient of the triple energy to **8.09e-10** relative
(`the_triple_force_is_minus_the_gradient_of_the_triple_energy`), so the gate is
reading integration error and not an inconsistency. And with no table loaded the
term is EXACTLY absent — a two-atom scene's energy, drift peak and drift bound
are bit-identical with and without a three-body table, so every gate written
before this campaign reads the float it always did.

### The drift bound's three-body term, and two corrections it needed

Per triple, with `E = F(s_a, s_b, s_c)`:

```
|d2F/dx_i^2| <= 4 G2 + 2 sum_a |F_a| / s_a
```

— atom `i` touches two of the three sides, `|ds_a/dx_i| <= 1` because each is a
component of a unit vector, and `||d2 s_a/dx_i^2|| <= 2/s_a` for a distance. The
sum runs over all three sides rather than the two at `i`, which only widens it.

Two things the first version got wrong, both found by looking at the number
rather than the derivation:

1. **The envelope was measuring a kink, not a curvature.** `eval` composes the
   interpolant with a SORT, and a sort is not differentiable where two sides
   cross. A finite difference straddling it reported 1283 Ha/bohr² at h = 1e-4,
   152 at 1e-3, 37 at 1e-2 — the signature of a jump. Sampled on ONE branch it is
   11.7 (46.9 after the 4× widening). The kink is real and is now MEASURED
   instead of swept up: **4.018e-2 Ha/bohr**, which is the price of an
   interpolant exactly symmetric in its first two arguments and only
   symmetric-to-interpolation-error in the third. The potential stays continuous
   — the sorted triple is a continuous function of the unsorted one — so it stays
   conservative; what the kink costs is a small force discontinuity, and the C1
   ratios above are what that cost measures out at.
2. **The count was a worst case that could not fail.** `C(n−1, 2)` triples all
   simultaneously at the table's global maximum gave 2.2e6 Ha/bohr², put `ω·dt`
   past the stability limit, and would have passed whatever the integrator did.
   The loop now accumulates the per-triple bound into a PER-ATOM total and keeps
   the largest, and the per-triple curvature takes the smaller of the absolute
   cap and the local one (`51.039/bohr × the triple's own gradient`). A dispersed
   scene then reads a small stiffness because it has small gradients, which is
   the fact a live bound is supposed to know.

---

## Plant (i) — the sign flip · **CAUGHT**

`tests/trimer.rs::plant_i_the_sign_flip_inverts_saturation`. The carrier is
AMENDMENT A1's corrected one, and the engine reproduces it: E(H4 tetrahedron,
edge r_e) = −1.112022505 Ha against 2 × E(H2) = −2.274612102, so two dimers win
by **+1.162590 Ha** — not the +0.426 the feasibility probe reported for a
geometry it had mislabelled. The H4 energies go through the general N-centre
route in the `Sz = 0` block, A1's pinned convention.

As the MBE3 sandbox itself sees the comparison — pairs from the curve, triples
from the table — the gap reads **+2.573423 Ha**; with the table NEGATED it reads
**−4.206562 Ha**. The comparison inverts. (The gap MBE3 reads is larger than the
exact one because order 4 opposes order 3; A1 records the referee's ratios at
0.36–0.48 of the triple sum, and 2.573 − 1.163 = 1.41 Ha against a triple sum of
3.43 Ha is 0.41 of it — the same number from the other side.)

## Plant (ii) — the symmetry plant · **CAUGHT**

`tests/trimer.rs::plant_ii_symmetry_and_its_deliberate_break`. Carrier: dE3 at
the staked scalene geometry (1.237, 2.041, 2.713) = **+2.960778768e-1 Ha**,
nonzero. All six permutations agree **bit-for-bit**, value AND gradient — the
evaluation sorts first and floating-point comparison is exact, so this is not a
tolerance. A deliberately desymmetrised table (one node moved by 1e-3 Ha) moves
the reading by **7.825e-4 Ha**, past the staked 1e-6.

## Plant (iii) — the far field

The structural half is a test
(`tests/saturation.rs::plant_iii_the_force_loop_reads_the_zeroed_region`): dE3 at
a compact triple of perimeter 3.6 bohr goes **+0.998862 → −0.008134 Ha** under
the plant, `E_three` on a three-atom scene goes +0.998370 → −0.008512, and the
force on the third atom moves by **0.1595 Ha/bohr**. The sector is not empty and
the force loop reads it.

The D1 spot check is below.

---

## D1 — THE PRODUCT: molecules · **BRANCH (a), 8 of 8 seeds**

Both arms ran the frozen protocol above, unchanged, on the eight staked seeds.

### The control: pair-only

```
seed              largest modal clusters free  hist(2..)      T(K)    drift/bound  dP/bound
0x...53415401           9     7        2    0  1x7 1x9        253.8      0.0045    0.0001
0x...53415402           9     7        2    0  1x7 1x9        293.0      0.0075    0.0000
0x...53415403          15    15        1    1  1x15           356.6      0.0115    0.0001
0x...53415404           8     3        3    0  1x3 1x5 1x8    273.7      0.0051    0.0001
0x...53415405           8     3        3    0  1x3 1x5 1x8    296.8      0.0051    0.0001
0x...53415406          11     4        2    1  1x4 1x11       321.6      0.0096    0.0002
0x...53415407           9     7        2    0  1x7 1x9        316.1      0.0064    0.0000
0x...53415408          10     6        2    0  1x6 1x10       363.2      0.0057    0.0002
```

**CONTROL criterion (`largest ≥ 8`): 8 / 8.** The gate is not VOID. The field
droplet reproduces: sixteen hydrogens with a pairwise-additive force loop
condense, one seed to a single cluster of fifteen.

### The intervention: MBE3

```
seed              largest modal clusters free  hist(2..)   T(K)   E_three   drift/bound  dP/bound
0x...53415401           4     2        7    0  6x2 1x4     303.0  +0.00095    0.0035     0.0000
0x...53415402           2     2        8    0  8x2         307.0  +0.00070    0.0064     0.0000
0x...53415403           4     2        7    0  6x2 1x4     285.2  +0.00586    0.0043     0.0000
0x...53415404           4     2        7    0  6x2 1x4     299.8  +0.00289    0.0115     0.0000
0x...53415405           2     2        8    0  8x2         298.4  +0.00094    0.0038     0.0000
0x...53415406           2     2        8    0  8x2         292.8  +0.00002    0.0040     0.0000
0x...53415407           2     2        8    0  8x2         319.8  +0.00018    0.0069     0.0000
0x...53415408           2     2        8    0  8x2         285.4  +0.00065    0.0061     0.0000
```

**BRANCH (a) criterion (`modal == 2` AND `largest ≤ 4`): 8 / 8.** Five seeds end
as **eight H₂ molecules and nothing else**; three end as six H₂ and one
four-atom cluster. Every seed has **zero free atoms**.

The two arms are complementary, not merely different: the control scores 0/8 on
branch (a) and the MBE3 arm scores 0/8 on the control criterion. Same protocol,
same seeds, same initial conditions — the only difference is whether the third
body pays.

C1 holds through both arms at N = 16: worst energy drift 1.15% of its derived
bound, worst momentum residual below 0.01% of the roundoff bound.

### The 1×4 is two molecules, not a tetramer

Three seeds end with one four-atom component. A component is a statement about
BOUNDNESS, not about closure, so the only way to tell a tetramer from two
molecules that happen to be near each other is to look at the separations. The
runner prints them (`quench_mbe3_bonds.log`):

| seed | the six separations inside the four-atom component, bohr |
|---|---|
| 0x…53415401 | **1.34  1.36**  6.05  6.18  7.31  7.32 |
| 0x…53415403 | **1.35  1.39**  5.15  6.03  6.22  7.22 |
| 0x…53415404 | **1.36  1.36**  5.87  6.40  7.23  7.71 |

Two bonds at 1.34–1.39 bohr — the H₂ equilibrium separation is 1.389 — and four
cross separations at 5.2 to 7.7 bohr. **Every four-atom component is two H₂
molecules**, joined into one component only because one of their cross pairs
satisfies the two-body bond criterion at 300 K, where the well is still ~1e-3 Ha
deep at 5 bohr.

So the honest headline is stronger than the histogram: **all eight seeds end as
eight H₂ molecules.** Three of them place two of those molecules close enough
that the pairwise boundness criterion draws an edge between them. That is the
boundness-versus-closure fence `sim.rs` already documents, showing up in the
product it was written for; the composite-holon layer's closure test is the
reading that would separate them, and it is a successor's job to run the two
side by side.

Over the same runs the closest any domain triple comes is a perimeter of 9.265
bohr, on 60,000 grain boundaries — see plant (iii).

---

## Plant (iii) — the far field · **VOID (empty sector)**

The freeze's own sentence decides this one and no judgment is needed: *"a plant
on an empty sector VOIDs."* It is scored VOID — not passed, not failed.

### The measurement

Zeroing `dE3` below a 4-bohr perimeter and re-running the MBE3 arm on the two
staked seeds returns cluster readings **identical to the unplanted arm** — 6×2 +
1×4 and 8×2, the same final temperatures to a tenth of a kelvin. The runner now
also prints why:

| MBE3 arm, 2 seeds × 20,000 boundaries | |
|---|---|
| closest approach of any domain triple | **perimeter 8.584 bohr** |
| boundaries with a triple inside the plant's 4 bohr | **0 of 40,000** |

The trajectory never enters the region the plant zeroes. Nothing was removed
from the dynamics, so nothing could shift.

### The void's cause is itself a finding

The sector is empty **because of the term under test**. A 4-bohr perimeter is an
equilateral triangle of side 1.33 bohr — tighter than the H₂ equilibrium
separation of 1.389 — and what keeps every trajectory out of it is the
three-body repulsion the plant was trying to remove. The plant is aimed at the
one part of the surface the surface itself makes unreachable. Even the
pair-only-like droplet produced by the localisation arm below, sixteen atoms
condensed with fourteen in one cluster, never puts a triple inside 4.852 bohr of
perimeter.

### The purpose, discharged separately

`tests/saturation.rs::plant_iii_a_driven_entry_diverges` (post-hoc, labelled,
NOT the staked plant) enters the sector on purpose: three atoms started as a
compact equilateral trimer of side 1.2 bohr — perimeter 3.6, inside the plant —
**at rest**, so the only thing that can move them is the surface.

| | intact | zeroed |
|---|---|---|
| `E_three` at t = 0 | +0.998880 Eh | −0.008118 Eh |
| widest separation after 60 × 64 | **65.446 bohr** | **1.838 bohr** |
| `E_kin` | 0.418023 Eh | 0.070544 Eh |

36× apart, same integrator, same everything else. The plant's PURPOSE — the
dynamics provably reads the table where the physics lives — is discharged. Its
staked INSTRUMENT is not, and both are on the record.

The structural half is a second test
(`plant_iii_the_force_loop_reads_the_zeroed_region`): `dE3` at that triple goes
+0.998862 → −0.008134 Ha under the plant, `E_three` on the three-atom scene goes
+0.998370 → −0.008512, and the force on the third atom moves by 0.1595 Ha/bohr.

### Localisation — post-hoc labelled experiments, NOT plants

Two arms, run to find out where on the surface the saturation actually lives.
Neither is a plant and neither is reported as one.

| arm | mutation | D1, 2 seeds | closest triple perimeter |
|---|---|---|---|
| `plant3b` | `dE3` zeroed INSIDE 9 bohr of perimeter | branch (a) **2/2**, unchanged | 8.584 → 3.313 bohr |
| `plant3c` | `dE3` zeroed OUTSIDE 6 bohr of perimeter | control criterion **2/2**, branch (a) **0/2** | 4.852 bohr |

`plant3b` removes more than twice the staked radius and D1 does not move, though
the trajectory's closest approach shifts by a factor of 2.6 — so the mutation IS
read, and it still does not matter. `plant3c` removes the complementary region
and the droplet comes straight back: largest clusters of 14 and 10, with the
14-cluster's separations running 1.21 to 9.53 bohr.

**The measured localisation: the saturation physics lives in the OUTER shell,
above ~6 bohr of perimeter — the geometries where a third atom approaches an
existing bond — and not in the compact core at all.** A far-field plant that
zeroes an INNER region cannot flip D1 at any inner radius, because it removes the
part of the surface the surface has already pushed the dynamics out of.

A frozen plant is not re-pointed after the fact. The record of why the next
freeze should aim differently is this section: **the successor's staked plant is
the outside-zero shape**, and this campaign's number for it is `plant3c` above.

---

## What the campaign says, in one page

**All of the engine lane's gates hold, one plant VOIDs on an empty sector, and
one staked shell fired and was re-pointed by the prereg's own pre-committed
remedy.**

| gate | verdict | the number |
|---|---|---|
| R1 · 50-digit referee | **HOLDS** | 8.99e-15 Ha over 75 geometries, 11,118× inside the stake |
| T1 · interpolant fidelity | **HOLDS** | 4.6005e-5 Ha held out over 256 staked-seed geometries, 21.7× inside the kill |
| T2 · as staked (any side = 7) | **FIRED** | 1.7720e-2 Ha, 1772× over — and the test asserts its own firing |
| T2 · as amended (middle side = 9) | **HOLDS** | 4.6758e-7 Ha, 21× margin |
| C1 · energy | **HOLDS** | ≤ 1.15% of the derived bound, across 3, 8 and 16-atom scenes |
| C1 · momentum | **HOLDS** | ≤ 0.05% of the roundoff bound |
| D1 · the product | **BRANCH (a), 8/8** | control 8/8, MBE3 8/8, zero free atoms |
| plant (i) · sign flip | **CAUGHT** | +2.573 Ha → −4.207 Ha under negation |
| plant (ii) · symmetry | **CAUGHT** | six permutations bit-identical; the break moves 7.825e-4 Ha |
| plant (iii) · far field | **VOID** | its sector is entered 0 times in 40,000 boundaries |

**The one-sentence statement of what the third body buys:** on the same protocol,
the same eight seeds and the same initial conditions, the pair-only arm scores
**0 of 8** on branch (a) and the MBE3 arm scores **0 of 8** on the control
criterion — the two arms are complementary, not merely different.

### What this is NOT

Everything the freeze said it is not, unchanged. It is exact-in-model for STO-3G
full CI and is not quantitative H3 dynamics against nature; order 3 is not
claimed to suffice beyond the gauged domain — AMENDMENT A1 records the referee's
|dE4| at 0.36–0.48 of the triple sum, which is slow convergence and is reported
as such; it is homonuclear hydrogen only; and MAX_ATOMS is 16 in a box that is a
pedagogy, so nothing here is a claim about liquid hydrogen at scale.

Two further things this lane will not let the record imply:

* **The tabulated surface is not the model.** The dynamics integrates the
  INTERPOLANT, and its held-out departure from direct FCI is T1's 4.6e-5 Ha.
  Conservation holds against the interpolant exactly, which is why C1 measures
  integration error and not that gap.
* **`eval` composes the interpolant with a sort**, and a sort is not
  differentiable where two sides cross. The potential stays continuous, so it
  stays conservative, but the force carries a measured discontinuity of
  **4.018e-2 Ha/bohr** there. It is in the table's own metadata, printed by the
  C1 gate, and it is the price of an interpolant exactly symmetric in its first
  two arguments and only symmetric-to-interpolation-error in the third.

### The costs, measured

| | |
|---|---|
| three-body table, native | 1.57 s quiet, 5.2 s at load average 65 (7,293 full-CI solves) |
| three-body table, wasm | 7.43 s at load average 65 — about 1.4× native |
| per substep, N = 16, native | 31.8 µs with the triple loop against 3.3 µs pairwise — **9.6×** |
| per substep, N = 16, wasm | 206 µs |
| shipped artifact | 252,128 bytes, up from 69,764; 35.8 KB gzipped |

The load cost is the honest deficiency and it is not small. Named successors,
none taken: a spline for the third side's pair energy (~25% of the build), a
coarser grid (21×21×13 costs 6.7× the held-out error for 2.4× the speed), and a
screen tighter than 1e-18 on the primitive pairs. Most of the artifact's growth
is the table's own 113 KB fixed array in the static `Sim`'s data segment.
