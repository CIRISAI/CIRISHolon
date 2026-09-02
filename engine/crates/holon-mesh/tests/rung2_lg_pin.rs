//! The A2 chart's pin: `holon-lens`'s copy of the FHP directions must equal `regplus`'s.
//!
//! `holon-lens` has zero dependencies by design and cannot import `ciris-sim-core`, so
//! `field_lg::DIRECTIONS_AXIAL` is a pinned COPY of `regplus::DIRECTIONS`. A copy without a
//! cross-check is a constant that no test derives — the defect `tier.rs`'s `REG_PLUS_MAX`
//! carried until an enumeration refuted it. This crate can see both sides, so it checks.

use ciris_sim_core::regplus;
use holon_lens::field_lg::{cartesian, mode_of, DIRECTIONS_AXIAL, MODES};

#[test]
fn the_pinned_directions_equal_regplus() {
    assert_eq!(
        DIRECTIONS_AXIAL, regplus::DIRECTIONS,
        "the A2 chart's directions have drifted from the runtime's REG+ table"
    );
}

/// The label the A2 runner passes must be `regplus::sector` and nothing else. This asserts
/// the shape the runner's adapter relies on, over the whole 64-state domain, so a change to
/// `SectorLabel` cannot silently reshape the chart.
#[test]
fn every_local_word_labels_within_the_lattice_object() {
    let mut seen = std::collections::HashSet::new();
    for w in 0..64u8 {
        let s = regplus::sector(w);
        assert_eq!(s.occupancy as u32, (w as u32).count_ones(), "N is the popcount");
        seen.insert((s.occupancy, s.momentum));
    }
    // Core/Lattice.lean's object, reproduced by regplus's own test: 53 sectors.
    assert_eq!(seen.len(), 53, "the (N,P) label must land in the 53 sectors");
}

/// The map's image must be the mode set the label is defined over: every mode reachable,
/// each direction its own fixed point. A map that could never emit some mode would shrink
/// the chart without any gate noticing.
#[test]
fn the_map_covers_the_mode_set_regplus_labels() {
    for d in 0..MODES {
        let e = cartesian(d);
        assert_eq!(mode_of(e[0], e[1]), Some(d));
        // And that mode alone must carry exactly that direction's momentum.
        let s = regplus::sector(1u8 << d);
        assert_eq!(s.occupancy, 1);
        assert_eq!(
            [s.momentum[0] as i64, s.momentum[1] as i64],
            regplus::DIRECTIONS[d]
        );
    }
}
