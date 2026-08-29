//! From two nuclear charges to a potential energy curve: basis assembly, orbitals,
//! the FCI solve, and the table the sandbox consumes.
//!
//! # What a "pair" is here
//!
//! Two species, a separation, and nothing else. Everything the renderer needs about a
//! pair of atoms — whether they bind, where, how deeply, and what force they exert at
//! every separation — is a consequence of the two nuclear charges and the declared basis,
//! computed here. In particular WHICH PAIRS BIND is not a table lookup and not a flag:
//! He2 and Ne2 come out repulsive because in this model they are, and the code path that
//! produces their curves is the same one that produces N2's.
//!
//! # The unbound pairs are not a special case
//!
//! ELEMENTS-1's gate E3 asks that unbound pairs emit repulsive-only tables. They do, and
//! not because anything checks for them: [`locate_well`] looks for a minimum,
//! reports `None` when there is not one deeper than the stated threshold, and the table
//! carries that `None`. The sandbox then shows helium bouncing off everything, which is
//! the in-model truth rather than a rule anybody wrote down.
//!
//! # Cost
//!
//! Curve generation is lazy and cached by design: a pair's curve is computed the first
//! time that pair is encountered and kept. That matters because the cost is not uniform
//! — He2 is one determinant and N2 is 14 400 — and a sandbox that paid N2's price to
//! show two hydrogens would be paying for chemistry nobody asked for.

use crate::dual::D2;
use crate::elements::{sz2_sector, Species};
use crate::fci::{
    cholesky_orthonormaliser, jacobi_eigh, solve, transform, FciSpace, MoIntegrals, Solution,
};
use crate::md::{ao_integrals, AoIntegrals, Basis};


/// Wall time since a marker, milliseconds — and zero in the browser.
///
/// `std::time::Instant` exists on `wasm32-unknown-unknown` but `now()` PANICS there:
/// there is no clock behind it. This crate is linked into `holon-render`'s browser
/// artifact, so a timing call that only fails when someone actually generates a curve in
/// the browser is exactly the kind of latent panic the crate's zero-dependency discipline
/// exists to avoid. The generation time is a diagnostic, so the honest browser answer is
/// "not measured" rather than a guess or a crash; the host has `performance.now()` and
/// can time the call itself.
#[cfg(not(target_arch = "wasm32"))]
struct Stopwatch(std::time::Instant);
#[cfg(not(target_arch = "wasm32"))]
impl Stopwatch {
    fn start() -> Self {
        Stopwatch(std::time::Instant::now())
    }
    fn ms(&self) -> f64 {
        self.0.elapsed().as_secs_f64() * 1e3
    }
}

#[cfg(target_arch = "wasm32")]
struct Stopwatch;
#[cfg(target_arch = "wasm32")]
impl Stopwatch {
    fn start() -> Self {
        Stopwatch
    }
    /// Zero, meaning NOT MEASURED.
    fn ms(&self) -> f64 {
        0.0
    }
}

/// Assemble the STO-3G basis for a list of species at given centres.
pub fn build_basis(species: &[Species], centers: Vec<[D2; 3]>) -> Basis {
    assert_eq!(species.len(), centers.len());
    let mut decls = Vec::new();
    for (c, sp) in species.iter().enumerate() {
        for sh in sp.shells {
            decls.push((c, sh.kind.l(), sh.alpha, sh.coeff));
        }
    }
    let charges = species.iter().map(|s| s.z as f64).collect();
    Basis::assemble(centers, charges, &decls)
}

/// Electron partition of a system: total, alpha, beta.
pub fn electron_counts(species: &[Species]) -> (usize, usize, usize) {
    let n: u32 = species.iter().map(|s| s.n_electrons()).sum();
    let sz2 = sz2_sector(n);
    let na = ((n + sz2) / 2) as usize;
    let nb = ((n - sz2) / 2) as usize;
    (n as usize, na, nb)
}

/// One-electron and two-electron integrals in the `X` basis, values only, for the SCF
/// that chooses the orbital rotation. Derivatives are deliberately absent: the rotation
/// is treated as constant, and computing derivatives of a quantity that is then declared
/// constant would be work whose only effect is to be discarded.
fn transform_values(ao: &AoIntegrals, c: &[f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    let n2 = n * n;
    let mut h = vec![0.0f64; n2];
    for p in 0..n {
        for q in 0..n {
            let mut acc = 0.0;
            for i in 0..n {
                for j in 0..n {
                    acc += c[i * n + p] * c[j * n + q] * ao.h(i, j).v;
                }
            }
            h[p * n + q] = acc;
        }
    }
    let mut t1 = vec![0.0f64; n2 * n2];
    let mut t2 = vec![0.0f64; n2 * n2];
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                for s in 0..n {
                    let mut acc = 0.0;
                    for l in 0..n {
                        acc += ao.g(i, j, k, l).v * c[l * n + s];
                    }
                    t1[((i * n + j) * n + k) * n + s] = acc;
                }
            }
        }
    }
    for i in 0..n {
        for j in 0..n {
            for r in 0..n {
                for s in 0..n {
                    let mut acc = 0.0;
                    for k in 0..n {
                        acc += t1[((i * n + j) * n + k) * n + s] * c[k * n + r];
                    }
                    t2[((i * n + j) * n + r) * n + s] = acc;
                }
            }
        }
    }
    t1.iter_mut().for_each(|x| *x = 0.0);
    for i in 0..n {
        for q in 0..n {
            for r in 0..n {
                for s in 0..n {
                    let mut acc = 0.0;
                    for j in 0..n {
                        acc += t2[((i * n + j) * n + r) * n + s] * c[j * n + q];
                    }
                    t1[((i * n + q) * n + r) * n + s] = acc;
                }
            }
        }
    }
    let mut g = vec![0.0f64; n2 * n2];
    for p in 0..n {
        for q in 0..n {
            for r in 0..n {
                for s in 0..n {
                    let mut acc = 0.0;
                    for i in 0..n {
                        acc += t1[((i * n + q) * n + r) * n + s] * c[i * n + p];
                    }
                    g[((p * n + q) * n + r) * n + s] = acc;
                }
            }
        }
    }
    (h, g)
}

