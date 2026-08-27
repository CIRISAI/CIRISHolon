# Q7b PREREG — per-region certification on a three-level box, where the regimes are genuinely distinct

**Frozen 2026-08-23.** The mandatory spread pre-check (§0) is the only Q7b measurement that exists
at this commit; no chart error, no certification map, no candidate has been scored on the sweep.
Everything in Q5/Q7 that is not restated here **carries unchanged**.

---

## 0. THE SPREAD PRE-CHECK — run FIRST, and it is what licenses this prereg

House rule after Q5 and Q7: *before a prereg exists, measure whether the honesty boundary actually
varies along the axis the design stakes.* Two campaigns died for want of this; it does not get paid
for a third time. Instrument: `crates/q-seam/examples/q7b_spread.rs`, 16 probe points at N = 10.

**Headline, and it is the number that licenses Q7b:**

> **11 of 16 probe points are SPATIALLY SPLIT at the staked margin** (`min_r E_r ≤ 0.5` **and**
> `max_r E_r ≥ 2.0`). **Q7's trap managed 0 of 84.**

At the design point `V = 4, U = 1` the per-region error and the exact block densities are

```
E_r      =  0.00   0.06   2.71   0.06   0.00
density  =  0.02   1.96   1.05   1.96   0.02
             ↑      ↑      ↑      ↑      ↑
           empty  filled  MOTT  filled  empty
```

Three genuinely distinct regimes at one `(N, U)`, the chart-hard one isolated in the centre, and a
2.71-vs-0.00 split across two sites. That is the object Q7 failed to build.

**Honest note on the ratio statistic.** The raw spread `max/min` reads 554× here and up to 1.4e6 at
`V = 16`, because the wings drive `min → 0`. **A ratio with a vanishing denominator is not a
statistic**, and it is not used for anything: the gate is and remains the scale-free staked
criterion (`min ≤ 0.5` and `max ≥ 2.0`). The 4× rule-of-thumb was the *entry* test; the split
criterion is the operative one.

**What was scanned, so the family selection is auditable:** `V ∈ {2,4,8,16} × U ∈ {1,2,4,8}`, all
16 printed in the results log. This selects the FAMILY, not any threshold — every certificate
constant below is carried from Q5/Q7 unchanged, and none was touched by this scan.

---

## 1. TWO DERIVED CONSTRAINTS THE PRE-CHECK FORCED

### 1.1 N = 8 is impossible, by arithmetic

Regions are blocks of two sites, half filling fixes the particle count, and reflection symmetry
forces the block-density pattern to read the same both ways. At `N = 8` there are 4 blocks, so a
symmetric pattern is `(A, B, B, A)` with `2(2A + 2B) = 8`, i.e. **`A + B = 2`** — admitting only
`{0,2}` (two chart-easy regimes) or `{1,1}` (uniform). **No `N = 8` arrangement contains an empty,
a Mott and a filled region at once.** Enumerated: N = 8 → none; N = 10 → exactly two patterns
(`[0,2,1,2,0]` and its complement); N = 12 → six.

