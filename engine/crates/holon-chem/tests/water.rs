//! GATES T1, T2, G1, G2 and the three plants, for the (O, H, H) three-body table.
//!
//! # What each gate is for
//!
//! * **T1** — the interpolant is faithful where it is READ, not only where it was built:
//!   256 held-out geometries from a staked seed, none of them on a node, max error
//!   reported and killed above 1e-3 Ha. Two-sided: an error of exactly zero would mean
//!   the draw landed on nodes and the gate was measuring nothing.
//! * **T2** — the truncation is as small as the domain says. Zeroing the surface outside
//!   `max(O-H) = R_HI` costs at most the worst `|dE3|` on that shell, and the prereg
//!   stakes 1e-5 Ha.
//! * **G1** — THE EMERGENT GEOMETRY. Minimising pairs-plus-`dE3` over the three-parameter
//!   space has to land on the model's OWN full-FCI optimum, which
//!   `examples/s2_design.rs` located first and independently. Nature's 104.5 degrees and
//!   0.957 angstrom are labelled context and nothing is scored against them.
//! * **G2** — VALENCE SATURATION, two-sided. The third hydrogen must refuse by a factor
//!   the prereg stakes at 5x, AND the second hydrogen must bind deeply, which is water
//!   existing at all.
//! * **the plants** — each demonstrates its own firing before anything is trusted, and
//!   each asserts its CARRIER nonzero in the sector it acts on first (M-PLANT-SECTOR): a
//!   plant on an empty sector proves nothing and VOIDs.
//!
//! # Why the held-out count moves between profiles
//!
//! One (O, H, H) geometry is 441 determinants and about 50 ms in release. The prereg's
//! 256 held-out points are therefore about fifteen seconds of release arithmetic and
//! minutes of debug arithmetic, and the house rule is that the suite is green in BOTH.
//! So the full staked draw runs in release and a prefix of the SAME draw runs in debug —
//! the same seed, the same points, the same kill, fewer of them. The debug run is a
//! weaker instrument and says so rather than being silently skipped.

// The shared referee helpers serve several gates; this one does not need all of them.
#[allow(dead_code)]
mod common;

use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point};
use holon_chem::water::{
    self, de3_with, hh_side, node_c, node_index, node_r, WaterTable, C_HI, C_LO, NR, NU, N_NODES,
    R_HI, R_LO,
};
use holon_chem::{
    WATER_G2_STAKED_RATIO, WATER_T1_KILL_E, WATER_T1_MEASURED_E, WATER_T2_KILL_E,
    WATER_T2_MEASURED_E,
};

const PI: f64 = std::f64::consts::PI;

/// The staked draw for T1's held-out set. "SATURAT2" in ASCII, the same seed the grid
/// sizing used, so the gate and the design measurement are drawing from one stream.
const T1_SEED: u64 = 0x5341_5455_5241_5432;
const T1_POINTS: usize = 256;
/// The debug prefix. See the module header.
const T1_POINTS_DEBUG: usize = 32;

fn lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

fn table_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/s2/s2_water_table.txt")
}

fn table() -> WaterTable {
    let src = std::fs::read_to_string(table_path()).expect("the committed (O,H,H) table is present");
    water::from_text(&src).expect(
        "the committed table parses and matches this build's grid rule; if the grid \
         constants moved, the table has to be regenerated, not re-read",
    )
}

/// The held-out draw: uniform in the grid's own coordinates, rejecting anything within a
/// twentieth of a cell of a node, so "none on nodes" is enforced rather than hoped for.
fn held_out(n: usize) -> Vec<(f64, f64, f64)> {
    let mut st = T1_SEED;
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        let (t1, t2, t3) = (lcg(&mut st), lcg(&mut st), lcg(&mut st));
        let near = |t: f64, m: usize| {
            let f = t * (m - 1) as f64;
            (f - f.round()).abs() < 0.05
        };
        if near(t1, NR) || near(t2, NR) || near(t3, NU) {
            continue;
        }
        let (x, y) = (water::r_of_tau(t1), water::r_of_tau(t2));
        let (x, y) = if x <= y { (x, y) } else { (y, x) };
        out.push((x, y, C_LO + (C_HI - C_LO) * t3));
    }
    out
}


// ============================================================ the pair curves, once

