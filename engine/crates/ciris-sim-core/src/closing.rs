//! The closing certificate: the wild pins of `NEWTON CLOSED`, declared in the type
//! system rather than in prose.
//!
//! `PROGRAM.md`'s PATH N defines `NEWTON CLOSED = N-a..N-e green, wild pins DECLARED on
//! the closing certificate`, and `DESCRIPTOR_CHAIN.md` §6 names the five wild pins:
//! feldspar potential, Charles law, grain-boundary data, compressive mode, and the
//! damping/restitution split. Both documents REQUIRE a closing certificate; neither IS
//! one. This module is the artifact.
//!
//! ## Why declaring is the whole job
//!
//! For GRANITE the descriptor chain is SEVERED at B2fm: no fracture-grade feldspar or
//! mica potential exists anywhere, and feldspar is ~60% of the demo stone. That is an
//! EXTERNAL dependency (feldspar MLIP development, not under project control), so the
//! certified-everywhere closure is not available and will not be for as long as the
//! dependency is open. `DESCRIPTOR_CHAIN.md` §6's own instruction is to plan for the pin
//! route and record it.
//!
//! So this certificate is NOT a formality en route to a stronger claim. It is the honest
//! form of CLOSE, and it is honest only if it can say **"I do not have this"** in its own
//! type system. Three of the five pins say exactly that, with
//! [`PinProvenance::OwedNoSource`]. Silent omission is the failure this artifact exists
//! to prevent, so a missing source is a typed state, never a gap in a comment.
//!
//! ## Every pin carries a falsifier, and the falsifier is typed
//!
//! Same standard the stance holds every claim to: a claim with no kill is not a claim
//! about the world and cannot be constructed. [`DeclaredPin`] has no constructor path
//! that omits [`PinFalsifier`], and the falsifier is a named observable with a decision
//! rule ([`PinKill`]), not a doc comment. A pin with no falsifier is a hope.
//!
//! One structural fact falls out and is enforced by the gate: for an OWED pin the
//! falsifier and the unlock are the SAME EVENT — the pin claims an absence, and an
//! absence claim dies the moment a source lands. For a warranted pin they differ: the
//! band kills the value, and something else entirely lifts the pin. The gate enforces the
//! biconditional in [`audit_pin`]; see [`PinField::KillMismatch`].
//!
//! ## Scope, stated so it cannot be laundered
//!
//! [`ClosingStatus::PinsDeclared`] discharges the PIN conjunct of `NEWTON CLOSED` and
//! nothing else. N-a..N-e is a separate conjunct with its own gates, and this module does
//! not evaluate it, does not import it, and must never be read as reporting on it.

/// The number of wild pins `DESCRIPTOR_CHAIN.md` §6's CLOSE node names.
///
/// If the record ever names more, [`WILD_PINS`] stops compiling rather than silently
/// certifying a subset — the const assertion below is the gate.
pub const RECORD_WILD_PIN_COUNT: usize = 5;

/// The five wild pins, by the names the record gives them. Closed: a sixth pin is an
/// edit here and a failed const assertion until it is declared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinId {
    /// B2fm's severance: no fracture-grade feldspar/mica potential exists anywhere.
    FeldsparPotential,
    /// The subcritical-crack-growth bridge across the strain-rate decades no simulation
    /// crosses.
    CharlesLaw,
    /// The measured interface record the T4 grain-adjacency relation network stands on.
    GrainBoundaryData,
    /// Compressive failure is cooperative; no extreme-value theorem transports it.
    CompressiveMode,
    /// A5: the split landed, the two warrants did not.
    DampingRestitutionSplit,
}

impl PinId {
    /// The record's own name for the pin, for reports that must match the GANTT.
    pub const fn label(self) -> &'static str {
        match self {
            PinId::FeldsparPotential => "feldspar potential",
            PinId::CharlesLaw => "Charles law",
            PinId::GrainBoundaryData => "grain-boundary data",
            PinId::CompressiveMode => "compressive mode",
            PinId::DampingRestitutionSplit => "damping/restitution split",
        }
    }
}

