# SATURATION-1 — the 50-digit referee

*What this file is: the method, the frozen geometry-staking rule, the selftest
record and the measured compute cost for `saturation_referee.py`, the
independent arbiter for gates R1 (trimer referee), T1 (interpolant fidelity,
cross-check basis), T2 (boundary gauge) and F1 (order-4 truncation gauge). It
reports numbers. It scores no gate — the campaign scores its own gates against
these numbers.*

## 1. Method

Every subsystem energy is a full CI in the STO-3G minimal basis over
closed-form Gaussian integrals, evaluated in mpmath, plus the classical nuclear
repulsion. Exact-in-model; not a prediction of experiment.

```
E1            = E(H)                                  1 electron,  1 determinant
V2(r)         = E(H2; r)          - 2 E1              2 electrons, 4 determinants
dE3(r12,r13,r23)
              = E(H3)  - sum_pairs V2   - 3 E1        3 electrons, 9 determinants
dE4(geometry) = E(H4)  - sum_pairs V2 - sum_triples dE3 - 4 E1
                                                      4 electrons, 36 determinants
```

Pipeline, all written in this file: three-dimensional s-type primitive
integrals (overlap, kinetic, nuclear attraction via the Boys function `F0`,
four-centre ERI) → contracted STO-3G 1s per proton, renormalised at working
precision → symmetric (Löwdin) orthogonalisation `X = S^(-1/2)` from a cyclic
Jacobi decomposition of `S` → four quarter-transformations of the ERI tensor →
determinant CI in the minimal-|Sz| block, built by explicit fermionic ladder
operators (no Slater–Condon case analysis anywhere, so no phase-rule
transcription risk) → cyclic Jacobi eigensolve → `<S^2>` of the ground vector
by `S_- S_+ + Sz^2 + Sz`.

The minimal-|Sz| block carries one component of every spin multiplet the system
can form, so its lowest eigenvalue is the global ground state. Multiplicity is
therefore **measured, not assumed**: `<S^2>` is reported at every point.

