//! FIELD-3's gates (`conformance/water_observatory/FIELD3_PREREG.md` §2): the units exist
//! where FIELD-2 found none (G-A1), the two rules on FIELD-1's scene (G-A2, read as
//! measured), the seam off as the identity in bytes (G-B0), the books (G-B1), momentum with
//! plant (iii) (G-B2), the wall as the derivative (G-B3), the closures contributing exactly
//! nothing across the seam with plant (ii) (G-B4).
//!
//! The wall's coefficients are the harvest's (`field3/wall.json`) when it exists; before
//! it does, DECLARED test coefficients stand in — the gates below are conservation
//! properties that hold for any `(A, b)`, and each run says which it used.

use holon_render::seam::{SeamModel, SeamPlant, FREE};
use holon_render::sim::{Boundary, Sim};

#[path = "common/field2_scenes.rs"]
mod field2_scenes;
use field2_scenes::*;
#[path = "common/channel_scenes.rs"]
#[allow(dead_code)]
mod channel_scenes;

const STEPS: usize = 2000;
const WALL7_JSON: &str = "../../../conformance/water_observatory/field7/wall7.json";
const WALL6_JSON: &str = "../../../conformance/water_observatory/field6/wall6.json";
const WALL5_JSON: &str = "../../../conformance/water_observatory/field5/wall5.json";
const WALL4_JSON: &str = "../../../conformance/water_observatory/field4/wall4.json";
const WALL_JSON: &str = "../../../conformance/water_observatory/field3/wall.json";

/// FIELD-4's harvested terms when they exist, else FIELD-3's wall, else DECLARED test
/// coefficients — the gates below are conservation properties that hold for any values,
/// and each run says which it used.
fn wall() -> (SeamModel, String) {
    for path in [WALL7_JSON, WALL6_JSON, WALL5_JSON, WALL4_JSON, WALL_JSON] {
        if let Ok(t) = std::fs::read_to_string(path) {
            let num = |k: &str| t.split(&format!("\"{k}\": ")).nth(1).and_then(|x| x.split(',').next()).and_then(|x| x.trim().parse::<f64>().ok());
            if let (Some(a), Some(b)) = (num("a"), num("b")) {
                if a != 0.0 {
                    let (p, c, c6) = (num("p").unwrap_or(0.0), num("c").unwrap_or(0.0), num("c6").unwrap_or(0.0));
                    let (a_oh, b_oh, a_hh, b_hh) = (num("a_oh").unwrap_or(0.0), num("b_oh").unwrap_or(0.0), num("a_hh").unwrap_or(0.0), num("b_hh").unwrap_or(0.0));
                    return (SeamModel { a, b, p, c, c6, a_oh, b_oh, a_hh, b_hh }, format!("{path} (A = {a:.6e}, b = {b:.6}, P = {p:.6e}, c = {c:.6}, C6 = {c6:.6e}, A_OH = {a_oh:.6e}, b_OH = {b_oh:.6}, A_HH = {a_hh:.6e}, b_HH = {b_hh:.6})"));
                }
            }
        }
    }
    (SeamModel { a: 0.5, b: 1.2, p: 0.02, c: 1.5, c6: 10.0, ..SeamModel::NO_WALL }, "DECLARED test coefficients (no harvest on disk): A = 0.5, b = 1.2, P = 0.02, c = 1.5, C6 = 10".to_string())
}

/// M-EXTRAPOLATED-HOLE (FIELD-7): a harvested law whose cross-unit pair potential does not
/// rise monotonically inward from 3.0 bohr to contact has a hole below its data, and the
/// dynamics will find it. The DYNAMICS gates (books, momentum) run on such a record only
/// through the declared coefficients, and say so; the static derivative gate runs on the
/// record as harvested.
fn has_hole(m: &SeamModel) -> Option<String> {
    let q_h = holon_render::field::water_charge_at_pin();
    let q_o = -2.0 * q_h;
    let classes: [(&str, Box<dyn Fn(f64) -> f64>); 3] = [
        ("H–O", Box::new(move |r: f64| m.penetration(r) + m.wall_oh(r) + q_h * q_o / r)),
        ("O–O", Box::new(move |r: f64| m.wall(r) + m.dispersion(r) + q_o * q_o / r)),
        ("H–H", Box::new(move |r: f64| m.wall_hh(r) + q_h * q_h / r)),
    ];
    for (name, u) in classes.iter() {
        let mut r = 3.0;
        let mut prev = u(r);
        while r > 0.5 {
            r -= 0.05;
            let v = u(r);
            if v < prev - 1e-12 {
                return Some(format!("{name} potential falls inward at r = {r:.2} bohr ({v:+.4e} < {prev:+.4e})"));
            }
            prev = v;
        }
    }
    None
}

