# Pre-registration — EDGE-0: the edge exists — a fluid element whose closure MOVES AS ONE, its coexistence read as a block-density histogram against two controls, its surface tension read TWICE by independent instruments (Laplace's law and the capillary spectrum), its interface width tracked for stationarity, its droplet's lifetime measured, and gravity entered as a momentum bias posted into the ledger

*Frozen 2026-09-07, committed ALONE by the lead, before any counted run. DRAFTED by a delegate —
the carrier, the six instruments, the two labelled screens, and this document's stakes, plants
and branches — and the lead owns the commit and the release of the counted runs. The `screen`,
`screen_chart` and `gate` phases were run before this freeze and are cited by name wherever their
numbers appear; the counted runs were not. FLUID-1 read branch (c) — a pair closure buys a factor
of two in the Schmidt number and no more — and left TWO INSTRUMENT FINDINGS for the next carrier,
in its own words: "the closure must be able to wait" and "the count is capped by the arm, not the
rent". This freeze builds both, finds that they are not enough and why, adds the rule the closure
grammar itself states, and asks the edge question `GANTT2.md` sets: does this element have a
DEFINED EDGE — two phases, an interface of finite and stationary width, and a surface tension
that two independent instruments agree on? The credited family is the lattice gas with a
liquid–gas transition (Appert & Zaleski 1990), the immiscible lattice gas (Rothman & Keller
1988), Laplace's law and the capillary spectrum as the two independent readings of σ and a body
force as a momentum bias (Rothman & Zaleski 1997), and the rest particle with the collisions that
create and destroy it (Frisch, d'Humières, Hasslacher, Lallemand, Pomeau & Rivet 1987,
FHP-II/III). What is new is that the cohesion is FLUID-1's bond rule read from exact molecular
solves, that a closure moves or waits AS A WHOLE by a rule which keeps every conserved integer
exact, and that every band a branch is read against is measured from the run's own controls
rather than typed.*

