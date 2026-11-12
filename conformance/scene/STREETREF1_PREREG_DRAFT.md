# STREETREF-1 — a reference sample of real streets and buildings, held as statistics, for the scene certificate. PREREGISTRATION (DRAFT)

*DRAFT of 2026-11-11, revised the same day with the twin's class rules. NOT FROZEN. The lead approved a road and building reference on
2026-11-11 ("yes, approved, please proceed"). Nothing has been fetched. No place has been
drawn. No generated street has been looked at for this purpose. Nothing here is a freeze
until the lead strikes this line and the file is committed alone as `STREETREF1_PREREG.md`.*

## The class rules, received 2026-11-11 (the hold of the same day is lifted)

From the twin's lead, at twin/globe `a29e87d` (`FSD/twin/globe.md` §"Street classes, the floor
and the region rule"; `docs/geo/classes.json`, schema `cewp-twin-street-classes/v1`). Written
here by value; all are the twin's MODEL choices and this reference takes them as given.

- **Density classes**, lower bound in people per km²: village `0`; suburban streets `250`;
  terraced streets `1,500`; apartment blocks `6,000`; dense fabric `12,000`. FIVE classes.
- **Floor.** A 1/8 degree cell under `1` person per km² of the cell's WHOLE area (not its
  land area) contributes nobody and is not drawn.
- **Where the class is judged.** On a street leaf. Today a leaf's density equals its cell's,
  because a cell's people are spread evenly over it, so the class of a 1/8 degree cell is
  computed here from GHS-POP people over the cell's whole area.
- **Region rule.** A 1/8 degree cell takes the region of the 1.5 degree cell holding it;
  each of those belongs to one of 181 localities and each locality to one of 9 regions.
- **In-cell weights.** None. A tile drawn uniformly within its cell IS the fair comparison.

Two things the twin says may still move, and what this reference does about each:
(1) the cell-to-locality assignment could change at region edges when the twin unifies its
root population: the draw records each cell's region AS ASSIGNED on the day, and a cell
later reassigned is re-labelled, not re-fetched; (2) persons per household enters the leaf
density and cancels today: it does not enter this reference, which reads people per km².

**Owed from the twin before the draw:** the region partition as a table (1.5 degree cell →
region), which `classes.json` describes in words but does not carry.

## Purpose (the lead's, 2026-11-11, as relayed by the twin's lead)

"goal is to enhance the holon engine with our worked example to become a better world
model." The twin is the worked example. So the observables below are chosen for what a
generator and its certificate need, and for the couplings the mesh reads (households per
street, distance between neighbours, distance to an exchange), not for what is easiest to
count. The same approval allows this reference's statistics table, and nothing else from
it, to be committed in the CEWP repository and shown on its page under the notice in
§"The licence reading".

## Why

The CEWP twin's own inventory (its lead's four-part answer of 2026-11-11, at twin/globe
`f9cc739`) says real data stops at the 1/8 degree cell. Below it every street, building,
household and device is generated and nothing real stands beside them. Issue #3's
statistical half needs a reference sample of reality. This campaign builds one, for the two
object kinds where open data exists: streets and buildings.

It is a reference, not a test of a generator. The two cautions it is built around:

- **The places are chosen before any generated output is looked at.** The draw is a seeded
  rule over the twin's own population grid, frozen here.
- **The certificate will be against this sample, not against the world.** What the sample
  cannot say is named below, and a class it covers too thinly is `unresolved`.

## What "a place" is, and how places are drawn

- **The parent.** One cell of the twin's 1/8 degree population grid (GHS-POP R2023A, epoch
  2020, as the twin ships it in `pop1.bin.gz`). This is the finest real reading the
  generator is conditioned on, so it is the parent the reference is conditioned on too.
- **The class of a parent.** Two labels, both computed by the twin's own rules so that a
  class means what the generator sees:
  - region: the twin's 9 regions;
  - density class: the twin's five, from the cell's people per km² of whole cell area
    (thresholds in §"The class rules" above), down to the twin's floor.
  That is 45 classes.
- **The draw.** Within each class, 40 parent cells are drawn without replacement by a
  seeded hash of the cell index (seed 20261111), with probability proportional to the
  cell's people. A class with fewer than 40 cells contributes all it has. 40 is the floor
  of 30 below plus a margin of 10 for cells that return nothing.
- **The scene.** Inside each drawn parent, 3 street tiles of 0.02 degrees are chosen by the
  same hash. The twin spreads a cell's people evenly over its tiles, so a tile chosen at
  random is the fair comparison to what it generates. Three tiles per parent measure how
  uneven a real cell is inside.
- **Total.** At most 45 × 40 × 3 = 5,400 tiles of about 2.2 km.
- **The block.** The parent cell. Three tiles of one cell are one block, never three
  samples.

