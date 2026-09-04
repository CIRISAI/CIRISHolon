# Pre-registration — EMBED-1: the embedding field — external charges in the Hamiltonian, the one-body density, and the far field of two fragments

*Frozen 2026-09-04, committed ALONE, before `embed.rs` existed. Built by the lead, no
lane. The git order is the check. This is Build 1 of the two water builds the
prior-art search (`PRIOR_ART_FARFIELD_SEAM.md`) reshaped: every embedded expansion
in the literature closes at two-body because each fragment is solved INSIDE the
field of the rest; ours never closed (dE5: 24/24 over bound) because every cluster
was solved in vacuum. This freeze builds the field and stakes what it must do.*

**Do not read this as chemistry.** STO-3G throughout. What is measured is whether
a fragment solved in the point-charge field of its partner reproduces the exact
supermolecule's FAR interaction in the same basis. Basis-set superposition error is
present in the exact side at short range and is the stated reason the near sector
is reported and never read.

---

misfits: contacts **M-ONE-MODEL-DELTA** (the referee is the exact determinant
solve of the supermolecule in the same basis; the comparison is exact-vs-embedded,
never approximate-vs-approximate); **M-VACUOUS-SUCCESS** (a node count sits on
every gate and a gate passing on fewer nodes than staked is VOID); **M-NULL-MISSTAKE**
(each gate is staked on the quantity its premise controls: the density on its
trace and its free-atom size, the field term on its derivative, the far field on the
interaction energy); **M-EXIT-DISCRIMINATOR** (S3's two branches are named here and
neither is the default; a non-converged self-consistent iteration is VOID, never a
number); **M-PLANT-OBS** and **M-PLANT-SECTOR** (three plants, each with its carrier
asserted nonzero in the sector the plant acts on — §5); **M-STALE-INSTRUMENT** (this
freeze is committed alone; the runner, its results document and the JSON it writes
are committed together); **M-CHEAPER-THAN-ITS-PRICE** (the water-dimer referee's
per-node price is measured on ONE node before its knots are admitted — G0);
**M-UNTESTED-GAP** (the far-field claim is staked on the distance range it is tested
on and nowhere else; the near knots are reported, not read); **M-FIXED-POINT-TRAJECTORY**
(the self-consistent charges must reach the SAME fixed point from two
different starts — G4 — so a fixed point that is an artifact of the start cannot
pass); **M-MAX-OVER-SUCCESSES** (S3 is a for-all over every far node, never the best
one); **M-DEVICE-CLASS** (one device class, the host determinant solver
`solve_determinant`, for every number here; `fci::solve`, which routes by size, is
not used); **M-HOMOG** (the words "local" and "distant" appear below; no homogeneity
is assumed — every claim is at a staked node); **M-BARE-CHARGE** (the word "charge"
appears throughout; these are CLASSICAL point charges in a Coulomb Hamiltonian, not
charged states of a gauge theory, and no dressing question arises); **M-VOLUME-SCALE**
(Ewald is NAMED as the exit for the periodic far field and not built; nothing here
takes a volume limit). NOT contacted, named so the absence is deliberate:
**M-GAUGE-LAUNDER**, **M-PARITY-PROTECT**, **M-LOOP-BLIND**, **M-COND-PROBE**,
**M-KINEMATIC-NONLOCAL**, **M-ELECTRIC-BASIS**, **M-RING-MIXING**,
**M-GAUGE-UNIFORM-MOMENTUM**, **M-PROBE-EIGENSTATE**, **M-NONBIJECTIVE-STEP**,
**M-FINAL-VIEW-COLLISIONS**, **M-MAINTENANCE-LENS**, **M-IDLE-CALIBRATED-TIMEOUT**,
**M-PROBE-THE-RESOURCE**, **M-CACHE-KIND**, **M-PLACEMENT-LOTTERY**,
**M-TRUNCATION-AS-ERRORBAR** — no gauge observable, no trajectory, no cached
artifact, no accelerator arm and no variational truncation arises in a
ground-state electrostatics campaign on staked rigid geometries.

