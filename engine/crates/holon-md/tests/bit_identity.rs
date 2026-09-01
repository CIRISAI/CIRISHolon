//! THE GATE THIS CRATE EXISTS TO PASS: the worker count is not an input to the physics.
//!
//! A parallel force loop that accumulates into a shared array gives a different answer for
//! every scheduling, and in a chaotic system that difference grows until the trajectories
//! are unrelated. The usual response is to call it "numerical noise" and stop measuring it.
//! Here it is measured, and the requirement is EQUALITY OF BITS — against the serial
//! reference, and across 1, 2, 3, 5 and 8 workers.
//!
//! Plants:
//!
//! | plant | what it breaks | what must fire |
//! |---|---|---|
//! | P-M1 | the pool is nominally many and effectively one | the progress check |
//! | P-M2 | a lease taken and not released | the ledger identity |
//! | P-M3 | accumulation in worker order rather than term order | the bit-identity gate |

use holon_md::WorkerPool;
use holon_render::cells::Route;
use holon_render::sim::{Boundary, Dims, Sim};

fn potential_source() -> String {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../holon-render/viewer/h2_potential.json"
    );
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("cannot read {path}: {e}. Run: cargo run -p holon-render --example make_placeholder")
    })
}

/// A periodic lattice big enough that the cell list engages and the term count is worth
/// handing to a pool.
fn scene() -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    holon_render::json::load_into(s.table_mut(), &potential_source()).expect("table loads");
    s.adopt_table_timescale();
    s.dims = Dims::Three;
    s.boundary = Boundary::Periodic;
    s.width = 96.0;
    s.height = 96.0;
    s.depth = 96.0;
    let side = 12usize;
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
    assert!(
        s.set_pair_cutoff(1e-6),
        "the scene could not declare a pair cutoff, so there is no term list to shard"
    );
    assert_eq!(
        s.route(),
        Route::Cells,
        "the cell list did not engage; this scene is not exercising what it claims to"
    );
    s
}

fn fingerprint(s: &Sim) -> Vec<u64> {
    let mut out = Vec::with_capacity(s.n * 7 + 8);
    for i in 0..s.n {
        let a = &s.atoms[i];
        for v in [a.x, a.y, a.z, a.vx, a.vy, a.vz] {
            out.push(v.to_bits());
        }
    }
    for v in [s.e_kin, s.e_pair, s.e_three, s.drift_peak, s.energy()] {
        out.push(v.to_bits());
    }
    out
}

const FRAMES: usize = 6;
const SUBSTEPS: u32 = 8;

/// THE REFERENCE: the serial run, which is the engine with no executor installed at all.
fn serial_reference() -> Vec<u64> {
    let mut s = scene();
    for _ in 0..FRAMES {
        s.step_frame(SUBSTEPS);
    }
    fingerprint(&s)
}

#[test]
fn the_worker_count_is_not_an_input_to_the_physics() {
    let reference = serial_reference();
    let terms_per_evaluation = {
        let s = scene();
        s.neighbours().pairs.len()
    };
    println!("scene: {terms_per_evaluation} pair terms per force evaluation");

    for workers in [1usize, 2, 3, 5, 8] {
        let pool = match WorkerPool::new(workers) {
            Ok(p) => p,
            Err(e) => {
                // A machine that will not give us eight threads is a fact about the
                // machine, and skipping is the honest response — but it is REPORTED, not
                // swallowed, because a gate that silently skipped is a gate that passed
                // without running.
                println!("SKIPPED {workers} workers: {}", e.plain());
                continue;
            }
        };
        let mut s = scene();
        let (pool, progress) = holon_md::run_frames(&mut s, pool, FRAMES, SUBSTEPS);
        let got = fingerprint(&s);

        let first = reference
            .iter()
            .zip(got.iter())
            .position(|(a, b)| a != b);
        assert!(
            first.is_none(),
            "{workers} workers disagreed with the serial reference at word {} \
             ({:.17e} vs {:.17e}); progress {progress:?}",
            first.unwrap(),
            f64::from_bits(reference[first.unwrap()]),
            f64::from_bits(got[first.unwrap()])
        );

        // PLANT P-M1, standing: a pool of many in which one worker does everything is a
        // serial run with idle leases, and it would pass the bit-identity gate perfectly.
        //
        // The reading is over the RUN, not over one evaluation. At a chunk count near the
        // worker count, one thread can legitimately claim every chunk of a single
        // evaluation before the others have started — which is what this assertion caught
        // when it was written per-call, and what `worth_it` is the honest predicate for.
        // Deliberately NOT gated on `worth_it`. That predicate is about COST, and cost on
        // this box is not measurable while another lane holds 32 threads
        // (M-CONTENDED-BASELINE). What is being gated here is CORRECTNESS, which contention
        // cannot touch: the bits either agree or they do not, whatever the scheduler did.
        if workers > 1 {
            let busy = progress.iter().filter(|&&n| n > 0).count();
            assert!(
                busy >= 2,
                "{workers} workers were leased and {busy} did any work ({progress:?}); the \
                 identity above is trivially true because nothing ran in parallel"
            );
        }
        assert!(
            progress.iter().sum::<usize>() > 0,
            "the pool evaluated no terms at all"
        );

        // The books balance before the pool is retired, and after.
        assert!(pool.balances(), "ledger: {:?}", pool.ledger());
        let ledger = pool.retire();
        assert_eq!(ledger.opened, workers as u64);
        assert_eq!(ledger.released, workers as u64);
        assert_eq!(ledger.convicted, 0);
        assert!(
            ledger.rent.0 > 0,
            "the workers paid no rent, so nothing recorded that they worked"
        );
    }
}

/// The pool's own refusals. A pool is built or it is not; there is no partial pool, because
/// a caller that asked for eight and silently got three is running a different experiment
/// from the one it described.
#[test]
fn a_pool_of_nothing_is_refused() {
    match WorkerPool::new(0) {
        Err(e) => assert!(e.plain().contains("not a pool")),
        Ok(_) => panic!("a pool of zero workers was built"),
    }
}

/// PLANT P-M2: the ledger identity is what convicts a leak, and it must be able to. A pool
/// that is never retired holds its leases, and the arena says so — `live` is non-zero and
/// `released` is short.
#[test]
fn p_m2_an_unretired_pool_is_visible_in_its_own_books() {
    let Ok(pool) = WorkerPool::new(2) else {
        println!("SKIPPED: the OS refused two threads");
        return;
    };
    let l = pool.ledger();
    assert_eq!(l.opened, 2);
    assert_eq!(l.released, 0, "nothing has been released yet");
    assert_eq!(
        l.live(),
        Some(2),
        "two leases are outstanding and the ledger does not say so"
    );
    assert!(pool.balances());
    let after = pool.retire();
    assert_eq!(after.released, 2);
    assert_eq!(after.live(), Some(0));
}

/// The `worth_it` predicate is advice the caller can read, not a silent fallback. A pool
/// that quietly ran a small job serially would make every small-scene benchmark a
/// measurement of the fallback rather than of the pool.
#[test]
fn the_pool_says_when_it_is_not_worth_using() {
    let Ok(pool) = WorkerPool::new(4) else {
        println!("SKIPPED: the OS refused four threads");
        return;
    };
    assert!(!pool.worth_it(100), "a hundred terms is not a parallel job");
    assert!(pool.worth_it(1_000_000));
    pool.retire();
}
