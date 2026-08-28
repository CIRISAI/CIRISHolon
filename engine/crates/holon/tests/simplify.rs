//! The simplification pass must be EXACT: identical amplitudes on every
//! basis state, before and after, on circuits carrying every gate class the
//! pass touches — and it must actually reduce the magic weight where the
//! algebra says it should.
use holon::qasm::Surface::{self, *};
use holon::run::amplitude;
use holon::simplify::{magic_weight, simplify};

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

fn rand_surface(rng: &mut Rng, n: usize, len: usize) -> Vec<Surface> {
    let mut g = Vec::new();
    for _ in 0..len {
        let q = rng.below(n);
        let mut q2 = rng.below(n);
        while q2 == q {
            q2 = rng.below(n);
        }
        let q3 = (0..n).find(|&x| x != q && x != q2).unwrap_or(q);
        g.push(match rng.below(12) {
            0 => X(q),
            1 => Z(q),
            2 => H(q),
            3 => S(q),
            4 => Sdg(q),
            5 => T(q),
            6 => Tdg(q),
            7 => Cx(q, q2),
            8 => Cz(q, q2),
            9 => Ccz(q, q2, q3),
            10 => DiagPow(rng.below(8) as i64, q),
            _ => Swap(q, q2),
        });
    }
    g
}

#[test]
fn simplification_preserves_every_amplitude() {
    let mut rng = Rng(0x5111);
    for trial in 0..20 {
        let n = 3;
        let orig = rand_surface(&mut rng, n, 40);
        let simp = simplify(&orig);
        let (co, _) = holon::qasm::lower(&orig);
        let (cs, _) = holon::qasm::lower(&simp);
        for b in 0..(1u32 << n) {
            let y: Vec<bool> = (0..n).map(|q| b >> q & 1 == 1).collect();
            let a = amplitude(n, &co, &y);
            let s = amplitude(n, &cs, &y);
            assert!(
                holon::magic::cyc_eq(a, s),
                "trial {trial} basis {b}: simplification changed the amplitude"
            );
        }
    }
}

#[test]
fn diagonal_runs_collapse_and_magic_drops() {
    // The hidden-shift shape: long diagonal runs with repeats.
    let mut prog = vec![H(0), H(1), H(2)];
    for _ in 0..50 {
        prog.push(Z(0));
        prog.push(Z(1));
        prog.push(Cz(0, 1));
        prog.push(Z(0));
        prog.push(Cz(0, 1));
    }
    prog.push(Ccz(0, 1, 2));
    prog.push(Ccz(2, 1, 0)); // same triple, must cancel: 14 T-equivalents gone
    prog.push(H(0));
    let simp = simplify(&prog);
    assert!(simp.len() < prog.len() / 4, "expected a large collapse, got {} from {}", simp.len(), prog.len());
    assert_eq!(magic_weight(&simp), 0, "the CCZ pair must cancel exactly");
    assert_eq!(magic_weight(&prog), 14, "the unsimplified weight is two CCZs");
    // and it is still exact
    let (co, _) = holon::qasm::lower(&prog);
    let (cs, _) = holon::qasm::lower(&simp);
    for b in 0..8u32 {
        let y: Vec<bool> = (0..3).map(|q| b >> q & 1 == 1).collect();
        assert!(holon::magic::cyc_eq(amplitude(3, &co, &y), amplitude(3, &cs, &y)));
    }
}

#[test]
fn face_and_symbolic_rotations_are_handled_exactly() {
    // Face(+1) and Face(−1) on the same qubit cancel; generic Rots do not.
    let prog = vec![H(0), Face(1, 0), Z(0), Face(-1, 0), Rot(0), Rot(0), H(0)];
    let simp = simplify(&prog);
    assert_eq!(
        simp.iter().filter(|g| matches!(g, Face(..))).count(),
        0,
        "opposite face rotations must cancel"
    );
    assert_eq!(
        simp.iter().filter(|g| matches!(g, Rot(_))).count(),
        2,
        "generic rotations at the same symbolic angle must NOT cancel"
    );
}
