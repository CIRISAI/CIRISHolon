//! What one node of the table is, and the conditions under which it VOIDs rather than
//! scores.

use holon_chem::fci::{SolveExit, Solution};

/// Why a node produced no usable table entry.
///
/// M-BUDGET-LAUNDER: exhaustion is a VOID, never a quietly-accepted number. Every variant
/// here is a case where the solve returned something that LOOKED like an answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VoidReason {
    /// The Davidson stopped because it ran out of iterations. It has not finished; the
    /// number it holds is wherever it happened to be.
    BudgetExhausted,
    /// The converged energy sits ABOVE the lowest diagonal of the Hamiltonian.
    ///
    /// This is the guard that catches a bad warm start, and it is the only one that works.
    /// A single determinant is itself a trial vector, so the variational principle gives
    /// `E_ground <= min_i H_ii` unconditionally; a Ritz value above that bound is not the
    /// ground state, whatever its residual says.
    ///
    /// It is not hypothetical. Measured on `(H,H,Cl)`
    /// (`holon-chem/examples/s3_warm_probe.rs`): a random warm start converged onto an
    /// eigenvector **7.47 hartree** above the ground state while reporting residual
    /// `5.98e-11` against the correct solve's `5.24e-11`, with the identical exit reason.
    /// Neither the residual nor the exit reason can separate those; this bound separates
    /// them by 7.4 hartree.
    /// Carried as RAW BITS for the same reason every other energy in this crate is: the
    /// enclosing record is compared for bit-identity and hashed into the certificate, and
    /// `f64` is not `Eq` because NaN is not equal to itself. Bits are.
    AboveLowestDiagonal { energy_bits: u64, min_diagonal_bits: u64 },
    /// The solve returned a non-finite energy.
    NotFinite,
}

impl VoidReason {
    /// The energies involved, where the reason carries any. Reconstructed from the bits so
    /// a log line can print numbers while the record still compares by bits.
    pub fn energies(&self) -> Option<(f64, f64)> {
        match *self {
            VoidReason::AboveLowestDiagonal {
                energy_bits,
                min_diagonal_bits,
            } => Some((f64::from_bits(energy_bits), f64::from_bits(min_diagonal_bits))),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            VoidReason::BudgetExhausted => "budget exhausted",
            VoidReason::AboveLowestDiagonal { .. } => "above lowest diagonal",
            VoidReason::NotFinite => "not finite",
        }
    }
}

/// Whether a node scored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeStatus {
    Ok,
    Void(VoidReason),
}

/// One node's published record.
///
/// Energies are carried as RAW BITS rather than as `f64`. That is deliberate: this record
/// is compared for bit-identity across shard counts and hashed into the certificate, and
/// `f64` has two zeros and a NaN that is not equal to itself. Bits have neither problem, so
/// the comparison means what it says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeRecord {
    pub node: u32,
    pub energy_bits: u64,
    pub d1_bits: u64,
    pub d2_bits: u64,
    pub davidson_iters: u32,
    pub cg_iters: u32,
    /// [`SolveExit`] as a stable code, so the record can be hashed and compared.
    pub exit_code: u8,
    pub status: NodeStatus,
    /// Whether this node was warm-started. Recorded because the cold/warm split is what
    /// G1's locality sweep measures, and because a node's cost is not interpretable
    /// without it.
    pub warm: bool,
}

impl NodeRecord {
    /// The status as an integer, for the digest. Distinct codes per void reason, so
    /// swapping one void reason for another is convicted.
    pub fn status_code(&self) -> u64 {
        match self.status {
            NodeStatus::Ok => 0,
            NodeStatus::Void(VoidReason::BudgetExhausted) => 1,
            NodeStatus::Void(VoidReason::AboveLowestDiagonal { .. }) => 2,
            NodeStatus::Void(VoidReason::NotFinite) => 3,
        }
    }

    pub fn energy(&self) -> f64 {
        f64::from_bits(self.energy_bits)
    }

    pub fn is_ok(&self) -> bool {
        matches!(self.status, NodeStatus::Ok)
    }
}

