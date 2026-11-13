# BUILDGEN-2 — a dwelling from the joint, keyed by STATE, with the level at which the joint is shared put at stake: state, census division, climate zone or nation. PREREGISTRATION DRAFT

*Draft 2026-11-12. Not frozen. It follows BUILDGEN-1's kill (`BUILDGEN1_RESULTS.md`): the
joint is real, its upper tail is heavier than a normal copula, and it does not travel
between states on a climate zone. Nothing has been downloaded for it.*

## An admission before anything else

This survey's file has been read by three campaigns, and BUILDGEN-1 read the columns used
here on both halves of its state split. Nothing in this campaign can be called "states
never seen". What is known from that read is listed below. The stakes here are fixed
before any NEW computation, and the claims they can support are of two kinds only: a
model's form holding on households it was not fitted to, and a comparison between levels
of pooling made by leaving each state out in turn.

## The two questions

1. **At what level is a dwelling's joint shared?** For each state in turn, predict its
   tables from everything but that state, pooled four ways: the rest of its census
   division; the rest of the country in its climate zone; the rest of its division in
   its climate zone; the whole rest of the country. Which is nearest to the state's own?
   BUILDGEN-1 suggests climate alone is poor. It did not test division.
2. **Given a state's own households, does the generator's form hold?** Fit on half of
   each state's households and score on the other half.

## Carrier

As BUILDGEN-1: the 2020 household energy survey's public microdata by its recorded
digest; public domain as the agency states; not committed; deleted after the run.

## What is known before the read

- BUILDGEN-1's results, in full: pooled and within-type rank correlations on both halves
  of its split; the joint tables' distances; mean energy by type, by fuel group and by
  vintage; shares heated by electricity by zone and era; the tail share (1.6 % of
  detached houses in the top 5 % of both size and total energy, in the held-out half).
- No state's own table of any kind. No division's.
- The survey was designed to give estimates for each state; how many households each
  state has is not known to this campaign and is read in the gate.

## Choices fixed here

- BUILDGEN-1's eras, fuel groups, storeys levels, area column, knots and normal table,
  unchanged, with its quantile tables held flat below the 1st and above the 99th
  percentile.
- **Household split:** within each state, a household is FIT if a hash of its record
  identifier and the seed 20261113 is even. Fixed before the read; checked by counts.
- **Divisions:** the survey's own 10 (the Mountain division in two).
- **The pooling ladder for a table cell** (first with 100 FIT records): the state with
  its climate zone; the state; the division with the climate zone; the division; the
  nation. A fuel group, a type or a wall material that defines the cell is NEVER let go.
  A cell with no record at the nation level is written as "no such dwelling" and the
  chain cannot draw it. The generator looks the ladder up at call time and returns the
  level it used for each field.
- **The copula:** a Student copula. Its correlation matrix is fitted per type as
  BUILDGEN-1's (by bisection through the shipped function, on the nation's FIT
  households, with states' zone and urban mix). Its degrees of freedom are 1 number for
  the nation, chosen from 3, 4, 6, 8, 12, 20 and "normal" as the value whose generated
  top-5 %-of-both share for detached houses is nearest FIT's. Committed integer tables of
  the Student quantile for those 6 values; integer square root; no float.

## What is built

`buildings::generate(seed, state, zone, urban) -> Dwelling` with BUILDGEN-1's fields, the
level used for each, and `borrowed: false` for a US state. Units per building and roof
form stay `unresolved`; roof tone stays `assumed`.

## Plants

BUILDGEN-1's nine, through the new function, and:

- **P-LEVELS.** A synthetic survey in which two states of one division differ in fuel
  mix: each state's draw follows its own; a third state with 20 records follows the
  division's and returns that level.
- **P-NEVER.** A cell thin at each level keeps its fuel group: no draw for an
  electric-heated home is made from gas-heated homes' energy.
- **P-STUDENT.** A synthetic survey drawn from a Student copula with 4 degrees of
  freedom: the choice lands on 3, 4 or 6, and the tail stake below reads met. Drawn from
  a normal copula: the choice lands on 20 or "normal".
- **P-LOSO.** A synthetic country in which divisions differ and climate zones do not:
  question 1 reads "division"; and the reverse construction reads "climate zone".
- **P-HASH.** The household split is the same on a second run and gives each state
  between 40 % and 60 % FIT.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic survey holds what its plant names.

## Stakes

The gap of a survey value is 2 jackknife standard errors (60 replicate weights). A cell
or a state needs 400 records to be scored on a mean or a correlation, 200 on a table.

