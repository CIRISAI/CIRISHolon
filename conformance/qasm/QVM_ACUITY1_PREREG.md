# QVM-ACUITY-1 — locate the cheap part, price the hard part, do only enough of it: PREREGISTRATION

*Frozen 2026-09-22, committed alone before any code. The general QVM's claim, in `OBJECT.md`'s
"The procedure, as arithmetic": a circuit is a motion; the tableau view is closed under
Clifford motions and splits under T (`Stabilizer.lean`); the cheap part is the closed sector
and runs exact; the hard part is the +1 — the stabilizer rank, priced before running by the
T-count oracle and the magic price `2^{M₂}`; and the acuity `ε` demanded of the output is the
budget that says how much of the hard part to do. The arithmetic underneath is Bravyi–Gosset
(2016), Qassim–Pashayan–Gosset (2021) and Kissinger–van de Wetering–Vilmart (2022), whose
partial decomposition `magic5.rs` already ports with credit; what this campaign adds is the
three things the object adds everywhere: the cheap sector FOUND rather than declared, the
price stated as a certificate BEFORE the run, and the remainder carried as a GATE.*

## 1. Instances

Random circuits over `{h, s, sdg, x, z, cx, t, tdg}` on `n ∈ {12, 16, 20, 24}` qubits with
`t ∈ {8, 12, 16, 20, 24, 28}` T-gates placed at random among `20n` Clifford gates, five seeds
each; the observable is one amplitude `⟨y|C|0⟩` for a declared `y` and one marginal
probability on the first four qubits; the acuity `ε ∈ {10⁻¹, 10⁻², 10⁻³}` is the absolute
error demanded on it. Referees: the exact statevector for `n ≤ 20`; the FULL exact branch
sum (`run_magic`, all branches, exact `Z[ω]` arithmetic) wherever `N(t) ≤ 3^6 · 2^2`.

## 2. Stakes, each with its kill

- **S1 — the cheap sector is located, not named.** A search over the circuit's views
  (`sector.rs`, to be built: the tableau view's closure tested gate by gate, and the light
  cone of each non-Clifford gate on the observable) returns the Clifford sector and the
  set of T-gates the observable depends on; T-gates outside the observable's light cone
  are REMOVED before pricing. Stake: on every instance the located sector reproduces the
  observable exactly when the removed gates are dropped (referee: the exact statevector);
  the removed count is reported. **Kill:** one instance where dropping a "removed" gate
  changes the observable by more than `10⁻¹²`.
- **S2 — the price is stated before the run.** `N_pred = expected_branches(t_eff)` with
  `t_eff` the located T-count, printed before any branch is evaluated; the executed branch
  count `N_exec` at acuity `ε = 0` equals `N_pred` exactly, and at `ε > 0` the executed count
  is reported as a fraction of it. **Kill:** `N_exec ≠ N_pred` at `ε = 0`.
