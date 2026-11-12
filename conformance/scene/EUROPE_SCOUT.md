# Europe scout: which country's open small-area household data to reference first

*Read 2026-11-11/12. Documentation only: publishers' web pages, dataset-description spreadsheets,
methodology PDFs, licence pages, catalogue metadata and HEAD requests. No data file was
downloaded and no value-returning API was called. Every statement is from the page cited next to
it unless marked **[my arithmetic]**, **[my reading]**, **[search summary]** (a search engine's
summary of a page I did not open) or **not confirmed**. German, French and Dutch passages are
translated by me with the original phrase given.*

## 0. Verdict

**Do GERMANY first (Zensus 2022 grid results). Runner-up: SWITZERLAND (STATPOP + GWS hectare
data), conditional on its licence being cleared; if it is not, the fallback is the UNITED KINGDOM
(Census 2021 Output Areas), which is not a grid.**

Germany is the only candidate that publishes, as counts, on a regular INSPIRE grid that it
aggregates itself to 1 km, under a plain attribution licence, with no registration, BOTH
- households by number of persons (1, 2, 3, 4, 5, 6-or-more), and
- BUILDINGS by number of dwellings in the building (1, 2, 3-6, 7-12, 13-or-more),

plus dwellings by the same building-size bands (so dwellings per building within each band is
measured, including the open top band), for a population of about 83 million.

What Germany cannot give: any device or internet-subscription variable; storeys or building
height; time use; household size above "6 or more"; unperturbed small counts.

| | size distribution | buildings by dwellings | storeys | regular grid | counts | licence | free, no sign-up |
|---|---|---|---|---|---|---|---|
| **Germany** | 1..6+ | 1, 2, 3-6, 7-12, 13+ (buildings AND dwellings) | no | 100 m, 1 km, 10 km | yes, cell-key perturbed | dl-de/by-2-0 | yes |
| Switzerland | 1..6+ | 0, 1, 2, 3, 4, 5, 6, 7-9, 10+ | **yes** 1..6, 7-9, 10-14, 15+ | 100 m | yes, 1-3 shown as 3 (persons/households) | conflicting: "OPEN-BY" vs "BY ASK, commercial use only with permission" | yes |
| UK (E&W) | 0, 1..8+ | no (accommodation type only) | no | no (Output Areas) | yes, swapped + perturbed | OGL v3.0 | yes |
| France | only 1-person and 5+ | no (households in collective vs house) | no | 200 m, 1 km, "natural" | cells under 11 households are IMPUTED group means | Licence Ouverte 2.0 | yes |
| Netherlands | only 1-person + mean (1 decimal) | no (count of multi-family dwellings) | no | 100 m, 500 m | rounded to 5; under 5 hidden | CC BY 4.0 | yes |
| Eurostat 1 km | none | none | no | 1 km | n/a | Eurostat reuse | yes |
| Finland | yes (per content list) | yes (per content list) | n.c. | 250 m, 1 km | n.c. | **paid** (EUR 1,720 to 12,700 + VAT) | no |
| Austria | yes (package "Haushalte") | yes (package GWR) | n.c. | 250 m, 500 m, 1 km | swapped + thresholds | **paid** (thousands of EUR) | no |

---

## A. GERMANY — Zensus 2022, "Ergebnisse in Gitterzellen"

**Where.** Listing page: <https://www.zensus2022.de/DE/Ergebnisse-des-Zensus/_inhalt.html>, which
now redirects to <https://www.destatis.de/DE/Themen/Gesellschaft-Umwelt/Bevoelkerung/Zensus2022/_inhalt.html>
(section "Gitterdaten zum Download für geografische Informationssysteme (GIS)"). The old
zensus2022.de deep links (methods paper, FAQ) now redirect to that landing page.

"Zu allen im Zensus-Atlas gezeigten Karten sowie zu weiteren Merkmalen auf Gitterzellebene
befinden sich hier die zugehörigen Daten zur freien Nutzung (mit Namensnennung) als OpenData.
Jedes ZIP-Archiv enthält die Daten des Themas für das 10km-, 1km- und 100m-Gitter als csv-Datei
sowie eine ausführliche Datensatzbeschreibung als Excel-Datei." (The data are free to use, with
attribution, as open data; each ZIP holds the topic for the 10 km, 1 km and 100 m grid as csv plus
a dataset description in Excel.)

**Reference date.** "Basis der Auswertung sind Ergebnisse des Zensus 2022 mit Stichtag 15. Mai
2022." (every dataset description, sheet "Erläuterungen zu CSV-Tabellen").

**Grid.** "INSPIRE-konforme Gitterzellen mit einer Seitenlänge von 100 m, 1 km oder 10 km ...
Lambert-Azimuthal-Equal Area Projektion ... (ETRS89-LAEA Europe - EPSG:3035)". Cell id example
`CRS3035RES100mN4534600E5066300`; columns give the id and the cell-centre x/y. Each address is
assigned to exactly one cell by its geocoordinate. Cells without people or dwellings are NOT in the
file: "Gitterzellen in denen keine Personen leben oder in denen es keine Wohnungen gibt, sind nicht
in der Datei enthalten. Dies kann ggf. auch durch das angewendete Geheimhaltungsverfahren bedingt
sein." (... this may also be a result of the disclosure procedure.) The same EPSG:3035 1 km grid
is the one the Eurostat census grid uses. **Number of populated cells: not confirmed** (not stated
in the documentation read).

### A1. Variables on the grid, relevant to the consumer

