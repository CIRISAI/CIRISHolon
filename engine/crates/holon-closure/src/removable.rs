//! The REMOVABILITY GATE — what a tier carries is what the tier above cannot remove within its
//! budget (REPLACE-1, `conformance/replace1/REPLACE1_PREREG.md` §1; `OBJECT.md`, CORRECTION of
//! 2026-09-25: *a tier is a Closed view of the tier below THAT THE TIER ABOVE CARRIES*).
//!
//! # What it is
//!
//! A tier is offered a DICTIONARY of candidate columns (or declared blocks of columns), a
//! KEPT set it already has, and a TARGET — the level above's next state, the columns it
//! predicts. For each candidate the gate reads one number, the CARRIED INCREMENT:
//!
//! > held-out `R²` of the target from {kept + candidate} minus held-out `R²` from {kept},
//!
//! with a RE-PAIRED NULL (the same candidate taken from a time-shifted copy of its own unit,
//! and from a partner that shares nothing with the target — both reported), and a
//! leave-one-block jackknife SE over the held-out blocks. The candidate is CARRIED if its
//! increment exceeds the budget `β` and neither null does, DROPPED otherwise, and the pair
//! (increment, null) is its PRICE CERTIFICATE: what dropping it costs the tier above. The
//! whole answer is an [`Admission`] `{ carried, dropped, prices, se }`.
//!
//! The statistic is not new. It is `slow1_search.py`'s S2 increment, `hbond_search.py`'s S3
//! increment and `order1_search.py`'s O2 increment, which differed only in how they pooled —
//! now one function whose [`Ridge`] names the pooling and whose [`Folds`] names the split.
//! [`Ridge::ORDER1`] and [`Ridge::HBOND`] are those two readers' conventions, spelled out,
//! so that a banked read can be reproduced rather than approximated.
//!
//! The same gate with an EXACT price — a proof that dropping the candidate changes the target
//! by at most `c` — is [`admit_certified`]: that is how `holon::sector`'s backward light cone
//! drops a T-gate (price `0`, certified by the cone; budget `10⁻¹²`), so the circuit's removal
//! and the fluid's are the same function with two kinds of receipt.
//!
//! # What it refuses
//!
//! * **Dropping a candidate whose price exceeds the budget** ([`refuse_drop`] returns a
//!   [`Refusal`] carrying the name and the price). Refusal by price is the point of a gate:
//!   a tier that drops what the level above needs has changed the level above.
//! * **Counting a redundant column as carried, or as anything at all.** A candidate column
//!   that is an affine copy of the kept set (relative residual `≤ 10⁻⁹` over every row the
//!   gate reads) is declared REDUNDANT and not fitted, so its increment is exactly `0` — the
//!   ridge would otherwise split the kept column's weight with its copy and read a spurious
//!   increment of order the penalty (PR-3).
//! * **A time-blocked split on units of unequal length** (the blocks would not be the same
//!   times in every unit), and a candidate with a different number of units than the kept set.
//!
//! Pure `std`, deterministic, no dependency — this crate's isolation profile.

use std::fmt;

// ---------------------------------------------------------------------------------------------
// A dense row-major matrix — the one shape every reader here hands the gate
// ---------------------------------------------------------------------------------------------

/// A dense row-major matrix: `rows × cols`, `data[r * cols + c]`.
#[derive(Clone, Debug, PartialEq)]
pub struct Mat {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
}

impl Mat {
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Mat { rows, cols, data: vec![0.0; rows * cols] }
    }

    /// From a list of equal-length rows.
    pub fn from_rows(rows: &[Vec<f64>]) -> Self {
        let cols = rows.first().map_or(0, |r| r.len());
        let mut data = Vec::with_capacity(rows.len() * cols);
        for r in rows {
            assert_eq!(r.len(), cols, "Mat::from_rows: ragged rows");
            data.extend_from_slice(r);
        }
        Mat { rows: rows.len(), cols, data }
    }

    /// From columns of equal length.
    pub fn from_cols(cols: &[Vec<f64>]) -> Self {
        let rows = cols.first().map_or(0, |c| c.len());
        let mut m = Mat::zeros(rows, cols.len());
        for (j, c) in cols.iter().enumerate() {
            assert_eq!(c.len(), rows, "Mat::from_cols: ragged columns");
            for (i, v) in c.iter().enumerate() {
                m.data[i * cols.len() + j] = *v;
            }
        }
        m
    }

    #[inline]
    pub fn at(&self, r: usize, c: usize) -> f64 {
        self.data[r * self.cols + c]
    }

    #[inline]
    pub fn set(&mut self, r: usize, c: usize, v: f64) {
        self.data[r * self.cols + c] = v;
    }

    #[inline]
    pub fn row(&self, r: usize) -> &[f64] {
        &self.data[r * self.cols..(r + 1) * self.cols]
    }

    /// The listed columns, in the listed order.
    pub fn select_cols(&self, idx: &[usize]) -> Mat {
        let mut m = Mat::zeros(self.rows, idx.len());
        for r in 0..self.rows {
            for (j, &c) in idx.iter().enumerate() {
                m.data[r * idx.len() + j] = self.at(r, c);
            }
        }
        m
    }

    /// `[self | other]`, row for row.
    pub fn hcat(&self, other: &Mat) -> Mat {
        assert_eq!(self.rows, other.rows, "Mat::hcat: row counts differ");
        let cols = self.cols + other.cols;
        let mut m = Mat::zeros(self.rows, cols);
        for r in 0..self.rows {
            m.data[r * cols..r * cols + self.cols].copy_from_slice(self.row(r));
            m.data[r * cols + self.cols..(r + 1) * cols].copy_from_slice(other.row(r));
        }
        m
    }

    /// The first `n` rows.
    pub fn head(&self, n: usize) -> Mat {
        let n = n.min(self.rows);
        Mat { rows: n, cols: self.cols, data: self.data[..n * self.cols].to_vec() }
    }

    /// `np.roll(self, shift, axis=0)`: row `t` of the result is row `(t − shift) mod rows`.
    pub fn roll(&self, shift: usize) -> Mat {
        let mut m = Mat::zeros(self.rows, self.cols);
        if self.rows == 0 {
            return m;
        }
        for t in 0..self.rows {
            let src = (t + self.rows - shift % self.rows) % self.rows;
            m.data[t * self.cols..(t + 1) * self.cols].copy_from_slice(self.row(src));
        }
        m
    }

    /// Rows permuted: row `t` of the result is row `perm[t]`.
    pub fn permute_rows(&self, perm: &[usize]) -> Mat {
        assert_eq!(perm.len(), self.rows);
        let mut m = Mat::zeros(self.rows, self.cols);
        for (t, &s) in perm.iter().enumerate() {
            m.data[t * self.cols..(t + 1) * self.cols].copy_from_slice(self.row(s));
        }
        m
    }
}

// ---------------------------------------------------------------------------------------------
// Small linear algebra — Cholesky, Jacobi, the inverse square root VAMP whitens with
// ---------------------------------------------------------------------------------------------

/// Solve `A X = B` for symmetric positive definite `A` (`n × n`) and `B` (`n × m`), both
/// row-major. `None` if `A` is not positive definite at working precision.
pub fn solve_spd(a: &[f64], n: usize, b: &[f64], m: usize) -> Option<Vec<f64>> {
    let mut l = vec![0.0f64; n * n];
    for i in 0..n {
        for j in 0..=i {
            let mut s = a[i * n + j];
            for k in 0..j {
                s -= l[i * n + k] * l[j * n + k];
            }
            if i == j {
                if s <= 0.0 || !s.is_finite() {
                    return None;
                }
                l[i * n + i] = s.sqrt();
            } else {
                l[i * n + j] = s / l[j * n + j];
            }
        }
    }
    let mut x = b.to_vec();
    for c in 0..m {
        for i in 0..n {
            let mut s = x[i * m + c];
            for k in 0..i {
                s -= l[i * n + k] * x[k * m + c];
            }
            x[i * m + c] = s / l[i * n + i];
        }
        for i in (0..n).rev() {
            let mut s = x[i * m + c];
            for k in i + 1..n {
                s -= l[k * n + i] * x[k * m + c];
            }
            x[i * m + c] = s / l[i * n + i];
        }
    }
    Some(x)
}

