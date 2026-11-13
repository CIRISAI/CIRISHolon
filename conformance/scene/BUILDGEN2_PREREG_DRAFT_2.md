# BUILDGEN-2 — a dwelling keyed by US STATE: each state's tables blended toward its census division and the nation by how many households it has, with the question "does the state key earn its place over the division" put at stake. PREREGISTRATION DRAFT 2

*NOT FROZEN (2026-11-13). Its adversarial review was started and stopped before it reported, when the session was wrapped up; a fresh review is owed before any freeze. Nothing has been downloaded and no survey row read for it. The keyed generator it names (`holon-scene::keyed`) is built and tested on planted tables only.*

*Revision of `BUILDGEN2_PREREG_DRAFT.md` after `BUILDGEN2_REVIEW.md` and after
REGIONNORM-1 was read (`REGIONNORM1_RESULTS.md`). The draft is kept as it was. Nothing
has been downloaded for this text.*

## An admission before anything else

The survey's file has been read by four campaigns. BUILDGEN-1 read these columns on both
halves of a split by state. REGIONNORM-1 read the FIT half used here (9,238 households)
and published readings from it. The other half (SCORE) of the split named below has
not been parsed by any campaign under that split, but its households were in
BUILDGEN-1's read. Nothing here can be called "states never seen" or "households never
seen". The claims this campaign can support are of two kinds: a model's form holding on
households it was not fitted to, and one level of pooling beating another on those
households.

## What changed from the draft, and why

