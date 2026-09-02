# B2 — THE LONG-RANGE PAIR SUBSYSTEM, FREEZE

*Frozen 2026-09-01, before `longrange.rs` existed, before one line of the subsystem was
written, and before any number in §5, §6 or §7 had been read off this engine. The one
measurement this freeze depends on is B1b's, which is already banked in
`B1B_RESULTS.md`; every number this freeze STAKES is either a committed constant with its
file and line, a figure printed in an already-committed log, or a threshold chosen here.
The git history is the check: this document lands in its own commit, ahead of the
subsystem's.*

**misfits:** M-BARE-CHARGE, M-HOMOG, M-VOLUME-SCALE, M-KINEMATIC-NONLOCAL, M-NULL-MISSTAKE,
M-PLANT-OBS, M-PLANT-SECTOR, M-VACUOUS-SUCCESS, M-MAX-OVER-SUCCESSES, M-EXIT-DISCRIMINATOR,
M-SORTS-NOT-SEPARATES, M-UNTESTED-GAP, M-ONE-MODEL-DELTA, M-FOREIGN-DOMAIN-CORROBORATION,
M-BUDGET-LAUNDER, M-STALE-INSTRUMENT, M-PROVENANCE-OVERREACH, M-PLACEMENT-LOTTERY,
M-IDLE-CALIBRATED-TIMEOUT, M-CHEAPER-THAN-ITS-PRICE, M-DEVICE-CLASS, M-LOOP-BLIND,
M-CACHE-KIND.

| id | the contact |
|---|---|
| M-BARE-CHARGE | the id's lesson is that a bare charge is not yet a physical object. Its transfer here is exact and it decides the method: **this engine's force law contains no charge at all** — `Z` is a species label for a bank lookup (`holon-render/src/lib.rs:914`) and the only Coulomb term in the tree sits within the solver (`holon-chem/src/md.rs:588`), never in `Sim::compute_forces`. A charged scene is therefore not a scene B2 can price, and R1 refuses it by name |
| M-HOMOG | the census box is 34.6 × 20.8 bohr with 12 atoms and no bulk. **Every density-based long-range estimate — the standard isotropic tail integral, `g(r) → 1` past the cutoff — is invalid here**, which is why the far residual is bounded by a pair count and a monotone kernel value (§5.3) and never by a density |
| M-VOLUME-SCALE | the far sum is over pairs and grows as N²; the image shells are a lattice whose count must be shown adequate at each box size, not carried from one. N-scaling is a MEASURED curve (G13), never an assumption, and it is the item B1b left owed |
| M-KINEMATIC-NONLOCAL | a locality stake must separate propagation from constraint. This scene carries no constraint correlating separated regions — the atoms are free, the boundary is walls or images — so the separation is trivial and is stated rather than assumed |
| M-NULL-MISSTAKE | **this is the registry form of the brief's "one gate per conservation law".** Each conservation gate is staked on the quantity ITS OWN law constrains: G4 on `ledger() − l0`, G5 on `P − P₀ − J_ext`, G6 on `L − L₀`. G5 passing is not evidence about G6, and P3 is the plant built to demonstrate exactly that |
| M-PLANT-OBS | each of the seven plants is re-derived for THIS subsystem and pre-checked to fire; the instrument REFUSES to score a plant whose carrier reads 0.0 (§7). P4 is in this freeze in its second form because its obvious form is provably invisible — see §7 |
| M-PLANT-SECTOR | every plant names the sector its carrier must be nonzero in, and why it is nonzero there (§7) |
| M-VACUOUS-SUCCESS | every gate asserts its WORK COUNT and prints it pass or fail (G12): steps integrated, frames scored, far pairs enumerated, `R_s` crossings observed. A conservation gate on a scene whose atoms never moved is the id's own founding case |
| M-MAX-OVER-SUCCESSES | every verdict that is a maximum is a maximum over the WHOLE admitted set; a refused frame VOIDs its class and never drops out of the max |
| M-EXIT-DISCRIMINATOR | B1b banked that the O–O curve exits `IterationCap` at 5000 iterations with residual 4.809e-6. **Every tail parameter fitted from that curve inherits it**, and R5 refuses to emit one without `solver_exit`, `solver_budget_iterations` and `uncertainty_hartree` beside it |
| M-SORTS-NOT-SEPARATES | G3's exponent bands could RANK the curves rather than separate them. The freeze therefore stakes what it means for NO curve to land in the adopting band, and that outcome is a fence rather than a retune |
| M-UNTESTED-GAP | G13 is printed and NOT gated on a value: no prior record carries an N-scaling curve for this engine's pair sector, and a freeze cannot gate a number it would have to invent. It VOIDs only on a non-monotone ordering, which convicts the measurement rather than the engine |
| M-ONE-MODEL-DELTA | the far kernel is ONE chosen model of the tail. A gate it passes earns "better than the exponential extrapolation by this much", never "correct" |
| M-FOREIGN-DOMAIN-CORROBORATION | **the reason §2 argues from this engine's source and not from MD practice.** That Ewald is standard for water says nothing about a force law with no charge in it; a result in molecular dynamics is not a result about `Sim::compute_forces` |
| M-BUDGET-LAUNDER | a VOID arm is printed as VOID with its condition named and is never scored; B1b's fired G1b stays fired and is reported beside every B2 outcome |
| M-STALE-INSTRUMENT | one commit produces every number; the instrument's commit and every artifact path are pinned as tree paths, and no citation is keyed to this session's scratch directory |
| M-PROVENANCE-OVERREACH | the reused manifest refusal names the file it hashed and infers nothing about which run produced it |
| M-PLACEMENT-LOTTERY | G13's timings are CPU time, pinned to an E-core, reported with n and spread. Wall clock is not the unit — that is the mistake B1's price gate made and B1b corrected |
| M-IDLE-CALIBRATED-TIMEOUT | the launch header records loadavg at start AND end, the core class, and the clock as a fraction of advertised, so a later ratio can check the regime before believing itself |
| M-CHEAPER-THAN-ITS-PRICE | B2 generates no new curves and therefore needs no price gate of its own; it CONSUMES B1b's, and inherits B1b's W1 solver certificate unchanged as the evidence that they were solved |
| M-DEVICE-CLASS | the bit-identity gate G9 is stated WITHIN one device class. Nothing here claims bit-agreement across devices |
| M-LOOP-BLIND | the id's shape is a marginal blind to the distinction it is asked about, and G1 is a marginal — a sum over pair separations. Two channels of opposite sign could cancel inside it, so G1 reports the two channels SEPARATELY and their signed sum, never the sum alone |
| M-CACHE-KIND | the far sector caches box-derived quantities. The kind of box is IN the cache key (dimensions, boundary, shell count) and a mismatch on read RAISES; G9 and P1 are the mechanization |

