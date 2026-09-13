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

## CORRECTION: the decay below is ONE SEED

The section that follows reports the MSD ratio decaying monotonically with lag and calls it a
property of the operator. It is seed 0's. Read to `1,239` fs the three seeds give `0.809`,
`0.913`, `1.131` — seed 2 rises. Against the seed spread, no lag past `40` fs carries a
resolved difference: the mean ratio stays in `[0.951, 1.022]` while the spread grows from
`0.015` to `0.322`. **This data does not resolve whether the coarse operator's long-time
dynamics differ from the fine model's**, and the gate that passed it compares arms within a
seed without ever asking whether the difference exceeds the scatter between seeds. The
structural and energy readings below are unaffected — their spreads are smaller than their
effects. Kept unedited beneath so the error stays legible.

## TRANSPORT, three seeds (2026-09-13): the walks agree, and the agreement is DECAYING with time

`transport_seed0..2/`, each its own branch point and bundle, `~3` ps counted on both arms from
it, `~22` hours a seed on one core. The question: does the coarse operator's water WALK like
the fine model's? It is the question that gates every use of this operator for transport,
because REPLACE-0 had measured structure and energy and never motion.

**The gate PASSES on all three seeds** — worst departure `0.103`, `0.059`, `0.061` against a
tolerance of `0.2` declared before the run, on 7 lags reaching `540` fs, about 5 crossovers.
Speedup `14.85x`, `14.27x`, `14.79x` with every overhead inside. NVE holds on both arms.

**But read the ladder rather than the verdict** (seed 0; the other two have the same shape):

| τ, fs | MSD flexible, bohr² | MSD rigid | ratio |
|---|---|---|---|
| 40.0 | 0.2576 | 0.2603 | **1.011** |
| 60.0 | 0.4883 | 0.4938 | 1.011 |
| 100.0 | 0.9866 | 0.9964 | 1.010 |
| 159.9 | 1.8449 | 1.8412 | 0.998 |
| 239.9 | 2.9760 | 2.9248 | 0.983 |
| 359.8 | 4.3615 | 4.1516 | 0.952 |
| 539.7 | 6.0460 | 5.4204 | **0.897** |

The departure is not noise and it is not flat. It is **monotone in lag**: the rigid molecules
walk `1 %` FASTER than the fine model's below `100` fs, cross over near `160` fs, and are
`10 %` SLOWER by `540` fs — and the worst departure sits at the LAST lag on every seed, which
is the signature of a trend the window truncated rather than a discrepancy the window
resolved. Both arms are still sub-diffusive there (log-log slopes `1.18`–`1.27`, where `1` is
diffusive), so neither has reached the regime a diffusion constant would be read in.

**What that licenses and what it does not.** It licenses the operator for the sub-picosecond
motion it was measured on. It does NOT license viscosity, stress relaxation or self-diffusion,
which live at picoseconds and beyond — exactly where this reading says the agreement is
getting worse. A constitutive response for a bulk fluid cell is made of precisely those
quantities, so **the honest state of the next tier up is: its inputs are not yet measurable on
this operator, and the reason is measured rather than assumed.**

## The replacement error, three seeds

| | rigid − flexible, mean ± spread |
|---|---|
| cross-unit potential | **`+0.325 ± 0.099` kT per water** |
| bonds per water (lens) | `−0.0125 ± 0.0106` |
| O–O first peak | `−0.0044 ± 0.0267` bohr |
| rigid-mode temperature | `+3.35 ± 4.87` K |

The structure and the temperature agree within their spread across seeds; the earlier `+17` K
was one seed on a shorter branch. The ENERGY does not: the rigid arm is systematically **less
bound, by about `3.4 %` of its own cross-unit energy**, on all three seeds and with a spread
smaller than the effect. That is physically coherent rather than surprising — a flexible
molecule stretches toward the partner it donates to, and a body frozen at the liquid's MEAN
geometry cannot, so it loses exactly the adaptive part of the binding. It is a property of the
lift, not a bug in it, and any use of this operator inherits it.

## Owed before any of this is a claim

An unseen seed and a nearby temperature; a settled branch point (the campaign's, when the
LIQUID-2 gate has one); predeclared tolerances on every observable above; the held units'
per-frame potential change under reconstruction (unmeasured, ~1e-6 bohr per frame); R3
through the lens on the rigid arm; and the validity record's first `Empirical` entry.
