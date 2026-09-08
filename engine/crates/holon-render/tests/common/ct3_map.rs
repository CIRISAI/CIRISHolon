//! CT-3's map, its serving rules and the derivation of the blend's inverse length — ONE
//! definition, shared by `examples/ct3_smooth.rs` and `tests/ct3_smooth.rs` through `#[path]`
//! the way `field2_scenes.rs` is, so the runner and the gates cannot disagree about which
//! four contacts a pair has, which way round each one is, or where `β` came from.
//!
//! Nothing here writes a record and nothing here types a number the map's own records do not
//! carry. The paths are relative to the CRATE ROOT, which is where a test runs.
#![allow(dead_code)]

use holon_render::seam::{ct_pair_contacts, CtLoad, CtTable, SeamModel, CT_CONTACTS, CT_DIM};
use std::path::{Path, PathBuf};

/// CT-3's own G-B3 finite-difference step and the tolerance it was read against.
pub const FD_H: f64 = 1.0e-5;
pub const FD_TOL: f64 = 1.0e-8;

/// CT-3's G-A0 sweep: the donor turned through a full turn in this many steps, at the map's
/// own four linear nodes. Naming the NODES rather than typing four separations keeps the
/// sweep on recorded geometries and rebuilds no monomer here.
pub const TURN_STEPS: usize = 3600;
pub const TURN_NODES: [&str; 4] = ["linear_R2.7", "linear_R2.9", "linear_R3.1", "linear_R3.4"];

// ------------------------------------------------------------------------------ record reading

pub fn json_num(t: &str, key: &str) -> f64 {
    t.split(&format!("\"{key}\": "))
        .nth(1)
        .and_then(|x| x.split(|c| c == ',' || c == '\n' || c == '}').next())
        .and_then(|x| x.trim().parse::<f64>().ok())
        .unwrap_or(f64::NAN)
}

pub fn json_str(t: &str, key: &str) -> String {
    t.split(&format!("\"{key}\": \"")).nth(1).and_then(|x| x.split('"').next()).unwrap_or("").to_string()
}

/// `[[x, y, z], ...]` under `key`.
pub fn json_centers(t: &str, key: &str) -> Vec<[f64; 3]> {
    let Some(rest) = t.split(&format!("\"{key}\": [")).nth(1) else { return Vec::new() };
    let Some(end) = rest.find("]]") else { return Vec::new() };
    let mut out = Vec::new();
    for chunk in rest[..end + 1].split('[').skip(1) {
        let nums: Vec<f64> = chunk
            .trim_end_matches(|c| c == ']' || c == ',' || c == ' ')
            .split(',')
            .filter_map(|x| x.trim().parse::<f64>().ok())
            .collect();
        if nums.len() == 3 {
            out.push([nums[0], nums[1], nums[2]]);
        }
    }
    out
}

/// THE GEOMETRY RECORDS' OWN RESOLUTION, read off the record rather than declared: the number
/// of decimals the centres are printed to. A separation below it is not a separation, and it
/// is how the map's two EXACT ties — the donor bent 90°, both its hydrogens equidistant from
/// the acceptor's oxygen by the builder's own symmetry — are named as ties rather than treated
/// as `1e-12`-bohr separations no finite `β` could resolve.
pub fn centers_resolution(t: &str) -> f64 {
    let Some(rest) = t.split("\"donor_centers\": [[").nth(1) else { return f64::NAN };
    let first = rest.split(',').next().unwrap_or("");
    let d = first
        .trim()
        .split('.')
        .nth(1)
        .map(|x| x.trim_end_matches(|c: char| !c.is_ascii_digit()).len())
        .unwrap_or(0);
    10f64.powi(-(d as i32))
}

// ------------------------------------------------------------------------------ the table

