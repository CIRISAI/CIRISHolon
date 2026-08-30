# Pre-registration — SATURATION-3: multi-element valence, engineered first, then fast

*Frozen 2026-08-30, committed ALONE. Two product systems in one campaign:
the complete H/Cl system (four triple types, all hole-count cheap) and
the completion of H/O (adding (O,O,H), with (O,O,O)'s necessity DECIDED
BY MEASUREMENT rather than assumed). The lead's order fixes the gate
order: ENGINEERING FIRST — the hardest staked combos proven trivial as
predicted, the generation sharded on the mesh under the machine-checked
merge law, GPU adoption measured not presumed — then the physics at
speed. The determinant counts below are the campaign's own predictions:
(H,H,Cl) 605, (H,Cl,Cl) 3,249, (Cl,Cl,Cl) 9,477, (O,O,H) 9,075,
(O,O,O) 207,025 — hole-count arithmetic committed here before any
solve, so G0 tests the arithmetic and not a memory of it. P1 of
MIXTURES-1 measured the fence composition that orders the physics:
48 of 52 firings were (O,O,H); 4 were (O,O,O).*

misfits: contacts M-UNTESTED-GAP (G0 exists so no cost claim rests on
interpolation — every staked combo is measured at its own worst
geometry), M-SORTS-NOT-SEPARATES and M-EXIT-DISCRIMINATOR (every solve
in every table records exit reason and iterations; a stagnated solve is
never published as converged), M-BUDGET-LAUNDER (budget exhaustion VOIDs
a node loudly, never scores), M-VACUOUS-SUCCESS (every generator and
verifier asserts its work count), M-CACHE-KIND (node records carry kind
and certification, counters count the certified shape),
M-FOREIGN-DOMAIN-CORROBORATION (GPU/mesh gates exercise the actual
table path, never a toy), M-MAX-OVER-SUCCESSES (any derived admission
bound is a wall-minus-one, never a max over successes), M-TAG-AS-PROPERTY
(species enter every formula by Z and basis, never by name),
M-PARITY-PROTECT (the spin audit carries per geometry; multiplicity
asserted where the gap resolves, parity 2S vs electron count enforced,
degeneracy reported), M-STALE-INSTRUMENT (instruments, tables, results
committed together), M-PLANT-OBS and M-PLANT-SECTOR (plants below),
M-NULL-MISSTAKE (branch (b) outcomes staked at full prominence),
M-HOMOG, M-FINAL-VIEW-COLLISIONS, M-IMPORT-EXECUTES, M-GAUGE-LAUNDER,
M-LOOP-BLIND, M-BARE-CHARGE, M-COND-PROBE, M-ELECTRIC-BASIS,
M-RING-MIXING, M-GAUGE-UNIFORM-MOMENTUM, M-KINEMATIC-NONLOCAL,
M-NONBIJECTIVE-STEP, M-FIXED-POINT-TRAJECTORY, M-PROBE-EIGENSTATE,
M-VOLUME-SCALE (not otherwise contacted).

## Scope, stated before the gates

All-electron STO-3G FCI throughout — NO frozen cores (a smaller model is
a different model). Symmetry per table is the table's own: S3 for
(Cl,Cl,Cl) and (O,O,O); the two-identical-one-distinct axis ONLY for
(H,H,Cl), (H,Cl,Cl), (O,O,H) — a wrongly full-symmetrised mixed table
must fire against the referee (the OHH lesson). Domains per table derive
from each species pair's own curve (A1-style second-smallest-side logic
with the pair's measured tail; the quadrupolar-tail warning from
SATURATION-2's hand-off applies to every (O,O,*) shell reading). Grids
are SIZED BY THE GATE (each table's own held-out kill), never inherited.
The angle-axis convergence anomaly from SATURATION-2's hand-off (5x per
doubling where C1 cubic owes 16x) is a STAKED INVESTIGATION at design
time: the builder must either explain it or show the chosen coordinate
beats it before committing any grid.

## Gates — in the lead's ordered sequence

- **G0 — THE HARDEST COMBOS ARE TRIVIAL AS PREDICTED (engineering, first,
  blocking)**: for every staked triple type, at its WORST compact staked
  geometry: determinant count equals the committed arithmetic above
  EXACTLY, and measured f64 solve time falls in the predicted cost class
  ((O,O,O) <= 5 minutes/point cold; every other type <= 5 seconds/point).
  Exit reason and iterations recorded per solve. Kill: any staked combo
  over 10x its class re-scopes the campaign BEFORE tables start.
  witness: none (measured)
