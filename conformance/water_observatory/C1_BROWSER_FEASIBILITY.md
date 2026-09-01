# C1_BROWSER_FEASIBILITY — can the ring-polymer carrier be a browser exhibit?

**Status: CRITERION STAKED, NOT YET MEASURED.** This section is committed BEFORE the
probe is built or timed, so that the verdict cannot be a description of whatever the
numbers turned out to be. Everything below the horizontal rule is written after.

**Subject:** `engine/crates/holon-chem/src/rpmd.rs`, the C1 carrier certified natively in
`conformance/water_observatory/C1_GATE_RESULTS.md`, driven from
`engine/crates/holon-render`'s wasm artifact under node.

**Question the FSD asks (§9b): C1 is "not workbench-facing this pass."** Is that a price
or a preference? A measurement decides; this lane does not.

---

## THE STAKED CRITERION (written 2026-09-01, before any timing)

The exhibit ships **iff both** of the following hold:

1. **SPEED.** A `P = 16` call to the wasm probe completes in **≤ 5.0 wall-seconds**
   under node on this box, taken as the **median of three repetitions** on the
   configuration the exhibit would actually ship (the configuration is frozen below
   before it is timed).
2. **CORRECTNESS.** The `P = 1` call reproduces the classical limit `rpmd.rs`
   documents. `harmonic_ring_energy`'s docstring states the tell: *any expression for
   `E_P` that does not give exactly `kT` at `P = 1` is not the energy of this ensemble*,
   and `C1_GATE_RESULTS.md` §4 measures it on the real curve as
   `E_1 = 0.000953421 ± 1.98e-6` against `kT = 1/beta = 0.000950043`, i.e. `kT` plus a
   `+0.36%` classical anharmonic correction. **The check: `E_1 − V_min` must sit within
   `[kT, 1.02 kT]`** — above `kT` (the well is anharmonic and softens outward, so the
   classical `<V>` exceeds `kT/2`), and not more than 2% above it (the native reading is
   +0.36%; a single short chain has more noise than the native 8×400 000, and 2% is
   ~5× the native offset — wide enough for one short chain, narrow enough that a broken
   estimator, a wrong `beta`, or a collapsed ring cannot pass).

**The ladder criterion is a REPORT, not a gate.** The P-ladder must move MONOTONICALLY
UP from the classical value toward the banked anharmonic ZPE
`ZPE_DVR(H2) = 0.011288114850 Ha` (C1_GATE_RESULTS.md §3), and each rung is reported
against the native ladder row for the same `P` (§4, "the bead ladder"). **No convergence
is claimed beyond `P = 32`**: the native ladder puts `P = 32` at `0.010533410`, which is
**6.7% BELOW** the reference, so an exhibit capped at 32 beads is showing the APPROACH,
not the answer, and must say so. A rung landing far outside its native row's
neighbourhood is a defect in the probe, not a discovery.

**If the criterion fails, the probe is REMOVED** and this file carries the numbers, the
criterion, and the refusal, so the FSD's next revision cites a measurement.

### The configuration, frozen before timing

The probe takes ONE argument (`beads`); everything else is a constant compiled into it,
and these are the constants:

| knob | value | why |
|---|---|---|
| curve | `rpmd::BankedPes::h2(n_knots)` | the engine's own generated H–H curve, computed in memory at call time. NOTHING is read from disk — see the dependency note below |
| knots | 1024 | the native campaign used 4096; G2 measured the interpolant's departure from the model at 3.41e-14 Ha there, so the grid is not the error budget. Knots cost `h2_point` calls, which are the expensive ones |
| temperature | 300 K | the native campaign's |
| `dt` | 4.0 a.u. | the native campaign's staked step (G1 headline); its `dt/2` control moved the answer by 0.05% |
| `gamma_centroid` | 0.001 a.u. | the native campaign's |
| chains | **1** | `run_pimd_chains` uses `std::thread::scope`; wasm has no threads. The exhibit runs ONE chain via `run_pimd` |
| steps | 20 000 equilibration + 60 000 sampled | chosen as the largest round number expected to fit the budget at `P = 16`; frozen here before it is timed |
| seed | `0xC1` | deterministic; the exhibit is reproducible or it is not an exhibit |
| `r_start` | the interpolant's own minimum, `BankedPes::minimum().0` | computed, not tabulated |
| returned | `e_virial − V_min`, hartree | the centroid-virial estimator, which `C1_GATE_RESULTS.md` names primary |

`V_min` is `BankedPes::minimum().1` — the INTERPOLANT's own minimum, located by golden
section on the interpolant, which is what the native campaign refers its ZPE to.

**Cap: `P ≤ 32`.** Above that the exhibit would be claiming a convergence it has not
measured, and the `O(P^2)` normal-mode transform is the sampler's cost at large `P`
(C1_GATE_RESULTS.md §7).

**Refusal: H–H only.** One molecule, one curve. Any other species is refused BY NAME.