/// A pair curve sampled ONCE and read by cubic Hermite interpolation.
///
/// # Why the gates below do not call `pair_point` in their loops
///
/// G1 minimises over a two-parameter space and G2 scans a spherical shell, so both would
/// otherwise pay one O-H solve and two H-H solves per trial point — thousands of full CI
/// solves for a quantity whose PAIR half is MIXTURES-1's gate rather than this campaign's.
///
/// The interpolation is cubic HERMITE and not linear, because `pair_point` hands back the
/// exact first derivative alongside the value and throwing it away would put a
/// `h^2`-sized kink in a surface whose minimum G1 is trying to locate: a linear read at
/// 0.02 bohr spacing carries a 5e-3 Ha/bohr slope error, which displaces the optimum by
/// 5e-3 bohr all on its own — a tenth of G1's whole tolerance, contributed by the
/// instrument. With the derivatives kept the error is `O(h^4)` and about 4e-7 Ha.
struct PairCurve {
    lo: f64,
    hi: f64,
    e: Vec<f64>,
    d: Vec<f64>,
}

impl PairCurve {
    fn sample(a: holon_chem::elements::Species, b: holon_chem::elements::Species) -> Self {
        let n = if cfg!(debug_assertions) { 64 } else { 192 };
        let (lo, hi) = (0.7f64, 9.5f64);
        let mut e = Vec::with_capacity(n);
        let mut d = Vec::with_capacity(n);
        for i in 0..n {
            let p = pair_point(a, b, lo + (hi - lo) * i as f64 / (n - 1) as f64);
            e.push(p.e);
            // `f` is the FORCE, -dE/dr.
            d.push(-p.f);
        }
        Self { lo, hi, e, d }
    }

    fn at(&self, r: f64) -> f64 {
        let n = self.e.len();
        let h = (self.hi - self.lo) / (n - 1) as f64;
        // Outside the sample the curve is extended linearly from the nearest end, which is
        // exact enough at the far end (flat) and is never reached at the near end: every
        // gate below fences its own geometries above `R_LO`.
        if r <= self.lo {
            return self.e[0] + (r - self.lo) * self.d[0];
        }
        if r >= self.hi {
            return self.e[n - 1] + (r - self.hi) * self.d[n - 1];
        }
        let t = (r - self.lo) / h;
        let i = (t.floor() as usize).min(n - 2);
        let s = t - i as f64;
        let (s2, s3) = (s * s, s * s * s);
        let (h00, h10, h01, h11) = (
            2.0 * s3 - 3.0 * s2 + 1.0,
            s3 - 2.0 * s2 + s,
            -2.0 * s3 + 3.0 * s2,
            s3 - s2,
        );
        h00 * self.e[i] + h10 * h * self.d[i] + h01 * self.e[i + 1] + h11 * h * self.d[i + 1]
    }
}

// ============================================================ the artifact is the solver's

#[test]
fn the_committed_table_is_this_build_s_own_output() {
    // The table is generated natively and committed because one water point is a thousand
    // times an H3 point. What keeps that from turning the sandbox back into a PLAYER of
    // someone's curve is this: a staked subset of the committed nodes is recomputed
    // through the crate's own solver, TODAY, and required to be BIT-IDENTICAL. A tolerance
    // here would be measuring the tolerance.
    let t = table();
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    assert_eq!(
        t.meta.e_o_atom.to_bits(),
        e_o.to_bits(),
        "the committed table's reference E(O) is not this build's"
    );
    assert_eq!(
        t.meta.e_h_atom.to_bits(),
        e_h.to_bits(),
        "the committed table's reference E(H) is not this build's"
    );

    let n = if cfg!(debug_assertions) { 8 } else { 32 };
    let mut st = 0x5741_5445_5220_4e44u64; // "WATER ND"
    let mut checked = 0usize;
    while checked < n {
        let i = (lcg(&mut st) * NR as f64) as usize % NR;
        let j = (lcg(&mut st) * NR as f64) as usize % NR;
        let k = (lcg(&mut st) * NU as f64) as usize % NU;
        let (a, b) = if i <= j { (i, j) } else { (j, i) };
        let (x, y) = (node_r(a), node_r(b));
        let c = node_c(k);
        let mine = de3_with(
            x,
            y,
            1.0 - c * c,
            e_o,
            e_h,
            pair_point(OXYGEN, HYDROGEN, x).e,
            pair_point(OXYGEN, HYDROGEN, y).e,
        );
        assert_eq!(
            t.node(a, b, k).to_bits(),
            mine.to_bits(),
            "committed node ({a}, {b}, {k}) is {:.17e}, this build's solver says {mine:.17e}. \
             The artifact and the code have diverged; regenerate it with \
             `--example s2_build -- --emit {NR} {NU}`.",
            t.node(a, b, k)
        );
        // The mirror is not a second rounding of the same number, it is the same float.
        assert_eq!(
            t.node(a, b, k).to_bits(),
            t.node(b, a, k).to_bits(),
            "the stored table is not exactly symmetric at ({a}, {b}, {k})"
        );
        checked += 1;
    }
}

