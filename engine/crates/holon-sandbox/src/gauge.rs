//! The vacuum tier: one U(1) plaquette, exactly.
//!
//! This tier is not an approximation of anything, which makes it the odd one on the
//! ladder. Everywhere else the certificate asks whether a coarse frontier resolves a
//! fine truth well enough; here the state space is four links carrying three flux values
//! each, the Gauss law is a machine predicate over it, and the ground state comes out of
//! a 3x3 eigenproblem the core already solves deterministically. There is no grain to be
//! coarse about, so the honest error bound is zero and the honest grain constant is
//! absent — the lattice spacing is not pinned to any length, and writing one in would be
//! inventing a metre.
//!
//! All of the physics is `ciris_sim_core::quantum_link` (on main as of f6007f5). This
//! module is a chart over it: it holds one plaquette's flux, drives the raising move,
//! and reads the vacuum out.
//!
//! # What this tier may and may not say — a labelling constraint, not a style note
//!
//! `Core/RouteGauge.lean` KILLED the route → gauge identification by machine. The
//! decomposition of the route Hamiltonian in the spin-1 ladder is exact, but its gauge
//! reading is pinned to one hand-selected representative: a route-side move that changes
//! no route observable relocates the Wilson phase, and no Gauss-generated link gauge
//! transformation can follow it. So **the eleven-kind taxonomy's route object and this
//! tier's gauge flux DO NOT share a carrier**, and nothing rendered here may imply they
//! do.
//!
//! What survives, exactly, and is the one thing this tier's panel is allowed to say
//! about the relationship: **link charge conjugation (flux → −flux) acts on the route
//! Hamiltonian as time reversal** (`charge_conj_is_time_reversal`), with exact amplitude
//! pairings at the evolution level. That is one finite symmetry read in two languages —
//! a dictionary entry, not a shared carrier. [`GaugeScene::charge_conjugate`] is that
//! symmetry, executable, and the demo labels it as the dictionary it is.

use ciris_sim_core::quantum_link::{
    is_gauss_closed, one_plaquette_vacuum, plaquette_lower, plaquette_raise, Plaquette,
    CLOSED_FLUX_BASIS,
};

/// Electric coupling `g²` of the minimal one-plaquette Hamiltonian. A dimensionless
/// stage value: it moves where the vacuum sits between the flux sectors, and nothing
/// about whether the state is Gauss-closed.
pub const COUPLING_G2: f64 = 0.35;

/// Magnetic coupling `kappa`, likewise dimensionless and likewise a stage value.
pub const COUPLING_KAPPA: f64 = 1.0;

/// What a move on the plaquette did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    /// The loop was raised or lowered and remains Gauss-closed.
    Applied,
    /// The spin-1 truncation. Flux is `-1, 0, +1` and no more, so a raise at `+1` has
    /// nowhere to go. This is a REFUSAL and it is exact — not a resolution limit, not a
    /// budget, not a missing evaluator, but the state space ending.
    FluxCeiling,
}

/// One plaquette and what can be read off it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaugeScene {
    flux: Plaquette,
    /// Ground-state probability of each closed-flux sector, `|-1>, |0>, |+1>`.
    pub vacuum: [f64; 3],
    pub ground_energy: f64,
    /// `beta` in `-log rho_link = a + beta E²`: the one-link modular spectrum is
    /// electric-quadratic, and this is how quadratic.
    pub modular_beta: f64,
    /// How far the vacuum departs from charge-conjugation symmetry. Exactly zero for
    /// this Hamiltonian, and read out rather than assumed so it stays a measurement.
    pub charge_conjugation_residual: f64,
    /// Relative residual of the three modular energies against `a + beta E²`.
    pub modular_fit_residual: f64,
    pub last_move: Option<Move>,
}

impl Default for GaugeScene {
    fn default() -> Self {
        Self::new()
    }
}

impl GaugeScene {
    pub fn new() -> Self {
        let mut scene = Self {
            // Start in the zero-flux sector: the loop is closed and carries nothing.
            flux: [0; 4],
            vacuum: [0.0; 3],
            ground_energy: 0.0,
            modular_beta: 0.0,
            charge_conjugation_residual: 0.0,
            modular_fit_residual: 0.0,
            last_move: None,
        };
        scene.read_vacuum();
        scene
    }

    fn read_vacuum(&mut self) {
        // The couplings are validated by the core: a non-finite or negative pair returns
        // `None` rather than a spectrum. Constants cannot fail it, and if they ever do,
        // the readout stays at zero rather than becoming a number nothing produced.
        if let Some(vacuum) = one_plaquette_vacuum(COUPLING_G2, COUPLING_KAPPA) {
            self.vacuum = vacuum.probabilities;
            self.ground_energy = vacuum.ground_energy;
            self.modular_beta = vacuum.modular_beta;
            self.charge_conjugation_residual = vacuum.charge_conjugation_residual;
            self.modular_fit_residual = vacuum.modular_electric_fit_residual;
        }
    }

    pub const fn flux(&self) -> Plaquette {
        self.flux
    }

    /// The common flux around the loop, `-1`, `0` or `+1`. Meaningful only on a
    /// Gauss-closed state, which is the only kind this scene holds.
    pub const fn loop_flux(&self) -> i8 {
        self.flux[0]
    }

