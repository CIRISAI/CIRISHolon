# COUNTY PRIOR ART AND DATA DOCUMENTATION

*Provenance, added when this report entered the repository (2026-11-11): written by a prior-art
subagent that opened full texts where it says so and abstracts elsewhere, and read data
documentation; no data file was downloaded. Its "not found" is of moderate confidence, weakest
for economics working papers.*

Read 2026-11-11. Papers and documentation only. No data file was downloaded (file sizes come from
directory listings and HTTP header requests). Nothing was computed. Follow-up to
`conformance/scene/SEPARATOR_SCOUT.md`, geography section.

**Evidence grades used throughout.**

- **[F]** full text opened and the passage or table read this session.
- **[A]** abstract only.
- **[S]** search-engine summary or a secondary source describing the item. Weak.
- **not obtained** = looked for and could not open. Nothing below is filled in from memory.

A "not found" from this report is stronger than the scout's (targeted searches plus full texts) but is
still a search, not a systematic review. Journals behind paywalls (Journal of Regional Science,
Regional Studies, Demography, Geographical Analysis) were reachable only through abstracts and
search summaries.

---

## PART A. PRIOR ART

### A.1 Has the flow question been done?

**(a) The closest single result: one relational variable against distance, with county fixed effects.**
Bailey, Cao, Kuchler, Stroebel and Wong, "Social Connectedness: Measurement, Determinants, and
Effects" (J. Economic Perspectives 2018; NBER w23608), Table 5 [F, conference version,
https://conference.nber.org/conf_papers/f93418/f93418.pdf]. IRS county-to-county flows 2013-14,
OLS on log flow, origin and destination county fixed effects, 25,305 county pairs (only pairs with
at least 20 movers):

| specification (all with county fixed effects) | R-squared |
|---|---|
| log distance (elasticity -0.973) | 0.610 |
| distance in 50 flexible bins | 0.618 |
| log Social Connectedness Index alone (elasticity 1.134) | 0.893 |
| SCI + log distance (distance coefficient 0.023, not significant) | 0.893 |
| SCI + flexible distance | 0.898 |
| + six dissimilarities (2008 vote share, income, education, race, religion, manufacturing) | 0.899 |

Their words: distance "has no additional predictive power" once SCI is in; the six dissimilarities do
"not have a significant effect on R2". This is, in-sample, the planned study's table for one
relational variable. What it left undone: no held-out year; only the 0.26 per cent of pairs that
survive the 20-return floor (25,305 of about 9.9 million directed pairs), so it is selected on the
outcome; no null; no ranking against other single variables; the SCI snapshot post-dates the flows
and friendships are partly made by past moves, so SCI is a mixture of "relationship" and "history";
nothing on net migration. Also note what the fixed effects mean here: R-squared 0.61 is fixed
effects plus distance together. The share due to size alone is not reported.

