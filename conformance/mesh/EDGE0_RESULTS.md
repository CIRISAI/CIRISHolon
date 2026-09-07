# EDGE-0 — results: the cluster carrier has two densities and no edge

*Freeze `EDGE0_PREREG.md` (151fc79, alone, before any counted run). Carrier
`holon-lattice/src/edge.rs` (the rest slot from the seven-slot census, two donor arms, the wait
at rest, the mover per connected component, gravity posted); instruments
`holon-lattice/src/surface.rs` (six); the shared union–find `holon-closure`'s `ComponentFinder`.
Runner `examples/edge0_lattice.rs` on `holon-campaign` (`screen`, `screen_chart`, `gate`, `run`,
`read`). Records under `conformance/mesh/edge0/`: `screen.json`, `screen_chart.json` (both
`dry: true`), `gate.json`, `price.json`, `run.json`, `run.log`, `run.done`, `read.json`; every
file validates as JSON. Cores 24–31; `run` 62.0 s of wall on four workers against a 296.7 s
single-core projection, every scene inside the price's band.*

## The verdict, first

**The cluster carrier has TWO DENSITIES and NO EDGE.** Coexistence is real and it is the rule 2b
carrier's alone: the block-density variance is 12.34 times the one-phase control's where the pair
arm at the same chart reads 1.86, every seed's histogram carries two separated modes where neither
control carries any, and the spread clears the control's by 8.6 times its band. **C BRANCH (a).**

But the dense phase is DISPERSED, not contiguous, and the three interface branches read on scenes
that did not hold. A slab built at the two measured phase densities loses its contrast in 100
steps (6.10 → 1.19) and a droplet inverts (6.45 → 0.45); 197 of 200 sampled frames had no
interface to find a height on. **L BRANCH (b)** — no Laplace line, slope negative and unresolved,
per-seed `R²` from 0.002 to 0.738. **W BRANCH (b)** by the letter, and unresolved in fact. **A
BRANCH (a) IS VACUOUS AND IS REPORTED AS SUCH**: the band is 3.886 because `σ_C`'s own spread
(1.943) exceeds its mean (0.876), so any `σ_L` would have agreed. **D1 BRANCH (b)**, the number
the campaign takes away: the cluster arm's bonded droplet starts at 107 particles, two of four
seeds keep half of it to the cap of 6,000 steps and two lose half at 625 steps, against the pair
arm's start of 5.75. **G1B BRANCH (b)**: gravity posted 3,694 units of momentum into the ledger
exactly, and the dense phase did not drift against its own spread. The four instrument gates PASS
and both plants FIRE.

**What EDGE-1 inherits, in one sentence:** rule 2b is a cohesion and the tier now has two
densities, but the object it makes is a bonded network of small clusters — largest component 4.6 %
of the particles — and not a liquid drop, so the edge is not there to be measured and the next
campaign has to make the dense phase contiguous before it asks for a surface tension.

