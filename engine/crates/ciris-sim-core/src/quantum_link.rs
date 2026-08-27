//! Finite U(1) quantum-link probe for the DM-vacuum research path.
//!
//! This is a realization over the existing three-state holon carrier, not a new holon
//! type. It deliberately does NOT identify `RouteSymmetry::routeH(0)` with the spin-1
//! quantum-link raising operator; they share a three-state carrier, while the operator
//! bridge remains something to derive or kill.
//!
//! Executable gates provided here:
//! * spin-1 electric flux -1,0,+1 and truncated raising/lowering;
//! * exact lattice Gauss charge on an oriented plaquette;
//! * magnetic plaquette moves preserve Gauss charge where the truncation permits them;
//! * the one-plaquette Gauss-closed basis is exactly three uniform-flux states;
//! * a minimal electric + plaquette Hamiltonian is diagonalized by the crate's existing
//!   deterministic Jacobi solver;
//! * the one-link modular spectrum is checked against `a I + beta E^2`;
//! * GF(2) boundary-channel rank counts independent cross-boundary channels rather than
//!   raw microscopic relations.

use crate::linalg::jacobi_eigen;

pub const FLUX_NEG: i8 = -1;
pub const FLUX_ZERO: i8 = 0;
pub const FLUX_POS: i8 = 1;

#[inline]
pub const fn valid_flux(e: i8) -> bool {
    e >= FLUX_NEG && e <= FLUX_POS
}

#[inline]
pub const fn raise_flux(e: i8) -> Option<i8> {
    match e {
        FLUX_NEG => Some(FLUX_ZERO),
        FLUX_ZERO => Some(FLUX_POS),
        _ => None,
    }
}

#[inline]
pub const fn lower_flux(e: i8) -> Option<i8> {
    match e {
        FLUX_POS => Some(FLUX_ZERO),
        FLUX_ZERO => Some(FLUX_NEG),
        _ => None,
    }
}

pub type Plaquette = [i8; 4];

/// Vertex Gauss charge on a consistently oriented four-link loop.
#[inline]
pub const fn gauss_charge(c: &Plaquette, vertex: usize) -> i8 {
    let incoming = if vertex == 0 { 3 } else { vertex - 1 };
    c[vertex] - c[incoming]
}

pub fn is_gauss_closed(c: &Plaquette) -> bool {
    valid_flux(c[0])
        && valid_flux(c[1])
        && valid_flux(c[2])
        && valid_flux(c[3])
        && gauss_charge(c, 0) == 0
        && gauss_charge(c, 1) == 0
        && gauss_charge(c, 2) == 0
        && gauss_charge(c, 3) == 0
}

/// Oriented magnetic plaquette raising move. `None` is the finite spin-1 truncation at
/// +1, not a gauge violation.
pub fn plaquette_raise(c: &Plaquette) -> Option<Plaquette> {
    Some([
        raise_flux(c[0])?,
        raise_flux(c[1])?,
        raise_flux(c[2])?,
        raise_flux(c[3])?,
    ])
}

pub fn plaquette_lower(c: &Plaquette) -> Option<Plaquette> {
    Some([
        lower_flux(c[0])?,
        lower_flux(c[1])?,
        lower_flux(c[2])?,
        lower_flux(c[3])?,
    ])
}

/// Exact Gauss-closed one-plaquette basis: one common flux around the full loop.
pub const CLOSED_FLUX_BASIS: [Plaquette; 3] = [
    [FLUX_NEG; 4],
    [FLUX_ZERO; 4],
    [FLUX_POS; 4],
];

