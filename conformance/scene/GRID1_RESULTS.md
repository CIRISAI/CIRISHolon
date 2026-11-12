# GRID-1 — results, 2026-11-12. Branch (a): merged as version 3. The file agrees with Ember's own pages and with the secondhand rows that claimed to be Ember; life-cycle is NOT reliably the higher figure across sources (S2 killed); 209 countries and territories now have a row of one kind.

*Prereg `GRID1_PREREG.md`, review `GRID1_REVIEW.md`, the record before the parse
`grid1/BEFORE_THE_PARSE.md`, extractor and merge `grid1/grid1.py`, log `grid1/run1.log`,
numbers `grid1/results.json`, table and sidecar `tables/grid0/` (version 3). One run. The
file: Ember, "Yearly electricity data – Global (CSV)", stamped 3 Nov 2026, CC BY 4.0,
sha256 `0893…5688`; not committed, deleted after the run.*

## R1 (the file against Ember's own pages): met

20 of 21 rows are within 2 % or 1 g. The one outside: Latin America, 247 on the page and
252.1 in the file (2.0 %), under the file's name "Latin America and Caribbean"; the record
before the parse named that mapping as the first suspect, and the two may not be the same
set of countries. The seven rows of the earlier method, no verdict: the United States 2023
reads 369 by the earlier method and 392.9 in the file; the EU 244 and 236.0.

## S1a (were the secondhand rows usable?): met, 5 of 5

| | secondhand | file 2025 | file 2024 |
|---|---|---|---|
| Norway | 28 | 28.1 | 29.6 |
| Sweden | 35 | 35.4 | 34.9 |
| Canada | 191 | 191.6 | 184.8 |
| Mexico | 474 | 481.4 | 489.8 |
| Indonesia | 680 | 705.3 | 710.4 |

Five rows can say a practice failed; they did not. They cannot say it is safe. The three
rows for 2023 by the earlier method (no verdict): Poland 662 against 650.9, Sweden 41
against 38.4, France 56 against 53.3.

## S2 (life-cycle against direct): KILLED

| place, year | official, direct | file, life-cycle | |
|---|---|---|---|
| United States 2023 (reproduction) | 348 | 392.9 | higher |
| France 2025 (reproduction) | 19.6 | 41.2 | higher, by a factor of two |
| Canada 2023 | 100 | 177.5 | higher |
| China 2022 | 536.6 | 590.0 | higher |
| India 2023 (fiscal year) | 716 | 714.2 | level |
| Germany 2025 (per kWh consumed, a near kind) | 344 | 334.1 | **lower** |
| South Africa 2024 | 994 | 717.4 | **lower, by 277 g** |

The file is lower in two of the five places in doubt. Across sources the two kinds are not
ordered, and a legend must not say life-cycle is the higher. South Africa is the large
case: the official factor and Ember's differ by more than a quarter, in the direction a
difference of kind cannot explain. This run does not say which is right. Both rows stay.

## S3 (what the map gains), no verdict

- 700 rows added; 209 countries and territories and 15 groupings have one.
- Latest year by place: 2025 for 94, 2024 for 99, 2023 or 2022 for 16. By the frozen rule
  the file's latest year is 2024 (half the countries have it); 2025 exists for under half.
  A one-year map must filter on the year.
- Among the 99 whose latest is 2024, the 90th percentile is 5.8 times the 10th (681
  against 117). Lowest that are not small systems: Bhutan 24, Nepal 24, Albania 25.
  Highest: Turkmenistan 1,306, Bahrain 903, Brunei 893. The Turkmenistan figure is as the
  file gives it and was not checked against another source.
- 50 are small systems (under 1 TWh). The file marks no year as estimated, so no row can
  be marked provisional; a 2025 figure may be an estimate and the table cannot say.
- All 12 countries that had only rows rated C now have a row rated B.
- Added to version-2 rows by the frozen rule: `superseded_by` 8, `map_alternative` 32,
  `same_series_as` 28, `disagrees_with` 23. No version-2 value changed (P-KEEP on the
  real merge).

## What this does not license

- One publisher's method now covers most of the world. It is rated B, not A. GRID-0
  already holds IEA figures that disagree with it for 2025.
- An average rate times a change in load is an attribution, not emissions avoided.
- The kind is generation-based and life-cycle. It says nothing of imports or of the
  marginal plant.
