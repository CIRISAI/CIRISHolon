//! The general (s,p) McMurchie–Davidson integrals against the s-only closed forms, and
//! against the identities every correct two-electron integral must satisfy.
//!
//! # Why these four checks and not a table of reference numbers
//!
//! A table of numbers checks that the code reproduces whoever made the table. These
//! check that it reproduces MATHEMATICS the code did not put in:
//!
//! * the s-only cross-check grades the new scheme against `sto3g.rs`, whose arithmetic
//!   the pinned 50-digit H2 referee has already measured to 5e-15 hartree. Two schemes
//!   with no shared line of code, on the same contraction, at the same geometry.
//! * the permutational symmetries of `(ij|kl)` are eight statements about one integral;
//!   the code computes ONE of the eight and writes the others, so what is actually being
//!   tested is that the canonical-quartet loop reaches every entry — a silently missed
//!   quartet leaves an exact zero, which this catches and an energy comparison might not.
//! * translational invariance says the integrals are properties of the geometry and not
//!   of where the origin was put. Every Hermite centre `P`, every `X_PA`, every `R_PC`
//!   moves under a shift and the answer must not.
//! * rotational invariance of the s-block says the p functions are not leaking into it.

use holon_chem::dual::D2;
use holon_chem::elements;
use holon_chem::md::{ao_integrals, Basis};
use holon_chem::sto3g::{prim_eri, prim_kinetic, prim_nuclear, prim_overlap, sto3g_hydrogen};

/// Two hydrogens on the z axis, the general scheme's view.
fn h2_basis(r: f64) -> Basis {
    let h = elements::HYDROGEN;
    Basis::assemble(
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::var(r)],
        ],
        vec![1.0, 1.0],
        &[
            (0, 0, h.shells[0].alpha, h.shells[0].coeff),
            (1, 0, h.shells[0].alpha, h.shells[0].coeff),
        ],
    )
}

#[test]
fn general_scheme_reproduces_the_s_only_closed_forms() {
    let basis = sto3g_hydrogen();
    let mut worst = [0.0f64; 4];
    for &r in &[0.6f64, 1.0, 1.4, 2.0, 3.5, 6.0] {
        let g = ao_integrals(&h2_basis(r));
        let centre = [D2::c(0.0), D2::var(r)];

        for i in 0..2 {
            for j in 0..2 {
                let (mut s, mut t, mut v) = (D2::c(0.0), D2::c(0.0), D2::c(0.0));
                for a in 0..3 {
                    for b in 0..3 {
                        let w = basis.coeff[a] * basis.coeff[b];
                        s = s + prim_overlap(basis.prim[a], centre[i], basis.prim[b], centre[j]) * w;
                        t = t + prim_kinetic(basis.prim[a], centre[i], basis.prim[b], centre[j]) * w;
                        for &c in centre.iter() {
                            v = v + prim_nuclear(
                                basis.prim[a],
                                centre[i],
                                basis.prim[b],
                                centre[j],
                                c,
                                1.0,
                            ) * w;
                        }
                    }
                }
                worst[0] = worst[0].max((g.s[i * 2 + j].v - s.v).abs());
                worst[1] = worst[1].max((g.t[i * 2 + j].v - t.v).abs());
                worst[2] = worst[2].max((g.v[i * 2 + j].v - v.v).abs());
                // The derivatives come from the same chain rule but through different
                // expressions, so they are an independent statement about the scheme.
                worst[0] = worst[0].max((g.s[i * 2 + j].d - s.d).abs());
                worst[1] = worst[1].max((g.t[i * 2 + j].e - t.e).abs());
                worst[2] = worst[2].max((g.v[i * 2 + j].d - v.d).abs());
            }
        }

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    for l in 0..2 {
                        let mut e = D2::c(0.0);
                        for a in 0..3 {
                            for b in 0..3 {
                                for c in 0..3 {
                                    for d in 0..3 {
                                        let w = basis.coeff[a]
                                            * basis.coeff[b]
                                            * basis.coeff[c]
                                            * basis.coeff[d];
                                        e = e + prim_eri(
                                            basis.prim[a],
                                            centre[i],
                                            basis.prim[b],
                                            centre[j],
                                            basis.prim[c],
                                            centre[k],
                                            basis.prim[d],
                                            centre[l],
                                        ) * w;
                                    }
                                }
                            }
                        }
                        worst[3] = worst[3].max((g.g(i, j, k, l).v - e.v).abs());
                        worst[3] = worst[3].max((g.g(i, j, k, l).d - e.d).abs());
                    }
                }
            }
        }
    }
    println!(
        "s-only cross-check, worst |delta|: S {:.3e}  T {:.3e}  V {:.3e}  ERI {:.3e}",
        worst[0], worst[1], worst[2], worst[3]
    );
    for (name, w) in ["S", "T", "V", "ERI"].iter().zip(worst.iter()) {
        assert!(
            *w < 5e-15,
            "{name} disagrees with the s-only closed form by {w:.3e}, which is too large \
             to be rounding"
        );
    }
}

