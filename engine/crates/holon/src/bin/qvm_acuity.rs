//! `qvm_acuity` — QVM-ACUITY-1's end-to-end driver: LOCATE the cheap sector,
//! PRICE the hard part before running it, then do only enough of it.
//!
//! One run prints three things, in this order, and the order is the claim:
//!
//! ```text
//! sector: clifford=400 hard=8 removed=0 light_cone=[0,…,11]
//! price:  t_eff=8 N_pred=12
//! {"n": 12, "t": 8, …}
//! ```
//!
//! The price line is printed BEFORE a single branch is evaluated (S2), and
//! the JSON line carries what the sweep tables: the executed branch count
//! against the predicted one, the certified remainder, the value, the
//! referee where one can exist (`n ≤ 20`), and the three walls.
//!
//! ## The backend
//!
//! The hard part is `holon::acuity::budgeted_amplitude(src, y, eps, shards)`,
//! built in parallel by the other half of this campaign. Until that module is
//! in this worktree the driver runs on `sector::FullSum` — every branch,
//! exact `Z[ω]`, `eps` IGNORED, remainder `0` — and says so in `backend`, on
//! stderr, and in every JSON line it writes. A full-sum number can therefore
//! never be read as a budgeted one. Swapping in the real backend is the one
//! `impl AcuityBackend` block marked MERGE POINT below.
//!
//! ## Usage
//!
//! ```text
//! qvm_acuity --random N T SEED [options]
//! qvm_acuity --qasm FILE [options]
//!
//!   --amp BITS              observable: ⟨y|C|0⟩, BITS[0] is qubit 0
//!                           (default: the all-zeros amplitude)
//!   --amp-skeleton          declare y by `sector::skeleton_bitstring` — the
//!                           tableau tier's sample of the circuit's Clifford
//!                           skeleton, which is where a random Clifford+T
//!                           state's amplitude actually lives (⟨0^n|C|0⟩ is
//!                           exactly zero on most of these instances)
//!   --amp-argmax            declare y by `sector::argmax_bitstring` — the
//!                           largest |⟨y|C|0⟩| the referee shows, the rule
//!                           the sweep uses wherever a referee exists,
//!                           because the skeleton's y can still land on an
//!                           exact zero (measured: n=12, t=8, seed 1)
//!   --marginal q0,q1,q2,q3  observable: p(bits on those qubits)
//!   --marginal-bits BITS    the pattern for --marginal (default all zeros)
//!   --eps E                 the acuity demanded (default 0)
//!   --shards S              mesh shards for the branch fold (default 1)
//!   --marginal-cap K        evaluate a marginal by summing at most K
//!                           completions inside the light cone (default 256)
//!   --max-referee N         the widest n the statevector referee runs at
//!                           (default 20)
//!   --label TEXT            echoed into the JSON line
//! ```

use holon::affine::Gate;
use holon::magic::Circuit;
use holon::sector::{
    self, referee, AcuityBackend, Budgeted, ObsValue, Observable, Sector,
};
use std::time::Instant;

