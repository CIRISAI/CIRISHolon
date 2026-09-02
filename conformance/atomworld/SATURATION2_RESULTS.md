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

### WHERE THE ERROR LIVES — a localization clause, labelled POST-DATA

*Added 2026-09-02 from evidence the published bound did not have. **The bound does
not move.** 7.68e−4 came from a pre-registered 384-point draw, every disagreement
measured since is inside it, and a published number is not revised on post-data
evidence. What follows says where inside it the error sits.*

**The error is not uniform over the grid. It concentrates in the near-collinear
band, and real scenes visit that band constantly.**

The de5 audit, on its own sample and its own instrument, binned 144 compact
(O, H, H) triples against the served table. The near-collinear band `θ ≥ 150°` is
**13 of 144 triples — 9.0% — and holds 9 of 9 disagreements ≥ 1e−4 Ha**, with a
median separation of 130× across the cut. All nine sit within 1.6 grid cells of
SATURATION-3's **Seam 1**, the electronic state crossing at `c ≈ 1.4128`,
`θ ≈ 174.9°`. The largest is 6.678e−4, inside the 7.68e−4 declared here.

Two instruments, two samples, one corner. This lane found the seam by walking a
slice's second and third differences and showed that no smooth interpolant
converges on a corner; the audit found that 9% of the triples a trajectory
actually visits land on it — without looking for it, and neither reading contains
the other.

**Why this clause exists rather than a different number.** A flat bar over a
domain containing a non-smooth feature is a summary that hides its own structure.
It is never a false bound, and it is uninformative in both directions: a consumer
averaging over many triples is told they carry 7.68e−4 when they carry far less,
and a consumer whose quantity lives near collinear is told a number that is true
and given no reason to look harder. Naming where the error lives costs a sentence
and turns the bar back into information. This is the same clause, for the same
reason, as the O-O disclosure's "the caps are in the dissociation tail past ~6
bohr" — which is the sentence the census lane reported as the most useful thing it
received, because it let them decide whether it touched them.

**What it does NOT license.** No refinement of the served table. Uniform
refinement cannot beat a corner — that is what the 5×-per-doubling reading was,
and gauging the interpolator at 9.3–10.9× on planted smooth data is what proved
the shortfall belongs to the surface rather than the scheme. For any FUTURE
(O, H, H)-class grid the options remain: put a grid line ON the seam, splitting
the domain into two smooth patches that each recover the gauged rate, or accept
the floor deliberately. The audit's 9% is a traffic estimate this lane did not
have, and it moves the lean toward the first. It changes nothing about the served
table, which stands as published on its published bound.

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
missing four-body term is **+0.183 Ha** there — repulsive, comparable to the O-H
bond itself.

*A correction to this file's own wording.* That number was first written here as
−0.183, which is the same quantity expressed as a difference of BINDINGS rather
than of energies. Both are correct; neither was named. The energy convention
`ΔE4 = E_FCI − E_MBE3` is the one used from here on, because it is the sign a
four-body TERM would be added with, and `examples/s2_mbe4_verify.rs` prints the
identity from one computation so the reader does not have to take it on trust. A
number stated in a convention it does not name is a number that can be read
backwards, and a downstream lane read this one backwards.

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
      the missing four-body term                  +0.131935 Ha  = 81% of the O-H bond
      (ENERGY convention, E_FCI - E_MBE3; repulsive, which is what saturation needs)

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



---

## P1 — THE PRODUCT · **BRANCH (b), 0 of 8 seeds. No water.**

*Run after the freeze above was committed. Controls first, then the mixed arm.*

### The controls, both PASS

| arm | outcome |
|---|---|
| `hydrogen` (12 H) | **44 × H2, 2 × H4** over 8 seeds; modal size 2 and zero free H in 8/8; largest ≤ 4 in 8/8. SATURATION-1's molecules, reproduced |
| `oxygen` (12 O) | **8 × O12** — one droplet every seed; zero free O in 8/8; the (O,O,O) fence counted at exactly 220 per force evaluation, which is `C(12,3)`, every triple |

The oxygen control is SATURATION-1's own pair-only finding appearing in a second
element: with no three-body term at all, twelve atoms over-coordinate into a
single droplet. It is MBE2 behaviour, labelled as such, and it is what the
freeze's Scope said it would be.

