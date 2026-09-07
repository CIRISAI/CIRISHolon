//! EDGE-0's instruments: what a surface looks like from the lattice, read five ways.
//!
//! `conformance/mesh/EDGE0_PREREG.md` is the freeze; `crate::edge` is the carrier. Nothing
//! here runs dynamics of its own — every routine reads a state or a window of steps the
//! carrier has already advanced — and every one of the five has a UNIT TEST ON A SCENE
//! WHOSE ANSWER IS KNOWN before the instrument sees it, which is the only way an instrument
//! is shown able to return an answer other than the one it was built to like.
//!
//! | | instrument | its known scene |
//! |---|---|---|
//! | (a) | [`block_density`] + [`bimodality`] — the block-density histogram and the DIP between its two modes | a planted two-mode histogram (dip 1) and a planted one-mode histogram (dip 0, unseparated) |
//! | (b) | [`Pressure`] — the momentum flux per cell, and [`line_fit`] for Laplace's law | one particle in one cell (`Π = c ⊗ c`, exactly), a uniform free gas (`p = ρ_move/2`), and a planted `Δp = σ/R` line |
//! | (c) | [`capillary`] — the height field's spectrum, `⟨\|ĥ_k\|²⟩ = kT/(σ L k²)` | a height field built FROM a chosen `σ`, recovered to machine precision |
//! | (d) | [`interface_width`] — the density profile's `tanh` fit | a planted `tanh` profile, its width and centre recovered |
//! | (e) | [`chart_view`] + [`edge_at`] — the block chart's edge through `holon_closure::Edge::at` | an empty lattice (the edge is EMPTY, not zero-width) and `b = L` (vacuous, labelled) |
//!
//! # Two temperatures, and the one this module uses
//!
//! [`capillary`] needs a MECHANICAL `kT`, and it takes one as an argument rather than
//! assuming it. [`ideal_gas_kt`] measures it on the carrier's own no-bond control as the
//! ratio of the scalar pressure to the MOVING mass density — the lattice gas's `c_s²`,
//! which is `1/2` for FHP by the hexagon's `Σ_d c_dα c_dβ = 3 δ_αβ` and is measured here
//! rather than typed. It is a different number from `EdgeRules::rent`, which is the bond's
//! dimensionless `E₀/kT_rent` and is not a mechanical temperature; the two are never mixed.
//!
//! **The fence this puts on the two σ readings.** `σ_L` comes out of Laplace's law in
//! lattice pressure × length and needs no `kT`. `σ_C` comes out of the capillary spectrum
//! and is `kT_mech / (slope · L)`, so any error in the `kT_mech` identification rescales
//! `σ_C` and only `σ_C`. A freeze that stakes the two against each other is staking that
//! identification too, and must say so.
//!
//! # Prior art, credited for the rules and not for the numbers
//!
//! * Appert & Zaleski, *Phys. Rev. Lett.* **64** (1990) 1 — a lattice gas with a cohesive
//!   rule separates into a dense phase and a sparse one with an emergent surface tension.
//! * Rothman & Keller, *J. Stat. Phys.* **52** (1988) 1119 — the immiscible lattice gas.
//! * Rothman & Zaleski, *Lattice-Gas Cellular Automata* (Cambridge, 1997) — the momentum
//!   flux tensor on a lattice gas, Laplace's law and the capillary spectrum as the two
//!   independent readings of σ, and a body force as a momentum bias.
//! * Frisch et al. 1987 — FHP-II/III and the rest particle, credited in `crate::edge`.

// The index IS the meaning here: a loop over `0..n` walks the cell's direction slots and the
// parallel role arrays are indexed by that same slot, so `enumerate` on one of them would name
// one array as the loop's subject and leave the others reading as incidental.
// `transport.rs` waives the same lint for the same reason.
// `!(x > bar)` is DELIBERATE where `x` may be `NaN`: a refused fit must fall on the failing
// side, and `NaN <= bar` is false while `!(NaN > bar)` is true. `holon-campaign` waives the
// same lint at the same two places for the same reason.
#![allow(
    clippy::needless_range_loop,
    clippy::manual_is_multiple_of,
    clippy::neg_cmp_op_on_partial_ord
)]

use crate::chart::{BlockChart, Field};
use crate::edge::{EdgeLattice, N_ORIENT};
use holon_closure::{BlockView, Edge};

// ─────────────────────────────────────────────── geometry on the hexagonal torus

/// The Euclidean position of a cell, in link units: `i · e₀ + j · e₁` where `e₀` and `e₁`
/// are the embeddings of the first two axial directions.
pub fn cell_position(l: usize, cell: usize, embed: &[[f64; 2]]) -> [f64; 2] {
    let (i, j) = ((cell / l) as f64, (cell % l) as f64);
    [i * embed[0][0] + j * embed[1][0], i * embed[0][1] + j * embed[1][1]]
}

/// Minimum-image distance between two cells on the torus, in link units.
///
/// The torus's period is `L·e₀` and `L·e₁`, which are NOT orthogonal, so the minimum image
/// is taken over the nine nearest lattice translates rather than by rounding each Cartesian
/// component — rounding the components of a non-orthogonal lattice is the classic way to get
/// a distance that is not the shortest one.
pub fn torus_distance(l: usize, a: usize, b: usize, embed: &[[f64; 2]]) -> f64 {
    let pa = cell_position(l, a, embed);
    let pb = cell_position(l, b, embed);
    let (dx, dy) = (pa[0] - pb[0], pa[1] - pb[1]);
    let mut best = f64::INFINITY;
    for m in -1i64..=1 {
        for n in -1i64..=1 {
            let sx = dx + l as f64 * (m as f64 * embed[0][0] + n as f64 * embed[1][0]);
            let sy = dy + l as f64 * (m as f64 * embed[0][1] + n as f64 * embed[1][1]);
            best = best.min(sx * sx + sy * sy);
        }
    }
    best.sqrt()
}

// ─────────────────────────────────────────────── (a) the density histogram

/// The per-slot occupancy of each `b × b` block, in `[0, 1]`, in row-major block order.
pub fn block_density(g: &EdgeLattice, b: usize) -> Vec<f64> {
    let chart = BlockChart::new(b, g.l).expect("b must divide L");
    let nb = chart.blocks_per_side();
    let mut out = vec![0.0f64; nb * nb];
    for i in 0..g.l {
        for j in 0..g.l {
            out[(i / b) * nb + (j / b)] += g.cells[i * g.l + j].count_ones() as f64;
        }
    }
    let per_block = (b * b * g.n) as f64;
    out.iter_mut().for_each(|v| *v /= per_block);
    out
}

/// The per-slot occupancy of each block counting only MOVING slots — the density a pressure
/// reading is proportional to, reported beside the total so a dense phase made of rest
/// particles is visible as such.
pub fn block_moving_density(g: &EdgeLattice, b: usize) -> Vec<f64> {
    let chart = BlockChart::new(b, g.l).expect("b must divide L");
    let nb = chart.blocks_per_side();
    let mask: u8 = ((1u16 << N_ORIENT) - 1) as u8;
    let mut out = vec![0.0f64; nb * nb];
    for i in 0..g.l {
        for j in 0..g.l {
            out[(i / b) * nb + (j / b)] += (g.cells[i * g.l + j] & mask).count_ones() as f64;
        }
    }
    let per_block = (b * b * N_ORIENT) as f64;
    out.iter_mut().for_each(|v| *v /= per_block);
    out
}

