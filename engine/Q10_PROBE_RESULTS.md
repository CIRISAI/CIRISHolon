# Q10 §9 pre-freeze probes 1 and 2 — results

**Both probes PASS, and the sweep that produced them is INCOMPLETE in a way that
matters.** Both statements are the headline.

Instrument `crates/q8-mps/examples/q10_probe.rs`, whose anchor conventions were taken
from `tests/full_grid_gates.rs` and validated by reproducing the grid harness's readings
digit for digit (`anchor_ph = 3.555534e-10`, `anchor_mag = 7.111037e-10` at N=8 U=16
χ=256). Detached run, log `output/q10/probe12.log`.

## Accounting, before any verdict

| | count |
|---|---|
| planned | 84 |
| **usable readings** | **49** |
| REFUSED (engine declined; a reading, not a gap) | 20 |
| per-config timeout (900 s) | 7 |
| never attempted (outer 3 h cap, `rc=124`) | 8 |

**The 8 never attempted are systematic, not random**: the entire N=12 U=16 column plus
N=12 U=4 χ=256 — the most expensive corner. The 7 timeouts are N=10/12 at χ=256. So the
sweep thins exactly where cost rises, and **it does not reach the operating point Q10's
§1 scope actually targets.**

## PROBE 1 — the fence must VARY. **PASSES.**

| | |
|---|---|
| minimum | 2.119e-23 |
| maximum | 7.332e-03 |
| **span** | **3.46e20 ×** |
| fence UP (≥ 1e-3, the staked threshold) | 6 of 49 |

The staked threshold is crossed, in both directions, on both N=8 and N=10: fence UP at
(8,0,16), (8,1,16), (8,4,16), (8,16,8), (10,1,16), (10,16,8) and DOWN everywhere else.
§9.1's kill — "if it does not vary, the family does not pose the question and Q10 is
VOID-not-killed" — **does not fire.** K2 (FENCE) does not fire.

## PROBE 2 — the anchors must be VIOLABLE. **PASSES.**

| | |
|---|---|
| anchors FIRE (> 1e-6 band) | 23 of 49 |
| anchors HOLD | 26 of 49 |
| worst particle-hole | 4.975e-01 |
| worst magnetization | 9.941e-01 |

The `m_i = 0` theorem anchor fails essentially completely at starved bond dimension
(0.9941 against a 1e-6 band) and holds at sufficient χ. **They both fire and hold**,
which is the whole requirement: an anchor that could not fail would certify nothing, and
one that always fired would gate nothing. **K3b (ANCHOR INERT) does not fire.**

## Bearing on §1's Branch B, unasked for but measured

All 20 refusals sit at **χ ≤ 16; none at χ ≥ 32.** The refusal discriminates rather than
firing everywhere — which is the condition §1 names as Branch B's *failure*: "if the
refusal fires everywhere … then the refusal is useless and BRANCH B HAS FAILED." On this
evidence it does not.

## What is still owed before freeze

- **§9.3 cost** — both arms of every timed contrast, one run, one environment. Not run.
- **§9.4 the §0 discriminator** — built (`examples/q10_discriminator.rs`, three
  bond-dimension-1 product-state starts, no `pad_to_chi`) and **not run**. §0 is already
  discharged by mechanism, but the probe was required independently and trapping is still
  not positively excluded.
- **Coverage at the operating point.** The probes pass, but they pass on a sweep that
  thins at large N and strong coupling. Re-running the missing corner with a longer cap
  is owed before §1's scope is fixed, or §1 must be scoped to where the probes reached.
