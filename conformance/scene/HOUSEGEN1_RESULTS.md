# HOUSEGEN-1 — results, 2026-11-11. The generator ships, with one certified verdict and the rest unresolved: neither branch of the freeze. It beats the rule it replaces at every grain, and real tiles vary two to twelve times more than its draws.

*Prereg `HOUSEGEN1_PREREG.md` (frozen at `dac7fd7`), the record before the parse
`housegen1/BEFORE_THE_PARSE.md`, generator `engine/crates/holon-scene/src/household.rs`,
scoring `housegen1/housegen1.py`, log `housegen1/run1.log`, numbers
`housegen1/results.json`, rows and sidecar under `tables/housegen1/`. One run. Every
household scored was drawn by the shipped function.*

## S1, on tiles it never saw: NEITHER met nor killed

Rows fitted on three bands of Germany; the generator run on each tile of the other three,
with that tile's own census population.

| held-out class | consumer | generator | census | budget | gap | verdict |
|---|---|---|---|---|---|---|
| village | persons per household | 2.27 to 2.31 | 2.225 | 0.111 | 0.055 | unresolved |
| village | one-person share | 0.327 | 0.324 | 0.03 | 0.012 | **certified** |
| suburban | persons per household | 2.20 to 2.23 | 2.105 | 0.105 | 0.046 | unresolved |
| suburban | one-person share | 0.352 | 0.372 | 0.03 | 0.011 | unresolved |

- No refusal, so the kill did not fire. One of four verdicts certified, so the stake was
  not met. The freeze's two branches did not provide for this; what ships is stated below.
- The misses are the TABLE's, not the draw's: the fitted row against the held-out class
  gives the same numbers before any household is drawn. Households in the held-out bands
  are smaller than in the fitted bands by about 0.1 persons at the same density class.
- The suburban one-person share misses the certificate by a hair: a miss of `0.019` and a
  gap of `0.011` sum to `0.030` against a budget of `0.03`.
- The range in the mean is the top band generated as 6, 7 and 8. The verdict is the worst.
- Terraced, apartment blocks and dense fabric are unresolved by count in the held-out half
  (effective blocks 27.8, 8.3 and 3.2). Their generated values against the held-out census:
  mean `2.05 / 1.88`, `1.85 / 1.74`, `1.73 / 1.72`; one-person share `0.44 / 0.50`,
  `0.55 / 0.59`, `0.60 / 0.62`.

## S4, what it replaces

The twin's legacy rule by the same scoring on the same held-out classes: refused on both
consumers in village and in suburban.

## S5, per tile, the grain the twin reads (no bar)

Held-out tiles of at least 80 households; household-weighted share of tiles within tolerance.

| class | household count within 5 % | | one-person share within tolerance | |
|---|---|---|---|---|
| | generator | legacy | generator | legacy |
| village | 0.29 | 0.12 | 0.75 | 0.38 |
| suburban | 0.32 | 0.11 | 0.57 | 0.16 |
| terraced | 0.25 | 0.02 | 0.32 | 0.01 |
| apartment blocks | 0.30 | 0.00 | 0.41 | 0.00 |
| dense fabric | 0.55 | 0.00 | 0.49 | 0.00 |

- The generator is better than the legacy rule in each class and each quantity. It is not
  good: for most households the tile they live in is outside one of the two tolerances
  (both inside: `0.18` to `0.32`).

## S6, variation between tiles: MET. A tile-level effect is the next thing to build

Standard deviation across tiles of the one-person share, census over generator:
village `2.0`, suburban `3.1`, terraced `7.1`, apartment blocks `9.1`, dense fabric `11.7`.
The stake was above `1.5` in each resolved class.

- Real tiles of one density class differ from each other by about `0.10` in the one-person
  share whatever the class. Independent draws give `0.05` in village tiles and `0.005` in
  dense ones, because a dense tile holds thousands of households and the draws average out.
- So a class row is the right centre and far too narrow. What a tile needs is its own
  offset from the row, and the same holds for its household count.
- The census spread includes the disclosure noise. At 80 households that is about `0.01`,
  a tenth of what is seen.

## S2 and S3, the draw against its own row: met

| people in the unit | mean household size | one-person share |
|---|---|---|
| 10 | −3.7 to −4.1 % | +0.017 to +0.027 |
| 20 | −1.9 to −2.0 % | +0.009 to +0.014 |
| 40 | −0.9 to −1.0 % | +0.004 to +0.008 |
| 160 | −0.3 % | +0.001 to +0.002 |

- Inside both budgets at each size read, in all five classes. The unit floor is `10`
  people, the smallest size read; below it a call returns `unresolved`.
- At 320 people each size's share is within `0.0012` of its row; `0.03 %` of households
  have a size not in the row (made by the last step).
- The bias is the review's amended draw's; the draft's truncating draw is kept as a plant
  and the scoring puts it outside the budget at 8 people.

## G0 and plants

- The section law holds over 100,000 calls; 1,000 fixed calls give one digest on native and
  wasm32 (`12780190466950996998`); each shipped share lies between its two halves' shares.
- Plants: in the crate, the law, P-ZERO, P-ONES, P-FOURS, P-SEED, P-BROKEN and the
  truncating plant; in the scoring, P-BIAS, P-SHIFTED, P-BETWEEN, P-ZERO. All pass. The
  crate has 67 tests.

## What ships

`holon_scene::world::household::generate(seed, people, class, reference, in_country)` and
`expected_households(people, class, reference)`, integers only, wasm32.

| call | households per unit | composition |
|---|---|---|
| Germany, village tile, 10 people or more | unresolved | **certified** |
| Germany, suburban tile | unresolved | unresolved |
| Germany, terraced, apartment blocks, dense fabric | unresolved (read, not certified) | the same |
| Germany, under 10 people; or outside Germany | unresolved | unresolved |
| any United States call | unresolved (key is a census tract) | the same |

- Each call generates a scene whatever its verdict. No call is refused, and none is silent.
- The compiled-in rows are fitted on all cells and labelled so.

## What this is not

- Not a certificate for the twin's own tiles: the class of a tile here is this census's
  count, and the twin classes tiles from a different population model.
- Not a statement about who is home, ages, buildings, tenure or vacancy.
- Not a claim that the unresolved verdicts would be refusals or certificates with more
  regions; three bands of six is one draw of the hold-out.

## What follows from it

- **A tile-level effect.** Give each tile an offset from its class row, drawn from the
  spread measured here, and score whether held-out tiles then fall inside tolerance.
- **A region key.** The held-out bands differ from the fitted ones by about the budget.
  Rows by class and region, tested with the same hold-out, is the other obvious step, and
  the two can be compared.
- The registry row `persons.per_household` in the crate is not yet updated to point at
  this generator; its wording for an unscored row would be untrue of this one.
