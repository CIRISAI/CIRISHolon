//! THE CHANNEL LEDGER'S GATES (`src/channel.rs`; OBJECT.md design rule 10).
//!
//! The channel ledger is a set of DECLARATIONS beside the force law, plus one allocator
//! that three dialects now call, plus an energy sum derived from a row table. None of it
//! may move a bit. So the first gate is a written receipt: every ledger row, both sums,
//! the physics digest, the receipt columns, the derived pair cutoffs at three budgets and
//! the far sector's closed-form radii were written to `tests/data/channel_ledger.receipt`
//! by the engine AS IT STOOD BEFORE `channel.rs` EXISTED (the example
//! `channel_receipt` run against the parent commit's library), and this test requires the
//! engine after it to reproduce every line. The remaining gates read the declarations
//! against the engine: the energy fold beside the hand-written chain, the standing report
//! against the rows, the one allocator against each dialect it replaced, and the exponent
//! reading against a tail that is the law and one that is not.

#[path = "common/channel_scenes.rs"]
#[allow(dead_code)]
mod channel_scenes;

use channel_scenes::{far_reading, four_waters, power, run, BUDGETS, FLOORS, RECEIPT, WATER_STEPS};
use holon_render::channel::{
    reach_for_budget, rows_carrying, Carriage, ChannelId, Kernel, Kind, Reach, Row, Shape, CHANNELS,
};
use holon_render::longrange::{FarRefusal, FarSector};
use holon_render::sim::{Boundary, Dims, Sim, PAIR_SWITCH_WIDTH};
use std::sync::OnceLock;

/// The four-water scene with the field on, stepped once for every gate below.
fn water() -> &'static Sim {
    static S: OnceLock<Box<Sim>> = OnceLock::new();
    S.get_or_init(|| {
        let mut w = four_waters(Boundary::Walls);
        w.set_field(true, None).expect("walls admit the field");
        run(&mut w, WATER_STEPS);
        w
    })
}

// ------------------------------------------------------------------ THE RECEIPT

#[test]
fn the_engine_reproduces_the_pre_ledger_receipt_bit_for_bit() {
    let want = std::fs::read_to_string(RECEIPT).expect(
        "tests/data/channel_ledger.receipt is missing: it is written ONLY by \
         `HOLON_CHANNEL_RECEIPT=write cargo run --example channel_receipt` at the commit \
         whose force law is being frozen",
    );
    let got = channel_scenes::receipt();
    let mut diffs = Vec::new();
    for (w, g) in want.lines().zip(got.lines()) {
        if w != g {
            diffs.push(format!("  receipt: {w}\n  now:     {g}"));
        }
    }
    let (nw, ng) = (want.lines().count(), got.lines().count());
    assert!(
        diffs.is_empty() && nw == ng,
        "the channel ledger moved {} of {} receipt lines (line counts {nw} vs {ng}):\n{}",
        diffs.len(),
        nw,
        diffs.join("\n")
    );
}

// ------------------------------------------------------ THE SUM, DERIVED FROM ROWS

#[test]
fn energy_folded_over_the_row_table_is_the_hand_written_chain() {
    let s = water();
    let chain = s.e_kin + s.e_pair + s.e_three + s.e_many + s.e_far + s.e_field + s.e_wall + s.e_spring + s.e_grav;
    assert_eq!(s.energy().to_bits(), chain.to_bits());
    assert_eq!(s.ledger().to_bits(), (chain - s.w_ext).to_bits());
    // every row reads its field, in the ledger's order
    let by_row: Vec<f64> = Row::ALL.iter().map(|r| s.row(*r)).collect();
    let fields = [s.e_kin, s.e_pair, s.e_three, s.e_many, s.e_far, s.e_field, s.e_wall, s.e_spring, s.e_grav];
    for (a, b) in by_row.iter().zip(fields.iter()) {
        assert_eq!(a.to_bits(), b.to_bits());
    }
    // the scene is live on the rows that matter: not a gate on zeros
    assert!(s.e_pair != 0.0 && s.e_three != 0.0 && s.e_field != 0.0 && s.e_kin != 0.0, "vacuous scene");
}

