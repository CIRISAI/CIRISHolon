//! THE BAROSTAT'S GATES: it is dynamics, not a rescale, and the tests are chosen so a
//! rescale would fail them.
//!
//! FSD-W1 WB-2.2 asks for NPT with its artifacts documented; WB-7.2 names the `P^-0.05` box
//! scaling in the mock shell as placeholder. The difference between the two is not that one
//! moves the volume further — both do — it is that one has a conserved quantity and an
//! ensemble and the other has a feedback constant. So the gates here are about the
//! conserved quantity, in this order:
//!
//! 1. **The pressure is the pressure.** Checked on an ideal gas, where the virial is
//!    exactly zero and `PV = N k T` is arithmetic rather than a model.
//! 2. **NPT reduces to NVE.** At infinite barostat mass with the chains idle, the extended
//!    system IS the physical one, and the trajectory must be the plain one.
//! 3. **`H'` is conserved.** The extended quantity, drifting within a bound over a real run.
//! 4. **The volume responds, with the right sign.** Compress under high `P_ext`, expand
//!    under low.
//! 5. **The ledger closes.** Every hartree the barostat moved is on its receipt column.
//! 6. **The refusals fire.** A walled box has no virial pressure and is refused.

use holon_render::barostat::{BarostatRefusal, AU_PRESSURE_PA, ONE_ATM};
use holon_render::sim::{Boundary, Dims, Sim, K_B};

fn potential_source() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/viewer/h2_potential.json");
    std::fs::read_to_string(path).expect("the placeholder curve is shipped")
}

fn loaded_sim() -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    holon_render::json::load_into(s.table_mut(), &potential_source()).expect("table loads");
    s.adopt_table_timescale();
    s
}

/// A periodic lattice at a given spacing, at rest.
fn lattice(side: usize, spacing: f64) -> Box<Sim> {
    let mut s = loaded_sim();
    s.dims = Dims::Three;
    s.boundary = Boundary::Periodic;
    let edge = side as f64 * spacing;
    s.width = edge;
    s.height = edge;
    s.depth = edge;
    let n = side * side * side;
    s.resize_storage(n);
    for i in 0..n {
        let (ix, iy, iz) = (i % side, (i / side) % side, i / (side * side));
        s.atoms[i].x = (ix as f64 + 0.5) * spacing;
        s.atoms[i].y = (iy as f64 + 0.5) * spacing;
        s.atoms[i].z = (iz as f64 + 0.5) * spacing;
    }
    s.sync_species();
    s.rebase();
    s
}

/// Give the scene a deterministic, zero-total-momentum velocity field at roughly the given
/// temperature. No RNG: a checkerboard of directions scaled to hit the target.
fn thermalize(s: &mut Sim, t_target: f64) {
    let n = s.n;
    for i in 0..n {
        let sx = if i % 2 == 0 { 1.0 } else { -1.0 };
        let sy = if (i / 2) % 2 == 0 { 1.0 } else { -1.0 };
        let sz = if (i / 4) % 2 == 0 { 1.0 } else { -1.0 };
        s.set_velocity_3d(i, sx * 1e-3, sy * 0.9e-3, sz * 1.1e-3);
    }
    // Remove any residual drift, then scale to the target temperature exactly.
    let (px, py, pz) = s.momentum();
    let m_total: f64 = (0..n).map(|i| s.atoms[i].mass()).sum();
    for i in 0..n {
        let (vx, vy, vz) = (s.atoms[i].vx, s.atoms[i].vy, s.atoms[i].vz);
        s.set_velocity_3d(
            i,
            vx - px / m_total,
            vy - py / m_total,
            vz - pz / m_total,
        );
    }
    s.recompute();
    let t_now = s.temperature();
    if t_now > 0.0 {
        let f = (t_target / t_now).sqrt();
        for i in 0..n {
            let (vx, vy, vz) = (s.atoms[i].vx, s.atoms[i].vy, s.atoms[i].vz);
            s.set_velocity_3d(i, vx * f, vy * f, vz * f);
        }
    }
    s.rebase();
}

