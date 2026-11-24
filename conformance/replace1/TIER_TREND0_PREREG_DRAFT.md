# TIER-TREND-0 — does the organised share rise with tier? The removability gate read in one unit on tiers it has never been read on. PREREGISTRATION (DRAFT)

**REVIEWED 2026-11-23: REDESIGN. See `TIER_TREND0_REVIEW.md`. The share `O` is not a unit (it moves with the target, its scale, the order and redundant charts); the reasoning stake cannot tell carried from persistent; the molecule stake is fixed by banked arithmetic; the calibration table misfiles the fluid's count block. Kept as drafted.**

*DRAFT of 2026-11-23, on the lead's word ("Yes, that makes sense" to drafting it). NOT FROZEN,
and not to be frozen before an adversarial review. Nothing here has been computed beyond
arithmetic on numbers already in git (§3), and no walk or trace was opened. The lead's
theory, 2026-11-23: "higher tiers, biological, civilization, would create more of the
dynamic portion, WAY more." In the record's words (`OBJECT.md`, CORRECTION 2026-11-23): every
tier has a static portion, the conserved core the level above carries, and an organised
portion, closed and priced; the claim staked here is that the organised portion's SHARE
rises with tier.*

**What the drafting found, said first.** (i) In the one unit that can be written identically
at every tier, the record's two "points on the trend" are NOT separated: the fluid reads
`0.00` to `0.03` with a standard error of `0.04` to `0.10`, the reasoning chains `0.028 ±
0.014` (§3). The sentence "physical tiers near zero, an agency tier carried" is true of the
labels and not yet of a number. (ii) The unit depends on the target, and the reasoning tier
has no level above on this record, so its target is a stand-in (§1, §9). (iii) Measured by
how CLOSED it is and not by what it costs the level above, the organised portion is already
large at the physical tiers (the vibrations are more closed than the molecule; tetrahedral
order closes where the hydrodynamic block barely predicts itself at 1 ps), so the claim can
only be about PRICE TO THE LEVEL ABOVE. (iv) Two cells are unread and readable; one of them
is clean and does not test the rise, the other tests the rise and is compromised. A trend
cannot be frozen on this record today. What can be frozen is the floor and one step.

## 0. What was searched (M-SCREEN-SCOPE-UNSTATED)

Both repositories, by text search on 2026-11-23, for "tier trend", "rises with tier",
"organised share" in either spelling, "energy rate density" and "Chaisson": this repository's
tracked Markdown, and the sibling seed's Markdown and Lean files (`/home/emoore/CIRISOntology`,
read-only). No hit in either. Not searched: the sibling's two branch checkouts, the
Constitution's tree, any trace directory, the literature beyond §8.

## 1. The quantity, defined once

At a tier, declare three things BEFORE reading:

- the **static block**: the sectors with an exact balance law under the tier below's
  dynamics (a conserved density with its flux, a rigid unit's momenta, a counter that
  increments by rule);
- the **organised block**: every other closed sector in the tier's dictionary;
- the **target**: the next state, at the tier's cadence, of what the level above carries.

Then, with the removability gate as banked (`holon-closure::removable`, held-out `R²`
increment over time blocks or units, a re-paired null read at every step, jackknife standard
error; `conformance/replace1/REPLACE1_RESULTS.md` §5):

- `P_static` = the static block's increment on the target with nothing else kept;
- `P_org` = the organised block's JOINT increment on the target with the static block kept;
- **`O = max(P_org, 0) / (max(P_org, 0) + P_static)`**, the organised share. Its standard
  error is propagated from the two jackknife errors; to first order it is
  `se(P_org) / P_static` when `P_org` is small.

`O` is dimensionless and has the same form at every tier. It is a fraction of what the level
above can predict of its own next state: how much of that the organised portion adds once the
conserved core is known.

**What `O` cannot compare, stated before any use.**

1. It is a fraction of a TARGET's predictable variance. Change the target and it changes.
   On the fluid, tetrahedral order adds nothing to the next density and current
   (`conformance/replace1/REPLACE1_RESULTS.md`, G2) and adds `+0.0283` to five-picosecond
   mobility (`conformance/water_observatory/SLOW1_RESULTS.md`, S2). With mobility as the
   target the fluid's share would be large. The target rule above is doing work, and it is a
   choice (§7).