// ============================================================ T1 and T2

#[test]
fn t1_the_interpolant_is_faithful_off_its_nodes() {
    let t = table();
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    let n = if cfg!(debug_assertions) {
        T1_POINTS_DEBUG
    } else {
        T1_POINTS
    };
    let held = held_out(n);

    // The O-H pair energy is a function of the side alone, and the draw has 2n distinct
    // sides, so there is nothing to cache; the triple solve dominates anyway.
    let mut worst = 0.0f64;
    let mut worst_at = (0.0, 0.0, 0.0);
    let mut nonzero = 0usize;
    for &(x, y, c) in &held {
        let u = 1.0 - c * c;
        let truth = de3_with(
            x,
            y,
            u,
            e_o,
            e_h,
            pair_point(OXYGEN, HYDROGEN, x).e,
            pair_point(OXYGEN, HYDROGEN, y).e,
        );
        let (got, _) = t.eval(x, y, hh_side(x, y, u));
        let d = (got - truth).abs();
        if d > 0.0 {
            nonzero += 1;
        }
        if d > worst {
            worst = d;
            worst_at = (x, y, c);
        }
    }

    // Two-sided, as the prereg requires. An all-zero error would mean the draw had landed
    // on nodes and the gate had measured its own construction.
    assert_eq!(
        nonzero, n,
        "{} of {n} held-out points returned an EXACTLY zero error; the draw is landing on \
         nodes and this gate is measuring nothing",
        n - nonzero
    );
    assert!(
        worst <= WATER_T1_KILL_E,
        "T1 FIRED: worst held-out error {worst:.3e} Ha at (x, y, c) = ({:.4}, {:.4}, {:.4}) \
         exceeds the staked kill {WATER_T1_KILL_E:.0e}",
        worst_at.0,
        worst_at.1,
        worst_at.2
    );
    assert!(
        worst <= WATER_T1_MEASURED_E,
        "the measured T1 error has regressed to {worst:.3e} Ha at (x, y, c) = ({:.4}, \
         {:.4}, {:.4}); the pin is {WATER_T1_MEASURED_E:.3e}",
        worst_at.0,
        worst_at.1,
        worst_at.2
    );
    // In debug the draw is a prefix, so its maximum is legitimately smaller and the
    // stale-pin check would fire on a correct run. It is enforced where the full draw runs.
    if !cfg!(debug_assertions) {
        assert!(
            worst > WATER_T1_MEASURED_E / 100.0,
            "the pinned T1 error {WATER_T1_MEASURED_E:.3e} is more than two decades looser \
             than the measured {worst:.3e}; re-pin it"
        );
    }
}

