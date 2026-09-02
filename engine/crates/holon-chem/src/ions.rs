//! Charge at the solver seam: an electron count the caller states, and a refusal for
//! everything it does not.
//!
//! # What this module is, and what it deliberately is not
//!
//! [`crate::pair::solve_basis`] has always taken `(n_alpha, n_beta)` explicitly — the
//! seam where charge enters was already open. What was missing was an HONEST LAYER above
//! it: something that turns a total charge into that pair by a rule written down in
//! advance, and refuses rather than guesses everywhere the rule does not reach. That is
//! all this module is. There is no ion table here, no dynamics, no census bookkeeping and
//! no species registry of "the ions we support" — a charged fragment is a species list, a
//! geometry, and an integer, exactly as a neutral one is a species list and a geometry.
//!
//! What remains, and who owns it, is `conformance/water_observatory/ION_STAKING.md`.
//!
//! # The electron count is arithmetic; the spin sector is a MODEL CHOICE
//!
//! `n_electrons = sum(Z) - charge` is a definition and there is nothing to choose. The
//! partition of those electrons into `(n_alpha, n_beta)` is not: it names the `S_z` sector
//! the CI solve runs in, and a wrong sector is a wrong energy with nothing in the output
//! to signal it. [`spin_partition`] states the choice this crate makes, its warrant, and
//! the honest caveat that goes with it. Read that doc comment before believing an energy
//! from here.
//!
//! # The three refusals, and why refusing is the point
//!
//! [`ChargeRefusal`] has three variants and each one is a place where a silent default
//! would have produced a number. `NegativeElectrons` is an ion with fewer than zero
//! electrons; `ChargeTooLarge` is an anion carrying more excess electrons than the whole
//! system has protons; `UnstatedSpinSector` is a sector the declared basis cannot seat, so
//! the rule named a partition that does not exist in this model. None of the three is a
//! tolerance and none has a "best effort" path around it.

use crate::dual::D2;
use crate::elements::Species;
use crate::pair::{build_basis, solve_basis, PointSolution};

/// Why [`solve_geometry_charged`] declined to produce a number.
///
/// Every variant carries the numbers that made it fire, so a caller reports the refusal
/// rather than re-deriving the condition — a caller that re-derives it is a second copy of
/// the rule, and the two copies are free to disagree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargeRefusal {
    /// `charge > sum(Z)`: the system would have fewer than zero electrons.
    ///
    /// This is the CATION half of `|charge| > sum(Z)`; [`ChargeRefusal::ChargeTooLarge`]
    /// is the anion half, and the two are disjoint by construction rather than by
    /// ordering. Stated that way on purpose: a single `|charge| > sum(Z)` test placed
    /// first would make this variant UNREACHABLE — every negative-electron input satisfies
    /// it — and an unreachable refusal is a refusal that has stopped existing. Which
    /// variant a caller gets is part of the contract, and `tests/ion_core.rs` plants a
    /// large positive charge specifically to pin that it comes back as this one.
    NegativeElectrons {
        /// Total nuclear charge of the species list.
        total_z: u32,
        /// The charge that was asked for.
        charge: i32,
        /// The electron count that would have followed, as a signed number.
        would_be_electrons: i64,
    },
    /// `charge < -sum(Z)`: an anion holding more excess electrons than the system has
    /// protons.
    ///
    /// This is the ANION half of `|charge| > sum(Z)`; the cation half is
    /// [`ChargeRefusal::NegativeElectrons`], which names the sharper fact. Nothing about
    /// such an input is a small-basis problem — it is nonsense before any basis is chosen
    /// — so it is refused without assembling one.
    ChargeTooLarge {
        /// Total nuclear charge of the species list.
        total_z: u32,
        /// The charge that was asked for.
        charge: i32,
    },
    /// The partition [`spin_partition`] named cannot be seated in the declared basis:
    /// `n_alpha` exceeds the number of spatial orbitals.
    ///
    /// # Why this is a SPIN-SECTOR refusal and not a basis-size one
    ///
    /// The rule in [`spin_partition`] is total: it names an `(n_alpha, n_beta)` for every
    /// electron count. What it cannot do is guarantee that pair exists as a sector of THIS
    /// model — and when it does not, the rule has stated nothing usable about the system in
    /// front of it. The alternative to refusing is a `FciSpace` with zero determinants,
    /// which is not an error anywhere downstream and would surface as a nonsense energy.
    ///
    /// With the parity rule as written this fires exactly when `n_electrons > 2 *
    /// n_orbitals`, i.e. the minimal basis cannot hold the electrons the charge implies.
    /// Both numbers are carried so the caller can say which.
    UnstatedSpinSector {
        /// Electrons the charge implied.
        n_electrons: usize,
        /// Spatial orbitals the declared basis supplies.
        n_orbitals: usize,
        /// The alpha count the rule named, which is the one that does not fit.
        n_alpha: usize,
    },
}