    /// Is the state Gauss-closed? A machine predicate from the core, checked rather than
    /// maintained by construction, so the demo is reading the law and not its own
    /// bookkeeping.
    pub fn closed(&self) -> bool {
        is_gauss_closed(&self.flux)
    }

    /// The throw: raise the loop by one unit of flux.
    ///
    /// At `+1` this refuses, and the refusal is the spin-1 truncation itself. It is the
    /// only refusal on the whole ladder that is not about resolution — everywhere else
    /// the engine is saying "I cannot see finely enough", and here it is saying "there
    /// is nothing finer".
    pub fn raise(&mut self) -> Move {
        match plaquette_raise(&self.flux) {
            Some(next) => {
                debug_assert!(is_gauss_closed(&next));
                self.flux = next;
                self.last_move = Some(Move::Applied);
                Move::Applied
            }
            None => {
                self.last_move = Some(Move::FluxCeiling);
                Move::FluxCeiling
            }
        }
    }

    pub fn lower(&mut self) -> Move {
        match plaquette_lower(&self.flux) {
            Some(next) => {
                debug_assert!(is_gauss_closed(&next));
                self.flux = next;
                self.last_move = Some(Move::Applied);
                Move::Applied
            }
            None => {
                self.last_move = Some(Move::FluxCeiling);
                Move::FluxCeiling
            }
        }
    }

    /// Link charge conjugation: flux → −flux on every link.
    ///
    /// This is the ONE structure that transports between the route object and the link
    /// object, and it transports as a DICTIONARY ENTRY, not as an identification of
    /// carriers: `Core/RouteGauge.lean` proves it acts on the route Hamiltonian as time
    /// reversal, and states in the same breath that "neither identifies the carriers:
    /// both are one finite symmetry (transposition) read in two languages". The route →
    /// gauge carrier identification is dead by machine, and this method is not a way to
    /// reintroduce it.
    pub fn charge_conjugate(&mut self) {
        for link in &mut self.flux {
            *link = -*link;
        }
        debug_assert!(is_gauss_closed(&self.flux));
    }

    /// Index of the current sector in `CLOSED_FLUX_BASIS`, i.e. `0`, `1`, `2` for flux
    /// `-1`, `0`, `+1`.
    pub fn sector(&self) -> usize {
        CLOSED_FLUX_BASIS
            .iter()
            .position(|state| *state == self.flux)
            .unwrap_or(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scene_is_gauss_closed_at_every_reachable_state() {
        let mut scene = GaugeScene::new();
        assert!(scene.closed());
        // Down to the floor and up to the ceiling, checking the law at every step.
        for _ in 0..4 {
            scene.lower();
            assert!(scene.closed(), "lowering left a Gauss-open state");
        }
        for _ in 0..6 {
            scene.raise();
            assert!(scene.closed(), "raising left a Gauss-open state");
        }
    }

    /// The ceiling is the state space ending, not a resolution limit — and it is
    /// reachable in exactly two raises from the zero sector, because there are exactly
    /// three closed-flux states.
    #[test]
    fn the_flux_ceiling_refuses_after_exactly_two_raises() {
        let mut scene = GaugeScene::new();
        assert_eq!(scene.loop_flux(), 0);
        assert_eq!(scene.raise(), Move::Applied);
        assert_eq!(scene.loop_flux(), 1);
        assert_eq!(scene.raise(), Move::FluxCeiling);
        assert_eq!(scene.loop_flux(), 1, "a refused move must not move anything");
        assert_eq!(scene.raise(), Move::FluxCeiling, "and it stays refused");
    }

    #[test]
    fn the_three_sectors_are_the_whole_closed_basis() {
        let mut scene = GaugeScene::new();
        let mut seen = std::collections::BTreeSet::new();
        scene.lower();
        scene.lower();
        for _ in 0..4 {
            seen.insert(scene.loop_flux());
            scene.raise();
        }
        assert_eq!(seen.len(), CLOSED_FLUX_BASIS.len());
        assert_eq!(seen, [-1, 0, 1].into_iter().collect());
    }

    /// The vacuum is symmetric under charge conjugation, and this reads that out of the
    /// core rather than assuming it. A non-zero residual would mean the demo is
    /// rendering a vacuum the Hamiltonian does not have.
    #[test]
    fn the_vacuum_is_charge_conjugation_symmetric_to_machine_precision() {
        let scene = GaugeScene::new();
        assert!(
            scene.charge_conjugation_residual < 1.0e-12,
            "vacuum charge-conjugation residual {:e}",
            scene.charge_conjugation_residual
        );
        let total: f64 = scene.vacuum.iter().sum();
        assert!((total - 1.0).abs() < 1.0e-12, "vacuum is not normalized");
        assert!(
            scene.modular_fit_residual < 1.0e-12,
            "the one-link modular spectrum should be exactly electric-quadratic, \
             residual {:e}",
            scene.modular_fit_residual
        );
    }

    /// Charge conjugation is an involution on the flux state and preserves the Gauss
    /// law. It is the one structure that transports to the route object — as a
    /// dictionary entry, never as a shared carrier.
    #[test]
    fn charge_conjugation_is_an_involution_that_preserves_the_law() {
        let mut scene = GaugeScene::new();
        scene.raise();
        let before = scene.flux();
        scene.charge_conjugate();
        assert!(scene.closed());
        assert_eq!(scene.loop_flux(), -before[0]);
        scene.charge_conjugate();
        assert_eq!(scene.flux(), before);
    }
}
