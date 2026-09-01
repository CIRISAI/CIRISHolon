//! **`sigma = H c` as an OBJECT that declares the device class it computes on.**
//!
//! RESOURCE_DESIGN **D0**: the device class belongs to the artifact, not to the schedule. That
//! rule needs somewhere to live in the solver, and this is it. Before this module the matvec was
//! a free function; a free function cannot be a different device, so "which device produced this
//! energy" was not a question the solve could answer, let alone stamp on its result.
//!
//! # Why the whole solve takes ONE provider and not a per-stage choice
//!
//! `solve_determinant_with` applies the Hamiltonian four times over: the Davidson eigensolve
//! (`H`), the first derivative (`H'`), the second (`H''`), and the conjugate-gradient response
//! (`H` again). Letting the Davidson run on the GPU while the derivatives ran on the host would
//! produce a `Solution` whose energy came from one class and whose curvature came from another —
//! a MIXED artifact, which is exactly what D0 forbids and exactly what a per-stage device
//! argument would quietly permit. So the class is chosen once, for the solve, by a
//! [`SigmaProvider`], and `solve_determinant_with` REFUSES if any operator it built disagrees
//! with the provider about what it is.
//!
//! # The size of the difference, so nobody treats this as bookkeeping
//!
//! SATURATION-3 G2 measured both arms on the real `(O,O,O)` problem: they agree to **3.033e-15**
//! relative and **91.0% of the 207,025 entries differ BITWISE**. Both are correct. A Davidson
//! driven by one of them takes a different path from a Davidson driven by the other, so a
//! GPU-built table is a different artifact from a CPU-built one and the two may never be mixed
//! inside one bit-gated table.
//!
//! # What is NOT here
//!
//! No CUDA. `holon-chem` has one dependency — `holon-device`, which is `no_std` and defines an
//! enum — and it ships into the browser wasm bundle, so the device arm cannot live here. It
//! lives in `holon-gpu`, which implements these traits from the outside. That inversion is the
//! point: this crate names the contract and stays portable; the device crate satisfies it.

use crate::fci::{CiInts, FciSpace};
use crate::scalar::Scalar;
use crate::tier::sigma_direct_t;

pub use holon_device::DeviceClass;

/// One application of the Hamiltonian, on a declared device class.
///
/// `&mut self` because a device operator owns scratch that the application writes through —
/// upload buffers, an intermediate `T`, a cuBLAS handle. A host operator needs none of it and
/// simply ignores the mutability.
pub trait SigmaOp<T: Scalar> {
    /// The dimension this operator is built for. Checked against the vectors it is handed, so a
    /// start vector from another geometry's space is stopped rather than reshaped.
    fn n_det(&self) -> usize;

    /// The class this operator computes on. Travels onto the artifact.
    fn device(&self) -> DeviceClass;

    /// `sigma <- H c`. Overwrites `sigma`; does not accumulate.
    fn apply(&mut self, c: &[T], sigma: &mut [T]);
}

/// The host arm: `sigma_direct_t`, the Knowles–Handy string factorisation, exactly as it was.
///
/// This is a wrapper and nothing more — it must stay one. The whole value of the refactor is
/// that the CPU path through the new driver is bit-identical to the old one, and any
/// "improvement" made here while passing through would destroy that property silently.
pub struct CpuSigma<'a, T: Scalar> {
    pub space: &'a FciSpace,
    pub k: &'a [T],
    pub g: &'a [T],
}

impl<'a, T: Scalar> CpuSigma<'a, T> {
    pub fn new(space: &'a FciSpace, k: &'a [T], g: &'a [T]) -> Self {
        CpuSigma { space, k, g }
    }
}

impl<T: Scalar> SigmaOp<T> for CpuSigma<'_, T> {
    fn n_det(&self) -> usize {
        self.space.n_det
    }
    fn device(&self) -> DeviceClass {
        DeviceClass::Cpu
    }
    fn apply(&mut self, c: &[T], sigma: &mut [T]) {
        sigma_direct_t(self.space, self.k, self.g, c, sigma);
    }
}

