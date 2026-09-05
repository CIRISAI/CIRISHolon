# The Reality Workbench — pure-dependency build graph

*No calendar. Nodes order by what each unblocks downstream. An arrow is a dependency; a node
is DONE when its receipt-gate is green, never before. Statuses: DONE · LAUNCHED (lane named)
· READY (deps met) · GATED (deps pending) · OPEN (no freeze yet). Each row carries its current
state and the one number that decides it; the campaign histories live in the records the
last column names, and in git. Rewritten 2026-09-04.*

## The graph

```
F   GPU production dispatch ─────────────▶ multiplies every SOLVE-BOUND node (E11–E14 are the path)
A   Species-generic MBE (done) ──┬─▶ C2 ion tables (done) ─▶ Grotthuss honesty
                                 └─▶ E  NQE in dynamics (launched)
B1  Long-range audit (done) ─▶ B2 split-kernel far field (landed; no charge in the force law)
EMBED-1  the embedding field (read) ─▶ SEAM-1 exact cores inside it (read) ─▶ EMBED-2 the field
         as densities (read; residual harvested) ─▶ EMBED-3 the residual's field dependence (read)
         ─▶ THE CHANNEL LEDGER (OBJECT.md rule 10; LEDGER-0 landed) ─▶ FIELD-1 channel 1 in the force law (landed, S1 b)
         ─▶ FIELD-2 a bonded start (read: (c) by letter, VOID in substance — the unit is not a pair verdict;
            the closure surfaces repel across the seam) ─▶ FIELD-3 the unit as a closure; closures serve only within;
            the wall NOT harvested: the residual is penetration (read (c)) ─▶ EWALD-1 the field in the wrapped box (read, landed)
            ─▶ FIELD-4 the density field at the seam, then induction ─▶ a periodic liquid
         ─▶ the water cores inside the field ─▶ G rung 1 gets electrostatics
H   dE5 audit (done: the bare ladder does not terminate) ─▶ MPS seam (re-shaped by EMBED/SEAM)
G   Upward closures (rungs 1, 2 measured, not certified) ◀── OOO cert · dE4 table · B1 · the field
LG  Continuum-native lattice tier (launched) — never composed through viewClosed_comp with G
GF0 (read a) ─▶ GF1 (open) ─▶ GF2 the Σ(1080) hadron box (gated) ─▶ NUC nuclear tier (gated)
W   the waterbench zoom ladder (up) ─▶ W2 gauge vacuum in the browser ─▶ L6 chiral leptons (open)
```

## Nodes

