//! THE LENS STACK: six readings of a trajectory, each declaring the variable it measures.
//!
//! The organising rule is M-MAINTENANCE-LENS, generalised past repair: *a lens computed on
//! a scene that cannot carry its variable is not a weak measurement of that variable, it
//! is not a measurement of it at all.* The registry's founding case is a defect metric on
//! a chart that discards the repaired coordinate; the case here is a TETRAHEDRAL order
//! parameter on a two-dimensional scene. The quench scenes this crate was built for are
//! `Dims::Two`. Reporting a tetrahedrality number on them would be exactly that error, so
//! the three-dimensional lenses REFUSE on them and name the gate whose passing would lift
//! the refusal (Object rule 9).
//!
//! That refusal is the finding, not an inconvenience. Every lens here is implemented for
//! real and gated against exact reference configurations, so the stack is ready for the
//! 3D tier the moment a 3D trajectory exists; what it will not do is manufacture a number
//! from a scene with no third dimension in it.

use crate::census::{closure_leg, ClosureLeg, Stakes};
use crate::traj::Trajectory;

/// A lens declining to report, with the gate that would lift the decline.
#[derive(Clone, Debug, PartialEq)]
pub struct LensRefusal {
    pub lens: &'static str,
    pub gate: &'static str,
    pub reason: String,
}

pub type Reading<T> = Result<T, LensRefusal>;

fn refuse<T>(lens: &'static str, gate: &'static str, reason: impl Into<String>) -> Reading<T> {
    Err(LensRefusal {
        lens,
        gate,
        reason: reason.into(),
    })
}

// ---------------------------------------------------------------- geometry helpers

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn norm(a: [f64; 3]) -> f64 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Indices of the `k` nearest neighbours of `i`, nearest first.
pub fn k_nearest(pos: &[[f64; 3]], i: usize, k: usize) -> Vec<usize> {
    let mut d: Vec<(f64, usize)> = (0..pos.len())
        .filter(|&j| j != i)
        .map(|j| (norm(sub(pos[j], pos[i])), j))
        .collect();
    d.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    d.into_iter().take(k).map(|(_, j)| j).collect()
}

// ---------------------------------------------------------------- 1. q-tetrahedral

/// Errington–Debenedetti tetrahedral order over the four nearest neighbours:
///
/// ```text
/// q = 1 − (3/8) Σ_{j<k} (cos ψ_jk + 1/3)²
/// ```
///
/// A perfect tetrahedron reads exactly 1; a planar square reads exactly 1/2; an ideal gas
/// averages 0.
///
/// **REFUSES on a two-dimensional scene.** In a plane the four-neighbour angle set cannot
/// reach the tetrahedral configuration at all — the quantity is bounded away from its own
/// definition's meaning, and calling the result "tetrahedrality" would be a lens that does
/// not contain its variable. The gate that lifts the refusal is a scene with `dims = 3`.
pub fn q_tetrahedral(dims: u32, center: [f64; 3], neighbours: &[[f64; 3]]) -> Reading<f64> {
    if dims != 3 {
        return refuse(
            "q-tetrahedral",
            "dims == 3",
            format!(
                "the scene is {dims}-dimensional; a tetrahedral order parameter on a plane \
                 does not contain the variable it names (M-MAINTENANCE-LENS)"
            ),
        );
    }
    if neighbours.len() != 4 {
        return refuse(
            "q-tetrahedral",
            "exactly 4 neighbours",
            format!("{} neighbours supplied; the definition is over four", neighbours.len()),
        );
    }
    let u: Vec<[f64; 3]> = neighbours
        .iter()
        .map(|&p| {
            let v = sub(p, center);
            let n = norm(v).max(1e-300);
            [v[0] / n, v[1] / n, v[2] / n]
        })
        .collect();
    let mut acc = 0.0;
    for j in 0..4 {
        for k in (j + 1)..4 {
            let c = dot(u[j], u[k]);
            acc += (c + 1.0 / 3.0).powi(2);
        }
    }
    Ok(1.0 - 0.375 * acc)
}

// ---------------------------------------------------------------- 2. Steinhardt q_l

/// Associated Legendre `P_l^m(x)` with the Condon–Shortley phase, by the standard
/// recurrence. Only `|q_lm|` is ever used downstream, so the phase convention cancels;
/// it is included so the function is the textbook one and can be checked against tables.
fn plm(l: usize, m: usize, x: f64) -> f64 {
    let mut pmm = 1.0f64;
    if m > 0 {
        let somx2 = ((1.0 - x) * (1.0 + x)).max(0.0).sqrt();
        let mut fact = 1.0f64;
        for _ in 0..m {
            pmm *= -fact * somx2;
            fact += 2.0;
        }
    }
    if l == m {
        return pmm;
    }
    let mut pmmp1 = x * (2.0 * m as f64 + 1.0) * pmm;
    if l == m + 1 {
        return pmmp1;
    }
    let mut pll = 0.0f64;
    for ll in (m + 2)..=l {
        let llf = ll as f64;
        let mf = m as f64;
        pll = ((2.0 * llf - 1.0) * x * pmmp1 - (llf + mf - 1.0) * pmm) / (llf - mf);
        pmm = pmmp1;
        pmmp1 = pll;
    }
    pll
}

fn ylm_norm(l: usize, m: usize) -> f64 {
    // sqrt((2l+1)/(4π) · (l−m)!/(l+m)!)
    let mut ratio = 1.0f64;
    for k in (l - m + 1)..=(l + m) {
        ratio /= k as f64;
    }
    ((2 * l + 1) as f64 / (4.0 * std::f64::consts::PI) * ratio).sqrt()
}

/// Steinhardt bond-orientational order `q_l` over an explicit neighbour set:
///
/// ```text
/// q_lm = (1/N) Σ_j Y_lm(θ_j, φ_j),   q_l = sqrt( 4π/(2l+1) · Σ_m |q_lm|² )
/// ```
///
/// Reference values it is gated against: FCC (12 neighbours) 0.57452, BCC (8+6
/// neighbours) 0.51057, simple cubic (6 neighbours) 0.35355 — the last being exactly
/// `sqrt(1/8)`, which makes it an analytic check rather than a transcribed one.
///
/// **REFUSES on a two-dimensional scene.** `Y_6m` is a function on the sphere; every
/// neighbour of a planar scene sits at `θ = π/2`, which collapses the reading onto one
/// latitude and makes it a disguised hexatic rather than a `q6`. The gate that lifts the
/// refusal is a scene with `dims = 3`; for planar scenes the honest lens is `hexatic_psi6`
/// below, and it exists so nobody has to fake this one.
pub fn steinhardt_q(l: usize, dims: u32, center: [f64; 3], neighbours: &[[f64; 3]]) -> Reading<f64> {
    if dims != 3 {
        return refuse(
            "steinhardt-q",
            "dims == 3",
            format!(
                "the scene is {dims}-dimensional; every neighbour sits at theta = pi/2 and \
                 the spherical-harmonic sum is a hexatic in disguise. Use hexatic_psi6."
            ),
        );
    }
    if neighbours.is_empty() {
        return refuse("steinhardt-q", "at least 1 neighbour", "empty neighbour set");
    }
    let n = neighbours.len() as f64;
    let mut sum = 0.0f64;
    for m in 0..=l {
        let (mut re, mut im) = (0.0f64, 0.0f64);
        for &p in neighbours {
            let v = sub(p, center);
            let r = norm(v).max(1e-300);
            let ct = (v[2] / r).clamp(-1.0, 1.0);
            let phi = v[1].atan2(v[0]);
            let y = ylm_norm(l, m) * plm(l, m, ct);
            re += y * (m as f64 * phi).cos();
            im += y * (m as f64 * phi).sin();
        }
        re /= n;
        im /= n;
        let mag2 = re * re + im * im;
        // |q_{l,-m}| = |q_{l,m}|, so the negative half is counted by doubling.
        sum += if m == 0 { mag2 } else { 2.0 * mag2 };
    }
    Ok((4.0 * std::f64::consts::PI / (2 * l + 1) as f64 * sum).sqrt())
}

// ---------------------------------------------------------------- 3. hexatic psi6

/// Two-dimensional bond-orientational order, `ψ6 = |N⁻¹ Σ_j exp(6 i θ_j)|`.
///
/// A perfect triangular lattice reads exactly 1; a square lattice reads 0 on its four
/// nearest neighbours. **REFUSES on a three-dimensional scene**, where the bond angle
/// projected into a plane is not the orientational variable and `steinhardt_q` is the
/// lens that carries it.
pub fn hexatic_psi6(dims: u32, center: [f64; 3], neighbours: &[[f64; 3]]) -> Reading<f64> {
    if dims != 2 {
        return refuse(
            "hexatic-psi6",
            "dims == 2",
            format!(
                "the scene is {dims}-dimensional; a projected bond angle is not the \
                 orientational variable in three dimensions. Use steinhardt_q."
            ),
        );
    }
    if neighbours.is_empty() {
        return refuse("hexatic-psi6", "at least 1 neighbour", "empty neighbour set");
    }
    let (mut re, mut im) = (0.0f64, 0.0f64);
    for &p in neighbours {
        let v = sub(p, center);
        let t = v[1].atan2(v[0]);
        re += (6.0 * t).cos();
        im += (6.0 * t).sin();
    }
    let n = neighbours.len() as f64;
    Ok(((re / n).powi(2) + (im / n).powi(2)).sqrt())
}