- **S3 — the budgeted sum meets the acuity with a certificate.** The hard part is evaluated
  as a branch sum in a DECLARED order with a running remainder bound `R_k` (the bound on the
  sum of the branches not yet evaluated, from the decomposition's per-branch norm bound);
  evaluation stops at the first `k` with `R_k ≤ ε`; the result carries `(value, R_k, k, N)`.
  Stake: on every instance where a referee exists, `|value − referee| ≤ R_k ≤ ε`. **Kill:** one
  instance with `|value − referee| > R_k` — the certificate lied, and nothing is read
  until the bound is repaired.
- **S4 — cost follows the price, and the mesh pays where the work is.** Executed branches
  against `ε` follow `N_exec ≤ N_pred` with the fraction falling as `ε` rises, reported as a
  table; and the branch sum sharded across `S ∈ {1, 2, 4, 8}` cores by `holon::mesh` scales
  with wall `≤ 0.35 ×` at `S = 4` and `≤ 0.2 ×` at `S = 8` on `N ≥ 3^5` branches, bit-identical
  to `S = 1`. **Kill:** scaling worse than `0.5 ×` at `S = 8` — the branch sum is not the
  parallel section either, and the finding is where the time goes.
- **S5 — the comparison class.** The referee statevector's wall and the full branch sum's
  wall are reported beside the budgeted sum's at every `(n, t, ε)`; the published scaling
  `2^{0.396 t}` is the curve the full sum is checked against (measured exponent within
  `0.05` of it over `t = 8 … 28`). No claim of beating a named tool is made: the claim is
  cost against acuity with the price predicted and the remainder certified.

## 3. Plants, each of which must fire before an instance is read

| plant | must |
|---|---|
| **PQ-1** | a Clifford-only circuit (`t = 0`): the located hard sector is EMPTY, the price is `1`, the sum is one branch, and the answer equals the tableau tier's to the bit |
| **PQ-2** | a circuit whose T-gates all lie outside the observable's light cone: `t_eff = 0` after S1's removal, the answer equals the tableau tier's |
| **PQ-3** | the remainder bound on a synthetic branch set with KNOWN tails: `R_k` is never below the true remainder at any `k` (a bound, not an estimate), and is within `4 ×` of it |
| **PQ-4** | a planted wrong branch (one term's sign flipped): the `ε = 0` sum disagrees with the referee, and the disagreement exceeds `R_k` — the certificate is not vacuous |
| **PQ-5** | the mesh: a corrupted shard's partial fold is convicted by name; a scrambled shard order changes nothing |

## 4. Branches

- **(a)** S1–S5 met → the general QVM's claim is MEASURED on Clifford+T: the cheap part found, the
  hard part priced, only enough of it done, with a certificate; banked in BENCHMARKS.md and
  the stance; the next freeze is the same procedure on the ring tower's other tiers (face
  ring, qutrits) and the GPU branch fold.
- **(b)** S3 killed → the bound is wrong; repaired and re-frozen; nothing else read.
- **(c)** S4's mesh clause killed → the finding is the profile of the branch sum; banked.
- **(d)** S5's exponent outside `0.05` → the port does not reach the published rate; the
  measured exponent is the number and the gap is named.
- **(e)** a plant fails → nothing is read.

## 5. Cost

Exact statevector at `n = 20` is `2^{20}` amplitudes — seconds; the full branch sum at
`t = 28` is `3^6 · 2^2 = 2,916` branches of affine evolution on `n ≤ 24` — minutes on one
core; the sweep is under an hour on the spare E-cores `21–27`. No new dynamics.

## 6. What this does not test

Circuits outside Clifford+T; sampling (the observable is an amplitude or a marginal, exact);
noise; anything above `n = 24`, where the referee is gone and only the certificate speaks.

---
witness: `Closed` (Object.lean), `tableau_not_closed_under_rotation` and `tableau_closed_under_hadamard` (Stabilizer.lean) for S1's sector; none for S2–S5 (measured gates)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-CHEAPER-THAN-ITS-PRICE, M-TRUNCATION-AS-ERRORBAR, M-DEVICE-CLASS, M-PLACEMENT-LOTTERY, M-HOMOG, M-PARITY-PROTECT, M-VACUOUS-SUCCESS, M-FLOOR-UNSTAKED, M-MAINTENANCE-LENS — contacted by keyword, cited.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier (a Clifford circuit, a light-cone-disjoint circuit, a synthetic branch set, a planted branch, a shard), and the sector the plant acts on is nonzero in that carrier by construction.

---

### Notes on building

*Appended by the HARD-PART build (S2–S5, PQ-3–PQ-5; `engine/crates/holon/src/acuity.rs`,
`tests/qvm_acuity_hard.rs`) after the code existed and the numbers came in. No stake is
moved here; these are the places where the frozen text and the machine disagree, named so
the reading is not quietly adjusted later.*

1. **§3's "the sum of the remaining coefficients" is a true bound and a VACUOUS one.**
   Derived honestly from `magic5`: its terms are unnormalised (`γ = 1` on a support of
   `2^k`), so `|⟨y|φ_b⟩| ≤ ‖φ_b‖ = 2^{k/2}` and the a-priori bound is
   `|coeff_b|·2^{k/2}` with the gadget's `2^{t/2}` inside `coeff_b`. That makes
   `R_0 ≈ 61` at `t = 8` and it grows like `2^{t/2}` — never below any `ε` in the ladder,
   so the executed fraction under it is `1.000` at every `(n, t, ε)` measured. The reason
   is structural, not a bug: the bound knows only the NORM of a branch, while the
   observable is one amplitude of an `n`-qubit state and is `2^{−n/2}` sized, so the bound
   runs `2^{rank/2}` above the truth. The truncation this campaign actually measures comes
   from a second, tighter bound the build adds — `|coeff_b·γ_b|`, exact because an affine
   state's amplitude is `γ·i^p` on its support and `0` off it, `y`-independent, one branch
   evolution to compute. Both are implemented, both are reported, and S3's certificate
   holds under either. A future freeze should say WHICH bound a reported executed fraction
   was taken under; this one does not distinguish them.
2. **The `ε` ladder is not on the observable's scale.** `|⟨y|C|0⟩| ~ 2^{−n/2}` — `4·10⁻³`
   at `n = 16`, `10⁻³` at `n = 20` — so `ε = 10⁻¹` is a demand that the answer `0` already
   meets, and the budget correctly returns `k = 0` branches with a certificate of `5·10⁻²`.
   That is the machine working, not failing, but "executed fraction at `ε = 10⁻¹`" is not
   a measurement of anything at `n ≥ 16` unless the ladder is read relative to `2^{−n/2}`.
   §1 fixes `ε` absolutely and should have fixed it relative.
3. **S4's `S = 8` clause and §5's "spare E-cores 21–27" are inconsistent.** Seven cores are
   pinned, so `S = 8` over-subscribes them and the ideal floor is `1/7 = 0.143`, not
   `0.125`. Measured (min of 25, `ε = 0`, cores 21–27, box otherwise loaded):
   `n = 24, t = 24, N = 972` → `S=2 0.53`, `S=4 0.272`, `S=7 0.298`, `S=8 0.248`;
   `n = 20, t = 20, N = 324` → `S=2 0.54`, `S=4 0.277`, `S=7 0.333`, `S=8 0.311`.
   The `S = 4 ≤ 0.35` clause is MET; the `S = 8 ≤ 0.2` clause is NOT met and the kill
   (`worse than 0.5×`) is NOT triggered, so branch (c) is the reading of that one clause
   and the profile is the finding: `0.248` is `1.7×` off the seven-core floor.
4. **S5 does not say whether the measured exponent is the WALL's or the BRANCH COUNT's,
   and they differ by more than the band.** Over `t = 8…28` at `n = 12` (20n Clifford
   gates, full `ε = 0` sum, medians): the branch-count exponent is `0.3962` — the published
   `0.396` to four figures, gap `0.0000` — and the wall exponent is `0.4622`, gap `0.0659`,
   OUTSIDE the `0.05` band. The gap is accounted for exactly: the per-branch cost is not
   constant in `t`, because the gadget widens the register to `n + t` qubits and adds `t`
   CX gates, and its own fitted exponent is `0.0659`, with
   `0.3962 + 0.0659 = 0.4622` identically. So branch (d) fires on the wall reading and does
   not fire on the branch-count reading, and the honest sentence is: **the decomposition
   realises the published rate; the implementation's wall carries an extra `2^{0.066 t}`
   that is the gadget's register growth, not the decomposition's.**
5. **PQ-4 as written is satisfied by any nonzero disagreement**, because at `ε = 0` the
   remainder is `0`, so "the disagreement exceeds `R_k`" costs nothing. The build therefore
   also runs the plant at `ε ∈ {10⁻², 10⁻³, 10⁻⁴}` where `R_k > 0` and the plant is inside
   the evaluated prefix (`t = 12, ε = 10⁻²`: disagreement `2.34·10⁻²` against `R_k`
   `7.81·10⁻³`). Also: a sign flip on a branch whose affine support misses `y` plants
   NOTHING — that branch contributes exactly zero — so the plant's carrier has to be chosen
   (the first branch in the declared order that is live at `y`), which is what §3's
   carrier-sector statement demands and what a first pass gets wrong.
6. **S2's kill has one silent escape.** The stopping rule is "the first `k` with
   `R_k ≤ ε`", so at `ε = 0` it executes every branch only because every `magic5` branch
   bound is strictly positive. A decomposition with a zero-coefficient branch would give
   `N_exec < N_pred` at `ε = 0` with nothing wrong: the skipped branches contribute exactly
   zero. Measured here, `N_exec = N_pred` at `t ∈ {8, 12, 16, 20}` under both bounds, so
   the escape is named and not taken.
### Notes on building (cheap part)

*Appended 2026-09-21 by the half that built `engine/crates/holon/src/sector.rs`,
the `qvm_acuity` driver, S1 and plants PQ-1/PQ-2. Nothing above is moved;
everything below is what the instances turned out to be once the search was
pointed at them. Numbers are from `cargo test --release -p holon --test
qvm_acuity_cheap -- --nocapture` and from the driver.*

**1. An amplitude's light cone is the whole register, so `removed = 0` on every
amplitude instance — by construction, not by luck.** `⟨y|C|0⟩` reads every bit
of `y`, so the backward cone is seeded with all `n` qubits and no gate can ever
fall outside it. S1's removal clause is exercised by the MARGINAL only.
Measured on the prereg's grid (`n ∈ {12,16,20}`, `t ∈ {8,12,16}`, three seeds,
27 instances): `removed(amplitude) = 0` on all 27; `removed(marginal)` ran 0–7
for **76 T gates removed in total**, and the cone fell short of the full
register on 6 of the 27.

