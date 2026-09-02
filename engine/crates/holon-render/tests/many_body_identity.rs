//! THE RECEIPT: the generic k-body sector is BIT-IDENTICAL to the (O,H,H,H) sector it
//! replaced, on two staked scenes.
//!
//! # Why a written receipt and not a tolerance
//!
//! A generalisation that moves the last bits of a many-body force is not a refactor — it
//! is a new model wearing the old name, and every trajectory and gate banked on the old
//! sector would be grading a different force while reporting the same headline (the rule
//! `tests/mbe_generic_identity.rs` states for the value path). So the sector's energy, its
//! virial, its evaluation count and EVERY force component on every atom were written to
//! `tests/data/many_body_identity.receipt` by the four-body sector as it stood before the
//! generalisation, as raw `f64` bits, and this gate requires the current sector to
//! reproduce them exactly.
//!
//! The receipt is machine-written, never hand-typed: run with
//! `HOLON_MANY_BODY_RECEIPT=write` to regenerate it, and do that ONLY at a commit whose
//! sector is the one being frozen — regenerating it after a change is how a change hides.
//!
//! The two scenes are `tests/common/quartet.rs`'s: one hub, and two hubs whose reaches
//! overlap so the enumeration itself is under test.

#[path = "common/quartet.rs"]
mod quartet;

use holon_render::sim::Sim;
use std::fmt::Write as _;

const RECEIPT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/many_body_identity.receipt");

struct Reading {
    name: &'static str,
    /// The sector's energy. Written under the receipt's original key `e_four=`, which is a
    /// label in a frozen file and not a claim about the sector's order.
    e_many: u64,
    virial: u64,
    evals: u64,
    forces: Vec<[u64; 3]>,
}

fn read(name: &'static str, mut s: Box<Sim>) -> Reading {
    assert!(s.pairs_ready(), "{name}: the bank is missing a curve this scene needs");
    s.compute_forces();
    let forces = (0..s.n)
        .map(|i| {
            let (x, y, z) = s.internal_force(i);
            [x.to_bits(), y.to_bits(), z.to_bits()]
        })
        .collect();
    Reading {
        name,
        e_many: s.e_many.to_bits(),
        virial: s.many_body_cached_virial.to_bits(),
        evals: s.many_body_evals,
        forces,
    }
}

fn render(rs: &[Reading]) -> String {
    let mut out = String::new();
    for r in rs {
        writeln!(out, "scene {} e_four={:016x} virial={:016x} evals={}", r.name, r.e_many, r.virial, r.evals).unwrap();
        for (i, f) in r.forces.iter().enumerate() {
            writeln!(out, "f {i} {:016x} {:016x} {:016x}", f[0], f[1], f[2]).unwrap();
        }
    }
    out
}

fn readings() -> Vec<Reading> {
    vec![
        read("quartet", quartet::quartet(true)),
        read("two_hubs", quartet::two_hubs(true)),
    ]
}

#[test]
fn the_many_body_sector_reproduces_its_receipt_bit_for_bit() {
    let now = render(&readings());
    if std::env::var("HOLON_MANY_BODY_RECEIPT").as_deref() == Ok("write") {
        std::fs::write(RECEIPT, &now).expect("the receipt is writable");
        eprintln!("WROTE {RECEIPT}");
        return;
    }
    let banked = std::fs::read_to_string(RECEIPT).expect("the receipt exists; write it at the frozen commit");
    assert!(
        banked.lines().filter(|l| l.starts_with("scene ")).count() == 2,
        "the receipt does not carry both scenes"
    );
    if banked != now {
        let mut diff = String::new();
        for (a, b) in banked.lines().zip(now.lines()) {
            if a != b {
                writeln!(diff, "  banked: {a}\n  now:    {b}").unwrap();
            }
        }
        panic!(
            "THE MANY-BODY SECTOR MOVED BITS against the frozen receipt. A generalisation that \
             changes the arithmetic is a new model wearing the old name; either the summation \
             order regressed or the physics changed, and either way it is not a refactor:\n{diff}"
        );
    }
    // And the receipt is not vacuous: the sector actually evaluated something on both.
    for r in readings() {
        assert!(r.evals > 0, "{}: no quadruple was evaluated, so the receipt pins nothing", r.name);
        assert!(f64::from_bits(r.e_many) != 0.0, "{}: the sector's energy is exactly zero", r.name);
    }
}

#[test]
fn the_second_scene_enumerates_across_the_hubs() {
    // The two-hub scene is only a test of the enumeration if the second oxygen really sees
    // four hydrogens: one quadruple from its own three plus three straddling ones.
    let mut s = quartet::two_hubs(true);
    s.compute_forces();
    assert_eq!(
        s.many_body_evals, 5,
        "expected 1 (hub A) + 4 (hub B: C(4,3)) evaluated quadruples, got {}",
        s.many_body_evals
    );
}

#[test]
fn an_order_with_no_measured_reach_is_refused_by_name_not_zeroed() {
    // Order five on a scene whose only measured reach is OHHH at order four: the sector
    // has no radius to enumerate to, evaluates nothing, and says so in its own counter
    // rather than running a five-cluster to a guessed cutoff. The energy is exactly zero
    // BECAUSE nothing was served, and the receipt of that is the count, not the zero.
    let mut s = quartet::two_hubs(true);
    s.many_body_order = 5;
    s.compute_forces();
    assert_eq!(s.many_body_evals, 0, "no class has a measured reach at order five");
    assert_eq!(s.e_many, 0.0);
    assert_eq!(s.many_body_cutoff(), 0.0, "no reach, no cutoff — nothing was guessed");
    // and back at four the same scene is served and its reach is the table's own
    s.many_body_order = 4;
    s.many_body_cached_valid = false;
    s.compute_forces();
    assert_eq!(s.many_body_evals, 5);
    assert_eq!(s.many_body_cutoff(), holon_chem::quaternary_table::R_HI);
}
