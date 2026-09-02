# THE MPS CLUSTER SEAM — prereg

*Frozen 2026-09-02, before any instrument of this node exists. This file lands ALONE, in
its own commit; `git log --oneline -- conformance/atomworld/MPS_SEAM_PREREG.md` is the
proof of order, and the instruments named in §6 are absent from the tree at that commit.*

**misfits:** M-UNTESTED-GAP, M-MAX-OVER-SUCCESSES, M-PLACEMENT-LOTTERY,
M-IDLE-CALIBRATED-TIMEOUT, M-CHEAPER-THAN-ITS-PRICE, M-BUDGET-LAUNDER, M-VACUOUS-SUCCESS,
M-BASE-RATE-OMITTED, M-PLANT-OBS, M-PLANT-SECTOR, M-EXIT-DISCRIMINATOR, M-DEVICE-CLASS,
M-STALE-INSTRUMENT, M-FOREIGN-DOMAIN-CORROBORATION, M-HOMOG, M-PARITY-PROTECT,
M-PROBE-EIGENSTATE, M-CACHE-KIND, M-PROVENANCE-OVERREACH.

Why each is contacted, in one line:

* **M-UNTESTED-GAP** — this node's whole subject. `MPS_MAX_ORBITALS`'s own header records
  the predecessor failure: S2's 23,409 determinants sat in a hole between all successes
  (≤225) and the one measured failure (44,100), and a reach was staked across it. Every
  reach claim below names ALL THREE axes — orbital count, filling, determinant count —
  and no rung is interpolated.
* **M-MAX-OVER-SUCCESSES** — the constant under re-measurement was itself repaired for
  this misfit (the ladder nominated `best_reached` = 14 where the admission door needs
  `first_wall − 1` = 9). Both numbers are reported separately at every rung below and are
  never conflated.
* **M-PLACEMENT-LOTTERY / M-IDLE-CALIBRATED-TIMEOUT** — see §2.3. The inherited cap was
  measured against a **300 s wall clock** on a heterogeneous box. This freeze does not
  budget in seconds.
* **M-CHEAPER-THAN-ITS-PRICE** — §5 states a cost model for both routes, in advance, and
  §7's gates refuse a result that arrives outside it.
* **M-BUDGET-LAUNDER** — §8: exhaustion VOIDs, never scores, and the VOID structure is
  printed so a correlated refusal pattern is visible.
* **M-VACUOUS-SUCCESS** — §4.3. A seam whose core already contains the whole cluster
  reproduces `E_FCI` trivially. The acceptance is a LADDER in the core radius with that
  vacuous point kept as a labelled control, never as a result.
* **M-BASE-RATE-OMITTED** — every verdict below carries the fraction of dE5's own 177-candidate
  landscape it speaks for.
* **M-PLANT-OBS / M-PLANT-SECTOR** — §9; every plant names its carrier and the sector the
  carrier must be nonzero in, and is pre-checked to fire.
* **M-EXIT-DISCRIMINATOR** — §6.2: every solve record carries WHY it stopped, and the
  discriminator is printed, not merely carried.
* **M-DEVICE-CLASS** — every reading is `cpu`; no GPU arm exists in this node. Declared in
  §6.2, not defaulted.
* **M-STALE-INSTRUMENT** — the results document will carry each instrument's commit; §6.3
  states the path discipline (gate 10a3), which this node needs by hand because 10a3's
  grep does not reach `engine/crates/`.
* **M-FOREIGN-DOMAIN-CORROBORATION** — §11. The crystal tier's referee is a QED₂ result. It
  is cited as a MACHINERY licence with its fence restated verbatim, and it is NOT evidence
  about molecular orbitals.
* **M-HOMOG** — the seam's whole content is a `local` core against a `distant` far field;
  §4.1 states the criterion that decides which is which, before any reading.
* **M-PARITY-PROTECT** — the Jordan–Wigner mapping carries parity strings, and total
  particle number and `Sz` are protected sectors of the electronic Hamiltonian. §7's G-C2
  gates that protection as a MEASURED invariant rather than assuming it: see §2.2, where
  it is already measured to be violated.
* **M-PROBE-EIGENSTATE** — the DMRG arm's carrier is the Hartree–Fock product state, which
  IS an eigenstate of the one-body part. §9's P-4 names the sector its plant must move.
* **M-CACHE-KIND** — §6.4: the two routes produce records of different KIND (exact-in-model
  vs variational-with-a-ledger). Kind is in the record key; a shape mismatch raises.
* **M-PROVENANCE-OVERREACH** — the results document records the sha256 of every banked
  reading and claims nothing about the run beyond what the launch header actually carries.

**Armed by keyword and NOT otherwise contacted**, each cited so the arming is answered
rather than dodged:

* **M-MAINTENANCE-LENS** — armed by "repair"/"maintenance". Nothing here is a rent-clause
  reading; the words appear only in the fence law's sense ("a fence is a bug under repair")
  and in naming a repaired constant.
* **M-VOLUME-SCALE** — armed by "lattice"/"N-convergence". This node runs no lattice and
  takes no thermodynamic limit; the only size axis is the cluster's atom count, which is
  enumerated, not extrapolated.
* **M-LOOP-BLIND** — armed by "loop". The only loops here are a sweep loop and a program
  loop; no holonomy or Wilson loop is computed.
* **M-BARE-CHARGE** — armed by "charged". No charged fragment enters this node; every
  cluster is neutral `OxHy`, and the ionic species are node C's.
* **M-COND-PROBE** — armed by "inside t…". No conditional probe and no endogenous
  conditioning appears; every sample is dE5's, drawn by dE5's frozen rule.

---

## 0. THE QUESTION

GANTT node `MPS` exists because two independent measurements fired the same requirement:

* **dE5** (`conformance/water_observatory/DE5_RESULTS.md`, branch (b)): on compact planar
  `O2H3`, the atom-based many-body expansion **does not terminate at four**. 24 of 24 live
  configs over the ladder's own 5.0e-5 Ha declared per-term uncertainty; worst
  `|dE5|` = 7.859411e-2 Ha = **1,572×** it; on the worst config the five-body residue is
  **2.83×** the entire four-body rung.