---

## 0. The claim, split

The engine carries no electrostatic term (`B2_PREREG.md` §1.1). Every cluster the
many-body ladder solves is solved in vacuum, and the ladder does not terminate
(`DE5_RESULTS.md`). The literature's embedded expansions close at two-body (Gillan et
al. 2013; Dahlke and Truhlar 2007) because each fragment is solved inside the
field of the rest. Build 1 is the instrument for that field and this freeze stakes
three separable things:

- **S1 — the one-body density is exact and is the record's own object.** The
  spin-summed one-body reduced density matrix built from the determinant vector has
  the electron count as its trace, and on a free atom it reproduces
  `atomic_rms_radius` — the same quantity by a second route.
- **S2 — the field term is exact in Hellmann–Feynman's sense.** The energy of a
  fragment in the field of a test charge has, as its derivative with respect to that
  charge, the electrostatic potential the fragment's density and nuclei produce at
  the charge's position. Full CI satisfies this exactly, so any disagreement is a
  defect in the integrals or the density, not in the physics.
- **S3 — the far field of two closed-shell fragments is captured by mutual
  point-charge embedding at staked distances.** Each fragment solved inside the
  other's charges, the charges taken from the embedded densities and iterated to a
  fixed point, reproduces the exact supermolecule's interaction energy on the far
  sector of a staked grid to within a staked fraction.

What this is NOT: not a basis-set claim; not Ewald (named exit for the periodic far
field, prior art since 1921, owned by whoever builds the ice box); not the seam
(Build 2 puts exact cores inside this field and is its own freeze).

## 1. The instrument (`holon-chem/src/embed.rs`, new; one refactor in `pair.rs`)

- `rdm1(space, vector)`: `γ_pq = Σ_σ ⟨ψ|a†_pσ a_qσ|ψ⟩` in the orbital basis of the solve,
  from the string tables' own single-excitation lists. `ao_density(γ, C)` back-
  transforms it to the AO basis: `P = C γ Cᵀ`.
- Moments in the AO basis by the existing Hermite tables, built two powers past
  `LMAX` for the kinetic term and therefore already able to carry `(x−C)` and
  `(x−C)²`: first moments (the dipole) and second moments about any point.
- `build_basis_embedded(species, centres, externals)`: the external charges are
  extra centres with a charge and NO shells. The nuclear-attraction loop already
  sums over every centre, so the electronic Hamiltonian acquires the field with no
  new integral code; `nuclear_repulsion` sums every centre pair, so the
  external–external self-energy is subtracted explicitly and asserted absent. The
  electron count comes from the species alone.
- `potential_at(P, basis, R)`: `V(R) = Σ_A Z_A/|R−A| − Σ_μν P_μν ⟨μ| 1/|r−R| |ν⟩`,
  the second term through the same nuclear-attraction machinery with a unit charge
  at `R`.
- Charge models. PRIMARY: **dipole-exact charges** — neutral, equivalent atoms
  equal, magnitude fixed so the charges reproduce the embedded density's exact
  dipole (a diatomic has two centres and is fully determined; water's two
  hydrogens are equal by symmetry and the dipole lies on the C₂ axis, so it is
  determined too). CONTROL, reported beside it and never read for S3: Mulliken
  charges `q_A = Z_A − Σ_{μ∈A} (PS)_μμ`. The primary is chosen so that its failure
  mode is the QUADRUPOLE and penetration terms of the far field, which is the
  physical question, rather than Mulliken's known failure to carry the dipole.
- `embed_pair(A, B, geometry, model)`: solve A in B's charges and B in A's, take
  each embedded density's charges, repeat until the largest charge change is below
  `1e-9`, at most 100 iterations (else VOID). `E_emb = E_A[q_B] + E_B[q_A] −
  Σ_{a∈A,b∈B} q_a q_b/r_ab` (the charge–charge energy is counted in both embedded
  solves and removed once — the EE-PA convention of Dahlke and Truhlar), and
  `ΔE_emb = E_emb − E_A⁰ − E_B⁰` with the isolated monomers in their own basis.