**2. The prereg's marginal is a CONSTANT at the prereg's depth.** At `20n`
Clifford gates the 4-qubit marginal `p(0000)` is `1/16` exactly on 25 of the 27
instances (9/9 at `n = 16` and at `n = 20`), and dropping any T gate anywhere in
the circuit moves it by at most `2.6 × 10⁻¹⁵`. The mechanism is exact, not
numerical: `p(0000) = (1/16)·Σ_{P ∈ ⟨Z₀,Z₁,Z₂,Z₃⟩} ⟨0|C†PC|0⟩`, and conjugating
each of the 15 nontrivial `P` backwards through the circuit gives a sum of
Paulis that all carry an `X` or a `Y` somewhere (a `T` only ever mixes `X` with
`Y` on its own wire), so every one of those expectations is exactly zero and
stays exactly zero when a `T` is deleted. S1's positive clause passes on this
observable, but it passes against a constant — M-VACUOUS-SUCCESS unless
something else carries the weight. Three things do, and all three are in
`engine/crates/holon/tests/qvm_acuity_cheap.rs`:

* the kill condition in its PER-GATE form — the prereg says "one instance where
  dropping *a* 'removed' gate changes the observable", so each of the 76
  removed gates is dropped ON ITS OWN and re-refereed, not only all at once
  (worst `Δ = 2.554 × 10⁻¹⁵` against the `10⁻¹²` stake);
