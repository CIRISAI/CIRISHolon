# The QASM conformance suite

The stratified simulator's standing acceptance record and harness. Twelve
pre-registered arms, adjudicated upstream (QASM-1 seven of seven, QASM-2
five of five; the adjudication records are MIRRORED IN-REPO at `upstream/`
with sha256 provenance — originals at CIRISOntology `scratchpad/qasm/`),
all CONFIDENCE:

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
- **Refusal honesty**: past every tier's budget the engine refuses naming
  `tableau_not_closed_under_rotation` and the price it declined to pay. This
  claim said *the router* until 2026-09-02, and the router was the only place
  it was true: the battlerig ran into an `amp` path with no T-cap guard at all
  (caveat 1 below), and enumerating the other doors found the same shape twice
  more — the router itself sent any wide low-T circuit to the magic tier's
  DISTRIBUTION path, whose 2^n accumulator asked for 88 TB and dumped core, and
  `--tier statevector` walked past that same wall. The wall now lives at every
  entry point rather than upstream of one, `holon-qasm`'s `tests/refusal.rs`
  holds one test per door, and `mutate_tcap.py` plants the removal of each
  guard and requires the test that names it to fire.

In-crate CI: `cargo test -p holon-qasm` runs the three-way tier-vs-carrier
conformance, the per-door refusal gate, and planted-mutation detection, with no
external deps. The
python harness here is the external referee (needs qiskit); its gauge lessons
are recorded in the upstream preregs: echoes cannot gauge Clifford mutants,
phase mutants need pinned witnesses, and the distribution mode must never be
used as a timing path (it enumerates measurement branches).

Named next steps: the Bravyi–Gosset rank reduction (2^{~0.48t}); the
contextuality/cost link is NOT posable on qubit circuits with the current
instruments (see upstream scope note) — the magic-contextuality equivalence
lives in odd prime dimensions (Howard et al. 2014).
