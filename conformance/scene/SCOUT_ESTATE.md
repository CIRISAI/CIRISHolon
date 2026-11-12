# SCOUT: the data-centre estate under full replacement — what stays, what vanishes, what relocates, and at what efficiency

Scouted 2026-11-11. Documentation only: agency and national-lab reports, operator papers, academic papers, vendor specification pages, one commercial analyst's public press releases. No dataset was downloaded, no value-returning data API was called, nothing was signed up for. Report and paper PDFs were opened and read as documents. Companion to `conformance/scene/SCOUT_UPSTREAM.md` and `conformance/scene/SCOUT_DEVICES_POWER.md`; figures already in those are cited by name and not re-derived.

Marking convention (same as the earlier reports). Unmarked statements were read in the source document itself at the URL beside them. `[mirror]` = read in full from a copy not hosted by the publisher. `[search summary]` = seen only in a search engine's summary. `[my reading]` = my inference or recollection. `[my arithmetic]` = computed by me from figures read. `[not confirmed]` = could not open the source. `[author's estimate]` = a bracket I built; no source states it.

Two cautions, carried beside every figure:

- **Kind of figure.** AVG = long-run average allocation. MARG = marginal. CAP = capacity or provisioning. In a FULL-REPLACEMENT thought experiment the long-run average is the right kind for the BASELINE (the estate is not built at all). It is not automatically the right kind for the EDGE side: there the question is marginal draw on devices already on, plus the whole draw of anything added.
- **A constant is a price in a regime.** Year, system, and rate of movement are given. Two figures in this report moved by more than an order of magnitude inside the period the literature covers (data-centre energy per delivered bit: 77x between a 2016 model and a 2020 measurement; hosted energy per AI prompt: 33x in twelve months at one operator).

---

## Headline findings

1. **The baseline is confirmed at source and has a by-owner split the team did not have.** IEA, *Energy and AI* (April 2025), Annex Table A.1: world data centres 2024 = **416 TWh**, of which **hyperscale 166, colocation and service provider 144, enterprise 106**. Modelled, bottom-up from server shipments. It excludes cryptocurrency mining and networks.
2. **The user count has been revised.** ITU published 5.5 billion (68 %) for 2024 in November 2024 and revised 2024 to **5.8 billion (71 %)** in the 2025 edition; 2025 is 6.0 billion (74 %). On the revised count the baseline is **72 kWh per user-year**, not 75.
3. **Input 1 (share that stays) should move UP, from 0.4 to about 0.6 (0.45 to 0.75).** Enterprise-owned data centres alone are 25 % of the energy and stay by definition. Meta and Google together, counted wholly as consumer, are 50.5 TWh, 12 % of the total. A stay share of 0.4 needs 250 TWh of consumer work, five times those two. No source splits energy by consumer versus business; this is a bracket from two derivations.
4. **Input 2 (vanish versus relocate) cannot be sourced and the guess of 0.5 stands (0.3 to 0.7).** What can be sourced cuts in one direction worth stating: video is 54 to 68 % of fixed downstream bytes and relocates, but its DATA-CENTRE energy is small (1.3 Wh per viewing hour, measured by operators, 2020; about 1 % of the streaming chain). Bytes are not data-centre energy. The data-centre energy of consumer platforms is mostly compute, and at the one operator that has said anything, recommendation was over 79 % of AI inference cycles (2019).
5. **Input 3 (edge-to-data-centre efficiency) is not one number. It is two: a marginal ratio on devices already on (about 0.1 to 1) and the fixed watts of anything added.** A dedicated two-bay home storage box draws 39 to 129 kWh a year, the same order as the entire per-user baseline. Every published result in which the edge wins (2009 to 2016) assumed the home gateway was already on and compared against data centres far less efficient than those of 2020.
6. **Input 4a (AI share): 0.15 is exactly the IEA's 2024 proxy and is a floor for any later year.** It counts accelerated-server electricity with no cooling overhead; with overhead it is 17 to 21 %; in the US in 2023 it was already 23 % of the total by the national lab's model; at the IEA's own growth rates it is about 19 % in 2026 and 32 % in 2030. It also overlaps Input 2: ranking and recommendation run on the same accelerated servers.
7. **Input 4b (device versus data centre): for the SAME model the device costs more, about 3 to 30 times per token `[author's estimate from measured endpoints]`; for the task AS SERVED (a small model on the device against a frontier model hosted) it can cost less, 0.3 to 5 times.** The literature compares the second and calls it a saving. Frontier-size models do not run on phones at all, so part of this workload neither vanishes nor relocates to homes.

---

## 0. The two baseline numbers

| Figure | Value | What it measures | Basis | Kind | Year | Source, how read | What it is NOT |
|---|---|---|---|---|---|---|---|
| World data-centre electricity | 415 TWh (text); 416 TWh (Annex A.1) | "Data centres accounted for around 1.5% of the world's electricity consumption in 2024, or 415 terawatt-hours (TWh)" | modelled; bottom-up from server shipments (IDC, Omdia, SemiAnalysis), utilisation, idle power and PUE by type and region | AVG | 2024 | IEA, *Energy and AI*, April 2025, https://www.iea.org/files/de9dea13-b07d-42c5-a398-d1b3ae17d866/EnergyandAI.pdf , read (text CC BY 4.0; Annex A is CC BY-NC-SA 4.0) | Not measured. Not cryptocurrency (about 125 TWh in 2023, counted apart). Not networks (about 280 TWh, 2023). Not devices (about 440 TWh, 2023). |
| Same, series | 269 (2020), 361 (2023), 416 (2024), 946 (2030 Base Case) | same | modelled | AVG | - | same, Table A.1 | 2030 is a scenario |
| Growth | "12% per year over the last five years"; 10 % a year 2015-2024; 3 % a year 2005-2015 | same | modelled | - | - | same | |
| IT electricity inside it | 295 TWh; fleet PUE 1.41 | servers, storage, network | modelled | AVG | 2024 | same | |
| Regional | US 45 % (about 180 TWh), China 25 % (about 100 TWh), Europe 15 % | consumption by location of the data centre | modelled | AVG | 2024 | same | Location of the building, not of the users served |
| Per capita | US about 540 kWh; Africa under 1 kWh | by location | modelled | AVG | 2024 | same | **A US data centre serves the world; do not read 540 as a US user's share** |
| Internet users, as first published | 5.5 billion, 68 % of world population | "In 2024 fully 5.5 billion people are online" | ITU estimate ("data modelling and imputation", nowcast for the latest year, from household surveys) | - | 2024 | ITU, Facts and Figures 2024, https://www.itu.int/itu-d/reports/statistics/2024/11/10/ff24-internet-use/ , read | Neither edition's text defines the recall period or device `[my reading: the ITU indicator is use in the last three months from any location; not confirmed on the pages read]` |
| Internet users, revised | 5.8 billion (71 %) for 2024; 6.0 billion (74 %) for 2025 | "up from 5.8 billion in 2024" | same | - | 2024, 2025 | ITU, Facts and Figures 2025, https://www.itu.int/itu-d/reports/statistics/2025/10/15/ff25-internet-use/ , read | A revision of 0.3 billion in one year: the denominator is itself a price in a year |
| Per user | 71.7 kWh/user-year (416 / 5.8); 75.5 on the first-published count | - | `[my arithmetic]` | AVG | 2024 | derived | A world mean over users whose use differs by two orders of magnitude |

Rate: total +15 % in 2024 (361 to 416); users +3 % a year. Per-user baseline rises about 11 % a year.

---

## 1. The share that stays

### 1.1 By owner type (read at source)

**IEA, *Energy and AI*, Annex Table A.1, world, 2024** (modelled, AVG):

| | Installed capacity, GW | IT capacity, GW | PUE | Load factor | Electricity, TWh | Share of TWh `[my arithmetic]` |
|---|---|---|---|---|---|---|
| Hyperscale | 36 | 31 | 1.14 | 53 % | 166 | 39.9 % |
| Colocation and service provider | 35 | 23 | 1.53 | 48 % | 144 | 34.6 % |
| Enterprise | 27 | 14 | 1.92 | 45 % | 106 | 25.5 % |
| Total | 97 | 68 | 1.41 | 49 % | 416 | 100 % |

- IEA's definitions: enterprise = "run by businesses or institutions for their own use ... around 28% of data centre capacity today. Their share has been steadily decreasing over time, from 85% in 2005"; colocation and service provider = "an estimated 36% of capacity"; hyperscale = "massive facilities operated by major technology companies, such as Amazon Web Services, Google, Meta and Microsoft ... from around 10% of data centre capacity in 2010 to 37% today".
- 2020: hyperscale 100, colocation 85, enterprise 85 TWh. Enterprise share fell from 32 % to 25 % in four years `[my arithmetic]`. 2030 Base Case: 378 / 355 / 213.
- The IEA states the gap outright: "There is no comprehensive data on the share of different kinds of workloads, and service providers or colocation data centre operators often have limited visibility over the specific workloads running in their facilities."
- What it is NOT: not a split by consumer versus business. A bank's own data centre is "enterprise" and serves consumers; a hyperscaler's cloud region serves businesses.

**LBNL, 2024 United States Data Center Energy Usage Report** (Shehabi et al.; https://eta-publications.lbl.gov/sites/default/files/2024-12/us_data_center_energy_usage_report_lbnl-2001637.pdf , read; modelled, US): nine space types (telco edge, commercial edge, SMB, enterprise branch, internal, communications service providers, colocation small/medium, colocation large, hyperscale). "In 2014, over 60% of server energy consumption was in internal data centers. By 2023, this fell to nearly 10%, with hyperscale and colocation data centers accounting for almost 80%." Servers by count: hyperscale 8 % (2014), 25 % (2020); hyperscale plus colocation 74 % (2023), 85 % by 2028. What it is NOT: energy by space type is given in figures only (Figures 5.2 and 5.7), not as a table; US only. The US has moved further from enterprise than the world: about 10 % of server energy internal against the IEA's 25 % of world electricity.

**Synergy Research Group** (commercial analyst; figures from its public press releases; the underlying data is sold; the releases state no unit for "capacity"):

| Date of release | Hyperscale | Non-hyperscale colocation | On-premise enterprise | Source, read |
|---|---|---|---|---|
| 7 August 2024 | 41 % | 22 % | 37 % | https://www.srgresearch.com/articles/hyperscale-operators-and-colocation-continue-to-drive-huge-changes-in-data-center-capacity-trends |
| 7 April 2026 (end Q4 2025) | 48 % | 20 % | 32 % | https://www.srgresearch.com/articles/hyperscale-operators-to-account-for-67-of-all-data-center-capacity-by-2031 |

- On-premise was 56 % in 2018; forecast 19 % in 2031. Hyperscale capacity: "Almost 60% ... is now in own-built, owned data centers with the balance being in leased facilities" (2026); "just over half" owned (2024).
- Synergy counts a hyperscaler's leased space as hyperscale; the IEA's hyperscale share of capacity (37 %) is lower than Synergy's (41 %) for the same year, and its colocation share higher (36 % against 22 %), which is consistent with the IEA placing leased space under colocation `[my reading]`. The two cannot be added.
- Kind: CAP. Seven points of share moved from on-premise and colocation to hyperscale in about 18 months.

**Germany** (Bitkom, study by Borderstep Institut, 21 November 2024, https://www.bitkom.org/Presse/Presseinformation/Rechenzentren-Deutschland-verliert-Anschluss , read): electricity 20 TWh in 2024 (12 in 2014). Connected IT load 2,730 MW: cloud 1,240 MW (45 %; 630 MW in 2019), traditional 1,310 MW (48 %, declining), edge 180 MW (7 %). AI and high-performance computing 15 % of capacity now, about 40 % forecast for 2030. What it is NOT: "cloud" is not "consumer"; no PUE value on the page.

**EU**: JRC (Kamiya and Bertoldi, 2024, JRC135926): EU-27 data centres 45 to 65 TWh in 2022 (1.8 to 2.6 % of EU electricity); telecommunication networks 25 to 30 TWh `[search summary; the JRC repository refused a scripted read]`. Any split by type inside it was not seen.

**Uptime Institute Global Data Center Survey 2025**: behind a form; nothing beyond what `SCOUT_DEVICES_POWER.md` F10 already has `[not confirmed]`.

### 1.2 Inside hyperscale: consumer service or business cloud

- **Synergy, 19 December 2025** (https://www.srgresearch.com/articles/hyperscale-spending-spree-is-driving-dramatic-growth-in-data-center-capacity , read): Amazon, Microsoft and Google "in aggregate ... now account for 58% of all hyperscale data center capacity" (Q3 2025; 60 % in an earlier release `[search summary]`). Next: Meta, Alibaba, Tencent, Oracle, Apple, ByteDance. Coverage: 21 firms, "the largest operators in SaaS, IaaS, PaaS, search, social media, e-commerce and gaming". Kind: CAP, commercial analyst. What it is NOT: not a split by use. Google's capacity carries Search, YouTube and Gmail as well as Cloud; Amazon's carries its shop; Microsoft's carries consumer services; and the business clouds host consumer platforms (streaming, social and games companies rent them).
- **Operator electricity, read at source** (from `SCOUT_UPSTREAM.md`): Meta 18.4 TWh (2024; data centres 18.06), wholly consumer. Google 32.1 TWh (2024), consumer services plus Cloud, split not published. IEA: "in 2023, the largest four hyperscalers (Google, Amazon, Meta and Microsoft) had a combined data centre electricity consumption in the order of 90 TWh"; Meta 15.3 and Google 25.3 in 2023, so Microsoft plus Amazon were of the order of 50 TWh `[my arithmetic on a rounded total]`. Amazon and Microsoft totals were not found at source.
- **Cisco Global Cloud Index 2016-2021** (white paper of February 2018; a FORECAST; Cisco has withdrawn the paper): consumer workloads 27 % of all data-centre workloads and compute instances by 2021 (24 % in 2016), enterprise 73 %; inside consumer, video streaming 34 % and social networking 25 % of consumer workloads `[search summary of a third-party copy; not read at source]`. Kind: a count of workloads, not energy; a forecast; eight years old. It is the only published consumer-versus-enterprise split of any kind that I found.
- **Segment revenue** is the weak proxy the brief names. I did not collect it: it prices a product, not a kilowatt-hour, and an advertising dollar and a cloud dollar buy very different amounts of compute. I would not use it.

### 1.3 Bracket and derivation `[author's estimate]`

Derivation A, by owner type (IEA shares, my stay fractions):

| Type | Share of TWh | Stay fraction, low to high | Why |
|---|---|---|---|
| Enterprise | 0.255 | 0.9 to 1.0 | businesses' and institutions' own IT |
| Colocation and service provider | 0.346 | 0.3 to 0.7 | mixed: enterprise tenants, telecoms, and hyperscalers' leased halls |
| Hyperscale | 0.399 | 0.2 to 0.5 | the three business clouds are 58 % of it and are themselves mixed; the rest is mostly consumer platforms |

Stay = 0.41 to 0.70 `[my arithmetic]`.

Derivation B, by operator: Meta plus Google, counted wholly as consumer, are 50.5 TWh = 12 % of 416. Adding everything else that is plausibly consumer (the consumer side of Apple, Amazon and Microsoft; the Chinese platforms, inside a Chinese total of about 100 TWh; consumer companies on rented cloud and colocation) I cannot defend more than about 105 to 185 TWh, 25 to 45 %. Stay = 0.55 to 0.75. Only the Meta and Google totals in this derivation are sourced.

Cross-check: Cisco's forecast, by workload count, would put stay at 0.73.

**Sourced range: 0.45 to 0.75, centre about 0.6. The team's 0.4 (0.2 to 0.6) should move up.** A value below 0.35 is not defensible: enterprise alone is 0.25 and colocation is not empty of enterprise tenants. Kind: AVG. Rate: the enterprise-owned share falls about 1.7 points a year, but that work moves to cloud and colocation and still stays; the consumer share is rising with consumer AI.

What this is NOT: telecom core is partly inside these totals (LBNL has a "communications service providers" space type) and stays. Cryptocurrency is outside the 416.

---

## 2. Of consumer work, what vanishes and what relocates

### 2.1 Traffic composition (bytes), read at source

**Sandvine, Global Internet Phenomena Report 2024** (data for 2023; https://www.sandvine.com/hubfs/Sandvine_Redesign_2019/Downloads/2024/GIPR/GIPR%202024.pdf , read). A network-equipment vendor's report from its operator customers' networks; measured by traffic classification; AVG; the sample of networks is not published.

| Fixed, downstream, by content category | Share | GB per subscriber per day |
|---|---|---|
| On-demand streaming | 54 % | 7.9 |
| Live streaming | 14 % | 2.0 |
| File delivery | 13 % | 2.0 |
| Browsing | 3 % | 0.44 |
| Game play | 3 % | 0.40 |
| Video call | 2 % | 0.30 |
| Messaging | 0.6 % | 0.08 |
| Voice call | 0.5 % | 0.07 |

- Mobile, downstream: on-demand streaming 57 %, file delivery 11 %, live streaming 7 %, game play 5 %, video call 5 %, browsing 2 %.
- By application category: fixed, video 39 %, social media 18 %, television 11 %, file sharing 9 %, device gaming 7 %, general web 6 %; mobile, social media 35 %, video 31 %. Fixed total 14.5 GB down per subscription per day; mobile 1.6 GB.
- Eight companies (Alphabet, Meta, Netflix, Microsoft, TikTok, Apple, Amazon, Disney) are "65% of total internet volume".
- **Added to `SCOUT_UPSTREAM.md` section 3:** the 2025 edition (AppLogic Networks, read): "Ad Analytics is gathered from 93% of Fixed network subscribers and 76% of Mobile network subscribers every day ... Ad Analytics traffic comprises less than 2% of an average subscriber's daily volume, but still comes to 66MB per subscriber per day in Fixed networks and 6 MB per subscriber per day for Mobile". Measured share of bytes, 2024. It is NOT the ad creatives themselves, and not energy.
- **Ericsson Mobility Report** (page dated June 2026, read): mobile data 146 EB a month at end 2025 (203 with fixed wireless); 22 GB per active smartphone per month. The video share is not on the page read `[my reading: Ericsson has put video at about three-quarters of mobile traffic; not confirmed]`.

**Reading for this scenario:** on-demand video is the largest consumer workload by bytes (54 to 57 %) and RELOCATES: people still want to watch what they chose. Social media's 18 to 35 % of bytes is mostly video too `[my reading]`; the feed that selects it vanishes, the video does not.

### 2.2 Bytes are not data-centre energy: how the literature prices video

| Figure | Value | What it measures | Basis | Kind | Year | Source, how read | What it is NOT |
|---|---|---|---|---|---|---|---|
| Data centres per viewing hour | 1.3 Wh/hour | hosting, encoding and content-delivery nodes, per hour viewed; "derived from a selection of DIMPACT members based on measured data in 2020" | operator-measured, averaged over members | AVG | 2020 | Carbon Trust, *Carbon impact of video streaming*, June 2021, https://www.carbontrust.com/sites/default/files/documents/resource/public/Carbon-impact-of-video-streaming.pdf , read | Professional video-on-demand and broadcasters. NOT user-uploaded video with recommendation and ads (YouTube, TikTok) |
| Its share of the chain | "approximately 1Wh/hour, representing roughly 1% of total emissions and energy"; network 20 Wh/h (10 %), home router 71 Wh/h (38 %), devices 96 Wh/h (51 %); total 188 Wh/h | European average, conventional (average-allocation) method | modelled around the measured data-centre term | AVG | 2020 | same | The router and network terms are average allocations of always-on equipment; the report's own power-model method gives far less |
| Why so small | "CDNs ... are therefore computationally light, only reading and writing data copied from the core content stack" | - | - | - | - | same | |
| IEA commentary | 0.077 kWh per hour; devices 72 %, data transmission 23 %, data centres 5 % (about 0.004 kWh/h `[my arithmetic]`) | "streaming a Netflix video in 2019 typically consumed around 0.077 kWh" | modelled, average viewing habits; data-centre term from Netflix's reported electricity | AVG | 2019 | Kamiya, IEA commentary, 10-11 December 2020, https://www.iea.org/commentaries/the-carbon-footprint-of-streaming-video-fact-checking-the-headlines , read (last part by summary) | Three times the Carbon Trust's data-centre term a year earlier |
| Netflix, everything | 0.45 TWh in 2019, including studios and offices; 2.7 kWh per paid membership-year at 167 million `[my arithmetic; member count from a search summary]` | company total, direct plus indirect including rented cloud | operator-reported | AVG | 2019 | via the IEA commentary | Not delivery alone |

- `[my arithmetic; viewing hours ASSUMED]` 5.8 billion users at 1 to 3 hours a day and 1.3 Wh/h is 3 to 8 TWh a year, 0.7 to 2 % of 416 TWh; at 3.9 Wh/h, 8 to 25 TWh, 2 to 6 %. **Video delivery is most of the bytes and a few per cent of data-centre energy.**
- So the relocating workload that dominates traffic does not dominate the estate. What does is compute.

### 2.3 Where data-centre energy goes: compute, storage, network

- IEA 2024 (read): servers "around 60% of electricity demand in modern data centres"; storage "around 5%"; networking "up to 5%"; cooling "from about 7% for efficient hyperscale data centres to over 30% for less-efficient enterprise data centres". Modelled AVG. By data-centre type the split is in Figure 2.2 only (a chart).
- LBNL, US 2023: storage 9 % (drives only), network about 3 %, infrastructure 30 % (already in `SCOUT_UPSTREAM.md` 1.4).
- So storage, the thing that plainly relocates, is 5 to 9 % of energy (up to 33 % of operational emissions when storage servers are counted, Azure, per `SCOUT_UPSTREAM.md`). Compute is 60 % and upward.

### 2.4 Ranking and recommendation as a share of compute

- **Gupta et al. (Facebook), "The Architectural Implications of Facebook's DNN-based Personalized Recommendation", HPCA 2020** (arXiv 1906.03109, submitted June 2019, read at https://arxiv.org/pdf/1906.03109): three recommendation model classes "consume 65% of AI inference cycles in Facebook's production datacenter"; with other recommendation models they "comprise over 79% of AI inference cycles" (body text: "up to 79% of AI inference cycles in a production-scale data center"). Kind: operator-reported share of CYCLES, within AI inference, on CPUs, 2019. What it is NOT: not a share of Meta's energy; not after the move to GPUs; not after generative AI. No later operator statement of the kind was found.
- Already in `SCOUT_UPSTREAM.md`: Meta AI fleet 10 : 20 : 70 experimentation : training : inference (2021); one recommendation model 31 : 29 : 40 data : training : inference; Google ML 10 to 15 % of energy (2019 to 2021); online advertising 106 of 1,059 TWh of the whole internet chain (2016, modelled). Not repeated.
- No operator publishes the share of its data-centre energy that is feed ranking, ad auctions, logging, experimentation or moderation. That finding of `SCOUT_UPSTREAM.md` stands.

### 2.5 Bracket and derivation `[author's estimate]`

By kind of consumer platform (weights unknown; vanish fractions mine):

| Kind | Vanishes | Why |
|---|---|---|
| Feed and social platforms | 0.6 to 0.9 | a serving-only messenger needs little (Signal); the warehouse and training corpora equal the content in size; recommendation dominates inference |
| Search | 0.2 to 0.5 | the ad auction vanishes; crawling, indexing and answering a query the user typed do not, and an index has to live somewhere |
| Video on demand | 0.1 to 0.4 | delivery relocates; recommendation and ad insertion vanish |
| Personal storage, mail, photos | 0.0 to 0.2 | relocates; scanning and extra analytic copies vanish |
| Consumer generative AI | 0 | relocates or stays hosted (Input 4) |

Anchored on the two sourced totals: Meta 18.4 TWh at 0.6 to 0.9 and the consumer part of Google's 32.1 TWh (taken as 22 to 28) at 0.2 to 0.5 give 15 to 30 TWh vanishing out of 40 to 46, a share of 0.33 to 0.65 `[my arithmetic]`.

**Sourced range: none. Author's bracket 0.3 to 0.7, centre 0.5. The team's guess of 0.5 stands; its range of 0.2 to 0.8 can be narrowed a little at both ends but not on evidence I would defend against a hostile reader.** Kind: AVG.

**Overlap to guard against:** ranking and recommendation run on accelerated servers, which are the IEA's proxy for AI (Input 4a). If AI is carved out first as 15 to 25 % of the estate and then "ranking vanishes" is applied to the remainder, ranking is counted once; if both are applied to the whole, it is counted twice.

---

## 3. Edge-to-data-centre efficiency for relocated work

### 3.1 The data-centre side: overhead, utilisation, idle

| Figure | Value | Regime | Basis | Kind | Source, how read |
|---|---|---|---|---|---|
| PUE by type | hyperscale 1.14; colocation 1.53; enterprise 1.92; all 1.41 | world, 2024 (1.19 / 1.67 / 2.05 / 1.53 in 2020) | modelled | AVG | IEA 2025, Table A.1, read |
| PUE, operators | 1.09 (Google, 2025); 1.08 (Meta, 2024) | own fleets | operator-reported | AVG | in the earlier scout reports |
| PUE, US | 1.4 in 2023 (1.6 in 2014) | US, all types | modelled | AVG | LBNL 2024, read |
| Utilisation | "enterprise data centres have average utilisation rates below 20%, while hyperscale data centres with optimised loads can have average utilisation rates of up to 50%" | world, today | IEA statement | AVG | IEA 2025, read |
| Utilisation, model input | internal 11 % (2014) rising to 20 % (2027); colocation 21 % to 35 %; hyperscale 45 % to 50 %; AI training 80 %, AI inference 40 % | US | modelling assumption | AVG | LBNL 2024, read |
| Server idle power | "from around 60% in 2010 to around 35% of rated power in modern servers"; 51 % of maximum (2014), 36 % (2023) | conventional servers | modelled from benchmark data | - | IEA 2025; LBNL 2024, read |
| Server efficiency | 44,168 overall ssj_ops per watt (highest 2025 result seen); others 19,500 to 25,000 | SPECpower_ssj2008, 2025 submissions | vendor-run benchmark | CAP | `[search summary of spec.org result pages]` |

`[my arithmetic]` With a linear power model (the IEA's own form: idle plus utilisation times the rest), energy per unit of work relative to a server at full load: hyperscale (utilisation 0.5, idle 0.35, PUE 1.14) = 1.35 x 1.14 = **1.5**; enterprise (0.2, 0.35, 1.92) = 2.4 x 1.92 = **4.6**. This is the overhead a device already on does not carry at the margin.

### 3.2 The edge side: what was found, with regime

| Figure | Value | Regime (year, hardware, workload, was the device already on) | Basis | Source, how read |
|---|---|---|---|---|
| Video on demand from home gateways against data centres | "Even under the most pessimistic scenarios, NaDa saves at least 20% to 30% of the energy"; optimistic "up to 60%"; data centres alone (network excluded) 58 % | 2009; ISP-managed gateways (a triple-play VDSL gateway); video on demand; trace-driven model; data-centre PUE 1.2; **gateway ASSUMED ALREADY ON** ("reuse of already committed baseline power"); gateways up 85 % of the time | modelled with measured gateway power and real access traces | Valancius, Laoutaris, Massoulié, Diot, Rodriguez, "Greening the Internet with Nano Data Centers", CoNEXT 2009, https://conferences.sigcomm.org/co-next/2009/papers/Valancius.pdf , read |
| Content served from home "nano servers" against a data centre | Either can win. Nano servers lose "when ... attached to an energy-inefficient access network or when the active time of dedicated nano servers is much greater than its idle time"; best case is data generated and consumed at home at low rates | 2014-2016; Raspberry Pi; content hosting; measured device power, modelled network; idle power allocated by share of active time | measured plus modelled | Jalali, Hinton, Ayre, Alpcan, Tucker, "Fog Computing May Help to Save Energy in Cloud Computing", IEEE JSAC 2016, read `[mirror]` at cesc.kth.se |
| Data-centre energy per bit used in that comparison | 4 to 7 microjoules per bit excluding PUE and replication; "around 20" including them; server utilisation "around 20%" | 2013-2016 model | modelled | same |
| Home gateway power, idle / maximum | Ethernet 2.8 / 4.6 W; ADSL2+ 4.1 / 6.7 W; GPON 5.2 / 8.3 W; 4G 0.5 / 1.75 W | about 2014 | measured | same, Table IV |
| Peer-to-peer against a content-delivery network | a CDN inside the ISP minimises total power; peer-to-peer "may reduce the power consumption of the service provider" but "increases the overall energy consumption" | 2010; internet television model | modelled | Feldmann, Gladisch, Kind, Lange, Smaragdakis, Westphal, CTTE 2010 `[search summary of the abstract]` |
| Cloud storage against a local disk | "private and public cloud storage services are more energy efficient than storage on local hard disk drives when files are only occasionally accessed"; as downloads per hour rise, transport dominates; public cloud storage "three to four times more power than private cloud storage" | 2010; a user's PC disk against cloud; **the PC was not assumed already on for this purpose** | modelled | Baliga, Ayre, Hinton, Tucker, "Green Cloud Computing", Proc. IEEE 99(1), 2011, read `[mirror]` at cesc.kth.se |
| Small nodes against a conventional machine | 364 queries per joule against 52 for a desktop | 2009; 500 MHz embedded nodes with flash; key-value lookups | measured | Andersen et al., FAWN, SOSP 2009 `[search summary; the desktop figure from a secondary summary]` |
| Home storage box | 14.69 W access; 4.41 W with disks hibernating | two-bay box, current model; drives fitted for the test not stated on the page | vendor specification | Synology DS224+, https://www.synology.com/en-us/products/DS224+#specs , read |
| Small board, idle | 0.6 A at 5.1 V = 3.1 W `[my arithmetic]` (Pi 4B, with a monitor, keyboard and Ethernet attached); Pi 5 off but plugged in "around 1 W to 1.4 W" | 2019-2023 boards | vendor documentation | https://www.raspberrypi.com/documentation/computers/raspberry-pi.html , read |
| Nedevschi et al. 2008 (network idle power) | not retrieved | - | - | `[not confirmed]` |

### 3.3 Three things the figures say when put side by side `[my arithmetic]`

1. **The edge-wins literature compared against a data centre that no longer exists.** The Carbon Trust's measured 1.3 Wh per viewing hour (2020) is about 0.26 microjoules per bit at an assumed 5 Mbit/s (the bit rate is my assumption). Jalali's model used 4 to 20. That is 15 to 77 times. A constant is a price in a regime: the 2009-2016 conclusions do not transfer.
2. **Media delivery from a gateway already on: about 0.1. From one charged its idle power: 2 to 4.** An Ethernet gateway's 1.8 W between idle and maximum over 100 Mbit/s is 18 nanojoules per bit; a 5 Mbit/s stream adds 0.09 W, 0.09 Wh per hour, against 1.3 at the data centre. Charge the gateway's idle 2.8 to 5.2 W to one stream and it is 2.8 to 5.2 Wh per hour. The whole answer is whether the device was on anyway. (Upstream capacity is a separate limit: the ADSL row has 3 Mbit/s up.)
3. **Storage on a dedicated box is about equal per terabyte PROVISIONED and far worse per terabyte STORED, and its absolute draw is the size of the baseline.** 4.41 to 14.69 W is 39 to 129 kWh a year. With 8 TB usable that is 4.8 to 16 kWh per TB-year against the data centre's 15.4 (11.5 to 28.2) per logical TB-year in `SCOUT_UPSTREAM.md`: ratio 0.3 to 1.0. But a household holds perhaps 0.2 to 1 TB of its own data `[my reading]`, so per stored terabyte the box is 2.5 to 40 times the data centre. And 39 to 129 kWh a year per household is 15 to 51 kWh per person at 2.5 people, against a whole baseline of 72. Storage on flash in a device already on adds close to nothing; I found no sourced idle wattage for that and do not give one.

### 3.4 General compute

No like-for-like published comparison of energy per operation between a current phone or small board and a current server was found. SPECpower has no small-board results. What exists is the token-per-joule evidence in section 4, which says small devices are worse per unit of the same work, by a factor that depends on batching.

**Sourced range by kind:**

| Kind | Marginal ratio on a device already on (edge / data centre) | With a dedicated added device |
|---|---|---|
| Media delivery | about 0.1 `[my arithmetic]`; published end-to-end 0.4 to 0.8 (2009, model) | 2 to 4, or the device's fixed draw |
| Storage | near 0 on existing flash (not sourced) | 0.3 to 1.0 per provisioned TB; 2.5 to 40 per stored TB; 39 to 129 kWh a year fixed |
| General compute | 0.5 to 3 `[author's estimate]`: the data centre carries a 1.5 overhead the device does not, the device's silicon is worse per operation by an unsourced factor | 3 W idle is 27 kWh a year whatever it computes |

**The team's single ratio of 1.0 (0.3 to 3) should be replaced by this two-part form.** As a single number it is not wrong at its centre, but its range should widen to about 0.1 to 4, and the range is almost entirely one binary input: was the device already on.

Not counted anywhere above: waste heat at home displaces heating in a heating season and adds to cooling load in a cooling season. Valancius et al. treat distributed heat as a benefit. No figure was found.

---

## 4. AI

### 4a. Share of data-centre electricity

| Figure | Value | Definition | Basis | Kind | Year | Source, how read | What it is NOT |
|---|---|---|---|---|---|---|---|
| World | "Accelerated servers accounted for 24% of server electricity demand and 15% of total data centre demand in 2024" | electricity of accelerated (GPU and similar) servers, as a proxy for AI | modelled | AVG | 2024 | IEA 2025, Box 2.1, read | "an imperfect proxy": some inference runs on conventional servers; some accelerated servers do scientific computing. The IEA: "the range of estimates for the share of AI in total data centre electricity consumption is very wide" |
| Same, with overhead | 17 to 21 % | 24 % x 60 % = 14.4 %, so the 15 % carries no cooling; times PUE 1.14 (if all in hyperscale) to 1.41 (fleet) | `[my arithmetic]` | AVG | 2024 | derived | my allocation of overhead |
| Growth | accelerated-server electricity "+30% annually in the Base Case"; conventional +9 %; accelerated servers "almost half of the net increase" to 2030 | same | modelled scenario | - | 2024-2030 | IEA 2025, read | a scenario |
| Implied later shares | about 17 % (2025), 19 % (2026), 32 % (2030), before overhead | 0.15 x (1.30 / 1.147)^n | `[my arithmetic]` | AVG | - | derived | not an IEA statement |
| United States | AI servers "more than 40 TWh" of "nearly 100 TWh" of server electricity, in a total of 176 TWh: 23 % of the total before overhead | accelerated and AI servers | modelled | AVG | 2023 | LBNL 2024 (via `SCOUT_UPSTREAM.md`) | US only; under 2 TWh in 2017 |
| Germany | AI and high-performance computing 15 % of capacity; about 40 % by 2030 | capacity | industry study | CAP | 2024 | Bitkom / Borderstep, read | capacity, includes scientific computing |
| Analyst estimates | Deloitte, Gartner, Goldman Sachs, Schneider Electric, SemiAnalysis | - | analyst estimate | - | 2024 | named by the IEA as the sources of its Figure 2.5 range; none read | not read |

**Sourced range: 0.15 (2024, world, servers only) to 0.25 (2026 with overhead, or the US in 2023). The team's 0.15 is correct for 2024 as the IEA defines it and should carry its year; for a 2026 scenario 0.20 to 0.25 is the better centre.** It is the fastest-moving number in the estate.

What it is NOT, for this scenario: not consumer AI. It includes ranking and recommendation (vanishes), business AI (stays), model training (cannot relocate to homes at all), and scientific computing on GPUs (stays). No source splits it. Training against inference: Google 2/5 training, 3/5 inference (2019-2021); Meta AI fleet 10 : 20 : 70 (2021); LBNL assumes training servers busy 80 % of the time and inference servers 40 %.

### 4b. Energy of inference: hosted against on the device

**Hosted (data centre):**

| Figure | Value | What it measures | Basis | Year | Source, how read | What it is NOT |
|---|---|---|---|---|---|---|
| Google | 0.24 Wh per median text prompt | accelerators 0.14 (58 %), host CPU and memory 0.06 (25 %), idle provisioned machines 0.02 (10 %), data-centre overhead 0.02 (8 %); fleet PUE 1.09 | operator-measured in production | May 2025 | Elsworth et al., "Measuring the environmental impact of delivering AI at Google Scale", arXiv 2508.15734, read | A median, one product, text only. Excludes training, networks and the user's device. On the narrow boundary others use: 0.10 Wh |
| Google, movement | "33x reduction in energy consumption ... over one year" | same prompt, May 2024 to May 2025 | operator-reported | 2024-2025 | same | **a factor of 33 in twelve months** |
| OpenAI | about 0.34 Wh per average prompt | not stated | operator statement, no method | June 2025 | as quoted in the Google paper; the original not read | no boundary given |
| IEA tests | 0.1 Wh (1.7 billion parameters), about 0.3 Wh (9 billion), about 4 to 5 Wh (70 billion) per text task; image 1.7 Wh; six-second video 115 Wh; batching about halves | GPU electricity only, open models, laboratory, unbatched | measured (with Luccioni) | 2025 | IEA 2025, section 1.4, read | "highly controlled experimental results"; GPU only |
| Open models, one GPU | 0.047 Wh mean per text generation; 2.9 Wh per image | 1,000 inferences on a benchmark, models up to about 11 billion parameters, A100 | measured | 2023 | Luccioni, Jernite, Strubell, "Power Hungry Processing", FAccT 2024, arXiv 2311.16863, read | unbatched; small models |
| Open models, production stack | 0.15 to 0.21 J per token (a 32-billion-parameter model, batch 128 to 512); 95 J = 0.026 Wh per conversational response of 627 tokens; mean over models 184 J = 0.05 Wh; with reasoning 4,625 J = 1.3 Wh | steady-state energy per generated token, current GPU node, batched | measured | 2025-2026 | Chung, Wu, Ma, Chowdhury, "Where Do the Joules Go?", arXiv 2601.22076, read | no PUE, no idle capacity; apply about 1.7 for Google's measured boundary `[my arithmetic: 0.24 / 0.14]` |
| de Vries 2023 | about 3 Wh per prompt | model-based estimate | author's estimate | 2023 | as summarised in the Google paper; not read | superseded by measurement |

**On the device:**

| Figure | Value | Regime | Basis | Source, how read | What it is NOT |
|---|---|---|---|---|---|
| Flagship phones | 0.16 to 0.21 mWh per token = 0.58 to 0.76 J per token `[conversion mine]`; 490 to 590 prompts per full battery; sustained draw 8.5 to 13.8 W | 2022-2023 phones (Galaxy S23, iPhone 14 Pro), a 3-billion-parameter model quantised to 4 bits, on the phone's GPU; measured at the battery terminals | measured | Laskaridis, Katevas, Minto, Haddadi, "MELTing point", MobiCom 2024, arXiv 2403.12844, read | a 3-billion-parameter model is not a hosted frontier model; no NPU; one batch of one |
| Laptop | 0.09 to 0.25 J per token for 3- to 9-billion-parameter models | Apple M4 Pro, 2025-2026; package power measured at 0.47 W, **system power ASSUMED at 10 W** | measured throughput, assumed power | Kannan et al., "GreenBench", arXiv 2608.28667 (a preprint, August 2026), read | The headline "30-40x better ... than datacenter GPUs" compares against an unbatched GPU estimated as 400 W / 100 tokens a second. Do not use the headline |
| Small board | 2.61 J per token | Raspberry Pi 4, a 0.5-billion-parameter model | measured, as cited by the above | cited in GreenBench; original not read | `[not confirmed]` |
| Neural processor power | "NPUs on laptops consume in the range of 1 W to 5 W for most AI tasks"; phone NPU "around 0.5 W, around 80% lower than the CPU" | 2023-2024 | early studies cited by the IEA | IEA 2025, section 2.4.1, read | watts, not energy per task |
| What a phone can run | "15 tokens per second for a 10 billion parameter model" (Snapdragon 8 Gen 3); 30 to 45 tokens a second for 3 billion (Tensor G4) | 2024 flagships | vendor claims via the IEA | IEA 2025, Box 2.3, read | the ceiling, on the newest handsets |
| MLPerf Mobile / Client power | not retrieved | - | - | `[not confirmed]` |

**Putting them together `[my arithmetic]`:**

- Per token, a phone running a 3-billion-parameter model (0.58 to 0.76 J) costs 3 to 5 times a batched data-centre GPU running a model TEN TIMES LARGER (0.15 to 0.21 J), or 2 to 3 times after the data centre's 1.7 overhead.
- For the SAME model the gap is larger by roughly the ratio of model sizes, since energy per token rises with active parameters (the IEA's 40-fold parameters for 40-fold energy): of the order of **10 to 30 for a phone, 3 to 10 for a current laptop**. Nobody has published this comparison like for like; it is my estimate from measured endpoints.
- For the task AS SERVED the sign can reverse: a 627-token reply from the phone's small model is 0.10 to 0.13 Wh, against 0.24 to 0.34 Wh for a hosted frontier prompt (ratio 0.3 to 0.5) and against 0.026 to 0.05 Wh for an open model on a batched GPU (ratio 2 to 5).

**How the literature handles "comparable":** it does not hold the model fixed. The IEA: "AI models that can run on user devices are smaller and more efficient versions of their cloud-based counterparts ... often involve trade-offs between efficiency and accuracy", and concludes that a shift to the edge "is likely to reduce energy use in data centres with only a limited increase in energy use by devices". That is a statement about a smaller model doing a lesser job. Said plainly: **a small on-device model is not comparable to a large hosted one; where the same model is run in both places the device loses, because it cannot batch.** Batching is the data centre's structural advantage and a mesh of single-user devices cannot have it.

**Sourced range: same-model 3 to 30 `[author's estimate]`; as-served 0.3 to 5. The team's 3 (1 to 10) is a fair centre for the same-model case on a laptop-class device and too low for phones; its lower bound should drop below 1 only if the scenario accepts a lesser model.** And a third category is needed: hosted AI of frontier size neither vanishes nor relocates to homes; under "entirely" it either stays (a shared GPU hall is a data centre) or is given up. Training is in the same category.

---

## (a) Table to transcribe

| input | sub-component | value or range | unit | basis | year | source | how read | what it is not |
|---|---|---|---|---|---|---|---|---|
| baseline | world data-centre electricity | 415 (text); 416 (table) | TWh/year | AVG; modelled bottom-up | 2024 | IEA, Energy and AI (2025) | read at source (PDF) | not crypto (125 TWh, 2023), not networks (280), not devices (440) |
| baseline | same, series | 269 / 361 / 416 / 946 | TWh/year | modelled; 2030 is Base Case | 2020 / 2023 / 2024 / 2030 | IEA, Table A.1 | read | 2030 is a scenario |
| baseline | internet users, first published | 5.5 (68 %) | billion | ITU estimate | 2024 | ITU Facts and Figures 2024 | read at source | revised a year later |
| baseline | internet users, revised | 5.8 (71 %); 6.0 (74 %) | billion | ITU estimate | 2024; 2025 | ITU Facts and Figures 2025 | read at source | definition of "using" not on the pages read |
| baseline | per user | 71.7 (75.5 on the old count) | kWh/user-year | AVG; 416 / 5.8 | 2024 | derived | scout's arithmetic | a world mean; US per capita by location is 540 |
| 1 stay | electricity by owner type | hyperscale 166 (39.9 %); colocation and service provider 144 (34.6 %); enterprise 106 (25.5 %) | TWh/year | AVG; modelled | 2024 | IEA, Table A.1 | read | by owner, not by consumer vs business |
| 1 stay | capacity by owner type | hyperscale 37 %; colocation 36 %; enterprise 28 % (85 % in 2005) | % of capacity | CAP; modelled | 2024 | IEA, section 2.1.2 | read | |
| 1 stay | capacity by owner type, analyst | hyperscale 41 / colocation 22 / on-premise 37; then 48 / 20 / 32 | % of capacity | CAP; commercial analyst press release | 2024; end 2025 | Synergy Research Group | read at source | unit not stated; leased hyperscale counted as hyperscale; data sold |
| 1 stay | three business clouds' share of hyperscale | 58 | % of hyperscale capacity | CAP; commercial analyst | Q3 2025 | Synergy Research Group | read at source | not a split by use; all three also run consumer services |
| 1 stay | US server energy in internal data centres | about 10 (over 60 in 2014); hyperscale + colocation almost 80 | % of server energy | modelled | 2023 | LBNL 2024 | read | US; values in text, table not given |
| 1 stay | Germany, connected IT load | cloud 1,240 (45 %); traditional 1,310 (48 %); edge 180 (7 %); 20 TWh total | MW; TWh | industry study | 2024 | Bitkom / Borderstep | read at source | cloud is not consumer |
| 1 stay | consumer share of workloads | 27 (24 in 2016) | % of workloads and compute instances | FORECAST; vendor white paper | 2021 (written 2018) | Cisco Global Cloud Index 2016-2021 | search summary of a third-party copy | a count, not energy; withdrawn by the publisher |
| 1 stay | largest four hyperscalers' data-centre electricity | about 90 | TWh/year | IEA statement from company reports | 2023 | IEA, Box 1.3 | read | "in the order of" |
| 1 stay | SHARE THAT STAYS | 0.45 to 0.75; centre 0.6 | fraction of 416 TWh | AUTHOR'S ESTIMATE; two derivations | 2024 | derived | scout's estimate | no source splits energy by consumer vs business |
| 2 vanish | fixed downstream bytes by content | on-demand streaming 54; live 14; file delivery 13; browsing 3; games 3; video call 2; messaging 0.6 | % of bytes | measured by a vendor on customer networks | 2023 | Sandvine GIPR 2024 | read at source | bytes, not data-centre energy; sample of networks not published |
| 2 vanish | mobile downstream bytes by content | on-demand streaming 57; file delivery 11; live 7 | % of bytes | same | 2023 | same | read | |
| 2 vanish | bytes by application category | fixed: video 39, social 18, television 11; mobile: social 35, video 31 | % of bytes | same | 2023 | same | read | |
| 2 vanish | ad-analytics traffic | under 2 % of a subscriber's daily volume; 66 MB/day fixed, 6 MB/day mobile; 93 % of fixed subscribers daily | % ; MB | measured, vendor | 2024 | GIPR 2025 (AppLogic Networks) | read at source | analytics beacons, not ad creatives; not energy |
| 2 vanish | data centre per viewing hour | 1.3 (about 1 % of the 188 Wh chain) | Wh/hour | AVG; operator-measured, DIMPACT members | 2020 | Carbon Trust 2021 | read at source | professional video on demand; not user-uploaded video platforms |
| 2 vanish | data centre per viewing hour, earlier | about 3.9 (5 % of 77) | Wh/hour | AVG; modelled from Netflix's reported total | 2019 | IEA commentary (Kamiya, 2020) | read at source; 5 % x 77 is scout's arithmetic | includes more than delivery |
| 2 vanish | video delivery, world data-centre energy | 3 to 25 (0.7 to 6 % of 416) | TWh/year | AUTHOR'S ESTIMATE; 1 to 3 hours a day assumed | 2024 | derived | scout's arithmetic | viewing hours assumed |
| 2 vanish | data-centre electricity by equipment | servers about 60; storage about 5; network up to 5; cooling 7 (hyperscale) to over 30 (enterprise) | % | AVG; modelled | 2024 | IEA, section 2.1.2 | read | by type only in a chart |
| 2 vanish | recommendation share of AI inference | over 79 (three classes 65) | % of AI inference cycles | operator-reported | 2019 | Gupta et al., HPCA 2020 (arXiv 1906.03109) | read at source (arXiv) | cycles on CPUs, inside AI inference; not energy; not the total |
| 2 vanish | SHARE OF CONSUMER WORK THAT VANISHES | 0.3 to 0.7; centre 0.5 | fraction | AUTHOR'S ESTIMATE | 2024 | derived | scout's estimate | no operator publishes it; overlaps input 4a |
| 3 edge | PUE by type | hyperscale 1.14; colocation 1.53; enterprise 1.92; all 1.41 | ratio | AVG; modelled | 2024 | IEA, Table A.1 | read | |
| 3 edge | utilisation | hyperscale up to 50 %; enterprise under 20 % | % | AVG; IEA statement | 2024 | IEA, section 2.1.2 | read | LBNL model: 45-50 / 21-35 / 11-20 |
| 3 edge | server idle power | about 35 (60 in 2010) | % of rated power | modelled | 2024 | IEA; LBNL (36 % in 2023) | read | |
| 3 edge | data-centre overhead per unit of work | 1.5 (hyperscale); 4.6 (enterprise) | ratio to a server at full load | derived from the three rows above, linear power model | 2024 | derived | scout's arithmetic | a model |
| 3 edge | video on demand from home gateways vs data centre | 0.4 to 0.8 (saves 20-30 % pessimistic, up to 60 %) | ratio edge / data centre | modelled with traces; gateway already on; PUE 1.2 | 2009 | Valancius et al., CoNEXT 2009 | read at source | 2009 data centres; ISP-managed gateways |
| 3 edge | home nano server vs data centre | either wins; depends on access network and idle-to-active time | - | measured device, modelled network | 2016 | Jalali et al., IEEE JSAC 2016 | mirror | data centre priced at 4-20 microjoule/bit |
| 3 edge | data-centre energy per bit, model vs measured | 4 to 20 (2016 model) vs about 0.26 (2020, at an assumed 5 Mbit/s) | microjoule/bit | modelled vs operator-measured | 2016 vs 2020 | Jalali et al.; Carbon Trust | read; conversion is scout's arithmetic | bit rate assumed by scout |
| 3 edge | home gateway idle / maximum | Ethernet 2.8 / 4.6; ADSL2+ 4.1 / 6.7; GPON 5.2 / 8.3; 4G 0.5 / 1.75 | W | measured | about 2014 | Jalali et al., Table IV | mirror | old hardware |
| 3 edge | media delivery, gateway already on | about 0.1 (0.09 vs 1.3 Wh/hour) | ratio | MARG; derived | 2014 / 2020 | derived | scout's arithmetic | excludes encoding; needs upstream capacity |
| 3 edge | media delivery, gateway's idle charged | 2 to 4 (2.8 to 5.2 vs 1.3 Wh/hour) | ratio | AVG; derived | same | derived | scout's arithmetic | one stream per gateway |
| 3 edge | peer-to-peer vs content-delivery network | provider's power falls, total energy rises | - | modelled | 2010 | Feldmann et al., CTTE 2010 | search summary of the abstract | not read |
| 3 edge | cloud storage vs local disk | cloud more efficient when files are rarely accessed; transport dominates at high access rates | - | modelled | 2010 | Baliga et al., Proc. IEEE 2011 | mirror | 2010 hardware and networks |
| 3 edge | home storage box | 14.69 access; 4.41 hibernating (39 to 129 kWh/year) | W | CAP; vendor specification | current | Synology DS224+ | read at source | drives fitted not stated |
| 3 edge | home storage per TB provisioned | 4.8 to 16 (ratio 0.3 to 1.0 to the data centre's 15.4) | kWh per usable TB-year | derived; 8 TB usable assumed | current | derived | scout's arithmetic | per STORED TB it is 2.5 to 40x |
| 3 edge | small board idle | about 3.1 (27 kWh/year) | W | vendor documentation; with peripherals | 2019 board | Raspberry Pi documentation | read at source | not bare-board |
| 3 edge | server efficiency | 19,500 to 44,168 | ssj_ops per watt | vendor-run benchmark | 2025 | SPECpower_ssj2008 results | search summary | no small-board results exist |
| 3 edge | EFFICIENCY RATIO | already on: 0.1 to 1; dedicated device: 2 to 4 or its fixed kWh | ratio edge / data centre | AUTHOR'S ESTIMATE by kind | - | derived | scout's estimate | general compute not sourced |
| 4a AI share | accelerated servers, world | 15 (24 % of server electricity) | % of data-centre electricity | AVG; modelled proxy | 2024 | IEA, Box 2.1 | read | servers only, no overhead; includes ranking, business AI, science |
| 4a AI share | same with overhead | 17 to 21 | % | derived | 2024 | derived | scout's arithmetic | |
| 4a AI share | growth | accelerated +30 % a year; conventional +9 % | % a year | scenario | 2024-2030 | IEA | read | |
| 4a AI share | implied | 19 (2026); 32 (2030) | % | derived from the IEA's rates | - | derived | scout's arithmetic | not an IEA figure |
| 4a AI share | United States | 23 (over 40 of 176 TWh) | % | modelled | 2023 | LBNL 2024 | read | servers only |
| 4a AI share | Germany | 15 now; about 40 by 2030 | % of capacity | industry study | 2024 | Bitkom / Borderstep | read | AI plus scientific computing |
| 4b AI energy | hosted, median text prompt | 0.24 (accelerators 0.14; host 0.06; idle 0.02; overhead 0.02); narrow boundary 0.10 | Wh/prompt | AVG; operator-measured | May 2025 | Google, arXiv 2508.15734 | read at source (arXiv) | one product; median; no training |
| 4b AI energy | hosted, movement | 33-fold fall | ratio | operator-reported | May 2024 to May 2025 | same | read | |
| 4b AI energy | hosted, other operator | about 0.34 | Wh/prompt | operator statement, no method | June 2025 | OpenAI, as quoted by Google's paper | secondary | no boundary |
| 4b AI energy | open models on a batched GPU | 0.15 to 0.21 J/token (32 bn parameters); 0.026 to 0.05 Wh per 627-token reply; 1.3 Wh with reasoning | J/token; Wh | measured | 2025-2026 | Chung et al., arXiv 2601.22076 | read at source (arXiv) | no PUE or idle capacity |
| 4b AI energy | laboratory, by model size | 0.1 (1.7 bn); 0.3 (9 bn); 4 to 5 (70 bn); image 1.7; video 115 | Wh/task | measured, GPU only, unbatched | 2025 | IEA, section 1.4 | read | "highly controlled experimental results" |
| 4b AI energy | open models, single GPU | 0.047 text; 2.9 image | Wh/inference | measured | 2023 | Luccioni et al., FAccT 2024 | read at source (arXiv) | unbatched; small models |
| 4b AI energy | phone | 0.58 to 0.76 (0.16 to 0.21 mWh) | J/token | measured at the battery | 2022-2023 phones | Laskaridis et al., MobiCom 2024 (arXiv 2403.12844) | read at source (arXiv) | 3 bn parameters, 4-bit; batch of one |
| 4b AI energy | laptop | 0.09 to 0.25 | J/token | measured speed, ASSUMED 10 W system power | 2025-2026 | Kannan et al., arXiv 2608.28667 | read (preprint) | power assumed; its "30-40x" headline is not usable |
| 4b AI energy | small board | 2.61 | J/token | measured, as cited | - | cited in the above | not confirmed | 0.5 bn parameters |
| 4b AI energy | DEVICE / DATA CENTRE, same model | 3 to 30 (laptop 3 to 10; phone 10 to 30) | ratio | AUTHOR'S ESTIMATE from measured endpoints | 2024-2026 | derived | scout's estimate | no like-for-like measurement exists |
| 4b AI energy | DEVICE / DATA CENTRE, as served | 0.3 to 5 | ratio | derived | 2024-2026 | derived | scout's arithmetic | a small model against a large one |

## (b) The four inputs: sourced range beside the guess

| Input | Team's guess (range) | What can be sourced | Scout's range | Move? |
|---|---|---|---|---|
| 1. Share that stays | 0.4 (0.2 to 0.6) | By owner type, yes (IEA). By consumer versus business, no. | 0.45 to 0.75, centre 0.6 | **Up.** Below 0.35 is not defensible. |
| 2. Vanishes, of consumer work | 0.5 (0.2 to 0.8) | Bytes by category, the data-centre cost of video, one 2019 ratio for recommendation. Not the share. | 0.3 to 0.7, centre 0.5 | **Stays.** Note video relocates but is only a few per cent of the estate; carve AI out first to avoid counting ranking twice. |
| 3. Edge-to-data-centre ratio | 1.0 (0.3 to 3) | Data-centre overhead, yes. Edge comparisons, only 2009-2016 models against obsolete data centres. | already on: 0.1 to 1; device added: 2 to 4, or its fixed draw (27 to 129 kWh a year) | **Replace the single ratio with two terms.** If one number must be kept, widen to 0.1 to 4. |
| 4a. AI share | 0.15 (0.05 to 0.30) | Yes, as a proxy with a year. | 0.15 (2024, servers only) to 0.25 (2026, with overhead) | **Up with the scenario year;** raise the lower bound to 0.15. |
| 4b. Device cost of inference | 3 times (1 to 10) | Per-prompt and per-token on both sides, never the same model on both. | same model: 3 to 30; as served: 0.3 to 5 | **Split in two.** Keep 3 for a laptop-class device and the same model; phones are worse. Add a "cannot relocate" share for frontier models and training. |

One consequence, on the scout's centres `[my arithmetic]`: 72 kWh per user-year x (1 - 0.6) = about 29 kWh of consumer work (18 to 40). The earlier arithmetic in `TWIN_THEORY_INPUTS.md` set 75 kWh against 0 to 17 added. The comparison that the sourced inputs support is nearer 29 against the same 0 to 17, before deciding how much of the 29 vanishes: still the same sign on the centres, but no longer "not close" if every home adds a dedicated box (17 kWh a person for a 5 W board; 15 to 51 for a storage box).

## (c) Trust

Most trusted:
1. **ITU: 5.5 billion as published, 5.8 billion as revised, 6.0 billion for 2025.** Both editions read at source; the revision is itself the finding.
2. **IEA Table A.1: 416 TWh split 166 / 144 / 106 by owner type, with PUE and load factor.** Read at source. It is a model, but it is the only published split of energy by type and it reconciles internally.
3. **Google's 0.24 Wh per median text prompt with its four components.** Operator-measured in production, boundary stated, read at source. One product and a median, and it fell 33-fold in a year.

Least trusted:
1. **The consumer share of workloads (27 %, Cisco).** A 2018 forecast of a count, seen only in a search summary of a third-party copy of a withdrawn paper. I list it because nothing else of its kind exists.
2. **My own split of hyperscale and colocation into consumer and business,** on which the stay share rests. Only two operator totals under it are sourced.
3. **Any single edge-to-data-centre ratio, mine included.** The published ones are 2009-2016 models; the Apple-laptop preprint's "30 to 40 times better" rests on an assumed wattage and an unbatched comparator and should not be quoted.

## (d) For a human to open

- **Uptime Institute Global Data Center Survey 2025** (and the 2026 results page it links): behind a form. Wanted: where enterprise workloads run (on-premises / colocation / public cloud), average PUE and respondents.
- **JRC, "Energy Consumption in Data Centres and Broadband Communication Networks in the EU"** (JRC135926): the repository refused a scripted read. Wanted: any EU split by data-centre type.
- **Cisco Global Cloud Index 2016-2021 white paper**: withdrawn from cisco.com; a copy is on a document-sharing site that needs an account. Wanted: the consumer and enterprise workload table, to confirm 27 / 73 and the consumer sub-shares.
- **IEA, Energy and AI, Figures 2.2, 2.3 and 2.5**: the equipment split by data-centre type and the range of AI-share estimates exist only as charts in the PDF. The IEA's report pages offer the chart data for download; that is a dataset download and was not done.
- **LBNL 2024, Figures 5.2 and 5.7**: energy by space type, in charts only.
- **Microsoft and Amazon sustainability reports**: total electricity not found at source; needed for the operator derivation of Input 1.
- **Netflix 2024 Environmental Social Governance Report** (PDF found, not read): a current total to replace the 2019 figure.
- **Synology knowledge-base article on how power is measured**: which drives were fitted for the 14.69 W and 4.41 W figures.
- **spec.org SPECpower_ssj2008 results**: the figures here are from search summaries of result pages.
- **Feldmann et al. 2010** (author's copy at gsmaragd.github.io/publications/CTTE2010/CTTE2010.pdf) and **Nedevschi et al., NSDI 2008**: named in the brief; the first seen only as an abstract summary, the second not retrieved.
- **MLPerf Mobile and MLPerf Client power results**, and the analyst notes the IEA cites for the AI share (Goldman Sachs, Deloitte, Gartner, Schneider Electric, SemiAnalysis): not opened.
- **Sandvine's sample**: neither edition states which networks, in which countries, its shares are drawn from.
