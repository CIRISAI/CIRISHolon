# GRID-1 draft, adversarial review — verdict: FREEZE WITH NAMED FIXES

Reviewed 2026-11-12 against `GRID1_PREREG_DRAFT.md`, `tables/grid0/grid_table.json` (271 rows,
counted by script), `tables/grid0/sidecar.json` and `SCOUT_GRID.md`. No dataset was fetched and
the network was not used. Statements about what the file *could* hold are marked as such; none is
a fact about the file. Synthetic checks (rounding, branch enumeration) ran in a scratch directory.

Fourteen findings. F1 to F5 change what the run would conclude or write and should be fixed
before the freeze; F6 to F14 are wording or added plants.

## Findings

**F1. Branch (a) merges a misread file; branch (e) can fire on a usable one.**
Evidence: (a) merges "whatever R1, S1 and S2 read". R1 is the only check of the extractor on
real data; the plants are synthetic and pass whatever column or area the extractor picks in the
real file. A wrong variable (say a direct-emissions series, or tonnes in place of g/kWh) would
fail R1 on most of 18 rows and still be merged at trust B. R1's kill sentence also presupposes
the cause ("the file is a later edition"). Separately, (e) says "no column for carbon intensity":
if the file is long (one row per area, year and variable, the intensity being a value of a
variable column — possible, not known), (e) as written fires on a good file. Enumeration: 600
outcome combinations, exactly one branch fires in each, but 100 are "(e) on a usable long file"
and 20 are "(a) on a gross R1 failure".
Fix: (i) reword (e) to "no carbon-intensity series, as a column or as a variable value";
(ii) add branch **(r)**: if 5 or more of R1's rows differ by more than 10 %, or any by a factor
of 2, nothing is merged and the reading is reported as an instrument fault; (iii) R1's
"later edition" sentence applies only when the differences are over 2 % and under 10 %.

**F2. S1 is confounded by method vintage and by an ambiguous year label; "5 of 8" has no verdict.**
Evidence: the eight rows are as stated (gc-ember2025-{norway, sweden, canada, mexico, indonesia};
statista-ember-2023-{poland, sweden, france}). But the three 2023 rows carry kind "said to be
Ember ... (earlier method)", and GRID-0 already records that the method change moved the United
States 2023 from 369 to 393 (6.1 %) and that the EU's 244 to 213 is "partly method". A current
file will hold 2023 on the current method, so those three can miss by method alone, and the kill
sentence ("said to be is not a stand-in for the publisher") would be the wrong reading. The five
2025 rows carry "year label 'Ember 2025' is ambiguous (edition or data year)"; "same place and
year" picks 2025 silently. If the three 2023 rows miss, the best case is 5 of 8, which is
neither the stake (6 or more) nor the kill (4 or fewer).
Fix: split S1. **S1a** (five gc rows): stake on data year 2025, with 2024 printed beside; 4 of 5
within tolerance on 2025 = met; 2 or fewer on both years = killed; met on 2024 only = "the label
was the edition", not a kill. **S1b** (three Statista rows): printed, no verdict, labelled a
method-vintage reading. State what 3 of 5 means ("not decided"). Drop "worth less than their
rating says": C is the lowest rating, so that sentence has no consequence; say instead what the
kill changes (such rows are not used to fill a map even as placeholders).

**F3. S2: the list is selective, two places are known before the read, Germany is not the stated kind, and years are unnamed.**
Evidence from the table:
- United States 2023: eGRID 348.0 against Ember's own revised 393 (row `ember-2023-united-states-rev`),
  already in GRID-0. Known before the read.
- France: RTE 2025 direct 19.6; RTE's own life-cycle is 29.0 and the "said to be Ember" 2023
  figure is 56. Not seriously in doubt.
- Germany: every UBA row is "direct CO2 of German generation divided by electricity CONSUMED".
  It is not generation-based per kWh generated; no year is named and seven are available.
- Canada 2023: 100, rounded to tens by ECCC and preliminary; utility sector only.
- China 2022: 536.6, a kind the scout could not define ("the page gives no definitions").
- **India is left out** though `in-cea-fy2022-23-avg-incl-res` (716, official, direct, generation
  average) is in the table, and its note already says "Compare Ember 2023 = 713". The draft's own
  vacuity section cites the IEA/Ember sign flip for China and India. The one place where GRID-0
  already shows direct above life-cycle is the place excluded.
