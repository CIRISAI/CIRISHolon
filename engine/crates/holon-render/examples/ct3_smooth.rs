//! CT-3 SMOOTH: the transfer table served as a PARTITION OF UNITY over the four cross-unit
//! H···O contacts, in place of the argmin LIQUID-2 measured a drift defect on.
//!
//! ```text
//! cargo run --release -p holon-render --example ct3_smooth -- beta       [DIR]
//! cargo run --release -p holon-render --example ct3_smooth -- handovers  [DIR]
//! cargo run --release -p holon-render --example ct3_smooth -- drift      [DIR] --law <blend|ct3-noct|ct3>
//! cargo run --release -p holon-render --example ct3_smooth -- gate       [DIR]
//! cargo run --release -p holon-render --example ct3_smooth -- nve        [DIR] [--step <x> --ps <t>]
//! ```
//!
//! `DIR` is `conformance/water_observatory/ct3` and every record lands under `ct3/smooth/`.
//!
//! # What was wrong, and what this is
//!
//! CT-3 serves channel 6 ONCE per unordered pair of units, at the pair's SHORTEST cross-unit
//! H···O contact — an ARGMIN over four candidates. LIQUID-2's labelled screen
//! (`liquid2/DRIFT_NOTE.md`, every file `dry: true`) measured what that costs a 128-water
//! periodic box: a drift peak of `6.585e-3` hartree against `4.720e-6` with channel 6 off on
//! the same box, seed, settling and frame count — three orders — and its handover counter
//! accounted `96.6 %` of it as the argmin's own jumps (3,803 handovers in 2,000 frames, the
//! running extremum of their SIGNED sum `6.361e-3` against the measured `6.585e-3`).
//!
//! The table's VALUES are not what failed. The argmin is. This runner builds, derives and
//! gates the declared replacement:
//!
//! ```text
//! E_pair = Σ_k w_k · S(r_k) · E_table(coords_k),   w_k = e^{−β r_k} / Σ_j e^{−β r_j}
//! −∇E    = −Σ_k w_k ∇Ẽ_k − Σ_k Ẽ_k ∇w_k,           ∇w_k = w_k(−β ∇r_k + β Σ_j w_j ∇r_j)
//! ```
//!
//! on all five atoms of all four contacts, with the switch applied PER CONTACT. The second
//! force term is the one the rule's first specification omitted and the second external
//! review caught; it is carried here and it is what the finite-difference gate tests.
//!
//! `β` IS DERIVED AND NEVER TYPED. `beta` computes it from the map's own records — the
//! shortest-to-second-shortest cross-unit H···O separation at each of the sixty-four nodes,
//! against the table's own resolution floor (its largest pole spread) as a fraction of that
//! node's `|E_CT|` — and prints the arithmetic and every input's path.
//!
//! # The gates
//!
//! * **(a) the value**: at every map node the blended reading differs from the argmin's by
//!   less than the table's resolution floor.
//! * **(b) the force**: central differences at `h = 1e-5` against the analytic gradient, on
//!   the 64 nodes, on the RECORDED handover geometries of LIQUID-2's own 128-water arm, on
//!   hydrogen-permuted and molecule-exchanged copies of those, on contact TIES and on
//!   periodic crossings; and the forces sum to zero.
//! * **(c) the continuity**: CT-3's own G-A0 sweep — the donor turned through a full turn at
//!   four separations — with the served energy's largest STEP under the resolution floor AND
//!   halving when the sweep's samples double, which is what separates a smooth variation from
//!   a jump; the argmin's own jump, which does not halve, is measured beside it.
//! * **(d) the drift**: the 128-water box at the tables' step, blended, within `2x` the
//!   channel-6-off control measured in the same session on the same box.
//!
//! The map, the two serving rules, the finite-difference machinery and the derivation of `β`
//! live in `tests/common/ct3_map.rs`, shared with `tests/ct3_smooth.rs` by `#[path]`, so the
//! runner and the unit tests run the SAME rule and cannot drift apart.
//!
//! Nothing here is a reading: every record carries `dry: true`.

use holon_campaign::{num, Gate, Record, RecordWriter};
use holon_lens::lens::{hbonds_periodic, rdf_oo};
use holon_render::seam::{CtTable, SeamModel, CT_CONTACTS};
use holon_render::sim::{Boundary, Sim};
use holon_render::waterbox::{first_peak, liquid_box};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/field2_scenes.rs"]
mod field2_scenes;
use field2_scenes::scene;

#[path = "../tests/common/ct3_map.rs"]
mod ct3_map;
use ct3_map::{
    argmin_pair, blend_pair, contact_r, contacts_of, derive_beta, fd_worst, fd_worst_wrapped, json_num, json_str,
    load_map, load_table, mind, read_beta, swap_ha, swap_hb, swap_units, tie_geometry, turn_sweep, wrap_pair, Six,
    FD_H, FD_TOL, TURN_NODES, TURN_STEPS,
};

// ------------------------------------------------------------- LIQUID-2's box, restated and CHECKED
//
// The state point is LIQUID-2's own (`examples/liquid2.rs`, the runner that wrote every
// record under `liquid2/screen/`). It is restated here rather than imported because an
// example is not a module, and it is CHECKED rather than trusted: `drift --law ct3-noct`
// re-runs the channel-6-off control on this box and the gate reports it against the recorded
// `4.719995e-6`. A box that did not reproduce that number would not be LIQUID-2's box, and
// the gate would say so.
const DENSITY_G_CM3: f64 = 0.997;
const TEMPERATURE_K: f64 = 293.0;
const SEED: u64 = 0x4c49_5155_4944;
const N_CELLS: usize = 4;
const N_WATERS: usize = 128;

/// How many of the worst handover geometries the dump keeps. A gate needs the liquid's worst
/// cases, not all of them; the counter's own worst single jump is among these by construction.
const HANDOVERS_KEPT: usize = 32;

/// The radial distribution's declared bin, LIQUID-1's and LIQUID-2's.
const RDF_DR: f64 = 0.1;

fn observatory(out: &Path) -> PathBuf {
    // `out` is `.../water_observatory/ct3`; the observatory is its parent
    out.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("../conformance/water_observatory"))
}

// ------------------------------------------------------------------------------ the law

/// Which law an arm runs. `Blend` and `Argmin` are the SAME law — CT-3's `wall_ct3.json` with
/// the table — differing only in the serving rule; `NoTransfer` is the control, channel 6 off
/// entirely, and it is LIQUID-2's own `ct3-noct`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Blend,
    Argmin,
    NoTransfer,
}

impl Arm {
    fn name(self) -> &'static str {
        match self {
            Arm::Blend => "ct3-blend",
            Arm::Argmin => "ct3",
            Arm::NoTransfer => "ct3-noct",
        }
    }
    fn parse(s: &str) -> Arm {
        match s {
            "blend" | "ct3-blend" => Arm::Blend,
            "ct3-noct" | "noct" => Arm::NoTransfer,
            "ct3" | "argmin" => Arm::Argmin,
            other => panic!("unknown law {other:?}: blend | ct3 | ct3-noct"),
        }
    }
}

struct Law {
    model: SeamModel,
    table: CtTable,
    arm: Arm,
    source: String,
    table_source: String,
    r_cut: f64,
    beta: f64,
}

