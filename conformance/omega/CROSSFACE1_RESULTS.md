# CROSS-FACE-1 — every gate on branch (a); the law lands on held-out substrates with no refit

*2026-08-28. Prereg frozen verbatim from the rung4-law opus track's
derivation; instrument built by the same track against the frozen copy;
verified by an independent fresh rerun in the lead session (exit 0).
Instrument, log and results committed together.*

```
G0/B3: PASS both substrates      G1: Theorem-2 inequality PASS on all 9 views
S1: BRANCH(a) — 2T closed views exact, and the transfer is exactly I₃
    (ambivalence controls STATIONARITY, not closure — the corrected guess)
S2: BRANCH(a) — 3/4 and 43/144 exact (Theorem 6 forward on a new group)
S3: BRANCH(a) — THE PERIMETER LAW: W = 2/9, 38/81, 422/729, 38/81 at
    L = 1, 3, 5, 3, with λ_e = 2/3 read off the circuit, no free parameter
S4: BRANCH(a) — the conditioned-edge correction confirmed: 122/243, not
    the naive 114/243 — a derived correction no parameter could absorb
S5: BRANCH(a) — carrier-independence exact across the stated set
R1: BRANCH(a) — THE RIVAL REFUTED: micro mixing rate predicts one cost for
    all four 2T views; the measured costs are 0, 0, 3/4, 43/144
plants: all FIRE
```

## What is earned — rung 4, in the review's own success criterion

> "It must predict held-out substrates without domain-specific refitting."

It did. The cross-face law — **rent = ceiling × (1 − retention)**, with the
view mixing modulus λ(T, μ, v) as the carrying variable and a perimeter
law `W(L) = (1−1/n)(1−λ_e^L)` for holonomy views — was derived in the
freeze from two substrates and landed EIGHT exact-rational predictions on
two substrates whose step maps the freeze never touched: 0, 0, 3/4,
43/144, 2/9, 38/81, 422/729, and the discriminator 122/243. The rival
(the ladder's own literal wording: cost as a function of the DYNAMICS'
mixing rate) is refuted by measurement, and the obstruction that forced
the view-relative variable is machine-backed
(`alpha_eq_one_of_injective`).

Two conceptual purchases beyond the numbers:
1. **Rent and closure are one object read at two strengths** (Theorem 1:
   rent is the Bayes error of the best memoryless coarse model — the
   minimax IS the definition, discharging M-ONE-MODEL-DELTA at the root).
2. **Blindness is not free maintenance** (S1's third stake): an
   orientation-blind view is cheap because its content is discarded, but
   what ambivalence buys is stationarity — a maintained view that sits
   still versus one that cycles at no charge. The exchange rate between
   discarded structure and cheap upkeep is finer than "blind is free."

## Scope, restated from the freeze

Finite exact models only. W counts displaced mass — not energy, no
Landauer normalisation (the predecessor's K4 fired on exactly that step
and is not repeated). Theorem 2 is proved by hand and checked on 4000
random kernels, not mechanized — its Lean brick is named as follow-up.


## Correction (2026-09-07): Theorem 3's prose claims an equality the argument only bounds

An external review of commit 2dd820b found, and the lead verified in exact arithmetic, that
`CROSSFACE1_PREREG.md` Theorem 3 is FALSE as an equality on its stated hypotheses. The
statement: for uniform `μ_i = 1/N`, `M` normal with every non-trivial eigenvalue of modulus
`λ` and the row maximum on the diagonal, `W = (1 − 1/N)(1 − λ)`. The counterexample:

```
M = [[0.5, 0.4, 0.1],
     [0.1, 0.5, 0.4],
     [0.4, 0.1, 0.5]]
```

is normal and doubly stochastic, its diagonal entries are strict row maxima, and both
non-trivial eigenvalues have modulus `√13/10 = 0.360555`. The programme's own Bayes-error
definition (Theorem 1) gives `W = 1 − Σ_i (1/3)·max_j M_ij = 1/2`, where the claimed formula
gives `(2/3)(1 − 0.360555) = 0.426297`. The reviewer also checked that this kernel arises by
coarse-graining a 30-state deterministic permutation, so it is a legitimate view.

**Where the proof went wrong.** Theorem 2 is an INEQUALITY, `W ≥ (1 − μ_max) − λ·(Σ_i σ_i)·σ_max`;
the prose proof of Theorem 3 "substitutes `μ_i = 1/N` into Theorem 2" and reads the result as
an equality. Substitution gives only `W ≥ (1 − 1/N)(1 − λ)`, a LOWER bound, which the
counterexample satisfies (`1/2 ≥ 0.4263`). Equality holds when `M = λI + (1 − λ)Π` — the
uniform-relaxation kernel — which is exactly and only what the machine-checked theorem states
(`lean/CIRISHolon/GiniRent.lean`, `rent_uniformRelax`, `rent_uniformRelax_gini`). The Lean
survives the counterexample because it never claimed the broader hypothesis.

**The corrected statement.** *Theorem 3 (uniform normal views).* Under Theorem 3's hypotheses,
`W ≥ (1 − 1/N)(1 − λ)`; equality holds for `M = λI + (1 − λ)Π`, and "rent = ceiling ×
(1 − retention)" is the equality case, not the general one. Every downstream use of Theorem 3
in this record is on kernels of the equality form (the circulant relaxations the campaign
built), so no measured number changes; the prose claim is narrowed, the freeze's text is left
as frozen, and this correction is the record of it. Registered as **M-INEQUALITY-READ-AS-EQUALITY**
in `conformance/gravity/MISFITS.md`. Credit: the external review of 2026-09-07.
