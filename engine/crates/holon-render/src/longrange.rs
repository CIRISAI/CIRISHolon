//! THE LONG-RANGE PAIR SECTOR (GANTT node B2) — the tail the cutoff cannot reach.
//!
//! Frozen design: `conformance/water_observatory/B2_PREREG.md`. Read that first; this
//! module is its instrument and nothing here may move a threshold it staked.
//!
//! # Why this is not Ewald
//!
//! B1b fired this node by measuring that a truncation at the engine's own cell-list radius
//! discards up to 1.15e-5 Ha per frame on oxygen-bearing scenes and fails the incurred-drift
//! criterion on 3 of 8 seeds. The obvious response — an Ewald summation — is the wrong
//! instrument for this force law, and the reason is structural rather than a preference.
//!
//! Ewald, PME and Wolf-style damped sums all exist to evaluate a **conditionally
//! convergent** `Σ 1/r` lattice sum: the reciprocal-space split, the neutralising
//! background and the surface term are there because that sum has no absolutely convergent
//! value and its answer depends on the order the images are added in. **`Sim` has no such
//! term.** Its force law is the tabulated pair curve, the tabulated three- and four-body
//! surfaces, the walls, the uniform field and the user's spring; nuclear charge appears
//! only as a species label resolved into a bank slot, and the only `q_a q_b / r` in the
//! tree lives within the electronic-structure solver that GENERATES the curves.
//!
//! So the sum this module performs is over a kernel that decays as a power `p`, and on a
//! `d`-dimensional image lattice such a sum converges absolutely exactly when `p > d`.
//! That condition is the licence for the whole design and it is CHECKED rather than
//! assumed: [`TailFit`] measures `p` from the table's own knots, and
//! [`FarRefusal::ExponentTooShallow`] refuses the `p ≤ d` case by name — which is where an
//! ionic `r⁻¹` scene lands, in 2D and in 3D alike — naming Ewald or PME as the exit rather
//! than summing it with an argument that does not cover it.
//!
//! # The two channels
//!
//! What a truncation at `c*` discards is not one thing. Pairs with `c* < r ≤ r_max` are
//! REAL tabulated interaction, thrown away because the cell-list radius is set by a
//! three-body table while the pair curve reaches further; pairs past `r_max` meet the
//! table's exponential extrapolation standing in for an unknown true tail. The first is a
//! radius-bookkeeping defect and [`FarRefusal::SubSupport`] makes it unrepresentable. Only
//! the second is a long-range METHOD question, and it is the one this module answers.
//!
//! # What the tail model is, and is not
//!
//! `u_far(r) = −C_p · r^(−p)` past `R_s`, with `p` MEASURED from the curve's last knots and
//! `C_p` fixed by matching `|u|` at `R_s` — one constant, determined, not fitted. The
//! curves are FCI in a minimal basis, which underestimates dispersion badly, so `C_p` is a
//! MODEL quantity and never a physical dispersion coefficient. When the measured exponent
//! lands outside the adopting band the sector emits a BRACKET (the exponential at one end,
//! the power law at the other) and refuses to hand anyone a scalar.

use crate::cells::BoxGeom;
use crate::sim::Dims;

/// Which side of B2's G3 band a curve's measured tail sits on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TailBand {
    /// `p_fit ∈ [P_FIT_LO, P_FIT_HI]` and the exponential's local index agrees within
    /// [`EXP_INDEX_FACTOR`]: the curve has reached a power-law asymptote inside its own
    /// support and the power-law tail may be ADOPTED.
    Adopting,
    /// Anything else: the curve has NOT been shown to reach its asymptote where the table
    /// ends, so the power-law tail is an extrapolation beyond measurement. The sector then
    /// carries a bracket rather than a number, and the exit is a longer curve or a basis
    /// with diffuse functions (`ION_STAKING.md` I-5, unowned).
    Fenced,
}

/// G3's adopting band, lower edge. Staked in the freeze, never fitted.
pub const P_FIT_LO: f64 = 5.0;
/// G3's adopting band, upper edge.
pub const P_FIT_HI: f64 = 7.0;
/// How far the exponential's local index `hi_b · r_max` may exceed the fitted power before
/// the curve is fenced. A genuinely power-law tail has them EQUAL — for `u ∼ r^(−p)` the
/// logarithmic derivative is exactly `p/r` — so this factor is slack around an identity,
/// not a tolerance around a guess.
pub const EXP_INDEX_FACTOR: f64 = 3.0;
/// Fraction of a curve's knots the exponent is fitted over. The LAST tenth: the asymptote
/// is what is being measured, and including the well would fit the bond instead.
pub const FIT_FRACTION: f64 = 0.10;
/// Fewest knots a fit will run on. Two points determine a line and say nothing about
/// whether it is one, so a residual needs a third.
pub const FIT_MIN_KNOTS: usize = 3;
/// G10's staked shell cap. Reaching it without meeting the budget is R2, not a pass.
pub const SHELL_CAP: usize = 8;

