//! THE SUB-ATOM BAND — the fold below the atom, solved EXACTLY in the page.
//!
//! The zoom ladder's floor was a fence: nothing drew below the nucleus, and its stated exit
//! was node GF2's three-dimensional hadron box. That exit is closed (2026-09-04) — the
//! physics it was reaching for is prior art, done on cluster hardware by
//! Silvi–Sauer–Tschirsich–Montangero (Phys. Rev. D 100, 074512, 2019) and by
//! Hayata–Hidaka–Nishimura (arXiv:2311.11643), both with tensor networks. So the band goes
//! live on what this engine can honestly compute instead: the EXACT colour-singlet ground
//! state of one-flavour QCD in 1+1 dimensions, on the colour-lane determinant solver, in the
//! browser, with no approximation of any kind.
//!
//! WHAT IS DRAWN. The quark density along the chain, `Σ_colours ⟨n_{k,c}⟩` per site, from the
//! exact ground-state vector. The difference between the `B = 0` sea and the `B = 1` state is
//! one baryon's worth of quarks, and where it sits is the picture this band exists to show.
//!
//! WHAT THIS IS NOT, stated in the band and not only here: one space dimension has no
//! transverse gluons, the gauge field is eliminated by Gauss's law rather than dynamical, and
//! there is no glueball sector and no asymptotic freedom in the four-dimensional sense. This
//! is a MODEL that shares confinement and colour-singlet structure with the real thing. It is
//! not the proton.
//!
//! COST, refused by name rather than attempted: the sector dimension is `C(N, n_q/3)³` and
//! grows fast — 3,375 at `N = 6, B = 1`, 175,616 at `N = 8, B = 1`, 9.3 million at `N = 10`.
//! The door refuses above [`MAX_DET`] so a zoom gesture can never hang the page.

use std::sync::Mutex;

use holon_chem::lanes::{solve_lanes, LaneSolution};
use holon_chem::qcd2::Qcd2;
use holon_chem::fci::SolveExit;

/// The largest sector this door will solve in a page. `N = 8, B = 1` is 175,616 determinants
/// and runs in about a second on the lane solver; `N = 10` is 9.3 million and would not.
pub const MAX_DET: usize = 400_000;

struct Band {
    n: usize,
    x: f64,
    b: i32,
    energy: f64,
    n_det: usize,
    residual: f64,
    exit: u32,
    margin: f64,
    seconds: f64,
    /// `Σ_c ⟨n_{k,c}⟩`, one entry per site.
    occ: Vec<f64>,
}

static BANDS: Mutex<[Option<Band>; 3]> = Mutex::new([None, None, None]);

fn exit_code(e: SolveExit) -> u32 {
    match e {
        SolveExit::Converged => 0,
        SolveExit::IterationCap => 1,
        SolveExit::Stagnated => 2,
        SolveExit::Trivial => 3,
    }
}

/// `Σ_c ⟨n_{k,c}⟩` per site from an exact lane vector: the determinant index factorises into
/// one string per colour, and each string's mask says which sites that colour occupies.
fn occupancy(space: &holon_chem::lanes::LaneSpace, sol: &LaneSolution, n_sites: usize) -> Vec<f64> {
    let mut occ = vec![0.0; n_sites];
    for (d, &c) in sol.vector.iter().enumerate() {
        let w = c * c;
        if w == 0.0 {
            continue;
        }
        for (l, lane) in space.lanes.iter().enumerate() {
            let idx = (d / space.strides[l]) % lane.masks.len();
            let mask = lane.masks[idx];
            for (k, o) in occ.iter_mut().enumerate().take(n_sites) {
                if mask >> k & 1 == 1 {
                    *o += w;
                }
            }
        }
    }
    occ
}