/// Minimal Hamiltonian in the closed-flux basis `|-1>, |0>, |+1>`:
/// `H = 4 g2 E^2 - kappa (U + U^dagger)`.
pub fn one_plaquette_hamiltonian(g2: f64, kappa: f64) -> Option<[[f64; 3]; 3]> {
    if !g2.is_finite() || !kappa.is_finite() || g2 < 0.0 || kappa < 0.0 {
        return None;
    }
    let electric = 4.0 * g2;
    Some([
        [electric, -kappa, 0.0],
        [-kappa, 0.0, -kappa],
        [0.0, -kappa, electric],
    ])
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OnePlaquetteVacuum {
    pub ground_energy: f64,
    /// Basis amplitudes in `|-1>,|0>,|+1>` order. Overall sign is irrelevant.
    pub amplitudes: [f64; 3],
    /// A one-link reduced state is diagonal with these probabilities because the
    /// environment flux labels are orthogonal Gauss-closed sectors.
    pub probabilities: [f64; 3],
    /// Coefficient of `E^2` in `-log rho_link`, beta = log(p0 / p_outer).
    pub modular_beta: f64,
    pub charge_conjugation_residual: f64,
    /// Relative residual of the three modular energies against `a + beta E^2`.
    pub modular_electric_fit_residual: f64,
}

pub fn one_plaquette_vacuum(g2: f64, kappa: f64) -> Option<OnePlaquetteVacuum> {
    let h = one_plaquette_hamiltonian(g2, kappa)?;
    let eig = jacobi_eigen(&h);
    if !eig.converged {
        return None;
    }

    let amplitudes = eig.vectors[0];
    let probabilities = [
        amplitudes[0] * amplitudes[0],
        amplitudes[1] * amplitudes[1],
        amplitudes[2] * amplitudes[2],
    ];
    let p_outer = 0.5 * (probabilities[0] + probabilities[2]);
    let p_zero = probabilities[1];
    if p_outer <= 0.0 || p_zero <= 0.0 {
        return None;
    }

    let beta = libm::log(p_zero / p_outer);
    let a = -libm::log(p_zero);
    let observed = [
        -libm::log(probabilities[0]),
        -libm::log(probabilities[1]),
        -libm::log(probabilities[2]),
    ];
    let predicted = [a + beta, a, a + beta];

    let mut err2 = 0.0;
    let mut norm2 = 0.0;
    for i in 0..3 {
        let d = observed[i] - predicted[i];
        err2 += d * d;
        norm2 += observed[i] * observed[i];
    }
    let modular_residual = if norm2 == 0.0 {
        0.0
    } else {
        libm::sqrt(err2 / norm2)
    };

    Some(OnePlaquetteVacuum {
        ground_energy: eig.values[0],
        amplitudes,
        probabilities,
        modular_beta: beta,
        charge_conjugation_residual: libm::fabs(probabilities[0] - probabilities[2]),
        modular_electric_fit_residual: modular_residual,
    })
}

/// Number of raw undirected adjacency relations crossing a region boundary. The caller
/// supplies a symmetric adjacency matrix; only region->outside entries are counted.
pub fn raw_cut_edges<const N: usize>(adj: &[[bool; N]; N], region: &[bool; N]) -> usize {
    let mut count = 0usize;
    for i in 0..N {
        if !region[i] {
            continue;
        }
        for j in 0..N {
            if !region[j] && adj[i][j] {
                count += 1;
            }
        }
    }
    count
}

/// GF(2) rank of the cross-boundary adjacency map: independent binary boundary
/// channels after redundant microscopic relations are eliminated.
pub fn boundary_channel_rank<const N: usize>(
    adj: &[[bool; N]; N],
    region: &[bool; N],
) -> usize {
    let mut m = [[0u8; N]; N];
    for i in 0..N {
        if region[i] {
            for j in 0..N {
                if !region[j] && adj[i][j] {
                    m[i][j] = 1;
                }
            }
        }
    }

    let mut pivot_row = 0usize;
    for col in 0..N {
        let mut found = N;
        let mut r = pivot_row;
        while r < N {
            if m[r][col] != 0 {
                found = r;
                break;
            }
            r += 1;
        }
        if found == N {
            continue;
        }
        if found != pivot_row {
            m.swap(found, pivot_row);
        }
        for rr in 0..N {
            if rr == pivot_row || m[rr][col] == 0 {
                continue;
            }
            for cc in col..N {
                m[rr][cc] ^= m[pivot_row][cc];
            }
        }
        pivot_row += 1;
        if pivot_row == N {
            break;
        }
    }
    pivot_row
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauss_closed_basis_is_closed() {
        for c in &CLOSED_FLUX_BASIS {
            assert!(is_gauss_closed(c));
        }
    }

    #[test]
    fn plaquette_moves_preserve_gauss_exactly() {
        let values = [FLUX_NEG, FLUX_ZERO, FLUX_POS];
        for &a in &values {
            for &b in &values {
                for &c in &values {
                    for &d in &values {
                        let q = [a, b, c, d];
                        if let Some(qp) = plaquette_raise(&q) {
                            for v in 0..4 {
                                assert_eq!(gauss_charge(&qp, v), gauss_charge(&q, v));
                            }
                        }
                        if let Some(qm) = plaquette_lower(&q) {
                            for v in 0..4 {
                                assert_eq!(gauss_charge(&qm, v), gauss_charge(&q, v));
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn one_plaquette_modular_generator_is_electric_energy() {
        let v = one_plaquette_vacuum(1.0, 1.0).unwrap();
        assert!(v.charge_conjugation_residual < 1e-12);
        assert!(v.modular_electric_fit_residual < 1e-12);
        assert!(v.modular_beta > 0.0);
    }

    #[test]
    fn c5_raw_edges_and_channel_rank_are_different_observables() {
        let mut a = [[false; 5]; 5];
        for &(i, j) in &[(0usize, 1usize), (1, 2), (2, 3), (3, 4), (4, 0)] {
            a[i][j] = true;
            a[j][i] = true;
        }
        let adjacent = [true, true, false, false, false];
        let separated = [true, false, true, false, false];
        assert_eq!(raw_cut_edges(&a, &adjacent), 2);
        assert_eq!(raw_cut_edges(&a, &separated), 4);
        assert_eq!(boundary_channel_rank(&a, &adjacent), 2);
        assert_eq!(boundary_channel_rank(&a, &separated), 2);
    }
}