// ---------------------------------------------------------------- 4. MSD / diffusion

/// The MEAN ELAPSED TIME of a `lag`-frame separation, in femtoseconds.
///
/// Not `lag * dt * substeps`. The engine's timestep adapts mid-run, so a fixed number of
/// frames is not a fixed duration, and a diffusion constant fitted against a frame axis is
/// a diffusion constant divided by whatever the timestep happened to be doing.
pub fn mean_lag_fs(traj: &Trajectory, lag: usize) -> f64 {
    let nf = traj.frames.len();
    if lag == 0 || lag >= nf {
        return 0.0;
    }
    let mut acc = 0.0f64;
    for t in 0..(nf - lag) {
        acc += traj.frames[t + lag].time - traj.frames[t].time;
    }
    acc / (nf - lag) as f64 * crate::traj::AU_TIME_FS
}

/// Mean squared displacement at `lag` frames, averaged over atoms and over time origins.
pub fn msd(traj: &Trajectory, lag: usize) -> f64 {
    let nf = traj.frames.len();
    if lag == 0 || lag >= nf {
        return 0.0;
    }
    let n = traj.header.n_atoms;
    let mut acc = 0.0f64;
    let mut count = 0usize;
    for t in 0..(nf - lag) {
        for i in 0..n {
            let d = sub(traj.frames[t + lag].pos[i], traj.frames[t].pos[i]);
            acc += dot(d, d);
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        acc / count as f64
    }
}

/// The exponent of `MSD ∝ τ^alpha` over the fit window, by least squares in log-log.
///
/// The Einstein relation IS the statement that this exponent is 1. Reporting a diffusion
/// constant without checking it is reporting the slope of a line through data that is not
/// a line.
pub fn msd_exponent(traj: &Trajectory, max_lag: usize) -> f64 {
    let mut pts: Vec<(f64, f64)> = Vec::new();
    let mut lag = 2usize;
    while lag <= max_lag {
        let x = mean_lag_fs(traj, lag);
        let y = msd(traj, lag);
        if x > 0.0 && y > 0.0 {
            pts.push((x.ln(), y.ln()));
        }
        lag = (lag as f64 * 1.5).ceil() as usize;
    }
    if pts.len() < 3 {
        return f64::NAN;
    }
    let n = pts.len() as f64;
    let mx = pts.iter().map(|p| p.0).sum::<f64>() / n;
    let my = pts.iter().map(|p| p.1).sum::<f64>() / n;
    let num: f64 = pts.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    let den: f64 = pts.iter().map(|p| (p.0 - mx).powi(2)).sum();
    num / den
}

/// The band the MSD exponent must lie in for an Einstein fit to mean anything.
pub const DIFFUSION_ALPHA_LO: f64 = 0.85;
pub const DIFFUSION_ALPHA_HI: f64 = 1.15;

/// The Einstein diffusion constant from a straight-line fit of MSD against elapsed time.
///
/// **REFUSES on two conditions, and the second one was added after the first proved
/// insufficient on real data.**
///
/// 1. *Wall domination.* These scenes are `Boundary::Walls`, so displacement saturates at
///    the box rather than growing, and a slope fitted across the saturation is a number
///    about the box. Refuse when `MSD` at the largest fitted lag exceeds `(L_min/4)²`.
///
/// 2. *No diffusive regime.* `MSD = 2 d D τ` is a fit to a LINE, and on the banked
///    hydrogen trajectories the MSD goes as `τ^1.7` — between ballistic and diffusive —
///    over every window the wall gate admits. The first version of this lens happily
///    returned a "diffusion constant" that grew monotonically with the fit window, from
///    0.0008 to 0.018 bohr²/fs across lags 2 to 200, which is the signature of fitting a
///    line to a curve. So the exponent is measured and the lens refuses outside
///    `[0.85, 1.15]`. That band is not tuned: the Einstein relation is the statement that
///    the exponent is 1, and the width is the tolerance on a log-log slope from ten points.
///
/// The gate that lifts either refusal is a longer trajectory in a larger box — which is
/// the T3 scale-up, and this is one more measurement saying so.
pub fn diffusion(traj: &Trajectory, max_lag: usize) -> Reading<f64> {
    let dims = traj.header.dims as f64;
    let l_min = if traj.header.dims == 2 {
        traj.header.box_w.min(traj.header.box_h)
    } else {
        traj.header.box_w.min(traj.header.box_h).min(traj.header.box_d)
    };
    if max_lag < 2 || max_lag >= traj.frames.len() {
        return refuse(
            "diffusion",
            "2 <= max_lag < n_frames",
            format!("max_lag {max_lag} against {} frames", traj.frames.len()),
        );
    }
    let cap = (l_min / 4.0).powi(2);
    let top = msd(traj, max_lag);
    if top > cap {
        return refuse(
            "diffusion",
            "MSD(max_lag) <= (L_min/4)^2",
            format!(
                "MSD at lag {max_lag} is {top:.3} bohr^2 against a wall-saturation cap of \
                 {cap:.3}; the fit would measure the box, not the fluid"
            ),
        );
    }
    let alpha = msd_exponent(traj, max_lag);
    if alpha.is_nan() {
        return refuse(
            "diffusion",
            "at least 3 lags below max_lag",
            format!(
                "max_lag {max_lag} leaves too few points to measure the MSD exponent; \
                 without it a straight-line fit is unchecked"
            ),
        );
    }
    if !(DIFFUSION_ALPHA_LO..=DIFFUSION_ALPHA_HI).contains(&alpha) {
        return refuse(
            "diffusion",
            "MSD exponent in [0.85, 1.15]",
            format!(
                "MSD goes as tau^{alpha:.2} over this window, not tau^1; there is no \
                 diffusive regime here and a straight-line fit would report the slope of a \
                 curve as a diffusion constant"
            ),
        );
    }
    // Least squares through the origin: MSD = 2 d D tau.
    let (mut sxy, mut sxx) = (0.0f64, 0.0f64);
    for lag in 1..=max_lag {
        let x = mean_lag_fs(traj, lag);
        let y = msd(traj, lag);
        sxy += x * y;
        sxx += x * x;
    }
    Ok(sxy / sxx / (2.0 * dims))
}

// ------------------------------------------- 4a. the BOUNDARY-AWARE diffusion lens

/// The boundary a trajectory was integrated under, as the LENS sees it.
///
/// Three cases, the same three `holon_render::sim::Boundary` carries, and a COPY rather
/// than an import: this crate has no path to `holon-render` on purpose (its manifest says
/// why), and the caller converts at the call site. The copy is checked in
/// `holon-render/tests/diffusion_boundary.rs`, which maps every engine boundary onto one
/// of these and fails if the engine grows a fourth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LensBoundary {
    /// Soft quadratic walls on every face. Displacement SATURATES at the box.
    Walls,
    /// No container at all. Displacement is unbounded.
    Open,
    /// The periodic box. Displacement is unbounded IN THE UNWRAPPED COORDINATE and
    /// saturates in the wrapped one, which is the whole reason this lens exists.
    Periodic,
}

/// Boltzmann's constant, hartree per kelvin — the engine's own `sim::K_B` to every digit.
/// Repeated rather than imported for the same reason [`LensBoundary`] is.
pub const K_B_HARTREE_PER_K: f64 = 3.166811563e-6;

/// The Yeh–Hummer self-interaction constant for a CUBIC periodic cell,
/// `ξ = 2.837297` (Yeh and Hummer, *J. Phys. Chem. B* **108** (2004) 15873, after the
/// Ewald self-term of Hummer, Gronbech-Jensen and Neumann 1998).
///
/// It is a property of the cubic lattice and of nothing in this programme.
pub const YEH_HUMMER_XI: f64 = 2.837297;

/// ONE declared lag interval, in frames, closed at both ends.
///
/// **The exponent and the slope are read on the SAME interval, and that is the point of
/// the type.** [`diffusion`] measured its exponent on a `×1.5` ladder up to `max_lag` and
/// then fitted its slope over `1..=max_lag` — two different windows, the second of them
/// reaching down to lag 1, which on a molecular trajectory is deep in the ballistic
/// regime. A gate that passes on one window and a number that is fitted on another is not
/// a gated number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LagWindow {
    pub lo: usize,
    pub hi: usize,
}

