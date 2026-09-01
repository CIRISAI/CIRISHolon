//! G1 — THE INSTRUMENT-IDENTITY GATE.
//!
//! `waterquench_traj` re-runs `waterquench`'s protocol in order to leave a trajectory
//! behind, and it does so by carrying a VERBATIM copy of the frozen-protocol block rather
//! than by adding a flag to a file another lane was editing. A copy is a liability the
//! moment nobody checks it, so this is the check: the block between the
//! `THE FROZEN PROTOCOL` banner and the `the run` banner must be byte-identical in both
//! sources.
//!
//! It covers the eight staked seeds, the box, the temperatures, the thermostat coupling,
//! the frame and substep counts, the jitter, the knot count, the RNG, the placement, and
//! the whole measurement rule — every number a reported run re-runs on. If any lane
//! changes one of them in either file, a census computed against the other's banked log is
//! no longer comparable, and this test says so before the census does.
//!
//! M-STALE-INSTRUMENT is the misfit this discharges: an instrument that has drifted from
//! the record it is being compared against produces a verdict nobody can reproduce.

const QUENCH: &str = include_str!("../examples/waterquench.rs");
const TRAJ: &str = include_str!("../examples/waterquench_traj.rs");

const OPEN: &str = "// ================================================================ THE FROZEN PROTOCOL";
const CLOSE: &str = "// ================================================================ the run";

fn frozen_block<'a>(src: &'a str, what: &str) -> &'a str {
    let start = src
        .find(OPEN)
        .unwrap_or_else(|| panic!("{what}: the FROZEN PROTOCOL banner is missing"));
    let end = src[start..]
        .find(CLOSE)
        .unwrap_or_else(|| panic!("{what}: the closing banner is missing"))
        + start;
    &src[start..end]
}

#[test]
fn the_frozen_protocol_block_is_byte_identical_in_both_runners() {
    let a = frozen_block(QUENCH, "waterquench.rs");
    let b = frozen_block(TRAJ, "waterquench_traj.rs");

    // The work count, so a passing gate cannot be a gate that found nothing to compare
    // (M-VACUOUS-SUCCESS). The block is the whole protocol; if it ever shrinks to a few
    // lines the comparison has stopped meaning anything.
    assert!(
        a.len() > 6000,
        "the extracted block is only {} bytes; the banners have moved and this gate is \
         no longer reading the protocol",
        a.len()
    );
    assert!(a.contains("const SEEDS: [u64; 8]"));
    assert!(a.contains("const FRAMES: usize"));
    assert!(a.contains("fn place("));
    assert!(a.contains("fn reading("));

    if a == b {
        return;
    }
    // Name the first divergence rather than dumping two files at the reader.
    let (mut line, mut col) = (1usize, 1usize);
    for (x, y) in a.chars().zip(b.chars()) {
        if x != y {
            break;
        }
        if x == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    let ctx = |s: &str| {
        s.lines()
            .nth(line - 1)
            .unwrap_or("<past end of block>")
            .to_string()
    };
    panic!(
        "the frozen protocol has DRIFTED between the two runners, first at block line \
         {line} column {col}:\n  waterquench.rs      : {}\n  waterquench_traj.rs : {}\n\
         Reconcile them before trusting any census computed against a banked log.",
        ctx(a),
        ctx(b)
    );
}
