# SEPARATOR SCOUT: is the coarse level told only the conserved additive quantity?

*Provenance, added when this report entered the repository (2026-11-11): written by a scouting
subagent from web search, abstracts and search summaries only; no full text was opened, nothing
was downloaded and nothing was computed. Its "not found" is weak evidence of absence. It is a
map of where to read, not a prior-art screen, and nothing may be frozen on it alone.*

Scouted 2026-11-11. Literature and data landing pages only; nothing downloaded, nothing computed.

**How to read the evidence grades.** Every claim was obtained through web search this session. In most
cases I saw the abstract or a search-engine summary of the abstract, not the full text. Grades:

- **[A]** stated in the paper's own abstract / publisher or data-provider page as returned by search.
- **[S]** secondary: stated by a later paper, a talk, a catalogue mirror, or a press release about the source.
- **not found** = I searched and could not confirm. Nothing below is filled in from memory; where the
  brief named a paper I could not retrieve, it is listed as not found rather than described.

No full texts were opened (no fetch tool was used), so effect sizes buried in tables are mostly "not found".
That is the main limitation of this scout and the first thing a follow-up should fix.

---

## CANDIDATE 1 — COSMOLOGY: halo / galaxy assembly (secondary) bias

### 1. State of knowledge

**Settled (simulations).**
- Halo clustering depends on properties other than mass. Later-forming low-mass halos are less clustered
  than typical halos of the same mass; the effect is strongest at low mass (studied at
  M <~ 6x10^12 Msun/h) and weak at the massive end. Gao, Springel & White 2005, as summarised by citing
  papers [S] (e.g. arXiv:0803.2250, arXiv:1612.08595). Original paper and its numerical amplitude: not found.
- The sign reverses at high mass for some formation-time definitions and the clustering depends on
  concentration; Wechsler et al. 2006 [S, via arXiv:0803.2250 and a 2015 talk]. Transition mass: not found.
- Amplitude and even sign depend on the definition of formation time [S, arXiv:0803.2250].
- Two mechanisms: at low mass, cessation of accretion onto a sub-population of small halos; at high mass,
  peak curvature. Dalal et al. 2008 [S].
- "Assembly bias" is the wrong name for part of it. For cluster-size halos, concentration and subhalo
  abundance give a secondary bias although assembly bias in the strict sense is absent. Mao, Zentner &
  Wechsler 2018, MNRAS, arXiv:1705.03888 [A]. So the non-conserved variable needed is not one variable:
  it is a different variable at different masses.
- Halo assembly bias is a robust LCDM prediction; GALAXY assembly bias is not (it depends on how galaxy
  formation couples to halo history). Mandelbaum, KITP talk 2017 [S].
- In hydrodynamic simulation (IllustrisTNG) ignoring environment-based secondary dependence shifts LRG
  clustering predictions by roughly 10-20 per cent, ELGs about 4 per cent and uncertain. arXiv:2310.09329 [S,
  search summary].

**Disputed (observations).**
- Yang, Mo & van den Bosch 2006: first observational claim; later work attributed the signal to halo-mass
  differences or satellite contamination. [S, as described by Lin et al. 2016]. Original: not found.
- Wang, Weinmann, De Lucia & Yang 2013 (arXiv:1305.0350, "Detection of galaxy assembly bias"): claimed
  detection from sSFR dependence of clustering of low-mass centrals [A].
- Lin, Mandelbaum et al. 2016 (arXiv:1504.07632): null. Early- versus late-forming centrals at mean halo
  mass ~9x10^11 Msun/h, lensing-checked: relative bias 1.00 +/- 0.12 [A].
- Miyatake et al. 2016 (arXiv:1601.06063): strong signal in redMaPPer clusters, larger than the LCDM
  maximum [S]. Rebuttals: Zu et al. 2017 (arXiv:1611.00366), Busch & White 2017 (MNRAS 470, 4767),
  Sunayama & More 2019 (arXiv:1905.07557): the signal is mostly projection, preferential contamination of
  low-concentration clusters by foreground/background groups; with a projection-robust radius it falls to at
  or below the simulation maximum [A/S]. This one is effectively settled as an artefact.
- Zentner et al. 2019 (arXiv:1606.07817): decorated-HOD fits to SDSS DR7 clustering cannot rule assembly bias
  out; faint thresholds favour central assembly bias, none at Mr < -21 [A]. Vakili & Hahn 2019: modest,
  centrals only [S].
