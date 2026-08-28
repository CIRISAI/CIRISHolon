//! The composed canonicalizer on the head-to-head ladder: what does it buy?
fn main() {
    let base = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/quizx";
    let mut fs: Vec<_> = std::fs::read_dir(base).unwrap().filter_map(|e| {
        let p = e.unwrap().path(); let n = p.file_name()?.to_string_lossy().to_string();
        if n.starts_with("h2h_hs_") && n.ends_with(".qasm") { Some((n,p)) } else { None }}).collect();
    fs.sort_by_key(|(n,_)| n.split("_q").nth(1).and_then(|s| s.split('_').next()).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0));
    for (name, p) in fs {
        let src = std::fs::read_to_string(&p).unwrap();
        let Ok((n, surf, _)) = holon::qasm::parse_surface(&src) else { continue };
        let t0 = std::time::Instant::now();
        match holon_zx::canonicalize(n, &surf) {
            Ok((_s, r)) => println!("{name:28} n={n:3}  T {:4} -> {:4}   gates {:5} -> {:5}   {:.3}s",
                                    r.t_before, r.t_after, r.gates_before, r.gates_after, t0.elapsed().as_secs_f64()),
            Err(e) => println!("{name:28} n={n:3}  ERR {e}"),
        }
    }
}
