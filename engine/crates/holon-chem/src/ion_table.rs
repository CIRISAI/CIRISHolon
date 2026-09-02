//! Charged species tabulated through the SAME door the neutral ones go through — with
//! the charge and the spin sector IN THE KEY.
//!
//! # The one sentence this module exists for
//!
//! **A table row that does not name its charge and its spin sector is unusable, and must
//! be impossible to construct.** [`IonKey`] is that sentence as a type: private fields, no
//! `Default`, and exactly one constructor, which takes the charge as an argument. There is
//! no way to hold a key that does not name what it is a key FOR. The sector is not taken
//! from the caller either — it is DERIVED by [`crate::ions::spin_partition`]'s parity rule
//! and then carried, so the sector in the key and the sector the solve ran in cannot
//! disagree.
//!
//! The failure shape being designed out is the one `MISFITS.md` registers as
//! `M-CACHE-KIND`: record kinds sharing one key namespace let existence stand in for
//! certification. A bank keyed by `(Z_a, Z_b)` has exactly one slot for "O and H", and
//! `OH`, `OH⁻` and `OH⁺` all want it.
//!
//! # No ion branch, anywhere
//!
//! Nothing here tests `charge != 0` to choose a code path. Charge is DATA, exactly as `Z`
//! is data in [`crate::cluster`] — the tower's law one tier up: *Z prices, Z never
//! branches*, and now *charge prices, charge never branches*. The neutral curve is the
//! `charge == 0` instance of this generator and `tests/ion_tables.rs` asserts, on raw
//! `f64` bits, that it reproduces [`crate::pair::generate_pair_table`]'s columns knot for
//! knot. If that gate ever fires, this module has become a second implementation of the
//! neutral path and must be deleted rather than repaired.
//!
//! The one place charge IS looked at is [`TableRefusal::AnionFenced`], and that is a
//! POLICY door rather than a physics branch — see below.
//!
//! # Why anions refuse HERE while [`crate::ions`] still serves them
//!
//! Node C's electron-affinity gate FIRED: in STO-3G, OH⁻ sits +0.3055 Ha ABOVE neutral OH,
//! and the one-determinant H⁻/H control fires the same way, so the cause is the declared
//! basis (no diffuse functions) and not the charged seam. That reading is kept, marked, and
//! is a measured fact about this model — so [`crate::ions::solve_geometry_charged`] must
//! keep producing it.
//!
//! A TABLE is a different object. A solve is a MEASUREMENT; a table is a PUBLICATION, and
//! fence I-5 forbids publishing this model's anion energies as chemistry. So the fence sits
//! on this door and not on the solver's, and it names itself in the refusal:
//! [`FENCE_I5`] and [`FENCE_I5_CAUSE`]. Nothing here tunes a basis to get past it; basis
//! extension is its own node, and `conformance/water_observatory/ION_STAKING.md` row I-5
//! carries its receipt-gate.
//!
//! # What is NOT here, and where it went
//!
//! * **No MBE decomposition of a charged cluster.** Splitting one needs a rule for which
//!   fragment carries the excess charge — precisely the ambiguity ION_STAKING row I-1 opens
//!   for the census. What this module does instead is make the rule EXPLICIT and CHECKED
//!   for the only place it is currently needed: a [`Channel`] states each fragment's charge
//!   and the sum is checked against the cluster's by integer identity ([`Channel::validate`]).
//! * **No long-range term.** The domain boundary's discarded interaction is MEASURED
//!   ([`IonMeta::boundary_systematic`]) and nothing rescues it. The ionic `r^-1` tail is
//!   GANTT node B2's, and this module neither serves it nor claims anything about it.
//! * **No relaxation.** Every geometry is staked by the caller's [`StretchCut`], so every
//!   energy is an energy AT A POINT and every fragment is frozen at its in-complex
//!   geometry. A frozen fragment sits above its relaxed self, so a depth from this table
//!   OVERSTATES the relaxed depth.
//! * **No multiplicity claim.** The parity rule fixes `S_z`, never `S`. The key names the
//!   sector solved in; it does not say what total spin was found (ION_STAKING I-4).

use crate::cluster::ClusterClass;
use crate::dual::D2;
use crate::elements::{by_symbol, Species};
use crate::fci::{SolveExit, SolverRoute};
use crate::ions::{solve_geometry_charged, spin_partition, ChargeRefusal};
use crate::pair::{
    build_basis, choose, provenance_for, solve_basis, PointSolution, Well, WELL_MIN_DEPTH,
};
use crate::sigma_op::DeviceClass;

/// The fence every anion table refuses under, named in the refusal so a caller reports the
/// fence rather than re-deriving the condition. Registered as `FENCES.md` M10 and
/// `ION_STAKING.md` I-5.
pub const FENCE_I5: &str = "I-5";

/// Why [`FENCE_I5`] fires. The CAUSE, not the symptom: the symptom is that OH⁻ came out
/// above OH, and the cause is the basis it came out of.
pub const FENCE_I5_CAUSE: &str =
    "anions are unbound in STO-3G: no diffuse functions, so the extra electron has nowhere \
     loosely bound to go (OH- sits +0.3055 Ha ABOVE neutral OH, and the one-determinant \
     H-/H control fires the same way). This model's anion energies may not be published as \
     chemistry. Exit: ION_STAKING.md I-5, a basis carrying diffuse functions, with the two \
     gates in tests/ion_core.rs re-run UNCHANGED";

/// How many bisection steps [`generate_ion_table`] spends locating the well's minimum.
///
/// DECLARED. Each step is a full cluster solve, so this is a price and not a tolerance: 24
/// halvings of a one-knot bracket put the located minimum far below any separation the
/// interpolant resolves, and doubling it would double the well's cost for digits nothing
/// downstream reads.
pub const WELL_BISECTION_STEPS: usize = 24;

// ------------------------------------------------------------------------ the key

