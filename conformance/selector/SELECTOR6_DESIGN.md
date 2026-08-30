# SELECTOR-6 — INSTRUMENT DESIGN

*Written by the selector5-refuter lane against `SELECTOR6_PREREG.md` (frozen and
admitted, commit 28ca6ae), BEFORE any instrument exists and before any number in
the campaign's scope has been computed. Nothing here adds, weights or thresholds a
gate. Where the freeze delegates a construction, the construction is stated in full
with the reason it is forced — a delegated degree of freedom that is not written
down is a fitted parameter.*

**Status: awaiting the lead's re-audit. Nothing runs until it lands. Two rulings are
requested in §9 and must be answered before the primary, because each of them is a
mid-run VOID if it is discovered rather than decided.**

---

## 0. Pins

| object | pin |
|---|---|
| criterion instrument | `conformance/omega/selector4.py`, blob `d33f0469`, sha256 `0c175215…f8e82a0e` |
| RULE-B label | `conformance/selector/refute_lib.py`, blob `5ddebd48`, sha256 `7444ddeb…6d047a33` |
| labelling rule text | `conformance/selector/FROZEN_LABEL_RULE.md`, sha256 `0a4b9565…3bad0d913` |
| refutation of record | `conformance/selector/SELECTOR5_REFUTATION.md` |

`selector4.py` is **imported, never reimplemented**. Its `build_world`,
`knobs_partial_sections`, `separates`, `closed_at`, `cycles_of`, `inv_perm`,
`acts_are_bijections` and `knobs_view_aligned` are used verbatim. SELECTOR-6 adds a
census, a family constructor, an aggregation layer and the gates — it changes no
line of the criterion. Drift between "the frozen criterion" and "what runs" is
M-STALE-INSTRUMENT, and importing at a pinned blob is the only way to foreclose it.

---

## 1. The criterion, restated exactly as inherited

For a finite group `G` given by `(MUL, INV)` alone:

- **World.** State space `G × G`, `N = |G|²`, state `i = a + |G|·b` ↔ `(a, b)`.
- **Step.** `T(a,b) = (a, ab)`, `S(a,b) = (b, bab⁻¹)`, `step = S ∘ T`. Verified
  bijective; VOID otherwise.
- **Acuity ladder**, cumulative joins `V[k] = join(R₀ … R_k)`:
  `R₀` constant · `R₁` `[a,b] = e` · `R₂` `class([a,b])` · `R₃` `(class a, class b)`
  · `R₄` simultaneous-conjugation orbit. Chain verified by `refines` at every rung.
- **Gauge.** `GAUGE[g](a,b) = (gag⁻¹, gbg⁻¹)`; each verified bijective.
- **Act vocabulary.** Invariant partial sections: the fiber-injective `step`-cycles
  of the home candidate, carriers verified `step`-invariant.
- **Separation.** The exact ρ-closure with no word-length cut; budget exhaustion
  returns `None` and VOIDs the cell.
- **Identity set.** `Ident(i) = { j : i is not separated from j at V[k] }`.
- **Selected set at rung k.** `sel_k = { i : |Ident(i)| = 1 }`.

Every one of these is a function of `MUL`/`INV`. No presentation, generating set,
name or family tag appears anywhere in the list. In particular **the step is
canonical — there is no step choice to make class-functional.** The mapping-class
step is written in the group operation itself. The only construction in
SELECTOR-4 that involved a non-canonical choice was the *dressing*, and §2 removes
it entirely rather than aggregating over it.

### 1.1 The one operational reading the freeze delegates

The freeze says: *"selected = passing a proper nonempty subset test at the finest
decided rung."* Pinned here, before anything runs:

> **`k*(G)` = the finest rung `k ∈ {0,…,4}` at which the run recorded zero VOIDs.**
> **`SELECT(G) := 0 < |sel_{k*}| < |F(G)|`.**
>
> If no rung is decided, `G` VOIDs (B1) and enters every table as VOID.
> `A₀` is always decided (it is analytic: `sel₀ = ∅`), so `k*` always exists and
> `SELECT(G) = False` on a group decided only at `A₀` — recorded, with `k* = 0`
> shown in the per-group record so the reader can see it was the coarse rung
> talking.