- No margin: a 1 g difference counts as ordered; a place lower in one year and higher in another
  is undefined; "1 lower" has no verdict.
Fix: name one row id per place (`egrid2023-us-total`, `de-uba-2023-co2-strommix` with its
denominator stated, `fr-rte-2025-direct`, `ca-nir-2023-canada-gen`, `za-dffe-2024-dggef`,
`cn-mee-2022-national`, and add `in-cea-fy2022-23-avg-incl-res` against the file's 2022 and 2023,
both printed). Say the United States and France are reproduction, not stake. Define "lower" as
lower by more than 2 % or 3 g, whichever is larger; inside that band = "not ordered". Kill at two
or more places lower among the five that are in doubt; one lower = the legend may say "usually".

**F4. The merge rule marks nothing or everything, and leaves duplicate B rows.**
Evidence: "a row rated C for a place and kind that a new row covers". The 40 C rows are 17
lowcarbonpower (consumption mix, generic factors, year field 2026 for 15 of them), 5 gc, 3
Statista, 3 SDES/IEA direct, 3 EEA direct, and 9 official summaries (India 2, France 2, UK 2,
eGRID2014, China 2023, Japan). None has the new kind string; by exact kind match zero rows are
marked, by a loose reading the lowcarbonpower rows are marked as superseded by a row of another
kind and year. Unmarked, a map that picks "latest year per country" will prefer the 2026
lowcarbonpower row over the new 2025 row. Whether "covers" includes the year is not said: the
Statista rows are 2023 and the draft writes only the latest year and the years five and ten
before, so no row would exist to name. Also the new rows for World, China, United States, EU-27,
India, Russia, Japan, Brazil, Asia and Latin America 2025 (and World 2015) duplicate existing B
rows of the same place, year and kind; the rule is silent on them.
Fix, exact wording:
> A version-2 row gets `superseded_by: <new row id>` only if all three hold: its trust is C; its
> kind says it is Ember's generation life-cycle figure (the eight "said to be Ember" rows); a new
> row exists for the same place and the same calendar year. Any other version-2 row for a place
> that has a new row, where the kind or the year differs (the 17 lowcarbonpower rows, the
> search-summary rows), is not superseded; it gets `map_alternative: <id of the new latest-year
> row for that place>` and keeps its kind and trust. A version-2 Ember row read on a page, for a
> place and year the file also gives, gets `same_series_as: <new row id>` and, when the values
> differ, both rows get `disagrees_with` and the difference is stated; neither is removed. No row
> rated A gets any of these fields. The sidecar states: `superseded_by` means same place, year
> and claimed kind, read at source; `map_alternative` means a different kind or year, offered as
> the row to draw on a one-kind map; neither means the old value was wrong.
Also write a new row for every place and year a stake reads (2024 for R1, 2023 for S1b, the S2
years), so the evidence is in the table and `superseded_by` has a row to name.

**F5. Latest year: per-country latest years are mixed on one map, provisional years are not marked, and "five and ten before" is ambiguous.**
Evidence: P-GAP gives each country its own latest year; nothing flags that country A is 2025 and
country B is 2023. S3's 90th/10th percentile ratio would then mix years. A latest year could be
an estimate or built from partial data for some countries (possible, not known); the draft has
no field for it. "Five and ten before" does not say before the file's latest year or the
country's own. If the latest is 2025, "five before" is 2020, an unusual year for demand; a rate
drawn from 2020 to 2025 inherits that.
Fix: define `file_latest_year` (the modal latest year with an intensity). Each latest row carries
`year_lag = file_latest_year - year`; the five- and ten-year rows are taken from
`file_latest_year`, not per country. S3's ratio is computed on rows with `year_lag = 0` only and
the count of lagging countries is printed. If the file carries any marker of estimate, provisional
or partial year (to be looked for when the header is read), the row gets `provisional: true` and
the marker's text. Add a note that the five-year row is 2020 and no rate is to be drawn from it
without saying so. Add plant **P-EST** (a flagged year comes back flagged) and extend P-GAP to
blank, zero, and non-numeric intensity.

**F6. Counts.** R1's 18 rows: correct (10 for 2025, 8 for 2024). S1's eight: correct. "28 read on
Ember's pages": correct. **S3's "14 countries rated C" does not reproduce**: 12 countries have
only C rows (Australia, Indonesia, Italy, Kenya, Mexico, Netherlands, Nigeria, Norway, Poland,
South Korea, Spain, Sweden), 21 have at least one C row, 17 have a lowcarbonpower row.
Fix: state which count is meant and list the countries by name.

