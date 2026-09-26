//! The fluid element's two charts behind one interface — REPLACE-1 §1 (`conformance/replace1/`).
//!
//! **The cell chart** keeps density AND momentum per cell and interpolates the momentum to a
//! face as the mean of the two cells beside it (the collocated finite-volume closure,
//! `field::continuity_integral`'s midpoint rule). **The staggered chart** keeps density per
//! cell and reads the momentum ON THE FACE: the `x`-velocity sum of the molecules within
//! `±h` of the face over `2h` — the MAC grid of 1965 (Harlow–Welch), nothing new, and the
//! chart the conservation law chose over the closure score (`RESPONSE1_AMENDMENT_5.md`:
//! R1″ `D = 0.180 ± 0.058` on the graded 36-cycle average against the cell chart's `0.62`).
//! `h = 0.25` Å is the value Amendment 5 fixed and did not tune.
//!
//! Both are read by the same continuity leg in its INTEGRAL form: a cell's occupancy change
//! over a window against the time-integrated net face flux over every readout inside it
//! (trapezoid in time), sign-aligned per drive cycle, lead windows against the relaxed tail
//! — `r1_closure_test.py` line for line, which is what PR-5 and G1 hold this module to:
//! the per-arm table on `response1_L200_seed0` to the printed digit, and R1″'s pooled `0.180`
//! from 36 cycles over three arms. The two-sided driven floor (`field::driven_floor_two_sided`,
//! RESPONSE-1 Amendment 4) is computed beside each pooled read and reported, not graded.

use crate::field::{continuity_spatial_floor, driven_floor_two_sided, Continuity};
use crate::walk::{min_image, pymod, RigidWalk, BOHR_A_SHORT};

/// `r1_closure_test.py`'s cadence (fs) and atomic unit of time (fs).
pub const DT_FS: f64 = 10.0;
pub const AU_FS: f64 = 0.024188843265857;

/// A fluid-element chart: where the momentum that closes the continuity law is read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FluidChart {
    /// Momentum per cell, interpolated to the face as the mean of the two neighbours (the
    /// collocated chart, "A slab-mean"), optionally lagged by `lag` readouts ("C").
    Cell { lag: usize },
    /// Momentum on the face: molecules within `±h_angstrom` of the face ("B face h").
    Staggered { h_angstrom: f64 },
}

impl FluidChart {
    /// The row label `r1_closure_test.py` prints.
    pub fn label(&self) -> String {
        match *self {
            FluidChart::Cell { lag: 0 } => "A slab-mean".to_string(),
            FluidChart::Cell { lag } => format!("C slab-mean lagged {:.0} fs", lag as f64 * DT_FS),
            FluidChart::Staggered { h_angstrom } => format!("B face h={} A", py_float(h_angstrom)),
        }
    }

    /// The rows `r1_closure_test.py` reads, in its order.
    pub fn r1_rows() -> Vec<FluidChart> {
        let mut v: Vec<FluidChart> = [0usize, 3, 5, 10].iter().map(|&lag| FluidChart::Cell { lag }).collect();
        v.extend([0.25, 0.5, 1.0, 2.0].iter().map(|&h| FluidChart::Staggered { h_angstrom: h }));
        v
    }

    /// The staggered chart of record (Amendment 5).
    pub const OF_RECORD: FluidChart = FluidChart::Staggered { h_angstrom: 0.25 };

