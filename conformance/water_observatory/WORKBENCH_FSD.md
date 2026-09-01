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
tier-separation exhibit: ~10⁻¹³ of kT at 1 nm (correctly invisible), the
thing that levels interfaces and sags droplets at 1 mm, and the whole
hydrostatic column at 1 km. One field, silent at the bottom, sovereign at
the top.

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
