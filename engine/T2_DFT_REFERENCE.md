# T2 DFT REFERENCE — α-quartz structure and elastic constants (quantum reference leg)

**Status: COMPLETE — structure and full elastic tensor measured against experiment.**

**Verdict up front:** structure lands in the expected GGA class (+0.9%/axis, signed:
expansion), but the "few percent" accuracy class for DFT elastic constants is **REFUTED
for this pinned setup**: the three large constants read **−9.5% to −13.8%** (signed: soft)
at the PBEsol relaxed volume. |C14| alone is at −0.6%. Details and the mechanism below.

This document is the reference leg only (DESCRIPTOR_CHAIN §3.1/§3.2): what DFT says on
this exact pinned setup, against measured values. No engine claims, no descriptors.

## Provenance (warrant coordinates)

| Coordinate | Value |
|---|---|
| Code | Quantum ESPRESSO **7.5** (`pw.x`), conda-forge build `qe-7.5-h19104ac_2` (OpenMPI, ELPA 2025.06.001), installed user-space via micromamba 2.9.0; no system packages touched |
| Functional | **PBEsol**, read from the pseudopotentials (output line `Exchange-correlation= SLA PW PSX PSC`) — PBE is disqualified per §3.1 (quartz/cristobalite ordering reversal) |
| Pseudo dataset | **SSSP 1.3.0 PBEsol Efficiency** (Prandini et al. npj Comput. Mater. 4, 72 (2018)), Materials Cloud archive record `rcyfm-68h65`, DOI 10.24435/materialscloud:f3-ym; tarball md5 `b6cad0d97c86d3b43ae416d6e5d8b771`, metadata JSON md5 `84fbbbd8f67b551abaec2f8a9df86801` (both verified after download) |
| Si pseudo | `Si.pbesol-n-rrkjus_psl.1.0.0.UPF`, md5 `c4212819de858c94c3a1644338846ac9` (ultrasoft; SSSP hints 30/240 Ry) |
| O pseudo | `O.pbesol-n-kjpaw_psl.0.1.UPF`, md5 `81d73d1479e654e5638b0319f0d6c2c7` (PAW; SSSP hints 50/400 Ry) |
| Production cutoffs | ecutwfc **100 Ry**, ecutrho **800 Ry** (dual 8) |
| k-mesh | **6×6×4** Monkhorst–Pack, unshifted (Γ-centred) |
| Occupations | fixed (insulator; gap ~8.9 eV) |
| Cell | 9-atom trigonal cell, P3₁21 (ITA setting; Wyckoff Si 3a (u,0,⅓), O 6c), ibrav=4 |
| Starting geometry | Levien–Prewitt–Weidner (1980): a=4.9134 Å, c=5.4052 Å, u_Si=0.4697, O=(0.4133, 0.2672, 0.2145) — O_z is 1/3−z_LPW, the origin mapping into the ITA P3₁21 setting, verified by the bond network (Si–O 1.6052/1.6134 Å, Si–O–Si 143.60°, O–Si–O 108.8–110.6°) |
| Machine | 24-core i9-13900HX, 31 GB RAM; `mpirun -np 8` (+`-nk 4` pools for strained cells) |
| Working dir | `/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/dft_quartz/` (inputs, outputs, pseudos, fit scripts) |

## Convergence evidence (not asserted — measured)

Total energy and pressure at the experimental cell, experimental internal coordinates,
k = 4×4×3 unless noted. ΔE in meV/atom (9 atoms, 1 Ry = 13605.7 meV):

| ecutwfc/ecutrho (Ry) | E (Ry) | ΔE from previous (meV/atom) | P (kbar) |
|---|---|---|---|
| 60/480 | −281.99712966 | — | +47.85 |
| 80/640 | −282.00314865 | 9.10 | +50.26 |
| 100/800 | −282.00352568 | 0.57 | +50.83 |
| 120/960 | −282.00407608 | 0.83 | +50.87 |

| k-mesh (at 100/800) | E (Ry) | P (kbar) |
|---|---|---|
| 4×4×3 (9 IBZ pts) | −282.00352568 | +50.83 |
| 6×6×4 (24 IBZ pts) | −282.00352669 | +50.79 |