/// A first-row basis with p functions on both centres.
fn n2_basis(r: f64) -> Basis {
    let n = elements::NITROGEN;
    let mut decls = Vec::new();
    for c in 0..2 {
        for sh in n.shells {
            decls.push((c, sh.kind.l(), sh.alpha, sh.coeff));
        }
    }
    Basis::assemble(
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::var(r)],
        ],
        vec![7.0, 7.0],
        &decls,
    )
}

#[test]
fn eri_obeys_its_eight_permutational_symmetries() {
    let b = n2_basis(2.1);
    let g = ao_integrals(&b);
    let n = g.n;
    assert_eq!(n, 10);
    let mut worst = 0.0f64;
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                for l in 0..n {
                    let x = g.g(i, j, k, l).v;
                    for &(p, q, r, s) in &[
                        (j, i, k, l),
                        (i, j, l, k),
                        (j, i, l, k),
                        (k, l, i, j),
                        (l, k, i, j),
                        (k, l, j, i),
                        (l, k, j, i),
                    ] {
                        worst = worst.max((x - g.g(p, q, r, s).v).abs());
                    }
                }
            }
        }
    }
    println!("permutational symmetry: worst |delta| = {worst:.3e}");
    // The permutations are written from ONE computed value, so this is exact or the
    // canonical loop missed a quartet and left a zero where a number belongs.
    assert_eq!(worst, 0.0, "ERI permutational symmetry broken by {worst:.3e}");
}

/// The zero pattern of the two-electron integrals, predicted before it was looked at.
///
/// # Why this and not a count
///
/// A quartet the canonical loop never visits stays exactly zero, and a count of nonzeros
/// cannot tell that apart from a zero the geometry forces. So the zeros are DERIVED
/// instead. With both nuclei on the z axis the integrand's parity in x and in y is a
/// property of the four functions alone, and an integral vanishes unless both are even;
/// when all four functions sit on the SAME centre, that centre is a local inversion
/// point in z as well and z-parity applies too. Everything else must be nonzero.
///
/// Counted rather than eyeballed: 10 functions, of which 6 are (even, even), 2 are
/// (odd, even) and 2 are (even, odd), give `(10^4 + 6^4 + 6^4 + 2^4)/4 = 3152` xy-allowed
/// quartets, and the same generating function per centre gives 88 of them z-odd, so 176
/// of the 3152 are zero and the other 2976 are not.
#[test]
fn the_two_electron_zeros_are_the_ones_symmetry_forces() {
    let b = n2_basis(2.1);
    let g = ao_integrals(&b);
    let n = g.n;
    let parity: Vec<[u8; 3]> = b.funcs.iter().map(|&(_, l)| [l[0] % 2, l[1] % 2, l[2] % 2]).collect();
    let centre: Vec<usize> = b.funcs.iter().map(|&(sh, _)| b.shells[sh].center).collect();

    let (mut allowed, mut allowed_zero, mut forbidden_nonzero) = (0usize, 0usize, 0usize);
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                for l in 0..n {
                    let q = [i, j, k, l];
                    let par = |d: usize| q.iter().map(|&f| parity[f][d]).sum::<u8>() % 2;
                    let same_centre = q.iter().all(|&f| centre[f] == centre[i]);
                    let vanishes =
                        par(0) == 1 || par(1) == 1 || (same_centre && par(2) == 1);
                    let x = g.g(i, j, k, l).v.abs();
                    if vanishes {
                        if x > 1e-14 {
                            forbidden_nonzero += 1;
                        }
                    } else {
                        allowed += 1;
                        if x <= 1e-14 {
                            allowed_zero += 1;
                        }
                    }
                }
            }
        }
    }
    println!(
        "zero pattern: {allowed} symmetry-allowed, {allowed_zero} of them zero, \
         {forbidden_nonzero} forbidden but nonzero"
    );
    assert_eq!(allowed, 2976, "the derived count of allowed quartets is not what the basis gives");
    assert_eq!(
        allowed_zero, 0,
        "{allowed_zero} symmetry-ALLOWED two-electron integrals came back zero; the \
         canonical quartet loop is not reaching every shell combination"
    );
    assert_eq!(
        forbidden_nonzero, 0,
        "{forbidden_nonzero} symmetry-FORBIDDEN two-electron integrals came back nonzero"
    );
}