/// A source of sigma operators bound to ONE device class.
///
/// A solve applies several different Hamiltonians (the energy's, and its first and second
/// derivatives'), and D0 is a statement about the whole artifact rather than about one matvec.
/// So the class is chosen once, here, and every operator the solve uses comes from the same
/// provider.
///
/// `f64` only, deliberately: the device arm is `f64`, and a provider generic over `Scalar` would
/// advertise a `Dd` GPU tier that does not exist. The double-double tier is a CPU rung of the
/// D3b ladder and reaches the solver through `tier::refine_determinant_dd`, not through here.
pub trait SigmaProvider {
    /// The class every operator from this provider computes on.
    fn device(&self) -> DeviceClass;

    /// Build the operator for one integral set.
    ///
    /// Fallible because a device provider can fail to build one — no device, no VRAM, a lease
    /// refused — and D4 says that is a LOUD refusal, never a quiet fall back to the host. The
    /// error is a `String` because the reasons come from outside this crate and this crate is
    /// not going to enumerate a foreign driver's failure modes.
    fn op_for<'a>(
        &self,
        space: &'a FciSpace,
        ci: &'a CiInts,
    ) -> Result<Box<dyn SigmaOp<f64> + 'a>, String>;
}

/// The host provider. The default everywhere, and the reference every device arm is measured
/// against.
#[derive(Clone, Copy, Debug, Default)]
pub struct CpuProvider;

impl SigmaProvider for CpuProvider {
    fn device(&self) -> DeviceClass {
        DeviceClass::Cpu
    }
    fn op_for<'a>(
        &self,
        space: &'a FciSpace,
        ci: &'a CiInts,
    ) -> Result<Box<dyn SigmaOp<f64> + 'a>, String> {
        Ok(Box::new(CpuSigma::new(space, &ci.k, &ci.g)))
    }
}

