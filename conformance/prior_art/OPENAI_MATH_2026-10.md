# OpenAI's math collection (2026-10-06) screened against this programme's problem space

*2026-10-09. Source: `github.com/openai/math` (Apache-2.0; 719 manuscripts in 372 families from an
unreleased internal model; Lean formalizations for a subset, each covering a SELECTED statement
that is often weaker than the paper's claim; unformalized results "could have issues", per the
README). Method: every family's title, summary and abstracts parsed from `CONTENTS.md` and screened
by keyword against thirteen topics of ours, then the hits read by hand; the Lean scope note
(`lean/docs/NNN.md`) read for each shortlisted family. Nothing from the collection was built or
run. Shortlist below; everything else (algebraic geometry, number theory, representation theory,
the bulk of the 372) does not touch our problem space.*

**Nothing touches the stabilizer rank.** No family mentions stabilizer rank, extent, magic states
or Clifford+T decompositions: RANK-1's open cell (`6 ≤ χ(|H⟩^⊗7) ≤ 9`) is untouched.

| family | what it proves | Lean covers | where it touches us | what we could do with it |
|---|---|---|---|---|
| **363** hard-sphere Boltzmann nonuniqueness | two distinct global solutions from the same data, both with exact local conservation of mass, momentum, energy | periodic nonuniqueness only (not the local-conservation refinement) | **the object's core**: conservation does not determine the dynamics — the conserved-density view of a kinetic system is not closed by conservation alone. Bears on reading C ("thinghood is conservation") against A′ (closure the level above carries) | cite in the stance's weights as a theorem against C's strong form; check the Lean statement's hypotheses against `closure_determines_dynamics` (does uniqueness fail because the view is not `Closed`, or because the solution class is too wide?) |
| **364** Boltzmann–Grad limit over the kinetic lifespan, with fluctuations | rigorous particle → Boltzmann closure for stable radial potentials with attractive wells; Gaussian fluctuations governed by the linear fluctuating Boltzmann equation | not formalized | the rigorous analogue of a certified coarse tier: a closed kinetic view with its fluctuations as the priced remainder — for dilute gases, not liquids | a referee carrier: run the removability gate on a dilute hard-sphere gas from our engine and check the carried set is the one-particle distribution's moments, against a theorem rather than a textbook |
| **265** 2D area law from a global spectral gap; PEPS with polynomial bond dimension | entanglement area law for unique gapped ground states on 2D domains; certified PEPS approximation, error `≤ L⁻¹` | not formalized | GF1's Fold III: ENTANGLEMENT obeys an area law in gapped 2D, while our measured MAGIC is extensive — the two are different quantities, as the corrected fold says; and a certified tensor view for the QVM dictionary in 2D | a sentence in GF1's results and TIERS §1 (consistent, not new for us); a 2D tensor view with a proven error bound is a candidate fifth entry in ACUITY's dictionary |
| **215** O(3) continuum limit with a mass gap; exact O(4) lattice mass asymptotic `m ~ 32 e^{π/4−1/2} √β e^{−πβ}` | the asymptotically free 2D model's continuum limit; the O(4) transfer gap's leading asymptotic | exponential correlation decay for O(n≥3) only | the fold below the atom: the 2D analogue of a gauge theory's mass gap and continuum limit, the shape SCHWINGER-3 measured for QED₂ | a referee for the MPS tier: compute the O(4) rotor chain's gap at moderate β by DMRG and compare with the proven asymptotic — a theorem-graded test of our exact tier, cheap |
| **227** SK dynamics: spectral gap, cutoff, critical slowing down `n^{2/3}` | mixing times across the SK transition; universal critical autocorrelation limits | Poincaré inequality for β < 1; cutoff for β < 1/2; the critical window | the SLOW lane with a PROOF behind it: a carrier whose slow dynamics are known rigorously | the theorem-graded test the instrument never had: run the closure search on SK heat-bath trajectories and check its slow sector and its timescale against the proven `n^{2/3}` — a re-finding there would be a re-finding against a theorem |
| **228** first-order phase transition for a radial pair potential in 3D continuum (Simon's problem) | a stable radial potential whose free energy has a derivative jump over a density interval | yes, both potentials | the freezing question on our ladder (TIERS §1: the network may be carried for a nucleation target) — the first rigorous continuum first-order transition of this type | context only: our water operator is not a radial pair potential |
| **376** universal computation in forced Navier–Stokes | smooth forced 3D NS flows from rest whose particle detects Turing-machine halting, at any positive viscosity | the sheet programs and the velocity-field detection, not the full detector | the fluid tier's limits: under forcing, predicting a designated particle from a finite closed view is undecidable in general — no budget closes it | a sentence in the object's "not licensed" list: closure of a driven fluid view is not decidable in general, so the window is a property of the drive as well as the substrate |
| **279** exact factoring over a fixed finite gate set | probability-one polynomial factoring circuits from one finite gate set | yes | the exact-ring tower: exact (not approximate) circuits over a fixed gate set are the workload our cyclotomic arithmetic exists for | a test workload: simulate the circuit family exactly at small `N` on the magic tier and check the output distribution is exactly the factorization |
| **266** exactly three MUBs in dimension six | `N(6) = 3`, by a certified binary64 computation | a weaker bound (≤ 5) and one supporting lemma | qudit stabilizer geometry in a composite dimension, next to our ζ₃/ζ₆ rings | an exact-arithmetic re-check of parts of the floating-point exclusion is conceivable; scope it before promising it |
| 261, 268, 271, 221, 222, 274 | Anderson localization (2D all disorder; 3D weak-disorder a.c. spectrum); the spin-one Haldane gap; Heisenberg magnetization; diluted spin glasses; perceptron jamming; parity ∉ QAC⁰ | various | adjacent (localization, gaps, glassiness, circuit lower bounds) | none planned; the Haldane gap is a known MPS benchmark we can already reproduce |

**How to decide whether any of these moves us, in order of cost.**

1. **Read, don't run, the two that bear on the object** (363, 376): one afternoon. 363 is a theorem
   that conservation does not fix dynamics; 376 that driven-fluid prediction is undecidable. Both
   are about the sentences in `OBJECT.md` and `TIERS.md`, and both should be cited as constraints
   on the weights and the "not licensed" list — after reading the papers' hypotheses, since the
   Lean covers only selected statements.
2. **Use 227 and 215 as theorem-graded referees** for the two instruments that have only ever been
   graded against literature consensus: the closure search (on SK dynamics, where the slow
   timescale is proved) and the MPS tier (on the O(4) chain, where the mass asymptotic is
   proved). Each is a small freeze with its bar taken from the theorem — the cure for
   `M-BAR-FROM-THE-READ`.
3. **Do not build on unformalized claims as if proved.** 364 and 265 are unformalized; 266's Lean
   proves ≤ 5, not 3. Cite them as preprints.
