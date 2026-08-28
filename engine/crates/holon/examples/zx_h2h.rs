//! THE SIMPLIFIER HEAD-TO-HEAD, all four of our passes against quizx.
//!
//! Columns:
//!   raw       — T gates in the circuit as written
//!   local     — `simplify::simplify`, the block-local pass
//!   ppoly     — plus `phasepoly::normalized_t_count`
//!   zx-open   — `zx::full_reduced_t_count`, our native ZX on the OPEN circuit
//!   zx-amp    — `zx::amplitude_t_count`, our native ZX on the CLOSED diagram
//!               ⟨0…0|C|0…0⟩ — the metric that prices a stabiliser
//!               decomposition, and the one entry sixteen measured quizx by
//!   quizx     — quizx `full_simp` on the same closed diagram (reference file)
//!
//! The reference column is read from a file produced by quizx itself
//! (`tcount_ref --plug`), so this harness never has to link the reference.
use holon::phasepoly::normalized_t_count;
use holon::simplify::{magic_weight, simplify};
use std::collections::HashMap;

const BASE: &str = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/quizx";
const REF: &str = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/quizx_ref_plugged.txt";

fn main() {
    let reference: HashMap<String, usize> = std::fs::read_to_string(REF)
        .map(|s| {
            s.lines()
                .filter_map(|l| {
                    let mut f = l.split('\t');
                    Some((f.next()?.to_string(), f.nth(1)?.trim().parse().ok()?))
                })
                .collect()
        })
        .unwrap_or_default();

    let mut files: Vec<_> = std::fs::read_dir(BASE)
        .unwrap()
        .filter_map(|e| {
            let p = e.unwrap().path();
            let n = p.file_name()?.to_string_lossy().to_string();
            let want = n.starts_with("rand_q") || n.starts_with("h2h_hs_q");
            if want && n.ends_with(".qasm") {
                Some((n, p))
            } else {
                None
            }
        })
        .collect();
    files.sort();

    println!(
        "{:<26} {:>3}  {:>5} {:>6} {:>6} {:>8} {:>7} {:>6}",
        "circuit", "n", "raw", "local", "ppoly", "zx-open", "zx-amp", "quizx"
    );
    for (name, path) in files {
        let src = std::fs::read_to_string(&path).unwrap();
        let (n, surf, _) = match holon::qasm::parse_surface(&src) {
            Ok(v) => v,
            Err(e) => {
                println!("{name:<26} REFUSED: {}", e.reason);
                continue;
            }
        };
        let raw = magic_weight(&surf);
        let loc = simplify(&surf);
        let local = magic_weight(&loc);
        let ppoly = normalized_t_count(n, &loc);
        let t0 = std::time::Instant::now();
        let zx_open = holon::zx::full_reduced_t_count(n, &surf).unwrap();
        let zero = vec![false; n];
        let zx_amp = holon::zx::amplitude_t_count(n, &surf, &zero, &zero).unwrap();
        let secs = t0.elapsed().as_secs_f64();
        let q = reference.get(&name).map(|v| v.to_string()).unwrap_or_else(|| "-".into());
        println!(
            "{name:<26} {n:>3}  {raw:>5} {local:>6} {ppoly:>6} {zx_open:>8} {zx_amp:>7} {q:>6}   ({secs:.2}s)"
        );
    }
}
