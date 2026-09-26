# ORDER-1 — PRIOR ART: what is already known about a non-conserved slow field that the hydrodynamics of water carries

*Written 2026-09-25, BEFORE `ORDER1_PREREG.md` and before any ORDER-1 reader exists, so the read
can be graded as RE-FINDING or DIRECTION against a record fixed in advance. The lead's question:
on every tier the search's carried directions were the conserved quantities, except two "weak
ones" — tetrahedral order on the sluggish liquid (`SLOW1_RESULTS.md`) and the conscience sector
on reasoning chains — each a non-conserved variable the level above carries anyway. The
analogue in physics is the ORDER PARAMETER, and for a liquid the relevant literatures are four:
generalized hydrodynamics (the non-hydrodynamic relaxing modes of the density–current sector),
mode-coupling theory (structural relaxation as the slow memory of the density), the two-state
picture of water (a structural order parameter coupled to the density), and dielectric relaxation
(the collective dipole mode). Each is summarised with what it predicts for THIS carrier: the
rigid operator, 250 waters in a 19.58 Å box, oxygen positions and velocities at 20 fs, 50 ps.
First-harmonic wavevector `k = 2π/L = 0.321 Å⁻¹`.*

## 1. Generalized hydrodynamics: the density–current sector is not closed at molecular `k`, and the extra variables are RELAXING modes

- **Mountain (1966)**, *Rev. Mod. Phys.* **38**, 205 — the Rayleigh–Brillouin spectrum from
  linearized hydrodynamics; in a liquid with an internal relaxing variable the longitudinal
  (bulk) viscosity acquires a memory, `η_L(ω) = η_∞ ... /(1 − iωτ_s)`: the **structural
  relaxation** of the "Mountain mode", a non-conserved variable coupled into the sound pair
  through the longitudinal stress. This is the textbook form of the lead's analogue: a
  non-conserved, slowly relaxing variable that the conserved sector CARRIES (its equation of
  motion for `j_L` depends on it).
- **Boon & Yip, *Molecular Hydrodynamics* (1980); Hansen & McDonald, *Theory of Simple Liquids*
  (ch. 8–9)** — the memory-function / Mori–Zwanzig formalism: the slow variables are the
  conserved densities `ρ_k, j_k, e_k`; everything else enters as memory kernels, and extended
  hydrodynamics promotes the slowest kernel to an explicit relaxing variable.
