//! THE EXACTNESS GATE for the native ZX canonical form.
//!
//! `zx.rs` ships a T-count oracle, not a circuit extractor — so the gate
//! cannot be "extract and compare circuits". It is stated one level down, on
//! the diagram itself, and it is stronger than a T-count comparison:
//!
//!   for every random Clifford+T circuit and every basis state, the CLOSED
//!   diagram's exact value — before rewriting, after Clifford simplification,
//!   and after full reduction — equals `run::amplitude` on the same circuit,
//!   EXACTLY as an element of `Z[ω]·2^{−m/2}`. Not up to a global phase, not
//!   to a tolerance: equal.
//!
//! That certifies three things at once: the circuit → diagram construction,
//! every rewrite rule's action, and every rewrite rule's scalar.
use holon::ledger::Cyc;
use holon::qasm::Surface::{self, *};
use holon::run::amplitude;
use holon::zx::{cyc_eq, from_surface};

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

/// `⟨y|C|0…0⟩` as the certified runner computes it.
fn reference(n: usize, surf: &[Surface], y: &[bool]) -> Cyc {
    let (core, _) = holon::qasm::lower(surf);
    amplitude(n, &core, y)
}

/// The same amplitude as the ZX diagram computes it, under `stage`.
fn zx_amplitude(n: usize, surf: &[Surface], y: &[bool], stage: fn(&mut holon::zx::ZxGraph)) -> Cyc {
    let mut g = from_surface(n, surf).unwrap();
    g.plug_inputs(&vec![false; n]);
    g.plug_outputs(y);
    stage(&mut g);
    g.eval()
}

fn sweep(seed: u64, trials: usize, n: usize, depth: usize, stage: fn(&mut holon::zx::ZxGraph)) {
    let mut rng = Rng(seed);
    for trial in 0..trials {
        let surf = rand_surface(&mut rng, n, depth);
        for b in 0..(1u32 << n) {
            let y: Vec<bool> = (0..n).map(|q| b >> q & 1 == 1).collect();
            let want = reference(n, &surf, &y);
            let got = zx_amplitude(n, &surf, &y, stage);
            assert!(
                cyc_eq(want, got),
                "trial {trial} basis {b:03b}: runner {:?} vs zx {:?}\ncircuit {surf:?}",
                want.to_complex(),
                got.to_complex()
            );
        }
    }
}

#[test]
fn construction_reproduces_the_runners_amplitude_exactly() {
    sweep(0x2ea1, 25, 3, 24, |_| {});
}

#[test]
fn clifford_simplification_preserves_every_amplitude_exactly() {
    sweep(0x2ea2, 25, 3, 24, |g| {
        g.clifford_simp();
    });
}

#[test]
fn full_reduction_preserves_every_amplitude_exactly() {
    sweep(0x2ea3, 25, 3, 24, |g| {
        g.full_reduce();
    });
}

#[test]
fn full_reduction_is_exact_on_wider_and_deeper_circuits() {
    sweep(0x2ea4, 8, 4, 60, |g| {
        g.full_reduce();
    });
}

/// The boundary path: reduce the OPEN diagram (boundaries still free, so
/// `gen_pivot` must unfuse them), THEN plug and evaluate. This is the rung
/// the old pass never reached, and it is where a wrong boundary rule shows.
#[test]
fn open_diagram_reduction_preserves_every_amplitude_exactly() {
    let mut rng = Rng(0x2ea5);
    for trial in 0..30 {
        let n = if trial % 2 == 0 { 3 } else { 4 };
        let surf = rand_surface(&mut rng, n, 24 + 12 * (trial % 3));
        let mut reduced = from_surface(n, &surf).unwrap();
        reduced.full_reduce();
        for b in 0..(1u32 << n) {
            let y: Vec<bool> = (0..n).map(|q| b >> q & 1 == 1).collect();
            let want = reference(n, &surf, &y);
            let mut g = reduced.clone();
            g.plug_inputs(&vec![false; n]);
            g.plug_outputs(&y);
            let got = g.eval();
            assert!(
                cyc_eq(want, got),
                "trial {trial} basis {b:03b}: runner {:?} vs open-reduced zx {:?}",
                want.to_complex(),
                got.to_complex()
            );
        }
    }
}

/// Full reduction must never make the T-count worse, on either diagram.
#[test]
fn reduction_never_increases_t_count() {
    let mut rng = Rng(0x2ea6);
    for _ in 0..30 {
        let n = 4;
        let surf = rand_surface(&mut rng, n, 40);
        let raw = from_surface(n, &surf).unwrap().t_count();
        assert!(holon::zx::full_reduced_t_count(n, &surf).unwrap() <= raw);
        let zero = vec![false; n];
        assert!(holon::zx::amplitude_t_count(n, &surf, &zero, &zero).unwrap() <= raw);
    }
}

/// The Clifford layer never INCREASES the count (its graph rewrites are
/// parity-preserving; only fusion and the scalar rules remove), and full
/// reduction is at least as good as it — the gadget layer is the difference.
#[test]
fn full_reduction_dominates_the_clifford_layer() {
    let mut rng = Rng(0x2ea7);
    for _ in 0..20 {
        let n = 3;
        let surf = rand_surface(&mut rng, n, 30);
        let raw = from_surface(n, &surf).unwrap().t_count();
        let cliff = holon::zx::simplified_t_count(n, &surf).unwrap();
        let full = holon::zx::full_reduced_t_count(n, &surf).unwrap();
        assert!(cliff <= raw, "Clifford layer grew the T-count: {raw} -> {cliff}");
        assert!(full <= cliff, "full reduction lost to the Clifford layer: {cliff} -> {full}");
    }
}
