# The Reality Workbench — pure-dependency build graph

*No calendar. Nodes order by OPTIMIZATION VALUE: what each unblocks
downstream. An arrow is a dependency; a node is DONE when its receipt-gate
is green, never before. Statuses: LAUNCHED (lane named) · READY (deps met,
unlaunched) · GATED (deps pending). In-flight prerequisites from the water
campaign are listed at the bottom; they feed this graph but predate it.*

## The graph

```
F  GPU production dispatch ──────────────┐  (multiplies every compute node below)
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
| F | GPU production dispatch: device-class-declared GPU sigma wired into the leased table generators; refuse mixed classes | gpu-prod's landed crates (done) | LAUNCHED · gpu-prod | same-table bit-identity WITHIN class; device class in every manifest; throughput registered as the ROUND-TRIP quantity with between-invocation spread |
| A | Species-generic MBE: holon-chem's 3/4-body machinery generic over Z-tuples; OHHH becomes the first instance of a general shape | none | LAUNCHED · mbe-generic | OHHH results BIT-IDENTICAL through the generic path (staked geometries + full suite); api items classified; sim.rs untouched (migration is its own later node) |
| B1 | Long-range residual audit: measure what cutoff-locality discards, per scene class, against a pre-staked negligibility criterion | parked trajectories (exist) | LAUNCHED · longrange-audit | prereg ADMITTED by gate 9c's auditor; measured residual table; verdict per scene class, VOID-not-KILL on budget |
| B2 | Ewald-class long-range subsystem | B1 verdict; mandatory if C ships ionic scenes | GATED | per-law gates incl. planted violations; energy ledger closed under the new term |
| C | Charged-fragments core: charge at the solver seam (electron-count assignment, spin-sector rule stated), H3O+/OH- certified single-points, census-charge staking doc | none for the core; A for generic ion TABLES | LAUNCHED · ion-core | ion energies with variational sanity vs fragments; refusals for unstated charge; the staking doc for what remains |
| D | Periodic-table availability: probe EVERY registered species — FCI-reachable (measured det count) / MPS-route / REFUSED with the reason (relativistic fence stated past staked Z; missing basis named) | none | LAUNCHED · bank-fences | a GENERATED availability table (probed, not asserted) + refusal tests firing per class |
| H | dE5 truncation audit: sample compact 5-clusters from real trajectories; E_FCI(5) − MBE4(5) distribution vs declared uncertainty | live dE4 path (done) | **DONE** · de5-audit | receipt: `conformance/water_observatory/DE5_RESULTS.md`. Prereg ADMITTED and committed alone before the instrument; 24 live configs (bar was 20), 0 VOID; **BRANCH (b) — DOES NOT TERMINATE**, worst \|dE5\| 7.86e-2 Ha = 1,572x the 5e-5 declared uncertainty, 24/24 over bound. Scope: planar, STO-3G, `O2H3` only. Strict reading of the audit's own frozen G2 is (d) VOID and is published beside it |
| E | NQE in dynamics: ring-polymer propagation coupled into Sim, not merely the C1 carrier | C1 (done); sim.rs landing-wave quiet; A's carrier plumbing | GATED | per-law gates under RPMD; P=1 bit-identical to classical Sim (the C1 pattern, in-engine) |
| MPS | DMRG cluster seam: exact-cluster solves for compact cores over MBE far-field | ~~H verdict~~ **DISCHARGED — H fired the seam requirement**; bulk-tier canonical brick (tower-complete, owed) — STILL GATED ON THIS ONE | GATED (one dep left) | seam defect budget staked and measured; crystal-tier referee inheritance stated. H supplies the measured size of what four-body truncation discards on compact `O2H3`: up to 7.86e-2 Ha, 2.83x the whole dE4 rung |
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