**Independence (gate R1's premise).** No code is shared with the Rust engine
(`holon-chem`), and none with `h2_core.py` either — the integrals here are 3-D
and freshly written, the CI is general in the number of centres, the
eigensolver is written here. `h2_core.py` is imported only inside `--selftest`,
where it acts as an external referee on this file's `E(H)` and `E2(r)`. The one
thing deliberately shared with every implementation is the **model definition**:
the STO-3G hydrogen contraction. A basis set is an input, not a derivation.

**Precision.** Working precision 80 decimal digits, output 50. The 30-digit
margin is load-bearing, not decorative: `dE3` is a difference of energies of
order 1 Ha whose true value is ~1e-29 at 20 bohr and below 1e-50 beyond 30, and
the MBE cancellation has to survive down there. `--selftest` re-runs a point at
110 dps and requires agreement to 1e-50, and `verify_saturation.py` recomputes
at 70 dps — so the 50 published digits are demonstrated at three independent
working precisions, not asserted.

## 2. The frozen geometry-staking rule

Seed **20260828**. Four blocks, emitted in this order, deduplicated on the
exact side triple (first occurrence wins). Re-running the seed reproduces the
set exactly; `python3 saturation_referee.py --list-geometries` prints it.

| block | n | rule |
|---|---|---|
| `A0-anchor` | 2 | the two unambiguous geometries disclosed in the prereg's feasibility paragraph: `(r_e,r_e,r_e)` and the exactly-collinear `(r_e,r_e,2r_e)` |
| `A-equilateral` | 11 | equilaterals on the fixed ladder 0.90, 1.00, 1.20, r_e, 1.60, 2.00, 2.50, 3.00, 3.75, 4.60, 5.75, 7.00 (r_e deduped against A0) |
| `B-nearlinear` | 12 | six fixed `(a,b)` pairs × squeeze `delta` in {1e-3, 1e-6}, sides `(a, b, (a+b)(1-delta))` — an arbitrarily controlled distance from the collinear wall |
| `C-shell` | 11 | `r23 = 7.0` exactly (the domain's outer wall) at twelve fixed `(r12,r13)`, one deduped against `(7,7,7)` |
| `D-random` | 32 | `random.Random(20260828)`; draw `u1,u2,u3 ~ U(0.90,7.00)` independently, sort, ACCEPT iff `s0+s1 > s2·(1+1e-6)`; keep the first 32 acceptances in draw order |

**68 geometries**, ≥ 64 as R1 requires, spanning compact (0.9 bohr equilateral),
scalene, near-linear and near-boundary. No draw is discarded for any reason
except the triangle inequality, so block D is manifestly not cherry-picked.
`r_e = 1.3886940` bohr throughout.

A separate seven-point **boundary-shell probe** (§4, T2) is carried in
`h3_referee.json` under `boundary_shell_probe`. It is labelled a probe and is
not part of the staked 68.

The **F1 set** is the prereg's six: regular tetrahedron, square, 60° rhombus
(two equilateral triangles sharing an edge), each at edge `r_e` and `1.5 r_e`.
The four triples of each are evaluated by this referee's own H3 machinery,
directly — never interpolated.

## 3. Selftest

`python3 saturation_referee.py --selftest` — 14/14 required checks PASS. A
check that does not run is a refusal (exit 2), not a pass.

```
T1_E_H_matches_closed_form         PASS   |CI - (T+V)/S| = 0.0 ; E(H) = -0.46658184955727545919
T2_E_H_matches_h2_core             PASS   |this - h2_core| = 0.0
T3_E2_matches_h2_core              PASS   |this - h2_core| = 4.217e-81 ; E2(r_e) = -1.1373060512221418804 ; V2(r_e) = -0.20414235210759096203
T4_MBE_zero_at_far_separation      PASS   dE3(40,40,40) = 0.0  (arithmetic noise floor at 80 dps)
T5_H2_plus_far_H_additive          PASS   |E(H3) - E2(r_e) - E(H) - V2(far pairs)| = 4.242e-44
T6_dE3_totally_symmetric           PASS   max-min over the 6 permutations = 2.108e-80
T7_ground_is_doublet               PASS   |<S^2> - 3/4| = 0.0 at the r_e equilateral
T8_eigen_residual_small            PASS   ||Hv - Ev||_inf = 6.325e-81 ; H asymmetry = 0.0
T9_jacobi_matches_mpmath_eigsy     PASS   max |lambda_here - lambda_eigsy| over 9 roots = 2.53e-80
T10_precision_stable_at_110dps     PASS   |dE3@80dps - dE3@110dps| = 1.312e-80
T11_H4_zero_at_far_separation      PASS   dE4(40-bohr tetrahedron of atoms) = 2.74097e-80
T12_anchors_match_disclosed_f64    PASS   E(H)=-0.466581849557 (d=4.43e-10) V2(r_e)=-0.204142352108 (d=1.08e-10) dE3_eq=0.858071012021 (d=1.2e-8) dE3_lin=0.35472768517 (d=3.15e-7)
T13_dE3_at_20_bohr_is_spin_frustration PASS   dE3(20,20,20) = 4.41821448872e-29 ; 3J/2 = 4.41821448872e-29 ; relative deviation = 6.106e-16
T14_dE4_two_dimers_is_four_body    PASS   dE4(30 bohr) = 1.52053435339e-8 ; dE4(60 bohr) = 5.53377018148e-10 ; log-log slope = -4.7801719
```

`python3 verify_saturation.py` — 11/11 required checks PASS, exit 0. It trusts
no number in the JSONs: six staked H3 geometries and two H4 geometries are
recomputed from scratch at 70 dps (a different working precision), the shell
probe is recomputed entire, and the defining identities are re-derived from the
stored strings alone on **all** 68 + 6 rows. Worst recomputation deviation
4.8e-50, worst identity deviation 1.0e-49 — both at the 50-digit rounding of
the published strings, as they should be. `--full` recomputes every row
(68 + 6 + 7): same verdict, worst deviation 6.6e-50.

### The verifier was mutation-tested — it can fail

A check that cannot fail is not a check. Five planted defects, each caught by
the check that should catch it, with the stated exit code:

| planted defect | caught by | exit |
|---|---|---|
| `dE3[0]` wrong in the 44th significant digit | `V3_h3_spot_recomputed`, `V4_h3_dE3_identity_from_stored` | 1 |
| `E_H4` of tetrahedron@r_e wrong in the 44th digit | `V9_h4_spot_recomputed`, `V10_h4_dE4_identity_from_stored` | 1 |
| one geometry silently dropped, count left stale | `V1_h3_contract_wellformed` | 1 |
| shell-probe maximum overstated as T2-compliant (1e-6) | `V11_shell_probe_recomputed` | 1 |
| unreadable contract | refusal, not a pass | 2 |
| clean tree | — | 0 |

Known coverage limit, stated rather than hidden: a defect that corrupts `E_H3`
and `dE3` *consistently* satisfies the stored-string identity, so outside the
six spot-check indices only `--full` catches it. Run `--full` before any
page-changing verdict.

### Two brief specifications were wrong about the model, and are corrected here

Stated plainly, per the house rule that a fired check is reported as plainly as
a survival:

1. **`dE3` at a 20-bohr geometry is NOT below 1e-40.** It is
   `+4.41821448872e-29` at the 20-bohr equilateral. This is physics, not
   arithmetic: the equilateral trimer is **spin-frustrated**. Three atoms
   cannot pair all three bonds into singlets, and mapping the wide-separation
   limit onto a Heisenberg trimer `H = J·sum(S_i·S_j - 1/4)` predicts, with no
   free parameter, `dE3 -> +3J/2` where `J = E_triplet - E_singlet` is the H2
   exchange gap at the same distance. Measured: `3J/2 = 4.41821448872e-29`,
   agreeing to **6.1e-16 relative** (T13). The engine's f64 probe reads
   `-2.3e-15` there and calls it machine zero; it is 14 decades below f64's
   floor and the referee resolves it. The MBE-closes-to-arithmetic-precision
   check it was meant to be now runs at 40 bohr, where the physics really is
   dead and `dE3` returns exact 0.0 (T4).
2. **`dE4` for two far-separated dimers is NOT zero.** It is `+1.52e-8` Ha at
   30 bohr. Each H2 carries a permanent quadrupole; a bare H atom in this basis
   has no permanent moment and no polarisability, so every atom **pair** and
   every atom **triple** is dead at that range and the entire molecule–molecule
   interaction lands at **order 4**. Measured log-log slope: −4.78 (30→60),
   −4.94 (100→200), −4.97 (200→400) — converging on the −5 of
   quadrupole–quadrupole, from above, as an attractive R^-6 dispersion
   correction requires. The order-4 zero check now uses four *atoms* at 40 bohr,
   where it returns 2.7e-80 (T11). **Consequence for F1: the four-body term does
   not become negligible just because the atoms are far apart — it becomes the
   *leading* term.** F1's ratio is meaningful only on the compact set, which is
   what the prereg stakes.

## 4. Measured values the campaign will want

Reported, not scored.

**Gate T2 (boundary gauge, kill if max |dE3| on the shell > 1e-5 Ha).** The
staked `C-shell` block and the seven-point collinear probe both read far above
that:

| shell geometry | dE3 (Ha) |
|---|---|
| max over the staked C block, at (3.55, 3.55, 7.00) | 1.7530e-2 |
| max over the collinear probe, at (3.50, 3.50, 7.00) | **1.9002e-2** |
| (3.00, 4.00, 7.00) | 1.1396e-2 |
| (2.00, 5.00, 7.00) | 1.9049e-3 |
| (0.90, 6.10, 7.00) | 2.3612e-4 |
| the far corner (7.00, 7.00, 7.00) | 5.8459e-5 |

Even the far corner is 5.8× the threshold; the compact edge of the shell is
**1900×** it. The mechanism is plain once seen: a triple with two short sides
and one 7.0-bohr side is a *contacting* trimer, not a distant one, and zeroing
`dE3` the moment any side crosses 7.0 puts a ~1.9e-2 Ha step, and an unbounded
force spike, into the potential. The probe walks the shell's compact edge
(`r12 + r13 = 7.0` exactly) and is the right instrument for this; the staked C
block alone would have under-read it by 8%.

**Gate F1 (order-4 gauge).** All six compact geometries, both spin readings:

| geometry | dE4 (Ha) | sum dE3 (Ha) | max dE3 (Ha) | \|dE4\|/max\|dE3\| | \|dE4\|/\|sum dE3\| |
|---|---|---|---|---|---|
| tetrahedron @ r_e | −1.45313 | +3.43228 | 0.85807 | 1.693 | 0.423 |
| square @ r_e | −0.91473 | +2.29905 | 0.57476 | 1.591 | 0.398 |
| rhombus60 @ r_e | −1.25227 | +2.58832 | 0.85807 | 1.459 | 0.484 |
| tetrahedron @ 1.5 r_e | −0.73574 | +1.76871 | 0.44218 | 1.664 | 0.416 |
| square @ 1.5 r_e | −0.34175 | +0.95001 | 0.23750 | 1.439 | 0.360 |
| rhombus60 @ 1.5 r_e | −0.53326 | +1.22273 | 0.44218 | 1.206 | 0.436 |

**The prereg's `|dE4|/|dE3|` is ambiguous per geometry, and the gate's verdict
flips on the reading.** Against the largest single triple term, the ratio
exceeds 1 at **6 of 6** geometries. Against the sum of the four triple terms —
the total order-3 contribution the order-4 term corrects — it is 0.36–0.48,
below 1 at **6 of 6**. Both columns are in `h4_referee.json`; the campaign must
say which it meant before scoring, and either way should note that a 36–48%
order-4 correction is a slowly-converging expansion, not a negligible tail.
`dE4` is negative at every geometry while `dE3` is positive at every geometry
in the whole 68-point grid: order 4 partially undoes order 3.

**The Sz=0 ground state is not always a singlet.** Measured, not assumed: at
the r_e tetrahedron and the r_e square the lowest state of the Sz=0 block is a
**triplet** (`<S^2> = 2.0` exactly, three-fold degenerate); the other four
geometries are singlets. `h4_referee.json` therefore carries both readings —
`E_H4`/`dE4` (the block minimum, i.e. the system's true ground state) and
`E_H4_S0`/`dE4_S0` (the lowest S=0 state, separated by simultaneous
diagonalisation of `H` and `S^2`). At the r_e tetrahedron they differ by
2.39 mHa, at the r_e square by 7.37 mHa; the F1 ratios move by ≤ 0.013, so F1's
verdict is robust to the choice, but the prereg's phrase "singlet block" is
ambiguous and the engine must say which it computes.

**Plant (i)'s carrier does not reproduce at its disclosed size.** The prereg
discloses "two separated dimers beat the r_e-edge H4 tetrahedron by +0.426 Ha
(singlet block)". This referee measures that gap at the r_e-edge tetrahedron as
**+1.16259 Ha** (block ground state) or **+1.16498 Ha** (lowest S=0) against
`2·E2(r_e) = −2.27461210244428376081`. Scanning the tetrahedron edge:

| edge (bohr) | E(H4) − 2·E2(r_e) |
|---|---|
| 1.3886940 (r_e) | +1.16259 |
| 2.0 | +0.60305 |
| 2.5 | +0.45931 |
| 3.0 | +0.41121 |
| 3.5 | +0.40053 (the shallow minimum) |
| ∞ (four free atoms) | +0.40828 |

+0.426 Ha corresponds to an edge near **2.85 bohr**, not to `r_e`. The **sign is
the same** — two dimers win at every edge — so the plant's carrier is nonzero
and plant (i) remains scoreable; only the disclosed magnitude is off, by 2.7×,
and it is a disclosed prior rather than a staked quantity. Per the
detector-not-verdict rule this says *look at the probe's geometry label*; it
does not by itself say the engine is wrong.

**Anchor agreement with the disclosed f64 values** (T12): `E(H)` to 4.4e-10,
`V2(r_e)` to 1.1e-10, `dE3` equilateral to 1.2e-8, `dE3` linear to 3.2e-7 —
each at the precision the prereg disclosed them to. R1's own 1e-10 Ha bar is
the engine's to meet against the 68-point table.

**Grid shape.** `dE3 > 0` at all 68 staked geometries and all 7 probe points —
the three-body term is repulsive everywhere in the domain. Range: 1.29272 Ha at
the 0.9-bohr equilateral down to 5.8459e-5 Ha at the (7,7,7) corner. `<S^2>` is
exactly 3/4 at all 68: the H3 ground state is a doublet throughout the domain,
as the prereg's scope assumes.

## 5. Measured compute cost

Machine: 32 logical cores. Compute and scope terms only — no calendar terms
(house ban).

| run | scope | wall | notes |
|---|---|---|---|
| full H3 grid, `--grid` | 68 staked + 7 probe H3 points | **27.7 s** | 16 worker processes; driver 0.13 s CPU |
| F1 set, `--h4` | 6 H4 points, each carrying 4 H3 triples, 6 pairs, and a spin-resolved second 36×36 diagonalisation | **40.6 s** | 6 worker processes |
| `--selftest` | 14 checks, incl. two H4 evaluations and a 110-dps re-run | **74.0 s** | 50.3 s CPU; serial by design |
| `verify_saturation.py` | 6 H3 + 2 H4 recomputed at 70 dps, all identities | **42.4 s** | 16 workers |
| `verify_saturation.py --full` | all 68 H3 + all 6 H4 + 7 probe points recomputed at 70 dps | **69.4 s** | 275.6 s CPU, 16 workers |

Per-point serial cost at 80 dps: one H3 geometry ≈ 1.4 s (of which the three
`V2` evaluations are ~0.15 s and 1863 Boys-function calls dominate); one H4
geometry with its four triples and spin resolution ≈ 35 s. The whole campaign's
referee side is therefore minutes of one machine, not a compute-limited item:
if T1's 256 held-out interpolant points are wanted from this referee directly,
that is ≈ 360 s of H3 evaluations on 16 workers.

Scaling notes for a successor: cost is dominated by contracted ERIs, 81
primitives each, `n(n+1)/2 · (n(n+1)/2 + 1) / 2` of them (21 at n=3, 55 at n=4),
each primitive one `erf`. The 36×36 Jacobi is the second term at n=4 and the
reason the H4 points cost 25× an H3 point. Heteronuclear or larger-basis
successors scale by those two counts.

## 6. Reproducing

```
cd conformance/atomworld
python3 saturation_referee.py --selftest          # 14/14, exit 0
python3 saturation_referee.py --grid              # writes h3_referee.json
python3 saturation_referee.py --h4                # writes h4_referee.json
python3 verify_saturation.py                      # 11/11, exit 0
python3 verify_saturation.py --full               # every row, exit 0
python3 saturation_referee.py --point 1.3886940 1.3886940 1.3886940
python3 saturation_referee.py --list-geometries
```

Deterministic: the geometry set is seeded and printed, the eigensolver has a
fixed sweep order, and the parallel map preserves order — every worker's
arithmetic is serial, so the process count never touches a digit.
