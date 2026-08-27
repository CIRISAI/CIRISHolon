use holon_qasm::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let usage = "usage: holon-qasm run <file.qasm> [--tier classical|tableau|statevector] [--mutate s-phase|cx-phase|cx-swap] | route <file.qasm>";
    if args.len() < 3 {
        eprintln!("{usage}");
        std::process::exit(2);
    }
    let src = std::fs::read_to_string(&args[2]).expect("read qasm");
    let c = parse(&src).unwrap_or_else(|e| {
        eprintln!("parse error: {e}");
        std::process::exit(2);
    });
    match args[1].as_str() {
        "route" => match route(&c) {
            Ok(t) => println!("{{\"tier\": \"{t:?}\", \"n\": {}}}", c.n_qubits),
            Err(msg) => {
                println!("{{\"tier\": \"REFUSED\", \"reason\": {:?}}}", msg);
                std::process::exit(3);
            }
        },
        "run" => {
            let mut tier = None;
            let mut m = Mutation::None;
            let mut i = 3;
            while i < args.len() {
                match args[i].as_str() {
                    "--tier" => {
                        tier = Some(match args[i + 1].as_str() {
                            "classical" => Tier::Classical,
                            "tableau" => Tier::Tableau,
                            "statevector" => Tier::Statevector,
                            _ => panic!("bad tier"),
                        });
                        i += 2;
                    }
                    "--sample" => {
                        // timing path: one deterministic shot
                        let t0 = std::time::Instant::now();
                        let bits = run_tableau_sample(&c, Mutation::None);
                        let dt = t0.elapsed().as_secs_f64();
                        println!("{{\"tier\": \"Tableau\", \"n\": {}, \"seconds\": {dt:.6}, \"sample\": \"{bits}\"}}", c.n_qubits);
                        return;
                    }
                    "--mutate" => {
                        m = match args[i + 1].as_str() {
                            "s-phase" => Mutation::TableauSPhase,
                            "cx-phase" => Mutation::TableauCxPhase,
                            "cx-swap" => Mutation::ClassicalCxSwap,
                            _ => panic!("bad mutation"),
                        };
                        i += 2;
                    }
                    _ => panic!("bad arg {}", args[i]),
                }
            }
            let tier = match tier {
                Some(t) => t,
                None => match route(&c) {
                    Ok(t) => t,
                    Err(msg) => {
                        eprintln!("{msg}");
                        std::process::exit(3);
                    }
                },
            };
            let t0 = std::time::Instant::now();
            let dist = run(&c, tier, m);
            let dt = t0.elapsed().as_secs_f64();
            let entries: Vec<String> = dist
                .iter()
                .map(|(k, v)| format!("\"{k}\": {v:.12}"))
                .collect();
            println!(
                "{{\"tier\": \"{tier:?}\", \"n\": {}, \"seconds\": {dt:.6}, \"dist\": {{{}}}}}",
                c.n_qubits,
                entries.join(", ")
            );
        }
        _ => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    }
}