The list of drawn cell indices and tile offsets is written to `streetref1/draw.json` and
committed alone before the first fetch.

## The source and the licence step

- The **source.** OpenStreetMap, through a public read API, one bounding box per tile, run one
  at a time with a pause between requests and a generic client name. No account, no bulk
  planet file.
- **Licence.** Read on 2026-11-11; the reading is §"The licence reading" below.
- **What is kept.** Per tile: the scalars and fixed-grid quantile functions in the table
  below, the parent cell index, and the tile's class. **What is not kept:** any coordinate,
  any geometry, any way or node id, any name, any tag value. Raw responses are deleted
  after each tile's statistics are written and checked.
- **What is never done.** Nothing real is drawn. No generated object is placed on a real
  road or building.

## The licence reading (done 2026-11-11, before any fetch; not legal advice)

Read at the publishers' own pages on 2026-11-11: the Open Database License v1.0
(opendatacommons.org), the OpenStreetMap Foundation's Community Guidelines (Produced Work,
Substantial, Trivial Transformations) and its Attribution Guidelines (adopted 2021-06-25),
and the public query service's usage page. The quotes are short excerpts; the full text at
those pages governs.

**What the texts say.**

- ODbL §1.0 defines a Produced Work as "a work (such as an image, audiovisual material,
  text, or sounds)" made from the whole or a Substantial part of the Contents, and a
  Derivative Database as "a database based upon the Database".
- The Foundation's Produced Work guideline gives the test: "If the published result of
  your project is intended for the extraction of the original data, then it is a database
  and not a Produced Work." "Otherwise it is a Produced Work."
- The Substantial guideline: under 100 features, or the features of an area of up to 1,000
  inhabitants, is insubstantial, and "we regard repeated small extractions as one big
  extraction".
- ODbL §4.3: a Produced Work used publicly must carry a notice; the licence's example is
  "Contains information from DATABASE NAME, which is made available here under the Open
  Database License (ODbL)."
- ODbL §4.5(c): use of a Derivative Database "internally within an organisation is not to
  the public".
- ODbL §4.6: publicly using "a Produced Work from a Derivative Database" obliges an offer
  of either the whole Derivative Database or "the method of making the alterations to the
  Database (such as an algorithm)", free of charge if over the internet.
- The Attribution Guidelines, on machine learning: "Training datasets that are substantial
  extractions from OpenStreetMap data are considered Derivative Databases"; models trained
  on them "must be attributed in documentation"; and "Predictions made using such a model
  are not implicated by ODbL." A Produced Work that can be used to recreate substantial
  parts of the data is treated as a Derivative Database.
- No guideline addresses statistics or aggregates by name.

**The reading taken.**

1. **The extraction is Substantial.** Up to 5,400 tiles, systematic, each holding more than
   100 features in any town. The insubstantial-use exemption does not apply and is not
   relied on.
2. **The fetched tiles are treated as a Derivative Database, used internally.** They are a
   substantial extraction held on one machine, filtered by a tag list, and deleted once
   each tile's statistics are written. They are never conveyed.
3. **The statistics table is treated as a Produced Work.** It holds no coordinate, id,
   name or tag value, it is not intended for extracting the original data, and it cannot
   recreate any part of it. P-STRIP is the plant that checks the second and third of those
   on the written bytes.
4. **Publishing the table therefore owes two things**, both met by the record itself:
   - the §4.3 notice, placed with the table and in every read that cites it:
     *"Contains information from OpenStreetMap, which is made available here under the
     Open Database License (ODbL). © OpenStreetMap contributors.
     https://www.openstreetmap.org/copyright"*;
   - the §4.6 offer, met by option (b), the method: `draw.json`, `tags.json` and the
     extractor are committed in this public repository, free of charge, and together they
     regenerate the internal tiles from the public source.
5. **A generator fitted to the table** is read by the machine-learning passage: attributed
   in its documentation, and the streets it generates are not implicated by the licence.
6. **The population labels come from a second source** (GHS-POP, CC BY 4.0). They are
   joined to the statistics, not to the OpenStreetMap data, so no outside data is added to
   the internal tiles. GHS-POP's own credit line travels with the table.

**What is uncertain, said plainly.** Point 3 rests on a general test, not on a provision
about statistics: none exists. If the table were held to be a Derivative Database instead,
it would have to be published under the ODbL with attribution. This record could do that
without loss, since nothing here needs the table under any other terms. So branch (l) is
not a wall: the choice it leaves the lead is between "Produced Work with a notice" and
"database under the ODbL", and either publishes.

**The query service's limits.** Its usage page asks for "a maximum of about 10000 requests
per day" and a download volume "below about 1 GB per day". The fetch is capped well under
both: at most 2,000 tiles and 500 MB in any one day, sequential, stopping for the day at
whichever comes first.

