//! Below the nucleus: what this engine computes there, and where its record ends.
//!
//! `holon-render`'s sub-atom band (`hadron.rs`): the EXACT ground state of one-flavour QCD in
//! one space dimension on a chain of sites, in the colour-singlet sector, by the colour-lane
//! determinant solver; and `exp(-iHt)` on it by Lanczos. Two rungs draw it:
//!
//! * the baryon: the `B = 1` state's quark density per site beside the `B = 0` sea's, and the
//!   STRING: the colour-electric energy on each link, a tube between the sites; with one site
//!   pulled so that the state, and the string, move, live, under the exact Hamiltonian;
//! * the sea: the `B = 0` ground state on the same chain, with its own string.
//!
//! Below the sea the ladder CUTS to another theory's rung (the 3+1D gauge field, `gauge4`),
//! which owns the end of the record.
//!
//! WHY FOUR SITES. Measured on this machine in release: N = 4 solves both sectors in 14 ms
//! (216 and 64 determinants) and steps in 0.3 ms; N = 6 takes 0.7 to 0.9 s for the sea (8,000
//! determinants), 0.35 to 0.45 s for the baryon (3,375) and 60 ms a step. A host that allows
//! no task over 200 ms cannot solve six sites on load or on a frame, so the live rung is four.
//!
//! The solver's state is one per process (a table in `holon-render`). A scene that finds
//! another has used it since solves again and starts its own clock from zero.
use super::nucleus::{PROTON, PROTON_RADIUS_FM};
use super::*;
use holon_draw::ladder::Status;
use holon_draw_atoms::hadron as band;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

/// Sites, and the coupling `x = 1/(g a)^2` (the workbench's value).
pub const SITES: u32 = 4;
pub const X: f64 = 4.0;
/// The chain's length is not the model's to give: its only scale is the coupling. It is SET
/// here to the proton's charge diameter, by choice, so that the nucleus rung's proton and
/// this chain are the same size on screen.
pub const CHAIN_FM: f64 = 2.0 * PROTON_RADIUS_FM as f64;
/// The distance between neighbouring sites under that choice, in fm.
pub const SPACING_FM: f64 = CHAIN_FM / SITES as f64;
/// A column's height for one quark, in fm (three quarks, a full site, stand 0.63 fm).
pub const QUARK_FM: f64 = 0.21;
/// The pull: a one-body well of this depth (the model's energy units) on this site, the same
/// for every colour, and the model time run per second of wall time (the workbench's rate).
pub const PULL: f64 = -1.0;
pub const PULLED_SITE: u32 = 1;
pub const RATE: f64 = 0.6;
/// What the solve returns at N = 4, x = 4 (checked against the live solve by a test), as the
/// captions quote them.
pub const E_SEA: f64 = -24.539_116_685_962_6;
pub const E_BARYON: f64 = -17.584_776_187_572_345;
pub const BARYON_MASS: f64 = 1.738_585_124_597_563_9;
/// The links' colour-electric energies in the two ground states, as the captions quote them.
pub const SEA_LINKS: [f64; 3] = [0.940_297_219, 0.313_827_909, 0.940_297_219];
pub const BARYON_LINKS: [f64; 3] = [0.401_744_135, 0.913_737_198, 0.401_744_135];
/// The first energy gap of the chain with site 1 pulled by a well of depth 1 (`holon-render`'s
/// `hadron_band` test computes it and checks the stepped string against it).
pub const FIRST_GAP: f64 = 3.480_168;

/// The two ground states, as solved.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub sea: Vec<f64>,
    pub baryon: Vec<f64>,
    /// The colour-electric energy on each link, in the Hamiltonian's units.
    pub sea_links: Vec<f64>,
    pub baryon_links: Vec<f64>,
    pub e_sea: f64,
    pub e_baryon: f64,
    pub mass: f64,
    pub n_det: (f64, f64),
    pub residual: (f64, f64),
    pub exit: (u32, u32),
}

