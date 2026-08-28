use holon::zx::from_surface;
fn main() {
    let base = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/quizx";
    let mut fs: Vec<_> = std::fs::read_dir(base).unwrap().filter_map(|e| {
        let p = e.unwrap().path();
        let n = p.file_name()?.to_string_lossy().to_string();
        if n.starts_with("rand_q") && n.ends_with(".qasm") { Some((n, p)) } else { None }
    }).collect();
    fs.sort();
    for (name, p) in fs {
        let src = std::fs::read_to_string(&p).unwrap();
        let (n, surf, _) = holon::qasm::parse_surface(&src).unwrap();
        let mut g = from_surface(n, &surf).unwrap();
        let (s0, t0) = (g.n_spiders(), g.t_count());
        g.simplify();
        let (s1, t1) = (g.n_spiders(), g.t_count());
        let mut g2 = from_surface(n, &surf).unwrap();
        g2.simplify();
        let (ng, nm) = g2.gadget_stats();
        g2.gadgetize();
        let (ng2, nm2) = g2.gadget_stats();
        g2.full_reduce();
        print!("gadgets {ng}/{nm}match -> after gadgetize {ng2}/{nm2}match | ");
        println!("{name:26} spiders {s0:6} -> {s1:5} -> {:5}   T {t0:4} -> {t1:4} -> {:4}",
                 g2.n_spiders(), g2.t_count());
    }
}