1. **No hard floor of 100 for the state level.** The review found a floor makes "keyed by
   state" nominal. A state's table is a BLEND instead: with `n` FIT households in the
   cell and a strength `k`, the state's shares are `(n * own + k * division's) / (n + k)`,
   and the division's are blended toward the nation's in the same way. Each state
   contributes in proportion to what it has.
2. **`k` is chosen on FIT alone,** from 10, 25, 50, 100, 200, 400, by leave-one-household-
   fold-out log loss (5 folds by a second hash of `DOEID`), one value for each table
   family. SCORE is not parsed until `k` is fixed and recorded.
3. **The climate zone leaves the key.** REGIONNORM-1 read the division nearer than the
   zone. The generator takes a state and an urban type. What is lost is stated under
   "What would make this vacuous".
4. **The copula stays normal.** The Student copula is not attempted here: one change at
   a time. The upper tail is known to be understated (BUILDGEN-1: 1.6 % against 0.8 %),
   so every dwelling ships marked `tail: understated`, and S5 is printed, not staked.
5. **The correlation matrix may differ by division** for detached houses (REGIONNORM-1's
   L3), where the division has 400 FIT detached houses; elsewhere the nation's.
6. **D3 is corrected for the distance a perfect model shows** (the review's finding 4).
7. **New functions beside the old.** The shipped `generate_with` is pinned by a digest
   test and not changed.

## Carrier

The 2020 household energy survey's public microdata, by the digest BUILDGEN-1 recorded
(M-PROVENANCE-OVERREACH: the digest identifies the file read, and says nothing of the
survey's accuracy); public domain as the agency states; not committed; deleted after the
run.

## Choices fixed here

- **Split:** REGIONNORM-1's. A household is FIT if the first byte of the SHA-256 hash
  (M-PROVENANCE-OVERREACH: a hash here only names a split) of `DOEID:20261113` is even.
  Expected FIT count 9,238; the run stops if it differs.
- **Folds for choosing `k`:** the second byte of the same hash, modulo 5.
- **Places:** the nation; the survey's 10 divisions; 51 states by their 2-digit federal
  code, as `tables/region0` codes them.
- **BUILDGEN-1's eras, fuel groups, storeys levels, area column, 33 knots and normal
  table, unchanged.**
- **The tables and what each is conditioned on.** The conditioning is never let go.
  - type, given place and urban type
  - vintage band, given place and type
  - heating fuel, given place, type and era
  - wall material, given place, type and era
  - roof material, given place and wall material (era dropped: BUILDGEN-1's cells were thin)
  - storeys, given place and type (houses)
  - floor area knots, given place and type
  - electricity knots, non-electric knots, share with no non-electric energy, end-use
    shares: given place, type and fuel group
  - split level, given type and era, the nation alone (as shipped)
- **The blend.** For shares as in item 1 above, `n` the cell's unweighted FIT count,
  `own` its weighted shares. A cell with no household at the nation is written "no such
  dwelling" and the chain cannot reach it. For quantile knots the same weights applied
  knot by knot. A state cell with fewer than 5 households takes the division's row.
- **Three families of `k`:** composition (type, vintage, storeys); materials and fuel
  (fuel, wall, roof); quantities (the knot tables, scored by the pinball loss at the 33
  knots in place of log loss).
- **The level a field reports:** the weight of the state's own households in the blend,
  `n / (n + k)`, in 1/255ths; and the same for the division within what is left.
- **The copula.** The nation's matrix for each type fitted as BUILDGEN-1's (bisection
  through the new function on FIT). For detached houses, 1 further sweep per division
  with 400 FIT detached houses, starting from the nation's.

## What is built

In `holon-scene::buildings`, beside the old: `key2(table, place, a, b, c)`,
`Tables2::find` returning `None` for an absent row, `division_of(state)`, and
`generate_keyed(tables, seed, state, urban) -> Keyed` with BUILDGEN-1's fields, the two
weights for each field, and `tail_understated: true`. A state code that is not one of the
51 is REFUSED (`None`): a US survey says nothing about a place outside the US, and
`borrowed` stays out of this campaign. Units per building and roof form stay unresolved;
roof tone stays assumed.

Also built, for the comparison only: the same tables with the state level switched off
(every state takes its division's blended row). This is the DIVISION RIVAL.

## Plants

Each through the new function, on synthetic surveys.

- **P-OLD.** `generate_with` on BUILDGEN-1's planted tables gives the digest pinned
  before this campaign's code was written.
- **P-BLEND.** Two states of one division differ in fuel mix; one has 2,000 households,
  one has 20. The big state's draws follow its own mix within 0.02; the small one's sit
  between its own and the division's at the weight the blend states, within 0.03.
- **P-NEVER.** No draw for an electric-heated home takes its energy from a gas-heated
  cell, in a survey where the electric cell is thin at each level.
- **P-ABSENT.** A state code of 3, 60 or 0 is refused. A type absent from the nation is
  never drawn.
- **P-K.** In a survey where states truly differ, the chosen `k` is 50 or less; where
  states are identical within divisions, 200 or more.
- **P-RIVAL-NULL.** In a survey where states are identical within divisions, S6 below
  does not read met. Where they differ as much as REGIONNORM-1 read (a median excess of
  0.05), S6 reads met at this survey's sizes.
- **P-D3-NULL.** A generator that is the truth reads D3 met in at least 19 of 20
  synthetic surveys of this survey's size; one whose fuel mix is wrong by 0.10 in total
  variation reads kill.
- **P-DIV-RHO.** Two divisions with vintage-with-storeys of −0.2 and +0.3: the generated
  values land within 0.05 of each.
- **P-HASH.** The split and folds repeat on a second run.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic survey holds what its plant names.

## Stakes

A survey value's gap is 2 jackknife standard errors (60 replicate weights). EXCESS of a
table is its total variation distance from the generated table MINUS the mean distance
that 200 samples of the same effective size drawn from the generated table show. Each
SCORE household is generated under 20 seeds from its own state and urban type.

- **G0, the gate, on FIT alone.** The plants; the section law; the chosen `k` recorded;
  generated rank correlations within 0.03 of FIT's for the nation, and within 0.05 for
  each fitted division. SCORE is not parsed until it passes.
- **D1 (dependences; a check of form on a random half).** BUILDGEN-1's D1a and D1b, bars
  unchanged (gap + 0.05): met at 9 of 10 pooled AND 14 of 16 within type; *kill* at 6 or
  fewer pooled, OR 10 or fewer within type.
- **D1c (the dependence that differs by region).** Vintage with storeys among detached
  houses, for each division with 400 SCORE detached houses: generated within gap + 0.07
  of SCORE's. Met: at least 80 % of scored divisions. *Kill:* under 50 %. Printed beside
  it: the same with the nation's matrix everywhere.
- **D2 (energy).** Mean total energy per dwelling for each state with 150 SCORE
  households, by BUILDGEN-1's verdict rule at a budget of 15 %. Met: at least 80 %
  certified and at most 2 refused. *Kill:* 6 or more refused. And BUILDGEN-1's D2 cells
  within detached houses, by its rule: *kill* 3 or more refused.
- **D3a (materials and fuel, the nation).** Excess at most 0.03 for each of 4 tables:
  wall by roof, fuel by division, fuel by era, type by urban type. *Kill:* 2 or more
  over 0.06.
- **D3b (materials and fuel, each state).** For each state with 100 SCORE households, the
  excess of its heating-fuel-by-type table and of its wall-by-era table. Met: the median
  over state-table pairs is at most 0.04. *Kill:* over 0.08.
- **S6 (does the state key earn its place?).** For the same state-table pairs, the
  state generator's distance against the division rival's. Met: the state generator is
  nearer in at least 60 % of pairs AND a sign test over states (a state counted for
  the generator that wins more of its tables; ties dropped) gives p under 0.05. *Kill:*
  nearer in under 45 % of pairs; then the state level is not needed.
- **S4 (the joint).** BUILDGEN-1's fair rival (identity matrices). Met: the rival is
  outside D1's bar on at least 4 of the 6 within-detached pairs. *Kill:* within on 5.
- **Printed, no verdict:** S5's tail ratio; for each state its electric-heating share
  and mean energy, survey and generated; for the states that span climate zones, the
  survey's fuel table by zone within the state against the generator's single one;
  the weights by table (how much of each state is its own).

## What would make this vacuous

- Every stake is on households of the same states the fit saw, drawn from one survey
  and one year. They test form and the worth of a key. They say nothing of a place the
  survey did not sample, and nothing outside the United States.
- The state is the finest place. Inside a state that spans climates (California, Texas,
  Arizona) every dwelling gets the state's blend. The printed table shows what that
  costs; it is not staked, and a finer key is not built.
- S6 compares two generators of mine. The division rival is fair only if its `k` is
  chosen the same way; it is.
- D1 was killed in BUILDGEN-1 on a split by state. Passing it on a random half is a
  weaker statement and is labelled as a check of form.
- Billed and modelled energy; material is not form or colour.

## Branches

The first that matches fires.

- **(e)** The digest or the FIT count differs; or G0 or a plant fails. Nothing ships.
- **(k)** A kill on D1, D1c, D2, D3a or D3b. Nothing ships; the result names which.
- **(r)** S4 is killed. Nothing ships from this campaign: the joint read real in
  BUILDGEN-1 and a second reading against it is reported as a conflict.
- **(d)** S6 is killed. The division rival's tables ship, marked `state: not needed`,
  if the rival itself has no kill on D1, D1c, D2 and D3a; else nothing ships.
- **(c)** No kill, and any of D1, D1c, D2, D3a, D3b, S4 or S6 is neither. The state
  generator ships with each such field marked `unresolved`.
- **(a)** All met. The state generator ships.

---
witness: none (a generator scored against a survey's held-out households, and against its own division-level rival)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function and the scoring code.
- M-BAR-FROM-THE-READ: `k` is chosen on FIT; the bars are BUILDGEN-1's or stated here; D3b's and S6's bars were written knowing REGIONNORM-1's median excess of 0.049 on FIT, and say so.
- M-POPULATION-CHOICE: each SCORE household under 20 seeds from its own state and urban type, with its weight.
- M-PROVENANCE-OVERREACH: the file by digest; country and year in each reason; weights returned for each field.
- M-FLOOR-UNSTAKED: P-BLEND, P-NEVER and P-K plant the blend; S6 stakes the state level.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
