# The Reality Workbench — pure-dependency build graph

*No calendar. Nodes order by OPTIMIZATION VALUE: what each unblocks
downstream. An arrow is a dependency; a node is DONE when its receipt-gate
is green, never before. Statuses: LAUNCHED (lane named) · READY (deps met,
unlaunched) · GATED (deps pending). In-flight prerequisites from the water
campaign are listed at the bottom; they feed this graph but predate it.*

## The graph

```
F  GPU production dispatch ──────────────┐  (multiplies SOLVE-BOUND work where sigma dominates;
                                         │   MEASURED CAVEAT: table generation is Amdahl-capped ~3% —
                                         │   sigma is 4% of a Davidson iteration at 207k dets, and the
                                         │   earlier VRAM story was retracted by measurement: 30 workers fit)
                                         ├─▶ dE4 table regen · dE5 audit · triple-point sweep · cluster solves
A  Species-generic MBE ──────┬─▶ C2 ion tables ─▶ Grotthuss/superionic honesty
   (Z prices, never branches)├─▶ full-table chemistry beyond water
                             └─▶ E  NQE-in-dynamics (shares carrier plumbing; also gated
                                    on sim.rs landing-wave quiet — 4 lanes in that file)
B1 Long-range residual AUDIT ─▶ B2 Ewald subsystem (built ONLY if B1's verdict demands;
   (measure what the cutoff        C makes it near-certain for ionic scenes)
    discards, per scene class)
C  Charged fragments core ───┬─▶ C2 ion pair/triple tables (needs A's generic machinery)
   (solver-seam charge,      └─▶ census charge bookkeeping ─▶ Grotthuss exhibit
    H3O+/OH- certified)
D  Periodic-table availability ─▶ heavy-element scenes · mixtures demos
   (probe every species: FCI/MPS/REFUSED-with-name; relativistic fence STATED)
H  dE5 truncation audit ─────▶ MEASURED: the ladder does NOT terminate at four
   (measure, never build)       on compact planar O2H3 → the seam requirement FIRED
MPS cluster seam (DMRG for compact cores, MBE far-field, seam defect-audited)
   ◀── gated on: H's verdict (DISCHARGED) + bulk-MPS tier formalization (canonical brick, tower-owed)
G  Upward tier closures (H-bond network ⊂ molecules ⊂ quantum; census as referee)
   ◀── gated on: OOO cert + dE4 table + B1 verdict  (the ontology's own ladder)
```

## Nodes, receipts, owners

