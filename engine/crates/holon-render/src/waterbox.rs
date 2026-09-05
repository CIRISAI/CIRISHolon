//! THE LIQUID CELL (LIQUID-1 §0, `conformance/water_observatory/LIQUID1_PREREG.md`): 128
//! waters on a body-centred cubic lattice, in a cubic cell whose edge is fixed by the STATE
//! POINT's density and by nothing else.
//!
//! Three things are declared here and each is declared out loud, because a box builder is
//! where a "derived" campaign quietly acquires free parameters:
//!
//! 1. **The edge comes from the density, never from the lattice.** `L = (N·M / (N_A·rho))^(1/3)`
//!    — the volume 128 waters occupy at the state point, cube-rooted. The lattice's spacing
//!    is then `L / n_cells`, an OUTPUT. (M-VOLUME-SCALE: the grid count is a start, not a
//!    scale.)
//! 2. **The monomer is EMBED-1's pin, unrotated internally.** `holon_chem::embed::water_centers`
//!    at [`crate::field::WATER_PIN_R_BOHR`] and [`crate::field::WATER_PIN_THETA_RAD`] — the
//!    same geometry FIELD-1 derived the charge at and FIELD-2/3's scenes are built from, with
//!    the oxygen at the origin, so a site is an OXYGEN position.
//! 3. **The orientations are one declared stream.** The 64-bit LCG below, seeded once
//!    (M-FIXED-POINT-TRAJECTORY: one seed, declared, one builder).
//!
//! Constants and their source:
//!
//! | | | |
//! |---|---|---|
//! | `M_WATER_G_PER_MOL` | `18.015` | the freeze's own arithmetic (LIQUID-1 §0) |
//! | `AVOGADRO_PER_MOL` | `6.02214076e23` | SI, exact by the 2019 redefinition |
//! | `BOHR_ANGSTROM` | `0.529177210903` | CODATA 2018; the same digits `holon-lens` carries |
//!
//! The returned coordinates are LATTICE coordinates and are deliberately NOT wrapped into
//! `[0, L)`: a hydrogen of a corner molecule sits a bohr or two outside the cell, and folding
//! it would put it a box length from its own oxygen in the OPEN-box arithmetic the scene is
//! assembled under — the closure reading would then call it free and the boundary door would
//! refuse the cell over an artifact of the builder. The engine folds them on its first drift
//! step, where the minimum image makes the fold free of consequence.

use holon_chem::elements::{Species, HYDROGEN, OXYGEN};
use holon_chem::embed::water_centers;

/// Water's molar mass in grams per mole, as the freeze states it (LIQUID-1 §0).
pub const M_WATER_G_PER_MOL: f64 = 18.015;
/// Avogadro's number, per mole (SI, exact since 2019).
pub const AVOGADRO_PER_MOL: f64 = 6.02214076e23;
/// One bohr in angstroms (CODATA 2018).
pub const BOHR_ANGSTROM: f64 = 0.529177210903;

/// Knuth's MMIX multiplier and increment — the 64-bit linear congruential recurrence
/// `x <- 6364136223846793005·x + 1442695040888963407`, whose TOP 53 bits are taken as a
/// uniform on `[0, 1)`. The low bits of an LCG are the bad ones and this drops them; 53 is
/// the mantissa, so the map onto a double loses nothing and repeats nothing.
struct Lcg(u64);

impl Lcg {
    fn new(seed: u64) -> Self {
        Lcg(seed)
    }
    fn uniform(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 11) as f64) / ((1u64 << 53) as f64)
    }
}

/// The cubic cell edge in BOHR holding `n` waters at `density_g_cm3`.
///
/// `V = n·M / (N_A·rho)` in cm³; `1 cm = 1e8 angstrom`, so `V` in angstrom³ is that times
/// `1e24`; the edge is its cube root divided by [`BOHR_ANGSTROM`].
pub fn cell_edge_bohr(n: usize, density_g_cm3: f64) -> f64 {
    let v_cm3 = n as f64 * M_WATER_G_PER_MOL / (AVOGADRO_PER_MOL * density_g_cm3);
    let v_ang3 = v_cm3 * 1.0e24;
    v_ang3.cbrt() / BOHR_ANGSTROM
}

/// The `2·n_cells³` sites of a body-centred cubic lattice of `n_cells` cells per edge in a
/// cube of edge `l`: every cell's corner, and every cell's body centre.
///
/// The nearest-site distance is `(sqrt(3)/2)·(l/n_cells)` — corner to body centre — which is
/// what makes this a starting arrangement rather than a crystal: at the state point it is
/// `6.41` bohr, just outside water's first shell.
pub fn bcc_sites(n_cells: usize, l: f64) -> Vec<[f64; 3]> {
    let a = l / n_cells as f64;
    let mut out = Vec::with_capacity(2 * n_cells * n_cells * n_cells);
    for i in 0..n_cells {
        for j in 0..n_cells {
            for k in 0..n_cells {
                out.push([i as f64 * a, j as f64 * a, k as f64 * a]);
                out.push([(i as f64 + 0.5) * a, (j as f64 + 0.5) * a, (k as f64 + 0.5) * a]);
            }
        }
    }
    out
}

