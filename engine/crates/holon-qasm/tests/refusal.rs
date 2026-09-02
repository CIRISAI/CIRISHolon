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
    let over = T_MAX_MAGIC + 1;
    let c = priced(6, over);
    let y = vec![false; c.n_qubits];
    let err = magic::try_magic_amplitude(&c, &y, false, false)
        .expect_err("the amp path must refuse past the T cap, not enumerate");
    names_the_numbers(
        &err,
        &[
            "REFUSED",
            &format!("T-count {over}"),
            &T_MAX_MAGIC.to_string(),
            &format!("2^{over} = {}", 1u64 << over),
            "tableau_not_closed_under_rotation",
        ],
    );
}

#[test]
#[should_panic(expected = "REFUSED by the magic wall")]
fn amp_library_call_refuses_past_the_t_cap() {
    // The unchecked entry point is public, so it is a door too. It refuses by
    // name rather than starting a 2^t enumeration it cannot finish.
    let c = priced(6, T_MAX_MAGIC + 1);
    let y = vec![false; c.n_qubits];
    let _ = magic::magic_amplitude(&c, &y, false, false);
}

#[test]
fn amp_path_is_open_at_the_cap() {
    // The wall is AT the cap, not below it: a mutation that tightens the cap
    // is as wrong as one that deletes it. The boundary is written as a LITERAL
    // and the constant is pinned against it — phrasing this as
    // `check_t_budget(priced(6, T_MAX_MAGIC)).is_ok()` reads the cap out of the
    // same constant it is checking, which is a tautology that survives any
    // change to the number (it did, on the first run of the mutation battery).
    assert_eq!(
        T_MAX_MAGIC, 24,
        "the magic tier's branch budget is a published number \
         (conformance/qasm/upstream/BATTLERIG.md); moving it moves the record"
    );
    assert!(check_t_budget(&priced(6, 24)).is_ok(), "t = 24 is inside the budget");
    assert!(check_t_budget(&priced(6, 25)).is_err(), "t = 25 is outside it");
    // Predicate only: actually summing 2^24 branches is not a unit test.
}

// ------------------------------ door: distribution mode (`run --tier magic`)

#[test]
fn distribution_path_refuses_past_the_t_cap() {
    let over = T_MAX_MAGIC + 1;
    let err = magic::try_run_magic(&priced(4, over), false, false)
        .expect_err("the distribution path must refuse past the T cap");
    names_the_numbers(&err, &[&format!("T-count {over}"), &T_MAX_MAGIC.to_string()]);
}

#[test]
fn distribution_path_refuses_past_the_n_wall() {
    // The sibling the enumeration turned up. The magic tier's DISTRIBUTION
    // mode accumulates 2^n exact amplitudes, so it pays the carrier's wall as
    // well as its own; before this guard, n=40 t=6 asked for 88 TB and dumped
    // core instead of refusing.
    let n = N_MAX_STATEVECTOR + 16;
    let err = magic::try_run_magic(&priced(n, 6), false, false)
        .expect_err("the distribution path must refuse past the n wall");
    names_the_numbers(
        &err,
        &[
            "REFUSED",
            &format!("n = {n}"),
            &N_MAX_STATEVECTOR.to_string(),
            &format!("2^{n} = {}", 1u64 << n),
        ],
    );
}

#[test]
#[should_panic(expected = "REFUSED by the carrier wall")]
fn distribution_library_call_refuses_past_the_n_wall() {
    let _ = magic::run_magic(&priced(N_MAX_STATEVECTOR + 16, 6), false, false);
}

// --------------------------------------------- door: the router's own choice

#[test]
fn router_does_not_send_a_wide_circuit_to_magic() {
    // The router picks a tier for the DISTRIBUTION path. Routing on T-count
    // alone walked a 40-qubit circuit past the very n wall the carrier arm
    // enforces, and it was the DEFAULT path that did it — no override needed.
    let c = priced(N_MAX_STATEVECTOR + 16, 6);
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
    let mut c = priced(N_MAX_STATEVECTOR + 1, 0);
    // neither classical (the h) nor Clifford (the ccx), and t = 0
    c.gates.push(Gate::H(0));
    c.gates.push(Gate::Ccx(0, 1, 2));
    let err = route(&c).expect_err("ccx above the n wall has no tier");
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
    let n = N_MAX_STATEVECTOR + 16;
    let err = try_run(&priced(n, 0), Tier::Statevector, Mutation::None)
        .expect_err("the carrier must refuse past its own wall");
    names_the_numbers(&err, &["REFUSED", &format!("n = {n}"), "2^n amplitudes"]);
}

#[test]
#[should_panic(expected = "REFUSED by the carrier wall")]
fn carrier_library_call_refuses_past_the_n_wall() {
    let _ = run_statevector(&priced(N_MAX_STATEVECTOR + 16, 0));
}

// ------------------------------------------------------- the poly tiers pass

#[test]
fn poly_tiers_carry_no_wall_because_they_have_no_exponential() {
    // Stated so the door table is complete rather than silently short: the
    // classical and tableau tiers are poly in n and hold no 2^k array, so
    // there is nothing here for a budget to refuse. A wall on them would be
    // theatre.
    let wide = priced(N_MAX_STATEVECTOR + 200, 0);
    assert!(try_run(&wide, Tier::Classical, Mutation::None).is_ok());
    assert!(try_run(&wide, Tier::Tableau, Mutation::None).is_ok());
}
