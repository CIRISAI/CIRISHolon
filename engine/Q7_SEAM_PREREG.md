# Q7 PREREG — the certificate in its habitat: per-region certification over an inhomogeneous quantum reference

**Frozen 2026-08-23, before any instrument exists.** No crate change, no ground state, no chart, no
error, no certification map has been computed for Q7. Every number below is (a) derived here in
closed form, (b) a threshold **STAKED** and hereby fixed, or (c) a forward prediction, marked **P-**.

**Why Q7 exists, in one paragraph.** Q5 delivered a working refusal and then convicted it: `C4`
passed the joint gate but a post-hoc cutoff `U/t ≤ 0.25` beat it, because the chart-honest boundary
sat at `U/t ≤ 0.25` at *every* N — the per-site error is intensive and converges, so the sweep's
second axis never moved. **A certificate can only beat a cutoff where the honesty boundary varies.**
Q7 puts the same quantum seam in a world where it varies *by construction*: a spatially
inhomogeneous chain, where at **fixed (N, U)** the chart is exact in one region and lying in
another. The boundary becomes spatial, and no global cutoff can serve it.

**The engine tie, which is the point of the output shape.** Per-region certification over a quantum
substrate is the design datum for the crystal tier's seam: *where mean-field suffices, light it;
where correlation bites, refuse and refine.* Q7's deliverable is therefore not a scalar verdict but
a **refusal map** — a Certified/Refused label per region per configuration — readable directly as
that policy. GrainFloor's world (identical shards, different certification cost) is the same shape
one level up.

---

## 0. DECLARATIONS UP FRONT

**D0-a — the PRIMARY candidate class has exactly one member, and that is a real limit on Q7.**
§2 derives which anchors survive a site potential. The answer is: **one**. Particle–hole pinning of
`⟨n_iσ⟩ = 1/2` **dies**; the spin pin `m_i = 0` **survives**. Since `Core/SelfAudit.lean` makes
theorem-pinned anchors the primary class and self-residuals the expected-failure control, Q7 tests
a class with a single strong member. Declared now rather than discovered later.

**D0-b — the severity baseline class is RESTRICTED, and the restriction is declared.** An
unrestricted post-hoc function of `(region, U, a)` is a lookup table over the exact coordinates of
every answer — it is the **oracle**, not a baseline, and it would win by memorisation against any
certificate whatever. §8 therefore restricts baselines to **threshold rules in `U` with a bounded
parameter count**, and gives the strongest of them (N4) *more* free parameters than the certificate
has (which is zero). Beating N4 would be strong; losing to it is outcome (b) in spatial dress.

**D0-c — N = 12 is optional, with the reason.** `C(12,6)² = 853 776`; full-reorthogonalization
Lanczos at ~250 iterations holds ~1.7 GB of Krylov basis on a shared box. N ∈ {8, 10} is the
required family; N = 12 runs only if the box is free, and its absence changes no verdict.

**D0-d — a correction to Q5's warrant (not its result).** `Q_SEAM_PREREG.md` §1.1(iii) justified
`m_i = 0` by **Lieb 1989**. That is heavier machinery than the fact needs: §2.2 below derives it
from spin-independence plus ground-state uniqueness alone — no bipartite lattice, no half filling,
no particle–hole. Q5's *result* stands unchanged; its *warrant* was over-strong, and the weaker
derivation is what makes the anchor survive into Q7 at all.

---

## 1. WHAT CARRIES OVER

The Q5 instruments carry unchanged and are not re-derived: the spin-factorised Hamiltonian
(`H = T ⊗ I + I ⊗ T + U·D`, largest object 252 × 252), the seeded Lanczos with its residual ladder
and restart policy (A1/P1), the dense cross-check in its A2 Rayleigh-quotient form, the exactness
ladder G-E1…G-E6, the UHF chart with its three pinned guesses, and gates G-C1 (SCF convergence),
G-C2 (idempotency **checked**, not assumed) and G-C3 (variational).

**Tolerances carry over unchanged** where the observable is the same — `τ_n = 0.02`, `τ_d = 0.02`,
`τ_m = 0.05`, `τ_bond = 0.02` — and `κ = 0.5` carries over as the single safety factor. **No
tolerance is re-tuned for Q7.** One new tolerance is needed (§6, the block Boolean defect) and is
staked at the same scale as Q5's global one.

---

## 2. THE ANCHOR DERIVATION — done before anything runs, because A2 already paid for the alternative

`Core/SelfAudit.lean` says a certificate must read the chart against **theorems the world is known
to satisfy**. Under inhomogeneity the available theorems change, and an anchor claimed without its
inhomogeneous derivation is exactly the category error A2 recorded. So: derive first.

The Hamiltonian, with `t = 1` throughout:

```
H(v) = −t Σ_{i,σ} (c†_{iσ}c_{i+1,σ} + h.c.)  +  U Σ_i n_{i↑}n_{i↓}  +  Σ_i v_i (n_{i↑} + n_{i↓})
```

### 2.1 Particle–hole pinning DIES — derived

