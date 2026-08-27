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
        "amp" => {
            // amplitude of |0...0> via the magic tier: 2^t · poly(n), no 2^n
            let y = vec![false; c.n_qubits];
            let t0 = std::time::Instant::now();
            let (re, im) = holon_qasm::magic::magic_amplitude(&c, &y, false, false);
            let dt = t0.elapsed().as_secs_f64();
            println!(
                "{{\"seconds\": {dt:.6}, \"re\": {re:.12}, \"im\": {im:.12}, \"p\": {:.12}}}",
                re * re + im * im
            );
        }
        "test-magic" => {
            // dev loop: magic tier vs in-crate statevector on THIS circuit
            let d1 = holon_qasm::magic::run_magic(&c, false, false);
            let d2 = run_statevector(&c);
            let keys: std::collections::BTreeSet<_> =
                d1.keys().chain(d2.keys()).collect();
            let mut worst = 0.0f64;
            for k in keys {
                let e = (d1.get(k).unwrap_or(&0.0) - d2.get(k).unwrap_or(&0.0)).abs();
                if e > worst {
                    worst = e;
                }
            }
            println!("{{\"max_err\": {worst:.3e}}}");
            if worst > 1e-9 {
                std::process::exit(1);
            }
        }
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
                            "magic" => Tier::Magic,
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
                            "magic-s-cross" => Mutation::MagicSCross,
                            "magic-gauss" => Mutation::MagicGauss,
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
