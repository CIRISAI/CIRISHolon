//! The row-count gate for GANTT node D's availability table.
//!
//! `conformance/atomworld/PERIODIC_AVAILABILITY.md` is the stdout of
//! `examples/periodic_availability.rs`. A generated document is only as trustworthy as the
//! claim that its generator still runs and still covers everything, so this gate asserts
//! exactly that and nothing more: the example EXECUTES, and it emits one row per registered
//! species.
//!
//! # Why the count is checked twice, against two different things
//!
//! The example prints the rows and separately prints a `# rows N` trailer. Checking only
//! the trailer would grade the generator's arithmetic against its own arithmetic; checking
//! only the parsed rows would miss a trailer that has gone stale beside them. Both are
//! compared to `ALL_ELEMENTS.len()`, so a disagreement between the table and its own
//! summary is a failure rather than a thing a reader is expected to notice.
//!
//! This gate does NOT check the classifications. Those are arithmetic over registry data
//! and the crate's own constants; a test asserting them here would be a second copy of the
//! generator, and two copies of one computation agree with each other for as long as they
//! are edited together.

use std::process::Command;

use holon_chem::elements::ALL_ELEMENTS;

/// Run the generator and hand back its stdout.
fn generate() -> String {
    let mut cmd = Command::new(env!("CARGO"));
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["run", "-q", "-p", "holon-chem", "--example", "periodic_availability"]);
    // Match the profile this test itself was built with, so the nested invocation reuses
    // the artifacts the surrounding `cargo test` already produced instead of building a
    // second copy of the crate under the other profile.
    if !cfg!(debug_assertions) {
        cmd.arg("--release");
    }
    let out = cmd.output().expect("could not spawn cargo to run the generator");
    assert!(
        out.status.success(),
        "examples/periodic_availability.rs did not run: exit {:?}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("generator emitted non-UTF-8")
}

/// A data row is one whose first field is a nuclear charge. Comments start with `#`, the
/// column header starts with `Z`, and blank lines are neither.
fn is_data_row(line: &str) -> bool {
    line.split_whitespace()
        .next()
        .is_some_and(|tok| tok.parse::<u32>().is_ok())
}

#[test]
fn the_generator_runs_and_covers_every_registered_species() {
    let stdout = generate();
    let expected = ALL_ELEMENTS.len();

    let rows = stdout.lines().filter(|l| is_data_row(l)).count();
    assert_eq!(
        rows, expected,
        "the availability table has {rows} rows but the registry has {expected} species. \
         Regenerate conformance/atomworld/PERIODIC_AVAILABILITY.md — a species was added or \
         removed and the document no longer covers the registry."
    );

    let trailer = stdout
        .lines()
        .find_map(|l| l.strip_prefix("# rows "))
        .and_then(|n| n.trim().parse::<usize>().ok())
        .expect("the generator printed no `# rows N` trailer");
    assert_eq!(
        trailer, expected,
        "the generator's own `# rows` trailer says {trailer} against {expected} registered \
         species: the table and its summary disagree."
    );

    // Every registered species appears by symbol. A row count alone would be satisfied by
    // fifty-four copies of hydrogen.
    for sp in ALL_ELEMENTS {
        let seen = stdout.lines().filter(|l| is_data_row(l)).any(|l| {
            let mut f = l.split_whitespace();
            f.next().and_then(|z| z.parse::<u32>().ok()) == Some(sp.z)
                && f.next() == Some(sp.symbol)
        });
        assert!(seen, "no row for Z = {} ({}) in the availability table", sp.z, sp.symbol);
    }
}
