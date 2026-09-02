//! A resumed run must be the SAME ARTIFACT, not merely a correct one.
//!
//! The claim this file exists to prove, rather than assert: a generation interrupted and
//! restarted from its checkpoint produces a table bit-identical to one that ran straight
//! through — same `table_bytes`, same digest, same iteration counts, same exit codes.
//!
//! It matters because the cheap version of resume does not have this property, though not
//! for the reason first given here: `{:.16e}` round-trips f64 exactly (measured, 2,999,033
//! values, zero failures — seventeen significant digits is the round-trip width), so the
//! sibling `s2_ozone_table`'s decimal log is not the problem. Its problem is that it
//! replays PER KNOT while updating its warm-start carrier only when it actually solves, so
//! a replayed knot leaves the next one starting from a stale vector. Different start,
//! different last bits. Region granularity is what rules that out here, and the test has to
//! be an equality rather than a tolerance because a tolerance would pass for a table that
//! is merely close.
//!
//! Region granularity is the load-bearing choice and the second test is its warrant. Under
//! `CanonicalChain` a region's first solved node is cold and every later one warm-starts
//! from its canonical predecessor IN THAT REGION, so replaying a PARTIAL region would hand
//! its successor a different starting vector and move the last bits. Regions are therefore
//! all-or-nothing, and an uncommitted one is re-solved from cold.

use holon_tables::checkpoint::Checkpoint;
use holon_tables::generate::{generate_surface, SurfaceSpec, WarmPolicy};
use holon_tables::grid::{Axis, NdGrid, Serpentine};
use holon_tables::surface::{Realised, Surface};
use holon_chem::elements::{Species, HYDROGEN};

/// A cheap deterministic stand-in: no electronic structure, because what is under test is
/// the checkpoint's fidelity, not the solver's. It still goes through `solve_surface_node`,
/// so the records carry real iteration counts and exit codes.
struct Tiny {
    species: [Species; 3],
}

impl Surface for Tiny {
    fn species(&self) -> &[Species] {
        &self.species
    }
    fn dim(&self) -> usize {
        3
    }
    fn realise(&self, c: &[f64]) -> Realised {
        Realised::Geometry(vec![
            [0.0, 0.0, 0.0],
            [c[0], 0.0, 0.0],
            [c[1] * c[2], c[1] * (1.0 - c[2] * c[2]).max(0.0).sqrt(), 0.0],
        ])
    }
    fn subtract(&self, _c: &[f64], e: f64) -> f64 {
        e
    }
    fn basis(&self) -> &'static str {
        // This stand-in stores the TOTAL, and says so. Naming it matters even here: the
        // basis is one of the three axes the regime line refuses to mix across, so a test
        // surface that lied about it would let a mismatched replay through the one gate
        // built to stop it.
        "E_total (this test surface subtracts nothing)"
    }
}

fn grid() -> NdGrid {
    NdGrid::new(vec![
        Axis::linear(4, 1.6, 2.4, 2),
        Axis::linear(4, 1.6, 2.4, 2),
        Axis::linear(3, -0.4, 0.4, 2),
    ])
    .with_serpentine(Serpentine::Reflected)
}

fn tmp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("de4ckpt_{}_{}", std::process::id(), name));
    let _ = std::fs::create_dir_all(&d);
    d.join("c.log")
}

#[test]
fn a_resumed_run_is_bit_identical_to_an_uninterrupted_one() {
    let s = Tiny { species: [HYDROGEN; 3] };

    // The reference: straight through, no checkpoint at all.
    let plain = generate_surface(&SurfaceSpec::new(&s, grid()), 3);

    // Run A: checkpointing, but killed after the FIRST region commits. There is no way to
    // kill a function mid-call from inside a test, so the interruption is simulated the way
    // a kill actually leaves the log — by truncating it to its first complete region, which
    // is exactly the state `Checkpoint::open` has to cope with in production.
    let p = tmp("resume");
    let _ = std::fs::remove_file(&p);
    {
        let c = Checkpoint::open(&p, "test").unwrap();
        let _ = generate_surface(&SurfaceSpec::new(&s, grid()).with_checkpoint(Some(&c)), 3);
    }
    let full = std::fs::read_to_string(&p).unwrap();
    let first_end = full.find("\nEND ").expect("at least one region committed");
    let cut = &full[..first_end + full[first_end..].find('\n').unwrap() + 1];
    // Add a torn region after it: a BEGIN with nodes and no END, which is what a process
    // killed mid-region leaves.
    let torn_tail = full[cut.len()..]
        .lines()
        .take(3)
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&p, format!("{cut}{torn_tail}\n")).unwrap();

    // Run B: resume from that partial log.
    let c = Checkpoint::open(&p, "test").unwrap();
    assert_eq!(c.replayed_regions(), 1, "exactly one region should replay");
    assert!(
        c.torn_regions() >= 1,
        "the deliberately torn tail must be REPORTED as torn, not replayed"
    );
    assert!(c.replayed_nodes() > 0, "the replayed region must carry nodes");
    let resumed = generate_surface(&SurfaceSpec::new(&s, grid()).with_checkpoint(Some(&c)), 3);

    assert_eq!(
        resumed.replayed,
        c.replayed_nodes(),
        "the outcome must report exactly the nodes it did not solve"
    );
    assert!(
        resumed.replayed > 0,
        "a resume that replayed nothing tests nothing"
    );
    // THE CLAIM. Byte equality over every field of every record, and the digest with it.
    assert_eq!(
        resumed.table_bytes(),
        plain.table_bytes(),
        "a resumed run produced a DIFFERENT table from the uninterrupted one"
    );
    assert_eq!(resumed.digest(), plain.digest(), "the digests diverged");
    assert!(resumed.certificate.is_clean());
    let _ = std::fs::remove_file(&p);
}

