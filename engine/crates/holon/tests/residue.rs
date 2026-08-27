//! The residue carrier against the direct ring, on the LIVE pipeline:
//! wherever the i128 ring exists, the child-holon carrier must agree with it
//! exactly — same lanes, same value, every shard count. And past the ring's
//! envelope the carrier must keep going. Not a workaround: the recursion IS
//! the object, and this file is its conformance gate.

use holon::ledger::Cyc;
use holon::prune::Gate;
use holon::run::{amplitude, amplitude_unbounded, amplitude_auto, Amplitude};

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

fn rand_circuit(rng: &mut Rng, n: usize, depth: usize, t_cap: usize) -> Vec<Gate> {
    let mut g = Vec::new();
    let mut t = 0;
    for _ in 0..depth {
        let q = rng.below(n);
        let mut q2 = rng.below(n);
        while q2 == q {
            q2 = rng.below(n);
        }
        match rng.below(8) {
            0 => g.push(Gate::X(q)),
            1 => g.push(Gate::Z(q)),
            2 => g.push(Gate::H(q)),
            3 => g.push(Gate::S(q)),
            4 | 5 if t < t_cap => {
                t += 1;
                g.push(Gate::T(q));
            }
            _ => g.push(Gate::Cx(q, q2)),
        }
    }
    g
}

/// Lift a (normalized) direct-ring value to exponent `m` for comparison.
fn align_cyc_to(x: Cyc, m: i32) -> Cyc {
    assert!(m >= x.m);
    let mut c = x.c;
    let delta = (m - x.m) as u32;
    for v in &mut c {
        *v <<= delta / 2;
    }
    if delta % 2 == 1 {
        c = [c[1] - c[3], c[0] + c[2], c[1] + c[3], c[2] - c[0]];
    }
    Cyc { c, m }
}

#[test]
fn residue_matches_direct_on_live_circuits() {
    let mut rng = Rng(0xC1B1_5EED);
    for trial in 0..8 {
        let n = 3 + rng.below(3);
        let depth = 40 + rng.below(40);
        let gates = rand_circuit(&mut rng, n, depth, 8);
        let mut y = vec![false; n];
        for b in y.iter_mut() {
            if rng.below(2) == 1 {
                *b = true;
            }
        }
        let direct = amplitude(n, &gates, &y);
        for shards in [1usize, 5, 16] {
            let reading = amplitude_unbounded(n, &gates, &y, shards);
            let got = reading
                .to_cyc_checked()
                .expect("small circuit fits the direct ring");
            assert!(got.m >= direct.m, "trial {trial}: exponent went backwards");
            assert_eq!(
                got,
                align_cyc_to(direct, got.m),
                "trial {trial} shards {shards}: residue and direct rings disagree"
            );
        }
    }
}

#[test]
fn auto_router_picks_direct_on_small() {
    let mut rng = Rng(7);
    let gates = rand_circuit(&mut rng, 3, 30, 6);
    let y = vec![false; 3];
    match amplitude_auto(3, &gates, &y, 4) {
        Amplitude::Direct(_) => {}
        Amplitude::Residue(_) => panic!("router sent a small circuit to the residue carrier"),
    }
}
