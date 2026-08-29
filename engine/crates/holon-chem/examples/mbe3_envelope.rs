//! SCRATCH: where does the trimer table's curvature envelope come from?
use holon_chem::trimer::{self, TrimerTable};

fn hess_rowsum(t: &TrimerTable, sides: [f64; 3], hh: f64) -> f64 {
    let mut worst = 0.0f64;
    for a in 0..3 {
        let mut lo = sides; let mut hi = sides;
        lo[a] -= hh; hi[a] += hh;
        let (_, glo) = t.eval(lo);
        let (_, ghi) = t.eval(hi);
        let row: f64 = (0..3).map(|b| ((ghi[b]-glo[b])/(2.0*hh)).abs()).sum();
        worst = worst.max(row);
    }
    worst
}

fn main() {
    let t = trimer::generate().unwrap();
    println!("envelopes as shipped: absolute {:.3} Ha/bohr^2, per-gradient {:.3} /bohr, sort kink {:.3e} Ha/bohr", t.curvature_envelope, t.curvature_per_gradient, t.sort_kink);
    for hh in [1e-4f64, 1e-3, 1e-2] {
        let mut worst = 0.0f64;
        let mut at = (0.0, 0.0, 0.0);
        let mut worst_in = 0.0f64;   // restricted to the staked domain x,y >= 0.9
        let mut at_in = (0.0, 0.0, 0.0);
        let mut worst_far = 0.0f64;  // and away from the s2 = s3 sort boundary
        let mut at_far = (0.0, 0.0, 0.0);
        for i in 0..(trimer::NR - 1) {
            for j in i..(trimer::NR - 1) {
                for k in 0..(trimer::NU - 1) {
                    let x = 0.5*(trimer::node_r(i)+trimer::node_r(i+1));
                    let y = 0.5*(trimer::node_r(j)+trimer::node_r(j+1));
                    let c = 0.5*(trimer::node_c(k)+trimer::node_c(k+1));
                    let u = 1.0 - c*c;
                    let z = (x*x + y*y - 2.0*x*y*u).max(0.0).sqrt();
                    let w = hess_rowsum(&t, [x,y,z], hh);
                    if w > worst { worst = w; at = (x,y,z); }
                    if x >= 0.9 && y >= 0.9 {
                        if w > worst_in { worst_in = w; at_in = (x,y,z); }
                        if (z - y).abs() > 20.0*hh && (y - x).abs() > 20.0*hh {
                            if w > worst_far { worst_far = w; at_far = (x,y,z); }
                        }
                    }
                }
            }
        }
        println!("hh = {hh:.0e}:  whole grid {worst:9.2} at {at:?}\n            staked domain {worst_in:9.2} at {at_in:?}\n            away from s2=s3 {worst_far:9.2} at {at_far:?}", );
    }

    // The RATIO the drift bound wants: how much stiffer than its own gradient can the
    // surface be? A local bound k <= B * max|g| makes the bound live and local instead of
    // pinned to a global maximum the trajectory never reaches.
    let hh = 1e-3f64;
    let mut worst_b = 0.0f64;
    let mut at_b = (0.0, 0.0, 0.0);
    let mut worst_h = 0.0f64;
    for i in 0..(trimer::NR - 1) {
        for j in i..(trimer::NR - 1) {
            for k in 0..(trimer::NU - 1) {
                let x = 0.5*(trimer::node_r(i)+trimer::node_r(i+1));
                let y = 0.5*(trimer::node_r(j)+trimer::node_r(j+1));
                let c = 0.5*(trimer::node_c(k)+trimer::node_c(k+1));
                let u = 1.0 - c*c;
                let z = (x*x + y*y - 2.0*x*y*u).max(0.0).sqrt();
                let mut sorted = [x, y, z];
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                // Stay away from the two sort boundaries, where the composed function has
                // a kink rather than a curvature.
                if (sorted[1]-sorted[0]).abs() < 40.0*hh || (sorted[2]-sorted[1]).abs() < 40.0*hh { continue; }
                if sorted[0] < 0.9 || sorted[1] > trimer::R_HI { continue; }
                let h = hess_rowsum(&t, [x,y,z], hh);
                worst_h = worst_h.max(h);
                let (_, g) = t.eval([x,y,z]);
                let gmax = g.iter().fold(0.0f64, |m, v| m.max(v.abs()));
                if gmax > 1e-9 {
                    let b = h / gmax;
                    if b > worst_b { worst_b = b; at_b = (x,y,z); }
                }
            }
        }
    }
    println!("smooth-branch, staked domain: max ||H||_rowsum = {worst_h:.3} Ha/bohr^2");
    println!("max ||H|| / max|g| = {worst_b:.3} /bohr at {at_b:?}");
}
