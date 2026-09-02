# FSD-W1 — THE WATER WORKBENCH
## Functional specification, locked 2026-08-31 at the operator's direction

*One recursive holon, observed at any scale from the vacuum to a kilometer of
water, with four live controls, one reset control, and a hand that grabs
scene-scaled volumes. This document is the destination the campaign records
feed; every requirement carries an ID so gates can cite it. The physics
authority remains the maximal tasking (OBSERVATORY_BRIEF v2): no external
potential, no fitted parameter, phases as certified Closed views. Where this
FSD says "measured," the number is staked-until-benchmarked and the first
real benchmark replaces it, pinned and core-class-declared.*

---

## 1. The scale axis (WB-1)

**WB-1.1** The viewport spans the vacuum to 1 km × 1 km, continuously. Zoom
is a TIER SELECTOR: the scale determines which certified chart of the holon
runs in the viewport.

| Scale band | Active chart | Content |
|---|---|---|
| Å – ~4 nm | atomistic quantum-emergent (tables + on-demand solves) | atoms, bonds forming/breaking, reactions live |
| ~4 nm – ~1 µm | promoted molecular (H₂O quotients on derived charts) | H-bond networks, interfaces, nucleation |
| ~1 µm – 1 m | continuum (T6 fields: density, stress, polarisation) | flow, droplets, ice fronts |
| 1 m – 1 km | bulk continuum + hydrostatics | waves, columns, weather-scale slabs |

**WB-1.2 — recursion on demand, never by default.** Each band runs its own
chart; the tier below is entered only where the chart's closure budget is
exceeded locally (an interface, a rupture, a perturbation) — a refinement
patch opens, computes, and closes. Nobody simulates 10²⁰ atoms; the
architecture exists so nobody has to.

**WB-1.3 — the zoom–timescale law ("it slows down as you zoom in").** Each
tier has a measured sim-rate budget at the target framerate; deeper charts
buy fidelity with simulated time. Representative staked rates on the 4090
at 30 fps: atomistic ~10–100 ps/s (1k–5k atoms), promoted ~50–200 ps/s
(10k–50k atoms), continuum up to realtime. Zooming hands the viewport to
the coarser or finer chart and the sim-rate follows automatically.

**WB-1.4 — display honesty.** The sim-rate readout shows real units (ps/s,
ns/s, ×realtime) with the %-realtime figure beside it. A label that says
"0.001% realtime" while running picoseconds per second is a lie and is
refused by this spec.

## 2. The live controls (WB-2) — present at every scale

**WB-2.1 TEMPERATURE.** 0 K – 3000 K, displayed °C/°F (unit toggle).
Thermostat target at atomistic/molecular tiers; boundary/field condition at
continuum tiers.

**WB-2.2 PRESSURE — and pressure changes the physical scale.** The pressure
control IS the barostat: at molecular tiers the box rescales under the set
pressure (NPT), at continuum tiers it enters the equation of state. The ice
polymorph ladder lives on this knob.

**WB-2.3 TIMESCALE.** The governor, not a setting: the free variable
auto-tuned to hold framerate, user-biasable within the tier's honest range,
calibrated in the running environment (M-IDLE-CALIBRATED-TIMEOUT applies —
never from an idle-machine number).

**WB-2.4 GRAVITY.** 1 G, downward toward the lower face, at every scale.
Exactly representable, costs nothing — and it is the workbench's cleanest
tier-separation exhibit. **LANDED 2026-09-01** (`Sim::set_gravity`,
`tests/gravity.rs`, six gates, two plants firing): a uniform field in the
same external-acceleration array the walls and the hand use, so the momentum
ledger books its impulse with no new accounting; its energy is a potential
term `Σ mᵢ g yᵢ` zeroed at the box's lower face, and it posts NOTHING to the
W_ext receipt columns, because a conservative field's energy is the potential
and a receipt would count the same joules twice.

**WB-2.4a — the exhibit, MEASURED, and this section's own figure was wrong
twice.** Both corrections come from `tests/gravity.rs`, which computes the
numbers rather than quoting them:

