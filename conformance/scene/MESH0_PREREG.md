# MESH-0 — who holds the message, and how soon it arrives with no infrastructure at all: for a generated population, the share of messages a person's phone takes at once, holds for the sender, hands to a host at home, or leaves to someone else; the first composition of three shipped generators. PREREGISTRATION

*Frozen 2026-11-12, committed alone, after one adversarial review (`MESH0_REVIEW.md`) whose
named fixes are applied here. It replaces `MESH0_PREREG_DRAFT.md`, kept. No data is
fetched: every input is a generator or table already shipped. At this commit nothing is
built or computed.*

## Why this and not a verdict on viability

The twin's lead reports from its code that the twin has no notion of a delivery failing:
a row held for a sleeping recipient waits without limit, and the only mailbox for a
household with no home node is a seat far away. The review of the draft found that most
of what the draft staked was arithmetic on published shares. So this campaign states what
is arithmetic, stakes only what the composition adds, and adds the one reading the review
named as bearing on viability: what a mesh delivers when NOTHING but the two phones exists.

## What is built

In `holon-scene`, integers only: `world::delivery`, composing `household`, `devices` and
`presence` for one household, returning counts and the list of what they rest on.

- **Population.** 100,000 households, United States, in five equal blocks, one per density
  class of HOUSEGEN-1 (results are printed by class and pooled with equal weight; the
  class shares of a real country are not used). Sizes from HOUSEGEN-1's rows; devices and
  composition from DEVICEGEN-1 with the building unknown; each person aged 18 or over a
  day from PRESENCEGEN-2's chain, employed drawn from the shipped share (adults 18 to 64
  by the 25-to-64 band, seniors by the 65-and-over band). Seed 20261112.
- **An addressee** is a person aged 18 or over holding a smartphone. A group's phones go
  to its first members in order. Children's phones and adults with none are counted apart.
- **Messages.** Four per addressee per day. The sender is another adult; the hour is drawn
  in proportion to the sender's awake hours. With share `t` the sender is another adult of
  the addressee's household when there is one, otherwise an adult of another household on
  the same clock. `t` takes three values, always reported together: `0.237` (TIES-1, a
  share of ties), `0.29` and `0.52` (TIES-1's bracket for the household's share of
  electronic contact, low end and middle). The share realised is printed.
