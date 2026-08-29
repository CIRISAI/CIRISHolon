//! Grid sizing for the SATURATION-1 trimer table: build the finest candidate once, then
//! measure the held-out interpolation error of every coarser grid it contains.
//!
//! The coarse grids are SUBSETS of the fine one (node `i` of the 21-grid is node `2i` of
//! the 41-grid), so one electronic-structure pass serves the whole sweep. The side axis
//! is stretched by `r = R_LO + (R_HI - R_LO) (e^{a tau} - 1)/(e^a - 1)`, which puts the
//! knots where the surface is steep without the unbounded `dtau/dr` a power law would
//! have at the lower edge — a coordinate singularity there would be a force singularity.
use holon_chem::trimer::{self, cr_weights};
use std::time::Instant;

const R_LO: f64 = trimer::R_LO;
const R_HI: f64 = trimer::R_HI;
// THIRD COORDINATE UNDER TEST: `c = sqrt(1 - u)`, a fixed monotone reparametrisation of
// the cosine. At `x = y` the third side is `z = x sqrt(2) c` EXACTLY, so a uniform `c`
// grid is a uniform `z` grid there — the resolution the raw cosine loses at small `z` —
// while staying a box, staying symmetric in `x <-> y`, and staying smooth (unlike a
// coordinate normalised by `min(x, y)`, which kinks on the diagonal).
const C_LO: f64 = 0.632_455_532_033_675_9; // sqrt(1 - 0.6)
const C_HI: f64 = core::f64::consts::SQRT_2; // sqrt(1 + 1)
const U_LO: f64 = C_LO;
const U_HI: f64 = C_HI;
const NR: usize = 41;
const NU: usize = 25;

/// The staked draw for the held-out set.
const T1_SEED: u64 = 0x5341_5455_5241_5431; // "SATURAT1"

fn lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

fn r_of_tau(a: f64, tau: f64) -> f64 {
    if a == 0.0 {
        return R_LO + (R_HI - R_LO) * tau;
    }
    R_LO + (R_HI - R_LO) * ((a * tau).exp() - 1.0) / (a.exp() - 1.0)
}

fn tau_of_r(a: f64, r: f64) -> f64 {
    if a == 0.0 {
        return (r - R_LO) / (R_HI - R_LO);
    }
    (1.0 + (r - R_LO) * (a.exp() - 1.0) / (R_HI - R_LO)).ln() / a
}

fn eval(v: &[f64], nr: usize, nu: usize, a: f64, x: f64, y: f64, u: f64) -> f64 {
    let tx = tau_of_r(a, x) * (nr - 1) as f64;
    let ty = tau_of_r(a, y) * (nr - 1) as f64;
    let tu = (u - U_LO) / (U_HI - U_LO) * (nu - 1) as f64;
    let (bx, wx, _) = cr_weights(nr, tx.clamp(0.0, (nr - 1) as f64));
    let (by, wy, _) = cr_weights(nr, ty.clamp(0.0, (nr - 1) as f64));
    let (bu, wu, _) = cr_weights(nu, tu.clamp(0.0, (nu - 1) as f64));
    let mut acc = 0.0;
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                acc += wx[i] * wy[j] * wu[k] * v[((bx + i) * nr + (by + j)) * nu + (bu + k)];
            }
        }
    }
    acc
}