/// A table row's identity: WHAT it is made of, WHAT charge it carries, and WHICH `S_z`
/// sector it was solved in.
///
/// # Impossible to construct without a charge
///
/// The fields are private and [`IonKey::state`] is the only constructor. There is no
/// `Default`, no `from_class`, and no builder that fills the charge in later, so a row
/// keyed by composition alone cannot exist — the compiler is the gate, which is the only
/// kind of gate that cannot be forgotten under deadline.
///
/// The electron count and the sector are DERIVED here rather than accepted: the count is
/// `sum(Z) - charge`, arithmetic with nothing to choose, and the partition is
/// [`spin_partition`]'s parity rule, a stated MODEL CHOICE. Deriving them is what makes the
/// key's sector and the solve's sector the same fact rather than two fields free to drift.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IonKey<const N: usize> {
    class: ClusterClass<N>,
    charge: i32,
    n_electrons: usize,
    sz2: u32,
}

impl<const N: usize> IonKey<N> {
    /// The key for a species list at a STATED total charge, or the [`ChargeRefusal`] that
    /// says why there is none.
    ///
    /// Refuses exactly the two ARITHMETIC impossibilities — an electron count below zero,
    /// and an anion holding more excess electrons than the system has protons. The third
    /// refusal, [`ChargeRefusal::UnstatedSpinSector`], needs an assembled basis to know how
    /// many orbitals there are and therefore cannot fire here; it fires in
    /// [`crate::ions::solve_geometry_charged`], on the same call this key's table will make.
    pub fn state(species: &[Species; N], charge: i32) -> Result<Self, ChargeRefusal> {
        let total_z: u32 = species.iter().map(|s| s.z).sum();
        let signed = i64::from(total_z) - i64::from(charge);
        if signed < 0 {
            return Err(ChargeRefusal::NegativeElectrons {
                total_z,
                charge,
                would_be_electrons: signed,
            });
        }
        if charge < 0 && i64::from(charge.unsigned_abs()) > i64::from(total_z) {
            return Err(ChargeRefusal::ChargeTooLarge { total_z, charge });
        }
        let n_electrons = signed as usize;
        let (na, nb) = spin_partition(n_electrons);
        Ok(Self {
            class: ClusterClass::of(species),
            charge,
            n_electrons,
            sz2: (na - nb) as u32,
        })
    }

    /// The sorted Z-multiset — node A's cluster class, unchanged and reused rather than
    /// re-invented, so a charged table and a neutral one are addressed by the same
    /// composition key.
    pub fn class(&self) -> ClusterClass<N> {
        self.class
    }

    /// The TOTAL charge in units of the elementary charge.
    pub fn charge(&self) -> i32 {
        self.charge
    }

    pub fn n_electrons(&self) -> usize {
        self.n_electrons
    }

    /// Twice the `S_z` the sector carries: 0 for an even electron count, 1 for an odd one.
    /// Same convention as [`crate::pair::PairMeta::sz2`].
    pub fn sz2(&self) -> u32 {
        self.sz2
    }

    /// The `(n_alpha, n_beta)` the parity rule named.
    pub fn partition(&self) -> (usize, usize) {
        spin_partition(self.n_electrons)
    }
}

// ---------------------------------------------------------------------- the cut

/// A staked one-dimensional cut through a cluster's configuration space: every slot frozen
/// except one, which moves along a fixed direction at distance `q` from a fixed origin.
///
/// # Why the neutral pair curve is an INSTANCE of this and not a cousin of it
///
/// With `N = 2`, slot 0 frozen at the origin, the moving slot 1 along `+z`, this produces
/// exactly the centres [`crate::pair::generate_pair_table`] builds by hand:
/// `q * 1.0` is `q` and `q * 0.0` is `+0.0` in IEEE, to the bit. That is not a coincidence
/// to be grateful for, it is the reason the bit-identity gate can exist at all — one cut
/// type serves the two-atom neutral curve and the four-atom cation, and neither is a
/// special case of the other.
///
/// The direction is normalised at construction, because a caller who states a direction of
/// length 1.4 means a direction, and silently letting `q` mean 1.4 bohr per unit would be a
/// coordinate nobody declared.
#[derive(Clone, Copy, Debug)]
pub struct StretchCut<const N: usize> {
    species: [Species; N],
    frozen: [[f64; 3]; N],
    moving: usize,
    origin: [f64; 3],
    dir: [f64; 3],
    coordinate: &'static str,
}

impl<const N: usize> StretchCut<N> {
    /// A cut. `frozen` gives every slot's position; the `moving` slot's entry is IGNORED
    /// and replaced by `origin + q * dir`.
    ///
    /// Panics if `moving` is out of range or `dir` has zero length: both are caller bugs
    /// about a geometry, and this module's refusals are about charge only — the same
    /// division [`crate::ions::solve_geometry_charged`] draws for a mismatched centre count.
    pub fn new(
        species: [Species; N],
        frozen: [[f64; 3]; N],
        moving: usize,
        origin: [f64; 3],
        dir: [f64; 3],
        coordinate: &'static str,
    ) -> Self {
        assert!(moving < N, "moving slot {moving} is not one of the {N} slots");
        let norm = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        assert!(norm > 0.0, "a cut direction must have a direction");
        Self {
            species,
            frozen,
            moving,
            origin,
            dir: [dir[0] / norm, dir[1] / norm, dir[2] / norm],
            coordinate,
        }
    }

    pub fn species(&self) -> [Species; N] {
        self.species
    }

    /// What `q` MEANS, in words and units. Carried into the emitted table, because a
    /// column of numbers whose coordinate is undeclared is a column nobody can re-derive.
    pub fn coordinate(&self) -> &'static str {
        self.coordinate
    }

    /// The centres at `q`, with `q` carrying its own derivative seed.
    ///
    /// Pass [`D2::var`] to get `E`, `dE/dq` and `d²E/dq²` out of one solve; pass
    /// [`D2::c`] for the value alone.
    pub fn centers(&self, q: D2) -> Vec<[D2; 3]> {
        (0..N)
            .map(|s| {
                if s == self.moving {
                    [
                        q * self.dir[0] + self.origin[0],
                        q * self.dir[1] + self.origin[1],
                        q * self.dir[2] + self.origin[2],
                    ]
                } else {
                    [
                        D2::c(self.frozen[s][0]),
                        D2::c(self.frozen[s][1]),
                        D2::c(self.frozen[s][2]),
                    ]
                }
            })
            .collect()
    }

    /// The centres of a SUBSET of slots, values only — a fragment's geometry, frozen where
    /// the cut puts it.
    fn fragment_centers(&self, q: f64, mask: u32) -> (Vec<Species>, Vec<[D2; 3]>) {
        let all = self.centers(D2::c(q));
        let mut sp = Vec::new();
        let mut ct = Vec::new();
        for s in 0..N {
            if mask & (1 << s) != 0 {
                sp.push(self.species[s]);
                ct.push(all[s]);
            }
        }
        (sp, ct)
    }
}

