# TIES-1 (United States, 2008) — a person's closest ties by where they live, from "same house" outward, by relation, and by how often each is contacted by each means: the bridge from ties to messages, with who is missing from it measured. PREREGISTRATION

*Frozen 2026-11-11, committed alone, after one adversarial review (`TIES1_REVIEW.md`) whose
named fixes are applied here. The lead approved the dataset by name and registered for it
himself. It replaces `TIES1_PREREG_DRAFT.md`, kept. Read so far: the questionnaire and the
data file's column names. NOT read: any data row, or the publisher's topline and
crosstabs, which ship in the same archive and hold the answers.*

*Known before the read, so not staked: the publisher's 2009 report says contact by text
falls with distance and by email does not. The reviewer's guesses: half to two thirds of
partners named and well under a quarter of children; ties within 5 miles near `0.45` to
`0.5`; 15 to 30 % of near ties under a mile. The author has no forecast for S0, S3, S6
or S7.*

## Why

`TIES0_RESULTS.md` gave how far all friendship links reach between districts and KILLED
the step below a district. The twin's headline turns on the household and close kin,
which no source so far has touched (`TWIN_THEORY_INPUTS.md`). This survey asks each
respondent to name up to ten people and, for each: how they are connected, how far away
the person lives in nine bands from "same house", and how often they are in touch by each
of eight means.

## Carrier

- **File.** Pew Internet & American Life Project, "Personal Networks and Communities",
  fielded 9 July to 10 August 2008, US adults by telephone. Licence: the publisher's
  dataset terms, quoted in the sidecar.
- **Unit.** A tie: one respondent and one named person, with the respondent's weight. An
  empty name slot is not a tie. Shares are of ties; the per-respondent mean is printed
  beside. Errors by a bootstrap over RESPONDENTS within sample frame (500 resamples, seed
  20261119).
- **Two keys per tie, no exclusive order.** Where: same house or not. Relation: partner,
  child, parent, sibling, other family, non-kin. Flags that may overlap: co-worker,
  neighbour, group member, online.
- **For the twin's kinds:** Household is same house; Kin is family elsewhere;
  CloseFriends is non-kin; Work, Neighbourhood and Interest appear only as FLOORS (the
  close members of those groups); School, Public and OwnDevices are not in this
  instrument, and the table says so.
- **Distance bands as asked,** in miles: same house; same block or street; under 1; 1 to
  5; 5 to 50; 50 to 100; 100 to 500; 500 to 3,000; 3,000 or more; also in kilometres.
- **Contact.** Days with contact a year, on a declared scale: several times a day and
  once a day 365; several times a week 156; once a week 52; once a month 12; less often
  4; never 0. ACTS: the top answer at 1, 3, 10 and 30 a day. Written means (email, text,
  instant message, social-network message) and voice (mobile, landline) are printed apart
  and summed; in person and post are printed and are not electronic.
- **Two populations for each table:** each respondent, a means they do not hold counted
  as zero (the 2008 reading); and respondents who hold the means (nearer the mesh's
  regime). "Don't know" and "refused" are missing, not zero.
- **What may last and what may not.** Tagged on each table. May last: where closest ties
  live, by relation; the in-person gradient. A 2008 price, not to be carried: each level
  of electronic contact, the mix of means, and the bridge ratio itself.

## Stakes

- **G0.** Column names as in the questionnaire; weights positive; the reader reproduces
  the publisher's own per-respondent counts of ties by distance band exactly; the number
  of respondents is printed; what `form`, `samptype`, `rbcounty` and `density3` hold is
  described from their own values before any tie is tabulated; between 1.5 and 6 ties per
  respondent on average; at least 90 % of ties have a distance and a relation. Also
  printed: respondents at the cap of five names in either question, respondents with no
  names, and the answered rate of distance and contact by name position. If the
  reproduction fails, nothing runs.
- **S0 (who is missing).** The share of respondents married or living with a partner who
  named a same-house partner; named same-house adults over (adults in the household
  less one); named same-house children over children in the household. Reported. The
  household rows are printed as named (FLOOR) and with each unnamed co-resident added at
  the mean contact of named co-residents of that relation (CEILING).
- **S1 (messages are nearer than ties).** Among ties outside the house: the share of
  written electronic contact-days going to ties within 5 miles exceeds their share of
  ties by at least `0.05`, on the days scale and at each acts scale. *Kill:* lower on the
  days scale. The direction is known from the publisher's report; the size is the reading.
