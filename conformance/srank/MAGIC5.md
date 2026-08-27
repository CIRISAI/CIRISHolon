# Magic5FromCat — landed in the engine, 2026-08-27

The partial stabiliser decomposition of **Kissinger, van de Wetering and
Vilmart**, *Classical simulation with partial and graphical stabiliser
decompositions*, TQC 2022 (arXiv:2202.09202) §4.2, built on
**Qassim–Pashayan–Gosset** (arXiv:2106.07740) cat states and on the
**Bravyi–Gosset** magic-state gadget. Reference implementation read:
**quizx**, file src/decompose.rs (`replace_magic5_0/1/2`), Apache-2.0, same
authors — compatible with this repo's AGPL-3.0, credited in the module header
of `engine/crates/holon/src/magic5.rs`. The mathematics is theirs; ours is the
translation into the affine phase-polynomial engine, the deterministic
mixed-radix branch index, and the exactness gates.

## The rule

Five magic states in, **three terms**, each keeping **one** magic state. Four
consumed per three terms, so `N(t) = 3·N(t−4)` and `α = log₂(3)/4 ≈ 0.3963`
**concretely at finite t**, where the previous path was `2^{0.5t}`.

## The identity, as derived and mechanised here

With `|A⟩ = (|0⟩+ω|1⟩)/√2`, `ω = e^{iπ/4}`, and three Clifford frames applied
to `|A⟩` on wire 0 and `|0⟩` on wires 1–4:

```
Φ₀ = [Z(0); CX(0→1..4)]
Φ₁ = [S†(0); H(0); CX(0→1..4); H(0..4)]
Φ₂ = [S†(0); Z(0); H(1..4); CX(1..4→0); CZ(all 10 pairs)]

|A^{⊗5}⟩ = ½·Φ₀ + ((−1+i)/2)·Φ₁ + ((−1−i)/2)·Φ₂
```

The three shapes depend on `x` only through `|x|`, so the coefficients are
**over-determined**: six Hamming-weight equations in three unknowns, and they
close on all six. That is the derivation. The coefficients are *not* quizx's
`Scalar4` constants — those carry ZX normalisation this engine does not share.

Because every Φ is a Clifford image of `|A⟩ ⊗ |0000⟩`, the retained magic state
is a genuine `|A⟩` on a named wire, which is what makes the rule recursive
rather than a one-shot block table. All three terms keep the **same** wire, so
the round schedule is branch-independent and the branch space stays a
deterministic mixed-radix index the mesh can shard by index alone.

## Gates (all in-tree, all exact, no tolerance)

| gate | what it checks |
|---|---|
| `magic5::magic5_is_exact` | `Σ_j c_j Φ_j(x) = ω^{|x|}·2^{−5/2}` at all 32 basis states, in `Z[ω]·2^{−m/2}`. Called by `Magic5Source::new` on **every** construction, so a wrong constant cannot ship. |
| `magic5::register_is_exact(t)` | the **whole recursive plan** re-derived against `|A^{⊗t}⟩` at every one of `2^t` basis states, `t = 0..10`. This is what says the recursion composes. |
| `tests/magic5.rs::magic5_equals_the_current_path_exactly` | THE REGRESSION GATE: 80 random Clifford+T circuits (`t ≤ 10`, `n ≤ 6`), all `2^n` basis states each — Magic5 vs `NaiveSource` **and** vs `BgSource`, bit-for-bit in the ring. 1984 amplitudes, 266 nonzero. Referee `holon-qasm::magic` at 1e-10: worst `|Δ| = 0e0`. |
| `…::magic5_matches_on_a_wide_register` | the same at `n = 16, 24` (`t = 9, 10`), support kept full so it is not comparing zeros. |
| `…::deep_t_rounds_compose` | `t = 11..14`, 2–3 rounds, exact on the whole register. |
| `…::the_mixed_radix_index_shards_invariantly` | the mesh fold is identical at 1/2/3/5/7/16 shards. |
| `…::planted_mutation_is_caught` | both planted affine-engine defects (`drop_s_cross`, `wrong_gauss`) are caught — the harness has teeth. |

## Measured branch counts (realised, not asymptotic)

`α` is an **asymptote**. What the source realises at finite `t` is
`log₂(3^{⌊(t−1)/4⌋}·2^{⌈tail/2⌉})/t`, which approaches 0.3963 **from above** and
never equals it. Below five magic states the plan falls back to this repo's own
verified pair blocks, so short circuits are exactly the old path.

| t | naive `2^t` | previous path `2^{⌈t/2⌉}` | Magic5 | vs previous | realised α |
|---:|---:|---:|---:|---:|---:|
| 4 | 2^4 | 4 | 4 | 1.00× | 0.5000 |
| 5 | 2^5 | 8 | 6 | 1.33× | 0.5170 |
| 8 | 2^8 | 16 | 12 | 1.33× | 0.4481 |
| 12 | 2^12 | 64 | 36 | 1.78× | **0.4308** |
| 16 | 2^16 | 256 | 108 | 2.37× | 0.4222 |
| 20 | 2^20 | 1024 | 324 | 3.16× | **0.4170** |
| 24 | 2^24 | 4096 | 972 | 4.21× | 0.4135 |
| 28 | 2^28 | 16384 | 2916 | 5.62× | **0.4111** |
| 32 | 2^32 | 65536 | 8748 | 7.49× | 0.4092 |
| 40 | 2^40 | 1048576 | 78732 | 13.32× | 0.4066 |
| 48 | 2^48 | 16777216 | 708588 | 23.68× | 0.4049 |
| 56 | 2^56 | 268435456 | 6377292 | 42.09× | 0.4037 |
| 64 | 2^64 | 4294967296 | 57395628 | **74.83×** | 0.4027 |

Read honestly: at `t = 28` the realised exponent is **0.4111**, not 0.3963, and
at `t = 64` it is **0.4027**. The `74.83×` at `t = 64` is `3^15·4` against
`2^32`.

## Independent cross-check

`gains2.py` in this directory — the decomposition-rule DP written during the
2026-08-27 prior-art sweep, **before** this port existed — reproduces the same
counts from the published rules: 108, 972, 8748, 78732, 708588, **6377292**,
57395628 at `t = 16, 24, 32, 40, 48, 56, 64`. The engine's plan and the paper
DP agree at every point, and 6,377,292 at `t = 56` is the count cited in the
prior-art literature. Two independent constructions, one number.

## Scope, stated

* The three terms are **not** stabiliser states. No stabiliser-rank claim about
  `|A^{⊗5}⟩` is made or implied — each term carries one magic state, which is
  the whole point and the reason the recursion pays.
* The Labib–Russo trap (`verify.py`, arXiv:2605.28586) is untouched by this: it
  was never imported, and this decomposition does not use their table.
* `Magic5Source` sits alongside `BgSource`; the production path (`run.rs`,
  prune-dedup + mesh fold) is unchanged. Wiring it into production is a
  separate decision with its own measurement.
* The exactness envelope is unchanged: `Cyc` still REFUSES on i128 overflow
  rather than wrapping.