static RECORD: OnceLock<Record> = OnceLock::new();
/// Which scene last put the solver's `B = 1` state where it is.
static OWNER: Mutex<u64> = Mutex::new(0);
static NEXT: AtomicU64 = AtomicU64::new(1);

impl Record {
    /// Solve both sectors now (14 ms in release) and read them back.
    pub fn solve() -> Record {
        let _guard = OWNER.lock().unwrap_or_else(|p| p.into_inner());
        Self::solve_locked()
    }
    fn solve_locked() -> Record {
        let exit = (band::holon_hadron_solve(SITES, X, 0), band::holon_hadron_solve(SITES, X, 1));
        let occ = |b: i32| (0..SITES).map(|k| band::holon_hadron_occ(b, k)).collect::<Vec<_>>();
        let links = |b: i32| (0..SITES - 1).map(|k| band::holon_hadron_link_energy(b, k)).collect::<Vec<_>>();
        Record { sea: occ(0), baryon: occ(1), sea_links: links(0), baryon_links: links(1), e_sea: band::holon_hadron_energy(0), e_baryon: band::holon_hadron_energy(1), mass: band::holon_hadron_baryon_mass(), n_det: (band::holon_hadron_n_det(0), band::holon_hadron_n_det(1)), residual: (band::holon_hadron_residual(0), band::holon_hadron_residual(1)), exit }
    }
    /// The process's record: solved the first time anything asks.
    pub fn get() -> &'static Record {
        RECORD.get_or_init(Record::solve)
    }
}

/// Site `k`'s place along the chain in fm, the chain centred on the origin.
pub fn site_fm(k: u32) -> f64 {
    (k as f64 - 0.5 * (SITES - 1) as f64) * SPACING_FM
}

const FLOOR: [u8; 4] = [148, 163, 184, 255];
const SEA: [u8; 4] = [156, 178, 212, 255];

/// The radius a link's tube is DRAWN at, in fm: its cross-section in proportion to the link's
/// energy, 0.1 fm at the energy 4/3 (the Casimir of one quark's colour charge).
pub fn tube_fm(energy: f64) -> f64 {
    0.1 * (energy.max(0.0) / (4.0 / 3.0)).sqrt()
}

/// A link's colour: dim ember at no energy, bright at 4/3 and beyond.
fn glow(energy: f64) -> [u8; 4] {
    let t = (energy / (4.0 / 3.0)).clamp(0.0, 1.0) as f32;
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
    [mix(96, 255), mix(48, 232), mix(24, 150), 255]
}

/// The chain's floor in a unit of `per_fm` to the femtometre: a plate per site and, between
/// neighbouring sites, the link: a thin line, and (given the links' energies) the string, a
/// tube whose cross-section and brightness are the colour-electric energy on that link.
fn floor(per_fm: f32, px: f32, links: Option<&[f64]>) -> Vec<Instance> {
    let a = SPACING_FM as f32 * per_fm;
    let x = |k: u32| site_fm(k) as f32 * per_fm;
    // two to four pixels wide at any zoom: a link has no width of its own
    let w = (0.012 * a).clamp(2.0 * px, 4.0 * px);
    let mut m = vec![line([x(0), 0.0, 0.0], [x(SITES - 1), 0.0, 0.0], w, alpha(FLOOR, 0.7))];
    for k in 0..SITES {
        m.push(disc([x(k), 0.0, -0.002 * a], 0.36 * a, shade(FLOOR, 0.45)));
    }
    for (k, e) in links.unwrap_or(&[]).iter().enumerate() {
        m.push(rod([x(k as u32), 0.0, 0.0], [x(k as u32 + 1), 0.0, 0.0], tube_fm(*e) as f32 * per_fm, glow(*e)));
    }
    m
}

/// A column on site `k`: `quarks` high, `width` of a spacing across.
fn column(per_fm: f32, k: u32, from: f64, quarks: f64, width: f32, c: [u8; 4]) -> Instance {
    let a = SPACING_FM as f32 * per_fm;
    let h = QUARK_FM as f32 * per_fm;
    block([site_fm(k) as f32 * per_fm, 0.0, from as f32 * h], [width * a, width * a, (quarks as f32 * h).max(1e-4 * a)], c)
}

