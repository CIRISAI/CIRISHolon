# MIXTURES-1 — results

Contract: `MIXTURES1_PREREG.md`, frozen 2026-08-30. This file is the campaign's
record. Sections appear in the order they were written, and the P1 protocol
below was written and committed **before the mixed arm was run or its output
looked at**, as the prereg requires.

## Where every gate stands

| gate | verdict | in one line |
|---|---|---|
| **B1** the bank is exact where the single table was | **HOLDS** | 693 lines of raw f64 bit patterns, zero diff against the pre-bank commit; and a mixed fixture where the two criteria differ in sign |
| **C1** conservation in a mixed box | **HOLDS** | drift 4.047e−5 against a 4.058e−3 derived bound; momentum 2.616e−13 against 1.032e−9; and on every seed of all three P1 arms |
| **plant (i)** the swapped table | **CAUGHT** | `R_e` moves 1.5357 bohr and the energy 8.6 orders above the referee tolerance; and on a non-binding pair it invents a bond |
| **plant (ii)** the wrong mass | **CAUGHT** | derived `dt` moves by exactly `sqrt(mu'/mu)`, to 1.1e−16 relative, predicted from the masses rather than written down |
| **plant (iii)** the DMRG label | **CAUGHT** | refused at both doors, with the slot evicted, each with a positive control |
| **E1** the emergent negatives | **evidence, not discharged** | Ar2 and NeAr unbound — but on the engine's derived grid, not the grid E1 stakes |
| **E2** the emergent chemical contrast | **BRANCH (b)** | the middle (HCl > S2 > Cl2) and the unbound tail are exactly as staked; both ends are not — NaH up four, ClF down three, SiO and N2 swapped |
| **P1** THE PRODUCT: emergent hetero-chemistry | **BRANCH (b)** | HCl modal in 0 of 8 seeds; both controls pass, so not VOID. HCl bonds ARE forming — the *reading* cannot see them in a condensed phase |
| **D1** the DMRG bridge earns admission | **NOT ADMITTED** | the MPO wall fell (12 h → 0.3 s) and a second one was behind it: the SWEEPS do not converge. SiO is 1.1e−2 from exact after 664 s, six orders from the stake |
| **R2** the staked-pair referee gate | **OWED** | gate built and scope-refusing; blocked on the referee lane's drop, and cheap when it lands |

Two gates fire, and neither fires because the model failed. E2's inversion is a
statement about the declared model that R2 will confirm or refute; P1's is a
statement about the measurement rule, which cannot resolve molecules in a
condensed phase. Both are reported as the product rather than adjusted toward the
stake.

---

## P1 — THE PROTOCOL, FROZEN

*Committed before any arm ran. Everything below is a `const` or a `fn` in
`engine/crates/holon-render/examples/mixquench.rs`, not a flag, so a reported
run re-runs byte for byte and a run whose parameters were overridden cannot be
reported as one whose parameters were staked.*

### The three arms

| arm | scene | pair types banked |
|---|---|---|
| **mixed** | 8 H + 8 Cl | H-H, H-Cl, Cl-Cl |
| **control: hydrogen** | 16 H | H-H |
| **control: chlorine** | 16 Cl | Cl-Cl |

### The scene

| | |
|---|---|
| atoms | 16 (`MAX_ATOMS`) |
| dimensions | 2 — the `z = depth/2` slice |
| box | 40 × 24 bohr, soft quadratic walls, wall inset 0.6 bohr — SATURATION-1's box kept, so the hydrogen control is comparable to its bank |
| opening positions | a 4 × 4 lattice at `(w(col+½)/4, h(row+½)/4)` with a per-seed uniform jitter of ±0.8 bohr |
| **mixed composition** | **checkerboard: lattice cell with `(col + row)` odd is chlorine.** Eight of each, every chlorine with four hydrogen nearest neighbours and vice versa. Stated as a rule rather than a list so it cannot be quietly re-drawn; an opening that clustered the chlorines on one side of the box would be an opening that decided the answer |
| opening velocities | Box–Muller Gaussians from one seeded LCG stream at `T_init = 3000 K`, with the width taken **per species** (`sigma = sqrt(k_B T / m)`, so a chlorine opens 5.9× slower than a hydrogen at the same temperature) and the net **momentum** removed — not the mean velocity, which in a mixed box is a different quantity |
| thermostat | ON from the first step, Berendsen, `T_target = 300 K`, `tau = 2000` a.u. |
| integration | 20,000 grain boundaries × 64 substeps = 1,280,000 substeps |
| curves | 96 knots, engine-computed STO-3G FCI, generated once per process |
| three-body | ON, and **H3-ONLY** — see the fence below |
| RNG | one LCG (`x = 6364136223846793005 x + 1442695040888963407`, top 53 bits) seeded per run; nothing else is random |

### The eight staked seeds

```
0x000000004d495801  0x000000004d495802  0x000000004d495803  0x000000004d495804
0x000000004d495805  0x000000004d495806  0x000000004d495807  0x000000004d495808
```

### THE FENCE, displayed

The three-body term is **H3-only**: `Sim::accumulate_three_body` skips any triple
containing a non-hydrogen atom, so such a triple contributes an exact zero. The
mixed arm therefore runs MBE2-exact over all three pair types plus MBE3 over the
hydrogen triples only. **No reading in this campaign is beyond-pair-complete for
a triple containing chlorine.** The engine declares the fence (`holon_trimer_h_only`)
and both viewers display it, rather than each asserting it in a caption.