- The 1 nm figure staked above as `~10⁻¹³ of kT` **measures 4.05×10⁻¹⁵** — about
  25× smaller. The claim's substance survives (gravity is invisible there, and
  more so than advertised) but the exponent was wrong by more than a rounding.
- **"Sovereign at 1 km" is not a per-particle statement, and reading it as one
  is wrong by five orders of magnitude.** A hydrogen atom raised a full
  kilometre gains 0.004 kT — still invisible. The per-atom crossover is the
  SCALE HEIGHT `kT/(m g)`, measured at **246.6 km**, which is the textbook value
  for an isothermal hydrogen atmosphere at room temperature and is therefore an
  independent check on the whole unit chain rather than merely a number.
  Gravity's sovereignty at the top is **COLLECTIVE**: ~9.8 MPa at 1 km, about 97
  atmospheres, summed over ~10²⁸ particles.

So the corrected statement of the exhibit, which is stronger than the one it
replaces: **what changes across the tiers is not the field's strength but
whether the quantity that matters is a per-particle energy or a sum over the
column.** The field is the same everywhere; the arithmetic that reads it is not.
That is a cleaner statement of tier separation than "small here, big there",
and it is the one the workbench page now makes.

**WB-2.4b — one boundary REFUSES the field, so "at every scale" has an
exception and it is recorded rather than glossed.** A PERIODIC box has no
bottom: `m g y` is linear, the wrap makes it discontinuous, and an atom leaving
the top face re-enters at the bottom with its potential changed by `m g H` and
nothing having done that work — the balance gate would open by exactly that jump
on every crossing and the result would be reported as integration drift.
`Sim::set_gravity` returns `GravityRefusal::PeriodicBox` there rather than
serving it. Conservation is chart-relative and this chart has no bottom to fall
toward. Reachability, stated because an instrument that cannot fire is worse
than none: `holon_set_boundary` exposes only Walls and Open, so this refusal is
reachable today only from a native caller, and a browser shell must not
advertise it as a live fence.

## 3. The reset control (WB-3)

**WB-3.1 MIXTURE resets the scene**: pure O · O:2H · pure H (and arbitrary
ratios between). Pure H is banked physics; O:2H is the water-formation
experiment itself; pure O rides the genuine (O,O,O) surface. Changing
mixture is a scene reset, never a live mutation — composition is identity,
not a slider.

**WB-3.2 PRE-WARM.** Default scene: room temperature (293 K), 1 atm,
loaded from SHIPPED, CERTIFIED reference states (the campaign's WP-5
checkpoints) per (scale band, mixture) — the user is never forced through
an equilibration cycle to see room-temperature water behave like water.

**WB-3.3** Off-default settings load the nearest reference and settle; the
settling state is INDICATED, never silent — an unequilibrated scene
presenting as equilibrium is the vacuous-success shape and is refused.

**WB-3.4** Every reference state carries its manifest: producing commit,
backend, seed, thermodynamic state, certification gates passed. The
provenance panel can show it at any time.

## 4. The hand (WB-4) — perturbation scaled to the scene

**WB-4.1** The mouse grabs a volume proportional to the viewport: a few
molecules at 4 nm; a droplet at 1 µm; **swimming pools' worth at 1 km** —
grab radius a fixed fraction (staked: ~5%) of the viewport edge, always
displayed in physical units.

**WB-4.2** A grabbed parcel is a moving boundary condition dragged through
the scene; release throws it with the hand's velocity.

**WB-4.3 — the hand is ledgered.** User forcing injects energy and
momentum; the conservation gates do not go blind to it — the hand's work is
a receipt in the energy ledger (an external-force term with its own
column), so the balance gate still closes on every frame. Conservation is
chart-relative and the hand is part of the chart.

**WB-4.4** A perturbation that locally exceeds the active chart's closure
budget opens the refinement patch (WB-1.2): throw a pool hard enough and
the splash zone recurses; the recursion cost is paid from the timescale
governor, visibly.

## 5. Integrity invariants (WB-5) — inherited, not optional

**WB-5.1** No external potential, force field, or phase-specific parameter
anywhere in any tier's physics (the maximal ruling). A published model may
be cited in reporting; it never executes.

