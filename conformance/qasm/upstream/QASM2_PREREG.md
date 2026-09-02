# Pre-registration — QASM-2: the magic tier, priced by T-count

**2026-08-27, frozen before any staked batch; admissible only if the audit
exits 0.** The stabilizer-rank-style tier: Clifford+T as an EXACT branch sum
of phase-tracked affine stabilizer states (Dehaene–De Moor / Van den Nest
form; exact Z[ω] arithmetic with power-of-√2 denominators — no floating point
until output). The staked claims: the tier conforms exactly to external
truth, its cost is priced by MAGIC (2 per T-gate) and only polynomially by
qubits — including qubit counts past the statevector cap — and exactness is
an arithmetic invariant. Naive 2^t branch sum; the Bravyi–Gosset rank
reduction (2^{~0.48t}) is credited and named as the next improvement, not
claimed.

Dev-loop record (pre-freeze instrument development, disclosed): four real
bugs found and fixed against the in-crate oracle — a zero-alignment hang, a
normalize-vs-alignment exponent cycle, an H-gate column-dependence invariant
break (pure-Clifford failure at err 0.375, now guarded by a loud rank
assertion), and a mod-2/mod-4 confusion in the odd-δ Gauss sum's factorized
form (the XOR expansion's pairwise terms). Final property record: 960/960
random Clifford+T circuits at ≤ 1.4e-15.

defects: D-DET (exact deterministic computation; conformance against an
external exact reference), D-UNITS (probabilities absolute; seconds;
dimensionless slopes).

gauge: scratchpad/qasm/gauge_magic.log

Family-wise: Bonferroni over the 5 staked readings.

| arm | stake (numeric) | witness: | posability |
|---|---|---|---|
| M1 conformance | 150 fresh magic-stratum circuits (seed 20260831, n ≤ 6, t ≤ 8 enforced): max abs probability error ≤ 1e-9 vs qiskit, with the Magic tier among the routed tiers | witness: `tableau_not_closed_under_rotation` — the wall this tier prices its way past, branch by branch | both planted mutations fire on seeded witnesses (gauge_magic.log) |
| M2 T-scaling | amplitude-query wall-time log2-slope over t = 5…12 at n = 10 ∈ [0.8, 1.2] per T-gate | witness: none (the 2^t branch count, measured) | slope free to read anything; a sub-exponential reading convicts the naive-sum claim |
| M3 n-scaling | amplitude-query log-log slope over n = 12…32 at t = 6 ≤ 4 — with n = 28, 32 PAST the statevector cap | witness: none (poly(n) per branch: the T-count, not the qubit count, is the price) | a super-poly reading fires; this is the claim a statevector cannot even pose |
| M4 exactness | max |Σ probabilities − 1| ≤ 1e-12 over the M1 batch | witness: none (exact arithmetic makes unitarity an invariant, not a tolerance) | a single dropped phase breaks it (the mutants do) |
| M5 refusal honesty | the router refuses non-Clifford at n > 24 only when t > 12; at t ≤ 12 it routes Magic at ANY n | witness: `tableau_not_closed_under_rotation` (the refusal's name) | route checks on constructed circuits both sides |