* **CRYO arm 3** (`conformance/atomworld/CRYO_HO_RESULTS.md`): the fragment-local expansion
  never converges for compressed hydrogen at any density, `dE3` and `dE4` opposite-signed
  and comparable, 10 of 10 rungs.

The staked answer is **exact solves of compact clusters over an MBE far field**. The node's
BOTTOM is `FENCES.md` finding **F-5**: the MPS-ROUTE band has no reachable member, so the
seam must move a cap or the routing, never merely add itself.

**This freeze asks four separable questions:**

* **Q-A (the determinant cap).** `fci::HARD_DETERMINANT_CAP` is a **policy**, not a
  capability — its own header says so and says how to move it: *"Raise it deliberately,
  with a measurement, the way `MPS_MAX_ORBITALS` was."* What does the determinant route
  actually cost at the cluster sizes the seam needs, and where is its real wall?
* **Q-B (the MPS cap).** Re-measured against the CURRENT MPO construction, on the CLUSTER
  axis rather than the pair axis, and with the discriminator the pair ladder lacks (§2.4):
  what does the DMRG route reach, at what stake, at what price?
* **Q-C (the canonical brick).** Are canonicality, the truncation ledger and the particle-number
  sector CHECKED invariants of the electronic DMRG path, or computed and discarded?
* **Q-D (the seam).** Does a core-exact / far-field-MBE seam reproduce `E_FCI(5)` on dE5's
  own 24 configs, inside a defect budget staked here before it is measured?

Each has its own kill and takes down nothing beneath it.

---

## 1. THE CORRECTION THIS NODE MAKES BEFORE ANY GATE

The node's own charter row in `GANTT.md` contains an arithmetic error which, uncorrected,
would point the whole node at the wrong wall. It is corrected here, pre-data, with the
derivation, because a freeze that inherits a wrong target measures the wrong thing.

> GANTT.md, MPS row, as written: *"a compact O2H3 is 13 orbitals / 17 electrons — past
> `MPS_MAX_ORBITALS` = 9 AND past `FCI_DET_MAX`"*.

`O2H3` in this campaign's minimal basis is, by the arithmetic `de5_audit.rs:618` already
uses (`n_orb = 5·nO + nH`, `n_elec = 8·nO + nH`, minimal `Sz`):

| | |
|---|---|
| orbitals | 5·2 + 3 = **13** ✓ |
| electrons | 8·2 + 3 = **19**, not 17 |
| determinants | `C(13,10)·C(13,9)` = 286·715 = **204,490** |

204,490 is **below** `de5_audit.rs`'s `FCI_DET_MAX` = 250,000 and **below**
`fci::HARD_DETERMINANT_CAP` = 2,000,000. It is not past either. The proof is dE5 itself:
`O2H3` was the audit's ONLY in-scope composition and all 24 configs were solved on the
determinant route. **The compact `O2H3` pentamer is exactly solvable today.**

What is NOT solvable today is the rest of dE5's own landscape:

| composition | orbitals | electrons | determinants | candidates | share | status |
|---|---:|---:|---:|---:|---:|---|
| `O2H3` | 13 | 19 | 204,490 | 31 | 17.5% | reachable, and dE5 reached it |
| `O3H2` | 17 | 26 | 5,664,400 | 110 | **62.1%** | refused by `HARD_DETERMINANT_CAP` (2.83× over) |
| `O4H1` | 21 | 33 | 121,788,765 | 36 | 20.3% | past the route by memory as well as time |

**So the node has two walls, not one, and they are made of different material.** The
62.1% rung is behind a POLICY constant whose own doc invites a measurement. The 20.3% rung
is behind arithmetic. This is the correction that shapes §4 and §7, and the results
document must repeat it in the same sentence as any cap verdict.

---

## 2. DISCLOSED — everything seen before this freeze

**These are PRIORS, not results.** They come from a scratch scoping probe
(`mps_scope_probe.rs`) written, run and DELETED before this freeze; it is not an instrument
of this node, nothing below is gated on it, and every number here is re-measured by a
committed instrument under §7's gates. They are disclosed because they shaped the design.

### 2.1 The MPO's bond-dimension profile is asymmetric, and the asymmetry grows with L

`Mpo::from_electronic_integrals` builds channels keyed by SITE INDEX
(`Channel::LeftPair(x1,x2)` and siblings). The count of live pair channels at bond `b`
grows like `b²`, so the profile rises monotonically toward the RIGHT end of the chain
instead of peaking in the middle:

| case | orbitals | L | `D` at middle | `D_max` | at bond | `D_max/D_mid` | Σ`D` | Σ`min(D_b, D_{L−b})` | ratio |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `O1H4` | 9 | 18 | 183 | 334 | 15 | 1.83 | 2,758 | 1,540 | **1.79** |
| `O2H3` | 13 | 26 | 371 | 704 | 21 | 1.90 | 8,632 | 4,198 | **2.06** |
| `O3H2` | 17 | 34 | 631 | 1,252 | 29 | 1.98 | 20,266 | 8,714 | **2.33** |
| `O4H1` | 21 | 42 | 947 | 2,389 | 34 | 2.52 | 41,412 | 15,678 | **2.64** |

The last column is the factor a left/right-symmetric construction would remove from the
sweep's dominant cost, which is linear in `D`. It is **worse exactly where the seam needs
it**. MPO build time and dense size were also read: 0.048 s / 17.9 MB at 9 orbitals rising
to 3.44 s / 1,946 MB at 21. The 1.9 GB is itself a wall and is not the sweep's.

### 2.2 The electronic DMRG path loses particle number, and never checks

`O1H4` (9 orbitals, 12 electrons), 3 sweeps from the HF product state:

| χ | s/sweep | `E` reported | `E_FCI − E` | `⟨N̂⟩` (want 12) | `‖ψ‖²` | max discarded |
|---:|---:|---:|---:|---:|---:|---:|
| 8 | 1.9 | −89.539522237661 | 2.333e-2 | 12.000000 | 1.000000 | 8.67e-5 |
| 16 | 11.8 | −89.553590690850 | 9.264e-3 | **12.000041** | 1.000000 | 1.19e-4 |
| 32 | 35.4 | −89.560342498538 | 2.512e-3 | **11.999976** | 1.000000 | 2.79e-5 |

`E_FCI` = −89.562854529277 Ha (determinant route, 0.5 s, 29 iterations, residual 8.4e-11).

Three separate facts, and they are not the same fact:

1. **`⟨N̂⟩` drifts off the requested integer by up to 4.1e-5.** The electronic Hamiltonian
   commutes with `N̂`, so an exact sweep from a definite-`N` seed cannot do this; the
   engine carries no quantum numbers and the drift is numerical leakage across the
   truncation. It is COMPUTED on every run (`DmrgResult::spin_orbital_occupations`) and
   **checked nowhere**. A variational energy in a leaked sector is not a bound on the
   sector that was asked for.
2. **The reported energy IS the variational expectation.** A hypothesis this node held
   before measuring — that `dmrg_sweep` returns a local Lanczos eigenvalue below the true
   `⟨ψ|H|ψ⟩` of the truncated state it hands back — is **FALSE**: measured
   `⟨ψ|H|ψ⟩ − E_reported` = +1.2e-12, +3.0e-12, +6.9e-12 at χ = 8, 16, 32. Recorded here
   as a tested-and-rejected prior so it is not re-litigated downstream.
3. **The truncation ledger is bypassed on the only path that reaches production.**
   `solve_electronic_ground_state` hardwires `RefusalPolicy::Silent`, so `FENCES.md`'s C9
   (`REFUSAL_THRESHOLD` = 1e-4 discarded weight, a typed FLOOR refusal) cannot fire for
   any caller of `fci::solve_mps`. The χ=16 reading above has a max discarded weight of
   1.19e-4 — **over C9's threshold** — and would have been returned as an answer.

### 2.3 The inherited cap was budgeted in wall-clock seconds on a contended, heterogeneous box

`MPS_MAX_ORBITALS` = 9 was measured (`mps_ladder.rs`) against `CELL_BUDGET_S` = **300 s of
wall clock**, and its verdicts are BUDGET verdicts: NaH, SiO and S2 are recorded as
not-reached because a 300 s timer expired. Measured on this machine while writing this
freeze: `13th Gen Intel Core i9-13900HX`, 16 P-cores at 5.2–5.4 GHz and 16 E-cores at
3.9 GHz, and a **load average of 63–65 on 32 hardware threads** — the box is roughly 2×
oversubscribed by sibling lanes at all times. M-PLACEMENT-LOTTERY registers a measured 57%
scaling between the two core classes on this exact box, and records a head-to-head verdict
that FLIPPED when both arms were pinned to the same core.

**A cap defined by a wall-clock timer on this box is a measurement of the queue.** No gate
in this freeze is budgeted in seconds. §5 defines the work unit both routes are priced in.

### 2.4 The pair ladder's χ rule cannot tell a truncation floor from a time limit

`mps_ladder.rs` stops climbing χ after a BUDGET verdict, on the stated reasoning that *"a
larger chi is strictly slower per sweep, so it cannot do better in the same wall clock."*
That is true of throughput and false of accuracy: an error that is TRUNCATION-limited does
not improve with more sweeps at the same χ and improves only with a larger one. The two
cases are distinguishable — by whether the per-sweep energy has plateaued while the
discarded weight has not — and the ladder does not distinguish them. §7's G-B2 makes the
discriminator a required field of every rung.

### 2.5 A determinant cost model that reproduces the referee's own count exactly

For a space of `n_det` determinants over `n_orb` orbitals at `(nα, nβ)`, the number of
nonzero Hamiltonian elements the direct sigma touches is

```
nnz/det = [nα·vα + C(nα,2)·C(vα,2)] + [nβ·vβ + C(nβ,2)·C(vβ,2)] + (nα·vα)(nβ·vβ) + 1
```

with `vσ = n_orb − nσ`. For SiO (14 orbitals, 3/3, 132,496 determinants) this gives
1,486 per determinant and **196,889,056** in total — which is, to the digit, the number the
mixtures referee lane measured for SiO. Two live timings against it:

| case | determinants | model `nnz` | Davidson iterations | wall | s/iteration |
|---|---:|---:|---:|---:|---:|
| `O1H4` | 7,056 | 3.18e6 | 29 | 0.5 s | 0.017 |
| `O2H3` | 204,490 | 3.06e8 | 71 | 106.4 s | 1.50 |

The ITERATION COUNT is the axis the model does not predict (29 vs 71 at a 29× size ratio),
which is why §7's G-A2 gates the price per iteration and reports the count separately
rather than folding them into one number.

### 2.6 What was NOT seen

No cluster larger than `O2H3` has been solved by any route. No DMRG run past χ = 32 or past
3 sweeps has been made. No seam has been assembled. No orbital reordering has been tried.
No reading in §7 exists yet.

---

## 3. SCOPE, AND THE FENCES THAT ARE NOT NEGOTIABLE

Restated in full beside every verdict, every time:

* **Compositions**: `OxHy` only, `x + y = 5`, from dE5's own enumeration. No other element
  enters this node.
* **Basis**: STO-3G, all electrons correlated, no frozen core — identical to dE5's, so the
  two are comparable.
* **Geometry**: the acceptance set is dE5's own 24 live configs, drawn by dE5's frozen
  sampling rule from the pinned trajectories, and therefore PLANAR and of diameter below
  6.0 bohr. This node does not re-draw them and does not add any.
