//! THE SMOOTH SERVING RULE for CT-3's transfer table, gated — one test per gate of
//! `examples/ct3_smooth.rs`, on the campaign's own records.
//!
//! LIQUID-2's labelled screen measured that serving channel 6 at the pair's SHORTEST
//! cross-unit H···O contact — an argmin — costs a 128-water periodic box three orders of
//! energy conservation, and accounted `96.6 %` of the drift to the argmin's own handovers
//! (`conformance/water_observatory/liquid2/DRIFT_NOTE.md`). These are the gates on the
//! replacement: a partition of unity over all four contacts at a DERIVED inverse length, with
//! the full two-term force.
//!
//! The map, the two serving rules and the derivation live in `tests/common/ct3_map.rs`, which
//! `examples/ct3_smooth.rs` includes by the same `#[path]`, so a test here and the runner's
//! own gate are the same arithmetic. Every path is relative to the CRATE ROOT, which is where
//! a test runs.

use holon_render::seam::{CtServe, CtTable, SeamModel};
use holon_render::sim::{Boundary, Sim};
use holon_render::waterbox::liquid_box;
use std::path::PathBuf;

#[path = "common/ct3_map.rs"]
mod ct3_map;
use ct3_map::*;

#[path = "common/field2_scenes.rs"]
mod field2_scenes;
use field2_scenes::scene;

fn observatory() -> PathBuf {
    PathBuf::from("../../../conformance/water_observatory")
}

/// The blend at its derived `β`, on the map's own law.
fn law() -> (SeamModel, CtTable, f64) {
    let obs = observatory();
    let (model, table, beta, _r_cut) = blended_law(&obs);
    (model, table, beta)
}

// ------------------------------------------------------------- the rule is opt-in, bit for bit

/// EVERY RECORD WRITTEN BEFORE THIS RULE STILL READS THE ARGMIN. A table that loads and is
/// never told otherwise serves `Argmin`, carries no inverse length, and refuses a `β` that is
/// not a positive length scale.
#[test]
fn the_argmin_is_the_default_and_the_blend_is_opt_in() {
    let obs = observatory();
    let (mut table, knots) = load_table(&obs.join("ct3").join("ct_table.json"));
    assert!(knots >= 60, "the map's sites are the knots; {knots} were read");
    assert_eq!(table.serve_mode(), CtServe::Argmin, "a table that loads serves the argmin");
    assert_eq!(table.beta(), 0.0, "the argmin carries no inverse length");
    assert_eq!(CtTable::empty().serve_mode(), CtServe::Argmin, "so does an empty one");
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(!table.set_blend(bad), "the table must refuse beta = {bad}");
        assert_eq!(table.serve_mode(), CtServe::Argmin, "a refused beta leaves the rule alone");
    }
    assert!(table.set_blend(read_beta(&obs)));
    assert_eq!(table.serve_mode(), CtServe::Blend);
    table.set_argmin();
    assert_eq!(table.serve_mode(), CtServe::Argmin);
    assert_eq!(table.beta(), 0.0);
    // and a table under the argmin serves nothing through the blend
    let (model, _, _) = law();
    let x: Six = [[0.0, 0.0, 0.0], [0.0, 0.0, 1.9], [1.8, 0.0, -0.2], [0.0, 0.0, 5.2], [1.4, 0.0, 6.5], [-1.4, 0.0, 6.5]];
    assert_eq!(blend_pair(&table, &model, &x).0, 0.0, "the blend is inert under the argmin");
}

// ------------------------------------------------------------------- (a) THE VALUE