/// The transfer table, built from `ct3/ct_table.json`'s own sites — `examples/liquid2.rs`'s
/// loader, unchanged, so no two runners build two tables.
pub fn load_table(p: &Path) -> (CtTable, usize) {
    let t = std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let c0 = json_num(&t, "c0_per_bohr");
    let body = t.split("\"sites\": [").nth(1).unwrap_or_else(|| panic!("{}: no sites", p.display()));
    let mut knots: Vec<([f64; CT_DIM], f64)> = Vec::new();
    for chunk in body.split("{\"site\":").skip(1) {
        let y = [json_num(chunk, "r"), json_num(chunk, "cos_theta_d"), json_num(chunk, "u_dot_b"), json_num(chunk, "q")];
        let v = json_num(chunk, "e_ct");
        if y.iter().all(|x| x.is_finite()) && v.is_finite() {
            knots.push((y, v));
        }
    }
    let mut table = CtTable::empty();
    assert!(table.begin(knots.len(), c0), "the table refused {} knots at c0 = {c0} ({:?})", knots.len(), table.status);
    for (i, (y, v)) in knots.iter().enumerate() {
        assert!(table.knot(i, *y, *v), "knot {i} refused");
    }
    assert_eq!(table.finish(), CtLoad::Ok, "the transfer table did not load: {:?}", table.status);
    (table, knots.len())
}

/// `β`, from the record the derivation wrote. Never typed and never defaulted.
pub fn read_beta(obs: &Path) -> f64 {
    let p = obs.join("ct3").join("smooth").join("beta.json");
    let t = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("{}: {e} — run `ct3_smooth beta` first; beta is derived, never typed", p.display()));
    let b = json_num(&t, "beta_per_bohr");
    assert!(b.is_finite() && b > 0.0, "{}: beta_per_bohr is {b}", p.display());
    b
}

/// CT-3's admitted law with LIQUID-1 Amendment 2's switch (read off `liquid1/door.json`) and
/// the SMOOTH serving rule installed at the derived `β`.
pub fn blended_law(obs: &Path) -> (SeamModel, CtTable, f64, f64) {
    let p = obs.join("ct3").join("wall_ct3.json");
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
    let (mut table, _) = load_table(&obs.join("ct3").join("ct_table.json"));
    let beta = read_beta(obs);
    assert!(table.set_blend(beta), "the table refused beta = {beta}");
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
        ct_table_on: true,
    };
    (model, table, beta, r_cut)
}

// ------------------------------------------------------------------------- one pair of units

/// A pair of water units as six atoms in ONE fixed order: `[O_A, h_A0, h_A1, O_B, h_B0, h_B1]`.
pub type Six = [[f64; 3]; 6];

/// Which six-atom slots each of the four contacts uses, in `ct_coords`' own five-atom order
/// `[H, O_a, O_d, h₁, h₂]` — the same four, in the same order, as `seam::ct_pair_contacts`.
pub const CONTACT_SLOTS: [[usize; 5]; CT_CONTACTS] =
    [[1, 3, 0, 4, 5], [2, 3, 0, 4, 5], [4, 0, 3, 1, 2], [5, 0, 3, 1, 2]];

pub fn contacts_of(x: &Six) -> [[[f64; 3]; 5]; CT_CONTACTS] {
    ct_pair_contacts(x[0], [x[1], x[2]], x[3], [x[4], x[5]])
}

/// The four contact separations, in `CONTACT_SLOTS`' order.
pub fn contact_r(x: &Six) -> [f64; CT_CONTACTS] {
    let c = contacts_of(x);
    let mut r = [0.0f64; CT_CONTACTS];
    for k in 0..CT_CONTACTS {
        let d = [c[k][0][0] - c[k][1][0], c[k][0][1] - c[k][1][1], c[k][0][2] - c[k][1][2]];
        r[k] = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    }
    r
}

/// THE BLENDED RULE on one pair, folded onto the six atoms: the energy and `∇E`. The engine's
/// own `CtTable::serve_blend` does the work; the fold is the same one `accumulate_seam` does.
pub fn blend_pair(table: &CtTable, model: &SeamModel, x: &Six) -> (f64, Six) {
    let (e, g) = table.serve_blend(model, &contacts_of(x));
    let mut out = [[0.0f64; 3]; 6];
    for (k, slots) in CONTACT_SLOTS.iter().enumerate() {
        for (i, &s) in slots.iter().enumerate() {
            for c in 0..3 {
                out[s][c] += g[k][i][c];
            }
        }
    }
    (e, out)
}