Source for each: the dataset description ("Datensatzbeschreibung", xlsx, about 106 KB) at
`https://www.destatis.de/DE/Themen/Gesellschaft-Umwelt/Bevoelkerung/Zensus2022/Publikationen/Downloads-Publikationen/Gitterdaten/<name>.xlsx?__blob=publicationFile`.
Data ZIPs are at `https://www.destatis.de/static/DE/zensus/gitterdaten/<name>.zip`.

**(1) Persons per household — "Größe des privaten Haushalts in Gitterzellen"**
(`datensatzbeschreibung_groesse_private_Haushalte.xlsx`; ZIP
`Zensus2022_Groesse_des_privaten_Haushalts_in_Gitterzellen.zip`; version 2 of 14.07.2025).
CSV files `Zensus2022_Groesse_des_privaten_Haushalts_{100m,1km,10km}-Gitter.csv`. Columns:
`GITTER_ID_*`, `x_mp_*`, `y_mp_*`, `Insgesamt_Haushalte`, `1_Person`, `2_Personen`, `3_Personen`,
`4_Personen`, `5_Personen`, `6_Personen_und_mehr`. Counts of households. Semicolon-separated,
decimal comma.
- Definition: "Dieses Merkmal gibt die Anzahl der in einem privaten Haushalt lebenden Personen an.
  Personen in Gemeinschaftsunterkünften sind hier nicht enthalten, sondern nur Personen mit eigener
  Haushaltsführung." (People in communal accommodation are not included.)
- Household concept: "Zugrunde gelegt wird das „Konzept des gemeinsamen Wohnens“: Alle Personen,
  die, unabhängig von ihrem Wohnstatus (alleinige Wohnung, Haupt- oder Nebenwohnung), gemeinsam in
  einer Wohnung leben, gelten als Mitglieder desselben privaten Haushalts, sodass es einen privaten
  Haushalt pro belegter Wohnung gibt." (Household-dwelling concept: **exactly one private
  household per occupied dwelling**; people with a secondary residence there are counted.)
- A labelling fault in the documentation itself: the column now called `Insgesamt_Haushalte` is
  still described as "Bevölkerungszahl in Personen am 15.05.2022", and the change note says the
  column was named `Insgesamt_Bevoelkerung` from 17.02.2025 to 14.07.2025, "Die Werte in den
  csv-Tabellen bleiben unverändert" (values unchanged). **[my reading]** it is the household
  total; check on first read that it is near the sum of the six size columns.

Also on the grid: "Durchschnittliche Haushaltsgröße" (`DurchschnHHGroesse`, with a flag column
`werterlaeuternde_Zeichen`: "KLAMMERN = Aussagewert eingeschränkt, da der Zahlenwert statistisch
relativ unsicher ist"); "Typ des privaten Haushalts (nach Familien)" (single-person; couples
without children; couples with children; lone parents; multi-person without nuclear family);
"Typ des privaten Haushalts (nach Lebensform)"; "Seniorenstatus eines privaten Haushalts"; "Größe
der Kernfamilie (bis 6 und mehr Personen)"; "Typ der Kernfamilie (nach Kindern)".

**(2) Buildings by number of dwellings — "Gebäude mit Wohnraum nach Anzahl der Wohnungen im Gebäude"**
(`datensatzbeschreibung_gebaeude_mit_wohnraum_anzahl_wohnungen.xlsx`; ZIP
`Gebaeude_nach_Anzahl_der_Wohnungen_im_Gebaeude.zip`). CSV
`Zensus2022_Geb_nachZahl_Wohnungen_Geb_{100m,1km,10km}-Gitter.csv`. Columns: `Insgesamt_Gebaeude`
("Anzahl der Gebäude mit Wohnraum"), `1_Wohnung`, `2_Wohnungen`, `3bis6_Wohnungen`,
`7bis12_Wohnungen`, `13undmehr_Wohnungen`, each "Anzahl der Gebäude mit Wohnraum mit ...".
**These are counts of BUILDINGS**, which the US source does not have at all.

**(2b) Dwellings and buildings by building type and size**
- "Wohnungen nach Gebäudetyp (Größe)" (`Wohnungen_nach_Gebaeudetyp_Groesse.zip`): counts of
  DWELLINGS: `Insgesamt_Wohnungen`, `FreiEFH` (detached one-family house), `EFH_DHH`
  (one-family, semi-detached half), `EFH_Reihenhaus` (one-family, row house), `Freist_ZFH`,
  `ZFH_DHH`, `ZFH_Reihenhaus` (two-family: detached, semi, row), `MFH_3bis6Wohnungen`,
  `MFH_7bis12Wohnungen`, `MFH_13undmehrWohnungen` (multi-family by 3-6, 7-12, 13+ dwellings),
  `AndererGebaeudetyp`.
- "Gebäude mit Wohnraum nach Gebäudetyp (Größe)" (`Gebaeude_mit_Wohnraum_nach_Gebaeudetyp_Groesse.zip`):
  the same ten categories as counts of BUILDINGS.
- **[my reading]** dwellings in band / buildings in band = mean dwellings per building within
  the band, including the open "13 und mehr" band. That closes the open top band that the US
  "50 or more" leaves unbounded, at the price of dividing two independently perturbed numbers.

**Other dwelling/building grid tables listed on the same page** (not opened): Wohnungen nach Zahl
der Räume; Fläche der Wohnung (10 m² intervals); Durchschnittliche Wohnfläche je Bewohner / je
Wohnung; Gebäude nach Baujahr (decades, and microcensus classes); Eigentümerquote; Leerstandsquote
and Marktaktive Leerstandsquote; Nettokaltmiete; heating type and energy source for dwellings and
buildings. **Storeys or building height: no such table is listed.** Confirmed absent from the
listing.

**(3) Devices / internet: none.** No grid table, and no topic in the list, concerns devices or
internet access. **(4) Time use: none.**

