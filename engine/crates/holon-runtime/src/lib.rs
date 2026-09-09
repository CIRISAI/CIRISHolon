//! The EXECUTABLE holon (GANTT2, the fourth review, 2026-09-09).
//!
//! `holon-closure` says what a holon IS: a lossy view the dynamics never splits, with a
//! ledger to its neighbours and a rent. It touches no dynamical state (`HolonLayer` labels;
//! `Closure::merge` clears the ledger and rent rather than deriving a law). This crate is the
//! other half: what a holon DOES when it is the thing being integrated instead of the fine
//! state under it. One contract, nine responsibilities, and ONE operator that implements it
//! — oriented rigid water — so the abstraction is grounded before it is general.
//!
//! The rule this crate is built under, from the fourth review: **no operator method lands
//! without rigid water calling it.** A method here that `rigid::RigidWater` does not exercise
//! is a wish.
//!
//! Units are the engine's atomic units throughout: hartree, bohr, electron masses, and the
//! atomic unit of time. Nothing here converts.

pub mod clock;
pub mod rigid;

/// The nine responsibilities of an executable holon, as a contract. `Fine` is the state the
/// operator REPLACES (its children's positions, velocities and masses); `Exchange` is what
/// crosses its boundary per step (site forces for a molecule; fluxes for a cell).
///
/// The contract is deliberately small. Refresh and refinement policy are a runtime's decision
/// made on the operator's [`Validity`] and [`Discarded`] records, not a method the operator
/// implements against itself.
pub trait Operator: Sized {
    type Fine;
    type Exchange;

    /// PROJECT: reduce the fine state to this operator's state, preserving the declared totals
    /// (mass, momentum, angular momentum) and RECORDING what was discarded.
    fn project(fine: &Self::Fine) -> Result<(Self, Discarded), Refusal>;

    /// RECONSTRUCT: a fine state compatible with this operator's state under its DECLARED lift.
    /// After projection has discarded information this cannot promise the original fine
    /// trajectory, and the lift says exactly what it promises instead.
    fn reconstruct(&self) -> Self::Fine;

    /// ADVANCE one step of `dt` under the exchange `now`, evaluating the exchange again at the
    /// end of the step through `eval` (velocity-Verlet's second half-kick needs it). Returns
    /// the exchange at the end of the step so the caller does not evaluate it twice.
    fn advance(&mut self, dt: f64, now: &Self::Exchange, eval: &mut dyn FnMut(&Self) -> Self::Exchange) -> Self::Exchange;

    /// CLOCK: the fastest retained mode under a stiffness envelope, and the step the accuracy
    /// hold admits. The hold is the SAME rule as the fine clock's (`omega * dt <= 2pi / 64`)
    /// fed the retained modes instead of the removed ones — it moves, it is never bypassed.
    fn clock(&self, envelope: &clock::StiffnessEnvelope) -> clock::ClockReading;

    /// The retained degrees of freedom (the thermostat's denominator).
    fn dof(&self) -> usize;
    /// The retained kinetic energy.
    fn kinetic(&self) -> f64;
    /// Conserved totals, for the exchange ledger: mass, momentum, angular momentum about the origin.
    fn totals(&self) -> Totals;
}

/// What a projection threw away, RECORDED. A coarse view is certified by what it discards
/// and over what horizon, never by the totals it keeps (the third review: a closed energy
/// ledger cannot certify a coarse representation).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Discarded {
    /// RMS displacement of the fine sites from their reconstructed positions, bohr: the
    /// deformation the rigid lift cannot represent.
    pub deformation_rms: f64,
    /// Kinetic energy of the fine velocity field that is not a rigid-body motion, hartree:
    /// the internal (vibrational) kinetic energy the operator stops integrating.
    pub internal_kinetic: f64,
}

/// Conserved totals about the lab origin.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Totals {
    pub mass: f64,
    pub momentum: [f64; 3],
    pub angular_momentum: [f64; 3],
}

/// Why an operator refused. Every refusal names its cause; none is a silent fallback.
#[derive(Clone, Debug, PartialEq)]
pub enum Refusal {
    /// The fine state has the wrong number of children for this operator.
    WrongArity { expected: usize, found: usize },
    /// The declared body frame cannot be built from these sites (collinear, coincident).
    DegenerateFrame(String),
    /// A mass is not finite and positive.
    BadMass(usize),
    /// The inertia tensor in the declared frame is not diagonal to tolerance: the declared
    /// frame is not the principal frame, and the free-rotor splitting would be wrong.
    NotPrincipal { off_diagonal: f64, tolerance: f64 },
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refusal::WrongArity { expected, found } => write!(f, "expected {expected} children, found {found}"),
            Refusal::DegenerateFrame(s) => write!(f, "the declared body frame is degenerate: {s}"),
            Refusal::BadMass(i) => write!(f, "mass {i} is not finite and positive"),
            Refusal::NotPrincipal { off_diagonal, tolerance } => {
                write!(f, "the declared frame is not principal: off-diagonal inertia {off_diagonal:e} against {tolerance:e}")
            }
        }
    }
}

