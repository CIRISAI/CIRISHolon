//! THE EWALD SUM — EWALD-1 (`conformance/water_observatory/EWALD_PREREG.md`).
//!
//! The `1/r` lattice sum of a periodic cell, split so that both halves converge absolutely:
//! `erfc(αr)/r` summed in real space over minimum-image pairs inside `r_c ≤ L_min/2`, plus
//! `erf(αr)/r` summed in reciprocal space over the cell's wave-vectors, minus the self term
//! `α/√π·Σq²` each charge acquires from its own Gaussian, minus a correction that removes
//! the DIRECT interaction of every pair inside one unit — exactly the pairs the open-box
//! field skips — while KEEPING that unit's interaction with its own periodic images, which
//! in a wrapped cell is a real interaction and not an artefact.
//!
//! Prior art: Ewald 1921 (the split); de Leeuw, Perram and Smith 1980 (the conditionally
//! convergent sum, and the conducting — "tin-foil" — boundary convention used here, which
//! is why there is NO surface dipole term); Essmann et al. 1995 (the parameter estimates
//! this module's [`params_for`] uses). Not compared to any other code's numbers or runtime.
//!
//! # What this module is NOT
//!
//! It is a pure function of positions, charges, unit ids and the cell. It does not know
//! about [`crate::sim::Sim`], does not read or write a ledger row, and does not decide when
//! the periodic branch is taken. The integration is separate and deliberate: a lattice sum
//! that can only be exercised through a stepping engine is a lattice sum whose invariances
//! nobody has checked directly.
//!
//! # The four sectors, and why the reading names all four
//!
//! ```text
//! E = e_real + e_recip + e_self + e_excluded          (exactly; see EwaldReading)
//! ```
//!
//! Each is a different kind of mistake. `e_self` is a constant of the charges and the
//! split; dropping it is a constant offset that a force gate cannot see, which is why it is
//! one of the two plants. `e_excluded` is the only sector that knows what a UNIT is;
//! dropping it leaves every molecule's own charges interacting through the reciprocal sum,
//! which is a large error that also cannot be seen from a force-derivative gate alone,
//! which is why it is the other. Reporting them separately is what makes both plants
//! observable in the sector they act on rather than only in the total.
//!
//! # The neutralising background
//!
//! A cell with net charge `Σq ≠ 0` has no finite Coulomb energy; the standard repair is a
//! uniform background of `−Σq` spread through the cell, which contributes
//! `−π(Σq)²/(2Vα²)`. This module includes it and does NOT panic on a charged cell — a
//! refusal there would make the module unusable for the one case where the term matters —
//! but the term is folded into `e_recip`, because it IS the `k = 0` term of the reciprocal
//! sum, and it is exactly `0` for the neutral cells the engine builds.
//!
//! # The virial convention
//!
//! `virial` is the engine's `Σ r·dU/dr` (see [`crate::sim::Sim::pressure`], which reads
//! `(2K − W)/3V`), obtained as `W = 3V·dE/dV` at fixed SCALED coordinates. For a Coulomb
//! system that has a closed form the gate exploits: `E` is homogeneous of degree `−1` under
//! a uniform scaling, so `W = −E` exactly, and the analytic sum below is checked against a
//! finite difference of the volume rather than against that identity.

use crate::cells::BoxGeom;
use std::f64::consts::PI;

/// `√π`, to the last bit of an `f64`.
const SQRT_PI: f64 = 1.772_453_850_905_516;
/// `2/√π`, the derivative constant of the error function.
const TWO_OVER_SQRT_PI: f64 = 1.128_379_167_095_512_6;