/// The measured tail of one curve — G3's reading, per bank slot.
#[derive(Clone, Copy, Debug)]
pub struct TailFit {
    /// `−d ln|u| / d ln r`, least squares over the last [`FIT_FRACTION`] of knots.
    pub p_fit: f64,
    /// RMS residual of that fit in `ln|u|`, so a curve that is not a power law over the
    /// window says so rather than returning a slope with no error bar.
    pub residual: f64,
    /// The exponential extrapolation's own local index at the last knot, `hi_b · r_max`.
    /// Equal to `p_fit` for a true power law; far above it for an exchange-dominated tail.
    pub exp_index: f64,
    pub band: TailBand,
    /// The last knot's radius, bohr — the support this curve actually carries.
    pub r_max: f64,
    /// `|u(r_max)|`, hartree.
    pub u_at_max: f64,
    pub knots_fitted: usize,
}

/// The adopted tail for one slot, with the provenance the disclosure law requires beside
/// every solver-derived number.
///
/// `solver_exit`, `solver_budget_iterations` and `uncertainty_hartree` are NOT decoration.
/// B1b banked that the O–O curve exits `IterationCap` at 5000 iterations with worst
/// residual 4.809e-6 Ha — four orders above `CONVERGED_RESIDUAL` — so every constant
/// fitted from it inherits that, and a capped residual is not monotone in effort. A number
/// without its budget is not a number, and [`FarRefusal::UndisclosedSolve`] is the refusal.
#[derive(Clone, Debug)]
pub struct TailModel {
    pub p: f64,
    /// `−u(R_s) · R_s^p`, so `u_far(R_s) == u(R_s)` exactly by construction.
    pub c_p: f64,
    pub r_s: f64,
    pub fit: TailFit,
    pub solver_exit: &'static str,
    pub solver_budget_iterations: u64,
    pub uncertainty_hartree: f64,
}

impl TailModel {
    /// `u_far(r)` and `du_far/dr`, from ONE expression. G8 checks the arithmetic; it is
    /// not checking two independent implementations against each other, because there is
    /// only one.
    #[inline]
    pub fn eval(&self, r: f64) -> (f64, f64) {
        let inv = r.powf(-self.p);
        (-self.c_p * inv, self.p * self.c_p * inv / r)
    }

    /// The radius at which one pair's far energy falls under `budget`. Solved in closed
    /// form because the kernel is a pure power; `Sim::derive_pair_cutoff` bisects because
    /// its kernel is an interpolant, and the two agree in what they mean by a budget.
    pub fn radius_for_budget(&self, budget: f64) -> f64 {
        if !(budget > 0.0) || self.c_p == 0.0 {
            return self.r_s;
        }
        (self.c_p.abs() / budget).powf(1.0 / self.p).max(self.r_s)
    }
}

