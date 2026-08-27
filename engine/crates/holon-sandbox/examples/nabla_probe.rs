//! NABLA PROBE — the ∇-face's engine instrument: per-cell fields on four
//! half-cell-shifted charts, sampled over time, for the chart-loop holonomy
//! measurement adjudicated in `nabla_adjudicate.py`.
//!
//! One session, no twins. The engine emits RAW per-cell fields only — mass,
//! momentum-x density numerator, kinetic-energy density numerator — on grid G
//! and its three half-cell shifts (+x, +y, +xy). Every transported/derived
//! quantity (the mass-weighted connection, the loop holonomy, the
//! uniform-weight prediction (I+Δx/4)(I+Δy/4)−I) is computed downstream, so
//! the instrument cannot smuggle the answer into the reading.
//!
//! Convergent art, credited: the half-cell plaquette transport composed of
//! two-point averages is the restriction/prolongation pair of MULTIGRID
//! (Brandt; Briggs–Henson–McCormick), and the derived loop operator
//! (I+Δx/4)(I+Δy/4) is its standard smoothing composite — ours is the
//! mass-weighted (state-dependent) reading of that classical object.
//!
//! Usage: `nabla_probe <out_dir> [warmup=60] [n_samples=8] [stride=60] [rethrow=120]`
//! Samples land at warmup + stride, warmup + 2·stride, …  Every `rethrow`
//! frames after warm-up a deterministic throw (alternating sides) keeps the
//! scene live — a settled pile has too few moving cells to pose the holonomy
//! question (smoke: 28 valid cell-samples against a floor of 100).

use holon_sandbox::sim::Session;
use holon_sandbox::tier::TierId;
use std::fmt::Write as _;

const DT: f64 = 1.0 / 60.0;
const NX: usize = 32;
const NY: usize = 16;
const CELLS: usize = NX * NY;

struct Grid {
    x0: f64,
    y0: f64,
    cw: f64,
    ch: f64,
}

impl Grid {
    fn over(session: &Session) -> Self {
        let n = session.nodes();
        let (mut x0, mut x1) = (f64::MAX, f64::MIN);
        let (mut y0, mut y1) = (f64::MAX, f64::MIN);
        for i in 0..n.holon.len() {
            if n.anchored[i] {
                continue;
            }
            x0 = x0.min(n.position[i][0]);
            x1 = x1.max(n.position[i][0]);
            y0 = y0.min(n.position[i][1]);
            y1 = y1.max(n.position[i][1]);
        }
        let w = if x1 > x0 { x1 - x0 } else { 1.0 };
        let h = if y1 > y0 { y1 - y0 } else { 1.0 };
        Self { x0, y0, cw: w / NX as f64, ch: h / NY as f64 }
    }

    /// Cell index under an origin shifted by (sx, sy) cell fractions.
    fn cell(&self, p: [f64; 2], sx: f64, sy: f64) -> usize {
        let cx = (((p[0] - self.x0) / self.cw - sx).floor() as isize).clamp(0, NX as isize - 1);
        let cy = (((p[1] - self.y0) / self.ch - sy).floor() as isize).clamp(0, NY as isize - 1);
        cy as usize * NX + cx as usize
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args.get(1).cloned().expect("out_dir");
    let warmup: usize = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(60);
    let n_samples: usize = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(8);
    let stride: usize = args.get(4).map(|s| s.parse().unwrap()).unwrap_or(60);
    let rethrow: usize = args.get(5).map(|s| s.parse().unwrap()).unwrap_or(120);
    std::fs::create_dir_all(&out).unwrap();

    let seed = Session::new(TierId::Sandbox);
    let xs: Vec<f64> = seed.nodes().position.iter().map(|p| p[0]).collect();
    let xmin = xs.iter().cloned().fold(f64::MAX, f64::min);
    let xmax = xs.iter().cloned().fold(f64::MIN, f64::max);
    let mid = 0.5 * (xmin + xmax);
    drop(seed);

    let mut s = Session::new(TierId::Sandbox);
    s.throw(mid, 0.8, 0.5);
    let grid = Grid::over(&s);
    for _ in 0..warmup {
        s.step(DT);
    }

    // The four chart origins of the plaquette, in cell fractions.
    const SHIFTS: [(f64, f64); 4] = [(0.0, 0.0), (0.5, 0.0), (0.5, 0.5), (0.0, 0.5)];

    let span = xmax - xmin;
    let mut csv = String::from("frame,chart,cell,mass,momx,ke\n");
    let mut since_throw = 0usize;
    let mut throw_ix = 0usize;
    for k in 1..=n_samples {
        for _ in 0..stride {
            since_throw += 1;
            if since_throw >= rethrow {
                let side = if throw_ix % 2 == 0 { -0.25 } else { 0.25 };
                s.throw(mid + side * span, 0.9, 0.05);
                throw_ix += 1;
                since_throw = 0;
            }
            s.step(DT);
        }
        let frame = warmup + k * stride;
        let n = s.nodes();
        for (ci, &(sx, sy)) in SHIFTS.iter().enumerate() {
            let mut mass = [0.0f64; CELLS];
            let mut momx = [0.0f64; CELLS];
            let mut ke = [0.0f64; CELLS];
            for i in 0..n.holon.len() {
                if n.anchored[i] {
                    continue;
                }
                let c = grid.cell(n.position[i], sx, sy);
                let m = n.mass_kg[i];
                let v = n.velocity[i];
                mass[c] += m;
                momx[c] += m * v[0];
                ke[c] += 0.5 * m * (v[0] * v[0] + v[1] * v[1]);
            }
            for c in 0..CELLS {
                writeln!(csv, "{frame},{ci},{c},{:.9e},{:.9e},{:.9e}", mass[c], momx[c], ke[c])
                    .unwrap();
            }
        }
        eprintln!("sampled frame {frame}");
    }
    std::fs::write(format!("{out}/fields.csv"), csv).unwrap();
    let meta = format!("grid = {NX}x{NY}\nwarmup = {warmup}\nn_samples = {n_samples}\nstride = {stride}\n");
    std::fs::write(format!("{out}/meta.txt"), meta).unwrap();
    eprintln!("fields.csv written");
}
