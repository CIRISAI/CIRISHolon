//! The production path certified against the frozen referee: run::amplitude
//! (prune-dedup + mesh-fold defaults) must match holon-qasm's certified naive
//! magic tier exactly, across strata and shard counts.

use holon::ledger::Cyc;
use holon::prune::Gate as PGate;
use holon::run::{amplitude, amplitude_sharded};
use holon_qasm::magic::magic_amplitude;
use holon_qasm::{Circuit, Gate as QGate};

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

fn rand_pair(rng: &mut Rng, n: usize, depth: usize) -> (Vec<PGate>, Vec<QGate>) {
    let mut pg = Vec::new();
    let mut qg = Vec::new();
    let mut t_count = 0;
    for _ in 0..depth {
        let q = rng.below(n);
        let mut q2 = rng.below(n);
        while q2 == q {
            q2 = rng.below(n);
        }
        match rng.below(8) {
            0 => {
                pg.push(PGate::X(q));
                qg.push(QGate::X(q));
            }
            1 => {
                pg.push(PGate::Z(q));
                qg.push(QGate::Z(q));
            }
            2 => {
                pg.push(PGate::H(q));
                qg.push(QGate::H(q));
            }
            3 => {
                pg.push(PGate::S(q));
                qg.push(QGate::S(q));
            }
            4 | 5 if t_count < 10 => {
                t_count += 1;
                pg.push(PGate::T(q));
                qg.push(QGate::T(q));
            }
            _ => {
                pg.push(PGate::Cx(q, q2));
                qg.push(QGate::Cx(q, q2));
            }
        }
    }
    (pg, qg)
}

#[test]
fn production_path_matches_frozen_referee() {
    let mut rng = Rng(77);
    for _ in 0..40 {
        let n = 2 + rng.below(5);
        let depth = 4 + rng.below(24);
        let (pg, qg) = rand_pair(&mut rng, n, depth);
        let c = Circuit { n_qubits: n, n_clbits: n, gates: qg, measures: vec![] };
        for _ in 0..4 {
            let mut y = vec![false; n];
            for q in 0..n {
                y[q] = rng.below(2) == 1;
            }
            let ours: Cyc = amplitude(n, &pg, &y);
            let (or, oi) = ours.to_complex();
            let (rr, ri) = magic_amplitude(&c, &y, false, false);
            assert!(
                (or - rr).abs() < 1e-10 && (oi - ri).abs() < 1e-10,
                "production path diverged from referee: ({or},{oi}) vs ({rr},{ri})"
            );
            // shard-invariance at value scope, through the whole pipeline
            let s3 = amplitude_sharded(n, &pg, &y, 3).to_complex();
            assert!((s3.0 - or).abs() < 1e-12 && (s3.1 - oi).abs() < 1e-12);
        }
    }
}