// ---------------------------------------------------------------------------
// MERGE POINT — the real backend
// ---------------------------------------------------------------------------
// `holon::acuity` is built on the other half of this campaign's branch and
// carries (confirmed against that author, 2026-09-21):
//
//     pub fn budgeted_amplitude<S: BranchSource + ?Sized>(
//         src: &S, y: &[bool], eps: f64, shards: usize) -> acuity::Budgeted;
//     pub fn certified_source_for(c: &Circuit, y: &[bool]) -> Bounded<Magic5Source>;
//     pub struct BudgetPlan;   // BudgetPlan::of(&src), budgeted_amplitude_with(…)
//
// Two edits merge it, and only two:
//
//   1. in `make_source`, return `holon::acuity::certified_source_for(c, y)`
//      instead of `Magic5Source::new(c)`. Use `certified_source_for`, NOT
//      `source_for`: the a-priori coefficient bound is VACUOUS (its author
//      measured k/N = 1.000 at every eps under it), and the certified source
//      installs the tight per-branch bound that makes the budget bite.
//   2. here, add
//
//          struct Acuity;
//          impl AcuityBackend for Acuity {
//              fn name(&self) -> &'static str { "acuity::budgeted_amplitude" }
//              fn budgeted_amplitude(&self, src: &dyn BranchSource, y: &[bool],
//                                    eps: f64, shards: usize) -> Budgeted {
//                  let b = holon::acuity::budgeted_amplitude(src, y, eps, shards);
//                  Budgeted { value_f64: b.value_f64, remainder: b.remainder,
//                             evaluated: b.evaluated, total: b.total }
//              }
//          }
//
//      and return `Box::new(Acuity)` below. (`budgeted_amplitude` is generic
//      over `S: BranchSource + ?Sized`, so `&dyn BranchSource` goes straight
//      in.) A marginal should then build one `BudgetPlan::of(&src)` and call
//      `budgeted_amplitude_with` per leg — the plan does not depend on `y`,
//      and re-sorting it per completion is the only waste left in this file.
//
// Nothing else changes; the sweep's `backend` column then reads the real name
// and its two wall columns stop being the same measurement.
/// The real backend (merged 2026-09-22): the budgeted branch sum with its certified
/// remainder, on the certified source whose per-branch bound is the one that truncates.
struct Acuity;
impl AcuityBackend for Acuity {
    fn name(&self) -> &'static str { "acuity::budgeted_amplitude" }
    fn budgeted_amplitude(&self, src: &dyn holon::BranchSource, y: &[bool], eps: f64, shards: usize) -> Budgeted {
        let b = holon::acuity::budgeted_amplitude(src, y, eps, shards);
        Budgeted { value_f64: b.value_f64, remainder: b.remainder, evaluated: b.evaluated, total: b.total }
    }
}

fn backend() -> Box<dyn AcuityBackend> {
    Box::new(Acuity)
}

/// The branch source for the located circuit: the CERTIFIED one (the a-priori coefficient
/// bound is vacuous; the per-branch scalar bound is what makes the budget bite).
fn make_source(c: &Circuit, y: &[bool]) -> impl holon::BranchSource {
    holon::acuity::certified_source_for(c, y)
}

// ---------------------------------------------------------------------------
// arguments
// ---------------------------------------------------------------------------

struct Args {
    circuit: Circuit,
    source: String,
    seed: Option<u64>,
    obs: Observable,
    eps: f64,
    shards: usize,
    marginal_cap: u64,
    max_referee: usize,
    label: String,
    phase_omega: u8,
}

fn die(msg: &str) -> ! {
    eprintln!("qvm_acuity: {msg}");
    std::process::exit(2);
}

fn bits_of(s: &str) -> Vec<bool> {
    s.chars()
        .map(|c| match c {
            '0' => false,
            '1' => true,
            _ => die("a bitstring is 0s and 1s, qubit 0 first"),
        })
        .collect()
}

