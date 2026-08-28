//! THE FRONT-END'S CERTIFICATES: an independent dense oracle in the ζ16
//! ring, certifying EVERY lowering rule individually (rule-level transport
//! certificates) and the whole rewriter compositionally. The oracle shares
//! no code with the lowering: each surface gate is implemented from its
//! matrix, in Z[ζ16] = Z[ζ8] ⊕ ζ16·Z[ζ8], exactly.
use holon::ledger::Cyc;
use holon::magic::cyc_eq;
use holon::qasm::{lower, parse, Surface};
use holon::run::amplitude_program;

// ---------------------------------------------------------- Z[zeta16] ring
#[derive(Clone, Copy, Debug)]
struct Z16 {
    a: Cyc, // + zeta16 * b, with zeta16^2 = omega
    b: Cyc,
}
const ZERO16: Z16 = Z16 { a: Cyc::ZERO, b: Cyc::ZERO };
const ONE16: Z16 = Z16 { a: Cyc::ONE, b: Cyc::ZERO };
fn omega() -> Cyc {
    Cyc { c: [0, 1, 0, 0], m: 0 }
}
impl Z16 {
    fn add(self, o: Z16) -> Z16 {
        Z16 { a: self.a.add(o.a), b: self.b.add(o.b) }
    }
    fn mul(self, o: Z16) -> Z16 {
        // (a + zb)(c + zd) = (ac + omega bd) + z(ad + bc)
        Z16 {
            a: self.a.mul(o.a).add(self.b.mul(o.b).mul(omega())),
            b: self.a.mul(o.b).add(self.b.mul(o.a)),
        }
    }
    fn scale_cyc(self, c: Cyc) -> Z16 {
        Z16 { a: self.a.mul(c), b: self.b.mul(c) }
    }
    fn eq16(self, o: Z16) -> bool {
        cyc_eq(self.a, o.a) && cyc_eq(self.b, o.b)
    }
}
fn zeta16_pow(k: i64) -> Z16 {
    let k = k.rem_euclid(16);
    let (w, r) = (k / 2, k % 2);
    let mut ww = Cyc::ONE;
    for _ in 0..w {
        ww = ww.mul(omega());
    }
    if r == 0 {
        Z16 { a: ww, b: Cyc::ZERO }
    } else {
        Z16 { a: Cyc::ZERO, b: ww }
    }
}