/// Eigen-decomposition of a symmetric `n × n` matrix by cyclic Jacobi: `(w, V)` with the
/// eigenvectors in the COLUMNS of `V` (row-major), `A = V diag(w) Vᵀ`.
pub fn sym_eigen(a: &[f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut m = a.to_vec();
    let mut v = vec![0.0f64; n * n];
    for i in 0..n {
        v[i * n + i] = 1.0;
    }
    for _sweep in 0..100 {
        let mut off = 0.0;
        for i in 0..n {
            for j in i + 1..n {
                off += m[i * n + j] * m[i * n + j];
            }
        }
        let scale: f64 = (0..n).map(|i| m[i * n + i] * m[i * n + i]).sum::<f64>() + off;
        if off <= 1e-30 * scale.max(1e-300) {
            break;
        }
        for p in 0..n {
            for q in p + 1..n {
                let apq = m[p * n + q];
                if apq == 0.0 {
                    continue;
                }
                let theta = (m[q * n + q] - m[p * n + p]) / (2.0 * apq);
                let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
                let t = if theta == 0.0 { 1.0 } else { t };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                for k in 0..n {
                    let (mkp, mkq) = (m[k * n + p], m[k * n + q]);
                    m[k * n + p] = c * mkp - s * mkq;
                    m[k * n + q] = s * mkp + c * mkq;
                }
                for k in 0..n {
                    let (mpk, mqk) = (m[p * n + k], m[q * n + k]);
                    m[p * n + k] = c * mpk - s * mqk;
                    m[q * n + k] = s * mpk + c * mqk;
                }
                for k in 0..n {
                    let (vkp, vkq) = (v[k * n + p], v[k * n + q]);
                    v[k * n + p] = c * vkp - s * vkq;
                    v[k * n + q] = s * vkp + c * vkq;
                }
            }
        }
    }
    ((0..n).map(|i| m[i * n + i]).collect(), v)
}

/// `(C + ridge·I)^{-1/2}` for symmetric `C`, eigenvalues clipped at `1e-300` — `view_search.py`'s
/// `inv_sqrt`, the whitening every VAMP read in this program uses.
pub fn inv_sqrt(c: &[f64], n: usize, ridge: f64) -> Vec<f64> {
    let mut a = c.to_vec();
    for i in 0..n {
        a[i * n + i] += ridge;
    }
    let (w, v) = sym_eigen(&a, n);
    let mut out = vec![0.0f64; n * n];
    for k in 0..n {
        let s = w[k].max(1e-300).powf(-0.5);
        for i in 0..n {
            for j in 0..n {
                out[i * n + j] += v[i * n + k] * s * v[j * n + k];
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// The split, the horizon and the ridge — the three things the banked readers differed in
// ---------------------------------------------------------------------------------------------

/// How the rows are held out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Folds {
    /// `k` contiguous blocks of ORIGIN TIMES, the same times in every unit, with the training
    /// pairs within `lag` of the test block dropped (`order1_search.py`'s `ccarried`:
    /// `edges = linspace(0, T − lag, k + 1).astype(int)`). Requires equal-length units.
    TimeBlocks { k: usize },
    /// Unit `u` is tested in fold `u mod k` and trained on in every other (`slow1_search.py`'s
    /// and `hbond_search.py`'s molecule folds; `reason_search0b.py`'s chain folds).
    Units { k: usize },
}

impl Folds {
    pub fn k(&self) -> usize {
        match *self {
            Folds::TimeBlocks { k } | Folds::Units { k } => k,
        }
    }
}

/// What the target row at origin `t` is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Horizon {
    /// `y(t) = target(t + lag)` — the next state (ORDER-1's O2, SLOW-1's own target).
    Future,
    /// `y(t) = target(t + lag) − target(t)` — the change over the lag (HBOND's `Δv`).
    Increment,
}

/// How the held-out `R²` is pooled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pool {
    /// Residual and total sums accumulated over every fold and every target column, one ratio
    /// at the end (`order1_search.py`'s `pooled`).
    SumThenRatio,
    /// One `R²` per fold over all target columns, then the mean over folds
    /// (`hbond_search.py`'s `r2_cols`, `slow1_search.py`'s `ridge_r2` averaged).
    MeanOfFolds,
}

/// The regression every increment is read with: ridge on standardised inputs, penalty
/// `lambda_rel · N` (N the training rows), intercept the training mean of the target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ridge {
    pub lambda_rel: f64,
    /// added to each input column's training SD before standardising
    pub x_eps: f64,
    /// residuals and totals in units of each target column's TRAINING SD (so every target
    /// column weighs the same), or raw
    pub standardize_target: bool,
    pub pool: Pool,
}

impl Ridge {
    /// `order1_search.py`'s `ridge_multi` + `pooled`: `λ = 10⁻³ N`, SD `+ 1e-300`, target
    /// standardised, pooled over folds and columns.
    pub const ORDER1: Ridge =
        Ridge { lambda_rel: 1e-3, x_eps: 1e-300, standardize_target: true, pool: Pool::SumThenRatio };
    /// `hbond_search.py`'s `r2_cols`: `λ = 10⁻³ N`, SD `+ 1e-12`, raw target, mean over folds.
    pub const HBOND: Ridge =
        Ridge { lambda_rel: 1e-3, x_eps: 1e-12, standardize_target: false, pool: Pool::MeanOfFolds };
}

/// The question a gate asks: a lag, a horizon, a split, a regression.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Question {
    pub lag: usize,
    pub horizon: Horizon,
    pub folds: Folds,
    pub ridge: Ridge,
}

/// One unit of the sample — a molecule, a wavevector's chain, a reasoning chain: its KEPT
/// columns and its TARGET columns on one time axis (row `t` of each is time `t`).
#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    pub kept: Mat,
    pub target: Mat,
}

impl Unit {
    pub fn len(&self) -> usize {
        self.kept.rows
    }
    pub fn is_empty(&self) -> bool {
        self.kept.rows == 0
    }
}

/// A candidate column or declared block: a name and its columns, one matrix per unit, on the
/// units' time axis.
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub name: String,
    pub cols: Vec<Mat>,
}

impl Candidate {
    pub fn new(name: &str, cols: Vec<Mat>) -> Self {
        Candidate { name: name.to_string(), cols }
    }
}

// ---------------------------------------------------------------------------------------------
// The design: every (unit, origin) row, the fold it is tested in, and its target row
// ---------------------------------------------------------------------------------------------

struct Design {
    x: Vec<f64>,
    y: Vec<f64>,
    p: usize,
    m: usize,
    unit: Vec<usize>,
    t: Vec<usize>,
}

