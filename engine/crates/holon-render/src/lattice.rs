//! SEEDED ICE: deterministic crystal initial conditions for the three ordered
//! polymorphs the engine can actually be pointed at.
//!
//! Each builder returns a SCENE SPEC — one `(Z, x, y, z)` per atom in bohr — together
//! with the periodic box `(Lx, Ly, Lz)` the spec was written into. Nothing here touches
//! the dynamics, the bank, or the ledger: this module places nuclei and hands them over.
//!
//! # What is and is not built here
//!
//! All three builders return PROTON-ORDERED crystals. That is the whole of what they
//! claim. In particular `ice_xi` is the ordered representative of the Ih family and
//! **disordered Ih is NOT built here** — a Bernal–Fowler disorder sampler (a random or
//! loop-swept draw from the exponentially many ice-rule-satisfying proton arrangements)
//! is a different object and is out of scope. Calling what this builds "ice Ih" would be
//! the unconditional statement the discipline bans: the residual proton entropy of real
//! Ih, k ln(3/2) per molecule, is exactly the thing an ordered seed does not have.
//!
//! # Determinism
//!
//! No RNG, no clock, no hash iteration order. The same `(cells, s)` gives the same
//! `Vec` in the same order, byte for byte, on every run and every host. `s` is a LINEAR
//! scale factor: every length below — cell edges, box edges, O–H bonds — is multiplied
//! by it, so `s` is a uniform dilation of one fixed structure and the geometry gates
//! read the same numbers at every `s`.
//!
//! # Periodicity
//!
//! The box is always an integer number of cells, so the structure tiles it exactly. An
//! atom's hydrogen may be placed across a face; every returned coordinate is wrapped
//! back into `[0, L)` per axis, and every distance the caller measures must therefore be
//! a minimum-image distance. `wrap` guarantees the half-open interval rather than
//! `rem_euclid`'s occasional exact `L` on a tiny negative input.
//!
//! # Units and provenance of the constants
//!
//! Lengths are bohr. The lattice constants below are the conventional experimental
//! values for each polymorph, converted at 1 bohr = 0.529177 Å and rounded to the digits
//! shown — DECLARED INPUTS, not outputs of anything this crate computes. The O–H
//! covalent length 1.81 bohr (0.958 Å) is the gas-phase monomer value, used unchanged in
//! all three ordered seeds; a seed is a starting point for a relaxation, not a
//! measurement of the crystal's bond length.

/// One atom of a scene spec: nuclear charge, then position in bohr.
pub type SpecAtom = (u8, f64, f64, f64);

/// A scene spec and the periodic box it lives in, both in bohr.
pub type IceScene = (Vec<SpecAtom>, (f64, f64, f64));

/// Ice XI / Ih hexagonal `a` = 4.5 Å.
pub const A_ICE_XI_BOHR: f64 = 8.504;
/// Ice XI / Ih hexagonal `c` = 7.32 Å.
pub const C_ICE_XI_BOHR: f64 = 13.833;
/// Ice VIII: edge of the CUBIC CELL THAT HOLDS 16 OXYGENS, 6.36 Å.
///
/// This is the cubic-ice (diamond) conventional cell of one sublattice, NOT the bcc cell
/// of the combined oxygen arrangement — that one is half this. Stating which cell the
/// number is the edge of is load-bearing: the two readings differ by a factor of two and
/// give O–O separations 2.75 Å apart.
pub const A_ICE_VIII_BOHR: f64 = 12.02;
/// Ice X: edge of the same 16-oxygen cubic cell, 5.29 Å.
pub const A_ICE_X_BOHR: f64 = 10.0;
/// The covalent O–H length used by `ice_xi` and `ice_viii`, 0.958 Å.
///
/// `ice_x` does NOT use it: symmetric ice has no covalent/H-bond distinction, which is
/// the polymorph's defining property.
pub const R_OH_BOHR: f64 = 1.81;

const Z_H: u8 = 1;
const Z_O: u8 = 8;

/// Fold a coordinate into `[0, l)`.
///
/// `rem_euclid` can return exactly `l` when its input is a negative number too small to
/// resolve, which would put an atom ON the far face rather than inside the box; the
/// second line is what makes the interval half-open in fact and not just in intent.
fn wrap(v: f64, l: f64) -> f64 {
    let w = v.rem_euclid(l);
    if w < l {
        w
    } else {
        0.0
    }
}