**F7. "Not known: any value in the file" overstates; three things are known.**
Evidence: (i) GRID-0 holds 21 current-method values from the same series, and notes that the 2026
edition already revised 2024 for the EU (213 to 211, 0.95 %) and India (708 to 705, 0.43 %), so
R1's 2024 rows are compared with values known to have been revised, inside the bar. (ii) The
scout's dataset table says "200+ geographies, 2000 to 2025", so the latest year and the rough
count S3 prints are known. (iii) The file's stamp (3 Nov 2026) is later than the review
(April 2026), so "the file is a later edition" is known before the read, not a finding.
R1 also omits three current-method rows it could test: `ember-2015-world` (533, exactly the
ten-year row), `ember-2023-united-states-rev` (393), `ember-2007-united-states` (598).
Fix: list these under "Known before the read"; extend R1 to 21 rows; print the file against the
seven earlier-method 2023 rows with no verdict (the size of the method move); add to
M-BAR-FROM-THE-READ that the bars were set knowing the 0.4 to 0.95 % revisions.

**F8. R1 has undefined cells.** "More than three differ" is the kill; one to three differing has
no reading; a place absent from the file (the file might say "Latin America and Caribbean", or
have no "Asia") has no rule; the denominator of "within 2 %" is not said.
Fix: relative difference = |file − page| / file. Zero differ = met. One to three = met with the
rows named. A place not found under the frozen name map counts as differing. The same rule for
S1 and S2: an absent place or year counts against the stake, never out of the denominator.

**F9. Bars against rounding (item 9).** Table values for these rows are integers. At 5 %:
Norway 28 allows the file 27 to 29 (1.4 g, of which rounding one side by 0.5 g is 36 %);
Sweden 35 allows 34 to 36; Sweden 41 allows 40 to 43; France 56 allows 54 to 58. Four of S1's
eight rows turn on 1 to 3 g, and hydro and nuclear grids move more than that between adjacent
years. R1's tightest row is Brazil 103 (2 % = 2.06 g); adequate.
Fix: S1 tolerance "within 5 % or 2 g, whichever is larger", stated before the read; the same
floor for R1 (2 % or 1 g). Compare the file's value unrounded and print both.