/// A constant orbital rotation, chosen to make the CI problem well conditioned.
///
/// # Why this is allowed to be crude, and why it is allowed to fail
///
/// Full CI is invariant under it (see this module's `fci` neighbour), so the rotation
/// changes the energy, its slope and its curvature by nothing at all. What it changes is
/// how concentrated the ground state is: with self-consistent-field orbitals one
/// determinant carries most of the weight and Davidson converges in tens of iterations;
/// with the raw Gram–Schmidt orbitals the state is a broad valence-bond expansion and it
/// takes hundreds. So a converged SCF is a speed-up and a diverged one is only a
/// slow-down — which is why the fallback here is silent about quality and loud only
/// about having happened.
///
/// Returns `(rotation, scf_iterations, converged)`.
fn orbital_rotation(
    h: &[f64],
    g: &[f64],
    n: usize,
    n_alpha: usize,
    n_beta: usize,
) -> (Vec<f64>, usize, bool) {
    let n2 = n * n;
    let (_, mut c) = jacobi_eigh(h, n);
    // Occupation numbers: doubly occupied up to `n_beta`, singly to `n_alpha`. An
    // average-of-configuration Fock rather than a spin-resolved one, because the only
    // thing being asked of it is a well-conditioned basis.
    let mut occ = vec![0.0f64; n];
    for (p, o) in occ.iter_mut().enumerate() {
        *o = if p < n_beta {
            2.0
        } else if p < n_alpha {
            1.0
        } else {
            0.0
        };
    }
    let mut last = f64::INFINITY;
    let mut converged = false;
    let mut iters = 0usize;
    let mut d = vec![0.0f64; n2];
    let mut f = vec![0.0f64; n2];
    for it in 0..200 {
        iters = it + 1;
        let mut dnew = vec![0.0f64; n2];
        for p in 0..n {
            if occ[p] == 0.0 {
                continue;
            }
            for i in 0..n {
                for j in 0..n {
                    dnew[i * n + j] += occ[p] * c[i * n + p] * c[j * n + p];
                }
            }
        }
        // Damping. Not tuning: a first-row atom's average-occupation SCF oscillates
        // between two densities without it, and the oscillation never resolves.
        if it == 0 {
            d = dnew;
        } else {
            for i in 0..n2 {
                d[i] = 0.5 * d[i] + 0.5 * dnew[i];
            }
        }
        for p in 0..n {
            for q in 0..n {
                let mut acc = h[p * n + q];
                for r in 0..n {
                    for s in 0..n {
                        acc += d[r * n + s]
                            * (g[(p * n + q) * n2 + (r * n + s)]
                                - 0.5 * g[(p * n + r) * n2 + (s * n + q)]);
                    }
                }
                f[p * n + q] = acc;
            }
        }
        let mut e = 0.0;
        for i in 0..n2 {
            e += 0.5 * d[i] * (h[i] + f[i]);
        }
        if !e.is_finite() {
            // The rotation is a convenience; a diverged one is discarded rather than
            // used, and the caller is told.
            let (_, c0) = jacobi_eigh(h, n);
            return (c0, iters, false);
        }
        let (_, cnew) = jacobi_eigh(&f, n);
        c = cnew;
        if (e - last).abs() < 1e-10 {
            converged = true;
            break;
        }
        last = e;
    }
    (c, iters, converged)
}

/// Everything one geometry produced, including the diagnostics that say how hard it was
/// and how well it converged.
pub struct PointSolution {
    /// Total energy — electronic plus nuclear repulsion — with its exact first and
    /// second derivatives with respect to the separation.
    pub e: D2,
    pub n_det: usize,
    pub n_basis: usize,
    pub davidson_iters: usize,
    pub cg_iters: usize,
    pub residual: f64,
    pub cg_residual: f64,
    pub scf_converged: bool,
    /// Smallest eigenvalue of the AO overlap matrix. The conditioning of the Cholesky
    /// orthogonalisation is set by it, so it is reported rather than assumed.
    pub s_min_eigenvalue: f64,
}

/// Solve one geometry: assemble, orthonormalise, rotate, transform, diagonalise.
pub fn solve_geometry(species: &[Species], centers: Vec<[D2; 3]>) -> PointSolution {
    let (_, n_alpha, n_beta) = electron_counts(species);
    solve_basis(&build_basis(species, centers), n_alpha, n_beta)
}

