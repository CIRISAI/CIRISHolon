//! THE OWED EXPERIMENT: the head-to-head ladder end to end THROUGH the
//! composed pipeline — canonicalize, then evaluate exactly — versus the
//! last measured engine-alone numbers. Entry twenty measured the T-count
//! reduction; this measures whether it converts into wall clock.
use holon::job::{run_surface, JobConfig};
fn main() {
    let base = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/quizx";
    let mut fs: Vec<_> = std::fs::read_dir(base).unwrap().filter_map(|e| {
        let p = e.unwrap().path(); let n = p.file_name()?.to_string_lossy().to_string();
        if n.starts_with("h2h_hs_") && n.ends_with(".qasm") { Some((n,p)) } else { None }}).collect();
    fs.sort_by_key(|(n,_)| n.split("_q").nth(1).and_then(|s| s.split('_').next())
                            .and_then(|s| s.parse::<usize>().ok()).unwrap_or(0));
    println!("{:>4} {:>6} {:>7} {:>10} {:>12} {:>12}", "q", "T_raw", "T_canon", "canon_s", "eval_s", "total_s");
    for (name, p) in fs {
        let src = std::fs::read_to_string(&p).unwrap();
        let Ok((n, surf, _)) = holon::qasm::parse_surface(&src) else { continue };
        let shift = std::fs::read_to_string(p.with_extension("shift")).unwrap_or_default();
        let target: String = shift.trim().to_string();
        let t0 = std::time::Instant::now();
        let (canon, red) = match holon_zx::canonicalize(n, &surf) {
            Ok(v) => v, Err(e) => { println!("{name}: canonicalize ERR {e}"); continue } };
        let canon_s = t0.elapsed().as_secs_f64();
        let cfg = JobConfig { target: if target.len()==n { Some(target) } else { None }, ..Default::default() };
        let t1 = std::time::Instant::now();
        match run_surface(n, &canon, &cfg) {
            Ok(r) => {
                let eval_s = t1.elapsed().as_secs_f64();
                println!("{n:4} {:6} {:7} {canon_s:10.3} {eval_s:12.3} {:12.3}   p={:.6}",
                         red.t_before, red.t_after, canon_s + eval_s, r.probability);
            }
            Err(e) => println!("{n:4} {:6} {:7} {canon_s:10.3}   eval ERR {e}", red.t_before, red.t_after),
        }
    }
}