/// The split, the real-space radius, the reciprocal grid, and the accuracy they came from.
///
/// Carried as a value rather than recomputed at each call so that a caller can hold `α`
/// fixed across a volume change — which is exactly what the virial's finite-difference gate
/// needs, because `α` is a parameter of the SPLIT and not of the physics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EwaldParams {
    /// The splitting parameter, in inverse bohr. Larger moves work into the reciprocal sum.
    pub alpha: f64,
    /// The real-space cutoff, in bohr. Must be at most half the shortest edge or the
    /// minimum-image convention is not the minimum image (see [`BoxGeom::delta`]).
    pub r_cut: f64,
    /// The reciprocal grid half-width per axis: `|n_axis| ≤ k_max[axis]`.
    pub k_max: [i32; 3],
    /// The accuracy target the three above were derived from.
    pub accuracy: f64,
}

/// The engine's declared accuracy target. A number, not a fit.
pub const DEFAULT_ACCURACY: f64 = 1e-8;

/// The standard parameter estimates from the cell and ONE accuracy target (freeze §0).
///
/// `r_c = L_min/2` — the largest radius at which the minimum image is still THE minimum
/// image, and therefore the choice that puts the most of the sum in the cheap sector.
/// `α = √(−ln ε)/r_c` makes the neglected real-space tail `erfc(α r_c) ≈ ε`, and
/// `k_max = ⌈α L √(−ln ε)/π⌉` per axis makes the neglected reciprocal tail the same size.
///
/// The two estimates are tied to each other through `√(−ln ε)`, so they are only both true
/// at the `α` this function returns: a caller who scales `α` down while holding `r_c` at
/// `L_min/2` degrades the REAL-space accuracy to `erfc(α r_c)`, which is no longer `ε`. That
/// is not a defect of the sum — it is what the split parameter means — and it is why the
/// α-invariance gate is a statement about a RANGE of `α` and not about all of them.
pub fn params_for(cell: [f64; 3], accuracy: f64) -> EwaldParams {
    let s = (-accuracy.ln()).sqrt();
    let l_min = cell[0].min(cell[1]).min(cell[2]);
    let r_cut = 0.5 * l_min;
    let alpha = s / r_cut;
    let mut k_max = [0i32; 3];
    for a in 0..3 {
        let raw = alpha * cell[a] * s / PI;
        k_max[a] = raw.ceil().max(1.0) as i32;
    }
    EwaldParams {
        alpha,
        r_cut,
        k_max,
        accuracy,
    }
}

/// One evaluation of the sum: the answer, and enough of its parts to convict a mistake.
#[derive(Clone, Debug)]
pub struct EwaldReading {
    /// `e_real + e_recip + e_self + e_excluded`, in hartree. Exactly that sum.
    pub energy: f64,
    /// Per-atom force, hartree per bohr, in the same order as `pos`.
    pub forces: Vec<[f64; 3]>,
    /// `Σ r·dU/dr` in the engine's convention — see the module header.
    pub virial: f64,
    /// Minimum-image pairs of DIFFERENT units inside `r_cut` that the real sum evaluated.
    pub real_pairs: u64,
    /// Wave-vectors `k ≠ 0` the reciprocal sum actually summed.
    pub k_vectors: u64,
    /// `Σ q_i q_j erfc(α r)/r` over those pairs.
    pub e_real: f64,
    /// `(2π/V) Σ_{k≠0} exp(−k²/4α²)/k² |S(k)|²`, PLUS the neutralising background term
    /// `−π(Σq)²/(2Vα²)` (the `k = 0` term), which is `0` on a neutral cell.
    pub e_recip: f64,
    /// `−α/√π Σ q²`, or `0` under [`EwaldPlant::DropSelfTerm`].
    pub e_self: f64,
    /// `−Σ_{same unit} q_i q_j erf(α r)/r` — the reciprocal sum's account of each unit's own
    /// charges interacting directly, removed. `0` under
    /// [`EwaldPlant::SkipExclusionCorrection`], which is what makes that plant's carrier
    /// readable: take it from the UNPLANTED reading of the same scene.
    pub e_excluded: f64,
}

