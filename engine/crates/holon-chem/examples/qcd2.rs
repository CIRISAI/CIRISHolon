//! GF2a on the determinant route: sector energies of massless one-flavour QCD₂, the
//! derived baryon mass and the finite-volume two-baryon energy, with the door's price.
//!
//!   cargo run --release -p holon-chem --example qcd2 -- N X [N X ...]
use holon_chem::budget::price_determinant;
use holon_chem::qcd2::Qcd2;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    assert!(!args.is_empty() && args.len() % 2 == 0, "pairs of N X");
    println!("# GF2a FCI: massless QCD2, axial gauge; sectors B=0,1,2; W-units; M_B/g = (E1-E0)/(2 sqrt x)");
    for pair in args.chunks(2) {
        let n: usize = pair[0].parse().expect("N");
        let x: f64 = pair[1].parse().expect("x");
        let q = Qcd2::new(n, x);
        let mut es = Vec::new();
        for b in 0..=2 {
            let space = q.space(b);
            let price = price_determinant(space.n_det);
            let t0 = Instant::now();
            let sol = q.ground(b);
            es.push(sol.e.v);
            println!(
                "N={n:2} x={x:4.1} B={b} n_q={:2} n_det={:9} price={:.2e}B  E0={:+.10}  iters={} resid={:.1e}  {:.1}s",
                q.quarks(b), space.n_det, price.bytes as f64, sol.e.v, sol.davidson_iters, sol.residual,
                t0.elapsed().as_secs_f64()
            );
        }
        let m_b = Qcd2::baryon_mass(es[0], es[1], x);
        let u_bb = es[2] - 2.0 * es[1] + es[0];
        println!("N={n:2} x={x:4.1}  M_B/g={m_b:.6}  U_BB={u_bb:+.6e}  (E1-E0={:+.6}, E2-E1={:+.6})", es[1] - es[0], es[2] - es[1]);
    }
}
