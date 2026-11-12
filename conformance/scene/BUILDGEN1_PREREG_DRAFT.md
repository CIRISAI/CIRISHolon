# BUILDGEN-1 — a dwelling drawn from the joint: type, vintage, storeys, size, wall and roof material, heating fuel and metered energy, for the United States in 2020, with the dependences themselves at stake. PREREGISTRATION DRAFT

*Draft 2026-11-12. Not frozen. Nothing has been downloaded but the survey's codebook (a
document). The lead's words, 2026-11-12: "You have approval for data sources, lets make
holon rich and gorgeous"; and, to the twin's lead and relayed to this session: "power is
a function of building types which are a function of developmental stage per year and
availability of materials and regional norms which have statistical patterns that should
inform a Coppula result".*

## What it is for

The twin draws buildings from code constants and takes power from a table that knows
nothing of the building. The lead's challenge is that these are one object: what a
dwelling is (its type, age, size, materials) and what it uses are dependent, and the
dependence is the science. This campaign builds a generator that draws a dwelling from
their JOINT, and stakes the dependences against states the fit never saw. An attribute
drawn alone from its own table would get each share right and each relation wrong.

## Carrier

DEVICEGEN-1's: the 2020 household energy survey's public microdata (18,496 households),
US Energy Information Administration; the agency's page: "U.S. government publications
are in the public domain and are not subject to copyright protection." The file is the
one recorded by digest in `devices0/raw_file_sha256.txt` (`8d5f6d31…3592`), or the run
stops. The codebook (digest `747f5a3d…`, equal to the one recorded) has been read. Not
committed; deleted after the run. The split into FIT and SCORE halves by state is
DEVICEGEN-1's, by the same seed, checked by its record counts.

## What is known before the read

- From the codebook: the columns and their codes. Type (mobile home; detached house;
  attached house; apartment in a building of 2 to 4 units; apartment in one of 5 or
  more). Year built in 9 bands from "before 1950" to "2016 to 2020". Storeys (houses
  alone). Energy-consuming floor area. Major outside wall material (8 codes) and major
  roofing material (7 codes; not asked for apartments in buildings of 5 or more).
  Building America climate zone (8). Urban type (urban area, urban cluster, rural). Main
  heating fuel. Electricity in kWh and total energy in thousand Btu for 2020, with space
  heating and cooling apart. Degree days. 60 replicate weights.
- From DEVICES-0 and DEVICEGEN-1: the all-sample device tables and the SCORE half's
  device results. No value of any column named above has been read by this campaign;
  DEVICES-0 read mean kWh by housing type for its power table (`tables/power0`), which
  is therefore known, and is listed as known.
- Not in the file: the number of units or floors of an apartment building; roof FORM;
  any colour; a location finer than the state and the climate zone.

## What is built

In `holon-scene`, integers only, wasm32: `buildings::generate(seed, zone, urban) ->
Dwelling`, for the United States 2020; any other reference returns `unresolved`.

- **Stratum:** the survey's climate zone (8) and urban type (3). A stratum under 100 FIT
  records pools urban types, then takes the zone's neighbour named before the parse.
- **Categorical chain, each a table from FIT:** type given stratum; wall material given
  type, zone and vintage era (3 eras fixed before the parse from the 9 bands); roof
  material given wall material, zone and era (houses and small apartment buildings);
  heating fuel given type, zone and era.
- **A rank copula over the ordered attributes:** vintage band, storeys (houses), floor
  area, electricity, total energy. Each has its marginal by type and zone as a quantile
  table; their dependence is 1 correlation matrix of normal scores per type, fitted on
  FIT, with the draw made by a seeded integer approximation to the normal named in the
  record before the parse.
- **Output:** the attributes above; space heating and cooling energy as shares of the
  total by type, zone and fuel (a table, outside the copula); each field's verdict; and
  `borrowed: false`. Units per building and roof form are returned as `unresolved`,
  reason "not in the survey".
- **Not built here:** the crosswalk from Köppen class and the twin's density class to
  the survey's zone and urban type (a further campaign, on county tables); how dwellings
  compose into an apartment building; anything outside the United States.

## Plants

- **P-INDEP.** A synthetic survey with independent attributes: the fitted correlations
  are within 0.03 of 0 and the rival below matches the generator.
- **P-DEP.** A synthetic survey with a planted rank correlation of 0.6 between area and
  energy and a planted material-by-era table: both come back within 0.03, and the rival
  fails on both.