| gate / branch | verdict | the number |
|---|---|---|
| G0 — the ledger, exact up to the posted injection | **PASS** (5,000 step-checks, 16 legs) | mass, `Px − Σ posted`, `Py − Σ posted`, red, the orientation total AND its per-orientation census, and `formed − broken_rent − blocked = held`, integer-identical at every step of 5 configurations × 2 seeds × 500 steps. Totals: 210,951 formed, **116,430 released BY THE RENT and 89,152 by the mover** (ratio 1.306), 170,156 components walked and **163,811 moved as one (96.3 %)**, 6,345 refused the drawn move, 1,069 refused the wait as well, largest component 580, 3,849 waits tried and 2,780 taken, 9,244 rest particles created, gravity 214 flips + 475 from rest |
| G1 — the pair, no-bond, no-gravity control IS FLUID-1 | **PASS** (300 steps, 8 legs) | occupation, orientation plane, colour plane and every bond's pair of slots bit-identical at `L = 32` over 300 steps of a BONDED run on FLUID-1's own seed; `formed / broken / blocked` = 127,640 / 81,487 / 45,771 on both; the wait's count exactly 0 |
| G2 — six instruments on known scenes | **PASS** (19 checks, 17 legs) | the planted two-mode dip exactly 1, one mode UNSEPARATED, a uniform spread exactly 0; one particle's flux `c ⊗ c` to `1e-9`; the free gas `c_s² = 0.5` to `0.02`; a planted Laplace line's slope to `1e-9` and a flat jump fitting none; a planted capillary `σ` to `1e-8`; a planted `tanh`'s width to `0.05` cells; an empty lattice's edge EMPTY with `NaN` width, `b = L` vacuous, a droplet in vacuum leaving the outside closed |
| plant (i) — the rent doubled | **FIRES** | measured `0.09644` against a stake of `0.04878`, itself half the exact analytic reach `0.09756 = \|f(E) − f(2E)\|/f(E)`; carrier `0.900` over a floor of `0.01` in the break-probability sector |
| plant (ii) — gravity's rate doubled | **FIRES** | measured `0.80952` against a stake of `0.5`, half a reach of exactly `1.0`; carrier `0.02` over a floor of `0.001` in the per-cell draw |
| P — the price | **PASS** | `1.224597e-3` s per step at `L = 128` (`9.202e-8` per cell per step, `0.729×` FLUID-1's measured `1.262e-7`), 242,274 steps projected at `296.7` s of single core; the run took `62.0` s of wall on four workers, ratio `0.209` inside the `0.1`–`10` band. Component finding measured apart: `2.675×` the pair step on this single-shot sample, `1.424×` (range `1.179`–`1.515`) on seven interleaved repeats, which is the stable figure |
| **C — coexistence** | **BRANCH (a)**, 3 legs, 6 reported | separated 4 of 4 seeds against 0 of 4 on both controls; variance ratio `12.337` against `3 × 1.860 = 5.581`; spread `0.06920` against the one-phase `0.02232`, a gap of `0.04688` against a band of `0.005468` (3σ of the two arms' combined seed spread) — `8.6×` |
| **D1 — the droplet's lifetime** | **BRANCH (b)**, 1 failing leg | cluster arm: start 107.0 particles, 2 of 4 survived the cap of 6,000, the other two lost half at **625 steps**. Pair arm: start 5.75, 4 of 4 "survived", no death — VACUOUS at that start, §3 |
| **L — Laplace's law** | **BRANCH (b)**, 2 failing legs | `σ_L = −0.012394 ± 0.012995` over four seeds; per-seed `R²` `0.510 / 0.003 / 0.738 / 0.002`; the slope is negative and `\|slope\| < 2·sd` |
| **A — the two σ agree** | **BRANCH (a) — VACUOUS** | `σ_L = −0.012394`, `σ_C = 0.875633 ± 1.943132`; gap `0.888` against a band of `3.886`. The band is wider than either reading, so the branch could not have read (b) — §3 |
| **W — the interface width** | **BRANCH (b)**, 1 failing leg | `dw/dt = −2.481 ± 5.721` cells per thousand steps against a bar of `0.05`; per-seed widths `0.235 / 0.519 / 0.633 / 14.330`. The drift is not resolved against its own spread, which is the leg that PASSED |
| **G1B — gravity settles the dense phase** | **BRANCH (b)**, 2 failing legs | posted `3,693.75` units of momentum (the ledger exact up to it at every step); slope on `−1.312e-4 ± 5.937e-4`, off `+4.029e-4 ± 5.063e-4` — neither resolved |
| the edge at `b ∈ {4, 8, 16}` | **read, not branched** | fractions `0.0486 / 0.1667 / 0.2222` of the chart's cells, widths `11.34 / 10.42 / 8.00`, none EMPTY, none vacuous, 36 / 36 / 39 probes run before every splitting block had its witness |
| `kT_mech`, measured | **read** | `0.4990908` against the hexagon's `c_s² = 1/2` — `0.18 %`, and it is what `σ_C` is scaled by |

## 1. Coexistence, and what kind it is

At the chart (`d = 0.02`, `f = 0.90`, `E₀/kT_rent = 2.19722`, the cluster rule) the three arms read:

| arm | block-density variance | ratio to the one-phase control | two modes | spread `q95 − q05` | the two phases (dense share) | bonds/particle | largest component |
|---|---|---|---|---|---|---|---|
| cluster | `5.405e-4` | **`12.337`** | 4 of 4 seeds | `0.06920 ± 0.00182` | `0.01038 / 0.05905` (`0.198`) | `1.146` | `0.0458` |
| pair (control) | `8.150e-5` | `1.860` | 0 of 4 | `0.02958 ± 0.00112` | `0.01392 / 0.02865` (`0.414`) | `0.510` | `0.0034` |
| one-phase (control) | `4.381e-5` | — | 0 of 4 | `0.02232 ± 0.00000` | `0.01493 / 0.02566` (`0.477`) | `0` | `0.0004` |

The two controls read exactly what the two-phase decomposition's own unit test says a single phase
looks like: their two "densities" sit together and their dense share is near a half. The cluster
arm's are a factor of `5.7` apart with the dense phase holding a fifth of the box. That is
coexistence and it belongs to rule 2b.

**But it is a dispersed coexistence.** The dense phase covers 19.8 % of the blocks while the
LARGEST connected bonded component holds only 4.6 % of the particles: many small clusters, not one
liquid. The three interface branches are about a macroscopic surface, and there is not one.

## 2. Why L, A and W read on scenes that did not hold

A post-hoc diagnostic (a probe, never a reading; the numbers below are not in any branch) measured
the two built scenes through the warm-up:

| | `t = 0` | `t = 100` | `t = 300` | `t = 1000` | `t = 3000` |
|---|---|---|---|---|---|
| slab: interior / outside density | `0.0593 / 0.0097` | `0.0381 / 0.0320` | `0.0357 / 0.0349` | `0.0403 / 0.0333` | `0.0342 / 0.0321` |
| slab contrast | `6.10` | `1.19` | `1.02` | `1.21` | `1.07` |
| droplet contrast (inside / outside) | `6.45` | — | `0.45` | `0.78` | `0.65` |

The scenes are BUILT correctly — the unit test that says so passes, and `t = 0` shows a contrast of
six — and they evaporate inside 100 steps, long before the 3,000-step warm-up ends. The droplet
does worse than evaporate: its inside becomes SPARSER than its outside. The mechanism follows from
what the screen already measured about rule 2b: a component moves only if every target is vacant
or vacated by itself, so a hand-built blob at the dense phase's own density is one large component
in a crowded neighbourhood, is refused both candidates, and has its bonds released. The carrier's
own dense phase is made of clusters small enough to move; a blob is not.

Everything downstream follows. `frames_with_holes` is 196–197 of 200 on every seed, so the
capillary spectrum was fitted on three frames of a scene with no interface, which is why `σ_C` is
`0.876 ± 1.943`. The width fit ran on two plateaus that differ by `0.002` — its own residual is
`0.010`, five times the thing it is fitting — and one seed's width ran away to `14.3` cells. The
droplet pressure jumps are `−8.5e-4` to `+1.4e-4` with no sign pattern and no line.

**These are readings about the scenes, not about a surface tension of this carrier.** No σ is
claimed. The freeze's L (b) and W (b) are entered as they read; A's (a) is entered and marked
vacuous.

## 3. Corrections to the freeze's letter

- **A has no resolution leg and no dependence on L, and both gaps fired at once.** L read (b) — "no
  surface tension is read from it" — and A then compared that unread slope with `σ_C` and reported
  agreement. The band, `2·sqrt(sd_L² + sd_C²) = 3.886`, is wider than either reading, so the branch
  could not have read (b) whatever the numbers. A must be VOID under L (b), and it needs a leg
  requiring each σ to be resolved against its own spread before "agree" means anything. The
  verdict stands as the letter gives it and is marked vacuous here.
- **The chart's confirmation pass consumed the counted run's own seeds.** `screen_chart` and the
  counted coexistence run use the same four seeds, the same box, the same warm-up and the same
  window, so C's three legs reproduce the confirmation pass BIT-IDENTICALLY (`12.337`, `0.06920`,
  4 of 4 on both). C (a) is therefore not independent of the pass that chose the chart. The
  substance is unaffected — the pair and one-phase controls are measured in the same run and are
  what C is read against — but a successor must confirm a chart on seeds the counted run will not
  use.
- **The droplet-survival criterion is vacuous at a tiny start.** The pair arm "survived" 4 of 4
  with a starting largest component of `5.75` particles; half of that is `2.9`, and a component of
  three exists almost always. The comparable numbers are the STARTS, `107.0` against `5.75`. D1
  needs a floor on the start before survival is a reading.
- **W's letters have no branch for "not resolved".** The width read `−2.481 ± 5.721` cells per
  thousand steps: it fails the stationarity bar and it is not resolved from zero either, so it is
  neither (a) "stationary" nor (b) "grows without bound". Recorded as (b) by the letter with this
  reading beside it — FLUID-1's W had the same shape and the same correction is owed.
- **The price's projection is single-core and the run is not.** `run` used four workers over
  independent scenes, so the `0.209` ratio is about a quarter of what one core would give. The band
  absorbed it; a successor should project the parallel arm or check the ratio per scene.
- **The single-shot step price is unstable under load.** The same measurement returned `5.03e-8`,
  `9.20e-8` per cell per step on different invocations and a cluster-over-pair ratio of `1.35×` to
  `2.68×`, because other lanes share cores 24–31. The stable figure for component finding is the
  interleaved one, `1.424×` with a range of `1.179`–`1.515` over seven repeats.

## 4. Bookkeeping, declared

- The carrier's four rules and their reasons for conserving are in the module header and nowhere
  else. Rule 2 (the pair mover) is untouched and is the freeze's pre-committed control arm; the
  five `pub` visibility changes in `orientation.rs` change no rule, and G1's bit-for-bit identity
  over 300 steps of a bonded run is what says so.
- `blocked_no_vacancy` and `blocked_claimed` are ZERO in every rule-2b total, not because nothing
  was released but because those two prongs are rule 2's sentences and this rule cannot say them;
  the cluster release's own prongs are `component_move_refused` (6,345) and `component_wait_refused`
  (1,069).
- The `read` phase needed one repair before it would run: a lifetime written as `null` — no seed of
  that arm died — was refused as "not a number". It is an ABSENCE by design and is now read as
  `NaN` with the survival count beside it, for those two fields and no others.
- `q05` of the cluster arm is EXACTLY zero (its sparse blocks are empty), which is why C's third
  leg is a SPREAD and not a ratio; and the one-phase control's own seed spread is exactly zero,
  which is why the band is the two arms' combined spread and why a band of zero VOIDS the branch
  rather than passing it.
- The only numbers from outside this engine are the amplitude table and the retention READ from
  CT-1's and CT-2's records (`0.5526` from `ct2/arms.json:dimer_293_seam.f`) and FLUID-1's own
  measured cost per step from `mesh/fluid1/price.json`, each printed with its path and its field.
  No experimental kill appears in this campaign.
- Every one of the 60 screen rows, every counted run and every gate configuration reported its
  ledger exact; `run` checked 4 seeds × 3 arms × 1,000 audited steps of coexistence alone.
