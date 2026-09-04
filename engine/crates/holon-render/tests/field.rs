//! FIELD-1's gates (`conformance/water_observatory/FIELD_PREREG.md` §1, §3): identity in
//! checkpoint bytes with the field off, the ledger and momentum with it on, the force as
//! the derivative, the charge as the record's, the wrapped box refused, and the three plants.

use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::embed::{fragment_charges, monomer, water_centers, ChargeModel, Fragment};
use holon_render::field::{water_charge_at_pin, FieldPlant, FieldRefusal, WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD};
use holon_render::sim::{Boundary, Dims, Sim};

#[path = "common/quartet.rs"]
mod quartet;

const SUBSTEPS: u32 = 16;
const STEPS: usize = 2000;

/// Four water molecules at EMBED-1's pin, oxygens on a square, in a walled 3D box, with a
/// seeded momentum-free velocity field (no library RNG: an LCG, so the scene re-runs byte
/// for byte).
fn four_waters(boundary: Boundary) -> Box<Sim> {
    let mono = water_centers(WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD);
    let oxygens = [[5.0, 5.0, 5.0], [12.0, 5.0, 5.0], [5.0, 12.0, 5.0], [12.0, 12.0, 5.0]];
    let mut species = Vec::new();
    let mut pos = Vec::new();
    for (k, o) in oxygens.iter().enumerate() {
        // alternate the molecular plane so the scene is not a symmetric artifact
        let flip = if k % 2 == 0 { 1.0 } else { -1.0 };
        for (m, c) in mono.iter().enumerate() {
            species.push(if m == 0 { OXYGEN } else { HYDROGEN });
            pos.push([o[0] + flip * c[0], o[1] + c[2], o[2] + c[1]]);
        }
    }
    let mut s = quartet::scene(&species, &pos, false);
    s.dims = Dims::Three;
    s.boundary = boundary;
    s.width = 17.0;
    s.height = 17.0;
    s.depth = 10.0;
    let mut st: u64 = 0x4649_454c;
    let mut lcg = || {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((st >> 11) as f64) / ((1u64 << 53) as f64)
    };
    let n = s.n;
    let (mut px, mut py, mut pz) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let scale = if s.atoms[i].species.z == 8 { 2.5e-4 } else { 1.0e-3 };
        s.atoms[i].vx = scale * (2.0 * lcg() - 1.0);
        s.atoms[i].vy = scale * (2.0 * lcg() - 1.0);
        s.atoms[i].vz = scale * (2.0 * lcg() - 1.0);
        let m = s.atoms[i].mass();
        px += m * s.atoms[i].vx;
        py += m * s.atoms[i].vy;
        pz += m * s.atoms[i].vz;
    }
    let mtot: f64 = (0..n).map(|i| s.atoms[i].mass()).sum();
    for i in 0..n {
        s.atoms[i].vx -= px / mtot;
        s.atoms[i].vy -= py / mtot;
        s.atoms[i].vz -= pz / mtot;
    }
    s.sync_species();
    // the timestep from the stiffest loaded pair (O–H): without it the integrator's own
    // error envelope — the drift bound — is tens of hartree and every ledger gate is vacuous
    s.adopt_table_timescale();
    s.rebase();
    s
}

fn run(s: &mut Sim, steps: usize) {
    for _ in 0..steps / SUBSTEPS as usize {
        s.step_frame(SUBSTEPS);
    }
}

fn water_rows(s: &Sim) -> usize {
    let z: Vec<u32> = (0..s.n).map(|i| s.atoms[i].species.z).collect();
    holon_render::field::water_units(&s.charge_row, &z).len()
}

#[test]
fn g0_the_field_off_is_the_identity_in_checkpoint_bytes_and_pure_hydrogen_carries_zero() {
    let mut a = four_waters(Boundary::Walls);
    let mut b = four_waters(Boundary::Walls);
    b.set_field(true, Some(0.2)).expect("walls admit the field");
    b.set_field(false, None).expect("off");
    run(&mut a, STEPS);
    run(&mut b, STEPS);
    assert_eq!(a.checkpoint().bytes, b.checkpoint().bytes, "G0: enabling then disabling before the first step must be the identity");
    assert_eq!(a.e_field, 0.0);
    // pure hydrogen: no water row, no charge, an EXACT zero at every step
    let mut h = quartet::scene(&[HYDROGEN; 4], &[[0.0, 0.0, 0.0], [1.4, 0.0, 0.0], [5.0, 0.0, 0.0], [6.4, 0.0, 0.0]], false);
    h.width = 10.0;
    h.height = 6.0;
    h.rebase();
    h.set_field(true, Some(0.2)).unwrap();
    for _ in 0..50 {
        h.step_frame(SUBSTEPS);
        assert_eq!(h.e_field, 0.0, "G0: hydrogen carries no charge");
    }
    assert_eq!(h.field_work.transitions, 0);
}