fn build(units: &[Unit], extra: Option<&[Mat]>, q: &Question) -> Design {
    let p = units[0].kept.cols + extra.map_or(0, |e| e[0].cols);
    let m = units[0].target.cols;
    let mut d = Design { x: Vec::new(), y: Vec::new(), p, m, unit: Vec::new(), t: Vec::new() };
    for (u, un) in units.iter().enumerate() {
        let tu = un.len();
        if tu <= q.lag {
            continue;
        }
        for t in 0..tu - q.lag {
            d.x.extend_from_slice(un.kept.row(t));
            if let Some(e) = extra {
                d.x.extend_from_slice(e[u].row(t));
            }
            let fut = un.target.row(t + q.lag);
            match q.horizon {
                Horizon::Future => d.y.extend_from_slice(fut),
                Horizon::Increment => {
                    let now = un.target.row(t);
                    d.y.extend(fut.iter().zip(now).map(|(a, b)| a - b));
                }
            }
            d.unit.push(u);
            d.t.push(t);
        }
    }
    d
}

/// Which rows a fold trains on and tests on.
fn fold_rows(d: &Design, units: &[Unit], q: &Question, b: usize) -> (Vec<usize>, Vec<usize>) {
    let mut tr = Vec::new();
    let mut te = Vec::new();
    match q.folds {
        Folds::Units { k } => {
            for (r, &u) in d.unit.iter().enumerate() {
                if u % k == b {
                    te.push(r)
                } else {
                    tr.push(r)
                }
            }
        }
        Folds::TimeBlocks { k } => {
            let n = units[0].len() - q.lag;
            let step = n as f64 / k as f64;
            let edge = |i: usize| if i == k { n } else { (i as f64 * step) as usize };
            let (s, e) = (edge(b) as i64, edge(b + 1) as i64);
            let l = q.lag as i64;
            for (r, &t) in d.t.iter().enumerate() {
                let t = t as i64;
                if t >= s && t < e {
                    te.push(r);
                } else if t < s - l || t >= e + l {
                    tr.push(r);
                }
            }
        }
    }
    (tr, te)
}

/// One fold's per-target-column residual and total sums of squares.
fn ridge_fold(d: &Design, cols: &[usize], tr: &[usize], te: &[usize], rg: &Ridge) -> (Vec<f64>, Vec<f64>) {
    let (p, m) = (cols.len(), d.m);
    let nt = tr.len() as f64;
    let xv = |r: usize, j: usize| d.x[r * d.p + cols[j]];
    let yv = |r: usize, c: usize| d.y[r * d.m + c];
    let mut mx = vec![0.0; p];
    let mut my = vec![0.0; m];
    for &r in tr {
        for (j, v) in mx.iter_mut().enumerate() {
            *v += xv(r, j);
        }
        for (c, v) in my.iter_mut().enumerate() {
            *v += yv(r, c);
        }
    }
    mx.iter_mut().for_each(|v| *v /= nt);
    my.iter_mut().for_each(|v| *v /= nt);
    let mut sx = vec![0.0; p];
    let mut sy = vec![0.0; m];
    for &r in tr {
        for j in 0..p {
            sx[j] += (xv(r, j) - mx[j]).powi(2);
        }
        for c in 0..m {
            sy[c] += (yv(r, c) - my[c]).powi(2);
        }
    }
    sx.iter_mut().for_each(|v| *v = (*v / nt).sqrt() + rg.x_eps);
    sy.iter_mut().for_each(|v| *v = (*v / nt).sqrt() + rg.x_eps);
    let mut g = vec![0.0; p * p];
    let mut ay = vec![0.0; p * m];
    let mut a = vec![0.0; p];
    for &r in tr {
        for j in 0..p {
            a[j] = (xv(r, j) - mx[j]) / sx[j];
        }
        for i in 0..p {
            for j in 0..p {
                g[i * p + j] += a[i] * a[j];
            }
            for c in 0..m {
                ay[i * m + c] += a[i] * (yv(r, c) - my[c]);
            }
        }
    }
    for i in 0..p {
        g[i * p + i] += rg.lambda_rel * nt;
    }
    let w = if p == 0 { Vec::new() } else { solve_spd(&g, p, &ay, m).expect("ridge normal equations are SPD") };
    let mut ym = vec![0.0; m];
    for &r in te {
        for (c, v) in ym.iter_mut().enumerate() {
            *v += yv(r, c);
        }
    }
    ym.iter_mut().for_each(|v| *v /= te.len().max(1) as f64);
    let mut ssr = vec![0.0; m];
    let mut sst = vec![0.0; m];
    for &r in te {
        for j in 0..p {
            a[j] = (xv(r, j) - mx[j]) / sx[j];
        }
        for c in 0..m {
            let mut pred = my[c];
            for j in 0..p {
                pred += a[j] * w[j * m + c];
            }
            ssr[c] += (yv(r, c) - pred).powi(2);
            sst[c] += (yv(r, c) - ym[c]).powi(2);
        }
    }
    if rg.standardize_target {
        for c in 0..m {
            ssr[c] /= sy[c] * sy[c];
            sst[c] /= sy[c] * sy[c];
        }
    }
    (ssr, sst)
}

/// Per-fold sums (residual, total), summed over target columns.
fn fold_sums(d: &Design, units: &[Unit], q: &Question, cols: &[usize]) -> Vec<(f64, f64)> {
    (0..q.folds.k())
        .map(|b| {
            let (tr, te) = fold_rows(d, units, q, b);
            let (ssr, sst) = ridge_fold(d, cols, &tr, &te, &q.ridge);
            (ssr.iter().sum(), sst.iter().sum())
        })
        .collect()
}

fn pooled_r2(f: &[(f64, f64)], pool: Pool, skip: Option<usize>) -> f64 {
    let it = f.iter().enumerate().filter(|(i, _)| Some(*i) != skip).map(|(_, v)| *v);
    match pool {
        Pool::SumThenRatio => {
            let (r, t) = it.fold((0.0, 0.0), |a, v| (a.0 + v.0, a.1 + v.1));
            1.0 - r / t
        }
        Pool::MeanOfFolds => {
            let v: Vec<f64> = it.map(|(r, t)| 1.0 - r / t).collect();
            v.iter().sum::<f64>() / v.len() as f64
        }
    }
}

/// Candidate columns that are an affine function of the kept columns (and of the candidate
/// columns before them) over every row the gate reads: relative residual `≤ 10⁻⁹` after
/// modified Gram–Schmidt against `{1, kept, earlier candidates}`.
fn redundant_cols(d: &Design, n_kept: usize) -> Vec<usize> {
    let n = d.unit.len();
    let mut basis: Vec<Vec<f64>> = vec![vec![1.0 / (n as f64).sqrt(); n]];
    let mut redundant = Vec::new();
    for j in 0..d.p {
        let mut v: Vec<f64> = (0..n).map(|r| d.x[r * d.p + j]).collect();
        // the scale the residual is judged against: the column's own variation about its mean
        let mean = v.iter().sum::<f64>() / n as f64;
        let norm0 = v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>().sqrt();
        for _pass in 0..2 {
            for b in &basis {
                let dot: f64 = v.iter().zip(b).map(|(a, c)| a * c).sum();
                v.iter_mut().zip(b).for_each(|(a, c)| *a -= dot * c);
            }
        }
        let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm0 == 0.0 || norm <= 1e-9 * norm0 {
            if j >= n_kept {
                redundant.push(j - n_kept);
            }
        } else {
            v.iter_mut().for_each(|x| *x /= norm);
            basis.push(v);
        }
    }
    redundant
}

/// The carried increment of one candidate over the kept set — the gate's measurement.
#[derive(Clone, Debug, PartialEq)]
pub struct IncrementRead {
    /// held-out `R²` of the target from the kept set
    pub r2_kept: f64,
    /// held-out `R²` from the kept set plus the candidate
    pub r2_with: f64,
    /// `r2_with − r2_kept`
    pub increment: f64,
    /// leave-one-block jackknife SE of the increment over the held-out blocks
    pub se: f64,
    /// the increment with each block left out, in block order
    pub jackknife: Vec<f64>,
    /// candidate columns found to be affine copies of what was already there (not fitted)
    pub redundant: Vec<usize>,
    /// (unit, origin) rows read
    pub rows: usize,
}

