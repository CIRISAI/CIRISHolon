# OMEGA-RATCHET-1 — the two rent faces: an exact bridge on the lift, three obstructions off it, and the maintained-holonomy result re-read

*Draft. OMEGA_LADDER.md rung 7, staked attack OMEGA-RATCHET-1. Nothing here is
committed to any repository; CIRISOntology was read only. Every claim below carries
one of three labels — **DERIVED** (proved here in exact rational arithmetic, and
machine-checked by `verify.py` in this directory), **MEASURED** (a number from an
existing campaign record, quoted with its source), **RE-READ** (an interpretation of a
MEASURED result through a DERIVED object; the weakest tier, and the one to attack).*

---

## 0. The verdict, in four lines

1. **A lawful bridge exists, and it is a bridge between different quantities, not an
   identity.** On the two-state rented registry — the unique minimal model on which
   both faces are simultaneously defined — the partition rent is the magnitude dose
   scaled by twice the retention shortfall: `W(v) = 2·δ·W*(γ,δ)`. **DERIVED.**
2. **The two prices are anti-correlated over the whole useful range.** Buying better
   retention raises the magnitude price toward its maximum and drives the partition rent
   to zero. They cross at exactly one point (`δ = 1/2`) and their derivatives have
   opposite signs everywhere above it. There is no universal exchange rate. **DERIVED.**
3. **Three obstructions bound how far the bridge carries**, and the first is fatal on the
   magnitude face's own native carrier: a deterministic magnitude recursion has a point
   mass for its only invariant measure, so its partition rent is identically zero for
   *every* view. The bridge exists only on the stochastic lift, where the magnitude *is*
   a measure. **DERIVED.**
4. **The maintained-holonomy "repair must know the design" finding is NOT the partition
   rent theorem wearing measured clothes.** It is a different phenomenon living one
   layer below Theorem 1 — in the policy's information class, which `W(v)` is defined to
   minimise away. Worse for the freeze: `W(v)`'s minimiser is *exactly* the design-blind
   policy class the campaign measured to fail. **RE-READ**, with four independent checks
   against the campaign record and one named disanalogy that does not pass.

---

## 1. The objects, transcribed from source (not from summary)

**MAGNITUDE face** — `~/CIRISHolon/lean/CIRISHolon/Object.lean`:

```
rentStep lam q s0 s = (1 - q) * lam * s + q * s0
Ginf lam q          = q / ((1 - lam) + q * lam)
Wstar γ δ           = (1 - δ) * γ / (γ + δ * (1 - γ))
rent_closed_form    : rentOrbit lam q s0 n = rentFix + ((1-q)*lam)^n * (s0 - rentFix)
Ginf_at_Wstar       : Ginf (1 - γ) (Wstar γ δ) = 1 - δ     (0<γ, 0<δ<1)
```

`Wstar` is a **dose**: a per-step fraction of the deposit target, valued in `(0,1)`.
`γ = 1 − lam` is the decay gap; `δ` is the retention shortfall.

**PARTITION face** — `~/CIRISHolon/conformance/omega/CROSSFACE1_PREREG.md`, Theorem 1:

```
W(v) = 1 - Σ_i max_j P_ij ,   P_ij = μ_i · K(j|i),  μ K-invariant
```

`W` is **displaced mass**: `min over holding policies of Pr[s'' ≠ s']`, valued in `[0,1)`.

Both are dimensionless per-step fractions. **They are fractions of different things**, and
the freeze already says so in its own words: *"They are different numbers about different
objects, and nothing here is inherited from there."* This note does not contradict that
fence. It locates the exact relation that nonetheless holds, and — more usefully — the
exact places where none can.

### The K4 / Landauer fence, restated because it is the one that already fired

Neither face is an energy and neither is a bit count. `W` counts displaced probability
mass per step; `W*` counts deposited fraction of a target magnitude per step. **No
Landauer normalisation is asserted anywhere in this note, and none may be read into
it.** The predecessor programme's K4 kill fired at 3–5 dex on exactly that step
(`DE_LEDGER_MODEL.md`; memory `de-ledger-precedent-is-bits`): pricing a bit count at a
temperature that is not the count's own gave an answer 3–5 orders of magnitude short of
`ρ_DE` against the Egan–Lineweaver 2010 budget. Every quantity below is a probability or
a fraction. Converting either face to joules is out of scope, and the conversion that
was tried is dead.

