# BUILDGEN-1 — a dwelling drawn from the joint: type, vintage, storeys, size, wall and roof material, heating fuel, and billed and modelled energy, for the United States in 2020, with the dependences themselves at stake. PREREGISTRATION

*Frozen 2026-11-12, committed alone, after one adversarial review (`BUILDGEN1_REVIEW.md`)
whose ten must-fixes are applied here. It replaces `BUILDGEN1_PREREG_DRAFT.md`, kept.
Nothing has been downloaded but the survey's codebook (a document). The lead's words,
2026-11-12: "You have approval for data sources, lets make holon rich and gorgeous"; and,
to the twin's lead and relayed to this session: "power is a function of building types
which are a function of developmental stage per year and availability of materials and
regional norms which have statistical patterns that should inform a Coppula result".*

## What it is for

What a dwelling is (type, age, size, materials, fuel) and what it uses are dependent, and
the dependence is the science. This campaign builds a generator that draws a dwelling
from their JOINT and stakes the dependences against states the fit never saw. A fair
rival draws the same marginals with no dependence; if it does as well, the joint is not
needed and is dropped.

## Carrier

The 2020 household energy survey's public microdata (18,496 households), US Energy
Information Administration; its reuse page: "U.S. government publications are in the
public domain and are not subject to copyright protection." The file is the one recorded
by digest in `devices0/raw_file_sha256.txt` (`8d5f6d31…3592`), or the run stops. Not
committed; deleted after the run. The split into FIT and SCORE halves by state is
DEVICEGEN-1's, by the same seed, checked by its record counts.

## What is known before the read

- The codebook: the columns and their codes.
- Read before, on both halves: housing type and urban type (DEVICES-0; DEVICEGEN-1's
  building contrast). Not read by any campaign here: vintage, storeys, floor area, wall,
  roof, fuel, electricity, total energy. This is the second campaign to read SCORE.
  These columns are read once on SCORE and not again.
- The energy columns are the agency's: totals from bills with imputation, end uses from
  a model. Total energy INCLUDES electricity.
- Not in the file: units or floors of an apartment building; roof form; any colour; a
  location finer than the state and the climate zone.
- Sizes in the review marked as general knowledge are priors and enter no bar.

## Choices fixed here, from the codebook alone

- **Eras:** before 1980 (bands 1 to 4); 1980 to 1999 (bands 5 and 6); 2000 to 2020
  (bands 7 to 9).
- **Fuel groups:** electricity; natural gas; other (propane, fuel oil, wood, other); no
  heating.
- **Storeys:** houses alone, ordered 1, 2, 3 or more; "split-level" is a separate flag.
- **Floor area:** the energy-consuming area (`TOTSQFT_EN`), which includes basements.
- **Code 99 ("other")** is kept as a category of wall and of roof.
- **Zone neighbours for pooling:** Subarctic to Very-Cold to Cold; Very-Cold to Cold;
  Cold to Mixed-Humid; Marine to Mixed-Humid; Mixed-Dry to Hot-Dry; Hot-Dry to Mixed-Dry;
  Hot-Humid to Mixed-Humid; Mixed-Humid to Cold.
- **Floor and drop order, each table:** a cell under 100 FIT records drops, in order,
  urban type, era, then pools its zone with the neighbour above, then with all zones.
  The level used is returned for each field.
- **Marginals:** 33 quantile knots from the 1st to the 99th percentile, linear between
  knots, so that no knot is one household's value.
- **The normal:** a committed table of 1,025 knots of the normal quantile in 16-bit
  fixed point, linear between knots, clipped at 4 standard deviations; the same table
  inverted by binary search. Tables are committed integers, never computed at build.
- **Seeds:** each SCORE household is generated under 20 seeds.

## What is built

In `holon-scene`, integers only, wasm32: `buildings::generate(seed, zone, urban) ->
Dwelling`, United States 2020. Any other reference returns `unresolved`.

1. **Type** given zone and urban type (a table).
2. **Vintage:** a normal score is drawn; its band comes from the vintage marginal by type
   and zone. The era follows from the band.
3. **Heating fuel** given type, zone and era. **Wall material** given type, zone and era.
   **Roof material** given wall, zone and era, for types other than apartments in
   buildings of 5 or more, where it is `not_applicable` by type. **Split-level** given
   type and era, for houses.
4. **The rank copula,** continuing from the vintage score: storeys (detached and attached
   houses alone; `not_applicable` by type otherwise, never drawn and discarded), floor
   area, electricity, and the positive part of NON-electric energy. One correlation
   matrix of normal scores per type: 5 by 5 for houses, 4 by 4 otherwise. Marginals: area
   by type and zone; electricity and non-electric energy by type, zone and fuel group.