/// Where a pinned value's warrant comes from — the provenance of the value AS THE CHAIN
/// USES IT, not of some cousin of it in the literature. Closed: a fifth kind is an edit
/// here, because a provenance coordinate that can absorb anything grades nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinProvenance {
    /// Measured in this project, with the record in-tree.
    Measured,
    /// A literature value, named by author and year, carried in-tree. Not measured here.
    Published,
    /// Chosen by us — a stage calibration or a mid-band pick. NOT a physics claim, and
    /// the warrant field says what the choice was made inside of.
    Stipulated,
    /// The value the chain needs and this artifact DOES NOT HAVE: no code, no table, no
    /// in-tree record supplies it. A literature or external route may exist and be
    /// unimported; the falsifier names exactly what would have to land. The warrant field
    /// is EMPTY, enforced — "I do not have this", in the type system.
    OwedNoSource,
}

impl PinProvenance {
    pub const fn label(self) -> &'static str {
        match self {
            PinProvenance::Measured => "Measured",
            PinProvenance::Published => "Published",
            PinProvenance::Stipulated => "Stipulated",
            PinProvenance::OwedNoSource => "OwedNoSource",
        }
    }

    /// True exactly for the provenance that names a hole. The gate reads this, so the
    /// "am I owed?" question has one implementation.
    pub const fn is_owed(self) -> bool {
        matches!(self, PinProvenance::OwedNoSource)
    }
}

/// How an observation decides a pin. Closed at two variants because the five pins need
/// exactly two: warranted pins are killed by a number, owed pins by an existence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinKill {
    /// A measurement of the observable outside `[lo, hi]` kills the pinned value. The
    /// declared value must itself lie inside the band — a pin already outside its own
    /// kill band is dead on arrival and must not be declared live.
    OutsideBand,
    /// The pin asserts an ABSENCE, so it dies the moment the observable EXISTS. For an
    /// owed pin the falsifier and the unlock are the same event; that identity is what
    /// an owed pin is. Carries no band: `lo` and `hi` are NaN, enforced, so that no stray
    /// number reads as a stake.
    SourceLands,
}

/// What would falsify a pin — mandatory, typed, and never a doc comment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PinFalsifier {
    /// The observable whose measurement (or whose existence) decides the pin. Named
    /// specifically enough that someone else could go and get it.
    pub observable: &'static str,
    /// How the observation decides.
    pub kill: PinKill,
    /// Band lower edge for [`PinKill::OutsideBand`]; NaN for [`PinKill::SourceLands`].
    pub lo: f64,
    /// Band upper edge for [`PinKill::OutsideBand`]; NaN for [`PinKill::SourceLands`].
    pub hi: f64,
    /// What lifting the pin looks like — the falsifier's second half, the same standard
    /// every refusal on the tier ladder is held to.
    pub unlock: &'static str,
}

/// One wild pin, declared: what value the chain is standing on, where that value's
/// warrant comes from, and what would take it away.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeclaredPin {
    pub id: PinId,
    /// What the number IS, in words, including where it was read off. For an owed pin
    /// this describes the SIZE OF THE HOLE, because that is the only honest number an
    /// absent quantity has.
    pub quantity: &'static str,
    pub value: f64,
    pub unit: &'static str,
    pub provenance: PinProvenance,
    /// The citation or in-tree location backing the value. EMPTY exactly when the
    /// provenance is [`PinProvenance::OwedNoSource`] — the gate enforces the
    /// biconditional in both directions.
    pub warrant: &'static str,
    pub falsifier: PinFalsifier,
}