/// Why the far sector declined to produce a number. Every variant carries what a reader
/// needs to act, and — where the fence law demands it — the exit by name.
#[derive(Clone, Debug, PartialEq)]
pub enum FarRefusal {
    /// R3. The near radius sits inside a loaded curve's support, so real tabulated
    /// interaction would be handed to a tail model that is not entitled to it. This is
    /// B1b's measured defect, made unrepresentable: it would have fired on the audited
    /// configuration at `r_s = 15.0` against `r_max = 20.0`.
    SubSupport { slot: usize, r_s: f64, r_max: f64 },
    /// R1. The kernel decays too slowly for its own image lattice to converge absolutely.
    /// `p ≤ d` is exactly the ionic `r⁻¹` case, and the exit is Ewald or PME on a force law
    /// that has charge in it — node C's, not this one's.
    ExponentTooShallow { p: f64, d: usize, exit: &'static str },
    /// R1's other prong: a scene declaring a point charge at all. This engine's force law
    /// has no electrostatic term, so such a scene is not one B2 can price.
    ChargedScene { charge: f64, exit: &'static str },
    /// R2. The image shells did not converge to the declared budget by [`SHELL_CAP`].
    /// Accepting the last shell anyway would replace a declared budget with an undeclared
    /// one.
    ImageBudget { achieved: f64, budget: f64, cap: usize },
    /// R4. G3 fenced a curve this scene needs, so the far energy exists only as a bracket
    /// and a caller asking for a scalar gets this instead.
    FencedTailScalar { lo: f64, hi: f64, factor: f64 },
    /// R5. A tail parameter reached a manifest without its solve's exit and budget.
    UndisclosedSolve { slot: usize, missing: &'static str },
    /// No curve is loaded for a slot the scene needs, so there is nothing to fit.
    NoCurve { slot: usize },
}

impl core::fmt::Display for FarRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FarRefusal::SubSupport { slot, r_s, r_max } => write!(
                f,
                "REFUSED (R3, sub-support): slot {slot} has r_max = {r_max} bohr but the \
                 near radius is R_s = {r_s} bohr — {:.4} bohr of tabulated interaction \
                 would be handed to a tail model. Raise R_s to at least r_max.",
                r_max - r_s
            ),
            FarRefusal::ExponentTooShallow { p, d, exit } => write!(
                f,
                "REFUSED (R1, exponent): measured tail exponent p = {p:.4} against scene \
                 dimension d = {d}; the image lattice converges absolutely only for p > d. \
                 EXIT: {exit}"
            ),
            FarRefusal::ChargedScene { charge, exit } => write!(
                f,
                "REFUSED (R1, charge): scene declares a point charge of {charge}, and this \
                 engine's force law carries no electrostatic term. EXIT: {exit}"
            ),
            FarRefusal::ImageBudget { achieved, budget, cap } => write!(
                f,
                "REFUSED (R2, image budget): shell-to-shell difference {achieved:.6e} Ha \
                 still above the declared budget {budget:.6e} Ha at the staked cap of \
                 {cap} shells."
            ),
            FarRefusal::FencedTailScalar { lo, hi, factor } => write!(
                f,
                "REFUSED (R4, fenced tail): the far energy is a BRACKET \
                 [{lo:.6e}, {hi:.6e}] Ha spanning a factor of {factor:.3e}; no scalar is \
                 available. EXIT: a curve carrying further support, or a basis with diffuse \
                 functions (ION_STAKING.md I-5, unowned)."
            ),
            FarRefusal::UndisclosedSolve { slot, missing } => write!(
                f,
                "REFUSED (R5, disclosure): slot {slot}'s tail parameter carries no \
                 {missing}. A capped residual is not monotone in effort."
            ),
            FarRefusal::NoCurve { slot } => {
                write!(f, "REFUSED: no loaded curve for slot {slot}; nothing to fit.")
            }
        }
    }
}

/// The mutations B2's plant battery installs. Each is a REAL defect, not a simulated one:
/// the sector genuinely computes the wrong thing while a plant is set, which is the only
/// way a gate's ability to see it can be demonstrated rather than asserted.
///
/// Each plant's carrier and the sector it must be nonzero in are stated in the freeze's §7;
/// [`FarReading::plant_carrier`] is what the instrument checks before trusting any gate,
/// and a carrier reading 0.0 REFUSES the plant instead of scoring it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FarPlant {
    /// P1, against G9: cache the image offsets and skip recomputation when the box moves.
    StaleLattice,
    /// P2, against G5: apply the far force to `i` and nothing to `j`.
    OneSidedForce,
    /// P3, against G6: rotate the far force in the scene plane, keeping it equal and
    /// opposite. Linear momentum still cancels exactly; angular momentum does not.
    NonCentralForce,
    /// P4, against G4, in its SECOND form. A bare constant added to `u_far` shifts `l0`
    /// and `rebase` absorbs it, so it is invisible to the drift gate — that is
    /// M-PLANT-OBS, and it is why the plant is a STEP at `R_s` instead.
    ZeroPointStep,
    /// P5, against G8: scale the far force by 1.001, leaving the energy alone.
    GradientMismatch,
    /// P6, against G1 and G14: sum the far sector only to `R_s + 0.1` bohr.
    TruncatedFarSum,
    /// P7, against G7: post the far energy and not its virial.
    OmittedVirial,
}

/// The constant P4 steps the far energy by, hartree. Large against the far energies it
/// perturbs and small against the well, so a crossing is visible without changing what the
/// trajectory does.
pub const PLANT_STEP_HARTREE: f64 = 1.0e-6;
/// P3's rotation, radians.
pub const PLANT_ROTATION_RAD: f64 = 1.0e-3;
/// P5's force scaling.
pub const PLANT_FORCE_SCALE: f64 = 1.001;
/// P6's truncation, bohr past `R_s`.
pub const PLANT_TRUNCATION_BOHR: f64 = 0.1;

/// The cache key for everything the far sector derives from the box.
///
/// The KIND of box is in the key, not beside it: M-CACHE-KIND's lesson is that when
/// record kinds share a namespace, existence stands in for certification. A mismatch on
/// read recomputes; P1 is what happens when it does not.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BoxKey {
    pub lx: f64,
    pub ly: f64,
    pub lz: f64,
    pub periodic: bool,
    pub three_d: bool,
    pub shells: usize,
}