/// The histogram of a set of densities over `bins` equal bins of `[0, 1]`, and the DIP
/// between its two highest separated modes.
///
/// **The statistic, stated.** `p₁` is the fullest bin. `p₂` is the fullest bin at least
/// `min_separation` bins away from `p₁`. `v` is the emptiest bin strictly between them.
/// `dip = max(0, (min(h[p₁], h[p₂]) − h[v]) / min(h[p₁], h[p₂]))`, in `[0, 1]`: one when the
/// two modes are separated by an empty valley, zero when the valley is at least as full as
/// the smaller mode. A distribution with no second mode at that separation reports
/// `separated = false` and `dip = 0` — an absence, never a small number.
///
/// **The clamp at zero is load-bearing and was put there by a screen.** Without it the
/// statistic is unbounded below: on a unimodal histogram `p₂` is whatever bin happens to sit
/// `min_separation` away on the shoulder, the "valley" between it and the peak is FULLER
/// than `p₂`, and the ratio goes negative — the first EDGE-0 screen read dips of `−3.17` and
/// a floor with a standard deviation of `1.31`, which is a statistic saying nothing. A dip
/// is the depth of a valley below the shallower of two modes, and a valley that is not below
/// them is not a dip; zero is the right answer and the magnitude of the nonsense is not
/// information.
///
/// The statistic has a NOISE FLOOR and it is not typed: on a one-phase lattice the block
/// densities are a sum of independent slots and the histogram is unimodal, but a finite
/// sample still shows a dip. [`bimodality_floor`] measures that floor on the control the
/// freeze names, over seeds, and reports its spread.
#[derive(Clone, Debug)]
pub struct Bimodality {
    pub bins: usize,
    pub counts: Vec<u64>,
    pub samples: usize,
    pub min_separation: usize,
    pub peak_lo: usize,
    pub peak_hi: usize,
    pub valley: usize,
    /// The two modes' bin centres, in density units.
    pub mode_lo: f64,
    pub mode_hi: f64,
    pub dip: f64,
    pub separated: bool,
    pub mean: f64,
}

pub fn bimodality(values: &[f64], bins: usize, min_separation: usize) -> Bimodality {
    assert!(bins >= 3, "a histogram with fewer than three bins has no valley");
    let mut counts = vec![0u64; bins];
    for &v in values {
        let k = ((v.clamp(0.0, 1.0) * bins as f64) as usize).min(bins - 1);
        counts[k] += 1;
    }
    let centre = |k: usize| (k as f64 + 0.5) / bins as f64;
    let mean = if values.is_empty() {
        f64::NAN
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    };
    let p1 = (0..bins).max_by_key(|&k| counts[k]).unwrap();
    let far: Vec<usize> =
        (0..bins).filter(|&k| k.abs_diff(p1) >= min_separation).collect();
    let p2 = far.iter().copied().max_by_key(|&k| counts[k]);
    let (peak_lo, peak_hi, separated) = match p2 {
        Some(q) if counts[q] > 0 => (p1.min(q), p1.max(q), true),
        _ => (p1, p1, false),
    };
    if !separated {
        return Bimodality {
            bins,
            counts,
            samples: values.len(),
            min_separation,
            peak_lo,
            peak_hi,
            valley: peak_lo,
            mode_lo: centre(peak_lo),
            mode_hi: centre(peak_hi),
            dip: 0.0,
            separated,
            mean,
        };
    }
    let valley = (peak_lo + 1..peak_hi).min_by_key(|&k| counts[k]).unwrap_or(peak_lo);
    let m = counts[peak_lo].min(counts[peak_hi]) as f64;
    let dip = if m > 0.0 { ((m - counts[valley] as f64) / m).max(0.0) } else { 0.0 };
    Bimodality {
        bins,
        counts,
        samples: values.len(),
        min_separation,
        peak_lo,
        peak_hi,
        valley,
        mode_lo: centre(peak_lo),
        mode_hi: centre(peak_hi),
        dip,
        separated,
        mean,
    }
}

/// The `q`-quantile of a set of block densities, by linear interpolation between the two
/// nearest order statistics.
///
/// **Why the freeze reads quantiles beside the histogram's modes.** The mode finder in
/// [`bimodality`] takes its second peak at least `min_separation` bins from the first, so a
/// distribution with a long shoulder reports a "second mode" sitting exactly at that
/// separation — which is a fact about the constraint, not about the distribution. The
/// quantiles have no such knob: `q05` and `q95` are the sparse and dense phases' own
/// densities as the sample carries them, and their ratio against the SAME statistic on the
/// one-phase control is what a two-phase reading has to beat.
pub fn quantile(values: &[f64], q: f64) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    let x = q.clamp(0.0, 1.0) * (v.len() - 1) as f64;
    let lo = x.floor() as usize;
    let hi = x.ceil() as usize;
    v[lo] + (x - lo as f64) * (v[hi] - v[lo])
}

/// The two phases' own densities, and how much of the box the dense one holds.
///
/// **The rule, stated.** The threshold is the midpoint of the sample's `q05` and `q95`; the
/// sparse phase's density is the mean of the blocks below it and the dense phase's the mean of
/// those above, with the dense fraction reported beside. That is the standard two-phase
/// decomposition of a density histogram and it has one knob — the threshold rule — which is
/// written here and nowhere else.
///
/// It is what the freeze BUILDS its droplet and its slab at, so those scenes are built at the
/// carrier's OWN two densities rather than at a construction. On a single-phase sample the two
/// numbers come back close together and the dense fraction near a half, which is how a caller
/// sees that there was one phase and not two.
pub fn two_phase_densities(values: &[f64]) -> (f64, f64, f64) {
    if values.is_empty() {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    let cut = 0.5 * (quantile(values, 0.05) + quantile(values, 0.95));
    let hi: Vec<f64> = values.iter().copied().filter(|&v| v > cut).collect();
    let lo: Vec<f64> = values.iter().copied().filter(|&v| v <= cut).collect();
    let m = |v: &[f64]| if v.is_empty() { f64::NAN } else { v.iter().sum::<f64>() / v.len() as f64 };
    (m(&lo), m(&hi), hi.len() as f64 / values.len() as f64)
}

/// The variance of a set of block densities — the coexistence PRECURSOR, reported beside the
/// dip because it is far more sensitive and far less than a verdict.
///
/// A one-phase lattice's block densities are a sum of independent slots, so their variance is
/// the binomial one and shrinks like `1/b²`; two phases put weight at two densities and the
/// variance is the separation squared times the phase fractions. The RATIO of the two,
/// measured against the same one-phase control the dip's floor is measured on, moves long
/// before the histogram splits — which makes it the right thing for a screen to steer on and
/// the wrong thing for a gate to read, since a single large domain and two coexisting phases
/// raise it alike.
pub fn density_variance(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return f64::NAN;
    }
    let m = values.iter().sum::<f64>() / values.len() as f64;
    values.iter().map(|v| (v - m) * (v - m)).sum::<f64>() / (values.len() - 1) as f64
}