`|sel|` and `|F(G)|` are both counted over the **full** family of §2, not over
orbit representatives. `sel` being empty and `sel` being everything both make
`SELECT` false, and the record distinguishes them by printing `(|sel|, |F|)` for
every group rather than the boolean alone.

---

## 2. Generalization: the canonical family, and why I1 holds by construction

SELECTOR-4's family was

```
F_reps(G) = { GAUGE[r] ∘ step^d : d | ord(step), r one representative per class }
```

**That set is not canonical, and it is the whole of the I1 risk.** Changing the
representative of a single class changes the set, and it changes it by conjugating
*one* member rather than all of them, so no single gauge element carries the old
family to the new one. Any verdict computed on `F_reps` is therefore a verdict about
a choice, and the refutation's founding scar — a criterion whose answer depends on
how the group was written down — would reappear in the successor.

SELECTOR-6 uses instead the **full dressed family**

```
F(G) = { GAUGE[g] ∘ step^d : g ∈ G, d | ord(step) },  deduplicated as permutations.
```

`F(G)` is a function of `(MUL, INV)` with no choices in it at all.

**Lemma (equivariance).** Let `x ∈ G` and write `γ = GAUGE[x]`. Then
`γ` permutes `F(G)`, and for all `i, j ∈ F(G)`,
`separates(i, j) = separates(γ·i, γ·j)`. Consequently `Ident(γ·i) = γ·Ident(i)`,
`|Ident|` is constant on gauge orbits, and **`sel` is a union of gauge orbits.**

*Proof.* Three facts, each already asserted and CHECKED inside `run_rung` and
re-checked here: (i) the view `V[k]` is gauge-invariant, (ii) the gauge action
commutes with `step`, (iii) `GAUGE[x]` is a bijection. From (ii),
`γ (GAUGE[g] step^d) γ⁻¹ = GAUGE[xgx⁻¹] step^d ∈ F(G)`, so `γ` permutes `F(G)`.
The act vocabulary of the home candidate is its set of fiber-injective `step`-cycles;
`γ` carries cycles of `T_i` to cycles of `γ T_i γ⁻¹` and preserves fiber-injectivity
because the view is gauge-blind by (i), so `γ` carries the vocabulary of `i` to the
vocabulary of `γ·i`. The ρ-closure is generated by `ρ ↦ T' ρ T⁻¹` and
`ρ ↦ a ρ a⁻¹`; conjugating every letter by `γ` conjugates the whole reachable set,
and the failure test `view ∘ ρ ≠ view` is preserved because the view is
gauge-blind. ∎

**Corollary (I1 by construction).** Let `φ : G → G'` be an isomorphism. `φ×φ`
carries the world of `G` to the world of `G'`, intertwines the steps, carries each
`R_k` partition to `R_k`, and carries `F(G)` **onto** `F(G')` — onto, because `F`
ranges over *all* `g`, not a chosen representative. Hence it carries `sel` onto
`sel'` and `|sel| = |sel'|`, `|F| = |F'|`, so `SELECT(G) = SELECT(G')`. There is no
aggregation to perform and no representative to argue about: the freeze's
requirement is met by deleting the choice rather than by averaging over it.

### 2.1 The economy the lemma buys back

`|F(G)|` reaches 3600 at `A₅` (measured: `|G| = 60`, `ord(step) = 8400`, 60 divisors),
and 3600² pairs is infeasible — the same wall SELECTOR-4 hit at `2I`, which it
climbed by choosing representatives. The lemma lets SELECTOR-6 have the economy
*without* the choice:

- Compute `Ident(i)` only for `i` ranging over **gauge-orbit representatives** of
  `F(G)` (at `A₅`: 300 rather than 3600), with `j` ranging over **all** of `F(G)`.
- `|sel| = Σ { |orbit(i)| : i an orbit representative with |Ident(i)| = 1 }`.
- `|F(G)| = Σ |orbit(i)|` over the same representatives.

Both totals are properties of `F(G)`. Which representative is picked from each
orbit affects only the order of the loop, and the lemma says the summand does not
change — which is **asserted as a run-time check on a declared sample**, not
assumed: for `S` sampled orbits per group (`S = 3`, or the whole orbit when it is
smaller), `|Ident|` is recomputed at a second, randomly chosen representative and
must agree exactly. Disagreement is an instrument bug and VOIDs the group loudly.

