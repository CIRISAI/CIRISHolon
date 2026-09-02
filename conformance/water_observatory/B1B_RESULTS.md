# B1b — THE LONG-RANGE RESIDUAL AUDIT, SUCCESSOR RESULTS

*Freeze: `B1B_PREREG.md`, ADMITTED by `Audit/prereg_audit.py`, committed at `1569288` —
one commit BEFORE the instrument carried any of its three changes (`b4ef534`). Same
ordering discipline as B1, same proof: `git log --oneline`, quoted in §7.*

**Instrument:** `engine/crates/holon-render/examples/longrange_audit.rs` at `cba50e3`.
**Raw output, committed beside this document:** `b1b_hydrogen.log`, `b1b_fenced.log`,
`b1b_b1_reproduction.log`.

---

## 0. VERDICTS

| class | B1 | **B1b** |
|---|---|---|
| **CLASS-MIX-FENCED** — mixed quench, 4 O + 8 H | VOID (price refusal fired) | **NON-NEGLIGIBLE — branch (c) SPLIT. The B2 Ewald requirement FIRES.** |
| **CLASS-H** — pure-hydrogen gas, 12 H | NEGLIGIBLE — branch (a) | **NEGLIGIBLE — branch (a)**, under a strictly harder test |
| CLASS-MIX-SERVED, CLASS-O | VOID (V1, missing arm) | VOID (V1) — not re-opened |

### CLASS-MIX-FENCED — **NON-NEGLIGIBLE, branch (c) SPLIT**

> At `c* = 15.0` bohr, a declared pair truncation would discard up to **1.150526e-5 Ha**
> per frame (seed `0x0000000053415424`, frame 10144). That **passes** against the drift
> bound the integrator was *entitled* to lose (`0.10·B_s`) and **fails** against the drift
> the run *actually incurred* (`0.10·D_s`) on **3 of 8 seeds**, reaching **2.496× the
> criterion** on the worst.
> **This is a LOWER BOUND: past the table's last knot the curve is an exponential while
> the true tail is a power law, so the discard is at least this and never at most this.**

The split is the finding, and the freeze staked it as its own branch precisely so it could
not be rounded to either side: **the discard is small against what the integrator was
entitled to lose and not against what it actually lost.** Under the conjunction B1b staked,
that is NON-NEGLIGIBLE, and **the B2 Ewald-class requirement fires for this class.**

### CLASS-H — **NEGLIGIBLE, branch (a)** — the control that makes the above trustworthy

Worst `|E_switch(c*)| = 5.233315e-13` Ha against a tightest `0.10·D_s` of 4.45e-6 Ha — a
margin of **1.2e-7**. B1b tightens the test in two coordinates at once (larger estimator,
tighter denominator), so a design that reported everything non-negligible would have
measured its own severity rather than the engine. **It did not flip its control.** The
mixed class's failure is a fact about the mixed class.

---

## 1. WHAT THE THREE CHANGES DID

| change | B1 | B1b | effect on the answer |
|---|---|---|---|
| estimator | `E_hard` | **`E_switch`** — what `set_pair_cutoff` actually applies | ×1.26 on mixed, ×25.24 on hydrogen |
| denominator | `0.10·B_s` only | **`0.10·B_s` AND `0.10·D_s`**, both required | **this is what changed the verdict** |
| price | wall-clock seconds | **W1 solver certificate + W2 in-run cost ratio** | **this is what let the class be scored at all** |

The 0.10 fraction is **inherited unchanged**. The one number most obviously tunable to
produce an outcome is the one number the successor freeze refused to touch.

**The price change is the whole reason there is a verdict here.** B1 refused this class at
1021.5 s against a 1200 s wall-clock floor. B1b prices the same work as
`t(O–O)/t(H–H) = 961.0/1.3 = 725.7` against a floor of 100 — comfortably inside the
555–944 band measured on three prior runs before the floor was staked. A kernel speedup
scales both curves and cancels; that is exactly what B1's floor could not do.

