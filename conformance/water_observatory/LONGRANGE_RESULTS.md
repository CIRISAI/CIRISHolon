# THE LONG-RANGE RESIDUAL AUDIT — results (GANTT node B1)

*Freeze: `conformance/water_observatory/LONGRANGE_PREREG.md`, ADMITTED by
`Audit/prereg_audit.py`, committed at `6e46d01` — one commit BEFORE the instrument
(`48dc51f`). `git log --oneline` on `lane/longrange-audit` is the ordering proof, quoted
in §8.*

**Instrument:** `engine/crates/holon-render/examples/longrange_audit.rs` at `42cab33`.
Every number in this document comes from that one build.

**Raw output, committed beside this document** so every number here can be checked against
the run that produced it: `longrange_hydrogen.log`, `longrange_fenced.log`,
`longrange_plant_p1_refusal.log`.

---

## 0. VOID STRUCTURE — first, before any verdict

M-BUDGET-LAUNDER: a class that could not be scored says so at the top, so a pattern of
refusals is visible rather than inferred from an absence.

| class | parked seeds | scored | outcome |
|---|---|---|---|
| CLASS-H — pure-hydrogen gas, 12 H | 8 | 8 | **NEGLIGIBLE — branch (a)** |
| CLASS-MIX-FENCED — mixed quench, (O,O,O) fenced, 4 O + 8 H | 8 | 8 | **VOID — branch (d), price refusal fired** |
| CLASS-MIX-SERVED — mixed quench, (O,O,O) served | **0** | 0 | **VOID — branch (d), V1 missing arm** |
| CLASS-O — oxygen control, 12 O | **0** | 0 | **VOID — branch (d), V1 missing arm** |

**Three of four classes are VOID.** One class in this audit produced a scorable verdict.
That is the honest headline and it is stated before the number that did survive.

CLASS-MIX-SERVED's VOID has a documented cause rather than a bare absence:
`p2_served_arm_refusal.log` shows the arm died after 2737.9 s of curve generation with
`panicked at waterquench_traj.rs:514: the Ozone table generates` — the (O,O,O) surface was
unavailable and the runner refused rather than substituting. CLASS-O was never dumped.
Neither is estimated from a sibling.

---

## 1. VERDICTS

### CLASS-H — **NEGLIGIBLE, branch (a)**

> At `c* = 15.0` bohr — the radius the engine's cell decomposition is already built at — a
> declared pair truncation would discard at most **2.074e-14 Ha** per frame across 160,000
> frames, against a per-seed allowance of 6.81e-4 … 4.76e-3 Ha. The worst frame sits
> **2.6e-11** of the way to the criterion. **This is a LOWER BOUND: past the table's last
> knot the curve is an exponential while the true tail is a power law, so the discard is
> at least this and never at most this.**

Worst frame: seed `0x0000000053415425`, frame 12376. Branch (e) does not fire — every
seed's `B_s/D_s` is 84–387, under the 10³ threshold — so this is an informative pass. It
also survives the fence (§5.2) and the alternative denominator (§4.1) with orders to
spare. **This is the only scorable verdict the audit produced.**

### CLASS-MIX-FENCED — **VOID, branch (d): the freeze's own price refusal fired**

```
GATE PRICE CLASS-MIX-FENCED VOID (setup 1021.5 s, floor 1200 s)
VERDICT CLASS-MIX-FENCED VOID — the freeze's price refusal fired: the curve setup
        finished under the floor
```

The freeze staked (§11): *"a CLASS-MIX-FENCED run whose curve setup completes in under
1200 s — under half the priced O–O time — is REFUSED as not having generated that curve,
and its reading is void with it."* The setup took **1021.5 s**. The gate fired, and it
fired against me. **No verdict about this class in either direction.** §2.2 reports the
numbers as computed-but-VOID, §7.4 says what went wrong with the gate, and §9 names the
exit.

### CLASS-MIX-SERVED, CLASS-O — **VOID**, no verdict in either direction

