# RUNG 2 / A2 — THE LATTICE-GAS CHART: results

*Stakes: `RUNG2_PREREG_A2.md`, frozen and committed (`e5bd812`) before a line of the A2
instrument was written. Instrument: `holon-lens/src/field_lg.rs` + `holon-mesh/examples/rung2_lg.rs`
+ `holon-mesh/tests/rung2_lg_pin.rs` (`92c7482`). Log: `rung2_lg.log`. This document does NOT
edit `RUNG2_RESULTS.md`; it sits beside it and says what changed.*

---

## A2.0 THE HEADLINE

### THE VERDICT STANDS, THE LATTICE-GAS CHART READS WORSE, AND THE REASON IS A SECOND SCISSOR

**Branch A2-(i) AND A2-(iii) together.** The lattice-gas `(N, P)` chart fails admissibility
on the same 75 of 75 cells and reads `NotClosed` on every live cell — the verdict census is
**bit-identical** to the banked cell-field chart, 183 `VoidVacuous` and 42 `NotClosed` — and
at comparable arity its defect is **3.4× to 9.2× worse**.

| arm / grid | rung pair | cell-field | lattice-gas | ratio |
|---|---|---|---|---|
| fenced 6×4 | density (`Occ` vs `W1`) | D = 0.0476, n_c = 9.65e5 | D = 0.4359, n_c = 3.88e5 | **9.16×** |
| fenced 6×4 | density+momentum (`Mom` vs `W2`) | D = 0.2241, n_c = 3.57e4 | D = 0.9473, n_c = 3.72e3 | **4.23×** |
| hydrogen 6×4 | density (`Occ` vs `W1`) | D = 0.0700, n_c = 5.05e5 | D = 0.2548, n_c = 3.18e5 | **3.64×** |
| hydrogen 6×4 | density+momentum (`Mom` vs `W2`) | D = 0.2294, n_c = 8.74e4 | D = 0.7886, n_c = 1.07e4 | **3.44×** |

Collision counts are printed beside every ratio because the two charts have different
denominators; `RUNG2_RESULTS.md` §4 is the reason that is not optional.

**MY STAKED PREDICTION HELD.** A2 §5 staked, before the instrument existed, that the G2
admissibility verdict does not move because the occupancy/transport scissor is a property of
the CARRIER. It does not move: the G2 column and the vacuity census are identical between
the two charts, because both read the same cells over the same trajectories. The scissor is
now **chart-independent**, which strengthens `RUNG2_RESULTS.md`'s branch (d) rather than
qualifying it.

### THE NEW FINDING: A BOOLEAN FHP-6 WORD CANNOT HOLD A FLUID ELEMENT

This is the thing A2 learned that the banked campaign could not, and it is a fact about the
**seam** — the molecular-to-lattice map — not about lattice gases.

FHP is an exclusion automaton: one particle per mode, six modes, so a cell's Boolean word
carries **at most 6 atoms**. Measured saturation, spatial chart, across all 15 trajectories:

| grid | atoms/cell | saturation (some mode holds ≥2) | atoms lost to the Boolean word |
|---|---|---|---|
| 1×1 | 12.0 | **1.000 – 1.000** | **0.546 – 0.561** |
| 2×1 | 6.0 | 0.781 – 0.942 | 0.330 – 0.380 |
| 2×2 | 3.0 | 0.396 – 0.489 | 0.166 – 0.242 |
| 4×2 | 1.5 | 0.144 – 0.292 | 0.078 – 0.158 |
| 6×4 | 0.5 | 0.066 – 0.149 | 0.043 – 0.096 |

**The exclusion cap and the fluid-element requirement pull in opposite directions, exactly
as the occupancy/transport scissor does.** At 12 atoms per cell the Boolean word already
loses 55% of them and every occupied cell-frame is saturated. G2's bar is ≥ 100 atoms per
cell; a cell at that density would deliver at most 6 of them to the word, discarding ~94%.
A Boolean FHP-6 occupancy is faithful only where a cell holds a handful of atoms pointing in
distinct directions — which is the opposite of a fluid element.

