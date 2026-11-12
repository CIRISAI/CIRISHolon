# SCOUT: carbon intensity of electricity by place — what is published, of which kind, and how far it varies

Scouted 2026-11-12. Documentation only: agency pages and report PDFs, one grid operator's annual review, one think tank's review pages and methodology PDF, one analyst article, one third-party aggregator's country pages. No dataset file was downloaded, no data explorer was exported, no API was called, no account was made, no terms were accepted. Report PDFs were opened and read as documents. Rows are in `conformance/scene/tables/grid0/grid_table.json` (271 rows).

Marking convention (same as the earlier scouts). Unmarked statements were read in the source at the URL beside them. `[mirror]` = read in full from a copy not hosted by the publisher. `[search summary]` = seen only in a search engine's summary; the source was not opened. `[my reading]` = my inference or recollection. `[my arithmetic]` = computed by me from figures read. `[not confirmed]` = could not open the source.

One caution on how pages were read. Web pages were read through a fetch tool that returns a model's extraction of the page, not the raw page. PDFs were different: the eGRID tables were read as page images, and the Canadian inventory, the German UBA report, the Indian CEA guide, the South African gazette, the UK changes report and Ember's methodology were read as extracted text by me. Figures from web pages (Ember, IEA, RTE, MEE, the Australian regulator, Carbon Brief, lowcarbonpower) therefore carry one more step of possible transcription error than figures from PDFs. The US state table (52 values) was transcribed by eye from a page image and has not been checked a second time.

Two cautions, carried beside every figure:

- **Kind of figure.** A grid's "carbon intensity" is at least five choices: (1) generation mix or consumption mix (imports counted); (2) direct stack emissions or life-cycle; (3) CO2 only or all gases; (4) per kWh generated or per kWh delivered after losses; (5) average or marginal. The JSON `kind` field states them. Germany 2025 reads **330, 344, 352 or 406 g/kWh** in one official table depending on kind. France 2025 reads **19.6** (direct) or **29.0** (life-cycle) from the grid operator, and **45** from an aggregator with generic factors.
- **A constant is a price in a regime.** Every value has a year and a method vintage. Ember printed **369** for the United States in 2023 in its 2024 review and **393** for the same year a year later, after changing its factors. The UK company-reporting factor fell **26 %** in one edition, of which 6 to 7 points were method.

---

## Headline findings

1. **The world figure depends on kind by 5 %.** 2025: **458 gCO2e/kWh** life-cycle (Ember, Global Electricity Review 2026) against **435 gCO2/kWh** direct CO2 (IEA, Electricity 2026). Both are generation averages. Neither is wrong.
2. **Among the large economies, in one kind and one year, the range is 6 to 1.** Ember, 2025, life-cycle generation average: Brazil **110**, EU **210**, US **384**, Russia **450**, world **458**, Japan **477**, China **525**, India **670**.
3. **At country level across official national sources the range is about 50 to 1, but only with the kinds named.** France **19.6 gCO2e/kWh** (RTE, 2025, direct, per kWh generated) to South Africa **994 gCO2e/kWh** (DFFE, 2024, direct, per MWh generated). Both are official, direct, generation-based. Norway and Sweden are of France's order (28 to 39) but only from third-party pages (trust C).
4. **Inside one country the range is as wide as between countries.** Canada, 2023, generation, direct: Quebec **1.5**, Manitoba **2.2**, Ontario **54**, Alberta **420**, Saskatchewan **590**, Nunavut **780** gCO2e/kWh (ECCC). China, 2022: Yunnan **107**, Sichuan **140**, Hebei **725** gCO2/kWh (MEE). United States, 2023, eGRID subregions: NYUP **109.8** to MROE **633.8** on the mainland, PRMS **699.9** with islands, gCO2/kWh. US states: Vermont **19.5** to West Virginia **886.5** — but a state is not a grid.
5. **The marginal map is flatter and higher than the average map.** eGRID2023 non-baseload rates run **401.5** (NEWE) to **858.2** (SPNO) gCO2/kWh: a ratio of 2.1, against 6.4 for the averages. The US non-baseload rate is **622.6**, the average **348.0**. Upstate New York is 109.8 on average and 412.5 at the margin. A map drawn with averages and a map drawn with margins are different maps.
6. **It moves by 1.5 to 5 % a year at national scale, faster in some regions, and not always down.** World −1.5 % a year (Ember, 2015 to 2025). US average −4.1 % a year (eGRID, 2018 to 2023) while US non-baseload moved −0.8 % a year. Germany −4.2 % a year over ten years but only −1.2 % over the last five, with a rise in 2021 and 2022. Alberta −7.3 % a year. South Africa **rose** from 942 to 994 in one year. Ontario **doubled** from 27 to 54 in five years. New York City's subregion rose from 270.5 to 392.1 g/kWh between eGRID2018 and eGRID2023. All rates `[my arithmetic]`.
7. **For most of the countries the brief names, no figure could be read at the primary source.** Ember's review pages give intensity for seven countries and the EU only; its country pages give shares and no intensity; the rest is in its dataset. The EEA indicator page gives no absolute values in text. The 17 rows from lowcarbonpower.org and the 11 from search summaries fill the list at trust C. Approval of one dataset (Ember's yearly CSV, CC BY 4.0) would replace nearly all of them.

