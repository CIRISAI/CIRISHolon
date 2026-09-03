//! Referee one (O,H,H) node against the double-double tier: `water_node_referee <flat index>`.
//!
//! The f64 Davidson converges to a residual, not to a value, and inside a near-degenerate
//! manifold two legitimate f64 answers can differ by far more than the residual. When a
//! solver change moves a committed table, the question is not "did the bytes move" — they
//! did — but WHICH ANSWER IS CLOSER TO THE TRUTH. This prices that: the trimer's energy at
//! one node under the current restart policy, and the same solve refined at the DD tier
//! (`tier::refine_determinant_dd`) from each f64 answer as its start.
//!
//! Run the same index under `HOLON_DAVIDSON_KEEP=1` (which reproduces the pre-2026-09-02
//! restart exactly) to get the other arm.
use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::fci::{ci_ints, Order};
use holon_chem::pair::geometry_problem;
use holon_chem::water::{hh_side, node_r, node_u, NR, NU};

fn main() {
    let idx: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    // the packed order the producer writes: i in 0..NR, j in i..NR, k in 0..NU
    let (mut i, mut j, mut k) = (0usize, 0usize, 0usize);
    let mut at = 0usize;
    'outer: for ii in 0..NR {
        for jj in ii..NR {
            for kk in 0..NU {
                if at == idx {
                    i = ii;
                    j = jj;
                    k = kk;
                    break 'outer;
                }
                at += 1;
            }
        }
    }
    // `node_u(k)` IS the cosine the table's axis carries; `hh_side` and `ohh_energy` both
    // take it directly (water.rs), so nothing is squared here.
    let (x, y, u) = (node_r(i), node_r(j), node_u(k));
    let z = hh_side(x, y, u);
    println!("node {idx} = (i {i}, j {j}, k {k}) -> O-H {x:.6} and {y:.6} bohr, H-H {z:.6} bohr");

    // the trimer at this geometry: O at the origin, H at x along z, H at y in the x-z plane
    // the producer's own frame (water::ohh_energy): O at the origin, H at x along the
    // first axis, H at (y*u, y*sqrt(1-u^2), 0)
    let sn = (1.0 - u * u).max(0.0).sqrt();
    let (space, mo, nuc) = geometry_problem(
        &[OXYGEN, HYDROGEN, HYDROGEN],
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(x), D2::c(0.0), D2::c(0.0)],
            [D2::c(y * u), D2::c(y * sn), D2::c(0.0)],
        ],
    );
    // and the table's own quantity at this node, through the producer's path
    let d3 = holon_chem::water::de3(x, y, u);
    println!("dE3  keep={:>7}  {d3:+.17e} Ha", std::env::var("HOLON_DAVIDSON_KEEP").unwrap_or_else(|_| "default".into()));

    let ci = ci_ints(&mo, Order::Value);
    let diag = space.diagonal(&ci);
    let sol = holon_chem::fci::solve_determinant(&space, &mo);
    let e_f64 = sol.e.v + nuc.v;
    let keep = std::env::var("HOLON_DAVIDSON_KEEP").unwrap_or_else(|_| "default".into());
    println!(
        "f64  keep={keep:>7}  E {:.17e}  ({} davidson iterations, residual {:.2e}, {:?})",
        e_f64, sol.davidson_iters, sol.residual, sol.exit
    );

    // the referee: the same solve refined at the double-double tier from that f64 vector
    let r = holon_chem::tier::refine_determinant_dd(&space, &ci, &diag, sol.e.v, &sol.vector, 1e-12, 4000);
    println!(
        "dd   from that vector  E {:.17e}  (moved {:.3e} from the f64 answer, {} iterations, \
         residual {:.2e}, {:?})",
        r.e_f64 + nuc.v,
        r.delta_vs_f64,
        r.iters,
        r.residual,
        r.exit
    );
}
