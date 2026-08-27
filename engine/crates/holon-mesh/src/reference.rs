//! The unsharded reference: ONE arena, the whole scene, no shards and no boundaries.
//!
//! This is the thing `meshed == unsharded` is measured against, and it is written here
//! independently of [`crate::mesh`] on purpose. If the two were one code path behind a
//! `sharded: bool`, agreement between them would be a tautology dressed as evidence — the
//! same argument `holon-swarm` makes for its sequential path, inherited deliberately.
//!
//! It knows nothing about shards, halos, boards, barriers, colours-per-exchange, or threads.
//! It sweeps the four edge colours in order and applies each edge's transfer. That is all.

use ciris_sim_core::holon::{Channels, Decomposition, HolonError};
use ciris_sim_core::regplus::GrossState;
use ciris_sim_core::runtime::{RuntimeArena, RuntimeHolonSpec, NO_RUNTIME_HOLON};
use holon_swarm::ledger::apply_delta;

use crate::error::MeshError;
use crate::grid::{edges_of_colour, Edge, Grid, EDGE_COLOURS};
use crate::state::{advance_energy, plan, seed_energy, seed_gross};

/// The whole scene in one arena. Holon 0 is the root; cell `c` is holon `c + 1`.
pub struct Reference {
    grid: Grid,
    arena: RuntimeArena,
    /// Live ledger, index-aligned with the arena. The arena's STORED gross states never
    /// change — `SANDBOX_4090` §6's weakness (2), carried here rather than rediscovered.
    gross: Vec<GrossState>,
    energy: Vec<f64>,
    colours: Vec<Vec<Edge>>,
}

impl Reference {
    pub fn new(grid: Grid) -> Result<Self, MeshError> {
        let n = grid.len();
        let gross: Vec<GrossState> = (0..n as u32).map(seed_gross).collect();
        let energy: Vec<f64> = (0..n as u32).map(seed_energy).collect();

        let mut root = GrossState::ZERO;
        for g in &gross {
            root = root
                .checked_combine(*g)
                .ok_or(MeshError::Core(HolonError::GrossStateDoesNotCompose))?;
        }
        let channels = Channels::REG_PLUS.union(Channels::MECHANICAL);
        let mut specs = Vec::with_capacity(n + 1);
        specs.push(RuntimeHolonSpec {
            parent: NO_RUNTIME_HOLON,
            depth: 0,
            grain_units: 2,
            gross: root,
            whole: &[],
            channels,
            boundary: true,
            decomposition: Decomposition::Expanded,
        });
        for g in &gross {
            specs.push(RuntimeHolonSpec {
                parent: 0,
                depth: 1,
                grain_units: 1,
                gross: *g,
                whole: &[],
                channels,
                boundary: false,
                decomposition: Decomposition::Leaf,
            });
        }
        let arena = RuntimeArena::from_specs(&specs, 0)?;

        // The live overlay carries the root too, so composition can be checked over live
        // values rather than over the arena's frozen headers.
        let mut live = Vec::with_capacity(n + 1);
        live.push(root);
        live.extend_from_slice(&gross);

        let colours = (0..EDGE_COLOURS)
            .map(|c| edges_of_colour(grid, c))
            .collect();

        Ok(Self {
            grid,
            arena,
            gross: live,
            energy,
            colours,
        })
    }

    pub fn grid(&self) -> Grid {
        self.grid
    }

    pub fn arena(&self) -> &RuntimeArena {
        &self.arena
    }

    /// Ledger of every cell, in global index order. The fingerprint the gate compares.
    pub fn cells(&self) -> &[GrossState] {
        &self.gross[1..]
    }

    pub fn energies(&self) -> &[f64] {
        &self.energy
    }

    /// Root ledger: the scene's total, held live.
    pub fn total(&self) -> GrossState {
        self.gross[0]
    }

    /// Run `colour_steps` sub-steps, sweeping colours `0,1,2,3,0,1,...`.
    pub fn run(&mut self, colour_steps: usize) -> Result<(), MeshError> {
        for step in 0..colour_steps {
            self.sweep(step % EDGE_COLOURS)?;
        }
        Ok(())
    }

    fn sweep(&mut self, colour: usize) -> Result<(), MeshError> {
        // A colour is a perfect matching, so no cell is written twice here and the visit
        // order below cannot matter. `grid::tests::each_colour_is_a_perfect_matching` is
        // what earns that sentence.
        for i in 0..self.colours[colour].len() {
            let edge = self.colours[colour][i];
            let lo = (edge.lo + 1) as usize;
            let hi = (edge.hi + 1) as usize;
            let d = plan(self.gross[lo], self.gross[hi]).ok_or(MeshError::LedgerRange {
                cell: edge.lo,
            })?;
            let debit = d.checked_neg().ok_or(MeshError::LedgerRange { cell: edge.lo })?;
            self.gross[lo] = apply_delta(self.gross[lo], debit)
                .ok_or(MeshError::LedgerRange { cell: edge.lo })?;
            self.gross[hi] =
                apply_delta(self.gross[hi], d).ok_or(MeshError::LedgerRange { cell: edge.hi })?;
        }
        for cell in 0..self.grid.len() {
            self.energy[cell] = advance_energy(self.energy[cell], self.gross[cell + 1]);
        }
        Ok(())
    }
}
