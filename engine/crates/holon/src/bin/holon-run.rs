//! The production-path CLI: the full stack (BG-aware pruned dedup + mesh
//! fold) behind the same QASM subset. `holon-run amp <file.qasm>` prints the
//! exact all-zeros amplitude and seconds, matching holon-qasm's `amp` output
//! shape so the battle-rig can swap binaries.
use holon::prune::Gate;


fn clifford_sample(n: usize, gates: &[Gate]) {
    // The gate path runs on the TRANSPOSED engine (word-parallel columns,
    // conformance-gated bit-identical to the reference); measurement flows
    // through the certified row-major reference after one transpose.
    use holon::coltableau::ColTableau;
    let t0 = std::time::Instant::now();
    let mut col = ColTableau::new(n);
    for g in gates {
        match *g {
            Gate::X(q) => col.x_gate(q),
            Gate::Z(q) => col.z_gate(q),
            Gate::H(q) => col.h(q),
            Gate::S(q) => col.s(q),
            Gate::Sdg(q) => col.sdg(q),
            Gate::Cx(c, q) => col.cx(c, q),
            _ => panic!("clifford-sample requires a Clifford circuit"),
        }
    }
    let gates_s = t0.elapsed().as_secs_f64();
    // Terminal sample entirely on flat planes — BORN-RANDOM (free bits from
    // a seeded stream, semantically the same work as stim's random
    // measurement), seed logged for replay. External review caught the
    // earlier version comparing a deterministic canonical witness against
    // stim's Born sampler; this closes that gap at the same one-pass cost.
    let seed: u64 = std::env::var("HOLON_SAMPLE_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0x5EED_0F_B0121);
    let y = col.sample_born_flat(seed);
    let ones: usize = y.iter().map(|&b| b as usize).sum();
    println!(
        "{{\"seconds\": {:.6}, \"gates_s\": {gates_s:.6}, \"ones\": {ones}, \"born_seed\": {seed}}}",
        t0.elapsed().as_secs_f64()
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args[1] == "clifford-sample" {
        let src = std::fs::read_to_string(&args[2]).expect("read");
        let p = holon::qasm::parse(&src).unwrap_or_else(|e| {
            eprintln!("REFUSED (line {}): {}", e.line, e.reason);
            std::process::exit(2);
        });
        // Clifford path still requires a Clifford circuit; T's panic below.
        let gates: Vec<Gate> = p.gates;
        clifford_sample(p.n_qubits, &gates);
        return;
    }
    assert_eq!(args[1], "amp", "usage: holon-run amp|clifford-sample <file.qasm> [y-bits]");
    let src = std::fs::read_to_string(&args[2]).expect("read");
    let p = holon::qasm::parse(&src).unwrap_or_else(|e| {
        eprintln!("REFUSED (line {}): {}", e.line, e.reason);
        std::process::exit(2);
    });
    let y: Vec<bool> = if args.len() > 3 {
        let bits = args[3].trim();
        assert_eq!(bits.len(), p.n_qubits, "y-bits length must equal qubit count");
        // convention: leftmost char = qubit 0
        bits.chars().map(|c| c == '1').collect()
    } else {
        vec![false; p.n_qubits]
    };
    let t0 = std::time::Instant::now();
    let (amp, residual) = holon::run::amplitude_program(&p, &y);
    let (re, im) = amp.to_complex();
    let pr = re * re + im * im;
    println!(
        "{{\"seconds\": {:.6}, \"re\": {:.12}, \"im\": {:.12}, \"p\": {:.12}, \"residual_zeta16\": {}}}",
        t0.elapsed().as_secs_f64(),
        re,
        im,
        pr,
        residual
    );
}