misfits: contacts **M-EMPTY-SECTOR** (three times over: branch C (b) is the empty branch by name,
branch D1 (c) names a droplet that never bound, and C's third leg VOIDS rather than passes when
the band it is read against is measured exactly zero — which is what the confirmation pass
measured on the one-phase control's own seed spread); **M-PLANT-OBS** and **M-PLANT-SECTOR** (two
plants, §5; each names the sector it acts on, its carrier is asserted nonzero in that sector, and
each stake is DERIVED from the analytic reach of the observable the plant acts on — FLUID-1's own
correction, whose two plants were typed at 0.20 against reaches of 0.085 and 0.038 and were
unreachable before a single step ran); **M-CHEAPER-THAN-ITS-PRICE** (every counted run priced on
FLUID-1's measured `chart_seconds_per_step` at `L = 256`, scaled to this box and multiplied by
this carrier's own overhead, measured on the first 200 steps IN THE SAME AUDITED MIX the counted
runs advance in and written to `price.json` before the counted steps; the CLUSTER rule's step is
priced apart from the pair rule's on the same scene, because component finding is the new cost);
**M-EXIT-DISCRIMINATOR** (a fit that refuses is a refusal by name and never a number; the Laplace
line and the capillary line each carry their own `R²`; a droplet that reaches its cap is reported
CAPPED and never as a lifetime); **M-STALE-INSTRUMENT** (the bond amplitude table and CT-2's
retention are READ from the molecular tier's records at run time with the path and the field
printed beside every value; FLUID-1's cost per step is READ from its own `price.json`; nothing is
retyped); **M-VACUOUS-SUCCESS** (G0 asserts that bonds formed, that the wait fired, that rest
particles were created, that gravity posted, that components were walked and moved as one and
that one larger than a pair was built, so a ledger gate cannot pass on a carrier that did none of
it); **M-NULL-MISSTAKE** (every band is measured on a control run at the same chart, the same
seeds and the same window — the one-phase control for the spread, the PAIR arm for the variance
ratio — and never against zero); **M-FIXED-POINT-TRAJECTORY** (four seeds, FLUID-0's rule; every σ
and every spread is reported with its own spread over them); **M-UNTESTED-GAP** (§4);
**M-FORMAT-FLOOR**; **M-FLOOR-UNSTAKED** (the dip's floor is a rule on the MEASURED control, not
a fraction; the width's stationarity bar is in cells per thousand steps and is stated; the
droplet's cap is stated and a cap is a refusal); **M-VOLUME-SCALE** (`L = 128` for every counted
scene so no comparison crosses a size, and the block charts at `b ∈ {4, 8, 16}` are node LG's;
the screens ran at `L = 64` and the chart's confirmation pass was repeated at `L = 128` for
exactly that reason; the edge is read at `L = 48` because its probe is one exhaustive perturbation
per movable cell and its cost is quadratic in the box, and that difference is named wherever the
edge is quoted); **M-BASE-RATE-OMITTED** (the release rate and every one of its prongs, the wait's
attempts and its successes, the components walked and moved, and the refusal count of every
gravity draw are reported beside every distribution); **M-FIRST-VIOLATION-ONLY** (every gate names
every failing leg, worst value first, through `holon-campaign`'s `Gate`); **M-TAG-AS-PROPERTY** (a
bond is a recorded pair of slots with its arm and its acceptor role, never a label on a particle);
**M-NONBIJECTIVE-STEP** (every collision table is VERIFIED bijective and sector-preserving at
construction, never assumed from how it was built); **M-HOMOG** (the torus is homogeneous and the
droplet and slab scenes are the structurally inhomogeneous ones; both controls run on the same
torus at the same density); **M-MAINTENANCE-LENS** (contacted by keyword: the rent clause here is
a break probability set from a retention and measured back on the carrier by a held-geometry
scene — the bond's persistence is the rule's own count, never a lens's verdict on a trajectory);
**M-DEVICE-CLASS** and **M-PLACEMENT-LOTTERY** (cores 24–31 only under `taskset`, one whole scene
per worker, single-threaded per scene; the wall clock is reported and the price is checked against
it); **M-ONE-MODEL-DELTA** (one collision law is named and its relation to FHP-I is measured, not
asserted; the finding is about THIS law and says so); **M-POPULATION-CHOICE** (the edge probe's
population is every movable cell exactly once, enumerated and never sampled);
**M-SORTS-NOT-SEPARATES** (the two-phase decomposition SORTS blocks at a stated threshold, so its
own unit test measures what it returns on a sample that has ONE phase — `0.01493 / 0.02566` with
a dense share of `0.477`, together and near a half — and the freeze reads the cluster arm against
that, never against the sorting alone). Not contacted: the rest of the registry.

## 0. What is built and measured

**The carrier** is `holon-lattice/src/edge.rs`, additive beside FLUID-1's `orientation.rs`, whose
rules are untouched (five of its private constants and its `unit` hash were made `pub` so this
carrier draws from the same streams; G1's bit-identity is the gate on that). Four rules are declared there and nowhere else, and each one's reason for conserving
is stated with it.

1. **The rest particle, and the collisions that make and unmake it.** The chart is `Model::fhp7`:
   FHP-6's six unit directions and a SEVENTH slot of zero velocity. A rest particle adds
   `(1, 0, 0)` to the conserved label, so on the hexagon — where `c_{d−1} + c_{d+1} = c_d` in the
   axial integers — the configurations `{rest, d}` and `{d−1, d+1}` carry the SAME `(N, P)` and a
   collision may exchange them. That is FHP-II's rest collision, and here it is not a typed table:
   it is one of the twelve dimension-2 fibers of the seven-slot census, which `state.rs` computes
   (80 sectors, `[(1, 52), (2, 12), (3, 14), (5, 2)]`, 76 movable states, pinned by unit test).
   The law is **E-I**: `{identity, fiber successor, fiber predecessor}`, one drawn per cell per
   step. Each table permutes within fibers, so each is a sector-preserving bijection and the draw
   is doubly stochastic on every fiber. On the two fibers FHP-I acts on, E-I's successor and
   predecessor ARE FHP-I's two chiralities — measured, not asserted. E-I is STRICTLY MORE
   COLLISIONAL than FHP-I: it also turns the seven other dimension-2 fibers of FHP-6, acting on
   all twenty movable six-direction states where FHP-I acts on five. **No transport coefficient of
   E-I has been measured and none of FLUID-0's census numbers transfer to it.**

2. **Two donor arms, and the WAIT.** A particle carries ONE orientation `o` and its arms point
   along `o` and `o + 2` — 120° apart on the hexagon, the lattice's nearest approach to the
   water's 104.5° and the Mercedes-Benz model's own angle (Ben-Naim 1971; Silverstein, Haymet &
   Dill 1998). Two arms and two acceptor roles put the each-bond-once ceiling at 2, which is the
   liquid's, and the degree ceiling at 4. A bonded closure's mover set is `{a, b, REST}`:
   FLUID-1's rule draws `m` from the two particles' own labels and displaces both by `dirs[m]`;
   what is new is the second candidate, `m = REST`, whose displacement is `dirs[rest] = [0,0]`, so
   both particles stay in their own cells keeping their own labels. **Why it conserves.** Momentum
   is `Σ dirs[label]` over occupied slots and a displacement never touches a label, so any
   displacement pattern whatever leaves both components integer-exact; mass needs only that the
   pattern be injective, and every displacement is a move within one plane into a slot CHECKED
   vacant with each particle displaced at most once. A rest state is how a particle can wait
   without carrying momentum, and the collision that puts one there is sector-preserving by
   construction — so the ledger never needs the wait's permission.

2b. **THE CLUSTER MOVES AS ONE** — the rule this freeze's carrier runs, selectable beside rule 2
   and never instead of it. **Rule 2 draws a mover per BOND, and EDGE-0's first gate measured what
   that costs**: of 6,281,540 bonds formed, 1,799 were released by the rent and 6,262,642 by the
   mover pass — and of those, **5,936,201 because a particle was ALREADY CLAIMED by another bond
   at a DIFFERENT mover**. With two arms the bond graph is dense, the bonds of one cluster fight
   each other, and the rule is anti-cohesive exactly where a dense phase would form. That is a
   finding about the pair rule and not about closure, so the closure grammar's own statement is
   the second rule:

   * **The mover is drawn per CONNECTED COMPONENT of the bond graph**, not per bond. The
     components are found each step with `holon_closure::ComponentFinder` — the same union–find
     `holon_closure::phase` reports the largest component with, kept in one place and fenced by
     that crate's own agreement test — over the PRE-shift slots; the draw picks one member
     uniformly by the counter hash and takes ITS label, keyed on the component's lowest pre-shift
     slot so it depends on no traversal order.
   * **Every member is displaced by the one drawn mover, keeping its own label.**
   * **The joint move of the whole component is admitted only if every target slot is vacant OR
     vacated by the same component.** A shift of a set onto itself is injective, and two members
     cannot share a target (that would need one cell and one label), so the whole map stays
     injective and mass is exact. Members are read out and cleared BEFORE any is written, because
     the source and target sets overlap.
   * **Otherwise the whole component WAITS at rest** — the same second candidate as rule 2 — **and
     is released only by the rent clause, per bond.**
   * **`blocked` counts the components refused BOTH**, with the prongs apart
     (`component_move_refused`, `component_wait_refused`), and releases that component's bonds so
     the ledger's balance still closes in bonds. Each particle is in exactly one component, so
     rule 2's per-bond claim table is not needed at all and its two prongs are left at ZERO rather
     than reused: "this bond had no vacancy" and "this bond's particle was claimed by another
     bond" are not sentences this rule can say.

   **What the rule does, measured by the gate at the chart before this freeze**
   (`conformance/mesh/edge0/gate.json`, `g0_totals`, 5,000 step-checks): 170,156 components walked,
   **163,811 moved as one (96.3 %)**, 6,345 refused the drawn move and 1,069 refused the wait as
   well; the largest component was 580 particles; 3,849 waits tried and 2,780 taken. And the
   headline for the tier: **210,951 bonds formed, 116,430 released BY THE RENT and 89,152 by the
   mover.** Under rule 2 that ratio was 1,799 to 6,262,642. The cluster rule hands the bond's
   lifetime back to the rent clause, which is what FLUID-1 said the carrier needed.

3. **Gravity as a momentum bias at collisions, POSTED.** After the collision and before formation,
   each cell is drawn against `rate`; where the draw fires and the slot along `g` is empty, a
   particle moving against `g` is turned to move along it (posting `2·dirs[g]`), or failing that a
   rest particle is set moving along `g` (posting `dirs[g]`). The two prongs are counted apart. The
   moved particle carries its orientation, its colour and every one of its roles, and every bond
   that touches it has its recorded slot rewritten. **The ledger is exact UP TO THE POSTED
   INJECTION and G0 says so**: `momentum(t) − Σ posted(t) = momentum(0)`, an integer identity at
   every step. Mass is untouched by gravity and is gated separately — one gate per conservation
   law, `lattice.rs`'s own rule for the wall.

**The bond rule and its rent are FLUID-1's, unchanged.** A bond forms on a link where an arm points
at an occupied neighbour with a free acceptor role, and breaks each step with
`p_break(φ) = exp(−A(φ)·E₀/kT_rent)`, where `A(φ)` is the amplitude table READ at run time from
CT-1's and CT-2's records at the six lattice angles. A bond pays no rent on the step it forms, so
the stationary held fraction is `1/(1 + p_break)`.

**Two temperatures, never conflated.** `E₀/kT_rent` is FLUID-1's DIMENSIONLESS chart parameter for
the bond's break probability and is not a mechanical temperature. The capillary spectrum needs a
mechanical `kT` and takes this carrier's OWN measured ideal-gas coefficient — the scalar pressure
over the moving mass density, FHP's `c_s²`, which reads `1/2` on the free gas by the hexagon's
`Σ_d c_dα c_dβ = 3 δ_αβ` and is MEASURED on this campaign's no-bond control. **The fence this puts
on the two σ readings**: `σ_L` comes out of Laplace's law and needs no `kT`; `σ_C` is
`kT_mech / (slope · L)` and any error in the `kT_mech` identification rescales `σ_C` and only
`σ_C`. Staking the two against each other stakes that identification too.

### The chart, and the rule that chose it — stated before the numbers

The rent is EDGE-0's OWN parameter, not a molecular reading. It was swept on a labelled screen
(`conformance/mesh/edge0/screen.json`, `dry: true`: 6 densities × 5 retentions × 3 seeds × BOTH
mover rules at `L = 64`, plus a one-phase control at every density, 60 chart rows and every one
ledger-exact). **The chart is the CLUSTER arm's swept point of largest block-density variance
against the one-phase control, among the points whose largest bonded component is UNDER HALF the
particles.** The second clause is not decoration: a point where the whole box is one component is
a GEL, and a gel has no droplet, so a chart there would be a chart for a different campaign. The
dip cannot choose the chart, and the freeze says why in §3.

| | pre-committed, by the rule above | from the screens |
|---|---|---|
| mover rule | **cluster (rule 2b)** | the pair arm's variance ratio never exceeded `2.489` at ANY of the 30 points it was swept on; the cluster arm reaches `13.378` at the selected chart |
| density `d` (per slot) | `0.02` | the selection rule's maximum. The rule's second clause bites: `(0.02, 0.999)` reads a higher ratio, `23.576`, but its largest component is `0.907` of the particles — a gel, refused |
| retention `f` | `0.90` (`E₀/kT_rent = 2.1972`) | at that point the largest component is `0.157` and the bond count `1.168` per particle, over rule 2's one-arm ceiling |
| dense / sparse densities the droplet and the slab are BUILT at | `0.05905` / `0.01038` | **MEASURED, not constructed**: the two-phase decomposition of the block-density sample at the chart, on the counted runs' own box, window and seeds (`screen_chart.json`, cluster arm). The dense phase holds `0.198` of the box |

**The confirmation pass** (`conformance/mesh/edge0/screen_chart.json`, `dry: true`) repeats the
chart at the counted runs' own `L = 128`, warm-up and window, on all three arms:

| arm | `q05 / q50 / q95` | spread `q95 − q05` | variance ratio | two modes | bonds/particle | largest component | two phases (dense share) |
|---|---|---|---|---|---|---|---|
| cluster | `0.00000 / 0.01116 / 0.06920` | `0.06920 ± 0.00182` | `12.337` | 4 of 4 seeds | `1.142` | `0.0495` | `0.01038 / 0.05905` (`0.198`) |
| pair (control) | `0.00670 / 0.01953 / 0.03627` | `0.02958 ± 0.00112` | `1.860` | 0 of 4 | `0.510` | `0.0034` | `0.01392 / 0.02865` (`0.414`) |
| one-phase (control) | `0.00949 / 0.02009 / 0.03181` | `0.02232 ± 0.00000` | — | 0 of 4 | `0` | `0.0004` | `0.01493 / 0.02566` (`0.477`) |

**Nothing either screen wrote enters a gate or a claim** (GANTT2's screen law); what they did was
choose the chart the freeze pre-commits and tell it which statistics discriminate.

**The instruments** are `holon-lattice/src/surface.rs`, six of them, each with a unit test on a
scene whose answer is known before the instrument sees it:

| | instrument | its known scene |
|---|---|---|
| (a) | the block-density histogram over blocks of `b = 8`: its two modes, its DIP, its variance, its quantiles and the two-phase decomposition | a planted two-mode histogram (dip exactly 1), a planted one mode (UNSEPARATED, dip 0), a uniform spread (dip exactly 0), a planted quantile ramp, and a planted two-phase sample whose two densities and dense fraction come back |
| (b) | the momentum flux tensor per cell, `Π_αβ = Σ Δx_α p_β` with `Δx` the particle's ACTUAL displacement, and Laplace's law `Δp = σ_L/R` as ONE slope against `1/R` | one particle in one cell reads `c ⊗ c` exactly; the free gas reads `c_s² = 1/2`; a planted `σ/R` line returns its slope and a flat jump does NOT fit one |
| (c) | the capillary spectrum of the flat interface, `⟨\|ĥ_k\|²⟩ = kT_mech/(σ_C L k²)`, fitted against `1/k²` over modes `2…16` | a height field built FROM a chosen `σ` returns that `σ` to nine digits |
| (d) | the interface width from the density profile's `tanh` fit, tracked frame by frame | a planted `tanh` returns its width and its centre |
| (e) | the block chart's edge, `holon_closure::Edge::at` on this carrier's own one-step split set | an empty lattice's edge is EMPTY and has NO width; `b = L` is labelled vacuous; a droplet in vacuum leaves the outside closed and its edge centroid sits on the droplet |
| (f) | the droplet's survival: the steps until one bonded droplet's largest component falls under HALF its start, sampled and capped | an empty droplet is EMPTY, a bond-free droplet is UNBOUND (start = 1), and on one scene the cluster arm's droplet survives its cap while the pair arm's dies at 75 steps |

**The runs.** At the chart, on `L = 128`, four seeds each: the coexistence run on the CLUSTER arm,
the same on the PAIR arm (the pre-committed control) and the one-phase control (bonds forbidden);
six droplet radii `{10, 14, 18, 22, 26, 30}` links × four seeds for Laplace's law; the flat slab
for the capillary spectrum and the width; the settling run with gravity and without; the free-gas
run that measures `kT_mech`; the droplet-survival run on both arms; and the edge at
`b ∈ {4, 8, 16}` on a droplet at `L = 48`. Every run warms up 3,000 steps with the per-step audit
off and then samples 200 frames at a stride of 5 with it on; the price is measured in that same
mix and written before anything is counted.

## 1. The expectation, written before any counted run (M-EMPTY-SECTOR's rule, met)

**The expectation is branch C (a): the cluster carrier has two phases at this chart.** The
statistics the two screens returned, stated here before the counted runs:

- **the block-density variance is 12.3 times the one-phase control's**, where the PAIR arm at the
  same chart reads `1.86` and never exceeded `2.489` anywhere on the whole swept chart;
- **the block-density spread clears the one-phase control's by 8.6 times the band** three standard
  deviations of the two arms' combined seed spread makes;
- **every seed's histogram carries two separated modes** where neither control carries any;
- **the two phases' own densities are `0.0104` and `0.0591`**, a factor of `5.7`, with the dense
  phase holding `0.198` of the box — against `0.414` and `0.477` on the two controls, which is
  what the decomposition's own unit test says a single phase looks like;
- **the bond graph is not a gel at this chart**: the largest component is `0.0495` of the
  particles, so there are droplets and not one spanning network;
- **and the droplet survives.** On the screen at the chart the cluster arm's droplet started at 76
  particles and both seeds kept half of it to the cap of 3,000 steps; the pair arm's started at 11
  and one of two seeds lost half in 125 steps.

The mechanism this rests on, stated so it can be wrong: **the cluster rule is all-or-nothing, and
that is the cohesion.** A component moves only if every target is vacant or vacated by itself, so a
component in free space moves and a component surrounded does not — which is a density-dependent
trapping, and a positive feedback: denser regions become more static and accrete. The rule's own
failure mode is the other side of the same coin and was measured on the screen: above `d ≈ 0.15`
the bond graph percolates, the giant component is refused BOTH candidates every step, and every
bond is released — the variance ratio falls to `1.0` and the bond count to `0.002` per particle.
The chart sits below that.

**If C reads (b) the branch is the EMPTY one and it is named**: a chart at which the histogram is
not two-phased is a reading about the chart, everything under it is VOID by that branch's own
letter, and D1's droplet lifetime is the number the campaign takes away — which is why D1 is NOT
void under C (b).

## 2. Gates

- **G0 — the ledger, exact up to the posted injection.** Mass, `Px − Σ posted`, `Py − Σ posted`,
  the tracer's red count, the orientation total AND its per-orientation census, and the bond
  balance `formed − broken_rent − blocked = held`, integer-identical at EVERY step, over 5
  configurations (the chart; the chart with gravity; the no-bond control; the cold limit; the wait
  switched off) × 2 seeds × 500 steps = 5,000 step-checks. EXACT as identities, never a tolerance.
  Beside them, and gating: bonds formed > 0, the wait fired > 0, rest particles created > 0,
  gravity posted > 0, components walked AND moved as one > 0, and a component larger than a pair
  built — a ledger gate that passed on a carrier that did none of those has not been shown able to
  fail.
  witness: `closed_iff_fiber_invariant` (node LG's; the conserved label is what the block charts
  below are charts of)
- **G1 — the no-bond, no-gravity, PAIR control IS FLUID-1's carrier.** With ONE donor arm, ONE
  acceptor role, no rest slot, the wait off, rule 2 and gravity off, this carrier and
  `orientation.rs`'s advance the same state: at `L = 32` over 300 steps of a BONDED run on
  FLUID-1's own seed, the occupation, the orientation plane, the colour plane and every bond's pair
  of slots are bit-identical, and the `formed / broken / blocked` counts agree exactly. Measured on
  a scene with bonds in it on at least 240 of the 300 steps, and the wait's count is exactly 0.
  witness: none (a bit-for-bit identity between two runtime objects; nothing mechanized in Lean)
- **G2 — each instrument on a scene whose answer is known.** Seventeen legs over the six
  instruments, the tolerances stated: the planted two-mode dip is exactly `1`, the planted one mode
  reads UNSEPARATED, the uniform spread is exactly `0`; one particle's flux is `c ⊗ c` to `1e-9`,
  the free gas's `c_s²` is `0.5` to `0.02` and its pressure isotropic to `0.05`, a planted Laplace
  line returns its slope to `1e-9` and a flat jump fits none; a planted capillary spectrum returns
  its `σ` to `1e-8` at `R² = 1` to `1e-8`; a planted `tanh` returns its width to `0.05` cells and
  its centre to `0.1`; an empty lattice's edge is EMPTY with `NaN` width, the `b = L` chart is
  labelled vacuous, and a droplet in vacuum reads an edge fraction under `0.7` whose centroid is
  within 6 cells of the droplet.
  witness: none (planted scenes and their arithmetic; nothing mechanized in Lean)
- **P — the price.** Measured on the first 200 steps in the SAME audited mix the counted runs
  advance in (150 unaudited + 50 audited), written before the counted steps, and every counted
  phase within `0.1×`–`10×` of its own projection. The gate's own reading, before this freeze:
  `5.153599e-8` s per cell per step against FLUID-1's measured `1.261504e-7` — an overhead of
  `0.409×`, this carrier being CHEAPER because the chart is dilute — and the cluster rule's step
  costs `1.325×` the pair rule's on the same scene, which is what component finding costs.
  227,700 counted steps project to 192 s.
  witness: none (a price, recorded)

## 3. The branches, and what each outcome means

- **C — coexistence.** At the chart, the block-density histogram over 4 seeds × 200 frames, read
  against BOTH controls: **(a)** every seed's histogram has two separated modes; AND the variance
  ratio against the one-phase control is over three times the PAIR arm's at the same chart; AND
  the block-density spread clears the one-phase control's by three standard deviations of the two
  arms' COMBINED measured seed spread. Then the carrier has two phases and the two-phase
  decomposition's densities are theirs. **(b)** it does not — the EMPTY branch, named. **The gate
  VOIDS rather than passes if that combined band is measured exactly zero**, which is not
  hypothetical: the one-phase control's own seed spread came back exactly `0.00000` on the
  confirmation pass, and a leg divided by it would have turned a real gap into a number with no
  meaning.

  **THE DIP IS REPORTED AND NOT GATED, and the reason is an instrument correction the screens
  earned.** At a chart whose dense phase is a minority of the blocks, the valley between the two
  modes is as full as the smaller of them, so the dip reads exactly zero — the confirmation pass
  measured `0.0000` on ALL THREE arms alike, including the one with two phases in it. A statistic
  that cannot tell the cluster arm from the one-phase control at this chart cannot decide this
  branch. It stays in the record beside the three legs that can.

- **D1 — the droplet's lifetime.** One bonded droplet of radius 12 in a background at the sparse
  phase's own density, per arm, per seed: **(a)** every seed keeps half its starting cluster to the
  cap of 6,000 steps; **(b)** it loses half, and the lifetime is the number; **(c)** there was no
  droplet to lose — EMPTY, named. **NOT void under C (b)**: a bonded droplet either keeps half its
  cluster or it does not, and where coexistence is absent that lifetime is what the campaign reads
  instead of a bare empty.
- **L — Laplace's law.** Over the 6 radii: **(a)** the pressure jump is linear in `1/R` with ONE
  positive slope resolved against its own spread over seeds, and that slope is `σ_L`; **(b)** it is
  not a line, and no surface tension is read from it. VOID under C (b).
- **A — the two σ agree.** **(a)** `|σ_L − σ_C| ≤ 2·sqrt(sd_L² + sd_C²)`, the band computed from
  the two readings' OWN measured spread over seeds and typed nowhere; **(b)** they disagree beyond
  it. A disagreement is a statement about the pair, not about either alone: `σ_C` carries the
  `kT_mech` identification and `σ_L` does not. VOID under C (b).
- **W — the interface width.** **(a)** the fitted width is stationary — `|dw/dt| < 0.05` cells per
  thousand steps over the sampled frames, and the drift is not resolved against its own spread over
  seeds; **(b)** it grows, which is the freeze's kill. VOID under C (b).
- **G1B — gravity settles the dense phase.** **(a)** with gravity on, the dense phase's centre of
  mass drifts along gravity and the control with gravity off does not, per seed; **(b)** it does
  not move, or the control moves too. NOT void under C (b): a built slab settles or does not, and
  the posted-injection ledger is exact either way.

C (a) with L (a), A (a) and W (a) is a fluid element with a defined edge, a measured surface
tension agreed by two instruments, and a stable interface — and EDGE-1 is framed on it. C (a) with
L (b) or A (b) says the carrier has two phases whose boundary is not a Laplace surface, and names
which instrument disagrees. C (b) is the finding that neither cohesion — the pair's nor the
cluster's — makes an edge on this carrier, and D1's lifetime is what the campaign reads; the named
next thing to price is then Appert & Zaleski's non-local momentum exchange, which is an
attraction rather than a constraint on displacement. Either way the carrier's four rules and the
six instruments stand, gated on their own scenes, and are what EDGE-1 and EDGE-2 inherit.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

One collision law, one box, one block size for the histogram, one interface orientation, four
seeds; a two-dimensional lattice standing in for a three-dimensional liquid, with six orientations
for a continuum of them and one orientation per particle for a molecule with two independent arms.
E-I's transport coefficients are unmeasured, so nothing here can be compared with FLUID-0's
census, and the cluster rule's own transport is unmeasured too — this freeze reads a surface, not a
Schmidt number. The mechanical `kT` is an identification, not a measurement of a temperature this
carrier has. The cluster rule is a rule about a lattice gas's closures and is not claimed to be
any molecular dynamics. Nothing here is a claim about water.

## 5. Plants

Both carriers are asserted nonzero in the sector the plant acts on, and both stakes are DERIVED
from the analytic reach of the observable the plant acts on, computed before any step runs. That is
FLUID-1's own correction, entered here as a rule: a plant's stake must be derived from the reach of
the observable it acts on, and FLUID-1's two were typed at 0.20 against reaches of 0.085 and 0.038.

- **(i) The rent doubled.** Sector: the bond's break probability, whose stationary held fraction on
  a held geometry is the observable. Carrier: the held fraction at the chart's rent, `0.900`,
  asserted nonzero (over a floor of `0.01`) in that sector. Analytic reach, exact before any run:
  `|f(E) − f(2E)| / f(E) = 0.0976` with `f(E) = 1/(1 + e^{−E})`. Stake: half of that reach,
  `0.0488`, so the plant is reachable by the freeze's own arithmetic by construction.
- **(ii) Gravity's rate doubled.** Sector: the per-cell gravity draw, whose posted momentum
  injection on the FIRST step from a common initial state is the observable. Carrier: the rate,
  `0.02`, asserted nonzero (over a floor of `0.001`) in that sector. Analytic reach: exactly `1.0`
  in relative terms — a cell fires when its own uniform draw is under the rate, so the cells firing
  at `r` are a subset of those firing at `2r` and the expected posted injection at a FIXED state is
  exactly proportional to the rate. Stake: `0.5`.

## 6. Discipline

Instrument: `holon-lattice/src/edge.rs` (the carrier: the rest slot, the two arms, the wait, the
cluster rule, gravity posted) and `holon-lattice/src/surface.rs` (the six instruments), with
`holon-closure`'s `ComponentFinder` carrying the one union–find both the phase and the cluster
pass use. Unit tests: the FLUID-1 bit-identity; the ledger exact under bonds, waits, rest particles
and gravity together, under both mover rules; the wait exhibited on a hand-built scene where
FLUID-1's rule releases the bond and this one holds it; a three-particle chain with three different
labels moving as ONE under rule 2b where rule 2 cannot hold it; the cluster rule's all-or-nothing
density dependence measured at both ends; the rent clause reproduced on a held geometry; and one
known scene per instrument. Runner `holon-lattice/examples/edge0_lattice.rs` on `holon-campaign`:
`screen` and `screen_chart` (both dry, the chart), `gate` (G0–G2, the plants, the price), `run`
(the counted runs → `run.json`, `run.done`), `read` (the six branches → `read.json`). Records under
`conformance/mesh/edge0/`; results `EDGE0_RESULTS.md`. Cores 24–31 only. The only numbers from
outside this engine are the amplitude table and the retention READ from CT-1's and CT-2's records
and FLUID-1's own measured cost per step, each printed with its path and its field.
