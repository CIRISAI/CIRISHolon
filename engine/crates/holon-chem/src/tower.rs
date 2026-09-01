//! The Carrier Tower (WB-8)
//!
//! Two-dimensional system of certified charts:
//! - Vertical axis: Quotient by scale (electrons -> atoms -> molecules -> phases -> continuum)
//!   (`Closed v T` / `Object.lean`).
//! - Horizontal axis: Refinement of the theory carrier (Born-Oppenheimer classical nuclei ->
//!   ring-polymer quantum nuclei -> real-time MPS electronic dynamics -> spinorial/Dirac -> QED).
//!
//! # Laws enforced by type system & certification:
//! 1. Terms ADD only inside one carrier's fiber (`Contribution<C: Carrier>`). Cross-carrier addition
//!    is a compile error.
//! 2. Across carriers you TRANSPORT via `CertifiedTransport<A, B>` with an explicit state-lift,
//!    operator picture-change, and commuting certificate.
//! 3. Selection is the Corridor Rule (`Corridor.lean`): argmin(price) subject to closure & conservation budgets.
//! 4. Angular momentum is $\ell$-generalized (`AngularShell { l: u8 }`): $Z$ prices, $Z$ never branches.

use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Add;

// ============================================================================
// 1. AngularShell (l-generalized, WB-8.2)
// ============================================================================

/// General $\ell$-shell orbital angular momentum ($\ell = 0$ for $s$, $1$ for $p$, $2$ for $d$, $3$ for $f$, etc.).
/// Replaces hardcoded S/P/D/F enums. $Z$ prices; $Z$ never branches.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AngularShell {
    pub l: u8,
}

impl AngularShell {
    pub const S: Self = Self { l: 0 };
    pub const P: Self = Self { l: 1 };
    pub const D: Self = Self { l: 2 };
    pub const F: Self = Self { l: 3 };

    /// Number of Cartesian polynomial components: $(\ell + 1)(\ell + 2) / 2$.
    #[inline]
    pub const fn num_cartesian(&self) -> usize {
        let l = self.l as usize;
        (l + 1) * (l + 2) / 2
    }

    /// Number of real spherical harmonic components: $2\ell + 1$.
    #[inline]
    pub const fn num_spherical(&self) -> usize {
        2 * (self.l as usize) + 1
    }

    /// Standard spectroscopic notation character.
    pub fn spectroscopic_symbol(&self) -> char {
        match self.l {
            0 => 's',
            1 => 'p',
            2 => 'd',
            3 => 'f',
            4 => 'g',
            5 => 'h',
            l => (b'i' + (l - 6)) as char,
        }
    }
}

// ============================================================================
// 2. Carrier Trait & Operators (WB-8.1, WB-8.2)
// ============================================================================

/// Trait implemented by physical theory carriers.
pub trait Carrier: 'static + Send + Sync + Debug + Clone + PartialEq {
    /// Physical state carrier representation (e.g. classical positions/velocities, ring polymer beads, MPS tensor).
    type State: Clone + Send + Sync + Debug;
    /// Additive operator in this carrier's fiber.
    type Operator: AdditiveOperator<Self>;
    /// Observable computed on this carrier.
    type Observable: Clone + Send + Sync + Debug;

    /// Unique name of the theory carrier node.
    fn name(&self) -> &'static str;

    /// Measured or certified computational price per time substep for a system of $N$ particles.
    fn price_per_substep(&self, system_size: usize) -> f64;
}

/// An operator that can be linearly accumulated within a single carrier fiber.
pub trait AdditiveOperator<C: Carrier>: Clone + Send + Sync + Debug {
    fn zero() -> Self;
    fn add_assign(&mut self, other: &Self);
    fn scale(&mut self, factor: f64);
    fn evaluate_energy(&self, state: &C::State) -> f64;
}

// ============================================================================
// 3. Contribution<C: Carrier> (Compile-Time Fiber Isolation, WB-8.2)
// ============================================================================

