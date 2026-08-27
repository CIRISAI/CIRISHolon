# CIRISHolon engine

Status: executable architectural slice, not a claim of complete multi-regime dynamics.
Date: 2026-08-22.

The governing WHY is [`MISSION.md`](MISSION.md).
This PR is the **Newton realization**. Einstein begins only after the classical trajectory,
contact, adaptive-grain, WASM, and rendered-pipeline gates below pass.

## Contract

There is one recursively closed holon type. A person, NPC, ball, storyline, and quark
are not different storage variants; they are the same recursively compositional holon.
Typed realizations are executable charts over that common maximal choice object, not
replacement ontologies.
`Holon` names the whole/part primitive; `root holon` names its role in one evaluation.
There is no absolute maximal holon. A root can become a child when a larger holarchy is
formed. In CEWP, the current root-scale realization is a **Constitutional Mesh**.

A holon contains:

- additive REG+ gross state;
- irreducible whole-state that is not required to factor through its children;
- zero or more simultaneous dynamical channels;
- a boundary marker;
- a grain diameter in multiples of `g0`;
- a decomposition state: terminal, latent, or resident.

## Representational closure

“Quark to storyline” is not a metaphor and does not mean one classical force law is
stretched across unrelated regimes. The closure is stronger and cleaner: every variation
of a holon is represented in the same maximal choice object already mechanized in
`Core/Generator.lean` and `Core/WrongKind.lean`:

- Priorities, Rules, Manner, Identity, Confidence, Facts, Circumstances, Process, Model,
  Structure, and Premises are the eleven artifact-local coordinates;
- Record is the relation between a holon and the frame from which its history can still be
  established;
- warrant/source is an orthogonal coordinate, not another entity class;
- the CEG/REG+ operations act on that common representation;
- recursive composition and whole-only state preserve what partial views cannot recover.

Mechanical, quantum, agentic, and narrative are therefore not sovereign ontologies with
unrelated state spaces. They are **realizations or charts of the same holon**: declarations
of which common coordinates are active, how host observations enter them, which observables
are read out, and how an approximation is certified. If a purported realization needs a
degree of freedom that cannot embed in the maximal object, that is a measured no-fit against
representational closure—not permission to bolt on a second object model.

What remains domain-sensitive is operational rather than metaphysical: an executable update
operator, an observation/readout map, and an error certificate must be supplied or derived
before that realization can produce numerical trajectories. Those are implementations of the
shared holon under a declared regime. They are not independent ontologies that define a quark,
person, or storyline as a different kind of being.

The arena is a **resident refinement window**, not the whole ontology. Latent holons
represent recursively implied children without enumerating them. An expanded holon is
accepted only when its resident children compose exactly to its REG+ gross state.

Two storage policies execute the same mathematics:

- `HolonArena<CAP, W>` is the allocator-free, compile-time path for sealed scenes and
  very small WASM/embedded deployments;
- `RuntimeArena` is the `no_std + alloc` path for CIRISGame-style browser/native hosts.
  It uses 56-byte fixed-width holon headers, one contiguous header vector, one flat
  variable-width whole-state pool, `u32` holon IDs, and a 64-holons-per-word frontier.

The runtime path therefore does not choose one universal whole-state width and does not
allocate one `Vec` per person, quark, storyline, or boundary element. A reusable
`RuntimeFrontier` makes repeated certifications allocation-free after scene construction.
Latent branches can be materialized transactionally: child IDs append without
invalidating existing handles, and no mutation commits unless the children have valid
depth/grain and compose exactly back to the parent REG+ ledger.

## Mathematics carried into the runtime

`regplus.rs` reproduces `Core/Lattice.lean`'s six-direction object directly. Its tests
enumerate all 64 local states and recover exactly 53 `(N,P)` sectors with dimensions
`44 × 1`, `7 × 2`, and `2 × 3`. Gross occupancy and momentum then compose additively
through arbitrary recursive holons. A transition can be checked for sector preservation;
no collision law is smuggled into that check.

Whole-state is stored separately. Consequently, requiring children to balance the REG+
ledger does not assert that the parent is computable from partial views.

## Gross plus boundary

Realization-specific mathematics implements `BoundaryModel`. It evaluates the current
frontier, supplies a normalized macroscopic error bound and conservation residual, and
ranks boundary holons for refinement. The selector begins at the encounter root and
returns the first frontier satisfying both tolerances. Failure at `g0` is reported as
`GrainFloor`; unavailable resident children are reported as `RefinementUnavailable`.

The first concrete realization is `SphereContactModel`. It deliberately demonstrates the
architecture's split:

- arbitrarily large object counts and unequal masses stay in the gross calculation;
- exact restitution supplies the aggregate contact impulse;
- deterministic Fibonacci boundary samples approximate the contact surface;
- only contact-side branches refine;
- an analytic sphere supplies the error bound and contact-time reference.

Run it with:

```sh
cargo run -p ciris-sim-core --example recursive_holons
```

The example represents a one-million-element person and a 64,000-element NPC in a
15-holon resident window. Mechanical contact is only one realization attached to those
holons; both also carry narrative and agentic channels.

The runtime-sized version begins with only the encounter, person, and NPC resident. A
procedural materializer then instantiates only branches requested by the certificate:

```sh
cargo run -p ciris-sim-core --example runtime_materialization --features alloc --release
```

It grows from 3 to 11 resident holons through 4 materializations, stops at grain 2 for
both objects, and returns the same six-evaluation certificate as the fully supplied
hierarchy: 0.074963% boundary error, `1.432e-16` conservation residual, contact time
`0.451311852`, and impulse `108270.676692`.

## Materials, relations, and cracks

