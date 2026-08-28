# holon-render — the interactive atom renderer

Push hydrogen atoms together in a browser and watch H<sub>2</sub> form, or fail to.
Every force comes from an exact tabulated pair potential, the integrator is symplectic,
and every energy and momentum flow — including the ones your own mouse causes — is
written to a ledger with its own gate.

## The point of the thing

Two isolated atoms **cannot bond**, however hard you push them. They approach with their
relative energy above the dissociation asymptote, the dynamics conserve it, so they climb
the repulsive wall and come straight back out. Making a molecule requires taking energy
*away*: a third atom to carry the surplus off, a thermostat, or your own spring used as a
brake. The ledger says exactly how much left and by which route.

Both halves are gated headlessly: `two_atoms_alone_can_never_bond` and
`scripted_push_forms_a_bond_and_the_ledger_stays_closed` run from the **same** initial
condition and differ only in the intervention.

## The curve is data

The potential arrives as `viewer/h2_potential.json`:

```json
{
  "R_grid_bohr": [...], "E_hartree": [...], "F_hartree_per_bohr": [...],
  "R_e": 1.40112, "D_e": 0.174490, "E_asymptote": -1.0,
  "provenance": "..."
}
```

Hartree atomic units throughout; `F` is the **force**, so `dE/dR = -F`. Swapping in a
different curve is the whole migration — **no code change**. The knots are interpolated
by piecewise cubic Hermite (C1), and forces are the analytic derivative of that same
interpolant, so the ledger closes whether or not the file's own `E` and `F` are mutually
consistent. Consistency is a separate question and gets a separate readout: `residual`
(assuming `dE/dR = -F`) against `residual_alt` (the opposite hypothesis). If those ever
swap, the file means the other thing and the viewer says so instead of quietly
simulating a mirror-image molecule.

### The curve is COMPUTED, not loaded

The default route does not read a file at all. At load the wasm calls
`holon_table_generate`, and `holon-chem` solves H<sub>2</sub> in the STO-3G basis
exactly — full CI from closed-form Gaussian integrals, forces and curvature by analytic
differentiation, `R_e`, `D_e` and the dissociation asymptote all located by the same
code. 492 knots in **24.6 ms** in the browser (12.3 ms native). Nothing about the physics is fetched.

The claim that this is the right curve is gated rather than asserted. `holon-chem` is
checked point by point against an independent 50-digit mpmath implementation of the same
model, pinned by digest, at all 492 of its separations:

| | staked | measured (native) | measured (wasm/V8) |
|---|---|---|---|
| `max abs dE` | `1e-12` Eh | `2.46e-15` | `2.44e-15` |
| `max abs dF` | — | `2.84e-14` Eh/a0 | `2.84e-14` |
| `max abs dR_e` | — | `5.8e-16` a0 | — |

The viewer's banner states that residual and the referee's digest, because a residual
without the identity of what it is a residual *from* is not a claim about anything. The
wasm column is measured separately by `check-wasm.mjs`: the browser runs Rust's own libm,
not the host's, so the native number is an inference about it rather than a measurement
of it (the two differ, at `9.7e-16`).

**The file path is still a supported FALLBACK** — a host that cannot run the generator,
or a deliberate A/B against a different curve. The shipped `viewer/h2_potential.json` is
still the Morse **PLACEHOLDER** and the viewer labels it as one if it is ever reached.
Replace it with the real curve using:

```sh
cargo run -p holon-chem --release --example emit_curve -- viewer/h2_potential.json 492
```

but note that doing so is not free: `a_pair_held_by_the_spring_is_bound_but_not_closed`
in `tests/amendments.rs` was staked against the placeholder's exponential wall and does
not survive the real `1/R` one (the driven pair is admitted as a molecule instead of
being rejected on closure, so `closure_rejections` stays 0). Every other gate in the
crate passes on either curve. That scenario needs re-staking before the swap lands.

## The three clocks

Never conflated, because conflating any two of them is the classic real-time-physics
defect.

1. **Physics dt — DERIVED, never chosen.** `omega_e = sqrt(|U''(R_e)| / mu)` is read off
   the curve's own curvature at its own minimum; `dt = period / 64`. Change the JSON and
   every clock moves. Measured on the placeholder: period 7.58 fs, `dt_reference`
   4.8988 a.u. The engine-computed curve is stiffer and re-derives all of them.
2. **Frame rate — MEASURED.** The host passes the wall interval it actually observed.
   Nothing assumes 60 Hz, or any Hz. A fixed-timestep accumulator converts sim-time into
   whole substeps and CARRIES the remainder; dt is never stretched to fit a frame.
3. **Sim-speed — femtoseconds of sim-time per wall-second**, user-visible. The default
   makes one vibration take 2.00 wall-seconds.

