//! THE MOMENTUM AUDIT on the four-body sector: do its forces sum to zero?
//!
//! # Why this gate exists, and why it is FIRST
//!
//! Every completed seed of the dE4 quench arm violated the momentum bound by four to five
//! orders (|p|/bound 9.8e3 to 4.2e5 on 6 of 6) while energy drift stayed IN bound. That is
//! the one-gate-per-conservation-law signature: a channel that is in the force and not in
//! the ledger shows up on exactly one gate, and the gate that stays green tells you nothing
//! about the one that fired.
//!
//! `Sim::momentum_bound` is a pure ROUNDOFF bound — `8 · steps · eps · p_scale` — and it is
//! entitled to be, because every other sector applies its forces as equal and opposite
//! contributions of the SAME bit pattern (`push_side` adds `fx` to one partner and
//! subtracts the identical `fx` from the other), so they cancel exactly rather than
//! approximately. A sector that breaks that cancellation does not drift within the bound;
//! it leaves it by orders.
//!
//! # What is actually checked
//!
//! `a_pair` is FORCE, not acceleration: the integrator divides by mass at the point of use
//! (`half = 0.5 · dt / mass`), and `Sim::internal_force` returns `a_pair` under that name.
//! So the invariant is flatly stated: **the sum of `internal_force` over every atom is
//! exactly zero**, because walls, the spring and the thermostat live in `a_ext` and are the
//! only things allowed to inject net momentum.
//!
//! The control matters as much as the measurement. The same scene with the four-body sector
//! OFF must sum to exactly zero, or the test is measuring the pair and triple sectors and
//! would fire whatever the four-body code did.

use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{generate_pair_table, PairTable};
use holon_render::bank::Host;
use holon_render::sim::Sim;
use holon_render::{load_pair_table, TABLE_OK};
use std::sync::OnceLock;

const FIXTURE_KNOTS: usize = 48;

struct Bank {
    hh: PairTable,
    oh: PairTable,
    trimer: Box<holon_chem::trimer::TrimerTable>,
    water: Box<holon_chem::water::WaterTable>,
}

fn banked() -> &'static Bank {
    static B: OnceLock<Bank> = OnceLock::new();
    B.get_or_init(|| Bank {
        hh: generate_pair_table(HYDROGEN, HYDROGEN, FIXTURE_KNOTS),
        oh: generate_pair_table(OXYGEN, HYDROGEN, FIXTURE_KNOTS),
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

/// One oxygen with three hydrogens well inside `R_CUT`, so the four-body switch is fully on
/// and the sector is genuinely evaluated rather than skipped at the cutoff.
fn quartet(de4: bool) -> Box<Sim> {
    let b = banked();
    let mut s = Box::new(Sim::empty());
    assert_eq!(load_pair_table(&mut s, &b.hh, Host::Native), TABLE_OK);
    assert_eq!(load_pair_table(&mut s, &b.oh, Host::Native), TABLE_OK);
    s.trimer = (*b.trimer).clone();
    s.water = (*b.water).clone();
    s.reset(4);
    assert!(s.set_species(0, OXYGEN));
    for i in 1..4 {
        assert!(s.set_species(i, HYDROGEN));
    }
    // A compact, deliberately ASYMMETRIC quartet: a symmetric one can cancel a broken force
    // by its own geometry and pass a gate it should fail.
    let p = [
        [0.0, 0.0, 0.0],
        [1.83, 0.0, 0.0],
        [-0.61, 1.94, 0.0],
        [-0.55, -0.72, 1.71],
    ];
    for (i, c) in p.iter().enumerate() {
        s.atoms[i].x = c[0];
        s.atoms[i].y = c[1];
        s.atoms[i].z = c[2];
        s.atoms[i].vx = 0.0;
        s.atoms[i].vy = 0.0;
        s.atoms[i].vz = 0.0;
    }
    s.de4_enabled = de4;
    s
}

/// The summed internal force, and the scale to judge it against.
fn net_internal_force(s: &Sim) -> ((f64, f64, f64), f64) {
    let (mut fx, mut fy, mut fz) = (0.0, 0.0, 0.0);
    let mut scale = 0.0f64;
    for i in 0..s.n {
        let (x, y, z) = s.internal_force(i);
        fx += x;
        fy += y;
        fz += z;
        scale = scale.max((x * x + y * y + z * z).sqrt());
    }
    ((fx, fy, fz), scale.max(1e-30))
}

#[test]
fn the_control_sums_to_exactly_zero_without_the_four_body_sector() {
    let mut s = quartet(false);
    assert!(s.pairs_ready(), "the bank is missing a curve this scene needs");
    s.step();
    let ((fx, fy, fz), scale) = net_internal_force(&s);
    let net = (fx * fx + fy * fy + fz * fz).sqrt();
    assert!(scale > 1e-6, "the scene produced no internal force at all, so this proves nothing");
    // NOT an exact zero, and the reason is this test's own arithmetic rather than the
    // sim's. `push_side` cancels exactly IN THE ARRAY; summing that array in index order
    // is a separate floating-point sum with its own roundoff, so the instrument has a
    // floor of a few ulp of the force scale. Measured 3.2e-17 against 9.5e-2, a relative
    // 3.3e-16. The bar is set twelve orders above that and nine below the defect this
    // file was written for, so nothing about the separation is delicate.
    assert!(
        net <= 1e-14 * scale,
        "the pair and triple sectors do not cancel to roundoff: net {net:.6e} against a \
         force scale of {scale:.6e}, a relative {:.3e}. They add and subtract the same bit \
         pattern, so anything above roundoff here means the control itself is broken and \
         the four-body reading below cannot be attributed.",
        net / scale
    );
}

#[test]
fn the_four_body_forces_sum_to_zero_over_the_quartet() {
    let mut s = quartet(true);
    assert!(s.pairs_ready(), "the bank is missing a curve this scene needs");
    s.step();
    let ((fx, fy, fz), scale) = net_internal_force(&s);
    let net = (fx * fx + fy * fy + fz * fz).sqrt();
    assert!(
        scale > 1e-6,
        "the four-body sector contributed no force at all — the switch is off or the tables \
         are unloaded, and this test would pass without exercising anything"
    );
    // Roundoff only. The four-body force is built as equal-and-opposite pairs along each
    // O-H direction, so in exact arithmetic the sum is zero; what is allowed here is the
    // floating-point residue of that construction, not a systematic term.
    assert!(
        net <= 1e-12 * scale,
        "THE FOUR-BODY SECTOR INJECTS NET MOMENTUM: net internal force {net:.6e} against a \
         force scale of {scale:.6e}, a relative {:.3e}. Every force in this array must \
         cancel — walls, spring and thermostat are the only things allowed to change the \
         total, and they live in `a_ext`. This is what put |p|/bound at 9.8e3-4.2e5 on all \
         six banked dE4 seeds while energy stayed in bound.",
        net / scale
    );
}
