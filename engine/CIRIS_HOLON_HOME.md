# Lifting the engine into CIRISHolon

Status: extraction decision and handoff note.
Date: 2026-08-22.
License: **AGPL-3.0-or-later**, with no permissive fallback.

The governing WHY is [`MISSION.md`](MISSION.md). This file records the extraction mechanics.

## Name and object vocabulary

The dedicated repository is **CIRISHolon**. `CIRISReal` is rejected: the engine carries
certified approximations and cannot honestly imply one privileged or fully reproduced
reality. `HolonDynamics` is rejected because it turns the primitive into a mouthful.

The public vocabulary is:

| term | meaning |
|---|---|
| `Holon` | the sole recursive primitive, simultaneously a whole and a part |
| `Holarchy` | a composition of holons; a structural relation, not a second entity kind |
| `root holon` | the chosen root of one evaluation; a role, not a special type |
| `realization` | a domain chart/evaluator over the common maximal choice object, not a new ontology |
| `grain constant` (`g0`) | the terminal resolution declared by the active model |
| `frontier` | the non-overlapping resident holons currently representing the root |
| `Constitutional Mesh` | CEWP's current root-scale holon, itself able to join a larger federation |

There is therefore no `MaximalObject` type and no absolute maximal holon. A quark, ball,
person, NPC, storyline, agent, fabric occurrence, and Constitutional Mesh use the same
primitive. “Maximal” is always relative to the root selected for an encounter.

`holon` and CIRISEdge's `holonomic` are complementary, not synonyms. A holon is the
whole/part unit. Holonomic federation is the path-independent reconstitution property by
which a federation can recover the same view from any sufficient signed fragment.

## Why the license is aggressively AGPL

CIRISEdge's `MISSION.md` makes the controlling argument: M-1 requires sustainable adaptive
coherence, independent participants cannot be collapsed through a hidden broker, every
principal is subject to the same Recursive Golden Rule, and the mesh may itself be a moral
subject. Section 10 therefore treats AGPL as a mission mechanism for **legibility under
audit**, not as branding.

CIRISHolon is an equally sensitive control point. It decides what is represented, what is
left latent, when a boundary is refined, what conservation residual is tolerated, and when
an approximation is presented as macro-equivalent. A private network fork could invisibly
coarsen one participant, waive a conservation gate, or privilege one root. Network copyleft
keeps those changes inspectable by the people and holons whose outcomes they shape.

Repository policy:

- every crate and package declares `AGPL-3.0-or-later` and ships the full license;
- there is no proprietary dual-license escape hatch or permissive core;
- build instructions, conformance vectors, benchmark scenes, error budgets, and certificate
  formats ship with the corresponding source;
- Rust, WIT, WASM, adapters, and server-side modifications remain auditable across the
  network boundary;
- the AGPL covers the program and its derivatives. Scene data, model weights, and unrelated
  services are not relabeled by assertion; integration boundaries are documented precisely.

## Dedicated repository shape

The extraction target is:

```text
CIRISHolon/
  crates/ciris-holon-core/       no_std kernel, holons, REG+, refinement, dynamics
  crates/ciris-holon-component/  zero-import WIT/component control boundary
  crates/ciris-holon-demo/       rendered Rust/WASM reference pipeline
  crates/engine-compare/         Rapier and analytic parity controls
  docs/                           mission, architecture, prior art, certificates
```

The core must not depend on CIRISGame, CEWPOS, CIRISClient, or CIRISEdge. Consumers depend
inward on the engine:

- CIRISGame and native clients use the Rust API and retain arenas/frontiers across frames;
- `ciris.ai/game` uses browser WASM through the same deterministic kernel;
- CEWPOS uses the zero-import WebAssembly Component as a sandbox/control-plane boundary;
- CIRISClient uses the typed adapter appropriate to its host;
- CIRISEdge may carry signed CEG envelopes and certificates but remains “reach, not meaning”;
- H3ERE2-G uses CIRISHolon as the structural middle described in `H3ERE2_RESTART.md`.

CEG/CEWP adapters belong in separate adapter crates or features. The core owns dynamics and
certificate semantics; CIRISOntology remains the proof/definition authority for the maximal
choice object and imported REG+ invariants until an explicit authority transfer is recorded.
Mechanical, quantum, agentic, and narrative integrations must be realizations of this common
object. They may add observation and certification machinery, but not competing entity types
or unaccounted state coordinates.

## History-preserving lift

1. Merge the complete `sim_engine` slice here and tag the extraction point.
2. Create the public `CIRISAI/CIRISHolon` repository with AGPL-3.0-or-later and this note.
3. Preserve history with `git filter-repo --path sim_engine/ --path-rename sim_engine/:`.
4. Rename packages to the target crate names above; retain a migration map in both repos.
5. Make CIRISHolon canonical for runtime code and certificates. Leave a pointer and pinned
   dependency in CIRISOntology instead of two editable copies.
6. Update consumers to a tagged CIRISHolon revision, one at a time, with their own parity
   and rendered-pipeline gates.
7. Archive releases in durable public indexes so the engineering disclosure remains citable.

## Extraction gates

The lift is ready only when all of these are repeatable from source:

- native and `wasm32-unknown-unknown`/`wasm32-wasip1` core builds;
- zero-import component build and validation;
- allocator-free and runtime-sized storage-policy parity;
- rendered particle-on-sphere screenshot from generated Rust frame data;
- analytic and Rapier controls with a solution-parity gate, never throughput-only comparison;
- adaptive contact certificates across the full test matrix, including grain-floor and
  refinement-unavailable outcomes rather than only successful cases;
- formatted, warning-free, deterministic tests and documented remaining proof obligations.

Mission line: **One recursively closed dynamical holon, from grain to Constitutional Mesh,
whose approximations and governing boundaries remain auditable by everyone they affect.**
