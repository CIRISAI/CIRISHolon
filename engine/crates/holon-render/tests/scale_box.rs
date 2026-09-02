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

/// A periodic 3D lattice with species synced, so a pair cutoff is derivable — the same
/// shape `t3_scale.rs` proves out (`scene()`'s `reset(8)` leaves no active slot, and a
/// zero cutoff can never break image legality, which is why these gates need their own
/// builder).
fn periodic_lattice() -> Box<Sim> {
    use holon_render::sim::Dims;
    let mut s = scene();
    s.dims = Dims::Three;
    s.boundary = Boundary::Periodic;
    let (n_side, spacing) = (4usize, 10.0);
    s.width = n_side as f64 * spacing;
    s.height = n_side as f64 * spacing;
    s.depth = n_side as f64 * spacing;
    let n = n_side * n_side * n_side;
    s.resize_storage(n);
    for i in 0..n {
        let ix = i % n_side;
        let iy = (i / n_side) % n_side;
        let iz = i / (n_side * n_side);
        s.atoms[i].x = (ix as f64 + 0.5) * spacing;
        s.atoms[i].y = (iy as f64 + 0.5) * spacing;
        s.atoms[i].z = (iz as f64 + 0.5) * spacing;
        s.atoms[i].vx = 0.0;
        s.atoms[i].vy = 0.0;
        s.atoms[i].vz = 0.0;
    }
    s.sync_species();
    s.rebase();
    s
}

#[test]
fn a_shrink_that_breaks_periodic_images_is_refused_and_the_state_is_untouched() {
    // B2's G9 found the hole: scale_box could walk a periodic scene past pbc_ok with no
    // complaint anywhere, and the symptom is a silently missing image force, not an
    // error. The door now opens only onto legal states.
    let mut s = periodic_lattice();
    assert!(
        s.set_pair_cutoff(1e-6),
        "precondition: the 40-bohr box holds the 1e-6 Ha cutoff (t3_scale.rs proves it)"
    );
    let (cut, half) = s.pbc_margin();
    assert!(
        cut.is_finite() && 0.0 < cut && cut <= half,
        "precondition: admitted legal with a REAL cutoff (cut {cut}, half-edge {half})"
    );

    // Derive the first illegal factor from the door's own numbers rather than guessing:
    // post-scale legality is cut <= half * f, so any f below cut/half must refuse.
    let f_bad = (cut / half) * 0.999;
    let (w0, h0) = (s.width, s.height);
    let x0: Vec<f64> = s.atoms.iter().map(|a| a.x).collect();
    let e0 = s.energy();

    let r = s.scale_box(f_bad);
    assert_eq!(
        r,
        Err(ScaleRefusal::BreaksPeriodicImages),
        "a shrink past the image-legality line refuses by name (f = {f_bad})"
    );
    assert_eq!(s.width, w0, "a refused move leaves the width untouched");
    assert_eq!(s.height, h0, "a refused move leaves the height untouched");
    for (a, x) in s.atoms.iter().zip(&x0) {
        assert_eq!(a.x, *x, "a refused move leaves every position untouched");
    }
    assert_eq!(s.energy().to_bits(), e0.to_bits(), "a refused move costs nothing");

    // And the same scene still accepts a legal shrink: the door refuses the move, not
    // the periodic boundary.
    let f_ok = (cut / half) * 1.001;
    assert!(f_ok < 1.0, "the probe factors are shrinks (cut/half = {})", cut / half);
    s.scale_box(f_ok)
        .expect("a shrink that keeps cut <= half*f is accepted");
    assert!(s.pbc_ok(), "and the accepted move lands on a legal state");
}

#[test]
fn the_open_boundary_is_exempt_from_the_image_check() {
    // pbc_ok is vacuously true without wrapping: there are no images to confuse, so the
    // same factor that refuses under Periodic is accepted under walls.
    let mut s = periodic_lattice();
    assert!(s.set_pair_cutoff(1e-6), "precondition shared with the gate above");
    let (cut, half) = s.pbc_margin();
    let f_bad = (cut / half) * 0.999;
    s.boundary = Boundary::Open;
    let floor = 2.0 * s.wall_inset;
    if s.width * f_bad > floor && s.height * f_bad > floor && s.depth * f_bad > floor {
        s.scale_box(f_bad)
            .expect("without a wrapping boundary the image condition does not apply");
    }
}
