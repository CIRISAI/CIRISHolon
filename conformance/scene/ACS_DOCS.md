# ACS 5-year household tables: what the publisher's documentation says

*Provenance, added when this report entered the repository (2026-11-11): written by a
documentation-reading subagent from the publisher's variable lists, technical documentation,
directory listings and header requests. No data file was downloaded and no data-returning call
was made. Formulas it transcribed from images in the handbook are to be checked before a
freeze. Nothing has been fetched; the dataset is not approved.*

*Read 2026-11-11. Documentation only: table shells and variable lists from the API's metadata
endpoints, technical PDFs, policy pages, directory listings and HEAD requests. No data file was
downloaded and no data-returning API call was made. Every statement below is from the page cited
next to it unless marked **[my arithmetic]**, **[my reading]** or **not confirmed**.*

## 0. The three things to know first

1. **Device COUNTS do not exist. Only presence.** The question is a Yes/No checkbox per device
   type, per household. No ACS table, at any geography, gives a number of devices.
2. **Terms.** No licence is attached to the files. The API has a terms-of-service agreement with a
   required notice, a no-re-identification clause and a no-misattribution clause; it does not
   restrict redistribution of derived tables. A sentence on census.gov stating "public domain" was
   **not confirmed** (searched, not found); the citation policy asks for the Bureau to be cited as
   source of the original data only.
3. **Most serious trap.** The device tables are presence indicators at near-saturation
   (smartphone in 90 % of households nationally in 2021), published for population-sized areas
   whose median coefficient of variation for a 10 %-share characteristic is about 0.25 to 0.35.
   A design that turns them into "devices per household by 1 km density class" invents the count,
   invents the tile, and reads mostly sampling noise in the one category that still varies.

---

## A. Tables, confirmed for the most recent 5-year release

