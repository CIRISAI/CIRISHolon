//! The production-path CLI: the full stack (BG-aware pruned dedup + mesh
//! fold) behind the same QASM subset. `holon-run amp <file.qasm>` prints the
//! exact all-zeros amplitude and seconds, matching holon-qasm's `amp` output
//! shape so the battle-rig can swap binaries.
use holon::prune::Gate;
use holon::run::{amplitude, amplitude_sharded};

fn parse(src: &str) -> Result<(usize, Vec<Gate>), String> {
    let mut n = 0usize;
    let mut gates = Vec::new();
    let idx = |tok: &str| -> Result<usize, String> {
        let o = tok.find('[').ok_or("bad operand")?;
        let c = tok.find(']').ok_or("bad operand")?;
        tok[o + 1..c].parse().map_err(|_| "bad index".into())
    };
    for raw in src.lines() {
        let line = raw.split("//").next().unwrap_or("").trim();
        for stmt in line.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty()
                || stmt.starts_with("OPENQASM")
                || stmt.starts_with("include")
                || stmt.starts_with("creg")
                || stmt.starts_with("measure")
            {
                continue;
            }
            if let Some(rest) = stmt.strip_prefix("qreg ") {
                n = idx(rest.trim())?;
                continue;
            }
            let (op, args) = stmt.split_once(' ').ok_or(format!("bad: {stmt}"))?;
            let a: Vec<usize> = args
                .split(',')
                .map(|s| idx(s.trim()))
                .collect::<Result<_, _>>()?;
            gates.push(match (op.trim(), a.len()) {
                ("x", 1) => Gate::X(a[0]),
                ("z", 1) => Gate::Z(a[0]),
                ("h", 1) => Gate::H(a[0]),
                ("s", 1) => Gate::S(a[0]),
                ("sdg", 1) => Gate::Sdg(a[0]),
                ("t", 1) => Gate::T(a[0]),
                ("tdg", 1) => Gate::Tdg(a[0]),
                ("cx", 2) => Gate::Cx(a[0], a[1]),
                _ => return Err(format!("unsupported: {stmt}")),
            });
        }
    }
    Ok((n, gates))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args[1], "amp", "usage: holon-run amp <file.qasm>");
    let src = std::fs::read_to_string(&args[2]).expect("read");
    let (n, gates) = parse(&src).expect("parse");
    let y = vec![false; n];
    let shards: Option<usize> = args.get(3).map(|s| s.parse().expect("shards"));
    let t0 = std::time::Instant::now();
    let a = match shards {
        Some(k) => amplitude_sharded(n, &gates, &y, k),
        None => amplitude(n, &gates, &y),
    };
    let dt = t0.elapsed().as_secs_f64();
    let (re, im) = a.to_complex();
    println!(
        "{{\"seconds\": {dt:.6}, \"re\": {re:.12}, \"im\": {im:.12}, \"p\": {:.12}}}",
        re * re + im * im
    );
}