* **Device class**: `cpu` for every reading. This node has no GPU arm; node F is the exit.
* **Arithmetic regime**: f64 throughout, `DAVIDSON_EXPANSION_FLOOR` unmoved. Nothing in
  this node edits a solver tolerance.
* **What a DMRG energy is**: variational, truncation-limited, and NOT exact in model. Every
  DMRG reading carries χ, the max discarded weight, the kept-spectrum floor, the canonical
  residual and `⟨N̂⟩`. A DMRG number that is presented without those five is a defect of this
  node, not a reading of it.

---

## 4. THE DESIGN

### 4.1 What the seam IS, stated before it is built

Given a scene, a **core radius** `R_core` partitions the atoms into a CORE (every atom
within `R_core` of the core seed) and a FAR FIELD. The seam energy is

```
E_seam(R_core) = E_exact(CORE) + Σ MBE terms of order ≤ 4 that involve at least one far-field atom
```

The exact leg goes through `geometry_problem` + `fci::solve_determinant`, never
`fci::solve` — which routes past 50,000 determinants into the very thing under test, and the
`O2H3` pentamer at 204,490 is past it. This is dE5's own correction C-2 and it is inherited
verbatim, not re-derived.

`R_core` is the ONLY knob, it is swept, and its two endpoints are known in advance:

* `R_core ≥ diameter` ⟹ the core is the whole cluster ⟹ `E_seam = E_FCI(5)` **exactly**.
  This is a bit-identity CONTROL, labelled as one wherever it appears (§4.3), never a result.
* `R_core = 0` ⟹ no core ⟹ `E_seam = MBE4` ⟹ the defect is dE5's own `dE5`, worst
  7.859411e-2 Ha. This endpoint is already measured and is the curve's other anchor.

The seam's claim lives strictly between them, and the seam defect is
`|E_seam(R_core) − E_FCI(5)|`.

### 4.2 The seam defect budget — STAKED HERE, before it is measured

**The budget is 5.0e-5 Ha**, at the smallest `R_core` that keeps at least one atom in the
far field. That number is not chosen for comfort: it is the ladder's own declared per-term
uncertainty, the bar dE5 measured the truncation against, and the bar the seam exists to
restore. A seam that cannot beat its own predecessor's declared uncertainty has not
replaced anything.

Stated as a ratio so it survives a change of units: the seam must reduce the worst-config
defect by a factor of **at least 1,572** against pure MBE4, since that is the factor by
which MBE4 misses.

### 4.3 The vacuity control (M-VACUOUS-SUCCESS)

Every seam reading is published as a PAIR: the defect at the staked `R_core`, and the
defect at `R_core ≥ diameter`. The second must be **exactly zero to the solver's own
residual**, on every config. If it is not, the seam's assembly is wrong and no other number
in the arm may be read. If a verdict is ever reported without the pair, the reading is void
by this section — a seam scored only where its core swallowed the cluster is a verifier
reporting success for work it did not do.

### 4.4 The two caps, and what "moving" one means

* **Q-A**: `HARD_DETERMINANT_CAP` moves iff a measured price and a measured peak resident
  set exist for the space that would newly be admitted, both inside §5's model, with the
  measurement committed beside the constant. Otherwise it does not move and the refusal
  stands with a NUMBER attached.
* **Q-B**: `MPS_MAX_ORBITALS` / `MPS_MAX_DETERMINANTS` move iff the DMRG route REACHES a
  stated stake on a space past `MPS_ROUTE_THRESHOLD`, at a priced work unit, with the
  reach demonstrated at every axis of the target (M-UNTESTED-GAP) and the door taken as
  `first_wall − 1`, never as `best_reached` (M-MAX-OVER-SUCCESSES). **F-5 is deleted iff
  some real input selects `AutomaticRoute::Mps`**, which requires
  `MPS_MAX_DETERMINANTS > MPS_ROUTE_THRESHOLD`. Anything less is progress on the cap and
  leaves F-5 standing, and this freeze will say so in those words.

### 4.5 The improvements this node is permitted to make, and the order

Bottoms-up, cheapest and safest first. Each is measured before and after, and each carries
its own bit-identity obligation:

1. **Bit-identical parallelism.** `apply_effective_h_mpo`'s four stages each accumulate into
   an output indexed by a leading channel or row index; parallelising over that INDEX (never
   over a reduction index) leaves every summation order untouched. The obligation is
   therefore exact: the parallel path must be **bit-identical** to the serial one, gated,
   with a planted mis-parallelisation that fires. Thread count is an explicit parameter
   defaulting to 1 — this box already runs 30-way process parallelism from sibling lanes,
   and a library that silently spawns 32 threads per process is the BLAS-spin-thread defect.
2. **MPO symmetrisation.** Carry pair channels from whichever END is nearer, so the profile
   peaks in the middle. §2.1 prices this at 1.79–2.64×. The obligation is that the MPO's
   dense contraction is unchanged: `Mpo::dense()` must agree with the current construction
   to the solver's own floor on every case small enough to form it, and the ground-state
   energy must agree at fixed χ.
3. **Orbital ordering.** The chain order is currently the canonical SCF order, which for a
   cluster of weakly-coupled fragments interleaves them and maximises entanglement across
   the middle bond. A fragment-blocked order is a PERMUTATION of the integrals: the exact
   energy is invariant under it, so it is a free axis with a free control. Measured as an
   axis, never assumed to help.
4. **Nothing else.** In particular this node does NOT build quantum-number-blocked tensors.
   If §7's readings say the cap needs them, that is a named successor with its own freeze,
   and this node reports the fence with its build named — not a half-built symmetry.

---

## 5. THE WORK UNIT, AND THE PRICE MODEL

Because §2.3 rules out seconds, every budget below is denominated in **arithmetic work
units (AWU)**, defined here so they are countable exactly, deterministically, and
identically on any machine.

**Determinant route.** One AWU = one nonzero Hamiltonian element touched.

```
AWU_det = n_det · nnz_per_det(n_orb, nα, nβ) · n_sigma_applications
```