#[test]
fn t2_the_truncation_costs_what_the_domain_says() {
    // The surface is zeroed outside `max(O-H) = R_HI`, so what that costs is the largest
    // |dE3| anywhere ON that boundary.
    //
    // # Two things this sweep does that the first design sweep did not, because it FIRED
    //
    // The design sweep put its angle floor at the table's own fence, `c = C_LO`, and every
    // shell's worst reading came back sitting exactly on it — which is the signature of a
    // maximum outside the grid rather than on it. The truncation zeroes every geometry
    // with `y > R_HI` WHATEVER its angle, so the sweep has to reach past the table's fence;
    // it goes to `c = 0.002`.
    //
    // And a grid maximum is a LOWER BOUND on its own supremum. Re-swept at five times the
    // resolution, the `b = 14` shell went from 9.71e-6 to 1.0091e-5 and crossed the stake.
    // So the search here is two-stage: a declared coarse grid, then a refinement around
    // whatever that grid's argmax turns out to be — computed at runtime, not chosen from a
    // result.
    const C_FLOOR: f64 = 0.002;
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    let e_oy = pair_point(OXYGEN, HYDROGEN, R_HI).e;
    let at = |x: f64, c: f64| {
        de3_with(
            x,
            R_HI,
            1.0 - c * c,
            e_o,
            e_h,
            pair_point(OXYGEN, HYDROGEN, x).e,
            e_oy,
        )
        .abs()
    };

    let (nx, nc) = if cfg!(debug_assertions) { (9, 9) } else { (17, 17) };
    let (mut lo_x, mut hi_x) = (R_LO, R_HI);
    let (mut lo_c, mut hi_c) = (C_FLOOR, C_HI);
    let (mut worst, mut wx, mut wc) = (0.0f64, 0.0f64, 0.0f64);
    for _ in 0..3 {
        worst = 0.0;
        for i in 0..nx {
            let x = lo_x + (hi_x - lo_x) * i as f64 / (nx - 1) as f64;
            for k in 0..nc {
                let c = lo_c + (hi_c - lo_c) * k as f64 / (nc - 1) as f64;
                let d = at(x, c);
                if d > worst {
                    worst = d;
                    wx = x;
                    wc = c;
                }
            }
        }
        let (dx, dc) = (
            (hi_x - lo_x) / (nx - 1) as f64,
            (hi_c - lo_c) / (nc - 1) as f64,
        );
        lo_x = (wx - dx).max(R_LO);
        hi_x = (wx + dx).min(R_HI);
        lo_c = (wc - dc).max(C_FLOOR);
        hi_c = (wc + dc).min(C_HI);
    }
    println!(
        "T2: truncation systematic {worst:.4e} Ha on the shell max(O-H) = {R_HI}, at \
         (x, c) = ({wx:.4}, {wc:.5})"
    );
    assert!(
        worst <= WATER_T2_KILL_E,
        "T2 FIRED: the truncation systematic is {worst:.4e} Ha at (x, c) = ({wx:.4}, \
         {wc:.5}) on the shell max(O-H) = {R_HI}, above the staked {WATER_T2_KILL_E:.0e}"
    );
    assert!(
        worst <= WATER_T2_MEASURED_E,
        "the measured T2 systematic has regressed to {worst:.4e} Ha; the pin is \
         {WATER_T2_MEASURED_E:.3e}"
    );
    assert!(
        worst > WATER_T2_MEASURED_E / 100.0,
        "the pinned T2 systematic {WATER_T2_MEASURED_E:.3e} is more than two decades looser \
         than the measured {worst:.4e}; re-pin it"
    );
}

// ============================================================ the symmetry

#[test]
fn the_exchange_symmetry_is_bit_exact_and_a_broken_table_fires() {
    // Plant (ii). Three separate claims, each with its carrier asserted before it is
    // scored:
    //   1. H <-> H exchange is EXACT, bit-level, on a staked asymmetric geometry;
    //   2. a deliberately desymmetrised table fires above 1e-6 Ha;
    //   3. the O-distinct axis is NOT symmetrised — a table wrongly symmetrised over all
    //      three sides disagrees with the true surface by orders.
    let t = table();

    // 1 — exactness. The geometry is asymmetric on purpose: equal O-H sides would make
    // the exchange a no-op and the test would pass on a table with no symmetry at all.
    let (a, b, z) = (1.6f64, 2.9f64, 3.1f64);
    let (v1, g1) = t.eval(a, b, z);
    let (v2, g2) = t.eval(b, a, z);
    assert!(
        (a - b).abs() > 0.5,
        "the carrier is empty: this geometry's two O-H sides are not distinct"
    );
    assert!(
        v1.abs() > 1e-6,
        "the carrier is empty: dE3 is {v1:.3e} here, so an exchange could not move anything"
    );
    assert_eq!(
        v1.to_bits(),
        v2.to_bits(),
        "H <-> H exchange moved the value by {:.3e} Ha; the sort was supposed to make it \
         bit-identical",
        (v1 - v2).abs()
    );
    for (p, q) in [(0usize, 1usize), (1, 0), (2, 2)] {
        assert_eq!(
            g1[p].to_bits(),
            g2[q].to_bits(),
            "H <-> H exchange moved gradient slot {p} by {:.3e}",
            (g1[p] - g2[q]).abs()
        );
    }

    // 2 — the plant fires. One node moved on ONE side of the diagonal only.
    let (i, j, k) = (12usize, 30usize, 12usize);
    let mut broken = t.clone();
    let before = broken.node(i, j, k);
    assert!(
        before.abs() > 1e-6,
        "the carrier is empty: node ({i}, {j}, {k}) is {before:.3e}, so desymmetrising it \
         moves nothing"
    );
    broken.set_node_asymmetric(i, j, k, before * 1.5);
    let (x, y) = (node_r(i), node_r(j));
    let c = node_c(k);
    let z2 = hh_side(x, y, 1.0 - c * c);
    let (bv1, _) = broken.eval(x, y, z2);
    let (bv2, _) = broken.eval(y, x, z2);
    assert!(
        (bv1 - bv2).abs() >= 1e-6,
        "plant (ii) MISSED: a desymmetrised table still reads the same both ways, to \
         {:.3e} Ha",
        (bv1 - bv2).abs()
    );

    // 3 — oxygen is not sorted in. A table read with all three sides sorted would answer
    // a different question at any geometry where the H-H side is not the largest; this
    // asserts the two readings actually differ, which is what makes the O axis distinct.
    let (p, q, r) = (1.6f64, 2.9f64, 2.2f64);
    let mut s = [p, q, r];
    s.sort_by(|m, n| m.partial_cmp(n).unwrap());
    let (correct, _) = t.eval(p, q, r);
    let (all_sorted, _) = t.eval(s[0], s[1], s[2]);
    assert!(
        (correct - all_sorted).abs() > 1e-4,
        "the O-distinct axis is not carrying anything: sorting all three sides changes the \
         reading by only {:.3e} Ha, so this table would be a relabelled H3 table",
        (correct - all_sorted).abs()
    );
}