/// FIELD_AMENDMENT_2: under walls the drift bound is 20 hartree (the wall stiffness sets the
/// integrator's envelope) and the gate cannot fire on its plant — VOID as a gate, kept in the
/// record; G1′ is the same gate on the open box.
#[test]
fn g1_under_walls_is_vacuous_and_says_so() {
    let mut s = four_waters(Boundary::Walls);
    run(&mut s, 64);
    s.set_field(true, None).unwrap();
    run(&mut s, 256);
    eprintln!("G1 under walls (VOID as a gate): drift {:.3e}, bound {:.3e}", s.drift(), s.drift_bound());
    assert!(s.drift_bound() > 1.0, "the walled bound is the reason this gate moved; it reads {}", s.drift_bound());
}

#[test]
fn g1_the_ledger_closes_with_the_field_on_and_plant_ii_opens_it() {
    let mut s = four_waters(Boundary::Open);
    run(&mut s, 64);
    s.set_field(true, None).expect("walls admit the field");
    s.compute_forces();
    assert!(water_rows(&s) >= 4, "the bond verdict must yield four water units, gave {}", water_rows(&s));
    let mut worst = 0.0f64;
    for _ in 0..STEPS / SUBSTEPS as usize {
        s.step_frame(SUBSTEPS);
        worst = worst.max(s.drift() / s.drift_bound());
        assert!(s.drift() <= s.drift_bound(), "G1: drift {} over its bound {}", s.drift(), s.drift_bound());
        assert!(s.work_columns_ok(), "G1: the receipt columns do not sum to w_ext");
    }
    assert!(s.field_work.transitions >= 1, "G1: enabling the field is a transition and must have been posted");
    assert!(s.e_field != 0.0 && s.work.field != 0.0, "G1: the field carried energy ({}) and posted its transition ({})", s.e_field, s.work.field);
    eprintln!("G1: e_field {:.6e} Ha, work.field {:.6e}, transitions {}, worst drift/bound {:.3}", s.e_field, s.work.field, s.field_work.transitions, worst);
    // plant (ii), FIELD_AMENDMENT_3: the transition applied without posting, read as a
    // two-arm discrimination on the ledger's shift — the envelope bound (20 hartree on
    // this scene) cannot see an 8e-4 transition and is not the instrument here
    let de = s.work.field.abs();
    assert!(de >= 1e-4, "plant (ii) carrier: the transition is {de:.2e} hartree, not nonzero in its sector");
    assert!(s.drift_peak <= 0.1 * de, "the honest arm's drift peak {:.2e} is not under a tenth of the transition {de:.2e}", s.drift_peak);
    let mut p = four_waters(Boundary::Open);
    run(&mut p, 64);
    p.field_plant = FieldPlant::SkipLedger;
    p.set_field(true, None).unwrap();
    run(&mut p, STEPS);
    eprintln!("plant (ii): transition {de:.3e} Ha; honest drift_peak {:.3e}, plant drift_peak {:.3e} (envelope bound {:.2e}, uninformative)", s.drift_peak, p.drift_peak, p.drift_bound());
    assert!(p.field_work.transitions >= 1, "plant (ii) carrier: a transition occurred");
    assert!(p.drift_peak >= 0.5 * de, "plant (ii) did not fire: the ledger shifted by only {:.2e} against a {de:.2e} transition", p.drift_peak);
}

#[test]
fn g2_momentum_is_conserved_with_the_field_on_and_plant_i_breaks_it() {
    let mut s = four_waters(Boundary::Open);
    run(&mut s, 64);
    s.set_field(true, None).unwrap();
    run(&mut s, STEPS);
    let (mut fx, mut fy, mut fz, mut scale) = (0.0, 0.0, 0.0, 0.0f64);
    for i in 0..s.n {
        let (x, y, z) = s.internal_force(i);
        fx += x;
        fy += y;
        fz += z;
        scale = scale.max((x * x + y * y + z * z).sqrt());
    }
    let net = (fx * fx + fy * fy + fz * fz).sqrt();
    assert!(scale > 1e-6, "the scene produced no internal force");
    assert!(net <= 1e-12 * scale.max(1.0) * (s.n as f64), "G2: internal forces sum to {net:.3e} against a scale of {scale:.3e}");
    assert!(s.momentum_residual() <= s.momentum_bound(), "G2: momentum residual {} over bound {}", s.momentum_residual(), s.momentum_bound());
    eprintln!("G2: net internal force {net:.2e} (scale {scale:.2e}), momentum residual {:.2e} / bound {:.2e}", s.momentum_residual(), s.momentum_bound());
    // plant (i): the reaction dropped
    let mut p = four_waters(Boundary::Open);
    run(&mut p, 64);
    p.field_plant = FieldPlant::DropReaction;
    p.set_field(true, None).unwrap();
    run(&mut p, STEPS);
    let (mut gx, mut gy, mut gz) = (0.0, 0.0, 0.0);
    for i in 0..p.n {
        let (x, y, z) = p.internal_force(i);
        gx += x;
        gy += y;
        gz += z;
    }
    let net_p = (gx * gx + gy * gy + gz * gz).sqrt();
    assert!(net_p >= 1e-6, "plant (i) carrier: the field force on some atom is at least 1e-6 (net {net_p:.2e})");
    assert!(p.momentum_residual() > p.momentum_bound(), "plant (i) did not fire");
}

