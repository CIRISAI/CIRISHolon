# LIQUID-2 — AMENDMENT 1: the settling criterion's temperature band is derived from the thermostat in force

*Written 2026-09-11, committed alone, BEFORE the gate is re-run and BEFORE any counted arm:
at the time of this commit no `liquid2` run without `--dry` has written a `gate.done`, no
counted seed has stepped a frame, and the only readouts the campaign has are its three
pilots', which this amendment does not touch. Written because the gate ran its settling for
`248,100` frames — `6.47` ps, 22 hours on one core, past the pilots' own frozen floor and two
thirds of the way to the physical cap — and its criterion never fired; the run then died at
exit 101 when the host's filesystem filled from unrelated work, which is incidental. The
criterion is the finding.*

## What the freeze says, and what is wrong with it

§4's criterion has two legs in conjunction. The **U leg** is a least-squares trend of the
window's block means of the cross-unit potential energy against the window's own scatter; it
is satisfiable and it is not amended. The **temperature leg** asks that *every* temperature
sample of the 5-block window — `50` samples — sit inside a band of `21.037` K around `293` K.

That band is `3 x 7.012` K: three times the scatter of **LIQUID-1's own temperature readout**
over the second half of its counted arm. The freeze validated it there, and says so in its
own words (§4 item 4): *"that leg, with this exact 5-block window and this exact band, first
holds at LIQUID-1's frame 18,000 = 469 fs, which is the very frame its bond count settles
at."* That validation is sound — **for LIQUID-1's trajectory**.

**LIQUID-1 ran Berendsen.** LIQUID-2 runs stochastic rescaling (Bussi–Donadio–Parrinello
2007), and the second review asked for exactly that because Berendsen does not sample the
canonical ensemble: its temperature fluctuation is *suppressed*, and being canonical — having
the full fluctuation — is the entire reason the replacement was made. This box's canonical
scatter is not a matter of opinion; the design already derives it, `T sqrt(2/(3N-3))` at
`384` atoms and `293` K, and prints it beside the measured one:

| | |
|---|---|
| LIQUID-1's measured readout scatter (Berendsen) | `7.012` K |
| this box's analytic canonical scatter | `12.224` K |
| the band, as frozen | `21.037` K |
| the band in units of the scatter it is applied to | **`1.72 sigma`**, where `3 sigma` was intended |

The gate record already carried both numbers side by side, with the note *"the measured
scatter is SMALLER than the analytic one, as a thermostatted box's must be"* — true, and the
reason to prefer the measured one under the thermostat that measured it, and the reason not
to under another.

## The arithmetic, which is why 22 hours bought nothing

The leg is a conjunction over `50` samples, so a band expressed in sigma enters it as a
fiftieth power:

| the band, in sigma of the applied trajectory | P(one sample inside) | **P(all 50) — the leg, per window** |
|---|---|---|
| `3.00` (intended) | `0.9973` | `0.87` |
| `1.72` (as applied) | `0.9147` | **`1.2e-2`** |
| `1.72`, mean `5` K off target | `0.8886` | `2.7e-3` |

The pilots' settled means are `294.9`, `289.1` and `284.8` K, so the last row is the operative
one. The gate tested about `82` overlapping windows past its floor — roughly `16` independent
— and zero passes is what that predicts. **The box was equilibrated and the criterion could
not say so.** Two further readings confirm it: the pilots' own settled block means scatter at
`7.4`–`8.8` K, already above the `7.01` K that set the band, and their frozen `t0` came from
Chodera's automated detection, a different rule, which worked.

## The amendment

**A band is a property of the trajectory it is applied to.** The criterion's letter is
otherwise unchanged — same two legs in conjunction, same window, same variable, same "every
sample" rule, same floor and cap — and the band alone is now derived from the configuration
in force, exactly as the settling block and the settling cap already are derived at the step
in force:

```
band = 3 x sigma(thermostat in force)
     = 3 x the analytic canonical sigma           under stochastic rescaling  ->  36.673 K
     = 3 x LIQUID-1's measured readout scatter    under Berendsen             ->  21.037 K
```

`settle_band_k` in `examples/liquid2.rs` is that rule. **The validation on LIQUID-1's log does
not move by a bit**: that validation runs on LIQUID-1's own Berendsen series and keeps
LIQUID-1's own band, carried in the record as `liquid1_temperature_band_k`, so
`settle_floor_from_liquid1` and every number derived through it are unchanged. The declared
comparison arm under `--thermostat berendsen` takes the Berendsen band by the same rule, so
the amendment does not silently widen it.

**What this amendment does not have, stated plainly.** The frozen band was validated against a
reference trajectory; the derived band is not, because this programme has no canonical-thermostat
reference trajectory to validate against. What replaces that validation is a measurement the
runs now make of themselves: every settling records `settling_sample_scatter` — the samples'
own mean, standard deviation, the fraction the band admitted and the worst deviation — and the
SETTLE gate carries a second leg requiring **the band to be at least three times the scatter of
the samples it was applied to**. A band that is again too tight for the trajectory under it
now fails a gate leg in a file, rather than being paid for in wall-clock and inferred
afterwards. On the evidence available the derived band is the right size: the pilots' block
means never leave it, and the U leg fires on their series at blocks `21`, `18` and `24`,
comfortably before the floor at block `43`, so the floor and not the criterion is what sets
the settling — which is what a floor is for.

## What is registered

**M-BAND-FROM-A-SUPPRESSED-SCATTER.** The rule, the third of this shape after the `8x`
constants and M-VALIDATED-NOT-WIRED: *a criterion's band is part of its configuration, and a
band derived under one thermostat, step or law does not travel to another any more than a
floor in frames travels across a step change.* The first two were caught by a reading; this
one was caught by a bill.