| # | node | deps | status · owner | done means (the receipt) |
|---|---|---|---|---|
| F | GPU production dispatch: device-class-declared GPU sigma wired into the leased table generators; refuse mixed classes | gpu-prod's landed crates (done) | LAUNCHED · gpu-prod | same-table bit-identity WITHIN class; device class in every manifest; throughput registered as the ROUND-TRIP quantity with between-invocation spread  **(lane silent since the 2026-09-02 limit wave; status-checked, respawn pending answer — node F is now also the MPS seam's named cap exit)** |
| A | Species-generic MBE: holon-chem's 3/4-body machinery generic over Z-tuples; OHHH becomes the first instance of a general shape | none | **DONE — receipts: bit-identity 10/10 on staked geometries through the generic path, plants fired at one bit (landed 4966658); full holon-chem suite green at HEAD — 26 binaries pass including mbe_generic_identity and the re-banked elements3_dimers; the ONE failure is the pre-existing water.rs committed-table test, cause pinned (table banked before c9b0cbc's budget raise), repair running (s2_build regeneration, provenance-pinned to e3d7eb6); api items classified; sim.rs untouched (migration is its own later node)** | landed — unblocks C's generic ion TABLES |
| B1+B1b | Long-range residual audit and its successor | parked trajectories | **DONE — B1b receipts added: B1B_PREREG.md ADMITTED pre-instrument; work-unit price VINDICATED (passed 725.7 vs floor 100 on a run wall-clock would have refused again; 1.07x spread where seconds spread 2.74x); B1 reproduces bit-identically under the changed instrument; the mixed verdict is B2's row. DONE — receipts: LONGRANGE_PREREG.md ADMITTED pre-instrument; CLASS-H NEGLIGIBLE branch (a) at 2.6e-11 of criterion over 160k frames, crossing bracketed [6.0, 9.0) bohr; three classes VOID honestly (mixed class on the audit's own price gate, un-re-argued). Finding: the engine currently discards NOTHING (no cutoff set — the question is counterfactual). Successor freeze B1b NAMED: work-unit pricing, E_switch estimator, drift denominator — the mixed/water class answers there** | landed |
| B2 | Long-range pair subsystem. **The row's old name said "Ewald-class"; the measurement says otherwise** and `B2_PREREG.md` §2 argues why from this engine's source: `Sim::compute_forces` carries no electrostatic term at all, so the conditionally convergent `r^-1` sum Ewald exists to evaluate is not present. B2 builds a split kernel whose far part is absolutely convergent on its own image lattice, with the licence (`p > d`) MEASURED and the `p <= d` case REFUSED by name with Ewald/PME as the exit | **FIRED BY MEASUREMENT (B1b banked): the water class is NON-NEGLIGIBLE — the O-O support reaches 20 bohr against the 15 bohr locality radius, the discard is real tabulated interaction (144/400 mixed rows vs 0/400 hydrogen), and NO safe radius exists in the box. Boundary runs through the seed set (3/8 over criterion vs incurred drift). Ionic r^-1 still unmeasured (node C's scenes neither shown nor excluded); N-scaling owed** | **LANDED · b2-ewald — receipts: `B2_PREREG.md` ADMITTED and committed one commit before `longrange.rs` existed; `B2_RESULTS.md` with `b2_frames.log` (160,000 frames, all 8 seeds admitted by digest), `b2_engine_full.log`, `b2_engine_hh.log`, `b2_refusals.log`, `b2_tests.log`. **B1b's discard was a RADIUS, not a tail**: G1 reads S/T = 9.9e8 at the deciding frame, so B2 does not claim to have solved a problem that was a radius. **G14 PASS 0/8** — the three seeds B1b failed at 1.898/1.574/2.496 come in at 0.0274/0.0178/0.0290. Three laws gated independently in BOTH the complete and the truncated configuration, P3 firing angular while momentum stays green. New angular-momentum ledger in `Sim`. G11 10/10 refusals. Suite 21 binaries exit 0, `t3_replay` included. **Four staked gates did NOT deliver a clean pass and are kept fired: G7** (coarsest step's own O(h^2) error), **G8** (a max-of-relative statistic saturated by an underflowing reference; the resolvable worst on the FULL set is 1.187e-7, 1.19x OVER the staked bar — 3.0e-8 is the H-H arm only), **G4 VOID under V2** (power certificate: it resolves 1e-2 Ha against a staked 1e-6 plant), and the periodic arm VOID on the hydrogen set by construction. **N-scaling MEASURED: exponent 2.122 — the far sector is O(N^2) and buys no O(N) far route**| per-law gates incl. planted violations; energy ledger closed under the new term |
| C | Charged-fragments core: charge at the solver seam (electron-count assignment, spin-sector rule stated), H3O+/OH- certified single-points, census-charge staking doc — plus the TABLE half (C2) | none for the core; A for generic ion TABLES | **CORE DONE · ion-core; TABLES DONE · ion-tables** — table receipts: freeze `conformance/atomworld/ION_TABLES_PREREG.md` ADMITTED and committed alone before the generator existed, results `conformance/atomworld/ION_TABLES_RESULTS.md`, code `holon-chem/src/ion_table.rs`, 11 gates green with 5 plants firing (`tests/ion_tables.rs`), bank `docs/atoms/tables/ions/`. Charge and spin sector are IN the key and a row without them cannot be constructed; the neutral path is bit-unmoved (192 raw-bit comparisons, H2 and OH); H3O+ tabulated through the same generic door, anchored to node C's pinned proton affinity at 8.3e-14 Ha. **Two findings:** the H3O+ single-bond stretch dissociates to H2O+ + H, 0.1597 Ha BELOW the naive H2O + H+ channel (a lane taking the obvious channel publishes a well that deep in error), and (H3O+ . H2O) — I-2's headline ionic pair — is priced out at 9,018,009 determinants over 15 orbitals, which makes it a COMPUTE-PRICED fence for node F, not a modelling gap. OH- tables REFUSED under fence I-5, which stays fired | landed — the ionic three-body surfaces remain, and they are gated on I-1's charge-assignment rule, not on machinery |
| D | Periodic-table availability + the fence ledger | none | **DONE — receipts: FENCES.md (49 fences: 19 physics-honesty / 20 compute-priced / 10 model-fence, owner+exit per row) and conformance/atomworld/PERIODIC_AVAILABILITY.md (54 species probed: 27 FCI-direct, 27 MPS-route, 0 unavailable; 18 relativity-fenced; generator gate green, mutation-tested)** | landed |
| H | dE5 truncation audit: sample compact 5-clusters from real trajectories; E_FCI(5) − MBE4(5) distribution vs declared uncertainty | live dE4 path (done) | **DONE** · de5-audit | receipt: `conformance/water_observatory/DE5_RESULTS.md`. Prereg ADMITTED and committed alone before the instrument; 24 live configs (bar was 20), 0 VOID; **BRANCH (b) — DOES NOT TERMINATE**, worst \|dE5\| 7.86e-2 Ha = 1,572x the 5e-5 declared uncertainty, 24/24 over bound. Scope: planar, STO-3G, `O2H3` only. Strict reading of the audit's own frozen G2 is (d) VOID and is published beside it |
| E | NQE in dynamics: ring-polymer propagation coupled into Sim, not merely the C1 carrier | C1 (done); sim.rs quiet; A's carrier plumbing (done) — **DEPS MET** | **LAUNCHED · node-e-rpmd (2026-09-02, operator's resolve order)** — bottom first: P=1 BIT-IDENTICAL to the classical Sim on staked scenes with a planted one-bit divergence the gate must see; then per-law gates under RPMD (one per conserved quantity, plants verified firing); bead count P joins the arithmetic-regime identity; the physics-honesty line stated (equilibrium strength yes, real-time coherence no, fences with exits). Named consumer: cryo's H2 arm re-run with quantum nuclei — the classical-nuclei fence's exit | per-law gates under RPMD; P=1 bit-identical to classical Sim (the C1 pattern, in-engine) |
| MPS | DMRG cluster seam: exact solves of compact clusters over an MBE far field. **FIRED BY MEASUREMENT twice over** — dE5 (24/24 over bound, worst 1,572x, the ladder does not terminate at four) and cryo arm 3 (the fragment-local expansion never converges for compressed H at any density). FINDING F-5 is the BOTTOM: the MPS-ROUTE band has no reachable member (a compact O2H3 is 13 orbitals / 17 electrons — past MPS_MAX_ORBITALS = 9 AND past FCI_DET_MAX), so the seam must MOVE THE CAP or the routing, never just add itself | ~~H verdict~~ DISCHARGED — H fired the seam; bulk-tier canonical brick (owed, part of this node) | **LAUNCHED · mps-seam (2026-09-02, operator's resolve order)** — bottoms-up: (1) re-measure the orbital cap against the CURRENT MPO construction per mps_ladder.rs's own protocol, improving the MPO build if the wall sits below 13; (2) the canonical brick with canonicality as a CHECKED invariant (the convergence-on-stagnation lesson); (3) the seam assembly with its defect budget staked then measured, dE5's own 24 configs as the acceptance referee; (4) crystal-tier referee inheritance stated. Pre-committed branch: cap-cannot-move converts the node to a measured fence with the node-F exit | seam defect budget staked and measured; crystal-tier referee inheritance stated |
| G | Upward closures: network and fluid tiers certified as Closed views, census as referee at each rung | OOO cert; dE4 table; B1 | **rung 1 MEASURED AND NOT CERTIFIED · rung1-network — branch (D): across 70 readings on seven frozen charts, in-budget and dynamic are EXACTLY DISJOINT (36 in budget all VOID by anti-vacuity; 32 dynamic none in budget; 0 both). The boundary is ALIGNMENT, not aggregation: molecules present and within H-bond distance 84-99.8% of frames, yet inter-molecular H-bonds in 0-18 of 20,000 frames — the staked expectation was wrong and is reported as the finding. G-ID cross-validated the census's own view to four decimals on nine banked readings. Doors: (a) refused with evidence (an aggregate would read 'essentially closed' from vacuity), (b) named as a PAIR (defect + reading-changes + distinct; serve C6/MOL-PART first), (c) not named (no measured cadence). Receipts: RUNG1_PREREG.md ADMITTED pre-instrument, RUNG1_RESULTS.md, holon-lens/src/network.rs + tests/rung1_plants.rs**; **rung 2 MEASURED AND NOT CERTIFIED · rung2-continuum — branch (d) of its own ADMITTED freeze: the carrier is inadmissible BY MEASUREMENT (occupancy/transport scissor — coarse cells never transport, the only transporting grid holds 0.5 atoms/cell at sigma/mean 1.55-1.88 vs a 0.10 bar; G2 0/75). Measured anyway per the pre-committed branch: 42/42 live cells NotClosed. The momentum chart beats its coherence-destroying control by +0.598 (spatial coherence at 5.8 bohr is real and the instrument sees it); occupancy separation is wrong-signed. F1 fired and stays dead (D_A is not a cross-rung comparator; within-rung G7 is the valid form). The fluid band's fence now carries NUMBERS (5.95e6 atoms for 1 µm, 16-atom artifact cap, the scissor) with exit UNDETERMINED per M-UNTESTED-GAP; successor routes named (>=400-atom carrier + format v2, or node LG standalone — never composed). Receipts: RUNG2_PREREG.md ADMITTED, RUNG2_RESULTS.md, holon-lens/src/field.rs + examples/rung2.rs** | each rung's closure certificate in the census two-leg form, with controls |
| LG | Continuum-native fluid tier: certify the FHP-6/FCHC-24 lattice-gas dynamics as its OWN closed tier on the existing machinery (Core/Lattice.lean's 53 sectors 44/7/2, Core/ModeChart.lean's charts, ciris-sim-core/regplus.rs, MESH_DESIGN.md §2.1's FCHC-24 warrant). FIRST LAW (rung 2's warning, adopted verbatim): this tier is NOT a view of the molecular dynamics and must NEVER be composed through viewClosed_comp as if it were — the molecular-to-lattice SEAM is a separate claim with its own measured status, informed by rung 2's measured exit | none for the tier (machinery exists); rung 2's exit informs the seam | LAUNCHED · lattice-tier | prereg ADMITTED; per-conservation-law gates with planted violations on its OWN trajectories; closure certificate in the census two-leg form at its own charts; the 53/44/7/2 sector reproduction as instrument control; the seam claim stated with its own honest status |

## In-flight prerequisites feeding this graph (owned elsewhere)

OOO tabulation → certification (resume-free check attached) → P2 fence 0.
dE4 table with checkpoint sink (de4-table; kill-and-relaunch ruled).
Browser surface evaluator (mesh) → tables served → dE4 door.
3D-only page, gravity vector, glow, panels (workbench-engine, FSD-W2).
Ice seeders · triple-point prereg · C1-browser probe (subagent worktrees).

## The fence law — operator's standing order, 2026-09-01

**Any fence is a bug waiting to be fixed once the GPU solve is the path.**
A fence without an owner and an exit is suppression (the CRATE_ALLOW ruling,
generalized to every refusal in the system). Refusing loudly stays a feature
— but a refusal is a DEBT with a fix path, never architecture. The ledger of
every fence, classed PHYSICS-HONESTY / COMPUTE-PRICED / MODEL-FENCE with
owner and exit per row, is node D's first deliverable (FENCES.md); the
COMPUTE-PRICED class's exit is node F, which is why F multiplies the graph.

## The law this file lives under

A node with no receipt-gate is a wish (TIERS.md's own rule). No timelines:
size is compute × scope, and the graph's only ordering is dependency and
downstream value. When a node lands, its row gains the receipt's location;
when a gated node's deps clear, its status flips here in the same commit
that clears them.

**Fence triage (from FENCES.md F-4/F-8):** two CRATE_ALLOW entries (q-seam,
sphere-demo) and two unowned model fences (the diffuse-basis exit I-5, and
FENCES.md's M2) await an ownership decision at the lead's next triage — named
here so the suppression clock is visible.
