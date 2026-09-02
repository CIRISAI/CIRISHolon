//! One (O, H, H) table node's solve, with its exit reason, residual, iteration count and
//! variational margin printed — the diagnostic for a node whose committed value moved
//! between two arithmetic regimes by more than rounding can explain.
//!
//!   cargo run --release -p holon-chem --example s2_node_probe -- i j k [i j k ...]
//!
//! The geometry is the table's own (`water::node_r`, `water::node_c`, `u = 1 − c²`) and the
//! solve is the one `water::ohh_energy` performs, so what prints is what the table build saw.
use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::solve_geometry;
use holon_chem::water::{node_c, node_r};

fn main() {
    let a: Vec<usize> = std::env::args().skip(1).map(|s| s.parse().expect("node index")).collect();
    assert!(!a.is_empty() && a.len() % 3 == 0, "triples i j k");
    for t in a.chunks(3) {
        let (x, y, c) = (node_r(t[0]), node_r(t[1]), node_c(t[2]));
        let u = 1.0 - c * c;
        let sn = (1.0 - u * u).max(0.0).sqrt();
        let sol = solve_geometry(
            &[OXYGEN, HYDROGEN, HYDROGEN],
            vec![[D2::c(0.0), D2::c(0.0), D2::c(0.0)], [D2::c(x), D2::c(0.0), D2::c(0.0)], [D2::c(y * u), D2::c(y * sn), D2::c(0.0)]],
        );
        println!(
            "node ({},{},{}) x={x:.4} y={y:.4} u={u:+.5}  E={:.12}  exit={}  residual={:.3e}  iters={}  n_det={}  route={}  device={}",
            t[0], t[1], t[2], sol.e.v, sol.exit.label(), sol.residual, sol.davidson_iters, sol.n_det, sol.route.label(), sol.device
        );
    }
}
