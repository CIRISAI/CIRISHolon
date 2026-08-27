//! # holon-mesh
//!
//! **ONE holon scene, sharded across cores, gated bit-identical against the unsharded run.**
//!
//! This is the LOCAL mesh of `sim_engine/MESH_DESIGN.md`. It is not federation: there is no
//! transport, no envelope, no attribution, no consent, and no node identity anywhere in the
//! crate. In-process, on-machine, one scene.
//!
//! ## What it is for
//!
//! `SANDBOX_4090.md` §4 measured that memory was never the constraint — 149 resident holons
//! against a 1.49e8 capacity. Wall-clock is. `MESH_DESIGN.md` §0 computes that a 3D sandbox's
//! *visible surface* at observer acuity is ~1.05e6 nodes, which is 7,000× the measured 2D
//! frontier and more than one core will step. Sharding is how the scene gets stepped.
//!
//! ## The relationship to `holon-swarm`
//!
//! `holon-swarm` shards MANY independent scenes and relates them by abstract index pairs.
//! This crate shards ONE scene and the boundaries are geometry. It reuses that crate's proved
//! parts verbatim — the transfer rule (`plan_transfer`, exactly antisymmetric, never
//! overdrawing), the ledger delta arithmetic, and the board idea — and adds the spatial index,
//! the halo, and the gate against an unsharded reference.
//!
//! ## The gate, and what makes it able to fail
//!
//! The claim is `meshed == unsharded`, **bit-identical**, on every cell's four integer lanes
//! and every whole-state f64 by `to_bits()`. Not 99.9%.
//!
//! Three independently written paths make that claim non-vacuous:
//!
//! | path | what it is |
//! |---|---|
//! | [`reference::Reference`] | one arena, the whole scene, no shards. Knows nothing of halos, boards, colours-per-exchange or threads. |
//! | [`mesh::Mesh::run_sequential`] | sharded, single-threaded, plain loops. |
//! | [`mesh::Mesh::run_threaded`] | sharded, scoped workers, atomic board, barriers. |
//!
//! And the mutation set ([`mutation::Mutation`]) is where the honesty lives. `MESH_DESIGN.md`
//! §6 names the trap and this crate encodes it: a reordered merge over integer lanes produces
//! the **identical** result, so "reorder and assert the answer changed" cannot pass against a
//! correct implementation. The mutation is therefore split — reordering must NOT fire, and a
//! planted cross-shard f64 reduction MUST — and only the pair together proves anything.
//!
//! ## The horizon is a theorem, and its tightness is a measurement
//!
//! A shard stepping `n` colours between exchanges carries a halo of depth `n·r`, which
//! `CIRISOntology/Core/Locality.lean::iterate_depends_within` proves sufficient. Whether it is
//! *necessary* the Lean does not say, so this crate does not assume it:
//! [`mutation::Mutation::HaloOneShallowerThanHorizon`] builds `n·r − 1` and the gate measures
//! whether the answer moves.
//!
//! ## Single tier, by construction
//!
//! [`mesh::MeshSpec`] carries one `g0_m` for the whole scene and offers no way to express a
//! second. That is `SANDBOX_4090` G4 as a type: nothing certifies a join across a re-root
//! (`Core/GrainFloor.lean::cert_does_not_transport_across_reroot`), so until the re-root
//! ledger gate lands a multi-tier mesh would be uncertified by construction.

pub mod error;
pub mod fchc;
pub mod grid;
pub mod mesh;
pub mod mutation;
pub mod reference;
pub mod sizing;
pub mod state;

pub use error::MeshError;
pub use fchc::{enumerate, fchc_directions, fhp_directions, Enumeration};
pub use grid::{Edge, Grid, Partition, EDGE_COLOURS, RADIUS};
pub use mesh::{Mesh, MeshShard, MeshSpec};
pub use mutation::{Mutation, VisitOrder};
pub use reference::Reference;
pub use sizing::SceneSizing;

/// Run the reference and a mesh over the same scene and compare them bit-for-bit.
///
/// Returns `Ok(())` when every cell agrees in all four integer lanes and the whole-state
/// scalar agrees by raw bits, and the index of the first disagreement otherwise. This is the
/// gate; every test in this crate is a call to it with a different spec.
pub fn compare_to_reference(
    spec: MeshSpec,
    colour_steps: usize,
    threads: Option<usize>,
) -> Result<Result<(), Disagreement>, MeshError> {
    let grid = spec.grid;
    let mut reference = Reference::new(grid)?;
    reference.run(colour_steps)?;

    let mut mesh = Mesh::new(spec)?;
    match threads {
        Some(t) => mesh.run_threaded(colour_steps, t)?,
        None => mesh.run_sequential(colour_steps)?,
    }

    let mesh_cells = mesh.cells();
    let mesh_energy = mesh.energies();
    for cell in 0..grid.len() {
        if mesh_cells[cell] != reference.cells()[cell] {
            return Ok(Err(Disagreement::Ledger {
                cell: cell as u32,
                meshed: mesh_cells[cell],
                unsharded: reference.cells()[cell],
            }));
        }
        if mesh_energy[cell].to_bits() != reference.energies()[cell].to_bits() {
            return Ok(Err(Disagreement::WholeState {
                cell: cell as u32,
                meshed: mesh_energy[cell].to_bits(),
                unsharded: reference.energies()[cell].to_bits(),
            }));
        }
    }
    Ok(Ok(()))
}

/// The first place a meshed run and the unsharded reference differ. Named, with both values,
/// because "the gate failed" is not a report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Disagreement {
    Ledger {
        cell: u32,
        meshed: ciris_sim_core::regplus::GrossState,
        unsharded: ciris_sim_core::regplus::GrossState,
    },
    WholeState {
        cell: u32,
        meshed: u64,
        unsharded: u64,
    },
}