**Release.** The most recent 5-year release that exists is **2020-2024** ("ACS 5-year 2024").
The accuracy document: "The data contained in these data products are based on the American
Community Survey (ACS) samples interviewed from January 1, 2020 through December 31, 2024."
<https://www2.census.gov/programs-surveys/acs/tech_docs/accuracy/MultiyearACSAccuracyofData2024.pdf>.
The summary-file data directory is dated 2026-01-29
(<https://www2.census.gov/programs-surveys/acs/summary_file/2024/table-based-SF/>). The API has
no 2025 5-year dataset (`https://api.census.gov/data/2025/acs/acs5/groups.json` returns 404);
the 2025 **1-year** estimates are out (banner on census.gov ACS pages, October 2026). The date
of the 2021-2025 5-year release: **not confirmed**.

Sources for ids, labels and universes: the API variable lists
`https://api.census.gov/data/2024/acs/acs5/groups/<TABLE>.json` (metadata only) and the Bureau's
table list <https://www2.census.gov/programs-surveys/acs/tech_docs/table_shells/table_lists/2024_DataProductList.xlsx>.

### A1. Devices and internet (all confirmed; "believed" ids were right except B28004/B28005 contents)

| id | title | universe | cells | what it holds |
|---|---|---|---|---|
| **B28001** | Types of Computers in Household | Households | 11 | see below |
| **B28002** | Presence and Types of Internet Subscriptions in Household | Households | 13 | see below |
| **B28003** | Presence of a Computer and Type of Internet Subscription in Household | Households | 6 | has a computer × {dial-up alone, broadband, no subscription}; no computer |
| **B28004** | Household Income in the Last 12 Months by Presence and Type of Internet Subscription | Households | 25 | 6 income bands × {dial-up alone, broadband, none}. **No computer type.** |
| **B28005** | Age by Presence of a Computer and Types of Internet Subscription | **Population in households** | 19 | 3 age bands × has computer × subscription. Persons, not households. |
| B28006, B28007 | Educational attainment / labour force status by the same | household population 25+ / civilian household population 16+ | | persons |
| **B28008** | Presence of a Computer and Type of Internet Subscription in Household | **Population in households** | 10 | the only table splitting "fixed broadband with / without a cellular plan" and "cellular data plan alone or with dial-up", in PERSONS |
| B28009A-I | the same by race / Hispanic origin | population in households who are ... | | persons |
| **B28010** | Computers in Household | Households | 7 | collapsed B28001 |
| **B28011** | Internet Subscriptions in Household | Households | 8 | collapsed B28002 (no cellular line) |
| B28012 | Age and Enrollment Status by Computer Ownership and Internet Subscription Status | Population in households | | persons |
| **S2801** | Types of Computers and Internet Subscriptions (subject table) | "Total households" | 31 lines | B28001 + B28002 + income (3 bands) × subscription, with percentages |

**There is no ACS table crossing computer type with household size, with tenure, or with units
in structure.** The B28 group in the 2024 5-year group list is exactly B28001-B28012 and
B28009A-I (<https://api.census.gov/data/2024/acs/acs5/groups.json>). The only crossings are
computer-presence (any) × subscription type, and subscription × income / age / education /
labour force / race.

**B28001 lines, verbatim:** Total; Has one or more types of computing devices; Desktop or laptop;
Desktop or laptop with no other type of computing device; Smartphone; Smartphone with no other
type of computing device; Tablet or other portable wireless computer; Tablet ... with no other
type of computing device; Other computer; Other computer with no other type of computing device;
No Computer.

So B28001 gives four overlapping "has at least one" marginals, four "only this type" cells, and
"none". It is NOT the 16-cell joint of the four Yes/No answers: households with two or three
types are not split by which types.

**Presence, not count. The question, from the 2024 subject definitions:** "The computer use
question (Question 9) asked if anyone in the household owned or used a computer and included
four response categories for a desktop or laptop, a smartphone, a tablet or other portable
wireless computer, and some other type of computer. Respondents selected a checkbox for 'Yes' or
'No' for each response category. Respondents could select all categories that applied."
<https://www2.census.gov/programs-surveys/acs/tech_docs/subject_definitions/2024_ACSSubjectDefinitions.pdf>
(section "Computer and Internet Use"). Note "owned **or used**".

"Other computer" is broad: the Bureau's 2021 report says "The ACS considers all desktops,
laptops, tablets, and smartphones as computers, along with selected computing technologies (such
as smart home devices) and single board computers (such as Raspberry Pi and Arduino boards)."
<https://www2.census.gov/library/publications/2024/demo/acs-56.pdf>

The microdata are the same: PUMS variables LAPTOP, SMARTPHONE, TABLET, COMPOTHX are each
"1 .Yes / 2 .No" (<https://www2.census.gov/programs-surveys/acs/tech_docs/pums/data_dict/PUMS_Data_Dictionary_2020-2024.txt>).
No count exists upstream of the tables either.

### A2. Internet subscription type (item 4)

**B28002 lines, verbatim:** Total; With an Internet subscription; Dial-up with no other type of
Internet subscription; Broadband of any type; Cellular data plan; **Cellular data plan with no
other type of Internet subscription**; Broadband such as cable, fiber optic or DSL; Broadband
such as cable, fiber optic or DSL with no other type of Internet subscription; Satellite Internet
service; Satellite Internet service with no other type of Internet subscription; Other service
with no other type of Internet subscription; Internet access without a subscription; No Internet
access.

- **Yes: "Cellular data plan with no other type of Internet subscription" is a category**
  (B28002_006).
- **Cable, fibre and DSL are ONE category.** The questionnaire asks them as one item: "five types
  of service: a.) cellular data plan for a smartphone or other mobile device, b.) broadband (high
  speed) Internet service such as cable, fiber optic, or DSL, c.) satellite, d.) dial-up or e.)
  some other service. Respondents could select 'Yes' for all categories that apply." (subject
  definitions, same section). The ACS cannot separate fibre from cable from DSL at any geography.
  Fixed wireless has no line of its own in the table (the 2021 report's footnote lists "a fixed
  wireless subscription" among broadband types; where a respondent puts it is **not confirmed**).
- It is a **subscription the household pays for**, not the technology available at the address,
  and it is a multi-select: the "X" lines overlap; only the "with no other type" lines are
  disjoint.
- "Internet access without a subscription" is a separate state (free municipal or campus service).

### A3. Household size (item 2)

| id | title | universe | categories |
|---|---|---|---|
| **B11016** | Household Type by Household Size | Households | family: 2, 3, 4, 5, 6, **7-or-more**; nonfamily: 1, 2, 3, 4, 5, 6, **7-or-more** (16 cells) |
| **B25009** | Tenure by Household Size | Occupied housing units | owner / renter × 1, 2, 3, 4, 5, 6, **7-or-more** (17 cells) |
| **B25010** | Average Household Size of Occupied Housing Units by Tenure | Occupied housing units | 3 cells: total, owner, renter |
| B11001 | Household Type (Including Living Alone) | Households | |
| B11002 | Household Type by Relatives and Nonrelatives for Population in Households | Population in households | gives the household population total |
| B25008 | Total Population in Occupied Housing Units by Tenure | Total population in occupied housing units | 3 cells |
| B08201 / B08202 | Household Size by Vehicles Available / by Number of Workers | Households | not wanted; also excluded at block group |

The top band is open ("7-or-more"). The mean of the open band is recoverable only as a residual:
(B25008 population − Σ k·households for k = 1..6) / households in the 7+ band, with compounded
error **[my reading]**. "Average Household Size – A measure obtained by dividing the number of
people in households by the number of households. ... rounded to the nearest hundredth."
(subject definitions.) "Households" and "occupied housing units" are the same count: "A household
includes all the people who occupy a housing unit." (same.)

### A4. Units in structure, year built, storeys (item 3)

| id | title | universe | bands |
|---|---|---|---|
| **B25024** | Units in Structure | **Housing units** (occupied AND vacant) | 1, detached; 1, attached; 2; 3 or 4; 5 to 9; 10 to 19; 20 to 49; **50 or more**; Mobile home; Boat, RV, van, etc. |
| **B25032** | Tenure by Units in Structure | Occupied housing units | owner / renter × the same ten |
| **B25033** | Total Population in Occupied Housing Units by Tenure by Units in Structure | Total population in occupied housing units | owner / renter × {1, detached or attached; 2 to 4; 5 or more; Mobile home; Boat, RV, van, etc.} |
| **B25124** | Tenure by Household Size by Units in Structure | Occupied housing units | owner / renter × {1, 2, 3, 4, 5-or-more persons} × {1, detached or attached; 2 to 4; 5 to 19; 20 to 49; 50 or more; Mobile home, boat, RV, van, etc.} (73 cells) |
| **B25034** | Year Structure Built | Housing units | 2020 or later; 2010-2019; 2000-2009; ... ; 1940-1949; 1939 or earlier |
| B25127 | Tenure by Year Structure Built by Units in Structure | Occupied housing units | |

- Mobile home and "Boat, RV, van, etc." are separate lines in B25024, B25032, B25033; merged in
  B25124.
- **No table gives buildings, or households per structure.** The definition is explicit: "In
  determining the number of units in a structure, all housing units, both occupied and vacant,
  are counted. Stores and office space are excluded. **The data are presented for the number of
  housing units in structures of specified type and size, not for the number of residential
  buildings.**" (subject definitions, "Units in Structure".) It is the respondent's answer about
  their own building (Housing Question 1).
- A "structure" is not always what a map calls a building: "1-Unit, Attached ... In row houses
  (sometimes called townhouses), double houses, or houses attached to nonresidential structures,
  each house is a separate, attached structure if the dividing or common wall goes from ground
  to roof." A terrace of ten row houses is ten structures.
- **Storeys: none.** The 2024 subject definitions contain no occurrence of "stories", "storeys"
  or "number of floors" (full-text search of the PDF), and no table in the 2024 5-year group
  list carries them. Confirmed absent.
- B25033 ÷ B25032 gives **persons per household by structure type** directly (five types), with
  no assumption **[my reading]**; B25124 gives the joint of size and structure at tract.

### A5. Group quarters (item 5)

**Out of every household table.** "People not living in households are classified as living in
group quarters." (subject definitions, "Household".) For the device questions: "Limitation of
the Data – These questions are not asked for the group quarters population, so do not include
data about people living in housing such as dorms, prisons, nursing homes, etc." (same,
"Computer and Internet Use".) Group-quarters people ARE in total-population tables: "Persons in
group quarters are included with persons in housing units (HUs) in all 2020-2024 ACS 5-year
estimates based on the total population." (accuracy document.) B26001 "Group Quarters
Population" (universe: Population in group quarters) is one cell, published at tract but not at
block group. Below state level the count is weak: "the weighting for the group quarters (GQ)
population is not controlled below the state level" (subject definitions).

