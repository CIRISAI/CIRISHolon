//! SATURATION-1's chemistry-side gates: R1 (the 50-digit referee), T1 (interpolant
//! fidelity, held out), T2 (the truncation systematic), and plants (i) and (ii).
//!
//! One gate per question, each printing its measured margin, because a passing assertion
//! that does not say by how much is a claim without a number behind it.
//!
//! The prereg is `conformance/atomworld/SATURATION1_PREREG.md`, and AMENDMENT A1 is in
//! force: the domain is `a >= 0.9`, `b <= 9.0`, `c <= a + b` on the SORTED sides, and T2
//! is pointed at the `b = 9` shell. The original any-side-7.0 shell is kept here as a
//! test that asserts its own FIRING, because a dead branch stays in the record.

#[allow(dead_code)]
mod common;

use common::decimal_minus_f64;
use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::pair::{atom_energy, solve_geometry};
use holon_chem::trimer::{self, TrimerTable};
use std::sync::OnceLock;

/// The table is built once for the whole test binary: 7,293 electronic-structure solves
/// is not a per-test cost worth paying six times.
fn table() -> &'static TrimerTable {
    static T: OnceLock<TrimerTable> = OnceLock::new();
    T.get_or_init(|| trimer::generate().expect("the trimer table generates"))
}

fn e_h() -> f64 {
    static E: OnceLock<f64> = OnceLock::new();
    *E.get_or_init(trimer::atom_energy)
}

/// The general N-centre route: the one `pair::solve_geometry` drives, whose H2 restriction
/// the pinned 50-digit referee grades on every build. It is also the ONLY route here that
/// handles four centres, so the H4 plant goes through it.
fn general(cs: &[[f64; 3]]) -> f64 {
    let h = by_symbol("H").expect("hydrogen");
    let species = vec![h; cs.len()];
    let d: Vec<[D2; 3]> = cs
        .iter()
        .map(|c| [D2::c(c[0]), D2::c(c[1]), D2::c(c[2])])
        .collect();
    solve_geometry(&species, d).e.v
}

fn triangle(x: f64, y: f64, z: f64) -> [[f64; 3]; 3] {
    let cos = ((x * x + y * y - z * z) / (2.0 * x * y)).clamp(-1.0, 1.0);
    let sin = (1.0 - cos * cos).max(0.0).sqrt();
    [[0.0, 0.0, 0.0], [x, 0.0, 0.0], [y * cos, y * sin, 0.0]]
}

const R_E: f64 = 1.388_694;

// ---------------------------------------------------------------- R1

/// FNV-1a (32-bit) of `conformance/atomworld/h3_referee.json`, the pinned 50-digit trimer
/// curve. Pinned by DIGEST rather than by shape, for the same reason the H2 referee is: a
/// length check or a spot-check would pass against a file whose interior had been edited,
/// and the gate would then be grading against a referee nobody had looked at.
pub const R1_REFEREE_DIGEST: u32 = 0xd5b1_07ba;

/// R1's stake, hartree. The prereg's number, looser than H2's 1e-12 because three centres
/// put more cancellation through the same closed forms.
pub const R1_STAKE_E: f64 = 1e-10;

fn referee_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../conformance/atomworld/h3_referee.json")
}

/// One referee row: the three sides, and the two quantities it grades.
struct Row {
    sides: [f64; 3],
    e_h3: String,
    de3: String,
    block: String,
}

/// The referee's `geometries` array. Its rows contain no nested objects, so splitting on
/// the closing brace is a parser rather than a guess; the field extractor below is the
/// same shape `common::string_array` uses on the flat H2 file.
fn referee_rows(src: &str) -> Vec<Row> {
    let at = src.find("\"geometries\"").expect("no geometries array");
    let body = &src[at..];
    let mut rows = Vec::new();
    for chunk in body.split('{').skip(1) {
        let obj = match chunk.find('}') {
            Some(e) => &chunk[..e],
            None => continue,
        };
        let field = |k: &str| -> Option<String> {
            let needle = format!("\"{k}\":");
            let p = obj.find(&needle)? + needle.len();
            let rest = obj[p..].trim_start();
            let rest = rest.strip_prefix('"')?;
            Some(rest[..rest.find('"')?].to_string())
        };
        let arr = |k: &str| -> Option<[f64; 3]> {
            let needle = format!("\"{k}\":");
            let p = obj.find(&needle)? + needle.len();
            let rest = obj[p..].trim_start().strip_prefix('[')?;
            let end = rest.find(']')?;
            let v: Vec<f64> = rest[..end]
                .split(',')
                .map(|t| t.trim().trim_matches('"').parse::<f64>().unwrap())
                .collect();
            Some([v[0], v[1], v[2]])
        };
        let (Some(sides), Some(e_h3), Some(de3)) =
            (arr("sides_bohr"), field("E_H3"), field("dE3"))
        else {
            continue;
        };
        rows.push(Row {
            sides,
            e_h3,
            de3,
            block: field("block").unwrap_or_default(),
        });
    }
    rows
}