// ------------------------------------------------------------------ the baryon

pub struct Hadron {
    id: u64,
    /// The `B = 1` density and the links' energies as last read from the solver.
    pub occ: Vec<f64>,
    pub links: Vec<f64>,
    /// Model time since the pull began, the state's norm and its energy under the pulled
    /// Hamiltonian as last read (the two must hold still while the density moves).
    pub time: f64,
    pub norm: f64,
    pub energy: f64,
    reads: u64,
    seen: bool,
}

/// Tenths of a femtometre.
pub const UNIT: i32 = -16;

impl Hadron {
    pub fn new() -> Hadron {
        Hadron { id: NEXT.fetch_add(1, Ordering::Relaxed), occ: Record::get().baryon.clone(), links: Record::get().baryon_links.clone(), time: 0.0, norm: 1.0, energy: Record::get().e_baryon, reads: 0, seen: false }
    }
    /// Run the exact evolution for `dt` of model time (in steps of at most 0.05).
    pub fn evolve(&mut self, dt: f64) {
        let mut owner = OWNER.lock().unwrap_or_else(|p| p.into_inner());
        if *owner != self.id {
            // someone else moved the solver's state: back to the ground state, pulled anew
            band::holon_hadron_solve(SITES, X, 1);
            band::holon_hadron_grab(1, PULLED_SITE, PULL);
            *owner = self.id;
        }
        let mut left = dt;
        while left > 1e-12 {
            let step = left.min(0.05);
            band::holon_hadron_step(1, step);
            left -= step;
        }
        self.occ = (0..SITES).map(|k| band::holon_hadron_occ(1, k)).collect();
        self.links = (0..SITES - 1).map(|k| band::holon_hadron_link_energy(1, k)).collect();
        self.time = band::holon_hadron_time(1);
        self.norm = band::holon_hadron_norm(1);
        // the energy readout costs as much as a step: read on every thirtieth
        self.reads += 1;
        if self.reads % 30 == 1 {
            self.energy = band::holon_hadron_live_energy(1);
        }
    }
}