**The bound uses the curvature ENVELOPE, not the equilibrium curvature.** The repulsive
wall is far stiffer than the well bottom, so a bound derived from `U''(R_e)` reads green
right through the collision that violates it. `omega_env` is taken over the whole range a
pair can REACH at the largest relative energy seen — and the scan is exact rather than
sampled, because the Hermite interpolant's second derivative is piecewise LINEAR, so its
extremes over a range sit at knots. Measured on the placeholder with a pair that can reach
the wall: `omega_env / omega_e = 2.45`, which refines dt to a quarter of the reference so
the accuracy target still holds.

## Degradation is a contract

Wired through the engine's own tuner (`holon::tune::Policy`), not reimplemented:

- **Default** — `Hold::Exactness` degrading `Latency` without limit. A shortfall dilates
  time: fewer steps per wall-second, every one of them exactly as accurate as declared.
- **Rung (ii), behind an explicit toggle** — `Hold::Latency` degrading `Accuracy` to a
  declared epsilon. dt grows and the enlarged bound is displayed.
- **Refusal** — at `omega_env * dt >= 2` the Verlet map is provably unstable and the engine
  stops stepping rather than producing garbage. Verified in the browser: dt grown 4x drove
  `omega_env*dt` to 2.7585 and the rung read REFUSED at 0 substeps.

The "declaredly, never silently" rule is enforced by the constructor, not by our care:
`Policy::new(Hold::Exactness, [Degrade::Accuracy{..}])` is
`PolicyError::AccuracyUnderExactness`. Rung (ii) has to be a *different policy*, so it
cannot be reached by accident.

A stalled frame (backgrounded tab) is capped at 0.25 s and the capping is REPORTED as
dilation — sim-time dropped on the floor is the same quiet clock-rewriting that silent
substep-dropping would be.

## The gates

