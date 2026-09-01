//! THE HAND ON THE BOX (WB-2.2): the page changes the SIZE of the box to change the
//! pressure, and every joule that move costs is posted to the ledger's hand column.
//!
//! One gate per law, as always: the geometry assertions say the move happened, the
//! LEDGER assertion says it was paid for (energy() minus w_ext is what the drift gate
//! reads, so an unledgered scale would open it by exactly the move's cost), and the
//! refusals are planted with the wrong inputs to prove the door can say no.

use holon_render::barostat::ScaleRefusal;
use holon_render::sim::{Boundary, Sim};

fn scene() -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    holon_render::json::load_into(
        s.table_mut(),
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/viewer/h2_potential.json"
        ))
        .expect("placeholder curve readable"),
    )
    .expect("table loads");
    s.adopt_table_timescale();
    s.reset(8);
    s
}

#[test]
fn the_scale_is_applied_and_paid_for() {
    let mut s = scene();
    let (w0, h0) = (s.width, s.height);
    let x0: Vec<f64> = s.atoms.iter().map(|a| a.x).collect();
    let e0 = s.energy();
    let hand0 = s.work.hand;

    s.scale_box(0.9).expect("a modest compression is accepted");

    assert!((s.width - 0.9 * w0).abs() < 1e-12 * w0, "width scales");
    assert!((s.height - 0.9 * h0).abs() < 1e-12 * h0, "height scales");
    for (a, x) in s.atoms.iter().zip(&x0) {
        assert!((a.x - 0.9 * x).abs() < 1e-9, "positions scale affinely");
    }
    // THE LEDGER LINE: the energy change equals the hand work posted, to roundoff.
    let de = s.energy() - e0;
    let dhand = s.work.hand - hand0;
    assert!(
        (de - dhand).abs() <= 1e-9 * de.abs().max(1.0),
        "the move's cost is ledgered: dE = {de:e}, hand = {dhand:e}"
    );
}

#[test]
fn the_energy_gate_survives_a_scale_mid_run() {
    let mut s = scene();
    for _ in 0..20 {
        s.step();
    }
    s.scale_box(0.95).expect("compression mid-run is accepted");
    for _ in 0..20 {
        s.step();
    }
    assert!(
        s.drift() <= s.drift_bound(),
        "an unledgered scale would open the energy gate by the move's cost: drift {} vs bound {}",
        s.drift(),
        s.drift_bound()
    );
}

#[test]
fn the_refusals_fire_by_name() {
    let mut s = scene();
    assert_eq!(s.scale_box(0.0), Err(ScaleRefusal::BadFactor));
    assert_eq!(s.scale_box(-1.0), Err(ScaleRefusal::BadFactor));
    assert_eq!(s.scale_box(f64::NAN), Err(ScaleRefusal::BadFactor));
    assert_eq!(s.scale_box(1e-9), Err(ScaleRefusal::CollapsesBox));
    // and the box is untouched by a refused move
    let w = s.width;
    let _ = s.scale_box(f64::INFINITY);
    assert_eq!(s.width, w, "a refused move changes nothing");
}

#[test]
fn pressure_reads_and_declares_its_domain() {
    let mut s = scene();
    assert!(!s.pressure_defined(), "walls: the virial is not the pressure");
    s.boundary = Boundary::Periodic;
    assert!(s.pressure_defined(), "periodic: it is");
    let _p = s.pressure();
}