### A2. Disclosure control ("Geheimhaltung")

Dataset descriptions: "Für die Wahrung der Geheimhaltung nach § 16 Bundesstatistikgesetz (BStatG)
wird ... ein Verfahren der stochastischen Überlagerung nach der Cell-Key-Methode (CKM) angewendet.
Einige Fallzahlen werden daher gegenüber ihrem Originalwert leicht verändert ausgewiesen. Dies hat
zur Folge, dass sich die ausgewiesenen Einzelwerte in der Tabelle nicht notwendigerweise zu den
ausgewiesenen Summen addieren." (Stochastic perturbation by the cell-key method; some counts are
shown slightly changed; values need not add to the stated totals.) Symbol legend: "– = Genau Null
oder auf Null geändert" (exactly zero OR changed to zero). "Leere Werte-Zellen kommen nicht vor."

Methods article (Enderle and Kleber, WISTA 6/2024,
<https://www.destatis.de/DE/Methoden/WISTA-Wirtschaft-und-Statistik/2024/06/geheimhaltung-cell-key-methode-zensus-062024.pdf?__blob=publicationFile&v=6>,
linked from <https://www.destatis.de/DE/Themen/Gesellschaft-Umwelt/Bevoelkerung/Zensus2022/Methoden/geheimhaltung.html>):
- **Zeros are kept; 1 and 2 never appear.** "Die 0 ist von der Überlagerung ausgeschlossen, sodass
  eine nicht vorhandene Beobachtung auch nicht künstlich erzeugt wird. Kleine Fallzahlen können
  hingegen auf eine 0 verändert werden. Fallzahlen von 1 oder 2 im Original werden immer überlagert
  und sind in den Zensus-Veröffentlichungen nie enthalten." (0 is excluded from perturbation, so
  no observation is invented; small counts can be changed to 0; original counts of 1 or 2 are
  always perturbed and never appear in census publications.)
- **Size of the noise.** "Der Überlagerungswert, der zu den Originalergebnissen addiert wird und
  im Mittel 0 ist, nimmt bei Fallzahltabellen ganzzahlige Werte überwiegend zwischen –2 und +2 an"
  (mean 0; integer values predominantly between -2 and +2). "Die Parameter des Verfahrens (zum
  Beispiel die maximale Überlagerung, die Varianz der Überlagerung oder die Wahrscheinlichkeiten,
  dass mit 0 überlagert wird) selbst können aus Gründen der Schutzwirkung nicht veröffentlicht
  werden." (**The parameters — maximum, variance, probability of no change — are not published.**)
  Quality statements given instead: "Der Absolutbetrag der Abweichungen ... liegt für reguläre
  Tabellen im Mittel unter 1. Mindestens 65 % der Fallzahlen in den Tabellen bleiben unverändert"
  (mean absolute deviation under 1; at least 65 % of counts unchanged). Whether these two
  statements hold for the grid files specifically: **not confirmed** (they are stated for
  "reguläre Tabellen").
- **Deterministic and consistent.** The perturbation is looked up from a cell key built from
  record keys, so the same cell gets the same value wherever it appears.
- **Non-additive.** Totals are perturbed separately from their parts ("Nicht-Additivität"): a
  1 km cell is NOT the sum of its 100 m cells, and `Insgesamt_*` is not the sum of its categories.
  The article's argument for it: summing perturbed parts would cumulate deviations.
- **Ratios.** Shares and means are computed from perturbed counts; "Übersteigt der aufgrund einer
  möglichen Überlagerung statistisch zu erwartende relative Fehler der Kennzahl 15 %, so wird das
  Ergebnis geklammert" (if the expected relative error exceeds 15 % the result is bracketed).
- Population totals of administrative units are published unperturbed; the grid population file
  is not exempt (its legend carries the same "auf Null geändert" symbol).
- Zensus 2011 used a different procedure, SAFE; the 2011 database results were later re-issued
  under the cell-key method (methods page).
- No minimum-population threshold for grid cells is stated in any document read: **not confirmed
  either way.**

### A3. Licence and attribution

