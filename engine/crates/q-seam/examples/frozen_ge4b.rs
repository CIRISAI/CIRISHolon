// Which configurations would VOID under the FROZEN G-E4b (raw dense eigenvalue vs Lanczos <= 1e-14)?
use q_seam::dense::jacobi;
use q_seam::hubbard::Hubbard;
use q_seam::lanczos::ground_state;
fn main() {
    let mut voided = vec![];
    for &n in &[2usize, 4, 6] {
        let mut worst = 0.0f64;
        for &u in &q_seam::SWEEP_U {
            let h = Hubbard::new(n, 1.0, u);
            let eig = jacobi(h.to_dense(), h.dim());
            let g = ground_state(&h).unwrap();
            let raw = (eig.values[0] - g.energy).abs() / g.energy.abs().max(1.0);
            if raw > 1e-14 { voided.push((n, u, raw)); }
            worst = worst.max(raw);
        }
        println!("N={n}: worst raw dense-vs-Lanczos over all 14 U = {worst:.3e}  -> {}",
            if worst > 1e-14 { "WOULD VOID under frozen G-E4b" } else { "passes frozen G-E4b" });
    }
    println!("\ntotal configurations voided under the frozen reading: {}", voided.len());
    let ns: std::collections::BTreeSet<usize> = voided.iter().map(|v| v.0).collect();
    println!("affected N: {ns:?}   (N=8,10 are not covered by G-E4b at all: no dense cross-check)");
}