So in a tract: B01003 total population = B11002/B25008 household population + B26001, and a tile
generator fed a population grid (which includes dormitories, barracks, prisons) will over-count
households near such places unless B26001 is subtracted.

---

## B. Geography

**Levels published, 5-year** (sheet "Summary Levels for 5-Year ACS Data Products" in the data
product list): tract (140): Detailed Tables, Data Profiles, Subject Tables for Specific Topics,
replicate-weight tables. Block group (150): Detailed Tables and replicate-weight tables only (no
S2801, no profiles). **No block level exists in the ACS.** The nation and state rows also carry
geographic components "Urban" (01) and "Rural" (43) for Detailed Tables.

| table | tract | block group |
|---|---|---|
| B28001, B28002, B28003, B28004, B28005, B28008, B28010, B28011 | yes | **yes** (only restriction listed: "Excludes Place/Remainder (070)") |
| B11016, B25009, B25010, B25024, B25032, B25033, B25034, B11002, B25008 | yes | **yes** (same) |
| B25124, B25127, B26001, B08201 | yes | **no** ("Excludes Place/Remainder and Block Group geographies (070,150,258,293,294)") |
| S2801 | yes | no |

The rule behind the exclusions: "No block groups ... will be published for ACS 5-year tables
that have 100 or more independent estimates" or that pose listed disclosure risks, including
"characteristics of people living in group quarters"
(<https://www2.census.gov/programs-surveys/acs/tech_docs/data_suppression/ACS_Data_Release_Rules.pdf>).

**How many.** 2020 Census tallies: census tracts 84,414 in the United States (50 states and DC),
981 in Puerto Rico, 133 in the Island Areas, 85,528 in all; block groups 239,780 / 2,555 / 412;
blocks 8,132,968 (US). <https://www.census.gov/geographies/reference-files/time-series/geo/tallies.html>.
The ACS covers the 50 states, DC and Puerto Rico. Whether tract counts changed between the 2020
tally and the 2024 vintage: **not confirmed**.

**Sizes.** "Census tracts generally have a population size between 1,200 and 8,000 people, with
an optimum size of 4,000 people. A census tract usually covers a contiguous area; however, the
spatial size of census tracts varies widely depending on the density of settlement." Block
groups "are generally defined to contain between 600 and 3,000 people."
<https://www.census.gov/programs-surveys/geography/about/glossary.html>. Extreme tract
populations and extreme LAND AREAS: **not confirmed** (the documentation gives no figures; they
are in the gazetteer file, which was not opened).

**[my arithmetic]** At the optimum 4,000 people a tract's area at each of the density-class lower
bounds in `sidecar.json` is: 250 /km² → 16 km²; 1,500 → 2.7 km²; 6,000 → 0.67 km²; 12,000 →
0.33 km². A village-class tract (say 20 /km²) is 200 km². So a tract is several to hundreds of
1 km tiles below the terraced class, about one tile at the apartment class, and a fraction of a
tile in dense fabric. The unit is neither the 1/8 degree cell nor the tile.

**Land area.** The gazetteer files: fields `ALAND` "Land area in square meters", `AWATER` "Water
area in square meters", `ALAND_SQMI`, `AWATER_SQMI` (square miles), `INTPTLAT`, `INTPTLONG`, and
`GEOIDFQ` "Fully qualified GEOID; used to join data tables from data.census.gov".
<https://www.census.gov/programs-surveys/geography/technical-documentation/records-layout/gaz-record-layouts.html>.
Files: <https://www2.census.gov/geo/docs/maps-data/data/gazetteer/2024_Gazetteer/> (national
tract file and per-state files; a block-group national file was not looked for: **not
confirmed**). The summary file's own `Geos20245YR.txt` holds labels; whether it carries areas:
**not confirmed**.

---

## C. Margins of error

**How published.** "In lieu of providing upper and lower confidence bounds in published ACS
tables, the margin of error is listed. All ACS published margins of error are based on a 90
percent confidence level." "Standard Error = Margin of Error / 1.645". (Accuracy document.)
Every estimate cell has its own MOE cell in the summary file and API (variables ending `M`).

**The Bureau's formulas.** In the handbook and in the "Instructions for Applying Statistical
Testing" the equations are typeset as images; the prose and the extractable fragments are quoted
and the algebra is transcribed. **Check the transcription against the PDF page before a freeze.**
Handbook: *Understanding and Using American Community Survey Data: What All Data Users Need to
Know* (2020), chapter 8,
<https://www.census.gov/content/dam/Census/library/publications/2020/acs/acs_general_handbook_2020.pdf>.
Instructions: <https://www2.census.gov/programs-surveys/acs/tech_docs/statistical_testing/2024_Instructions_for_Stat_Testing_ACS.pdf>.