/// R1 — the engine's f64 H3 against an independent 50-digit Python/mpmath referee that
/// shares no code, no language and no arithmetic with it, only the model definition.
#[test]
fn r1_the_trimer_matches_the_fifty_digit_referee() {
    let path = referee_path();
    let raw = std::fs::read(&path)
        .unwrap_or_else(|e| panic!("referee file {} is missing ({e})", path.display()));
    let digest = holon_chem::fnv1a32(&raw);
    assert_eq!(
        digest, R1_REFEREE_DIGEST,
        "the SATURATION-1 referee file has changed (digest {digest:#010x}). If that was \
         deliberate, re-derive the residual against the new file rather than re-pinning \
         the digest alone."
    );
    let src = String::from_utf8(raw).expect("utf-8");
    let rows = referee_rows(&src);
    assert!(
        rows.len() >= 64,
        "the referee set has {} geometries; R1 stakes at least 64",
        rows.len()
    );

    let mut worst_e = 0.0f64;
    let mut worst_d = 0.0f64;
    let mut at_e = 0usize;
    let mut at_d = 0usize;
    for (i, r) in rows.iter().enumerate() {
        let mine = trimer::hydrogen_energy(&triangle(r.sides[0], r.sides[1], r.sides[2]));
        let de = decimal_minus_f64(&r.e_h3, mine).abs();
        if de > worst_e {
            worst_e = de;
            at_e = i;
        }
        let mine3 = trimer::de3_sides(r.sides[0], r.sides[1], r.sides[2], e_h());
        let dd = decimal_minus_f64(&r.de3, mine3).abs();
        if dd > worst_d {
            worst_d = dd;
            at_d = i;
        }
    }
    println!(
        "R1: {n} geometries, digest {digest:#010x}\n  \
         max |E(H3) - referee|  = {worst_e:.4e} Ha  at {:?} [{}]\n  \
         max |dE3  - referee|   = {worst_d:.4e} Ha  at {:?} [{}]\n  \
         stake {R1_STAKE_E:.0e}, margin {:.0}x",
        rows[at_e].sides,
        rows[at_e].block,
        rows[at_d].sides,
        rows[at_d].block,
        R1_STAKE_E / worst_e.max(worst_d),
        n = rows.len()
    );
    assert!(
        worst_e <= R1_STAKE_E,
        "THE R1 STAKE FIRED on E(H3): {worst_e:.4e} Ha past {R1_STAKE_E:.0e} at {:?}",
        rows[at_e].sides
    );
    assert!(
        worst_d <= R1_STAKE_E,
        "THE R1 STAKE FIRED on dE3: {worst_d:.4e} Ha past {R1_STAKE_E:.0e} at {:?}",
        rows[at_d].sides
    );
}

// ---------------------------------------------------------------- the two implementations