- Wang et al. 2022 (arXiv:2204.05332, SDSS DR7 count statistics): no central assembly bias in the most
  luminous sample, marginal preference for negative satellite assembly bias in fainter samples; says
  results in the field are mixed [S].
- Yuan et al. 2021 (arXiv:2010.04182, MNRAS 502, 3582) and AbacusHOD (arXiv:2110.11412): BOSS CMASS
  redshift-space clustering fits improve markedly with concentration-based and environment-based secondary
  terms (talk slides: delta-BIC 21, 17, 36 combined; concentration term degenerate with sigma_8);
  environment is the stronger tracer [A for the claim, S for the numbers].
- Pearl et al. 2024 (ApJ 963; arXiv:2309.08675), DESI One-Percent, counts-in-cylinders: Mr < -21 at z~0.25
  favours positive central assembly bias at 94.8 per cent; z~0.15 no significant evidence [S, search summary].
- Shao et al. 2025 (arXiv:2510.20896), "first direct measurement", DESI DR1 BGS, group-galaxy
  cross-correlation plus weak lensing for halo mass: Q_sat = 0.05 +/- 0.14, consistent with zero and in
  tension with many empirical galaxy-formation models [A]. Central value Q_cen: not found. A 2026 seminar
  abstract by the same author gives Q_env = -0.06 +/- 0.13 [S].
- Wang et al. 2026 (MNRAS 545; arXiv:2511.15354), DESI DR2 galaxy multiplets: standard HOD fails; adding an
  environment-dependent secondary term improves the fit [S, search summary].
- Rodriguez & Montero-Dorta 2025 (A&A; arXiv:2511.19607), SDSS groups: several secondary biases at fixed
  group mass robustly detected (central colour, environment); the authors state assembly bias itself is
  still not robustly established observationally [S, search summary].

**Consensus as of late 2026.** There is none for a detection. The pattern is consistent: MODEL-FIT analyses
(HOD with a secondary parameter) prefer a non-zero secondary term, mostly environment-based, at modest to
strong significance; DIRECT analyses that pin the halo mass by lensing return zero within ~0.12-0.14.
The two are not in formal contradiction because they measure different things. Main systematics:
halo-mass proxy (the whole problem), projection, satellite/central misclassification, and for model fits
the degeneracy of the secondary term with sigma_8 and with the HOD's functional form.

### 2. Sharpest open question on real data
At fixed lensing-calibrated halo mass, does the large-scale bias of central galaxies differ between the
upper and lower halves of a galaxy property (colour, sSFR, or local density), by more than the ~12 per cent
that current direct measurements can exclude? Either answer is allowed by published work. A sharper form:
do the model-fit preference for an environment term and the direct null survive on the SAME sample?

### 3. Smallest open dataset
- SDSS DR7 group catalogues (Yang et al. 2007/2012), gax.sjtu.edu.cn/data/Group.html, public [A]. Size and
  licence: not found (expected small, catalogue-scale). Tinker's self-calibrating group catalogue: not found.
- DESI DR1 (public; first 13 months of the main survey, 18.7M redshifts). Whole release ~270-300 TB
  (two figures in secondary sources, they disagree); redshift catalogues ~0.1 TB; LSS catalogues with
  randoms and weights are part of the release [S, China-VO mirror and press]. Licence: not confirmed
  (the CC BY 4.0 claim in the brief: not found). DESI DR2: NOT public as of a January 2026 paper
  (arXiv:2601.02463); no release date found.
- DESI Legacy imaging group catalogue (Yang 2021): the page says spectroscopic version is for collaborators
  on request, with a conflicting line that the DR8 imaging catalogue is available [A, conflict noted].
- Euclid: DR1 was moved. ESA's 15 June 2026 notice splits it: November 2026 "DR1-Foundation" = raw data,
  calibrated images, catalogues, spectra over ~1900 deg^2; full release mid-2027 (the latter from a blog,
  unconfirmed) [A/S]. No clustering or lensing-mass products are promised in what I could see. Not a
  shortcut to this question.
- Simulations: AbacusSummit ~2 PB full (OLCF tape), 750 TB at NERSC, Globus, stated to be public domain,
  no explicit licence text found. Quijote >1 PB, Globus/Binder, licence not found. IllustrisTNG ~750 TB to
  1.1 PB, registration required. Uchuu: Skies & Universes / CESGA platform, size and licence not found.