/// Place a hydrogen at `r` along `dir` from `o`. `dir` need not be normalised.
fn h_along(out: &mut Vec<SpecAtom>, o: (f64, f64, f64), dir: (f64, f64, f64), r: f64) {
    let n = (dir.0 * dir.0 + dir.1 * dir.1 + dir.2 * dir.2).sqrt();
    out.push((
        Z_H,
        o.0 + r * dir.0 / n,
        o.1 + r * dir.1 / n,
        o.2 + r * dir.2 / n,
    ));
}

fn wrap_all(mut atoms: Vec<SpecAtom>, b: (f64, f64, f64)) -> Vec<SpecAtom> {
    for a in atoms.iter_mut() {
        a.1 = wrap(a.1, b.0);
        a.2 = wrap(a.2, b.1);
        a.3 = wrap(a.3, b.2);
    }
    atoms
}

// ---------------------------------------------------------------- ice XI (Ih family)

/// ICE XI: the PROTON-ORDERED REPRESENTATIVE of the Ih family. **Disordered Ih is NOT
/// built here** — see the module header; a Bernal–Fowler disorder sampler is out of
/// scope and this function must never be described as producing one.
///
/// # The oxygen sublattice
///
/// Wurtzite, the ice-Ih oxygen arrangement, written in its ORTHORHOMBIC cell rather than
/// its hexagonal one so that the periodic box is a box: edges `(a, a√3, c)` holding 8
/// oxygens, which is exactly two hexagonal cells. At `s = 1`, `a` = 8.504 bohr and
/// `c` = 13.833 bohr.
///
/// Each oxygen has four neighbours: ONE along `c` (length `3c/8` = 5.187 bohr) and THREE
/// lateral (length `√(a²/3 + c²/64)` = 5.205 bohr). Those two lengths differ by 0.35%
/// because the real `c/a` = 1.627 is not the ideal `√(8/3)` = 1.633 — the tetrahedron is
/// very slightly squashed, as it is in the mineral. A gate that demands ONE
/// nearest-neighbour distance must therefore carry a tolerance wider than 0.35%.
///
/// Sites split into two kinds by which way the `c`-axis bond points: kind A (`z = 0` and
/// `z = c/2`) bonds UP along `c` and laterally DOWN; kind B (`z = 3c/8`, `7c/8`) bonds
/// DOWN along `c` and laterally UP. The lateral in-plane offsets are `±(d1, d2, d3)`,
/// the sign fixed by which of the two hexagonal columns the site sits in — getting that
/// sign from the site's KIND instead of its COLUMN is a real and silent bug: it leaves
/// every count in the stoichiometry gate correct and breaks the ice rules on half the
/// bonds, because the hydrogens then point at empty space.
///
/// # The proton ordering, and a correction to the brief this was built from
///
/// The ordering placed here is FERROELECTRIC: every molecule's dipole carries a positive
/// `c` component and the cell's summed O→H vector is `(0, 0, +)`, not zero. That is
/// deliberate and it is what ice XI is — the ordered phase of Ih is the polar `Cmc2₁`
/// structure of Line & Whitworth, with the polarisation along `c`. The commissioning
/// note for this module said "antiferroelectric", and the word is wrong for XI; it is
/// right for VIII, which is why `ice_viii` below really is antiferroelectric. The
/// correction is recorded here rather than implemented as written.
///
/// What is NOT claimed: the `Cmc2₁` SPACE GROUP. This builds a specific ice-rule-obeying
/// ferroelectric ordering of the right character; the space-group symmetry of the result
/// has not been computed and is not asserted.
///
/// # The ice rule, and why the ordering is forced once one choice is made
///
/// Each of the four bonds at an oxygen must carry exactly one hydrogen, and each oxygen
/// must donate exactly two. Kind A spends one donation on its `c`-axis bond, so it has
/// one left for its three lateral bonds; kind B's `c`-axis bond is already fed by the A
/// below it, so B must donate on two of its three laterals. The laterals form a
/// honeycomb between the A and B sublattices, so the assignment is exactly a PERFECT
/// MATCHING of that honeycomb: the matched bond is donated by A, the unmatched two by B.
/// The matching chosen is the single-direction dimer covering "always `d3`", which is a
/// perfect matching because every A has one `d3` bond and every B has one `−d3` bond.
pub fn ice_xi(cells: (usize, usize, usize), s: f64) -> IceScene {
    let a = A_ICE_XI_BOHR * s;
    let c = C_ICE_XI_BOHR * s;
    let b = a * 3.0f64.sqrt();
    let r = R_OH_BOHR * s;
    let bx = (
        cells.0 as f64 * a,
        cells.1 as f64 * b,
        cells.2 as f64 * c,
    );

    // The three lateral in-plane offsets, from a column-P site to its column-Q partners.
    let d1 = (0.5 * a, -a * 3.0f64.sqrt() / 6.0, 0.0);
    let d2 = (-0.5 * a, -a * 3.0f64.sqrt() / 6.0, 0.0);
    let d3 = (0.0, a / 3.0f64.sqrt(), 0.0);
    let dz = c / 8.0; // the lateral bond's rise
    let dv = 3.0 * c / 8.0; // the c-axis bond's length

    // (fx, fy, fz, kind-A?, column sign): 8 oxygens, the orthorhombic cell of ice Ih.
    // Column P (sign +1) is the hexagonal (1/3, 2/3) column, Q (sign −1) is (2/3, 1/3).
    let sites: [(f64, f64, f64, bool, f64); 8] = [
        (0.0, 1.0 / 3.0, 0.0, true, 1.0),
        (0.0, 1.0 / 3.0, 3.0 / 8.0, false, 1.0),
        (0.5, 1.0 / 6.0, 0.5, true, -1.0),
        (0.5, 1.0 / 6.0, 7.0 / 8.0, false, -1.0),
        (0.5, 5.0 / 6.0, 0.0, true, 1.0),
        (0.5, 5.0 / 6.0, 3.0 / 8.0, false, 1.0),
        (0.0, 2.0 / 3.0, 0.5, true, -1.0),
        (0.0, 2.0 / 3.0, 7.0 / 8.0, false, -1.0),
    ];

    let mut out: Vec<SpecAtom> = Vec::with_capacity(24 * cells.0 * cells.1 * cells.2);
    for ix in 0..cells.0 {
        for iy in 0..cells.1 {
            for iz in 0..cells.2 {
                let off = (ix as f64 * a, iy as f64 * b, iz as f64 * c);
                for &(fx, fy, fz, kind_a, sg) in sites.iter() {
                    let o = (off.0 + fx * a, off.1 + fy * b, off.2 + fz * c);
                    out.push((Z_O, o.0, o.1, o.2));
                    if kind_a {
                        // donate up the c-axis bond, and on the single MATCHED lateral
                        h_along(&mut out, o, (0.0, 0.0, dv), r);
                        h_along(&mut out, o, (sg * d3.0, sg * d3.1, -dz), r);
                    } else {
                        // donate on the two UNMATCHED laterals
                        h_along(&mut out, o, (sg * d1.0, sg * d1.1, dz), r);
                        h_along(&mut out, o, (sg * d2.0, sg * d2.1, dz), r);
                    }
                }
            }
        }
    }
    (wrap_all(out, bx), bx)
}

