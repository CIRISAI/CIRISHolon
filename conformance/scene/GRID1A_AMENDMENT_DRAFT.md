# GRID-1A — an amendment to GRID-1: the same Ember file read again, for a row in each year from 2014 on, so that a consumer may choose its year. DRAFT

*Draft 2026-11-12. Not frozen. The refetch was approved by the lead on 2026-11-12 ("please
do the fetches", after "refetch Ember" was put to them by name as one of three). At this
draft the file has not been downloaded again.*

## Why

GRID-1 wrote, for each place, its latest year and the years 5 and 10 before the file's
latest (2019 and 2014). 109 places have 2025 as their latest year and so no row for 2024;
113 have 2024 and no 2025. The twin's map defaults to 2019 and its consumer was told by
this campaign to "use 2024", which the table cannot serve for 109 places. That advice was
wrong and is withdrawn in `GRID1_RESULTS.md`'s correction at the run. A constant is a
price in a regime: the year is the consumer's to choose, so the table should hold each
year.

## Carrier

The carrier, the licence check at the fetch and the handling of the raw file are GRID-1's,
unchanged: the same address, the licence page read again (if it no longer reads CC BY 4.0,
nothing is merged), the file not committed and deleted after the run, its digest and byte
count recorded.

## What is known before the read

- Version 3: 971 rows, 700 of them read from the file stamped 2026-11-03, sha256
  `08932763…5688`, 16,120,810 bytes, 224 areas, years 1985 to 2025.
- The layout, the name map and the groupings fixed in `grid1/BEFORE_THE_PARSE.md`. They
  are used as they stand.
- Not known: whether the file at that address is still that edition, and each value of a
  year not already in the table.

## What is built

GRID-0's table, version 4.

- **New rows.** For each place with a version-3 dataset row, one row for each year from
  2014 to its latest year in which the file gives an intensity and for which version 3
  has no dataset row. Fields as GRID-1's new rows, with `latest_for_place: false`,
  `year_lag` empty and no shares (GRID-1 gave shares for the latest year alone; that rule
  stands). If the place's latest year in the file is later than its version-3 latest row,
  the new latest row is written with `latest_for_place: true` and its lag and shares, and
  the version-3 row is NOT changed (see P-KEEP); the sidecar names each such place.
- **Edition.** If the file's digest is the recorded one, new rows carry GRID-1's edition
  string. If not, they carry the new stamp (the server's last-modified date) and the new
  digest is recorded.
- **Existing rows.** No field of any of the 971 rows changes, none is removed, and none
  gains a field.

## Plants (synthetic files in the real layout)

- **P-YEARS.** A place with intensities in 2014 to 2024 except a blank 2017 gets a row for
  each year but 2017, and none before 2014.
- **P-NODUP.** A year that already has a dataset row in the table gets no second row, and
  each new id collides with no existing id.
- **P-KEEP4.** After the merge each of the version-3 rows is present and identical in
  each field. Run on the real merge too.
- **P-NEWLATEST.** A place whose synthetic file has a year after its table latest gets a
  new latest row; the old row is untouched.
- **P-GRID1.** GRID-1's own 11 plants still pass through the same extractor.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic file holds what its plant names (a blank year,
a year already tabled, a later year).

## Stake

- **R2 (is it the same reading?).** For each of the 700 version-3 dataset rows the file's
  value for that place and year is compared with the row's.
  - *Same file* (the digest is the recorded one): met if all 700 are equal to within
    `0.0005` g (the table's rounding). If any is not, the script no longer reads as
    GRID-1's did. Branch (m).
  - *Another edition* (the digest differs): the count that differ by more than `2 %` or
    `1 g`, whichever is larger, is printed with the 10 largest. If more than 70 of the 700
    differ by over `10 %`, or any by a factor of 2, the file is taken as misread. Branch
    (r). Otherwise the differing rows are listed in the sidecar as `moved_in_later_edition`
    (place, year, old, new) and no row is changed.
- **Printed, no verdict.** New rows; places with a row in both 2019 and 2024; places still
  without 2024 and why.

## What would make this vacuous

- R2 on the same file tests this script against GRID-1's, not Ember against the world.
- Twelve years of one publisher's method are still one method. Trust stays B.
- A year series invites reading a trend; a revision of method between editions can sit
  inside it (United States 2023 moved 369 to 393 between editions). The sidecar says so.

## Branches

Exactly one fires, tested in this order.

- **(e)** A plant fails; or the licence page no longer reads CC BY 4.0; or the file lacks
  the intensity column. Nothing is merged.
- **(m)** Otherwise, the digest is the recorded one and R2 is not met. Nothing is merged.
- **(r)** Otherwise, the digest differs and R2 reads misread. Nothing is merged.
- **(a)** Otherwise. Version 4 is written.

---
witness: none (a table of priors; R2 is measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the extractor and merge that write the table.
- M-BAR-FROM-THE-READ: R2's bars are stated here before the file is fetched again.
- M-PROVENANCE-OVERREACH: digest, stamp and licence recorded; trust stays B.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
