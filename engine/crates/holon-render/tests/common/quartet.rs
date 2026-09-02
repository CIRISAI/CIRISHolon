//! Staked many-body scenes shared by the sector's gates.
//!
//! One fixture, included by `#[path]` (the arrangement `tests/common/b1_dump.rs` uses),
//! so the momentum audit and the bit-identity receipt judge the SAME atoms. Two scenes:
//!
//! * `quartet` — one oxygen with three hydrogens well inside the four-body reach, the
//!   switch fully on, deliberately asymmetric (a symmetric one can cancel a broken force
//!   by its own geometry).
//! * `two_hubs` — two such clusters seven bohr apart, placed so that ONE hydrogen of the
//!   first sits inside the second oxygen's reach: the second hub therefore sees four
//!   hydrogens and enumerates four quadruples, three of them straddling the clusters with
//!   the switch partly on. That is the enumeration a generic hub rule has to reproduce.

use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{generate_pair_table, PairTable};
use holon_render::bank::Host;
use holon_render::sim::Sim;
use holon_render::{load_pair_table, TABLE_OK};
use std::sync::OnceLock;

pub const FIXTURE_KNOTS: usize = 48;

pub struct Bank {
    pub hh: PairTable,
    pub oh: PairTable,
    /// The two-hub scene has two oxygens, so its pair sector needs the O-O curve too.
    pub oo: PairTable,
    pub trimer: Box<holon_chem::trimer::TrimerTable>,
    pub water: Box<holon_chem::water::WaterTable>,
}

pub fn banked() -> &'static Bank {
    static B: OnceLock<Bank> = OnceLock::new();
    B.get_or_init(|| Bank {
        hh: generate_pair_table(HYDROGEN, HYDROGEN, FIXTURE_KNOTS),
        oh: generate_pair_table(OXYGEN, HYDROGEN, FIXTURE_KNOTS),
        oo: generate_pair_table(OXYGEN, OXYGEN, FIXTURE_KNOTS),
        trimer: Box::new(holon_chem::trimer::generate().expect("the H3 table generates")),
        water: Box::new(
            holon_chem::water::from_text(&water_table_text())
                .expect("the committed (O,H,H) table parses under this build's grid rule"),
        ),
    })
}

fn water_table_text() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../holon-chem/tests/data/s2/s2_water_table.txt"
    ))
    .expect("the committed water table is readable")
}

/// A scene of `positions.len()` atoms with the species and coordinates given, at rest,
/// with the bank and both three-body tables loaded and the four-body sector set as asked.
pub fn scene(species: &[holon_chem::elements::Species], positions: &[[f64; 3]], de4: bool) -> Box<Sim> {
    assert_eq!(species.len(), positions.len());
    let b = banked();
    let mut s = Box::new(Sim::empty());
    assert_eq!(load_pair_table(&mut s, &b.hh, Host::Native), TABLE_OK);
    assert_eq!(load_pair_table(&mut s, &b.oh, Host::Native), TABLE_OK);
    assert_eq!(load_pair_table(&mut s, &b.oo, Host::Native), TABLE_OK);
    s.trimer = (*b.trimer).clone();
    s.water = (*b.water).clone();
    s.reset(species.len());
    for (i, sp) in species.iter().enumerate() {
        assert!(s.set_species(i, *sp));
    }
    for (i, c) in positions.iter().enumerate() {
        s.atoms[i].x = c[0];
        s.atoms[i].y = c[1];
        s.atoms[i].z = c[2];
        s.atoms[i].vx = 0.0;
        s.atoms[i].vy = 0.0;
        s.atoms[i].vz = 0.0;
    }
    s.many_body_order = if de4 { 4 } else { 0 };
    s
}

pub const QUARTET_POSITIONS: [[f64; 3]; 4] = [
    [0.0, 0.0, 0.0],
    [1.83, 0.0, 0.0],
    [-0.61, 1.94, 0.0],
    [-0.55, -0.72, 1.71],
];

/// One oxygen with three hydrogens well inside the four-body reach.
pub fn quartet(de4: bool) -> Box<Sim> {
    scene(&[OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN], &QUARTET_POSITIONS, de4)
}

/// Two hubs seven bohr apart; hydrogen 1 of the first cluster is 5.2 bohr from the second
/// oxygen, inside its reach and inside the switch, so the second hub enumerates four
/// hydrogens. Species order is deliberately NOT hub-first for the second cluster: its
/// oxygen is atom 6, after two of its hydrogens, so a hub rule that assumed "the first
/// atom" or "the lowest index" would miss it.
pub fn two_hubs(de4: bool) -> Box<Sim> {
    let species = [
        OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN, // cluster A: atoms 0..4
        HYDROGEN, HYDROGEN, OXYGEN, HYDROGEN, // cluster B: hydrogens 4, 5, 7 around oxygen 6
    ];
    let positions = [
        [3.0, 3.0, 3.0],
        [4.83, 3.0, 3.0],
        [2.39, 4.94, 3.0],
        [2.45, 2.28, 4.71],
        [9.3, 5.3, 3.4],
        [9.6, 2.7, 4.9],
        [10.0, 3.5, 3.2],
        [11.6, 3.9, 3.1],
    ];
    scene(&species, &positions, de4)
}
