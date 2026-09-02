//! Is momentum-x reading momentum-y? An independent recomputation from direction counts.
use holon_lattice::{state::Model, Lattice};
fn main() {
    let m = Model::fhp6();
    for (l, seed) in [(256usize, 0xC1A5u64), (64, 0xC1A5), (128, 0xC1A5), (256, 0xBEEF)] {
        let g = Lattice::seeded(m.clone(), l, seed, 0.35, m.fhp_i(true));
        // route 1: the ledger under test
        let led = g.ledger();
        // route 2: per-direction counts, combined by hand from DIRECTIONS
        let mut nd = [0i64; 6];
        for &s in &g.cells {
            for d in 0..6 {
                if s >> d & 1 == 1 { nd[d] += 1; }
            }
        }
        let px: i64 = (0..6).map(|d| nd[d] * m.dirs[d][0]).sum();
        let py: i64 = (0..6).map(|d| nd[d] * m.dirs[d][1]).sum();
        let mass: i64 = nd.iter().sum();
        println!(
            "L={l:<4} seed={seed:#x}  ledger(mass {}, Px {}, Py {})  independent(mass {mass}, Px {px}, Py {py})  agree={}  counts {nd:?}",
            led.mass, led.momentum[0], led.momentum[1],
            led.mass == mass && led.momentum[0] == px && led.momentum[1] == py
        );
    }
}
