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

use holon_chem::lanes::{solve_lanes, LaneSigma, LaneSolution};
use holon_chem::sigma_op::SigmaOp;
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
    /// `Σ_c ⟨n_{k,c}⟩`, one entry per site — of the CURRENT state, which is the ground
    /// state until a grab and an evolution move it.
    occ: Vec<f64>,
    // ---- the dynamics. The state is complex; the Hamiltonian is real symmetric, so the
    // two real parts are all that is stored and `exp(-iHt)` is built from `cos(Ht)` and
    // `sin(Ht)` on each (see `propagate`).
    re: Vec<f64>,
    im: Vec<f64>,
    /// Elapsed time since the grab, in the model's own units (ħ = 1).
    t: f64,
    /// The user's grab: a one-body potential per site, added to the Hamiltonian. All zero
    /// until someone reaches in and pulls.
    pin: Vec<f64>,
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
        re: sol.vector.clone(),
        im: vec![0.0; sol.vector.len()],
        t: 0.0,
        pin: vec![0.0; n],
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


// ------------------------------------------------------------------ the grab, and the dynamics

/// The sector's Hamiltonian with the user's grab added: a one-body potential on each site,
/// the same term for every colour, which is what a colour-blind poke IS. Zero pin gives back
/// exactly the campaign's Hamiltonian, bit for bit.
fn pinned_hamiltonian(q: &Qcd2, pin: &[f64]) -> holon_chem::lanes::LaneHamiltonian<f64> {
    let mut ham = q.lane_hamiltonian();
    for (k, &v) in pin.iter().enumerate() {
        if v != 0.0 {
            for c in 0..3 {
                ham.one_body(c, k, k, v);
            }
        }
    }
    ham
}

/// Eigen-decomposition of a small symmetric matrix by cyclic Jacobi rotations. Used on the
/// Krylov tridiagonal only, where `m ≤ KRYLOV`, so its cubic cost is nothing.
fn jacobi_sym(a: &mut [f64], v: &mut [f64], m: usize) {
    for i in 0..m {
        for j in 0..m {
            v[i * m + j] = f64::from(i == j);
        }
    }
    for _ in 0..60 {
        let off: f64 = (0..m).flat_map(|i| (i + 1..m).map(move |j| (i, j))).map(|(i, j)| a[i * m + j] * a[i * m + j]).sum();
        if off <= 1e-30 {
            break;
        }
        for p in 0..m {
            for qq in p + 1..m {
                let apq = a[p * m + qq];
                if apq.abs() <= 1e-300 {
                    continue;
                }
                let theta = (a[qq * m + qq] - a[p * m + p]) / (2.0 * apq);
                let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                for k in 0..m {
                    let (akp, akq) = (a[k * m + p], a[k * m + qq]);
                    a[k * m + p] = c * akp - s * akq;
                    a[k * m + qq] = s * akp + c * akq;
                }
                for k in 0..m {
                    let (apk, aqk) = (a[p * m + k], a[qq * m + k]);
                    a[p * m + k] = c * apk - s * aqk;
                    a[qq * m + k] = s * apk + c * aqk;
                }
                for k in 0..m {
                    let (vkp, vkq) = (v[k * m + p], v[k * m + qq]);
                    v[k * m + p] = c * vkp - s * vkq;
                    v[k * m + qq] = s * vkp + c * vkq;
                }
            }
        }
    }
}

/// Krylov dimension of one time step. Small on purpose: the step is what the page calls per
/// frame, and accuracy comes from taking more steps, not from a deeper space.
const KRYLOV: usize = 12;