// ---------------------------------------------------------------- THE DECLARATIONS

#[test]
fn the_five_channels_wear_the_five_thing_kinds_in_rate_order() {
    let kinds: Vec<Kind> = CHANNELS.iter().map(|c| c.kind).collect();
    assert_eq!(kinds, [Kind::Circumstances, Kind::Structure, Kind::Process, Kind::Rules, Kind::Identity]);
    let powers: Vec<Option<f64>> = CHANNELS.iter().map(|c| c.rate.power()).collect();
    assert_eq!(powers, [Some(1.0), Some(4.0), Some(6.0), Some(9.0), None]);
    let shapes: Vec<Shape> = CHANNELS.iter().map(|c| c.shape).collect();
    assert_eq!(shapes, [Shape::Sum, Shape::FixedPoint, Shape::Sum, Shape::Sum, Shape::Solve]);
    assert_eq!(CHANNELS.iter().map(|c| c.arity).collect::<Vec<_>>(), [2, 2, 2, 3, 2]);
    assert_eq!(ChannelId::Field.record().receipt, Some("work.field"));
    for c in CHANNELS.iter().skip(1) {
        assert_eq!(c.receipt, None, "{:?} is conservative and posts no receipt", c.id);
    }
}

#[test]
fn the_standing_report_reads_the_rows_it_names_and_nothing_else() {
    let s = water();
    let standing = s.channel_standing();
    assert_eq!(standing.len(), 5);
    for st in standing.iter() {
        assert_eq!(rows_carrying(st.channel.id).len(), st.rows.len());
        for (row, carriage, v) in st.rows.iter() {
            assert!(row.carries().contains(&(st.channel.id, *carriage)));
            assert_eq!(v.to_bits(), s.row(*row).to_bits());
        }
    }
    // the field is on: channel 1 has its own whole row and reaches the scene
    let field = &standing[ChannelId::Field as usize];
    assert!(field.has_own_row());
    assert_eq!(field.reach, Reach::Scene);
    assert_eq!(field.rows.iter().find(|(r, _, _)| *r == Row::Field).unwrap().2.to_bits(), s.e_field.to_bits());
    // induction has no row of its own in this engine — FIELD-2 is named, not built
    let ind = &standing[ChannelId::Induction as usize];
    assert!(!ind.has_own_row());
    assert_eq!(ind.reach, Reach::Absent);
    // three-body reaches by the tables' declared reach
    let three = &standing[ChannelId::ThreeBody as usize];
    assert!(matches!(three.reach, Reach::Radius { r, .. } if r > 0.0));
    // the sectors and the channels are different partitions: some row carries more than
    // one channel, and some channel is carried by more than one row
    assert!(Row::ALL.iter().any(|r| r.carries().len() > 1));
    assert!(CHANNELS.iter().any(|c| rows_carrying(c.id).len() > 1));
    // and no interaction row carries nothing
    for r in Row::ALL.iter() {
        let container = matches!(r, Row::Kin | Row::Wall | Row::Spring | Row::Grav);
        assert_eq!(container, r.carries().is_empty(), "{:?}", r);
    }
    // FOLDED is the honest carriage for the pair table: it never claims a channel's value
    assert!(Row::Pair.carries().iter().all(|(_, c)| *c == Carriage::Folded));
}

// --------------------------------------------------------------- THE ONE ALLOCATOR

