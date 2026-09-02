# THE dE5 TRUNCATION AUDIT — results

*GANTT node H. Freeze: `conformance/water_observatory/DE5_PREREG.md`, committed ahead of
the instrument; `git log --oneline` is the proof of order. Instrument:
`engine/crates/holon-chem/examples/de5_audit.rs`. Run receipts:
`de5_score.log`, `de5_audit.csv`, `de5_plants.log`, `de5_scf_probe.log`.*

---

## THE VERDICT

> # BRANCH (b) — THE LADDER DOES NOT TERMINATE AT FOUR.
>
> **Worst measured `|dE5|` = 7.859411e-02 Ha — 1,572x the ladder's own declared per-term
> uncertainty of 5.0e-5 Ha.** Every one of **24 of 24** live configs is over that bound;
> the smallest is still 36x it and the median is 456x. Zero VOID, zero VACUOUS.
>
> **On the worst config the five-body residue is 2.83x the ENTIRE four-body rung** it is
> supposed to be a correction to (`dE5` = +7.859e-2 against a `dE4` sum of +2.777e-2), and
> **3 of 24 configs have a `|dE5|` larger than any single four-body term in the same
> cluster** (worst ratio 1.1285). At those geometries the expansion is not merely
> un-terminated — it is not decreasing at the point of truncation.
>
> **The DMRG-cluster seam requirement fires.** GANTT's `MPS` node — "DMRG for compact
> cores, MBE far-field, seam defect-audited" — lists itself as gated on this verdict, and
> this is the verdict.
>
> **No five-body term was written.** Node H's rule is measure, never build, and a lane
> that answered a truncation question by extending the truncation would have answered a
> different question.

**And the scope, in the same sentence, every time:** planar (2D) geometries, STO-3G,
composition `O2H3` only, cluster diameter below 6.0 bohr, subtraction basis `fci_live`,
atom-based many-body expansion. It is NOT a statement about three-dimensional water, about
a molecule-based MBE, or about the compositions the determinant fence refused.

**Read this beside it:** under G2 exactly as frozen, this audit is **BRANCH (d) VOID** and
has no verdict at all. See §7.

---

## 1. RECEIPTS — the order, and the gate

The freeze landed alone, then a pre-data correction, then the instrument:

```
2c8243d  The dE5 instrument, its three plants firing, and a gate the data convicted
6654405  Two pre-data corrections to the dE5 freeze, before the instrument exists
5c27f42  Stake the dE5 audit before the instrument that could bias it exists
151362c  (the lane's base commit)
```

`5c27f42` touches one file, `conformance/water_observatory/DE5_PREREG.md`, and no
instrument existed in the tree at that commit. Gate 9c's auditor, on the frozen document:

```
ADMITTED DE5_PREREG.md
```

---

## 2. THE PLANTS — all three fire

The freeze requires every plant to be shown firing before any reading is trusted
(M-PLANT-OBS), and each to name the sector its carrier must be nonzero in
(M-PLANT-SECTOR). Full transcript in `de5_plants.log`.

### P-1 — the dropped quadruple must shift dE5 by EXACTLY that term

Carrier: `dE4(Q*) = +6.652414206064e-2` Ha, the largest four-body term in the config.
Sector: the FOUR-BODY sector, and it is nonzero there — `|dE4(Q*)| = 6.652e-2`, far above
the `1.0e-6` floor below which the instrument REFUSES to run the plant at all.

```
    dE5 unplanted  +7.507207076645e-2
    dE5 planted    +1.415962128271e-1
    shift          +6.652414206064e-2
    expected       +6.652414206064e-2   (= +dE4(Q*))
    |shift - dE4(Q*)| = 2.109e-15   tolerance 1.0e-12
    P-1 FIRES — the instrument sees what it measures
```

Deleting one term from the assembly and nothing else moves `dE5` by that term and nothing
else, to **2.1e-15 Ha** — four hundred times inside the staked tolerance, and eleven orders
below the bound.

### P-2 — the separated atom must read ZERO