impl Default for Hadron {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for Hadron {
    fn name(&self) -> &'static str {
        "below the nucleus: a baryon on a chain"
    }
    fn unit(&self) -> i32 {
        UNIT
    }
    fn status(&self) -> Status {
        Status::Simulated
    }
    fn source(&self) -> &'static str {
        "Computed in this page by this repository's engine (holon-render's sub-atom band): the EXACT colour-singlet ground state of one-flavour QCD in ONE space dimension, 4 sites, x = 4, 64 determinants. Columns: each site's quark density summed over colours; grey ghost, the sea's 1.5; warm cap, the baryon's excess, three quarks in all. Tubes: the computed colour-electric energy on each link (the Casimir of the colour charge to its left; 0.40, 0.91, 0.40 at rest), the one-dimensional flux tube. IT MOVES because one site is pulled (gold well, depth 1) and the state runs under exp(-iHt) exactly: the outer links swing most (0.24 to 0.49), at the pulled chain's first energy gap, 3.48, every 3 s here; the middle one at the gaps near 7.3 and 9.2. NOT the proton and NOT the 3+1D vacuum of lattice QCD: no transverse gluons, Gauss's law fixes the field, the tube's width is drawn and not computed, the chain's length is SET to the proton's diameter (the ring) by choice, and it is four sites because six take a second to solve."
    }
    fn span(&self) -> (f64, f64) {
        (-14.6 - VIEW_DECADES, -14.1 - VIEW_DECADES)
    }
    fn advance(&mut self, dt: f64) {
        if self.seen && dt.is_finite() && dt > 0.0 {
            self.evolve(RATE * dt.min(0.1));
        }
        self.seen = false;
    }
    fn tiles(&mut self, z: f64, _clock: f64, _alpha: f64) -> Vec<Tile> {
        self.seen = true;
        let px = 10f64.powf(z - UNIT as f64) as f32; // this rung's units to the pixel
        let rec = Record::get();
        let per_fm = 10.0;
        let mut solid = floor(per_fm, px, Some(&self.links));
        let mut ghost = Vec::with_capacity(SITES as usize + 1);
        for k in 0..SITES {
            let (occ, sea) = (self.occ[k as usize], rec.sea[k as usize]);
            // up to the sea's level (or the state's, where it is lower) in the sea's colour,
            // and the excess over the sea above it, warmer the more there is (the workbench's
            // rule: full warmth at 1.5 quarks of excess)
            solid.push(column(per_fm, k, 0.0, occ.min(sea), 0.32, SEA));
            if occ > sea {
                let t = ((occ - sea) / 1.5).min(1.0) as f32;
                let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t) as u8;
                solid.push(column(per_fm, k, sea, occ - sea, 0.32, [mix(70, 240), mix(200, 150), mix(215, 60), 255]));
            }
            ghost.push(column(per_fm, k, 0.0, sea, 0.46, alpha(FLOOR, 0.3)));
        }
        // the pull: a well under its site, as deep as it is strong
        solid.push(column(per_fm, PULLED_SITE, PULL, -PULL, 0.24, [255, 214, 0, 230]));
        // the proton's charge radius, which the chain's length was set to: one ring, flat
        let (r, n) = (PROTON_RADIUS_FM * per_fm, 64);
        let at = |i: usize| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            [r * t.cos(), r * t.sin(), 0.0]
        };
        for i in 0..n {
            solid.push(line(at(i), at(i + 1), (0.02f32).max(2.0 * px), [PROTON[0], PROTON[1], PROTON[2], 150]));
        }
        vec![tile(1, 0, solid), tile(2, 1, ghost)]
    }
}

// ------------------------------------------------------------------ the sea

pub struct Sea;

impl Sea {
    pub fn new() -> Sea {
        Sea
    }
}

impl Default for Sea {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for Sea {
    fn name(&self) -> &'static str {
        "the gauge vacuum: the sea"
    }
    fn unit(&self) -> i32 {
        UNIT
    }
    fn status(&self) -> Status {
        Status::Simulated
    }
    fn source(&self) -> &'static str {
        "Computed in this page by the same engine: the B = 0 ground state of the same 4-site chain (one-flavour QCD in one space dimension, x = 4, 216 determinants, exact), the sea the baryon stands in. Its quark density is 1.5 on every site, half filling, summed over colours; the tubes are the computed colour-electric energy on each link (0.94, 0.31, 0.94 in the model's units), drawn as in the baryon's rung. It is an eigenstate and does not move, so nothing here moves. A model's vacuum, not the vacuum: one dimension, no transverse gluons, the gauge field removed by Gauss's law; the spacing (0.42 fm) is set by choice. The gauge vacuum this repository has MEASURED is a different model's: the Schwinger (U(1)) vacuum's magic density, 0.437 bits per site at x = 4, converged in volume to 3.7% (conformance/crystal/GF1_RESULTS.md)."
    }
    fn span(&self) -> (f64, f64) {
        (-15.0 - VIEW_DECADES, -14.6 - VIEW_DECADES)
    }
    /// Below the sea the ladder shows a different theory (the 3+1D gauge field's rung): a cut,
    /// not a fade, since a blend of the two would be a state of neither.
    fn cut_below(&self) -> bool {
        true
    }
    fn tiles(&mut self, z: f64, _clock: f64, _alpha: f64) -> Vec<Tile> {
        let px = 10f64.powf(z - UNIT as f64) as f32;
        let mut m = floor(10.0, px, Some(&Record::get().sea_links));
        for (k, sea) in Record::get().sea.iter().enumerate() {
            m.push(column(10.0, k as u32, 0.0, *sea, 0.32, SEA));
        }
        vec![tile(1, 0, m)]
    }
}