* the NEGATIVE check taken on the AMPLITUDE, where dropping a kept `T` moves
  the observable by `1.2 × 10⁻³ … 2.4 × 10⁻²` — the same order as `|a|` itself —
  and fires on 27 of 27 instances;
* an ADDED family at Clifford depth `2n` (30 instances, 223 gates removed,
  worst `Δ = 4.4 × 10⁻¹⁶`), where `p(0000)` is `1/16` on NONE of the instances
  and ranges over `0 … 0.85`, so the positive clause has room to fail. Even
  there only 2 of 30 marginals are moved by any kept `T`: this observable is
  insensitive to the T-sector almost everywhere, which is a fact about the
  observable and not about the search.

**3. The light cone is SOUND but not COMPLETE, and the tail is where that
shows.** A `T` is diagonal, so a `T` standing after the last entangling gate
cannot change any computational-basis probability at all — yet its support is
squarely inside the cone and the search keeps it. First met as a test failure:
scanning the LAST kept gates for the negative check gave `Δ = 0` exactly at
`n = 16, t = 8, seed 1`. `t_eff` is therefore an UPPER BOUND on the relevant
T-count and the price built on it an upper bound on the price; neither should
be read as tight. A completeness test (a kept gate that provably cannot matter)
would be a different search — commuting the diagonal sector forward — and is
not staked here.

**4. `y = 0^n` is a vacuous declaration, and the skeleton's `y` can be too.** A
random Clifford+T state is supported on a coset, so `⟨0^n|C|0⟩` is exactly zero
on most of these instances — it is on the very first one, `n=12, t=8, seed 1`.
The tableau tier's sample of the circuit's Clifford skeleton
(`sector::skeleton_bitstring`: poly time, no referee) lands in the skeleton's
support but not necessarily in the full circuit's — at `n=12, t=8, seed 1` it
returns `0^12` and the branches cancel there EXACTLY. Both the driver
(`--amp-argmax`) and the stakes therefore declare `y` as the largest
`|⟨y|C|0⟩|` the referee shows wherever a referee exists, and the rule is stated
before the run rather than chosen after it.

