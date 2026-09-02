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
H  dE5 truncation audit ─────▶ the ladder's termination CERTIFICATE, or the
   (measure, never build)       DMRG-cluster seam requirement (below)
MPS cluster seam (DMRG for compact cores, MBE far-field, seam defect-audited)
   ◀── gated on: H's verdict + bulk-MPS tier formalization (canonical brick, tower-owed)
G  Upward tier closures (H-bond network ⊂ molecules ⊂ quantum; census as referee)
   ◀── gated on: OOO cert + dE4 table + B1 verdict  (the ontology's own ladder)
```

## Nodes, receipts, owners

| # | node | deps | status · owner | done means (the receipt) |
|---|---|---|---|---|
| F | GPU production dispatch: device-class-declared GPU sigma wired into the leased table generators; refuse mixed classes | gpu-prod's landed crates (done) | LAUNCHED · gpu-prod | same-table bit-identity WITHIN class; device class in every manifest; throughput registered as the ROUND-TRIP quantity with between-invocation spread |
| A | Species-generic MBE: holon-chem's 3/4-body machinery generic over Z-tuples; OHHH becomes the first instance of a general shape | none | **DONE — receipts: bit-identity 10/10 on staked geometries through the generic path, plants fired at one bit (landed 4966658); full holon-chem suite green at HEAD — 26 binaries pass including mbe_generic_identity and the re-banked elements3_dimers; the ONE failure is the pre-existing water.rs committed-table test, cause pinned (table banked before c9b0cbc's budget raise), repair running (s2_build regeneration, provenance-pinned to e3d7eb6); api items classified; sim.rs untouched (migration is its own later node)** | landed — unblocks C's generic ion TABLES |
| B1+B1b | Long-range residual audit and its successor | parked trajectories | **DONE — B1b receipts added: B1B_PREREG.md ADMITTED pre-instrument; work-unit price VINDICATED (passed 725.7 vs floor 100 on a run wall-clock would have refused again; 1.07x spread where seconds spread 2.74x); B1 reproduces bit-identically under the changed instrument; the mixed verdict is B2's row. DONE — receipts: LONGRANGE_PREREG.md ADMITTED pre-instrument; CLASS-H NEGLIGIBLE branch (a) at 2.6e-11 of criterion over 160k frames, crossing bracketed [6.0, 9.0) bohr; three classes VOID honestly (mixed class on the audit's own price gate, un-re-argued). Finding: the engine currently discards NOTHING (no cutoff set — the question is counterfactual). Successor freeze B1b NAMED: work-unit pricing, E_switch estimator, drift denominator — the mixed/water class answers there** | landed |
| B2 | Long-range pair subsystem. **The row's old name said "Ewald-class"; the measurement says otherwise** and `B2_PREREG.md` §2 argues why from this engine's source: `Sim::compute_forces` carries no electrostatic term at all, so the conditionally convergent `r^-1` sum Ewald exists to evaluate is not present. B2 builds a split kernel whose far part is absolutely convergent on its own image lattice, with the licence (`p > d`) MEASURED and the `p <= d` case REFUSED by name with Ewald/PME as the exit | **FIRED BY MEASUREMENT (B1b banked): the water class is NON-NEGLIGIBLE — the O-O support reaches 20 bohr against the 15 bohr locality radius, the discard is real tabulated interaction (144/400 mixed rows vs 0/400 hydrogen), and NO safe radius exists in the box. Boundary runs through the seed set (3/8 over criterion vs incurred drift). Ionic r^-1 still unmeasured (node C's scenes neither shown nor excluded); N-scaling owed** | **LANDED · b2-ewald — receipts: `B2_PREREG.md` ADMITTED and committed one commit before `longrange.rs` existed; `B2_RESULTS.md` with `b2_frames.log` (160,000 frames, all 8 seeds admitted by digest), `b2_engine_full.log`, `b2_engine_hh.log`, `b2_refusals.log`, `b2_tests.log`. **B1b's discard was a RADIUS, not a tail**: G1 reads S/T = 9.9e8 at the deciding frame, so B2 does not claim to have solved a problem that was a radius. **G14 PASS 0/8** — the three seeds B1b failed at 1.898/1.574/2.496 come in at 0.0274/0.0178/0.0290. Three laws gated independently in BOTH the complete and the truncated configuration, P3 firing angular while momentum stays green. New angular-momentum ledger in `Sim`. G11 10/10 refusals. Suite 21 binaries exit 0, `t3_replay` included. **Four staked gates did NOT deliver a clean pass and are kept fired: G7** (coarsest step's own O(h^2) error), **G8** (a max-of-relative statistic saturated by an underflowing reference; the resolvable worst on the FULL set is 1.187e-7, 1.19x OVER the staked bar — 3.0e-8 is the H-H arm only), **G4 VOID under V2** (power certificate: it resolves 1e-2 Ha against a staked 1e-6 plant), and the periodic arm VOID on the hydrogen set by construction. **N-scaling MEASURED: exponent 2.122 — the far sector is O(N^2) and buys no O(N) far route**| per-law gates incl. planted violations; energy ledger closed under the new term |
| C | Charged-fragments core: charge at the solver seam (electron-count assignment, spin-sector rule stated), H3O+/OH- certified single-points, census-charge staking doc — plus the TABLE half (C2) | none for the core; A for generic ion TABLES | **CORE DONE · ion-core; TABLES DONE · ion-tables** — table receipts: freeze `conformance/atomworld/ION_TABLES_PREREG.md` ADMITTED and committed alone before the generator existed, results `conformance/atomworld/ION_TABLES_RESULTS.md`, code `holon-chem/src/ion_table.rs`, 11 gates green with 5 plants firing (`tests/ion_tables.rs`), bank `docs/atoms/tables/ions/`. Charge and spin sector are IN the key and a row without them cannot be constructed; the neutral path is bit-unmoved (192 raw-bit comparisons, H2 and OH); H3O+ tabulated through the same generic door, anchored to node C's pinned proton affinity at 8.3e-14 Ha. **Two findings:** the H3O+ single-bond stretch dissociates to H2O+ + H, 0.1597 Ha BELOW the naive H2O + H+ channel (a lane taking the obvious channel publishes a well that deep in error), and (H3O+ . H2O) — I-2's headline ionic pair — is priced out at 9,018,009 determinants over 15 orbitals, which makes it a COMPUTE-PRICED fence for node F, not a modelling gap. OH- tables REFUSED under fence I-5, which stays fired | landed — the ionic three-body surfaces remain, and they are gated on I-1's charge-assignment rule, not on machinery |
| D | Periodic-table availability + the fence ledger | none | **DONE — receipts: FENCES.md (49 fences: 19 physics-honesty / 20 compute-priced / 10 model-fence, owner+exit per row) and conformance/atomworld/PERIODIC_AVAILABILITY.md (54 species probed: 27 FCI-direct, 27 MPS-route, 0 unavailable; 18 relativity-fenced; generator gate green, mutation-tested)** | landed |
| H | dE5 truncation audit: sample compact 5-clusters from real trajectories; E_FCI(5) − MBE4(5) distribution vs declared uncertainty | live dE4 path (done) | LAUNCHED · de5-audit | prereg ADMITTED; measured distribution over ≥20 sampled configs; verdict: ladder terminates / seam required |
| E | NQE in dynamics: ring-polymer propagation coupled into Sim, not merely the C1 carrier | C1 (done); sim.rs landing-wave quiet; A's carrier plumbing | GATED | per-law gates under RPMD; P=1 bit-identical to classical Sim (the C1 pattern, in-engine) |
| MPS | DMRG cluster seam: exact-cluster solves for compact cores over MBE far-field. FINDING F-5 (FENCES.md): the MPS-ROUTE band currently has NO reachable member — every species past the FCI threshold is also past the hard cap, so 27 of 54 atoms have no automatic route; the seam work must move the cap or the routing, not just add the seam | H verdict; bulk-tier canonical brick (tower-complete, owed) | GATED | seam defect budget staked and measured; crystal-tier referee inheritance stated |
| G | Upward closures: network and fluid tiers certified as Closed views, census as referee at each rung | OOO cert; dE4 table; B1 | GATED | each rung's closure certificate in the census's two-leg form, with controls |

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
