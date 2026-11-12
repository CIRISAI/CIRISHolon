# PRESENCEGEN-2, the record before the parse

2026-11-12. Written before `presencegen2.py run` reads a diary. The freeze is
`../PRESENCEGEN2_PREREG.md`.

## The instrument

- Generators: `engine/crates/holon-scene/src/presence.rs`, `day_with` with
  `Generator::Chain` (G1), `Frame` (G3, one chain at home), `FrameBand` and
  `FrameCountdown` (the two keys of G4), run through the example `presencegen_draw`.
- Script: `presencegen2.py` in this directory; it imports the loader and the spell
  statistics from `../presencegen1/presencegen1.py`. That script's own `run` and `plants`
  match the crate as it stood at PRESENCEGEN-1's record and are not run again: the shared
  frame's first cells changed here.
- The script loads 2022 and 2023, chooses the key, reads the gate, and loads 2024 only
  after the plants and the gate have passed.
- Raw files as in PRESENCEGEN-1, digests equal to `../presence0/raw_file_sha256.txt`; kept
  in scratch between the two campaigns, not committed, deleted after this run.

## Plants, on synthetic carriers alone (`plants.log`)

| plant | result |
|---|---|
| P-HEAP, P-MEAN, P-CHAIN | PASS, as in PRESENCEGEN-1 |
| P-LATE | PASS: 0.539 planted; chain 0.409; frame and both keys 0.533 |
| P-SELECT | PASS: one chain at home +0.027 too awake; band +0.003; countdown +0.003. Asleep in the hour before leaving (planted 0): one chain 0.404, band 0.138, countdown 0.000 |
| P-RETURN | PASS: worst two-hour bin 0.005 from the planted share, which runs 0.15 to 0.90 |
| P-NAP | PASS: 0.0176 of person-hours planted, 0.0176 generated; 0.106 of days planted, 0.105 generated |
| P-FIRST | PASS: 0.106 planted; both keys 0.106; the plain frame 0.028 |
| P-S3-START | PASS: 1.00 of generated Saturdays with no outing start awake |
| P-BORROW, P-ONE, P-SEED, P-WEEK | crate tests, PASS; two tests added (`the_countdown_wakes_before_leaving`, `the_return_is_drawn_and_a_borrowed_group_is_flagged`) |

The crate stands at 82 tests passing.

## Said now about the plants

- P-NAP's carrier is built so that the days with sleep between outings are the only days
  within a frame at those hours. It shows the sixth state is carried; it does not show
  the fill holds the joint of a nap with a particular frame when days mix.
- P-SELECT shows the band leaving `0.138` asleep in the hour before leaving where none
  were planted. The frozen rule chooses the key on the fit years; this is what the
  reviewer forecast.

## Choices the freeze leaves to the instrument, fixed here

- A transition into an hour at home is keyed on that hour's group; the countdown of an
  hour is the hours from it to the hour first left, so "1" is the hour before leaving.
- The state on return has a pooled row over all hours of return for bins under 30 diaries.
- The frame's first state within is keyed on starting at the day's first hour; with under
  30 diaries the plain frame's row is used.
- Across the two fit years (the key's choice and G0c): 10 seeds per diary; the diaries'
  gap by hour from 200 resamples. The gate on all of FIT: 5 seeds. Scoring: 20 seeds, 300
  resamples.
- S6's first reading among generated days uses the same definition as for diaries: the
  first hour away, from the 24 states.
- A group's flagged share is over generated days that have the group.
- The one digest native and wasm32 (G0) is read on the shipped tables when the generator
  ships, as for DEVICEGEN-1; the generators hold integers alone.