---

## 0. WHY THIS EXISTS, AND WHAT IT MAY NOT ASSUME

B1b banked, on the record: at `c* = 15.0` bohr the mixed class discards up to
**1.150526e-5 Ha** per frame, **fails** the incurred-drift gate `0.10·D_s` on **3 of 8
seeds**, reaching **2.496×** the criterion on the worst, and the boundary runs through the
seed set rather than past it. The mechanism it named is structural: **the engine's cell
decomposition radius sits 5 bohr inside its own O–O curve's support** (`r_max = 20.0` bohr
against `c* = 15.0`), so what a truncation at `c*` would throw away is real tabulated
interaction and not tail. `E_band(c*)` is nonzero in 144 of 400 published stride rows for
the mixed class and in 0 of 400 for hydrogen.

That fired B2. It did not design B2, and this freeze's first job is to say what B1b's
verdict does NOT license.

**It does not license Ewald.** B1b's classes are neutral H and O. Every curve in them
decays; none is `r⁻¹`. §2 argues the method choice from this engine's source rather than
from the practice of a neighbouring field, because a method is warranted by the kernel it
sums and not by the fluid it is famous for.

**It does not license one fix.** B1b measured a SUM. The discard at `c*` has two channels
with different causes and different cures, and B1b never separated them. Separating them
is G1, it is staked before it is measured, and every branch in §8 turns on it.

---

## 1. WHAT THE ENGINE ACTUALLY DOES — read from committed source, before building

### 1.1 There is no electrostatic term in the force law

`Sim::compute_forces` (`holon-render/src/sim.rs:2387`) accumulates exactly: the tabulated
pair sector, the tabulated three-body sector, the four-body `(O,H,H,H)` sector, the walls,
the uniform field, and the user's spring. **No Coulomb term, no point charges, no
multipoles.** `Sim` stores nuclear charge only through `Atom`'s species, resolved once per
force pass into a bank slot (`Sim::refresh_slots`); `lib.rs:914` is the accessor and it
feeds a table lookup. The only `q_a q_b / r` in the tree is
`holon-chem/src/md.rs:588`, within the electronic-structure solver that GENERATES the
curves.

**Consequence, and it is the whole method argument: the quantity Ewald exists to sum is
not present.** Ewald summation, PME and Wolf-style damped sums are all machinery for the
**conditionally convergent** `Σ 1/r` lattice sum — the reciprocal-space split, the
neutralising background, the surface term and the tinfoil convention exist because that sum
has no absolutely convergent value and its answer depends on summation order. A kernel with
no `r⁻¹` component has none of those problems, and building the apparatus that solves them
would be building an instrument whose defining difficulty this class does not have.