/// THE GATE THAT PAYS FOR THE SECOND IMPLEMENTATION.
///
/// `trimer::hydrogen_energy` is a second path through one model: s-only, f64-only, and
/// ~25x faster than the general N-centre route, which is what makes a table of thousands
/// of nodes buildable at load. A second implementation is only honest if it is held to the
/// first, so it is — over a set that spans the table's whole domain, at 1e-12 hartree,
/// which is the scale the general route's own referee gate works at.
#[test]
fn the_fast_path_agrees_with_the_general_n_centre_route() {
    let mut cases: Vec<Vec<[f64; 3]>> = vec![
        vec![[0.0, 0.0, 0.0]],
        vec![[0.0, 0.0, 0.0], [0.7, 0.0, 0.0]],
        vec![[0.0, 0.0, 0.0], [R_E, 0.0, 0.0]],
        vec![[0.0, 0.0, 0.0], [9.0, 0.0, 0.0]],
        vec![[0.0, 0.0, 0.0], [18.0, 0.0, 0.0]],
    ];
    // Compact, scalene, near-linear, near-boundary — the same spread R1 stakes.
    for &(x, y, z) in &[
        (0.7f64, 0.7f64, 0.7f64),
        (0.9, 0.9, 0.9),
        (R_E, R_E, R_E),
        (R_E, R_E, 2.0 * R_E),
        (0.9, 2.0, 2.8),
        (1.2, 5.5, 6.4),
        (3.5, 3.5, 6.99),
        (9.0, 9.0, 18.0),
        (9.0, 9.0, 9.0),
        (2.31, 4.77, 6.13),
        (0.95, 8.9, 8.95),
    ] {
        cases.push(triangle(x, y, z).to_vec());
    }
    let mut worst = 0.0f64;
    let mut at = String::new();
    for c in &cases {
        let fast = trimer::hydrogen_energy(c);
        let gen = general(c);
        let d = (fast - gen).abs();
        if d > worst {
            worst = d;
            at = format!("{c:?}");
        }
    }
    println!("fast vs general N-centre route: max |dE| = {worst:.4e} hartree at {at}");
    assert!(
        worst <= 1e-12,
        "the fast s-only path disagrees with the referee'd general route by {worst:.4e} \
         hartree at {at}"
    );
    // And the pair restriction against the OTHER banked route, the one the sandbox's own
    // curve is built from.
    let mut worst_pair = 0.0f64;
    for k in 0..45 {
        let r = 0.7 + 0.4 * k as f64;
        worst_pair = worst_pair.max((trimer::pair_energy(r) - holon_chem::h2_point(r).e).abs());
    }
    println!("fast pair vs banked h2_point: max |dE| = {worst_pair:.4e} hartree");
    assert!(worst_pair <= 1e-12, "pair paths disagree by {worst_pair:.4e}");
}

/// The three numbers the prereg DISCLOSED before the freeze, recomputed by the path that
/// actually builds the table. They are priors, not results; this checks the new path
/// reproduces them rather than quietly computing a different function.
///
/// The far-field line carries AMENDMENT A1's precision fence: "dE3 vanishes far away" is
/// an f64 statement only. The referee's T13 shows the true equilateral tail is spin
/// frustration, `dE3 -> 3J/2`, `+4.4e-29 Ha` at 20 bohr — twelve decades below anything
/// f64 can carry through this cancellation. The assertion is therefore an f64 FLOOR, never
/// a literal zero, and the arithmetic-closure check sits at 40 bohr.
#[test]
fn the_disclosed_probe_priors_are_reproduced() {
    let eq = trimer::de3_sides(R_E, R_E, R_E, e_h());
    let lin = trimer::de3_sides(R_E, R_E, 2.0 * R_E, e_h());
    let app = trimer::de3_sides(R_E, 2.0, R_E + 2.0, e_h());
    let far20 = trimer::de3_sides(R_E, 20.0, R_E + 20.0, e_h());
    let far40 = trimer::de3_sides(R_E, 40.0, R_E + 40.0, e_h());
    println!(
        "priors: equilateral {eq:+.6}  linear {lin:+.6}  H2+H@2 {app:+.6}\n  \
         far field: {far20:+.3e} at 20 bohr, {far40:+.3e} at 40 bohr (f64 floor, not zero: \
         the true tail is 3J/2 ~ 4.4e-29 Ha)"
    );
    assert!((eq - 0.858_071).abs() < 5e-6, "equilateral prior moved: {eq}");
    assert!((lin - 0.354_728).abs() < 5e-6, "linear prior moved: {lin}");
    assert!((app - 0.216_860).abs() < 5e-6, "approach prior moved: {app}");
    assert!(
        far20.abs() < 1e-12,
        "dE3 did not reach the f64 floor when the third atom left: {far20:.3e}"
    );
    assert!(
        far40.abs() < 1e-12,
        "the arithmetic did not close at 40 bohr: {far40:.3e}"
    );
}

// ---------------------------------------------------------------- T1

/// THE STAKED DRAW. Frozen in the source, so the 256 geometries are the same 256 on every
/// machine and every run: no RNG state escapes this constant.
const T1_SEED: u64 = 0x5341_5455_5241_5431; // "SATURAT1"

/// The prereg's kill for T1, hartree.
const T1_KILL: f64 = 1e-3;

fn lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

