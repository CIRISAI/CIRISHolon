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

/// The Einstein diffusion constant from a straight-line fit of MSD against lag.
///
/// **REFUSES when the fit window is wall-dominated.** These scenes are `Boundary::Walls`,
/// so displacement saturates at the box rather than growing linearly, and a slope fitted
/// across the saturation is a number about the box and not about the fluid. The stated
/// criterion: refuse when `MSD` at the largest fitted lag exceeds `(L_min / 4)²`, where
/// `L_min` is the shortest box dimension the scene actually uses. The gate that lifts the
/// refusal is a shorter fit window, or periodic boundaries.
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
    // Least squares through the origin: MSD = 2 d D tau.
    let (mut sxy, mut sxx) = (0.0f64, 0.0f64);
    for lag in 1..=max_lag {
        let x = lag as f64 * traj.header.frame_fs();
        let y = msd(traj, lag);
        sxy += x * y;
        sxx += x * x;
    }
    Ok(sxy / sxx / (2.0 * dims))
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
}