## Observables

Streets are ways tagged as roads open to vehicles, from residential up to trunk; footpaths,
service alleys and private drives are excluded by a tag list frozen in `streetref1/tags.json`.
A segment is a stretch of street between two intersections or ends.

| id | observable | per tile | why it is here |
|---|---|---|---|
| `len` | street length per km² | scalar | what a leaf's street budget stands for |
| `seg` | segment length | quantile function, 201 points | block size |
| `deg` | share of nodes that are dead ends, three-way, four-way or more | 3 shares | issue #3's intersection-type mix |
| `ori` | orientation entropy over 36 bearing bins, weighted by length | scalar | the twin's streets are all north-south or east-west |
| `sin` | segment length over the straight distance between its ends | quantile function | issue #3's circuity, per segment |
| `bld` | buildings per km² | scalar | building density |
| `area` | building footprint area | quantile function | footprint size |
| `btw` | the Gini coefficient of length-weighted shortest-path betweenness over a tile's segments | scalar | owed by the prior-art screen: street models fail on the heterogeneity of betweenness (Barthelemy and Boeing 2025), and orientation alone would pass them. Computed within the tile, so paths that would leave it are missing: a tile-edge artefact, the same for every tile |
| `bps` | buildings fronting each segment (within 30 m of it) | quantile function | the stand-in for households per street, the first coupling the mesh reads |
| `nn` | distance from each building to its nearest building | quantile function | the stand-in for distance between neighbours, the second coupling |
| `clus` | share of a tile's buildings in its largest cluster, linking buildings closer than 30 m, 60 m and 120 m | 3 shares | whether neighbours chain into one mesh: a reading of the wiring, which no count fixes |

- **Each tile's record is kept whole.** All of a tile's scalars sit in one row, so a
  consumer that reads two observables together can be scored on the joint and not on each
  alone (`SceneCert.marginals_match_joint_differs`).
- **`clus` is the one relational observable.** `ScenePopulation.edge_law_does_not_fix_wiring`
  says the coarse edge count cannot answer it, so it is measured.
- **Distance to an exchange has no reference here.** Exchanges and cabinets are mapped too
  sparsely in open street data to stand for the real network. It is named as a coupling
  with no reference, and a generator's value for it can only be `unresolved`.
- **Households per street has no real reference here.** Open street data holds no
  households. `bps` is its stand-in, counting buildings and not households, and is labelled as one
  wherever it is printed. The same holds for `nn`.
- The **buildings are reported, and owe a completeness argument before they certify anything.**
  Building coverage in open street data is uneven across regions and this campaign has no
  second source to measure it against. Until one is named, `bld`, `area`, `bps`, `nn` and `clus` can
  return `refused` or `unresolved` for a generator and never `certified`.

## The resolution floor

A class is resolved for an observable when at least 30 of its blocks returned at least 1
street segment. A quantile level of a per-tile quantile function is resolved when at least
30 blocks each hold at least 30 segments beyond it. Tiles with no street at all are kept
and counted: an empty tile is a real reading of a populated cell, and the share of empty
tiles per class is itself reported.

## Stakes

These are stakes about the reference. No generator is scored in this campaign.

- **S1 (the reference agrees with itself).** For each resolved class and each street
  observable, the class's blocks are split into two halves 1,000 times; the budget `β₀` is
  set as in SCENE-1 (each component's median times one common multiplier, all components
  passing together in 95 % of splits). Stake: a held-out 10 blocks of the class pass at
  `β₀` in at least 2 of every 3 resolved classes, with at least 12 classes resolved. **Kill:**
  fewer than 1 in 2 of the resolved classes. Then a class
  of this size is not one population and the class labels are the wrong ones.
- **S2 (density class is a real envelope dimension).** Within a region, adjacent density
  classes are told apart: the W1 between them on `len` is at least `3 β₀` in at least 7 of
  the 9 regions. **Kill:** fewer than 5. Then density class does not partition real
  streets and the envelope should not be cut along it.
- **S3 (region is a real envelope dimension).** Within the terraced density class, the
  orientation entropy `ori` of some pair of regions differs by at least `3 β₀`. **Kill:**
  no pair does at `1 β₀`. Then one generator may serve every region on this observable.
- **S4 (cells are uneven inside).** The share of tiles with no street, in cells at or above
  250 people per km², is at least 10 %. No kill: either number is a reading. It prices the
  twin's even spread of a cell's people over its tiles before any generator is scored.

## Plants

Each runs on the extractor before the first real tile is read.

- **P-GRID** (carrier: a synthetic square grid of streets 100 m apart on a 2 km tile;
  sector: every street observable). Must return `len` = 20 km per km² to 2 %, `ori` at the
  two-bin value to 1 %, four-way share above 0.9, `sin` = 1 to 3 significant digits.