Dataset descriptions: "Die mit dieser Datensatzbeschreibung beschriebenen Daten aus dem
Datenangebot der Statistischen Ämter des Bundes und der Länder sind lizenziert unter der
Datenlizenz Deutschland – Namensnennung – Version 2.0 (dl-de/by-2-0;
http://www.govdata.de/dl-de/by-2-0)." Imprint sheet: "© Statistische Ämter des Bundes und der
Länder, 2025. Vervielfältigung und Verbreitung, auch auszugsweise, mit Quellenangabe gestattet."

Licence text, official English, <https://www.govdata.de/dl-de/by-2-0>: "(1) Any use will be
permitted provided it fulfils the requirements of this 'Data licence Germany – attribution –
Version 2.0'. The data and meta-data provided may, for commercial and non-commercial use, in
particular be copied, printed, presented, altered, processed and transmitted to third parties; be
merged with own data and with the data of others and be combined to form new and independent
datasets; ... (2) The user must ensure that the source note contains the following information:
the name of the provider, the annotation 'Data licence Germany – attribution – Version 2.0' or
'dl-de/by-2-0' referring to the licence text available at www.govdata.de/dl-de/by-2-0, and a
reference to the dataset (URI). ... (3) Changes, editing, new designs or other amendments must be
marked as such in the source note."

### A4. Access and sizes

Direct HTTPS download, no registration (HEAD returns 200 for each). Each ZIP holds all three
resolutions; there is no 1 km-only file. The server sends no Content-Length; the byte figure
below is read from the ETag's size field **[my reading of the header]**, next to the size printed
on the listing page.

| ZIP | listing page | from ETag (bytes) | last-modified |
|---|---|---|---|
| Zensus2022_Groesse_des_privaten_Haushalts_in_Gitterzellen.zip | 10,8 MB | 24,208,870 | 2025-07-14 |
| Gebaeude_nach_Anzahl_der_Wohnungen_im_Gebaeude.zip | 19,07 MB | 19,529,852 | 2025-06-25 |
| Wohnungen_nach_Gebaeudetyp_Groesse.zip | 21,57 MB | 27,285,217 | 2025-04-07 |
| Gebaeude_mit_Wohnraum_nach_Gebaeudetyp_Groesse.zip | 26,65 MB | 22,088,112 | 2025-04-07 |
| Zensus2022_Bevoelkerungszahl.zip | 18 MB | 18,501,524 | 2024-10-08 |
| Durchschnittliche_Haushaltsgroesse_in_Gitterzellen.zip | 17 MB | 17,522,811 | 2025-02-05 |

The page and the header disagree for some files (the household-size file was re-issued in July
2025). Uncompressed sizes: **not confirmed**; the page warns the 100 m csv files are too large for
a spreadsheet.

### A5. Zensus 2011 grid

The 2022 page says 2011 grid datasets "werden hier in Kürze ... als INSPIRE-konforme 1
Kilometer- und 100 Meter-Gitter zur Verfügung stehen" (will shortly be available here). An older
English page "Grid cell results of the Census [2011]" listed "Wohnungen im 100 Meter-Gitter"
(zip, 256 MB) and "Gebäude im 100 Meter-Gitter" **[search summary]**; that URL now redirects.
Variables, disclosure method of the original 2011 grid files (SAFE) and current location:
**not confirmed.**

### A6. Traps (Germany)

1. **No 1 or 2 anywhere.** A true 1 or 2 becomes 0 or at least 3. At 100 m most category cells
   are that small (a hectare rarely holds three 5-person households), so the 100 m size
   distribution is mostly "–" and 3s in its tail. Summing 100 m cells up to a tile cumulates this.
   The 1 km file is perturbed once per cell on the true 1 km count and is the one to use.
2. **"–" is two things:** true zero, or a small count changed to zero. Indistinguishable.
3. **Parameters are secret.** The resolution floor cannot be taken from a published variance; only
   "mostly within ±2", "mean absolute deviation under 1", "at least 65 % unchanged".
4. **Non-additivity** in both directions (categories vs total; fine vs coarse grid).
5. **Households are constructed, not observed.** "Haushaltegenerierung": households are built from
   population registers, the building and dwelling census and the sample survey; register persons
   are matched to dwellings by the names of up to two occupants, and "Haushalte bzw. Personen, die
   ... noch keiner Wohnung zugeordnet sind, [werden] anhand von statistischen Kriterien auf die
   Wohnungen innerhalb des Gebäudes verteilt" (unmatched households are distributed over the
   dwellings in the building by statistical criteria)
   (<https://www.destatis.de/DE/Themen/Gesellschaft-Umwelt/Bevoelkerung/Zensus2022/Methoden/Haushaltegenerierung/haushaltegenerierung.html>).
   The building total is anchored; the split of persons across dwellings inside a large building
   is partly modelled. Size distributions in big-building cells inherit that.
6. **One household per occupied dwelling** by definition. Flat-shares are one household;
   "households per building" is "occupied dwellings per building", and the building tables count
   ALL dwellings including vacant ones. A vacancy rate is on the grid (Leerstandsquote) but is a
   ratio with bracket flags.
7. **Open top bands:** "6 Personen und mehr", "13 und mehr Wohnungen". The dwelling table bounds
   the latter's mean; nothing on the grid bounds the former except mean household size.
8. **Communal accommodation** is outside the household tables and inside the population grid
   (the US group-quarters trap again). 277 addresses in 163 municipalities (8,766 people) are
   missing from all grid files.
9. **Epoch.** 15 May 2022 against a 2020-epoch population grid.
10. **"Gebäude mit Wohnraum"** = buildings containing living space; non-residential buildings
    are not counted, and a building is not a footprint polygon.
11. **Envelope: Germany.**

---

## B. FRANCE — INSEE "données carroyées" (Filosofi)

**Latest:** Filosofi **2021**, published 12/02/2026: 200 m grid
<https://www.insee.fr/fr/statistiques/8735162>, 1 km grid <https://www.insee.fr/fr/statistiques/8735171>,
"niveau naturel"; documentation
<https://www.insee.fr/fr/statistiques/fichier/8735106/documentation_donnees-carroyees_filosofi2021.pdf>.
Coverage: metropolitan France, Martinique, La Réunion. Income year 2021; housing occupation known
to the dwelling tax at 1 January of the following year.

**Variables (200 m file, 34):** `ind` (individuals), `men` (households), `men_pauv`, **`men_1ind`
"Nombre de ménages d’un seul individu"**, **`men_5ind` "Nombre de ménages de 5 individus ou
plus"**, `men_prop` (owners), `men_fmp` (single-parent), `ind_snv`, `men_surf` (sum of dwelling
floor area), **`men_coll` "Nombre de ménages en logements collectifs"**, **`men_mais` "Nombre de
ménages en maison"**, `log_av45`, `log_45_70`, `log_70_90`, `log_ap90`, `log_inc` (dwellings by
construction period), `log_soc` (social housing), ten age bands `ind_0_3` ... `ind_80p`,
`ind_inc`; flags `i_est_200`, `i_est_1km`. All counts or sums.
- **(1)** Only three points of the size distribution: 1 person, 5 or more, and the mean
  (`ind`/`men`). Households of 2, 3 and 4 are one lump. **Not a distribution.**