/// CT-3's admitted law, with LIQUID-1 Amendment 2's switch read off `liquid1/door.json` and
/// the serving rule installed. `β` comes from `ct3/smooth/beta.json`, which `beta` wrote.
fn load_law(obs: &Path, arm: Arm) -> Law {
    let p = obs.join("ct3").join("wall_ct3.json");
    let tp = obs.join("ct3").join("ct_table.json");
    let dp = obs.join("liquid1").join("door.json");
    let t = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let d = std::fs::read_to_string(&dp).unwrap_or_else(|e| panic!("{}: {e}", dp.display()));
    let r_cut = json_num(&d, "seam_switch_r_cut_bohr");
    assert!(r_cut.is_finite() && r_cut > 0.0, "{}: no seam_switch_r_cut_bohr", dp.display());
    let g = |k: &str| {
        let v = json_num(&t, k);
        if v.is_finite() {
            v
        } else {
            0.0
        }
    };
    let (mut table, _knots) = load_table(&tp);
    let mut beta = 0.0;
    if arm == Arm::Blend {
        beta = read_beta(obs);
        assert!(table.set_blend(beta), "the table refused beta = {beta}");
    }
    let model = SeamModel {
        a: g("a"),
        b: g("b"),
        p: g("p"),
        c: g("c"),
        c6: g("c6"),
        a_oh: g("a_oh"),
        b_oh: g("b_oh"),
        a_hh: g("a_hh"),
        b_hh: g("b_hh"),
        p_hh: g("p_hh"),
        c_hh: g("c_hh"),
        p_ct: g("p_ct"),
        c_ct: g("c_ct"),
        m_ct: g("m_ct") as u8,
        k_ct: g("k_ct") as u8,
        lambda_ct: g("lambda_ct"),
        r_cut,
        ct_table_on: arm != Arm::NoTransfer,
    };
    let model = match arm {
        Arm::NoTransfer => SeamModel { p_ct: 0.0, c_ct: 0.0, m_ct: 0, k_ct: 0, lambda_ct: 0.0, ..model },
        _ => model,
    };
    Law { model, table, arm, source: p.display().to_string(), table_source: tp.display().to_string(), r_cut, beta }
}

// -------------------------------------------------------------------------- the derivation of beta

fn beta_phase(obs: &Path, out: &Path) {
    let b = derive_beta(obs);
    let w = RecordWriter::screen(&out.join("smooth"), "beta");
    println!("THE INVERSE LENGTH, DERIVED — every input a record's, nothing typed\n");
    println!(
        "  the table's resolution floor          {:.6e} hartree   (largest pole spread, {}/ct3/ct_table.json)",
        b.floor,
        obs.display()
    );
    println!(
        "  the geometry records' own resolution  {:.6e} bohr      (the decimals ct2/node_*.json and ct2/gd0_*.json print)",
        b.resolution
    );
    println!("  nodes read                            {}                (ct2/node_*.json + ct2/gd0_*.json)", b.rows.len());
    println!();
    println!("  EXACT TIES (Δr at or below the records' own resolution; the argmin is a coin flip there):");
    for name in b.ties.iter() {
        let r = b.rows.iter().find(|x| x.name == *name).unwrap();
        println!(
            "    {:<24} r1 = {:.9}  r2 = {:.9}  Δr = {:.3e}  |E_CT| = {:.6e}",
            r.name,
            r.r1,
            r.r2,
            r.dr,
            r.e_ct.abs()
        );
    }
    println!();
    println!("  the criterion, per node:  w₂ ≤ 1/(1 + e^(βΔr)) < floor/|E_CT|  ⇒  β > ln(|E_CT|/floor − 1)/Δr");
    println!("  vacuous where floor/|E_CT| ≥ 1/2 ({} nodes: {})", b.vacuous.len(), b.vacuous.join(", "));
    println!();
    println!("  Δr_min (the smallest real separation)  {:.9} bohr at {}", b.dr_min, b.dr_min_node);
    let first = b.rows.iter().find(|r| r.name == b.dr_min_node).unwrap();
    println!("    |E_CT| there                         {:.6e} hartree", first.e_ct.abs());
    println!("    fraction floor/|E_CT|                {:.9}", first.fraction);
    println!("    ln(1/f − 1)/Δr                       {:.9} per bohr", b.beta_at_dr_min);
    println!();
    let hardest = b.rows.iter().find(|r| r.name == b.beta_node).unwrap();
    println!("  THE BINDING NODE (the largest requirement, which is what is adopted): {}", b.beta_node);
    println!("    Δr                                   {:.9} bohr", hardest.dr);
    println!("    |E_CT|                               {:.6e} hartree", hardest.e_ct.abs());
    println!("    fraction floor/|E_CT|                {:.9}", hardest.fraction);
    println!("    ln(1/f − 1)/Δr                       {:.9} per bohr", hardest.required);
    println!();
    println!("  β = {:.9} per bohr", b.beta);

    let rows_json: Vec<String> = b
        .rows
        .iter()
        .map(|r| {
            format!(
                "{{\"node\": {:?}, \"r_shortest_bohr\": {}, \"r_second_bohr\": {}, \"separation_bohr\": {}, \"e_ct_hartree\": {}, \"floor_over_abs_e_ct\": {}, \"beta_required_per_bohr\": {}, \"tie\": {}, \"criterion_vacuous\": {}}}",
                r.name,
                num(r.r1),
                num(r.r2),
                num(r.dr),
                num(r.e_ct),
                num(r.fraction),
                num(r.required),
                r.tie,
                r.vacuous
            )
        })
        .collect();
    let rec = Record::new("beta")
        .text("what", "the blend's inverse length, DERIVED from the map's own records; no number here is typed")
        .text(
            "rule",
            "at each of the map's 64 nodes take the shortest and second-shortest cross-unit H...O contact; over four contacts the second one's weight is at most 1/(1 + e^(beta * separation)); require it below the table's own resolution floor as a fraction of that node's |E_CT|; beta is the largest requirement over every node where the criterion has content",
        )
        .text("resolution_floor_source", &format!("{}/ct3/ct_table.json (the largest pole spread over its sites)", obs.display()))
        .text(
            "geometry_source",
            &format!("{}/ct2/node_*.json and {}/ct2/gd0_*.json (donor_centers, acceptor_centers)", obs.display(), obs.display()),
        )
        .text("e_ct_source", &format!("{}/ct3/ct_table.json (each node's site value)", obs.display()))
        .number("resolution_floor_hartree", b.floor)
        .number("geometry_record_resolution_bohr", b.resolution)
        .int("nodes", b.rows.len() as i64)
        .number("separation_min_bohr", b.dr_min)
        .text("separation_min_node", &b.dr_min_node)
        .number("beta_at_separation_min_per_bohr", b.beta_at_dr_min)
        .number("beta_per_bohr", b.beta)
        .text("beta_binding_node", &b.beta_node)
        .raw("ties", format!("[{}]", b.ties.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>().join(", ")))
        .text(
            "ties_note",
            "the donor bent 90 degrees puts both its hydrogens equidistant from the acceptor's oxygen by the builder's own symmetry; the separation there is round-off, no finite beta resolves it, and the argmin itself is a coin flip. Gate (a) MEASURES what the blend does at those two nodes rather than assuming it.",
        )
        .raw("criterion_vacuous", format!("[{}]", b.vacuous.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>().join(", ")))
        .raw("nodes_table", format!("[{}]", rows_json.join(", ")));
    w.write("beta.json", &rec).expect("the derivation writes");
    w.done("beta.done", "a derivation, not a reading").expect("the marker writes");
}

