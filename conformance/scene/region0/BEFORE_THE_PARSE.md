# REGION-0, the record before the parse

2026-11-12. Written before `region0.py run` reads a coordinate. Freeze: `../REGION0_PREREG.md`.

- **The file.** `ne_50m_admin_1_states_provinces.zip`, 911,408 bytes, sha256
  `61f79e67…0bf6`, from `naciscdn.org`. Natural Earth's terms page was read again at the
  fetch and still says its data "are in the public domain".
- **Read:** the archive's 7 file names (a shapefile, its attribute table, a version
  file, a README); the attribute table's 121 column names; the distinct values of the
  country column (`iso_a2`: 294 shapes, 51 of them `US`); no coordinate and no other value.
- **Columns fixed:** country `iso_a2` (the value `US`); state name `name`; postal code
  `postal`. The table has a column named `fips`. Its format was not read; the script
  compares the DIGITS in it with its own code for each US shape, and a disagreement is
  branch (e). The column's name is not copied into any output.
- **51 US shapes:** 50 states and, by the count, the District of Columbia.
- **The version** is read from the archive's version file at the run.
- The plants pass, 8 of 8 (`plants.log`).