---

## 2. WHAT THE NUMBERS SHOW

### 2.1 The structural finding, and it is the one that matters most

Two facts about the engine, both read from committed source *before* any measurement and
both confirmed by the sweep:

**(i) The census scenes discard nothing at all.** `waterquench_traj.rs` never calls
`Sim::set_pair_cutoff`, so `pair_switch == None` and `sim.rs:2382` runs the complete
`N²/2` pair sum. B1's question is necessarily the counterfactual the freeze staked: what
*would* a declared truncation drop. **Anyone quoting this audit as "the engine discards X"
has quoted it wrong.** It discards zero; X is what it would discard if it stopped paying
O(N²).

**(ii) The two classes differ in kind, not just in size.** Curve `r_max` values, measured:

| curve | `r_max`, bohr | relative to `c* = 15.0` |
|---|---|---|
| H–H | 10.2400 | **inside** — everything past `c*` is extrapolated |
| O–H | 10.2400 | **inside** — same |
| O–O | **20.0000** | **outside** — real table interior extends 5 bohr past `c*` |

So for CLASS-H, `E_band(c*) = 0.000000e0` in **400 of 400** published rows and identically
zero by construction: **100% of its discard at `c*` is the table's own exponential
extrapolation**, and the fence covers the entire number. For CLASS-MIX-FENCED, `E_band` is
nonzero in **144 of 400** rows, because O–O pairs between 15 and 20 bohr are genuine
tabulated interactions being dropped. The mixed class's residual is four orders larger
than hydrogen's for exactly this reason, and it is real table, not extrapolation.

### 2.2 CLASS-MIX-FENCED, computed but NOT SCORED

Reported because VOID means not-scored, not not-computed, and because the numbers are what
a successor freeze will have to gate. **None of this is a verdict.**

| seed | frames | max &#124;E_hard(c*)&#124; | at frame | `B_s` | `0.10·B_s` | ratio | `D_s` | `B_s/D_s` |
|---|---|---|---|---|---|---|---|---|
| 0x…5421 | 20000 | 6.068053e-6 | 18218 | 2.040e1 | 2.040e0 | 2.975e-6 | 1.500e-4 | 136000 |
| 0x…5422 | 20000 | 7.455787e-6 | 11762 | 8.850e0 | 8.850e-1 | 8.425e-6 | 4.800e-5 | 184375 |
| 0x…5423 | 20000 | 6.639902e-6 | 12418 | 1.580e0 | 1.580e-1 | 4.202e-5 | 5.620e-5 | 28114 |
| 0x…5424 | 20000 | 8.347427e-6 | 17821 | 2.050e0 | 2.050e-1 | 4.072e-5 | 4.610e-5 | 44469 |
| 0x…5425 | 20000 | 6.996747e-6 | 14051 | 1.330e1 | 1.330e0 | 5.261e-6 | 2.450e-4 | 54286 |
| 0x…5426 | 20000 | 8.699410e-6 | 2643 | 1.220e1 | 1.220e0 | 7.131e-6 | 1.130e-4 | 107965 |
| 0x…5427 | 20000 | **9.096429e-6** | **5682** | 8.220e0 | 8.220e-1 | 1.107e-5 | 2.260e-4 | 36372 |
| 0x…5428 | 20000 | 6.549196e-6 | 9360 | 1.360e1 | 1.360e0 | 4.816e-6 | 1.280e-4 | 106250 |

Worst frame: seed `0x0000000053415427`, frame 5682, at `|E_hard(c*)| = 9.096429e-6` Ha.

**The second reason this class delivers no usable answer, independent of the price gate.**
Every `B_s/D_s` is 2.8e4 – 1.8e5, all **far above** the freeze's 10³ threshold. Had the
price gate not fired, every seed would have passed G1 and the class would have landed in
**branch (e), NEGLIGIBLE (uninformative bound)** — a pass against a denominator five
orders above the energy the run actually failed to conserve. Two independent grounds, and
neither is a negligibility result.