2. Where no level above exists on the record (the reasoning chains), the target is the
   tier's own next state, and then the organised block is partly predicting ITSELF. That is
   a closure share presented as a price (§9, point 1).
3. It is not an energy and not a mass. The one energy reading on the record is the
   molecule's (`+0.325 ± 0.099` kT a water, `conformance/water_observatory/replace0/README.md`);
   today's review of `conformance/omega/COSMOS_BASE.md` found no rule measured at two tiers in
   one unit (`conformance/omega/COSMOS_BASE_CMB_REVIEW.md` §5). `O` is a proposal for that
   unit in the gate's currency; it says nothing about the cosmological rung, which has no
   instrument.
4. A different unit, the CLOSURE share (how much of the tier's own predictable variance is
   non-conserved), reads large at the physical tiers already and is not what is staked.

## 2. The rank of a tier, fixed before reading

The containment order of `TIERS.md` §1, then §3, then the generated world:

| rank | tier | on the record? |
|---|---|---|
| 1 | the fold below the atom | yes; no gate price |
| 2 | nucleus | declared input |
| 3 | atom → chemistry | yes; no gate price |
| 4 | molecule | yes; the gate has never been run on it |
| 5 | fluid element | yes; the gate's home |
| 6 | continuum | measured once; no dictionary |
| 7 | the living cell, the organism | **NOT ON THE RECORD AT ALL** |
| 8 | reasoning (thought to thought) | yes; one read |
| 9 | the generated world: households, devices, a person's day | yes, in another currency |
| — | the cosmos | a wager; no instrument; not a cell |

The gap at rank 7 is the middle of the lead's sentence ("biological") and nothing here fills
it. A public, permissively licensed carrier for it is OWED and is not designed in this draft.
The QVM's views (`TIERS.md` §2) are a dictionary of representations of one computation, not
tiers of matter; they have certificates and no level above, and they are excluded.

## 3. What is already read: CALIBRATION, not evidence (M-RETRODICTION-AS-PREDICTION)

Every number below was on the record before the claim was made. None of it can count for or
against the claim. Arithmetic on banked numbers only.

| rank | tier | static (carried) | organised (closed) | gate price of the organised block | `O` | record |
|---|---|---|---|---|---|---|
| 1 | the fold | Gauss's law charges | the bound pair's one bit (FERMION-0); the kink (KINK-0) | none: no gate was run | — | `conformance/crystal/FERMION0_RESULTS.md`, `conformance/crystal/KINK0_RESULTS.md` |
| 4 | molecule | the rigid unit, six degrees of freedom | the three vibrations; the hydrogen-bond network | network over velocity: `−0.0001`, `−0.0008`, `+0.0000` on `R²` `0.3992`, `0.3510`, `0.3988`. Vibrations: NOT READ by the gate (an energy, `+0.325 ± 0.099` kT) | network alone: `0.000` | `conformance/replace1/REPLACE1_RESULTS.md` G2; `conformance/water_observatory/replace0/README.md` |
| 5 | fluid, 293 K, 1 ps, target the next density and current | `(ρ, j_L)`: `+0.0622`, `+0.0390`, `+0.1039` | `(q, s2)` and the counts | removal steps `−0.0015`, `−0.0010`; `−0.0031`, `−0.0017`; `−0.0030`, `+0.0033` | `0.000`, `0.000`, `0.031`; standard error about `0.04` to `0.10` | `conformance/replace1/REPLACE1_RESULTS.md`, Amendment 1; `conformance/replace1/OBJECTIVE1_RESULTS.md` §1 |
| 5 | fluid, 400 K, 0.5 ps | `(ρ, j_L)`: `+0.0012` | the counts: `+0.0092 ± 0.0044` | — | not defined: `P_static` is within its error of zero | `conformance/replace1/OBJECTIVE1_RESULTS.md` §1 |
| 5 | fluid, 293 K, target five-picosecond mobility | velocity | tetrahedral order | `+0.0283` (the same on three seeds); `+0.0002` at 400 K | not computable from git: the static block's own `R²` on this target is not printed | `conformance/water_observatory/SLOW1_RESULTS.md` S2 |
| 8 | reasoning chains, target the next thought's process sector | as banked: the process sector (depth, tokens, action), `+0.5754` alone | the conscience block | `+0.0164 ± 0.0081`, null `−0.0013` | `0.028 ± 0.014` | `conformance/replace1/OBJECTIVE1_RESULTS.md` §2 |
| 9 | the generated world | counts (children sum to the parent, exactly) | composition; a dwelling's make-up | none in this unit: households per leaf certified and composition refused; a state differs from its own division in `0.524` of pairs | — | `conformance/scene/HOUSEHOLD0_RESULTS.md`, `conformance/scene/REGIONNORM1_RESULTS.md` |