- **Sum (or difference).** "1. Obtain the MOE of each component estimate. 2. Square the MOE of
  each component estimate. 3. Sum the squared MOEs. 4. Take the square root of the sum of the
  squared MOEs." MOE(X1+...+Xn) = ±sqrt(Σ MOE(Xi)²).
  Its stated weakness: "this method does not consider the correlation or covariance between the
  component estimates ... as the number of estimates involved in a sum or difference increases,
  the results of the approximation formula become increasingly different from the SE derived
  directly from the ACS microdata. Users are encouraged to work with the fewest number of
  estimates possible. If there are estimates involved in a sum that are controlled in the
  weighting, then the approximate SE can be increasingly different." The Instructions say
  "tremendously different".
- **Proportion** (numerator a subset of the denominator), P = X/Y:
  MOE(P) = (1/Y)·sqrt(MOE(X)² − P²·MOE(Y)²). "If the value under the square root sign is
  negative, then instead use the formula below. Note that this is the same formula used for a
  ratio." A further special case exists "where the proportion, P, equals 1" (formula is an image:
  **not confirmed**).
- **Ratio** (numerator not a subset; "persons per household" is named as an example):
  MOE(R) = (1/Y)·sqrt(MOE(X)² + R²·MOE(Y)²).
- **Product** and **percent change** have their own forms (percent change reduces to the ratio).
- CV = SE / estimate; "There are no hard-and-fast rules for determining an acceptable range of
  error in ACS estimates."
- "Data users may use the MOE in place of the SE in the formulas" (Instructions).

