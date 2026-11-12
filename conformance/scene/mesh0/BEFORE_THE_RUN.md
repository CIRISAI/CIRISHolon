# MESH-0, the record before the run

2026-11-12. Written before the carrier is run at the frozen size and seed. The freeze is
`../MESH0_PREREG.md`. No data is fetched; there is no parse.

## The instrument

- `engine/crates/holon-scene/src/delivery.rs` (`world::delivery`): `simulate`,
  `us_population`, `hosts`, `closed_form_ppm`, `compose`. It calls the shipped
  `household::generate`, `devices::generate` and `presence::day` and nothing else.
- Carrier `engine/crates/holon-scene/examples/mesh0.rs`: 20,000 households in each of the
  five density classes, seed 20261112, one JSON object per line.
- Assembler `mesh0.py` in this directory: reads those lines and PRESENCE-0's published
  table, prints the arithmetic, the gate, the stakes and the readings. It generates nothing.

## Plants (crate tests, all passing; the crate stands at 92 tests)

| plant | test |
|---|---|
| P-SUM | `outcomes_sum_to_messages` |
| P-FLAT | `a_flat_population_is_the_closed_form` |
| P-KIND | `senders_like_their_addressee_beat_the_closed_form` |
| P-WAIT | `a_host_never_waits_longer_than_the_sleep` |
| P-PEER | `two_phones_never_on_together_deliver_nothing` |
| P-HOUSE | `the_tie_share_puts_senders_in_the_household` |
| P-JOIN, P-SEED | `the_join_and_the_seed` |
| P-WEAK | `the_composed_verdict_takes_the_lowest_and_names_it` |
| the host settings keep their promises | `host_settings` |

## What was run before this record, said plainly

- The plants call the shipped generators on populations of 300 and 2,000 households with
  other seeds, and assert structure (sums, equality, ordering), not the campaign's shares.
- A smoke run of the carrier: 200 households a class, seed 5, to check that every line
  parses and that the assembler exits. Its output was written to scratch and not read; I
  know only that the assembler returned success, which means the gate's two published
  figures were met at that size.

## Choices the freeze leaves to the instrument, fixed here

- A household's sizes are drawn by HOUSEGEN-1 for units of 2,500 people, repeated until
  the class has its 20,000 households.
- A sender inside the household is another ADDRESSEE of it (an adult with a phone); where
  there is none the sender comes from outside, and the share realised is printed.
- The hour sent is drawn evenly over the sender's on-hours of the first day; an addressee
  whose drawn sender is never on gets no message and is counted.
- Waits are looked for through the end of the second day (at most 47 hours). "Within 24"
  is therefore defined for every hour sent.
- The phone rule "0.15 off" turns off a seeded 0.15 of each person's awake hours, hour by
  hour and independently.
- TWIN's seeded 0.60 uses one seed for the campaign; the product reading uses 20 other
  seeds of the whole population.
- S1 is read at tie share 0.237 under NONE; S2 and S3 at 0.237 with every household given
  a host (the host changes only who holds, not when a phone is on).
- For S3, "met" needs at least 0.03 under each of the four join and phone settings, and
  "killed" at most 0.01 under each; anything else is neither.
- The digest (G0) folds counts of each run; it is read from the carrier built native and
  for wasm32-wasip1.