impl BoxKey {
    pub fn of(geom: BoxGeom, dims: Dims, shells: usize) -> Self {
        Self {
            lx: geom.lx,
            ly: geom.ly,
            lz: geom.lz,
            periodic: geom.periodic,
            three_d: dims == Dims::Three,
            shells,
        }
    }
}

/// What one far-sector pass computed, and everything a gate needs to judge it.
#[derive(Clone, Copy, Debug, Default)]
pub struct FarReading {
    /// The far sector's energy row, hartree.
    pub energy: f64,
    /// `Σ r · du_far/dr` over every far contribution — the virial this channel owes the
    /// pressure. A channel present in the force law that does not post here reads to the
    /// barostat as a missing pressure, which is P7.
    pub virial: f64,
    /// Far pair contributions evaluated, minimum image and images together. G12's floor.
    pub contributions: u64,
    /// Contributions from a nonzero image offset. Exactly zero in a non-wrapping box, and
    /// that zero is a fact about the SCENE rather than about this instrument's coverage.
    pub image_contributions: u64,
    /// Channel S: pairs with `c* < r ≤ r_max`, the sub-support band. G1's first half.
    pub channel_s: f64,
    /// Channel T: pairs past `r_max`. G1's second half.
    pub channel_t: f64,
    /// Pairs whose minimum-image separation crossed `R_s` since the previous pass. P4 is
    /// vacuous without one, so G12 floors it.
    pub crossings: u64,
    /// The rigorous residual past `R_f`: `(N(N−1)/2)·|u_far(R_f)|`, assuming only that the
    /// kernel is monotone there. NOT a density estimate — M-HOMOG forbids one on a
    /// 12-atom box with no bulk.
    pub residual_bound: f64,
    /// Whichever plant was installed for this pass moved by this much in its own sector.
    /// The instrument REFUSES to score a plant whose carrier reads 0.0.
    pub plant_carrier: f64,
}

/// THE LONG-RANGE PAIR SECTOR.
pub struct FarSector {
    /// One model per bank slot; `None` where no curve is loaded or the fit was refused.
    models: Vec<Option<TailModel>>,
    r_s: f64,
    r_f: f64,
    budget: f64,
    dims: Dims,
    /// Any curve this scene needs landed in [`TailBand::Fenced`], so the energy is a
    /// bracket and R4 refuses a scalar.
    fenced: bool,
    /// The image offsets, and the key they were built for. Recomputed whenever the key
    /// moves — unless [`FarPlant::StaleLattice`] is installed, which is P1.
    offsets: Vec<(f64, f64, f64)>,
    key: Option<BoxKey>,
    shells: usize,
    pub plant: Option<FarPlant>,
    /// P4's step, hartree. [`PLANT_STEP_HARTREE`] is the STAKED value and is what the
    /// plant runs at; the field exists so a POWER CERTIFICATE can sweep it — the smallest
    /// step a gate can resolve is a property of the gate, and B2's G4 turned out not to
    /// resolve the staked one. Sweeping it is a measurement OF the gate, never a retune of
    /// it: the plant's verdict is always reported at the staked value.
    pub plant_step: f64,
    /// Previous pass's `r > R_s` flags, so `crossings` counts transitions rather than
    /// occupancy. Keyed by packed pair index.
    prev_beyond: Vec<bool>,
}

