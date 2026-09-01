# C2 — REAL-TIME MPS ELECTRONIC DYNAMICS: the carrier, and what its gates measured

**Status: the carrier is MATERIALIZED and GATED.** WB-8.3's third node exists, runs,
and is checked against an independent referee. Banked 2026-09-01.

Instrument: `engine/crates/q8-mps/src/tdvp.rs` (single-site TDVP, complex, zero
dependencies). Gates: `engine/crates/q8-mps/tests/c2_tdvp_gates.rs`. Carrier shell:
`engine/crates/holon-chem/src/tower.rs` §10 (`C2_MpsTdvp`). CI: `engine/ci-gates.sh`
gate 16b. This document is not a prereg — the campaign it would pre-register is
`DIMER_PREREG.md`; this is the record of an instrument being built and tested, including
the part of its stake that was falsified.

---

## 1. What was staked, and what the first run did to it

The stake was frozen in the `const` block of the gate file before the first run. One prong
of it was **wrong**, and it is recorded here rather than quietly edited, because the
correction is the most informative thing the campaign produced.

**Staked:** at the natural bond cap, the MPS manifold is the whole Hilbert space, the
tangent projector is the identity, and the only remaining error is the integrator's
second-order splitting error — so the trajectory error should fall as `dt²`.

**Measured, first run:** the error was flat at `7.1e-14 … 3.1e-13` across a factor of
eight in step size. Fitted "order": `−0.64`. The gate fired on its own author.

**Why:** the projector-splitting integrator has the **exactness property** — when the
manifold contains the exact trajectory, the splitting reproduces it with *no step-size
error at all* (Lubich–Oseledets 2014 for the matrix case; Lubich–Vandereycken–Walach and
Haegeman et al. PRB 94 165116 (2016) for tensor networks). At the cap the manifold is
everything, so the integrator is exact, not second-order. The staked claim was strictly
weaker than the truth.

**What changed.** The trajectory gate was split in two, and the second half only exists
because the first prong failed:

* `G-C2-2a` gates **exactness at the cap** — a flat ceiling the integrator must sit under
  however coarsely it is stepped. Strictly stronger than the band it replaced.
* `G-C2-2b` moves the **order** measurement below the cap, where the manifold is a genuine
  submanifold, the flow is the TDVP flow rather than the Schrödinger flow, and an order is
  a meaningful thing to read. Its referee is a fine-step run of the *clean* integrator, not
  the exact propagator.

That split is load-bearing for the mutation battery: **`ForwardSweepOnly` is exact at the
cap too.** The exactness property does not care about sweep direction, so the palindrome
looks free there and its removal is invisible to `G-C2-2a`. Had the first stake stood, one
of the four planted defects would have had no sensor at all — and the gate battery would
have looked complete while missing the one bug that turns a second-order integrator into a
first-order one.

---

## 2. The system, and why the referee is independent

Hubbard chain, 3 chain sites = 6 Jordan–Wigner spin-orbitals, Hilbert dimension 64,
`t = 1.0`, `U = 4.0`, `mu = 0`. Initial state: a deterministic full-rank complex MPS at the
declared cap, seed `20260901` pinned in the source (an LCG, not an RNG — no gate in this
crate depends on a seed the runner chooses).

The referee shares **no code** with the thing under test. `Mpo::dense` builds `H` by
Kronecker products; `jacobi_eigen` diagonalises it; `exact_propagate` applies one phase per
eigenvalue. `Tdvp::step` contracts environments and Krylov-exponentiates effective
Hamiltonians. Two implementations agreeing is evidence; one implementation agreeing with
itself is not. `G-C2-0a` additionally pins the builder's MPO against `dense_from_mpo`, the
object the crate's own G1 gate already validated, at `1e-13`.

---

## 3. The gates and their readings

| gate | staked | measured | verdict |
|---|---|---|---|
| G-C2-0a | MPO builder vs `dense_from_mpo`, `≤ 1e-13` | agrees | PASS |
| G-C2-0b | canonicalised start normalised, `≤ 1e-12` | agrees | PASS |
| G-C2-0c | MPS energy vs dense energy, `≤ 1e-11` relative | agrees | PASS |
| G-C2-0d | referee is the identity at `t=0` and unitary after | agrees | PASS |
| G-C2-1a | norm deviation `≤ 1e-12` | `6.9e-14 … 2.9e-13` | PASS |
| G-C2-1b | energy drift `≤ 1e-10` relative | `5.4e-15 … 7.2e-14` | PASS |
| G-C2-1c | Krylov subspace truncations `= 0` (EXACT) | 0, worst estimate `2.3e-14` | PASS |
| G-C2-2a | distance to the exact propagator `≤ 1e-11` at EVERY step size | `7.1e-14` (n=4) … `3.1e-13` (n=32) | PASS |
| G-C2-2b | self-convergence order in `[1.70, 2.30]` below the cap | `2.0063`, `2.0055`, `2.0174` | PASS |
| G-C2-2b | finest-step self-error `≤ 1e-4` | `6.36e-6` | PASS |
| G-C2-4 | the state is genuinely complex and genuinely moves | imaginary weight non-trivial, distance moved `> 0.1` | PASS |

