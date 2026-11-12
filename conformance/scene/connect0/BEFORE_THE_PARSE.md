# CONNECT-0, the record before the parse

2026-11-12. Written before `connect0.py run` reads a value. The freeze is
`../CONNECT0_PREREG.md`.

## What was obtained, and what was not

| unit | file | bytes | sha256 |
|---|---|---|---|
| Eurostat countries | `isoc_ci_in_h` as TSV from the dissemination interface | 114,735 | `4fed29fe…3538` |
| Eurostat regions | `isoc_r_iacc_h` as TSV | 52,448 | `2f507eb4…1b3e` |
| ACS 2024 1-year | `acsdt1y2024-b28002.dat`, table-based summary file | 1,150,958 | `ac6ff542…e8e2` |
| ITU K1, K3, K4 | — | — | — |

- **ITU: not obtained, for a reason the freeze did not list.** ITU's DataHub answers this
  machine's requests with HTTP 403, its licence page included, and no browser was
  connected to this session. So the licence sentence could not be read again and no file
  could be had. The freeze's "not obtained" names an account, a key, a form, terms, or a
  licence sentence gone from its page; a refusal of the client is none of these. It is
  recorded here as not obtained all the same, and said as a departure. P-KIND and P-K4
  act on ITU files alone and are not run. S1 and S3 are not run. The three units can be
  taken up again, under this freeze, when a browser can read the pages.
- Licence sentences read again at the fetch. Eurostat: "Reuse of statistical data,
  metadata, publications, and other dissemination tools published on this website for
  commercial or non-commercial purposes is authorised provided the source is
  acknowledged." and "A disclaimer regarding the non-responsibility of Eurostat shall be
  included." Census citation page: `Author/s, "Table Name," Full Product Name, Vintage,
  <URL>, accessed on date.`
- Raw files are in the session's scratch folder and are deleted after the run.

## What was read, and what was not

Read: each file's header; the distinct labels of the key columns; the distinct flag
letters; the ACS file's column names and the prefixes of its area codes; the two Eurostat
datasets' titles and the first one's description, from the structure service. No value
was read.

## Fixed from the labels

**Eurostat countries (`isoc_ci_in_h`, "Households - level of internet access").**
- Key: `freq,unit,hhtyp,geo`. The file has one unit (`PC_HH`) and no indicator column:
  the dataset IS the indicator. The freeze's "household-internet-access indicator" label
  is therefore taken from the dataset's title and description ("Percentage of households
  who have internet access at home. All forms of internet use are included... At least
  one member in the household should be aged 16 to 74 years."). This reading is mine;
  the freeze said a label not present is branch (e), and it meant a column.
- Selection: `unit=PC_HH`, `hhtyp=TOTAL`, `geo=EU27_2020` for R2. Neighbours printed for
  R2: `geo` of `EA`, `EU27_2007`, `EU28`; and under `EU27_2020` each other `hhtyp`.
- Groupings, never written as countries: `EA`, `EU15`, `EU25`, `EU27_2007`, `EU27_2020`,
  `EU28`. Countries: the 38 two-letter codes, with `EL` Greece, `UK` United Kingdom, `XK`
  Kosovo. The 27 members for R2 are the EU's 27 since 2020.
- Flags: `b` break in series (starts a new `series_segment`), `e` estimated, `u` low
  reliability, `:` not available (no row).

**Eurostat regions (`isoc_r_iacc_h`, "Households with access to the internet at home").**
- Key: `freq,unit,geo`; one unit (`PC_HH`). A code of 2 letters is a national row (used
  for R2b alone); 3, 4 or 5 characters is NUTS level 1, 2 or 3. Flags `b`, `u`, `:`.
- R2b's year: for each country in both tables, the latest year from 2014 in which both
  give a value.

**ACS (`B28002`, 2024 1-year).**
- Columns `B28002_E001` to `E013` with margins `M001` to `M013`. State rows are those
  whose `GEO_ID` begins `0400000US`; there must be 52 (50 states, the District of
  Columbia, Puerto Rico) or the run stops.
- K1 = (E002 + E012) / E001; any subscription = E002 / E001; K2 = E007 / E001; cellular
  only = E006 / E001; satellite = E009 / E001; no access = E013 / E001. Shares are exact
  fractions, compared as such, and written to 2 decimals.
- A margin that is a negative sentinel is written as empty with the sentinel kept.
- Percentiles for S2b over the 52 gaps: nearest rank, the 6th and the 47th in order.

**Rows.** From 2014 on for Eurostat. Regional rows carry `nuts_level`; the NUTS edition
is not stated in the file and is written as "not stated by the file".