impl FarSector {
    /// Build the sector from measured tails.
    ///
    /// `curves` supplies, per slot, the knot radii and energies plus the exponential
    /// extrapolation's index — read from the table, never supplied by a caller. `budget` is
    /// the largest energy one far pair may carry at `R_f`, the same declaration
    /// `Sim::set_pair_cutoff` takes.
    ///
    /// REFUSES rather than rounding: `R_s` below any needed curve's support is R3, and a
    /// measured exponent at or below the scene's dimension is R1.
    pub fn build(
        curves: &[Option<CurveTail>],
        r_s: f64,
        budget: f64,
        dims: Dims,
    ) -> Result<Self, FarRefusal> {
        let d = if dims == Dims::Three { 3 } else { 2 };
        let mut models: Vec<Option<TailModel>> = Vec::with_capacity(curves.len());
        let mut fenced = false;
        let mut r_f = r_s;
        for (slot, c) in curves.iter().enumerate() {
            let Some(c) = c else {
                models.push(None);
                continue;
            };
            if r_s < c.r_max() {
                return Err(FarRefusal::SubSupport {
                    slot,
                    r_s,
                    r_max: c.r_max(),
                });
            }
            let fit = c.fit();
            if fit.p_fit <= d as f64 {
                return Err(FarRefusal::ExponentTooShallow {
                    p: fit.p_fit,
                    d,
                    exit: "Ewald or PME, on a force law carrying charge (GANTT node C)",
                });
            }
            if fit.band == TailBand::Fenced {
                fenced = true;
            }
            // The match is at R_s, so `u_far(R_s) == u(R_s)` exactly. Past r_max the table
            // itself is only an exponential extrapolation, so `u_at(R_s)` is read through
            // the same extrapolation the engine already uses — this model REPLACES that
            // extrapolation past R_s and agrees with it at the seam.
            let u_s = c.u_at(r_s);
            let model = TailModel {
                p: fit.p_fit,
                c_p: -u_s * r_s.powf(fit.p_fit),
                r_s,
                fit,
                solver_exit: c.solver_exit,
                solver_budget_iterations: c.solver_budget_iterations,
                uncertainty_hartree: c.uncertainty_hartree,
            };
            if model.solver_exit.is_empty() {
                return Err(FarRefusal::UndisclosedSolve {
                    slot,
                    missing: "solver_exit",
                });
            }
            if model.solver_budget_iterations == 0 {
                return Err(FarRefusal::UndisclosedSolve {
                    slot,
                    missing: "solver_budget_iterations",
                });
            }
            r_f = r_f.max(model.radius_for_budget(budget));
            models.push(Some(model));
        }
        Ok(Self {
            models,
            r_s,
            r_f,
            budget,
            dims,
            fenced,
            offsets: Vec::new(),
            key: None,
            shells: 0,
            plant: None,
            plant_step: PLANT_STEP_HARTREE,
            prev_beyond: Vec::new(),
        })
    }

    pub fn r_s(&self) -> f64 {
        self.r_s
    }
    pub fn r_f(&self) -> f64 {
        self.r_f
    }
    pub fn budget(&self) -> f64 {
        self.budget
    }
    pub fn shells(&self) -> usize {
        self.shells
    }
    pub fn is_fenced(&self) -> bool {
        self.fenced
    }
    pub fn model(&self, slot: usize) -> Option<&TailModel> {
        self.models.get(slot).and_then(|m| m.as_ref())
    }

    /// R4: a caller wanting one number from a fenced sector gets the bracket and a refusal.
    pub fn scalar_ok(&self, lo: f64, hi: f64) -> Result<(), FarRefusal> {
        if !self.fenced {
            return Ok(());
        }
        let factor = if lo.abs() > 0.0 {
            (hi / lo).abs()
        } else {
            f64::INFINITY
        };
        Err(FarRefusal::FencedTailScalar { lo, hi, factor })
    }

    /// R1's charge prong. This engine has no charge in its force law, so a scene declaring
    /// one is refused rather than summed by a method that has never seen one.
    pub fn admit_charge(charge: f64) -> Result<(), FarRefusal> {
        if charge == 0.0 {
            Ok(())
        } else {
            Err(FarRefusal::ChargedScene {
                charge,
                exit: "Ewald or PME, on a force law carrying charge (GANTT node C)",
            })
        }
    }

    /// How many image shells this box needs to meet the budget, and the difference it
    /// achieved. G10 reads both; R2 refuses at [`SHELL_CAP`].
    ///
    /// Non-wrapping boxes need none, and that zero is a fact about the scene.
    pub fn resolve_shells(
        &mut self,
        atoms: &[(f64, f64, f64)],
        slots: &[usize],
        geom: BoxGeom,
    ) -> Result<(usize, f64), FarRefusal> {
        if !geom.periodic {
            self.set_shells(0, geom);
            return Ok((0, 0.0));
        }
        let mut prev = self.energy_at_shells(atoms, slots, geom, 0);
        for m in 1..=SHELL_CAP {
            let now = self.energy_at_shells(atoms, slots, geom, m);
            let diff = (now - prev).abs();
            if diff < self.budget {
                self.set_shells(m, geom);
                return Ok((m, diff));
            }
            prev = now;
        }
        Err(FarRefusal::ImageBudget {
            achieved: (self.energy_at_shells(atoms, slots, geom, SHELL_CAP)
                - self.energy_at_shells(atoms, slots, geom, SHELL_CAP - 1))
            .abs(),
            budget: self.budget,
            cap: SHELL_CAP,
        })
    }

    fn set_shells(&mut self, m: usize, geom: BoxGeom) {
        self.shells = m;
        let key = BoxKey::of(geom, self.dims, m);
        self.rebuild_offsets(key);
    }

