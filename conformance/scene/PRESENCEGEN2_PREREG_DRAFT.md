# PRESENCEGEN-2 — DRAFT, not frozen. The successor to PRESENCEGEN-1, whose gate failed: the same question (a chain with one bit against a frame with a fill, judged on what neither holds as a parameter) with the fill at home keyed on its place in the day.

*Draft of 2026-11-12, for one adversarial review. It builds on `PRESENCEGEN1_RESULTS.md`
(branch (e)). The stakes S1 to S5, budgets, kills, the split of years, seeds and the
bootstrap are those of `PRESENCEGEN1_PREREG.md` with its CORRECTION, by reference, and are
not restated differently here. At this commit nothing of the successor is built.*

*What has been seen.* The gate's numbers on 2022 and 2023, and the diagnosis in
`presencegen1/diagnosis.json` (fit years). No diary of 2024 has been read by any script
of this campaign; no spell length, quartile, Saturday or 2015–2019 figure exists.
PRESENCE-0 read 2024's hourly shares by state and day type; they were known at
PRESENCEGEN-1's freeze too.

*Forecasts on the record.* Mine: the gate passes for the new fill; S1 is met on the
eight-hour line and close on the two-hour line; S3 is met; S4 I do not know. The
reviewer's are in the review.

## What changes from PRESENCEGEN-1

G1, the chain with one bit, is unchanged. The frame's kind, hour first left (two-hour
bins) and hours away (one-hour cells) are unchanged. The fill changes in four places,
each answering one line of the diagnosis:

- **At home, keyed on place in the day.** The two-state chain (asleep, awake) by hour is
  held in eight groups: a day with no outing; before the frame, by the four-hour band of
  the hour first left (six bands); after the frame. Its 4 a.m. state is keyed the same
  way. A row seen in under 30 diaries uses the row over all groups, and the day is flagged.
- **The state on return** is drawn, by the two-hour bin of the hour of return, in place of
  being set awake.
- **Within the frame** the chain gains a sixth state, home asleep.
- **The frame's first state** is keyed on whether the frame starts at the day's first hour.

Call this fill G4. G3 as built in PRESENCEGEN-1 stays in the crate as a comparator and is
scored beside G4 on the gate, and on nothing else if it fails it again.

## Gate

G0 as frozen in PRESENCEGEN-1, for G1 and G4: the generated weekly mean awake share within
`0.01` of the diaries' on FIT. Added, because the last gate would not have seen it:

- **G0b.** On FIT, for G4 and for G1, the generated share at home asleep within `0.02` of
  the diaries' in at least 22 of 24 hours, all day types together. (PRESENCEGEN-1's G3
  missed by more than `0.02` in 9 hours.)

If G0 or G0b fails for G4, nothing ships (branch (e)). If G1 fails G0b and G4 passes, G1
is scored and cannot ship.

## Plants (added to PRESENCEGEN-1's eight, all of which are rerun)

- **P-SELECT** (synthetic days: those who leave wake two hours before leaving, those who
  stay wake at 10). A fill with one chain at home is more than `0.01` too awake over the
  day; G4 is within `0.005`.
- **P-RETURN** (a third of returns straight to sleep). G3 has none asleep in the hour of
  return; G4 within `0.03` of a third.
- **P-NAP** (a tenth of days with two outings and sleep at home between). G4's share of
  hours at home asleep within the frame is within `0.005` of the planted; G3's is zero.
- **P-FIRST** (a tenth of days start away asleep at 4). G4's share elsewhere asleep in
  the first hour within `0.01` of the planted.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names,
and each plant is run through the shipped function.

## Branches

As in PRESENCEGEN-1, with G4 in G3's place: (e) the gate or a plant fails, nothing
ships; (a) S1 or S2 met, G4 ships, pooled over day types if S3 is not killed; (b) S1
killed and S2 not met, G1 ships; (c) neither within budget, the better ships marked
`unresolved` for durations.

## What would make this vacuous

- The fill was redesigned after seeing where the last one failed on FIT. The gate on FIT
  is therefore weak evidence for G4; the stakes on 2024 are what judge it.
- G4 has more rows than G3 (eight groups at home). A richer fill fitting the awake share
  is expected. The fallback share is printed; if over a tenth of generated days are
  flagged, the result says the keyed fill is thinner than its table.
- S1's eight-hour line now sits on a cell edge by the CORRECTION; the two-hour line always
  did. The lines were fixed before the cells moved.
- Everything in PRESENCEGEN-1's section of the same name.

---
witness: `CIRISHolon.SceneRhythm.conserved` (one state per person per hour)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (each hour's shares do not fix a person's day; S1 and S2 read the joint)
witness: none (G0b, S1–S5 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function or the scoring that judges it.
- M-BAR-FROM-THE-READ: G0b's `0.02` and 22 of 24 are stated here, after seeing G3's miss by hour on FIT and before any G4 exists; S1 to S5 keep the budgets frozen before any diary was read.
- M-BASE-RATE-OMITTED: G1 and G3 are scored beside G4.
- M-POPULATION-CHOICE: each diary of 2024 with its weight; 2020 and 2021 not read.
- M-PROVENANCE-OVERREACH: file digests are recorded; tables carry the survey's notice and years.
- M-FLOOR-UNSTAKED: P-BORROW is the plant for thin rows; the fallback share is printed.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