### 1.2 What the tail actually is, in source

Past its last knot every pair table is an exponential matched in value and log-slope at
that knot (`holon-render/src/table.rs:324 build_extrapolations`, evaluated at
`table.rs:383`):

```
u(r) = hi_a · exp(−hi_b · (r − r_max)),   hi_a = u(r_max),  hi_b = −u'(r_max)/u(r_max)
```

This is an EXTRAPOLATION, not a computed value. The curves are FCI in a declared minimal
basis (`holon-chem/src/pair.rs`; B1b's W1 records `n_basis = 10`, `n_det = 2025` for O–O),
so the underlying method carries correlation and its true asymptote is a power law, not an
exponential. **Whether this table has REACHED that asymptote inside its own support is not
known and is G3.**

### 1.3 Where `c*` came from, and why it is a bookkeeping fact

`Sim::list_cutoff` (`sim.rs:1554`) is `max(three_body_cutoff, four_body_cutoff, pair r_cut)`.
At the commit that produced the parked artifacts the three-body radius is `15.0`
(`holon_chem::water::R_HI`), the four-body sector is off, and no pair cutoff is declared —
so `list_cutoff() = 15.0` and it is set entirely by a THREE-BODY table. The O–O PAIR curve
reaches `20.0`.

`Sim::derive_pair_cutoff` (`sim.rs:1584`) bisects OUTWARD from each curve's `r_max`, so a
cutoff the engine DERIVES is never inside a curve's support. B1b's `c*` was the cell-list
radius, which is: **the engine has a radius at which it is already local, and that radius
does not respect its own pair curves.** Making that unrepresentable is R3.

### 1.4 The periodic box refuses the radius this scene needs

`Sim::pbc_ok` (`sim.rs:1496`) requires `list_cutoff ≤ 0.5 · min_edge`. The census box's
short edge is 20.8 bohr, so a periodic version of this scene may declare at most **10.4**
bohr — below `c* = 15.0`, and half the O–O curve's support. **A periodic box of this size
cannot legally carry this scene's own pair interaction under a single-image convention.**
That is the second half of B1b's "no safe radius", and unlike the first half it is a
statement a bigger cutoff can never fix. It is what forces images into the design.

### 1.5 The two ledger columns a new channel must post to

`Sim::energy()` sums `e_kin + e_pair + e_three + e_four + e_wall + e_spring + e_grav`;
`ledger() = energy() − w_ext` is the invariant. The four-body sector's pattern is the one to
follow: its own energy row (`e_four`), its own virial contribution accumulated where the
slopes already are, and no `w_ext` posting because it is conservative. `Sim::scale_box`
(`barostat.rs:535`) rescales the box and every position affinely and posts
`after − before` to BOTH `w_ext` and `work.hand`, with `work_columns_ok` checking they never
part. **A far sector that caches anything derived from the box breaks that**, which is G9.

---

## 2. THE METHOD, AND THE ARGUMENT FOR IT

### 2.1 The three candidates, priced against this engine's kernel and scenes

| candidate | what it solves | does this engine have that problem? |
|---|---|---|
| **Ewald / PME** | the conditionally convergent `r⁻¹` lattice sum; splits it into a screened real-space part and a reciprocal-space part | **No.** §1.1: no charge in the force law. PME additionally needs a charge grid and an FFT whose crossover against direct summation sits in the thousands of atoms; the census scenes are 12 |
| **Wolf / damped shifted force** | the same `r⁻¹` sum, cheaper, by neutralising each cutoff sphere | **No**, and for the same reason. Its charge-neutralisation step has nothing to neutralise |
| **Split kernel with an absolutely convergent far part** | a far tail with `Σ_images \|u\|` finite, so real-space image summation is exact to a computable truncation error | **Yes.** This is the shape of the problem B1b measured |

**The staked choice is the third, and B2 does not build Ewald.** The refusal in R1 is what
keeps that from being a silent decision: a scene whose kernel is `r⁻p` with `p ≤ d` is
REFUSED with the exponent, the dimension and the exit named, rather than summed by a method
whose convergence argument does not cover it.

### 2.2 The subsystem

```
u(r)  =  u_near(r)                      r ≤ R_s     the table, on the neighbour list
      +  u_far(r)                       r > R_s     the tail model, summed to a budget
```

with, in a wrapping box, `u_far` summed over the minimum image AND over image shells
`1..=m`.

* **`R_s ≥ max over loaded slots of r_max`, enforced.** Channel S (§3) becomes exactly zero
  by construction rather than small. This is R3, and it is the direct cure for §1.3.
* **`u_far` is a DECLARED tail model**, its exponent measured from the table's own knots by
  G3 and never assumed, matched in value at `R_s`, and carrying the generating solve's exit
  status and budget (R5).
* **The far sum runs to a declared per-pair energy budget**, deriving `R_f` by the same
  bisection `Sim::derive_pair_cutoff` already uses, so the engine has one derivation rather
  than two that can disagree.
* **The far residual past `R_f` is bounded WITHOUT a density** (§5.3), because M-HOMOG
  forbids the density argument on a 12-atom box.

### 2.3 What makes the image sum legitimate, stated as the condition it is

For a monotone kernel `|u(r)| ≤ A r⁻p` on a `d`-dimensional lattice of images, the shell at
radius `~ nL` holds `O(n^(d−1))` images each contributing `O((nL)^(−p))`, so the shell sums
converge iff `p > d`. **`p > d` is the licence for this whole design and it is CHECKED, not
assumed**: G3 measures `p`, R1 refuses `p ≤ d`, and G10 measures the shell-to-shell
difference and reports it as the truncation uncertainty rather than discarding it.

`d` here is the SCENE's dimension (`Sim::dims`), not three: the census scenes are
`Dims::Two`, so the licence condition is `p > 2` and the ionic case `p = 1` fails it in 2D
exactly as it fails in 3D.

---

## 3. THE TWO CHANNELS — the split this freeze is built around

At `c*`, the pair energy a truncation discards decomposes into:

| channel | pairs | what it is | what fixes it |
|---|---|---|---|
| **S — sub-support** | `c* < r ≤ r_max` | REAL tabulated interaction, exactly evaluable, thrown away because the list radius is set by a different sector | a list radius that respects every loaded pair curve's support (`R_s`, R3). Cost: cell volume grows as `(r_max/c*)^d` |
| **T — beyond-support** | `r > r_max` | the table's EXPONENTIAL extrapolation standing in for an unknown true tail | a tail model, and it is the only channel where a long-range METHOD is the answer |

B1b reported both quantities and gated their sum. Its own numbers hint at the split without
settling it: mixed-class `max|E_switch(c*)| = 1.150526e-5` against `max E_tail = 2.239049e-6`
— but those are maxima over *different frames*, and a max of a part is not a part of a max.
**G1 measures the split per frame, and every branch in §8 turns on the answer.**

This is stated now because the two channels have different consequences for what B2 is.
If S dominates, B1b's headline is a radius-bookkeeping defect and the long-range METHOD
content of B2 is the periodic case (§1.4) and the tail's honesty, not the mixed-class
number. If T dominates, the extrapolation form is the finding. **Both are bankable results
and the freeze refuses to prefer one.**

---

## 4. SCOPE — what B2 warrants and what it does not

Written before any result, and it travels with every verdict sentence.

**Warranted, if the gates pass:** the PAIR sector of scenes composed of species whose
curves are loaded, all nuclei neutral, in `Walls`, `Open` or `Periodic` boxes, at the scene
sizes G13 actually measures.

**NOT warranted, each for a stated reason:**

1. **Ionic `r⁻¹` scenes are neither shown nor excluded.** Node C's charged fragments enter
   through the SOLVER seam (`holon-chem/src/ions.rs`), not as point charges in the force
   law, so B2 has measured nothing about them. R1 refuses them by name rather than letting
   a green B2 gate be read as coverage. **The exit is named: Ewald or PME, owned by whoever
   ships a force law with charge in it.**
2. **The many-body sectors are untouched.** They return exact zeros outside their domains
   (`sim.rs:1481`) and discard nothing by truncation. witness: `DependsWithinExact`
3. **The tail's `C_p` is a MODEL quantity.** It is fitted from a minimal-basis FCI curve,
   which underestimates dispersion badly; FENCES.md M10 / `ION_STAKING.md` I-5's
   diffuse-basis rung is the named exit, and it is UNOWNED. B2 does not claim a physical
   dispersion coefficient and R4 forbids emitting one as a scalar when G3 lands fenced.
4. **The O–O curve is budget-limited.** `IterationCap` at 5000 iterations, worst residual
   4.809e-6 Ha, four orders above `CONVERGED_RESIDUAL = 1e-9`. Every tail parameter fitted
   from it inherits that, and R5 refuses to emit one without it.
5. **N-scaling is measured at the sizes G13 runs**, and says nothing above them.
6. **Angular momentum is gated only where the box permits it** (G6): `Open`, no field, no
   walls, no hand. Walls torque and a periodic box is not isotropic, so `L` is not conserved
   there and a gate claiming otherwise would be measuring nothing.

---

## 5. THE INSTRUMENT

**One binary, one commit, every number.** `engine/crates/holon-render/examples/b2_longrange.rs`,
with the subsystem in `engine/crates/holon-render/src/longrange.rs`. Its commit is printed
in its own launch header together with loadavg at start and end, the core class, and the
clock as a fraction of advertised.

### 5.1 The B1b frames are reused unchanged

Same trajectories, same `census_traj_manifest.sha256` refusal, same admitted seeds, same
stride, same `B_s` and `D_s` read from the committed arm logs. **No frame is re-chosen and
no threshold B1b staked is moved.** B2 is measured against the bill B1b wrote.

### 5.2 The tail model

`u_far(r) = −C_p · r^(−p)` for `r > R_s`, with `p` from G3 and `C_p` fixed by matching
`|u|` at `R_s` — **one free constant, determined, not fitted.** The force is the analytic
derivative of that same expression, so G8 is a check on the arithmetic and not on two
independent implementations.

The kernel is CENTRAL and PAIRWISE-ADDITIVE by construction: `F_ij = −du_far/dr · r̂_ij`
applied as `+F` to `i` and `−F` to `j` in one expression. G5 and G6 are what check that the
construction survived contact with the code, and P2 and P3 are what check that G5 and G6
can see it fail.

### 5.3 The far residual, bounded without a density

M-HOMOG forbids the standard isotropic tail integral here. The bound B2 uses instead
assumes only that `|u_far|` is monotone decreasing past `R_f`:

```
|Σ_{r > R_f} u_far(r)|  ≤  (N(N−1)/2) · |u_far(R_f)|
```

Crude, rigorous, and free of any assumption about how the atoms are arranged. `R_f` is
derived so this bound is under the declared budget. The tighter packing-based bound is
computed and REPORTED beside it, so a successor can gate the tighter one; it is not gated
here, because no prior record carries it (M-UNTESTED-GAP).

---

## 6. THE GATES

Each conservation gate is staked on the quantity its own law constrains, and each fails
independently of the others. That independence is not asserted — P2, P3 and P7 are the
plants built to demonstrate it.

- **G1 — THE CHANNEL SPLIT.** On every admitted frame of every admitted seed, the discard at `c* = 15.0` is decomposed into channel S (`15.0 < r ≤ r_max`) and channel T (`r > r_max`) and BOTH are printed with their signed sum, never the sum alone. Verdict band: S/T ≥ 10 → S-dominant; T/S ≥ 10 → T-dominant; otherwise MIXED. Taken at the frame maximising `|E_switch(c*)|` and separately at the frame maximising each channel, all three printed. witness: `DependsWithinUpTo`
- **G2 — CHANNEL S CLOSES EXACTLY.** With `R_s ≥ max_slot r_max`, the near sector's energy on every admitted frame equals the complete-sum reference to within 1e-12 relative, and the count of pairs in `(c*, r_max]` excluded from the near list is EXACT 0. witness: `DependsWithinExact`
- **G3 — THE TAIL EXPONENT, MEASURED NOT ASSUMED.** Per loaded curve, the local log-log slope `p_fit = −d ln|u| / d ln r` over the last 10% of knots, and the exponential's equivalent local index `hi_b · r_max`, both printed with their fit residual. Adopting band `p_fit ∈ [5.0, 7.0]` AND `hi_b · r_max ≤ 3.0 · p_fit`; fenced band otherwise. Every curve's band is printed whether or not it is used. witness: `none (a measured property of a committed table; M-SORTS-NOT-SEPARATES is its warrant, since the bands may rank the curves rather than separate them)`
- **G4 — ENERGY: THE LEDGER CLOSES UNDER THE NEW TERM.** With the far sector on, over at least 20000 integration steps on an admitted configuration: `drift_peak ≤ drift_bound()` AND `work_columns_ok()`. The measured-over-bound ratio is printed so the margin is visible rather than absorbed. witness: `none (a ledger-closure gate on this engine's integrator; the symplectic bound is `Sim::drift_bound`'s own derivation, not this term's)`
- **G5 — MOMENTUM, INDEPENDENTLY.** Over the same run, `momentum_residual() ≤ momentum_bound()`. Additionally, on an isolated far pair, `F_i + F_j` is EXACT 0.0 in each component. Staked on `P − P₀ − J_ext`, the quantity the momentum law constrains, and on nothing else. witness: `none (a conservation gate on the engine's own momentum ledger; M-NULL-MISSTAKE is its warrant)`
- **G6 — ANGULAR MOMENTUM, INDEPENDENTLY.** New to this engine. On `Boundary::Open`, `g_vec == 0`, no hand, no thermostat, no barostat, far sector ON: `|L(t) − L(0)| ≤ 8 · steps · eps · L_scale` over at least 20000 steps. Refused with the reason printed on `Walls` (walls torque) and `Periodic` (the box is not isotropic) rather than run and passed vacuously. witness: `none (a conservation gate the engine did not previously carry; M-NULL-MISSTAKE is its warrant)`
- **G7 — THE VIRIAL POSTS.** A LEDGER-COMPLETENESS gate and NOT a conservation law, labelled so in its own output. With the far sector on, the pressure built from `w_virial` matches a central finite-difference `−dE/dV` taken through `Sim::scale_box` at 3 step sizes on a frozen configuration, to 1e-6 relative, with the Richardson estimate of the differencing error printed beside it. witness: `none (a completeness gate on the virial accumulator; the brief's channel-must-post-to-the-ledger rule is its warrant)`
- **G8 — THE FORCE IS MINUS THE GRADIENT.** For the far term alone, central finite differences of `E_far` against the analytic force agree to 1e-7 relative on at least 100 configurations drawn from admitted frames. This is the precondition that makes G4 a measurement of integration error rather than of an inconsistency. witness: `none (a gradient-consistency check on one term; it has no theorem, it is arithmetic against arithmetic)`
- **G9 — NOTHING BOX-DERIVED GOES STALE.** After `Sim::scale_box(f)` for `f` in {0.90, 1.10}, the far sector's energy, force on every atom, and virial are BIT-IDENTICAL to a subsystem constructed fresh at the scaled box. Stated within one device class. witness: `none (a cache-invalidation gate; M-CACHE-KIND and the barostat seam are its warrants)`
- **G10 — IMAGE CONVERGENCE, WITH ITS UNCERTAINTY DISCLOSED.** In a wrapping box, `|E_far(m+1) − E_far(m)|` is below the declared per-pair budget at the reported shell count `m`, and that difference is emitted as `uncertainty_hartree` beside the energy in every manifest row. A shell count reaching the staked cap of 8 without meeting the budget is R2, not a pass. witness: `dependsWithinUpTo_mono_radius`
- **G11 — THE REFUSAL FIRES.** A scene declaring a nonzero point charge, and a kernel with measured `p ≤ d`, are each constructed and each REFUSED by name, with the exponent, the dimension and the named exit printed. A refusal that does not print all three is a failure of this gate. witness: `none (a refusal-observability gate; R1 is the behaviour it checks)`
- **G12 — WORK COUNT.** Printed for every arm including refused and VOID ones: integration steps (floor 20000 where a conservation gate is claimed), frames scored (floor 50 per class), far pairs enumerated (floor 1), and `R_s` crossings observed during the G4 run (floor 1, since P4 is vacuous without one). witness: `none (an anti-vacuity assertion; M-VACUOUS-SUCCESS is its warrant)`
- **G13 — N-SCALING, MEASURED AND NOT GATED ON A VALUE.** Far-sector CPU time against N at fixed number density, at least 5 sizes spanning at least 8× in N, at least 3 repetitions each, pinned to an E-core, reported as a fitted exponent WITH its spread. **No pass/fail on the exponent**: no prior record carries this curve and a freeze may not gate a number it would have to invent. It VOIDs on a non-monotone ordering in N, which convicts the measurement and not the engine. witness: `none (a measured cost curve; M-UNTESTED-GAP and M-PLACEMENT-LOTTERY are its warrants)`
- **G14 — THE B1b BILL IS PAID, OR IT IS NOT.** On B1b's 8 admitted mixed-class seeds and its frames, the residual discard under the new subsystem is compared to `0.10 · D_s` seed by seed, and the recomputed ratio is printed beside B1b's per-seed ratios including its worst of 2.496. The bill is paid only if all 8 seeds are under 1.0. A seed still over is a FIRED gate, reported as plainly as a pass and kept in the record marked fired. witness: `DependsWithinUpTo`

### 6.1 Refusal semantics — what the subsystem refuses, and how loudly

- **R1 — THE `r⁻¹` REFUSAL.** Any scene declaring a nonzero point charge, or any kernel whose measured exponent satisfies `p ≤ d` for the scene's own `d` (2 or 3), is refused. The message prints `p`, `d`, and the exit by name: Ewald or PME, on a force law that has charge in it. This is a DEBT with a fix path, never architecture (the fence law). witness: `none (a refusal, gated by G11)`
- **R2 — THE IMAGE-BUDGET REFUSAL.** A wrapping box whose shell-to-shell difference has not met the declared per-pair budget by the staked cap of 8 shells is refused with the achieved difference and the cap printed. Silently accepting the 8th shell would be an undeclared budget replacing a declared one. witness: `none (a refusal, gated by G10)`
- **R3 — THE SUB-SUPPORT REFUSAL.** `R_s` below any loaded curve's `r_max` is refused with both radii printed. This makes B1b's measured defect unrepresentable rather than merely fixed, and it is the one refusal that would have fired on the configuration B1b audited: `R_s = 15.0` against `r_max = 20.0`. witness: `none (a refusal; §1.3 is its warrant)`
- **R4 — THE FENCED-TAIL REFUSAL.** When G3 lands any needed curve in the fenced band, the far energy is emitted ONLY as a bracket — the exponential as one end, the power law as the other — and a caller asking for a scalar is refused with both ends printed. A bracket whose ends differ by more than 1 order of magnitude additionally carries that factor in its own field. witness: `none (a refusal; M-ONE-MODEL-DELTA and scope item 3 are its warrants)`
- **R5 — THE DISCLOSURE REFUSAL.** A tail parameter emitted into any manifest without `solver_exit`, `solver_budget_iterations` and `uncertainty_hartree` beside it is refused. A capped residual is not monotone in effort, so a number without its budget is not a number. The O–O curve's own values — `IterationCap`, 5000, 4.809e-6 — are the founding case. witness: `none (a refusal; the engine-wide disclosure law and M-EXIT-DISCRIMINATOR are its warrants)`

---

## 7. PLANTS

Seven, one per gate family. **Each is pre-checked to fire before its gate is trusted, and
the instrument REFUSES to score a plant whose carrier reads 0.0.** Each names the sector
its carrier must be nonzero in, and why it is nonzero there.

**P1 — THE STALE LATTICE, against G9.** Cache the image offsets at construction and skip
recomputation in `scale_box`. *Carrier:* `E_far(scaled box, stale offsets) − E_far(scaled
box, fresh offsets)`. *Sector:* the periodic image sector. Nonzero in it because scaling by
`f ≠ 1` moves every image offset by `(f−1)·L`, changing every image separation, and the
kernel is strictly monotone so no two image contributions cancel exactly. Pre-check at
`f = 0.90`.

**P2 — THE ONE-SIDED FAR FORCE, against G5.** Apply `+F` to `i` and nothing to `j` for far
pairs. *Carrier:* `Σ_i F_far,i`. *Sector:* the far pair force sector. Nonzero in it because
a single admitted far pair contributes `F ≠ 0` at the staked separation, and the sum has no
other contributor to cancel against. **This plant must leave G6 in whatever state it was:
it is P3 that separates them.**

**P3 — THE NON-CENTRAL FAR FORCE, against G6, and the demonstration that G5 does not vouch
for G6.** Rotate each far pair force by 1e-3 rad in the scene plane, keeping it equal and
opposite. *Carrier:* `Σ_i r_i × F_far,i`. *Sector:* the angular sector. Nonzero in it
because a force with a component perpendicular to `r̂_ij` exerts a couple that does not
cancel between the partners, while the equal-and-opposite construction leaves the linear sum
exactly zero. **The staked observation is that G6 FIRES while G5 STAYS GREEN.** A plant that
fires both has not demonstrated the independence and is reported as failing its own purpose.

**P4 — THE ZERO-POINT STEP, against G4 — and it is in its SECOND form because the first is
provably invisible.** The obvious plant — add a constant to `u_far` — shifts `l0` and
produces no drift at all, so `Sim::rebase` absorbs it and the energy gate cannot see it.
That is M-PLANT-OBS exactly, and it is written down here rather than discovered afterwards.
The observable form applies the constant `1e-6` Ha only for `r > R_s`, making a step
discontinuity at `R_s` that the force does not carry. *Carrier:* the energy jump at each
`R_s` crossing. *Sector:* the energy ledger. Nonzero in it because a pair crossing `R_s`
changes `E` by the constant while `F` is unchanged, which is precisely an unexplained loss.
**Vacuous unless a pair actually crosses**, so G12 counts the crossings and the plant is
refused at zero crossings.

**P5 — THE GRADIENT MISMATCH, against G8.** Scale the far force by 1.001 while leaving the
energy alone. *Carrier:* the 1e-3 relative discrepancy between the analytic force and the
finite difference. *Sector:* the far force sector. Nonzero in it wherever `du_far/dr ≠ 0`,
which is everywhere past `R_s` for a monotone kernel.

**P6 — THE TRUNCATED FAR SUM, against G1 and G14.** Sum the far sector only to `R_s + 0.1`
bohr instead of to `R_f`. *Carrier:* the omitted band's energy. *Sector:* the far pair
sector. Nonzero in it because B1b measured the O–O switched carrier at 16.0 bohr as
−2.028788e-6 Ha, which lies within the omitted band, so the plant is pre-checked to fire from
an already-committed number rather than from an expectation.

**P7 — THE OMITTED VIRIAL, against G7.** Post `u_far` to the energy row and not to
`w_virial`. *Carrier:* `Σ_far r · du_far/dr`. *Sector:* the virial. Nonzero in it for the
same reason as P5. **The staked observation is that G7 FIRES while G4 and G5 STAY GREEN** —
a channel can be perfectly conservative and still be missing from the pressure.

---

## 8. THE BRANCHES — every answer's meaning, written before any of them

* **BRANCH (a) — S-DOMINANT.** G1 reads S/T ≥ 10 at the worst frame. → **B1b's headline is a
  RADIUS-BOOKKEEPING defect**: the cell decomposition was built at a three-body radius while
  the pair curve reached further, and R3 makes that unrepresentable. The tail model is then
  carrying a minority of the discard, and B2's long-range METHOD content is the periodic
  case of §1.4 — where no legal single-image radius exists at all — rather than the mixed
  class's number. Reported that way, in those words, and **B2 does not claim to have solved a
  problem that was a radius**.
* **BRANCH (b) — T-DOMINANT.** G1 reads T/S ≥ 10. → **The extrapolation form is the finding.**
  The tail model does the work, G3's band governs how strongly it may be stated, and R4's
  bracket is the honest emission if G3 lands fenced.
* **BRANCH (c) — MIXED.** Neither ratio reaches 10. → **Both channels are load-bearing and
  neither fix alone discharges B1b.** Both are reported with their sizes; no single sentence
  is allowed to stand in for the pair.
* **BRANCH (d) — THE BILL IS PAID.** G14 under 1.0 on all 8 seeds with every conservation
  gate green and every plant fired. → **Cutoff-locality is available for oxygen-bearing
  scenes through the far sector**, at the cost G13 measured, with the scope of §4 attached to
  the sentence every time it is quoted.
* **BRANCH (e) — THE BILL IS NOT PAID.** Any seed at or above 1.0 under the new subsystem. →
  **A FIRED GATE.** The failing seeds are named, the residual is sized, and the gate is not
  retuned. This is a result: it would say the far sector as designed does not cover what
  B1b measured, and it names the next instrument rather than moving the criterion.
* **BRANCH (f) — VOID.** Any of V1–V6. → No verdict in either direction for that arm, the
  condition named, and the VOID structure at the head of the results.

**Pre-committed follow-up, designed in rather than rescued.** If branch (e) fires, the same
instrument re-runs with `R_f` derived at a budget 10× tighter, with NOTHING else changed —
no new estimator, no new denominator, no re-chosen frames — and the budget at which the bill
first closes is reported. If it never closes inside a factor of 1000, the far sector is
reported as insufficient and the finding is that the discard is not a tail phenomenon.

---

## 9. VOID CONDITIONS

- **V1 — MANIFEST MISMATCH.** Refusals leaving a class under 50 scorable frames.
- **V2 — PLANT DID NOT FIRE.** Any plant whose carrier reads 0.0, or which fires the wrong
  gate. A gate whose plant did not fire is not a gate and its arm is VOID.
- **V3 — GRADIENT INCONSISTENCY.** G8 fails. Every conservation reading downstream is then
  measuring an inconsistency rather than the integrator, and none of them is scored.
- **V4 — REFUSAL DID NOT FIRE.** G11 fails, so the scope fence in §4 is unenforced and no
  claim about what B2 does not warrant can be trusted.
- **V5 — WORK FLOOR.** G12 fails on any floor.
- **V6 — NON-MONOTONE COST.** G13's ordering in N is not monotone, which convicts the timing
  measurement; the cost curve alone is VOID and the correctness gates are unaffected.

A VOID arm is never scored, never inferred from a sibling, and never reported with a number
in its verdict column. **B1b's fired G1b stays fired** and is reported beside every B2
outcome whatever that outcome is.

---

## 10. WHAT THIS MEASUREMENT CANNOT SAY

1. **Nothing about ionic scenes.** §4 item 1. R1 is the mechanization of that silence.
2. **Nothing about whether the many-body DOMAINS are wide enough.** Different question,
   different instrument. witness: `DependsWithinExact`
3. **Nothing about a physical dispersion coefficient.** §4 item 3: the constant is a model
   quantity from a minimal basis, and the diffuse-basis exit is unowned.
4. **Nothing above the scene sizes G13 measures**, and nothing about a device class other
   than the one it ran on.
5. **Nothing about angular momentum in a walled or periodic box**, where it is not conserved
   and G6 refuses rather than reads.
6. **The far model is ONE model of the tail.** Passing G14 earns "better than the
   exponential extrapolation by this much", never "correct".
