//! THE RECEIPT: the species-generic cluster path is BIT-IDENTICAL to the hand-written
//! (O, H, H, H) path it replaced.
//!
//! # Why a frozen copy and not a tolerance
//!
//! `src/quaternary.rs` used to contain the arithmetic; it now contains an
//! instantiation of `src/cluster.rs`. A generalisation that moves the last bits of a
//! four-body term is not a refactor — it is a new model wearing the old name, and every
//! certified `dE4` table, every trajectory, and every gate pinned against them would be
//! grading a different surface while reporting the same headline. `close enough` is not
//! available here, so the gate is `assert_eq!` on `f64::to_bits`.
//!
//! `mod frozen` below is a VERBATIM copy of the pre-generic implementation, taken from
//! the git blob at the commit before the generalisation. It is deliberately not tidied,
//! not deduplicated, and not made to share a line with the new path: its whole job is to
//! be the arithmetic that used to run. Reproduce it with
//!
//! ```text
//! git show 892c982:engine/crates/holon-chem/src/quaternary.rs
//! ```
//!
//! and the seven items below are byte-identical to their bodies there, modulo the
//! wrapper's four-space indent and one forced path change: `crate::fci::` becomes
//! `holon_chem::fci::` because the copy now lives outside the crate. Compared per
//! geometry: `E_FCI`, `E_MBE3`, `dE4`, and then the whole gradient object — its energy,
//! all twelve Cartesian components, its full 1,568-double CI vector, its Davidson
//! iteration count and its worst residual — cold and warm-started.
//!
//! # The staked geometries
//!
//! Five ASYMMETRIC OHHH configurations, staked before the comparison was run. Asymmetric
//! is the requirement that carries the test: on a C2v geometry two O-H distances are
//! equal, the two water triples that differ only by which hydrogen is listed first
//! collapse onto one another, and a permutation defect in the generic path's
//! slot-to-canonical-position assignment would be invisible. All six internal distances
//! are distinct in every one of the five, and both hub-and-cycle orderings
//! (`QUAD_PAIRS`, `QUAD_TRIPLES`) are therefore exercised on six and four distinct
//! numbers respectively.
//!
//! # The plant, and what it printed
//!
//! A bit gate nobody has seen fire is a bit gate nobody has tested, so before this
//! landed, two one-ULP perturbations were planted on the GENERIC side, run, and removed.
//! Note that a `#[cfg(test)]` plant would NOT have worked: the library a `tests/*.rs`
//! binary links is compiled WITHOUT `cfg(test)`, so a plant gated that way is invisible
//! to exactly the gate it is meant to prove. Both were therefore edited in, run, and
//! reverted. What they printed:
//!
//! ```text
//! P1 — `cluster_atom_energy`'s multiplicity unit, 1.0 -> 1.0 + f64::EPSILON:
//!   S1 ... / E_MBE3: the generic path is NOT bit-identical.
//!     old = -7.55094205745019167e1  bits 0xc052e09a58c0d94c
//!     new = -7.55094205745019309e1  bits 0xc052e09a58c0d94d
//!
//! P2 — `cluster_fci_grad`'s gradient unit, 1.0 -> 1.0 + f64::EPSILON:
//!   S1 ... / grad cold / grad[0][0]: the generic path is NOT bit-identical.
//!     old =  6.21232806391987941e-2  bits 0x3fafce9f6554c80e
//!     new =  6.21232806391988079e-2  bits 0x3fafce9f6554c810
//! ```
//!
//! P2 left the ENERGY untouched and fired on a gradient component, which is the
//! separation the two plants exist to demonstrate: the twelve components are graded
//! independently of the scalar, and neither leg is riding on the other.

use holon_chem::cluster::{ClusterClass, SurfaceFamily};
use holon_chem::ooh::{OohMeta, OohTable};
use holon_chem::quaternary;
use holon_chem::trimer::{self, TrimerTable};
use holon_chem::water::{self, WaterTable};
use std::sync::OnceLock;

// ============================================================================
// The frozen pre-generic path. VERBATIM from `src/quaternary.rs` at the commit
// before `src/cluster.rs` existed. Do not tidy, do not deduplicate, do not share
// a line of it with the library: it is the old arithmetic, kept to be graded against.
// ============================================================================
#[allow(clippy::excessive_precision)]
mod frozen {
    use holon_chem::dual::D2;
    use holon_chem::elements::{HYDROGEN, OXYGEN};
    use holon_chem::pair::{atom_energy, geometry_problem, pair_point, solve_geometry};
    use holon_chem::trimer::TrimerTable;
    use holon_chem::water::WaterTable;
    use std::sync::OnceLock;

