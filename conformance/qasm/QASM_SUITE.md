# The QASM conformance suite

The stratified simulator's standing acceptance record and harness. Twelve
pre-registered arms, adjudicated upstream (CIRISOntology `scratchpad/qasm/`:
QASM-1 seven of seven, QASM-2 five of five), all CONFIDENCE:

- **Conformance**: max probability error 0.0 vs qiskit exact statevector over
  650 fresh seeded circuits across all four tiers (classical, tableau, magic,
  carrier), every circuit routed to its predicted tier.
- **The boundary**: tableau poly at log-log slope 2.15 (256 Clifford qubits,
  depth 5120, 24 ms — where the carrier needs 2^256 amplitudes); carrier
  exponential at 1.10 log2-seconds/qubit; magic tier at 1.005 log2-seconds
  per T-gate and poly 1.26 in qubits through n = 32, past the carrier cap.
  Closure violations price simulation — magic exponentially, qubits
  polynomially — measured on both axes.
- **Exactness**: unitarity defect exactly 0.0 (Z[ω] arithmetic; a tolerance
  nowhere, an invariant everywhere).
- **Refusal honesty**: past every tier's budget the router refuses naming
  `tableau_not_closed_under_rotation` and the T-price.

In-crate CI: `cargo test -p holon-qasm` runs the three-way tier-vs-carrier
conformance plus planted-mutation detection with no external deps. The
python harness here is the external referee (needs qiskit); its gauge lessons
are recorded in the upstream preregs: echoes cannot gauge Clifford mutants,
phase mutants need pinned witnesses, and the distribution mode must never be
used as a timing path (it enumerates measurement branches).

Named next steps: the Bravyi–Gosset rank reduction (2^{~0.48t}); the
contextuality/cost link is NOT posable on qubit circuits with the current
instruments (see upstream scope note) — the magic-contextuality equivalence
lives in odd prime dimensions (Howard et al. 2014).