---

## 1. Country-level averages

### 1.1 Ember — generation average, life-cycle, gCO2e/kWh (trust B: read at source, modelled)

Method, read in Ember's methodology PDF (v1.6): emissions are generation multiplied by life-cycle factors that "aim to include full lifecycle emissions including upstream methane, supply chain and manufacturing emissions, and include all gases, converted into CO2 equivalent over a 100-year timescale". Coal factors are per country and year (UNECE 2022 and plant-tracker data). Gas factors are per country, fixed at 2017. Bioenergy 230, hydro 24, solar 48, other fossil 700 g/kWh from IPCC AR5. Ember says its bioenergy value "is very likely to be a significant underestimate". It is a production mix: imports are not counted. Licence: CC BY 4.0.

| Place | 2023 (2024 edition, older method) | 2024 (2025 edition) | 2025 (2026 edition) |
|---|---|---|---|
| World | 480 | 473 | 458 |
| China | 581 | 560 | 525 |
| United States | 369 (later revised to 393) | 384 | 384 |
| EU-27 | 244 | 213 (2026 edition says 211) | 210 |
| India | 713 | 708 (2026 edition says 705) | 670 |
| Russia | 441 | 449 | 450 |
| Japan | 485 | 482 | 477 |
| Brazil | not on the page | 103 | 110 |
| Asia | | | 544 |
| Latin America | | | 247 |

Sources: https://ember-energy.org/latest-insights/global-electricity-review-2026/major-countries-and-regions ; .../global-electricity-review-2025/major-countries-and-regions/ ; .../global-electricity-review-2024/major-countries-and-regions . World 2015 = 533 (2026 edition, supply-trends page). US 2007 = 598 and 2023 = 393 (Ember, US Electricity 2025 Special Report).

The 2023 column is a different method vintage. The EU fall from 244 to 213 in one year is partly method. Do not compute a rate across that column boundary.

### 1.2 IEA — generation average, direct CO2, gCO2/kWh (trust B: read at source, IEA estimate)