    #[inline]
    fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
        let dx = a[0] - b[0];
        let dy = a[1] - b[1];
        let dz = a[2] - b[2];
        (dx * dx + dy * dy + dz * dz).sqrt().max(1e-12)
    }

    /// Evaluates E_FCI for (O, H, H, H) in STO-3G minimal basis (1,568 determinants).
    pub fn ohhh_fci_energy(centers: &[[f64; 3]; 4]) -> f64 {
        let species = [OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN];
        let dual_centers = vec![
            [D2::c(centers[0][0]), D2::c(centers[0][1]), D2::c(centers[0][2])],
            [D2::c(centers[1][0]), D2::c(centers[1][1]), D2::c(centers[1][2])],
            [D2::c(centers[2][0]), D2::c(centers[2][1]), D2::c(centers[2][2])],
            [D2::c(centers[3][0]), D2::c(centers[3][1]), D2::c(centers[3][2])],
        ];
        solve_geometry(&species, dual_centers).e.v
    }

    /// Evaluates E_MBE3 for the 4-atom system (O, H, H, H): 6 pairs + 4 triples + isolated atoms.
    pub fn ohhh_mbe3_energy(
        centers: &[[f64; 3]; 4],
        water_table: &WaterTable,
        trimer_table: &TrimerTable,
    ) -> f64 {
        let e_o = atom_energy_o();
        let e_h = atom_energy_h();

        let o = &centers[0];
        let h1 = &centers[1];
        let h2 = &centers[2];
        let h3 = &centers[3];

        let r1 = dist(o, h1);
        let r2 = dist(o, h2);
        let r3 = dist(o, h3);
        let r12 = dist(h1, h2);
        let r23 = dist(h2, h3);
        let r31 = dist(h3, h1);

        // 6 Pair terms
        let v2_oh1 = pair_point(OXYGEN, HYDROGEN, r1).e - e_o - e_h;
        let v2_oh2 = pair_point(OXYGEN, HYDROGEN, r2).e - e_o - e_h;
        let v2_oh3 = pair_point(OXYGEN, HYDROGEN, r3).e - e_o - e_h;
        let v2_h12 = pair_point(HYDROGEN, HYDROGEN, r12).e - 2.0 * e_h;
        let v2_h23 = pair_point(HYDROGEN, HYDROGEN, r23).e - 2.0 * e_h;
        let v2_h31 = pair_point(HYDROGEN, HYDROGEN, r31).e - 2.0 * e_h;
        let pairs = v2_oh1 + v2_oh2 + v2_oh3 + v2_h12 + v2_h23 + v2_h31;

        // 4 Triple terms: 3 (O,H,H) + 1 (H,H,H)
        let triples = water_table.eval(r1, r2, r12).0
            + water_table.eval(r2, r3, r23).0
            + water_table.eval(r3, r1, r31).0
            + trimer_table.eval([r12, r23, r31]).0;

        e_o + 3.0 * e_h + pairs + triples
    }

    pub fn atom_energy_o() -> f64 {
        static E: OnceLock<f64> = OnceLock::new();
        *E.get_or_init(|| atom_energy(OXYGEN))
    }
    pub fn atom_energy_h() -> f64 {
        static E: OnceLock<f64> = OnceLock::new();
        *E.get_or_init(|| atom_energy(HYDROGEN))
    }

    /// The FCI half of the four-body term with its EXACT Cartesian gradient.
    pub struct OhhhFciGrad {
        pub e: f64,
        pub grad: [[f64; 3]; 4],
        pub ci: Vec<f64>,
        pub davidson_iters_total: usize,
        pub worst_residual: f64,
    }

    /// E_FCI(OH3) and its exact Cartesian gradient in nine seeded dual solves.
    pub fn ohhh_fci_grad(centers: &[[f64; 3]; 4], warm: Option<&[f64]>) -> OhhhFciGrad {
        let species = [OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN];
        let mut grad = [[0.0f64; 3]; 4];
        let mut e = 0.0f64;
        let mut ci: Vec<f64> = Vec::new();
        let mut iters = 0usize;
        let mut worst = 0.0f64;
        let mut start: Option<Vec<f64>> = warm.map(|w| w.to_vec());
        for atom in 1..4usize {
            for axis in 0..3usize {
                let dual: Vec<[D2; 3]> = (0..4)
                    .map(|a| {
                        core::array::from_fn(|x| {
                            if a == atom && x == axis {
                                D2::var(centers[a][x])
                            } else {
                                D2::c(centers[a][x])
                            }
                        })
                    })
                    .collect();
                let (space, mo, nuc) = geometry_problem(&species, dual);
                let sol = holon_chem::fci::solve_determinant_from(&space, &mo, start.as_deref());
                let tot = sol.e + nuc;
                grad[atom][axis] = tot.d;
                if atom == 1 && axis == 0 {
                    e = tot.v;
                    ci = sol.vector.clone();
                }
                iters += sol.davidson_iters;
                worst = worst.max(sol.residual);
                start = Some(sol.vector);
            }
        }
        for x in 0..3 {
            grad[0][x] = -(grad[1][x] + grad[2][x] + grad[3][x]);
        }
        OhhhFciGrad {
            e,
            grad,
            ci,
            davidson_iters_total: iters,
            worst_residual: worst,
        }
    }

    /// Exact ab-initio 4-body term dE4 = E_FCI - E_MBE3 from Cartesian coordinates.
    pub fn de4_ohhh_fci(
        centers: &[[f64; 3]; 4],
        water_table: &WaterTable,
        trimer_table: &TrimerTable,
    ) -> f64 {
        let ef = ohhh_fci_energy(centers);
        let em = ohhh_mbe3_energy(centers, water_table, trimer_table);
        ef - em
    }
}