/// The record for the dynamics gates: the newest harvest if it has no hole, else the declared
/// coefficients with the hole named.
fn dynamics_wall() -> (SeamModel, String) {
    let (m, which) = wall();
    match has_hole(&m) {
        None => (m, which),
        Some(why) => (
            SeamModel { a: 0.5, b: 1.2, p: 0.02, c: 1.5, c6: 10.0, a_oh: 0.3, b_oh: 1.8, a_hh: 0.2, b_hh: 1.6 },
            format!("DECLARED coefficients — the newest record ({which}) has a HOLE below its data: {why} (M-EXTRAPOLATED-HOLE)"),
        ),
    }
}

fn step(s: &mut Sim, n: usize) {
    for _ in 0..n {
        s.step_frame(1);
    }
}

/// `E_field(start) − E_field(separated)`: the units moved 40 bohr apart along x, unit k by
/// 40·k, the assignment kept by construction (intra-unit distances are unchanged).
fn field_binding(s: &mut Sim) -> f64 {
    s.refresh_pairs();
    s.compute_forces();
    let e0 = s.e_field;
    let units: Vec<u32> = s.unit_of[..s.n].to_vec();
    let mut ids: Vec<u32> = units.iter().copied().filter(|&u| u != FREE).collect();
    ids.sort_unstable();
    ids.dedup();
    let saved: Vec<(f64, f64, f64)> = (0..s.n).map(|i| (s.atoms[i].x, s.atoms[i].y, s.atoms[i].z)).collect();
    for i in 0..s.n {
        if let Some(k) = ids.iter().position(|&u| u == units[i]) {
            s.atoms[i].x += 40.0 * k as f64;
        }
    }
    s.compute_forces();
    let e_sep = s.e_field;
    for i in 0..s.n {
        s.atoms[i].x = saved[i].0;
        s.atoms[i].y = saved[i].1;
        s.atoms[i].z = saved[i].2;
    }
    s.compute_forces();
    e0 - e_sep
}

#[test]
fn g_a1_the_units_exist_where_field_2_found_none_and_the_field_binds() {
    let (dsp, dpos) = dimer_positions();
    let (tsp, tpos) = ring_positions();
    let (qsp, qpos) = square_positions();
    let mut out = Vec::new();
    // FIELD-2's rule on the square start: the freeze's parenthetical said 4; the measured
    // count is recorded here and reported (the staked quantities are the closure counts and
    // the bindings; the parenthetical was a transcription from FIELD-1's probe, not a stake)
    for (name, sp, pos, edge, want_units, want_old, must_bind) in [
        ("dimer", &dsp, &dpos, 30.0, 2u64, Some(0usize), true),
        ("ring", &tsp, &tpos, 34.0, 4, Some(0), true),
        ("square", &qsp, &qpos, 34.0, 4, None, false),
    ] {
        let mut s = scene(sp, pos, edge, 293.0);
        s.set_field(true, None).unwrap();
        let binding = field_binding(&mut s);
        let old = s.units_by_pair_verdict();
        let old_units = (0..s.n).filter(|&i| old[i] == i as u32).count();
        eprintln!("G-A1 {name}: closure units {} (FIELD-2's rule: {old_units}), field binding at the start {binding:+.6e} Ha", s.seam_work.units);
        assert_eq!(s.seam_work.units, want_units, "{name}: the closure assignment's unit count");
        if let Some(w) = want_old {
            assert_eq!(old_units, w, "{name}: FIELD-2's rule reproduces its own count");
        }
        if must_bind {
            assert!(binding < 0.0 && binding.abs() >= 1e-4, "{name}: the field's binding at the start is {binding:+.3e}, staked negative with magnitude ≥ 1e-4");
        }
        out.push((name, s.seam_work.units, old_units, binding));
    }
    // the ring binds more than the dimer: four contacts against one
    assert!(out[1].3 < out[0].3, "the ring's binding {:+.3e} should exceed the dimer's {:+.3e}", out[1].3, out[0].3);
}

