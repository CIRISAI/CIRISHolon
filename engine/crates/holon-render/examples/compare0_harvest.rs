//! COMPARE-0's harvest (`conformance/water_observatory/COMPARE0_PREREG.md`): THE SERVED LAW,
//! PRICED AGAINST REFERENCES.
//!
//! CT-3 served charge transfer as a table and the whole law passed its boundedness gate. What no
//! record yet carries is the served law's TOTAL interaction energy, node by node, beside the
//! exact one — the C1 fit reported a residual and a within-count, never the totals. This harvest
//! writes them, with the channel decomposition and the analytic interaction FORCE on all six
//! atoms, for every geometry the programme has solved exactly: CT-2's sixty-four-node map and
//! CT-3's held-out node. Nothing here solves anything new; every exact number is read from a
//! record and cited by its path.
//!
//! ```text
//! cargo run --release -p holon-render --example compare0_harvest -- served [OUT_DIR]
//! ```
//!
//! The reference models (MB-pol, TIP4P/2005) are scored by the Python beside this file, on the
//! geometries this phase writes, so that no reference number enters the engine's tree.
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::{water_dimer_linear, Fragment, ANGSTROM_TO_BOHR};
use holon_render::seam::{ct_coords, CtLoad, CtTable, SeamModel, SeamPlant, CT_DIM};
use holon_render::sim::{Boundary, Dims, Sim};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[path = "../tests/common/quartet.rs"]
#[allow(dead_code)]
mod quartet;

// ------------------------------------------------------------------------------- the constants

/// EMBED-1's water pins — the same numbers FIELD-3's … CT-3's runners carry.
const H2O_R: f64 = 1.9435738400;
const H2O_THETA: f64 = 1.6887434037;

/// CT-2's map, corrected before its freeze: sixty-four DISTINCT geometries.
const MAP_NODES: usize = 64;

/// CT-3's pole rule, carried verbatim: two knots closer than this in the SCALED box are one site.
const POLE_MERGE: f64 = 1e-9;

/// The engine's reference on both sides of every difference (bohr): the acceptor moved away.
const FAR_BOHR: f64 = 40.0;

/// G-T0's bar, carried: the interpolant reproduces the knot it was built on.
const KNOT_TOL: f64 = 1e-12;

/// G-C1's bar, carried: the engine's own `e_seam` against this runner's independent evaluation.
const G_C1_TOL: f64 = 1e-10;

/// G-F0: the analytic interaction force against a central difference of the interaction energy.
/// The bar is ABSOLUTE (hartree/bohr) because a force component can pass through zero and a
/// relative reading explodes there; the relative reading is kept beside it, scaled by the
/// node's OWN largest component rather than by the component being differenced.
const FD_H: f64 = 1e-5;
const FD_TOL: f64 = 1e-8;

/// G-X0: this runner's served numbers against CT-3's own record for the held-out node.
const X0_TOL: f64 = 1e-12;

/// CT-3's held-out node, by name and by its freeze's own construction.
const HO_ANGSTROM: f64 = 2.7;
const HO_TWIST_DEGREES: f64 = 20.0;
const HO_TILT_DEGREES: f64 = 85.0;
const HO_DONOR_DEGREES: f64 = 0.0;
const HO_NAME: &str = "twistbent_R2.7_tw20_t85_d0";

// ----------------------------------------------------------------------------------- plumbing

fn cpu_seconds() -> f64 {
    let s = fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let tail = &s[s.rfind(')').map(|i| i + 2).unwrap_or(0)..];
    let f: Vec<&str> = tail.split_whitespace().collect();
    let ut: f64 = f.get(11).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    let st: f64 = f.get(12).and_then(|x| x.parse().ok()).unwrap_or(0.0);
    (ut + st) / 100.0
}

fn json_num(t: &str, key: &str) -> f64 {
    t.split(&format!("\"{key}\": ")).nth(1).and_then(|x| x.split(|c| c == ',' || c == '\n' || c == '}').next()).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(f64::NAN)
}

fn json_str(t: &str, key: &str) -> String {
    t.split(&format!("\"{key}\": \"")).nth(1).and_then(|x| x.split('"').next()).unwrap_or("").to_string()
}