/// `Sim::derive_pair_cutoff` AS IT STOOD before the allocator existed, kept here so the
/// collapse is gated on identity with the dialect it replaced, not on a tolerance.
fn derive_pair_cutoff_before(s: &Sim, floor: f64) -> Option<(f64, f64)> {
    if !(floor > 0.0) {
        return None;
    }
    let (slots, ns) = s.active_slots();
    let mut r_in = 0.0f64;
    let mut any = false;
    for &slot in slots[..ns].iter() {
        let t = s.bank.table_slot(slot);
        if !t.is_loaded() {
            continue;
        }
        any = true;
        let base = t.r_max();
        if t.u(base).abs() <= floor {
            r_in = r_in.max(base);
            continue;
        }
        let mut hi = base + 1.0;
        let mut guard = 0;
        while t.u(hi).abs() > floor && guard < 64 {
            hi = base + (hi - base) * 2.0;
            guard += 1;
        }
        if t.u(hi).abs() > floor {
            return None;
        }
        let mut lo = base;
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if t.u(mid).abs() > floor {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        r_in = r_in.max(hi);
    }
    if !any {
        return None;
    }
    Some((r_in, r_in + PAIR_SWITCH_WIDTH))
}

#[test]
fn the_sampled_arm_is_the_bisection_it_replaced_to_the_bit() {
    let s = water();
    let mut posed = 0;
    for &f in FLOORS.iter().chain([3.0e-7, 2.5e-9, 1.0e-12].iter()) {
        let now = s.derive_pair_cutoff(f);
        let before = derive_pair_cutoff_before(s, f);
        match (now, before) {
            (Some((a, b)), Some((c, d))) => {
                assert_eq!(a.to_bits(), c.to_bits(), "r_in at floor {f:e}");
                assert_eq!(b.to_bits(), d.to_bits(), "r_cut at floor {f:e}");
                posed += 1;
            }
            (None, None) => {}
            (x, y) => panic!("floor {f:e}: now {x:?}, before {y:?}"),
        }
    }
    assert!(posed >= 3, "the gate posed on {posed} floors only");
    assert_eq!(s.derive_pair_cutoff(0.0), None);
}

#[test]
fn the_power_arm_is_the_closed_form_it_replaced_to_the_bit() {
    for &p in [6.0, 5.5, 6.5].iter() {
        let far = FarSector::build(&[Some(power(p, 20.0))], 20.0, 1.0e-9, Dims::Two).expect("builds");
        let m = far.model(0).expect("model");
        for &b in BUDGETS.iter().chain([0.0, -1.0].iter()) {
            let before = if !(b > 0.0) || m.c_p == 0.0 { m.r_s } else { (m.c_p.abs() / b).powf(1.0 / m.p).max(m.r_s) };
            assert_eq!(m.radius_for_budget(b).to_bits(), before.to_bits(), "p={p} budget={b:e}");
            let via = reach_for_budget(Kernel::Power { c: m.c_p, p: m.p, r_min: m.r_s }, b).unwrap();
            assert_eq!(via.to_bits(), before.to_bits());
        }
    }
    // the declared arm is the registry's dialect: returned as given
    assert_eq!(reach_for_budget(Kernel::Declared { reach: 12.5 }, 1.0e-9), Some(12.5));
    // and the far reading the receipt froze is reproduced through it
    let mut out = String::new();
    far_reading(&mut out);
    assert!(out.lines().count() >= 8);
}

// --------------------------------------------------- THE EXPONENT: LAW VS FIT

#[test]
fn a_tail_that_is_the_law_agrees_and_one_that_is_not_refuses_by_name_when_asked() {
    // channel 3, pair dispersion, R^-6: a pure R^-6 fixture is the law
    let far = FarSector::build(&[Some(power(6.0, 20.0))], 20.0, 1.0e-9, Dims::Two).expect("builds");
    let r = far.exponent_readings();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].channel, ChannelId::PairDispersion);
    assert_eq!(r[0].assigned, 6.0);
    assert!(r[0].agrees, "{:?}", r[0]);
    assert!(far.require_assigned_exponent().is_ok());
    // a tail inside B2's adopting band but past the ledger's slack is a FINDING: the
    // sector builds and sums exactly as before (nothing consults the reading), and the
    // opt-in refusal names it
    let far = FarSector::build(&[Some(power(7.0, 20.0))], 20.0, 1.0e-9, Dims::Two).expect("builds: B2's band admits 7");
    let r = far.exponent_readings();
    assert!(!r[0].agrees, "{:?}", r[0]);
    match far.require_assigned_exponent() {
        Err(FarRefusal::ExponentDisagrees { slot, assigned, .. }) => {
            assert_eq!((slot, assigned), (0, 6.0));
        }
        other => panic!("expected the exponent refusal, got {other:?}"),
    }
    let text = format!("{}", far.require_assigned_exponent().unwrap_err());
    assert!(text.contains("REFUSED (channel ledger, exponent)"), "{text}");
}
