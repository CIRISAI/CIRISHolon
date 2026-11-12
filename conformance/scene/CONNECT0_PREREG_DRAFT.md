# CONNECT-0 — home connectivity by place, one table per kind and per licence: ITU for the world, Eurostat for Europe and its regions, the American Community Survey for US states. PREREGISTRATION DRAFT

*Draft 2026-11-12. Not frozen. The scout is `SCOUT_CONNECTIVITY.md`; its section 6 put four
sources to the lead by name. The lead's word, 2026-11-12: "please do the fetches" (after
"the connectivity fetch, which is scoped but unapproved" was listed to them). This draft
reads that as approval of the scout's items 1 to 3. Item 4 (OECD) is NOT fetched: its
permitted-use sentence was only part-read, and a licence not read is not fetched under.
At this draft nothing has been downloaded.*

## What it is for

MESH-0's open input is "whether a household has a host that is on". No table says that.
What is published is nearer or further from it, and of different kinds that must not be
mixed (the scout's section 3):

- **K1** households with internet access at home, any network, a shared phone included.
- **K2** households with a fixed broadband line.
- **K3** households with a computer.
- **K4** fixed-broadband subscriptions per 100 inhabitants, business lines included.

The hosting question needs K2. K2 has no world table. This campaign reads what exists and
measures how wrong the stand-ins are where both can be seen.

## Carriers, and the licence each table carries

- **ITU DataHub**, indicators 12047 (K1), 8917 (K3), 19257 (K4). Licence read: "Creative
  Commons Attribution-ShareAlike 3.0 IGO". The derived table is its OWN file under that
  same licence, with ITU's citation and access date, marked as changed; it is not merged
  into any CC BY table. That placement is this campaign's choice and the lead may reverse it.
- **Eurostat** `isoc_ci_in_h` (countries) and `isoc_r_iacc_h` (regions), through Eurostat's
  public dissemination address with no key. Licence read: reuse authorised with the source
  acknowledged; a changed table carries Eurostat's disclaimer of responsibility and the
  citation form it asks for (dataset identifier, access date).
- **American Community Survey 1-year, table B28002, by state**, latest release, from the
  Census Bureau with no key and no account. A sentence at census.gov stating that the
  data may be reused is read at the fetch; if none is found on the Bureau's own pages, the
  ACS table is not written and the result says so.
- For each carrier: the licence page is read again at the fetch; no account is made, no
  key entered, no terms ticked. If a file can be had only by one of those, that carrier
  is recorded as not obtained. Raw files are not committed and are deleted after the run;
  digest, byte count and address are recorded. The tables hold places, no person.

## What is known before the read

From the scout, read on pages: ITU K1 for six economies (Central African Rep. 4.54 in
2019, Mozambique 11.1 in 2024, India 86.3 in 2025, United States 94.9 in 2024, Norway 99.5
in 2024, Korea (Rep. of) 100 in 2025); Eurostat's EU household share 94.7 in 2025 (read)
and 93.1 in 2023 (from a search summary). Not known: each file's format, columns, flags
for estimates, year coverage, and each other value.

## Before the parse

For each file the record before the parse may read the header and the distinct labels of
the columns that are not numbers, and no value. It fixes the columns used, the name map to
ISO codes, which rows are groupings, and how an estimate or a break in series is flagged.

## What is built

`tables/connect0/`: `itu_connectivity.json`, `eurostat_household_internet.json`,
`acs_b28002_states.json`, each with its sidecar and licence block.

- A row is a place, a year, a kind (K1 to K4, with the publisher's own definition string
  and denominator), a value with its unit, `read: dataset`, the edition, and flags as the
  file gives them (estimated, break in series, unreliable). Each year the file gives from
  2014 on is written, so that a consumer chooses its year.
- ACS rows: for each state the shares of households with any subscription, with "Broadband
  such as cable, fiber optic or DSL" (K2), with a cellular data plan and no other
  subscription, and with no access; margins of error as given.
- Eurostat rows carry its denominator on their face: households with a member aged 16 to 74.
- No value is converted between kinds. No K4 figure is turned into a household share.

## Plants (synthetic files; the last re-run in each file's real layout)

- **P-KIND.** A file holding two indicators never writes one under the other's kind.
- **P-FLAG.** A value flagged estimated, or after a break, is written with the flag.
- **P-GROUP.** A grouping (world, a region, EU-27) is never written as a country.
- **P-DUP.** The same place, year and kind twice stops the run.
- **P-BLANK.** A blank, a colon or a letter where a number belongs writes no row.
- **P-SHARE.** A household share outside 0 to 100 stops the run; K4 may exceed 100 only
  with the value kept and flagged.
- **P-MOE.** An ACS share is the estimate over the state's total households, and the
  margin of error is carried, not dropped.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic file holds what its plant names.

## Stakes

A share is in percentage points. A place a file lacks counts against the stake it is in.

- **R1 (ITU read as its pages read), 6 values.** Each within `0.15` points for its year.
  Met: 5 or 6. *Later edition:* 3 or 4; the result names the movers. *Misread:* 2 or
  fewer; no ITU table is written.
- **R2 (Eurostat read as its page reads).** EU-27 2025 within `0.1` of 94.7. Met, or
  *misread* and no Eurostat table is written. The 2023 figure (93.1, from a summary) is
  printed beside the file with no verdict.
- **S1 (do two publishers agree on K1?).** For each country in both the ITU and Eurostat
  tables, in the latest year both give: the absolute difference. Met: at most `3` points
  in at least 80 % of those countries. *Kill:* in fewer than 50 %. Then a K1 map must not
  join the two publishers, and the tables say so. Between: neither.
- **S2 (does K1 overstate the fixed line?), US states and DC and Puerto Rico, 52.** In the
  ACS table, the share with any subscription less the share with a fixed line (K2). Met:
  over `5` points in at least 40 of 52. *Kill:* over 5 points in 12 or fewer. Then, in the
  one country where both are seen, "has internet" stands in for "has a line" and the
  scout's warning is withdrawn for it. Between: neither. The spread across states of the
  cellular-only share is printed.
- **S3 (ITU against the ACS for one country).** ITU's K1 for the United States in the ACS
  release's year against the ACS national share with any subscription or access without
  one. Printed with the difference; met if within `3` points, else not met. No kill: the
  two ask different questions.
- **Printed, no verdict.** Per kind: countries with a row, the year range, the count
  flagged estimated, the 10th and 90th percentiles in the latest year.

## What would make this vacuous

- R1 and R2 test the reading, not the truth of a survey.
- S2 is one country with one survey's wording; "over 5 points" says a gap exists there
  and nothing about any other country.
- S1's two publishers may share the national survey behind both; agreement is then one
  source read twice.
- None of K1 to K4 is "a host that is on". A full table here still leaves MESH-0's input
  unresolved, and the result must say so.

## Branches

For each carrier exactly one fires, in this order: **not obtained** (the file needs an
account, a key or accepted terms, or the licence page no longer reads as recorded, or for
the ACS no reuse sentence is found); **(e)** a plant fails for its layout or its kind's
column is absent; **(r)** its reconciliation reads misread (R1 for ITU, R2 for Eurostat;
the ACS has none and cannot fire this); **(a)** its table is written. A stake between two
carriers (S1, S3) is run only if both are written, and is otherwise recorded as not run.

---
witness: none (tables of priors; R1, R2, S1, S2 and S3 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the extractor that writes the table.
- M-BAR-FROM-THE-READ: the bars are stated here before a file is fetched; the six ITU values and the Eurostat figure were read on pages and are listed.
- M-PROVENANCE-OVERREACH: digest, edition and licence per file; one table per licence; kinds never converted.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
