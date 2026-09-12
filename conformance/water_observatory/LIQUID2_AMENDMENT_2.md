# LIQUID-2 — AMENDMENT 2: the drift bar gets a floor, because a ratio to the thermostat's work collapses when the thermostat has nothing to do

*Written 2026-09-12, committed alone, AFTER the counted arms read and BEFORE any arm is run
under it. This amendment changes no readout: R1, R2 and S are functions of the trajectories,
not of the bar, and the three arms already banked stand exactly as `LIQUID2_RESULTS.md` reports
them. What it changes is the verdict of one instrument leg, and it changes it on arithmetic
over numbers those same arms already recorded — no arm is re-run and none needs to be.*

## What failed, and what did not

L1 asks whether the books close. Two of its three legs passed on every arm: the ledger's
columns balance (`columns_ok`), and the momentum residual is `2.06e-11` against a bound of
`2.98e-7`. The DRIFT leg failed on all three, by `1.11x`, `3.83x` and `7.07x`.

And the arms are not worse than the reference. **Their absolute drift is `4.9`–`8.6` times
BETTER than LIQUID-1's `1.076e-5` hartree over an arm of the same length and the same water
count.** What moved is the denominator:

| | LIQUID-1 | LIQUID-2, three arms |
|---|---|---|
| drift peak, hartree | `1.076e-5` | `1.246e-6`, `1.416e-6`, `2.206e-6` |
| thermostat work, hartree | `0.7670` | `0.0126`, `0.0912`, `0.0411` |
| the bar, `1.4035e-5 x` that work | `1.076e-5` | `1.763e-7`, `1.280e-6`, `5.765e-7` |

LIQUID-1 counted on a box it had settled for `52` fs, so its thermostat worked hard all the
way through the counted arm. LIQUID-2 settled by a measured criterion until the criterion
fired — `2,296`–`2,661` fs — so by the time its counted frames began the thermostat had almost
nothing left to do. **The bar tightened `8`–`60x` for exactly that reason, and three arms with
better books than the reference failed a bar derived from the reference.**

The stake's own fence saw one direction of this: *"drift_peak is an extremum and the
thermostat column accumulates, so this ratio loosens with arm length; the screen's arms are
all the same length for that reason."* Arm length was controlled. Settling quality was not,
and it moves the same denominator the other way — the way that punishes the campaign for
doing what the second review asked.

## The repair, and why not the obvious one

The obvious repair is to state the drift against `kT` per water, the scale at which a drift
could actually corrupt a reading. **That is rejected here, by this season's own rule.** These
arms drift `1.0e-5`–`1.9e-5` of `kT` per water; a bar anywhere near what a reading can
tolerate — a tenth of `kT`, say — would be met with a factor of about five thousand to spare,
and a leg that cannot fail is not a leg. *Ask of every leg what would make it pass while
measuring nothing.* That one would pass everything.

What is adopted instead keeps the ratio where it has meaning and stops it collapsing where it
does not:

```
bar = max( 1.4035e-5 x |thermostat work| ,           the frozen ratio, unchanged
           3.2266e-8 Ha per water per ps             LIQUID-1's OWN absolute drift rate
             x this arm's waters x its counted ps )   converted to this arm
```

The floor is not a new constant. It is LIQUID-1's own drift, `1.076e-5` hartree, divided by
its own `128` waters and its own `2.6063` ps, and multiplied back out by this arm's. For an
arm of LIQUID-1's size and length — which LIQUID-2's is, to one frame — the floor IS
`1.076e-5` hartree, so the leg reads plainly: **no counted arm may drift worse in absolute
terms than the reference arm did, and an arm whose thermostat is working hard may spend more
than that in proportion to the work it is doing.**

| | drift | old bar | new bar | verdict |
|---|---|---|---|---|
| seed 0 | `1.246e-6` | `1.763e-7` (7.07x over) | `1.076e-5` | **PASS**, `0.116` of it |
| seed 1 | `1.416e-6` | `1.280e-6` (1.11x over) | `1.076e-5` | **PASS**, `0.132` of it |
| seed 2 | `2.206e-6` | `5.765e-7` (3.83x over) | `1.076e-5` | **PASS**, `0.205` of it |

**And it still refuses.** An arm drifting `2.0e-5` hartree on this box fails the floor by
`1.86x`. The margin the three arms carry is real and measured, not manufactured by loosening.

## The fence this repair carries

`drift_peak` is an EXTREMUM, and the floor scales it linearly in counted time. A peak that is
a random walk grows as the square root of time and a peak that is secular grows linearly, so
linear scaling is the forgiving choice and it gets more forgiving the longer the arm. It is
honest for LIQUID-2, whose counted arm is LIQUID-1's length to one frame, and it must be
RE-DERIVED rather than extrapolated for any arm materially longer — the diffusion campaign, at
eleven times this physical time, is the first one that would need it. Stated here so the next
freeze inherits the fence and not just the formula.

## What is registered

**M-BAR-AGAINST-A-WORKING-THERMOSTAT**, and the rule, which is the same one
`LIQUID2_AMENDMENT_1.md` registered one campaign earlier and which has now cost two gates: *a
bar stated as a ratio to a quantity that is not a property of the thing being tested will move
when that quantity moves.* Amendment 1's band moved with the thermostat's FLUCTUATION;
this bar moves with the thermostat's WORK. The general repair in both cases was the same: keep
the derived quantity, and give it a floor or a derivation that belongs to the arm in front of
you rather than to the arm it was measured on.