/// The 256 held-out geometries: drawn inside the domain, sorted, and spanning it.
fn held_out() -> Vec<(f64, f64, f64)> {
    let mut st = T1_SEED;
    let mut pts = Vec::with_capacity(256);
    while pts.len() < 256 {
        let x = 0.9 + (trimer::R_HI - 0.9) * lcg(&mut st);
        let y = x + (trimer::R_HI - x) * lcg(&mut st);
        // `u <= x/(2y)` is exactly the condition that keeps `z` the longest side, so the
        // draw stays inside the sorted domain the table is defined on.
        let u = -1.0 + (x / (2.0 * y) + 1.0) * lcg(&mut st);
        let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();
        pts.push((x, y, z));
    }
    pts
}

#[test]
fn t1_interpolant_fidelity_on_held_out_geometries() {
    let t = table();
    let pts = held_out();

    // The prereg's VOID condition, checked before the gate rather than after: a draw that
    // landed on grid nodes would test nothing.
    for &(x, y, _) in &pts {
        for r in [x, y] {
            let idx = trimer::tau_of_r(r) * (trimer::NR - 1) as f64;
            assert!(
                (idx - idx.round()).abs() > 1e-9,
                "a drawn side landed on a grid node ({r}); the draw must be redrawn (VOID)"
            );
        }
    }

    let mut worst = 0.0f64;
    let mut at = (0.0, 0.0, 0.0);
    let mut sum2 = 0.0;
    let mut zeros = 0usize;
    for &(x, y, z) in &pts {
        let exact = trimer::de3_sides(x, y, z, e_h());
        let (got, _) = t.eval([x, y, z]);
        let e = (got - exact).abs();
        if e == 0.0 {
            zeros += 1;
        }
        sum2 += e * e;
        if e > worst {
            worst = e;
            at = (x, y, z);
        }
    }
    println!(
        "T1: 256 held-out geometries, seed {T1_SEED:#018x}\n  \
         max |interpolant - direct FCI| = {worst:.4e} Ha  (kill at {T1_KILL:.0e}, margin \
         {:.1}x)\n  rms = {:.4e} Ha   worst at (s1,s2,s3) = ({:.3}, {:.3}, {:.3})\n  \
         grid {}x{}x{} on [{}, {}] bohr, {} nodes, {} solves, {} exact-zero errors",
        T1_KILL / worst,
        (sum2 / pts.len() as f64).sqrt(),
        at.0,
        at.1,
        at.2,
        trimer::NR,
        trimer::NR,
        trimer::NU,
        trimer::R_LO,
        trimer::R_HI,
        trimer::N_NODES,
        t.meta.solves,
        zeros
    );
    // Two-sided, exactly as staked: an exact zero everywhere would mean the draw hit
    // nodes and the check tested nothing.
    assert!(
        worst > 0.0,
        "every held-out error was exactly zero: the draw tested nothing (VOID, redraw)"
    );
    assert!(
        worst <= T1_KILL,
        "THE T1 KILL FIRED: held-out interpolation error {worst:.4e} Ha exceeds \
         {T1_KILL:.0e} at (s1,s2,s3) = {at:?}"
    );
}

// ---------------------------------------------------------------- T2

/// The prereg's kill for T2, hartree — unchanged by AMENDMENT A1.
const T2_KILL: f64 = 1e-5;

/// THE ORIGINALLY STAKED SHELL, and it FIRES. Kept in the record, marked, per the
/// discipline: a prereg's dead branch is reported as plainly as its live one, and
/// AMENDMENT A1 is the pre-committed response ("the domain must grow") executing.
///
/// The freeze staked the truncation on "any side at 7.0 bohr". That shell is not where the
/// tail lives. `dE3` vanishes only when one atom is far from BOTH others, which for sorted
/// sides is a statement about `b`, not `c` — a near-collinear chain's longest side is the
/// SUM of two short ones and is not a distance anything decays over. This test asserts the
/// FIRING, so a later change that quietly made it pass would fail here and be looked at.
#[test]
fn t2_the_originally_staked_longest_side_shell_fires() {
    let mut worst = 0.0f64;
    let mut at = (0.0, 0.0);
    for i in 0..=60 {
        for j in 0..=60 {
            let a = 0.9 + (7.0 - 0.9) * i as f64 / 60.0;
            let b = 0.9 + (7.0 - 0.9) * j as f64 / 60.0;
            if a > b || a + b < 7.0 {
                continue;
            }
            let d = trimer::de3_sides(a, b, 7.0, e_h()).abs();
            if d > worst {
                worst = d;
                at = (a, b);
            }
        }
    }
    println!(
        "T2 (as originally staked, longest side = 7.0 bohr): max |dE3| = {worst:.4e} Ha \
         at (a,b) = ({:.2}, {:.2}) — kill at {T2_KILL:.0e}, so it FIRES by {:.0}x; \
         AMENDMENT A1 re-points the shell",
        at.0,
        at.1,
        worst / T2_KILL
    );
    assert!(
        worst > T2_KILL,
        "the originally staked shell now reads {worst:.4e}, inside the {T2_KILL:.0e} kill. \
         That would be a different surface from the one AMENDMENT A1 was written against; \
         re-derive the domain rather than deleting this test."
    );
}

