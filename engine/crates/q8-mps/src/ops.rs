//! Local `2x2` spin-orbital operators and the Kronecker product `mpo.rs` builds the dense
//! reference from. Basis order `(|0>,|1>)`: index 0 = empty, index 1 = occupied.

pub type Op2 = [[f64; 2]; 2];

/// Identity.
pub const I2: Op2 = [[1.0, 0.0], [0.0, 1.0]];
/// `n = diag(0,1)`.
pub const N2: Op2 = [[0.0, 0.0], [0.0, 1.0]];
/// `Z = 1-2n = diag(1,-1)`, the Jordan–Wigner string factor.
pub const Z2: Op2 = [[1.0, 0.0], [0.0, -1.0]];
/// `sigma+`: raises `|0> -> |1>` (creation, with the string handled separately).
pub const CD2: Op2 = [[0.0, 0.0], [1.0, 0.0]];
/// `sigma-`: lowers `|1> -> |0>` (annihilation).
pub const CM2: Op2 = [[0.0, 1.0], [0.0, 0.0]];

/// `kron(outer, inner)`, `outer` a `2x2` block, `inner` a row-major `n x n` dense matrix, result
/// row-major `2n x 2n`. Placing the NEW site as the outer (high-bit) factor is what makes basis
/// index bit `q` mean "occupation of site `q`" after building the chain up one site at a time —
/// checked directly against `mpo.rs`'s bulk contraction, not merely assumed.
pub fn kron(outer: &Op2, inner: &[f64], n: usize) -> Vec<f64> {
    let dim = 2 * n;
    let mut out = vec![0.0; dim * dim];
    for (oi, row) in outer.iter().enumerate() {
        for (oj, &ov) in row.iter().enumerate() {
            if ov == 0.0 {
                continue;
            }
            for ii in 0..n {
                let dst = (oi * n + ii) * dim + oj * n;
                let src = ii * n;
                for ij in 0..n {
                    out[dst + ij] += ov * inner[src + ij];
                }
            }
        }
    }
    out
}