// ------------------------------------------------------------------------------ the box

/// LIQUID-2's own box, its own field, its own law and its own step, in the order
/// `examples/liquid2.rs::build_sized` sets them — the table INSTALLED before the seam, and the
/// step taken through the engine's sanctioned toggle rather than assigned — and then
/// `Boundary::Periodic`, which LIQUID-2 sets at the END of its own door and which is the whole
/// difference between this box and an open one. A first pass of this runner left it out; the
/// channel-6-off control then read `5.034204065e-6` where the record reads `4.719995e-6`, and
/// the mismatch is what found it. The control is re-measured against that record for exactly
/// this reason.
fn build(law: &Law, step_mult: f64) -> (Box<Sim>, f64, f64) {
    let (species, pos, l) = liquid_box(N_CELLS, DENSITY_G_CM3, SEED);
    let mut sim: Box<Sim> = scene(&species, &pos, l, TEMPERATURE_K);
    sim.set_field(true, None).expect("the open box admits the field");
    if law.arm != Arm::NoTransfer {
        sim.ct_table = law.table.clone();
    }
    sim.set_seam(Some(law.model)).expect("no acuity frame is installed");
    let dt_ref = sim.timescale.dt_reference;
    let dt_tables = sim.dt();
    if step_mult != 1.0 {
        sim.timescale.allow_dt_growth = true;
        sim.timescale.set_dt_multiplier(step_mult * dt_tables / dt_ref);
        let want = dt_tables * step_mult;
        assert!((sim.dt() - want).abs() <= 1e-12 * want, "the step did not take: {} against {want}", sim.dt());
    }
    sim.set_boundary(Boundary::Periodic).expect("the image rule admits the cell");
    (sim, l, dt_tables)
}

/// LIQUID-2's screen arm, read off its own record: the settling and counted frame counts, and
/// the drift peak the arm measured. Nothing about the screen is retyped here.
struct ScreenRecord {
    settle: usize,
    count: usize,
    drift_peak: f64,
    path: String,
}

fn screen_record(obs: &Path, label: &str) -> ScreenRecord {
    let p = obs.join("liquid2").join("screen").join(format!("{label}.json"));
    let t = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    ScreenRecord {
        settle: json_num(&t, "settle_frames") as usize,
        count: json_num(&t, "counted_frames") as usize,
        drift_peak: json_num(&t, "drift_peak"),
        path: p.display().to_string(),
    }
}

fn positions(sim: &Sim) -> Vec<[f64; 3]> {
    (0..sim.n).map(|i| [sim.atoms[i].x, sim.atoms[i].y, sim.atoms[i].z]).collect()
}

/// The oxygens and each unit's two hydrogens, as `Sim::accumulate_seam` reads them.
fn units_of(sim: &Sim) -> (Vec<usize>, Vec<[usize; 2]>) {
    let z: Vec<u32> = (0..sim.n).map(|i| sim.atoms[i].species.z).collect();
    let oxy: Vec<usize> = (0..sim.n).filter(|&i| z[i] == 8).collect();
    let units = sim.units_reading();
    let mut unit_h = vec![[usize::MAX; 2]; sim.n];
    for &o in oxy.iter() {
        let hs: Vec<usize> = (0..sim.n).filter(|&i| z[i] == 1 && units[i] == o as u32).collect();
        if hs.len() == 2 {
            unit_h[o] = [hs[0], hs[1]];
        }
    }
    (oxy, unit_h)
}

/// One pair of units as six atoms, relative to unit A's oxygen under the minimum image — the
/// same origin and the same order `Sim::accumulate_seam` uses under the blend.
fn six_of(pos: &[[f64; 3]], ua: usize, ha: [usize; 2], ub: usize, hb: [usize; 2], l: f64) -> Six {
    let p0 = pos[ua];
    [
        [0.0; 3],
        mind(p0, pos[ha[0]], l),
        mind(p0, pos[ha[1]], l),
        mind(p0, pos[ub], l),
        mind(p0, pos[hb[0]], l),
        mind(p0, pos[hb[1]], l),
    ]
}

// ---------------------------------------------------- the handover geometries, dumped from the arm

