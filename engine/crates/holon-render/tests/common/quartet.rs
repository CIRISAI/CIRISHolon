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

/// The bank and both three-body tables, loaded, and no atoms yet. The half both constructors
/// below share. The four-body order is deliberately NOT set here: both constructors set it
/// after they have placed the scene, because setting it before would put the four-body sector
/// into the opening force pass, which is not what either of them did.
fn banked_sim() -> Box<Sim> {
    let b = banked();
    let mut s = Box::new(Sim::empty());
    assert_eq!(load_pair_table(&mut s, &b.hh, Host::Native), TABLE_OK);
    assert_eq!(load_pair_table(&mut s, &b.oh, Host::Native), TABLE_OK);
    assert_eq!(load_pair_table(&mut s, &b.oo, Host::Native), TABLE_OK);
    s.trimer = (*b.trimer).clone();
    s.water = (*b.water).clone();
    s
}

/// A scene of `positions.len()` atoms with the species and coordinates given, at rest,
/// with the bank and both three-body tables loaded and the four-body sector set as asked.
///
/// # Why this one still opens on the placeholder
///
/// `reset(n)` lays a placeholder configuration — a ring or shell of radius 6 bohr — and takes
/// the ledger's baselines (`l0`, `p0`, `e_ref`, the curvature envelope) ON IT, before the real
/// coordinates arrive. For a scene that is `rebase`d afterwards that is invisible and pure
/// cost, and [`scene_placed`] is the constructor for those. For a scene that is NOT rebased
/// it is not invisible: `quartet(false)` is stepped straight from here into the banked
/// receipt `tests/data/channel_ledger.receipt`, whose `quartet.drift` line is the real scene's
/// ledger measured against the PLACEHOLDER's `l0`. Moving that is a decision about a banked
/// record, not about the cost of construction, so this constructor is left exactly as it was
/// and the dependence is named here rather than discovered by a failing receipt.
///
/// The scenes that carry the cost — the liquid boxes, through `field2_scenes::scene` — all
/// rebase, and they take [`scene_placed`].
pub fn scene(species: &[holon_chem::elements::Species], positions: &[[f64; 3]], de4: bool) -> Box<Sim> {
    assert_eq!(species.len(), positions.len());
    let mut s = banked_sim();
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

/// THE SAME SCENE, PLACED: the species and coordinates installed before any force is
/// evaluated, so no placeholder configuration is ever built.
///
/// Identical to [`scene`] in every float for a caller that `rebase`s afterwards — `rebase` IS
/// `zero_ledger`, so it retakes every baseline the placeholder had touched — and that identity
/// is machine-checked on a 54-water box by `examples/liquid2.rs::instrument_residual`, which
/// builds both and compares coordinates, velocities, forces, energies, `l0`, `p0`, `l0_ang`,
/// `e_ref` and the step bit for bit.
///
/// What it saves is the placeholder itself. The opener puts every atom on a 6-bohr shell
/// whatever the atom count, so at liquid-box sizes every atom is inside every other's
/// three-body cutoff and the cutoff-local triple enumeration degenerates to the complete
/// `C(N, 3)`. Measured, one process per size (`liquid2/size/cost_before.json` against
/// `cost_after.json`): peak resident set 0.953 GiB and 2.69 s of construction at 384 atoms
/// against 0.192 GiB and 0.81 s, and 6.852 GiB and 17.17 s at 750 atoms against 0.445 GiB and
/// 2.32 s. The 432-water box that could not be built at all now builds in 0.947 GiB.
pub fn scene_placed(species: &[holon_chem::elements::Species], positions: &[[f64; 3]], de4: bool) -> Box<Sim> {
    assert_eq!(species.len(), positions.len());
    let mut s = banked_sim();
    assert!(s.reset_with(species, positions), "the bank refused a species this scene carries");
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
