# C1_GATE_PREREG — the ring-polymer carrier gets a real gate

**Frozen 2026-09-01.** Instrument commit `7a70256` (`engine/crates/holon-chem/src/rpmd.rs`,
`engine/crates/holon-chem/examples/c1_gauge.rs`). Nothing below was written after a target
number was seen; the gauging that sized the bands ran on PLANTS ONLY and its output is
reproduced in §7.

---

## 0. What is being tested, and what was owed

`engine/crates/holon-chem/src/tower.rs` declares the C1 ring-polymer carrier — its state,
its operator, its certified transport from C0 — and solves no quantum-nuclear problem
anywhere. The only test that mentioned zero-point energy,
`tests/carrier_tower.rs::test_h2_harmonic_zpe_sanity`, says so in its own comment: it does
harmonic arithmetic on a TRANSCRIBED curvature constant, runs no ring-polymer dynamics, and
reads no banked table. `WORKBENCH_FSD.md` §"OWED" names the real gate: *RPMD ZPE on the
banked H–H curve vs an exact anharmonic reference, plus the D₂ isotope shift.* This freeze
is that gate, staked before it runs.

**The claim under test.** C1 is a certified refinement of C0: transporting to the
ring-polymer carrier buys quantum nuclear structure that C0 cannot represent, at a stated
price, with a stated error budget, and the object it computes is the same object an exact
solution of the same Hamiltonian computes.

**The single kill.** If ring-polymer dynamics on the engine's own H–H curve does not
reproduce that curve's exact anharmonic vibrational ground state — its own, not a fitted
one — then C1 is scaffolding wearing a carrier's name, and the tower has one fewer rung
than it advertises.

---

## 1. Scope, stated before the result

- **One dimension, `J = 0`.** The observable is the vibrational ground state of a diatomic
  in its relative coordinate with reduced mass `mu = m_a m_b / (m_a + m_b)`. Rotation is
  excluded BY CONSTRUCTION, not by approximation: a thermal three-dimensional ring polymer
  at 300 K carries a quantum rotational energy (`beta B ~ 0.32` for H₂ on this curve) that
  no `J = 0` reference can grade, so the instrument and its referee are held to one
  Hamiltonian rather than compared across two.
- **The curve is the engine's.** `holon_chem::h2::h2_point` — STO-3G full CI, closed-form
  Gaussian integrals, analytic first and second derivatives. Nothing about the potential is
  transcribed, quoted, or fitted. `R_e`, `D_e`, `V(R_e)` and `E''(R_e)` are read from
  `h2::equilibrium()` and `h2_point` at run time.
- **The masses are declared inputs.** `Species::HYDROGEN.mass_u = 1.00782503207` from
  `elements.rs`; `rpmd::MASS_U_DEUTERIUM = 2.01410177812`, declared in the module where the
  isotope prediction is made and carrying `elements.rs`'s own convention (nucleus plus its
  electrons, Born–Oppenheimer). Reduced masses are COMPUTED from these; no reduced mass is
  tabulated anywhere.
- **STO-3G is not hydrogen.** Every number is EXACT-IN-MODEL. This freeze compares an
  instrument to a referee on ONE model Hamiltonian; it makes no claim about the measured
  vibrational constants of H₂ or D₂, and the D₂ prediction is free WITHIN THE MODEL — the
  Born–Oppenheimer surface does not know about isotopes, so nothing but the reduced mass is
  allowed to move.

**Two surfaces, and why.** The exact solver costs ~65 µs a call (measured, §7); a converged
sampling run needs ~10⁹ of them. So the referee runs on the exact solver AND on the engine's
banked cubic-Hermite table (`table::generate_table`, the same interpolant `holon-render`
integrates), the sampler runs on the banked table, and the DIFFERENCE BETWEEN THE TWO
REFEREES is the interpolation systematic — a number in the gate's own currency, gated at
**G2** below, not a hope.

---

## 2. The instrument and the referee