Carrier: one atom translated to 40 bohr from the cluster centre. Sector: the GEOMETRY
sector, nonzero there by tens of bohr, while the FIVE-BODY sector must read exactly zero —
the MBE is size-consistent, so a four-cluster plus a distant atom has an identically
vanishing five-body term.

```
    dE5 = +9.350742e-12 Ha   tolerance 1.0e-8   P-2 FIRES
```

**This is the control that makes the headline believable.** A large `dE5` could in
principle be an assembly error — a term double-counted, a sign wrong, a subset enumerated
twice. P-2 exercises the identical assembly on a geometry where the answer is known
exactly, and gets zero to 9.4e-12 Ha. The 7.5e-2 Ha read elsewhere is therefore physics
and not arithmetic, and the zero is a fact about the SCENE (the separation) rather than
about the instrument's coverage.

### P-3 — the exhausted budget must VOID, never score

Carrier: `fci::DAVIDSON_MAX_ITER`, lowered to 2 so a real solve genuinely fails. Sector:
the SOLVER-EXIT sector, nonzero there — `IterationCap` where the unplanted run reports
`Converged`.

```
    VOID as staked: atom Z8 (n_det 25): exit iteration cap
    P-3 FIRES — the exhausted case is refused, not scored
```

M-BUDGET-LAUNDER's refusal is mechanized rather than promised: the instrument was made to
fail through its production path and refused the case instead of scoring it.

---

## 3. THE LANDSCAPE — and the base rate that limits everything below

Every compact five-atom candidate the frozen sampling rule found, in scope or not
(freeze 3.6, M-BASE-RATE-OMITTED):

| composition | candidates | determinants | in scope |
|---|---:|---:|:--|
| `O2H3` | 31 | 204,490 | **yes** |
| `O3H2` | 110 | 5,664,400 | no |
| `O4H1` | 36 | 121,788,765 | no |
| **TOTAL** | **177** | | |

> **146 of 177 — 82.5% — of the compact five-clusters these trajectories actually visit
> are OUT OF SCOPE for this audit**, refused by the determinant fence and not by any
> judgement about them. The verdict speaks for 17.5% of the visited landscape.

That is the single largest limitation of this result and it is stated first, not last.

**Two staked compositions were never visited at all.** The freeze named `OH4` and `O2H3`
"at minimum, chosen because water scenes visit them", and added `H5` for free. Measured:
`OH4` has **0** compact candidates and `H5` has **0**. The scene is 4 O and 8 H in a
34.6 x 20.8 bohr box, oxygens cluster at ~2.4 bohr O–O, and no five-set containing four or
five hydrogens ever falls inside a 6.0 bohr diameter. So:

* the audit is **single-composition**, and BRANCH (c) — mixed by composition — was
  structurally unreachable, because it needs two compositions with six live configs each;
* the freeze's expectation about `OH4` was simply wrong about this scene, and is recorded
  as wrong rather than quietly dropped.

---

## 4. THE DRAW — the cap, declared, with the excess counted

| composition | enumerated | drawn | excess |
|---|---:|---:|---:|
| `H5` | 0 | 0 | 0 |
| `OH4` | 0 | 0 | 0 |
| `O2H3` | 31 | 24 | **7** |

Drawn total **24**, against `N_TARGET = 24`. The seven undrawn candidates are the declared
excess; nothing was silently capped.

*(The draw's first implementation over-ran this cap — it drew 31 — because allocation and
sampling were interleaved. It was found by printing the counts beside the target, fixed
before any config was scored, and the allocation is now integer arithmetic settled before
a candidate is touched. Recorded because a cap that has been wrong once is worth saying so
about.)*

---

## 5. THE READING

