# SCOUT: presence by hour (A), ties by distance (B), daily mobility (C)

Scouted 2026-11-11. Documentation only: publishers' pages, data dictionaries, methodology notes, licence pages, catalogue metadata, HEAD requests. No data file was downloaded and no data values were requested.

Marking convention. A statement with a URL beside it and no tag was read on that page today. Tags:
- **[search summary]** = from a web-search result summary, not read on the publisher's page.
- **[my reading]** = my inference or my recollection, not stated by the publisher.
- **[not confirmed]** = looked for and not found.
- **[archived copy]** = read from web.archive.org because bls.gov refuses automated readers ("Access Denied ... bot activity ... is prohibited"). I did not work around that block on bls.gov itself.

Documentation files I did fetch (all are dictionaries or methodology, none are data): ATUS interview data dictionary PDF (archived copy), LODES Tech Doc 8.5 PDF, SCI methodology PDF, ZVE 2022 PUF variable description xlsx (180 KB) and activity list PDF, NHTS 2022 User's Guide PDF, MiD 2023 study summary and small-area report PDFs, Pendlerrechnung quality report PDF, three papers (Bailey 2018, Bailey 2020, Backstrom 2010), Eurostat catalogue table of contents and three SDMX content-constraint (code list) files.

---

## Headline findings

1. **The German ZVE 2022 Public Use File has everything A asks for, including device use, per 10-minute slot.** The variable description lists, for each of 144 slots: main activity, secondary activity, "computer/tablet/smartphone used during this" (yes/no), place or transport mode (with "at home" as a code), who was present, plus weekday, household size 1-7 and a day weight. Free, but needs registration. The Destatis web summary of the survey omits the place variable; the dictionary has it.
2. **Meta re-released the Social Connectedness Index in 2026 at finer grain, marked CC0 on HDX**: US ZCTA pairs, EU NUTS3 pairs, GADM2, snapshot of 25 January 2026. Still nowhere near 1 km.
3. **No open table gives ties at building / street / 1 km grain.** The finest measured curve is Backstrom, Sun and Marlow 2010 (Facebook home addresses, 0.1-mile buckets). Below roughly a mile the curve is flatter than 1/d. That is a literature constant, not a table.
4. **"Share who do not leave home" has a direct open US series** (BTS Daily Mobility Statistics, county by day, "no trip ending more than one mile from home") and a direct German survey figure (MiD 2023: 82 % of people are out on an average day).
5. **Trap that recurs**: the convenient tables (Eurostat HETUS time-of-day, SCI, LODES) each answer a question adjacent to ours. Details under each source.

---

# A. PRESENCE BY HOUR OF WEEK

## A1. Zeitverwendungserhebung (ZVE) 2022, Public Use File. Germany

**What is published.** "Das Public Use File der Zeitverwendungserhebung 2022 ist eine absolut anonymisierte 80%-Substichprobe aller Haushalte ... Es umfasst insgesamt 7.819 Haushalte." (The PUF is an absolutely anonymised 80 % subsample of all households; 7,819 households.) Four linked files: household, person, time-slot ("Zeittaktdaten") and sum data. DOI 10.21242/63911.2022.00.00.4.1.1. https://www.forschungsdatenzentrum.de/de/10-21242-63911-2022-00-00-4-1-1

Variables, from the variable description `zve_2022_puf_dsb_v2_0.xlsx` (180 KB, same page), sheet "Taktdaten":
- `tb1x_1 … _144` main activity per 10-minute slot, 04:00 to 04:00 ("Hauptaktivität 04:00-04:10", codes 110-999).
- `tb2x_*` secondary activity ("Nebenaktivität", 0 = none).
- `tb3_*` "Dabei Computer/Tablet/Smartphone genutzt" (computer/tablet/smartphone used during this), 0 = no, 1 = yes ("IKT-Nutzung").
- `tb4_*` "Ort oder Verkehrsmittel" (place or transport mode), codes 00-39.
- `tb5_* … tb10_*` who was present: alone, partner, children under 10, parent, other household members, other known persons.
- `wtagfei` weekday with holidays folded in: 1 = Monday … 6 = "Samstag/Heiligabend/Silvester", 7 = "Sonntag/Feiertag".
- `hrf_t80` day weight ("Hochrechnungsfaktor für den Tag"); `tagnr` diary day 1-3.
- Household file: `ha1x` "Anzahl der im Haushalt lebenden Personen" 1-7, `hhtyp` household type, `hb133a/bx` smartphone/tablet yes/no and count, `hb141-143` internet access, fixed, mobile.
- Person file: age groups, "Dauer Internetnutzung am Werktag / am Wochenendtag" (duration of internet use on a working day / weekend day), "Haupterwerbstätigkeit von zu Hause" (main job from home).

Place codes, from the activity list `zve_2022_puf_aktivitaeten_2.pdf` (158 KB, same page), "Liste Ort/Verkehrsmittel PUF": 00 unspecified; 10 unspecified place; **11 Zu Hause (at home)**; 12 holiday home; 13 own workplace or school [label truncated in my extract]; 14 at others' homes; 15 restaurant/café/bar; 16 shopping centre/supermarket/other; 17 hotel/guest house/campsite; 19 other place; 20 unspecified mode; 21 on foot; 22 e-scooter; 23 bicycle; 24 moped/motorcycle; 25 car; 26 lorry/minibus; 29 other private; 31 taxi; 32 bus; 33 tram/underground; 34 train/S-Bahn; 35 aeroplane; 36 boat/ferry; 39 other public.

**Grain.** Person aged 10+ in private households, 10-minute slots, three diary days per person (two weekdays, one weekend day). Reference period January to December 2022. Region: only "Gebietsstand (Alte Bundesländer und Neue Bundesländer einschließlich Berlin)" (old vs new Länder). https://ec.europa.eu/eurostat/cache/metadata/EN/tus_20_simstu_de.htm

**Sample.** Gross sample 11,940 households (Eurostat national metadata, same URL). Full-sample person and diary counts [not confirmed] from pages read.

**Licence and access.** "Für die Nutzung von Public Use Files ist eine Anmeldung über ein Registrierungstool notwendig ... Die Nutzung ist kostenfrei." (Use of PUFs requires registration through a registration tool; use is free.) Two-step e-mail registration. https://www.forschungsdatenzentrum.de/de/public-use-files . The national metadata says the PUF tier exists because data "anonymized to an extent that re-identification ... is not possible at all" may go to "any person or institution". A licence text covering commercial use or redistribution of derived statistics: [not confirmed]; the page points to citation rules only. **This needs a sign-up, which the lead told me not to do.**

**File sizes.** Data file sizes are behind the registration: [not confirmed]. Documentation: 356 KB + 503 KB + 180 KB + 158 KB + questionnaires 181 KB, 608 KB, 1.18 MB.

**Disclosure control, weighting, biases.**
- Quota sample, not a probability sample: "The ZVE is based on a non-probabilistic sampling method, the quota sample. In a strict sense, non-probabilistic samples are not representative". Quotas on Land, household type, status of main earner, municipality size class, from the 2019 Microcensus.
- "a bias toward individuals with academic qualifications ... underrepresentation of people with foreign" [citizenship/background; sentence truncated in my extract].
- "Travel times were not always recorded by respondents and are therefore underreported." "56% of respondents reported they did not fill out the diary in real-time".
- Christmas Eve and New Year's Eve are assigned to Saturdays and public holidays to Sundays.
- Publication rule for our own derived tables: not fewer than 25 persons or 50 diary days per cell; bracket 25-99 persons.
- Children: "Kinder unter 10 Jahren" in the with-whom column, not under 18 as HETUS specifies.
All from https://ec.europa.eu/eurostat/cache/metadata/EN/tus_20_simstu_de.htm

**Cannot give.** Any geography below old/new Länder, so no density class from the PUF itself [my reading; a municipality-size variable was not in the part of the dictionary I listed, not confirmed either way]. Asleep-but-device-on. Children under 10 have no diary. 2022 still carries some pandemic-era home working.

