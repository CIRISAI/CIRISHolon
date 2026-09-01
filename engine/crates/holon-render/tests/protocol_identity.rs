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
//!
//! ## The second gate, and the near miss that bought it
//!
//! Byte-comparing the block is NOT enough, because the physics is not all inside it.
//! `waterquench.rs` sets `de4_enabled = true` in its own `main`, well below the block's
//! closing banner. When the exact four-body work landed, the block was updated in both
//! files (a `MAX_ATOMS` to `DEFAULT_SCENE_ATOMS` rename) and the gate passed — while
//! `waterquench_traj` silently kept `Sim::empty()`'s `de4_enabled: false`.
//!
//! Regenerating "the dE4 seed" with that runner would have run the four-body term SWITCHED
//! OFF, produced a trajectory of different physics under the right filename, and let a
//! closure census report a failure to certify. Every number would have looked reasonable.
//!
//! So the second test below inventories the PHYSICS KNOBS each runner touches and requires
//! the trajectory runner to name every one its reference names. A knob it does not name is
//! a knob it silently defaults, and defaults are how a runner stops being the thing it is
//! standing in for.

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

/// Every `base.<field> = ...` physics assignment a runner makes, as a set of field names.
///
/// Deliberately syntactic and deliberately over-broad: it is cheap to add a knob to the
/// trajectory runner and expensive to discover months later that one was missing.
fn physics_knobs(src: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for line in src.lines() {
        let t = line.trim();
        if t.starts_with("//") {
            continue;
        }
        if let Some(rest) = t.strip_prefix("base.") {
            if let Some(field) = rest.split(['=', '.', ';', ' ']).next() {
                if !field.is_empty() {
                    out.insert(field.to_string());
                }
            }
        }
    }
    out
}

/// G1b — THE PHYSICS-KNOB GATE.
///
/// The trajectory runner must NAME every scene field its reference runner sets. It may set
/// more (it takes `--ozone` and `--de4` explicitly where `waterquench` hardcodes them), but
/// it may never set FEWER: a field it does not mention is one it silently inherits from
/// `Sim::empty()`, and `Sim::empty()` is not the protocol.
#[test]
fn the_trajectory_runner_names_every_physics_knob_its_reference_sets() {
    let reference = physics_knobs(QUENCH);
    let traj = physics_knobs(TRAJ);

    // Work count: a gate that found no knobs would pass silently forever.
    assert!(
        reference.len() >= 6,
        "only {} knobs found in waterquench.rs -- the scan has stopped seeing them: {:?}",
        reference.len(),
        reference
    );

    let missing: Vec<&String> = reference.difference(&traj).collect();
    assert!(
        missing.is_empty(),
        "waterquench_traj.rs does not set {missing:?}, which waterquench.rs does. \
         Whatever it does not set it inherits from Sim::empty(), so a run of it is a run of \
         DIFFERENT PHYSICS under the same protocol banner. This gate exists because \
         `de4_enabled` went missing exactly this way and would have regenerated the dE4 \
         seed with the four-body term switched off.\n  reference sets: {reference:?}\n  \
         traj sets     : {traj:?}"
    );
}
