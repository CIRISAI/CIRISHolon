//! The chart auditing itself — the two instruments the certificate is built from.
//!
//! Both read **chart data only**. Neither touches the exact state, which is the whole point: a
//! certificate that needs the answer certifies nothing.
//!
//! * [`Mp2Audit`] is C1: the chart's own leading-order estimate of its error in each of the six
//!   observables, from second-order amplitudes over its own orbitals.
//! * [`Stability`] is C2: the lowest eigenvalue of the chart's stability Hessian under
//!   occupied→virtual rotations, computed **generalized** (spin-orbitals may mix spins), so the
//!   spin-flip block is present as amendment A1 requires.
//!
//! A warning that belongs next to C2 and is reported rather than papered over: a determinant that
//! has broken a continuous symmetry of `H` carries exact **Goldstone zero modes**, which are flat
//! directions and not instabilities. The staked criterion reads the raw lowest eigenvalue, so
//! this module reports both that and the value with the numerically-null space projected out, and
//! the verdict uses the staked one.

use crate::chart::Chart;
use crate::dense::jacobi;

/// C1's instrument: the chart's estimate of its own error, per observable.
#[derive(Clone, Debug)]
pub struct Mp2Audit {
    /// Correlation energy per site in units of `t` — the estimate of the O1 error.
    pub energy_per_site: f64,
    /// Estimate of the O2 (double occupancy) error, by Hellmann–Feynman on `E₂ ∝ U²`.
    pub double_occ: f64,
    /// Estimate of the O3 (site density) error.
    pub density: f64,
    /// Estimate of the O4 (magnetization) error.
    pub magnetization: f64,
    /// Estimate of the O5 (bond order) error.
    pub bond: f64,
    /// Estimate of `D_bool` — the chart's own prediction of the ModeChart fence quantity.
    pub d_bool: f64,
    /// Largest amplitude, reported as the perturbative-validity diagnostic.
    pub max_amplitude: f64,
}