fn main() {
    let a: f64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(2.0);
    println!("stretch a = {a}   (0 = uniform in r)");
    let e_h = trimer::atom_energy();
    let t0 = Instant::now();
    let mut fine = vec![0.0f64; NR * NR * NU];
    let rs: Vec<f64> = (0..NR)
        .map(|i| r_of_tau(a, i as f64 / (NR - 1) as f64))
        .collect();
    let vc: Vec<f64> = rs.iter().map(|&r| trimer::pair_energy(r)).collect();
    let mut solves = 0usize;
    for i in 0..NR {
        for j in i..NR {
            let (x, y) = (rs[i], rs[j]);
            for k in 0..NU {
                let cc = C_LO + (C_HI - C_LO) * k as f64 / (NU - 1) as f64;
                let u = 1.0 - cc * cc;
                let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();
                let s = (1.0 - u * u).max(0.0).sqrt();
                let e3 =
                    trimer::hydrogen_energy(&[[0.0, 0.0, 0.0], [x, 0.0, 0.0], [y * u, y * s, 0.0]]);
                let d = e3 + 3.0 * e_h - (vc[i] + vc[j] + trimer::pair_energy(z));
                fine[(i * NR + j) * NU + k] = d;
                fine[(j * NR + i) * NU + k] = d;
                solves += 1;
            }
        }
    }
    let build = t0.elapsed().as_secs_f64();
    println!(
        "fine grid {NR}x{NR}x{NU}: {solves} unique dE3 solves in {build:.2} s ({:.0} us each)",
        build * 1e6 / solves as f64
    );
    println!("  h_r near r=1.15: {:.4}   near r=7: {:.4}",
        r_of_tau(a, tau_of_r(a, 1.15) + 1.0 / (NR - 1) as f64) - 1.15,
        r_of_tau(a, tau_of_r(a, 7.0) + 1.0 / (NR - 1) as f64) - 7.0);

    let mut st = T1_SEED;
    let mut pts = Vec::new();
    while pts.len() < 256 {
        let x = 0.9 + (R_HI - 0.9) * lcg(&mut st);
        let y = x + (R_HI - x) * lcg(&mut st);
        let umax = x / (2.0 * y);
        let u = -1.0 + (umax + 1.0) * lcg(&mut st);
        pts.push((x, y, u));
    }

    for (nr, nu) in [
        (11usize, 13usize),
        (21, 13),
        (41, 13),
        (11, 25),
        (21, 25),
        (41, 25),
        (21, 7),
        (41, 7),
        (41, 9),
        (33, 13),
    ] {
        // Non-divisor sizes are rebuilt exactly rather than subsampled.
        let exact_subset = (NR - 1) % (nr - 1) == 0 && (NU - 1) % (nu - 1) == 0;
        let mut v = vec![0.0f64; nr * nr * nu];
        if exact_subset {
            let sr = (NR - 1) / (nr - 1);
            let su = (NU - 1) / (nu - 1);
            for i in 0..nr {
                for j in 0..nr {
                    for k in 0..nu {
                        v[(i * nr + j) * nu + k] = fine[((i * sr) * NR + j * sr) * NU + k * su];
                    }
                }
            }
        } else {
            let rr: Vec<f64> = (0..nr).map(|i| r_of_tau(a, i as f64 / (nr - 1) as f64)).collect();
            let vv: Vec<f64> = rr.iter().map(|&r| trimer::pair_energy(r)).collect();
            for i in 0..nr {
                for j in i..nr {
                    let (x, y) = (rr[i], rr[j]);
                    for k in 0..nu {
                        let cc = C_LO + (C_HI - C_LO) * k as f64 / (nu - 1) as f64;
                        let u = 1.0 - cc * cc;
                        let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();
                        let sn = (1.0 - u * u).max(0.0).sqrt();
                        let e3 = trimer::hydrogen_energy(&[[0.0,0.0,0.0],[x,0.0,0.0],[y*u, y*sn, 0.0]]);
                        let d = e3 + 3.0 * e_h - (vv[i] + vv[j] + trimer::pair_energy(z));
                        v[(i * nr + j) * nu + k] = d;
                        v[(j * nr + i) * nu + k] = d;
                    }
                }
            }
        }
        let mut worst = 0.0f64;
        let mut at = (0.0, 0.0, 0.0);
        let mut sum2 = 0.0;
        for &(x, y, u) in &pts {
            let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();
            let cc = (1.0 - u).sqrt();
            let exact = trimer::de3_sides(x, y, z, e_h);
            let e = (eval(&v, nr, nu, a, x, y, cc) - exact).abs();
            sum2 += e * e;
            if e > worst {
                worst = e;
                at = (x, y, z);
            }
        }
        println!(
            "  {nr:>2}x{nr:>2}x{nu:>2}  unique nodes {:>6}  max |err| = {worst:.3e} Ha  rms = {:.3e}  worst at ({:.2},{:.2},{:.2})",
            nr * (nr + 1) / 2 * nu,
            (sum2 / 256.0).sqrt(),
            at.0, at.1, at.2
        );
    }
}
