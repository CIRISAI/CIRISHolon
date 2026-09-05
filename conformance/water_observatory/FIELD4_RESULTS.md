# FIELD-4 — results

*Freeze `FIELD4_PREREG.md` (729c45e, alone). Instrument: FIELD-3's engine with `SeamModel`
extended by the penetration term on cross-unit H–O pairs and the dispersion term on cross-unit
O–O pairs (each an exact `0.0` off; checkpoint v8; `Row::Seam` now carries exchange whole,
field and induction folded, pair dispersion whole); the harvest
`examples/field4_harvest.rs` (`density`, `predict`; written by a delegate to the freeze),
G-C1 and plant (i) by `examples/field4_check.rs`; gates `tests/seam.rs` (G-D0–G-D2 with
FIELD-4's plant (ii)); the arms runner reads `wall4.json`. The referee is FIELD-3's six exact
linear dimers, reused as frozen. JSON under `field4/`.*

## The verdict, first

**The density field is not a field at contact, and the harvest reads BRANCH (c) by the
freeze's own rule.** EMBED-2's Coulomb-only frozen-density embedding — each monomer relaxed
in the other's Coulomb potential with no exchange between them — binds the dimer by
`−24.2` mHa at 2.5 Å where the exact dimer is `+2.5` mHa repulsive, and by `−12.6` at 2.7 Å
against `−4.3`: without exchange the fragments polarise into each other unopposed, and the
"penetration" it measures at contact is that collapse. The residual over it is therefore a
wall of `+26.7`, `+8.3`, `+1.7` mHa at 2.5, 2.7, 2.9 Å whose exponent is not constant (3.1 then
4.1 per bohr), which no single exponential fits within the staked tenth — a positive prefix of
three, the fit missing by 25 % — and (c) fires by the harvest rule, not the prefix count. No
wall is harvested, the arms do not run. What the campaign did measure, and banks: **at the
hydrogen-bond minimum the point charges' missing attraction is mostly penetration and
induction** — of the `−1.49` mHa FIELD-3 found missing at 3.1 Å, the density field carries
`−1.26` and inter-fragment correlation the remaining `−0.22`; at 3.4 and 3.7 Å correlation
is the larger part (`−0.53`, `−0.29` of `−0.84`, `−0.36`). C1 passes (the density field
binds at least as much as the charges at every node, monotonically), G-C1 fails by letter by
exactly the engine's own field at the 40-bohr reference and agrees to `5e-16` once it is
subtracted, plant (i) fires, and the flipped-dimer prediction was filed before its solve
(S2 read (c): the flipped dimer is `+6.01` mHa repulsive against a filed `+2.31`; the `+3.7` mHa both placements miss is the exchange no wall carried, now measured on a second orientation). FIELD-5 is named: exchange IN the embedding — the antisymmetrised product
of the two monomer wavefunctions on the seam programme's solver, whose energy over the
monomers and the classical interaction IS the exchange wall, harvested from a referee that
cannot collapse.

| gate | verdict | the number |
|---|---|---|
| G-D0 — the identity | **PASS** | FIELD-3's seam gates unchanged with the new coefficients at `0.0`: bytes identical on the enable-then-disable scene, the receipt line for line (7/7, 7/7, 7/7) |
| G-D1 — the terms are the derivative | **PASS** | worst relative `3.75e-9` over EVERY atom and component (stake 1e-8), penetration on 4 H–O pairs, wall on 1 O–O pair (declared test coefficients; no harvest to load, named in the output) |
| G-D2 — momentum and the books | **PASS** | net internal force `1.3e-17` against `2.9e-2`; residual `9.4e-14` under `7.7e-11`; dimer columns close (`seam −2.08e-2` = the switch), drift peak `1.26e-7` against `2.08e-3`; ring likewise |
| C1 — the density field binds at least as much as the charges | **PASS** | `p(R) = ΔE_ρ − E_q`: `−17.1`, `−7.31`, `−3.08`, `−1.26`, `−0.31`, `−0.063` mHa at 2.5–3.7 Å — negative everywhere, monotone outward from 2.5 Å, every fixed point converged (6–7 sweeps) |
| S1 — what the residual over the density field is | **BRANCH (c) by the harvest rule** | `r_ρ = ΔE_exact − ΔE_ρ`: `+26.7`, `+8.33`, `+1.73` mHa (2.5, 2.7, 2.9 Å), then `−0.22`, `−0.53`, `−0.29`; positive prefix 3; the log-linear wall over the three misses 24.9 % of `ΔE_exact` (stake 10 %); no wall, `C₆` not read |
| the penetration fit (transferred, arms not run) | — | `P = 9.389` Ha, `c = 2.270` /bohr on the H–O placement (weighted residual `7.97e-4`); the O–O alternative `P = 777`, `c = 2.27` (`8.00e-4`) — the two placements are indistinguishable on one line, which is what S2 exists for |
| G-C1 — the harvest is the engine's arithmetic | **FAIL by letter, read** | worst `2.28e-6` (stake 1e-10) — exactly the engine's field at the 40-bohr reference (`+2.28e-6` at 2.5 Å, `+1.42e-6` at 3.7 Å): the freeze compares the engine's DIFFERENCE against the RAW field of record; subtracting the engine's own far field the worst miss is `5.1e-16` |
| plant (i) — the sign of the penetration term | **FIRES** | at 2.9 Å the miss is `6.151349e-3` = `2·|p_HO|` to `1e-10`; carrier `|p_HO(2.9 Å)| = 3.08e-3` (≥ 1e-4) |
| plant (ii) — the reaction dropped on the new terms | **FIRES** | the internal force sum exceeds `1e-6` of its scale and the residual its bound; carrier `|F_pen + F_disp| ≥ 1e-6` at the start |
| S2 — the placement, decided forward | **BRANCH (c)** | filed before the solve: `E_pred(H–O) = +2.315e-3`, `E_pred(O–O) = +2.418e-3` hartree on the flipped dimer at 3.4 Å (`E_q = +2.778e-3`, `p_HO = −4.63e-4`, `p_OO = −3.60e-4`, no wall, no dispersion); tolerance `max(0.25·|ΔE_exact|, 5e-4)`. The two placements are near-degenerate on the linear geometry (same `c`, residuals differing in the fourth digit) and their flipped predictions differ by `1.03e-4` — a fifth of the tolerance floor — so S2 as staked CANNOT separate them and its branch (b) is unreachable (`placement_separable: false`, filed with the prediction). The solve: `1,002,001` determinants, 177 iterations, residual `6.5e-11`, `Converged`, 13,176 core-seconds; `ΔE_exact = +6.009e-3` — H–O miss `3.69e-3` (61.5 %), O–O miss `3.59e-3` (59.8 %) against `1.50e-3` ⇒ (c) |
| S3 — retention | **not run** | the letter of (c) |

## 1. The density field on the six geometries

| R_OO (Å) | ΔE_exact | E_q (charges) | ΔE_ρ (density field) | p = ΔE_ρ − E_q | r_ρ = ΔE_exact − ΔE_ρ | FIELD-3's r_q = ΔE_exact − E_q |
|---|---|---|---|---|---|---|
| 2.5 | `+2.507e-3` | `−7.069e-3` | `−24.16e-3` | `−17.09e-3` | `+26.67e-3` | `+9.58e-3` |
| 2.7 | `−4.320e-3` | `−5.337e-3` | `−12.65e-3` | `−7.31e-3` | `+8.33e-3` | `+1.02e-3` |
| 2.9 | `−5.480e-3` | `−4.134e-3` | `−7.210e-3` | `−3.08e-3` | `+1.73e-3` | `−1.35e-3` |
| 3.1 | `−4.757e-3` | `−3.270e-3` | `−4.535e-3` | `−1.26e-3` | `−0.22e-3` | `−1.49e-3` |
| 3.4 | `−3.217e-3` | `−2.377e-3` | `−2.687e-3` | `−0.31e-3` | `−0.53e-3` | `−0.84e-3` |
| 3.7 | `−2.141e-3` | `−1.784e-3` | `−1.848e-3` | `−0.06e-3` | `−0.29e-3` | `−0.36e-3` |

Two readings, both measured. **Inside 2.9 Å the Coulomb-only embedding is not a field:**
it over-binds the exact dimer by 27 mHa at 2.5 Å because nothing in it opposes the fragments'
mutual polarisation — the instrument EMBED-3 validated at 4–8 Å (within 5 % of the charges,
98–99.6 % of the far interaction) has no Pauli term and fails where the densities overlap.
**At and beyond the hydrogen-bond minimum it is the missing channel:** FIELD-3's `−1.49` mHa
gap at 3.1 Å is `−1.26` penetration-and-induction plus `−0.22` correlation; at 3.4 and 3.7 Å
the gap is mostly correlation (`−0.53` of `−0.84`, `−0.29` of `−0.36`), which is channel 3 at
this basis and is not transferred here because the wall it sits behind was not harvested.

## 2. Why (c), and what it is not

The freeze's (c) reads "the exchange wall is not separable from correlation at this basis;
FIELD-5 stakes a larger basis". The measured cause is different and is entered as such: the
wall over the density field is not a single exponential because the density field's own
error grows exponentially inward (its over-relaxation), so `r_ρ` carries the embedding's
defect on top of exchange. A larger basis would not remove that; exchange in the embedding
would. The reading of the freeze's letter stands ((c) fired by its rule) and its
interpretation is corrected here.

## 3. The seam law as it stands

Point charges (FIELD-1, closure-assigned by FIELD-3), no wall, no dispersion; the
penetration term `(P, c)` is transferred but, without a wall to hold the contact, is not run
in dynamics — a `−P·exp(−c·r)` on H–O with `P = 9.4` hartree and no repulsion would collapse
the dimer, and the freeze's (c) is the reason the arms do not run. The engine carries all
three terms as exact zeros until FIELD-5 harvests them together over a referee with exchange.

## 4. S2 — the placement, decided forward

The flipped dimer (the acceptor's hydrogens toward the donor, `R_OO = 3.4` Å) is `+6.009` mHa
repulsive on the exact solver. The filed predictions — point charges `+2.778` mHa plus the
penetration term, `−0.463` (H–O placement) or `−0.360` (O–O) — land at `+2.315` and `+2.418`
and both miss by `+3.6` to `+3.7` mHa, sixty percent of the exact value, against a tolerance
of `1.50` mHa: branch (c), the seam law orientation-dependent beyond either placement. Read
plainly, the miss is the term the prediction could not carry: no wall was harvested, and on
this geometry the closest cross-unit contacts are the hydrogens (H···H about 3.5 bohr, H···O
about 4.6 bohr), so `+3.7` mHa is the flipped dimer's exchange plus whatever the density
field over-relaxes there. It is a second measured point for FIELD-5's referee, on an
orientation the wall was never fit to. The placement question remains open at this
tolerance by construction (the two predictions differ by `0.10` mHa; a placement gate needs
a geometry where they differ by more than the floor, which is FIELD-5's to stake).

## 5. FIELD-5, named

Exchange in the embedding: on the seam programme's determinant solver, the antisymmetrised
product of the two monomers' exact wavefunctions is one vector in the dimer's determinant
space, and its energy expectation is one Hamiltonian application; `E_HL − E_A0 − E_B0 −
E_es` is the first-order exchange repulsion (Heitler–London), a referee that cannot
collapse. The harvest then: the wall from `E_HL`'s exchange (a clean exponential expected),
the penetration-and-induction term from the density field OUTSIDE 2.9 Å only (where it is a
field), dispersion from what the exact dimer leaves, and S2/S3 as frozen here. Prior art to
credit at that freeze: Heitler–London 1927; the SAPT decomposition (Jeziorski, Moszynski,
Szalewicz 1994) for the names of the pieces.

## 6. Bookkeeping, declared

- The freeze compared the engine's interaction (a difference against 40 bohr) with the raw
  field of record; the `2.3e-6` is that reference and is recorded as G-C1's letter-failure.
- The G-D1/G-D2 gates ran with declared test coefficients (named in the output) because the
  harvest transferred no wall; they are conservation properties for any coefficients.
- The exact flipped solve ran detached on 24 threads (`field4/predict.log`); the density
  fixed points took seconds per node.
- No number enters from outside the engine and its own exact solver.
