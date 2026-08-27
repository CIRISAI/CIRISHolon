//! The 3D sizing arithmetic, derived rather than quoted.
//!
//! `MESH_DESIGN.md` §0 rests the entire design on one table, and §3 on its resident-set
//! consequence. Those numbers were computed once in a scratchpad script and typed into
//! prose — the same defect this lane already found and closed for the FCHC sector count
//! (§8's M-G3), and the same defect `tier.rs`'s `REG_PLUS_MAX` had when it declared a
//! momentum maximum of 3 that no subset of the six directions could reach.
//!
//! A headline number with no witness that runs is a number that will drift. This module is
//! the witness.
//!
//! # What is arithmetic here, and what is an assumption
//!
//! Stated plainly, because the two carry very different weight:
//!
//! * **Arithmetic** (checked here): octree node counts, the depth an acuity or grain claim
//!   demands, the surface-versus-volume ratio, and the byte totals against a measured card.
//!   These follow from the declared geometry and cannot be wrong unless the geometry is.
//! * **An ASSUMPTION** (not checked here, and not checkable here): that the observer's claim
//!   reaches only the **visible surface** — that the interior of an opaque pile need not be
//!   resolved to acuity. That is the load-bearing premise of §0's 146×, and this module
//!   computes what it is worth IF it holds. It does not establish that it holds.
//!
//! Its kill, stated so the assumption is falsifiable rather than assumed: **if a certified
//! frontier on a real 3D scene resolves interior cells to acuity — because the certifier's
//! claim reaches them, or because the renderer demands them — then §0's saving is not 146×
//! and the resident-set budget must be re-derived from the measured frontier.** §8's M-G2
//! is that measurement, and it is still owed. Nothing in this module substitutes for it.

/// A declared 3D scene, in the tier's own terms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneSizing {
    /// Side of the cubic domain, metres.
    pub domain_m: f64,
    /// The tier's declared grain.
    pub g0_m: f64,
    /// Smallest feature a viewer distinguishes, in stage pixels (`tier.rs::ACUITY_PIXELS`).
    pub acuity_pixels: f64,
    /// Stage size the acuity is quoted against (`tier.rs::STAGE_PIXELS`).
    pub stage_pixels: f64,
    /// Fraction of the domain's height below the matter line.
    pub fill: f64,
}

impl SceneSizing {
    /// The sandbox tier's declared 3D scene: a 0.6 m cube of 0.5 mm grains, sand to 45% of
    /// the height, viewed on a 900-pixel stage at 3-pixel acuity.
    pub const SANDBOX: Self = Self {
        domain_m: 0.6,
        g0_m: 5.0e-4,
        acuity_pixels: 3.0,
        stage_pixels: 900.0,
        fill: 0.45,
    };

    /// The grain the OBSERVER claims, never finer than the tier's own terminal holon.
    /// Mirrors `tier.rs::acuity_m`.
    pub fn acuity_m(&self) -> f64 {
        (self.domain_m * self.acuity_pixels / self.stage_pixels).max(self.g0_m)
    }

    /// Octree subdivisions needed to reach `target_m`, rounded up so the tree bottoms out at
    /// or below the demand. A power of two, because the 2x2x2 split is exact only then.
    pub fn divisions_to(&self, target_m: f64) -> u32 {
        if !(target_m.is_finite() && target_m > 0.0) || target_m >= self.domain_m {
            return 0;
        }
        (self.domain_m / target_m).log2().ceil() as u32
    }

    /// Cells across one axis after `divisions` subdivisions.
    pub fn across(&self, divisions: u32) -> u64 {
        1u64 << divisions
    }

    /// Leaf size, metres, after `divisions`.
    pub fn leaf_m(&self, divisions: u32) -> f64 {
        self.domain_m / self.across(divisions) as f64
    }
}

/// Leaves of a full octree of the given depth — `8^d`, the whole VOLUME resolved.
pub fn volume_leaves(divisions: u32) -> u64 {
    8u64.pow(divisions)
}

/// Every node of a full octree of the given depth, leaves included: `(8^(d+1) − 1) / 7`.
/// Exact integer arithmetic, not the `× 8/7` approximation.
pub fn volume_nodes(divisions: u32) -> u64 {
    (8u64.pow(divisions + 1) - 1) / 7
}

/// Leaves covering ONE face of the domain at the given depth — `4^d`. A face is a
/// 2-manifold, which is the whole point: it does not grow with the third power.
pub fn surface_leaves(divisions: u32) -> u64 {
    4u64.pow(divisions)
}

/// Every octree node needed to resolve one face to the given depth: `(4^(d+1) − 1) / 3`.
pub fn surface_nodes(divisions: u32) -> u64 {
    (4u64.pow(divisions + 1) - 1) / 3
}

