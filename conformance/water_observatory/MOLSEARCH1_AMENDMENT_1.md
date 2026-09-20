# MOLSEARCH-1 — AMENDMENT 1: closed is not slow; the molecule is a block, not a top rank

*Written 2026-09-20 on building the reader, from plant PM-1, BEFORE the model's walk is
read. The prereg is kept as written.*

PM-1's synthetic carrier — rigid rotors decorrelating in 200 fs plus harmonic stretches of
9 fs period — read the STRETCHES as the most closed directions (`σ = 0.98` at 50 fs) and the
rotors below them (`0.74`). That is not a fault of the reader: a harmonic mode with its rate
is an exactly closed two-dimensional linear view at EVERY lag (`Closed`, with `h` a rotation),
and the variational score ranks by closure, not by slowness. The prereg's S1 and S2 staked
"the slow directions are the rigid ones" and "the vibrations are not closed at 50 fs" — the
first is a slowness claim dressed as a closure claim, the second is false by theorem for a
harmonic vibration. The fourteenth instance: a stake on a quantity a theorem holds fixed.

What the molecule tier's object actually is, to the search: **two closed sectors that do not
couple.** The rigid modes (centre-of-mass and angular velocity) form one closed view; the
vibrations form another; the molecule as the rigid operator's unit is licensed exactly when
the Koopman matrix is BLOCK-DIAGONAL between them, so that dropping the vibrational sector
costs only the cross-block coupling — which is REPLACE-0's measured price, not a free
assumption. That is the search's version of "the molecule is a rigid unit".

**Re-staked.** S1′: at every lag, each of the dictionary's top-6 singular functions has
weight `≥ 0.9` in ONE of the two subspaces (rigid or internal) — no mixed direction. **Kill:**
a top-6 singular function with weight under `0.7` in both, a vibration–rotation mode the
rigid operator cannot omit. S2′: the cross-block Frobenius norm of the whitened Koopman
`‖K_RI‖ / ‖K‖ < 0.1` at `τ = 50` fs — the coupling that the rigid operator drops. **Kill:**
`≥ 0.3`. S3 stands (the rigid view's fraction of the bound at `k = 6`, now against the bound
restricted to its own sector's rank where the two sectors are separate: reported both ways).
PM-1 is re-read on the theorem: stretches at `σ ≈ 1`, rotors at `e^{−50/200} = 0.78`, every
singular function in one subspace, cross-block norm at noise.

## A2 — the carrier is 128 waters, not 16 (found on launching)

The prereg's 16-water box (`n_cells = 2`, half-edge `7.4` bohr) is REFUSED by the periodic
gate: the law's reach is `14` bohr, and a box whose half-edge is under it sees its own
images. The smallest admitted box is `n_cells = 4`, 128 waters, 384 atoms — the walk is
1,000 settle frames then 40,000 NVE frames (`1.05` ps) at a row every `1.0` fs, 128
molecules × 1,000 rows as the ensemble, held out in four folds of 32 molecules. The
prereg's 16 × 2,000 was a price set without the gate; the gate was right.