// ------------------------------------------------- dense oracle evolution
struct Dense {
    n: usize,
    v: Vec<Z16>,
}
impl Dense {
    fn zero_state(n: usize) -> Dense {
        let mut v = vec![ZERO16; 1 << n];
        v[0] = ONE16;
        Dense { n, v }
    }
    fn bit(i: usize, q: usize) -> bool {
        i >> q & 1 == 1
    }
    fn diag1(&mut self, q: usize, phase: Z16) {
        for i in 0..self.v.len() {
            if Self::bit(i, q) {
                self.v[i] = self.v[i].mul(phase);
            }
        }
    }
    fn permute_x(&mut self, q: usize) {
        for i in 0..self.v.len() {
            if !Self::bit(i, q) {
                self.v.swap(i, i | (1 << q));
            }
        }
    }
    fn h(&mut self, q: usize) {
        // v'[i0] = (v[i0]+v[i1])/sqrt2 ; v'[i1] = (v[i0]-v[i1])/sqrt2
        let half = Cyc { c: [1, 0, 0, 0], m: 1 };
        let m1 = Cyc { c: [-1, 0, 0, 0], m: 0 };
        for i in 0..self.v.len() {
            if !Self::bit(i, q) {
                let j = i | (1 << q);
                let (a, b) = (self.v[i], self.v[j]);
                self.v[i] = a.add(b).scale_cyc(half);
                self.v[j] = a.add(b.scale_cyc(m1)).scale_cyc(half);
            }
        }
    }
    fn apply(&mut self, g: Surface) {
        use Surface::*;
        match g {
            X(q) => self.permute_x(q),
            Z(q) => self.diag1(q, zeta16_pow(8)),
            S(q) => self.diag1(q, zeta16_pow(4)),
            Sdg(q) => self.diag1(q, zeta16_pow(12)),
            T(q) => self.diag1(q, zeta16_pow(2)),
            Tdg(q) => self.diag1(q, zeta16_pow(14)),
            H(q) => self.h(q),
            Cx(a, b) => {
                for i in 0..self.v.len() {
                    if Self::bit(i, a) && !Self::bit(i, b) {
                        self.v.swap(i, i | (1 << b));
                    }
                }
            }
            // superset gates FROM THEIR MATRICES — independent of the rules
            Y(q) => {
                // Y = [[0,-i],[i,0]]: |0> -> i|1>, |1> -> -i|0>
                self.permute_x(q);
                for i in 0..self.v.len() {
                    let ph = if Self::bit(i, q) { zeta16_pow(4) } else { zeta16_pow(12) };
                    self.v[i] = self.v[i].mul(ph);
                }
            }
            Sx(q) => {
                // 1/2 [[1+i, 1-i],[1-i, 1+i]]
                let pp = zeta16_pow(4); // i
                let half = Cyc { c: [1, 0, 0, 0], m: 2 };
                for i in 0..self.v.len() {
                    if !Self::bit(i, q) {
                        let j = i | (1 << q);
                        let (a, b) = (self.v[i], self.v[j]);
                        // (1+i)a + (1-i)b ; (1-i)a + (1+i)b, all /2
                        let onep = ONE16.add(pp);
                        let onem = ONE16.add(pp.mul(zeta16_pow(8))); // 1 - i
                        self.v[i] = a.mul(onep).add(b.mul(onem)).scale_cyc(half);
                        self.v[j] = a.mul(onem).add(b.mul(onep)).scale_cyc(half);
                    }
                }
            }
            Sxdg(q) => {
                let pm = zeta16_pow(12); // -i
                let half = Cyc { c: [1, 0, 0, 0], m: 2 };
                for i in 0..self.v.len() {
                    if !Self::bit(i, q) {
                        let j = i | (1 << q);
                        let (a, b) = (self.v[i], self.v[j]);
                        let onep = ONE16.add(pm); // 1 - i
                        let onem = ONE16.add(pm.mul(zeta16_pow(8))); // 1 + i
                        self.v[i] = a.mul(onep).add(b.mul(onem)).scale_cyc(half);
                        self.v[j] = a.mul(onem).add(b.mul(onep)).scale_cyc(half);
                    }
                }
            }
            Cz(a, b) => {
                for i in 0..self.v.len() {
                    if Self::bit(i, a) && Self::bit(i, b) {
                        self.v[i] = self.v[i].mul(zeta16_pow(8));
                    }
                }
            }
            Swap(a, b) => {
                for i in 0..self.v.len() {
                    if Self::bit(i, a) && !Self::bit(i, b) {
                        self.v.swap(i, i ^ (1 << a) ^ (1 << b));
                    }
                }
            }
            Ccx(a, b, c) => {
                for i in 0..self.v.len() {
                    if Self::bit(i, a) && Self::bit(i, b) && !Self::bit(i, c) {
                        self.v.swap(i, i | (1 << c));
                    }
                }
            }
            Ccz(a, b, c) => {
                for i in 0..self.v.len() {
                    if Self::bit(i, a) && Self::bit(i, b) && Self::bit(i, c) {
                        self.v[i] = self.v[i].mul(zeta16_pow(8));
                    }
                }
            }
            Face(..) => unreachable!("Face is not in the core oracle's alphabet — the face engine (face.rs) carries it, with its own exact-ring tests"),
            DiagPow(k, q) => self.diag1(q, zeta16_pow(2 * k)),
            RzPow(k, q) => {
                self.diag1(q, zeta16_pow(2 * k));
                let s = zeta16_pow(-k);
                for x in self.v.iter_mut() {
                    *x = x.mul(s);
                }
            }
        }
    }
}

fn oracle_run(n: usize, gs: &[Surface]) -> Dense {
    let mut d = Dense::zero_state(n);
    for &g in gs {
        d.apply(g);
    }
    d
}

/// Engine value at basis y as a Z16 (ledger phase and residual folded in).
fn engine_value(p: &holon::qasm::Program, y: &[bool]) -> Z16 {
    let (amp, residual) = amplitude_program(p, y);
    let base = Z16 { a: amp, b: Cyc::ZERO };
    base.mul(zeta16_pow(residual as i64))
}