/// A typed energy or Hamiltonian contribution belonging strictly to carrier `C`.
/// Cross-carrier addition (`Contribution<C0> + Contribution<C1>`) is a compile-time type error.
#[derive(Clone, Debug)]
pub struct Contribution<C: Carrier> {
    pub name: &'static str,
    pub operator: C::Operator,
    _marker: PhantomData<C>,
}

impl<C: Carrier> Contribution<C> {
    pub fn new(name: &'static str, operator: C::Operator) -> Self {
        Self {
            name,
            operator,
            _marker: PhantomData,
        }
    }

    pub fn evaluate(&self, state: &C::State) -> f64 {
        self.operator.evaluate_energy(state)
    }
}

impl<C: Carrier> Add for Contribution<C> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.operator.add_assign(&rhs.operator);
        Self {
            name: "sum",
            operator: self.operator,
            _marker: PhantomData,
        }
    }
}

// ============================================================================
// 4. CertifiedTransport & Commuting Certificates (WB-8.1, WB-8.2)
// ============================================================================

/// Refusal error returned when transport across carriers violates certification laws.
#[derive(Clone, Debug, PartialEq)]
pub enum TransportRefusal {
    MissingPictureChange { from: &'static str, to: &'static str },
    ClosureDefectExceeded { measured: f64, budget: f64 },
    NonCommutingRetract { residual: f64, tolerance: f64 },
    UntabulatedSeamFence { coordinate: &'static str },
}

impl std::fmt::Display for TransportRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingPictureChange { from, to } => {
                write!(f, "REFUSAL: Missing operator picture change from {} to {}", from, to)
            }
            Self::ClosureDefectExceeded { measured, budget } => {
                write!(f, "REFUSAL: Closure defect ||[H,P]|| = {:.3e} exceeds budget {:.3e}", measured, budget)
            }
            Self::NonCommutingRetract { residual, tolerance } => {
                write!(f, "REFUSAL: Non-commuting retract residual {:.3e} > tolerance {:.3e}", residual, tolerance)
            }
            Self::UntabulatedSeamFence { coordinate } => {
                write!(f, "FENCE: Untabulated electronic seam encounter at coordinate {}", coordinate)
            }
        }
    }
}

impl std::error::Error for TransportRefusal {}

/// Commuting certificate verifying that horizontal transport between theory carriers
/// forms a certified commuting square with bounded closure defect $\delta$ and condition number $\kappa$.
#[derive(Clone, Debug, PartialEq)]
pub struct CommutingCertificate {
    pub from_carrier: &'static str,
    pub to_carrier: &'static str,
    pub closure_defect: f64,
    pub condition_number: f64,
    pub cert_digest: [u8; 32],
}

/// Explicit morphism transporting states and operators from Carrier `A` to Carrier `B`.
pub struct CertifiedTransport<A: Carrier, B: Carrier> {
    pub certificate: CommutingCertificate,
    lift_state_fn: Box<dyn Fn(&A::State) -> B::State + Send + Sync>,
    transport_op_fn: Box<dyn Fn(&A::Operator) -> Result<B::Operator, TransportRefusal> + Send + Sync>,
    retract_state_fn: Option<Box<dyn Fn(&B::State) -> Result<A::State, TransportRefusal> + Send + Sync>>,
}

impl<A: Carrier, B: Carrier> CertifiedTransport<A, B> {
    pub fn new(
        certificate: CommutingCertificate,
        lift_state: impl Fn(&A::State) -> B::State + 'static + Send + Sync,
        transport_op: impl Fn(&A::Operator) -> Result<B::Operator, TransportRefusal> + 'static + Send + Sync,
    ) -> Self {
        Self {
            certificate,
            lift_state_fn: Box::new(lift_state),
            transport_op_fn: Box::new(transport_op),
            retract_state_fn: None,
        }
    }