---

## 2. PER-SEED — CLASS-MIX-FENCED, all 20,000 frames of each seed

`E_switch(c*)` is the gated quantity. `B_s` and `D_s` are the run's own drift bound and
drift peak, read from the committed arm log `census_traj_arm_fenced.log`.

| seed | max &#124;E_switch(c*)&#124; | at frame | `0.10·B_s` | G1a | `0.10·D_s` | ratio | **G1b** |
|---|---|---|---|---|---|---|---|
| 0x…5421 | 6.811667e-6 | 18253 | 2.040e0 | PASS | 1.500e-5 | 0.454 | pass |
| 0x…5422 | 9.112721e-6 | 12228 | 8.850e-1 | PASS | 4.800e-6 | **1.898** | **FAIL** |
| 0x…5423 | 8.846165e-6 | 2110 | 1.580e-1 | PASS | 5.620e-6 | **1.574** | **FAIL** |
| 0x…5424 | **1.150526e-5** | **10144** | 2.050e-1 | PASS | 4.610e-6 | **2.496** | **FAIL** |
| 0x…5425 | 7.635210e-6 | 15412 | 1.330e0 | PASS | 2.450e-5 | 0.312 | pass |
| 0x…5426 | 1.109996e-5 | 1907 | 1.220e0 | PASS | 1.130e-5 | 0.982 | pass |
| 0x…5427 | 1.001624e-5 | 5753 | 8.220e-1 | PASS | 2.260e-5 | 0.443 | pass |
| 0x…5428 | 7.611945e-6 | 15442 | 1.360e0 | PASS | 1.280e-5 | 0.595 | pass |

**G1a: 0 of 8 fail. G1b: 3 of 8 fail.** Worst frame: seed `0x0000000053415424`, frame
10144, at **2.496×** the incurred-drift criterion — the discard is **25% of the drift that
run actually incurred**. Seed `0x…5426` sits at 0.982, just inside; **the boundary runs
through this seed set rather than past it**, which matters because it means the answer is
not driven by one outlier.

### 2.1 CLASS-H, the control

| seed | max &#124;E_switch(c*)&#124; | `0.10·B_s` | `0.10·D_s` | G1a | G1b |
|---|---|---|---|---|---|
| 0x…5421 | 4.971991e-13 | 6.810e-4 | 8.080e-6 | PASS | PASS |
| 0x…5422 | **5.233315e-13** | 2.360e-3 | 1.260e-5 | PASS | PASS |
| 0x…5423 | 4.668607e-13 | 8.030e-4 | 5.230e-6 | PASS | PASS |
| 0x…5424 | 4.519501e-13 | 1.120e-3 | 4.450e-6 | PASS | PASS |
| 0x…5425 | 4.655616e-13 | 4.760e-3 | 1.230e-5 | PASS | PASS |
| 0x…5426 | 4.153109e-13 | 1.180e-3 | 7.720e-6 | PASS | PASS |
| 0x…5427 | 4.716530e-13 | 9.790e-4 | 6.450e-6 | PASS | PASS |
| 0x…5428 | 4.260891e-13 | 1.360e-3 | 6.800e-6 | PASS | PASS |

Seven orders of margin on the tighter gate. The two classes are separated by nine orders of
magnitude in the gated quantity, and the reason is structural.

---

## 3. WHY THE TWO CLASSES DIFFER BY NINE ORDERS

Not a matter of degree. The curve ranges, measured:

| curve | `r_max` | relative to `c* = 15.0` | `u(r_max)` |
|---|---|---|---|
| H–H | 10.2400 bohr | **inside** | −6.272736e-9 Ha |
| O–H | 10.2400 bohr | **inside** | −1.645191e-10 Ha |
| **O–O** | **20.0000 bohr** | **5 bohr OUTSIDE** | −6.641237e-7 Ha |