/// THE LIQUID'S OWN WORST CASES. LIQUID-2's handover counter recorded the jumps and not the
/// geometries, so this re-runs the SAME 1x argmin arm — the same box, seed, settling and
/// counted frames — and dumps the six atoms of every pair that hands over, keeping the
/// largest jumps. It is a screen: the counter reads positions and writes none, so the
/// trajectory is bit-identical to the arm the drift note measured, and the phase checks that
/// by reporting its own drift peak against the record's.
fn handovers_phase(obs: &Path, out: &Path) {
    let law = load_law(obs, Arm::Argmin);
    let rec = screen_record(obs, "x1_ct3_handovers");
    let recorded_events = {
        let t = std::fs::read_to_string(&rec.path).unwrap_or_default();
        json_num(&t, "handovers") as i64
    };
    let (mut sim, l, dt_tables) = build(&law, 1.0);
    let dir = out.join("smooth");
    let w = RecordWriter::screen(&dir, "handovers");
    sim.rebase();
    for _ in 0..rec.settle {
        sim.step_frame(1);
    }
    sim.rebase();
    let (oxy, unit_h) = units_of(&sim);
    let n = oxy.len();
    let mut prev: Vec<usize> = vec![usize::MAX; n * n];
    let mut kept: Vec<(f64, Six, String)> = Vec::new();
    let mut events = 0u64;
    let mut abs_sum = 0.0f64;
    let mut signed = 0.0f64;
    let mut signed_peak = 0.0f64;
    let mut worst = 0.0f64;
    let t0 = Instant::now();
    for f in 0..rec.count {
        sim.step_frame(1);
        let pos = positions(&sim);
        for ia in 0..n {
            let ua = oxy[ia];
            for ib in (ia + 1)..n {
                let ub = oxy[ib];
                let six = six_of(&pos, ua, unit_h[ua], ub, unit_h[ub], l);
                let r = contact_r(&six);
                let mut k0 = 0;
                for k in 1..CT_CONTACTS {
                    if r[k] < r[k0] {
                        k0 = k;
                    }
                }
                let served = law.model.switch(r[k0]).0 != 0.0;
                let now = if served { k0 } else { usize::MAX };
                let key = ia * n + ib;
                let was = prev[key];
                if was != usize::MAX && now != usize::MAX && was != now {
                    // the jump, both contacts read at THIS frame's geometry, exactly as
                    // LIQUID-2's counter reads it
                    let cs = contacts_of(&six);
                    let e_of = |k: usize| -> f64 {
                        let (sw, _) = law.model.switch(r[k]);
                        if sw == 0.0 {
                            0.0
                        } else {
                            sw * law.table.serve(cs[k][0], cs[k][1], cs[k][2], cs[k][3], cs[k][4]).0
                        }
                    };
                    let jump = e_of(now) - e_of(was);
                    events += 1;
                    abs_sum += jump.abs();
                    signed += jump;
                    if jump.abs() > worst {
                        worst = jump.abs();
                    }
                    if signed.abs() > signed_peak {
                        signed_peak = signed.abs();
                    }
                    kept.push((jump.abs(), six, format!("frame {f}, units {ua} and {ub}, contact {was} -> {now}")));
                    if kept.len() > 4 * HANDOVERS_KEPT {
                        kept.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
                        kept.truncate(HANDOVERS_KEPT);
                    }
                }
                prev[key] = now;
            }
        }
    }
    kept.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    kept.truncate(HANDOVERS_KEPT);
    let secs = t0.elapsed().as_secs_f64();
    println!(
        "handovers: {events} events over {} frames; mean |jump| {:.6e}, worst {:.6e}; signed peak {:.6e}; DRIFT PEAK {:.6e} against the record's {:.6e}",
        rec.count,
        abs_sum / (events.max(1) as f64),
        worst,
        signed_peak,
        sim.drift_peak,
        rec.drift_peak
    );
    let geoms: Vec<String> = kept
        .iter()
        .map(|(j, six, where_)| {
            let atoms: Vec<String> = six.iter().map(|p| format!("[{}, {}, {}]", num(p[0]), num(p[1]), num(p[2]))).collect();
            format!("{{\"abs_jump_hartree\": {}, \"where\": {:?}, \"six\": [{}]}}", num(*j), where_, atoms.join(", "))
        })
        .collect();
    let r = Record::new("handovers")
        .text("what", "the RECORDED handover geometries of LIQUID-2's own 128-water 1x argmin arm, for the finite-difference gate")
        .text("law", law.arm.name())
        .text("law_source", &law.source)
        .text("table_source", &law.table_source)
        .text("screen_source", &rec.path)
        .text("atom_order", "[O_A, h_A0, h_A1, O_B, h_B0, h_B1], bohr, minimum-image relative to O_A")
        .int("settle_frames", rec.settle as i64)
        .int("counted_frames", rec.count as i64)
        .number("dt_tables_au", dt_tables)
        .int("handovers", events as i64)
        .number("mean_absolute_jump_hartree", abs_sum / (events.max(1) as f64))
        .number("worst_jump_hartree", worst)
        .number("sum_of_absolute_jumps_hartree", abs_sum)
        .number("signed_sum_hartree", signed)
        .number("signed_running_peak_hartree", signed_peak)
        .number("drift_peak", sim.drift_peak)
        .number("drift_peak_recorded", rec.drift_peak)
        // the record carries ten significant digits, so the comparison is made AT the record's
        // own precision — a tighter tolerance would be comparing against digits nobody wrote
        .flag("reproduces_the_recorded_arm", num(sim.drift_peak) == num(rec.drift_peak))
        .int("handovers_recorded", recorded_events as i64)
        .text(
            "handover_count_note",
            "the TRAJECTORY reproduces the recorded arm (the drift peak agrees at the record's own ten significant digits and the worst single jump agrees to every digit written), but this counter registers a slightly different NUMBER of handovers from LIQUID-2's own. Both are diagnostics on one identical trajectory; the difference is in the two counters, not in the arm, and it was not chased down. The gate that matters here needs the WORST geometries, and the worst jump is the same one.",
        )
        .number("seconds", secs)
        .int("geometries_kept", kept.len() as i64)
        .raw("geometries", format!("[{}]", geoms.join(", ")));
    w.write("handovers.json", &r).expect("the dump writes");
    w.done("handovers.done", "a screen, not a reading").expect("the marker writes");
}

/// The geometries the dump kept, read back.
fn read_handovers(out: &Path) -> Vec<(f64, Six, String)> {
    let p = out.join("smooth").join("handovers.json");
    let Ok(t) = std::fs::read_to_string(&p) else { return Vec::new() };
    let mut v = Vec::new();
    for chunk in t.split("{\"abs_jump_hartree\":").skip(1) {
        let j = chunk.split(',').next().and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(f64::NAN);
        let where_ = json_str(chunk, "where");
        let Some(body) = chunk.split("\"six\": [").nth(1) else { continue };
        let Some(end) = body.find("]]") else { continue };
        let mut six = [[0.0f64; 3]; 6];
        let mut k = 0;
        for c in body[..end + 1].split('[').skip(1) {
            let nums: Vec<f64> = c
                .trim_end_matches(|ch| ch == ']' || ch == ',' || ch == ' ')
                .split(',')
                .filter_map(|x| x.trim().parse::<f64>().ok())
                .collect();
            if nums.len() == 3 && k < 6 {
                six[k] = [nums[0], nums[1], nums[2]];
                k += 1;
            }
        }
        if k == 6 && j.is_finite() {
            v.push((j, six, where_));
        }
    }
    v
}

// ---------------------------------------------------------------------------- THE DRIFT GATE's arms

/// One arm of gate (d): LIQUID-2's screen protocol at the tables' step — the same box, seed,
/// settling, counted frames and rebasing — under the named law. The channel-6-off control is
/// RE-MEASURED here rather than cited, and its own record's number is carried beside it so a
/// box that is not LIQUID-2's box cannot pass unnoticed.
fn drift_phase(obs: &Path, out: &Path, arm: Arm) {
    let law = load_law(obs, arm);
    let rec = screen_record(obs, "x1_ct3-noct");
    let (mut sim, _l, dt_tables) = build(&law, 1.0);
    let dir = out.join("smooth");
    let label = format!("drift_{}", arm.name());
    let w = RecordWriter::screen(&dir, &label);
    let t0 = Instant::now();
    sim.rebase();
    let mut void: Option<String> = None;
    for k in 0..rec.settle {
        sim.step_frame(1);
        if void.is_none() && (!sim.pbc_ok() || sim.seam_work.units != N_WATERS as u64) {
            void = Some(format!("settle frame {k}: units {}, pbc_ok {}", sim.seam_work.units, sim.pbc_ok()));
        }
    }
    sim.rebase();
    let mut temp_sum = 0.0;
    for k in 0..rec.count {
        sim.step_frame(1);
        temp_sum += sim.temperature();
        if void.is_none() && (!sim.pbc_ok() || sim.seam_work.units != N_WATERS as u64) {
            void = Some(format!("counted frame {k}: units {}, pbc_ok {}", sim.seam_work.units, sim.pbc_ok()));
        }
    }
    let secs = t0.elapsed().as_secs_f64();
    println!(
        "drift {}: drift peak {:.9e}, thermostat {:.6e} Ha, T {:.1} K, {:.4} s/pass{}",
        arm.name(),
        sim.drift_peak,
        sim.work.thermostat,
        temp_sum / rec.count as f64,
        secs / rec.count as f64,
        match &void {
            Some(v) => format!(", VOID: {v}"),
            None => String::new(),
        }
    );
    let r = Record::new("drift")
        .text("what", "LIQUID-2's screen protocol at the tables' step, under one named serving rule")
        .text("law", law.arm.name())
        .text("serving_rule", if arm == Arm::Blend { "blend" } else { "argmin" })
        .text("law_source", &law.source)
        .text("protocol_source", &rec.path)
        .number("blend_beta_per_bohr", law.beta)
        .number("seam_switch_r_cut_bohr", law.r_cut)
        .int("settle_frames", rec.settle as i64)
        .int("counted_frames", rec.count as i64)
        .number("dt_tables_au", dt_tables)
        .number("step_au", sim.dt())
        .number("drift_peak", sim.drift_peak)
        .number("control_drift_peak_recorded", rec.drift_peak)
        .number("work_thermostat_hartree", sim.work.thermostat)
        .flag("columns_ok", sim.work_columns_ok())
        .number("momentum_residual", sim.momentum_residual())
        .number("momentum_bound", sim.momentum_bound())
        .number("mean_temperature_k", temp_sum / rec.count as f64)
        .int("units_at_end", sim.seam_work.units as i64)
        .number("seam_ho_pairs", sim.seam_work.ho_pairs as f64)
        .number("seconds", secs)
        .text("void", &void.clone().unwrap_or_else(|| "none".to_string()));
    w.write(&format!("{label}.json"), &r).expect("the arm writes");
    w.done(&format!("{label}.done"), "a screen, not a reading").expect("the marker writes");
}

