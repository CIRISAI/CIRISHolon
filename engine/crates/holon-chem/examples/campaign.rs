//! The ELEMENTS-1 staked pairs: curves, well depths, and the E1/E2 branches.
use holon_chem::elements::*;
use holon_chem::pair::{generate_pair_table, PairTable, WELL_MIN_DEPTH};

fn main() {
    let knots: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(64);
    let staked: Vec<(&str, Species, Species)> = vec![
        ("H2",  HYDROGEN, HYDROGEN),
        ("LiH", LITHIUM,  HYDROGEN),
        ("Li2", LITHIUM,  LITHIUM),
        ("HF",  HYDROGEN, FLUORINE),
        ("N2",  NITROGEN, NITROGEN),
        ("F2",  FLUORINE, FLUORINE),
        ("CO",  CARBON,   OXYGEN),
        ("He2", HELIUM,   HELIUM),
        ("Ne2", NEON,     NEON),
    ];
    // Default under `target/`, which is gitignored, NOT the working directory. Defaulting
    // to "." put nine untracked result files in the repository root the first time
    // somebody ran this without the variable set, where they sat looking like committed
    // artifacts and carried a superseded basis. An example that writes results should not
    // be able to litter the tree by being run the obvious way.
    let outdir = std::env::var("ELEMENTS1_OUT").unwrap_or_else(|_| "target/elements1".into());
    std::fs::create_dir_all(&outdir).expect("output directory");
    println!("knots per curve: {knots}\n");
    println!("{:>5} {:>6} {:>6} {:>7} {:>13} {:>10} {:>10} {:>9} {:>9} {:>9}",
             "pair","nbas","ndet","bound","E_asym (Ha)","R_e (a0)","D_e (Ha)","D_e (eV)","k_e","gen (s)");
    let mut rows = vec![];
    for (label, a, b) in staked {
        eprintln!("  [{label}] starting...");
        let t = generate_pair_table(a, b, knots);
        let m = &t.meta;
        let (bound, re, de, ke) = match m.well {
            Some(w) => ("yes", w.r_e, w.d_e, w.k_e),
            None => ("NO", f64::NAN, f64::NAN, f64::NAN),
        };
        println!("{:>5} {:>6} {:>6} {:>7} {:>13.6} {:>10.5} {:>10.6} {:>9.4} {:>9.5} {:>9.2}",
                 label, m.n_basis, m.n_det, bound, m.e_asymptote, re, de, de*27.211386245988, ke,
                 m.generation_ms/1e3);
        let path = format!("{outdir}/{label}_sto3g_fci.json");
        std::fs::write(&path, t.to_json()).unwrap();
        rows.push((label.to_string(), t));
    }
    println!("\n--- E1: the emergent negatives (staked: no well deeper than {WELL_MIN_DEPTH:.0e} Ha) ---");
    for (label, t) in rows.iter().filter(|(l,_)| l=="He2" || l=="Ne2") {
        let deepest = t.e.iter().cloned().fold(f64::INFINITY, f64::min);
        let depth = t.meta.e_asymptote - deepest;
        println!("  {label}: deepest point on grid is {:+.3e} Ha relative to asymptote \
                 (a well would be positive); well = {:?}  => branch {}",
                 depth, t.meta.well.map(|w| w.d_e),
                 if t.meta.well.is_none() && depth <= WELL_MIN_DEPTH { "(a) HOLDS" } else { "(b) FIRED" });
    }
    println!("\n--- E2: the emergent periodic pattern (staked order N2 > CO > HF > Li2 ~ LiH > F2 >> He2, Ne2) ---");
    let mut ord: Vec<(&str, f64)> = rows.iter()
        .map(|(l, t)| (l.as_str(), t.meta.well.map(|w| w.d_e).unwrap_or(0.0))).collect();
    ord.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    println!("  measured order: {}", ord.iter()
        .map(|(l,d)| format!("{l} ({:.4} Ha)", d)).collect::<Vec<_>>().join(" > "));
    let _ : &PairTable = &rows[0].1;
}