fn json_centers(t: &str, key: &str) -> Vec<[f64; 3]> {
    let Some(rest) = t.split(&format!("\"{key}\": [")).nth(1) else { return Vec::new() };
    let Some(end) = rest.find("]]") else { return Vec::new() };
    let nums: Vec<f64> = rest[..end + 1].split(|c: char| c == '[' || c == ']' || c == ',').filter_map(|x| x.trim().parse::<f64>().ok()).collect();
    nums.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect()
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// ONE JSON NUMBER, valid whatever the arithmetic did (M-FORMAT-FLOOR): twelve digits, never a
/// leading `+`, and `null` rather than `NaN`/`inf`.
fn jn(x: f64) -> String {
    if x.is_finite() { format!("{x:.12e}") } else { "null".to_string() }
}

fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn sibling(out: &Path, name: &str) -> PathBuf {
    let sib = out.parent().unwrap_or(Path::new(".")).join(name);
    if sib.exists() { sib } else { PathBuf::from(format!("../conformance/water_observatory/{name}")) }
}

// ------------------------------------------------------------------------------ the geometries

/// FIELD-3's `linear` verbatim.
fn linear(o: Species, h: Species, r_oo_angstrom: f64) -> (Fragment, Fragment) {
    water_dimer_linear(o, h, H2O_R, H2O_THETA, r_oo_angstrom * ANGSTROM_TO_BOHR)
}

/// FIELD-5's rotation: a fragment turned about the x-axis through ITS OWN oxygen.
fn rot_x(f: &Fragment, theta_degrees: f64) -> Fragment {
    let oi = f.species.iter().position(|s| s.z == 8).expect("an oxygen");
    let origin = f.centers[oi];
    let th = theta_degrees * std::f64::consts::PI / 180.0;
    let (s, c) = (th.sin(), th.cos());
    let centers: Vec<[f64; 3]> = f
        .centers
        .iter()
        .map(|p| {
            let (x, y, z) = (p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]);
            [origin[0] + x, origin[1] + y * c - z * s, origin[2] + y * s + z * c]
        })
        .collect();
    Fragment::new(f.species.clone(), centers, f.weights.clone())
}

/// CT-2's `twist_and_bend` verbatim.
fn twist_and_bend(o: Species, h: Species, r_oo_angstrom: f64, twist_degrees: f64, tilt_degrees: f64, donor_degrees: f64) -> (Fragment, Fragment) {
    let (donor, acc) = linear(o, h, r_oo_angstrom);
    let oi = acc.species.iter().position(|s| s.z == 8).expect("an acceptor oxygen");
    let origin = acc.centers[oi];
    let tw = twist_degrees * std::f64::consts::PI / 180.0;
    let ti = tilt_degrees * std::f64::consts::PI / 180.0;
    let (sw, cw) = (tw.sin(), tw.cos());
    let (st, ct) = (ti.sin(), ti.cos());
    let centers: Vec<[f64; 3]> = acc
        .centers
        .iter()
        .map(|p| {
            let (x, y, z) = (p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]);
            let (x1, y1, z1) = (x * cw - y * sw, x * sw + y * cw, z);
            let (x2, y2, z2) = (x1, y1 * ct - z1 * st, y1 * st + z1 * ct);
            [origin[0] + x2, origin[1] + y2, origin[2] + z2]
        })
        .collect();
    (rot_x(&donor, donor_degrees), Fragment::new(acc.species.clone(), centers, acc.weights.clone()))
}

fn oxygen_of(f: &Fragment) -> usize {
    f.species.iter().position(|s| s.z == 8).expect("an oxygen")
}

fn hydrogens_of(f: &Fragment) -> [usize; 2] {
    let hs: Vec<usize> = (0..f.species.len()).filter(|&i| f.species[i].z == 1).collect();
    assert_eq!(hs.len(), 2, "a water unit is an oxygen with exactly two hydrogens");
    [hs[0], hs[1]]
}

/// Every CROSS-UNIT pair distance (bohr) by class: `(O–O, H–O, H–H)`.
fn cross_classes(a: &Fragment, b: &Fragment) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let (mut oo, mut ho, mut hh) = (Vec::new(), Vec::new(), Vec::new());
    for (sa, ca) in a.species.iter().zip(a.centers.iter()) {
        for (sb, cb) in b.species.iter().zip(b.centers.iter()) {
            let d = dist(ca, cb);
            match (sa.z, sb.z) {
                (8, 8) => oo.push(d),
                (8, 1) | (1, 8) => ho.push(d),
                (1, 1) => hh.push(d),
                _ => {}
            }
        }
    }
    (oo, ho, hh)
}

/// CT-3's serving rule, carried: the contact of an ORDERED pair is the donor's hydrogen nearest
/// the acceptor's oxygen.
fn contact_of(donor: &Fragment, acceptor: &Fragment) -> ([f64; CT_DIM], [[f64; 3]; 5]) {
    let od = donor.centers[oxygen_of(donor)];
    let oa = acceptor.centers[oxygen_of(acceptor)];
    let [a1, a2] = hydrogens_of(acceptor);
    let [d1, d2] = hydrogens_of(donor);
    let hi = if dist(&donor.centers[d1], &oa) <= dist(&donor.centers[d2], &oa) { d1 } else { d2 };
    let pts = [donor.centers[hi], oa, od, acceptor.centers[a1], acceptor.centers[a2]];
    (ct_coords(pts[0], pts[1], pts[2], pts[3], pts[4]).0, pts)
}

fn node_contact(a: &Fragment, b: &Fragment) -> ([f64; CT_DIM], [[f64; 3]; 5]) {
    let (ya, pa) = contact_of(a, b);
    let (yb, pb) = contact_of(b, a);
    if ya[0] <= yb[0] { (ya, pa) } else { (yb, pb) }
}

// ---------------------------------------------------------------------------------- the engine