fn parse_args() -> Args {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut circuit: Option<Circuit> = None;
    let mut source = String::new();
    let mut seed = None;
    let mut amp: Option<String> = None;
    let mut skeleton = false;
    let mut argmax = false;
    let mut marg: Option<Vec<usize>> = None;
    let mut marg_bits: Option<String> = None;
    let mut eps = 0.0f64;
    let mut shards = 1usize;
    let mut marginal_cap = 256u64;
    let mut max_referee = 20usize;
    let mut label = String::new();
    let mut phase_omega = 0u8;
    let mut i = 0;
    while i < argv.len() {
        let next = |i: usize| -> String {
            argv.get(i + 1).cloned().unwrap_or_else(|| die("a flag is missing its value"))
        };
        match argv[i].as_str() {
            "--random" => {
                let n: usize = next(i).parse().unwrap_or_else(|_| die("--random N T SEED"));
                let t: usize =
                    argv.get(i + 2).and_then(|s| s.parse().ok()).unwrap_or_else(|| die("--random N T SEED"));
                let s: u64 =
                    argv.get(i + 3).and_then(|s| s.parse().ok()).unwrap_or_else(|| die("--random N T SEED"));
                circuit = Some(sector::random_instance(n, t, s));
                source = format!("random({n},{t},{s})");
                seed = Some(s);
                i += 4;
            }
            "--qasm" => {
                let path = next(i);
                let src = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| die(&format!("cannot read {path}: {e}")));
                let p = holon::qasm::parse(&src)
                    .unwrap_or_else(|r| die(&format!("line {}: {}", r.line, r.reason)));
                if p.residual_zeta16 != 0 {
                    die("the program carries a ζ16 residual outside Z[ω]; refused by name");
                }
                phase_omega = p.phase_omega;
                circuit = Some(Circuit { n_qubits: p.n_qubits, gates: p.gates });
                source = path;
                i += 2;
            }
            "--amp" => {
                amp = Some(next(i));
                i += 2;
            }
            "--amp-skeleton" => {
                skeleton = true;
                i += 1;
            }
            "--amp-argmax" => {
                argmax = true;
                i += 1;
            }
            "--marginal" => {
                marg = Some(
                    next(i)
                        .split(',')
                        .map(|s| s.trim().parse().unwrap_or_else(|_| die("--marginal q0,q1,…")))
                        .collect(),
                );
                i += 2;
            }
            "--marginal-bits" => {
                marg_bits = Some(next(i));
                i += 2;
            }
            "--eps" => {
                eps = next(i).parse().unwrap_or_else(|_| die("--eps wants a float"));
                i += 2;
            }
            "--shards" => {
                shards = next(i).parse().unwrap_or_else(|_| die("--shards wants an integer"));
                i += 2;
            }
            "--marginal-cap" => {
                marginal_cap = next(i).parse().unwrap_or_else(|_| die("--marginal-cap wants an integer"));
                i += 2;
            }
            "--max-referee" => {
                max_referee = next(i).parse().unwrap_or_else(|_| die("--max-referee wants an integer"));
                i += 2;
            }
            "--label" => {
                label = next(i);
                i += 2;
            }
            other => die(&format!("unknown flag {other}")),
        }
    }
    let circuit = circuit.unwrap_or_else(|| die("one of --random N T SEED or --qasm FILE"));
    let n = circuit.n_qubits;
    let obs = match (amp, marg) {
        (Some(_), Some(_)) => die("--amp and --marginal name two different observables"),
        (Some(y), None) => {
            let b = bits_of(&y);
            if b.len() != n {
                die("--amp wants one bit per qubit");
            }
            Observable::Amplitude(b)
        }
        (None, Some(q)) => {
            if q.iter().any(|&x| x >= n) {
                die("--marginal names a qubit the circuit does not have");
            }
            let bits = match marg_bits {
                Some(s) => bits_of(&s),
                None => vec![false; q.len()],
            };
            if bits.len() != q.len() {
                die("--marginal-bits wants one bit per marginal qubit");
            }
            Observable::Marginal { qubits: q, bits }
        }
        (None, None) if argmax => {
            if n > max_referee {
                die("--amp-argmax needs the referee; raise --max-referee or use --amp-skeleton");
            }
            Observable::Amplitude(sector::argmax_bitstring(n, &circuit.gates))
        }
        (None, None) if skeleton => {
            Observable::Amplitude(sector::skeleton_bitstring(&circuit.gates, n))
        }
        (None, None) => Observable::Amplitude(vec![false; n]),
    };
    if (skeleton || argmax) && !matches!(obs, Observable::Amplitude(_)) {
        die("--amp-skeleton/--amp-argmax declare an amplitude; neither combines with --marginal");
    }
    Args {
        circuit,
        source,
        seed,
        obs,
        eps,
        shards,
        marginal_cap,
        max_referee,
        label,
        phase_omega,
    }
}

// ---------------------------------------------------------------------------
// the hard part, through the seam
// ---------------------------------------------------------------------------

