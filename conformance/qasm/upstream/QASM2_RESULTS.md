# QASM-2 — verdict: **CONFIDENCE**, five of five. Magic is priced by T-count, measured.

| arm | reading | band | verdict |
|---|---|---|---|
| M1 conformance | max err **0.0** over 150 fresh circuits vs qiskit, Magic tier routed | ≤ 1e-9 | **PASS** |
| M2 T-scaling | **1.005 log2-seconds per T-gate** at n = 10 — the 2^t branch law to half a percent | [0.8, 1.2] | **PASS** |
| M3 n-scaling | log-log slope **1.261** at t = 6 over n = 12…32 — n = 28, 32 are PAST the statevector cap (n = 32 answered in 0.8 ms where the carrier needs 68 GB) | ≤ 4 | **PASS** |
| M4 exactness | max |Σp − 1| = **0.0** — unitarity as an arithmetic invariant of the Z[ω] representation | ≤ 1e-12 | **PASS** |
| M5 refusal honesty | Magic at any n when t ≤ 12 (n = 40 routed); refusal names the wall AND the T-budget when t > 12 past the carrier cap; statevector when it fits | exact | **PASS** (the refusal message was updated post-verdict to stop calling the now-existing tier "owed" — wording, not adjudication) |

## What was learned building it (the dev-loop record, disclosed in the freeze)

Four real bugs, each found by the oracle before any staked run: a
zero-alignment hang; a normalize-vs-alignment exponent cycle; an H-gate
COLUMN-DEPENDENCE invariant break that failed pure Clifford circuits at err
0.375 (now guarded by a loud rank assertion — a silent wrong answer became an
impossible one); and a mod-2/mod-4 confusion in the odd-δ Gauss sum — the
factorized form (1+i^δ)(−i^δ)^Λ is only valid for Λ ∈ {0,1}, and the correct
XOR expansion needs pairwise (−1)^{u_a u_b} terms across the coupling set,
exactly like the S gate. Final property record: 960/960 at ≤ 1.4e-15.

## The stratified simulator, complete at its first scope

Four tiers now run behind one router: classical bits, Aaronson–Gottesman
tableau, exact Z[ω] stabilizer branch-sum priced at 2 per T-gate, and the
statevector carrier — with refusal by name when every tier's budget is
exceeded. The QASM-1 owed item is PAID. Credits: Dehaene–De Moor and Van den
Nest for the affine form; Aaronson–Gottesman for the tableau;
Bravyi–Gosset for the stabilizer-rank programme whose 2^{~0.48t} reduction is
the named next improvement, not claimed. The measured claim the season adds:
on this simulator, simulation cost is priced by CLOSURE VIOLATIONS — T-count
at slope 1.005, qubits at slope 1.26 — which is the ladder's signature made
quantitative on both axes at once.
