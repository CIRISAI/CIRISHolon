# HOUSEHOLD-0 (Germany) — results, 2026-11-11. Branch (a). The twin's household rule is refused on both consumers; refitting its two numbers does not rescue it (the family is wrong); and on tiles density class is a strong key, the opposite of the US reading.

*Prereg `HOUSEHOLD0_DE_PREREG.md` (frozen at `745fe78`), the record before the parse
`household0_de/BEFORE_THE_PARSE.md`, instrument `household0_de/household0_de.py`, log
`household0_de/run1.log`, numbers `household0_de/results.json`, table
`tables/household0_de/household_table.json` with its sidecar. 196,428 populated 1 km cells;
40,221,501 households. One run; the gate passed first time.*

## The verdicts (S1): expectation stands

| class of the 1 km tile | persons per household | verdict | one-person share | verdict |
|---|---|---|---|---|
| nation | 2.042 (twin 2.500) | **refused** | 0.434 (twin 0.182) | **refused** |
| village | 2.269 | refused | 0.323 | refused |
| suburban | 2.174 | refused | 0.359 | refused |
| terraced | 1.995 | refused | 0.456 | refused |
| apartment blocks | 1.812 | unresolved | 0.563 | unresolved |
| dense fabric | 1.729 | unresolved | 0.615 | unresolved |

- The budget on the mean is 5 % of the reference value, about `0.10` here. The twin misses
  by `0.23` to `0.51`.
- The two densest classes are unresolved by count: effective 25 and 8 blocks of 50 km.
  Their rows are readings. In them more than half of households are one person.

## The family or its numbers (S1b): KILLED. The family is wrong

The twin's rule with its mean set to the fitted national value (`2.086`) and its spread
fitted to the national one-person share (spread `2.1`, share matched to `0.001`):

| share of households of | the refitted rule | village | suburban |
|---|---|---|---|
| 2 persons | 0.262 | 0.367 | 0.353 |
| 3 persons | 0.180 | 0.152 | 0.139 |
| 4 persons | 0.088 | 0.105 | 0.096 |
| 5 or more | 0.049 | 0.051 | 0.041 |

- Largest error `0.105` and `0.091`; the kill line was over `0.06` in each resolved class.
  The interleaved hold-out gives the same outcome (`0.089`, `0.075`).
- A rounded normal cannot hold both a large one-person share and a larger two-person
  share. Real households peak at one and two and fall off fast; the rule spreads them.
- Scored with this campaign's port of the rule, labelled as one. Only village and suburban
  are resolved on the held-out bands; terraced misses the count by a little (27.8 of 30).

## Is density a useful key, on tiles (S2): MET, by a wide margin

Units: a held-out 50 km block's households in one class, at least 5,000 (164 units).

| rule offered | households in units within both budgets |
|---|---|
| one national rule | 5.7 % |
| a rule per density class | 42.7 % |

- The stake needed a difference of `0.05`; it is `0.37`. At unit floors of 1,000 and 20,000
  the result is the same (`0.37`, `0.39`).
- **The hold-out matters, as the review said.** With interleaved 100 km squares the class
  rule reaches 63.9 %. Holding out contiguous bands drops it to 42.7 %. The difference is
  regional structure a density class does not carry.
- **This bounds the US finding.** There a class rule barely beat a national one (18.3 %
  against 17.2 %). The US unit was a census tract and a sample; here it is a tile and a
  full count. Which of country, unit and survey makes the difference is not separated by
  these two campaigns.

## Classes differ (S3): met on both quantities

Among resolved classes persons per household spans `0.27` (bar `0.10`) and the one-person
share `0.13` (bar `0.03`). Both move in one direction with density across all five classes:
`2.27, 2.17, 2.00, 1.81, 1.73` and `0.32, 0.36, 0.46, 0.56, 0.62`.

## The US beside it (S5, a reading, no stake)

