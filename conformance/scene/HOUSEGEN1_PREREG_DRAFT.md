# HOUSEGEN-1 — the first generator Holon ships for the twin to call: households in a leaf, sizes drawn from a certified table by country and density class, summing to the leaf's people for every seed. PREREGISTRATION (DRAFT)

*DRAFT of 2026-11-11. NOT FROZEN; to be attacked by a second reader first. The division of
labour is the twin session's proposal of the same day, which it reports the lead approved
there: Holon builds the generator and produces both sides of each comparison; the twin
calls it and draws it. Nothing below is built or computed. It needs the two approved
datasets fetched again (US household tables, German grid), because the published tables
were fitted on all the data and a certificate needs a table fitted on one half.*

## What is built

`holon_scene::world::household::generate(seed, people, class, region) -> (Vec<u8>, Verdict)`,
integers only, no dependencies, wasm32.

- **The table.** Per country and density class, shares of households by size in parts per
  million, compiled in with its digest and source notice. Germany: sizes `1 … 5` and a top
  band generated as `7`. United States: sizes `1 … 6` and a top band generated as `8`.
  Classes that are unresolved in the reference borrow the nearest resolved class's row and
  the call returns `unresolved`.
- **The draw.** Household sizes are drawn one at a time from the row, from a seeded stream,
  until the leaf's people are used up. The last household takes what is left if its drawn
  size would exceed it. Nothing is rescaled.
- **The law.** Sizes sum to `people` for every seed; `people = 0` gives no household; no
  household has size zero. Tested in the crate; `ScenePopulation.pop_total` is the witness
  that this is a section.
- **The verdict of a call.** `certified` only for a country and class whose row was
  certified below; `unresolved` outside that; never silent.
- **What it does not key on.** Building type and tenure. Neither reference publishes
  household size jointly with building type; the US publishes it jointly with tenure, and
  that is a second generator.

## The certificate: both sides produced here

- **FIT and SCORE** as in the two reference campaigns (US: the seeded split of states;
  Germany: the seeded three bands of six). The instruments are extended to write each
  class's size shares for FIT and for SCORE separately. The generator under test uses the
  FIT rows only.
- **The generator's side.** For each country and class, 20,000 leaves of `150` people,
  seeds `0 … 19,999`; pooled shares, persons per household, one-person share.
- **Budgets and verdict rule:** those of `HOUSEHOLD0_DE_PREREG.md` (5 % of the reference's
  persons per household; `0.03` in the one-person share; certified if miss + gap ≤ budget,
  refused if miss − gap > budget). The top band's generated size makes the generator's mean
  depend on a declared number; the mean is also printed with the band at one less and one
  more.
- **The table that ships** is fitted on all the data and labelled so. Its certificate is
  the FIT-against-SCORE one, stated as such.

## Stakes

- **G0.** The section law holds for 100,000 calls with `people` drawn from `0 … 2,000` and
  each class and country; the same seed gives the same households on native and wasm32.
- **S1 (the generator, on places it never saw).** Certified on both consumers in each class
  resolved in SCORE, in each country. *Kill:* refused on either consumer in 2 or more of
  those classes across the two countries. Outcome not known: the reference campaigns scored
  held-out units one at a time, not the held-out class as a whole.
- **S2 (the last household).** Truncating the last household adds small households. With
  the FIT rows, the generator's pooled output stays within both budgets OF ITS OWN ROW for
  leaves of `20` people and more. Read at `3, 5, 10, 20, 40, 80, 150`. *Kill:* outside a
  budget at `150`. The smallest leaf that stays inside is published as the generator's
  envelope in leaf size, and calls below it return `unresolved`.
- **S3 (the shape).** At `150` people each size's share is within `0.01` of the row.
- **S4 (against what it replaces).** The twin's legacy file scored by the same rule on the
  same SCORE classes, printed beside S1. Expected: refused throughout Germany and on
  composition in the US. No kill; both are already read.

## What would make a stake vacuous

- S1 can pass because FIT and SCORE are halves of one survey or one census.
- S3 and S2 compare the generator with its own table. They show the draw is faithful, not
  that the table is right.
- Leaves in the twin hold at most 60 households. Nothing here checks variation BETWEEN
  leaves: independent draws give each leaf the row's own scatter, and no reference at leaf
  scale exists to say whether that is too much or too little.
- The US rows are keyed by census tract density. A twin leaf has no tract. Germany's are
  keyed by tile, by that census's count and not the twin's population grid.

## Plants

- **P-ONES** (a row of all one-person households). Each leaf of `n` people gives `n`
  households of one.
- **P-TRUNC** (a row of all four-person households, leaves of 10). Each leaf gives `4, 4, 2`.
- **P-SEED** (two calls, one seed). Identical. Two seeds: different in at least 99 % of
  leaves of 150.
- **P-BROKEN** (the generator with the last household NOT truncated). The section-law test
  fails.
- **P-SHIFTED** (the generator run with a row whose one-person share is raised `0.10`).
  S1's scoring refuses it on composition.
- **P-ZERO** (`people = 0`). No household.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing ships.
- **(a)** S1 met. The generator ships with its rows, verdicts and leaf-size envelope, and
  the twin is told the call is ready.
- **(b)** S1 killed. It ships marked `refused` for the classes that failed, with witnesses;
  the twin's legacy rule stays the fallback there.

---
witness: `CIRISHolon.ScenePopulation.pop_total` (sizes sum to the leaf's people for every seed)
witness: `CIRISHolon.SceneCert.verdict_certified_iff` (the order: envelope, floor, match)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (why building type is not a key here: two margins, no joint)
witness: none (S1–S4 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS, M-FOREIGN-DOMAIN-CORROBORATION. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function.
- M-BAR-FROM-THE-READ: the budgets and the rule are the German freeze's; the SCORE halves' class-level values have not been computed by anyone.
- M-BASE-RATE-OMITTED: S4 prints the legacy rule beside the generator.
- M-POPULATION-CHOICE: FIT and SCORE are the reference campaigns' seeded splits, unchanged.
- M-PROVENANCE-OVERREACH: the rows carry their source notices and digests; the certificate is for the FIT rows and the shipped rows are labelled as the all-data fit.
- M-FLOOR-UNSTAKED: S2 is the stake on the leaf-size floor.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
- M-FOREIGN-DOMAIN-CORROBORATION: a row certified in one country is used in no other; a call for any other region returns `unresolved`.
