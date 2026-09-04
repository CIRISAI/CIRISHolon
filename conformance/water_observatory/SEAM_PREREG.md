# Pre-registration — SEAM-1: exact cores INSIDE the field — does the embedded expansion close at two-body where the bare one does not?

*Frozen 2026-09-04, committed ALONE, before `seam.rs` existed. Built by the lead, no
lane. Build 2 of the two water builds. EMBED-1 read branch (a): mutual point-charge
embedding is the far field of two closed-shell fragments. This freeze tests the seam's
corrected premise on the smallest system that has a three-body term: THREE fragments,
the embedded two-body expansion against the exact trimer, beside the bare two-body
expansion whose error IS the bare three-body term. The seam node (GANTT `MPS`) was
fired by measurement twice — dE5's 24 of 24 over bound, worst 1,572× — and the
prior-art search found the one word its design had wrong: the far field must be a
field the cores are solved INSIDE (`PRIOR_ART_FARFIELD_SEAM.md`). This is that
premise, staked.*

**Do not read this as chemistry.** STO-3G throughout. What is measured is whether the
three-body term the bare ladder cannot terminate is carried by the embedding, on
staked rigid geometries, in the same basis, against the same exact referee.

---

misfits: contacts **M-ONE-MODEL-DELTA** (the referee is the exact determinant solve
of the trimer in the same basis; both expansions are compared to it, never to each
other alone); **M-VACUOUS-SUCCESS** (node counts on every gate; a node whose bare
three-body term is below the resolution floor is VOID for S1, never a pass);
**M-NULL-MISSTAKE** (the embedded residual is staked against the bare three-body
term of the SAME node, not against zero); **M-EXIT-DISCRIMINATOR** (S1's branches
named here, neither the default; a non-converged fixed point is VOID);
**M-PLANT-OBS** and **M-PLANT-SECTOR** (two plants, carriers asserted nonzero in
the sector each acts on — §5); **M-STALE-INSTRUMENT** (this freeze alone; runner,
results document and JSON together); **M-CHEAPER-THAN-ITS-PRICE** (the trimer's
per-node price measured on ONE node before the grid is admitted — G0);
**M-PLACEMENT-LOTTERY** and **M-DEVICE-CLASS** (the price is staked in WALL time on
purpose this time, with the thread count the solver used recorded beside it in the
JSON and the device class declared — EMBED-1's G0 refused an affordable referee by
staking processor time on a solver that uses every core, a constant inherited across
a threading regime; the lottery this trades for is named and accepted: one machine
class, one run, both clocks reported); **M-UNTESTED-GAP** (the seam claim is staked
on the far sector it is tested on; the near sector is reported, not read);
**M-FIXED-POINT-TRAJECTORY** (two starts, one fixed point — G2); **M-MAX-OVER-SUCCESSES**
(S1 is a for-all over the far nodes); **M-HOMOG** (the words "local" and "distant"
appear; nothing homogeneous is assumed); **M-BARE-CHARGE** (classical point charges,
not charged gauge states); **M-VOLUME-SCALE** (Ewald named as the periodic exit, not
built). NOT contacted, named so the absence is deliberate: **M-GAUGE-LAUNDER**,
**M-PARITY-PROTECT**, **M-LOOP-BLIND**, **M-COND-PROBE**, **M-KINEMATIC-NONLOCAL**,
**M-ELECTRIC-BASIS**, **M-RING-MIXING**, **M-GAUGE-UNIFORM-MOMENTUM**,
**M-PROBE-EIGENSTATE**, **M-NONBIJECTIVE-STEP**, **M-FINAL-VIEW-COLLISIONS**,
**M-MAINTENANCE-LENS**, **M-IDLE-CALIBRATED-TIMEOUT**, **M-PROBE-THE-RESOURCE**,
**M-CACHE-KIND**, **M-TRUNCATION-AS-ERRORBAR**.

---

## 0. The claim

The bare many-body expansion of three fragments has error exactly the bare three-body
term:

```
E_PA      = E_AB + E_AC + E_BC − E_A − E_B − E_C          (every solve in vacuum)
ΔE_3^bare = E_ABC − E_PA
```

