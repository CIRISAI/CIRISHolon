//! The budgets move with the machine and the horizon — tests that MOVE them.
//!
//! These live in their own test binary because the horizon and the per-branch
//! coefficient are process-wide declarations; a test that sets them would race
//! every test in `refusal.rs` that reads them. One process, one horizon.

use holon_qasm::*;

fn priced(n: usize, t: usize) -> Circuit {
    Circuit {
        n_qubits: n,
        n_clbits: n,
        gates: (0..t).map(|_| Gate::T(0)).collect(),
        measures: (0..n).map(|q| (q, q)).collect(),
    }
}

/// Every test here holds this lock: the declarations are process-wide.
static ONE_HORIZON: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn doubling_the_horizon_buys_exactly_one_more_t_gate() {
    let _g = ONE_HORIZON.lock().unwrap();
    let c = priced(6, 24);
    let base = t_budget_for(&c);
    set_magic_horizon_seconds(Some(magic_horizon_seconds() * 2.0));
    let doubled = t_budget_for(&c);
    set_magic_horizon_seconds(None);
    assert_eq!(doubled, base + 1, "2^t branches: twice the seconds is one more T gate");
    assert_eq!(t_budget_for(&c), base, "restoring the horizon restores the wall");
}

#[test]
fn a_measured_coefficient_moves_the_wall_and_the_refusal_cites_it() {
    let _g = ONE_HORIZON.lock().unwrap();
    let c = priced(6, 24);
    let base = t_budget_for(&c);
    // a machine four times slower than the battlerig
    set_branch_seconds_per_gate_qubit(Some(3.2e-8 * 4.0));
    let slower = t_budget_for(&c);
    let err = check_t_budget(&priced(6, base)).expect_err("the old wall is now past the horizon");
    set_branch_seconds_per_gate_qubit(None);
    assert_eq!(slower, base - 2, "4x the seconds per branch is two fewer T gates");
    assert!(
        err.contains(&format!("{:.2e} s per gate-qubit", 3.2e-8 * 4.0)),
        "the refusal must cite the constant in force; it said: {err}"
    );
    assert_eq!(t_budget_for(&c), base);
}

#[test]
fn the_ram_probe_measures_headroom_and_never_commits_the_pages() {
    let _g = ONE_HORIZON.lock().unwrap();
    // Beyond any address space: refused by the reservation whatever the kernel says.
    assert!(ram_admits(u64::MAX / 2).is_err());
    // Beyond the kernel's own reading of what is available: refused by the
    // headroom half BEFORE any reservation is attempted — the half that keeps
    // a probe from becoming the allocation (the OOM killer answered the first
    // version of this probe, 2026-09-02).
    if let Ok(text) = std::fs::read_to_string("/proc/meminfo") {
        let avail_kb: u64 = text
            .lines()
            .find_map(|l| l.strip_prefix("MemAvailable:"))
            .and_then(|r| r.trim().trim_end_matches("kB").trim().parse().ok())
            .expect("MemAvailable is present on Linux");
        let err = ram_admits(avail_kb * 1024 * 2).expect_err("twice the available RAM is refused");
        assert!(err.contains("available RAM"), "the headroom half must answer first: {err}");
    }
    // A megabyte is admitted everywhere, and the probe touches only a bounded
    // sample, so admitting it a thousand times costs nothing.
    for _ in 0..1000 {
        assert!(ram_admits(1 << 20).is_ok());
    }
}