/// THE VALIDITY RECORD: what an operator claims, over what domain, at what resolution and
/// horizon, with what evidence. Kept APART from `holon-closure`'s rent on purpose — a refresh
/// budget is a computational quantity and a rent is a physical one, and the two are bound
/// only by a tier with measured coefficients. Sampling evidence is labelled EMPIRICAL and is
/// never the universal hypothesis of a Lean theorem.
#[derive(Clone, Debug, PartialEq)]
pub struct Validity {
    pub operator: &'static str,
    pub version: u32,
    /// The reference (fine) model this operator was validated against, by name and hash.
    pub reference: String,
    /// The conditions under which the claim holds: temperature and density ranges.
    pub temperature_k: (f64, f64),
    pub density_g_cm3: (f64, f64),
    /// Spatial resolution and time horizon the claim covers, in bohr and atomic time.
    pub resolution_bohr: f64,
    pub horizon_au: f64,
    /// Declared error tolerances on the observables the claim bounds, by name.
    pub tolerances: Vec<(String, f64)>,
    pub evidence: Evidence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evidence {
    /// Nothing measured yet: the operator exists and has passed its own invariants only.
    InvariantsOnly,
    /// Measured against the reference on a stated set of windows: EMPIRICAL, bounded by them.
    Empirical,
}

impl Validity {
    /// A record for an operator that has passed its invariants and NOTHING else: every claim
    /// field empty, so a page or a gate cannot read a domain into it.
    pub fn invariants_only(operator: &'static str, version: u32) -> Validity {
        Validity {
            operator,
            version,
            reference: String::new(),
            temperature_k: (f64::NAN, f64::NAN),
            density_g_cm3: (f64::NAN, f64::NAN),
            resolution_bohr: f64::NAN,
            horizon_au: 0.0,
            tolerances: Vec::new(),
            evidence: Evidence::InvariantsOnly,
        }
    }

    /// Does the record admit these conditions? `InvariantsOnly` admits nothing.
    pub fn admits(&self, temperature_k: f64, density_g_cm3: f64, horizon_au: f64) -> bool {
        self.evidence == Evidence::Empirical
            && temperature_k >= self.temperature_k.0
            && temperature_k <= self.temperature_k.1
            && density_g_cm3 >= self.density_g_cm3.0
            && density_g_cm3 <= self.density_g_cm3.1
            && horizon_au <= self.horizon_au
    }

    pub fn json(&self) -> String {
        let num = |x: f64| if x.is_finite() { format!("{x:.9e}") } else { "null".to_string() };
        format!(
            "{{\"operator\": {:?}, \"version\": {}, \"reference\": {:?}, \"temperature_k\": [{}, {}], \"density_g_cm3\": [{}, {}], \"resolution_bohr\": {}, \"horizon_au\": {}, \"tolerances\": [{}], \"evidence\": {:?}}}",
            self.operator,
            self.version,
            self.reference,
            num(self.temperature_k.0),
            num(self.temperature_k.1),
            num(self.density_g_cm3.0),
            num(self.density_g_cm3.1),
            num(self.resolution_bohr),
            num(self.horizon_au),
            self.tolerances.iter().map(|(k, v)| format!("[{k:?}, {}]", num(*v))).collect::<Vec<_>>().join(", "),
            format!("{:?}", self.evidence).to_lowercase()
        )
    }
}

/// THE COST LEDGER: every responsibility counted together, because a speedup that counts
/// advancement and forgets projection, validation and reconstruction is the acuity
/// prototype's mistake in a new costume.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cost {
    pub projections: u64,
    pub reconstructions: u64,
    pub advances: u64,
    pub exchange_evaluations: u64,
    pub validations: u64,
    pub refreshes: u64,
    pub seconds: f64,
}

impl Cost {
    pub fn json(&self) -> String {
        format!(
            "{{\"projections\": {}, \"reconstructions\": {}, \"advances\": {}, \"exchange_evaluations\": {}, \"validations\": {}, \"refreshes\": {}, \"seconds\": {:.6e}}}",
            self.projections, self.reconstructions, self.advances, self.exchange_evaluations, self.validations, self.refreshes, self.seconds
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_invariants_only_record_admits_nothing() {
        let v = Validity::invariants_only("rigid-water", 1);
        assert!(!v.admits(293.0, 0.997, 1.0));
        assert!(v.json().contains("\"evidence\": \"invariantsonly\""));
    }

    #[test]
    fn an_empirical_record_admits_its_domain_and_refuses_outside_it() {
        let mut v = Validity::invariants_only("rigid-water", 1);
        v.evidence = Evidence::Empirical;
        v.temperature_k = (280.0, 310.0);
        v.density_g_cm3 = (0.95, 1.05);
        v.horizon_au = 1000.0;
        assert!(v.admits(293.0, 0.997, 500.0));
        assert!(!v.admits(350.0, 0.997, 500.0), "temperature outside");
        assert!(!v.admits(293.0, 0.997, 5000.0), "horizon beyond the claim");
    }
}