**5. "Equals the tableau tier's to the bit" needed a reading, and got an exact
one.** The tableau tier produces a DISTRIBUTION, not an amplitude. PQ-1 and
PQ-2 therefore compare `|a|²` — computed exactly in the ring as `int · 2^{−m}`
with the `√2` part checked to be zero (`sector::cyc_prob_exact`) — against
`holon_qasm::run_tableau`'s probability, with `==` on `f64`. Both sides are
exact dyadic rationals, so `==` is the right operator; it holds on all five
PQ-1 seeds (`p = 0.03125, 0.015625, 0.03125, 0.0625, 0.0625`) and on PQ-2
(`p = 0.0625`).

**6. `holon-qasm` is a DEV-dependency of `holon`, so the driver cannot use
it.** Nothing under `holon/src/` — `src/bin/` included — can reach
`holon_qasm::magic::magic_amplitude`; the workspace manifest calls that
isolation gate load-bearing. The driver's full-sum fallback is therefore
`mesh::fold_amplitude` over `magic5::Magic5Source`, which IS §1's second
referee ("the FULL exact branch sum, all branches, exact `Z[ω]`") rather than a
substitute for it. The tableau-tier comparison lives in `tests/`, where the
dev-dependency is reachable, and the driver's own referee is the dense `2^n`
carrier.

**7. A marginal is not priced by an amplitude's budget.** A marginal on `|S|`
qubits inside a cone of `|L|` is a `2^{|L|−|S|}`-fold sum of amplitudes. The
acuity `ε` is an absolute error on ONE amplitude; propagated to a probability
the bound is `2|a|R + R²` per leg, which is what the driver reports, and the
`|L| − |S|` exponent is why the marginal's value comes back `null` above
`--marginal-cap`. S3's certificate as written covers the amplitude; the
marginal arm of the sweep reports the sector, the price and the removed count,
and says so.

And one thing the ε ladder does to the AMPLITUDE arm, which the sweep's table
now carries a legend for: `|⟨y|C|0⟩| ~ 2^{−n/2}` is `10⁻³` at `n = 20`, so at
`ε = 10⁻¹` the whole amplitude is already inside the acuity and a correct
budget stops before evaluating one branch — a row reading `evaluated = 0`,
`k/N = 0.000`, `value = 0` is the machine working. §1's ladder is ABSOLUTE
where a relative one (`ε · 2^{−n/2}`) would have made the three rungs mean the
same thing at every `n`; that is a property of the prereg, not of the budget,
and the hard half's notes say it from the other side.

**8. Where the cheap part's time actually goes — and one number S5 should
expect.** `locate` costs `0.2–0.6 ms` on a 400-gate circuit. The branch
source's CONSTRUCTION dominates the branch sum: at `n = 20, t = 28` the two
together are a median `1.39 s` against `0.035 s` for the fold alone, while the
statevector referee is `0.46 s`. So at the prereg's largest `(n, t)` with
`n ≤ 20` the full branch sum is SLOWER than the `2^n` carrier (and at
`n = 20, t = 8` it is 180× FASTER, `0.0025 s` against `0.45 s`) — the crossing
is near `t = 24`. The driver reports the walls separately
(`wall_locate_s`, `wall_source_s`, `wall_sum_s`, `wall_referee_s`) so the sweep
cannot report one and mean another. Whole sweep, 450 runs over
`n ∈ {12,16,20} × t ∈ {8…28} × 5 seeds × (ε = 0 plus 3 acuities)` plus the
marginal arm and both referees: `214 s` on cores 21–27, worst
`|value − referee|` anywhere `1.5 × 10⁻¹⁶`, 332 T gates removed across the
90 marginal instances.

**9. S5's exponent is TWO numbers and the table now prints both.** The BRANCH
count follows `expected_branches` exactly — `12, 36, 108, 324, 972, 2916` at
`t = 8 … 28`, a fitted slope of `0.3962` per T against the published `0.3963`,
gap `−0.0001`. The WALL slope on this machine is `0.459 / 0.460 / 0.457` at
`n = 12 / 16 / 20`, which against the published rate would read as a gap of
`+0.061 … +0.064` — OUTSIDE the prereg's `0.05`. But that excess is not the
decomposition's rate: it is the per-branch cost, which grows with `t` because
each branch evolves an affine state `n + t` wide.
`conformance/qasm/qvm_acuity_sweep.py` reports the branch slope, the wall
slope and their difference in one table, so branch **(d)** cannot be triggered
by charging the decomposition for the carrier's width. Whichever S5's reader
means, the number is there and it is labelled.
