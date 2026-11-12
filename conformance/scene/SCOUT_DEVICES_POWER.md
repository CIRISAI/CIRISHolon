# SCOUT: devices (D), home network (E), power (F), traffic and locality (G)

Scouted 2026-11-11. Documentation only: publishers' pages, data dictionaries, methodology notes, licence pages, catalogue metadata. No data file was downloaded and no data-value API was called. File sizes are from HEAD requests or from the publisher's page. Codebooks and PDFs that were opened are documentation, not data.

Marking convention. Unmarked statements were read on the publisher's own page at the URL beside them. `[search summary]` = seen only in a search engine's summary of the page. `[my reading]` = my inference or recollection, not on the page. `[not confirmed]` = I could not open the publisher's page.

Pages I could NOT open (bot walls), so everything about them is weaker than the rest:
- ofcom.org.uk (Cloudflare challenge, 403; only a 2024 archive copy of the copyright page and a 2026 archive copy of the Connected Nations landing page were readable)
- fcc.gov and help.bdc.fcc.gov (403); data.fcc.gov was readable
- energy-efficiency.us (403), ericsson.com PDF (blocked), cisco.com white paper (empty), onlinelibrary.wiley.com (empty), iea.org report pages (empty; terms page readable)
- JavaScript-only pages returned nothing: GENESIS-Online, atlas.ripe.net, radar.cloudflare.com, stats.labs.apnic.net

---

## Headline findings