- Main systematic that sinks a naive analysis: the halo-mass proxy. Any group-finder mass correlates with
  the very galaxy properties used to split the sample, so a "secondary" signal at fixed proxy mass is
  largely a primary mass signal. Needs shear catalogues (a second large dataset) to do honestly.

### 4. Chance an outsider produces something experts regard as new: **LOW.**
The direct measurement has just been done on DR1 by people inside the collaboration, with lensing, and the
next step (DR2, three times the data) is not public. An outsider on SDSS re-does 2013-2022. What a
preregistered exact-statistics method could add is narrow but real: a blind, same-sample comparison of
the model-fit and the direct estimator with planted signals, to show whether the HOD preference is a
functional-form artefact. That is a methods note, not a discovery, and it needs lensing data and mocks at
the 100 GB to TB scale.

### 5. Ill-formed as posed?
Partly. (i) Halo mass is not observable and not uniquely defined (boundary choice), so "the conserved
additive quantity" is itself a model output in real data. (ii) In simulations the answer is already "no,
mass is not enough", and the extra variable is mass-dependent (Mao et al. 2018), so "one extra number" is
false there. (iii) On real data the question is about galaxies, where the answer depends on the tracer
and the statistic; there is no single yes/no.

---

## CANDIDATE 2 — GEOGRAPHY / DEMOGRAPHY: is a place more than its head-count?

### 1. State of knowledge

**Settled.**
- City-size distribution: bulk lognormal (Eeckhout 2004, AER 94, 1429), upper tail Pareto for roughly the
  largest 1000 US cities (Levy 2009 comment; Malevergne, Pisarenko & Sornette, arXiv:0909.1281, by a
  uniformly most powerful unbiased test) [A for the last, S for the first two].
- Gibrat's law (growth rate independent of size) fails at fine scale: with clusters built from gridded
  population (GB at 200 m, USA ~1 km, Africa), mean and standard deviation of growth rates fall with size,
  in some cases as a power law, attributed to long-range spatial correlations in growth. Rozenfeld, Rybski,
  Andrade, Batty, Stanley & Makse 2008, PNAS 105, 18702; arXiv:0808.2202 [A]. Exponents: not found.
  This is direct evidence that growth depends on relationships to other places, not on own count alone.
- Flows: gravity with exponential distance decay outperforms the radiation law for commuting in six
  countries, on flows, network structure and distance distribution, and still fails at long distance;
  earlier comparisons were unfair because the models were constrained differently. Lenormand, Bassolas &
  Ramasco 2016, J. Transport Geography 51, 158; arXiv:1506.04889 [A]. Numerical CPC values: not found
  (searched twice). England and Wales test (arXiv:1206.5735): gravity better overall, radiation competitive
  at large scale [A]. Simini, Gonzalez, Maritan & Barabasi 2012, Nature 484, 96 [A for existence]; Zipf 1946:
  not found.
- Adding non-count structure to flows helps a lot, by a machine-learning yardstick: Deep Gravity (Simini,
  Barlacchi, Luca & Pappalardo 2021, Nature Communications 12, 6576) uses 18 geographic features per place
  (land use, roads, POIs) plus population and distance and reports up to 66 per cent relative CPC
  improvement over gravity, far more in dense areas [A/S]. That is 36 extra numbers, not one.
- Migration, sociology: Huang & Butts 2023 (ASR 88, 1031; arXiv:2205.02347), US intercounty flows 2011-2015
  with network models: fewer moves between counties dissimilar in politics, urbanisation and race;
  removing those mechanisms would give ~4.6 million (27 per cent) more migrants a year [A]. Relational,
  non-count structure with a quantified size.
- Gravity models of internal migration "often" reach adjusted R^2 0.8-0.9 [S, a review; specification not
  stated, and such R^2 on log flows is dominated by the size terms].
- Data source changes the verdict: Morefield & Leslie 2025 (J. Geographical Systems 27, 455): gravity,
  Poisson and radiation on IRS, ACS and Census flows; IRS looks better only because it censors flows under
  20 people; censoring ACS the same way closes the gap [S, EPA abstract]. Replication archive is public.
- Forecasting: for total population, complex/causal models have been no more accurate than simple
  extrapolation; error grows with horizon, growth volatility and small size (Smith 1992, 1997; Smith &
  Tayman 2003; Rayer 2008) [S, search summary of abstracts]. This is the closest existing answer to "how
  much does structure add to next-period count": historically, about nothing for totals.