/// T2 AS AMENDED: the truncation systematic of the domain that ships, on the `b = R_cut`
/// shell, with the collinear geometries named as the worst case.
#[test]
fn t2_the_shipped_truncation_systematic() {
    let mut worst = 0.0f64;
    let mut at = (0.0, 0.0, 0.0);
    let b = trimer::R_HI;
    for i in 0..=90 {
        let a = 0.9 + (b - 0.9) * i as f64 / 90.0;
        for k in 0..=120 {
            let u = -1.0 + 2.0 * k as f64 / 120.0;
            let c = (a * a + b * b - 2.0 * a * b * u).max(0.0).sqrt();
            // `b` has to really be the MIDDLE side for this to be the shell in question.
            if c < b {
                continue;
            }
            let d = trimer::de3_sides(a, b, c, e_h()).abs();
            if d > worst {
                worst = d;
                at = (a, b, c);
            }
        }
    }
    // The named worst-case instrument: the collinear chain, where the referee found the
    // b-shell's maximum.
    let collinear = trimer::de3_sides(b, b, 2.0 * b, e_h()).abs();
    println!(
        "T2 (as amended, middle side = {b} bohr): max |dE3| = {worst:.4e} Ha \
         (kill {T2_KILL:.0e}, margin {:.0}x) at (a,b,c) = ({:.2},{:.2},{:.2});\n  \
         collinear probe (b,b,2b) = {collinear:.4e} Ha",
        T2_KILL / worst,
        at.0,
        at.1,
        at.2
    );
    assert!(
        worst <= T2_KILL,
        "THE T2 KILL FIRED on the amended domain: {worst:.4e} Ha at {at:?} exceeds \
         {T2_KILL:.0e}; the domain must grow again"
    );
}

// ---------------------------------------------------------------- plant (i)

