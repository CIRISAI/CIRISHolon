# Pre-registration — QASM-1: the boundary, on external ground truth

**2026-08-27, frozen before any staked batch runs; admissible only if the
audit exits 0.** The stratified QASM simulator (CIRISHolon
`engine/crates/holon-qasm`, zero-dep) against qiskit's exact probabilities as
EXTERNAL reference. The claim under test is the ladder's signature: closure,
not "quantumness", prices simulation — the tableau view is Closed under
Clifford motions and runs where the carrier cannot exist, and the wall
refuses by name. Staked seeds are frozen here; every staked circuit is
generated AT adjudication, never seen before.

defects: D-DET (the tiers are deterministic exact computations — conformance
is against an external exact reference on full distributions, no statistics,
no floors), D-UNITS (errors are absolute probabilities; timings in seconds;
slopes are log-log dimensionless).

gauge: scratchpad/qasm/gauge_qasm.log

Family-wise: Bonferroni over the 7 staked readings. Recorded gauge lessons:
echo circuits cannot gauge Clifford mutants (a consistently wrong
implementation passes its own inverse — 0/40 measured); phase mutants are
rare-visible in random circuits (1–2/40), so detection is gauged on three
PINNED firing witnesses per mutant found by seeded search.

| arm | stake (numeric) | witness: | posability |
|---|---|---|---|
| Q1 retract conformance | 200 fresh classical circuits (seed 20260827, n ≤ 10, depth ≤ 60): max abs probability error ≤ 1e-9 vs qiskit, every circuit routed Classical | witness: `lift_commutes` — the diagonal tier IS the retract, and this is its engineering conformance | the cx-swap mutant fires at err 1.0 (pinned) |
| Q2 tableau conformance | 200 fresh Clifford circuits (seed 20260828, n ≤ 8, depth ≤ 60): max error ≤ 1e-9, every circuit routed Tableau | witness: `tableau_closed_under_hadamard` — the n-qubit engineering face of the stabilizer closure; algorithm Aaronson–Gottesman 2004, credited | s-phase and cx-phase mutants fire on pinned witnesses (err 0.5–1.0) |
| Q3 carrier conformance | 100 fresh magic circuits (seed 20260829, n ≤ 8, T/Tdg present): max error ≤ 1e-9 on the statevector tier | witness: none (reference-tier check) | same harness, same mutant sensitivity |
| Q4a poly scaling | tableau wall-time log-log slope over n ∈ {16,…,256} at depth 20n ≤ 4 | witness: none (the O(n²)-per-gate claim, measured) | slope varies freely; a super-poly implementation fires |
| Q4b exponential carrier | statevector log2(seconds) slope over n ∈ {16,…,24} ≥ 0.5 per qubit | witness: none (the 2ⁿ carrier cost, measured) | a sub-exponential reading here would convict the bench, not physics |
| Q4c the cliff | tableau at n = 256, depth 5120 completes in < 60 s — a scale at which the carrier (2²⁵⁶ amplitudes) cannot exist in this universe's memory | witness: `tableau_closed_under_hadamard` — the Closed view beats its carrier exponentially INSIDE its scope | trivially fails if the tableau is not actually poly |
| Q4d refusal | a non-Clifford circuit at n = 30 exits with the REFUSAL naming `tableau_not_closed_under_rotation` (rc 3) | witness: `tableau_not_closed_under_rotation` — the wall, refusing by name | a silent fallback or crash fires this arm |

Exits: CONFIDENCE — the strata conform to external truth exactly, the
boundary sits where the theorems put it, and the wall refuses honestly:
milestone one of the QASM path is real. FALSIFICATION — a conformance miss
convicts the tier implementation against its Lean contract (an engineering
conviction, reported as such, distinct from a physics kill); a boundary miss
convicts the cost claims. Owed and named, not staked: the stabilizer-rank
tier (CH-form phase tracking) that prices magic by T-count, and with it the
magic-cost scaling stake.