/// `cos(H·dt)·v` and `sin(H·dt)·v` for a real vector, by Lanczos: build the Krylov basis of
/// `v` under `H`, diagonalise the tridiagonal, and apply the scalar functions to its
/// eigenvalues. The Hamiltonian is real symmetric, which is the whole reason the complex
/// propagator needs no complex arithmetic.
fn cos_sin_apply(sig: &mut LaneSigma<f64>, v: &[f64], dt: f64) -> (Vec<f64>, Vec<f64>) {
    let n = v.len();
    let nrm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if nrm == 0.0 {
        return (vec![0.0; n], vec![0.0; n]);
    }
    let mut basis: Vec<Vec<f64>> = vec![v.iter().map(|x| x / nrm).collect()];
    let (mut alpha, mut beta) = (Vec::new(), Vec::new());
    let mut w = vec![0.0; n];
    for j in 0..KRYLOV {
        sig.apply(&basis[j], &mut w);
        let a = basis[j].iter().zip(&w).map(|(x, y)| x * y).sum::<f64>();
        alpha.push(a);
        for (k, wk) in w.iter_mut().enumerate() {
            *wk -= a * basis[j][k];
            if j > 0 {
                *wk -= beta[j - 1] * basis[j - 1][k];
            }
        }
        // full re-orthogonalisation: the space is tiny and a lost orthogonality here shows
        // up as a norm drift on screen, which is the one thing this must not do
        for b in basis.iter() {
            let d = b.iter().zip(w.iter()).map(|(x, y)| x * y).sum::<f64>();
            for (k, wk) in w.iter_mut().enumerate() {
                *wk -= d * b[k];
            }
        }
        let bnorm = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        if bnorm <= 1e-13 || j + 1 == KRYLOV {
            break;
        }
        beta.push(bnorm);
        basis.push(w.iter().map(|x| x / bnorm).collect());
    }
    let m = alpha.len();
    let mut t = vec![0.0; m * m];
    for i in 0..m {
        t[i * m + i] = alpha[i];
        if i + 1 < m {
            t[i * m + i + 1] = beta[i];
            t[(i + 1) * m + i] = beta[i];
        }
    }
    let mut vecs = vec![0.0; m * m];
    let mut work = t.clone();
    jacobi_sym(&mut work, &mut vecs, m);
    let lambda: Vec<f64> = (0..m).map(|i| work[i * m + i]).collect();
    // e_0 in the Krylov basis, rotated, scaled by cos/sin, rotated back
    let (mut cc, mut ss) = (vec![0.0; m], vec![0.0; m]);
    for i in 0..m {
        let z = vecs[i]; // vecs[0*m + i] — the first row
        cc[i] = z * (lambda[i] * dt).cos();
        ss[i] = z * (lambda[i] * dt).sin();
    }
    let (mut cy, mut sy) = (vec![0.0; m], vec![0.0; m]);
    for i in 0..m {
        for j in 0..m {
            cy[i] += vecs[i * m + j] * cc[j];
            sy[i] += vecs[i * m + j] * ss[j];
        }
    }
    let (mut co, mut si) = (vec![0.0; n], vec![0.0; n]);
    for (j, b) in basis.iter().enumerate().take(m) {
        for k in 0..n {
            co[k] += nrm * cy[j] * b[k];
            si[k] += nrm * sy[j] * b[k];
        }
    }
    (co, si)
}

/// GRAB a site: add `strength` to the one-body potential there, for every colour. This does
/// NOT re-solve — the state stays where it was and the Hamiltonian changes under it, which
/// is a quantum quench and is exactly what makes the next step interesting. `0` accepted,
/// `4` no band solved, `5` no such site.
#[no_mangle]
pub extern "C" fn holon_hadron_grab(b: i32, k: u32, strength: f64) -> u32 {
    if !(0..=2).contains(&b) {
        return 5;
    }
    let mut g = BANDS.lock().expect("hadron band");
    let Some(band) = g[b as usize].as_mut() else { return 4 };
    let Some(slot) = band.pin.get_mut(k as usize) else { return 5 };
    *slot += strength;
    0
}

/// Let go: clear the grab and put the state back in the ground state of the unpinned
/// Hamiltonian. `0` done, `4` no band solved.
#[no_mangle]
pub extern "C" fn holon_hadron_release(b: i32) -> u32 {
    let (n, x) = {
        let g = BANDS.lock().expect("hadron band");
        match g.get(b.clamp(0, 2) as usize).and_then(|s| s.as_ref()) {
            Some(band) if (0..=2).contains(&b) => (band.n, band.x),
            _ => return 4,
        }
    };
    holon_hadron_solve(n as u32, x, b)
}

