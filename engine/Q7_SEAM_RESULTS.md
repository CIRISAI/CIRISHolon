# Q7 RESULTS — VOID at the family gate: the trap moved the density, not the honesty

**Verdict up front. Q7 is VOID, not killed.** `G7-FIT` — the precondition that the family actually
pose the question — **failed: 0 of 84 configurations were spatially split** at the staked margin.
The harmonic trap produced a large spatial variation in **density** (0.01 → 1.79 across a single
chain) and almost none in **honesty**: the largest within-configuration spread in `E_r` is a factor
of **2.10**, against a `U`-axis that moves `E_r` by a factor of ~20. **This is a verdict on my
potential and my grid. It licenses no statement whatever about whether certificates can beat
cutoffs.**

**The VOID does not depend on the margin, and that was checked rather than assumed.** At the loose
criterion (`min_r E_r ≤ 1 < max_r E_r`, no margin) exactly **8** configurations split — which would
have met G7-FIT's `≥ 8` on the nose. So the standing robustness rule (A3/R2) was applied: **under
the loose criterion, every candidate still discriminates on 0 of 8.** No candidate ever certifies an
honest region while refusing a wrong one in the same configuration, under either reading. **Both
adjudications agree; the margin is not what decided this.**

**84 of 84 configurations passed every exactness gate. Zero VOID configurations.**

---

## 1. Every staked prediction, including the two I got wrong

| Prediction | Staked in | Outcome |
|---|---|---|
| **P-D1** — refuses the plant, fails `FP = 0` | A1(Q7)/P1 | **CONFIRMED** (plant refused; FP = 78) |
| **P-D2** — **certifies** the plant, fails `FP = 0` | §7 (derived) | **CONFIRMED** (plant CERTIFIED; FP = 90) |
| **P-D1b** — silent everywhere | A1(Q7)/OPT | **CONFIRMED** (FP = 281 = certify-everywhere exactly) |
| **P-D3** — `FP = 0`, plant refused, coverage ≥ 0.5 | A1(Q7)/P1 | **CONFIRMED** (FP = 0, refused, cov 0.598) |
| **P-D3 / P-D2-SPATIAL** — passes clause 5 | A1(Q7)/P1 | **REFUTED** — 0 discriminating, under both readings |
| **P-VOID-a** — deep-trap columns mass-VOID | A1(Q7)/P4 | **REFUTED** — 0 VOID configurations anywhere |
| **G7-FIT** — ≥ 8 spatially split | §4 | **FAILED: 0 of 84** → Q7 VOID |
| G7-E7 free-fermion ruler ≤ 1e-12 | §5 | PASS (5.8e-14) |
| G7-E9 mirror identity ≤ 1e-11 | §5 | PASS (7.1e-13) |
| G-E5b spin residual ≤ 1e-11 | §3 | PASS (1.1e-12) |

Four confirmed, two refuted, one precondition failed. Nothing reinterpreted after the fact.

---

## 2. Why the family failed, with the mechanism

The trap does exactly what the physics says: at `a = 8, N = 10` the exact density profile runs
**0.01, 0.30, 1.29, 1.60, 1.79, 1.79, 1.60, 1.29, 0.30, 0.01**. That is a wedding cake. But
honesty does not follow density, and the reason is that **both ends of the density range are
chart-friendly**:

- **dilute wings** (`n → 0`): almost no double occupancy, mean field is near-exact;
- **near-filled core** (`n → 2` at large `a`): a band insulator, locally determinate, mean field is
  **also** near-exact.

The chart-hard regions are the **`n ≈ 1` shoulders in between**. So a harmonic trap moves regions
from one chart-easy limit to the other *through* a chart-hard shoulder, and at fixed `U` the
shoulder is only modestly worse than the ends. Measured, at the largest spread in the whole sweep
(`N = 10, U = 1, a = 8`):

```
E_r  =  0.83   1.75   0.94   1.75   0.83
          ↑      ↑      ↑      ↑      ↑
        wing  shoulder core  shoulder wing
```

Non-monotone, worst at the shoulders — physically correct, and a factor of 2.1 from best to worst.

**And the spatial variation that does exist is mostly not the trap's.** Of the 8 loose-split
configurations, **seven are at `a ≤ 1` and three at `a = 0` — no trap at all**. Their pattern is
`X..X`: the **open-boundary edge** regions are the wrong ones. That is a Friedel/edge effect present
in Q5's family already, and the trap adds little on top of it. **The knob I chose was not the knob
that moves honesty.**

---

## 3. What was measured anyway (diagnostics, not verdicts)

Since G7-FIT gates everything, none of the following is a verdict about certificates. All of it is
reported because it was computed.

| criterion | FP | coverage | `U=0` | plant | discriminating | clauses passed |
|---|---|---|---|---|---|---|
| D1 spin anchor | 78 | 1.000 | ok | refused | 0 | 3 of 5 |
| D1b reflection anchor | 281 | 1.000 | ok | **CERTIFIED** | 0 | 2 of 5 |
| D2 self-residual | 90 | 0.598 | ok | **CERTIFIED** | 0 | 2 of 5 |
| **D3 = D1 ∧ D1b ∧ D2** | **0** | **0.598** | ok | refused | **0** | **4 of 5** |
| N1 certify-everywhere | 281 | 1.000 | ok | CERTIFIED | 0 | fails (as required) |
| N2 refuse-everywhere | 0 | 0.000 | FAIL | refused | 0 | fails (as required) |

