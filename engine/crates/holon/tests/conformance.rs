//! The holon crate's tiers certified against the QASM-suite reference tiers
//! (themselves externally refereed: QASM-1 seven of seven, QASM-2 five of
//! five). Plus the packed-layout mutation observability and the recursion
//! smoke.

use holon::tableau::PackedTableau;
use holon_qasm::{run_tableau, Circuit, Gate, Mutation, Tableau as RefTableau};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn apply_packed(t: &mut PackedTableau, g: &Gate) {
    match *g {
        Gate::X(q) => t.x_gate(q),
        Gate::Z(q) => t.z_gate(q),
        Gate::H(q) => t.h(q),
        Gate::S(q) => t.s(q),
        Gate::Sdg(q) => t.sdg(q),
        Gate::Cx(c, tt) => t.cx(c, tt),
        _ => panic!("clifford only"),
    }
}

fn rand_clifford(rng: &mut Rng, n: usize, depth: usize) -> Vec<Gate> {
    let mut gates = Vec::new();
    for _ in 0..depth {
        let q = rng.below(n);
        let mut q2 = rng.below(n);
        while q2 == q {
            q2 = rng.below(n);
        }
        gates.push(match rng.below(6) {
            0 => Gate::X(q),
            1 => Gate::Z(q),
            2 => Gate::H(q),
            3 => Gate::S(q),
            4 => Gate::Sdg(q),
            _ => Gate::Cx(q, q2),
        });
    }
    gates
}

/// Packed vs certified-unpacked: identical measurement structure — same
/// deterministic outcomes, same random-measurement pattern, on sequential
/// measurement of every qubit with random outcomes collapsed to `false`.
#[test]
fn packed_tableau_matches_certified_reference() {
    let mut rng = Rng(11);
    for _ in 0..60 {
        let n = 2 + rng.below(6);
        let depth = 5 + rng.below(60);
        let gates = rand_clifford(&mut rng, n, depth);
        let mut packed = PackedTableau::new(n);
        let mut reference = RefTableau::new(n, Mutation::None);
        for g in &gates {
            apply_packed(&mut packed, g);
            reference.apply(*g);
        }
        for q in 0..n {
            let a = packed.measure_peek(q);
            let b = reference.measure_peek(q);
            assert_eq!(a, b, "peek diverged at qubit {q}");
            if a.is_none() {
                packed.collapse(q, false);
                reference.collapse(q, false);
            }
        }
    }
}

/// Distribution-level conformance through the reference's own machinery:
/// full-circuit distributions must match the certified tier exactly.
#[test]
fn packed_distributions_match() {
    let mut rng = Rng(23);
    for _ in 0..30 {
        let n = 2 + rng.below(4);
        let depth = 5 + rng.below(40);
        let gates = rand_clifford(&mut rng, n, depth);
        let c = Circuit {
            n_qubits: n,
            n_clbits: n,
            gates: gates.clone(),
            measures: (0..n).map(|q| (q, q)).collect(),
        };
        let ref_dist = run_tableau(&c, Mutation::None);
        // Packed distribution by branching on random outcomes.
        let mut stack = vec![(
            {
                let mut t = PackedTableau::new(n);
                for g in &gates {
                    apply_packed(&mut t, g);
                }
                t
            },
            0usize,
            vec![false; n],
            1.0f64,
        )];
        let mut dist = std::collections::BTreeMap::new();
        while let Some((t, qi, bits, w)) = stack.pop() {
            if qi == n {
                let key: String =
                    (0..n).rev().map(|i| if bits[i] { '1' } else { '0' }).collect();
                *dist.entry(key).or_insert(0.0) += w;
                continue;
            }
            match t.measure_peek(qi) {
                Some(o) => {
                    let mut b = bits;
                    b[qi] = o;
                    stack.push((t, qi + 1, b, w));
                }
                None => {
                    for outcome in [false, true] {
                        let mut t2 = PackedTableau {
                            n: t.n,
                            rows: t.rows.clone(),
                        };
                        t2.collapse(qi, outcome);
                        let mut b = bits.clone();
                        b[qi] = outcome;
                        stack.push((t2, qi + 1, b, w * 0.5));
                    }
                }
            }
        }
        let keys: std::collections::BTreeSet<_> = ref_dist.keys().chain(dist.keys()).collect();
        for k in keys {
            let e = (ref_dist.get(k).unwrap_or(&0.0) - dist.get(k).unwrap_or(&0.0)).abs();
            assert!(e < 1e-12, "distribution diverged at {k}: {e}");
        }
    }
}