// ------------------------------------------------------------------- the channels

/// One fragment of a dissociation channel: which slots it takes, and WHAT CHARGE IT
/// CARRIES.
///
/// The charge is stated per fragment rather than inferred, because inferring it is the
/// open question: "which component carries the excess proton" is exactly the ambiguity
/// `ION_STAKING.md` row I-1 opens for the census, and a module that guessed here would be
/// answering that question silently in a place nobody would look for the answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fragment {
    /// Bit `s` set means slot `s` belongs to this fragment.
    pub slots: u32,
    pub charge: i32,
    pub label: &'static str,
}

/// A named dissociation channel: a partition of the cluster's slots into charged fragments.
#[derive(Clone, Copy, Debug)]
pub struct Channel<'a> {
    pub label: &'static str,
    pub fragments: &'a [Fragment],
}

impl Channel<'_> {
    /// The two structural checks, both EXACT and neither a tolerance.
    ///
    /// 1. The fragments PARTITION the slots: every slot in exactly one, none left over.
    /// 2. The fragment charges SUM to the cluster's declared total charge, as integers.
    ///
    /// The second is the identity ION_STAKING I-1 stakes for the census's charge column,
    /// enforced here at the first door that can enforce it. Integer arithmetic on purpose:
    /// charge is not a quantity that can be nearly conserved.
    pub fn validate(&self, n_slots: usize, total_charge: i32) -> Result<(), TableRefusal> {
        let full = if n_slots >= 32 { u32::MAX } else { (1u32 << n_slots) - 1 };
        let mut union = 0u32;
        for f in self.fragments {
            if union & f.slots != 0 {
                return Err(TableRefusal::ChannelSlotsNotAPartition {
                    channel: self.label,
                    covered: union,
                    offending: f.slots,
                    expected: full,
                });
            }
            union |= f.slots;
        }
        if union != full {
            return Err(TableRefusal::ChannelSlotsNotAPartition {
                channel: self.label,
                covered: union,
                offending: 0,
                expected: full,
            });
        }
        let sum: i32 = self.fragments.iter().map(|f| f.charge).sum();
        if sum != total_charge {
            return Err(TableRefusal::ChannelChargeNotConserved {
                channel: self.label,
                declared_total: total_charge,
                fragment_sum: sum,
            });
        }
        Ok(())
    }
}

/// What one channel came out at, fragment by fragment. Emitted whole: a channel sum with
/// its terms hidden is a number nobody can check.
#[derive(Clone, Debug)]
pub struct ChannelReading {
    pub label: &'static str,
    /// `(fragment label, charge, energy)`, in the order they were summed.
    pub fragments: Vec<(&'static str, i32, f64)>,
    /// The sum, accumulated in fragment order — seeded on the first term rather than on a
    /// `0.0`, the same rounding discipline `cluster.rs` states for its reference atoms.
    pub sum: f64,
}

// ------------------------------------------------------------------- the refusals

/// Why a charged table was not produced. Every variant carries the numbers that made it
/// fire, so a caller reports the refusal instead of re-deriving the condition.
#[derive(Clone, Debug, PartialEq)]
pub enum TableRefusal {
    /// A spec that named species and no charge. The loudest refusal in this module,
    /// because it is the one a caller reaches by habit: a neutral formula is what every
    /// other bank in this tree accepts.
    UnstatedCharge { spec: String },
    /// A spec token that is not an element this crate's registry holds.
    UnknownElement { symbol: String },
    /// A spec with no species at all.
    EmptySpec { spec: String },
    /// An anion, refused under fence [`FENCE_I5`] before anything is spent.
    AnionFenced {
        fence: &'static str,
        charge: i32,
        cause: &'static str,
    },
    /// The arithmetic refusals of the charged seam, passed through unchanged rather than
    /// re-worded — one rule, one place, one set of names.
    Charge(ChargeRefusal),
    /// Priced before it was attempted, and REFUSED BY THIS MACHINE'S RESOURCE DOOR — both
    /// the determinant working set and the MPS route's MPO. Not a property of the space:
    /// the refusal carries the bytes so the reader knows which machine admits it.
    PastAutomaticRoute {
        n_det: usize,
        n_orb: usize,
        /// The determinant working set this machine's resource door refused, bytes.
        det_price_bytes: u64,
        /// The MPS route's (provisional) dense-MPO price, also refused, bytes.
        mpo_price_bytes: u64,
    },
    ChannelSlotsNotAPartition {
        channel: &'static str,
        covered: u32,
        offending: u32,
        expected: u32,
    },
    ChannelChargeNotConserved {
        channel: &'static str,
        declared_total: i32,
        fragment_sum: i32,
    },
    /// A knot that did not converge. This VOIDS the table rather than downgrading it: a
    /// published curve carrying a knot that gave up is a dead result presenting as a live
    /// one, and the budget it was solved under is carried so the reader can tell "ran out"
    /// from "cannot get there".
    NotConverged {
        exit: SolveExit,
        worst_residual: f64,
        solver_budget_iterations: usize,
    },
    /// Knots from two device classes. They agree to 3e-15 and differ bitwise on most
    /// entries, so one label cannot describe the mixture.
    MixedDeviceClass { first: DeviceClass, later: DeviceClass },
    /// [`IonTable::to_json`] was asked to serialise a table whose interpolant error has not
    /// been measured. The grid's error and the solver's residual are different quantities
    /// and a published table states both.
    InterpolantErrorNotMeasured,
    /// A grid request that is not one.
    UnusableGrid { q_min: f64, q_max: f64, n_knots: usize },
}

impl From<ChargeRefusal> for TableRefusal {
    fn from(c: ChargeRefusal) -> Self {
        TableRefusal::Charge(c)
    }
}

// --------------------------------------------------------------------- the spec door

/// Parse a SLOT-ORDERED species list with a STATED charge: `"O H H H +1"`, `"H H 0"`.
///
/// # Why this is not a chemical formula
///
/// A formula says composition; a cut needs SLOTS, in the order the geometry names them.
/// `"O H H H"` and `"H H H O"` are the same substance and different tables, so the spec is
/// the slot list and says so.
///
/// # The refusal this door exists for
///
/// The last token must be an integer. A spec ending in an element symbol has not stated a
/// charge, and comes back [`TableRefusal::UnstatedCharge`] rather than being read as
/// neutral. Defaulting to neutral is the whole failure mode: it is right most of the time,
/// which is what makes the times it is wrong impossible to notice.
pub fn parse_spec(spec: &str) -> Result<(Vec<Species>, i32), TableRefusal> {
    let toks: Vec<&str> = spec.split_whitespace().collect();
    if toks.is_empty() {
        return Err(TableRefusal::EmptySpec { spec: spec.to_string() });
    }
    let last = toks[toks.len() - 1];
    let Ok(charge) = last.parse::<i32>() else {
        return Err(TableRefusal::UnstatedCharge { spec: spec.to_string() });
    };
    let mut species = Vec::with_capacity(toks.len() - 1);
    for t in &toks[..toks.len() - 1] {
        match by_symbol(t) {
            Some(s) => species.push(s),
            None => return Err(TableRefusal::UnknownElement { symbol: (*t).to_string() }),
        }
    }
    if species.is_empty() {
        return Err(TableRefusal::EmptySpec { spec: spec.to_string() });
    }
    Ok((species, charge))
}

// ------------------------------------------------------------------- the plant door

/// A NAMED DEFECT, injected into the generator's own path.
///
/// Production calls [`generate_ion_table`], which is [`generate_ion_table_planted`] at
/// [`Plant::None`]; the gates call the planted form. That direction matters: a plant that
/// has to bypass the code it is planted in is not testing that code, which is the reason
/// [`crate::pair::solve_basis`] is public for the pair plants.
///
/// Every variant here is a DEFECT and none is a mode. Each is pre-checked to fire on this
/// instrument in `tests/ion_tables.rs`, because a mutation that stays silent for a
/// numerical reason reports a gate as working when nothing was tested.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Plant {
    #[default]
    None,
    /// One ULP added to every knot's energy. The smallest defect a bit-identity gate must
    /// still see; carrier is the energy column, nonzero on every knot.
    EnergyUlp,
    /// The solve runs at `charge + 1` while the key says what the caller stated. Carrier is
    /// the cluster energy, nonzero in the charge sector the plant acts on.
    ChargeOffByOne,
    /// `(n_alpha, n_beta) -> (n_alpha + 1, n_beta - 1)`: a genuinely HIGHER `S_z` sector.
    ///
    /// Deliberately not the alpha/beta SWAP, which is degenerate by spin symmetry and would
    /// leave the energy unmoved — an unobservable mutation dressed as a test.
    SectorShift,
    /// The channel enumeration loses its lowest member, so the declared asymptote becomes
    /// the wrong one. Carrier is the asymptote, nonzero in the channel-sum sector.
    DropLowestChannel,
}

