# Q7b RESULTS — the family posed the question, the kill fired on SOUNDNESS, and the certificate beat every cutoff

**Verdict up front.**

**G7-FIT PASSED — 22 of 56 configurations spatially split** (Q7: 0 of 84). For the first time in this
programme the family actually posed the question, so this verdict is about certificates and not
about my sweep design.

**THE Q7b KILL FIRES: no candidate passes all five clauses.** D3 fails **only clause 1**, with
**6 false positives out of 115 wrong region-instances (5.2%)**. Recorded dead, kept, marked.

**But the kill is a SOUNDNESS failure, not an informativeness failure, and the difference is the
result.** For the first time the chart-internal criteria decisively beat every coordinate baseline:

| | coverage | discrimination | FP |
|---|---|---|---|
| **D3** (chart data, theorem-anchored) | **0.909** | **21** | 6 |
| N4 best per-region cutoff (5 post-hoc params) | 0.436 | **0** | 0 |
| N5 best trap-aware cutoff | 0.339 | 0 | 0 |
| N3 best global cutoff | 0.242 | 0 | 0 |

**Every baseline also fails the joint gate** — they buy soundness by refusing nearly everything
(coverage 0.24–0.44) and never once discriminate within a configuration. D3 covers **2.1× more**
than the best baseline and discriminates **21 times to zero**.

> **A wording correction, not a verdict change.** The prereg's outcome (e) called a fired kill
> "per-region certification is decoration". **The measurement does not support that word.** A
> criterion that covers 91% of honest regions, discriminates on 21 configurations where every
> cutoff manages none, and certifies 6 of 115 wrong regions is not decoration — it is
> **informative and unsound**. The kill stands exactly as staked; its prereg gloss does not.

**56 of 56 configurations passed every exactness gate. Zero VOID.**

---

## 1. Every staked prediction

| Prediction | Outcome |
|---|---|
| **G7-FIT ≥ 8 split** | **PASS — 22 of 56** |
| **P-D1** — refuses plant, fails `FP = 0` in the weak-breaking threshold band | **CONFIRMED, and precisely** (§3.2) |
| **P-D1b** — expected silent; *if* it fires, it fires here | **THE INTERESTING HALF WON** — it fired 9 times, **9/9 on wrong regions** (§3.1) |
| **P-D2** — certifies the plant, fails `FP = 0` | **CONFIRMED, transferred verbatim from Q7** (FP = 33, worst on R3 at `E_r` 18.7–19.8) |
| **P-D3** — passes `FP = 0`, plant, coverage, and clause 5 | **SPLIT: confirmed on clauses 2–5, REFUTED on `FP = 0`** (6 FPs) |
| **P-D4-COVERAGE** — D4 fails clause 3, by derivation | **CONFIRMED, with the derived mechanism visible** (§3.3) |
| **P-D4-D1b-COMPLEMENT** — D4's FPs concentrate on reflection-broken configs, D1b catches them | **REFUTED** — `D5 ≡ D4` exactly (both FP = 4); D1b caught none |
| G7-E7 ruler / G7-E9 mirror / G-E5b | PASS (2.5e-14 / 9.3e-14 / 4.4e-12) |

Five confirmed, one refuted, one split, one precondition passed.

---

## 2. The refusal map — the deliverable, at `V = 4`

```
        truth      D3         D4
U=0     .....      CCCCC      CCRCC     <- D4 already refuses the honest Mott centre
U=0.5   ..X..      CCCCC      CCRCC     <- D3's false positive band
U=1     ..X..      CCRCC      CCRCC     <- both correct
U=2     ..X..      CCRCC      CCRCC
U=4     ..X..      RRRRR      CCRCC     <- D3 turns conservative, D4 stays sharp
U=8     XXXXX      RRRRR      RRRRR
U=16    XXXXX      RRRRR      RRRRR
```

This is the crystal-tier seam policy in one picture: at `U = 1` and `U = 2` the machine lights the
wings and the wells and **refuses the Mott centre**, from chart data alone, correctly. That is the
object the whole path was built to produce, and it exists — it is simply not sound at every point
of the sweep.