**Better than the formulas.** Variance Replicate Estimate tables: "sets of 80 replicate
estimates, which allow users to calculate measures of error for derived estimates using the same
methods that are used to produce the published MOEs ... These methods incorporate the covariance
between estimates that the approximation formulas in this document leave out", "for a subset of
the 5-year Detailed Tables for eleven summary levels, including the nation, states, counties,
tracts, and block groups" (accuracy document;
<https://census.gov/programs-surveys/acs/data/variance-tables.html>). Whether the B28 and B25
tables are in that subset: **not confirmed**. Also: where a collapsed table is published (B28010,
B28011, B25033's bands) its MOEs "will be more accurate than those users can approximate"
(handbook).

**Zero estimates.** "The estimate of the number or proportion of people, households, families,
or housing units in a geographic area with a specific characteristic is zero. A special procedure
is used to estimate the standard error." (accuracy document.) So a zero cell carries a non-zero
MOE. Lower bounds are clipped at zero ("the lower confidence bound should be reported as zero"),
and then "the level of confidence of the adjusted range of values is less than the prescribed
confidence level." The rule for summing several zero cells (use the largest zero-cell MOE once)
was **not confirmed** in the 2024 documents read; the 2024 worked-examples document
(<https://www2.census.gov/programs-surveys/acs/tech_docs/accuracy/2024_ACS_Accuracy_Document_Worked_Examples.pdf>)
does not state it in its extractable text.

**Controlled estimates.** "If the margin of error is displayed as '*****' (five asterisks), the
estimate has been controlled to be equal to a fixed value and so it has no sampling error. A
standard error of zero should be used for these controlled estimates". (Accuracy document.) The
controls are the Population Estimates Program's totals; "ACS estimates are controlled to
Population Estimates for age at the nation, state and county levels" and for sex likewise
(subject definitions); housing-unit and person totals are controlled in weighting, with the
2020 year controlled separately as one fifth (accuracy document). **Controls are not at tract
level.** A tract's total households and total population are sample estimates with MOEs. Other
MOE symbols: "**" too few observations; "***" median in an open-ended interval.

**MOEs are a lower bound.** "The estimated standard errors (and thus margins of error) included
in these data products do not account for variability due to nonsampling error ... The standard
errors calculated are only lower bounds of the total error."

**How many tracts exceed a CV of 30 %.** The Bureau's own statement, user note 2022-04
<https://www.census.gov/programs-surveys/acs/technical-documentation/user-notes/2022-04.html>:
the quality standard requires "that the coefficient of variation ... for a majority of the key
estimates is less than 0.30"; the ACS tests 17 key estimates that "Occur in about 10 percent of
the population at the national level"; "For each estimate, the median census tract-level CV is
calculated and compared to the benchmark of 0.30. ... In a typical year, 12 to 13 of the 17 key
estimates pass". For the 2016-2020 window: "caused an increase in the CVs by approximately
15-20% in relative terms (e.g., a CV of 0.25 would increase to 0.29-0.30) ... only 8 of the 17
key estimates passed the threshold with 9 of the 17 estimates having a median tract-level CV
that exceeded 0.30. ... Thus the 2016-2020 ACS 5-year data fails the quality standard
requirement ... We anticipate similar impacts to the margins of error in future 5-year products
that contain the 2020 data." **The 2020-2024 window is the last that contains 2020.**
Reading **[mine]**: for a characteristic held by about 10 % of people, about half of all tracts
have a CV near or above 0.30. Categories at 1 to 5 % (dial-up alone, satellite alone,
no-computer, 7-or-more persons, boat/RV) will be worse, at block group much worse. The exact
fraction for the B28 lines: **not confirmed**; it can be computed from the published MOEs.
A secondary paper the Bureau itself cites (Spielman, Folch and Nagle 2014, *Applied Geography*
46) was not read.

---

## D. Suppression and disclosure avoidance

**5-year tract and block-group tables are not suppressed for reliability or population size.**
"5-year estimates are published for all geographic areas regardless of population size"; the CV
filter (median CV above 0.61 drops a table) is used "extensively in data quality filtering rules
for 1-year estimates". (Release rules, issued 2024-10-18.) Handbook: "ACS 5-year estimates are
not subject to the same data suppression rules that the Census Bureau applies to 1-year
estimates." What applies at 5-year: the per-table geographic restrictions of section B.
Individual cells with too few observations show "-" and "**".

**What was done to the 2020-2024 data.** Accuracy document, "Confidentiality of the data":
"The Census Bureau has modified or suppressed some data on this site to protect confidentiality."
"Data swapping is done by editing the source data or exchanging records for a sample of cases
... A sample of households is selected and matched on a set of selected key variables with
households in neighboring geographic areas that have similar characteristics (such as the same
number of adults and same number of children). Because the swap often occurs within a
neighboring area, there is no effect on the marginal totals for the area or for totals that
include data from multiple areas." Synthetic data: "Persons are identified as being at risk for
disclosure based on certain characteristics. The synthetic data technique then models the values
for another collection of characteristics". "The data use the same disclosure avoidance
methodology as the original 1-year data." The swap rate is not published. So tract tables for
2020-2024 contain swapped households, and the swap keys include household composition; devices,
subscription and structure type travel with the swapped household. Differential privacy was
never applied to ACS tabulations (the only formal-privacy mechanism named is for the unweighted
sample-count tables B98001-B98003).

**The 2026 change.** Department of Commerce order DAO 216-26, "Disclosure Avoidance for
Statistical Products" (June 2026; the order's own page returned 403 to this reader, so its text
is **not confirmed** at source). The Bureau's account, director's blog, 2026-08-17
<https://www.census.gov/newsroom/blogs/director/2026/08/understanding-the-new-disclosure-avoidance-policy.html>:
"The administrative order allows two methods of disclosure avoidance: Coarsening ... Suppression
... This is permitted only as a last resort. ... a third method of disclosure avoidance called
noise infusion is no longer allowed. ... This includes statistical methods such as record
swapping and the use of synthetic data. ... Coarsening and suppression can reduce the amount and
detail of data published, especially for small geographic areas or population groups." "The
order also applies only to new products going forward. Products that have already been
published, such as the 2020 Census or the 2024 American Community Survey (ACS), won't be
affected. Internal microdata files with noise already infused can still be used if no additional
noise is added to new products derived from them." "ACS. We're researching new disclosure
avoidance methods. Transitioning to methods compliant with the new order will occur gradually,
with a full transition by 2029."

ACS user note, October 2026
<https://www.census.gov/programs-surveys/acs/technical-documentation/user-notes/2026-06.html>:
"The use of household record swapping ... has been discontinued for ACS 1-year tabulated data
products starting with the 2025 release. ... Although swapping was applied during the 1-year
microdata creation, it was designed to provide protection for small population areas (e.g.,
tracts, block groups) included in the 5-year data products, as well as individual records
included in the PUMS files."

**What this means for a design** **[my reading]**: the 2020-2024 5-year tables are settled and
will not change. The 2021-2025 5-year (not yet out) pools four swapped years and one unswapped
year, and the Bureau has not said what replaces swapping for tracts and block groups; the stated
alternatives are coarser categories and suppression "especially for small geographic areas".
Block-group publication of the B28 and B25 tables is the thing most at risk by 2029. What the
5-year 2025 release does: **not confirmed**. A reference frozen on 2020-2024 is safe; a plan to
"refresh annually" is not.

**A loose end.** The 2020-2024 PUMS dictionary marks the internet variables (ACCESSINET,
BROADBND, HISPEED, SATELLITE, DIALUP, OTHSVCEX) with "8 . Suppressed for data year 2023 and 2024
for select PUMAs" (ACCESSINET: 2024 only). The cause, and whether the tabulated B28002 tract
tables are affected in those areas, was **not confirmed**: no user note for it was found (2024-01
to 03 and 2025-01 to 07 are on other subjects; the errata list could not be read without
scripts). Read the errata before a freeze.

---

## E. Access and terms

**Summary file.** "The Table-Based ACS Summary File provides a separate file containing both the
estimates and MOE for all available geographies for a given Detailed Table. Data users only need
to merge in geography labels". Three file types: table labels (`ACS20245YR_Table_Shells.txt`),
geography labels (`Geos20245YR.txt`), and data files "organized by table ID"
(`5YRData/acsdt5y2024-b28001.dat`). The release rules call it "a pipe-delimited text file".
<https://www.census.gov/programs-surveys/acs/data/summary-file.html>. **One file per table, all
geographies together; there is no tract-only national file.** Tract rows are selected by the
geography id prefix (summary level 140; block group 150). Subject tables (S2801) are not in the
summary file. The directory holds 1,199 table files and one `5YRData.zip` of 12 G.

**API.** Terms: <https://www.census.gov/data/developers/about/terms-of-service.html> (page last
revised 2026-04-14).
- (ii) Attribution: "All services, which utilize or access the API, should display the following
  notice prominently within the application: 'This product uses the Census Bureau Data API but
  is not endorsed or certified by the Census Bureau.' You may use the Census Bureau name in
  order to identify the source of API content subject to these rules. You may not use the Census
  Bureau name, or the like to imply endorsement of any product, service, or entity".
- "Modification or False Representation of Content: You may not modify or falsely represent
  content accessed through the API and still claim the source is the Census Bureau."
- (iii) Re-identification: "users will not use these data, alone or in combination with any other
  Census or non-Census data, to identify any individual person, household, business or other
  entity; not link or combine these data with information in any other Census or non-Census
  dataset in a manner that identifies an individual person, household, business or other entity;
  not publish information from these data files, particularly in combination with any other
  Census or non-Census data, in a manner that identifies any individual ...; and not use the
  identity of any person or establishment discovered inadvertently and advise the Director of
  the Census Bureau of any such discovery".
- Use: "You may use the Census Bureau API to develop a service or service to search, display,
  analyze, retrieve, view and otherwise 'get' information from Census Bureau data." Nothing in
  the terms restricts redistribution of the data or of derived tables. The terms include an
  indemnification clause and a right to block.