- One-person share: the direction agrees in both countries (village below suburban below
  terraced). The US step from suburban to terraced, moved to Germany three ways, predicts
  `0.374` to `0.378`; Germany reads `0.456`.
- Persons per household: the direction DISAGREES for terraced. In the US terraced tracts
  hold larger households than suburban ones; in Germany smaller.
- Village against suburban carries well in both quantities (`0.321` to `0.329` predicted,
  `0.323` read). So one step of the US shape travels and the next does not.

## Disclosure control: the check fired, in the village class

Sum of 1 km cells over the 10 km file, per column, for 10 km cells that are mostly one class:

| class | 1 | 2 | 3 | 4 | 5 | 6+ |
|---|---|---|---|---|---|---|
| village (837 cells) | 0.986 | 0.985 | 0.978 | 0.981 | **0.962** | **0.946** |
| suburban (2,022) | 0.997 | 0.996 | 0.994 | 0.994 | 0.988 | 0.983 |
| terraced (929) | 0.999 | 0.999 | 0.999 | 0.998 | 0.998 | 0.994 |
| apartment blocks (27) | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 0.999 |

- In village cells every column is low and the tail is 4 to 5 % low. So small counts are
  not kept in expectation at 1 km: households are lost, more of them large. The village row
  is published as 1, 2, 3 and 4-or-more, by the frozen rule.
- The old check (parts over total) read `0.979` for the village class and would have
  flagged it, but could not have said which columns.
- The village class's residual mean of the 6+ band reads `9.5`, outside the possible 6 to
  8, for the same reason. Nationally it reads `7.40`.
- The one-person SHARE in the village class is little affected: its column and the total
  are low by nearly the same amount.
- Dense fabric has three such 10 km cells and is unchecked.

## Buildings (S6, a reading): the first reference for households per building

| class | dwellings per building | households per building | share of buildings that are detached one-family houses | share of dwellings in buildings of 13 or more |
|---|---|---|---|---|
| village | 1.36 | 1.23 | 0.64 | 0.012 |
| suburban | 1.59 | 1.47 | 0.53 | 0.036 |
| terraced | 2.51 | 2.35 | 0.30 | 0.125 |
| apartment blocks | 5.90 | 5.62 | 0.09 | 0.299 |
| dense fabric | 10.16 | 9.81 | 0.02 | 0.462 |

- Buildings of 13 or more dwellings hold about `22` dwellings each in every class (`20.8`
  to `22.9`). The alarm (a value under 13) did not fire.
- Nationally 19.9 million buildings with dwellings and 43.1 million dwellings.
- No generator is scored against this. The twin's building rule arrives as a histogram
  from its own code and is for a later freeze.

## Checks that came out as hoped

- G0: parts over total `0.998` at 1 km and `1.000` at 10 km; the two grids' totals agree to
  `0.04 %`; persons per household `2.042`; the 6+ band's residual mean `7.40`.
- That residual read about 12 in the US survey and reads 7.4 in this full count. It is
  consistent with the review's suggestion that the US figure is an artefact of survey
  weighting. It does not establish it.
- 303 cells have no mean (0.002 % of households); 840 household cells have no population
  row (2,821 households) and are left out.

## What this is and is not

- A verdict against one census's construction of households (one per occupied dwelling;
  unmatched register persons placed by statistical criteria).
- The class of a tile is this census's count, not the twin's population grid's.
- Both budgets are the author's. S1b's bars reuse the composition budget for other shares.
- The 6+ band has no stated mean; household size and building size have no joint.
- FIT holds 64 % of households and SCORE 36 %: the six bands are equal in area, not people.

## What follows from it

- The twin's rule should be replaced, not retuned. A generator that draws household size
  from a table keyed by density class would be inside both budgets for about two fifths of
  German tiles' households where the national rule manages one in seventeen.
- What the table does not carry is regional. A region key is the next thing to add and to
  test with the same hold-out.
- The US conclusion "density is a weak key" is not general and is marked so there.
