# SATURATION-2 — results

*The record for `SATURATION2_PREREG.md` (frozen 2026-08-30, commit 75f1a87): the
first heteronuclear three-body surface, (O, H, H), and what it makes emerge.
Gates R1, T1, T2, G1, G2, C1, P1 and plants (i), (ii), (iii) are this lane's.
Everything here is EXACT-IN-MODEL — STO-3G, full CI, Born–Oppenheimer. Nature's
104.5 degrees and 0.957 angstrom appear as labelled context and nothing is ever
scored against them.*

---

## THE REFERENCE, computed before the table existed

Gate G1 asks whether minimising pairs-plus-`dE3` reproduces the model's own
optimum, so the model's own optimum had to be a number first. It was located by
Newton on the EXACT first and second derivatives the dual-number route already
carries (`examples/s2_design.rs`), in the symmetric stretch and in the angle,
before a single node of the three-body table had been solved:

| | in-model, STO-3G full CI |
|---|---|
| `E(H2O)` | −75.023291531289 Ha |
| `r_OH` | 1.9435740105 bohr |
| `theta_HOH` | 96.75788837 degrees |
| `dE/dr`, `dE/dtheta` at that point | 8.9e−15 Ha/bohr, −7.5e−16 Ha/rad |
| `d2E/dr2` (symmetric) | +0.9238403 Ha/bohr² |
| `d2E/dtheta2` | +0.2849532 Ha/rad² |
| `d2E/ds2` (ANTISYMMETRIC) | **+1.036226 Ha/bohr²** — positive, so this is a minimum and not a saddle |
| relaxed LINEAR (`theta = 180`) | `r` = 1.78852205 bohr, `E` = −74.888555515412 Ha |
| what the bend is worth | 0.134736 Ha |

The antisymmetric curvature is REPORTED rather than assumed: "the optimum is on
the symmetric line" is a claim about this surface, and a two-parameter search
that never looked off that line would have been asserting it.