### The mixed arm

    seeds with H2O as the modal O-containing molecule : 0 / 8
    seeds with zero free oxygen                       : 8 / 8
    molecule census over all 8 seeds:  20 x H2   2 x O2H2   4 x O4H2   3 x O4H4
    fence incidence: 52 triples refused, every seed
    worst drift / bound 3.6e-6, worst |p| / bound 9.5e-5, T 292-339 K

**No seed made water.** Every seed made hydrogen molecules and an oxygen
AGGREGATE carrying two to four hydrogens: O4H2 four times, O4H4 three times, and
one seed splitting into two O2H2. Zero free oxygen and zero free hydrogen
everywhere — everything is bound to something, just not into H2O.

### The forward stake: RIGHT IN DIRECTION, WRONG IN MECHANISM

The stake committed before this run said branch (a) would fail, and it did, 0 of
8. But it named the wrong reason, and the distinction is the finding.

**Predicted:** the MBE3 sandbox over-binds a THIRD HYDROGEN to water by ~0.05 Ha
(G2's measurement), so the quench makes H3O and heavier.

**Observed:** no oxygen ever collects a third hydrogen. The census has no species
with more than one H per O. What dominates instead is **oxygen–oxygen
aggregation**, and the mechanism is this campaign's own DECLARED SCOPE FENCE
rather than the four-body term.

SATURATION-2 tabulates `(O,H,H)` and nothing else. `(O,O,H)` and `(O,O,O)` are
not tabulated and run pair-only — the freeze says so in its Scope, and the
sandbox counts every occurrence. In a box with four oxygens that fence fires **52
times per force evaluation**, and 52 is exactly `C(4,2)·8 + C(4,3)` = 48 + 4:
every two-oxygen and three-oxygen triple there is. Oxygen–oxygen has no
three-body term at all, so it over-coordinates for precisely the reason the pure-O
control demonstrates in isolation, and it does so FASTER than a single oxygen can
gather hydrogens. The H3O mechanism never got its chance to show.

So the honest reading is that **P1 is limited by the campaign's declared scope,
not by the order of the expansion.** Both are real; only one is operative here,
and the prediction that identified the wrong one is on the record above,
unedited.

### What this does and does not say

It does **not** say water fails to form in this model. G1 says the model's water
is bent at 96.77° and G2 says it saturates at two hydrogens — both against full CI,
both holding. What P1 says is that a sandbox with a three-body term for ONE triple
type out of three cannot assemble that molecule from a gas, because the two
untabulated types decide the outcome first.

The named successor is therefore not the four-body term this lane's G2 pointed
at, but the two MISSING THREE-BODY TABLES: `(O,O,H)` and `(O,O,O)`. That is a
change of successor driven by a measurement, and it is worth saying that G2's
four-body finding — which was the more interesting physics — is not the thing
standing between this campaign and its product.

### DISCLOSED

**The O-O curve does not fully converge, and it does not matter — measured, not
assumed.** Its `worst_residual` is 1.3e-4 Ha against the crate's own
`CONVERGED_RESIDUAL = 1e-10`, and the harness printed that and did not act on it,
which is the exact defect shape that constant's doc comment describes. So it was
located (`examples/s2_oo_residual.rs`): every non-converged knot is at `r ≥ 4.34`
bohr, in the dissociation tail where O(³P) + O(³P) is near-degenerate — the same
near-degeneracy the referee met — while the entire well region from 1.6 to 4.07
bohr, `R_e = 2.44` included, converges at 1e-10. The energy moves 4e-5 Ha between
5.16 and 9.0 bohr while the residual there is 6e-5, so the unconverged region is
flat as well as distant. The bond criterion and the aggregation P1 reports do not
read it.

#### CORRECTION (2026-08-30): three of those sentences are wrong, and the verdict survives anyway

*The paragraph above is left exactly as it was published. It was written before
`SolveExit` existed, so its central word was an inference from a residual, and
the lane that wrote it owes the re-reading rather than a defence. Instruments:
`holon-chem --example s3_oo_reexam` (the production 96-knot grid, solved twice)
and `--example s3_oo_trace` (one knot across a ladder of iteration caps), both
built and run in a worktree pinned at `179db95` — an uncommitted refactor of
`fci.rs` was live in the shared tree at the time and a measurement of the shipped
solver cannot be taken in a tree where the shipped solver is being replaced. That
refactor has since landed (`fe18572`) and both readings were re-taken against it:
bit-identical, so the numbers below are main's as well as the pin's.*

**Wrong 1 — it does not stagnate; it gives up.** Of the 96 knots, 21 exit
`IterationCap` and **zero** exit `Stagnated`. "Stagnation" means the subspace
stopped producing new directions — the f64 tier exhausted, which under the
2026-08-30 ruling is the case that overflows to a higher-precision tier.
`IterationCap` means the solve ran out of budget at 1200 iterations with the
residual still falling. Those have opposite remedies and the record named the
wrong one. The ladder settles it: at the worst knot the residual falls
1.08e-4 → 9.53e-11 and the energy settles, converging at **3738 iterations
against a cap of 1200**. Same arithmetic tier, more iterations. This is an
underspend, not an overflow. It is also worse than "not yet finished": the
residual at the shipped cap of 1200 (1.3206e-4) is **ten times worse than the
same solve's residual at cap 300** (1.3299e-5), because under thick restart the
sequence is not monotone. The number the curve publishes is where an oscillation
happened to be standing when the budget ran out.

**Wrong 2 — the boundary is 4.1173 bohr, not 4.34.** The 4.34 came from
`s2_oo_residual.rs`, which sweeps 28 UNIFORM points over [1.6, 9.0]; the curve
`waterquench.rs` loads is 96 knots placed by `table::grid_point` over
[1.5261, 20.0]. 4.34 was the first point of the PROBE that failed, and it was
reported as though it were a property of the curve. The failures are also not a
tail: they run from 4.1173 to 8.5269 bohr **interleaved with converged knots**
(49–52, 54–55 converge inside the band), and every knot from 8.7938 out to 20.0
converges. "Distant and flat" describes a shape the curve does not have.

**Wrong 3 — the bond criterion does read it.** `Sim::refresh_pairs` computes
`e_rel = ke_rel + u(r)` and `bonded = e_rel < 0 && r < r_outer`, where `u` is the
knot energy minus the asymptote and `outer_turning_point` searches for a crossing
in exactly the region that fails. The honest claim is quantitative, not
categorical, and it is below.

**What was right, and is the load-bearing half.** Zero non-converged knots at
`r <= R_e = 2.4309` bohr. The repulsive wall, the well bottom and the whole
binding region converge at ≤ 1e-10. The part of the curve that decides whether a
pair is bound is exact-in-model.

### The score

Both runs are the same code path with one difference — the iteration cap — so
`dE = E_prod − E_ref` is a lower bound on production's error at that knot, and an
equality wherever the reference exits `Converged`.

| | |
|---|---|
| knots exiting `IterationCap` | 21 of 96 |
| of those, scored (reference converged) | 12 |
| unresolved (both runs capped, 6.72–8.53 bohr) | 9 — reference residual 1e-9…3.5e-7, all with \|dE\| ≤ 2.04e-7 Ha |
| worst \|dE\| | **4.315e-6 Ha** at r = 4.2244 bohr |
| worst \|dF\| | **5.362e-5 Ha/bohr**, same knot |
| against kT at the quench's 300 K target | 0.45% |
| against kT at its 3000 K start | 0.045% |
| against the O-O well depth `D_e` = 0.1476 Ha | 2.9e-5 |
| shift it puts on an outer turning point (dU/dR ≈ 1.18e-2 there) | ≈ 3.7e-4 bohr |

### WHY P1's verdict survives, against the corrected characterisation

**Leg 1, and it is exact rather than a margin: the named blocker cannot be moved
by an energy error of any size.** P1's finding is that the mixed arm is limited
by the campaign's declared SCOPE — `(O,O,H)` and `(O,O,O)` are not tabulated and
run pair-only. `Sim::accumulate_three_body` loops over every `i < j < k` with **no
distance gate**, and every triple whose composition has two or more oxygens
increments `fence_untabulated`. That is why the count is exactly
`C(4,2)·8 + C(4,3) = 52` per force evaluation in the mixed arm and exactly
`C(12,3) = 220` in the oxygen control: the fence incidence is an identity of the
box's COMPOSITION, not a function of any potential. A perfect O-O curve leaves
all 52 triples running pair-only. The blocker is a scope fact.

**Leg 2, for the half that is dynamical: the census.** "No seed made water; every
seed made an oxygen aggregate" is an outcome a wrong force could in principle
change. The force that could change it is wrong by 5.4e-5 Ha/bohr at worst, on a
part of the curve where `u` is already within 5e-3 Ha of the asymptote, while the
well that decides binding is converged. 0.45% of the thermal energy the quench is
holding the scene at is not a lever on which aggregate forms.

So the verdict stands and the reason it stands is not the reason the DISCLOSED
paragraph gave. It gave "the criterion does not read the bad region", which is
false. The true reason is "the criterion reads it, and the error there is three
orders below the thermal scale it competes with, while the blocker itself is
combinatorial".

### What does NOT survive, and is separable from the verdict

**P1 ran a curve the crate's own guard would have refused.** `PairCache::get`
asserts `meta.converged()` and panics on a curve whose solve gave up — "a curve
that did not converge is a wrong curve, and handing one back with a flag set is
how the flag stays unread". `waterquench.rs` does not go through the cache; it
calls `generate_pair_table` directly and only WARNS. The score above says the
refusal would have been conservative HERE. The bypass is still a hole, it is
independent of whether this particular curve mattered, and it is named for the
successor rather than patched under a frozen protocol.

**A capped solve can report a residual under the bar.** Knot 48 (r = 4.5673)
exits `IterationCap` at 2.36e-10 and knot 70 (r = 8.5269) at 2.32e-10 — both a
quarter of the derived bar of 1e-9. Under thick restart the residual is not
monotone (the ladder at r = 4.224364 reads 1.0797e-4, 1.3299e-5, 8.8651e-5,
1.3206e-4, 1.1029e-6 at caps 150/300/600/1200/2400 before reaching 9.5346e-11),
so where a capped solve stops is a sample of an oscillating sequence. O-O's curve-level verdict is saved only by `worst_residual` being a
maximum over knots — another knot reads 1.3e-4. A curve whose every knot stopped
under the bar with one of them capped would pass `converged()` and fail
`solve_finished()`, and nothing in the physics forbids one. That is the argument
for keeping the two verdicts separate, and it is now measured rather than
asserted.

**The remedy is a budget, not a tier.** `DAVIDSON_MAX_ITER = 1200` is the binding
constraint at 12 of the 21 knots. Raising it re-banks every artifact that pins
these energies, which is the same cost the ruling attached to lowering the
expansion floor, so it is a campaign decision and not this lane's. Recorded with
its measurement so nobody prices a high-precision referee for a solve that only
needed to keep going. The other 9 knots do not converge at 20000 either; what
they need is not yet known.

**The mixed arm was run once before the timestep report was fixed**, and its
result was identical to every printed digit. The header had been reporting the
EMPTY box's fallback `dt` rather than the placed scene's; the runs were never
affected, because `hold_exactness` drives `dt` to the envelope's value whatever it
starts from. `dt` is now derived from the placed scene and recorded per seed —
1.0772 at the well bottom, held to 0.5386 once the envelope sees the scene.

**The energy bound is loose by five orders in this arm** (3.6e-6 of a 2.4e1 Ha
bound), for the reason C1 records: the curvature envelope's compact corner is
unreachable and sets the absolute cap for every configuration. The momentum
bound, which does not depend on it, is tight to four decades and is the
informative half.

---

## ASSIGNED VERIFICATION — `src/quaternary.rs`'s four-body surface · **DID NOT VERIFY; MODULE REMOVED**

*Assigned to this lane because G2 is its gate and the (O, H, H, H) full CI is its
measurement. Scored on the SAME 40 staked geometries gate G2 uses — eight
directions by five radii around relaxed water — so the comparison was against
referee numbers already in the record.*

**The module was removed from the crate on the lead's order following this
verdict** (`src/quaternary.rs`, its `mod` line, its test, its audit entries), on
the `selector.rs` precedent: a five-parameter fit in a crate whose header declares
zero fitted parameters, failed against forty staked geometries, must not remain
presenting as physics. The record is this section plus git history.

`examples/s2_mbe4_verify.rs` is KEPT and converted. Its verification half is gone
with the module it verified; what remains is the half that was never about that
module and is the successor's instrument — `E_MBE3` assembled for a four-atom
system, `E_FCI` at 1568 determinants, their difference at the same forty
geometries, and the far field. It prints `dE4_true` as the reference column a
candidate surface has to reproduce.

### 1. The sign: a CONVENTION difference, and this file's own wording was the fault

`quaternary.rs` carries `G2_DEFICIT = +0.183`; this file said −0.183. Computed
from one evaluation at `O-H3 = 2.25` bohr on the C2 axis:

    dE4_energy  = E_FCI - E_MBE3 = +0.182863 Ha    <- quaternary.rs's convention
    dE4_binding = E_MBE3 - E_FCI = -0.182863 Ha    <- this file's earlier wording

Same magnitude, opposite sign, and the two sum to 0.0e0. **Their constant is
right and this lane's wording was the ambiguous one** — a number stated in a
convention it does not name is a number that can be read backwards, and a
downstream lane read it backwards. Corrected above.

### 2. Against full CI it does not verify, and the failure is structural

The assignment's bar: the artifact must flip sign AND land within a stated
tolerance of full CI, not merely become repulsive. Measured, with
`residual = [E_MBE3 + dE4_theirs] − E_FCI`:

| | |
|---|---|
| worst \|residual\| over 40 geometries | **0.2755 Ha** |
| mean \|dE4_true\| over the same 40 | 0.1119 Ha |
| T1's interpolation scale, for reference | 2.47e−4 Ha |
| geometries where their term has the **WRONG SIGN** | **11 of 40** |
| geometries where it overshoots by more than 2× | 8 of 40 |

The residual is a **thousand times** T1's scale and **2.5× the mean magnitude of
the term being modelled**. Adding this surface would not correct the MBE3
sandbox; it would make it wrong in a new direction.

**The failure is structural, not a tuning question.** The true `dE4` CHANGES SIGN
with geometry and their form cannot: `G2_DEFICIT * radial_env * hh_env * s1*s2*s3`
is a positive constant times positive envelopes, so it is repulsive everywhere by
construction. Direction 1 — the C2 axis *between* the two hydrogens — is where
that bites:

    r      dE4 true      dE4 theirs     residual
    1.4   -0.026923       0.183000      +0.209923
    1.8   -0.030727       0.183000      +0.213727
    2.2   -0.102448       0.173052      +0.275500
    2.8   -0.172557       0.098105      +0.270662

The true four-body term is **attractive** along that approach, by up to 0.17 Ha,
and their surface returns its maximum repulsion there. It reproduces the ONE
geometry it was anchored to and inverts the physics on a whole approach direction.

### 3. The far field is the part that holds

`R_CUT_4BODY = 6.0` bohr was the item this lane expected to fail, given that its
own three-body table needed `R_HI = 15` for an algebraic tail. It does not fail:
the true `dE4` is 7.8e−5 Ha at 5.9 bohr and 4.9e−5 just outside the cut, falling
to 1.7e−6 by 9 bohr. A 6-bohr cut on the four-body term costs about 5e−5 Ha,
which is inside T1's own scale. The four-body term is genuinely shorter-ranged
than the three-body one, and their cut is defensible.

### The verdict, and the reason it is not a close call

**Does not verify. P1 gains no supplementary arm**, per the assignment's own
condition.

Beyond the numbers there is a discipline point the numbers happen to confirm.
`G2_DEFICIT = 0.183` is this lane's single measured value at one geometry,
hard-coded; `ALPHA_OH = 0.85`, `BETA_HH = 0.15` and `R_CUT_4BODY = 6.0` are
hand-chosen widths. `holon-chem`'s own header says "There is no fitted parameter
anywhere in this crate and no table of chemical results." This module introduces
five. The residual above is what a five-parameter fit to one point does when it
meets forty.

**What would verify — the successor, MBE4-1.** The same thing that made the
three-body term work: a TABULATED exact-in-model surface. `E(OHHH)` is 1568
determinants a point — about four times a water point — over a six-coordinate
space with `S3` symmetry, which is the real cost. Its sizing is already here:

* the reference column, `dE4_true` at the forty geometries, mean `|dE4|` 0.1119 Ha
  and max 0.2284 — a ready-made held-out set, produced by
  `examples/s2_mbe4_verify.rs`;
* the bar: T1's 2.47e-4 Ha, which is where a working surface lands its residual;
* the property that ended the analytic candidate and that any successor must
  carry: **the true term changes sign with geometry**, attractive at 11 of the 40
  and repulsive at the other 29;
* and the domain, from the credited far-field finding below: **start from a
  six-bohr cut, not from this lane's fifteen.**

### The refutation, confirming itself from the other direction

That last point was not in the verification when it was written. It fell out of
converting the instrument, after the module was removed, and it closes the
question rather than merely restating it.

The reference column reports the true four-body term is **attractive at 11 of the
40** geometries. Those are the **same eleven** at which the removed surface had
the wrong sign — not eleven points where a fit was mistuned, but eleven points
lying on the far side of a zero that `positive constant × positive envelopes` has
no way to cross. The structural argument said the form could not change sign; the
data says the eleven failures are exactly the eleven sign changes. Two
independent readings of one fact, and the second was found by looking at what
survived rather than at what was being refuted.


---

## What the campaign says, in one page

**All seven gates discharged.** R1, T1, T2, G1, G2 and C1 HOLD; P1 is branch (b),
reported and investigated.

The first heteronuclear three-body surface exists and is what it claims to be.
`(O,H,H)`, exact-in-model STO-3G full CI, 105,105 solves, checked against an
independent 50-digit referee at 5.9e−12 Ha over 84 result-blind staked
geometries. Out of `Z`, the masses and a basis, with no molecular preset
anywhere:

* **water's SHAPE emerges.** Minimising pairs-plus-`dE3` lands at 1.943467 bohr
  and 96.7738°, against the model's own full CI at 1.9435740105 and 96.75788837 —
  a reference computed by a different route before a single node of the table
  existed. (G1)
* **water's VALENCE emerges.** The third hydrogen refuses at every one of 40
  staked full-CI geometries; there is no well anywhere, against a second O-H bond
  of +0.163077 Ha. (G2)
* **water's FORMATION does not.** No seed of the quench made H2O. (P1, branch b)

### What is NOT claimed

`(O,O,H)` and `(O,O,O)` completeness — the fence is displayed and counted, and P1
shows it is decisive. Quantitative agreement with nature's 104.5° (STO-3G's
in-model angle is the claim, and nature's numbers appear only as labelled
context). Liquid water, hydrogen bonding, anything at scale. And nothing at
order four: the one four-body surface offered to this campaign was verified
against these referee numbers and did not verify.