The EMBEDDED pairwise expansion (Dahlke and Truhlar's EE-PA) solves every monomer in
the field of the OTHER TWO fragments' charges and every dimer in the field of the
THIRD's, the charges taken self-consistently from the embedded monomers:

```
E_EE-PA   = E_AB[q_C] + E_AC[q_B] + E_BC[q_A] − E_A[q_B,q_C] − E_B[q_A,q_C] − E_C[q_A,q_B]
r_emb     = E_ABC − E_EE-PA
```

where every energy is the fragment system's own energy in the field — its electrons
and nuclei with the charges, never a charge–charge term (`embed::solve_embedded`
subtracts the external self-energy and asserts it). At the charge level the QM–charge
terms cancel pairwise between the dimer and monomer sums (six enter, six leave), so
`E_EE-PA` differs from `E_PA` by exactly what the field DOES to each solve: the
polarisation of each fragment and pair by the third. That difference is a
three-body quantity carried at two-body cost.

**S1 — the seam's premise.** On the far sector, `|r_emb| ≤ 0.25 · |ΔE_3^bare|` at
every node — the embedding carries at least three quarters of the three-body term
the bare ladder cannot terminate — and `|r_emb|` falls with distance at least as fast
as `|ΔE_3^bare|` does.

**What this is not.** Not a water result (the water cores come under the next freeze
if this one reads (a)); not the periodic far field (Ewald, named); not a claim about
the near sector, where non-additive exchange lives and the embedding is not staked.

## 1. The instrument (`holon-chem/src/seam.rs`, new, on `embed.rs`)

- `embed_many(fragments, model, start)`: every fragment solved in the field of all
  the others' charges, charges from each embedded density (dipole-exact PRIMARY,
  Mulliken CONTROL), iterated until the largest charge change is below `1e-9`,
  at most 100 iterations (else VOID). EMBED-1's `embed_pair` is the `N = 2` case.
- `ee_pa(fragments, charges)`: the six embedded solves above and their sum.
- `bare_pa(fragments)`: the six vacuum solves and their sum.
- `supermolecule` of three: the exact trimer, `solve_determinant` on the host.
- The plants as switches on the runner (§5).

## 2. The system, staked

The HF trimer as a LINEAR CHAIN, F–H···F–H···F–H, all six atoms collinear, rigid
monomers at EMBED-1's G1 pin (`R_HF = 1.8794379298` bohr, re-checked by the runner
to `|dE/dR| ≤ 1e-6`), equal fluorine–fluorine spacing

```
R_FF (Å) ∈ { 2.6, 2.8, 3.0, 3.5, 4.0, 5.0, 6.0, 8.0 }
```

Eight nodes. Trimer: 18 spatial orbitals, 30 electrons, `C(18,15)² = 665,856`
determinants; dimers 4,356; monomers 36. FAR SECTOR: `R_FF ≥ 5.0 Å`, three nodes.
NEAR SECTOR: `R_FF ≤ 3.0 Å`, three nodes, reported, never read for S1. The two
middle nodes are the transition. Fragment A is the chain's first monomer, B the
middle, C the last; the pair AC sits at `2 R_FF`.

The chain is chosen because its three-body term at long range is COOPERATIVE
INDUCTION — B is polarised by both neighbours, and that polarisation changes what A
and C see — which is exactly the term a self-consistent charge embedding is built to
carry and a bare pair expansion cannot. Non-additive exchange (near) and three-body
dispersion (Axilrod–Teller, `R⁻⁹` on a chain) are what it cannot carry, and they
are the residual's expected content under branch (b).

## 3. Gates

Node counts stated on every gate; a gate passing on fewer nodes than staked is VOID.

- **G0 — the price, in the clock the budget is spent in.** One trimer node
  (`R_FF = 8.0` Å) on `solve_determinant`; WALL seconds recorded with the thread
  count the solver used and the device class, processor seconds beside them. The
  eight nodes are admitted only if that wall time is `≤ 900` s; else the grid is cut
  to the far sector plus `3.0` Å (four nodes) if `≤ 1800` s, else the campaign
  returns a refusal and says so. 1 node.
  witness: none (a measured price)