/// `ω^k` as a float pair — the lowering's global phase, carried back onto the
/// value and onto the referee alike so the two are comparable.
fn omega_pow_f(k: u8) -> (f64, f64) {
    let a = std::f64::consts::FRAC_PI_4 * k as f64;
    (a.cos(), a.sin())
}

fn cmul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

struct Summed {
    value: Option<ObsValue>,
    why_null: Option<String>,
    evaluated: u64,
    total: u64,
    remainder: f64,
    /// Branch-source construction (where the recursion's branches are built
    /// and cached) — part of the sum's cost, reported apart from the fold.
    wall_source: f64,
    wall_fold: f64,
}

/// Evaluate the located sector's observable through the backend.
fn run_sum(
    be: &dyn AcuityBackend,
    sec: &Sector,
    gates: &[Gate],
    obs: &Observable,
    args: &Args,
) -> Summed {
    let phase = omega_pow_f(args.phase_omega);
    match obs {
        Observable::Amplitude(y) => {
            let reduced = sec.reduced_circuit(gates);
            let t0 = Instant::now();
            let src = make_source(&reduced, y);
            let wall_source = t0.elapsed().as_secs_f64();
            let t1 = Instant::now();
            let b: Budgeted = be.budgeted_amplitude(&src, y, args.eps, args.shards);
            let wall_fold = t1.elapsed().as_secs_f64();
            Summed {
                value: Some(ObsValue::Amp(cmul(b.value_f64, phase))),
                why_null: None,
                evaluated: b.evaluated,
                total: b.total,
                remainder: b.remainder,
                wall_source,
                wall_fold,
            }
        }
        Observable::Marginal { qubits, bits } => {
            // Inside the light cone every other wire is still |0⟩ (no kept
            // gate can touch one — that is what the cone IS), so the sum runs
            // over the cone's free wires only. The prereg prices AMPLITUDES;
            // a marginal is a 2^{|L|−|S|}-fold sum of them, and that count is
            // stated rather than hidden.
            let free: Vec<usize> =
                sec.light_cone.iter().copied().filter(|q| !qubits.contains(q)).collect();
            let completions = 1u64 << free.len().min(63);
            if free.len() >= 63 || completions > args.marginal_cap {
                return Summed {
                    value: None,
                    why_null: Some(format!(
                        "a marginal on {} qubits inside a light cone of {} needs {} amplitude \
                         sums, over the --marginal-cap of {}; the sector and the price above \
                         are what this run reports",
                        qubits.len(),
                        sec.light_cone.len(),
                        completions,
                        args.marginal_cap
                    )),
                    evaluated: 0,
                    total: 0,
                    remainder: f64::NAN,
                    wall_source: 0.0,
                    wall_fold: 0.0,
                };
            }
            // The stronger reduction the cone licenses: drop EVERY out-of-cone
            // gate, so the free wires really are the only ones left alive.
            let cone_gates = sec.light_cone_circuit(gates);
            let reduced = Circuit { n_qubits: sec.n_qubits, gates: cone_gates };
            let t0 = Instant::now();
            let mut y0 = vec![false; sec.n_qubits];
            for (&q, &b) in qubits.iter().zip(bits) {
                y0[q] = b;
            }
            let src = make_source(&reduced, &y0);
            let wall_source = t0.elapsed().as_secs_f64();
            let t1 = Instant::now();
            let mut p = 0.0f64;
            let (mut ev, mut tot) = (0u64, 0u64);
            let mut rem = 0.0f64;
            for m in 0..completions {
                let mut y = vec![false; sec.n_qubits];
                for (&q, &b) in qubits.iter().zip(bits) {
                    y[q] = b;
                }
                for (j, &q) in free.iter().enumerate() {
                    y[q] = m >> j & 1 == 1;
                }
                let b = be.budgeted_amplitude(&src, &y, args.eps, args.shards);
                let v = cmul(b.value_f64, phase);
                p += v.0 * v.0 + v.1 * v.1;
                ev += b.evaluated;
                tot += b.total;
                // |‖a+r‖² − ‖a‖²| ≤ 2|a|·R + R²: the remainder on a
                // probability is not the remainder on an amplitude, and is
                // reported as the propagated bound, not as the amplitude's.
                let a = (v.0 * v.0 + v.1 * v.1).sqrt();
                rem += 2.0 * a * b.remainder + b.remainder * b.remainder;
            }
            let wall_fold = t1.elapsed().as_secs_f64();
            Summed {
                value: Some(ObsValue::Prob(p)),
                why_null: None,
                evaluated: ev,
                total: tot,
                remainder: rem,
                wall_source,
                wall_fold,
            }
        }
    }
}