/// GATE 1: on an ideal gas the virial is exactly zero, so `P = 2K/3V = N k T / V` is
/// arithmetic. If the pressure is wrong here it is wrong everywhere, and no amount of
/// barostat machinery on top would show it.
#[test]
fn the_pressure_is_the_ideal_gas_law_when_there_is_nothing_else() {
    // Atoms far enough apart that the curve contributes nothing measurable: at 40 bohr the
    // placeholder H-H tail is under 2e-18 hartree.
    let mut s = lattice(4, 40.0);
    thermalize(&mut s, 300.0);
    assert!(s.pressure_defined());

    let v = s.volume();
    let t = s.temperature();
    let expected = s.n as f64 * K_B * t / v;
    let got = s.pressure();
    let rel = (got - expected).abs() / expected.abs();
    println!(
        "ideal gas: N = {}, T = {t:.2} K, V = {v:.3e} a0^3, P = {got:.6e} vs NkT/V = \
         {expected:.6e}  (rel {rel:.2e}); virial = {:.3e}",
        s.n, s.w_virial
    );
    assert!(
        rel < 1e-6,
        "the virial pressure is not the ideal-gas pressure on a scene with no interactions"
    );
    // And the virial itself must be the thing that is negligible, not a cancellation.
    assert!(
        s.w_virial.abs() < 1e-12,
        "the 'ideal' gas has a virial of {:.3e}; it is not ideal and this test proves \
         nothing about the pressure",
        s.w_virial
    );
}

/// GATE 2: NPT REDUCES TO NVE. With an enormous barostat mass and the chains frozen, the
/// extended system is the physical one — so the MTK integrator must reproduce the plain
/// velocity-Verlet trajectory. A rescale hack cannot pass this: it has no mass to send to
/// infinity.
#[test]
fn npt_reduces_to_nve_at_infinite_barostat_mass() {
    let mut nve = lattice(3, 6.0);
    thermalize(&mut nve, 300.0);
    let mut npt = lattice(3, 6.0);
    thermalize(&mut npt, 300.0);

    npt.enable_barostat(ONE_ATM, 300.0).expect("periodic box");
    {
        let b = npt.barostat.as_mut().unwrap();
        // Infinite mass: the strain momentum cannot move, so `v_eps` stays zero and the box
        // is fixed. Chain masses likewise: no thermostatting.
        b.w = f64::MAX;
        b.p_eps = 0.0;
        for k in 0..b.particles.q.len() {
            b.particles.q[k] = f64::MAX;
            b.strain.q[k] = f64::MAX;
        }
    }

    let v0 = npt.volume();
    for _ in 0..20 {
        nve.step_frame(16);
        npt.step_frame(16);
    }
    assert_eq!(npt.volume(), v0, "the box moved with an infinite barostat mass");

    let mut worst: f64 = 0.0;
    for i in 0..nve.n {
        for (a, b) in [
            (nve.atoms[i].x, npt.atoms[i].x),
            (nve.atoms[i].y, npt.atoms[i].y),
            (nve.atoms[i].z, npt.atoms[i].z),
        ] {
            worst = worst.max((a - b).abs());
        }
    }
    println!("NPT-vs-NVE worst coordinate difference over 320 steps: {worst:.3e} bohr");
    // BIT-IDENTICAL, and it is exact for a reason rather than by luck. At `W = f64::MAX`
    // the strain velocity `p_eps/W` is subnormal, `exp` of a subnormal is exactly `1.0`,
    // multiplying by `1.0` is exact, and `scale_velocities` returns early on `s == 1.0`. So
    // every barostat factor is an exact identity and the MTK path computes the Verlet
    // arithmetic. Asserted exactly, so that a change which made one of those factors merely
    // NEARLY one would fire here instead of being absorbed into a tolerance.
    assert_eq!(
        worst, 0.0,
        "NPT at infinite barostat mass diverged from NVE by {worst:.3e} bohr; every barostat \
         factor should be an exact 1.0 in this limit, so one of them is not"
    );
}