/// The dip's noise floor, measured over seeds on a ONE-PHASE control the caller supplies.
/// Returns `(mean, standard deviation, max)` of the dip over the readings.
pub fn bimodality_floor(dips: &[f64]) -> (f64, f64, f64) {
    let n = dips.len() as f64;
    if dips.is_empty() {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    let m = dips.iter().sum::<f64>() / n;
    let var = dips.iter().map(|d| (d - m) * (d - m)).sum::<f64>() / n.max(1.0);
    (m, var.sqrt(), dips.iter().cloned().fold(f64::NEG_INFINITY, f64::max))
}

// ─────────────────────────────────────────────── (b) the pressure tensor

/// The momentum flux tensor of a region, per cell per step.
///
/// **The definition, stated.** The flux of momentum component `β` through a surface with
/// normal `α̂` is `Σ_particles Δx_α · p_β` per unit area per unit time, and on the lattice
/// `Δx` is the particle's ACTUAL displacement this step and `p` is `c[label]`. For a free
/// particle `Δx = p` and the tensor is the textbook `Σ_d c_dα c_dβ n_d`; for a bonded
/// particle the two differ, which is exactly how the cohesion enters the pressure. The
/// carrier accumulates it per cell during streaming, reading the displacement off the
/// joint-move pass's own claim table (`crate::edge`), so there is no second copy of the
/// rule that says where a particle went.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pressure {
    pub xx: f64,
    pub xy: f64,
    pub yx: f64,
    pub yy: f64,
    pub cells: usize,
    pub steps: u64,
}

impl Pressure {
    /// The isotropic pressure `½ tr Π`.
    pub fn scalar(&self) -> f64 {
        0.5 * (self.xx + self.yy)
    }
    /// How far from isotropic the reading is, relative to the scalar — reported beside every
    /// pressure so an anisotropic reading cannot be quoted as a pressure without saying so.
    pub fn anisotropy(&self) -> f64 {
        let s = self.scalar();
        if s == 0.0 {
            return f64::NAN;
        }
        ((self.xx - self.yy).abs().max(self.xy.abs()).max(self.yx.abs())) / s.abs()
    }
}

/// The accumulated flux over the cells a predicate keeps.
pub fn pressure_over(g: &EdgeLattice, keep: impl Fn(usize) -> bool) -> Pressure {
    let flux = g.flux.as_ref().expect("a pressure reading on a carrier with no flux accumulator");
    let (mut xx, mut xy, mut yx, mut yy) = (0.0, 0.0, 0.0, 0.0);
    let mut cells = 0usize;
    for (c, f) in flux.iter().enumerate() {
        if !keep(c) {
            continue;
        }
        xx += f[0];
        xy += f[1];
        yx += f[2];
        yy += f[3];
        cells += 1;
    }
    let d = (cells as f64) * (g.flux_steps.max(1) as f64);
    Pressure { xx: xx / d, xy: xy / d, yx: yx / d, yy: yy / d, cells, steps: g.flux_steps }
}

/// The whole lattice's pressure.
pub fn pressure(g: &EdgeLattice) -> Pressure {
    pressure_over(g, |_| true)
}

/// The mechanical `kT` of this carrier, MEASURED: the scalar pressure over the moving mass
/// density, on whatever state the caller has accumulated. On a free gas this is FHP's `c_s²`
/// and equals `1/2` exactly; on a bonded one it is not a temperature and the caller must not
/// use it as one, which is why the routine takes the state and returns a ratio rather than
/// pretending to know which state it was handed.
pub fn ideal_gas_kt(g: &EdgeLattice) -> f64 {
    let p = pressure(g);
    let mask: u8 = ((1u16 << N_ORIENT) - 1) as u8;
    let moving: usize = g.cells.iter().map(|&s| (s & mask).count_ones() as usize).sum();
    let rho = moving as f64 / g.cells.len() as f64;
    if rho == 0.0 {
        f64::NAN
    } else {
        p.scalar() / rho
    }
}

/// A least-squares straight line, with the coefficient of determination beside it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearFit {
    pub slope: f64,
    pub intercept: f64,
    pub r2: f64,
    pub n: usize,
}

pub fn line_fit(x: &[f64], y: &[f64]) -> LinearFit {
    assert_eq!(x.len(), y.len());
    let n = x.len();
    if n < 2 {
        return LinearFit { slope: f64::NAN, intercept: f64::NAN, r2: f64::NAN, n };
    }
    let nf = n as f64;
    let mx = x.iter().sum::<f64>() / nf;
    let my = y.iter().sum::<f64>() / nf;
    let sxx: f64 = x.iter().map(|v| (v - mx) * (v - mx)).sum();
    let sxy: f64 = x.iter().zip(y).map(|(a, b)| (a - mx) * (b - my)).sum();
    let syy: f64 = y.iter().map(|v| (v - my) * (v - my)).sum();
    let slope = if sxx == 0.0 { f64::NAN } else { sxy / sxx };
    let intercept = my - slope * mx;
    let r2 = if sxx == 0.0 || syy == 0.0 { f64::NAN } else { sxy * sxy / (sxx * syy) };
    LinearFit { slope, intercept, r2, n }
}

/// Laplace's law in two dimensions: `Δp = σ / R`. The slope of the pressure jump against
/// `1/R` over a set of radii IS `σ_L`, and the fit's `R²` is reported so "one slope" is a
/// reading and not an assumption.
pub fn laplace_sigma(radii: &[f64], dp: &[f64]) -> LinearFit {
    let inv: Vec<f64> = radii.iter().map(|r| 1.0 / r).collect();
    line_fit(&inv, dp)
}

// ─────────────────────────────────────────────── (c) the capillary spectrum

/// The capillary-wave reading of the surface tension.
#[derive(Clone, Debug)]
pub struct Capillary {
    /// The wavenumbers used, `k = 2π m / L`.
    pub k: Vec<f64>,
    /// `⟨|ĥ_k|²⟩` at each of them.
    pub power: Vec<f64>,
    /// The fit of `power` against `1/k²`.
    pub fit: LinearFit,
    /// `σ_C = kT / (slope · L)`.
    pub sigma: f64,
    pub kt: f64,
    pub l: usize,
    pub frames: usize,
}

/// `⟨|ĥ_k|²⟩` for one height field, with `ĥ_k = (1/L) Σ_x h(x) e^{−2πikx/L}`.
///
/// The `1/L` in the transform is part of the convention the formula below is written in and
/// is stated because a different one moves `σ` by a factor of `L`.
pub fn height_power(h: &[f64]) -> Vec<f64> {
    let l = h.len();
    let mut out = vec![0.0f64; l];
    for (m, o) in out.iter_mut().enumerate() {
        let (mut re, mut im) = (0.0f64, 0.0f64);
        let w = -2.0 * core::f64::consts::PI * m as f64 / l as f64;
        for (x, &v) in h.iter().enumerate() {
            let t = w * x as f64;
            re += v * t.cos();
            im += v * t.sin();
        }
        re /= l as f64;
        im /= l as f64;
        *o = re * re + im * im;
    }
    out
}

/// The capillary spectrum over a set of frames: `⟨|ĥ_k|²⟩ = kT / (σ L k²)`, fitted against
/// `1/k²` over the mode band `m ∈ [m_min, m_max]`.
///
/// The band is the caller's because it is a decision: the smallest `m` is the box-sized mode
/// (one wavelength across the interface) and the largest is where the height field stops
/// being a smooth function of `x` — a mode of a few lattice cells is not a capillary wave.
pub fn capillary(frames: &[Vec<f64>], kt: f64, m_min: usize, m_max: usize) -> Capillary {
    assert!(!frames.is_empty(), "a spectrum needs at least one frame");
    let l = frames[0].len();
    assert!(m_min >= 1 && m_max < l / 2 && m_min <= m_max, "the mode band is outside the field");
    let mut acc = vec![0.0f64; l];
    for f in frames {
        assert_eq!(f.len(), l, "the frames are not one field");
        for (a, p) in acc.iter_mut().zip(height_power(f)) {
            *a += p;
        }
    }
    let nf = frames.len() as f64;
    let mut k = Vec::new();
    let mut power = Vec::new();
    for m in m_min..=m_max {
        k.push(2.0 * core::f64::consts::PI * m as f64 / l as f64);
        power.push(acc[m] / nf);
    }
    let inv_k2: Vec<f64> = k.iter().map(|kk| 1.0 / (kk * kk)).collect();
    let fit = line_fit(&inv_k2, &power);
    let sigma = kt / (fit.slope * l as f64);
    Capillary { k, power, fit, sigma, kt, l, frames: frames.len() }
}