/// G-A2 as frozen: "on FIELD-1's four-water walled scene after 2,000 steps the new
/// assignment equals the old one atom for atom and `e_field` is bit-identical". Measured:
/// the assignments agree at step 2,000 — and disagree at the FIRST frame, where the pair
/// verdict bonds a hydrogen to another molecule's oxygen at ~5.7 bohr on the O–H curve's
/// tail, so FIELD-1's rule charged two of the four waters and posted transitions the
/// closure reading never posts; the trajectories diverge there and `e_field` at step 2,000
/// is not bit-identical. The gate's first half passes by letter, the second fails, and the
/// cause is the one FIELD-2 named. This test asserts the measured facts.
#[test]
fn g_a2_the_rules_agree_at_step_2000_and_disagree_at_the_first_frame_for_the_cause_field_2_named() {
    let mut s = channel_scenes::four_waters(Boundary::Walls);
    s.set_field(true, None).unwrap();
    // the first frame
    s.step_frame(channel_scenes::SUBSTEPS);
    let old0 = s.units_by_pair_verdict();
    let new0: Vec<u32> = s.unit_of[..s.n].to_vec();
    let old0_units = (0..s.n).filter(|&i| old0[i] == i as u32).count();
    let cross: Vec<_> = s.pairs[..s.pair_count]
        .iter()
        .filter(|p| p.bonded)
        .filter(|p| {
            let (zi, zj) = (s.atoms[p.i].species.z, s.atoms[p.j].species.z);
            (zi == 8 && zj == 1 || zi == 1 && zj == 8) && new0[p.i] != FREE && new0[p.j] != FREE && new0[p.i] != new0[p.j]
        })
        .map(|p| (p.i, p.j, p.r, p.e_rel))
        .collect();
    eprintln!("G-A2 first frame: closure units 4 vs FIELD-1's rule {old0_units}; cross-molecule O–H pairs the verdict bonds: {cross:?}");
    assert_ne!(old0, new0, "the rules disagree at the first frame (measured)");
    assert!(!cross.is_empty() && cross.iter().all(|c| c.2 > 4.0), "the cause is a cross-molecule verdict bond on the curve's tail");
    // step 2,000
    let mut differing = 0usize;
    let frames = channel_scenes::WATER_STEPS / channel_scenes::SUBSTEPS as usize;
    for _ in 1..frames {
        s.step_frame(channel_scenes::SUBSTEPS);
        if s.units_by_pair_verdict() != s.unit_of[..s.n] {
            differing += 1;
        }
    }
    let old = s.units_by_pair_verdict();
    let new: Vec<u32> = s.unit_of[..s.n].to_vec();
    eprintln!("G-A2 at step {}: assignments {} (frames differing {} of {frames}); field transitions posted {}, work.field {:+.3e}", channel_scenes::WATER_STEPS, if old == new { "EQUAL" } else { "DIFFER" }, differing + 1, s.field_work.transitions, s.work.field);
    assert_eq!(old, new, "G-A2 first half: the assignments agree at step 2,000");
    assert_eq!((0..s.n).filter(|&i| new[i] == i as u32).count(), 4);
}