/// CT-3'S OWN ARGMIN RULE on one pair, served: the shortest contact, switched. Written so the
/// gates can put the two rules side by side on one geometry without running a box.
pub fn argmin_pair(table: &CtTable, model: &SeamModel, x: &Six) -> (f64, usize) {
    let r = contact_r(x);
    let mut k0 = 0;
    for k in 1..CT_CONTACTS {
        if r[k] < r[k0] {
            k0 = k;
        }
    }
    let (sw, _) = model.switch(r[k0]);
    if sw == 0.0 {
        return (0.0, k0);
    }
    let c = contacts_of(x)[k0];
    (sw * table.serve(c[0], c[1], c[2], c[3], c[4]).0, k0)
}

/// The finite-difference check on one geometry: the worst RELATIVE disagreement between the
/// analytic gradient and a central difference at `h` over six atoms and three coordinates,
/// with the residual of the force sum (translation invariance).
pub fn fd_worst(table: &CtTable, model: &SeamModel, x: &Six, h: f64) -> (f64, f64, usize, usize) {
    let (_, g) = blend_pair(table, model, x);
    let mut worst = 0.0f64;
    let (mut wi, mut wc) = (0usize, 0usize);
    for i in 0..6 {
        for c in 0..3 {
            let mut xp = *x;
            let mut xm = *x;
            xp[i][c] += h;
            xm[i][c] -= h;
            let ep = blend_pair(table, model, &xp).0;
            let em = blend_pair(table, model, &xm).0;
            let num_g = (ep - em) / (2.0 * h);
            let rel = (g[i][c] - num_g).abs() / g[i][c].abs().max(num_g.abs()).max(1.0);
            if rel > worst {
                worst = rel;
                wi = i;
                wc = c;
            }
        }
    }
    let mut sum = [0.0f64; 3];
    for i in 0..6 {
        for c in 0..3 {
            sum[c] += g[i][c];
        }
    }
    (worst, (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]).sqrt(), wi, wc)
}

/// Minimum-image displacement from `a` to `b` in a cube of edge `l`.
pub fn mind(a: [f64; 3], b: [f64; 3], l: f64) -> [f64; 3] {
    let mut d = [0.0f64; 3];
    for c in 0..3 {
        let mut x = b[c] - a[c];
        x -= l * (x / l).round();
        d[c] = x;
    }
    d
}

/// The same six atoms with unit A's two hydrogens exchanged.
pub fn swap_ha(x: &Six) -> Six {
    [x[0], x[2], x[1], x[3], x[4], x[5]]
}
/// The same six atoms with unit B's two hydrogens exchanged.
pub fn swap_hb(x: &Six) -> Six {
    [x[0], x[1], x[2], x[3], x[5], x[4]]
}
/// The same pair with the two MOLECULES exchanged — the unordered pair, relabelled.
pub fn swap_units(x: &Six) -> Six {
    [x[3], x[4], x[5], x[0], x[1], x[2]]
}

/// The pair with unit B pushed one box along an axis and the whole thing re-read under the
/// minimum image about unit A's oxygen. The wrapped geometry IS the pair; a rule that
/// depended on where the box edge fell would say otherwise.
pub fn wrap_pair(x: &Six, l: f64, axis: usize) -> Six {
    let mut raw = *x;
    for i in 3..6 {
        raw[i][axis] += l;
    }
    let p0 = raw[0];
    let mut out = [[0.0f64; 3]; 6];
    for i in 0..6 {
        out[i] = mind(p0, raw[i], l);
    }
    out
}

