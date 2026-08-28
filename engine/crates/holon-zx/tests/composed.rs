//! THE COMPOSED PIPELINE'S CERTIFICATE: quizx may simplify however it
//! likes, but the amplitude must not move. Every basis state, exact ring
//! equality — the same bar our own passes carry, applied to a third-party
//! simplifier we do not control.
#![cfg(feature = "zx")]
use holon::magic::cyc_eq;
use holon::qasm::Surface::*;
use holon::run::amplitude;
use holon_zx::canonicalize;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

#[test]
fn canonicalization_preserves_every_amplitude() {
    let mut rng = Rng(0xC0FFEE);
    for trial in 0..10 {
        let n = 3;
        let mut g = Vec::new();
        for _ in 0..25 {
            let q = rng.below(n);
            let mut q2 = rng.below(n);
            while q2 == q {
                q2 = rng.below(n);
            }
            g.push(match rng.below(7) {
                0 => H(q),
                1 => S(q),
                2 => T(q),
                3 => Tdg(q),
                4 => Cx(q, q2),
                5 => Cz(q, q2),
                _ => Z(q),
            });
        }
        let (simplified, red) = match canonicalize(n, &g) {
            Ok(v) => v,
            Err(e) => panic!("trial {trial}: {e}"),
        };
        assert!(red.t_after <= red.t_before, "canonicalizer increased T-count");
        let (a, _) = holon::qasm::lower(&g);
        let (b, _) = holon::qasm::lower(&simplified);
        // The recovered ω-phase is applied to the simplified side, so the
        // gate is on AMPLITUDES, not probabilities.
        let mut corr = holon::ledger::Cyc::ONE;
        for _ in 0..red.phase_omega.rem_euclid(8) {
            corr = corr.mul(holon::affine::omega_pow(1));
        }
        for k in 0..(1u32 << n) {
            let y: Vec<bool> = (0..n).map(|q| k >> q & 1 == 1).collect();
            let lhs = amplitude(n, &a, &y);
            let rhs = amplitude(n, &b, &y).mul(corr);
            assert!(
                cyc_eq(lhs, rhs),
                "trial {trial} basis {k}: canonicalizer changed the amplitude \
                 (phase_omega={}) {:?} vs {:?}",
                red.phase_omega,
                lhs.to_complex(),
                rhs.to_complex()
            );
        }
    }
}

#[test]
fn face_and_generic_rotations_do_not_cross() {
    let e = holon_zx::to_qasm(1, &[Rot(0)]).unwrap_err();
    assert!(e.contains("Clifford+T only"), "the boundary must be named: {e}");
}