For `CLASS-H`, every pair beyond `c*` is also beyond the table's last knot, so the whole
discard is exponential extrapolation and is numerically nothing. For `CLASS-MIX-FENCED`,
**O–O pairs between 15 and 20 bohr are genuine tabulated interaction being thrown away** —
real table interior, not tail. `E_band(c*)` is nonzero in 144 of 400 published stride rows
for the mixed class and in 0 of 400 for hydrogen.

> **The engine's locality radius sits 5 bohr inside its own O–O curve's support.**

That is the mechanism behind the branch (c) verdict, and it is what B2 has to fix — not a
numerical accident of these eight seeds.

---

## 4. THE LADDER — CLASS-MIX-FENCED, and what B2 gets

| `c`, bohr | mean &#124;E_hard&#124; | max &#124;E_hard&#124; |
|---|---|---|
| 6.0 | 1.138238e-4 | 9.834113e-4 |
| 9.0 | 1.385262e-5 | 1.052265e-4 |
| 10.4 | 8.108298e-6 | 5.233803e-5 |
| 14.0 | 1.827632e-6 | 1.398439e-5 |
| **15.0 = `c*`** | 1.267956e-6 | 9.096429e-6 |
| 41.0 (zero control) | 0.000000e0 | 0.000000e0 |

The mixed class falls **two** orders across the whole ladder where hydrogen falls eleven.
There is **no radius inside this box at which the O–O pair interaction has died**: at the
largest cutoff the box can carry, the residual is still 1e-5 Ha. The freeze's pre-committed
follow-up asks for the radius at which the class first crosses `0.10·D_s`; **this ladder
does not contain one.** That is a stronger statement than a crossing radius would have
been: for oxygen-bearing scenes at this density, cutoff-locality has no safe radius
available in a 34.6 × 20.8 bohr box, and **B2's answer is a long-range method, not a bigger
cutoff.**

### 4.1 The fence, sized