/// The finite-difference check on a geometry that crosses a face: the perturbation is applied
/// to the RAW coordinates and the minimum image is re-taken, so the wrap is inside the loop.
pub fn fd_worst_wrapped(table: &CtTable, model: &SeamModel, raw: &Six, l: f64, h: f64) -> (f64, f64) {
    let wrap = |r: &Six| -> Six {
        let p0 = r[0];
        let mut out = [[0.0f64; 3]; 6];
        for i in 0..6 {
            out[i] = mind(p0, r[i], l);
        }
        out
    };
    let (_, g) = blend_pair(table, model, &wrap(raw));
    let mut worst = 0.0f64;
    for i in 0..6 {
        for c in 0..3 {
            let mut xp = *raw;
            let mut xm = *raw;
            xp[i][c] += h;
            xm[i][c] -= h;
            let num_g = (blend_pair(table, model, &wrap(&xp)).0 - blend_pair(table, model, &wrap(&xm)).0) / (2.0 * h);
            worst = worst.max((g[i][c] - num_g).abs() / g[i][c].abs().max(num_g.abs()).max(1.0));
        }
    }
    let mut sum = [0.0f64; 3];
    for i in 0..6 {
        for c in 0..3 {
            sum[c] += g[i][c];
        }
    }
    (worst, (sum[0] * sum[0] + sum[1] * sum[1] + sum[2] * sum[2]).sqrt())
}

/// FIELD-5's rotation on six atoms: unit A turned about the x-axis through ITS OWN oxygen.
/// The map's `linear_*` nodes put that oxygen at the origin and the acceptor along `z`, which
/// is exactly the frame `ct3_harvest::rot_x` turns the donor in.
pub fn turn_a(x: &Six, degrees: f64) -> Six {
    let th = degrees * std::f64::consts::PI / 180.0;
    let (s, c) = (th.sin(), th.cos());
    let o = x[0];
    let mut out = *x;
    for i in 0..3 {
        let (px, py, pz) = (x[i][0] - o[0], x[i][1] - o[1], x[i][2] - o[2]);
        out[i] = [o[0] + px, o[1] + py * c - pz * s, o[2] + py * s + pz * c];
    }
    out
}

/// A CONSTRUCTED CONTACT TIE: the donor turned about its own oxygen until its two hydrogens
/// are equidistant from the acceptor's — the geometry at which the argmin is a coin flip.
/// Bisected on the turn angle, and the residual `|r₀ − r₁|` is returned, so the tie is a
/// measured tie and not a claimed one.
pub fn tie_geometry(x: &Six, lo_deg: f64, hi_deg: f64) -> (Six, f64) {
    let gap = |d: f64| -> f64 {
        let r = contact_r(&turn_a(x, d));
        r[0] - r[1]
    };
    let (mut a, mut b) = (lo_deg, hi_deg);
    let fa = gap(a);
    if fa * gap(b) > 0.0 {
        let g = turn_a(x, a);
        let r = contact_r(&g);
        return (g, (r[0] - r[1]).abs());
    }
    for _ in 0..200 {
        let m = 0.5 * (a + b);
        if gap(m) * fa > 0.0 {
            a = m;
        } else {
            b = m;
        }
    }
    let g = turn_a(x, 0.5 * (a + b));
    let r = contact_r(&g);
    (g, (r[0] - r[1]).abs())
}

// --------------------------------------------------------------- CT-3's G-A0 sweep, both rules

/// What one full turn of the donor reads, under both serving rules.
pub struct SweepReading {
    /// The largest step of the BLENDED served energy between adjacent samples.
    pub blend_step: f64,
    pub blend_at: f64,
    /// The largest step of the ARGMIN's served energy between adjacent samples — the same
    /// statistic, so the two rules are read side by side and not one against the other's.
    pub argmin_step: f64,
    /// The largest of those steps that lands ON a handover: the argmin's discontinuity.
    pub argmin_jump: f64,
    pub argmin_at: f64,
    pub handovers: u64,
    pub samples: usize,
}

