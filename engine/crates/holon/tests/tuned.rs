//! The tuned path against the untuned truth: whatever route the policy
//! selects, the amplitude is the same exact value — routing is a speed
//! decision, never a semantics decision.

use holon::ledger::Cyc;
use holon::magic::cyc_eq;
use holon::prune::Gate;
use holon::run::{amplitude, amplitude_tuned};
use holon::tune::{Decomp, Degrade, Hold, Policy};

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

fn check_route(n: usize, depth: usize, t_cap: usize, seed: u64, expect: Decomp) {
    let mut rng = Rng(seed);
    let gates = rand_circuit(&mut rng, n, depth, t_cap);
    let mut y = vec![false; n];
    for b in y.iter_mut() {
        if rng.below(2) == 1 {
            *b = true;
        }
    }
    let truth: Cyc = amplitude(n, &gates, &y);
    let p = Policy::exact();
    for shards in [1usize, 4] {
        let (amp, choice) = amplitude_tuned(&p, n, &gates, &y, shards).unwrap();
        assert_eq!(choice.decomp, expect, "route for n={n} t_cap={t_cap}");
        assert!(cyc_eq(amp, truth), "tuned route {:?} disagrees with truth", choice.decomp);
    }
}

#[test]
fn every_route_agrees_with_truth() {
    // t <= n: sliced route.
    check_route(12, 60, 8, 0xA11CE, Decomp::Sliced);
    // t > n, t >= 5: magic5 route.
    check_route(4, 40, 9, 0xB0B, Decomp::Magic5);
    // t > n, t < 5: pruned default.
    check_route(2, 20, 3, 0xC0FFEE, Decomp::Pruned);
}

#[test]
fn scope_refusal_reaches_the_caller() {
    let mut rng = Rng(9);
    let gates = rand_circuit(&mut rng, 6, 60, 10);
    let y = vec![false; 6];
    let p = Policy::new(Hold::Exactness, vec![Degrade::Scope { max_t: 4 }]).unwrap();
    let r = amplitude_tuned(&p, 6, &gates, &y, 2).unwrap_err();
    assert!(r.reason.contains("max_t=4"));
}