/// The same, from an already-assembled basis and an explicit electron partition.
///
/// Separate from [`solve_geometry`] so the mutation plants can drive the whole pipeline
/// with ONE input changed — a nuclear charge, or one contraction coefficient — without
/// going through the species registry that would refuse to hold the mutated value. A
/// plant that has to bypass the code it is planted in is not testing that code.
pub fn solve_basis(basis: &Basis, n_alpha: usize, n_beta: usize) -> PointSolution {
    let n = basis.n;
    let ao = ao_integrals(basis);
    let x = cholesky_orthonormaliser(&ao.s, n)
        .expect("the overlap matrix is not positive definite: the basis is linearly dependent");

    let s_values: Vec<f64> = ao.s.iter().map(|d| d.v).collect();
    let (s_eigs, _) = jacobi_eigh(&s_values, n);

    let xv: Vec<f64> = x.iter().map(|d| d.v).collect();
    let (hx, gx) = transform_values(&ao, &xv, n);
    let (u, _scf_iters, scf_converged) = orbital_rotation(&hx, &gx, n, n_alpha, n_beta);

    // C = X U, with U carrying ZERO derivative. See the module header: the FCI energy is
    // invariant under U for every R, so its R-derivatives are too, and declaring U
    // constant is exact rather than approximate.
    let mut c = vec![D2::c(0.0); n * n];
    for i in 0..n {
        for p in 0..n {
            let mut acc = D2::c(0.0);
            for m in 0..n {
                acc = acc + x[i * n + m] * u[m * n + p];
            }
            c[i * n + p] = acc;
        }
    }

    let mo = transform(&ao, &c, n);
    let space = FciSpace::new(n, n_alpha, n_beta);
    let sol = solve(&space, &mo);
    let e = sol.e + basis.nuclear_repulsion();
    PointSolution {
        e,
        n_det: space.n_det,
        n_basis: n,
        davidson_iters: sol.davidson_iters,
        cg_iters: sol.cg_iters,
        residual: sol.residual,
        cg_residual: sol.cg_residual,
        scf_converged,
        s_min_eigenvalue: s_eigs[0],
    }
}

/// Solve one geometry and hand back the MO integrals and CI space as well, for the tests
/// that want to run a second, independent route on the same numbers.
pub fn solve_geometry_detailed(
    species: &[Species],
    centers: Vec<[D2; 3]>,
) -> (FciSpace, MoIntegrals, Solution, D2) {
    let basis = build_basis(species, centers);
    let n = basis.n;
    let ao = ao_integrals(&basis);
    let x = cholesky_orthonormaliser(&ao.s, n).expect("overlap not positive definite");
    let (_, n_alpha, n_beta) = electron_counts(species);
    let xv: Vec<f64> = x.iter().map(|d| d.v).collect();
    let (hx, gx) = transform_values(&ao, &xv, n);
    let (u, _, _) = orbital_rotation(&hx, &gx, n, n_alpha, n_beta);
    let mut c = vec![D2::c(0.0); n * n];
    for i in 0..n {
        for p in 0..n {
            let mut acc = D2::c(0.0);
            for m in 0..n {
                acc = acc + x[i * n + m] * u[m * n + p];
            }
            c[i * n + p] = acc;
        }
    }
    let mo = transform(&ao, &c, n);
    let space = FciSpace::new(n, n_alpha, n_beta);
    let sol = solve(&space, &mo);
    (space, mo, sol, basis.nuclear_repulsion())
}

/// The isolated atom's energy in this model, at the same level of theory as the curve.
///
/// COMPUTED, never quoted. It is the zero of every pair potential the sandbox draws, so
/// an error here would shift a whole ledger by a constant and nothing downstream would
/// notice.
pub fn atom_energy(sp: Species) -> f64 {
    solve_geometry(&[sp], vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)]]).e.v
}

/// One point of a pair curve.
#[derive(Clone, Copy, Debug)]
pub struct PairPoint {
    pub r: f64,
    /// Total energy, hartree.
    pub e: f64,
    /// The FORCE, `-dE/dR`, hartree/bohr. Positive means repulsive.
    pub f: f64,
    /// `d2E/dR2`, hartree/bohr^2.
    pub e2: f64,
}

/// Total energy and its two exact derivatives for a species pair at separation `r`.
pub fn pair_point(a: Species, b: Species, r: f64) -> PairPoint {
    let sol = solve_geometry(
        &[a, b],
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::var(r)],
        ],
    );
    PairPoint {
        r,
        e: sol.e.v,
        f: -sol.e.d,
        e2: sol.e.e,
    }
}

// ------------------------------------------------------------------ the pair table
//
// E3's contract: every species pair emits the SAME Hermite table the renderer already
// consumes — knots, energies, forces, curvatures, and the scalars that describe the whole
// curve — extended with the species metadata a multi-element sandbox needs, and with the
// well made OPTIONAL so an unbound pair can say so in the schema rather than in a comment.

/// The worst Davidson residual a generated curve may carry and still be called converged.
///
/// # Why a curve needs a verdict and not only a number
///
/// `PairMeta` has always RECORDED `worst_residual`, and nothing ever checked it. A curve
/// whose solve hit its iteration cap at some geometry would be emitted looking perfectly
/// healthy, carrying a wrong energy, with the evidence sitting in a field no consumer is
/// required to read. That is a dead result presenting as a live one — the same shape as an
/// exception that cannot cross a process boundary, and the sibling referee lane lost two
/// hours of a loaded machine to exactly that before asking whether this side had an
/// equivalent. It did, and this is it.
///
/// Set an order above `davidson`'s own 1e-11 target, so an ordinary solve clears it and a
/// solve that gave up does not.
pub const CONVERGED_RESIDUAL: f64 = 1e-10;

