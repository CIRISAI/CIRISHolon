# PRESENCEGEN-2 — the successor to PRESENCEGEN-1, whose gate failed: where a person is in each hour of a day, with the frame's fill at home keyed on its place in the day, and a stake that reads asleep against awake, which no earlier stake did. PREREGISTRATION

*Frozen 2026-11-12, committed alone, after one adversarial review (`PRESENCEGEN2_REVIEW.md`)
whose named fixes are applied here. It replaces `PRESENCEGEN2_PREREG_DRAFT.md`, kept. It
builds on `PRESENCEGEN1_RESULTS.md` (branch (e)) with its CORRECTION. The stakes S1 to S5,
their budgets and kills, the split of years, the seeds and the bootstrap are those of
`PRESENCEGEN1_PREREG.md` with its CORRECTION, by reference. At this commit nothing of the
successor is built or computed.*

*This is the last successor scored on 2024.* If its gate or a plant fails, or a third fill
is wanted, 2024 is spent for this question and a later year or another survey is needed.

## What has been seen

- On 2022 and 2023: the gate's numbers; the shares of each kind of day; the state shares;
  the counts of table rows; the frame's miss by hour in home asleep, home awake and
  elsewhere asleep; the four figures of the diagnosis (`presencegen1/diagnosis.json`).
- Of 2024: each diary was parsed into 24 states by PRESENCEGEN-1's run before its gate,
  and their count recorded (7,669). No other statistic of 2024 was computed by this
  campaign. PRESENCE-0's published table pools 2022 to 2024 by state, day type and hour,
  so the targets of S3 and S4 are known in pooled form, as they were at the first freeze.
- The reviewer's checks were on made-up populations and opened no survey file.

*Forecasts on the record.* Mine: the gate passes; S1 within on the eight-hour line and
open on the two-hour line; S3 met; S4 unknown; S6 met for the countdown key if it is the
one chosen. The reviewer's: the gate passes, G0c narrowly; S1 met slightly under even; S2
met; S3 met with modest confidence; S4 a coin; the band key fails S6's first reading.

## What changes from PRESENCEGEN-1

G1, the chain with one bit, is unchanged.

**The frame** (shared by G3 and G4). Kind of day and hours away (one-hour cells) as
before. The hour first left gets its own cell for the day's first hour (already away at
4) and for the second; two-hour bins after that. A frame that would run past the day's
end is clipped, and the share clipped is printed.

**The fill, G4.** Four changes, each answering one line of the diagnosis:

- **At home, keyed on place in the day.** The two-state chain (asleep, awake) by hour is
  held in eight groups: six before the frame; a day with no outing; after the frame. Two
  keys for the six groups before the frame are built, and one is chosen by the rule below:
  - *band:* the four-hour band of the hour first left;
  - *countdown:* the hours until first leaving, 1 to 5, and 6 or more.
  The 4 a.m. state at home is keyed the same way.
- **The state on return** is drawn, by the two-hour bin of the hour of return, employed
  or not. A frame clipped at the day's end has no return.
- **Within the frame** the chain gains a sixth state, home asleep. The frame's first and
  last hours are away from home by definition, for both home states.
- **The frame's first state** is keyed on whether the frame starts at the day's first hour.