**Verdict: usable with caveats, and the first choice for A in Germany.** Caveats: registration required; quota sample; licence for redistribution of derived statistics not stated. The SUF (95 %) is restricted to research institutions and is fee-bearing ("Die Nutzung der amtlichen Mikrodaten ist entgeltpflichtig", https://www.forschungsdatenzentrum.de/de/entgelte); not for us.

## A2. American Time Use Survey (ATUS) 2003-2024 multi-year microdata. United States

**What is published** [archived copy of https://www.bls.gov/tus/data/datafiles-0324.htm, snapshot 2026-10-20]. Zip files, each with a data file and SAS/SPSS/Stata read-in programs:

| File | Size | Content |
|---|---|---|
| atusact-0324.zip | 77.1 MB | Activity file: "activity codes, activity start and stop times, and locations" |
| atusresp-0324.zip | 15.5 MB | Respondent file: labour force status, earnings, diary day, weights |
| atusrost-0324.zip | 3.0 MB | Roster: household members, age, sex |
| atuswho-0324.zip | 17.3 MB | Who file: "who was present during each activity" |
| atussum-0324.zip | 15.0 MB | Activity summary (total minutes per activity per respondent) |
| atuscps-0324.zip | 95.6 MB | CPS variables for all household members, collected 2 to 5 months earlier |
| atuswgts-0324.zip | 285.2 MB | Replicate weights (not needed for point estimates) |

**Location variable**, from the 2003-2024 Interview Data Dictionary [archived copy of https://www.bls.gov/tus/dictionaries/atusintcodebk0324.pdf]: `TEWHERE` "Edited: where were you during the activity?" 1 Respondent's home or yard; 2 Respondent's workplace; 3 Someone else's home; 4 Restaurant or bar; 5 Place of worship; 6 Grocery store; 7 Other store/mall; 8 School; 9 Outdoors away from home; 10 Library; 11 Other place; 12 Car, truck, or motorcycle (driver); 13 (passenger); 14 Walking; 15 Bus; 16 Subway/train; 17 Bicycle; 18 Boat/ferry; 19 Taxi/limousine service; 20 Airplane; 21 Other mode of transportation; 30 Bank; 31 Gym/health club; 32 Post Office; 89 Unspecified place; 99 Unspecified mode of transportation. Start and stop times are `TUSTARTTIM`, `TUSTOPTIME`; who-present is `TUWHO_CODE`.

**Grain.** One respondent aged 15+ per household, one diary day, 4 a.m. to 4 a.m. [my reading for the age and day boundary; consistent with the dictionary's examples]. National; no small-area geography in the public file [my reading]. Total respondents across 2003-2024: [not confirmed] (roughly 245,000 by my recollection).

**Licence.** BLS material is "public domain, except for previously copyrighted photographs and illustrations. You are free to use our public domain material without" [specific permission; sentence truncated] [archived copy of https://www.bls.gov/bls/linksite.htm]. No sign-up. No payment.

**Traps.**
- **"TEWHERE is not collected for activities with activity codes of 0101xx, 0102xx, 0104xx, 500105, or 500106."** Sleeping, grooming and personal activities have no location. "At home asleep" must be imputed from the neighbouring episodes. A naive "share at home by hour" from TEWHERE alone will show almost nobody at home at 3 a.m.
- Who-present was not collected for work activities before 2010 ("Starting in January 2010, TUWHO_CODE information was collected for work activities").
- Weights: use `TUFNWGTP` for 2003-2019 and 2021 onward; 2020 needs `TU20FWGT` and "Annual estimates with the 2020 data are not possible" (collection suspended 18 March to 9 May 2020).
- Age top-coded at 80/85.
- Weekend days are oversampled, so weights are mandatory for any day-of-week pooling [my reading].
- One person per household: no within-household joint presence ("is anyone home") except through the who-present codes.
- No device-use variable. Secondary activities are not collected except childcare and (in some years) eating [my reading]. So "awake and using a device" cannot come from ATUS beyond primary activities coded as computer use, phone calls or TV.

**Cannot give.** Device use while doing something else; children under 15; simultaneous presence of all household members; geography.

**Verdict: usable now, first choice for A in the United States.** Smallest set: activity + respondent + roster = 95.6 MB (112.9 MB with the who file). IPUMS ATUS-X offers the same data with an extract builder but needs registration and restricts redistribution [search summary for IPUMS terms generally: "other redistribution requires contacting IPUMS"].

## A3. Eurostat HETUS wave 2020 tables. Germany 2022 plus 10 other countries

**What is published.** "HETUS 2020 results are organised in 23 tables in Eurobase." https://ec.europa.eu/eurostat/cache/metadata/en/tus_20_esms.htm . Countries so far: "Bulgaria, Germany, Estonia, Croatia, Hungary, Netherlands, Austria, Poland, Finland, Norway and Serbia"; "2022: Germany". Last table update 29 July 2026 (catalogue).

The three tables that matter, with dimensions read from the SDMX content constraints (https://ec.europa.eu/eurostat/api/dissemination/sdmx/2.1/contentconstraint/ESTAT/TUS_20STARTIME and likewise TUS_20LOC, TUS_20WWHOM):

| Table | Values | Dimensions |
|---|---|---|
| `tus_20startime` "Participation rate in the main activity (wide groups) by sex and time of the day" | 43,065 | sex (T, M, F) × 144 ten-minute slots (H0400-0409 …) + TOTAL × 8 activity groups + TOTAL × 11 countries; unit PTP_RT |
| `tus_20loc` "Time spent on main activities by sex and location" | 3,267 | sex × place (TOTAL, HOM, 2HOM, WPL_SCH, HOM_OTH, REST, SHOP, OTH, NSP) × 10 activity groups; unit TIME_SP |
| `tus_20wwhom` "Time spent alone or with others by sex, age group and person category" | 3,439 | sex × 15 age groups × (child, parent, partner, other household member, other known person, alone) |

Activity groups in `tus_20startime` are code-named AC0_X_021, AC021, AC1_2, AC3_713_936-980, AC4-8_998_X_713_821, AC821, AC910_920, AC995-999. [my reading of the codes: personal care except sleep; sleep; employment and study; household and family care; leisure and social; TV; travel to/from work or study; unspecified. Labels not read from the code list.]

Also present: `tus_20week` (by day of the week, 32,001 values), `tus_20hhstatus` (by household composition, 34,340), `tus_20age`, `tus_20screen` (screen time by age).

**Licence.** "Reuse of statistical data, metadata, publications, and other dissemination tools published on this website for commercial or non-commercial purposes is authorised provided the source is acknowledged." Modifications must be stated. Exception: "Data identified as belonging to sources other than Eurostat". https://ec.europa.eu/eurostat/help/copyright-notice . No sign-up.

**Sizes.** Tens of thousands of values per table; each is well under 1 MB as TSV [my reading from the value counts].

**Traps.**
- **The time-of-day table has no weekday/weekend split, no age, no household size and no location.** It is activity by slot for an average day. It gives 144 slots, not 168 hours.
- **The location table has no time of day.** It is minutes per day by place. Presence at home by hour cannot be built from these two tables without an assumption joining them.
- The 2000/2010 tables "refer to the population 20 to 74 years old only" (https://ec.europa.eu/eurostat/cache/metadata/en/tus_00_esms.htm); whether the 2020 time-of-day table uses the same age window: [not confirmed].
- Germany is a quota sample (see A1). "some countries collected data during the COVID-19 pandemic".
- Cells below 25 observations are flagged ":u".
- "Microdata for the HETUS 2020 are not yet available." Eurostat HETUS microdata are for "Universities and certified research centres" only.

**Verdict: usable now as a no-sign-up cross-check, not sufficient alone.** It is the fastest reference for "share asleep / at work / travelling by slot" in Germany, and a check on whatever we compute from the PUF.

## A4. Destatis published ZVE 2022 tables. Germany

"Statistischer Bericht - Erhebung zur Zeitverwendung privater Haushalte - 2022 (revidierte Fassung vom 06.06.2025)", xlsx, 528 KB. https://www.destatis.de/DE/Themen/Gesellschaft-Umwelt/Einkommen-Konsum-Lebensbedingungen/Zeitverwendung/Publikationen/Downloads-Zeitverwendung/statistischer-bericht-zeitverwendung-priv-haushalte-5639102229005.html . Also GENESIS-Online statistic 63911. Results were revised for the Zensus 2022 population ("revidierte Ergebnisse ... Grund sind die aktualisierten Bevölkerungsdaten des Zensus 2022").

Licence: "Datenlizenz Deutschland – Namensnennung – Version 2.0"; "Die bereitgestellten Daten und Metadaten dürfen für die kommerzielle und nicht kommerzielle Nutzung ..." (may be used commercially and non-commercially) with source note. https://www.destatis.de/DE/Service/Impressum/copyright-genesis-online.html

Whether the report contains a time-of-day sheet: [not confirmed]; the web tables listed are all average minutes per day by sex, age, household type. **Verdict: usable now for daily totals only; no hourly profile established.**

## A5. Not recommended for A

- **Multinational Time Use Study (MTUS)**: registration with a project description of at least 35 words; "other redistribution requires contacting IPUMS" [search summary]. Not usable under our no-sign-up rule; adds nothing over ATUS + ZVE for DE/US.
- **UK Time Use Survey 2014-15**: UK Data Service registration and End User Licence [my reading, not confirmed today]. Not usable now.
- **Device use by hour**: no open population table found. A search surfaced the Cambridge Device Analyzer dataset and Lancaster University hourly files derived from it (app foreground time and data demand by hour and day of week); volunteer Android users, licence [not confirmed] [search summary]. Treat as a shape prior at most. The ZVE `tb3` flag is the only population-representative hourly device-use measure I found.

---

# B. TIES BY DISTANCE

## B1. Meta Social Connectedness Index, 2026 release. 178 countries

**What is published.** HDX dataset "Facebook Social Connectedness Index", read through the catalogue metadata at https://data.humdata.org/dataset/social-connectedness-index . Data period 2025-12-26 to 2026-01-25. "This release updates the SCI with substantially expanded global coverage and finer geographic resolution, including new subnational units across 178 countries."

| Resource | Size (catalogue) | Content |
|---|---|---|
| nuts_2024.zip | 14.2 MB | "European subnational SCI pairs based on NUTS 2024 regions. Includes all NUTS levels (1, 2, 3)." Three files |
| us_counties.csv | 245.4 MB | County-to-county pairs, 5-digit FIPS |
| us_zcta.zip | catalogue shows 2,459 bytes, so the entry is a pointer; true size [not confirmed] | "US ZIP Code Tabulation Area (ZCTA) SCI pairs. Split into 10 shards" by first digit |
| gadm2.zip | pointer, 12 shards; size [not confirmed] | GADM 4.1 level 2 pairs |
| gadm1.csv | 272.8 MB | GADM level 1 pairs |
| country.csv | 0.56 MB | country pairs |
| The Social Connectedness Index.pdf | 5.4 MB | methodology |

**Definition** (methodology note, Johnston, Kuchler, Kulkarni, Stroebel 2026, https://pages.stern.nyu.edu/~jstroebe/PDF/JKKS_SCI.pdf): SCI(i,j) = Friendships(i,j) / (Pop_i × Pop_j), where Pop is the number of Facebook users. "where i = j, a small adjustment is made to the denominator". "The SCI is scaled within each dataset to lie between 1 and 1,000,000,000." Sample: users "active on the platform in the 30 days prior to January 25, 2026 and who had more than 20 Facebook friends". Home location is "derived from a proprietary model that combines multiple signals, including self-reported location information, IP addresses ... and patterns of on-platform activity".

**Licence.** HDX licence field: "Public Domain / No restrictions (CC0)". No sign-up on HDX [my reading]. Meta's own terms page for the dataset: [not confirmed]; the HDX field is the only licence statement I read.

**Disclosure control.** "Geographic units with fewer than 500 users are excluded from the data release. In addition, all published statistics are protected using µ-Gaussian Differential Privacy". So small-cell values carry injected noise; magnitude not stated.

**Traps.**
- **It is a relative probability, not a share of friends.** To turn SCI into "share of a region's friends who live in region j" one needs Pop_j in Facebook users, which is not published ("the number of Facebook users per county is not part of the public data release", Bailey et al. 2018 footnote 2). Substituting resident population is an assumption that must be stated.
- **Scaling is per dataset**, so values are not comparable between the NUTS3 file and the ZCTA file.
- "We do not provide older versions ... we do not recommend it being interpreted as a time series."
- Friendship links, not message or traffic volume. Each link weighted equally; a stale tie counts as much as a daily contact.
- Users with 20 or fewer friends are excluded; platform coverage differs by age and country.
- The diagonal (i = i) is present with an adjusted denominator: that is our only handle on "ties inside the same unit", and for a ZCTA that is thousands of people over tens of km², not 1 km.

**Cannot give.** Anything below ZCTA (US) or NUTS3 (EU): no building, street or 1 km grain. No traffic volume. No hour of day.

**Verdict: usable with caveats; first choice for B.** It gives a checkable distance-decay curve between small areas in both Germany (NUTS3, about 400 units) and the US (ZCTA). It cannot by itself price the same-building or same-tile share.

## B2. Published SCI analyses (constants with a table behind them). United States

- Bailey, Cao, Kuchler, Stroebel, Wong 2018, JEP 32(3) (https://pages.stern.nyu.edu/~jstroebe/PDF/JEP_SCI.pdf): "For the average county, 62.8 percent of all friendship links are to individuals living within 100 miles", 5th to 95th percentile 46.0 to 76.9. "For the average (population-weighted) US county, 55.4 percent of friends live within 50 miles, with a 10–90 percentile range of 42.5 to 67.4 percent; and over 70 percent of friends live within 200 miles". Elasticity of friendship links to distance: −1.48 overall with county fixed effects ("a 10 percent increase in the distance ... 14.8 percent decline"); "−1.99" for county pairs under 200 miles; "−1.16" beyond 200 miles.
- Bailey, Farrell, Kuchler, Stroebel 2020, J. Urban Economics 118 (https://pages.stern.nyu.edu/~jstroebe/PDF/BFKS_SocialConnectednessUrbanAreas_JUE.pdf): "For residents of the median zip code, 29.0% of U.S.-based friends live within 5 miles ... from 19.5% to 39.6% between the 5th and the 95th percentiles"; "22.0% of U.S.-based friends live among the nearest 1 million people". Within NYC, "an elasticity of social connectedness to distance of -0.87"; "to travel time is -1.42". 182 NYC zip codes, 1,181 in the New York CSA.

Trap: the within-city elasticity (−0.87) is far shallower than the between-county one (−1.99 under 200 miles). **One exponent does not hold across scales**, which is the programme's usual fault in another coat: a constant is a price in a regime.

## B3. Social contact surveys (POLYMOD, CoMix and successors). Europe

**What is published.** Zenodo community `social_contact_data`, 48 records (https://zenodo.org/communities/social_contact_data). Standard file set per survey: participant, contact, household, survey-day CSVs plus a dictionary. Each contact row carries age, duration, frequency, physical or not, and the setting (home, work, school, transport, leisure, other) [my reading for the column set; the dictionary files were not opened].

| Record | Licence (Zenodo) | Total size |
|---|---|---|
| POLYMOD (Mossong 2008; 8 countries incl. Germany), record 3874557 | **cc-by-nc-4.0** | 7.8 MB (contact file 6.5 MB) |
| CoMix UK, 13684044 | cc-by-4.0 | 214 MB |
| CoMix 2.0 (multi-country), 7331926 | cc-by-4.0 | 85.7 MB, 44 files |
| CoMix Netherlands / Belgium / Austria / … | cc-by-4.0 | 4 to 49 MB each |
| Reconnect (UK, 2026), 21218377 | cc-by-4.0 | 13.8 MB |
| France, UK (older), Hong Kong, Peru, Vietnam, Zimbabwe | cc-by-nc-nd-4.0 | small |

No sign-up. A CoMix Germany record was not in the listing [not confirmed whether one exists elsewhere].

**Traps.**
- **POLYMOD, the only one with German respondents that I found, is non-commercial.**
- These are face-to-face contacts on one day, not social ties and not messages.
- **Setting, not distance.** "Home / work / school" tells us the place type; distance from home is not a standard column. Surveys that did record distance (Danon et al. 2013 Great Britain; Read et al. 2014 southern China; BBC Pandemic) are not in the files I listed [my reading].
- CoMix was run during pandemic restrictions.

**Verdict: usable with caveats, as a reference for the share of daily in-person contacts that occur at home by household size** (which bridges to A and to the household generator), not as a distance law. POLYMOD not usable commercially.

## B4. Call-detail-record datasets

No open CDR dataset with pairwise ties and home locations at small-area grain was found; the published studies used operator data under agreement. **Not usable as data; usable as literature constants** (list at the end).

## B5. Local share of traffic or messaging

No open dataset or paper giving "share of messages to the same city / same km" was found. What exists is country-level: the Internet Society "50/50" methodology measures the share of content served from in-country servers or caches (https://pulse.internetsociety.org) [search summary: Malawi 3 %, Algeria 13 %, Egypt 15 %, Thailand 44 % local]. That measures where servers sit, not where correspondents sit. **Not usable for the locality dividend; flag as a gap the theory must carry as an assumption.**

---

# C. DAILY MOBILITY

## C1. LEHD Origin-Destination Employment Statistics (LODES) 8.5. United States

**What is published.** "Data files are state-based and organized into three types: Origin-Destination (OD), Residence Area Characteristics (RAC), and Workplace Area Characteristics (WAC), all at census block geographic detail." "Version 8 of LODES was enumerated by 2020 census blocks." https://lehd.ces.census.gov/data/ . Tech Doc 8.5 (dated 2026-10-28, https://lehd.ces.census.gov/data/lodes/LODES8/LODESTechDoc8.5.pdf): years 2002-2024.

OD file columns: `w_geocode` (15-character workplace block), `h_geocode` (residence block), `S000` total jobs, then age, earnings and industry segments. File name pattern `[ST]_od_[PART]_[TYPE]_[YEAR].csv.gz`; PART = "main" (home and work both in the state) or "aux" (work in the state, home outside); TYPE JT00 all jobs, JT01 primary jobs, JT02-JT05 private and federal variants. Crosswalk `[st]_xwalk.csv.gz` maps blocks to tract, county, ZCTA, place and more.

**Sizes (HEAD requests).** ca_od_main_JT00_2022.csv.gz 96.4 MB; 2023 97.3 MB; 2024 98.1 MB; tx 2022 70.3 MB; ri 2022 2.1 MB; ca_xwalk.csv.gz 9.3 MB. A national OD set is on the order of 1 GB compressed per year [my reading, extrapolating from three states].

**Licence.** US Census Bureau product; requested citation form is given on the data page. Explicit licence wording: [not confirmed]; US federal government works are public domain [my reading]. No sign-up.

**Disclosure control and traps.**
- "The U.S. Census Bureau reviewed this data product for unauthorized disclosure of confidential information" (Tech Doc). The protection method is not described in the Tech Doc. [my reading, from Census OnTheMap documentation not re-read today: workplace counts get noise infusion and the residence side of OD pairs is synthetic, so a single block-to-block cell is not a real count. Aggregate to tract or to distance bands before use.]
- **Jobs, not people, and not trips.** A job held is counted whether or not the worker travels that day. Remote work is invisible.
- Coverage is wage-and-salary jobs under state unemployment insurance plus federal jobs from 2010; self-employed are out [my reading].
- **"2022-2024: 49 [states] — Alaska, Michigan"** have no OD data in those years; earlier years miss other states (Massachusetts, Mississippi and more).
- Workplace is the employer's reported establishment; multi-site employers can be mis-located [my reading].
- Only home-to-work. Nothing on school, shopping, social trips.

**Cannot give.** Share of people who leave home on a day; non-work mobility; time of day.

**Verdict: usable now; first choice for commuting-distance distribution in the US**, aggregated to our 1 km tiles or to distance bands.

## C2. BTS Daily Mobility Statistics ("Trips by Distance"). United States

**What is published.** Socrata dataset w96p-f2qv, https://data.bts.gov/Research-and-Statistics/Trips-by-Distance/w96p-f2qv . National, state and county rows by date; 6,142,062 rows. Columns: "Population Staying at Home — Number of residents staying at home, i.e., persons who make no trips with a trip end more than one mile away from home"; "Population Not Staying at Home"; "Number of Trips"; trips by ten distance bands (<1, 1-3, 3-5, 5-10, 10-25, 25-50, 50-100, 100-250, 250-500, ≥500 miles). "Trips were defined as movements that included a stay of longer than 10 minutes at an anonymized location away from home."

**Licence.** "Public Domain U.S. Government" (https://www.usa.gov/government-works). No sign-up. File size: [not confirmed] (about 6.1 million rows of 22 columns, several hundred MB as CSV by my estimate).

**Traps.**
- "The Daily Mobility Statistics data on this page, which cover the COVID and Post-COVID periods, are experimental ... they [do] not meet all BTS data quality standards."
- Source is "a data panel constructed from several mobile data providers", weighted to population. Device-panel bias is not quantified on the page.
- "no data were reported for a county if it had fewer than 50 devices in the sample on any given day."
- Rows were last updated 2024-04-30 (catalogue timestamp); the series appears to have stopped [my reading].
- **The stay-home threshold is one mile, not our 1 km tile**, and a person who walks 800 m to a shop counts as staying home.

**Verdict: usable with caveats; first choice for "share who leave the home tile on a day" in the US** because the definition is almost exactly ours and it comes by county and by day of week. Pre-2020 months give the non-pandemic baseline.

## C3. NHTS 2022 (NextGen). United States

User's Guide V2 (https://nhts.ornl.gov/assets/2022/doc/2022%20NextGen%20NHTS%20User's%20Guide%20V2_PubUse.pdf): "all trips taken within a 24-hour period by all household members aged 5 or older"; travel day begins at 4 AM; fielded January 2022 to January 2023. Table 10: **7,893 households, 16,997 persons, 31,074 trips**. "Overall Response Rate 10.8%". Estimates designed for "the Nation, census division, urban, and rural geographies". Version 2.0.1 fixed `VMT_MILE` [search summary]. Formats CSV/SAS [search summary]. Licence wording: [not confirmed]; FHWA public-use file, no sign-up [my reading].

Trap: the 2022 sample is about one sixteenth of 2017's (the Guide's comparison table shows earlier waves with far larger samples). Trip-distance distributions by density class will be thin.

**Verdict: usable with caveats; runner-up for the US**, mainly for trip-length distribution and time-of-day of trips, where LODES and BTS are silent.

## C4. Mobilität in Deutschland (MiD) 2023. Germany

**What is published.** https://www.mobilitaet-in-deutschland.de/publikationen2023.html . Study summary: realised sample "218.101 Haushalte mit insgesamt 420.979 Personen, die für ihren zugewiesenen Stichtag insgesamt 1.087.393 Wege berichteten" (218,101 households, 420,979 persons, 1,087,393 trips on the assigned day), field phase April 2023 to July 2024, sample mostly from population registers.

Key figure: "Die Mobilitätsquote (definiert als der Anteil der Personen, die unterwegs sind) ... 2008 im Schnitt noch 90 Prozent und 2017 insgesamt 85 Prozent ... erreicht dieser Wert 2023 nur noch 82 Prozent." (The mobility rate, the share of persons who are out and about: 90 % in 2008, 85 % in 2017, 82 % in 2023.) Trips per person per day 3.3 (2002), 3.4 (2008), 3.1 (2017), 2.9 (2023).

| File | Size (HEAD) |
|---|---|
| MiD23_SAE_…_Gemeindedaten.xlsx (small-area estimates by municipality) | 412 KB |
| MiD23_SAE_…_Kreisdaten.xlsx (by district) | 114 KB |
| infas_Tabellen_MiD2023_7555_Bund.pdf (table volume) | 1.7 MB |
| MiD2023_SAE-Bericht.pdf | 1.5 MB |
| MiD2023_Ergebnisbericht.pdf | 3.5 MB |

The small-area report lists the modelled indicators: "Mobilitätsquote", "Wege pro Person (und Tag)", "km pro Person (Tagesstrecke)", and two modal splits, for districts and for "Gemeinden ab 10.000 Einwohnern" (municipalities of 10,000 inhabitants and more).

**Licence.** Not stated on the publications page: [not confirmed]. Microdata: "Die Daten zur aktuellen Studie sind ausschließlich über den MobilityData-Campus der BASt verfügbar" (available only through the BASt MobilityData-Campus); terms and cost [not confirmed]. Interactive tables: "Mobilität in Tabellen" at https://mobilitaet-in-tabellen-2023.bast.de/ .

**Traps.**
- The municipality figures are **model estimates, not direct survey estimates**: of 1,287 municipalities of 10,000+ in the person sample, "In 596 dieser 1.287 Gemeinden liegt die Fallzahl unter n=50 Personen". Only 2,889 of nearly 11,000 municipalities are in the sample at all.
- "unterwegs" means made at least one trip, of any length. It is not "left the 1 km tile".
- The table volume is a PDF, not a spreadsheet.
- Trips inside Germany only.

**Verdict: usable now for the headline rate and the municipality/district spreadsheets (licence to be confirmed before redistribution); first choice for "share who leave home" in Germany.** Trip-length distribution by density class needs the table volume PDF or the BASt microdata.

## C5. Pendlerrechnung der Länder / Pendleratlas. Germany

**What is published.** Quality report dated 2025-10-23, https://www.statistikportal.de/sites/default/files/2025-10/Qualit%C3%A4tsbericht_PendL_2025-10-23.pdf : "Ergebnisse für alle Länder auf Ebene der Gemeinden (LAU ...) mit Ausnahme von Mecklenburg-Vorpommern, Rheinland-Pfalz, Schleswig-Holstein und Thüringen (Ergebnisse auf Ebene der Gemeindeverbände)". Content: "Mobilitätsströme ... einschließlich der Luftlinienentfernungen zwischen Arbeits- und Wohnort" (flows including straight-line distances between workplace and home); counts by sex, age class, employment status, hours, sector; "Tagesbevölkerung, Pendelquoten und Pendelsaldo" (daytime population, commuting rates, balance). Reference date 30 June each year; annual from reporting year 2021. Sources: BA employment statistics, civil-service personnel statistics, Microcensus for the self-employed.

Distribution: "Regionaldatenbank Deutschland (GENESIS-Online) https://www.regionalstatistik.de/genesis/online/ > zum Thema > 193" and https://pendleratlas.statistikportal.de . "Kostenfreier Download" (free download).

**Licence.** Statistikportal open-data page: "Die statistischen Daten können unter den Bedingungen der Datenlizenz Deutschland 2.0 weiterverwendet werden" (https://www.statistikportal.de/de/open-data). That this applies to table 193 specifically: [my reading]. Whether the Regionaldatenbank needs an account for large table downloads: [not confirmed].

**Disclosure control and traps.**
- "primäre Geheimhaltung (Sperrung bei Fallzahl kleiner 3) und entsprechende sekundäre Geheimhaltung" (cells under 3 suppressed, plus secondary suppression). Municipality-to-municipality flow tables will have many blanks.
- **Potential, not daily, mobility**: "Der Pendelweg muss nicht zwangsläufig täglich, sondern kann wochenweise oder an einzelnen beziehungsweise gar keinen Tagen zurückgelegt werden" (the commute need not be daily; it may be weekly, on single days or on no days at all). Home working is invisible.
- Nothing below the municipality: "Personen mit Wohn- und Arbeitsort auf demselben Grundstück ... werden ... als innerörtlich Pendelnde eingestuft" (people living and working on the same plot are classed as within-municipality commuters). A municipality can be hundreds of km².
- Employers may report all staff at the head office.
- Cross-border commuters are undercounted [search summary].

**BA alternative.** The Bundesagentur für Arbeit publishes "Pendlerverflechtungen der sozialversicherungspflichtig Beschäftigten" by district and municipality at 30 June, and its own Pendleratlas [search summary; the BA pages returned nothing to my reader]. BA terms require approval for commercial use other than press and scientific publication [search summary]. **Prefer the Länder product for licence reasons.**

**Verdict: usable with caveats; first choice for commuting flows and commuting-distance distribution in Germany**, at municipality grain only.

## C6. Not recommended for C

- **Google COVID-19 Community Mobility Reports**: "no longer being updated as of 2022-10-15"; historical data remain; differential-privacy noise; values are percent change from a January-February 2020 baseline, not levels (https://www.google.com/covid19/mobility/). Licence: Google Terms of Service linked; CC BY [not confirmed]. Not usable: no levels, pandemic period only.
- **Apple Mobility Trends**: ended 14 April 2022; no longer hosted by Apple [search summary]. Not usable.
- **Destatis experimental mobility indicators from mobile network data**: "The indicators will no longer be updated from 1 January 2023"; change against 2019 by district; source Teralytics (https://www.destatis.de/EN/Service/EXSTAT/Datensaetze/mobility-indicators-mobilephone.html). Not usable: relative change only, discontinued.
- **OpenCelliD**: cell-tower locations, CC BY-SA (3.0 or 4.0, sources disagree), free API key needed for the database download [search summary]. Tower positions say nothing about presence, ties or mobility; share-alike would bind derived data. Not usable for A, B or C.

---

# RANKED RECOMMENDATION

## A. Presence by hour of week
1. **First choice, Germany: ZVE 2022 Public Use File.** Four files (household, person, time-slot, sums); data size not stated before registration; documentation about 1.2 MB of dictionary and lists. Gives presence at home, at work, in transit, asleep, device use, by household size and weekday, in 10-minute slots. Needs a free registration, which only the lead or the user can authorise.
   **First choice, United States: ATUS 2003-2024**, activity + respondent + roster = **95.6 MB**, no sign-up, public domain. Sleep location must be imputed; no device flag.
2. **Runner-up and no-sign-up check: Eurostat `tus_20startime` + `tus_20loc` + `tus_20week`** (about 43,000 + 3,300 + 32,000 values; under 1 MB each). Activity by slot for Germany 2022, without weekday, household size or location by slot.

My recommendation: compute the US profile from ATUS first (nothing blocks it), stake the German profile's shape against `tus_20startime` before opening anything, and ask the user whether to register for the PUF.

## B. Ties by distance
1. **First choice: Meta SCI 2026**, `nuts_2024.zip` **14.2 MB** for Germany (NUTS3) and `us_zcta.zip` shards for the US (size not stated; `us_counties.csv` 245 MB as fallback). CC0 on HDX. Gives distance decay between small areas and a within-unit diagonal.
2. **Runner-up: literature constants** for the sub-ZCTA range (Backstrom 2010 at 0.1-mile grain; Bailey 2020 "29.0 % within 5 miles"), with **CoMix (CC BY 4.0)** as the reference for the at-home share of daily in-person contacts.

Honest limit: no source measured the same-building or same-1-km share. The locality dividend at tile grain will rest on an extrapolation of a curve measured from 0.1 mile upward on 2010 Facebook data, and that must be written as an assumption with its regime named.

## C. Daily mobility
1. **First choice, United States: LODES 8.5 OD** (one state: 2 to 98 MB compressed, plus crosswalk 9 MB) for commuting distance, with **BTS Daily Mobility Statistics** for the stay-home share (one file, 6.1 million rows).
   **First choice, Germany: MiD 2023** (municipality xlsx 412 KB + district xlsx 114 KB + table volume PDF 1.7 MB) for the share out on a day, with **Pendlerrechnung der Länder** (Regionaldatenbank theme 193) for commuting flows and distances.
2. **Runner-up: NHTS 2022** (7,893 households) for trip length and trip start time in the US.

---

# LITERATURE CONSTANTS (priors where no table exists)

| Value | Paper | Population measured | Note |
|---|---|---|---|
| P(friend at distance d) ∝ a(b + d)^−c with c ≈ 1; "At shorter scales the curve is flatter" than d^−1 | Backstrom, Sun, Marlow, WWW 2010, "Find me if you can" (https://archives.iw3c2.org/www2010/proceedings/www/p61.pdf) | US Facebook users with home addresses: about 6 % of roughly 100 million; 2.9 million geolocated; 0.1-mile buckets | The only measurement reaching below 1 mile. By rank the exponent is also about −1; people in low-density areas are more likely to know someone a few miles away; curves by density converge at about 50 miles |
| P(d) ∝ ε + 1/d; "just over two-thirds of the friendships ... are generated by geographic processes"; "about one-third ... independent of geography"; 2.5 of about 8 friends non-geographic | Liben-Nowell, Novak, Kumar, Raghavan, Tomkins, PNAS 2005 (https://pmc.ncbi.nlm.nih.gov/articles/PMC1187977/) | 495,836 US LiveJournal bloggers, February 2004, city-level locations, 10 km rounding | City grain only: "our data do not allow conclusions about routing within a city". The "d^−1.2" often quoted was not found in the passages I read [not confirmed] |
| P(link) ∝ d^−2 (gravity form) | Lambiotte et al., Physica A 387 (2008) 5317 [search summary] | Belgian mobile operator, 2.5 million customers, 810 million calls and texts over 6 months, billing-address location | Preprint gives 3.3 million customers; the two versions disagree |
| Tie probability ∝ d^−α, α ≈ 1.58 (voice), ≈ 1.49 (text), fitted below 800 km; tie strength "varies only weakly with distance" | Onnela, Arbesman, González, Barabási, Christakis, PLoS ONE 2011 [search summary] | Mobile phone records, one unnamed European country | Useful second point: who you call depends on distance, how much you call them barely does |
| Elasticity of friendship links to distance −1.99 (under 200 miles), −1.16 (over 200 miles), −1.48 overall; 55.4 % of friends within 50 miles, 62.8 % within 100 miles | Bailey et al., JEP 2018 | US Facebook, county pairs, 2016 snapshot | Read today from the paper |
| 29.0 % of US-based friends within 5 miles (5th-95th percentile 19.5-39.6 %); 22.0 % among the nearest 1 million people; elasticity to distance −0.87, to transit time −1.42 | Bailey, Farrell, Kuchler, Stroebel, J. Urban Econ. 2020 | Facebook, 182 NYC zip codes | A dense city; do not carry to villages |
| "more than 60 percent of ties" between people living under 10 km apart | Büchel and von Ehrlich, CESifo WP 6568 (later J. Urban Econ. 2020) [search summary] | Swiss mobile operator, about 15 million calls | Ties, not call volume |
| Mean distance between friends 79.9 km against 531.2 km for random pairs; distance constrains who interacts, "not the intensity" | Kaltenbrunner et al. 2012 / Laniado et al., Inf. Syst. Front. 2018 [search summary] | Tuenti (Spain), about 10 million users, city-level | Same-city pairs are distance zero |
| E-mail contact "generally insensitive to distance"; face-to-face contact tied to within 5 miles; phone little affected within 100 miles | Mok, Wellman, Carrasco, Urban Studies 47(13) 2010 [search summary] | Personal networks in East York, Toronto, compared with 1978 | Small survey; the only source separating medium of contact |
| 82 % of people out on an average day (85 % in 2017, 90 % in 2008); 2.9 trips per person per day | MiD 2023 study summary | Germany, 420,979 persons | Read today |

Three cautions on using these as priors:
1. The exponent runs from about −0.9 (inside New York) through −1 (Facebook addresses) and −1.5 (calls) to −2 (Belgian calls; counties under 200 miles). The spread is the finding. Pick the one whose population and scale match the claim and name it.
2. All of them are friendship or call ties among platform or operator users. None is traffic volume.
3. Every one predates or excludes the 1 km tile. Onnela's weak dependence of tie strength on distance is the best available argument for turning a tie share into a traffic share, and it is a single study.

---

# Open items for the lead

- Registration for the ZVE 2022 PUF is the one action that would most improve A for Germany. It is a sign-up, so it is the user's call.
- Licence wording for derived statistics is unconfirmed for: ZVE PUF, MiD 2023 publications, NHTS 2022, LODES (assumed public domain).
- Not checked: German Zensus 2022 commuting outputs, US ACS/CTPP commuting flows (lead said ACS is known), UK and other national time-use microdata beyond a note.

---
---

# B-ADDENDUM (2026-11-11): the tie-kind × distance-band table, source by source

Written against the lead's sharper specification. Same rules and tags as above. New documentation fetched for this part: SHARE wave 7 generic showcards PDF, SHARE conditions of use and registration pages, POLYMOD data dictionary (42 KB xls, read as strings), Pew 2009 report pages and Pew terms, HDX metadata for the Social Capital Atlas, DIW/SOEP data-access page, GGP data page, GSS download page. gesis.org and understandingsociety.ac.uk returned nothing to my reader ("You are unable to access gesis.org"), so ISSP and Understanding Society facts are search summaries.

## What the table can and cannot be filled with

**No single source fills the table.** The honest state, kind by kind:

| Kind of tie | Best source for share by distance band | Best source for contact frequency by mode | State |
|---|---|---|---|
| Own devices (self) | none found | none found | **Gap.** Distance is zero by definition; the open question is count of devices, which the ZVE PUF household file has (`hb132bx` computers, `hb133bx` smartphones/tablets) |
| Same household | by definition; household generator already built | Pew 2009: in person 359 of 365 days, mobile voice 276 of 365 days for core ties in the same home | Filled for core ties, US 2008 |
| Kin in other households, incl. abroad | SOEP (7 bands, by kin type, Germany); SHARE (8 bands, children and network members, age 50+); GSS 1986 (travel time to mother etc., US) | SHARE (frequency, modes lumped); GSS 1986 (visits and phone/letter separately); Pew 2009 (by mode and distance) | Best-covered kind. German sources need a contract |
| Close friends | Understanding Society (4-5 bands, 3 closest friends, UK); Bailey 2020 (NYC, all Facebook friends); SCI (all Facebook friends, area pairs) | Pew 2009; Understanding Society (how often see) | Covered for UK and US; Germany only through SCI |
| Work colleagues | no tie-level source; proxy is the commuting-distance distribution (LODES, Pendlerrechnung), since colleagues are met at the workplace | ATUS who-present (in person only); CoMix/POLYMOD "contact at work" | **Proxy only** |
| School class | none found; proxy is school catchment size | CoMix/POLYMOD "contact at school" | **Gap, proxy only** |
| Neighbours | by definition within the tile | Pew 2009: 61 % talk face to face with neighbours at least monthly, 25 % phone, 10 % of internet users email | Frequency filled, US 2008 |
| Interest / community groups | none found on where fellow members live | none found | **Gap.** The Social Capital Atlas has civic-engagement rates by ZIP but not member distances |
| Public / broadcast follows | none found as a table; literature only (see constants) | n/a | **Gap** |

Every source lumps at least two of the lead's kinds. The usual lumps: "friends" without separating close from other; "other relative"; colleagues folded into "non-kin"; groups and follows absent.

## Band mapping at a glance

Lead's bands: same household | same building | within about 1 km | same city (under 25 km) | same region (under about 200 km) | elsewhere in country | abroad.

| Source | Its own bands | Maps to ours? |
|---|---|---|
| **SHARE** | 1 In the same household; 2 In the same building; 3 Less than 1 kilometre away; 4 Between 1 and 5 kilometres away; 5 Between 5 and 25 kilometres away; 6 Between 25 and 100 kilometres away; 7 Between 100 and 500 kilometres away; 8 More than 500 kilometres away (showcard, read today) | **Nearly exact.** First four of ours map one to one (4 + 5 = same city). 100-500 km straddles our 200 km line. Abroad is a separate code only in some waves [search summary] |
| **SOEP** | In the same household; In the same house; In the same neighborhood; In the same town, but more than 15 minutes away by foot; In another town, but within a one hour drive; Farther away (but in Germany); Abroad [search summary of paneldata.org] | **Good.** Household, building, neighbourhood (about 1 km), town, region (one-hour drive), country, abroad. Bands are verbal, not kilometres |
| **pairfam** | same house; under 10 minutes; 10 to under 30 minutes; 30 minutes to under 1 hour; 1 to under 3 hours; 3 hours or more [search summary] | Travel time; needs a speed assumption. No "abroad" |
| **GSS 1986 (ISSP 1986)** | travel time door to door, first band "LT 15 MIN" [search summary]; remaining bands [not confirmed; by my recollection 15-30 min, 30-60 min, 1-2 h, 2-3 h, 3-5 h, 5-12 h, over 12 h] | Travel time; coarse below 15 minutes, so no building or 1 km band |
| **Understanding Society** | closest friends: less than 1 mile; less than 5 miles; 5 to 50 miles; over 50 miles; a later version adds "over fifty miles but still within the UK" and another country [search summary] | **Fair.** 1 mile ≈ our 1 km band (too wide by 60 %); under 5 miles ≈ city; 5-50 miles ≈ region; rest of UK; abroad. No building band |
| **Pew 2009** | same home; same block or street; 1-5 miles; …; 50-100 miles; 50-500 miles; 500-3000 miles; more than 3,000 miles (bands as they appear in the report text; the full list is [not confirmed]) | **Fair.** "Same block or street" is the only source with a sub-1-km band other than SHARE and SOEP |
| **Bailey 2020** | within 5 miles; within 10 miles; nearest 1 million people | City band only |
| **Bailey 2018** | within 50, 100, 200 miles | Region band only |
| **SCI 2026** | area pairs: ZCTA (US), NUTS3 (EU), GADM2 | Bands are built by us from centroid distances; floor is the area's own size |
| **CoMix / POLYMOD** | place type: home, work, school, transport, leisure, other | **No distance.** Home = same household or visitors |
| **ATUS who-present** | place type (TEWHERE) | **No distance** |

---

## Tier 1: open, no application

### T1-a. General Social Survey (US), network modules

- **Distance or travel time to named contacts?** Yes in 1986 only. `MATIME` is door-to-door travel time to where the respondent's mother lives; first band "LT 15 MIN" (27.3 %, 215 cases in the codebook frequency) [search summary of the SDA codebook mirror and gssrdoc]. The 1986 ISSP module asks the same for other relatives and for the best friend [my reading; the best-friend mnemonics were not found by the search].
- **Frequency by mode?** 1986: `MAVISIT` (how often sees or visits; includes "lives in the same household", daily, several times a week …) and `MACALL` (contact "by telephone or letter", daily = 14.9 %) as separate items [search summary]. 1985 and 2004: `TALKTO1-5`, "Do you talk to (him/her) almost every day, at least once a week, at least once a month, or less than once a month?" for up to five confidants; all modes lumped [search summary].
- **Kinds separated.** 1985/2004 name generator: spouse, parent, sibling, child, other family, co-worker, member of group, neighbour, friend, adviser, other [my reading; the search did not return the relationship codes]. This is the only source I found that has **"member of a group"** and **co-worker** as tie kinds, but it records no distance for them.
- **Access.** gss.norc.org "Download the Data" in Stata and other formats; no registration mentioned on the page read today. Licence wording [not confirmed].
- **Traps.** 1986 is forty years old and predates mobile phones; phone and letter are one category. Confidants ("discuss important matters") are a mean of about two people, not a person's whole tie set. The 2004 wave has a known interviewer/coding dispute about the share with zero confidants [my reading].
- **Verdict: usable now** for kin and best-friend travel time and visit/call frequency as a dated US prior, and for the kind mix of the core network.

### T1-b. Pew "Personal Networks and Community" survey, published report (US, 2008 fieldwork)

- Report: Hampton, Sessions, Her, Rainie, "Social Isolation and New Technology", 4 Nov 2009, https://www.pewresearch.org/internet/2009/11/04/social-isolation-and-new-technology/ and its part 2 page.
- **Distance to named contacts?** Yes: each core tie has a distance band (see mapping). The report states frequencies by band, not the share of ties per band; that share needs the dataset (tier 2).
- **Frequency by mode? Yes, and this is the bridge to messages.** "we asked participants how many days per month they were in contact with each of their core ties using a variety of media, including face-to-face". Read today:
  - All core ties, mean days per year: in person 210; mobile phone 195; text messaging 125 (tied with landline); email 72; instant messaging 55; social networking sites 39.
  - In person by distance: "from nearly daily contact for those with whom a person shares a home (359/365 days), to less than one-third as often for core ties who live 50-100 miles away (107/365 days)".
  - Landline: 173 days for ties "on the same block or street", 82 for 50-100 miles.
  - Text: "137/365 days for those who live 1-5 miles away; it drops sharply ... 69/365 days for those 50-500 miles away".
  - Mobile voice: 276 days within the same home, 138 at 50-100 miles.
  - Email: "81/365 days per year for core ties within 1-5 miles, and 73/365 days for core ties who are 500-3000 miles away". Social networking sites: 48 and 43. IM: 72 and 55.
  - Neighbours: 61 % talk face to face at least monthly, 25 % phone, 10 % of internet users email; 40 % know all or most neighbours, 31 % know none.
- **Kinds.** Core ties "may include spouses and household members, extended family, workmates, neighbors, and other friends"; the report separates kin from non-kin (70 % have a kin confidant, 45 % a non-kin one).
- **Reading for us:** texting falls by half between 1-5 miles and 50-500 miles; email, IM and social-network contact are nearly flat in distance. **A message weight is therefore mode-specific**, and a single distance law for "messages" would be wrong.
- **Traps.** 2008: before smartphones and messaging apps were common, so the mode mix is obsolete even if the distance gradients are informative. Core ties only (a handful per person). Days of contact, not message counts. Sample size [not confirmed].
- **Verdict: usable now as published constants**; the only source found with frequency by mode by distance.

### T1-c. Meta Social Connectedness Index 2026 (tie-level proxy by area)

- **Smallest grain:** US ZCTA pairs; EU NUTS3 pairs; GADM level 2 elsewhere.
- **Licence:** "Public Domain / No restrictions (CC0)" (HDX field).
- **Noise:** "Geographic units with fewer than 500 users are excluded ... all published statistics are protected using µ-Gaussian Differential Privacy"; noise scale not stated.
- **Kinds:** none separated. All Facebook friendships, each weighted equally, among users with more than 20 friends. Kin, colleagues, classmates, group co-members and acquaintances are lumped.
- **Distance to named contacts / frequency:** neither.
- **Verdict: usable now** as the all-ties distance profile from about 5 km upward in both countries; it cannot separate kinds and cannot see inside a ZCTA or Kreis.

### T1-d. Social Capital Atlas (Meta / Opportunity Insights), US

- HDX metadata: counties CSV 0.72 MB, **ZIP codes CSV 4.05 MB**, high schools CSV 2.9 MB, colleges CSV 0.52 MB, README.pdf, LICENSE.pdf; data date 2022-08-01. Three families of measures: connectedness, cohesiveness (clustering, support ratio), "Civic Engagement - Indices of trust or participation in civic organizations. This includes our volunteering rate measure." Noise: "methods from the differential privacy literature". Users aged 25-44 [search summary].
- Licence field on HDX: "Other" (see LICENSE.pdf, **not read**). So commercial use is [not confirmed].
- **Does it say whether group members live near each other?** No. It gives rates of group participation and how clustered friendships are **within a ZIP**, plus per-high-school measures. The high-school file is the nearest thing to a "school class" source, but it has no distance.
- **Verdict: usable with caveats** as a per-ZIP covariate (how locally clustered ties are); not a distance table.

### T1-e. CoMix / POLYMOD contact surveys

- POLYMOD dictionary (read today): per contact, `cnt_home`, `cnt_work`, `cnt_school`, `cnt_transport`, `cnt_leisure`, `cnt_otherplace` (place flags), `phys_contact`, `duration_multi` ("Total duration of contact with reported person during the whole day"), `frequency_multi` ("Frequency of contacting the reported contact": 1 daily, 2 weekly, 3 monthly, further codes), contact age and gender; `hh_size`.
- **Distance to contacts?** No. **Kind of tie?** No relationship field in POLYMOD; kind must be inferred from place (home ≈ household, work ≈ colleague, school ≈ classmate). CoMix adds a household-member flag [my reading].
- **Frequency by mode?** In-person conversation or touch only.
- Licence: POLYMOD CC BY-NC 4.0; CoMix and Reconnect CC BY 4.0.
- **Verdict: usable with caveats** for the in-person side of colleagues and classmates, where nothing else exists: number of distinct people met at work and at school per day, and how often each recurs.

### T1-f. ATUS who-present

- `TUWHO_CODE` (dictionary read earlier): alone; spouse; unmarried partner; own household child; grandchild; parent; brother/sister; other related person; housemate; parents not in household; other non-household family under 18 / 18 and older; **friends; neighbours/acquaintances;** co-workers/colleagues/clients (to 2009), then boss or manager, people whom I supervise, co-workers, customers (from 2010).
- **Distance?** No; only the place type of the activity. **Frequency by mode?** In-person minutes; phone calls appear as an activity without the other party's kind [my reading].
- **Verdict: usable now** for minutes per day in the presence of each kind, by hour. It is the best open separation of household / kin / friends / neighbours / colleagues, in person only.

### T1-g. Published constants from area data (Bailey 2018, 2020)

Already in the constants table: 55.4 % of friends within 50 miles (US counties); 29.0 % within 5 miles (NYC ZIPs). All friends lumped.

---

## Tier 2: free, but needs registration or an agreement (the project lead decides)

Listed with exactly what each asks.

### T2-a. Pew 2009 survey dataset
- **Asks:** a free Pew account. Dataset terms (read today, https://www.pewresearch.org/about/terms-and-conditions/): "a nonexclusive, non-sublicensable, non-transferable, revocable, worldwide, and royalty-free license to access, copy, reproduce, cite, link, display, download, distribute, broadcast, transmit, publish, modify, create derivatives of, or otherwise exploit the survey datasets". Commercial use is not excluded in the passage read; the full clause was truncated in my extract.
- **Gives:** share of core ties per distance band by kind, and days of contact per mode per tie. Whether this particular 2008 dataset is still downloadable: [not confirmed].
- **Verdict:** lowest-friction registration, and it would turn T1-b from constants into a table.

### T2-b. SHARE (Survey of Health, Ageing and Retirement in Europe), incl. Germany
- **Distance:** the eight-band card above, for each child (`ch007` [search summary]) and for each social-network member (`sn006` in the SN module [search summary]).
- **Frequency:** contact with each child and network member on a seven-step scale from daily to never, all modes lumped ("in person, by phone, mail, email ..." [my reading]); parents are assumed in daily contact with co-resident children [search summary].
- **Kinds:** SN module records the relationship of up to seven confidants (spouse, child, sibling, friend, neighbour, colleague and more) [my reading].
- **Asks (read today):** registration plus a **signed SHARE User Statement** sent to the data centre. "the SHARE data may be used for scientific purposes only". Applicants must show "an affiliation with a scientific institution" or, "in justified cases", details of "the specific scientific project". "Users are not allowed to make copies of the data available to others". Free of charge.
- **Traps:** respondents are aged 50 and over. Scientific-use-only is likely incompatible with a commercial product; derived aggregates may be publishable, but that is for the lead to judge against the conditions.
- **Verdict:** the best band match of any source, and it covers Germany; usable only if the scientific-purpose condition can be met.

### T2-c. SOEP (German Socio-Economic Panel)
- **Distance:** for each kind of relative (partner, parents, step-parents, children, siblings, grandparents, aunts/uncles/nieces/nephews, other relatives): "how many such relatives you have, whether they live in your household, and if not, how far away they reside", seven verbal bands ending in "Abroad" [search summary of paneldata.org]. Asked in several waves including 2011 and 2016.
- **Frequency by mode:** [not confirmed].
- **Friends:** SOEP has asked about closest friends in some waves [my reading, not confirmed].
- **Asks (DIW page read today):** a signed **data distribution contract**, checked by DIW; applications come from a professor or project head at a research institution; each participating institute signs its own. Fees and commercial terms [not confirmed].
- **Verdict:** the best German kin-by-distance source, including relatives abroad, but the heaviest access route.

### T2-d. ISSP "Social Networks" modules (2017, ZA6980; 2001; 1986)
- **Asks:** GESIS registration ("registration is required for data download"; "free data download") [search summary].
- **2017:** 30 countries; position generator and resource generator; a section on "Social interaction in the social media era" (5 items in the drafting report) [search summary]. **Whether 2017 has a distance or travel-time item: [not confirmed]; I believe it does not** [my reading].
- **2001 and 1986:** travel time to mother, father, sibling, child, best friend, and frequency of visits and of other contact [my reading for 2001; 1986 confirmed through the GSS variables above]. Germany took part in 2001 (ZUMA report exists) [search summary].
- **Verdict:** 2001 is the most recent cross-national travel-time source with Germany and the US in one file; check the questionnaire before registering.

### T2-e. Understanding Society (UK)
- **Distance:** three closest friends, bands above; in the public variable pages, "under 1 mile is roughly 27–30 % of valid answers and under 5 miles about 25–26 %" for the first closest friend [search summary]. A family-network module records relatives outside the household and travel time [not confirmed].
- **Frequency:** "How often see 1st closest friend" exists [search summary]; phone/message frequency [not confirmed].
- **Asks:** UK Data Service registration and End User Licence [my reading; the access page returned nothing].
- **Trap:** filter on the close-friend items is an AND condition [search summary].
- **Verdict:** best close-friend distance bands; UK only. The published frequencies on the variable pages are usable as constants without registering.

### T2-f. pairfam (Germany) and Generations and Gender Survey
- pairfam: travel time to each parent and between parents and children, six bands [search summary]; contact frequency items exist [my reading]. Anchors were born 1971-73, 1981-83, 1991-93 [my reading], so it is not the whole population. Access through GESIS under a scientific-use agreement [my reading, not confirmed].
- GGS: "All datasets are freely available for scientific, non-commercial use via the Data Portal" (ggp-i.org, read today); registration "with an institutional email" and a signed data agreement [search summary]. Travel time to parents and meeting frequency [my reading; items not confirmed].
- **Verdict:** both are redundant with SOEP and SHARE for our purpose and both are non-commercial.

## Tier 3: paid

None needed. SOEP may charge outside academic use [not confirmed]. No paid source would fill the gaps (groups, follows, school class).

---

## Published distance-decay results for calls and messages

| Result | Paper | Population, year | Range over which it holds | Volume or ties? |
|---|---|---|---|---|
| Exponent 1.20 (email), 1.12 (Facebook), by maximum likelihood; cumulative email distances fit Zipf, R = −0.989 | Goldenberg and Levy 2009, arXiv 0906.3202 [search summary] | email and Facebook samples; sizes and country [not confirmed] | [not confirmed] | **Volume** of electronic communication |
| Link probability ∝ d^−2 | Levy and Goldenberg, Physica A 393 (2014), "The gravitational law of social interaction" [search summary] | Facebook, small-world experiment, email, interviews | [not confirmed] | Ties. **Same authors, exponent 2 versus 1.1-1.2 five years earlier**: different quantity (pair probability against volume by distance) |
| Number of conversations falls with distance with a bump near 500 km; "the number of exchanged messages remains constant before decreasing slowly"; longer links "carry more and longer conversations" | Leskovec and Horvitz, WWW 2008, "Planetary-scale views on an instant-messaging network" [search summary] | Microsoft Messenger, June 2006, world | world scale; no small-distance detail | **Messages per conversation**: the one direct look at message weight by distance, and it says weight per tie does not fall with distance |
| α ≈ 1.58 voice, 1.49 text; tie strength nearly flat in distance | Onnela et al. 2011 [search summary] | mobile operator, one European country | 1 km to below 800 km | Ties; strength separately |
| d^−2 | Lambiotte et al. 2008 [search summary] | Belgian mobile operator, 2.5 million customers, 6 months | [not confirmed]; commune-level billing addresses | Ties |
| ε + 1/d; one third geography-independent | Liben-Nowell et al. 2005 (read) | 495,836 US LiveJournal users, 2004 | 10 km buckets up to about 1,000 km, beyond which ε dominates | Ties |
| a(b + d)^−c, c ≈ 1; flatter at short scale | Backstrom, Sun, Marlow 2010 (read) | 2.9 million US Facebook users with addresses | 0.1 mile to continental; "medium to long-range" for the −1 | Ties |
| Texting 137 days/yr at 1-5 miles, 69 at 50-500 miles; email 81 and 73 | Pew 2009 (read) | US adults, 2008, core ties | same home to over 3,000 miles | **Frequency by mode** |
| Spatial distance constrains who interacts, "not the intensity" | Laniado et al. 2018 [search summary] | Tuenti, Spain, about 10 million users | city pairs | Both |

**No messaging-app study from 2018 onward with a fitted distance exponent was found.** A 2024 arXiv study of a global mobile social app reports that proximity still shapes link formation; details not read [search summary].

**The one consistent finding across the volume studies** (Onnela, Leskovec and Horvitz, Laniado, Pew for email and IM): distance decides *who* is a tie far more than *how much* is exchanged per tie, except for texting and voice, which Pew found do fall with distance. So the defensible construction is: ties by kind and band from the survey sources, then a per-mode weight, not one exponent applied to messages.

---

## Ranked recommendation for the part-B table

1. **Open now, no application.** Build the frame from:
   - ATUS who-present for the in-person minutes per kind (household, kin, friends, neighbours, colleagues);
   - GSS 1985/2004 for the kind mix of core ties and GSS 1986 for kin and best-friend travel time;
   - Pew 2009 published figures for frequency by mode by distance;
   - SCI 2026 (`nuts_2024.zip` 14.2 MB; US ZCTA shards) for the all-ties share by band from city scale outward;
   - commuting distance (LODES, Pendlerrechnung) as the colleague proxy;
   - CoMix for people met at work and school.
   Smallest set: GSS cumulative file (size not checked), ATUS who + activity + respondent (110 MB), `nuts_2024.zip`.
2. **One free registration that pays most:** the Pew 2009 dataset (account only, permissive licence). It yields ties per band by kind and days of contact per mode per tie in one file.
3. **For Germany, by kin and band:** SOEP (contract) or SHARE (signed statement, scientific use only, age 50+). Either needs the lead's decision; SHARE's bands match ours almost exactly, SOEP covers all ages and all kin types.
4. **Carry as stated assumptions**, because nothing measures them: own devices; school class; group co-members; broadcast follows; and the building band for non-kin.

Runner-up for close friends specifically: Understanding Society's published variable frequencies (UK), usable as constants without registration.