- **A phone is on** under two rules, always together: whenever its holder is awake; and
  the same with a seeded `0.15` of awake person-hours off (the twin's number).
- **Where a message goes.** Taken at once if the addressee's phone is on in that hour.
  Otherwise, by who may hold it:
  - *the sender holds* (no infrastructure): delivered in the first later hour in which
    both phones are on;
  - *a host at home holds:* delivered in the addressee's first later hour on, if the
    household has a host that is on; else left to someone else (the seat).
- **A host that is on** is not known for any household. Settings, always together: NONE
  (the floor); ANY-COMPUTER (a fixed line, or any laptop or desktop: its complement is
  the event DEVICEGEN-1 certified, "nothing to host on"); DECLARED (a fixed line or a
  desktop: the ceiling of what is declared able to host; its complement is wider than the
  certified event and rests on an unresolved count); TWIN (a seeded 0.60 of all
  households); TWIN-IN-DECLARED (the same number of households, drawn among DECLARED ones).
- **The next day.** Waits run into a second day of the same person. Two joins, always
  together: a day drawn afresh, same day type; and the same day again. A wait of `h`
  hours means delivered in the `h`-th hour after the hour sent; "within T" means `h ≤ T`.
- **The verdict of a composed count.** Certified ranks above unresolved above refused; the
  count takes the lowest among its inputs; an input used outside what it was scored on
  enters as unresolved. The call returns one line per input. Known at this freeze: the US
  household row is unresolved in each of the five classes; the join between two days, the household
  joint, and unbroken sleep were not scored. So every count here is unresolved, and the
  list says on what.

## Arithmetic, printed first and not staked

- **C1.** From PRESENCE-0's hourly awake shares `a(h)` alone: the share taken at once with
  an independent sender is `Σ a(h)² / Σ a(h)`. Printed before any composed count.
- **C2.** DEVICES-0's published shares (desktop 40.8 %, fixed broadband 72.8 %) put the
  twin's 0.60 between them. Printed; no stake.
- **C3.** Left to someone else under NONE is one minus taken-at-once; under TWIN it is
  that times 0.40.

## Gate

- **G0.** Each message has exactly one outcome under each setting, and the outcomes sum
  to the messages. One digest native and wasm32. The generated awake share is within
  `0.02` of `0.623`, and the share of households with nothing to host on within `0.01` of
  `0.088` (the two published figures the inputs were certified on).

## Stakes

- **S1 (does the composition add anything to the arithmetic?).** Taken at once, measured,
  minus C1, at `t = 0.237`, phone on when awake. Stake: within `0.02`. *Kill:* over
  `0.05`. Then who sends to whom (employed to employed, a household's members) moves
  delivery beyond the hourly shares, and the composition is needed. Between: neither.
  Printed by sender kind and by employed × employed.
- **S2 (a mesh of two phones and nothing else).** The sender holds; no host, no seat.
  Stake: at least `0.95` of messages are delivered within 12 hours under both joins and
  both phone rules. *Kill:* under `0.90` under any of the four. Between: neither.
  Printed: the share within 0, 1, 4, 8, 12 and 24 hours for each.
- **S3 (what a host at home buys in time).** The share delivered within 4 hours when a
  host at home holds (every household given one), minus the same when the sender holds.
  Stake: at least `0.03`. *Kill:* at most `0.01`. Then an always-on host buys custody
  while the sender is offline and almost no speed, and the case for home nodes is not
  latency. Between: neither. I do not know the outcome.

## Printed, no verdict

- The share left to someone else under each host setting, phone rule and `t`; and TWIN
  against TWIN-IN-DECLARED (how much of the twin's 0.60 lands on households with nothing
  declared).
- The product reading: left to someone else under ANY-COMPUTER, over (not taken at once)
  × (addressees with nothing to host on), with its standard deviation over 20 seeds. The
  generators tie the two only through household size and building, so this is a lower
  bound on any real tie; a setting HOSTLESS-OLD (the hostless share placed on households
  of seniors first) brackets it from the other side.
- Waits of messages not taken at once: the full distribution in whole hours under each
  join, with the chain's own unbroken-sleep quartiles printed first. The chain was not
  scored on unbroken sleep (hours asleep: 7 / 9 / 11 generated against 8 / 9 / 10).
- The seat's queue by clock hour under NONE and ANY-COMPUTER: peak over mean.
- Swings in the share left to someone else, one at a time: host setting (expected about
  `0.2`, by C3); phone rule; `t`; day type; and household members sharing one day.

## What would make this vacuous

- S2 and S3 run across the seam between two days, which no campaign scored. Both joins
  are reported; a stake that holds under one and not the other is read as neither.
- S2 is two independent adults. People who message each other keep hours more alike than
  that; the reading is then a floor on what two phones deliver.
- Nothing here says the mesh works. It says how much must be held, by whom, for how long,
  between a floor and a ceiling. A verdict needs a bound on hold time and a seat's
  capacity, which are not this repository's to state.

## Plants

- **P-SUM.** Outcomes sum to messages under each setting; a household with no addressee
  adds nothing.
- **P-FLAT** (each person awake in a random 15 of 24 hours, independently). Measured
  taken-at-once is within `0.01` of C1 computed from that population.
- **P-KIND** (two kinds of person awake in disjoint halves of the day, senders of the
  addressee's own kind). Measured minus C1 is over `0.20`.
- **P-WAIT** (each person asleep in one block of 8 hours, the same block on both days).
  With a host at home no wait is over 8 hours.
- **P-PEER** (sender and addressee awake in disjoint halves of the day). The sender holds:
  nothing is delivered within 24 hours.
- **P-HOUSE** (`t = 1`, two adults per household). Each sender is of the addressee's household.
- **P-JOIN.** The two joins give different waits on a population whose days differ, and
  the same on one whose days repeat.
- **P-SEED.** One seed twice gives the same counts; two seeds differ.
- **P-WEAK.** Inputs certified, unresolved, certified give unresolved; with a refused
  input, refused; a certified input used outside its scoring enters as unresolved. The
  call names each.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each planted population holds what its plant names, and each
plant runs through the shipped function.

## Branches

Exactly one fires.

- **(e)** The gate or a plant fails. Nothing is published.
- **(a1)** Otherwise, S1 met. The table is published with C1 named as sufficient for the
  share taken at once.
- **(a2)** Otherwise. The table is published and says the composition is needed, or that
  it is not resolved, as S1 read.

In (a1) and (a2) alike: every count unresolved with its list; S2 and S3 as read; the
swing table names what moves the seat's load most.

---
witness: `CIRISHolon.SceneRhythm.conserved` (one state per person per hour, so one outcome per message)
witness: none (S1–S3 are measured on generated populations)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function.
- M-BAR-FROM-THE-READ: bars are stated here. Known: PRESENCE-0's hourly shares, DEVICES-0's shares, TIES-1's shares, the twin's 0.60 and 0.15. Not known: any count of this composition.
- M-BASE-RATE-OMITTED: the arithmetic is printed first; host settings, phone rules, joins and tie shares are reported together.
- M-VACUOUS-SUCCESS: §"Arithmetic, printed first and not staked" and §"What would make this vacuous".

## CORRECTION 2026-11-12, after the first run stopped at its gate (branch (e)), before any stake was read

**What failed.** G0: the generated share of households with nothing to host on read
`0.1069` against the certified `0.088` (budget `0.01`). The awake share passed (`0.632`
against `0.623`), outcomes summed, and native and wasm32 output were the same bytes. The
assembler stops at a failed gate, so S1, S2 and S3 and the printed readings were not
computed; the carrier's lines for them exist in scratch and were not read.

**Why.** The freeze says ANY-COMPUTER is "a fixed line, or any laptop or desktop" and that
"its complement is the event DEVICEGEN-1 certified". That is wrong. DEVICEGEN-1's
certified event is no fixed line and no computer, where a computer is a laptop, a desktop
OR A TABLET (`DEVICEGEN1_PREREG.md`, P-CONSISTENT). The gate compared a wider event with
the certified number. The fault is this text's, written on the reviewer's third finding
without reading the definition; the generator is as certified.

**Amendment.**
- G0 reads the certified event as it was certified: no fixed line and no laptop, desktop
  or tablet. Its bar is unchanged (`0.01` of `0.088`).
- A host setting CERTIFIED-EVENT is added: a host wherever the household is not in that
  event. It is the one setting whose complement is certified, and it is generous: it
  gives a host to a household whose only computer is a tablet.
- ANY-COMPUTER keeps its definition (a fixed line, or a laptop or desktop). The sentence
  calling its complement the certified event is withdrawn; its complement is wider by the
  households whose only computer is a tablet, and rests on counts that are unresolved.
- The product reading and the seat's queue are read under CERTIFIED-EVENT as well as
  ANY-COMPUTER.

Nothing else changes: stakes, bars, kills, the population, seeds, joins, phone rules, tie
shares and branches are as frozen. S1, S2 and S3 do not read a host setting.
