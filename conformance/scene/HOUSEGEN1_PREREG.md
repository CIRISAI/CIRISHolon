# HOUSEGEN-1 — the first generator Holon ships for the twin to call: households in a 1 km tile, sizes drawn from a table by density class, summing to the tile's people for every seed; certified for Germany on tiles it never saw. PREREGISTRATION

*Frozen 2026-11-11, committed alone, after one adversarial review (`HOUSEGEN1_REVIEW.md`)
whose named fixes are applied here. It replaces `HOUSEGEN1_PREREG_DRAFT.md`, kept. The
division of labour (Holon builds the generator and produces both sides of each comparison;
the twin calls it) was proposed by the twin's session and confirmed by the lead in this
session the same day ("yes confirm"). At this commit no generator code exists and
nothing has been computed. It needs three files of the approved German dataset fetched
again, because a certificate needs rows fitted on one half of the country.*

*Forecasts on the record. The reviewer expects the real spread between tiles to exceed the
generator's (S6). The author has no forecast for S1 or S5.*

## What is built

In `holon-scene`, integers only, no dependencies, wasm32:
`world::household::generate(seed, people, class, reference, in_country) -> Households`,
holding the sizes and two verdicts (households per unit; household composition), and
`expected_households(people, class, reference)`, the integer the twin may apportion from
above (`people × 1,000,000` over the row's mean in millionths, rounded half up).

- **The unit of a call is a 1 km tile.** The twin calls once per tile and divides the
  households, whole, over its leaves by its own exact rule.
- **The rows.** Germany, per density class: shares in parts per million at sizes `1 … 5`
  and a top band generated as `7`. Each class uses its OWN reading. The village row's
  shares at 4, 5 and 6+ come from columns that failed the disclosure check and are marked
  so. United States rows are compiled in from the published table (top band `8`) and no
  US call is certified: its key is a census tract.
- **The draw.** Sizes are drawn one at a time from a seeded stream. When a drawn size `s`
  exceeds the `r` people left, a household of `r` is kept with probability `r/s` from the
  same stream; otherwise the `r` people join one earlier household chosen from the stream.
  A unit with no earlier household keeps `r`. Nothing is rescaled.
- **The law.** Sizes sum to `people` for every seed; `people = 0` gives none; no household
  is empty.
- **The verdicts of a call** are statements about the rule over many tiles of that class,
  not about one draw, and their reason says so. `certified` needs: Germany, `in_country`
  true, a class certified below, and `people` at or above the floor S2 sets. Otherwise
  `unresolved` with a reason ("read, not certified"; "outside the country read"; "key is
  a census tract"; "below the unit floor"), or `refused` where S1 refused.
- **What it cannot give**, said in the sidecar: who is home; adults against children; which
  households share a building; building type or tenure; vacant dwellings. Communal
  residents are in the people passed and become household members.

## The certificate: both sides produced here

- **FIT and SCORE**: the three-of-six bands of `HOUSEHOLD0_DE_PREREG.md`, same seed. The
  instrument writes each class's shares for FIT, for SCORE and for all cells.
- **The generator under test** uses the FIT rows. Its carrier is the SCORE tiles
  themselves: for each, `people` is the census population count of the tile and the seed
  is the tile's index in sorted order. No synthetic unit size is chosen.
- **Budgets and rule**: 5 % of the reference's persons per household; `0.03` in the
  one-person share; certified if miss + gap ≤ budget, refused if miss − gap > budget, else
  unresolved. The gap is the reference campaign's, between random halves of SCORE's blocks.
- The **top band**: the households-per-unit verdict is the worst of three, with the band
  generated as `6`, `7` and `8`.
- **The rows that ship** are fitted on all cells and labelled so. Gate: each shipped share
  lies between its FIT and SCORE shares (to one part per million).

## Stakes

- **G0.** The section law holds for 100,000 calls, `people` from `0 … 2,000`, each class;
  the output for 1,000 fixed calls has one digest on native and on wasm32; the shipped-row
  gate above holds. If not, nothing ships.
- **S1 (on tiles it never saw), in two parts.** (i) The FIT row against the held-out class
  as a whole. (ii) The generator's pooled output over the held-out tiles against the same.
  A class is certified only if both parts certify on both consumers. Met if each class
  resolved in the held-out half is certified. *Kill:* a refusal in either part on either
  consumer in any such class. Outcome not known.
- **S2 (the draw against its own row).** Synthetic units of `10, 20, 40, 80, 160, 320`
  people, 20,000 each, per class, FIT rows. Expected: within both budgets of the row at
  `40` and above in each class. *Kill:* outside a budget at `160` in any class. The
  smallest size inside both budgets in all five classes is published as the unit floor.
- **S3 (the shape).** At `320` people each size's share is within `0.01` of the row.
- **S4 (what it replaces).** The twin's legacy file scored by S1's rule on the same
  held-out classes, printed beside it. Already read in the reference campaign; no stake.
- **S5 (per tile, the grain the twin reads).** Held-out tiles of at least 80 households:
  the household-weighted share whose generated household count is within 5 % of the
  census count, and whose generated one-person share is within `0.03` plus two binomial
  standard errors for that tile's households. No bar. It is the generator's envelope at
  the consumer's grain, and the same two shares are printed for the legacy rule.
- **S6 (variation between tiles).** Per class, held-out tiles of at least 80 households:
  the standard deviation across tiles of the census one-person share, over that of the
  generator's on the same tiles. Stake: above `1.5` in each resolved class, so a tile-level
  effect is the next thing to build. *Kill:* under `1.2` in each. Then independent draws
  are enough.

## What would make a stake vacuous

- S1 (i) repeats, at class level, a comparison the reference campaign made unit by unit;
  its two halves are one census.
- S2 and S3 compare the generator with its own row: the draw is faithful, not the row right.
- S5 has no bar. Its two shares are worth as much as the reader's sense of what is enough.
- S6's census spread includes the disclosure noise (a count near 30 moved by 1 or 2). The
  ratio is biased upward by it; the floor of 80 households limits that and does not remove it.
- The class of a tile is this census's count. A twin tile gets its class from another
  population model, and nothing here says the two agree.
- Passing the census population makes households high by the communal share (0.6 %).

## Plants

- **P-ONES** (a row of all ones). A unit of `n` people gives `n` households of one.
- **P-FOURS** (a row of all fours, units of 10). Each gives two or three households
  summing to 10, and both outcomes occur over 1,000 seeds.
- **P-SEED** (one seed twice; then two seeds). Identical; different in at least 99 % of
  units of 300.
- **P-BROKEN** (the draw with the last step removed). The section-law test fails.
- **P-BIAS** (the plain truncating draw of the draft, a made-up row of mean 2.68, units of
  8 people). S2's scoring puts it outside the mean budget.
- **P-SHIFTED** (a row with the one-person share raised `0.10`). S1's scoring refuses it
  on composition.
- **P-ZERO** (`people = 0`). No household.
- **P-BETWEEN** (a shipped share set outside its two halves' shares). The gate stops it.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing ships.
- **(a)** S1 met. The generator ships with its rows, verdicts and unit floor.
- **(b)** S1 killed. It ships marked `refused` for the classes that failed, with witnesses;
  the twin's legacy rule stays the fallback there, labelled as having no certificate.

---
witness: `CIRISHolon.ScenePopulation.pop_total` (sizes sum to the unit's people for every seed)
witness: `CIRISHolon.SceneCert.verdict_certified_iff` (the order: envelope, floor, match)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (why building type is not a key: two margins, no joint)
witness: none (S1–S6 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS, M-FOREIGN-DOMAIN-CORROBORATION. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function or the scoring that judges it.
- M-BAR-FROM-THE-READ: budgets and rule are the German freeze's; the held-out halves' class-level shares and every per-tile quantity are uncomputed at this commit. The national and all-cell class values ARE known from the reference campaign, and S1 does not use them.
- M-BASE-RATE-OMITTED: S4 and S5 print the legacy rule beside the generator.
- M-POPULATION-CHOICE: the held-out tiles are the reference campaign's seeded bands, each tile once.
- M-PROVENANCE-OVERREACH: rows carry source notices and digests; the certificate is for the FIT rows and the shipped rows are labelled as the all-cell fit.
- M-FLOOR-UNSTAKED: S2 sets the unit floor and P-BIAS shows its scoring can see a biased draw.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
- M-FOREIGN-DOMAIN-CORROBORATION: a German row certifies no call outside Germany, and no US call is certified at all.
