# SCOUT: the data-centre estate under full replacement — what stays, what vanishes, what relocates, and at what efficiency

Scouted 2026-11-11. Documentation only: agency and national-lab reports, operator papers, academic papers, vendor specification pages, one commercial analyst's public press releases. No dataset was downloaded, no value-returning data API was called, nothing was signed up for. Report and paper PDFs were opened and read as documents. Companion to `conformance/scene/SCOUT_UPSTREAM.md` and `conformance/scene/SCOUT_DEVICES_POWER.md`; figures already in those are cited by name and not re-derived.

Marking convention (same as the earlier reports). Unmarked statements were read in the source document itself at the URL beside them. `[mirror]` = read in full from a copy not hosted by the publisher. `[search summary]` = seen only in a search engine's summary. `[my reading]` = my inference or recollection. `[my arithmetic]` = computed by me from figures read. `[not confirmed]` = could not open the source. `[author's estimate]` = a bracket I built; no source states it.

Two cautions, carried beside every figure:

- **Kind of figure.** AVG = long-run average allocation. MARG = marginal. CAP = capacity or provisioning. In a FULL-REPLACEMENT thought experiment the long-run average is the right kind for the BASELINE (the estate is not built at all). It is not automatically the right kind for the EDGE side: there the question is marginal draw on devices already on, plus the whole draw of anything added.
- **A constant is a price in a regime.** Year, system, and rate of movement are given. Two figures in this report moved by more than an order of magnitude inside the period the literature covers (data-centre energy per delivered bit: 77x between a 2016 model and a 2020 measurement; hosted energy per AI prompt: 33x in twelve months at one operator).

**AMENDMENT, 2026-11-11, same day: item 4 of the brief was re-pointed twice by the lead.** AI inference in the scenario is a MIX across tiers (phone, prosumer machine, office appliance, on-premises server, hosted model with zero retention) and "the market optimizes". The amended item 4 is at the end of this file under "AMENDMENT: item 4, AI as a mix across tiers". It supersedes section 4b, headline finding 7 and the 4b row of table (b), and it carries one dated CORRECTION of a statement in 4b. Sections 0 to 3 and 4a stand.

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

---

# AMENDMENT: item 4, AI as a mix across tiers

Added 2026-11-11 after two messages from the lead. The scenario's AI is "a mixture of on-device, edge, prosumer, office appliance, servers, etc... - the market optimizes". Same marking convention and cautions as above.

## CORRECTION (dated 2026-11-11, not a quiet fix)

Section 4b says of the same-model comparison between a device and a data centre: "Nobody has published this comparison like for like; it is my estimate from measured endpoints", and gives "laptop 3 to 10". **Both are wrong.** The comparison has been published (Saad-Falcon et al., below), and it gives 1.6 to 2.3 with neither side batching and, once the data centre batches, about 18 to 46 `[my arithmetic on their two factors]`. My "3 to 10" for a laptop was too low at the top and missed that the answer is set by batching, not by the chip. The phone figure (10 to 30) remains my estimate; no like-for-like phone measurement was found.

## Amended headline findings (replace finding 7)

A. **Where a model runs matters less than how big it is and how full the machine is.** Same model, laptop against data-centre accelerator, one request at a time: the data centre is 1.6 to 2.3 times better per joule. Let the data centre batch 64 requests and it gains another 11 to 20 times. A machine that serves one person or one office cannot batch across users, wherever it sits. So an office appliance or on-premises server at low load is nearer the laptop's efficiency than the hosted model's.
B. **The saving from a mix comes from sending easy work to small models, not from moving it to the edge.** In the one published simulation, routing 80 % of queries to small local models cuts energy 64 to 80 % against sending everything to a frontier model, and the authors state "The dominant effect is the reduction in model size, not a hardware efficiency advantage."
C. **No one has measured the share of inference run on devices.** What exists is capability (local models answer 71 to 89 % of single-turn chat and reasoning queries correctly, up from 23 % in 2023) and analyst counts of capable hardware shipped (about a third of phones and of PCs in 2025).
D. **Inference people explicitly ask for is a small part of AI electricity today: of the order of 5 to 20 % `[author's estimate]`.** One operator's whole chat product comes to about 0.3 TWh a year on its own per-prompt figure, against 62 TWh for accelerated servers worldwide in 2024. The often-quoted "inference is 60 to 90 %" figures count ranking and recommendation inference and predate generative AI.
E. **Agents multiply tokens per task by about 4 (one operator's statement), multi-agent systems by about 15; a coding agent multiplies tokens per person per day by far more.** The team's 1, 2 and 5 covers the single-agent case only. Energy rises less than tokens, because agent traffic is mostly input.
F. **"Zero data retention" in the providers' own terms removes stored prompts and responses. It does not remove metadata, abuse-screening exceptions or in-memory caching, and hosted API traffic is already not trained on by default.** So for the hosted tier it removes almost no energy; what vanishes is the platforms' own ranking, advertising and training on harvested data (finding D's remainder).

## A1. Energy per unit of inference, by tier

| Tier | Hardware, year | Model it ran | Value | Scope of the measurement | Device already on? | Basis | Source, how read |
|---|---|---|---|---|---|---|---|
| Phone | Galaxy S23, iPhone 14 Pro; 2022-2023; phone GPU, **not the neural processor** | 3 billion parameters, 4-bit | 0.58 to 0.76 J/token (0.16 to 0.21 mWh); 490 to 590 prompts per battery | whole phone at the battery, 8.5 to 13.8 W sustained | yes, in use; the figure is the phone's whole draw while generating, not the increment | measured | Laskaridis et al., MobiCom 2024, arXiv 2403.12844, read |
| Phone, ceiling | 2024 flagships | 10 billion at 15 tokens/s; 3 billion at 30 to 45 | no energy figure | - | - | vendor claims | IEA 2025, Box 2.3, read |
| Phone or laptop neural processor | 2023-2024 | small vision and image models | 0.5 W (phone), 1 to 5 W (laptop) | chip power, not energy per task | - | early studies | IEA 2025, section 2.4.1, read |
| Small board | Raspberry Pi 4 | 0.5 billion | 2.61 J/token | not stated | - | measured, as cited | cited in Kannan et al.; original `[not confirmed]` |
| Laptop (prosumer) | Apple M4 Pro, 2025-2026 | 3 to 9 billion, 4-bit | 0.09 to 0.25 J/token | **system power ASSUMED 10 W**; measured chip package 0.47 W | assumed on | measured speed, assumed power | Kannan et al., arXiv 2608.28667 (preprint), read |
| Laptop (prosumer), same model as cloud | Apple M4 Max, 2025 | 8 to 120 billion | 1.6 to 2.3 times WORSE per joule than a data-centre accelerator (B200) on the same models, both at batch 1; 6.5 to 7.4 times worse than a purpose-built inference chip | accelerator power only on both sides | - | measured, 3 to 5 runs per cell | Saad-Falcon et al. (Stanford, Together AI), "Intelligence per Watt", arXiv 2511.07885, November 2025, read |
| Desktop, consumer graphics card | RTX 4060 Ti 16 GB, 2026 | 1 billion: 0.27 to 0.32 J/token; 7 billion: up to 4.4 times that `[search summary for the multiple]` | graphics card only (nvidia-smi), batch 1 | not counted: the rest of the PC | measured | Zähl and Hennig, arXiv 2608.00008 (preprint), abstract read |
| Desktop, older cards | two RTX 3090, 2023 | 13 billion | 12 to 19 J/token | cards only | - | one person's blog | `[search summary]`; do not use |
| Office or home appliance | workstation card (RTX 6000 Ada), mini-PC, small GPU box | - | **no published joules per token found.** Vendor data points only: a 25 W module "capable of 67 trillion operations per second" | - | - | vendor claim via the IEA | IEA 2025, Box 2.3, read |
| On-premises or hosted accelerator, one request at a time | H100-class | 7 billion | about 4 J/token | an estimate (400 W / 100 tokens a second), not a measurement | - | author's estimate in the source | Kannan et al., read |
| Hosted accelerator, batched | B200 node, vLLM, batch 128 to 512; 2025-2026 | 32 billion | 0.15 to 0.21 J/token; 95 J (0.026 Wh) per 627-token reply; with reasoning 0.31 J/token and 2,192 J per reply | steady-state energy of the GPU node; no cooling, no idle capacity | - | measured | Chung et al., arXiv 2601.22076, read |
| Effect of batching alone | B200, batch 64 against batch 1 | same models | 11 to 20 times more work per joule | accelerator | - | measured | Saad-Falcon et al., App. E.9, read |
| Hosted frontier model, in production | Google fleet, May 2025 | not disclosed | 0.24 Wh per median text prompt (accelerators 0.14, host 0.06, idle capacity 0.02, overhead 0.02) | everything inside the data centre | - | operator-measured | Elsworth et al., arXiv 2508.15734, read |
| MLPerf Inference and MLPerf Client power | - | - | not retrieved | - | - | - | `[not confirmed]` |

**Where sizes are not comparable, and how the literature handles it.** Three ways, and they give different answers:
1. *Same model, different hardware* (Saad-Falcon et al.): the data centre wins by 1.6 to 2.3 unbatched and by roughly 18 to 46 batched `[my arithmetic: 1.6-2.3 x 11-20; accelerator scope; multiply the data-centre side by about 1.7 for Google's measured overheads, leaving 10 to 27]`. This is the only like-for-like comparison found, and it is for a laptop, not a phone.
2. *Same model, lower precision*: phone and laptop figures are all for 4-bit versions; MELT reports "a non-negligible accuracy cost". The hosted figures are for full or 16-bit precision. No source prices the accuracy difference in energy.
3. *Smaller model for an easier task*: the IEA ("smaller and more efficient versions of their cloud-based counterparts ... trade-offs between efficiency and accuracy") and Saad-Falcon et al. (accuracy per joule, with a judged right-or-wrong for each query). This is the honest form for a market that "optimizes": the unit is a correctly answered query, not a token.

**The tiers in one line each `[my reading of the table]`:** phone, about 0.6 to 0.8 J/token for a 3-billion model, cannot run more than about 10 billion; laptop, about 0.1 to 0.25 J/token for 3 to 9 billion, can run up to about 120 billion slowly; consumer graphics card, about 0.3 to 1.4 J/token for 1 to 7 billion (card only); hosted and batched, about 0.15 to 0.2 J/token for a model four to ten times larger. Per token the tiers are within one order of magnitude of each other only because the small tiers run small models.

**What is missing for the office-appliance and on-premises tiers:** any measured figure, and above all idle draw. A hosted accelerator's advantage is that it is busy. A box serving one household or one office is idle most of the day; its energy is then its idle watts times 8,760 hours, the same structure as the storage box in section 3.3. No source gave the idle draw of a single-GPU inference box.

## A2. How the mix stands and moves

| Evidence | Value | Kind | Year | Source, how read | What it is NOT |
|---|---|---|---|---|---|
| Where AI energy is today | "Most AI-related energy demand currently comes from large, centralised cloud and hyperscale data centres – both for training and inference"; "Some inference tasks are already conducted on user devices, as well as hybrid approaches where initial processing is done on the device and the final request is sent to a data centre" | agency statement | 2025 | IEA 2025, section 2.4.1, read | no share given |
| Share of queries a local model answers correctly | 88.7 % with the best local model per query; 71.3 % with one model on one laptop | measured on 1 million real single-turn chat and reasoning queries, 20+ models, 8 accelerators | 2025 | Saad-Falcon et al., read | single-turn; capability, not use. Their multi-turn agent tests are in an appendix I did not read |
| Its movement | 23.2 % (2023), 48.7 % (2024), 71.3 % (2025); accuracy per watt up 5.3 times in two years (3.1 from models, 1.7 from hardware) | measured | 2023-2025 | same | |
| What routing saves | all queries to a 235-billion model in the cloud (batch 16) against routing: perfect router -80.4 % energy; 80 %-accurate router -64.3 %; 60 %-accurate -48.4 %. 80.7 % of queries go local | simulation over 80.2 million queries in 24 hours | 2025 | same | "The dominant effect is the reduction in model size, not a hardware efficiency advantage": the same routing to small models INSIDE a data centre would save more |
| Phones able to run generative models | about 30 % of smartphone sales in the first half of 2025 (15 % a year before); about one in three shipments in 2025 | ANALYST estimate (Counterpoint) | 2025 | `[search summary]` | capable hardware sold, not inference run on it |
| "AI PCs" | 31 % of the PC market in 2025 (15.6 % in 2024), about 78 million units; 55 % FORECAST for 2026. The same firm's 2024 forecast for 2025 was 43 % | ANALYST estimate and forecast (Gartner) | 2025 | `[search summary]` | defined by the presence of a neural processor; a forecast that already missed once |
| Device replacement | the IEA warns on-device AI "could accelerate device replacement cycles" (average smartphone replacement 3.5 years) | agency statement | 2025 | IEA 2025, read | embodied energy, outside this report |
| Share of inference run on devices | **not found, measured or estimated, in any source I would cite** | - | - | - | one blog forecast (60 % by 2029) is opinion |

**What routes where:** the only evidence is the simulation above (easy single-turn queries local, the hard fifth hosted) and the IEA's list of reasons to run locally (latency, poor connectivity, large local data such as video, privacy). No usage study of what people actually run locally was found.

**Direction:** capability on devices is rising fast (threefold in two years on the one measured series) and capable hardware is about a third of new sales. Whether use follows is not measured.

## A3. Of AI energy, inference people ask for against the rest

No operator publishes this split. What can be said:

- **The often-quoted inference shares, with their sources and years.** Google: inference 3/5, training 2/5 of ML energy (2019-2021; in `SCOUT_UPSTREAM.md`). Meta: 70 % of AI power capacity is inference (2021; same). "Inference is 80 to 90 %" and "90 percent of the cost" trace to vendor and cloud-provider statements of about 2019 and are about cost or compute, not energy `[search summary; originals not read]`. LBNL (2024, read): training "has dominated" to date and inference "could dramatically exceed training"; its model takes training servers as busy 80 % of the time and inference servers 40 %. **All of these predate generative AI at scale or model it, and in all of them "inference" is mostly ranking, recommendation, search and advertising inference, not requests a person typed.** At Meta in 2019 recommendation was over 79 % of AI inference cycles (section 2.4).
- **Meta, fourth-quarter 2025 call** (28 January 2026; transcript read at https://s21.q4cdn.com/399680738/files/doc_financials/2025/q4/META-Q4-2025-Earnings-Call-Transcript.pdf): "demands for compute resources across the company have increased even faster than our supply"; the advertising retrieval and ranking models are being scaled up in "data, complexity and compute". No share of compute by use is stated. Earlier calls describe hardware bought for generative training as reusable for ranking and recommendation `[search summary]`.
- **A bound from the operators' own per-prompt figures `[my arithmetic]`.** 2.5 billion prompts a day `[search summary: OpenAI to the press, July 2025]` at 0.34 Wh is 0.85 GWh a day, **0.31 TWh a year**. Google's stated 1.3 quadrillion tokens a month across all its products `[search summary, October 2025]` at 0.15 to 0.4 J per token and 1.7 for overheads is about 1 to 3 TWh a year, against Google's 32 TWh total. Against 62 TWh for accelerated servers worldwide in 2024, requested generative inference across all operators is plausibly 3 to 12 TWh: **5 to 20 % of AI electricity** `[author's estimate]`. Weaknesses: the per-prompt figures are a median and an unexplained average for text; images, video, reasoning and coding agents cost far more per request; the token figure mixes input and output; the years do not line up.
- **So the remainder, 80 to 95 %, is training and experimentation, ranking and advertising models, business and scientific use, and capacity held idle.** Of that remainder, what VANISHES in the scenario is ranking and advertising and training on retained user data; frontier-model training on public data and business AI do not. No source separates them. Bracket for the share of AI electricity that vanishes: **0.2 to 0.6** `[author's estimate]`: the low end if most accelerated capacity is now generative training and business use, the high end if consumer platforms' ranking still dominates as it did before 2023.
- Kind: AVG. Rate: this is the fastest-moving split in the report; requested inference grew several-fold in 2025 on the operators' own counts.

## A4. AI's share of data-centre electricity, by source and definition

Section 4a stands. The definitions side by side:

| Source | Definition of "AI" | Share | Year |
|---|---|---|---|
| IEA 2025 | electricity of accelerated servers, as a proxy; no cooling overhead | 15 % of data-centre electricity (24 % of server electricity) | 2024, world |
| IEA 2025, Base Case | same | grows 30 % a year against 15 % for the total: about 17 % (2025), 19 % (2026), 32 % (2030) `[my arithmetic]` | scenario |
| LBNL 2024 | "AI servers" (accelerated, plus non-accelerated servers used for AI) | over 40 of 176 TWh, 23 %, servers only | 2023, US |
| Google (Patterson et al.) | all machine learning, training and inference | 10 to 15 % of the company's energy | 2019-2021 |
| Bitkom / Borderstep | AI and high-performance computing | 15 % of capacity; about 40 % by 2030 | 2024, Germany |
| Analysts (Goldman Sachs and others) | various, mostly generative | named by the IEA as the spread of its Figure 2.5; not read | 2024 |

None is "generative only" and none is "consumer only".

## A5. Demand per person with agents, and what "zero data retention" means

**Demand:**

| Evidence | Value | Kind | Year | Source, how read | What it is NOT |
|---|---|---|---|---|---|
| Tokens, agent against chat | "agents typically use about 4× more tokens than chat interactions, and multi-agent systems use about 15× more tokens than chats" | operator statement | June 2025 | Anthropic engineering post, https://www.anthropic.com/engineering/multi-agent-research-system , read | one operator, per task, research tasks; not per person per day |
| Tokens per request, a router's whole traffic | average prompt "roughly fourfold from around 1.5K to over 6K"; completions "from about 150 to 400 tokens"; programming requests "routinely exceed 20K input tokens" and are 3 to 4 times general-purpose prompts | measured from request metadata over 100 trillion tokens | early 2024 to late 2025 | Aubakirova et al. (OpenRouter and a16z), arXiv 2601.10088, read | per request, not per person; a router's users are developers |
| Chat baseline | about 3 prompts per weekly user per day (2.5 billion a day over 800 million) | `[my arithmetic on search summaries]` | mid-2025 | press reports of operator statements | users and prompts are from different months |
| A coding agent, per person | "around $13 per developer per active day ... below $30 per active day for 90% of users"; suggested rate limits 10,000 to 300,000 tokens per minute per user by team size | operator documentation; dollars | current | https://code.claude.com/docs/en/costs , read | dollars, not tokens; enterprise deployments; no method |
| One operator's token throughput | 480 trillion a month (May 2025), 980 trillion (July), 1.3 quadrillion (October) | operator statements | 2025 | `[search summary]` | all products, input plus output |

- **What supports a value of the team's multiple:** 1 is not supported for agent use by anything found. **4 is the one stated typical multiple per task** (single agent), and the router's traffic independently shows a fourfold rise in tokens per request as agent use spread. 15 is stated for multi-agent systems. So 1, 2 and 5 brackets a single agent; add 15.
- **Per person per day the multiple can be far larger** than per task, because an agent runs many tasks unattended. A chat user at 3 prompts a day uses a few thousand tokens; $13 a day of a coding agent is millions of tokens at any current price `[my arithmetic; prices assumed]`. That is a developer, not a household, and nothing found measures a household's agent.
- **Energy rises less than tokens.** Agent traffic is input-heavy (6,000 in to 400 out on the router), and input tokens cost much less energy than generated ones (decoding dominates; Chung et al.; and repeated context is cached). No source gives the energy multiple; it is below the token multiple by an unsourced factor.

**"Zero data retention" in the providers' own terms** (documentation pages read 2026-11-11):

| Provider | What it means | What is still kept or excepted | Training |
|---|---|---|---|
| OpenRouter (router), https://openrouter.ai/docs/features/zdr | "no routine storage of prompts or responses after processing, apart from the CSAM exception" | "in-memory caching of prompts is *not* considered 'retaining' data"; a separate "Safety Retention" tier may keep prompts and responses to investigate abuse; what the router itself logs as metadata is not stated on the page | providers that do not retain "are also unable to train on your data" |
| Anthropic, https://platform.claude.com/docs/en/build-with-claude/zero-data-retention and its privacy article of 1 July 2026 | "does not store customer prompts or responses at rest after the API response is returned"; without it, inputs and outputs are deleted "within 30 days" | flagged content up to 2 years and safety classification scores up to 7 years; batch, files and several agent features are not eligible; some model families require 30-day retention `[this row partly from a summary of the page's tail]` | "never used for model training without your express permission" |
| OpenAI, https://developers.openai.com/api/docs/guides/your-data | "excludes customer content from abuse monitoring logs"; without it, such logs are "retained for up to 30 days" | stateful endpoints (files, threads, batches, fine-tuning) keep data "Until deleted"; flagged images kept for review; a safety exception can suspend it | "As of March 1, 2023, data sent to the OpenAI API is not used to train or improve OpenAI models (unless you explicitly opt in...)" |

- **What vanishes under these terms:** stored prompts and responses, and with them any later training or profiling on that content.
- **What does not:** metadata. The router's own usage study was built on "timing, model and provider identifiers, token usage, geographic routing, and usage context" without touching content. Abuse and child-safety exceptions. Transient caching.
- **Consequence for energy `[my reading]`:** retention of text is a negligible storage load, and hosted API traffic is already not trained on by default. So applying zero retention to the hosted tier removes close to no data-centre energy by itself. The energy that vanishes in the scenario is the consumer platforms' own (A3), not the hosted model tier's.

## Rows to add to table (a) (they replace the "4b AI energy" rows where the sub-component matches)

| input | sub-component | value or range | unit | basis | year | source | how read | what it is not |
|---|---|---|---|---|---|---|---|---|
| 4 tier | phone, 3 bn model, 4-bit | 0.58 to 0.76 | J/token | measured, whole phone | 2022-2023 | Laskaridis et al., arXiv 2403.12844 | read | phone GPU not neural processor; total draw, not increment |
| 4 tier | laptop, 3-9 bn, 4-bit | 0.09 to 0.25 | J/token | speed measured, 10 W ASSUMED | 2025-2026 | Kannan et al., arXiv 2608.28667 | read (preprint) | power assumed |
| 4 tier | laptop vs data-centre accelerator, same model, batch 1 | 1.6 to 2.3 (6.5 to 7.4 against a purpose-built chip) | ratio, laptop worse | measured, accelerator scope | 2025 | Saad-Falcon et al., arXiv 2511.07885 | read | no cooling or idle on the data-centre side; laptop, not phone |
| 4 tier | batching gain in the data centre | 11 to 20 | ratio, batch 64 vs 1 | measured | 2025 | same, App. E.9 | read | needs many concurrent users |
| 4 tier | laptop vs batched data centre, same model | 18 to 46 (10 to 27 after data-centre overheads) | ratio | derived | 2025 | derived | scout's arithmetic | product of two measured factors |
| 4 tier | desktop graphics card, 1 bn model | 0.27 to 0.32 | J/token | measured, card only, batch 1 | 2026 | Zähl and Hennig, arXiv 2608.00008 | abstract read (preprint) | excludes the rest of the PC |
| 4 tier | office appliance / on-premises single GPU | not found | - | - | - | - | - | idle draw is the missing number |
| 4 tier | hosted, batched, 32 bn open model | 0.15 to 0.21 | J/token | measured, GPU node | 2025-2026 | Chung et al., arXiv 2601.22076 | read | no cooling or idle capacity |
| 4 tier | hosted frontier, production | 0.24 | Wh per median text prompt | operator-measured, full boundary | May 2025 | Google, arXiv 2508.15734 | read | fell 33-fold in a year |
| 4 mix | queries a local model answers correctly | 71.3 (one model) to 88.7 (best local per query); 23.2 in 2023 | % of single-turn queries | measured, 1 M queries | 2025 | Saad-Falcon et al. | read | capability, not use |
| 4 mix | energy saved by routing easy queries to small local models | 48 to 80 | % vs all-to-frontier | simulation | 2025 | same | read | saving is from model size, not location |
| 4 mix | capable phones; "AI PCs" | about 30 % of phone sales; 31 % of PCs (55 % forecast 2026) | % | ANALYST | 2025 | Counterpoint; Gartner | search summary | hardware sold, not inference run |
| 4 mix | share of inference run on devices | not found | - | - | - | - | - | - |
| 4 split | requested generative inference, of AI electricity | 5 to 20 | % | AUTHOR'S ESTIMATE from operator per-prompt and volume figures | 2024-2025 | derived | scout's arithmetic | mismatched years; text prompts only |
| 4 split | AI electricity that vanishes (ranking, ads, training on retained data) | 0.2 to 0.6 | fraction of AI electricity | AUTHOR'S ESTIMATE | 2024-2025 | derived | scout's estimate | no source separates it |
| 4 demand | tokens, agent vs chat, per task | about 4 (multi-agent about 15) | ratio | operator statement | June 2025 | Anthropic engineering post | read at source | one operator; per task |
| 4 demand | tokens per request, router traffic | prompt 1.5K to over 6K; completion 150 to 400 | tokens | measured, metadata | 2024 to 2025 | OpenRouter study, arXiv 2601.10088 | read | developers' traffic |
| 4 demand | coding agent per developer | about 13 (under 30 for 90 %) | USD per active day | operator documentation | current | Claude Code documentation | read at source | dollars; not households |
| 4 retention | zero data retention | prompts and responses not stored after processing; metadata, safety exceptions and in-memory caching remain; API data not trained on by default | - | provider documentation | 2026 | OpenRouter, Anthropic, OpenAI | read at source (one page's tail by summary) | removes close to no energy on the hosted tier |

## Item 4 in table (b), replaced

| Input | Team's guess | Scout's range | Move? |
|---|---|---|---|
| 4b, as it was (device costs 3 times) | 3 (1 to 10) | same model: 2 unbatched, 18 to 46 against a batched data centre (laptop, measured factors); phone not measured like for like | **Drop the single multiple.** Carry per-tier J/token with model size, and a batching or utilisation term for every tier that serves few users. |
| Mix across tiers | not yet an input | up to about 80 % of single-turn queries can be served locally (capability); actual share unmeasured | carry as a swept input, 0 to 0.8 local; the saving it yields is from model size |
| Share of AI electricity that vanishes | not yet an input | 0.2 to 0.6 `[author's estimate]` | carry as a range; unsourceable |
| Agent demand multiple | 1, 2, 5 | 4 per task (stated), 15 multi-agent; per person unmeasured and possibly much higher | **add 15; treat 1 as "no agents"**; energy multiple is below the token multiple |

## Trust, for the amendment

Most trusted: the same-model laptop-against-accelerator ratios and the batching gain (Saad-Falcon et al.: measured, repeated, scope stated); the providers' retention terms (their own pages); the 4-times and 15-times token statement as a statement (it is one operator's and unexplained).
Least trusted: my 5 to 20 % for requested inference (mismatched years, text prompts only) and my 0.2 to 0.6 for what vanishes; every analyst shipment share (search summaries); the consumer-graphics-card figures (card-only scope, preprints, and two versions of one abstract that disagree).

## For a human to open, for the amendment

- **MLPerf Inference and MLPerf Client power results** (mlcommons.org): the standard like-for-like source for the tiers; not retrieved.
- **Saad-Falcon et al., appendices E.8, E.9 and E.12**: absolute joules per query by accelerator, and the multi-turn agent results; I read the summary statements, not every table.
- **A measured idle and loaded draw for a single-GPU inference box** (a workstation card, a mini-PC cluster): nothing found; this decides the office-appliance tier.
- **Meta's 2024 to 2026 call transcripts** (investor.atmeta.com) for any statement of compute share by use; one transcript read, none found.
- **The original 2019 "80 to 90 % inference" statements** (a chip vendor's and a cloud provider's): seen only second-hand.
- **Counterpoint and Gartner releases** on capable phones and PCs: search summaries only.
- **OpenRouter's privacy and logging page**: what the router itself stores as metadata is not on the page read.

---

# 4c. Inference by tier

Added 2026-11-12 at the lead's request. Same rules and marking as the rest of the file. Parts 1, 3 and 4 of the request were answered in the AMENDMENT directly above (sections A1, A2 and A5), written after the lead's two follow-up messages; they are indexed here, not repeated. What is new in 4c: idle draw by tier (part 2), the two tiers the amendment left empty, what MLPerf does and does not publish, and the closing table with a trust rating per row.

## 4c.1 Energy per unit, by tier (index to A1, plus additions)

All per-token figures are in A1 with hardware, year, model size and scope. Additions:

- **MLPerf Client** (mlcommons.org/benchmarks/client, read): reports time to first token and tokens per second only. Power is not a result; the page mentions only experimental "power efficiency testing tools". Models: Llama 3.1 8B, Phi 4 Mini, Phi 4 Reasoning 14B, Qwen 3 8B, all "int4-quantized"; latest version 2.0. **So the standard client benchmark fixes the "largest useful model" for a PC at about 8 to 14 billion parameters at 4-bit, and gives no joules.**
- **MLPerf Inference, datacenter and edge** (mlcommons.org/benchmarks/inference-datacenter, read): power is an optional add-on, "the measured average AC power (energy) consumed by the entire system", at the wall, reported as system power or energy per stream. Latest round on the page: v6.1 (2026). In the v5.1 round (September 2025) only two submissions carried power: one data-centre (Lenovo), one edge `[search summary]`. The values sit in an interactive results table that is a dataset, not opened. **MLPerf is the right like-for-like source and almost nobody submits power to it.**
- **Small inference appliance** (NVIDIA DGX Spark product page, read): 240 W power supply, chip rated 140 W, 128 GB unified memory, "inference with AI models up to 200 billion parameters" (quantisation not stated; 400 billion on two units, 700 billion on four). Vendor claim, CAP. No tokens per second, no joules, no idle figure on the page.
- **On-premises GPU server** (NVIDIA DGX H100 user guide, read): eight H100 accelerators; input "10.2 kW max."; 38,557 BTU/hr. CAP. No idle figure on the page.
- **Prosumer workstation, one high-end consumer card:** no measured joules per token found for a current top card. The only measured consumer-card figures are for a mid-range card (0.27 to 0.32 J/token on 1-billion models, card only; A1). A1's dual-card blog figure should not be used.

"Already on?" by tier, as the sources treat it: the phone figure is the whole phone's draw while generating (it is on and in the hand); the laptop figures assume the machine on and count either the chip (0.47 W) or an assumed 10 W system; the consumer-card figures count the card alone and ignore the PC around it; hosted figures count the accelerator node, and only Google's adds idle reserve and cooling. **No source counts a home or office box's idle hours against its inference.**

## 4c.2 Idle draw when not inferring

| Tier | Idle draw | What it measures | Basis | Year | Source, how read | What it is NOT |
|---|---|---|---|---|---|---|
| Phone | 11.4 mW screen off (2012 handset); order of 10 to 50 mW today `[my reading]` | radio idle, screen off | measured (old) | 2012 | Huang et al. via `SCOUT_DEVICES_POWER.md` | a 2012 handset; the phone is on regardless, so its idle is not chargeable to the mesh |
| Small desktop used as a node | 4 W (M4, 2024), 5 W (M4 Pro), 7 W (M2 and M2 Pro, 2023); maximum 65 to 140 W | "power used with only Finder open, using the default power management settings", "measured from the wall power source and includes all power supply and system losses" | vendor-measured, at the wall | 2023-2024 | Apple, Mac mini power consumption, https://support.apple.com/en-us/103253 , read (page dated 21 September 2026) | awake and idle, not asleep; one vendor |
| Prosumer workstation, integrated accelerator | 6 W (M4 Max, 2025), 9 W (M3 Ultra, 512 GB, 2025), 9 to 13 W (2022-2023 models); maximum 145 to 295 W | same definition | vendor-measured, at the wall | 2022-2025 | Apple, Mac Studio power consumption, https://support.apple.com/en-us/102027 , read | "Max" is a processor stress test, not inference |
| Prosumer workstation, one high-end consumer card | card alone about 22 W (RTX 4090) and 30 W (RTX 5090) at idle; others report 45 to 100 W for the newer card | graphics card only | users' forum measurements | 2025 | `[search summary of forum posts]` | not a review, not the whole PC. A whole desktop with such a card idles at perhaps 60 to 100 W `[my reading; not sourced]` |
| Small inference appliance | not published; supply 240 W, chip 140 W | - | vendor specification, CAP | 2025-2026 | NVIDIA DGX Spark page, read | no idle figure exists on the page |
| On-premises eight-GPU server | not published; 10.2 kW maximum | - | vendor specification, CAP | 2023 | NVIDIA DGX H100 user guide, read | idle of such a server is of the order of 1 to 2 kW `[my reading; not sourced]` |
| Hosted hyperscale | idle is inside the per-prompt figure: 0.02 of 0.24 Wh (10 %) for machines held in reserve, plus 0.02 for cooling and conversion | energy "consumed by idle systems provisioned for reliability and low latency", allocated per prompt | operator-measured | May 2025 | Google, arXiv 2508.15734, read | one operator; conventional servers idle at about 35 % of rated power (section 3.1) |
| Small board, home gateway | 3.1 W; 2.8 to 5.2 W | section 3.2 | - | - | section 3.2 | - |

`[my arithmetic]` What idle does to energy per query on a box that is on for the mesh. Take the best case found, a small desktop idling at 5 W and answering 50 queries a day at 0.1 Wh each (a 600-token reply at about 0.6 J per token): inference 5 Wh a day, idle 120 Wh a day. **Idle is 96 % of the box's energy and the effective cost is 2.5 Wh per query, ten times the hosted frontier figure of 0.24.** At 6 W it is 53 kWh a year; a desktop with a high-end card at an assumed 80 W is 700 kWh a year, ten times the whole per-user data-centre baseline of 72. The conclusion of section 3 holds for AI as it did for storage: the tier is decided by whether the machine was on anyway and how many people share it. A machine already on for other reasons, sleeping otherwise, or shared by an office, changes the sum entirely; the query count is my assumption.

## 4c.3 Which jobs route where, and the direction (index)

In A2. In one line: no source measures the share of inference run on devices; capability is measured (local models answer 71 to 89 % of single-turn queries correctly in 2025, 23 % in 2023); capable hardware is about a third of new phones and PCs (analysts, `[search summary]`); the one routing study sends the easy four-fifths of queries to small local models and the hard fifth to a hosted frontier model, and attributes the 48 to 80 % energy saving to model size. Addition from 4c.1: the industry's own client benchmark stops at 8 to 14 billion parameters at 4-bit, which is where "runs usefully on a PC" sits today.

## 4c.4 Tokens per person under agents, and zero data retention (index)

In A5. In one line: agents about 4 times the tokens of chat per task and multi-agent systems about 15 (one operator's statement, June 2025); tokens per request on a router's whole traffic up fourfold in two years; a chat user sends about 3 prompts a day `[my arithmetic]`; per-person-per-day agent use is not measured for households. Zero data retention means prompts and responses are not stored after processing; metadata, safety exceptions and in-memory caching remain.

## 4c.5 Table to transcribe

| Tier | Energy per unit | Idle draw | Largest useful model | Year | Source | Trust |
|---|---|---|---|---|---|---|
| Phone | 0.58 to 0.76 J/token (3 bn, 4-bit; whole phone; phone GPU, not the neural processor) | about 0.01 to 0.05 W, on regardless | about 3 bn comfortably; 10 bn at 15 tokens/s on 2024 flagships (vendor claim) | 2022-2024 | Laskaridis et al. 2024; IEA 2025 | **Medium.** Measured at the battery, but two-year-old phones and no neural-processor figure exists |
| Laptop or small desktop, integrated accelerator | 0.09 to 0.25 J/token (3 to 9 bn, 4-bit) | 4 to 7 W at the wall | 8 to 14 bn at 4-bit (the client benchmark's range) | 2024-2026 | Kannan et al. 2026; Apple; MLPerf Client | **Energy: low** (system power assumed, a preprint). **Idle: high** (vendor-measured at the wall) |
| Prosumer workstation, integrated accelerator (128 to 512 GB) | 1.6 to 2.3 times worse per joule than a data-centre accelerator on the same 8 to 120 bn models, unbatched; no absolute J/token read | 6 to 13 W at the wall | up to about 120 bn (measured running) | 2025 | Saad-Falcon et al. 2025; Apple | **Ratio: high** (measured, repeated, like for like). **Idle: high** |
| Prosumer workstation, one consumer graphics card | 0.27 to 0.32 J/token (1 bn; card only); up to 4.4 times that at 7 bn | card 22 to 30 W; whole PC not sourced | about 7 to 14 bn on a 16 to 24 GB card `[my reading]` | 2025-2026 | Zähl and Hennig 2026; forum posts | **Low.** Mid-range card, card-only scope, idle from forum posts |
| Small office or home appliance | not published | not published (240 W supply, 140 W chip) | "up to 200 billion parameters" (vendor claim, precision unstated) | 2025-2026 | NVIDIA product page | **None for energy.** Capacity claims only |
| Edge or on-premises GPU server | unbatched, within about 2 times a workstation on the same model; no absolute figure. About 4 J/token for 7 bn is an estimate, not a measurement | not published (10.2 kW maximum for eight accelerators) | hundreds of billions | 2023-2025 | NVIDIA user guide; Saad-Falcon et al.; Kannan et al. | **Low.** No measured energy at realistic office load |
| Hosted, batched, open model | 0.15 to 0.21 J/token (32 bn); 0.026 Wh per 627-token reply; 11 to 20 times better than unbatched | carried in the operator figure below | any | 2025-2026 | Chung et al. 2026; Saad-Falcon et al. | **High** for the accelerator node; excludes cooling and idle reserve |
| Hosted, frontier, in production | 0.24 Wh per median text prompt, everything inside the data centre | 10 % of the above is idle reserve, 8 % cooling | any | May 2025 | Google (Elsworth et al. 2025) | **High as measured; low as a constant:** it fell 33-fold in a year |

## 4c.6 For a human to open

- **MLPerf Inference results tables, rounds v5.1 to v6.1, filtered to power** (mlcommons.org): the only wall-power, like-for-like figures for edge and data-centre systems. An interactive dataset; not opened under the documentation-only rule.
- **A measured idle and loaded wall draw for a single-card workstation and for the small appliance**: reviews exist (hardware press) but were seen only as forum summaries.
- **Any neural-processor energy-per-token measurement on a 2025 phone**: not found.
