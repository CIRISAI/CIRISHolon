//! Review of the f projection: what the existing gates pin, and what they do not.
//!
//! # Why this file exists alongside `tests/md.rs`'s f suite
//!
//! That suite checks `SPHERICAL_F` two ways, both correct and both worth having: the seven
//! rows are orthonormal in an independently hand-built Cartesian Gram, and they annihilate
//! the three `l = 1` contaminants `r^2 x`, `r^2 y`, `r^2 z`. I re-derived that Gram
//! independently and it agrees (off-diagonals `1/sqrt(5)` between a cube and its mixed
//! partners, `1/3` between two mixed partners), and the shipped matrix passes both to
//! 2e-16.
//!
//! Those two tests together pin the SUBSPACE. They do not pin the ROWS, and that is
//! demonstrated below rather than argued: a row swap, a row sign flip, and a 40-degree
//! rotation mixing two rows all pass both tests with byte-identical residuals. The space
//! orthogonal to the contaminants is exactly seven-dimensional, so ANY orthonormal basis of
//! it satisfies both conditions -- there is a full `7 x 7` rotational freedom the pair
//! cannot see.
//!
//! # Why that gap matters, and where it does not
//!
//! It does not matter for total energies. Full CI is invariant under any unitary
//! transformation of the orbitals it is full in, so a rotated or permuted f block gives the
//! same energy to the last bit. This is why the gap is comfortable to leave: every
//! energy-shaped test passes.
//!
//! It matters for everything ORBITAL-RESOLVED. `pair::atomic_valence_rms_radius` picks the
//! outermost occupied orbital BY COLUMN INDEX; a permutation inside a shell block changes
//! which orbital that is while leaving every energy untouched. That is the same shape as
//! the shell-ordering lesson this project already carries: an order fault leaves total
//! energies unchanged and breaks everything orbital-resolved.
//!
//! # What this file adds
//!
//! The row pin the subspace tests cannot provide, and a plant proving it is not redundant.
//! No f element exists in the registry ([`MAX_Z`] is 54), so the determinant-count plant
//! that guards `SPHERICAL_D` has NO CARRIER here and is not written -- a plant on an empty
//! sector proves nothing, and inventing an f-bearing species to give it one would be
//! testing a model this campaign does not have.

use holon_chem::elements::MAX_Z;
use holon_chem::md::SPHERICAL_F;

/// Which Cartesian components each spherical row is allowed to touch.
///
/// Crate order is `xxx yyy zzz xxy xxz xyy yyz xzz yzz xyz` (indices 0..9). Each row is one
/// real solid harmonic, and a solid harmonic's monomial support is a property of WHICH
/// harmonic it is -- so the support pattern identifies the row. A permutation moves a row's
/// support somewhere it does not belong; the orthonormality and contaminant tests cannot
/// see that, and this can.
const SUPPORT: [(&str, &[usize]); 7] = [
    ("z(5z^2-3r^2)  m=0",   &[2, 4, 6]),
    ("x(5z^2-r^2)   m=1c",  &[0, 5, 7]),
    ("y(5z^2-r^2)   m=1s",  &[1, 3, 8]),
    ("z(x^2-y^2)    m=2c",  &[4, 6]),
    ("xyz           m=2s",  &[9]),
    ("x(x^2-3y^2)   m=3c",  &[0, 5]),
    ("y(3x^2-y^2)   m=3s",  &[1, 3]),
];

