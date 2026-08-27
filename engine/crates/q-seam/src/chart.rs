//! The classical chart: a single Slater determinant — Boolean occupancy over spin-orbital modes,
//! which is `Core/ModeChart.lean`'s `OccState M` made executable.
//!
//! Spin-unrestricted Hartree–Fock, `Q_SEAM_PREREG.md` §2. Three fixed initial guesses, linear
//! mixing 0.3, converged to `1e-12`, and **the chart is the lowest-energy converged solution of
//! the three** — no adaptive restarts, no tuning. A configuration where none converges is VOID.
//!
//! The chart's one-body density matrix is idempotent by construction, so its mode occupations are
//! exactly `{0,1}`. That is precisely why it cannot see its own fractionality, and why the
//! certificate must be built from something else (§4). Gate G-C2 *checks* the idempotency rather
//! than trusting it.

use crate::dense::jacobi;

/// A1/T2's corrected SCF guess seed (the frozen text printed `0x5EAM`, which is not hexadecimal).
pub const GUESS_SEED: u64 = 0x5EA0;

const MIXING: f64 = 0.3;
const SCF_TOL: f64 = 1e-12;
const SCF_MAX_ITERS: usize = 5000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Guess {
    Uniform,
    Neel,
    Seeded,
}

pub const GUESSES: [Guess; 3] = [Guess::Uniform, Guess::Neel, Guess::Seeded];

#[derive(Clone, Debug)]
pub struct Chart {
    pub sites: usize,
    pub t: f64,
    pub u: f64,
    /// Site potential (Q7). All-zero is the Q5 chart exactly.
    pub potential: Vec<f64>,
    pub guess: Guess,
    /// Self-consistent site occupations, `[up, down]`.
    pub occupation: [Vec<f64>; 2],
    /// Orbital energies per spin, ascending.
    pub orbital_energy: [Vec<f64>; 2],
    /// Orbital coefficients: `orbitals[s][p][i]` is amplitude of orbital `p` on site `i`.
    pub orbitals: [Vec<Vec<f64>>; 2],
    /// One-body density matrices, row-major `N × N`, idempotent by construction.
    pub rdm: [Vec<f64>; 2],
    pub energy: f64,
    pub iterations: usize,
    pub converged: bool,
    /// G-C2: `‖ρ² − ρ‖_max`, measured.
    pub idempotency: f64,
}

fn hopping_matrix(sites: usize, t: f64, potential: &[f64]) -> Vec<f64> {
    let mut h = vec![0.0; sites * sites];
    for i in 0..sites {
        h[i * sites + i] = potential[i];
        if i + 1 < sites {
            h[i * sites + i + 1] = -t;
            h[(i + 1) * sites + i] = -t;
        }
    }
    h
}

fn initial_occupations(sites: usize, guess: Guess) -> [Vec<f64>; 2] {
    match guess {
        Guess::Uniform => [vec![0.5; sites], vec![0.5; sites]],
        Guess::Neel => {
            let up = (0..sites)
                .map(|i| if i % 2 == 0 { 0.75 } else { 0.25 })
                .collect::<Vec<_>>();
            let dn = up.iter().map(|x| 1.0 - x).collect();
            [up, dn]
        }
        Guess::Seeded => {
            // SplitMix64, the same generator the Lanczos start uses, at the pinned guess seed.
            let mut state = GUESS_SEED;
            let mut next = || {
                state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
                let mut z = state;
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64
            };
            let up = (0..sites).map(|_| 0.25 + 0.5 * next()).collect::<Vec<_>>();
            let dn = (0..sites).map(|_| 0.25 + 0.5 * next()).collect();
            [up, dn]
        }
    }
}

impl Chart {
    /// One SCF run from one guess. `converged` is reported, never assumed.
    pub fn solve(sites: usize, t: f64, u: f64, guess: Guess) -> Self {
        Self::solve_with(sites, t, u, &vec![0.0; sites], guess)
    }

    pub fn solve_with(sites: usize, t: f64, u: f64, potential: &[f64], guess: Guess) -> Self {
        let h = hopping_matrix(sites, t, potential);
        let n_occ = sites / 2;
        let mut occ = initial_occupations(sites, guess);

        let mut orbital_energy = [vec![0.0; sites], vec![0.0; sites]];
        let mut orbitals: [Vec<Vec<f64>>; 2] = [Vec::new(), Vec::new()];
        let mut rdm = [vec![0.0; sites * sites], vec![0.0; sites * sites]];
        let mut iterations = 0;
        let mut converged = false;

        for it in 0..SCF_MAX_ITERS {
            iterations = it + 1;
            let mut new_occ = [vec![0.0; sites], vec![0.0; sites]];

            for s in 0..2 {
                let mut fock = h.clone();
                for i in 0..sites {
                    fock[i * sites + i] += u * occ[1 - s][i];
                }
                let eig = jacobi(fock, sites);
                orbital_energy[s] = eig.values.clone();
                orbitals[s] = eig.vectors.clone();

                let mut d = vec![0.0; sites * sites];
                for p in 0..n_occ {
                    let v = &eig.vectors[p];
                    for i in 0..sites {
                        for j in 0..sites {
                            d[i * sites + j] += v[i] * v[j];
                        }
                    }
                }
                for i in 0..sites {
                    new_occ[s][i] = d[i * sites + i];
                }
                rdm[s] = d;
            }

            let delta = (0..2)
                .flat_map(|s| (0..sites).map(move |i| (s, i)))
                .map(|(s, i)| (new_occ[s][i] - occ[s][i]).abs())
                .fold(0.0, f64::max);

            for s in 0..2 {
                for i in 0..sites {
                    occ[s][i] += MIXING * (new_occ[s][i] - occ[s][i]);
                }
            }

            if delta <= SCF_TOL {
                converged = true;
                break;
            }
        }

        // E = Σ_σ Σ_occ ε − U Σ_i n_i↑ n_i↓ (the mean-field double-counting correction).
        let sum_eps: f64 = (0..2)
            .map(|s| orbital_energy[s][..n_occ].iter().sum::<f64>())
            .sum();
        let double_count: f64 = (0..sites).map(|i| occ[0][i] * occ[1][i]).sum();
        let energy = sum_eps - u * double_count;

        let idempotency = (0..2)
            .map(|s| {
                let mut worst = 0.0f64;
                for i in 0..sites {
                    for j in 0..sites {
                        let mut r2 = 0.0;
                        for k in 0..sites {
                            r2 += rdm[s][i * sites + k] * rdm[s][k * sites + j];
                        }
                        worst = worst.max((r2 - rdm[s][i * sites + j]).abs());
                    }
                }
                worst
            })
            .fold(0.0, f64::max);

        Self {
            sites,
            t,
            u,
            potential: potential.to_vec(),
            guess,
            occupation: occ,
            orbital_energy,
            orbitals,
            rdm,
            energy,
            iterations,
            converged,
            idempotency,
        }
    }

