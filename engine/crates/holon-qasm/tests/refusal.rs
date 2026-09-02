//! Refusal honesty, per door.
//!
//! The suite's standing claim is "past every tier's budget the router refuses
//! naming `tableau_not_closed_under_rotation` and the T-price"
//! (conformance/qasm/QASM_SUITE.md). The battlerig found it false on the `amp`
//! path — `run_magic` asserted its cap, `magic_amplitude` had no guard at all,
//! and the t=28 hidden-shift points began enumerating 2^28 branches and died
//! as a TIMEOUT (upstream/BATTLERIG.md caveat 1). Enumerating the doors turned
//! up two siblings of the same shape, so what is gated here is not one guard
//! but the class: EVERY entry point that can start an exponential enumeration
//! refuses by name, and the refusal carries the numbers.
//!
//! The budgets are PRICES, not caps (2026-09-02): the magic tier is priced in
//! seconds against a horizon, the 2^n carriers in bytes against this machine's
//! headroom. So every wall here is DERIVED — `over_budget(n)` walks the T-count
//! up until the price crosses the horizon, and for the carriers an n whose
//! price exceeds any machine (2^48 amplitudes is petabytes) — never read out of
//! a constant. The wall moves with
//! the horizon; `tests/budget.rs` holds the tests that move it, in their own
//! process, because the horizon is process-wide.
//!
//! Each test names the door it holds. Deleting any one guard fires the test
//! that names it — `conformance/qasm/mutate_tcap.py` plants exactly that and
//! checks it fires.

use holon_qasm::*;

/// t T-gates on qubit 0 of an n-qubit register, measured. Nothing about the
/// circuit matters here except its two prices, t and n.
fn priced(n: usize, t: usize) -> Circuit {
    Circuit {
        n_qubits: n,
        n_clbits: n,
        gates: (0..t).map(|_| Gate::T(0)).collect(),
        measures: (0..n).map(|q| (q, q)).collect(),
    }
}

/// The first T-count past the horizon for the `priced` family at width n,
/// found by walking the door — not read out of a constant.
fn over_budget(n: usize) -> usize {
    let mut t = 1;
    while check_t_budget(&priced(n, t)).is_ok() {
        t += 1;
        assert!(t < 200, "the horizon admits everything: the price is gone");
    }
    t
}

/// An n no machine admits: 2^48 amplitudes is 4.5 PB in the carrier and
/// 22.5 PB in the magic tier's distribution accumulator.
const N_BEYOND_ANY_MACHINE: usize = 48;

/// A refusal that does not carry its numbers is not a refusal by name.
fn names_the_numbers(msg: &str, must: &[&str]) {
    for needle in must {
        assert!(
            msg.contains(needle),
            "refusal does not name {needle:?}; it said: {msg}"
        );
    }
}

// ------------------------------------------------- door: `amp` (the defect)

#[test]
fn amp_path_refuses_past_the_t_cap() {
    let over = over_budget(6);
    let c = priced(6, over);
    let y = vec![false; c.n_qubits];
    let err = magic::try_magic_amplitude(&c, &y, false, false)
        .expect_err("the amp path must refuse past the horizon, not enumerate");
    names_the_numbers(
        &err,
        &[
            "REFUSED",
            &format!("T-count {over}"),
            &format!("horizon of {:.3e} s", magic_horizon_seconds()),
            &format!("2^{over} = {}", 1u64 << over),
            "tableau_not_closed_under_rotation",
            "PROVISIONAL",
        ],
    );
}

#[test]
#[should_panic(expected = "REFUSED by the magic wall")]
fn amp_library_call_refuses_past_the_t_cap() {
    // The unchecked entry point is public, so it is a door too. It refuses by
    // name rather than starting a 2^t enumeration it cannot finish.
    let c = priced(6, over_budget(6));
    let y = vec![false; c.n_qubits];
    let _ = magic::magic_amplitude(&c, &y, false, false);
}