**And the alternative denominator says the opposite.** Measured against `D_s`, the drift
the run actually incurred, rather than `B_s`, the bound it was entitled to:

| seed | &#124;E_hard(c*)&#124; / `D_s` | above 10% of measured drift? |
|---|---|---|
| 0x…5421 | 0.0405 | no |
| 0x…5422 | 0.1553 | **YES** |
| 0x…5423 | 0.1181 | **YES** |
| 0x…5424 | 0.1811 | **YES** |
| 0x…5425 | 0.0286 | no |
| 0x…5426 | 0.0770 | no |
| 0x…5427 | 0.0403 | no |
| 0x…5428 | 0.0512 | no |

> **Three of eight mixed seeds would FAIL a 10% test taken against the measured drift.**
> The staked criterion is the bound and the staked criterion is what governs; this is not
> a re-scored verdict and is not offered as one. It is the disclosure the freeze's §6.1
> promised to make, and it says plainly that the mixed class's long-range residual is
> **not** demonstrably small in any sense a reader would naturally mean. CLASS-H is
> negligible under both denominators (worst ratio to `D_s`: 2.6e-10). CLASS-MIX-FENCED is
> negligible under neither a scorable gate nor an honest one.

---

## 3. THE RESIDUAL TABLE

Published at the staked stride of every 400th frame — 50 frames per seed, 400 per class,
exactly as the freeze's §12 specified. Energies in hartree; `ratio` is
`|E_hard(c*)| / B_s`. Per-seed extrema in §2.2 and §4 are over all 20,000 frames of each
seed, not over the stride.

```
FRAME class            seed                 frame   E_band        E_tail        E_hard        E_switch      bound        ratio
FRAME CLASS-H          0x…5421                  0   0.000000e0   -1.785825e-15 -1.785825e-15 -6.385217e-14 6.810000e-3  2.622357e-13
FRAME CLASS-H          0x…5421                400   0.000000e0   -4.857292e-16 -4.857292e-16 -1.183247e-13 6.810000e-3  7.132588e-14
FRAME CLASS-H          0x…5421                800   0.000000e0   -1.949698e-15 -1.949698e-15 -4.502425e-14 6.810000e-3  2.862992e-13
FRAME CLASS-H          0x…5421               1200   0.000000e0   -2.450389e-15 -2.450389e-15 -7.127871e-14 6.810000e-3  3.598222e-13
FRAME CLASS-MIX-FENCED 0x…5421                  0  -2.625079e-6  -1.133875e-7  -2.738467e-6  -2.738467e-6  2.040000e1   1.342386e-7
FRAME CLASS-MIX-FENCED 0x…5421                400   0.000000e0   -1.740104e-7  -1.740104e-7  -3.592673e-6  2.040000e1   8.529921e-9
FRAME CLASS-MIX-FENCED 0x…5421                800   0.000000e0   -1.804901e-7  -1.804901e-7  -3.407832e-6  2.040000e1   8.847553e-9
FRAME CLASS-MIX-FENCED 0x…5421               1200   0.000000e0   -1.887775e-7  -1.887775e-7  -2.936373e-6  2.040000e1   9.253797e-9
```

`E_band` is zero in 400/400 CLASS-H rows and in 256/400 CLASS-MIX-FENCED rows, for the
structural reason in §2.1(ii).

---

## 4. PER-SEED, CLASS-H — all 20,000 frames of each seed

`allow` = `0.10 · B_s`. `B_s` and `D_s` are the run's own drift bound and drift peak, read
from the arm log (now committed as `census_traj_arm_hydrogen.log`, §7.3(b)).