// -------------------------------------------------- the shared bcc double cell

/// Same-network bond directions from a CORNER site of the bcc arrangement: the four
/// `(±1, ±1, ±1)` octants with an even number of minus signs.
const T_PLUS: [(f64, f64, f64); 4] = [
    (1.0, 1.0, 1.0),
    (1.0, -1.0, -1.0),
    (-1.0, 1.0, -1.0),
    (-1.0, -1.0, 1.0),
];
/// Same-network bond directions from a BODY-CENTRE site: the complementary tetrahedron.
const T_MINUS: [(f64, f64, f64); 4] = [
    (-1.0, -1.0, -1.0),
    (-1.0, 1.0, 1.0),
    (1.0, -1.0, 1.0),
    (1.0, 1.0, -1.0),
];

/// The 16 oxygens of the cubic cell of edge `a`, as `(position, is_corner, network)`.
///
/// The oxygen arrangement of the VII/VIII/X family is bcc with cube edge `a/2`, and it
/// carries TWO interpenetrating diamond networks. A bcc site has eight neighbours at
/// `a√3/4`; four of them are in its own network (hydrogen-bonded, a tetrahedron) and
/// four are in the other (not bonded, and at the SAME distance — that degeneracy is a
/// property of the real crystal, not an artifact of this construction).
///
/// Membership is the parity of the bcc index sum: writing a corner site as `(m,n,p)·b`
/// and a body-centre site as `(m+½,n+½,p+½)·b` with `b = a/2`, both belong to network
/// `(m+n+p) mod 2`. Corner sites reach their own network through `T_PLUS`, body-centre
/// sites through `T_MINUS`, and the two rules are each other's inverse, so a bond found
/// from one end is the same bond found from the other. Because the parity has period two
/// in `b`, the smallest cell that carries the two-network structure is `2×2×2` bcc cells
/// — the 16-oxygen cube this returns, which is why `a` is documented as the edge of THAT
/// cell rather than of the bcc one.
fn bcc_double_cell(a: f64) -> [((f64, f64, f64), bool, u8); 16] {
    let b = 0.5 * a;
    let mut out = [((0.0, 0.0, 0.0), false, 0u8); 16];
    let mut k = 0usize;
    let mut m = 0usize;
    while m < 2 {
        let mut n = 0usize;
        while n < 2 {
            let mut p = 0usize;
            while p < 2 {
                let net = ((m + n + p) % 2) as u8;
                out[k] = ((m as f64 * b, n as f64 * b, p as f64 * b), true, net);
                out[k + 1] = (
                    (
                        (m as f64 + 0.5) * b,
                        (n as f64 + 0.5) * b,
                        (p as f64 + 0.5) * b,
                    ),
                    false,
                    net,
                );
                k += 2;
                p += 1;
            }
            n += 1;
        }
        m += 1;
    }
    out
}