1. **The US RECS microdata has device COUNTS per household**, not just presence, for TVs, set-top boxes, streaming devices, consoles, desktops, laptops, tablets, smartphones, other cell phones, smart speakers, with household size and age composition on the same record. It is public domain. This is the first-choice source for D and it also gives per-household kWh for TVs. The 2024 round was released in May 2026 (preliminary characteristics file).
2. **Germany publishes device stock per 100 households** (LWR 2024 and EVS 2023) under dl-de/by-2-0, including a smart-home table. The household-size split is not confirmed in the current files.
3. **No source gives "always on" as a measured variable.** It has to be a stated assumption per device class (router, set-top box, smart speaker = always on; the rest duty-cycled), tied to the power-state definitions in the EU Code of Conduct.
> CORRECTED 2026-11-11: headlines 4 and 5 are amended by the ADDENDUM at the end of this file (measured gateway draw exists in Arcep's 2026 survey; the fixed network per line is 1 to 5 W in France in 2024, not 6.5 W everywhere). The original wording is kept below.

4. **Router and ONT power: the EU Code of Conduct v9 (CC BY 4.0) is the best open table**, but its numbers are TARGETS (maxima for signatories), not measured draw. Measured-model lists exist only in the US Voluntary Agreement annual reports (407 models in 2025), whose site I could not open. ENERGY STAR's small-network-equipment and set-top-box lists are gone (programmes sunset 2021 and 2023).
5. **Network energy per GB is the wrong shape for the power-savings theory.** The literature the field itself now uses (Carbon Trust 2021, after Malmodin) is a power model: a large per-line baseload plus a very small per-Mbps term. Under that model keeping traffic local saves almost nothing in the access network in the short run. The simulation team should be told this before a stake is written.
6. **Locality has one good measured public source: Arcep's interconnection barometer** (France, four largest ISPs): 21 % of traffic to end users came from on-net CDNs at end 2025 (range 14 to 48 % by ISP), public peering at IXPs was 1.5 % of inbound traffic, five origin networks carry 48.9 %.
7. **Share of households behind carrier-grade NAT has no public table.** It is a prior from a 2016 paper, measured per network not per household.

---

## D. DEVICES

### D1. US EIA Residential Energy Consumption Survey (RECS) 2020 and 2024 microdata

- What: household-level public-use microdata. 2020: "nearly 18,500 households ... statistically selected to represent the 123.5 million housing units that are occupied as primary residences" (DOEID range 100001-118496, so 18,496 records). https://www.eia.gov/consumption/residential/data/2020/index.php?view=microdata
- Device variables, from the 2020 codebook ("RECS 2020 Codebook for Public File - v7.xlsx", 58,382 bytes), section ELECTRONICS. All are COUNTS unless noted:
  - `TVCOLOR` Number of televisions used (0-14); per-TV size, display type, main use (live / set-top box / built-in smart apps / separate streaming device), hours on per weekday and weekend day for the three most-used TVs (`TVSIZE1-3`, `TVTYPE1-3`, `TVUSE1-3`, `TVONWD1-3`, `TVONWE1-3`)
  - `CABLESAT` cable or satellite boxes without DVR (0-12); `COMBODVR` boxes with DVR (0-10); `SEPDVR` separate DVRs (0-6)
  - `INTSTREAM` over-the-top internet streaming devices for TV (0-10); `PLAYSTA` video game consoles (0-8); `DVD`, `VCR`, `TVAUDIOSYS`
  - `DESKTOP` desktop computers (0-8); `NUMLAPTOP` laptops (0-20); `NUMTABLET` tablets or e-readers (0-10); `ELPERIPH` printers etc. (0-10)
  - `NUMSMPHONE` smartphones (0-11); `CELLPHONE` other cell phones (0-12)
  - `SMARTSPK` Number of smart speakers (0-9), and yes/no for what they control: lighting, temperature, security/cameras/locks, TV, other (`SSLIGHT`, `SSTEMP`, `SSSECURE`, `SSTV`, `SSOTHER`)
  - `INTERNET` Internet access at home (paying / without paying / none); `INTYPECELL`, `INTYPEBROAD`, `INTYPEOTH` (cellular data; "broadband service"; "satellite, dial-up, or other") yes/no
  - `TELLWORK`, `TELLDAYS`, `ONLNEDUC` (telework, days, online education), and which device is used for telework
  - each has an imputation flag `Z...`
- Household keys on the same record: `NHSLDMEM` members (top-coded 1-7), `NUMCHILD` (0-4, top-coded), `NUMADULT1` 18-64, `NUMADULT2` 65+, `HHAGE` respondent age, `ATHOME` weekdays someone is home, `MONEYPY` income bands, `TYPEHUQ` housing type, `KOWNRENT`.
- Geography: `REGIONC`, `DIVISION`, `STATE_FIPS`, `UATYP10` (U urban area / C urban cluster / R rural), `BA_climate`. Nothing finer than state.
- Energy on the same record (2020 final file): `KWHTVREL` "Calibrated electricity usage for televisions and related peripherals, in kilowatt-hours, 2020" (0-5911), `KWHTV1-3`, `KWH` total. These are END-USE MODEL estimates, not metered per device. There is no separate modelled kWh for computers, phones or network equipment that I found; they fall in `KWHNEC` / `KWHOTH` ("not elsewhere classified").
- Weights: `NWEIGHT` final weight plus 60 replicate weights (seen in the 2024 codebook as `NWEIGHT1-60`). EIA: "we strongly encourage data users to read Using the 2024 microdata file to compute estimates and standard errors".
- Reference year: calendar 2020 (a pandemic year: telework and online-education variables exist because of it). 2020 mode: "self-administered Web or paper questionnaire" [search summary of EIA methodology page].
- **RECS 2024**: released May 2026; "nearly 17,000 households ... to represent the 132.5 million housing units"; "The first version of the 2024 RECS microdata file reflects preliminary household characteristics data." https://www.eia.gov/consumption/residential/data/2024/index.php?view=microdata . 2024 codebook (40,397 bytes) keeps the same electronics counts, adds `MONITORS` (external monitors, 0-20), drops VCR and the telework block, renames household size to `NHSLDMEM_PUB` (0-10, derived), and has `UATYP20` (U/R only). No end-use kWh yet in v1 [my reading of "preliminary household characteristics"].
- Files: `recs2020_public_v7.csv` 56,044,261 bytes (last modified 2024-01-22); `recs2020_public_v7.zip` (SAS) 21,123,948 bytes; `recs2024_public_v1.csv` 24,685,540 bytes (last modified 2026-03-09). CSV, SAS. The `csv/public_layout.csv` link on both pages returned a non-CSV page.
- Licence (EIA, https://www.eia.gov/about/copyrights_reuse.php): "U.S. government publications are in the public domain and are not subject to copyright protection. You may use and/or distribute any of our data, files, databases, reports, graphs, charts, and other information products". Requested acknowledgment with publication date. No sign-up.
- Traps: the wording is devices "used", self-reported; smartphone count is per household, so per-person needs division by a top-coded household size; 2020 is a pandemic year; smart-home coverage is smart speakers (count) plus what they control (yes/no), no count of cameras, plugs, thermostats (a smart-thermostat yes/no exists elsewhere in the questionnaire [my reading]); no router/modem variable at all; state is the finest geography, so the density-class link must go through `UATYP10` and housing type.
- Cannot give: which devices are always on; device age; anything outside the US; per-person ownership.
- **Verdict: usable now.** First choice for D. It supports exactly the table the generator needs: P(count of each device type | household size, urban type, housing type), with design weights.

### D2. US NTIA Internet Use Survey (CPS Computer and Internet Use Supplement), November 2023

- What: person- and household-level microdata. Household yes/no for each device type AND "Who is that?" per person: `HELAPTOP`/`PELAPTOP`, `HEDESKTP`/`PEDESKTP`, `HETABLET`/`PETABLET` ("tablet or e-book reader"), `HEMPHONE`/`PEMPHONE` (smartphone), `HEWEARAB`/`PEWEARAB` (smart watch or fitness band), `HETVBOX` ("smart TV, a game, or another device that connects to the Internet and plays through a TV"), `HEMOBDAT`, home connection type items `HEHOMTE*` ("High-speed Internet service installed at home, such as cable, DSL, or fiber optic"; "Satellite Internet service"), streaming and telework items per person. Source: november-2023-techdocs.pdf (1,965,813 bytes) at https://www.ntia.gov/sites/default/files/data/2023-survey-data/november-2023-techdocs.pdf
- Sample: CPS, "approximately 60,000 eligible housing units" (same PDF). Rounds: Nov 2023, 2021, 2019, 2017, July 2015. https://www.ntia.gov/page/download-ntia-internet-use-survey-datasets
- Files: `nov23-cps-csv.zip` 125,467,150 bytes; also Stata, R, raw fixed; plus a small "Analyze Table" of summary statistics (size not stated).
- Licence: NTIA says it "has made public use datasets available"; no licence text on the page. US federal work, public domain [my reading]. No sign-up.
- Traps: it is USE not ownership, and yes/no not counts. Home technology does not separate fibre from cable from DSL (one combined category).
- Cannot give: device counts; fibre versus cable.
- **Verdict: usable with caveats.** Its unique value is PERSON-level device use joined to age, which RECS lacks. Runner-up to RECS, and its complement for "by age".

### D3. Germany, Destatis: Ausstattung privater Haushalte (LWR 2024, EVS 2023)

- What: national tables, share of households with a good ("Ausstattungsgrad", % of households) and count per 100 households ("Ausstattungsbestand", "Anzahl der Güter je 100 Haushalte"). https://www.destatis.de/DE/Themen/Gesellschaft-Umwelt/Einkommen-Konsum-Lebensbedingungen/Ausstattung-Gebrauchsgueter/Tabellen/liste-infotechnik-d.html (page dated 17 August 2026).
- LWR 2024 values on that page, per 100 households: personal computers 257.2 (stationary 50.9, mobile 206.3; laptop/notebook 116.2; tablet 90.1); telephones 274.5 (landline 87.2; mobile 187.4, "darunter Smartphone" 170.6). Share with internet access 98.6 %; smartphone 90.8 % of households. Households represented: 38,713 thousand. Footnote: "Internetzugang ... Kann nicht ausgewiesen werden, da die Anzahl nicht abgefragt wird" (number of internet connections is not asked).
- EVS 2023 (same page, dated 9 December 2025): PCs 257.8 per 100 (stationary 52.3, mobile 205.5; laptop 118.4; tablet 87.1); 40,836 thousand households.
- Consumer electronics table (TV share 97.2 % in 2024; satellite 40.7 %, cable 36.8 %, IPTV 29.8 %; pay-TV receiver 19.4 %): https://www.destatis.de/DE/Themen/Gesellschaft-Umwelt/Einkommen-Konsum-Lebensbedingungen/Ausstattung-Gebrauchsgueter/Tabellen/liste-unterhaltungselektronik-d.html . TV counts per 100 and console counts are on that page but I did not capture them.
- **Smart-home table** (LWR, share of households, 2022 then 2024): Smart TV 58.5 / 66.8; smart speaker 15.2 / 22.7; smart energy management (thermostat, meter, lighting) 9.7 / 16.1; smart security (alarm, smoke detector, camera) 9.5 / 12.2; smart household appliance 13.0 / 18.0. https://www.destatis.de/DE/Themen/Gesellschaft-Umwelt/Einkommen-Konsum-Lebensbedingungen/Ausstattung-Gebrauchsgueter/Tabellen/a-smarte-geraete-systeme-d-lwr.html . Shares only, no counts.
- Timing notes on the page: "2018 und 2023 fand keine LWR-Erhebung statt, da in den Jahren der Einkommens- und Verbrauchsstichprobe (EVS) keine LWR durchgeführt wird" (no LWR in EVS years); reference date 1 January, and for 2024 "Beginn des Berichtsmonats" (start of the reporting month), so 2024 is not strictly comparable with earlier years [my reading].
- Files: "Statistischer Bericht - Laufende Wirtschaftsrechnungen - 2024 (xlsx, 331KB)" and "Statistischer Bericht - Einkommens- und Verbrauchsstichprobe: Hauptprogramm und Geld- und Sachvermögen - 2023 (xlsx, 2MB)" from https://www.destatis.de/DE/Themen/Gesellschaft-Umwelt/Einkommen-Konsum-Lebensbedingungen/Ausstattung-Gebrauchsgueter/_inhalt.html ; GENESIS statistics 63111 (LWR) and 63211 (EVS) are linked from the same page.
- Breakdown by household size: older Fachserie 15 Reihe 2 editions carried "Tabelle 1: Ausstattung privater Haushalte mit ausgewählten Gebrauchsgütern nach der Haushaltsgröße" [search summary]; whether the 2024 xlsx and GENESIS 63111 still carry it is `[not confirmed]` (GENESIS is JavaScript-only). This is the one thing to check first when files may be opened.
- Licence (https://www.destatis.de/DE/Service/Impressum/copyright-genesis-online.html): "Datenlizenz Deutschland – Namensnennung – Version 2.0"; "Die bereitgestellten Daten und Metadaten dürfen für die kommerzielle und nicht kommerzielle Nutzung insbesondere vervielfältigt, ausgedruckt, präsentiert, verändert, bearbeitet sowie an Dritte übermittelt werden" (may be copied, modified and passed on for commercial and non-commercial use); source note required, with "eigene Berechnung" when derived. That page is the GENESIS notice; I take it to cover the topic tables too [my reading].
- Traps: LWR is a quota sample that excludes the highest-income households and, I believe, households of the self-employed and farmers [my reading]; stock per 100 is a mean, so the distribution across households has to be assumed.
- Cannot give: sub-national grain; a count distribution; per-person; always-on.
- **Verdict: usable now** as a national mean and as the German check on a RECS-shaped conditional. Runner-up for D (first choice for the German twin).

### D4. Eurostat ICT survey in households and by individuals (isoc_*)

- What: annual survey since 2002; "The statistical units are Households and Individuals"; household population "all private households having at least one member in the age group 16 to 74 years", individuals aged 16 to 74; "data refer to the first quarter of the reference year"; units "% of households" and "% of individuals". Household breakdowns: NUTS 1 (NUTS 2 optional), degree of urbanisation, "type of household", income. Individuals: age, sex, education, employment. Regional tables exist "only for a selection of indicators". https://ec.europa.eu/eurostat/cache/metadata/en/isoc_i_esms.htm (metadata last updated 11 December 2025).
- Tables confirmed from catalogue metadata (title, period, observation count):
  - `isoc_ci_dev_i` "Individuals - devices used to access the internet", 2016-2025, 290,018 obs
  - `isoc_iiot_use` "Internet of Things - use", 2020-2024, 241,829 obs (the smart-home table)
  - `isoc_ci_in_h` "Households - level of internet access", 2002-2025
  - `isoc_ci_it_h` "Households - type of connection to the internet (2003-2021)" (ended 2021)
  - `isoc_r_broad_h` "Households with broadband access", 2006-2021 (regional, ended 2021)
  - `isoc_cbt` "Broadband internet coverage by technology", 2013-2025, 9,869 obs
  - `isoc_ci_id_h` (households - devices to access the internet) did not resolve in the catalogue: discontinued or renamed `[not confirmed]`
- Licence (https://ec.europa.eu/eurostat/help/copyright-notice): "Reuse of statistical data, metadata, publications, and other dissemination tools published on this website for commercial or non-commercial purposes is authorised provided the source is acknowledged"; Commission content under "Creative Commons Attribution 4.0 International licence". No sign-up for tables. Microdata: "available for scientific purposes" only (research-entity recognition needed).
- Traps: everything is a SHARE of people who used a device type in the last three months, never a count; results from fewer than 20 respondents are flagged unreliable; the household connection-type table stopped in 2021.
- Cannot give: device counts; households by size in most tables (only "type of household"); fibre versus cable subscription after 2021.
- **Verdict: usable with caveats**, as the cross-country check and the by-age gradient for Europe. Not a source of counts.

### D5. Pew Research Center

- What: fact sheets of % of US adults owning a cellphone, smartphone, etc., by age and other demographics. 2026: "based on findings from the National Public Opinion Reference Survey, a Center survey of 5,511 U.S. adults ... conducted Jan. 30-June 18, 2026 ... margin of error ... plus or minus 1.8 percentage points"; "98% own a cellphone of some kind ... 91% own a smartphone". Mode changes: "Polls from 2002-2021 were conducted via phone. In 2023 ... web and mail. In 2024 through 2026 ... web, mail and phone." https://www.pewresearch.org/internet/fact-sheet/mobile/
- Licence (Terms of Use, effective 05/25/2018, https://www.pewresearch.org/about/terms-and-conditions/): content licence is "personal, revocable, nonexclusive, nontransferable"; users "may access, print, copy, reproduce, cite, link, display, download, distribute ... modify, create derivatives of, or otherwise exploit the Content" with notices kept and no implied endorsement. Section 13, survey datasets: "nonexclusive, non-sublicensable, non-transferable, revocable, worldwide, and royalty-free license to access, copy, reproduce, cite ... modify, create derivatives of, or otherwise exploit the survey datasets (other than American Trends Panel survey datasets, which are governed by their own terms and conditions)"; no re-identification or linking of respondent records. Dataset download needs a free account ("My Account") [my reading].
- Cannot give: counts; households; anything but the US.
- **Verdict: usable with caveats**, as a by-age check on person-level smartphone ownership. Published percentages are enough; the microdata is not needed.

### D6. Ofcom Technology Tracker (UK)

- Ofcom's site refused all automated reads. From search summaries only: the 2026 questionnaire, technical report and an xlsx of core data tables exist under `.../statistics/2026/technology-tracker/`; the 2025 core tables were PDF only; the 2026 survey is "100% face-to-face" while 2022-2025 used a hybrid sample, so 2026 is not comparable with 2025; a 2021 CATI omnibus table was titled "COUNT OF INDIVIDUAL DEVICES IN THE HOUSEHOLD - PC, LAPTOP, NETBOOK, TABLET, SMARTPHONE" `[search summary]`. Whether counts are still asked in 2026 is `[not confirmed]`.
- Reuse wording (Ofcom copyright page dated 11 November 2019, archive copy of 2024-06-01): "Unless indicated as Crown-owned material or otherwise, the material featured on this website or provided in hard copy may be reproduced free of charge in any format or medium, provided that it is reproduced accurately and not used in a misleading context. The material must be acknowledged as Ofcom copyright and the title of the document/publication specified." That is not the Open Government Licence by name. Connected Nations data files point to OGL v3 `[search summary]`.
- **Verdict: usable with caveats; not confirmed.** Needs a human to open the 2026 questionnaire and check for count questions.

### D7. Canada and Australia, briefly

- Statistics Canada, Canadian Internet Use Survey 2022 (fieldwork December 2022 to April 2023): smart-home devices in the primary residence, shares: any 47 % (42 % in 2020), smart speakers 30.0 %, connected video cameras 19.8 %, smart plugs or lights 13.8 %, smart thermostats 12.7 %, smart appliances 7.0 %, smart locks 6.0 % `[search summary of StatCan Daily and "Canada at a Glance 2023"]`. Shares, not counts. StatCan Open Licence allows commercial reuse with attribution [my reading]. "The Canadian Internet Use Survey was redesigned in 2018. Comparisons with previous iterations should be made with caution" `[search summary]`.
- Australia: the ABS "Household Use of Information Technology" series ended with 2016-17 [my reading; not confirmed]. Nothing current found.
- **Verdict: usable with caveats** (Canada, shares only); **not usable** (Australia).

### D8. Cisco Annual Internet Report

- Status: last edition covers 2018-2023, published 2020, and has not been continued [my reading; cisco.com page returned nothing]. Its devices-per-capita figures were forecasts, not measurements. **Verdict: not usable** except as a dated historical comparison.

---

## E. HOME NETWORK

### E1. FCC National Broadband Map (Broadband Data Collection)

- What (FCC "Specifications for Data Downloads from the National Broadband Map", June 28, 2024, read from a third-party mirror because fcc.gov refused: https://data.wprdc.org/dataset/e464669e-4271-4a03-ac31-74c3c1eb583a/resource/6221327f-2dc5-4740-9849-98c9fac7735a/download/broadband-map-data-downloads.pdf , 1,048,375 bytes): "Fixed Broadband Availability Data: A list of locations served by a selected technology", one record per provider per location, with `location_id` (Broadband Serviceable Location Fabric), `technology`, `max_advertised_download_speed` and `max_advertised_upload_speed` in Mbps, `low_latency`, `business_residential_code`, `block_geoid` (15-digit census block), `h3_res8_id`.
- Technology codes: "10 – Copper Wire; 40 – Coaxial Cable / HFC; 50 – Optical Carrier / Fiber to the Premises; 60 – Geostationary Satellite; 61 – Non-geostationary Satellite; 70 – Unlicensed Fixed Wireless; 71 – Licensed Fixed Wireless; 72 – Licensed-by-Rule Fixed Wireless; 0 – Other".
- Files: CSV in zip per state and technology, "bdc_{State FIPS}_{Technology}_fixed_broadband_{Data As-of Date}_{Revision Date}.zip"; also "Fixed Broadband Summary by Geography Type" (nationwide, state, county, congressional district, tribal area, CBSA, census place) with `total_units` and the share of units at speed tiers (`speed_02_02`, `speed_10_1`, `speed_25_3`, `speed_100_20`, `speed_250_25`, ...) by technology. Sizes not stated. Twice-yearly as-of dates (June 30 and December 31) [my reading].
- Licence: none stated in the specification. US federal data, public domain [my reading]. The location Fabric itself is licensed from a vendor and is NOT in the public files; public files carry only the location ID and block [my reading].
- **Trap, the important one: this is AVAILABILITY (what a provider says it could sell at a location), not SUBSCRIPTION**, and speeds are maximum advertised. It answers "which technologies could this tile have" and not "what share of households has fibre".
- Subscription side: the ACS (already known) for subscription type; FCC "Internet Access Services" reports and the old Form 477 tract-level "residential fixed connections per 1,000 households" in bands [my reading; not confirmed, fcc.gov unreadable].
- **Verdict: usable with caveats.** The county or census-place summary file is the small set for US technology availability by area.

### E2. FCC Measuring Broadband America (MBA)

- What (Technical Appendix to the Thirteenth MBA Report, https://data.fcc.gov/download/measuring-broadband-america/2023/Technical-Appendix-fixed-2023.pdf , 1,090,693 bytes): a panel of volunteer subscribers with a measurement device ("Whitebox") at the home gateway; "Test results from a total of 2,025 panelists were used in the Thirteenth MBA Report"; measurements September/October 2022; panel "initially drawn from a pool of over 175,000 volunteers" in 2010. "The data collected in this program are made available as open data for review and use by the public."
- It includes a **data usage test** that replaced the old "Consumption" test (methodology at https://transition.fcc.gov/oet/mba/Data-Usage-Technical-Methodology_2018-08-24_Final-v1.3.pdf , not opened), and a traffic monitor sampling WAN bytes every 5 seconds to decide whether the line is idle. So the raw data should contain bytes up and down per home over time: a diurnal household traffic curve `[not confirmed: field list and whether the usage data is in the public release]`.
- "As of August 1, 2023, the FCC no longer partners with SamKnows on fixed broadband performance data collection" `[search summary of the FCC MBA page]`. So the series is frozen at the 2022 measurement period [my reading].
- Traps: volunteers on tested tiers of the largest ISPs only; 2,025 homes; 2022.
- **Verdict: usable with caveats**, mainly for G (time-of-day shape of household bytes), not for E.

### E3. Ookla Speedtest open data

- What (https://github.com/teamookla/ookla-open-data README): "global fixed broadband and mobile (cellular) network performance metrics in zoom level 16 web mercator tiles (approximately 610.8 meters by 610.8 meters at the equator)"; per tile `avg_d_kbps`, `avg_u_kbps`, `avg_lat_ms`, `tests`, `devices`, `quadkey`. Quarterly from Q1 2019 "through the most recently completed quarter (Q1 2026)". "Measurements are filtered to results containing GPS-quality location accuracy."
- Licence: "Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International (CC BY-NC-SA 4.0)". Non-commercial, and derived work must carry the same licence. No sign-up (public S3 bucket).
- Files: one parquet per quarter and type; fixed Q4 2025 `2025-10-01_performance_fixed_tiles.parquet` 363,645,901 bytes; fixed Q1 2026 348,853,499 bytes (global). Also shapefile zips.
- Traps: self-selected tests (people test when something is wrong or when they have just upgraded); tile mean of test results, not of lines; Wi-Fi limits many results; counts vary between accesses ("Data is subject to be reaggregated regularly in order to honor Data Subject Access Requests"). The README still names Q1 2026 as latest in November 2026, so updates may have stopped or slowed [my reading].
- **Verdict: not usable as a shipped dependency** if the generator or its tables are to be commercially usable or relicensed (NC and SA both bite). Usable for an internal sanity check only.

### E4. M-Lab NDT

- What: active speed tests run by users (many triggered from a search-engine widget [my reading]); download, upload, latency, with client IP geolocation and AS.
- Licence (https://www.measurementlab.net/data/): "All data collected by M-Lab tests are available to the public without restriction under a No Rights Reserved Creative Commons Zero Waiver"; "released under CC0 1.0 Universal with no restrictions on use". Access via BigQuery or Google Cloud Storage archives; BigQuery access needs a Google account, and I believe joining a group for free query quota [my reading]. No file size: it is a queryable archive.
- Traps: self-selected; single-stream NDT7 understates fast lines [my reading]; geolocation is by IP, city-level at best.
- **Verdict: usable with caveats.** The only speed-test source with a licence that permits everything. First choice for measured speed distributions IF a query run is acceptable (that is a data pull, outside this scout).

### E5. RIPE Atlas

- Terms (https://www.ripe.net/about-us/legal/ripe-atlas-service-terms-and-conditions/): "The RIPE NCC will publish the RIPE Atlas Data in various forms and make this available"; "Users agree that the RIPE Atlas Data collected by the Probe(s) they host may be publicly available"; "Users agree to make responsible use of the RIPE Atlas Data in line with the purposes of the RIPE Atlas Service ... Users may perform research using RIPE Atlas Data". Probe network parameters are visible, including "'shared address space' addresses if their probe is behind a double NAT (CGN) (RFC6598)". Creating measurements needs an account and credits; reading public results does not [my reading].
- No explicit open licence and no explicit statement on commercial reuse in what I read. 
- Use for us: the probe list is a small, biased (enthusiast-hosted) sample that reveals, per probe, IPv4-only / dual-stack and whether it sits behind CGN address space. A rough CGN and IPv6 share per country among probe hosts.
- **Verdict: usable with caveats** as a prior for CGN, with the host bias stated.

### E6. APNIC Labs (IPv6) and Google IPv6 statistics

- APNIC method (https://blog.apnic.net/2025/11/14/how-we-measure-ipv6/): "APNIC Labs uses online advertisements (ads) to perform such measurements"; "configured to present between 25M to 30M ad impressions per day"; "The per-economy ad data is weighted by the relative user count per economy to adjust for this implicit ad presentation bias". Per-economy and per-AS "IPv6 Capable" and "IPv6 Preferred" rates, 30-day averages, daily since 2012 (https://stats.labs.apnic.net/ipv6). No licence statement found on the pages read.
- Google (https://www.google.com/intl/en/ipv6/statistics.html): "the percentage of users that access Google over IPv6", per country. No licence statement read.
- APNIC CGN measurement: I found no current APNIC table of the share of users behind CGN `[not confirmed]`.
- Traps: per user or per ad impression, mobile-heavy, not per household; a household on a dual-stack fixed line and IPv6 mobile counts several times.
- **Verdict: usable with caveats** as a per-country IPv6 share prior (cite the page and date; do not redistribute a table).

### E7. Carrier-grade NAT share

- Richter et al., "A Multi-perspective Analysis of Carrier-Grade NAT Deployment" (IMC 2016, https://arxiv.org/abs/1605.05606): detects CGN from BitTorrent DHT peer lists and Netalyzr sessions. Headline numbers as I recall them: CGN in roughly 17-18 % of non-cellular eyeball networks tested and in more than 90 % of cellular networks `[my reading; the abstract page cut off before the numbers]`. Unit is networks (ASes), not households, and the data is from 2014-2016.
- Livadariu et al. (INFOCOM 2018): CGN inferred "in 4.1K of the 17.4K measured networks" `[search summary]`.
- Cloudflare (https://blog.cloudflare.com/detecting-cgn-to-reduce-collateral-damage/): a classifier trained on "more than 200K CGNAT IPs"; no published per-country share that I saw.
- **Verdict: no table exists. Prior only**, and it must be a range by access type: fixed-line fibre/cable/DSL low, fixed wireless and mobile-only near-universal [my reading].

### E8. Cloudflare Radar

- Licence (https://developers.cloudflare.com/radar/): "Data available via Radar API endpoints is made available under the CC BY-NC 4.0 license." The API needs a free account token [my reading]. Shares by country and AS of IPv6, device type (mobile/desktop), HTTP versions, traffic trends as normalised indices, not bytes [my reading].
- **Verdict: not usable as a dependency** (non-commercial); fine to cite a figure.

### E9. PeeringDB (terms read, as asked)

- Acceptable Use Policy (https://www.peeringdb.com/aup): "Except for Internet operational purposes approved by PeeringDB, no part of the PeeringDB data may be reproduced, stored in a retrieval system, or transmitted, in any form or by any means ... without prior permission of PeeringDB on behalf of the copyright holders." "The PeeringDB data may not be passed on in bulk to any other person or organization unless approved by PeeringDB." "Users will not be able to download the full contents of the database unless the intended use is for 'Internet operational issues'. These words are tightly defined and would include network trouble-shooting, abuse reporting, and Internet research and analysis. It would not include compiling marketing lists, demographic mapping, or any other commercial application."
- Reading: "Internet research and analysis" is inside the permitted purpose; bulk redistribution is not, and "demographic mapping" and "any other commercial application" are named as outside. A scene generator that ships a table of exchanges and facilities per metro derived from PeeringDB would be redistribution and arguably demographic mapping [my reading].
- **Verdict: usable with caveats for our own analysis; not usable as shipped data without asking PeeringDB.** Ask before building on it.

### E10. Germany: Bundesnetzagentur Breitbandatlas / Gigabit-Grundbuch, and the annual report

- Breitbandatlas FAQ (https://www.bundesnetzagentur.de/GIGA/DE/Breitbandatlas/Dokumente/FAQ.pdf , 173,604 bytes): "Die Informationen werden in Rastern dargestellt, also nicht adressgenau. Zudem sind im Downloadbereich (https://gigabitgrundbuch.bund.de/GIGA/DE/Downloads_Suche/start.html) bis auf Gemeindeebene aggregierte Daten verfügbar." (shown in grid cells, not address-exact; data aggregated down to municipality level in the download area). Operators report by address, minimum and maximum available down and up rates of the strongest tariff; "Die haushaltsbezogene Übersicht wird im Nachgang durch Verschneidung mit Haushaltsdaten Dritter erzeugt" (the household view is produced afterwards by overlay with third-party household data).
- Licence of the download area: `[not confirmed]`. A 2022 Bavarian extract was catalogued as CC BY 4.0 `[search summary]`. Grid cell size `[not confirmed]`.
- Like the FCC map this is AVAILABILITY, not take-up.
- Take-up, from the Jahresbericht Telekommunikation 2025 press page (https://www.bundesnetzagentur.de/SharedDocs/Pressemitteilungen/DE/2026/20260605_TK_JB.html): connections under 100 Mbit/s maximum download fell "von 17,1 auf 15,2 Millionen", faster ones rose "von 21,5 auf 23,6 Millionen", of which three million at 1,000 Mbit/s or more; active fibre connections (FttH and FttB) rose "von 5,3 auf 6,4 Millionen"; fibre share of active fixed broadband "von 13,7 Prozent im Jahr 2024 auf 16,5 Prozent zum Jahresende 2025"; "Der DSL-Anteil sank im selben Zeitraum von 61,1 auf 58,5 Prozent". National only.
- **Verdict: usable now** for the national technology mix (annual report); **usable with caveats** for municipality-level availability once the download licence is read.

### E11. Ofcom Connected Nations (UK)

- Archive copy of the 2025 landing page: "Data downloads 2025: We have made some of the data that underpins the Connected Nations 2025 report available to download." A "Fixed broadband performance" dataset with its own "About this data" document exists, published 19 November 2025, and the files point to the Open Government Licence v3 `[search summary]`. Earlier years carried, per postcode and per local authority, average download and upload speed and average data usage per line [my reading; not confirmed for 2025].
- Northern Ireland: "average monthly broadband data use ... 551 GB per month, rising to 597 GB over full-fibre connections" `[search summary of Ofcom Nations report 2025]`. UK-wide figure `[not confirmed]`.
- **Verdict: probably the best small-area source anywhere for speed AND data use per line together, but not confirmed.** Needs a human to open the data downloads page.

### E12. Eurostat `isoc_cbt` and the ended connection-type tables

- `isoc_cbt` "Broadband internet coverage by technology", 2013-2025: coverage (availability) by country. Household connection type (`isoc_ci_it_h`) ended 2021. See D4 for licence. **Verdict: usable with caveats** (country-level availability only).

### E13. Router always-on

- No survey asks it. The EU Code of Conduct (F1) defines a "ready-state" precisely because home gateways are powered continuously; Carbon Trust (F4) models the home router as 10 W for all 8,760 hours. **Prior: duty cycle 1.0**, stated as an assumption [my reading].

---

## F. POWER

### F1. EU Code of Conduct on Energy Consumption of Broadband Equipment, Version 9.0 (JRC, 2024)

- https://e3p.jrc.ec.europa.eu/sites/default/files/2024-04/jrc136991_kjna31875enn.pdf (2,792,417 bytes; JRC136991, EUR 31875 EN). "the reuse of this document is authorised under the Creative Commons Attribution 4.0 International (CC BY 4.0) licence". A Version 10.0 is listed on the JRC site `[search summary]`; not read.
- What: power TARGETS in watts per functional component (central functions, WAN interface, each LAN port, each Wi-Fi radio, USB), in two states, for Tier 2024 and Tier 2025; a home gateway's target is the sum. v9 replaces the old idle-state with a "ready-state" ("more representative of typical" home use) and keeps the on-state of ETSI EN 301 575.
- Worked examples in the document (ready / on, watts):
  - GPON home gateway, 4 GbE LAN, Wi-Fi 6, 2 USB: 6.4 / 7.5 (2024), 6.2 / 7.3 (2025)
  - XGS-PON home gateway with one 10 GbE LAN: 7.1 / 8.4 (2024), 6.9 / 8.1 (2025)
  - Gigabit Ethernet router, 4 GbE LAN: 3.3 / 3.7
  - Cable DOCSIS 3.0 CPE: 13.7 / 18.4 (2024), 13.6 / 18.3 (2025)
  - "Complex HNID" Wi-Fi access point: 7.4 / 9.4
- WAN interface allowances (ready / on, Tier 2024): ADSL2plus 0.8 / 0.8; VDSL2 (8, 12a, 17a) 1.8 / 1.8; VDSL2 35b 1.8 / 1.8; G.fast 1.5 / 1.6; GPON 0.7 / 0.7 (from the example); XGS-PON 1.2 / 2.1; DOCSIS 3.1 6.5 / 9.7.
- Network side, per subscriber port at the operator's end (Table 18b, uplink interfaces, Tier 2024 / 2025, W): GPON 3.3 / 2.9; XGS-PON 8.3 / 8.0. OLT targets per port, fully equipped: XGS-PON 9.0 / 8.0 (two tiers shown; layer-2 only). A PON port is shared by up to 32 or 64 homes, so the per-home OLT figure is a fraction of a watt [my reading].
- Traps: these are ceilings that signatories commit to for new equipment, not measured draw and not the installed base; an operator-supplied gateway in a home today may be five years older. The gap between ready and on is about 1 W: **home gateway power is almost flat in traffic.**
- **Verdict: usable now**, labelled as targets. First choice for F (home network equipment) because it is open, component-level and covers DSL, cable and fibre in one table.

### F2. EU ecodesign standby regulation, Commission Regulation (EU) 2023/826

- https://eur-lex.europa.eu/legal-content/EN/TXT/HTML/?uri=CELEX:32023R0826 . Annex III: "Power consumption of equipment in off mode shall not exceed 0,50 W. Two years after the application of this Regulation ... shall not exceed 0,30 W." Standby with only reactivation: 0.50 W; with information or status display: 0.80 W. "The power consumption of HiNA equipment or equipment with HiNA functionality, in networked standby shall not exceed 8,00 W. Two years after the application of this Regulation ... shall not exceed 7,00 W. The power consumption of networked equipment, other than HiNA equipment ... in networked standby shall not exceed 2,00 W." (HiNA = high network availability: routers, switches, access points and the like.) Recital: products in scope used an estimated "59,4 TWh in 2015" in these modes in the EU.
- Application date 9 May 2025, so the tighter values from 9 May 2027 [my reading; not read on the page]. EU legal texts are reusable with attribution (Commission Decision 2011/833/EU).
- Use: legal ceilings for smart TVs, consoles, smart speakers and smart-home devices in networked standby (2 W) and for routers (8 W, then 7 W). **Verdict: usable now** as an upper bound per always-on device.

### F3. ENERGY STAR product lists

- Small Network Equipment: "ENERGY STAR Small Network Equipment Product Sunset - July 14, 2021" `[search summary of energystar.gov spec page]`. Set-top boxes: "ENERGY STAR Set-Top Boxes Sunset - February 22, 2023" (https://www.energystar.gov/products/spec/set_top_box_specification_version_5_0_pd). A catalogue search of data.energystar.gov on 2026-11-11 returned no dataset for either. **Not usable: the lists no longer exist.**
- Still live (catalogue metadata, data updated 2026-11-11):
  - "ENERGY STAR Certified Computers V9.0" (id rxdj-2c88; 1,848 models): `Type`, `Off Mode (watts)`, `Sleep Mode (watts)`, `Long Idle (watts)`, `Short Idle (watts)`, `TEC of Model (kWh)`, `Total Battery Capacity (watt-hours)`, dates.
  - "ENERGY STAR Certified Televisions" (id pd96-rr3d): screen size, `Average On Mode Power Consumption for Certification (watts)`, `Reported Annual Energy Consumption (kWh)`, `Power Consumption in Standby Mode when Connected to a Network (watts)`, and the same when not connected.
- Licence: the catalogue metadata gives no licence. US federal data, public domain [my reading]. File size not stated; a few thousand rows.
- Traps: certified models are the efficient end of the market, list of models not sales-weighted, test-procedure power not home use; TEC assumes a duty cycle fixed by the specification.
- **Verdict: usable with caveats** for laptop/desktop idle and sleep watts and TV on/standby watts by size and year.

### F4. US Voluntary Agreements (set-top boxes; small network equipment)

- CableLabs summary of the 2025 reports (https://www.cablelabs.com/blog/voluntary-agreements-2025-results): small network equipment: "99.4% New SNE purchased or sold met the Tier 3 energy levels in 2025"; "91% Improvement in energy efficiency since reporting began in 2015" (NCTA words this as relative to rising broadband speeds `[search summary]`, so it is NOT a 91 % fall in watts); "102.8 million Residential broadband subscriptions covered"; "The 2025 report includes Ready State Power values for 407 SNE models, including fixed wireless access (FWA) devices for the first time." Set-top boxes: "79% Reduction in nationwide annual set-top box energy consumption since 2012"; "96% Coverage of the U.S. Pay-TV market". Canada (CEEVA): "All new set-top box purchases were Non-PVR models for the second consecutive year".
- The annual reports (independent administrator D+R International) with per-model measured ready-state watts are on energy-efficiency.us, which refused my reads. Licence `[not confirmed]`.
- **Verdict: probably the best MEASURED list of modem/router/gateway watts by model and year; not confirmed.** Needs a human to open the 2025 SNE annual report.

### F5. Carbon Trust, "Carbon impact of video streaming" (June 2021)

- https://www.carbontrust.com/sites/default/files/documents/resource/public/Carbon-impact-of-video-streaming.pdf (5,277,639 bytes). Funded by Netflix, written with DIMPACT members. Copyright: "the property of the Carbon Trust"; no open licence, so cite values, do not reproduce tables.
- Conventional (per-GB) constants it uses: "Fixed network energy = 0.0065 kWh/GB" for 2020, "Derived from Aslan et. al, 2018 using the regression analysis presented in the paper"; "Mobile network energy = 0.1 kWh/GB" (Finland 2020, Pihkola et al. 2018); home router "0.025 kWh/GB ... Based on 10W home router ... and average household fixed network data consumption derived from Ofcom figures" (2019).
- **Power model** (after Malmodin): fixed broadband core "1.5W + 0.03 W/Mbps" per line; fixed access "5W + 0.02 W/bitrate%" per line; mobile (4G) core "0.2W + 0.03 W/Mbps" per subscriber; mobile access "1W + 1.5 W/Mbps"; home router "BHR = 10W/line"; assumed line speed 75 Mbps. Text: "the dynamic relation of energy to data volume in a network is very flat – i.e. there is a high fixed power baseload which does not vary in relation to the data volume".
- Device power used (Table 10, W): Android phone 1; laptop 22; desktop with monitor 115; TV 100 (50 inch, "conservative estimate"); gaming console 89 (PlayStation 4); set-top box 18 (D+R International 2020); home router 10 (D+R International 2020). These are in-use streaming powers, not daily averages.
- **Verdict: usable now as priors**, and it is the source that sets the SHAPE of the network power function. Runner-up for F.

### F6. Aslan, Mayers, Koomey, France (2018), "Electricity Intensity of Internet Data Transmission: Untangling the Estimates"

- Journal of Industrial Ecology, published online August 2017, doi 10.1111/jiec.12630; licence CC BY 4.0 (Crossref metadata). Abstract: "provides a new estimate of 0.06 kWh/GB for 2015"; "the electricity intensity of data transmission (core and fixed-line access networks) has decreased by half approximately every 2 years since 2000 (for developed countries)".
- Traps: average allocation (total network kWh divided by total GB), excludes home equipment and data centres; extrapolating the halving to 2026 gives about 0.0013 kWh/GB [my arithmetic: 0.06 x 2^-5.5], which is an extrapolation eleven years past the data. The halving is mostly traffic growth over a flat energy base, and traffic growth has slowed to about 10 % a year (Arcep, G1), so the halving has almost certainly slowed too [my reading].
- **Verdict: usable with caveats** as the historical anchor, never as a marginal saving per GB.

### F7. Malmodin et al.

- "ICT sector electricity consumption and greenhouse gas emissions – 2020 outcome", Telecommunications Policy, April 2024, doi 10.1016/j.telpol.2023.102701 (Crossref metadata; Elsevier licence, open-access status not confirmed). From memory: ICT sector about 4 % of global electricity in 2020, with networks and data centres each a few hundred TWh `[my reading; abstract not available from Crossref]`.
- Malmodin 2020 power model: reached only through Carbon Trust (F5). The Ericsson research brief "Just measure it" was unreadable.
- **Verdict: usable with caveats**; read the paper before quoting a number from it.

### F8. Smartphones

- DXOMARK, 22 April 2023 (https://www.dxomark.com/earth-day-how-every-smartphone-can-contribute-to-saving-the-planet/): an "annual power consumption index" from measured full-charge energy "from the grid to the battery" including charger residual draw, times estimated yearly cycles under moderate use: "wide divergence among the 146 devices ... The average annual power consumption was 4.5 kilowatt hours per year, with the lowest coming in at 3 kWh per year, and the highest ... 7.7 kWh"; "trending slightly downward". Proprietary formula; commercial site, no open licence.
- 4.5 kWh/yr is an average grid draw of about 0.5 W [my arithmetic].
- Phones on mains only while charging: a phone cannot be an always-on mains load, but it IS an always-on network endpoint on battery. For a mesh the relevant number is the extra battery drain from relaying, which no source here measures [my reading].
- **Verdict: usable with caveats** (one commercial lab, its own formula). Prior only.

### F9. Lawrence Berkeley National Laboratory

- Home network equipment: Lanzisera, Nordman and Brown, "Data network equipment energy use and savings potential in buildings" (Energy Efficiency, 2012), bottom-up US stock estimate; and LBNL field metering of miscellaneous loads `[my reading; not opened]`. Old (2008-2012 stock). **Verdict: not usable for levels; usable for the method** (stock x power x hours).
- "2024 United States Data Center Energy Usage Report": US data centres 176 TWh in 2023, projected 325 to 580 TWh by 2028 `[search summary; LBNL page did not render]`. Public report of a US national lab.

### F10. Data centres: IEA, Uptime, Google

- IEA "Energy and AI" (2025): data centres "about 415 TWh in 2024", about 1.5 % of global electricity; base case about 945 TWh by 2030; US 45 %, China 25 %, Europe 15 % of 2024 consumption `[search summary of the IEA executive summary]`. IEA terms (https://www.iea.org/terms): "all text content, reports, articles, commentaries, standalone graphs, figures and infographics produced by and/or sourced to the IEA ... are licensed under a Creative Commons Attribution 4.0 International (CC BY 4.0) licence", with listed exceptions (paid data products). A free account is needed to download report PDFs. IEA's data-transmission-network estimate (roughly 260-360 TWh in 2022) is from memory `[my reading; page did not render]`.
- Uptime Institute Global Data Center Survey 2025 (https://intelligence.uptimeinstitute.com/resource/uptime-institute-global-data-center-survey-2025): "Average PUE levels show little change for the sixth consecutive year". The value 1.54 (2025; 1.56 in 2024), over 800 respondents, April-May 2025 `[search summary]`. Self-reported by respondents, mostly enterprise and colocation. Report behind registration.
- Google (https://datacenters.google/efficiency/): "In 2025, the average annual power usage effectiveness for our global fleet of data centers was 1.09"; "trailing twelve-month (TTM) PUE of 1.09 across all our large-scale data centers (once they reach stable operations)".
- **Verdict: usable now as constants.** PUE is the right multiplier on server energy; none of these gives energy per GB served or per subscriber. That number does not exist in an open source I found.

---

## G. TRAFFIC AND LOCALITY

### G1. Arcep, "Baromètre de l'interconnexion de données en France", 2026 edition (data to end 2025)

- https://www.arcep.fr/fileadmin/user_upload/barometre_interconnexion/2025/arcep-barometre-interconnexion-ip-2026.pdf (4,316,030 bytes), published 16 July 2026 `[search summary for the date]`. Based "sur des données agrégées des quatre principaux FAI en France (Bouygues Telecom, Free, Orange et SFR)" (aggregated data of the four main ISPs), collected under an Arcep decision.
- Inbound traffic at interconnection: "passé de 50,8 Tbit/s fin 2024 à 56 Tbit/s fin 2025, soit une augmentation de 10,4 %" (9.2 % the year before); growth "entre 20 % et 50 % au cours de la période 2013 - 2022" (20 to 50 % a year in 2013-2022, about 10 % now).
- Outbound: "environ 6,6 Tbit/s, soit une augmentation annuelle de 28,1 %"; so inbound to outbound is about 8.5 to 1 [my arithmetic].
- Path mix of inbound traffic, second half 2025: "essentiellement du transit (55,2 % environ) et du peering privé (43,2 % environ). Le peering public, celui ayant lieu dans les points d'échanges internet, ou IXP, n'en représente qu'une part minime (1,5 %)".
- **On-net CDN share: "la part de trafic provenant des CDN internes représente, fin 2025, environ 21 % du trafic à destination des clients des FAI, une part en augmentation (19 % en 2024). Elle varie toutefois entre 14 % et 48 % selon les FAI."** (traffic from CDNs inside the ISP's own network: about 21 % of traffic to ISP customers, 19 % in 2024, between 14 and 48 % by ISP).
- Concentration: "Akamai, Netflix, Amazon, Google, et Meta. Leur part agrégée est estimée fin 2025 à 48,9 % du trafic à destination des utilisateurs finals contre 46,9 % en 2024"; Netflix 11.6 %, Akamai 12 %.
- Licence: not read on the legal page (it returned nothing for the search terms). French public bodies default to the Licence Ouverte 2.0 [my reading; not confirmed].
- Traps: France only; four ISPs; "on-net CDN" means inside the ISP network, which is not the same as "same metro"; transit here includes traffic handed over in Paris by transit providers, so 55 % transit does not mean 55 % crosses a long backbone. Capacity-based (Tbit/s at peak [my reading]) rather than bytes.
- Cannot give: metro-level locality; household-to-household (peer-to-peer) share.
- **Verdict: usable now.** First choice for G (locality). It is the only regulator-collected, repeated, public measurement of how much traffic is served from inside the access network.

### G2. Bytes per household

- OpenVault Broadband Insights 2Q26 (https://openvault.com/ovbi-upstream-growth-nears-20-yoy-tops-downstream-increase-by-3x/): "Average Monthly Usage was 711.0 GB per subscriber, an increase of 7.1% over the 664.2 GB recorded in the second quarter of 2025"; average upstream "58.2 GB per subscriber per month", up 19.2 %; upstream is "8.2% of broadband providers' total traffic mix"; downstream to upstream ratio "from 16.6:1 in 2Q 2019 to 11.2:1"; "Super Power Users of 2 TB per month or more ... (5.86%) of all subscribers"; "Fiber Upstream Average Usage of 94.1 GB per subscriber". Median 482.3 GB `[search summary; the publisher's page text is garbled at that number]`. Source: operators using OpenVault's tools, mostly North American cable [my reading]; commercial report, PDF linked from the press page, no open licence.
- Germany, Bundesnetzagentur Jahresbericht 2025: fixed networks carried about 175 billion GB in 2025, "an average of about 376 GB per broadband connection per month" (about 322 GB in 2024) `[search summary; the press page I read did not show this passage]`. A BNetzA data-portal page "Datenvolumen Festnetz" exists.
- UK: Ofcom, see E11 (551 GB/month in Northern Ireland `[search summary]`).
- So about 12 GB/day (Germany) to 23 GB/day (US cable) per fixed line, growing 7 to 17 % a year [my arithmetic].
- Bytes per DEVICE or by traffic TYPE: only commercial reports (Sandvine/AppLogic "Global Internet Phenomena", behind a registration form, no licence found, page did not show terms). Mobile bytes per smartphone: Ericsson Mobility Report [my reading; not opened]. **No open source gives bytes per device per day by type.** The by-type split has to come from Arcep's origin-AS shares (video streaming dominates) or from Sandvine cited as a figure.
- Time-of-day shape: FCC MBA data usage test (E2) is the only open candidate.
- **Verdict: usable now as three national constants; nothing below national except possibly Ofcom.**

### G3. CDN off-nets, the academic measurement

- Gigis et al., "Seven years in the life of Hypergiants' off-nets" (ACM SIGCOMM 2021, doi 10.1145/3452296.3472928): TLS-certificate scans to find hypergiant (Google, Netflix, Meta, Akamai) servers hosted inside other networks; found thousands of ASes hosting off-nets, tripling over 2013-2021, covering a large share of internet users `[my reading; abstract not retrieved]`. It measures WHERE caches are, not what share of bytes they serve.
- **Verdict: usable with caveats** as support for "most hypergiant traffic to a large-ISP customer is already served from inside or next to the ISP". Read before quoting.

### G4. IXP local share and peer-to-peer share

- IXP: Arcep's 1.5 % (France) is the only measured share of ISP inbound traffic over public peering that I found. It is low because the four large ISPs peer privately; small ISPs would be far higher [my reading]. PeeringDB (E9) and exchange statistics pages give capacity and peak throughput per exchange, not share of a region's traffic.
- Peer-to-peer share: no current open measurement. Sandvine's reports are the usual citation (BitTorrent a few per cent of downstream, larger upstream) [my reading]. Upstream as 8.2 % of total (OpenVault) is an upper bound on anything households originate.
- TeleGeography: terms page did not render; not read.

---

## Ranked recommendation

| Need | First choice | Runner-up | Smallest file set and size |
|---|---|---|---|
| **D devices** | EIA RECS microdata: counts per household of each device type with household size and age composition; public domain | Destatis LWR 2024 / EVS 2023 stock per 100 households (German check; dl-de/by-2-0). For by-age person-level use: NTIA Nov 2023 | `recs2020_public_v7.csv` 56.0 MB (or SAS zip 21.1 MB) + codebook 58 KB; optionally `recs2024_public_v1.csv` 24.7 MB + codebook 40 KB. Destatis: LWR xlsx 331 KB + EVS xlsx about 2 MB. NTIA: `nov23-cps-csv.zip` 125.5 MB |
| **E home network** | National technology mix and speed tiers from regulators: BNetzA Jahresbericht 2025 (DE), ACS already held (US), plus FCC BDC "Fixed Broadband Summary by Geography" for availability by county/place | Eurostat `isoc_cbt` (coverage) and `isoc_ci_in_h`; M-Lab NDT (CC0) for measured speeds if a query run is allowed. CGN and IPv6: priors only (table below) | BNetzA: figures on one press page (0 MB) or the annual report PDF (size not checked). FCC summary file: size not stated. Eurostat tables: under 1 MB each [my reading]. Ookla is 349-364 MB per quarter and NC-SA: avoid |
| **F power** | EU Code of Conduct v9 (CC BY 4.0) for gateway/ONT/OLT watts by technology; ENERGY STAR Computers V9.0 and Televisions lists for device watts; regulation 2023/826 for standby ceilings | Carbon Trust 2021 power model and device table (cite, do not copy); Google and Uptime PUE; US Voluntary Agreement 2025 SNE report for MEASURED gateway watts once a human can open it | CoC v9 PDF 2.8 MB (a table to transcribe by hand, about 40 numbers). ENERGY STAR: two lists, a few thousand rows, size not stated |
| **G traffic and locality** | Arcep interconnection barometer 2026 (on-net CDN share, path mix, origin concentration, growth) | OpenVault 2Q26 and BNetzA for GB per line per month; FCC MBA data-usage raw data for time-of-day shape | Arcep PDF 4.3 MB (about 10 numbers to transcribe). OpenVault and BNetzA: figures on a page |

Order of work I would suggest: (1) RECS 2020 and 2024 together, since both rounds exist and the same variables repeat ("why not both and compare?"): it gives a four-year drift per device type for free. (2) Transcribe the constants table below with sources. (3) Have a human open the three blocked items: Destatis table by household size, the Voluntary Agreement SNE 2025 report, Ofcom Connected Nations fixed-performance data.

What no source gives, so the generator must state as assumptions: which devices are always on; device counts by household size for Germany (until the Destatis table is confirmed); households behind CGN; bytes per device by type; share of traffic that could stay within a 1 km tile or a metro.

---

## Constants usable as priors

"Rate" is how fast the value has been moving. Each row is a price in a regime: the regime is the "measured on" column, and none of these should be carried to another regime without saying so.

| Quantity | Value | Unit | Source, year | Measured on / basis | Rate of change |
|---|---|---|---|---|---|
| Smartphones per 100 households, Germany | 170.6 | per 100 hh | Destatis LWR, 2024 | household budget survey, national | +5 to +6 a year (149.2 in 2019) |
| Laptops / tablets / desktops per 100 hh, Germany | 116.2 / 90.1 / 50.9 | per 100 hh | Destatis LWR, 2024 | same | laptops +2, tablets +5, desktops -1 a year |
| Households with smart TV / smart speaker / smart energy / smart security / smart appliance, Germany | 66.8 / 22.7 / 16.1 / 12.2 / 18.0 | % of hh | Destatis LWR, 2024 | same | +3 to +4 points a year each (2022 to 2024) |
| US adults owning a smartphone | 91 | % of adults | Pew NPORS, 2026 | 5,511 adults, web/mail/phone | about +1 point a year; saturating |
| Smart-home device in primary residence, Canada | 47 | % | StatCan CIUS 2022 `[search summary]` | individuals | +2.5 points a year (42 in 2020) |
| Home gateway, GPON, Wi-Fi 6, ready / on | 6.2 / 7.3 | W | EU CoC v9, Tier 2025 | TARGET ceiling for new equipment, not measured | -0.2 W per tier year |
| Home gateway, XGS-PON, ready / on | 6.9 / 8.1 | W | EU CoC v9, Tier 2025 | target | -0.2 to -0.3 W a year |
| Cable DOCSIS 3.0 CPE, ready / on | 13.6 / 18.3 | W | EU CoC v9, Tier 2025 | target | about flat |
| VDSL2 WAN interface / GPON WAN interface | 1.8 / 0.7 | W | EU CoC v9, Tier 2024 | target, interface only | about flat |
| Simple Ethernet router, ready / on | 3.3 / 3.7 | W | EU CoC v9 | target | flat |
| Home router as modelled | 10 | W, 24 h | Carbon Trust 2021 citing D+R International 2020 | US voluntary-agreement fleet average `[my reading]` | slowly down |
| Networked standby ceiling, router-class (HiNA) | 8.0, then 7.0 | W | Reg. (EU) 2023/826 | legal maximum; dates 2025 and 2027 `[my reading]` | step |
| Networked standby ceiling, other networked devices | 2.0 | W | Reg. (EU) 2023/826 | legal maximum | step |
| Off mode ceiling | 0.5, then 0.3 | W | Reg. (EU) 2023/826 | legal maximum | step |
| OLT port, XGS-PON / GPON uplink interface | 8.0 / 2.9 | W per PON port | EU CoC v9, Tier 2025 | target; port shared by 32-64 homes `[my reading]` | -0.3 to -0.4 W a year |
| Fixed access network baseload | 5 | W per line | Carbon Trust 2021 (Malmodin model) | operator data, Sweden/Europe, about 2018-2020 `[my reading]` | slowly down |
| Fixed core network baseload | 1.5 | W per line | same | same | slowly down |
| Fixed network marginal power | 0.03 (core) | W per Mbps | same | same | down |
| Mobile (4G) network: baseload / marginal | 1.2 / 1.53 | W per subscriber / W per Mbps | same (core + access summed by me) | same | down with 5G `[my reading]` |
| Fixed network average intensity | 0.06 (2015); 0.0065 (2020, regression) | kWh/GB | Aslan et al. 2018; Carbon Trust 2021 | national operators, developed countries, average allocation | halving every 2 years to 2015; slower since `[my reading]` |
| Mobile network average intensity | 0.1 | kWh/GB | Carbon Trust 2021 citing Pihkola 2018 | Finland, 2020 | falling fast |
| Smartphone, mains energy | 4.5 (3.0 to 7.7) | kWh/year | DXOMARK, 2023 | 146 models, lab charge measurement x modelled cycles | slightly down |
| Smartphone in use (streaming) | 1 | W | Carbon Trust 2021 | assumption from battery life | flat |
| Laptop / desktop+monitor in use | 22 / 115 | W | Carbon Trust 2021 citing Singh et al. 2019 | US stock estimate | down |
| TV in use (50 inch) / console / set-top box | 100 / 89 / 18 | W | Carbon Trust 2021 | "conservative"; PS4; D+R 2020 | TVs and set-top boxes down; set-top boxes disappearing (US fleet energy -79 % since 2012) |
| Data centre PUE, industry survey average | 1.54 | ratio | Uptime 2025 `[search summary]` | 800+ self-reporting operators | flat for six years |
| Data centre PUE, hyperscale | 1.09 | ratio | Google, 2025 | fleet, trailing twelve months | flat (1.10 a few years ago `[my reading]`) |
| Data centre electricity, world / US | 415 (2024) / 176 (2023) | TWh | IEA 2025 / LBNL 2024 `[search summaries]` | model estimates | world roughly doubling by 2030 (IEA base case) |
| Fixed data per line | 376 (DE, 2025); 711 (US cable, 2Q26) | GB/month | BNetzA `[search summary]`; OpenVault | all fixed lines; cable operators' subscribers | +17 % (DE), +7 % (US) a year |
| Upstream share of fixed traffic | 8.2 | % | OpenVault 2Q26 | US cable | rising (ratio 16.6:1 in 2019 to 11.2:1) |
| Traffic to end users served from on-net CDNs | 21 (range 14 to 48) | % | Arcep 2026 | four French ISPs, end 2025 | +2 points a year |
| Inbound traffic over public peering at IXPs | 1.5 | % | Arcep 2026 | four French ISPs, H2 2025 | flat |
| Inbound traffic: transit / private peering | 55.2 / 43.2 | % | Arcep 2026 | same | transit +1 point a year |
| Top five origin networks' share | 48.9 | % | Arcep 2026 | same | +2 points a year |
| ISP inbound traffic growth | 10.4 | % a year | Arcep 2026 | same | was 20 to 50 % in 2013-2022 |
| Networks using CGN: fixed eyeball / cellular | about 17-18 / over 90 | % of networks | Richter et al. 2016 `[my reading]` | BitTorrent DHT + Netalyzr, 2014-2016 | unknown; likely up for fixed wireless and new fibre entrants |
| Router duty cycle | 1.0 | fraction | assumption `[my reading]` | consistent with CoC "ready-state" and Carbon Trust | n/a |

---

## Things to tell the simulation team before a stake is written

1. Under the power model the field uses, a fixed line costs about 6.5 W of network baseload plus about 10 W of home router whether or not a byte moves, and the traffic-dependent part is hundredths of a watt per Mbps. A locality dividend measured in GB kept off the backbone converts to very little network energy in the short run. Savings would have to come from (a) fewer or lower-power always-on boxes, (b) data-centre server energy avoided (times PUE 1.1 to 1.5), or (c) long-run capacity not built. Using an average kWh/GB to price saved traffic would overstate the saving by one to two orders of magnitude [my reading of F5 and F6].
2. A mesh that keeps phones or laptops awake to relay moves load onto devices that are otherwise asleep. The networked-standby ceilings (2 W) and laptop idle watts (ENERGY STAR list) are the right comparison, and they are the same order as the router itself.
3. About a fifth of traffic is already served from inside the access network and about half comes from five origins. The headroom for "more local" is the remainder that is cacheable, and no public source measures that.

---
---

# ADDENDUM 2026-11-11: power as a DELTA from the status quo

Added at the project lead's request. Same rules: documentation only, nothing downloaded, marks as above. New documents opened: Arcep "Enquête annuelle Pour un numérique soutenable", édition 2026 (PDF, 2,674,267 bytes); NRDC "Home Idle Load" (PDF, 1,341,311 bytes); Huang et al. MobiSys 2012; Narayanan et al. SIGCOMM 2021; Xu et al. MobiSys 2013; D. Mytton's page for Mytton, Lundén and Malmodin 2024.

## CORRECTION to the main report (dated 2026-11-11, not a quiet fix)

The main report said (headline 4, F1, F4) that measured draw of home gateways existed only in the US Voluntary Agreement reports, which I could not open, and (headline 5, closing note 1) priced the fixed network at "about 6.5 W baseload per line" from the Carbon Trust model. Both statements were made before I had read Arcep's environmental survey. They are superseded as follows.

- **Measured gateway draw exists, is public and is recent**: Arcep measured 36 internet box models, 12 Wi-Fi repeater models and 21 TV decoder models (France, 2024 data, published May 2026). See A1.1. The EU Code of Conduct numbers remain targets; Arcep's are measurements, and they are HIGHER than the targets (9.1 W measured idle against 6.2 to 6.9 W target ready-state for a fibre gateway).
- **The fixed network per line is smaller than the Carbon Trust model says, in France in 2024**: Arcep reports 20 kWh per fixed subscription per year excluding boxes (2.3 W), under 10 kWh for fibre (under 1.1 W), 40 kWh for copper (4.6 W) [watts are my arithmetic]. The Carbon Trust figure (5 W access + 1.5 W core per line) is a c. 2018-2020 mixed-technology model. The 6.5 W figure is a price in that regime; it is roughly right for copper and about five times too high for fibre.
- The qualitative conclusion stands and is strengthened: the box in the home (about 9 W, always on) draws four to eight times what the network draws for that line, and neither moves with traffic.

## A1. Always-on devices households already have

### A1.1 Arcep, "Enquête annuelle Pour un numérique soutenable", édition 2026 (data year 2024)

- https://www.arcep.fr/fileadmin/user_upload/observatoire/enquete-pns/edition-2026/enquete-annuelle-pour-un-numerique-soutenable_edition2026_mai2026.pdf . Regulator data collection from operators under an Arcep decision; published 21 May 2026 `[search summary for the date]`. Licence not read (French public body; Licence Ouverte presumed, `[not confirmed]`).
- Method: "Les mesure de consommation électrique ont été réalisées sur 36 modèles de box internet, 12 modèles de répéteurs Wi-Fi et 21 modèles de décodeurs TV" (measurements on 36 box models, 12 repeater models, 21 decoder models), over two minutes for boxes and repeaters and five minutes for decoders in standby, converted to instantaneous watts. Who measured (operators or a lab) I did not read `[not confirmed]`.
- **Internet box, Wi-Fi on, not in use: "9,1 watts", range "3,3 watts à 25,0 watts"**, stable in 2024.
  - fibre, DSL and 5G boxes without extras: "entre 6,7 et 8,7 watts", mean 8.2 W, "en baisse d'environ 9 % en un an"
  - "box équipées d'un disque dur intégré consomment en moyenne 19,4 watts" (boxes with a built-in hard disk: 19.4 W). This is the only measured figure I found for a storage device left on in the home.
  - boxes with a separate external fibre unit (ONT): "11,3 watts"
  - fibre boxes first sold after 2020: 8.0 W, about 20 % less than those sold 2014-2017
  - Decomposition for ordinary use: on with Wi-Fi off "7,3 watts"; Wi-Fi adds "1,8 watt"; connected devices add 0.4 W; internet use adds 0.6 W; total "10,1 watts". "hors toute sollicitation de l'utilisateur et avec le Wi-Fi activé, la consommation électrique des box représente 90 % de la consommation totale" (idle with Wi-Fi on is 90 % of the total).
  - Marginal on traffic: "une box FttH générant un trafic de 5 Mbit/s en Wi-Fi consomme en moyenne 0,1 watt supplémentaire"; at about 50 Mbit/s, xDSL/FttH boxes add "0,8 watt en Wi-Fi, contre 0,3 watt via câble Ethernet"; at 500 Mbit/s 4G/5G boxes reach +4.3 W on Wi-Fi and +2.5 W on Ethernet (more than double, and four times, the xDSL/FttH figures). Connecting a device adds at most 0.6 W.
  - Switched off: the power supply alone draws "0,1 watt en moyenne" (0.05 to 0.3 W).
- **Wi-Fi repeater, idle: "environ 5,6 watts"**, up to 9.6 W (5.3 W in 2023; Wi-Fi 7 models draw more); HD video through it adds 0.3 W.
- **TV decoder (set-top box): standby "4,1 watts en moyenne"**, range "0,2 watt à 15,4 watts"; in use "7,4 watts"; models sold after 2020: 4.4 W [in use, my reading of the sentence]. "77 % des abonnements internet disposent également du service audiovisuel" (77 % of internet subscriptions include the TV service), an ownership share for the decoder.
- National totals: boxes and decoders "3,4 TWh" in 2024, falling 3 % a year, "cinq fois supérieure à la consommation totale d'énergie des réseaux d'accès fixes" (five times the fixed access networks, 0.6 TWh).
- Annual energy implied [my arithmetic]: box 9.1 W = 80 kWh/yr; decoder in standby all year 4.1 W = 36 kWh; repeater 5.6 W = 49 kWh.
- Traps: France, operator-supplied equipment of the four main ISPs; French "box" is an integrated gateway, often with TV and voice functions, so not the same object as a US cable modem plus retail router; model average, I did not read whether it is weighted by installed base `[not confirmed]`.
- **Verdict: usable now. First choice for measured always-on draw of home network equipment and set-top boxes**, replacing the Code of Conduct as first choice for levels (the Code remains first choice for a by-technology component table and for the states' definitions).

### A1.2 NRDC, "Home Idle Load: Devices Wasting Huge Amounts of Electricity When Not in Active Use" (issue paper, May 2015)

- https://www.nrdc.org/sites/default/files/home-idle-load-IP.pdf . With Home Energy Analytics and the Stanford Sustainable Systems Lab.
- What: three northern California data sets. "The largest data set of 70,000 homes contained only smart meter energy use data ... It was used to calculate the 23 percent always-on share"; "A smaller sample of 2,750 San Francisco Bay Area homes"; "a detailed audit of a subset of 10 Bay Area homes".
- Findings: average always-on load "164 watts, or about 1,300 kilowatt-hours annually per home. This represents 23 percent of annual electricity consumption"; the 10-home audit found "an average of 65 electrical devices" per home; "the idle load of printers ranged from 2 to 26 watts per home, and cordless phones from 1 to 12 watts"; consumer electronics ("televisions, computers, printers, game consoles, etc.") are the largest group (share on the page I did not capture).
- Traps: "always-on" here is the minimum of the whole-home meter, so it includes refrigerator-cycle floor, recirculation pumps, fans and anything else, not just electronics; per-device watts come from TEN homes; California; 2014 data. NRDC copyright, no open licence.
- **Verdict: usable with caveats** as the size of the pond: an added always-on watt is compared with about 164 W already there. Not a source of per-device watts.

### A1.3 Counts and ownership shares of the always-on classes

| Device | Count or share per household: where it comes from | Watts by state: where it comes from |
|---|---|---|
| Router / gateway / ONT | No survey counts it. Equals fixed-broadband households (ACS; RECS `INTYPEBROAD`; Destatis internet access 98.6 % of households 2024) | Arcep 2024 measured: 9.1 W idle with Wi-Fi (3.3 to 25.0), +0.1 to +0.8 W under traffic; external ONT variant 11.3 W. Code of Conduct v9 targets 6.2 to 8.4 W (fibre), 13.6 / 18.3 W (DOCSIS 3.0). Legal ceiling 8 W then 7 W networked standby (Reg. 2023/826) |
| Wi-Fi repeater / mesh node | none found | Arcep 2024: 5.6 W idle (to 9.6) |
| Set-top box | RECS counts: `CABLESAT`, `COMBODVR`, `SEPDVR`; France 77 % of subscriptions; Germany pay-TV receiver 19.4 % of households (2024) | Arcep 2024: 4.1 W standby (0.2 to 15.4), 7.4 W in use. Carbon Trust 2021: 18 W (US, D+R 2020). Falling fast; DVR models disappearing |
| Streaming stick / box | RECS count `INTSTREAM` | none read; legal ceiling 2 W networked standby |
| Network storage | No survey counts it (RECS has no such variable) | Arcep: box with built-in disk 19.4 W. Stand-alone NAS: no source read; 10 to 30 W idle for a two- to four-bay unit `[my reading]` |
| Smart speaker | RECS count `SMARTSPK`; Destatis 22.7 % of households (2024); StatCan 30 % (2022) `[search summary]` | no measured source read; about 2 to 3 W idle listening `[my reading]`; legal ceiling 2 W networked standby in the EU |
| Smart TV in standby | RECS count `TVCOLOR` and `TVUSE1-3` (built-in smart apps as main use); Destatis smart TV 66.8 % of households (2024) | ENERGY STAR Televisions list has the column "Power Consumption in Standby Mode when Connected to a Network (watts)" per model (values not read: that would be a data pull). Legal ceiling 2 W networked standby, 0.5 W off |
| Games console in standby | RECS count `PLAYSTA` | Xbox Series X "Instant On" about 10 to 13 W, energy-saver under 1 W; PS5 rest mode about 0.4 W by default and 3 to 4 W with USB charging; default differs by region (energy-saver default in Europe, Instant On in the US at the time) `[search summary of press reports of NRDC and other tests, 2020-2022]`. In play 160 to 200+ W `[search summary of NRDC]` |
| Desktop left on | RECS count `DESKTOP`; Destatis 50.9 stationary PCs per 100 households (2024). Whether it is left on: no source | ENERGY STAR Computers V9.0 list has `Short Idle`, `Long Idle`, `Sleep Mode`, `Off Mode` watts per model (values not read). Carbon Trust: 115 W desktop with monitor in use |
| Hours of TV on | RECS `TVONWD1-3`, `TVONWE1-3` (hours per weekday and weekend day, three most-used TVs) | n/a |

Every count in this table for the US is one pass over the RECS file already recommended; no further source is needed for D. The smallest added file set for A1 is the Arcep PDF (2.7 MB, about 25 numbers to transcribe).

What does not exist: a survey that says which devices a household leaves on. RECS hours-on for TVs is the only measured duty cycle. Everything else is "on by design" (router, repeater, set-top box, smart speaker, streaming stick) or an assumption.

## A2. Marginal energy on a phone or laptop

All of the following are measurements with a power monitor on a specific handset on a specific network. Each is a price in that regime: handset, radio generation, carrier timer settings and year are part of the number.

### A2.1 Huang, Qian, Gerber, Mao, Sen, Spatscheck, "A Close Examination of Performance and Power Characteristics of 4G LTE Networks" (MobiSys 2012)

- Read from an author-hosted copy (University of Minnesota page of F. Qian). Device: "an HTC phone with LTE data plan from a cellular ISP ... 1 GHz Qualcomm MSM8655 CPU, running Android 2.2.1"; US commercial LTE, 2011-2012.
- Radio state machine: "Ttail, RRC inactivity timer, 11.576s: UE stays in RRC_CONNECTED for Ttail before demoting to RRC_IDLE".
- Power by state (Table 3): LTE promotion 1210.7 mW; **LTE tail base 1060.0 mW**; 3G promotion 659.4 mW for 582 ms; 3G DCH tail base 803.9 mW for 8.09 s; 3G FACH tail base 601.3 mW for 0.82 s; **Wi-Fi promotion 124.4 mW for 79 ms; Wi-Fi tail base 119.3 mW for 238 ms**. Idle with screen off 11.36 mW (847.15 mW with screen on).
- **One packet, including promotion and tail: "for LTE, 3G and WiFi is 12.76J, 7.38J and 0.04J, respectively."** That is a factor of about 320 between LTE and Wi-Fi for a small transfer [my arithmetic].
- Data transfer power model (Table 4), P = alpha x throughput + beta: LTE uplink 438.39 mW/Mbps, downlink 51.97 mW/Mbps, base 1288.04 mW; 3G 868.98 / 122.12 / 817.88; Wi-Fi 283.17 / 137.01 / 132.86.
- Bulk transfer: energy per bit falls from about 10 µJ/bit to under 1 µJ/bit as transfer size grows; "With bulk data size of 10MB, LTE consumes 1.62 times the energy of" Wi-Fi; 3G many times more. Abstract: "LTE is as much as 23 times less power efficient compared with WiFi" (small transfers, trace-driven), and the tail is the key contributor.
- What it means for keep-alives [my arithmetic, 2012 LTE regime]: one keep-alive every 5 minutes over LTE is 288 x 12.76 J = 3.7 kJ = about 1.0 Wh a day, some 6 to 8 % of a 13 to 17 Wh phone battery; the same over Wi-Fi is 288 x 0.04 J = 11.5 J = 0.003 Wh a day.
- **Verdict: usable with caveats as the canonical constants**, with the regime stated. Carrier timers have shortened and handsets changed since.

### A2.2 Narayanan et al., "A Variegated Look at 5G in the Wild: Performance, Power, and QoE Implications" (SIGCOMM 2021)

- Author-hosted copy (same site), 2,037,249 bytes. Devices: Google Pixel 5, Samsung Galaxy S20 Ultra 5G, Samsung Galaxy S10 5G; carriers Verizon and T-Mobile (US), mmWave, low-band, NSA and SA; measured 2020-2021.
- Timers: "the timers of NSA 5G and 4G LTE are very similar"; "T-Mobile SA 5G has a tail timer of 10s". 5G consumes more energy than 4G during the tail period, "for mmWave 5G the tail power is especially higher".
- Throughput: power rises linearly with throughput; "5G can be 79% (74%) less efficient than 4G at a low throughput" on downlink (uplink), and more efficient at high throughput (there is a crossover).
- Exact tail milliwatts are in a table I did not capture.
- **Verdict: usable with caveats.** It says the 2012 shape (a roughly 10 s high-power tail after any small transfer) still held on 5G in 2021, and that small, chatty traffic is where 5G is worst.

### A2.3 Xu et al., "Optimizing Background Email Sync on Smartphones" (MobiSys 2013)

- https://www.cs.wm.edu/~liqun/paper/mobisys13.pdf (758,997 bytes). Device: "Nexus S smartphone running Android ICS 4.0.4", 3G network of China Unicom, Monsoon power monitor.
- "the energy cost of receiving a small email on an Android smartphone is more than 14,000mJ, which is as high as the energy consumed by a typical smartphone in standby mode for 10 minutes." Their fixes (fast dormancy, shorter and adaptive 3G tail, decoupling processing from the radio) cut it "by 49.9% on average".
- **Verdict: usable with caveats**: the best single statement of the cost of one push event, 14 J on 3G in 2013.

### A2.4 Not retrieved, named for whoever reads next

- Balasubramanian, Balasubramanian, Venkataramani, "Energy Consumption in Mobile Phones: A Measurement Study and Implications for Network Applications" (IMC 2009): the origin of "tail energy"; 3G tail of about 12.5 s and well over half of the energy of a small transfer spent in the tail; GSM and Wi-Fi compared (Nokia N95) `[my reading; not opened]`.
- Aucinas et al., "Staying Online While Mobile: The Hidden Costs" (CoNEXT 2013): cost of always-connected apps and keep-alives `[my reading; a search did not find it]`.
- Wi-Fi power-save mode and relaying on a phone (hotspot or mesh relay duty): **no measured source found.** A phone acting as a Wi-Fi relay cannot sleep its radio in power-save mode the way a client does, so the Huang Wi-Fi base power (about 120 to 130 mW radio, 2012) held continuously is the floor: about 3 Wh a day, a fifth of a phone battery [my arithmetic and reading]. This is the number the theory most needs and nobody has published it for a current phone.
- Laptops: no radio-tail literature of the same kind; the relevant figure is that a relaying laptop cannot enter sleep (ENERGY STAR `Sleep Mode` against `Long Idle` watts per model, values not read).

### A2.5 Constants

| Quantity | Value | Unit | Source, year | Device and network | Rate of change |
|---|---|---|---|---|---|
| LTE tail time | 11.576 | s | Huang 2012 | HTC LTE phone, US carrier | carrier-set; about 10 s still on T-Mobile SA 5G in 2021 |
| LTE tail base power | 1060 | mW | Huang 2012 | same | 5G higher, mmWave much higher (2021) |
| 3G tail (DCH + FACH) | 8.9 s at 804 / 601 | s, mW | Huang 2012 | same phone, 3G | 3G being switched off |
| Wi-Fi tail | 0.238 s at 119 | s, mW | Huang 2012 | same phone | newer chipsets lower `[my reading]` |
| One small packet, all-in | 12.76 / 7.38 / 0.04 | J (LTE / 3G / Wi-Fi) | Huang 2012 | same | ratio persists; absolute values dated |
| One push email received | over 14 | J | Xu 2013 | Nexus S, 3G, China | halved by fast dormancy |
| Transfer power slope, uplink / downlink | LTE 438 / 52; Wi-Fi 283 / 137 | mW per Mbps | Huang 2012 | HTC LTE phone | n/a |
| Transfer base power | LTE 1288; Wi-Fi 133 | mW | Huang 2012 | same | n/a |
| Bulk energy per bit | 10 falling to under 1 | µJ/bit | Huang 2012 | same | lower on newer radios at high rates |
| 10 MB download, LTE against Wi-Fi | 1.62 | ratio | Huang 2012 | same | n/a |
| 5G against 4G at low throughput | 79 % (down), 74 % (up) less efficient | per bit | Narayanan 2021 | S20 Ultra / S10, Verizon and T-Mobile | unknown since 2021 |
| Phone idle, screen off | 11.4 | mW | Huang 2012 | HTC, 2012 | n/a |

## A3. Upstream energy for a comparable workload, and whether network energy scales with traffic

### A3.1 Networks: measured national totals per subscription (Arcep, 2024 data)

Same document as A1.1.
- "La consommation énergétique des réseaux fixes et mobiles se stabilise en 2024 à 4,1 TWh" (3.8, 3.9, 4.1, 4.1, 4.1 TWh for 2020-2024), under 1 % of French electricity (442 TWh).
- "la consommation énergétique annuelle par abonnement est deux fois plus faible sur les réseaux fixes (20 kWh par abonnement hors consommation électrique des box et décodeurs TV) que sur les réseaux mobiles (40 kWh par carte SIM) ... un abonnement cuivre consomme en moyenne 40 kWh par an, contre moins de 10 kWh pour un abonnement en fibre optique".
- Mobile: energy per gigabyte on mobile access "s'élève à 0,19 kWh en 2024 (- 10 % en un an) contre 0,21 kWh en 2023", falling 12 % a year since 2020; mobile access energy still rising (+14 % 2022, +6 % 2023, +4 % 2024) while mobile data grew +20 % then +13 %. (A search summary quoted 0.14 kWh mobile against 0.02 kWh fixed per GB, seven to one; I did not find that sentence in the PDF passages I read `[search summary]`.)
- Data centres in France (the operators Arcep surveys, i.e. colocation, not all servers [my reading]): 2.7 TWh in 2024, +12 %; IT equipment 1.9 TWh; "le PUE de l'ensemble des centres de données étudiés s'élève à 1,42 en 2024".
- **Verdict: usable now.** First choice for per-subscription network energy, because it is a regulator's collection from operators and separates fibre, copper, mobile, and the home box.

### A3.2 Does network energy scale with traffic? The debate, by side

- **Average-allocation side (energy per GB).** Aslan et al. 2018: 0.06 kWh/GB in 2015, halving every two years (F6). Used by most footprint calculators. Arcep itself publishes kWh per GB for mobile.
- **Power-model side (baseload per line plus a small per-rate term).** Mytton, Lundén and Malmodin, "Network energy use not directly proportional to data volume: The power model approach for more reliable network energy consumption calculations", Journal of Industrial Ecology 28(4), 966-980, August 2024, doi 10.1111/jiec.13512, open access per the author's page (https://davidmytton.blog/?p=4707, 24 June 2024). Abstract: "It is commonly assumed that data volume and network energy consumption are directly proportional ... This paper challenges this assumption ... explain why the relationship between energy consumption and data volume is nonlinear ... simple energy intensity calculations, expressed as kilowatt hours per gigabyte of data, are insufficient for accurately estimating real-world network energy consumption." Operator evidence cited there: traffic more than tripled since 2016 while network energy fell 2.4 % at Telefónica and 50 % at Cogent `[search summary of the same page]`. Carbon Trust 2021 (F5) uses this model.
- **Measured support for the power-model side from a regulator**: Arcep's box measurements (A1.1): +0.1 W at 5 Mbit/s on a 9.1 W box, i.e. about 1 % ; and France's network total flat at 4.1 TWh for three years while traffic grew about 10 % a year fixed and 13 to 20 % mobile.
- **Where traffic does cost energy.** Mobile radio access: Carbon Trust's 1.5 W per Mbps marginal term, and Arcep's mobile access energy still rising with data. And on the handset (A2). So the per-byte view is closer to right on cellular and close to wrong on fixed.
- **The long-run objection** [my reading; not from a page read today]: networks are provisioned for peak traffic, so sustained traffic growth is what causes new capacity, sites and generations to be built; a per-GB figure is a crude proxy for that long-run cost, and the power model is a short-run statement. Neither side has published a peak-attribution number an outsider can check.
- Koomey and Masanet, "Does not compute: Avoiding pitfalls in assessing the Internet's energy and carbon impacts" (Joule, 2021) makes the general warning against extrapolating per-GB intensities `[my reading; not opened]`.

### A3.3 Data-centre energy attributable to messaging, feeds or small file sync

- **I found no published per-user or per-GB figure for messaging, social feeds or file sync that separates them from the operator's total.** This is a gap, stated as such.
- What exists:
  - Company totals in sustainability reports (Meta, Google, Microsoft) that could be divided by users to give a ceiling of a few kWh per user per year across ALL of the company's workloads including advertising and AI [my reading; Meta's data page did not render and no number was read].
  - Carbon Trust 2021 for video streaming: of one hour of European streaming, network transmission is "20Wh/hour (10% of total ...)", home router "71Wh/hour (38%)", viewing device about half, which leaves data centres and CDN at about 1 % [my arithmetic from the stated shares]. Streaming is the best-measured workload and its data-centre share is tiny.
  - A per-request benchmark at the other extreme: Google reports a median Gemini text prompt at 0.24 Wh including idle capacity and data-centre overhead (0.10 Wh for active accelerators only), May 2025 data, self-reported and not independently verified `[search summary of Google's paper, arXiv 2508.15734]`. A message relay or feed fetch is orders of magnitude below an inference call [my reading].
  - Data-centre overhead multipliers: PUE 1.09 (Google 2025), 1.42 (France, Arcep 2024), 1.54 (Uptime 2025 `[search summary]`).
- Reading for the theory: the upstream data-centre energy displaced by moving a message or a small sync off a server is small against everything else in the chain, and cannot be given a checkable number today. If the simulation needs one, it has to be a stated bound (for example, the company-total-per-user ceiling), not a measurement.

## Revised constants for the delta calculation (supersede the corresponding rows above)

| Quantity | Value | Unit | Source, data year | Measured on | Rate of change |
|---|---|---|---|---|---|
| Internet box, idle with Wi-Fi | 9.1 (3.3 to 25.0) | W | Arcep 2026 ed., 2024 | 36 models, four French ISPs, MEASURED | flat; newer fibre boxes 8.0 |
| Box marginal on traffic | +0.1 at 5 Mbit/s; +0.3 (Ethernet) to +0.8 (Wi-Fi) at 50 Mbit/s | W | same | xDSL/FttH boxes | n/a |
| Box Wi-Fi radio | 1.8 (to 2.6) | W | same | 36 models | up with Wi-Fi 7 |
| Box with built-in disk | 19.4 | W | same | subset of models | n/a |
| Wi-Fi repeater, idle | 5.6 (to 9.6) | W | same | 12 models | +0.3 in a year |
| TV decoder, standby / in use | 4.1 (0.2 to 15.4) / 7.4 | W | same | 21 models | flat; fleet energy -3 % a year |
| Fixed network per subscription, all / fibre / copper | 20 / under 10 / 40 | kWh per year | same | French operators' reported energy | fixed networks falling as copper closes |
| Mobile network per SIM | 40 | kWh per year | same | same | access energy +4 % a year |
| Mobile access energy per GB | 0.19 | kWh/GB | same | same | -10 to -12 % a year |
| Network total, France | 4.1 | TWh | same | fixed + mobile | flat 2022-2024 |
| Whole-home always-on load | 164 (23 % of use) | W | NRDC 2015, 2014 data | 70,000 northern California smart meters | unknown since |
| Small transfer on phone, LTE / Wi-Fi | 12.76 / 0.04 | J | Huang 2012 | HTC LTE phone, US | timers similar in 2021 |
| Data-centre PUE, France | 1.42 | ratio | Arcep 2026 ed., 2024 | surveyed colocation operators | n/a |

## What this changes for a stake on "power savings"

1. **The status quo already contains about 9 W of router, often 4 W of set-top box, and (per NRDC) of the order of 160 W of always-on load per home.** A mesh that runs on the existing router adds nothing measurable (traffic moves the box by tenths of a watt). A mesh that needs one more always-on box adds about 5 to 10 W, which is more than the whole fixed network spends on that home (1 to 5 W).
2. **On phones the cost is set by radio wake-ups, not bytes**, and by which radio: a small exchange costs hundreds of times more on cellular than on Wi-Fi (2012 handset). A local-first design saves phone energy if it replaces cellular wake-ups with Wi-Fi ones or batches them, and costs phone energy if it adds keep-alives or holds a radio awake to relay. The relay cost on a current phone is unmeasured in anything I found.
3. **Upstream, the checkable saving is in the access network only where the technology is cellular**, and in the data centre it is too small and too unseparated to price. Fixed-network and data-centre savings from locality should be stated as bounds.
4. The cleanest measurable delta is therefore per household: (always-on boxes added or removed) x (measured watts from Arcep) + (phone radio wake-ups added or removed per day) x (joules per wake-up for that radio). Both terms have a reference; the second needs a current-handset measurement to be trusted.