impl Mp2Audit {
    pub fn of(chart: &Chart) -> Self {
        let sites = chart.sites;
        let n_occ = sites / 2;
        let u = chart.u;

        // Opposite-spin amplitudes only: the Hubbard interaction never couples like spins, so
        // every same-spin two-electron integral is identically zero here.
        // t[i][a][j][b] with (i,a) spin up, (j,b) spin down.
        let mut amp = vec![0.0; n_occ * (sites - n_occ) * n_occ * (sites - n_occ)];
        let n_virt = sites - n_occ;
        let idx = |i: usize, a: usize, j: usize, b: usize| {
            ((i * n_virt + a) * n_occ + j) * n_virt + b
        };

        let mut e2 = 0.0;
        let mut max_amplitude: f64 = 0.0;
        for i in 0..n_occ {
            for a in 0..n_virt {
                let d_up = chart.orbital_energy[0][n_occ + a] - chart.orbital_energy[0][i];
                for j in 0..n_occ {
                    for b in 0..n_virt {
                        let d_dn = chart.orbital_energy[1][n_occ + b] - chart.orbital_energy[1][j];
                        // On-site integral <ij|ab> = U sum_m phi_i(m) phi_a(m) phi_j(m) phi_b(m).
                        let mut v = 0.0;
                        for m in 0..sites {
                            v += chart.orbitals[0][i][m]
                                * chart.orbitals[0][n_occ + a][m]
                                * chart.orbitals[1][j][m]
                                * chart.orbitals[1][n_occ + b][m];
                        }
                        v *= u;
                        let denom = d_up + d_dn;
                        let t = if denom.abs() > 1e-14 { -v / denom } else { 0.0 };
                        amp[idx(i, a, j, b)] = t;
                        e2 += t * v;
                        max_amplitude = max_amplitude.max(t.abs());
                    }
                }
            }
        }

        // Second-order (unrelaxed) density correction in the MO basis. A one-body observable has
        // no first-order correction — a double excitation is orthogonal to it — so this is the
        // leading term, and it is quadratic in the amplitudes.
        let mut dr_mo = [vec![0.0; sites * sites], vec![0.0; sites * sites]];
        for i in 0..n_occ {
            for i2 in 0..n_occ {
                let mut s = 0.0;
                for a in 0..n_virt {
                    for j in 0..n_occ {
                        for b in 0..n_virt {
                            s += amp[idx(i, a, j, b)] * amp[idx(i2, a, j, b)];
                        }
                    }
                }
                dr_mo[0][i * sites + i2] -= s;
            }
        }
        for a in 0..n_virt {
            for a2 in 0..n_virt {
                let mut s = 0.0;
                for i in 0..n_occ {
                    for j in 0..n_occ {
                        for b in 0..n_virt {
                            s += amp[idx(i, a, j, b)] * amp[idx(i, a2, j, b)];
                        }
                    }
                }
                dr_mo[0][(n_occ + a) * sites + n_occ + a2] += s;
            }
        }
        // The down block by the mirrored contraction.
        for j in 0..n_occ {
            for j2 in 0..n_occ {
                let mut s = 0.0;
                for b in 0..n_virt {
                    for i in 0..n_occ {
                        for a in 0..n_virt {
                            s += amp[idx(i, a, j, b)] * amp[idx(i, a, j2, b)];
                        }
                    }
                }
                dr_mo[1][j * sites + j2] -= s;
            }
        }
        for b in 0..n_virt {
            for b2 in 0..n_virt {
                let mut s = 0.0;
                for j in 0..n_occ {
                    for i in 0..n_occ {
                        for a in 0..n_virt {
                            s += amp[idx(i, a, j, b)] * amp[idx(i, a, j, b2)];
                        }
                    }
                }
                dr_mo[1][(n_occ + b) * sites + n_occ + b2] += s;
            }
        }

        // Transform to the site basis: Δρ_site = C Δρ_MO Cᵀ.
        let mut dr_site = [vec![0.0; sites * sites], vec![0.0; sites * sites]];
        for s in 0..2 {
            for m in 0..sites {
                for n in 0..sites {
                    let mut acc = 0.0;
                    for p in 0..sites {
                        for q in 0..sites {
                            acc += chart.orbitals[s][p][m]
                                * dr_mo[s][p * sites + q]
                                * chart.orbitals[s][q][n];
                        }
                    }
                    dr_site[s][m * sites + n] = acc;
                }
            }
        }

        let density = (0..sites)
            .map(|m| (dr_site[0][m * sites + m] + dr_site[1][m * sites + m]).abs())
            .fold(0.0, f64::max);
        let magnetization = (0..sites)
            .map(|m| (dr_site[0][m * sites + m] - dr_site[1][m * sites + m]).abs())
            .fold(0.0, f64::max);
        let bond = (0..sites - 1)
            .map(|m| {
                (2.0 * (dr_site[0][m * sites + m + 1] + dr_site[1][m * sites + m + 1])).abs()
            })
            .fold(0.0, f64::max);

        // D_bool estimate: natural occupations of the corrected one-body density matrix.
        let mut d_bool: f64 = 0.0;
        for s in 0..2 {
            let corrected: Vec<f64> = (0..sites * sites)
                .map(|k| chart.rdm[s][k] + dr_site[s][k])
                .collect();
            for x in jacobi(corrected, sites).values {
                d_bool = d_bool.max(x.min(1.0 - x));
            }
        }

        // Double occupancy by Hellmann–Feynman: at fixed orbitals E₂ ∝ U², so ∂E₂/∂U = 2E₂/U.
        let double_occ = if u.abs() > 1e-12 {
            (2.0 * e2 / u).abs() / sites as f64
        } else {
            0.0
        };

        Self {
            energy_per_site: (e2 / (sites as f64 * chart.t)).abs(),
            double_occ,
            density,
            magnetization,
            bond,
            d_bool,
            max_amplitude,
        }
    }