// --------------------------------------------------------------------- the table

/// Everything about a charged curve that is not one of its four columns.
#[derive(Clone, Debug)]
pub struct IonMeta<const N: usize> {
    /// Composition, charge and sector. The whole point of the module.
    pub key: IonKey<N>,
    pub symbols: Vec<&'static str>,
    /// What `q` means, from the cut.
    pub coordinate: &'static str,
    pub q_min: f64,
    pub q_max: f64,
    pub n_knots: usize,
    pub n_basis: usize,
    pub n_det: usize,
    /// The channels as enumerated by the caller and MEASURED here, all of them, in the
    /// order given.
    pub channels: Vec<ChannelReading>,
    /// The MINIMUM over the enumerated channels — measured, never assumed. A curve whose
    /// asymptote was taken from the obvious channel rather than the lowest one publishes a
    /// well as deep as the gap between them.
    pub e_asymptote: f64,
    pub asymptote_channel: &'static str,
    /// `|E(q_max) - e_asymptote|`: the interaction this table's domain truncation
    /// discards. MEASURED and reported; nothing here rescues it, and the ionic `r^-1` tail
    /// is GANTT node B2's question, not this table's.
    pub boundary_systematic: f64,
    pub well: Option<Well>,
    pub route: SolverRoute,
    /// The WORST exit over the knots. [`generate_ion_table`] refuses to return a table
    /// whose worst exit is not converged, so this field is a record rather than a warning.
    pub exit: SolveExit,
    pub provenance: &'static str,
    /// The Davidson iteration BUDGET every knot was solved under. Part of the artifact's
    /// identity, not a diagnostic: a capped residual is not monotone in solver effort, so a
    /// residual quoted without its budget is a number missing its meaning.
    pub solver_budget_iterations: usize,
    pub device: DeviceClass,
    /// Worst Davidson residual over the knots — the SOLVER's uncertainty.
    pub worst_residual: f64,
    pub worst_cg_residual: f64,
    pub worst_s_eigenvalue: f64,
    pub scf_converged_everywhere: bool,
    /// The GRID's uncertainty: the worst held-out departure of this table's cubic Hermite
    /// interpolant from a direct solve. `None` means NOT MEASURED, and
    /// [`IonTable::to_json`] refuses a `None` rather than writing a table that looks
    /// perfect on a check nobody ran.
    pub interpolant_uncertainty: Option<f64>,
}

/// A generated charged curve.
#[derive(Clone, Debug)]
pub struct IonTable<const N: usize> {
    pub q: Vec<f64>,
    pub e: Vec<f64>,
    /// `-dE/dq`, the FORCE, matching the renderer's contract.
    pub f: Vec<f64>,
    pub e2: Vec<f64>,
    pub meta: IonMeta<N>,
}

