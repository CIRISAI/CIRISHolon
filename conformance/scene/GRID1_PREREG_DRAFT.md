# GRID-1 — DRAFT, not frozen. One dataset read into GRID-0's table: Ember's yearly electricity data, for country rows of one kind in place of the rows now rated C. PREREGISTRATION

*REVIEWED 2026-11-12 (`GRID1_REVIEW.md`): freeze with named fixes. Kept as written; the
freeze is `GRID1_PREREG.md`.*

*Draft of 2026-11-12, for one adversarial review. The fetch was approved by the lead on
2026-11-12 ("Approved, proceed", after this dataset alone was put to them by name). The
other twelve datasets listed in `SCOUT_GRID.md` are not approved and are not fetched. At
this commit the file has not been downloaded; its header has not been read.*

## Carrier

- **File:** "Yearly electricity data – Global (CSV)", Ember,
  `https://files.ember-energy.org/public-downloads/generation/outputs/release_generation_yearly_global.csv`,
  linked from `https://ember-energy.org/data/yearly-electricity-data/` (page says last
  updated 11 Nov 2026; the file's own stamp is 3 Nov 2026).
- **Licence, read on Ember's page** (`https://ember-energy.org/creative-commons/`): CC BY 4.0,
  "free to share and adapt our work – as long as you credit us". Each row carries the credit.
- **Known before the read:** GRID-0's rows, 28 of them read on Ember's review pages; the
  scout's report. Not known: any value in the file, its columns, how it marks a region
  against a country, or which year is its latest with an intensity.
- The raw file is not committed and is deleted after the run; its digest and byte count
  are recorded. The table holds countries and regions alone, no person or building.

## What is built

GRID-0's table, version 3. For each area the file marks as a country, and for the world
and each region it gives:

- one row for the latest year that has a carbon intensity, and one each for the years five
  and ten before it where present;
- the shares of generation by source for the latest year, as the file gives them;
- kind: "generation average, life-cycle (Ember's method)", unit as the file states it;
- trust B (read at source; an analyst's estimate from generation and factors, not an
  official inventory), licence CC BY 4.0, the file's date in the note.

No existing row is removed or changed in value. A row rated C for a place and kind that a
new row covers gets a field `superseded_by` naming the new row.

## Checks on the instrument before the read (plants, on a synthetic file)

- **P-UNIT.** A synthetic file with known areas, years, intensities and shares comes back
  as the rows expected, values unchanged.
- **P-AGG.** An area marked as a region is written as a region and never as a country.
- **P-GAP.** A country whose latest year has no intensity gets its latest year that has
  one, and the row says the year.
- **P-SHARE.** Shares that do not sum to within 1 of 100 are written with a flag
  `shares_partial`.
- **P-KEEP.** After the merge every row of version 2 is present with its value unchanged.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: the synthetic file holds a region, a gap year and a short
set of shares.

The columns are named from the file's header. The record before the parse fixes them
after reading the header line and nothing else.

## Stakes

- **R1 (the file against Ember's own pages), a reconciliation.** For the rows of GRID-0
  read on Ember's 2026 review pages for 2025 and 2024 (world, EU-27, China, United
  States, India, Russia, Japan, Brazil, Asia, Latin America; 18 rows), the file's value
  for the same place and year is within 2 %. *If more than three differ by over 2 %:* the
  file is a later edition than the pages; both are kept as disagreeing rows and the
  result says by how much the edition moved.
- **S1 (were the secondhand rows usable?).** Eight rows rated C are "said to be Ember"
  on other sites (Norway, Sweden, Canada, Mexico, Indonesia for 2025; Poland, Sweden,
  France for 2023). Stake: at least 6 of 8 are within 5 % of the file for the same place
  and year. *Kill:* 4 or fewer. Then a figure "said to be" from a publisher is not a
  stand-in for the publisher, and the report's C rows of that sort are worth less than
  their rating says.
- **S2 (life-cycle against direct).** For each place and year that has an official
  direct, generation-based row in GRID-0 (the United States 2023, Germany, France, Canada,
  South Africa, China 2022), the file's life-cycle figure is the higher of the two.
  *Kill:* it is lower in two or more places. Then the two kinds are not ordered and a
  legend must not say so. The differences are printed.
- **S3 (what the map gains), printed, no verdict.** How many countries get a row; the
  ratio of the 90th to the 10th percentile of their latest values; how many of the 14
  countries rated C are now covered at B.

## What would make this vacuous

- R1 compares Ember with Ember. It tests the reading and the edition, not the method.
- A trust B from one publisher for most of the world is one method's view. IEA and Ember
  disagreed for 2025 in GRID-0 with a sign that flips for China and India; this fetch
  does not settle that.
- S2 compares different sources as well as different kinds; a difference is not the
  life-cycle share alone.

## Branches

- **(e)** A plant fails, or the file has no column for carbon intensity. Nothing is merged.
- **(a)** Otherwise the rows are merged as version 3 with the readings above stated in
  the sidecar, whatever R1, S1 and S2 read.

---
witness: none (a table of priors; R1, S1–S3 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the extractor and merge that write the table.
- M-BAR-FROM-THE-READ: the bars (2 %, 5 %, 6 of 8, two places) are stated here before the file is fetched.
- M-PROVENANCE-OVERREACH: the file's digest, date and licence are recorded; trust stays B.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