#[test]
fn g3_the_force_is_the_derivative_and_plant_iii_flips_it() {
    for (plant, expect_fire) in [(FieldPlant::None, false), (FieldPlant::FlipSign, true)] {
        let mut s = four_waters(Boundary::Walls);
        run(&mut s, 64);
        s.field_plant = plant;
        s.set_field(true, None).unwrap();
        s.compute_forces();
        let charge = s.charge.clone();
        let honest_e = |s: &mut Sim| -> f64 {
            // the honest energy at these positions, sign plant removed
            let saved = s.field_plant;
            s.field_plant = FieldPlant::None;
            let e = s.field_energy_of(&charge);
            s.field_plant = saved;
            e
        };
        let mut worst = 0.0f64;
        let h = 1e-5;
        let mut carrier = 0.0f64;
        for i in 0..s.n {
            if s.charge[i] == 0.0 {
                continue;
            }
            // the analytic field force alone: recompute forces with everything else unchanged
            // by differencing the internal force with the field on and off at fixed positions
            s.compute_forces();
            let with = s.internal_force(i);
            let saved_field = s.field;
            s.field = None;
            s.compute_forces();
            let without = s.internal_force(i);
            s.field = saved_field;
            let f = [with.0 - without.0, with.1 - without.1, with.2 - without.2];
            let fmag = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
            carrier = carrier.max(fmag);
            for (k, coord) in [0usize, 1, 2].iter().enumerate() {
                let x0 = match coord { 0 => s.atoms[i].x, 1 => s.atoms[i].y, _ => s.atoms[i].z };
                let set = |s: &mut Sim, v: f64| match coord { 0 => s.atoms[i].x = v, 1 => s.atoms[i].y = v, _ => s.atoms[i].z = v };
                set(&mut s, x0 + h);
                let ep = honest_e(&mut s);
                set(&mut s, x0 - h);
                let em = honest_e(&mut s);
                set(&mut s, x0);
                let fd = -(ep - em) / (2.0 * h);
                // relative to the atom's field FORCE, not to one component of it: a
                // component near zero has no relative error worth reading
                let rel = (f[k] - fd).abs() / fmag.max(1e-12);
                worst = worst.max(rel);
            }
        }
        assert!(carrier >= 1e-6, "plant (iii) carrier: |F_field| = {carrier:.2e}");
        if expect_fire {
            assert!(worst >= 1.0, "plant (iii) did not fire: worst relative {worst:.2e}");
        } else {
            assert!(worst <= 1e-8, "G3: worst relative |F − (−∂E)| = {worst:.2e}");
            eprintln!("G3: worst relative {worst:.2e}; |F_field| max {carrier:.3e}");
        }
    }
}

#[test]
fn g4_the_charge_is_the_records() {
    let q = water_charge_at_pin();
    let (o, h) = (OXYGEN, HYDROGEN);
    let frag = Fragment::new(vec![o, h, h], water_centers(WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD).to_vec(), vec![-2.0, 1.0, 1.0]);
    let m = monomer(&frag, &[], ChargeModel::DipoleExact);
    let q2 = fragment_charges(ChargeModel::DipoleExact, &frag, &m.p, &m.solve.basis, &m.mom)[1];
    assert!((q - q2).abs() <= 1e-12, "G4: {q} vs {q2}");
    assert!(q > 0.1 && q < 0.5, "G4: a water hydrogen charge of {q} is not the record's kind of number");
    eprintln!("G4: q_H = {q:.9} (q_O = {:.9})", -2.0 * q);
}

#[test]
fn g5_the_wrapped_box_is_refused_by_name() {
    let mut s = four_waters(Boundary::Periodic);
    assert_eq!(s.set_field(true, Some(0.2)), Err(FieldRefusal::PeriodicNeedsEwald));
    assert!(s.field.is_none(), "the refusal left the state unchanged");
    assert!(FieldRefusal::PeriodicNeedsEwald.to_string().contains("Ewald"));
}
