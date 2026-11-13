# BUILDGEN-1, the record before the parse

2026-11-12. Written before any row of the survey is read by this campaign. Freeze:
`../BUILDGEN1_PREREG.md`. Script `buildgen1.py`; carrier `examples/buildgen_draw.rs` over
`holon_scene::buildings::generate_with`; plants in `plants.log` (9 of 9 pass on synthetic
surveys).

## Choices the freeze left open, fixed here before the read

- **A split-level house** takes storeys level 2, with its flag set. The freeze made
  split-level a flag and did not say what ordinal level such a house has.
- **Non-electric energy** is the survey's total less its electricity in thousand Btu
  (`TOTALBTU` less `BTUEL`), and is zero if under 1 thousand Btu.
- **Cooling energy** is the electricity used for space cooling (`BTUELCOL`); heating is
  the total for space heating (`TOTALBTUSPH`).
- **D2′'s area terciles** are FIT's weighted terciles among detached houses, applied to
  survey and generated records alike, so that no cut point comes from SCORE.
- **P-MARG on the real tables** is read on the population's marginal of each attribute
  (type, vintage, fuel, wall as categories; area and electricity in FIT's quintile
  bands), FIT against generated.
- **Pairs with non-electric energy** are fitted among records where it is positive, since
  the zero is drawn apart from the copula.
- **A type with under 100 FIT records** keeps the identity matrix and says so.

## Departures from the freeze, said before the read

- **The latent correlations are fitted by bisection through the SHIPPED function** (the
  draw binary), not through a Python mirror of it. This is the stricter of the two the
  review named; there is no mirror to disagree with the generator.
- **P-DET is run as "the same digest twice, native"** here. The comparison of the native
  digest with wasm32 needs the compiled tables and is made at shipping, before anything
  is called shipped; the crate builds for wasm32 now and uses integers alone.
- **A fault found by a plant and fixed before the read.** The first draw stretched each
  33-knot table across the whole range of the uniform, which put 19.4 % of draws under
  the 20th percentile. P-MARG caught it on a large synthetic survey (excess 0.0053
  against 0.005). The draw now holds the first knot through the lowest 1 % and the last
  through the highest 1 %, as "knots from the 1st to the 99th percentile" should have
  been read. Three plants also failed at first for faults in the SYNTHETIC surveys (too
  few records for a bar of 0.03; a planted tail drowned by random heating energy), and
  the synthetic surveys were enlarged and corrected. No bar was changed.

## What has been read

The codebook, and nothing of the file. The file is fetched after this commit and must
carry the recorded digest.