- Urban scaling: Bettencourt et al. 2007 (PNAS 104) superlinear socioeconomic, sublinear infrastructure [S];
  exponents are not robust to city definition (Arcaute et al. 2015, J. R. Soc. Interface) [A/S] nor to the
  statistical model (Leitao et al. 2016) [S]; Shalizi 2011 on alternative functional forms [S].

**Disputed.** Universality of scaling exponents; whether radiation or gravity is the better parameter-light
law (leaning gravity).

**Open / not found.** I found NO paper that asks the question in the brief's form: given head-counts and
distances only, what fraction of flow and of growth is explained on held-out data, and what is the
increment from exactly ONE named non-count variable, ranked across candidate variables. The pieces exist
(Deep Gravity's feature attributions, Huang & Butts's knockouts, the forecasting literature's null for
totals) but not the single ranked table.

### 2. Sharpest open question on real data
For US counties, with measured components of change: does any ONE non-count variable (median age or age
share, within-county unevenness of density, prior-year net migration, network position in the measured
flow graph) reduce out-of-sample error in next-year NET DOMESTIC MIGRATION beyond a count-and-distance
baseline, by more than a pre-stated margin, in every year block? The forecasting literature predicts "no"
for totals; Rozenfeld, Huang & Butts and Deep Gravity predict "yes" for a relational variable. It can come
out either way, and natural increase (births minus deaths) is a planted positive control: age structure
MUST help there, so a method that fails to see it is broken.

### 3. Smallest open dataset
- US Census Bureau county population totals and components of change (births, deaths, domestic and
  international migration, residual), 2020-2024 file with Vintage 2025 now current; earlier decades as
  separate vintages. census.gov/data/datasets/time-series/demo/popest/2020s-counties-total.html [A].
  One CSV, ~3,100 counties; exact size and column names not confirmed (expected megabytes). US federal
  work; explicit licence statement not found. The RESIDUAL is published and is the change no component
  explains (older glossary: it comes from applying national controls) [A/S].
- IRS SOI county-to-county migration, 1990-2022; CSV from 2011, zipped per-state Excel before
  (irs.gov/statistics/soi-tax-stats-migration-data) [A]. Size: not found (expected tens of MB a year).
- LODES commuting, 2002-2023, block-level OD, per-state CSV, lehd.ces.census.gov/data [A]. Size: not found.
- ACS county-to-county flows, Eurostat regional demography, UK ONS internal migration: not found this pass.
- GHSL GHS-POP R2023A: twelve 5-year epochs 1975-2030 at 100 m; open with attribution. **Its time series
  is not measured dynamics.** Population is CIESIN GPW census/administrative totals "spatially and
  temporally interpolated or extrapolated" to the epochs, then spread within units by that epoch's built-up
  surface; 2025 and 2030 are projections [A, JRC/Earth Engine metadata]. Independent check: gridded products
  including GHS-POP predicted under 20 per cent of inter-census county change in China (Population Council)
  and a six-product evaluation found decadal change "severely limited" [S]. Any within-unit "dynamics" is
  built-up-area change, and between-unit dynamics is census interpolation. Use it for one-epoch spatial
  pattern only. WorldPop annual: same class of problem per those evaluations; details not found.