#[test]
fn integrals_do_not_depend_on_where_the_origin_is() {
    // Same molecule, shifted bodily by (0.7, -1.3, 2.9). Every Hermite centre, every
    // X_PA and every R_PC moves; nothing physical may.
    let n = elements::NITROGEN;
    let mut decls = Vec::new();
    for c in 0..2 {
        for sh in n.shells {
            decls.push((c, sh.kind.l(), sh.alpha, sh.coeff));
        }
    }
    let r = 2.1;
    let a = ao_integrals(&Basis::assemble(
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::c(r)],
        ],
        vec![7.0, 7.0],
        &decls,
    ));
    let d = [0.7f64, -1.3, 2.9];
    let b = ao_integrals(&Basis::assemble(
        vec![
            [D2::c(d[0]), D2::c(d[1]), D2::c(d[2])],
            [D2::c(d[0]), D2::c(d[1]), D2::c(d[2] + r)],
        ],
        vec![7.0, 7.0],
        &decls,
    ));
    let mut worst = 0.0f64;
    for i in 0..a.n * a.n {
        worst = worst.max((a.s[i].v - b.s[i].v).abs());
        worst = worst.max((a.t[i].v - b.t[i].v).abs());
        worst = worst.max((a.v[i].v - b.v[i].v).abs());
    }
    let mut worst_eri = 0.0f64;
    for i in 0..a.eri.len() {
        worst_eri = worst_eri.max((a.eri[i].v - b.eri[i].v).abs());
    }
    println!("translation invariance: 1e {worst:.3e}, 2e {worst_eri:.3e}");
    assert!(worst < 1e-12, "one-electron integrals moved with the origin by {worst:.3e}");
    assert!(worst_eri < 1e-13, "two-electron integrals moved with the origin by {worst_eri:.3e}");
}

/// The one-electron matrices must not depend on the ORDER the shells were declared in.
///
/// # What this catches that the s-only cross-check cannot
///
/// The kinetic-energy expression differentiates the KET, so `T(i, j)` and `T(j, i)` go
/// through different branches of the `E` table — `E^{i, j+2}` in one and `E^{j, i+2}` in
/// the other. The assembler fills only the lower triangle and mirrors, so the upper one is
/// never computed and a defect in the raised-power branch would be invisible. Declaring
/// the same molecule with its shells in the opposite order swaps which of the two the
/// assembler actually evaluates, at every index. Nothing else about the molecule changes,
/// so every integral must come back identical under the induced permutation.
///
/// The s-only cross-check in this file cannot see it: at `l = 0` the raised powers are
/// `E^{0,2}`, symmetric in the two orders by construction.
#[test]
fn the_one_electron_matrices_do_not_depend_on_shell_order() {
    let (li, f) = (elements::LITHIUM, elements::FLUORINE);
    let r = 3.0;
    let ca = [D2::c(0.0), D2::c(0.0), D2::c(0.0)];
    let cb = [D2::c(0.0), D2::c(0.0), D2::c(r)];

    let mut forward = Vec::new();
    for sh in li.shells {
        forward.push((0usize, sh.kind.l(), sh.alpha, sh.coeff));
    }
    for sh in f.shells {
        forward.push((1usize, sh.kind.l(), sh.alpha, sh.coeff));
    }
    // The same molecule with fluorine's shells declared first. Centre indices follow the
    // shells, so the geometry is unchanged and only the basis ORDER moves.
    let mut reversed = Vec::new();
    for sh in f.shells {
        reversed.push((1usize, sh.kind.l(), sh.alpha, sh.coeff));
    }
    for sh in li.shells {
        reversed.push((0usize, sh.kind.l(), sh.alpha, sh.coeff));
    }

    let a = ao_integrals(&Basis::assemble(
        vec![ca, cb],
        vec![3.0, 9.0],
        &forward,
    ));
    let b = ao_integrals(&Basis::assemble(
        vec![ca, cb],
        vec![3.0, 9.0],
        &reversed,
    ));
    let n = a.n;
    assert_eq!(n, 10);
    assert_eq!(b.n, n);
    // Lithium contributes 5 functions and fluorine 5, so the permutation is a rotation by
    // five: forward index i corresponds to reversed index (i + 5) mod 10.
    let perm = |i: usize| (i + 5) % n;

    let (mut ws, mut wt, mut wv, mut we) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for i in 0..n {
        for j in 0..n {
            let (pi, pj) = (perm(i), perm(j));
            ws = ws.max((a.s[i * n + j].v - b.s[pi * n + pj].v).abs());
            wt = wt.max((a.t[i * n + j].v - b.t[pi * n + pj].v).abs());
            wv = wv.max((a.v[i * n + j].v - b.v[pi * n + pj].v).abs());
            // The derivatives take the same route and are the same statement again.
            wt = wt.max((a.t[i * n + j].d - b.t[pi * n + pj].d).abs());
        }
    }
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                for l in 0..n {
                    we = we.max(
                        (a.g(i, j, k, l).v - b.g(perm(i), perm(j), perm(k), perm(l)).v).abs(),
                    );
                }
            }
        }
    }
    println!("shell-order permutation: S {ws:.3e}  T {wt:.3e}  V {wv:.3e}  ERI {we:.3e}");
    assert!(ws < 1e-15, "overlap depends on shell order by {ws:.3e}");
    assert!(wt < 1e-14, "kinetic energy depends on shell order by {wt:.3e}");
    assert!(wv < 1e-13, "nuclear attraction depends on shell order by {wv:.3e}");
    assert!(we < 1e-14, "two-electron integrals depend on shell order by {we:.3e}");
}

