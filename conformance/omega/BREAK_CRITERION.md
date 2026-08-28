# OMEGA-BREAK-1 — the break criterion, frozen before the instrument

*Rung 7's falsifier. Written before `hunt.py` exists; the predictions in §6 are
staked here and answered only by exhaustive exact search.*

Rung 3 found structure invisible to every probe **and harmless**: the
fiber-internal difference between `ProbeConverse.H₁` and `H₂` is quotient-able,
the Moore quotient is a legal holon with identical probe behaviour, and
`Identity.lean` therefore declares fibers GAUGE. This hunt demands the stronger
object: structure invisible to every current face yet **consequential** — it
cannot be quotiented away without changing an admissible observation.

---

## 0. What the current tuple can see, stated exactly

The faces, from `CROSSFACE1_PREREG.md` §1: a face is `(S, K, μ, v)` with derived
transfer `M_{j|i} = P_ij/μ_i`, view mixing modulus `λ(v) = ‖M − Π‖`, and rent
`W(v) = 1 − Σ_i max_j P_ij`. Every one of these is a function of the joint
one-step law `P_ij = Pr[v(s)=i, v(s')=j]` under `s ~ μ`, `s' ~ K(·|s)`. Probes
(`Probe.lean`) are `view(dyn^[n] s)`. Identity (`Identity.lean`) is
`SameHolon`: `∃ σ τ, ∀ n s, v₂(dyn₂^[n](σ s)) = τ(v₁(dyn₁^[n] s))`.

**Working normal form.** This hunt uses the *strictest* invisibility available:
one micro space `S`, one view `v : S → V`, one measure (uniform), and two
deterministic dynamics `f, g : S → S` with

> **(PE)** `v(f^[n] s) = v(g^[n] s)` for every `n` and every `s`.

(PE) is `SameHolon` with `σ = τ = id` — no relabelling excuse is available, and
by the lemma below the pair is invisible to the entire tuple, not merely to
probes. General `σ` reduces to this: replace `g` by `σ⁻¹ ∘ dyn₂ ∘ σ` and every
side-2 operation `a` by `σ⁻¹ a σ`; probe agreement becomes (PE) verbatim. A
break found in normal form is therefore a break for `SameHolon` as written.

> **Lemma 0 (total face-invisibility of (PE)).** (PE) at depth 1 says
> `v(f s) = v(g s)` for every `s`. With uniform `μ`, `P_ij` counts
> `|{s : v(s)=i, v(f s)=j}|/|S|`, which is therefore identical for `f` and `g`.
> Hence `f` and `g` share `P`, `M`, `λ`, `W`, their closure status, and every
> quantity any current face computes. *Checked on every witness by
> `verify_break.py`, not assumed.*

So the current tuple returns one object for the pair. The question is whether
one object is the truth.

---

## 1. The trivialisation hazard, named first

The break must not be bought by admitting an observation that presupposes what
is in dispute. If the admissible class contains *arbitrary state-addressed
maps chosen after reading which micro state the system occupies*, every
non-observable holon breaks instantly and the criterion says nothing: an
operator that addresses states the meter cannot distinguish is an operator
whose specification already contains the hidden structure.

The discipline this hunt adopts:

> **ADMISSIBILITY.** An operation is admissible only if it is (i) **blind** —
> applied without conditioning on anything the meter does not report — and
> (ii) **specifiable in a vocabulary the substrate supplies independently of
> the dynamics under test**. A knob may move states the meter cannot separate.
> It may not be *chosen* using knowledge of which of them it is acting on.