/// CT-3's `engine_dimer` verbatim.
fn engine_dimer(a: &Fragment, b: &Fragment, seam: Option<SeamModel>, plant: SeamPlant, table: Option<&CtTable>) -> Box<Sim> {
    let mut species = a.species.clone();
    species.extend_from_slice(&b.species);
    let pos: Vec<[f64; 3]> = a.centers.iter().chain(b.centers.iter()).map(|c| [c[0] + 15.0, c[1] + 15.0, c[2] + 10.0]).collect();
    let mut s = quartet::scene(&species, &pos, false);
    s.dims = Dims::Three;
    s.boundary = Boundary::Open;
    s.width = 80.0;
    s.height = 30.0;
    s.depth = 30.0;
    s.sync_species();
    s.adopt_table_timescale();
    s.rebase();
    s.set_field(true, None).expect("open box admits the field");
    if let Some(t) = table {
        s.ct_table = t.clone();
    }
    s.seam_plant = plant;
    s.set_seam(seam).expect("no acuity frame");
    s.refresh_pairs();
    s.compute_forces();
    s
}

/// CT-3's `engine_interaction` verbatim: `E(geometry) − E(acceptor moved 40 bohr along x)`.
fn engine_interaction(a: &Fragment, b: &Fragment, seam: Option<SeamModel>, plant: SeamPlant, table: Option<&CtTable>) -> (f64, f64, f64) {
    let s = engine_dimer(a, b, seam, plant, table);
    let near = (s.e_pair + s.e_three) + s.e_field + s.e_seam;
    let far_b = b.translated([FAR_BOHR, 0.0, 0.0]);
    let f = engine_dimer(a, &far_b, seam, plant, table);
    let far = (f.e_pair + f.e_three) + f.e_field + f.e_seam;
    (near - far, s.e_field - f.e_field, s.e_seam - f.e_seam)
}

/// THE INTERACTION FORCE, on all six atoms (hartree/bohr): the analytic internal force at the
/// geometry MINUS the analytic internal force with the acceptor moved 40 bohr away. The far
/// configuration is the near one with one fragment rigidly translated, so every intramolecular
/// term is bit-identical on both sides and the difference is the cross-unit force alone — the
/// same subtraction `engine_interaction` makes on the energy.
fn engine_interaction_force(a: &Fragment, b: &Fragment, seam: Option<SeamModel>, table: Option<&CtTable>) -> [[f64; 3]; 6] {
    let s = engine_dimer(a, b, seam, SeamPlant::None, table);
    let far_b = b.translated([FAR_BOHR, 0.0, 0.0]);
    let f = engine_dimer(a, &far_b, seam, SeamPlant::None, table);
    let mut out = [[0.0; 3]; 6];
    for i in 0..6 {
        let (nx, ny, nz) = s.internal_force(i);
        let (fx, fy, fz) = f.internal_force(i);
        out[i] = [nx - fx, ny - fy, nz - fz];
    }
    out
}

/// A fragment with one atom moved (for the finite-difference gate).
fn moved(f: &Fragment, i: usize, c: usize, h: f64) -> Fragment {
    let mut centers = f.centers.clone();
    centers[i][c] += h;
    Fragment::new(f.species.clone(), centers, f.weights.clone())
}

// ----------------------------------------------------------------------------- the law of record

struct Law {
    m: SeamModel,
    c0: f64,
    p_ct: f64,
    m_ct: u8,
    source: String,
}

/// CT-3's SERVED law: FIELD-9's three walls and the charge from `ct2/wall_ct2.json`, the two
/// contact terms from `ct3/wall_ct3.json` (C1's re-fit under the served boundedness walk), the
/// transfer from the table. This is `assemble`'s law in `ct3_harvest.rs`, read the same way.
fn load_law(out: &Path) -> Law {
    let p2 = sibling(out, "ct2/wall_ct2.json");
    let t2 = fs::read_to_string(&p2).unwrap_or_else(|e| panic!("{}: {e}", p2.display()));
    let p3 = sibling(out, "ct3/wall_ct3.json");
    let t3 = fs::read_to_string(&p3).unwrap_or_else(|e| panic!("{}: {e}", p3.display()));
    let g = |k: &str| json_num(&t2, k);
    Law {
        m: SeamModel {
            a: g("a"),
            b: g("b"),
            p: json_num(&t3, "p"),
            c: json_num(&t3, "c"),
            c6: g("c6"),
            a_oh: g("a_oh"),
            b_oh: g("b_oh"),
            a_hh: g("a_hh"),
            b_hh: g("b_hh"),
            p_hh: json_num(&t3, "p_hh"),
            c_hh: json_num(&t3, "c_hh"),
            p_ct: 0.0,
            c_ct: 0.0,
            m_ct: 0,
            k_ct: 0,
            lambda_ct: 0.0,
            r_cut: 0.0,
            ct_table_on: true,
        },
        c0: g("c_ct"),
        p_ct: g("p_ct"),
        m_ct: g("m_ct") as u8,
        source: format!("{} (contacts) + {} (walls, charge, c0)", p3.display(), p2.display()),
    }
}