impl LagWindow {
    pub fn new(lo: usize, hi: usize) -> Self {
        LagWindow { lo, hi }
    }
}

/// The finite-size context a periodic diffusion constant is read IN — reported beside the
/// value and never folded into it.
///
/// A diffusion constant measured in a periodic box of edge `L` is smaller than the
/// infinite-system one by the hydrodynamic self-interaction of the tracer with its own
/// images, `D_∞ − D_PBC = k_B T ξ / (6 π η L)` (Yeh and Hummer 2004; ξ =
/// [`YEH_HUMMER_XI`]). On a 128-water box that term is not small, and a lens that applied
/// it would be publishing a number nobody measured: the correction needs the SHEAR
/// VISCOSITY `η` of the very model whose diffusion is being read, and this programme has
/// not measured one (the fluid tier's viscosity is a lattice-gas number on a different
/// carrier, FLUID-0).
///
/// So the lens computes everything it has — the box edge, ξ, the temperature the frames
/// carry, and the whole correction DIVIDED BY the viscosity it does not have — and states
/// that the correction is NOT APPLIED. `applied` is a field rather than a comment so a
/// reader of the record can check it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FiniteSize {
    pub boundary: LensBoundary,
    /// The SHORTEST box edge, bohr. The correction's `L`.
    pub box_edge_bohr: f64,
    /// [`YEH_HUMMER_XI`], carried so the record does not have to be trusted to know it.
    pub xi: f64,
    /// The mean temperature over the frames the fit used, kelvin, off the frames.
    pub temperature_k: f64,
    /// `k_B T ξ / (6 π L)`, in bohr²/fs times one atomic unit of shear viscosity —
    /// so `correction = this / η_au`. `None` off a periodic box, where the images the
    /// correction is about do not exist.
    pub yeh_hummer_over_viscosity: Option<f64>,
    /// ALWAYS `false`. The lens does not apply it.
    pub applied: bool,
    pub note: &'static str,
}

impl FiniteSize {
    /// The Yeh–Hummer correction at a shear viscosity the CALLER supplies, in bohr²/fs.
    /// `None` off a periodic box or at a non-positive viscosity.
    pub fn yeh_hummer_correction(&self, shear_viscosity_au: f64) -> Option<f64> {
        if shear_viscosity_au > 0.0 {
            self.yeh_hummer_over_viscosity.map(|q| q / shear_viscosity_au)
        } else {
            None
        }
    }
}

/// What [`diffusion_periodic`] reports: the value, the gate readings that admitted it,
/// the window it was read on, and the finite-size context it was read in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiffusionReading {
    /// THE READING: the SLOPE of `MSD` against elapsed time over the declared interval,
    /// divided by `2 d`. Free intercept, because above the ballistic crossover the MSD is
    /// `2 d D τ + b` with `b > 0` the ballistic offset, and a line forced through the
    /// origin over a window that does not reach the origin reports `D` too high.
    pub d_bohr2_per_fs: f64,
    /// The same window fitted THROUGH THE ORIGIN, which is [`diffusion`]'s convention.
    /// Carried so the two lenses can be compared on one arm rather than argued about.
    pub d_through_origin_bohr2_per_fs: f64,
    /// The free fit's intercept, bohr². Positive on a real diffusive window; a negative
    /// one is a window that has not left the crossover and is worth seeing.
    pub intercept_bohr2: f64,
    /// The log-log slope over the declared interval. The Einstein relation IS `α = 1`.
    pub alpha: f64,
    pub lags: LagWindow,
    /// The declared interval in femtoseconds, `(t_lo, t_hi)`, off the frames' own clock.
    pub window_fs: (f64, f64),
    /// Points on the `×1.5` ladder inside the interval — what the exponent was fitted on.
    pub ladder_points: usize,
    /// Lags in the interval — what the slope was fitted on.
    pub fit_points: usize,
    pub finite_size: FiniteSize,
}

/// The shortest box edge of a trajectory, bohr, in its own dimensionality.
fn shortest_edge(traj: &Trajectory) -> f64 {
    if traj.header.dims == 2 {
        traj.header.box_w.min(traj.header.box_h)
    } else {
        traj.header.box_w.min(traj.header.box_h).min(traj.header.box_d)
    }
}

/// The largest single-frame displacement of any atom over the trajectory, bohr.
///
/// The unwrapping check: a WRAPPED periodic coordinate jumps by nearly a box edge the
/// frame an atom crosses a face, and nothing a liquid does between two readout frames
/// looks like that.
fn largest_frame_step(traj: &Trajectory) -> f64 {
    let mut worst = 0.0f64;
    for t in 1..traj.frames.len() {
        let (a, b) = (&traj.frames[t - 1], &traj.frames[t]);
        for i in 0..a.pos.len().min(b.pos.len()) {
            worst = worst.max(norm(sub(b.pos[i], a.pos[i])));
        }
    }
    worst
}

