# QVM-ACUITY-1 — READ: the cheap sector is found by a closure search, the hard part is priced before it runs, and the budgeted sum meets every acuity with a certificate that never lied — branch (a), with one clause of S4 missed on a seven-core box

*2026-09-22. Prereg `conformance/qasm/QVM_ACUITY1_PREREG.md` (frozen alone, `65dafcb`; two
"Notes on building" sections appended by the two halves, no stake moved). Instruments:
`holon::sector` (the cheap part), `holon::acuity` (the hard part), `qvm_acuity` (the driver),
`qvm_acuity_sweep.py`; results `qvm_acuity_results.json`, `qvm_acuity_table.md` (450 runs on
cores 21–27, sweep wall 218 s, loadavg 10.6). Tests: `tests/qvm_acuity_hard.rs`,
`tests/qvm_acuity_cheap.rs`, all green with the crate's suite.*

## 0. Verdict

| stake | staked | read | verdict |
|---|---|---|---|
| **S1** the cheap sector located, not named | dropping every removed T-gate changes the observable by `≤ 10⁻¹²`; kill on one gate | the search reads each gate's unitary out of the engine and tests the tableau view's closure by Pauli conjugation (one term and a tracked phase ⇒ closed; CZ, √X, a phased H found cheap; Toffoli, controlled-S, `diag(1, ω)` found hard); 76 T-gates removed on the grid's marginals, worst change `2.6 × 10⁻¹⁵`, tested per gate; dropping a KEPT gate moves the amplitude by `10⁻³`–`10⁻²` on 27 of 27 | **MET** |
| **S2** the price before the run | `N_exec = N_pred` at `ε = 0` | `12, 36, 108, 324` at `t = 8, 12, 16, 20`, exactly, under both bounds; the `ε = 0` value bit-identical to the mesh fold | **MET** |
| **S3** the certificate | `\|value − referee\| ≤ R_k ≤ ε` on every instance with a referee | on all 450 sweep rows and the 18 test instances against two independent referees (the full exact sum; a dense statevector sharing no code) — never violated; worst `R_k / error` 29 | **MET** |
| **S4** cost follows the price; the mesh pays | executed fraction falls with `ε`; wall `≤ 0.35 ×` at `S = 4`, `≤ 0.2 ×` at `S = 8`; kill `> 0.5 ×` | fractions in §1; mesh `0.27 ×` at `S = 4` (MET), **`0.25 ×` at `S = 8` on SEVEN cores** (floor `0.143`; the stake's `0.2` was written for eight cores on a seven-core range — the prereg's own inconsistency, noted by both halves); bit-identical at every `S` | **S = 4 MET; S = 8 clause missed on the prereg's arithmetic, kill not triggered** |
| **S5** the comparison class | full-sum exponent within `0.05` of `0.396`; walls beside each other | branch-count exponent **`0.3962`** (gap `0.0000`); wall exponent `0.46`, the difference `0.066` being the gadget's per-branch cost (an `n + t`-wide register), not the decomposition; the three walls in §1 | **MET on the count; the wall's excess named** |

**Branch (a).** The general QVM's claim is MEASURED on Clifford+T: the closed sector is found
by the same closure test the object is built on, the +1 is priced by `expected_branches`
before a branch runs, and the sum is cut at a remainder the acuity licenses, with the
certificate holding on every one of 450 runs. Plants PQ-1 to PQ-5 all fire, two of them
exactly (the remainder bound equals the true remainder when signs align and is `3.000 ×` it
when they alternate).

## 1. The numbers that matter

**Executed fraction `k/N` against acuity** (mean over seeds; the certified per-branch bound):

| `n` | `ε = 10⁻¹` | `10⁻²` | `10⁻³` |
|---|---|---|---|
| 12 | 0.17–0.97 (rising with `t`) | 0.92–1.00 | 1.00 |
| 16 | 0.00–0.90 | 0.75–0.99 | 0.97–1.00 |
| 20 | 0.00–0.21 | 0.08–0.82 | 0.92–0.98 |
| 24, `t = 24` (test) | 0.00 | 0.60 | 0.94 |

A row with `k = 0` is the machine working: `|⟨y|C|0⟩| ~ 2^{−n/2}` is `10⁻³` at `n = 20`, so an
absolute `ε = 10⁻¹` is met by the answer `0` with a certificate of `0.03`. **The prereg's
acuity ladder was absolute where the observable scales as `2^{−n/2}`**; a relative ladder
(`ε × 2^{−n/2}`) would have made the three rungs mean the same thing at every `n`. Recorded
by both halves; the next freeze uses it.

**Walls, one core** (`n = 20`): the branch sum beats the `2²⁰` statevector by `160 ×` at `t = 8`
(`2.6` ms against `418`), matches it near `t = 24`, and loses by `3 ×` at `t = 28`; at `n = 12`
and `16` the statevector is trivial and wins everywhere. The crossing is where the price
says it is: `N(t) · c_branch(n + t)` against `2^n`.

**The mesh on the branch sum, `N = 972`, bit-identical at every `S`:** `0.53 ×` at `S = 2`,
`0.27 ×` at `S = 4`, `0.30 ×` at `S = 7`, `0.25 ×` at `S = 8` on cores `21–27`.

**What the a-priori bound cost.** The prereg's §3 bound — each branch's coefficient times
the normalised amplitude's `≤ 1` — is TRUE and VACUOUS: `R₀ ≈ 61` at `t = 8`, growing as
`2^{t/2}` while the observable shrinks as `2^{−n/2}`; under it `k/N = 1.000` at every `(n, t,
ε)`. The truncation comes from the exact per-branch fact that an affine amplitude is either
exactly its coefficient's magnitude or exactly zero. The certified source installs that
bound; the driver uses it.

## 2. What the search found that the prereg did not expect

- **An amplitude's light cone is the whole register**, so removal bites only on marginals
  (`removed = 0` on all 27 amplitude instances, `76` on the marginals).
- **At `20n` depth a four-qubit marginal is exactly `1/16` on 25 of 27 instances**, for an
  exact reason (every non-trivial element of `⟨Z₀…Z₃⟩` conjugates back to Paulis carrying an
  `X` or `Y`), so S1 on that observable was settled against a constant — hence the per-gate
  kill, the amplitude negative check, and a shallow family (`2n` depth, 30 instances, 223
  removed, worst `4.4 × 10⁻¹⁶`) added on building.
- **The cone is sound, not complete**: a diagonal T after the last entangler cannot matter
  yet sits inside it, so `t_eff` and the price are upper bounds.
- `y = 0ⁿ` is a vacuous observable (exactly zero on some instances): the driver takes the
  argmax bitstring.

## 3. Against the state of the art, said exactly

The arithmetic is Bravyi–Gosset, Qassim–Pashayan–Gosset and Kissinger–van de Wetering–
Vilmart, ported with credit; the branch-count rate is theirs to four digits. What is ours
is what the object adds everywhere: the closed sector found by a closure test rather than a
gate table (and it classifies unitaries the gate enum cannot spell), the price printed
before the run and checked against the executed count, and the remainder carried as a
gate that a planted wrong branch breaks. No named tool is claimed beaten; the comparison
class is cost against acuity with the referees beside it.

## 4. Owed

The relative acuity ladder; the marginal's own budget (`2^{|L|−|S|}` legs, the remainder
propagating as `2|a|R + R²`); the cone's completeness (diagonal gates after the last
entangler); the eight-core mesh point on eight cores; the face ring and qutrit tiers under
the same three moves; the GPU branch fold.