/// The recursion smoke: a coarse holon reads a Closed view of its children.
#[test]
fn recursion_reads_children() {
    use holon::{Arena, Certificate, ClassicalHolon, CoarseHolon};
    let mut arena = Arena::new();
    for i in 0..8 {
        let mut c = ClassicalHolon::new(4);
        if i % 3 == 0 {
            c.x(0);
        }
        let id = arena.push(c);
        assert_eq!(id, i, "identity must be the arena index");
    }
    let coarse = CoarseHolon {
        children: arena,
        cert: Certificate::exact("parity-per-child", "child-classical", "parity-update"),
    };
    let p = coarse.read();
    for i in 0..8 {
        assert_eq!(p.get(i), i % 3 == 0);
    }
}

/// Tier-2 ledger viability: the holon crate's Z[ω] ring must be isomorphic in
/// action to the certified magic tier's (QASM-2 five of five) — products,
/// sums, and complex values agree on random ring elements.
#[test]
fn cyc_ledger_matches_certified_ring() {
    use holon::ledger::Cyc as HCyc;
    use holon_qasm::magic::Cyc as QCyc;
    let mut rng = Rng(31);
    for _ in 0..500 {
        let mk = |rng: &mut Rng| {
            let c = [
                rng.below(9) as i128 - 4,
                rng.below(9) as i128 - 4,
                rng.below(9) as i128 - 4,
                rng.below(9) as i128 - 4,
            ];
            let m = rng.below(7) as i32;
            (HCyc { c, m }, QCyc { c, m })
        };
        let (h1, q1) = mk(&mut rng);
        let (h2, q2) = mk(&mut rng);
        let hp = h1.mul(h2).to_complex();
        let qp = q1.mul(q2).to_complex();
        assert!((hp.0 - qp.0).abs() < 1e-9 && (hp.1 - qp.1).abs() < 1e-9);
        let hs = h1.add(h2).to_complex();
        let qs = q1.add_fixed(q2).to_complex();
        assert!((hs.0 - qs.0).abs() < 1e-9 && (hs.1 - qs.1).abs() < 1e-9);
    }
}

/// ℝ-plane viability: conditioning is computed, and the season's measured
/// pattern reproduces in miniature — a signed near-cancelling aggregate is
/// ill-conditioned, an all-nonnegative one is perfectly conditioned.
#[test]
fn real_holon_conditioning_is_measured() {
    use holon::real::{RealHolon, RealPlane};
    let momx = RealPlane { lanes: vec![1.0, -0.99, 0.98, -1.01, 1.02, -1.0] };
    let ke = RealPlane { lanes: vec![0.5, 0.49, 0.51, 0.5, 0.52, 0.48] };
    let h = RealHolon::new(vec![("momx".into(), momx), ("ke".into(), ke)]);
    let cx = h.conditioning.iter().find(|(n, _)| n == "momx").unwrap().1;
    let ck = h.conditioning.iter().find(|(n, _)| n == "ke").unwrap().1;
    assert!(cx < 0.01, "signed near-cancelling aggregate must read ill-conditioned: {cx}");
    assert!((ck - 1.0).abs() < 1e-12, "all-nonnegative chart must read coherence 1: {ck}");
}

/// Bulk-tier viability: the MPS shape holds states and ops on the same
/// object family; amplitudes exact on the minimal instance.
#[test]
fn mps_holon_minimal() {
    use holon::real::MpsHolon;
    let mut m = MpsHolon::product_state(&[false, true, false, true]);
    m.apply_x(0);
    assert_eq!(m.amplitude(&[true, true, false, true]), 1.0);
    assert_eq!(m.amplitude(&[false, true, false, true]), 0.0);
}