// ============================================================ G1

/// The MBE3 energy of one (O, H, H) geometry: the pair terms plus the TABULATED
/// three-body term.
struct Mbe3<'a> {
    t: &'a WaterTable,
    oh: &'a PairCurve,
    hh: &'a PairCurve,
    e_o: f64,
    e_h: f64,
}

impl Mbe3<'_> {
    fn at(&self, x: f64, y: f64, theta: f64) -> f64 {
        let u = theta.cos();
        let z = hh_side(x, y, u);
        let pairs = self.oh.at(x) + self.oh.at(y) - 2.0 * (self.e_o + self.e_h) + self.hh.at(z)
            - 2.0 * self.e_h;
        let (d, _) = self.t.eval(x, y, z);
        self.e_o + 2.0 * self.e_h + pairs + d
    }
}

#[test]
fn g1_the_bent_geometry_emerges() {
    // THE REFERENCE, computed by `examples/s2_design.rs` from the model's own full CI by
    // Newton on exact derivatives, BEFORE this table existed. Reproduced here as the two
    // numbers G1 scores against; the design example re-derives them on demand.
    //
    // [LABELLED CONTEXT, never compared against: nature's water is 104.5 degrees and
    // 0.957 angstrom = 1.8085 bohr. STO-3G's in-model answer is the claim.]
    const FCI_R: f64 = 1.9435740105;
    const FCI_THETA_DEG: f64 = 96.75788837;

    let t = table();
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    let (oh, hh) = (
        PairCurve::sample(OXYGEN, HYDROGEN),
        PairCurve::sample(HYDROGEN, HYDROGEN),
    );
    let m = Mbe3 { t: &t, oh: &oh, hh: &hh, e_o, e_h };

    // Minimise over the symmetric stretch and the angle. The antisymmetric direction is
    // checked separately below rather than searched: `s2_design` measured its curvature
    // positive, so the minimum is on the symmetric line and a two-parameter search that
    // found otherwise would be reporting a different surface.
    let mut best = (f64::INFINITY, 0.0f64, 0.0f64);
    let mut lo_r = 1.4f64;
    let mut hi_r = 2.6f64;
    let mut lo_t = 50.0f64 * PI / 180.0;
    let mut hi_t = 180.0f64 * PI / 180.0;
    for _ in 0..6 {
        best = (f64::INFINITY, 0.0, 0.0);
        for i in 0..21 {
            let r = lo_r + (hi_r - lo_r) * i as f64 / 20.0;
            for j in 0..21 {
                let th = lo_t + (hi_t - lo_t) * j as f64 / 20.0;
                let e = m.at(r, r, th);
                if e < best.0 {
                    best = (e, r, th);
                }
            }
        }
        let (dr, dt) = ((hi_r - lo_r) / 20.0, (hi_t - lo_t) / 20.0);
        lo_r = best.1 - dr;
        hi_r = best.1 + dr;
        lo_t = best.2 - dt;
        hi_t = best.2 + dt;
    }
    let (e_min, r_min, th_min) = best;
    let deg = th_min * 180.0 / PI;

    // THE KILL, in the prereg's own words: an MBE3 optimum QUALITATIVELY WRONG — linear
    // where the FCI is bent — kills the truncation's fitness for this triple.
    assert!(
        deg < 175.0,
        "G1 FIRED: the MBE3 optimum is at {deg:.3} degrees, which is the LINEAR geometry. \
         The model's own full CI puts water at {FCI_THETA_DEG:.3} degrees, so the \
         tabulated three-body term is not fit for this triple and the campaign says so."
    );
    assert!(
        deg > 20.0 && r_min > R_LO && r_min < 4.0,
        "G1 FIRED: the MBE3 optimum ran to the edge of the search box at r = {r_min:.4} \
         bohr, {deg:.3} degrees"
    );

    // BOTH DEVIATIONS REPORTED, as the prereg requires. The bounds are generous on
    // purpose: what G1 asserts is that the optimum is the right one qualitatively and
    // close quantitatively, and T1 is where the interpolation error is actually gauged.
    let d_r = (r_min - FCI_R).abs();
    let d_deg = (deg - FCI_THETA_DEG).abs();
    println!(
        "G1: MBE3 optimum r = {r_min:.6} bohr ({d_r:+.2e} vs FCI), theta = {deg:.4} deg \
         ({d_deg:+.2e} vs FCI), E = {e_min:.9} Ha"
    );
    assert!(
        d_r < 0.05,
        "G1: the MBE3 bond length {r_min:.6} bohr is {d_r:.3e} from the model's own FCI \
         optimum {FCI_R}; T1's interpolation error cannot account for that"
    );
    assert!(
        d_deg < 2.0,
        "G1: the MBE3 angle {deg:.4} degrees is {d_deg:.3e} from the model's own FCI \
         optimum {FCI_THETA_DEG}; T1's interpolation error cannot account for that"
    );

    // The antisymmetric direction, as a reading rather than an assumption: the minimum
    // must actually be ON the symmetric line.
    let s = 0.05;
    let e_asym = m.at(r_min + s, r_min - s, th_min);
    assert!(
        e_asym > e_min,
        "G1: stretching one O-H bond and shortening the other LOWERS the MBE3 energy by \
         {:.3e} Ha, so the symmetric point is a saddle and the search was on the wrong line",
        e_min - e_asym
    );
}