// ---------------------------------------------------------------------------

fn main() {
    let args = parse_args();
    let be = backend();
    let gates = args.circuit.gates.clone();
    let n = args.circuit.n_qubits;
    let t = gates.iter().filter(|g| g.is_t()).count();

    // --- locate ---
    let t0 = Instant::now();
    let sec = sector::locate(&gates, &args.obs);
    let wall_locate = t0.elapsed().as_secs_f64();
    println!("{}", sec.line());

    // --- price, BEFORE a branch is evaluated (S2) ---
    let n_pred = sec.price();
    println!("price: t_eff={} N_pred={}", sec.t_eff, n_pred);

    if be.name() == "full-sum-fallback" {
        eprintln!(
            "qvm_acuity: backend = full-sum-fallback (holon::acuity absent in this worktree): \
             every branch evaluated, --eps {} IGNORED, remainder certified 0 because nothing \
             was left out.",
            args.eps
        );
    }

    // --- the hard part ---
    let sum = run_sum(be.as_ref(), &sec, &gates, &args.obs, &args);

    // --- the referee ---
    let (referee_val, wall_referee) = if n <= args.max_referee {
        let t2 = Instant::now();
        let phase = omega_pow_f(args.phase_omega);
        let v = match referee::value(n, &gates, &args.obs) {
            ObsValue::Amp(a) => ObsValue::Amp(cmul(a, phase)),
            p => p,
        };
        (Some(v), Some(t2.elapsed().as_secs_f64()))
    } else {
        (None, None)
    };

    let abs_error = match (sum.value, referee_val) {
        (Some(a), Some(b)) => Some(a.abs_diff(b)),
        _ => None,
    };

    let f = |x: Option<f64>| x.map_or("null".to_string(), |v| format!("{v:.17e}"));
    let json = format!(
        "{{\"source\": \"{}\", \"seed\": {}, \"label\": \"{}\", \"observable\": \"{}\", \
         \"n\": {}, \"t\": {}, \"t_eff\": {}, \"removed\": {}, \"clifford\": {}, \
         \"light_cone\": {}, \"N_pred\": {}, \"eps\": {:.17e}, \"shards\": {}, \
         \"backend\": \"{}\", \"evaluated\": {}, \"total\": {}, \"remainder\": {}, \
         \"value\": {}, \"value_null_reason\": {}, \"referee\": {}, \"abs_error\": {}, \
         \"wall_locate_s\": {:.9}, \"wall_source_s\": {:.9}, \"wall_sum_s\": {:.9}, \
         \"wall_referee_s\": {}}}",
        args.source,
        args.seed.map_or("null".to_string(), |s| s.to_string()),
        args.label,
        args.obs.describe(),
        n,
        t,
        sec.t_eff,
        sec.removed.len(),
        sec.clifford.len(),
        sec.light_cone.len(),
        n_pred,
        args.eps,
        args.shards,
        be.name(),
        sum.evaluated,
        sum.total,
        if sum.remainder.is_nan() { "null".to_string() } else { format!("{:.17e}", sum.remainder) },
        sum.value.map_or("null".to_string(), |v| v.as_json()),
        sum.why_null.map_or("null".to_string(), |w| format!("\"{w}\"")),
        referee_val.map_or("null".to_string(), |v| v.as_json()),
        f(abs_error),
        wall_locate,
        sum.wall_source,
        sum.wall_fold,
        wall_referee.map_or("null".to_string(), |w| format!("{w:.9}")),
    );
    println!("{json}");
}