- **P-ROT** (carrier: P-GRID turned by 17 degrees; sector: the bearing). `len`, `seg`, `deg`
  and `ori` must be unchanged to 1 %. An extractor whose entropy depends on the frame fails
  here.
- **P-RADIAL** (carrier: a synthetic set of 36 spokes; sector: the bearing distribution).
  `ori` must read the maximum for 36 bins to 1 %.
- **P-EMPTY** (carrier: a tile with 0 ways; sector: the count). Must return an empty
  record, counted, with no division by zero and no dropped tile.
- **P-DUP** (carrier: one real class with every tile's record repeated 10 times; sector:
  the sample count). The class's resolved levels must not move, since the block count has
  not.
- **P-STRIP** (carrier: one fetched tile's written record; sector: its bytes). The record
  must hold 0 coordinates, 0 ids and 0 strings beyond the field names; checked by a reader
  that knows only the schema.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction for the four synthetic plants: each carrier is built to hold
the streets, bearings or absence the plant names. For P-DUP the carrier is a class with at
least 30 blocks, checked before the plant is scored. For P-STRIP the carrier is a tile
with at least 1 street.

## Branches

- **(e)** A plant fails. Nothing is fetched beyond the tiles already read; the extractor is
  the finding.
- **(l)** The lead reads the table as a database, not a Produced Work. It is then published
  under the ODbL with attribution; nothing else in this campaign changes.
- **(a)** S1 to S3 met. The reference is one population per class, and both class labels
  cut real streets. It is frozen as the scene certificate's street reference.
- **(b)** S1 met, S2 or S3 killed. The reference stands; the envelope is redrawn along the
  dimension that survived, stated with the one that did not.
- **(c)** S1 killed. The classes are not populations. The sample is kept, the classes are
  not used, and a finer class (settlement size, terrain, or the land-cover majority class
  the twin already ships) is proposed in a new freeze.

## Order

1. This file, frozen by the lead, committed alone.
2. The licence step. DONE in this draft (§"The licence reading"); it is frozen with the file.
3. The draw. `streetref1/draw.json` and `tags.json` committed alone.
4. The extractor is built. Every plant runs and prints.
5. The fetch, one tile at a time.
6. Budgets from the split halves, then the stakes.

## Cost

- At most 5,400 small requests to a public service, sequential, with a pause, under the
  daily caps in §"The licence reading". The download size per tile is not known: it is
  measured on the first 20 tiles and printed before the rest are fetched, and the run
  stops if the service asks it to.
- Per tile, a planar graph of at most a few thousand segments. No priced compute.

## What this does not say

- **Not the world.** 40 cells a class, drawn by people. Places open street data maps
  poorly are under-read, and the sample cannot tell that from a place with few streets.
- **Not households, devices, or any network.** Streets and buildings only.
- **Not relationships between places.** Nothing here reads how one tile's streets join the
  next. The edge law in `ScenePopulation.lean` has no reference in this campaign.
- **Not time.** One reading of a map that changes daily; the fetch dates are recorded.
- **Not a generator's verdict.** Scoring the twin's streets against this is the next
  freeze, after the twin has a street object to score.

---
witness: `CIRISHolon.SceneCert.verdict_certified_iff` (the floor and envelope order this reference feeds)
witness: `CIRISHolon.ScenePopulation.pop_total` (the parent a place is drawn under is a population reading)
witness: none (S1 to S4 are measured stakes about a sample; no gate here is a Lean theorem of its own)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-POPULATION-CHOICE, M-BAR-FROM-THE-READ, M-FLOOR-UNSTAKED, M-FORMAT-FLOOR, M-BASE-RATE-OMITTED, M-HOMOG, M-PROVENANCE-OVERREACH. These are the registered ids this text contacts.

- M-PLANT-OBS: every plant is built for this extractor and runs before a real tile is read.
- M-POPULATION-CHOICE: the places are drawn by a declared seeded rule over a declared grid, not chosen. Completeness is not gated against an outside count, and §"What this does not say" says so.
- M-BAR-FROM-THE-READ: `β₀` comes from split halves of the reference and is fixed before the held-out blocks are scored.
- M-FLOOR-UNSTAKED: the resolution floor counts blocks, and P-DUP is its plant.
- M-FORMAT-FLOOR: P-GRID's sinuosity bar is stated in significant digits and the extractor writes float64.
- M-BASE-RATE-OMITTED: the share of empty tiles is printed beside each class's statistics.
- M-HOMOG: "region" and "density class" are the twin's labels on a cell. Nothing is claimed about how streets in one place depend on streets in another.
- M-PROVENANCE-OVERREACH: the draw file is committed with its digest. A digest names bytes; that the fetched tiles are the drawn ones rests on the fetch log.
