# EWALD-1 — results

*Freeze `EWALD_PREREG.md` (80f5876, alone). Module `holon-render/src/ewald.rs` (built by the
lead's delegate to the freeze), gates `tests/ewald.rs`; the integration (`Sim::accumulate_field`
and `field_energy_of` dispatch on `boundary.wraps()`, FIELD-1's `PeriodicNeedsEwald` retired
with its type made uninhabited, the boundary door's field-on refusal retired) and G6/G7 by
the lead. Prior art as the freeze states: Ewald 1921; de Leeuw, Perram and Smith 1980;
Essmann et al. 1995. Not compared to any other code.*

## The verdict, first

**The lattice sum is right and the freeze over-staked two of its own numbers.** The
Madelung limit is reproduced to `3.8e-9`, the forces are the derivative to `2.2e-9`, the
virial is the volume derivative to `8.8e-10`, the large-cell limit approaches the open box
as `L⁻³` with the measured exponent `−3.06`, both plants fire, and the two splits that meet
the freeze's accuracy agree to `1e-11`. Two gates FAIL BY LETTER: E1's 0.7× leg is a split the
freeze's own accuracy formula puts outside its `ε`, and E4's `1e-6` at 80 bohr is a number
the freeze's own `L⁻³` prediction puts at 137 bohr. Both are recorded as failures with their
measured causes; the freeze is unchanged; the tests assert the measured facts.

| gate | verdict | the number |
|---|---|---|
| erfc | — | own implementation (series below 2, Abramowitz–Stegun 7.1.14 continued fraction above): `erf` to `2.2e-16`, `erfc` to `2.5e-14` against tabulated values; derivative, `erf + erfc = 1`, oddness and branch continuity checked without outside values |
| parameters | — | `params_for([20,20,20], 1e-8)`: `α = 0.42919`, `r_c = 10.0`, `k_max = [12,12,12]` |
| E1 — α-invariance | **FAIL by letter, read** | 1.0× vs 1.4×: `|ΔE| 4.3e-12`, `|ΔF| 3.9e-11` (stake 1e-7); 0.7× leg: `1.686e-7` / `4.33e-7`. Cause measured: at 0.7× `α·r_c = 3.0`, real-space error `≈ 1e-4` relative; the neglected erfc tail summed over the dropped image pairs is `−1.686e-7`, the deviation to five digits. Work: 48 real pairs; 6,858 / 15,624 / 42,874 wave-vectors |
| E2 — the Madelung limit | **PASS** | energy per ion pair `−1.747564590826` against `−1.747564594633` (miss `3.8e-9`, stake 1e-6); 12 real pairs (the nearest neighbour sits exactly at `r_c`, inclusive as in `cells.rs`), 15,624 wave-vectors |
| E3 — the force is the derivative | **PASS** | worst relative `2.23e-9` (stake 1e-7), force sum `2.3e-16` against max `1.69e-3` (ratio `1.4e-13`, stake 1e-12) |
| E4 — the large-cell limit | **FAIL by letter on one leg, read** | `|ΔE|` 3.870e-4 (20) → 4.114e-5 (40) → 4.929e-6 (80): monotone, exponent `−3.061` (stake ≤ −2.5) — PASS; `|ΔE(80)| = 4.9e-6` against the staked `1e-6` — FAIL. The same law continued: 6.1e-7 at 160, 7.6e-8 at 320; the stake is met near `L = 137` |
| E5 — the virial is the volume derivative | **PASS** | `W = 2.42866656e-3` analytic against `3V·dE/dV = 2.42866656e-3`, relative `8.8e-10` (stake 1e-6); the homogeneity identity `W = −E_coulomb` holds to `5.8e-8` |
| plant (i) — the self term dropped | **FIRES** | carrier `α/√π·Σq² = 19.37` (≥ 1e-2); the planted reading exceeds the unplanted by the carrier to `3.6e-15`; against `−M/a` the per-pair miss is `4.84` |
| plant (ii) — the excluded pairs not corrected | **FIRES** | at `L = 40` the deviation from the open box goes from `4.11e-5` to `7.51e-2` (stake ≥ 1e-3); carrier `Σ_intra qq·erf(αr)/r = 0.0751` (≥ 1e-3); under the plant `e_excluded` reads exactly `0.0` |
| G6 — the wrapped box is served | **FAIL by letter on the staked scene, PASS on the smallest legal cell, read** | the freeze's scene (FIELD-1's four waters in 17 × 17 × 10) is refused by the engine's own image rule (`BreaksPeriodicImages { reach: 20, half_edge: 5 }`) and drifts `1.64e-2` hartree in 2,000 steps with the FIELD OFF (`6.33e-2` with it on); on a 42-bohr cube (the smallest legal cell) the field's drift peak `9.1e-6` against the bare law's `8.3e-6`, `e_field = 1.886e-3`, transition `−7.93e-4` (a tenth is `7.9e-5`), momentum residual `9.6e-14` under `3.4e-10`, columns closed, 54 real pairs and 15,624 wave-vectors |
| G7 — the open box is untouched | **PASS** | the channel receipt (`tests/data/channel_ledger.receipt`, whose `water4` block is a field-on walled scene) reproduces line for line with the dispatch in place, 7/7; the direct-sum path gained no arithmetic |