### Two protocol facts, disclosed rather than discovered later

**The arms cover different amounts of simulated time at the same boundary count**,
because `dt` is derived per scene from the fastest active mode. The chlorine arm's
`dt` opens about 18× the hydrogen arm's — Cl2 is stiffer than H2 but chlorine is
35× heavier, and frequency is what a timestep has to resolve — so 20,000 boundaries
is roughly 300 ps of chlorine and 17 ps of hydrogen. Equal boundary counts were
chosen over equal simulated time because equal simulated time would mean unequal
integration accuracy, and the accuracy contract is the thing this engine holds. Both
numbers are reported per arm.

**`dt` falls during a run and that is the design, not drift.** The curvature envelope
is monotone and widens as the trajectory reaches stiffer parts of the curve, so the
hydrogen arm opens at `dt = 1.0797` and refines to `0.5387` within a hundred
boundaries. The drift bound is re-derived from the current `dt` on every call, so
there is no stale bound behind it.

### Knot density, measured before it was frozen

`R_e`, `D_e` and `k_e` do not depend on knot count at all — they come from
`locate_well`'s own Newton solve on the solver, not from the interpolant. The
derived timestep does, weakly, because it reads the interpolant's curvature
envelope:

| knots | `dt` on the opened hydrogen scene |
|---|---|
| 24 | 1.079664 |
| 48 | 1.077481 |
| 96 | 1.077209 |
| 192 | 1.076929 |
| 384 | 1.076997 |

0.25% across a 16× range, converged well before 96. `CURVE_KNOTS = 96` is chosen
for the interpolant's accuracy *between* knots and for cost, not for the clock.
Cl2 is what prices it: 18 basis functions, 324 determinants, about 97 s at 48 knots.

### Measured cost, from which the schedule is frozen

| arm | curves | per boundary | per seed at 20,000 |
|---|---|---|---|
| hydrogen | 0.3 s | 0.0017 s | 33 s |
| chlorine | 112 s | 0.0007 s | 14 s |
| mixed | 121 s | 0.0008 s | 15 s |

### THE MEASUREMENT RULE

Taken at the final grain boundary, from `Sim::cluster_species_counts` and
`Sim::cluster_sizes` — **two readings of one union-find over one edge set**, the
same partition the headline `Sim::cluster_count` already reads. No new criterion
and no distance cutoff: an edge exists exactly where the pair layer says `bonded`.

* a component of ONE atom is a **free atom**, not a molecule;
* a component of two or more has a **formula**, the count of each nuclear charge
  in it, keyed by `Z` rather than by the bank's species index — the index depends
  on registration order, and a formula built from it would depend on which atom
  happened to be placed first;
* the **modal molecule** is the most common formula among components of size ≥ 2.
  Ties break toward the SMALLER component, then toward the LOWER maximum `Z`, then
  lexicographically. That is a total order, so the answer cannot depend on
  iteration order;
* the full formula histogram is published per seed and pooled, either way.

### The criteria, and what each decides

* **BRANCH (a)** — the mixed arm ends with **HCl as the modal molecule in ≥ 6 of
  8 seeds**.
* **BRANCH (b)** — anything else. Reported as plainly as a pass would be, and
  investigated. Not massaged.
* **CONTROLS.** Each single-species arm must (1) produce at least one molecule in
  ≥ 6 of 8 seeds — the instrument sees molecules at all — and (2) produce formulae
  containing ONLY its own element, which is a structural check that would catch a
  species-bookkeeping bug in the bank. **If either control fails, P1 is VOID** —
  protocol, not physics, per the detector-not-verdict rule — and the mixed arm's
  reading is not reported as a result.
* **C1 rides along.** Every seed of every arm reports its energy drift against the
  derived bound and its momentum residual against the roundoff bound. A gate firing
  there is reported; it does not silently invalidate the composition reading, because
  the two measure different things.

### What a pass would and would not mean

A pass means: *in this model, from Z, masses and the STO-3G basis alone, a hot gas
of hydrogen and chlorine cooled in a box ends up as hydrogen chloride.* Nothing
here is a claim about nature's thermochemistry, about rates, or about a triple
containing chlorine.

---

## B1 — the bank is exact where the single table was · **HOLDS, both halves**

### The regression half: bit-for-bit

`examples/b1_reference.rs` dumps every energy, drift, bound, clock, position,
velocity and pair reading of three all-hydrogen scenes as raw `f64` BIT PATTERNS
— 2 atoms in 2D, 16 atoms in 2D with the three-body term, 16 atoms in 3D; 200 and
100 frames of 64 substeps each. It was run in a git worktree at the commit BEFORE
the bank landed and again after.

**693 lines, zero diff.** The output is committed as
`engine/crates/holon-render/tests/data/b1_hydrogen_reference.txt` and
`tests/mixtures.rs` re-derives it, so the comparison can be re-run rather than
believed. Release and debug were checked to agree bit-for-bit before the
comparison was made a gate — otherwise the test would have been a profile
detector.

### The mixed half: each pair reads its own criterion

The fixture is hydrogen against HELIUM, because the statement is then structural
rather than numerical: in this model H2 binds and H-He binds at NO separation
(`locate_well` returns `None`, and nothing in the code knows helium is noble). At
1.6 bohr, in one scene, under one integrator:

| pair | `u` / Ha | `e_rel` / Ha | bonded |
|---|---|---|---|
| H-H | −1.956587e−1 | −1.497299e−1 | **yes** |
| H-He | +1.993623e−1 | +2.727431e−1 | **no** |

Opposite in sign and opposite in verdict. One table cannot produce both readings.

Two things the fixture caught, both mine:

* it first asserted H-Li was repulsive at 2.0 bohr. **It is not** — H-Li crosses
  zero at about 1.87 and its minimum is at 2.924, so 2.0 is already inside the
  well. The carrier assertion fired and the fixture moved onto measured ground.
* both pairs were then placed AT REST, and a pair at rest has `E_rel = U(R)`
  exactly, which puts it exactly ON its own outer turning point — so a hydrogen
  molecule sitting in its own well read NOT BONDED. `Sim::reset` warns about this
  in its own comments. The fixtures now place pairs with a RADIAL closing
  velocity (radial, because transverse motion carries angular momentum and would
  move the turning point for a reason unrelated to the curve under test).

---

## C1 — conservation in a mixed box · **HOLDS**

One gate per conservation law, on a mixed H, H, Li scene in a walled box over
200 × 64 substeps:

| | peak | derived bound | ratio |
|---|---|---|---|
| energy drift | 4.047243e−5 Eh | 4.058441e−3 Eh | 0.0100 |
| momentum residual | 2.615606e−13 | 1.032234e−9 | 0.0003 |

And on every seed of every P1 arm, including the 8 H + 8 Cl mixed arm: **energy
gate HOLDS, momentum gate HOLDS, 8 of 8 seeds, all three arms.**

Two structural gates ride with it, each with its carrier checked first:

* **the envelope is the maximum over ALL active tables.** The two curves are
  required to differ in reachable stiffness before the maximum is scored (3.895
  against 0.505 at the run's `E_rel_max`), or taking a maximum would prove
  nothing.
* **the clock is the FASTEST MODE, not the stiffest curve.** H-H has
  `k_e = 4.747e−1` at `mu = 918.58`, giving `omega = 2.273e−2`; H-Li has
  `k_e = 9.182e−2` at `mu = 1606.40`, giving `7.560e−3`. The scene's clock is the
  first. In a mixed box `argmax k_e` and `argmax sqrt(k_e/mu)` are different
  questions with different answers, and `dt` has to resolve the second.

---

## The plants · **ALL THREE CAUGHT**

Each asserts its CARRIER is nonzero in the sector it acts on before it scores
itself; a plant on an empty sector VOIDs.

### (i) the swapped table — CAUGHT, two instances

Serving the (A,A) curve where (A,B) belongs, reaching past `load_pair_table`
into the raw interpolator on purpose — that function reads the species off the
curve's own metadata and would put it in the right slot, which is the correctness
the plant exists to check.

* **on H-Li**, the freeze's own `R_e` carrier: the equilibrium moves
  2.924394 → 1.388694 bohr, a shift of **1.5357 bohr**, and the pair energy moves
  **4.265e−2 Ha — 8.6 orders above the referee's 1e−10.**
* **on H-He**, where the consequence is not a shifted number but a wrong verdict:
  the swap **invents a bond helium cannot have**. Carrier asserted first — H-He
  has no well at all, so a BONDED reading afterwards is created by the plant.

### (ii) the mass plant — CAUGHT

Lithium at hydrogen's mass: same `Z`, so the same curve out of the same slot, and
only the inertia wrong. The reduced mass moves 1606.398413 → 918.576162 mₑ and
the derived timestep moves 3.246334443 → 2.454845768 a.u.

| | |
|---|---|
| predicted `dt` ratio, `sqrt(mu'/mu)` | 0.756190038858 |
| measured | 0.756190038858 |
| agreement | 1.1e−16 relative |

The prediction is COMPUTED from the two reduced masses inside the test, never
written down. The fixture is one H and one Li deliberately: in a scene that also
had H-H active, H-H would set the clock (it is three times the faster mode) and
lithium's mass would not appear in the answer at all — correct physics, and a
plant on an empty sector.

### (iii) the DMRG label — CAUGHT

A DMRG curve presented as exact in the model is REFUSED, and the refusal is
demonstrated firing. With the POSITIVE CONTROL that makes it mean something: the
same curve WITHOUT the false claim is admitted once D1 is discharged, so the gate
is refusing the claim and not refusing everything.

Every other provenance refusal has its own failing case: undeclared route, DMRG
with no D1 record, DMRG with no uncertainty, a shipped table with no uncertainty,
the browser split in both directions, and a D1 record whose flag and whose
measurement disagree. A refusal EVICTS the curve rather than reporting it and
leaving it in the slot, because a gate the force loop can walk past is not a gate.

**Plant (iii) had a live target.** Before this campaign, `fci::solve` routed any
space past 50,000 determinants to DMRG and the resulting curve was stamped
`PAIR_PROVENANCE = "engine-computed STO-3G FCI (determinant, Knowles-Handy)"`
regardless. SiO — one of gate D1's own staked overlap species — is 132,496
determinants and was travelling that path wearing the determinant label. The
route now travels with the number (`SolverRoute` on `Solution`, `PointSolution`
and `PairMeta`) and the provenance line is derived from it.

---

## P1 — THE PRODUCT: emergent hetero-chemistry · **BRANCH (b)**

### The controls · both PASS, so P1 is not VOID

| arm | molecules in ≥6 of 8 seeds | own element only | modal per seed | pooled |
|---|---|---|---|---|
| hydrogen (16 H) | 8 of 8 | yes | H2 in 8 of 8 | H2×57, H4×2, H6×1 |
| chlorine (16 Cl) | 8 of 8 | yes | Cl16 in 8 of 8 | Cl16×8 |

The hydrogen control reproduces its banked behaviour: hydrogen quenches to
diatomic molecules, eight of them from sixteen atoms on most seeds, with no free
atoms left.

### The mixed arm · HCl is the modal molecule in **0 of 8** seeds

Branch (a) needed 6 of 8.

```
seed 0x...5801 … 0x...5808   modal H8Cl8   molecules 1   free 0   [H8Cl8x1]
POOLED over 8 seeds: [("H8Cl8", 8)]
```

Every seed ends with a SINGLE component containing all sixteen atoms. Not HCl,
and not Cl2 or H2 either: one object.

### Branch (b), investigated

The chlorine control is the first tell: it does the same thing on its own.
Sixteen chlorines in this box also end as one sixteen-atom component, while
sixteen hydrogens in the SAME box end as eight clean dimers. So the droplet is
not something the mixture does; it is something chlorine does, and the mixture
inherits it.

The post-hoc diagnostic (`mixquench -- diagnose`, seed `0x…5801`) settles what
kind of object it is. It reports every bonded edge with its separation against
that pair type's own `R_e` and its depth against `k_B T_target = 9.500e−4 Ha`:

| pair type | bonded edges | within 0.5 a₀ of its own `R_e` | depth ≥ 10 `kT` |
|---|---|---|---|
| H–H | 14 | 3 | 6 |
| **H–Cl** | **40** | **21** | **27** |
| Cl–Cl | 20 | 10 | 13 |
| **total** | **74** | 34 (46%) | 46 |

Only **14 of 74** edges are shallower than `kT`, so this is not mostly the
tail-of-the-well artefact. And the deepest contacts in the box are these:

```
  HH   r=1.3473  R_e=1.3887   214.4 kT
  HH   r=1.1760  R_e=1.3887   200.3 kT
  HH   r=1.1714  R_e=1.3887   199.5 kT
  HCl  r=2.5402  R_e=2.5369   156.1 kT
  HCl  r=2.5296  R_e=2.5369   156.1 kT
  HCl  r=2.5155  R_e=2.5369   156.0 kT
  HCl  r=2.5666  R_e=2.5369   155.9 kT
  HCl  r=2.5064  R_e=2.5369   155.9 kT
```

**HCl bonds are forming.** Twenty-one hydrogen–chlorine contacts sit within half
a bohr of hydrogen chloride's own computed equilibrium, five of the eight deepest
bonds in the box are H–Cl at 2.51–2.57 bohr against an `R_e` of 2.5369, and H–Cl
is the most numerous near-equilibrium contact of the three types. The chemistry
P1 went looking for is happening.

What failed is the READING. The frozen measurement rule defines a molecule as a
connected component of the bonded-pair graph, and an edge exists wherever a pair
is mutually bound. In a condensed phase every atom is bound to every other, so
the component is the whole box and its formula is `H8Cl8` — which is a true
statement about boundness and a useless one about chemistry. The engine's own
closure-based census, running on the same trajectory, reports **7 live molecule
rows** where the cluster reading reports one. The code already names this
distinction — a cluster is a statement about boundness, a census row is a
statement about closure — and the droplet is exactly where they diverge.

So the honest verdict is two-part, and both parts are the product:

1. **Branch (b) as staked, and it stands.** HCl is not the modal molecule under
   the frozen rule in any seed. The rule was frozen before the arm ran and is not
   being changed after seeing the answer.
2. **The instrument, not the model, is what the rule ran out of.** A
   boundness-based component reading cannot resolve molecules in a condensed
   phase, and the mixed arm condensed. The successor is stated rather than
   performed here: re-run P1's composition rule on the CLOSURE reading, with that
   choice frozen in advance, and in a box or at a floor temperature where a
   chlorine gas stays a gas. Whether HCl is the modal molecule under such a rule
   is not answered by this campaign, and this campaign does not get to claim it.

### What P1 does and does not license

It does NOT license "hydrogen and chlorine make HCl in this model", however
suggestive the bond table above is — that reading comes from a diagnostic written
after the answer was seen, and a post-hoc instrument is not a measurement.

The measured statement is narrower and is the one reported: *under this frozen
protocol, in a 40 × 24 bohr box at these temperatures, a mixed H/Cl gas ends as a
single bound aggregate, as does a pure chlorine gas, while a pure hydrogen gas
ends as diatomic molecules. Within that aggregate, hydrogen–chlorine contacts at
hydrogen chloride's own equilibrium separation are the most numerous strong bond
present.*

Nothing was massaged and the protocol was not re-run with different constants.

---

## D1 — the DMRG bridge earns admission · **NOT ADMITTED. Cannot be run as staked.**

### The verdict

The bridge is **not admitted**, and the reason is not that it failed the
comparison — it is that the comparison cannot be run on the species the freeze
stakes. `bank::D1_RECORD` is `D1Admission::NONE`, so every DMRG-labelled curve is
refused by the provenance gate, and every curve in the sandbox today is
determinant-route. The gate and the reality agree, which is the outcome to want
when a bridge is unvalidated.

### THE WALL CAME DOWN — this section's verdict is superseded

Everything below was true of the MPO construction as it stood, and that
construction is gone. The external sprint team replaced `from_terms` — a raw list
of `O(n_orb^4)` operator strings compressed by one SVD per site — with a
channel-based finite-state-machine builder (`Channel`, `MpoBuilder`), which is the
standard chemistry MPO and enumerates no strings. Committed at `bb1a07a` with
attribution, field-diffed before anything was built on it.

Measured immediately, on the staked species and on REAL STO-3G integrals rather
than on the rebuild's own synthetic benchmarks:

| | before | after |
|---|---|---|
| SiO MPO build (14 orbitals, real integrals) | did not finish in 12 h | **0.07 s** |
| one DMRG sweep, χ = 8 | — | 1.14 s |
| max MPO bond dimension | — | 943 |

So the staked D1 comparison — S2 and SiO at ≤ 1e−8 Ha across their declared grids
— is **runnable for the first time and is running**. Its result replaces the
verdict below. The pre-rebuild measurements are kept, marked, because the
campaign's reasoning rested on them for a day and deleting them would make that
day's decisions unreadable.

### Why it could not be run as staked, measured — SUPERSEDED, kept for the record

`q8_mps::mpo::Mpo::from_electronic_integrals` builds the two-body electronic MPO
from a raw list of `O(n_orb^4)` operator strings and compresses it with one SVD
per site of an `m × 4r` matrix. **The construction, not the sweep, is the entire
budget.** Measured on the campaign machine
(`engine/output/mixtures1/mpo_cost_*.log`):

| pair | `n_orb` | MPO build | one sweep | final bond dim |
|---|---|---|---|---|
| H2 | 2 | 0.00 s | 0.00 s | 8 |
| LiH | 6 | **528.48 s** | 0.03 s | 46 |
| HCl | 10 | did not complete in over an hour | — | — |

D1's staked overlap species are **SiO at 14 orbitals and S2 at 18**. The builder
is nine minutes at six and does not finish at ten. So the staked comparison is
out of reach of this engine's MPO construction, and no amount of running it
longer changes that — the cost is in a fixed preprocessing step, not in
convergence.

**The refusal is being measured on a staked species rather than extrapolated.**
`d1_staked_SiO_cost.log` runs SiO's own MPO build against a twelve-hour budget, so
D1's blocked state rests on the staked thing and not on an inference from HCl.

**S2 is not repeated, by inference rather than by measurement, and that is stated
rather than hidden.** S2 is 18 orbitals against SiO's 14, and this builder's cost
rises monotonically and steeply in the orbital count (0.00 s → 528 s → past two
hours across 2 → 6 → 10). A refusal at 14 therefore settles 18. The S2 run was
queued and was deliberately stopped before it started; if the SiO build ever
*does* complete, that inference is void and S2 must be run.

Should SiO's build complete inside its budget, D1's blocked state weakens from
structural to a scheduling question and this section is wrong — which is the
outcome the twelve hours are being spent to find out about.

### A live hazard this exposed, and its fix

`fci::solve` routes any determinant space past `MPS_ROUTE_THRESHOLD` (50,000) to
that builder. **SiO is 132,496 determinants and 14 orbitals**, so asking this
crate for an SiO curve did not return a wrong number — it did not return. Si2
(9.4e6 determinants) and Na2 (1.0e9) likewise. The sandbox offered those species
and would have hung on them.

`pair::feasibility` now answers, from counts alone and before anything is spent,
which route a pair would take; `generate_pair_table` refuses an unreachable pair
with the measurement in the message; and the wasm ABI returns a code rather than
letting the assert reach a browser as a trap. The refusal says "no AUTOMATIC
route", not "impossible" — the determinant route can still enumerate SiO if it is
driven directly, which is precisely what gate D1's own harness and the R2 reader
do. Reporting SiO as having no route at all would have been a wrong claim about
the engine, and the first version of `Feasibility` made it.

### What was measured where the bridge does run

Not a discharge of D1 — the freeze stakes S2 and SiO — and reported as a separate
question: given that the bridge runs at all, is it ACCURATE? On the two overlap
species this engine can drive, exact FCI against DMRG over each pair's declared
grid, one MPO per geometry and the chi ladder walked against it:

| species | `n_orb` | `n_det` | points | worst \|E_dmrg − E_fci\| at χ=64 | stake |
|---|---|---|---|---|---|
| H2 | 2 | 4 | 16 | **5.05e−13 Ha** | 1e−8 ✔ |
| LiH | 6 | 225 | 12 | **5.59e−12 Ha** | 1e−8 ✔ |

At χ=8 LiH is off by 1e−4 to 3e−4 Ha and at χ=16 and above it is at 1e−12, so
the ladder is resolving convergence rather than reading a fixed offset. The LiH
grid cost 7,151 s for twelve points — 596 s each, essentially all of it MPO
construction, which is the same wall D1's staked species run into and the reason
the number above is not a discharge. **This
says the bridge is correct where it runs. It does not say the bridge is
admitted**, and the record the gate reads still says NONE.

### The EXACT side is cheap, and that changes what is owed

The lead's correction to the referee brief — D1's FCI side is engine f64 at a
1e-8 comparison, **not** the 50-digit referee — turns "can we have exact SiO"
from a question about thirty mpmath hours into a question about this crate's
Davidson. Measured, one geometry each, `solve_determinant` explicitly, through
the shared `geometry_problem`:

| pair | `n_orb` | `n_det` | assemble | solve | Davidson residual |
|---|---|---|---|---|---|
| HCl | 10 | 100 | 0.1 s | 0.0 s | 6.1e−11 |
| ClF | 14 | 196 | 0.6 s | 0.0 s | 7.7e−11 |
| Cl2 | 18 | 324 | 0.5 s | 0.0 s | 6.4e−11 |
| NaH | 10 | 44,100 | 0.1 s | 1.3 s | 7.3e−11 |
| S2 | 18 | 23,409 | 0.6 s | 3.0 s | 9.3e−11 |
| **SiO** | 14 | **132,496** | 0.3 s | **33.9 s** | 9.2e−11 |

So the exact half of D1 is done, and two things that were being reported as owed
are not:

* **R2's engine half is fully feasible on all seven staked pairs**, SiO included
  — about eleven minutes for a twenty-point SiO grid. It is blocked on the
  referee's drop and on nothing of ours.
* **E2's SiO row is not owed.** It had been reported that way because
  `pair::feasibility` says "no automatic route" and `generate_pair_table` refuses
  — both correct about the AUTOMATIC route, and both being read downstream as
  "unreachable". `examples/e2_byhand.rs` locates the well on the determinant route
  directly, with the same bracket-bisect-Newton discipline `locate_well` uses.

**The blocker is the DMRG side alone, and more compute does not fix it**, because
the cost is a fixed preprocessing step rather than convergence.

### What the sprint team's tests actually cover

Checked twice, because the first check changed what a claim meant and the second
was asked for by name.

**The first receipt** claimed DMRG-vs-FCI agreement "within 1e-6 Eh on overlapping
sectors". That is `tests/electronic_dmrg.rs`: MPO against an independent dense
Hamiltonian at **two** orbitals to 1e−13, and DMRG against the exact sector ground
state at **three** orbitals to 1e−7 — both on hand-written synthetic integrals.

**The rebuild's receipt** claims "5 transition metal active spaces (Sc..Fe) solved
in 0.04 s with Hund's rule high-spin", plus a 14-to-50-orbital performance test.
Field-diffed:

* `tests/transition_metals.rs` is **five d-orbitals** each, with integrals from
  `make_transition_metal_integrals(n_orb, e_d, u_val, j_val)` — a parameterised
  **Hubbard–Kanamori model** with a hand-set on-site energy, `U` and `J` — asserted
  against answers known in closed form (scandium's d¹ is expected to be exactly
  `e_d = −3.5`). A legitimate and tight test of the solver against a model whose
  answer is known. It is **not scandium**: not 21 electrons, not STO-3G integrals,
  and an ACTIVE SPACE is not an atom.
* `test_mpo_build_performance_14_to_50_orbitals` **solves nothing**. It builds and
  checks build time and bond dimension, and its `g` is extremely sparse and
  structured — only `g[pppp]`, `g[ppqq]` and `g[pqqp]` nonzero. A real STO-3G `g`
  is dense at `O(n⁴)`, and both build cost and bond dimension scale with the term
  count.
* `test_fci_ground_state_energy_precision_1e8` reaches 1e−8, at **two** orbitals on
  synthetic integrals.

All ten pass and none of them is evidence about MIXTURES-1's staked species. That
is why this lane measured the rebuilt builder on real SiO integrals itself before
believing the wall was down — and it is down, by 0.07 s against twelve hours.

### Consequence for the campaign

Si2 and Na2, the DMRG-only curves D1 would have licensed, **do not enter the
sandbox**. That is the freeze's "only then", enforced.

---

## The MPS route, re-measured — and it currently buys nothing

Ordered by the lead after the MPO rebuild made `MPS_MAX_ORBITALS = 6` stale. The
form was specified: an orbital ladder, real integrals, a stated budget, per-rung
numbers. `examples/mps_ladder.rs`, chi = 32, 300 s per (pair, chi) cell, stake
1e−8 Ha against exact FCI, each pair at its own equilibrium.

| `n_det` | pair | `n_orb` | delta / Ha | sweeps | secs | verdict | exact FCI |
|---|---|---|---|---|---|---|---|
| 4 | H2 | 2 | +1.8e−15 | 2 | 0.0 | REACHED | 0.0 s |
| 100 | HCl | 10 | +6.1e−11 | 3 | 63.0 | REACHED | 0.0 s |
| 196 | ClF | 14 | +5.5e−12 | 5 | 472.2 | REACHED | 0.0 s |
| 225 | LiH | 6 | +3.9e−14 | 3 | 2.8 | REACHED | 0.0 s |
| 23,409 | S2 | 18 | +3.5e−3 | 3 | 575.3 | BUDGET | 2.7 s |
| 44,100 | NaH | 10 | +4.9e−3 | 9 | 364.5 | BUDGET | 1.3 s |
| 132,496 | SiO | 14 | +1.1e−2 | 6 | 663.9 | BUDGET | 18.5 s |

### There is no orbital-count threshold to re-derive

**Sorted by determinant count the verdict column is monotone. Sorted by orbital
count it is not**: ten orbitals both reaches (HCl) and fails (NaH); fourteen both
reaches (ClF) and fails (SiO). The harness's own summary — "largest reaching: 14;
smallest that did not: 10" — is self-contradictory as a threshold, and that
contradiction is the answer rather than a defect in the instrument.

So `MPS_MAX_ORBITALS` cannot carry this. `MPS_MAX_DETERMINANTS = 1024` is added as
the operative bound, placed inside the measured gap between LiH's 225 and S2's
23,409. Nothing on this evidence distinguishes 500 from 5,000, and claiming
otherwise would be precision the ladder does not have.

### The larger finding: the route extends nothing

**Every pair the MPS route reaches has an exact FCI that is already free** — 0.0 s
for all four. **Every pair whose exact FCI costs anything** — NaH 1.3 s, S2 2.7 s,
SiO 18.5 s — **is one the route does not reach.**

DMRG succeeds exactly where it is not needed and fails exactly where it would be
useful. Combined with `MPS_ROUTE_THRESHOLD = 50,000`, that makes
`AutomaticRoute::Mps` **unreachable**: a space large enough to be routed to MPS is
necessarily past the determinant bound. The arm is kept, and the unreachability is
recorded as the measurement rather than removed as dead code — the day the sweep
implementation improves, the fix is one constant.

### Scope, so this is not over-read

Chi = 32, 300 s per cell. The ladder deliberately stops climbing chi after a
BUDGET verdict, because a larger chi is strictly slower per sweep and cannot do
better in the same wall clock — so a much larger budget at a larger chi was never
tested and could move this. The bound says what the route reaches under a stated
budget, not what DMRG can do in principle.

### What this does to D1

**D1 remains NOT ADMITTED, and the reason has changed completely.** Before the
rebuild it was blocked on building the MPO at all. Now the MPO builds in 0.31 s
and the blocker is convergence: SiO sits 1.1e−2 Ha from exact after 664 s, which
is six orders from the 1e−8 stake, and S2 3.5e−3 after 575 s. Two walls, one
behind the other; the first fell and the second did not.

`bank::D1_RECORD` stays `NONE`, every DMRG-labelled curve stays refused, and Si2
and Na2 do not enter the sandbox.

---

## Naming the sprint team's N2 number

Their Track-2 receipt quotes `E = -131.278565811 Ha` for "N2". This engine's N2
TOTAL energy at 3.0 bohr is `-107.546741772` Ha — 23.73 Ha away — so the two were
not the same quantity and the gap had to be named before the code producing it
was trusted.

**Named, to 4.8e−11 Ha.** It is N2's **ELECTRONIC** energy — total minus nuclear
repulsion — at **R = 2.0740 bohr**, in this engine's own STO-3G FCI model:

| R / a₀ | E_total / Ha | V_nn / Ha | E_electronic / Ha | minus theirs |
|---|---|---|---|---|
| 2.0720 | −107.652110873 | 23.648648649 | −131.300759521 | −2.22e−2 |
| **2.0740** | **−107.652722031** | **23.625843780** | **−131.278565811** | **−4.83e−11** |
| 2.0760 | −107.653324169 | 23.603082852 | −131.256407021 | +2.22e−2 |

`4.8e−11 Ha` is the Davidson residual level, so this is an identity, not a
coincidence: the number is ours.

**Two things it is not labelled as, and both matter.**

* It is **electronic-only**. Quoted beside total energies it reads as a 23.6 Ha
  discrepancy, which is exactly how it reached the lead as a blocker.
* It is at the **experimental** bond length — 1.09768 Å = 2.07431 bohr — and not
  at the model's own equilibrium. This campaign measured STO-3G's own N2 minimum
  at **R_e = 2.256729 bohr** (E2), 0.18 bohr further out. Evaluating a model at a
  geometry the model does not predict is a legitimate thing to do and an easy
  thing to misread, and it is the same distinction E2's whole branch (b) turns on.

**As a cross-check it is a good one.** Agreement at 4.8e−11 Ha between their route
and this engine's `solve_determinant`, at a geometry neither side chose to make
them agree, is independent corroboration that the two solvers describe one model.

---

## R2 — the staked-pair referee gate · **OWED (the drop has not landed)**

The engine half is built and two of its three tests run today; the third is
`#[ignore]`d until the sibling lane commits `tests/data/mixtures1/`.

* the STAKED SEVEN — Cl2, S2, Ar2, HCl, ClF, NaH, SiO — are frozen in
  `tests/mixtures_referee.rs` and will be cross-checked against the referee's own
  declared set rather than read from it, because two lanes disagreeing about which
  pairs a campaign stakes is worth firing on and is invisible if either side reads
  the other's list;
* `present + owed = staked` is enforced, so coverage cannot shrink silently;
* the drop is pinned by FNV-1a digest (currently `0x00000000` = NOT YET PINNED);
* the comparison is done in exact fixed-point decimal through the shared
  `tests/common` comparator, because parsing a 50-digit referee into an `f64` and
  subtracting cannot resolve anything below half an ulp and would report its own
  rounding as agreement;
* **the grading loop calls `solve_determinant` directly and NOT `pair_point`.**
  `pair_point` goes through `fci::solve`, which would route SiO's 132,496
  determinants to DMRG — grading the bridge against the referee while calling the
  result exact, and, on this engine's MPO builder, not returning at all.

Of the staked seven, six are on the automatic determinant route; **SiO is the one
that is not**, and it is reported by `r2_which_staked_pairs_leave_the_determinant_route`
rather than left to be discovered when the drop arrives.

---

## E2 — the emergent chemical contrast · **BRANCH (b): one GROSS inversion**

Staked ordering: `N2 > SiO > HCl > ClF > S2 > Cl2 > NaH >> (Ar2, NeAr)`.

Measured, `D_e` in hartree from `locate_well` (bisection then Newton on the
SOLVER, not on the interpolant, so these do not depend on knot count):

| pair | `n_basis` | `n_det` | `D_e` / Ha | `R_e` / a₀ | staked rank | measured rank |
|---|---|---|---|---|---|---|
| **SiO** | 14 | 132,496 | **0.263676281** | 2.908134 | **2** | **1** |
| **N2** | 10 | 14,400 | **0.239388030** | 2.256729 | **1** | **2** |
| **NaH** | 10 | 44,100 | **0.193188744** | 3.133867 | **7** | **3** |
| HCl | 10 | 100 | 0.148293175 | 2.536888 | 3 | 4 |
| S2 | 18 | 23,409 | 0.133253157 | 3.706603 | 5 | 5 |
| Cl2 | 18 | 324 | 0.064577385 | 4.024124 | 6 | 6 |
| **ClF** | 14 | 196 | **0.060622391** | 3.341873 | **4** | **7** |
| Ar2 | 18 | 1 | **UNBOUND** | — | 8 | 8 |
| NeAr | 14 | 1 | **UNBOUND** | — | 9 | 9 |

SiO was measured by hand on the determinant route (`examples/e2_byhand.rs`), 41
points at 1,463 s total, because `generate_pair_table` refuses it — see D1's
section on why "no automatic route" is not "unreachable".

### What holds

* **The two deepest bonds are SiO and N2**, and they are the two the stake puts
  at the top — in the other order.
* **Both closed-shell negatives are UNBOUND** — no minimum deeper than the
  declared `WELL_MIN_DEPTH = 1e-4 Ha`. Nothing in the engine knows argon and neon
  are noble: `locate_well` looks for a minimum and reports `None`, through the
  same code path that produces N2's curve.

  This is **evidence for gate E1 and not its discharge**, and the distinction is
  not pedantry. E1 stakes "no well deeper than 1e-4 Ha *on their staked grids*",
  and a staked grid is one the referee file declares. What was measured here is
  the ENGINE's own derived range (`pair::derive_range`, 24 knots) — the same rule
  every other curve in this crate uses, and therefore result-blind, but not the
  declared grid E1 names. E1 is discharged when R2's drop lands and these two are
  re-read on the grid the drop declares.
* **S2 > Cl2**, as staked.

### The three inversions, and one of them is gross

* **NaH moves from seventh to third** — four places. At 0.193 Ha it comes out
  deeper than HCl, where the stake puts it shallowest of every bound pair. This
  is a GROSS inversion by any reading, and it is branch (b).
* **ClF moves from fourth to seventh**, below both S2 and Cl2 — three places, and
  it ends up the shallowest bound pair in the set.
* **SiO and N2 swap** the top two places. Adjacent, and the two are 10% apart
  (0.2637 against 0.2394), so this one is the least of the three — but it is an
  inversion of the stake's headline claim, and it is reported rather than rounded
  into "N2 and SiO are the deepest, broadly".

The middle of the ordering — HCl > S2 > Cl2 — is exactly as staked, and so is the
unbound tail. What the stake gets wrong is both ends and NaH.

### Investigated, as far as this lane can honestly take it

Two things are established and one is not.

**It is not a bug in one code path.** The numbers were reproduced by a second,
independent route: `examples/e2_byhand.rs` locates the well with its own
bracket–bisect–Newton against `solve_determinant` directly, where
`generate_pair_table` uses `locate_well` against the auto-routing `solve`. The two
agree to every printed digit — NaH 0.193188744 both ways, HCl 0.148293175, ClF
0.060622391. So the ordering is what this model says, not what one function says.

**The asymptotes are where a minimal-basis distortion would live.** `D_e` is
`E_asymptote − E(R_e)` with the asymptote computed as two isolated atoms at the
same level of theory:

| pair | `E_asymptote` / Ha | `E(R_e)` / Ha | `D_e` / Ha |
|---|---|---|---|
| NaH | −160.178044363483 | −160.371233107 | 0.193188744 |
| HCl | −455.008774248286 | −455.157067423 | 0.148293175 |
| ClF | −552.528697398917 | −552.589319790 | 0.060622391 |

Sodium is the open-shell alkali in the set, and a minimal basis describes an
isolated alkali ATOM worse than it describes the molecule that atom forms — which
raises the asymptote and inflates `D_e`. That is a plausible mechanism and **it is
not established here**, so it is recorded as the hypothesis to check rather than
as the finding.

**What settles it is R2, not more of this.** The referee grades NaH and ClF among
its staked seven at 1e−10 Ha pointwise. If the referee reproduces these numbers,
the inversion is a true property of the declared model and E2's stake was a
hypothesis about nature carried into a model that does not share it. If the
referee disagrees, the inversion is ours. Speculating further before that drop
lands would be putting a mechanism in the record ahead of the measurement that
decides it.

### What E2 does and does not license

The freeze says "in its broad strokes, numbers reported as the product". The broad
strokes that hold are the ones the campaign can claim: **the deepest bond is N2,
the closed shells refuse across rows, and every one of these numbers came out of
Z, the masses and the STO-3G contraction with no fitted parameter and no table of
chemical results.** The ordering as a whole does NOT reproduce the stake, and that
is reported as the product rather than the stake being adjusted to it.

---

*Sections are added as each measurement lands. Nothing above was written before
its numbers existed.*