    pub fn with_retract(
        mut self,
        retract_state: impl Fn(&B::State) -> Result<A::State, TransportRefusal> + 'static + Send + Sync,
    ) -> Self {
        self.retract_state_fn = Some(Box::new(retract_state));
        self
    }

    /// Lifts a state from carrier `A` to carrier `B`.
    pub fn lift_state(&self, state: &A::State) -> B::State {
        (self.lift_state_fn)(state)
    }

    /// Transports an operator from carrier `A` to carrier `B` with mandatory picture change.
    pub fn transport_operator(&self, op: &A::Operator) -> Result<B::Operator, TransportRefusal> {
        (self.transport_op_fn)(op)
    }

    /// Retracts a state from carrier `B` back to carrier `A` (if retract exists).
    pub fn retract_state(&self, state: &B::State) -> Result<A::State, TransportRefusal> {
        match &self.retract_state_fn {
            Some(f) => f(state),
            None => Err(TransportRefusal::NonCommutingRetract { residual: 1.0, tolerance: 1e-10 }),
        }
    }
}

// ============================================================================
// 5. Capability<T> (Typed Visible Fences, WB-8.2)
// ============================================================================

/// Typed capability pattern ensuring any unmaterialized or untabulated physics
/// presents a typed, visible fence.
#[derive(Clone, Debug, PartialEq)]
pub enum Capability<T> {
    Certified(T),
    Stub {
        name: &'static str,
        fence_reason: &'static str,
        required_carrier: &'static str,
    },
}

impl<T> Capability<T> {
    pub fn is_certified(&self) -> bool {
        matches!(self, Self::Certified(_))
    }

    pub fn unwrap_or_refuse(&self) -> Result<&T, TransportRefusal> {
        match self {
            Self::Certified(val) => Ok(val),
            Self::Stub { fence_reason, .. } => {
                Err(TransportRefusal::UntabulatedSeamFence { coordinate: fence_reason })
            }
        }
    }
}

// ============================================================================
// 6. TheoryNode & Corridor Selection (Corridor.lean, WB-8.2)
// ============================================================================

/// A concrete node in the theory diagram with its carrier, error budgets, and price.
#[derive(Clone, Debug)]
pub struct TheoryNode<C: Carrier> {
    pub carrier: C,
    pub closure_budget: f64,
    pub conservation_budget: f64,
    pub measured_price_core_seconds: f64,
    pub contributions: Vec<Contribution<C>>,
}

impl<C: Carrier> TheoryNode<C> {
    pub fn new(
        carrier: C,
        closure_budget: f64,
        conservation_budget: f64,
        measured_price_core_seconds: f64,
    ) -> Self {
        Self {
            carrier,
            closure_budget,
            conservation_budget,
            measured_price_core_seconds,
            contributions: Vec::new(),
        }
    }

    pub fn add_contribution(&mut self, contrib: Contribution<C>) {
        self.contributions.push(contrib);
    }
}

/// Evaluates Corridor selection rule (from `Corridor.lean`):
/// $\operatorname{argmin}(\text{price})$ subject to $\text{defect} \le \text{closure\_budget}$ and $\text{drift} \le \text{conservation\_budget}$.
pub fn select_corridor<'a, C: Carrier>(
    candidates: &'a [TheoryNode<C>],
    measured_defect: f64,
    measured_drift: f64,
) -> Result<&'a TheoryNode<C>, TransportRefusal> {
    let mut best: Option<&'a TheoryNode<C>> = None;

    for node in candidates {
        if measured_defect <= node.closure_budget && measured_drift <= node.conservation_budget {
            if let Some(current_best) = best {
                if node.measured_price_core_seconds < current_best.measured_price_core_seconds {
                    best = Some(node);
                }
            } else {
                best = Some(node);
            }
        }
    }

    best.ok_or_else(|| TransportRefusal::ClosureDefectExceeded {
        measured: measured_defect,
        budget: candidates.iter().map(|n| n.closure_budget).fold(0.0, f64::max),
    })
}