Measured family sizes, for the budget in §5 (`|F|` is the full dressed family,
already deduplicated):

| G | \|G\| | N | ord(step) | #div | #classes | \|F\| |
|---|---|---|---|---|---|---|
| Z₁₂ | 12 | 144 | 24 | 8 | 12 | 8 |
| Q₈ | 8 | 64 | 6 | 4 | 5 | 16 |
| D₈ | 8 | 64 | 6 | 4 | 5 | 16 |
| A₄ | 12 | 144 | 48 | 10 | 4 | 120 |
| Δ(27) | 27 | 729 | 8 | 4 | 11 | 36 |
| F₂₁ | 21 | 441 | 112 | 10 | 5 | 210 |
| Dic₅ | 20 | 400 | 60 | 12 | 8 | 120 |
| 2T | 24 | 576 | 48 | 10 | 7 | 120 |
| S₄ | 24 | 576 | 144 | 15 | 5 | 360 |
| D₆₂ | 62 | 3844 | 30 | 8 | 17 | 496 |
| Z₆₀ | 60 | 3600 | 120 | 16 | 60 | 16 |
| A₅ | 60 | 3600 | 8400 | 60 | 5 | 3600 |

---

## 3. The census: complete by construction, gated by A000001

### 3.1 The completeness argument

Every group of order `n` with a proper quotient of prime order — equivalently,
every non-perfect group — has a normal subgroup `M` of prime index `p`, and is
therefore a **cyclic extension** of a group of order `n/p` by `Z_p`. Enumerating all
such extensions of all groups of order `n/p`, for every prime `p | n`, is therefore
**complete** for the non-perfect groups of order `n`.

The only perfect group of order `2..63` is `A₅` (order 60). It is added by hand,
with its perfection verified (`G' = G`) rather than asserted.

So the census is built recursively from the trivial group, and its completeness is a
theorem plus one named exception, not a hopeful list of families. This is the whole
reason the campaign can honestly say "every isomorphism type of order 1..63" where
SELECTOR-5 said "all standard families".

### 3.2 The cyclic-extension constructor

Given `N` of order `m` and a prime `p`, every extension `1 → N → G → Z_p → 1` is
realised as `G = { n t^i : n ∈ N, 0 ≤ i < p }` with

```
(n t^i)(m t^j) = n · α^i(m) · [c if i+j ≥ p else e] · t^{(i+j) mod p}
```

