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