IEA, Electricity 2026, Emissions chapter (https://www.iea.org/reports/electricity-2026/emissions ; text CC BY 4.0). "CO2 intensity is calculated as total CO2 emissions divided by total generation."

| Place | 2025 | IEA's 2030 forecast |
|---|---|---|
| World | 435 | 360 |
| European Union | 170 | 90 |
| China | 530 | 415 |
| India | 695 | 585 |
| Southeast Asia | 640 | 615 |

Same year, same places, against Ember: world 435 against 458; EU 170 against 210; China 530 against 525; India 695 against 670. The sign of the gap is not constant. Life-cycle should exceed direct; for China and India it does not. Both rows are kept. I cannot explain the sign from the pages read `[my reading: different coal factors and different generation totals]`.

IEA's country-by-country emission factors are a paid database and are excluded from the CC licence (IEA terms page, read).

### 1.3 National official sources

| Place | Year | Value | Kind | Source, how read | Trust |
|---|---|---|---|---|---|
| Germany | 2025 | 344 gCO2/kWh | direct CO2 of German generation per kWh **consumed**; not trade-adjusted | UBA, Climate Change 16/2026, Table 2, PDF read | B (estimated year) |
| Germany | 2025 | 330 gCO2/kWh | same, **trade-adjusted** | same | B |
| Germany | 2025 | 352 gCO2e/kWh | direct, all gases | same | B |
| Germany | 2025 | 406 gCO2e/kWh | **life-cycle**, all gases, with upstream chains | same | B |
| Germany | 2024 / 2023 / 2022 / 2020 / 2015 / 1990 | 353 / 379 / 433 / 365 / 529 / 765 | direct CO2 per kWh consumed | same | A for final years; 2024 preliminary |
| France | 2025 | 19.6 gCO2e/kWh | direct, per kWh generated | RTE, Annual Electricity Review 2025, page read | A |
| France | 2025 | 29.0 gCO2e/kWh | **life-cycle**, per kWh generated | same | A |
| France | 2024 | 21.7 and 21.3 | direct | two search summaries that disagree | C |
| Canada | 2023 | 100 gCO2e/kWh | generation, direct, utility sector | ECCC, NIR 1990-2023, Annex 13, PDF read | A (preliminary, rounded) |
| Canada | 2023 | 110 gCO2e/kWh | per kWh **consumed** after losses, plus SF6 | same | A |
| South Africa | 2024 | 994 gCO2e/kWh | generation, direct ("DGGEF") | DFFE gazette, September 2026, PDF read | A |
| South Africa | 2024 | 960 gCO2e/kWh | **consumption-based**, imports and exports counted ("NGGEF") | same | A |
| China | 2022 | 536.6 gCO2/kWh | national average, direct CO2 | MEE/NBS Announcement 2024 No. 33, page read | B (older) |
| China | 2022 | 585.6 gCO2/kWh | average **excluding market-traded non-fossil power** | same | B |
| China | 2022 | 832.5 gCO2/kWh | fossil generation only | same | B |
| China | 2023 | 530.6 | national average | third-party listing | C `[search summary]` |
| India | FY 2022-23 | 716 gCO2/kWh | average **including renewables**; CEA's "Grid Emission Factor" | CEA, User Guide v19.0, PDF read | B (older) |
| India | FY 2022-23 | 823 gCO2/kWh | average **excluding renewables** | same | B |
| India | FY 2023-24 | 727 | average including renewables | trade press | C `[search summary]` |
| United States | 2023 | 348.0 gCO2/kWh (767.2 lb/MWh) | generation, direct CO2 | EPA eGRID2023, PDF read | A |
| United Kingdom | 2025 | 126 gCO2/kWh | includes imports and biomass life-cycle | Carbon Brief analysis of NESO and DESNZ data, page read | B (analyst's estimate) |
| United Kingdom | 2025 data | 131 gCO2e/kWh | company-reporting factor, 2026 edition | aggregators | C `[search summary]`; only the −26 % was read at DESNZ |
| Japan | FY 2024 survey | 423 gCO2/kWh | all-source, **receiving end** (403 at sending end) | METI committee paper | C `[search summary]`; another summary gives about 450 |
| Australia | 2025-26 | by state only (section 3) | | | |

### 1.4 The rest of the brief's list — third-party pages only (trust C)

lowcarbonpower.org country pages. Kind: **consumption mix, life-cycle generic factors** (coal about 820, gas about 490 g/kWh for every country). Period: a rolling 12 months ending August to October 2026 unless stated. No licence or terms on the pages read. The site names Ember, IEA and the World Bank as sources.

| Place | gCO2e/kWh | Period | Same place from another source |
|---|---|---|---|
| Norway | 32 | to Oct 2026 | 28 (third party citing Ember) `[search summary]` |
| Sweden | 39 | to Oct 2026 | 35 (same); 27 (IEA 2023 via SDES) and 41 (Ember 2023 via Statista), both `[search summary]` |
| France | 45 | to Oct 2026 | RTE 19.6 direct, 29.0 life-cycle (2025) |
| Kenya | 115 | to Aug 2026 | none found |
| Canada | 132 | to Aug 2026 | ECCC 100 (2023, direct); 191 (third party citing Ember) |
| Spain | 146 | to Oct 2026 | none found |
| United Kingdom | 221 | to Sep 2026 | Carbon Brief 126 (2025) |
| Netherlands | 293 | to Oct 2026 | none found |
| Germany | 302 | to Oct 2026 | UBA 406 life-cycle, 344 direct (2025) |
| Italy | 310 | to Oct 2026 | none found |
| Nigeria | 383 | calendar 2024 | none found |
| Mexico | 405 | to Jul 2026 | 474 (third party citing Ember) |
| South Korea | 434 | to Aug 2026 | none found |
| Australia | 435 | to Oct 2026 | state factors, section 3 |
| Poland | 519 | to Oct 2026 | 636 (IEA 2023) and 662 (Ember 2023), both `[search summary]` |
| Indonesia | 625 | calendar 2023 | 680 (third party citing Ember) |
| South Africa | 661 | to Aug 2026 | DFFE 994 (2024, direct) |

The disagreements are large and have causes. South Africa: a generic coal factor of 820 against real plant near 1,000. United Kingdom: bioenergy at 13 % of supply charged at 230 g/kWh. Germany: lowcarbonpower is below UBA's life-cycle figure by 104 g. These rows are placeholders for a map, not values to cite.

---

## 2. Generation shares

Complete shares by source were read for the United States and all 27 eGRID subregions (EPA, 2023; in the JSON) and for all US states (EPA table 4; **not transcribed** into the JSON). Elsewhere the published pages give partial shares.

| Place, year | Coal | Gas | Nuclear | Hydro | Wind | Solar | Other stated | Source |
|---|---|---|---|---|---|---|---|---|
| World, 2025 | 33.0 | 21.8 | 8.9 | 14.0 | 8.5 | 8.7 | — | Ember 2026 |
| China, 2025 | 54 | not stated | 4.6 | 13 | about 11 | 11.1 | — | Ember 2026 |
| United States, 2025 | 16 | 40 | not stated | not stated | wind+solar 19 | 8.6 | — | Ember 2026 |
| United States, 2023 | 16.1 | 43.2 | 18.5 | 5.7 | 10.0 | 3.9 | oil 0.5, biomass 1.1, geothermal 0.4 | EPA eGRID2023 |
| United States, 2018 | 27.5 | 35.1 | 19.4 | 6.9 | 6.5 | 1.5 | biomass 1.6 | EPA eGRID2018 |
| EU-27, 2025 | 9.2 | 16.7 | 23 | 11.7 | 17.1 | 13.1 | — | Ember 2026 |
| India, 2025 | 71 | not stated | not stated | 8.6 | wind+solar 14 | 9.4 | — | Ember 2026 |
| Russia, 2025 | not stated (19 in 2024) | 45 | 18 | 17 | under 1 with solar | | — | Ember 2026 |
| Japan, 2025 | 32 | 33 | 9.1 | not stated | 1.3 | 9.8 | — | Ember 2026 |
| Brazil, 2025 | not stated | not stated | not stated | 52 | 15.7 | 11.8 | bioenergy 7.3 | Ember 2026 |
| Germany, 2025 | 21 | not stated | 0 `[my reading]` | not stated | 27 | 18 | fossil 41 | Ember country page |
| Poland, 2025 | 51 | not stated | — | not stated | 14 | wind+solar 25 | fossil 69 | Ember country page |
| South Africa, 2025 | not stated | | | | wind+solar 13 | | fossil 83 | Ember country page |
| France, 2025 | | fossil under 4 | 68.1 `[search summary]` | wind+solar+hydro about 25 | | | low-carbon 95.2 `[search summary]` | RTE |
| Canada, 2023 | 3.9 | 12.8 | 15.0 | 59.2 | wind+solar+tidal 7.8 | | — | ECCC, shares `[my arithmetic]` from GWh |
| United Kingdom, 2025 | — | 28 | 11 | | | | renewables 47, net imports 10 (shares of supply) | Carbon Brief |

Approximate shares for the 17 lowcarbonpower countries are in the JSON; they are the page's rounded words and are shares of consumption, not generation.

---

## 3. Subregional figures

### 3.1 United States — EPA eGRID2023 (trust A)

Source: eGRID Summary Tables 2023, Rev 2, https://www.epa.gov/system/files/documents/2025-06/summary_tables_rev2.pdf , table 1 read as a page image; total-rate values to three decimals from https://www.epa.gov/egrid/summary-data . Public domain per EPA's FAQ. Kind: CO2 only, direct, per net MWh generated in the subregion. EPA: subregions are drawn to "minimize the effect of electricity transfer between subregions"; rates are emissions divided by generation; non-baseload rates "can be used to estimate the emissions that could be avoided" and are "sometimes called 'marginal emission rates'". Converted at 1 lb = 453.59237 g.

| Subregion | Total, lb/MWh | Total, g/kWh | Non-baseload, lb/MWh | Non-baseload, g/kWh | Total in eGRID2018, lb/MWh |
|---|---|---|---|---|---|
| AKGD | 899.6 | 408.1 | 1,077.6 | 488.8 | 1,039.6 |
| AKMS | 520.5 | 236.1 | 1,548.6 | 702.4 | 525.1 |
| AZNM | 703.7 | 319.2 | 1,175.1 | 533.0 | 1,022.4 |
| CAMX | 428.5 | 194.3 | 958.3 | 434.7 | 496.5 |
| ERCT | 733.9 | 332.9 | 1,242.7 | 563.7 | 931.7 |
| FRCC | 782.3 | 354.8 | 981.7 | 445.3 | 931.8 |
| HIMS | 1,123.4 | 509.6 | 1,596.4 | 724.1 | 1,110.7 |
| HIOA | 1,489.5 | 675.6 | 1,753.4 | 795.3 | 1,669.9 |
| MROE | 1,397.3 | 633.8 | 1,701.2 | 771.7 | 1,678.0 |
| MROW | 920.1 | 417.4 | 1,776.7 | 805.9 | 1,239.8 |
| NEWE | 539.3 | 244.6 | 885.1 | 401.5 | 522.3 |
| NWPP | 631.7 | 286.6 | 1,613.4 | 731.8 | 639.0 |
| NYCW | 864.5 | 392.1 | 976.9 | 443.1 | 596.4 |
| NYLI | 1,180.7 | 535.5 | 1,316.4 | 597.1 | 1,184.2 |
| NYUP | 242.1 | 109.8 | 909.3 | 412.5 | 253.1 |
| PRMS | 1,543.1 | 699.9 | 1,636.7 | 742.4 | not in the 2018 table |
| RFCE | 596.9 | 270.8 | 1,175.6 | 533.2 | 716.0 |
| RFCM | 970.6 | 440.3 | 1,508.1 | 684.1 | 1,312.6 |
| RFCW | 911.4 | 413.4 | 1,757.4 | 797.1 | 1,166.1 |
| RMPA | 1,036.6 | 470.2 | 1,620.7 | 735.1 | 1,273.6 |
| SPNO | 862.0 | 391.0 | 1,892.1 | 858.2 | 1,163.2 |
| SPSO | 872.0 | 395.6 | 1,476.3 | 669.6 | 1,166.6 |
| SRMV | 739.7 | 335.5 | 1,145.5 | 519.6 | 854.6 |
| SRMW | 1,239.8 | 562.4 | 1,818.6 | 824.9 | 1,664.2 |
| SRSO | 842.3 | 382.1 | 1,386.0 | 628.7 | 1,027.9 |
| SRTV | 898.1 | 407.4 | 1,665.8 | 755.6 | 1,031.5 |
| SRVC | 593.4 | 269.2 | 1,286.8 | 583.7 | 743.3 |
| **U.S.** | **767.2** | **348.0** | **1,372.5** | **622.6** | **947.2** |

Grid gross loss is 4.1 to 4.4 % by subregion (4.2 % US). The rates above are per kWh generated; a kWh at the socket costs about 4 % more `[my arithmetic]`.

State rates (EPA table 3, 52 rows) are in the JSON. Lowest: VT 43.0, WA 265.1, NH 272.9 lb/MWh. Highest: WV 1,954.4, WY 1,820.3, KY 1,735.0. EPA recommends subregion rates for electricity use. A state rate describes the plants inside a border.

Ember's US figure for 2023 is 393 gCO2e/kWh (life-cycle); eGRID's is 348.0 gCO2/kWh (direct CO2). Different kinds; both kept.

### 3.2 Canada — provinces, 2023 (trust A; preliminary; rounded by ECCC)

ECCC, National Inventory Report 1990-2023, Part 3, Annex 13, tables A13-1 to A13-14 (PDF hosted by UNFCCC, read). Crown copyright; no commercial redistribution without permission. Generation intensity = utility-sector combustion emissions over generation, CO2e, direct; reservoir emissions excluded. Consumption intensity = same plus SF6, over generation less losses. **It is not import-adjusted**; ECCC notes Prince Edward Island's own figure is not a guide to what it consumes.

| Province | Generation, 2023 | Consumption, 2023 | Generation, 2015 | Generation, 2018 |
|---|---|---|---|---|
| Quebec | 1.5 | 1.9 | 1.1 | 1.3 |
| Manitoba | 2.2 | 2.5 | 3.5 | 1.3 |
| Prince Edward Island | 7.0 | see New Brunswick | 23 | 4.0 |
| Newfoundland and Labrador | 16 | 17 | 33 | 26 |
| British Columbia | 17 | 18 | 14 | 15 |
| Ontario | 54 | 59 | 41 | 27 |
| Yukon | 70 | 70 | 41 | 69 |
| New Brunswick | 230 | 230 | 320 | 330 |
| Northwest Territories | 390 | 420 | 360 | 200 |
| Alberta | 420 | 440 | 770 | 630 |
| Nova Scotia | 540 | 580 | 670 | 680 |
| Saskatchewan | 590 | 630 | 700 | 690 |
| Nunavut | 780 | 800 | 720 | 840 |
| **Canada** | **100** | **110** | **140** | **120** |

Units gCO2e/kWh. A 2026 edition of the report (data to 2024) should exist; it was not found.

### 3.3 Australia — state grids, scope 2 location-based factors (trust A)

Clean Energy Regulator, NGER amendments page, read. kg CO2-e/kWh. These are reporting factors set for a financial year from earlier data. Licence not read (the copyright page returned 404).

| Grid | 2019-20 | 2022-23 | 2024-25 | 2025-26 |
|---|---|---|---|---|
| NSW and ACT | 0.81 | 0.73 | 0.66 | 0.64 |
| Victoria | 1.02 | 0.85 | 0.77 | 0.78 |
| Queensland | 0.81 | 0.73 | 0.71 | 0.67 |
| South Australia | 0.44 | 0.25 | 0.23 | 0.22 |
| WA (South West Interconnected System) | 0.69 | 0.51 | 0.51 | 0.50 |
| Tasmania | 0.15 | 0.17 | 0.15 | 0.20 |
| Northern Territory | 0.63 | 0.54 | 0.56 | 0.56 |

The market-based residual mix factor is **0.81** for every state (2023-24). That is a different kind: what is left after renewable certificates are removed. A national factor was not found.

### 3.4 China — regional grids and provinces, 2022 (trust B: official, read at source, older)

MEE and National Bureau of Statistics, Announcement 2024 No. 33 (page read). kgCO2/kWh, CO2 only. The page gives no definitions beyond table headings.

Regional grids: North 0.6776, Northwest 0.5857, East 0.5617, Northeast 0.5564, Central 0.5395, South 0.3869, Southwest 0.2268. National 0.5366.

Provinces (30; Tibet absent) are in the JSON. Lowest: Yunnan 0.1073, Sichuan 0.1404, Qinghai 0.1567. Highest: Hebei 0.7252, Shanxi 0.7096, Tianjin 0.7041. Range 6.8 to 1.

These are average factors. They are not the CDM "baseline" operating-margin and build-margin factors for regional grids, which MEE has also published; those were not read.

### 3.5 India

One synchronous grid since December 2013; CEA publishes one set of factors and no regional ones (CEA User Guide v19.0, read). FY 2022-23, tCO2/MWh: average including renewables **0.716**; average excluding renewables **0.823**; operating margin **0.971**; build margin **0.867**; combined margin **0.919**. Versions 20, 21 and 22 exist. Version 20's PDF returned 404; FY 2023-24 = 0.727 is `[search summary]`.

### 3.6 Not published as a readable table, as far as I found

UK regions (NESO publishes regional intensity through a data service, not a table that I found). German states. Other EU sub-national. Japanese utility areas (one utility figure, TEPCO 0.421 kg-CO2/kWh, `[search summary]`). See "Not found".

---

## 4. How fast it moves

All rates are compound, per year, `[my arithmetic]` from the two endpoints shown. Endpoints are in one kind and one source each.

| Place | Kind, source | Earlier | Later | Rate |
|---|---|---|---|---|
| World | life-cycle, generation; Ember | 533 (2015) | 458 (2025) | −1.5 % a year |
| World | same | 473 (2024) | 458 (2025) | −2.7 % in the year (Ember's figure) |
| United States | direct CO2, generation; eGRID | 947.2 lb/MWh (2018) | 767.2 (2023) | −4.1 % a year |
| United States | same | 1,122.9 (2014) `[search summary]` | 767.2 (2023) | −4.1 % a year |
| United States | **non-baseload**; eGRID | 1,432.3 (2018) | 1,372.5 (2023) | −0.8 % a year |
| United States | life-cycle; Ember | 598 (2007) | 384 (2024) | −2.6 % a year |
| Germany | direct CO2 per kWh consumed; UBA | 529 (2015) | 344 (2025, estimated) | −4.2 % a year |
| Germany | same | 365 (2020) | 344 (2025) | −1.2 % a year; 433 in 2022 |
| Germany | life-cycle CO2e; UBA | 603 (2015) | 406 (2025) | −3.9 % a year |
| EU-27 | direct, generation; EEA | — | 2025 is "about 41 % lower" than a decade earlier | −5.1 % a year; absolute values not on the page |
| Canada | direct, generation; ECCC | 140 (2015) | 100 (2023) | −4.1 % a year (two-digit rounding) |
| Alberta | same | 770 (2015) | 420 (2023) | −7.3 % a year |
| Ontario | same | 27 (2018) | 54 (2023) | **+14.9 % a year** |
| New South Wales | scope 2 factor; CER | 0.81 (2019-20) | 0.64 (2025-26) | −3.9 % a year |
| South Australia | same | 0.44 | 0.22 | −10.9 % a year |
| Tasmania | same | 0.15 | 0.20 | up |
| China | life-cycle; Ember | 560 (2024) | 525 (2025) | −5 % in the year (Ember's figure) |
| India | life-cycle; Ember | 708 or 705 (2024) | 670 (2025) | about −5 % in the year |
| South Africa | direct, generation; DFFE | 942 (2023) | 994 (2024) | **+5.5 %** in the year |
| CAMX (California) | direct, generation; eGRID | 496.5 (2018) | 428.5 (2023) | −2.9 % a year |
| NYCW (New York City) | same | 596.4 (2018) | 864.5 (2023) | **+7.7 % a year** |

Three things follow. The average falls about five times faster than the non-baseload rate in the US, so the two maps drift apart. A ten-year rate hides reversals (Germany 2021 to 2022, South Africa 2024, Ontario, New York City). A table stamped with one year is wrong by 2 to 5 % for each year it ages, in a direction that is usually but not always down.

IEA's forecast for 2025 to 2030 is −3.7 % a year for the world (435 to 360). That is a forecast.

---

## 5. The range

| Scope | Kind | Cleanest | Dirtiest | Ratio | Trust |
|---|---|---|---|---|---|
| Large economies, 2025 | life-cycle, generation; Ember | Brazil 110 | India 670 | 6.1 | B |
| Countries, official national sources | direct, generation | France 19.6 (2025) | South Africa 994 (2024) | 51 | A, years differ by one |
| Countries, one third-party method | consumption mix, generic life-cycle | Norway 32 | South Africa 661 | 21 | C |
| US eGRID subregions, 2023 | direct CO2, generation, total output | NYUP 109.8 | PRMS 699.9 (MROE 633.8 without islands) | 6.4 (5.8) | A |
| US eGRID subregions, 2023 | **non-baseload** | NEWE 401.5 | SPNO 858.2 | 2.1 | A |
| US states, 2023 | direct CO2, by plant location | VT 19.5 | WV 886.5 | 45 | A, but a state is not a grid |
| Canadian provinces, 2023 | direct, generation | Quebec 1.5 | Nunavut 780 (Saskatchewan 590 among provinces) | 520 (390) | A |
| Chinese provinces, 2022 | direct CO2, average | Yunnan 107.3 | Hebei 725.2 | 6.8 | B |
| Australian state grids, 2025-26 | scope 2 factor | Tasmania 200 | Victoria 780 | 3.9 | A |

Ratios `[my arithmetic]`. The low end is fragile to kind. Quebec at 1.5 is direct emissions with reservoir emissions excluded; any life-cycle figure for hydro (6 by RTE, 24 by IPCC as used by Ember) raises it four to sixteen times and cuts the Canadian ratio accordingly. A map's dynamic range is therefore about **50 to 1 direct** and about **20 to 1 life-cycle** at country level `[my reading of the two rows above]`, and about **2 to 1** if drawn with marginal rates where those exist.

---

## Which kind for which question

**(b) A short-run marginal change** (one load switched on or off this year, the fleet fixed). The right kind is a short-run marginal rate. The published proxies are eGRID's non-baseload rate (EPA says it is for "emissions that could be avoided") and the CDM operating margin (India 0.971 against an average of 0.716). Both exclude must-run and renewable plant by rule, not by observing dispatch; they are approximations. Outside the US, India and the CDM tables, no marginal figure was read.

**(a) A long-run "this load never existed, or moved"** thought experiment. The literature offers two answers and they disagree.

- The **average** (location-based) is the attributional answer: it allocates the grid's actual emissions to all load in proportion. It is what inventories, GHG Protocol scope 2 and every national figure above are. It adds up: the sum over all loads is the grid's emissions. It does not say what would change.
- The **long-run marginal** rate is the consequential answer: what the grid emits differently once it has also built differently in response. NREL's authors (Gagnon and Cole) define it as "the emissions per unit change in electricity consumption, where the influence of the change in demand on both the operation and structure of the grid is taken into account" and report that it "outperformed both the SRMER and AER at estimating emission impacts" `[search summary; paper not opened; it is the paper's own conclusion]`. The CDM build margin is an older proxy (India 0.867 excluding renewables; 0.552 once renewables are counted, `[search summary]`). Long-run marginal rates are published for the US only, as a dataset (NREL Cambium), not read.

Why they disagree. A short-run margin is mostly gas and coal, so it is high and nearly flat across places. A long-run margin in a grid that is building wind and solar is lower than the short-run margin and can be lower than the average. The average sits between in some grids and outside in others. Which is right depends on whether the question is "who is responsible for what was emitted" or "what changes if this load changes", and on whether new clean plant would have been built anyway `[my reading]`.

For this program's use: the estate being moved is about 1.5 % of world electricity (SCOUT_ESTATE), which is not a marginal change in the textbook sense but is small against a grid. The **average, location-based, for a stated year** is the kind that can be sourced for every place on a map today, and it is the same kind SCOUT_ESTATE chose for the baseline. It should be shown beside a marginal reading where one exists (US subregions, India), because finding 5 says the two maps differ. A caller should not present the average-based result as "emissions avoided".

**Market-based** figures (residual mix, certificates, power purchase agreements) are a third thing. Australia's residual mix is 0.81 everywhere while its location factors run 0.20 to 0.78. China's factor excluding traded non-fossil power is 585.6 against 536.6. Data-centre operators commonly report market-based `[my reading]`; a map by place needs location-based.

**Direct or life-cycle.** Pick one and hold it. Direct is what national agencies publish and is 9 to 54 g/kWh lower for the same grid in the two cases read (Germany 352 against 406; France 19.6 against 29.0). Life-cycle is what Ember publishes for every country. Mixing them puts a false step of that size between neighbours.

**Generation or consumption.** For a large interconnected country the difference is small (Germany 14 g, South Africa 34 g, Canada's losses 10 g). For a small importing region it is the whole answer (Prince Edward Island generates at 7 and consumes New Brunswick's 230).

---

## What this does not license

- It does not license a single number per country. Every country row is one of several kinds; the JSON says which.
- It does not license comparing an Ember row with a national-agency row as if the gap were real. Ember is life-cycle on generation; agencies are mostly direct, and some are per kWh consumed.
- It does not license reading the trust-C rows (lowcarbonpower, search summaries) as measurements. They disagree with official figures by up to 330 g/kWh (South Africa) and by a factor of two (France, UK).
- It does not license "emissions avoided". An average intensity times kWh is an allocation, not a consequence. Where a marginal rate exists it is 1.06 to 3.8 times the average in the US subregions `[my arithmetic]`.
- It does not license a US state map as a consumption map. EPA advises subregions.
- It does not license using the 2022 Chinese or FY 2022-23 Indian factors as current. Both grids moved about 5 % in 2025 alone by Ember's count.
- It does not license a rate of change taken across Ember editions before and after its method change (2024 edition against later ones).
- It does not license treating hydro-heavy places as near zero on a life-cycle basis, nor bioenergy-heavy places as low: Ember itself calls its bioenergy factor a likely significant underestimate.
- It does not say where a relocated load would actually draw power. Devices at the edge draw at the socket, after 4 % or more of losses, at the hours people are home. Hourly variation was not scouted; RTE reports France never exceeded 58 g/kWh in any hour of 2025 `[search summary]`, and other grids swing far more `[my reading]`.
- No figure here was estimated by me. Conversions (lb/MWh to g/kWh, kg to g, tonnes per MWh to g/kWh) and rates are marked.

---

## Datasets that would improve this, each needing the lead's approval by name

None of these was downloaded.

| # | Dataset | Publisher | URL | Licence as read | What it would add |
|---|---|---|---|---|---|
| 1 | Yearly Electricity Data (CSV, global) | Ember | https://files.ember-energy.org/public-downloads/generation/outputs/release_generation_yearly_global.csv (linked from https://ember-energy.org/data/yearly-electricity-data/ ) | "All content is released under a Creative Commons Attribution Licence (CC-BY-4.0)" | One kind (life-cycle, generation) for 200+ geographies, 2000 to 2025, with full shares by source. Replaces every trust-C country row and fills items 1, 2 and 4 of the brief at one stroke. The single most useful approval. |
| 2 | Carbon intensity of electricity (grapher CSV) | Our World in Data, from Ember and the Energy Institute | https://ourworldindata.org/grapher/carbon-intensity-electricity | OWID's own work CC BY; third-party data under the providers' terms | The same series as 1, already merged to long history. Redundant if 1 is approved. |
| 3 | eGRID2023 workbook and summary-tables workbook | US EPA | https://www.epa.gov/egrid ; https://www.epa.gov/system/files/documents/2025-06/summary_tables_rev2.xlsx | public domain per EPA FAQ | Machine-read copy of what is here by eye; balancing-authority and plant level; a check on the hand-transcribed state table. |
| 4 | Long-run marginal emission rates (Cambium workbooks) | NREL | https://catalog.data.gov/dataset/long-run-marginal-emission-rates-for-electricity-workbooks-for-2023-cambium-data-8cfdc | not read | The only published long-run marginal rates found; US regions; would let question (a) be answered both ways. |
| 5 | Indicator data, GHG emission intensity of electricity generation, by country | EEA | https://www.eea.europa.eu/en/analysis/indicators/greenhouse-gas-emission-intensity-of-1 (chart data download) | CC BY (EEA legal notice) | EU-27 and each member state, direct, 1990 to 2025: the official European series, which the page shows only as a chart. |
| 6 | Emissions Factors database, 2026 edition | IEA | https://www.iea.org/data-and-statistics/data-product/emissions-factors-2026 `[URL not confirmed]`; method PDF at https://www.iea.org/files/ed5a518b-d36b-4b87-91e8-5a64a7c1e476/IEA_Methodology_Emission_Factors_2026.pdf | excluded from CC BY; IEA terms for non-CC material; paid | Direct CO2 per kWh for all countries 1990 to 2024, with trade-adjusted and loss-adjusted variants. The method PDF is a document and could be read without approval; it was not. |
| 7 | Annex 13 electricity intensity tables (spreadsheet) | Environment and Climate Change Canada | via https://www.canada.ca/ghg-inventory | not read | Unrounded provincial series, every year, and the 2026 edition. |
| 8 | National Greenhouse Accounts Factors workbook, 2025 and 2026 | Australia DCCEEW | via https://www.dcceew.gov.au `[page not opened]` | not read | The national factor, scope 3 loss factors, and the method note for the state factors. |
| 9 | Conversion factors 2026 (full set workbook) | UK DESNZ | https://www.gov.uk/government/publications/greenhouse-gas-reporting-conversion-factors-2026 | Open Government Licence `[my reading; not read]` | The UK factor at source (replaces two trust-C rows) with its T&D-loss and well-to-tank companions. |
| 10 | CO2 Baseline Database, version 22 (workbook and guide) | India CEA | https://cea.nic.in/wp-content/uploads/baseline/2026/09/User_Guide__Version_22.0.pdf (returned 404 when tried) | not found | FY 2025-26 average, OM, BM, CM; the guide alone is a document and needs only a working link. |
| 11 | Regional carbon intensity data | UK NESO | https://www.neso.energy `[page not opened]` | not read | GB regional intensities; the only UK sub-national source known to me `[my reading]`. |
| 12 | Yearly and monthly country files (CSV/JSON) | lowcarbonpower.org | https://lowcarbonpower.org/methodology | no licence stated | Nothing that dataset 1 does not give better. Listed for completeness; not recommended. |
| 13 | Harmonized grid emission factor data set (operating and build margins by country) | UNFCCC / international financial institutions' working group | `[URL not found this session]` | not read | A marginal-kind figure for most countries `[my reading; existence recalled, not confirmed]`. |

---

## Not found

- **EU-27 and member states, official (EEA), absolute values for any recent year.** The indicator page gives only relative changes in text. 1990 = 510, 2018 = 281, 2019 = 249 are `[search summary]` of an older edition.
- **Ember intensity at source** for Germany, France, United Kingdom, Italy, Spain, Poland, Netherlands, Sweden, Norway, Canada, Australia, South Africa, Indonesia, Nigeria, Kenya, Mexico, South Korea. Ember's country pages give shares and per-person emissions, not intensity.
- **Official national figures** for Italy, Spain, Poland, Netherlands, Sweden, Norway, Indonesia, Nigeria, Kenya, Mexico, South Korea, Russia, Brazil. Not searched one by one for all; none surfaced.
- **Japan at source.** METI's page returned 403. Two search summaries give 0.423 (receiving end) and about 0.45 kg-CO2/kWh.
- **The IEA country table republished by the French statistics service** (Chiffres clés du climat, electricity page). The page redirected in a loop. Three values from it are `[search summary]`.
- **UK official factor value at source.** Only the −26 % change was read in DESNZ's document.
- **Australia, national factor.** Only states.
- **China 2023 or later, regional or provincial.** Only 2022 was read. China's CDM-style regional baseline (OM/BM) factors: not read.
- **India: regional grids.** None exist since 2013. FY 2023-24 and later at source: not reached.
- **US ten years earlier at source.** eGRID2014 (1,122.9 lb/MWh) is `[search summary]`; eGRID2018 was read.
- **Sub-national tables for Germany, other EU states, UK, Japan, Brazil, Russia, Mexico, South Africa.**
- **Marginal figures** for any place other than US subregions and India. **Long-run marginal** figures for any place, as a readable table.
- **Hourly or seasonal variation.** Not scouted.
- **Licences not read:** UBA (none on the pages read), Australian regulator (copyright page 404), MEE (none on the page), CEA, DFFE gazette, Carbon Brief, lowcarbonpower (none stated), DESNZ, ECCC's spreadsheet.
