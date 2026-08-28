//! THE SCALE CHECK for the native ZX pass, against an answer nobody in this
//! repo chose: quizx's hidden-shift generator emits a circuit whose amplitude
//! ⟨shift|C|0…0⟩ is exactly 1, and ships the shift string beside the QASM.
//!
//! So: plug the closed diagram, reduce it to a scalar, and read the scalar.
//! It must be exactly 1 in `Z[ω]·2^{−m/2}` — at q40 with 350 T gates, which
//! is far past anything the unit tests can brute-force. A two-sided control
//! runs the same pipeline on a CORRUPTED shift, where the answer must not be 1.
use holon::zx::from_surface;

const BASE: &str = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/quizx";

fn amplitude_of(n: usize, surf: &[holon::qasm::Surface], y: &[bool]) -> holon::ledger::Cyc {
    let mut g = from_surface(n, surf).unwrap();
    g.plug_inputs(&vec![false; n]);
    g.plug_outputs(y);
    g.full_reduce();
    assert_eq!(g.t_count(), 0, "hidden shift must reduce to a Clifford scalar");
    g.eval()
}

fn main() {
    let mut files: Vec<_> = std::fs::read_dir(BASE)
        .unwrap()
        .filter_map(|e| {
            let p = e.unwrap().path();
            let n = p.file_name()?.to_string_lossy().to_string();
            if n.starts_with("h2h_hs_q") && n.ends_with(".qasm") { Some((n, p)) } else { None }
        })
        .collect();
    files.sort();
    for (name, path) in files {
        let src = std::fs::read_to_string(&path).unwrap();
        let (n, surf, _) = holon::qasm::parse_surface(&src).unwrap();
        let shift_path = path.with_extension("shift");
        let shift: Vec<bool> = std::fs::read_to_string(&shift_path)
            .unwrap()
            .trim()
            .chars()
            .map(|c| c == '1')
            .collect();
        assert_eq!(shift.len(), n);
        let t0 = std::time::Instant::now();
        let amp = amplitude_of(n, &surf, &shift);
        let secs = t0.elapsed().as_secs_f64();
        // the two-sided control: one bit of the shift flipped
        let mut wrong = shift.clone();
        wrong[0] = !wrong[0];
        let ctl = amplitude_of(n, &surf, &wrong);
        let is_one = holon::zx::cyc_eq(amp, holon::ledger::Cyc::ONE);
        println!(
            "{name:<26} n={n:3}  <shift|C|0> = {:?}  {}   control(1 bit flipped) = {:?}   ({secs:.2}s)",
            amp.to_complex(),
            if is_one { "EXACTLY 1" } else { "*** NOT 1 ***" },
            ctl.to_complex()
        );
    }
}