// ------------------------------------------------------------------------------- the gates

fn gate_phase(obs: &Path, out: &Path) {
    let law = load_law(obs, Arm::Blend);
    let (nodes, floor, resolution) = load_map(obs);
    let b = derive_beta(obs);
    let dir = out.join("smooth");
    let w = RecordWriter::screen(&dir, "gate");
    let t0 = Instant::now();
    let table = &law.table;
    let model = &law.model;

    // --- (a) THE VALUE: the blend against the argmin at every node of the map -----------------
    let mut a_worst = 0.0f64;
    let mut a_at = String::new();
    let mut a_rows = Vec::new();
    for n in nodes.iter() {
        let blended = blend_pair(table, model, &n.six).0;
        let (argmin, k0) = argmin_pair(table, model, &n.six);
        let d = (blended - argmin).abs();
        if d > a_worst {
            a_worst = d;
            a_at = n.name.clone();
        }
        a_rows.push(format!(
            "{{\"node\": {:?}, \"blend_hartree\": {}, \"argmin_hartree\": {}, \"difference_hartree\": {}, \"argmin_contact\": {}, \"e_ct_hartree\": {}}}",
            n.name,
            num(blended),
            num(argmin),
            num(blended - argmin),
            k0,
            num(n.e_ct)
        ));
    }
    let g_a = Gate::new("a-value")
        .work(nodes.len() as u64)
        .leg_at(
            "the blend differs from the argmin by less than the table's own resolution floor at every node",
            a_worst < floor,
            a_worst,
        )
        .detail(format!("worst {a_worst:.6e} hartree at {a_at}, against the floor {floor:.6e}"));

    // --- (b) THE FORCE: central differences against the analytic gradient ----------------------
    // Every class the second review named, and the liquid's own worst cases among them.
    let handovers = read_handovers(out);
    let l_box = {
        let p = obs.join("liquid1").join("door.json");
        let t = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        json_num(&t, "cell_edge_bohr")
    };
    let mut classes: Vec<(String, Vec<Six>)> = Vec::new();
    classes.push(("the map's 64 nodes".to_string(), nodes.iter().map(|n| n.six).collect()));
    if !handovers.is_empty() {
        classes.push((
            "LIQUID-2's recorded handover geometries".to_string(),
            handovers.iter().map(|(_, s, _)| *s).collect(),
        ));
        let mut perm = Vec::new();
        let mut exch = Vec::new();
        for (_, s, _) in handovers.iter() {
            perm.push(swap_ha(s));
            perm.push(swap_hb(s));
            perm.push(swap_ha(&swap_hb(s)));
            exch.push(swap_units(s));
        }
        classes.push(("hydrogen permutations of those".to_string(), perm));
        classes.push(("molecule exchanges of those".to_string(), exch));
    }
    // contact ties: the map's own two exact ones, and one bisected on each linear node
    let mut ties: Vec<Six> = nodes.iter().filter(|n| n.name.ends_with("_b90")).map(|n| n.six).collect();
    let mut tie_residual = 0.0f64;
    for name in TURN_NODES.iter() {
        if let Some(n) = nodes.iter().find(|n| n.name == *name) {
            let (g, res) = tie_geometry(&n.six, 1.0, 179.0);
            tie_residual = tie_residual.max(res);
            ties.push(g);
        }
    }
    let n_ties = ties.len();
    classes.push(("contact ties".to_string(), ties));

    let mut b_worst = 0.0f64;
    let mut b_at = String::new();
    let mut b_trans = 0.0f64;
    let mut b_work = 0u64;
    let mut b_rows = Vec::new();
    for (name, set) in classes.iter() {
        let mut cw = 0.0f64;
        let mut ct = 0.0f64;
        for (i, g) in set.iter().enumerate() {
            let (worst, trans, ai, ac) = fd_worst(table, model, g, FD_H);
            b_work += 1;
            if worst > cw {
                cw = worst;
            }
            ct = ct.max(trans);
            if worst > b_worst {
                b_worst = worst;
                b_at = format!("{name} #{i}, atom {ai} coordinate {ac}");
            }
            b_trans = b_trans.max(trans);
        }
        b_rows.push(format!(
            "{{\"class\": {:?}, \"geometries\": {}, \"worst_relative\": {}, \"worst_translation_residual\": {}}}",
            name,
            set.len(),
            num(cw),
            num(ct)
        ));
    }
    // periodic crossings: the nodes and the handovers again, one box along each axis
    let mut p_worst = 0.0f64;
    let mut p_trans = 0.0f64;
    let mut p_work = 0u64;
    let mut wrap_identity = 0.0f64;
    let crossers: Vec<Six> = nodes.iter().map(|n| n.six).chain(handovers.iter().map(|(_, s, _)| *s)).collect();
    for g in crossers.iter() {
        for axis in 0..3 {
            let mut raw = *g;
            for i in 3..6 {
                raw[i][axis] += l_box;
            }
            let (worst, trans) = fd_worst_wrapped(table, model, &raw, l_box, FD_H);
            p_worst = p_worst.max(worst);
            p_trans = p_trans.max(trans);
            p_work += 1;
            // the wrapped pair IS the pair: the served energy must agree
            let e0 = blend_pair(table, model, g).0;
            let e1 = blend_pair(table, model, &wrap_pair(g, l_box, axis)).0;
            wrap_identity = wrap_identity.max(if e0 == 0.0 { (e1 - e0).abs() } else { ((e1 - e0) / e0).abs() });
        }
    }
    b_rows.push(format!(
        "{{\"class\": \"periodic crossings (one box along each axis, the wrap inside the difference loop)\", \"geometries\": {}, \"worst_relative\": {}, \"worst_translation_residual\": {}, \"worst_wrap_identity_relative\": {}}}",
        p_work,
        num(p_worst),
        num(p_trans),
        num(wrap_identity)
    ));
    let b_all = b_worst.max(p_worst);
    let b_trans_all = b_trans.max(p_trans);
    let g_b = Gate::new("b-force")
        .work(b_work + p_work)
        .leg_at(
            "worst relative |analytic - central difference| at h = 1e-5 over 6 atoms x 3 coordinates",
            b_all <= FD_TOL,
            b_all,
        )
        .leg_at("the forces sum to zero (translation invariance)", b_trans_all <= FD_TOL, b_trans_all)
        .leg_at("a pair read across a face is the same pair", wrap_identity <= FD_TOL, wrap_identity)
        .leg("the liquid's own recorded handover geometries were in the set", !handovers.is_empty())
        .detail(format!(
            "worst {b_all:.6e} at {b_at}; translation residual {b_trans_all:.6e}; {n_ties} contact ties, worst tie residual {tie_residual:.3e} bohr"
        ));

    // --- (c) THE CONTINUITY: CT-3's own G-A0 sweep, both rules, at TWO resolutions -------------
    // The raw largest step is not by itself a statement about continuity: between adjacent
    // samples of a SMOOTH function it is `O(dθ)` and it shrinks when the samples double, while
    // a JUMP is the same size however finely it is approached. Both resolutions are run and
    // the RATIO is the leg; the raw step is reported against the floor beside it, and so is
    // the argmin's own jump.
    let mut c_blend_step = 0.0f64;
    let mut c_blend_at = String::new();
    let mut c_blend_step_fine = 0.0f64;
    let mut c_argmin_step = 0.0f64;
    let mut c_argmin_jump = 0.0f64;
    let mut c_argmin_jump_fine = 0.0f64;
    let mut c_argmin_at = String::new();
    let mut c_handovers = 0u64;
    let mut c_work = 0u64;
    let mut c_rows = Vec::new();
    for name in TURN_NODES.iter() {
        let Some(n) = nodes.iter().find(|n| n.name == *name) else { continue };
        let coarse = turn_sweep(table, model, &n.six, TURN_STEPS);
        let fine = turn_sweep(table, model, &n.six, 2 * TURN_STEPS);
        c_work += (coarse.samples + fine.samples) as u64;
        c_handovers += coarse.handovers;
        if coarse.blend_step > c_blend_step {
            c_blend_step = coarse.blend_step;
            c_blend_at = format!("{name} at {:.2} degrees", coarse.blend_at);
        }
        c_blend_step_fine = c_blend_step_fine.max(fine.blend_step);
        c_argmin_step = c_argmin_step.max(coarse.argmin_step);
        if coarse.argmin_jump > c_argmin_jump {
            c_argmin_jump = coarse.argmin_jump;
            c_argmin_at = format!("{name} at {:.2} degrees", coarse.argmin_at);
        }
        c_argmin_jump_fine = c_argmin_jump_fine.max(fine.argmin_jump);
        c_rows.push(format!(
            "{{\"node\": {:?}, \"samples\": {}, \"blend_max_step_hartree\": {}, \"blend_max_step_refined_hartree\": {}, \"argmin_max_step_hartree\": {}, \"argmin_max_jump_hartree\": {}, \"argmin_max_jump_refined_hartree\": {}, \"handovers\": {}}}",
            name,
            coarse.samples,
            num(coarse.blend_step),
            num(fine.blend_step),
            num(coarse.argmin_step),
            num(coarse.argmin_jump),
            num(fine.argmin_jump),
            coarse.handovers
        ));
    }
    // a smooth sweep's largest step halves when the samples double; a jump does not move
    let c_ratio = c_blend_step / c_blend_step_fine.max(f64::MIN_POSITIVE);
    let c_argmin_ratio = c_argmin_jump / c_argmin_jump_fine.max(f64::MIN_POSITIVE);
    let g_c = Gate::new("c-continuity")
        .work(c_work)
        .leg_at(
            "the blended served energy's largest step across the sweep is under the table's resolution floor",
            c_blend_step < floor,
            c_blend_step,
        )
        .leg_at(
            "and it HALVES when the sweep's samples are doubled, which a discontinuity would not",
            c_ratio >= 1.8 && c_ratio <= 2.2,
            c_ratio,
        )
        .leg("the sweep found handovers to be continuous across (M-VACUOUS-SUCCESS)", c_handovers > 0)
        .detail(format!(
            "blend's largest step {c_blend_step:.6e} hartree at {c_blend_at}, against the floor {floor:.6e}; refined it is {c_blend_step_fine:.6e}, a ratio of {c_ratio:.4}. The argmin on the same sweep: largest step {c_argmin_step:.6e}, of which its largest JUMP is {c_argmin_jump:.6e} at {c_argmin_at} and refines to {c_argmin_jump_fine:.6e}, a ratio of {c_argmin_ratio:.4} — it does not shrink, which is what a discontinuity is. {c_handovers} handovers."
        ));

    // --- (d) THE DRIFT: the 128-water box, both arms measured in this session ------------------
    let read_arm = |label: &str| -> Option<(f64, String)> {
        let p = dir.join(format!("drift_{label}.json"));
        std::fs::read_to_string(&p).ok().map(|t| (json_num(&t, "drift_peak"), p.display().to_string()))
    };
    let blend_arm = read_arm("ct3-blend");
    let control_arm = read_arm("ct3-noct");
    let recorded = screen_record(obs, "x1_ct3-noct");
    let recorded_table_arm = screen_record(obs, "x1_ct3");
    let g_d = match (blend_arm, control_arm) {
        (Some((bd, bp)), Some((cd, cp))) => {
            let ratio = bd / cd;
            let reproduces = (cd - recorded.drift_peak).abs() <= 1e-12 * recorded.drift_peak.abs();
            Gate::new("d-drift")
                .work(2)
                .leg_at(
                    "the blended law's drift peak is within 2x the channel-6-off control measured on the same box in this session",
                    ratio <= 2.0,
                    ratio,
                )
                .leg("the control re-measured here reproduces LIQUID-2's own record bit for bit", reproduces)
                .detail(format!(
                    "blend {bd:.6e} Ha ({bp}); control {cd:.6e} Ha ({cp}) against the record's {:.6e}; ratio {ratio:.4}; the argmin arm's own recorded drift peak {:.6e}",
                    recorded.drift_peak, recorded_table_arm.drift_peak
                ))
        }
        _ => Gate::new("d-drift")
            .void("both arms must be run first: `ct3_smooth drift --law blend` and `ct3_smooth drift --law ct3-noct`"),
    };

    for g in [&g_a, &g_b, &g_c, &g_d] {
        println!("{}", g.line());
    }
    let admitted =
        g_a.verdict().admits() && g_b.verdict().admits() && g_c.verdict().admits() && g_d.verdict().admits();
    println!("\nADMITTED: {admitted}");

    let r = Record::new("gate")
        .text("what", "the smooth serving rule for CT-3's transfer table: its value, its force, its continuity and its drift")
        .text("law_source", &law.source)
        .text("table_source", &law.table_source)
        .text("beta_source", &dir.join("beta.json").display().to_string())
        .number("blend_beta_per_bohr", law.beta)
        .text("beta_binding_node", &b.beta_node)
        .number("resolution_floor_hartree", floor)
        .number("geometry_record_resolution_bohr", resolution)
        .number("finite_difference_h", FD_H)
        .number("finite_difference_tolerance", FD_TOL)
        .number("cell_edge_bohr", l_box)
        .gate(&g_a)
        .gate(&g_b)
        .gate(&g_c)
        .gate(&g_d)
        .raw("a_nodes", format!("[{}]", a_rows.join(", ")))
        .raw("b_classes", format!("[{}]", b_rows.join(", ")))
        .raw("c_sweep", format!("[{}]", c_rows.join(", ")))
        .number("c_blend_worst_step_hartree", c_blend_step)
        .number("c_blend_worst_step_refined_hartree", c_blend_step_fine)
        .number("c_blend_refinement_ratio", c_ratio)
        .number("c_argmin_worst_step_hartree", c_argmin_step)
        .number("c_argmin_worst_jump_hartree", c_argmin_jump)
        .number("c_argmin_worst_jump_refined_hartree", c_argmin_jump_fine)
        .number("c_argmin_refinement_ratio", c_argmin_ratio)
        .text("c_argmin_worst_at", &c_argmin_at)
        .int("c_handovers", c_handovers as i64)
        .number("contact_tie_worst_residual_bohr", tie_residual)
        .int("handover_geometries", handovers.len() as i64)
        .flag("admitted", admitted)
        .number("seconds", t0.elapsed().as_secs_f64());
    w.write("gate.json", &r).expect("the gate writes");
    w.done("gate.done", "a screen, not a reading").expect("the marker writes");
}

