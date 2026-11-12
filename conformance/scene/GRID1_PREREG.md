# GRID-1 — one dataset read into GRID-0's table: Ember's yearly electricity data, for country rows of one kind where the table now has secondhand rows. PREREGISTRATION

*Frozen 2026-11-12, committed alone, after one adversarial review (`GRID1_REVIEW.md`) whose
named fixes are applied here. It replaces `GRID1_PREREG_DRAFT.md`, kept. The fetch was
approved by the lead on 2026-11-12 ("Approved, proceed", after this dataset alone was put
to them by name). The other twelve datasets listed in `SCOUT_GRID.md` are not approved and
are not fetched. At this commit the file has not been downloaded.*

## Carrier

- **File:** "Yearly electricity data – Global (CSV)", Ember,
  `https://files.ember-energy.org/public-downloads/generation/outputs/release_generation_yearly_global.csv`,
  linked from `https://ember-energy.org/data/yearly-electricity-data/` (the page says last
  updated 11 Nov 2026; the file's stamp is 3 Nov 2026).
- **Licence, read on Ember's page** (`https://ember-energy.org/creative-commons/`): "Ember
  content is released under a Creative Commons Attribution Licence (CC-BY-4.0)". The page
  is read again at the fetch; if it no longer says so, nothing is merged.
- The raw file is not committed and is deleted after the run; its digest and byte count
  are recorded. The table holds countries and groupings of countries, no person or building.

## What is known before the read

- GRID-0's 271 rows. 28 were read on Ember's own pages, 21 of them by Ember's current
  method, with two revisions of 2024 already seen between editions (EU 213 to 211, India
  708 to 705) and one move of method (United States 2023: 369, then 393).
- The file's stamp is later than the review pages, so a later edition is expected, not
  discovered.
- The scout's description: over 200 geographies, 2000 to 2025.
- Not known: the file's columns, how it marks a grouping against a country, its names for
  places, whether it marks a year as estimated, and any value not on the pages above.

## What is built

GRID-0's table, version 3. `file_latest_year` is the latest year for which at least half of
the file's countries have a carbon intensity.

- **New rows.** For each country, and for the world and each grouping the file gives: one
  row for its latest year with an intensity, with `year_lag` (how many years before
  `file_latest_year`); one each for `file_latest_year` less 5 and less 10 where present;
  and one for each place and year a stake below reads.
- **Fields.** The kind string of GRID-0's existing current-method Ember rows, unchanged;
  `edition` (the file's stamp); unit "gCO2e/kWh" with `original_unit` and
  `original_value` if the file's differ; `read: dataset`; trust B; `licence: "CC BY 4.0"`;
  `iso3` if the file has a code; `place_as_published`; for a grouping, `place_kind:
  region` and `grouping: geographic | political | economic`; `provisional: true` if the
  file marks the year as estimated; `small_system: true` if the file gives the place's
  generation and it is under 1 TWh in that year; shares of generation by source for the
  latest year as given, subtotals left out, with `shares_partial` if they do not sum to
  within 1 of 100.
- **Existing rows.** No value, kind or trust of a version-2 row changes, and none is
  removed. Fields are added by this rule alone:
  - `superseded_by: <new row>` only if the row is rated C, its kind says it is Ember's
    generation life-cycle figure (the eight "said to be Ember" rows), and a new row exists
    for the same place and calendar year;
  - `map_alternative: <the place's new latest-year row>` for any other row rated C of a
    place that has a new row (another kind, or another year);
  - `same_series_as: <new row>` for an Ember row read on a page where the file gives the
    same place and year, and `disagrees_with` on both, with the difference, when the
    values differ;
  - no row rated A gets any of these.
- The sidecar gets an `attribution` block: creator and title as published; the file's and the
  data page's addresses; stamp and digest; "CC BY 4.0,
  https://creativecommons.org/licenses/by/4.0/"; what was changed (a selection of years
  and areas; names reconciled; values not altered); that no endorsement is implied; and
  that a consumer must show the credit and the licence link wherever these rows are drawn
  or republished. Any form of credit Ember asks for is recorded word for word from its
  page, and none is invented. The meaning of the three added fields is stated: none of
  them says an old value was wrong.

## Before the parse

The record before the parse may read the header and the distinct values of the columns
that are not numbers (area, type of area, code, variable, unit), and no value. It fixes:
the columns; the name map for GRID-0's 30 existing places that are not subregions; which
groupings are geographic, political or economic.

## Plants (on synthetic files, the last re-run in the file's real layout)

- **P-UNIT.** Known areas, years, intensities and shares come back as the rows expected.
- **P-AGG.** A grouping is never written as a country.
- **P-GAP.** A country whose latest year is blank, zero or not a number gets its latest
  year with a value, and the row carries the lag.
- **P-EST.** A year marked estimated is written `provisional`.
- **P-SHARE.** Shares that do not sum to within 1 of 100 are flagged; subtotals are not summed.
- **P-NAME.** "United States of America" and "EU" land on "United States" and "EU-27"; an
  area in neither the map nor the file's own list of countries stops the run.
- **P-DUP.** The same area and year twice stops the run; the last is never taken silently.
- **P-ID.** New ids are unique and collide with none of the 271.
- **P-UNITCHK.** An intensity in another unit is converted with the original kept, or
  stops the run.
- **P-KEEP.** After the merge, each version-2 row is present and each of its version-2
  fields is identical; the fields added are among the three named. Run on the real merge too.
- **P-LONG.** The plants above pass on a synthetic file in the layout the header shows.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic file holds what its plant names (a grouping, a
gap year, an estimated year, short shares, a second name, a repeated row, another unit).

## Stakes

Bars compare the file's value, unrounded, with the table's; a difference is over the
file's value. A place the file does not have counts against the stake it is in.

- **R1 (the file against Ember's own pages), a reconciliation, 21 rows:** the 18 for 2025
  and 2024 (world, EU-27, China, United States, India, Russia, Japan, Brazil; Asia and
  Latin America for 2025), and world 2015, United States 2023 as revised, United States
  2007. Within `2 %` or `1 g`, whichever is larger.
  - Met: at most three differ; they are named.
  - *Later edition:* more than three differ, each by under `10 %`. Both sets are kept as
    disagreeing rows and the result says how far the edition moved.
  - *Misread:* five or more differ by over `10 %`, or any by a factor of two. Branch (r).
  - The seven rows of the earlier method are printed beside the file, no verdict.
- **S1a (were the secondhand rows usable?).** Five rows rated C are "said to be Ember" for
  2025 (Norway, Sweden, Canada, Mexico, Indonesia). Within `5 %` or `2 g`, whichever is
  larger, of the file for 2025; the same against 2024 is printed, because the sites'
  year may be the edition's. Met: 4 or 5 of 5. *Kill:* 2 or fewer on both years. Then a
  figure "said to be" from a publisher is not a stand-in for it: the scout's reports stop
  entering such rows in a table, and list them as leads alone. Three of five: neither.
- **S1b,** printed, no verdict: the three rows "said to be Ember" for 2023 (Poland,
  Sweden, France), which are by the earlier method.
- **S2 (life-cycle against direct).** "Lower" means by more than `2 %` or `3 g`.
  - Reproduction, labelled so: United States 2023 (`egrid2023-us-total`, 348; Ember's own
    page already reads 393) and France 2025 (`fr-rte-2025-direct`, 19.6).
  - In doubt, five: Canada 2023 (`ca-nir-2023-canada-gen`), South Africa 2024
    (`za-dffe-2024-dggef`), China 2022 (`cn-mee-2022-national`), India 2023
    (`in-cea-fy2022-23-avg-incl-res`, a fiscal year; GRID-0 already shows it at 716
    against a page figure of 713 by the earlier method) and Germany 2025
    (`de-uba-2025-co2-strommix`, which divides by electricity consumed, a near kind and
    marked so).
  - Met: the file's life-cycle figure is not lower in at least four of the five. *Kill:*
    lower in two or more. Then the two kinds are not ordered across sources and a legend
    must not say that life-cycle is the higher. The differences are printed.
- **S3 (what the map gains), printed, no verdict.** Countries with a row; how many at lag
  zero, provisional, small; the ratio of the 90th to the 10th percentile over rows at lag
  zero; and which of the 12 countries that have only rows rated C (Australia, Indonesia,
  Italy, Kenya, Mexico, Netherlands, Nigeria, Norway, Poland, South Korea, Spain, Sweden)
  now have a row rated B.

## What would make this vacuous

- R1 compares Ember with Ember. It tests the reading and the edition, not the method.
- Trust B from one publisher for most of the world is one method's view. IEA and Ember
  disagreed for 2025 in GRID-0 with a sign that flips for China and India; this fetch
  does not settle that.
- S2 compares different sources as well as different kinds, and one of its five is a
  near kind; a difference is not the life-cycle share alone.
- S1a has five rows. It can say a practice failed; it cannot say it is safe.

## Branches

Exactly one fires.

- **(e)** A plant fails; or the file has no carbon-intensity series, as a column or as a
  value of a variable; or the licence page no longer reads CC BY 4.0. Nothing is merged.
- **(r)** Otherwise, R1 reads *misread*. Nothing is merged.
- **(a)** Otherwise. The rows are merged as version 3 with R1, S1a and S2 stated in the
  sidecar as read. No kill of S1a or S2 stops the merge: each changes a practice or a
  legend, as its sentence says.

---
witness: none (a table of priors; R1, S1a, S2 and S3 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the extractor and merge that write the table.
- M-BAR-FROM-THE-READ: the bars are stated here before the file is fetched; §"What is known before the read" lists the reads.
- M-PROVENANCE-OVERREACH: the file's digest, stamp and licence are recorded with an attribution block; trust stays B, with flags for estimated years and small systems.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