- Limits: the current user guide (2026-05-12) says "You can include up to 50 variables in a
  single API query. **An API key must be used with all data queries made to the Census Data
  API.**" (`group(TABLE)` returns a whole table regardless.)
  <https://www2.census.gov/data/api-documentation/api-user-guide.pdf>. The older "500 queries
  per IP address per day without a key" figure is **not in the current guide: not confirmed**. A
  key needs an e-mail registration, which this task forbids; **the summary file needs no key and
  no agreement** and is the route to take.
- Whether the API notice is owed when the data come from the summary file and not the API
  **[my reading]**: the terms bind "access to or use of the API"; a product built from the
  summary file does not use it.

**(i) Public-domain status.** A census.gov sentence saying the data are in the public domain or
not subject to copyright was **not confirmed**: the policies, privacy, open-data and citation
pages read contain none. The basis usually given is 17 U.S.C. § 105 (works of the United States
Government), which was not read at source. What census.gov does say, citation page
<https://www.census.gov/about/policies/citation.html>: "Data users who create their own estimates
using data from disseminated tables and other data should cite the Census Bureau as the source of
the original data only. Conclusions drawn from any analysis of these data are the sole
responsibility of the performing party." Form: "U.S. Census Bureau, 'Table Name,' Full Product
Name, Vintage, <URL>, accessed on date."

**data.census.gov.** No separate terms page was found; it falls under "census.gov Website
Policies and Notices" named in the API terms. **Not confirmed** beyond that.

**For the sidecar's licence line** **[my reading]**: "U.S. Census Bureau, American Community
Survey 2020-2024 5-Year Estimates, Detailed Tables Bxxxxx, summary file; U.S. Government work,
no licence stated by the publisher; derived estimates are the author's, the Bureau is cited as
source of the original data only." The table holds class-level statistics and no tract id, which
also keeps well clear of the re-identification clause.

---

## F. Sizes

