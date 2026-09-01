//! THE MEASURED SCALING: what the pool actually buys, and where it stops buying anything.
//!
//! `WorkerPool::worth_it` is a threshold, and a threshold nobody measured is a guess with a
//! constant in front of it. This example measures the two things behind it — the speedup at
//! each worker count, and the scene size at which the thread starts stop being noise — and
//! prints them so the constant can be set from a reading rather than from an intuition.
//!
//! Run: `cargo run -p holon-md --release --example scaling`
//!
//! M-PLACEMENT-LOTTERY and M-CONTENDED-BASELINE both apply and neither is discharged here:
//! this reports a wall-clock ratio on whatever cores the scheduler gave it, on a box that
//! is running other lanes' work. Treat the SHAPE as the reading and the absolute numbers as
//! contended. A citable timing needs a quiet, pinned box and a declared core class.

use holon_md::WorkerPool;
use holon_render::sim::{Boundary, Dims, Sim};
use std::time::Instant;

fn scene(side: usize) -> Box<Sim> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../holon-render/viewer/h2_potential.json"
    );
    let src = std::fs::read_to_string(path).expect("the placeholder curve is shipped");
    let mut s = Box::new(Sim::empty());
    holon_render::json::load_into(s.table_mut(), &src).expect("table loads");
    s.adopt_table_timescale();
    s.dims = Dims::Three;
    s.boundary = Boundary::Periodic;
    let edge = side as f64 * 8.0;
    s.width = edge;
    s.height = edge;
    s.depth = edge;
    let n = side * side * side;
    s.resize_storage(n);
    for i in 0..n {
        let (ix, iy, iz) = (i % side, (i / side) % side, i / (side * side));
        s.atoms[i].x = (ix as f64 + 0.5) * 8.0;
        s.atoms[i].y = (iy as f64 + 0.5) * 8.0;
        s.atoms[i].z = (iz as f64 + 0.5) * 8.0;
        let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        s.atoms[i].vx = sign * 0.003;
        s.atoms[i].vy = sign * 0.001;
        s.atoms[i].vz = sign * 0.002;
    }
    s.sync_species();
    s.rebase();
    assert!(s.set_pair_cutoff(1e-6), "no pair cutoff could be derived");
    s
}

const FRAMES: usize = 4;
const SUBSTEPS: u32 = 8;

fn main() {
    println!(
        "chunk = {} terms;  reported parallelism = {}",
        holon_render::sim::FORCE_CHUNK,
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(0)
    );

    for side in [6usize, 8, 10, 12, 14, 16] {
        let probe = scene(side);
        let n = probe.n;
        let terms = probe.neighbours().pairs.len();
        drop(probe);

        // The serial reference, timed the same way.
        let mut s = scene(side);
        let t0 = Instant::now();
        for _ in 0..FRAMES {
            s.step_frame(SUBSTEPS);
        }
        let serial = t0.elapsed().as_secs_f64();
        drop(s);

        print!("N = {n:>5}  terms = {terms:>7}  serial {serial:>7.3} s   ");
        for workers in [2usize, 4, 8] {
            let Ok(pool) = WorkerPool::new(workers) else {
                print!("  {workers}w: refused");
                continue;
            };
            let mut s = scene(side);
            let t0 = Instant::now();
            let (pool, progress) = holon_md::run_frames(&mut s, pool, FRAMES, SUBSTEPS);
            let par = t0.elapsed().as_secs_f64();
            let busy = progress.iter().filter(|&&x| x > 0).count();
            print!("  {workers}w: {:>5.2}x ({busy} busy)", serial / par);
            pool.retire();
        }
        println!();
    }
}