| # | node | deps | status | record |
|---|---|---|---|---|
| F | GPU production dispatch: device-class-declared GPU sigma in the leased table generators; mixed classes refused | gpu-prod crates | LAUNCHED · lane silent since 2026-09-02; F is also the compute-priced fences' exit | FENCES.md |
| A | Species-generic, order-generic many-body machinery (`cluster.rs`: any arity, any order, measured reaches in the registry, refusal by name where none) | — | DONE 2026-09-02; bit-identical to the four-body sector it replaced on staked scenes | `tests/many_body_identity.rs`, `many_body_order.rs` |
| B1 | Long-range residual audit and its successor B1b (work-unit pricing) | trajectories | DONE; class-H negligible at 2.6e-11 of criterion; water class non-negligible (O–O support to 20 bohr) | LONGRANGE_PREREG/RESULTS, B1B_PREREG |
| B2 | Split-kernel far field for a force law with NO electrostatic term; Ewald named as the exit, not built | B1b | LANDED; N-scaling exponent 2.122; four gates kept fired (G4 VOID, G7, G8, periodic arm) | B2_PREREG.md, B2_RESULTS.md |
| EMBED-1 | The embedding field: external charges in the Hamiltonian, the one-body density, the potential, dipole-exact charges (Mulliken control), two fragments to a fixed point (`embed.rs`) | dE5, prior art | READ 2026-09-04: branch (a) on the HF dimer — ρ ≤ 1.5e-2 on the far sector against 0.25; water dimer refused by its own price gate (207.6 processor-min on 27 threads vs 30 staked) | EMBED_PREREG.md, EMBED_RESULTS.md |
| SEAM-1 | Exact cores inside the field: N fragments to a fixed point, the embedded pairwise expansion vs the exact 665,856-det HF trimer (`seam.rs`) | EMBED-1 | READ 2026-09-04: the embedding carries 99.93 % of the three-body term (κ = 5.1e-4…6.8e-4); S1 branch (b) by its letter on the monotonicity clause; AMENDMENT 1 measured the floor (3–4e-12) and S1′ reads (b) on measured ground; M-FLOOR-UNSTAKED registered | SEAM_PREREG.md, SEAM_AMENDMENT_1.md, SEAM_RESULTS.md |
| EMBED-2 | The field as the partners' DENSITIES (Coulomb-only frozen-density embedding, credited); the residual harvested | SEAM-1 | READ 2026-09-04: branch (b) by its letter (κ_ρ > κ_q) — and the residual IS the three-body dispersion: `r = −C/R⁹`, C = 8.5 Ha·bohr⁹, one constant on three nodes, floor 1e-12; the seam's far-field defect is DERIVED | EMBED2_PREREG.md, EMBED2_RESULTS.md |
| EMBED-3 | The harvested residual's dependence on the field its core sits in (channel 4 inside channel 1 — the channel ledger's separability); the water dimer's far field priced in wall time | EMBED-2 | READ 2026-09-05: **System A BRANCH (a)** — inside a fourth monomer's density the harvested three-body term moves by 3.8e-3, 3.2e-3, 3.2e-3 of itself at 6, 8, 12 Å against a 0.10 stake (against the in-process reference, at the floor beyond 8 Å); plant (ii) fires at 3× the term; the frozen reference's printed precision named (M-FORMAT-FLOOR). **System B BRANCH (a)** — the charge field carries 98–99.6 % of the water dimer's far interaction, the density field within 5 % of it; five million-determinant dimers at 574–950 wall-seconds. Two nodes killed under the lead's own CPU contention and rerun; the price gate priced a solve where a node holds three | EMBED3_PREREG.md, EMBED3_RESULTS.md |
| FIELD-1 | **THE FIELD ENTERS THE FORCE LAW as channel 1 of the channel ledger** (OBJECT.md design rule 10): fixed derived charges on water units (the engine's own bond verdict — A1), the Coulomb term with analytic forces, its energy row, its transfer column `work.field`, the wrapped box refused (Ewald the exit) | EMBED-1, ACUITY-B's transition pattern | LANDED 2026-09-04: G0–G5 green (identity in bytes; ledger closed to 8e-6 with transitions posted; momentum 5e-14; force = derivative to 1.2e-9; q_H = 0.2314 from the density; refusal), three plants fire; **S1 BRANCH (b)**: fixed charges form no hydrogen bond in 20,000 frames — the field is net repulsive on the staked scene and the waters separate; FIELD-2 stakes polarisation, the charges' geometry dependence and the O···H table's role. Fence P20: the drift bound's envelope is 20 hartree on water | FIELD_PREREG.md, FIELD_AMENDMENT_1/2/3.md, FIELD_RESULTS.md |
| LEDGER-0 | **The channel ledger as DECLARATIONS in the engine** (OBJECT.md rule 10): the five channels as records (kind, arity, derived rate, shape, receipt, prior art), the ledger rows as a table in `energy()`'s order with their carriage per channel, ONE allocator replacing three budget-to-radius dialects, the tail exponent read as law vs fit with an opt-in refusal; no sum reordered, no number moved | EMBED-3's reading; Backpass III §7 | DONE 2026-09-05 on the branch: bit-identical to the pre-ledger engine on a 46-line receipt (rows, sums, digest, cutoffs, far radii; two scenes), 7/7 gates; channel 4's far side and FIELD-2 named as the freezes it does NOT include | `channel.rs`, `tests/channel_ledger.rs`, CHANNEL_LEDGER.md |
| FIELD-2 | Does the fixed-charge field HOLD a hydrogen bond? A bonded start (dimer, cyclic tetramer), 293 K and 150 K, the field's binding at the start (M1) and the charge's geometry sensitivity (M2) measured before the arms | FIELD-1 | READ 2026-09-05: **S1 BRANCH (c) BY LETTER, VOID IN SUBSTANCE** — M1 = 0 exactly on every start: the engine's pair verdict reads the O···H hydrogen bond (3.56 bohr, E_rel −0.017) as BONDED, the unit rule (two hydrogens, none shared) then assigns no water, and the field is never on a bonded configuration; ON and OFF dimers bit-identical until parted; both plants VOID by carrier; M2 = 0.34 (material). Unstaked finding: the bare force law REPELS at the hydrogen-bond geometry, +21 mHa (pair −21, three-body +42 from cross-molecule triples served by the monomer's (O,H,H) surface; one triple alone +20), and the released energy is the 198 → 543 K the arms read. M-EMPTY-SECTOR registered and armed. FIELD-3 named: the unit as a closure reading (strongest bond), closure surfaces confined within units with cross-unit contacts on the ledger's channels, then polarisation | FIELD2_PREREG.md, FIELD2_RESULTS.md |
| FIELD-3 | **The unit as a closure, closure surfaces confined within units, channel 5's wall HARVESTED from the exact dimer's residual over the field, the hydrogen bond re-asked** (OBJECT.md rule 10 in the force law) | FIELD-2, EMBED-3's exact dimers, LEDGER-0 | READ 2026-09-05: the identity and the seam rule LAND — units 2/4 where FIELD-2 found 0/0, bindings −4.1/−12.0 mHa, the +42 mHa served across the seam dropped EXACTLY (G-B4), books/momentum/derivative green, the switch posting −20.4 mHa = FIELD-2's cross-unit sum; G-A2: the rules disagree at the FIRST frame on FIELD-1's own scene (receipt re-banked, cause named); the engine change adversarially reviewed (two virial signs fixed, one FIELD-1's; three refusals named). **S1 BRANCH (c) by letter**: six exact nodes (2.5–3.7 Å, all converged, 16.6–52.7k core-s) put the residual over the field at +9.6 mHa (2.5 Å), +1.0 (2.7), then −1.3, −1.5, −0.84, −0.36 mHa — a wall inside 2.8 Å and a MISSING ATTRACTION of a third to a half of the field beyond it, decaying faster than any power (penetration); a two-node positive prefix harvests no wall, S2/S3 not run. Unstaked diagnostic with a two-node wall: dimer f 0.03 (293 K) / 0.68 (150 K), ring 0.47 / 0.90 — the seam law binds, kT unbinds, the gap is the harvest's −1.4 mHa. FIELD-4 named: the density field at the seam, then induction | FIELD3_PREREG.md, FIELD3_AMENDMENT_1.md, FIELD3_RESULTS.md |
| EWALD-1 | **The field in the wrapped box**: the Ewald sum as a pure module, then served where FIELD-1 refused (G5 superseded by G6) — the door to a periodic liquid | FIELD-1's refusal, FIELD-3's units | READ 2026-09-05: the lattice sum is right — Madelung to 3.8e-9, force = derivative to 2.2e-9, virial = 3V·dE/dV to 8.8e-10, the open-box limit as L⁻³ (exponent −3.06), both plants fire; two gates FAIL BY LETTER for the freeze's own arithmetic (E1's 0.7× split outside its ε; E4's 1e-6 staked at 80 bohr where L⁻³ reaches it at 137) and G6's staked scene is one the engine's image rule refuses (drifts 1.6e-2 with the field OFF) — on the smallest legal cell the field's drift is the bare law's (9.1e-6 vs 8.3e-6). Integrated: `accumulate_field` dispatches on `wraps()`, `PeriodicNeedsEwald` retired (type uninhabited), the receipt unchanged (G7) | EWALD_PREREG.md, EWALD_RESULTS.md |
| C | Charged fragments at the solver seam; H3O⁺/OH⁻ certified; ion tables through the generic door | A | DONE (core and tables); H3O⁺·H2O priced out at 9.0M determinants (compute fence, exit F); OH⁻ tables refused under I-5 | ION_TABLES_PREREG/RESULTS, `ion_table.rs` |
| D | Periodic-table availability and the fence ledger | — | DONE; 54 species probed (38 determinant, 16 MPS, 18 relativity-fenced); 49 fences with owner and exit | FENCES.md, PERIODIC_AVAILABILITY.md |
| H | dE5 truncation audit: the bare ladder at order five | dE4 | DONE: does NOT terminate — 24/24 over bound, worst 1,572×; explained by EMBED/SEAM (the ladder was bare) | DE5_RESULTS.md |
| E | Nuclear quantum effects in dynamics (ring-polymer in `Sim`); P = 1 bit-identical to classical; per-law gates | C1, A | LAUNCHED · node-e-rpmd | — |
| MPS | The cluster seam: exact cores over a far field, DMRG where determinants are priced out | H, the field | RE-SHAPED: the caps are retired (admission by price); the seam's design is now EMBED/SEAM's — field + harvested residuals; the water triple (21 orbitals) is this node's core, on the E14 base | GF2A_QCD2_RESULTS.md (E14) |
| G | Upward closures: H-bond network and fluid tiers certified as Closed views | OOO cert, dE4, B1, the field | rungs 1 and 2 MEASURED, NOT CERTIFIED: in-budget and dynamic exactly disjoint; the boundary is ALIGNMENT (inter-molecular H-bonds in 0–18 of 20,000 frames) — the missing electrostatics EMBED-1 now supplies | RUNG1_RESULTS.md, RUNG2_RESULTS.md |
| LG | Continuum-native lattice-gas tier certified on its OWN dynamics; the molecular–lattice seam a separate claim | — | LAUNCHED · lattice-tier | — |
| ACUITY-B | The observer's frame as an allocation law (coarse-step path in `Sim`, every transition ledgered) | two-box law | DONE 2026-09-02: branch (a) — 0.018 bohr / 0.041 D_e cost at 76.4 % pair work saved; crossover to load-bearing at 6 bohr | ACUITY_B_RESULTS.md |