| seed | frames | max &#124;E_hard(c*)&#124; | at frame | `B_s` | allow | ratio to allow | `D_s` | `B_s/D_s` | G1 |
|---|---|---|---|---|---|---|---|---|---|
| 0x…5421 | 20000 | 1.771078e-14 | 18902 | 6.810e-3 | 6.810e-4 | 2.600702e-11 | 8.080e-5 | 84.3 | PASS |
| 0x…5422 | 20000 | 1.879542e-14 | 17913 | 2.360e-2 | 2.360e-3 | 7.964160e-12 | 1.260e-4 | 187.3 | PASS |
| 0x…5423 | 20000 | 1.814852e-14 | 18699 | 8.030e-3 | 8.030e-4 | 2.260089e-11 | 5.230e-5 | 153.5 | PASS |
| 0x…5424 | 20000 | 1.883596e-14 | 15509 | 1.120e-2 | 1.120e-3 | 1.681782e-11 | 4.450e-5 | 251.7 | PASS |
| 0x…5425 | 20000 | **2.073722e-14** | **12376** | 4.760e-2 | 4.760e-3 | 4.356559e-12 | 1.230e-4 | 387.0 | PASS |
| 0x…5426 | 20000 | 1.741802e-14 | 283 | 1.180e-2 | 1.180e-3 | 1.476104e-11 | 7.720e-5 | 152.8 | PASS |
| 0x…5427 | 20000 | 2.012320e-14 | 16448 | 9.790e-3 | 9.790e-4 | 2.055485e-11 | 6.450e-5 | 151.8 | PASS |
| 0x…5428 | 20000 | 1.849504e-14 | 6887 | 1.360e-2 | 1.360e-3 | 1.359929e-11 | 6.800e-5 | 200.0 | PASS |

Worst frame of the class: **seed `0x0000000053415425`, frame 12376**, at
`|E_hard(c*)| = 2.073722e-14` Ha. Worst ratio to the criterion: **2.600702e-11**, on seed
`0x0000000053415421` — the seed with the tightest bound, not the one with the largest
residual, which is why both columns are printed.

### 4.1 CLASS-H against the alternative denominator

Worst `|E_hard(c*)| / D_s` = 2.074e-14 / 8.080e-5 ≈ **2.6e-10**. CLASS-H is negligible
against the measured drift as well as the bound, so its verdict does not depend on which
denominator a reader prefers. This is the contrast with §2.2 and it is why CLASS-H's
branch (a) is worth quoting while the mixed class's numbers are not.

---

## 5. THE LADDER, AND THE FENCE SIZED

### 5.1 The ladder, over all 160,000 frames per class

| `c`, bohr | provenance | CLASS-H mean / max | CLASS-MIX-FENCED mean / max |
|---|---|---|---|
| 6.0 | `sim::DE4_R_CUT` | 1.654100e-4 / **1.194844e-3** | 1.138238e-4 / 9.834113e-4 |
| 9.0 | `trimer::R_HI` | 2.932972e-7 / 1.543524e-6 | 1.385262e-5 / 1.052265e-4 |
| 10.4 | half the short box edge | 4.560080e-9 / 2.666497e-8 | 8.108298e-6 / 5.233803e-5 |
| 14.0 | `ooh::R_HI` | 7.242175e-14 / 4.525927e-13 | 1.827632e-6 / 1.398439e-5 |
| **15.0 = `c*`** | `water::R_HI` = `list_cutoff()` | 3.412073e-15 / 2.073722e-14 | 1.267956e-6 / 9.096429e-6 |
| 41.0 | above the box diagonal (zero control) | 0.000000e0 / 0.000000e0 | 0.000000e0 / 0.000000e0 |

Note the shape difference: CLASS-H falls **eleven orders** between 6.0 and 15.0 bohr;
CLASS-MIX-FENCED falls **two**. The O–O curve reaches to 20 bohr, so the mixed scene has
no radius inside its box at which the pair interaction has actually died.