// ---------------------------------------------------------------- ice VIII (VII family)

/// ICE VIII: the ORDERED representative of the VII family — two interpenetrating
/// cubic-ice sublattices on a bcc oxygen arrangement, with ANTIFERROELECTRIC proton
/// ordering.
///
/// # Geometry
///
/// `A_ICE_VIII_BOHR` is the edge of the 16-oxygen cubic cell; the bcc cube edge is half
/// of it and the O–O nearest-neighbour separation is `a√3/4` = 5.2048 bohr (2.754 Å) at
/// `s = 1`. Every oxygen has eight neighbours at exactly that distance, four of them
/// hydrogen-bonded (its own network) and four not (the other network). Two covalent
/// hydrogens per oxygen at `R_OH_BOHR·s`, each on one of the four bonded axes.
///
/// # The ordering
///
/// Within one network every molecule points the same way; the two networks point
/// opposite ways, so the summed O→H vector over the cell is EXACTLY zero. That is what
/// makes VIII the antiferroelectric member of the family and VII the disordered one.
///
/// Concretely: a network-0 oxygen donates along the two of its four bonded axes whose
/// `z` component is positive, a network-1 oxygen along the two whose `z` component is
/// negative. Both tetrahedra `T_PLUS` and `T_MINUS` have exactly two directions of each
/// `z` sign, so the rule always selects two. The ice rule follows without a search: the
/// far end of a donated axis sees the reversed direction, whose `z` sign is opposite, so
/// it never donates on that axis — one hydrogen per bond, two donated and two accepted
/// per oxygen.
pub fn ice_viii(cells: (usize, usize, usize), s: f64) -> IceScene {
    let a = A_ICE_VIII_BOHR * s;
    let r = R_OH_BOHR * s;
    let bx = (
        cells.0 as f64 * a,
        cells.1 as f64 * a,
        cells.2 as f64 * a,
    );
    let sites = bcc_double_cell(a);

    let mut out: Vec<SpecAtom> = Vec::with_capacity(48 * cells.0 * cells.1 * cells.2);
    for ix in 0..cells.0 {
        for iy in 0..cells.1 {
            for iz in 0..cells.2 {
                let off = (ix as f64 * a, iy as f64 * a, iz as f64 * a);
                for &(p, corner, net) in sites.iter() {
                    let o = (off.0 + p.0, off.1 + p.1, off.2 + p.2);
                    out.push((Z_O, o.0, o.1, o.2));
                    let dirs = if corner { &T_PLUS } else { &T_MINUS };
                    // network 0 points +z, network 1 points −z: the antiferroelectric pair.
                    let want = if net == 0 { 1.0 } else { -1.0 };
                    for &d in dirs.iter() {
                        if d.2 == want {
                            h_along(&mut out, o, d, r);
                        }
                    }
                }
            }
        }
    }
    (wrap_all(out, bx), bx)
}