// ------------------------------------------------------------------------------- the step, NVE

/// ONE identical settled checkpoint, then NVE — thermostat OFF — at the named multiple of the
/// tables' step over the SAME physical duration. The sweep LIQUID-2 ran varied the duration
/// with the step, which the second review named; this one does not.
fn nve_phase(obs: &Path, out: &Path, step: f64, ps: f64) {
    let dir = out.join("smooth");
    if step <= 0.0 {
        return nve_collate(&dir, ps);
    }
    let law = load_law(obs, Arm::Blend);
    let rec = screen_record(obs, "x1_ct3-noct");
    // the settled state, reached at the TABLES' step under the thermostat, exactly as the
    // screen settles; the checkpoint's own digest is written so the four arms can be SHOWN to
    // have started from one state rather than four that resemble each other
    let (mut warm, l, dt_tables) = build(&law, 1.0);
    warm.rebase();
    for _ in 0..rec.settle {
        warm.step_frame(1);
    }
    let ckpt = warm.checkpoint();
    let digest = ckpt.digest();
    drop(warm);

    let (mut sim, _l, _dt) = build(&law, 1.0);
    sim.restore(&ckpt).expect("the settled checkpoint restores");
    // the step is set AFTER the restore, because the restore carries the checkpoint's own
    // `allow_dt_growth` and `dt`
    if step != 1.0 {
        sim.timescale.allow_dt_growth = true;
        sim.timescale.set_dt_multiplier(step * dt_tables / sim.timescale.dt_reference);
    }
    let want = dt_tables * step;
    assert!((sim.dt() - want).abs() <= 1e-12 * want, "the step did not take: {} against {want}", sim.dt());
    sim.thermostat_on = false;
    sim.rebase();

    let step_fs = sim.dt() * holon_lens::traj::AU_TIME_FS;
    let frames = ((ps * 1000.0) / step_fs).round() as usize;
    let label = format!("nve_x{}", step as u64);
    let w = RecordWriter::screen(&dir, &label);
    let t0 = Instant::now();
    let z: Vec<u32> = (0..sim.n).map(|i| sim.atoms[i].species.z).collect();
    let e0 = sim.ledger();
    let mut e_sum = 0.0f64;
    let mut e_sq = 0.0f64;
    let mut t_sum = 0.0f64;
    let mut rdf_sum: Vec<f64> = Vec::new();
    let mut rdf_r: Vec<f64> = Vec::new();
    let mut rdf_frames = 0usize;
    let mut bond_sum = 0.0f64;
    let mut bond_frames = 0usize;
    let mut void: Option<String> = None;
    // sampled on a stride that gives the same number of readouts at every step, so the
    // observables are compared over the same physical cadence and not the same frame count
    let stride = (frames / 200).max(1);
    for k in 0..frames {
        sim.step_frame(1);
        let e = sim.ledger() - e0;
        e_sum += e;
        e_sq += e * e;
        t_sum += sim.temperature();
        if void.is_none() && (!sim.pbc_ok() || sim.seam_work.units != N_WATERS as u64) {
            void = Some(format!("frame {k}: units {}, pbc_ok {}", sim.seam_work.units, sim.pbc_ok()));
        }
        if k % stride == 0 {
            let pos = positions(&sim);
            if let Ok(g) = rdf_oo(&pos, &z, [l, l, l], RDF_DR, 0.5 * l) {
                if rdf_sum.is_empty() {
                    rdf_sum = vec![0.0; g.g.len()];
                    rdf_r = g.r.clone();
                }
                for (i, v) in g.g.iter().enumerate() {
                    rdf_sum[i] += v;
                }
                rdf_frames += 1;
            }
            if let Ok(bonds) = hbonds_periodic(&pos, &z, [l, l, l]) {
                bond_sum += 2.0 * bonds.len() as f64 / N_WATERS as f64;
                bond_frames += 1;
            }
        }
    }
    let secs = t0.elapsed().as_secs_f64();
    let mean = e_sum / frames as f64;
    let rms = (e_sq / frames as f64 - mean * mean).max(0.0).sqrt();
    let ps_run = frames as f64 * step_fs / 1000.0;
    let drift_per_ps = sim.drift_peak / ps_run;
    let g_mean: Vec<f64> = rdf_sum.iter().map(|x| x / rdf_frames.max(1) as f64).collect();
    let peak = first_peak(&rdf_r, &g_mean);
    let bonds = bond_sum / bond_frames.max(1) as f64;
    println!(
        "nve x{}: dt {:.6} au ({:.6} fs), {frames} frames = {:.4} ps; fluctuation RMS {:.6e} Ha, drift peak {:.6e}, drift/ps {:.6e}; O-O peak {:?}; bonds/water {:.4}; T {:.1} K; {:.4} s/pass",
        step as u64,
        sim.dt(),
        step_fs,
        ps_run,
        rms,
        sim.drift_peak,
        drift_per_ps,
        peak,
        bonds,
        t_sum / frames as f64,
        secs / frames as f64
    );
    let r = Record::new("nve")
        .text(
            "what",
            "NVE from ONE settled checkpoint, thermostat OFF, at one multiple of the tables' step over a declared physical duration",
        )
        .text("law", law.arm.name())
        .text("serving_rule", "blend")
        .text("law_source", &law.source)
        .number("blend_beta_per_bohr", law.beta)
        .number("step_multiple_of_the_tables_step", step)
        .number("dt_tables_au", dt_tables)
        .number("step_au", sim.dt())
        .number("step_fs", step_fs)
        .number("requested_ps", ps)
        .number("physical_ps", ps_run)
        .int("frames", frames as i64)
        .int("settle_frames", rec.settle as i64)
        .int("checkpoint_digest_low", (digest & 0x7fff_ffff_ffff_ffff) as i64)
        .int("checkpoint_bytes", ckpt.len() as i64)
        .flag("thermostat_on", sim.thermostat_on)
        .number("energy_fluctuation_rms_hartree", rms)
        .number("energy_mean_offset_hartree", mean)
        .number("drift_peak", sim.drift_peak)
        .number("drift_per_ps", drift_per_ps)
        .number("mean_temperature_k", t_sum / frames as f64)
        .number("oo_first_peak_bohr", peak.map(|p| p.0).unwrap_or(f64::NAN))
        .number("oo_first_peak_height", peak.map(|p| p.1).unwrap_or(f64::NAN))
        .number("hbonds_per_water_both_ends", bonds)
        .int("rdf_samples", rdf_frames as i64)
        .flag("columns_ok", sim.work_columns_ok())
        .number("momentum_residual", sim.momentum_residual())
        .number("momentum_bound", sim.momentum_bound())
        .int("units_at_end", sim.seam_work.units as i64)
        .number("seconds", secs)
        .text("void", &void.clone().unwrap_or_else(|| "none".to_string()));
    w.write(&format!("{label}.json"), &r).expect("the arm writes");
    w.done(&format!("{label}.done"), "a screen, not a reading").expect("the marker writes");
}

