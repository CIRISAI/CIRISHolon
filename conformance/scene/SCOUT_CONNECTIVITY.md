# SCOUT: home connectivity by country — what is published, of which kind, under which licence

Scouted 2026-11-12. Documentation only: publishers' landing pages, indicator-definition and metadata pages, terms-of-use and licence pages. No dataset file was downloaded, no bulk or data API was called, no data explorer was exported, no account was made, no form was filled, no terms were accepted. Nothing is proposed for fetching here except by name in section 6.

Marking convention. Unmarked = read on the publisher's own page at the URL beside it. `[summary]` = from a search result only; the page was not opened. `[mirror]` = read on a third party's copy. `[my reading]` = my inference. `[not read]` = could not open.

Three cautions on how pages were read.

- Most web pages were read through a fetch tool that returns a model's extraction of the page, not the raw page. Quotes from those pages carry one more step of possible transcription error.
- Two publishers (ITU DataHub, OECD) refused the fetch tool (HTTP 403). Their pages were opened in a browser tab and read as page text: `datahub.itu.int/about/`, `datahub.itu.int/indicators/`, `datahub.itu.int/data/?i=12047`, `oecd.org/en/about/terms-conditions.html`, `oecd.org/en/topics/sub-issues/broadband-statistics.html`. Those quotes are verbatim page text.
- **One overreach, said plainly.** Opening the ITU indicator page for "Households with Internet access at home" to read its definition also rendered the page's full latest-value list (about 200 economies). I did not export or save it. I quote six values from it below as illustration and no more. The lead may count this as more than "a handful of figures printed in text"; it was a rendered explorer page, not a report.
- Three metadata PDFs were opened and read as extracted text: the World Bank Data360 metadata sheet for the ITU fixed-broadband series, the UN SDG metadata sheet for indicator 17.6.1, and ITU's household-indicator definitions. They are documents, not data.

Two cautions carried beside every figure:

- **Kind.** "Connectivity" is at least four different quantities: (1) share of *households* with internet access at home, any network, mobile included; (2) share of *households* with a fixed broadband connection; (3) fixed-broadband *subscriptions* per 100 *inhabitants*, business lines included; (4) share of *individuals* who used the internet, anywhere. They have different numerators and different denominators. One kind per table.
- **A constant is a price in a regime.** Every value has a year, a survey or an administrative count behind it, and a definition vintage. The EU household share printed by Eurostat is 93.1 % for 2023 `[summary]` and 94.7 % for 2025.

---

## Headline findings