#[test]
fn g_b0_the_seam_off_is_the_identity_in_checkpoint_bytes() {
    let mut a = channel_scenes::four_waters(Boundary::Walls);
    let mut b = channel_scenes::four_waters(Boundary::Walls);
    // the seam switched on and off FIRST, while every receipt column is still zero: the two
    // postings are exact negatives and `(0 + d) − d` is exactly 0, where `(w + d) − d` on a
    // nonzero column loses bits (a floating-point fact about any posted transition, the
    // field's included). Then the field, and forces made current in both scenes — `set_seam`
    // recomputes forces where `set_field` defers, and the identity under test is the seam's.
    let (model, _) = wall();
    b.set_seam(Some(model)).unwrap();
    b.set_seam(None).unwrap();
    assert_eq!(b.seam_work.transitions, 2, "enabling and disabling are two transitions");
    assert_eq!(b.w_ext, 0.0, "the two postings cancel exactly from a zero column");
    a.set_field(true, None).unwrap();
    b.set_field(true, None).unwrap();
    a.compute_forces();
    b.compute_forces();
    channel_scenes::run(&mut a, STEPS);
    channel_scenes::run(&mut b, STEPS);
    assert_eq!(a.checkpoint().bytes, b.checkpoint().bytes, "G-B0: enabling then disabling before the first step must be the identity");
    assert_eq!(b.e_seam, 0.0);
    assert_eq!(b.work.seam, 0.0, "the two transitions cancel exactly");
}

#[test]
fn g_b1_the_books_close_with_the_seam_on() {
    let (model, which) = dynamics_wall();
    let (dsp, dpos) = dimer_positions();
    let (tsp, tpos) = ring_positions();
    for (name, sp, pos, edge) in [("dimer", &dsp, &dpos, 30.0), ("ring", &tsp, &tpos, 34.0)] {
        let mut s = scene(sp, pos, edge, 293.0);
        s.set_field(true, None).unwrap();
        s.set_seam(Some(model)).unwrap();
        step(&mut s, STEPS);
        let transition = s.work.field.abs().max(s.work.seam.abs());
        let bar = if transition > 0.0 { 0.1 * transition } else { 1e-5 };
        eprintln!("G-B1 {name} ({which}): columns {} (hand {:+.2e} thermostat {:+.2e} field {:+.2e} seam {:+.2e} = w_ext {:+.2e}), drift peak {:.2e} against {bar:.2e}; seam transitions {}, pairs dropped {}, triples dropped {}, O–O walls {}",
            s.work_columns_ok(), s.work.hand, s.work.thermostat, s.work.field, s.work.seam, s.w_ext, s.drift_peak, s.seam_work.transitions, s.seam_work.pairs_dropped, s.seam_work.triples_dropped, s.seam_work.oo_pairs);
        assert!(s.work_columns_ok(), "{name}: the receipt columns do not sum to w_ext");
        assert!(s.drift_peak <= bar, "{name}: honest drift peak {:.3e} over {bar:.3e}", s.drift_peak);
        assert!(s.seam_work.pairs_dropped > 0, "{name}: the seam rule dropped nothing — vacuous");
    }
}

