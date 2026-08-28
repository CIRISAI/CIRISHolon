//! The model's own gates: the two CI routes against each other, the analytic derivatives
//! against difference quotients, the structural shape of the curve, and the table.
//!
//! These are DIFFERENT evidence from `referee.rs`. The referee gate says this
//! implementation agrees with another implementation of the same model; these say the
//! implementation is internally consistent and that the curve has the shape a bound
//! diatomic must have. Either can pass while the other fails, which is why both exist.

use holon_chem::h2::{fci_route_b, h2_pieces};
use holon_chem::table::{generate_table, grid_point, stream_table, PROVENANCE};
use holon_chem::{asymptote, equilibrium, h2_energy, h2_point};

/// Separations spanning the whole shipped range, including both ends.
fn probe_grid() -> Vec<f64> {
    let mut v = vec![0.3, 0.35, 0.5, 0.7, 0.9, 1.0, 1.2, 1.3886940180177756, 1.5];
    v.extend([1.8, 2.2, 2.6, 3.0, 4.0, 5.0, 6.5, 8.0, 9.0, 10.0]);
    v
}

#[test]
fn the_two_ci_routes_agree() {
    // Route (a) is the closed-form 2x2 reached through the Slater-Condon rules; route (b)
    // builds the six-determinant Hamiltonian from raw ladder operators and diagonalises
    // it. They share the integrals and nothing else, so this tests the CI algebra: a
    // wrong Slater-Condon coefficient or a dropped exchange term shows up here and
    // nowhere else in this file.
    let mut worst = 0.0f64;
    let mut at = 0.0f64;
    for r in probe_grid() {
        let d = (h2_energy(r) - fci_route_b(r)).abs();
        if d > worst {
            worst = d;
            at = r;
        }
    }
    println!("max |E_route_a - E_route_b| = {worst:.4e} hartree at R = {at}");
    assert!(worst < 1e-14, "the CI routes disagree by {worst:.3e} at R = {at}");
}

#[test]
fn the_analytic_derivatives_are_derivatives_of_this_function() {
    // A five-point central difference of the SAME evaluator. This is a weak instrument
    // on purpose and is used as one: its own truncation-plus-roundoff noise is around
    // 1e-11 relative, so it cannot certify the derivative to the precision the referee
    // gate does. What it CAN catch is the whole class of errors the referee gate would
    // also catch but this localises: a sign, a chain rule attached to the wrong operand,
    // a derivative rule that is right for a different function.
    let mut worst_d1 = 0.0f64;
    let mut worst_d2 = 0.0f64;
    for r in probe_grid() {
        let h = 1e-3;
        if r - 2.0 * h <= 0.0 {
            continue;
        }
        let e = |x: f64| h2_energy(x);
        let d1 =
            (e(r - 2.0 * h) - 8.0 * e(r - h) + 8.0 * e(r + h) - e(r + 2.0 * h)) / (12.0 * h);
        let d2 = (-e(r - 2.0 * h) + 16.0 * e(r - h) - 30.0 * e(r) + 16.0 * e(r + h)
            - e(r + 2.0 * h))
            / (12.0 * h * h);
        let p = h2_point(r);
        worst_d1 = worst_d1.max((p.slope() - d1).abs() / d1.abs().max(1e-3));
        worst_d2 = worst_d2.max((p.e2 - d2).abs() / d2.abs().max(1e-3));
    }
    println!("analytic vs 5-point difference: dE/dR rel {worst_d1:.2e}, d2E/dR2 rel {worst_d2:.2e}");
    assert!(worst_d1 < 1e-8, "dE/dR disagrees with the difference quotient");
    assert!(worst_d2 < 1e-5, "d2E/dR2 disagrees with the difference quotient");
}

#[test]
fn the_force_sign_convention_is_the_renderers() {
    // `F = -dE/dR`, so F > 0 on the repulsive branch (pushing the pair apart) and F < 0
    // on the attractive one. Getting this backwards produces a curve that still conserves
    // energy perfectly while simulating a mirror-image molecule -- the failure the
    // renderer's own residual check exists for, caught here at the source instead.
    let (r_e, _, _) = equilibrium();
    assert!(h2_point(0.5).f > 0.0, "the wall is not repulsive");
    assert!(h2_point(3.0).f < 0.0, "the tail is not attractive");
    assert!(
        h2_point(r_e).f.abs() < 1e-13,
        "the force does not vanish at R_e: {:.3e}",
        h2_point(r_e).f
    );
}