/// The declared threshold below which a dip in the curve is not called a well.
///
/// ELEMENTS-1's gate E1 stakes the negative controls at this depth ("no well deeper than
/// 1e-4 hartree"), so the code and the gate use ONE number and it is this one. It is a
/// stake, not a tolerance: a pair whose deepest minimum is shallower than this is
/// reported as unbound, and if a staked negative control ever comes back deeper the gate
/// fires rather than the constant moving.
pub const WELL_MIN_DEPTH: f64 = 1e-4;

/// How far above the dissociation asymptote the inner end of the grid sits, hartree.
///
/// DECLARED. The renderer needs the repulsive wall sampled well enough to turn an
/// incoming atom around, and one hartree is far above any kinetic energy the sandbox can
/// give a pair; going closer costs knots on a region nothing reaches, and going further
/// out lets a hard push tunnel through the end of the table.
pub const WALL_CEILING: f64 = 1.0;

/// How close to the asymptote the outer end of the grid sits, hartree. DECLARED, and
/// chosen four orders below [`WELL_MIN_DEPTH`] so the tail cannot hide a well.
pub const TAIL_TOLERANCE: f64 = 1e-8;

/// Outermost separation the range search will consider, bohr. DECLARED: beyond this the
/// in-model interaction of any first-row pair is below f64 noise on a total energy of
/// order 100 hartree, so a larger table would be sampling roundoff.
pub const R_SEARCH_MAX: f64 = 20.0;

/// The bound minimum of a pair curve, when there is one.
#[derive(Clone, Copy, Debug)]
pub struct Well {
    /// Equilibrium separation, bohr — the ROOT of `dE/dR`, not the smallest sampled knot.
    pub r_e: f64,
    /// Well depth, `E_asymptote - E(R_e)`, hartree.
    pub d_e: f64,
    pub e_at_r_e: f64,
    /// Harmonic force constant at the minimum, `d2E/dR2(R_e)`, hartree/bohr^2.
    pub k_e: f64,
}

/// Everything about a pair that is not one of its knots.
#[derive(Clone, Debug)]
pub struct PairMeta {
    pub symbol_a: &'static str,
    pub symbol_b: &'static str,
    pub z_a: u32,
    pub z_b: u32,
    /// Nuclear masses in ELECTRON masses — the unit the integrator works in.
    pub mass_a: f64,
    pub mass_b: f64,
    pub isotope_a: &'static str,
    pub isotope_b: &'static str,
    /// Reduced mass of the two ATOMS, electron masses. Born–Oppenheimer, so the
    /// electrons ride with the nuclei; `sim.rs` states the same convention.
    pub reduced_mass: f64,
    pub n_electrons: usize,
    /// Twice the `S_z` the pair was solved in.
    pub sz2: u32,
    pub n_basis: usize,
    pub n_det: usize,
    pub n_knots: usize,
    pub r_min: f64,
    pub r_max: f64,
    /// Two isolated atoms at the same level of theory. COMPUTED, and the zero of the
    /// renderer's pair potential.
    pub e_asymptote: f64,
    /// `None` for a pair with no minimum deeper than [`WELL_MIN_DEPTH`] — the schema's
    /// way of saying "repulsive only", which is the in-model truth for He2 and Ne2.
    pub well: Option<Well>,
    pub provenance: &'static str,
    /// Worst Davidson residual over the knots, and worst response-solve residual. A
    /// curve that did not converge everywhere must say so in its own metadata.
    pub worst_residual: f64,
    pub worst_cg_residual: f64,
    /// Smallest overlap eigenvalue seen, the conditioning of the orthogonalisation.
    pub worst_s_eigenvalue: f64,
    pub scf_converged_everywhere: bool,
    /// Wall time the curve cost to generate, milliseconds. ZERO means NOT MEASURED —
    /// the browser has no clock behind `std::time::Instant`, so the host times the call
    /// instead. See `Stopwatch` above.
    pub generation_ms: f64,
}

/// The label every generated pair curve carries. It states the model, the arithmetic and
/// the route, and deliberately does NOT say "exact", which would be a claim about the
/// world rather than about the basis.
pub const PAIR_PROVENANCE: &str = "engine-computed STO-3G FCI (determinant, Knowles-Handy), f64";

/// A generated pair curve: the renderer's four columns plus the metadata above.
#[derive(Clone, Debug)]
pub struct PairTable {
    pub r: Vec<f64>,
    pub e: Vec<f64>,
    pub f: Vec<f64>,
    pub e2: Vec<f64>,
    pub meta: PairMeta,
}