/// CT-2's fitted family, carried so the table's error can be read against the thing it replaced.
fn family_ct(a: &Fragment, b: &Fragment, p_ct: f64, c0: f64, m_ct: u8) -> f64 {
    let mut tot = 0.0;
    for (donor, acceptor) in [(a, b), (b, a)] {
        let od = donor.centers[oxygen_of(donor)];
        let oa = acceptor.centers[oxygen_of(acceptor)];
        for &hi in hydrogens_of(donor).iter() {
            let xh = donor.centers[hi];
            let r = dist(&xh, &oa);
            let av = [od[0] - xh[0], od[1] - xh[1], od[2] - xh[2]];
            let bv = [oa[0] - xh[0], oa[1] - xh[1], oa[2] - xh[2]];
            let (na, nb) = ((av.iter().map(|x| x * x).sum::<f64>()).sqrt(), (bv.iter().map(|x| x * x).sum::<f64>()).sqrt());
            let cd = ((0..3).map(|i| av[i] * bv[i]).sum::<f64>() / (na * nb)).clamp(-1.0, 1.0);
            tot += (-c0 * r).exp() * (0.5 * (1.0 - cd)).powi(m_ct as i32);
        }
    }
    -p_ct * tot
}

// ------------------------------------------------------------------------------- the map's nodes

struct MapNode {
    name: String,
    family: String,
    a: Fragment,
    b: Fragment,
    e_ct: f64,
    de_exact: f64,
    e_exact: f64,
    e_noct: f64,
    y: [f64; CT_DIM],
    exact_source: String,
    sector_source: String,
    #[allow(dead_code)]
    r_oo: f64,
}

/// CT-3's `load_map` verbatim: every exact number from a record, the thirteen of record from
/// `gd0_*.json` and CT-1's `sector_*.json`, never a third source.
fn load_map(out: &Path, o: Species, h: Species) -> Vec<MapNode> {
    let ct2 = sibling(out, "ct2");
    let ct1 = sibling(out, "ct1");
    let mut files: Vec<PathBuf> = fs::read_dir(&ct2)
        .unwrap_or_else(|e| panic!("{}: {e}", ct2.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let n = p.file_name().and_then(|x| x.to_str()).unwrap_or("");
            (n.starts_with("node_") || n.starts_with("gd0_")) && n.ends_with(".json")
        })
        .collect();
    files.sort();
    let mut out_nodes = Vec::new();
    for f in files {
        let t = fs::read_to_string(&f).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        let name = json_str(&t, "node");
        let dc = json_centers(&t, "donor_centers");
        let ac = json_centers(&t, "acceptor_centers");
        assert_eq!(dc.len(), 3, "{name}: a donor is an oxygen and two hydrogens");
        assert_eq!(ac.len(), 3, "{name}: an acceptor is an oxygen and two hydrogens");
        let a = Fragment::new(vec![o, h, h], dc, vec![-2.0, 1.0, 1.0]);
        let b = Fragment::new(vec![o, h, h], ac, vec![-2.0, 1.0, 1.0]);
        let new_node = f.file_name().and_then(|x| x.to_str()).unwrap_or("").starts_with("node_");
        let (e_ct, de_exact, e_exact, e_noct, sector_source) = if new_node {
            (json_num(&t, "e_ct"), json_num(&t, "de_exact"), json_num(t.split("\"exact\": {").nth(1).unwrap_or(&t), "e_total"), json_num(t.split("\"sector\": {").nth(1).unwrap_or(&t), "e_noct"), f.display().to_string())
        } else {
            let sec = ct1.join(format!("sector_{name}.json"));
            let ts = fs::read_to_string(&sec).unwrap_or_else(|e| panic!("{}: {e}", sec.display()));
            let ex = json_num(&t, "e_full");
            let nc = json_num(&ts, "e_noct");
            (ex - nc, ex - (json_num(&ts, "e_a0") + json_num(&ts, "e_b0")), ex, nc, sec.display().to_string())
        };
        let (y, _) = node_contact(&a, &b);
        out_nodes.push(MapNode {
            name,
            family: json_str(&t, "family"),
            r_oo: dist(&a.centers[oxygen_of(&a)], &b.centers[oxygen_of(&b)]),
            a,
            b,
            e_ct,
            de_exact,
            e_exact,
            e_noct,
            y,
            exact_source: f.display().to_string(),
            sector_source,
        });
    }
    out_nodes
}

struct Site {
    y: [f64; CT_DIM],
    value: f64,
    members: Vec<usize>,
    spread: f64,
}

fn sites_of(nodes: &[MapNode], lo: [f64; CT_DIM], rng: [f64; CT_DIM]) -> Vec<Site> {
    let scaled = |y: [f64; CT_DIM]| -> [f64; CT_DIM] {
        let mut s = [0.0; CT_DIM];
        for j in 0..CT_DIM {
            s[j] = (y[j] - lo[j]) / rng[j];
        }
        s
    };
    let mut sites: Vec<Site> = Vec::new();
    for (i, n) in nodes.iter().enumerate() {
        let si = scaled(n.y);
        let hit = sites.iter().position(|s| {
            let sj = scaled(s.y);
            (0..CT_DIM).map(|j| (si[j] - sj[j]) * (si[j] - sj[j])).sum::<f64>().sqrt() < POLE_MERGE
        });
        match hit {
            Some(k) => sites[k].members.push(i),
            None => sites.push(Site { y: n.y, value: 0.0, members: vec![i], spread: 0.0 }),
        }
    }
    for s in sites.iter_mut() {
        let vals: Vec<f64> = s.members.iter().map(|&i| nodes[i].e_ct).collect();
        s.value = vals.iter().sum::<f64>() / vals.len() as f64;
        s.spread = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - vals.iter().cloned().fold(f64::INFINITY, f64::min);
    }
    sites
}