    /// The face flux series: `F[t·nx + f]` = number flux density per unit time at face `f`
    /// (face `f` between cell `f` and cell `f + 1`, periodic), and the readout lag applied.
    fn face_flux(&self, g: &Gridded) -> (Vec<f64>, usize) {
        let (t_len, nx) = (g.frames, g.nx);
        match *self {
            FluidChart::Cell { lag } => {
                let mut f = vec![0.0; t_len * nx];
                for t in 0..t_len {
                    for c in 0..nx {
                        f[t * nx + c] = 0.5 * (g.pc[t * nx + c] + g.pc[t * nx + (c + 1) % nx]) / g.edge;
                    }
                }
                (f, lag)
            }
            FluidChart::Staggered { h_angstrom } => {
                let h = h_angstrom / BOHR_A_SHORT;
                let mut f = vec![0.0; t_len * nx];
                for t in 0..t_len {
                    for face in 0..nx {
                        let xf = (face + 1) as f64 * g.edge;
                        let mut s = 0.0;
                        for i in 0..g.n {
                            let dx = min_image(g.x[t * g.n + i] - xf, g.l);
                            if dx.abs() < h {
                                s += g.vx[t * g.n + i];
                            }
                        }
                        f[t * nx + face] = s / (2.0 * h);
                    }
                }
                (f, 0)
            }
        }
    }

    /// The chart's own spatial floor on one mode at `k = 2π/L` read on `nx` cells: the
    /// midpoint rule's `1 − sinc(π/n) cos(π/n)` for the cell chart; for the staggered chart
    /// the face flux is the face's own averaged over `±h`, so `1 − sinc(k h)`.
    pub fn spatial_floor(&self, nx: usize, box_bohr: f64) -> f64 {
        match *self {
            FluidChart::Cell { .. } => continuity_spatial_floor(nx),
            FluidChart::Staggered { h_angstrom } => {
                let kh = 2.0 * std::f64::consts::PI / box_bohr * (h_angstrom / BOHR_A_SHORT);
                1.0 - kh.sin() / kh
            }
        }
    }
}

/// Python's `str(float)` for the few values the labels use.
fn py_float(v: f64) -> String {
    if v == v.trunc() {
        format!("{v:.1}")
    } else {
        format!("{v}")
    }
}

/// A walk put on an `nx`-slab grid along `x`: wrapped `x`, `vx`, occupancy and momentum sums.
struct Gridded {
    frames: usize,
    n: usize,
    nx: usize,
    l: f64,
    edge: f64,
    x: Vec<f64>,
    vx: Vec<f64>,
    occ: Vec<f64>,
    pc: Vec<f64>,
}

fn grid(w: &RigidWalk, nx: usize) -> Gridded {
    let l = w.box_bohr;
    let edge = l / nx as f64;
    let (t_len, n) = (w.frames, w.n);
    let mut x = vec![0.0; t_len * n];
    let mut vx = vec![0.0; t_len * n];
    let mut occ = vec![0.0; t_len * nx];
    let mut pc = vec![0.0; t_len * nx];
    for t in 0..t_len {
        for i in 0..n {
            let xi = pymod(w.p(t, i, 0), l);
            let vi = w.v(t, i, 0);
            x[t * n + i] = xi;
            vx[t * n + i] = vi;
            let c = ((xi / edge).floor() as i64).rem_euclid(nx as i64) as usize;
            occ[t * nx + c] += 1.0;
            pc[t * nx + c] += vi;
        }
    }
    Gridded { frames: t_len, n, nx, l, edge, x, vx, occ, pc }
}

/// The window `a / c_s(water)` in readouts, as staked: `int(round(edge·Å/1500 m/s / 10 fs))`.
pub fn window_readouts(edge_bohr: f64) -> usize {
    ((edge_bohr * BOHR_A_SHORT * 1e-10 / 1500.0) / (DT_FS * 1e-15)).round_ties_even() as usize
}

/// One chart's read on one grid: the aligned stats and every cycle's aligned fields.
#[derive(Clone, Debug)]
pub struct ChartRead {
    pub chart: FluidChart,
    pub alpha: f64,
    pub r2: f64,
    pub d_lead: f64,
    pub d_tail: f64,
    pub rms_obs: f64,
    pub rms_tail: f64,
    /// RMS of the aligned tail prediction (the prediction side's noise, for the floor)
    pub rms_tail_pred: f64,
    /// per cycle, `[obs, pred, tail obs, tail pred]`, each `lead × nx`, flattened
    pub cycles: Vec<[Vec<f64>; 4]>,
}