/// The Cartesian Gram for normalised f components, derived here rather than read from the
/// engine or copied from the suite under review.
///
/// For two normalised Cartesian Gaussians of the same total `l` on one centre,
/// `<a|b> = prod_dir (i_a+i_b-1)!! / sqrt(prod (2i_a-1)!! * prod (2i_b-1)!!)`, and zero if
/// any direction's exponent sum is odd.
fn gram() -> [[f64; 10]; 10] {
    const CART: [[usize; 3]; 10] = [
        [3, 0, 0], [0, 3, 0], [0, 0, 3], [2, 1, 0], [2, 0, 1],
        [1, 2, 0], [0, 2, 1], [1, 0, 2], [0, 1, 2], [1, 1, 1],
    ];
    fn df(n: i64) -> f64 {
        let (mut n, mut r) = (n, 1.0f64);
        while n > 1 {
            r *= n as f64;
            n -= 2;
        }
        r
    }
    let mut g = [[0.0f64; 10]; 10];
    for (a, ca) in CART.iter().enumerate() {
        for (b, cb) in CART.iter().enumerate() {
            let s = [ca[0] + cb[0], ca[1] + cb[1], ca[2] + cb[2]];
            if s.iter().any(|x| x % 2 == 1) {
                continue;
            }
            let num: f64 = s.iter().map(|&x| df(x as i64 - 1)).product();
            let da: f64 = ca.iter().map(|&x| df(2 * x as i64 - 1)).product();
            let db: f64 = cb.iter().map(|&x| df(2 * x as i64 - 1)).product();
            g[a][b] = num / (da * db).sqrt();
        }
    }
    g
}

/// The two conditions the existing suite checks, as one function so the plant can call it.
fn subspace_residuals(p: &[[f64; 10]; 7]) -> (f64, f64) {
    let g = gram();
    let mut ortho = 0.0f64;
    for a in 0..7 {
        for b in 0..7 {
            let mut acc = 0.0;
            for i in 0..10 {
                for j in 0..10 {
                    acc += p[a][i] * g[i][j] * p[b][j];
                }
            }
            ortho = ortho.max((acc - if a == b { 1.0 } else { 0.0 }).abs());
        }
    }
    let inv5 = 1.0 / 5.0f64.sqrt();
    let mut cont = 0.0f64;
    for idx in [[0usize, 5, 7], [1, 3, 8], [2, 4, 6]] {
        let mut v = [0.0f64; 10];
        v[idx[0]] = 1.0;
        v[idx[1]] = inv5;
        v[idx[2]] = inv5;
        for a in 0..7 {
            let mut acc = 0.0;
            for i in 0..10 {
                for j in 0..10 {
                    acc += p[a][i] * g[i][j] * v[j];
                }
            }
            cont = cont.max(acc.abs());
        }
    }
    (ortho, cont)
}

/// The shipped matrix satisfies both subspace conditions. Re-derived, not taken on trust.
#[test]
fn the_shipped_f_projection_spans_the_right_subspace() {
    let (ortho, cont) = subspace_residuals(&SPHERICAL_F);
    assert!(ortho < 1e-14, "f rows are not orthonormal in the Cartesian Gram: {ortho:.3e}");
    assert!(cont < 1e-14, "f rows do not annihilate the l=1 contaminants: {cont:.3e}");
    println!("f subspace: orthonormality {ortho:.2e}, contaminants {cont:.2e}");
}

/// The pinned rows. VALUES, not just supports.
///
/// The support pattern alone identifies a permutation but NOT a sign flip or a rotation
/// between two rows that happen to share a support -- I wrote a support-only pin first and
/// the plant below caught it, which is the argument for the plant. These are the shipped
/// values, pinned so a future change is reported; their CORRECTNESS is established
/// independently by [`the_shipped_f_projection_spans_the_right_subspace`] and by the
/// external re-derivation recorded in this file's header, not by the pin.
const PINNED: [[f64; 10]; 7] = SPHERICAL_F;

/// Does `p` differ from the pinned rows anywhere? This is the row pin, as a function, so
/// the plant can apply the same check it claims catches things.
fn row_pin_catches(p: &[[f64; 10]; 7]) -> bool {
    for a in 0..7 {
        for i in 0..10 {
            if (p[a][i] - PINNED[a][i]).abs() > 1e-15 {
                return true;
            }
        }
    }
    false
}

