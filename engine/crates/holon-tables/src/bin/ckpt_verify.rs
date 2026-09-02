//! Is this checkpoint actually replayable? Answered against the real log, read-only.
//!
//! # Why this exists
//!
//! The resume path is proven in `tests/checkpoint.rs`: a resumed run is bit-identical to an
//! uninterrupted one, a complete log solves zero nodes, torn regions are discarded. Those
//! tests run on a 4x4x3 grid where a region is a handful of nodes and completes in
//! milliseconds.
//!
//! A production log is a different scene. Its regions carry ~1400 records each, its regime
//! line is long and contains spaces and quotes, and it is being appended to by a live
//! process while anyone reading it does so concurrently. **Every one of those is a way the
//! parse could fail that a 4x4x3 grid cannot express**, and the campaign has already paid
//! once for a test that was non-vacuous on its statistic and vacuous on its scene — the
//! region-granularity defect, which no unit test could see because a tiny grid has no wall
//! clock in it.
//!
//! So this reads the log the way a resume would, reports what it found, and recomputes every
//! committed region's digest from its own records. It opens the file read-only and writes
//! nothing, so it is safe to point at a RUNNING generation: the worst it can see is a region
//! mid-append, which it reports as torn exactly as a resume would.
//!
//! It is also the operator's answer to "can I rely on this checkpoint?", which before this
//! could only be established by killing the run and watching what happened.
//!
//! Run: `ckpt_verify <checkpoint-path> [--expect-regime <string>]`

use holon_tables::checkpoint::Checkpoint;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(path) = args.get(1) else {
        eprintln!("usage: ckpt_verify <checkpoint-path> [--expect-regime <string>]");
        std::process::exit(2);
    };
    let p = Path::new(path);
    if !p.exists() {
        eprintln!("ckpt_verify: {path} does not exist");
        std::process::exit(2);
    }

    // The regime the caller claims to be. Default: whatever the log says, which makes this a
    // parse check rather than a match check. Pass --expect-regime to make it both.
    let expect = args
        .iter()
        .position(|a| a == "--expect-regime")
        .and_then(|i| args.get(i + 1).cloned());

    let raw = std::fs::read_to_string(p).unwrap_or_default();
    let logged = raw
        .lines()
        .find(|l| l.starts_with("REGIME "))
        .map(|l| l.trim_start_matches("REGIME ").trim().to_string());

    println!("=== ckpt_verify: {path} ===");
    println!("size            {} bytes", raw.len());
    match &logged {
        Some(r) => println!("regime          {r}"),
        None => println!("regime          ABSENT — this log predates the regime line, or was truncated"),
    }

    // Open exactly as a resume would, with the log's own regime unless the caller named one.
    let probe = expect.clone().or_else(|| logged.clone()).unwrap_or_default();
    let c = match Checkpoint::open(p, &probe) {
        Ok(c) => c,
        Err(e) => {
            println!("\nREFUSED by the resume path:\n  {e}");
            println!("\nVERDICT: this checkpoint would NOT be replayed by a run declaring that regime.");
            std::process::exit(1);
        }
    };

    let regions = c.replayed_regions();
    let nodes = c.replayed_nodes();
    let torn = c.torn_regions();
    println!("\nreplayable      {regions} region(s), {nodes} node(s)");
    println!("torn            {torn} region(s) — incomplete or digest-mismatched, would be RE-SOLVED");
    if torn > 0 {
        println!("                (one torn region is EXPECTED against a live run: the generator is");
        println!("                 mid-append, and a resume would re-solve exactly that region.)");
    }

    if regions == 0 {
        println!("\nVERDICT: nothing to replay. Not a defect if the run is young — but a resume");
        println!("         right now would start from zero, so the survivability claim is not yet");
        println!("         cashed for this log.");
        return;
    }

    // The digests recompute, or `Checkpoint::open` would have called them torn — so this is
    // reporting the shape of what survived rather than re-deriving the check. What it adds is
    // the DISTRIBUTION, which is the thing a tiny test grid cannot show: whether the regions
    // are the size the grid says they should be.
    let mut sizes: Vec<usize> = Vec::new();
    for r in 0..u32::MAX {
        match c.region(r) {
            Some(v) => sizes.push(v.len()),
            None => {
                if sizes.len() == regions {
                    break;
                }
            }
        }
        if r > 100_000 {
            break;
        }
    }
    if !sizes.is_empty() {
        sizes.sort_unstable();
        let total: usize = sizes.iter().sum();
        println!(
            "region sizes    min {} median {} max {} (mean {:.0})",
            sizes[0],
            sizes[sizes.len() / 2],
            sizes[sizes.len() - 1],
            total as f64 / sizes.len() as f64
        );
        // A region of one node is the shape that says the grid was sharded far finer than
        // intended — the opposite of the v2 defect and just as worth seeing.
        if sizes[0] <= 1 {
            println!("                WARNING: a region of {} node(s). Check the region shape.", sizes[0]);
        }
    }

    println!("\nVERDICT: this checkpoint is REPLAYABLE. A run declaring the same regime would");
    println!("         skip {regions} region(s) and {nodes} solved node(s), and re-solve the rest.");
}
