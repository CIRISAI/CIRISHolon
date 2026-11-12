# GRID-1A — results (2026-11-12)

Freeze: `GRID1A_AMENDMENT.md` (committed alone at `1cdfdb8`), review `GRID1A_REVIEW.md`.
Script `grid1a/grid1a.py`, log `grid1a/run1.log`, numbers `grid1a/results.json`.

- **Branch (a): version 4 is written.** 8 of 8 plants pass, GRID-1's own 11 among them.
- **The file is the same edition.** 16,120,810 bytes, sha256 `08932763…5688`, the
  server's stamp 3 Nov 2026; Ember's licence page read again and still CC BY 4.0. The raw
  file was deleted after the run.
- **R2 met:** 700 of 700 version-3 rows compared, each equal exactly. As the freeze says,
  on the same file this tests the script against GRID-1's and nothing more.
- **1,856 rows added,** one for each year from 2014 to each place's latest that version 3
  lacked. The table has 2,827 rows (B 2,694, A 93, C 40). No version-3 row changed
  (P-KEEP4 on the real merge), and each place still has the 1 latest row it had.
- **208 of 224 places now have both a 2019 and a 2024 row.** 16 have no 2024 because the
  file gives them none: Central African Republic, Comoros, Dominica, Falkland Islands,
  Faroe Islands, French Guiana, Guadeloupe, Lesotho, Martinique, Reunion, Saint Pierre
  and Miquelon, Sao Tome and Principe, Ukraine, Vanuatu, and the British and US Virgin
  Islands.

## What this does not license

- A year series here is one publisher's method in one edition. It is rated B.
- A trend read from it is Ember's estimate of a trend. A later edition can move each
  year; the sidecar says to filter on edition before year.
- Nothing about the kind changed: generation-based, life-cycle, no imports, not marginal.