with `nnz_per_det` exactly as §2.5, which reproduces the referee's SiO count to the digit.
`n_sigma_applications` is READ from the solve, never assumed.

**DMRG route.** One AWU = one multiply-add of the two-site effective-Hamiltonian
application, summed over the four stages of `apply_effective_h_mpo`:

```
AWU_mv(bond) = 4·χ_l·χ_r·(D_r·χ_r + D_l·χ_l) + (nnz(W_l) + nnz(W_r))·χ_l·χ_r
AWU_dmrg     = Σ over every local solve of AWU_mv · (Lanczos iterations at that solve)
```

Both are integers a run counts and prints. **The staked price** is that measured wall clock
per AWU lies within a factor of 4 of the calibration reading taken in the same run on the
same box (the factor absorbs the 57% P/E scaling and the contention M-PLACEMENT-LOTTERY
names; it is deliberately loose because it is a sanity bound on the COST MODEL, not a
performance claim). A reading outside it means the model does not describe what ran, and
the reading is VOID rather than scored (M-CHEAPER-THAN-ITS-PRICE runs in both directions —
a result that arrives too cheap is as suspect as one that arrives too dear).

**The budgets, staked:**

| arm | budget | why this number |
|---|---:|---|
| Q-A per exact solve | 1.0e12 AWU | ~50× the `O3H2` pentamer's predicted 2.1e10 × ~90 iterations, so the 62.1% rung fits with room and `O4H1` (9.3e11 per iteration) does not |
| Q-A peak resident | 8 GB | leaves headroom on a 31 GB box already running sibling lanes at load 65 |
| Q-B per (case, χ) cell | 5.0e12 AWU | set so that the largest cell this node intends to drive fits, and a cell an order past it is a caller that has not priced its ask |
| Q-D per config | 4.0e12 AWU | one pentamer plus its subsets at the acceptance set's size |

Exceeding a budget is `SolveExit::IterationCap`-shaped: reported as BUDGET and VOID, never
as a reach and never as a failure of the method (§8).

---

## 6. THE INSTRUMENTS

### 6.1 What is built

| instrument | arm | what it does |
|---|---|---|
| `engine/crates/holon-chem/examples/cluster_ladder.rs` | Q-A, Q-B | the cap ladder on the CLUSTER axis: both routes, both caps, one work-unit currency |
| `engine/crates/q8-mps/tests/canonical_invariants.rs` | Q-C | the canonical brick's gates and their plants |
| `engine/crates/holon-chem/examples/cluster_seam.rs` | Q-D | the seam assembly and its `R_core` ladder |

`de5_audit.rs` is REUSED as the acceptance referee and is not rebuilt: its 24 configs, its
`Sub::void_reason`, its scope constants and its `--audit` pin-only mode are the referee's
own, and this node calls them rather than re-deriving them.

### 6.2 Disclosure fields — every reading carries all of them

`route`, `solver_exit` (the discriminator, PRINTED not merely carried — M-EXIT-DISCRIMINATOR),
`n_det`, `n_orb`, `(nα, nβ)`, `device_class` (= `cpu`, written as a fact about the run),
`AWU`, wall clock (reported, never gating), peak resident set, and for every DMRG reading
additionally: **bond dimension χ, max discarded weight, kept-spectrum floor, canonical
residual, `⟨N̂⟩` and `⟨Ŝz⟩`, sweeps used, and the truncation-vs-time discriminator of §2.4.**

**The arithmetic-regime law gains an axis here.** `TIERS.md` law 6 currently reads
"device class, solver budget, subtraction basis — one law, three axes". A χ-truncated MPS
energy is a different artifact from the exact-in-model energy of the same geometry, and two
tables built at different χ are two artifacts. The axis this node adds is named
**BOND LEDGER**: the pair (χ_max, truncation policy). It is recorded in every DMRG record's
key and manifest. The `TIERS.md` edit is coordinated with node E's in-flight bead-count
axis rather than made unilaterally, and if that coordination has not happened when this
node banks, the axis is stated HERE and the `TIERS.md` line is left for the lead — a fence
this node names rather than a line it races.

### 6.3 Instrument discipline (gate 10a3)

Gate 10a3's grep covers `conformance/` only; these instruments live in `engine/crates/` and
are therefore outside it and comply BY HAND. No session-keyed, lane-keyed or worktree-keyed
path appears in any of them. Paths resolve from the instrument's own location
(`env!("CARGO_MANIFEST_DIR")`), then a named environment override, then a loud refusal by
name. Every launcher takes `--dry-run`, which prints resolved paths, the rung list and the
AWU estimate and exits 0 having created nothing. Refusals carry discriminated exit codes:
2 bad arguments, 3 a path did not resolve, 4 a version mismatch, 5 a digest mismatch,
7 an envelope refusal (budget exceeded). Unknown arguments are REFUSED, never ignored. Long
runs use `setsid`, a `.DONE` marker carrying the exit status, and a `RESUME.md`; the tail
checks the exit code FIRST and a nonzero writes a KILLED marker and touches nothing else.

### 6.4 Record kind (M-CACHE-KIND)

Exact-in-model and variational-with-a-ledger are different KINDS of record. Kind is in the
record key, a completion counter counts the CERTIFIED kind, and a shape mismatch on read
RAISES rather than silently recomputing over the other kind.

---

## 7. THE GATES

Each gate is separable: its kill takes down its own claim and nothing beneath it.

### Arm A — the determinant cap

- **G-A1 — the model predicts the count.** For every case solved, the measured nonzero
  count agrees with §2.5's `nnz_per_det` model EXACTLY (integer equality), as it does for
  SiO. A mismatch kills the price model, not the solve.
  witness: none (a counting identity of the instrument, no Lean object)
- **G-A2 — the price is inside its model.** Measured wall clock per AWU is within a factor
  of 4 of the same run's own calibration cell. Iteration count is reported SEPARATELY and
  is never folded into the price.
  witness: none (measured, no Lean object)