## The fold below the atom — GF nodes (LOCKED, `OBJECT.md`)

| # | node | deps | status | record |
|---|---|---|---|---|
| GF0 | SCHWINGER-4: two screened pairs' residual interaction decays at the banked vector-meson mass | — | READ: branch (a) on both columns, rate at the gap to 0.6 %; Python cross-check to 1e-10 | SCHWINGER4_RESULTS.md |
| GF1 | The magic price of gauge vacua across the coupling; the kill is a volume law | exact tiers | OPEN · prereg owed | — |
| GF2 | The Σ(1080) hadron box with staggered quarks; referees p, n, π and the deuteron | GF0 (a), GF1, the QVM split | GATED; its 1+1D rehearsal (E7 → E14) met the exact referee on all six N = 8 sectors and was closed as prior art at volume (Silvi 2019; Hayata 2023) | GF2A_QCD2_RESULTS.md |
| NUC | The nuclear tier on A's machinery from GF2's derived tables | GF2, A | GATED | — |

### GF2 engineering (the base; each row DONE when its gate is green)

| # | task | status |
|---|---|---|
| E1–E3 | q8-mps: relative residual gate; χ-ladder in one process; MPO sparsity and threading | DONE; N = 8 χ = 40 point 730 s → 227 s |
| E4–E6 | static colour sources (GF2b); excited states by penalty; two flavours (GF2c) | NEXT, in that order |
| E7 | U(1)³-symmetric MPS blocks on the colour lanes | INSTRUMENT DONE; G0‴ met on all six N = 8 sectors at rank-derived χ; the volume ladder closed as prior art | GF2A_QCD2_RESULTS.md |
| E8 | general local dimension in q8-mps sites | after E7 |
| E9 | GPU sigma for FCI sectors | SUPERSEDED by E11 |
| E10 | the 3D finite-group box | after E8 (GF2 proper) |
| E11 | The k-lane determinant solver: one string per conserved integer lane, host and device bit-identical; the whole chemistry engine is the k = 2 case | DONE 2026-09-02; N = 8 B = 0 (343k dets) resident device sigma 2.4 ms | `lanes.rs`, `lanes_sigma.cu` |
| E12 | Davidson subspace bound as a priced parameter; Gram matrix cached | DONE 2026-09-02 |
| A1 | Structural audit of the lane stack, 17 findings applied | DONE 2026-09-02 |
| E13 | The Davidson as row programs on a vector space under ONE reduction law, device-resident | DONE 2026-09-02; N = 8 B = 0: 38.6 s → 1.2 s resident | `vecspace.rs`, `vec.cu` |
| E14 | The MPS arm on the vector space: block-sparse two-site contraction (112× one matvec), change instrumentation, re-seeding and White's mixing, the exact and two-site variances, the device kernel (4.8× host, bit-identical) | DONE 2026-09-03; the labelled arm the water triple runs on | GF2A_QCD2_RESULTS.md (E14) |