/// GATE (a). At every one of the map's sixty-four nodes the blended reading differs from the
/// argmin's by less than the table's OWN resolution floor — the largest spread the four
/// coordinates cannot resolve, which is the smallest difference the table is entitled to
/// claim. `β` was derived to make this true and this is where it is measured.
#[test]
fn a_the_blend_reproduces_the_argmin_at_every_node() {
    let obs = observatory();
    let (model, table, beta) = law();
    let (nodes, floor, _resolution) = load_map(&obs);
    assert_eq!(nodes.len(), 64);
    let mut worst = 0.0f64;
    let mut at = String::new();
    for n in nodes.iter() {
        let d = (blend_pair(&table, &model, &n.six).0 - argmin_pair(&table, &model, &n.six).0).abs();
        if d > worst {
            worst = d;
            at = n.name.clone();
        }
    }
    assert!(
        worst < floor,
        "beta = {beta} per bohr leaves {worst:.6e} hartree between the rules at {at}, over the table's own floor {floor:.6e}"
    );
}

// ------------------------------------------------------------------- (b) THE FORCE

/// GATE (b). The analytic gradient against central differences at `h = 1e-5`, on every class
/// the second external review named: the map's nodes, hydrogen permutations, molecule
/// exchange, contact TIES and periodic crossings — and the forces sum to zero. The recorded
/// handover geometries of LIQUID-2's own arm are the runner's business (they need the arm);
/// what is testable without one is every other class, and the ties are the ones the argmin
/// could not do at all.
#[test]
fn b_the_forces_are_the_analytic_gradient() {
    let obs = observatory();
    let (model, table, _beta) = law();
    let (nodes, _floor, _resolution) = load_map(&obs);
    let l_box = {
        let t = std::fs::read_to_string(obs.join("liquid1").join("door.json")).expect("liquid1/door.json");
        json_num(&t, "cell_edge_bohr")
    };
    assert!(l_box.is_finite() && l_box > 0.0, "the cell edge comes from liquid1/door.json");

    let mut set: Vec<(String, Six)> = Vec::new();
    for n in nodes.iter() {
        set.push((n.name.clone(), n.six));
        set.push((format!("{} h_A swapped", n.name), swap_ha(&n.six)));
        set.push((format!("{} h_B swapped", n.name), swap_hb(&n.six)));
        set.push((format!("{} units exchanged", n.name), swap_units(&n.six)));
    }
    // the map's own two EXACT ties — the donor bent 90°, both hydrogens equidistant — and one
    // bisected tie on each of CT-3's four G-A0 separations
    for n in nodes.iter().filter(|n| n.name.ends_with("_b90")) {
        set.push((format!("{} (an exact contact tie)", n.name), n.six));
    }
    let mut ties = 0;
    for name in TURN_NODES.iter() {
        let n = nodes.iter().find(|n| n.name == *name).expect("the map carries its linear nodes");
        let (g, residual) = tie_geometry(&n.six, 1.0, 179.0);
        assert!(residual < 1e-9, "{name}: the bisected tie is {residual:.3e} bohr from a tie");
        set.push((format!("{name} turned to a contact tie"), g));
        ties += 1;
    }
    assert_eq!(ties, 4, "one constructed tie per G-A0 separation");

    let mut worst = 0.0f64;
    let mut worst_at = String::new();
    let mut trans = 0.0f64;
    for (name, g) in set.iter() {
        let (w, t, ai, ac) = fd_worst(&table, &model, g, FD_H);
        if w > worst {
            worst = w;
            worst_at = format!("{name}, atom {ai} coordinate {ac}");
        }
        trans = trans.max(t);
    }
    assert!(worst <= FD_TOL, "worst relative force error {worst:.6e} at {worst_at}, over {FD_TOL:e}");
    assert!(trans <= FD_TOL, "the forces do not sum to zero: {trans:.6e}");

    // periodic crossings: the wrap is INSIDE the difference loop, and a pair read across a
    // face is the same pair
    let mut p_worst = 0.0f64;
    let mut p_trans = 0.0f64;
    let mut identity = 0.0f64;
    for n in nodes.iter() {
        for axis in 0..3 {
            let mut raw = n.six;
            for i in 3..6 {
                raw[i][axis] += l_box;
            }
            let (w, t) = fd_worst_wrapped(&table, &model, &raw, l_box, FD_H);
            p_worst = p_worst.max(w);
            p_trans = p_trans.max(t);
            let e0 = blend_pair(&table, &model, &n.six).0;
            let e1 = blend_pair(&table, &model, &wrap_pair(&n.six, l_box, axis)).0;
            identity = identity.max(if e0 == 0.0 { (e1 - e0).abs() } else { ((e1 - e0) / e0).abs() });
        }
    }
    assert!(p_worst <= FD_TOL, "worst relative force error across a face {p_worst:.6e}");
    assert!(p_trans <= FD_TOL, "the wrapped forces do not sum to zero: {p_trans:.6e}");
    assert!(identity <= FD_TOL, "a pair read across a face is not the same pair: {identity:.6e}");
}