Apply the bipartite particle–hole map `c_{iσ} → (−1)^i c†_{iσ}`, under which `n_{iσ} → 1 − n_{iσ}`:

- hopping: **invariant** (the `(−1)^i` is what buys this);
- interaction: `U Σ(1−n_↑)(1−n_↓) = U Σ n_↑n_↓ − U Σ_i n_i + U·N`;
- potential: `Σ_i v_i (2 − n_i) = −Σ_i v_i n_i + 2Σ_i v_i` — **the potential term flips sign.**

At half filling `Σ_i n_i = N` is a constant on the sector, so the interaction's linear piece is the
constant `−U·N`, and altogether

> **`H(v) → H(−v) + 2 Σ_i v_i`**, hence **`E₀(v) = E₀(−v) + 2 Σ_i v_i`** and
> **`⟨n_i⟩_v = 2 − ⟨n_i⟩_{−v}`**, exactly.

Particle–hole is therefore a symmetry **iff `v ≡ 0`**. For any non-trivial potential `⟨n_{iσ}⟩ = 1/2`
is **not pinned**, and Q5's O3 anchor is gone. What survives is an identity relating **two different
Hamiltonians** — worthless as an anchor, excellent as an **exactness gate** (G7-E9 below), and it is
demoted to exactly that.

### 2.2 The spin pin SURVIVES — derived, on weaker hypotheses than Lieb

Let `F` be the global spin flip `↑ ↔ ↓`. For any **spin-independent** `H` (the site potential is
spin-independent, so this includes every member of the Q7 family), `[H, F] = 0`, and `F` maps the
`S_z = 0` sector to itself. If the ground state `|ψ⟩` is **unique in that sector**, then `F|ψ⟩` is a
ground state of the same energy, so `F|ψ⟩ = λ|ψ⟩` with `λ = ±1` (as `F² = 1`). Then for every site

```
⟨ψ| n_{i↑} |ψ⟩ = ⟨Fψ| F n_{i↑} F† |Fψ⟩ = λ² ⟨ψ| n_{i↓} |ψ⟩ = ⟨ψ| n_{i↓} |ψ⟩
```

> **`m_i ≡ ⟨n_{i↑} − n_{i↓}⟩ = 0` exactly, at every site, for every `v`, `U` and filling.**

The hypotheses are **spin-independence** and **uniqueness in the `S_z = 0` sector** — nothing else.
No bipartite lattice, no half filling, no particle–hole, no Lieb. Uniqueness is not assumed: it is
gated numerically per configuration (G-E6, the in-sector gap), so the anchor's own hypothesis is
measured. This is `pinned_error_computable_from_chart` instantiated at `v₀ = 0`, and it is the whole
of Q7's primary class.

**Not assumed, MEASURED:** `⟨S²⟩`. Lieb's `S = 0` used half filling on a bipartite lattice and is
not inherited under a potential. Q7 measures `⟨S²⟩` and reports it; **no anchor depends on it**,
because §2.2 needs only `S_z`-sector uniqueness, which a spin-`S` multiplet does not violate (each
multiplet contributes exactly one `S_z = 0` state).

### 2.3 Reflection survives conditionally — and yields a one-sided bound

For a reflection-symmetric potential (`v_i = v_{N+1−i}`, which the §3 trap is), `R` commutes with
`H`, so a unique ground state gives `⟨n_i⟩ = ⟨n_{N+1−i}⟩` exactly. Writing `i' = N+1−i` and
`err_i = n^MF_i − n^exact_i`, the two errors share one exact value, so

```
max(|err_i|, |err_{i'}|)  ≥  ½ |n^MF_i − n^MF_{i'}|
```

a **chart-computable lower bound on the chart's density error**. It can only ever *refuse*, never
certify — the right asymmetry for a refusal product. **Declared weak, and why:** the SCF converges
to a reflection-symmetric density for a symmetric potential (the broken-symmetry solution breaks the
*spin* pattern, not the density profile), so this bound is expected to read ≈ 0 and say nothing. It
is carried as a **gate and diagnostic**, not as a candidate, and its expected silence is staked.

### 2.4 The general obstruction, and why only §2.2 escapes it

Worth stating because it explains the shape of the candidate set. A converged SCF solution is a
**fixed point of its own stationarity conditions**, so any exact operator identity *evaluated on
chart data* is satisfied identically and returns zero residual. That is the concrete face of
`error_not_computable_from_chart`: the chart cannot catch itself out with its own equations. The
only escape is a statement whose content is **not implied by the chart's stationarity** — an
external theorem the exact state must obey and a variational determinant need not. §2.2 is exactly
such a statement: UHF is free to break spin symmetry, and the exact state is not.

---

## 3. THE FAMILY

**Inhomogeneous Hubbard chain, open boundaries, half filling (`N_↑ = N_↓ = N/2`), harmonic trap:**

```
v_i = a · ( (i − c) / (N/2) )²,        c = (N+1)/2
```

so `v` runs from `0` at the centre to `≈ a` at the edges.