#[test]
fn the_curve_has_the_shape_a_bound_diatomic_must_have() {
    let (r_e, d_e, e_at_r_e) = equilibrium();
    let asym = asymptote();
    println!("R_e = {r_e:.15} bohr, D_e = {d_e:.15} hartree, E_asymptote = {asym:.15} hartree");

    // Exactly one minimum: dE/dR changes sign once, from - to +, over the whole range.
    let table = generate_table(0.3, 10.0, 492).expect("table");
    let mut minima = 0;
    let mut maxima = 0;
    for i in 1..table.r.len() {
        let (a, b) = (-table.f[i - 1], -table.f[i]);
        if a < 0.0 && b >= 0.0 {
            minima += 1;
        }
        if a > 0.0 && b <= 0.0 {
            maxima += 1;
        }
    }
    assert_eq!((minima, maxima), (1, 0), "the curve is not single-welled");

    // Bound: the minimum lies below the dissociation asymptote.
    assert!(d_e > 0.0, "the model does not bind");
    assert!(e_at_r_e < asym);

    // Approached from BELOW: everything past R_e is under the asymptote and rising.
    //
    // The tail is SELECTED first and then walked. Walking the whole grid and skipping
    // knots below R_e instead compares the first tail knot against a predecessor on the
    // OTHER side of the minimum, where E is under no obligation to be smaller — a false
    // failure that says nothing about the curve, and it fired here before this was fixed.
    let tail: Vec<usize> = (0..table.r.len()).filter(|&i| table.r[i] >= r_e).collect();
    assert!(tail.len() > 100, "the tail is too short to be evidence");
    for (k, &i) in tail.iter().enumerate() {
        assert!(table.e[i] < asym, "E exceeds the asymptote at R = {}", table.r[i]);
        if k > 0 {
            assert!(
                table.e[i] > table.e[tail[k - 1]],
                "E is not monotone on the tail at R = {}",
                table.r[i]
            );
        }
    }

    // Divergent as R -> 0: nuclear repulsion dominates and E rises without bound.
    let mut prev = h2_energy(0.3);
    for &r in &[0.2, 0.1, 0.05, 0.02, 0.01] {
        let e = h2_energy(r);
        assert!(e > prev, "E does not diverge as R -> 0 (at R = {r})");
        prev = e;
    }
    assert!(h2_energy(0.01) > 50.0, "the wall is too soft to be 1/R");
}

#[test]
fn the_asymptote_is_the_curve_it_claims_to_be() {
    // The dissociation limit is computed from the ONE-ATOM problem, so agreeing with the
    // far tail of the TWO-atom curve is a real check rather than a restatement: nothing
    // in `h_atom_energy` knows about H2.
    let asym = asymptote();
    let far = h2_energy(10.0);
    println!("E_asymptote = {asym:.15}, E(10 bohr) = {far:.15}, gap = {:.3e}", asym - far);
    assert!(asym - far > 0.0, "the tail is not approaching from below");
    assert!(asym - far < 1e-5, "the tail is still {:.3e} short at 10 bohr", asym - far);
}

#[test]
fn the_grid_is_uniform_in_the_variable_it_says_it_is() {
    // The knot rule is a derivation (equidistributing the Hermite error against the
    // nuclear repulsion's fourth derivative), and a derivation that is not what the code
    // does is a comment. u = R^{-1/4} must come out equally spaced.
    let (n, lo, hi) = (200usize, 0.3f64, 10.0f64);
    let us: Vec<f64> = (0..n)
        .map(|i| grid_point(lo, hi, n, i).powf(-0.25))
        .collect();
    let step = us[1] - us[0];
    for i in 1..n {
        let d = us[i] - us[i - 1];
        assert!(
            (d - step).abs() < 1e-12 * step.abs(),
            "grid is not uniform in R^-1/4 at knot {i}"
        );
    }
    assert_eq!(grid_point(lo, hi, n, 0), lo, "first knot is not r_min");
    assert_eq!(grid_point(lo, hi, n, n - 1), hi, "last knot is not r_max");
}

