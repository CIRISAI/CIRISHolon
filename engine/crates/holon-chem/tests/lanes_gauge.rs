//! The lane solver's gates: (1) at `k = 2` it IS the alpha/beta string solver, on an integral
//! set it has never seen; (2) on QCD₂ it reproduces the two-string determinant route and the
//! frozen referees; (3) shards are bit-identical to one shard; (4) the freeze's plants transfer.

use holon_chem::dual::D2;
use holon_chem::fci::{ci_ints, sigma_direct, FciSpace, MoIntegrals, Order, SolveExit};
use holon_chem::lanes::{LaneHamiltonian, LaneSigma, LaneSpace};
use holon_chem::qcd2::{Mutation, Qcd2};
use holon_chem::sigma_op::SigmaOp;

/// The frozen FCI referees at x = 4, N = 4 (GF2A prereg, plant (i)).
const REFEREE: [(i32, f64); 2] = [(0, -24.5391166860), (1, -17.5847761876)];

fn lcg(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64) - 0.5
}

/// A random real integral set with the eight-fold symmetry of `(pq|rs)` and a symmetric `h`.
fn random_integrals(n: usize, seed: u64) -> MoIntegrals {
    let mut st = seed;
    let mut h = vec![0.0; n * n];
    for p in 0..n {
        for q in p..n {
            let v = lcg(&mut st);
            h[p * n + q] = v;
            h[q * n + p] = v;
        }
    }
    let n2 = n * n;
    let raw: Vec<f64> = (0..n2 * n2).map(|_| lcg(&mut st)).collect();
    let at = |p: usize, q: usize, r: usize, s: usize| raw[(p * n + q) * n2 + (r * n + s)];
    let mut g = vec![0.0; n2 * n2];
    for p in 0..n {
        for q in 0..n {
            for r in 0..n {
                for s in 0..n {
                    g[(p * n + q) * n2 + (r * n + s)] = (at(p, q, r, s)
                        + at(q, p, r, s)
                        + at(p, q, s, r)
                        + at(q, p, s, r)
                        + at(r, s, p, q)
                        + at(s, r, p, q)
                        + at(r, s, q, p)
                        + at(s, r, q, p))
                        / 8.0;
                }
            }
        }
    }
    MoIntegrals { n, h: h.into_iter().map(D2::c).collect(), g: g.into_iter().map(D2::c).collect() }
}

fn random_vector(n: usize, seed: u64) -> Vec<f64> {
    let mut st = seed;
    (0..n).map(|_| lcg(&mut st)).collect()
}

fn max_abs_diff(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max)
}

#[test]
fn two_lanes_are_the_string_solver_on_a_random_integral_set() {
    for (n, na, nb, seed) in [(6usize, 3usize, 2usize, 11u64), (5, 2, 2, 12), (6, 4, 1, 13)] {
        let mo = random_integrals(n, seed);
        let ci = ci_ints(&mo, Order::Value);
        let space = FciSpace::new(n, na, nb);
        let c = random_vector(space.n_det, seed + 100);
        let mut s_ref = vec![0.0; space.n_det];
        sigma_direct(&space, &ci, &c, &mut s_ref);

        let lanes = LaneSpace::uniform(n, &[na, nb]);
        assert_eq!(lanes.n_det, space.n_det);
        let ham = LaneHamiltonian::from_ci_ints(&ci, 2);
        let mut op = LaneSigma::new(&lanes, &ham, 1);
        let mut s_lane = vec![0.0; space.n_det];
        op.apply(&c, &mut s_lane);
        let scale = s_ref.iter().map(|v| v.abs()).fold(0.0, f64::max);
        let d = max_abs_diff(&s_ref, &s_lane);
        assert!(d <= 1e-12 * scale, "n={n} ({na},{nb}): lane sigma vs sigma_direct differ by {d:e} (scale {scale:e})");

        let d_ref = space.diagonal(&ci);
        let d_lane = op.diagonal();
        let dd = max_abs_diff(&d_ref, &d_lane);
        assert!(dd <= 1e-12 * scale, "n={n} ({na},{nb}): lane diagonal vs FciSpace::diagonal differ by {dd:e}");
    }
}

#[test]
fn colour_lanes_reproduce_the_two_string_route_and_the_referees_at_four_sites() {
    let q = Qcd2::new(4, 4.0);
    for (b, e_ref) in REFEREE {
        let two = q.ground(b).e.v;
        let lane = q.ground_lanes(b, 2);
        assert_eq!(lane.exit, SolveExit::Converged, "B={b}: {:?}", lane.exit);
        assert!((lane.energy - two).abs() <= 1e-9, "B={b}: lanes {:.12} vs two-string {two:.12}", lane.energy);
        assert!((lane.energy - e_ref).abs() <= 1e-8, "B={b}: lanes {:.10} vs referee {e_ref:.10}", lane.energy);
        assert!(lane.variational_margin >= 0.0, "B={b}: the Ritz value sits above a diagonal element");
    }
}