/// The five wild pins of `DESCRIPTOR_CHAIN.md` §6's CLOSE node.
///
/// Zero are Measured. One is Published, one Stipulated, three OwedNoSource — and the
/// three owed ones were confirmed absent by search before being declared, not assumed
/// absent.
pub static WILD_PINS: [DeclaredPin; RECORD_WILD_PIN_COUNT] = [
    DeclaredPin {
        id: PinId::FeldsparPotential,
        quantity: "granite-class modal mass fraction with NO fracture-grade interatomic \
                   potential anywhere: alkali feldspar 0.35 + plagioclase 0.25 + mica 0.10 \
                   (DESCRIPTOR_CHAIN.md §3.4's modal composition). Only quartz's 0.30 is \
                   covered, so the demo stone's T4 descriptor cannot be reached bottom-up \
                   for any field and hangs off declared experiment pins instead.",
        value: 0.70,
        unit: "mass fraction",
        provenance: PinProvenance::OwedNoSource,
        warrant: "",
        falsifier: PinFalsifier {
            observable: "a fracture-grade interatomic potential for alkali feldspar, \
                         plagioclase or mica — validated against fracture observables, not \
                         merely elastic ones — published anywhere and imported in-tree",
            kill: PinKill::SourceLands,
            lo: f64::NAN,
            hi: f64::NAN,
            unlock: "B2fm. This is an EXTERNAL dependency (feldspar MLIP development, not \
                     under project control), and until it lands CLOSE is reachable only in \
                     the with-declared-pins form — DESCRIPTOR_CHAIN.md §6's honest caveat \
                     (L21). The severance is why this certificate exists.",
        },
    },
    DeclaredPin {
        id: PinId::CharlesLaw,
        quantity: "decades of strain rate between the MD-certified floor (1e7 /s) and the \
                   quasi-static laboratory ceiling (1e0 /s) that no simulation crosses and \
                   no in-tree bridge covers. The demo's ball-on-wall impact (~1e0-1e3 /s) \
                   sits INSIDE this gap: the flagship use case is certified by \
                   interpolation the consumer must explicitly accept. The gap itself is \
                   already declared in code (STRAIN_RATE_GAP, crates/holon-sandbox/src/\
                   sim.rs) and surfaced to the UI; the BRIDGE across it is what is absent.",
        value: 7.0,
        unit: "decades of strain rate",
        provenance: PinProvenance::OwedNoSource,
        warrant: "",
        falsifier: PinFalsifier {
            observable: "a rate-dependent strength law for granite (subcritical crack \
                         growth / stress corrosion) covering 1e0-1e3 /s, imported in-tree \
                         with Record provenance",
            kill: PinKill::SourceLands,
            lo: f64::NAN,
            hi: f64::NAN,
            unlock: "Atkinson 1984's laboratory record spans the decades and has never \
                     been imported. Importing it lifts the pin as a warrant VALUE on an \
                     ordinary certificate edge — the orthogonal warrant coordinate, NOT a \
                     third edge species (DESCRIPTOR_CHAIN.md §5, L21).",
        },
    },
    DeclaredPin {
        id: PinId::GrainBoundaryData,
        quantity: "grain-boundary relation descriptors in the tree carrying a measured or \
                   published INTERFACE warrant. The T4 grain-adjacency CohesiveBond \
                   network's per-bond parameters are back-derived from continuum \
                   properties (material::CohesiveLaw::from_continuum), never from boundary \
                   data; B4BC's second join is this experiment pin \
                   (DESCRIPTOR_CHAIN.md §6).",
        value: 0.0,
        unit: "count of measured interface records",
        provenance: PinProvenance::OwedNoSource,
        warrant: "",
        falsifier: PinFalsifier {
            observable: "a measured granite grain-boundary interface record — bicrystal or \
                         in-situ per-boundary normal/shear stiffness and cohesion — \
                         imported in-tree with Record provenance",
            kill: PinKill::SourceLands,
            lo: f64::NAN,
            hi: f64::NAN,
            unlock: "B4BC's grain-boundary join. Until it lands the relation network's \
                     parameters are a continuum back-derivation wearing interface clothes, \
                     and this certificate says so rather than omitting it.",
        },
    },
    DeclaredPin {
        id: PinId::CompressiveMode,
        quantity: "uniaxial compressive strength of the certified specimen class (Lac du \
                   Bonnet granite), supplied by EXPERIMENT because compressive failure is \
                   cooperative wing-crack coalescence and no extreme-value transport \
                   theorem exists for it — Weibull is a tensile theorem \
                   (DESCRIPTOR_CHAIN.md §3.3, L23).",
        value: 200.0e6,
        unit: "Pa",
        provenance: PinProvenance::Published,
        warrant: "Martin & Chandler 1994, Int. J. Rock Mech. Min. Sci. 31(6):643-659 \
                  (200 +/- 22 MPa); carried in-tree as \
                  material::IsotropicMaterial::LAC_DU_BONNET.compressive_strength_pa",
        falsifier: PinFalsifier {
            observable: "uniaxial compressive strength of the certified specimen class — \
                         measured again, or DERIVED bottom-up once A3 lands (sigma_c as a \
                         theorem of tensile strength and friction on closed relations)",
            kill: PinKill::OutsideBand,
            lo: 178.0e6,
            hi: 222.0e6,
            unlock: "A3. When the bottom-up route delivers sigma_c on closed relations the \
                     pin retires to a validation output (DESCRIPTOR_CHAIN.md §3.4, C3). \
                     Note the pin is NOT lifted by the demo preset: DEMO_CALIBRATION's \
                     95 MPa is a stage number that this record flunks by 2x (C4).",
        },
    },
    DeclaredPin {
        id: PinId::DampingRestitutionSplit,
        quantity: "intrinsic material damping ratio zeta = 1/(2Q), standing in for an \
                   unmeasured specimen attenuation record. A5's SPLIT has landed \
                   (material_damping_ratio vs solver_damping_ratio, the latter named as \
                   unwarranted solver stabilization); the two WARRANTS have not. This half \
                   carries CLASS warrant only. The other half — restitution — carries no \
                   chart-level value at all: it was removed from IsotropicMaterial as a \
                   pair/velocity/geometry OUTCOME and is taken per contact pair by \
                   mechanical::SphereContactModel, clamped but never warranted.",
        value: 2.0e-3,
        unit: "damping ratio (dimensionless)",
        provenance: PinProvenance::Stipulated,
        warrant: "mid-band pick at Q = 250 inside the published granite intrinsic-Q band \
                  Q ~ 1e2-1e3; no publication states 2.0e-3 for this specimen and none is \
                  claimed (material::IsotropicMaterial::LAC_DU_BONNET field warrants)",
        falsifier: PinFalsifier {
            observable: "resonant-column or ultrasonic quality factor Q measured on the \
                         certified specimen class, converted to zeta = 1/(2Q)",
            kill: PinKill::OutsideBand,
            lo: 5.0e-4,
            hi: 5.0e-3,
            unlock: "A5's two named benchmarks, one per half: resonant column / ultrasonic \
                     Q for damping, restitution-vs-velocity for restitution. While either \
                     half is unbenchmarked the bottom-up pincer provably cannot reach the \
                     shipped struct — a concrete interface falsifier that stands today \
                     (DESCRIPTOR_CHAIN.md §3.4 A5, §4.2).",
        },
    },
];