/// THE BOUNDARY-AWARE EINSTEIN DIFFUSION LENS, on ONE declared lag interval.
///
/// The second external review's fourth source claim, in its own words: *"the diffusion
/// lens refuses large displacements by a WALL-saturation cap regardless of boundary, and
/// LIQUID-2 sized its window to that cap on unwrapped periodic positions"*. Verified, and
/// this is the repair. Four things change and one does not.
///
/// 1. **The wall cap applies ONLY under [`LensBoundary::Walls`].** In a walled box a
///    displacement cannot exceed the box, so a slope fitted across the saturation is a
///    number about the container; that refusal is right and it is kept, unchanged, for
///    walled scenes. In a PERIODIC box there is no wall to saturate against and the
///    unwrapped displacement is unbounded — the cap there refuses the very measurement the
///    box was built to make, and LIQUID-2 had been sizing its lag window down to clear a
///    gate that means nothing on its scene.
/// 2. **The positions must be UNWRAPPED, and that is now CHECKED rather than assumed.**
///    Under `Periodic` the lens refuses when any single-frame displacement exceeds half
///    the shortest edge, which is the signature of a face crossing left in the coordinate.
///    The caller unwraps (LIQUID-2 accumulates `dd -= L·round(dd/L)` per frame); this gate
///    is what catches the day somebody forgets.
/// 3. **ONE interval.** The exponent gate and the slope are both read on `lags`, so the
///    window the gate passed is the window the number came from. See [`LagWindow`].
/// 4. **Finite size is REPORTED, never applied.** See [`FiniteSize`]: the box edge, ξ, the
///    frames' own temperature and the Yeh–Hummer correction per unit viscosity, with
///    `applied: false`.
///
/// What does not change: **the exponent gate**. `MSD = 2 d D τ` is a fit to a LINE and the
/// Einstein relation is the statement that the exponent is 1, so the log-log slope over
/// the declared interval is measured and the lens refuses outside
/// `[`[`DIFFUSION_ALPHA_LO`]`, `[`DIFFUSION_ALPHA_HI`]`]`. That gate is what refused
/// LIQUID-1's own reading (`τ^1.64` over `[5.2, 547]` fs) and it is untouched here.
///
/// [`diffusion`] is left exactly as it stands for its callers of record.
pub fn diffusion_periodic(
    traj: &Trajectory,
    lags: LagWindow,
    boundary: LensBoundary,
) -> Reading<DiffusionReading> {
    let dims = traj.header.dims as f64;
    let nf = traj.frames.len();
    if lags.lo < 1 || lags.hi <= lags.lo || lags.hi >= nf {
        return refuse(
            "diffusion-periodic",
            "1 <= lag_lo < lag_hi < n_frames",
            format!(
                "the declared interval is [{}, {}] against {nf} frames; a fit needs an \
                 interval inside the trajectory",
                lags.lo, lags.hi
            ),
        );
    }
    let l_min = shortest_edge(traj);

    // (b) THE WALL CAP, under walls and nowhere else.
    if boundary == LensBoundary::Walls {
        let cap = (l_min / 4.0).powi(2);
        let top = msd(traj, lags.hi);
        if top > cap {
            return refuse(
                "diffusion-periodic",
                "under Walls: MSD(lag_hi) <= (L_min/4)^2",
                format!(
                    "MSD at lag {} is {top:.3} bohr^2 against a wall-saturation cap of \
                     {cap:.3}; in a WALLED box the fit would measure the box, not the fluid",
                    lags.hi
                ),
            );
        }
    }

    // (2) THE UNWRAPPING CHECK, under the periodic box and nowhere else.
    if boundary == LensBoundary::Periodic {
        let step = largest_frame_step(traj);
        if step > 0.5 * l_min {
            return refuse(
                "diffusion-periodic",
                "under Periodic: no single-frame displacement exceeds L_min/2",
                format!(
                    "some atom moved {step:.3} bohr in one frame against a half-edge of \
                     {:.3}; these positions are WRAPPED, and an MSD on wrapped coordinates \
                     saturates at the cell instead of growing. Unwrap them (accumulate the \
                     minimum-image displacement) and read again",
                    0.5 * l_min
                ),
            );
        }
    }

    // (c) ONE interval: the ladder for the exponent, the whole interval for the slope.
    let mut ladder: Vec<(f64, f64)> = Vec::new();
    let mut lag = lags.lo;
    while lag <= lags.hi {
        let x = mean_lag_fs(traj, lag);
        let y = msd(traj, lag);
        if x > 0.0 && y > 0.0 {
            ladder.push((x.ln(), y.ln()));
        }
        let next = (lag as f64 * 1.5).ceil() as usize;
        lag = if next > lag { next } else { lag + 1 };
    }
    if ladder.len() < 3 {
        return refuse(
            "diffusion-periodic",
            "at least 3 points on the x1.5 ladder inside the declared interval",
            format!(
                "the interval [{}, {}] carries {} usable ladder points; without three the \
                 exponent is not measured and the straight-line fit would be unchecked",
                lags.lo,
                lags.hi,
                ladder.len()
            ),
        );
    }
    let n = ladder.len() as f64;
    let mx = ladder.iter().map(|p| p.0).sum::<f64>() / n;
    let my = ladder.iter().map(|p| p.1).sum::<f64>() / n;
    let num: f64 = ladder.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    let den: f64 = ladder.iter().map(|p| (p.0 - mx).powi(2)).sum();
    let alpha = num / den;

    // (a) THE EXPONENT GATE, unchanged in band and now read on the fitted interval.
    if !alpha.is_finite() || !(DIFFUSION_ALPHA_LO..=DIFFUSION_ALPHA_HI).contains(&alpha) {
        return refuse(
            "diffusion-periodic",
            "MSD exponent in [0.85, 1.15] on the declared interval",
            format!(
                "MSD goes as tau^{alpha:.3} over [{}, {}] frames = [{:.3}, {:.3}] fs, not \
                 tau^1; there is no diffusive regime on the interval the slope would be \
                 fitted over, and a straight-line fit would report the slope of a curve as \
                 a diffusion constant",
                lags.lo,
                lags.hi,
                mean_lag_fs(traj, lags.lo),
                mean_lag_fs(traj, lags.hi)
            ),
        );
    }

    // THE SLOPE, on the same interval: free intercept, and the through-origin value beside
    // it so the two conventions are visible rather than argued about.
    let pts: Vec<(f64, f64)> = (lags.lo..=lags.hi)
        .map(|k| (mean_lag_fs(traj, k), msd(traj, k)))
        .collect();
    let m = pts.len() as f64;
    let sx = pts.iter().map(|p| p.0).sum::<f64>() / m;
    let sy = pts.iter().map(|p| p.1).sum::<f64>() / m;
    let sxy: f64 = pts.iter().map(|p| (p.0 - sx) * (p.1 - sy)).sum();
    let sxx: f64 = pts.iter().map(|p| (p.0 - sx) * (p.0 - sx)).sum();
    if sxx <= 0.0 {
        return refuse(
            "diffusion-periodic",
            "the declared interval spans a nonzero duration",
            format!("every lag in [{}, {}] has the same elapsed time", lags.lo, lags.hi),
        );
    }
    let slope = sxy / sxx;
    let intercept = sy - slope * sx;
    let (o_xy, o_xx) = pts.iter().fold((0.0f64, 0.0f64), |(a, b), p| (a + p.0 * p.1, b + p.0 * p.0));

    // (d) THE FINITE-SIZE CONTEXT, computed and NOT applied.
    // the temperature the correction is evaluated at is the trajectory's own, over every
    // frame it carries — the reading is about the arm, not about a sub-window of it
    let t_mean = traj.frames.iter().map(|f| f.temperature).sum::<f64>() / nf as f64;
    let over_eta = if boundary == LensBoundary::Periodic && l_min > 0.0 {
        // k_B T xi / (6 pi L) is bohr^2 per ATOMIC time unit times one atomic viscosity;
        // the lens's time axis is femtoseconds, so it converts here and nowhere else.
        Some(K_B_HARTREE_PER_K * t_mean * YEH_HUMMER_XI
            / (6.0 * std::f64::consts::PI * l_min)
            / crate::traj::AU_TIME_FS)
    } else {
        None
    };
    Ok(DiffusionReading {
        d_bohr2_per_fs: slope / (2.0 * dims),
        d_through_origin_bohr2_per_fs: o_xy / o_xx / (2.0 * dims),
        intercept_bohr2: intercept,
        alpha,
        lags,
        window_fs: (mean_lag_fs(traj, lags.lo), mean_lag_fs(traj, lags.hi)),
        ladder_points: ladder.len(),
        fit_points: pts.len(),
        finite_size: FiniteSize {
            boundary,
            box_edge_bohr: l_min,
            xi: YEH_HUMMER_XI,
            temperature_k: t_mean,
            yeh_hummer_over_viscosity: over_eta,
            applied: false,
            note: "Yeh-Hummer 2004: D_inf - D_PBC = k_B T xi / (6 pi eta L). NAMED, NOT \
                   APPLIED - this programme has not measured the shear viscosity of this \
                   law, and the field carries the correction DIVIDED BY that viscosity so \
                   a reader with one can finish the arithmetic.",
        },
    })
}

// ---------------------------------------------------------------- 5. H-bond census

/// Luzar–Chandler geometric criterion, in bohr because the engine's unit is bohr.
pub const HB_R_OO_BOHR: f64 = 6.6140; // 3.5 Angstrom
pub const HB_R_OH_BOHR: f64 = 4.6298; // 2.45 Angstrom
pub const HB_ANGLE_DEG: f64 = 30.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HBond {
    pub donor_o: usize,
    pub hydrogen: usize,
    pub acceptor_o: usize,
}