#[test]
fn plant_i_the_sign_flip_inverts_the_bend() {
    // Plant (i): negating the table must invert G1's bent-vs-unbent preference. The
    // carrier is the geometry shift, and it is asserted large before the plant is scored.
    let t = table();
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    let (oh, hh) = (
        PairCurve::sample(OXYGEN, HYDROGEN),
        PairCurve::sample(HYDROGEN, HYDROGEN),
    );
    let r = 1.9435740105;
    let bent = 96.75788837 * PI / 180.0;
    let linear = PI;

    let m = Mbe3 { t: &t, oh: &oh, hh: &hh, e_o, e_h };
    let true_bent = m.at(r, r, bent);
    let true_linear = m.at(r, r, linear);
    assert!(
        true_linear - true_bent > 1e-2,
        "the carrier is empty: on the true table the bent geometry is only {:.3e} Ha below \
         the linear one, so a sign flip has nothing to invert",
        true_linear - true_bent
    );

    let mut flipped = t.clone();
    flipped.negate();
    let f = Mbe3 { t: &flipped, oh: &oh, hh: &hh, e_o, e_h };
    let p_bent = f.at(r, r, bent);
    let p_linear = f.at(r, r, linear);
    assert!(
        p_linear < p_bent,
        "plant (i) MISSED: with the three-body table negated the bent geometry is STILL \
         preferred, by {:.3e} Ha. The bend is then not coming from the tabulated term and \
         G1 is not measuring what it claims.",
        p_bent - p_linear
    );
}

