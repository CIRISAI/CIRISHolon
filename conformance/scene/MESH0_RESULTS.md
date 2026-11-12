# MESH-0 — results, 2026-11-12. Branch (a1). Two phones and nothing else deliver 0.86 of messages at once and 0.98 within 12 hours; a host at home buys almost no speed; what a host changes is who holds the message, and that turns on one unknown. Every count is unresolved, and no reading here says a mesh works.

*Prereg `MESH0_PREREG.md` with its CORRECTION, review `MESH0_REVIEW.md`, the record before
the run `mesh0/BEFORE_THE_RUN.md`, composition `engine/crates/holon-scene/src/delivery.rs`,
carrier `examples/mesh0.rs`, assembler `mesh0/mesh0.py`, numbers `mesh0/results.json`,
table and sidecar `tables/mesh0/`. 100,000 generated households, 154,382 addressees, four
messages each. No data was fetched.*

*Two runs.* The first stopped at its gate (branch (e)): the freeze had called a wider event
"the certified event" (a tablet is a computer in DEVICEGEN-1; my setting left it out), so
the gate compared `0.107` with the certified `0.088`. No stake was computed. After a dated
CORRECTION, committed alone, the gate reads the event as certified and passes (`0.092`).
The fault was the freeze's wording, mine.

## Arithmetic, printed first

- **C1.** From PRESENCE-0's published hourly awake shares alone, a message from an
  independent sender is taken at once with share `0.849`. People are awake `0.63` of the
  day, but they send while awake, and people are awake at the same hours.
- **C2.** Generated: a desktop in `0.402` of households, a fixed line in `0.722`, either
  in `0.791`. The twin's `0.60` lies between, as DEVICES-0 already said.

## Gate: passed (second run)

Outcomes sum to messages in every run. Awake share `0.632` (published `0.623`). Nothing
to host on, as certified: `0.092` (certified `0.088`). Native and wasm32 output are the
same bytes; digest `9036240698812664959`.

## The stakes

- **S1 (does the composition add to the arithmetic?): MET, it does not.** Taken at once,
  measured: `0.858`, against C1's `0.849` (`+0.009`; bar `0.02`). By sender and addressee:
  employed to employed `0.866`, not employed to not employed `0.843`. The hourly shares
  are enough for this share. Branch (a1).
- **S2 (a mesh of two phones and nothing else): MET.** The sender's phone holds the
  message until both are on; no host, no seat.

  | delivered within | at once | 1 h | 4 h | 8 h | 12 h | 24 h |
  |---|---|---|---|---|---|---|
  | phone on when awake | 0.858 | 0.890 | 0.931 | 0.955 | 0.988 | 0.9997 |
  | 0.15 of awake hours off | 0.729 | 0.835 | 0.911 | 0.938 | 0.979 | 0.9995 |

  The lowest of the four settings within 12 hours is `0.977` (bar `0.95`). The two joins
  between days agree to the third decimal.
- **S3 (what a host at home buys in time): NEITHER.** Within 4 hours a host at home
  delivers `+0.005` more than the sender holding when phones are on whenever people are
  awake, and `+0.016` when phones are off 0.15 of awake hours (met needed `0.03`; killed
  needed `0.01` or less in each setting). A host at home buys little speed. What it buys
  is that the sender may go offline.

## Who holds what is not taken at once (printed, no verdict)