    /// Energy only, at a stated shell count — G10's probe and nothing else. Deliberately
    /// separate from [`FarSector::accumulate`] so that measuring convergence cannot
    /// disturb the forces or the crossing counter.
    pub fn energy_at_shells(
        &self,
        atoms: &[(f64, f64, f64)],
        slots: &[usize],
        geom: BoxGeom,
        shells: usize,
    ) -> f64 {
        let offsets = Self::offsets_for(BoxKey::of(geom, self.dims, shells));
        let r_f = self.effective_r_f();
        let mut e = 0.0;
        let n = atoms.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let Some(m) = self.model_for(slots, i, j) else {
                    continue;
                };
                let (dx, dy, dz) = geom.delta(atoms[i], atoms[j]);
                for (k, o) in core::iter::once(&(0.0, 0.0, 0.0)).chain(offsets.iter()).enumerate() {
                    let (sx, sy, sz) = (dx + o.0, dy + o.1, dz + o.2);
                    let r = (sx * sx + sy * sy + sz * sz).sqrt();
                    if !(r > self.r_s) || r > r_f {
                        continue;
                    }
                    let _ = k;
                    e += m.eval(r).0;
                }
            }
            // Self-images: an atom interacts with its own periodic copies. Halved because
            // the pair (i, image) and (image, i) are the same interaction seen twice.
            if !offsets.is_empty() {
                if let Some(m) = self.model_for(slots, i, i) {
                    for o in offsets.iter() {
                        let r = (o.0 * o.0 + o.1 * o.1 + o.2 * o.2).sqrt();
                        if !(r > self.r_s) || r > r_f {
                            continue;
                        }
                        e += 0.5 * m.eval(r).0;
                    }
                }
            }
        }
        e
    }

    /// THE FORCE PASS. Accumulates the far sector's energy, forces and virial, and every
    /// quantity B2's gates read.
    ///
    /// Forces land in `a_pair` — the INTERNAL column, the one that cancels from the
    /// momentum sum — because the far term is a conservative pairwise interaction like the
    /// near one and not a boundary condition.
    pub fn accumulate(
        &mut self,
        atoms: &[(f64, f64, f64)],
        slots: &[usize],
        geom: BoxGeom,
        forces: &mut [(f64, f64, f64)],
        r_max_by_slot: &[f64],
    ) -> FarReading {
        let key = BoxKey::of(geom, self.dims, self.shells);
        // M-CACHE-KIND: the key carries the KIND of box, and a mismatch recomputes. P1 is
        // exactly the branch that does not.
        if self.plant != Some(FarPlant::StaleLattice) && self.key != Some(key) {
            self.rebuild_offsets(key);
        }
        let r_f = self.effective_r_f();
        let n = atoms.len();
        let mut out = FarReading::default();
        let pairs = n * (n + 1) / 2;
        if self.prev_beyond.len() != pairs {
            self.prev_beyond = vec![false; pairs];
        }
        let (sin_t, cos_t) = if self.plant == Some(FarPlant::NonCentralForce) {
            (PLANT_ROTATION_RAD.sin(), PLANT_ROTATION_RAD.cos())
        } else {
            (0.0, 1.0)
        };
        let fscale = if self.plant == Some(FarPlant::GradientMismatch) {
            PLANT_FORCE_SCALE
        } else {
            1.0
        };
        let step = if self.plant == Some(FarPlant::ZeroPointStep) {
            self.plant_step
        } else {
            0.0
        };

        for i in 0..n {
            for j in (i + 1)..n {
                let Some(m) = self.model_for(slots, i, j).cloned() else {
                    continue;
                };
                let (dx, dy, dz) = geom.delta(atoms[i], atoms[j]);
                let r0 = (dx * dx + dy * dy + dz * dz).sqrt();
                let idx = i * n + j - i * (i + 1) / 2;
                let beyond = r0 > self.r_s;
                if beyond != self.prev_beyond[idx] {
                    out.crossings += 1;
                }
                self.prev_beyond[idx] = beyond;
                let r_max = Self::slot_of(slots, i, j)
                    .and_then(|s| r_max_by_slot.get(s).copied())
                    .unwrap_or(0.0);

                for (which, o) in core::iter::once(&(0.0, 0.0, 0.0))
                    .chain(self.offsets.iter())
                    .enumerate()
                {
                    let (sx, sy, sz) = (dx + o.0, dy + o.1, dz + o.2);
                    let r = (sx * sx + sy * sy + sz * sz).sqrt();
                    if !(r > self.r_s) || r > r_f {
                        continue;
                    }
                    let (u, du) = m.eval(r);
                    out.energy += u + step;
                    if self.plant != Some(FarPlant::OmittedVirial) {
                        out.virial += r * du;
                    }
                    out.contributions += 1;
                    if which > 0 {
                        out.image_contributions += 1;
                    } else if r <= r_max {
                        // Channel S is defined on the MINIMUM IMAGE separation only: an
                        // image copy of a sub-support pair is a different pair at a
                        // different distance, and folding it in would let the two channels
                        // borrow from each other.
                        out.channel_s += u;
                    } else {
                        out.channel_t += u;
                    }
                    let f_over_r = du / r;
                    let (mut fx, mut fy, fz) =
                        (f_over_r * sx * fscale, f_over_r * sy * fscale, f_over_r * sz * fscale);
                    if sin_t != 0.0 {
                        let (rx, ry) = (cos_t * fx - sin_t * fy, sin_t * fx + cos_t * fy);
                        fx = rx;
                        fy = ry;
                    }
                    forces[i].0 += fx;
                    forces[i].1 += fy;
                    forces[i].2 += fz;
                    if self.plant != Some(FarPlant::OneSidedForce) {
                        forces[j].0 -= fx;
                        forces[j].1 -= fy;
                        forces[j].2 -= fz;
                    }
                }
            }
            if !self.offsets.is_empty() {
                if let Some(m) = self.model_for(slots, i, i).cloned() {
                    // `take` and put back rather than `self.offsets.clone()`: the clone was
                    // one heap allocation PER ATOM PER FORCE PASS, which is a cost that
                    // grows with N and would have shown up in G13's curve as the far sum's
                    // scaling when it is the allocator's.
                    let offs = core::mem::take(&mut self.offsets);
                    for o in offs.iter() {
                        let r = (o.0 * o.0 + o.1 * o.1 + o.2 * o.2).sqrt();
                        if !(r > self.r_s) || r > r_f {
                            continue;
                        }
                        let (u, du) = m.eval(r);
                        out.energy += 0.5 * u;
                        if self.plant != Some(FarPlant::OmittedVirial) {
                            out.virial += 0.5 * r * du;
                        }
                        out.contributions += 1;
                        out.image_contributions += 1;
                        out.channel_t += 0.5 * u;
                        // No force: the self-image contributions come in ± pairs from
                        // opposite lattice offsets and cancel identically. Stated rather
                        // than silently omitted.
                    }
                    self.offsets = offs;
                }
            }
        }
        out.residual_bound = self.residual_bound(n, r_f);
        out
    }

    /// The far residual past `R_f`, bounded with NO assumption about how the atoms are
    /// arranged: every pair, at worst, carries `|u_far(R_f)|`.
    ///
    /// The standard isotropic tail integral would be far tighter and is NOT used, because
    /// it needs `g(r) → 1` past the cutoff and a bulk density — and this campaign's scenes
    /// are 12 atoms in a walled box with no bulk (M-HOMOG). A bound that assumes what the
    /// scene does not have is not a bound.
    pub fn residual_bound(&self, n: usize, r_f: f64) -> f64 {
        let mut worst = 0.0f64;
        for m in self.models.iter().flatten() {
            worst = worst.max(m.eval(r_f).0.abs());
        }
        (n as f64) * ((n as f64) - 1.0) * 0.5 * worst
    }

    fn effective_r_f(&self) -> f64 {
        if self.plant == Some(FarPlant::TruncatedFarSum) {
            self.r_s + PLANT_TRUNCATION_BOHR
        } else {
            self.r_f
        }
    }

    /// The tail for the pair `(i, j)`, resolved through the bank's OWN pair-to-slot map.
    ///
    /// `crate::bank::slot_index` and not an ad-hoc combination of the two species indices:
    /// the near sector dispatches through `PairBank::table_at`, which is that same map, and
    /// two routes to one table are two things that can disagree about which curve a pair
    /// gets.
    fn model_for(&self, slots: &[usize], i: usize, j: usize) -> Option<&TailModel> {
        let s = crate::bank::slot_index(slots.get(i).copied()?, slots.get(j).copied()?);
        self.models.get(s).and_then(|m| m.as_ref())
    }

    /// The same resolution, exposed so an instrument can name the slot a pair will use
    /// without duplicating the map.
    pub fn slot_of(slots: &[usize], i: usize, j: usize) -> Option<usize> {
        Some(crate::bank::slot_index(
            slots.get(i).copied()?,
            slots.get(j).copied()?,
        ))
    }

    fn rebuild_offsets(&mut self, key: BoxKey) {
        self.offsets = Self::offsets_for(key);
        self.key = Some(key);
    }

    /// The image lattice for one box key. A pure function of the key, which is what makes
    /// G9's bit-identity check meaningful: a fresh sector and a rescaled one must produce
    /// the same list, and P1 is what happens when the rescaled one keeps the old.
    pub fn offsets_for(key: BoxKey) -> Vec<(f64, f64, f64)> {
        let mut v = Vec::new();
        if !key.periodic || key.shells == 0 {
            return v;
        }
        let m = key.shells as i64;
        let mz = if key.three_d { m } else { 0 };
        for nx in -m..=m {
            for ny in -m..=m {
                for nz in -mz..=mz {
                    if nx == 0 && ny == 0 && nz == 0 {
                        continue;
                    }
                    v.push((
                        nx as f64 * key.lx,
                        ny as f64 * key.ly,
                        nz as f64 * key.lz,
                    ));
                }
            }
        }
        v
    }
}

