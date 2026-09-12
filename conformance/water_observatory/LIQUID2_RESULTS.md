# LIQUID-2 — results: the periodic liquid on CT-3's served law, read on three seeds

*Freeze `LIQUID2_PREREG_DRAFT.md` with `LIQUID2_AMENDMENT_1.md` (committed alone, before the
gate was re-run). Runner `engine/crates/holon-render/examples/liquid2.rs` on `holon-campaign`
(`screen`, `size`, `cost`, `gate`, `pilot`, `run`, `read`). Records under
`conformance/water_observatory/liquid2/`: `gate.json` + `gate.done`, `pilot_1x/` (three
60-block pilots and the settling they froze), `seed0..2/arm.json`, `read.json`. Cores 24–26;
each arm about 19.5 hours of wall on one core, `212,407` core-seconds for the three against a
`299,478`-second ceiling. Every file validates as JSON.*

## The verdict, first

**R1 (a), R2 (a), S (a) — the three forward predictions the campaign staked, all three in
band on the seed mean over three seeds. R3 VOID BY DESIGN: self-diffusion was split out as
its own campaign on price and is not read here. L1's drift leg FAILED on all three arms, on a
bar that tightens when a box is settled properly, and the section below states that plainly
rather than around it.**

| | reading, seed mean | the band, pre-committed | branch |
|---|---|---|---|
| **R1** first O–O peak | `5.25` bohr = **`2.778` Å**, height **`3.075`** | `[5.0, 5.6]` bohr, height `[2.0, 4.0]` (Soper 2000; Skinner 2013) | **(a)** |
| **R2** hydrogen bonds per molecule | **`1.6876`** on the lens = **`3.375`** both-ends | `[1.5, 2.0]` on the lens (`[3.0, 4.0]` both-ends; experiment `3.5`) | **(a)** |
| **S** spanning-cluster fraction | **`1.0000`**; largest component `0.99825`; mean degree `3.3753` | a majority of frames winding AND the largest component at or above the Erdős–Rényi giant `0.8738` | **(a)** |
| **R3** self-diffusion | not measured | — | **VOID** (its own campaign) |

Per seed, so the spread is visible rather than summarised:

| | seed 0 | seed 1 | seed 2 | spread |
|---|---|---|---|---|
| settling fired at | `102,100` fr = `2,661` fs | `88,100` fr = `2,296` fs | `90,100` fr = `2,348` fs | — |
| mean temperature | `292.28` K | `296.27` K | `297.24` K | `4.96` K |
| R1 peak position | `5.25` bohr | `5.25` bohr | `5.25` bohr | `0.000` |
| R1 peak height | `3.1059` | `3.0172` | `3.1022` | `0.0887` |
| R2 on the lens | `1.68637` | `1.69001` | `1.68655` | `0.00363` |
| S spanning | `1.0` | `1.0` | `1.0` | `0.000` |

R2's pooled standard error over the three arms is `5.335e-3` against a seed spread of
`3.633e-3`, with Chodera's `g = 58.6` samples per independent one and `n_eff = 19.0` on a
single arm — so the agreement between seeds is not finer than the arms can resolve, which is
the check that makes a small spread meaningful rather than lucky.

## What moved, against LIQUID-1

LIQUID-1 read the same box on a different served law, one seed, and `52` fs of settling.

| | LIQUID-1 | LIQUID-2 | |
|---|---|---|---|
| first O–O peak | `3.04` Å — **outside** experiment's band by `0.09` Å | **`2.778` Å — inside it** | the peak moved into the band |
| peak height | `2.20` | `3.075` | both in band |
| bonds per molecule | `1.184` lens = `2.37` both-ends | `1.688` lens = **`3.375`** both-ends | two thirds of experiment's `3.5` became `96 %` of it |
| second shell | `6.0` Å — a simple liquid's packing | (the RDF is banked in `seed*/rdf.json`) | |
| spanning fraction | OWED, never read | **`1.0000`** on `3,000` sampled frames | the network spans |
| settling | `52` fs, `3.9 %` under its own plateau | `2,296`–`2,661` fs, by a measured criterion | |
| seeds | 1 | 3 | |

**What this is not.** It is not a validated water model. Every band here is this programme's
own pre-committed band around an experimental number, the law is a minimal-basis fit whose
gap to MB-pol COMPARE-0 measured at `3.4x` the law's own error, the density is IMPOSED and
not predicted, and no certificate flips any tier. What it is: the structural readouts LIQUID-1
missed or never took, met on three seeds by a campaign that declared them before it ran.

## The one that failed: L1's drift bar punishes a settled box

The books close. `columns_ok` is true on every arm and the momentum residual is `2.06e-11`
against a bound of `2.98e-7`. What failed is the DRIFT leg, and its arithmetic is worth
stating because the fault is in the bar and not in the arm:

| | seed 0 | seed 1 | seed 2 |
|---|---|---|---|
| drift peak, hartree | `1.246e-6` | `1.416e-6` | `2.206e-6` |
| the bar | `1.763e-7` | `1.280e-6` | `5.765e-7` |
| over by | `7.07x` | `1.11x` | `3.83x` |
| thermostat work, hartree | `0.0126` | `0.0912` | `0.0411` |

The bar is `1.4035e-5 x |thermostat work|`, the fraction read off LIQUID-1's own arm. **The
absolute drift of every LIQUID-2 arm is 4.9–8.6 times BETTER than LIQUID-1's `1.076e-5`
hartree.** What moved is the denominator: LIQUID-1's thermostat did `0.767` hartree of work
over its counted arm because it was counting on a box settled for `52` fs, and LIQUID-2's did
`0.013`–`0.091` because its box was settled by a measured criterion first and the thermostat
had almost nothing left to do. The bar tightened by the same `8`–`60x`, and three arms with
better books than the reference failed a bar derived from the reference.

**The stake's own fence saw one direction of this and not the other.** It says: *"drift_peak
is an extremum and the thermostat column accumulates, so this ratio loosens with arm length;
the screen's arms are all the same length for that reason."* Arm length was controlled.
Settling quality was not, and it moves the same denominator the other way.

Registered as **M-BAR-AGAINST-A-WORKING-THERMOSTAT**, and it is the same shape as
`LIQUID2_AMENDMENT_1.md`'s finding one campaign later: *a bar stated as a ratio to a quantity
that is not a property of the thing being tested will move when that quantity moves, and here
it moves against the campaign for doing the thing the review asked for.* No repair is made
here. The candidates, for whoever freezes the repair: the drift against the arm's own ENERGY
SCALE (`kT` per water over the counted time, which is what a drift costs a reading), or
against the counted time itself, or the fraction re-derived on a settled box — each of which
is a change to a frozen criterion's letter and belongs in an amendment, not in a results file.

## What is owed

- **R3, self-diffusion.** Split out on price: its arm at this law costs `4.66x` this
  campaign's ceiling. It is its own campaign with its own ceiling still owed, and the thing
  that changes its arithmetic is a cheaper dynamics (REPLACE-0, unfrozen) rather than a
  bigger budget.
- **L1's bar**, above.
- **The node-G closure certificate**, which is what would flip this tier's band on the page
  and which no reading here provides. The band stays `measured`.
- **R1's position resolution.** All three seeds report `5.25` bohr because the RDF's bin is
  `0.1` bohr: the zero spread is the histogram's floor, not a precision claim
  (M-FLOOR-UNSTAKED). The height's spread, `0.0887`, is the one that carries information.