5. **Non-electric energy** is zero with a probability from a table by type, zone and fuel
   group, drawn apart from the copula (a simplification, and printed). **Total energy**
   is 3.412 thousand Btu per kWh times electricity, plus non-electric.
6. **Heating and cooling energy** as shares of the total by type, zone and fuel group (a
   table, outside the copula; `unresolved` in each branch, having no stake).
7. **Roof tone,** `roof_tone_assumed`, by the table at the end. **Units in building** as
   the type gives them (1; 1 attached; 2 to 4; 5 or more), with counts and floors
   `unresolved`, reason "not in the survey".

**The section law:** total is at least electricity's 3.412 per kWh; heating plus cooling
is at most the total; no storeys for a mobile home or an apartment; no roof material for
an apartment in a building of 5 or more.

**Fitting.** Tables with the survey's final weights. Spearman is the weighted Pearson
correlation of weighted mid-ranks (ties share the mean rank), the same function on survey
and generated records. Each latent correlation is fitted pairwise by bisection so that
the GENERATED mid-rank Spearman equals FIT's, on a Python mirror of the draw that P-DEP
and the gate check against the shipped function. A matrix that is not positive definite
is made so by clipping eigenvalues at 0.01 and rescaling, and the change is reported.
The shipped tables are refitted on all records and are not the tables scored; the result
says so. One matrix per type across all zones is an assumption; detached houses in Cold
against Hot-Humid are printed.

**Not built here:** the crosswalk from a Köppen class and the twin's density class to the
survey's zone and urban type; how dwellings compose into an apartment building; anything
outside the United States.

## Plants

- **P-INDEP.** A synthetic survey with independent attributes: fitted correlations within
  0.03 of 0, and the rival matches the generator.
- **P-DEP.** Planted rank correlations of 0.6 between area and electricity and of 0.5
  between the 9-band vintage and the 3-level storeys, and a planted wall-by-era table:
  each comes back within 0.03, and the rival fails on each.
- **P-NA.** In 100,000 calls: no storeys for types 1, 4 and 5; no roof for type 5.
- **P-LAW.** The section law holds in 100,000 calls.
- **P-FUEL.** A synthetic survey in which electric-heated homes use twice the electricity
  and no other fuel: the generator's means by fuel group are within 5 %.
- **P-TAIL.** A synthetic survey of 5,700 detached houses whose top 5 % coincide twice as
  often as a normal copula allows: S5 reads its kill.
- **P-MARG.** Each marginal reproduces FIT's to 0.005 in each band plus 3 standard errors
  of the simulation.
- **P-STRAT.** A thin cell drops in the stated order and returns its level.
- **P-DET.** One digest native and wasm32 for 100,000 calls.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic survey holds what its plant names.

## Stakes

Each SCORE household is generated under 20 seeds from its own zone and urban type (type
is DRAWN) and carries its final weight. The gap is 2 jackknife standard errors of the
SCORE value (60 replicate weights, ranks and thresholds recomputed in each); the added
constants stand for FIT's sampling error and the model's slack. No quantity computed from
SCORE enters a bar. A cell needs 400 SCORE records to be scored.

- **G0, the gate,** in a separate process that reads the state column of each row (for
  the split's counts) and each other column of FIT rows alone: the plants; P-MARG on the
  real tables; the section law; generated Spearman on FIT within 0.02 of FIT's for each
  pair. The scoring process starts only on its written pass.
- **D1 (the dependences).** Pairs among vintage, storeys, area, electricity and total
  energy; generated Spearman within the survey's ± (gap + `0.05`).
  - D1a: 10 pairs pooled over types (storeys pairs on houses).
  - D1b: 10 pairs within detached houses and 6 within apartments in buildings of 5 or
    more.
  - Met: D1a at least 9 of 10 AND D1b at least 14 of 16. *Kill:* D1a 6 or fewer, OR D1b
    10 or fewer. Otherwise neither. The other three types are printed with their gaps.
- **D2 (power through the joint), within detached houses.**
  - D2′: mean total energy and mean electricity by area tercile (3), by era (3) and by
    fuel group (electricity, gas, other), 18 cells, by DEVICES-0's verdict rule with a
    budget of 15 %.
  - D2″: the ratio of mean total energy per unit of floor area, 2000 to 2020 over before
    1980: generated within the survey's ± (gap + `0.05`).
  - Met: no D2′ cell refused, at least 80 % of scored cells certified, and D2″ within its
    bar. *Kill:* 3 or more D2′ cells refused, or D2″ outside the survey's ± (gap +
    `0.10`). Otherwise neither.