// ============================================================================
// The staked set
// ============================================================================

/// Five asymmetric OHHH geometries, bohr, staked before the comparison was run.
/// Slot order is `[O, H1, H2, H3]`. All six internal distances are distinct in each.
const STAKED: [(&str, [[f64; 3]; 4]); 5] = [
    (
        "S1 near-equilibrium water plus a third H out of plane",
        [
            [0.0, 0.0, 0.0],
            [1.8100, 0.0, 0.0],
            [-0.4700, 1.7300, 0.0],
            [0.3100, -0.6200, 2.9000],
        ],
    ),
    (
        "S2 whole cluster translated off the origin, no coordinate zero shared",
        [
            [0.1000, -0.2000, 0.0500],
            [1.9500, 0.3100, -0.1200],
            [-0.6100, 1.6400, 0.2200],
            [0.9000, -2.4000, 1.3000],
        ],
    ),
    (
        "S3 compressed O-H pair with the third H below the plane",
        [
            [0.0, 0.0, 0.0],
            [1.7000, 0.2000, 0.1000],
            [-0.9000, 1.5000, -0.3000],
            [-1.2000, -1.1000, -2.2000],
        ],
    ),
    (
        "S4 strongly unequal O-H sides, third H nearest H2 not H1",
        [
            [-0.3000, 0.4000, -0.1000],
            [1.6000, -0.5000, 0.7000],
            [0.2000, 1.9000, -0.6000],
            [2.6000, 1.4000, 1.9000],
        ],
    ),
    (
        "S5 third H far out, approaching the R_CUT shell",
        [
            [0.0, 0.0, 0.0],
            [2.0500, 0.1000, -0.3000],
            [-0.3000, -1.9800, 0.4000],
            [0.7000, 0.9000, 3.7000],
        ],
    ),
];

fn tables() -> &'static (WaterTable, TrimerTable) {
    static T: OnceLock<(WaterTable, TrimerTable)> = OnceLock::new();
    T.get_or_init(|| {
        let src = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/data/s2/s2_water_table.txt"),
        )
        .expect("the committed (O, H, H) table");
        let w = water::from_text(&src).expect("water table parses");
        let t = trimer::generate().expect("the (H, H, H) table");
        (w, t)
    })
}

#[track_caller]
fn same_bits(what: &str, old: f64, new: f64) {
    assert_eq!(
        old.to_bits(),
        new.to_bits(),
        "{what}: the generic path is NOT bit-identical.\n  old = {old:.17e}  bits {ob:#018x}\
         \n  new = {new:.17e}  bits {nb:#018x}\n  difference = {d:.6e}",
        ob = old.to_bits(),
        nb = new.to_bits(),
        d = new - old,
    );
}

