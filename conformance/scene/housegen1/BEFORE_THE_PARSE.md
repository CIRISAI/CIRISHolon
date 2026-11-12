# HOUSEGEN-1 — the record made before any census file is parsed for it (2026-11-11)

- **The draw exists** (`engine/crates/holon-scene/src/household.rs`) with the plants that
  live in the crate as tests: the section law over 100,000 calls, P-ZERO, P-ONES, P-FOURS,
  P-SEED, P-BROKEN, and the truncating plant draw. The crate has 63 tests, all passing.
- **Native against wasm32:** the 1,000-call digest is `12780190466950996998` on both
  (`wasm_digest_shim.rs`, run under node).
- **The scoring's plants** pass (`plants.log`): P-BIAS, P-SHIFTED, P-BETWEEN, P-ZERO.
- **The carrier** is the built example `housegen_draw`: each household scored is drawn by
  the shipped function, not by a port in the scoring script.
- **Stated before the read, not in the freeze.**
  - A tile's seed is its index among all household cells sorted by grid coordinate.
  - A tile's people is the census population count rounded to an integer.
  - S5's tile floor uses the published household total; its census one-person share is the
    one-person column over the sum of the six.
  - S6 is computed on the same tiles as S5; its ratio of household-count spread is printed
    as well and is not staked.
  - S3 counts a household whose size is not in the row (made by the last step) as "other".
  - The compiled-in rows, `generate` and `expected_households` are written AFTER the read,
    because they hold its numbers; the draw they call is the one tested here.
- **What is already known and not used by a stake:** the all-cell class values and the
  national values, from the reference campaign.