/// Bisect for the separation at which `f` changes sign. Returns `None` if the interval is
/// not a bracket.
///
/// The refusal is the point. Bisection on an interval whose endpoints share a sign does
/// not fail — it converges, to whichever endpoint the halving walks toward, and returns a
/// number that looks like a root. Every caller here feeds it a bracket it believes in for
/// a physical reason, and every one of those reasons could be wrong for a species nobody
/// has run yet, which is exactly when a plausible wrong number is worst.
pub fn bisect(lo: f64, hi: f64, steps: usize, f: &mut dyn FnMut(f64) -> f64) -> Option<f64> {
    let (mut lo, mut hi) = (lo, hi);
    let flo = f(lo);
    let fhi = f(hi);
    if !(flo.is_finite() && fhi.is_finite()) || flo * fhi > 0.0 {
        return None;
    }
    for _ in 0..steps {
        let mid = 0.5 * (lo + hi);
        if f(mid) * flo > 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// Derive the grid range for a pair from its own curve.
///
/// Not declared per species, because "a good range for N2" and "a good range for H2" are
/// different by a factor of three and a table of them would be a table of results. The
/// two DECLARED numbers are the energy the inner end sits at ([`WALL_CEILING`] above the
/// asymptote) and the energy the outer end sits within ([`TAIL_TOLERANCE`] of it); the
/// separations that realise them are computed.
pub fn derive_range(a: Species, b: Species, e_asymptote: f64) -> (f64, f64) {
    let u = |r: f64| solve_geometry(
        &[a, b],
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::c(r)],
        ],
    )
    .e
    .v
        - e_asymptote;

    // Inner end: walk in from a separation that is certainly below the ceiling until one
    // above it is bracketed, then bisect. Walking rather than assuming, because the wall
    // of He2 and the wall of N2 are nowhere near each other.
    let mut inner_hi = 2.0f64;
    while u(inner_hi) > WALL_CEILING && inner_hi < 6.0 {
        inner_hi += 0.5;
    }
    let mut inner_lo = 0.2f64;
    if u(inner_lo) <= WALL_CEILING {
        // Nothing in range reaches the ceiling: keep the whole bracket.
        return (inner_lo, R_SEARCH_MAX.min(12.0));
    }
    let Some(r_min) = bisect(inner_lo, inner_hi, 24, &mut |r| u(r) - WALL_CEILING) else {
        // The wall was never bracketed between 0.2 bohr and the outward walk's cap. That
        // is not a geometry this model describes, so the whole scanned interval is
        // returned rather than a root that is not one.
        return (inner_lo, R_SEARCH_MAX.min(12.0));
    };
    inner_lo = r_min;

    // Outer end: the first separation past the wall where the interaction has fallen
    // inside the tail tolerance. Searched from the outside in, so a curve that is still
    // moving at `R_SEARCH_MAX` returns that rather than a shorter table.
    let mut r_max = R_SEARCH_MAX;
    let mut probe = R_SEARCH_MAX;
    while probe > inner_lo * 2.0 {
        if u(probe).abs() >= TAIL_TOLERANCE {
            break;
        }
        r_max = probe;
        probe *= 0.8;
    }
    (r_min, r_max.min(R_SEARCH_MAX))
}

