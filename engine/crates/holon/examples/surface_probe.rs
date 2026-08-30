//! SURFACE PROBE — correctness of syndrome extraction on the certified
//! row-major reference, plus the instrumentation that decides how to make it
//! fast.
//!
//! Two jobs, in this order:
//!
//! 1. CORRECTNESS. Run real syndrome-extraction rounds on a real rotated
//!    surface code and check the four things that make it a code: the Z
//!    syndromes are silent on a fresh |0…0⟩; a second noiseless round
//!    REPEATS the first exactly; an injected X error lights exactly the Z
//!    plaquettes that contain it; and after the feed-forward correction the
//!    syndromes return to their pre-error values. Every seed.
//!
//! 2. COST. The row-major measurement path has two loops whose lengths are
//!    state-dependent — the destabilizer product on a DETERMINISTIC outcome,
//!    and the rowsum cascade on a RANDOM one. Their lengths decide whether a
//!    column-major port is worth building, so they are counted here rather
//!    than guessed.
//!
//! Run: cargo run --release --example surface_probe -- <d> [<d> ...]

use holon::surface::{Kind, SurfaceCode};
use holon::tableau::{PackedTableau, PauliRow};
use std::time::Instant;

/// Counters for the two state-dependent loops.
#[derive(Default, Clone, Copy)]
struct Cost {
    det_measures: u64,
    det_terms: u64,
    det_terms_max: u64,
    rand_measures: u64,
    cascade_terms: u64,
    cascade_terms_max: u64,
}

/// `measure_peek`, instrumented: returns the outcome and counts the
/// destabilizer product's length.
fn peek(t: &PackedTableau, q: usize, cost: &mut Cost) -> Option<bool> {
    for p in t.n..2 * t.n {
        if t.rows[p].x.get(q) {
            return None;
        }
    }
    let mut scratch = PauliRow::identity(t.n);
    let mut terms = 0u64;
    for i in 0..t.n {
        if t.rows[i].x.get(q) {
            let stab = t.rows[i + t.n].clone();
            scratch.mul_assign(&stab);
            terms += 1;
        }
    }
    cost.det_measures += 1;
    cost.det_terms += terms;
    cost.det_terms_max = cost.det_terms_max.max(terms);
    Some(scratch.r % 4 == 2)
}

/// `collapse`, instrumented: counts the rowsum cascade's length.
fn collapse(t: &mut PackedTableau, q: usize, outcome: bool, cost: &mut Cost) {
    let n = t.n;
    let p = (n..2 * n)
        .find(|&p| t.rows[p].x.get(q))
        .expect("collapse requires a random measurement");
    let pivot = t.rows[p].clone();
    let mut terms = 0u64;
    for i in 0..2 * n {
        if i != p && t.rows[i].x.get(q) {
            t.rows[i].mul_assign(&pivot);
            terms += 1;
        }
    }
    t.rows[p - n] = pivot;
    let mut fresh = PauliRow::identity(n);
    fresh.z.set(q, true);
    fresh.r = if outcome { 2 } else { 0 };
    t.rows[p] = fresh;
    cost.rand_measures += 1;
    cost.cascade_terms += terms;
    cost.cascade_terms_max = cost.cascade_terms_max.max(terms);
}

fn splitmix(state: &mut u64) -> bool {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (z ^ (z >> 31)) & 1 == 1
}

/// One full syndrome-extraction round: entangle, measure, reset every ancilla.
/// Returns the syndrome vector.
fn round(
    t: &mut PackedTableau,
    code: &SurfaceCode,
    rng: &mut u64,
    cost: &mut Cost,
) -> Vec<bool> {
    let mut syn = Vec::with_capacity(code.stabs.len());
    for s in &code.stabs {
        let a = s.ancilla;
        match s.kind {
            Kind::Z => {
                for &q in &s.data() {
                    t.cx(q, a);
                }
            }
            Kind::X => {
                t.h(a);
                for &q in &s.data() {
                    t.cx(a, q);
                }
                t.h(a);
            }
        }
        let outcome = match peek(t, a, cost) {
            Some(b) => b,
            None => {
                let b = splitmix(rng);
                collapse(t, a, b, cost);
                b
            }
        };
        // Reset the ancilla to |0⟩ so the next round starts clean.
        if outcome {
            t.x_gate(a);
        }
        syn.push(outcome);
    }
    syn
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ds: Vec<usize> = if args.is_empty() {
        vec![3, 5, 7, 9, 11]
    } else {
        args.iter().filter_map(|a| a.parse().ok()).collect()
    };

    println!("SURFACE PROBE — syndrome extraction on the row-major reference");
    println!();

    for &d in &ds {
        let code = SurfaceCode::new(d);
        println!("d = {d}   n = {}   stabilizers = {}", code.n, code.stabs.len());

        for seed in 0..4u64 {
            let mut rng = seed;
            let mut cost = Cost::default();
            let mut t = PackedTableau::new(code.n);
            let t0 = Instant::now();

            // ---- round 1: establishes the codestate ----
            let s1 = round(&mut t, &code, &mut rng, &mut cost);
            // Z syndromes must be silent on a fresh |0…0⟩.
            for (i, s) in code.stabs.iter().enumerate() {
                if s.kind == Kind::Z {
                    assert!(!s1[i], "d={d} seed={seed}: Z syndrome {i} fired on |0…0⟩");
                }
            }

            // ---- round 2: noiseless repeat must reproduce round 1 exactly ----
            let s2 = round(&mut t, &code, &mut rng, &mut cost);
            assert_eq!(s2, s1, "d={d} seed={seed}: noiseless round did not repeat");

            // ---- inject a known X error on a data qubit ----
            let bad = code.data(d / 2, d / 2);
            t.x_gate(bad);
            let expect: Vec<usize> = code.z_stabs_touching(bad);
            assert!(!expect.is_empty());

            // ---- round 3: exactly the Z plaquettes containing the error fire
            let s3 = round(&mut t, &code, &mut rng, &mut cost);
            for (i, s) in code.stabs.iter().enumerate() {
                let flipped = s3[i] != s1[i];
                let should = expect.contains(&i);
                assert_eq!(
                    flipped, should,
                    "d={d} seed={seed}: stabilizer {i} ({:?}) flipped={flipped}, expected={should}",
                    s.kind
                );
            }

            // ---- feed-forward correction, then round 4 must be clean ----
            t.x_gate(bad);
            let s4 = round(&mut t, &code, &mut rng, &mut cost);
            assert_eq!(s4, s1, "d={d} seed={seed}: syndromes did not return after correction");

            let el = t0.elapsed();
            if seed == 0 {
                let dm = cost.det_measures.max(1);
                let rm = cost.rand_measures.max(1);
                println!(
                    "  seed 0 OK  {:.3} s   4 rounds, {} measurements",
                    el.as_secs_f64(),
                    cost.det_measures + cost.rand_measures
                );
                println!(
                    "    deterministic: {:6} measures, product terms mean {:7.2} max {:5}",
                    cost.det_measures,
                    cost.det_terms as f64 / dm as f64,
                    cost.det_terms_max
                );
                println!(
                    "    random:        {:6} measures, cascade terms mean {:7.2} max {:5}",
                    cost.rand_measures,
                    cost.cascade_terms as f64 / rm as f64,
                    cost.cascade_terms_max
                );
            }
        }
        println!("  all 4 seeds PASS");
        println!();
    }
    println!("done.");
}