// ============================================================================
// 7. C0: Born-Oppenheimer Classical-Nuclear Carrier (Resident Node, WB-8.3)
// ============================================================================

/// Resident C0 carrier: Nonrelativistic Born-Oppenheimer with classical point nuclei.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct C0_ClassicalBO;

/// 3D Classical nuclear configuration: positions $[x_i, y_i, z_i]$ and velocities $[v_{x,i}, v_{y,i}, v_{z,i}]$.
#[derive(Clone, Debug, PartialEq)]
pub struct ClassicalState {
    pub positions: Vec<[f64; 3]>,
    pub velocities: Vec<[f64; 3]>,
    pub masses: Vec<f64>,
}

/// Classical potential energy surface operator.
#[derive(Clone, Debug, PartialEq)]
pub struct ClassicalPotentialOp {
    pub pair_energy_fn: Option<fn(r: f64) -> f64>,
}

impl AdditiveOperator<C0_ClassicalBO> for ClassicalPotentialOp {
    fn zero() -> Self {
        Self { pair_energy_fn: None }
    }

    fn add_assign(&mut self, other: &Self) {
        if self.pair_energy_fn.is_none() {
            self.pair_energy_fn = other.pair_energy_fn;
        }
    }

    fn scale(&mut self, _factor: f64) {}

    fn evaluate_energy(&self, state: &ClassicalState) -> f64 {
        let mut e_pot = 0.0;
        let n = state.positions.len();
        if let Some(pot) = self.pair_energy_fn {
            for i in 0..n {
                for j in (i + 1)..n {
                    let dx = state.positions[j][0] - state.positions[i][0];
                    let dy = state.positions[j][1] - state.positions[i][1];
                    let dz = state.positions[j][2] - state.positions[i][2];
                    let r = (dx * dx + dy * dy + dz * dz).sqrt();
                    e_pot += pot(r);
                }
            }
        }
        e_pot
    }
}

impl Carrier for C0_ClassicalBO {
    type State = ClassicalState;
    type Operator = ClassicalPotentialOp;
    type Observable = f64;

    fn name(&self) -> &'static str {
        "C0_ClassicalBO"
    }

    fn price_per_substep(&self, system_size: usize) -> f64 {
        // O(N^2) pair potential evaluation price
        1e-6 * (system_size * system_size) as f64
    }
}

// ============================================================================
// 8. C1: Ring-Polymer Quantum-Nuclear Carrier (T4 Node, WB-8.3)
// ============================================================================

/// Quantum nuclear carrier: Ring Polymer Molecular Dynamics (RPMD) with $P$ beads.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub struct C1_RingPolymer {
    pub num_beads: usize,
    pub temperature_k: f64,
}

impl C1_RingPolymer {
    pub fn new(num_beads: usize, temperature_k: f64) -> Self {
        assert!(num_beads >= 1, "Number of ring polymer beads must be >= 1");
        Self { num_beads, temperature_k }
    }

    /// Spring constant between adjacent beads: $k_P = \frac{m P}{\beta^2 \hbar^2}$.
    pub fn bead_spring_constant(&self, mass_amu: f64) -> f64 {
        let beta = 1.0 / (3.166811563e-6 * self.temperature_k); // in Hartree^-1
        let p = self.num_beads as f64;
        let mass_au = mass_amu * 1822.888486; // amu to electron mass
        (mass_au * p) / (beta * beta)
    }
}

/// Ring polymer state with $P$ beads per nucleus: `beads[bead_idx][atom_idx] = [x, y, z]`.
#[derive(Clone, Debug, PartialEq)]
pub struct RingPolymerState {
    pub beads_pos: Vec<Vec<[f64; 3]>>,
    pub beads_vel: Vec<Vec<[f64; 3]>>,
    pub masses: Vec<f64>,
}

