# The tier ladder — rewritten 2026-09-27 from the record

*References to `TIERS.md` written before 2026-09-27 — the exact-ring tower, the arithmetic-regime laws, what quizx is for per tier, the four-physics table, §fold's text — resolve in `TIERS_ARCHIVE_2026-09-26.md`, which is kept whole.*

*The ladder as the evidence now draws it. The previous document, with every section that
earned these rows, is kept whole in `TIERS_ARCHIVE_2026-09-26.md`; nothing below contradicts a
number there, but several of its rows were drawings that this month's measurements
reclassified. Statuses: MEASURED (a record cited), IN THE ENGINE (a gate in CI), DECLARED
(an input, not derived), WAGER (a chosen position with a kill), NOT A TIER (closed, but the
tier above does not carry it).*

## 0. What a tier is, now

**A tier is a closed view of the tier below that the tier above CARRIES** (`OBJECT.md`,
CORRECTION 2026-09-25). Closed: it predicts its own next state within a budget (the square,
`Closed`). Carried: the conservation law of the tier above cannot remove it within that
budget. Both are measured, and the second is decided by the engine's REMOVABILITY GATE
(`holon-closure::removable`, ordered admission, `REPLACE1_RESULTS.md`): a candidate is carried
if removing it costs the target more than the budget, with a price certificate (held-out
increment, re-paired null, jackknife SE). The rungs of this ladder are decided by the gate,
not by the drawing; a band proposed from now on passes the gate first.

**A tier's objectivity has a measure: its WINDOW** — the range of budgets over which its
carried set does not change (`OBJECTIVE1_RESULTS.md`). The window's edges are the prices of
its most expensive dropped sector and its cheapest carried one. A wide window is a fact about
the substrate; a narrow or empty one puts the observer's budget into what the tier is.

**What is shared across tiers is the procedure, not the law.** Each tier's law is forced by
its closed view (`closure_determines_dynamics`) and is therefore different on every tier; what
repeats is search, select by the tier above, price the drop, compose — and the remainder owed
upward, counted once (the "+1").

## 1. The physical tower

| tier | carried (the object) | dropped sectors, at a measured price | the remainder owed upward | window | status |
|---|---|---|---|---|---|
| **the fold below the atom** (gauge vacuum, hadron) | colour-singlet closure on Gauss's law; the vacuum's conserved charges | — | the magic density `c(x)·N`, extensive at every coupling; a ten-site box's price `2^{M₂} ≤ 16` (`GF1_RESULTS.md`; a re-finding of PRB 111 L081102) | not measured | MEASURED on 1+1D only; SCHWINGER-3 `M_V/g = 0.553116`, `2.0 %` from the continuum; GF0 SCHWINGER-4: the residual between screened static pairs decays at the banked meson mass to 0.6% — Fold II's first measurement below the atom, 1+1D only; GF2 GATED |
| **nucleus** | `Z`, mass, spin, charge radius | — | none measured; the fold's interior is the owed item | — | DECLARED input |
| **atom → chemistry** | exact full-CI cores; the derived force law (charges, walls, contacts, charge transfer as a table) | the higher many-body terms, priced at the seam | the seam: three-body dispersion harvested to `10⁻¹²` (`EMBED2_RESULTS.md`); charge transfer, the sixth channel (CT-1…3); dE5 does NOT terminate (24/24 over bound) | not measured | MEASURED; derived, not fitted; `2.10` mHa RMS against MB-pol (COMPARE-0) — behind the fitted state of the art, by design |
| **molecule** | the rigid unit: six degrees of freedom — mass, momentum, orientation | the three vibrations: MORE closed than the object (purity `1.00`, cross-block `0.009`) and dropped, `+0.325 kT`/water at the handoff | none the fluid needs at these cadences | not measured | MEASURED on three all-atom walks (`MOLSEARCH1_RESULTS.md`); CERTIFIED-STRICT on the 12-atom scene (893.8 fs against 834, 0/111 controls) |
| **fluid element** | the conserved densities with fluxes on the FACES: density in cells, momentum on the faces within `0.25 Å` (the staggered chart) | the structural block `(q, s2)` and the count block, at `≤ 0.003`; the collective order parameter at `≤ 0` (ORDER-1); the hydrogen-bond count at exactly `0` | the leak the cell chart cannot carry at one molecular diameter — carried by the faces: `D = 0.180 ± 0.058` on the graded 36-cycle average against the cell chart's `0.31` | **`(0.0033, 0.039)`** — one decade | IN THE ENGINE (REPLACE-1: `holon-lens::staggered`, the gate reproducing every banked number); driven shear rent `η = 4.4 × 10⁻⁴` Pa s equal to the equilibrium read |
| **continuum / the cube** | the hydrodynamic fields | — | textbook; the viscosity as the driven mode's rent | — | MEASURED once (RESPONSE-1 R3); the cube's dynamics FENCED |

