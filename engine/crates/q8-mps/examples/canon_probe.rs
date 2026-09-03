//! Is the labelled sweep's output right-canonical? Both variance routes assume the tail
//! tensors satisfy sum_{s,r} B[s,m,r] B[s,m',r] = delta_{m m'}.
use q8_mps::qcd2::Qcd2;
use q8_mps::symmetric::{random_start, SymConfig};
fn main() {
    for (n, b, chi) in [(4usize, 0i32, 16usize), (6, 2, 16)] {
        let q = Qcd2::new(n, 4.0);
        let n_q = q.quarks(b);
        let sector = q.sector(n_q).unwrap();
        let cfg = SymConfig::amendment(chi, 40);
        let (r, _) = q.ground_energy_sym_from(&[], n_q, &cfg, Some(random_start(&sector, 256, 7))).unwrap();
        let t = &r.tensors;
        let mut worst_r = 0.0f64; let mut worst_l = 0.0f64;
        let (mut jr, mut jl) = (0, 0);
        for (j, x) in t.iter().enumerate() {
            // right-canonical? B B^dag = I over (s,r)
            let mut d = 0.0f64;
            for m in 0..x.chi_l { for mp in 0..x.chi_l {
                let mut acc = 0.0;
                for s in 0..2 { for rr in 0..x.chi_r { acc += x.get(s, m, rr) * x.get(s, mp, rr); } }
                d = d.max((acc - if m == mp { 1.0 } else { 0.0 }).abs());
            }}
            if d > worst_r { worst_r = d; jr = j; }
            // left-canonical? A^dag A = I over (s,l)
            let mut e = 0.0f64;
            for rr in 0..x.chi_r { for rp in 0..x.chi_r {
                let mut acc = 0.0;
                for s in 0..2 { for m in 0..x.chi_l { acc += x.get(s, m, rr) * x.get(s, m, rp); } }
                e = e.max((acc - if rr == rp { 1.0 } else { 0.0 }).abs());
            }}
            if e > worst_l { worst_l = e; jl = j; }
        }
        println!("N={n} B={b} chi={chi}: worst right-canonical defect {worst_r:.3e} (site {jr}), worst left-canonical defect {worst_l:.3e} (site {jl}); norm^2 {:.9}", q8_mps::observables::norm_squared(t));
        // per-site, first four
        for (j, x) in t.iter().enumerate().take(4) {
            let mut d = 0.0f64;
            for m in 0..x.chi_l { for mp in 0..x.chi_l {
                let mut acc = 0.0;
                for s in 0..2 { for rr in 0..x.chi_r { acc += x.get(s, m, rr) * x.get(s, mp, rr); } }
                d = d.max((acc - if m == mp { 1.0 } else { 0.0 }).abs());
            }}
            println!("   site {j} ({}x{}): right defect {d:.3e}", x.chi_l, x.chi_r);
        }
    }
}