`G-C2-1a/b` are round-off bands, not physics bands: every substep applies
`exp(-i θ H_eff)` with `H_eff` Hermitian to the orthogonality centre of a canonical MPS,
so the norm and the energy are conserved *exactly*. Drift there is a bug and never a
step-size effect, which is what makes those two the sharpest sensors in the battery.

The `chi = 4` submanifold used by `G-C2-2b` is a real restriction and its cost is reported
rather than hidden: the fine-step reference sits `8.3e-2` away from the exact propagator.
That is the **projection error** — what a `chi = 4` manifold cannot represent — and it does
not shrink with `dt`. Quoting it beside the `6.4e-6` step error is the point: the two
errors are different objects and only one of them is the integrator's.

---

## 4. The gates are demonstrated FIRING

Four planted defects, each a bug a person actually writes when implementing this
integrator. Each is required to break a **named** prong, because an OR-gate that only ever
fires on one disjunct is a one-prong gate wearing a disguise.

| planted defect | named prong | measured |
|---|---|---|
| `NoBondBackstep` — "the projector is just the sum over sites" | G-C2-2a exactness | err `1.44` (vs `3e-13` clean) |
| `BondBackstepWrongSign` — the subtracted term's sign | G-C2-2a exactness | err `1.41` |
| `ForwardSweepOnly` — Lie instead of Strang | G-C2-2b order below the cap | order `1.022` (vs `2.006` clean) |
| `LeftEnvNoConjugate` — no bra conjugation in the left environment | G-C2-1b energy | drift `0.397` (vs `3e-14` clean) |

`LeftEnvNoConjugate` is the one worth naming twice: on real arithmetic it is **invisible**,
which is exactly why it is planted. A real-valued MPS test suite cannot see it, and it
makes `H_eff` non-Hermitian, so the sensor that catches it is the energy — the quantity
that is conserved by construction.

The control is checked too: the unmutated run must fire *no* prong, so the battery is
measuring the mutation and not the instrument.

---

## 5. The price, measured

`M-CHEAPER-THAN-ITS-PRICE` says the banked cost model is itself a falsifying check, so
`C2_MpsTdvp::price_per_substep` is a measurement, not a model. Measured on this box,
2026-09-01, `--release`, Hubbard MPO (`D ≤ 7`), at the natural cap `chi = 2^(L/2)`:

| L | chi | s/step | implied `c` in `t = c · L · chi³` |
|---|---|---|---|
| 6 | 8 | 0.004499 | 3.7e-7 |
| 8 | 16 | 0.012031 | 3.7e-7 |
| 10 | 32 | 0.103841 | 3.2e-7 |
| 12 | 64 | 0.689846 | 2.2e-7 |

`c = 3.7e-7 s` is banked — the **largest** of the four, so the price is an upper bound
rather than an average a caller can be surprised by. The spread is a factor 1.7 across
three decades of work. A wall clock on this heterogeneous box carries
`M-PLACEMENT-LOTTERY`: this number specifies an order of magnitude and a scaling, not a
benchmark, and the measurement itself is `#[ignore]`d so no gate ever decides on it.

---

## 6. What is fenced, by name

Two fences, both typed and both visible in `tower.rs` rather than living in a comment.
WB-8.4 says a fence is transient — pay the price, climb, it lifts — so each names its
discharge route.

1. **The C1 → C2 climb is not built** (`c1_to_c2_transport_capability`, a
   `Capability::Stub`). C2 is a certified NODE; it is not yet reachable as an EDGE. The
   picture change needs a state lift (nuclear configuration to an electronic MPS in a
   basis), an operator picture change (a bead-averaged potential is not a second-quantised
   Hamiltonian), and a *measured* commuting certificate. None of the three exists. A node
   and an edge are different objects and the tower now says so with different types.

2. **Single-site TDVP cannot grow a bond dimension.** The tangent space at `A` only
   contains directions the existing bond can represent, so a rank-deficient start — a
   product state zero-padded up to `chi` — stays rank-deficient forever and `H_eff` is
   singular on the padding. `deterministic_state` therefore builds a full-rank start, and
   a `pad_to_chi`-style zero padding is deliberately **not** offered in this module. The
   discharge route is two-site TDVP, which can grow bonds at the price of a truncation,
   and it is not built.

Neither fence is a caveat about the readings above. Both are statements about what the
carrier can be *asked*, and both are `Capability`-typed so that asking is a refusal rather
than a wrong answer.

---

## 7. What this does NOT establish

The gates establish that the integrator integrates. They say nothing about chemistry.
Every number above is on a Hubbard model chosen because it has an affordable exact
referee; no molecular Hamiltonian, no basis-set question, and no electronic seam has been
touched by this carrier. `q8-mps`'s `Mpo::from_electronic_integrals` is the path from here
to a molecule and this campaign did not walk it.

The reproducibility discipline applies as it does everywhere (`M-STALE-INSTRUMENT`): the
instrument and its gates are committed with this document, and the numbers above are from
`cargo test --release -p q8-mps --test c2_tdvp_gates` at that commit. A results document
without its instrument's commit is not banked.