// ---------------------------------------------------------------- ice X (symmetric)

/// ICE X: SYMMETRIC ice. Every hydrogen sits at the EXACT MIDPOINT of its O–O axis, so
/// there is no covalent bond and no hydrogen bond — only one kind of O–H contact. That
/// is the polymorph's defining property and the reason this builder ignores
/// `R_OH_BOHR` entirely.
///
/// The oxygen arrangement is the same bcc double cell as `ice_viii`, at
/// `A_ICE_X_BOHR` = 10.0 bohr (5.29 Å), giving an O–O nearest-neighbour separation of
/// `a√3/4` = 4.3301 bohr (2.292 Å) and an O–H of half that, 2.1651 bohr. Those are
/// REPRESENTATIVE of the symmetrisation regime rather than a measurement at a stated
/// pressure: symmetric ice exists above roughly 60 GPa and its lattice constant runs
/// with pressure, so the number is a declared choice inside the right range and not a
/// datum.
///
/// # The axis-to-hydrogen assignment, and why the count comes out at exactly two
///
/// Hydrogens go on the SAME-NETWORK (diamond) axes only, never on the four
/// other-network neighbours at the same distance — putting one on all eight would give
/// four hydrogens per oxygen and the wrong compound.
///
/// The rule is: **the corner-type oxygen of a bonded pair emits the hydrogen.** Every
/// same-network axis joins one corner site to one body-centre site (a bcc site's
/// neighbours are all of the other kind), so every axis has exactly one corner end and
/// is emitted exactly once — no double-counting, no missed bond, and no need to compare
/// coordinates or sort. Each corner oxygen emits 4, body-centre oxygens emit 0, and
/// corner and body-centre sites are equinumerous, so the scene carries
/// `4·N/2 = 2·N` hydrogens for `N` oxygens: exactly 2 H per O.
///
/// The per-oxygen count in the OTHER sense — how many hydrogens are nearest to a given
/// oxygen — is FOUR here, at 2.1651 bohr each, and that is correct rather than a defect:
/// in symmetric ice each hydrogen is shared between two oxygens. The stoichiometric "2
/// H per O" is a statement about the scene's totals, and the geometry gate for this
/// builder has to be the midpoint condition, not a covalent-shell count.
pub fn ice_x(cells: (usize, usize, usize), s: f64) -> IceScene {
    let a = A_ICE_X_BOHR * s;
    let b = 0.5 * a; // the bcc cube edge
    let bx = (
        cells.0 as f64 * a,
        cells.1 as f64 * a,
        cells.2 as f64 * a,
    );
    let sites = bcc_double_cell(a);

    let mut out: Vec<SpecAtom> = Vec::with_capacity(48 * cells.0 * cells.1 * cells.2);
    for ix in 0..cells.0 {
        for iy in 0..cells.1 {
            for iz in 0..cells.2 {
                let off = (ix as f64 * a, iy as f64 * a, iz as f64 * a);
                for &(p, corner, _net) in sites.iter() {
                    let o = (off.0 + p.0, off.1 + p.1, off.2 + p.2);
                    out.push((Z_O, o.0, o.1, o.2));
                    if corner {
                        // The neighbour lies at d·(b/2); the midpoint is d·(b/4).
                        for &d in T_PLUS.iter() {
                            out.push((
                                Z_H,
                                o.0 + d.0 * b / 4.0,
                                o.1 + d.1 * b / 4.0,
                                o.2 + d.2 * b / 4.0,
                            ));
                        }
                    }
                }
            }
        }
    }
    (wrap_all(out, bx), bx)
}