/// THE STATISTIC: held-out `R²` of the target from `kept + cand` minus from `kept`.
pub fn increment(units: &[Unit], cand: &[Mat], q: &Question) -> IncrementRead {
    assert!(!units.is_empty(), "removable: no units");
    assert_eq!(units.len(), cand.len(), "removable: the candidate has a different number of units than the kept set");
    if let Folds::TimeBlocks { .. } = q.folds {
        let t0 = units[0].len();
        assert!(units.iter().all(|u| u.len() == t0), "removable: a time-blocked split needs equal-length units");
    }
    for (u, c) in units.iter().zip(cand) {
        assert_eq!(u.len(), c.rows, "removable: candidate and unit are on different time axes");
        assert_eq!(u.target.rows, u.kept.rows, "removable: kept and target are on different time axes");
    }
    let d = build(units, Some(cand), q);
    let nk = units[0].kept.cols;
    let redundant = redundant_cols(&d, nk);
    let kept_cols: Vec<usize> = (0..nk).collect();
    let with_cols: Vec<usize> =
        (0..d.p).filter(|&j| j < nk || !redundant.contains(&(j - nk))).collect();
    let fk = fold_sums(&d, units, q, &kept_cols);
    // A candidate that is wholly redundant is not refitted: the model with it IS the model
    // without it, and the increment is exactly zero rather than zero plus rounding.
    let fw = if with_cols.len() == kept_cols.len() { fk.clone() } else { fold_sums(&d, units, q, &with_cols) };
    let r2_kept = pooled_r2(&fk, q.ridge.pool, None);
    let r2_with = pooled_r2(&fw, q.ridge.pool, None);
    let k = q.folds.k();
    let jackknife: Vec<f64> =
        (0..k).map(|b| pooled_r2(&fw, q.ridge.pool, Some(b)) - pooled_r2(&fk, q.ridge.pool, Some(b))).collect();
    let mean = jackknife.iter().sum::<f64>() / k as f64;
    let se = ((k as f64 - 1.0) / k as f64 * jackknife.iter().map(|v| (v - mean).powi(2)).sum::<f64>()).sqrt();
    IncrementRead { r2_kept, r2_with, increment: r2_with - r2_kept, se, jackknife, redundant, rows: d.unit.len() }
}

/// Held-out `R²` of the target from the kept set alone — the tier's closure score toward the
/// level above; `1 − r2` is its closure defect. Read by an independent call so that "the
/// defect rises by the price" (G5) is measured, not assumed.
pub fn closure_r2(units: &[Unit], q: &Question) -> f64 {
    let d = build(units, None, q);
    let cols: Vec<usize> = (0..d.p).collect();
    pooled_r2(&fold_sums(&d, units, q, &cols), q.ridge.pool, None)
}

// ---------------------------------------------------------------------------------------------
// The re-paired nulls
// ---------------------------------------------------------------------------------------------

/// Where the partner-swapped null takes its candidate from.
#[derive(Clone, Debug, PartialEq)]
pub enum Swap {
    /// Unit `u` gets the candidate of unit `(u + shift) mod n` (SLOW-1's PS-2: `i + n/2`).
    Derange { shift: usize },
    /// Unit `u` gets the candidate from a donor sample (ORDER-1's PO-2: the next seed's
    /// chains); the unit and the donor are truncated to the shorter of the two.
    Donor(Vec<Mat>),
    /// Every origin's candidate row is re-paired with another origin's, across all units, by a
    /// numpy-compatible PCG64 permutation of this seed (`reason_search0b.py`'s cross-chain
    /// re-pairing, for units too short and too unequal to derange).
    Permute { seed: u64 },
}

/// The nulls an admission reads. Both are reported; a candidate is carried only if neither
/// exceeds the budget.
#[derive(Clone, Debug, PartialEq)]
pub struct Nulls {
    /// the candidate from its own unit rolled by half the unit's length (ORDER-1's null)
    pub shift: bool,
    pub swap: Option<Swap>,
}

fn null_shift(units: &[Unit], cand: &[Mat], q: &Question) -> f64 {
    let rolled: Vec<Mat> = cand.iter().map(|c| c.roll(c.rows / 2)).collect();
    increment(units, &rolled, q).increment
}

fn null_swap(units: &[Unit], cand: &[Mat], swap: &Swap, q: &Question) -> f64 {
    match swap {
        Swap::Derange { shift } => {
            let n = units.len();
            let mut us = Vec::with_capacity(n);
            let mut cs = Vec::with_capacity(n);
            for u in 0..n {
                let donor = &cand[(u + shift) % n];
                let t = units[u].len().min(donor.rows);
                us.push(Unit { kept: units[u].kept.head(t), target: units[u].target.head(t) });
                cs.push(donor.head(t));
            }
            increment(&us, &cs, q).increment
        }
        Swap::Donor(donors) => {
            assert_eq!(donors.len(), units.len(), "removable: one donor per unit");
            let mut us = Vec::new();
            let mut cs = Vec::new();
            for (u, dn) in units.iter().zip(donors) {
                let t = u.len().min(dn.rows);
                us.push(Unit { kept: u.kept.head(t), target: u.target.head(t) });
                cs.push(dn.head(t));
            }
            increment(&us, &cs, q).increment
        }
        Swap::Permute { seed } => {
            let mut rows: Vec<(usize, usize)> = Vec::new();
            for (u, un) in units.iter().enumerate() {
                for t in 0..un.len().saturating_sub(q.lag) {
                    rows.push((u, t));
                }
            }
            let perm = NumpyPcg64::new(*seed).permutation(rows.len());
            let mut cs: Vec<Mat> = cand.to_vec();
            for (i, &(u, t)) in rows.iter().enumerate() {
                let (su, st) = rows[perm[i]];
                let w = cand[u].cols;
                cs[u].data[t * w..(t + 1) * w].copy_from_slice(cand[su].row(st));
            }
            increment(units, &cs, q).increment
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The admission
// ---------------------------------------------------------------------------------------------

/// What the gate decided about one candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// increment over the budget, every null within it
    Carried,
    /// increment within the budget: removable, at the stated price
    Dropped,
    /// increment over the budget but a null over it too: the gate cannot tell the candidate
    /// from its null, so it is not admitted (the prereg's "DROPPED otherwise") — and it is
    /// not droppable either, since its price exceeds the budget ([`refuse_drop`] refuses it)
    Unresolved,
}

/// A candidate's PRICE CERTIFICATE: what dropping it costs the level above.
#[derive(Clone, Debug, PartialEq)]
pub struct Price {
    pub name: String,
    /// the carried increment — for a certified price, the proven bound on the change
    pub increment: f64,
    pub se: f64,
    pub null_shift: Option<f64>,
    pub null_swap: Option<f64>,
    pub r2_kept: f64,
    pub r2_with: f64,
    pub redundant: Vec<usize>,
    /// `"measured"` (held-out regression) or the certificate's own words
    pub basis: String,
    pub verdict: Verdict,
}

impl Price {
    /// The largest null reported, or `None` when none was read.
    pub fn null_max(&self) -> Option<f64> {
        match (self.null_shift, self.null_swap) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        }
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = |v: Option<f64>| v.map_or("—".to_string(), |x| format!("{x:+.4}"));
        write!(
            f,
            "{}: increment {:+.4} ± {:.4} (R² {:.4} → {:.4}) | null shifted {} swapped {}{} -> {:?}",
            self.name,
            self.increment,
            self.se,
            self.r2_kept,
            self.r2_with,
            n(self.null_shift),
            n(self.null_swap),
            if self.redundant.is_empty() { String::new() } else { format!(" | redundant cols {:?}", self.redundant) },
            self.verdict
        )
    }
}

/// The gate's answer: what the tier carries, what it drops, and the price of every call.
#[derive(Clone, Debug, PartialEq)]
pub struct Admission {
    pub budget: f64,
    pub carried: Vec<String>,
    pub dropped: Vec<String>,
    pub prices: Vec<Price>,
    /// the jackknife SE of each candidate's increment, in candidate order
    pub se: Vec<f64>,
    /// ORDERED admission only (empty for the marginal one): the removals in the order the gate
    /// took them, each with the price it cost at that step
    pub order: Vec<Step>,
}

/// One step of ordered admission: the candidate removed, the price of removing it from the set
/// that was then standing, and the set left standing after it.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub price: Price,
    pub remaining: Vec<String>,
}

