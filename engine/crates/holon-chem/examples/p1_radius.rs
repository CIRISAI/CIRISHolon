//! P1's third radius rule, measured before it is trusted.
use holon_chem::elements::by_symbol;
use holon_chem::pair::{atomic_rms_radius, atomic_valence_rms_radius};

fn main() {
    println!("{:>3} {:>4} {:>14} {:>16}", "sym", "Z", "rms r (all)", "rms r (valence)");
    for sym in ["H", "He", "C", "Ne", "Na", "Ar", "K", "Se", "Kr", "Te", "Xe"] {
        let sp = by_symbol(sym).unwrap();
        println!("{:>3} {:>4} {:>14.6} {:>16.6}", sym, sp.z, atomic_rms_radius(sp), atomic_valence_rms_radius(sp));
    }
}
