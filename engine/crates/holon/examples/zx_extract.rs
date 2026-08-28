//! THE EXTRACTION HEAD-TO-HEAD.
//!
//! Columns:
//!   raw        — T gates in the circuit as written
//!   local      — `simplify::simplify`, the block-local pass
//!   ppoly      — plus `phasepoly::normalized_t_count`
//!   oracle     — `zx::full_reduced_t_count`: the reduced OPEN diagram
//!   extracted  — T-count of the circuit our extractor hands back
//!   q-extract  — T-count of the circuit QUIZX's extractor hands back
//!   gates      — total core gates in our extracted circuit
//!
//! `oracle` and `extracted` must agree: extraction may not create a T.
use std::collections::HashMap;

const BASE: &str = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/quizx";
const REF: &str = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/quizx_extract_ref.txt";

fn t_of(surf: &[holon::qasm::Surface]) -> (usize, usize) {
    let (core, _) = holon::qasm::lower(surf);
    (core.iter().filter(|g| g.is_t()).count(), core.len())
}

/// Surface-level gate census. This is the fair unit against quizx, whose
/// extracted circuits are counted in an alphabet that keeps CZ as ONE gate —
/// our `lower` would expand each CZ into H·CX·H and treble the number.
fn census(surf: &[holon::qasm::Surface]) -> (usize, usize, usize, usize, usize) {
    use holon::qasm::Surface::*;
    let mut h = 0; let mut cx = 0; let mut cz = 0; let mut oneq = 0; let mut sw = 0;
    for g in surf {
        match g {
            H(_) => h += 1,
            Cx(..) => cx += 1,
            Cz(..) => cz += 1,
            Swap(..) => sw += 1,
            _ => oneq += 1,
        }
    }
    (surf.len(), h, cx + sw * 3, cz, oneq)
}

fn main() {
    let reference: HashMap<String, String> = std::fs::read_to_string(REF)
        .map(|s| {
            s.lines()
                .filter_map(|l| {
                    let mut f = l.split('\t');
                    let name = f.next()?.to_string();
                    let ext = f.find(|x| x.starts_with("extracted_t="))?;
                    Some((name, ext.trim_start_matches("extracted_t=").to_string()))
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
            if want && n.ends_with(".qasm") { Some((n, p)) } else { None }
        })
        .collect();
    files.sort();
    println!(
        "{:<26} {:>3} {:>5} {:>6} {:>6} {:>7} {:>10} {:>10} {:>7} {:>7} {:>5} {:>5} {:>5} {:>5}",
        "circuit", "n", "raw", "local", "ppoly", "oracle", "extracted", "q-extract",
        "round2", "gates", "H", "CX", "CZ", "1q"
    );
    for (name, path) in files {
        let src = std::fs::read_to_string(&path).unwrap();
        let (n, surf, _) = holon::qasm::parse_surface(&src).unwrap();
        let (raw, _) = t_of(&surf);
        let loc = holon::simplify::simplify(&surf);
        let (local, _) = t_of(&loc);
        let ppoly = holon::phasepoly::normalized_t_count(n, &loc);
        let oracle = holon::zx::full_reduced_t_count(n, &surf).unwrap();
        // Time the EXTRACTION only, matching what quizx's harness times
        // (its clock starts after full_simp).
        let mut g = holon::zx::from_surface(n, &surf).unwrap();
        g.full_reduce();
        let t0 = std::time::Instant::now();
        let _ = g.extract();
        let secs_extract = t0.elapsed().as_secs_f64();
        let t0 = std::time::Instant::now();
        match holon::zx::extract_circuit(n, &surf) {
            Ok(ex) => {
                let secs = t0.elapsed().as_secs_f64();
                let (et, _) = t_of(&ex.gates);
                let (tot, h, cx, cz, oneq) = census(&ex.gates);
                // Does a SECOND round of reduce-and-extract find anything the
                // first missed? If it did, iterating would be free T-count.
                let round2 = holon::zx::full_reduced_t_count(n, &ex.gates).unwrap_or(usize::MAX);
                let q = reference.get(&name).cloned().unwrap_or_else(|| "-".into());
                let flag = if et == oracle { "" } else { "  *** T CREATED ***" };
                println!("{name:<26} {n:>3} {raw:>5} {local:>6} {ppoly:>6} {oracle:>7} {et:>10} {q:>10} {round2:>7} {tot:>7} {h:>5} {cx:>5} {cz:>5} {oneq:>5}  (extract {secs_extract:.4}s, total {secs:.3}s){flag}");
            }
            Err(e) => println!("{name:<26} {n:>3} {raw:>5} {local:>6} {ppoly:>6} {oracle:>7}   EXTRACT FAILED: {e}"),
        }
    }
}