- **G1 — the monomer pin.** EMBED-1's pin re-derived by the runner: `|dE/dR| ≤ 1e-6`
  hartree/bohr at `R_HF = 1.8794379298` bohr. 1 pin.
  witness: none (an engine-internal pin)
- **G2 — one fixed point for three.** On every node, the iteration from zero
  charges and from the isolated monomers' charges converge within 100 iterations to
  the same charges within `1e-8` per centre and the same `E_EE-PA` within `1e-10`
  hartree; a node failing is VOID and named. All admitted nodes.
  witness: none (a measured fixed-point property)
- **G3 — the embedded machinery reduces EXACTLY to the bare one.** With every charge
  set to zero, `E_EE-PA` equals `E_PA` to `1e-12` hartree on every node — the
  identity that catches a charge–charge term left in, a field wired to the wrong
  fragment, or a monomer counted twice. All admitted nodes.
  witness: none (an in-engine identity)
- **S1 — THE DECISION.** On the far sector (`R_FF ≥ 5.0` Å, 3 nodes), with the
  PRIMARY charge model, define `κ(R) = |r_emb(R)| / |ΔE_3^bare(R)|`. A far node
  whose `|ΔE_3^bare| < 1e-7` hartree is VOID for S1 (its three-body term is below
  the referee's resolution) and reported. **Branch (a):** `κ ≤ 0.25` at EVERY
  posable far node AND `κ` non-increasing across the far sector's consecutive nodes
  ⇒ the seam's premise holds: exact cores solved inside the field close at
  two-body to the staked fraction where the bare ladder needs three, and Build 2's
  next freeze puts the water cores inside this field. **Branch (b):** any posable
  far node with `κ > 0.25`, or `κ` rising with distance ⇒ the residual is reported
  with its consecutive-node scaling (an `R⁻⁹` residual on the chain is three-body
  dispersion, which no charge model carries, and is named as such; a slower one is
  induction the point charges mis-carry and the next freeze stakes polarisable
  embedding). Fewer than 2 posable far nodes ⇒ VOID, and VOID is not (a). The near
  and transition sectors' `κ` are reported under both branches and read under
  neither; the Mulliken control's `κ` is reported beside the primary's and read for
  nothing.
  witness: none (a measured residual against a measured term)

## 4. What each outcome means

Branch (a) is the seam's premise measured true on the smallest carrier that has the
term: the cost of the far field is a self-consistent set of charges and a second
solve per fragment, and the higher-body terms that made dE5 un-terminable are
absorbed, which is Gillan's ice result reproduced in this engine's own arithmetic.
Branch (b) says what the field is missing by the residual's scaling, and is the more
informative outcome: three-body dispersion is a known price, polarisation a known
next model. G0 cutting the grid removes nodes and touches no claim; a G2 VOID
removes its node by name.

## 5. Plants

Each plant is a switch on the runner; each names the gate that must fire and
asserts its carrier NONZERO IN THE SECTOR the plant acts on, checked before the
plant's verdict is trusted.

- **(i) The field left out of the pairs.** The dimers are solved in vacuum while the
  monomers stay embedded. G3 must still pass (zero charges make the two identical)
  and S1 must fire: with the QM–charge terms no longer cancelling, `|r_emb|` exceeds
  `|ΔE_3^bare|` at every far node. Carrier: the field's effect on the pair,
  `|E_AB[q_C] − E_AB| ≥ 1e-6` hartree at every far node, asserted nonzero in the
  sector the plant acts on.
- **(ii) The twin confusion.** Mulliken charges in place of dipole-exact. S1's
  margin must shrink: `κ(Mulliken) > κ(primary)` at every posable far node.
  Carrier: the two models' dipoles differ by `≥ 0.05` a.u. on the embedded
  monomers (EMBED-1 measured `0.279` against `0.410`), asserted nonzero in the
  sector the plant acts on.

## 6. Provenance and discipline

Runner `holon-chem/examples/seam_campaign.rs`, JSON per node under
`conformance/water_observatory/seam/`, resumable by skipping nodes on disk; tests in
`holon-chem/tests/seam.rs` carry G2, G3 and both plants at two nodes; results in
`SEAM_RESULTS.md`, committed with the runner. Every solve `solve_determinant` on the
host; both clocks and the thread count in every JSON. No number enters from outside
the engine.
