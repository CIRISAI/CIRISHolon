# QASM-1 — verdict: **CONFIDENCE**, seven of seven.

The boundary sits exactly where the theorems put it, measured against
external exact ground truth (qiskit statevector probabilities).

| arm | reading | verdict |
|---|---|---|
| Q1 retract conformance | max err **0.0** over 200 fresh classical circuits, all routed Classical | **PASS** |
| Q2 tableau conformance | max err **0.0** over 200 fresh Clifford circuits, all routed Tableau | **PASS** |
| Q3 carrier conformance | max err **0.0** over 100 fresh magic-stratum circuits | **PASS** |
| Q4a poly tier | tableau wall-time log-log slope **2.15** (band ≤ 4) over n = 16…256 | **PASS** |
| Q4b exponential carrier | statevector log2-slope **1.101 s/qubit** (band ≥ 0.5) — cost doubles per qubit | **PASS** |
| Q4c the cliff | n = 256, depth 5120 Clifford in **0.0241 s**, where the carrier needs 2²⁵⁶ amplitudes | **PASS** |
| Q4d refusal | non-Clifford at n = 30 exits rc 3 naming `tableau_not_closed_under_rotation` | **PASS** |

## Deviations, disclosed

- Q1's generator crashed pre-reading (ccx drawn at n = 2); an arity guard was
  added before any Q1 circuit was adjudicated. Q2/Q3 ran under the frozen
  code and stand.
- Q3's stratum draws gates at random, so 2/100 circuits (measured post hoc) happened to contain
  no T/Tdg and routed Tableau; conformance read 0.0 on both tiers; the
  T-presence guarantee was not enforced by the generator.
- The bench initially timed the DISTRIBUTION mode, which enumerates
  measurement branches — 2^256 of them at n = 256: the harness had rebuilt
  the exponential wall inside the poly tier. The timing path is now a single
  deterministic shot; the lesson is recorded in the crate.

## What this is

Milestone one of the QASM path, real: three strata conforming EXACTLY to
external truth, the poly/exponential boundary measured on both sides of the
tableau wall, and the wall refusing by name with its Lean witness. The
conformance failures this campaign could have produced would have convicted
tier implementations against their contracts (engineering convictions); none
occurred. Gauge lessons banked: echoes cannot gauge Clifford mutants (a
consistently wrong implementation passes its own inverse), and phase mutants
need pinned witnesses. OWED, named: the stabilizer-rank tier (CH-form) that
prices magic by T-count — the magic-cost scaling stake waits on it.
