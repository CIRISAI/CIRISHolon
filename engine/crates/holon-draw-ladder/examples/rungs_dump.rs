//! Print the standard ladder's rungs, one per line, tab separated: name, unit (log10 m), status
//! label, the rung's band of z (lo, hi), whether its boundary with the finer and the coarser
//! rung is a cut, the kinds of mark it emits, and its source line.
//! `standard/tools/holonc.py --ladder` reads this and refuses a registry whose declarations
//! disagree with the engine.
//!
//! The marks are the union over nine zooms spread evenly across the rung's band (ends included,
//! a hundredth of a decade in from each), at clock 0 and full weight: what the scene's `tiles`
//! returns there. A kind a scene emits only between those zooms would be missed.
//!
//!     cargo run -q -p holon-draw-ladder --example rungs_dump
use holon_draw::ladder::Band;
use holon_draw_ladder::Ladder;
use std::collections::BTreeSet;

fn main() {
    // an empty band: no host, the ladder's own rungs end to end
    let mut l = Ladder::standard(Band { z_lo: 0.0, z_hi: 0.0, clamp: false });
    let rungs = l.rungs.clone();
    for (s, r) in l.scenes.iter_mut().zip(&rungs) {
        let mut kinds = BTreeSet::new();
        for k in 0..9 {
            let z = r.z_lo + 0.01 + (r.z_hi - r.z_lo - 0.02) * k as f64 / 8.0;
            for t in s.tiles(z, 0.0, 1.0) {
                for m in &t.marks {
                    kinds.insert(format!("{:?}", m.kind).to_lowercase());
                }
            }
        }
        println!(
            "{}\t{}\t{}\t{:.3}\t{:.3}\t{}\t{}\t{}\t{}",
            s.name(),
            s.unit(),
            s.status().label(),
            r.z_lo,
            r.z_hi,
            s.cut_below(),
            s.cut_above(),
            kinds.into_iter().collect::<Vec<_>>().join(","),
            s.source().replace(['\t', '\n'], " ")
        );
    }
}
