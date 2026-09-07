# Pre-registration — CT-3: the term as a table — charge transfer served as CT-2's own sixty-four-node map instead of a declared family; the map's coordinates read off the record, the interpolant IS the term, no parameter chosen; one held-out bond predicted forward against the family's prediction on the same geometry, and the kill is that the family is nearer

*Frozen 2026-09-06, committed ALONE, before any new solve. CT-2 harvested `E_CT = E_exact −
E_noCT` on sixty-four dimer geometries and then fitted a DECLARED angular family to them. The
family missed: S1 read branch (c), 41 of 64 inside tolerance, the twisted dimer of record missed
by MORE than CT-1's term missed it, and the map said why — the transfer follows the acceptor's
PLANE NORMAL and rises when the acceptor turns away, where the family's two lone pairs predict a
peak near the lone-pair angle and a fall beyond it. CT-2's results named CT-3 for the answer: read
the term off the table. This freeze does exactly that and nothing else. The sixty-four measured
numbers become the knots of an interpolant over four coordinates that `SeamModel::ct_angular`
already defines, the interpolant becomes the term the way the Hermite spline IS the pair
potential, and the only thing left to choose is which geometry to hold out. Nothing is fitted;
no amplitude, no exponent, no lone-pair angle. The one number that is not a knot is the exponent
divided out before interpolating, and that is CT-2's own `c_ct` read from `wall_ct2.json`.*