/// Which route the automatic router would take for a charged cluster, WITHOUT computing
/// anything: counts read off the registry, so the door is cheap enough to stand in front
/// of hours of work.
///
/// Returns `(n_det, n_orb, route_exists)`. The classification mirrors
/// [`crate::pair::automatic_route`] exactly and deliberately shares its `choose` so the
/// two doors cannot drift into pricing the same space differently.
pub fn charged_route<const N: usize>(species: &[Species; N], key: &IonKey<N>) -> (usize, usize, bool) {
    let n_orb: usize = species.iter().map(|s| s.n_basis()).sum();
    let (na, nb) = key.partition();
    let n_det = choose(n_orb, na).saturating_mul(choose(n_orb, nb));
    let exists = crate::pair::route_for(n_det, n_orb).exists();
    (n_det, n_orb, exists)
}

/// One knot, through the production door — or through a named defect.
fn solve_knot<const N: usize>(
    cut: &StretchCut<N>,
    key: &IonKey<N>,
    q: D2,
    plant: Plant,
) -> Result<PointSolution, ChargeRefusal> {
    let species = cut.species();
    match plant {
        Plant::ChargeOffByOne => solve_geometry_charged(&species, cut.centers(q), key.charge() + 1),
        Plant::SectorShift => {
            let (na, nb) = key.partition();
            assert!(nb >= 1, "the sector plant needs a beta electron to move");
            let basis = build_basis(&species, cut.centers(q));
            Ok(solve_basis(&basis, na + 1, nb - 1))
        }
        _ => solve_geometry_charged(&species, cut.centers(q), key.charge()),
    }
}

/// Generate a charged curve along a staked cut. The production entry point.
pub fn generate_ion_table<const N: usize>(
    cut: &StretchCut<N>,
    charge: i32,
    q_min: f64,
    q_max: f64,
    n_knots: usize,
    channels: &[Channel],
) -> Result<IonTable<N>, TableRefusal> {
    generate_ion_table_planted(cut, charge, q_min, q_max, n_knots, channels, Plant::None)
}

/// The same, with a named defect injected. See [`Plant`] — every variant is a defect and
/// the production path is `Plant::None`.
pub fn generate_ion_table_planted<const N: usize>(
    cut: &StretchCut<N>,
    charge: i32,
    q_min: f64,
    q_max: f64,
    n_knots: usize,
    channels: &[Channel],
    plant: Plant,
) -> Result<IonTable<N>, TableRefusal> {
    let species = cut.species();

    // 1. The key first, so an input that is not a species list at any charge gets the
    //    sharper name. Ordering stated rather than incidental: an anion beyond the nuclear
    //    charge is refused as ChargeTooLarge, not as the fence, because the fence is about
    //    what may be PUBLISHED and that input is not a system.
    let key = IonKey::state(&species, charge)?;

    // 2. The fence, before anything is spent.
    if key.charge() < 0 {
        return Err(TableRefusal::AnionFenced {
            fence: FENCE_I5,
            charge: key.charge(),
            cause: FENCE_I5_CAUSE,
        });
    }

    // 3. The price, before anything is spent.
    let (n_det_priced, n_orb_priced, exists) = charged_route(&species, &key);
    if !exists {
        return Err(TableRefusal::PastAutomaticRoute {
            n_det: n_det_priced,
            n_orb: n_orb_priced,
            det_price_bytes: crate::budget::price_determinant(n_det_priced).bytes,
            mpo_price_bytes: crate::budget::price_mpo(n_orb_priced).bytes,
        });
    }

    // 4. The grid and the channels, before anything is spent. A partition that is not
    //    one, or a charge that does not conserve, is a statement about the caller's model
    //    and not about the curve — so it must not cost a curve to discover.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if n_knots < 2 || !(q_min > 0.0) || !(q_max > q_min) {
        return Err(TableRefusal::UnusableGrid { q_min, q_max, n_knots });
    }
    for ch in channels {
        ch.validate(N, key.charge())?;
    }

    let mut q = Vec::with_capacity(n_knots);
    let mut e = Vec::with_capacity(n_knots);
    let mut f = Vec::with_capacity(n_knots);
    let mut e2 = Vec::with_capacity(n_knots);
    let (mut worst_residual, mut worst_cg, mut worst_s) = (0.0f64, 0.0f64, f64::INFINITY);
    let mut scf_ok = true;
    let (mut n_det, mut n_basis) = (0usize, 0usize);
    let mut route = SolverRoute::Determinant;
    let mut device: Option<DeviceClass> = None;
    let mut worst_exit = SolveExit::Trivial;

    for i in 0..n_knots {
        let qi = crate::table::grid_point(q_min, q_max, n_knots, i);
        let sol = solve_knot(cut, &key, D2::var(qi), plant)?;
        q.push(qi);
        e.push(if plant == Plant::EnergyUlp {
            next_ulp(sol.e.v)
        } else {
            sol.e.v
        });
        f.push(-sol.e.d);
        e2.push(sol.e.e);
        worst_residual = worst_residual.max(sol.residual);
        worst_cg = worst_cg.max(sol.cg_residual);
        worst_s = worst_s.min(sol.s_min_eigenvalue);
        scf_ok &= sol.scf_converged;
        match device {
            None => device = Some(sol.device),
            Some(d) => {
                if d != sol.device {
                    return Err(TableRefusal::MixedDeviceClass { first: d, later: sol.device });
                }
            }
        }
        if sol.route == SolverRoute::Dmrg {
            route = SolverRoute::Dmrg;
        }
        // The WORST exit, downgrading only: a curve is as good as its worst knot.
        if worst_exit.is_converged() && !sol.exit.is_converged() {
            worst_exit = sol.exit;
        } else if worst_exit == SolveExit::Trivial && sol.exit == SolveExit::Converged {
            worst_exit = sol.exit;
        }
        n_det = sol.n_det;
        n_basis = sol.n_basis;
    }

    // M-BUDGET-LAUNDER: exhaustion VOIDS, loudly, and never degrades into a published
    // artifact carrying a knot that gave up.
    if !worst_exit.is_converged() {
        return Err(TableRefusal::NotConverged {
            exit: worst_exit,
            worst_residual,
            solver_budget_iterations: crate::fci::davidson_budget(),
        });
    }

    // The channels, MEASURED, all of them, at the far end of the domain — where "frozen at
    // the in-complex geometry" and "separated" are the same geometry.
    let mut readings: Vec<ChannelReading> = Vec::with_capacity(channels.len());
    for ch in channels {
        let mut terms: Vec<(&'static str, i32, f64)> = Vec::with_capacity(ch.fragments.len());
        let mut sum = 0.0f64;
        for (k, frag) in ch.fragments.iter().enumerate() {
            let (sp, ct) = cut.fragment_centers(q_max, frag.slots);
            let v = solve_geometry_charged(&sp, ct, frag.charge)?.e.v;
            terms.push((frag.label, frag.charge, v));
            sum = if k == 0 { v } else { sum + v };
        }
        readings.push(ChannelReading { label: ch.label, fragments: terms, sum });
    }
    if plant == Plant::DropLowestChannel && readings.len() > 1 {
        let lowest = readings
            .iter()
            .enumerate()
            .fold(0usize, |b, (i, r)| if r.sum < readings[b].sum { i } else { b });
        readings.remove(lowest);
    }
    assert!(
        !readings.is_empty(),
        "a charged table with no dissociation channel has no zero, and a curve with no zero \
         is a column of totals nothing can be read against"
    );
    let best = readings
        .iter()
        .enumerate()
        .fold(0usize, |b, (i, r)| if r.sum < readings[b].sum { i } else { b });
    let e_asymptote = readings[best].sum;
    let asymptote_channel = readings[best].label;
    let boundary_systematic = (e[n_knots - 1] - e_asymptote).abs();

    let well = locate_cut_well(cut, &key, &q, &e, e_asymptote, plant);

    Ok(IonTable {
        q,
        e,
        f,
        e2,
        meta: IonMeta {
            key,
            symbols: species.iter().map(|s| s.symbol).collect(),
            coordinate: cut.coordinate(),
            q_min,
            q_max,
            n_knots,
            n_basis,
            n_det,
            channels: readings,
            e_asymptote,
            asymptote_channel,
            boundary_systematic,
            well,
            route,
            exit: worst_exit,
            provenance: provenance_for(route),
            solver_budget_iterations: crate::fci::davidson_budget(),
            device: device.expect("a curve with no solved knot has no device class"),
            worst_residual,
            worst_cg_residual: worst_cg,
            worst_s_eigenvalue: worst_s,
            scf_converged_everywhere: scf_ok,
            interpolant_uncertainty: None,
        },
    })
}