- **G0, the gate, on FIT households alone:** the plants; the marginals; the section law;
  generated rank correlations within 0.02 of FIT's. SCORE households are not parsed
  until it passes.
- **L1 (the level at which the joint is shared), leave one state out, all households of
  the others.** For each state with at least 200 households, and for each of 4 tables
  (heating fuel by type; wall material by era; roof material; type by urban type), the
  total variation distance from the state's own table to each of the 4 predictions.
  - Stake: the division's prediction is nearer than the climate zone's in at least 60 %
    of the state-table pairs. *Kill:* in under 45 %. Then region, as the survey's
    divisions cut it, carries no more than climate does, and the generator's ladder
    puts climate above division. Between: neither, and the ladder stands as fixed.
  - Printed with it: the share of pairs won by each of the 4; the median distance for
    each; the same for each table apart; and the distance a state shows to a resample of
    itself (the floor).
- **L2 (does a state differ from its division by more than noise?).** For the same pairs:
  the share in which the division's distance exceeds 2 times the state's own resampling
  floor. Met (states are their own thing): at least 50 %. *Kill:* under 20 %; then the
  division is enough and the state level is dropped from the ladder. Between: neither.
- **D1 (the dependences, on SCORE households).** As BUILDGEN-1's D1a and D1b, bars
  unchanged (gap + `0.05`): met at least 9 of 10 pooled AND at least 14 of 16 within
  type; *kill* 6 or fewer, OR 10 or fewer.
- **D2 (power through the joint, on SCORE households).** As BUILDGEN-1's D2′ and D2″,
  bars unchanged, within detached houses; AND mean total energy by type for the 5 types
  by the same verdict rule (the cell BUILDGEN-1 overstated). Met: no cell refused, at
  least 80 % certified, D2″ in its bar. *Kill:* 3 or more refused, or D2″ outside
  gap + `0.10`.
- **D3 (materials and fuel, on SCORE households).** Joint distance for wall by roof, fuel
  by STATE, fuel by era, type by urban type, each at most `0.05` plus gap; *kill* 2 or
  more over `0.10` plus gap.
- **S4 (the rival).** BUILDGEN-1's fair rival (identity matrix, no era). Met: the rival
  is outside D1's bar on at least 4 of the 6 within-detached pairs AND the generator
  meets D1. *Kill:* the rival within on at least 5 of 6.
- **S5 (the tail).** BUILDGEN-1's, within detached houses: survey over generated share
  in the top 5 % of both. Met: not over `1.5`, or within the gap. *Kill:* over 1.5 and
  outside the gap; then the Student copula with one number for the nation does not
  carry the tail either, and it ships marked `tail: understated`.
- **Printed, no verdict:** for each state, its electric-heating share, mean energy per
  dwelling and energy per unit of floor area by era, survey and generated; the chosen
  degrees of freedom; the levels used.

## What would make this vacuous

- D1 to D3 are on households of the SAME states the fit saw. They test the model's
  form. They say nothing of a place the survey did not sample, and nothing abroad.
- L1 and L2 are leave-one-out over 51 states that share one survey design and one year.
- The division is the survey's own cut. A better region may exist; this tests one.
- 2020; billed and modelled energy; material is not form or colour.

## Branches

The first that matches fires.

- **(e)** The digest differs; or G0 or a plant fails. Nothing ships.
- **(k)** The generator is killed on D1, D2 or D3. Nothing ships; the result names which.
  L1 and L2 are reported whatever fires, being readings of the survey and not of the
  generator.
- **(r)** S4 is killed. The rival ships, `joint: not needed`.
- **(c)** Any of D1, D2, D3 or S4 is neither. The generator ships with that field
  `unresolved`.
- **(a)** D1, D2, D3 and S4 are met. The generator ships.

L1's and L2's kills change the ladder's order as their sentences say, and the changed
generator is scored again on D3 before it ships. S5 marks.

---
witness: none (a generator scored against a survey's held-out households; two readings of the survey by leave-one-state-out)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function.
- M-BAR-FROM-THE-READ: bars are BUILDGEN-1's or stated here; what BUILDGEN-1 read is listed, and the claims are limited to match.
- M-POPULATION-CHOICE: each SCORE household under 20 seeds from its own state, zone and urban type, with its weight.
- M-PROVENANCE-OVERREACH: the file by digest; country and year in each reason; levels returned per field.
- M-FLOOR-UNSTAKED: P-LEVELS and P-NEVER plant the ladder; L1 and L2 stake its order.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