Read honestly: the absolute total-energy tail is non-monotone at the ~1 meV/atom level
(100→120 gives 0.83 meV/atom after 80→100 gave 0.57 — normal for US/PAW augmentation
charges), but **pressure is converged to 0.04 kbar at 100/800** and the k-mesh moves E by
0.002 meV/atom and P by 0.04 kbar. For structure and stress-derived elastic constants the
operative quantity is the stress; 100/800 + 6×6×4 was picked as production. The +50 kbar
at the experimental cell is the frozen-internal-coordinate stress reading, not an error —
quartz's compliance is hinge-dominated, so the fixed-internals lattice is much stiffer than
the relaxed one (see below; the relaxed equilibrium sits only ~1%/axis away).

## Phase 1 result — relaxed structure vs experiment (SIGNED errors)

Variable-cell relaxation (BFGS, `cell_dofree='ibrav'`, press_conv_thr 0.05 kbar), then a
**fresh-basis SCF at the relaxed cell: residual P = +0.04 kbar** (the vc-relax Pulay-basis
caveat is measured away, not assumed away).

| Quantity | This setup (PBEsol) | Experiment (296 K) | Signed error |
|---|---|---|---|
| a | 4.9601 Å | 4.9134 Å (LPW 1980) | **+0.95%** |
| c | 5.4527 Å | 5.4052 Å (LPW 1980) | **+0.88%** |
| c/a | 1.09931 | 1.10009 | −0.07% |
| V | 116.179 Å³ | 113.011 Å³ | **+2.80%** |
| density | 2576 kg/m³ | 2648 kg/m³ (§3.2 pin) | −2.70% |
| u(Si) | 0.47096 | 0.4697 | +0.0013 (toward β-quartz ½) |
| O (x,y,z) | (0.41289, 0.26579, 0.21346) | (0.4133, 0.2672, 0.2145) | (−0.0004, −0.0014, −0.0010) |
| Si–O bonds | 1.6211 / 1.6246 Å | 1.6052 / 1.6134 Å | +0.99% / +0.69% |
| Si–O–Si angle | 143.77° | 143.60° (from same coords) | +0.17° |

**Reading.** The comparison is athermal DFT (0 K, no zero-point) against room-temperature
experiment — ~0.5–1%/axis deviations are the expected class for a GGA, so neither
celebration nor panic: this is a *characterization* of the reference instrument. The signed
systematic of this exact setup is **expansion**: both axes ~+0.9%, volume +2.8%. Quartz
thermal expansion 0→300 K is ~+0.1%/axis, so against a 0 K-extrapolated experiment the
signed error would be slightly *larger* (~+1.0%), i.e. the sign is robust, not a thermal
artifact. The c/a ratio and internal coordinates are nearly exact (−0.07% and ~1e-3
respectively): the error lives almost entirely in the isotropic scale, the tetrahedral
units expand with it (+0.7–1.0% bonds), and the soft Si–O–Si hinge is reproduced to 0.2°.
For §3.1's B1 gate framing (lattice within 1.5% of the named reference): a hypothetical
candidate gated against THIS reference inherits this +0.9% expansion bias against
experiment; the bias direction must be carried per §3.1's certificate clause (3), not
laundered into a symmetric band.

## Phase 2 — elastic tensor vs experiment (SIGNED errors)

Method: finite strains ±0.5% and ±1% on the relaxed (P=+0.04 kbar) cell in three patterns
— ε₁ (yields C11, C12, C13, C14), ε₃ (C33, C13 cross-check), ε₄ (C44, C14 cross-check via
σ₁=−σ₂) — internal coordinates relaxed at each strain (forc_conv_thr 1e-5 Ry/au), stress
read after relaxation, linear+quadratic fits through the five points including the
unstrained zero. Sign convention pinned empirically: QE's printed stress is
positive-under-compression (the compressed experimental cell printed +50 kbar and
vc-relax *expanded*), so C_ij = −(slope of printed σᵢ vs εⱼ). All 12 relaxations
converged; symmetry-forbidden components (σ₅, σ₆ everywhere; σ₄ under ε₃) read
numerically zero, as class 32 requires.