---

## 3. The three findings

### 3.1 D1b FIRED — the reflection anchor is live, and it is 9/9

Q7 reported D1b as **UNTESTED, not null**, because its only plausible firing domain was a column
Q7 expected to VOID. On the symmetric double well it got its real test and **fired on 9
region-instances, every one of them a wrong region** — precision 9 of 9, no false refusals.

Where: `V = 4, U = 8` (all five regions) and `V = 8, U = 16` (four of five), with chart reflection
asymmetries of 0.027–0.087 against a threshold of 0.010. Deep well, strong interaction — exactly
the domain A1(Q7)/OPT named in advance. **The primary class genuinely has two members**, and Q7's
"untested" is now a measurement. This is prereg outcome **(g)**.

It is also the one place a theorem earned something no heuristic did: D1b's refusals rest on
`⟨n_i⟩ = ⟨n_{N+1−i}⟩` being exactly true of the reference, and the chart's own asymmetry is the
deviation — `pinned_error_computable_from_chart` doing work in the field.

### 3.2 D3's soundness failure is one region, at one interaction strength

All six false positives are **the same region (the Mott centre) at the same `U = 0.5`**, at
`V ∈ {3, 4, 6, 8, 12, 16}`:

| | value |
|---|---|
| `E_r` | 1.31 → 1.53 (just over tolerance) |
| worst observable | **R2, double occupancy**, every time |
| `break_spin` | **0.0000** — the chart is spin-symmetric there, so D1 cannot see it |
| `self_audit` | 0.0079–0.0089 against a threshold of **0.0100** — D2 misses by 11–21% |

This is **exactly P-D1's staked band (i)**: the chart is symmetric, so the theorem-pinned anchor is
silent, while a non-symmetry observable has already crossed. And it is a *near-miss*: the
self-residual's estimate sits just inside its threshold on all six. **The failure is one narrow band
in `(U)`, not a systemic unsoundness** — 6 of 115 wrong regions, 5.2%.

**What I am not doing:** moving `κ`. Raising it would convert these six FPs into a pass, and the
threshold was frozen in Q5 precisely so that this move is unavailable.

### 3.3 P-D4-COVERAGE confirmed, with its derived mechanism visible in the map

D4 **fails clause 3**, as derived before the run. The mechanism is legible in the `V = 4` map above:
**at `U = 0`, D4 refuses region 2** — the `n ≈ 1` centre — because its density is maximally far from
both determinate fillings, even though at `U = 0` the chart is *exact* there.

That is precisely the derivation: **density extremity is a sufficient route to local determinacy,
not the criterion, and `U → 0` is a second route D4 is structurally blind to.**

**The consequence the stake was written for held:** D4 scored well (FP = 4, coverage 0.897,
discrimination 22 — marginally *better* than D3 on all three) and **still cannot own the headline,
because it dies on a clause D1, D1b and D3 all pass.** Outcome (b′) never had to be argued.

**P-D4-D1b-COMPLEMENT is refuted:** `D5 = D4 ∧ D1b` scored **identically to D4** (FP = 4). D1b
caught none of D4's false positives, so the two are not complementary here — D4's unsoundness is
not the chart lying about density.

---

## 4. Scope

`N = 10` only — N = 8 is impossible by arithmetic (§1.1 of the prereg), so **nothing here addresses
size-dependence**. And the scope sentence stands as staked: **Q7b tests whether the machinery works
where the question exists.** The family was selected by the pre-check *because* it splits, so a
certificate performing well here has not been shown to perform anywhere else. Transfer to families
nobody tuned is a separate claim with its own campaign — the natural out-of-family test being the
engine's own tiers, where nobody chooses the potential. **Neither the kill nor the coverage result
settles that.**

## 5. Files

Prereg `sim_engine/Q7B_SEAM_PREREG.md` (+ A1(Q7b)); pre-check
`crates/q-seam/examples/q7b_spread.rs` → `output/q7b_seam/spread_precheck.log`; runner
`crates/q-seam/src/bin/q7b_run.rs`; output `output/q7b_seam/q7b.{json,log}`.