/// One curve's knots, as the far sector reads them — the input to G3's fit.
///
/// A borrowed view rather than a copy of the table: the fit is a measurement OF the
/// committed table, and a second copy is a second thing that can disagree with it.
#[derive(Clone, Debug)]
pub struct CurveTail {
    /// Knot radii, ascending, bohr.
    pub r: Vec<f64>,
    /// Knot energies, asymptote-zeroed, hartree.
    pub u: Vec<f64>,
    /// The table's own exponential extrapolation index, `−u'(r_max)/u(r_max)`.
    pub hi_b: f64,
    pub solver_exit: &'static str,
    pub solver_budget_iterations: u64,
    pub uncertainty_hartree: f64,
}

impl CurveTail {
    pub fn r_max(&self) -> f64 {
        self.r.last().copied().unwrap_or(0.0)
    }

    /// The table's own value at `r`, using its exponential extrapolation past the last
    /// knot — the same arithmetic `Table::eval` runs, so the seam match is exact.
    pub fn u_at(&self, r: f64) -> f64 {
        let n = self.r.len();
        if n == 0 {
            return 0.0;
        }
        let rm = self.r[n - 1];
        if r >= rm {
            return self.u[n - 1] * (-self.hi_b * (r - rm)).exp();
        }
        // Linear in the knots is enough here: this is only ever called at `R_s ≥ r_max`.
        let mut k = 0;
        while k + 2 < n && self.r[k + 1] < r {
            k += 1;
        }
        let t = (r - self.r[k]) / (self.r[k + 1] - self.r[k]);
        self.u[k] + t * (self.u[k + 1] - self.u[k])
    }