/// THE RULE, APPLIED: the largest step whose drift per ps is within `2x` the `1x` run's and
/// whose observables agree with the `1x` run within their own spread across the sweep.
#[allow(clippy::type_complexity)]
fn nve_collate(dir: &Path, ps: f64) {
    let steps = [1u64, 2, 4, 8];
    let mut rows = Vec::new();
    let mut read: Vec<(u64, f64, f64, f64, f64, f64, f64, f64, String)> = Vec::new();
    for s in steps.iter() {
        let p = dir.join(format!("nve_x{s}.json"));
        let Ok(t) = std::fs::read_to_string(&p) else {
            println!("nve x{s}: not run ({})", p.display());
            continue;
        };
        read.push((
            *s,
            json_num(&t, "drift_per_ps"),
            json_num(&t, "energy_fluctuation_rms_hartree"),
            json_num(&t, "oo_first_peak_bohr"),
            json_num(&t, "hbonds_per_water_both_ends"),
            json_num(&t, "physical_ps"),
            json_num(&t, "checkpoint_digest_low"),
            json_num(&t, "mean_temperature_k"),
            p.display().to_string(),
        ));
    }
    if read.is_empty() {
        println!("no NVE arm has run");
        return;
    }
    let one = read.iter().find(|r| r.0 == 1).cloned();
    let digests_agree = read.iter().all(|r| r.6 == read[0].6);
    let durations_agree = read.iter().all(|r| (r.5 - read[0].5).abs() <= 1e-9 * read[0].5.abs().max(1.0));
    // the observables' own spread across the sweep is what "within their own spread" means:
    // no external tolerance is typed, the sweep supplies its own
    let peaks: Vec<f64> = read.iter().map(|r| r.3).filter(|x| x.is_finite()).collect();
    let bonds: Vec<f64> = read.iter().map(|r| r.4).filter(|x| x.is_finite()).collect();
    let spread = |v: &[f64]| -> f64 {
        if v.len() < 2 {
            return f64::NAN;
        }
        let m = v.iter().sum::<f64>() / v.len() as f64;
        (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (v.len() - 1) as f64).sqrt()
    };
    let peak_sd = spread(&peaks);
    let bond_sd = spread(&bonds);
    let mut chosen = 0u64;
    for r in read.iter() {
        let Some(o) = one.as_ref() else { break };
        let drift_ok = r.1.abs() <= 2.0 * o.1.abs();
        let peak_ok = !r.3.is_finite() || !o.3.is_finite() || (r.3 - o.3).abs() <= peak_sd.max(0.0);
        let bond_ok = !r.4.is_finite() || !o.4.is_finite() || (r.4 - o.4).abs() <= bond_sd.max(0.0);
        rows.push(format!(
            "{{\"step_multiple\": {}, \"physical_ps\": {}, \"drift_per_ps\": {}, \"energy_fluctuation_rms_hartree\": {}, \"oo_first_peak_bohr\": {}, \"hbonds_per_water_both_ends\": {}, \"mean_temperature_k\": {}, \"drift_within_2x_of_1x\": {}, \"oo_peak_within_spread\": {}, \"bonds_within_spread\": {}, \"source\": {:?}}}",
            r.0,
            num(r.5),
            num(r.1),
            num(r.2),
            num(r.3),
            num(r.4),
            num(r.7),
            drift_ok,
            peak_ok,
            bond_ok,
            r.8
        ));
        if drift_ok && peak_ok && bond_ok && r.0 > chosen {
            chosen = r.0;
        }
        println!(
            "  x{:<2} {:.4} ps  drift/ps {:.6e}  RMS {:.6e}  O-O peak {:.4}  bonds {:.4}  T {:.1}  [{}{}{}]",
            r.0,
            r.5,
            r.1,
            r.2,
            r.3,
            r.4,
            r.7,
            if drift_ok { "drift" } else { "DRIFT" },
            if peak_ok { " peak" } else { " PEAK" },
            if bond_ok { " bonds" } else { " BONDS" }
        );
    }
    println!("\n  the rule picks x{chosen}");
    let w = RecordWriter::screen(dir, "nve");
    let r = Record::new("nve")
        .text("what", "the step, chosen honestly: NVE from ONE settled checkpoint at four steps over EQUAL physical durations")
        .text(
            "rule",
            "the largest step whose drift per ps is within 2x the 1x run's and whose observables agree with the 1x run within their own spread across the sweep",
        )
        .number("requested_ps", ps)
        .flag("all_arms_started_from_one_checkpoint", digests_agree)
        .flag("equal_physical_durations", durations_agree)
        .number("oo_first_peak_spread_bohr", peak_sd)
        .number("hbonds_spread", bond_sd)
        .int("step_chosen", chosen as i64)
        .raw("arms", format!("[{}]", rows.join(", ")));
    w.write("nve.json", &r).expect("the collation writes");
    w.done("nve.done", "a screen, not a reading").expect("the marker writes");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let phase = args.first().cloned().unwrap_or_else(|| "gate".to_string());
    let val = |k: &str| -> Option<String> { args.iter().position(|a| a == k).and_then(|i| args.get(i + 1).cloned()) };
    let flags: Vec<usize> = args.iter().enumerate().filter(|(_, a)| a.starts_with("--")).map(|(i, _)| i + 1).collect();
    let out = PathBuf::from(
        args.iter()
            .enumerate()
            .skip(1)
            .find(|(i, a)| !a.starts_with("--") && !flags.contains(i))
            .map(|(_, a)| a.clone())
            .unwrap_or_else(|| "../conformance/water_observatory/ct3".to_string()),
    );
    let obs = observatory(&out);
    eprintln!("phase {phase}, out {}, observatory {}", out.display(), obs.display());
    match phase.as_str() {
        "beta" => beta_phase(&obs, &out),
        "handovers" => handovers_phase(&obs, &out),
        "drift" => drift_phase(&obs, &out, Arm::parse(&val("--law").unwrap_or_else(|| "blend".to_string()))),
        "gate" => gate_phase(&obs, &out),
        // `--step` absent (or non-positive) collates the arms that have run into `nve.json`
        "nve" => nve_phase(
            &obs,
            &out,
            val("--step").and_then(|v| v.parse().ok()).unwrap_or(0.0),
            val("--ps").and_then(|v| v.parse().ok()).unwrap_or(f64::NAN),
        ),
        other => panic!("unknown phase {other:?}: beta | handovers | drift | gate | nve"),
    }
}