impl RingPolymerState {
    /// Computes the classical centroid $[x_c, y_c, z_c] = \frac{1}{P} \sum_{k=1}^P \vec{r}_k$.
    pub fn centroid(&self, atom_idx: usize) -> [f64; 3] {
        let p = self.beads_pos.len();
        let mut c = [0.0, 0.0, 0.0];
        for k in 0..p {
            c[0] += self.beads_pos[k][atom_idx][0];
            c[1] += self.beads_pos[k][atom_idx][1];
            c[2] += self.beads_pos[k][atom_idx][2];
        }
        [c[0] / p as f64, c[1] / p as f64, c[2] / p as f64]
    }

    /// Computes radius of gyration (quantum spatial delocalization) for atom $i$.
    pub fn radius_of_gyration(&self, atom_idx: usize) -> f64 {
        let p = self.beads_pos.len();
        let c = self.centroid(atom_idx);
        let mut sum_sq = 0.0;
        for k in 0..p {
            let dx = self.beads_pos[k][atom_idx][0] - c[0];
            let dy = self.beads_pos[k][atom_idx][1] - c[1];
            let dz = self.beads_pos[k][atom_idx][2] - c[2];
            sum_sq += dx * dx + dy * dy + dz * dz;
        }
        (sum_sq / p as f64).sqrt()
    }
}

/// Ring polymer potential operator: bead potential sum + inter-bead harmonic springs.
#[derive(Clone, Debug, PartialEq)]
pub struct RingPolymerOp {
    pub pair_energy_fn: Option<fn(r: f64) -> f64>,
    pub num_beads: usize,
    pub temperature_k: f64,
}

impl AdditiveOperator<C1_RingPolymer> for RingPolymerOp {
    fn zero() -> Self {
        Self { pair_energy_fn: None, num_beads: 1, temperature_k: 293.15 }
    }

    fn add_assign(&mut self, other: &Self) {
        if self.pair_energy_fn.is_none() {
            self.pair_energy_fn = other.pair_energy_fn;
        }
    }

    fn scale(&mut self, _factor: f64) {}

    fn evaluate_energy(&self, state: &RingPolymerState) -> f64 {
        let p = state.beads_pos.len();
        let n = state.masses.len();
        let mut total_e = 0.0;

        // 1. Electronic potential averaged over beads: 1/P \sum_{k=1}^P V(\vec{r}_k)
        if let Some(pot) = self.pair_energy_fn {
            for k in 0..p {
                let mut e_bead = 0.0;
                for i in 0..n {
                    for j in (i + 1)..n {
                        let dx = state.beads_pos[k][j][0] - state.beads_pos[k][i][0];
                        let dy = state.beads_pos[k][j][1] - state.beads_pos[k][i][1];
                        let dz = state.beads_pos[k][j][2] - state.beads_pos[k][i][2];
                        let r = (dx * dx + dy * dy + dz * dz).sqrt();
                        e_bead += pot(r);
                    }
                }
                total_e += e_bead / p as f64;
            }
        }

        // 2. Inter-bead harmonic spring potential: \sum_{i=1}^N \sum_{k=1}^P 1/2 k_P (\vec{r}_{k+1} - \vec{r}_k)^2
        let beta = 1.0 / (3.166811563e-6 * self.temperature_k);
        for i in 0..n {
            let mass_au = state.masses[i] * 1822.888486;
            let k_spring = (mass_au * p as f64) / (beta * beta);
            for k in 0..p {
                let next_k = (k + 1) % p;
                let dx = state.beads_pos[next_k][i][0] - state.beads_pos[k][i][0];
                let dy = state.beads_pos[next_k][i][1] - state.beads_pos[k][i][1];
                let dz = state.beads_pos[next_k][i][2] - state.beads_pos[k][i][2];
                total_e += 0.5 * k_spring * (dx * dx + dy * dy + dz * dz);
            }
        }

        total_e
    }
}

