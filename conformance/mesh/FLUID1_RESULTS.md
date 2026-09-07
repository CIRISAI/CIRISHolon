# FLUID-1 — results: the orientation lattice, read

*Freeze `FLUID1_PREREG.md` (a91ccab, alone, before any run). Instrument
`holon-lattice/src/orientation.rs` (delegate; the previous build was killed by a machine
reboot mid-way with 8 of 12 unit tests failing, and a second delegate finished it: one rule
completed — the bonded-streaming pass walks the bonds in an order fixed before any hop and
each hop rewrites both of the moved particle's bonds — and one measurement corrected, never a
target). Runner `examples/fluid1_lattice.rs` (`gate`, `run`, `read`). Records under
`conformance/mesh/fluid1/`: `amplitude_table.json`, `gate.json`, `price.json`, `gate.log`,
`run.json`, `run.log`, `run.done`, `read.json`; every file validates as JSON. Cores 24–31;
`gate` 152 s, `run` 953 s of wall, about 2,000 core-seconds, every run inside its own price.*

## The verdict, first

**A pair is not enough of a closure. Bonds read from the molecular map, held by the rent
clause at the dimer's own retention, trap the particle — the tracer's diffusion falls by
40 % — and leave the momentum where it was: the Schmidt number doubles from FLUID-0's 0.108 to
0.21, three decades short of water's 435. S BRANCH (c), B BRANCH (c), W VOID by the freeze's
letter with a reading beside it, both plants FAIL by the freeze's own arithmetic, the four
instrument gates PASS.**

1. **The rule is the map, and the rent clause holds on the lattice (G0).** The amplitude
   table is read at run time from CT-1's and CT-2's records — `1, 1.337, 1.288, 1.420, 1.288,
   1.337` at 0°–300° — with the map's rise at 60° and 120° exact on the read values; the chart
   `E₀/kT_lat = 0.2112` is set from the dimer's read retention `0.5526`, and the lattice gives
   that retention back at `0.5525` on a held link and the chart's break probability at
   `0.80960` against `0.80963` over 39.9 million in-lattice break tests. G3's detailed balance
   holds at 0° and 120° to `2e-4` and `1.2e-3`.
