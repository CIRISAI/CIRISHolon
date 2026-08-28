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
    if args[1] == "face-amp" {
        // The face engine: exact amplitudes for programs carrying face
        // rotations (Z[ω][√3]), with the canonical merge between rounds.
        let src = std::fs::read_to_string(&args[2]).expect("read");
        let (n, surface, _) = holon::qasm::parse_surface(&src).unwrap_or_else(|e| {
            eprintln!("REFUSED (line {}): {}", e.line, e.reason);
            std::process::exit(2);
        });
        let y: Vec<bool> = if args.len() > 3 {
            args[3].trim().chars().map(|c| c == '1').collect()
        } else {
            vec![false; n]
        };
        let merge_every: usize = std::env::var("HOLON_FACE_MERGE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let faces = surface
            .iter()
            .filter(|g| matches!(g, holon::qasm::Surface::Face(..) | holon::qasm::Surface::T(_) | holon::qasm::Surface::Tdg(_)))
            .count();
        let t0 = std::time::Instant::now();
        let (amp, peak) = holon::face::amplitude_face(n, &surface, &y, merge_every);
        let (re, im) = amp.to_complex();
        println!(
            "{{\"seconds\": {:.6}, \"n\": {n}, \"magic_gates\": {faces}, \"peak_branches\": {peak}, \"re\": {re:.12}, \"im\": {im:.12}, \"p\": {:.12}}}",
            t0.elapsed().as_secs_f64(),
            re * re + im * im
        );
        return;
    }
    assert_eq!(args[1], "amp", "usage: holon-run amp|face-amp|clifford-sample <file.qasm> [y-bits]");
    // SIMPLIFY-then-run is the default (BENCHMARKS entry thirteen): the pass
    // is exact and cuts t before any exponent applies. HOLON_NO_SIMPLIFY=1
    // disables it for A/B measurement.
    let src = std::fs::read_to_string(&args[2]).expect("read");
    let (nq, surf, _meas) = holon::qasm::parse_surface(&src).unwrap_or_else(|e| {
        eprintln!("REFUSED (line {}): {}", e.line, e.reason);
        std::process::exit(2);
    });
    let simplify_on = std::env::var("HOLON_NO_SIMPLIFY").is_err();
    let t_before = holon::simplify::magic_weight(&surf);
    let gates_before = surf.len();
    let t_simp = std::time::Instant::now();
    // Two passes compose: local cancellation shrinks the bulk, then the
    // phase-polynomial pass cancels magic AT A DISTANCE (the ceiling entry
    // fourteen named). HOLON_NO_PHASEPOLY=1 isolates the local pass.
    let surf = if simplify_on { holon::simplify::simplify(&surf) } else { surf };
    let t_local = holon::simplify::magic_weight(&surf);
    let ppoly_on = simplify_on && std::env::var("HOLON_NO_PHASEPOLY").is_err();
    let surf = if ppoly_on { holon::phasepoly::optimize(nq, &surf) } else { surf };
    let surf = if ppoly_on { holon::simplify::simplify(&surf) } else { surf };
    let simplify_s = t_simp.elapsed().as_secs_f64();
    let t_after = holon::simplify::magic_weight(&surf);
    let (core, phase16) = holon::qasm::lower(&surf);
    let p16 = phase16.rem_euclid(16);
    let p = holon::qasm::Program {
        n_qubits: nq,
        gates: core,
        measured: vec![],
        phase_omega: (p16 / 2) as u8,
        residual_zeta16: (p16 % 2) as u8,
    };
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
    // OUTPUT: the Qiskit Result schema's shape (success/results/data +
    // metadata), so standard tooling can read it, with everything this
    // engine knows that no spec has a field for living under `metadata`
    // (INTERFACE.md's four undefined types, carried honestly rather than
    // silently dropped).
    println!(
        "{{\"backend_name\": \"cirisholon\", \"success\": true, \"results\": [{{\"shots\": 1, \"status\": \"DONE\", \"data\": {{\"amplitude\": {{\"re\": {re:.12}, \"im\": {im:.12}}}, \"probability\": {pr:.12}}}, \"metadata\": {{\"exact\": true, \"ring\": \"Z[omega]\", \"residual_zeta16\": {residual}, \"n_qubits\": {}, \"simplify\": {{\"enabled\": {simplify_on}, \"seconds\": {simplify_s:.6}, \"gates_before\": {gates_before}, \"gates_after\": {}, \"magic_before\": {t_before}, \"magic_after_local\": {t_local}, \"magic_after\": {t_after}, \"phasepoly\": {ppoly_on}}}, \"seconds\": {:.6}}}}}]}}",
        p.n_qubits,
        surf.len(),
        t0.elapsed().as_secs_f64()
    );
}