    /// The six estimates in observable order, for comparison against `κ·τ`.
    pub fn as_vector(&self) -> [f64; 6] {
        [
            self.energy_per_site,
            self.double_occ,
            self.density,
            self.magnetization,
            self.bond,
            self.d_bool,
        ]
    }
}

/// C2's instrument: the generalized stability Hessian's lowest eigenvalue.
#[derive(Clone, Debug)]
pub struct Stability {
    /// The staked quantity: the raw lowest Hessian eigenvalue.
    pub lambda_min: f64,
    /// Diagnostic only: the lowest eigenvalue after projecting out numerically-null directions,
    /// which is what separates a Goldstone flat direction from a genuine instability.
    pub lambda_min_projected: f64,
    /// How many eigenvalues sit at the numerical null threshold.
    pub null_modes: usize,
}

const FD_STEP: f64 = 1e-4;
const NULL_TOL: f64 = 1e-6;

impl Stability {
    /// Numerical Hessian of the generalized-HF energy under occupied→virtual rotations.
    ///
    /// Numerical rather than analytic on purpose: the quadratic form's overall factor convention
    /// is easy to get wrong analytically, and C2 is *self-normalized* against its own `U = 0`
    /// value, so any consistent overall factor cancels. The `U = 0` check that the eigenvalues
    /// reproduce the particle–hole excitation ladder is what validates it.
    pub fn of(chart: &Chart) -> Self {
        let sites = chart.sites;
        let n_so = 2 * sites;
        let n_occ = sites / 2;

        // Spin-orbital coefficient matrix, block diagonal: up on rows 0..sites, down after.
        // Columns are ordered occupied-first so the rotation block is contiguous.
        let mut cols: Vec<Vec<f64>> = Vec::with_capacity(n_so);
        for s in 0..2 {
            for p in 0..n_occ {
                let mut v = vec![0.0; n_so];
                for m in 0..sites {
                    v[s * sites + m] = chart.orbitals[s][p][m];
                }
                cols.push(v);
            }
        }
        let n_occ_so = cols.len();
        for s in 0..2 {
            for p in n_occ..sites {
                let mut v = vec![0.0; n_so];
                for m in 0..sites {
                    v[s * sites + m] = chart.orbitals[s][p][m];
                }
                cols.push(v);
            }
        }

        let n_par = n_occ_so * (n_so - n_occ_so);
        let energy = |theta: &[f64]| -> f64 {
            generalized_energy(chart, &cols, n_occ_so, n_so, theta)
        };

        let mut theta = vec![0.0; n_par];
        let e0 = energy(&theta);
        let mut hess = vec![0.0; n_par * n_par];
        for p in 0..n_par {
            theta[p] = FD_STEP;
            let ep = energy(&theta);
            theta[p] = -FD_STEP;
            let em = energy(&theta);
            theta[p] = 0.0;
            hess[p * n_par + p] = (ep - 2.0 * e0 + em) / (FD_STEP * FD_STEP);
            for q in (p + 1)..n_par {
                theta[p] = FD_STEP;
                theta[q] = FD_STEP;
                let epp = energy(&theta);
                theta[q] = -FD_STEP;
                let epm = energy(&theta);
                theta[p] = -FD_STEP;
                let emm = energy(&theta);
                theta[q] = FD_STEP;
                let emp = energy(&theta);
                theta[p] = 0.0;
                theta[q] = 0.0;
                let v = (epp - epm - emp + emm) / (4.0 * FD_STEP * FD_STEP);
                hess[p * n_par + q] = v;
                hess[q * n_par + p] = v;
            }
        }

        let eig = jacobi(hess, n_par);
        let lambda_min = eig.values[0];
        let null_modes = eig.values.iter().filter(|v| v.abs() <= NULL_TOL).count();
        let lambda_min_projected = eig
            .values
            .iter()
            .copied()
            .find(|v| v.abs() > NULL_TOL)
            .unwrap_or(0.0);

        Self { lambda_min, lambda_min_projected, null_modes }
    }
}