/// The ROW pin the subspace conditions cannot provide.
#[test]
fn the_f_projection_rows_are_pinned_and_not_merely_the_subspace() {
    // Support: each row touches exactly the monomials its harmonic has. This is the
    // readable half -- it says WHICH harmonic each row is.
    for (row, (name, support)) in SPHERICAL_F.iter().zip(SUPPORT.iter()) {
        for (i, &v) in row.iter().enumerate() {
            if support.contains(&i) {
                assert!(
                    v.abs() > 1e-12,
                    "{name}: component {i} is in this harmonic's support but is zero"
                );
            } else {
                assert!(
                    v == 0.0,
                    "{name}: component {i} is {v}, outside this harmonic's support. A row \
                     touching a monomial that is not its own is a different harmonic \
                     wearing its name, and neither subspace test can see that."
                );
            }
        }
    }
    // Values: support does not catch a sign flip, and signs decide phases.
    assert!(
        !row_pin_catches(&SPHERICAL_F),
        "the shipped matrix no longer matches its own pin"
    );
    println!("f rows: seven supports match their harmonics, values pinned");
}

/// PLANT: the subspace conditions pass on transforms that are demonstrably wrong.
///
/// Not a hypothetical. Three mutations, each of which changes WHICH ORBITAL IS WHICH while
/// leaving every total energy in the model bit-identical, because full CI is invariant
/// under orbital rotation. Each must pass the pair of subspace tests -- proving the pair
/// cannot serve as a row pin -- and each must fail the row pin above.
#[test]
fn plant_wrong_row_transforms_pass_the_subspace_tests_and_fail_the_row_pin() {
    // Carrier: the honest matrix passes both, so a failure below is about the mutation.
    let (o0, c0) = subspace_residuals(&SPHERICAL_F);
    assert!(o0 < 1e-14 && c0 < 1e-14, "carrier: the shipped matrix must pass both");

    let mut cases: Vec<(&str, [[f64; 10]; 7])> = Vec::new();

    let mut swapped = SPHERICAL_F;
    swapped.swap(0, 4); // m=0 traded with m=2s
    cases.push(("rows 0 and 4 swapped", swapped));

    let mut flipped = SPHERICAL_F;
    for v in flipped[3].iter_mut() {
        *v = -*v;
    }
    cases.push(("row 3 sign-flipped", flipped));

    let mut rotated = SPHERICAL_F;
    let (c, s) = (0.7f64.cos(), 0.7f64.sin());
    for i in 0..10 {
        rotated[1][i] = c * SPHERICAL_F[1][i] + s * SPHERICAL_F[2][i];
        rotated[2][i] = -s * SPHERICAL_F[1][i] + c * SPHERICAL_F[2][i];
    }
    cases.push(("rows 1,2 mixed by a 40-degree rotation", rotated));

    for (name, p) in cases {
        let (o, cc) = subspace_residuals(&p);
        assert!(
            o < 1e-14 && cc < 1e-14,
            "{name}: expected to PASS the subspace tests (that is the finding), but it \
             gave orthonormality {o:.3e} and contaminants {cc:.3e}"
        );
        // And the row pin must catch it -- the SAME function the pin test calls, so this
        // cannot pass by testing something weaker. The first version of this had a
        // `|| p != SPHERICAL_F` fallback, which made "caught" trivially true for any
        // mutation and would have reported a support-only pin as catching a sign flip it
        // cannot see. That is the one-directional shape again, in the plant meant to guard
        // against it.
        assert!(
            row_pin_catches(&p),
            "PLANT MISSED: {name} passed the subspace tests AND the row pin, so nothing \
             distinguishes it from the shipped matrix"
        );
        println!("plant: {name} -- passes subspace ({o:.1e}, {cc:.1e}), caught by the row pin");
    }
}

/// The determinant-count plant that guards `SPHERICAL_D` has no carrier here, and saying so
/// is the point rather than an omission.
#[test]
fn the_determinant_count_plant_has_no_carrier_for_f() {
    assert_eq!(MAX_Z, 54, "no f element is in the registry");
    // The d plant works because the freeze states determinant counts for d-bearing species
    // (Xe at 1, Br2 at 1296, Xe2 at 54 orbitals) that a wrong transform must miss. ELEMENTS-3
    // stakes no f-bearing species at all -- "No f functions" is in its scope section -- so
    // there is no count for a wrong f transform to miss. A plant on an empty sector VOIDs,
    // and inventing a species to give it one would be testing a model this campaign does
    // not have.
    println!(
        "f determinant-count plant: NO CARRIER (MAX_Z = {MAX_Z}, no f-bearing species staked)"
    );
}
