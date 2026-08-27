//! PERSISTENT GRAIN IDENTITY — the D-IDENT root repair.
//!
//! The arena is append-only (`RuntimeArena::materialize` pushes children, never
//! removes or reorders), so `nodes.holon` is a STABLE identity for the life of a
//! session. Twin comparison joins BY HOLON ID: grains active in both sessions
//! compare truly; holons refined away (and their fresh children) drop out of the
//! join instead of poisoning it. Predictions this example tests:
//!   1. sham twins: full join, divergence EXACTLY 0, every frame;
//!   2. probed twin at the probe frame: joined divergence ~machine-zero where the
//!      index-paired instrument read a pedestal of ~48/116 — the pedestal was
//!      never physics;
//!   3. the per-grain light-cone: left-sector joined divergence rises before
//!      right, at full per-node resolution.
use holon_sandbox::sim::Session;
use holon_sandbox::tier::TierId;
use std::collections::HashMap;
use std::fmt::Write as _;

const DT: f64 = 1.0 / 60.0;
const FRAMES: usize = 2400;
const PROBE: usize = 240;

fn by_id(s: &Session) -> HashMap<usize, ([f64; 2], [f64; 2], f64)> {
    let n = s.nodes();
    let mut m = HashMap::with_capacity(n.holon.len());
    for i in 0..n.holon.len() {
        m.insert(n.holon[i], (n.position[i], n.velocity[i], n.mass_kg[i]));
    }
    m
}

fn main() {
    // `idjoin_probe [offset] [out_dir]`. The probe lands at `mid - offset * (xmax - xmin)`,
    // so the default 0.2 is the LEFT placement both omega runs used.
    let args: Vec<String> = std::env::args().collect();
    let offset: f64 = args.get(1).map(|s| s.parse().expect("offset")).unwrap_or(0.2);
    let out: &str = args
        .get(2)
        .map(String::as_str)
        .unwrap_or("../../../scratchpad/omega/idjoin");
    std::fs::create_dir_all(out).unwrap();
    let tier = TierId::Sandbox;
    let probe_x_off = -offset;

    let probe = Session::new(tier);
    let xs: Vec<f64> = probe.nodes().position.iter().map(|p| p[0]).collect();
    let (xmin, xmax) = (xs.iter().cloned().fold(f64::MAX, f64::min),
                       xs.iter().cloned().fold(f64::MIN, f64::max));
    let mid = 0.5 * (xmin + xmax);

    for (name, probed) in [("sham", false), ("probed", true)] {
        let mut a = Session::new(tier);
        let mut b = Session::new(tier);
        a.throw(mid, 0.8, 0.5);
        b.throw(mid, 0.8, 0.5);
        let mut csv = String::from("frame,join,only_a,only_b,divL,divR\n");
        for f in 0..FRAMES {
            a.step(DT);
            b.step(DT);
            if probed && f == PROBE {
                b.throw(mid + probe_x_off * (xmax - xmin), 0.9, 0.05);
            }
            let ma = by_id(&a);
            let mb = by_id(&b);
            let (mut dl, mut dr, mut join) = (0.0, 0.0, 0usize);
            for (id, (pa, va, _)) in &ma {
                if let Some((pb, vb, _)) = mb.get(id) {
                    join += 1;
                    let d = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2)).sqrt()
                          + ((va[0] - vb[0]).powi(2) + (va[1] - vb[1]).powi(2)).sqrt() * DT;
                    if pa[0] < mid { dl += d } else { dr += d }
                }
            }
            writeln!(csv, "{f},{join},{},{},{dl:.9e},{dr:.9e}",
                     ma.len() - join, mb.len() - join).unwrap();
        }
        std::fs::write(format!("{out}/{name}.csv"), csv).unwrap();
        eprintln!("{name} written");
    }
}