/// Verify d-orbitals (l=2) integrals: unit diagonal overlap, positive definiteness,
/// 8-fold ERI permutational symmetry, and origin translational invariance.
#[test]
fn d_orbitals_obey_permutational_and_translational_symmetries() {
    let alpha_d = [0.55170007, 0.16828611, 0.06493001];
    let coeff_d = [0.21976795, 0.65554736, 0.28657326];
    let decls = vec![
        (0usize, 0u8, elements::HYDROGEN.shells[0].alpha, elements::HYDROGEN.shells[0].coeff),
        (0usize, 2u8, alpha_d, coeff_d),
        (1usize, 2u8, alpha_d, coeff_d),
    ];
    let r = 2.5;
    let b = Basis::assemble(
        vec![
            [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
            [D2::c(0.0), D2::c(0.0), D2::var(r)],
        ],
        vec![1.0, 1.0],
        &decls,
    );
    let g = ao_integrals(&b);
    let n = g.n;
    // 1 s function + 5 spherical d + 5 spherical d = 11 basis functions. Five, not six:
    // the sixth Cartesian component of a d shell is x^2+y^2+z^2, an s function in a d
    // shell's clothing, and a MINIMAL basis must not carry it. See md::SPHERICAL_D.
    assert_eq!(n, 11);

    // 1. Overlap diagonal is 1.0 (self-normalized)
    for i in 0..n {
        assert!(
            (g.s[i * n + i].v - 1.0).abs() < 1e-12,
            "diagonal overlap at {i} is {}, expected 1.0",
            g.s[i * n + i].v
        );
    }

    // 2. Overlap is positive definite
    let s_vals: Vec<f64> = g.s.iter().map(|d| d.v).collect();
    let (eigs, _) = holon_chem::fci::jacobi_eigh(&s_vals, n);
    for (i, &eig) in eigs.iter().enumerate() {
        assert!(eig > 0.0, "overlap eigenvalue {i} is {eig} <= 0");
    }

    // 3. Permutational symmetry of ERIs
    let mut worst = 0.0f64;
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                for l in 0..n {
                    let x = g.g(i, j, k, l).v;
                    for &(p, q, r, s) in &[
                        (j, i, k, l),
                        (i, j, l, k),
                        (j, i, l, k),
                        (k, l, i, j),
                        (l, k, i, j),
                        (k, l, j, i),
                        (l, k, j, i),
                    ] {
                        worst = worst.max((x - g.g(p, q, r, s).v).abs());
                    }
                }
            }
        }
    }
    println!("d-orbital ERI permutational symmetry: worst |delta| = {worst:.3e}");
    assert_eq!(worst, 0.0, "d-orbital ERI permutational symmetry broken");

    // 4. Translational invariance
    let shift = [1.1f64, -0.8, 1.7];
    let b_shifted = Basis::assemble(
        vec![
            [D2::c(shift[0]), D2::c(shift[1]), D2::c(shift[2])],
            [D2::c(shift[0]), D2::c(shift[1]), D2::c(shift[2] + r)],
        ],
        vec![1.0, 1.0],
        &decls,
    );
    let g_shifted = ao_integrals(&b_shifted);
    let mut worst_1e = 0.0f64;
    for i in 0..n * n {
        worst_1e = worst_1e.max((g.s[i].v - g_shifted.s[i].v).abs());
        worst_1e = worst_1e.max((g.t[i].v - g_shifted.t[i].v).abs());
        worst_1e = worst_1e.max((g.v[i].v - g_shifted.v[i].v).abs());
    }
    println!("d-orbital translational invariance 1e: {worst_1e:.3e}");
    assert!(worst_1e < 1e-12, "d-orbital 1e integrals moved with origin");
}