for `α ∈ Aut(N)` and `c ∈ N` satisfying `α(c) = c` and `α^p = conj_c`. Both
conditions are checked, never assumed; associativity of the produced table is
verified exhaustively before the group is admitted (§6, plant (ii)'s refusal half).

Cost control, in the order applied, none of which can drop an isomorphism type
because each is applied *after* the table is built and only merges duplicates:
`Aut(N)` is enumerated once per `N` and cached; candidate `(α, c)` pairs are
pre-filtered by the two conditions; produced tables are bucketed by the invariant
fingerprint of §3.3; explicit isomorphism search runs only inside a bucket.

`|Aut(N)|` is largest at `N = Z₂⁴` (`|GL(4,2)| = 20160`); with `|N| = 16` choices of
`c` that is 322,560 candidate pairs for the worst single `N`, before the `α^p = conj_c`
filter. This is the campaign's dominant construction cost and it is bounded and
declared.

### 3.3 Deduplication

Fingerprint: `(order, sorted element orders, sorted class sizes, abelianization
invariants, |Z(G)|, derived length, sorted sizes of the derived series)`. Explicit
generator-image isomorphism search on **every** collision, with full
multiplication-table verification of any bijection found — the routine already
written and exercised in `refute_dedup.py`.

**The regression demonstration is the refutation's own three pairs**: `A₄ ≅ Δ(12)`,
`D₁₂ ≅ Z₂×D₆`, `D₂₀ ≅ Z₂×D₁₀`. The census must count each of them once, and the
`landscape_sweep.py` builders that produced them are used as an adversarial input
to prove it does.

### 3.4 S1 — the completeness gate

`A000001(n)` for `n = 1..63` is pinned as `conformance/selector/A000001.tsv` with a
provenance line naming its source. For each order the census count must **equal**
the pinned value; any order that disagrees is VOID for every claim in the campaign,
named loudly in every table, and no landscape statement covers it.

The pinned file is itself audited, because a pinned constant that nothing checks is
just a number someone typed. Two independent internal checks:

- **Abelian rows, exactly.** The number of abelian groups of order `n` is
  `∏_p P(a_p)` where `n = ∏ p^{a_p}` and `P` is the partition function. The census's
  abelian count must match this for every `n` — a closed form the census cannot
  cheat, covering every order.
- **Squarefree rows, exactly.** For squarefree `n` the group count is given by
  Hölder's formula, checked independently for every squarefree `n ≤ 63`.

A disagreement between the pinned sequence and either internal check is a VOID of
the *pin*, escalated rather than patched.

---

## 4. Labelling

RULE-B only, from `refute_lib.py` at the pinned sha: `SM(G)` iff there are a
degree-3 determinant-1 representation, a degree-2 determinant-1 representation and a
linear character of `G` whose kernels intersect trivially — i.e. iff `G` embeds in
`SU(3) × SU(2) × U(1)`. Character tables by Burnside–Dixon, validated per group by
degree-sum, integrality and column orthogonality; a group whose table fails
validation VOIDs rather than being labelled.

Construction tags are not merely unused — the census carries none. A group entering
the criterion is a `(MUL, INV)` pair and an opaque serial number. Its human-readable
name is attached to the *record* after the verdict is computed, by fingerprint
lookup, and is never available to any scoring function.

---

## 5. Budgets (B1)

Declared in the runner's header, and every exhaustion is a loud VOID that appears in
every table:

| budget | value | scope |
|---|---|---|
| `BUDGET` | 2000 ρ-nodes | per separation pair — **inherited unchanged** from `selector4.py` |
| `GROUP_PAIR_BUDGET` | 4,000,000 | separation pairs per group summed over all rungs |
| `GROUP_WALL` | 1800 s | per group, checked between rungs and between orbit reps |
| `ORBIT_CHECK_S` | 3 | orbit representatives re-checked per group for §2.1 |

Exhausting `BUDGET` VOIDs the cell, hence the rung, hence removes that rung from
`k*`. Exhausting `GROUP_PAIR_BUDGET` or `GROUP_WALL` VOIDs the group. A group that
VOIDs is never silently absent: it appears as VOID in the census table, the E1 table
and the per-group record, and it is excluded from both the numerator and the
denominator of E1 with its count stated.

---

## 6. Plants, and how each is demonstrated before the primary

- **(i) tag blindness.** The census is re-run with every name and serial permuted by
  a random bijection, and the full verdict vector must be **bit-identical**. Carrier:
  the verdict vector, asserted nonempty before scoring. This plant is cheap and total
  here because the criterion's inputs are `(MUL, INV)` — the plant proves the wiring,
  not the theory.
- **(ii) the isomorphic-pair plant, both halves.**
  *Re-presentation:* `A₄`/`Δ(12)` (mandatory) plus at least seven more pairs — the
  other two refutation pairs and five groups re-presented by a random relabelling of
  their multiplication table — must receive bit-identical verdicts *and identical*
  `(|sel|, |F|)`, not merely the same boolean.
  *Refusal:* a table with one entry corrupted, verified to break associativity or an
  inverse, must be REFUSED by the census validator and never reach the criterion.
  Carrier: both outcomes demonstrated and printed.
- **(iii) null integrity.** The E1 permutation null is run against a label vector
  that is itself a uniform random relabelling; it must report no enrichment at the
  staked threshold in ≥ 99% of trials. Carrier: the null's draw distribution,
  asserted non-degenerate.

A plant that does not fire, or fires on an empty sector, VOIDs the campaign — it is
not written up as a curiosity.

---

## 7. T1 — the theorem audit, in operational form

Both predictions run **inside** the primary, on every group, and a violation VOIDs
for diagnosis and is never reported as data.

- **Stasis half.** For every abelian `G`, `SELECT(G)` must be `False`. See §9,
  ruling 1 — the freeze's wording and the criterion's mechanism need one sentence
  from the lead before this is coded.
- **Orientation half.** For every ambivalent `G` (every class equal to its own
  inverse class, checked directly), the oriented sector must be empty, in this exact
  form: the map `(a,b) ↦ R₂(a,b) = class([a,b])` must be invariant under commutator
  inversion, i.e. `class([a,b]) = class([b,a])` for all `a,b`. For an ambivalent group
  this holds because `[b,a] = [a,b]⁻¹`; the check is that the instrument's `R₂` sees
  it. Witness: `frob_not_ambivalent` (`lean/CIRISHolon/FrobOrient.lean`) supplies the
  contrast case — `F₂₁` is in the census at order 21 and must have a **non**-empty
  oriented sector, so the audit has a live positive control and is not vacuous.

The stasis half has no Lean brick; the freeze names it as this campaign's owed
formalization, and the design does not pretend otherwise.

---

## 8. E1 — the stake

Statistic: the RULE-B fraction among `{G : SELECT(G)}` versus the eligible
population `{G : SELECT(G) decided, not VOID}`. Null: `10⁵` draws (≥ the staked
`10⁴`) of equal-size subsets of the eligible population, **seed 20260830**, staked
here. Branch (a) iff the observed fraction exceeds the null's 99th percentile.
Branch (b) otherwise, written up at the same length and in the same place.

Reported alongside, always, because the refutation's founding omission was a base
rate: the eligible-pool rate itself, the survivor count, the VOID count, and recall
(`|SELECT ∧ SM| / |SM|`) — the half of the ledger SELECTOR-5 never printed.

No thinning statement of any kind is computed or reported (M-CONJUNCTION-MONOTONE).
There is no gate sequence in this campaign to be monotone about.

---

## 9. Two rulings requested before the primary

**Ruling 1 — what "every abelian group selects nothing" means.** The freeze states
the stasis prediction as *selects nothing*. Traced through the inherited criterion,
an abelian `G` behaves like this: `GAUGE` is trivial, so `F(G) = { step^d }`; the
commutator is constant, so `V[0] = V[1] = V[2]` is the one-block view, where
`sel = ∅` analytically; and conjugation is trivial, so `V[3] = V[4]` is the
**discrete** view, where every candidate separates from every other and
`sel = F(G)` — everything identified. `SELECT(G) = False` either way, since neither
`∅` nor the whole family is a proper nonempty subset. But an audit coded literally as
`assert sel == []` VOIDs the entire abelian half of the census at rung `A₄`.

I propose T1's stasis half be read as **`SELECT(G) = False` for every abelian `G`**,
with the per-group record printing `(k*, |sel|, |F|)` so the mechanism is visible
and the "selects everything at the discrete view" behaviour is on the page rather
than hidden inside a boolean. I will not code either reading until the lead rules.

**Ruling 2 — the provenance of the A000001 pin.** The freeze requires the sequence
"pinned as a data file with its own provenance line". I can supply it from the
published classification, but a sequence I type from memory is exactly the kind of
unchecked constant this programme distrusts, so the design puts two independent
internal audits on it (§3.4: the exact abelian count for every order, and Hölder's
formula for every squarefree order). What I need from the lead is whether that is
the accepted provenance, or whether the pin must come from a fetched primary source
— and if the latter, whether this box is permitted to make that request.

---

## 10. Build order

1. This document, committed, re-audited. **Nothing below starts before that.**
2. `census.py`: cyclic-extension constructor, `A₅`, fingerprint + isomorphism dedup,
   associativity validator. Run the S1 gate. Commit census + counts + VOID list.
3. `plants.py`: (i), (ii) both halves, (iii). Each fires or refuses as staked, is
   shown, and only then is trusted. Commit.
4. `selector6.py`: the runner — imports `selector4.py` at its pinned blob, builds
   `F(G)`, computes gauge orbits, aggregates, runs T1 inline. Primary run under §5's
   budgets, detached (`setsid`, done-markers, `RESUME.md`), niced behind SCHWINGER-3
   and the referee pools.
5. Commit the record and the E1 branch verdict, branch (b) at branch (a)'s
   prominence.
6. `H1`: commit the held-out predictions — orders 65..71 complete, plus the named
   swamp panel (both extraspecial groups of order 32, both of order 64, `Q₆₄`,
   `Q₁₂₈`) — **before any held-out number exists**. Then run and score.