impl Admission {
    pub fn price(&self, name: &str) -> Option<&Price> {
        self.prices.iter().find(|p| p.name == name)
    }

    fn from_prices(budget: f64, prices: Vec<Price>) -> Admission {
        let carried = prices.iter().filter(|p| p.verdict == Verdict::Carried).map(|p| p.name.clone()).collect();
        let dropped = prices.iter().filter(|p| p.verdict != Verdict::Carried).map(|p| p.name.clone()).collect();
        let se = prices.iter().map(|p| p.se).collect();
        Admission { budget, carried, dropped, prices, se, order: Vec::new() }
    }

    /// The one-at-a-time admission, kept for the record (REPLACE-1 Amendment 1): each candidate
    /// against the kept set plus EVERY other candidate. It drops each of two redundant blocks —
    /// the defect the amendment names. Identical to [`admit`] with the other candidates folded
    /// into the kept set.
    pub fn marginal(base: &[Unit], candidates: &[Candidate], q: &Question, nulls: &Nulls, budget: f64) -> Admission {
        let all: Vec<usize> = (0..candidates.len()).collect();
        let prices = (0..candidates.len())
            .map(|b| {
                let us = with_blocks(base, candidates, &all, Some(b));
                let mut p = price_of(&us, &candidates[b], q, nulls);
                p.verdict = verdict_of(p.increment, p.null_max(), budget);
                p
            })
            .collect();
        Admission::from_prices(budget, prices)
    }

    /// THE TIER'S ADMISSION (REPLACE-1 Amendment 1): backward elimination with a joint price.
    /// Starting from the full dictionary (with `base` always kept), repeatedly remove the
    /// candidate whose removal from the set then standing costs the LEAST, and stop when the
    /// cheapest removal would cost more than `budget`. The survivors are carried, each with the
    /// price of removing it from the final set; of a set of mutually redundant blocks exactly
    /// one survives, and its price is the price of removing the whole set. Ties (prices within
    /// `10⁻¹²`) go to the candidate declared FIRST. The kept columns are assembled in name
    /// order, so the arithmetic does not depend on the order the dictionary was declared in.
    /// The stopping rule reads the price alone; the nulls are read at every step and reported.
    pub fn ordered(base: &[Unit], candidates: &[Candidate], q: &Question, nulls: &Nulls, budget: f64) -> Admission {
        let mut alive: Vec<usize> = (0..candidates.len()).collect();
        let mut order = Vec::new();
        loop {
            if alive.is_empty() {
                return Admission { budget, carried: Vec::new(), dropped: order.iter().map(|s: &Step| s.price.name.clone()).collect(), prices: order.iter().map(|s| s.price.clone()).collect(), se: order.iter().map(|s| s.price.se).collect(), order };
            }
            let now: Vec<Price> = alive
                .iter()
                .map(|&b| price_of(&with_blocks(base, candidates, &alive, Some(b)), &candidates[b], q, nulls))
                .collect();
            let mut k = 0;
            for j in 1..now.len() {
                if now[j].increment < now[k].increment - 1e-12 {
                    k = j;
                }
            }
            if now[k].increment > budget {
                let mut prices: Vec<Price> = order.iter().map(|s: &Step| s.price.clone()).collect();
                let mut carried = Vec::new();
                let mut dropped: Vec<String> = order.iter().map(|s| s.price.name.clone()).collect();
                for mut p in now {
                    p.verdict = verdict_of(p.increment, p.null_max(), budget);
                    if p.verdict == Verdict::Carried {
                        carried.push(p.name.clone());
                    } else {
                        dropped.push(p.name.clone());
                    }
                    prices.push(p);
                }
                let se = prices.iter().map(|p| p.se).collect();
                return Admission { budget, carried, dropped, prices, se, order };
            }
            let mut p = now[k].clone();
            p.verdict = Verdict::Dropped;
            alive.remove(k);
            order.push(Step { price: p, remaining: alive.iter().map(|&i| candidates[i].name.clone()).collect() });
        }
    }
}

/// The units with the standing candidate blocks (all of `set` but `except`) appended to the kept
/// columns, in NAME order.
fn with_blocks(base: &[Unit], candidates: &[Candidate], set: &[usize], except: Option<usize>) -> Vec<Unit> {
    let mut idx: Vec<usize> = set.iter().copied().filter(|&i| Some(i) != except).collect();
    idx.sort_by(|&a, &b| candidates[a].name.cmp(&candidates[b].name));
    base.iter()
        .enumerate()
        .map(|(u, un)| {
            let mut kept = un.kept.clone();
            for &i in &idx {
                kept = kept.hcat(&candidates[i].cols[u]);
            }
            Unit { kept, target: un.target.clone() }
        })
        .collect()
}

/// A candidate's measured price over a kept set, with its nulls (verdict left to the caller).
fn price_of(units: &[Unit], c: &Candidate, q: &Question, nulls: &Nulls) -> Price {
    let r = increment(units, &c.cols, q);
    Price {
        name: c.name.clone(),
        increment: r.increment,
        se: r.se,
        null_shift: if nulls.shift { Some(null_shift(units, &c.cols, q)) } else { None },
        null_swap: nulls.swap.as_ref().map(|s| null_swap(units, &c.cols, s, q)),
        r2_kept: r.r2_kept,
        r2_with: r.r2_with,
        redundant: r.redundant,
        basis: "measured".to_string(),
        verdict: Verdict::Dropped,
    }
}

fn verdict_of(increment: f64, null: Option<f64>, budget: f64) -> Verdict {
    if increment <= budget {
        Verdict::Dropped
    } else if null.is_some_and(|n| n > budget) {
        Verdict::Unresolved
    } else {
        Verdict::Carried
    }
}

/// THE GATE, measured: each candidate against the kept set, with its nulls, at a budget.
pub fn admit(units: &[Unit], candidates: &[Candidate], q: &Question, nulls: &Nulls, budget: f64) -> Admission {
    let mut prices = Vec::with_capacity(candidates.len());
    for c in candidates {
        let mut p = price_of(units, c, q, nulls);
        p.verdict = verdict_of(p.increment, p.null_max(), budget);
        prices.push(p);
    }
    Admission::from_prices(budget, prices)
}

/// An EXACT price: a proof about what dropping the candidate does to the target.
#[derive(Clone, Debug, PartialEq)]
pub enum Certificate {
    /// dropping the candidate changes the target by at most this much, provably
    Bound { change: f64, why: String },
    /// no bound is certified — the candidate cannot be dropped at any finite budget
    Unbounded { why: String },
}

