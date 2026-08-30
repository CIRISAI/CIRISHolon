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

## The table, as shipped

| | |
|---|---|
| grid | 65 × 65 × 49 nodes, `x, y ∈ [0.7, 15]` stretched at `a = 3`, `c ∈ [0.05, √2]` |
| solves | 105,105 electronic-structure points, the sorted half `i ≤ j` |
| peak `\|dE3\|` on a node | 0.763782 Ha |
| curvature envelope | 3969.31 Ha/bohr² absolute, 18102.4 /bohr per-gradient |
| **sort kink** | **0.000e0 Ha/bohr — EXACTLY zero** |
| artifact | 1,787,303 bytes, IEEE-754 bit patterns |

The sort kink being an exact zero is the claim from the coordinates section, measured:
`TrimerTable` has a real force discontinuity where two sorted sides cross and
reports it; here the H-H side never enters the sort and the table is exactly
symmetric in the only pair that does, so the composed potential has no kink at
all.

**The size choice.** The rule was stated before the tableau was read: the
smallest `(NR, NU)` subgrid whose held-out maximum is at most a third of T1's
1e−3 kill. **No subgrid met it** — the best available, 65 × 49, reads 7.68e−4 on
the sizing sweep's 384-point draw. So the rule selects the only pair that clears
the kill at all, and the margin is reported rather than dressed up. The angle
axis is the binding one and it converges at about 5× per doubling where a C1
cubic should give 16×, which is a fact about the surface that this campaign has
NOT explained; `examples/s2_third.rs` measures a third candidate coordinate (the
bend angle) beating both `u` and `c` on the worst slice by 2.2×, and that is the
named successor.

---

## R1 — the 50-digit (O, H, H) referee · **HOLDS**

    worst engine-vs-referee disagreement 5.9214e-12 Ha
    over 84 staked geometries x 5 energy columns
    worst on dE3, at x = 4.868, y = 15, u = 0.745;   stake 1e-10 — 17x inside

The referee (`conformance/atomworld/saturation2_referee.py`, mpmath at 60 working
digits reported to 50) shares its integrals, its determinant CI and its certified
eigensolver with the committed `elements1_referee/` **by import rather than by
copy** — a second transcription of a bank is how a bank stops being one. What is
written for this campaign is the (O, H, H) geometry construction, the many-body
decomposition, the staked set and the comparison. It shares the MODEL with the
engine and nothing else.

5.9e−12 is what the arithmetic predicts rather than a number to be relieved
about: `E(H2O)` is about −75 hartree, so an f64 carries roughly 1e−14 of absolute
room, and `dE3` is a difference of FIVE such numbers, four of them near −75.

**The staked set, result-blind.** 84 geometries against the prereg's ≥ 48, every
one a function of the declared domain constants and a fixed integer ladder — a
six-rung geometric ladder of O-H sides from 0.9 to `R_HI`, crossed with itself
under `x ≤ y`, crossed with four staked angles. Nothing in it consults an energy,
a minimum, a bond length or an angle. Coverage, computed by the referee from the
geometry alone and asserted non-empty per family: compact 4, closed 11, bent 22,
linear 11, near-boundary 24, stretched 12.

### The spin audit: asserted where resolved, reported where degenerate

M-PARITY-PROTECT, in the prereg's own words. The multiplicity is measured from
`<S²>` of the converged vector, and whether it MEANS anything is measured too:
the referee solves the `Sz = 1` sector as well, where the lowest state is by
construction the lowest triplet, so the difference is the exact singlet-triplet
gap.

    28 of 84 geometries have a RESOLVED gap — every one of them a singlet
    56 are degenerate — 2S = 0, 1 and 2 all occur, and none is asserted on

Both branches are non-empty and the gate requires that. A bonded geometry is a
resolved singlet; a geometry with one hydrogen at the far edge of the domain is a
dissociated OH + H whose singlet and triplet are exactly degenerate, and there
the label is a fact about which component the eigensolver returned, not about the
state. **A gate that demanded "singlet everywhere" would have fired on correct
physics** — and the first version of this one did.