**F10. Names, codes and aggregates; plants missing.**
Evidence: GRID-0's place strings are "United States", "EU-27", "European Union" (IEA rows),
"Russia", "South Korea", "Latin America", "Asia", "World". The file could say "United States of
America", "EU", "Russian Federation", "Republic of Korea", "Latin America and Caribbean". Unmapped,
the merge makes a second place for the same country and R1 reads "absent". The file could also
carry overlapping groupings (OECD, G20, G7, ASEAN, continents) and territories marked as countries
(a Puerto Rico row would sit beside eGRID's PRMS subregion). "The world and each region it gives"
would write all of them with `place_kind: region`. "Fixes [the columns] after reading the header
line and nothing else" is not enough to build a name map or to learn how regions are marked.
Fix: the record before the parse may read the header and the distinct values of the non-numeric
columns (area, area type, code, variable, unit), no value. It freezes a name map for GRID-0's 30
existing non-subregion places. New rows carry `iso3` (from the file if it has one, else null),
`place_as_published`, and for groupings `place_kind: region` plus `grouping: geographic |
political | economic`, with only geographic regions and EU-27 offered to the map. Add plants:
**P-NAME** (a synthetic "United States of America" and "EU" land on "United States" and "EU-27";
an area that matches no map entry and no ISO code fails loudly), **P-DUP** (the same area and
year twice fails, never last-wins), **P-ID** (new ids are unique and collide with none of the
271), **P-UNITCHK** (an intensity unit other than g/kWh is converted with `original_value` kept,
or fails; GRID-0's convention is "value always grams per kWh"), **P-LONG** (plants are re-run on
a synthetic file in the real layout after the header is read; the draft's order, plants "before
the read" but columns named after it, cannot hold otherwise).

**F11. Kind and unit strings fork the existing kind.** The draft's kind, "generation average,
life-cycle (Ember's method)", is not the string the 21 existing rows carry ("generation average,
life-cycle (Ember: generation x life-cycle factors incl. upstream methane, supply chain,
manufacturing; all gases, 100-year); production mix, no imports; location-based"). A consumer
grouping by kind sees two kinds. Existing Ember rows already carry two unit strings
("gCO2e/kWh" 19, "gCO2/kWh (as printed)" 9); "unit as the file states it" may add a third.
`read` has no value for a dataset (conventions list page, mirror, summary).
Fix: reuse the existing kind string exactly; add `edition` (file stamp and digest) as a separate
field; unit "gCO2e/kWh" with the file's own unit text in `original_unit`; add `read: dataset`
to the conventions.

**F12. P-KEEP cannot pass as written, and P-SHARE's bar is loose.** The merge adds
`superseded_by` to version-2 rows, so "every row present with its value unchanged" needs a
definition. Fix: every version-2 row is present and every version-2 field is byte-identical; only
the named new fields are added; run on the real merge as well as the synthetic. P-SHARE: say
whether shares are of generation, whether the file's own subtotals (fossil, clean, renewables —
possible) are excluded before summing, since including them would sum to 200 or 300 and flag
every country.

**F13. Trust B for every new row.** Defensible under the stated convention (B = read at source
but modelled or estimated; C = secondhand), and consistent with the 28 page rows. Two gaps:
GRID-0's own precedent lowers a row one grade for an estimated year (UBA 2025 is B where final
years are A), and tiny systems may rest on imputed data. C would misstate them as secondhand.
Fix: keep B; carry `provisional` (F5) and a `small_system` flag below a stated generation
threshold fixed before the read (for example under 1 TWh); `trust_reason` names the flag; S3
reports "covered at B" split into unflagged and flagged. State in the sidecar that the new rows
are a better read than the page rows (file against a model's extraction of a page) though both
are B.

**F14. Licence and attribution.** "Each row carries the credit" is not enough for a derived
table a third party republishes verbatim. CC BY 4.0 (section 3(a)) asks in general for:
identification of the creator in the manner the licensor reasonably requests; a copyright notice
and a notice referring to the licence and to the disclaimer of warranties, where supplied; a URI
to the material where practicable; an indication that the material was modified; and the licence
name with its URI. It also bars implying endorsement. The existing sidecar says "values are
facts quoted with their sources"; that posture does not describe taking several hundred rows
from one database, and the 4.0 licence covers database rights too.
Fix: do not invent Ember's wording. When the licence page is re-read at the header step, record
any attribution form Ember asks for, verbatim, and whether the data page carves out third-party
inputs. The sidecar gets an `attribution` block: creator and title as published; the file URL
and the data page URL; edition stamp and digest; "CC BY 4.0, https://creativecommons.org/licenses/by/4.0/";
a changes statement ("a selection of years and areas; place names reconciled to this table's;
values not altered; shares as given"); "no endorsement by the publisher is implied"; and an
instruction to consumers that the credit and licence link must be shown wherever these rows are
drawn or republished. Each new row carries `licence: "CC BY 4.0"`, `source` with the title and
edition, and `url`. Reword the sidecar's `licences` line so that it excepts these rows from
"facts quoted". Add a branch condition: if the licence page no longer reads CC BY 4.0 at fetch
time, nothing is merged.

## Forecasts (mine, made without the file; from GRID-0 and recollection, not facts)

- **R1: met.** 16 to 18 of 18 within 2 %. Most likely misses: EU-27 2024 and India 2024 are
  already known to have moved (inside the bar); a further mid-year revision could push Brazil or
  Latin America over. A name-map miss for "Asia" or "Latin America" is the likelier failure than a
  value. Chance of more than three over 2 %: about 10 %.
- **S1: as drafted, lands on 4 to 6 of 8 — most likely 5, the undefined cell.** By part: gc 2025
  rows 3 or 4 of 5 (Canada 191 is the one I expect to miss; Norway and Sweden turn on 1 to 2 g);
  Statista 2023 rows 1 or 2 of 3, missing by method vintage. Chance of the draft's kill (4 or
  fewer): about 35 %.
- **S2: met, with South Africa the likely exception.** I expect the file below DFFE's 994 for
  South Africa (a factor-based figure against measured plant), the others above. If India is
  added (F3) I expect it within the tie band or lower, and the kill (two or more lower) becomes
  about even. Germany is within 5 % either way once the denominator is counted.
