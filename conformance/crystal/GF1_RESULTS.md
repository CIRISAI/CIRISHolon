# GF1 — the magic price of gauge vacua: READ. S1 converged at five couplings and refused at the sixth; S2 KILLED as staked at x = 0.25 and met on branch (c)'s extension; S3 retired by theorem; S4 met at every coupling

*2026-09-19. Prereg `GF1_PREREG.md` (frozen alone `b4a42e8`, corrected on building
`79fa075`), Amendment 1 `GF1_AMENDMENT_1.md` (`79fa075`, corrected on building `9c87a8e`,
the lead's reading `0865f15`), instrument `q8-mps::magic` (merged `526befb`, `f254106`),
plants P1–P7, P3′, P9 firing on main (`11 passed, 5 ignored, 513 s`), ladder
`gf1_ladder.sh` + `gf1_ladder_ext.sh`, reader `gf1_ladder_read.py`, every point's JSON and
the read in `gf1_ladder/`. Node GF1 of the fold below the atom (`OBJECT.md` Fold III).
Written after the ladder, from its read; nothing here was known when the stakes were set,
except what the two corrections say.*

## 0. Verdict

| stake | staked | read | verdict |
|---|---|---|---|
| **S1** density converges | successive `M₂` differences per site constant to 10 % from `N = 16` | spread `0.0 %` at `x = 1/64, 1/16, 1/4`; `0.1 %` at `x = 1`; `3.7 %` at `x = 4`; **`x = 16` unreadable** — `N ≥ 16` refused at `χ = 11` | **CONVERGED at five couplings; REFUSED at `x = 16`** (a reading about the vacuum's bond dimension, A1) |
| **S2** the fixed point | `M₂/N < 0.05` at `x = 0.25` | **`0.2066`** at `x = 0.25`; `0.0212` at `x = 1/16`; `0.0014` at `x = 1/64` | **KILLED as staked**; branch (c) extended to `x = 1/16` and is MET there — the density vanishes as `≈ 5.7 x²` and crosses the bar at `x ≈ 0.09`, not `0.25` |
| **S3** the area law | slope of the non-local magic in `N` under `0.01` | `M₂^nl ≡ M₂` on a parity-definite real vacuum, by theorem per site and by a positive-definite joint Hessian | **RETIRED — branch (b) on vacuity**: the sentence has no quantity distinct from `M₂` |
| **S4** the hadron's price | `M₂` of a ten-site box under `10` at every coupling | **`0.013, 0.206, 2.02, 3.98, 3.79, 3.02`** at `x = 1/64 … 16`; price bound `2^{M₂} ≤ 16` | **MET at every coupling**, with the price six hundred times under the stake's `2^{10}` |

**What the fold keeps.** Its price clause is measured: a ten-site box of the Schwinger vacuum
carries under four bits of magic at every coupling read, and the magic density `c(x)` is a
banked function of the coupling, converged in volume to a tenth of a percent below `x = 4`.
**What the fold loses.** Its "area-law" clause, as written, names no quantity: on the very
states it is for, the local frame removes nothing, so the non-local magic IS `M₂`, which is
extensive — `c(x) N` — at every coupling. The extensive density is the finding. **Fold III
was reworded the same day** around `c(x)` and the box price, the owner's call ("of course
re-write; this is just clarity"), the locked text kept legible beside it (`OBJECT.md`). **What the instrument cannot do.** Read the
weak-coupling vacuum: at `x = 16` the variance gate admits only `N ≤ 12` at `χ = 11`, the
exact instrument's lease. The two exits (a compressed Pauli MPS; perfect Pauli sampling)
are named in A1 and not built.

## 1. The ladder

`M₂` at each point, the admitting `χ` in parentheses (the smallest of `{6, 8, 11}` under the
variance gate `10⁻³` per site); `REF` = refused at `χ = 11`.

| `x` | `N = 8` | 12 | 16 | 24 | 32 | 48 | `c(x)` | S1 spread |
|---|---|---|---|---|---|---|---|---|
| 1/64 † | 0.00984 (6) | 0.01546 (6) | 0.02108 (6) | 0.03232 (6) | 0.04356 (6) | 0.06604 (6) | **0.00141** | 0.0 % |
| 1/16 ‡ | 0.15168 (6) | 0.23812 (6) | 0.32457 (6) | 0.49745 (6) | 0.67034 (6) | 1.01611 (6) | **0.02161** | 0.0 % |
| 1/4 | 1.50770 (6) | 2.34869 (6) | 3.18968 (6) | 4.87167 (6) | 6.55366 (6) | 9.91764 (6) | **0.21025** | 0.0 % |
| 1 | 2.81478 (6) | 4.50706 (6) | 6.19052 (6) | 9.55584 (6) | 12.92109 (6) | 19.65158 (6) | **0.42066** | 0.1 % |
| 4 | 2.09170 (6) | 3.88689 (6) | 5.70104 (8) | 9.26177 (8) | 12.77198 (8) | 19.76958 (8) | **0.43735** | 3.7 % |
| 16 | 1.80756 (11) | 3.32387 (11) | REF | REF | REF | REF | — | refused |

† a LABELLED EXTRA beside branch (c)'s extension, not a gate. ‡ branch (c)'s extension.
Variances per site at the admitted points: `10⁻¹⁷` to `10⁻¹²` at `x ≤ 1/4`, `2 × 10⁻⁶` at
`x = 1`, `2–8 × 10⁻⁴` at `x = 4`, `1–7 × 10⁻⁴` at `x = 16, N ≤ 12`; the `x = 16` refusals read
`1.3, 2.0, 2.5, 3.3 × 10⁻³` at `N = 16, 24, 32, 48`, `χ = 11` — over the gate by less than
a factor of four, so `χ ≈ 16` would likely admit them, and `χ⁸` says the exact instrument
cannot afford it.

`c(x)` against `x`: `0.00141, 0.0216, 0.210, 0.421, 0.437` — `c/x² = 5.8, 5.5, 3.4` at the three
strong couplings, so the density leaves the stabilizer fixed point quadratically in the
hopping, as second-order perturbation theory would say, and saturates near `0.44` bits
per site between `x = 1` and `4` (the `x = 16` points at `N = 8, 12` read `0.23, 0.28` per
site and are too small a volume to place on the curve).

## 2. S4 — the ten-site box

`M₂` of the reduced state of sites `1…10` (mixed-state stabilizer Rényi entropy, `sre2_box`)
at the largest admitted `N`, with its spread over `N ≥ 12`:

| `x` | 1/64 | 1/16 | 1/4 | 1 | 4 | 16 (`N = 12`) |
|---|---|---|---|---|---|---|
| `M₂^box(10)` | 0.01335 | 0.20555 | 2.02188 | 3.98344 | 3.78505 | 3.01803 |
| spread over `N` | `10⁻⁶` | `10⁻⁶` | `1.2 × 10⁻³` | `2.9 × 10⁻²` | `2.7 × 10⁻¹` | one `N` |
| `2^{M₂}` | 1.0 | 1.2 | 4.1 | 15.8 | 13.8 | 8.1 |

The box's magic is volume-independent to the digits shown at strong coupling and to a few
percent at `x = 1, 4` — the ten-site price is a property of the coupling, not the chain,
which is the one sense in which "area-law" survives: the magic INSIDE a box of fixed size
converges with the surrounding volume. It is not the sentence Fold III wrote.

## 3. What was wrong before the ladder, in order

1. The prereg's cost `O(N·4·χ⁴)` — the fourth powers need four replicas, `χ⁸`/`64χ⁹`; exact
   reading at `χ ≤ 11` (`79fa075`).
2. The prereg's `M₂^loc` — identically `M₂` by Clifford invariance (`79fa075`).
3. Amendment 1's `M₂^nl` over continuous rotations — identically `M₂` on a parity-definite
   real state, per site by theorem, jointly by the Hessian (`9c87a8e`, `184ecb4`).
4. Amendment 1's cost "seconds to a minute" per reading at `N = 48, χ = 11` — measured `8.8` s
   per site (`9c87a8e`); the ladder as run needed `χ ≤ 8` below `x = 16` and took `31` minutes
   on two lanes.
5. Amendment 1's "GHZ's `M₂^nl` is not zero" — GHZ is a stabilizer state (`9c87a8e`).
6. The prereg's S2 bar at `x = 0.25` — set without the `x²` law, at a coupling where the
   density is four times the bar; branch (c) was written for exactly this and taken.
7. The ladder launcher's gate — a default expression evaluated on a float, so every point
   read "refused" for one minute before the fix (`8ca0169`); no point was banked under it.

## 4′. The `x²` law, derived after the read (2026-09-19, the same afternoon)

At strong coupling the vacuum is the staggered stabilizer state and the hopping creates one
pair across a link at electric cost `1`, so to first order `|ψ⟩ = |vac⟩ − x Σ_n |pair_n⟩`. On
one link that is `|01⟩ − x|10⟩`, whose stabilizer Rényi entropy is EXACTLY
`M₂ = −log₂(1 − ¼ sin² 4θ)` with `tan θ = x` (the non-zero Pauli expectations are `ZZ = −1`,
`Z₁ = −Z₂ = cos 2θ`, `XX = YY = sin 2θ`). Hence, per link,

> **`c(x) = (4 / ln 2) x² + O(x⁴) = 5.7708 x²`.**

Brute-force SRE on the first-order vacuum at `N = 6, 8` gives `0.001406–0.001407` per link
at `x = 1/64` against the ladder's **`0.00141`** (`5.775 x²`, agreement to `0.3 %`), and
`0.02193` at `x = 1/16` against the ladder's `0.02161` (second order, `1.5 %`); at `x = 1/4`
first order is `30 %` high and perturbation theory has ended (`gf1_x2_law.py`,
`gf1_ladder/x2_law_check.log`). So the strong-coupling density is a theorem's number, S2's
bar is crossed at `x = √(0.05/5.77) = 0.093`, and the prereg's bar at `x = 0.25` was set
where the first-order state already carries four times it. **The measurement rendered by
the fast side** (`FAST_AND_SLOW.md` §2, negation's cousin: a stake that a derivation prices
before it is run).

## 4. Branch

S1 read at five couplings, refused at one; S2 killed as staked, met on the extension; S3
vacuous by theorem; S4 met. **The prereg's branch (a) needs S2, S3, S4 — it is not entered.
Branch (b) is entered on S3's vacuity (not on a slope); branch (c) was entered and
discharged; S1's refusal at `x = 16` is a refusal, not (d), since it is the bond dimension
and not the volume that refused.** Fold III's price clause stands measured; its area-law
clause had no quantity and was reworded the same day.