/// THE GATE, certified: the same decision rule with a proof in place of a regression. No null
/// is read (a proof has none) and the SE is zero.
pub fn admit_certified(names: &[String], certs: &[Certificate], budget: f64) -> Admission {
    assert_eq!(names.len(), certs.len());
    let prices = names
        .iter()
        .zip(certs)
        .map(|(n, c)| {
            let (inc, why) = match c {
                Certificate::Bound { change, why } => (*change, why.clone()),
                Certificate::Unbounded { why } => (f64::INFINITY, why.clone()),
            };
            Price {
                name: n.clone(),
                increment: inc,
                se: 0.0,
                null_shift: None,
                null_swap: None,
                r2_kept: f64::NAN,
                r2_with: f64::NAN,
                redundant: Vec::new(),
                basis: why,
                verdict: verdict_of(inc, None, budget),
            }
        })
        .collect();
    Admission::from_prices(budget, prices)
}

/// The receipt for a drop the gate allowed.
#[derive(Clone, Debug, PartialEq)]
pub struct DropReceipt {
    pub name: String,
    pub price: f64,
    pub budget: f64,
}

/// The gate's refusal, by name and price.
#[derive(Clone, Debug, PartialEq)]
pub struct Refusal {
    pub name: String,
    pub price: f64,
    pub budget: f64,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "REFUSED: dropping '{}' costs the level above {:+.4} against a budget of {:+.4}",
            self.name, self.price, self.budget
        )
    }
}

/// Ask the gate to drop a candidate at a budget. It REFUSES, by name and with the price,
/// when the price exceeds the budget; otherwise it returns the receipt. A name the admission
/// never priced is refused too (price `+∞`): the gate does not drop what it has not read.
pub fn refuse_drop(adm: &Admission, name: &str, budget: f64) -> Result<DropReceipt, Refusal> {
    match adm.price(name) {
        None => Err(Refusal { name: name.to_string(), price: f64::INFINITY, budget }),
        Some(p) if p.increment > budget => Err(Refusal { name: name.to_string(), price: p.increment, budget }),
        Some(p) => Ok(DropReceipt { name: name.to_string(), price: p.increment, budget }),
    }
}

// ---------------------------------------------------------------------------------------------
// VAMP's held-out canonical score — the conscience sector's banked 8.9x is read with it
// ---------------------------------------------------------------------------------------------

fn standardise_pair(a: &Mat, b: &Mat) -> (Vec<f64>, Vec<f64>) {
    let d = a.cols;
    let na = a.rows as f64;
    let nb = b.rows as f64;
    let mean: Vec<f64> = (0..d)
        .map(|j| {
            let ma: f64 = (0..a.rows).map(|r| a.at(r, j)).sum::<f64>() / na;
            let mb: f64 = (0..b.rows).map(|r| b.at(r, j)).sum::<f64>() / nb;
            0.5 * (ma + mb)
        })
        .collect();
    let scale: Vec<f64> = (0..d)
        .map(|j| {
            let all: Vec<f64> = (0..a.rows).map(|r| a.at(r, j)).chain((0..b.rows).map(|r| b.at(r, j))).collect();
            let m = all.iter().sum::<f64>() / all.len() as f64;
            let v = all.iter().map(|x| (x - m).powi(2)).sum::<f64>() / all.len() as f64;
            v.sqrt().max(1e-12)
        })
        .collect();
    (mean, scale)
}

fn cov(x: &[f64], y: &[f64], n: usize, dx: usize, dy: usize) -> Vec<f64> {
    let mut c = vec![0.0; dx * dy];
    for r in 0..n {
        for i in 0..dx {
            for j in 0..dy {
                c[i * dy + j] += x[r * dx + i] * y[r * dy + j];
            }
        }
    }
    c.iter_mut().for_each(|v| *v /= n as f64);
    c
}

fn matmul(a: &[f64], b: &[f64], n: usize, k: usize, m: usize) -> Vec<f64> {
    let mut c = vec![0.0; n * m];
    for i in 0..n {
        for l in 0..k {
            let x = a[i * k + l];
            for j in 0..m {
                c[i * m + j] += x * b[l * m + j];
            }
        }
    }
    c
}

/// `reason_search0b.py`'s `heldout(trA, trB, teA, teB, k)` at `k` = the full dimension: VAMP
/// fitted on the training pairs (standardised by their pooled mean and SD, whitened with a
/// ridge of `10⁻³ tr(C00)/d`), the test pairs projected and the squared Frobenius norm of
/// their whitened cross-covariance returned. At full `k` the singular vectors are an
/// orthogonal rotation that the score is invariant to, so they are not computed.
pub fn vamp_heldout_full(tr_a: &Mat, tr_b: &Mat, te_a: &Mat, te_b: &Mat) -> f64 {
    let d = tr_a.cols;
    let (mean, scale) = standardise_pair(tr_a, tr_b);
    let z = |m: &Mat| -> Vec<f64> {
        let mut out = vec![0.0; m.rows * d];
        for r in 0..m.rows {
            for j in 0..d {
                out[r * d + j] = (m.at(r, j) - mean[j]) / scale[j];
            }
        }
        out
    };
    let (a, b) = (z(tr_a), z(tr_b));
    let n = tr_a.rows;
    let c00 = cov(&a, &a, n, d, d);
    let ctt = cov(&b, &b, n, d, d);
    let ridge = 1e-3 * (0..d).map(|i| c00[i * d + i]).sum::<f64>() / d as f64;
    let w0 = inv_sqrt(&c00, d, ridge);
    let wt = inv_sqrt(&ctt, d, ridge);
    let (ta, tb) = (z(te_a), z(te_b));
    let nt = te_a.rows;
    let mut pa = matmul(&ta, &w0, nt, d, d);
    let mut pb = matmul(&tb, &wt, nt, d, d);
    for p in [&mut pa, &mut pb] {
        for j in 0..d {
            let m = (0..nt).map(|r| p[r * d + j]).sum::<f64>() / nt as f64;
            (0..nt).for_each(|r| p[r * d + j] -= m);
        }
    }
    let caa = cov(&pa, &pa, nt, d, d);
    let cab = cov(&pa, &pb, nt, d, d);
    let cbb = cov(&pb, &pb, nt, d, d);
    let tr = (0..d).map(|i| caa[i * d + i] + cbb[i * d + i]).sum::<f64>();
    let r = 1e-6 * tr / (2 * d) as f64;
    let ia = inv_sqrt(&caa, d, r);
    let ib = inv_sqrt(&cbb, d, r);
    let m = matmul(&matmul(&ia, &cab, d, d, d), &ib, d, d, d);
    m.iter().map(|v| v * v).sum()
}

// ---------------------------------------------------------------------------------------------
// numpy's default_rng, ported — so a banked re-paired null can be reproduced to the permutation
// ---------------------------------------------------------------------------------------------

/// `numpy.random.default_rng(seed)` (PCG64 seeded through `SeedSequence`) for non-negative
/// integer seeds below `2³²`, with `permutation(n)` and `random()` as numpy computes them.
/// It exists so that the banked readers' re-paired nulls (`default_rng(0).permutation(...)`)
/// are reproduced to the permutation instead of re-drawn; it is checked against numpy's
/// output in the tests. Gaussian draws are NOT numpy's (numpy uses a ziggurat): the plants
/// here are re-derived for this function, not replayed.
#[derive(Clone, Debug)]
pub struct NumpyPcg64 {
    state: u128,
    inc: u128,
    has_u32: bool,
    u32_buf: u32,
}

const PCG_MULT: u128 = (0x2360_ed05_1fc6_5da4u128 << 64) | 0x4385_df64_9fcc_f645u128;