// ------------------------------------------------------------------- (c) THE CONTINUITY

/// GATE (c). CT-3's own G-A0 sweep — the donor turned through a full turn at four separations
/// — read at TWO resolutions, because the largest step between adjacent samples is not by
/// itself a statement about continuity. Between samples of a smooth function it is `O(dθ)` and
/// HALVES when the samples double; a jump is the same size however finely it is approached.
/// The blend's step must be under the table's resolution floor and must halve; the argmin's
/// jump is measured on the same sweep and does not.
#[test]
fn c_the_blend_is_continuous_where_the_argmin_jumps() {
    let obs = observatory();
    let (model, table, _beta) = law();
    let (nodes, floor, _resolution) = load_map(&obs);
    let mut blend_step = 0.0f64;
    let mut blend_step_fine = 0.0f64;
    let mut argmin_jump = 0.0f64;
    let mut argmin_jump_fine = 0.0f64;
    let mut handovers = 0u64;
    for name in TURN_NODES.iter() {
        let n = nodes.iter().find(|n| n.name == *name).expect("the map carries its linear nodes");
        let coarse = turn_sweep(&table, &model, &n.six, TURN_STEPS);
        let fine = turn_sweep(&table, &model, &n.six, 2 * TURN_STEPS);
        blend_step = blend_step.max(coarse.blend_step);
        blend_step_fine = blend_step_fine.max(fine.blend_step);
        argmin_jump = argmin_jump.max(coarse.argmin_jump);
        argmin_jump_fine = argmin_jump_fine.max(fine.argmin_jump);
        handovers += coarse.handovers;
    }
    assert!(handovers > 0, "a sweep with no handover would gate nothing (M-VACUOUS-SUCCESS)");
    assert!(
        blend_step < floor,
        "the blend's largest step over the sweep is {blend_step:.6e} hartree, over the table's own floor {floor:.6e}"
    );
    let ratio = blend_step / blend_step_fine;
    assert!(
        (1.8..=2.2).contains(&ratio),
        "the blend's largest step is {blend_step:.6e} coarse and {blend_step_fine:.6e} refined, a ratio of {ratio:.4}; a continuous sweep halves and a discontinuity does not"
    );
    // and the argmin's jump is the thing that does NOT shrink — the control on the leg above
    let argmin_ratio = argmin_jump / argmin_jump_fine;
    assert!(
        argmin_ratio < 1.8,
        "the argmin's jump {argmin_jump:.6e} refined to {argmin_jump_fine:.6e}, a ratio of {argmin_ratio:.4}; if the argmin's own jump halves too then this sweep is not resolving a handover and the leg above proves nothing"
    );
}

// ------------------------------------------------------------------- (d) THE DRIFT