- **Generalized collective modes (GCM)** — de Schepper, Cohen, Kamgar-Parsi and co-workers
  (kinetic modes, 1980s); **Mryglod, Omelyan & Tokarchuk, *Mol. Phys.* 84, 235 (1995)**; Bryk &
  Mryglod in a long series on simple, binary, ionic and supercritical fluids. GCM is, operationally,
  **the same instrument as ours restricted to one `k`**: a dictionary of dynamical variables
  (`ρ_k, j_k, e_k` and their time derivatives), the generalized hydrodynamic matrix built from
  their static and time-integrated correlations, its eigenvalues read as propagating and relaxing
  modes. It routinely finds, beyond the hydrodynamic heat and sound modes, non-hydrodynamic
  **relaxing "kinetic" modes** in the longitudinal sector, one of which is identified with
  structural relaxation. Applied to water: **Bertolini & Tani, *Phys. Rev. E* 51, 1091 (1995)**
  ("Generalized hydrodynamics and the acoustic modes of water", TIP4P; density, energy, current
  correlations), *PRE* 52, 1699 (1995) (stress and viscosity), *PRE* 56, 4135 (1997) (thermal
  conductivity); Bryk & Mryglod on water; and **the BK3 / SPC/E study, *J. Chem. Phys.* 161,
  184507 (2024)** ("Longitudinal and transverse collective dynamics in water by simulation
  using the BK3 model"), fitting GCM with relaxing modes in the longitudinal component.
- **What GH predicts here.** At `k = 0.32 Å⁻¹` the longitudinal sector has the sound pair
  (VIEW-SEARCH-1 read its oscillation: period `~1.6–2` ps on the smaller boxes), the heat mode
  (`D_T k²`; for real water `D_T ≈ 1.4 × 10⁻⁷ m²/s` gives `~0.7` ps at this `k`), and one or
  more relaxing non-hydrodynamic modes with the structural relaxation time. **The walks carry no
  energies, so the heat mode is NOT in our hydrodynamic dictionary** — any slow scalar field
  correlated with local temperature or potential energy will partly re-find it. This is a named
  confound for ORDER-1: the kinetic-energy density `K_k = Σ|v_i|² e^{ik·r_i}` is the only energy
  proxy the walk has, and it misses the potential energy.

## 2. Mode-coupling theory: structural relaxation IS the slow memory of the density

- **Leutheusser (1984); Bengtzelius, Götze & Sjölander (1984); Götze & Sjögren, *Rep. Prog. Phys.*
  55, 241 (1992); Götze, *Complex Dynamics of Glass-Forming Liquids* (2009).** The density
  correlator's memory is a functional of the density correlators at all `k`; the α-relaxation
  (two-step decay, stretched-exponential α tail, time-temperature superposition) is the slow
  variable, and it is a property of the density field itself at the structure-factor peak, not
  a separate order parameter.
- **Water: Sciortino, Gallo, Tartaglia & Chen, *PRE* 54, 6331 (1996)** (self dynamics of
  supercooled SPC/E) **and *PRE* 56, 5397 (1997)** (collective dynamics, two-step relaxation,
  `Q`-dependence) — MCT as the framework for water's α relaxation; the α time at ambient
  temperature is `~1` ps and grows steeply on supercooling. Experiment: **Monaco, Cunsolo,
  Ruocco & Sette, *PRE* 60, 5505 (1999)** — IXS of water in the THz range fitted with a
  viscoelastic memory: a structural relaxation time of order `0.5–1` ps at room temperature,
  consistent with lower-frequency techniques; **Torre, Bartolini & Righini, *Nature* 428, 296
  (2004)** — structural relaxation of supercooled water by time-resolved spectroscopy.
- **What MCT predicts here.** At `k = 0.32 Å⁻¹`, far below the structure-factor peak (`~2 Å⁻¹`),
  the density's own memory carries the structural relaxation only through the longitudinal
  viscosity; the slow structural variable lives at molecular `k`. A coarse structural field at
  the FIRST harmonic is expected to decay at the rate of the LOCAL structure (its self part,
  `F_s(k,t) ≈ 1` at this `k` over tens of ps), i.e. with the single-molecule memory of `q` —
  **unless** structure is collectively correlated over the box, which §3 says it is only weakly.

## 3. The two-state picture: a structural order parameter coupled to density

- **Tanaka, *J. Chem. Phys.* 112, 799 (2000); *EPJE* 35, 113 (2012)** — water as a
  two-order-parameter liquid: density `ρ` and a bond-orientational order `S` (the fraction of
  locally favoured tetrahedral structures), with a Landau coupling between them; the anomalies
  follow from `S`'s temperature dependence. **Russo & Tanaka, *Nat. Commun.* 5, 3556 (2014)** —
  the `ζ` order parameter (second-shell translational order) and the LFS fraction; **Shi, Russo
  & Tanaka, *PNAS* 115, 9444 (2018)** — the dynamics as a two-state ("fast" and "slow" local
  structures, both Arrhenius) crossover; **Shi & Tanaka, *JACS* 142, 2868 (2020)** — two peaks in
  the first diffraction peak, the tetrahedral one a density wave at longer period.
- **Nilsson & Pettersson, *Nat. Commun.* 6, 8998 (2015)** ("The structural origin of anomalous
  properties of liquid water"); **Wernet et al., *Science* 304, 995 (2004)**; **Huang et al.,
  *PNAS* 106, 15214 (2009)** — X-ray evidence read as fluctuations between tetrahedral
  low-density and disordered high-density local structures; the small-angle scattering
  enhancement read as their density contrast (contested: Clark et al., *PNAS* 107, 14003, 2010).
- **Poole, Sciortino, Essmann & Stanley, *Nature* 360, 324 (1992); Palmer et al., *Nature* 510,
  385 (2014)** (a metastable liquid–liquid transition in ST2 by free-energy methods);
  **Debenedetti, Sciortino & Zerze, *Science* 369, 289 (2020)** (a second critical point in
  TIP4P/2005 and TIP4P/Ice); **Gallo et al., *Chem. Rev.* 116, 7463 (2016)** (review); **Holten
  & Anisimov, *Sci. Rep.* 2, 713 (2012)**; **Anisimov et al., *PRX* 8, 011004 (2018)** — the
  order parameter of fluid polyamorphism is the fraction of the interconverting state, and it
  couples to density; the critical point is where that coupling makes the order-parameter
  fluctuation collective and long-ranged.
- **Errington & Debenedetti, *Nature* 409, 318 (2001)** — the tetrahedral order `q` used here and
  in SLOW-1; `q`'s space-time correlations: **Kumar, Buldyrev & Stanley, *PNAS* 106, 22130
  (2009)** and arXiv:0807.4699 — `C_q(t)` shows a two-step decay at low temperature, similar to
  `F_s`; spatial correlation of `q` is short and anticorrelated at the first shell at high `T`.
- **The directly comparable measurement: Sedlmeier, Horinek & Netz, *JACS* 133, 1391 (2011)** —
  "Spatial correlations of density and structural fluctuations in liquid water", several force
  fields: all pure and mixed two-point correlations of density, tetrahedrality and H-bond count.
  **Except density–density, all show only weak features: water's tendency to form structural
  clusters is much weaker than its tendency to form density clusters.** At ambient conditions,
  then, the static cross-correlation `S_ρQ(k)` at small `k` is expected to be WEAK; the
  two-state order parameter becomes collective only near the (supercooled) critical region.
- **What the two-state literature predicts here.** At 293–304 K (this operator's production
  temperature), far above any putative second critical point, `Q_k` should be a sum of nearly
  independent local fluctuations: its static correlation with `ρ_k` small but nonzero with the
  sign "more tetrahedral ⇔ lower density", its relaxation the local one. The two-state picture's
  DISTINCTIVE dynamical signature — two relaxation times (fast/slow local structures) or a
  collective component slower than the local one — is what would make a read here a direction
  rather than a re-finding; its absence at ambient `T` is the expected outcome.

## 4. Dielectric relaxation and the collective dipole mode (Arm D; for the record)

- **Debye, *Polar Molecules* (1929)**; **Buchner, Barthel & Stauber, *Chem. Phys. Lett.* 306, 57
  (1999)** — water's permittivity from 0.2 to 410 GHz, 0–35 °C, fitted by two Debye processes:
  the main one `≈ 8.3` ps at 25 °C, a fast one `≈ 1` ps. **Madden & Kivelson, *Adv. Chem. Phys.*
  56, 467 (1984)** — the collective (macroscopic) and single-molecule times related through the
  Kirkwood factor; confirmed in SPC/E and TIP3P simulations. **Agmon, *J. Phys. Chem.* 100, 1072
  (1996)** — "tetrahedral displacement" as the molecular mechanism of the Debye relaxation.
  **Hansen, Kisliuk, Sokolov & Gainaru, *PRL* 116, 237601 (2016)** — the structural relaxation
  appears only as a high-frequency shoulder in the dielectric spectrum; the main Debye peak is
  supramolecular (as in monoalcohols): the collective dipole mode is NOT the structural mode.
- **For this carrier:** the rigid walks bank the oxygen positions and velocities only (`# walk N
  L`, `3 × 250` columns per readout, written by `write_flexible_walk` in `replace0.rs`). No
  hydrogen position or orientation quaternion is on disk, so the dipole cannot be formed and
  Arm D is refused by name in the prereg.

## 5. What this record means for grading ORDER-1

1. **A closed direction of the first-harmonic tetrahedral-order field, decaying with one time
   equal to the single-molecule `q` memory, outside the span of `ρ_k, j_k`,** is the expected
   outcome from §2–3: a local relaxing variable summed over the box. It is a RE-FINDING of
   extended hydrodynamics' relaxing structural variable (Mountain; GCM's relaxing modes;
   Monaco et al.'s viscoelastic time), with `q` as the named field (Errington–Debenedetti).
2. **Its being CARRIED into the density–current sector** is predicted by GH/MCT only through the
   longitudinal stress (the relaxing longitudinal viscosity), i.e. into `j_L` and thence `ρ`,
   and at ambient `T` the coupling should be weak (Sedlmeier et al.). A large carried increment
   would be beyond what this literature leads one to expect at this `T` and `k`; a small one is
   Mountain's mechanism re-found.
3. **A DIRECTION** would be: two relaxation times in the collective field where the single
   molecule shows one, a collective component slower than the local memory, or a `Q`–`ρ`
   mixture closed and carried that the hydrodynamic sector does not carry — and then the
   two-state literature (§3) is what it must be compared against before any claim.
4. **Confounds named in advance:** the heat mode (no energies on the walk; `K_k` is a partial
   proxy); the per-seed static density offsets (VIEW-SEARCH-1 §3); the self part dominating the
   collective field at this `k`; a 250-water box at 293–304 K with a rigid operator whose own
   `D` and `q` memory are its own (SLOW-1), not real water's.

*Sources consulted in the search of 2026-09-25 (web search results; primary citations as above):*
[Mountain 1966](https://link.aps.org/doi/10.1103/RevModPhys.38.205);
[Bertolini & Tani 1995](https://journals.aps.org/pre/abstract/10.1103/PhysRevE.51.1091);
[Bertolini & Tani 1995b](https://journals.aps.org/pre/abstract/10.1103/PhysRevE.52.1699);
[Bertolini & Tani 1997](https://journals.aps.org/pre/abstract/10.1103/PhysRevE.56.4135);
[BK3/SPC/E GCM study 2024](https://pubs.aip.org/aip/jcp/article-abstract/161/18/184507/3320040/Longitudinal-and-transverse-collective-dynamics-in);
[Sciortino et al. 1996](https://link.aps.org/doi/10.1103/PhysRevE.54.6331);
[Sciortino et al. 1997](https://journals.aps.org/pre/abstract/10.1103/PhysRevE.56.5397);
[Monaco et al. 1999](https://doi.org/10.1103/physreve.60.5505);
[Torre et al. 2004](https://www.nature.com/articles/nature02409);
[Russo & Tanaka 2014](https://www.nature.com/articles/ncomms4556);
[Shi, Russo & Tanaka 2018](https://www.pnas.org/doi/10.1073/pnas.1807821115);
[Shi & Tanaka 2020](https://pubs.acs.org/doi/10.1021/jacs.9b11211);
[Nilsson & Pettersson 2015](https://www.nature.com/articles/ncomms9998);
[Holten & Anisimov 2012](https://www.nature.com/articles/srep00713);
[Anisimov et al. 2018](https://journals.aps.org/prx/abstract/10.1103/PhysRevX.8.011004);
[Sedlmeier, Horinek & Netz 2011](https://pubs.acs.org/doi/10.1021/ja1064137);
[Kumar, Buldyrev & Stanley, q space-time correlations](https://arxiv.org/pdf/0807.4699);
[Buchner, Barthel & Stauber 1999](https://www.sciencedirect.com/science/article/abs/pii/S0009261499004558);
[Agmon 1996](https://pubs.acs.org/doi/abs/10.1021/jp9516295);
[Hansen et al. 2016](https://link.aps.org/doi/10.1103/PhysRevLett.116.237601).