// The record names five. If it comes to name more, this stops compiling rather than
// certifying a subset in silence.
const _: () = assert!(WILD_PINS.len() == RECORD_WILD_PIN_COUNT);

/// Which field of which pin is unpopulated. Closed, so a defect cannot be reported as a
/// vague failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinField {
    /// `quantity` is empty: the number has no stated meaning.
    Quantity,
    /// `unit` is empty: an unlabelled number is not a declaration.
    Unit,
    /// `value` is not finite.
    Value,
    /// Provenance claims a source and `warrant` is empty. The pin says it has something
    /// it does not have.
    MissingWarrant,
    /// Provenance is `OwedNoSource` and `warrant` is non-empty. The pin says it has
    /// nothing while naming something — one of the two is a lie.
    WarrantOnOwedPin,
    /// `falsifier.observable` is empty: no kill, so not a claim about the world.
    Observable,
    /// `falsifier.unlock` is empty: a refusal without its second half.
    Unlock,
    /// An `OutsideBand` kill whose band is missing, non-finite, or not ordered `lo < hi`.
    Band,
    /// A `SourceLands` kill carrying band numbers. An absence has no band, and a stray
    /// number would read as a stake.
    StrayBand,
    /// The kill kind and the provenance disagree: owed pins are killed by a source
    /// landing and warranted pins are not, in both directions.
    KillMismatch,
    /// The declared value lies outside its own kill band — dead on arrival, and must not
    /// be declared live.
    ValueOutsideOwnBand,
    /// The same pin id appears twice in the set.
    DuplicateId,
}