- **(2)** Households in flats versus houses. **No buildings, no dwellings-per-building, no
  storeys.**
- **(3), (4)** nothing.

**Unit is the FISCAL household,** not the census household: "constitué par le regroupement des
foyers fiscaux répertoriés dans un même logement". Excluded: "les contribuables vivant en
collectivité (foyers de travailleurs, maisons de retraite, centres d'hébergement, maisons de
détention…) ; les sans-abri"; about 70,000 households with negative disposable income. Adult
children attached to their parents' tax return are counted in the parents' household even if they
live elsewhere: "cela génère une sous-estimation du nombre de ménages et d’habitants des villes
étudiantes et une surestimation de la taille des ménages des parents". Dwellings are those occupied
by fiscal households: "les logements vacants ne sont donc pas inclus". Footnote: since the phasing
out of the dwelling tax, "la couverture du fichier associé se dégrade. Les variables fondées sur
des informations issues de la taxe d’habitation doivent donc être utilisées avec prudence" — that
covers house/collective and the household-dwelling link itself. 28.8 million fiscal households,
63.8 million individuals.

**Confidentiality and imputation.** "Leur diffusion doit donc respecter la règle d’au moins 11
ménages par unité géographique d’observation." At 200 m: "79 % des carreaux habités de France
métropolitaine comprennent moins de 11 ménages fiscaux et doivent faire l’objet d’une imputation.
Ils représentent 20 % de la population totale." At 1 km: "64 % des carreaux de 1 km font l’objet
d’une imputation ; ils représentent 9 % de la population."
How: cells under 11 households are grouped, working down from the "natural level" quadtree, until
each group holds at least 11 households (a cell of 11 or more can be pulled into a group; the
largest group has 40 cells); then "on impute à chaque carreau, et pour toutes les variables de
diffusion, la valeur moyenne du groupe pondérée par le nombre d’individus" (each cell gets, for
every variable, the group's mean weighted by number of individuals). Flag `i_est_200` = 1 "si le
carreau est imputé par une valeur approchée"; `i_est_1km` likewise ("0 ou 2 sinon" in the 1 km
file; meaning of 2 **not confirmed**). The group id is no longer published ("Depuis le millésime
2019, le groupe d’imputation ... n’est plus diffusé"). The "niveau naturel" file has variable-size
cells and no imputation. Some sensitive variables (poor households) are also truncated.
**[my reading]** an imputed cell's composition is its neighbours' composition scaled to its own
head count: exactly the even-spread assumption, built in by the publisher, in four cells out of
five at 200 m.

**Licence.** INSEE legal notice <https://www.insee.fr/fr/information/2008466>: public information
on the site is under "Licence Ouverte / Open Licence version 2.0 (Etalab)", free reuse including
commercial, with "Source : Insee" and the date of last update **[search summary; page not
opened]**.

**Access and sizes** (from the pages): 200 m csv zip 87 Mo (shapefile 240 Mo, geopackage 294 Mo);
1 km csv zip 16 Mo. Direct download, HEAD 200, no registration.

**IRIS census tables** (households by size, dwellings by type, by number of dwellings in the
building): **not confirmed** — not read.

**Main trap:** it is a tax file. No size distribution, no buildings, most fine cells are
neighbourhood averages, and the house/flat split rests on a tax that is being abolished.

---

## C. NETHERLANDS — CBS "Kaart van 100 meter bij 100 meter met statistieken" / 500 m

Documentation: <https://www.cbs.nl/nl-nl/longread/diversen/2025/statistische-gegevens-per-vierkant-en-postcode-2022-2023-2024>
(chapters 2 and 4); download page
<https://www.cbs.nl/nl-nl/dossier/nederland-regionaal/geografische-data/kaart-van-100-meter-bij-100-meter-met-statistieken>.
Latest year 2024, added mid-2025 **[search summary]**. Reference date 1 January. Register-based.

**Variables:** `aantal_part_huishoudens` (private households), **`aantal_eenpersoonshuishoudens`**,
`aantal_meerpersoonshuishoudens_zonder_kind`, `aantal_eenouderhuishoudens`,
`aantal_tweeouderhuishoudens`, **`gemiddelde_huishoudensgrootte`** ("Afgerond op één decimaal");
`aantal_woningen`, `aantal_niet_bewoonde_woningen`, eight construction-period counts,
**`aantal_meergezins_woningen`** (dwellings that share a building with other dwellings or
business space), `percentage_koopwoningen`, `percentage_huurwoningen`, population by five age
bands. Institutional households excluded.
- **(1)** one-person count and a one-decimal mean; household TYPE, not size. **Not a
  distribution.** **(2)** a count of multi-family dwellings only; no buildings, no
  dwellings-per-building, no storeys. **(3), (4)** nothing.

**Disclosure:** "Gepubliceerde cijfers zijn afgerond op vijftallen. Percentages zijn afgerond op
tientallen procenten. Bij minder dan in totaal vijf inwoners worden geen gegevens over inwoners
opgenomen. Ditzelfde geldt ook voor particuliere huishoudens en woningen. Om redenen van striktere
geheimhouding kunnen additionele regels zijn toegepast." (Counts rounded to fives; percentages to
tens of per cent; under five, nothing.) Hidden or empty = `-99997` ("0 tot en met 4 / geheim /
niet aanwezig"). Sums of fine cells need not match the coarser cell. **[my reading]** rounding to
5 on a hectare (typically a few dozen households) is a 10 to 50 % error per category cell.

**Licence:** <https://www.cbs.nl/nl-nl/over-ons/website/copyright>: "Tenzij anders vermeld, is op
de inhoud van deze website de Creative Commons Naamsvermelding (CC BY 4.0) van toepassing ...
hergebruik ... is toegestaan, mits CBS als bron wordt vermeld." Sizes: **not confirmed** (the page
lists ZIP links without sizes). "Kerncijfers wijken en buurten": not read, **not confirmed**.

