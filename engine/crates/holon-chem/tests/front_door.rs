//! The refusal, planted through the PRODUCTION ENTRY POINT.
//!
//! # Why this test is in its own binary, and why it plants rather than asserts
//!
//! `PairMeta::converged` spent an hour being computed, serialised, and consulted by nothing
//! on the production path. The contract test asserted the verdict was CORRECT, and it was,
//! the whole time — that assertion could never have found the defect, because the defect was
//! that nobody asked.
//!
//! So this drives `PairCache::get`, which is the sandbox's real entry point to the
//! chemistry, with a solve made to genuinely fail, and requires it to come back REFUSED. A
//! plant into `PairMeta` directly would test the unit beneath the entry point — the exact
//! level at which the defect was invisible.
//!
//! Its own binary because the iteration cap is a global: a separate test process cannot leak
//! an under-converged solver into a test running in parallel.

use holon_chem::elements::HYDROGEN;
use holon_chem::pair::PairCache;
use std::sync::atomic::Ordering;

#[test]
fn the_cache_refuses_a_curve_whose_solve_gave_up() {
    // Sanity first: the front door works when the solve is allowed to converge. Without
    // this, a `should_panic` could be passing on any panic at all — a missing file, a bad
    // index — rather than on the refusal.
    let mut ok = PairCache::new(8);
    let table = ok.get(HYDROGEN, HYDROGEN);
    assert!(table.meta.converged(), "H2 must converge at the default cap");
    assert!(table.meta.worst_residual < 1e-10);
    println!("  front door open: H2 converged, worst residual {:.3e}", table.meta.worst_residual);

    // Now make the solve genuinely fail. One Davidson iteration cannot converge anything,
    // so the eigenvector really is wrong and the residual really is large — the failure the
    // guard exists for, not a forged flag.
    holon_chem::fci::DAVIDSON_MAX_ITER.store(1, Ordering::Relaxed);
    let refused = std::panic::catch_unwind(|| {
        let mut cache = PairCache::new(8);
        cache.get(HYDROGEN, HYDROGEN);
    });
    holon_chem::fci::DAVIDSON_MAX_ITER.store(1200, Ordering::Relaxed);

    assert!(
        refused.is_err(),
        "PairCache::get returned a curve whose solve gave up. The convergence verdict is \
         computed and emitted but nothing on the production path acts on it — which is the \
         defect this test exists for, and which a test asserting the verdict's VALUE cannot \
         detect."
    );
    println!("  front door closed: an unconverged solve was refused by PairCache::get");

    // And the door opens again afterwards, so the plant cannot leave the crate wedged.
    let mut after = PairCache::new(8);
    assert!(after.get(HYDROGEN, HYDROGEN).meta.converged());
}
