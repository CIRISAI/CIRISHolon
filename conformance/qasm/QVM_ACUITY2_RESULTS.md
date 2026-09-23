# QVM-ACUITY-2 — READ: one blind procedure picks tableau, MPS and dense where they are cheapest and every certificate held, but S1 is KILLED on family T (the dense vector beats the sum at `n ≤ 20`, so the family's label was wrong, not the pick) and S4 is KILLED on family L (a depth-6 CX brickwork never needs more than `χ = 8`, so a `χ = 64` cap costs nothing extra). Branches (b) and (d), with a cost the stakes left out: the search itself runs 100× longer than the view it picks on family C

*2026-09-23. Prereg `conformance/qasm/QVM_ACUITY2_PREREG.md` (frozen alone, `4e91d8e`; "Notes
on building" appended below it, no stake moved); `QVM_ACUITY2_AMENDMENT_1.md` (committed alone,
`e959fa7`, before the campaign seed ran: the MPS probe runs at cadence, the certificate is
`Σ√w`, the open choices fixed). Instruments: `holon::views` (the dictionary, a complex MPS
built for this campaign, the search), `qvm_acuity2` (the driver), `qvm_acuity2_sweep.py`.
Results in `qvm_acuity2_results.json` and `qvm_acuity2_table.md`: 80 instances (C/T/L/D × 20,
seed 2026, shuffled, search given no label), driver wall 1390 s on cores 21–27, load average
about 9.5. Plants in `tests/qvm_acuity2_plants.rs`, all green with the crate's suite.*

## 0. Verdict

This is the reading of record: the cadence probe from Amendment 1. §2 grades the frozen prefix probe beside it.