/// The `S_z` sector this crate solves a charged fragment in: even electron count to a
/// singlet, odd to a doublet.
///
/// Returns `(n_alpha, n_beta)` with `n_alpha + n_beta = n_electrons` and `n_alpha - n_beta`
/// either 0 or 1.
///
/// # This is a MODEL CHOICE, stated here so nothing has to guess it
///
/// It is the same rule [`crate::elements::sz2_sector`] applies to neutral species, and it
/// is chosen for the same reason: a multiplet of total spin `S` has a component in every
/// sector with `|S_z| <= S`, so the sector with the smallest `|S_z|` consistent with the
/// electron count contains EVERY state of the system whatever its spin. Solving there
/// therefore cannot miss the ground state by having picked the wrong multiplicity, which a
/// guessed sector can — and the cost is that the determinant space is at its largest.
///
/// # The honest caveat
///
/// The argument above says the minimal sector CONTAINS the ground state. It does not say
/// the returned energy IS the ground state's, and the difference is where this rule can
/// still be wrong:
///
/// * **It fixes `S_z`, not `S`.** Nothing here reports which total spin the state it found
///   has, so an energy from this function is not a statement about multiplicity. Use
///   [`crate::fci::s_squared`] when that matters.
/// * **Containment is not attainment.** The minimal sector is also the LARGEST determinant
///   space, and the reported energy is the ground state's only if the solver reached the
///   global minimum of that space. No residual can certify that it did — a residual is
///   small for any eigenvector — and the cheap check that can is
///   [`crate::fci::Solution::variational_margin`], which [`PointSolution`] does not
///   currently carry.
///
/// **The promote path is a variational sweep over sectors**: solve `S_z = 0, 1, 2, …` (or
/// `1/2, 3/2, …`) at one geometry, take the lowest, and report the sector that won. State
/// precisely what that buys, because it is easy to oversell: in EXACT arithmetic the sweep
/// cannot beat the minimal sector, so it is not a correction to the rule. What it is, is a
/// check on the SOLVER — a smaller sector converging below the big one is the signature of
/// a solve that did not reach the minimum — and it is how the winning total spin becomes a
/// measurement rather than an assumption. Not done here because it multiplies the cost by
/// the number of sectors swept and nothing in this seam needs it yet;
/// `conformance/water_observatory/ION_STAKING.md` carries it as row I-4 with its
/// receipt-gate, and the missing margin as I-6.
pub fn spin_partition(n_electrons: usize) -> (usize, usize) {
    let n_beta = n_electrons / 2;
    (n_electrons - n_beta, n_beta)
}

/// Solve one geometry at a stated total charge.
///
/// `charge` is the TOTAL charge of the fragment in units of the elementary charge: `+1` for
/// H3O+, `-1` for OH−, `0` for a neutral. The electron count is `sum(Z) - charge` and the
/// spin sector is [`spin_partition`]'s; both are stated rather than inferred, and every
/// input for which neither is stateable comes back as a [`ChargeRefusal`].
///
/// # Bit-identical to [`crate::pair::solve_geometry`] at `charge == 0`
///
/// Not approximately, and not by test tolerance: at zero charge the electron count is
/// `sum(Z)`, [`spin_partition`] and [`crate::elements::sz2_sector`] name the same partition
/// for it, and the call handed to [`solve_basis`] is therefore the identical call. The
/// neutral path is not re-implemented here and must not be — `tests/ion_core.rs` asserts
/// equality on the raw bits of the energy for H2 and H2O, which a re-implementation that
/// agreed to fifteen digits would fail.
///
/// # What is NOT claimed
///
/// The returned energy is exact-in-model only if `PointSolution::route` says
/// [`crate::fci::SolverRoute::Determinant`]; past
/// [`crate::fci::MPS_ROUTE_THRESHOLD`] determinants [`crate::pair::solve_basis`] routes to
/// DMRG and the number is a variational upper bound inside a bond-dimension budget. Charge
/// makes this MORE likely, not less: adding a proton adds a basis function without removing
/// an electron. Callers comparing two charge states must read the route on both.
///
/// # What is a PANIC here rather than a refusal, and why
///
/// `species.len() != centers.len()` panics, inside [`crate::pair::build_basis`], exactly as
/// it does on the neutral path. That is deliberate rather than an omission: it is a caller
/// bug about the geometry and has nothing to do with charge, and giving it a
/// [`ChargeRefusal`] variant here would mean the charged and neutral doors disagreed about
/// what a malformed geometry IS. The refusals in this module are about charge only.
pub fn solve_geometry_charged(
    species: &[Species],
    centers: Vec<[D2; 3]>,
    charge: i32,
) -> Result<PointSolution, ChargeRefusal> {
    let total_z: u32 = species.iter().map(|s| s.z).sum();
    let signed_electrons = i64::from(total_z) - i64::from(charge);

    // The two arithmetic refusals partition `|charge| > sum(Z)` by SIGN — cation half
    // first, anion half second, disjoint by the `charge < 0` guard rather than by their
    // order. Collapsing them into one `|charge| > sum(Z)` test would retire
    // `NegativeElectrons`; see its doc comment, and the plant that pins it.
    if signed_electrons < 0 {
        return Err(ChargeRefusal::NegativeElectrons {
            total_z,
            charge,
            would_be_electrons: signed_electrons,
        });
    }
    if charge < 0 && i64::from(charge.unsigned_abs()) > i64::from(total_z) {
        return Err(ChargeRefusal::ChargeTooLarge { total_z, charge });
    }

    let n_electrons = signed_electrons as usize;
    let (n_alpha, n_beta) = spin_partition(n_electrons);

    // The basis has to be assembled to know how many orbitals it supplies, so the seating
    // check cannot come earlier. It comes before the SOLVE, which is the expensive half.
    let basis = build_basis(species, centers);
    if n_alpha > basis.n {
        return Err(ChargeRefusal::UnstatedSpinSector {
            n_electrons,
            n_orbitals: basis.n,
            n_alpha,
        });
    }

    Ok(solve_basis(&basis, n_alpha, n_beta))
}