#[test]
fn plant_iii_the_swapped_table_fires() {
    // Plant (iii): serving the (H, H, H) table to an (O, H, H) triple must move G1 beyond
    // tolerance by ORDERS. The carrier is the energy shift, asserted before scoring.
    let t = table();
    let h3 = holon_chem::trimer::generate().expect("the H3 table builds");
    let r = 1.9435740105;
    let bent = 96.75788837 * PI / 180.0;
    let z = hh_side(r, r, bent.cos());

    let (mine, _) = t.eval(r, r, z);
    let (wrong, _) = h3.eval([r, r, z]);
    assert!(
        mine.abs() > 1e-3,
        "the carrier is empty: the (O,H,H) table reads {mine:.3e} Ha at water's own \
         optimum, so swapping tables could not move anything"
    );
    assert!(
        (mine - wrong).abs() > 100.0 * (mine.abs() * 1e-2).max(1e-3),
        "plant (iii) MISSED: the H3 table reads {wrong:.6e} Ha where the (O,H,H) table \
         reads {mine:.6e} Ha — a difference of only {:.3e}, which is not the orders the \
         plant stakes",
        (mine - wrong).abs()
    );
}

// ============================================================ G2

#[test]
fn g2_the_third_hydrogen_refuses_and_the_second_binds() {
    // VALENCE SATURATION, two-sided.
    //
    // The SECOND hydrogen binding deeply is water existing at all: E(OHH) at the relaxed
    // geometry against relaxed OH plus a free H.
    //
    // The THIRD hydrogen refusing is the claim. In an MBE3 world a third hydrogen brought
    // to relaxed water feels three new pair terms and three new three-body terms, and the
    // best binding it can find anywhere must be shallower than water's own second O-H
    // bond by the staked factor.
    let t = table();
    let e_o = atom_energy(OXYGEN);
    let e_h = atom_energy(HYDROGEN);
    let (oh, hh) = (
        PairCurve::sample(OXYGEN, HYDROGEN),
        PairCurve::sample(HYDROGEN, HYDROGEN),
    );
    let m = Mbe3 { t: &t, oh: &oh, hh: &hh, e_o, e_h };
    let r_w = 1.9435740105;
    let th_w = 96.75788837 * PI / 180.0;

    // --- the control: the second hydrogen binds -------------------------------------
    // D(second) = [E(OH) + E(H)] - E(OHH), all in-model, the OH fragment relaxed.
    let mut r_oh = 1.8f64;
    for _ in 0..40 {
        let (lo, hi) = (r_oh - 0.01, r_oh + 0.01);
        if pair_point(OXYGEN, HYDROGEN, lo).e < pair_point(OXYGEN, HYDROGEN, hi).e {
            r_oh -= 0.01;
        } else {
            r_oh += 0.01;
        }
    }
    let e_oh = pair_point(OXYGEN, HYDROGEN, r_oh).e;
    let e_water = m.at(r_w, r_w, th_w);
    let d_second = (e_oh + e_h) - e_water;
    assert!(
        d_second > 0.05,
        "G2's CONTROL FAILED: the second hydrogen binds by only {d_second:.4e} Ha, so \
         water does not exist in this model and the refusal of a third proves nothing"
    );

    // --- the claim: the third hydrogen refuses ---------------------------------------
    // A third hydrogen is placed on a spherical shell around the relaxed water and the
    // deepest binding anywhere on the scan is taken.
    //
    // The pair curves come from [`PairCurve`] — sampled once, read by cubic Hermite with
    // the solver's own exact derivatives. See its doc comment for why the loops below do
    // not call `pair_point` directly.
    let e_oh_at = |r: f64| oh.at(r);
    let e_hh_at = |r: f64| hh.at(r);
    let o = [0.0f64, 0.0, 0.0];
    let h1 = [r_w * (th_w / 2.0).cos(), r_w * (th_w / 2.0).sin(), 0.0];
    let h2 = [r_w * (th_w / 2.0).cos(), -r_w * (th_w / 2.0).sin(), 0.0];
    let dist = |a: [f64; 3], b: [f64; 3]| {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    };
    // The H3 table is built ONCE, outside the scan: the third hydrogen makes a
    // hydrogen-only triple with the two that are already there, and that triple is
    // SATURATION-1's surface, not this one's.
    let h3 = holon_chem::trimer::generate().expect("the H3 table builds");
    let mut deepest = 0.0f64;
    let (n_r, n_th, n_ph) = if cfg!(debug_assertions) { (7, 7, 7) } else { (19, 17, 17) };
    for i in 0..n_r {
        let rr = 1.2 + (6.0 - 1.2) * i as f64 / (n_r - 1) as f64;
        for j in 0..n_th {
            let a = PI * j as f64 / (n_th - 1) as f64;
            for k in 0..n_ph {
                let b = 2.0 * PI * k as f64 / n_ph as f64;
                let probe = [
                    rr * a.sin() * b.cos(),
                    rr * a.sin() * b.sin(),
                    rr * a.cos(),
                ];
                let (d_oh3, d_h1h3, d_h2h3) =
                    (dist(o, probe), dist(h1, probe), dist(h2, probe));
                if d_oh3 < 0.9 || d_h1h3 < 0.9 || d_h2h3 < 0.9 {
                    continue;
                }
                // Every new pair, and every new triple the fourth atom creates. The
                // (O, O, H) fence does not arise here — there is one oxygen — so the
                // three new triples are (O, H1, H3), (O, H2, H3) and (H1, H2, H3).
                let new_pairs = (e_oh_at(d_oh3) - e_o - e_h)
                    + (e_hh_at(d_h1h3) - 2.0 * e_h)
                    + (e_hh_at(d_h2h3) - 2.0 * e_h);
                let (t_a, _) = t.eval(r_w, d_oh3, d_h1h3);
                let (t_b, _) = t.eval(r_w, d_oh3, d_h2h3);
                let (t_c, _) = h3.eval([dist(h1, h2), d_h1h3, d_h2h3]);
                let binding = -(new_pairs + t_a + t_b + t_c);
                if binding > deepest {
                    deepest = binding;
                }
            }
        }
    }

    let ratio = d_second / deepest.max(1e-12);
    println!(
        "G2: second O-H bond {d_second:.6} Ha, deepest third-H binding {deepest:.6} Ha, \
         ratio {ratio:.2}x (staked >= {WATER_G2_STAKED_RATIO})"
    );
    assert!(
        ratio >= WATER_G2_STAKED_RATIO,
        "G2 FIRED (branch b — investigate, never massage): the third hydrogen binds at \
         {deepest:.6} Ha against water's own second O-H bond of {d_second:.6} Ha, a ratio \
         of {ratio:.2}x against the staked {WATER_G2_STAKED_RATIO}x"
    );
}