**Clause 5 is the one D3 fails, and it is the clause Q7 existed to test.** D3 passes soundness
(`FP = 0`), coverage, `U = 0` and plant-refusal, and never once certifies one region while refusing
another where the truth is split. Its refusal maps are uniform across space — `CCCCC` at `U = 0`,
`RRRRR` at every `U ≥ 0.5`. That is prereg outcome **(c)**: *spatially blind, a global cutoff by
another name even with `FP = 0`* — **but it cannot be read as a verdict, because the family never
presented a split for it to get right.**

### 3.1 D2's derived failure, confirmed exactly

§7 derived, before the instrument existed, that `σ_m² = U²·n↑(1−n↑)·n↓(1−n↓)` vanishes at a fully
spin-polarised site — the broken-symmetry core — where the chart lies maximally. Measured: **D2
certifies the plant**, and its false positives are worst on **R3, the magnetization, at `E_r` up to
19.4 tolerances.** The self-residual certifies regions whose magnetization is wrong by nineteen
times its tolerance. `SelfAudit.error_not_computable_from_chart` with an algebraic witness and a
measured face.

### 3.2 D1b was silent, exactly as staked — and it is UNTESTED, not null

D1b's FP count is **281, identical to certify-everywhere**: it never refused a single region.
A1(Q7)/OPT predicted precisely this, and predicted that its one real chance was the deep-trap
column. That column produced no reflection-broken SCF solution, so **D1b's honest reading is
UNTESTED, not null** — as written in advance.

### 3.3 P-VOID-a was refuted, and my mechanism was backwards

I predicted the deep-trap columns would mass-VOID through reflection-paired quasi-degeneracy: a
core separating two wings whose even/odd excitation pair splits exponentially. **Zero
configurations VOIDed.** The mechanism was backwards: a harmonic trap is *low* at the centre, so
particles concentrate there and the wings go **empty** — there are no left/right wing states to be
near-degenerate. Reported as loudly as a confirmation.

### 3.4 Two free readings

`⟨S²⟩ ≤ 4.8e-24` at every configuration, so **Lieb's `S = 0` conclusion survives the trap
empirically** even though §2.2 deliberately did not assume it — the anchor never needed it, and now
we know it holds anyway. And **G7-E9**, the demoted particle–hole identity `E₀(v) = E₀(−v) + 2Σv_i`,
holds to **7.1e-13** across all 84 configurations: a dead anchor repurposed into a live gate, and
it earned its keep by validating the potential machinery against an independent Hamiltonian.

---

## 4. What the next campaign should do differently

Named concretely, because "pick a better family" is not a finding.

The requirement is a potential that leaves **different regions in different correlation regimes at
the same `U`**, which a smooth trap cannot do — it sweeps every region through the same shoulder.
What would:

1. **A step/box potential** pinning some regions at `n = 1` (Mott, chart-hard) and others at `n = 0`
   or `n = 2` (chart-easy), with a **sharp** boundary rather than a smooth ramp — so regions sit in
   distinct regimes rather than at different points of one ramp.
2. **Binary disorder** (`v_i ∈ {0, W}` on a staked realization), which does the same without a
   smooth profile — at the cost of a random seed, which is why the prereg declined it, and that
   trade now looks worth paying.
3. **Any family whose honesty spread exceeds ~4×** at fixed `(N, U)`, since the staked margin needs
   `min ≤ 0.5` and `max ≥ 2.0`. **Measure that spread on two configurations before freezing the
   next prereg** — it is cheap, and it is exactly the check whose absence cost Q5 and Q7 in turn.

Point 3 is the transferable lesson and it is the same one twice: **Q5 died because the honesty
boundary did not move along the sweep axis; Q7 died because it did not move across space.** Both
were checkable in advance for the price of two configurations, and neither was checked.

---

## 5. Scope

One model family (1D open Hubbard chain, harmonic trap, half filling, `N ∈ {8,10}`), one potential
shape, one block size. **No claim about certificates**, about mean-field theory, or about the
crystal tier's seam follows from a VOID. The instruments — the inhomogeneous reference, the
free-fermion ruler, the mirror gate, the per-region observables and the refusal-map machinery — are
built, gated and reusable against a family that does pose the question.

## 6. Files

Prereg: `sim_engine/Q7_SEAM_PREREG.md` (frozen + A1(Q7)). Crate: `sim_engine/crates/q-seam`
(`hubbard.rs` potential + `free_reference`, `chart.rs` potential + `sigma`/`reflection_asymmetry`,
`region.rs`, `src/bin/q7_run.rs`). Output: `sim_engine/output/q7_seam/q7.{json,log}`, ledger
`output/q7_seam/RESUME.md`.