/// A gate failure: which pin, which field. Naming the pin is the point — a gate that
/// fails without saying where is a gate nobody can act on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PinDefect {
    pub pin: PinId,
    pub field: PinField,
}

/// How much of the certificate is owed, counted by provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PinCensus {
    pub measured: usize,
    pub published: usize,
    pub stipulated: usize,
    pub owed: usize,
}

impl PinCensus {
    pub const fn total(&self) -> usize {
        self.measured + self.published + self.stipulated + self.owed
    }
}

/// Audit one pin. Every rule here is a way the certificate can REFUSE; a certificate
/// that cannot refuse proves nothing.
pub fn audit_pin(pin: &DeclaredPin) -> Result<(), PinDefect> {
    let defect = |field| {
        Err(PinDefect {
            pin: pin.id,
            field,
        })
    };

    if pin.quantity.is_empty() {
        return defect(PinField::Quantity);
    }
    if pin.unit.is_empty() {
        return defect(PinField::Unit);
    }
    if !pin.value.is_finite() {
        return defect(PinField::Value);
    }

    // "I do not have this" is a typed state, and it is exclusive: an owed pin names no
    // warrant, and every other provenance must name one.
    if pin.provenance.is_owed() {
        if !pin.warrant.is_empty() {
            return defect(PinField::WarrantOnOwedPin);
        }
    } else if pin.warrant.is_empty() {
        return defect(PinField::MissingWarrant);
    }

    if pin.falsifier.observable.is_empty() {
        return defect(PinField::Observable);
    }
    if pin.falsifier.unlock.is_empty() {
        return defect(PinField::Unlock);
    }

    // An absence claim dies when a source lands; a warranted value dies on a number.
    // The two are exclusive, in both directions.
    let kills_on_existence = matches!(pin.falsifier.kill, PinKill::SourceLands);
    if kills_on_existence != pin.provenance.is_owed() {
        return defect(PinField::KillMismatch);
    }

    match pin.falsifier.kill {
        PinKill::SourceLands => {
            if !pin.falsifier.lo.is_nan() || !pin.falsifier.hi.is_nan() {
                return defect(PinField::StrayBand);
            }
        }
        PinKill::OutsideBand => {
            if !pin.falsifier.lo.is_finite()
                || !pin.falsifier.hi.is_finite()
                || pin.falsifier.lo >= pin.falsifier.hi
            {
                return defect(PinField::Band);
            }
            if pin.value < pin.falsifier.lo || pin.value > pin.falsifier.hi {
                return defect(PinField::ValueOutsideOwnBand);
            }
        }
    }

    Ok(())
}

/// Audit a whole pin set: every pin populated, no id declared twice.
pub fn audit_pins(pins: &[DeclaredPin]) -> Result<PinCensus, PinDefect> {
    let mut census = PinCensus::default();
    for (i, pin) in pins.iter().enumerate() {
        audit_pin(pin)?;
        for other in &pins[..i] {
            if other.id == pin.id {
                return Err(PinDefect {
                    pin: pin.id,
                    field: PinField::DuplicateId,
                });
            }
        }
        match pin.provenance {
            PinProvenance::Measured => census.measured += 1,
            PinProvenance::Published => census.published += 1,
            PinProvenance::Stipulated => census.stipulated += 1,
            PinProvenance::OwedNoSource => census.owed += 1,
        }
    }
    Ok(census)
}

/// The verdict of a closing certification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClosingStatus {
    /// Every wild pin the record names is declared, with a value, a provenance and a
    /// typed falsifier.
    ///
    /// This discharges the PIN conjunct of `PROGRAM.md`'s `NEWTON CLOSED` clause and
    /// NOTHING ELSE. The N-a..N-e conjunct has its own gates; this status does not
    /// report on it and must never be read as doing so.
    PinsDeclared,
    /// A pin is unpopulated or the set is incoherent: no certificate is issued, and
    /// `defect` names which pin and which field.
    Undeclarable,
}

/// The closing certificate for the demo stone: the pin half of `NEWTON CLOSED`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClosingCertificate {
    pub status: ClosingStatus,
    pub defect: Option<PinDefect>,
    /// How much of the close is owed, in the certificate's own arithmetic.
    pub census: PinCensus,
}

