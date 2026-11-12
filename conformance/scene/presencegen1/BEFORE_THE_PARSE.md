# PRESENCEGEN-1, the record before the parse

2026-11-12. Written before `presencegen1.py run` reads a diary. The freeze is
`../PRESENCEGEN1_PREREG.md` with its CORRECTION of this date (hours away in one-hour cells).

## The instrument

- Generators: `engine/crates/holon-scene/src/presence.rs` (`day_with`, `Generator::Chain`
  and `Generator::Frame`), run through the example `presencegen_draw` with tables read
  from a file. The scoring script fits tables and scores; it holds no generator of its own.
- Script: `presencegen1.py` in this directory. Diary states by minute follow PRESENCE-0's
  placement rule (`../presence0/presence0.py`), the state of an hour read at a seeded minute.
- Raw files: the three ATUS 2003-2024 archives, digests equal to
  `../presence0/raw_file_sha256.txt`. Not committed; deleted after the run.

## Plants, on synthetic carriers alone

| plant | where | result |
|---|---|---|
| P-HEAP | script | PASS |
| P-MEAN | script | PASS: mean spell 5.758 planted, chain 5.767, frame 5.721 |
| P-LATE | script | FAILED as frozen (frame 0.445 against 0.539); PASS after the CORRECTION (frame 0.533, chain 0.412) |
| P-CHAIN | script | PASS: spells of two hours or less 0.442 planted, chain 0.448, frame 0.442 |
| P-BORROW | crate test `a_borrowed_row_is_flagged` | PASS |
| P-ONE | crate tests `one_state_per_hour`, `the_frame_is_respected` | PASS |
| P-SEED | crate test `one_seed_one_day` | PASS |
| P-WEEK | crate test `the_week_knob` | PASS |

The crate stands at 80 tests passing.

## Seen on P-CHAIN and said now

On days made by a chain, the frame also reproduces the spell lengths (0.442 against
0.442). The plant shows the chain is not penalised on its own ground; it does not show
the frame is. S1's kill (the chain within budget on both lines) is unchanged.

## Choices the freeze leaves to the instrument, fixed here

- A kind's WORK line: 120 minutes or more at a workplace or school; "early" is first
  there before noon (minute 480 of a day that starts at 4).
- Changes of place: over minutes, travel removed, among home, workplace and elsewhere;
  capped at 12.
- A cell with no diary writes no row; the generator's fallback applies and flags the day.
- Fit-side generation for the gate: 5 seeds per diary; scoring: 20 seeds per diary.
- Bootstrap for the diaries' own spread: 300 resamples of diaries.
