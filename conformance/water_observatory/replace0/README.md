# REPLACE-0 — the records, and what each run taught

The runner is `engine/crates/holon-render/examples/replace0.rs`; the plan and the reading are
`GANTT2.md`, "The fourth review". Every run here is 128 waters at LIQUID-2's state point on
CT-3's served law (the blend at the derived beta, the tables' step, the periodic box), 20,000
settling frames under the stochastic thermostat, then 20,000 counted frames (0.52 ps) NVE on
both arms from ONE branch point, the rigid arm run twice (plain, and with the refinement
demonstration). One core each.

**Nothing here is a claim.** No freeze covers these numbers: one seed, one temperature, a
branch point that is itself still relaxing, and no predeclared tolerances. The runner's
`Validity` record stays `InvariantsOnly` for exactly that reason.

## The reference bundle

`branch.ckpt` (the settled branch state, restored bit for bit) and `flexible.series` (the
flexible arm's readouts at the declared frames and stride) are written by the first run that
pays for them and reused by `--reuse`, so changing the operator re-runs the rigid arms in
minutes instead of repaying the settling and the reference. A reused flexible arm is the same
trajectory to the bit; every record says which it was (`branch_reused`).

## The runs, oldest first

| directory | what changed | what it taught |
|---|---|---|
| `first_unmatched/` | as built | the rigid modes AS PROJECTED read 446 K where the fine box's 3N reading was 319 K, and projection discarded only 0.41 kT/water of vibrational kinetic energy against 1.5 at equipartition — the flexible reference is not equilibrated between its modes |
| `second_pin_geometry/` | momenta rescaled to the 3N reading; the disturbance declared in the rule's own units; the physical event clock; the bundle written | matched to 319 K at t = 0, the rigid arm was back at 386 K one readout later with energy conserved to 3e-5 Ha — the rescaling is undone by the configuration itself |
| `third_mean_geometry_matched3n/` | the lift's reference geometry is the LIQUID's own mean at the branch (O–H 1.964 bohr against the gas-phase pin's 1.944; H–O–H 1.659 rad against 1.689), measured through the box's minimum image | the liquid is elongated and REFINE's energy ledger closes once held units snap to the liquid mean — but the plain arm still heated, so the snap was never the cause |
| `.` (run 4, current) | no rescaling by default; the flexible arm's units PROJECTED at every readout so its rigid modes have their own temperature beside the 3N reading; the replacement error taken on that | the first fair reading — see `run.json`, `replacement_error_second_half` |

## Run 4, as read

Second half of the counted arm, rigid minus flexible, each with its own autocorrelation-aware
error: rigid-mode temperature **+17 ± 9 K**; cross-unit potential **+0.005 kT per water**;
bonds per water **+0.044 ± 0.015** on the lens; O–O first peak **−0.048 ± 0.04 bohr**. All
five gates pass. The rigid step is 16.82 au against the fine 1.0775 (**15.6×**) at the fine
clock's own accuracy target, from a MEASURED contact stiffness envelope (5.61e-2 Ha/bohr²,
`stiffness.json`); the cost is **12.8×** lower per picosecond with projection, reconstruction,
torque accumulation and validation all inside (run 3 read 17.0× on the same 1,321 passes —
the pass count is the number, the seconds are the day's machine load). Energy holds to 4.6e-7
Ha per water over the run. The refinement demonstration spent 32.5 % of its time refined and
still ran 2.85× faster than the flexible arm, with its ledger closed.

The one difference outside its error is the temperature, and the instrument that found it is
in this record too: the flexible box's own bath climbs 461 → 492 K across the counted arm
while its O–H stretches hold 0.44 → 0.49 kT per water. A rigid replacement inherits the bath
and has no vibrational sink for the potential the box is still releasing. That is a candidate
cause with a test, not a conclusion.

## Owed before any of this is a claim

An unseen seed and a nearby temperature; a settled branch point (the campaign's, when the
LIQUID-2 gate has one); predeclared tolerances on every observable above; the held units'
per-frame potential change under reconstruction (unmeasured, ~1e-6 bohr per frame); R3
through the lens on the rigid arm; and the validity record's first `Empirical` entry.