// ─────────────────────────────────────────────── (d) the interface's profile and width

/// The density profile along the second axial index, averaged over the first: `ρ(j)`.
pub fn profile_j(g: &EdgeLattice) -> Vec<f64> {
    let l = g.l;
    let mut out = vec![0.0f64; l];
    for i in 0..l {
        for j in 0..l {
            out[j] += g.cells[i * l + j].count_ones() as f64;
        }
    }
    let per = (l * g.n) as f64;
    out.iter_mut().for_each(|v| *v /= per);
    out
}

/// The density of each column, `ρ(i, j)` with `i` the column index — the field the height
/// function is read off.
pub fn column_profiles(g: &EdgeLattice) -> Vec<Vec<f64>> {
    let l = g.l;
    let per = g.n as f64;
    (0..l)
        .map(|i| (0..l).map(|j| g.cells[i * l + j].count_ones() as f64 / per).collect())
        .collect()
}

/// A `tanh` interface fitted to a profile: `ρ(j) = ½(ρ_hi + ρ_lo) − ½(ρ_hi − ρ_lo)·tanh((j − j₀)/w)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InterfaceFit {
    pub rho_hi: f64,
    pub rho_lo: f64,
    pub centre: f64,
    pub width: f64,
    /// Root-mean-square residual of the fit, in density units.
    pub rms: f64,
    pub points: usize,
}

/// Fit the `tanh` on a window `[j0, j1)` of a profile, with the two plateau densities SUPPLIED
/// (they are read off the two phases, not fitted, so the width is the only free shape).
///
/// The fit is a scan over centre and width — a grid then two refinements — because the
/// function is two-parameter and a scan cannot land in a local minimum a gradient method
/// would. `width` is in cells of the `j` index.
pub fn interface_width(
    profile: &[f64],
    j0: usize,
    j1: usize,
    rho_hi: f64,
    rho_lo: f64,
) -> InterfaceFit {
    assert!(j1 > j0 + 3 && j1 <= profile.len(), "the fit window is too short");
    let pts: Vec<(f64, f64)> =
        (j0..j1).map(|j| (j as f64, profile[j])).collect();
    let amp = 0.5 * (rho_hi - rho_lo);
    let mid = 0.5 * (rho_hi + rho_lo);
    let model = |c: f64, w: f64, x: f64| mid - amp * ((x - c) / w).tanh();
    let cost = |c: f64, w: f64| -> f64 {
        pts.iter().map(|&(x, y)| { let r = y - model(c, w, x); r * r }).sum::<f64>()
    };
    let (mut c_lo, mut c_hi) = (j0 as f64, (j1 - 1) as f64);
    let (mut w_lo, mut w_hi) = (0.25f64, (j1 - j0) as f64);
    let (mut best_c, mut best_w, mut best) = (0.5 * (c_lo + c_hi), 1.0, f64::INFINITY);
    for _ in 0..4 {
        let steps = 60usize;
        for a in 0..=steps {
            let c = c_lo + (c_hi - c_lo) * a as f64 / steps as f64;
            for b in 0..=steps {
                let w = w_lo + (w_hi - w_lo) * b as f64 / steps as f64;
                if w <= 0.0 {
                    continue;
                }
                let e = cost(c, w);
                if e < best {
                    best = e;
                    best_c = c;
                    best_w = w;
                }
            }
        }
        let (dc, dw) = ((c_hi - c_lo) / 20.0, (w_hi - w_lo) / 20.0);
        c_lo = best_c - dc;
        c_hi = best_c + dc;
        w_lo = (best_w - dw).max(1e-3);
        w_hi = best_w + dw;
    }
    InterfaceFit {
        rho_hi,
        rho_lo,
        centre: best_c,
        width: best_w,
        rms: (best / pts.len() as f64).sqrt(),
        points: pts.len(),
    }
}

/// The height of the interface in each column: the interpolated `j` at which the column's
/// density crosses `ρ_mid`, searched over the window `[j0, j1)` and taken at the FIRST
/// crossing from the dense side.
///
/// `None` for a column with no crossing in the window — reported as an absence, never as a
/// window edge, because a column with no interface in it is a fact about the scene.
pub fn height_field(
    columns: &[Vec<f64>],
    j0: usize,
    j1: usize,
    rho_mid: f64,
    dense_is_low_j: bool,
) -> Vec<Option<f64>> {
    columns
        .iter()
        .map(|col| {
            let range: Vec<usize> = if dense_is_low_j {
                (j0..j1.saturating_sub(1)).collect()
            } else {
                (j0..j1.saturating_sub(1)).rev().collect()
            };
            for j in range {
                let (a, b) = (col[j], col[j + 1]);
                let crosses = if dense_is_low_j { a >= rho_mid && b < rho_mid } else { a < rho_mid && b >= rho_mid };
                if crosses {
                    let t = if (a - b).abs() < 1e-12 { 0.0 } else { (a - rho_mid) / (a - b) };
                    return Some(j as f64 + t);
                }
            }
            None
        })
        .collect()
}

/// A height field with its mean removed and its gaps named — what the spectrum takes.
/// `None` when any column had no crossing: a spectrum on a field with holes in it is not a
/// spectrum, and filling the holes would be inventing the interface where it was not found.
pub fn centred_height(h: &[Option<f64>]) -> Option<Vec<f64>> {
    if h.iter().any(|x| x.is_none()) {
        return None;
    }
    let v: Vec<f64> = h.iter().map(|x| x.unwrap()).collect();
    let m = v.iter().sum::<f64>() / v.len() as f64;
    Some(v.into_iter().map(|x| x - m).collect())
}

// ─────────────────────────────────────────────── (f) the droplet's survival

/// How long one bonded droplet keeps half of the cluster it started with.
///
/// **The statistic, stated.** A droplet is built dense in a sparse background, the carrier is
/// settled for `settle` steps so the bond rule can bind it, and the largest connected
/// component of the bond graph at that instant is the START. The lifetime is the first step
/// at which the largest component falls strictly under half the start, sampled every `stride`
/// steps and capped at `cap`. A run that reaches the cap is reported CAPPED — a cap is a
/// refusal, never a number — so "it survived the whole run" and "it died at the cap" cannot
/// be confused.
///
/// **Two absences are named rather than counted as short lifetimes.** A scene with no
/// particles has no droplet (`empty`), and a scene whose bond rule never bound anything
/// starts with a largest component of ONE, which is a fact about the rule and not a lifetime
/// (`unbound`). In both cases `steps` is `None`.
#[derive(Clone, Debug)]
pub struct DropletLife {
    pub start_largest: usize,
    pub particles: usize,
    /// The first sampled step at which the largest component was under half the start.
    pub steps: Option<usize>,
    pub capped: bool,
    pub empty: bool,
    pub unbound: bool,
    /// The largest component at each sample, so the decay's shape is in the record and not
    /// only its crossing.
    pub trace: Vec<usize>,
    pub settle: usize,
    pub stride: usize,
    pub cap: usize,
    pub ledger_exact: bool,
}

