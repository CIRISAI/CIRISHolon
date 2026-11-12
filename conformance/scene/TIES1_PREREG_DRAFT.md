# TIES-1 (United States, 2008) — a person's closest ties by kind and by how far away they live, from "same house" outward, and how often each is contacted by each means: the bridge from ties to messages. PREREGISTRATION (DRAFT)

*REVIEWED 2026-11-11 (`TIES1_REVIEW.md`): fixed in `TIES1_PREREG.md`, which replaces this text. Kept as written.*

*DRAFT of 2026-11-11. NOT FROZEN; to be attacked by a second reader first. The lead approved
the dataset by name ("pew, sure") and registered for it himself; the file is on this
machine. Read so far: the questionnaire and the data file's column names. NOT read: any
data row, the publisher's topline or its crosstabs (both ship in the same archive and hold
the answers).*

## Why

`TIES0_RESULTS.md` gave how far friendship links reach between districts and KILLED the
step below a district. `TWIN_THEORY_INPUTS.md` says the twin's locality headline turns on
where a person's groups live and on which of them the messages go to: the household and
close kin, which no source so far has touched. This survey asks each respondent to name
up to ten people (those they discuss important matters with, and those most significant
to them) and, for each: how they are connected, how far away the person lives in nine
bands from "same house" and "same block or street", and how often they are in touch by
each of eight means.

## Carrier

- **File.** Pew Internet & American Life Project, "Personal Networks and Communities"
  survey, fielded 9 July to 10 August 2008, US adults by telephone. Licence: the
  publisher's dataset terms (a royalty-free licence to copy, modify and create
  derivatives), quoted in the sidecar.
- **Unit.** A tie: one respondent and one named person. Each tie carries its respondent's
  weight. Errors by a bootstrap over RESPONDENTS (500 resamples, seed 20261119), so a
  respondent's ties stay together.
- **Kind,** made exclusive in this order from the "how connected" answers (several may be
  given) and the distance: **household** (lives in the same house); **kin** (spouse or
  partner, parent, sibling, child, other family, not in the house); **work**; **group**
  (church, association, volunteer group); **neighbour**; **friend**; **other** (advisor,
  online acquaintance, other). The table with a tie counted under EACH of its connections
  is printed beside it.
- **Distance bands as asked,** in miles: same house; same block or street; under 1; 1 to
  5; 5 to 50; 50 to 100; 100 to 500; 500 to 3,000; 3,000 or more. Published in kilometres
  as well.
- **Contact, per means:** in person; mobile voice; landline voice; card or letter; email;
  text; instant message; social-network message. The seven answers are turned into DAYS
  WITH CONTACT A YEAR by a declared scale: several times a day and once a day 365;
  several times a week 156; once a week 52; once a month 12; less often 4; never 0. A
  means not asked of a respondent (no mobile, not online) is zero for that tie.
- **Electronic contact** is the six means other than in person and post. A second,
  declared scale for ACTS counts "several times a day" as three a day; both are printed.
- **What this is.** A person's CLOSEST ties, at most ten, in one country, in 2008: before
  smartphones, when text and social-network messages were a small part of contact. The
  mix of means is a price in that regime and is not carried forward. The distances at
  which close ties live are the reading that may last.

## Readings

- **T1.** Ties by kind and distance band: shares of all ties, and within each kind.
- **T2.** Days with contact a year per tie, by kind, distance band and means.
- **T3 (the bridge).** The share of electronic contact-days, and of acts, that go to ties
  in each distance band; cumulative from the nearest outward. Beside it the same shares of
  TIES.
- **T4 (beside the twin).** The twin places a household's family in the signer's area
  (about 600 km across) and counts 65 % of acts as self or family. Printed: the share of
  household-and-kin ties, and of electronic contact with them, within 100 miles and within
  500 miles; and the share of ALL electronic contact that goes to household and kin.

## Stakes

- **G0.** Column names as in the questionnaire; weights positive; between 1.5 and 6 ties
  per respondent on average; at least 90 % of ties have a distance and at least 90 % a
  kind; each respondent's ties number at most ten. If not, nothing runs.