- The referee: `ΔE_exact = E_AB − E_A⁰ − E_B⁰`, the exact determinant solve of the
  supermolecule in the same basis, raw (no counterpoise on either side; both sides
  in one convention). Every solve is `solve_determinant` on the host.

## 2. The systems, staked

**System 1 — the HF dimer, the cheap referee.** Two rigid HF monomers at the
engine's own STO-3G determinant minimum (G1), in the linear hydrogen-bonded
arrangement F–H···F–H, all four atoms collinear, with the fluorine–fluorine
distance on the grid

```
R_FF (Å) ∈ { 2.4, 2.6, 2.8, 3.0, 3.5, 4.0, 5.0, 6.0, 8.0, 10.0 }
```

Ten nodes. Supermolecule: 12 spatial orbitals, 20 electrons, `C(12,10)² = 4,356`
determinants. FAR SECTOR: `R_FF ≥ 5.0 Å` — four nodes (5.0, 6.0, 8.0, 10.0). NEAR
SECTOR: `R_FF ≤ 3.0 Å` — four nodes, reported, never read for S3 (exchange
repulsion, charge transfer and basis-set superposition live there and the
embedding is not staked against them). The two middle nodes are the transition
and are reported as such.

**System 2 — the water dimer, the target, PRICED FIRST.** Two rigid H₂O monomers
at the engine's own STO-3G determinant minimum (the DIMER-1 G1 rule, computed here
because DIMER-1 has not run), in DIMER-1's `LINEAR` arrangement, at
`R_OO (Å) ∈ { 4.0, 4.5, 5.0, 6.0, 8.0 }`. Supermolecule: 14 orbitals, 20 electrons,
`C(14,10)² = 1,002,001` determinants. FAR SECTOR: `R_OO ≥ 5.0 Å`, three nodes.
Admitted only if G0 passes; dropped and SAID SO otherwise, in which case S3 is
read on System 1 alone.

## 3. Gates

Node counts are stated on every gate; a gate passing on fewer than its staked
count is VOID rather than a pass.

- **G0 — the price of System 2, measured on one node before its knots are
  admitted.** One (H₂O)₂ node (`LINEAR`, R_OO = 8.0 Å) on `solve_determinant`, the
  processor time recorded. The five water nodes are admitted only if that time is
  `≤ 30` processor-minutes; else System 2 is dropped from this campaign and the
  results document says so. 1 node.
  witness: none (a measured price has no Lean object)
- **G1 — the monomer pins (EXACT, in the sense of a pin).** HF: the bond length at
  the engine's own STO-3G determinant minimum, central-difference gradient
  `|dE/dR| ≤ 1e-6` hartree/bohr. H₂O: the bond length and the angle at the minimum,
  both gradients `≤ 1e-6`. Recorded to ten digits in the results document; every
  dimer node uses them. 2 pins.
  witness: none (an engine-internal geometry pin)
- **G2 — the density is exact.** On every monomer solve of both systems (isolated
  and embedded): `|tr γ − N| ≤ 1e-12`, `|tr(PS) − N| ≤ 1e-12`, `|Σ_A q_A| ≤ 1e-12`
  for both charge models, `max|γ − γᵀ| ≤ 1e-12`. On the free H, O and F atoms the
  density route's `sqrt(⟨r²⟩/N)` reproduces `atomic_rms_radius` to `1e-10`
  relative — the same object by two routes. 3 atoms + every monomer solve.
  witness: none (an identity between two in-engine routes)
- **G3 — the field term is exact (Hellmann–Feynman).** The HF monomer and a test
  charge at six staked points: on the molecular axis at 3, 6 and 12 bohr beyond the
  hydrogen, and off-axis at the same three distances from the fluorine along a
  perpendicular. At each point the central difference `(E(+h) − E(−h))/2h` with
  `h = 1e-4` agrees with `V(R)` from the density and the nuclei to `≤ 1e-7` hartree
  per unit charge. 6 points.
  witness: none (Hellmann–Feynman on an exact eigenvector; the check is arithmetic)