/// The next representable `f64` above `x`. The one-ULP plant's whole mechanism, written
/// out so the smallest observable defect is exactly that and not a rounding of one.
fn next_ulp(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    // Energies here are negative, so "next above" walks the bit pattern DOWN.
    let bits = x.to_bits();
    if x < 0.0 {
        f64::from_bits(bits - 1)
    } else {
        f64::from_bits(bits + 1)
    }
}

/// The bound minimum of a cut, located by bisecting the EXACT slope the solver reports.
///
/// Returns `None` unless the minimum is interior to the grid and deeper than
/// [`WELL_MIN_DEPTH`] below the declared asymptote — the same two conditions
/// [`crate::pair::locate_well`] applies, against a charged asymptote that is the minimum
/// over stated channels rather than a sum of two atoms.
fn locate_cut_well<const N: usize>(
    cut: &StretchCut<N>,
    key: &IonKey<N>,
    q: &[f64],
    e: &[f64],
    e_asymptote: f64,
    plant: Plant,
) -> Option<Well> {
    let mut best = 0usize;
    for i in 1..e.len() {
        if e[i] < e[best] {
            best = i;
        }
    }
    if best == 0 || best + 1 >= e.len() {
        return None;
    }
    if e_asymptote - e[best] <= WELL_MIN_DEPTH {
        return None;
    }
    let mut slope = |x: f64| {
        solve_knot(cut, key, D2::var(x), plant)
            .expect("the well bisection re-solves a geometry the curve already solved")
            .e
            .d
    };
    let (lo, hi) = (q[best - 1], q[best + 1]);
    if slope(lo) > 0.0 || slope(hi) < 0.0 {
        return None;
    }
    let r_e = crate::pair::bisect(lo, hi, WELL_BISECTION_STEPS, &mut slope)?;
    let at = solve_knot(cut, key, D2::var(r_e), plant).ok()?;
    Some(Well {
        r_e,
        d_e: e_asymptote - at.e.v,
        e_at_r_e: at.e.v,
        k_e: at.e.e,
    })
}

// ------------------------------------------------------- the held-out interpolant check

/// The worst departure of this table's cubic Hermite interpolant from a direct solve, over
/// a STAKED set of held-out points.
///
/// `intervals` are grid intervals to sample and `positions` are fractions inside each,
/// both supplied by the caller and both declared in the freeze rather than chosen here —
/// a ruler whose sample points are picked by the thing being measured is not a ruler.
///
/// Returns `(worst |dE|, worst |dF|, points)`. The POINT COUNT is returned because a
/// verifier must assert its work count: a sweep that sampled nothing reports a perfect
/// zero, which is the same reading a perfect table gives.
pub fn interpolant_error<const N: usize>(
    table: &IonTable<N>,
    cut: &StretchCut<N>,
    intervals: &[usize],
    positions: &[f64],
) -> (f64, f64, usize) {
    let key = table.meta.key;
    let (mut worst_e, mut worst_f) = (0.0f64, 0.0f64);
    let mut points = 0usize;
    for &i in intervals {
        if i + 1 >= table.q.len() {
            continue;
        }
        let (q0, q1) = (table.q[i], table.q[i + 1]);
        let h = q1 - q0;
        let (y0, y1) = (table.e[i], table.e[i + 1]);
        // The stored column is the force; the interpolant is built on dE/dq.
        let (d0, d1) = (-table.f[i], -table.f[i + 1]);
        for &t in positions {
            let t2 = t * t;
            let t3 = t2 * t;
            let value = (2.0 * t3 - 3.0 * t2 + 1.0) * y0
                + (t3 - 2.0 * t2 + t) * h * d0
                + (-2.0 * t3 + 3.0 * t2) * y1
                + (t3 - t2) * h * d1;
            let slope = ((6.0 * t2 - 6.0 * t) * y0 + (-6.0 * t2 + 6.0 * t) * y1) / h
                + (3.0 * t2 - 4.0 * t + 1.0) * d0
                + (3.0 * t2 - 2.0 * t) * d1;
            let exact = solve_knot(cut, &key, D2::var(q0 + t * h), Plant::None)
                .expect("a held-out point of a table that generated");
            worst_e = worst_e.max((value - exact.e.v).abs());
            worst_f = worst_f.max((-slope - (-exact.e.d)).abs());
            points += 1;
        }
    }
    (worst_e, worst_f, points)
}

