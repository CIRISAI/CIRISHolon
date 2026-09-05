# FIELD-5 — results

*Freeze `FIELD5_PREREG.md` (1888798, alone). Instrument `holon-chem/src/heitler_london.rs`
(`heitler_london`, `fci_in_hl_basis`, the plant; unit-tested on a hydrogen-molecule pair:
the full CI in the orthogonalised basis reproduces the supermolecule to `1e-9`, the plant
misses by more than `1e-3`, the product state is normalised and vanishes at distance);
harvest `examples/field5_harvest.rs` (delegate, to the freeze); FIELD-4's engine unchanged.
JSON under `field5/`.*

## The verdict, first

**The instrument is right and its referee is deformed.** Every product state on the six
water nodes is normalised to `4e-15`, has exactly `441 × 441 = 194,481` nonzero
determinants of `1,002,001`, and at 40 bohr its exchange reads `8e-12` — zero — while one
Hamiltonian application on the million-determinant space costs 3 seconds. But the exchange
it measures cannot be a wall: `+40` mHa at the hydrogen-bond minimum where the exact dimer
binds by `5.5`, `+166` at 2.5 Å, with an exponent drifting from `1.88` to `2.04` per bohr,
and **S1 reads (c) by the harvest rule** with all six nodes positive — no set of three fits
the tenth. The cause is in the freeze's own physics: symmetrically orthogonalising the two
monomers' FULL orbital sets deforms each monomer's wavefunction at order `S²`, not `S⁴` as
the freeze stated, and that deformation penalty, not Pauli exchange, is most of the number
(on a hydrogen-molecule pair at 3 bohr the deformed state's "exchange" is 3.7 times the
undeformed state's). No wall, no arms. G-H1 also fails by its letter: the 40-bohr limit was
staked against zero and the two waters still interact by `2.0e-6` hartree there — the
freeze's third stake in two days to forget the electrostatics at the reference; the
exchange itself is `8e-12`. Banked: the penetration term re-fit on the outer four nodes
alone is eight times cleaner than FIELD-4's all-six fit (`P = 16.04`, `c = 2.42`, weighted
residual `9.7e-5`, per-node misses under `0.8 %` of the exact interaction), G-C1 passes with
the same reference on both sides (`1.4e-16`), plant (i) fires, and the bent-bond prediction
was filed before its solve with no wall in it and READ (a): the 30°-bent bond at 2.9 Å binds by `−6.18` mHa on the exact solver against a filed `−7.05` — a 14 % miss inside the 25 % tolerance, carried by the point charges and the contact term alone, with no exchange term at all. FIELD-6 is frozen (f242e51)
on the UNDEFORMED Heitler–London state — the monomers' own orbitals expanded in the
orthonormalised basis through the minors of the fragment overlap's square root — built and
unit-tested before that freeze.