/// GATE 3: the extended quantity is conserved. This is the gate that separates dynamics
/// from a feedback loop; a rescale hack has no `H'` at all.
#[test]
fn the_extended_hamiltonian_is_conserved() {
    // A GAS, deliberately, and this choice is load-bearing. The first version of this test
    // used a 7-bohr lattice; atomic hydrogen at 7 bohr and 300 K is far inside its own well
    // (D_e = 0.174 Ha against kT = 9.5e-4), so it CONDENSES — the box collapsed 650x, the
    // temperature ran to 34,000 K, and every assertion here still passed against an energy
    // scale that had become the condensation energy. That is real physics and a useless
    // test: it measured the integrator's behaviour during a collapse, not during NPT.
    // At 40 bohr the pair term is under 2e-18 Ha and the scene stays a gas.
    let mut s = lattice(4, 40.0);
    thermalize(&mut s, 300.0);
    let v_start = s.volume();
    s.enable_barostat(10.0 * ONE_ATM, 300.0)
        .expect("periodic box");

    // Let the barostat engage, then measure from there: the first few frames are the
    // extended system finding its own initial condition, which is not a drift.
    for _ in 0..20 {
        s.step_frame(16);
    }
    let h0 = s.h_prime();
    let scale = s.e_kin.abs().max(s.e_pair.abs()).max(1e-6);
    let mut worst: f64 = 0.0;
    for _ in 0..100 {
        s.step_frame(16);
        worst = worst.max((s.h_prime() - h0).abs());
    }
    println!(
        "H' drift over 1600 steps: {worst:.4e} Ha against an energy scale of {scale:.4e} \
         ({:.3}%);  V = {:.4e},  P = {:.4e},  T = {:.1} K",
        100.0 * worst / scale,
        s.volume(),
        s.pressure(),
        s.temperature()
    );
    assert!(
        worst / scale < 0.05,
        "H' drifted by {:.2}% of the energy scale, so the MTK integration is not conserving \
         the quantity it is built around",
        100.0 * worst / scale
    );
    // NON-VACUITY: the run has to have been an NPT run. A box that never moved, or a scene
    // that collapsed, would conserve H' trivially or measure something else entirely.
    let ratio = s.volume() / v_start;
    assert!(
        (0.05..20.0).contains(&ratio),
        "the box went from {v_start:.3e} to {:.3e} ({ratio:.3}x): this is a collapse or a \
         runaway, not an equilibrated NPT run",
        s.volume()
    );
    assert!(
        s.temperature() < 5.0 * 300.0,
        "the chains lost control of the temperature ({:.0} K against a 300 K target), so \
         this is not the ensemble it claims to be",
        s.temperature()
    );
}

/// GATE 4: THE VOLUME RESPONDS, AND WITH THE RIGHT SIGN. Two runs from the same
/// configuration, one at high external pressure and one at low; the box must move in
/// opposite directions.
///
/// The sign is the whole content. A barostat with an inverted virial sign compresses under
/// vacuum and every other gate here still passes.
#[test]
fn the_box_follows_the_pressure_and_in_the_right_direction() {
    // A gas again, for the reason spelled out in the H' test: a condensing scene collapses
    // under BOTH pressures and the comparison then says nothing. At 40 bohr the opening
    // internal pressure is the ideal-gas one, about 4.3 atm, so 100 atm must compress and
    // 0.1 atm must expand — opposite directions from one starting point.
    let make = |p: f64| {
        let mut s = lattice(4, 40.0);
        thermalize(&mut s, 300.0);
        let v0 = s.volume();
        let p0 = s.pressure();
        s.enable_barostat(p, 300.0).expect("periodic box");
        for _ in 0..200 {
            s.step_frame(16);
        }
        (v0, p0, s.volume(), s.pressure(), s.temperature())
    };

    let (v0, p0, v_high, p_high, t_high) = make(100.0 * ONE_ATM);
    let (_, _, v_low, p_low, t_low) = make(0.1 * ONE_ATM);
    println!(
        "V0 = {v0:.4e} at P_int = {:.3} atm;\n  100 atm -> V {:.3}x, P_int {:.3} atm, T {t_high:.0} K\n  \
         0.1 atm -> V {:.3}x, P_int {:.4} atm, T {t_low:.0} K",
        p0 / ONE_ATM,
        v_high / v0,
        p_high / ONE_ATM,
        v_low / v0,
        p_low / ONE_ATM
    );
    assert!(
        v_high < v0,
        "100 atm did not compress a box whose internal pressure was {:.2} atm: {v_high:.4e} \
         against {v0:.4e}",
        p0 / ONE_ATM
    );
    assert!(
        v_low > v0,
        "0.1 atm did not expand a box whose internal pressure was {:.2} atm: {v_low:.4e} \
         against {v0:.4e}",
        p0 / ONE_ATM
    );
    // NON-VACUITY, and the assertion the sign error would survive: it is not enough that
    // the two runs differ, they must BRACKET the starting volume. A barostat with an
    // inverted virial sign moves both the same way and would pass a bare `v_low > v_high`.
    assert!(
        v_high < v0 && v0 < v_low,
        "the two runs did not bracket the starting volume ({v_high:.4e}, {v0:.4e}, \
         {v_low:.4e}), so they are not responding to the pressure difference's SIGN"
    );
    // And the compressed run must have moved TOWARD its target rather than past it.
    assert!(
        p_high > p0,
        "compressing the box did not raise its internal pressure ({:.3} atm from {:.3} atm)",
        p_high / ONE_ATM,
        p0 / ONE_ATM
    );
}