// -------------------------------------------------------------- the gates
/// RULE-LEVEL CERTIFICATES: for every superset gate, the oracle's direct
/// matrix action must equal the oracle's action of the LOWERED word times
/// the rule's scalar — on every basis state of a 3-qubit register. A
/// program-level failure can then never hide which square broke.
#[test]
fn every_rule_carries_its_certificate() {
    use Surface::*;
    let n = 3;
    let rules: Vec<Surface> = vec![
        Y(0), Sx(1), Sxdg(2), Cz(0, 1), Cz(1, 2), Swap(0, 2),
        Ccx(0, 1, 2), Ccx(2, 0, 1), Ccz(0, 1, 2),
        DiagPow(3, 1), RzPow(1, 0), RzPow(5, 2), RzPow(-3, 1),
    ];
    for g in rules {
        for basis in 0..1u32 << n {
            // prepare basis state via X's
            let prep: Vec<Surface> =
                (0..n).filter(|&q| basis >> q & 1 == 1).map(X).collect();
            let mut direct = oracle_run(n, &prep);
            direct.apply(g);
            let (word, scalar) = match holon::qasm::rule(g) {
                Err(wp) => wp,
                Ok(_) => continue,
            };
            let mut low = oracle_run(n, &prep);
            for w in &word {
                low.apply(*w);
            }
            let s = zeta16_pow(scalar);
            for i in 0..direct.v.len() {
                assert!(
                    direct.v[i].eq16(low.v[i].mul(s)),
                    "rule certificate FAILED for {g:?} on basis {basis} amp {i}"
                );
            }
        }
    }
}

/// COMPOSITIONAL CERTIFICATE: random surface circuits through the whole
/// pipeline (parse-shape → lower → production amplitude × ledger) must
/// equal the independent oracle at every basis amplitude, in Z[ζ16].
#[test]
fn pipeline_matches_oracle_on_random_surface_circuits() {
    use Surface::*;
    let mut seed = 0xFACADE_u64;
    let mut next = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        seed >> 11
    };
    for trial in 0..12 {
        let n = 3 + (next() % 2) as usize;
        let mut gs = Vec::new();
        for _ in 0..24 {
            let q = (next() % n as u64) as usize;
            let mut q2 = (next() % n as u64) as usize;
            if q2 == q {
                q2 = (q2 + 1) % n;
            }
            let q3 = (0..n).find(|&x| x != q && x != q2).unwrap();
            gs.push(match next() % 14 {
                0 => X(q), 1 => Y(q), 2 => Z(q), 3 => H(q), 4 => S(q),
                5 => T(q), 6 => Sx(q), 7 => Cx(q, q2), 8 => Cz(q, q2),
                9 => Swap(q, q2), 10 => Ccx(q, q2, q3), 11 => Ccz(q, q2, q3),
                12 => RzPow((next() % 8) as i64, q),
                _ => DiagPow((next() % 8) as i64, q),
            });
        }
        let (core, phase_16) = lower(&gs);
        let p16 = phase_16.rem_euclid(16);
        let prog = holon::qasm::Program {
            n_qubits: n,
            gates: core,
            measured: vec![],
            phase_omega: (p16 / 2) as u8,
            residual_zeta16: (p16 % 2) as u8,
        };
        let oracle = oracle_run(n, &gs);
        for basis in 0..1u32 << n {
            let y: Vec<bool> = (0..n).map(|q| basis >> q & 1 == 1).collect();
            let ev = engine_value(&prog, &y);
            assert!(
                ev.eq16(oracle.v[basis as usize]),
                "trial {trial} basis {basis}: pipeline diverges from oracle"
            );
        }
    }
}

/// The text front door: quizx-style and tracker-style constructs parse and
/// lower; the tracker's non-π/4 rz REFUSES with the named routes.
#[test]
fn text_surface_accepts_and_refuses_as_declared() {
    let ok = "OPENQASM 2.0;\ninclude \"qelib1.inc\";\nqreg q[3];\nh q[0];\ncz q[0],q[1];\nccz q[0],q[1],q[2];\nsx q[2];\nrz(pi/4) q[1];\nmeasure q[0] -> c[0];\n";
    let p = parse(ok).expect("superset must parse");
    assert_eq!(p.n_qubits, 3);
    assert_eq!(p.measured, vec![0]);
    assert_eq!(p.residual_zeta16, 1, "one odd rz leaves the declared residual");
    // The tracker's face angle is now RECOGNIZED exactly (face.rs's ring),
    // and core consumers refuse it by design, naming the face engine.
    let face = "qreg q[1];\nrz(0.9553166181245092) q[0];\n";
    let (_, surf, _) = holon::qasm::parse_surface(face).expect("face angle must parse");
    assert!(matches!(surf[0], Surface::Face(1, 0)), "face angle must become a Face gate");
    let e = parse(face).unwrap_err();
    assert!(e.reason.contains("face engine"), "core refusal must name the face engine");
    // A genuinely unrepresentable angle still refuses at the surface, with
    // its named routes.
    let bad = "qreg q[1];\nrz(0.3) q[0];\n";
    let e2 = parse(bad).unwrap_err();
    assert!(e2.reason.contains("CAMPAIGNS.md #2"), "refusal must name the route");
}
