# HOUSEHOLD-0 — the record made before any survey table is parsed (2026-11-11)

- **The generator under test.** `twin_household_hist.json` is a byte copy of
  `docs/geo/household_hist.json` from the twin's tree, commit `b939214` on branch
  `twin/hhsize` (library code `943ab2d`), sha256
  `438568abd0258d56bad0d2e286c98e3e03d855848969465af79914971bb0a418`, recomputed here.
  1,000,020 households in 16,667 leaves of 60 households and 150 people. Final counts at
  sizes 1 to 7: 182,423 / 321,136 / 354,758 / 106,254 / 27,337 / 7,404 / 708. Mean exactly
  `5/2`; one-person share `0.1824`. The twin states that size is conditioned on nothing but
  the leaf's seed, people and dwelling count: one histogram.
- **The twin's own caveat.** The file is for leaves at exactly 2.5 people per household.
  Real leaves inherit people and households rounded separately, so the distribution on the
  page can differ. Not scored here.
- **The port.** This campaign's implementation gives one-person share `0.1815` and shares
  within `0.002` of the twin's file at each size (largest difference `0.0014`, at size 2).
  The reviewer's two observations (the mean moves to exactly 2.5; households of 7 appear)
  are properties of the twin's code and are in its own file under `drawn` and `final`.
- **Plants.** All eleven pass on synthetic carriers (`plants.log`).
- **Files on this machine, unparsed.** Digests in `raw_file_sha256.txt`. Only directory
  listings, the geography file's header line and file sizes have been looked at.
- **One departure from the freeze, stated before the read.** Block-group land area is taken
  from the Bureau's 2024 cartographic boundary file (`cb_2024_us_bg_500k`, 97 MB,
  attribute `ALAND`), not the TIGER/Line files the freeze names. The gazetteer carries no
  block groups, and the per-state TIGER/Line files are several times the approved download.
  The attribute is the same quantity from the same source. Block groups absent from the
  boundary file are excluded and counted.
- **A second departure.** The freeze's P-TOP says the distance "does not move"; the plant
  checks that W1 of the carrier against itself is zero with the band placed at 8, which is
  true of any carrier. The meaningful check is that the residual moves (9.0 to 9.55) and
  enters no verdict; the code has no path from the residual to a verdict.