#[test]
fn the_streaming_and_collecting_paths_are_the_same_numbers() {
    // The browser uses `stream_table` (allocation-free) and the file fallback uses
    // `generate_table`. Two paths to one curve is two chances to be wrong, so they are
    // required to be bit-identical rather than close.
    let collected = generate_table(0.3, 10.0, 120).expect("table");
    let mut streamed = Vec::new();
    let meta = stream_table(0.3, 10.0, 120, |i, r, e, f, e2| {
        assert_eq!(i, streamed.len());
        streamed.push((r, e, f, e2));
        true
    })
    .expect("stream");
    for i in 0..120 {
        assert_eq!(
            (collected.r[i], collected.e[i], collected.f[i], collected.e2[i]),
            streamed[i],
            "streamed and collected knot {i} differ"
        );
    }
    assert_eq!(meta.r_e, collected.meta.r_e);
    assert_eq!(meta.d_e, collected.meta.d_e);
    assert_eq!(meta.e_asymptote, collected.meta.e_asymptote);
}

#[test]
fn a_refused_request_is_refused_rather_than_defaulted() {
    assert!(generate_table(0.3, 10.0, 1).is_none(), "n = 1 is not a grid");
    assert!(generate_table(0.0, 10.0, 10).is_none(), "R = 0 is not a separation");
    assert!(generate_table(10.0, 0.3, 10).is_none(), "the range is inverted");
    assert!(generate_table(f64::NAN, 10.0, 10).is_none(), "NaN is not a range");
}

#[test]
fn the_emitted_json_round_trips_bit_for_bit() {
    let t = generate_table(0.3, 10.0, 64).expect("table");
    let json = t.to_json();
    assert!(json.contains(PROVENANCE), "the JSON does not carry its provenance");
    // `{:?}` is the shortest representation that round-trips, so re-parsing must give
    // back exactly the f64 the physics produced -- otherwise the file-fallback path
    // simulates a slightly different curve from the engine-computed path.
    for (name, col) in [
        ("R_grid_bohr", &t.r),
        ("E_hartree", &t.e),
        ("F_hartree_per_bohr", &t.f),
        ("E2_hartree_per_bohr2", &t.e2),
    ] {
        let at = json.find(&format!("\"{name}\": [")).expect("column present")
            + name.len() + 5;
        let rest = &json[at..];
        let end = rest.find(']').unwrap();
        let parsed: Vec<f64> = rest[..end]
            .trim_start_matches('[')
            .split(',')
            .map(|s| s.trim().parse::<f64>().unwrap())
            .collect();
        assert_eq!(&parsed, col, "{name} did not round-trip");
    }
}

#[test]
fn the_interpolant_the_renderer_integrates_tracks_the_model() {
    // The renderer never evaluates this crate at run time: it integrates the cubic
    // Hermite interpolant built from the knots. So the number that matters for the
    // physics on screen is not the knot accuracy (1e-15) but the interpolant's departure
    // from the model BETWEEN knots, which is set by the grid density.
    for n in [246usize, 492, 984] {
        let t = generate_table(0.3, 10.0, n).expect("table");
        let (de, df) = t.hermite_error(7);
        println!("n = {n:>4}: interpolant max |dE| = {de:.3e} Eh, max |dF| = {df:.3e} Eh/a0");
        if n == 492 {
            assert!(de < 5e-9, "the shipped grid's interpolant is off by {de:.3e} Eh");
            assert!(df < 5e-6, "the shipped grid's interpolant force is off by {df:.3e}");
        }
    }
}

#[test]
fn the_pieces_add_up_to_the_total() {
    // `Pieces` exists so a caller can check the build rather than only its answer; that
    // is only true if the parts it exposes are the parts the answer is made of.
    for r in probe_grid() {
        let p = h2_pieces(r);
        assert_eq!(p.e_total.v, p.e_electronic.v + p.e_nuclear.v);
        assert_eq!(p.e_nuclear.v, 1.0 / r);
        // The AO overlap matrix is symmetric with unit diagonal by construction, and the
        // off-diagonal must fall monotonically to zero as the atoms separate.
        assert!((p.overlap[0][0].v - 1.0).abs() < 1e-15);
        assert_eq!(p.overlap[0][1].v, p.overlap[1][0].v);
        assert!(p.s_ab.v > 0.0 && p.s_ab.v < 1.0, "S_AB left (0,1) at R = {r}");
    }
    assert!(
        h2_pieces(10.0).s_ab.v < h2_pieces(1.0).s_ab.v,
        "overlap does not decrease with separation"
    );
}