2. **With bonds forbidden the carrier IS FLUID-0 (G1).** Not within 5 %: bit-identical on all
   five readouts in both chiralities (`ν = 0.5284306411` against FLUID-0's `0.5284306411`).
   The ledger is integer-exact on 8,000 step-checks including the per-orientation census and
   the bond balance (G2).
3. **Trapping halves diffusion and does nothing for stress (S (c)).** At the chart `D(k=1)`
   is `2.93 / 2.84` against FLUID-0's `4.88`; `ν(k=1)` is `0.577 / 0.592` against `0.528`;
   `Sc = 0.197 / 0.209`. The cold control, no rent paid at all, reaches `Sc = 0.33 / 0.34`.
   Water's hundreds are momentum carried across a network while particles wait; a bonded pair
   that streams as one carries its momentum with it and transmits none across. The network's
   dynamics must be in the carrier (§3 of the freeze, branch (c)'s own words).
4. **The bond graph never spans (B (c)).** `0.289` bonds per particle counted once (a lattice
   ceiling of 1, one donor arm per particle; the box's `1.18` is on a ceiling of 2, so the
   lattice sits at 29 % of its ceiling against the box's 59 %); the largest component is
   `8e-5` of the particles — pairs and short chains — and no sampled step spans.
5. **The lattice, not the rent, is the ceiling on the bond.** Of every bond released at the
   chart, 64 % go to the rent clause and 36 % to the instrument's own exclusion: a bonded
   pair's joint move needs a vacancy and a common mover, and when it has neither the bond is
   released as `blocked`, never folded into a rent break. In the cold control the rent
   releases nothing and the exclusion releases every bond formed: 3.65 million formed,
   3.65 million blocked, `0.45` bonds per particle at a break probability of zero. On an FHP-6
   torus at `d = 0.2` with no rest particle the closure cannot wait, and that is why the count
   stays low whatever the rent.
6. **The block chart's defect grows with a closure in the carrier, and the freeze's letters do
   not cover it (W VOID).** The witness rate sits ABOVE node LG's `W(b) = 1 − (b−2)²/b²` at
   every scale — `4.5, 8.2, 8.6, 10.9 %` at `b = 4, 8, 16, 32` — with the excess growing as
   the block grows, so it is neither (a) within 10 % everywhere (b = 32 is outside), nor (b)
   above and converging, nor (c) below. Reading beside the letter: bonds carry a perturbation
   from an interior cell to the block's face, so the defect penetrates a bond's correlation
   length into the block, and the boundary fraction understates it by more the larger the
   block — a closure in the carrier makes the block LESS closed than its boundary, by a
   margin that scales.
7. **Both plants fail by arithmetic the freeze could have done (P(i), P(ii)).** The table
   enters only through `p_break = exp(−A·E₀/kT)`; with `A ∈ [1, 1.42]` at this chart the
   per-angle retention spans `0.5526`–`0.5744`, so flattening the table can move the count by
   at most a few percent (measured `1.7 %`, stake 20 %). Doubling the rent moves a logistic
   at `0.5526` by exactly `0.0852` (measured `0.0856`, stake 20 %). Both carriers are present
   and both observables move the way the rule says; both stakes were typed with no arithmetic
   behind them — FLUID-0's lesson, again (stake-gates-from-the-freeze's-own-arithmetic).
   Reported as the letter reads: FAIL, kept, targets not moved.

| gate | verdict | the number |
|---|---|---|
| G0 — the rule is the map | **PASS** (5 legs) | `A = [1, 1.337178, 1.288496, 1.420097, 1.288496, 1.337178]`; 0° from `ct2/gd0_linear_R2.9.json:e_full − ct1/sector_linear_R2.9.json:e_noct` (the 2.9 Å tilt family has no 0° node), 60/120/180° from `ct2/node_tilt_R2.9_t{60,120,180}.json:e_ct`, 240/300° by tilt symmetry; `f = 0.5526` from `ct2/arms.json:dimer_293_seam.f`; `E₀/kT_lat = 0.211181`; held-geometry retention `0.552500` (rel `1.8e-4`); run-measured `p_break(0°) = 0.8095971` vs `0.8096272` (rel `3.7e-5`) |
| G1 — the no-bond control is FLUID-0 | **PASS** (14 legs) | `ν(k1), ν(k2), D(k1), D(k2), Sc` bit-identical to `fluid0/gate.json` in both chiralities; all eight readouts `Fitted` |
| G2 — the ledger, exact | **PASS** (117 legs) | 16 runs × 500 steps at `L = 256`, 8,000 step-checks: mass, `P_x`, `P_y`, red, orientation total, orientation census, bond balance `formed − broken_rent − blocked = held` |
| G3 — detailed balance | **PASS** (4 legs) | 0°: `0.552500` vs `1/(1+p) = 0.552600` (rel `1.8e-4`, 110,494 tests, 2 live links — the reverse link forms by the angle's definition, reported, never subtracted); 120°: `0.568300` vs `0.567610` (rel `1.2e-3`); 10⁵ steps each, streaming off |
| P(i) — the acceptor factor flattened | **FAIL by letter** | count `0.289440 → 0.284384`, change `1.7 %` (stake 20 %); carrier `A(120°) − 1 = 0.288 ≥ 0.2`; analytic reach of the table `0.038` in retention |
| P(ii) — the rent doubled | **FAIL by letter** | retention `0.552500 → 0.604200`, change `8.6 %` (stake 20 %); carrier `0.5526 ∈ (0,1)`; a doubling's exact reach `0.0852`; beside: `p_break` moved `19.0 %`, the count `7.8 %` |
| P — the price | **PASS** | orientation overhead `6.63×` (chart step `8.27` ms, plain `1.25` ms); FLUID-0's `25.8` s per law → `170.9`; ten priced runs, actual/price `0.89`–`1.00` (band `0.1`–`10`); `run` 953 s wall |
| B — the bond count and the phase | **BRANCH (c)** | `0.2894` per particle (each bond once; lattice ceiling 1) = `0.5789` by degree (ceiling 2); box `1.1843` (each bond once, ceiling 2; `liquid1/arm.json`); band `[0.79, 1.78]` not entered; spanning steps `0/200`; largest component `7.8e-5` |
| S — the Schmidt stake | **BRANCH (c)** | chart `Sc = 0.1973 / 0.2086` (chiralities), all four readouts `Fitted`, `R² ≥ 0.994`; cold control `0.3290 / 0.3434`; no-bond `0.1083 / 0.1103`; water `434.8` is the kill (FLUID-0 §1's sources) |
| W — the block chart's defect | **VOID by letter; reading in §2** | rates `0.7853, 0.4765, 0.2563, 0.1359` (± `0.005`) vs `W(b) = 0.7500, 0.4375, 0.2344, 0.1211` at `b = 4, 8, 16, 32`; relative `+0.045, +0.082, +0.086, +0.109`; 7,131 movable-cell probes per `b` |

## 1. The transport, for the record

| run | chirality | ν(k=1) | ν(k=2) | D(k=1) | D(k=2) | Sc | bonds/particle |
|---|---|---|---|---|---|---|---|
| chart | + / − | 0.577 / 0.592 | 0.512 / 0.514 | 2.925 / 2.839 | 3.006 / 2.986 | 0.197 / 0.209 | 0.289 |
| cold (p_break = 0) | + / − | 0.872 / 0.929 | 0.707 / 0.725 | 2.651 / 2.704 | 2.739 / 2.717 | 0.329 / 0.343 | 0.451 |
| no-bond | + / − | 0.528 / 0.527 | 0.539 / 0.521 | 4.881 / 4.779 | 5.125 / 5.299 | 0.108 / 0.110 | 0 |
| plant (i) A ≡ 1 | + / − | 0.585 / 0.569 | 0.510 / 0.503 | 2.914 / 2.873 | 3.045 / 3.030 | 0.201 / 0.198 | 0.284 |
| plant (ii) rent × 2 | + / − | 0.621 / 0.638 | 0.529 / 0.565 | 2.766 / 2.803 | 2.898 / 2.960 | 0.225 / 0.228 | 0.314 |

(`run.json`; link² per step; every readout `Fitted`, the two wavevectors within the 10 % gap.)
The pattern across the five: `D` falls with the bond count and `ν` rises with it, both
gently; the cold control's `ν` is the largest move in the table (+65 %) and still leaves `Sc`
under one.

## 2. What it means for the element

The freeze asked whether closure ALONE — a pair bound by the rent clause — buys water's
ratio. It buys a factor of two, from the tracer side only. FLUID-0's finding stands
sharpened: what water's `Sc` needs is momentum carried ACROSS bonds while particles wait,
i.e. a network that transmits stress, and the pair transmits none. Two things follow for the
carrier of the next campaign, both instrument findings rather than physics:

- **The closure must be able to wait.** On FHP-6 there is no rest state, so a bonded pair
  refused its joint move has to break; at `d = 0.2` that release is 36 % of the chart's and
  100 % of the cold control's. A carrier with rest particles (FHP-II/III) lets a blocked pair
  hold, and the rent clause then sets the lifetime instead of the exclusion.
- **The count is capped by the arm, not the rent.** One donor arm per particle puts the
  each-bond-once ceiling at 1; the liquid's is 2. Two arms per particle (the water's two
  hydrogens) is the lattice's version of the molecular tier's ceiling.

The W reading is the first measured statement about a coarse chart on this tier that is
not the boundary fraction: a closure in the carrier makes the block's defect grow with the
block. EDGE-2 (`GANTT2.md`, the edge) takes it as its starting point, since the edge of a
holon at resolution `b` is exactly the set of cells whose fiber the dynamics splits.

## 3. Corrections to the freeze's letter

- **B's band crossed conventions.** The band `[0.79, 1.78]` was 1.5× the box's `1.18`, which
  is an each-bond-once count on a ceiling of 2; the lattice's each-bond-once ceiling is 1 (one
  donor arm), so the band's upper half was unreachable by construction. The verdict stands
  by the letter; the ceiling fractions (29 % against 59 %) are the comparable numbers.
- **The plants' stakes were typed.** Both are computable from the freeze's two read inputs
  before any run, and both are out of reach. A plant's stake must be derived from the reach
  of the observable it acts on.
- **W's letters had no branch for "above and diverging".** Recorded as VOID with the reading
  beside it, never forced into (b).
- **The 0° node.** The 2.9 Å tilt family starts at 60°; the linear node's `E_CT` is built from
  CT-2's `gd0_linear_R2.9` total and CT-1's `sector_linear_R2.9` no-transfer total, by the
  tilt records' own rule. Both records and fields print with the table.

## 4. Bookkeeping, declared

- The instrument's four declared rules (the collision's slot remap; a bond pays no rent on
  the step it forms, which is what makes the stationary held fraction `1/(1+p_break)` and is
  confirmed by G3; bonded streaming keeps labels and moves positions, with `blocked` counted
  apart; formation as a fixed-order greedy pass) are in the module header, and the second
  delegate's completion of the third is recorded there and here.
- The only numbers from outside the crate are the amplitude table and the retention READ
  from CT-1/CT-2's records, and water's Schmidt number, named as the kill where it appears.
- The G3 scene at 0° carries two live links (the reverse link forms by the angle's
  definition); the held fraction is the tracked donor's role counted once per step, and the
  second link is reported, never subtracted. The first build's measurement counted both and
  read exactly `2/(1+p)`; the measurement was corrected, the target was not.
