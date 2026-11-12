# CONNECT-0 — home connectivity by place, one table per kind and per licence: ITU for the world, Eurostat for Europe and its regions, the American Community Survey for US states. PREREGISTRATION

*Frozen 2026-11-12, committed alone, after one adversarial review (`CONNECT0_REVIEW.md`)
whose named fixes are applied here. It replaces `CONNECT0_PREREG_DRAFT.md`, kept. The
scout is `SCOUT_CONNECTIVITY.md`; its section 6 put four sources to the lead by name. The
lead's word, 2026-11-12: "please do the fetches" (after "the connectivity fetch, which is
scoped but unapproved" was listed to them). This text reads that as approval of the
scout's items 1 to 3. Item 4 (OECD) is NOT fetched: its permitted-use sentence was only
part-read. At this commit nothing has been downloaded.*

## What it is for

MESH-0's open input is "whether a household has a host that is on". No table says that.
What is published is of different kinds that must not be mixed:

- **K1** households with internet access at home, any network, a shared phone included.
- **K2** households with a paid cable, fibre or DSL line.
- **K3** households with a computer.
- **K4** fixed-broadband subscriptions per 100 inhabitants, business lines included
  (satellite and fixed wireless count as fixed here; they do not in K2).

The hosting question is nearest K2, and K2 has no world table. This campaign reads what
exists and measures how far the stand-ins sit from K2 where both are seen.

## Carriers, units of branching, and the licence each table carries

There are 6 units, each with its own branch: 3 ITU indicators, 2 Eurostat tables, 1 ACS
year.

- **ITU DataHub**, indicators 12047 (K1), 8917 (K3), and the fixed-broadband
  subscriptions series per 100 inhabitants (19257 or 19303; which is read at the header
  step). Licence as read: "The Data is licensed to the public through a Creative Commons
  Attribution-ShareAlike 3.0 IGO." The derived table is its OWN file under that licence.
  Its sidecar says: the licence and its address; ITU's suggested citation word for word
  with the access date; what was changed (indicators and years selected, names mapped,
  values unaltered); that no endorsement is implied and ITU's name and logo are not used;
  that the file is not under the repository's licence nor any CC BY table's; that
  anything adapted from these rows, a map or a prior in MESH-0 included, carries this
  licence; and which rows are third-party, from the source line read at the header step.
  Per-country differences in S1 and S3 are adapted from ITU values and live in this file;
  the write-up prints counts and summaries. That placement is this campaign's choice
  and the lead may reverse it.
- **Eurostat** `isoc_ci_in_h` (countries) and `isoc_r_iacc_h` (regions), from Eurostat's
  dissemination interface (`ec.europa.eu/eurostat/api/dissemination`), which needs no
  key. Licence as read: reuse "for commercial or non-commercial purposes is authorised
  provided the source is acknowledged." A changed table carries Eurostat's disclaimer of
  responsibility and the citation form it asks for.
- **American Community Survey 1-year, table B28002, by state,** for the year of ITU's
  latest United States value (2024 as read), from the table-based summary file at
  `www2.census.gov` (no key, no agreement; `ACS_DOCS.md`). `ACS_DOCS.md` already searched
  census.gov for a reuse sentence and found none. The house's standing wording for ACS
  rows is used, as in `tables/household0_us`: "no licence text attached; the Bureau asks
  to be cited as the source". The lead may withdraw the table on that ground.
- For each: the page that states the licence is read again at the fetch and must still
  hold the sentence quoted above (for the ACS: the citation page). No account is made, no
  key entered, no terms ticked, no form filled. Raw files are not committed and are
  deleted after the run; digest, byte count and address are recorded. The tables hold
  places, no person.

## What is known before the read

- ITU K1, read as three significant figures on a rendered list: Central African Rep.
  4.54 (2019), Mozambique 11.1 (2024), India 86.3 (2025), United States 94.9 (2024),
  Norway 99.5 (2024), Korea (Rep. of) 100 (2025).
- Eurostat's EU household share: 94.7 in 2025 (read), 93.1 in 2023 (a search summary).
- From DEVICES-0 (the 2020 household energy survey): US households with fixed broadband
  72.8 %, cellular only 14.6 %, no internet 7.1 %. So a national gap between K1 and K2
  near 20 points is expected, not discovered.
- The B28002 lines, verbatim in `ACS_DOCS.md`: 001 total; 002 with a subscription; 004
  broadband of any type; 007 cable, fibre or DSL; 009 satellite; 012 access without a
  subscription; 013 no access.
- Not known: each file's format, columns, flags, year coverage; which ITU id is per 100;
  whether ITU's European and US household values are Eurostat's and the ACS's own; each
  other value. To confirm at the header step and not assumed: Eurostat's table titles,
  year ranges, labels and the 16-to-74 denominator; ITU's flags and source column.

## Before the parse

For each file the record before the parse may read the header and the distinct labels of
the columns that are not numbers, and no value. It fixes: the columns; the selection by
label (for Eurostat: the EU-27 code, the household-internet-access indicator, the unit
"percentage of households", all household types; a label not present is branch (e)); the
name map to ISO codes; which rows are groupings; how an estimate or a break is flagged;
ITU's source column; which ITU id is per 100 inhabitants.

## What is built

`tables/connect0/`: `itu_connectivity.json`, `eurostat_household_internet.json`,
`acs_b28002_states.json`, each with a sidecar and licence block.