## The zoom ladder — the waterbench to the nucleus, then below

*The nucleus is the deepest OBJECT the bench shows — mass, charge, spin, its quantum spread —
carried on its conserved totals as ACUITY-B carries a composite outside the acuity. The
nucleus's interior is the fold below the atom (E7 → E8 → E10, GF2), fenced at the nucleus rung
with that owner and exit, never suppressed.*

| # | node | deps | status | record |
|---|---|---|---|---|
| W | The waterbench zoom ladder: cube → fluid element → H-bond network → molecular → atom → nucleus → the fold, each band LIVE from the artifact or FENCED by name; the two-box law and the fluid-zoom law on screen; the filmstrip as the fluidity check; the atom band drawn at the molecular solve's own density | ACUITY-B, the wasm lane engine | UP; smoke 431/431; three bands fenced on node G, the interior on GF2; the fine bands' provenance on their cards (`OBJECT.md` "The surface, audited") | WORKBENCH_FSD.md §9c, §11 |
| W2 | The gauge vacuum in the browser: the Schwinger meson and a QCD₂ baryon sector at exact sizes, derived in front of the viewer | W, E7 | after W | — |
| L6 | Chiral leptons in 3+1D: the Nielsen–Ninomiya wall named; routes to be PRICED before any freeze | W2, E10 | OPEN · no claim carried; the electron is an input | LEPTON_LADDER.md |

## Standing laws

**The fence law (operator, 2026-09-01).** Any fence is a bug waiting to be fixed once the GPU
solve is the path. A fence without an owner and an exit is suppression; refusing loudly stays
a feature, but a refusal is a DEBT with a fix path, never architecture. The ledger is
FENCES.md, classed PHYSICS-HONESTY / COMPUTE-PRICED / MODEL-FENCE with owner and exit; the
COMPUTE-PRICED class's exit is node F.

**The law this file lives under.** A node with no receipt-gate is a wish. No timelines: size is
compute × scope, and the only ordering is dependency and downstream value. When a node lands,
its row gains the record's location in the same commit.

**Fence triage owed:** two CRATE_ALLOW entries (q-seam, sphere-demo) and two unowned model
fences (I-5, FENCES.md M2) await the lead's next triage.