**This is `Core/ModeChart.lean`'s own fence arriving as a measurement**, and it names the
repair without this campaign having to invent one: *Boolean occupancy is exact only for
DETERMINATE states; over mixtures the exact invariant is the CAP — mean occupancy in
`[0,1]`.* The chart that can carry a fluid element is the **fractional / mean-occupancy**
one (`meanOcc_le_one`, `meanOcc_fractional_exists`) — the lattice-Boltzmann regime — not the
Boolean word this map builds. That is a different chart with a different fence, and it is
an input to the LG node, not a repair available to rung 2.

---

## A2.1 THE GATES

| gate | reading | verdict |
|---|---|---|
| **A1** map non-degeneracy (≥ 8 distinct words) | 27–64 distinct words on every grid; 225/225 cells ok | **PASS** |
| **A2g** saturation disclosure | table above; **zero-velocity atoms across the whole campaign: 0**, so the no-rest-mode loss is exactly nil and the exclusion loss is the whole cost | **DISCLOSED** |
| **A3** phase-resolved defect (door c) | **0 of 75 sweeps found a grain boundary**; best `D_A` over every `(p, r)` with an adequate work count is **0.466** at `p = 8, r = 5` | **EMPTY — the staked NULL** |
| **A4** one-variable comparison | table in §0 | **lattice-gas WORSE, 3.4–9.2×** |
| **G2** admissibility | identical to the banked chart, 0/75 admissible | **FAIL, unchanged** |
| **G3** vacuity fence | 183/225 spatial cells VOID, identical census | **FIRED, unchanged** |
| **G5** Leg A | 42 live cells, all `NotClosed`; medians `W1` 0.436/0.255, `W2` 0.947/0.789, `W3` 0.936/0.786 (fenced/hydrogen at 6×4) | **NOT CLOSED** |
| **G7** control floor | **inverts against the banked chart** — see §2 | **FAILS at `W2`/`W3`** |
| **G8** ladder self-check | strong form `refines` true **675/675**; weak form true on every cell | **PASS — never quoted as support** |
| **G11 / A2.7** cost | 13,500,000 chart evaluations against a modelled 19,200,000 | **UNDER the model, and explained — §4** |

---

## A2.2 THE CONTROL INVERTS, AND THAT IS THE SEAM SPEAKING

`RUNG2_RESULTS.md` found the cell-field chart separating strongly from the
coherence-destroying control at the MOMENTUM and ENERGY rungs (+0.598, +0.288) and not at
all at the density rung (−0.002, wrong sign). The lattice-gas chart does the **opposite**:

| chart | density rung | density+momentum rung |
|---|---|---|
| cell-field | −0.0019 (0/7 clear) | **+0.5984 (7/7 clear)** |
| lattice-gas | **+0.1093 (5/7 clear)** | −0.0366 (2/7 clear) |

*(hydrogen arm, 6×4, blind minus spatial; fenced arm: lattice-gas `W1` −0.0907, `W2` +0.0014, both 0/6.)*

The reading, and it follows from §0: the cell-field chart's separation came from the
momentum field, and the exclusion cap is what destroys it. `W2`'s `P` is a sum of unit
vectors over OCCUPIED modes, so once a mode saturates the chart cannot tell one atom from
four, and the spatial coherence the cell-field momentum measured is gone. The lattice-gas
chart's only surviving discrimination is at `W1`, the mode count — and even that clears the
bar on 5 of 7 hydrogen seeds and 0 of 6 fenced ones.

**So neither chart discriminates at both rungs, and they fail in complementary places.**
Nothing here certifies; what it establishes is that the lossy step is the map.

---

## A2.3 DOOR (c) IS A MEASURED NULL, AS STAKED