**The crossing — B2's design input, CLASS-H only.** At `c = 6.0` bohr, **three of eight
CLASS-H seeds go OVER their own `0.10·B_s`** (`0x…5421`: 1.031e-3 vs 6.81e-4; `0x…5423`:
9.556e-4 vs 8.03e-4; `0x…5424`: 1.195e-3 vs 1.12e-3). At 9.0 bohr every seed is under. So
this class's negligibility boundary is bracketed in **[6.0, 9.0) bohr**, and the engine's
actual 15.0 bohr radius sits well clear of it. Locating the crossing more finely needs a
finer ladder, which needs its own freeze; the bracket is what this stake supports and it is
not widened past that. CLASS-MIX-FENCED shows no crossing anywhere on the ladder — but
only because its bound is enormous, which is §2.2's point, not a result.

### 5.2 The fence, sized rather than only warned about

Past `r_max` the table is `a·exp(−b·(R − r_edge))`, matched in value and slope at the last
knot (`table.rs:315`); the true tail is a power law. The comparison tails are matched to
the table's *own* last value, nothing fitted.

| at `c*` | CLASS-H mean / max | CLASS-MIX-FENCED mean / max |
|---|---|---|
| `E_tail` (table's exponential) | −3.412073e-15 / 2.073722e-14 | −2.848148e-7 / 2.239049e-6 |
| `E_tail_pow6` (dispersion-shaped) | −5.018870e-9 / 1.165985e-8 | −2.724874e-7 / 2.184035e-6 |
| `E_tail_pow3` (dipole-shaped) | −2.703069e-8 / 5.042154e-8 | −4.034592e-7 / 2.408732e-6 |

**CLASS-H: the exponential understates a power law by ~5.6e5 (r⁻⁶) to ~2.4e6 (r⁻³).** The
fence is enormous in relative terms — and the verdict survives it anyway: the harshest
tail at its maximum, 5.042154e-8 Ha against the tightest allowance 6.810e-4 Ha, is
**7.4e-5**, still four orders inside. CLASS-H is negligible not merely on the table's word
but on the most unfavourable power law consistent with the table's own edge value.

**CLASS-MIX-FENCED: the three agree to within 8%.** Because O–O's table reaches 20 bohr,
few pairs sit in the extrapolated region and those that do are just past the match point,
where exponential and power law have not yet diverged. The fence is nearly closed for that
class — which would have been worth having, if the class had a verdict.

### 5.3 `E_switch` is the larger estimator — the freeze had this backwards

The freeze's §3 says *"where they disagree, `E_hard` is the larger and is the one gated,
which is the conservative direction"*. **That is false, and the measurement says so.** The
C² switch begins removing energy at `c − W = 13.0` bohr, *inside* the cutoff, so
`|E_switch| ≥ |E_hard|` always:

```
SWITCHMAX CLASS-H           max|E_switch(c*)| 5.233315e-13 at seed 0x…5422 frame 16072 (ratio 25.24)
SWITCHMAX CLASS-MIX-FENCED  max|E_switch(c*)| 1.150526e-5  at seed 0x…5424 frame 10144 (ratio 1.26)
```

The gated quantity is **25.24× smaller** than the truncation the engine would actually
apply on CLASS-H, and 1.26× smaller on the mixed class. The gate stays on `E_hard` because
that is what was staked and a gate is not moved after seeing the data; the correction is
reported instead. It does not change CLASS-H's verdict — 5.233315e-13 Ha against 6.810e-4
Ha is 7.7e-10, still nine orders inside. **A successor freeze should gate `E_switch`.**

---

## 6. GATES

| gate | CLASS-H | CLASS-MIX-FENCED |
|---|---|---|
| G1 negligibility | **PASS** (0/8 seeds fail) | PASS (0/8) — *not scored, class is VOID* |
| G2 curve identity | **PASS** | **PASS** (H–H, O–H, O–O all within band) |
| G3 manifest refusal | **PASS** (9 files hashed, 0 refusals) | **PASS** (9 hashed, 0 refusals) |
| G4 work count | **PASS** (160,000 frames vs floor 50) | **PASS** (160,000) |
| G5 ladder monotonicity | **PASS** (0 violations) | **PASS** (0 violations) |
| G6 zero control | **PASS** (EXACT 0.0; max separation seen 37.8711 bohr) | **PASS** (EXACT 0.0; 37.1773 bohr) |
| G7 beyond-cutoff population | **PASS** (160000/160000) | **PASS** (160000/160000) |
| G8 plant P2 | **PASS** (rel 3.016e-14) | **PASS** (rel 9.493e-16) |
| **PRICE** | PASS (no floor staked, §7.3a) | **VOID — FIRED (1021.5 s vs 1200 s floor)** |

G6 is worth one line of validation: the largest pair separation actually seen (37.8711 /
37.1773 bohr) sits just inside the *walled region's* own diagonal of 38.7262 bohr
(`34.6 − 2×0.6` by `20.8 − 2×0.6`, `wall_inset = 0.6`), and well inside the 41.0 control
radius. The zero is a fact about the scene, not about the instrument's coverage.

G2's detail for the mixed class, because one number moved: O–O reproduced `R_e` to 2.0e-5
bohr and `D_e` to 1.4e-7 Ha, but its **worst residual came back 4.8e-6 against the arm
log's 2.7e-6 — a factor of 1.78** (inside the staked 2× band, so G2 passes). See §7.4:
that movement and the price shortfall have the same likely cause.

---

## 7. RECEIPTS

### 7.1 P1 — the manifest refusal, fired on a corrupted copy, both prongs

One byte of a copy of `seed_0x0000000053415422.traj` was flipped (offset 6,000,000,
`0x31 → 0x30`), and a byte-exact copy of an admitted trajectory was staged under a name the
manifest does not list. G3 refuses on two conditions and a demonstration of only one would
leave the other untested.

```
# ADMITTED hydrogen.log  sha256 f5f1896ddc2e2ac160bc09e7e8629d495a341f74fb35e12c877c1c28ce4fa7a2
# ADMITTED hydrogen/seed_0x0000000053415421.traj  sha256 77d51288…
# REFUSED  hydrogen/seed_0x0000000053415422.traj: sha256 9e4e221c… != manifest 93fbac50…
# REFUSED  hydrogen/seed_0x00000000deadbeef.traj: sha256 77d51288… but the manifest does not list this path
# admitted 1 of 3 parked files; refusals 2
# REFUSAL-REASON hydrogen/seed_0x0000000053415422.traj: sha256 9e4e221c… != manifest 93fbac50…
# REFUSAL-REASON hydrogen/seed_0x00000000deadbeef.traj: not listed in manifest
GATE G3 CLASS-H refusals 2 of 4 files hashed
```

The gate **discriminates** rather than blanket-refusing: the pristine sibling in the same
directory is admitted in the same pass. Note what the second refusal shows — the unlisted
file's digest `77d51288…` **is** in the manifest, as seed 5421's, just under another path.
The gate is **path-keyed, not digest-keyed**, which is correct: a byte-valid artifact under
the wrong name is unattributable, and admitting it would let one file be counted twice. The
two reasons are distinguishable in the printed field (M-EXIT-DISCRIMINATOR).

*Carrier and sector (M-PLANT-SECTOR):* the carrier is the file's sha256 digest, nonzero in
the artifact-identity sector by construction — a flipped byte changes sha256 with
probability 1 — and nonzero in no other sector the audit reads.

### 7.2 P2 — the injected pair

```
CLASS-H           carrier u_ab(16.0) = -1.547664e-16 Ha; predicted -1.716446346e-17,
                  measured -1.716446346e-17, relative 3.016e-14  → G8 PASS
CLASS-MIX-FENCED  carrier u_ab(16.0) = -2.028788e-6  Ha; predicted -6.691681669e-7,
                  measured -6.691681669e-7, relative 9.493e-16  → G8 PASS
```

The carrier is four orders larger on the mixed class because 16.0 bohr is *table interior*
for O–O (`r_max = 20.0`) and deep extrapolation for H–H (`r_max = 10.24`) — the plant is
probing two different regimes and fires in both. The instrument refuses to run the plant if
the carrier reads zero: a plant that could not have fired is not run.

*Execution note, recorded rather than made silently.* The freeze predicts the change as
"`u_ab(16.0)` minus that pair's prior contribution". Displacing an atom moves **every**
pair it is in, so the prediction carries the displaced atom's other pairs alongside — which
is what makes P2 a check of the estimator's inclusion bookkeeping rather than an identity.
Both components are printed separately (CLASS-H: −1.548e-16 target, +1.376e-16 others) so
the freeze's term and the correction term can be read apart.

### 7.3 Disclosures

**(a) CLASS-H's curve arrived 3.5× cheaper than its logged time** (1.0 s against 3.5 s).
The freeze set no price floor for CLASS-H and said why in advance: a few-second floor
cannot discriminate a real solve from a stub on this box. G2 discriminates, and passes to
6.0e-6 bohr and 3.5e-7 Ha. Recorded because a result cheaper than its banked price is
exactly what M-CHEAPER-THAN-ITS-PRICE says to look at.

**(b) The freeze called the arm logs "committed" and they were not.** The gate's
denominator is read from `hydrogen.log` / `fenced.log`, which live in
`/home/emoore/holon-artifacts/census-traj/`, outside the repository. The similarly-named
`census_hydrogen.log` and `census_mixed_fenced.log` in the tree are the **closure census's**
output — a different artifact. The arm logs were pinned only by a digest line in the
committed manifest, which the instrument does check and print. Fixed rather than annotated:
both are now committed as `census_traj_arm_hydrogen.log` and `census_traj_arm_fenced.log`,
digests matching the manifest exactly (`f5f1896d…`, `a790fbcc…`). M-STALE-INSTRUMENT's
third variant, caught before it cost anything.

**(c) The price refusal VOIDs and does not abort.** The freeze says the reading "is void
with it". The instrument sets a VOID flag the verdict line reads and still computes and
prints everything, because VOID means NOT SCORED and not NOT COMPUTED — aborting would have
discarded exactly the VOID structure M-BUDGET-LAUNDER exists to make visible. Departure
recorded rather than made silently. It is also why §2.2 exists at all.

**(d) A first CLASS-MIX-FENCED run was killed mid-curve.** It had been launched on the
instrument one commit earlier (`48dc51f`, lacking only the `SWITCHMAX` reporting line) and
was terminated rather than allowed to finish, so that every number here comes from one
instrument commit and a whole second O–O generation was not spent on top of it. It produced
no `VERDICT`; nothing was lost but a partial curve. Recorded because killing one's own
running compute is worth saying out loud.

**(e) The instrument is not build-gated by CI.** `engine/ci-gates.sh` builds and tests
crates but never builds `holon-render`'s examples, so neither this instrument nor
`waterquench_traj.rs` is compiled by CI. Observation, not a fix — owner is the lane that
owns `ci-gates.sh`, exit is one `cargo build -p holon-render --examples` line. Noted
because an instrument nothing compiles is one refactor from being a non-instrument.

### 7.4 THE FIRED GATE, examined

The price refusal is the freeze's own, it fired, and the class is VOID. That stands. But
the gate should be understood before it is re-used, because **the evidence says the curve
was genuine and the gate was wrong about how to tell.**

* **G2 says the curve is the curve.** O–O reproduced `R_e` to 2.0e-5 bohr and `D_e` to
  1.4e-7 Ha against the arm log's printed physics. A fabricated or stubbed curve does not
  do that. G2 is the discriminator the price gate was trying to be.
* **The likely cause is a faster kernel, not a skipped solve.** The O–O worst residual also
  moved, 2.7e-6 → 4.8e-6 (×1.78), on a solve that is deterministic given the same code.
  Both a changed cost and a changed residual are what an unannounced change to FCI sigma
  summation order produces — the regime already recorded in M-STALE-INSTRUMENT's case law,
  where one such landing put three lanes' identity gates red while moving no physics.
* **Wall time was the wrong unit.** The freeze priced work in seconds taken from a log
  measured while eight trajectory dumps ran concurrently. Wall time is confounded by
  contention and by placement (M-PLACEMENT-LOTTERY), and — as here — by the engine simply
  getting faster. A price gate in seconds cannot separate "did less work" from "did the
  same work quicker".

**None of this un-VOIDs the class.** The freeze is the freeze; a gate that fires is not
re-argued into passing by its author on the strength of evidence assembled afterwards, and
re-running until the clock cooperates would be gaming my own gate. What it does is name the
repair, in §9.

---

## 8. THE ORDERING PROOF

```
42cab33 The freeze called the arm logs committed; they were not, so commit them
48dc51f B1's instrument: the residual the cutoff would drop, and the refusal that guards it
6e46d01 B1's freeze: what cutoff-locality discards, staked before the instrument exists
892c982 The reality workbench gets its build graph: ...        <- branch point, main
```

The freeze landed one commit before the instrument existed, and gate 9c's auditor returned
`ADMITTED LONGRANGE_PREREG.md` before that commit was made. Gate 9c's own loop over the
whole tree passes with this freeze in it: **38 preregs seen, 0 refused.**

---

## 9. WHAT B2 GETS, AND WHAT IS OWED

**For B2's decision:** one class, CLASS-H, is measured NEGLIGIBLE at the radius the engine
is already local at, robustly — under both denominators and under the harshest power-law
fence. Its negligibility boundary is bracketed in [6.0, 9.0) bohr, so the engine's 15.0
bohr has real margin. **That is the whole of what this audit establishes.**

**It does not clear B2.** Three of four classes are VOID; the mixed class has no verdict
and, on the alternative denominator, three of its eight seeds sit above 10% of measured
drift. B1 has not shown that cutoff-locality is safe for oxygen-bearing scenes.

**Owed, in order:**

1. **B1b — a successor freeze for CLASS-MIX-FENCED**, running this instrument unchanged,
   differing from this freeze in exactly three staked places: the price gate expressed in
   *work units* (determinant count × Davidson iterations) or dropped in favour of G2, which
   is what actually discriminates; the denominator taken as the measured drift `D_s` beside
   the bound `B_s`, so the answer is not five orders from meaning anything; and `E_switch`
   gated rather than `E_hard`. The mixed class's numbers are in §2.2 and are the ones that
   freeze must be sized against.
2. **The (O,O,O) served arm**, blocked on the ozone table generating at all
   (`p2_served_arm_refusal.log`).
3. **N-scaling.** The discard is a sum over pairs, growing as N², while the drift bound
   does not. Not measured here (M-VOLUME-SCALE).

---

## 10. WHAT THIS DOES NOT SAY

Written into the freeze before the answer, so a convenient answer could not widen them
afterwards. Reproduced unchanged.

1. **Pair sector only.** The three- and four-body surfaces return exact zeros outside their
   domains and discard nothing by truncation. Whether those domains are wide enough is a
   different question with a different instrument.
2. **No ionic species.** Every nucleus here is neutral H or O and every curve decays
   exponentially. The r⁻¹ case — the one GANTT says makes B2 near-certain once node C ships
   ionic scenes — is **not touched by this measurement**. CLASS-H's negligible verdict is
   not a statement that B2 is unnecessary, and any use of it to defer B2 for ionic scenes is
   a misuse of it.
3. **Twelve atoms, two dimensions, walls.** The gated ratio is N-dependent and its N-scaling
   is not measured here (M-VOLUME-SCALE). Owed.
4. **`E_hard` is a lower bound** past the table edge — and for CLASS-H at `c*` that covers
   100% of the number (§2.1).
5. **The gated estimator is the smaller one**, by 25.24× on CLASS-H (§5.3).