*[LABELLED CONTEXT, never compared against: nature's water is 104.5 degrees and
0.957 angstrom = 1.8085 bohr. STO-3G's in-model answer is the claim.]*

---

## The instrument

```text
V2_AB(r) = E(AB; r) - E(A) - E(B)
dE3      = E(OHH) - [E(O) + 2 E(H)] - V2_OH(x) - V2_OH(y) - V2_HH(z)
         = E(OHH) + E(O) + 2 E(H) - E(OH; x) - E(OH; y) - E(HH; z)
```

Every energy comes from `pair::solve_geometry`, the general N-centre route — the
same solver the 50-digit referee grades, and NOT a second implementation. A water
point is 7 basis functions, 10 electrons, 441 determinants in the minimal-|Sz|
sector.

### The coordinates, and what the heteronuclear case changes

`dE3` for three hydrogens is totally symmetric in the triangle's three sides, and
`trimer.rs` sorts all three. Here the symmetry is **H ↔ H exchange only** —
oxygen is a distinct vertex — so the table is built on

```text
x = min(r_OH1, r_OH2),   y = max(r_OH1, r_OH2),   c = sqrt(1 - cos theta_HOH)
```

the two O-H sides sorted, and the angle **at oxygen**. Evaluation sorts those two
sides, which is exact in floating point, so H ↔ H invariance is bit-for-bit
rather than to a tolerance.

One consequence worth naming, because it is a property this table has that the
H3 table does not: **there is no sort kink.** `TrimerTable` sorts across two
different table axes and is only symmetric-to-interpolation-error in the third,
so its force carries a small discontinuity where two sorted sides cross. Here the
H-H side never enters the sort and the table is exactly symmetric in the only
pair that does.

### Why `R_HI` is 15 bohr — and the two ways 14 was wrong

SATURATION-1's AMENDMENT A1 truncates on the SECOND-SMALLEST side, because `dE3`
vanishes exactly when some atom is far from BOTH of the others — two long sides,
i.e. a long `s2`. That is geometry, not species, and it transfers. What does not
transfer is the box: this table's axes are the two O-H sides, and the smallest
box in those that CONTAINS `{s2 <= R_cut}` is `x, y <= 2 R_cut`, because
`s2 <= R_cut` forces every side below `2 R_cut` through the triangle inequality.

`examples/s2_domain.rs` swept the shell `max(O-H) = b`, over `x` in `[0.9, b]`
and the angle across the range, and reported the worst `|dE3|` anywhere on it:

| b | worst \|dE3\| | at x | theta | s2 there |
|---|---|---|---|---|
| 4 | 1.67e−1 | 2.14 | 4.1° | 2.14 |
| 6 | 3.86e−2 | 2.94 | 4.1° | 3.07 |
| 8 | 5.92e−3 | 4.09 | 4.1° | 4.09 |
| 9 | 2.29e−3 | 4.54 | 4.1° | 4.54 |
| 11 | 3.03e−4 | 5.45 | 4.1° | 5.58 |
| 12 | 1.02e−4 | 5.89 | 4.1° | 6.13 |
| 13 | 3.25e−5 | 6.34 | 4.1° | 6.69 |
| 14 | 9.71e−6 | 6.79 | 4.1° | 7.24 |

which reads as "14 is the first integer shell inside the 1e−5 stake, and it is
`2 x 7`". **Both halves of that were wrong.**

**One — a grid maximum understates its own supremum.** Every shell's worst
reading sits at `theta = 4.1°`, the sweep's own closed-angle floor, which is the
signature of a maximum outside the grid rather than on it. Re-swept at five times
the `x` resolution and with the angle carried down to `c = 0.002`, the `b = 14`
shell reads **1.0091e−5** — *above* the stake. A 3% margin taken from a lower
bound is a gate passing on its own resolution.

**Two — the tail is ALGEBRAIC.** Past `b = 14` the worst point stops being the
near-collinear chain and becomes a stretched hydrogen MOLECULE with the oxygen
far away: 3.54e−6 at `b = 15`, 2.48e−6 at `b = 16`. Those fall far too slowly for
an exponential, so `examples/s2_dispersion.rs` staked the obvious explanation and
tested it.

#### The staked exponent FIRED, and the discriminator was sharper than the stake

| | |
|---|---|
| **staked, before the run** | `dE3 ~ R^-6` — dipole-dipole dispersion |
| **measured, collinear** | slope **−5.0099** (clean tail, `R > 13` bohr) |
| **measured, broadside** | slope −4.93 |
| successive two-point exponents | 5.016, 5.012, 5.008, **5.007** |

`R^-5` is the quadrupole–quadrupole law, not a dispersion law. The stake fired
and is kept fired: `PREDICTED_SLOPE` in that example is still `-6.0`, because
re-pinning it would delete the evidence that a prediction was made and missed.

What replaces it: oxygen's open 2p shell has a quadrupole and H2's **bond** has
one, while an isolated hydrogen atom is spherical in this basis and has none —
which is the same fact that keeps the whole effect out of the pair terms.
Measured, `V2_OH(12.41)` is 3.9e−14 against a triple holding 3.1e−6 at the same
separation.

That reading predicts something falsifiable and cheap: replace the oxygen with
**neon**, closed-shell and spherical, and the `R^-5` channel should close leaving
the `R^-6` behind. **It came back sharper than the prediction.** Neon has no
algebraic tail at all — `|dE3|` is 3.5e−8 at 8 bohr, 8.0e−12 at 10, and inside
the f64 cancellation floor from 12 out, zero points measurable past 13 bohr. So
removing the open-shell quadrupole removes the algebraic sector *entirely*, and
minimal-basis dispersion sits below anything this campaign can resolve: it was
never the right story at any separation, not merely the wrong exponent.

**So `R_HI = 15`**, measured at 3.54e−6 with 2.8× of margin under the stake.

### Why the third coordinate is `c`, and where its singularity went

`trimer.rs` uses `c = sqrt(1 - u)` and records why: at `x = y` the third side is
`z = x sqrt(2) c` exactly, so a uniform `c` grid is a uniform `z` grid there.
That argument transfers. What does not is the domain — a SORTED hydrogen triple
has `u <= 1/2`, so H3's grid never approaches `u = 1`, whereas here `u = 1` means
both hydrogens on one ray from the oxygen, which is exactly a hydrogen molecule
approaching an oxygen head-on: the reaction this campaign exists to watch. And at
`u = 1` the `c` map is singular, because `dE3` is analytic in `u`, so `dF/dc`
vanishes proportionally to `c` and the chain rule back to the sides needs
`dF/du = -F_c/(2c)`.

Both candidates were measured rather than argued about (`examples/s2_third.rs`),
one-dimensionally at five staked `(x, y)`, against exact node values and an exact
`dE3/du` from the dual-number route, on the VALUE and on the derivative the force
loop reads:

| nodes | worst \|ΔV\| in u | in c | worst \|Δ dF/du\| in u | in c |
|---|---|---|---|---|
| 25 | 3.9e−3 | **2.5e−3** | 1.2e−1 | **5.9e−2** |
| 49 | 1.27e−3 | 1.28e−3 | 7.7e−2 | **5.0e−2** |
| 97 | 2.3e−4 | 2.9e−4 | 3.5e−2 | **3.1e−2** |

`c` wins or ties, including on the derivative, because the better node placement
outweighs the `1/c` amplification. So the grid is in `c` and the singularity is
handled where it belongs — in the chain rule. Every derivative the table returns
is converted to `u` **at the clamp point**, so the only division is by
`max(c, C_LO)` with `C_LO = 0.05`, and inside that fence the surface is extended
LINEARLY IN `u`, never in `c`. An exactly collinear head-on approach therefore
meets a finite force.

### The closed-angle fence, measured

`dE3` itself does not diverge as the two hydrogens meet — the `1/z` nuclear
repulsion cancels between `E(OHH)` and `E(HH)` by construction — and it is smooth
and SATURATING down to `c = 0.005`, two hydrogens 0.005 bohr apart. What degrades
is the f64 solve:

| c | z (H-H), bohr | Davidson residual | dE3 at x = y = 0.7 |
|---|---|---|---|
| 0.05 | 0.0495 | 1.0e−10 | 0.52895536 |
| 0.02 | 0.0198 | 2.3e−10 | 0.53090585 |
| 0.01 | 0.0099 | 1.8e−9 | 0.53118472 |
| 0.005 | 0.0049 | 2.2e−8 | 0.53125444 |

So `C_LO = 0.05`, where the solve is still clean, and the sliver inside it
(`theta < 4.05°`) is the linear-in-`u` extension.

### The grid, sized by measurement

`examples/s2_grid.rs` builds one fine 49 × 49 × 25 table per candidate stretch and
reads the held-out error of every coarser grid inside it (384 held-out points,
staked seed, none on nodes):

| stretch a | compact-end spacing | held-out max at 49 × 49 × 25 |
|---|---|---|
| 2 | 0.0886 bohr | 7.84e−4 Ha |
| **3** | **0.0449 bohr** | **6.72e−4 Ha** |
| 4 | 0.0216 bohr | 8.00e−4 Ha |

A shallow minimum at `a = 3`. The full (nr, nu) tableau at `a = 3` also shows the
side axis is the binding one: at `nu = 25`, going `nr` 25 → 49 buys 2.9×, while at
`nr = 49`, going `nu` 13 → 25 buys 8.6×.

### How the table is FILLED, and the fence that puts on the record

One (O, H, H) point is 441 determinants and about 50 ms, a thousand times an H3
point (nine determinants on a bespoke s-only path). **So this table is not built
at page load the way H3's is.** It is generated natively by
`examples/s2_build.rs`, committed as a text artifact of raw IEEE-754 bit
patterns, and streamed. That is a real difference from SATURATION-1 and it is
stated rather than left to be discovered.

What keeps it from turning the sandbox back into a *player* of someone's curve:
`tests/water.rs` recomputes a staked subset of the committed nodes through the
crate's own solver, today, and requires **bit-identity** — a tolerance there
would be measuring the tolerance — and the artifact refuses to load against a
grid rule that is not this build's.

---

## R1 — the 50-digit (O, H, H) referee · *pending*

## T1 — interpolant fidelity, held out · *pending*

## T2 — the truncation systematic · *pending*

## G1 — THE EMERGENT GEOMETRY · *pending*

## G2 — VALENCE SATURATION · *pending*

## Plants (i), (ii), (iii) · *pending*

## C1 — conservation with the heteronuclear triple paying · *pending*

---

## P1 — THE PROTOCOL · **NOT YET FROZEN**

This section is deliberately empty of numbers. The prereg requires the 8 H + 4 O
quench protocol to be staked and committed BEFORE the mixed arm is run or looked
at, and the protocol cannot be sized until MIXTURES-1's pair bank can serve O-H,
O-O and H-H curves in one box — that lane is building it in parallel.

What is already fixed by the prereg and will not move: 8 H + 4 O, eight staked
seeds, a thermostat, and two control arms (pure-H reproducing SATURATION-1's
molecules, pure-O reproducing O2 through its pair curve, labelled MBE2). Branch
(a) is H2O as the modal O-containing molecule with zero free O; branch (b) is
reported and investigated; a failing control VOIDs rather than fires.

Sizing will be done on the CONTROL arms only and DISCLOSED here, exactly as
SATURATION-1 disclosed its three control-only protocol variants. The mixed arm
will not be run until the freeze below is committed.

The (O, O, H) and (O, O, O) fence is already instrumented: `Sim::accumulate_three_body`
dispatches on the triple's composition with three cases and no default, and a
triple with no table of its own increments `Sim::fence_untabulated` rather than
being served some other composition's surface. The counter is exported through
the ABI as `holon_fence_untabulated`.