impl NumpyPcg64 {
    pub fn new(seed: u64) -> Self {
        assert!(seed < (1u64 << 32), "NumpyPcg64: seeds below 2^32 only");
        // SeedSequence(seed).generate_state(4, uint64)
        const INIT_A: u32 = 0x43b0_d7e5;
        const MULT_A: u32 = 0x931e_8875;
        const INIT_B: u32 = 0x8b51_f9dd;
        const MULT_B: u32 = 0x58f3_8ded;
        const MIX_L: u32 = 0xca01_f9dd;
        const MIX_R: u32 = 0x4973_f715;
        let entropy = [seed as u32];
        let mut hc = INIT_A;
        let hashmix = |v: u32, hc: &mut u32| -> u32 {
            let mut v = v ^ *hc;
            *hc = hc.wrapping_mul(MULT_A);
            v = v.wrapping_mul(*hc);
            v ^ (v >> 16)
        };
        let mix = |x: u32, y: u32| -> u32 {
            let r = MIX_L.wrapping_mul(x).wrapping_sub(MIX_R.wrapping_mul(y));
            r ^ (r >> 16)
        };
        let mut pool = [0u32; 4];
        for (i, p) in pool.iter_mut().enumerate() {
            *p = hashmix(if i < entropy.len() { entropy[i] } else { 0 }, &mut hc);
        }
        for src in 0..4 {
            for dst in 0..4 {
                if src != dst {
                    let h = hashmix(pool[src], &mut hc);
                    pool[dst] = mix(pool[dst], h);
                }
            }
        }
        let mut hb = INIT_B;
        let mut words = [0u32; 8];
        for (i, w) in words.iter_mut().enumerate() {
            let mut v = pool[i % 4];
            v ^= hb;
            hb = hb.wrapping_mul(MULT_B);
            v = v.wrapping_mul(hb);
            v ^= v >> 16;
            *w = v;
        }
        let w64 = |i: usize| (words[2 * i] as u64) | ((words[2 * i + 1] as u64) << 32);
        let initstate = ((w64(0) as u128) << 64) | w64(1) as u128;
        let initseq = ((w64(2) as u128) << 64) | w64(3) as u128;
        let mut r = NumpyPcg64 { state: 0, inc: (initseq << 1) | 1, has_u32: false, u32_buf: 0 };
        r.step();
        r.state = r.state.wrapping_add(initstate);
        r.step();
        r
    }

    fn step(&mut self) {
        self.state = self.state.wrapping_mul(PCG_MULT).wrapping_add(self.inc);
    }

    pub fn next_u64(&mut self) -> u64 {
        self.step();
        let s = self.state;
        let x = ((s >> 64) as u64) ^ (s as u64);
        x.rotate_right((s >> 122) as u32)
    }

    pub fn next_u32(&mut self) -> u32 {
        if self.has_u32 {
            self.has_u32 = false;
            return self.u32_buf;
        }
        let n = self.next_u64();
        self.has_u32 = true;
        self.u32_buf = (n >> 32) as u32;
        n as u32
    }

    /// numpy's `random_interval(max)`: uniform on `0..=max` by masked rejection.
    fn interval(&mut self, max: u64) -> u64 {
        if max == 0 {
            return 0;
        }
        let mut mask = max;
        mask |= mask >> 1;
        mask |= mask >> 2;
        mask |= mask >> 4;
        mask |= mask >> 8;
        mask |= mask >> 16;
        mask |= mask >> 32;
        if max <= 0xffff_ffff {
            loop {
                let v = (self.next_u32() as u64) & mask;
                if v <= max {
                    return v;
                }
            }
        } else {
            loop {
                let v = self.next_u64() & mask;
                if v <= max {
                    return v;
                }
            }
        }
    }