/// GATE (d)'s SHAPE, on a box a test can afford. The full gate is the 128-water box at the
/// tables' step for 2,000 frames and it lives in the runner; this is the same comparison on a
/// 16-water periodic cell over a short arm — the blended law's drift peak against the SAME box
/// with channel 6 switched off. It is the cheap standing guard: a serving rule that reopened
/// the defect would fail here long before a campaign paid for the big box.
#[test]
fn d_the_blend_drifts_like_the_channel_off_control() {
    let obs = observatory();
    let (model, table, _beta) = law();
    // the state point is LIQUID-2's; the CELL is deliberately small, and it is the same cell
    // for both arms, so the only knob is the serving rule.
    //
    // THE SWITCH IS RE-DERIVED FOR THIS CELL by LIQUID-1 Amendment 2's own rule, which is the
    // half-edge less the margin that record itself carries (`liquid1/door.json`:
    // `half_edge_bohr` less `seam_switch_r_cut_bohr`). A 54-water cell cannot honour the
    // 128-water cell's 14-bohr reach under the minimum image, and pretending otherwise is how
    // a small test comes to run a law nobody serves.
    const DENSITY_G_CM3: f64 = 0.997;
    const TEMPERATURE_K: f64 = 293.0;
    const SEED: u64 = 0x4c49_5155_4944;
    const CELLS: usize = 3;
    const SETTLE: usize = 60;
    const COUNT: usize = 250;
    let margin = {
        let t = std::fs::read_to_string(obs.join("liquid1").join("door.json")).expect("liquid1/door.json");
        json_num(&t, "half_edge_bohr") - json_num(&t, "seam_switch_r_cut_bohr")
    };
    assert!(margin > 0.0 && margin.is_finite(), "the switch's own margin comes from liquid1/door.json");

    let arm = |with_table: bool| -> f64 {
        let (species, pos, l) = liquid_box(CELLS, DENSITY_G_CM3, SEED);
        let mut sim: Box<Sim> = scene(&species, &pos, l, TEMPERATURE_K);
        sim.set_field(true, None).expect("the open box admits the field");
        let model = SeamModel { r_cut: 0.5 * l - margin, ..model };
        let m = if with_table {
            sim.ct_table = table.clone();
            model
        } else {
            SeamModel { p_ct: 0.0, c_ct: 0.0, m_ct: 0, k_ct: 0, lambda_ct: 0.0, ct_table_on: false, ..model }
        };
        sim.set_seam(Some(m)).expect("no acuity frame is installed");
        sim.set_boundary(Boundary::Periodic).expect("the image rule admits the cell");
        sim.rebase();
        for _ in 0..SETTLE {
            sim.step_frame(1);
        }
        sim.rebase();
        for _ in 0..COUNT {
            sim.step_frame(1);
        }
        assert!(sim.pbc_ok(), "the arm stayed in the box");
        sim.drift_peak
    };
    let blend = arm(true);
    let control = arm(false);
    assert!(control > 0.0, "a control that drifts by an exact zero would gate nothing");
    assert!(
        blend <= 2.0 * control,
        "the blended law drifts {blend:.6e} hartree against the channel-6-off control's {control:.6e} on the same cell — a factor {:.2}",
        blend / control
    );
}

// ------------------------------------------------------------------- the derivation itself

/// `β` IS DERIVED. Re-derive it from the records and check the written one against it, so a
/// hand-edited `beta.json` cannot quietly become the law; and check the two properties the
/// derivation rests on — the map has exactly two EXACT ties, and the binding node's own
/// requirement is what was adopted.
#[test]
fn beta_is_the_records_own_number() {
    let obs = observatory();
    let b = derive_beta(&obs);
    let written = read_beta(&obs);
    assert_eq!(b.rows.len(), 64, "the map is sixty-four nodes");
    assert!(b.floor > 0.0, "the resolution floor is the table's largest pole spread");
    assert_eq!(b.ties.len(), 2, "the map's exact ties are the two b90 nodes: {:?}", b.ties);
    for t in b.ties.iter() {
        assert!(t.ends_with("_b90"), "an exact tie that is not a bent-90 donor: {t}");
    }
    assert!(b.beta > 0.0 && b.beta.is_finite(), "beta is a positive inverse length");
    assert!(
        (written - b.beta).abs() <= 1e-9 * b.beta,
        "beta.json carries {written} where the records give {}",
        b.beta
    );
    // the adopted value is the LARGEST requirement, so the criterion holds at every node that
    // has one — not only at the closest pair of contacts
    assert!(b.beta >= b.beta_at_dr_min, "the adopted beta must cover the smallest separation too");
    let binding = b.rows.iter().find(|r| r.name == b.beta_node).expect("the binding node is one of the map's");
    assert!((binding.required - b.beta).abs() <= 1e-12 * b.beta, "the binding node's requirement is the adopted beta");
    for r in b.rows.iter().filter(|r| r.required.is_finite()) {
        assert!(r.required <= b.beta + 1e-12 * b.beta, "{} needs {} which is over the adopted {}", r.name, r.required, b.beta);
    }
}

