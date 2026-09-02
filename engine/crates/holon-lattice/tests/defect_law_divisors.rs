//! The defect law off the staked grid: EVERY divisor of several lattice sizes, not only the
//! powers of two `LG_PREREG.md` §5.3 tabulated at `L = 64`.
//!
//! The freeze's table listed `b ∈ {1,2,4,8,16,32,64}` because that was the campaign's grid,
//! and a law checked only where it was staked is a law checked only where it was convenient.
//! Odd blocks, non-power-of-two blocks, and lattices whose side is not a power of two are all
//! exercised here. This is an EXTENSION of the staked claim obtained after the freeze, and
//! `LG_RESULTS.md` §3 reports it as one rather than folding it into the pre-registered part.
//!
//! Cheap because the derivation says it should be: only the perturbed cell's position within
//! its block can matter at one step, so `L` is kept small and the probe is exhaustive.

use holon_lattice::chart::BlockChart;
use holon_lattice::lattice::Lattice;
use holon_lattice::probe::{probe, Move, Population};
use holon_lattice::state::Model;

fn divisors(l: usize) -> Vec<usize> {
    (1..=l).filter(|d| l % d == 0).collect()
}

#[test]
fn the_defect_law_holds_at_every_divisor_of_several_lattice_sizes() {
    let m = Model::fhp6();
    let mut checked = 0;
    for &l in &[12usize, 18, 24, 30] {
        let g = Lattice::seeded(m.clone(), l, 0xD1F5, 0.35, m.fhp_i(true));
        for b in divisors(l) {
            let chart = BlockChart::new(b, l).expect("a divisor is a valid block size");
            let r = probe(&g, chart, 1, Population::Exhaustive, Move::Fiber);
            let predicted = BlockChart::predicted_witness_rate(b, l);
            assert!(r.probes > 0, "L={l} b={b}: the probe did no work");
            assert!(
                (r.rate() - predicted).abs() < 1e-12,
                "L={l} b={b}: measured {} vs derived {predicted}",
                r.rate()
            );
            checked += 1;
        }
    }
    // A count, not a banner: the assertions above pass vacuously if the loop never runs.
    assert_eq!(checked, 6 + 6 + 8 + 8, "the divisor sweep did not cover what it claims");
}

/// The odd and non-power-of-two values specifically, with the numbers written out, so a
/// change to `predicted_witness_rate` cannot pass by moving both sides of the comparison.
#[test]
fn the_odd_and_non_power_of_two_blocks_carry_their_stated_values() {
    for (b, l, w) in [
        (3usize, 12usize, 8.0 / 9.0),
        (5, 30, 0.64),
        (6, 12, 5.0 / 9.0),
        (9, 18, 0.395_061_728_395_061_7),
        (15, 30, 0.248_888_888_888_888_9),
    ] {
        assert!(
            (BlockChart::predicted_witness_rate(b, l) - w).abs() < 1e-12,
            "W({b}) at L={l} is {} not {w}",
            BlockChart::predicted_witness_rate(b, l)
        );
    }
}