- **D3 (materials, fuel and type through the joint).** Total variation distance over the
  JOINT cells of: wall by roof (types 1 to 4); fuel by zone; fuel by era; type by urban
  type. Met: each at most `0.05` plus its gap. *Kill:* 2 or more over `0.10` plus gap.
  Otherwise neither. The distance a perfect generator shows at that sample size, from
  resampling FIT, is printed beside each.
- **S4 (the rival: is the joint needed?).** The rival is the shipped generator with the
  identity matrix and with era removed from the chain. On the 6 within-detached pairs not
  involving storeys: Met: the rival is outside D1's bar on at least 4 of 6 AND the
  generator meets D1. *Kill:* the rival is within the bar on at least 5 of 6. Otherwise
  neither.
- **S5 (the tail), within detached houses.** The share in the top 5 % of both area and
  total energy, thresholds taken within each population: the survey's over the
  generator's. *Kill:* the ratio is over `1.5` and outside the gap; the generator is
  marked `tail: understated` and a heavier-tailed copula is a further amendment. Under
  1/1.5 and outside the gap: marked `tail: overstated`. Else met.
- **S6 (does era earn its place?), wall and fuel apart.** A second rival is the generator
  without era in that one table. If its joint distance for the table by era is over
  `0.10` while the generator's is within D3's bar, era stays. *Kill:* the second rival is
  within the generator's bar; era is dropped from that table and the pruned generator is
  scored again on D3 before it ships. Otherwise era stays, marked `era: not shown`.
- Printed as reproduction, no verdict: mean energy by type and zone; type by era; wall
  by zone; wall by era; roof by zone.
- Also printed, no verdict: the matrices by type; detached Cold against Hot-Humid; energy
  per unit of floor area by vintage band and zone; storeys by era; the share with no
  non-electric energy by zone; the 5 strongest and 5 weakest dependences.

## What would make this vacuous

- One survey, one country, one year (2020, a pandemic year with people at home).
- The energy is billed and modelled by the agency, not metered by end use.
- D1 at 0.05 plus gap cannot resolve the three small types; they are printed, not staked.
- Material is not form or colour. `roof_tone_assumed` is an assumption with a name.
- The generator is keyed by the survey's own zone and urban type. Until a crosswalk is
  certified, a caller holding a Köppen class has no key to it.
- The zero of non-electric energy is drawn apart from the copula; a home's being
  all-electric is tied to its size and age only through type, zone and fuel group.

## Branches

The first that matches fires.

- **(e)** The digest differs; or the split's counts differ; or G0 or a plant fails.
  Nothing ships.
- **(k)** The generator is killed on D1, D2 or D3. Nothing ships; the result names which.
- **(r)** S4 is killed. The rival ships (the same marginals, chain without era, and
  not-applicable structure; the identity matrix), with `joint: not needed`.
- **(c)** Any of D1, D2, D3 or S4 is neither met nor killed. The generator ships with:
  D1 neither, `dependence: unresolved`; D2 neither, `electricity` and `total_energy`
  unresolved; D3 neither, `wall`, `roof`, `fuel` and `type` unresolved; S4 neither,
  `joint: not shown to beat the rival`.
- **(a)** D1, D2, D3 and S4 are met. The generator ships.

In (c) and (a), S5 and S6 mark or prune as their sentences say. Each reason carries
"United States, 2020". A field with no stake is `unresolved` in each branch.

## Roof tone: ASSUMED, not certified

No source gives a roof's colour; the survey gives its material. The twin's lead proposed
this mapping on 2026-11-12 as "what the material usually looks like from above, nothing
more". No stake touches it, and no verdict but `assumed` may be shown for a roof's colour.

| roof material (the survey's code) | tone | approximate sRGB |
|---|---|---|
| ceramic or clay tiles | red_tile | 178, 98, 72 |
| concrete tiles | brown_tile | 142, 122, 104 |
| wood shingles or shakes | weathered_wood | 128, 112, 94 |
| slate or synthetic slate | grey_slate | 92, 100, 110 |
| shingles, composition or asphalt | dark_shingle | 84, 82, 80 |
| metal | metal_light | 168, 172, 176 |
| other | neutral | the renderer's present roof colour |
| not asked (buildings of 5 or more units) | concrete_flat | 150, 150, 146 |

The weakest row is the largest: composition shingles are sold in many colours.

---
witness: none (a generator scored against a survey's held-out half)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function.
- M-BAR-FROM-THE-READ: bars stated here; what earlier campaigns read of this file is listed.
- M-POPULATION-CHOICE: each SCORE household under 20 seeds from its own zone and urban type, with its weight.
- M-PROVENANCE-OVERREACH: the file by digest; country and year in each reason; `roof_tone_assumed` named as assumed.
- M-FLOOR-UNSTAKED: P-STRAT is the plant for the pooling floor; the floor and drop order are fixed here.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