#[test]
fn amp_path_is_open_at_the_cap() {
    // The wall is AT the derived budget, not below it: a mutation that
    // tightens the horizon or the price is as wrong as one that deletes the
    // guard. The boundary is written as a LITERAL and the derivation is pinned
    // against it — phrasing this as `check_t_budget(priced(6, t_budget_for(..)))`
    // alone reads the budget out of the same function it is checking, which is
    // a tautology that survives any change to the horizon (the constant-era
    // version of this test did exactly that on the battery's first run).
    //
    // Default horizon 120 s, provisional 3.2e-8 s per gate-qubit per branch;
    // `priced(n, t)` has t gates, so its price is 2^t · t · n · 3.2e-8 s:
    //   n = 6:  t = 24 → 77.3 s,  t = 25 → 161 s
    //   n = 20: t = 22 → 59.1 s,  t = 23 → 123.5 s
    assert_eq!(magic_horizon_seconds(), 120.0, "the default horizon is the battlerig's per-point cap");
    assert_eq!(branch_seconds_per_gate_qubit(), 3.2e-8, "the provisional fit is the published one");
    assert_eq!(t_budget_for(&priced(6, 24)), 24, "the horizon buys T-count 24 for a 24-gate n = 6 circuit");
    assert!(check_t_budget(&priced(6, 24)).is_ok(), "t = 24 is inside the horizon at n = 6");
    assert!(check_t_budget(&priced(6, 25)).is_err(), "t = 25 is outside it");
    assert!(check_t_budget(&priced(20, 22)).is_ok(), "t = 22 is inside the horizon at n = 20");
    assert!(check_t_budget(&priced(20, 23)).is_err(), "t = 23 is outside it");
    // Predicate only: actually summing 2^24 branches is not a unit test.
}

#[test]
fn the_magic_price_is_the_battlerig_within_its_stated_tolerance() {
    // The fit's provenance: the battlerig's t=14 hidden-shift lane measured
    // 170 gates at n=20 in 2.0176 s, 323 at n=40 in 5.9966 s, 470 at n=60 in
    // 14.9095 s (upstream/BATTLERIG.md). The refusal cites this fit; if the
    // fit drifts from the record the refusal cites a number that is no longer
    // true. Fourteen T gates and the rest Clifford, as the lane's circuits were.
    for (n, gates, measured) in [(20usize, 170usize, 2.0176f64), (40, 323, 5.9966), (60, 470, 14.9095)] {
        let mut c = priced(n, 14);
        c.gates.extend((14..gates).map(|_| Gate::H(0)));
        let fit = predicted_magic_seconds(&c);
        let ratio = fit / measured;
        assert!(
            (0.87..=1.13).contains(&ratio),
            "fit {fit:.2} s vs measured {measured} s at n = {n}: ratio {ratio:.2} outside the stated 13%"
        );
    }
}

// ------------------------------ door: distribution mode (`run --tier magic`)

#[test]
fn distribution_path_refuses_past_the_t_cap() {
    let over = over_budget(4);
    let err = magic::try_run_magic(&priced(4, over), false, false)
        .expect_err("the distribution path must refuse past the horizon");
    names_the_numbers(
        &err,
        &[
            &format!("T-count {over}"),
            &format!("horizon of {:.3e} s", magic_horizon_seconds()),
        ],
    );
}

#[test]
fn distribution_path_refuses_past_the_n_wall() {
    // The sibling the enumeration turned up. The magic tier's DISTRIBUTION
    // mode accumulates 2^n exact amplitudes, so it pays the carrier's price as
    // well as its own; before this guard, n=40 t=6 asked for 88 TB and dumped
    // core instead of refusing. The wall is this machine's headroom, so the
    // refusal names the BYTES and the probe's reason, not a qubit count.
    let n = N_BEYOND_ANY_MACHINE;
    let err = magic::try_run_magic(&priced(n, 6), false, false)
        .expect_err("the distribution path must refuse past the machine's RAM");
    let bytes = (1u64 << n) * MAGIC_DISTRIBUTION_BYTES_PER_AMPLITUDE;
    names_the_numbers(
        &err,
        &[
            "REFUSED",
            &format!("n = {n}"),
            &format!("2^{n} = {}", 1u64 << n),
            &format!("{bytes} bytes"),
            "probe",
        ],
    );
}