- **S1 (messages are nearer than ties).** The share of electronic contact-days going to
  ties within 5 miles (same house included) exceeds the share of ties within 5 miles by at
  least `0.05`. *Kill:* it is lower. Then distance thins who is close and not how often
  they are contacted, and a ties table can stand in for a messages table.
- **S2 (the household's share).** At least `0.25` of electronic contact-days go to people
  in the same house. *Kill:* under `0.10`. This is the mesh's most local traffic and the
  twin has no number for it.
- **S3 (kind beyond distance).** Among ties 5 to 50 miles away, electronic contact-days a
  year differ between kin and friends by at least 20 % of their mean. *Kill:* by under
  10 %. Then distance alone can key contact and kind need not be generated.
- **S4 (places that reach farther).** The share of ties more than 50 miles away is higher
  for respondents in urban areas than in rural ones by at least `0.03` (the survey's own
  three-way community type). *Kill:* rural higher. TIES-0 found dense districts reach
  farther on another instrument, country and year.
- **S5 (the building, a reading).** The share of ties on the same block or within a mile,
  for respondents in detached houses and in apartments.
- **Gap and verdict.** For S1 to S4 the gap is two bootstrap standard errors; a stake is
  met or killed only if the bar is cleared by more than the gap, otherwise "neither".

## What would make a stake vacuous

- The published report from this survey (2009) says text contact falls with distance and
  email does not. S1's direction is therefore partly known to the author; its size over
  all means is not.
- Closest ties are not all ties. A person's hundreds of weaker ties, which TIES-0 counted,
  live farther away and are contacted less. The two tables bracket a person's contacts
  from two sides and do not add up to a whole.
- People in the same house are named less often than they matter ("who do you discuss
  important matters with" does not always bring a spouse to mind), and electronic contact
  with someone in the room is low for a reason that has nothing to do with the mesh.
- The scale from answers to days is the author's. "Several times a day" as 365 days
  understates acts; the second scale is a guess.
- 2008. Not asked of people without the means. Telephone sample.

## Plants

- **P-KIND** (a tie marked spouse and co-worker in the same house; one marked friend and
  neighbour a mile away). Household; neighbour.
- **P-SCALE** (one tie, each answer in turn). 365, 365, 156, 52, 12, 4, 0.
- **P-NOTASKED** (a respondent with no mobile). Mobile and text contact are zero, not
  missing, and the tie is kept.
- **P-CLUSTER** (respondents whose ties all share one distance). The bootstrap's error is
  larger than one that resamples ties, by at least a third.
- **P-NEAR** (ties whose contact does not depend on distance). S1's difference is within
  `0.02` of zero and S1 is killed or neither, not met.
- **P-FAR** (contact halving with each band). S1 is met.
- **P-WEIGHT** (a group with ten times the weight and all ties far). The weighted share
  far is recovered.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** Otherwise. T1 to T4 are published with errors, the year on each table, and the
  stakes' outcomes.

---
witness: `CIRISHolon.ScenePopulation.edge_law_does_not_fix_wiring` (shares by kind and band fix no one's ties)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (ties by distance and contact by distance are different margins of one joint; T3 is the joint)
witness: none (S1–S5 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS, M-FOREIGN-DOMAIN-CORROBORATION. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through this reader before a row is read.
- M-BAR-FROM-THE-READ: bars are stated here. Known to the author from the publisher's report as quoted by a scout: mean days of contact with core ties by four means, and that text falls with distance while email does not. The topline and crosstabs in the archive are unread.
- M-BASE-RATE-OMITTED: T3 prints the share of ties beside the share of contact; P-NEAR is the null in which they are equal.
- M-POPULATION-CHOICE: each tie of each respondent, with the respondent's weight.
- M-PROVENANCE-OVERREACH: the archive's digest is recorded; the licence is quoted from the publisher's terms.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
- M-FOREIGN-DOMAIN-CORROBORATION: S4 asks whether a finding from one instrument, country and year appears in another; nothing of TIES-0 is credited to this survey or the reverse unless it does.