| Constant | This setup (GPa) | Measured (GPa) | Signed error (GPa) | Signed error (%) |
|---|---|---|---|---|
| C11 | 74.9 | 86.8 | **−11.9** | **−13.7%** |
| C33 | 91.2 | 105.8 | **−14.6** | **−13.8%** |
| C44 | 52.7 | 58.2 | **−5.5** | **−9.5%** |
| C12 | 3.3 | 7.0 | −3.7 | −53% |
| C13 | 7.5 | 11.9 | −4.4 | −37% |
| \|C14\| | 17.9 | 18.0 | −0.1 | **−0.6%** |
| B_Voigt (derived) | 30.8 | 37.9 | −7.1 | −18.7% |

Measured anchors: Bechmann 1958 / Heyliger–Ledbetter–Kim 2003 class values per
DESCRIPTOR_CHAIN §3.2 (their ~1% mutual spread is the honest tolerance floor). C14's
sign: −17.9 GPa in **our frame** (ITA P3₁21 setting, QE ibrav=4 axes, the O_z = 1/3−z_LPW
origin mapping); the sign flips under the IEEE-1949↔1978 axis convention and under
handedness/Dauphiné twin swap (§3.2's mandatory-field caveat), so the magnitude is the
convention-free comparand.

**Internal quality checks (all passed):**
- Linearity: pure-linear, quadratic-corrected, and ±0.5%-only fits agree to ≤0.4 GPa on
  every component — the ±1% window is comfortably in the linear regime.
- Cross-checks: C13 = 7.30 (from ε₁) vs 7.60 (from ε₃, both σ₁ and σ₂ identical) — 4%
  internal spread on a small constant; C14 = −17.80 / −17.92 / −17.96 from three
  independent readings (ε₁/σ₄, ε₄/σ₁, −ε₄/σ₂) — 0.9% spread.
- Precision floor: the σ(0) residual (0.04–0.06 kbar) against ~9 kbar signals puts the
  numerical floor at ~0.5% on large constants, ~0.2 GPa on C12/C13 — the internal spreads
  above are consistent with it. The measurement precision is far tighter than the
  physical error it reveals.

**Verdict on the accuracy class.** §3.1's B1 gate frames "C_ij within 2%" and the task's
prior was "a few percent" for DFT elastic constants. For THIS pinned setup (PBEsol,
SSSP 1.3.0 efficiency, athermal, at the DFT-relaxed volume) that claim is **REFUTED**:
the large constants are soft by 9.5–13.8%, signed, all in the same direction. The
absolute errors on C12/C13 are small (−3.7/−4.4 GPa) but their percentages are large
because the constants themselves are small; |C14| is the outlier that lands at −0.6%.

**Mechanism, and what it means for the programme.** The elastic softening is coupled to
the Phase 1 volume expansion: quartz's C_ij have large positive pressure derivatives, so
evaluating at a +2.8% expanded equilibrium volume reads soft — this is the known GGA
systematic on quartz (the §3.1 certificate clause (3) names "PBE softens quartz C_ij";
PBEsol softens too, and here is its measured size). The bias is one-signed and
mechanistically understood, which is exactly what clause (3) requires be carried: a
signed bias per observable, never a symmetric band. Comparison is athermal-DFT vs 296 K
experiment; quartz constants stiffen on cooling, so a 0 K-extrapolated comparison would
make the deficit slightly LARGER — the sign is robust, as with the lattice. Routes to a
tighter reference, each a declared choice rather than a free lunch: (a) evaluate C_ij at
the experimental volume (imports experiment into the reference — provenance changes from
"pure DFT" to "DFT constrained by measured cell"); (b) a stiffer rung of the functional
ladder (SCAN is §3.1-admissible and typically halves GGA volume errors — untested here);
(c) carry the −10 to −14% signed bias explicitly on any certificate that consumes this
reference. What survives at the few-percent class from this leg: the geometry
(c/a −0.07%, internal coordinates ~1e-3, hinge angle 0.2°), |C14| (−0.6%), and the
*anisotropy pattern* (C33/C11 = 1.218 vs measured 1.219 — signed error −0.1%): the
SHAPE of quartz's elasticity is captured to sub-percent; the absolute STIFFNESS is soft
by ~10–14%.

## Run ledger

12 strained-cell relaxations + 1 vc-relax + 7 convergence/smoke SCFs, ~2 h wall total on
8 MPI ranks (of 24 cores; machine shared). All inputs/outputs under the working
directory's `runs/` and `elastic/`; fit script `elastic/fit_elastic.py` reproduces every
number in the Phase 2 table from the raw outputs.