    /// `rng.permutation(n)`: Fisher–Yates from the top, as numpy's `shuffle` does it.
    pub fn permutation(&mut self, n: usize) -> Vec<usize> {
        let mut v: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            let j = self.interval(i as u64) as usize;
            v.swap(i, j);
        }
        v
    }

    /// `rng.random()`: a double in `[0, 1)` from the top 53 bits.
    pub fn random(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }

    /// A standard normal by Box–Muller on [`Self::random`] — NOT numpy's ziggurat; used only
    /// by plants that are re-derived for this engine.
    pub fn normal(&mut self) -> f64 {
        let u1 = 1.0 - self.random();
        let u2 = self.random();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

// ---------------------------------------------------------------------------------------------
// A planted field — the Ornstein–Uhlenbeck process every plant here is made of
// ---------------------------------------------------------------------------------------------

/// `cols` independent unit-variance OU processes with memory `tau_rows` (in rows), `len` rows,
/// started from the stationary law: `z(t+1) = a z(t) + √(1 − a²) ξ`, `a = e^{−1/τ}`.
pub fn ou_field(len: usize, cols: usize, tau_rows: f64, rng: &mut NumpyPcg64) -> Mat {
    let a = (-1.0 / tau_rows).exp();
    let b = (1.0 - a * a).sqrt();
    let mut z: Vec<f64> = (0..cols).map(|_| rng.normal()).collect();
    let mut m = Mat::zeros(len, cols);
    for t in 0..len {
        m.data[t * cols..(t + 1) * cols].copy_from_slice(&z);
        for v in z.iter_mut() {
            *v = a * *v + b * rng.normal();
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn numpy_default_rng_is_reproduced() {
        // numpy 2.4: default_rng(0).permutation(10) and .integers(0, 2**63, 3)'s raw words
        assert_eq!(NumpyPcg64::new(0).permutation(10), vec![4, 6, 2, 7, 3, 5, 9, 0, 8, 1]);
        // one stream, two draws in sequence: default_rng(0).permutation(333), then (5000)
        let mut r = NumpyPcg64::new(0);
        assert_eq!(r.permutation(333)[..5], [182, 254, 212, 323, 292]);
        assert_eq!(r.permutation(5000)[4995..], [845, 1567, 3104, 2537, 810]);
        // default_rng(11): permutation(2501) then random()
        let mut r = NumpyPcg64::new(11);
        assert_eq!(r.permutation(2501)[..4], [1219, 1410, 1930, 1812]);
        assert_eq!(r.random(), 0.5810593010114219);
    }

    #[test]
    fn jacobi_and_inv_sqrt_invert_a_known_matrix() {
        let a = [4.0, 1.0, 0.5, 1.0, 3.0, 0.2, 0.5, 0.2, 2.0];
        let s = inv_sqrt(&a, 3, 0.0);
        // s · a · s = I
        let m = matmul(&matmul(&s, &a, 3, 3, 3), &s, 3, 3, 3);
        for i in 0..3 {
            for j in 0..3 {
                assert!(close(m[i * 3 + j], if i == j { 1.0 } else { 0.0 }, 1e-12), "{m:?}");
            }
        }
        let x = solve_spd(&a, 3, &[1.0, 2.0, 3.0], 1).unwrap();
        let b = matmul(&a, &x, 3, 3, 1);
        assert!(close(b[0], 1.0, 1e-12) && close(b[1], 2.0, 1e-12) && close(b[2], 3.0, 1e-12));
    }

    /// A small synthetic: target(t+1) = 0.8 kept(t) + 0.5 carried(t) + noise; an unrelated
    /// column; a copy of the kept column.
    fn synth(seed: u64, units: usize, len: usize) -> (Vec<Unit>, Candidate, Candidate, Candidate) {
        let mut rng = NumpyPcg64::new(seed);
        let (mut us, mut car, mut unr, mut cp) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for _ in 0..units {
            let k = ou_field(len, 1, 3.0, &mut rng);
            let c = ou_field(len, 1, 3.0, &mut rng);
            let x = ou_field(len, 1, 3.0, &mut rng);
            let mut y = Mat::zeros(len, 1);
            for t in 1..len {
                y.data[t] = 0.8 * k.data[t - 1] + 0.5 * c.data[t - 1] + 0.3 * rng.normal();
            }
            let copy = Mat { rows: len, cols: 1, data: k.data.iter().map(|v| 2.0 * v - 1.0).collect() };
            us.push(Unit { kept: k, target: y });
            car.push(c);
            unr.push(x);
            cp.push(copy);
        }
        (us, Candidate::new("carried", car), Candidate::new("unrelated", unr), Candidate::new("copy", cp))
    }

    fn q(folds: Folds, ridge: Ridge) -> Question {
        Question { lag: 1, horizon: Horizon::Future, folds, ridge }
    }

    #[test]
    fn the_gate_carries_the_wired_column_drops_the_rest_and_refuses_by_price() {
        let (us, car, unr, cp) = synth(3, 12, 400);
        let qq = q(Folds::Units { k: 4 }, Ridge::HBOND);
        let nulls = Nulls { shift: true, swap: Some(Swap::Derange { shift: 6 }) };
        let adm = admit(&us, &[car, unr, cp], &qq, &nulls, 0.02);
        assert_eq!(adm.carried, vec!["carried".to_string()], "{:#?}", adm.prices);
        assert_eq!(adm.dropped, vec!["unrelated".to_string(), "copy".to_string()]);
        let c = adm.price("carried").unwrap();
        assert!(c.increment > 0.1 && c.null_max().unwrap() < 0.01, "{c}");
        assert_eq!(adm.price("copy").unwrap().increment, 0.0, "a linear copy is redundant: exactly zero");
        assert_eq!(adm.price("copy").unwrap().redundant, vec![0]);
        let r = refuse_drop(&adm, "carried", 0.0).unwrap_err();
        assert_eq!(r.name, "carried");
        assert!(r.price > 0.1);
        assert!(refuse_drop(&adm, "carried", 1.0).is_ok());
        assert!(refuse_drop(&adm, "unrelated", 0.02).is_ok());
        assert!(refuse_drop(&adm, "never priced", 1.0).is_err());
    }

    /// Amendment 1's defect and its repair on the synthetic: two affine copies of a carried
    /// column are EACH removable given the other (marginal drops both); ordered admission keeps
    /// exactly one, at the column's own price; the unrelated column goes first.
    #[test]
    fn ordered_admission_keeps_one_of_two_copies_at_the_columns_price() {
        let (us, car, unr, _) = synth(3, 12, 400);
        let copy: Vec<Mat> = car.cols.iter().map(|m| Mat { rows: m.rows, cols: 1, data: m.data.iter().map(|v| 2.0 * v - 1.0).collect() }).collect();
        let dict = [Candidate::new("carried", car.cols.clone()), Candidate::new("carried copy", copy), unr.clone()];
        let qq = q(Folds::Units { k: 4 }, Ridge::HBOND);
        let nulls = Nulls { shift: false, swap: None };
        let m = Admission::marginal(&us, &dict, &qq, &nulls, 0.02);
        assert!(m.carried.is_empty(), "marginal drops both copies: {:?}", m.carried);
        let o = Admission::ordered(&us, &dict, &qq, &nulls, 0.02);
        assert_eq!(o.carried.len(), 1);
        let single = admit(&us, std::slice::from_ref(&car), &qq, &nulls, 0.02).prices[0].increment;
        let kept = o.price(&o.carried[0]).unwrap().increment;
        assert!((kept - single).abs() < 1e-12, "{kept} vs {single}");
        // determinism, and independence of the declared order (the two copies tie at exactly 0:
        // the tie goes to the one declared first, so it is the one REMOVED first)
        let o2 = Admission::ordered(&us, &dict, &qq, &nulls, 0.02);
        assert_eq!(o, o2);
        let rev = [dict[2].clone(), dict[1].clone(), dict[0].clone()];
        let o3 = Admission::ordered(&us, &rev, &qq, &nulls, 0.02);
        assert_eq!(o3.carried, vec!["carried".to_string()]);
        assert_eq!(o.carried, vec!["carried copy".to_string()]);
    }

    #[test]
    fn the_defect_rises_by_the_price_when_the_carried_column_is_dropped() {
        let (us, car, _, _) = synth(5, 8, 300);
        let qq = q(Folds::Units { k: 4 }, Ridge::ORDER1);
        let r = increment(&us, &car.cols, &qq);
        let with: Vec<Unit> =
            us.iter().zip(&car.cols).map(|(u, c)| Unit { kept: u.kept.hcat(c), target: u.target.clone() }).collect();
        let d_with = 1.0 - closure_r2(&with, &qq);
        let d_without = 1.0 - closure_r2(&us, &qq);
        assert!(close(d_without - d_with, r.increment, 1e-12), "{} vs {}", d_without - d_with, r.increment);
    }

    #[test]
    fn time_blocks_follow_linspace_and_drop_the_gap() {
        let (us, car, _, _) = synth(7, 3, 101);
        let qq = Question { lag: 5, horizon: Horizon::Future, folds: Folds::TimeBlocks { k: 5 }, ridge: Ridge::ORDER1 };
        let d = build(&us, Some(&car.cols), &qq);
        let (tr, te) = fold_rows(&d, &us, &qq, 1);
        // n = 96 origins, edges 0, 19, 38, ... ; block 1 = [19, 38); train = t < 14 or t >= 43
        let tt: Vec<usize> = te.iter().map(|&r| d.t[r]).collect();
        assert_eq!(*tt.iter().min().unwrap(), 19);
        assert_eq!(*tt.iter().max().unwrap(), 37);
        assert!(tr.iter().all(|&r| d.t[r] < 14 || d.t[r] >= 43));
        assert_eq!(te.len(), 3 * 19);
    }

    #[test]
    fn the_shuffled_dictionary_admits_nothing() {
        let (us, car, _, _) = synth(9, 12, 400);
        let mut rng = NumpyPcg64::new(4);
        let mut su = Vec::new();
        let mut sc = Vec::new();
        for (u, c) in us.iter().zip(&car.cols) {
            let p = rng.permutation(u.len());
            su.push(Unit { kept: u.kept.permute_rows(&p), target: u.target.permute_rows(&p) });
            sc.push(c.permute_rows(&p));
        }
        let adm = admit(&su, &[Candidate::new("carried", sc)], &q(Folds::Units { k: 4 }, Ridge::HBOND), &Nulls { shift: true, swap: None }, 0.02);
        assert!(adm.carried.is_empty());
        assert!(adm.prices[0].increment <= 0.01, "{}", adm.prices[0]);
    }

    #[test]
    fn certified_prices_drop_at_the_budget_and_never_drop_the_unbounded() {
        let names: Vec<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        let certs = vec![
            Certificate::Bound { change: 0.0, why: "outside the cone".into() },
            Certificate::Unbounded { why: "inside the cone".into() },
            Certificate::Bound { change: 1e-9, why: "a loose bound".into() },
        ];
        let adm = admit_certified(&names, &certs, 1e-12);
        assert_eq!(adm.dropped, vec!["a".to_string()]);
        assert_eq!(adm.carried, vec!["b".to_string(), "c".to_string()]);
        assert!(refuse_drop(&adm, "b", 1e300).is_err());
    }

    #[test]
    fn the_full_k_vamp_score_is_the_canonical_correlation_sum() {
        // B = A (lag 0): every canonical correlation is 1, the score is the dimension
        let mut rng = NumpyPcg64::new(1);
        let a = ou_field(500, 3, 2.0, &mut rng);
        let s = vamp_heldout_full(&a, &a, &a, &a);
        assert!(close(s, 3.0, 1e-4), "{s}");
    }
}
