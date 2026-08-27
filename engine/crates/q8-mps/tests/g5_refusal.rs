//! G5 — the refusal gate, mutation-tested. `Q8_MPS_PREREG.md` §6.
//!
//! Taxonomy note (the commission's phrasing, carried into the prereg): this refusal is
//! FLOOR-type (`GrainFloor.lean`) — a larger `chi_max` serves the same request this `chi_max`
//! refuses, it is not that nothing finer exists.

use q8_mps::dmrg::{self, Params, RefusalPolicy, REFUSAL_THRESHOLD};

/// The forcing configuration, staked: N=12, U/t=0 (gapless — the smallest gap in the whole
/// grid, per `Δ(N) ∝ 1/(N+1)`, hence the largest correlation length and the hardest case for a
/// small `chi`), `chi` capped at 4 — two orders of magnitude below G2's generous 256.
fn forcing_params() -> Params {
    Params { sites: 12, t: 1.0, u: 0.0, chi_max: 4, max_sweeps: 20, sweep_tol: 1e-10 }
}

#[test]
fn g5_typed_refuses_and_silent_does_not() {
    let p = forcing_params();

    // Typed: must refuse, naming a bond whose discarded weight clears the staked threshold.
    let typed_outcome = dmrg::run(&p, RefusalPolicy::Typed);
    let refusal = match typed_outcome {
        Err(r) => r,
        Ok(_) => panic!(
            "G5 FAILED: RefusalPolicy::Typed did NOT refuse at the forcing configuration \
             (N=12, U=0, chi=4) — the mechanism does not fire where it is staked to"
        ),
    };
    assert!(
        refusal.weight >= REFUSAL_THRESHOLD,
        "G5 FAILED: Typed refused, but weight {:e} is below the staked threshold {REFUSAL_THRESHOLD:e} \
         — a refusal must be justified by the ledger it names",
        refusal.weight
    );
    eprintln!("G5: Typed refused at bond={} weight={:e} (>= {REFUSAL_THRESHOLD:e})", refusal.bond, refusal.weight);

    // Silent (the mutant): must NOT refuse at the identical configuration, proving the test
    // discriminates rather than the forcing configuration being trivially unrunnable.
    let silent_outcome = dmrg::run(&p, RefusalPolicy::Silent);
    assert!(
        silent_outcome.is_ok(),
        "G5 FAILED: RefusalPolicy::Silent ALSO refused at the forcing configuration — the \
         mutation test proves nothing if both policies behave identically"
    );
    eprintln!("G5: Silent proceeded (as required) at the identical configuration — mutation discriminates.");
}