- Main systematics for a naive analysis: (a) the components are ESTIMATES, with domestic migration itself
  derived largely from IRS and Medicare records, so IRS flows and Census migration are not independent
  (this dependence is my inference from the methods-statement title, not confirmed in text; check first);
  (b) IRS breaks in 2011 and 2022 (matching change, ~5 per cent more returns) and censoring of flows under
  20 people, which alone flips model rankings (Morefield & Leslie); (c) vintage revisions and the 2020
  census re-base; (d) county is an arbitrary unit (Arcaute's point).

### 4. Chance of something new: **MEDIUM** (low-medium for experts' interest, high for feasibility).
The single ranked "one extra variable" table on measured components, with planted controls and
year-blocked holdouts, does not appear to exist. Regional scientists will find a null for totals
unsurprising and a relational positive unsurprising; what would be new is the exact size, the ranking,
and the demonstration on the published residual. The project's own within-cell unevenness table is a
candidate variable nobody else has in that form.

### 5. Ill-formed as posed?
In two ways. (i) Head-count is additive but NOT conserved: births, deaths and international migration are
sources. Only domestic migration is a conserved transfer, so the clean test is on that component.
(ii) "Counts and distances" already smuggles in a relational variable (distance), and gravity has fitted
parameters; the count-only baseline must be stated exactly. Also the unit is a choice, not an object.

---

## CANDIDATE 3 — WEATHER / CLIMATE: what must a coarse cell carry beyond column budgets?

### 1. State of knowledge

**Settled (models and simulations).**
- Precipitation rises sharply with column water vapour / column relative humidity over tropical oceans:
  P = exp[11.4 (r - 0.522)] mm/day from 4 years of daily SSM/I; correlation of actual with r-predicted
  precipitation exceeds 0.5 on intraseasonal and longer scales. Bretherton, Peters & Back 2004, J. Climate
  [A/S]. Critical value depends on tropospheric temperature (Neelin, Peters & Hales 2009, JAS) [S].
  Fraction of DAILY or sub-daily variance explained by CWV alone: not found.
- A coarse cell needs more than grid-mean conserved fields. In storm-resolving simulation, a network given
  only large-scale variables predicts precipitation with R^2 ~0.45 and misses extremes; given one learned
  organisation metric (from the sub-grid precipitable-water field) R^2 ~0.9; the metric is predictable as a
  simple memory process from earlier time steps. Shamekh, Lamb, Huang & Gentine 2023, PNAS,
  doi:10.1073/pnas.2216158120 [A, abstract]. This is the cleanest published "one extra number" result in any
  of the three fields, and it is in a SIMULATION.
- Memory: most "microstate" memory lives in low-level thermodynamic structure (cold pools, thermals) and is
  strengthened by organisation. Colin, Sherwood, Geoffroy, Bony & Fuchs 2019, JAS 76, 947 [A]. Existing
  schemes: Zhang-McFarlane has none; the "org" scheme (Mapes & Neale 2011, JAMES 3, M06004) and LMDZ
  cold pools capture it partly. Hwong et al. 2023, JAMES 15, e2023MS003726 [A].
- ML parameterisations: previous-step tendencies as memory inputs reduce error and improve stability
  (Han et al. 2020; "Navigating the Noise", arXiv:2309.16177; Lin et al. 2025; Behrens et al. 2025;
  arXiv:2407.00124) [S]. Rasp, Pritchard & Gentine 2018; Yuval & O'Gorman 2020; Brenowitz & Bretherton;
  Buizza; Berner et al. 2017: not found this pass.

**Shown on observations.**
- More aggregated deep convection goes with a drier free troposphere away from convection. Tobin, Bony &
  Roca 2012, J. Climate 25, 6885 (SCAI index, satellite plus reanalysis) [A]; ~10 per cent in precipitable
  water across the SCAI range per a poster [S].
- Organisation and lower-tropospheric stability together explain over 60 per cent of the variance of the
  tropical-mean radiation budget (monthly anomalies). Bony et al. 2020, AGU Advances 1, e2019AV000155 [A].
  That is the tropics-wide mean, not a grid cell.

**Disputed on observations (this is the opening).**
- Clustering enhances rain: at given saturation fraction, strongly clustered oceanic convection precipitates
  more intensely; mostly via convective intensity when the column is under ~70 per cent saturated.
  Angulo-Umana & Kim 2023, Science Advances 9, eabo5317 [A/S].
- Against: domain-mean precipitation extremes depend primarily on the NUMBER of deep convective entities,
  while local extremes (mainly over land) increase with organisation. Semie & Bony 2020, GRL,
  doi:10.1029/2019GL086927 [A]. And in a global storm-resolving simulation, mesoscale precipitation "varies
  independently from the spatial arrangement of its convective cells": moisture adds cells, not clustering.
  Brueck, Hohenegger & Stevens 2020, QJRMS 146, doi:10.1002/qj.3742 [A].
- The pickup is carried mostly by stratiform rain; convective rain picks up only weakly. Ahmed & Schumacher
  2015, GRL 42, 10453 [A/S]. So "organisation" may be a proxy for stratiform fraction.
- The observational size of the increment from one organisation variable at fixed CWV, as variance
  explained on held-out scenes: not found.

### 2. Sharpest open question on real data
At fixed column relative humidity and tropospheric temperature, on ~1 to 10 degree ocean scenes, does one
organisation number computed from a field INDEPENDENT of the rain estimate (infrared brightness-temperature
cloud clusters) add out-of-sample skill for scene-mean precipitation, and if so is it distinguishable from
cell COUNT? Angulo-Umana & Kim say yes; Semie & Bony and Brueck et al. say count carries it. A second,
equally open form: does the PREVIOUS state of the scene (memory) add skill at fixed current CWV, which
simulations assert (Colin, Shamekh) and no observational paper I found quantifies.

### 3. Smallest open dataset
- GPM IMERG Final V07, half-hourly 0.1 degree, NASA GES DISC; ~3.5 TB for the whole record (UCAR catalogue
  figure) [S]; per-file size, login requirement and use terms: not found. A tropical-ocean box for a few
  years is tens of GB by proportion (my arithmetic, not a published figure).
- ERA5 column water vapour and temperature: licence became CC BY 4.0 on 2 July 2025 [A, ECMWF forum].
  A single-level hourly subset is small; exact size not found. Requires a free account to retrieve, which
  the scouting rules forbade me to test.
- Satellite water vapour (SSM/I, as used by Bretherton et al.): access terms not found.
- Simulation archives: ClimSim, CC BY 4.0 on Hugging Face; low-resolution 744 GB (100 M samples),
  high-resolution 41.2 TB (5.7 billion samples) [A]. But ClimSim stores column inputs and outputs of an
  embedded cloud model, not the sub-grid fields, so it can test MEMORY but not spatial organisation
  (my reading of the dataset description; verify). DYAMOND access terms: not found.
- Main systematic: circularity and shared retrievals. An organisation index computed from the rain field
  is a function of the quantity being predicted; IMERG fills gaps by morphing with infrared, so an
  infrared-based index is not independent of IMERG either; and reanalysis CWV is partly model. The clean
  version needs radar-only rain (GPM DPR swaths) against infrared organisation, which costs sample size.
  Second systematic: the index changes with scene size and resolution, so a result at one scale is a
  price in a regime.

### 4. Chance of something new: **MEDIUM.**
The dispute is real and current, the three sides used different definitions and scales, and nobody appears
to have published the held-out increment with count and organisation separated and the retrieval
dependence controlled. Experts would regard a clean, preregistered separation of "count" from
"arrangement" from "memory" at fixed CWV as a contribution. The cost is data volume (tens of GB at least)
and retrieval expertise, and the risk is that the honest answer is "undecidable with gap-filled products".

### 5. Ill-formed as posed?
Slightly. Column water is conserved only as a budget (sources and sinks are the thing predicted), so
"depends only on the conserved quantity" means "on the current column state". "Organisation" has no single
definition (SCAI, cell count, learned metric), so "one extra number" is true only relative to a chosen
index and scale. Within those limits the question is well posed and the simulation answer is known
(R^2 0.45 to 0.9 with one learned number).

---

## RANKING: (novelty attainable) x (feasibility with modest data)

| | novelty attainable | feasibility | product | data needed |
|---|---|---|---|---|
| 2 Geography | medium-low | high | **first** | megabytes, public, measured components |
| 3 Weather | medium | medium-low | second | tens of GB to TB, accounts, retrieval dependence |
| 1 Cosmology | low | low | third | TB plus lensing plus mocks; decisive data (DR2) not public |

**Do first: Candidate 2, on US counties, on the domestic-migration component.** Reasons: (a) the data are
megabytes, measured rather than simulated, and have a published residual; (b) there is a built-in positive
control (age structure must predict natural increase) and a built-in conserved transfer (domestic migration
sums to zero nationally), which is exactly what a planted-signal method needs; (c) the specific table,
one extra variable at a time against a stated count-and-distance baseline, was not found in the literature;
(d) the forecasting literature's standing claim that structure adds nothing for totals gives a real chance
of a null, so the measurement can come out either way. Do NOT use the GHSL time series as dynamics.

**Candidate 3 is the more interesting science** and has the only published one-number result (in
simulation). It is second only because a naive observational analysis is circular, and a clean one needs
more data and care. If a second campaign is wanted, the memory form of the question on ClimSim low-res
(744 GB, CC BY) is the cheapest entry, but it is simulation, which the brief ranks below observations.

**Candidate 1: do not start.** The field has expert teams, the direct measurement was published in late
2025 with a null at +/-0.14, and the dataset that will move it is not public.

## What this scout did not establish (follow-up before any freeze)
1. No full texts read. Needed first: Lenormand et al. CPC table; Rozenfeld exponents; Shamekh's exact R^2
   definition; Shao et al. Q_cen; Angulo-Umana & Kim's effect size; Morefield & Leslie's radiation result.
2. Not found at all: Zipf 1946; Tinker group catalogue; Rasp 2018; Yuval & O'Gorman 2020; Brenowitz &
   Bretherton; Buizza; Berner 2017; ACS flows; Eurostat; ONS; WorldPop construction; DYAMOND terms;
   DESI DR1 licence; exact sizes of the Census, IRS and LODES files.
3. Whether Census domestic-migration estimates are derived from IRS records (decides independence of the
   two main Candidate 2 datasets): asserted here as an inference, not confirmed.
4. A dedicated search for "incremental value of one covariate over population in migration/growth
   prediction" in regional-science journals; my not-found is from general web search, which is weak
   evidence of absence.

## Sources (all reached by web search on 2026-11-11)
Cosmology: arxiv.org/abs/1705.03888; arxiv.org/abs/0803.2250; arxiv.org/abs/1612.08595;
arxiv.org/abs/1305.0350; arxiv.org/abs/1504.07632; arxiv.org/abs/1601.06063; arxiv.org/abs/1611.00366;
academic.oup.com/mnras/article/470/4/4767/3892350; arxiv.org/abs/1905.07557; arxiv.org/abs/1606.07817;
arxiv.org/abs/2204.05332; arxiv.org/abs/2110.11412; cosmology.lbl.gov/talks/Yuan_20.pdf;
arxiv.org/abs/2309.08675; arxiv.org/abs/2510.20896; arxiv.org/abs/2511.15354; arxiv.org/abs/2511.19607;
arxiv.org/abs/2310.09329; arxiv.org/abs/2601.02463; gax.sjtu.edu.cn/data/Group.html;
gax.sjtu.edu.cn/data/DESI.html; nadc.china-vo.org/res/r102043/; cosmos.esa.int/web/euclid/dr1-timeline;
abacussummit.readthedocs.io/en/latest/data-access.html; quijote-simulations.readthedocs.io/en/latest/access.html;
tng-project.org/data; skiesanduniverses.iaa.es/Simulations/Uchuu/;
online.kitp.ucsb.edu/online/galhalo-c17/mandelbaum/.
Geography: arxiv.org/abs/0909.1281; arxiv.org/abs/0808.2202 (PNAS 105, 18702); arxiv.org/abs/1506.04889;
arxiv.org/abs/1206.5735; arxiv.org/abs/1111.0586; arxiv.org/abs/2012.00489 (Nat. Commun. 12, 6576);
arxiv.org/abs/2205.02347; doi.org/10.1007/s10109-025-00470-7; arxiv.org/abs/1301.1674;
redalyc.org/journal/289/28966571004/html/; bebr.ufl.edu (Smith 1992, 1997; Rayer 2008);
census.gov/data/datasets/time-series/demo/popest/2020s-counties-total.html;
irs.gov/statistics/soi-tax-stats-migration-data; irs.gov/pub/irs-soi/2223inpublicmigdoc.pdf;
lehd.ces.census.gov/data; data.jrc.ec.europa.eu/dataset/2ff68a52-5b5b-4a22-8f40-c41da8332cfe;
developers.google.com/earth-engine/datasets/catalog/JRC_GHSL_P2023A_GHS_POP;
popcouncil.org/insight/the-limits-of-gridded-data-for-tracking-population-shifts-evidence-from-china/.
Weather: doi.org/10.1073/pnas.2216158120; ui.adsabs.harvard.edu/abs/2019JAtS...76..947C;
research-explorer.ista.ac.at (Hwong et al. 2023 JAMES); ui.adsabs.harvard.edu/abs/2012JCli...25.6885T;
doi.org/10.1029/2019AV000155; doi.org/10.1126/sciadv.abo5317; doi.org/10.1029/2019GL086927;
doi.org/10.1002/qj.3742; doi.org/10.1002/2015GL066957; aos.wisc.edu/~lback/wvpprecip.pdf;
arxiv.org/abs/2309.16177; arxiv.org/abs/2407.00124; arxiv.org/abs/2306.08754; github.com/leap-stc/climsim;
forum.ecmwf.int/t/cc-by-licence-to-replace-licence-to-use-copernicus-products-on-02-july-2025/13464;
cmr.earthdata.nasa.gov (GPM_3IMERGHH V07); data.ucar.edu (IMERG size).