| stake | staked | read | verdict |
|---|---|---|---|
| **S1** the right view, unlabelled | the family's hand-picked view on `≥ 18/20`; kill `< 15/20` on any family | **C 20/20** tableau · **T 5/20** (sum 5, **dense 15**) · **L 20/20** MPS · **D 16/20** (dense 16, MPS 4). The prices behind the 15 T misses: dense `1.4·10⁻³–0.61 s` against sum `4.8·10⁻³–6.1 s` | **KILLED on T** (5 < 15); MET on C and L; D missed but not killed |
| **S2** the price is honest | selected wall `≤ 2×` the best by-hand wall on every instance; kill: `> 5/20` over `4×` in a family | within 2×: **C 20/20, T 19/20, L 20/20, D 18/20**; over 4×: C 0, T 0, L 0, **D 2** (`5.3×`, `62×`: two MPS picks at `χ = 16` whose wall the price missed); T's one miss is `2.2×` (the best there was an MPS at `χ > 16`, beyond the probe's range) | **MISSED on T and D, killed nowhere**; MET on C and L |
| **S3** the certificate never lies | `\|value − referee\| ≤ certificate ≤ ε` on every instance; kill: one violation | **0 violations in 80** (with the referee's `τ = 10⁻¹²` floor, Amendment 1 item 3). Tableau and dense are exact to `≤ 10⁻¹⁶`. The sum always ran in full (certificate `0`). MPS certificates run `10⁻¹⁵–10⁻¹⁴` against errors `10⁻¹⁷–10⁻¹⁵`, a worst certificate/error of 223 (L) and 31 (D) | **MET** |
| **S4** the budget bites where the terms decay | L: MPS at the acuity's `χ` over `χ = 64`, wall `≤ 0.25` at `10⁻¹` and `≤ 0.5` at `10⁻²`; kill `> 0.75` at `10⁻¹` | median ratio **1.013** at `10⁻¹`, **1.021** at `10⁻²`. The acuity's `χ` is 4 or 8, but no bond ever exceeds `8`, so the `χ = 64` run *is* the `χ = 8` run. T's executed fraction (reported, not staked) is **1.000 on 20/20** | **KILLED** — branch (d) |
| **S5** the comparison class | stated beside every row; the claim is that the one procedure lands `≤ 2×` of each family's hand-picked method | selected view's wall / comparison, median: **C 0.55** against stim (20/20 within 2×) · **T 0.19** against the full sum (20/20) · **L 1.01** against `χ = 64` (20/20) · **D 1.00** against dense (18/20). **Counting the search as part of the procedure:** C **112×** (0/20 within 2×), T 0.38 (17/20), L 4.1 (0/20), D 4.2 (1/20) | **the view is within 2× on all four families; the procedure with its search is not, on C, L and D** |

**Plants:** PQ-1, PQ-3, PQ-4 and PQ-5 fire as frozen. PQ-2 fires under Amendment 1's probe (12/12 cells) and FAILS under the frozen probe. That failure is why the amendment exists, and a test pins it. Numbers are in §3.

**Branches (b) and (d)**, not (a). The general claim is not banked on this dictionary. Each kill has a specific cause:

- **(b) on T: the label was wrong, not the closure test.** The prereg calls the sum "the cheapest known" for T, but at `n ≤ 20` and `t ≤ 20` the `2^n` vector is cheaper on 15 of the 20 instances. The search priced that correctly: on those 15, S2's ratio is `0.42–1.02`, so the dense pick was the best wall by hand.
  - ACUITY-1 had already measured this. It put the sum-vs-statevector crossing near `t = 24` at `n = 20`, and said the statevector "wins everywhere" at `n = 12, 16`.
  - Every one of the 5 sum picks is an amplitude at `n ≥ 16`. No marginal picks the sum, because a marginal costs the sum `2^{|L|−4}` legs.
  - S1's kill is read as staked. The repair (b) calls for belongs to the family, not the search: a T family that means "the sum is cheapest" has to sit above the crossing, which means `n > 20` and no referee there.
- **(d) on L: the decay the budget needs is not in this family.** A CX brickwork six entangling layers deep has every bond `≤ 2³ = 8` (named before the read, Amendment 1 item 4). An MPS capped at 64 therefore does exactly the work of one capped at 8, and the budget has nothing to trim. The acuity is `10⁻¹·2^{−n/2}`, and at the bonds that exist (`≤ 8`) it tolerates no truncation.

## 1. The families

Cadence reading, 20 instances each. Walls are medians.

| family | what it is | selected | selected wall | best by hand | S2 median / worst | cert / \|err\| worst | comparison (S5) |
|---|---|---|---|---|---|---|---|
| **C** | Clifford, `20n` deep, `n ∈ {12,16,20}` | tableau 20 | `3.9·10⁻⁵ s` | tableau | 1.15 / 1.46 (timing noise on a 40 µs view) | exact, `≤ 10⁻¹⁷` | stim `6.7·10⁻⁵ s`: 0.55× |
| **T** | Clifford + `t ∈ {12,16,20}` | dense 15, sum 5 | — | dense 14, sum 5, MPS 1 | 1.00 / 2.20 | exact (the full sum ran) | full sum: 0.19× |
| **L** | CX brickwork, depth 6, `t = 40–48` | MPS 20 (`χ` = 4 ×11, 8 ×9) | `≤ 10⁻³ s` | MPS | 1.00 / 1.05 | `3.6·10⁻¹⁵` / `10⁻¹⁵` | `χ = 64`: 1.01× |
| **D** | `n = 12`, `20n` deep, `t = 40–48` | dense 16, MPS 4 | `1.7·10⁻³ s` | dense 19, MPS 1 | 1.00 / 62 | `2.2·10⁻¹⁴` / `2.9·10⁻¹⁵` | dense: 1.00× |

**Every family, the selection by view:** C → tableau 20. T → dense 15, sum 5. L → MPS 20. D → dense 16, MPS 4.

**D's MPS picks are not probe errors.** Four `n = 12` instances with `t ≥ 40` were closed by the probe at `χ = 16`, with a certificate at the floor (`10⁻¹⁴`). Their entanglement really never passes 16: `20n` random gates with one CX in six on twelve wires does not saturate a 64-wide middle bond. The certificates held (errors `≤ 7·10⁻¹⁵`).

Two of those four cost `5×` and `62×` the dense wall, against a price within `1.4×`, at the same `χ`, bond and op count as the two that ran at the dense wall. The cause is not isolated. The likely one is the one-sided Jacobi SVD, whose sweep count depends on the spectrum; the MPS price `ops·(c_m0 + c_m3·χ³)` has no term for that. The price model is wrong there. S2 missed it and did not kill.

**The cost of the search itself.** It is not staked (S2 compares view walls), but it is the number a user pays:

| family | median search wall | worst search wall |
|---|---|---|
| C | 7.2 ms | 40 ms |
| T | 8.2 ms | 48 ms |
| L | 0.7 ms | 2.2 ms |
| D | 5.6 ms | 106 ms |

The views it chooses run in 40 µs (C) to 1.7 ms (D). On C the search is a median 157× the tableau view it selects; most of that is the cadence probe running MPS before it gives up. S5's "ONE procedure within `2×`" holds for the view and fails for the procedure on three families. A dictionary that probes every view is only worth its search when the selected view costs more than the probes do. Here, the instances are small enough that it never does.

## 2. The frozen probe, graded beside

Under the prefix probe as written in §1 (the first `2n` gates, projected), the outcome would have been branch (e), and no reading at all:

- **S1:** T 0/20, D 5/20.
- **S2 killed on T, L and D:** 17, 20 and 15 instances over 4×.
- **S3 violated on 52 of 60 non-Clifford instances.** The probe closes at `χ = 2` whenever the first `2n` gates discard nothing, and the whole-circuit run then certifies `0.5–3.4` against `ε ~ 10⁻⁵–10⁻³`.

The certificate did not lie there; it refused. The run said, correctly, that it was far from `ε`, and S3's clause "certificate `≤ ε`" is what fails. Both readings are in the table.

## 3. Plants

- **PQ-1:** five seeds. Each is a Clifford circuit on wires 0–7 plus a disjoint block carrying two T gates, read through the marginal on 0–3. Every seed prints `hard=0 removed=2` and selects the tableau, with the value within `10⁻¹²` of the referee.
- **PQ-2:**
  - Amendment 1's probe: on a `20n`-deep brickwork at `n = 12, 16, 20`, every `χ ≤ 16` projects `7–127` against `ε`, so the MPS view is not closed and not selected, on 12/12 cells.
  - The frozen prefix probe closes at `χ = 2` on all three (test `pq2_as_frozen_the_prefix_probe_is_blind_to_depth`).
- **PQ-3:** four pairs with Schmidt weights `(cos²π/8, sin²π/8)` or `(½, ½)` are pulled across the middle cut by a SWAP network.
  - The middle-cut spectrum of the untruncated state matches the analytic product to `10⁻¹²`.
  - At `χ = 1, 2, 4, 8` the certificate `Σ√w` is `1.83, 1.73, 1.69, 1.000` times the true 2-norm error. It is never below the error, and its worst ratio is `1.83` (`≤ 4` staked).
  - At `χ = 8` exactly one truncation carries all the weight, so the bound is tight.
- **PQ-4:** the planted halving goes on the most frequent runner-up, the sum (on C, where t_eff = 0 makes it one branch).
  - The sweep: 10 flips (all tableau → sum on C, price ratios `1.07–1.78`). S2 catches all 10, at walls `7–34×` the best. The sum's price at `t_eff = 0` is `~10×` optimistic: `Magic5Source` re-derives its identities on every construction, a fixed cost the per-branch constant does not carry.
  - The test's 16-instance version: 2 flips, walls `12–21×`.
- **PQ-5:** 80/80 on the sweep (every price equal to the bit), 24/24 in the test. `Query::label` exists and nothing reads it.

## 4. How the MPS probe decides, and what it cost

- **Closure** (Amendment 1): for `χ = 2, 4, 8, 16` in turn, the circuit runs on the MPS at that cap. Every `2n` circuit gates the certificate so far, `Σ√w` (times `1 + ‖ψ‖` for a marginal), is projected by `× G/g`. The run stops the moment a projection or the running certificate passes `ε`. The first `χ` that finishes under `ε` closes the view.
- **The MPS itself:** complex, mixed-canonical (QR moves the centre onto each pair), SWAP-routed with the qubit map carried rather than undone, and truncated by a one-sided Jacobi SVD. It resolves small singular values to `~ε_mach` absolute. Values `≤ 10⁻¹³` are dropped and charged.
- **Measured constants** (calibrated once per driver run, seed `0xCA11`, printed as `calibrate:`):

| constant | value | unit |
|---|---|---|
| `c_tab` | `9.8·10⁻¹⁰` | s per `n²·G` |
| `c_sb` | `2.1·10⁻⁸` | s per branch·`(G+t)(n+t)` |
| `c_sa` | `2.5·10⁻⁹` | s per branch·leg·`(n+t)²` |
| `c_m0` | `7.7·10⁻⁶` | s per two-site op |
| `c_m3` | `1.7·10⁻¹⁰` | s per op·`χ³` |
| `c_dense` | `1.4·10⁻⁹` | s per `2ⁿ·G` |

- **What the constants mean:** below `χ ≈ 36` the MPS cost is the fixed per-op overhead (QR, SVD setup), not the `χ³` the prereg priced.
- **What is missing from the MPS view:**
  - a term for the SVD's spectrum-dependent sweep count (the likely cause of the D misses, not isolated);
  - `χ > 16` in the probe: T id 9's best was an MPS at `χ = 32/64` the probe could not close;
  - interval arithmetic: the certificate is a true bound on the exact-arithmetic truncation, and f64 rounding (`~10⁻¹⁵` per step) is covered only by the referee floor `τ`.

## 5. Owed

- A T family above the crossing (`n > 20`, no referee: only the certificate speaks).
- An L family whose bricks have operator Schmidt rank 4 (two CX each), so the bond can pass the budget's `χ`.
- A search whose own cost is priced and staked: probe only the views whose price could beat the cheapest closed exact view, which removes the MPS probe on C.
- The SVD's cost in the MPS price.
- The next freeze's views as §5(a) names them: matchgates, Pauli propagation, the GPU fold.