Three things the table says against the easy reading.

- **Fluid against chains is not a separation.** `0.031 − 0.028` against errors of `0.04` and
  `0.014`. `TIERS.md` §3 already says the chains' sector is "narrow: its price is one SE from
  its null". The two points are two labels.
- **The banked chains partition is not the rule of §1.** Its "process sector" holds tokens
  and the action, which have no balance law. By §1 only the depth counter is static. The
  partition that the rule asks for has not been read (§4, cell R).
- **At 400 K the organised block is the last one standing.** A physical tier, with the
  conserved block priced at nothing: the share is undefined there, not zero.

## 4. The cells that are unread and readable

| cell | rank | what is unread | data | compute | blocked by |
|---|---|---|---|---|---|
| **M** | 4 | the gate on the molecule with the vibrations in the dictionary: static = centre-of-mass velocity and angular velocity; organised = the internal coordinates and their rates, with the network block; target = `Δv_com` over 50 fs (the fluid's momentum, HBOND's target) | the three all-atom walks REPLACE-1's G2 already read (`molsearch1`, `molsearch1_warm_396K`, `molsearch2_5ps`), on disk, not in git | one small driver beside the existing ones; minutes on three cores by REPLACE-1's own timing | a build (barred on the shared box today) |
| **R** | 8 | the chains under §1's partition: static = `{depth}`; organised = `{tokens, action, conscience block}`; target = the next thought's process sector (REPLACE-1's) | the scrubbed release REPLACE-1's driver already reads (the file named accord_traces in RATCHET's scrubbed release, in another repository's tree, read-only); NOT one of the trace directories that need the lead's word, and still the lead's to confirm before it is read again | the same driver | a build; the lead's confirmation |
| W | 4 | OBJECTIVE-1's window on the molecule (`TIERS.md` §6 owes it) | cell M's walks | cell M's driver, swept over budgets | a build; reported, not staked |
| G | 9 | any cell of the generated world in this unit | none on disk: raw data is deleted after each run by rule | a freeze and a fetch of its own | OWED, not designed here |
| T | 8 | a task-level target for the reasoning tier (the honest level above) | the trace directories | — | the lead's word; OWED |
| B | 7 | anything biological | none | — | OWED |

## 5. The stakes

**S_M — the floor.** On each of the three walks, `O_M` as defined in §1. The claim under test
is not the rise. It is the premise the rise stands on: that at a physical tier the organised
portion costs the level above nothing, although it is MORE closed than the object.

- Hypothesis `floor` (held by the two-portion reading and by conservation alone): the
  vibrations add nothing to the next momentum. `O_M` reads `0.00` to `0.02`.
- Hypothesis `share` (a tier-independent share, at the one energy fraction the record has:
  `0.325 / 3 = 0.108 ± 0.033` of the rigid unit's `3 kT`): `O_M` reads `0.075` to `0.141`.
- LOW: `O_M ≤ 0.04` on all three walks. HIGH: `O_M ≥ 0.055` on all three. Anything else is
  BETWEEN, including walks that disagree.

**S_R — one step up.** `O_R` on the chains under §1's partition.

- Hypothesis `rising`: what the organised sectors add beyond the counter is a substantial
  part of what the next thought's process sector can be predicted by. `O_R` reads `0.20` to
  `0.65`. This interval is the author's expectation, NOT derived: the banked total is
  `0.5754 + 0.0164` and the counter's own part of it is unread; no rule on the record turns
  the calibration tiers into a number for this cell.
- Hypothesis `conservation` (reading C of `OBJECT.md`'s table: organisation carries no price
  of its own): beyond the counter the organised sectors add nothing. `O_R` reads `0.00` to
  `0.03`.
- FLAT: `O_R ≤ 0.05`. RISES: `O_R ≥ 0.15`. Between them, BETWEEN.

Because no interval could be derived for `rising`, S_R is in substance an ORDERING stake
(the chains read above the molecule by more than their combined error, and by at least
`0.10`), which is weaker than a predicted number. The bars `0.04`, `0.055`, `0.05` and `0.15`
are proposals for the lead: `0.05` is the largest physical-tier reading on the record
(`0.031`) plus a margin, and `0.15` is three times that. They are set from calibration
numbers, which is allowed, and not from either staked cell.

```prereview
[
 {"kind": "stake", "id": "S_M", "quantity": "O_M, the organised share on the molecule (three walks)",
  "margin": 0.015, "margin_reason": "the network block's increment moves by 0.0008 across the three walks on R2 0.35-0.40, so O moves by about 0.003; five times that",
  "regions": [{"name": "LOW", "hi": 0.04}, {"name": "HIGH", "lo": 0.055}],
  "separates": [["floor", "share"]],
  "table": [{"hypothesis": "floor", "label": "vibrations add nothing to the next momentum", "reads": [0.0, 0.02], "expect": "LOW"},
            {"hypothesis": "share", "label": "a tier-independent share at the energy fraction 0.108 +- 0.033", "reads": [0.075, 0.141], "expect": "HIGH"}]},
 {"kind": "plant", "id": "P-CLOSED-IRRELEVANT", "stake": "S_M", "lands": 0.0, "must": "LOW"},
 {"kind": "plant", "id": "P-CARRIED-M", "stake": "S_M", "lands": 0.626, "must": "HIGH"},
 {"kind": "stake", "id": "S_R", "quantity": "O_R, the organised share on the chains under the rule's partition",
  "margin": 0.015, "margin_reason": "the conscience block's standard error 0.0081 on a total of 0.59",
  "regions": [{"name": "FLAT", "hi": 0.05}, {"name": "RISES", "lo": 0.15}],
  "separates": [["rising", "conservation"]],
  "table": [{"hypothesis": "rising", "label": "tokens, action and conscience add beyond the counter (author's expectation, not derived)", "reads": [0.20, 0.65], "expect": "RISES"},
            {"hypothesis": "conservation", "label": "beyond the counter, nothing", "reads": [0.0, 0.03], "expect": "FLAT"}]},
 {"kind": "plant", "id": "P-CARRIED-R", "stake": "S_R", "lands": 0.626, "must": "RISES"},
 {"kind": "plant", "id": "P-REDUNDANT", "stake": "S_R", "lands": 0.0, "must": "FLAT"},
 {"kind": "plant", "id": "P-FLUID-KNOWN", "stake": "S_R", "lands": 0.031, "must": "FLAT"},
 {"kind": "branches",
  "outcomes": {"unit": ["defined", "undefined"], "S_M": ["LOW", "BETWEEN", "HIGH"], "S_R": ["FLAT", "BETWEEN", "RISES"]},
  "branches": [{"id": "not-comparable", "when": {"unit": ["undefined"]}},
               {"id": "trend-holds", "when": {"unit": ["defined"], "S_M": ["LOW"], "S_R": ["RISES"]}},
               {"id": "flat", "when": {"unit": ["defined"], "S_M": ["LOW"], "S_R": ["FLAT"]}},
               {"id": "reversed", "when": {"unit": ["defined"], "S_M": ["HIGH"], "S_R": ["FLAT"]}},
               {"id": "share-everywhere", "when": {"unit": ["defined"], "S_M": ["HIGH"], "S_R": ["RISES"]}},
               {"id": "between-floor", "when": {"unit": ["defined"], "S_M": ["BETWEEN"]}},
               {"id": "between-step", "when": {"unit": ["defined"], "S_M": ["LOW", "HIGH"], "S_R": ["BETWEEN"]}}]},
 {"kind": "numbers", "cited": [
   {"value": "+0.0164 ± 0.0081", "source": "conformance/replace1/OBJECTIVE1_RESULTS.md"},
   {"value": "+0.5754", "source": "conformance/replace1/OBJECTIVE1_RESULTS.md"},
   {"value": "+0.0622", "source": "conformance/replace1/REPLACE1_RESULTS.md"},
   {"value": "+0.0390", "source": "conformance/replace1/REPLACE1_RESULTS.md"},
   {"value": "+0.1039", "source": "conformance/replace1/REPLACE1_RESULTS.md"},
   {"value": "+0.1041", "source": "conformance/replace1/REPLACE1_RESULTS.md"},
   {"value": "+0.0033", "source": "conformance/replace1/OBJECTIVE1_RESULTS.md"},
   {"value": "+0.0092 ± 0.0044", "source": "conformance/replace1/OBJECTIVE1_RESULTS.md"},
   {"value": "0.3992", "source": "conformance/replace1/REPLACE1_RESULTS.md"},
   {"value": "+0.325 ± 0.099", "source": "conformance/water_observatory/replace0/README.md"},
   {"value": "+0.0283", "source": "conformance/water_observatory/SLOW1_RESULTS.md"},
   {"value": "0.524", "source": "conformance/scene/REGIONNORM1_RESULTS.md"},
   {"value": "0.108", "source": "derived: 0.325 / 3"},
   {"value": "0.626", "source": "derived: 0.1041 / (0.1041 + 0.0622)"},
   {"value": "0.028", "source": "derived: 0.0164 / (0.0164 + 0.5754)"},
   {"value": "0.031", "source": "derived: 0.0033 / (0.0033 + 0.1039)"}]},
 {"kind": "new_files", "paths": ["engine/crates/holon-closure/examples/tier_trend0/main.rs"]}
]
```

## 6. Gates, plants and branches

**The unit gate, G-UNIT (it decides `defined` or `undefined`, mechanically, before either
stake is read).** At each cell: the static block, the organised block and the target are the
ones written in §4, typed into the driver before the first read; `P_static` is at least five
of its own jackknife errors above zero and above its re-paired null on every walk (cell M)
and on the chains (cell R); and the three plants below fire on that cell's own carrier. If
any clause fails at either cell, the unit is `undefined` and the branch is NOT COMPARABLE,
whatever the two numbers read.

**Plants, each with its paper value.** Each plant acts in the organised block of the
dictionary, on the target the stake is graded on; that is the sector the plant acts on, and
the driver asserts the planted column is nonzero in that block before any plant is scored.

- **P-CARRIED** (M and R): PO-4, REPLACE-1's planted field at unit amplitude, added to the
  organised block. On the fluid carrier it is priced `+0.1041` over a static block worth
  `+0.0622`, so `O = 0.1041 / (0.1041 + 0.0622) = 0.626`. It must read HIGH on M and RISES
  on R. Its amplitude on each new carrier is set so that its own planted price is that
  fraction of the cell's `P_static`; the gate must recover it within `0.05`.
- **P-CLOSED-IRRELEVANT** (M): the same field drawn from another axis's chain, closed and
  unrelated to the target (REPLACE-1's PR-2, read `−0.0056`): `O = 0.000`, LOW.
- **P-REDUNDANT** (R): an affine copy of a static column (REPLACE-1's PR-3, exactly `0`):
  `O = 0.000`, FLAT.
- **P-FLUID-KNOWN** (R's scale): the fluid's own structural and count blocks re-read by the
  new driver, `O = 0.000`, `0.000`, `0.031`: FLAT. This is a reproduction of calibration
  and says only that the new driver computes `O` as §3 does.

**Branches, exactly one per outcome.**

- **NOT COMPARABLE** (the unit is undefined at either cell). The claim is not yet a
  measurable in the gate's currency, and that is the finding. Nothing moves in any table.
- **TREND HOLDS** (S_M LOW at `≤ 0.04`, S_R RISES at `≥ 0.15`): on two cells, four ranks apart, with the reasoning
  cell's target a stand-in. Reading A″ of `OBJECT.md`'s table gains over C; the entry says
  "one step, not a trend".
- **FLAT** (S_M LOW at `≤ 0.04`, S_R FLAT at `≤ 0.05`): the organised portion costs the level above nothing at
  either tier. Conservation alone (reading C) gains.
- **REVERSED** (S_M HIGH at `≥ 0.055`, S_R FLAT at `≤ 0.05`): the physical tier's share is the larger. The claim is
  wrong in sign on this record.
- **SHARE EVERYWHERE** (S_M HIGH at `≥ 0.055`, S_R RISES at `≥ 0.15`): the organised portion is priced at both
  tiers; the premise that it is near zero at physical tiers is wrong; the rise is undecided.
- **BETWEEN** (S_M within `(0.04, 0.055)` or the walks disagreeing, or S_R within `(0.05, 0.15)`; entered as `between-floor` or `between-step`):
  nothing is concluded; the two numbers are recorded.

## 7. The counterexample already on the record, and the cosmos

**Tetrahedral order.** SLOW-1 found a non-conserved sector that IS carried at a physical
tier: `+0.0283` of five-picosecond mobility beyond velocity and every other candidate at
293 K, on three seeds, and `+0.0002` at 400 K (`conformance/water_observatory/SLOW1_RESULTS.md`).
For the target the continuum carries (the next density and current) the same sector adds
nothing (ORDER-1, reproduced by REPLACE-1). So "the organised portion is priced near zero at
physical tiers" is true for one target and false for another on one liquid, and it turns on
with cooling. Two readings: the claim needs "to the level above's own carried state" in it
(then mobility is not the continuum's state, and the floor stands); or the organised
portion's price is a property of the TARGET and the temperature at least as much as of the
tier (then a trend in tier is confounded, and S_M's result at one temperature and one target
says little). This draft takes the first and cannot exclude the second. A reviewer should
decide whether a mobility target belongs in cell M as a second arm.

**The cosmos.** Not a cell. No instrument reads an organised share there, and today's five
notes found none forced (`conformance/omega/COSMOS_BASE_CMB_REVIEW.md`). If the trend held
on carriers it would give the wager a direction (small today, growing with what the higher
tiers do) and no number.

## 8. Prior art

**Chaisson's energy rate density.** E. J. Chaisson, "Energy rate density as a complexity
metric and evolutionary driver", Complexity 16 (2011) 27 (published online 2010), and its
sequels: free energy flowing through a system per unit time per unit mass, in erg/s/g: the
Milky Way about `0.5`, the Sun about `2`, the Earth's geosphere about `75`, plants about
`900`, animals about `40,000`, human society about `500,000` (read at a search summary of
the paper's table, 2026-11-23, not at the paper). That IS "more dynamics per gram at higher
tiers", by five to six orders, and it is fifteen years old. What is not the same: his is a
FLOW of free energy per mass through an open system; `O` is a PRICE, what a tier's
non-conserved structure adds to the level above's prediction of its own next state. A star
has a large flow and, if the floor holds, an organised share near zero. The two would agree
in direction and measure different things; neither implies the other. Summed over mass, his
numbers also show why the cosmic total today stays small: the high-flow tiers hold
negligible mass.

**The instrument** is credited in `TIERS.md` §5: a slow-mode search inside a stricter
discipline. The gate, its ordered admission and its window are this record's.

## 9. Open to the reviewer — weakest first

1. **Cell R's target is the tier's own next state.** With tokens and the action in both the
   organised block and the target, `O_R` is likely to read high for a reason that has
   nothing to do with carriage: things predict themselves. `rising` may be a foregone
   conclusion and `conservation` a straw man. The honest target is a task-level outcome
   (cell T), which needs the lead's word. If the reviewer agrees, S_R should be cut and the
   campaign frozen as the floor alone.
2. **The static/organised split is a rule the author wrote today** (an exact balance law, or
   a counter). It moves the chains' share from `0.028` (as banked) to something unread. A
   different rule (anything the level above carries is static) makes every share zero by
   definition.
3. **`rising`'s interval is not derived.** No rule turns calibration into a number for R.
4. **Two cells are not a trend,** and ranks 6, 7 and 9 are empty in this unit.
5. **Cell M at one target and one lag.** The cross-block coupling of the vibrations rises
   from `0.009` at 5 fs to `0.25` at 100 fs on the record
   (`conformance/water_observatory/MOLSEARCH1_RESULTS.md`); the 50 fs target sits inside
   that rise, and the floor may hold at 50 fs and fail at 100.
6. **P-CARRIED's amplitude is tuned per carrier** to land at `0.626`; it shows the gate can
   read a share, not that it reads the right one for a natural sector.
7. **The fluid's calibration error (`0.04` to `0.10`) is larger than the effect S_R's bars
   resolve.** The chains have 444 transitions; `se(O_R)` should be near `0.02` but is unread.
8. **The author expects S_M LOW and S_R RISES** and designed the cells after seeing the
   calibration. The stake is blind in its two numbers and not in its design.

## 10. What it would earn, and compute

Against the bar of 2026-11-13 (a forced derivation of a nontrivial piece of known physics):
nothing. This is a measurement in a new unit. Against the purpose sentence of 2026-11-23
("to identify out of all this mess what is real"): it would say whether "the organised
portion matters more to the level above at higher tiers" is a number or a way of talking,
and S_M alone would say whether the two-portion reading's premise at a physical tier
survives on a tier the gate has never seen. FLAT would be the first evidence for
conservation alone since the table was rewritten.

Compute, as tasks; nothing runs until the shared box is free and the lead says so: (1) the
driver `engine/crates/holon-closure/examples/tier_trend0/main.rs`, with the blocks and targets
of §4 typed in and committed before any read; (2) the plants on the synthetic carrier in CI;
(3) G-UNIT on each cell; (4) cell M on three walks; (5) cell R on the chains, after the
lead confirms the release may be read again; (6) cell W reported.

---
witness: `Closed` (Object.lean), cited in words; `O` is measured, and no theorem states it
witness: none (S_M and S_R are measured)
**misfits:** M-RETRODICTION-AS-PREDICTION, M-SCREEN-SCOPE-UNSTATED, M-SORTS-NOT-SEPARATES, M-PLANT-OBS, M-PLANT-SECTOR, M-JOINT-PASS-REGION, M-BAR-FROM-THE-READ, M-POPULATION-CHOICE, M-VACUOUS-SUCCESS, M-IDLE-CALIBRATED-TIMEOUT, M-FOREIGN-DOMAIN-CORROBORATION, M-NULL-MISSTAKE. These are the registered ids this text contacts.

- M-RETRODICTION-AS-PREDICTION: every banked number is labelled calibration (§3); the stakes are on two cells no one has read.
- M-SCREEN-SCOPE-UNSTATED: §0 says what was searched and what was not.
- M-SORTS-NOT-SEPARATES: each stake carries a table under two hypotheses with its bars a stated margin from every cell; §3 says outright that fluid against chains does not separate.
- M-PLANT-OBS and M-PLANT-SECTOR: each plant has a paper value and acts in the organised block on the graded target; P-CARRIED is REPLACE-1's own plant re-used.
- M-JOINT-PASS-REGION: the gate, both stakes and the plants are jointly satisfiable at (LOW, RISES) with P-CARRIED at 0.626 and the two null plants at 0; shown in the block.
- M-BAR-FROM-THE-READ: the bars come from calibration cells, and none from M or R.
- M-POPULATION-CHOICE: the tiers excluded (the QVM views, the cosmos, ranks without a dictionary) and the cells owed (G, T, B) are named.
- M-VACUOUS-SUCCESS: G-UNIT refuses a cell whose static block is not priced, so a share of zero over nothing cannot pass as a floor; point 1 of §9 names the opposite vacuity at R.
- M-IDLE-CALIBRATED-TIMEOUT: the fluid's numbers are from 293 K at 1 ps on one target and are not carried to another regime; the 400 K cell is shown undefined.
- M-FOREIGN-DOMAIN-CORROBORATION: Chaisson's trend is credited as a different quantity and is not counted for the claim.
- M-NULL-MISSTAKE: the nulls are the gate's re-paired ones on the graded target, read at every step as banked.