// -------------------------------------------------------------------- serialisation

impl<const N: usize> IonTable<N> {
    /// Record the measured interpolant error. Separate from generation because the sweep
    /// costs a solve per held-out point, and because the number belongs to a STAKED sample
    /// set rather than to the generator's own choice of one.
    pub fn with_interpolant_uncertainty(mut self, worst: f64) -> Self {
        self.meta.interpolant_uncertainty = Some(worst);
        self
    }

    /// Serialise to the ion-table contract.
    ///
    /// REFUSES a table whose interpolant error was never measured. The two uncertainties
    /// are different quantities — the residual describes the SOLVE, the interpolant error
    /// describes the GRID — and the pair-table schema's single `uncertainty_hartree`
    /// conflates them. A reader who integrates this curve is exposed to the second one.
    pub fn to_json(&self) -> Result<String, TableRefusal> {
        let m = &self.meta;
        let Some(interp) = m.interpolant_uncertainty else {
            return Err(TableRefusal::InterpolantErrorNotMeasured);
        };
        let mut s = String::with_capacity(64 * self.q.len() + 4096);
        s.push_str("{\n  \"schema\": \"ION-TABLES/charged-cut/v1\",\n");
        s.push_str(&format!("  \"provenance\": \"{}\",\n", m.provenance));
        // NOT `sto3g::MODEL_NAME`. That constant reads "H2/STO-3G/FCI" — it is the label
        // of the crate's founding H2 curve, and stamping it on a four-atom cation would
        // name a species this table is not. The model here is the basis and the CI space,
        // and the `provenance` line above says which solver produced them.
        s.push_str("  \"model\": \"STO-3G/FCI\",\n");
        // ---- the key. First, and whole: a row that does not name these is unusable.
        s.push_str(&format!("  \"charge\": {},\n", m.key.charge()));
        s.push_str(&format!("  \"sz2\": {},\n", m.key.sz2()));
        s.push_str(&format!("  \"n_electrons\": {},\n", m.key.n_electrons()));
        let (na, nb) = m.key.partition();
        s.push_str(&format!("  \"n_alpha\": {na},\n  \"n_beta\": {nb},\n"));
        s.push_str("  \"class_z\": [");
        for (i, z) in m.key.class().zs().iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(&z.to_string());
        }
        s.push_str("],\n");
        s.push_str("  \"slots\": [");
        for (i, sym) in m.symbols.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }
            s.push_str(&format!("\"{sym}\""));
        }
        s.push_str("],\n");
        s.push_str(&format!("  \"coordinate\": \"{}\",\n", m.coordinate));
        s.push_str(
            "  \"units\": \"Hartree atomic units: q in bohr, E in hartree, F in \
             hartree/bohr. F is the FORCE, so dE/dq = -F.\",\n",
        );
        s.push_str(
            "  \"sector_rule\": \"parity: even electron count -> singlet, odd -> doublet. A \
             MODEL CHOICE; it fixes S_z and does NOT certify the total spin S found.\",\n",
        );
        // ---- the three disclosure fields, together, on every row source.
        s.push_str(&format!("  \"solver_exit\": \"{}\",\n", m.exit.label()));
        s.push_str(&format!("  \"uncertainty_hartree\": {:?},\n", m.worst_residual));
        s.push_str(&format!(
            "  \"solver_budget_iterations\": {},\n",
            m.solver_budget_iterations
        ));
        s.push_str(&format!("  \"interpolant_uncertainty_hartree\": {interp:?},\n"));
        s.push_str(
            "  \"uncertainty_note\": \"TWO uncertainties, deliberately not one. \
             uncertainty_hartree is the worst Davidson residual: it describes the SOLVE, and \
             it is not monotone in solver effort, so it is meaningless without \
             solver_budget_iterations beside it. interpolant_uncertainty_hartree is the \
             worst held-out departure of this table's cubic Hermite interpolant from a \
             direct solve: it describes the GRID, and it is what a consumer integrating \
             this curve is actually exposed to.\",\n",
        );
        s.push_str(&format!("  \"solver_route\": \"{}\",\n", match m.route {
            SolverRoute::Determinant => "determinant",
            SolverRoute::Dmrg => "DMRG",
        }));
        s.push_str(&format!(
            "  \"exact_in_model\": {},\n",
            m.route.is_exact_in_model()
        ));
        s.push_str(&format!("  \"device_class\": \"{}\",\n", m.device));
        s.push_str(&format!("  \"n_basis\": {},\n  \"n_determinants\": {},\n", m.n_basis, m.n_det));
        s.push_str(&format!(
            "  \"scf_converged_everywhere\": {},\n",
            m.scf_converged_everywhere
        ));
        s.push_str(&format!(
            "  \"worst_s_eigenvalue\": {:?},\n  \"worst_cg_residual\": {:?},\n",
            m.worst_s_eigenvalue, m.worst_cg_residual
        ));
        // ---- the zero, and how it was chosen.
        s.push_str(&format!("  \"E_asymptote\": {:?},\n", m.e_asymptote));
        s.push_str(&format!("  \"asymptote_channel\": \"{}\",\n", m.asymptote_channel));
        s.push_str(&format!(
            "  \"boundary_systematic_hartree\": {:?},\n",
            m.boundary_systematic
        ));
        s.push_str("  \"channels\": [\n");
        for (i, c) in m.channels.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"channel\": \"{}\", \"sum_hartree\": {:?}, \"fragments\": [",
                c.label, c.sum
            ));
            for (k, (lab, ch, en)) in c.fragments.iter().enumerate() {
                if k > 0 {
                    s.push_str(", ");
                }
                s.push_str(&format!(
                    "{{\"fragment\": \"{lab}\", \"charge\": {ch}, \"E_hartree\": {en:?}}}"
                ));
            }
            s.push_str(&format!("]}}{}\n", if i + 1 == m.channels.len() { "" } else { "," }));
        }
        s.push_str("  ],\n");
        match m.well {
            Some(w) => s.push_str(&format!(
                "  \"well\": {{\"q_e\": {:?}, \"D_e\": {:?}, \"E_at_q_e\": {:?}, \"k_e\": {:?}}},\n",
                w.r_e, w.d_e, w.e_at_r_e, w.k_e
            )),
            None => s.push_str("  \"well\": null,\n"),
        }
        s.push_str(
            "  \"well_note\": \"Fragments are FROZEN at their in-complex geometry and \
             nothing is relaxed, so D_e OVERSTATES the relaxed depth.\",\n",
        );
        push_array(&mut s, "q_grid_bohr", &self.q);
        push_array(&mut s, "E_hartree", &self.e);
        push_array(&mut s, "F_hartree_per_bohr", &self.f);
        push_array(&mut s, "E2_hartree_per_bohr2", &self.e2);
        s.push_str(&format!("  \"n_grid\": {}\n", self.q.len()));
        s.push_str("}\n");
        Ok(s)
    }
}