One gate per conservation law, never combined — a single "is it OK" number can be green
while energy is right and momentum is 5x wrong. Both close on GRAIN BOUNDARIES
(`grain.rs`'s closure-aligned scheduling); the composite-holon layer runs there too.

**Energy.** `E_kin + E_pair + E_wall + E_spring - W_ext` must not move. Velocity Verlet on
a harmonic oscillator exactly conserves `H~ = 1/2 v^2 + 1/2 w^2 (1 - w^2 dt^2/4) x^2` (the
softening is on the *stiffness*; verified against the step map before it was written
down), so the true energy swings by exactly `(w dt)^2 / 4` of the turning-point energy and
does not drift secularly. Validated against the real curve: predicted 2.963e-8 Eh, measured
2.869e-8 Eh, **ratio 0.968**.

**The amplitude factor is the MODE energy, not the signed total.** The derivation bounds
the error by the sum over modes of each mode's own energy. Using `|E_kin + E_pair + ...|`
instead — the signed total — reads the CONSERVED quantity, and in a bonded scene the
kinetic and (negative) bond terms cancel almost exactly, so it collapses while the
oscillation amplitudes underneath it grow. That shipped, and it produced a live FALSE
ALARM: the gate read 114.4% of bound on physics that was correct. Measured on the repro,
the two are 8.4x apart at N = 11 and up to 37x on the configuration that breached.
`Sim::mode_energy()` sums magnitudes, so no cancellation is possible. See `tests/longrun.rs`.

> One thing worth knowing if you touch the scheduling: the drift EXTREMUM is tracked per
> substep even though the gate VERDICT is evaluated at boundaries. Boundary-only sampling
> is stroboscopic against the vibration — with `dt = period/64`, a 64-substep frame is
> exactly one period, so every boundary lands at the same phase. Measured
> (`examples/diagnose.rs`, probe 5): at 64 and 128 substeps/frame the boundary sample reads
> **0.1110** of the true peak; at 16, 32, 48, 61, 63, 65 and 96 it reads **1.0000**. A gate
> that goes blind exactly when the frame divides the period evenly is a gate that fails on
> the tidy configuration. The extra cost is about seven flops.

**Momentum.** `P - J_ext` must not move, where `J_ext` accumulates the wall and spring
impulse from the very half-kick terms that enter the velocities. Pairwise forces are
applied as one computed value with opposite signs, so they cancel exactly; what is left is
roundoff, bounded by `8 * steps * eps * |p|`. Sampled at boundaries only, and that is fair:
it is a random walk, not an oscillation, so it has no period to alias against.

**Your hand is on the ledger.** The drag spring is a term in the Hamiltonian with a moving
anchor, sampled once per frame and held constant across that frame's substeps — a
ZERO-ORDER HOLD, declared as the interaction model. Under a constant anchor the spring is
conservative, so no work accrues *during* substeps; the work enters exactly at the anchor
move, and `dU` is that work with no path integral to approximate. Grabbing places the
anchor *on* the atom (injects nothing); releasing subtracts the still-stored spring energy,
because the hand leaves with it. The thermostat is on the same ledger for the same reason.

## Molecules are composite holons, not drawn lines

A molecule is a row: `{members, ledger, formed_at, kind}`. Formation is closure
acquisition; dissolution is rent unpaid. Three properties, each enforced rather than hoped:

- **Formation is ACCOUNTING-ONLY.** Creating a row redistributes ledger LABELS and touches
  no dynamical state. Asserted BIT-IDENTICALLY (`E_before.to_bits() == E_after.to_bits()`),
  because "close enough" is exactly the gap a leak hides in. The row's `e_bond` is a VIEW
  of energy the global ledger already holds, never a second reservoir.
- **Closure is MEASURED.** An energy threshold proves a bound pair, not an autonomous
  molecular view. Every candidate scores its own one-step closure defect at each boundary:
  the composite view claims the pair is autonomous, an autonomous pair conserves its bond
  energy, and the defect is how much that claim missed by. A pair being driven by the
  user's spring scores badly and is REFUSED a row — and the refusals are counted. At
  dissolution the defect must RISE.
- **Formation is DETERMINISTIC.** Dwell hysteresis (K = 3 consecutive boundaries, symmetric)
  stops threshold-grazing pairs flickering rows in and out; multi-eligibility resolves
  canonically — most-bound first, ties broken by pair index.

Live in the browser at 16 atoms: 32 pairs bound by energy, **6 composite molecules**, 48
closure rejections. The gap between those numbers is the point.

**The census cost is measured, not asserted.** Frame cost with the composite layer on
against off, N = 16, 120 pairs, 64 substeps/frame: **+0.90% worst case** (every pair
mutually bound) and **+1.79% / -0.02%** on a hot scene. Being matter is expensive — the
O(N^2) force loop is the whole budget. Being a holon is cheap.

## This device, measured

"How many atoms on low-end mobile" is not answerable from a developer's laptop, so the page
finds out on load: a ~200 ms burst of pure physics at N = 16, no rendering, timed by the
host. From the measured pair throughput and the substep rate the sim-speed demands,

```text
N(N-1)/2 <= P   =>   N_max = floor( (1 + sqrt(1 + 8P)) / 2 )
```

solved exactly rather than approximated. ATOMWORLD.md banks `N_max ~ sqrt(P)`; the exact
answer is asymptotically `sqrt(2P)`, so the banked form understates capacity by sqrt(2) —
the 2 from `pairs = N^2/2`. Measured here (shared, contended Linux box): 2.56e+5
substeps/s, 3.07e+7 pairs/s, **N_max = 693** at the default watchable speed and **88** at
120 fs/s. The atom slider is clamped to it.

The force loop is structured so a cell list drops in where the O(N^2) double loop is,
without touching the ledger or the predicate.

## Running it

```sh
# headless gates (native)
cargo test -p holon-render

# browser build
./build-web.sh                       # -> viewer/holon_render.wasm

# the viewer needs http; a file:// page cannot fetch the potential
cd viewer && python3 -m http.server 8731
# then open http://127.0.0.1:8731/index.html
```

Drag an atom with the mouse. The ledger panel is live: the drift meter shows the measured
drift as a fraction of its derived bound, and the curve inset draws `U(R)` by asking the
wasm point by point — the same function the integrator differentiates, not a copy.

## Layout

| | |
|---|---|
| `src/table.rs` | the Hermite interpolant, its extrapolations, the turning-point solve, the curvature envelope |
| `src/clock.rs` | the three clocks, the accumulator, the degradation ladder, capacity |
| `src/holon.rs` | composite holons: rows, dwell, measured closure, the census |
| `src/sim.rs` | atoms, velocity Verlet, THE LEDGER, the gates, the bond predicate, grain boundaries |
| `src/json.rs` | the contract reader — **native only**, the browser uses its own |
| `src/lib.rs` | the raw `extern "C"` ABI |
| `tests/ledger.rs` | the ledger and bond gates (12) |
| `tests/amendments.rs` | clocks, capacity, the capture plant, composite holons (16) |
| `tests/longrun.rs` | the long-run many-body repro for the live gate failure (5) |
| `examples/gate_repro.rs` · `gate_scaling.rs` · `gate_sweep.rs` | the diagnosis: instrumented repro, secularity + dt-scaling, and the sweep that found the breaching configuration |
| `tests/engine_curve.rs` | the engine-computed curve: both routes, the interpolant, NVE, the ABI (7) |
| `examples/make_placeholder.rs` | writes the placeholder fallback curve |
| `check-wasm.mjs` | measures the SHIPPED wasm: generation time, and its residual against the referee |
| `examples/diagnose.rs` | the probe that set the test thresholds from measurement |
| `viewer/` | `index.html`, `styles.css`, `app.js` — input and pixels, no physics |

Two dependencies, both dependency-free themselves: `holon`, for the tuner's
`Policy`/`Hold`/`Degrade` and `grain`'s `Grain` (the real types, not a copy; LTO strips
everything else it carries), and `holon-chem`, for the curve. `holon-chem` is NOT
stripped — it is reached from `holon_table_generate` and is meant to be; it costs
**+13,240 bytes** of wasm, 100,800 to 114,040 (+13.1%). No wasm-bindgen: the ABI is raw `extern "C"` scalars
over a shared static, the same shape `holon-ball-game` and `holon-sandbox` use.