- **P-TAIL.** A synthetic survey whose top deciles coincide more than a normal copula
  allows: S5 below reads its kill.
- **P-MARG.** Each marginal by type and zone is reproduced on FIT to 0.005 in each
  quantile band.
- **P-STRAT.** A thin stratum pools as stated and says so.
- **P-DET.** One digest native and wasm32 for 100,000 calls.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic survey holds what its plant names.

## Stakes (on SCORE; "gap" is two jackknife standard errors of the SCORE value, as in DEVICEGEN-1; no quantity computed from SCORE enters a bar)

- **G0, the gate, on FIT alone** (SCORE's rows are not parsed until it passes): plants;
  P-MARG on the real tables; the section law. If not, nothing ships.
- **D1 (the ordered dependences), 10 pairs** among vintage, storeys, area, electricity,
  total energy, pooled over types with houses alone for storeys: generated Spearman rank
  correlation within the survey's ± (gap + `0.05`). Met: at least 9 of 10. *Kill:* 6 or
  fewer. Then a normal copula of ranks does not carry this joint and nothing ships.
- **D2 (power as a function of building type).** Mean electricity and mean total energy
  per dwelling by type and zone, in each cell of at least 100 SCORE records: certified
  by DEVICES-0's verdict rule with a budget of 15 %. Met: none refused and at least 80 %
  certified. *Kill:* 3 or more cells refused.
- **D3 (type and materials as a function of when and where).** Total variation distance
  between generated and survey shares on SCORE for: type by vintage era; wall material
  by zone; wall material by era; roof material by zone. Met: each at most `0.05` plus
  its gap. *Kill:* 2 or more over `0.10` plus gap.
- **S4 (the rival: is the joint needed?).** A rival draws each attribute alone from its
  own table given stratum. Stake: the rival is outside D1's bar for at least 5 of 10
  pairs and the generator meets D1. *Kill:* the rival meets D1 at 9 of 10. Then stratum
  alone carries the dependences, the copula and the chain are dropped, and the rival
  ships.
- **S5 (the tail).** The share of dwellings in the top decile of both area and total
  energy, within type: generated within the survey's ± (gap + `0.01`). Met, or *kill*:
  the generated share is lower by more than the bar; then a normal copula understates
  the joint tail (the largest homes with the largest use), the generator ships marked
  `tail: understated`, and a heavier-tailed copula is a further amendment.
- **S6 (does vintage earn its place?).** A second rival is the generator without era in
  the material and fuel tables. Stake: its distance for "wall material by era" is over
  `0.10` while the generator's is at most 0.05 plus gap. *Kill:* the second rival is
  within the generator's bar; then era is dropped from those tables.
- **Printed, no verdict.** The fitted correlation matrices by type; energy per unit of
  floor area by vintage band and zone; the 5 strongest and 5 weakest dependences; what a
  dwelling's total energy would be at each country-year carbon intensity in GRID-0 is
  NOT computed (a US dwelling is not carried abroad).

## What would make this vacuous

- One survey, one country, one year (2020, a pandemic year with people at home). Energy
  is modelled by the agency from bills and an end-use model, not metered by end use.
- Rank correlations pooled over types can be met by getting types right and nothing
  within a type; the matrices by type are printed for that reason and D2 is by type.
- The SCORE half has been read before for devices, not for these columns.
- Wall and roof MATERIAL is not roof form or colour; nothing here says what a street
  looks like from above beyond a material share.
- The generator is keyed by the survey's own zone and urban type. Until a crosswalk is
  certified, a caller holding a Köppen class has no key to it.

## Branches

Exactly one fires, in this order.

- **(e)** G0 fails; or the digest differs; or the split's counts differ. Nothing ships.
- **(r)** S4 is killed and the rival is not killed on D2 or D3 (computed for the rival).
  The rival ships.
- **(k)** The generator is killed on D1, D2 or D3. Nothing ships; the result names which.
- **(c)** The generator is not killed but one of D1, D2, D3 or S4 is neither met nor
  killed. It ships with that field marked `unresolved` and the reason.
- **(a)** D1 to D3 and S4 are met. It ships. S5 and S6 mark or prune as their sentences
  say and do not stop shipping.

---
witness: none (a generator scored against a survey's held-out half)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function.
- M-BAR-FROM-THE-READ: bars stated here; what earlier campaigns already read of this file is listed.
- M-PROVENANCE-OVERREACH: the file by digest; the year and country are part of each value; `borrowed` is a field.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