/// GATE 5: the barostat is on the ledger. Everything it moved is on its own receipt column,
/// so the energy balance still closes and the attribution still says who moved it.
#[test]
fn the_barostat_is_on_the_ledger() {
    let mut s = lattice(3, 7.0);
    thermalize(&mut s, 300.0);
    assert_eq!(s.work.barostat, 0.0);
    s.enable_barostat(500.0 * ONE_ATM, 300.0)
        .expect("periodic box");
    for _ in 0..60 {
        s.step_frame(16);
    }
    assert!(
        s.work.barostat != 0.0,
        "a barostat that compressed the box moved no energy, which cannot be true"
    );
    assert_eq!(s.work.hand, 0.0, "the barostat posted to the hand's column");
    assert_eq!(
        s.work.thermostat, 0.0,
        "the barostat posted to the Berendsen thermostat's column; NPT owns the temperature \
         through its own chains and that column must stay empty"
    );
    assert!(
        s.work_columns_ok(),
        "columns {:?} sum to {:.17e} but w_ext is {:.17e}",
        s.work,
        s.work.total(),
        s.w_ext
    );
    // And the ordinary drift — E minus the receipts — is still what it always was.
    println!(
        "barostat receipt {:.4e} Ha;  drift {:.4e} against bound {:.4e}",
        s.work.barostat,
        s.drift(),
        s.drift_bound()
    );
}

/// GATE 6: the refusals. A walled box has no virial pressure, and a barostat that ran
/// anyway would be controlling a number that is not the quantity named.
#[test]
fn a_barostat_refuses_what_it_cannot_control() {
    let mut walled = lattice(3, 7.0);
    walled.boundary = Boundary::Walls;
    assert_eq!(
        walled.enable_barostat(ONE_ATM, 300.0),
        Err(BarostatRefusal::NotPeriodic)
    );
    assert!(!walled.barostat_on());
    assert!(!walled.pressure_defined());

    let mut tiny = loaded_sim();
    tiny.dims = Dims::Three;
    tiny.boundary = Boundary::Periodic;
    tiny.resize_storage(1);
    assert_eq!(
        tiny.enable_barostat(ONE_ATM, 300.0),
        Err(BarostatRefusal::TooFewAtoms)
    );

    // And the control: the same call on a periodic box with atoms is accepted, so the
    // refusals above are about the conditions and not about the call always failing.
    let mut ok = lattice(3, 7.0);
    assert_eq!(ok.enable_barostat(ONE_ATM, 300.0), Ok(()));
    assert!(ok.barostat_on());
}

/// One atmosphere is one atmosphere. A unit constant nobody checks is a unit constant that
/// is wrong by 10^n in somebody's report — and this one WAS wrong, by 3.0e-5, until this
/// test was written and fired.
#[test]
fn the_pressure_unit_is_what_it_says() {
    // The atomic unit of pressure is E_h/a0^3 = 2.9421015697e13 Pa (CODATA 2018), and a
    // standard atmosphere is 101325 Pa by definition. Both halves are asserted, because a
    // test that only checked the quotient would pass on two errors that cancel.
    assert!(
        (AU_PRESSURE_PA - 2.942_101_569_7e13).abs() < 1.0,
        "the atomic unit of pressure is not the CODATA value"
    );
    let expected = 101_325.0 / 2.942_101_569_7e13;
    assert!(
        (ONE_ATM - expected).abs() / expected < 1e-12,
        "ONE_ATM is {ONE_ATM:.9e}, which is not 101325 Pa in hartree per cubic bohr \
         ({expected:.9e})"
    );
    // And the sanity check a reader can do in their head: an atmosphere is a few
    // nano-hartree per cubic bohr, not a few micro- or a few pico-.
    assert!((3.0e-9..4.0e-9).contains(&ONE_ATM));
}