**Why a trap and not a disorder realisation** (two sentences, as asked). A harmonic trap produces
the wedding-cake profile — a locally half-filled, strongly correlated core with dilute, nearly-free
wings — so at fixed `(N, U)` the chart is near-exact in one region and maximally wrong in another,
which is precisely the spatial honesty boundary a global cutoff cannot serve. Unlike a disorder
realisation it carries no random seed, so the family stays deterministic and replayable, and its
profile is smooth and reflection-symmetric, which supplies the §2.3 consistency gate for free.

**The grid — fixed now.**

- `N ∈ {8, 10}` required (dims 4 900 and 63 504); `N = 12` optional (D0-c).
- `U/t ∈ {0, 0.5, 1, 2, 4, 8, 16}` (7 values).
- `a/t ∈ {0, 0.5, 1, 2, 4, 8}` (6 values).
- **84 configurations.** The `a = 0` column reduces exactly to Q5's family and is both the
  no-spatial-variation **control** and a **cross-campaign regression tie** (§5).

**REGIONS — staked now:** contiguous blocks of two sites, `r_k = {2k−1, 2k}` for `k = 1 … N/2`.
Four regions at N = 8, five at N = 10; **378 region-instances** in total. Blocks of two are the
smallest size for which a block-restricted density matrix has a non-trivial spectrum (§6, R5).

**THE PLANT, spatial:** at `U/t = 16`, **the region containing the chain centre** — the locally
half-filled, strongly correlated core the trap builds. Every surviving criterion must refuse it.

---

## 4. THE FAMILY-FITNESS PRECONDITION — Q5's lesson, operationalised

Q5's certificate lost to a cutoff because the family's honesty boundary did not move. Q7 must
therefore gate **the family's fitness to pose the question** before adjudicating anything about
certificates.

A configuration is **SPATIALLY SPLIT** iff it contains both a chart-honest region (`E_r ≤ 1`) and a
chart-wrong region (`E_r > 1`).

> **G7-FIT (STAKED): at least 8 of the 84 configurations must be spatially split** (≈ 10%).
> If fewer are, **Q7 is VOID — not a kill.** The family failed to instantiate the question, which
> is a statement about the family and about my choice of potential, and says nothing whatever about
> whether certificates can beat cutoffs. Reported in exactly those words.

This clause exists so that a null result cannot be silently laundered into a verdict about
certificates when it is really a verdict about my sweep design — the mistake Q5 made and paid for.

---

## 5. THE RULERS — what replaces the `U = 0` closed form

Q5 gauged its instrument on the closed-form free chain. A trap has no such closed form, so:

- **G7-E7 (the `U = 0` column, every `a`).** At `U = 0` the exact ground state is a Slater
  determinant of the single-particle Hamiltonian `h_ij = −t(δ_{i,j±1}) + v_i δ_{ij}`, an `N × N`
  real symmetric tridiagonal matrix diagonalised **exactly** by the crate's dense solver. Energy,
  full density profile and every 1-RDM element follow in closed form from filling the lowest `N/2`
  levels per spin. **STAKED: the many-body Lanczos must reproduce it to `≤ 1e-12` relative, at every
  `N` and every `a`**, and the chart must reproduce it to the same tolerance (mean field is exact at
  `U = 0`). This is the ruler, and it is exact rather than asymptotic.
- **G7-E10 (the cross-campaign tie).** At `a = 0` every quantity must reproduce Q5's verified
  closed forms — `E(U=0)/N/t` and `Δ(N)` from `Q_SEAM_PREREG.md` §1.1(i), and the whole N = 2 dimer
  column if N = 2 is run — to `≤ 1e-12`. A Q7 instrument that cannot reproduce Q5's gauged numbers
  is broken before it is interesting.
- **G7-E9 (the demoted particle–hole identity, §2.1).** `E₀(v) = E₀(−v) + 2Σ_i v_i` and
  `⟨n_i⟩_v = 2 − ⟨n_i⟩_{−v}`, **STAKED to `≤ 1e-11`**, run at every `(N, U, a)` by solving the
  mirrored potential. A dead anchor repurposed as a live gate, and it is a genuinely independent
  check because it relates two different Hamiltonians rather than testing one against itself.

All Q5 gates carry: G-E1…G-E6 (with G-E4b in its A2 form), G-C1…G-C3.

---

## 6. THE CHART AND THE PER-REGION OBSERVABLES — fixed now, five entries

Chart: UHF exactly as Q5 (three pinned guesses, mixing 0.3, `1e-12`, lowest converged energy wins;
seed `0x5EA0`), with the site potential added to the Fock diagonal.