> **N = 8 is therefore OUT of Q7b — derived, not preferred.** `N = 10` is the family; `N = 12`
> stays optional (D0-c's memory cost is unchanged).

This is a real cost: Q7b has **one** required chain length, so nothing here can speak to
size-dependence. Declared, not discovered.

### 1.2 The step must stay reflection-SYMMETRIC — the off-center suggestion is declined

The attack proposed placing the step off-centre so D1b gets a real chance. **That would destroy
D1b, not test it.** D1b's anchor is `⟨n_i⟩ = ⟨n_{N+1−i}⟩`, which holds *because* `v` is
reflection-symmetric; an asymmetric potential kills it exactly the way the site potential killed
particle–hole (§2.1 of Q7), and it would take the G7-E9 mirror gate with it.

**D1b's real chance requires a symmetric potential in which the CHART nonetheless breaks
reflection** — degenerate left- and right-localised SCF solutions, which a symmetric double-well
box can support. So the box is symmetric, and D1b is tested the only way it can be.

---

## 2. THE FAMILY

**Symmetric three-level box, `N = 10`, half filling, `S_z = 0`, open boundaries:**

```
v = [ +V, +V, −V, −V, 0, 0, −V, −V, +V, +V ]
```

Outer blocks pushed empty, the two deep wells pulled to double occupancy, the centre block left at
`n ≈ 1`. Block boundaries coincide with the region boundaries by construction, so the step is sharp
on the region scale — the requirement the attack named.

**Grid — fixed now.** `V/t ∈ {1, 2, 3, 4, 6, 8, 12, 16}` (8 values) × `U/t ∈ {0, 0.5, 1, 2, 4, 8, 16}`
(7 values) = **56 configurations, 280 region-instances.** `V = 0` is *not* in the grid — it is Q5's
family, already measured; `U = 0` remains the control column where the chart is exact for any `v`.

**THE PLANT:** the **centre** block (the Mott region) at `U/t = 16`, at every `V`.

---

## 3. WHAT CARRIES UNCHANGED

Stated once so this document does not restate a hundred lines:

- **The anchors, exactly as derived in Q7 §2**: the spin pin `m_i = 0` (spin-independence + gated
  `S_z`-sector uniqueness) is PRIMARY; the reflection anchor `⟨n_i⟩ = ⟨n_{N+1−i}⟩` is the second
  primary; particle–hole is **not** an anchor and rides along as the two-Hamiltonian gate
  **G7-E9**, which measured **7.1e-13** across a potential in Q7 and is a genuinely good instrument
  for free.
- **Candidates D1, D1b, D2, D3** with their thresholds (`κ = 0.5`, `τ` carried from Q5), and D2's
  closed-form `σ_m² = U²·n↑(1−n↑)·n↓(1−n↓)`.
- **Per-region observables R1–R5** including block-restricted `D_bool`, and their tolerances.
- **The five-clause joint gate**, `G7-FIT` (≥ 8 of 56 split), the spatial-discrimination clause,
  the VOID budget (>12 of 56 ⇒ underpowered), and the **A3/R2 both-readings rule**.
- **Baselines N1, N2, N3, N4, N5** with **D0-b's oracle line**: N4 is one `U`-threshold per region
  fitted **jointly across all `V`**, and a parameter count that scales with the swept axis is the
  oracle, not a baseline.
- The exactness ladder, the free-fermion ruler (`free_reference`, exact at any potential), the
  seeded Lanczos and its restart policy, and the detached-compute rule.

---

## 4. THE NEW CANDIDATE — D4, and why it is a candidate and not a baseline

The pre-check exposes a design hazard I must name before running: **on this family, local density
almost gives the answer away.** Empty and doubly-occupied regions are near-product states where the
chart is near-exact; the Mott region is where it fails. A rule as crude as *certify iff the local
density is near 0 or near 2* would likely score well.

> **D4: certify region `r` iff `max_{i∈r} min(n^MF_i, |n^MF_i − 2|) ≤ 0.25`.**
> Threshold **STAKED** at 0.25 — a quarter of a particle from a determinate filling.

**It is a CANDIDATE, not a baseline, and the distinction is principled:** D4 reads **chart data**
(`n^MF`), whereas every baseline reads only the sweep **coordinates** `(region, U, V)`. That is the
line Q7 drew and it does not move because the answer became easier.

**But D4 has no theorem behind it.** It is a heuristic in a third class — neither theorem-pinned nor
a self-residual — and `SelfAudit` gives it no warrant at all. So:

> **P-D4 (STAKED): D4 scores well on this family, plausibly better than D3.** If it does, the honest
> reading is **"on this family a chart-internal heuristic suffices, and the theorem-pinned route is
> not necessary here"** — a real finding, reported in those words, and **not** a vindication of the
> certificate programme. If D4 beats D3 while both pass, the title line says so.

---

## 5. EVERY CANDIDATE'S FATE, STAKED

| candidate | class | staked fate |
|---|---|---|
| **D1** spin anchor | theorem-pinned | refuses the plant; **fails `FP = 0`** — certifies empty/filled regions correctly but also certifies the Mott region in the band where `\|m^MF\| ≤ 0.025` before breaking |
| **D1b** reflection anchor | theorem-pinned | **its one real test.** Expected still silent, but the symmetric double-well can support left/right-localised SCF solutions; if it fires anywhere it fires here. If it fires only where G-E6 VOIDs, the reading is **UNTESTED**, not null |
| **D2** self-residual | self-residual (control) | **CERTIFIES the plant** — `σ_m = 0` at the polarised core, transferring verbatim from Q7 where it was confirmed with FPs at 19.4 tolerances. Also **certifies the empty and filled regions correctly** (`σ → 0` there), so on this family D2 should look *good* except at the plant |
| **D3** = D1 ∧ D1b ∧ D2 | conjunction | passes `FP = 0` and the plant clause; **and this time is expected to pass clause 5**, because the wings and the Mott centre are genuinely different regimes rather than points on one ramp |
| **D4** density heuristic | heuristic (no theorem) | scores well, plausibly best; **and that is a finding about the family, not a certificate success** |

**Anti-shopping clause carried:** five candidates, every fate staked above, and **the results title
line names all five**, never only the survivor. A D3 pass that does not beat N3/N4/N5 is **CORRECT
BUT UNINFORMATIVE**; a D3-alone pass is **"the conjunction passed; neither component did"**.

---

## 6. THE MEANING OF EVERY OUTCOME

- **(a) D3 passes all five clauses and beats N3/N4/N5 and D4.** The theorem-pinned certificate earns
  its keep where the regimes are distinct — the result Q5 and Q7 could not produce.
- **(b) D3 passes but a baseline matches it.** CORRECT BUT UNINFORMATIVE, in those words.
- **(b′) D3 passes but D4 beats it.** The heuristic suffices here; the theorem-pinned route is not
  necessary on this family. Reported as a finding about the family. **Not** a certificate success.
- **(c) D3 fails clause 5.** Spatially blind even on a family built to be split — the strongest
  possible negative for per-region certification, because the family cannot be blamed.
- **(d) D2 refuses the plant.** Q7's closed-form derivation fails to transfer; title line.
- **(e) The kill fires (no candidate passes all five).** Per-region certification is decoration on a
  family explicitly built to favour it. Dead, marked — and a much stronger kill than Q7 could have
  delivered.
- **(f) G7b-FIT fails (< 8 of 56 split).** Q7b VOID. **This would also convict the pre-check**, since
  11 of 16 probe points split — so the reading would be that the probe grid was unrepresentative,
  and the pre-check rule itself needs strengthening. Named now.
- **(g) D1b fires somewhere real.** The reflection anchor is live; the primary class genuinely has
  two members, and Q7's "untested" upgrades to a measurement.

## 7. KNOWN HOLES

1. **One chain length** (§1.1), so nothing here addresses size-dependence.
2. **The family was selected by the pre-check.** That selects the family, not any threshold — but it
   does mean Q7b is run on a family chosen *because* it splits, and a certificate succeeding here
   has not been shown to succeed anywhere else. Scope accordingly.
3. **The regimes may be too easy** — §4's hazard, which is why D4 exists and why (b′) is written in
   advance.
4. `V` and `U` grids are stakes; no post-hoc grid extension.
5. Tolerances still carried from Q5, still not derived.

---

# AMENDMENT A1(Q7b) — adversarial review, adopted 2026-08-23, before the Q7b instrument exists

Four items, all adopted. The pre-check (§0) remains the only Q7b measurement in existence; nothing
below was measured before staking, and the two new predictions are derived, not observed.

## A1(Q7b)/W — the WHY, in the attack's words

> **A family built to have distinct regimes is what the crystal-tier seam looks like, so a
> heuristic winning there is itself a design answer.**

That is the honest version of the question this whole path exists to answer, and it is why §4's D4
is not a threat to be managed but a result to be read.

## A1(Q7b)/P-D4-COVERAGE — the derived reason D4 cannot own the headline

**Derived, before any Q7b chart error exists.** D4 certifies on density extremity. But density
extremity is a **SUFFICIENT route to local determinacy, not the criterion** — and `U → 0` is a
second route to the same place that D4 is structurally blind to. At `U = 0` **every** region is
chart-exact for any potential whatever, including regions whose density sits at an undramatic
intermediate value; and at shallow `V` (1–2, against a bandwidth of `4t`) the profile cannot be
driven to the 0/2 extremes, so intermediate densities are exactly what the `U = 0` column contains.

> **P-D4-COVERAGE (STAKED): D4 FAILS clause 3** — it refuses honest `U = 0` regions at low `V`, by
> construction, because it cannot see the non-density route to determinacy.

**The consequence, which is why this stake matters more than the philosophy:** if the derivation
holds, **"a heuristic beats the theorems" can never be the headline**, because the heuristic dies on
a clause D1, D1b and D3 all pass. Outcome (b′) is thereby demoted from a shrug to a mechanism: D4
may well out-cover D3 *among the clauses it survives*, and still fail the gate on the one clause
that asks whether it understands what it is certifying.

**And if D4 survives clause 3**, the derivation is wrong in a specific, informative way: it would
mean every `U = 0` column of this family already sits at density extremes — a fact about the family
worth a line, and reported as one.

## A1(Q7b)/P-D4-D1b-COMPLEMENT — D4's blind spot has a theorem-pinned partner already on the sheet

D4 reads the **chart's** density, so it inherits `SelfAudit.error_not_computable_from_chart` in
full: its verdict is hostage to the chart being right about the very quantity it reads. Where could
the chart lie about density on this family? The one candidate is the **reflection-broken SCF pair**
a symmetric double well can support — asymmetric charge where the truth is reflection-symmetric.
**That is precisely D1b's firing domain**, and D1b's warrant for it is a theorem.

> **P-D4-D1b-COMPLEMENT (STAKED): D4's false positives, if any, concentrate on reflection-broken
> configurations, and D1b catches exactly those.** If confirmed, D4 and D1b are C1-and-C3 again —
> complementary blindnesses, one heuristic and one theorem-pinned — and the composite is
> pre-registered here rather than assembled afterwards:

> **D5 ≡ D4 ∧ D1b.** No new constant (both thresholds already frozen). Scored as a candidate.

Note the conditional honesty: P-D1b already stakes that D1b is probably silent, in which case D5
reduces to D4 exactly and this pairing costs nothing and shows nothing. The pairing is registered
because it is the *right* pairing if D4 has FPs at all — **the answer to a strong crude candidate is
to derive its blind spot, not to weaken it.**

**Six candidates now (D1, D1b, D2, D3, D4, D5), all fates staked, and the results title line names
all six.**

## A1(Q7b)/S — scope sentence added to Known Hole 2

> **Q7b tests whether the machinery works WHERE THE QUESTION EXISTS.** Transfer to families nobody
> tuned is a separate claim needing its own campaign — the natural out-of-family test being the
> engine's own tiers, where nobody chooses the potential. Neither a success nor a failure here may
> be read as settling that.

## A1(Q7b)/D — optional diagnostic: post-hoc-fitted D4

A density window `[0, w] ∪ [2−w, 2]` with `w` **fitted after results**. **Labelled a DIAGNOSTIC, not
a baseline**, and the reason is D0-b's own line: it reads **chart data** with **fitted** parameters,
which is outside the N-class (coordinates only, bounded parameters) in both respects. Its single
legitimate use is to answer *did the staked `w = 0.25` matter?* — and it can never appear in a
verdict, a clause, or a comparison against D3.