**Thin rows.** A keyed row seen in under 30 diaries uses, in order: for the band, the next
earlier band; for the countdown, the next longer countdown; then the row over all people
at home (PRESENCEGEN-1's chain). The day is flagged. Keyed rows have their own table ids.

**Choosing the key (on FIT, before 2024 is loaded).** Fit on 2022 and score on 2023, and
the reverse. For each key: the largest miss by hour in home asleep, plus the miss in the
share asleep at home in the hour before first leaving, summed over the two directions.
The key with the smaller sum is G4. The other is printed on the gate and not scored on 2024.

**Declared unresolved, not built:** how long a person stays awake after returning. The
group after the frame grows as people return and holds its hourly shares; it does not
hold "awake for two hours after a late return".

G3 as built in PRESENCEGEN-1 (with the shared frame's new first cells) stays as a
comparator, scored beside G4.

## Gate

The script loads 2022 and 2023 alone until the gate and the plants have passed.

- **G0,** as frozen, for G1 and G4: sampled-minute shares within `0.01` of the minute
  shares; generated weekly mean awake share within `0.01` of the diaries'.
- **G0c (across years within FIT).** For G4: fitted on 2022 and scored on 2023, and the
  reverse, the generated shares at home asleep, at home awake and elsewhere asleep are
  each within `0.02` plus the diaries' gap in at least 22 of 24 hours, all day types
  together, in both directions.
- A reproduction check, labelled so and no gate: the same three states by hour, fitted
  and read on all of FIT, for G1, G3 and G4.

If G0 or G0c fails for G4, nothing ships (branch (e)).

## The added stake

- **S6 (asleep against awake, the joint).** On 2024, G3 scored beside G4:
  1. the share asleep at home in the sampled hour before first leaving, among days with
     an outing that starts after the day's first hour;
  2. the quartiles of hours asleep in the day (at home or elsewhere), all persons;
  3. the share at home asleep at 7 among those who first leave at 10 or later.
  Met: G4 within `0.03` on (1) and (3) and within one hour on each quartile of (2).
  *Kill:* G3 is within on all three. Then the keyed fill earns nothing over one chain at
  home on what a caller would read, and the result says so.

## What each stake judges

- **The frame:** S1's two-hour line; S3; S4. These read home against away, or home asleep
  and awake together, and return nearly the same for G3 and G4.
- **Reproduction for either frame generator,** labelled so in the result: S1's eight-hour
  line and S2's quartiles of hours away (the hours away are the frame's own table read
  back); S2's three hours in good part.
- **The fill:** S6, and S3 through its home-awake share.

## Which days each table is fitted on, under S3 and S4

| table | S3 | S4 |
|---|---|---|
| kind shares | Saturday, FIT | 2022–2023 |
| G1's 4 a.m. state | Saturday, FIT | 2022–2023 |
| the 4 a.m. state at home, by group | Saturday, FIT | 2022–2023 |
| hour first left, hours away, the fill within the frame and its first state | Monday to Thursday, FIT | 2015–2019 |
| the chain at home by group, the state on return | Monday to Thursday, FIT | 2015–2019 |
| changes of place | Monday to Thursday, FIT | 2015–2019 |

## Plants (PRESENCEGEN-1's eight are rerun; these are added)

- **P-SELECT** (synthetic days: those who leave wake two hours before leaving, those who
  stay wake at 10). One chain at home is more than `0.01` too awake over the day; each
  key of G4 is within `0.005`. The countdown key's share asleep in the hour before leaving
  is within `0.02` of the planted zero; the band's is printed.
- **P-RETURN** (the share of returns straight to sleep rises with the hour of return).
  G4 within `0.015` of the planted share in each two-hour bin of return that holds a
  twentieth of returns or more.
- **P-NAP** (a tenth of days with two outings and sleep at home between). G4's share of
  person-hours at home asleep within the frame is within a tenth of the planted share,
  and its share of days with any within `0.02`.
- **P-FIRST** (a tenth of days start away and asleep at 4). G4's share elsewhere asleep in
  the first hour is within `0.01` of the planted; G3's is short by more than `0.02`.
- **P-S3-START** (synthetic Saturdays that all start awake, weekdays that all start
  asleep). Under S3's fitting, generated Saturdays with no outing start awake in over
  `0.95` of days.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names,
and each plant is run through the shipped function.

## Branches

- **(e)** The gate or a plant fails. Nothing ships.
- **(a)** S1 met; or S2 met with G4 within budget on S1's two-hour line. The frame ships
  with G4's fill, pooled over day types if S3 is not killed. Its split of asleep and awake
  is returned `certified` if S6 is met and `unresolved` if not.
- **(b)** S1 killed and (a) not reached. G1 ships.
- **(c)** Otherwise. The better of G1 and G4 on S1 ships, marked `unresolved` for durations.
- **Thin rows.** If over a tenth of generated days of any of the eight groups are flagged,
  the result names the group and the split of asleep and awake is returned `unresolved`
  whatever S6 reads.

## What would make this vacuous

- The fill was redesigned after seeing where the last one failed on FIT. G0 on FIT is weak
  evidence for G4. G0c is across years but both years were seen in pooled form. The
  readings of 2024 are what judge it, and S6 is the one that reads what changed.
- S6 was written by the same hands that redesigned the fill, after the diagnosis. Its
  first reading is nearly what the countdown key is fitted on, one year over; its third
  is not held by either key.
- The key is chosen on FIT by a rule fixed here; a choice made on 2024 would be a second
  look.
- Everything in PRESENCEGEN-1's section of the same name.

---
witness: `CIRISHolon.SceneRhythm.conserved` (one state per person per hour)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (each hour's shares do not fix a person's day; S1, S2 and S6 read the joint)
witness: none (G0c, S1–S6 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function or the scoring that judges it.
- M-BAR-FROM-THE-READ: G0c's and S6's budgets are stated here, after the diagnosis on FIT and before any G4 exists; S1 to S5 keep budgets frozen before any diary was read. "What has been seen" lists the reads.
- M-BASE-RATE-OMITTED: G1 and G3 are scored beside G4; S6's kill is G3 doing as well.
- M-POPULATION-CHOICE: each diary of 2024 with its weight; 2020 and 2021 not read.
- M-PROVENANCE-OVERREACH: file digests are recorded; tables carry the survey's notice and years.
- M-FLOOR-UNSTAKED: P-BORROW is the plant for thin rows; the flagged share is printed per group and has a stated effect on the branch.
- M-VACUOUS-SUCCESS: §"What would make this vacuous" and §"What each stake judges".