- **G4 — the self-consistent field has ONE fixed point.** On every dimer node of
  both admitted systems, the iteration started from zero external charges and the
  iteration started from the isolated monomers' charges converge (largest charge
  change `< 1e-9`) within 100 iterations to the same charges within `1e-8` per
  centre and the same `E_emb` within `1e-10` hartree. A node failing either is VOID
  and reported. All nodes.
  witness: none (a measured fixed-point property)
- **S3 — THE DECISION: the far field.** On the far sector of each admitted system,
  `ρ(R) = |ΔE_exact(R) − ΔE_emb(R)| / |ΔE_exact(R)|` with the PRIMARY charge model.
  **Branch (a):** `ρ ≤ 0.25` at EVERY far node of System 1 (4 nodes), and of
  System 2 if admitted (3 nodes), AND the absolute residual `|ΔE_exact − ΔE_emb|`
  is non-increasing across the far sector's consecutive nodes or below `1e-8`
  hartree ⇒ mutual point-charge embedding is the far field the seam (Build 2)
  is built inside, and the pair-tail model of B2 becomes its control.
  **Branch (b):** any far node with `ρ > 0.25`, or a residual that grows with
  distance ⇒ the far field needs more than charges; the residual is reported with
  its distance scaling (a residual falling as `R⁻⁶` is dispersion, which no
  charge model carries, and is named as such rather than read as a failure of
  the field), and the next freeze stakes what to add. The near sector's `ρ` is
  reported under both branches and read under neither. The Mulliken control's
  `ρ` is reported beside the primary's and read for nothing.
  witness: none (a measured far-field residual; no Lean object covers it)

## 4. What each outcome means

Branch (a) is what the prior art predicts for a polar closed-shell pair at five
ångström and beyond, and it is what licenses Build 2's shape. Branch (b) is the
more informative outcome: it says which term the field is missing, by the
residual's scaling, and it is why the primary charge model is dipole-exact — a
Mulliken failure would have been a failure of the model, not of the idea, and
would have taught nothing. G0 failing removes the water referee and does not touch
the claim, which is then read on the HF dimer alone and says so. A G4 VOID on any
node removes that node from S3 and is reported by name.

## 5. Plants

Each plant is a deliberate defect the runner can switch on; each names the gate
that must fire and asserts its carrier NONZERO IN THE SECTOR the plant acts on,
checked before the plant's verdict is trusted.

- **(i) The field's sign.** The external-charge attraction enters with the wrong
  sign. G3 must fire: the finite-difference derivative then disagrees with `V(R)`
  by `2|V|`. Carrier: `|V(R)| ≥ 1e-3` hartree per unit charge at the nearest
  staked point, asserted nonzero in the sector the plant acts on — a polar
  molecule at 3 bohr.
- **(ii) The double count.** The charge–charge subtraction is omitted from
  `E_emb`. S3's far residual must then read `ρ ≈ 1` on every far node. Carrier:
  `|Σ q_a q_b / r_ab| ≥ 1e-6` hartree at every far node, asserted nonzero in the
  sector the plant acts on.
- **(iii) The fermionic phase.** The sign in the single-excitation lists is
  dropped when the density is built. The trace is sign-blind and G2's trace check
  must NOT fire — that is the point — while G3 must fire, because the
  off-diagonal density enters `V(R)`. Carrier: the off-diagonal contribution to
  `V(R)` at the staked points, `≥ 1e-6` hartree per unit charge, asserted nonzero
  in the sector the plant acts on.

## 6. Provenance and discipline

Runner: `holon-chem/examples/embed_campaign.rs`, writing
`conformance/water_observatory/embed/*.json`; tests in `holon-chem/tests/embed.rs`
carry G2, G3, G4 and the three plants on System 1; the results document
`EMBED_RESULTS.md` is committed with the runner and cites this freeze. Every solve
is `solve_determinant` on the host, one device class; processor time is what is
recorded. No number enters from outside the engine: monomer geometries are the
engine's own minima (G1), charges are the engine's own densities, the referee is
the engine's own exact solve.