/// Generalized (non-collinear) HF energy of the determinant obtained by rotating `cols` by
/// `theta` in the occupied→virtual block.
fn generalized_energy(
    chart: &Chart,
    cols: &[Vec<f64>],
    n_occ_so: usize,
    n_so: usize,
    theta: &[f64],
) -> f64 {
    let sites = chart.sites;
    let n_virt_so = n_so - n_occ_so;

    // exp(kappa) applied to the occupied columns only. kappa is antisymmetric with
    // kappa[occ i][virt a] = -theta, kappa[virt a][occ i] = +theta.
    let mut kappa = vec![0.0; n_so * n_so];
    for i in 0..n_occ_so {
        for a in 0..n_virt_so {
            let t = theta[i * n_virt_so + a];
            kappa[(n_occ_so + a) * n_so + i] = t;
            kappa[i * n_so + n_occ_so + a] = -t;
        }
    }
    let expk = matrix_exp(&kappa, n_so);

    // Rotated occupied spin-orbitals, in the site-spin basis.
    let mut occ_vecs = vec![vec![0.0; n_so]; n_occ_so];
    for (i, out) in occ_vecs.iter_mut().enumerate() {
        for (k, col) in cols.iter().enumerate() {
            let w = expk[k * n_so + i];
            if w == 0.0 {
                continue;
            }
            for m in 0..n_so {
                out[m] += w * col[m];
            }
        }
    }

    // Density matrix over spin-orbitals.
    let mut rho = vec![0.0; n_so * n_so];
    for v in &occ_vecs {
        for m in 0..n_so {
            if v[m] == 0.0 {
                continue;
            }
            for n in 0..n_so {
                rho[m * n_so + n] += v[m] * v[n];
            }
        }
    }

    // One-body: -t on each bond, within each spin block.
    let mut e = 0.0;
    for s in 0..2 {
        for m in 0..sites - 1 {
            let (a, b) = (s * sites + m, s * sites + m + 1);
            e += -chart.t * 2.0 * rho[a * n_so + b];
        }
    }
    // Interaction, generalized: U Σ_m [ρ_{m↑m↑} ρ_{m↓m↓} − ρ_{m↑m↓} ρ_{m↓m↑}].
    for m in 0..sites {
        let (up, dn) = (m, sites + m);
        e += chart.u
            * (rho[up * n_so + up] * rho[dn * n_so + dn] - rho[up * n_so + dn] * rho[dn * n_so + up]);
    }
    e
}

/// `exp(A)` by scaling and squaring with a Taylor series. `A` here is small and antisymmetric.
fn matrix_exp(a: &[f64], n: usize) -> Vec<f64> {
    let norm = a.iter().map(|x| x.abs()).fold(0.0, f64::max) * n as f64;
    let s = if norm > 0.5 { (norm / 0.5).log2().ceil() as u32 } else { 0 };
    let scale = 1.0 / (1u64 << s) as f64;

    let mut term: Vec<f64> = (0..n * n).map(|k| if k % (n + 1) == 0 { 1.0 } else { 0.0 }).collect();
    let mut acc = term.clone();
    for k in 1..=16 {
        let mut next = vec![0.0; n * n];
        for i in 0..n {
            for p in 0..n {
                let tp = term[i * n + p];
                if tp == 0.0 {
                    continue;
                }
                let ap = a[p * n..p * n + n].iter();
                for (j, aval) in ap.enumerate() {
                    next[i * n + j] += tp * aval * scale;
                }
            }
        }
        for v in next.iter_mut() {
            *v /= k as f64;
        }
        for (dst, src) in acc.iter_mut().zip(next.iter()) {
            *dst += src;
        }
        term = next;
    }
    for _ in 0..s {
        let mut sq = vec![0.0; n * n];
        for i in 0..n {
            for p in 0..n {
                let v = acc[i * n + p];
                if v == 0.0 {
                    continue;
                }
                for j in 0..n {
                    sq[i * n + j] += v * acc[p * n + j];
                }
            }
        }
        acc = sq;
    }
    acc
}