/// Certify a pin set. Fails, by design, on any unpopulated pin.
pub fn certify_closing(pins: &[DeclaredPin]) -> ClosingCertificate {
    match audit_pins(pins) {
        Ok(census) => ClosingCertificate {
            status: ClosingStatus::PinsDeclared,
            defect: None,
            census,
        },
        Err(defect) => ClosingCertificate {
            status: ClosingStatus::Undeclarable,
            defect: Some(defect),
            census: PinCensus::default(),
        },
    }
}

/// The closing certificate over the record's five wild pins.
pub fn closing_certificate() -> ClosingCertificate {
    certify_closing(&WILD_PINS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::IsotropicMaterial;

    fn pin(id: PinId) -> DeclaredPin {
        *WILD_PINS
            .iter()
            .find(|p| p.id == id)
            .expect("the record's five are all declared")
    }

    // ---------------------------------------------------------------------------
    // The certificate itself.
    // ---------------------------------------------------------------------------

    #[test]
    fn every_claimed_pin_is_populated() {
        let cert = closing_certificate();
        assert_eq!(cert.status, ClosingStatus::PinsDeclared);
        assert_eq!(cert.defect, None);
        assert_eq!(cert.census.total(), RECORD_WILD_PIN_COUNT);
    }

    #[test]
    fn the_census_is_zero_measured_one_published_one_stipulated_three_owed() {
        let census = audit_pins(&WILD_PINS).expect("the five audit clean");
        assert_eq!(census.measured, 0, "nothing here was measured by us");
        assert_eq!(census.published, 1, "compressive mode only");
        assert_eq!(census.stipulated, 1, "the damping half only");
        assert_eq!(census.owed, 3, "feldspar, Charles law, grain boundaries");
    }

    #[test]
    fn each_record_pin_is_declared_exactly_once() {
        for id in [
            PinId::FeldsparPotential,
            PinId::CharlesLaw,
            PinId::GrainBoundaryData,
            PinId::CompressiveMode,
            PinId::DampingRestitutionSplit,
        ] {
            assert_eq!(
                WILD_PINS.iter().filter(|p| p.id == id).count(),
                1,
                "{} declared other than once",
                id.label()
            );
        }
    }

    /// The two pins with a value in the tree must carry the SAME number the tree carries.
    /// A certificate transcribing a stale copy of a constant certifies the copy.
    #[test]
    fn the_warranted_pins_track_the_values_actually_shipped() {
        assert_eq!(
            pin(PinId::CompressiveMode).value,
            IsotropicMaterial::LAC_DU_BONNET.compressive_strength_pa
        );
        assert_eq!(
            pin(PinId::DampingRestitutionSplit).value,
            IsotropicMaterial::LAC_DU_BONNET.material_damping_ratio
        );
    }

    /// The A5 split is the reason the damping pin has a companion half with no value:
    /// restitution is not a material field, and the shipped solver number is 1-2 orders
    /// above what intrinsic granite permits. Both facts are load-bearing for the pin's
    /// text, so both are asserted rather than described.
    #[test]
    fn the_damping_pins_companion_half_is_really_absent_from_the_material_chart() {
        let demo = IsotropicMaterial::DEMO_CALIBRATION;
        assert!(demo.solver_damping_ratio > 10.0 * demo.material_damping_ratio);
        assert_eq!(IsotropicMaterial::LAC_DU_BONNET.solver_damping_ratio, 0.0);
    }

    // ---------------------------------------------------------------------------
    // MUTATION TESTS: the gate must fail, and must name which pin.
    // ---------------------------------------------------------------------------

    #[test]
    fn removing_a_pins_source_fails_the_gate_and_names_the_pin() {
        let mut pins = WILD_PINS;
        pins[3].warrant = "";
        assert_eq!(pins[3].id, PinId::CompressiveMode);

        let cert = certify_closing(&pins);
        assert_eq!(cert.status, ClosingStatus::Undeclarable);
        assert_eq!(
            cert.defect,
            Some(PinDefect {
                pin: PinId::CompressiveMode,
                field: PinField::MissingWarrant,
            })
        );
    }

    #[test]
    fn removing_the_other_sourced_pins_warrant_names_that_pin_instead() {
        // The gate must discriminate, not just fail somewhere.
        let mut pins = WILD_PINS;
        pins[4].warrant = "";
        assert_eq!(pins[4].id, PinId::DampingRestitutionSplit);
        assert_eq!(
            certify_closing(&pins).defect,
            Some(PinDefect {
                pin: PinId::DampingRestitutionSplit,
                field: PinField::MissingWarrant,
            })
        );
    }

    #[test]
    fn an_owed_pin_that_names_a_source_fails_the_gate() {
        let mut pins = WILD_PINS;
        pins[0].warrant = "a feldspar potential we do not have";
        assert_eq!(
            certify_closing(&pins).defect,
            Some(PinDefect {
                pin: PinId::FeldsparPotential,
                field: PinField::WarrantOnOwedPin,
            })
        );
    }

    #[test]
    fn upgrading_an_owed_pins_provenance_without_changing_its_kill_fails() {
        let mut pins = WILD_PINS;
        pins[1].provenance = PinProvenance::Published;
        pins[1].warrant = "Atkinson 1984";
        // The value would now claim a warrant while still being killed by a source
        // landing — the incoherence the biconditional exists to catch.
        assert_eq!(
            certify_closing(&pins).defect,
            Some(PinDefect {
                pin: PinId::CharlesLaw,
                field: PinField::KillMismatch,
            })
        );
    }

    #[test]
    fn an_absence_pin_carrying_a_band_fails() {
        let mut pins = WILD_PINS;
        pins[2].falsifier.lo = 0.0;
        assert_eq!(
            certify_closing(&pins).defect,
            Some(PinDefect {
                pin: PinId::GrainBoundaryData,
                field: PinField::StrayBand,
            })
        );
    }

    #[test]
    fn a_value_outside_its_own_kill_band_is_dead_on_arrival() {
        let mut pins = WILD_PINS;
        pins[3].value = 260.0e6;
        assert_eq!(
            certify_closing(&pins).defect,
            Some(PinDefect {
                pin: PinId::CompressiveMode,
                field: PinField::ValueOutsideOwnBand,
            })
        );
    }

    #[test]
    fn an_inverted_band_fails() {
        let mut pins = WILD_PINS;
        pins[4].falsifier.lo = 5.0e-3;
        pins[4].falsifier.hi = 5.0e-4;
        assert_eq!(
            certify_closing(&pins).defect,
            Some(PinDefect {
                pin: PinId::DampingRestitutionSplit,
                field: PinField::Band,
            })
        );
    }

    #[test]
    fn a_pin_with_no_falsifier_text_cannot_be_certified() {
        let mut pins = WILD_PINS;
        pins[0].falsifier.observable = "";
        assert_eq!(
            certify_closing(&pins).defect,
            Some(PinDefect {
                pin: PinId::FeldsparPotential,
                field: PinField::Observable,
            })
        );

        let mut pins = WILD_PINS;
        pins[0].falsifier.unlock = "";
        assert_eq!(
            certify_closing(&pins).defect,
            Some(PinDefect {
                pin: PinId::FeldsparPotential,
                field: PinField::Unlock,
            })
        );
    }

    #[test]
    fn declaring_the_same_pin_twice_fails() {
        let mut pins = WILD_PINS;
        pins[2] = pins[1];
        assert_eq!(
            certify_closing(&pins).defect,
            Some(PinDefect {
                pin: PinId::CharlesLaw,
                field: PinField::DuplicateId,
            })
        );
    }

    #[test]
    fn a_non_finite_value_fails() {
        let mut pins = WILD_PINS;
        pins[1].value = f64::NAN;
        assert_eq!(
            certify_closing(&pins).defect,
            Some(PinDefect {
                pin: PinId::CharlesLaw,
                field: PinField::Value,
            })
        );
    }

    #[test]
    fn a_refused_certificate_reports_no_census() {
        let mut pins = WILD_PINS;
        pins[0].quantity = "";
        let cert = certify_closing(&pins);
        assert_eq!(cert.status, ClosingStatus::Undeclarable);
        assert_eq!(cert.census.total(), 0, "a refusal counts nothing");
    }
}