**(b) Huang and Butts 2023 in full** (ASR 88:1031; arXiv:2205.02347) [F, preprint v3,
https://arxiv.org/pdf/2205.02347]. Outcome: ACS 2011-2015 county-to-county migrant counts, 3,142
counties. Model: count-valued temporal ERGM by regularised pseudo-likelihood on one million sampled
dyads (all 274 thousand non-zero dyads plus 726 thousand sampled zeros of 9.87 million). Covariates
from the 2010 Census and ACS 2006-2010, and the 2006-2010 flow itself as a lagged edge covariate.
Coefficients: destination log population 0.351, origin about 0.37, log distance -0.568 ("a 10%
increase in distance ... a 5.3% decrease"). Dissimilarity scores are L1 distances in political,
urbanisation and racial composition. Knockout (Table 2): observed 17,176,675 migrants a year;
removing political segmentation +788,661 (4.6 per cent); removing all three +4,564,346 (26.6 per
cent). Fit reported as BIC only (2,221,363 without rural dissimilarity, 2,210,125 with) and
simulated in/out volumes correlating above 0.95 with observed. Left undone: everything is in-sample;
no baseline-only fit statistic; no held-out period; the knockout is a model counterfactual, not a
predictive increment; the lagged flow is in every model, so "history" is never separated out. Their
footnote 8 is itself a data warning: they chose ACS because IRS "post 2011-2012 ... currently
suffers from systemic problems that are not yet resolved (DeWaard et al. 2022)".

**(c) Machine-learning flow models on US counties.** Robinson and Dilkina, "A Machine Learning
Approach to Modeling Human Migration" (arXiv:1711.05462) [F]. IRS county flows 2004-2014, 3,106
counties. Temporal split: train on year t-2, validate t-1, test t; results averaged over test years
2006-2014 (which straddle the 2011-12 IRS break without comment). Seven county features (population,
land area, density, median income, water area, coastal flag, number of neighbours) and six pair
features (distance and five "intervening" sums). Common part of commuters on the full matrix:
traditional models (two gravity forms, radiation, extended radiation) 0.53 to 0.58; XGBoost and a
neural net on population-and-distance features only 0.51 to 0.63; with the extended features 0.63 to
0.69. (The table's columns did not survive text extraction cleanly; the ranges are safe, the
per-model assignment is not.) Feature importance by XGBoost information gain: intervening number of
counties 25.3 per cent, origin population 15.7, destination population 14.2, intervening population
6.1, coastal destination 4.3, distance 3.7. Left undone: gain importance is not a held-out increment
and is order- and correlation-dependent; no single-variable ablations; no nulls; zeros undersampled
in training; nothing relational or historical among the features.

Deep Gravity (Simini, Barlacchi, Luca, Pappalardo; Nat. Commun. 12:6576; arXiv:2012.00489) [F,
preprint]. Not US counties: commuting-type flows at fine units in England, Italy and New York
State, tested on spatially held-out tiles. England: gravity CPC 0.11, non-linear gravity on
population and distance only 0.12, linear multi-feature gravity 0.23, Deep Gravity 0.32. Italy and
New York: +66 and +1076 per cent over gravity. Useful finding for this study: non-linearity on
counts and distance alone buys almost nothing (0.11 to 0.12); the gain is from the 18 place
features. They report that in Italy and New York population and distance carry the largest
attribution, in England the other features do. Left undone: 18 features at once, not one.

Others, abstract only [A]: arXiv:2412.01242 (Bayesian hierarchical, US interstate, origin-
destination-specific preference parameter beats gravity and ML); arXiv:2604.10570 (11,177 US county
flows, ML with attribution maps, heat policies; attribution not held-out increments);
arXiv:2607.00128 (panel prediction method with a county-flow forecasting application, rolling
out-of-sample error on edges with past flows); arXiv:2503.20989 (MIGRATE, block-group flows from
proprietary address histories fused with ACS; a possible future data source, not prior art on the
question).

**(d) Gravity, radiation and intervening-opportunities accuracy.** Lenormand, Bassolas and Ramasco
2016 (arXiv:1506.04889) [F]: eight commuting case studies including 3,108 US counties (2000 census).
Averaged over cases, unconstrained models "reproduce on average around 45% of the observed commuting
network against 65% for the doubly constrained model"; common part of links never exceeds 0.55.
For the USA with population inputs the best law by CPC is normalised gravity with exponential decay,
by NRMSE the extended radiation law (their Table 2). Per-case CPC values are in figures only.
Simini et al. 2012 (arXiv:1111.0586) [F, skimmed]: tests the radiation law on IRS 2007-08
migration but reports no CPC or R-squared for it in the text I could extract, and evaluates only
pairs with non-zero flux. Yang et al. 2014 (arXiv:1407.6256) [A]: a one-parameter radiation
extension for US commuting; numbers not obtained.

Morefield and Leslie 2025 (J. Geographical Systems 27:455; open access, PMC12774388) [F]. OLS
gravity, Poisson, zero-inflated Poisson and radiation, with Stouffer intervening opportunities, on
IRS (1990-2020), ACS (eleven 5-year files) and 2000 Census flows; population at t predicts flow at
t+1. Validation: "20 percent of the data for each year and data source combination ... withheld",
that is, a random split of pairs within a year. Finding: IRS looks best until ACS and Census are
truncated at 20, after which the IRS advantage "is reversed for the ZIP and POISSON models and
reduced for the OLS models but persists for the radiation models". Coefficients differ by under 5
per cent across sources while errors differ a lot. All numbers are in figures; no table of
deviance or R-squared. Left undone: no covariates beyond population, distance and intervening
population; random-pair hold-out; no increments.

**(e) Origin and destination fixed-effects gravity for US internal migration.** I did not find a
paper that reports, for US county-to-county migration, the deviance or pseudo-R-squared of a PPML
model with origin and destination fixed effects and distance, against a size-and-distance-only
model. Bailey et al. (OLS, truncated sample, R-squared 0.61) is the nearest. Candidates seen only
through search summaries [S]: NBER w32123 (commuting-zone pair fixed effects, PPML, IRS-based
administrative flows); Paudel and Lahiri 2024 (state level, PPML, 2000-2017); Howard and Shao,
"The Dynamics of Internal Migration" (Review of Economic Studies, forthcoming; formerly "Internal
Migration and the Microfoundations of Gravity"), which argues that persistent, spatially correlated
preferences rather than moving costs produce gravity. Full texts not obtained. This is a real gap
in this report: an economics working paper with the exact statistic may exist.

**(f) Variable-importance or variance decompositions.** A targeted search for dominance analysis,
Shapley or variance decomposition of US county migration flows returned nothing [S]. An older
IRS-based study is described as finding "surprisingly little effect of economic variables" on
county out-migration [S, not opened].

**(g) The Census residual as an object of study.** Not found. The documentation (Part B.1) explains
why: the residual is a by-product of raking and rounding, not an unexplained demographic quantity.

### A.2 Net migration and total change: does structure help beyond extrapolating counts?

Rayer 2008 (J. Planning Education and Research) [F,
https://www.bebr.ufl.edu/sites/default/files/Research%20Reports/Rayer%20(2008)%20-%20JPER.pdf].
All US counties, the whole twentieth century, five trend-extrapolation methods and two averages.

MAPE by horizon (Table 1a, average over base periods): linear 9.6 per cent at 10 years, 18.8 at 20,
30.1 at 30; trimmed average of methods 9.6, 18.9, 30.8. Errors grow "by about 10% to 14%" per
decade of horizon, close to linearly.

MAPE by size at a 20-year horizon, linear method (Table 3a): under 2,500 people 41.0; 2,500-7,500
28.6; 7,500-15,000 21.4; 15,000-30,000 17.3; 30,000-100,000 16.1; over 100,000 15.7.

Trend versus cohort-component, counties in ten states, about a 5-year horizon to 2000 (Table 5a):
trend averages 3.4 per cent MAPE, cohort-component 3.5. Rayer: "for counties, and at least for
short horizons, projections of total population made with simple trend extrapolation techniques
are comparable to those made with cohort-component models."

Tayman, Smith and Rayer 2011 (Population Research and Policy Review) [F, partial]: 2,482
counties; population size and prior growth rate are significant for forecast error but "neither
variable explained much of the variation"; multivariate models with census division and launch year
do better. Exact R-squared values are in tables I did not extract.

Census Bureau's own postcensal accuracy: MAPE of the final 2010-2020 county estimates against the
2020 Census "only about 2.9 percent across all counties" [F, Vintage 2025 methodology statement].
53.8 per cent of counties within 2 per cent [S].

Not obtained: Smith, Tayman and Swanson's book (a secondary source gives roughly 12 per cent state
and 24 per cent county MAPE at 20 years [S]); Smith 1992 and 1997; Wilson et al. 2022 review
(Population Research and Policy Review 41:865) beyond its abstract, which lists model averaging and
socio-economic covariates as open research needs [A]. An Alabama county study reports
cohort-component 6.5 per cent against neural network 6.1 per cent at the 2010 census [S].

What this literature does and does not say. It says that for TOTAL population at 5 to 30 years,
age-structured cohort-component models do not beat extrapolation of the count's own history. Note
what the winning baseline is: a trend, that is, the count's HISTORY, not the count. Nobody in this
literature tests "count alone, no history" because it is obviously worse. It says nothing about the
one-year net domestic migration component, and nothing about held-out increments from one named
variable. Age structure and natural increase: I found no paper measuring how much age structure
adds to one-year county natural-increase prediction; the Census method itself is the evidence that
it is treated as mechanical (deaths from life-table rates applied to age groups; see B.1).

### A.3 Gibrat's law and spatial correlation of growth

Rozenfeld, Rybski, Andrade, Batty, Stanley and Makse 2008 (PNAS 105:18702; arXiv:0808.2202) [F].
Units are clusters built from populated places, not counties; USA 1990 to 2000. Mean growth falls
with size with exponent alpha = 0.28 +/- 0.08; standard deviation of growth falls as S^-beta with
beta_USA = 0.20 +/- 0.06 (Great Britain 0.27 +/- 0.04, Africa 0.19 +/- 0.04). Shuffling cells
gives beta = 1/2, the uncorrelated value; beta below 1/2 is their evidence of long-range spatial
correlation in growth. For Great Britain the correlation exponent is gamma = 0.93 +/- 0.08 and
beta = gamma/4 is predicted. At coarser US cell sizes (8 km) the mean-growth scaling flattens
toward Gibrat. So: growth variance does not fall as fast as independent sub-units would give, which
is direct evidence that a place's growth reads its neighbours.

County-level tests: Desmet and Rappaport (J. Urban Economics 2017; Kansas City Fed RWP 13-02) [A/S]:
US counties 1800-2000; Gibrat historically fails (convergence among small and young counties, mild
divergence among large); the size-growth relation flattens over time; for 1980-2000 deviations are
still significant in the full sample and not rejected only for long-settled counties. Holmes and
Lee 2008: six-mile grid squares, inverted-U relation of growth to size [S]. Eeckhout 2004: not
opened; cited by others for lognormal sizes and proportionate growth of places [S]. A county-level
measurement of the spatial correlation of growth residuals was searched for and not found.

### A.4 The closest papers, and the verdict

| paper | did | left undone |
|---|---|---|
| Bailey et al. 2018, Table 5 [F] | county FE gravity on IRS 2013-14; distance 0.61, one relational variable 0.89, six dissimilarities +0.001 | in-sample; 25,305 uncensored pairs only; no null; no other single variables ranked; SCI confounds relationship with history; no net migration |
| Huang and Butts 2023 [F] | network count model on ACS 2011-15 with lagged flows; three dissimilarities worth 26.6 per cent of migrants by knockout | in-sample; no baseline fit; no held-out; history never separated; not one variable at a time |
| Robinson and Dilkina 2018 [F] | year-held-out prediction of IRS county flows; counts and distance CPC about 0.53-0.58, plus 7+5 extra features up to 0.69 | features added all at once; gain importance, not ablation; no nulls; no relational or history variable; spans the IRS break |
| Morefield and Leslie 2025 [F] | same models across IRS, ACS, Census; shows censoring at 20 decides which source "wins" | counts and distance only; random-pair hold-out; no increments |
| Rayer 2008 and the Smith-Tayman line [F] | structure does not beat trend for total population at 5-30 years | decadal totals, not the annual domestic-migration component; baseline is history, not count |

**Verdict: (b), done in pieces that nobody assembled, and the largest piece already points to the
answer.** For flows, the qualitative result is foreseeable from Bailey et al.: with margins
absorbed, distance explains about 0.6 and one relational variable about 0.9 of log-flow variance
among large flows, and place-dissimilarity variables add almost nothing on top. A study that
reports "relationships and history matter beyond counts and distance" will tell regional scientists
and economists nothing new. What does not exist, as far as I could find: (1) the held-out-year
version; (2) the version that includes the censored and zero pairs; (3) a ranked table of single
variables each against its own null, with the categories internal-arrangement / relationship /
history kept apart; (4) any of this for next-year net domestic migration; (5) the second-order
question, whether anything about a place's internal arrangement survives once pair history is in.
Items (4) and (5) are the parts that are genuinely open. Confidence in "not found": moderate for the
physics and data-science literature, lower for economics working papers.

---

## PART B. THE DATA, FROM DOCUMENTATION

### B.1 Census Population Estimates Program, county components of change

Sources read in full: Vintage 2025 methodology statement, January 2026
(https://www2.census.gov/programs-surveys/popest/technical-documentation/methodology/2020-2025/methods-statement-v2025.pdf);
file layouts CO-EST2024-ALLDATA and CO-EST2025-ALLDATA; layouts for 2019 and 2009; the datasets
directory listings; the November 2024 intercensal blog post [all F].

**Vintages and years.** The datasets directory holds one folder per vintage: 2000-2001 through
2000-2009, 2000-2010 (intercensal), 2010-2011 through 2010-2020, 2020-2021 through 2020-2025, plus
1970-1980, 1980-1990, 1990-2000. Each postcensal vintage has a county `co-estYYYY-alldata.csv`
carrying the whole series from the decade's base to the vintage year. So every year's components
exist both "as first published" and "as last revised". Sizes from listings: co-est2011 648 KB;
co-est2015 2.1 MB; co-est2019 3.5 MB; co-est2020 3.8 MB; co-est2021 631 KB; co-est2023 1.3 MB;
co-est2024 1.77 MB; co-est2025 2.07 MB (released March 2026). The 1990s are fixed-width text
tables by state, not checked for components.

**Fields** (2025 layout, per year suffix): SUMLEV, REGION, DIVISION, STATE, COUNTY, STNAME,
CTYNAME, ESTIMATESBASE2020, POPESTIMATE, NPOPCHG, BIRTHS, DEATHS, NATURALCHG (named NATURALINC in
2019 and earlier), INTERNATIONALMIG, DOMESTICMIG, NETMIG, RESIDUAL, GQESTIMATESBASE2020,
GQESTIMATES, and rates RBIRTH, RDEATH, RNATURALCHG, RINTERNATIONALMIG, RDOMESTICMIG, RNETMIG.
Periods are July 1 to June 30; the first period of a decade is April 1 to June 30 only. Rates are
per thousand of the mid-period average population. The 2009 and 2019 files have the same component
set including RESIDUAL and group quarters.

**How domestic migration is estimated.** Quoted from the methodology statement:

> "For state and county total estimates, we calculate county-specific net domestic migration based
> on four data sources: 1. Internal Revenue Service (IRS) tax return data for ages 0 to 64;
> 2. Medicare enrollment data from Centers for Medicare and Medicaid Services (CMS) for the
> population aged 65 and older; 3. Social Security Administration's (SSA) Numerical Identification
> File (NUMIDENT) for all ages; and 4. Change in the GQ population".

> "we then compare the addresses between the two years of IRS data to identify the number of
> exemptions that moved from one county to another between tax filings."

> "Not all residents are represented in the tax exemption data, since not everyone files taxes. ...
> To overcome this coverage limitation, we calculate NDM rates instead of using observed flows in
> the tax data."

The rate is (in-migrant minus out-migrant exemptions) divided by (non-migrant plus out-migrant
exemptions), by age group 0-17 and 18-64; the same from Medicare enrolment for 65 and over. Rates
are "applied to an estimated population at risk of migrating in each age group which includes half
of the components of change occurring in the period". Then: "We also treat change in GQ as an
indirect measure of domestic migration", and "These total net domestic migration values are then
controlled to sum to zero at the national level".

Consequences. (i) Published DOMESTICMIG is not a count of movers. It is an IRS/Medicare net RATE
times a household population, plus group-quarters change, then raked to a national zero.
(ii) It is derived from the same tax records as the IRS public flows, processed separately by
Census at person level with ages attached. Predicting Census net domestic migration from IRS
flows is reconstruction, not prediction. (iii) A county whose college dormitory, prison or base
population is revised shows "domestic migration" with no movers in any flow file. (iv) Which IRS
filing year feeds which estimate year is not stated in the statement; DeWaard et al. cite
Toukabri 2017 for the mechanics (not obtained).

**The residual.** Quoted: "the demographic components of change do not account for all of the
year-to-year change in the estimates series. The difference between the result of the balancing
equation and the final estimate is referred to as the residual." It arises because sub-national
estimates are raked to national totals by three age groups and then put through controlled
rounding; "The national population estimates ... do not contain a residual." So the residual is a
consistency adjustment. It is not "change the components fail to explain" in any demographic
sense, and it should not be a target.

**Births and deaths (the positive control).** Final vital statistics "lag by two years". For the
most recent periods, state provisional totals are distributed to counties "using characteristic and
geographic distributions from the last year of final data". So in any vintage, the last two years
of county births and deaths are the previous final year's county shares rescaled. In a first-
published file, natural increase in year t+1 is predictable from year t by construction. The
positive control must use values taken from a vintage at least two years later.

**Revision.** "With each annual release ... the Population Estimates Program revises and updates
the entire time series ... The release of a new vintage of estimates supersedes any previous
series." Inputs to revision: final replacing provisional vital statistics, updated ACS proxy
universes for international migration, group-quarters reports, method changes, and boundary
updates. Size of revision to domestic migration: Connecticut's data office reports its 2022-23 net
domestic migration changed by 2,554 (28 per cent) between Vintage 2023 and Vintage 2024 [S]. The
aging step also uses "the proportions of the previous vintage county population" at ages 17 and 64.
Vintage 2021 to 2024 used a blended base (2020 Census, Vintage 2020, Demographic Analysis); Vintage
2025 switched to the 2020 Census via the Modified Age and Race Census file with a young-child
adjustment. Within the 2020s the base itself changed between vintages.

**Intercensal versus postcensal.** Postcensal = built forward from the last census. Intercensal =
postcensal series adjusted after the next census so it meets both. Quoted from the Census blog:
"intercensal estimates do not adjust the components of change in the vintage estimates, just the
population and housing unit values." For 2010-2020 the error of closure was distributed linearly
(previous decades exponentially). County intercensal files for 2010-2020 carry populations by
age and sex (17 MB) and full characteristics (169 MB); no component files. So populations on the
intercensal series and components on the Vintage 2020 series do not satisfy the balancing equation
together. Pick one series for both.

**Boundaries and FIPS** [F, Census county-changes pages]. Every vintage restates its whole series
on January 1 boundaries of the vintage year, so vintages are internally consistent but differ from
each other. Changes: Connecticut's eight counties replaced by nine planning regions (in the
estimates from Vintage 2022 or 2023; Vintage 2025 text confirms "recently implemented Connecticut
Planning Regions"); Valdez-Cordova Census Area (02-261) split into Chugach (02-063) and Copper
River (02-066), January 2019; Petersburg Borough (02-195) created January 2013 with parts moving
to Prince of Wales-Hyder; Wade Hampton (02-270) renamed Kusilvak (02-158), July 2015; Shannon
County SD (46-113) renamed Oglala Lakota (46-102), May 2015; Bedford city VA (51-515) absorbed into
Bedford County (51-019), July 2013; Clifton Forge VA (51-560) into Alleghany County, July 2001;
Broomfield County CO (08-014) created November 2001 from Adams, Boulder, Jefferson and Weld; several
Alaska splits in 2007-2008. IRS files of 2022-23 still describe Connecticut "councils of
governments", and IRS geocodes with a ZIP+4 crosswalk revised annually, so the two sources need not
agree on Connecticut or Alaska in a given year.

### B.2 IRS Statistics of Income county-to-county migration

Sources read in full or in the relevant sections: Migration Data Users Guides for 2011-12, 2013-14,
2018-19 and 2022-23 (https://www.irs.gov/pub/irs-soi/2223inpublicmigdoc.pdf and siblings); Pierce,
"SOI Migration Data: A New Approach", SOI Bulletin Summer 2015
(https://www.irs.gov/pub/irs-soi/soi-a-inmig-id1509.pdf); Hauer and Byars 2019 (Demographic
Research 40:1153); DeWaard et al. 2022 (PMC8974493) [all F].

**Years and unit.** Annual pairs of filing years from 1990-91; CSV from 2011-12, per-state
spreadsheets before; latest seen 2022-23. Each record: N1 returns (about households), N2
exemptions (called "individuals" in recent guides; about persons), AGI in thousands of dollars,
"based on the year 2 tax return". Separate inflow and outflow files; six header records per county
(total US and foreign, total US, same state, different state, foreign, non-migrants). File sizes:
not obtained (the server returns no length on a header request).

**Suppression, by era.** This is a four-regime series:

| filing years | county-pair floor | what happens below the floor |
|---|---|---|
| to 2012-13 | 10 returns | folded into "Other Flows" rows (same state, different state, four regions, foreign); Other Flows rows under 10 combined with another county's within the state |
| 2013-14 to 2017-18 | 20 returns ("thresholds ... have been raised to 10 (for the state files) and 20 (for the county files)") | folded as before; plus dominance rule: a return that is too large a share of a cell is excluded from county detail |
| 2018-19 onward | 20 returns | "Counts below 20 at the county level will now be deleted. Previously, counts below 20 were moved to a similar category to another county within the state." State totals removed from county files; "county totals may not add to the state totals" |
| 2022-23 | 20 returns | as above, and "Disclosure protection procedures were updated to consistently apply dominance and suppression rules" |

The 2022-23 guide still also says small flows are "aggregated into various 'Other Flows'
categories" and that Other Flows categories under 20 "have been removed" (-1 in CSV). Read
together: pair flows under 20 go to Other Flows rows; Other Flows rows that are themselves under
20 are now deleted rather than re-assigned. Either way a pair that is absent is "0 to 19 returns",
never "zero".

**Method breaks.** 2011-12 (quoted from the guide): data now "based on individual income tax
returns filed and received by the IRS from January 1 to December 31. Previous versions ... through
late September"; and "returns are matched on the taxpayer identification numbers of the primary,
secondary, and dependent tax filers. Prior versions ... primary taxpayer only." Production moved
from Census to IRS. Pierce: matched returns rose 4.7 per cent (111.5 to 116.8 million), 24.6 per
cent in the $200,000-and-over class; "the data are not directly comparable"; Wyoming's net rate
changed 27.6 per cent. 2013-14: file re-released March 2016 because the first release "identified
returns as migrants that should have been identified as non-migrants". 2015 on: identity-theft
processing changes "may have an impact on the migration data". 2022-23: matching updated, "about a
5 percent increase in the number of returns included".

**An unexplained artefact, 2012-13 to 2017-18.** DeWaard et al. 2022: after IRS took over, county
out- and in-migration rates "fell precipitously between 2012-13 and 2014-15", then "increased
dramatically through 2016-17" to the highest on record, swings of 2 to 4 standard deviations in
every region, urbanisation quartile and income quartile, "not readily explained by the IRS's own
account". Net rates are "relatively muted" because the in and out artefacts offset. Their advice:
do not use post-2011-12 IRS gross flows until resolved. I found no later paper saying it was
resolved.

**Coverage.** "Data do not represent the full U.S. population because many individuals are not
required to file". Hauer and Byars: 95 to 98 per cent of the tax-filing universe, about 87 per cent
of households; missing are "undocumented populations, the poor, the elderly, and college students".
The address is a mailing address; a filer "may move after the end of the tax year but file ... up
to nine months later from another location", so the move window is not a calendar or July-to-June
year. Annual ZIP+4 crosswalk revisions move returns between counties without a move; the guide says
these land in non-migrant totals, not flows, and publishes the affected counties.

**Circularity with Census.** Confirmed (B.1). The public IRS file and the Census domestic-migration
component are two tabulations of the same matched tax returns, with different matching rules,
different censoring (Census uses uncensored person records) and, for Census, Medicare for the old
and group-quarters change added.

### B.3 ACS county-to-county flows

Source: 2016-2020 ACS Migration Flow Files Documentation [F,
https://www.census.gov/content/dam/Census/topics/population/migration/guidance-for-data-users/acs-migration-tutorial/2016-2020-Migration-Flows-Documentation.pdf].
Files for every 5-year ACS from 2005-2009 through 2016-2020 (later ones not checked). The
question is residence one year ago; a file is the average annual flow over five survey years.
Windows overlap; the only non-overlapping sets are 2006-10, 2011-15, 2016-20 (and 2005-09,
2010-14, 2015-19). Characteristics only through 2011-2015; later files are counts only.
Each estimate has a 90 per cent margin of error. "Flow counts and mover counts without
characteristics ... are not suppressed"; flows by characteristic with fewer than three cases are.
"Flows with no records in the database ... are not included in the files. The flows with no records
are considered zero count estimates" whose margin of error "cannot be calculated using the standard
replicate variance formula"; a model-based value is given by county size, from 9-13 for counties
under 5,000 to 20-31 for 50,000 and over. Group quarters included from 2006, with whole-person
imputation from the 2011 ACS that copies a donor's previous residence. National file 2016-2020:
103 MB text (header request). Behaviour of small flows: an ACS sample of roughly one per cent a
year means a published flow of a few dozen people rests on a handful of respondents; Morefield and
Leslie find ACS has many more distinct small streams than IRS and that models fit ACS much better
once flows under 20 are dropped.

### B.4 LODES commuting

Source: LODES technical documentation 8.3 [F,
https://lehd.ces.census.gov/data/lodes/LODES8/LODESTechDoc8.3.pdf]. Block-level origin-destination
job counts, 2002-2022 in version 8.3 (the scout saw 2023 on the landing page). Missing state-years
for OD: 2002 Arkansas, Arizona, DC, Massachusetts, Mississippi, New Hampshire; 2003 Arizona, DC,
Massachusetts, Mississippi; 2004-2009 DC and Massachusetts; 2010 Massachusetts; 2011-2016 none;
2017-2018 Alaska; 2019-2021 Alaska and Mississippi; 2022 Alaska, Michigan, Mississippi. Federal
jobs only from 2010 (from OPM). Covered employment is unemployment-insurance wage records, so the
self-employed and uncovered work are absent (stated in LEHD materials generally; the exact sentence
not located in this document). Enumerated on 2020 blocks in version 8, 2010 blocks in version 7.
One state-year OD file: California 2022 main file 96 MB compressed (header request), so a national
year is of order a few GB before aggregation to counties.

Confidentiality [S, Census workshop slides and arXiv:2007.13275 via search]: workplace counts carry
permanent multiplicative noise; residences are partially synthetic, "sampled from a posterior
predictive distribution conditional on destination block and worker characteristics", and "for
small populations the synthetic place of residence data are relatively more noisy"; cells of zero,
one or two jobs are synthesised. So small block flows are not observations; aggregation to county
pairs averages much of it out, but small county pairs remain model output. Policy change: a
Census Director's blog of August 2026 is reported to say a Commerce order now permits only
coarsening and suppression, with noise infusion and synthetic data no longer allowed [S, not
opened]. If so, future LODES and County Business Patterns releases will change method.

### B.5 Candidate single variables

| variable | source | years | main trap |
|---|---|---|---|
| Age structure | Census PEP county age-sex files (`cc-est*-agesex`), same folders as B.1; 2010-2020 intercensal 17 MB | annual, every vintage | derived from the same cohort-component machine, including the IRS-based migration by age; age shares at t already contain migration at t; revised by vintage |
| Industry mix | County Business Patterns (2022 county file 13.7 MB zip); QCEW | annual | CBP used noise infusion from 2007 and suppression before [S]; small cells unreliable; QCEW suppresses to protect employers (rule not obtained); NAICS revisions |
| Housing permits | Census Building Permits Survey, county annual text file (2023: 375 KB) | annual | permit-issuing places only; imputation for non-reporters (documentation not opened); permits respond to expected in-migration, so they are an anticipation of the target, not a cause |
| House prices | FHFA annual county index (5.3 MB) | annual, long | 2,787 county series, so roughly one county in nine has none, and those are the small ones [S]; repeat-transactions with appraisals; missing values "initially estimated but removed" [S]; Zillow and others are not federal and carry their own terms |
| Climate | NOAA climate normals / county climate divisions | 30-year normals | constant in time: can only explain cross-section, and is collinear with region; not opened this session |
| Land area, density | Census Gazetteer counties file (142 KB zip) | per boundary vintage | area is not a head-count: declare whether density is "count" or "extra" |
| Within-county unevenness | a population grid at one epoch (the scout's warning stands: GHS-POP time series is census interpolation, use one epoch) | one epoch | the grid is built by spreading county or tract totals over built-up area, so unevenness is partly a model of the grid maker; constant in time |
| Network position | flights (BTS T-100), highways; or the flow graph itself | varies | if taken from the migration graph it is the target; if from LODES it has missing states; not documented this session |
| Social connectedness | Meta SCI county-pair file on the Humanitarian Data Exchange; 2021 release listed "Public Domain / No Restrictions"; a 2026 release exists [S] | snapshots (2016, 2021, 2026 reported) | friendships are partly produced by past migration; snapshots post-date most flow years; relative scale only |
| Prior flows | IRS or ACS, lagged | as B.2, B.3 | same instrument as the target: persistent coverage bias and persistent censoring are "predicted" along with real persistence; using ACS history to predict IRS flows (or the reverse) is cleaner |
| Distance | NBER County Distance Database (2010, 500-mile file 26 MB zip) | 2000, 2010 | centroid great-circle distance; internal distance for adjacent large counties is undefined |

### B.6 Licences and redistribution

Works of the US Government are not subject to copyright under 17 U.S.C. 105; that is the basis
usually cited. I could not open a Census, IRS or BLS page stating this for their data in their own
words [not obtained]; Census does publish a citation request and separate terms of service for its
API [S]. Morefield and Leslie restricted themselves to sources "free and publicly available without
any notable restrictions" and used IRS, ACS and Census flows, and deposited a replication archive
[F]. Hauer and Byars republished the entire IRS 1990-2010 series as one flat file [F], so
redistribution of derived IRS tables has precedent. Non-federal items need their own check: NBER
distance files, SCI (terms of the current release not confirmed), any private housing index.
Disclosure-review notes on Census files (for example "CBDRB-FY25-0078") are approvals, not use
restrictions. Recommendation: before any derived table is published, open and quote each source's
own terms page; this report does not settle it.

---

## PART C. DESIGN TRAPS, MOST SERIOUS FIRST

1. **Circularity between target and predictor.** Census net domestic migration is IRS and Medicare
   net rates applied to a household population plus group-quarters change, raked to zero (B.1).
   IRS flows, lagged Census migration, Census age structure and Census population are all outputs
   of one system. A "history" or "IRS network" variable will score well for the wrong reason.
   Literature handling: mostly ignored; DeWaard et al. note the dependence. Clean options: predict
   IRS from ACS-derived variables or the reverse; never use same-year products of the same pipeline
   on both sides; state the lineage of every variable.

2. **Censoring and processing regimes that change by year.** The pair floor is 10 then 20 returns;
   below-floor flows are folded, then deleted; matching changed in 2011-12 and 2022-23; gross rates
   swing unexplained in 2012-13 to 2017-18. A year-blocked hold-out therefore mixes "new year" with
   "new processing". And censoring makes history mechanically predictive: a pair listed last year
   had at least 20 returns. Morefield and Leslie show the floor alone reverses model rankings.
   Literature handling: Bailey et al. and most others simply analyse listed pairs (selection on the
   outcome); Morefield and Leslie truncate all sources identically; Huang and Butts avoid IRS.
   Clean options: treat absent pairs as interval-censored (0-19), fit and score with a censored
   count likelihood; keep fit and hold-out inside one disclosure regime; score destination SHARES
   given origin totals so a year-wide level artefact cancels; replicate on ACS, which has noise in
   place of censoring.

3. **Margins absorb every place-level variable.** In a doubly constrained model, or one with origin
   and destination fixed effects, any variable that is a property of one place is collinear with
   the margins and can add exactly nothing. Only pair-level variables can improve the pairing.
   So "does internal arrangement matter for flows" is ill-posed unless the arrangement variable
   enters as a pair function (a dissimilarity) or the target is the margin itself. The literature
   splits here without saying so: economists fix effects and study pairs (Bailey); physicists leave
   margins free and add place features (Deep Gravity, Robinson). The study needs two separate
   questions: what predicts the margins (net and gross totals), and given margins, what predicts
   who pairs with whom.

4. **Random-pair hold-outs leak.** With 3,100 counties, a held-out pair has both its endpoints and
   its reverse flow in training. Morefield and Leslie hold out 20 per cent of pairs at random;
   Deep Gravity holds out spatial tiles; Robinson and Dilkina hold out years. Flows are also highly
   persistent year to year (Huang and Butts put the lagged flow in every model), so a held-out
   YEAR is easy if history is allowed and is then a test of persistence, not of the variable. Use
   whole years AND whole regions, and report the persistence-only score as a reference line.

5. **Vintage revision.** A first-published year is revised in every later vintage; the last two
   years of births and deaths are rescaled copies of the last final year; the decade's base changed
   between Vintage 2024 and 2025. A "held-out year" read from a later vintage has been revised
   using information from after it; one read from its own first vintage has births and deaths that
   are mechanical. Literature handling: forecasting studies score against census counts only, which
   is why they work at decadal horizons. Options: score against both first-published and final
   values and report the difference as a noise floor; for natural increase use final values only.

6. **Zero inflation and the loss.** More than 90 per cent of pairs are zero or censored in every
   source (Morefield and Leslie). OLS on logs drops them; CPC is dominated by a few huge flows;
   MAPE explodes near zero. Morefield and Leslie's ranking of models changes with the metric.
   Literature handling: PPML (Santos Silva and Tenreyro, as cited there), zero-inflated Poisson,
   count ERGM with a non-zero term. Declare one proper scoring rule in advance (held-out Poisson or
   negative-binomial deviance with censoring) and report CPC only for comparability.

7. **Order dependence of "one extra variable".** Bailey et al. is the worked example: distance
   explains 0.61 alone and nothing after SCI; six dissimilarities explain nothing after SCI though
   Huang and Butts find them worth a quarter of all migration. Any single-variable ranking depends
   on what is already in the baseline. Literature handling: none adequate; gain importances and
   SHAP are reported without this caveat. Options: report two fixed, declared tables (each variable
   added to the baseline alone; each variable added to baseline plus history), not a Shapley
   average, and say which question each answers.

8. **Distance is already a relationship, and already a proxy.** Distance stands in for intervening
   population, for social ties (Bailey) and for information. A baseline of "counts and distances"
   is therefore not a pure-count baseline. Options: report a count-only rung (no distance) below
   it, and a counts-and-distance rung with intervening population (still only counts and
   distances) above it, so the reader sees how much of the "extra" is recoverable from counts
   arranged in space.

9. **Unit size.** County populations span more than four orders of magnitude; 20-year forecast MAPE
   is 41 per cent under 2,500 people and 16 per cent over 100,000 (Rayer). An unweighted score is
   about small rural counties, a person-weighted one about a few dozen metropolitan counties.
   Rozenfeld's result is that the conclusion about growth changes with the cell size. Report both
   weightings and at least one coarser unit (commuting zones or states) as a regime check.

10. **Group quarters and special populations.** College, prison and military counties show
    "domestic migration" from administrative reports, and students and the poor are missing from
    IRS. Flag or exclude high-group-quarters counties using GQESTIMATES; the literature rarely does.

11. **Timing misalignment.** IRS: address at filing in consecutive filing years. Census: July to
    June. ACS: residence one year before interview, pooled over five years. "Next period" is a
    different interval in each.

12. **Nulls that are too weak.** A fully shuffled variable loses its spatial autocorrelation and its
    correlation with size, so beating it proves little. Rozenfeld's shuffle is the model: state
    what the shuffle preserves. Permute within size-by-distance or size-by-region strata.

---

## PART D. RECOMMENDED SMALLEST STUDY

Two targets, kept separate because of trap 3, and both run, so the comparison is itself a result.

**D.1 Pairing target (flows given margins).**
- Unit: directed county pairs, boundaries harmonised to one vintage; Connecticut and the Alaska
  splits aggregated to a stable super-unit.
- Outcome: IRS individuals (N2), as destination shares of each origin's out-migrants, absent pairs
  treated as censored at 0-19 returns.
- Years: fit 2013-14, 2014-15, 2015-16; hold out 2016-17 and 2017-18 (one disclosure regime: floor
  20, folding). Second, declared replication on a different instrument: fit ACS 2011-2015, hold out
  ACS 2016-2020 (no censoring, sampling noise instead, margins of error used as weights).
  Region hold-out crossed with the year hold-out (whole census divisions left out in turn).
- Baseline ladder, all from counts and distances: B0 destination population only; B1 plus distance
  (flexible bins); B2 plus intervening population. Reference lines: persistence only (last fit
  year's shares) and the saturated pair effect from the fit years.
- Candidates, pair-level only, each with a stratified-permutation null (within origin size class by
  distance bin): (1) history: ACS 2006-2010 flow for the pair (different instrument, earlier);
  (2) social connectedness, earliest snapshot; (3) same state; (4) commuting tie from LODES, lagged,
  with the missing state-years declared; (5) dissimilarity in urbanisation; (6) dissimilarity in age
  structure; (7) dissimilarity in within-county unevenness; (8) dissimilarity in industry mix.
  Each gets the same number of parameters.
- Score: held-out censored-Poisson deviance per origin; skill = share of the gap between B2 and the
  saturated reference that the variable closes, minus the same for its null. CPC reported beside it.
- Two tables: each candidate on B2; each candidate on B2 plus history.

**D.2 Margin target (next-year net domestic migration rate).**
- Unit: county-year. Outcome: Census RDOMESTICMIG for year t+1, scored against both its
  first-published and its end-of-decade value.
- Years: fit t+1 = 2012-2016, hold out 2017-2019, all within the 2010s decade and read from
  Vintage 2019 or 2020 for "final" and from Vintages 2012-2019 for "first published". The 2020s
  as a second block only with the base change of Vintage 2025 declared.
- Baseline: M0 log population; M1 plus population potential (population within distance bands),
  which is the counts-and-distance rung.
- Candidates, each with a null permuted within size class by census division: (1) history: own
  net domestic migration rate at t (same-pipeline warning attached) and, separately, (2) history
  from a different instrument: ACS-based net rate from the earlier 5-year file; (3) age structure
  (share 18-34 and 65+); (4) within-county unevenness; (5) housing permits per head; (6) industry
  mix (one declared index); (7) January temperature normal; (8) group-quarters share.
- Positive control: natural-increase rate at t+1 from final-vintage values, where age structure
  must help; if it does not, the pipeline is broken.
- Score: held-out mean absolute error of the rate, unweighted and person-weighted; skill relative
  to M1, minus the null's.

**D.3 What counts as which answer.** Declared before any run, with margins the lead should set:
- "Reads essentially only counts and distance": no candidate, in either table and on both
  instruments, closes more than a small pre-stated share (suggest 5 per cent) of the baseline-to-
  reference gap above its null, in every hold-out year and region block.
- "Reads one more thing": some candidate clears a larger pre-stated share (suggest 25 per cent)
  above its null in every block and on both instruments. Then the category of that variable
  (relationship, history, internal arrangement) is the finding.
- Expected from the literature, so not by itself a finding: history and social connectedness clear
  the bar for pairing. The informative outcomes are whether any internal-arrangement dissimilarity
  survives in the second table (on top of history), and whether anything but own history clears the
  bar for the margin target.
- Kills to consider: positive control fails; a null clears the bar; results flip between IRS and
  ACS; results flip between first-published and final values.

**D.4 What this recommendation rests on.**
- Full texts: the censoring regimes and breaks (IRS guides, Pierce, DeWaard et al.); the Census
  derivation of domestic migration, the residual, vital-statistics lag and vintage revision (Census
  methodology statement and layouts); ACS zero and margin-of-error behaviour (ACS documentation);
  the choice of years inside one disclosure regime; treating censoring identically across sources
  (Morefield and Leslie); the expectation that history and social ties dominate (Bailey et al.;
  Huang and Butts); the size weighting (Rayer); the stratified null (Rozenfeld).
- Abstracts or search summaries only: LODES confidentiality method; County Business Patterns and
  house-price coverage; SCI licence and snapshot dates; the Connecticut revision figure; the
  county-level Gibrat results; the claim that no fixed-effects PPML deviance for US county
  migration is published.
- My own reasoning, not from a source: trap 3 (margins absorb place-level variables) as the reason
  to split the study in two; the censored-likelihood treatment; the two-table answer to order
  dependence; the suggested 5 and 25 per cent bars.

---

## NOT ESTABLISHED

1. IRS file sizes; which IRS filing year feeds which Census estimate year (Toukabri 2017).
2. Whether the 2012-13 to 2017-18 IRS artefact was ever explained or corrected.
3. Full texts of Howard and Shao, NBER w32123, Smith-Tayman-Swanson, Wilson et al. 2022, Eeckhout
   2004, Desmet and Rappaport, Yang et al. 2014; any PPML fixed-effects deviance for US counties.
4. Each source's own licence statement; current SCI terms; QCEW and Building Permits methods.
5. Whether ACS flow files exist beyond 2016-2020 and whether the 1990s Census files carry components.
6. The August 2026 Census disclosure-policy change and what it does to LODES and CBP.

## SOURCES OPENED THIS SESSION

Census: methods-statement-v2025.pdf; CO-EST2024-ALLDATA.pdf and CO-EST2025-ALLDATA.pdf;
co-est2019-alldata.pdf; co-est2009-alldata.pdf; https://www2.census.gov/programs-surveys/popest/datasets/
(listings); https://www.census.gov/newsroom/blogs/research-matters/2024/11/2010-2020-intercensal-population-and-housing-unit-estimates.html;
https://www.census.gov/programs-surveys/geography/technical-documentation/county-changes.html (and
.2010, .2000); 2016-2020-Migration-Flows-Documentation.pdf; LODESTechDoc8.3.pdf.
IRS: 1112, 1314, 1819, 2223 inpublicmigdoc.pdf; soi-a-inmig-id1509.pdf.
Papers: https://conference.nber.org/conf_papers/f93418/f93418.pdf; arxiv.org/pdf/2205.02347;
arxiv.org/pdf/1711.05462; arxiv.org/pdf/2012.00489; arxiv.org/pdf/1506.04889; arxiv.org/pdf/0808.2202;
arxiv.org/pdf/1111.0586; europepmc.org/articles/PMC12774388; pmc.ncbi.nlm.nih.gov/articles/PMC8974493;
demographic-research.org/volumes/vol40/40/40-40.pdf; bebr.ufl.edu Rayer (2008) JPER and Tayman,
Smith and Rayer (2011) PRPR. Abstracts via arXiv: 2412.01242, 2604.10570, 2607.00128, 2503.20989,
1407.6256, 1206.5735, 2504.01586.