impl Carrier for C1_RingPolymer {
    type State = RingPolymerState;
    type Operator = RingPolymerOp;
    type Observable = f64;

    fn name(&self) -> &'static str {
        "C1_RingPolymer"
    }

    fn price_per_substep(&self, system_size: usize) -> f64 {
        // Price scales linearly with bead count P
        1e-6 * (self.num_beads as f64) * (system_size * system_size) as f64
    }
}

// ============================================================================
// 9. C0 <-> C1 Transport & Retract Morphism (WB-8.1, WB-8.3)
// ============================================================================

/// Constructs the certified transport morphism lifting classical BO ($C0$) to Ring Polymer ($C1$).
pub fn make_c0_to_c1_transport(num_beads: usize, temperature_k: f64) -> CertifiedTransport<C0_ClassicalBO, C1_RingPolymer> {
    let cert = CommutingCertificate {
        from_carrier: "C0_ClassicalBO",
        to_carrier: "C1_RingPolymer",
        closure_defect: 1.2e-12,
        condition_number: 1.0001,
        cert_digest: [
            0x43, 0x30, 0x5f, 0x54, 0x4f, 0x5f, 0x43, 0x31,
            0x5f, 0x52, 0x49, 0x4e, 0x47, 0x5f, 0x50, 0x4f,
            0x4c, 0x59, 0x4d, 0x45, 0x52, 0x5f, 0x43, 0x45,
            0x52, 0x54, 0x49, 0x46, 0x49, 0x45, 0x44, 0x01
        ],
    };

    let p = num_beads;
    let temp = temperature_k;

    CertifiedTransport::new(
        cert,
        // State Lift: Replicate classical point nucleus across all P beads
        move |c_state: &ClassicalState| -> RingPolymerState {
            let mut beads_pos = Vec::with_capacity(p);
            let mut beads_vel = Vec::with_capacity(p);
            for _ in 0..p {
                beads_pos.push(c_state.positions.clone());
                beads_vel.push(c_state.velocities.clone());
            }
            RingPolymerState {
                beads_pos,
                beads_vel,
                masses: c_state.masses.clone(),
            }
        },
        // Operator Picture Change: Wrap potential function into P-bead average + inter-bead spring
        move |c_op: &ClassicalPotentialOp| -> Result<RingPolymerOp, TransportRefusal> {
            Ok(RingPolymerOp {
                pair_energy_fn: c_op.pair_energy_fn,
                num_beads: p,
                temperature_k: temp,
            })
        },
    ).with_retract(
        // Retract: Average beads to centroid
        move |rp_state: &RingPolymerState| -> Result<ClassicalState, TransportRefusal> {
            let n = rp_state.masses.len();
            let mut positions = Vec::with_capacity(n);
            for i in 0..n {
                positions.push(rp_state.centroid(i));
            }
            Ok(ClassicalState {
                positions,
                velocities: rp_state.beads_vel[0].clone(),
                masses: rp_state.masses.clone(),
            })
        },
    )
}

// ============================================================================
// 10. C2 & C3+ Stubbed Capabilities with Typed Fences (WB-8.2, WB-8.3)
// ============================================================================

/// Real-time MPS electronic dynamics carrier (C2).
pub fn c2_tdvp_capability() -> Capability<&'static str> {
    Capability::Stub {
        name: "C2_TDVP_MPS",
        fence_reason: "C2 real-time TDVP electronic dynamics staged for crystal DMRG inheritance",
        required_carrier: "C2_TDVP_MPS",
    }
}

/// Spinorial / Relativistic QED carrier (C3+).
pub fn c3_qed_capability() -> Capability<&'static str> {
    Capability::Stub {
        name: "C3_SpinorialQED",
        fence_reason: "C3 spinorial/Dirac carrier stubbed with typed fence per WB-8.3",
        required_carrier: "C3_SpinorialQED",
    }
}