| # | seed | frame | diam (bohr) | dE5 (Ha) | \|dE5\| / bound | max\|dE4\| (Ha) | ratio | verdict |
|---:|---|---:|---:|---:|---:|---:|---:|:--|
| 1 | `0x53415422` | 13000 | 5.653 | +7.859411e-02 | 1,572x | 7.4580e-02 | 1.0538 | **OVER** |
| 2 | `0x53415421` | 4500 | 5.727 | +7.507207e-02 | 1,501x | 6.6524e-02 | 1.1285 | **OVER** |
| 3 | `0x53415422` | 16250 | 5.767 | +6.584658e-02 | 1,317x | 7.4659e-02 | 0.8820 | **OVER** |
| 4 | `0x53415422` | 14500 | 5.872 | +5.718526e-02 | 1,144x | 7.1794e-02 | 0.7965 | **OVER** |
| 5 | `0x53415421` | 10000 | 5.631 | +4.825916e-02 | 965x | 4.5098e-02 | 1.0701 | **OVER** |
| 6 | `0x53415422` | 9750 | 5.969 | +4.034204e-02 | 807x | 7.2263e-02 | 0.5583 | **OVER** |
| 7 | `0x53415421` | 10000 | 5.631 | +3.728186e-02 | 746x | 4.0824e-02 | 0.9132 | **OVER** |
| 8 | `0x53415421` | 16000 | 5.669 | +3.579893e-02 | 716x | 4.5512e-02 | 0.7866 | **OVER** |
| 9 | `0x53415421` | 18000 | 5.616 | +3.408666e-02 | 682x | 4.1058e-02 | 0.8302 | **OVER** |
| 10 | `0x53415421` | 14000 | 5.626 | +3.074337e-02 | 615x | 4.0277e-02 | 0.7633 | **OVER** |
| 11 | `0x53415421` | 8000 | 5.787 | +2.748646e-02 | 550x | 3.3701e-02 | 0.8156 | **OVER** |
| 12 | `0x53415421` | 12000 | 5.723 | +2.478885e-02 | 496x | 4.2310e-02 | 0.5859 | **OVER** |
| 13 | `0x53415421` | 16000 | 5.669 | +2.077987e-02 | 416x | 4.0154e-02 | 0.5175 | **OVER** |
| 14 | `0x53415421` | 6000 | 5.772 | +1.767187e-02 | 353x | 3.6192e-02 | 0.4883 | **OVER** |
| 15 | `0x53415421` | 5000 | 5.557 | +1.605783e-02 | 321x | 4.7162e-02 | 0.3405 | **OVER** |
| 16 | `0x53415422` | 7500 | 5.886 | +1.565850e-02 | 313x | 5.2947e-02 | 0.2957 | **OVER** |
| 17 | `0x53415421` | 16000 | 5.669 | -4.407974e-03 | 88x | 2.3616e-02 | 0.1867 | **OVER** |
| 18 | `0x53415421` | 14000 | 5.626 | -3.849224e-03 | 77x | 2.4934e-02 | 0.1544 | **OVER** |
| 19 | `0x53415421` | 12000 | 5.723 | -3.739379e-03 | 75x | 2.4309e-02 | 0.1538 | **OVER** |
| 20 | `0x53415421` | 18000 | 5.616 | -3.738177e-03 | 75x | 2.6360e-02 | 0.1418 | **OVER** |
| 21 | `0x53415421` | 10000 | 5.631 | -3.553050e-03 | 71x | 2.5586e-02 | 0.1389 | **OVER** |
| 22 | `0x53415421` | 6000 | 5.772 | -3.436216e-03 | 69x | 2.0572e-02 | 0.1670 | **OVER** |
| 23 | `0x53415421` | 8000 | 5.787 | -3.049465e-03 | 61x | 2.5007e-02 | 0.1219 | **OVER** |
| 24 | `0x53415421` | 5000 | 5.557 | -1.788368e-03 | 36x | 2.2784e-02 | 0.0785 | **OVER** |

### The distribution, in one line each