## What the two letter-failures are

**E1.** The freeze asked for invariance across `α` at 0.7×, 1.0× and 1.4× of the derived
value with `r_c` re-derived — but `r_c` is pinned at `L/2` by the freeze's own formula, so
the 0.7× split lowers `α·r_c` from 4.29 to 3.00 and its real-space truncation error is
`erfc(3.0) ≈ 2e-5` of the pair term, three orders above `ε = 1e-8`. The gate therefore asked
a split that does not meet the freeze's accuracy to agree with two that do. Measured: the
deviation IS the neglected tail (five digits), the leg passes from 0.85× upward
(`1.1e-9 / 6.4e-9`), and the two converged splits agree at `1e-11`. The module is invariant
under the split wherever the split is inside its own accuracy. The freeze's range was wrong
by its own arithmetic and is not amended after the fact; the reading is entered here.

**E4.** The freeze predicts the leading image term of a neutral dipolar pair falls as `L⁻³`
and then stakes `1e-6` at 80 bohr. Measured at 40 bohr the deviation is `4.1e-5`; `L⁻³` from
there gives `5.1e-6` at 80 (measured `4.9e-6`) and reaches `1e-6` near 137 bohr. The exponent
gate confirms the law; the absolute leg was staked at the wrong cell. Same handling.

Two wording notes, the delegate's: E1's scene text says "8 charges" where four waters carry
twelve (the test builds twelve in four units — the freeze's scene, miscounted in its prose);
plant (i)'s `1e-9` is met against the unplanted reading of the same scene rather than
against `−M/a`, since E2's own residual (`3.8e-9`) exceeds `1e-9` and no implementation could
meet the literal form.

## The integration

`Sim::accumulate_field` and `Sim::field_energy_of` dispatch on `boundary.wraps()`: under a
wrapping boundary the lattice sum's energy, forces (into `a_pair`) and virial (into `w_virial`,
the engine's convention) with the parameters derived per pass from the cell at the default
accuracy; the open and walled boxes keep the direct sum, untouched (G7). FIELD-1's plant
(iii), the sign, negates the whole term; its plant (i), the reaction dropped, is a property
of a pairwise sum and has no reading under the lattice sum. `FieldRefusal::PeriodicNeedsEwald`
is retired and the type made uninhabited; the boundary door's field-on refusal is retired
with it; `FieldWork` gains the wave-vector count and the page a door for it.

**G6, read.** The freeze staked G6 on "FIELD-1's four-water scene under `Boundary::Periodic`"
— a 17 × 17 × 10 cell against a 20-bohr table reach, which the engine's `set_boundary`
refuses (`BreaksPeriodicImages`) and which FIELD-1's own test only reached by assigning the
boundary field directly. With the field OFF that scene drifts `1.6e-2` hartree in 2,000
steps: the tables' minimum image switches partners inside their support. The letter fails on
that scene for the tables' reason, not the field's; on the smallest legal cell the field's
drift is the bare law's to within a tenth. Recorded as such; the test asserts the refusal
and the legal-cell reading (`examples/ewald_probe.rs` is the diagnostic).

## What this lands

The field is served in a wrapping cell, with the pair-within-a-unit exclusion the open-box
field already made, the neutralising background named, the conducting boundary convention
stated, and its virial in the engine's convention. FIELD-1's refusal by name is retired by
the exit it named. The periodic liquid (GANTT: after FIELD-3) has its electrostatics.