**WB-5.2** M-CHEAPER-THAN-ITS-PRICE is a runtime law: any chart or surface
arriving below its certified compute price refuses; any untabulated
encounter FENCES VISIBLY (rendered as such) or recurses to a genuine
solve — it is never faked, interpolated across, or silently zeroed.

**WB-5.3** The closure-defect lens is available at every scale: the user
can see, live, how well the current chart commutes with the dynamics —
the holon claim as an on-screen number.

**WB-5.4** Determinism: seeded scenes replay bit-identically per device
class (D0: the class is part of the artifact); checkpoints are exact.

**WB-5.5** The blind classifier runs at molecular tiers and its phase call
is displayed beside — never derived from — the preset that launched the
scene (plant P-1 stands in CI forever).

## 6. Performance contract (WB-6)

**WB-6.1** Target: 30 fps on the RTX 4090, device class declared. The
staked capacity table (§1.3) is replaced by measured benchmarks, pinned,
both core types declared for any CPU arm (M-PLACEMENT-LOTTERY).

**WB-6.2** Framerate is held by the timescale governor (WB-2.3), never by
physics degradation: the chart in the viewport is always the certified one
or a visible fence — there is no "reduced accuracy mode," only honest
slower time.

## 7. Build dependencies (what feeds this FSD)

Ozone tabulation + certification (in flight) → frozen P2 rerun (water
formation verdict) → MBE4 in the trajectory loop → T3 scale-up (PBC, cell
lists, checkpointing) → T4 ring-polymer quantum nuclei → promoted-molecule
charts + derived V(H₂O–H₂O) → T5 phase certification (reference states =
WB-3.2's inventory) → T6 continuum charts (WB-1's upper bands). Each
dependency lands through its own campaign gates; this FSD is their
integration target and adds no physics of its own.

## 8. The mock law (WB-7) — added 2026-08-31 after review of the first shell

**WB-7.1** Any interface artifact that does not execute certified physics
MUST self-declare MOCK: on screen, in its manifest, and in its source
header. A displayed quantity either traces to a computed value or is
labeled SYNTHETIC beside its digits. A synthesized conservation ledger
(`sin(performance.now())` presenting as ΔE) is the vacuous-success shape
in a costume and is the specific incident this law is written from.