### Where the referee could not check itself, and why it says so

The referee's independence check re-solves each geometry in a randomly rotated
orbital basis. At a dissociated geometry the ground state is near-degenerate
(oxygen's ³P times two hydrogen doublets), so the Temple bound has no gap to
certify against and the rotated route does not converge: measured at
`x = y = 8.545, c = 0.959`, route A finishes in 32 s at dps 60 while route A plus
route B was **still running after twenty minutes** and had to be killed.

So route B is declared unavailable wherever route A's own gap says it cannot be
certified — decided from a quantity route A already computed, before route B is
paid for, deterministically, with no wall clock consulted. At that geometry the
gap is 7.42e−08 Ha against a declared 1e−6 threshold.

    74 of 84 geometries carry the second CI route
    10 are single-route, all of them degenerate, each recording its own reason

A referee that stalls on a tenth of its staked set is not a referee. One that
says which tenth it could not double-check is, and the gate reads that record
rather than letting it sit in a field nobody is required to look at.

## T1 — interpolant fidelity, held out · **HOLDS**

    worst held-out error 2.4670e-4 Ha over 256 points, staked seed, none on nodes
    at (x, y, c) = (3.3917, 3.5763, 0.6805);  kill 1e-3

Two-sided as the prereg requires: all 256 points return a nonzero error, so the
draw is not landing on nodes and measuring its own construction.

**Reported, not buried:** the sizing sweep's INDEPENDENT 384-point draw over the
same domain reads 7.68e−4, three times the gate's. Both are inside the kill;
neither is the supremum. The spread between two honest draws *is* the evidence
that a maximum over a finite draw understates the thing it stands for — the same
fact that cost this campaign its first truncation radius, below.

## T2 — the truncation systematic · **HOLDS**

    truncation systematic 3.5487e-6 Ha on the shell max(O-H) = 15
    at (x, c) = (12.3606, 0.00200);  kill 1e-5, margin 2.8x

The gate is two-stage — a declared coarse grid, then a refinement around whatever
that grid's argmax turns out to be at runtime — and reaches past the table's own
closed-angle fence to `c = 0.002`. Both features are the direct consequence of
the `b = 14` near-miss recorded above.

## G1 — THE EMERGENT GEOMETRY · **HOLDS**

    MBE3 optimum   r = 1.943467 bohr    theta = 96.7738 deg    E = -75.023293230 Ha
    model's own FCI r = 1.9435740105     theta = 96.75788837    E = -75.023291531 Ha
    deviation        +1.07e-4 bohr        +1.59e-2 deg           1.7e-6 Ha

**The bent molecule emerges.** Minimising pairs-plus-`dE3` over the three-parameter
space lands on the model's own full-CI optimum to 1.1e−4 bohr in the bond length
and 0.016 degrees in the angle — and the reference was computed, by a different
route, before a single node of the table existed. The kill (an optimum
qualitatively wrong, linear where the FCI is bent) did not fire; the symmetric
point is checked to be a minimum and not a saddle in the antisymmetric direction
as well.

*[LABELLED CONTEXT, never compared against: nature's 104.5 degrees and 1.8085
bohr.]*

## G2 — VALENCE SATURATION · **HOLDS on the model, and the route there is the result**

    the CONTROL: water's own second O-H bond          +0.163077 Ha
    the CLAIM:   deepest third-H binding anywhere
                 over 40 full-CI four-atom geometries -0.004449 Ha

**The third hydrogen refuses**, and not merely shallowly: over every staked
direction and radius the best it can do is be REPELLED. There is no well. Along
the C2 approach the repulsion is monotone — 0.5395 Ha at 1.2 bohr, 0.1552 at 1.8,
0.0520 at 2.5, 0.0022 at 4.0. Water saturates at two hydrogens in this model.

### How this gate was first written, why it FIRED, and what the investigation found

This is the part worth reading. G2 was first implemented against the MBE3
ESTIMATE — pairs plus the tabulated three-body term — because the campaign is
about the table. **It fired**, at 1.76× against the staked 5×: the MBE3 surface
says a third hydrogen binds to relaxed water by +0.0939 Ha. Branch (b) is
"investigate, never massage", and `examples/s2_g2_probe.rs` is the investigation.

At the very geometry the MBE3 scan calls deepest, the model's own full CI says
the third hydrogen is repelled by 0.0890 Ha. **The two disagree in SIGN.** The
missing four-body term is −0.183 Ha there, comparable to the O-H bond itself.

So the firing is a fact about the EXPANSION, not about the model and not about
the table. Re-reading the prereg, `E(H3O)` is the model's own four-atom energy —
so the gate now scores the quantity the freeze names, and the MBE3 reading is
kept as the scope finding it is, gated separately in
`the_three_body_expansion_cannot_saturate_oxygen_and_says_so`.

The control that makes the whole thing readable, and that separates "the
expansion is wrong" from "the table is wrong":

    water's second O-H bond, full CI  +0.163077 Ha
    water's second O-H bond, MBE3     +0.163078 Ha      (differ by 1.7e-6)

On a THREE-atom system the three-body expansion is EXACT, so the only gap is the
table's own interpolation — and it is 1.7e−6 Ha, exactly the scale T1 measures.
The table is fine. The expansion runs out at four atoms.

### THE SCOPE FINDING, and a forward stake for P1

    at O-H3 = 2.250 bohr on the C2 axis:
      the MBE3 sandbox BINDS a third hydrogen by  +0.051973 Ha
      the model REPELS it by                       0.079962 Ha
      the missing four-body term                  -0.131935 Ha  = 81% of the O-H bond

SATURATION-1 found this shape one order down: a pair-only sandbox cannot saturate
hydrogen's valence, and order three is where the saturation appears. Here an
order-three sandbox cannot saturate OXYGEN's valence, and order four is where it
would. Stated as a reading with its own kill rather than as a law: it predicts
that a valence-three centre would need order five, and a nitrogen trimer table
that saturated NH3 at order four would falsify it.

**The consequence for P1, staked here BEFORE the mixed arm is run:** an MBE3
sandbox will over-bind a third hydrogen to water by about 0.05 Ha, so the quench
is expected to produce H3O and heavier, NOT H2O as the modal O-containing
molecule. Branch (a) is therefore at risk for a reason that is now measured and
named. That is a prediction, and P1 is where it gets to be wrong.

## Plant (i) — the sign flip · **CAUGHT**

Negating the table inverts the bent-vs-linear preference. The carrier is asserted
first: on the true table the bent geometry is more than 1e−2 Ha below the linear
one, so there is something for a sign flip to invert.

## Plant (ii) — the symmetry plant · **two parts CAUGHT, one VOID (empty sector)**

* **bit-level exchange** — CAUGHT. On a staked asymmetric geometry (O-H sides 1.6
  and 2.9 bohr) the value and all three gradient components are BIT-IDENTICAL
  across `H ↔ H`, not merely equal to a tolerance.
* **the O-distinct axis is not symmetrised** — CAUGHT. Reading the same triangle
  with all three sides sorted moves the answer by more than 1e−4 Ha, so this is
  not a relabelled H3 table.
* **"a desymmetrised table must fire ≥ 1e−6 Ha"** — **VOID, on an empty sector,
  and the void's cause is itself the finding.** It cannot fire: `eval` sorts the
  two O-H sides before it reads, so both orders reach the same stored entry and a
  broken mirror is invisible to the exchange. The symmetry is carried by the
  SORT, not by the storage. Per M-PLANT-SECTOR a plant on an empty sector voids
  rather than passes, and the test asserts the emptiness rather than assuming it.
  What the corruption does break is the VALUE, by ≥ 1e−6 Ha — which is what T1
  would catch — and that half is asserted live.

## Plant (iii) — the swapped table · **CAUGHT**

Serving the (H, H, H) table to an (O, H, H) triple at water's own optimum moves
the reading by orders. Carrier asserted: the (O, H, H) table reads more than
1e−3 Ha there, so there is something for a swap to move.

## C1 — conservation with the heteronuclear triple paying · **HOLDS**

One gate per law, never combined.

| scene | quantity | measured | derived bound | ratio |
|---|---|---|---|---|
| NVE, O + 2H | energy drift | 1.134e−6 Ha | 8.248e0 | 1.4e−7 |
| NVE, O + 2H | momentum residual | 1.179e−13 | 9.292e−10 | 1.2e−4 |
| thermostat, 1 O + 5H | energy drift | 4.736e−5 Ha | 4.977e−1 | 9.5e−5 |
| thermostat, 1 O + 5H | momentum residual | 7.312e−13 | 7.763e−10 | 9.4e−4 |

C1 rests entirely on the analytic side-derivatives being the derivatives of the
value the ledger books, and that is gated directly: central differences against
the tabulated surface at six geometries spanning the domain — including one
INSIDE the closed-angle fence, where the linear-in-`u` extension has to be a
genuine first-order Taylor — agree to **0.000 of their own tolerance**.

### Two things disclosed rather than left to be found

**The scene that did not move.** The mixed scene was first written with two
oxygens. It reported a drift of EXACTLY ZERO and passed — because `Sim::step`
guards on `pairs_ready()`, the bank held no O-O curve, and the scene never
integrated a single substep. A conservation gate on a frozen box passes for the
wrong reason. `run` now refuses any scene whose atoms did not move, and the mixed
scene is one oxygen and five hydrogens; the (O, O, H) fence is gated statically
instead, on a single force evaluation, where it costs nothing. The O-O curve is
not a fixture because it is not affordable as one: O2 is 2025 determinants a
point, measured at 0.21 s / 0.40 s / 5.11 s at 2.2 / 3.0 / 5.0 bohr in RELEASE.

**The energy bound is loose by four to seven orders**, and that is the curvature
envelope's compact corner rather than a healthy margin. The peak `|dE3|` is
0.7638 Ha at `x = y = 0.7`, `c = 0.05` — two hydrogens 0.05 bohr apart — where
the grid spacing is 0.0334 bohr, giving a second derivative of order 685 before
the 4× widening. That corner is unreachable in any dynamics the sandbox runs, and
it sets the absolute cap for all of them. The momentum bound, which does not
depend on it, is tight to within four decades and is the informative half of C1.

---
## P1 — THE PROTOCOL, FROZEN

*Written and committed BEFORE the mixed arm was run or its output looked at, as
the prereg requires. Everything below is a `const` in
`engine/crates/holon-render/examples/waterquench.rs`, not a flag, so a reported
run re-runs byte for byte.*

### The scene

| | |
|---|---|
| atoms | 12, the same in every arm |
| mixed arm | 4 O then 8 H — the freeze's 8 H + 4 O |
| controls | 12 H (`hydrogen`), 12 O (`oxygen`) |
| dimensions | 2 — the `z = depth/2` slice |
| box | 34.6 × 20.8 bohr, soft quadratic walls. SATURATION-1 ran sixteen atoms in 40 × 24; this is twelve at the same number density, so the hydrogen control is comparable to its own bank |
| opening positions | a 4 × 3 lattice at `(w(col+½)/4, h(row+½)/3)` with a per-seed uniform jitter of ±0.8 bohr |
| opening velocities | Box–Muller Gaussians from the same seeded stream at `T_init = 3000 K`, **with sigma taken PER SPECIES** — a Maxwellian is a distribution over speeds and oxygen is sixteen times hydrogen, so one sigma for the box would open the scene at two different temperatures — and the net MOMENTUM removed, which with two masses is not the net velocity |
| thermostat | ON from the first step, Berendsen, `T_target = 300 K`, `tau = 2000` a.u. |
| integration | 20,000 grain boundaries × 64 substeps; `dt` derived from the curves |
| pair curves | 96 knots each, generated by this crate's own solver: H-H and O-H for the mixed arm, O-O too, only what an arm contains |
| three-body | H3 generated at run; (O,H,H) loaded from the committed artifact |
| RNG | one LCG (`x = 6364136223846793005 x + 1442695040888963407`, top 53 bits) seeded per run; nothing else is random |

### The eight staked seeds

```
0x0000000053415421  0x0000000053415422  0x0000000053415423  0x0000000053415424
0x0000000053415425  0x0000000053415426  0x0000000053415427  0x0000000053415428
```

### The measurement rule

Taken at the final grain boundary from `Sim::cluster_species_counts` — one
union-find over the one edge set the headline `cluster_count` already reads, read
for its COMPOSITION. It is a MOLECULE rule and not a size rule, because this
campaign's question is which molecules form and a size histogram cannot tell H2O
from H3 plus a free oxygen.

* a molecule is written `O_a H_b` with `a` and `b` its nuclear counts;
* a component of ONE atom is a **free atom**, not a molecule;
* **O-containing** means `a ≥ 1`;
* the **modal O-containing molecule** is the most common composition among them,
  ties broken toward the one with FEWER hydrogens — the conservative direction
  for a gate whose branch (a) is `H2O`, so a tie can never be resolved in the
  claim's favour;
* the full molecule census is published either way.

### The criteria, and what each decides

* **CONTROL — hydrogen arm**: modal cluster size 2 with no free hydrogen, in ≥ 6
  of 8 seeds. If it fails the gate is VOID — protocol, not physics — and the
  protocol is not re-tuned afterwards.
* **CONTROL — oxygen arm**: oxygen must BIND through its own pair curve (largest
  component ≥ 2 in ≥ 6 of 8 seeds), and the (O,O,O) fence must be counted at
  `C(12,3) = 220` per force evaluation. This arm is MBE2-ONLY by construction —
  SATURATION-2 does not tabulate (O,O,O) — so a droplet is the EXPECTED outcome
  and is reported, not scored: it is SATURATION-1's own pair-only finding
  reproduced in a second element.
* **BRANCH (a)**: `H2O` is the modal O-containing molecule AND there is zero free
  oxygen, in ≥ 6 of 8 seeds.
* **BRANCH (b)**: anything else, reported and investigated as a finding about the
  in-model expansion, not massaged.

Both arms also report the energy-drift and momentum-residual ratios against their
own derived bounds, per seed, and the fence incidence per seed.

### DISCLOSED: everything that was run before this freeze

The control arms only. The mixed arm was not run and its output was not looked
at.

| what | outcome |
|---|---|
| `hydrogen`, seed 1 | 6 × H2, zero free H, largest 2 |
| `hydrogen`, all 8 seeds | **44 × H2, 2 × H4**; modal size 2 and zero free H in 8/8; largest ≤ 4 in 8/8 |
| `oxygen`, seed 1 | one O12 droplet, fence 220 |
| `oxygen`, all 8 seeds | **8 × O12**; fence exactly 220 every seed; zero free O in 8/8 |

**Nothing was tuned.** No protocol variant was tried: the box came from
density-matching SATURATION-1 rather than from a sweep, the schedule,
temperatures and coupling time are SATURATION-1's unchanged, and the 96-knot
curves are MIXTURES-1's own measured choice. Both controls passed on their first
run. That is worth saying plainly because SATURATION-1 had to disclose three
control-only variants before it found its schedule, and a lane that reports "no
tuning" without having looked is making a different claim from one that looked
and found none needed.

The two H4s in the hydrogen control are the artefact SATURATION-1 documented and
named: a cluster is a statement about boundness, not about closure, and two H2
molecules whose cross pair happens to read `bonded` are one component of four.

### THE FORWARD STAKE, and it is a prediction that can be wrong

From gate G2's investigation, already committed: the MBE3 sandbox **over-binds a
third hydrogen to water by about 0.05 Ha**, where the model's own full CI repels
it, because the four-body term the expansion does not have is −0.13 Ha at that
geometry. The oxygen control makes the same point from the other side — 12
oxygens with no three-body term at all condense into a single droplet in 8 of 8
seeds.

So **branch (a) is predicted to FAIL**: the mixed arm is expected to produce H3O
and heavier rather than H2O as the modal O-containing molecule. Staked here,
before the run, with the mechanism named and measured. P1 is where it gets to be
wrong.