#[test]
#[should_panic(expected = "REFUSED by the carrier wall")]
fn distribution_library_call_refuses_past_the_n_wall() {
    let _ = magic::run_magic(&priced(N_BEYOND_ANY_MACHINE, 6), false, false);
}

// --------------------------------------------- door: the router's own choice

#[test]
fn router_does_not_send_a_wide_circuit_to_magic() {
    // The router picks a tier for the DISTRIBUTION path. Routing on T-count
    // alone walked a 40-qubit circuit past the very carrier price the carrier
    // arm enforces, and it was the DEFAULT path that did it — no override
    // needed.
    let c = priced(N_BEYOND_ANY_MACHINE, 6);
    let err = route(&c).expect_err("route must refuse a circuit no tier can afford");
    names_the_numbers(
        &err,
        &["REFUSED", "T-count 6", &format!("n = {}", c.n_qubits)],
    );
    // and the arm it used to take is still open where it is affordable
    assert_eq!(route(&priced(6, 6)), Ok(Tier::Magic));
}

#[test]
fn router_refusal_does_not_misreport_the_t_count() {
    // The old wall message read "T-count {} > 12" unconditionally, so a
    // refusal caused by n (or by a ccx) reported a T-count comparison that
    // was false. A refusal that misstates its own reason is not a refusal by
    // name either.
    let mut c = priced(N_BEYOND_ANY_MACHINE, 0);
    // neither classical (the h) nor Clifford (the ccx), and t = 0
    c.gates.push(Gate::H(0));
    c.gates.push(Gate::Ccx(0, 1, 2));
    let err = route(&c).expect_err("ccx past the machine's RAM has no tier");
    assert!(
        !err.contains("T-count 0 > "),
        "refusal claims a T-count comparison that did not fire: {err}"
    );
}

// ------------------------------------------ door: the carrier (`--tier statevector`)

#[test]
fn carrier_refuses_past_the_n_wall_by_name() {
    // `--tier statevector` and every library caller reach `run_statevector`
    // without the router. It used to assert "router guarantees the cap" — a
    // true statement about the router and a false one about its callers.
    let n = N_BEYOND_ANY_MACHINE;
    let err = try_run(&priced(n, 0), Tier::Statevector, Mutation::None)
        .expect_err("the carrier must refuse past the machine's RAM");
    names_the_numbers(&err, &["REFUSED", &format!("n = {n}"), "2^n amplitudes"]);
}

#[test]
#[should_panic(expected = "REFUSED by the carrier wall")]
fn carrier_library_call_refuses_past_the_n_wall() {
    let _ = run_statevector(&priced(N_BEYOND_ANY_MACHINE, 0));
}

#[test]
fn the_carrier_wall_is_the_machine_not_a_number() {
    // 2^20 amplitudes is 16 MB: every machine that can build this crate admits
    // it, so the door is OPEN there whatever the old constant would have said,
    // and it is open by the probe's verdict, not by a comparison with 24.
    assert!(check_n_budget(&priced(20, 0), "the statevector carrier stores 2^n amplitudes").is_ok());
    // and n = 63 is more bytes than a u64 of address space: refused everywhere.
    assert!(check_n_budget(&priced(63, 0), "the statevector carrier stores 2^n amplitudes").is_err());
}

// ------------------------------------------------------- the poly tiers pass

#[test]
fn poly_tiers_carry_no_wall_because_they_have_no_exponential() {
    // Stated so the door table is complete rather than silently short: the
    // classical and tableau tiers are poly in n and hold no 2^k array, so
    // there is nothing here for a budget to refuse. A wall on them would be
    // theatre.
    let wide = priced(N_BEYOND_ANY_MACHINE + 200, 0);
    assert!(try_run(&wide, Tier::Classical, Mutation::None).is_ok());
    assert!(try_run(&wide, Tier::Tableau, Mutation::None).is_ok());
}