- **G-A3 — the memory is measured, not estimated.** Peak resident set is READ from the
  process for every case, and the `O3H2` case is below the 8 GB budget of §5. An estimate
  substituted for a reading fails this gate.
  witness: none (measured, no Lean object)
- **G-A4 — the raised cap admits what it says and refuses what it says.** If
  `HARD_DETERMINANT_CAP` moves, a case one determinant above the new value is REFUSED and
  a case one below is ADMITTED, both demonstrated. A cap that is raised without both
  demonstrations is not raised.
  witness: none (a property of the admission door, mechanized in the crate's tests)

### Arm B — the MPS cap

- **G-B1 — the reference is never the thing under test.** Every DMRG reading is compared
  against `solve_determinant` on the same geometry, and the comparison is refused if the
  reference's `Solution::route` is anything but `Determinant`.
  witness: none (dE5's correction C-2, inherited)
- **G-B2 — truncation-limited and time-limited are distinguished.** Every rung reports
  whether its residual error is bounded by χ or by the sweep budget, by the stated
  discriminator: a cell is TRUNCATION-limited iff the last-sweep energy change is below
  1e-9 Ha while the max discarded weight is above 1e-10. A rung that cannot say which it is
  is VOID, not BUDGET.
  witness: none (measured, no Lean object)
- **G-B3 — the door is the wall minus one.** The reported `MPS_MAX_ORBITALS` candidate is
  `first_wall − 1` over the ladder, and `best_reached` is printed beside it as a separate
  number. Conflating them fails this gate (M-MAX-OVER-SUCCESSES).
  witness: none (a property of the door, mechanized in `tests/pair.rs`)
- **G-B4 — every axis of the target is exercised.** No reach claim is made at an
  (orbital count, filling, determinant count) triple that was not itself measured. Rungs
  are placed so no claim interpolates across a hole (M-UNTESTED-GAP).
  witness: none (a property of the ladder's design)
- **G-B5 — F-5 is deleted or it is not.** The instrument prints, from the two constants at
  run time, whether any input can select `AutomaticRoute::Mps`. This is the node's own
  bottom and it is answered with a boolean, not a narrative.
  witness: none (derived at run time from the constants)

### Arm C — the canonical brick

- **G-C1 — canonicality is checked, not hoped.** The electronic sweep path
  (`dmrg_sweep`, which today carries NO canonical-defect fields at all) reports the worst
  left and right canonical defects over every two-site update, and a run whose defect
  exceeds 1e-10 REFUSES with a typed refusal rather than returning a state.
  witness: none (measured invariant; the Lean object is owed and named in §12)
- **G-C2 — the particle-number sector is checked.** `|⟨N̂⟩ − (nα+nβ)|` and `|⟨Ŝz⟩ − (nα−nβ)/2|`
  are computed and gated at 1e-6. §2.2 measured 4.1e-5 today, so this gate FIRES on the
  current engine and is expected to; the deliverable is that it fires LOUDLY rather than
  passing silently.
  witness: none (measured invariant)
- **G-C3 — the truncation ledger is live on the production path.**
  `solve_electronic_ground_state` no longer hardwires `RefusalPolicy::Silent`; C9's typed
  refusal is reachable from `fci::solve_mps`, demonstrated by a case that fires it.
  witness: none (a property of the route; C9 in `FENCES.md`)
- **G-C4 — convergence is not stagnation.** A sweep loop that stops because the energy
  stopped moving reports STAGNATED, distinct from CONVERGED, and the two are different
  values of the exit discriminator. A run that stagnates at a discarded weight above the
  ledger is refused, not returned.
  witness: none (LESSONS.md rule 6, mechanized here)
- **G-C5 — parallelism changes no bit.** Every threaded path is bit-identical to the serial
  path on every case in the ladder, `assert_eq!` on the raw `f64` bits, not on a tolerance.
  witness: none (bit-identity, mechanized in the crate's tests)

### Arm D — the seam

- **G-D1 — the vacuity control is exactly zero.** `R_core ≥ diameter` reproduces
  `E_FCI(5)` to the solver's own residual on all 24 configs. Published as the PAIR of §4.3.
  witness: none (identity control)
- **G-D2 — the seam beats its budget.** The worst-config seam defect at the staked
  `R_core` is below 5.0e-5 Ha, i.e. a reduction of at least 1,572× against MBE4's measured
  worst.
  witness: none (measured against dE5's banked 24)
- **G-D3 — the far field is a far field.** The criterion that assigns an atom to the core
  is stated before any reading, applied identically to all 24 configs, and the resulting
  core/far-field split is printed per config (M-HOMOG).
  witness: none (a property of the partition rule)
- **G-D4 — the base rate travels with the verdict.** Every seam verdict names the fraction
  of dE5's 177-candidate landscape it speaks for (M-BASE-RATE-OMITTED).
  witness: none (a reporting obligation)

---

## 8. VOID — what it is, how it is counted, why it is never scored

A cell is VOID, never scored, when any of:

* a solve's `route` is not the route the arm requires (G-B1);
* a solve's exit is not `Converged` or `Trivial`, or its residual exceeds
  `pair::CONVERGED_RESIDUAL`;
* an AWU budget of §5 is exhausted (BUDGET — a limit, not a failure of the method);
* the peak resident set exceeds §5's memory budget;
* measured wall clock per AWU falls outside G-A2's factor of 4 (the cost model does not
  describe what ran);
* a DMRG cell cannot say whether it is truncation- or time-limited (G-B2).

**VOID STRUCTURE is printed** — VOIDs by case, by composition and by reason — so a
correlated refusal pattern is visible rather than averaged away (M-BUDGET-LAUNDER).
Compute expense correlates with cluster size, which is the very axis under test, so a
silently-dropped expensive cell would move every verdict in this freeze.

A verdict for an arm requires **at least 4 non-VOID rungs** for the cap arms and **at least
20 of 24 non-VOID configs** for the seam arm (dE5's own live bar). Below that, the arm is
branch (0): VOID, no verdict, and the VOID structure is the finding.

---

## 9. PLANTS — carrier and sector, each pre-checked to fire

Every plant names its CARRIER and the SECTOR the carrier must be nonzero in, and is shown
firing before any reading of that arm is trusted (M-PLANT-OBS, M-PLANT-SECTOR).

* **P-1 — the dropped far-field term must shift the seam by exactly that term.**
  Carrier: the largest MBE term involving a far-field atom. Sector: the FAR-FIELD sector,
  in which it must be nonzero above 1.0e-6 Ha or the instrument REFUSES to run the plant.
  Fires iff the seam energy moves by that term to 1e-12 Ha. (dE5's P-1, transported.)
* **P-2 — the separated atom must contribute exactly zero.**
  Carrier: one atom translated to 40 bohr. Sector: the GEOMETRY sector, nonzero by tens of
  bohr, while the seam's far-field correction must vanish identically. Fires iff below 1e-8 Ha.
* **P-3 — the exhausted budget must VOID, never score.**
  Carrier: the AWU budget, lowered so a real solve genuinely exhausts it. Sector: the
  SOLVER-EXIT sector, which must read BUDGET where the unplanted run reads Converged.
* **P-4 — the canonicality gate must see a broken canonical form.**
  Carrier: a single site tensor scaled by 1+1e-6 after its SVD, breaking left-canonicality
  without changing the state's direction. Sector: the CANONICAL sector — the environment's
  identity defect — which must be nonzero there while the ENERGY moves by less than the
  gate's own tolerance, so the plant proves the gate sees canonicality and not energy.
  The HF product state is an eigenstate of the one-body part (M-PROBE-EIGENSTATE), so the
  plant is applied at a bond the sweep has already updated, never at the seed.
* **P-5 — the sector gate must see a leaked electron.**
  Carrier: a small admixture of a wrong-`N` component into one site tensor. Sector: the
  PARTICLE-NUMBER sector, in which `⟨N̂⟩` must move by more than 1e-6 while the norm stays 1.
* **P-6 — the parallel path must fail bit-identity when mis-parallelised.**
  Carrier: a deliberate parallelisation over a REDUCTION index instead of an output index.
  Sector: the LAST-BITS sector — the energies must still agree to 1e-12 while the raw bits
  differ, which is precisely the defect class G-C5 exists to catch and which a tolerance
  test would pass.
* **P-7 — the MPO symmetrisation must be caught if it changes the operator.**
  Carrier: one channel's coefficient perturbed by 1e-9 in the rebuilt MPO. Sector: the
  OPERATOR sector — `Mpo::dense()` must differ, and the ground-state energy must move.

A plant that does not fire is suspected BEFORE the gate is trusted: three of seven planted
mutations in a sibling campaign stayed silent for numerical reasons (LESSONS.md rule 9).

---

## 10. THE BRANCHES — every answer's meaning, written before any of it is seen

Evaluated in this precedence, per arm, independently.

### Arm A (the determinant cap)

* **(A-0) VOID** — fewer than 4 non-VOID rungs. No verdict about the cap.
* **(A-a) THE CAP MOVES.** `O3H2` (5,664,400 determinants) solves inside §5's AWU and
  memory budgets, G-A1..G-A4 green. `HARD_DETERMINANT_CAP` is raised to a measured value
  with the measurement committed beside it, and **62.1% of dE5's landscape becomes exactly
  reachable**. Pre-committed follow-up: the seam's exact leg is offered at `O3H2`, and the
  node reports what that costs per config rather than promising it.
* **(A-b) THE CAP MOVES PARTWAY.** `O3H1` (1,019,200) fits and `O3H2` does not — by AWU, by
  memory, or by G-A2. The cap moves to the measured wall, the refusal stands above it WITH
  THE NUMBER THAT SET IT, and the finding is that the wall is at the measured place rather
  than at the policy's round 2,000,000.
* **(A-c) THE CAP DOES NOT MOVE.** The measured price at `O3H1` already exceeds the budget.
  Reported as a measured fence with node F as the exit, and the price is published so the
  next lane inherits a number and not an opinion.

### Arm B (the MPS cap)

* **(B-0) VOID** — as §8.
* **(B-a) F-5 IS DELETED.** Some real input selects `AutomaticRoute::Mps`: the DMRG route
  reaches a stated stake on a space past `MPS_ROUTE_THRESHOLD` = 50,000, at a priced AWU,
  at every axis of the target. Both constants move together and `tests/pair.rs`'s
  unreachability assertion is replaced by its opposite.
* **(B-b) THE CAP MOVES, F-5 STANDS.** `MPS_MAX_ORBITALS` rises against the current MPO
  (with §4.5's improvements measured in) but the reachable determinant window still lies
  below `MPS_ROUTE_THRESHOLD`. Reported in exactly those words: the cap moved, the band is
  still empty, F-5 is not discharged. The improvement is banked; the finding is not
  inflated.
* **(B-c) THE CAP CANNOT BE MOVED.** The DMRG route does not reach any useful stake at the
  seam's sizes inside §5's budget, with the truncation/time discriminator saying WHICH
  (G-B2). **This is a real outcome, not a failure.** The node converts to a measured fence
  with the node-F exit and a named successor: quantum-number-blocked tensors, whose
  absence is measured here rather than asserted. The fence names its build (§4.5's
  improvements are IN the measurement, so the fence sits at the true frontier), and the
  χ required for the stake is published so the successor inherits a target.

### Arm C (the canonical brick)

* **(C-a) THE BRICK LANDS.** G-C1..G-C5 green with all plants firing. Canonicality, the
  sector and the ledger are checked invariants of the electronic path, and the
  convergence-on-stagnation defect is closed at its source.
* **(C-b) THE BRICK LANDS AND CONVICTS.** As (C-a), and G-C2 fires on real cases, which
  §2.2 already predicts. The finding is published as loudly as the gate: readings taken
  through `fci::solve_mps` before this node carried an unchecked sector leak, and the
  affected records are named.
* **(C-c) THE BRICK DOES NOT LAND.** A gate cannot be made to fire, or a plant stays
  silent for a reason that is not understood. Nothing downstream is trusted, and the arm
  reports the unfired plant rather than the gates that passed around it.

### Arm D (the seam)

* **(D-0) VOID** — fewer than 20 live configs.
* **(D-a) THE SEAM HOLDS.** G-D1..G-D4 green, worst defect below 5.0e-5 Ha. The seam is the
  answer dE5's branch (b) asked for, at the scope both share.
* **(D-b) THE SEAM HOLDS PARTWAY.** The defect is below MBE4's by a large factor but above
  the staked budget. Published with the factor and the budget side by side, and NOT rounded
  into a pass. The `R_core` at which it would hold is reported if it exists inside the
  configs' own diameter.
* **(D-c) THE SEAM DOES NOT HOLD.** The defect is not materially better than MBE4's. Then
  the core/far-field partition is the wrong idea at this scope, and the node says so — that
  is a finding about the physics and it is worth more than a seam that was going to be
  built anyway.

**Pre-committed follow-ups** (branches, not rescues): if (A-a) and (B-c) both fire — the
outcome §1 and §2 make most likely — the node's banked answer is that **the seam's exact
leg is the determinant route with a measured cap, and the DMRG route is a measured fence**,
and the GANTT row is rewritten to say that rather than to keep DMRG in its title. That
sentence is written here, before the data, so that outcome cannot be presented as a
surprise or as a defeat.

---

## 11. CRYSTAL-TIER REFEREE INHERITANCE — stated, per the node's own charter

The GANTT row's fourth deliverable. Stated here in the freeze so it cannot be assembled
after the fact to fit a result.

1. **The citation.** SCHWINGER-3's S1(a) bank —
   `conformance/crystal/SCHWINGER3_RESULTS.md`, run 2026-08-31, `M_V/g` = 0.553116 against
   the continuum analytic `1/√π` = 0.564190 ± 0.05, |Δ| = 0.0111 (2.0% relative, 22% of the
   band's half-width), 18/18 points, zero VOIDs — is this node's prerequisite bank for the
   proposition that **this repository's exact-first DMRG machinery reproduces real physics
   under frozen gates**. It is CITED, not re-run and not assumed silently.
2. **What is inherited: the MACHINERY licence only.** That a DMRG sweep on a chain, with a
   declared χ ladder and a demonstrated-slack bond budget, can reproduce an independently
   known continuum observable. SCHWINGER-3's own χ-premise passed with two orders of margin
   (staked `|M(χ=64) − M(χ=40)| ≤ 1e-3`, worst measured 6e-6), which is the precise sense in
   which the bond ledger was shown not to be the limiting error there.
3. **The fence, restated verbatim and inherited exactly.** *"this is QED₂ — one spatial
   dimension, one flavour, the model's own continuum limit as the referee. It licenses the
   MACHINERY (exact-first DMRG on gauge-coupled chains reproducing continuum physics), not
   any claim about 3+1D."* **A molecular-orbital MPO on 13 to 21 orbitals in three
   dimensions is NOT covered by that licence**, and no energy this node computes is
   warranted by it. This sentence appears beside every DMRG verdict this node publishes.
4. **What is NOT inherited: the two-sided gauge.** SCHWINGER-3 could inherit SCHWINGER-2's
   ED-plant certification because it ran the same instrument on the same Hamiltonian family.
   The electronic MPO is a different instrument on a different family, so it inherits no
   plants: §9's P-4..P-7 are derived for THIS instrument and pre-checked to fire
   (M-PLANT-OBS), and §7's G-B1 supplies this node's own two-sided reference — the exact
   determinant solve on the identical geometry.
5. **The boundary this states, and does not cross** (M-FOREIGN-DOMAIN-CORROBORATION). A
   passing result in QED₂ is not evidence about `O2H3`. What crosses the boundary is a
   statement about CODE — that `q8-mps`'s sweep, truncation and Lanczos have been exercised
   against an external ground truth once — and the code exercised there is
   `dmrg_sweep`/`split_two_site`/`lanczos`, which IS the code this node drives. What does
   not cross is `Mpo::from_electronic_integrals`, which SCHWINGER-3 never called. That
   asymmetry is the whole content of the inheritance and it is stated in both directions.
6. **The sibling obligation.** `WORKBENCH_FSD.md:533` records as OWED "the C2
   crystal-inheritance staking (DMRG vs FCI referee)". This node's G-B1 IS that comparison
   at this node's scope; whether it discharges the C2 row is the tower lane's call and this
   node does not claim it.

---

## 12. WHAT THIS NODE DOES NOT CLAIM

* It does not claim anything about three-dimensional water, a molecule-based MBE, a basis
  larger than STO-3G, or any composition outside `OxHy`, `x+y = 5`.
* It does not claim a DMRG energy is exact in model. Every one is variational and
  truncation-limited and carries its ledger.
* It does not claim the seam is the right partition for any scope but the one measured.
* It does not claim a Lean witness for any gate here. Arm C's invariants are the natural
  candidates — a canonical-form predicate and its preservation under a two-site update —
  and that brick is **OWED and named**, not quietly implied. No gate above is advertised as
  machine-checked, and `witness: none` appears on every one of them for that reason.
* It does not claim the crystal referee licenses any molecular result (§11.3).
* It does not claim the improvements of §4.5 are sufficient. If the cap does not move, the
  fence names them as the build it sits behind, and names what would move it next.

---

## The law this freeze lives under

Pre-register method and the meaning of every possible answer before any result is seen
(rule 1). Stake kills first and make them separable (rule 2). A residual is never support;
support comes only from confirmed advance predictions (rule 6). Report the fired kill as
plainly as the survival, and keep the dead claim in the record, marked dead (rule 7).

A fence is a bug under repair. This node exists to delete F-5; if it cannot, the fence it
leaves must name its build, its price and its exit.