The Newtonian material slice adds a second concrete chart without adding a second entity
class. `MaterialBinding` points from a subject holon to a descriptor holon and supplies the
isotropic SI properties the mechanical evaluator reads. In the demo, wall holon `#2` is
bound to stone descriptor holon `#3`. "Stone" is therefore a warranted description of the
same wall, not a `Stone` subclass or a string that secretly selects bespoke physics.

Connections are addressable too. Each `CohesiveBond` carries its own relation-holon ID, its
two endpoint holons, rest length, bilinear cohesive law, and irreversible damage. A weaker
connection has lower peak force, lower fracture work, or both. A crack is not painted or
declared separately: it is the observable set of relation holons with `damage == 1`.

The browser gate uses an exact 10,000-constituent ball gross state and an exact
1,000,000-constituent wall gross state. The wall's resident mechanical frontier has 288
nodes and 797 sparse cohesive relations; terminal representation weights sum exactly to
one million. The current gate fixes that resident frontier so the material law can be
validated independently. Coupling its damage residual to transactional runtime
materialization—refining only the crack tip until a declared crack-path/impulse tolerance
passes—is the next adaptive-fracture step.

The compiled browser module is raw `wasm32-unknown-unknown`. JavaScript supplies pointer
input and Canvas drawing only; all contacts, forces, integration, damage, and crack state
advance in Rust. The screenshot artifact is
[`output/playwright/cirisholon-ball-wall-fracture.png`](output/playwright/cirisholon-ball-wall-fracture.png).

## Rust and WebAssembly boundary

The hot path is a direct Rust dependency. CIRISGame/Bevy should enable the core's `alloc`
feature and retain `RuntimeArena` plus a `RuntimeFrontier` workspace as resources. This
keeps JSON, JavaScript callbacks, trait objects, and ABI copies out of stepping.

CEWPOS-style sandboxing is supplied separately by `crates/ciris-sim-component`. Its WIT
world imports nothing and accepts typed scene/contact records. The adapter flattens them
into the same runtime arena and returns a typed resolution certificate. It is a trust and
control-plane boundary, not a per-frame call. The built core module has zero imports and
an embedded component type; the repository's `examples/lift.rs` produces and validates
the final WebAssembly Component without a globally installed `wasm-tools`.

On this Mac mini M4, both release component artifacts round to 38 KiB (38,101 bytes
before lifting and 38,443 bytes after). Both `wasm32-unknown-unknown` and
`wasm32-wasip1` compile gates pass for the runtime-enabled core.

## Storage-policy parity and cost

The same 15-holon adaptive sphere contact is executed through both arenas in a unit test.
Status, observables, error bound, conservation residual, evaluation count, active count,
and represented grain agree; floating results are checked by raw IEEE-754 bits.

`cargo bench -p ciris-sim-core --bench runtime_refinement --features alloc` isolates only
storage-policy overhead. These are medians of nine runs of 250,000 identical
two-evaluation certificates on the M4:

| path | ns / certificate | vs const |
|---|---:|---:|
| const arena | 15.79 | 1.000x |
| runtime arena, reused frontier | 15.07 | 0.954x |
| runtime arena, owned certificate/frontier | 32.32 | 2.046x |

The recommended runtime API is the reusable form: runtime sizing and variable-width
whole-state add no measurable penalty in this small selector benchmark (the compact
bitset is slightly faster in this run). The owned form is for one-shot/component calls,
where returning the frontier is more useful than avoiding one bitset allocation.

## Acceptance in this slice

1. Runtime REG+ sector table equals the machine-checked Lean table.
2. Expanded children exactly conserve constituent count, occupancy, and momentum.
3. Frontier refinement never changes the root gross state.
4. Irreducible whole-state survives without a factoring assertion.
5. Multiple realization channels coexist on one holon.
6. The mechanical evaluator certifies <=0.1% boundary error and <=1e-12 momentum
   residual without expanding latent interiors.
7. The library remains `no_std`, deterministic, and free of unsafe code. Its default
   feature set is allocator-free; runtime sizing opts into `alloc` without opting into
   `std`.
8. The const and runtime storage policies return bit-identical mechanical certificates.
9. The zero-import WIT guest compiles and lifts to a validated WebAssembly Component.
10. Adaptive runtime certification starts from three holons, materializes only unresolved
    boundary branches, and reproduces the prebuilt hierarchy's certificate exactly.
11. A deterministic material gate expresses matter through descriptor holons, weakness
    through relation holons, and cracks through irreversible cohesive failure.
12. A real pointer event drives the Rust/WASM ball-wall simulation to visible fracture in a
    browser with zero console errors.

## Explicit remaining work

- Prove the aggregation/commutation theorem that bounds fine REG+ evolution by a
  boundary-supported residual. The current Rust composition check establishes the
  conserved ledger, not dynamical lumpability.
- Allocate one global error budget across nested encounters; the current certificate is
  for one interaction frontier.
- Replace the analytic sphere sampler with the CIRIS-generated boundary representation.
- Implement quantum/route, narrative/temporal, and agentic realization maps and evaluators
  over the same maximal choice object. They must not introduce sibling entity types or
  independent ontologies; each owes an update/readout/certificate before numerical claims.
- Couple refined mechanical boundary holons to the iterative simultaneous-contact solver
  still owed by the sparse contact layer.

Rapier remains a limiting-case control for classical rigid behavior, not the definition
of a holon and not the reference for non-mechanical channels.

The extraction and naming decision is recorded in [`CIRIS_HOLON_HOME.md`](CIRIS_HOLON_HOME.md).
The paused Qwen3-0.6B/H3ERE2-G work and its restart gate are recorded in
[`H3ERE2_RESTART.md`](H3ERE2_RESTART.md).
