//! SCALE VERIFICATION for the extractor.
//!
//! `tests/zx.rs` checks extraction against the certified runner entry by
//! entry over the whole 2^n × 2^n matrix — but only to five qubits, because
//! the runner sums branches and these circuits carry up to 350 T gates. So
//! the twelve circuits the head-to-head actually reports on are not covered
//! by that gate, and a report that leaned on it would be overclaiming.
//!
//! This closes the hole with a check that does not care about size: compose
//! each circuit with the ADJOINT of its own extraction and reduce. If
//! extraction was exact, `C · extracted†` is the identity times a scalar, and
//! the reducer says so — with the scalar it derives independently having to
//! match the one extraction reported. The composite never touches the
//! extractor, so this is a second reader, not a restatement.
//!
//! The hidden-shift circuits get an EXTERNAL anchor as well: their amplitude
//! ⟨shift|C|0…0⟩ is exactly 1 by construction of quizx's generator — a number
//! nobody in this repo chose — and it must survive extraction.
use holon::ledger::Cyc;

const BASE: &str = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/quizx";

fn main() {
    let mut files: Vec<_> = std::fs::read_dir(BASE)
        .unwrap()
        .filter_map(|e| {
            let p = e.unwrap().path();
            let n = p.file_name()?.to_string_lossy().to_string();
            let want = n.starts_with("rand_q") || n.starts_with("h2h_hs_q");
            if want && n.ends_with(".qasm") {
                Some((n, p))
            } else {
                None
            }
        })
        .collect();
    files.sort();

    let (mut ok, mut bad) = (0, 0);
    for (name, path) in files {
        let src = std::fs::read_to_string(&path).unwrap();
        let (n, surf, _) = holon::qasm::parse_surface(&src).unwrap();
        let t0 = std::time::Instant::now();
        let verdict = holon::zx::certify_extraction(n, &surf);
        let secs = t0.elapsed().as_secs_f64();

        // The external anchor, where the generator shipped one.
        let anchor = std::fs::read_to_string(path.with_extension("shift")).ok().map(|s| {
            let y: Vec<bool> = s.trim().chars().map(|c| c == '1').collect();
            let ex = holon::zx::extract_circuit(n, &surf).unwrap();
            let mut g = holon::zx::from_surface(n, &ex.gates).unwrap();
            g.plug_inputs(&vec![false; n]);
            g.plug_outputs(&y);
            g.full_reduce();
            if g.t_count() != 0 {
                "   anchor: not a Clifford scalar".to_string()
            } else if holon::zx::cyc_eq(ex.scalar.mul(g.eval()), Cyc::ONE) {
                "   anchor <shift|extracted|0> = EXACTLY 1".to_string()
            } else {
                "   anchor *** NOT 1 ***".to_string()
            }
        });

        // A third, fully independent leg where the runner can still reach:
        // sum the extracted circuit's branches directly and compare with the
        // original's. Only feasible while the T-count is small.
        let runner = {
            let (core_o, _) = holon::qasm::lower(&surf);
            let ex2 = holon::zx::extract_circuit(n, &surf).unwrap();
            let (core_e, _) = holon::qasm::lower(&ex2.gates);
            let t = core_e.iter().filter(|g| g.is_t()).count();
            let t_orig = core_o.iter().filter(|g| g.is_t()).count();
            // BOTH sides go through the branch sum, so the ORIGINAL's T-count
            // gates feasibility too — the hidden-shift circuits carry 56-350.
            if t.max(t_orig) > 26 {
                String::new()
            } else {
                let y: Vec<bool> = (0..n).map(|q| q % 3 == 0).collect();
                let a = holon::run::amplitude(n, &core_o, &y);
                let b = ex2.scalar.mul(holon::run::amplitude(n, &core_e, &y));
                if holon::zx::cyc_eq(a, b) {
                    format!("   runner cross-check (t={t}): EXACT")
                } else {
                    format!("   runner cross-check (t={t}): *** MISMATCH ***")
                }
            }
        };

        match verdict {
            Ok(s) => {
                ok += 1;
                println!(
                    "{name:<26} n={n:3}  C·extracted† = IDENTITY, scalar {:?}  ({secs:.2}s){}",
                    s.to_complex(),
                    format!("{}{}", anchor.unwrap_or_default(), runner)
                );
            }
            Err(e) => {
                bad += 1;
                println!("{name:<26} n={n:3}  *** {e} ***");
            }
        }
    }
    println!("\n{ok} circuits certified exact, {bad} failed");
}