/// PLANT (i): the sign-flip plant.
///
/// Negating the tabulated `dE3` must invert the two-dimers-vs-tetrahedron comparison. The
/// carrier is the gap itself, asserted nonzero before the plant is scored — and it is
/// AMENDMENT A1's corrected carrier: at the TRUE `r_e` edge the exact gap is +1.16259 Ha,
/// not the +0.426 the feasibility probe reported for a geometry it had mislabelled.
///
/// The H4 energies go through the general N-centre route, which is the only one here that
/// handles four centres, in the `Sz = 0` block — A1's pinned convention, and the block
/// whose minimum is the molecule's true ground energy.
#[test]
fn plant_i_the_sign_flip_inverts_saturation() {
    let e_h = atom_energy(by_symbol("H").expect("hydrogen"));
    // A regular tetrahedron of edge r_e.
    let a = R_E;
    let tet = [
        [0.0, 0.0, 0.0],
        [a, 0.0, 0.0],
        [0.5 * a, a * 0.866_025_403_784_438_6, 0.0],
        [
            0.5 * a,
            a * 0.288_675_134_594_812_9,
            a * 0.816_496_580_927_726,
        ],
    ];
    // The exact carrier: E(H4 tetrahedron) against two separated dimers.
    let e_tet = general(&tet);
    let e_2h2 = 2.0 * general(&[[0.0, 0.0, 0.0], [R_E, 0.0, 0.0]]);
    let carrier = e_tet - e_2h2;
    println!(
        "plant (i) carrier: E(H4 tet, edge r_e) = {e_tet:+.9}, 2 x E(H2) = {e_2h2:+.9}, \
         two dimers win by {carrier:+.6} Ha"
    );
    assert!(
        carrier.abs() > 1e-3,
        "the carrier is empty: the plant would be scored on nothing"
    );
    assert!(
        (carrier - 1.162_59).abs() < 1e-4,
        "the corrected carrier moved: {carrier:+.6} vs AMENDMENT A1's +1.16259"
    );

    // The same comparison as the MBE3 sandbox sees it: pairs from the curve, triples from
    // the table. This is the quantity the dynamics actually integrates.
    let d = |i: usize, j: usize| -> f64 {
        let (p, q) = (tet[i], tet[j]);
        ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
    };
    let mbe3_gap = |t: &TrimerTable| -> f64 {
        let mut e = 0.0;
        for i in 0..4 {
            for j in (i + 1)..4 {
                e += trimer::pair_energy(d(i, j)) - 2.0 * e_h;
            }
        }
        for i in 0..4 {
            for j in (i + 1)..4 {
                for k in (j + 1)..4 {
                    e += t.eval([d(i, j), d(i, k), d(j, k)]).0;
                }
            }
        }
        // Two separated dimers, in the same asymptote-zeroed convention.
        e - 2.0 * (trimer::pair_energy(R_E) - 2.0 * e_h)
    };
    let straight = mbe3_gap(table());
    let mut flipped_table = table().clone();
    flipped_table.negate();
    let flipped = mbe3_gap(&flipped_table);
    println!(
        "plant (i): MBE3 reads the gap as {straight:+.6} Ha; with the table NEGATED it \
         reads {flipped:+.6} Ha"
    );
    assert!(
        straight > 0.0,
        "MBE3 does not see saturation at all: gap {straight:+.6}"
    );
    assert!(
        flipped < 0.0,
        "negating the three-body table did not invert the comparison ({flipped:+.6}): the \
         gate cannot see the sign of the term it credits"
    );
}

// ---------------------------------------------------------------- plant (ii)