misfits: contacts **M-EXTRAPOLATED-HOLE** (the served term is an attractive exponential times a
bounded shape, and it is walked below its data on the law AS SERVED, with a declared inward
fence walked after it, §2 G-B0 and G-B0F); **M-FIRST-VIOLATION-ONLY**
(the boundedness gate names every class and both legs, and all four transfer legs are reported —
the SERVED walk is the gate and none of the other three is hidden behind it); **M-PLANT-OBS** and
**M-PLANT-SECTOR** (two plants, each with its carrier asserted nonzero in the sector it acts on
AND its analytic reach printed against its stake, §5); **M-CHEAPER-THAN-ITS-PRICE** (the held-out
node's per-node price is CT-2's measured cost at its own separation, §2 S1; a solve returning
under a tenth of its own iteration count times the 55 core-second floor is refused);
**M-EXIT-DISCRIMINATOR** (every solve records its exit and iteration count; a solve at the
iteration cap is VOID); **M-STALE-INSTRUMENT** (every geometry is REBUILT by the map's own builders
and every knot's centers come from its own record, never from a results document);
**M-VACUOUS-SUCCESS** (the interpolant's knot reproduction is stated as a fact about the linear
solve's ARITHMETIC and not about the physics; the double count the serving rule refuses is
MEASURED on every node rather than argued small; the argmin's own discontinuity is measured, not
asserted); **M-EMPTY-SECTOR** (the expectation rule keeps its EMPTY branch: a held-out geometry
whose closure reading finds fewer than two units is not solved, is named, and S1 is VOID);
**M-NULL-MISSTAKE**; **M-FIXED-POINT-TRAJECTORY**; **M-UNTESTED-GAP** (§4); **M-FORMAT-FLOOR**
(every record validates as JSON at twelve significant digits, never a leading `+`);
**M-FLOOR-UNSTAKED** (the transfer floor `1e-6` hartree, carried from CT-2; the knot-reproduction
floor `1e-12`; the resolution floor of the coordinate system itself is the merged sites' own
spread, §0); **M-PLACEMENT-LOTTERY** (cores 0–23 under `taskset`, and no gate is a wall clock);
**M-MAINTENANCE-LENS** (no rent clause is claimed here; the table is an instrument, not a
maintenance reading); **M-TRUNCATION-AS-ERRORBAR** (the leave-one-out gauge is an independent
referee, never the interpolant's own residual — which is zero by construction);
**M-BARE-CHARGE**, **M-HOMOG**, **M-COND-PROBE**, **M-DEVICE-CLASS**, **M-VOLUME-SCALE**,
**M-PROVENANCE-OVERREACH**, **M-GAUGE-LAUNDER**, **M-PARITY-PROTECT**, **M-LOOP-BLIND**,
**M-ONE-MODEL-DELTA**, **M-IDLE-CALIBRATED-TIMEOUT**, **M-PROBE-THE-RESOURCE**. Not contacted:
the rest of the registry.

## 0. What is built and measured

**The coordinates.** For one transfer contact — a hydrogen `H`, the acceptor's oxygen `O_a`, the
hydrogen's own oxygen `O_d`, and the acceptor unit's two hydrogens `h₁, h₂` — with
`u = (H − O_a)/r`, `b̂ = unit(h₁ + h₂ − 2O_a)` the acceptor's bisector and
`n̂ = unit((h₁−O_a) × (h₂−O_a))` its plane normal:

```
r    = |H − O_a|                                          bohr
c_d  = cos θ_d = (O_d−H)·(O_a−H)/(|O_d−H||O_a−H|)         the donor's alignment
p    = u·b̂                                                the acceptor's polar alignment
q    = 2(u·n̂)² + p² − 1                                   the acceptor's azimuth
```

Every one of these is a definition `SeamModel::ct_angular` already uses; none is new. `q = +1`
puts the donor on the acceptor's PLANE NORMAL, which is the direction CT-2's map named; `q = −1`
puts it in the acceptor's own plane; `q = 0` on the bisector at either end.

**Why four coordinates and not three, and why these four.** Measured on the map's own records
before this freeze was written, and the reason the answer is not the two angles and a distance
that CT-2's results and the plan both name:

* **Three coordinates cannot carry this map.** Dropping the azimuth and keeping
  `(θ_d, θ_a, R)` collides twenty-four of the sixty-four nodes onto twelve sites — 52 distinct
  sites where four coordinates give 60 — with spreads up to `1.076e-2` hartree, which is LARGER
  than the value at those sites. Eight of those twelve groups are resolved by the azimuth and
  four are not, and the four that are not are the poles below. The tilt family and the twist family
  are two orthogonal great circles of the acceptor's orientation sphere, and a single acceptor
  angle cannot tell them apart: `tilt_R2.7_t60` reads `−21.535` mHa where `twist_R2.7_t60` reads
  `−11.072` at the same three coordinates. The azimuth is not a refinement here; it is the axis
  the map's second family varies.
* **`cos θ_d` and `p` rather than the angles.** Fifty of the sixty-four nodes sit at `θ_d = 180°`
  EXACTLY and every tilt node at an exact multiple of 30°. `dθ/d cos θ` is singular at
  `cos θ = ±1`, so a table in the angles has an infinite force at the geometries the map is
  densest at.
* **`q` rather than an azimuth.** The azimuth folded by the acceptor's own symmetry is
  `atan2(|u·n̂|, |u·t̂|)`, whose absolute values put kinks exactly on the two sheets the map
  samples — the tilt family at `u·t̂ = 0`, the twist family at `u·n̂ = 0`. `q` is the same
  information, smooth everywhere, invariant under both of the acceptor's mirrors and under
  relabelling its hydrogens, and it vanishes at both poles by construction.

**The knots.** One per node of CT-2's map, its value that node's own `E_CT` — the exact total
minus the closed-sector total, the records' own rule. Fifty-one carry `e_ct` directly in
`ct2/node_*.json`; the thirteen of record carry the exact total in `ct2/gd0_*.json` and the
closed-sector total in `ct1/sector_*.json`, and their `E_CT` is that difference. No third source.
Every knot's centers are the record's own and every geometry is rebuilt from the map's builders
before its numbers are used.

**The pole rule, and the coordinate system's own resolution.** Four groups of nodes land on ONE
site, and every one of them is a POLE of the acceptor's azimuth — `u` along the bisector, where
`q = 0` for both sheets by construction and the azimuth is genuinely undefined. They are merged to
their mean and the SPREAD is recorded, because a table cannot hold two values at one site and
averaging one away in silence would launder a coordinate degeneracy into an interpolation:

| the group | spread (hartree) |
|---|---|
| `twist_R2.7_t0` / `linear_R2.7` | `3.036e-6` |
| `tilt_R2.7_t180` / `twist_R2.7_t180` | `7.091e-4` |
| `twist_R3.4_t0` / `linear_R3.4` | `3.862e-7` |
| `twist_R3.4_t180` / `flipped_R3.4` | `7.985e-6` |

Sixty-four nodes, SIXTY sites. The largest spread, `7.091e-4` hartree, is what four coordinates
cannot resolve about this map: at `q = 0` the acceptor's azimuth about its bisector is free, and
the donor's SECOND hydrogen — off-axis, and outside these coordinates — sees it. That number is
the table's own floor and nothing below it is a reading.

**The interpolant, declared.** The cubic polyharmonic spline with a linear polynomial tail, on
the four coordinates scaled to the box the knots span:

```
E(y) = −S(ỹ)·exp(−c₀·r)
S(ỹ) = Σ_i w_i ‖ỹ − x̃_i‖³ + w_n + Σ_j w_{n+1+j} ỹ_j
ỹ_j  = (y_j − lo_j)/(hi_j − lo_j)
```

with the axis box `r ∈ [2.402796, 6.172148]` bohr, `c_d ∈ [−1, 0.334255]`, `p ∈ [−1, 1]`,
`q ∈ [−1, 1]`, all from the knots themselves, and `c₀ = 1.400000` per bohr read from
`ct2/wall_ct2.json`. Three reasons, each measured rather than assumed:

* **`ρ³` carries no shape parameter.** Every Gaussian, multiquadric and inverse-multiquadric
  kernel has a width that would have to be chosen, and this programme does not type a number a
  record does not carry. `ρ³` has none. It was also the best of the six kernels swept on the
  map's own leave-one-out.
* **It is C² everywhere, knots included**, so the force is continuous and differentiable AT the
  data — where a simplex interpolant puts a facet and a Shepard weighting puts a flat spot.
* **The exponential prefactor carries the decay.** Tabling `E_CT` directly was measured at three
  times the leave-one-out error and leaves the far field a polynomial instead of a decay.

**The serving rule, and which contact defines the coordinates.** ONE reading per UNORDERED pair
of units. THE PAIR'S COORDINATES ARE THOSE OF ITS SHORTEST CROSS-UNIT H···O CONTACT, taken over
all four such pairs — both of the first unit's hydrogens against the second unit's oxygen and both
of the second's against the first's. That contact fixes which unit is the donor and which the
acceptor, hence `O_d`, `O_a` and the frame `(b̂, n̂)`, hence every one of `r`, `c_d`, `p` and `q`.
Nothing else about the pair enters. The engine's `accumulate_seam` and this runner take the same
argmin, and G-A0 measures what it costs where it ties. That is what the map measured: `E_CT` is a
property of the DIMER, one number per node, already carrying whatever transfer runs in either
direction at that geometry. Serving it once per ORDERED pair counts it twice, and the harvest
measured what that would cost: up to `2.505e-2` hartree on the twist family, larger than the
node's own `E_CT`. The contact is an ARGMIN and an argmin is discontinuous where it ties; the jump
is measured (§2 G-A0), not asserted small.

**The engine.** `seam.rs` gains `CtMode` (the three shapes of channel 6 — `Pair`, `Angular`,
`Table` — exclusive, selected by `SeamModel::ct_mode`), `SeamModel::ct_table_on`, `ct_coords`
(the four coordinates and their gradients on the five atoms), `CtTable` (`begin`/`knot`/`finish`
like the pair tables, the saddle solve, `eval`, `eval_grad`, `serve`, `deepest`, `reach`), and
`bounded_ct`/`hole_ct` — the same walks with the transfer row supplied by the caller, because the
three shapes are not the same function of one separation. `sim.rs` serves the table in
`accumulate_seam` with analytic forces on the five atoms and the switch on the contact.
`checkpoint.rs` v14 adds `seam_ct_table_on`; the TABLE is not in the checkpoint, for the same
reason the pair tables are not. The door: `holon_ct_table_begin`/`knot`/`finish` and
`holon_set_seam_ct_table`. `ct_table_on` is `false` in every record written before this freeze, so
every one of them still reads `Pair` or `Angular`, bit for bit.

**The law, and why one thing in it IS re-fit here.** FIELD-9's three walls are HELD. The transfer
row is the table. The dispersion is an exact `C₆ = 0`, FIELD-6's rule. The TWO CONTACT TERMS ARE
RE-FIT (§2 C1), and the reason is a measurement made before this freeze: **the served boundedness
walk was RUN on CT-2's own contact terms with the table in place, and it REFUSED** — the H–O class
falls 2.2 kT below its fit floor with its minimum at 2.48 bohr. CT-2 clamped its amplitudes
(`P_HO = 22.174044 e^{−2.44 r}`, `P_HH = 0.0169714 e^{−1.02 r}`; the record's own digits —
`CT2_RESULTS.md`'s gate table prints the H–O amplitude as `22.17048`, a transposition of
`wall_ct2.json`'s `22.174044`) against ITS OWN transfer row, which is roughly half the table's
depth at those radii, and a clamp taken against the shallower term does not bound the deeper one.
The lever is CT-2's own C1 machinery applied to CT-3's transfer row, and it is IN this freeze
rather than deferred because nothing about the held-out node has been seen. Everything else is
held: no wall, no exponent of a wall, no charge and no dispersion coefficient moves.

## 1. The expectation, written before the solve (M-EMPTY-SECTOR discharged)

Written separately, each part, before the held-out node is built or solved.

The **table's own knots** reproduce exactly: that is arithmetic, not a result, and it is stated so
(§2 G-T0). The **held-out node** is expected to sit inside the closure identity with TWO units;
if the engine's closure reading finds fewer than two, the node is named in
`ct3/outside_<name>.json`, is NOT solved, and S1 is VOID rather than failed. Its charge transfer
is expected attractive (`E_CT < −1e-6` hartree), its shortest cross-unit H···O being
`3.158687` bohr, well inside CT-2's 5-bohr expectation. The **table** predicts
`E_CT = −2.312788e-2` hartree there and the **family** predicts `−1.711144e-2` on the same
geometry: both are filed in `ct3/prediction.json` before the solve, and they differ by
`6.016437e-3` hartree, which is what makes the node decisive (§2 S1).

## 2. Gates

- **G-T0 — the interpolant is an interpolant.** `|E(x_i) − v_i| ≤ 1e-12` hartree at every one of
  the sixty SITES. This is a statement about the LINEAR SOLVE's arithmetic and about nothing
  else; an interpolant interpolates by definition, and the gate exists so that a singular or
  ill-conditioned system cannot pass unnoticed (M-VACUOUS-SUCCESS).
  witness: none (a residual at sixty points)
- **G-T1 — every NODE of the map.** `|E(y_i) − E_CT_i| ≤ 1e-12` hartree at every one of the
  fifty-six unmerged nodes; each of the eight nodes in a merged group reads its site's mean, so
  its miss is exactly half that site's spread and is reported as such, per node, with the spread
  beside it. Nothing is asserted zero that is not.
  witness: none (a residual per node, and four spreads)
- **G-B3 — the force is the derivative of the served term.** Worst relative
  `|analytic − central difference|` at `h = 1e-5` bohr over five atoms × three coordinates × 64
  nodes `≤ 1e-8`; and the gradients sum to zero on every node (the term posts no net force).
  witness: none (a finite difference on 960 coordinates, and a translation identity)
- **G-C1 — the engine serves what the table says.** `|e_seam(engine) − the runner's own sum| ≤
  1e-10` hartree on every node, the table entering both sides as ONE reading per unordered pair
  of units; every node served with two units (EXACT count). The double count that serving each
  ORDERED pair would have added is measured and reported beside it, per node, with the worst
  named.
  witness: none (arithmetic, a count, and a measured quantity the rule refuses)
- **G-A0 — the argmin's own seam, measured.** The donor swept through a full turn in 3,601 samples about
  its own oxygen at each of `R_OO ∈ {2.7, 2.9, 3.1, 3.4}` Å; the largest jump in the served value
  where the CONTACT atom changes hands is reported in hartree beside the tolerance of the nodes
  it sits among. This gate has no pass: it is a measurement of a known discontinuity, and a jump
  at or above `5e-4` hartree (the map's own tolerance floor) is a REFUSAL of the serving rule for
  the dynamics and is named as one.
  witness: none (a maximum over a sweep, against a stated floor)
- **G-B0 — bounded, walked on the law AS SERVED, every class and every leg named.** The gate is
  the SERVED walk: `bounded(q_h, r_min, kT)` on the full law with the transfer row the
  interpolant ITSELF at each walk radius, minimised over every angular geometry a real contact
  can present — `cos θ_d` over its full physical `[−1, 1]`, and `(p, q)` over the REACHABLE set
  rather than the knot box, since `u` is a unit vector in the acceptor's orthonormal frame and
  therefore `|q| ≤ 1 − p²`, which makes the box's corners at `|p| = |q| = 1` unreachable. The
  angular minimum is taken on a grid of 41 points per axis, a coarse pass then a fine pass over
  one coarse cell; a minimum over a grid is an UPPER bound on the true minimum, so the resolution
  is reported with the depth and never left implicit. Three further legs are walked beside it and
  none is hidden behind another (M-FIRST-VIOLATION-ONLY): the STRICT leg, the largest knot shape
  `−max_i S_i · e^{−c₀ r}` carried inward, which describes a term the law does not serve — the
  largest shape of this map sits at 4.5 bohr where the twist family decays more slowly than the
  divided-out exponent; the NEAR leg, the largest shape at or inside the H–O class's own fit floor
  `r_min = 2.780741` bohr, carried the same way; and the LINEAR leg, the table at
  `(c_d, p, q) = (−1, −1, 0)`, which is CT-2's own letter carried for comparability. `q_h =
  2.313804e-1`, `kT = 9.278758e-4` hartree, `r_min = [4.724315, 2.780741, 1.314606]` bohr, all
  from `wall_ct2.json`. The monotone walk is reported beside the boundedness walk on both the
  served and the fenced legs.
  witness: none (an order on a grid, with a depth, every class, four legs and a stated angular resolution)
- **G-B0F — the declared inward fence, walked only if G-B0's served leg refuses.** The shape
  `S(y)` is read at the innermost knot's separation `r_clamp = 2.402796` bohr for every `r` below
  it while the prefactor `exp(−c₀ r)` keeps running, and the served walk is repeated. It is a
  stated fence and not a fit: nothing is chosen, the value held is the interpolant's own at the
  knot floor, and it caps the shape the way `reach` caps at `r_cut`. Its price is a force
  discontinuity at `r_clamp` — the energy is continuous, `dS/dr` is not — and that jump is
  measured in hartree per bohr and reported, never asserted small. The fence CANNOT change a
  violation that lies at or outside `r_clamp`, and the freeze says so here rather than after the
  fact: a refusal above the innermost knot is a statement about the table's own DATA and no
  inward rule touches it.
  witness: none (the same walk, under a stated rule, with the rule's own price measured)
- **G-R0 — the reach.** The table's own reach at the `1e-10` hartree budget, `ln(max_i S_i)/c₀`,
  reported against the law's; the served law's reach is the larger, and LIQUID-1 Amendment 2's
  C² switch at `r_cut = 14.0` bohr is what the liquid would read.
  witness: none (arithmetic on the table's own knots)
- **C1 — the two contact terms re-fit UNDER the gate, with the table held.** The remainder each
  class is fit on is `ΔE_exact − [E_q(g) − E_q(40)]_engine − wall(g) − TABLE(g)` on all sixty-four
  nodes; the table is EXACT at the knots, so that remainder is the map's own residual and not a
  fit's. Two classes, cross-unit H–O and cross-unit H–H, exponents on CT-2's own grid
  `0.50 ..= 4.00` step `0.02` per class (176 values, 30,976 pairs), amplitudes by weighted least
  squares in closed form with weights `1/max(|ΔE_exact|, 5e-3)²` and `P ≥ 0` — and for every grid
  point each amplitude is CLAMPED at the largest value the SERVED walk admits with every other
  term held, by bisection on `[0, P_ls]` to `1e-6` relative, the two classes iterated to a fixed
  point over at most 20 rounds. The unconstrained fit is recorded beside the constrained one with
  the ratio of their residuals; a ratio above `2` says the remainder wants a shape a bounded
  exponential cannot take, named, not a failure. An exponent at either edge of its grid is
  reported as the GRID's number, not the physics'. Tolerance CT-2's own
  `max(0.25·|ΔE_exact|, 5e-4)`. **(a)** the line at 2.7, 2.9 and 3.1 Å all within AND at least 50
  of 64 within; **(b)** the line within, fewer than 50; **(c)** the clamp leaves the line
  unserved. Pre-committed: on **(c)** the law is NOT admitted for a liquid and CT-3 reports the
  table as a dimer instrument.
  witness: none (a fit against a stated tolerance, a count, a ratio, and a clamp)
- **G-B0W — the WHOLE served law, walked; LIQUID-2's admission gate.** `bounded(q_h, r_min, kT)`
  returns `None` on the table plus the re-fit contacts plus the three walls plus the charges, the
  transfer row served exactly as G-B0's is. RUN even though the clamp makes it pass by
  construction, because a gate that is assumed is not a gate (M-VACUOUS-SUCCESS). The monotone
  walk is reported beside it. `None` admits the law for LIQUID-2; anything else refuses it and
  names the class.
  witness: none (an order on a grid, with a depth, every class)
- **S2 — the held-out node's FULL dimer energy, predicted forward.** The whole law's prediction of
  `ΔE_exact` at the held-out geometry, filed in `prediction.json` beside S1's `E_CT` prediction
  BEFORE the solve, with the field row and the seam row separated. Two bars, both derived, and
  which of them is tighter is MEASURED rather than assumed (a per-unit ratio needs its
  fixed-target control beside it). **The band** is the re-fit law's own worst in-sample miss over
  the sixty-four, taken the tighter of two ways: ABSOLUTE, and that same worst expressed as a
  FRACTION of its own node's `|ΔE_exact|` and applied to the predicted total — the absolute worst
  sits at the innermost node, where every term of the law is an order larger than at the held-out
  geometry, so quoting it alone would be a bar borrowed from a different regime. **The
  comparative bar** is CT-2 S2's own letter, `max(0.25·|ΔE_exact|, 5e-4)`; the programme's two
  landed forward predictions missed by 10.9 % (CT-1 S2) and 22.9 % (CT-2 S2), and both were read
  against that letter. **(a)** inside the TIGHTER of the two; **(b)** inside the looser only;
  **(c)** neither.
  witness: none (a prediction filed before its measurement, against two derived bars)
- **S1 — the held-out bond, predicted forward, against the family on the same geometry.** The
  held-out node and its stakes are §2's own subsection below.
  witness: none (a prediction filed before its measurement)

### S1 — the held-out node, its rule, its price and its stakes

**The node, by a rule stated here.** Among geometries of the map's OWN builder family
`twisted(R, twist, tilt)` with the donor bent — `R` on the map's own tenths of an ångström from
2.7 to 3.4, every angle a whole multiple of 5° — the node is the one that MAXIMISES
`|E_table − E_family|` subject to two conditions: it lies inside the convex hull of the sixty
knot sites in the scaled coordinates, so the table interpolates and does not extrapolate; and its
distance to the nearest site lies in `[0.121140, 0.402992]`, the MEAN and the MAXIMUM of the
knots' own nearest-neighbour spacings. The upper condition is why the rule is not simply
"farthest inside the hull": without it the rule picks a corner five nearest-neighbour spacings
from any knot, where the leave-one-out gauge below covers nothing and no stake could be derived
(M-UNTESTED-GAP). The lower condition is why it is not simply "most decisive": without it the
rule picks a geometry 0.028 from an existing knot, where the table is repeating its neighbour and
the test is trivial.

**The node.** `twisted(2.7 Å, 20°)` then tilted `85°`, the donor unbent —
`twistbent_R2.7_tw20_t85_d0` in the map's naming. Its coordinates are `r = 3.158687` bohr,
`c_d = −1.000000`, `p = −0.087156`, `q = +0.760226`: three quarters of the way from the twist
sheet toward the plane normal, on an azimuth NO node of the map carries, at a twist angle no node
of the map carries. Its distance to the nearest site (`tilt_R2.7_t90`) is `0.127562`, just above
the band floor. `R_OO = 5.102261` bohr; cross-unit H–O distances `3.158687, 5.089895, 5.669603,
6.000859` bohr.

**Its price.** CT-2 measured twelve nodes at this separation: the exact solve from the product
start `1,783–2,893` core-seconds at 16–26 Davidson iterations, the closed-sector solve
`1,555–2,257` core-seconds at 13–19, `3,337–5,151` core-seconds per node in total, mean `3,843`.
This node is priced at that band and nothing wider; CT-1's `1,450–57,600` core-second band is the
outer bound the discipline carries. A solve returning under `0.1 × iterations × 55` core-seconds
is refused (M-CHEAPER-THAN-ITS-PRICE), and a solve at the 300-iteration cap is VOID
(M-EXIT-DISCRIMINATOR).

**The gauge the stake is derived from.** Each of the sixty sites was removed in turn, the table
rebuilt from the remaining fifty-nine, and the removed site predicted. That is the only number in
this campaign that says anything about a geometry the map does not carry. Measured: WORST
`2.277118e-3` hartree (at `twist_R2.7_t120`), MEDIAN `4.172969e-5`. On the same sixty-four nodes,
under CT-2's own S1 tolerance `max(0.25·|ΔE_exact|, 5e-4)`: the family, which was fitted on all
sixty-four, lands 41 of 64 with a worst miss of `9.031149e-3`; the leave-one-out table, which
never saw the node it is scored on, lands 64 of 64 with a worst of `2.277118e-3`, and is nearer
than the family on 60 of the 64.

**The stakes, with the arithmetic.**

- **(a)** `|E_table − E_CT| ≤ 2.277118e-3` hartree AND `|E_table − E_CT| < |E_family − E_CT|`.
  The band is the table's own WORST leave-one-out miss over its sixty sites — not a typed
  tolerance and not the interpolant's own residual, which is zero by construction. The
  comparative half is the GANTT's kill, read the way it was written.
- **(b)** the table is nearer than the family but outside the band.
- **(c)** the family is nearer — **the kill FIRES**, and the table is not the term.

Two derived facts about this pair of stakes, both arithmetic on numbers filed before the solve:

1. The two predictions differ by `6.016437e-3` hartree, so half the gap is `3.008218e-3`. The band
   `2.277118e-3` is SMALLER than that half, so branch (a) and "the family is inside the same
   band" are mutually exclusive. Whichever way the solve falls, it separates them.
2. The looser comparative bar is named beside the tighter one and is not the stake. CT-2's family
   missed ITS held-out node's transfer term by `8.1e-4` hartree on a measured `−4.098e-3`, which
   is 19.8 %; 19.8 % of the table's `−2.312788e-2` prediction here is `4.579e-3` hartree, twice
   the band. The band is the bar because the leave-one-out gauge is this table's own record, and
   a bar taken from the family's performance on a different geometry is a weaker claim.

The TOTAL is scored beside the transfer term under CT-2 S2's own letter: `|E_pred − ΔE_exact| ≤
max(0.25·|ΔE_exact|, 5e-4)`, with the full law's parts filed separately in `prediction.json`.

## 3. What each outcome means

S1 (a) with G-T0, G-T1, G-B3, G-C1 and G-B0 passing says the transfer term is the map, that
serving the measurement beats fitting a shape to it by the margin the leave-one-out gauge
predicted, and that LIQUID-2 has its term. S1 (b) says the table is the better instrument and the
gauge under-reported its own error — the interesting failure, and the one that would say the
azimuth axis is thinner than the leave-one-out sweep could see, since no knot sits between the
two sheets at high tilt. S1 (c) is the kill as the plan wrote it: a table built on sixty-four
measurements is beaten, on a geometry inside its own hull, by a family that misses 23 of those
same sixty-four. That would say the four coordinates are the wrong four and would name the
azimuth as the suspect, because the held-out node's azimuth is the coordinate the map does not
sample.

C1 (a) with G-B0W returning `None` is the law LIQUID-2 would run on: the map served as the
transfer term, the contacts re-clamped against it, bounded in every class. C1 (b) says the
re-clamped contacts serve the bond's own separations but not the map's edges, which is a reading
about the remainder's shape and not about the table. C1 (c) says the gate's clamp cannot admit a
contact term that also serves the line, and the pre-committed outcome is the lead's: the law is
not admitted for a liquid and CT-3 reports the table as a dimer instrument. A constrained /
unconstrained residual ratio above 2 says, in either case, that the remainder after the table
wants a shape a bounded exponential cannot take — the reading that names the next shape.

G-B0's served leg returning `None` admits the law for LIQUID-2. Its refusing — under the
declared fence too — does not, and the pre-committed outcome is the lead's: the law is NOT
admitted for a liquid, CT-3 reports the transfer term as a DIMER INSTRUMENT, and the inner fence
is named for the next freeze rather than invented in this one's postmortem. The three legs beside
the gate are what make that verdict readable rather than a bare word. A refusal on the strict leg
alone would say only that an outer knot's shape is large; a refusal on the served leg says the
law the dynamics would actually integrate has a well the gate does not admit, and WHERE the
violation sits decides what can be done about it. A violation INSIDE the knot range — at or
outside `r_clamp = 2.402796` bohr — is a statement about the table's own measured data and no
inward rule reaches it; a violation below the knot range is an extrapolation and G-B0F is the
rule that answers it. Which of the two it is must be read off the walk and stated, not assumed.

G-A0 measuring a jump at or above `5e-4` hartree would refuse the serving rule for the dynamics
even if every other gate passed: a law that steps when a contact changes hands does not conserve
energy, and the map's per-dimer measurement would then need a per-pair carrier that this freeze
does not have.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

Sixty-four dimer geometries at one basis become sixty knots of a four-dimensional interpolant;
one bond is predicted once; and then a liquid of 128 waters would run on it. Three specific
things the map cannot say and this term therefore cannot serve, each named rather than discovered
later:

1. **No node has a pair donating in BOTH directions at once.** Every geometry of the map names a
   donor and an acceptor. The serving rule gives such a pair ONE reading, at its shorter contact,
   and the liquid has pairs the map never posed.
2. **The azimuth between the sheets is unsampled at high tilt.** The tilt family sits at one
   azimuth and the twist family at the other; between them, at `|p| < 1`, there is no knot. The
   held-out node is chosen there on purpose, and one node is one node.
3. **Below `r = 2.402796` bohr and above `6.172148` bohr the table extrapolates**, held only by
   the exponential prefactor and the seam's own switch. G-B0 and G-R0 are the fences and both are
   measured, not assumed.

## 5. Plants

Each plant names its carrier, asserts it nonzero in the sector the plant acts on, and prints its
own ANALYTIC REACH against its stake — the reach being what the plant must move the observable by
if the mechanism is what the freeze says it is, computed from the freeze's own equations before
the plant is run (FLUID-1's correction: a plant's stake is derived from the observable's reach,
never typed).

- **(i) One knot moved by its own tolerance.** The knot at `linear_R2.9` has its value moved by
  `δ = max(0.25·|E_CT|, 5e-4) = 2.334195e-3` hartree and the table is rebuilt. **Analytic reach:**
  the interpolant passes through its knots exactly, so the served value AT THAT SITE moves by
  exactly `δ` and by nothing else. **Stake:** `|ΔE(site) − δ| ≤ 1e-12` hartree. **Carrier:**
  `δ = 2.334195e-3 ≥ 1e-6` (the transfer floor), asserted nonzero in the sector the plant acts on
  — the served value at that site, `−9.336780e-3` hartree.
- **(ii) The sign of the transfer term.** `SeamPlant::FlipChargeTransfer` in the engine, at the
  linear 2.9 Å node. **Analytic reach:** the seam energy moves by `2·|table(node)| =
  1.867356e-2` hartree, that node carrying exactly one transfer reading under the serving rule.
  **Stake:** `|Δe_seam − 2·|table(node)|| ≤ 1e-12` hartree. **Carrier:**
  `|table(node)| = 9.336780e-3 ≥ 1e-6`, asserted nonzero in the sector the plant acts on — the
  engine's `e_seam` row at that geometry.

## 6. Discipline

Engine: `seam.rs` (`CtMode`, `ct_coords`, `CtTable`, `bounded_ct`/`hole_ct`), `sim.rs` (the table
in `accumulate_seam`, its five-atom forces, the switch), `checkpoint.rs` (v14), `lib.rs` (the
door); gates in `seam.rs`'s own suite (the interpolation, the finite-difference gradient, the
refusals by name, the mode selector). Runner `holon-render/examples/ct3_harvest.rs`: `gate`
(G-T0, G-T1, G-B3, G-C1, G-A0, G-B0, G-B0F, C1, G-B0W, G-R0, S2's band, the leave-one-out gauge, both plants, `ct_table.json`
and `gate.json`), `predict` (`prediction.json` for the held-out node BEFORE its solve, carrying S1's `E_CT` prediction and S2's full-energy prediction together), `solve`
(the closure identity first, then the exact solve checkpointed on its own and the closed-sector
solve, `node_<name>.json`), `read` (`prediction_check.json`, S1's branch). `solve` REFUSES to run
without `prediction.json` on disk. Records under `conformance/water_observatory/ct3/`, every file
valid JSON at twelve significant digits, never a leading `+`. Cores 0–23 under `taskset`; no gate
is a wall clock and no cost is stated in calendar time. Results `CT3_RESULTS.md` with the
instruments. No number enters from outside the engine, its own solver and the map's own records;
the one coefficient the table does not hold as a knot is `c₀ = c_ct` read from
`ct2/wall_ct2.json`, with its path in the record.