fn axis_box(nodes: &[MapNode]) -> ([f64; CT_DIM], [f64; CT_DIM]) {
    let mut lo = [f64::INFINITY; CT_DIM];
    let mut hi = [f64::NEG_INFINITY; CT_DIM];
    for n in nodes {
        for j in 0..CT_DIM {
            lo[j] = lo[j].min(n.y[j]);
            hi[j] = hi[j].max(n.y[j]);
        }
    }
    let mut rng = [0.0; CT_DIM];
    for j in 0..CT_DIM {
        rng[j] = hi[j] - lo[j];
    }
    (lo, rng)
}

fn table_from(sites: &[Site], c0: f64) -> CtTable {
    let mut t = CtTable::empty();
    assert!(t.begin(sites.len(), c0), "the table refused {} knots at c0 = {c0}", sites.len());
    for (i, s) in sites.iter().enumerate() {
        assert!(t.knot(i, s.y, s.value), "knot {i}");
    }
    let st = t.finish();
    assert_eq!(st, CtLoad::Ok, "the table did not load: {st:?}");
    t
}

// -------------------------------------------------------------------------------- the phase

/// One scored geometry.
struct Row {
    name: String,
    family: String,
    held_out: bool,
    r_oo: f64,
    min_ho: f64,
    donor: Vec<[f64; 3]>,
    acceptor: Vec<[f64; 3]>,
    de_exact: f64,
    e_ct_exact: f64,
    e_exact: f64,
    e_noct: f64,
    served_total: f64,
    served_field: f64,
    served_seam: f64,
    wall_oo: f64,
    wall_oh: f64,
    wall_hh: f64,
    contact_ho: f64,
    contact_hh: f64,
    ct_table: f64,
    ct_family: f64,
    loo_miss: f64,
    loo_rule: String,
    force: [[f64; 3]; 6],
    exact_source: String,
    sector_source: String,
}

