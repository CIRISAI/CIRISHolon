//! REFUTER for the ZX head-to-head: run our native pass against quizx on
//! circuits neither implementation was tuned on, and compare BOTH readings —
//! open-diagram and closed-diagram T-count — file by file. Agreement on the
//! twelve benchmark circuits is a match; agreement on fresh ones is evidence.
use std::collections::HashMap;

fn load(path: &str) -> HashMap<String, usize> {
    std::fs::read_to_string(path)
        .map(|s| {
            s.lines()
                .filter_map(|l| {
                    let mut f = l.split('\t');
                    let name = f.next()?;
                    let name = name.rsplit('/').next()?.to_string();
                    Some((name, f.nth(1)?.trim().parse().ok()?))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn main() {
    let dir = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/fresh";
    let open = load("/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/fresh_open.txt");
    let plug = load("/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/fresh_plug.txt");
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| {
            let p = e.unwrap().path();
            let n = p.file_name()?.to_string_lossy().to_string();
            if n.ends_with(".qasm") { Some((n, p)) } else { None }
        })
        .collect();
    files.sort();
    let (mut ok, mut bad) = (0, 0);
    println!("{:<26} {:>4} {:>6} {:>7} {:>7}   {:>7} {:>7}", "circuit", "n", "raw", "open", "q-open", "closed", "q-closed");
    for (name, path) in files {
        let src = std::fs::read_to_string(&path).unwrap();
        let (n, surf, _) = holon::qasm::parse_surface(&src).unwrap();
        let raw = holon::simplify::magic_weight(&surf);
        let o = holon::zx::full_reduced_t_count(n, &surf).unwrap();
        let zero = vec![false; n];
        // time the REDUCTION only, matching what quizx's harness times.
        let mut g = holon::zx::from_surface(n, &surf).unwrap();
        g.plug_inputs(&zero);
        g.plug_outputs(&zero);
        let t0 = std::time::Instant::now();
        g.full_reduce();
        let secs = t0.elapsed().as_secs_f64();
        let c = g.t_count();
        let qo = open.get(&name).copied();
        let qc = plug.get(&name).copied();
        let agree = qo == Some(o) && qc == Some(c);
        if agree { ok += 1 } else { bad += 1 }
        println!(
            "{name:<26} {n:>4} {raw:>6} {o:>7} {:>7}   {c:>7} {:>7}  {}  ({secs:.3}s)",
            qo.map(|v| v.to_string()).unwrap_or("-".into()),
            qc.map(|v| v.to_string()).unwrap_or("-".into()),
            if agree { "agree" } else { "*** DISAGREE ***" }
        );
    }
    println!("\n{ok} circuits agree with quizx on BOTH readings, {bad} disagree");
}