/// The whole comparison for one geometry.
fn identical_through(label: &str, g: &[[f64; 3]; 4]) {
    let (w, t) = tables();

    // 1. E_FCI, value only.
    same_bits(
        &format!("{label} / E_FCI"),
        frozen::ohhh_fci_energy(g),
        quaternary::ohhh_fci_energy(g),
    );

    // 2. E_MBE3: six pair excesses in hub-and-cycle order, four triples through the
    //    Z-keyed surface registry, reference atoms grouped by species.
    same_bits(
        &format!("{label} / E_MBE3"),
        frozen::ohhh_mbe3_energy(g, w, t),
        quaternary::ohhh_mbe3_energy(g, w, t),
    );

    // 3. dE4, the difference the certified tables are built from.
    same_bits(
        &format!("{label} / dE4"),
        frozen::de4_ohhh_fci(g, w, t),
        quaternary::de4_ohhh_fci(g, w, t),
    );

    // 4. The gradient object, cold.
    let old = frozen::ohhh_fci_grad(g, None);
    let new = quaternary::ohhh_fci_grad(g, None);
    compare_grad(&format!("{label} / grad cold"), &old, &new);

    // 5. The gradient object, warm-started from each path's OWN cold CI vector — which,
    //    step 4 having passed, is the same vector. This is the shape `sim.rs` runs.
    let old_w = frozen::ohhh_fci_grad(g, Some(&old.ci));
    let new_w = quaternary::ohhh_fci_grad(g, Some(&new.ci));
    compare_grad(&format!("{label} / grad warm"), &old_w, &new_w);
}

#[track_caller]
fn compare_grad(label: &str, old: &frozen::OhhhFciGrad, new: &quaternary::OhhhFciGrad) {
    same_bits(&format!("{label} / e"), old.e, new.e);
    for atom in 0..4 {
        for axis in 0..3 {
            same_bits(
                &format!("{label} / grad[{atom}][{axis}]"),
                old.grad[atom][axis],
                new.grad[atom][axis],
            );
        }
    }
    assert_eq!(
        old.ci.len(),
        new.ci.len(),
        "{label}: CI vector length changed ({} vs {})",
        old.ci.len(),
        new.ci.len()
    );
    assert!(old.ci.len() > 1000, "{label}: CI vector is suspiciously short");
    for (i, (a, b)) in old.ci.iter().zip(new.ci.iter()).enumerate() {
        same_bits(&format!("{label} / ci[{i}]"), *a, *b);
    }
    assert_eq!(
        old.davidson_iters_total, new.davidson_iters_total,
        "{label}: Davidson iteration count changed"
    );
    same_bits(&format!("{label} / worst_residual"), old.worst_residual, new.worst_residual);

    // The construction the whole translation-invariance argument rests on, stated as the
    // ASSOCIATION it actually is: slot 0's row is minus the LEFT-TO-RIGHT sum of the
    // others. A generic fold that reassociated to `-(g1 + (g2 + g3))`, or that seeded on
    // a `0.0`, would move the last bits here and nowhere else obvious.
    for axis in 0..3 {
        let s = new.grad[1][axis] + new.grad[2][axis] + new.grad[3][axis];
        same_bits(&format!("{label} / translation invariance, axis {axis}"), -s, new.grad[0][axis]);
    }
}

// --- one test per geometry: the harness runs them in parallel, and a failure names
// --- which geometry moved rather than stopping the sweep at the first.

#[test]
fn staked_s1_is_bit_identical() {
    identical_through(STAKED[0].0, &STAKED[0].1);
}

#[test]
fn staked_s2_is_bit_identical() {
    identical_through(STAKED[1].0, &STAKED[1].1);
}

#[test]
fn staked_s3_is_bit_identical() {
    identical_through(STAKED[2].0, &STAKED[2].1);
}

#[test]
fn staked_s4_is_bit_identical() {
    identical_through(STAKED[3].0, &STAKED[3].1);
}

#[test]
fn staked_s5_is_bit_identical() {
    identical_through(STAKED[4].0, &STAKED[4].1);
}

// ============================================================================
// The staked set's own preconditions, and the adapters' conventions.
// Cheap; they run whether or not the FCI legs do.
// ============================================================================