### The three things this campaign got wrong and caught

1. **`R_HI = 14` would have passed T2 on its own resolution.** A grid maximum is a
   lower bound on its supremum, and the shell re-swept at 5× reads 1.0091e−5
   against a 1e−5 stake.
2. **The tail is not dispersion.** `R^-6` was staked and measured −5.01; the
   discriminator was sharper than the prediction, since closed-shell neon removes
   the algebraic sector entirely rather than leaving an `R^-6` behind.
3. **A conservation gate passed on a box that never moved**, and a spin gate would
   have fired on correct physics. Both now assert the thing they assumed.

And one published number was stated in a convention it did not name — `−0.183` —
which a downstream lane then read backwards.

### THE SATURATION LADDER — a wager-shaped observation, labelled as one

Not a claim. Two points, and the shape they make:

    pair-only cannot saturate HYDROGEN's valence; order three can       (SATURATION-1)
    order three cannot saturate OXYGEN's valence; order four would      (this campaign, G2)

which suggests *valence `v` needs order `v+2`*. That is a wager with two data
points and it is written here as one. Its kill is cheap and named: a valence-three
centre should need order five, so a nitrogen trimer table that saturated NH3 at
order four would falsify it.

Worth keeping separate from P1's result, which is about something else entirely —
P1 is limited by two MISSING THREE-BODY tables, not by the order of the expansion.

### A convergence, credited

The `mbe3-referee` lane measured H2–H2's long range living entirely at order four
as `R^-5` quadrupole–quadrupole. This lane measured `(O,H,H)`'s algebraic tail at
−5.01 and showed the closed-shell swap removes it. Two campaigns, two systems, one
long-range law — and this lane's FIRED `R^-6` stake stays in the source as the
honest route to it. STO-3G has no dispersion to speak of; the model's true tail is
quadrupolar.

### The successors, in the order the measurements put them

1. **`(O,O,H)` and `(O,O,O)` three-body tables.** P1 says these, not order four,
   are what stand between this campaign and its product.
2. **A tabulated exact-in-model `(O,H,H,H)` surface.** 1568 determinants a point,
   six coordinates, `S3` symmetry. The 40 geometries here are a ready-made
   held-out set and G2's 0.183 Ha is its sizing.
3. **The angle as a third coordinate.** `s2_third.rs` measures it beating both `u`
   and `c` on the worst slice by 2.2×; the shipped table's angle axis converges at
   5× per doubling where a C1 cubic should give 16×, and this campaign has not
   explained that.