/// Generate the whole curve for a species pair, with its metadata.
///
/// `n_knots` is the caller's choice of grid density; the knots themselves are placed by
/// [`crate::table::grid_point`], uniform in `R^{-1/4}`, which is the spacing that
/// equidistributes the cubic Hermite interpolant's error (see `table.rs` for the
/// derivation).
pub fn generate_pair_table(a: Species, b: Species, n_knots: usize) -> PairTable {
    let t0 = Stopwatch::start();
    let e_asymptote = atom_energy(a) + atom_energy(b);
    let (r_min, r_max) = derive_range(a, b, e_asymptote);

    let mut r = Vec::with_capacity(n_knots);
    let mut e = Vec::with_capacity(n_knots);
    let mut f = Vec::with_capacity(n_knots);
    let mut e2 = Vec::with_capacity(n_knots);
    let (mut worst_residual, mut worst_cg, mut worst_s) = (0.0f64, 0.0f64, f64::INFINITY);
    let mut scf_ok = true;
    let (mut n_det, mut n_basis) = (0usize, 0usize);

    for i in 0..n_knots {
        let ri = crate::table::grid_point(r_min, r_max, n_knots, i);
        let sol = solve_geometry(
            &[a, b],
            vec![
                [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                [D2::c(0.0), D2::c(0.0), D2::var(ri)],
            ],
        );
        r.push(ri);
        e.push(sol.e.v);
        f.push(-sol.e.d);
        e2.push(sol.e.e);
        worst_residual = worst_residual.max(sol.residual);
        worst_cg = worst_cg.max(sol.cg_residual);
        worst_s = worst_s.min(sol.s_min_eigenvalue);
        scf_ok &= sol.scf_converged;
        n_det = sol.n_det;
        n_basis = sol.n_basis;
    }

    let well = locate_well(a, b, &r, &e, e_asymptote);
    let (n_electrons, _, _) = electron_counts(&[a, b]);
    let sz2 = sz2_sector(n_electrons as u32);
    let (ma, mb) = (a.mass_me(), b.mass_me());
    PairTable {
        r,
        e,
        f,
        e2,
        meta: PairMeta {
            symbol_a: a.symbol,
            symbol_b: b.symbol,
            z_a: a.z,
            z_b: b.z,
            mass_a: ma,
            mass_b: mb,
            isotope_a: a.isotope,
            isotope_b: b.isotope,
            reduced_mass: ma * mb / (ma + mb),
            n_electrons,
            sz2,
            n_basis,
            n_det,
            n_knots,
            r_min,
            r_max,
            e_asymptote,
            well,
            provenance: PAIR_PROVENANCE,
            worst_residual,
            worst_cg_residual: worst_cg,
            worst_s_eigenvalue: worst_s,
            scf_converged_everywhere: scf_ok,
            generation_ms: t0.ms(),
        },
    }
}

/// Find the bound minimum, or report that there is not one.
///
/// The knot grid locates a candidate; the answer is then the ROOT of `dE/dR`, refined by
/// bisection and finished by Newton with the exact second derivative in hand — the same
/// two-stage discipline `h2.rs` uses, and for the same reason: bisection cannot leave the
/// bracket, and Newton lands on the last bit.
///
/// Returns `None` unless the minimum is deeper than [`WELL_MIN_DEPTH`] below the
/// asymptote. That is the whole of the "unbound pair" logic: nothing checks which
/// elements are noble, and nothing has to.
pub fn locate_well(
    a: Species,
    b: Species,
    r: &[f64],
    e: &[f64],
    e_asymptote: f64,
) -> Option<Well> {
    let mut best = 0usize;
    for i in 1..e.len() {
        if e[i] < e[best] {
            best = i;
        }
    }
    // A minimum at either end of the grid is the end of the grid, not a well.
    if best == 0 || best + 1 >= e.len() {
        return None;
    }
    if e_asymptote - e[best] <= WELL_MIN_DEPTH {
        return None;
    }
    let slope = |x: f64| -pair_point(a, b, x).f;
    let (mut lo, mut hi) = (r[best - 1], r[best + 1]);
    if slope(lo) > 0.0 || slope(hi) < 0.0 {
        return None;
    }
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if slope(mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo < 1e-9 {
            break;
        }
    }
    let mut x = 0.5 * (lo + hi);
    for _ in 0..6 {
        let p = pair_point(a, b, x);
        if p.e2 == 0.0 {
            break;
        }
        let step = -p.f / p.e2;
        x -= step;
        if step.abs() <= f64::EPSILON * x.abs() {
            break;
        }
    }
    let p = pair_point(a, b, x);
    let d_e = e_asymptote - p.e;
    if d_e <= WELL_MIN_DEPTH {
        return None;
    }
    Some(Well {
        r_e: x,
        d_e,
        e_at_r_e: p.e,
        k_e: p.e2,
    })
}

impl PairMeta {
    /// Did every solve behind this curve converge?
    ///
    /// A VERDICT, not a number. See [`CONVERGED_RESIDUAL`]: the residual was recorded here
    /// long before anything asked it a question, which is how a curve that gave up could
    /// have shipped looking healthy.
    pub fn converged(&self) -> bool {
        self.worst_residual <= CONVERGED_RESIDUAL
            && self.worst_residual.is_finite()
            && self.worst_cg_residual.is_finite()
    }
}

impl PairTable {
    /// Serialise to the renderer's contract, extended with the species block.
    ///
    /// Numbers are written with `{:?}`, Rust's shortest round-tripping representation, so
    /// re-reading gives bit-identical knots and the file carries no digits the
    /// computation did not have.
    pub fn to_json(&self) -> String {
        let m = &self.meta;
        let mut s = String::with_capacity(64 * self.r.len() + 4096);
        s.push_str("{\n");
        s.push_str(&format!("  \"provenance\": \"{}\",\n", m.provenance));
        s.push_str(
            "  \"units\": \"Hartree atomic units: R in bohr, E in hartree, \
             F in hartree/bohr, masses in electron masses. F is the FORCE, so dE/dR = -F.\",\n",
        );
        s.push_str(&format!(
            "  \"model\": \"{}{}/STO-3G/FCI\",\n",
            m.symbol_a, m.symbol_b
        ));
        s.push_str(
            "  \"note\": \"EXACT-IN-MODEL for STO-3G full CI, computed from closed-form \
             Gaussian integrals in f64. Every scalar below is computed by the same code; \
             none is quoted. NOT a prediction of experiment.\",\n",
        );
        s.push_str("  \"species\": {\n");
        s.push_str(&format!(
            "    \"a\": {{\"symbol\": \"{}\", \"Z\": {}, \"mass_me\": {:?}, \"isotope\": \"{}\"}},\n",
            m.symbol_a, m.z_a, m.mass_a, m.isotope_a
        ));
        s.push_str(&format!(
            "    \"b\": {{\"symbol\": \"{}\", \"Z\": {}, \"mass_me\": {:?}, \"isotope\": \"{}\"}},\n",
            m.symbol_b, m.z_b, m.mass_b, m.isotope_b
        ));
        s.push_str(&format!("    \"reduced_mass_me\": {:?},\n", m.reduced_mass));
        s.push_str(&format!("    \"n_electrons\": {},\n", m.n_electrons));
        s.push_str(&format!("    \"ground_Sz\": {:?},\n", m.sz2 as f64 / 2.0));
        s.push_str(&format!("    \"n_basis\": {},\n", m.n_basis));
        s.push_str(&format!("    \"n_determinants\": {}\n", m.n_det));
        s.push_str("  },\n");
        s.push_str(&format!("  \"E_asymptote\": {:?},\n", m.e_asymptote));
        match m.well {
            Some(w) => {
                s.push_str("  \"bound\": true,\n");
                s.push_str(&format!("  \"R_e\": {:?},\n", w.r_e));
                s.push_str(&format!("  \"D_e\": {:?},\n", w.d_e));
                s.push_str(&format!("  \"E_at_R_e\": {:?},\n", w.e_at_r_e));
                s.push_str(&format!("  \"k_e\": {:?},\n", w.k_e));
            }
            None => {
                // The repulsive-only table. `R_e` and `D_e` are null rather than absent
                // so a consumer that reads the schema positionally still finds them, and
                // rather than zero so nobody can read a zero as a measurement.
                s.push_str("  \"bound\": false,\n");
                s.push_str("  \"R_e\": null,\n");
                s.push_str("  \"D_e\": null,\n");
                s.push_str("  \"E_at_R_e\": null,\n");
                s.push_str("  \"k_e\": null,\n");
            }
        }
        s.push_str(&format!("  \"converged\": {},\n", m.converged()));
        s.push_str(&format!(
            "  \"convergence\": {{\"worst_davidson_residual\": {:?}, \
             \"worst_response_residual\": {:?}, \"min_overlap_eigenvalue\": {:?}, \
             \"scf_converged_everywhere\": {}}},\n",
            m.worst_residual, m.worst_cg_residual, m.worst_s_eigenvalue, m.scf_converged_everywhere
        ));
        s.push_str(&format!("  \"generation_ms\": {:?},\n", m.generation_ms));
        push_array(&mut s, "R_grid_bohr", &self.r);
        push_array(&mut s, "E_hartree", &self.e);
        push_array(&mut s, "F_hartree_per_bohr", &self.f);
        push_array(&mut s, "E2_hartree_per_bohr2", &self.e2);
        s.push_str(&format!("  \"n_grid\": {}\n", self.r.len()));
        s.push_str("}\n");
        s
    }
}

fn push_array(s: &mut String, name: &str, v: &[f64]) {
    s.push_str("  \"");
    s.push_str(name);
    s.push_str("\": [");
    for (i, x) in v.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        if x.is_finite() {
            s.push_str(&format!("{x:?}"));
        } else {
            s.push_str("null");
        }
    }
    s.push_str("],\n");
}

