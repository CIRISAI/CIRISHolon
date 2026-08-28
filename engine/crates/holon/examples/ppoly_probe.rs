//! What does the non-local pass buy on the real head-to-head circuits?
use holon::phasepoly::normalized_t_count;
use holon::simplify::{magic_weight, simplify};
fn main() {
    let base = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/quizx";
    for entry in std::fs::read_dir(base).unwrap() {
        let p = entry.unwrap().path();
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        if !name.starts_with("h2h_hs_") || !name.ends_with(".qasm") {
            continue;
        }
        let src = std::fs::read_to_string(&p).unwrap();
        let (n, surf, _) = holon::qasm::parse_surface(&src).unwrap();
        let local = simplify(&surf);
        println!(
            "{name:34}  n={n:3}  raw magic {:4}  after local {:4}  after PHASE-POLY {:4}",
            magic_weight(&surf),
            magic_weight(&local),
            normalized_t_count(n, &local)
        );
    }
}