- **S2 (the household's share).** The household's share of electronic contact as a
  floor-to-ceiling bracket at each scale. Met: the floor on the days scale is at least
  `0.15`. *Kill:* the ceiling at 30 a day is under `0.15`.
- **S3 (relation beyond distance).** Among respondents who hold mobile, text and email,
  and ties 5 to 50 miles away: total electronic contact-days of kin and of non-kin differ
  by at least 20 % of their mean. *Kill:* by under 10 %. Written and voice are printed
  apart whatever the total does.
- **S4 (places that reach farther).** If the file carries a county code or a density: the
  slope of the share of ties beyond 50 miles (outside the house) on that density is
  positive by more than its gap. *Kill:* negative by more than its gap. Otherwise the
  difference between urban and rural respondents, a reading with its gap.
- **S6 (below a district).** Among ties outside the house and within 50 miles, the share
  on the same block or under a mile. Beside it the value a 1/distance law on uniform
  population gives, and TIES-0's bracket. A reading: the measured size, for close ties,
  of what that bracket could not say.
- **S7 (does the bridge grow with the means).** T3's near excess (S1's quantity) for
  respondents who use four or more of the electronic means against those who use two or
  fewer. A reading: if it grows with adoption the 2008 ratio is a floor for today, if it
  shrinks a ceiling.
- **S8 (close against all, a reading).** The share of close ties outside the house that
  lie within 50 miles, beside the published `0.554` of ALL friendship links within 50
  miles (another instrument and year). The two are an envelope.
- The **gap and verdict.** The gap is two bootstrap standard errors. A stake is met or killed
  only if its bar is cleared by more than the gap; otherwise "neither".

## Readings published

- **T1.** Ties by where, relation and distance band; flags beside.
- **T2.** Days with contact a year per tie by relation, distance band and means.
- **T3 (the bridge).** The share of written, of voice and of all electronic contact going
  to each distance band, cumulative outward, beside the share of ties; at each scale and
  for both populations.
- **T4 (two bounds beside the twin).** The share of electronic contact that goes to
  household and kin, as named and with the ceiling. It is NOT a score against the twin's
  "65 % of acts self or family": weak ties are absent here, unnamed co-residents are
  absent, and no survey records acts to oneself.

## What would make a stake vacuous

- Closest ties are not all ties; TIES-0 counted all links. The two do not add to a whole.
- The ceiling in S0 gives an unnamed co-resident the contact of a named one, who was
  named because they matter more. It is an upper bound for that reason.
- Electronic contact with someone in the same room is low for reasons that have nothing
  to do with a mesh.
- Respondents are adults; a teenager's own messaging is absent.
- 2008; telephone frames; the bars are the author's.

## Plants

- **P-KEYS** (a same-house spouse who is also a co-worker; a friend and neighbour a mile
  away). Same house, partner, flag co-worker; elsewhere, non-kin, flag neighbour.
- **P-SCALE** (one tie, each answer in turn). 365, 365, 156, 52, 12, 4, 0 days; the top
  answer gives 365, 1,095, 3,650 and 10,950 acts.
- **P-NOTHELD** (a respondent with no mobile). Zero in the first population; left out of
  the second.
- **P-DK** ("don't know" on one means). Missing, not zero.
- **P-EMPTY** (a respondent with three names). Three ties.
- **P-COVER** (a married respondent with two adults and two children at home who names
  one friend). S0 reads none of one partner and none of two children; the ceiling adds
  three ties.
- **P-TOP** (household ties all in the top answer). The household's share rises with the
  acts scale and S2 reports the range.
- **P-CLUSTER** (respondents whose ties all share one distance). The bootstrap over
  respondents gives a larger error than one over ties.
- **P-WEIGHT** (a group with ten times the weight and all ties far). The weighted share
  far is recovered.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** Otherwise. T1 to T4 are published with errors, each tagged with its year and
  with whether it may last, and the stakes' outcomes.

---
witness: `CIRISHolon.ScenePopulation.edge_law_does_not_fix_wiring` (shares by relation and band fix no one's ties)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (ties by distance and contact by distance are margins; T3 is the joint)
witness: none (S0–S8 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS, M-FOREIGN-DOMAIN-CORROBORATION. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through this reader before a row is read.
- M-BAR-FROM-THE-READ: bars are stated here. Known to the author from the publisher's report as quoted by a scout: mean days of contact with core ties by four means, and that text falls with distance while email does not. The topline and crosstabs in the archive are unread.
- M-BASE-RATE-OMITTED: T3 prints the share of ties beside the share of contact; S6 prints the uniform-population value beside the read one.
- M-POPULATION-CHOICE: each tie of each respondent with the respondent's weight; S0 measures who the name questions leave out.
- M-PROVENANCE-OVERREACH: the archive's digest is recorded; the licence is quoted from the publisher's terms; the reader must reproduce the publisher's own counts.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
- M-FOREIGN-DOMAIN-CORROBORATION: S4, S6 and S8 set this survey beside TIES-0 and a published US figure; nothing of one instrument is credited to another unless the reading agrees.