// ------------------------------------------------------------------ the lazy cache

/// Pair curves, computed on first encounter and kept.
///
/// # Why lazy and not a pre-built table of everything
///
/// The first row has 55 unordered pairs and their costs differ by four orders of
/// magnitude: He2 is one determinant and N2 is 14 400. A sandbox that showed two
/// hydrogens would otherwise be paying for nitrogen chemistry nobody had asked for, and
/// in a browser that is the difference between a page that loads and one that does not.
/// So a pair is solved the first time it appears in a scene, and never again.
#[derive(Default)]
pub struct PairCache {
    entries: Vec<((u32, u32), PairTable)>,
    pub knots: usize,
    /// Cumulative generation time across every miss, milliseconds — so the sandbox can
    /// report what its chemistry actually cost rather than estimating it.
    pub total_ms: f64,
    pub misses: usize,
    pub hits: usize,
}

impl PairCache {
    pub fn new(knots: usize) -> PairCache {
        PairCache {
            entries: Vec::new(),
            knots,
            total_ms: 0.0,
            misses: 0,
            hits: 0,
        }
    }

    /// The curve for a pair, computing it if this is the first time it has been asked
    /// for. The key is ordered so `(H, F)` and `(F, H)` are one entry: the curve is a
    /// function of the unordered pair, and storing it twice would be two things to keep
    /// in step.
    pub fn get(&mut self, a: Species, b: Species) -> &PairTable {
        let key = if a.z <= b.z { (a.z, b.z) } else { (b.z, a.z) };
        if let Some(pos) = self.entries.iter().position(|(k, _)| *k == key) {
            self.hits += 1;
            return &self.entries[pos].1;
        }
        let (lo, hi) = if a.z <= b.z { (a, b) } else { (b, a) };
        let table = generate_pair_table(lo, hi, self.knots);
        self.misses += 1;
        self.total_ms += table.meta.generation_ms;
        self.entries.push((key, table));
        &self.entries.last().unwrap().1
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ------------------------------------------------------------------ the species palette
//
// What the sandbox needs to DRAW an element, as opposed to what it needs to integrate one.

/// The energy above the asymptote at which two atoms are called "in contact", hartree.
///
/// DECLARED, and it exists for one reason: a pair with no well has no `R_e`, so a size
/// rule built on `R_e` alone has nothing to say about helium or neon — the two species the
/// sandbox most needs to draw correctly, because their whole point is bouncing off things.
/// This is the fallback, and it is still a reading of the computed curve rather than a
/// number somebody chose for helium: the separation at which the repulsion has risen this
/// far above the asymptote. Set at ten times [`WELL_MIN_DEPTH`] so a curve too flat to
/// count as bound is still far enough up its own wall for the separation to be sharp.
pub const CONTACT_ENERGY: f64 = 1e-3;

/// How a species' drawn radius was arrived at.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadiusRule {
    /// DERIVED: half the equilibrium separation of the element's own homonuclear pair.
    HalfEquilibrium,
    /// DERIVED, with a declared threshold: half the separation at which the homonuclear
    /// pair's repulsion reaches [`CONTACT_ENERGY`] above the asymptote. Used for the
    /// elements whose homonuclear pair does not bind.
    HalfContact,
}

/// One element's drawing size, and how it was obtained.
#[derive(Clone, Copy, Debug)]
pub struct SpeciesSize {
    pub radius_bohr: f64,
    pub rule: RadiusRule,
    /// The homonuclear separation the radius is half of.
    pub separation_bohr: f64,
    /// The homonuclear well depth, or `None` if the pair does not bind.
    pub d_e: Option<f64>,
    pub n_det: usize,
    pub ms: f64,
}

/// The drawn radius of one element, computed from its own homonuclear curve.
///
/// # Why a coarse scan and not a full table
///
/// The radius needs ONE separation, not a curve: either the root of `dE/dR` or the
/// crossing of a declared energy. A full 64-knot table would compute both and 62 other
/// points to throw away, and for the middle of the first row those points cost seconds
/// each. So this walks a coarse grid to bracket the answer and then refines only the
/// bracket.
pub fn homonuclear_size(sp: Species) -> SpeciesSize {
    let t0 = Stopwatch::start();
    let asym = 2.0 * atom_energy(sp);
    let scan: Vec<f64> = (0..15).map(|i| 1.0 + 0.5 * i as f64).collect();
    let mut e = Vec::with_capacity(scan.len());
    let mut n_det = 0usize;
    for &r in scan.iter() {
        let sol = solve_geometry(
            &[sp, sp],
            vec![
                [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                [D2::c(0.0), D2::c(0.0), D2::c(r)],
            ],
        );
        e.push(sol.e.v);
        n_det = sol.n_det;
    }
    if let Some(w) = locate_well(sp, sp, &scan, &e, asym) {
        return SpeciesSize {
            radius_bohr: 0.5 * w.r_e,
            rule: RadiusRule::HalfEquilibrium,
            separation_bohr: w.r_e,
            d_e: Some(w.d_e),
            n_det,
            ms: t0.ms(),
        };
    }
    // No well: fall back to the contact separation. Bracket by walking in from the
    // outermost scanned point, so the search cannot start inside the wall.
    let u = |r: f64| {
        solve_geometry(
            &[sp, sp],
            vec![
                [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                [D2::c(0.0), D2::c(0.0), D2::c(r)],
            ],
        )
        .e
        .v
            - asym
    };
    let mut hi = *scan.last().unwrap();
    let mut lo = hi;
    while lo > 0.3 && u(lo) < CONTACT_ENERGY {
        hi = lo;
        lo *= 0.8;
    }
    // If the contact energy is never bracketed the pair is too soft to have a contact
    // separation on this interval, and the outermost scanned point is the honest answer.
    let sep = bisect(lo, hi, 30, &mut |r| u(r) - CONTACT_ENERGY).unwrap_or(hi);
    SpeciesSize {
        radius_bohr: 0.5 * sep,
        rule: RadiusRule::HalfContact,
        separation_bohr: sep,
        d_e: None,
        n_det,
        ms: t0.ms(),
    }
}

/// The DECLARED colour rule: hue walks the first row from teal to warm as `Z` rises.
///
/// This is a choice and it is labelled as one. Nothing physical picks a colour, and the
/// alternative — the CPK convention chemists use — is a different arbitrary choice with a
/// longer history. What this rule buys over CPK is that it is MONOTONE in `Z`: two
/// elements next to each other in the row are next to each other on screen, so a scene
/// says at a glance which atoms are heavier. Returned as `#rrggbb`, computed here so the
/// rule lives in one place rather than in a table of ten strings.
pub fn declared_colour(z: u32) -> String {
    let t = (z as f64 - 1.0) / 9.0;
    let hue = 168.0 - 126.0 * t;
    let sat = 0.38 + 0.16 * t;
    let light = 0.60 - 0.14 * t;
    let (r, g, b) = hsl_to_rgb(hue, sat, light);
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = h / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match hp as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - 0.5 * c;
    let q = |v: f64| ((v + m).clamp(0.0, 1.0) * 255.0).round() as u8;
    (q(r1), q(g1), q(b1))
}

#[cfg(test)]
mod bisect_tests {
    use super::bisect;

    /// A bracketed root is found, and to the precision the halvings buy.
    #[test]
    fn a_bracketed_root_is_located() {
        // sqrt(2) on [1, 2]: 40 halvings of a unit interval is 1e-12.
        let r = bisect(1.0, 2.0, 40, &mut |x| x * x - 2.0).expect("bracketed");
        assert!(
            (r - std::f64::consts::SQRT_2).abs() < 1e-11,
            "bisection landed at {r}, not sqrt(2)"
        );
        // Either orientation of the same bracket.
        let r2 = bisect(2.0, 1.0, 40, &mut |x| x * x - 2.0).expect("bracketed, reversed");
        assert!((r2 - std::f64::consts::SQRT_2).abs() < 1e-11);
    }

    /// An interval whose endpoints share a sign is REFUSED rather than converged on.
    ///
    /// This is the whole reason the signature returns an `Option`. Without the check,
    /// bisection on `x^2 + 1` over `[1, 2]` does not error — it walks to an endpoint and
    /// returns it, and the caller receives a number indistinguishable from a root.
    #[test]
    fn an_unbracketed_interval_is_refused() {
        assert!(bisect(1.0, 2.0, 40, &mut |x| x * x + 1.0).is_none(), "no root, both positive");
        assert!(bisect(1.0, 2.0, 40, &mut |x| -(x * x) - 1.0).is_none(), "no root, both negative");
        // A root outside the interval is still not a root inside it.
        assert!(bisect(1.0, 2.0, 40, &mut |x| x - 5.0).is_none());
        // And a non-finite endpoint is a refusal rather than a NaN comparison.
        assert!(bisect(1.0, 2.0, 40, &mut |x| if x < 1.5 { f64::NAN } else { 1.0 }).is_none());
    }

    /// A root exactly at an endpoint is a bracket, because the product is zero rather
    /// than positive — the boundary case the `> 0.0` test is written for.
    #[test]
    fn a_root_on_the_endpoint_is_still_bracketed() {
        assert!(bisect(2.0, 3.0, 30, &mut |x| x - 2.0).is_some());
        assert!(bisect(2.0, 3.0, 30, &mut |x| x - 3.0).is_some());
    }
}
