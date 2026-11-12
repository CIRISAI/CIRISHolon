# TIES-1 (United States, 2008) — results, 2026-11-11. Branch (a). A quarter of a person's closest ties live in the same house and three quarters within 50 miles; electronic contact follows ties band for band; the household takes 29 to 61 % of it; and near ties are seven times more concentrated than a distance law on even ground allows.

*Prereg `TIES1_PREREG.md`, the record before the parse `ties1/BEFORE_THE_PARSE.md`,
instrument `ties1/ties1.py`, log `ties1/run1.log`, numbers `ties1/results.json`, table
`tables/ties1_us/ties_by_kind_table.json` with its sidecar. 2,512 respondents, 7,059 ties.
One run. The reader reproduces the publisher's own per-respondent counts for all 2,512.*

## What it is

A person's CLOSEST ties: up to ten people they discuss important matters with or who are
most significant to them. US adults by telephone, summer 2008. 14 % named nobody. Not weak
ties, not school, not public follows, not one's own devices.

## Who the name questions leave out (S0)

- 58 % of married or partnered respondents named a partner living with them.
- Named adults at home are 47 % of the other adults at home.
- Named children at home are 16 % of the children at home.

(Households of three or more are counted as three, so these are overstated.) Each
household figure below is therefore given twice: as named, a FLOOR; and with unnamed
co-residents added at the contact of named ones, a CEILING.

## Where closest ties live (T1; may last)

| | share of ties | cumulative |
|---|---|---|
| same house | 0.237 | 0.237 |
| same block or street | 0.033 | 0.270 |
| under 1 mile | 0.040 | 0.310 |
| 1 to 5 miles | 0.123 | 0.433 |
| 5 to 50 miles | 0.317 | 0.750 |
| 50 to 100 miles | 0.048 | 0.798 |
| 100 to 500 miles | 0.089 | 0.887 |
| 500 to 3,000 miles | 0.087 | 0.974 |
| 3,000 miles or more | 0.027 | 1 |

By relation: partner 15 % of ties (82 % of them in the house); child 19 %; parent 11 %;
sibling 11 %; other family 12 %; non-kin 32 % (4 % in the house, 46 % at 5 to 50 miles).
Flags: co-worker 9 % of ties, group member 6 %, neighbour 2 % (three quarters of those on
the block or under a mile).

## The bridge from ties to contact (T3; a 2008 price)

Share going to each band (days with contact, each respondent):

| | same house | block | under 1 mi | 1–5 | 5–50 | 50–100 | 100–500 | 500–3,000 | 3,000+ |
|---|---|---|---|---|---|---|---|---|---|
| ties | 0.237 | 0.033 | 0.040 | 0.123 | 0.317 | 0.048 | 0.089 | 0.087 | 0.027 |
| written contact | 0.236 | 0.029 | 0.039 | 0.149 | 0.322 | 0.041 | 0.073 | 0.082 | 0.028 |
| voice | 0.325 | 0.036 | 0.043 | 0.142 | 0.294 | 0.036 | 0.062 | 0.046 | 0.016 |
| in person | 0.437 | 0.046 | 0.050 | 0.132 | 0.258 | 0.026 | 0.026 | 0.019 | 0.005 |

- **Written contact follows ties band for band**, to within `0.03` in any band. Voice
  leans to the house; meeting in person leans to it far more.
- Among respondents who hold a mobile, text and email the rows are the same to `0.01`.

## Stakes

- **S1 (messages are nearer than ties): neither met nor killed.** Outside the house,
  ties within 5 miles get `0.285` of written contact-days against `0.257` of ties: a
  difference of `0.028` with a gap of `0.030`. Counting "several times a day" as 3, 10 or
  30 a day raises it to `0.046`, `0.059`, `0.064`, each with a gap as large. The stake
  needed `0.05` beyond the gap. So for written contact a table of TIES can nearly stand in
  for a table of messages, and the survey cannot say they differ.
- **S2 (the household's share): MET.** Of electronic contact with closest ties, the
  household takes at least `0.29` as named (gap `0.02`; the bar was `0.15`), and `0.52`
  with unnamed co-residents. At 30 acts a day for the top answer: `0.38` to `0.61`.
- **S3 (relation beyond distance): neither.** At 5 to 50 miles kin and non-kin have the
  same total electronic contact (534 and 532 days a year; difference `0.5 %`, gap
  `11 %`). The mix differs: kin by voice (+26 %), non-kin in writing (+28 %).
- **S4 (places that reach farther): not replicated.** The file's density code has five
  unlabelled levels. The share of ties beyond 50 miles is `0.35, 0.33, 0.32, 0.35, 0.29`
  from level 1 to 5, and `0.35` rural, `0.33` suburban, `0.32` urban. No rise with
  density; if anything a fall, inside the gap. TIES-0's "dense districts reach farther"
  (all links, Germany, 2026) does not appear for closest ties in the US in 2008. The two
  instruments may both be right: weak ties may be what reaches farther from dense places.

## Readings

- **S6 (below a district).** Of closest ties outside the house and within 50 miles,
  `0.142` are on the same block or under a mile (`0.064` on the block alone). A
  1/distance law on evenly spread population puts `0.02` there. **Near ties are seven
  times more concentrated than that law allows.** This is the measured size, for closest
  ties, of what TIES-0's bracket (`0.004` to `0.48`) could not say.
- **S8 (close against all).** `0.672` of closest ties outside the house lie within 50
  miles, against the published `0.554` of all friendship links. Closest ties are nearer.
- **S7 (adoption).** The near excess of electronic contact is `0.064` for respondents
  using four or more means and `0.056` for those using two or fewer. It did not grow with
  adoption, so nothing here says the 2008 bridge has moved; nothing says it has not.
- **Building.** Ties on the block or under a mile: `0.089` from detached houses, `0.087`
  from apartments. No difference.
- **Days with contact a year by means and distance (T2):** in person falls from 358 in
  the house to 34 at 3,000 miles; mobile from 227 to 91; text from 75 to 35; email is
  flat at 47 to 71.

## Two bounds beside the twin (T4; not a score)

- Household and kin take `0.70` of electronic contact with closest ties as named and
  `0.80` to `0.83` with unnamed co-residents. The twin's "65 % of acts are self or family"
  is a different quantity (it includes weak ties in its denominator and acts to oneself);
  these are bounds on the family share among CLOSE ties only.
- Kin who live elsewhere: `0.68` of those ties are within 100 miles and `0.82` within 500;
  by electronic contact `0.75` and `0.86`. The twin places a family wholly in one area
  about 600 km across. By this survey between one in seven and one in four of a person's
  electronic contacts with kin elsewhere leave such an area.

## What this is not

- Not today. Each level of electronic contact and the mix of means is a 2008 price.
- Not weak ties, which are most of a person's links and reach farther (TIES-0).
- Not a measurement of the household: the ceiling gives an unnamed co-resident the
  contact of a named one, who was named because they matter more.
- Not children's or teenagers' own contact.

## What follows from it

- The twin can place a person's close ties from T1 by relation and band, including the
  house, the block and the mile, which no other source here reaches.
- For written messages it may use the ties table as the messages table, with the
  household apart; the survey gives no licence to weight near ties more.
- Family should not be placed wholly in the signer's area: about a fifth of kin elsewhere
  are beyond 500 miles.
- Between this (closest ties) and TIES-0 (all links) the weak middle is unmeasured: work
  groups, interest groups, school.