/// The engine's own blend and the runner's fold agree with a hand-written partition of unity,
/// so the closed form `∇E = Σ w_k ∇Ẽ_k − β Σ w_k (Ẽ_k − E) ∇r_k` is checked against the
/// SPECIFICATION's own two terms rather than only against itself.
#[test]
fn the_blend_is_the_specified_partition_of_unity() {
    let obs = observatory();
    let (model, table, beta) = law();
    let (nodes, _floor, _resolution) = load_map(&obs);
    let mut worst = 0.0f64;
    for n in nodes.iter() {
        let pts = contacts_of(&n.six);
        let r = contact_r(&n.six);
        // the weights, written out
        let rmin = r.iter().cloned().fold(f64::INFINITY, f64::min);
        let ex: Vec<f64> = r.iter().map(|x| (-beta * (x - rmin)).exp()).collect();
        let z: f64 = ex.iter().sum();
        let w: Vec<f64> = ex.iter().map(|x| x / z).collect();
        assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-12, "the weights are a partition of unity");
        let mut e = 0.0f64;
        for (k, c) in pts.iter().enumerate() {
            let (sw, _) = model.switch(r[k]);
            let ek = if sw == 0.0 { 0.0 } else { sw * table.serve(c[0], c[1], c[2], c[3], c[4]).0 };
            e += w[k] * ek;
        }
        worst = worst.max((e - blend_pair(&table, &model, &n.six).0).abs());
    }
    assert!(worst < 1e-15, "the engine's blend differs from the written-out rule by {worst:.3e}");
}

/// The engine serves the blend through `Sim::accumulate_seam`, and what it serves is what the
/// rule says: a two-water scene's seam energy under the blend equals the blended pair energy
/// plus the pair rows, and the SWITCH kills it past the cutoff exactly.
#[test]
fn the_engine_serves_the_rule_it_declares() {
    let obs = observatory();
    let (model, table, _beta) = law();
    let (nodes, _floor, _resolution) = load_map(&obs);
    let n = nodes.iter().find(|n| n.name == "linear_R2.7").expect("the map carries linear_R2.7");

    let build = |six: Six, with_blend: bool| -> f64 {
        let species = vec![
            holon_chem::elements::OXYGEN,
            holon_chem::elements::HYDROGEN,
            holon_chem::elements::HYDROGEN,
            holon_chem::elements::OXYGEN,
            holon_chem::elements::HYDROGEN,
            holon_chem::elements::HYDROGEN,
        ];
        let pos: Vec<[f64; 3]> = six.iter().map(|p| [p[0] + 20.0, p[1] + 15.0, p[2] + 15.0]).collect();
        let mut s = scene(&species, &pos, 60.0, 1.0);
        s.boundary = Boundary::Open;
        s.set_field(true, None).expect("the open box admits the field");
        let mut t = table.clone();
        if !with_blend {
            t.set_argmin();
        }
        s.ct_table = t;
        s.set_seam(Some(model)).expect("no acuity frame");
        s.refresh_pairs();
        s.compute_forces();
        s.e_seam
    };
    // the argmin and the blend are the same law within the table's own floor at a node
    let (_, floor, _) = load_map(&obs);
    let d = (build(n.six, true) - build(n.six, false)).abs();
    assert!(d < floor, "the engine's two rules differ by {d:.6e} at linear_R2.7, over the floor {floor:.6e}");
    // and past the switch both are an exact zero
    let mut far = n.six;
    for i in 3..6 {
        far[i][2] += 3.0 * model.r_cut;
    }
    assert_eq!(build(far, true), 0.0, "past r_cut the blend serves an exact zero");
    assert_eq!(build(far, false), 0.0, "and so does the argmin");
}
