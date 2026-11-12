# GRID-1, the record before the parse

2026-11-12. Written before `grid1.py run` reads a value. The freeze is `../GRID1_PREREG.md`.

## The file

- `release_generation_yearly_global.csv` from Ember's public downloads, 16,120,810 bytes,
  sha256 `08932763076fa781634371dd1b75d8156f76592710c7e984836ed56a663d5688`, fetched
  2026-11-12 after the freeze. Not committed; deleted after the run.
- The licence page was read again at the fetch and still says "Creative Commons
  Attribution Licence (CC-BY-4.0)". It asks for credit ("as long as you credit us") and
  gives no form of words for it; none is invented. The data page names multi-country and
  national sources Ember collects from; it states no exception to the licence for them.

## What was read of it, and what was not

Read: the header; the distinct labels of area, type of area, code, electricity source and
the membership columns; the span of years (1985 to 2025); 104,256 rows, 224 areas, 15 of
them groupings. One slip, said here: the first pass also counted the distinct values of
the numeric columns. It printed the counts and no value, and no value was looked at.

## Fixed from the labels

- **Layout.** One row per area, year and electricity source. The intensity is the column
  `Emissions intensity (gCO2e/kWh)` on the row whose source is `Total generation`. Shares
  are `Share of generation (%)` on the rows not marked as aggregated sources (solar, wind,
  hydro, bioenergy, other renewables, nuclear, coal, gas, other fossil).
- **Estimated years.** The file has no column marking one. `provisional` is left empty for
  each row, and the result says so. P-EST passes on a synthetic file that has the column.
- **Names.** `EU` is written as `EU-27`; `Latin America and Caribbean` as `Latin America`
  (GRID-0's name for Ember's own page row; if R1 reads those two apart, that mapping is
  the first suspect). All other areas keep the file's name, which already equals GRID-0's
  for its 23 countries, Asia and World. GRID-0's `European Union` (IEA rows), `EU plus
  Switzerland, Norway and the UK` and `Southeast Asia` have no counterpart written.
- **Groupings.** Geographic: World, Africa, Asia, Europe, Latin America and Caribbean,
  Middle East, MENA, North America, Oceania. Political: EU, AU, ASEAN. Economic: G20, G7,
  OECD.
- **Territories.** The file's type is "Country or economy" and includes Puerto Rico, Hong
  Kong, Macao, Taiwan and Kosovo under its own names. They are written as the file has
  them, with the name as published.
- **"Differ"** for `disagrees_with`: by more than 0.5 g (the page rows are whole numbers).
- **S1a's second year** is 2024, printed. The row for it is written so the evidence is in
  the table.

## Plants (`plants.log`)

P-UNIT, P-AGG, P-GAP, P-EST, P-SHARE, P-NAME, P-DUP, P-ID, P-UNITCHK, P-KEEP and P-LONG
pass. Every synthetic file is in the layout above.
