# TIES-0 — the record made before any index value is parsed (2026-11-11)

- **Plants.** All seven pass as coded (`plants.log`). Three are WEAKER as coded than as
  frozen, and that is said here before the read:
  - **P-FLOOR.** The freeze says the off-diagonal fit alone misses the own share by more
    than `0.05`. In this instrument's synthetic world it misses by a median of `0.016`: the
    synthetic areas are 30 km apart with a 4 km floor, close enough for off-diagonal pairs
    to see the floor. The coded plant checks only that the floor fitted on own indices
    recovers the planted one and that the stake is met. So the plant does NOT show the
    rung failing; whether it fails on the real file is read, not assumed.
  - **P-CENTROID.** The coded plant checks that the exponent fitted on centre distances is
    within `0.15` of the planted law. The artefact between two unit sizes is computed in
    the run itself (one law for individuals on a uniform plane at Germany's two unit
    sizes) and is printed; it is not separately planted.
  - **P-NOISE.** The coded plant reads a planted noise from two noisy copies of one set of
    numbers. The run reads noise the same way from regions that are a single unit at two
    levels.
- **Files.** The index's European file, the statistical office's region boundaries (1:10
  million), centre points and 2024 populations; digests in `raw_file_sha256.txt`. Looked
  at: header lines, row counts, one feature's properties. No index value.
- **Choices the freeze left open:**
  - An area's land area is computed from its boundary polygon on the sphere; its radius
    is that of the disc of equal area. Distances are great-circle between centre points.
  - The tilt is searched on a grid from −0.6 to 0.6 in steps of 0.02; its spread is over
    60 random halves of parent pairs.
  - The scale between levels is taken from single-unit regions where at least 3 pairs
    exist, otherwise from off-diagonal parent pairs at the fitted tilt.
  - S4's exponent and area terms are fitted on pairs at least 25 km apart and farther
    than the sum of the two radii. The floor is searched from 0.05 to 40 km.
  - S4's own shares are over ties within Germany, since the kernel is fitted on German
    pairs. S1's shares are over all of Europe.
  - S4 stands only if its outcome is the same at both values of the tilt.
  - S3 needs at least 5 areas in a cell.
  - East, for S5, is the five eastern Länder; Berlin is left out of the cross-border term.