**Main trap:** rounding to five on tiny cells, and no size distribution at all.

---

## D. UNITED KINGDOM — Census 2021 (England and Wales)

Bulk page <https://www.nomisweb.co.uk/sources/census_2021_bulk>: "Bulk data products package all
the data from a 2021 Census dataset as a single zip file. Each zip file contains separate CSV
files for each geographic type (OA, LSOA, MSOA etc)." `census2021-ts017.zip`, `-ts044.zip`,
`-ts050.zip` all return HTTP 200 without registration; sizes **not confirmed** (no Content-Length).

- **(1) TS017 Household size** (<https://www.nomisweb.co.uk/datasets/c2021ts017>): "classify all
  households in England and Wales by household size ... as at Census Day, 21 March 2021". 10
  categories: Total; 0 people; 1; 2; 3; 4; 5; 6; 7; **8 or more**. Counts. The widest size
  distribution of any candidate. "Visitors staying at an address do not count."
- **(2) TS044 Accommodation type** (9 categories): detached; semi-detached; terraced; flats in
  purpose-built blocks; converted or shared house; another converted building; commercial
  building; caravan or other mobile structure (labels beyond the definitions page: partly **not
  confirmed**). A banded TYPE, like the US table and coarser: **no number of dwellings in the
  building, no buildings, no storeys.** TS050 number of bedrooms exists.
- **(3)** No device or internet question was found in the census tables: **not confirmed**. Ofcom
  Connected Nations: its pages returned 403 to the earlier reader (see ACS_DOCS.md section G) and
  were not retried; coverage (availability), not take-up; terms **not confirmed**.
- **Unit:** Output Areas, "between 40 and 250 households and ... a usually resident population
  between 100 and 625 persons" (<https://www.ons.gov.uk/methodology/geography/ukgeographies/censusgeographies/census2021geographies>).
  Count of OAs: **not confirmed** at source. Population-sized units, not a grid: the tract trap,
  at about one-twentieth of a tract's size.
- **Disclosure:** "Swapped records (targeted record swapping), for example, if a household was
  likely to be identified ... we swapped the record with a similar one from a nearby small area.
  Very unusual households could be swapped with one in a nearby local authority." and "Added small
  changes to some counts (cell key perturbation), for example, we might change a count of four to
  a three or a five." Parameters: not found.
- **Licence:** Open Government Licence v3.0; attribution "Contains public sector information
  licensed under the Open Government Licence v3.0."
  (<https://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/>).
- Scotland 2022 and Northern Ireland: not read. NISRA publishes a census grid-square product
  (1 km and 100 m; statistics only for squares with at least 30 residents in 10 households)
  **[search summary]**.

**Main trap:** not a grid, and building information stops at accommodation type.

---

## E. EUROSTAT / pan-European

- **Census 2021 1 km grid** (<https://ec.europa.eu/eurostat/en/web/gisco/geodata/population-distribution/population-grids>,
  version 2021 dated 30 May 2026): "Datasets are available for the 13 variables as defined in EU
  regulation 1799/2018 for: sex (male and female); age (under 15 years, 15-64, 65 years and over);
  current activity status (number of employed persons), voluntary bases; country / place of
  birth ...; place of usual residence (total population); place of usual residence one year prior
  to the census ...". **No household, dwelling or building variable. Confirmed.** It is useful as
  the common 1 km frame: the German 1 km cells are on the same EPSG:3035 grid.
- **ICT usage in households (isoc_*):** `isoc_ci_in_h` (level of internet access) has breakdowns
  by household type and degree of urbanisation; `isoc_ci_it_h` (type of connection) has fixed and
  mobile broadband codes as percentages of households, some series ending 2021; `isoc_r_iacc_h` is
  NUTS 2 **[all search summaries from mirror sites; the Eurostat tables were not opened: not
  confirmed]**. Country (or NUTS 2) grain, sample survey, presence not counts. Reuse: see
  ACS_DOCS.md section G for the copyright notice quoted at source.
- **EUBUCCO v0.1:** about 202 million buildings, EU-27 and Switzerland; height for about 73 %,
  type 46 %, construction year 24 %; "the majority of the data (95+%) is licensed under ...
  ODbL", with Prague CC-BY-SA and Abruzzo CC-BY-NC in separate files **[search summary of the
  Zenodo record; not opened]**. ODbL is share-alike for derived databases: a different obligation
  from the census licences. It has footprints and heights, not dwellings or households.
- **EU Building Stock Observatory; GHSL built-height licence:** **not confirmed** (not reached).

---

## F. Nordic countries, Switzerland, Austria, Spain, Estonia

### Switzerland — STATPOP and GWS hectare geodata (the runner-up on content)

Catalogue metadata `https://dam-api.bfs.admin.ch/hub/api/dam/assets/<id>`; documents at
`.../<id>/master`. The BFS website was relaunched; old geodata pages show only a language chooser.

- **(1) STATPOP, households (from 2012):** variable list (asset 27065724): `HjjPTOT`
  Privathaushalte Total; `HjjP01` mit 1 Person; `HjjP02`; `HjjP03`; `HjjP04`; `HjjP05`; `HjjP06`
  "mit 6 und mehr Personen"; `HjjPI` plausibility class of the hectare's households (1 = all
  plausible, 2 = at least one implausible household). Register survey, annual, reference date
  31 December; hectare grid, south-west corner in LV95 (data description, asset 27065723).
