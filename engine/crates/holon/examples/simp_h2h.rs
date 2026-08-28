//! Simplifier head-to-head: OUR passes vs quizx's full_simp, T-count
//! reduction on identical random Clifford+T circuits.
use holon::phasepoly::normalized_t_count;
use holon::simplify::{magic_weight, simplify};
fn main() {
    let base = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/quizx";
    let mut files: Vec<_> = std::fs::read_dir(base).unwrap()
        .filter_map(|e| { let p = e.unwrap().path();
            let n = p.file_name()?.to_string_lossy().to_string();
            if n.starts_with("rand_q") && n.ends_with(".qasm") { Some((n, p)) } else { None } })
        .collect();
    files.sort();
    for (name, p) in files {
        let src = std::fs::read_to_string(&p).unwrap();
        match holon::qasm::parse_surface(&src) {
            Ok((n, surf, _)) => {
                let raw = magic_weight(&surf);
                let loc = simplify(&surf);
                let after_local = magic_weight(&loc);
                let after_pp = normalized_t_count(n, &loc);
                println!("{name:28} n={n:3} raw_t={raw:4} local={after_local:4} ours_final={after_pp:4}");
            }
            Err(e) => println!("{name:28} REFUSED: {}", e.reason),
        }
    }
}