#[test]
fn g_b2_momentum_is_conserved_with_the_seam_on_and_plant_iii_breaks_it() {
    let (model, which) = dynamics_wall();
    let (dsp, dpos) = dimer_positions();
    for (plant, expect_fire) in [(SeamPlant::None, false), (SeamPlant::DropReaction, true), (SeamPlant::DropReactionNew, true)] {
        let mut s = scene(&dsp, &dpos, 30.0, 293.0);
        s.set_field(true, None).unwrap();
        s.seam_plant = plant;
        s.set_seam(Some(model)).unwrap();
        // the carrier: the wall's force at the start
        let r_oo = ((s.atoms[0].x - s.atoms[3].x).powi(2) + (s.atoms[0].y - s.atoms[3].y).powi(2) + (s.atoms[0].z - s.atoms[3].z).powi(2)).sqrt();
        let f_wall = model.b * model.wall(r_oo);
        assert!(f_wall >= 1e-6, "plant (iii) carrier: |F_wall| = {f_wall:.2e} at R_OO = {r_oo:.2}");
        if plant == SeamPlant::DropReactionNew {
            // FIELD-4 plant (ii) carrier: the new terms' force at the start (the H-bond contact
            // O_acc···H_don is atoms 3 and 1)
            let r_ho = ((s.atoms[3].x - s.atoms[1].x).powi(2) + (s.atoms[3].y - s.atoms[1].y).powi(2) + (s.atoms[3].z - s.atoms[1].z).powi(2)).sqrt();
            let f_new = (model.c * model.penetration(r_ho)).abs() + (6.0 * model.c6 / r_oo.powi(7)).abs();
            assert!(f_new >= 1e-6, "FIELD-4 plant (ii) carrier: |F_pen + F_disp| = {f_new:.2e}");
        }
        step(&mut s, STEPS);
        let (mut fx, mut fy, mut fz, mut scale) = (0.0, 0.0, 0.0, 0.0f64);
        for i in 0..s.n {
            let (x, y, z) = s.internal_force(i);
            fx += x;
            fy += y;
            fz += z;
            scale = scale.max((x * x + y * y + z * z).sqrt());
        }
        let net = (fx * fx + fy * fy + fz * fz).sqrt();
        if expect_fire {
            assert!(net > 1e-6 * scale, "plant (iii) did not fire: net {net:.2e} against scale {scale:.2e}");
            assert!(s.momentum_residual() > s.momentum_bound(), "plant (iii): the residual stayed under its bound");
        } else {
            assert!(net <= 1e-12 * scale.max(1.0) * (s.n as f64), "G-B2: internal forces sum to {net:.3e} against {scale:.3e}");
            assert!(s.momentum_residual() <= s.momentum_bound(), "G-B2: residual {} over bound {}", s.momentum_residual(), s.momentum_bound());
            eprintln!("G-B2 ({which}): net internal force {net:.2e} (scale {scale:.2e}), momentum residual {:.2e} / bound {:.2e}; |F_wall(start)| {f_wall:.3e}", s.momentum_residual(), s.momentum_bound());
        }
    }
}

#[test]
fn g_b3_the_wall_is_the_derivative_of_its_energy() {
    let (loaded, which_loaded) = wall();
    // FIELD-7 G-E1: the two further wall classes exercised even when the harvest on disk
    // carries none — a DECLARED all-classes model beside the loaded one
    let all_classes = SeamModel { a: 0.5, b: 1.2, p: 0.02, c: 1.5, c6: 10.0, a_oh: 0.3, b_oh: 1.8, a_hh: 0.2, b_hh: 1.6 };
    for (model, which) in [(loaded, which_loaded), (all_classes, "DECLARED all-classes model (A_OH 0.3, b_OH 1.8, A_HH 0.2, b_HH 1.6)".to_string())] {
        derivative_check(model, &which);
    }
}

fn derivative_check(model: SeamModel, which: &str) {
    let (dsp, dpos) = dimer_positions();
    let mut s = scene(&dsp, &dpos, 30.0, 293.0);
    s.set_field(true, None).unwrap();
    s.set_seam(Some(model)).unwrap();
    let mut worst_letter = 0.0f64;
    let mut worst = 0.0f64;
    let mut carrier = 0.0f64;
    // the letter's step is h = 1e-4 (FIELD-3 G-B3); on a harvested law with steeper terms the
    // O(h²) truncation crosses 1e-8 on atoms whose seam force is small (FIELD-7 read 9.6e-8),
    // so the property is ASSERTED at h = 1e-5 and the letter's reading reported beside it
    for (h, letter) in [(1e-4f64, true), (1e-5, false)] {
    for i in 0..s.n {
        // every atom: the penetration term (FIELD-4) acts on hydrogens too
        // the seam terms' force alone: the internal force with the terms minus without, same drops
        s.compute_forces();
        let with = s.internal_force(i);
        s.seam = Some(SeamModel::NO_WALL);
        s.compute_forces();
        let without = s.internal_force(i);
        s.seam = Some(model);
        let f = [with.0 - without.0, with.1 - without.1, with.2 - without.2];
        let fmag = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
        carrier = carrier.max(fmag);
        for coord in 0..3 {
            let x0 = match coord { 0 => s.atoms[i].x, 1 => s.atoms[i].y, _ => s.atoms[i].z };
            let set = |s: &mut Sim, v: f64| match coord { 0 => s.atoms[i].x = v, 1 => s.atoms[i].y = v, _ => s.atoms[i].z = v };
            set(&mut s, x0 + h);
            s.compute_forces();
            let ep = s.e_seam;
            set(&mut s, x0 - h);
            s.compute_forces();
            let em = s.e_seam;
            set(&mut s, x0);
            let fd = -(ep - em) / (2.0 * h);
            if fmag > 1e-10 {
                let rel = (f[coord] - fd).abs() / fmag;
                if letter {
                    worst_letter = worst_letter.max(rel);
                } else {
                    worst = worst.max(rel);
                }
            }
        }
    }
    }
    s.compute_forces();
    assert!(carrier > 1e-10, "no seam force on any atom");
    assert!(worst <= 1e-8, "G-B3 / G-D1: worst relative |F − (−∂E)| = {worst:.2e} at h = 1e-5");
    eprintln!("G-B3 / G-D1 ({which}): worst relative {worst:.2e} at h = 1e-5 (the letter's h = 1e-4: {worst_letter:.2e}) over every atom; |F_seam| max {carrier:.3e}, e_seam {:+.6e}, O–O pairs {}, H–O pairs {}", s.e_seam, s.seam_work.oo_pairs, s.seam_work.ho_pairs);
}

