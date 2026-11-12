# GRID-1A — an amendment to GRID-1: the same Ember file read again, for a row in each year from 2014 on, so that a consumer may choose its year. FROZEN

*Frozen 2026-11-12, committed alone, after one adversarial review (`GRID1A_REVIEW.md`)
whose named fixes are applied here. It replaces `GRID1A_AMENDMENT_DRAFT.md`, kept. The
refetch was approved by the lead on 2026-11-12 ("please do the fetches", after "refetch
Ember" was put to them by name as one of three). At this commit the file has not been
downloaded again.*

## Why

GRID-1 wrote, for each place, its latest year and the years 5 and 10 before the file's
latest (2019 and 2014), and each year a stake read. 111 of 224 places have no 2024 row: 95
whose latest is 2025, and 16 whose latest is 2023 or 2022, which the file may not be able
to fill. 99 have 2024 and no 2025. This session advised the twin's lead, in a message and
in no file, to "use 2024" for its map; the table cannot serve that for 111 places. The
advice will be withdrawn by a dated CORRECTION in `GRID1_RESULTS.md`, committed with the
result. A constant is a price in a regime: the year is the consumer's to choose, so the
table should hold each year.

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

- **New rows, same file** (the digest is the recorded one). For each place with a
  version-3 dataset row, one row for each year from 2014 to the place's latest year with
  an intensity in the file now read, for which version 3 has no dataset row. At most
  1,856 rows. Ids as GRID-1's (`ember-data-<slug>-<year>`), GRID-1's edition string, and
  the sidecar records that the stamp was read again.
- **New rows, another edition** (the digest differs). For each such place, one row for
  EVERY year from 2014 to its latest in the new file, years already tabled included, with
  ids `ember-data-<slug>-<year>-e<first 8 hex digits of the new digest>` and an edition
  string made of the server's stamp, or "no stamp; fetched 2026-11-12", always with those
  8 digits. At most 2,556 rows. The sidecar states: "a year series must be drawn within
  one edition; filter on edition before year", and lists both editions.
- **Fields.** As GRID-1's new rows, except: no new row carries `latest_for_place: true`,
  a `year_lag` or shares. A year later than a place's version-3 latest is an ordinary
  year row, listed in the sidecar under `later_year_in_later_edition`. Which row is
  "latest" changes only by a further amendment.
- **Existing rows.** The merge appends rows only; GRID-1's rule for adding fields to
  existing rows is not run again. No field of any of the 971 rows changes, none is
  removed, and none gains a field.
- Areas in the file with no version-3 dataset row are not written and are printed.

## Plants (synthetic files in the real layout)

- **P-YEARS.** A place with intensities in 2014 to 2024 except a blank 2017 gets a row for
  each untabled year but 2017, and none before 2014.
- **P-NODUP** (same file). A year that already has a dataset row gets no second row, and
  each new id collides with no existing id.
- **P-EDITION.** A synthetic file with another digest yields the full series with
  suffixed ids, the new edition string and no id collision.
- **P-KEEP4.** After the merge each version-3 row is present with the same set of keys
  and the same value in each. Run on the real merge too.
- **P-ONELATEST.** After the merge each place has exactly 1 row with
  `latest_for_place: true`, and it is the version-3 one. Run on the real merge too.
- **P-ABSENT.** A synthetic file missing one tabled place and one tabled year reads as R2
  not met on the same digest.
- **P-SAME.** On a synthetic file and a table built from it, R2 reads 0 differing; with
  the years shifted by one, it does not.
- **P-GRID1.** GRID-1's own 11 plants still pass through the same extractor.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic file holds what its plant names (a blank year,
a year already tabled, another digest, a missing place, shifted years).

## Stake

- **R2 (is it the same reading?).** Each of the 700 version-3 dataset rows is matched to
  the file on `place_as_published` and year. The number compared is printed and must be
  700; if it is not, branch (e). A row whose place or year has no intensity in the file
  counts against R2 and is listed as `absent_in_later_edition`.
  - *Same file* (the digest is the recorded one): met if all 700 are equal as parsed
    numbers (EXACT; the table stores the file's value unrounded). If any is not, the
    script no longer reads as GRID-1's did. Branch (m).
  - *Another edition* (the digest differs): the count that differ by more than `2 %` or
    `1 g`, whichever is larger, over the file's value as in GRID-1, is printed with the
    10 largest. Misread if any of these holds: more than 70 of the 700 differ by over
    `10 %` (absent rows counted); any of the 545 rows not flagged `small_system` moved by
    a factor of 2 (old over new outside 0.5 to 2); more than 15 of the 155 flagged rows
    moved by a factor of 2. Branch (r). Otherwise the differing rows are listed in the
    sidecar as `moved_in_later_edition` (place, year, old, new) and no row is changed.
- **Printed, no verdict.** New rows; places with a row in both 2019 and 2024; places still
  without 2024 and why; factor-of-2 moves in small systems.

## What would make this vacuous

- R2 on the same file tests this script against GRID-1's, not Ember against the world; it
  is a tautology unless the script changed, and (m) is then the only test of the new
  script on real data.
- On another edition nothing checks this script against GRID-1's reading but the plants;
  a one-year shift would read as "moved" were it not for P-SAME.
- A version-3 latest row that a later edition moves or withdraws still carries
  `latest_for_place: true`; the sidecar's `moved_in_later_edition` and
  `absent_in_later_edition` lists are where a consumer checks.
- Twelve years of one publisher's method are still one method. Trust stays B.
- A year series invites reading a trend; a revision of method between editions can sit
  inside it (United States 2023 moved 369 to 393 between editions). The sidecar says so.

## Branches

Exactly one fires, tested in this order.

- **(e)** A plant fails; or the fetch does not return the file; or the licence page no
  longer reads CC BY 4.0; or the extractor or merge stops or errors for any reason (no
  intensity column, another unit, a repeated row, an unknown area or grouping, an id
  collision, a missing column); or the number compared in R2 is not 700; or P-KEEP4 or
  P-ONELATEST fails on the real merge. Nothing is merged.
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