/// The grab's current strength at a site, `NaN` off the end.
#[no_mangle]
pub extern "C" fn holon_hadron_pin_at(b: i32, k: u32) -> f64 {
    band(b, |x| x.pin.get(k as usize).copied().unwrap_or(f64::NAN), f64::NAN)
}

/// Advance the state by `dt` under the CURRENT (possibly grabbed) Hamiltonian and refresh
/// the density. `0` done, `4` no band solved, `5` bad arguments.
///
/// The propagator is `exp(-iH·dt)` by Lanczos on the real Hamiltonian — unitary by
/// construction, and the page can see that for itself in `holon_hadron_norm`, which must
/// stay at one. A drifting norm on screen is a defect, not a rounding story.
#[no_mangle]
pub extern "C" fn holon_hadron_step(b: i32, dt: f64) -> u32 {
    if !(0..=2).contains(&b) || !dt.is_finite() {
        return 5;
    }
    let (n, x, pin, re, im) = {
        let g = BANDS.lock().expect("hadron band");
        let Some(band) = g[b as usize].as_ref() else { return 4 };
        (band.n, band.x, band.pin.clone(), band.re.clone(), band.im.clone())
    };
    let q = Qcd2::new(n, x);
    let space = q.lane_space(b);
    let mut sig = LaneSigma::new(&space, &pinned_hamiltonian(&q, &pin), 1);
    // exp(-iH dt)(re + i·im) = [cos·re + sin·im] + i[cos·im − sin·re]
    let (cre, sre) = cos_sin_apply(&mut sig, &re, dt);
    let (cim, sim) = cos_sin_apply(&mut sig, &im, dt);
    let new_re: Vec<f64> = cre.iter().zip(&sim).map(|(c, s)| c + s).collect();
    let new_im: Vec<f64> = cim.iter().zip(&sre).map(|(c, s)| c - s).collect();
    let occ = occupancy_complex(&space, &new_re, &new_im, n);
    let mut g = BANDS.lock().expect("hadron band");
    if let Some(band) = g[b as usize].as_mut() {
        band.re = new_re;
        band.im = new_im;
        band.occ = occ;
        band.t += dt;
    }
    0
}

/// `Σ_c ⟨n_{k,c}⟩` for a complex state.
fn occupancy_complex(space: &holon_chem::lanes::LaneSpace, re: &[f64], im: &[f64], n_sites: usize) -> Vec<f64> {
    let mut occ = vec![0.0; n_sites];
    for d in 0..re.len() {
        let w = re[d] * re[d] + im[d] * im[d];
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

/// Elapsed model time since the state was last put in a ground state.
#[no_mangle]
pub extern "C" fn holon_hadron_time(b: i32) -> f64 {
    band(b, |x| x.t, f64::NAN)
}

/// `⟨ψ|ψ⟩` of the evolving state — ONE for a unitary propagator, and a defect on screen
/// otherwise. This is the band's own honesty readout.
#[no_mangle]
pub extern "C" fn holon_hadron_norm(b: i32) -> f64 {
    band(b, |x| x.re.iter().map(|v| v * v).sum::<f64>() + x.im.iter().map(|v| v * v).sum::<f64>(), f64::NAN)
}

/// `⟨ψ|H|ψ⟩` under the CURRENT Hamiltonian — conserved by the evolution, so a drift here is
/// the propagator failing rather than physics happening.
#[no_mangle]
pub extern "C" fn holon_hadron_live_energy(b: i32) -> f64 {
    if !(0..=2).contains(&b) {
        return f64::NAN;
    }
    let (n, x, pin, re, im) = {
        let g = BANDS.lock().expect("hadron band");
        let Some(band) = g[b as usize].as_ref() else { return f64::NAN };
        (band.n, band.x, band.pin.clone(), band.re.clone(), band.im.clone())
    };
    let q = Qcd2::new(n, x);
    let space = q.lane_space(b);
    let mut sig = LaneSigma::new(&space, &pinned_hamiltonian(&q, &pin), 1);
    let mut w = vec![0.0; re.len()];
    sig.apply(&re, &mut w);
    let mut e = re.iter().zip(&w).map(|(a, c)| a * c).sum::<f64>();
    sig.apply(&im, &mut w);
    e += im.iter().zip(&w).map(|(a, c)| a * c).sum::<f64>();
    e
}