- **G1 — MESH GENERATION UNDER THE MERGE LAW**: table generation sharded
  across workers; the assembled table is BIT-IDENTICAL across shard
  counts (1, 4, N) per the mesh discipline, and a deliberately corrupted
  shard is CONVICTED by the merge digest (the merge law's existing
  machinery, exercised on THIS path — never a toy). The warm-start
  generator improvement is measured here: cold vs warm Davidson
  iterations along a locality sweep, speedup reported, and the WARM
  result bit-identical to cold at every node (a warm start may change
  the path, never the answer; plant: a wrong warm start must still
  converge to the identical energy or the node VOIDs).
  witness: shardedFold_invariant, digest_convicts (the merge law's
  theorems; the runs are measured)
- **G2 — GPU ADOPTION, MEASURED NOT PRESUMED**: the dominant kernel
  (sparse H matvec at the (O,O,O) scale) benchmarked GPU vs CPU on the
  actual table path. Adopted ONLY if it wins with a determinism gate
  stated (fixed reduction order, or CPU-vs-GPU agreement bounded and
  declared per node); refusal is a RESULT (VOID with the measurement,
  not a failure) if the kernels do not fit these sizes. witness: none
  (measured; adoption gated)
- **R1 — referees, tiered by the ELEMENTS-3 rule**: 50-digit referee for
  every table whose point cost is <= 3e4 determinants ((H,H,Cl),
  (H,Cl,Cl), (Cl,Cl,Cl), (O,O,H)) on staked result-blind geometry sets;
  (O,O,O) carries f64 dual-route agreement at every node plus the
  fast-path-vs-general-route gate at 1e-12. Spin audit per geometry.
  witness: none (measured)
- **T1 — held-out fidelity per table**: 256 staked-seed held-out
  geometries per table, max error REPORTED, kill 1e-3 Ha, nonzero
  required; T2 boundary systematic <= 1e-5 Ha per table on its own
  derived domain. witness: none (measured)
- **P1-HCl — THE FIRST PRODUCT**: frozen quench protocol (committed on
  controls before the mixed arm runs; 8 H + 8 Cl, 8 seeds, thermostat;
  H-only and Cl-only controls), all four triple types live. Branch (a):
  HCl is the modal molecule with zero free atoms in >= 6 of 8. Branch
  (b): reported and investigated at full prominence. VOID on control
  failure. witness: none (measured)
- **P2-H2O — THE DECIDING PRODUCT**: the SATURATION-2 quench re-run
  under its SAME frozen protocol with (O,O,H) added (three of four
  types live), the (O,O,O) fence now counted alone. Branch (a): H2O is
  the modal O-molecule — and (O,O,O) is thereby MEASURED non-blocking,
  demoting to a named successor (ozone chemistry) without ever being
  built. Branch (b): if the census shows O-O aggregation persisting
  with the (O,O,O) fence dominant, the (O,O,O) table is built (G0
  already priced it) and P2 re-runs as a labelled second arm. Either
  way the necessity question is answered by the fence counter, not by
  assumption. witness: none (measured; the decision tree is the prereg's)
- **C1 — conservation per scene, per law**, with every new table's
  stiffness in the envelope, exit-reason-aware refusals (a scene whose
  atoms did not move cannot pass), per-species masses. witness: none
  (measured)

## plants (carrier and sector per M-PLANT-SECTOR)

Each plant's carrier is asserted nonzero in the sector the plant acts on
before the plant is scored; a plant on an empty sector VOIDs.

- **(i) swapped-species-table**: serving a triple the wrong species'
  table must fire T1-scale errors by orders (carrier: the energy shift).
- **(ii) the symmetry-axis plant, both directions**: exact symmetry on
  the identical-pair axis (bit-level), AND a wrongly FULL-symmetrised
  mixed table must fire against the referee (carrier: both deviations).
- **(iii) the warm-start plant**: a deliberately wrong warm start must
  yield the bit-identical converged energy or VOID the node — never a
  silently different table entry (carrier: the demonstrated firing).
- **(iv) the shard plant**: one corrupted shard must be convicted by the
  merge digest with zero false positives on clean shards (carrier: the
  conviction).

## Meaning

All gates => "multi-element valence emerges at speed: chlorine chemistry
complete, water assembling from a gas, every table referee-pinned, the
hardest solve proven cheap before it was trusted, the generation sharded
under a machine-checked merge law, GPU adoption a measurement, and the
one expensive table built only if the fence counter says the physics
needs it." NOT claimed: (O,O,O) unless built and gated; ozone chemistry;
any mixed system beyond H/Cl and H/O; quantitative thermochemistry
against nature; GPU speedups not measured on this path.