- **(2) GWS, buildings and dwellings** (variable list, asset 27905169): buildings by category
  (`GA03` Einfamilienhäuser, `GA05` Mehrfamilienhäuser, ...); by construction period; **"Wohngebäude
  nach Anzahl Geschossen": `GG01` 1 Geschoss ... `GG06` 6 Geschosse, `GG07` 7-9, `GG08` 10-14,
  `GG09` 15 und mehr**; **"Wohngebäude nach Anzahl Wohnungen": `GW00` ohne Wohnung, `GW01` 1,
  `GW02` 2, `GW03` 3, `GW04` 4, `GW05` 5, `GW06` 6, `GW07` 7-9, `GW10` 10 und mehr**; dwellings
  total, floor area, by rooms and period. Counts of buildings. Storeys: "ist das Parterre
  eingeschlossen. Dach- und Untergeschosse wurden nur gezählt, sofern sie mindestens teilweise für
  Wohnzwecke vorgesehen waren" (ground floor included; attic and basement only if at least
  partly residential). **The only candidate with storeys.**
- **Disclosure (STATPOP):** "Absolute Werte von 1 bis 3 dürfen in Standardauswertungen nicht
  abgegeben werden und sind daher als eine Klasse mit dem Wert «3» ausgewiesen." (Absolute values
  of 1 to 3 are shown as a class with the value 3.) Unclassed data only under a data-protection
  contract. Whether the same rule applies to GWS building counts: **not confirmed** (the GWS
  description read does not state it). People and households without a building id are placed on
  the municipality's centre hectare ("Sammelhektare"), listed in a `_noloc` table.
  **[my reading]** on a hectare almost every household-size cell is 1 to 3, so the published
  distribution is biased upward cell by cell, and summing hectares to 1 km sums the bias; no 1 km
  product is published.
- **Licence — conflicting.** Catalogue metadata for all four assets: termsOfUse code `OPEN-BY`,
  "Open use - Must provide the source". GWS geodata description (2023): "Opendata BY ASK: Freie
  Nutzung. Quellenangabe ist Pflicht. Kommerzielle Nutzung nur mit Bewilligung des Datenlieferanten
  zulässig. Die Quellenangabe ist in jeder Veröffentlichung obligatorisch: «Gebäude- und
  Wohnungsstatistik (GWS), BFS GEOSTAT»" (commercial use only with the supplier's permission).
  STATPOP description imprint: "Wiedergabe unter Angabe der Quelle für nichtkommerzielle Nutzung
  gestattet"; source note "STATPOP20jj, BFS GEOSTAT". Which governs the current files: **not
  confirmed.**
- **Access:** direct zip from the asset store, HEAD 200, free. Sizes: **not confirmed** (no
  Content-Length; an older GWS file is quoted at about 10 MB in a search summary). The assets
  read are flagged OLD/REDUNDANT, so later editions exist; the current edition's ids were not
  found: **not confirmed.**
- **Trap:** the 1-to-3 class at hectare grain, a licence that may forbid commercial reuse, and a
  population of under 9 million in mostly mountain terrain as the envelope.

