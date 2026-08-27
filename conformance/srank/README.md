# Stabilizer-rank sweep — verification artifacts

Numerical verifications behind the magic tier's benchmark ladder (TIERS.md),
produced by the 2026-08-27 adversarial prior-art sweep. All checks are exact
(machine-precision fidelities of published identities), runnable standalone.

| script | what it verifies |
|---|---|
| `verify_qpg.py` | Qassim–Pashayan–Gosset (arXiv:2106.07740): cat₂/cat₆ identities, χ(T^⊗6)≤6, the cat-chain contraction — all fidelity 1.0 |
| `verify.py` | the Labib–Russo trap (arXiv:2605.28586): their 3-term 4-copy identity is EXACT for the Bravyi–Kitaev FACE state (fidelity 1.0) and WRONG for the π/8 state (0.8249) — their table must not be imported as a Clifford+T block |
| `gains2.py` | the DP over decomposition rules; reproduces the published 6,377,292 terms at t=56 (Wan–Zhong–Zapirain's cited prior-art count) |
