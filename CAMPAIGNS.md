# CAMPAIGNS — finding the edges of what the holon can do

*The instrument: exact Clifford+T amplitudes in Z[ω] with NO coefficient
envelope (the CRT residue carrier), a machine-checked merge law with
two-directional certificates (convictions never wrong: `digest_convicts`;
misses impossible on the design window: `digest_window_faithful`), a
transposed tier-1 engine at stim parity at n=1024, GPU + mesh, and a Lean 4
layer. Every campaign below names its referee and its kill before it runs —
house discipline. Ranked by impact × feasibility, from the 2026-08-27
six-lane adversarial sweep (verified claims marked in the sweep record).*

## 1. Beat α = 0.3963 — the QPG linear-code question

**The question (verbatim open, QPG, Quantum 5, 606 (2021), appendix):** does a
binary linear code L (length m, dim k < m/2) exist with
log₂χ(L̂)/(m−2k) < log₂(3)/4? Sharpest instance: **χ(cat₈) ≤ 5** (only ≤6
known; ≤5 gives α ≤ 0.3870, a world record unmoved for five years).
**Why us:** χ is exact rank over Z[ω] — floats cannot distinguish rank-5 to
1e−14 from rank 5; an existence hit needs no exhaustion; the search space
([8,3] through [12,5] codes up to equivalence) is mesh-sized.
**Win:** a new world-record simulation exponent, cascading into every
stabilizer-rank paper and the IBM tracker's quoted hardness constants.
**Kill / honest negative:** search exhausts the staked code family with no
hit → publish the certified closure of the length-8 repetition route
(χ(cat₈)=6 needs the exhaustive branch and the two-directional certificate).

## 2. The IBM tracker's fstate instance, on its magic axis

**Target:** `random_graph_sampling/nq70_depth70_checks27_basis_fstate.qasm`
(quantum-advantage-tracker, IBM-staffed, Active, rolling submissions): 70
qubits, 70 face-state rotations (arccos(1/√3) — the F-state QPG analyse),
stabilizer extent 2^23.97, defended against tensor networks at Schmidt rank
2^30 — an axis a stabilizer-rank simulator is structurally oblivious to. The
tracker's own magic-axis hardness figure is a Qiskit Aer extrapolation
(1e7 s ≈ one second per stabilizer term) — an implementation constant, not a
wall. Our exact cost at 70 F-gates: ~2.2e8 terms at the F-state exponent.
**GATE (pre-staked):** a two-day pilot reproducing their Figure 2 at
N=20/30/42 and comparing slopes. Run the pilot regardless — it prices the
exact tier against the incumbent on a public instance.
**Kills:** the `_checks` ancilla variant carries more magic than counted;
CVP scoring needs many amplitudes, not one; the pilot shows their
extrapolation was not the bottleneck. Each kills the submission, none kills
the pilot's value.
**Side audit:** tracker issue #242 claims an exact TN contraction of the
doped instance in complex64 — an exact-arithmetic cross-check of amplitude
batches would be a first-of-kind audit (verify the claim from
arXiv:2608.13110 first).

## 3. Smallest counterexample to stabilizer-fidelity multiplicativity

**The question (verbatim, BBCCGH, Quantum 3, 181 (2019) §5.3):** "whether
stabilizer fidelity is always multiplicative for all Clifford magic states."
Reduces to: is every Clifford magic state stabilizer-aligned? Extent
non-multiplicativity is known only asymptotically and non-constructively
(Heimendahl et al. 2021) — **no explicit finite counterexample exists, and
the first open case is 8 qubits.**
**Why us — best whole-instrument fit:** F(ψ) is a max over a finite
stabilizer set (|S₆|=3.15e8 a 4090 afternoon; |S₈|=4.18e13 the mesh); the
alignment test is exact algebraic equality (needs the CRT carrier); the
exhaustive max is a distributed max-reduction (needs the machine-checked
merge law — and the no-miss direction, `digest_window_faithful`, is exactly
what makes a certified "the maximum is X" a theorem, not an assertion).
**Win either way:** an explicit counterexample converts directly into
cheaper CCZ decompositions (the authors say so); a certified "all Clifford
magic states at ≤6 qubits are aligned" is also a first.
**Bonus in the same paper:** the CCZ-hypergraph transversal-number
conjecture — anticipated, unproven, cheap to test at small n.

## 4. The 3-qubit certified T-count frontier: 6 → 8–10

Gosset–Kliuchnikov–Mosca–Russo's exhaustive meet-in-the-middle (2013) proved
Toffoli=7 and certified the n=3 frontier at T≤6; **it has not moved in 13
years**. Glaudell et al. (Jan 2026) pushed n=2 to T≤12/18 with packed exact
arithmetic and named cluster-scale runs as the next step — with an
explicitly non-cryptographic dedup path, which is precisely the soundness
gap our certificates close. Rebuild the coset MITM on the mesh with the CRT
carrier and the certified merge; depth k certifies optimality to T=2k.
**Also in range:** Gidney's overlapping-Toffoli question (8 T known, lower
bound 5 — the {5,6,7} gap); auditing AlphaTensor-Quantum's unverified gadget
mappings (the field has one prior confirmed miscount: qcla-mod7).

## 5. The verified constant-memory streaming LRAT checker (Lean 4)

Named as the plan of record by the Lean KS-bound formalization
(arXiv:2607.26413: 40.3 TiB of certificate, two uncertified order-23
obligations tracked as machine-readable goals) and independently as "the
next concrete engineering target" by Lean-QEC (arXiv:2605.16523). One
artifact, two papers' stated needs; the machine-checked merge law is already
a large fraction of its soundness argument. Days-scale first deliverable:
the two order-23 obligations.

## Side-bets (a day or two each)

- **Cryptanalyse Clifford obfuscation** (Bin Yan, arXiv:2608.15963, days
  old): a heuristic hardness claim about Clifford-plus-injected-magic where
  "direct simulation" — us — is the named adversary. High signal either way.
- **Answer MQT Bench issue #924**: the maintainer asked for a reference-
  output design and said dense statevectors won't scale; a poly(n)-memory
  pointwise amplitude oracle dissolves the stated objection. Unanswered.
- **QED-C issue #692**: stale reference data, suite capped at 20 qubits —
  an MPS-tier fix in a NIST-convened suite.
- **DARPA QBI IV&V (DARPA-PA-26-02-01, deadline 2026-10-15)**: best
  whole-instrument institutional fit; pull the SAM.gov text by hand.

## Closed doors, recorded so nobody re-opens them

Arbitrary-angle families (RCS, peaked circuits, certified-randomness
pipelines at 56q) are out of the exact tier's scope — one generic rotation
costs ~3log₂(1/ε) T. The asymptotic stabilizer-rank LOWER bound cannot be
moved by finite search (Ω(n) PSV 2022 / Ω̃(n²) approximate, Mehraban–
Tahmasbi 2024). KS-set minimality is settled (Xu–Chen–Gühne, PRL 2020).
The IBM utility experiment is settled; Willow RCS refutation is crowded.
Labib–Russo 2026 MATCHES the qubit exponent on a different orbit (face
state) — it is not a 4→3 block for Clifford+T (verified twice,
independently: `conformance/srank/verify.py`).