| # | per-region observable | chart | exact | τ (**STAKED**) |
|---|---|---|---|---|
| R1 | density, `max_{i∈r} \|Δn_i\|` | `n^MF_{i↑}+n^MF_{i↓}` | measured | `0.02` (carried) |
| R2 | double occupancy, `max_{i∈r} \|Δd_i\|` | `n^MF_{i↑}n^MF_{i↓}` | measured | `0.02` (carried) |
| R3 | magnetization, `max_{i∈r} \|m^MF_i\|` | `n^MF_{i↑}−n^MF_{i↓}` | **`0` — pinned by §2.2** | `0.05` (carried) |
| R4 | intra-block bond, `\|Δb\|` | from `ρ^MF` | measured | `0.02` (carried) |
| R5 | block Boolean defect, `\|ΔD_bool^r\|` | spectrum of `ρ^MF` restricted to `r` | same, exact `ρ` | `0.05` (**new**, same scale as Q5's global) |

`E_r ≡ max over the five of |err| / τ`. A region is **chart-honest** iff `E_r ≤ 1`.

**On R5 and the Q6 lesson.** Q5's global `D_bool` had a *structural* zero on the chart side, which
is what made Q6's correlation partly circular (A1/H2). The **block-restricted** version does not:
a sub-block of an idempotent matrix is not idempotent, so the chart makes a real, varying,
falsifiable prediction for `D_bool^r`, and R5 is a genuine two-sided comparison. That is why the
per-region wrongness-meter is admissible here where the global one was not.

---

## 7. THE CANDIDATES — three, each with its class and its staked fate

### D1 — the theorem-pinned anchor (PRIMARY, per `SelfAudit.pinned_error_computable_from_chart`)

> **Certify region `r` iff `max_{i∈r} |m^MF_i| ≤ κ·τ_m = 0.025`.** No new constant.

Note what this is: R3's exact value is pinned at zero by §2.2, so **the chart's error in R3 is pure
chart data** — D1 certifies on one of the very observables it is judged against, without ever
consulting the exact state. That is the SelfAudit door in operational form.

**P-D1 (STAKED): D1 refuses the plant at every configuration, and has FPs in the pre-breaking
regime** where the chart is symmetric but the density/double-occupancy error has already crossed
tolerance — the spatial descendant of Q5's C3 failure (FP = 19). I name the direction: FPs
concentrated in the **wings** at intermediate `U`, where the trap makes the local density
metallic-but-correlated without polarising it.

### D2 — the self-residual (the staked EXPECTED-FAILURE control, per `error_not_computable_from_chart`)

The chart's local residual weight, in **closed form derived here**. For a converged SCF the singles
vanish (Brillouin), and the double-excitation amplitude through the site-`m` interaction vertex is
`U φ^↑_a(m)φ^↑_i(m)φ^↓_b(m)φ^↓_j(m)`. Summing the squared amplitudes at fixed `m` and using
`Σ_{occ}φ_p(m)² = n_m`, `Σ_{virt}φ_p(m)² = 1 − n_m`:

> **`σ_m² = U² · n_{m↑}(1−n_{m↑}) · n_{m↓}(1−n_{m↓})`**

and the local energy estimate `ê_m ≡ σ_m² / Δ_MF`, with `Δ_MF` the chart's own HOMO–LUMO gap.

> **Certify region `r` iff `max_{i∈r} ê_i ≤ κ·τ_E = 0.01`.**

**Its failure is DERIVED, in closed form, before the instrument exists.** `σ_m` vanishes exactly at
four points: `U = 0`, `n_{m↑} ∈ {0,1}`, `n_{m↓} ∈ {0,1}`. Three of those are places the chart **is**
exact (no interaction; empty site; fully doubly-occupied site). The fourth — a **fully spin-polarised
site**, `n_↑ = 1, n_↓ = 0`, which is exactly the broken-symmetry Mott core the plant builds — is
where the chart lies **maximally**: it reports `d_i^MF = 0` where the exact double occupancy is
`≈ 4t²/U² > 0`, and its magnetization is a pure fabrication.

> **P-D2 (STAKED): D2 CERTIFIES THE PLANT and fails `FP = 0`.** This is Q5's A1/H1 restated as a
> one-line algebraic identity rather than a mechanism argument, and it is `SelfAudit`'s limit
> theorem with an explicit witness. If D2 refuses the plant, the derivation above is wrong and I
> will say so in the title line.

**Caveat, stated:** `Σ_m σ_m²` is **not** an orthogonal decomposition of the total energy variance
(amplitudes from different vertices interfere), so `σ_m` is a per-site residual *weight*, not an
exact variance decomposition. It is used as an indicator with a staked threshold, not as a bound.

### D3 = D1 ∧ D2 — the conjunction

> **Certify iff both certify. No new threshold, no new constant.**

**P-D3 (STAKED): D3 passes `FP = 0` and the plant clause** (D1 refuses the plant even though D2
certifies it), **and is the only candidate to pass the joint gate** — Q5's C4 pattern, one level up.
**And, carrying the lead's Ruling 1 forward verbatim: a D3 pass that does not beat N3/N4/N5 is
reported as CORRECT BUT UNINFORMATIVE.** Two weak components conjoining into a spatial cutoff is
still a cutoff. A D3-alone pass is reported as **"the conjunction passed; neither component did"**,
never as "the certificate works".

**Anti-shopping clause.** Three candidates, each with its expected fate staked above
(**P-D1, P-D2, P-D3**), and **the results title line names the fate of all three, never only the
survivor.**

---

## 8. THE GATES, AND THE UPGRADED ADVERSARY

### 8.1 The joint gate — a criterion passes iff all five hold

Classification is per **region-instance** (378 of them):

1. **`FP = 0`** — no region certified while `E_r > 1`. Fatal, absolute.
2. **coverage `≥ 0.50`** over chart-honest region-instances.
3. **every region at `U = 0` certified**, at every `a` (the chart is exact there for any potential).
4. **the plant refused**: the centre region at `U/t = 16`, at every `a`.
5. **SPATIAL DISCRIMINATION (new, and the real point of Q7): at least 5 configurations must carry a
   MIXED map** — at least one region certified and at least one refused within the same `(N, U, a)`.
   A certificate that always answers uniformly across space is a global cutoff wearing a lab coat,
   whatever its FP count. This is the spatial analogue of the mutation test.

### 8.2 The mutants and baselines — run and reported, never asserted

| | definition | parameters | role |
|---|---|---|---|
| **N1** | certify everywhere | 0 | mutant, must fail (1) and (4) |
| **N2** | refuse everywhere | 0 | mutant, must fail (2), (3), (5) |
| **N3** | best global cutoff `U ≤ u*` | 1 | baseline |
| **N4** | best **per-region** cutoff `U ≤ u*(r)` | one per region (4–5) | **the hard baseline** |
| **N5** | best trap-aware cutoff `U ≤ α + β·d(r)`, `d` = distance from centre | 2 | baseline |

All baselines are fitted **post hoc to maximise coverage subject to `FP = 0` AND the plant refused**
(pin A1/P3, carried), tie-broken to the most conservative admissible rule; an infeasible baseline is
reported as INFEASIBLE, which is itself informative.

**N4 is deliberately given more free parameters than the certificate has, which is zero.** If N4
matches or beats the surviving criterion, the spatial honesty boundary is a per-region threshold in
`U` and **outcome (b) fires again, in those words** — we learn the boundary is low-dimensional even
spatially. **The adversary is not weakened to let the certificate win.**

### 8.3 KILL — Q7, separable

> **The Q7 kill FIRES iff NONE of D1, D2, D3 passes all five clauses of §8.1.**

It takes down the per-region certificate and nothing else: not Q5's verdict, not `SelfAudit.lean`,
not `ModeChart.lean`, not the stance. And it cannot fire if **G7-FIT** failed — a family that never
posed the question cannot answer it, and that path is VOID, not a kill.

---

## 9. Q6′ — OPTIONAL, FULLY SEPARABLE, ITS OWN KILL

Q6 does not respin; the share-as-error-meter is dead and stays dead. The honest successor is a
**chart-grade** predictor:

> **Does the chart's own per-region Boolean defect `D_bool^r(MF)` track the exact `D_bool^r`?**

**The confound, named before the test:** both track the local density profile, which the trap
imposes. So the null must control for it.

- **P-Q6′ (STAKED): partial Spearman `ρ(D_bool^r(MF), D_bool^r(exact) | n̄^MF_r) ≥ 0.50`, `p < 0.01`**
  under a permutation null that shuffles region labels **within each `(N, U, a)` stratum** — which
  preserves the configuration-level structure and tests only the across-region information. Null
  shape reported before the `p`; `p` quoted, not `z`.
- **KILL (its own, separable):** `ρ < 0.20` or `p > 0.05`. Firing kills the chart-grade predictor and
  touches nothing in Q7's certificate verdict.

If time is short, Q6′ is **dropped**, and dropping it changes no Q7 verdict. Stated so that its
absence is not later read as a suppressed negative.

---

## 10. THE MEANING OF EVERY OUTCOME — fixed before any result

- **(a) D3 passes §8.1 and beats N3, N4 and N5.** The certificate earns its keep: chart data,
  read against a theorem, separates honest from lying **spatially**, better than any bounded cutoff
  on the coordinates. This is the result Q5 could not have produced, and it is the crystal tier's
  seam policy with a measured warrant.
- **(b) D3 passes §8.1 but N4 (or N3/N5) matches its coverage.** **CORRECT BUT UNINFORMATIVE**, in
  those words, exactly as Q5. The refusal is sound; the chart-internal warrant is not supported,
  because a bounded threshold rule on the coordinates does the same work. Reported in the title line.
  No rescue by adding a candidate.
- **(c) D3 fails clause 5 (no mixed map).** The certificate is spatially blind — it answers
  uniformly across regions — and is a global cutoff by another name **even if `FP = 0`**. Reported
  as such, and it is the most likely quiet failure, so it is gated explicitly.
- **(d) D2 refuses the plant.** §7's closed-form derivation of `σ_m`'s zeros is **wrong**. Reported
  in the title line as loudly as a confirmation, with the measured `σ` at the polarised core.
- **(e) The kill fires (§8.3).** The per-region certificate is decoration on this family: record
  dead, keep marked. What survives is the inhomogeneous reference and its ruler.
- **(f) G7-FIT fails (< 8 spatially split configurations).** **Q7 VOID, not a kill.** The trap did
  not produce a spatial honesty boundary; that is a verdict on my potential and my grid, and it
  licenses no statement about certificates. The next move would be a stronger potential or a
  disorder family, named as such.
- **(g) `⟨S²⟩ ≠ 0` at some configuration.** Interesting, reported, and **it changes no anchor** —
  §2.2 does not use it. It would refute any residual expectation that Lieb's conclusion survives a
  trap, which is worth knowing on its own.
- **(h) The §2.3 reflection bound reads non-zero anywhere.** The chart broke reflection symmetry,
  which the staked expectation says it will not. Reported, and it would hand the certificate a
  second theorem-pinned anchor for free.

---

## 11. HOUSE GATES CARRIED

1. **Pre-registration** — this file, committed before the Q7 instrument exists.
2. **Separable kills** — Q7's kill (§8.3) and Q6′'s (§9) are independent, and neither touches Q5,
   the Lean, or the stance.
3. **VOID vs KILL kept distinct** — G7-FIT (§4) and outcome (f) exist precisely so a family failure
   cannot be laundered into a verdict about certificates.
4. **Null matched to the generative structure** — Q6′ permutes within `(N,U,a)` strata and controls
   for the local density, because the density profile is the confound the trap imposes.
5. **p, not z**, with the null's shape reported first.
6. **A residual is never support** — every P- above is an advance prediction with a fixed threshold.
7. **Fired kills reported as loudly as survivals**, in the title line, dead candidates kept and
   marked.
8. **The robustness clause (A3/R2) stands**: if any amendment changes which configurations are
   admissible, both adjudications are computed, and a disagreement makes the kill UNADJUDICATED.
9. **Named denominators** — 84 configurations, 378 region-instances, coverage over chart-honest
   region-instances only.
10. **Detached compute** — any run over 5 minutes goes under `setsid` with a done-marker and a
    RESUME entry; the closed Q5 ledger is not reopened, Q7 gets its own.
11. **Shared-tree hygiene** — pathspec commits only; no stance or Lean edit; findings return to the
    integrator.

## 12. PRIOR ART

Trapped-lattice wedding-cake density profiles: Batrouni et al., PRL 89, 117203 (2002); Rigol &
Muramatsu, PRA 70, 043627 (2004). Local-density approximation for trapped Hubbard systems:
Rigol et al., PRL 91, 130403 (2003). Mean-field stability and symmetry breaking: Coulson & Fischer
(1949), Thouless (1960). Spin-reflection positivity: Lieb, PRL 62, 1201 (1989) — **cited here for
what it does not cover** (§2.2 derives the surviving anchor without it). Natural occupations as the
correlation measure: Löwdin, Phys. Rev. 97, 1474 (1955).

**Not borrowed, and the actual deliverable:** the **per-region refusal** — a spatial
Certified/Refused map from chart data read against a derived anchor, staked before the instrument
existed, mutation-tested, and adversarially compared against a per-region post-hoc cutoff that is
handed more free parameters than the certificate has.

## 13. KNOWN HOLES

1. **One primary anchor** (D0-a). If `m_i = 0` is the only strong pin, Q7's primary class is thin,
   and a D1 failure is close to a class failure. Named now so it is not a surprise later.
2. **Tolerances are carried, not derived.** They inherit Q5's arbitrariness; the defence is that
   carrying them unchanged removes the freedom to re-tune, not that they are principled.
3. **Blocks of two sites** are a staked choice. A different block size could change coverage; no
   block-size scan will be run after seeing results.
4. **`σ_m` is an indicator, not a bound** (§7 caveat) — its threshold is a stake, not a theorem.
5. **N ≤ 10 (12 optional)** limits spatial resolution to 4–5 regions; a boundary finer than one
   block is invisible to this instrument.

---

# AMENDMENT A1(Q7) — adversarial review, adopted 2026-08-23, BEFORE any instrument exists

Five required pins, all adopted, none disputed; one optional anchor, **accepted**; and §2.4
restated with precise hypotheses so it lifts to Lean cleanly. **Still no crate change, no ground
state, no chart, no error.** No threshold frozen above is loosened; A1(Q7) adds one candidate that
introduces **no new constant**.

## A1(Q7)/P1 — every candidate's expected fate, staked

Q5's anti-shopping clause carried in full: the results title line names the fate of **all four**
candidates, never only the survivor.

**P-D1 (the theorem-pinned anchor).**
- **Refuses the plant** at every configuration (deep breaking gives `|m^MF| → 1 ≫ 0.025`).
- **Fails `FP = 0`**, in **two named bands**:
  (i) the **pre-breaking** band — the chart is spin-symmetric, so D1 certifies, while R1/R2/R4/R5
  have already crossed tolerance (Q5's C3 failure, FP = 19, one level up);
  (ii) the **weak-breaking threshold band** just past the *local* Coulson–Fischer point, where the
  order parameter has turned on but `0 < |m^MF| ≤ κ·τ_m = 0.025` — D1 still certifies because it has
  a threshold, not a zero, while the other four observables are already out.
- **Coverage cost:** none of note; D1 is permissive by construction.

**P-D2 (the self-residual control).**
- **CERTIFIES the plant** and fails `FP = 0` and clause 4 — derived in closed form (§7).
- **Coverage cost concentrated in the CORE at intermediate `U`.** `σ_m = U/4` is **maximal** at
  `n_↑ = n_↓ = ½` (local density 1, unpolarised), which is precisely the trap's half-filled core, so
  D2 refuses honest core regions and drags any conjunction's coverage exactly as C1 dragged C4's.
- **Certifies the dilute wings correctly** (`σ → 0` as `n → 0`), where the chart really is near-exact.
- **P-D2-SPATIAL:** because it certifies wings and refuses cores, **D2 alone passes clause 5** while
  failing `FP = 0`. Staked because it separates the two failure modes: spatial discrimination and
  soundness are independent properties, and a candidate can have one without the other.

**P-D1b (the reflection anchor, new below).** **Silent** — never fires — at every configuration
where the SCF converges reflection-symmetrically, which the staked expectation says is everywhere
except possibly the deep-trap column. See the honesty note in A1(Q7)/OPT.

**P-D3 (the conjunction `D1 ∧ D1b ∧ D2`).**
- **Passes `FP = 0`** and **clause 4** — D1's refusal of the plant overrides D2's certification of it.
- **Passes clause 5** (inherits D2's wing/core split).
- **Clause 2 is where it is at risk:** D2's core refusals cost coverage. Staked: **D3 passes clause 2
  with coverage `≥ 0.5`**, and if it does not, **the named failure mode is D2's intermediate-`U` core
  refusals** — recorded now so the diagnosis is not invented afterwards.
- **The only candidate expected to pass the joint gate**, and subject to the severity rule: a D3 pass
  that does not beat N3/N4/N5 is **CORRECT BUT UNINFORMATIVE**, and a D3-alone pass is **"the
  conjunction passed; neither component did"**.

## A1(Q7)/P2 — SPATIALLY SPLIT and the mixed-map clause, pinned numerically

The error used is `E_r`, the **five-observable normalised max** of §6 (chart against the exact
state) — the truth-side per-region error, not any chart-internal estimate.

> **SPATIALLY SPLIT (STAKED, with a margin):** a configuration `(N, U, a)` is spatially split iff
> **`min_r E_r ≤ 0.5` AND `max_r E_r ≥ 2.0`.**
> The margin is deliberate: `min ≤ 1 < max` would let a configuration qualify on two regions
> straddling the tolerance by numerical noise, and G7-FIT must not pass on borderline arithmetic.

> **G7-FIT (unchanged in force, now numeric): at least 8 of 84 configurations spatially split**, else
> **Q7 VOID, not killed.**

> **CLAUSE 5, SPATIAL DISCRIMINATION (sharpened): on at least 5 SPATIALLY SPLIT configurations, the
> criterion must certify at least one region with `E_r ≤ 1` AND refuse at least one region with
> `E_r > 1`, within the same `(N, U, a)`.**
> Sharpened from the frozen text on purpose: a mixed map on a uniformly-honest configuration is a
> pattern of false negatives, not discrimination. Discrimination means getting the *split* right
> where a split exists, and only split configurations can evidence it.

Exact block `D_bool` is reported per region as a **secondary diagnostic** and cannot change the
split determination.

## A1(Q7)/P3 — N4's parameterization, pinned

> **N4: certify region `r` at `(U, a)` iff `U ≤ u*(r)`** — one threshold per region,
> **fitted JOINTLY ACROSS ALL `a`**, `N/2` parameters total (4 at N = 8, 5 at N = 10).

**Why this sits below the oracle line, stated as the general principle:**

> **A baseline's parameter count must not scale with the number of questions it answers along a
> sweep axis.** `u*(r, a)` would carry one parameter per `(region, a)` cell — 30 at N = 10 — and
> would answer the `a`-axis by memorising it. N4 spends 5 parameters answering 210 region-instances
> at N = 10 and must generalise across the whole `a`-sweep from a single threshold vector.

N4 is still handed **more** free parameters than the certificate has, which is **zero**. That is the
point: losing to N4 means the spatial boundary is a per-region threshold in `U` and outcome (b)
fires in spatial dress.

## A1(Q7)/P4 — the VOID budget, and G-E6's anticipated failure region

> **UNDERPOWERED (STAKED): more than 12 of 84 configurations VOID** (≈ 14%, matching Q5's `>10/70`)
> → neither Q7's kill nor Q6′'s is adjudicated.
> **Per-column:** an `a`-column is the 14 configurations at fixed `a` (2 N × 7 U). **If more than 7
> of 14 VOID in any `a`-column, that column is declared UNUSABLE**, reported by name, and excluded
> from every spatial statistic. **If two or more `a`-columns are unusable, the campaign is
> UNDERPOWERED** regardless of the global count.

> **P-VOID-a (STAKED IN ADVANCE, so it reads as anticipated behaviour rather than a surprise):**
> **the deep-trap columns `a/t ∈ {4, 8}` at large `U` are the anticipated VOID region**, through
> **reflection-paired quasi-degeneracy**. Mechanism: once the trap builds a Mott or band-insulating
> core, it separates a **left** and a **right** wing whose excitations are related by reflection and
> coupled only by tunnelling through the core; the even/odd pair is then split exponentially in the
> core width, and the in-sector gap `E₁ − E₀` collapses below G-E6's staked `1e-6 t`.

**The distinction that must be reported with it, because it is the whole point:** the anchor's
*mathematical* hypothesis (uniqueness of the ground state in the `S_z = 0` sector) **still holds** —
the splitting is exponentially small but non-zero, so `m_i = 0` remains exactly true. What fails is
**numerical resolution**: Lanczos cannot separate a pair that close, and the computed vector may be
a mixture, which would show up as a blown `m_i` residual in G-E5b. Configurations there are
**VOID — anticipated instrument behaviour, plus a family re-examination — never a kill**, and never
evidence against the anchor.

## A1(Q7)/OPT — the reflection anchor D1b: ACCEPTED

**Two-sentence reason.** It costs **no new constant** (its threshold is `κ·τ_n = 0.010`, already
frozen), it relieves D0-a's single-member primary class, and it is a genuine instance of
`pinned_error_computable_from_chart` — a unique ground state of a reflection-symmetric `H` has
`⟨n_i⟩ = ⟨n_{N+1−i}⟩` exactly, so the chart's own density asymmetry is a theorem-pinned deviation
readable from chart data alone. Declining it would leave the primary class at one member for the
sake of avoiding a candidate whose expected fate I can state precisely in advance, which is the
opposite of what pre-registration is for.

> **D1b: certify region `r` iff `max_{i∈r} |n^MF_i − n^MF_{N+1−i}| ≤ κ·τ_n = 0.010`.**
> Refusal-only in character: it can detect error, never certify its absence (§2.3's one-sided bound).

**P-D1b (staked honestly, including the uncomfortable part).** D1b is expected to be **silent
everywhere**: the broken-symmetry UHF solution breaks the *spin* pattern while leaving the *density*
profile reflection-symmetric, so its refusal set is expected to be empty and its contribution to D3
exactly nil. **Its one real chance is the deep-trap column**, where a reflection-broken
(left- or right-localised) SCF solution could appear — **which is the same column P-VOID-a expects
to VOID.** So D1b may well be silent for the boring reason *and* unmeasurable for the interesting
one. That is predicted here rather than discovered later, and if D1b's only firings land in VOIDed
configurations the honest reading is **"untested", not "null"**.

**Consequences:** the primary class has **two** members (D1, D1b); `D1* ≡ D1 ∧ D1b` is reported as a
diagnostic; **`D3 ≡ D1 ∧ D1b ∧ D2`**, which reduces exactly to `D1 ∧ D2` if D1b never fires. Four
candidates are gated (D1, D1b, D2, D3); all four fates are staked; no new constant enters.

## A1(Q7)/L — §2.4 restated with precise hypotheses, for the Lean brick

Stated so it lifts without reinterpretation. Mechanization is the integrator's, later.

**THE STATIONARITY-IDEAL OBSTRUCTION.** Let `Θ` be the chart's parameter space and
`S : Θ → ℝ^m` its stationarity conditions, so a *converged chart* is any `θ*` with `S(θ*) = 0`.
Call a function `f : Θ → ℝ` a **self-audit signal**. Suppose:

- **(H1)** `θ*` is converged: `S(θ*) = 0`;
- **(H2)** `f` lies in the **ideal generated by `S`**: there exist `g_1 … g_m : Θ → ℝ` with
  `f = Σ_i g_i · S_i` pointwise on `Θ`.

**Then `f(θ*) = 0`** — identically, for every converged chart, in every world. Hence `f` carries no
information about the deviation from the truth: it is zero when the chart is right and zero when it
is lying.

**Plain reading:** *the chart cannot catch itself out with its own equations.* Any exact operator
identity that the variational problem already imposes is, at the solution, the equation `0 = 0`.

**The complement, which is the door:** a signal with `f ∉ ideal(S)` can be non-zero at `θ*`.
Theorem-pinned anchors are exactly of this kind — the external theorem (`m_i = 0`, `⟨n_i⟩ =
⟨n_{N+1−i}⟩`) is a constraint the variational problem **does not impose**, which is precisely why a
determinant is free to violate it and why the violation is readable. The relation to the existing
bricks: this is a *sharper, structured* companion to
`SelfAudit.error_not_computable_from_chart` — that theorem says no function of chart data computes
the error in general; this one identifies a large, checkable **syntactic class** of candidate audits
that are guaranteed to be identically zero, and so tells a designer which audits to not even try.

**Scope note for the mechanization:** (H2) is a statement about the *presentation* of `f`, not about
its values, so the brick should carry the ideal-membership hypothesis explicitly rather than
attempting to decide it. Q7 supplies the instance, not the proof: `σ_m` is a self-audit signal, and
§7's four zeros are its measured face.