impl DropletLife {
    /// The lifetime as a number of steps for a record, or `NaN` where there was no droplet to
    /// lose or the run reached its cap — a cap and a death are different readings.
    pub fn lifetime(&self) -> f64 {
        match self.steps {
            Some(s) if !self.empty && !self.unbound => s as f64,
            _ => f64::NAN,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn droplet_lifetime(
    l: usize,
    seed: u64,
    r: f64,
    d_in: f64,
    d_out: f64,
    rules: crate::edge::EdgeRules,
    settle: usize,
    stride: usize,
    cap: usize,
) -> DropletLife {
    let mut g = droplet_scene(l, seed, r, d_in, d_out, rules);
    g.audit_every_step = false;
    for _ in 0..settle {
        g.step();
    }
    g.audit_every_step = true;
    g.reset_initial();
    let start = g.phase();
    let particles = start.nodes;
    let start_largest = start.largest;
    let empty = particles == 0;
    let unbound = !empty && start_largest <= 1;
    let mut trace = vec![start_largest];
    let mut steps = None;
    let mut done = 0usize;
    if !empty && !unbound {
        while done < cap {
            for _ in 0..stride {
                g.step();
            }
            done += stride;
            let largest = g.phase().largest;
            trace.push(largest);
            if 2 * largest < start_largest {
                steps = Some(done);
                break;
            }
        }
    }
    DropletLife {
        start_largest,
        particles,
        steps,
        capped: steps.is_none() && !empty && !unbound,
        empty,
        unbound,
        trace,
        settle,
        stride,
        cap,
        ledger_exact: g.audit.all_exact(),
    }
}

// ─────────────────────────────────────────────── (e) the block chart's edge

/// A block chart with the per-cell split reading beside it — `holon_closure::BlockView` on
/// THIS carrier's dynamics, so the edge is read against the motion that made it.
#[derive(Clone, Debug)]
pub struct EdgeChartView {
    pub chart: BlockChart,
    /// One flag per BLOCK: does a fiber-preserving perturbation anywhere inside it change the
    /// stepped chart? The negation of `Core/Closure.lean`'s `ViewClosed`, localised.
    pub splits: Vec<bool>,
    pub l: usize,
    /// Cells actually probed, so a reading on a lattice with nothing movable in it is
    /// visible as such rather than as a clean zero.
    pub probes: u64,
}

impl BlockView for EdgeChartView {
    fn b(&self) -> usize {
        self.chart.b
    }
    fn cells(&self) -> usize {
        self.splits.len()
    }
    fn splits(&self, cell: usize) -> bool {
        self.splits[cell]
    }
    fn position(&self, cell: usize) -> [f64; 2] {
        let nb = self.chart.blocks_per_side();
        let half = self.chart.b as f64 / 2.0;
        [
            (cell % nb) as f64 * self.chart.b as f64 + half,
            (cell / nb) as f64 * self.chart.b as f64 + half,
        ]
    }
    fn period(&self) -> Option<[f64; 2]> {
        Some([self.l as f64, self.l as f64])
    }
    fn vacuous(&self) -> bool {
        self.chart.is_vacuous_by_conservation()
    }
}

fn field_eq(a: &Field, b: &Field) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x == y)
}

/// Read WHICH blocks the dynamics splits, one micro-cell at a time.
///
/// The expensive route, and the only one that answers this question: `probe.rs` reports a
/// RATE over a population, and an edge needs the SET. One exhaustive single-cell probe per
/// movable cell, short-circuited once a block has its witness. The perturbation is the fiber
/// move, which leaves `(N, P)` — hence every chart at once — exactly unchanged, so the only
/// thing that can change the stepped chart is the dynamics splitting the fiber.
pub fn chart_view(base: &EdgeLattice, b: usize, steps: usize) -> EdgeChartView {
    let chart = BlockChart::new(b, base.l).expect("b must divide L");
    let nb = chart.blocks_per_side();
    let mut splits = vec![false; nb * nb];
    let mut probes = 0u64;
    let snap = base.snapshot();
    let movable = base.model().movable();
    let mut g = base.clone();
    g.audit_every_step = false;
    for _ in 0..steps {
        g.step();
    }
    let reference = g.chart(b).expect("b divides L");
    for c in 0..base.cells.len() {
        let block = (c / base.l / b) * nb + (c % base.l) / b;
        if splits[block] || !movable.contains(&base.cells[c]) {
            continue;
        }
        g.restore(&snap);
        if !g.fiber_perturb(c) {
            continue;
        }
        probes += 1;
        for _ in 0..steps {
            g.step();
        }
        if !field_eq(&g.chart(b).expect("b divides L"), &reference) {
            splits[block] = true;
        }
    }
    EdgeChartView { chart, splits, l: base.l, probes }
}

/// The edge at one resolution, through `holon-closure`'s own reading.
pub fn edge_at(base: &EdgeLattice, b: usize, steps: usize) -> (Edge, EdgeChartView) {
    let v = chart_view(base, b, steps);
    (Edge::at(&v), v)
}

// ─────────────────────────────────────────────── scenes the instruments are read on

/// A dense disc of radius `r` in a sparse background, on the hexagonal torus.
pub fn droplet_scene(
    l: usize,
    seed: u64,
    r: f64,
    d_in: f64,
    d_out: f64,
    rules: crate::edge::EdgeRules,
) -> EdgeLattice {
    let embed = crate::edge::hex_embed(&crate::state::Model::fhp7());
    let centre = (l / 2) * l + l / 2;
    EdgeLattice::seeded_by(l, seed, rules, move |i, j| {
        let c = i * l + j;
        if torus_distance(l, c, centre, &embed) <= r {
            d_in
        } else {
            d_out
        }
    })
}

/// A dense slab spanning the first axial index, with two flat interfaces at `j = j0` and
/// `j = j1`. The interface normal is the `j` axis, which is the axis the profile and the
/// height field are read along.
pub fn slab_scene(
    l: usize,
    seed: u64,
    j0: usize,
    j1: usize,
    d_in: f64,
    d_out: f64,
    rules: crate::edge::EdgeRules,
) -> EdgeLattice {
    EdgeLattice::seeded_by(l, seed, rules, move |_, j| if j >= j0 && j < j1 { d_in } else { d_out })
}

/// Cells within `r` of the droplet's centre, for a pressure reading INSIDE it.
pub fn within(l: usize, centre: usize, r: f64, embed: &[[f64; 2]]) -> impl Fn(usize) -> bool + '_ {
    let embed = embed.to_vec();
    move |c: usize| torus_distance(l, c, centre, &embed) <= r
}

