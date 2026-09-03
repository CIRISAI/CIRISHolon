//! Produce the committed (O, H, H) three-body table.
//!
//! # Why an artifact rather than a build at load
//!
//! `trimer.rs`'s H3 table is generated in the browser at page load: nine determinants on
//! a bespoke s-only path, 7,293 solves, under a second. One (O, H, H) point is 441
//! determinants through the general N-centre route and costs about 50 ms, so the same
//! table here is tens of minutes of arithmetic. It is therefore generated natively, once,
//! and committed — and the fence that keeps that from turning the sandbox back into a
//! PLAYER of someone's curve is `tests/water.rs`, which recomputes a staked subset of the
//! nodes through [`holon_chem::water::stream_water_table`] and requires the committed
//! bytes to be bit-identical to what this crate's own solver produces today.
//!
//! Threads live HERE and not in the crate: `holon-chem` is linked into a browser artifact
//! and has no threads and no dependencies, and this producer is a native tool.
//!
//! ```text
//! cargo run --release -p holon-chem --example s2_table -- [threads] [out_path]
//! ```

use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point};
use holon_chem::water::{
    self, de3_with, node_index, node_r, node_u, N_NODES, N_SOLVED, NR, NU,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

fn main() {
    let mut args = std::env::args().skip(1);
    let threads: usize = args
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);
    // this producer owns the machine and runs its own pool: split the cores between
    // the pool and the lane kernel beneath it, or the two multiply (scheduling only)
    holon_chem::lanes::set_lane_threads_for_pool(threads);
    let out = args.next().unwrap_or_else(|| {
        format!(
            "{}/tests/data/s2_water_table.txt",
            env!("CARGO_MANIFEST_DIR")
        )
    });

    let t0 = Instant::now();
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    let v_cache: Vec<f64> = (0..NR)
        .map(|i| pair_point(OXYGEN, HYDROGEN, node_r(i)).e)
        .collect();
    println!("# grid {}", water::grid_line());
    println!("# E(O) = {e_o:.17}  E(H) = {e_h:.17}");
    println!("# {N_SOLVED} solves ({N_NODES} nodes), {threads} threads");

    // One row `i` per unit of work: row `i` owns every `(i, j >= i, k)`. Rows are handed
    // out dynamically because they are wildly unequal in size — row 0 is `NR * NU`
    // points and the last is `NU`.
    let vals: Vec<Mutex<f64>> = (0..N_NODES).map(|_| Mutex::new(0.0)).collect();
    let done = AtomicUsize::new(0);
    let next = AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                if i >= NR {
                    break;
                }
                for j in i..NR {
                    let (x, y) = (node_r(i), node_r(j));
                    for k in 0..NU {
                        let d = de3_with(x, y, node_u(k), e_o, e_h, v_cache[i], v_cache[j]);
                        *vals[node_index(i, j, k)].lock().unwrap() = d;
                        *vals[node_index(j, i, k)].lock().unwrap() = d;
                    }
                }
                let n = done.fetch_add(1, Ordering::SeqCst) + 1;
                println!(
                    "  row {i:3} done ({n}/{NR} rows, {:.0} s elapsed)",
                    t0.elapsed().as_secs_f64()
                );
            });
        }
    });
    let vals: Vec<f64> = vals.into_iter().map(|m| m.into_inner().unwrap()).collect();

    let mut t = water::WaterTable::empty();
    t.begin();
    let mut peak = 0.0f64;
    for (i, v) in vals.iter().enumerate() {
        peak = peak.max(v.abs());
        assert!(t.knot(i, *v), "node {i} refused: {v}");
    }
    let meta = water::WaterMeta {
        e_o_atom: e_o,
        e_h_atom: e_h,
        peak,
        solves: N_SOLVED,
        ..water::WaterMeta::empty()
    };
    assert!(t.finish(meta), "the table did not close");

    let text = water::to_text(&t);
    std::fs::write(&out, &text).unwrap_or_else(|e| panic!("cannot write {out}: {e}"));
    println!(
        "\nwrote {out} ({} bytes) in {:.0} s\n  peak |dE3|            = {peak:.6e} Ha\n  \
         curvature_envelope   = {:.6e} Ha/bohr^2\n  curvature_per_gradient = {:.6e} /bohr\n  \
         sort_kink            = {:.3e} Ha/bohr",
        text.len(),
        t0.elapsed().as_secs_f64(),
        t.curvature_envelope,
        t.curvature_per_gradient,
        t.sort_kink
    );

    // Round-trip immediately: a producer that cannot read its own output back is a
    // producer of something else.
    let back = water::from_text(&text).expect("the artifact reloads");
    for i in 0..NR {
        for j in 0..NR {
            for k in 0..NU {
                assert_eq!(
                    t.node(i, j, k).to_bits(),
                    back.node(i, j, k).to_bits(),
                    "node ({i}, {j}, {k}) did not survive the round trip"
                );
            }
        }
    }
    println!("  round trip           = bit-identical on all {N_NODES} nodes");
}
