# MESH-0 — DRAFT, not frozen. Who holds the message: for a generated population, whether a message to a person is taken at once, held in their own household, or must be held by someone else, and for how long. The first reading of the mesh's viability from the generators, and the first composition of three of them. PREREGISTRATION

*REVIEWED 2026-11-12 (`MESH0_REVIEW.md`): freeze with named fixes. Most of what this draft
staked was found to be arithmetic on published shares. Kept as written; the freeze is
`MESH0_PREREG.md`.*

*Draft of 2026-11-12, for one adversarial review. No data is fetched: every input is a
generator or table already shipped (`household`, `devices`, `presence` in `holon-scene`;
TIES-1). At this commit nothing is built or computed.*

## Why this and not a verdict on viability

The twin's lead reports, from its code (`twin/globe`, `crates/cewp-sim/src/micro.rs`), that
the twin has no notion of a delivery failing: each send takes one of five routes, a row
held for a sleeping recipient waits without limit, and the only mailbox for a household
with no home node is a seat far away. The one number between "held at home" and "held at
the seat" is `home_node_household_share = 0.60`, tagged MODEL. So this campaign does not
read viability. It measures what viability would be read from, at the two ends of what is
not known, and says which unknown decides.

## What is built

In `holon-scene`, integers only: `world::delivery`, which composes three shipped
generators for one household and returns counts, with the verdict of the composition.

- **Population.** 100,000 households, United States: sizes from HOUSEGEN-1's rows over the
  five density classes in the share UNEVEN-0 gives them; composition and devices from
  DEVICEGEN-1; each person aged 18 or over a day from PRESENCEGEN-2's chain, employed
  drawn from the shipped share by age band, on one day type at a time.
- **An addressee** is a person aged 18 or over who holds a smartphone. Children get no
  day and are not addressees; persons with no phone are counted apart.
- **The state of an addressee in an hour:**
  - *taken at once:* awake (any of the four awake states);
  - *held at home:* not awake, and the household has a host that is on;
  - *held outside:* not awake, and it has none.
- **A host that is on** is not known for any household. Three settings, always reported
  together: NONE (no household has one); TWIN (a seeded 0.60 of households, the twin's
  number, independent of everything else); DECLARED (every household with a fixed line or
  a desktop, which is everything DEVICEGEN-1 declares able to host).
- **A message** is sent by another adult at an hour drawn in proportion to that sender's
  awake hours. With share `0.237` (TIES-1: closest ties in the same household) the sender
  is a member of the addressee's household, where it has two adults; otherwise a random
  adult of another household, same clock.
- **Hold time** of a message not taken at once: whole hours until the addressee's next
  awake hour, running into a second day drawn afresh.
- **The verdict of a composed count** is the weakest of its inputs' verdicts, and the call
  returns the list of what it rests on.

## What is declared and not measured

- Members of a household are drawn independently (PRESENCEGEN-2, S5). Two days of one
  person are drawn independently.
- A phone is on whenever its holder is awake. The twin's `mobile_offline_share` is not used.
- Ties in another time zone are not modelled (TIES-1: 0.75 of closest ties within 50 miles).
- Whether a declared host is on, can host, or is reachable from outside: not known. NAT
  is not in this campaign.

## Gate

- **G0.** Each addressee is in exactly one state in each hour; the three states sum to
  the addressee-hours for each setting. One digest native and wasm32. The share of
  addressee-hours taken at once equals the awake share of the same generated days to
  `1e-9` (it is the same count).

## Stakes

- **S1 (the twin's 0.60 against what a household declares).** The share of households
  that DEVICEGEN-1 declares able to host. Stake: `0.60` lies between the share with a
  desktop and the share with a desktop or a fixed line. *Kill:* it lies outside. Then the
  twin's number is not a reading of any declared fleet. Printed: both shares.
- **S2 (who is awake when a message is sent).** The share of messages taken at once,
  minus the share of addressee-hours awake. Stake: the difference is at least `0.10`
  (people send while awake, and people are awake at the same hours). *Kill:* under
  `0.05`. Then hour-of-day timing does not help delivery and a flat duty cycle loses
  little. Forecast: `0.15` to `0.22`.
- **S3 (does having no host go with being asleep more?).** Under DECLARED, the share of
  messages held outside, over the product of (share of messages not taken at once) and
  (share of addressees in a household with no declared host). Stake: the ratio is within
  `0.10` of 1: the two are close to independent in the generators. *Kill:* outside. Then
  who lacks a host and who is asleep are tied through household composition, and the
  seat's load cannot be had by multiplying two shares. I do not know the direction.
- **S4 (how long a held message waits).** Among messages not taken at once, the share
  that wait over 8 hours. Stake: under `0.10`. *Kill:* over `0.20`. Printed: the share
  over 1, 4, 8 and 12 hours and the median, for each day type.
- **S5 (which unknown decides), printed, no verdict.** The share of messages held outside
  under NONE, TWIN and DECLARED; and, one at a time, the swing in that share from: the
  host setting; the tie share in the same household set to 0 and to 0.5; the day type.

## What would make this vacuous

- The share held outside under NONE is one minus the share taken at once, and under TWIN
  it is that times 0.40: arithmetic on inputs. Only S2, S3 and S4 read something the
  inputs do not state.
- S3 can only find what the generators carry. DEVICEGEN-1 ties devices to household size,
  building and composition; PRESENCEGEN-2 ties a day to employment alone. A tie through
  age is carried; a tie through income is not.
- S4's waits cross midnight on two days drawn independently, and the chain was not scored
  on the join between days.
- No reading here is the mesh working or failing. That needs a bound on hold time and a
  seat's capacity, neither of which is this repository's to state.

## Plants

- **P-SUM** (any population). States sum; a household with no addressee adds nothing.
- **P-FLAT** (a planted population in which each person is awake in a random 15 of 24
  hours, independently). S2's difference is within `0.01` of zero.
- **P-TIED** (a planted population in which households with no host are all asleep 12
  hours and the rest 6). S3's ratio is over `1.3`.
- **P-WAIT** (each person asleep in one block of 8 hours). No wait is over 8 hours, and
  the mean wait of held messages is within `0.2` of 4.5 hours.
- **P-WEAK** (inputs with verdicts certified, unresolved, certified). The composed verdict
  is unresolved and names the unresolved input.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each planted population holds what its plant names, and each
plant runs through the shipped function.

## Branches

Exactly one fires.

- **(e)** The gate or a plant fails. Nothing is published.
- **(a)** Otherwise. The counts are published as a table with its sidecar, each stake as
  read, every composed count `unresolved` (the host setting is unknown for every
  household), and S5 naming the unknown with the largest swing as the next reference to find.

---
witness: `CIRISHolon.SceneRhythm.conserved` (one state per person per hour, so one delivery state per addressee per hour)
witness: none (S1–S5 are measured on generated populations)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function.
- M-BAR-FROM-THE-READ: bars are stated here. Known: the awake share (0.623), the share with nothing to host on (0.09), TIES-1's shares, the twin's 0.60. Not known: any joint of the three generators.
- M-BASE-RATE-OMITTED: the three host settings are always reported together; S2 is read against the flat awake share.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