/// Deliberate defects for the gates (EWALD_PREREG §3).
///
/// Each names the sector it acts on, and each is a DROP rather than a perturbation, so the
/// miss it produces is exactly the sector it removed and nothing else.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EwaldPlant {
    #[default]
    None,
    /// The self term omitted (sector: `e_self`; the lattice-sum gate E2).
    DropSelfTerm,
    /// Each unit's own charges left interacting through the reciprocal sum (sector:
    /// `e_excluded`; the large-cell-limit gate E4).
    SkipExclusionCorrection,
}

/// The error function, to the last few bits.
///
/// Two branches, and the crossover is where each stops being the accurate one. Below `2`
/// the all-positive series `erf(x) = (2x/√π) e^{−x²} Σ (2x²)ⁿ/(1·3···(2n+1))` has no
/// cancellation at all (every term positive), so its relative accuracy is the accumulated
/// rounding of ~30 additions. Above `2` that series is still correct but `erfc` would be
/// formed as `1 − erf` from two nearly equal numbers, so the continued fraction is used
/// instead and `erf` is formed from IT. Rust's `std` has neither function; the
/// Numerical-Recipes Chebyshev `erfcc` is ~1e-7 relative and would have been the dominant
/// error in every force this module returns.
#[inline]
pub fn erf(x: f64) -> f64 {
    if x.abs() < 2.0 {
        erf_series(x)
    } else if x > 0.0 {
        1.0 - erfc_continued_fraction(x)
    } else {
        erfc_continued_fraction(-x) - 1.0
    }
}

/// The complementary error function, `1 − erf(x)`, computed so that it stays accurate
/// RELATIVE to itself out where it is tiny — which is the whole point of the Ewald split.
#[inline]
pub fn erfc(x: f64) -> f64 {
    if x >= 2.0 {
        erfc_continued_fraction(x)
    } else if x <= -2.0 {
        2.0 - erfc_continued_fraction(-x)
    } else {
        1.0 - erf_series(x)
    }
}

/// `erf` by its all-positive series. Accurate for `|x| ≲ 3`; used below `2`.
fn erf_series(x: f64) -> f64 {
    let x2 = x * x;
    let mut term = 1.0f64;
    let mut sum = 1.0f64;
    let mut n = 0usize;
    loop {
        n += 1;
        term *= 2.0 * x2 / (2 * n + 1) as f64;
        sum += term;
        // Every term is positive, so `sum` only grows and the test is a true bound on the
        // tail rather than a hope about one signed term.
        if term <= sum * 1e-18 || n >= 200 {
            break;
        }
    }
    TWO_OVER_SQRT_PI * x * (-x2).exp() * sum
}

/// `erfc(x)` for `x > 0` by the continued fraction (Abramowitz & Stegun 7.1.14),
/// `√π e^{x²} erfc(x) = 1/(x + (1/2)/(x + 1/(x + (3/2)/(x + …))))`, evaluated by modified
/// Lentz. Convergent for every `x > 0` and fast where it is used.
fn erfc_continued_fraction(x: f64) -> f64 {
    const TINY: f64 = 1e-300;
    let mut f = x;
    if f == 0.0 {
        f = TINY;
    }
    let mut c = f;
    let mut d = 0.0f64;
    for n in 1..=400 {
        let a = 0.5 * n as f64;
        d = x + a * d;
        if d == 0.0 {
            d = TINY;
        }
        c = x + a / c;
        if c == 0.0 {
            c = TINY;
        }
        d = 1.0 / d;
        let delta = c * d;
        f *= delta;
        if (delta - 1.0).abs() < 1e-17 {
            break;
        }
    }
    (-x * x).exp() / (SQRT_PI * f)
}