/// [`SolveExit`] as a stable u8. Written out rather than derived so that adding a variant
/// upstream cannot silently renumber a committed table's digest.
pub fn exit_code(e: SolveExit) -> u8 {
    match e {
        SolveExit::Converged => 0,
        SolveExit::IterationCap => 1,
        SolveExit::Stagnated => 2,
        SolveExit::Trivial => 3,
    }
}

/// The verdict on one solve: `None` if it scores, `Some(reason)` if the node VOIDs.
///
/// # Why stagnation is not itself a VOID
///
/// It would be the obvious rule and it is the wrong one. Measured on `(H,H,Cl)`: EVERY
/// solve on that system exits `Stagnated`, cold and warm alike, because the hard-coded
/// `1e-11` Davidson tolerance is below the residual floor for all-electron energies near
/// -467 hartree. If stagnation VOIDed, no chlorine table would build at all.
///
/// So stagnation is RECORDED and not judged (M-SORTS-NOT-SEPARATES: it sorts these solves
/// without separating them). What is judged is the variational bound, which does separate
/// them — and the iteration cap, which really does mean the solve never finished.
pub fn void_reason(sol: &Solution) -> Option<VoidReason> {
    if !sol.e.v.is_finite() {
        return Some(VoidReason::NotFinite);
    }
    if sol.exit == SolveExit::IterationCap {
        return Some(VoidReason::BudgetExhausted);
    }
    // The variational bound is read from `Solution::variational_margin` rather than
    // recomputed here. The solver already has the diagonal for its preconditioner, so
    // recomputing it would cost a second pass AND create a second place for the same rule
    // to live — which is how two copies of a check come to disagree.
    //
    // `margin = min_i H_ii - E`, so a NEGATIVE margin is a solve above the bound. The slack
    // absorbs the rounding in the bound itself: both sides are sums of the same integrals
    // and agree to a few ulps, and a one-determinant space sits exactly ON the bound. It is
    // tiny against the 7.4 hartree the plant misses by, and against the 0.018-0.073 hartree
    // margins correct solves were measured to have.
    if let Some(margin) = sol.variational_margin {
        let slack = 1e-8 * sol.e.v.abs().max(1.0);
        if margin < -slack {
            return Some(VoidReason::AboveLowestDiagonal {
                energy_bits: sol.e.v.to_bits(),
                min_diagonal_bits: (sol.e.v + margin).to_bits(),
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The status code must be injective over the variants — two different statuses that
    /// hashed the same would be a hole in the certificate.
    #[test]
    fn status_codes_are_distinct() {
        let base = NodeRecord {
            node: 0,
            energy_bits: 0,
            d1_bits: 0,
            d2_bits: 0,
            davidson_iters: 0,
            cg_iters: 0,
            exit_code: 0,
            status: NodeStatus::Ok,
            warm: false,
        };
        let statuses = [
            NodeStatus::Ok,
            NodeStatus::Void(VoidReason::BudgetExhausted),
            NodeStatus::Void(VoidReason::AboveLowestDiagonal {
                energy_bits: 1.0f64.to_bits(),
                min_diagonal_bits: 0.0f64.to_bits(),
            }),
            NodeStatus::Void(VoidReason::NotFinite),
        ];
        let codes: Vec<u64> = statuses
            .iter()
            .map(|s| NodeRecord { status: *s, ..base }.status_code())
            .collect();
        let mut sorted = codes.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), codes.len(), "two statuses share a code: {codes:?}");
    }

    /// `exit_code` must be injective too, and it is written out by hand precisely so that
    /// this test guards the hand-written table.
    #[test]
    fn exit_codes_are_distinct() {
        let all = [
            SolveExit::Converged,
            SolveExit::IterationCap,
            SolveExit::Stagnated,
            SolveExit::Trivial,
        ];
        let mut codes: Vec<u8> = all.iter().map(|e| exit_code(*e)).collect();
        let n = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), n, "two SolveExit variants share a code");
    }
}