| gate | verdict | the number |
|---|---|---|
| G-H0 — the orthogonalised basis is the dimer's basis | **PASS** | the dimer's full CI over `C'` reproduces FIELD-3's `e_super = −150.0520630749` to `1.64e-11` (stake 1e-8), `Converged`, `1,002,001` determinants, 14,910 core-seconds |
| G-H1 — the product state is what it says | **FAIL by letter on one leg, read** | norm `1 − 4.4e-15` (stake 1e-12) PASS; count `194,481 = 441 × 441` PASS; the 40-bohr limit `|E_HL − E_A0 − E_B0| = 2.03e-6` against `1e-8` FAIL — the whole miss is `E_es(40) = +2.030e-6` (agreeing to `8e-12`), the electrostatics of two waters at 40 bohr; `E_exch(40) = 8.4e-12`, `S_max(40) = 1.3e-52` |
| H1 — exchange is a wall | **PASS** | `E_exch` = `+166.5`, `+82.6`, `+40.1`, `+18.9`, `+5.77`, `+1.62` mHa at 2.5–3.7 Å: positive, non-increasing |
| S1 — what the wall is | **BRANCH (c) by the harvest rule** | positive prefix 6 of 6; every set of ≥ 3 misses the tenth of `|ΔE_exact|`: k = 3 by 0.137, 4 by 0.307, 5 by 1.14, 6 by 2.93; `b` drifts `1.878 → 2.044` per bohr as nodes are added |
| the penetration fit (outer four nodes, transferred) | — | `P = 16.040` Ha, `c = 2.42` /bohr, weighted residual `9.70e-5` (FIELD-4's all-six: `7.97e-4`); misses `−0.2 %`, `+0.6 %`, `−0.1 %`, `−0.8 %` of `ΔE_exact` on the fit nodes |
| dispersion | **not transferred** | with no wall the remainder is `−E_exch` and its log-log slopes are `−10.4`, `−12.1`, `−14.2` (band `[−8, −4]`); `C₆ = 0` recorded |
| G-C1 — the harvest is the engine's arithmetic | **PASS** | worst `1.37e-16` (stake 1e-10), the engine's own `E_q(R) − E_q(40)` on both sides — FIELD-4's letter-failure closed |
| plant (i) — the sign of the penetration term | **FIRES** | `6.173650e-3` observed against `2·|p_HO| = 6.173650e-3`, to `9.7e-17`; carrier `3.09e-3` |
| plant (ii) — the orthogonalisation skipped | **FIRES** | the full CI over the block-diagonal orbitals used as if orthonormal misses the record by `6.64e-2` hartree (stake ≥ 1e-2), `Converged`, 13,920 core-seconds; carrier: largest cross-fragment orbital overlap `7.85e-2` (≥ 1e-3) |
| S2 — a bent hydrogen bond, predicted forward | **BRANCH (a), read** | filed before the solve: `E_pred = −7.050e-3` on the 30°-tilted dimer at 2.9 Å (`E_q(R) − E_q(40) = −3.962e-3`, `p_HO = −3.087e-3`, wall `0` — not harvested, dispersion `0`); the solve: `ΔE_exact = −6.1758e-3`, `Converged`, 165 iterations, residual `7.9e-11`, 17,795 core-seconds; miss `8.74e-4` (14.2 %) against `1.54e-3` ⇒ (a). What transferred is charges plus contact attraction; the deformed exchange there reads `+41.3` mHa, seven times the interaction, and is cancelled in net by nothing the prediction carried — leg (b) is nowhere near (`4.1e-2` against `1.5e-3`). The tilt BINDS more than the line (`−6.18` against `−5.48`) |
| S3 — retention | **not run** | the letter of (c) |

## 1. The readings

| R_OO (Å) | ΔE_exact | E_es (isolated densities) | E_HL − E_A0 − E_B0 | E_exch (orthogonalised) | σ (s) | core-s |
|---|---|---|---|---|---|---|
| 2.5 | `+2.507e-3` | `−23.60e-3` | `+142.9e-3` | `+166.5e-3` | 2.9 | 55 |
| 2.7 | `−4.320e-3` | `−12.26e-3` | `+70.3e-3` | `+82.6e-3` | 2.9 | 55 |
| 2.9 | `−5.480e-3` | `−6.927e-3` | `+33.1e-3` | `+40.1e-3` | 2.8 | 58 |
| 3.1 | `−4.757e-3` | `−4.337e-3` | `+14.6e-3` | `+18.9e-3` | 2.9 | 58 |
| 3.4 | `−3.217e-3` | `−2.573e-3` | `+3.20e-3` | `+5.77e-3` | 2.9 | 58 |
| 3.7 | `−2.141e-3` | `−1.782e-3` | `−0.16e-3` | `+1.62e-3` | 2.7 | 59 |

Read against the budget: at 2.9 Å the exact dimer is `−5.5` mHa and this referee's
interaction is `+33`; induction, dispersion and correlation would have to supply `−38.6`
mHa, which is not what any of them is worth. The deformed referee is not a decomposition.
`E_es` itself — the classical interaction of the isolated densities, penetration included
— is a clean reading and is `1.7` times the point-charge field at 2.9 Å (`−6.93` against
`−4.13` mHa): the size of the penetration FIELD-4 could only see through a collapsing
embedding.

## 2. Why the orthogonalised state is not the Heitler–London state

`C' = C·M^{−1/2}` mixes each monomer's orbitals with the other's; the monomer's CI vector,
applied on the mixed orbitals, is no longer the monomer's eigenstate, and its energy rises
by an amount second order in the cross-fragment overlap (here up to `0.15`) times the
monomer's own energy scale. That is a deformation penalty, carried in `E_exch` on top of
Pauli exchange, and it is why the "wall" is four times too high and not one exponential.
The freeze wrote "order `S⁴`"; the correct statement is order `S²`, and the unit test that
would have caught it (the ordering exact ≤ undeformed ≤ orthogonalised, with the gap
measured) did not exist until FIELD-6's instrument did. The undeformed state keeps the
monomers' own orbitals — `C = C'·M^{1/2}`, a determinant on original orbitals `P` expanding
as `Σ_Q det(T[Q,P]) |Q⟩` — contracted with both CI vectors and evaluated in the same
orthonormal space with its own overlap divided out. On the hydrogen-molecule pair at 3
bohr: exact `−2.18190`, undeformed `−2.15791` (exchange `+0.129`, overlap `0.885`),
orthogonalised `−1.80660` (exchange `+0.481`). FIELD-6 harvests from the undeformed one.

## 3. The free reading: exchange is not an O–O quantity

FIELD-4's flipped dimer at 3.4 Å (the acceptor's hydrogens toward the donor, `R_OO = 6.425`
bohr) reads a deformed exchange of `+33.4` mHa where the LINEAR node at the same `R_OO`
reads `+5.77` — `5.8` times, with the largest cross-fragment overlap `3.9` times (`0.116`
against `0.030`). The deformed numbers are inflated, but the ratio is a measurement: at one
oxygen–oxygen separation the exchange moves by a factor of six with orientation, because it
lives on the overlapping contacts (H···O, H···H), not on the oxygens. An O–O-placed wall
cannot carry it across orientation. FIELD-6's undeformed readings on the same geometries
quantify the same ratio without the deformation; FIELD-7 places the wall on atom pairs.

## 4. Bookkeeping, declared

- The delegate ran the two full solves (G-H0, plant (ii)) and the tilted exact solve
  sequentially on 24 threads, because concurrent solves would contend for the price
  window; the readings themselves cost 55–59 core-seconds each against a 7 core-second
  refusal floor.
- A free finding, not a stake: the same 2.9 Å full CI cost `14,910` core-seconds in the
  orthogonalised monomer-orbital basis against FIELD-3's `52,739` in the supermolecule's own
  SCF basis, for energies agreeing to `1.6e-11` — the block-orthogonalised monomer orbitals
  condition the Davidson 3.5 times better. Every later exact dimer can use them.
- Plant (ii)'s planted energy sits BELOW the true one (`−150.1185` against `−150.0521`): a
  non-orthonormal basis used as if orthonormal breaks the variational bound rather than
  perturbing the answer.
- No number enters from outside the engine and its own solver. The freeze's `S⁴` claim is
  corrected here, not in the freeze.