#[test]
fn checkpointing_does_not_move_the_table_when_nothing_is_replayed() {
    // The control for the test above. If merely ATTACHING a checkpoint changed the table,
    // the equality there would be comparing two altered runs and would pass for the wrong
    // reason.
    let s = Tiny { species: [HYDROGEN; 3] };
    let plain = generate_surface(&SurfaceSpec::new(&s, grid()), 3);
    let p = tmp("fresh");
    let _ = std::fs::remove_file(&p);
    let c = Checkpoint::open(&p, "test").unwrap();
    assert_eq!(c.replayed_regions(), 0);
    let written = generate_surface(&SurfaceSpec::new(&s, grid()).with_checkpoint(Some(&c)), 3);
    assert_eq!(written.replayed, 0, "a fresh run must replay nothing");
    assert_eq!(written.table_bytes(), plain.table_bytes());
    assert_eq!(written.digest(), plain.digest());
    let _ = std::fs::remove_file(&p);
}

#[test]
fn a_replayed_run_is_bit_identical_at_every_worker_count() {
    // Resume must not reintroduce the schedule dependence the crate exists to rule out: a
    // run resumed on 1 worker and the same log resumed on 8 must agree.
    let s = Tiny { species: [HYDROGEN; 3] };
    let plain = generate_surface(&SurfaceSpec::new(&s, grid()), 1);

    let p = tmp("workers");
    let _ = std::fs::remove_file(&p);
    {
        let c = Checkpoint::open(&p, "test").unwrap();
        let _ = generate_surface(&SurfaceSpec::new(&s, grid()).with_checkpoint(Some(&c)), 2);
    }
    for w in [1usize, 4, 8] {
        let c = Checkpoint::open(&p, "test").unwrap();
        assert!(c.replayed_regions() > 0);
        let r = generate_surface(&SurfaceSpec::new(&s, grid()).with_checkpoint(Some(&c)), w);
        assert_eq!(
            r.table_bytes(),
            plain.table_bytes(),
            "resuming on {w} worker(s) produced a different table"
        );
    }
    let _ = std::fs::remove_file(&p);
}

#[test]
fn the_committed_log_replays_the_whole_table_and_solves_nothing() {
    // The far end: a log with every region in it must reproduce the table with ZERO solves.
    // This is what makes the checkpoint a checkpoint rather than a cache that still pays.
    let s = Tiny { species: [HYDROGEN; 3] };
    let plain = generate_surface(&SurfaceSpec::new(&s, grid()), 3);
    let p = tmp("complete");
    let _ = std::fs::remove_file(&p);
    {
        let c = Checkpoint::open(&p, "test").unwrap();
        let _ = generate_surface(&SurfaceSpec::new(&s, grid()).with_checkpoint(Some(&c)), 3);
    }
    let c = Checkpoint::open(&p, "test").unwrap();
    assert_eq!(c.torn_regions(), 0, "a clean run must leave no torn region");
    let r = generate_surface(&SurfaceSpec::new(&s, grid()).with_checkpoint(Some(&c)), 3);
    let solved = r.records.len() - r.mirrored - r.replayed;
    assert_eq!(solved, 0, "a fully committed log still solved {solved} node(s)");
    assert_eq!(r.table_bytes(), plain.table_bytes());
    assert_eq!(r.digest(), plain.digest());
    let _ = std::fs::remove_file(&p);
}

#[test]
fn a_log_from_another_regime_is_refused_not_mixed() {
    // The regime line REFUSES rather than warning, and rather than silently discarding.
    // A table assembled out of two regimes passes every bit-identity gate in this
    // repository and is still two artifacts -- SATURATION-3 G2 measured two device classes
    // agreeing to 3.033e-15 with 91.0% of 207,025 entries differing bitwise. A silent
    // DISCARD would be almost as bad in the other direction: it would quietly re-solve the
    // whole run while reporting success.
    let s = Tiny { species: [HYDROGEN; 3] };
    let p = tmp("regime");
    let _ = std::fs::remove_file(&p);
    {
        let c = Checkpoint::open(&p, "device=cpu budget=1200").unwrap();
        let _ = generate_surface(&SurfaceSpec::new(&s, grid()).with_checkpoint(Some(&c)), 2);
    }
    // Same path, different regime: must refuse.
    let msg = match Checkpoint::open(&p, "device=gpu budget=1200") {
        Ok(_) => panic!("a log from another regime must be REFUSED, and it opened"),
        Err(e) => format!("{e}"),
    };
    assert!(msg.contains("DIFFERENT REGIME"), "the refusal must say why: {msg}");
    assert!(msg.contains("device=cpu"), "the refusal must name what the log holds: {msg}");
    assert!(msg.contains("device=gpu"), "the refusal must name what this run is: {msg}");
    // CONTROL: the matching regime still opens and still replays, or the refusal above
    // would be a gate that rejects everything.
    let ok = Checkpoint::open(&p, "device=cpu budget=1200").expect("matching regime opens");
    assert!(ok.replayed_regions() > 0, "the matching regime must still replay");
    let _ = std::fs::remove_file(&p);
}