/// CT-3's G-A0 SWEEP: the donor turned about its own oxygen through a full turn in `steps`
/// samples, with both serving rules read at every one.
///
/// **A DISCONTINUITY DOES NOT SHRINK WHEN THE SWEEP IS REFINED, AND A SMOOTH VARIATION DOES.**
/// That is why this takes the sample count as an argument: the largest step of a continuous
/// function between adjacent samples is `O(dθ)` and halves when the samples double, while a
/// jump is the same size however finely it is approached. The ratio between two resolutions is
/// therefore the measurement of continuity, and the raw step at one resolution is not.
pub fn turn_sweep(table: &CtTable, model: &SeamModel, six: &Six, steps: usize) -> SweepReading {
    let mut r = SweepReading {
        blend_step: 0.0,
        blend_at: 0.0,
        argmin_step: 0.0,
        argmin_jump: 0.0,
        argmin_at: 0.0,
        handovers: 0,
        samples: steps + 1,
    };
    let mut prev: Option<(f64, f64, usize)> = None;
    for k in 0..=steps {
        let deg = 360.0 * (k as f64) / (steps as f64);
        let g = turn_a(six, deg);
        let eb = blend_pair(table, model, &g).0;
        let (ea, k0) = argmin_pair(table, model, &g);
        if let Some((pb, pa, pk)) = prev {
            let db = (eb - pb).abs();
            if db > r.blend_step {
                r.blend_step = db;
                r.blend_at = deg;
            }
            let da = (ea - pa).abs();
            r.argmin_step = r.argmin_step.max(da);
            if pk != k0 {
                r.handovers += 1;
                if da > r.argmin_jump {
                    r.argmin_jump = da;
                    r.argmin_at = deg;
                }
            }
        }
        prev = Some((eb, ea, k0));
    }
    r
}

// ------------------------------------------------------------------------------- the map

pub struct MapNode {
    pub name: String,
    /// The six atoms, the record's own donor first and its own acceptor second, verbatim.
    pub six: Six,
    pub e_ct: f64,
}

/// The sixty-four nodes, REBUILT from the map's own records the way `ct3_harvest::load_map`
/// does: `ct2/node_*.json` and `ct2/gd0_*.json` carry the centres, `ct3/ct_table.json` carries
/// each node's `E_CT` through the site it belongs to and, as its largest pole spread, the
/// table's own resolution floor.
pub fn load_map(obs: &Path) -> (Vec<MapNode>, f64, f64) {
    let ct2 = obs.join("ct2");
    let tp = obs.join("ct3").join("ct_table.json");
    let tt = std::fs::read_to_string(&tp).unwrap_or_else(|e| panic!("{}: {e}", tp.display()));
    let mut floor = 0.0f64;
    let mut value: Vec<(String, f64)> = Vec::new();
    for chunk in tt.split("{\"site\":").skip(1) {
        let e = json_num(chunk, "e_ct");
        let s = json_num(chunk, "spread");
        if s.is_finite() && s > floor {
            floor = s;
        }
        let members = chunk.split("\"members\": [").nth(1).and_then(|x| x.split(']').next()).unwrap_or("");
        for m in members.split(',') {
            let name = m.trim().trim_matches('"').to_string();
            if !name.is_empty() {
                value.push((name, e));
            }
        }
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(&ct2)
        .unwrap_or_else(|e| panic!("{}: {e}", ct2.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let n = p.file_name().and_then(|x| x.to_str()).unwrap_or("");
            (n.starts_with("node_") || n.starts_with("gd0_")) && n.ends_with(".json")
        })
        .collect();
    files.sort();
    let mut nodes = Vec::new();
    let mut resolution = f64::NAN;
    for f in files {
        let t = std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        let name = json_str(&t, "node");
        let dc = json_centers(&t, "donor_centers");
        let ac = json_centers(&t, "acceptor_centers");
        assert_eq!(dc.len(), 3, "{name}: a donor is an oxygen and two hydrogens");
        assert_eq!(ac.len(), 3, "{name}: an acceptor is an oxygen and two hydrogens");
        let r = centers_resolution(&t);
        if resolution.is_nan() || r < resolution {
            resolution = r;
        }
        let e_ct = value
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, e)| *e)
            .unwrap_or_else(|| panic!("{name}: no site in {}", tp.display()));
        nodes.push(MapNode { name, six: [dc[0], dc[1], dc[2], ac[0], ac[1], ac[2]], e_ct });
    }
    (nodes, floor, resolution)
}

