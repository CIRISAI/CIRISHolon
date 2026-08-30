//! Where does a deterministic measurement actually spend its time?
//!
//! The flagship's steady-state rounds read ~0.22 ms per measurement at
//! d=101, where the algorithm predicts ~10 µs. That is a 20× gap between
//! the cost model and the clock, and a cost model that wrong is not a cost
//! model. This decomposes one measurement into its three parts and times
//! each in isolation, on a real surface-code state.
//!
//! Run: cargo run --release --example measure_profile -- <d>

use holon::coladaptive::ColAdaptive;
use holon::coltableau::ColTableau;
use holon::surface::{Kind, SurfaceCode};
use holon::tableau::{PackedTableau, PauliRow};
use std::time::Instant;

fn main() {
    let d: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(101);
    let code = SurfaceCode::new(d);
    let n = code.n;
    println!("d={d} n={n}");

    // Drive one full round so the state is a real codestate.
    let mut a = ColAdaptive::new(n, 1);
    for s in &code.stabs {
        if s.kind == Kind::X {
            a.h(s.ancilla);
        }
    }
    for t in 0..4 {
        for s in &code.stabs {
            if let Some(q) = s.sched[t] {
                match s.kind {
                    Kind::Z => a.cx(q, s.ancilla),
                    Kind::X => a.cx(s.ancilla, q),
                }
            }
        }
    }
    for s in &code.stabs {
        if s.kind == Kind::X {
            a.h(s.ancilla);
        }
    }
    a.begin_batch();
    let syn: Vec<bool> = code.stabs.iter().map(|s| a.measure(s.ancilla).0).collect();
    a.end_batch();
    for (k, s) in code.stabs.iter().enumerate() {
        if syn[k] {
            a.x_gate(s.ancilla);
        }
    }
    // Second round's gate phase, so we are in the steady state.
    for s in &code.stabs {
        if s.kind == Kind::X {
            a.h(s.ancilla);
        }
    }
    for t in 0..4 {
        for s in &code.stabs {
            if let Some(q) = s.sched[t] {
                match s.kind {
                    Kind::Z => a.cx(q, s.ancilla),
                    Kind::X => a.cx(s.ancilla, q),
                }
            }
        }
    }
    for s in &code.stabs {
        if s.kind == Kind::X {
            a.h(s.ancilla);
        }
    }

    // Materialize both representations by hand so each piece can be timed.
    let t0 = Instant::now();
    let packed: PackedTableau = a.to_packed();
    let transpose_s = t0.elapsed().as_secs_f64();
    let t0 = Instant::now();
    let col = ColTableau::from_packed(&packed);
    let untranspose_s = t0.elapsed().as_secs_f64();
    println!("  to_packed   {transpose_s:8.3} s");
    println!("  from_packed {untranspose_s:8.3} s");

    let anc: Vec<usize> = code.stabs.iter().map(|s| s.ancilla).collect();
    let reps = anc.len();

    // ---- 1. the column determinism scan ----
    let t = Instant::now();
    let mut acc = 0usize;
    for &q in &anc {
        if col.first_x_row_in(q, n, 2 * n).is_some() {
            acc += 1;
        }
    }
    let scan_s = t.elapsed().as_secs_f64();
    println!(
        "  scan        {:8.3} s  ({:7.2} us/meas, {acc} random)",
        scan_s,
        scan_s * 1e6 / reps as f64
    );

    // ---- 2. collecting the destabilizer hit set ----
    let t = Instant::now();
    let mut hits = Vec::new();
    let mut total_hits = 0usize;
    for &q in &anc {
        hits.clear();
        col.x_rows_in(q, 0, n, &mut hits);
        total_hits += hits.len();
    }
    let hits_s = t.elapsed().as_secs_f64();
    println!(
        "  hit set     {:8.3} s  ({:7.2} us/meas, mean {:.2} terms)",
        hits_s,
        hits_s * 1e6 / reps as f64,
        total_hits as f64 / reps as f64
    );

    // ---- 3. the destabilizer product itself ----
    let t = Instant::now();
    let mut sink = 0u32;
    for &q in &anc {
        hits.clear();
        col.x_rows_in(q, 0, n, &mut hits);
        let mut scratch = PauliRow::identity(n);
        for &i in &hits {
            scratch.mul_assign(&packed.rows[i + n]);
        }
        sink += scratch.r as u32;
    }
    let prod_s = t.elapsed().as_secs_f64();
    println!(
        "  + product   {:8.3} s  ({:7.2} us/meas)   [sink {sink}]",
        prod_s,
        prod_s * 1e6 / reps as f64
    );

    // ---- 3b. the same, with the scratch row allocated ONCE ----
    let t = Instant::now();
    let mut scratch = PauliRow::identity(n);
    let mut sink2 = 0u32;
    for &q in &anc {
        hits.clear();
        col.x_rows_in(q, 0, n, &mut hits);
        for w in scratch.x.words.iter_mut() {
            *w = 0;
        }
        for w in scratch.z.words.iter_mut() {
            *w = 0;
        }
        scratch.r = 0;
        for &i in &hits {
            scratch.mul_assign(&packed.rows[i + n]);
        }
        sink2 += scratch.r as u32;
    }
    let prod2_s = t.elapsed().as_secs_f64();
    println!(
        "  + product (reused scratch) {:8.3} s  ({:7.2} us/meas)   [sink {sink2}]",
        prod2_s,
        prod2_s * 1e6 / reps as f64
    );

    // ---- 4. the whole thing through the engine, for comparison ----
    let mut b = ColAdaptive::new(n, 1);
    // put b in the same place as a
    b.load_state(&packed);
    let t = Instant::now();
    b.begin_batch();
    for &q in &anc {
        std::hint::black_box(b.measure(q));
    }
    b.end_batch();
    let engine_s = t.elapsed().as_secs_f64();
    println!(
        "  engine      {:8.3} s  ({:7.2} us/meas)  fast {} fallback {}",
        engine_s,
        engine_s * 1e6 / reps as f64,
        b.stats.scan_fast,
        b.stats.scan_fallback
    );
}
