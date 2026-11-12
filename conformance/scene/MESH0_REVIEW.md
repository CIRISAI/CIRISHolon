# MESH-0 draft — adversarial review, 2026-11-12. Verdict: FREEZE WITH NAMED FIXES (fourteen; four of them change what a stake reads). As drafted, S1 and S3 are decided before the read, S2 is decided to within about 0.01, S5 names the host setting by construction, two plants fail or read nothing as written, and S4 reads the one quantity the chain was never scored on.

*Reviewer: a subagent with the draft, the three generators' source, and the published
results of PRESENCEGEN-2, PRESENCEGEN-1, PRESENCE-0, DEVICEGEN-1, DEVICES-0, HOUSEGEN-1,
TIES-1 and the twin survey. No shipped generator or compiled-in table was called. Every
number marked SYNTHETIC is from a made-up population in plain numpy (a scratch directory,
seconds, one thread), chosen away from the campaign's point: two kinds of sleeper
(45 % early and short, 55 % late and long; awake share 0.66), or one kind with a
persistent bedtime offset. They show mechanisms, not the campaign's values.*

## Findings

**1. S2 is arithmetic on the published hourly awake shares, to about 0.01.**
- With the sender drawn independently of the addressee, the share taken at once is
  `Σ a(h)² / Σ a(h)` where `a(h)` is the pooled awake share at hour `h`; S2 is that minus
  the mean of `a`. PRESENCE-0's table and the chain's reproduction of hours by day type
  (mean miss 0.008) give `a(h)` already.
- SYNTHETIC: measured S2 `0.1406`; from the pooled hourly shares alone `0.1318`; from the
  two kinds' profiles and their mix `0.1334`. The whole residual (`0.007` to `0.009`) is
  that each sender normalises over their own awake hours.
- What is NOT in the hourly shares: senders who resemble their addressee. SYNTHETIC, all
  senders of the addressee's own kind: S2 `0.2148`, a residual of `0.083`. In the draft
  that resemblance exists only for the same-household share and only through age
  (members are drawn independently), so almost none of it is carried.
- The stake (at least 0.10) cannot fail: any profile with a night gives 0.13 or more.
- **Fix (S2-RESIDUAL):** compute `Σa²/Σa − mean a` from the PRESENCE-0 table before the
  read and print it as the baseline. Restate S2 as the measured value minus that
  baseline, by sender kind (same household / other) and by employed × employed. Stake on
  the residual (my forecast: under `0.02`, so "the hourly table is enough"); kill if it
  is over `0.05`.

**2. S3 is forced to about 1 by construction, and the draft's sentence "a tie through age is carried" is wrong.**
- `devices.rs`: composition is drawn from `key(0, size, 0, building, 0)`; the cell
  (connection × any computer × any speaker) from `key(1, size, 0, building, 0)`. Given
  size and building the two draws are independent. Seniors enter the device draw nowhere;
  only "any child" keys the per-kind histograms.
- So "no host" and "asleep" can be tied only through household size and building: small
  households hold more adults over 64 (employed `0.206`) and fewer fixed lines
  (DEVICES-0: fixed broadband `60.9 %` at one person, `81.9 %` at four). A second, weaker
  path: phones per person are keyed on the connection, which changes who is an addressee.
- Size of the first path, SYNTHETIC two-group arithmetic: no-host `0.30 / 0.10` and
  not-taken `0.30 / 0.24` give a ratio of `1.054`; no-host `0.14 / 0.11` gives `1.010`.
  The direction is above 1 if the not-employed are asleep for more of senders' hours.
- Resolution, SYNTHETIC under exact independence: one message per addressee (about
  190,000), no-host share `0.07`: sd of the ratio `0.014`; no-host `0.15`: `0.009`; ten
  messages each: `0.005`. A bar of `0.10` is five to ten sd and several times the largest
  effect the generators can carry. The stake cannot be killed.
- In the survey the tie is real and larger (older and poorer households lack both the
  computer and the line); DEVICEGEN-1 R5 only checked any-senior by size. A "met" here
  would be read as "the seat's load is a product of two shares", which the generators
  cannot support.