/// Run five applications of `op` on the same input and require every one to be BITWISE
/// identical to the first.
///
/// This is the determinism gate D0 makes an adoption condition, and it is a function rather
/// than a comment because "the reduction order is fixed" is a claim about a build, a driver and
/// a library's kernel selection — none of which this repository controls. It is checked where
/// the operator is, on the operator that will actually be used.
///
/// Returns the number of entries that differed, and on which repeat. Zero is the only passing
/// answer: a bound on the disagreement is not a substitute when the output is compared
/// bit-for-bit.
///
/// **The carrier is asserted first.** A sigma that is identically zero repeats bit-identically
/// for the most useless of reasons, so a degenerate input is refused rather than scored — the
/// same reason `s3_sigma_export` refuses an all-zero reference.
pub fn bit_identity_over_runs(
    op: &mut dyn SigmaOp<f64>,
    c: &[f64],
    runs: usize,
) -> Result<(), String> {
    assert!(runs >= 2, "a bit-identity gate over fewer than two runs checks nothing");
    let nd = op.n_det();
    assert_eq!(c.len(), nd, "the probe vector is not this operator's dimension");

    let mut first = vec![0.0f64; nd];
    op.apply(c, &mut first);

    // THE CARRIER. An operator that returns zeros, or a handful of nonzeros, would pass a
    // bit-identity gate while computing nothing.
    let nz = first.iter().filter(|x| **x != 0.0).count();
    if nz * 2 < nd {
        return Err(format!(
            "the determinism gate has an empty carrier: only {nz} of {nd} sigma entries are \
             nonzero, so run-to-run agreement would be a statement about the zeros. Refused \
             rather than passed."
        ));
    }

    let mut again = vec![0.0f64; nd];
    for r in 1..runs {
        op.apply(c, &mut again);
        let differing = first
            .iter()
            .zip(again.iter())
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count();
        if differing != 0 {
            return Err(format!(
                "repeat {r} of {runs} differed from the first run in {differing} of {nd} \
                 entries BITWISE. {:?} does not have a fixed reduction order, so it cannot \
                 carry a bit-gated workload however small the numerical difference is.",
                op.device()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::by_symbol;
    use crate::fci::{ci_ints, sigma_direct, Order};
    use crate::pair::geometry_problem;
    use crate::dual::D2;

    fn h3_problem() -> (FciSpace, CiInts, Vec<f64>) {
        let h = by_symbol("H").unwrap();
        let at = |x: f64| [D2::c(x), D2::c(0.0), D2::c(0.0)];
        let (space, mo, _) = geometry_problem(&[h, h, h], vec![at(0.0), at(1.6), at(3.2)]);
        let ci = ci_ints(&mo, Order::Value);
        let mut c = vec![0.0f64; space.n_det];
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        for x in c.iter_mut() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            *x = ((seed >> 33) as f64 / (1u64 << 31) as f64) - 0.5;
        }
        (space, ci, c)
    }

    /// The wrapper is a wrapper: `CpuSigma` must be BIT-identical to `sigma_direct`, not merely
    /// close to it. If this ever fails, every table in the repository was re-keyed by a
    /// refactor that was supposed to be structural.
    #[test]
    fn the_host_operator_is_bit_identical_to_sigma_direct() {
        let (space, ci, c) = h3_problem();
        let mut want = vec![0.0f64; space.n_det];
        sigma_direct(&space, &ci, &c, &mut want);

        let mut op = CpuSigma::new(&space, &ci.k, &ci.g);
        let mut got = vec![0.0f64; space.n_det];
        op.apply(&c, &mut got);

        let diff = want
            .iter()
            .zip(got.iter())
            .filter(|(a, b)| a.to_bits() != b.to_bits())
            .count();
        assert_eq!(diff, 0, "{diff} of {} entries differ bitwise", space.n_det);
        assert_eq!(op.device(), DeviceClass::Cpu);
    }

    /// The host arm passes its own determinism gate — and the gate is capable of failing, which
    /// the next test demonstrates. Without both halves "it always passes" would satisfy this.
    #[test]
    fn the_host_operator_passes_the_bit_identity_gate() {
        let (space, ci, c) = h3_problem();
        let mut op = CpuSigma::new(&space, &ci.k, &ci.g);
        bit_identity_over_runs(&mut op, &c, 5).expect("the host arm is not run-to-run identical");
    }

    /// **The gate can fail.** An operator that perturbs its answer by one ULP on every repeat is
    /// numerically indistinguishable from the honest one and must still be refused, because
    /// bit-identity is the property a bit-gated table needs.
    #[test]
    fn the_bit_identity_gate_refuses_a_one_ulp_wobble() {
        struct Wobbly<'a> {
            inner: CpuSigma<'a, f64>,
            calls: usize,
        }
        impl SigmaOp<f64> for Wobbly<'_> {
            fn n_det(&self) -> usize {
                self.inner.n_det()
            }
            fn device(&self) -> DeviceClass {
                DeviceClass::Gpu
            }
            fn apply(&mut self, c: &[f64], sigma: &mut [f64]) {
                self.inner.apply(c, sigma);
                if self.calls > 0 {
                    // one ULP on one entry: ~1e-16 relative, and fatal to a bit-gated table
                    sigma[0] = f64::from_bits(sigma[0].to_bits() ^ 1);
                }
                self.calls += 1;
            }
        }
        let (space, ci, c) = h3_problem();
        let mut op = Wobbly {
            inner: CpuSigma::new(&space, &ci.k, &ci.g),
            calls: 0,
        };
        let err = bit_identity_over_runs(&mut op, &c, 5)
            .expect_err("a one-ULP wobble survived the determinism gate");
        assert!(err.contains("BITWISE"), "{err}");
    }

    /// **The carrier is asserted.** An operator that returns zeros repeats bit-identically, and
    /// the gate must refuse it rather than score it — M-PLANT-SECTOR at the instrument level.
    #[test]
    fn the_bit_identity_gate_refuses_a_degenerate_carrier() {
        struct Zeros(usize);
        impl SigmaOp<f64> for Zeros {
            fn n_det(&self) -> usize {
                self.0
            }
            fn device(&self) -> DeviceClass {
                DeviceClass::Cpu
            }
            fn apply(&mut self, _c: &[f64], sigma: &mut [f64]) {
                sigma.fill(0.0);
            }
        }
        let mut op = Zeros(64);
        let c = vec![1.0f64; 64];
        let err = bit_identity_over_runs(&mut op, &c, 5)
            .expect_err("a constant-zero operator passed a determinism gate");
        assert!(err.contains("empty carrier"), "{err}");
    }
}