/// THE SUM. Positions, charges, unit ids and an orthorhombic cell in; energy, per-atom
/// forces, the scalar virial and the two work counts out.
///
/// `unit_of[i] == unit_of[j]` means the pair is INSIDE one molecule: it takes no real-space
/// term and gets the exclusion correction instead, which is the same rule
/// `Sim::accumulate_field` applies with `charge_row`. A zero charge is skipped outright, so
/// free atoms (which the engine gives charge `0.0` and unit `u32::MAX`) cost nothing and
/// cannot be confused with each other by sharing that sentinel.
///
/// Separations come from [`BoxGeom::delta`] — the engine's one implementation of the
/// minimum image — with `periodic = true`, since a lattice sum of an open box is not a
/// thing this function can be asked for.
pub fn ewald(
    pos: &[[f64; 3]],
    charge: &[f64],
    unit_of: &[u32],
    cell: [f64; 3],
    p: &EwaldParams,
    plant: EwaldPlant,
) -> EwaldReading {
    let n = pos.len();
    assert_eq!(charge.len(), n, "one charge per position");
    assert_eq!(unit_of.len(), n, "one unit id per position");

    let geom = BoxGeom::new(cell[0], cell[1], cell[2], true);
    let v = cell[0] * cell[1] * cell[2];
    let alpha = p.alpha;
    let two_a_over_sqrt_pi = 2.0 * alpha / SQRT_PI;
    let rc2 = p.r_cut * p.r_cut;

    let mut forces = vec![[0.0f64; 3]; n];
    let mut virial = 0.0f64;
    let mut e_real = 0.0f64;
    let mut e_excluded = 0.0f64;
    let mut real_pairs = 0u64;

    // ---- the two pair sectors, in one loop so the minimum image is taken once per pair.
    for i in 0..n {
        let qi = charge[i];
        if qi == 0.0 {
            continue;
        }
        for j in (i + 1)..n {
            let qj = charge[j];
            if qj == 0.0 {
                continue;
            }
            // `delta(a, b) = b − a`, so this is `r_i − r_j` — the direction the open-box
            // field's `F_i = q_i q_j (r_i − r_j)/r³` is written in.
            let (dx, dy, dz) = geom.delta(
                (pos[j][0], pos[j][1], pos[j][2]),
                (pos[i][0], pos[i][1], pos[i][2]),
            );
            let r2 = dx * dx + dy * dy + dz * dz;
            // The engine's floor (`cells.rs`), for the same reason: two atoms at the same
            // point are a broken scene, not a physics regime, and a NaN hides that.
            let r = r2.sqrt().max(1e-9);
            let qq = qi * qj;
            let same_unit = unit_of[i] == unit_of[j];

            // `g` is defined by `F_i = g·(r_i − r_j)`, i.e. `g = −U'(r)/r`. The virial then
            // follows from the SAME number: `r·U'(r) = −g·r²`. One derivative, two uses —
            // the alternative is two expressions that can disagree.
            let g = if same_unit {
                if plant == EwaldPlant::SkipExclusionCorrection {
                    continue;
                }
                let ex = erf(alpha * r);
                let gauss = (-(alpha * r) * (alpha * r)).exp();
                e_excluded += -qq * ex / r;
                qq * (two_a_over_sqrt_pi * gauss / r2 - ex / (r2 * r))
            } else {
                if r2 > rc2 {
                    continue;
                }
                real_pairs += 1;
                let ec = erfc(alpha * r);
                let gauss = (-(alpha * r) * (alpha * r)).exp();
                e_real += qq * ec / r;
                qq * (ec / (r2 * r) + two_a_over_sqrt_pi * gauss / r2)
            };

            forces[i][0] += g * dx;
            forces[i][1] += g * dy;
            forces[i][2] += g * dz;
            forces[j][0] -= g * dx;
            forces[j][1] -= g * dy;
            forces[j][2] -= g * dz;
            virial -= g * r2;
        }
    }

    // ---- the reciprocal sector.
    //
    // The FULL box of wave-vectors `|n_axis| ≤ k_max[axis]`, `k ≠ 0`, is summed — not the
    // half-space with a factor of two — so that `k_vectors` is the count of terms actually
    // evaluated and nothing has to be believed about a symmetry that was assumed rather
    // than used.
    let mut e_recip = 0.0f64;
    let mut k_vectors = 0u64;
    let inv_4a2 = 1.0 / (4.0 * alpha * alpha);
    let inv_2a2 = 0.5 / (alpha * alpha);
    let two_pi_over_v = 2.0 * PI / v;
    let mut cos_i = vec![0.0f64; n];
    let mut sin_i = vec![0.0f64; n];
    for nx in -p.k_max[0]..=p.k_max[0] {
        let kx = 2.0 * PI * nx as f64 / cell[0];
        for ny in -p.k_max[1]..=p.k_max[1] {
            let ky = 2.0 * PI * ny as f64 / cell[1];
            for nz in -p.k_max[2]..=p.k_max[2] {
                if nx == 0 && ny == 0 && nz == 0 {
                    continue;
                }
                let kz = 2.0 * PI * nz as f64 / cell[2];
                let k2 = kx * kx + ky * ky + kz * kz;
                let a = (-k2 * inv_4a2).exp() / k2;
                k_vectors += 1;

                // The structure factor `S(k) = Σ q_j e^{i k·r_j}`, over EVERY charge —
                // including the ones inside one unit, whose direct term the exclusion
                // sector above has already removed.
                let mut s_cos = 0.0f64;
                let mut s_sin = 0.0f64;
                for m in 0..n {
                    let ph = kx * pos[m][0] + ky * pos[m][1] + kz * pos[m][2];
                    let (s, c) = ph.sin_cos();
                    cos_i[m] = c;
                    sin_i[m] = s;
                    s_cos += charge[m] * c;
                    s_sin += charge[m] * s;
                }
                let ek = two_pi_over_v * a * (s_cos * s_cos + s_sin * s_sin);
                e_recip += ek;
                // `dE_k/dλ` under `r → λr`, `k → k/λ`, `V → λ³V` at `λ = 1`: the `|S|²` is
                // invariant (`k·r` is), so only the `1/λ` prefactor and the Gaussian move.
                virial += ek * (k2 * inv_2a2 - 1.0);

                let pref = 2.0 * two_pi_over_v * a;
                for m in 0..n {
                    let f = pref * charge[m] * (sin_i[m] * s_cos - cos_i[m] * s_sin);
                    forces[m][0] += f * kx;
                    forces[m][1] += f * ky;
                    forces[m][2] += f * kz;
                }
            }
        }
    }

    // ---- the self term: every charge's own Gaussian, which the reciprocal sum counted.
    let sum_q2: f64 = charge.iter().map(|q| q * q).sum();
    let e_self = if plant == EwaldPlant::DropSelfTerm {
        0.0
    } else {
        -alpha / SQRT_PI * sum_q2
    };

    // ---- the neutralising background, folded into the reciprocal sector as its `k = 0`
    // term. Zero on a neutral cell; `∝ 1/V`, so its volume derivative is `−3` times it.
    let q_tot: f64 = charge.iter().sum();
    let e_background = -PI * q_tot * q_tot / (2.0 * v * alpha * alpha);
    e_recip += e_background;
    virial -= 3.0 * e_background;

    EwaldReading {
        energy: e_real + e_recip + e_self + e_excluded,
        forces,
        virial,
        real_pairs,
        k_vectors,
        e_real,
        e_recip,
        e_self,
        e_excluded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The error function against its own definition, with no value taken from anywhere:
    /// `erf' = (2/√π) e^{−x²}` by central difference, `erf + erfc = 1`, `erf(−x) = −erf(x)`,
    /// and the two branches agreeing across their crossover.
    #[test]
    fn the_error_function_is_its_own_integral() {
        let h = 1e-5;
        for k in 1..=60 {
            let x = 0.05 * k as f64;
            let fd = (erf(x + h) - erf(x - h)) / (2.0 * h);
            let exact = TWO_OVER_SQRT_PI * (-x * x).exp();
            assert!(
                (fd - exact).abs() < 1e-10,
                "erf' at {x}: finite difference {fd}, exact {exact}"
            );
            assert!((erf(x) + erfc(x) - 1.0).abs() < 2e-16, "erf + erfc at {x}");
            assert!((erf(-x) + erf(x)).abs() < 1e-16, "erf is odd at {x}");
        }
        // The crossover at 2 is a branch boundary, so it is where a disagreement would hide.
        for x in [1.999_999_999f64, 2.0, 2.000_000_001] {
            let series = erf_series(x);
            let cf = 1.0 - erfc_continued_fraction(x);
            assert!(
                (series - cf).abs() < 1e-15,
                "the branches disagree at {x}: series {series}, continued fraction {cf}"
            );
        }
    }

    /// And against the error function's tabulated values — mathematical constants, in the
    /// same class as the Madelung constant the freeze declares, and the only numbers in this
    /// module that did not come from the engine. `erfc` is checked RELATIVE to itself, which
    /// is the property the split needs and the property a `1 − erf` implementation loses.
    #[test]
    fn the_error_function_matches_its_tabulated_values() {
        let table: [(f64, f64, f64); 8] = [
            (0.25, 0.276_326_390_168_236_9, 0.723_673_609_831_763_1),
            (0.5, 0.520_499_877_813_046_5, 0.479_500_122_186_953_5),
            (1.0, 0.842_700_792_949_714_9, 0.157_299_207_050_285_13),
            (1.5, 0.966_105_146_475_310_8, 0.033_894_853_524_689_274),
            (1.9, 0.992_790_429_235_257_5, 0.007_209_570_764_742_532_5),
            (2.0, 0.995_322_265_018_952_7, 0.004_677_734_981_047_265),
            (3.0, 0.999_977_909_503_001_4, 2.209_049_699_858_543_8e-5),
            (5.0, 0.999_999_999_998_462_6, 1.537_459_794_428_035_1e-12),
        ];
        let mut worst_erf = 0.0f64;
        let mut worst_erfc = 0.0f64;
        for (x, e, ec) in table {
            worst_erf = worst_erf.max((erf(x) - e).abs() / e);
            worst_erfc = worst_erfc.max((erfc(x) - ec).abs() / ec);
        }
        eprintln!("erf worst relative {worst_erf:.3e}, erfc worst relative {worst_erfc:.3e}");
        assert!(worst_erf < 1e-13, "erf relative error {worst_erf}");
        assert!(worst_erfc < 1e-13, "erfc relative error {worst_erfc}");
    }

    /// The parameter estimates, on the engine's 20-bohr cube.
    #[test]
    fn the_parameters_come_from_the_cell_and_one_number() {
        let p = params_for([20.0, 20.0, 20.0], DEFAULT_ACCURACY);
        assert_eq!(p.r_cut, 10.0);
        assert_eq!(p.k_max, [12, 12, 12]);
        // `α r_c = √(−ln ε)`, which is what makes the neglected real-space tail `ε`.
        assert!((erfc(p.alpha * p.r_cut) - DEFAULT_ACCURACY).abs() < 1e-8);
        // A cell that is not a cube gets a grid that is not a cube.
        let q = params_for([20.0, 20.0, 10.0], DEFAULT_ACCURACY);
        assert_eq!(q.r_cut, 5.0);
        assert!(q.k_max[0] == q.k_max[1] && q.k_max[2] * 2 <= q.k_max[0] + 1);
    }

    /// The reading's four sectors ARE the energy — no fifth term hiding in the total.
    #[test]
    fn the_sectors_sum_to_the_energy() {
        let pos = [[1.0, 1.0, 1.0], [3.0, 1.5, 1.0], [6.0, 6.0, 6.0]];
        let charge = [1.0, -1.0, 0.5];
        let unit_of = [0u32, 0, 1];
        let p = params_for([12.0, 12.0, 12.0], DEFAULT_ACCURACY);
        let r = ewald(&pos, &charge, &unit_of, [12.0, 12.0, 12.0], &p, EwaldPlant::None);
        let s = r.e_real + r.e_recip + r.e_self + r.e_excluded;
        assert!((r.energy - s).abs() < 1e-15, "energy {} vs sectors {s}", r.energy);
        // A net charge of +0.5 means the background term is live and negative.
        assert!(r.e_recip.is_finite() && r.energy.is_finite());
    }
}