// -------------------------------------------------------------------------- the derivation of beta

pub struct BetaRow {
    pub name: String,
    pub r1: f64,
    pub r2: f64,
    pub dr: f64,
    pub e_ct: f64,
    pub fraction: f64,
    pub required: f64,
    pub tie: bool,
    pub vacuous: bool,
}

pub struct Beta {
    pub floor: f64,
    pub resolution: f64,
    pub rows: Vec<BetaRow>,
    /// The smallest separation that is a separation at all (above the records' own resolution).
    pub dr_min: f64,
    pub dr_min_node: String,
    /// What the literal recipe gives from `dr_min` and THAT node's own `|E_CT|`.
    pub beta_at_dr_min: f64,
    /// What is adopted: the largest requirement over every node where the criterion has
    /// content, so it holds at EVERY node and not only at the closest one.
    pub beta: f64,
    pub beta_node: String,
    pub ties: Vec<String>,
    pub vacuous: Vec<String>,
}

/// `β` FROM THE MAP'S OWN RECORDS. At each of the sixty-four nodes take the shortest and the
/// second-shortest cross-unit H···O contact. Over four contacts the second one's weight is at
/// most `1/(1 + e^{βΔr})`; requiring it below the table's own resolution floor as a fraction
/// of that node's `|E_CT|` gives `β > ln(|E_CT|/floor − 1)/Δr`. The criterion has content only
/// where that fraction is under a half — no positive `β` is needed to put the second of four
/// weights under a half — and no finite `β` satisfies it at a TIE, where the argmin is a coin
/// flip and the difference has to be measured instead.
pub fn derive_beta(obs: &Path) -> Beta {
    let (nodes, floor, resolution) = load_map(obs);
    assert_eq!(nodes.len(), 64, "the map is sixty-four nodes; {} were read", nodes.len());
    assert!(floor > 0.0, "the table's resolution floor is its largest pole spread and it is {floor}");
    let mut rows: Vec<BetaRow> = Vec::new();
    for n in nodes.iter() {
        let mut r = contact_r(&n.six).to_vec();
        r.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let dr = r[1] - r[0];
        let fraction = floor / n.e_ct.abs();
        let tie = dr <= resolution;
        let vacuous = !(fraction < 0.5);
        let required = if tie || vacuous { f64::NAN } else { (1.0 / fraction - 1.0).ln() / dr };
        rows.push(BetaRow { name: n.name.clone(), r1: r[0], r2: r[1], dr, e_ct: n.e_ct, fraction, required, tie, vacuous });
    }
    rows.sort_by(|a, b| a.dr.partial_cmp(&b.dr).unwrap());
    let first = rows.iter().find(|r| !r.tie).expect("some node has two distinguishable contacts");
    let dr_min = first.dr;
    let dr_min_node = first.name.clone();
    let beta_at_dr_min = if first.vacuous { f64::NAN } else { (1.0 / first.fraction - 1.0).ln() / first.dr };
    let mut beta = 0.0f64;
    let mut beta_node = String::new();
    for r in rows.iter() {
        if r.required.is_finite() && r.required > beta {
            beta = r.required;
            beta_node = r.name.clone();
        }
    }
    let ties: Vec<String> = rows.iter().filter(|r| r.tie).map(|r| r.name.clone()).collect();
    let vacuous: Vec<String> = rows.iter().filter(|r| r.vacuous && !r.tie).map(|r| r.name.clone()).collect();
    Beta { floor, resolution, rows, dr_min, dr_min_node, beta_at_dr_min, beta, beta_node, ties, vacuous }
}