/// The z–y–z rotation for Euler angles `(alpha, beta, gamma)`, as a row-major 3×3.
fn euler_zyz(alpha: f64, beta: f64, gamma: f64) -> [[f64; 3]; 3] {
    let (ca, sa) = (alpha.cos(), alpha.sin());
    let (cb, sb) = (beta.cos(), beta.sin());
    let (cg, sg) = (gamma.cos(), gamma.sin());
    [
        [ca * cb * cg - sa * sg, -ca * cb * sg - sa * cg, ca * sb],
        [sa * cb * cg + ca * sg, -sa * cb * sg + ca * cg, sa * sb],
        [-sb * cg, sb * sg, cb],
    ]
}

/// THE BOX: `2·n_cells³` waters at `density_g_cm3`, one per BCC site, each at EMBED-1's pin
/// geometry with its oxygen ON the site and its orientation drawn from `seed`.
///
/// Returns `(species, positions, edge)` — species and positions interleaved `O, H, H` per
/// water in site order, and the cell edge in bohr.
///
/// **The orientation measure is DECLARED**, because "three Euler angles" does not by itself
/// say which three. Each site draws `u1, u2, u3` in order from the stream and takes
/// `alpha = 2 pi u1`, `beta = arccos(1 − 2 u2)`, `gamma = 2 pi u3` in the z–y–z convention.
/// The `arccos` is the point: drawing `beta` uniformly on `[0, pi]` instead would pile the
/// molecular axes up at the poles, and a box whose orientations are anisotropic is a box with
/// a direction in it that the state point never declared.
pub fn liquid_box(n_cells: usize, density_g_cm3: f64, seed: u64) -> (Vec<Species>, Vec<[f64; 3]>, f64) {
    let sites_per_cell = 2;
    let n = sites_per_cell * n_cells * n_cells * n_cells;
    let l = cell_edge_bohr(n, density_g_cm3);
    let sites = bcc_sites(n_cells, l);
    debug_assert_eq!(sites.len(), n);
    let mono = water_centers(crate::field::WATER_PIN_R_BOHR, crate::field::WATER_PIN_THETA_RAD);
    let mut rng = Lcg::new(seed);
    let mut species = Vec::with_capacity(3 * n);
    let mut pos = Vec::with_capacity(3 * n);
    for site in sites.iter() {
        let alpha = 2.0 * std::f64::consts::PI * rng.uniform();
        let beta = (1.0 - 2.0 * rng.uniform()).clamp(-1.0, 1.0).acos();
        let gamma = 2.0 * std::f64::consts::PI * rng.uniform();
        let r = euler_zyz(alpha, beta, gamma);
        for (m, c) in mono.iter().enumerate() {
            species.push(if m == 0 { OXYGEN } else { HYDROGEN });
            pos.push([
                site[0] + r[0][0] * c[0] + r[0][1] * c[1] + r[0][2] * c[2],
                site[1] + r[1][0] * c[0] + r[1][1] * c[1] + r[1][2] * c[2],
                site[2] + r[2][0] * c[0] + r[2][1] * c[1] + r[2][2] * c[2],
            ]);
        }
    }
    (species, pos, l)
}

/// THE FIRST PEAK of a radial distribution, declared: the smallest `r` at which `g` is above
/// one and strictly above both neighbouring bins.
///
/// `dr` is its resolution floor and belongs beside it in any record that quotes it
/// (M-FLOOR-UNSTAKED). No smoothing is applied — a smoothed peak position is a position in a
/// filter's coordinates and not in the histogram's. Lives here rather than in the runner so
/// the gate and the arm read the SAME peak.
pub fn first_peak(r: &[f64], g: &[f64]) -> Option<(f64, f64)> {
    for k in 1..g.len().saturating_sub(1) {
        if g[k] > 1.0 && g[k] > g[k - 1] && g[k] > g[k + 1] {
            return Some((r[k], g[k]));
        }
    }
    None
}

/// `1 bohr² per femtosecond` in `cm²/s` — the ONE conversion the campaign makes out of atomic
/// units, stated once so the arm and its gate cannot disagree about it.
///
/// The TIME unit is already spent by the time a caller reaches this: `holon_lens::lens::diffusion`
/// returns bohr² per femtosecond because `mean_lag_fs` multiplies the frames' own elapsed
/// ATOMIC time by `holon_lens::traj::AU_TIME_FS` (`holon-lens/src/traj.rs:52`). The engine
/// states the same unit in seconds as [`crate::sim::AU_TIME_S`] (`holon-render/src/sim.rs:91`).
/// What is left is the LENGTH, and it is the engine's own: [`crate::sim::BOHR_M`]
/// (`holon-render/src/sim.rs:89`), metres, times 100 for centimetres, squared, over `1e-15`
/// seconds.
pub fn bohr2_per_fs_to_cm2_per_s() -> f64 {
    let bohr_cm = crate::sim::BOHR_M * 100.0;
    bohr_cm * bohr_cm / 1.0e-15
}
