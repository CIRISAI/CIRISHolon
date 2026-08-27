# CIRISHolon defensive publication and prior-art map

Date of repository disclosure: 2026-08-22.
License: AGPL-3.0-or-later.

This document is a defensive publication, not a patent claim or legal opinion. The enabling
implementation is the adjacent Rust/WASM source, tests, examples, rendered frame bundle, and
solution-gated Rapier comparison.

## Disclosed construction

CIRISHolon combines these concrete mechanisms:

1. One recursive holon type represents particles, balls, persons, agents, narratives, and
   root-scale meshes through the same maximal choice object rather than disjoint entity
   classes or independent domain ontologies.
2. Every holon carries additive REG+ gross state and separate irreducible whole-state.
   Expanded children must compose the gross ledger exactly; no factoring claim is imposed
   on the whole-only state.
3. Latent decomposition and a resident, non-overlapping frontier avoid enumerating the full
   holarchy. Const-capacity and runtime-sized arenas execute the same certification math.
4. A realization supplies observables, normalized macro error, conservation residual, and a
   deterministic boundary-refinement ranking. The engine returns the coarsest frontier that
   meets the declared error budget, or an explicit grain-floor/refinement-unavailable result.
5. Mechanical contact demonstrates the gross/boundary split with exact aggregate impulse,
   deterministic boundary samples, selective contact-side refinement, and a certified macro
   error. Mechanical, quantum, narrative, and agentic realizations share the holon's choice,
   record, warrant, gross, and whole coordinates. A realization supplies an executable
   update/readout/certificate; it does not supply a separate metaphysics.
6. The kernel is `no_std`, deterministic, allocator-free by default, runtime-sized under
   `alloc`, compilable for native and WebAssembly, and exposed through an optional zero-import
   WebAssembly Component control boundary.
7. Comparisons against another solver are admitted only when the resulting trajectory agrees
   to the declared solution threshold. Throughput from divergent trajectories is withheld.
8. H3ERE2-G applies the same construction to a language interface: a small model perceives
   and renders; soft per-item state seeds a certified structural middle; real-vs-scrambled
   dynamics separates the construction from scaffold/compute effects.

## Prior-art families and boundary of contribution

The design is informed by, and does not claim invention of, multibody dynamics, multiscale
methods, adaptive mesh refinement, constraint solvers, graph coarsening, holonic systems,
entity-component systems, error estimators, or WebAssembly components individually. Relevant
public families include:

- adaptive mesh refinement and a posteriori error estimation;
- multigrid, algebraic multigrid, graph partitioning, and lumped/coarse dynamics;
- quadtree/octree, Barnes–Hut, fast multipole, and boundary-element methods;
- material point, discrete element, peridynamic, and position-based dynamics;
- holonic manufacturing/multi-agent architectures and recursive whole/part modeling;
- adaptive fidelity, level-of-detail, and surrogate/reduced-order simulation;
- deterministic lockstep game simulation and ECS storage;
- WebAssembly Interface Types and the Component Model;
- neuro-symbolic planning, constrained decoding, model-based control, and placebo/ablation
  evaluation of learned or structured reasoning systems.

The disclosed engineering boundary is their integration with CIRIS's REG+ gross/whole split,
root-relative holons, certificate-directed minimal grain, explicit failure at the grain floor,
multi-channel realizations, portable deterministic Rust/WASM storage parity, and an
apples-to-apples trajectory gate against a conventional engine.

## Reproduction entry points

- `CIRIS_HOLON_ENGINE.md` — executable architecture and measured results;
- `CIRIS_HOLON_HOME.md` — terminology, mission, license, and extraction contract;
- `SPHERE_RAPIER_BENCHMARK.md` — analytic and Rapier comparison discipline;
- `crates/ciris-sim-core/` — kernel, tests, examples, and benchmark;
- `crates/ciris-sim-component/` — WIT/component boundary;
- `crates/sphere-demo/` and `output/playwright/` — rendered end-to-end frame evidence;
- `H3ERE2_RESTART.md` — bounded application to Qwen3-0.6B response generation.