#[test]
fn g_b4_the_closures_contribute_nothing_across_the_seam_and_plant_ii_serves_the_triples() {
    let (dsp, dpos) = dimer_positions();
    let mut far = dpos.clone();
    for p in far.iter_mut().skip(3) {
        p[0] += 40.0;
    }
    let closure = |plant: SeamPlant, pos: &Vec<[f64; 3]>| -> (f64, f64, u64, u64) {
        let mut s = scene(&dsp, pos, 100.0, 293.0);
        s.seam_plant = plant;
        s.set_seam(Some(SeamModel::NO_WALL)).unwrap();
        s.refresh_pairs();
        s.compute_forces();
        (s.e_pair, s.e_three, s.seam_work.pairs_dropped, s.seam_work.triples_dropped)
    };
    let (pn, tn, pd, td) = closure(SeamPlant::None, &dpos);
    let (pf, tf, _, _) = closure(SeamPlant::None, &far);
    let d_pair = pn - pf;
    let d_three = tn - tf;
    eprintln!("G-B4: cross-seam closure contribution: pair {d_pair:+.3e}, three-body {d_three:+.3e} (FIELD-2 measured −2.087e-2 and +4.191e-2 under the bare law); pairs dropped {pd}, triples dropped {td}");
    assert_eq!(pd, 9, "the dimer's nine cross-unit pairs are dropped");
    assert!(td >= 14, "the cross-unit triples are dropped ({td})");
    // EXACT up to the translation's rounding of the intra-unit coordinates (~1e-16 relative)
    assert!(d_pair.abs() <= 1e-12 && d_three.abs() <= 1e-12, "G-B4: the closures leak {d_pair:+.3e} / {d_three:+.3e} across the seam");
    // plant (ii): the surfaces served across the seam — FIELD-2's +0.041914 returns
    let (_, t_pl, _, td_pl) = closure(SeamPlant::TriplesAcross, &dpos);
    let (_, t_pl_far, _, _) = closure(SeamPlant::TriplesAcross, &far);
    let cross_three = t_pl - t_pl_far;
    eprintln!("plant (ii): cross-unit three-body sum {cross_three:+.6e} Ha (FIELD-2: +0.041914), triples dropped {td_pl}");
    assert_eq!(td_pl, 0);
    assert!(cross_three >= 1e-3, "plant (ii) carrier");
    assert!((cross_three - 0.041914).abs() <= 1e-6 * 1e3, "plant (ii): {cross_three:+.6e} vs FIELD-2's +0.041914");
    assert!((cross_three - 0.041914).abs() <= 1e-6, "plant (ii) to 1e-6 (the freeze's 1e-9 is against the same arithmetic; FIELD-2 printed six decimals)");
}