- A row is a place, a year, a kind with the publisher's definition and denominator, a
  value with its unit, `read: dataset`, the edition, and flags as the file gives them.
  ITU and Eurostat: each year the file gives from 2014 on (the span of GRID-0's series).
  A `series_segment` is incremented at each break flag and the sidecar says "no change
  over time is computed across a break". Where a file has no estimate flag, rows carry
  `estimated: not stated by the file`.
- ACS, one year: K1 = lines (002 + 012) over 001, with 002 over 001 beside it; K2 = line
  007 over 001; satellite (009), cellular-only and no-access shares printed; the counts'
  margins of error carried as given, and no margin derived for a share.
- Eurostat rows say on their face: households with a member aged 16 to 74. Regional rows
  carry the NUTS level; the regional file's national rows are used for R2b alone.
- ITU K3 and K4 rows carry `reconciled: none`. No value is converted between kinds.

## Plants (synthetic files; the last re-run in each file's real layout)

- **P-KIND.** A file holding two indicators never writes one under the other's kind.
- **P-FLAG.** A flagged value is written with its flag; a break starts a new segment.
- **P-GROUP.** A grouping (world, a region, EU-27, the euro area) is never a country.
- **P-DUP.** The same place, year and kind twice stops the run.
- **P-BLANK.** A blank, a colon or a letter where a number belongs writes no row.
- **P-SHARE.** A household share outside 0 to 100 stops the run.
- **P-K4.** A subscriptions file whose unit label is not per 100 inhabitants, or whose
  latest-year median is outside 0 to 60, is not written.
- **P-ACS.** Shares are computed from the lines named; a file whose identities (R3) fail
  is not written.
- **P-SEL.** A Eurostat file read with the euro-area code, another indicator or another
  unit gives a different row set from the fixed selection.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic file holds what its plant names.

## Stakes

Shares are in percentage points, compared as decimals; "over 5" excludes 5.0.

- **R1 (ITU K1 as its page read), 6 values.** Each within `0.15` points for its year.
  Met: 5 or 6 match, Central African Rep., Mozambique and India among them (the other
  three sit at the ceiling and tell little). *Later edition:* 3 or 4 match and each
  mover is within `2` points. *Misread:* otherwise; K1 is not written.
- **R2 (Eurostat as its page read).** EU-27 2025 under the fixed selection rounds to
  94.7 at 1 decimal, it lies between the lowest and highest member value, and 27 members
  are present. The value under each neighbouring selection is printed; if another also
  rounds to 94.7 the result says R2 does not discriminate. Met, or *misread* and no
  Eurostat table is written. The 2023 figure is printed with no verdict.
- **R2b (the two Eurostat tables agree).** National rows of the regional table against
  the country table, same year: within `0.1` in at least 90 % of countries. Otherwise
  the regional table is not written.
- **R3 (ACS identities), 52 areas.** For each: 002 + 012 + 013 = 001 and 007 ≤ 004 ≤ 002,
  as counts, EXACT. Any failure: the ACS table is not written.
- **S1 (two publishers on K1), in this order.** *Not run:* fewer than 10 countries in
  both for the latest common year. *Same source:* at least 80 % within `0.1`; then S1 is
  a second reading check, not agreement, and those ITU rows get `denominator: as Eurostat
  (member 16 to 74)` and `source_shared: true`. Otherwise: *met* if at most `3` points
  apart in at least 80 % (a bar that is a guess); *kill* if in fewer than 50 %, and then
  a K1 map must not join the two; else neither.
- **S2 (K1 less K2 across US states), a reproduction, expected met:** the gap is over
  `5` points in at least 40 of 52.
- **S2b (is the gap one number?).** The 90th less the 10th percentile of the state gap.
  Met: at most `5` points; then, within the United States, K1 less a constant stands in
  for K2. *Kill:* over `8`; then no single offset is carried from K1 to K2 even within
  one country. Between: neither.
- **S3 (ITU against the ACS, United States, the ACS's year).** *Not run* if ITU has no US
  value for that year. *Same source* if under `0.1` apart. Else printed with the
  difference; met if within `3` points. No kill.
- **Printed, no verdict.** Per kind: places with a row, year range, count flagged, the
  10th and 90th percentiles in the latest year; the spread of the cellular-only share.

## What MESH-0 may take from this

K2 from the ACS for a US state as a CEILING on "a host that is on", never as its value.
It may not carry the US gap to another country, nor use K1 or K4 as a hosting share
anywhere. A subscription says nothing of a public address, uptime or power. A value taken
from the ITU file carries ITU's licence into the table that takes it.

## What would make this vacuous

- R1, R2 and R3 test the reading, not the truth of a survey. K3 and K4 have no check.
- S2 is a reproduction. S2b is one country and one survey's wording.
- S1 and S3 may each be one source read twice; the order of outcomes says so first.
- None of K1 to K4 is "a host that is on". MESH-0's input stays unresolved.

## Branches

For each of the 6 units exactly one fires, in this order: **not obtained** (the file
needs an account, a key, a form or accepted terms, or the licence sentence is no longer
on its page); **(e)** a plant fails for its layout, a label of the fixed selection is
absent, or the extractor stops for any reason; **(r)** its check reads misread (R1 for
ITU K1; R2 for the Eurostat country table; R2b for the regional table, which is also not
written if the country table is not; R3 for the ACS; P-K4's gate for K4; K3 has none);
**(a)** written. ITU K3 and K4 do not depend on R1. S1 needs ITU K1 and the Eurostat
country table written; S3 needs ITU K1 and the ACS; S2 and S2b need the ACS; a stake
whose tables are not written is recorded as not run.

---
witness: none (tables of priors; R1 to R3, S1, S2b and S3 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the extractor that writes the table.
- M-BAR-FROM-THE-READ: the bars are stated here before a file is fetched; what was read on pages and what DEVICES-0 already holds are listed.
- M-PROVENANCE-OVERREACH: digest, edition and licence per file; one table per licence; kinds never converted; a rule for what MESH-0 may take.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