/// Cells beyond `r` of the droplet's centre, for a pressure reading OUTSIDE it.
pub fn beyond(l: usize, centre: usize, r: f64, embed: &[[f64; 2]]) -> impl Fn(usize) -> bool + '_ {
    let embed = embed.to_vec();
    move |c: usize| torus_distance(l, c, centre, &embed) > r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::{EdgeLattice, EdgeRules};
    use crate::state::Model;

    fn amp() -> [f64; N_ORIENT] {
        [1.0, 1.337178, 1.288496, 1.420097, 1.288496, 1.337178]
    }

    fn free_rules() -> EdgeRules {
        let mut r = EdgeRules::edge0(amp(), 1.0);
        r.bonds_enabled = false;
        r
    }

    // ---------------------------------------------------------------- (a)

    /// The dip on two PLANTED histograms whose answers are arithmetic: two equal modes with
    /// an empty valley read exactly 1, and one mode reads unseparated and 0.
    #[test]
    fn the_dip_is_one_on_two_planted_modes_and_zero_on_one() {
        let two: Vec<f64> = (0..500)
            .map(|k| if k % 2 == 0 { 0.105 } else { 0.605 })
            .collect();
        let b = bimodality(&two, 100, 10);
        assert!(b.separated);
        assert_eq!(b.peak_lo, 10);
        assert_eq!(b.peak_hi, 60);
        assert_eq!(b.counts[b.valley], 0);
        assert_eq!(b.dip, 1.0);
        assert!((b.mode_lo - 0.105).abs() < 0.01 && (b.mode_hi - 0.605).abs() < 0.01);

        let one: Vec<f64> = (0..500).map(|_| 0.305).collect();
        let u = bimodality(&one, 100, 10);
        assert!(!u.separated, "a single mode was read as two");
        assert_eq!(u.dip, 0.0);

        // A valley as full as the smaller mode reads EXACTLY zero, which is the statistic's
        // own floor and not a special case: a uniform spread over every bin.
        let flat: Vec<f64> = (0..1000).map(|k| (k % 100) as f64 * 0.01 + 0.005).collect();
        let f = bimodality(&flat, 100, 10);
        assert!(f.separated, "a uniform spread has two bins ten apart");
        assert_eq!(f.dip, 0.0, "a uniform spread read a dip");

        // The clamp: a shoulder whose "valley" is FULLER than the second peak reads zero,
        // never a negative number. This is the shape the first screen read as -3.17.
        let shoulder: Vec<f64> = (0..600)
            .map(|k| match k % 6 {
                0..=3 => 0.305,
                4 => 0.345,
                _ => 0.405,
            })
            .collect();
        let sh = bimodality(&shoulder, 100, 10);
        assert!(sh.dip >= 0.0, "the dip went negative at {}", sh.dip);
    }

    /// The quantile on a KNOWN sample: a uniform ramp's quantiles are its own positions, and
    /// the routine interpolates between order statistics rather than rounding to one.
    #[test]
    fn the_quantile_reads_a_known_sample() {
        let v: Vec<f64> = (0..=100).map(|k| k as f64 / 100.0).collect();
        assert!((quantile(&v, 0.0) - 0.0).abs() < 1e-12);
        assert!((quantile(&v, 0.05) - 0.05).abs() < 1e-12);
        assert!((quantile(&v, 0.5) - 0.5).abs() < 1e-12);
        assert!((quantile(&v, 0.95) - 0.95).abs() < 1e-12);
        assert!((quantile(&v, 1.0) - 1.0).abs() < 1e-12);
        // Interpolation, not rounding: two points give the midpoint at q = 0.5.
        assert!((quantile(&[0.0, 1.0], 0.5) - 0.5).abs() < 1e-12);
        assert!(quantile(&[], 0.5).is_nan(), "an empty sample was given a quantile");
        // Order does not matter, and the sample is not mutated.
        let scrambled = [0.9f64, 0.1, 0.5, 0.3, 0.7];
        assert!((quantile(&scrambled, 0.5) - 0.5).abs() < 1e-12);
        assert_eq!(scrambled[0], 0.9);
        // Two planted phases: the quantile ratio sees them where a single phase reads ~1.
        let two: Vec<f64> = (0..1000).map(|k| if k % 2 == 0 { 0.005 } else { 0.085 }).collect();
        let one: Vec<f64> = (0..1000).map(|k| 0.045 + 0.001 * ((k % 11) as f64 - 5.0)).collect();
        assert!(quantile(&two, 0.95) / quantile(&two, 0.05) > 10.0);
        assert!(quantile(&one, 0.95) / quantile(&one, 0.05) < 1.3);
    }

    /// The two-phase decomposition on a PLANTED sample: two densities and a dense fraction
    /// come back as they were put in, and a single phase reads its two numbers together.
    #[test]
    fn the_two_phase_decomposition_returns_what_was_planted() {
        // 30 % of the blocks at 0.09, 70 % at 0.004.
        let mut v: Vec<f64> = Vec::new();
        for k in 0..1000 {
            v.push(if k % 10 < 3 { 0.09 } else { 0.004 });
        }
        let (lo, hi, frac) = two_phase_densities(&v);
        assert!((lo - 0.004).abs() < 1e-9, "sparse phase {lo}");
        assert!((hi - 0.09).abs() < 1e-9, "dense phase {hi}");
        assert!((frac - 0.30).abs() < 1e-9, "dense fraction {frac}");

        // One phase: the two numbers sit together and the fraction is near a half.
        let one: Vec<f64> = (0..1000).map(|k| 0.02 + 0.0005 * ((k % 21) as f64 - 10.0)).collect();
        let (a, b, f) = two_phase_densities(&one);
        assert!(b - a < 0.01, "a single phase was split into {a} and {b}");
        assert!((f - 0.5).abs() < 0.1, "a single phase's dense fraction is {f}");

        let (x, y, z) = two_phase_densities(&[]);
        assert!(x.is_nan() && y.is_nan() && z.is_nan(), "an empty sample was given densities");
    }

    /// The floor is not typed: a one-phase lattice's block densities read a dip, and the
    /// instrument measures it over seeds.
    #[test]
    fn the_one_phase_control_has_a_measured_floor() {
        let dips: Vec<f64> = (0..6u64)
            .map(|s| {
                let g = EdgeLattice::seeded(48, 0xF100 ^ s, 0.3, free_rules());
                bimodality(&block_density(&g, 8), 100, 10).dip
            })
            .collect();
        let (m, sd, max) = bimodality_floor(&dips);
        assert!(m.is_finite() && sd.is_finite() && max >= m);
        assert!(m < 0.9, "a one-phase control read a dip of {m}, which is not a floor");
    }

    // ---------------------------------------------------------------- (b)

    /// ONE particle in ONE cell: the momentum flux tensor is `c ⊗ c` for its own direction,
    /// exactly, with no bond and no gravity to change the displacement.
    #[test]
    fn one_particle_reads_its_own_outer_product() {
        let embed = crate::edge::hex_embed(&Model::fhp7());
        for d in 0..N_ORIENT {
            let mut g = EdgeLattice::seeded_by(8, 5, free_rules(), |_, _| 0.0).with_flux();
            g.cells[20] = 1 << d;
            g.orient[20 * g.n + d] = 0;
            g.reset_initial();
            g.step();
            let p = pressure_over(&g, |c| c == 20);
            let c = embed[d];
            assert!((p.xx - c[0] * c[0]).abs() < 1e-12, "xx at d={d}");
            assert!((p.yy - c[1] * c[1]).abs() < 1e-12, "yy at d={d}");
            assert!((p.xy - c[0] * c[1]).abs() < 1e-12, "xy at d={d}");
            assert!((p.scalar() - 0.5).abs() < 1e-12, "a unit direction carries ½ tr = ½");
        }
        // The rest particle carries no flux at all — the reading that says a dense phase of
        // rest particles exerts no kinetic pressure.
        let mut g = EdgeLattice::seeded_by(8, 5, free_rules(), |_, _| 0.0).with_flux();
        let rest = g.rest.unwrap();
        g.cells[20] = 1 << rest;
        g.orient[20 * g.n + rest] = 0;
        g.reset_initial();
        g.step();
        let p = pressure_over(&g, |c| c == 20);
        assert_eq!(p.scalar(), 0.0);
    }

    /// A uniform FREE gas reads the lattice gas's own equation of state: the pressure tensor
    /// is isotropic and `p / ρ_moving = 1/2` — FHP's `c_s²`, which the hexagon's
    /// `Σ_d c_dα c_dβ = 3 δ_αβ` forces and which `ideal_gas_kt` measures rather than types.
    #[test]
    fn a_free_gas_reads_the_lattice_sound_speed() {
        let mut g = EdgeLattice::seeded(48, 0x1DEA, 0.3, free_rules()).with_flux();
        for _ in 0..50 {
            g.step();
        }
        g.clear_flux();
        for _ in 0..200 {
            g.step();
        }
        let p = pressure(&g);
        assert!(p.anisotropy() < 0.02, "a free gas read anisotropy {}", p.anisotropy());
        let kt = ideal_gas_kt(&g);
        assert!((kt - 0.5).abs() < 0.01, "c_s^2 read {kt}, not 1/2");
        assert!(g.audit.all_exact());
    }

    /// Laplace's law recovered from a PLANTED line: `Δp = σ/R` with a chosen `σ` and a
    /// chosen offset, over the radii the freeze would use.
    #[test]
    fn the_laplace_fit_recovers_a_planted_sigma() {
        let radii = [6.0f64, 8.0, 10.0, 13.0, 16.0, 20.0];
        let sigma = 0.037_5;
        let offset = -0.002;
        let dp: Vec<f64> = radii.iter().map(|r| sigma / r + offset).collect();
        let f = laplace_sigma(&radii, &dp);
        assert!((f.slope - sigma).abs() < 1e-12, "slope {} against {sigma}", f.slope);
        assert!((f.intercept - offset).abs() < 1e-12);
        assert!((f.r2 - 1.0).abs() < 1e-12);
        // And it does NOT report a line where there is none: a constant jump has zero slope
        // and no correlation with 1/R.
        let flat: Vec<f64> = radii.iter().map(|_| 0.01).collect();
        let g = laplace_sigma(&radii, &flat);
        assert!(g.slope.abs() < 1e-12);
        assert!(!(g.r2 > 0.5), "a flat set fitted a Laplace line at R2 = {}", g.r2);
    }

    // ---------------------------------------------------------------- (c)

    /// The capillary instrument on a height field built FROM a chosen `σ`: the amplitudes
    /// are set to `kT/(σ L k²)` exactly and the fit must return that `σ`.
    #[test]
    fn the_capillary_fit_recovers_a_planted_sigma() {
        let l = 128usize;
        let kt = 0.5f64;
        let sigma = 0.08f64;
        let mut h = vec![0.0f64; l];
        // A real field with |h_k|^2 = kT/(sigma L k^2) at each mode of the band, built as a
        // sum of cosines with the amplitudes that convention forces: for a real field the
        // mode m and L-m share the power, and a cosine of amplitude A has |h_m|^2 = A^2/4.
        for m in 1..l / 2 {
            let k = 2.0 * core::f64::consts::PI * m as f64 / l as f64;
            let power = kt / (sigma * l as f64 * k * k);
            let a = 2.0 * power.sqrt();
            let phase = 0.7 * m as f64;
            for (x, hv) in h.iter_mut().enumerate() {
                *hv += a * (k * x as f64 + phase).cos();
            }
        }
        let c = capillary(&[h], kt, 2, 20);
        assert!((c.sigma - sigma).abs() / sigma < 1e-9, "sigma {} against {sigma}", c.sigma);
        assert!((c.fit.r2 - 1.0).abs() < 1e-9, "R2 {}", c.fit.r2);
        // A stiffer interface must read a larger sigma, so the instrument is shown able to
        // return more than one answer.
        let stiff: Vec<f64> = c.power.iter().map(|p| p / 4.0).collect();
        let inv: Vec<f64> = c.k.iter().map(|k| 1.0 / (k * k)).collect();
        let f2 = line_fit(&inv, &stiff);
        assert!((kt / (f2.slope * l as f64) - 4.0 * sigma).abs() / sigma < 1e-9);
    }

    // ---------------------------------------------------------------- (d)

    /// The width fit on a PLANTED `tanh`: the width and the centre come back.
    #[test]
    fn the_width_fit_recovers_a_planted_tanh() {
        for (w, c) in [(1.5f64, 40.3f64), (4.0, 33.0), (0.8, 45.7)] {
            let (hi, lo) = (0.62f64, 0.08f64);
            let profile: Vec<f64> = (0..64)
                .map(|j| 0.5 * (hi + lo) - 0.5 * (hi - lo) * ((j as f64 - c) / w).tanh())
                .collect();
            let f = interface_width(&profile, 20, 62, hi, lo);
            assert!((f.width - w).abs() < 0.05, "width {} against {w}", f.width);
            assert!((f.centre - c).abs() < 0.1, "centre {} against {c}", f.centre);
            assert!(f.rms < 1e-3, "rms {}", f.rms);
        }
    }

    /// The height field on a PLANTED interface: each column's crossing comes back where it
    /// was put, and a column with no crossing is reported ABSENT rather than clamped.
    #[test]
    fn the_height_field_finds_a_planted_crossing_and_names_a_missing_one() {
        let l = 32usize;
        let (hi, lo) = (0.6f64, 0.1f64);
        let mid = 0.5 * (hi + lo);
        let want: Vec<f64> = (0..l).map(|i| 16.0 + 2.0 * (i as f64 * 0.3).sin()).collect();
        let columns: Vec<Vec<f64>> = (0..l)
            .map(|i| {
                (0..l)
                    .map(|j| 0.5 * (hi + lo) - 0.5 * (hi - lo) * ((j as f64 - want[i]) / 0.6).tanh())
                    .collect()
            })
            .collect();
        let h = height_field(&columns, 8, 28, mid, true);
        let got = centred_height(&h).expect("every column crosses");
        let mean = want.iter().sum::<f64>() / l as f64;
        for i in 0..l {
            assert!((got[i] + mean - want[i]).abs() < 0.1, "column {i}");
        }
        // A window that does not contain the interface finds nothing, and says so.
        let none = height_field(&columns, 2, 6, mid, true);
        assert!(none.iter().all(|x| x.is_none()));
        assert!(centred_height(&none).is_none(), "a field with holes was handed on");
    }

    // ---------------------------------------------------------------- (e)

    /// The edge on an EMPTY lattice is EMPTY, not zero-width; and the `b = L` chart is
    /// vacuous and carries the label through.
    #[test]
    fn an_empty_lattice_has_an_empty_edge_and_the_global_chart_is_vacuous() {
        let g = EdgeLattice::seeded_by(16, 3, free_rules(), |_, _| 0.0);
        let (e, v) = edge_at(&g, 4, 1);
        assert!(e.empty, "an empty lattice split something");
        assert!(e.width.is_nan(), "an absent edge was given a width");
        assert_eq!(e.position, None);
        assert_eq!(v.probes, 0, "an empty lattice had something movable in it");
        assert!(!e.vacuous);

        let f = EdgeLattice::seeded(16, 11, 0.3, free_rules());
        let (g16, _) = edge_at(&f, 16, 1);
        assert!(g16.vacuous, "the b = L chart is held by conservation and must say so");
    }

    /// Two positive controls, and the reading that separates them.
    ///
    /// **A one-phase lattice's edge is EVERYWHERE**: every block holds movable cells and the
    /// dynamics splits almost every one of them (63 of 64 at this size and seed — a block can
    /// happen to close, and the reading says so rather than rounding), so the edge fraction
    /// is at the ceiling and the reading carries no interface. That is not a defect of the
    /// instrument — it is what the instrument should say — and it is why an edge reading
    /// means something only where the interior closes. **A droplet in vacuum** is that case: the blocks outside it hold nothing
    /// movable and do not split, so the edge is a proper subset, its centroid sits on the
    /// droplet and its width is finite.
    #[test]
    fn a_one_phase_lattice_splits_everywhere_and_a_droplet_does_not() {
        let g = EdgeLattice::seeded(32, 0xED6E, 0.3, free_rules());
        let (e4, v4) = edge_at(&g, 4, 1);
        assert!(!e4.empty && v4.probes > 0);
        assert!(e4.fraction > 0.95, "a one-phase lattice's edge read {}", e4.fraction);

        let l = 32usize;
        let r = 8.0f64;
        let d = droplet_scene(l, 4, r, 0.6, 0.0, free_rules());
        let (ed, vd) = edge_at(&d, 4, 1);
        assert!(!ed.empty && vd.probes > 0);
        assert!(
            ed.fraction > 0.05 && ed.fraction < 0.7,
            "the droplet's edge covered {} of the chart",
            ed.fraction
        );
        let p = ed.position.expect("a non-empty edge has a centroid");
        let want = (l / 2) as f64;
        assert!(
            (p[0] - want).abs() < 6.0 && (p[1] - want).abs() < 6.0,
            "the edge's centroid {p:?} is not on the droplet at ({want}, {want})"
        );
        assert!(ed.width.is_finite() && ed.width > 0.0 && ed.width < l as f64);
    }

    // ---------------------------------------------------------------- (f)

    /// The droplet-survival instrument on scenes whose answers are known: an EMPTY droplet
    /// has none to lose and says so, a droplet whose bonds are forbidden starts UNBOUND and
    /// says so, and a droplet held by a rent so deep it never pays keeps its cluster to the
    /// cap. None of the three is reported as a short lifetime.
    #[test]
    fn the_droplet_instrument_names_its_two_absences_and_reads_a_survivor() {
        let rules = EdgeRules::edge0_cluster(amp(), 40.0);

        let none = droplet_lifetime(32, 1, 8.0, 0.0, 0.0, rules, 5, 10, 200);
        assert!(none.empty, "an empty lattice had a droplet");
        assert_eq!(none.steps, None);
        assert!(!none.capped, "an absent droplet was reported as a survivor");
        assert!(none.lifetime().is_nan());

        let free = droplet_lifetime(32, 2, 8.0, 0.6, 0.02, free_rules(), 5, 10, 200);
        assert!(!free.empty && free.unbound, "a bond-free droplet reported a cluster");
        assert_eq!(free.start_largest, 1, "a bond-free graph has singletons only");
        assert_eq!(free.steps, None);
        assert!(free.lifetime().is_nan());

        // A small droplet under a rent so deep it never pays: the CLUSTER rule gels it and it
        // keeps its cluster to the cap, and the PAIR rule loses half of its start in 75 steps.
        // Two answers from one instrument on one scene, which is what says it can return more
        // than the one it was built to like.
        let held = droplet_lifetime(32, 3, 3.0, 0.6, 0.02, rules, 20, 25, 3_000);
        assert!(!held.empty && !held.unbound);
        assert!(held.start_largest > 2, "the held droplet never bound: {}", held.start_largest);
        assert!(held.capped, "the cluster rule's droplet died at {:?}", held.steps);
        assert!(held.lifetime().is_nan(), "a capped run reported a lifetime");
        assert!(held.ledger_exact);
        assert!(held.trace.len() > 2);

        let mut pair = rules;
        pair.mover = holon_lattice_pair();
        let died = droplet_lifetime(32, 3, 3.0, 0.6, 0.02, pair, 20, 25, 3_000);
        assert!(!died.empty && !died.unbound);
        assert!(died.steps.is_some(), "the pair rule's droplet survived to the cap");
        assert!(died.lifetime() > 0.0 && died.lifetime() <= 3_000.0);
        assert!(
            died.lifetime() < held.cap as f64,
            "the death and the cap were not told apart"
        );
    }

    fn holon_lattice_pair() -> crate::edge::MoverRule {
        crate::edge::MoverRule::Pair
    }

    // ---------------------------------------------------------------- geometry

    /// The torus distance is a minimum image on a NON-orthogonal lattice: the two axes are
    /// 60° apart, so a distance computed by rounding each component separately would read
    /// long, and the check is against the shortest of the nine translates by construction.
    #[test]
    fn the_torus_distance_is_the_shortest_translate() {
        let l = 16usize;
        let embed = crate::edge::hex_embed(&Model::fhp7());
        assert!((torus_distance(l, 0, 1, &embed) - 1.0).abs() < 1e-12);
        assert!((torus_distance(l, 0, l, &embed) - 1.0).abs() < 1e-12);
        // Wrapping: cell (0, L-1) is one link from (0, 0).
        assert!((torus_distance(l, 0, l - 1, &embed) - 1.0).abs() < 1e-12);
        // Symmetric, and never longer than the plain (unwrapped) separation.
        for a in [0usize, 5, 37, 200] {
            for b in [3usize, 44, 111, 250] {
                let d = torus_distance(l, a, b, &embed);
                assert!((d - torus_distance(l, b, a, &embed)).abs() < 1e-12);
                let pa = cell_position(l, a, &embed);
                let pb = cell_position(l, b, &embed);
                let plain = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2)).sqrt();
                assert!(d <= plain + 1e-12);
            }
        }
    }

    /// The two scenes are the shapes they say they are, measured on the block densities.
    #[test]
    fn the_droplet_and_the_slab_are_the_shapes_they_name() {
        let rules = EdgeRules::edge0(amp(), EdgeRules::rent_from_retention(0.5526));
        let l = 48usize;
        let embed = crate::edge::hex_embed(&Model::fhp7());
        let centre = (l / 2) * l + l / 2;
        let d = droplet_scene(l, 1, 10.0, 0.7, 0.05, rules);
        let inside: Vec<f64> = (0..l * l)
            .filter(|&c| torus_distance(l, c, centre, &embed) < 6.0)
            .map(|c| d.cells[c].count_ones() as f64 / d.n as f64)
            .collect();
        let outside: Vec<f64> = (0..l * l)
            .filter(|&c| torus_distance(l, c, centre, &embed) > 16.0)
            .map(|c| d.cells[c].count_ones() as f64 / d.n as f64)
            .collect();
        let mi = inside.iter().sum::<f64>() / inside.len() as f64;
        let mo = outside.iter().sum::<f64>() / outside.len() as f64;
        assert!((mi - 0.7).abs() < 0.05, "the droplet's inside read {mi}");
        assert!((mo - 0.05).abs() < 0.02, "the droplet's outside read {mo}");

        let s = slab_scene(l, 2, 12, 36, 0.7, 0.05, rules);
        let p = profile_j(&s);
        assert!((p[24] - 0.7).abs() < 0.06, "the slab's middle read {}", p[24]);
        assert!((p[2] - 0.05).abs() < 0.03, "the slab's outside read {}", p[2]);
    }
}