### Finland — Ruututietokanta (Grid Database)
<https://stat.fi/en/services/statistical-data-services/geographic-data/grid-database>: content
covers household size and life stage, buildings and dwellings, on 250 m and 1 km **[search
summary]**; the page carries a price table ("EUR 4,800 ... EUR 12,700", "Value added tax of 25.5
per cent will be added to the price", by number of licences, with separate commercial prices) and
"data ordered for basic use may not be released or resold to a third party" **[search summary]**.
**Paid and not redistributable: out.** The free products are population-only 1 km / 5 km grids.

### Austria — Regionalstatistischer Raster
Price list <https://www.statistik.at/fileadmin/pages/454/RegionalstatistischeRasterDatenPreise.pdf>:
packages include "Paket Haushalte" with "Größe des Privathaushalts, Haushalte nach Typ" and "Paket
Gebäude- und Wohnungsregister" with "Gebäude nach Wohnnutzung und Anzahl der Wohnungen", at 250 m,
500 m, 1 km; "Preise in Euro für gesamt Österreich" (e.g. the population package 5,767.6 at the
finest size, internal or scientific use); "Größe 10km werden teilweise kostenlos abgegeben";
"Aus Datenschutzgründen werden Merkmale ... nur für ausreichend besetzte Zellen ausgewiesen",
based on swapped data plus thresholds. **Paid at useful resolution: out.**

### Norway — SSB "Statistikk på rutenett"
"Boligstatistikk på rutenett 250 m" (dwelling statistics) and "Bygningsmassestatistikk på ruter"
(building stock) exist and are publicly accessible
(<https://data.norge.no/en/datasets/662a7c03-5514-374c-b194-ab95fc7279c4/boligstatistikk-pa-rutenett-250-m>).
Variables, a household-size grid, disclosure rules and licence: **not confirmed.** Worth one more
documentation pass if a third country is wanted.

### Sweden, Denmark, Spain, Estonia
**Not confirmed** — the one combined search returned nothing usable and they were not pursued
once Germany's content was established.

---

## G. Ranking for this consumer

1. **Germany.** Both wanted distributions, as counts, natively at 1 km on the European grid, one
   attribution licence, no sign-up, largest population. Weakness: secret-parameter perturbation
   with no 1s or 2s; households partly modelled inside buildings; no storeys; no devices.
2. **Switzerland.** Richer buildings (exact 1 to 6 dwellings, storeys), same size distribution.
   Weakness: licence wording conflicts on commercial use; hectare-only with the 1-to-3 class; small
   and atypical envelope. First choice only if storeys are the row that matters and the licence
   is cleared in writing.
3. **United Kingdom.** Best size distribution (to 8+), clean licence. Not a grid; no buildings.
4. **France.** Clean licence, real grid, but a tax file: no size distribution, no buildings, 79 %
   of 200 m cells imputed.
5. **Netherlands.** Clean licence, real grid, annual; one-person count and a mean only; rounded
   to five.
6. **Eurostat grid.** No household variable; use as the frame only.
7. **Finland, Austria.** The content exists and is paid.

Devices (priority 3) do not separate the candidates: **no European census or register grid read
here has any device or subscription variable.** The only European source is the Eurostat ICT
survey at country or NUTS 2 grain with a three-class urbanisation key, presence not counts.
Time use (priority 4): nothing found at small-area grain anywhere.

---

## H. The winner: what to fetch, what must travel, what the design must do

**Smallest file set** (each ZIP contains 100 m + 1 km + 10 km csv and the description xlsx):

| need | ZIP | bytes (ETag) |
|---|---|---|
| households by size 1..6+ | `Zensus2022_Groesse_des_privaten_Haushalts_in_Gitterzellen.zip` | 24,208,870 |
| buildings by number of dwellings | `Gebaeude_nach_Anzahl_der_Wohnungen_im_Gebaeude.zip` | 19,529,852 |

**Total 43.7 MB [my arithmetic].** Two recommended additions, 89.5 MB in all:
`Wohnungen_nach_Gebaeudetyp_Groesse.zip` (27,285,217; dwellings per band, so the "13 und mehr"
mean is measured and households-per-building has a numerator) and `Zensus2022_Bevoelkerungszahl.zip`
(18,501,524; the population on the same cells, for the density key and to compare against the
consumer's own population grid). Base URL `https://www.destatis.de/static/DE/zensus/gitterdaten/`.

**Licence sentence to carry:** "Data licence Germany – attribution – Version 2.0 (dl-de/by-2-0),
www.govdata.de/dl-de/by-2-0."

**Attribution line for derived tables** **[my reading of clauses (2) and (3)]:**
"Source: Statistische Ämter des Bundes und der Länder, Zensus 2022, Ergebnisse in Gitterzellen
(reference date 15 May 2022), https://www.destatis.de/DE/Themen/Gesellschaft-Umwelt/Bevoelkerung/Zensus2022/_inhalt.html;
Data licence Germany – attribution – Version 2.0 (dl-de/by-2-0), www.govdata.de/dl-de/by-2-0;
data changed: aggregated into density classes by the author." The exact provider wording
"nach dessen Maßgabe": the imprint gives "© Statistische Ämter des Bundes und der Länder, 2025";
a prescribed citation string beyond that was **not confirmed.**

**What the design must do about perturbed small cells:**
1. **Read the 1 km file, never sum the 100 m file.** The 1 km cell is perturbed once on its true
   count; a sum of a hundred 100 m cells carries a hundred perturbations and the loss of every 1
   and 2. Keep the 100 m file only as a stated comparison (a kill: if class shares from summed
   100 m cells differ from the 1 km shares by more than the floor, the 100 m file is unusable for
   composition — expected).
2. **Never reconstruct a total from its parts.** Shares use the sum of the published categories as
   denominator, and the discrepancy between that sum and `Insgesamt_*` is itself recorded per cell
   as an observed size of the noise.
3. **Treat "–" as "0 or a small count set to 0"**, not as zero evidence; and treat any published 3
   in a tail category as "1 to about 5".
4. **Set a cell floor.** Admit a cell to a class only if its household (or building) total is
   large enough that ±2 per category is small against the category share; state the threshold
   before the read. Report how many cells and what share of households the floor removes per
   density class (the sparse classes will lose most).
5. **The resolution floor has no published variance.** Use the publisher's three statements
   (mostly within ±2; mean absolute deviation under 1; at least 65 % unchanged) as the per-cell
   bound, and the observed non-additivity from point 2 as a check on it. Class-level aggregates
   over thousands of cells benefit from the stated mean-zero noise; single-cell readings do not.
6. **Tail categories carry a known asymmetry:** zeros are never raised, 1s and 2s always move. In
   categories that are mostly 0, 1 or 2 per cell (6+ persons; 13+ dwellings outside cities) the
   published class total is not an unbiased sum. Flag those rows rather than correct them.
7. **Dwellings-per-building in a band is a ratio of two separately perturbed counts** from two
   files; compute it at class level only.
8. **Key by the cell's own census population** (from the population file) rather than joining to
   the consumer's 2020 grid, and report the comparison between the two as a separate reading. The
   cell IS the tile here, which removes the US tract-versus-tile mismatch.
9. Subtracting communal-accommodation residents is not possible from these files: household
   population per cell is recoverable only as Σ k·households with the open 6+ band; state it.

**What Germany still cannot give:** device counts or even device presence; internet subscription
type; storeys or height; who is home when; household size beyond 6+; the joint of household size
with building size (two separate marginal tables: `marginals_match_joint_differs` applies);
households per building as observed (it is occupied dwellings per building, with vacancy as a
separate bracketed ratio); anything outside Germany.

**Before a freeze, read at source:** whether `Insgesamt_Haushalte` is the household total (the
description mislabels it); the actual byte sizes and digests; whether a prescribed citation string
exists; whether any minimum-population rule applies to grid cells; the 2011 grid's status; the
quality report (<https://www.destatis.de/DE/Methoden/Qualitaet/Qualitaetsberichte/Bevoelkerung/zensus-2022.pdf?__blob=publicationFile&v=4>,
not opened) for the household-generation error.