Share of all messages left to someone outside the household (the twin's seat):

| which households have a host that is on | share left to the seat |
|---|---|
| NONE (floor) | 0.142 |
| the twin's 0.60, placed at random | 0.057 |
| the same number, placed on households that declare one | 0.050 |
| DECLARED (a fixed line or a desktop) | 0.020 |
| a fixed line, laptop or desktop | 0.008 |
| everything but the certified have-nots (ceiling) | 0.006 |

- **The unknown that decides it is the host setting:** it moves the seat's share by
  `0.136`. The phone rule moves it `0.051`; a household sharing one day `−0.010`;
  Saturday `+0.006`; the tie share `0.000`.
- **The twin's 0.60 puts 0.21 of its hosts on households that declare nothing able to
  host.**
- **Product reading.** The share left to the seat is `1.012 ± 0.015` times (not taken at
  once) × (addressees with no host): independent, as the generators are built. Placing
  the hostless on households of seniors first multiplies the seat's share by `2.4`. That
  is the bracket; the truth is not measured.
- **Waits when something holds the message:** quartiles 2, 4 and 8 hours; `0.23` wait
  over 8 hours and `0.045` over 12. These cross the seam between days, which was not scored.
- **The seat's queue** peaks at twice its mean over the day.
- **Counted apart:** 25,576 phones held by children; 37,564 adults drawn with no phone
  (`0.196` of adults). DEVICES-0 read 0.999 smartphones per adult on average, so the
  generator gives some adults two and some none. One adult in five with none looks high
  and was not checked here.

## What this says about viability, and what it does not

- For person-to-person messages between people on the same clock, the phones alone do
  most of the work: `0.86` at once, `0.98` within half a day. A home node or a seat is
  needed for the sender to go offline, not for the message to arrive soon.
- How much lands on a far seat runs from `0.006` to `0.142` of messages, and the whole
  range is one unknown: which households have something on all day that can hold a
  message and be reached. No reference exists for it.
- Not read here: reachability from outside (address translation), time zones, group and
  public rooms (which carry most deliveries in the twin), media, and anything a user
  would call too slow. Every count is `unresolved`; the list of what each rests on is in
  the table.

## NOTE 2026-11-12, from the twin's lead: what "the twin's 0.60" is on its default street

This file compared the seat's share under "the twin's 0.60, placed at random" (0.057) and
said that draw puts 0.21 of its home nodes on households that declare nothing able to
host. The twin's lead reports from its code that this describes the twin's OLDER tile
path alone. On its default street a household must be able to host AND pass the 0.60
draw; hosting by a gateway is possible on 0.716 of households there, so the home-node
share is 0.6 × 0.716 = 0.43, not 0.60, and the nearer row of the table above is "the same
number placed on households that declare one" (0.050). The open reference is unchanged:
the share of households that offer to host at all.

The twin has taken the three changes in on a candidate build (a phone's reachability from
its person's generated day; counts of taken at once, held at home, held at the seat; a
hold histogram; "late" after 12 hours as a readout bound of its own choosing). Its first
probe reads 0.77 taken at once on one city against this file's 0.858 and 0.729; it has
not reconciled the difference and claims no agreement.

**Sending by hour: neither side has a reference.** The twin sends on an assumed curve by
clock hour (peak 1.6 times at 19:30) and keeps the curve's total, so late in the evening,
with few people awake, the rate per awake person about triples. This campaign drew the
hour sent evenly over the SENDER'S awake hours. Neither is measured. This campaign's
choice has no free number and cannot put a message on a sleeping sender; that is all
that can be said for it. A reference for messaging by hour has not been found.

## NOTE 2026-11-12, from the twin's lead: why the twin's waits were shorter, and a definition a viability test has to fix first

The twin adopted the sending recommendation (a per-person day leads where one exists; its
assumed curve is a second, labelled setting; the page says sending by hour has no
reference). It then switched its street to this campaign's assumptions one at a time,
nothing tuned, against this file's 0.858 taken at once, a 4-hour median wait and 0.23
waiting over 8 hours:

| the twin's street, with | at once | median wait | over 8 h |
|---|---|---|---|
| the page as it runs | 0.730 | 1.0 h | 0.000 |
| each message followed a further 24 h | 0.730 | 1.1 h | 0.030 |
| only messages to people | 0.615 | 1.1 h | 0.030 |
| the addressee a person, received at their PHONE | 0.806 | 2.8 h | 0.171 |
| all of this campaign's assumptions together | 0.838 | 2.1 h | 0.114 |

- **The twin's short waits came from what it addresses: a device.** 65 % of its messages go
  to another device in the sender's household, which can be a laptop or the always-on
  home node, and a held row waits only for a mailbox to pass it on. Requiring the
  addressee's phone, as this campaign does, closes most of the gap in "at once" and about
  half of the gap in waits.
- **Unexplained after each switch:** a median of 2.1 to 2.5 hours against 4, and 0.11 to
  0.13 over 8 hours against 0.23. Its untested guesses: one street of about 40 households
  on one weekday against 100,000 here, and four messages an addressee here against its
  flat rate per awake device.
- **"Delivered to a person" against "delivered to a device" is the definition to fix
  first.** The real system delivers to a device, and a person may read on any of theirs.
  This campaign counted a person at their phone; that is one choice, and it is the choice
  that makes the waits long.
- **Adults with no phone** are 0.286 of adults on that street in the twin, against 0.196
  here. Both are far above what a survey of ownership would give, and both come from how
  the device generator's phones per household are shared among its adults. Unchecked,
  and it moves each share in this file.

## A dated note, 2026-11-12: the long waits are the addressee asleep, and "no phone" is a defect

Eric's objection: a message that reaches a person's laptop shows on their phone as soon as
the phone is on, so waits of many hours do not add up. A diagnosis on the same generated
population (`examples/mesh0_dig.rs`, seed 20261112, 100,000 households, 617,334 messages;
output in `tables/mesh0/mesh0_dig.jsonl`). Exploratory: no stake was set on any of it.

- **In this campaign "the phone is on" was coded as "the person is awake".** A real phone
  stays on overnight and takes the message while its owner sleeps. So the waits in this
  file are the time until a PERSON can read, and they were written up as a delay of
  delivery. That wording was wrong; the objection is right.
- **The waits over 8 hours are nights.** 0.142 of messages are held. Of the held, 0.231
  wait over 8 hours, and 0.810 of those were sent between 20:00 and 04:00 to someone
  already asleep (sent at 21:00 the mean wait is 9.6 hours; sent at 09:00 it is 2.5).
  0.74 of held messages are read between 05:00 and 11:00. No mesh can shorten these.
- **What the mesh itself adds is small.** With any holder that is on (a laptop, a home
  node, a seat) it adds 0 hours by construction: the message is there when the addressee
  wakes. With no holder, the sender's phone must be awake at the same hour as the
  addressee's, and among held messages that adds 0 hours for 0.787, 1 hour for 0.064,
  2 to 3 for 0.085, 4 to 8 for 0.052, over 8 for 0.011, and never within 48 hours for
  0.0004.
- **The measure a viability test should use** is the hours a message waits BEYOND the
  addressee's own next waking hour. On that measure a holder scores 0 and two phones alone
  are late by over 8 hours for about 0.0015 of all messages (0.142 x 0.011).
- **"Adults with no phone" at 0.196 is a defect in how phones are handed out, and it is
  confirmed.** The device generator gives 0.994 phones per adult in all (0.860 held by
  adults, the rest by children), which is what its reference fixes: a mean. It draws 0, 1
  or 2 for each person independently, and that spread has no reference. The result: 0.281
  of adults who live alone have no phone, and 0.404 of people over 64 who live alone.
  This campaign drops those adults as addressees and senders, so each share in this file
  was measured on phone holders only, with older people under-counted. Unfixed here; a
  fix needs a reference for ownership by age, which is a fetch and is not approved.
- **Still open against the twin:** its median of 2.1 hours against 4 here. Untested
  guesses: this campaign counts whole hours, and the chain's nights run long in the tail
  (hours asleep at quartiles 7, 9, 11 against the diaries' 8, 9, 10).

## CORRECTION 2026-11-12, later the same day: "adults with no phone is a confirmed defect" was wrong in kind and overstated

The note above called 0.196 "a confirmed defect", set it against "what a survey of
ownership would give", and asked for a fetch. Preparing that fetch showed three faults in
the note. Its diagnosis of the waits is not touched.

- **Wrong kind.** The surveys I had in mind count any telephone (a scout of published
  pages today: about 1 % of US adults have no phone at all, about 2.6 % no mobile). This
  campaign's phone is a SMARTPHONE, the only kind that can run the mesh. For that kind
  the reference is already in this repository (`tables/devices0_us/device_table.json`,
  the 2020 household energy survey read by DEVICES-0): 0.230 (se 0.007) of one-person
  households have no smartphone. No fetch was needed to see it.
- **Overstated.** Against that reference the generator is wrong, by less than the note
  implied and in both directions (`examples/mesh0_dig.rs`, 100,000 households):

| persons in household | no smartphone, generated | reference (se) | smartphones, generated | reference |
|---|---|---|---|---|
| 1 | 0.297 | 0.230 (0.007) | 0.76 | 0.88 |
| 2 | 0.076 | 0.105 (0.005) | 1.63 | 1.73 |
| 3 | 0.018 | 0.060 (0.007) | 2.45 | 2.48 |
| 4 | 0.006 | 0.037 (0.004) | 3.10 | 3.03 |
| 5 | 0.007 | 0.039 (0.007) | 3.68 | 3.58 |
| 6 | 0.003 | 0.068 (0.016) | 4.38 | 3.79 |
| 7 or more | 0.000 | 0.094 (0.021) | 6.04 | 4.24 |

- **What the defect is.** Phones are drawn for each person independently. That gives too
  many one-person households with none (+0.067) and too few larger households with none
  (0.018 against 0.060 at 3 persons): in the reference, going without clusters by
  household. DEVICEGEN-1 had already reported the one-person miss (0.74 against 0.86).
  Whether the generated share of ADULTS without a smartphone (0.196) is high or low is
  not known: the reference counts households, and no person-level figure for 2020 is held.
- **The regime.** The table is 2020. The same survey's 2024 reading has 8.4 % more
  smartphones per household, and a 2026 poll read on its page today gives 91 % of adults
  owning one (79 % of those over 64). A delivery share for "now" needs the year stated.
- **What follows.** The fix is an amendment to the device generator scored against the
  none-by-size column already held, with the year named; it is not a fetch. Each share in
  this file stays as measured, on smartphone holders as generated for 2020.
