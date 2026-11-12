# DEVICEGEN-1A — an amendment to the device generator's smartphone step: households with no smartphone, by household size. DRAFT

*Draft 2026-11-12. Not frozen. It needs the 2020 household energy survey's public
microdata again (the carrier of DEVICES-0 and DEVICEGEN-1, deleted after those runs). That
refetch has NOT been approved by name; this draft is written so that it can be put to the
lead. Nothing has been downloaded.*

## Why

- MESH-0 drops each adult the generator gives no smartphone: 0.196 of adults.
- The generator draws smartphones for each person independently (0, 1 or 2, by the
  person's group and the household's connection cell). Against DEVICES-0's all-sample
  table (2020) that gives too many one-person households with none and too few larger
  ones (`MESH0_RESULTS.md`, the correction of 2026-11-12):

| persons | no smartphone, generated | survey (se) |
|---|---|---|
| 1 | 0.297 | 0.230 (0.007) |
| 2 | 0.076 | 0.105 (0.005) |
| 3 | 0.018 | 0.060 (0.007) |
| 4 | 0.006 | 0.037 (0.004) |
| 5 or more | 0.006 | 0.050 or so, pooled from three rows |

- DEVICEGEN-1 had reported the means bending the wrong way at both ends (0.74 against
  0.86 at one person; 4.08 against 3.72 at five or more) and the variance over mean
  failing at five or more (0.49 against 0.79).

## Carrier

DEVICEGEN-1's, unchanged: the 2020 public-use microdata (18,496 households) with its
codebook, public domain, digest `8d5f6d31…3592` as recorded in
`devices0/raw_file_sha256.txt`. If the digest differs, the run stops (branch (e)): the
split and the earlier results belong to that file. Not committed; deleted after the run.
The split into FIT and SCORE halves by state is DEVICEGEN-1's, by the same seed, and is
checked to give the same record counts.

## What is known before the read

The table above, DEVICES-0's and DEVICEGEN-1's published results, and that the SCORE half
has been read once, by DEVICEGEN-1, for these same quantities. So SCORE is not fresh for
the smartphone step: this is a second read of it, and the result says so.

## What is built

Step 3 of `devices::generate` only. Its draws come from a child stream of the household's
seed, so that no other step's draws move. A new `reference` value selects the amended
step; the old one stays callable and bit-for-bit as shipped.

- **3a, the household switch.** With a probability read from FIT by pooled household size
  (1, 2, 3, 4, 5 or more) and connection cell, the household has no smartphone. A stratum
  under 30 FIT records uses its size over all connections.
- **3b, per person.** Otherwise each person draws 0, 1 or 2 by group, connection and a
  size band (1; 2; 3 or 4; 5 or more), from FIT households that have at least one
  smartphone, redrawn until the household's total is at least 1.

## Plants

- **P-SWITCH.** A synthetic survey in which half the households of each size have none
  and the rest have one per person comes back with 0.5 none at each size.
- **P-STREAM.** With the amended step, each other output of 100,000 calls (people by
  group, the cell, each other kind's count) equals the shipped generator's.
- **P-OLD.** The shipped reference's digest is unchanged.
- **P-COND.** No household that passed the switch has 0 smartphones.
- **P-NULL.** On a synthetic survey with independent persons (no clustering), the switch
  fitted from it changes the none-by-size shares by under 0.01.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic survey holds what its plant names.

## Stakes (all on SCORE, generated against the survey's weighted value; "gap" is the survey's FIT-to-SCORE difference as in DEVICEGEN-1)

- **A1 (households with none), 5 pooled sizes.** Generated share within the survey's ±
  (its gap + `0.015`). Met: 5 of 5. *Kill:* outside at 2 or more. Then the switch as
  built does not carry to held-out states and the shipped step stays.
- **A2 (the mean), 5 pooled sizes.** Smartphones per household certified by DEVICES-0's
  verdict rule and budget at each size. Met: 5 of 5. *Kill:* refused at 2 or more.
- **A3 (the shape), 5 pooled sizes.** Variance over mean within 20 % of the survey's plus
  its gap, DEVICEGEN-1's R4. Met: 5 of 5; 4 of 5 is neither; *kill:* 3 or fewer.
- **A4 (nothing else moved).** DEVICEGEN-1's R2 (the have-nothing ratio) is within the
  survey's gap, and P-STREAM holds on the real tables. Not met: nothing ships.
- **A5 (the rival: is the switch needed?).** A rival has 3b's size band and no switch.
  Stake: the rival is outside A1's bar at 2 or more sizes. *Kill:* the rival meets A1 at
  5 of 5. Then the size band alone carries it, the switch is dropped and the rival ships.
- **Printed, no verdict.** The share of adults 18 to 64 and of adults over 64 with no
  smartphone, generated; the same figure a 2026 poll gives (9 % of adults, 21 % over 64,
  read on its page) beside it as ANOTHER YEAR and another instrument; the 2024 survey's
  none-by-size if DEVICES-0's 2024 file is re-read, as a reading of the regime; MESH-0's
  shares re-measured on the amended population.

## What would make this vacuous

- SCORE was read before for these quantities. A1 to A3 are a second read.
- A1 fits a switch to none-by-size and then scores none-by-size; only the split into
  states keeps it honest. A3 and A5 are the checks with something at risk.
- A household count says nothing of WHICH adult has the phone. The person-level share
  is printed and not scored; no person-level reference for 2020 is held.
- The table is 2020, a pandemic year. "Now" is another regime.

## Branches

Exactly one fires, in this order.

- **(e)** A plant fails; or the digest differs; or the split's record counts differ from
  DEVICEGEN-1's. Nothing ships.
- **(n)** Otherwise, A4 is not met. Nothing ships.
- **(k)** Otherwise, A1 is killed for the shipped candidate and A5 is not killed (the
  rival also fails A1). Nothing ships; the shipped step stays and the result says the
  household clustering was not captured.
- **(r)** Otherwise, A5 is killed. The rival (size band, no switch) ships if it is not
  killed on A2 or A3; if it is, nothing ships.
- **(a)** Otherwise. The amended step ships as a new reference value, with A1 to A3 as
  read (a kill of A2 or A3 alone stops shipping and is stated).

---
witness: none (a generator scored against a survey's held-out half)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function.
- M-BAR-FROM-THE-READ: the bars are DEVICEGEN-1's or stated here; what is already known of SCORE is said.
- M-PROVENANCE-OVERREACH: the file is the one recorded by digest; the year (2020) is part of each value and "now" is named as another regime.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