**WB-7.2** Commits 84759ca and 2d0fc5e are so marked: a valuable
interaction/ergonomics prototype (orbit camera, tier dock, touch targets,
telemetry drawer — the vocabulary survives) whose physics is placeholder
throughout: 2D canvas presenting as WebGL, `Math.random()` initial states
(breaks WB-5.4), hardcoded 104.5°/0.096 nm insertion (molecules are
discovered, not defined — §5 of the tasking), a 450.0 harmonic + 0.015 LJ
(fitted potentials, banned from ENGINE physics by WB-5.1; tolerated in a
declared mock's display layer only, and they must never migrate), P^−0.05
box scaling standing in for NPT, refinement as a banner with no solve,
and a manifest claiming WebGL2/T6 certification the code does not carry.
The real shell replaces these with the Rust/wasm engine per §6–7.

**WB-7.3** The naming and identity note: "Molecular Workbench" is
established prior branding (Concord). The scientific identity of this
instrument is **closure-certified recursive simulation** — coarse charts
DERIVED from the same lower dynamics, admitted by measured closure,
refined locally on budget failure, with ledgered handovers — which is the
part the prior-art register (AdResS adaptive resolution, Narupa
interactive MD, MB-pol/Deep-Potential water, Concord's workbench) does
not contain. The name may say water; the claim says closure.

**WB-7.4 — the acceptance demo (first-publication milestone), staked:**
(1) H₂O emerges from unrestricted H/O dynamics; (2) promotion to the
molecular quotient; (3) a DERIVED water–water interaction; (4) a
perturbation drives the closure defect past its budget; (5) automatic
local atomistic refinement; (6) the seam crossed with the energy ledger
still closed; (7) a held-out observable predicted after the handoff.
Seven steps, no kilometer tier required. When this loop runs live, the
instrument is the novelty; until then every shell is WB-7.1 MOCK.

## 9. THE CARRIER TOWER (WB-8) — prerequisite for full quantum effects
### Added 2026-08-31 at the operator's direction. This section IS the sprint tasking.

**WB-8.0 — status: PREREQUISITE.** Full quantum effects in the workbench
(T4 nuclei and everything past them) build on the carrier tower, not
around it. Land the interfaces before the physics that needs them.

**WB-8.1 — the fold, stated once.** The holon lives in a two-dimensional
system of certified charts: vertically, quotient by scale (electrons →
atoms → molecules → phases → continuum) — this axis is the existing
Object contract, `Closed v T`, unchanged; horizontally, refinement of the
theory carrier (Born–Oppenheimer classical-nuclear → ring-polymer quantum
nuclei → real-time MPS electronic dynamics → spinorial/Dirac → QED) — the
SAME commuting square rotated: a `CertifiedTransport` is lift-state +
picture-change + commuting certificate. Terms ADD only inside one
carrier's fiber; across carriers you TRANSPORT, never add. The pattern's
existence proof already runs in this codebase: the Scalar seam (one
solver body, f64/Dd carriers, promotion as explicit transport, mixing a
type error).

**WB-8.2 — the interfaces to land** (architecture per the reviewed
sketch; names negotiable, semantics not):
- `Carrier` (State / Operator: AdditiveOperator / Observable) and
  `Contribution<C: Carrier>` — cross-carrier addition is a COMPILE ERROR;
- `CertifiedTransport<A, B>` with its commuting certificate — the
  missing-picture-change refusal test ships with it;
- `Capability<T>::{Certified, Stub}` — every stub carries a visible fence
  (the existing refusal pattern, typed);
- `TheoryNode`/`TheoryDiagram` with dependency-closed term sets, per-node
  error budget and MEASURED price;
- selection = the corridor rule, proved in `lean/CIRISHolon/Carrier.lean` §5:
  argmin price subject to closure + conservation budgets — cheapness
  alone selects the dead chart and is refused by theorem;
- `AngularShell { l: u8 }` — kill the S/P/D/F enum; ℓ is a value.
  Z prices; Z never branches.
- Refusal test battery: double counting (type-level), missing picture
  change, budget-violating selection, stub-without-fence. Each
  demonstrated firing (a gate that has never failed has never gated).

**WB-8.3 — the water tower's carriers**, in build order: C0 the resident
node (nonrelativistic BO, classical nuclei — today's banked physics,
untouched, re-expressed as a node); C1 ring-polymer quantum nuclei (T4 —
ZPE, tunnelling, isotopes; classical limit = C0 as the diagonal retract);
C2 real-time MPS electronic dynamics (TDVP on the crystal-banked
machinery — the 0:1:0:1 alternation entering chemistry as dynamics);
C3+ spinorial/QED nodes exist as visible STUBS with fences — reachable,
not materialized; not on the water path and honestly so.

**WB-8.4 — THE ACCEPTANCE LAW: the most exotic dynamics, no
undischargeable refusals — or the DRY is wrong.** The fold's falsifier,
staked now: within the water domain, EVERY refusal must be dischargeable
by climbing the tower — a fence is transient (pay the price, transport to
the adequate carrier, the fence lifts); a refusal that NO reachable
carrier can discharge falsifies the fold's design, and that verdict is
recorded, not argued with. Honesty unchanged: fences stay visible while
undischarged (WB-5.2); "no refusals" means no PERMANENT ones, never
fake-served ones — the three fabrication convictions of this campaign
define exactly what this clause does not license.

**WB-8.5 — the exotic showcase, staked as targets** (each lands with its
carrier and its own gates): proton tunnelling and H/D isotope
fractionation (C1); Grotthuss proton hopping — autoionization, H₃O⁺/OH⁻
wires (C0/C1 reactive dynamics + quantum protons); ice X's symmetric
hydrogen bond (C1 essential — classical nuclei cannot produce it);
SUPERIONIC ICE — oxygen lattice, liquid protons, the flagship exotic
(C1 + the pressure knob); supercritical water (C0/C1); thermal seam
crossings at the tabulated state crossings (C2); coherent proton dynamics
(C2, the deepest rung). If the tower is right, this list is a tour;
each entry that instead dead-ends in an undischargeable refusal is a
WB-8.4 conviction.

**WB-8.6 — division.** Sprint team: WB-8.2 interfaces + refusal battery +
C0 re-expression + C1 construction (the ring-polymer holon per the
maximal tasking §4, with its exact-reference gates). Crystal-inheritance
measurement (DMRG-vs-FCI staking) feeds C2. Ozone tabulation,
certification, and the frozen P2 rerun continue unblocked in parallel —
the fold does not gate them; they do not gate the fold.

## 9b. FSD-W2 — 2026-09-01: the machinery is fixed, the bench goes full

*Written after the exact four-body landing (21e6be3), the certified water
(census_mixed_fenced.log:250), and the full-strength reproduction
(conformance/atomworld/p2_de4_full/). This section records what is BUILT
against what §1–7 asked for, and stakes the page update.*

### Built and verified (engine + ABI, at HEAD)

| capability | state |
|---|---|
| exact four-body (O,H,H,H) | ohhh_fci_grad: nine seeded dual solves, exact Cartesian gradient, momentum zero by construction; gate battery 4/4 (momentum, control, torque, force-is-the-gradient); native only — no ABI door yet (see chain below) |
| gravity (WB-2.4) | LANDED and LIVE on the page; measured 4.05e-15 of kT at 1 nm; collective sovereignty; refuses periodic by name (WB-2.4a/b above) |
| the hand on the box (WB-2.2) | **LANDED this pass: the control IS the box.** `holon_box_scale(f)` scales container and contents affinely, posts the move's cost to BOTH ledger columns, refuses bad factors and collapse by name; `holon_pressure`/`holon_pressure_defined` are the READOUT (virial; defined on periodic, and boundary mode 2 now reaches it). Gates: tests/scale_box.rs 4/4 + smoke block 5c. No setpoint door ships — pressure is read, never chased |
| scene scale (WB-1) | the sixteen-atom cap is GONE (T3): holon_reset(n) arbitrary, cutoff-local loops, cell lists, PBC with a wrap that does no work by theorem, calibration governor unchanged |
| closure census | IN THE ABI already: holon_census_*, per-row closure defects, formations/dissolutions/rejections — the page can show CERTIFIED THINGS live, which is the whole ontology on screen |
| trimer surface door | render-3d's SurfaceGrid + explicit-coordinate push door (begin/axis/energy/digest/finish), species-tagged, fence-counted |
| quantum nuclei (C1) / real-time (C2) | engine carriers landed with their gate batteries; not workbench-facing this pass |

### The page update (WB-9), staked

1. **3D ONLY.** The 2D canvas shell retires from the waterbench page; the
   Bevy shell (same rlib, 10/10 headless conservation gates, dual
   webgpu/webgl2) is the only rendering. No 2D fallback — a fence with the
   reason, not a degraded mode.
2. **WB-2.4c — GRAVITY LIVES IN THE WORLD, NOT THE BOX.** The engine grows a
   gravity DIRECTION: `holon_set_gravity_vec(gx, gy, gz)` (uniform field,
   same ledger discipline — V_g from the vector, conservative, posts
   nothing to W_ext; the periodic refusal applies per-axis where the field
   has a component through a wrapping face). The shell keeps the vector
   pointing WORLD-down while the user rotates the BOX, so tilting the box
   sloshes the water — the rotation changes the field's direction in box
   coordinates, which is exactly what a tilted bucket is.
3. **TEMPERATURE GLOW.** A faint background glow keyed to the measured
   kinetic temperature (blue cold through red hot), driven by
   holon_temperature() — a READOUT, never a control; the thermostat panel
   stays the control.
4. **PRESSURE PANEL** wired to the landed door: drag compresses/expands the
   box (holon_box_scale), the readout is holon_pressure with its
   defined-flag honored (under walls the panel says why the number is not a
   pressure), and the hand column shows what the compression cost.
5. **CENSUS PANEL**: live molecule rows with closure defects — formations,
   dissolutions, rejections — the certified-thing view.
6. **THE WATER STORY** on the page: the certified molecule and the
   full-strength reproduction, at the strength the record carries
   (CERTIFIED-STRICT 893.8 fs vs the 834 fs staked window; the causal
   reading provisional pending the same-commit control, and the page says
   so exactly as the README does).

### 9c. THE SITE — operator's order, 2026-09-01

**The workbench REPLACES the current .io UI entirely.** Not a page beside the
site; the site. The old UI retires when the 3D workbench is green under its
full gate battery — no gap where neither serves, same law as the 2D canvas
retirement.

**THE HERO DEMO: the 1 km × 1 km water cube — the scale ladder made visible.**
A kilometre of water is ~3e31 molecules; nobody simulates that atomistically
and the page never pretends to. The hero is the ZOOM AXIS itself (WB-1's law):
at the bottom, the atomistic cube LIVE — the engine's own certified molecules,
the tier that passed the closure test; each step outward hands off to the next
COARSER CLOSED VIEW, and the handoff is the exhibit — the commuting square on
screen. The honest ladder at launch:

| zoom band | what runs | status shown on screen |
|---|---|---|
| molecular (~nm) | the live engine, full physics ladder | CERTIFIED (the census's own verdict, cited) |
| H-bond network (~10 nm) | GATED — GANTT node G, rung 1 | its fence, with owner and exit, per the fence law |
| fluid element (~µm+) | GATED — node G, rung 2; the grain law banked | fence, owner, exit |
| the cube (1 km) | the continuum face of the ladder | "this face becomes live as each rung beneath it certifies" |

**The one law that makes this hero possible now:** no tier ever fakes. A zoom
band either runs its certified physics or wears its fence — and a fence is a
BUG UNDER REPAIR, never content (operator's law, 2026-09-02: any "content"
saying "we refuse, and the honesty is the point" is not content, it is a
bug). A displayed fence states its debt, its owner, and the build paying it —
present tense — and the site's story is the ladder CLIMBING: bands going
live, fences being deleted. WB-7 (no synthetic telemetry) applies at every
altitude; the gravity-tier exhibit (per-particle vs column-sum) already shows
how a scale-dependent truth is displayed without faking dynamics.

**THE ZOOM DE-ALLOCATES — NO TRANSITION MACHINERY. The math is already
banked (operator's correction; prior art in this repo):**

* `lean/CIRISHolon/Grain.lean` + `engine/crates/holon/src/grain.rs` — a
  Grain is a MEASURED closure schedule: it names the steps where a coarse
  view is EXACT (zero defect), so refreshing a coarse representation on
  closure boundaries is FREE, with a stated bound between them. Machine-
  checked kernel; constants pinned in CI.
* `engine/crates/holon-render/src/holon.rs` — composites are first-class
  rows with a MEASURED per-row closure defect at every grain boundary
  ("the composite view asserts the members are autonomous; the defect is
  how much that assertion missed by"). Formation/dissolution is
  ACCOUNTING-ONLY: E_before == E_after bit-identically, the row's ledger a
  VIEW of energy the global ledger already holds.
* The census is the same instrument at verdict strength.

So the zoom needs nothing new: **a holon that is not load-bearing for the
scene DE-ALLOCATES** — its members' fine DOF release and the freed budget
goes to the visible region — and "load-bearing" is not a heuristic, it is
the MEASURED closure defect: a row whose defect reads ~0 is autonomous by
measurement and its composite carries it exactly on grain boundaries; a
row being buffeted, grabbed, or coupled into the visible region scores
badly and KEEPS its fine allocation. Zoom in far enough and most of the
scene is invisible and non-load-bearing: those rows run as their
composites (which conserve by construction), and re-allocation on zoom-out
is the same accounting-only event in reverse. No seam machinery, no
handoff protocol, no resampling story: one layer, one defect number, the
grain law saying when coarse is free.

**THE ACUITY LAW (operator's design): the observer's resolution bounds the
allocation, and the seed is ONE pinned holon.** The arithmetic that makes
the hero cheap: the cube is 1 km and a water molecule is ~3 Å — a linear
ratio of ~3e12 — so when the view has zoomed to a
band's OWN scale, the in-view population at that tier is ONE (the earlier
parenthetical — first resolvability at a pixel — names a moment nine orders
wider, 3.0e9 in view; the page measured both readings and ships the one
that reproduces this paragraph's own figures, per the ladder gate). Even at full molecular zoom, the focal view admits thousands of
molecules, never 3e31: acuity itself is the allocator. Therefore the
zoom-in rule is: when the next tier starts to matter, PIN ONE HOLON of
that tier near the view center and populate only what acuity admits as the
zoom deepens — by the time finer structure is visible at all, the number
of other holons at that scale in view is negligible, and everything
outside the view stays coarse under the de-allocation law above. No
representative slabs, no bulk fine-simulation: one seed, acuity-bounded
growth, measured closure defects deciding what stays fine.

**THE TWO-BOX LAW (operator's design, 2026-09-02): four WORLD boxes, one
SCENE box, and they are never the same knob.** Each ladder band has its own
WORLD box — the physics domain. Pressure (the virial), temperature, and the
census's phase fractions are WHOLE-ONLY observables and are computed on the
world box, never on a fraction of it; the HAND acts on the world box, which
is the pressure control ("we change the size of the box to change
pressure"). The SCENE box is the single view volume, always the same size
as the active world box or smaller, and it scales with the ZOOM: zooming in
shrinks the scene box around the view center, and holons falling outside it
are REMOVED from the scene across all six faces — released at the boundary,
their allocation freed for the finer population the acuity law admits —
while the world box keeps the physics whole. Drawing is the scaled fraction
inside the scene box. ZOOM IS A RATIO, NOT A LENGTH: the scene box is the
world box divided by the zoom factor, so 3x zoom on a 1 km world and 3x
zoom on a 0.5 km world are DIFFERENT scene-box sizes. Zoom never touches
the physics; the hand touches BOTH — stretching the world box at fixed zoom
stretches the scene box with it, because the view is coupled to the world
through the ratio and only through the ratio. When the scene box shrinks to
where the next tier matters, the next band's world box seeds at the view
center (one holon, per the acuity law above), and zoom-out is the same
event in reverse. Every removal and re-admission is a ledgered scene event,
never a silent edit.

**THE BAND-FLIP LAW, restated because it was briefly blurred:** a band goes
live ONLY on a node-G closure certificate — a coarse view of the dynamics
beneath it, certified by the census. A tier certified on its OWN dynamics
(node LG's lattice gas) is supporting machinery and research content; its
certificate does NOT flip a band, because running physics that is not the
certified coarse truth of THIS water would be exactly the fake §9c bans.

**THE HERO'S FIRST INTERACTION: stretch the cube.** The box-scale door at
hero scale — drag the cube larger, watch density fall, pressure fall (the
virial readout), and the census's phase fractions move as water leaves the
liquid for vapor, LIVE, with every number a measured readout and none of it
a staked claim (the phase-DIAGRAM claims belong to TRIPLE_POINT_PREREG's
campaign; the page shows the census's live reading and cites the campaign
for the map). The demo axis and the campaign's f-axis are the same door,
which is the point.

Deployment: pages.yml serves the workbench at root; the committed cdylib stays
the gated artifact; the Bevy build stays CI-built with its sha in the page
manifest (item-1 ruling unchanged).

### The dE4-in-browser chain, staked with owners (NOT this pass)

evaluator for pushed (x,y,u) surfaces (mesh, next increment) → the (O,H,H)
and (O,O,H) tables served through the SurfaceGrid door (page pushes the
committed artifacts, 207,025 + 9,075 nodes) → `holon_set_de4` +
`holon_de4_evals` (the functional counter, per the census lane's
symbol-absence lesson) → four-body water in the browser. Each arrow is a
gate, not a hope.

## 10. Status ledger & next steps — 2026-08-31, updated at the operator's reorder

**LANDED:** carrier tower skeleton (WB-8.2) merged at PR #2 — typed fiber
isolation with its compile_fail proof, five refusal tests, ℓ-generalized
shells, C0↔C1 transport + centroid retract; WB-7 mock law enforced in the
shell (badge + SYNTHETIC tags); OOH surface certified and seam-scanned;
ab-initio dE₄ evaluation with the 40-witness gate (11/29 sign structure
reproduced); ozone seam scan (51 points, no seams, grid frozen); the last
VALID P2: OOH-complete MBE3, water 0/8, O₄ aggregation, OOO fence fired —
both pre-committed forks active (build ozone; dE₄ rides).

**IN FLIGHT:** (O,O,O) tabulation (~900/14,025 knots at the measured
~720/hr, price closing); certification suite banked and waiting on it.

**THE REORDER (operator, 2026-08-31): the water verdict does NOT wait for
ozone.** The immediate next run is frozen P2 with MBE3(OOH-complete) plus
dE₄(O,H,H,H) riding in the trajectory loop, cutoff-gated (a quadruple
solves only when compact under R_CUT = 6; the switch zeroes the rest
without a solve — the evaluation counter reports how many actually
fired). The four OOO triples stay HONESTLY FENCED at exactly 4/seed.
Acceptance: fence = [4×8], dE₄ counter > 0 on compact encounters, census
with the water count as the headline. When the ozone table lands, its
certification upgrades the fence to served and P2 reruns once more with
fence 0 — the ozone arm then measures what OOO changes, cleanly separated
from what dE₄ changed.

**C1'S REAL GATE: DELIVERED 2026-09-01.** `C1_GATE_PREREG.md` (frozen and
audit-admitted before the first stage ran) and `C1_GATE_RESULTS.md`. Ring-polymer
dynamics on the engine's own STO-3G FCI H–H curve hits that curve's exact
anharmonic vibrational zero-point energy, against a sinc-DVR reference that
certifies its own convergence on four axes and refuses rather than returning a
number it has not convinced itself of; the D₂ isotope shift is measured with the
bead masses as the only thing that moved; the bead-convergence law is confirmed
as a parameter-free forward prediction; `P = 1` reproduces the classical
trajectory bit for bit; and the bead-forgetting commuting square is exactly
closed at one bead, open above it, with both of its scaling laws measured. Seven
of eight gates pass. **G6 — the freeze's own discriminating-power condition —
FIRED**, because the freeze sized it from a Morse plant 49% more anharmonic than
the curve it stood in for; the results document reports it as fired and does not
retro-fit the band. G4's ratio clause also fired, with the wrong sign staked in
the freeze and the derivation, the reference and the instrument all agreeing
against it. One row was added to `DRY_RESIDUALS.md` (**R-8**) against a staked
zero, and three further candidates were folded instead.

**OWED, in order after the P2-with-dE₄ verdict:** the C2 crystal-inheritance staking (DMRG vs FCI referee
on water-dimer nodes); T3 scale-up (dynamic storage, cell lists, PBC,
the ledgered-hand column); reference-state inventory (WB-3.2); the real
wasm engine replacing the WB-7.1 mock.

**WB-8.7 — THE TOWER IS THE INSTRUMENT (operator's law, 2026-09-01).**
The machinery is not preparation for the experiment; it IS the experiment.
Building the complete tower is how the maximal claim gets tested: every
special case, hardcoded branch, or per-composition carve-out the build
FORCES us to write is a measurement against the claim — a witness pair at
the architecture level. Therefore: (1) a **DRY-residual register** is kept
beside the misfit registry — every irreducible special case is entered
with its reason, and the register's GROWTH RATE against domain size is the
claim's live falsifier: short and closed = the fold is winning; growing
with the domain = the DRY is wrong, said quantitatively; (2) the grep-armed
audit extends to code: a hardcoded species/composition branch must cite
either its fold or its residual entry, or the gate refuses; (3) unbuilt
machinery is UNEVALUATED CLAIM-SURFACE — the standing rule that machinery
debt gates campaigns is not project hygiene, it is the requirement that
the instrument be complete enough for its reading to mean something. The
dE₄ incident is the founding case: the unfolded path concealed an 80×
price surprise precisely because nothing had forced its shape into the
open.