#[test]
fn colour_lanes_reproduce_the_two_string_route_at_six_sites() {
    // B = 2 at N = 6 is 816 determinants on the two-string route and 6³ = 216 on the lanes; the
    // two solvers share nothing but the operator.
    for x in [4.0, 9.0] {
        let q = Qcd2::new(6, x);
        let two = q.ground(2).e.v;
        let lane = q.ground_lanes(2, 3).energy;
        assert!((lane - two).abs() <= 1e-9, "x={x}: lanes {lane:.12} vs two-string {two:.12}");
    }
}

#[test]
fn shards_are_bit_identical_to_one_shard() {
    let q = Qcd2::new(6, 9.0);
    let space = q.lane_space(0);
    assert_eq!(space.n_det, 8000);
    let ham = q.lane_hamiltonian();
    let c = random_vector(space.n_det, 7);
    let mut one = vec![0.0; space.n_det];
    // one operator, its shard count varied: the tables do not depend on it
    let mut op = LaneSigma::new(&space, &ham, 1);
    op.apply(&c, &mut one);
    for threads in [2usize, 4, 7, 32] {
        op.threads = threads;
        let mut many = vec![0.0; space.n_det];
        op.apply(&c, &mut many);
        let mismatched = one.iter().zip(&many).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
        assert_eq!(mismatched, 0, "{threads} shards differ from one shard on {mismatched} entries");
    }
    let e1 = q.ground_lanes(0, 1).energy;
    let e4 = q.ground_lanes(0, 4).energy;
    assert_eq!(e1.to_bits(), e4.to_bits(), "Davidson on 1 vs 4 shards: {e1:.15} vs {e4:.15}");
}

#[test]
fn four_lanes_with_two_empty_are_two_lanes_to_the_bit() {
    // The k ≥ 4 decode path, exercised on both the sigma and the diagonal with an exact
    // referee: an empty lane has one string and no singles, so a four-lane space whose lanes
    // 2 and 3 are empty is the two-lane space with the same index order, term for term.
    let (n, na, nb) = (6usize, 3usize, 2usize);
    let mo = random_integrals(n, 21);
    let ci = ci_ints(&mo, Order::Value);
    let two = LaneSpace::uniform(n, &[na, nb]);
    let four = LaneSpace::uniform(n, &[na, nb, 0, 0]);
    assert_eq!(four.n_det, two.n_det);
    assert_eq!(four.n_lanes(), 4);
    let c = random_vector(two.n_det, 22);
    let (mut s2, mut s4) = (vec![0.0; two.n_det], vec![0.0; two.n_det]);
    let mut op2 = LaneSigma::new(&two, &LaneHamiltonian::from_ci_ints(&ci, 2), 3);
    let mut op4 = LaneSigma::new(&four, &LaneHamiltonian::from_ci_ints(&ci, 4), 3);
    op2.apply(&c, &mut s2);
    op4.apply(&c, &mut s4);
    let mism = s2.iter().zip(&s4).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
    assert_eq!(mism, 0, "four lanes (two empty) differ from two lanes on {mism} entries");
    let (d2, d4) = (op2.diagonal(), op4.diagonal());
    let mism = d2.iter().zip(&d4).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
    assert_eq!(mism, 0, "diagonals differ on {mism} entries");
}

#[test]
fn the_plants_transfer_to_the_lanes() {
    // (iii) x = 0: three quarks on one site are a singlet and cost nothing — EXACT
    let q0 = Qcd2::new(4, 0.0);
    let (e0, e1) = (q0.ground_lanes(0, 2).energy, q0.ground_lanes(1, 2).energy);
    assert!((e1 - e0).abs() <= 1e-12, "x=0: E1-E0 = {:e}", e1 - e0);
    // (ii) the Fierz trace mutation moves the baryon mass, and both routes agree on the mutant
    let q = Qcd2::new(4, 4.0);
    let m = q.ground_lanes(1, 2).energy - q.ground_lanes(0, 2).energy;
    let qm = Qcd2::new(4, 4.0).with_mutation(Mutation::FierzTraceOff);
    let mm = qm.ground_lanes(1, 2).energy - qm.ground_lanes(0, 2).energy;
    assert!((m - mm).abs() > 1e-3, "the planted Fierz defect did not move E1-E0 on the lanes: {m} vs {mm}");
    let two_m = qm.ground(1).e.v - qm.ground(0).e.v;
    assert!((mm - two_m).abs() <= 1e-9, "mutant: lanes {mm:.12} vs two-string {two_m:.12}");
}