fn push_array(s: &mut String, name: &str, v: &[f64]) {
    s.push_str("  \"");
    s.push_str(name);
    s.push_str("\": [");
    for (i, x) in v.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        if x.is_finite() {
            s.push_str(&format!("{x:?}"));
        } else {
            s.push_str("null");
        }
    }
    s.push_str("],\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::{HYDROGEN, OXYGEN};

    #[test]
    fn the_key_derives_the_sector_it_cannot_be_told() {
        let k = IonKey::state(&[OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN], 1).expect("H3O+");
        assert_eq!(k.charge(), 1);
        assert_eq!(k.n_electrons(), 10);
        assert_eq!(k.sz2(), 0);
        assert_eq!(k.partition(), (5, 5));
        assert_eq!(k.class().zs(), [1, 1, 1, 8]);
        // The odd branch, on the neutral it is compared against.
        let n = IonKey::state(&[OXYGEN, HYDROGEN], 0).expect("OH");
        assert_eq!((n.n_electrons(), n.sz2(), n.partition()), (9, 1, (5, 4)));
    }

    #[test]
    fn the_arithmetic_refusals_pass_through_unreworded() {
        assert_eq!(
            IonKey::state(&[HYDROGEN], 2).unwrap_err(),
            ChargeRefusal::NegativeElectrons { total_z: 1, charge: 2, would_be_electrons: -1 }
        );
        assert_eq!(
            IonKey::state(&[HYDROGEN], -2).unwrap_err(),
            ChargeRefusal::ChargeTooLarge { total_z: 1, charge: -2 }
        );
    }

    #[test]
    fn a_channel_that_does_not_conserve_charge_is_refused() {
        let bad = [
            Fragment { slots: 0b0111, charge: 0, label: "H2O" },
            Fragment { slots: 0b1000, charge: 0, label: "H" },
        ];
        let ch = Channel { label: "wrong", fragments: &bad };
        assert_eq!(
            ch.validate(4, 1).unwrap_err(),
            TableRefusal::ChannelChargeNotConserved {
                channel: "wrong",
                declared_total: 1,
                fragment_sum: 0
            }
        );
        // And the same partition with the charge stated correctly passes.
        let good = [
            Fragment { slots: 0b0111, charge: 0, label: "H2O" },
            Fragment { slots: 0b1000, charge: 1, label: "H+" },
        ];
        assert!(Channel { label: "A", fragments: &good }.validate(4, 1).is_ok());
    }

    #[test]
    fn a_channel_that_is_not_a_partition_is_refused() {
        let overlap = [
            Fragment { slots: 0b0111, charge: 0, label: "H2O" },
            Fragment { slots: 0b1100, charge: 1, label: "H+" },
        ];
        assert!(matches!(
            Channel { label: "overlap", fragments: &overlap }.validate(4, 1),
            Err(TableRefusal::ChannelSlotsNotAPartition { .. })
        ));
        let gap = [Fragment { slots: 0b0111, charge: 1, label: "H2O+" }];
        assert!(matches!(
            Channel { label: "gap", fragments: &gap }.validate(4, 1),
            Err(TableRefusal::ChannelSlotsNotAPartition { .. })
        ));
    }

    #[test]
    fn the_spec_door_refuses_an_unstated_charge_and_serves_a_stated_one() {
        assert_eq!(
            parse_spec("O H H H").unwrap_err(),
            TableRefusal::UnstatedCharge { spec: "O H H H".to_string() }
        );
        let (sp, c) = parse_spec("O H H H +1").expect("a stated charge is served");
        assert_eq!(c, 1);
        assert_eq!(sp.len(), 4);
        assert_eq!(sp[0].symbol, "O");
        // Neutral must still be STATED, not defaulted.
        assert_eq!(parse_spec("H H 0").unwrap().1, 0);
        assert!(matches!(
            parse_spec("Xx H 0"),
            Err(TableRefusal::UnknownElement { .. })
        ));
    }

    #[test]
    fn the_cut_reproduces_the_pair_path_centres_to_the_bit() {
        let cut = StretchCut::new(
            [HYDROGEN, HYDROGEN],
            [[0.0; 3]; 2],
            1,
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            "H-H separation, bohr",
        );
        let got = cut.centers(D2::var(1.4));
        let want = [
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::var(1.4)],
        ];
        for (a, b) in got.iter().zip(want.iter()) {
            for k in 0..3 {
                assert_eq!(a[k].v.to_bits(), b[k].v.to_bits(), "value slot {k}");
                assert_eq!(a[k].d.to_bits(), b[k].d.to_bits(), "derivative slot {k}");
                assert_eq!(a[k].e.to_bits(), b[k].e.to_bits(), "second slot {k}");
            }
        }
    }

    #[test]
    fn one_ulp_is_one_ulp_on_a_negative_energy() {
        let x = -75.392010513557f64;
        let y = next_ulp(x);
        assert!(y > x, "the plant must move the energy UP");
        assert_eq!(y.to_bits(), x.to_bits() - 1);
        assert_ne!(x.to_bits(), y.to_bits());
    }
}