Blindness is the whole of the discipline, and it is not a weakening: every
physical knob is blind. An `X` gate on an unread qubit, `local1.py`'s
`zshift_idx(e, k)` (add `k` to edge `e`'s ℤ₃ digit), `gauge_at(st, v, g)`, a
scheduler writing a shard field — all are applied without looking. §5 grades
the classes by how much *more* than blindness they satisfy, so that a break
surviving in a narrow class cannot be dismissed as a smuggled meter.

---

## 2. Extension class I — INTERVENTIONS (blind open-loop perturbation)

**Data.** A substrate is `(S, v, A)` where `A ⊆ S^S` is the knob set. A system
on it is a dynamics. `A` is the *same* on both sides: one lab, one meter, one
set of knobs, two hypotheses about what is inside.

**Experiment.** A word `w = (n_k, a_k, …, n_1, a_1, n_0)`: run `n_0` steps,
apply `a_1`, run `n_1` steps, …, then read `v`. Its readings under `f` and
under `g` are compared from a common start `s`.

**INTERVENTION-DISTINGUISHABLE.** `∃ w, s : v(w_f(s)) ≠ v(w_g(s))`.
Equivalently: in the product automaton on `S × S` with letters
`(x,y) ↦ (f x, g y)` and `(x,y) ↦ (a x, a y)` for `a ∈ A`, some pair reachable
from the diagonal has `v(x) ≠ v(y)`. *Computed exactly by BFS; the shortest
witness word is the BFS path.*

**BREAK (class I).** A pair satisfying (PE) — hence `SameHolon`, hence one
object for every current face by Lemma 0 — that is intervention-distinguishable
by a single knob from an admissible class. The quotient clause is then
automatic and needs no separate test: the identity *identifies* the pair, an
admissible experiment *separates* it, so any quotient that respects the current
identity destroys an admissible observation. Equivalently and more sharply: the
knob **fails to descend to the Moore quotient** — it is an operator that is not
gauge-invariant, which in gauge theory is exactly the evidence that the gauge
was not a gauge.

**Why admissible on the substrates the programme owns.** Three, in ascending
force:

1. *External.* `conformance/gravity/local1.py` already implements
   `zshift_idx(e, k)` and `gauge_at(st, v, g)`: blind, substrate-addressed
   operations on a state space whose views (holonomies, class labels) coarse-
   grain the edge digits. The engine's stabilizer tiers supply Paulis and
   Cliffords on qubits no coarse view reports.
2. *Structural.* The act vocabulary and the read vocabulary are independent
   axes of a laboratory. Every real lab can flip a bit it cannot read. The
   current tuple carries only the read axis.
3. *Internal, and decisive.* The rent face's own definition
   (`CROSSFACE1_PREREG.md` §1) quantifies over **maintenance policies**
   `R(s''|s, s')` — kernels that read the micro pair and write a micro state;
   Theorem 1's attaining policy explicitly *"moves it to a fixed point of block
   `F(v(s))`"*, a distinguished micro state chosen inside a fiber. The tuple
   already treats fiber-addressed operations as performable, in the definition
   of one of its own faces. If fibers are gauge, `W`'s minimisation ranges over
   operations that are not well-defined; if they are performable, class I is
   admissible. The tuple cannot have both.

---

## 3. Extension class II — COMPOSITION (probe the composite with a test holon)

**Data.** A test holon `K = (S_K, v_K, d_K)` and a coupling. The composite has
state `S × S_K` and view `(v, v_K)`; probes are composite probes.

Four couplings, graded by what the interaction reads and writes:

| | composite step | reads | writes |
|---|---|---|---|
| `C_prod` | `(f s, d_K t)` | nothing | nothing |
| `C_viewread` | `(f s, d_K(t, v s))` | the meter | nothing |
| `C_write` | `(a_t(f s), d_K(t, v s))` | the meter | knobs from `A` |
| `C_microread` | `(f s, d_K(t, s))` | the micro state | nothing |

**BREAK (class II).** A (PE) pair whose composites have different composite
probe streams from a common start.

`C_microread` is the physically ordinary case, not an exotic one: an
interaction is specified by its coupling term in the substrate's vocabulary
(a CNOT from a system qubit to an ancilla; a term `d_e · d_{e'}`), never in
the meter's. `C_write` is closed-loop control — a controller with memory,
reading the meter and acting through knobs.

---

## 4. What would make the criterion trivial or vacuous — declared in advance

* **Trivial** if `A` may be chosen after reading the micro state. Excluded by
  ADMISSIBILITY (§1).
* **Trivial** if `A` is allowed to differ between the two sides. Excluded: same
  substrate, same knobs.
* **Vacuous** if the only breaking knobs are ones no substrate supplies. This
  is why §5 grades the classes and §6 stakes the minimal size *per class*: a
  break at the narrowest class is the one that counts.
* **A non-break** if the pair is separated by a current face. Excluded by
  Lemma 0 and re-checked per witness.
* **Not a break, but a rescue** if the answer is "then the view was wrong":
  §7 treats this as a live fork, not an escape, because it converts into a
  standing obligation on every instrument in the programme.

---

## 5. The admissibility ladder for `A` (narrowest first)

| class | definition | what it grants beyond blindness |
|---|---|---|
| `A_prep` | `a = c ∘ v`, `c : V → S` | acts **only** in the meter's vocabulary |
| `A_vpres` | permutations with `v ∘ a = v` | commutes with the meter exactly |
| `A_vcov` | permutations with `v ∘ a = α ∘ v` | descends to the view quotient |
| `A_vcovmap` | all maps with `v ∘ a = α ∘ v` | as above, irreversible allowed |
| `A_perm` | all permutations of `S` | blind and reversible only |
| `A_all` | all maps `S → S` | blind only |

A break at `A_prep` would be impossible on general grounds (a preparation
depends on the state only through the reading). A break at `A_vpres` or
`A_vcov` is the strong one: the knob is definable from `(S, v)` alone, is
compatible with the declared meter, and still separates.

**The gauge-safety condition.** Write `x ~ y` for equality of probe streams
(the Moore partition; identical for `f` and `g` by (PE)). Say `A` **preserves
the gauge** when `x ~ y ⇒ a x ~ a y` for every `a ∈ A`.

---

## 6. Predictions, staked before the search

**T1 (safety).** If `A` preserves the gauge, no `A`-experiment distinguishes a
(PE) pair. *Proof to be machine-confirmed exhaustively; by hand: (PE) gives
`f z ~ g z` for all `z`, and `~` is `f`-invariant, so `x ~ y` is an invariant of
the product reachability, and `~` refines `ker v`.*

**T2.** `A_prep` preserves the gauge always ⇒ **never breaks**.

**T3.** A holon is `Omega.Closed` **iff** its Moore partition equals its view
partition **iff** `W(v) = 0`. Hence for a closed view, `A_vcov` preserves the
gauge ⇒ **closed views are safe against every view-covariant knob**. The danger
zone is exactly `W(v) > 0`.

**P4.** Minimal `|S|` for a break with a single blind permutation (`A_perm`):
**3** — and the witness is predicted to be `ProbeConverse.H₁/H₂` itself, the
very pair rung 3 declared gauge, with a single transposition.

**P5.** Minimal `|S|` for a break with a view-covariant permutation
(`A_vcov`): **4**, and it requires a non-closed view (by T3).

**P6.** Minimal `|S|` for a break with a view-preserving permutation
(`A_vpres`): **5** (the breaking knob needs a fiber holding a gauge-degenerate
pair *and* a fiber holding a gauge-split pair, and the view must be non-closed).

**P7 (top of the ladder).** With `A = A_perm` and knobs allowed anywhere in the
word, intervention-equivalence collapses to `f = g` exactly — raw Ω. Hence
identity is a *function of the act vocabulary*, interpolating between the Moore
quotient (`A ⊆ gauge-preserving`) and Ω itself (`A = A_perm`).

**P8 (adaptive = non-adaptive).** Closed-loop control (`C_write`) has exactly
the distinguishing power of open-loop words: until the first disagreement the
controller sees identical readings, so it plays the same letters.

**P9 (composition).** `C_prod` and `C_viewread` never break (the test holon's
trajectory is a function of the `v`-stream, which agrees). `C_microread`
breaks, minimally with `|S_K| = 2` on the `|S| = 3` pair.

**Falsifier of the whole hunt.** If T1 holds *and* every admissible class turns
out to preserve the gauge on every (PE) pair at `|S| ≤ 5`, there is no break
and the report is exhaustive adequacy evidence for `Identity.lean`.

---

## 7. What each outcome costs the tuple

**Break found.** The tuple must grow a face: the **act/couple vocabulary `A`**,
declared per substrate alongside `v`. Identity becomes the Moore quotient of
the *input automaton* `(S, inputs A ∪ {step}, output v)` — rung 3's theorem
verbatim with "dynamics" replaced by "input alphabet", so the ladder extends
rather than collapses, and `Identity.lean`'s own note ("a future face that
measures fibers would REFINE this identity") is discharged as written.

**The rescue fork.** One may instead declare that `v` must be *saturated*: the
view is by definition the finest thing an admissible coupling can extract. This
is honest, and it is not free — it converts into a conformance obligation with
teeth: **every view used anywhere in the programme must be verified saturated
against the knobs its substrate supplies**, and the saturation is computable
(the automaton Moore quotient). Views failing it are not views.

Both branches are definition-level commitments, exactly as rung 3's fork was.
This document does not choose; it makes the choice unavoidable.

---

## 8. Scope

Finite, deterministic, exact. Uniform `μ`. No stochastic kernels: the Born-
kernel substrates of CROSS-FACE-1 need the criterion restated with
distinguishability of output *distributions*, which is a strictly larger
admissible class and can only make breaks easier — so the deterministic hunt is
the conservative one. Exhaustive to `|S| = 5` for every class, `|S| = 6` where
compute permits. No sampling anywhere.