Each file is the national file for ALL geographies of that table (Content-Length by HEAD,
<https://www2.census.gov/programs-surveys/acs/summary_file/2024/table-based-SF/data/5YRData/>):

| file | bytes |
|---|---|
| acsdt5y2024-b28001.dat | 51,500,223 |
| acsdt5y2024-b28002.dat | 58,063,218 |
| acsdt5y2024-b28003.dat | 33,574,046 |
| acsdt5y2024-b28008.dat | 52,158,306 |
| acsdt5y2024-b28010.dat | 38,673,800 |
| acsdt5y2024-b28011.dat | 39,744,152 |
| acsdt5y2024-b28004.dat | 89 M (listing) |
| acsdt5y2024-b28005.dat | 78 M (listing) |
| acsdt5y2024-b11016.dat | 65,584,175 |
| acsdt5y2024-b25009.dat | 69,564,843 |
| acsdt5y2024-b25010.dat | 28,240,177 |
| acsdt5y2024-b25024.dat | 47,866,463 |
| acsdt5y2024-b25032.dat | 84,505,736 |
| acsdt5y2024-b25033.dat | 58,309,822 |
| acsdt5y2024-b25124.dat | 132,075,511 |
| acsdt5y2024-b25034.dat | 50,103,592 |
| acsdt5y2024-b26001.dat | 7,328,564 |
| acsdt5y2024-b01003.dat | 18,313,708 |
| acsdt5y2024-b25001.dat | 17,790,069 |
| acsdt5y2024-b25003.dat | 26,901,770 |
| acsdt5y2024-b25008.dat | 24,671,457 |
| documentation/Geos20245YR.txt | 90,707,242 |
| documentation/ACS20245YR_Table_Shells.txt | 5.4 M (listing) |
| gazetteer 2024_Gaz_tracts_national.zip | 2,420,663 |
| gazetteer 2025_Gaz_tracts_national.zip | 2,332,377 |

The share of each file that is tract rows: **not confirmed** (block groups, about 240 thousand,
outnumber tracts, about 85 thousand, so tracts are a minority of rows where block groups are
published **[my reading]**). Which gazetteer vintage matches the 2020-2024 release's geography:
**not confirmed**; read it from the release's geography notes.

---

## G. Comparable open sources elsewhere (documentation only)

**Eurostat, ICT usage in households and by individuals (isoc_*).** Reuse: "Reuse of statistical
data, metadata, publications, and other dissemination tools published on this website for
commercial or non-commercial purposes is authorised provided the source is acknowledged ... When
reuse involves ... modifications to the data or text, this must be stated clearly to the end user
of the information. A disclaimer regarding the non-responsibility of Eurostat shall be included."
<https://ec.europa.eu/eurostat/web/main/help/copyright-notice>. Grain: country, with a
degree-of-urbanisation breakdown (cities / towns and suburbs / rural) used in Eurostat's own
analyses of `isoc_ci_in_h`; NUTS breakdowns exist "only for a selection of indicators" in
regional tables per the metadata page <https://ec.europa.eu/eurostat/cache/metadata/en/isoc_i_esms.htm>
(search summary; the table structure itself was not opened: **not confirmed**, including which
NUTS level and whether any device-type series survives in current years). Presence and use, not
device counts, as far as read. A three-class urbanisation key is the nearest thing anywhere to a
density key that needs no spatial join.

**UK Census 2021 (ONS) and Ofcom Connected Nations.** ONS: "Most content on this website is
subject to Crown copyright protection and is published under the Open Government Licence (OGL)."
<https://www.ons.gov.uk/help/terms-conditions>. Household size and accommodation type tables
(believed TS017 and TS044) at Output Area grain: **not confirmed** (not read). The census is a
full count, so no margins of error, with its own disclosure control. No device question is known
to be in it: **not confirmed**. Ofcom: fixed-coverage files exist at postcode and at census
Output Area grain; a Spring 2025 "about this data" document speaks of "All 238,867 census output
areas" and says gigabit-capable but not full-fibre coverage is given at that grain for commercial
confidentiality (search summary; Ofcom's pages returned 403 to this reader). That is AVAILABILITY
of a technology at premises, the thing the ACS lacks, but not take-up. Licence wording for the
current files: **not confirmed** (an older readme points to OGL v3).

**Canada, Census Profile 2021 (Statistics Canada).** Statistics Canada Open Licence: "grants you
a worldwide, royalty-free, non-exclusive licence to: use, reproduce, publish, freely distribute,
or sell the Information; use, reproduce, publish, freely distribute, or sell Value-added
Products", with the notice "Adapted from Statistics Canada, name of product, reference date. This
does not constitute an endorsement by Statistics Canada of this product." and a no-linking-to-
identify clause. <https://www.statcan.gc.ca/en/terms-conditions/open-licence>. Household size and
structural type of dwelling at dissemination-area grain: **not confirmed** (not read). No device
question known.

**India, NFHS, and the DHS Program.** Microdata: "The DHS Program is authorized to distribute
datasets for legitimate academic research ... Agree to use the requested data only for the
registered research or study ... Agree that the datasets will not be shared with other
researchers without the written consent of The DHS Program."
<https://dhsprogram.com/data/Terms-of-Use.cfm>. Registration and a project are required: out of
bounds under the tranche's rules. STATcompiler (aggregate indicators, national and first-level
regions): terms page not found by this reader: **not confirmed**. Household assets (mobile
telephone, computer) are presence items, not counts, as far as known: **not confirmed**.

**IPUMS International.** "Redistribution: You will not redistribute the data ... redistribution
of the data to third parties is prohibited ... You agree to use IPUMS International data for
scholarly research and educational purposes only. Commercial use is strictly prohibited."
<https://international.ipums.org/international/terms.shtml>. Out of bounds. No open, harmonised,
global collection of household aggregates with a free licence was found.

**US alternatives inside the same publisher.** (a) ACS PUMS: household records with LAPTOP,
SMARTPHONE, TABLET, COMPOTHX, ACCESSINET, BROADBND, HISPEED, SATELLITE, DIALUP, NP (persons, 0 to
20), BLD (units in structure, ten codes), TEN, and PUMA, "areas with population of 100,000 or
more". This is the only place the full joint of device presence × household size × structure
type exists; its grain is far coarser than a tract and it still has no counts. (b) 2020 Census
DHC: tables H9 "HOUSEHOLD SIZE" and H12 "TENURE BY HOUSEHOLD SIZE" exist
(<https://api.census.gov/data/2020/dec/dhc/groups.json>); a full count at April 2020, the same
epoch as the population grid; the Bureau says many DHC tables go to block level, but block
availability of H9 specifically is **not confirmed**, and the DHC was protected by differentially
private noise, which is worst exactly at block level. It has no devices and no units in
structure. (c) The Bureau publishes no gridded product of these characteristics that this reader
found: **not confirmed**.

---

## H. Traps, most serious first

1. **Presence is not a count, and nothing upstream has the count.** Four Yes/No answers per
   household, "owned or used". A household of five with five phones and a household of one with
   one phone are the same row. Any "devices per household" from these tables is at most "number
   of device TYPES present" (0 to 4), and even that distribution is not published: B28001 gives
   the marginals, the four "only this type" cells and "none", not the joint. The consumer's
   device mix would be invented in two steps (types present, then count per type); only the
   first has a reference, and only its marginals.
2. **Near-saturation leaves little to key on, and what is left is the noisy part.** Nationally in
   2021: "95 percent had at least one type of computer, and 90 percent had a broadband internet
   subscription"; "90 percent of households owned at least one smartphone, while 81 percent ...
   a desktop or laptop", 64 % a tablet; urban 96 % against rural 93 % for any device (ACS-56).
   Those are 2021 figures; the 2020-2024 window will be higher. The proposed stake (fixed
   subscription share differs between lowest and highest density class) is reading a few points
   of difference, and the informative cells are the complements (no computer, smartphone only,
   cellular plan only, no subscription), each a small share with a tract CV likely above 0.3.
   Aggregated over thousands of tracts in a class the sampling error shrinks; the per-tract
   distribution within a class does not become readable.
3. **The tract is not the tile and not the cell.** Tracts are drawn to hold about 4,000 people,
   so area is roughly inverse to density. Joining tract statistics to 1 km tiles assumes every
   tile inside a tract has the tract's device mix, household size and structure mix, which is
   the even-spread assumption UNEVEN-0 measured as false for population, now applied to
   attributes. It also assumes a density for the tract (people over ALAND) equals the tile
   class, which holds only where the tract is about one tile. Honest keys, in order: (i) the
   tract's own density, stated as such, with tracts as the unit and no tiles; (ii) block groups
   (600 to 3,000 people), nearer a tile in suburban classes at the cost of roughly doubled
   error; (iii) the national Urban / Rural components, two classes, no join, published by the
   Bureau. The Bureau publishes no gridded or block-level ACS product. The decennial block
   tables give household size only, with their own noise.
4. **Structure type is the respondent's band, not a building.** "not for the number of
   residential buildings"; top band "50 or more" open; row houses are one structure each; the
   universe of B25024 includes vacant units while B25032 does not. Households per building is
   bounded, not measured: within a band, buildings = units / k for some k in the band, so the
   "20 to 49" band is uncertain by a factor of 2.4 and "50 or more" has no upper bound. A table
   named "households per building" from this source would be an assumption about k with a
   census label on it.
5. **MOEs must enter the resolution floor, and the easy way of combining them is wrong.** The
   sum formula ignores covariance and degrades with the number of terms; summing a class of
   thousands of tracts with it is exactly the use the Bureau warns against. Tract totals are not
   controlled. MOEs are "only lower bounds of the total error". Zero cells carry non-zero MOEs
   and clipped intervals. Use shares within a tract (proportion formula), few terms, and the
   replicate tables if the B28 tables are among them.
6. **Window against epoch, with 2020 inside it.** The estimates are "period estimates that
   describe the average characteristics ... over the period ... cannot be used to describe what
   is going on in any particular year". The window is 2020-2024; the population grid is epoch
   2020 and is itself a model from census counts; the tract boundaries are 2020 Census tracts.
   The 2020 year had "only two-thirds of the responses", special vacancy and non-response
   weighting ("COVID impacted months"), and is controlled as one fifth of the window. Device and
   subscription presence moved during these years (remote work and schooling, subsidy
   programmes), so the window average sits to the adoption side of 2020 and is not a 2020
   reading. This is a mismatch to state, not to correct.
7. **The universe excludes group quarters, and the grid does not.** Dormitories, barracks,
   prisons and nursing homes have no device, size or structure data, are in the tract's total
   population, are in the population grid, and their tract counts are weak below state level.
   Class means of persons per household must use household population (B11002 or B25008), never
   B01003.
8. **Persons against households.** B28005, B28008 and B28009 count PEOPLE in households;
   B28001-B28004, B28010, B28011 count HOUSEHOLDS. The one table with "cellular data plan alone"
   against "fixed with or without cellular" as a clean partition (B28008) is in persons. Mixing
   them double-counts large households.
9. **Subscription is not access network.** Cable, fibre and DSL are one answer; the answer is a
   paid subscription, not availability; cellular is multi-select with fixed. It can stand behind
   three states (fixed broadband; cellular only; none), not behind an access-technology kind,
   and says nothing on NAT class.
10. **Marginals, not the joint.** No table crosses device type with household size, tenure or
    structure. Size × structure × tenure exists (B25124, tract only, size capped at 5-or-more).
    Any generator drawing device mix conditional on household size is unreferenced here:
    `marginals_match_joint_differs` applies as the review said. PUMS has the joint, at 100,000+
    population grain.
11. **Question history.** Asked since 2013; "In 2016, questions ... were revised ... Caution
    should be used when comparing pre-2016 estimates". The whole 2020-2024 window is after the
    revision. No change to these questions in 2025 was found in the 2025 user notes (which list
    health insurance, educational attainment, and new housing items on electric vehicles, sewer,
    solar panels); the 2025 content-changes page itself was not read: **not confirmed**.
12. **The source is not stable going forward.** Swapping ends; coarsening and suppression are the
    permitted tools; "full transition by 2029"; a 2025 collection gap from the lapse in
    appropriations ("six-week suspension ... noticeable impacts to response rates as well as the
    margins of error"). Freeze on 2020-2024 and record its digest; do not promise a refresh.
13. **Swapped households inside the frozen data.** Marginal totals for an area are said to be
    unaffected "often", not always; cross-tabulated attributes of swapped households are from a
    neighbouring area. Class-level aggregates are insensitive to this; single-tract readings
    are not.
14. **Envelope.** United States (and Puerto Rico if kept). Applying a US tract-density
    relationship to the twin's world classes is the DEVICES-0 error in another direction.

---

## I. Smallest table set and geography for a first reference

**Geography: census tract (summary level 140), 2020-2024 5-year, from the table-based summary
file (no key, no agreement), keyed by the tract's own density** = household population over
`ALAND`, binned at the twin's class bounds and labelled "tract density", with tracts as the unit
and no tile join. Hold block groups as a second view to compare, not as the first read.

**Tables (seven data files, about 0.4 GB, plus the gazetteer):**

| need | table | why this one |
|---|---|---|
| denominator, household population | B25008 (3 cells) | population in occupied units, total and by tenure |
| household size distribution | B25009 (17) | 1 to 7-or-more; sum over tenure for the plain distribution, so one table serves both |
| device presence | B28001 (11) | the four marginals, the four "only" cells, none |
| subscription type | B28002 (13) | fixed, cellular, cellular only, satellite, none |
| persons per household by structure | B25032 (23) with B25033 (13) | units and people by structure type |
| size × structure joint | B25124 (73), tract only | the one joint that bears on households per building class |
| group quarters to subtract | B26001 (1), tract only | |
| area | gazetteer tract file | ALAND, square metres |

B25010 is redundant (it is B25008 over the B25009 total, rounded). B28003 is derivable in part
and adds only "has a computer, no subscription". B11016 adds household type, not needed. S2801
is not in the summary file and adds only percentages.

**What it CAN say**

- *Persons per household:* the distribution 1, 2, 3, 4, 5, 6, 7+ and the mean, by tract-density
  class, by tenure, and the mean by five structure types; the joint of size (to 5+) with six
  structure classes. This is a full reference for the row, in the United States, as a 2020-2024
  period average, with a published error on every cell.
- *Device mix per household:* the share of households with at least one desktop or laptop,
  smartphone, tablet, other; the share with exactly one type, by which; the share with none; and
  the share with fixed broadband, cellular plan, cellular only, satellite, no subscription, each
  by tract-density class. Whether those shares differ across density classes by more than their
  error: a stake that can fail, and may.
- *Households per building:* the share of housing units and of households in each structure-type
  band by tract-density class, and persons per household within five of those types. A bound on
  buildings per band.

**What it CANNOT say**

- *Persons per household:* nothing about a tile; nothing above 7 except a noisy residual mean;
  nothing outside the United States; nothing for group quarters.
- *Device mix per household:* **no count of devices of any kind**; not the joint of types beyond
  "only one type"; not devices by household size, tenure or building type; not ownership as
  distinct from use; not phones per person; nothing on NAT, router or access technology beyond
  three subscription states. The twin's "device mix per household" row, if it is a count by
  class, stays `unresolved: reference gives presence only`.
- *Households per building:* no building count, no storeys, no footprint, no households per
  structure. "50 or more" has no ceiling. Vacant units are in B25024 and absent from B25032.

**Before a freeze, read at source:** the proportion-equals-one and zero-sum MOE rules; whether
B28 and B25 tables have replicate tables; the errata for the 2023-2024 internet-variable
suppression; the geography vintage that matches the release; the text of DAO 216-26; a
census.gov statement of copyright status.