/// Solve the exact colour-singlet sector of 1+1D QCD on `n` sites at coupling `x`, baryon
/// number `b`, and keep it for the getters.
///
/// Returns `0` converged, `1` iteration cap, `2` stagnated, `3` trivial, `5` bad parameters
/// (odd `n`, `n < 2`, `b` outside 0..2, non-positive `x`), `6` the sector exceeds [`MAX_DET`].
/// A refusal leaves the previous solve in place and changes nothing on screen.
#[no_mangle]
pub extern "C" fn holon_hadron_solve(n: u32, x: f64, b: i32) -> u32 {
    let n = n as usize;
    if n < 2 || n % 2 != 0 || !(0..=2).contains(&b) || !(x > 0.0) {
        return 5;
    }
    let q = Qcd2::new(n, x);
    let n_q = q.quarks(b);
    if n_q % 3 != 0 || n_q / 3 > n {
        return 5;
    }
    let space = q.lane_space(b);
    if space.n_det > MAX_DET {
        return 6;
    }
    let t0 = crate::hadron_clock();
    let sol = solve_lanes(&space, &q.lane_hamiltonian(), 1);
    let seconds = crate::hadron_elapsed(t0);
    let occ = occupancy(&space, &sol, n);
    let code = exit_code(sol.exit);
    BANDS.lock().expect("hadron band")[b as usize] = Some(Band {
        n,
        x,
        b,
        energy: sol.energy,
        n_det: space.n_det,
        residual: sol.residual,
        exit: code,
        margin: sol.variational_margin,
        seconds,
        occ,
    });
    code
}

fn band<T>(b: i32, f: impl Fn(&Band) -> T, default: T) -> T {
    if !(0..=2).contains(&b) {
        return default;
    }
    BANDS.lock().expect("hadron band")[b as usize].as_ref().map_or(default, f)
}

/// The number of determinants a sector WOULD have, without solving it — what the page reads
/// before it offers a zoom, so a refusal is shown as a price rather than as a hang.
#[no_mangle]
pub extern "C" fn holon_hadron_dim_for(n: u32, b: i32) -> f64 {
    let n = n as usize;
    if n < 2 || n % 2 != 0 || !(0..=2).contains(&b) {
        return -1.0;
    }
    let q = Qcd2::new(n, 1.0);
    let n_q = q.quarks(b);
    if n_q % 3 != 0 || n_q / 3 > n {
        return -1.0;
    }
    let k = n_q / 3;
    let c = (0..k).fold(1.0f64, |acc, i| acc * (n - i) as f64 / (i + 1) as f64);
    c * c * c
}

#[no_mangle]
pub extern "C" fn holon_hadron_max_det() -> f64 {
    MAX_DET as f64
}

#[no_mangle]
pub extern "C" fn holon_hadron_energy(b: i32) -> f64 {
    band(b, |x| x.energy, f64::NAN)
}

#[no_mangle]
pub extern "C" fn holon_hadron_n_det(b: i32) -> f64 {
    band(b, |x| x.n_det as f64, -1.0)
}

#[no_mangle]
pub extern "C" fn holon_hadron_residual(b: i32) -> f64 {
    band(b, |x| x.residual, f64::NAN)
}

/// `0` converged, `1` iteration cap, `2` stagnated, `3` trivial, `4` not solved.
#[no_mangle]
pub extern "C" fn holon_hadron_exit(b: i32) -> u32 {
    band(b, |x| x.exit, 4)
}

/// `min diag − E`: non-negative for a true ground state. Negative is a defect on screen.
#[no_mangle]
pub extern "C" fn holon_hadron_margin(b: i32) -> f64 {
    band(b, |x| x.margin, f64::NAN)
}

#[no_mangle]
pub extern "C" fn holon_hadron_seconds(b: i32) -> f64 {
    band(b, |x| x.seconds, f64::NAN)
}

#[no_mangle]
pub extern "C" fn holon_hadron_sites(b: i32) -> u32 {
    band(b, |x| x.n as u32, 0)
}

/// `Σ_c ⟨n_{k,c}⟩` at site `k` — the quark density this band draws. `NaN` off the end.
#[no_mangle]
pub extern "C" fn holon_hadron_occ(b: i32, k: u32) -> f64 {
    band(b, |x| x.occ.get(k as usize).copied().unwrap_or(f64::NAN), f64::NAN)
}

/// The baryon mass in the Hamer–Kogut units the campaign uses: `(E₁ − E₀) / (2√x)`, from the
/// two stored sectors. `NaN` unless both are solved at the SAME `n` and `x` — a mass built
/// from two different boxes is not a mass, and this returns nothing rather than a number.
#[no_mangle]
pub extern "C" fn holon_hadron_baryon_mass() -> f64 {
    let g = BANDS.lock().expect("hadron band");
    match (g[0].as_ref(), g[1].as_ref()) {
        (Some(a), Some(c)) if a.n == c.n && a.x == c.x && a.b == 0 && c.b == 1 => {
            (c.energy - a.energy) / (2.0 * a.x.sqrt())
        }
        _ => f64::NAN,
    }
}