| at `c*` | mean | max |
|---|---|---|
| `E_tail` (table's exponential) | −2.848148e-7 | 2.239049e-6 |
| `E_tail_pow6` (dispersion-shaped) | −2.724874e-7 | 2.184035e-6 |
| `E_tail_pow3` (dipole-shaped) | −4.034592e-7 | 2.408732e-6 |

The three agree to within 8% for this class, because O–O's table reaches 20 bohr so few
pairs sit in the extrapolated region. **The fence is nearly closed here** — the mixed
class's number is close to its own true value rather than a distant lower bound, **and it
fails anyway.** For `CLASS-H` the fence is enormous (the exponential understates a power
law by 5.6e5 – 2.4e6) and that class passes even against the harshest tail.

---

## 5. GATES

| gate | CLASS-MIX-FENCED | CLASS-H |
|---|---|---|
| **G1a** entitled bound `0.10·B_s` | **PASS** (0/8 fail) | **PASS** (0/8) |
| **G1b** incurred drift `0.10·D_s` | **FAIL (3/8 fail)** | **PASS** (0/8) |
| G2 curve identity | PASS (H–H, O–H, O–O) | PASS |
| G3 manifest refusal | PASS (9 hashed, 0 refusals) | PASS (9 hashed, 0 refusals) |
| G4 work count | PASS (160,000 frames vs floor 50) | PASS (160,000) |
| G5 ladder monotonicity | PASS (0 violations) | PASS (0) |
| G6 zero control | PASS (EXACT 0.0; max sep 37.1773 bohr) | PASS (EXACT 0.0; 37.8711 bohr) |
| G7 beyond-cutoff population | PASS (160000/160000) | PASS (160000/160000) |
| G8 plant P2 (vs `E_switch`) | PASS (rel 9.493e-16 vs 1e-9) | PASS (rel 1.473e-16 vs 1e-9) |
| **W1** solver certificate | **PASS** (3 curves) | **PASS** (1 curve) |
| **W2** in-run cost ratio | **PASS** (725.7 vs floor 100) | NOT APPLICABLE (one curve) |

---

## 6. RECEIPTS

### 6.1 B1 reproduces bit-identically under the changed instrument

`--freeze=b1 --class=hydrogen` on the instrument at `cba50e3` reproduces every `FRAME`,
`SEED`, `SEEDLADDER`, `LADDER`, `FENCE`, `FENCEMAX`, `SWITCHMAX` and `VERDICT` row of the
banked B1 log with **zero diff** (`b1b_b1_reproduction.log`). B1b is a change of *design*,
not of arithmetic, and **B1's VOID is undisturbed** — it remains a fact about B1, reported
in `LONGRANGE_RESULTS.md`, not erased by this freeze.

### 6.2 W1 — the solver certificates, on the record for the first time

```
# W1 H-H: route Determinant  exit Converged     n_det    4  n_basis  2  solver_budget 5000  worst_residual 8.719e-11
# W1 O-H: route Determinant  exit Converged     n_det   90  n_basis  6  solver_budget 5000  worst_residual 9.945e-11
# W1 O-O: route Determinant  exit IterationCap  n_det 2025  n_basis 10  solver_budget 5000  worst_residual 4.809e-6
```

An analytic stub has no determinant space, no route, no exit and no residual; these are the
evidence that electronic-structure work was done, and they are what the founding
M-CHEAPER-THAN-ITS-PRICE case lacked entirely. Printed and **not** gated on a value,
because no prior record carried them and a freeze cannot gate a number it would have to
invent — recording them is what lets a successor gate them.

**One of them is a finding in its own right.** O–O exits `IterationCap`, not `Converged`:
the solve **ran out of its 5000-iteration budget** with the residual at 4.8e-6, four orders
above `CONVERGED_RESIDUAL = 1e-9`. `IterationCap` and `Stagnated` are different facts — the
first says more budget would help, the second says it would not — and this is exactly the
discriminator M-EXIT-DISCRIMINATOR exists to make legible. The census's own arm log carries
only a `# WARNING` about the residual and never says which exit produced it. **The O–O
curve underlying every mixed-class result in this campaign is budget-limited, and that is
now on the record.** It bounds how finely this class can be read, and it is a candidate
cause of the 1.78× residual movement between runs noted in `LONGRANGE_RESULTS.md` §7.4.

### 6.3 W2 — the work-unit price, where the wall clock failed

```
# GATE W2 CLASS-MIX-FENCED: t(O-O)/t(H-H) = 961.0/1.3 = 725.7 against floor 100 — PASS
```

725.7 sits inside the 555–944 band measured on three prior runs *before* the floor was
staked, and 7.3× above the floor. The same run's absolute setup — 980.9 s — **would have
been refused again** by B1's wall-clock floor of 1200 s. That is the whole difference
between the two freezes in one line: the same work, priced in a unit that does not move
when the kernel gets faster.

The ratio has its own spread and it is small: the two B1b mixed runs, same design on the
same box minutes apart, read 675.9 and 725.7 — a factor of **1.07**, against the 1.70 this
quantity spans across sessions and the 2.74 the absolute second-count spans. Both sit ~7×
above the floor. The unit is stable in the way the freeze claimed it would be, and the
claim is now measured rather than asserted.

### 6.4 P2 — the plant, re-derived for the changed estimator

```
CLASS-MIX-FENCED  carrier (1-S2)*u_ab at 16.0 bohr = -2.028788e-6 Ha
                  target-pair -2.028788129e-6 | other pairs +1.359619962e-6
                  predicted -6.691681669e-7 | measured -6.691681669e-7 | relative 9.493e-16
CLASS-H           carrier (1-S2)*u_ab at 16.0 bohr = -1.547664e-16 Ha
                  predicted 1.071235614e-14 | measured 1.071235614e-14 | relative 1.473e-16
```

The carrier moved with the estimator — B1b's is what the switch *removes* at 16 bohr, not
the bare table value — and the independent prediction path uses the same inclusion rule as
the gate. Both fire; both are nonzero in the sector they act on. The hydrogen carrier is
nine orders smaller because 16 bohr is deep extrapolation for H–H and table interior for
O–O: the plant probes two regimes and fires in both.

### 6.5 A receipt that misstated its own gate, found and fixed before banking

The first B1b mixed run printed `GATE PRICE ... PASS (setup 973.3 s, floor 1200 s)` — false
on its face, since 973.3 is under 1200. The **gating** was correct (B1b prices with W1 and
W2, whose own lines were right and both of which passed); the **summary line** was still
printing B1's wall-clock floor beside a verdict that floor had no part in. Fixed at
`cba50e3` and the class re-run, so the banked log states what actually gated it.

Recorded because it is the exact shape this campaign keeps finding elsewhere: *the number
is right and the warrant printed beside it is not*, and nothing numerical catches it
because nothing numerical is wrong. A reader auditing that log would have concluded either
that the gate was broken or that I had scored through a failing price. Neither was true,
and neither should have been guessable from the record.

---

## 7. THE ORDERING PROOF

```
cba50e3 A receipt that misstated its own gate: the price line named B1's floor
b4ef534 B1b's three changes, and B1 still reproduces bit-identically under them
1569288 B1b's freeze: price the work, gate the estimator the engine actually applies
50d51d5 B1 measured: one class negligible, three VOID, and my own price gate fired
42cab33 The freeze called the arm logs committed; they were not, so commit them
48dc51f B1's instrument: the residual the cutoff would drop, and the refusal that guards it
6e46d01 B1's freeze: what cutoff-locality discards, staked before the instrument exists
892c982 (branch point, main)
```

Both freezes landed before the instrument work they govern. Gate 9c's auditor returned
`ADMITTED B1B_PREREG.md` before `1569288` was made, and its full-tree loop passes with both
freezes present: **39 preregs seen, 0 refused.**

---

## 8. WHAT THIS BANKS, AND WHAT IT DOES NOT

**Banked:**

1. **Cutoff-locality at `c* = 15.0` bohr is NOT safe for oxygen-bearing scenes.** The
   discard reaches **25% of the incurred drift** on the worst of eight seeds and exceeds
   10% on three. **B2 is required for this class**, measured rather than assumed.
2. **The mechanism is named**: the engine's locality radius sits 5 bohr *inside* the O–O
   curve's own support, so real tabulated interaction is discarded — not tail.
3. **There is no safe radius on the ladder.** The residual is still 1e-5 Ha at the largest
   cutoff the box can carry. B2's answer is a long-range method, not a bigger number.
4. **Cutoff-locality IS safe for the pure-hydrogen class** — by seven orders on the tighter
   gate and against the harshest power-law tail. The savings case holds exactly where the
   curve's support ends inside the cutoff, which is the condition worth carrying forward.
5. **The O–O curve is budget-limited** (`IterationCap` at 5000 iterations, residual 4.8e-6).

**Not banked, and unchanged from B1:**

1. **No ionic species.** Every nucleus here is neutral H or O. The r⁻¹ case is untouched;
   B2's necessity for ionic scenes is neither shown nor excluded by this measurement.
2. **Twelve atoms, two dimensions, walls.** The gated ratio is N-dependent and its
   N-scaling is still **not measured** (M-VOLUME-SCALE). Still owed.
3. **The census scenes still discard nothing as they stand** — `pair_switch == None`, so
   the pair sector runs the complete `N²/2` sum. Every number here is a counterfactual
   about the O(N) route a scaled-up scene must take.
4. **Every number is a LOWER BOUND** past the table edge — though for the mixed class the
   fence is nearly closed (§4.1), so this one sits close to its own true value.