- **Fix (S3-BOUND):** S3 becomes a printed reading with its sd, labelled "what the
  generators carry through size and building; a lower bound on the tie". Add one
  bracketing setting, HOSTLESS-OLD: among households of each size and building, give the
  no-host flag to the households with adults over 64 first, at the same share. Print the
  ratio under both. Correct the vacuity paragraph. State the number of messages per
  addressee (it is nowhere in the draft).

**3. DECLARED's complement is not the certified "nothing to host on".**
- Certified by DEVICEGEN-1: no computer (laptop, desktop or tablet) AND no fixed line,
  `0.088`. DECLARED's complement is no DESKTOP and no fixed line: a superset, bounded
  above by the `0.272` with no fixed line. It rests on the desktop count, whose verdict
  is `unresolved` (and R3: desktops in apartments −35 % generated against −46 %).
- The misfit note lists "the share with nothing to host on (0.09)" as the known base
  rate. It is not the base rate of the event the campaign uses.
- **Fix (HOST-EVENT):** name the event exactly; state that it rests on the unresolved
  counts call; print it beside the certified `0.088` event; add the setting ANY-COMPUTER
  (fixed line or any laptop, desktop, tablet), whose complement IS the certified event.

**4. S1 is known before the read.**
- DEVICES-0 publishes desktop `40.8 %`, fixed broadband `72.8 %`. Desktop-or-fixed-line
  is at least `0.728`. `0.60` lies between. Nothing can kill it; the interval is 0.3 wide.
- **Fix (S1-MISPLACED):** demote the two shares to printed. Stake instead on what TWIN
  gets wrong against the declarations: the share of TWIN's hosts that fall in households
  with nothing declared (expected `≈ 1 − declared share`, so about `0.15` to `0.25` of
  them), and the difference in held-outside between TWIN and a setting with the same
  0.60 placed only in declared households (finding 9).

**5. S4 reads the chain where it was not scored, with a bar set below the known median sleep.**
- The chain was certified on spells AWAY, three far-apart hours, total hours asleep and
  sleep before leaving. Contiguous sleep across the 4 a.m. boundary was never scored.
  Its nearest scored relative, hours asleep, is 7 / 9 / 11 against the diaries'
  8 / 9 / 10: within the one-hour budget and twice the spread, in exactly the tail S4
  reads.
- The bar: median sleep is 9 hours (PRESENCE-0: mean 9.04). A message that arrives as
  its addressee falls asleep waits 9. "Under 0.10 wait over 8" is a bet on arrival phase
  with nothing known behind it.
