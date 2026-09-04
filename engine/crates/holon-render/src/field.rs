//! THE EMBEDDING FIELD IN THE FORCE LAW — FIELD-1 (`conformance/water_observatory/FIELD_PREREG.md`).
//!
//! Point charges on every census-certified water molecule, derived from the engine's own
//! density — the dipole-exact charge of the isolated monomer at EMBED-1's pin — and the
//! Coulomb term between DIFFERENT molecules. Forces are internal (they cancel pairwise from
//! the momentum sum), the virial gains the energy, `energy()` gains `e_field`, and nothing is
//! posted to `w_ext` for the conservative term. A change in the charge ASSIGNMENT (a row
//! forms or dies, the field is enabled or disabled) is an energy jump at fixed positions and
//! is posted to `w_ext` and to `work.field` as one ledgered event — the ACUITY-B pattern.
//!
//! The periodic box is REFUSED while the field is on: a bare Coulomb sum under wrapping is
//! conditionally convergent and Ewald is the named exit, not built. Polarisation and the
//! charges' geometry dependence are FIELD-2, not claimed here.

use holon_chem::elements::{by_z, Species};
use holon_chem::embed::{fragment_charges, monomer, water_centers, ChargeModel, Fragment};

/// EMBED-1's water pin (`conformance/water_observatory/embed/pins.json`), the geometry the
/// charge is derived at. Declared here because the charge is a constant of the model.
pub const WATER_PIN_R_BOHR: f64 = 1.9435738400;
pub const WATER_PIN_THETA_RAD: f64 = 1.6887434037;

/// The dipole-exact hydrogen charge of the isolated water monomer at the pin, from the
/// engine's own density (`q_O = −2q`). One full-CI solve of 441 determinants; computed when
/// the field is enabled and carried in the checkpoint.
pub fn water_charge_at_pin() -> f64 {
    let (o, h): (Species, Species) = (by_z(8).expect("oxygen"), by_z(1).expect("hydrogen"));
    let frag = Fragment::new(vec![o, h, h], water_centers(WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD).to_vec(), vec![-2.0, 1.0, 1.0]);
    let m = monomer(&frag, &[], ChargeModel::DipoleExact);
    let q = fragment_charges(ChargeModel::DipoleExact, &frag, &m.p, &m.solve.basis, &m.mom);
    q[1]
}

/// The field model: one number, the hydrogen charge; zero means off.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldModel {
    pub q_h: f64,
}

/// Deliberate defects for the gates (FIELD_PREREG §3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldPlant {
    #[default]
    None,
    /// The reaction on `j` dropped while `i`'s force is applied (sector: momentum).
    DropReaction,
    /// A charge-assignment transition applied without posting it (sector: the ledger).
    SkipLedger,
    /// The energy negated (sector: the force-derivative gate).
    FlipSign,
}

/// The field's own counters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FieldWork {
    /// Charge-assignment transitions posted (or, under the plant, applied unposted).
    pub transitions: u64,
    /// Charged pairs evaluated in the last force pass.
    pub pairs: u64,
}

impl FieldWork {
    pub const fn zero() -> Self {
        Self { transitions: 0, pairs: 0 }
    }
}

/// Why the field could not be enabled, or the box could not be wrapped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldRefusal {
    /// A bare Coulomb sum under a wrapping boundary is conditionally convergent.
    PeriodicNeedsEwald,
}

impl core::fmt::Display for FieldRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FieldRefusal::PeriodicNeedsEwald => write!(
                f,
                "REFUSED: the embedding field under a wrapping boundary is a conditionally \
                 convergent Coulomb sum; Ewald (or PME) is the exit and is not built. Use walls \
                 or an open box, or leave the field off."
            ),
        }
    }
}

/// Is this set of atoms a water unit's composition: one oxygen and two hydrogens. The
/// assignment itself is the engine's bond verdict (FIELD_AMENDMENT_1); this is the shape test.
pub fn is_water_row(zs: &[u32]) -> bool {
    zs.len() == 3 && zs.iter().filter(|&&z| z == 8).count() == 1 && zs.iter().filter(|&&z| z == 1).count() == 2
}

/// The water units under the current charge assignment: `(oxygen, [h1, h2])` for every unit.
pub fn water_units(charge_row: &[u32], z: &[u32]) -> Vec<(usize, [usize; 2])> {
    let mut out = Vec::new();
    for o in 0..z.len() {
        if z[o] == 8 && charge_row[o] == o as u32 {
            let hs: Vec<usize> = (0..z.len()).filter(|&h| h != o && charge_row[h] == o as u32).collect();
            if hs.len() == 2 {
                out.push((o, [hs[0], hs[1]]));
            }
        }
    }
    out
}
