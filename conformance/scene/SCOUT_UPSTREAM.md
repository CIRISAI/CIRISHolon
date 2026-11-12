# SCOUT: upstream energy avoided, by named component

Scouted 2026-11-11. Documentation only: operator reports, engineering papers, regulator and national-lab reports, academic papers, vendor sustainability sheets. No dataset was downloaded, no value-returning data API was called, nothing was signed up for. PDFs of reports and papers were opened and read as documents. Companion to `conformance/scene/SCOUT_DEVICES_POWER.md` and `conformance/scene/tables/power0/power_table.json`; nothing in those is repeated here except where a number is needed for arithmetic.

Marking convention (same as the earlier report). Unmarked statements were read in the source document itself at the URL beside them (publisher's page, or the author's/operator's own PDF). `[mirror]` = read in full, but from a copy not hosted by the publisher. `[search summary]` = seen only in a search engine's summary. `[my reading]` = my inference or recollection. `[my arithmetic]` = computed by me from figures read. `[not confirmed]` = could not open the source.

Two cautions, carried beside every figure:

- **Kind of figure.** AVG = average allocation (a total divided by users, bytes or impressions). MARG = marginal cost (what changes when one more unit is served or removed). CAP = a capacity or provisioning figure. Only MARG is a short-run saving; AVG becomes a saving only in the long run and only if the operator actually builds less.
- **A constant is a price in a regime.** Each figure carries year, system and how fast it is moving.

---

## Headline findings

1. **The only operator-published per-user figure is Meta's own: 0.0055 MWh of electricity per daily active person in 2024 (5.5 kWh a year, 0.63 W continuous).** It was 3.3 kWh in 2021. It is an average allocation of everything Meta runs (serving, storage, ranking, ads, AI research, generative AI, offices) and it rose 53 % in one year. It is the upper bound on "everything upstream per user" for Meta and it is not a saving.
2. **Nobody publishes the split by workload.** No operator states what share of its electricity is serving versus analytics/ML versus storage. What exists: Google's statement that ML was 10 to 15 % of its total energy in 2019-2021; Meta's ratios INSIDE its AI fleet (10:20:70 experimentation : training : inference by power capacity; 31:29:40 data : training : inference for one recommendation model); Azure's statement that storage is 33 % of operational emissions in its general-purpose cloud; LBNL's model that storage is 9 % of US data-centre electricity. Each is a different system.
3. **The lead's correction is supported for HARVESTING and weakly for STORING.** By my arithmetic, bytes at rest on disk are cheap in energy: 1 to 10 GB per user, replicated and cooled, is 0.015 to 0.15 kWh a year, under 3 % of Meta's 5.5 kWh per daily person. The energy is in computing on the data (ranking, ad selection, training, the pipelines that feed them, and the flash and hot tiers that serve them), not in keeping it. Top-down storage shares (9 to 33 %) are much larger than the bottom-up disk floor because they include flash, storage-server CPUs and derived data (the warehouse and training sets, which Meta says are as large as the user content itself).
4. **Short run, removing one household saves almost nothing upstream.** Azure's measured storage clusters: a 28 % drop in IO moved power by 0.04 %. Training capacity is provisioned per product, not per user. The avoided term is a long-run, capacity-not-built term and should be labelled so.
5. **Advertising-chain figures exist but are average allocations of carbon, not marginal energy**, and the two best-known ones (Pärssinen 2018; the 2023 tracking paper) rest on kWh/GB constants that the earlier report already showed to be the wrong shape.
6. **Substitution: the evidence says adoption is additive.** In the one study of a real migration (WhatsApp, 2021), 26 % of users wanted to move, a quarter of those managed it and 0.5 % uninstalled. Forced removal experiments do show time falling rather than moving elsewhere, but a mesh messenger is an adoption, not a removal.

---

## 1. RETENTION

### 1.1 Replication and erasure-coding overhead (read at source)

| Figure | Value | System, year | Kind | Source |
|---|---|---|---|---|
| Hot blob store, effective replication factor | 3.6 (3 copies x 1.2 RAID-6) | Facebook Haystack, 2014 | CAP | Muralidhar et al., "f4: Facebook's Warm BLOB Storage System", OSDI 2014 `[mirror]` https://www.cs.princeton.edu/~wlloyd/papers/f4-osdi14.pdf |
| Same, as actually provisioned when IOPS-bound | 5.3 | Haystack, before 2021 | CAP | Pan et al., "Facebook's Tectonic Filesystem", FAST 2021 `[mirror]` https://www.cs.princeton.edu/~wlloyd/papers/tectonic-fast21.pdf |
| Warm blob store | 2.8 (RS(10,4) = 1.4x, in two datacentres), then 2.1 (with cross-datacentre XOR) | Facebook f4, 2014 | CAP | f4 paper |
| Consolidated blob storage | about 2.8 | Tectonic, 2021 | CAP | Tectonic paper |
| Warm corpus size | over 65 PB logical; over 400 billion photos stored | Facebook, 2014 | CAP | f4 paper |
| Deleted but not yet reclaimed | 6.8 % of f4 bytes | 2014 | CAP | f4 paper |
| Photos move hot to warm after | three months (one month for other types) | 2014 | policy | f4 paper |
| Blob storage AND data warehouse | "both of which store exabytes of data" | Facebook, 2021 | CAP | Tectonic paper |
| Data warehouse | "upwards of 300 PB of Hive data, with an incoming daily rate of about 600 TB"; 3x growth in a year | Facebook, April 2014 | CAP | https://engineering.fb.com/2014/04/10/core-infra/scaling-the-facebook-data-warehouse-to-300-pb/ |

What these are NOT: none is bytes per user, none is current, and none says how long a user's data is kept. Retention period for a large social platform's user content is "until deleted" `[my reading]`; no operator publishes a figure in years.

Rate: effective replication fell 3.6 to 2.1-2.8 over 2010-2021 (erasure coding), a one-off step; bytes grew 3x a year in 2014 and the AI training corpus 2.4x in two years to 2021 (Wu et al., below).

### 1.2 Bytes per user: NOBODY PUBLISHES IT. Bracket and derivation

- Floor from Meta's own statements `[my arithmetic]`: at least 2 EB logical (blob + warehouse, each "exabytes", 2021) over about 3.5 billion monthly people = at least 0.6 GB per person. In 2014: (65 PB warm blobs + 300 PB warehouse) over about 1.3 billion monthly users `[my reading for the user count]` = about 0.28 GB per person, of which the WAREHOUSE (derived, analytic data) was four fifths.
- **Bracket I would defend: 1 to 10 GB logical per user for a social platform in the mid-2020s, of which a half or more is derived data (warehouse, logs, features, training sets) rather than the user's own content.** `[author's estimate]` The upper end is not bounded by anything published.
- The important structural fact is in the sources: the analytic copy is as big as the content. 2014: 300 PB warehouse against 65 PB warm blobs. 2021: both "exabytes".
- **A messenger with end-to-end encryption stores almost nothing server-side.** Signal: "the Signal service temporarily queues that message for delivery. As soon as your message is delivered, that small bundle of encrypted data ... can be dropped from the queue" (https://signal.org/blog/signal-is-expensive/). WhatsApp deletes delivered messages from its servers and holds undelivered ones 30 days `[my reading; the WhatsApp help page did not render]`. So the RETENTION component applies to the platform's social, feed and metadata side, not to message bodies of an encrypted messenger. What a WhatsApp-class service keeps is metadata and logs, and no figure exists for that.

### 1.3 Storage energy per TB-year

| Figure | Value | Unit | System, year, basis | Kind | Source, how read |
|---|---|---|---|---|---|
| Disk enclosure with nearline drives, idle / operating | 0.61 / 0.69 | W per raw TB | Seagate Exos enclosure with Exos X24 24 TB drives; vendor sustainability sheet, 2026 | CAP | https://www.seagate.com/content/dam/seagate/assets/sustainability/files/new-2026/exos-4u74-system-with-hdds-lca-report.pdf , read |
| Same, annual | 5.09 | kWh per raw TB-year | same; 1,560 W idle, 1,750 W operating, 12,943 kWh a year per unit; operating life "Five years" | CAP | same |
| Drive alone, 24 TB nearline | 6.3 idle; 7.1 write; 8.9 read | W per drive (0.26 to 0.37 W/TB) | Seagate Exos X24 SATA, reseller listings | CAP | `[search summary]` |
| HDD, model input | 14 (2006), 8.6 (2015), 7 (2020), 6.4 (2025) | W per drive | LBNL stock model, US | modelled | LBNL 2024 report, read (below) |
| Flash, NVMe, model input | 11 | W per drive | LBNL, 2020-2025 | modelled | same |
| SSD storage rack vs HDD storage rack | about 4x | operational emissions per TB | Azure racks, "component energy draws measured under a representative load", 2024 | CAP | McAllister et al., "A Call for Research on Storage Emissions", HotCarbon 2024, author's copy https://saramcallister.github.io/files/2024-hotcarbon-mcallister.pdf |
| Share of a storage rack's operational emissions that is the drives | 48 % (HDD rack), 39 % (SSD rack) | % | Azure, 2024 | CAP | same |
| **Power response to load** | IO fell 28 %, power fell 0.04 % | % | Azure SSD cluster, 7 days | **MARG** | same |

Caution on the Seagate sheet: the 4U74 and 4U100 sheets carry the identical power block, and 1,560 W at 0.61 W/TB implies 2,544 TB, which is 106 drives of 24 TB, not 74 `[my arithmetic]`. The per-TB figure is the vendor's and is consistent with Azure's "drives are 48 % of an HDD rack" (0.26 W/TB drive / 0.48 = 0.55 W/TB) `[my arithmetic]`, so I use the per-TB figure and not the per-unit one.

**Energy per LOGICAL TB-year, disk, with cooling and replication** `[my arithmetic]`: 5.09 kWh raw x PUE x replication.
- hyperscale, warm: 5.09 x 1.08 x 2.1 = 11.5
- hyperscale, typical: 5.09 x 1.08 x 2.8 = **15.4 kWh per logical TB-year** (1.8 W per logical TB)
- enterprise worst case: 5.09 x 1.54 x 3.6 = 28.2
- flash: about four times the disk figure per TB (Azure ratio), so 45 to 60 kWh per logical TB-year at hyperscale `[my arithmetic]`

**Storage-only floor per user** `[my arithmetic]`: 1 to 10 GB x 15.4 kWh/TB-yr = **0.015 to 0.15 kWh per user per year** (0.002 to 0.018 W). All on flash it would be 0.05 to 0.6 kWh. Kind: AVG/CAP. Short-run saving if one user's bytes are deleted: none (the drive spins regardless).

Rate: W per TB falls about 15 to 20 % a year with drive density (LBNL assumes +20 % density a year); W per drive falls 1.6 % a year.

### 1.4 Storage as a share of data-centre energy (the top-down check)

- **LBNL, "2024 United States Data Center Energy Usage Report"** (Shehabi et al., December 2024; the copy read carries an embargo header) https://eta-publications.lbl.gov/sites/default/files/2024-12/us_data_center_energy_usage_report_lbnl-2001637.pdf . US, bottom-up shipment model: total 176 TWh in 2023 (4.4 % of US electricity; 58-60 TWh in 2014-2016); servers "nearly 100 TWh" (AI servers "more than 40 TWh", conventional "nearly 60 TWh"); **storage 16 TWh** (flash 25 %; HDD 12.3 TWh, its peak); network "flat around 3 %" to 2022; infrastructure 30 % of total. So storage is 9 % of total and about 13 % of IT electricity `[my arithmetic]`. Storage was 7 TWh in 2014. Kind: modelled AVG for all US data centres, not for social platforms. It counts drives only `[my reading: the model pairs drive counts with W per drive]`, so storage-server CPUs sit under "servers".
- **Azure**: "storage ... causes 33% of operational emissions and 61% of embodied emissions in Azure's general-purpose cloud"; "Storage racks alone account for 24% of operational and 45% of embodied" (McAllister et al. 2024, citing an Azure study). Kind: operator-reported AVG for a general-purpose cloud, emissions not kWh; includes storage servers.
- Applied to Meta's 5.5 kWh per daily person (a transfer between systems, so a bracket only): 9 to 33 % = **0.5 to 1.8 kWh per daily person per year for storage, top-down** `[my arithmetic]`, ten times the bottom-up disk floor. The gap is flash, hot tiers, caches, storage CPUs and derived data.

---

## 2. HARVESTING AND ANALYTICS

### 2.1 Operator totals and the per-user upper bound

**Meta, 2025 Environmental Data Index** (data to 2024; read) https://sustainability.atmeta.com/asset/2025-environmental-data-index

| | 2020 | 2021 | 2022 | 2023 | 2024 |
|---|---|---|---|---|---|
| Total electricity, MWh | 7,170,000 | 9,420,839 | 11,508,131 | 15,325,314 | 18,423,634 |
| Data centres total, MWh | 6,966,000 | 9,117,122 | 11,167,416 | 14,975,435 | 18,061,781 |
| of which leased, MWh | 795,000 | 964,650 | 1,105,834 | 2,187,020 | 3,069,504 |
| **Electricity per daily active person, MWh** | - | 0.0033 | 0.0039 | 0.0036 | **0.0055** |
| PUE | 1.10 | 1.09 | 1.08 | 1.08 | 1.08 |

- 5.5 kWh per daily person per year = 0.63 W continuous `[my arithmetic]`; implied 3.35 billion daily people `[my arithmetic]`. A household with two daily users: 11 kWh a year, 1.3 W.
- Kind: **AVG, operator-reported.** What it is NOT: not the cost of messaging; not a marginal saving; not US-specific (a US user with more time and more ad revenue plausibly carries more than the world average `[my reading]`); includes Meta's generative-AI build, which a messenger replacement does not touch.
- Rate: total x2.57 in four years; per person +53 % from 2023 to 2024. A figure for 2025 was not found (I guessed a 2026 index URL and it did not exist).
- `[my arithmetic]` (5.5 - 3.3) / 5.5 = 40 % of 2024's per-person electricity was added after 2021, a period in which the serving of messages and feeds did not change in kind. That increment is the AI build (recommendation and generative) `[my reading]`.

**Google, 2025 Environmental Report** (read) https://www.gstatic.com/gumdrop/sustainability/google-2025-environmental-report.pdf : purchased electricity 15,125,700 (2020), 18,238,400, 21,685,300, 25,252,600, **32,106,200 MWh (2024)**. Google publishes no per-user intensity and the total includes Cloud customers, so no per-user bound can be formed. 2025: data-centre electricity up 37 % `[search summary]`.

### 2.2 The only published workload ratios

- **Wu et al., "Sustainable AI: Environmental Implications, Challenges and Opportunities"** (Meta; HPCA 2022; arXiv 2111.00364, read):
  - "At Facebook, we observe a rough power capacity breakdown of 10:20:70 for AI infrastructures devoted to the three key phases - Experimentation, Training, and Inference". Kind: **CAP, within the AI fleet only.** It does NOT say what share of Meta's total the AI fleet is.
  - "the energy footprint of RM1 is roughly 31:29:40 over Data, Experimentation/Training, and Inference". One recommendation model, end to end. Kind: AVG for one model. **This is the nearest thing to a harvesting split: 31 % of a ranking model's energy is the data pipeline (storage and ingestion), 29 % training, 40 % inference.**
  - Training data for two recommendation use cases grew "2.4x and 1.9x in the last two years, reaching exabyte scale"; data-ingestion bandwidth demand up 3.2x; "trillions of inference per day"; total 7.17 million MWh in 2020.
- **Patterson et al., "The Carbon Footprint of Machine Learning Training Will Plateau, Then Shrink"** (Google; arXiv 2204.05149, read): three one-week samples (April 2019, 2020, 2021): "Each time the ML portion was 10% to 15% of Google's total energy consumption for that week"; "about 3/5 of ML energy use is for inference and 2/5 for training". Kind: operator-reported AVG share, Google-wide including Cloud, before generative AI. NOT current, NOT a social platform.
- **LBNL 2024**: AI (accelerated) servers were over 40 of about 100 TWh of US server electricity in 2023, from under 2 TWh in 2017. US-wide modelled.

### 2.3 Event logging volume (operator engineering posts)

- Scribe, Meta's log transport, 2019: "The total size of these logs is several petabytes every hour"; "an input rate that can exceed 2.5 terabytes per second and an output rate that can exceed 7 terabytes per second"; retention "typically a few days" before the warehouse (https://engineering.fb.com/2019/10/07/data-infrastructure/scribe/ , read).
- `[my arithmetic]` 2.5 TB/s = 216 PB a day; over about 2.2 billion daily people then `[my reading]` = about **100 MB of log input per daily person per day** at peak rate. NOT all user events: it is every machine's logs. Uncompressed, transient.
- 2025: ingest over 15 TB/s, deliver over 110 TB/s `[search summary of a VLDB 2025 paper]`. Six times in six years.
- Kind: CAP (throughput). No energy figure is attached by the operator.

### 2.4 What a serving-only messenger costs (a contrast, in dollars, not energy)

Signal, November 2023 (https://signal.org/blog/signal-is-expensive/ , read): infrastructure "Approximately $14 million dollars per year": storage $1.3 M, servers $2.9 M, registration (SMS) fees $6 M, bandwidth $2.8 M, other $0.7 M; "around 100,000 requests per second"; at peak "nearly 600 servers" for private contact discovery alone. Signal does not publish a user count; third-party estimates of 40 to 70 million monthly users `[not confirmed]` give $0.10 to $0.18 per user-year for servers, storage and bandwidth `[my arithmetic]`. What it is NOT: energy; and Signal rents cloud capacity. It shows the order of a messenger with no harvesting: a few hundred to a few thousand servers for tens of millions of users.

### 2.5 The honest row: energy of harvesting per user is not published

Widest defensible bracket for "storage + analytics + ranking + ads machinery, per daily user, per year, long-run average":
- **Upper bound: 5.5 kWh** (Meta 2024, everything).
- **Lower bound: about 0.5 kWh** = Google's 10 % ML share applied to Meta's 2021 figure of 3.3 kWh, plus the disk floor `[my arithmetic]`. This lower bound transfers a Google ratio to Meta and is weak.
- **Central reading, mine: 2 to 4 kWh per daily person per year in 2024**, i.e. most of the total, on the argument that (a) a serving-only encrypted messenger needs very little, (b) the warehouse and training corpora equal the content in size, (c) 40 % of the 2024 figure arrived with the AI build. `[author's estimate]` Kind: AVG. Short-run MARG: about zero.

---

## 3. THE ADVERTISING SYSTEM

| Figure | Value | Unit | What it measures; basis; year; system | Kind | Source, how read | What it is NOT |
|---|---|---|---|---|---|---|
| Online advertising, world | 106 (range 20 to 282) | TWh/year | energy of devices + networks + data centres allocated to ads by traffic share; modelled; 2016; of 1,059 TWh for the whole internet (791 to 1,334) | AVG | Pärssinen, Kotila, Cuevas, Phansalkar, Manner, Env. Impact Assessment Review 73 (2018), read at https://e-archivo.uc3m.es/bitstream/10016/33900/1/Environmental_EIAR_2018.pdf | Not measured; not marginal; allocation by bytes; 2016 constants; includes the user's own device and the access network, which a mesh does not remove |
| same, emissions | 60 (12 to 159) | Mt CO2e | same | AVG | same | same |
| Per internet user | about 31 (6 to 83) | kWh/user-year | 106 TWh over about 3.4 billion users `[my arithmetic; user count my reading]` | AVG | derived | whole chain, 2016 |
| Tracking share of web bytes | 21.1 | % of bytes | 1.67 of 8.14 GB per user per year of web browsing is tracking-related; browser telemetry of about 100k users plus a crawl of 2.7 M sites; 2021-2022 | measured share, AVG | Pachilakis, Dambra, Sanchez-Rola, Bilge (Norton Research), arXiv 2304.00927, read | Web pages in a browser, not apps; "tracking" includes analytics; 8 GB a year is far below a household's traffic, so it is page payload only |
| Tracking emissions, world | 10.79 (6.28 conservative) | Mt CO2e/year | the above x 1.8 to 3.1 kWh/GB x grid factors | AVG | same | **Uses kWh/GB two to three orders above the Carbon Trust 0.0065 already in the table.** With 0.0065 kWh/GB, 1.67 GB is 0.011 kWh per user-year of network energy `[my arithmetic]`. Do not use the Mt figure |
| Ads share of mobile traffic | 1 | % of bytes | one day of a European carrier with over 3 million subscribers; 2012 | measured | Vallina-Rodriguez et al., "Breaking for Commercials", IMC 2012, author's copy https://haddadi.github.io/papers/adcache.pdf , read | 2012, banner ads, before video ads |
| Ad refresh interval | 12 to 120 (AdMob); 15 min (Millennial); 20 min (InMobi) | seconds | ad-network SDK settings, 2012 | setting | same | banner SDKs of 2012 |
| Facebook feed ad load | about 13 (one item in eight); cross-user s.d. 4.6 points | % of feed items | field experiment with a browser extension; untreated users; 2020s | measured | Beknazar-Yuzbashev, Jiménez-Durán, Simonov, Stalinski, "Social Media Advertising Loads as Prices", Warwick WP 1602, Feb 2026, read at https://warwick.ac.uk/fac/soc/economics/research/workingpapers/2026/twerp_1602-stalinski.pdf | Desktop-heavy sample, Facebook feed only; consumer-site counts of 21 % (2020) are `[search summary]` |
| Programmatic advertising | 215,000 | t CO2e/month | five countries (US, UK, FR, DE, AU); Scope3 model; Q1 2023 | AVG, modelled, commercial | `[search summary]`; scope3.com page and report PDF did not open | Carbon, not energy; includes corporate and embodied allocations `[my reading]`; vendor that sells the measurement |
| Per thousand impressions | 514.8 average; 187 to 1,772 by publisher | g CO2e per 1,000 impressions | same | AVG | `[search summary]` of trade press | same. A different vendor quotes 4.24 g per impression `[search summary]`: an eightfold disagreement |
| Ad blocker, page-load time | -11 % (AdBlock+), -22.2 % (Privacy Badger), -28.5 % (uBlock Origin) | % of load time | desktop browser, popular sites, 2019-2020 | measured (time) | Pearce, Technologies 8(2):18, 2020, read at mdpi | TIME, converted to energy by the author with device watts; its kWh totals are a model. Device side only |
| Ad blocking, desktop energy | -3.4 | % | Simons and Pras 2010 as cited by Rasmussen et al. 2014 | measured | `[search summary]` | 2010 desktops |

**Impressions per user-hour by app class: not published by any operator** (Meta reports year-on-year growth of impressions, not levels `[my reading]`). From the 13 % ad load and an assumed 300 to 600 feed items an hour, 40 to 80 ad impressions per feed-hour `[author's estimate, weak: the items-per-hour figure is mine]`. A messenger with no feed shows none.

**GARM / Ad Net Zero Global Media Sustainability Framework**: v1.2 June 2025 and a v1.3 guide (2026) exist; the framework gives formulae and tiered default values, with an ads.txt-line proxy for programmatic ad selection `[search summary]`. Its default kWh per impression was not found. Kind: an accounting standard, AVG by construction.

Rate: programmatic supply paths were being cut (Scope3 reported changes of several per cent per half year `[search summary]`); nothing reliable.

Verdict for the component: **there is no measured marginal energy per impression.** The usable facts are shares (13 % of feed items, 21 % of web page bytes) and the statement that ad selection, auction and measurement compute sits inside the operator totals of section 2, where it cannot be separated.

---

## 4. THE DEVICE SIDE

All are measurements on specific handsets and networks; several are from the 3G era and are prices in that regime.

| Figure | Value | What it measures; system; year | Kind | Source, how read | What it is NOT |
|---|---|---|---|---|---|
| Share of a free app's energy in third-party ad modules | 65 to 75 % | eprof accounting; six apps incl. Angry Birds, on Android 2.x and Windows Mobile handsets, 3G; 2011-2012 | measured, per app session | Pathak, Hu, Zhang, EuroSys 2012, read `[mirror]` https://www.cs.columbia.edu/~nieh/teaching/e6998_s13/papers/eprof_eurosys2012.pdf | A few games and news apps on 3G, where the radio tail dominates. Not a share of phone energy. Not social or messaging apps |
| Task energy in those apps | 10 to 30 % | same | same | same | same |
| Ads' share of an app's communication energy / total energy | 65 % / 23 % | top 15 free Windows Phone apps; 2013 | measured | Mohan, Nath, Riva, EuroSys 2013 `[search summary]`; the Microsoft PDF did not open | 15 apps, one platform |
| Apps with ads vs the same apps without | +16 % energy, +48 % CPU time, +79 % network data | 21 Google Play apps, instrumented with and without ad calls; 2014-2015 | measured | Gui, Mcilroy, Nagappan, Halfond, ICSE 2015, read at https://swag.uwaterloo.ca/assets/other/papers/gui-icse-2015.pdf | Lab workloads; Android of 2014 |
| Ads on 3G tails, top 100 apps | 3.2 % of total energy (57 of 100 apps show ads) | later study | measured | `[search summary]`, source not identified | not confirmed |
| OS telemetry cadence, phone idle | about every 4.5 minutes (Android 255 s; iOS about the same) | Pixel 2 and iPhone 8, minimally configured, idle; 2020 | measured | Leith, "Mobile Handset Privacy", 2021, read at https://www.scss.tcd.ie/Doug.Leith/pubs/apple_google2.pdf | **OS-level, to Apple and Google. A mesh messenger does not remove it.** |
| OS telemetry volume, idle | about 1 MB per 12 h (Android) vs 52 kB (iOS) | same | measured | same | same |
| Apps contacting ad/tracking services | the dominant tracking organisations are present "in over 82%" of apps; 82 % of apps connect to at least one such domain `[search summary]` | Lumen on-device monitor, 14,599 apps, to 2017 | measured prevalence | Razaghpanah et al., NDSS 2018, read `[mirror]` https://people.cs.umass.edu/~phillipa/papers/ndss18.pdf | Prevalence, not request rate or energy |
| Ad activity timing | 40 % of ad activity periods under 10 s apart, over 80 % under 100 s | European carrier trace, 2012 | measured | Vallina-Rodriguez et al. 2012 | 2012 |
| Ad-only traffic energy, cut by caching | about 50 % at a 20 s refresh | Galaxy Nexus S, 3G and Wi-Fi, power meter | measured | same | one handset |

Requests per hour from common apps while idle: **no source gives a current per-app figure.** The published cadences are the OS's (4.5 min) and ad SDK refresh while an ad-carrying app is in the foreground (12 to 120 s).

Arithmetic with the constants already in `power_table.json` (2012 LTE handset: 12.76 J per isolated small exchange, 11.6 s tail) `[my arithmetic]`:
- An ad refresh every 30 to 60 s over LTE keeps the radio in its tail 19 to 39 % of foreground time: 0.2 to 0.4 W while the app is open. On Wi-Fi: negligible (0.04 J per exchange).
- OS telemetry every 4.5 min over LTE: 320 x 12.76 J = 1.1 Wh a day. It stays under either design.
- Device-side saving bracket: 16 % (Gui) of the energy of ad-carrying app sessions. With a phone at 4.5 kWh a year from the mains and a third of use in ad-carrying social apps `[my assumption]`, about **0.2 to 0.3 kWh per phone per year**, upper end. Kind: MARG (this one really is saved when the ads are not fetched and rendered), measured in 2015.

Rate: ad formats moved from banners to video (more bytes, more decode); radios moved from 3G to LTE/5G with similar tails; ad fetches are now batched with content in first-party feeds, which removes separate wake-ups `[my reading]`. The 2012 shares overstate today's.

---

## 5. SUBSTITUTION

| Finding | Value | Population, method, year | Source, how read | What it is NOT |
|---|---|---|---|---|
| Users who wanted to leave WhatsApp after the 2021 policy notice | 25.97 % | survey of 1,525 WhatsApp users in the UK, Spain, Mexico, South Africa; 2021 | Griggio, Nouwens, Klokmose, CHI 2022; figures read on the Aarhus University page https://cs.au.dk/news-events/news/show-news/artikel/are-we-really-free-to-choose-how-we-communicate-online ; paper itself not opened | Self-report; four countries |
| Of those, managed to switch as much as they wanted | "only a quarter" (about 6 % of all) | same | same | |
| Uninstalled WhatsApp | 0.5 % (8 people `[search summary]`) | same | same | |
| Increased use of other apps / used WhatsApp less, by May 2021 | 27 % / 16 % | same | `[search summary]` of the abstract | |
| Messaging apps per person | median 5 installed, 4 used regularly, 1 primary | same | `[search summary]` of author's slides | |
| Installs in January 2021 | Telegram 63.5 million, Signal 50.6 million (from 0.99 million a year before) | world, app-store estimates | `[search summary]`; Sensor Tower, a commercial analytics firm, estimates under its own terms | Installs, not use. Another firm (Apptopia) reported no decline in WhatsApp `[search summary]` |
| Facebook deactivated for four weeks | freed 60 minutes a day; the treatment group "spent less time on both non-Facebook social media and other online activities" | randomised, 2,844 US users, autumn 2018 | Allcott, Braghieri, Eichmeyer, Gentzkow, AER 2020 (NBER w25514, read) | Paid deactivation, not adoption of an alternative; self-reported time |
| Instagram blocked on the phone | +22.7 % time on other social apps, +10.4 % on communication apps (marginal); overall phone time -27 min/day | randomised, Android users from university lab pools, 2021 | Aridor, RAND J. Econ. 56(3), 2025; working-paper version read at tse-fr.eu | Students; one to two weeks |
| YouTube blocked | +15.1 % on social apps; overall phone time -44 min/day | same | same | |
| Ads hidden in the feed | time on platform rises "only 6%" | field experiment, Facebook | Beknazar-Yuzbashev et al. 2026 (abstract, read) | |
| UK: WhatsApp reach / daily reach / minutes | 90 % of online adults (87 % in 2024) / 74 % daily (64 %) / 17 min a day | Ipsos iris passive panel (commercial, UKOM-accredited), May 2025 | Ofcom, Online Nation 2025, read `[mirror]` (ofcom.org.uk refuses automated reads) https://www.ot.gr/wp-content/uploads/2026/01/online-nations-report-2025.pdf | Reach, not exclusivity |
| UK: Facebook Messenger | 58 % monthly, 23 % daily (30 % in 2024) | same | same | |
| UK: Facebook + Messenger | 93 % of online adults, 42 min a day | same | same | |
| UK: overlap inside Meta | 79 % of the Facebook/Messenger audience also use both Instagram and WhatsApp (76 % in 2024); 94 % of Instagram's; 1.7 million use Facebook/Messenger only | same | same | |
| UK: apps used per month | 41 per smartphone user | same | same | |

Not found: Signal or Telegram reach in the Ofcom text I searched; any Pew or Reuters Institute table of messaging-app overlap (Reuters reports weekly use for news only `[search summary]`); any time-use panel of people before and after adopting Signal; any study of the October 2021 outage's lasting effect.

Reading for the model: **multi-homing is the norm (median five messaging apps) and adoption of a private messenger is additive.** A replacement fraction prior of 0.005 (uninstall) to 0.16 (use the incumbent less) per adopter is what the one real migration supports; 1.0 holds only under forced removal. And because upstream energy is provisioned per account and per product, a user who keeps the account but uses it less saves less than proportionally. `[my reading]`

---

## (a) Table to transcribe

Columns: component | state or sub-component | value | unit | basis | year | source | how read | what it is not. "Kind" (AVG / MARG / CAP) is in the basis column.

| component | state or sub-component | value | unit | basis | year | source | how read | what it is not |
|---|---|---|---|---|---|---|---|---|
| upstream, everything | electricity per daily active person | 5.5 | kWh/person-year | AVG; operator-reported (0.0055 MWh) | 2024 | Meta 2025 Environmental Data Index | read at source | not messaging alone; not marginal; includes generative AI; series 3.3, 3.9, 3.6, 5.5 for 2021-2024 |
| upstream, everything | operator total electricity | 18,423,634 | MWh/year | operator-reported; data centres 18,061,781 | 2024 | Meta 2025 Environmental Data Index | read at source | x2.57 since 2020 |
| upstream, everything | operator total electricity | 32,106,200 | MWh/year | operator-reported, purchased electricity | 2024 | Google 2025 Environmental Report | read at source | includes Cloud; no per-user figure exists |
| data centre overhead | PUE, Meta fleet | 1.08 | ratio | operator-reported | 2024 | Meta 2025 Environmental Data Index | read at source | flat since 2022 |
| retention | effective replication, hot blobs | 3.6 (5.3 as provisioned) | ratio | CAP; engineering paper | 2014 (2021) | f4, OSDI 2014; Tectonic, FAST 2021 | mirror of the paper | one operator |
| retention | effective replication, warm / consolidated | 2.1 to 2.8 | ratio | CAP; RS(10,4) x two sites, or with XOR | 2014-2021 | same | mirror of the paper | one operator |
| retention | analytic copy vs content | warehouse 300 PB vs warm blobs 65 PB; both "exabytes" by 2021 | bytes | CAP; operator engineering | 2014; 2021 | Facebook engineering post 2014; f4; Tectonic | read at source / mirror | not per user |
| retention | bytes per user, social platform | 1 to 10 | GB logical per user | AUTHOR'S ESTIMATE; floor 0.6 GB from "exabytes" over 3.5 bn | mid-2020s | derived | scout's arithmetic | nobody publishes it |
| retention | messenger with end-to-end encryption | near zero (queue until delivered) | - | operator statement | 2023 | Signal, "Privacy is Priceless, but Signal is Expensive" | read at source | metadata and logs not covered |
| storage energy | disk enclosure, raw | 5.09 (0.61 W/TB idle, 0.69 operating) | kWh per raw TB-year | CAP; vendor sheet; five-year life | 2026 | Seagate Exos system sustainability report | read at source | per-unit figures do not reconcile with 74 drives; excludes PUE and replication |
| storage energy | disk, per logical TB with cooling and replication | 15.4 (11.5 to 28.2) | kWh per logical TB-year | AVG/CAP; 5.09 x PUE 1.08-1.54 x 2.1-3.6 | 2026 | derived | scout's arithmetic | not flash (about 4x) |
| storage energy | flash vs disk | about 4 | ratio per TB | CAP; operator racks under representative load | 2024 | McAllister et al., HotCarbon 2024 | read (author's copy) | emissions ratio; Azure |
| storage energy | drive, model input | HDD 6.4; NVMe 11 | W per drive | modelled | 2025 | LBNL 2024 US Data Center Energy Usage Report | read (embargo-marked copy) | US stock average |
| storage energy | response to load | -0.04 % power for -28 % IO | % | MARG; measured, SSD cluster | 2024 | McAllister et al. 2024 | read | one cluster, 7 days |
| storage, floor per user | disk only | 0.015 to 0.15 | kWh/user-year | AVG; 1-10 GB x 15.4 | mid-2020s | derived | scout's arithmetic | marginal saving is zero in the short run |
| storage share | of US data-centre electricity | 9 (16 of 176 TWh) | % | modelled | 2023 | LBNL 2024 | read | drives only; all workloads |
| storage share | of operational emissions, general-purpose cloud | 33 (racks alone 24) | % | operator-reported via paper | 2024 | McAllister et al. 2024 | read | Azure, emissions, includes storage servers |
| storage, top-down per user | 9-33 % of 5.5 kWh | 0.5 to 1.8 | kWh/person-year | AVG; transfer between systems | 2024 | derived | scout's arithmetic | a bracket, not a measurement |
| harvesting | ML share of operator energy | 10 to 15 | % | AVG; operator-reported, three sample weeks | 2019-2021 | Patterson et al. 2022 (Google) | read | pre-generative-AI; Google incl. Cloud |
| harvesting | ML split | inference 3/5, training 2/5 | share | operator-reported | 2019-2021 | same | read | same |
| harvesting | AI fleet power capacity | 10 : 20 : 70 | experimentation : training : inference | CAP; operator-reported | 2021 | Wu et al. 2022 (Meta) | read | within the AI fleet; not a share of the total |
| harvesting | one recommendation model, end to end | 31 : 29 : 40 | data : training : inference | AVG; operator-reported | 2021 | Wu et al. 2022 | read | one model |
| harvesting | training data growth | 2.4x in two years, exabyte scale; ingestion bandwidth 3.2x | ratio | operator-reported | 2019-2021 | Wu et al. 2022 | read | two use cases |
| harvesting | log transport | over 2.5 in, over 7 out | TB/s | CAP; operator engineering post | 2019 | Meta, Scribe post | read at source | all machine logs; about 100 MB per daily person per day by scout's arithmetic; 15 TB/s in 2025 is a search summary |
| harvesting | AI servers, US | over 40 of about 100 TWh of server electricity | TWh | modelled | 2023 | LBNL 2024 | read | US-wide |
| harvesting, per user | storage + analytics + ranking + ads | 0.5 to 5.5; scout's central 2 to 4 | kWh/person-year | AVG; NOBODY PUBLISHES THE SPLIT | 2024 | derived from Meta, Google, Wu | scout's estimate | long-run only; short-run about zero |
| serving-only messenger | infrastructure cost | 14 (servers 2.9, storage 1.3, bandwidth 2.8, SMS 6) | million USD/year | operator-reported | 2023 | Signal | read at source | dollars, not energy; user count not published |
| advertising | online advertising, world | 106 (20 to 282) | TWh/year | AVG; modelled, allocation by traffic share | 2016 | Pärssinen et al. 2018 | read (university repository) | includes user devices and networks; not marginal |
| advertising | tracking share of web page bytes | 21.1 | % | measured share | 2021-2022 | Pachilakis et al. 2023 (arXiv 2304.00927) | read | browser pages, not apps; its Mt figure uses 1.8-3.1 kWh/GB and should not be used |
| advertising | Facebook feed ad load | 13 (s.d. 4.6) | % of feed items | measured, field experiment | 2020s | Beknazar-Yuzbashev et al., Warwick WP 1602 (2026) | read | Facebook feed, extension users |
| advertising | ads share of mobile bytes | 1 | % | measured, carrier trace | 2012 | Vallina-Rodriguez et al., IMC 2012 | read (author's copy) | banner era |
| advertising | programmatic, per 1,000 impressions | 514.8 (187 to 1,772) | g CO2e | AVG; commercial model | 2023 | Scope3 | search summary | carbon not energy; vendor; another vendor says 4,240 |
| advertising | impressions per feed-hour | 40 to 80 | per hour | AUTHOR'S ESTIMATE from 13 % and assumed items per hour | - | derived | scout's estimate | not published by any operator |
| device | energy of apps with ads vs without | +16 (CPU +48, network data +79) | % | MARG; measured, 21 apps | 2015 | Gui et al., ICSE 2015 | read | lab workloads, Android 2014 |
| device | share of free-app energy in ad modules | 65 to 75 | % | measured, six apps, 3G | 2012 | Pathak et al., EuroSys 2012 | mirror of the paper | 3G tails; games and news apps; obsolete regime |
| device | ads' share of app communication / total energy | 65 / 23 | % | measured, 15 apps | 2013 | Mohan et al., EuroSys 2013 | search summary | Windows Phone |
| device | ad SDK refresh interval | 12 to 120 | s | SDK setting | 2012 | Vallina-Rodriguez et al. 2012 | read | banner SDKs |
| device | OS telemetry cadence, idle | 4.5 | minutes | measured, two handsets | 2020 | Leith 2021 | read at source | to Apple/Google; NOT removed by a mesh |
| device | ad energy saved per phone | 0.2 to 0.3 | kWh/phone-year | MARG; 16 % x assumed third of use x 4.5 kWh | - | derived | scout's arithmetic | upper end; the share of use is assumed |
| substitution | uninstalled the incumbent | 0.5 | % of users | survey, 1,525 users, four countries | 2021 | Griggio et al., CHI 2022 | university news page | self-report |
| substitution | wanted to switch / succeeded | 26 / about 6 | % of users | same | 2021 | same | university news page | |
| substitution | used the incumbent less / used others more | 16 / 27 | % of users | same | 2021 | same | search summary | |
| substitution | forced removal, time freed | 60 min/day; other social media also fell | minutes | randomised, 2,844 US users | 2018 | Allcott et al., AER 2020 | read (NBER) | removal, not adoption |
| substitution | forced removal, diversion | +22.7 % other social, +10.4 % communication; phone time -27 min/day | % | randomised, students | 2021 | Aridor, RAND 2025 | working paper read | short, small |
| substitution | multi-homing, UK | WhatsApp 90 %, Messenger 58 %, Facebook+Messenger 93 % of online adults; 79 % overlap | % | passive commercial panel | 2025 | Ofcom Online Nation 2025 | mirror | reach only |

Proposed additions to `not_available`:
- energy or capacity by workload (serving / analytics and ML / storage) for any messaging or social operator
- bytes stored per user, and years retained
- marginal upstream energy of one user or one household leaving (the measured storage response to load is near zero)
- ad impressions per user-hour by app class
- energy per ad impression (only vendors' carbon models, disagreeing eightfold)
- current tracker and telemetry requests per hour per app on a current phone
- time use before and after adopting a private messenger

---

## (b) Trust

Most trusted:
1. **Meta's electricity per daily active person, 5.5 kWh in 2024**, with its series. The operator's own audited-style disclosure, the only per-user figure there is. Its kind (average allocation) is the caveat, not its accuracy.
2. **The replication factors (3.6, 2.8, 2.1) and "blob storage and data warehouse both store exabytes"**, from the operator's peer-reviewed systems papers.
3. **Griggio et al.: 26 % wanted to leave, a quarter managed, 0.5 % uninstalled.** One real migration, a designed survey. (Read on the university's page; the paper should be opened before a stake cites it.)

Least trusted:
1. **Bytes per user (1 to 10 GB)** and everything multiplied from it. My estimate; no source.
2. **Any per-impression figure** (Scope3's 0.5 g, the rival 4.24 g, my 40 to 80 impressions an hour). Vendor carbon models read from search summaries, and one assumption of mine.
3. **Pathak's 65 to 75 % and Pärssinen's 106 TWh.** Both real, both famous, both prices in a regime that has gone (3G radio tails; 2016 traffic-share allocation with a factor-of-fourteen range).

---

## (c) For a human to open

- **scope3.com** (JavaScript page; report PDFs gated): the State of Sustainable Advertising report, for the per-impression figure, its boundary, and whether it states kWh.
- **adnetzero.com**: Global Media Sustainability Framework v1.2/v1.3 digital formulae and default values (not opened; a default kWh per impression may be there).
- **dl.acm.org**: Griggio et al. CHI 2022 (the 27 % / 16 % and the median of five apps are from search summaries); Mohan et al. EuroSys 2013 (the Microsoft-hosted PDF returned a web page).
- **ofcom.org.uk**: Online Nation 2025 and its data annex, for Signal and Telegram reach and any cross-app overlap table (read here from a third-party mirror).
- **Meta's 2026 Environmental Data Index** (2025 data), when published: the next point in the per-person series. The URL I guessed did not exist.
- **Google's 2026 Environmental Report** (2025 data): the +37 % is a search summary.
- **VLDB 2025 Scribe paper** (15 TB/s): search summary only.
- **usenix.org** returned "page not found" for the paper URLs I had; f4 and Tectonic were read from a co-author's university mirror, which is adequate.
- **faq.whatsapp.com** (did not render): the server-side retention statement for delivered and undelivered messages.
- **IEA "Energy and AI"** and its servers/storage/network split, **Pew** and **Reuters Institute** app-overlap tables, **SNIA Emerald**, **Backblaze**: not opened in this pass; LBNL and Azure covered the storage split, and Ofcom and Griggio the overlap.
- The LBNL report copy on eta-publications.lbl.gov carries "EMBARGOED REPORT: DO NOT DISTRIBUTE" in its page footers although it is the publicly linked file; cite the report, not the footer.