/// PLANT (ii): the symmetry plant.
///
/// `dE3` is totally symmetric in its three sides. Evaluating the table at a staked scalene
/// geometry under all six permutations must agree EXACTLY — not to a tolerance, exactly,
/// because the evaluation sorts first and floating-point comparison is exact. The carrier
/// is asserted nonzero before the plant is scored, and the plant itself is a deliberately
/// desymmetrised table, which must disagree by at least 1e-6 hartree.
#[test]
fn plant_ii_symmetry_and_its_deliberate_break() {
    let t = table();
    let sides = [1.237, 2.041, 2.713];
    let (v0, g0) = t.eval(sides);
    println!("plant (ii) carrier: dE3 at the staked scalene geometry = {v0:+.9e} Ha");
    assert!(
        v0.abs() > 1e-4,
        "the carrier is empty: the plant would be scored on nothing"
    );

    let perms = [
        [0usize, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for p in perms {
        let (v, g) = t.eval([sides[p[0]], sides[p[1]], sides[p[2]]]);
        assert_eq!(
            v.to_bits(),
            v0.to_bits(),
            "permutation {p:?} moved the value: {v} vs {v0}"
        );
        for a in 0..3 {
            assert_eq!(
                g[a].to_bits(),
                g0[p[a]].to_bits(),
                "permutation {p:?} mis-routed gradient component {a}"
            );
        }
    }
    println!("plant (ii): all six permutations agree bit-for-bit, value and gradient");

    // The plant: break the table's x <-> y symmetry on purpose at ONE node, and confirm
    // the reading moves. The probe sits off the diagonal so the stencil reaches the
    // mutated node from one side only.
    let (i, j, k) = (6usize, 9usize, 5usize);
    let mut mutated = t.clone();
    let before = mutated.node(i, j, k);
    mutated.set_node(i, j, k, before + 1e-3);
    let x = trimer::node_r(i) + 0.03;
    let y = trimer::node_r(j) + 0.03;
    let c = trimer::node_c(k);
    let u = 1.0 - c * c;
    let z = (x * x + y * y - 2.0 * x * y * u).sqrt();
    let broke = (mutated.eval([x, y, z]).0 - t.eval([x, y, z]).0).abs();
    // And the mutated table is no longer symmetric under the swap that the sorted lookup
    // makes invisible: read the mirrored node directly.
    let asym = (mutated.node(i, j, k) - mutated.node(j, i, k)).abs();
    println!(
        "plant (ii): the desymmetrised table moved the reading by {broke:.3e} Ha; its node \
         asymmetry is {asym:.3e} Ha"
    );
    assert!(
        asym > 0.0,
        "the mutation did not desymmetrise anything: empty sector, VOID"
    );
    assert!(
        broke >= 1e-6,
        "the deliberate desymmetrisation was invisible ({broke:.3e} Ha): the check is not \
         reading the table"
    );
}

// ---------------------------------------------------------------- the interpolant itself

/// The forces the dynamics uses come from differentiating the interpolant, so the
/// interpolant's analytic gradient must BE its own gradient. If it is not, the ledger
/// would be measuring an inconsistency rather than an integration error — the same
/// precondition the pair table's `force_is_exactly_minus_the_gradient` states.
#[test]
fn the_analytic_gradient_is_the_interpolants_own_gradient() {
    let t = table();
    let h = 1e-6;
    let mut worst = 0.0f64;
    let mut at = [0.0f64; 3];
    let cases = [
        [1.2f64, 1.3, 1.4],
        [0.95, 2.0, 2.6],
        [R_E, R_E, 2.4],
        [2.5, 3.0, 5.0],
        [1.1, 6.5, 7.2],
        [4.0, 4.5, 6.0],
        [3.0, 3.1, 6.05],
        [1.05, 8.5, 8.6],
    ];
    for c in cases {
        let (_, g) = t.eval(c);
        for a in 0..3 {
            let mut lo = c;
            let mut hi = c;
            lo[a] -= h;
            hi[a] += h;
            let numeric = (t.eval(hi).0 - t.eval(lo).0) / (2.0 * h);
            let e = (numeric - g[a]).abs() / (g[a].abs() + 1e-4);
            if e > worst {
                worst = e;
                at = c;
            }
        }
    }
    println!(
        "interpolant: worst |numeric - analytic| / (|analytic| + 1e-4) = {worst:.3e} at {at:?}"
    );
    assert!(
        worst < 1e-5,
        "the analytic gradient is not the interpolant's gradient: {worst:.3e} at {at:?}"
    );
}

/// C1 across a node: value and slope both continuous where two cells meet. Cubic Hermite
/// with a fixed node-slope functional is C1 by construction; this measures it rather than
/// asserting the construction.
#[test]
fn the_interpolant_is_c1_across_its_nodes() {
    let t = table();
    let eps = 1e-7;
    let mut worst_v = 0.0f64;
    let mut worst_g = 0.0f64;
    for i in [4usize, 9, 17, 25] {
        for k in [2usize, 6, 10] {
            let x = trimer::node_r(i);
            let y = trimer::node_r(i + 3) + 0.017;
            let c = trimer::node_c(k);
            let u = 1.0 - c * c;
            let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();
            let (vl, gl) = t.eval([x - eps, y, z]);
            let (vr, gr) = t.eval([x + eps, y, z]);
            worst_v = worst_v.max((vr - vl).abs() - eps * (gl[0].abs() + gr[0].abs()));
            worst_g = worst_g.max((gr[0] - gl[0]).abs());
        }
    }
    println!("C1 across nodes: excess value gap {worst_v:.3e} Ha, slope gap {worst_g:.3e} Ha/bohr");
    assert!(worst_v < 1e-12, "value jumps at a node: {worst_v:.3e}");
    assert!(worst_g < 1e-6, "slope jumps at a node: {worst_g:.3e}");
}

/// The table's own truncation, as the dynamics meets it: outside the domain the reading is
/// an EXACT zero — value and gradient — so a triple that leaves the domain costs nothing
/// and contributes nothing, and the step it leaves behind is the T2 systematic and no more.
#[test]
fn outside_the_domain_the_reading_is_exactly_zero() {
    let t = table();
    for sides in [
        [1.4f64, 9.5, 10.0],
        [9.5, 9.6, 12.0],
        [0.9, 20.0, 20.5],
    ] {
        let (v, g) = t.eval(sides);
        assert_eq!(v, 0.0, "nonzero value outside the domain at {sides:?}");
        assert_eq!(g, [0.0; 3], "nonzero gradient outside the domain at {sides:?}");
    }
    // And just inside it, the reading is the truncation systematic — small, but not zero.
    let (v, _) = t.eval([1.4, trimer::R_HI - 0.01, trimer::R_HI + 1.0]);
    println!("just inside the domain wall: dE3 = {v:.4e} Ha");
    assert!(v != 0.0, "the table reads zero where it should still be reading");
}