    /// G3'S MEASUREMENT. The local log-log slope over the last [`FIT_FRACTION`] of knots,
    /// with its residual, and the band it puts the curve in.
    ///
    /// Least squares on `(ln r, ln|u|)`: for `u ∼ r^(−p)` the slope is `−p` exactly, and
    /// for `u ∼ exp(−b r)` it is `−b·r`, which is not constant — so the RESIDUAL is what
    /// distinguishes them and it is reported rather than assumed small.
    pub fn fit(&self) -> TailFit {
        let n = self.r.len();
        let want = ((n as f64 * FIT_FRACTION).ceil() as usize).max(FIT_MIN_KNOTS);
        let start = n.saturating_sub(want);
        let mut sx = 0.0;
        let mut sy = 0.0;
        let mut sxx = 0.0;
        let mut sxy = 0.0;
        let mut k = 0.0f64;
        for idx in start..n {
            let (ri, ui) = (self.r[idx], self.u[idx].abs());
            if !(ri > 0.0) || !(ui > 0.0) {
                continue;
            }
            let (x, y) = (ri.ln(), ui.ln());
            sx += x;
            sy += y;
            sxx += x * x;
            sxy += x * y;
            k += 1.0;
        }
        let (slope, intercept) = if k >= 2.0 && (k * sxx - sx * sx).abs() > 0.0 {
            let s = (k * sxy - sx * sy) / (k * sxx - sx * sx);
            (s, (sy - s * sx) / k)
        } else {
            (0.0, 0.0)
        };
        let mut res = 0.0;
        let mut m = 0.0f64;
        for idx in start..n {
            let (ri, ui) = (self.r[idx], self.u[idx].abs());
            if !(ri > 0.0) || !(ui > 0.0) {
                continue;
            }
            let d = ui.ln() - (intercept + slope * ri.ln());
            res += d * d;
            m += 1.0;
        }
        let residual = if m > 0.0 { (res / m).sqrt() } else { f64::INFINITY };
        let p_fit = -slope;
        let exp_index = self.hi_b * self.r_max();
        let band = if (P_FIT_LO..=P_FIT_HI).contains(&p_fit)
            && exp_index <= EXP_INDEX_FACTOR * p_fit
        {
            TailBand::Adopting
        } else {
            TailBand::Fenced
        };
        TailFit {
            p_fit,
            residual,
            exp_index,
            band,
            r_max: self.r_max(),
            u_at_max: self.u.last().copied().unwrap_or(0.0).abs(),
            knots_fitted: (n - start),
        }
    }
}