**Referee** (`rpmd::dvr_reference`) — Colbert–Miller sinc DVR (JCP 96, 1982 (1992)) with
the `(-inf, inf)` kernel restricted to a box, diagonalised by Lanczos with full
reorthogonalisation and an ADAPTIVE Krylov dimension, plus an independent Numerov
node-counting solve on the same Hamiltonian. It returns `Err` rather than a number unless
all four of these hold: the Lanczos Ritz residual, a grid halving, a box widening, and the
Numerov cross-check. It carries `solves` and `potential_calls` with the answer.

**Instrument** (`rpmd::run_pimd`) — the `P`-bead ring-polymer Hamiltonian
`H_P = sum_k [p_k^2/2m + (1/2) m omega_P^2 (q_k - q_{k+1})^2 + V(q_k)]`,
`omega_P = P/(beta hbar)`; state carried in normal modes; EXACT free-ring-polymer
propagation; PILE thermostat (Ceriotti–Parrinello–Markland–Manolopoulos, JCP 133, 124104
(2010)) with the PILE-L optimum `gamma_k = 2 omega_k` on internal modes and a DECLARED
centroid friction `gamma_0 = omega_harm`; centroid-virial and primitive energy estimators
both reported; error bars from the blocking plateau within chains and the between-chain
spread across them, the LARGER of the two quoted.

---

## 3. Declared run parameters — frozen here, not chosen later

| | |
|---|---|
| temperature | **300 K** (`beta = 1052.5834 Ha^-1`), the tower's own `C1_RingPolymer` default scale |
| time step | **`dt = 4.0` a.u.** (`omega_harm dt = 0.0912`), with a `dt/2` repeat as a convergence check |
| centroid friction | **`gamma_0 = omega_harm`** (near-critical damping of the centroid mode) |
| bead ladder | **`P` in {1, 2, 4, 8, 16, 32, 64, 128, 256, 512}** |
| headline bead count | **`P = 256`** |
| chains | **8**, seeds `0xC1_0001 + k * 0x9e3779b97f4a7c15` |
| DVR box | **[0.50, 9.00] bohr**, `n = 601`, floor = the banked table's inner knot |
| levels | **6** |
| banked knots | **4096** |

**The thermal-population correction is computed, not waved away.** At 300 K,
`beta omega = 23.99` for H₂ and `16.97` for D₂, so the thermal energy of the mode differs
from its zero-point energy by `< 1e-9 Ha`. The referee computes that difference from its
OWN spectrum (`DvrReference::thermal_energy`) and the results document prints it; the
comparison is made against the thermal energy at the sampling temperature, and the
zero-point energy is reported as its `T -> 0` reading.

---

## 4. The gates

Every band is a fraction of `ZPE_DVR(H2)` unless stated. Every gate names the fate of BOTH
answers.

- **G0 — the referee refuses when it should.** The DVR reference must return `Ok` for H₂
  and D₂ on both surfaces with all four residuals `<= 1e-9 Ha`, and must reproduce the two
  closed-form plants of §5 to `<= 1e-12 Ha` on all 6 levels. FAILS: no reference exists and
  every gate below is **VOID**, not failed — an instrument cannot be graded by a ruler that
  will not certify itself. witness: `closed_iff_fiber_invariant`