/// The H-bond census under the stated criterion: `r(O···O) < 6.6140`, `r(O···H) < 4.6298`,
/// and the angle at the donor between its O–H vector and the O···O vector below 30°.
///
/// A hydrogen is assigned to its NEAREST oxygen as its covalent donor. **REFUSES on a
/// scene with no oxygen or no hydrogen** — there is no H-bond variable in such a scene,
/// and returning zero would be a null that means "not applicable" reported as a
/// measurement.
pub fn hbonds(pos: &[[f64; 3]], z: &[u32]) -> Reading<Vec<HBond>> {
    let oxygens: Vec<usize> = (0..z.len()).filter(|&i| z[i] == 8).collect();
    let hydrogens: Vec<usize> = (0..z.len()).filter(|&i| z[i] == 1).collect();
    if oxygens.is_empty() || hydrogens.is_empty() {
        return refuse(
            "hbond-census",
            "the scene holds at least one O and one H",
            format!(
                "{} oxygens and {} hydrogens; there is no hydrogen bond to count, and a \
                 zero here would read as a measured absence",
                oxygens.len(),
                hydrogens.len()
            ),
        );
    }
    let cos_cut = (HB_ANGLE_DEG * std::f64::consts::PI / 180.0).cos();
    let mut out = Vec::new();
    for &h in &hydrogens {
        // Covalent donor = nearest oxygen.
        let donor = *oxygens
            .iter()
            .min_by(|&&a, &&b| {
                norm(sub(pos[a], pos[h]))
                    .partial_cmp(&norm(sub(pos[b], pos[h])))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        for &acc in &oxygens {
            if acc == donor {
                continue;
            }
            let r_oo = norm(sub(pos[acc], pos[donor]));
            if r_oo >= HB_R_OO_BOHR {
                continue;
            }
            if norm(sub(pos[acc], pos[h])) >= HB_R_OH_BOHR {
                continue;
            }
            let a = sub(pos[h], pos[donor]);
            let b = sub(pos[acc], pos[donor]);
            let c = dot(a, b) / (norm(a).max(1e-300) * norm(b).max(1e-300));
            if c > cos_cut {
                out.push(HBond {
                    donor_o: donor,
                    hydrogen: h,
                    acceptor_o: acc,
                });
            }
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------- 6. largest domain

/// The largest connected component of an EXPLICIT edge set.
///
/// The edge set is a parameter and never a default. A "largest domain" reported without
/// saying which graph it is a domain of is two different numbers wearing one name: the
/// bonded-pair graph and the H-bond graph disagree by construction, and the whole
/// boundness-versus-closure fence lives in that disagreement.
pub fn largest_domain(n: usize, edges: &[(usize, usize)]) -> usize {
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    for &(a, b) in edges {
        let (x, y) = (find(&mut parent, a), find(&mut parent, b));
        if x != y {
            parent[x] = y;
        }
    }
    let mut size = vec![0usize; n];
    for i in 0..n {
        let r = find(&mut parent, i);
        size[r] += 1;
    }
    size.into_iter().max().unwrap_or(0)
}

// ---------------------------------------------------------------- 7. closure defect

/// The closure-defect lens for a SCALAR macro view: bin the reading, then run the same
/// fiber-invariance test the census runs on partitions.
///
/// Binning is a declaration, not a detail — a view is only as closed as its resolution,
/// and two runs binned differently are two different views. The bin edges are parameters
/// and the count is reported with the defect, so a defect quoted without its resolution
/// cannot be produced by this function.
pub fn binned_closure_defect(
    values: &[f64],
    bins: usize,
    lo: f64,
    hi: f64,
    st: &Stakes,
) -> ClosureLeg {
    let span = (hi - lo).max(f64::MIN_POSITIVE);
    let keys: Vec<u64> = values
        .iter()
        .map(|&v| {
            let t = ((v - lo) / span * bins as f64).floor();
            t.clamp(0.0, (bins - 1) as f64) as u64
        })
        .collect();
    closure_leg(&keys, st)
}


// ------------------------------------------- 7. the periodic readouts (LIQUID-1)
//
// Two readings a WRAPPING box needs and the open-box lenses above cannot give it. Both
// take the cell explicitly rather than reading it from a header: a lens that guessed the
// periodicity would be a different instrument on the same numbers, which is the
// M-STALE-INSTRUMENT shape. `hbonds_periodic` is `hbonds`'s criterion verbatim with every
// difference vector reduced to the minimum image; `rdf_oo` is the oxygen-oxygen radial
// distribution normalised to the ideal gas at the box's own density.

/// `b - a` under the minimum image of an ORTHORHOMBIC cell.
///
/// An axis whose edge is not a finite positive length is left UNREDUCED — that is the
/// open-axis case stated rather than assumed, and it is what makes a slab (one edge zero
/// or infinite) a well-defined argument instead of a NaN.
#[inline]
fn min_image(a: [f64; 3], b: [f64; 3], cell: [f64; 3]) -> [f64; 3] {
    let mut d = sub(b, a);
    for k in 0..3 {
        let l = cell[k];
        if l.is_finite() && l > 0.0 {
            d[k] -= l * (d[k] / l).round();
        }
    }
    d
}

/// The H-bond census of [`hbonds`], under the minimum image of an orthorhombic `cell`.
///
/// The SAME rung-1 criterion — `r(O···O) < 6.6140`, `r(O···H) < 4.6298`, and the angle at
/// the donor under 30° — with every difference vector, INCLUDING the one that picks each
/// hydrogen's covalent donor, taken under the minimum image. A hydrogen bond whose partner
/// sits across a face is a hydrogen bond; read with the open-box lens it is both invisible
/// (the O···O separation reads a box length) and mis-assigned (the nearest oxygen by raw
/// difference is the wrong molecule), which is LIQUID-1's plant (ii).
///
/// **REFUSES exactly where [`hbonds`] refuses** and nowhere else: a scene with no oxygen or
/// no hydrogen has no hydrogen-bond variable, and a zero there would read as a measured
/// absence.
/// The rung-1 criterion under the minimum image (LIQUID-1). A DIAGNOSTIC may ask the same
/// census at another angle through `hbonds_periodic_with`; this one is the frozen readout.
pub fn hbonds_periodic(pos: &[[f64; 3]], z: &[u32], cell: [f64; 3]) -> Reading<Vec<HBond>> {
    hbonds_periodic_with(pos, z, cell, HB_ANGLE_DEG)
}

/// `hbonds_periodic` with the donor angle as a parameter (degrees). Not a frozen readout.
pub fn hbonds_periodic_with(pos: &[[f64; 3]], z: &[u32], cell: [f64; 3], angle_deg: f64) -> Reading<Vec<HBond>> {
    let oxygens: Vec<usize> = (0..z.len()).filter(|&i| z[i] == 8).collect();
    let hydrogens: Vec<usize> = (0..z.len()).filter(|&i| z[i] == 1).collect();
    if oxygens.is_empty() || hydrogens.is_empty() {
        return refuse(
            "hbond-census-periodic",
            "the scene holds at least one O and one H",
            format!(
                "{} oxygens and {} hydrogens; there is no hydrogen bond to count, and a \
                 zero here would read as a measured absence",
                oxygens.len(),
                hydrogens.len()
            ),
        );
    }
    let cos_cut = (angle_deg * std::f64::consts::PI / 180.0).cos();
    let mut out = Vec::new();
    for &h in &hydrogens {
        // Covalent donor = nearest oxygen UNDER THE MINIMUM IMAGE.
        let donor = *oxygens
            .iter()
            .min_by(|&&a, &&b| {
                norm(min_image(pos[h], pos[a], cell))
                    .partial_cmp(&norm(min_image(pos[h], pos[b], cell)))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();
        for &acc in &oxygens {
            if acc == donor {
                continue;
            }
            let b = min_image(pos[donor], pos[acc], cell);
            let r_oo = norm(b);
            if r_oo >= HB_R_OO_BOHR {
                continue;
            }
            if norm(min_image(pos[h], pos[acc], cell)) >= HB_R_OH_BOHR {
                continue;
            }
            let a = min_image(pos[donor], pos[h], cell);
            let c = dot(a, b) / (norm(a).max(1e-300) * norm(b).max(1e-300));
            if c > cos_cut {
                out.push(HBond {
                    donor_o: donor,
                    hydrogen: h,
                    acceptor_o: acc,
                });
            }
        }
    }
    Ok(out)
}

/// A radial distribution function: the bin centres, the reading, and the two numbers the
/// normalisation was taken at, so a `g` quoted without its density cannot be produced here.
#[derive(Clone, Debug, PartialEq)]
pub struct Rdf {
    /// Bin centres, bohr: bin `k` covers `[k·dr, (k+1)·dr)` and its centre is `(k+½)·dr`.
    pub r: Vec<f64>,
    pub g: Vec<f64>,
    pub dr: f64,
    /// Oxygens in the scene.
    pub n_o: usize,
    /// The number density the ideal-gas reference was taken at, `N_O / V`, bohr⁻³.
    pub rho_o: f64,
}

/// The oxygen–oxygen radial distribution under the minimum image of an orthorhombic cell,
/// normalised to the ideal gas at the BOX's density:
///
/// ```text
/// g(r) = 2 · N_pairs(bin) / (N_O · rho_O · 4 pi r^2 dr),   rho_O = N_O / V,  r the bin centre
/// ```
///
/// The factor of two is there because the loop counts each unordered pair once and each
/// oxygen's shell counts it twice; with it, `sum over bins of rho_O·4 pi r^2·g(r)·dr` over a
/// shell is the mean COORDINATION NUMBER of that shell, which is the form the gate reads.
///
/// **REFUSES** past half the shortest edge — beyond that the minimum image is no longer the
/// only image and the histogram is counting one partner twice — and on a scene with fewer
/// than two oxygens, where there is no pair to bin and an all-zero `g` would read as a
/// measured absence.
pub fn rdf_oo(pos: &[[f64; 3]], z: &[u32], cell: [f64; 3], dr: f64, r_max: f64) -> Reading<Rdf> {
    let oxygens: Vec<usize> = (0..z.len()).filter(|&i| z[i] == 8).collect();
    if oxygens.len() < 2 {
        return refuse(
            "rdf-oo",
            "the scene holds at least two oxygens",
            format!("{} oxygens; there is no O–O pair to bin", oxygens.len()),
        );
    }
    let min_edge = cell[0].min(cell[1]).min(cell[2]);
    if !(dr > 0.0) || !(r_max > dr) {
        return refuse(
            "rdf-oo",
            "0 < dr < r_max",
            format!("dr {dr} and r_max {r_max} leave no bin to fill"),
        );
    }
    if !(r_max <= 0.5 * min_edge) {
        return refuse(
            "rdf-oo",
            "r_max <= half the shortest cell edge",
            format!(
                "r_max {r_max:.4} bohr against a shortest edge of {min_edge:.4}; past half \
                 the edge the minimum image is not the only image and the histogram counts \
                 one partner twice"
            ),
        );
    }
    let n_bins = (r_max / dr).floor() as usize;
    if n_bins == 0 {
        return refuse("rdf-oo", "at least one whole bin inside r_max", format!("r_max {r_max} at dr {dr}"));
    }
    let n_o = oxygens.len();
    let volume = cell[0] * cell[1] * cell[2];
    let rho_o = n_o as f64 / volume;
    let mut counts = vec![0.0f64; n_bins];
    for (ii, &i) in oxygens.iter().enumerate() {
        for &j in oxygens[ii + 1..].iter() {
            let r = norm(min_image(pos[i], pos[j], cell));
            let k = (r / dr).floor();
            if k >= 0.0 && (k as usize) < n_bins {
                counts[k as usize] += 1.0;
            }
        }
    }
    let mut rs = Vec::with_capacity(n_bins);
    let mut g = Vec::with_capacity(n_bins);
    for k in 0..n_bins {
        let r = (k as f64 + 0.5) * dr;
        let shell = 4.0 * std::f64::consts::PI * r * r * dr;
        rs.push(r);
        g.push(2.0 * counts[k] / (n_o as f64 * rho_o * shell));
    }
    Ok(Rdf { r: rs, g, dr, n_o, rho_o })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;

    // ---------------------------------------------------- q-tetrahedral

    #[test]
    fn perfect_tetrahedron_reads_exactly_one() {
        let c = [0.0, 0.0, 0.0];
        let nb = [
            [1.0, 1.0, 1.0],
            [1.0, -1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
        ];
        let q = q_tetrahedral(3, c, &nb).unwrap();
        assert!((q - 1.0).abs() < EPS, "q = {q}");
    }

    /// A planar square is exactly 1/2 by hand: four 90-degree pairs and two 180-degree
    /// pairs give 4·(1/3)² + 2·(−2/3)² = 4/3, and 1 − (3/8)(4/3) = 1/2.
    #[test]
    fn planar_square_reads_exactly_one_half() {
        let c = [0.0, 0.0, 0.0];
        let nb = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
        ];
        let q = q_tetrahedral(3, c, &nb).unwrap();
        assert!((q - 0.5).abs() < EPS, "q = {q}");
    }

    #[test]
    fn q_tetrahedral_refuses_a_planar_scene() {
        let e = q_tetrahedral(2, [0.0; 3], &[[1.0, 0.0, 0.0]; 4]).unwrap_err();
        assert_eq!(e.gate, "dims == 3");
        assert!(e.reason.contains("M-MAINTENANCE-LENS"));
    }

    // ---------------------------------------------------- Steinhardt q6

    fn q6_of(nb: &[[f64; 3]]) -> f64 {
        steinhardt_q(6, 3, [0.0; 3], nb).unwrap()
    }

    /// Simple cubic is `sqrt(1/8)` exactly — an analytic reference, not a transcribed one.
    #[test]
    fn simple_cubic_q6_is_sqrt_one_eighth() {
        let nb = [
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ];
        let q = q6_of(&nb);
        assert!((q - (1.0f64 / 8.0).sqrt()).abs() < 1e-12, "q6(sc) = {q}");
    }

    /// FCC's twelve nearest neighbours: the literature value 0.57452.
    #[test]
    fn fcc_q6_matches_the_published_value() {
        let mut nb = Vec::new();
        for &(a, b) in &[(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
            nb.push([a, b, 0.0]);
            nb.push([a, 0.0, b]);
            nb.push([0.0, a, b]);
        }
        assert_eq!(nb.len(), 12);
        let q = q6_of(&nb);
        assert!((q - 0.574524).abs() < 1e-4, "q6(fcc) = {q}");
    }

    /// BCC counted over 8 + 6 neighbours: the literature value 0.51057. The neighbour
    /// count is part of the reference and is stated here, because BCC over 8 alone is a
    /// different published number (0.62854) and quoting one against the other is how a
    /// gate passes for the wrong reason.
    #[test]
    fn bcc_q6_matches_the_published_value_at_fourteen_neighbours() {
        let mut nb = Vec::new();
        for &sx in &[-1.0f64, 1.0] {
            for &sy in &[-1.0f64, 1.0] {
                for &sz in &[-1.0f64, 1.0] {
                    nb.push([sx, sy, sz]); // 8 at sqrt(3)
                }
            }
        }
        for &s in &[-2.0f64, 2.0] {
            nb.push([s, 0.0, 0.0]);
            nb.push([0.0, s, 0.0]);
            nb.push([0.0, 0.0, s]); // 6 at 2
        }
        assert_eq!(nb.len(), 14);
        let q = q6_of(&nb);
        assert!((q - 0.510688).abs() < 1e-3, "q6(bcc,14) = {q}");
    }

    #[test]
    fn steinhardt_refuses_a_planar_scene() {
        let e = steinhardt_q(6, 2, [0.0; 3], &[[1.0, 0.0, 0.0]]).unwrap_err();
        assert_eq!(e.gate, "dims == 3");
        assert!(e.reason.contains("hexatic"));
    }

    // ---------------------------------------------------- hexatic

    #[test]
    fn triangular_lattice_psi6_is_exactly_one() {
        let nb: Vec<[f64; 3]> = (0..6)
            .map(|k| {
                let t = std::f64::consts::PI / 3.0 * k as f64 + 0.37; // arbitrary rotation
                [t.cos(), t.sin(), 0.0]
            })
            .collect();
        let p = hexatic_psi6(2, [0.0; 3], &nb).unwrap();
        assert!((p - 1.0).abs() < 1e-12, "psi6 = {p}");
    }

    #[test]
    fn square_lattice_psi6_is_exactly_zero_on_four_neighbours() {
        let nb = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
        ];
        let p = hexatic_psi6(2, [0.0; 3], &nb).unwrap();
        assert!(p < 1e-12, "psi6 = {p}");
    }

    /// The prereg staked the square lattice below 0.5 on SIX neighbours; the two
    /// diagonals a tie-break may pick give 0 or 1/3, and both clear the bar.
    #[test]
    fn square_lattice_psi6_stays_under_a_half_on_six_neighbours() {
        let base = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, -1.0, 0.0],
        ];
        let diag = [
            [1.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0],
            [-1.0, -1.0, 0.0],
            [1.0, -1.0, 0.0],
        ];
        for a in 0..4 {
            for b in (a + 1)..4 {
                let mut nb = base.to_vec();
                nb.push(diag[a]);
                nb.push(diag[b]);
                let p = hexatic_psi6(2, [0.0; 3], &nb).unwrap();
                assert!(p < 0.5, "psi6 = {p} for diagonals {a},{b}");
            }
        }
    }

    #[test]
    fn hexatic_refuses_a_three_dimensional_scene() {
        let e = hexatic_psi6(3, [0.0; 3], &[[1.0, 0.0, 0.0]]).unwrap_err();
        assert_eq!(e.gate, "dims == 2");
    }

    // ---------------------------------------------------- H-bond

    fn dimer(r_oo: f64, angle_deg: f64) -> (Vec<[f64; 3]>, Vec<u32>) {
        // Donor O at the origin, its H along +x at the O–H bond length, acceptor O at
        // `r_oo` and `angle_deg` away from the O–H direction.
        let a = angle_deg * std::f64::consts::PI / 180.0;
        (
            vec![
                [0.0, 0.0, 0.0],
                [1.814, 0.0, 0.0],
                [r_oo * a.cos(), r_oo * a.sin(), 0.0],
            ],
            vec![8, 1, 8],
        )
    }

    #[test]
    fn ideal_dimer_reads_one_hydrogen_bond() {
        let (p, z) = dimer(5.6, 0.0);
        let b = hbonds(&p, &z).unwrap();
        assert_eq!(b.len(), 1);
        assert_eq!(b[0], HBond { donor_o: 0, hydrogen: 1, acceptor_o: 2 });
    }

    #[test]
    fn a_stretched_dimer_reads_none() {
        let (p, z) = dimer(20.0, 0.0);
        assert!(hbonds(&p, &z).unwrap().is_empty());
    }

    #[test]
    fn a_rotated_dimer_reads_none_past_thirty_degrees() {
        assert_eq!(hbonds(&dimer(5.6, 25.0).0, &dimer(5.6, 25.0).1).unwrap().len(), 1);
        assert!(hbonds(&dimer(5.6, 35.0).0, &dimer(5.6, 35.0).1).unwrap().is_empty());
    }

    /// The exponent gate must ACCEPT a random walk and REFUSE ballistic motion, or it is
    /// a gate that only ever says no.
    #[test]
    fn the_diffusion_lens_accepts_a_walk_and_refuses_a_flight() {
        use crate::synthetic::{self, Spec};
        // A random walk in a big box: MSD ~ tau, and the walls stay out of the window.
        let mut sp = Spec::quench_like(4000, vec![1; 12]);
        sp.box_w = 4000.0;
        sp.box_h = 4000.0;
        sp.seed = 3;
        let walk = synthetic::liquid(sp.clone());
        let a_walk = msd_exponent(&walk, 200);
        assert!(
            (a_walk - 1.0).abs() < 0.15,
            "a random walk must read tau^1, read tau^{a_walk:.3}"
        );
        assert!(diffusion(&walk, 200).is_ok(), "and the lens must report on it");

        // Ballistic: every atom on a straight line, MSD ~ tau^2.
        let n = 12usize;
        let flight = synthetic::build(sp, move |t, pos, vel| {
            for i in 0..n {
                let v = 0.01 * (i + 1) as f64;
                pos[i] = [2000.0 + v * t as f64, 2000.0 - v * t as f64, 0.0];
                vel[i] = [v, -v, 0.0];
            }
            crate::traj::BondSet::empty()
        });
        let a_flight = msd_exponent(&flight, 200);
        assert!(
            a_flight > 1.8,
            "ballistic motion must read near tau^2, read tau^{a_flight:.3}"
        );
        let e = diffusion(&flight, 200).unwrap_err();
        assert_eq!(e.gate, "MSD exponent in [0.85, 1.15]");
    }

    // ------------------------------------- the boundary-aware lens (the review's item 3)

    /// A Gaussian deviate from the fixtures' own LCG, so the walk below is a function of
    /// its stated seed and nothing else.
    fn gauss(rng: &mut crate::synthetic::Lcg) -> f64 {
        let u = rng.next().max(1e-12);
        let v = rng.next();
        (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
    }

    /// `n_atoms` independent Gaussian random walks in a cubic box of edge `l`, step
    /// standard deviation `s` bohr per frame per dimension. Positions are UNWRAPPED: the
    /// walk is allowed to leave the cell, which is what an unwrapping accumulator produces.
    ///
    /// The construction fixes the answer: `MSD(τ) = 3 s² k` at lag `k`, and `MSD = 6 D τ`,
    /// so `D = s² / (2 Δt)` with `Δt` the frame duration in femtoseconds.
    fn periodic_walk(seed: u64, n_atoms: usize, n_frames: usize, l: f64, s: f64) -> Trajectory {
        use crate::synthetic::{self, Lcg, Spec};
        let mut sp = Spec::quench_like(n_frames, vec![8; n_atoms]);
        sp.dims = 3;
        sp.dt = 1.0;
        sp.substeps = 10;
        sp.box_w = l;
        sp.box_h = l;
        sp.box_d = l;
        sp.seed = seed;
        let mut rng = Lcg(seed ^ 0xD1B5_4A32_D192_ED03);
        synthetic::build(sp, move |t, pos, _vel| {
            if t == 0 {
                for p in pos.iter_mut() {
                    *p = [0.5 * l, 0.5 * l, 0.5 * l];
                }
            } else {
                for p in pos.iter_mut() {
                    for c in 0..3 {
                        p[c] += s * gauss(&mut rng);
                    }
                }
            }
            crate::traj::BondSet::empty()
        })
    }

    /// The same trajectory with every coordinate folded back into the cell — a WRAPPED
    /// periodic trajectory, which is what the engine's own atom positions are.
    fn wrapped(t: &Trajectory, l: f64) -> Trajectory {
        let mut w = t.clone();
        for f in w.frames.iter_mut() {
            for p in f.pos.iter_mut() {
                for c in 0..3 {
                    p[c] = p[c].rem_euclid(l);
                }
            }
        }
        w
    }

    /// THE REVIEW'S FOURTH SOURCE CLAIM, as a test: on unwrapped positions from a periodic
    /// box the OLD lens refuses by a wall it does not have, and the new one reads the
    /// construction's own `D` back.
    #[test]
    fn a_periodic_walk_reads_its_own_d_where_the_wall_cap_refused_it() {
        let (l, s, n_frames) = (30.0f64, 0.30f64, 1200usize);
        let traj = periodic_walk(0x5741_4c4b, 64, n_frames, l, s);
        let dt_fs = crate::traj::AU_TIME_FS * 1.0 * 10.0;
        let d_true = s * s / (2.0 * dt_fs);
        let win = LagWindow::new(20, 400);

        // (i) the OLD lens, on the very same unwrapped positions, refuses by the wall cap.
        let old = diffusion(&traj, win.hi).unwrap_err();
        assert_eq!(
            old.gate, "MSD(max_lag) <= (L_min/4)^2",
            "the old lens must refuse these positions by the wall cap: {}",
            old.reason
        );

        // (ii) the new lens, told the box is periodic, reads the walk.
        let r = diffusion_periodic(&traj, win, LensBoundary::Periodic)
            .unwrap_or_else(|e| panic!("{}: {}", e.gate, e.reason));
        assert!(
            (r.d_bohr2_per_fs - d_true).abs() / d_true < 0.10,
            "D read {:.6} against the construction's {d_true:.6} bohr^2/fs",
            r.d_bohr2_per_fs
        );
        assert!((r.alpha - 1.0).abs() < 0.10, "a walk must read tau^1, read tau^{:.3}", r.alpha);
        assert_eq!(r.lags, win);
        assert!(r.ladder_points >= 3 && r.fit_points == win.hi - win.lo + 1);

        // (iii) the finite-size context is REPORTED and NOT applied.
        assert!(!r.finite_size.applied, "the lens must never apply Yeh-Hummer");
        assert_eq!(r.finite_size.box_edge_bohr, l);
        assert_eq!(r.finite_size.xi, YEH_HUMMER_XI);
        let over = r.finite_size.yeh_hummer_over_viscosity.expect("periodic carries the term");
        assert!(over > 0.0);
        // and the correction is the term divided by a viscosity the CALLER supplies
        let eta = 2.0e-4; // an atomic-unit viscosity, supplied here and nowhere in the lens
        assert!(
            (r.finite_size.yeh_hummer_correction(eta).unwrap() - over / eta).abs() < 1e-18,
            "the correction is the reported term over the caller's viscosity"
        );
    }

    /// The WRAPPED positions of the same walk are refused, and refused for the right
    /// reason: the face crossings are still in the coordinate.
    #[test]
    fn wrapped_positions_are_refused_by_name() {
        let (l, s) = (30.0f64, 0.30f64);
        let traj = wrapped(&periodic_walk(0x5741_4c4b, 64, 1200, l, s), l);
        let e = diffusion_periodic(&traj, LagWindow::new(20, 400), LensBoundary::Periodic)
            .unwrap_err();
        assert_eq!(e.gate, "under Periodic: no single-frame displacement exceeds L_min/2");
        assert!(e.reason.contains("WRAPPED"), "the refusal must name the cause: {}", e.reason);
    }

    /// The exponent gate is kept, and it is what stops a flight.
    #[test]
    fn a_ballistic_flight_is_refused_by_the_exponent_gate() {
        use crate::synthetic::{self, Spec};
        let n = 16usize;
        let l = 4000.0;
        let mut sp = Spec::quench_like(1200, vec![8; n]);
        sp.dims = 3;
        sp.dt = 1.0;
        sp.substeps = 10;
        sp.box_w = l;
        sp.box_h = l;
        sp.box_d = l;
        let flight = synthetic::build(sp, move |t, pos, _vel| {
            for (i, p) in pos.iter_mut().enumerate() {
                let v = 0.01 * (i + 1) as f64;
                *p = [2000.0 + v * t as f64, 2000.0 - v * t as f64, 2000.0];
            }
            crate::traj::BondSet::empty()
        });
        let e = diffusion_periodic(&flight, LagWindow::new(20, 400), LensBoundary::Periodic)
            .unwrap_err();
        assert_eq!(e.gate, "MSD exponent in [0.85, 1.15] on the declared interval");
        assert!(e.reason.contains("tau^1.9") || e.reason.contains("tau^2.0"), "{}", e.reason);
    }

    /// AND THE WALL CAP IS NOT WEAKENED: a WALLED scene whose displacement has run past
    /// the box is refused exactly as it was.
    #[test]
    fn a_walled_walk_is_still_capped() {
        let (l, s) = (30.0f64, 0.30f64);
        let traj = periodic_walk(0x5741_4c4b, 64, 1200, l, s);
        let e =
            diffusion_periodic(&traj, LagWindow::new(20, 400), LensBoundary::Walls).unwrap_err();
        assert_eq!(e.gate, "under Walls: MSD(lag_hi) <= (L_min/4)^2");
        // and an OPEN box has neither gate: it is unbounded and its coordinates never wrap
        let r = diffusion_periodic(&traj, LagWindow::new(20, 400), LensBoundary::Open)
            .expect("an open box has no wall to saturate against");
        assert!(r.finite_size.yeh_hummer_over_viscosity.is_none(), "no images, no correction");
    }

    /// A short declared interval is refused rather than fitted: three ladder points is the
    /// floor, and the refusal names it.
    #[test]
    fn a_declared_interval_too_short_to_check_is_refused() {
        let traj = periodic_walk(0x5741_4c4b, 8, 400, 30.0, 0.30);
        let e = diffusion_periodic(&traj, LagWindow::new(100, 120), LensBoundary::Periodic)
            .unwrap_err();
        assert_eq!(
            e.gate,
            "at least 3 points on the x1.5 ladder inside the declared interval"
        );
    }

    #[test]
    fn hbond_census_refuses_a_scene_with_no_oxygen() {
        let e = hbonds(&[[0.0; 3], [1.4, 0.0, 0.0]], &[1, 1]).unwrap_err();
        assert_eq!(e.gate, "the scene holds at least one O and one H");
    }

    // ---------------------------------------------------- largest domain

    #[test]
    fn largest_domain_reads_the_edge_set_it_is_given() {
        // Same six atoms; two different edge sets; two different answers, on purpose.
        assert_eq!(largest_domain(6, &[(0, 1), (1, 2)]), 3);
        assert_eq!(largest_domain(6, &[(0, 1), (2, 3), (4, 5)]), 2);
        assert_eq!(largest_domain(6, &[]), 1);
    }

    // ---------------------------------------------------- closure defect

    #[test]
    fn binned_defect_is_void_without_repeats() {
        let v: Vec<f64> = (0..500).map(|i| i as f64).collect();
        let leg = binned_closure_defect(&v, 500, 0.0, 500.0, &Stakes::default());
        assert!(leg.void);
    }

    #[test]
    fn binned_defect_finds_a_split_reading() {
        // One bin, two futures: coarse enough that the view cannot be a function of itself.
        let mut v = Vec::new();
        for i in 0..600 {
            v.push(0.10);
            v.push(if i < 300 { 0.40 } else { 0.90 });
        }
        let leg = binned_closure_defect(&v, 3, 0.0, 1.0, &Stakes::default());
        assert!(!leg.void);
        assert!(leg.defect > 0.0);
        assert!(leg.witness_pair_count >= 1);
    }

    // ---------------------------------------------------- the periodic readouts

    /// EMBED-1's water pin, repeated here rather than imported: this crate has ZERO
    /// dependencies by design and importing the constant would point the dependency the
    /// wrong way. `holon_render::field::{WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD}`.
    const PIN_R: f64 = 1.9435738400;
    const PIN_THETA: f64 = 1.6887434037;

    /// A water with its oxygen at `o`, its first hydrogen along `+dir`, and the second at
    /// the pin angle in the plane `dir`–`up`.
    fn water(o: [f64; 3], dir: [f64; 3], up: [f64; 3]) -> [[f64; 3]; 3] {
        let n = norm(dir);
        let d = [dir[0] / n, dir[1] / n, dir[2] / n];
        let m = norm(up);
        let u = [up[0] / m, up[1] / m, up[2] / m];
        let (c, s) = (PIN_THETA.cos(), PIN_THETA.sin());
        [
            o,
            [o[0] + PIN_R * d[0], o[1] + PIN_R * d[1], o[2] + PIN_R * d[2]],
            [
                o[0] + PIN_R * (c * d[0] + s * u[0]),
                o[1] + PIN_R * (c * d[1] + s * u[1]),
                o[2] + PIN_R * (c * d[2] + s * u[2]),
            ],
        ]
    }

    /// A simple-cubic lattice of oxygens has SIX nearest neighbours, and the first peak of
    /// `g_OO` integrates to exactly that: `sum rho·4 pi r^2 g dr` over the peak = 6, by the
    /// normalisation's own arithmetic and independently of the bin centre. Below the
    /// spacing the reading is an exact zero — there is no pair there to bin.
    #[test]
    fn rdf_of_a_simple_cubic_lattice_integrates_to_six_neighbours() {
        let a = 1.0f64;
        let n = 4usize;
        let l = a * n as f64;
        let mut pos = Vec::new();
        let mut z = Vec::new();
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    pos.push([i as f64 * a, j as f64 * a, k as f64 * a]);
                    z.push(8u32);
                }
            }
        }
        let dr = 0.1;
        let rdf = rdf_oo(&pos, &z, [l, l, l], dr, 0.5 * l).unwrap();
        assert_eq!(rdf.n_o, 64);
        assert!((rdf.rho_o - 64.0 / (l * l * l)).abs() < EPS);
        // below the spacing: exactly zero, in every bin wholly inside r < a
        for (k, &r) in rdf.r.iter().enumerate() {
            if r + 0.5 * dr <= a {
                assert_eq!(rdf.g[k], 0.0, "bin at r = {r} reads {}", rdf.g[k]);
            }
        }
        // the first peak, integrated: the next shell is at a·sqrt(2) = 1.414a, so a window
        // out to 1.2a holds the first shell and nothing else
        let mut coord = 0.0f64;
        for (k, &r) in rdf.r.iter().enumerate() {
            if r < 1.2 * a {
                coord += rdf.rho_o * 4.0 * std::f64::consts::PI * r * r * rdf.g[k] * dr;
            }
        }
        assert!((coord - 6.0).abs() < 1e-9, "first-shell coordination {coord}");
        // and the refusal past half the shortest edge
        let e = rdf_oo(&pos, &z, [l, l, l], dr, 0.5 * l + 0.001).unwrap_err();
        assert_eq!(e.gate, "r_max <= half the shortest cell edge");
        let e = rdf_oo(&pos[..1], &z[..1], [l, l, l], dr, 1.0).unwrap_err();
        assert_eq!(e.gate, "the scene holds at least two oxygens");
    }

    /// A hydrogen bond ACROSS A FACE: the periodic lens finds it, the open-box lens on the
    /// same wrapped coordinates finds nothing — it reads the O···O separation as a box
    /// length and assigns the donor hydrogen to the wrong molecule. This is LIQUID-1's
    /// plant (ii) as a two-molecule fixture.
    #[test]
    fn a_hydrogen_bond_across_a_face_is_seen_only_under_the_minimum_image() {
        let l = 20.0f64;
        let cell = [l, l, l];
        // donor at x = 1.0 pointing along −x; acceptor 5.5 bohr away at x = −4.5, wrapped
        // to 15.5 — so the bond runs through the x = 0 face
        let donor = water([1.0, 10.0, 10.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let (ch, sh) = ((0.5 * PIN_THETA).cos(), (0.5 * PIN_THETA).sin());
        let acc_o = [15.5, 10.0, 10.0];
        let acceptor = [
            acc_o,
            [acc_o[0] + PIN_R * ch, acc_o[1], acc_o[2] + PIN_R * sh],
            [acc_o[0] + PIN_R * ch, acc_o[1], acc_o[2] - PIN_R * sh],
        ];
        let mut pos: Vec<[f64; 3]> = Vec::new();
        let mut z: Vec<u32> = Vec::new();
        for w in [donor, acceptor] {
            for (m, p) in w.iter().enumerate() {
                // wrapped into [0, L), as the engine's drift step leaves them
                pos.push([
                    p[0] - l * (p[0] / l).floor(),
                    p[1] - l * (p[1] / l).floor(),
                    p[2] - l * (p[2] / l).floor(),
                ]);
                z.push(if m == 0 { 8 } else { 1 });
            }
        }
        assert!(pos.iter().all(|p| p.iter().all(|&x| (0.0..l).contains(&x))), "every atom is inside the cell");
        let across = hbonds_periodic(&pos, &z, cell).unwrap();
        assert_eq!(across.len(), 1, "one bond across the face: {across:?}");
        assert_eq!(across[0].donor_o, 0);
        assert_eq!(across[0].hydrogen, 1);
        assert_eq!(across[0].acceptor_o, 3);
        let open = hbonds(&pos, &z).unwrap();
        assert_eq!(open.len(), 0, "the open-box lens sees none of it: {open:?}");
    }

    /// Inside the cell with nothing crossing a face the two lenses are the SAME reading —
    /// the periodic one is the open one plus a reduction that never fires.
    #[test]
    fn inside_the_cell_the_periodic_lens_is_the_open_lens() {
        let l = 30.0f64;
        let c = 15.0f64;
        let donor = water([c, c, c], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        let (ch, sh) = ((0.5 * PIN_THETA).cos(), (0.5 * PIN_THETA).sin());
        let acc_o = [c, c, c + 5.5];
        let acceptor = [
            acc_o,
            [acc_o[0] + PIN_R * sh, acc_o[1], acc_o[2] + PIN_R * ch],
            [acc_o[0] - PIN_R * sh, acc_o[1], acc_o[2] + PIN_R * ch],
        ];
        let mut pos: Vec<[f64; 3]> = Vec::new();
        let mut z: Vec<u32> = Vec::new();
        for w in [donor, acceptor] {
            for (m, p) in w.iter().enumerate() {
                pos.push(*p);
                z.push(if m == 0 { 8 } else { 1 });
            }
        }
        let a = hbonds(&pos, &z).unwrap();
        let b = hbonds_periodic(&pos, &z, [l, l, l]).unwrap();
        assert_eq!(a, b, "no face is crossed, so the reduction never fires");
        assert_eq!(a.len(), 1, "and the fixture is a bonded dimer: {a:?}");
    }

    #[test]
    fn the_periodic_hbond_lens_refuses_where_the_open_one_does() {
        let e = hbonds_periodic(&[[0.0; 3]], &[8], [10.0; 3]).unwrap_err();
        assert_eq!(e.gate, "the scene holds at least one O and one H");
        let e = hbonds_periodic(&[[0.0; 3]], &[1], [10.0; 3]).unwrap_err();
        assert_eq!(e.gate, "the scene holds at least one O and one H");
    }
}