#[test]
fn every_staked_geometry_is_asymmetric() {
    assert_eq!(STAKED.len(), 5, "the brief stakes at least five geometries");
    for (label, g) in STAKED.iter() {
        let d = |a: usize, b: usize| {
            let (p, q) = (g[a], g[b]);
            ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
        };
        let r = [d(0, 1), d(0, 2), d(0, 3), d(1, 2), d(2, 3), d(3, 1)];
        for (i, a) in r.iter().enumerate() {
            assert!(*a > 0.9, "{label}: distance {i} is {a:.4}, atoms are on top of each other");
            assert!(*a < 9.0, "{label}: distance {i} is {a:.4}, outside the (H,H,H) domain");
            for b in r.iter().skip(i + 1) {
                assert!(
                    (a - b).abs() > 1e-3,
                    "{label}: two internal distances coincide ({a:.6}, {b:.6}) — a \
                     symmetric geometry cannot see a permutation defect"
                );
            }
        }
    }
}

#[test]
fn surface_families_key_on_the_sorted_z_triple() {
    let w = WaterTable::default();
    let t = TrimerTable::empty();
    let o = OohTable::empty();
    assert_eq!(SurfaceFamily::class(&w), ClusterClass::from_z([1, 1, 8]));
    assert_eq!(SurfaceFamily::class(&t), ClusterClass::from_z([1, 1, 1]));
    assert_eq!(SurfaceFamily::class(&o), ClusterClass::from_z([1, 8, 8]));
    // The canonical ORDER is the family's own, and is not the sorted key: water's first
    // two arguments are the O-H sides, so oxygen sits at canonical position 0.
    assert_eq!(SurfaceFamily::canonical_z(&w), [8, 1, 1]);
    assert_eq!(SurfaceFamily::canonical_z(&o), [1, 8, 8]);
}

#[test]
fn the_water_adapter_forwards_its_arguments_unpermuted() {
    let (w, _) = tables();
    for &(a, b, c) in &[(1.81f64, 1.93f64, 2.90f64), (2.40, 1.75, 3.10), (1.70, 2.85, 3.55)] {
        let direct = w.eval(a, b, c);
        let through = SurfaceFamily::eval_lex(w, [a, b, c]);
        same_bits("water value", direct.0, through.0);
        for i in 0..3 {
            same_bits(&format!("water grad[{i}]"), direct.1[i], through.1[i]);
        }
    }
}

#[test]
fn the_trimer_adapter_undoes_the_cycle_order_in_both_directions() {
    let (_, t) = tables();
    // Lexicographic [d01, d02, d12] enters TrimerTable::eval as the cycle
    // [d01, d12, d02], and the gradient comes back with its last two slots swapped.
    for &(d01, d02, d12) in &[(2.90f64, 3.32f64, 3.81f64), (1.95, 4.44, 3.23), (2.93, 3.92, 3.23)]
    {
        let direct = t.eval([d01, d12, d02]);
        let through = SurfaceFamily::eval_lex(t, [d01, d02, d12]);
        same_bits("trimer value", direct.0, through.0);
        same_bits("trimer grad d01", direct.1[0], through.1[0]);
        same_bits("trimer grad d02", direct.1[2], through.1[1]);
        same_bits("trimer grad d12", direct.1[1], through.1[2]);
    }
}

#[test]
fn the_ooh_adapter_forwards_its_arguments_unpermuted() {
    // A synthetic (O, O, H) table: the adapter's job is argument routing, and routing is
    // testable on any loaded surface. Generating the real one is thousands of FCI solves.
    let mut o = OohTable::empty();
    o.begin();
    for i in 0..holon_chem::ooh::N_NODES {
        // Deterministic, smooth, and NOT symmetric under swapping the two O-H sides, so
        // a transposed adapter would move the gradient.
        o.knot(i, ((i % 97) as f64) * 1e-4 - ((i % 13) as f64) * 3e-4);
    }
    assert!(o.finish(OohMeta::empty()), "the synthetic table must load");
    for &(a, b, c) in &[(1.90f64, 2.60f64, 2.70f64), (2.10, 1.85, 3.00)] {
        let direct = o.eval(a, b, c);
        let through = SurfaceFamily::eval_lex(&o, [a, b, c]);
        same_bits("ooh value", direct.0, through.0);
        for i in 0..3 {
            same_bits(&format!("ooh grad[{i}]"), direct.1[i], through.1[i]);
        }
    }
}