`RUNG2_PREREG_A2.md` §4 staked in advance: **A3 returns EMPTY.** It does.

Across 75 sweeps — 15 trajectories × 5 grids, periods `p ∈ {1,2,3,4,6,8}`, every residue,
each requiring 200 informative transitions of its own — **no `(p, r)` reached `D_A = 0`**.
The best any phase achieved was 0.466, which is 23× the budget. There is no step at which
this chart is free.

Door (c) therefore displays **"no free refresh point measured"** and NOT a period. That is
the honest readout and it respects `grain.rs`'s fence: a period belongs to the coupling that
measured it, never to nature and never to the engine, and `Grain::measured` refuses an
unprovenanced schedule. An empty `exact_at` is not constructible-usable in any case —
`Grain::steps_to_close` panics on it — so there is nothing for the page to show but the
null.

**Door (b) stands as named**: the defect against the `(N, P)` chart, displayed beside
β = 0.02 **and beside the measured saturation**, because §0 shows the defect without the
saturation would be a number without its scope. On current evidence door (b) reads
0.79–0.95 at the operator's chart — visibly not closed, which is the content.

**Neither door may display a number implying the band is live.** Rung 2 did not certify.

---

## A2.4 COST, AND WHY IT CAME IN UNDER ITS MODEL

13,500,000 chart evaluations against A2 §7's modelled 19,200,000. A result arriving cheaper
than its own banked cost model is not that result (`M-CHEAPER-THAN-ITS-PRICE`), so the
shortfall is accounted to the unit rather than waved at:

* the model assumed **4** chart kinds; the run used **3** — `BlindIndex` was omitted because
  `RUNG2_RESULTS.md` §5.4 already established it is degenerate (constant membership, zero
  transport, one distinct reading) and re-running a control already proven inert buys
  nothing;
* the model assumed **16** trajectories; **15** were read, because hydrogen seed
  `0x53415425` REFUSED at R1 exactly as it did in the banked run.

15 × 20,000 × (5 grids × 3 rungs × 3 kinds = 45) = **13,500,000**. The arithmetic closes
exactly.

---

## A2.5 WHAT A2 CHANGES, AND WHAT IT DOES NOT

**Changed:** nothing in `RUNG2_RESULTS.md`'s verdict. Branch (d) stands, and now stands on
two charts rather than one.

**Added:**
1. the scissor is **chart-independent** — a staked prediction, confirmed;
2. a **second scissor**, specific to the lattice-gas seam: the exclusion cap and the
   fluid-element requirement are in direct conflict, quantified at 55% of atoms lost at 12
   per cell and ~94% at the admissibility bar;
3. door (c) is a **measured null** — a staked prediction, confirmed;
4. the two charts' controls **fail in complementary places**, which locates the loss in the
   map rather than in either chart.

**The operator's rule, discharged as it was written.** The instruction was that any
deviation from the existing machinery must be argued against it **by measurement**. The
measurement is above: on this carrier the banked cell-field chart reads 3.4–9.2× better than
the lattice-gas chart at comparable arity, with the exclusion cap the identified cause. That
is not a claim that the lattice-gas machinery is the wrong object — §0 says the opposite,
that its own Lean fence predicts this and names the fractional chart as the repair. It is a
measurement of the **Boolean bridge** between a molecular scene and that machinery, and the
finding is that the bridge, not the destination, is what fails here.

**For the LG node** (the continuum-native lattice-gas tier): the seam this campaign measured
is that node's first input. A Boolean occupancy map is not the bridge — the mean-occupancy /
lattice-Boltzmann form is, and `Core/ModeChart.lean`'s CAP fence is already the Lean that
covers it. Nothing in this document composes the lattice-gas object through
`viewClosed_comp` as if it were a view of the molecular dynamics; it is not, and A2 did not
test it as one. A2 tested a MAP from the molecular scene into that object, measured what the
map costs, and reports that cost.