    /// The chart, per §2: the lowest-energy converged solution of the three pinned guesses.
    /// `None` when none of the three converges — the configuration is then VOID, not refused.
    pub fn best(sites: usize, t: f64, u: f64) -> Option<Self> {
        Self::best_with(sites, t, u, &vec![0.0; sites])
    }

    pub fn best_with(sites: usize, t: f64, u: f64, potential: &[f64]) -> Option<Self> {
        GUESSES
            .iter()
            .map(|&g| Chart::solve_with(sites, t, u, potential, g))
            .filter(|c| c.converged)
            .min_by(|a, b| a.energy.partial_cmp(&b.energy).unwrap())
    }

    /// The chart's per-site reflection asymmetry — D1b's quantity, theorem-pinned to zero by
    /// `Q7_SEAM_PREREG.md` §2.3 for a reflection-symmetric potential.
    pub fn reflection_asymmetry(&self) -> Vec<f64> {
        let d = self.density();
        (0..self.sites).map(|i| (d[i] - d[self.sites - 1 - i]).abs()).collect()
    }

    /// D2's per-site self-residual weight, closed form (`Q7_SEAM_PREREG.md` §7):
    /// `σ_m² = U²·n↑(1−n↑)·n↓(1−n↓)`. Its four zeros are derived there, and the fourth — a fully
    /// spin-polarised site — is where the chart lies maximally.
    pub fn sigma(&self) -> Vec<f64> {
        (0..self.sites)
            .map(|m| {
                let (a, b) = (self.occupation[0][m], self.occupation[1][m]);
                (self.u * self.u * a * (1.0 - a) * b * (1.0 - b)).max(0.0).sqrt()
            })
            .collect()
    }

    /// The chart's own HOMO–LUMO gap, the denominator of D2's local energy estimate.
    pub fn gap(&self) -> f64 {
        let n_occ = self.sites / 2;
        (0..2)
            .map(|s| self.orbital_energy[s][n_occ] - self.orbital_energy[s][n_occ - 1])
            .fold(f64::INFINITY, f64::min)
    }

    pub fn energy_per_site(&self) -> f64 {
        self.energy / (self.sites as f64 * self.t)
    }

    /// Chart prediction for O2: the product of mean-field occupations, which is what a
    /// determinant says double occupancy is.
    pub fn double_occ_mean(&self) -> f64 {
        (0..self.sites).map(|i| self.occupation[0][i] * self.occupation[1][i]).sum::<f64>()
            / self.sites as f64
    }

    pub fn density(&self) -> Vec<f64> {
        (0..self.sites).map(|i| self.occupation[0][i] + self.occupation[1][i]).collect()
    }

    /// The chart's magnetization. Where the exact value is zero by Lieb's theorem, this **is**
    /// the chart's error in O4 — computable without the exact state at all, which is the fact
    /// amendment A1/H1 builds criterion C3 on.
    pub fn magnetization(&self) -> Vec<f64> {
        (0..self.sites).map(|i| self.occupation[0][i] - self.occupation[1][i]).collect()
    }

    pub fn bond(&self) -> Vec<f64> {
        (0..self.sites - 1)
            .map(|i| {
                2.0 * (self.rdm[0][i * self.sites + i + 1] + self.rdm[1][i * self.sites + i + 1])
            })
            .collect()
    }

    /// How far the chart's own order parameters sit from the symmetries the reference provably
    /// keeps: `(spin, particle-hole, reflection)`. C3 reads exactly these three.
    pub fn symmetry_breaking(&self) -> (f64, f64, f64) {
        let spin = self.magnetization().iter().map(|x| x.abs()).fold(0.0, f64::max);
        let ph = (0..2)
            .flat_map(|s| self.occupation[s].iter())
            .map(|x| (x - 0.5).abs())
            .fold(0.0, f64::max);
        let d = self.density();
        let refl = (0..self.sites)
            .map(|i| (d[i] - d[self.sites - 1 - i]).abs())
            .fold(0.0, f64::max);
        (spin, ph, refl)
    }
}