/// Every chart's read on one grid of one arm (or of a pool).
#[derive(Clone, Debug)]
pub struct GridRead {
    pub nx: usize,
    pub edge_bohr: f64,
    pub w: usize,
    pub lead: usize,
    pub box_bohr: f64,
    pub rows: Vec<ChartRead>,
}

/// `Σ` of the net inflow to each cell over `[t0, t1)`: left-face flux minus right-face flux,
/// trapezoid in time, times the readout in au.
fn pred_from_face_flux(f: &[f64], nx: usize, t0: usize, t1: usize, shift: usize) -> Vec<f64> {
    let dt_au = DT_FS / AU_FS;
    (0..nx)
        .map(|c| {
            let fl = (c + nx - 1) % nx;
            let fr = c;
            let mut s = 0.0;
            for i in t0..t1 {
                let a = i.saturating_sub(shift);
                let b = (i + 1).saturating_sub(shift);
                s += 0.5 * ((f[a * nx + fl] + f[b * nx + fl]) - (f[a * nx + fr] + f[b * nx + fr]));
            }
            s * dt_au
        })
        .collect()
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn d_of(o: &[f64], p: &[f64]) -> f64 {
    let num: f64 = o.iter().zip(p).map(|(a, b)| (a - b) * (a - b)).sum();
    (num / dot(o, o).max(1e-300)).sqrt()
}

fn rms(v: &[f64]) -> f64 {
    (dot(v, v) / v.len() as f64).sqrt()
}

fn stats(chart: FluidChart, cycles: Vec<[Vec<f64>; 4]>) -> ChartRead {
    let len = cycles[0][0].len();
    let nc = cycles.len() as f64;
    let mut m: [Vec<f64>; 4] = [vec![0.0; len], vec![0.0; len], vec![0.0; len], vec![0.0; len]];
    for c in &cycles {
        for j in 0..4 {
            for (a, b) in m[j].iter_mut().zip(&c[j]) {
                *a += b;
            }
        }
    }
    for part in m.iter_mut() {
        part.iter_mut().for_each(|v| *v /= nc);
    }
    let (o, p, to, tp) = (&m[0], &m[1], &m[2], &m[3]);
    ChartRead {
        chart,
        alpha: dot(o, p) / dot(p, p).max(1e-300),
        r2: dot(o, p).powi(2) / (dot(o, o) * dot(p, p)).max(1e-300),
        d_lead: d_of(o, p),
        d_tail: d_of(to, tp),
        rms_obs: rms(o),
        rms_tail: rms(to),
        rms_tail_pred: rms(tp),
        cycles,
    }
}

/// THE READ: one arm, one grid, every chart — `r1_closure_test.py`'s `run` for one `nx`.
pub fn closure_read(w: &RigidWalk, nx: usize, charts: &[FluidChart], cycles: usize, relax: usize, lead: usize) -> GridRead {
    let g = grid(w, nx);
    let wn = window_readouts(g.edge);
    let mut rows = Vec::new();
    for chart in charts {
        let (f, shift) = chart.face_flux(&g);
        let mut per = Vec::with_capacity(cycles);
        for cyc in 0..cycles {
            let sgn = if cyc % 2 == 0 { 1.0 } else { -1.0 };
            let start = 1 + cyc * relax;
            let mut part: [Vec<f64>; 4] = [vec![0.0; lead * nx], vec![0.0; lead * nx], vec![0.0; lead * nx], vec![0.0; lead * nx]];
            for k in 0..lead {
                let (t0, t1) = (start + k * wn, start + k * wn + wn);
                let p = pred_from_face_flux(&f, nx, t0, t1, shift);
                let tt0 = start + relax - 1 - (lead - k) * wn;
                let tt1 = tt0 + wn;
                let tp = pred_from_face_flux(&f, nx, tt0, tt1, shift);
                for c in 0..nx {
                    part[0][k * nx + c] = sgn * (g.occ[t1 * nx + c] - g.occ[t0 * nx + c]);
                    part[1][k * nx + c] = sgn * p[c];
                    part[2][k * nx + c] = sgn * (g.occ[tt1 * nx + c] - g.occ[tt0 * nx + c]);
                    part[3][k * nx + c] = sgn * tp[c];
                }
            }
            per.push(part);
        }
        rows.push(stats(*chart, per));
    }
    GridRead { nx, edge_bohr: g.edge, w: wn, lead, box_bohr: w.box_bohr, rows }
}

/// The POOL: every arm's sign-aligned per-cycle fields (each arm's own signs, not re-derived)
/// averaged with equal weight per cycle — `r1_closure_test.py`'s `pooled`.
pub fn pool(reads: &[&GridRead]) -> GridRead {
    let first = reads[0];
    let rows = (0..first.rows.len())
        .map(|r| {
            let mut all = Vec::new();
            for g in reads {
                assert_eq!(g.rows[r].chart, first.rows[r].chart, "pool: arms read different charts");
                all.extend(g.rows[r].cycles.iter().cloned());
            }
            stats(first.rows[r].chart, all)
        })
        .collect();
    GridRead { rows, ..first.clone() }
}

/// Leave-one-cycle-out jackknife on a pooled row: `(SE of D_lead, SE of |D_lead − D_tail|)`,
/// `SE = sd(D₋ᵢ)·√(n − 1)` with the population SD (the estimator R1′ and R1″ use).
pub fn jackknife(row: &ChartRead) -> (f64, f64) {
    let n = row.cycles.len();
    let len = row.cycles[0][0].len();
    let mut s: [Vec<f64>; 4] = [vec![0.0; len], vec![0.0; len], vec![0.0; len], vec![0.0; len]];
    for c in &row.cycles {
        for j in 0..4 {
            for (a, b) in s[j].iter_mut().zip(&c[j]) {
                *a += b;
            }
        }
    }
    let mut jl = Vec::with_capacity(n);
    let mut jd = Vec::with_capacity(n);
    for c in &row.cycles {
        let m: Vec<Vec<f64>> =
            (0..4).map(|j| s[j].iter().zip(&c[j]).map(|(a, b)| (a - b) / (n - 1) as f64).collect()).collect();
        let dl = d_of(&m[0], &m[1]);
        jl.push(dl);
        jd.push((dl - d_of(&m[2], &m[3])).abs());
    }
    let sd = |v: &[f64]| {
        let mu = v.iter().sum::<f64>() / v.len() as f64;
        (v.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
    };
    (sd(&jl) * ((n - 1) as f64).sqrt(), sd(&jd) * ((n - 1) as f64).sqrt())
}

/// The two-sided driven floor of a (pooled) row: `(s, floor)` from the lead RMS and the
/// relaxed tail's observed and predicted RMS, with the chart's own spatial floor.
pub fn two_sided_floor(row: &ChartRead, nx: usize, box_bohr: f64) -> (f64, f64) {
    let lead = Continuity { windows_compared: 0, rms_observed: row.rms_obs, rms_residual: row.d_lead * row.rms_obs, rms_predicted: 0.0 };
    let tail = Continuity { windows_compared: 0, rms_observed: row.rms_tail, rms_residual: 0.0, rms_predicted: row.rms_tail_pred };
    driven_floor_two_sided(&lead, &tail, row.chart.spatial_floor(nx, box_bohr))
}

/// The table exactly as `r1_closure_test.py` prints it for one grid (without the `\n==` lead).
pub fn format_grid(g: &GridRead, pooled_cycles: Option<usize>) -> String {
    let mut s = String::new();
    let head = match pooled_cycles {
        None => format!(
            "== {} cells (slab {:.2} A, window {:.0} fs): aligned lead-window obs RMS {:.3}, tail RMS {:.3} counts\n",
            g.nx,
            g.edge_bohr * BOHR_A_SHORT,
            g.w as f64 * DT_FS,
            g.rows[0].rms_obs,
            g.rows[0].rms_tail
        ),
        Some(n) => format!(
            "== POOLED {} cells (slab {:.2} A, window {:.0} fs, {} cycles): aligned lead-window obs RMS {:.3}, tail RMS {:.3} counts\n",
            g.nx,
            g.edge_bohr * BOHR_A_SHORT,
            g.w as f64 * DT_FS,
            n,
            g.rows[0].rms_obs,
            g.rows[0].rms_tail
        ),
    };
    s.push_str(&head);
    s.push_str(&format!("   {:28} {:>6} {:>6} {:>7} {:>7}\n", "closure", "alpha", "R2", "D_lead", "D_tail"));
    for r in &g.rows {
        s.push_str(&format!(
            "   {:28} {:6.2} {:6.2} {:7.3} {:7.3}\n",
            r.chart.label(),
            r.alpha,
            r.r2,
            r.d_lead,
            r.d_tail
        ));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_the_readers() {
        let l: Vec<String> = FluidChart::r1_rows().iter().map(|c| c.label()).collect();
        assert_eq!(
            l,
            vec![
                "A slab-mean",
                "C slab-mean lagged 30 fs",
                "C slab-mean lagged 50 fs",
                "C slab-mean lagged 100 fs",
                "B face h=0.25 A",
                "B face h=0.5 A",
                "B face h=1.0 A",
                "B face h=2.0 A"
            ]
        );
    }

    #[test]
    fn the_window_is_the_sound_crossing_time() {
        // 432 waters, L = 44.39 bohr: 8 slabs of 2.94 A cross in 196 fs -> 20 readouts
        assert_eq!(window_readouts(44.39 / 8.0), 20);
        assert_eq!(window_readouts(44.39 / 4.0), 39);
        assert_eq!(window_readouts(44.39 / 16.0), 10);
    }

    /// Exact advection: every molecule moves at the same `vx`, crossings are exact, so the
    /// staggered chart's integral-form continuity closes to the face-averaging error and the
    /// cell chart does no better than its midpoint rule.
    #[test]
    fn a_uniform_drift_closes_on_the_staggered_chart() {
        let n = 400usize;
        let frames = 700usize;
        let l = 40.0f64;
        let v = 2e-4; // bohr per au
        let dt_au = DT_FS / AU_FS;
        let mut pos = Vec::with_capacity(frames * n * 3);
        let mut vel = Vec::with_capacity(frames * n * 3);
        for t in 0..frames {
            for i in 0..n {
                // a density wave: positions from a quantile map of 1 + 0.3 cos(2πx/L)
                let u = (i as f64 + 0.5) / n as f64;
                let mut x = u * l;
                for _ in 0..30 {
                    let f = x / l + 0.3 / (2.0 * std::f64::consts::PI) * (2.0 * std::f64::consts::PI * x / l).sin() - u;
                    let df = 1.0 / l + 0.3 / l * (2.0 * std::f64::consts::PI * x / l).cos();
                    x -= f / df;
                }
                pos.extend_from_slice(&[x + v * dt_au * t as f64, (i % 7) as f64, (i % 11) as f64]);
                vel.extend_from_slice(&[v, 0.0, 0.0]);
            }
        }
        let w = RigidWalk { frames, n, box_bohr: l, pos, vel, readout_fs: Some(10.0), temperature_k: None };
        let g = closure_read(&w, 8, &[FluidChart::Cell { lag: 0 }, FluidChart::Staggered { h_angstrom: 0.25 }], 2, 314, 2);
        let (cell, stag) = (&g.rows[0], &g.rows[1]);
        assert!(stag.d_lead < 0.2, "staggered chart on exact advection: {}", stag.d_lead);
        assert!((stag.alpha - 1.0).abs() < 0.2, "{}", stag.alpha);
        assert!(cell.d_lead.is_finite());
    }
}