1. **One publisher has all three indicators at country level: ITU DataHub.** Its catalogue lists "Households with Internet access at home", "Households with a computer" and "Fixed-broadband subscriptions" as separate indicators (https://datahub.itu.int/indicators/). Everyone else at world scale republishes ITU.
2. **ITU's own licence allows a republished derived table, with share-alike.** "The Data is licensed to the public through a Creative Commons Attribution-ShareAlike 3.0 IGO." (https://datahub.itu.int/about/, read in full.) This is CC BY-SA, not CC BY. A derived table must carry the same or a compatible licence.
3. **The same ITU series carries three different licence labels in three places.** ITU: CC BY-SA 3.0 IGO. World Bank WDI metadata glossary: "CC BY-4.0". World Bank Data360 metadata sheet for the ITU DataHub series: "CC BY-NC-SA 3.0 IGO". The publisher's own page is the one to follow `[my reading]`. The World Bank labels disagree with each other and with ITU; neither should be relied on for ITU-origin series.
4. **WDI does not carry the household indicators.** The WDI glossary page for IT.CMP.HOUS.ZS (households with a computer) says: "This indicator is not available in the World Development Indicators time series database." No WDI code for households with internet access was found. WDI carries only (c), as IT.NET.BBND.P2.
5. **ITU's household-internet indicator counts mobile access.** "Access can be via a fixed or mobile network." A household where one member shares a phone connection counts. This is not the kind for "where could a home node be hosted".
6. **The fixed-connection household kind exists only regionally.** Eurostat has households with fixed broadband (H_BBFIX) `[summary]`; the US ACS table B28002 has "Broadband such as cable, fiber optic or DSL" `[mirror]`; ITU has "Households with access to Internet, by type of service" (catalogue entry read, definition and coverage not read).
7. **Households with a computer is thin and ageing.** Eurostat's table is reported to end in 2017 `[summary]`. ITU still lists it. Its definition changed what a computer is (tablets in, phones out).

---

## 1. Sources, by indicator

### 1.1 ITU DataHub — all three indicators (a), (b), (c)

- **Title / publisher.** ITU DataHub, International Telecommunication Union (ICT Data and Analytics Division). Landing page https://datahub.itu.int/ ; catalogue https://datahub.itu.int/indicators/ ("93 indicators").
- **The series one would fetch** (links read in the catalogue; none followed for download):
  - (a) "Households with Internet access at home" — view https://datahub.itu.int/data/?i=12047 , download link https://datahub.itu.int/query/?i=12047
  - (b) "Households with a computer" — view `/data/?i=8917`, download link `/query/?i=8917`
  - (c) "Fixed-broadband subscriptions" — view `/data/?i=19257`, download link `/query/?i=19257`. The World Bank's metadata sheet cites `/data/?i=19303` for the same title with unit "Subscriptions, Subscriptions per 100 inhabitants". Which id is the per-100 series: **not read**.
  - Related, for the caveats: "Households with access to Internet, by type of service" (`i=100001`), "Households with a computer, by type" (`i=100106`), "Persons per household", "Fixed-broadband subscriptions for organizations" (`i=19305`), "Individuals using the Internet" (`i=11624`).
- **Format and size.** Not stated on the pages read. The download link goes to a "Data query" tool; its output format was not looked at. Whether one query can return all three indicators in one file: **not read**.
- **Coverage.** "about 200 economies worldwide" for both the household set and the telecommunication set (https://datahub.itu.int/about/). Year coverage per indicator not stated on the pages read; the World Bank sheet gives 2000 to 2024 for fixed broadband.
- **Definitions.**
  - (a) "Refers to the proportion of households with Internet access at home. Access can be via a fixed or mobile network." Unit "% of households". Denominator: households (in-scope households of the national survey `[my reading]`).
  - (b) ITU definitions PDF, indicator HH4: "This is the proportion of households that have a computer. A computer refers to a desktop computer, a laptop (portable) computer or a tablet (or similar handheld computer)." (https://www.itu.int/en/ITU-D/Statistics/Documents/datacollection/ITU_ICT_HHindicators_Definitions_6Languages_2022.pdf). The World Bank glossary adds that phones and TVs with embedded computing are excluded.
  - (c) "Fixed subscriptions to high-speed access to the public Internet (a TCP/IP connection), at downstream speeds equal to, or greater than, 256 kbit/s, divided by population and multiplied by 100." (World Bank Data360 metadata sheet, https://data360files.worldbank.org/data360-data/metadata/ITU_DH/IT_NET_BBND.pdf). The UN SDG sheet adds: "It includes both residential subscriptions and subscriptions for organizations", includes satellite and fixed wireless, excludes mobile-cellular.
- **Caveats.**
  - (a) and (b) are household surveys run by national statistical offices and sent to ITU by questionnaire; (c) is an administrative count from regulators, who get it from operators. Different instruments.
  - Modelled values. Not stated on the pages read. On the rendered list, five small territories show the identical value 51.3 % for 2017, which looks like an imputed regional figure `[my reading]`. Many rows are dated 2024 for countries with no annual ICT survey that I know of `[my reading]`. The file's own flags must say which rows are ITU estimates; **not read**.
  - Stale rows. The latest value for some economies is very old (Réunion 2000, Anguilla 2001 on the rendered list). A "latest year" map would mix 2000 with 2025.
  - (c) is not households: business lines are in the numerator, people are the denominator.
  - (b) breaks: the definition of "computer" has been revised (tablets added) `[my reading]` from the 2022 definitions; year of change **not read**.
  - Collection changed in 2025: "Since 2025, data are collected once a year" for the telecommunication indicators.
- **Illustration of range** (six values, read on the rendered explorer page, `i=12047`, households with internet at home): Central African Rep. 4.54 % (2019); Mozambique 11.1 % (2024); India 86.3 % (2025); United States 94.9 % (2024); Norway 99.5 % (2024); Korea (Rep. of) 100 % (2025).
- **Licence (read in full).** "The Data is licensed to the public through a Creative Commons Attribution-ShareAlike 3.0 IGO." And: "you may copy, redistribute, and adapt the Data, provided the work is appropriately cited." (https://datahub.itu.int/about/)
- **Third-party clause.** "The DataHub features some third-party data, as indicated in the source on the indicator page." The page for (a) shows "Source: ITU". Sources for (b) and (c): **not read**.
- **Redistribution of a derived table.** Allowed, with attribution, under share-alike. No non-commercial term in ITU's own text.
- **Attribution asked.** "Suggested citation: International Telecommunication Union, ITU DataHub, https://datahub.itu.int/." With: "It is recommended that users indicate the access date". Also: "The unauthorized use of the ITU name or logo is not permitted."
- **Do not confuse with** ITU's general website terms, which are restrictive: "You may not modify, reproduce, distribute, sell, transmit, create derivative works" without written permission (https://www.itu.int/en/about/Pages/terms-of-use.aspx). Those cover the website; the DataHub page states its own licence. The old paid "World Telecommunication/ICT Indicators Database" is reported discontinued and folded into the DataHub `[summary]`.
- **Not a source for this table:** the Excel file on ITU's statistics page (`ITU_regional_global_Key_ICT_indicator_aggregates_Nov_2026.xlsx`, 2005 to 2026) holds world and regional aggregates only, and no household indicators (https://www.itu.int/en/ITU-D/Statistics/Pages/stat/default.aspx).

### 1.2 World Bank, World Development Indicators — (c) only

- **Title.** "Fixed broadband subscriptions (per 100 people)", code IT.NET.BBND.P2. Page https://data.worldbank.org/indicator/IT.NET.BBND.P2 . Source line: "World Telecommunication/ICT Indicators Database, International Telecommunication Union ( ITU )". Years "1998 - 2025".
- **File.** Per-indicator download from that page (format not read). The whole WDI is one bulk file: CSV ZIP 270 MB or Excel ZIP 78 MB, updated Oct 9, 2026, "more than 200 countries and territories", 1960 to 2025 (https://datacatalog.worldbank.org/search/dataset/0037712/World-Development-Indicators). Far more than is needed.
- **Definition.** As ITU's: at least 256 kbit/s, mobile-cellular excluded, "both residential and organizational subscriptions", per 100 people (https://databank.worldbank.org/metadataglossary/world-development-indicators/series/IT.NET.BBND.P2).
- **Caveat.** The glossary's limitations text says "Data are collected by national statistics offices through household surveys." That contradicts ITU and the SDG sheet (administrative, from operators). It reads like text copied from a household indicator `[my reading]`.
- **Households.** (b) IT.CMP.HOUS.ZS "Households with a computer (%)" has a glossary page that says: "This indicator is not available in the World Development Indicators time series database." (a): no WDI code found. **Not found.**
- **Licence as labelled by the World Bank.** Glossary pages for both codes: License Type "CC BY-4.0", URL https://datacatalog.worldbank.org/public-licenses#cc-by . The WDI catalogue page: "This dataset is licensed under Creative Commons Attribution 4.0".
- **The third-party question.** The World Bank's licence page says CC BY 4.0 is "the default license for all Datasets produced by the World Bank itself", and that for third parties "the World Bank provides access to datasets from third parties according non-standard licenses." It does not name ITU. The World Bank's own Data360 metadata sheet for the ITU DataHub series states: License "CC BY-NC-SA 3.0 IGO". So the World Bank says CC BY 4.0 in one place and CC BY-NC-SA 3.0 IGO in another for the same origin, and ITU says CC BY-SA 3.0 IGO.
- **Finding.** Do not take the WDI "CC BY-4.0" label as clearing ITU-origin data for a CC BY table. Fetching from the World Bank does not launder ITU's share-alike term `[my reading]`.
- **Attribution asked.** None quoted on the pages read. **Not found.**

### 1.3 Eurostat — (a) for Europe, with regions; (b) to 2017; no per-100 series read

- **Publisher / survey.** Eurostat, "ICT usage in households and by individuals". Metadata https://ec.europa.eu/eurostat/cache/metadata/en/isoc_i_esms.htm .
- **Tables.**
  - (a) country: `isoc_ci_in_h`, "Households - level of internet access" `[summary]` for the title; the Eurostat data-browser page returned no text to the fetch tool.
  - (a) regional: `isoc_r_iacc_h`, "Households with access to the internet at home" `[summary]`; reported coverage 2006 to 2024 `[mirror, summary]`.
  - Fixed-connection household kind: `isoc_ci_it_h`, "Households - type of connection to the internet", series H_BBFIX, reported 2003 to 2021 `[summary, mirror]`. Whether it continues after 2021: **not read**.
  - (b): `isoc_ci_cm_h`, "Households - availability of computers", reported 2006 to 2017 `[summary, mirror]`. The Eurostat metadata page does not mention household computers at all. Treat (b) from Eurostat as ended in 2017, not confirmed at source.
  - (c): no Eurostat subscriptions-per-100 table was read. **Not found.**
- **Format / size.** Not read (no data page opened).
- **Coverage.** "EU Member States, Iceland, Switzerland, Norway, and candidate and potential candidate countries" (extraction). "Time series (annual data) are available from 2003 onwards."
- **Definition and denominator.** Population: "all private households having at least one member in the age group 16 to 74 years." Household internet access is any connection a member could use; mobile is not excluded `[my reading]`. This denominator is narrower than "all households": households of people all aged 75 or over are outside it.
- **Regions.** NUTS 1 mandatory since 2008, NUTS 2 optional (metadata, extraction). Statistics Explained: data are generally NUTS 2; Germany, Greece and Türkiye at NUTS 1; Croatia, Norway and Albania national only (https://ec.europa.eu/eurostat/statistics-explained/index.php?title=Digital_society_statistics_at_regional_level).
- **Caveats.** Annual sample survey, about 172 000 households across the EU in 2025. Questions change; some series break; some items asked every other year.
- **Headline.** "In 2025, the share of EU households connected to the internet was 94.7%." (Statistics Explained, read in text.)
- **Licence (read).** Reuse "for commercial or non-commercial purposes is authorised provided the source is acknowledged." Editorial content is "licensed under the Creative Commons Attribution 4.0 International licence." (https://ec.europa.eu/eurostat/help/copyright-notice). Exception: the permission "does not extend to any material whose copyright is identified as belonging to a third-party."
- **Redistribution of a derived table.** Allowed with attribution. If modified, "A disclaimer regarding the non-responsibility of Eurostat shall be included."
- **Attribution asked.** "Source: [digital object identifier (DOI) number of the Eurostat dataset], [access date]".

### 1.4 OECD — (c) for OECD members

- **Title / publisher.** "Broadband statistics", OECD. https://www.oecd.org/en/topics/sub-issues/broadband-statistics.html . Page sections read: "Penetration and data usage", "Speeds", "Household and business usage", "Price and mobile termination rates", "Coverage and geography", and "Data downloads". Latest statistical release dated 13 October 2026.
- **Series.** Going Digital Toolkit indicator "Fixed broadband subscriptions per 100 inhabitants", advertised download speed of 256 kbit/s or more (https://goingdigital.oecd.org/en/indicator/10). File names, formats, sizes, country list, latest period: **not read** (they sit behind the download and explorer controls).
- **Households.** A "Household and business usage" section exists. What it holds: **not read**.
- **Caveats.** OECD members and partners only `[my reading]`; no use for a world map. Half-yearly, from regulators `[my reading]`.
- **Licence (read on the terms page, in part).** Data section: "Data may be subject to restrictions beyond the scope of these Terms and Conditions". Permitted Use begins: "Except where additional restrictions apply as stated above, you can extract from, download, copy, ad" — the rest of the sentence was cut off by the page reader. **Operative sentence only partly read.** The Going Digital page footer says "© 2026 OECD. All rights reserved."
- **Redistribution.** Probably allowed with attribution, per the cut-off sentence and the OECD's stated default of CC BY 4.0 for content from 1 July 2024 `[summary]`. Not confirmed. **Attribution wording: not read.**

### 1.5 UN SDG Global Database — (c) again, same origin

- **Indicator.** 17.6.1, "Fixed broadband subscriptions per 100 inhabitants, by speed"; series IT_NET_BBND. Custodian: ITU. Metadata https://unstats.un.org/sdgs/metadata/files/Metadata-17-06-01.pdf (read as text, last updated 2024-07-29).
- **Useful sentences.** "It includes both residential subscriptions and subscriptions for organizations." And: data "could in theory be broken down by geographic location and urban/rural, but the International Telecommunication Union (ITU) does not collect this information." So there is no sub-national (c) from ITU.
- **Adds over ITU.** The speed tiers (256 kbit/s to 2 Mbit/s; 2 to 10; 10 and above). Nothing else. No household indicator for (a) or (b) was found among SDG series.
- **Licence.** The database page returned no text. The general UN site terms allow only "personal, non-commercial use, without any right to resell or redistribute them" (https://www.un.org/en/about-us/terms-of-use). Whether the SDG database has its own, more open terms: **licence not read.** No reason to prefer this over ITU directly.

### 1.6 US Census Bureau, American Community Survey — sub-national, United States

- **Tables.** B28002, "Presence and Types of Internet Subscriptions in Household"; universe "Households" `[mirror]` (https://censusreporter.org/tables/B28002/). Columns include "With an Internet subscription", "Broadband of any type", "Cellular data plan", "Cellular data plan with no other type of Internet subscription", "Broadband such as cable, fiber optic or DSL", "Satellite Internet service", "Internet access without a subscription", "No Internet access". S2801, "Types of Computers and Internet Subscriptions": title only `[summary]`; the data.census.gov page returned no text. A forum post claims the B28002 universe is housing units `[summary]`; the table shell at source would settle it.
- **Coverage.** ACS 1-year, 2005 to 2025 releases; "the nation, all states, DC, Puerto Rico"; counties and places only at 65 000 people or more (https://www.census.gov/data/developers/data-sets/acs-1year.html). The computer and internet questions "were added to the ACS in 2013 and modified in 2016" (https://www.census.gov/acs/www/about/why-we-ask-each-question/computer/). So the series starts in 2013 and breaks at 2016.
- **Fetch.** The API needs a key ("Required for all queries"). A key is an account; out under the rules. A table download from data.census.gov needs none `[my reading]`; format and size **not read**.
- **Kind.** Household, with the fixed line separable from the cellular-only household. This is the best match found anywhere for "households with a fixed connection". It also gives (b) for states. It does not give (c).
- **Caveat.** Sample survey with margins of error. "Internet subscription" is paid service; "Internet access without a subscription" is a separate row. Not the same question as Eurostat's or ITU's.
- **Licence.** No sentence on census.gov stating public domain was found on the pages opened (open-data page, citation page, API terms). The open-data page says the Bureau shares "its public data as open data." The usual basis, 17 U.S.C. §105, was seen only on third-party pages `[summary]`. API terms (apply if the API is used): "You may not modify or falsely represent content accessed through the API and still claim the source is the Census Bureau." And the notice: "This product uses the Census Bureau Data API but is not endorsed or certified by the Census Bureau." (https://www.census.gov/data/developers/about/terms-of-service.html)
- **Redistribution.** Allowed `[my reading]`; **public-domain statement not read at source.**
- **Attribution asked.** Format: `Author/s, "Table Name," Full Product Name, Vintage, <URL>, accessed on date.` with author "U.S. Census Bureau" (https://www.census.gov/about/policies/citation.html).

### 1.7 Others looked at

- **Our World in Data** republishes (c) as "International Telecommunication Union (ITU), via World Bank (2026) – processed by Our World in Data", and says third-party data "are subject to the license terms from the original providers." (https://ourworldindata.org/grapher/broadband-penetration-by-country). It is a third hand on ITU data and does not change the licence. Not a candidate.
- **National statistics compilations** beyond Eurostat and ACS: none scouted. **Not found.**

---

## 2. Ranking

| Indicator | First to put to the lead | Why | Next-best under plain CC BY 4.0 |
|---|---|---|---|
| (a) households with internet at home | ITU DataHub, `i=12047` | Only world-wide source; origin of everyone else's numbers; licence read | Eurostat `isoc_ci_in_h`: Europe only, about 35 to 40 countries |
| (b) households with a computer | ITU DataHub, `i=8917` | Only world-wide source; WDI does not carry it | Eurostat `isoc_ci_cm_h`: Europe only, ends 2017 `[summary]` |
| (c) fixed-broadband subscriptions per 100 inhabitants | ITU DataHub, `i=19257` (or `19303`) | Origin of the WDI, SDG and OWID series | OECD broadband statistics: members only, terms part-read. WDI IT.NET.BBND.P2 is labelled CC BY-4.0 but is ITU's data; the label is in doubt |

- **The single fetch that covers the most** is the ITU DataHub: one publisher, one licence, one citation, three indicators. It is three series, and I could not confirm that they come as one file.
- **The licence, said plainly.** ITU's licence does allow a republished derived table. It is not plain attribution: it is share-alike (CC BY-SA 3.0 IGO). A table derived from it must be offered under the same or a compatible licence, and must say it was changed. GRID-1's table is CC BY 4.0 material; an ITU-derived table should be its own file with its own licence block, not rows merged into a CC BY table `[my reading]`. This is the lead's call.
- **If share-alike is not acceptable,** the openly licensed remainder is: Europe only for (a); Europe to 2017 for (b); OECD members for (c), subject to reading the rest of the OECD sentence. No world map can be drawn from it.
- **Sub-national.** United States: ACS B28002 (and S2801), by state, households, fixed line separable. Europe: Eurostat `isoc_r_iacc_h`, NUTS 2 with the stated exceptions. No sub-national (c) exists at ITU.

---

## 3. Kinds to keep apart

| Kind | Numerator | Denominator | Instrument | Sources |
|---|---|---|---|---|
| K1 households with internet at home, any network | households | households | survey | ITU `12047`; Eurostat `isoc_ci_in_h`, `isoc_r_iacc_h` |
| K2 households with a fixed broadband connection | households | households | survey | ACS B28002 row "cable, fiber optic or DSL"; Eurostat H_BBFIX `[summary]`; ITU `100001` (not read) |
| K3 households with a computer | households | households | survey | ITU `8917`; Eurostat to 2017 `[summary]`; ACS S2801 `[summary]` |
| K4 fixed-broadband subscriptions per 100 inhabitants | subscriptions, homes and organisations | people | administrative | ITU `19257`; WDI IT.NET.BBND.P2; SDG 17.6.1; OECD |
| K5 individuals using the internet | people | people | survey | ITU `11624`; WDI IT.NET.USER.ZS `[summary]` |

- **"Where could a home node be hosted"** needs **K2**: a household with a fixed line. K2 has no world-wide table that I could confirm. K4 is the world-wide proxy for fixed lines, with the wrong denominator. K1 overstates it wherever homes are online by phone only: ITU's own definition counts a shared phone.
- **"Who is online at all"** is **K5**. Mobile included, any place of use. It is not asked for here and must not be mixed in.
- Indicator (a) as asked is K1. The downstream table should say on its face that K1 includes mobile-only homes.
- Do not mix Eurostat's denominator (households with a member aged 16 to 74) with ITU's or the ACS's (all in-scope households) in one column without a flag.

---

## 4. What the map would and would not license

- A K1 share says a home has some internet access. It does not say the home has a fixed line, a wired router, or a machine that stays on.
- No household share is reachability. Carrier-grade NAT, lack of a public address, uplink speed, data caps and power reliability are separate facts, not in any of these tables.
- K4 is not a household share. Household size varies by country (ITU lists "Persons per household" as its own indicator), and business lines are in the count. 40 subscriptions per 100 people can mean nearly every home in one country and far fewer in another. Converting K4 to households is an estimate; I have not made one and the table should not.
- The 256 kbit/s floor in K4 is a 2000s definition. A subscription that qualifies may be far too slow to serve anything.
- "Latest value" maps mix years. On the ITU list the latest year runs from 2000 to 2025.
- Some values are modelled by ITU. Until the estimate flags are read in the file, no row should be called a survey result.
- A country value hides the inside. Eurostat regions and US states show the spread; nothing here shows urban against rural except ITU aggregates.
- A computer in the home (K3) is not a host. Tablets count; an always-on machine is not asked about.

---

## 5. Not found / not defensible

- A world-wide **K2** table (households with a fixed connection): not found. ITU `i=100001` may be it; its definition and coverage were not read.
- **(a) and (b) in the WDI**: not there. IT.CMP.HOUS.ZS is "not available in the World Development Indicators time series database"; no household-internet code found.
- **(b) after 2017 for Europe**: not found at Eurostat `[summary]`.
- **(c) at sub-national level**: not found. ITU does not collect it. Eurostat and ACS regional tables are household kinds, not subscriptions per 100 people.
- **Which ITU values are modelled**: not read.
- **Format and size of any ITU or Eurostat file**: not read.
- **OECD's full permitted-use sentence and citation wording**: not read (cut off).
- **A census.gov sentence stating public domain**: not found on the pages opened.
- **UN SDG database terms**: not read.
- **Eurostat table titles and year ranges at source**: the data-browser pages gave no text; codes and ranges are from search summaries and mirrors.
- **A consistent licence label for ITU-origin series at the World Bank**: there is none; two labels conflict.
- NUTS-2 for every EU country: not available; Germany, Greece and Türkiye are NUTS 1, Croatia national.

---

## 6. For approval by name (at most four)

1. **ITU DataHub — "Households with Internet access at home", "Households with a computer", "Fixed-broadband subscriptions"** — International Telecommunication Union — https://datahub.itu.int/indicators/ (ids 12047, 8917, 19257) — CC BY-SA 3.0 IGO (read) — world-wide (a), (b), (c), about 200 economies; share-alike applies to the derived table.
2. **Eurostat `isoc_ci_in_h` with regional `isoc_r_iacc_h`** — Eurostat — https://ec.europa.eu/eurostat/databrowser/view/isoc_ci_in_h/default/table?lang=en — reuse authorised with source acknowledged, CC BY 4.0 (read) — (a) for Europe by country and by NUTS 2 / NUTS 1 region; titles and years to be confirmed at fetch.
3. **American Community Survey 1-year, table B28002 (and S2801), by state** — U.S. Census Bureau — https://data.census.gov/ — US government work; public-domain sentence not read at source; citation format read — US states: households with any subscription, with a fixed line, cellular only, none; computers via S2801. No API key route.
4. **OECD broadband statistics, fixed broadband subscriptions per 100 inhabitants** — OECD — https://www.oecd.org/en/topics/sub-issues/broadband-statistics.html — OECD terms, permitted-use sentence only part-read — (c) for OECD members only; a cross-check on ITU, not a replacement. Lowest priority.

Not offered: World Bank WDI IT.NET.BBND.P2. It is ITU's series under a licence label ("CC BY-4.0") that ITU's own page and the World Bank's own Data360 sheet both contradict.