fn run_served(out: &Path) {
    let t0 = Instant::now();
    let cpu0 = cpu_seconds();
    let (o, h) = (by_symbol("O").expect("O"), by_symbol("H").expect("H"));
    let law = load_law(out);
    let nodes = load_map(out, o, h);
    assert_eq!(nodes.len(), MAP_NODES, "the map is sixty-four nodes; {} were read", nodes.len());
    let (lo, rng) = axis_box(&nodes);
    let sites = sites_of(&nodes, lo, rng);
    let table = table_from(&sites, law.c0);

    // G-T0, carried: the interpolant reproduces every knot it was built on.
    let mut t0_worst = 0.0f64;
    for s in sites.iter() {
        t0_worst = t0_worst.max((table.eval(s.y) - s.value).abs());
    }

    // the per-SITE leave-one-out: the site removed, the table rebuilt from the rest, the site
    // predicted. This is the INTERPOLANT's own error and it is reported apart from the law's.
    let mut loo_of_site = vec![0.0f64; sites.len()];
    for k in 0..sites.len() {
        let rest: Vec<Site> = sites.iter().enumerate().filter(|(i, _)| *i != k).map(|(_, s)| Site { y: s.y, value: s.value, members: s.members.clone(), spread: s.spread }).collect();
        loo_of_site[k] = (table_from(&rest, law.c0).eval(sites[k].y) - sites[k].value).abs();
    }
    let mut site_of_node = vec![usize::MAX; nodes.len()];
    for (k, s) in sites.iter().enumerate() {
        for &m in s.members.iter() {
            site_of_node[m] = k;
        }
    }

    // the held-out node: CT-3 solved it AFTER the table was built, so its interpolant error is
    // measured directly (table against the exact E_CT) and not by leave-one-out.
    let ho_path = sibling(out, &format!("ct3/node_{HO_NAME}.json"));
    let ho_text = fs::read_to_string(&ho_path).unwrap_or_else(|e| panic!("{}: {e}", ho_path.display()));
    let (ha, hb) = twist_and_bend(o, h, HO_ANGSTROM, HO_TWIST_DEGREES, HO_TILT_DEGREES, HO_DONOR_DEGREES);
    // the record's own coordinates must be the ones this construction makes, atom for atom
    let hd = json_centers(&ho_text, "donor_centers");
    let hac = json_centers(&ho_text, "acceptor_centers");
    let mut ho_geom_worst = 0.0f64;
    for (i, c) in ha.centers.iter().enumerate() {
        ho_geom_worst = ho_geom_worst.max(dist(c, &hd[i]));
    }
    for (i, c) in hb.centers.iter().enumerate() {
        ho_geom_worst = ho_geom_worst.max(dist(c, &hac[i]));
    }

    let mut rows: Vec<Row> = Vec::new();
    let mut g_c1_worst = 0.0f64;
    let mut g_c1_node = String::new();

    let push = |name: &str,
                    family: &str,
                    held_out: bool,
                    a: &Fragment,
                    b: &Fragment,
                    de_exact: f64,
                    e_ct_exact: f64,
                    e_exact: f64,
                    e_noct: f64,
                    loo_miss: f64,
                    loo_rule: &str,
                    exact_source: &str,
                    sector_source: &str,
                    rows: &mut Vec<Row>,
                    g_c1_worst: &mut f64,
                    g_c1_node: &mut String| {
        let (total, field, seam) = engine_interaction(a, b, Some(law.m), SeamPlant::None, Some(&table));
        let (oo, ho, hh) = cross_classes(a, b);
        let wall_oo: f64 = oo.iter().map(|&r| law.m.wall(r) + law.m.dispersion(r)).sum();
        let wall_oh: f64 = ho.iter().map(|&r| law.m.wall_oh(r)).sum();
        let wall_hh: f64 = hh.iter().map(|&r| law.m.wall_hh(r)).sum();
        let contact_ho: f64 = ho.iter().map(|&r| law.m.penetration(r)).sum();
        let contact_hh: f64 = hh.iter().map(|&r| -law.m.p_hh * (-law.m.c_hh * r).exp()).sum();
        let (y, _) = node_contact(a, b);
        let ct = table.eval(y);
        // G-C1, carried: the engine's `e_seam` against this runner's own sum of the channels
        let formula = wall_oo + wall_oh + wall_hh + contact_ho + contact_hh + ct;
        if (seam - formula).abs() > *g_c1_worst {
            *g_c1_worst = (seam - formula).abs();
            *g_c1_node = name.to_string();
        }
        let force = engine_interaction_force(a, b, Some(law.m), Some(&table));
        let mut min_ho = f64::INFINITY;
        for &r in ho.iter() {
            min_ho = min_ho.min(r);
        }
        rows.push(Row {
            name: name.to_string(),
            family: family.to_string(),
            held_out,
            r_oo: dist(&a.centers[oxygen_of(a)], &b.centers[oxygen_of(b)]),
            min_ho,
            donor: a.centers.clone(),
            acceptor: b.centers.clone(),
            de_exact,
            e_ct_exact,
            e_exact,
            e_noct,
            served_total: total,
            served_field: field,
            served_seam: seam,
            wall_oo,
            wall_oh,
            wall_hh,
            contact_ho,
            contact_hh,
            ct_table: ct,
            ct_family: family_ct(a, b, law.p_ct, law.c0, law.m_ct),
            loo_miss,
            loo_rule: loo_rule.to_string(),
            force,
            exact_source: exact_source.to_string(),
            sector_source: sector_source.to_string(),
        });
    };

    for (i, n) in nodes.iter().enumerate() {
        let k = site_of_node[i];
        push(
            &n.name,
            &n.family,
            false,
            &n.a,
            &n.b,
            n.de_exact,
            n.e_ct,
            n.e_exact,
            n.e_noct,
            loo_of_site[k],
            "the node's SITE removed, the table rebuilt from the rest, the site predicted; a merged site's members share its number",
            &n.exact_source,
            &n.sector_source,
            &mut rows,
            &mut g_c1_worst,
            &mut g_c1_node,
        );
    }
    let ho_ct = json_num(&ho_text, "e_ct");
    let ho_de = json_num(&ho_text, "de_exact");
    let ho_ex = json_num(ho_text.split("\"exact\": {").nth(1).unwrap_or(&ho_text), "e_total");
    let ho_nc = json_num(ho_text.split("\"sector\": {").nth(1).unwrap_or(&ho_text), "e_noct");
    let (hy, _) = node_contact(&ha, &hb);
    let ho_loo = (table.eval(hy) - ho_ct).abs();
    push(
        HO_NAME,
        "twistbent",
        true,
        &ha,
        &hb,
        ho_de,
        ho_ct,
        ho_ex,
        ho_nc,
        ho_loo,
        "HELD OUT: the table was built before this node was solved, so its interpolant error is the table's own miss against the measured E_CT, not a leave-one-out",
        &ho_path.display().to_string(),
        &ho_path.display().to_string(),
        &mut rows,
        &mut g_c1_worst,
        &mut g_c1_node,
    );

    // G-X0: this runner's served numbers for the held-out node against CT-3's own record.
    let g3 = sibling(out, "ct3/gate.json");
    let g3t = fs::read_to_string(&g3).unwrap_or_else(|e| panic!("{}: {e}", g3.display()));
    let x0_total = (rows[rows.len() - 1].served_total - json_num(&g3t, "s2_total_predicted")).abs();
    let x0_field = (rows[rows.len() - 1].served_field - json_num(&g3t, "s2_field")).abs();
    let x0_seam = (rows[rows.len() - 1].served_seam - json_num(&g3t, "s2_seam")).abs();
    let x0_ct = (rows[rows.len() - 1].ct_table - json_num(&g3t, "table")).abs();
    let x0_worst = x0_total.max(x0_field).max(x0_seam).max(x0_ct);

    // G-F0: the analytic interaction force against a central difference of the interaction
    // ENERGY. The GATE is the held-out node, which is NOT a knot of the table; the two
    // deepest-transfer map nodes are differenced beside it as a reported leg, because a central
    // difference AT a knot of an r³ polyharmonic kernel carries that kernel's own third-derivative
    // discontinuity and its truncation there is a fact about the difference, not about the force.
    let mut fd_worst = 0.0f64;
    let mut fd_rel_worst = 0.0f64;
    let mut fd_node = String::new();
    let mut fd_knot_worst = 0.0f64;
    let mut fd_knot_rel_worst = 0.0f64;
    let mut fd_knot_node = String::new();
    let mut fd_sum_worst = 0.0f64;
    let mut fd_checked = 0usize;
    let held = rows.len() - 1;
    let mut fd_targets: Vec<usize> = vec![held];
    {
        let mut by_ct: Vec<(usize, f64)> = (0..nodes.len()).map(|i| (i, rows[i].ct_table.abs())).collect();
        by_ct.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap_or(std::cmp::Ordering::Equal));
        fd_targets.push(by_ct[0].0);
        fd_targets.push(by_ct[1].0);
    }
    for &g in fd_targets.iter() {
        let (a, b) = if g == held { (ha.clone(), hb.clone()) } else { (nodes[g].a.clone(), nodes[g].b.clone()) };
        let f = &rows[g].force;
        let mut sum = [0.0f64; 3];
        let mut node_scale = 0.0f64;
        for i in 0..6 {
            for c in 0..3 {
                sum[c] += f[i][c];
                node_scale = node_scale.max(f[i][c].abs());
            }
        }
        fd_sum_worst = fd_sum_worst.max(sum.iter().fold(0.0f64, |m, x| m.max(x.abs())));
        for i in 0..6 {
            for c in 0..3 {
                let (ap, bp) = if i < 3 { (moved(&a, i, c, FD_H), b.clone()) } else { (a.clone(), moved(&b, i - 3, c, FD_H)) };
                let (am, bm) = if i < 3 { (moved(&a, i, c, -FD_H), b.clone()) } else { (a.clone(), moved(&b, i - 3, c, -FD_H)) };
                let (ep, _, _) = engine_interaction(&ap, &bp, Some(law.m), SeamPlant::None, Some(&table));
                let (em, _, _) = engine_interaction(&am, &bm, Some(law.m), SeamPlant::None, Some(&table));
                let fd = -(ep - em) / (2.0 * FD_H);
                let abs = (fd - f[i][c]).abs();
                let rel = abs / node_scale.max(1e-12);
                fd_checked += 1;
                if g == held {
                    if abs > fd_worst {
                        fd_worst = abs;
                        fd_node = rows[g].name.clone();
                    }
                    fd_rel_worst = fd_rel_worst.max(rel);
                } else {
                    if abs > fd_knot_worst {
                        fd_knot_worst = abs;
                        fd_knot_node = rows[g].name.clone();
                    }
                    fd_knot_rel_worst = fd_knot_rel_worst.max(rel);
                }
            }
        }
    }

    // ------------------------------------------------------------------------------ the record
    fs::create_dir_all(out).unwrap_or_else(|e| panic!("{}: {e}", out.display()));
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str("  \"phase\": \"served\",\n");
    s.push_str(&format!("  \"law_source\": \"{}\",\n", esc(&law.source)));
    s.push_str(&format!("  \"table_source\": \"{}\",\n", esc(&sibling(out, "ct3/ct_table.json").display().to_string())));
    s.push_str("  \"rule\": \"the served law is CT-3's: FIELD-9's three walls and the point charge from ct2/wall_ct2.json, C1's re-fit contacts from ct3/wall_ct3.json, the transfer from the table rebuilt HERE out of the map's own records (never read back from ct_table.json), the dispersion an exact 0. The interaction is E(geometry) - E(acceptor 40 bohr along x) on the engine's own rows.\",\n");
    s.push_str("  \"no_new_solves\": true,\n");
    s.push_str(&format!("  \"nodes\": {}, \"held_out\": 1, \"sites\": {},\n", nodes.len(), sites.len()));
    s.push_str(&format!(
        "  \"g_t0\": {{\"pass\": {}, \"rule\": \"|table(y_i) - value_i| <= {} at every SITE\", \"worst\": {}}},\n",
        t0_worst <= KNOT_TOL,
        jn(KNOT_TOL),
        jn(t0_worst)
    ));
    s.push_str(&format!(
        "  \"g_c1\": {{\"pass\": {}, \"rule\": \"|engine e_seam - this runner's own channel sum| <= {} on every geometry\", \"worst\": {}, \"at\": \"{}\"}},\n",
        g_c1_worst <= G_C1_TOL,
        jn(G_C1_TOL),
        jn(g_c1_worst),
        esc(&g_c1_node)
    ));
    s.push_str(&format!(
        "  \"g_x0\": {{\"pass\": {}, \"rule\": \"this runner's served total, field, seam and table reading for the held-out node against ct3/gate.json's own filed numbers, <= {}\", \"worst\": {}, \"total\": {}, \"field\": {}, \"seam\": {}, \"table\": {}, \"geometry_worst_bohr\": {}}},\n",
        x0_worst <= X0_TOL && ho_geom_worst <= 1e-9,
        jn(X0_TOL),
        jn(x0_worst),
        jn(x0_total),
        jn(x0_field),
        jn(x0_seam),
        jn(x0_ct),
        jn(ho_geom_worst)
    ));
    s.push_str(&format!(
        "  \"g_f0\": {{\"pass\": {}, \"rule\": \"THE GATE is the HELD-OUT node, which is not a knot of the table: worst ABSOLUTE |analytic interaction force - central difference of the interaction energy| at h = {} over 6 atoms x 3 coordinates, bar {} hartree per bohr. The relative reading beside it is scaled by the NODE's own largest force component, never by the component being differenced, because a component through zero makes a relative reading meaningless.\", \"worst_absolute\": {}, \"worst_relative_to_node\": {}, \"at\": \"{}\", \"knot_leg\": {{\"rule\": \"the two deepest-transfer MAP nodes, which ARE knots: a central difference at a knot of an r^3 polyharmonic kernel carries that kernel's own third-derivative discontinuity, so this leg is REPORTED and does not gate\", \"worst_absolute\": {}, \"worst_relative_to_node\": {}, \"at\": \"{}\"}}, \"checked\": {}, \"translation_sum_worst\": {}}},\n",
        fd_worst <= FD_TOL,
        jn(FD_H),
        jn(FD_TOL),
        jn(fd_worst),
        jn(fd_rel_worst),
        esc(&fd_node),
        jn(fd_knot_worst),
        jn(fd_knot_rel_worst),
        esc(&fd_knot_node),
        fd_checked,
        jn(fd_sum_worst)
    ));
    s.push_str("  \"loo_rule\": \"each SITE removed, the table rebuilt from the rest, the site predicted; this is the INTERPOLANT's error and it is reported apart from the law's\",\n");
    s.push_str("  \"rows\": [\n");
    for (i, r) in rows.iter().enumerate() {
        let f = r
            .force
            .iter()
            .map(|v| format!("[{}, {}, {}]", jn(v[0]), jn(v[1]), jn(v[2])))
            .collect::<Vec<_>>()
            .join(", ");
        let dc = r.donor.iter().map(|c| format!("[{:.10}, {:.10}, {:.10}]", c[0], c[1], c[2])).collect::<Vec<_>>().join(", ");
        let ac = r.acceptor.iter().map(|c| format!("[{:.10}, {:.10}, {:.10}]", c[0], c[1], c[2])).collect::<Vec<_>>().join(", ");
        s.push_str(&format!(
            "    {{\"node\": \"{}\", \"family\": \"{}\", \"held_out\": {}, \"r_oo_bohr\": {}, \"min_cross_ho_bohr\": {},\n     \"donor_centers\": [{}], \"acceptor_centers\": [{}],\n     \"de_exact\": {}, \"e_ct_exact\": {}, \"e_exact\": {}, \"e_noct\": {},\n     \"served_total\": {}, \"served_field\": {}, \"served_seam\": {},\n     \"wall_oo\": {}, \"wall_oh\": {}, \"wall_hh\": {}, \"contact_ho\": {}, \"contact_hh\": {}, \"ct_table\": {}, \"ct_family\": {},\n     \"interpolant_miss\": {}, \"interpolant_rule\": \"{}\",\n     \"served_force\": [{}],\n     \"exact_source\": \"{}\", \"sector_source\": \"{}\"}}{}\n",
            esc(&r.name),
            esc(&r.family),
            r.held_out,
            jn(r.r_oo),
            jn(r.min_ho),
            dc,
            ac,
            jn(r.de_exact),
            jn(r.e_ct_exact),
            jn(r.e_exact),
            jn(r.e_noct),
            jn(r.served_total),
            jn(r.served_field),
            jn(r.served_seam),
            jn(r.wall_oo),
            jn(r.wall_oh),
            jn(r.wall_hh),
            jn(r.contact_ho),
            jn(r.contact_hh),
            jn(r.ct_table),
            jn(r.ct_family),
            jn(r.loo_miss),
            esc(&r.loo_rule),
            f,
            esc(&r.exact_source),
            esc(&r.sector_source),
            if i + 1 == rows.len() { "" } else { "," }
        ));
    }
    s.push_str("  ],\n");
    s.push_str(&format!("  \"seconds\": {}, \"cpu_seconds\": {}\n", jn(t0.elapsed().as_secs_f64()), jn(cpu_seconds() - cpu0)));
    s.push_str("}\n");
    let p = out.join("served.json");
    fs::write(&p, s).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    eprintln!("served.json   {} rows ({} map + 1 held out)", rows.len(), nodes.len());
    eprintln!("G-T0 {}  worst {:.3e}", if t0_worst <= KNOT_TOL { "PASS" } else { "FAIL" }, t0_worst);
    eprintln!("G-C1 {}  worst {:.3e} at {g_c1_node}", if g_c1_worst <= G_C1_TOL { "PASS" } else { "FAIL" }, g_c1_worst);
    eprintln!("G-X0 {}  worst {:.3e} (geometry {:.3e} bohr)", if x0_worst <= X0_TOL { "PASS" } else { "FAIL" }, x0_worst, ho_geom_worst);
    eprintln!("G-F0 {}  worst absolute {:.3e} (rel {:.3e}) at {fd_node}; knot leg {:.3e} at {fd_knot_node}", if fd_worst <= FD_TOL { "PASS" } else { "FAIL" }, fd_worst, fd_rel_worst, fd_knot_worst);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let phase = args.get(1).map(|s| s.as_str()).unwrap_or("served");
    let out = PathBuf::from(args.get(2).map(|s| s.as_str()).unwrap_or("../conformance/water_observatory/compare0"));
    match phase {
        "served" => run_served(&out),
        other => panic!("unknown phase {other}; the phases are: served"),
    }
}