- The join, SYNTHETIC (one kind, bed 23:30, 7.5 h, a persistent bedtime offset of sd
  1.2 h; truth is the person's own next day):

  | second day | wait ≥ 8 | wait ≥ 9 | ≥ 13 | mean |
  |---|---|---|---|---|
  | the person's own (truth) | 0.152 | 0.052 | 0.000 | 4.23 |
  | drawn afresh | 0.245 | 0.167 | 0.007 | 4.61 |
  | the same day wrapped | 0.182 | 0.089 | 0.000 | 4.33 |

  A fresh day triples the long tail and manufactures waits that cannot happen; the wrap
  is nearer. At the seam, asleep at 4 after awake at 3: truth `0.09`, fresh `0.92`,
  wrap `0.66`.
- Caveat on that table: the chain carries one bit, so a generated evening is already
  nearly independent of the same generated morning. For the chain the two joins will
  differ much less than here, and BOTH will sit on the "afresh" side of the truth. The
  bias is long waits, upward.
- Whole hours: a state is one seeded minute per hour, so a wait of `k` slots is a true
  wait between `k − 1` and `k + 1`. "Over 8" on integers is `k ≥ 9` and drops half of
  `k = 8`. SYNTHETIC: `≥ 8` and `≥ 9` differ by a factor of 1.5 to 3.
- The second day's day type is not stated (after Friday comes Saturday).
- **Fix (S4-JOIN):** S4 has no verdict. Print the full distribution of `k` for three
  joins: afresh; the same day wrapped; afresh but redrawn until slot 0 agrees
  (asleep/awake) with slot 23. State the convention (`k ≥ 9`, with `k ≥ 8` beside it).
  The spread across joins is the reading's error bar. State the next day's type. Before
  the read, print the chain's contiguous sleep-spell quartiles next to 8 / 9 / 10 and
  say S4 inherits that miss.

**6. The message model: 0.237 is a share of TIES, and the rule does not apply it.**
- TIES-1 T3: the house takes `0.237` of ties, `0.236` of written contact-days, `0.325`
  of voice, and by S2 `0.29` (floor, as named) to `0.52`–`0.61` of electronic contact.
  It includes ties to children at home; the draft's senders are adults only.
- "Where it has two adults": a one-adult household gets every message from outside, so
  the realised same-household share is `0.237 ×` (share of addressees with another adult
  at home), about `0.19`. One-person households are also the oldest and the least
  hosted, so the shortfall is not spread evenly.
- Children's phones (`phones[2]`) are dropped as addressees and as senders. Persons of
  15 to 17 have diaries and are dropped too.
- `phones` is a total per group, not per person, and a person can hold two. The draft
  does not say how "a person who holds a smartphone" is assigned.
- It does not matter much as built: members are drawn independently, so a same-household
  sender is statistically an outside sender. That is finding 8.
- **Fix (TIE-SHARE):** call it a share of ties used as a share of messages (TIES-1 S1
  licenses that for written contact only). Central `0.237`, bracket `0.29` and `0.52`
  from TIES-1 S2. Print the REALISED same-household share. State the phone assignment
  rule and count children's phones apart, as the draft does persons without one.

**7. "Awake" is not "phone on and reachable", and "held at home" is an upper bound.**
- Four awake states count: home, workplace or school, elsewhere, travelling. The twin
  carries `mobile_offline_share = 0.15`. No reference here says a phone is reachable at
  work or in transit.
- A desktop sleeps; a gateway is declared present, not able to host; for the roughly
  three quarters of messages from outside, a home node must be reachable inbound (NAT,
  excluded). So DECLARED is a ceiling on held-at-home and NONE the floor. TWIN is
  neither.
- **Fix (ON-BRACKET):** report every count twice: phone on in all awake states; and on
  only at home awake plus `0.85` of the other awake hours (seeded). Label NONE "floor",
  DECLARED "ceiling", in the table header.

**8. S5 returns the host setting by construction.**
- Host swing NONE → DECLARED is `(1 − taken) × (declared share)`: about `0.25 × 0.8`.
- The tie-share swing is about zero, because members of a household are drawn
  independently (declared; PRESENCEGEN-2 S5 unresolved). The day-type swing is a few
  points of the awake share.
- So "S5 names the next reference" is known now: the always-on host. The one unknown
  that could compete, co-residents' shared clock, is switched off by declaration.
- **Fix (S5-FAIR):** add a fourth swing: household members share one day (the other
  bound of the independence declaration; a same-household sender then sends only when
  the addressee's own day is awake). Say in the text that the host swing is arithmetic
  and state its expected size before the read.

**9. TWIN is not a fair comparator.**
- An independent 0.60 puts hosts in households with no line and no computer and
  withholds them from 40 % of households with both.
- **Fix (TWIN-PLACED):** add TWIN-IN-DECLARED (0.60 of all households, placed only among
  declared ones; if declared is under 0.60, all of them), and DESKTOP-ONLY and
  LINE-ONLY as settings, since S1 prints those shares anyway.

**10. The verdict rule is unsound as stated.**
- "Weakest of its inputs" has no stated order (is `refused` below `unresolved`?), and
  P-WEAK only tries certified with unresolved.
- An input used outside what it was certified on is not certified for that use: the
  chain for the join of two days, for the household joint, for contiguous sleep, for
  adults of 18 and over only; devices for an event that is not the certified one.
- The US household call is `unresolved` for every class ("key is a census tract"), so
  every composed count is unresolved three times over before the host setting.
- Three inputs to the composition have no verdict at all and are not listed: the density
  class shares "UNEVEN-0 gives" (UNEVEN-0 is a grid reading, not a US household share;
  the draft does not say which numbers); the building passed to `devices::generate`
  (not stated; DEVICEGEN-1 S8 allows drawing it apart for MEANS, not for the
  no-host joint); the age band of an adult of 18 to 64 (devices returns 18–64 and 65+,
  `employed_ppm` wants 15–24, 25–64, 65+).
- **Fix (VERDICT-LIST):** order `certified > unresolved > refused`; the composed verdict
  is the minimum, and any input used outside its scored envelope enters as `unresolved`
  with the reason. The call returns one row per input: the call, its verdict, what it
  was scored on, whether this use is inside. Add a P-WEAK case with a refused input, and
  one with a certified input used out of envelope. State the three unstated rules.

**11. Plants: two fail or read nothing as written; G0's third clause cannot fail.**
- P-WAIT, SYNTHETIC: blocks at random hours with the second day afresh: max wait `16`,
  `0.027` over 8: the plant fails by the draft's own join. Same block both days: max 8,
  mean `4.51`, passes. Everyone on the same block: no sender is awake in it, nothing is
  ever held, the plant reads nothing.
- P-TIED: with sleep at random hours the ratio is `2 / (1 + q)` for a no-host share
  `q`: `1.82` at 0.1, `1.33` at 0.5, `1.25` at 0.6. The bar of 1.3 fails above
  `q ≈ 0.54`, and `q` is not stated.
- P-FLAT can fail and is sound. P-SUM is sound.
- G0, "taken at once equals the awake share to `1e-9` (it is the same count)": the same
  count compared with itself, in integers, with a float tolerance.
- **Fix (PLANTS):** P-WAIT: random block per person, the SAME on both days. P-TIED:
  state `q = 0.2` and random hours. Replace G0's third clause by a count that can fail:
  the awake share of generated adults within `0.02` of the published `0.623`, and the
  generated share of households in the certified nothing-to-host-on event within `0.01`
  of `0.088`. Add: P-HOUSE (tie share 1, every household two adults: every sender is in
  the addressee's household; tie share 0: none is); P-JOIN (a planted person awake only
  in slots 0 to 11 on both days, a message at slot 12: the wait is exactly 12, and the
  wrap and afresh joins agree); P-SEED (one seed, one table; two seeds differ; the
  stakes move by less than their bars across five seeds); P-NULL-TIE (host flag
  permuted across households: S3's ratio within three sd of 1).

**12. Branches: one fires, and it does not depend on any stake.**
- (e) or (a) is exhaustive and exclusive. But (a) is the same sentence whatever S1 to S4
  read, so the stakes decide nothing. S2 and S4 have unnamed middle zones (0.05 to 0.10;
  0.10 to 0.20).
- (a) says S5 "names the next reference"; S3's kill says the load cannot be had by
  multiplying shares; S2's kill says timing does not help. None contradicts (a), because
  (a) says nothing.
- **Fix (BRANCHES):** state the middle zones as "neither". Split (a) by what is licensed:
  (a1) the residual of S2 is under its bar and the joins of S4 agree: the hourly table
  plus a host share is a sufficient model of message timing, and the twin may use the
  closed form; (a2) otherwise: the composition is needed and the table ships.

**13. Unstated quantities.** Messages per addressee; seeds and how many; the density
class shares; the building; the 18–64 age band; the phone assignment; the next day's
type; whether "share of addressees in a household with no declared host" in S3 is
weighted as messages are. **Fix (STATE-THEM):** one line each, before the freeze.

**14. Viability.** As designed it reads message timing under one ordering of holders
(addressee, home, seat). It says nothing about the mesh, as the draft admits.
- **Fix (SENDER-HOLDS), the one addition:** add the holder that needs nobody else: the
  sender's own phone keeps the message and delivers at the first hour both are awake.
  Print, for `T` = 0, 1, 4, 8, 12, 24 hours, the share of messages delivered within `T`
  with no host and no seat, beside the same curve with a host. The gap between the two
  curves is exactly what a host or seat buys, as a function of the bound the draft says
  is not this repository's to state. Beside it, the held-outside queue by clock hour per
  1,000 addressees (peak over mean): what a seat is sized for. Both use only what is
  already generated.

## Forecasts (mine, before any read; not from the shipped tables)

- **S1:** desktop about `0.41`; desktop or fixed line `0.78` to `0.85`. Met, as it must be.
- **S2:** `0.19` to `0.25` (a hand-made profile near the published 0.623 gives `0.21`);
  met, as it must be. Residual over the closed form: under `0.015`.
- **S3:** ratio `0.99` to `1.04`, more likely above 1. Met, as it must be.
- **S4:** share with `k ≥ 9`: `0.18` to `0.32`. Not met; killed more likely than not.
  Median wait 3 to 5 hours. The afresh join above the wrapped join by under `0.03`.
- **S5:** the host setting, by `0.18` to `0.22` of messages; tie share under `0.01`;
  day type under `0.03`.