* **N_live = 24** (the freeze's bar is 20)
* worst `|dE5|` = **7.859411e-02 Ha** = **1,572x** the 5.0e-5 Ha bound
* median `|dE5|` = 2.278436e-02 Ha = 456x the bound
* smallest `|dE5|` = 1.788368e-03 Ha = 36x the bound
* configs at or over the bound: **24 of 24**
* convergence ratio `|dE5| / max|dE4|`: min 0.0785, median 0.5379, max 1.1285
* configs where `|dE5|` EXCEEDS the largest four-body term: **3 of 24**

* VACUOUS (excluded, freeze 3.5): 0
* VOID (never scored, freeze 6): 0
* scored total: 24 of 24 drawn

### Numerical headroom — the measurement resolves the bound it is read against

* worst single-solve Davidson residual over every scored solve: **9.993e-11 Ha** (the freeze's bar is 1.0e-9)
* worst per-config residual SUM over its 31 solves: **1.779e-09 Ha**
* that sum is **28,106x** below the 5.0e-5 Ha bound and **1,005,266x** below the SMALLEST measured `|dE5|`

So the readings are not arithmetic noise, and G8's claim is a measured number rather
than a hope.

### The strict reading, published beside the amended one (freeze 5b)

* configs that would VOID under G2 exactly as frozen: **24 of 24**
* solves reporting `scf_converged = false`: **70**
* solves where the flag is UNOBSERVABLE (freeze C-2): **24** — one per config, the `O2H3` pentamer

**STRICT VERDICT: BRANCH (d) VOID** — 0 of 24 configs survive the frozen clause, below the bar of 20, so G2 as frozen returns no verdict about the
ladder at all. The amended reading below is the one that carries information, and it is
never quoted without this paragraph beside it.

---

## 6. THE WORST CONFIG, printed in full

The instrument prints it in full at the end of every completed run
(`de5_score.log`), verbatim here:

```
  worst LIVE |dE5| = 7.859411e-2 Ha   bound 5.0e-5   OVER
  worst config: O2H3 seed 0x53415422 frame 13000 idx [0, 1, 4, 5, 6] diameter 5.652645 bohr
    atom 0 Z=8 at [4.230387029735, 9.546631275821, 12.000000000000]
    atom 1 Z=8 at [2.731176115476, 7.602983350484, 12.000000000000]
    atom 4 Z=1 at [3.291512546341, 11.351373899681, 12.000000000000]
    atom 5 Z=1 at [3.063362921829, 5.703335262854, 12.000000000000]
    atom 6 Z=1 at [0.680444960503, 7.490735480802, 12.000000000000]
    dE4 terms: [0.003479677056907271, -0.00045344211718550964, 0.07458010415015937, -0.01960675054300315, -0.030231297871973817]
    rung sums:  dE2 -5.947892691e-1 (max |1.476e-1|)   dE3 +3.299263201e-1 (max |1.036e-1|)   dE4 +2.776829067e-2 (max |7.458e-2|)
    E_FCI -149.166546592311   E_MBE4 -149.245140698642   dE5 +7.859410633e-2
```

**Read the rung sums.** The ladder's rungs on this cluster are

| rung | sum (Ha) | largest single term (Ha) |
|---|---:|---:|
| dE2 | -5.947892691e-1 | 1.476e-1 |
| dE3 | +3.299263201e-1 | 1.036e-1 |
| dE4 | +2.776829067e-2 | 7.458e-2 |
| **dE5** | **+7.859410633e-2** | — |

**The five-body residue is 2.83x the entire four-body rung it is supposed to be a
correction to.** A truncated expansion whose next term is nearly three times the sum of the
last one it kept is not converged at the point of truncation, and this is the plainest form
the finding takes.

The geometry is a planar `O2H3` — two oxygens at 2.45 bohr, three hydrogens — with every
`z` exactly 12.0, which is the 2D scene's plane (freeze section 2.3).

---

## 7. THE STRICT READING, AND AMENDMENT A-1

**The audit's own primary gate, as frozen, returns no verdict.** G2 required
`scf_converged` on every solve, and it VOIDs every config.

The freeze's amendment A-1 (§5b) is post-data and labelled so there. Its trigger, its
evidence and its cost:

* **Trigger.** 24 of 24 configs VOID on one clause and one rung — `(O,H,H)` triples at
  ordinary water geometries (`r_min` 2.02–2.16 bohr, 441 determinants). A gate that
  refuses 100% of its sample is measuring itself.
* **Evidence.** The question went to an INDEPENDENT reference — the committed `(O,H,H)`
  surface in `engine/crates/holon-chem/tests/data/s2/s2_water_table.txt`, built by another
  campaign, on another day, through `examples/s2_table.rs` — and was answered by
  measurement (`de5_scf_probe.log`):

  | SCF flag on the live solve | n | worst \|live − served\| |
  |---|---:|---:|
  | `scf_converged = true` | 76 | 5.386e-4 Ha |
  | `scf_converged = false` | 68 | 6.678e-4 Ha |

  **The flag does not predict disagreement.** The two worst cases differ by 1.24x, both
  land on the same stretched near-linear `H–O···H` shape, and the largest disagreements
  fall on both sides of the flag. Pairs are 240 converged and 0 not; the failure is
  entirely at arity 3.
* **What changed.** `scf_converged = false` is recorded per config, not VOIDed. The
  residual bound (1.0e-9 Ha) and the other three VOID clauses are untouched.
* **What it costs.** A post-data amendment is the weakest change this programme permits.
  The strict reading is therefore computed on every run and published in §5 beside the
  amended one, and `Sub::strict_scf_void` survives in the instrument for exactly that
  purpose — an amendment that deleted its own predecessor would leave nothing to compare
  against.

**Two findings handed on rather than absorbed:**

1. **For the crate.** 28% of `(O,H,H)` solves at trajectory geometries fail
   `pair::orbital_rotation`'s 1e-10-in-200-damped-iterations test. The served water table
   was built through those same solves. This is a fact about the convenience SCF, not
   about this audit, and it is reported, not fixed here.
2. **For `pair::geometry_problem`.** It is the only public entry point that returns the
   `(space, mo, nuc)` triple `solve_determinant` needs — and it DISCARDS `scf_converged`
   (`let (u, _, _) = orbital_rotation(...)`). So the only route to an exact-in-model solve
   is also the only route that cannot report SCF convergence. That is M-EXIT-DISCRIMINATOR
   one level up from where the misfit was registered: a diagnostic one entry point carries
   and its sibling does not.

---

## 8. G9 — the two-basis cross-check, and what it independently validates

For every `OHHH` quadruple in the sample the instrument prints this audit's live `dE4`
beside `quaternary::de4_ohhh_fci` on the same four centres. Representative rows from
`de5_score.log`:

```
    G9 OHHH(drop 0): live +4.957483e-3  served +4.957905e-3  diff -4.215991e-7
    G9 OHHH(drop 1): live -2.332925e-3  served -2.327251e-3  diff -5.673442e-6
```

Differences sit at **1e-7 to 1e-5 Ha**, far inside the served-table budget of §1.1 and far
below the 1.0e-3 Ha level the freeze set as a FINDING.

**This matters more than a gate passing.** It is an independent check that the audit's
rung-4 machinery reproduces the engine's own production four-body term. The instrument is
not measuring a private quantity: where the runtime has an answer, the audit agrees with
it, and the disagreement it reports is at rung FIVE, where the runtime has nothing.

**It also vindicates the `fci_live` subtraction basis, numerically.** The probe of §7
measured live-vs-served `dE3` differences up to **6.678e-4 Ha** — 13x the 5.0e-5 Ha bound,
and above `water.rs`'s own stated held-out maximum. A served-table assembly of ten triples
would have injected error of that order into the residue and called it a five-body term.
The freeze argued this from published table errors; the run measured it.

---

## 9. THE READING OF THE PHYSICS — labelled as a reading

The estimator is an **atom-based** many-body expansion: its fragments are bare O and H
atoms, and its rungs are atom pairs, triples and quadruples. That is the expansion the
engine runs, which is why it is the one audited.

An atom-based MBE on a covalently bonded cluster is not the regime in which many-body
expansions converge quickly; the well-behaved case is a MOLECULE-based expansion over
weakly interacting fragments. Reading the measured `dE5` beside the measured `dE4` on the
same clusters is consistent with that: the ratio `|dE5| / max|dE4|` has median **0.5379**
and reaches **1.1285**, and on 3 of 24 configs the five-body term is larger than any
four-body term in the same cluster. On the worst config the five-body residue is 2.83x the
whole four-body rung sum.

**This is a reading, not a result.** What is measured is the residue's size on this
sample. The mechanism above is the most parsimonious account of it and is offered as such;
this audit did not test it, and a molecule-based comparison would be the instrument that
did.

---

## 10. WHAT FIRES, AND WHAT DOES NOT

**Fires:** GANTT's `MPS` node — the DMRG-cluster seam. Its receipt is "seam defect budget
staked and measured", and this verdict is the dependency it named. The seam's own budget
is its business, not this audit's; what this audit supplies is the reason it is needed and
a measured size for what four-body truncation discards on this class of cluster.

**Does not fire, and must not be read as firing:**

* Nothing about `O3H2` or `O4H1`, which are 82.5% of the visited landscape.
* Nothing about three-dimensional geometries.
* Nothing about the engine's four-body FORCE path being wrong. `de4_ohhh_fci` agrees with
  this audit's independent live computation to 1e-7–1e-5 Ha (§8). The four-body term is
  correct; the finding is that it is not the LAST term.
* Nothing about a molecule-based MBE, which is a different expansion and is not audited.

**A gap this audit found and hands to GANTT node A** (species-generic MBE): the engine's
four-body force path covers `OHHH` only, and `O2H3` clusters contain three `O2H2`
quadruples with no runtime machinery at all. This audit computed them live; the runtime
cannot.

---

## 11. WHAT THIS AUDIT DOES NOT CLAIM

* It does not claim the many-body expansion diverges. It measures one residue, at one
  arity, on one basis, at planar geometries of one composition drawn by one stated rule.
* It does not claim a null anywhere, and no `|dE5|` here is reported as zero
  (M-NULL-MISSTAKE).
* It does not claim its maximum bounds the ladder. The worst number is a bound on the
  SCORED set only, and it is quoted beside the VOID count every time
  (M-MAX-OVER-SUCCESSES).
* It does not claim the `fenced` arm's dynamics was correct. The trajectory is a geometry
  sampler; the pin claims byte identity with the census's banked bytes and nothing more.
* It does not claim its own primary gate worked. It did not, and §7 says so at full volume.

---

## 12. CORRECTIONS TO THE FREEZE, recorded rather than absorbed

| id | when | what |
|---|---|---|
| **C-1** | pre-data | `OHHH` is 1,568 determinants, not 52,920. Exactly ONE subsystem class crosses `MPS_ROUTE_THRESHOLD` (the `O2H3` pentamer at 204,490); the next largest, `O2H2` at 48,400, clears it by only 3.2%, so the instrument reads `Solution::route` on every solve rather than inferring it. |
| **C-2** | pre-data | G2's `scf_converged` clause cannot be delivered on the one solve that must go through `geometry_problem`. Required and checked where observable; recorded UNOBSERVABLE by name where not. |
| **A-1** | **post-data** | The `scf_converged` VOID clause convicted by measurement against an independent reference; recorded per config instead. Strict reading published beside the amended one always. §7. |
| **C-3** | post-data | The freeze counts "26 solves per config" (1 pentamer + 5 quadruples + 10 triples + 10 pairs) and forgot the five ATOM solves. The true count is **31**, which is what the instrument gates and what every CSV row reports. No gate changes — all of them were always applied to *every* solve — but the number in the freeze's prose was wrong and is corrected here rather than left to be discovered. |

---

## 13. THE RUN

| | |
|---|---|
| instrument | `engine/crates/holon-chem/examples/de5_audit.rs` |
| freeze | `conformance/water_observatory/DE5_PREREG.md` |
| device_class | `cpu` |
| solver_budget | 5000 (`fci::DAVIDSON_DEFAULT_BUDGET`, unmodified) |
| subtraction_basis | `fci_live` (every rung, every arity, determinant route) |
| trajectory pin | blob `f62486aa908ba8f382099049853f28d5a04f1b27`, 8 `fenced` seeds, all sha256-verified |
| placement | `nice -n 10`, detached via `setsid`, loadavg recorded at both ends of each run |
| receipts | `de5_score.log`, `de5_audit.csv`, `de5_plants.log`, `de5_scf_probe.log` |