**Not tiers — closed sectors the tier above does not carry:**

| sector | closed? | carried? | where it lives | record |
|---|---|---|---|---|
| the hydrogen-bond network | yes: the count's own `σ = 0.46` at 100 fs, memory `~350` fs; 3.375 bonds/molecule, spanning cluster `1.0` | **no**: increment to the momentum law exactly `0` on three walks | an uncarried sector of the molecular tier, for the dynamics the fluid needs at 293 K | `TSCAN_HBOND_SEARCH_RESULTS.md`, `LIQUID2_RESULTS.md` |
| the collective order parameter (tetrahedral order at the box's longest wavelength) | yes: `σ₁ = 0.29–0.36` at 1 ps, outside the hydrodynamic span | **no**: increments `≤ 0` into density and current | an uncarried sector; its per-molecule form carries `+0.03` of single-molecule mobility, never reaching the fields | `ORDER1_RESULTS.md` §6, `SLOW1_RESULTS.md` |
| the VAMPnet's slow shell-geometry coordinate | yes, slower than q (`0.50` vs `0.33`) | a sliver, `+0.013` beyond q, under the bar | an uncarried sector | `SLOW2_RESULTS.md` |

**"Not carried" is relative to the targets tested** — the momentum law at 50 fs, the collective
density and current at the box's longest wavelength, mobility at 5 ps, on the rigid operator at
293 K. The network may be the carried sector for targets nobody has pointed the gate at: the
dielectric response, and above all FREEZING, where it is the whole story. Until a target is
named and the gate is run on it, it is not a rung.

**The reclassification this makes.** The waterbench ladder was drawn cube → fluid element →
H-bond network → molecule → atom → nucleus → the fold. It is six tiers, not seven: node G's
rung 1 was never an under-evidenced tier, it was a closed sector drawn as a level. Its
"measured, not certified" status in `GANTT.md` is closed as NOT A TIER, and every number
measured on the network stands as a reading of a sector.

## 2. The computational ladder (the QVM) — views, not levels

These are not tiers of matter; they are the DICTIONARY of views the dispatch searches, each a
closed view of the circuit it can carry exactly, with a price for the rest.

| view | closed on | the +1 (what it cannot carry) and its price | status |
|---|---|---|---|
| classical bit-planes | permutation / diagonal circuits | everything else, refused by name | BANKED |
| stabilizer tableau | Clifford motions (`tableau_closed_under_hadamard`) | the T-gates in the light cone, `expected_branches(t_eff)` — exact, printed before the run | BANKED; parity with stim on one core at `d = 221`; the column-sharded cut correct and not paying (branch (b)); the transpose re-aim `0.41 ×` on a load-refused table |
| stabilizer-rank sum | Clifford+T via Magic5FromCat (credited) | a certified remainder `R_k ≤ ε`; 450 of 450 runs | IN THE ENGINE (ACUITY-1 (a)); on the GPU bit-identical at `0.022–0.025 ×` the CPU mesh (GPUFOLD-1 (a)) |
| certified MPS | low-entanglement circuits | the discarded weight `Σ√w`, a true 2-norm bound | BUILT for ACUITY-2 |
| dense | `n ≤ 24` | none | referee |
| **the dispatch** | picks the view unlabelled | 77 of 80 within `2 ×` of the hand-picked method, certificate 80/80; at `n ≤ 20` choosing costs more than running | MEASURED (ACUITY-2; two stakes void by prereg error) |
| the stabilizer rank of `|H⟩^⊗7` | — | open cell `6 ≤ χ ≤ 9`: no six-term witness in 30.4 h; no 3+3 or 2+4 split; a second class of six-copy decompositions found | RANK-1 (b) |

## 3. The reasoning carrier — the one tier whose carried sector is not a conservation law

| tier | carried | window | status |
|---|---|---|---|
| **reasoning chains** (444 parent→child transitions) | depth, tokens, the action; **the conscience sector** — closed `8.9 ×` above a marginal-preserving null, surviving ordered admission at `+0.016` with a clean null | **narrow**: its price is one SE from its null, and the carried set changes inside `β ∈ [0.005, 0.02]` — level-RELATIVE | MEASURED (`REASON_SEARCH0_RESULTS.md` under Amendment 1; REPLACE-1; OBJECTIVE-1 Q3) |
| **the 11 + 1 on its native carrier** (269 of this programme's own corrections, two blind coders) | every change takes one of the eleven kinds for both coders; the partition shared at κ = `0.52` (`0.88` with runner-ups); **the Record axis sharp**: κ = `0.72`, one-way, zero reversals | — | MEASURED (`RECORD1_RESULTS.md`, branch (b)); the transition map's succession rules ABSENT |

The eleven kinds are coordinates of change in a RECORD, not of matter; no water trajectory
can distinguish a Fact from a Process, and none was ever used to. The one part of the 11 + 1
that runs through the physical tower is the +1 — the remainder the local pieces do not
generate, counted once: the leak through the faces, the T-gates in the light cone, the seam,
the magic density.

## 4. The cosmological rung — a wager, and the final remainder unmeasured

Move 5 of the steelman: precedent carried as classical bits (the dark-matter ROLE), receipts
and append-only ledgers as the Record (dark energy's). Two legs dead and kept (Landauer
normalisation at 3–5 dex; the flow/maintenance rescue); the shape survives (DESI DR2
`Δχ² = −2.13` against ΛCDM), DESI DR3 the standing kill. **The tower's measured remainders stop
at hydrodynamics.** The final remainder at the cosmological rung would be the Record itself —
what no closed view carries — and no instrument on this record measures it. The full text is
`TIERS_ARCHIVE_2026-09-26.md` §"The top rung".

## 5. What the ladder licenses, and what it does not

- **Licensed:** six physical tiers, each with its carried shape measured, the fluid element's
  in the engine with its window; three closed-but-uncarried sectors named with their prices;
  a computational dictionary with certificates; one non-conservation carried sector
  (reasoning), level-relative.
- **Not licensed:** that any tier's LAW is shared with another (each is forced by its own view);
  that the agency-bearing ladder bears physical load (it runs the same procedure and shares the
  +1, and nothing in the physical tower rests on it); anything about real water (the carrier
  is a derived model, sluggish, `D` a quarter of water's); any cosmological remainder.
- **The instrument, stated once:** a TICA/VAMP search (credited) inside a stricter discipline —
  plants, stakes frozen before the run, the closed-versus-carried split measured, ordered
  removability with a price. After five carriers and one head-to-head it re-finds and has not
  discovered; what it has produced of its own is the gate, the window, and the reclassification
  above.

## 6. Owed, by tier

| tier | owed | gate that closes it |
|---|---|---|
| fold below the atom | GF2, the hadron box | its own freeze |
| molecule | orientation on a walk, for the dipole sector (ORDER-1's refused arm) | the gate on the dielectric target |
| fluid element | whose numbers these are: the fine-model seeds (Amendment 6), ~2026-10-07 | a fine number inside the rigid three-seed range |
| the network | a target on which it might be carried: freezing | the gate on a nucleation target |
| reasoning | text with parent links, for the eleven as a dictionary | the gate with kinds as columns |
| the eleven | the coding convention (the rule adopted vs the content failed) stated in the definition | a re-coded agreement above κ = 0.7 |
| the window | measured on a second tier | the sweep of OBJECTIVE-1 on the molecule |

---

*The archived document (`TIERS_ARCHIVE_2026-09-26.md`) keeps: the four-physics status of
2026-09-03, the 2026-09-01 consolidated table, the ladder's assumptions and the top rung in
full, the honest boundary and its challenges, the Ossicle TODO, the tuning and grain modules,
the exact-ring tower, what quizx is for, and the requirement adjudications of 2026-08-28.*