- **G1 — RPMD reproduces the exact anharmonic zero-point energy.** At `P = 256`,
  `|ZPE_RPMD(H2) - ZPE_DVR(H2)| <= 0.40%` of `ZPE_DVR(H2)`, with the quoted statistical
  error `<= 0.12%`. PASSES: C1 computes the object its Hamiltonian defines. FAILS: the
  headline kill fires and C1 is scaffolding. Statistical error above 0.12% with the central
  value within the band is **VOID** (insufficient sampling), never a pass.
  witness: none (a sampling result is measured, not proved; the Lean side of this rung is G5's square)
- **G2 — the interpolation systematic is not the answer.**
  `|ZPE_DVR(banked 4096 knots) - ZPE_DVR(exact solver)| <= 0.05%` of `ZPE_DVR(H2)`, i.e. at
  least 8x inside G1's band. FAILS: G1 is **VOID** — the sampler would be grading the
  interpolant, and the knot count is raised until this holds or the gate is abandoned.
  witness: none (a numerical systematic, measured against the same model by two grids)
- **G3 — the ladder converges as the theory says, and the theory is a closed form.**
  The prediction is NOT the asymptotic exponent 2: it is the exact `P`-bead energy of a
  harmonic oscillator at this curve's own `omega_harm` (§5, `harmonic_ring_energy`),
  evaluated with no free parameters. Two clauses.
  **(a) The whole shape.** For each `P` in {2, 4, 8, 16, 32, 64}, the measured
  `E_cv(P) - E_cv(256)` must match the closed form's `E_P - E_256` to within
  `max(3 sigma, 12%)` of the prediction. The prediction spans `-9.51e-3` at `P = 2` down to
  `-1.83e-4` at `P = 64`, so this is one forward prediction tested across a factor of 52 in
  the quantity itself. The 12% allowance is the anharmonicity: the correction scales as
  `omega^3`, this curve's zero-point energy sits about 1.4% below its harmonic value, so a
  ~4% departure is expected and 12% is 3x that headroom.
  **(b) The exponent, on the window the noise floor allows.** The fitted `x` in
  `|E_cv(P) - E_cv(256)| ~ A P^-x` over `P` in {16, 32, 64} must lie within **±0.25** of
  **1.7730**, which is the CLOSED FORM'S OWN exponent on that window — not 2, which is the
  `P -> infinity` asymptote the window has not reached. FAILS: the ring polymer is
  converging to something, but not by the mechanism claimed, and G1 becomes a coincidence at
  one `P`.
  witness: none (a convergence law, measured; its closed form is arithmetic, in harmonic_ring_energy)
- **G4 — the D₂ isotope shift, the free prediction.** Bead masses only change.
  `|ZPE_RPMD(D2) - ZPE_DVR(D2)| <= 0.40%` of `ZPE_DVR(D2)`, AND the measured ratio
  `ZPE(D2)/ZPE(H2)` must sit **below** the harmonic value `sqrt(mu_H2/mu_D2) = 0.70724` by
  more than the combined band — anharmonicity is lighter-isotope-heavy, so the ratio is
  pushed the other way. FAILS: the isotope axis is where a curve-fitted instrument dies, and
  this is the one number in the freeze that nothing here was built to reproduce.
  witness: none (a forward prediction, measured)
- **G5 — the bead-forgetting square, with its budget printed.** The centroid chart
  `v : RingPolymerState -> ClassicalState` against one step of each dynamics.
  (i) At `P = 1` the square is **EXACT**: `defect_pos <= 1e-15` bohr and
  `defect_vel <= 1e-15` bohr/a.u. over 5000 steps, i.e. `v` is `Closed` in the sense of
  `Object.lean`. (ii) At `P = 64` on a thermally spread ring the square is NOT closed:
  `defect_pos > 1e-12` bohr, a measured `NonFactoring` witness. (iii) The mechanism is
  named and measured: `force_gap = |mean_k F(q_k) - F(q_c)|` scales as `R_g^2` with fitted
  exponent in **[1.6, 2.4]**, and `defect_pos` scales as `dt^2` with fitted exponent in
  **[1.7, 2.3]**. FAILS on (i): the diagonal retract is not a retract and the C0→C1
  certificate is void. FAILS on (ii): the gate is **VOID** for measuring nothing — a
  closed square at `P = 64` means the ring never spread, which is a broken run, not a
  theorem. FAILS on (iii): the defect is real and its cause is not the one claimed.
  witness: `nonfactoring_iff_not_closed`
- **G6 — the gate can tell the two apart (the discriminator).** `|ZPE_DVR(H2) -
  0.5 * omega_harm|` must exceed **3x** G1's band. Anharmonicity is the content of G1; if
  the exact answer sits within the band around the harmonic arithmetic that
  `test_h2_harmonic_zpe_sanity` already does, then G1 passing proves nothing that was not
  already true, and the whole freeze is **VOID** for want of discriminating power. This gate
  is stated FIRST-CLASS because the sanity test it replaces would pass a purely harmonic
  instrument.
  witness: none (a discriminating-power condition on this freeze's own bands)
- **G7 — the price closes.** The banked run's wall time divided by the measured per-call
  price of the banked surface and the per-step normal-mode cost must reproduce the observed
  potential-evaluation count to within a factor **3**, on a core whose class is declared.
  A result arriving cheaper than its own cost model is not that result. FAILS: the run is
  **VOID** and re-run under `taskset` on a declared core.
  witness: none (a cost-model check, arithmetic)

---

## 5. Plants

Both plants are closed forms, and both are evaluated by the SAME code path as the target —
same DVR, same Numerov, same sampler, same estimators. The sector each acts in is named,
because a plant that is nonzero somewhere is not a plant for the sector the gate reads.

- **P1 — harmonic.** `V = V(R_e) + (1/2) E''(R_e) (R - R_e)^2`, curvature read from
  `h2_point(R_e).e2`. Exact spectrum `omega (n + 1/2)`. Its carrier is **nonzero in the
  HARMONIC sector**: it exercises the referee's kinetic operator, box, and grid, and it
  exercises the sampler's thermostat, propagator and estimators — and it is deliberately
  **ZERO in the anharmonic sector**, which is why it is not sufficient on its own.
- **P2 — Morse.** `V = V(R_e) + D_e (1 - exp(-a (R - R_e)))^2` with `D_e` from
  `h2::equilibrium()` and `a = omega sqrt(mu / 2 D_e)` derived from the same two numbers,
  nothing fitted. Exact spectrum `w(n+1/2) - w^2 (n+1/2)^2 / 4 D_e`. Its carrier is
  **nonzero in the ANHARMONIC sector** — the sector G1, G4 and G6 read — by a planted
  `-1.590e-4 Ha`, `-1.395%` of the harmonic zero-point energy, which is the quantity the
  gate must be able to see. Pre-checked observable: that displacement is 3.5x G1's band,
  so a referee that could not see it would fail P2 before any target was touched.
- **P3 — the exact `P`-bead ring-polymer energy.** For a harmonic oscillator,
  `E_P = (1/beta) sum_k omega^2 / (omega_k^2 + omega^2)`, `omega_k = (2P/beta) sin(k pi/P)`,
  derived in the module docstring from the Gaussian path integral. `kT` at `P = 1`,
  `(omega/2) coth(beta omega/2)` as `P -> inf`. Its carrier is **nonzero in the BEAD-NUMBER
  sector** — the sector G3 reads — because it is a function of `P` at fixed physics, which
  is exactly the axis the convergence ladder is a claim about.

---

## 6. Registered misfits this freeze contacts

**misfits:** M-VACUOUS-SUCCESS, M-CHEAPER-THAN-ITS-PRICE, M-PLANT-SECTOR, M-PLANT-OBS,
M-FIXED-POINT-TRAJECTORY, M-SORTS-NOT-SEPARATES, M-EXIT-DISCRIMINATOR, M-BUDGET-LAUNDER,
M-PLACEMENT-LOTTERY, M-STALE-INSTRUMENT, M-ONE-MODEL-DELTA, M-MAX-OVER-SUCCESSES,
M-VOLUME-SCALE, M-HOMOG, M-MAINTENANCE-LENS, M-DEVICE-CLASS, M-NULL-MISSTAKE,
M-UNTESTED-GAP

- **M-VACUOUS-SUCCESS** — the referee asserts its WORK: `DvrReference` carries `solves` and
  `potential_calls`, the sampler carries `samples`, `potential_calls`, `chains` and an
  `excursions` counter, and G0/G7 read them. A convergence banner over zero work is refused
  by construction, because the counts are printed beside the verdict.
- **M-CHEAPER-THAN-ITS-PRICE** — G7 IS this misfit made into a gate: the run's wall time is
  divided by its own measured per-call price and must reproduce the evaluation count.
- **M-PLANT-SECTOR** — §5 names, for each plant, the sector its carrier is **nonzero in**,
  and P1 is explicitly recorded as ZERO in the sector G1 reads, which is why P2 exists.
- **M-PLANT-OBS** — each plant's displacement is re-derived for THIS instrument and
  pre-checked observable: P2's `-1.395%` against G1's `0.40%` band, P3's `P^-2` law against
  G3's exponent window.
- **M-FIXED-POINT-TRAJECTORY** — G5(i) would be vacuous on a carrier that is a fixed point
  of the dynamics, so the `P = 1` trajectory is launched DISPLACED from `R_e` with nonzero
  velocity and the gate additionally requires the trajectory to MOVE: total path length
  `> 0.1` bohr over the 5000 steps, asserted, or G5(i) is VOID rather than passed.
- **M-SORTS-NOT-SEPARATES** — G3's ladder is required to SEPARATE (a fitted exponent inside
  a window), not merely to rank: a monotone decreasing sequence with the wrong exponent
  fails, which a "converges" bar would not have caught.
- **M-EXIT-DISCRIMINATOR** — `RefereeRefusal` has four distinct variants and the results
  document names WHICH one fired; a referee that recorded only "unconverged" could not tell
  a small box from a coarse grid from an under-resourced Krylov space.
- **M-BUDGET-LAUNDER** — insufficient statistics is a **VOID** verdict in G1 and G4, never
  a scorable pass, and the VOID condition is stated in the gate rather than decided later.
- **M-PLACEMENT-LOTTERY / M-DEVICE-CLASS** — no gate here is a timing comparison, and the
  ONE timing number that appears (G7's price) declares its core class and is pinned with
  `taskset`. No verdict in this freeze is a function of wall time.
- **M-STALE-INSTRUMENT** — the instrument commit is on line 3 of this freeze and will be on
  the results document.
- **M-ONE-MODEL-DELTA** — G1's band is a defect against ONE reference (the DVR on this
  Hamiltonian), so a pass earns exactly "agrees with the exact solution of the same model",
  never "agrees with hydrogen". §1 states this and the results document repeats it.
- **M-MAX-OVER-SUCCESSES** — the residuals quoted by the referee are maxima over levels that
  ALL converged; a level that fails to bracket is a panic in `numerov_levels` and a refusal
  in `dvr_reference`, so no maximum is ever taken over a set containing a failure.
- **M-UNTESTED-GAP** — the closed form was plotted on the hypothesised axis BEFORE G3 was
  staked, and it moved the stake twice (§7 item 3): the fit window shrank to where the
  signal clears `3 sigma`, and the target exponent moved from the asymptotic 2 to the
  closed form's own 1.773 on that window.
- **M-VOLUME-SCALE** — the analogue of a lattice `N` here is the bead count `P`, and G3 is
  precisely the requirement that it be scaled with the coupling `beta omega` rather than
  fixed: `P = 256` is `10.7x` `beta omega`, and the ladder demonstrates the scaling rather
  than assuming it.
- **M-HOMOG** — no spatial homogeneity is claimed or used: the problem is one relative
  coordinate of a two-body system and there is no extended medium whose local structure
  could stand in for a distant one.
- **M-MAINTENANCE-LENS** — no repair or maintenance operator appears in this freeze; the
  ring polymer is sampled, never repaired, and the thermostat is a sampling device whose
  effect on the estimators is exactly zero in expectation by construction.
- **M-NULL-MISSTAKE** — the conserved quantity gated in G5 is the one the law constrains:
  `H_P` is the generator of `ring_step_3d`, so its drift IS the integrator error, and the
  classical energy is checked against the same pair potential's own zero.

---

## 7. The gauging that sized the bands — plants only, run before this freeze

Reproduced verbatim from `examples/c1_gauge.rs` at commit `7a70256`. No target quantity
appears in it.

```
R_e 1.388694018 bohr   D_e 0.204142352 Ha   V(R_e) -1.137306051 Ha
E''(R_e) 0.477097668 Ha/bohr^2   mu(H2) 918.576323 m_e   mu(D2) 1835.741470 m_e
omega_harm 0.022790089 a.u. (5001.85 cm^-1)   beta(300K) 1052.5834   beta*omega 23.988
banked range derived from WALL_CEILING/TAIL_TOLERANCE: [0.392722, 10.240000] bohr

plant 1 (harmonic): ritz 2.96e-14 grid 3.13e-14 box 5.77e-15 numerov 4.72e-13
  worst |DVR - omega(n+1/2)| over 6 levels: 4.0e-15 Ha
plant 2 (Morse):    ritz 5.43e-14 grid 2.07e-14 box 1.22e-13 numerov 9.75e-13
  worst |DVR - exact| over 6 levels: 6.6e-15 Ha
  planted anharmonicity: ZPE_morse - ZPE_harm = -1.590e-4 Ha (-1.395 %)
```

```
plant 3a (harmonic, P=64, dt ladder, 8 chains, steps scaled as 1/dt):
   dt      E_cv - exact E_P      err
  8.00        +9.00e-6         6.72e-6
  4.00        +5.06e-6         7.84e-6
  2.00        -5.65e-6         6.55e-6
  1.00        -4.25e-6         6.79e-6
```

**Two things this gauging CHANGED before the freeze, both found on plants and neither on a
target.**

1. **The referee's Krylov dimension.** The first version reported a Lanczos residual of
   `1.5e-2` on a grid whose eigenvalues were right to `2e-13`. The residual was honest and
   the dimension was a guess; the fix was to make the dimension ADAPTIVE, not to relax the
   tolerance. A fixed dimension cannot tell an under-resourced solver from a hard problem,
   and this freeze would have carried that confusion into every refusal it issued.
2. **The integrator ordering.** The sampler was first written OBABO (thermostat outermost,
   as in the original PILE paper). Plant 3 measured its bias against the closed-form
   `E_P` at `+2e-5` to `+4e-5 Ha` at `dt = 4` — `0.2-0.3%` of the zero-point energy G1
   reads, which would have eaten most of a `0.40%` band before any physics was tested. The
   scheme was changed to **BAOAB**, which costs exactly the same (one force evaluation and
   two normal-mode transforms a step either way) and whose configurational error is
   `O(dt^4)` rather than `O(dt^2)`. The `dt` ladder above is the re-gauge: at `dt = 4` the
   residual bias is below the `7e-6` noise floor, and no trend in `dt` is resolvable. That
   is why `dt = 4.0` can be staked in §3 rather than a `dt` four times smaller and sixteen
   times dearer.

3. **G3's fit window and its target exponent.** The first G3 staked a fit of `A P^-x` over
   `P` in {32, 64, 128, 256, 512} against a band around **2**. Plotting the closed form on
   its own axis before staking — which is the whole of M-UNTESTED-GAP — showed the stake
   was wrong twice over. The signal `|E_P - E_inf|` at this curve's `beta omega = 23.99`
   falls below `3 sigma` (with the `sigma ~ 1e-5` the gauge above achieves) at `P` above
   about 110, so three of the five staked points would have been noise; and the closed
   form's OWN exponent is **1.576** over {8,16,32,64}, **1.773** over {16,32,64} and
   **1.933** over {32,64,128} — the asymptotic 2 is not reached anywhere the measurement
   can see. A band around 2 would have graded a correct instrument against the wrong truth.
   G3 is now stated against the closed form itself, on the window the noise floor allows.

```
exact E_P at omega_harm = 0.022790089111, beta = 1052.583416:
   P      E_P - V_min       E_P - E_inf     |signal| / 3 sigma
   1     0.000950043469      -1.0445e-02          348.2
   2     0.001874386066      -9.5207e-03          317.4
   4     0.003604991653      -7.7901e-03          259.7
   8     0.006322936371      -5.0721e-03          169.1
  16     0.009117612318      -2.2774e-03           75.9
  32     0.010670144560      -7.2490e-04           24.2
  64     0.011200055064      -1.9499e-04            6.5
 128     0.011345343844      -4.9701e-05            1.7
 256     0.011382558166      -1.2486e-05            0.4
 512     0.011391919107      -3.1254e-06            0.1
 inf     0.011395044556        0                    -
```

All three corrections cost edits and no data, which is the whole reason the plants run first.

---

## 8. What the results document must say

Verdict first, per gate, with the fired kill stated as plainly as the survival; the fate of
ALL EIGHT gates named, never only the ones that passed; the referee's four residuals and its
work counts; the statistical error beside every sampled number; the interpolation systematic;
the commuting-square budget as a table over `P`; and the instrument commit. A dead gate stays
in the record, marked dead.
