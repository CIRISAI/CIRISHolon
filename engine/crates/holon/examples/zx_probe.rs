//! Diagnostic for the native ZX pass: spider count and T-count at each stage,
//! plus the gadget census that measured the old pass's defect as a flat zero.
//!
//! Read it as a ladder. Clifford simplification collapses the SPIDERS (the
//! published 5–6×) and barely touches T; `gen_pivot` turns the surviving
//! non-Pauli phases into GADGETS; fusion is what finally moves the T-count.
use holon::zx::from_surface;

const BASE: &str = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/quizx";

fn main() {
    let mut files: Vec<_> = std::fs::read_dir(BASE)
        .unwrap()
        .filter_map(|e| {
            let p = e.unwrap().path();
            let n = p.file_name()?.to_string_lossy().to_string();
            if n.starts_with("rand_q") && n.ends_with(".qasm") {
                Some((n, p))
            } else {
                None
            }
        })
        .collect();
    files.sort();
    for (name, path) in files {
        let src = std::fs::read_to_string(&path).unwrap();
        let (n, surf, _) = holon::qasm::parse_surface(&src).unwrap();
        let zero = vec![false; n];

        let mut g = from_surface(n, &surf).unwrap();
        g.plug_inputs(&zero);
        g.plug_outputs(&zero);
        let (s0, t0) = (g.n_spiders(), g.t_count());
        g.clifford_simp();
        let (s1, t1, gad1) = (g.n_spiders(), g.t_count(), g.gadget_stats());
        g.full_reduce();
        let (s2, t2, gad2) = (g.n_spiders(), g.t_count(), g.gadget_stats());
        println!(
            "{name:<26} spiders {s0:6} → {s1:5} → {s2:5}   T {t0:4} → {t1:4} → {t2:4}   \
             gadgets(all/shared) {}/{} → {}/{}",
            gad1.0, gad1.1, gad2.0, gad2.1
        );
    }
}