---

## 2. The bridge model, and why it is forced

The magnitude face needs a scalar under an affine decay-and-deposit recursion. The
partition face needs a finite state space, a kernel, an **invariant** measure, and a view.
The smallest object carrying both is the **two-state rented registry** `R(γ, q)`:

- `S = {0, 1}`: the entry has decayed away / the entry stands.
- One step: with probability `q` the maintainer **refreshes** the entry to `1`;
  otherwise **decay** removes it with probability `γ` (and nothing is created from
  nothing, matching `Core/Creation.lean`'s `percell_no_creation` and
  `Core/Valve.lean`'s `valve_from_nothing`).
- The view under test is `v_id`, the finest one: the two blocks are the two states.

The kernel is

```
K(1|0) = q                   K(0|0) = 1 - q
K(0|1) = (1-q)·γ             K(1|1) = 1 - (1-q)·γ
```

### R1 — three identifications, all exact (**DERIVED**)

| | |
|---|---|
| **(a)** | `rentOrbit lam q 1 n` **is** `Pr[the entry stands at step n]`, term by term, with `lam = 1−γ`. The magnitude recursion is not *like* the chain's occupancy dynamics; it is that dynamics. |
| **(b)** | `Ginf lam q` **is** the chain's stationary occupancy `μ₁`. The magnitude face's retention is the partition face's block measure. |
| **(c)** | `λ(v_id) = (1−q)·lam`. The partition face's **view mixing modulus** — the freeze's new carrying variable, `‖M − Π‖` on `1^⊥` in the `μ`-inner product — **is** the base of the geometric transient in `rent_closed_form`. |

(c) is the load-bearing one, and it is exact rather than approximate because **every
two-state chain with a stationary measure is `λ·I + (1−λ)·Π`** — verified entrywise. The
two faces are therefore not merely commensurable in units; they are **two prices computed
from one shared variable**, and that variable already had two names.

---

## 3. The exchange rate (**DERIVED**)

Writing `G = Ginf` for the retention and `δ = 1 − G` for the shortfall:

```
magnitude price     q  = G · (1 − λ)
partition rent      W  = 2 · G · (1 − G) · (1 − λ)          [regime R1, see below]
                       = 2 · (1 − G) · q
                       = 2 · δ · W*(γ, δ)                    [at q = W*(γ,δ)]
```

**Both faces are the same relaxation `(1−λ)` multiplied by a different moment of the same
view.** The magnitude face charges for the **mean** of the maintained view; the partition
face charges for **twice its variance**. That is the whole content of the bridge, and it
explains every qualitative feature below.

The mechanical reading: the magnitude face bills every unit of deposit. The partition face
bills only the deposit that actually **crosses the view's block boundary** — a refresh
landing on an entry that already stands moves no probability between blocks and is
partition-free. The fraction of deposits landing on an empty entry is `1 − G = δ`, and
the factor 2 is the balancing outflow, since stationarity forces `P₀₁ = P₁₀` exactly.

### The anti-correlation, and the single crossing

`γ = 1/4`, exact rationals:

| `δ` (shortfall) | `W*` magnitude price | `W` partition rent |
|---|---|---|
| 1/100 | 99/103 = 0.9612 | 99/5150 = 0.0192 |
| 1/20 | 19/23 = 0.8261 | 19/230 = 0.0826 |
| 1/10 | 9/13 = 0.6923 | 9/65 = 0.1385 |
| 1/5 | 1/2 = 0.5000 | 1/5 = 0.2000 |
| 1/3 | 1/3 = 0.3333 | 2/9 = 0.2222 |
| **1/2** | **1/5 = 0.2000** | **1/5 = 0.2000** |
| 2/3 | 1/9 = 0.1111 | 4/27 = 0.1481 |
| 99/100 | 1/397 = 0.0025 | 99/19850 = 0.0050 |

`W/W* = 2δ`, so the two agree **only** at `δ = 1/2` and disagree without bound in ratio as
`δ → 0`. Over the entire useful range (`δ < 1/2`, i.e. more than half the entry retained)
the partition face **undercharges** relative to the magnitude face, and the gap widens
exactly as maintenance improves. As `δ → 0` the magnitude price rises to its ceiling
(`W* → 1`: refresh every step) while the partition rent falls to zero, because a
perfectly maintained entry is a **closed** view and closed views are free (Theorem 1's
`W = 0` case). At the other end an entirely abandoned entry is also deterministic, and
also free. **Only a partially maintained entry pays partition rent.**

### The maximum

`W(δ) = 2δ(1−δ)γ / (γ + δ(1−γ))` is maximised at

```
δ* = q* = √γ / (1 + √γ)        W_max = 2γ / (1 + √γ)²
```

so the most expensive operating point in the partition currency is exactly the one where
**the dose equals the shortfall**. Verified at five rational squares (`γ = 1/4 → δ*=q*=1/3,
W_max = 2/9`; `γ = 9/16 → 3/7, 18/49`; …) against a 400-point grid.

### The regime map — the closed form is not global

`W(v_id)` is exactly `min(2Φ, δ, G, 1−2Φ)` with `Φ = G(1−G)(1−λ)`, always; the four
arguments are the four deterministic updates `F` of Theorem 1. The trichotomy (exact on
an 11×11 rational grid; the fourth cell is provably empty since `γ ≤ 1`):

| regime | condition | `W(v_id)` | reading |
|---|---|---|---|
| **R1** | `q ≤ 1/2` and `γ(1−q) ≤ 1/2` | `2(1−G)q` | the maintained-registry regime; `F` = identity |
| **R2** | `q > 1/2` | `δ` | heavy dosing; cheapest `F` declares everything ON, and pays the residual OFF mass |
| **R3** | `γ(1−q) > 1/2` | `G` | heavy decay; cheapest `F` declares everything OFF, pays the residual ON mass |

**The headline `W = 2δW*` is an R1 statement.** Outside R1 the bridge still exists but
takes a different form, and nothing in it is a rescaling of `W*`.

---

## 4. A generalisation of the freeze's Theorem 3, and a cross-check against its own record (**DERIVED**)

The `2G(1−G)` in the bridge is the Gini–Simpson index `1 − Σμ_i²` at `N = 2`. That
suggests a form for `N` blocks, and it is true under a stated fence:

> **GINI RENT.** Let `M = λ·I + (1−λ)·Π` on `N` blocks with stationary `μ`. Then
> exactly, with no further hypothesis,
> ```
> W(v) = 1 − Σ_i μ_i · max( λ + (1−λ)μ_i , (1−λ)μ_max )
> ```
> and when `λ ≥ (1−λ)(μ_max − μ_min)` this collapses to
> ```
> W(v) = (1 − Σ_i μ_i²) · (1 − λ)          "rent = Gini index × (1 − retention)"
> ```

Checked exact on 130 `(μ, λ)` cells across `N = 2…5`, against both halves of Theorem 1
independently, and against Theorem 2's inequality on every cell. It contains:

- **CROSS-FACE-1 Theorem 3** as the uniform-`μ` case: `(1 − 1/N)(1 − λ)`. Verified for
  `N = 2,3,4,5` and all `λ ∈ {0, 1/12, …, 1}`.
- **CROSS-FACE-1 Theorem 6's `λ = 0` form** as the amnesia case: `1 − μ_max`.

**The fence is load-bearing, and the freeze's own measured table proves it.** D4's
`v_classA` has class sizes `(1,1,2,2,2)/8` and `λ = 0`, which is *outside* the fence. The
general form returns **3/4** — CROSS-FACE-1's measured value, reproduced. The naive Gini
form would return **25/32**. Anyone extending Theorem 3 off the uniform line by pattern
match gets that wrong number; the exact statement above does not.

This is the note's one clean offer back to the partition face: **an exact rent formula
valid off the uniform-`μ` line, containing both of the freeze's exact theorems as special
cases, and a candidate Lean brick** — the freeze's `G1` witness line records that Theorem
2 is proved by hand and not mechanized.

---

## 5. Three obstructions (**DERIVED**)

### O1 — deterministic-magnitude vacuity. *The bridge does not exist on the magnitude face's own carrier.*

Take `rentStep` at face value: a deterministic affine map `T` on `ℝ` with contraction
factor `(1−q)·lam`, `|(1−q)lam| < 1`, and unique fixed point `rentFix`. Every orbit
converges to it, so for any bounded continuous `f` and any `T`-invariant probability `μ`,
`∫f dμ = ∫f∘Tⁿ dμ → f(rentFix)`; hence **`μ = δ_{rentFix}`, the only invariant measure**.
For a face `(S, K, μ, v)` with `μ` a point mass, `P` is a single unit cell, `Σ_i max_j P_ij
= 1`, and

> **`W(v) = 0` for every view `v` whatsoever**, while `W*(γ,δ) > 0` for every `δ < 1`.

On the magnitude face's native carrier the two faces are **incommensurable, and the
obstruction is named**: the partition price of the magnitude model is identically zero
because the model has no probability to displace. This is the *second horn* of the
freeze's own §0 obstruction. `rentStep` is injective, so `alpha_eq_one_of_injective`
already voids the micro-mixing route on it; O1 says the view-relative route dies too, for
a different reason. The bridge of §3 exists **only on the stochastic lift**, where the
maintained magnitude *is* an occupancy measure rather than a coordinate — and that lift is
canonical exactly when the magnitude is a normalised retention, which `Ginf` is.

**Kill for O1:** exhibit a `T`-invariant probability measure on the deterministic
`rentStep` that is not `δ_{rentFix}`, or a nondegenerate face built from `rentStep`
without a stochastic lift.

### O2 — `W` prices displaced mass, not displacement distance.

Theorem 1's optimal repair *"move[s] it to a fixed point of block `F(v(s))`"* at a cost of
one unit of displaced mass, **however far the move is**. A repair that must drag the state
across the whole block pays exactly what a nudge pays. Consequence for this task: the
maintained-holonomy campaign's central quantitative residual — the P4 closed form
overpredicting the plateau by up to **9.8 %**, one-signed at every `q`, attributed to
misalignment between the deposit and the decayed 64×64 operator, and *worsening to 13.7 %*
when the non-geometricity was removed (`HOLONOMY_RENT_RESULTS.md` §5) — **has no image in
the partition face at all.** It is a distance effect in a currency that has no distance.
Any attempt to "explain" the 9.8 % through `W` is a category error.

### O3 — `W(v)` is blind to the policy's information class, and its minimiser is the blind policy.

`W(v)` is a minimum over **all** kernels `R(s''|s,s')`. It therefore cannot express a
constraint on *which* repairs are available, and — sharper — on the fiber model of §6 the
argmin is attained by a repair that is **equivariant under the fiber symmetry**, i.e. by
exactly the design-blind class. Measured exactly there: `W(v_view) = γ`, attained by the
fiber-inert policy; the design-pinning policy that holds the same view closed pays
`γ + p(1−γ)` in its own steady state, strictly more.

> **The quantity CROSS-FACE-1 minimises is minimised by the class of repair that loses
> the design.**

This is not a defect in the freeze — it is a correct consequence of its definitions, and
it is precisely the gap §7 exploits.

---

## 6. The fiber model, and the exact split (**DERIVED**)

`S = V × Φ` with `|V| = |Φ| = 2`; the motion is two independent symmetric binary channels
(view flips at `γ`, fiber flips at `p`), so uniform `μ` is invariant and this is a legal
CROSS-FACE-1 face. Two nested views: `v_view` (the view coordinate, 2 blocks) and
`v_full` (the identity, 4 blocks).

```
W(v_view) = γ                       (= Theorem 3 at N=2 with λ = 1−2γ, checked)
W(v_full) = γ + p(1−γ)
surcharge = W(v_full) − W(v_view) = p(1−γ)      exactly, every cell
```

**The surcharge for holding the design is exactly the fiber's own rent.** Now put the two
repair *forms* of `HOLONOMY_RENT_PREREG.md` (lines 147, 155) on it, both depositing with
probability `q` as `Rep_q(A) = (1−q)A + qD`:

- **design-BLIND** (`D` = the state's own current fiber; R-POL's `polar(A)` names a
  *manifold*, not a point):
  ```
  alignment(n) = ( 1 + (1 − 2p(1−q))ⁿ ) / 2   →   1/|Φ|  at every q < 1
  ```
- **design-KNOWING** (`D` = a named point of the fiber):
  ```
  alignment(n) = f* + ((1−q)(1−2p))ⁿ (1 − f*),   f* = (q + (1−q)p) / (1 − (1−q)(1−2p))
  ```
  with `f* > 1/|Φ|` for every `q > 0`, `f*(0) = 1/|Φ|`, `f*(1) = 1`, monotone in `q`.

Both laws exact as rationals over 120 steps × a `(γ, p, q)` grid.

**The dose `q` is a rate knob for the design-blind arm and a level knob for the
design-knowing arm.** For the blind repair `q` appears only inside the decay constant; the
**limit is `1/|Φ|` at every budget short of total replacement**. Budget cannot buy what the
policy has no coordinate to address. For the knowing repair `q` sets a plateau strictly
above the floor.

---

## 7. RE-READING the maintained-holonomy result — the ruthless part

### 7.1 What was measured

`HOLONOMY_RENT_RESULTS.md` (2026-07-27, prereg at `3ae9c9b` before the instrument existed;
the pre-registered `q=0` void **fired** at 2.01e−3 and the §4 table was recomputed):

| | |
|---|---|
| **size** | R-POL holds gain at **0.434945**, constant to six decimals from `R ≈ 200` to `R = 4001`, against an unmaintained loop at **2.521e−66** — sixty-five orders |
| **direction, knowing** | R-DES fidelity **0.990884** at `q=ε`, flat to six decimals over the same 4000 rungs, slope `d log f/d log R = +0.0000`; **0.998785** at `q=0.3`, also flat |
| **direction, blind** | R-POL fidelity **0.987914 → 0.061457**, power law, slopes **−1.5540 / −1.1084 / −0.2758 / −0.0270** at `q = ε, 0.1, 0.3, 0.7` |
| **the floor** | C-RAND *measured* the chance floor at **0.0155 ≈ 1/d = 0.015625**, correcting the prereg's assumed `1/√d = 0.125` by 8× |
| **the control** | R-DES and R-POL run the identical pipeline, so the split is physics, not arithmetic |

### 7.2 Is it CROSS-FACE-1 Theorem 1 wearing measured clothes? **No.** Three reasons, any one sufficient.

1. **Cost is not the ordering variable.** If this were the rent theorem, the arm with the
   larger budget would hold the finer view. It does not: R-DES holds direction **flat at
   `q = ε`, the smallest dose on the grid**, while R-POL fails to hold it **at `q = 0.7`**,
   twenty times larger. Theorem 7 ("a policy budgeted below `W` cannot hold the view
   closed") is not violated and is not engaged — it is simply not the operative statement.
2. **The obstruction is feasibility, not price.** By O3, `W(v)` is a min over *all*
   policies. There is no budget at which R-POL holds the direction view, and there is no
   budget at which R-DES fails to. That is a 0/∞ dichotomy on the policy class, orthogonal
   to Theorem 1's continuous cost.
3. **The minimiser points the wrong way.** By O3 again, `W(v_size)`'s optimal policy *is*
   the fiber-blind class. The partition face, asked to price the size view, hands back the
   repair that destroys the design and calls it optimal. It cannot be the theory of a
   result whose content is that this repair fails.

### 7.3 Is it `Core/Maintenance.lean`'s `unpaid_decays` on the fiber? **Also no** — and the campaign's own control says why.

The tempting re-reading is "the fiber is an unmaintained coordinate, so `unpaid_decays`
applies." It does not: `unpaid_decays` gives **convergence to zero**. The measured
fiber limit is **`1/d`, not `0`** — C-RAND measured `0.0155 ≈ 1/64`. The limit object is
the fiber's **uniform measure**, which is `Core/FrameEntropy.lean`'s `fiber`, not
`Core/Maintenance.lean`'s decaying entry. Relaxation-to-maximum-entropy and decay-to-zero
are different limits with different fixed points; conflating them is exactly the
`shared-lemma-one-witness` error (two quantities that agree on the witnesses in hand,
separated by asking what their definitions actually say).

### 7.4 What it *is*: the fiber floor. **RE-READ**, with four checks.

> **The maintained-holonomy split is a statement about the fixed-point set of the repair,
> not about its budget. A repair whose fixed-point set is a manifold holds the manifold
> and lets the state diffuse inside it, to the fiber's uniform measure. A repair whose
> fixed-point set is a point holds the point. The dose sets the rate in the first case and
> the level in the second.**

The prereg itself states both fixed-point sets, and they are the whole mechanism:
*"For R-POL the fixed-point set is exactly the isometries … For R-DES the fixed point [is
the design]"* (`HOLONOMY_RENT_PREREG.md` line 167f). §6's model is that sentence made
finite and exact.

| # | §6 predicts | campaign measured | |
|---|---|---|---|
| 1 | blind arm's limit is the fiber's **uniform**, not zero | C-RAND floor **0.0155 ≈ 1/d = 0.015625**, prereg's assumed `1/√d` corrected 8× | ✓ |
| 2 | `q` slows the blind arm's approach but never stops it (rate `< 1` for all `q < 1`) | slopes −1.554, −1.108, −0.276, −0.027 — monotone in `q`, **never zero** | ✓ |
| 3 | knowing arm plateaus **above** the floor at every `q`, flat forever | 0.990884 flat at `q=ε`; 0.998785 flat at `q=0.3` | ✓ |
| 4 | that plateau **rises with `q`** (`f*` monotone, `f*(1) = 1`) | 0.990884 (`q=ε`) → 0.998785 (`q=0.3`) | ✓ |

### 7.5 The disanalogy that does **not** pass, stated because rule 6 requires it

§6's blind-arm rate is `1 − 2p(1−q)`, so its collapse exponent should scale as `(1−q)`.
It does not:

| `q` | measured `|d log f/d log R|` | slope / (1−q) |
|---|---|---|
| 0.0345 | 1.5540 | 1.6095 |
| 0.1 | 1.1084 | 1.2316 |
| 0.3 | 0.2758 | 0.3940 |
| 0.7 | 0.0270 | 0.0900 |

The column is not flat — it falls by 18×. **The finite model reproduces the *shape* of the
split (floor, monotonicity, never-zero, rising plateau) and not the *rate law*.** Under
rule 6 this is a residual, and a residual is never support. The four checks in §7.4 are
consistency checks against a record that already existed; **none of them is a confirmed
advance prediction**, and this note claims none. §8's R-C exists to make one.

### 7.6 The sharpest consequence, and the one worth attacking

The campaign compared the arms at **matched `q`** through an identical pipeline — and `q` is
a magnitude-face quantity. (The phrase *"at the same mean effort"* in
`HOLONOMY_RENT_RESULTS.md` §8 belongs to the *schedule* comparison, 0.560 vs 0.435, not to
the arms; it is not quoted here as if it did.) In the partition currency the two arms were
never matched at all: by §6 the design-knowing
repair holds a strictly finer view and pays a strictly larger rent, `W(v_full) − W(v_view)
= p(1−γ)` exactly. **"Design-knowing beats design-blind at the same effort" is
currency-relative.** It is true in the magnitude currency, and in the partition currency
the comparison is between two different views at two different prices — which is not a
comparison of policies at all. What survives, and is currency-independent, is the
qualitative claim: *no budget in either currency buys the design for a repair whose
fixed-point set does not contain it.*

---

## 8. What a follow-up would stake

Four, separable, each with its own kill. None is run here.

- **R-A (Lean brick, cheap, no compute).** Mechanize GINI RENT: `W(v) = 1 − Σ_i μ_i
  max(λ+(1−λ)μ_i, (1−λ)μ_max)` for `M = λI + (1−λ)Π`, with the corollary under
  `λ ≥ (1−λ)(μ_max − μ_min)`. It contains the freeze's Theorem 3, reproduces its measured
  D4 `v_classA = 3/4`, and discharges part of `G1`'s standing "not yet mechanized" line.
  **Kill:** any exact-rational counterexample; the audit bar is sorry-free.

- **R-B (the missing object).** Define **informed rent** `W(v | 𝒫)` — the minimum work over
  a *restricted* policy class — and prove the 0/∞ dichotomy: for `𝒫` equivariant under a
  group `Γ` and `v` not `Γ`-invariant, no policy in `𝒫` holds `v` closed at any budget,
  while `W(v) = W(v | all)` stays finite. `W(v | all)` is the freeze's `W`; O3 is the
  statement that the freeze only ever defined the easy end. **Staked forward number** on
  §6's model: `W(v_full) − W(v_view) = p(1−γ)` exactly, for any `(γ, p)` a reader picks.
  **Kill:** a `Γ`-equivariant policy that holds a non-`Γ`-invariant view closed.

- **R-C (a real advance prediction — the only one that can earn rule-6 support).** §7.4's
  four checks are retrospective. Stake, *before running*, on the frozen `holonomy_rent.py`
  at held-out `q`: (i) the R-DES plateau is monotone increasing in `q` with a pre-computed
  band from the `f*` form fitted to the two existing points, (ii) R-POL's fidelity at
  `R = 4001` for a new `q` between 0.3 and 0.7, (iii) the R-POL floor is `1/d` and not
  `1/√d` on a **second** `d`. Item (iii) is the strongest: it moves the fiber-floor reading
  off the single `d = 64` it was found on. **Kill:** any band missed; and (iii) is
  separable — it can fire without touching (i)/(ii).

- **R-D (the disanalogy, staked to fail honestly).** §7.5's rate law is wrong. Either
  derive the correct blind-arm exponent for a *continuous* fiber — R-POL's fixed-point set
  is the whole isometry manifold, a positive-dimensional fiber against §6's single bit, and
  the dimension count must be done rather than guessed — and stake it forward; or record
  the finite model as **shape-only** and stop quoting its rate.
  **Kill:** the derived exponent misses the four existing slopes outside a pre-declared
  band — in which case the fiber-floor reading survives only as a qualitative shape claim.

**Not staked, and named so it is not later smuggled in:** the sawtooth campaign
(`SAWTOOTH_FORWARD_RESULTS.md`: 30/30 planted readings in pre-staked bands, P-LINEAR
**1.9847** against a staked 2.000, and a column-rule control that **fired** — a degenerate
code gives **0.26×** the predicted tooth, the reciprocal of the 3.8× recorded in
`CLAUDE.md`) measures a whole-only share in **nats**. Neither rent face prices nats.
The sawtooth is in a third currency and this bridge does not reach it.

---

## 9. Scope, in one block

- Finite exact models only. All arithmetic rational; `verify.py` in this directory
  re-derives every number and prints PASS/FAIL per claim. Floats appear only in display
  columns.
- `W` is displaced mass; `W*` is a deposit fraction. **Neither is an energy, neither is a
  bit count, and no Landauer normalisation is asserted.** The predecessor's K4 fired at
  3–5 dex on exactly that step and is not repeated.
- §3's headline `W = 2δW*` is an **R1-regime** statement (`q ≤ 1/2` and `γ(1−q) ≤ 1/2`).
  §4's Gini collapse holds under `λ ≥ (1−λ)(μ_max − μ_min)`; the general closed form is
  unconditional for `M = λI + (1−λ)Π`, and that kernel class is itself a hypothesis, not a
  fact about arbitrary faces.
- §7 is **RE-READ**, the weakest tier. Four retrospective consistency checks and one named
  failed check; **no confirmed advance prediction is claimed**, and §7.5's disanalysis is
  reported as plainly as §7.4's agreements.
- Nothing here is a claim about any wild process, and nothing here modifies CIRISOntology's
  stance. CIRISOntology was read only; no commit was made in any repository.

---

## R-A / R-B executed (2026-08-28, GiniRent.lean, sorry-free, builds green)

**R-A landed, narrowly quoted:** GINI RENT is mechanized — on the kernel
class M = λI + (1−λ)Π the exact rent is `(1 − Σμ_i²)·(1 − λ)` under its
stated fence (rent = Gini × (1 − retention)), with CROSS-FACE-1's Theorem
3 recovered as the uniform corollary and the freeze's measured D4 3/4
reproduced from class sizes — AND used as a refuter: the pattern-match
extension off the uniform line gives 25/32, wrong. What this does NOT
discharge, stated in the file: Theorem 2's witness line (the general
inequality), Theorems 4 and 6, and Theorem 1's (≤) policy construction.

**R-B: THE STAKE IS DEAD, by its own author's counterexample.** The
informed-rent 0/∞ dichotomy as staked is FALSE —
`staked_R_B_dichotomy_is_false` exhibits the memory-carrying restore
policy R(s,s') = s: Γ-equivariant, holds the FINEST view closed, finite
cost. Equivariance of the policy is not the obstruction. What survives,
proved: a Γ-equivariant kernel with Γ transitive is DOUBLY STOCHASTIC
(`equivariant_uniform_invariant`) — the actual mechanism of the measured
1/d fiber floor: a repair that names no point of the fiber cannot prefer
one. The design-knowing separator is exact (`knowing_stationary`, f* > 1/2
for q > 0), and the staked forward number is proved:
**`fiber_surcharge`: W(v_full) − W(v_view) = p(1−γ) exactly** — the
surcharge for holding the design is the fiber's own rent.

The informed-rent object survives with its definition corrected: the
restriction that binds is not equivariance of the policy but MEMORYLESSNESS
plus fiber-blindness — the next stake, if taken, must restrict the policy's
INFORMATION about the fiber, not its symmetry.