/// Bytes for `nodes` resident holons at the 3D per-holon cost of `MESH_DESIGN.md` §2.2.
pub fn bytes(nodes: u64, bytes_per_holon: u64) -> u64 {
    nodes.saturating_mul(bytes_per_holon)
}

/// Resident cost of one holon in the 3D chart: header + child/sibling index + live ledger
/// overlay + whole-state pool at W = 2, with `momentum: [i64; 4]`. `SANDBOX_4090` §4's stack,
/// re-added for the wider `GrossState`.
pub const BYTES_PER_HOLON_3D: u64 = 144;

/// The 2D figure, for contrast: `momentum: [i64; 2]`.
pub const BYTES_PER_HOLON_2D: u64 = 112;

/// Measured usable device memory, bytes (15,944 MiB — `SANDBOX_4090` §1, measured on the
/// local device, not a published spec).
pub const DEVICE_BYTES: u64 = 15_944 * 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    const S: SceneSizing = SceneSizing::SANDBOX;

    /// The acuity claim, and its relation to the tier's grain. `tier.rs` asserts the same two
    /// facts about its own 2D scene; agreeing with it is the cross-check.
    #[test]
    fn acuity_is_two_millimetres_and_exactly_four_grains() {
        assert!((S.acuity_m() - 2.0e-3).abs() < 1e-12, "{}", S.acuity_m());
        assert!((S.acuity_m() / S.g0_m - 4.0).abs() < 1e-9);
    }

    /// The two depths the design quotes: 9 subdivisions to reach acuity, 11 to reach `g0`.
    #[test]
    fn the_octree_depths_are_nine_to_acuity_and_eleven_to_grain() {
        let acuity = S.divisions_to(S.acuity_m());
        let grain = S.divisions_to(S.g0_m);
        assert_eq!(acuity, 9, "MESH_DESIGN §3: 9 divisions to acuity");
        assert_eq!(grain, 11, "MESH_DESIGN §3: 11 divisions to g0");
        assert_eq!(S.across(acuity), 512);
        assert_eq!(S.across(grain), 2048);
        // The leaf is FINER than acuity, because the power-of-two rounding overshoots. Worth
        // pinning: it means the acuity claim is served with a little margin, never short.
        assert!(S.leaf_m(acuity) < S.acuity_m());
        assert!((S.leaf_m(acuity) - 1.171875e-3).abs() < 1e-12);
    }

    /// **§0's table, derived.** Every row, against the measured device.
    #[test]
    fn the_headline_table_reproduces() {
        let acuity = S.divisions_to(S.acuity_m());
        let grain = S.divisions_to(S.g0_m);

        // Row 1: every cell refined to g0 — refused, ~74x over the card.
        let all_grain = volume_leaves(grain);
        assert_eq!(all_grain, 8_589_934_592, "8^11");
        let over = bytes(all_grain, BYTES_PER_HOLON_3D) as f64 / DEVICE_BYTES as f64;
        assert!((72.0..76.0).contains(&over), "g0 volume is {over:.1}x the card");

        // Row 2: every cell refined to acuity — refused, ~1.3x over the card.
        let all_acuity = volume_nodes(acuity);
        assert_eq!(all_acuity, 153_391_689, "(8^10 - 1)/7");
        let over = bytes(all_acuity, BYTES_PER_HOLON_3D) as f64 / DEVICE_BYTES as f64;
        assert!((1.30..1.35).contains(&over), "acuity volume is {over:.2}x the card");

        // Row 3: matter only at acuity — fits, but spends most of the card rendering 1.2 mm
        // detail through opaque sand.
        let matter = (volume_leaves(acuity) as f64 * S.fill) as u64 * 8 / 7;
        let share = bytes(matter, BYTES_PER_HOLON_3D) as f64 / DEVICE_BYTES as f64;
        assert!((0.55..0.62).contains(&share), "matter-only is {share:.2} of the card");

        // Row 4: acuity on the VISIBLE SURFACE only — 0.9% of the card.
        let visible = 3 * surface_nodes(acuity);
        assert_eq!(surface_nodes(acuity), 349_525, "(4^10 - 1)/3");
        assert_eq!(visible, 1_048_575, "MESH_DESIGN §3: ~1.049e6 nodes at 3 faces");
        let share = bytes(visible, BYTES_PER_HOLON_3D) as f64 / DEVICE_BYTES as f64;
        assert!((0.008..0.010).contains(&share), "surface is {share:.4} of the card");
        assert!((150_000_000..152_000_000).contains(&bytes(visible, BYTES_PER_HOLON_3D)));
    }

    /// **§0's headline: the occlusion saving is 146x.** The single number the whole design
    /// rests on. Derived here so the prose cannot drift from it.
    #[test]
    fn the_occlusion_saving_is_one_hundred_and_forty_six_fold() {
        let acuity = S.divisions_to(S.acuity_m());
        let saving = volume_nodes(acuity) as f64 / (3 * surface_nodes(acuity)) as f64;
        assert!(
            (146.0..147.0).contains(&saving),
            "MESH_DESIGN §0 states 146x; derived {saving:.1}"
        );
    }

    /// **Why it works, stated as a scaling law rather than as one lucky number.**
    ///
    /// The acuity claim is a claim about a 2-MANIFOLD, so it grows as `4^d` while the volume
    /// grows as `8^d`. The saving therefore DOUBLES with every extra subdivision — it is not
    /// a constant that happens to be large at this scene's size, and a finer tier makes
    /// occlusion more valuable, never less.
    #[test]
    fn the_saving_doubles_with_every_subdivision() {
        let mut previous: Option<f64> = None;
        for d in 4..12u32 {
            let saving = volume_nodes(d) as f64 / surface_nodes(d) as f64;
            if let Some(p) = previous {
                let ratio = saving / p;
                assert!(
                    (1.95..2.05).contains(&ratio),
                    "depth {d}: saving grew {ratio:.3}x, expected ~2x"
                );
            }
            previous = Some(saving);
        }
    }

    /// The 3D chart's per-holon cost against the 2D one — **and a correction to §2.2**.
    ///
    /// §2.2 read "the 3D chart costs 29% of the card's holon capacity". That conflated two
    /// different numbers, and this test is what caught it:
    ///
    /// * the 3D holon costs **28.6% MORE bytes** (144 against 112) — this is the ~29%;
    /// * the card's holon **capacity falls by 22.2%** (1.493e8 → 1.161e8).
    ///
    /// Same fact, two framings, and the prose attached the first number to the second
    /// quantity. Both are asserted here so the pair cannot drift apart again.
    #[test]
    fn the_three_d_holon_costs_29_percent_more_and_capacity_falls_22_percent() {
        let cap_2d = DEVICE_BYTES / BYTES_PER_HOLON_2D;
        let cap_3d = DEVICE_BYTES / BYTES_PER_HOLON_3D;
        assert!((1.49e8..1.50e8).contains(&(cap_2d as f64)), "{cap_2d}");
        assert!((1.16e8..1.17e8).contains(&(cap_3d as f64)), "{cap_3d}");

        let extra_bytes = BYTES_PER_HOLON_3D as f64 / BYTES_PER_HOLON_2D as f64 - 1.0;
        assert!(
            (0.28..0.29).contains(&extra_bytes),
            "per-holon cost should rise ~28.6%; derived {extra_bytes:.3}"
        );

        let capacity_lost = 1.0 - cap_3d as f64 / cap_2d as f64;
        assert!(
            (0.22..0.23).contains(&capacity_lost),
            "capacity should fall ~22.2%, NOT 29%; derived {capacity_lost:.3}"
        );
    }

    /// The sharding consequence of §3: the visible-surface budget over 64 shards is ~16,384
    /// nodes each, which §10.4 measured as ABOVE the best-scaling point (4,096 cells/shard).
    #[test]
    fn the_visible_surface_budget_puts_shards_above_the_measured_scaling_point() {
        let acuity = S.divisions_to(S.acuity_m());
        let per_shard = 3 * surface_nodes(acuity) / 64;
        assert!(
            (16_000..16_500).contains(&per_shard),
            "MESH_DESIGN §3: ~16,384 nodes per shard at 64 shards; derived {per_shard}"
        );
        assert!(
            per_shard > 4_096,
            "the 3D target must sit above §10.4's best measured scaling point"
        );
    }

    /// A degenerate scene must not silently produce a number. `divisions_to` returns 0 rather
    /// than a negative or NaN depth when the demand is coarser than the domain.
    #[test]
    fn a_demand_coarser_than_the_domain_asks_for_no_subdivision() {
        assert_eq!(S.divisions_to(S.domain_m), 0);
        assert_eq!(S.divisions_to(S.domain_m * 10.0), 0);
        assert_eq!(S.divisions_to(f64::NAN), 0);
        assert_eq!(S.divisions_to(0.0), 0);
    }

    /// Node counts are exact integer series, not the `x 8/7` and `x 4/3` approximations the
    /// prose uses. Pinned because the approximation is what a reader will reach for.
    #[test]
    fn node_counts_are_the_exact_geometric_series() {
        for d in 0..8u32 {
            let by_series: u64 = (0..=d).map(|k| 8u64.pow(k)).sum();
            assert_eq!(volume_nodes(d), by_series, "volume at depth {d}");
            let by_series: u64 = (0..=d).map(|k| 4u64.pow(k)).sum();
            assert_eq!(surface_nodes(d), by_series, "surface at depth {d}");
        }
    }
}