// ============================================================ housekeeping

#[test]
fn the_table_is_zero_outside_its_domain_and_finite_inside() {
    let t = table();
    for (a, b, z) in [
        (2.0f64, R_HI + 0.1, 3.0f64),
        (R_HI + 5.0, R_HI + 6.0, 2.0),
        (f64::NAN, 2.0, 3.0),
        (2.0, f64::NAN, 3.0),
    ] {
        let (v, g) = t.eval(a, b, z);
        assert_eq!(
            (v, g),
            (0.0, [0.0; 3]),
            "outside the domain the surface must be an exact zero, not {v:?} / {g:?}"
        );
    }
    // Inside, at the corners the grid is built on, nothing is a NaN and nothing is huge.
    for &(i, j, k) in &[(0usize, 0usize, 0usize), (0, NR - 1, NU - 1), (NR - 1, NR - 1, 0)] {
        let (x, y) = (node_r(i), node_r(j));
        let c = node_c(k);
        let z = hh_side(x, y, 1.0 - c * c);
        let (v, g) = t.eval(x, y, z);
        assert!(
            v.is_finite() && g.iter().all(|q| q.is_finite()),
            "node ({i}, {j}, {k}) evaluates to {v} / {g:?}"
        );
    }
    assert_eq!(t.meta.n_nodes, N_NODES);
    assert_eq!(node_index(NR - 1, NR - 1, NU - 1), N_NODES - 1);
}

#[test]
fn the_sort_kink_is_at_roundoff_because_oxygen_is_never_sorted() {
    // `TrimerTable` has a real sort kink and reports it. This table should not: the H-H
    // side never enters the sort, and the table is exactly symmetric in the only pair
    // that does. That is a claim, so it is measured.
    let t = table();
    assert!(
        t.sort_kink < 1e-9,
        "the H <-> H sort boundary carries a force discontinuity of {:.3e} Ha/bohr, which \
         is far above roundoff — the stored table is not exactly symmetric after all",
        t.sort_kink
    );
    assert!(
        t.curvature_envelope > 0.0 && t.curvature_per_gradient > 0.0,
        "the curvature envelopes are zero, so the drift bound has nothing to read"
    );
}
