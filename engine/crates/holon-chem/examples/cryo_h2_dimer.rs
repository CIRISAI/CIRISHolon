//! CRYO-H-O ARM 1 — the model's own H2–H2 interaction, and the engine's.
//!
//! Frozen by `conformance/atomworld/CRYO_HO_PREREG.md` (2026-09-02, commit fc7b6a0).
//! Gates G1, G2, G3 and plants P1, P2. Nothing here is fitted, tabulated or
//! interpolated: every sub-cluster energy is a fresh exact-in-model FCI solve, so a
//! residual this file reports belongs to the many-body EXPANSION and never to a table.
//!
//! ```text
//! cargo run --release -p holon-chem --example cryo_h2_dimer
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{Species, HYDROGEN};
use holon_chem::fci::SolveExit;
use holon_chem::pair::solve_geometry;

// ============================================================= THE FROZEN PROTOCOL

/// The referee's H2 bond length, `h2_potential.json` `R_e`, to all its digits. Written
/// out rather than recomputed: the prereg names this number, and a protocol whose
/// geometry is whatever a minimiser found today is not a frozen protocol.
const R_E: f64 = 1.388_694_018_017_776_3;

/// Centre-to-centre separations, bohr. Staked in the freeze.
const SEPARATIONS: [f64; 13] = [
    2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0,
];

/// G1's bar: a well this deep or deeper kills the no-well reading. 1.0e-5 Ha = 3.16 K.
const G1_WELL_BAR: f64 = 1.0e-5;
/// G1 is asked from here outward; below it the two molecules are one compressed H4 and
/// the question "is there an intermolecular well" has no referent.
const G1_FROM: f64 = 3.0;
/// G2's bar and band.
const G2_BAR: f64 = 1.0e-4;
const G2_BAND: (f64, f64) = (3.0, 6.0);
/// P1 must invert G1 by at least this much.
const P1_BAR: f64 = 1.0e-4;
/// P2 must move the reported residual by at least this factor.
const P2_FACTOR: f64 = 10.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Orient {
    /// Parallel axes, centres offset perpendicular to both.
    H,
    /// One axis along the centre line, one across it.
    T,
    /// Collinear, end to end.
    L,
}

impl Orient {
    fn label(self) -> &'static str {
        match self {
            Orient::H => "H (parallel)",
            Orient::T => "T (perpendicular)",
            Orient::L => "L (collinear)",
        }
    }

    /// The four hydrogens: molecule A's two, then molecule B's two. Every scene is
    /// coplanar in `z = 0` — the campaign's 2D standing fence.
    fn scene(self, r: f64) -> [[f64; 3]; 4] {
        let h = 0.5 * R_E;
        match self {
            Orient::H => [
                [-h, 0.0, 0.0],
                [h, 0.0, 0.0],
                [-h, r, 0.0],
                [h, r, 0.0],
            ],
            Orient::T => [
                [-h, 0.0, 0.0],
                [h, 0.0, 0.0],
                [0.0, r - h, 0.0],
                [0.0, r + h, 0.0],
            ],
            Orient::L => [
                [-h, 0.0, 0.0],
                [h, 0.0, 0.0],
                [r - h, 0.0, 0.0],
                [r + h, 0.0, 0.0],
            ],
        }
    }
}

// ================================================================ the solve, disclosed

/// One FCI solve with every disclosure field the freeze demands beside it.
#[derive(Clone, Debug)]
struct Solve {
    e: f64,
    exit: SolveExit,
    residual: f64,
    n_det: usize,
    n_basis: usize,
    iters: usize,
    scf_converged: bool,
    s_min: f64,
    device: String,
}

impl Solve {
    fn converged(&self) -> bool {
        self.exit.is_converged() && self.scf_converged
    }
}

fn fci(species: &[Species], pos: &[[f64; 3]]) -> Solve {
    let centers: Vec<[D2; 3]> = pos
        .iter()
        .map(|p| [D2::c(p[0]), D2::c(p[1]), D2::c(p[2])])
        .collect();
    let s = solve_geometry(species, centers);
    Solve {
        e: s.e.v,
        exit: s.exit,
        residual: s.residual,
        n_det: s.n_det,
        n_basis: s.n_basis,
        iters: s.davidson_iters,
        scf_converged: s.scf_converged,
        s_min: s.s_min_eigenvalue,
        device: format!("{:?}", s.device),
    }
}

fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let (dx, dy, dz) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    (dx * dx + dy * dy + dz * dz).sqrt()
}

// ============================================================ the expansion, exactly

/// Everything one geometry produces at all three levels, plus the work count.
struct Reading {
    r: f64,
    orient: Orient,
    /// `E_FCI(4H) − 2 E_FCI(H2)`, the MODEL's own answer.
    e_int_exact: f64,
    /// The four CROSS pair excesses. The two intramolecular pairs sit at `R_E` in both
    /// the dimer and the isolated molecules and cancel to the last bit, so the pair-level
    /// interaction IS the cross sum — written out rather than differenced, because a
    /// difference of two nearly equal sums is where a cancellation error would live.
    e_int_mbe2: f64,
    /// `+ Σ dE3` over the four triples. The isolated molecules carry no triple, so the
    /// three-body interaction is the whole sum.
    e_int_mbe3: f64,
    /// The carrier P2 acts on, printed so the plant's sector can be seen to be nonzero.
    sum_de3: f64,
    /// `E_FCI − E_MBE3`: the four-body term, which is what the expansion is missing.
    de4: f64,
    /// The shortest cross distance, for the localization clause.
    r_min_cross: f64,
    quad: Solve,
    /// How many of this reading's sub-cluster solves did not exit converged.
    unconverged_subclusters: usize,
    worst_sub_residual: f64,
    solves: usize,
}

/// One separation, one orientation, all four levels, from exact sub-cluster solves.
fn read(orient: Orient, r: f64, e_h: f64, e_h2: f64) -> Reading {
    let sp4 = [HYDROGEN; 4];
    let pos = orient.scene(r);
    let quad = fci(&sp4, &pos);
    let mut solves = 1usize;
    let mut bad = 0usize;
    let mut worst = 0.0f64;

    // Pair excesses over all six pairs, and the cross sum kept separately.
    let mut v2 = [[0.0f64; 4]; 4];
    let mut sum_v2 = 0.0f64;
    let mut cross = 0.0f64;
    let mut r_min_cross = f64::INFINITY;
    for i in 0..4 {
        for j in (i + 1)..4 {
            let s = fci(&[HYDROGEN; 2], &[pos[i], pos[j]]);
            solves += 1;
            if !s.converged() {
                bad += 1;
            }
            worst = worst.max(s.residual);
            let e = s.e - 2.0 * e_h;
            v2[i][j] = e;
            v2[j][i] = e;
            sum_v2 += e;
            // Molecule A is slots 0,1 and molecule B is slots 2,3; a cross pair has one
            // slot from each.
            let is_cross = (i < 2) != (j < 2);
            if is_cross {
                cross += e;
                r_min_cross = r_min_cross.min(dist(&pos[i], &pos[j]));
            }
        }
    }

    // Exact three-body terms over all four triples.
    let mut sum_de3 = 0.0f64;
    for i in 0..4 {
        for j in (i + 1)..4 {
            for k in (j + 1)..4 {
                let s = fci(&[HYDROGEN; 3], &[pos[i], pos[j], pos[k]]);
                solves += 1;
                if !s.converged() {
                    bad += 1;
                }
                worst = worst.max(s.residual);
                sum_de3 += s.e - 3.0 * e_h - v2[i][j] - v2[i][k] - v2[j][k];
            }
        }
    }

    let e_mbe2 = 4.0 * e_h + sum_v2;
    let e_mbe3 = e_mbe2 + sum_de3;
    Reading {
        r,
        orient,
        e_int_exact: quad.e - 2.0 * e_h2,
        e_int_mbe2: cross,
        e_int_mbe3: cross + sum_de3,
        sum_de3,
        de4: quad.e - e_mbe3,
        r_min_cross,
        unconverged_subclusters: bad,
        worst_sub_residual: worst,
        solves,
        quad,
    }
}

// ==================================================================== the gate report

fn main() {
    println!("# CRYO-H-O ARM 1 — the H2-H2 interaction, model and engine");
    println!("# prereg conformance/atomworld/CRYO_HO_PREREG.md, frozen fc7b6a0");
    println!("# STANDING FENCES: 2D scene | classical nuclei (no NQE) | STO-3G minimal basis");
    println!("# r(H2) = {R_E:.16} bohr, the 50-digit referee's R_e, frozen");

    // The two references. Both are exact-in-model solves and both are disclosed.
    let e_h_s = fci(&[HYDROGEN], &[[0.0, 0.0, 0.0]]);
    let e_h2_s = fci(&[HYDROGEN; 2], &[[-0.5 * R_E, 0.0, 0.0], [0.5 * R_E, 0.0, 0.0]]);
    let (e_h, e_h2) = (e_h_s.e, e_h2_s.e);
    println!("#");
    println!(
        "# E(H)  = {:.12} Ha   exit {:?}  resid {:.2e}  n_det {}  n_basis {}  scf {}  device {}",
        e_h, e_h_s.exit, e_h_s.residual, e_h_s.n_det, e_h_s.n_basis, e_h_s.scf_converged,
        e_h_s.device
    );
    println!(
        "# E(H2) = {:.12} Ha   exit {:?}  resid {:.2e}  n_det {}  n_basis {}  scf {}  device {}",
        e_h2, e_h2_s.exit, e_h2_s.residual, e_h2_s.n_det, e_h2_s.n_basis, e_h2_s.scf_converged,
        e_h2_s.device
    );
    println!(
        "# D_e(H2) = {:.12} Ha   [banked referee 0.204142352107591]",
        2.0 * e_h - e_h2
    );

    let mut all: Vec<Reading> = Vec::new();
    let mut solves = 0usize;
    for orient in [Orient::H, Orient::T, Orient::L] {
        println!("#");
        println!("# ---- {} ----", orient.label());
        println!(
            "#   R      E_int(exact)    E_int(MBE2)     E_int(MBE3)     sum dE3        \
             dE4           r_min_x  exit        resid     bad"
        );
        for &r in SEPARATIONS.iter() {
            let rd = read(orient, r, e_h, e_h2);
            solves += rd.solves;
            println!(
                "  {:5.2}  {:+.8e}  {:+.8e}  {:+.8e}  {:+.6e}  {:+.6e}  {:6.3}  {:<10}  {:.1e}  {}",
                rd.r,
                rd.e_int_exact,
                rd.e_int_mbe2,
                rd.e_int_mbe3,
                rd.sum_de3,
                rd.de4,
                rd.r_min_cross,
                format!("{:?}", rd.quad.exit),
                rd.quad.residual.max(rd.worst_sub_residual),
                rd.unconverged_subclusters
            );
            all.push(rd);
        }
    }

    // ------------------------------------------------------------------ G1
    println!("\n# ================================ G1 — has the MODEL an H2-H2 well?");
    println!("# staked: E_int(R) > -{G1_WELL_BAR:.1e} Ha for every R >= {G1_FROM} bohr, all orientations");
    let mut g1_worst: Option<&Reading> = None;
    for rd in all.iter().filter(|d| d.r >= G1_FROM) {
        if g1_worst.map_or(true, |b| rd.e_int_exact < b.e_int_exact) {
            g1_worst = Some(rd);
        }
    }
    let g1w = g1_worst.expect("the sweep is non-empty");
    let g1_holds = g1w.e_int_exact > -G1_WELL_BAR;
    println!(
        "# deepest E_int anywhere at R >= {}: {:+.6e} Ha at R = {:.2} bohr, {}  ({:.3} K)",
        G1_FROM,
        g1w.e_int_exact,
        g1w.r,
        g1w.orient.label(),
        g1w.e_int_exact * 315_775.0
    );
    println!("# G1: {}", if g1_holds { "HOLDS — no well" } else { "KILLED — a well" });

    // Sign census: is the exact interaction repulsive everywhere it is asked?
    let n_attractive = all
        .iter()
        .filter(|d| d.r >= G1_FROM && d.e_int_exact < 0.0)
        .count();
    let n_asked = all.iter().filter(|d| d.r >= G1_FROM).count();
    println!("# sign census at R >= {G1_FROM}: {n_attractive} of {n_asked} points attractive at all");

    // ------------------------------------------------------------------ G2
    println!("\n# ================================ G2 — does the ENGINE's pair level invent binding?");
    println!(
        "# staked: E_int_MBE2 < E_int_exact - {:.1e} Ha somewhere in R in [{}, {}]",
        G2_BAR, G2_BAND.0, G2_BAND.1
    );
    let mut g2_worst: Option<&Reading> = None;
    for rd in all.iter().filter(|d| d.r >= G2_BAND.0 && d.r <= G2_BAND.1) {
        let gap = rd.e_int_mbe2 - rd.e_int_exact;
        if g2_worst.map_or(true, |b| gap < b.e_int_mbe2 - b.e_int_exact) {
            g2_worst = Some(rd);
        }
    }
    let g2w = g2_worst.expect("the band is non-empty");
    let g2_gap = g2w.e_int_mbe2 - g2w.e_int_exact;
    let g2_holds = g2_gap < -G2_BAR;
    println!(
        "# worst MBE2 over-binding in band: {:+.6e} Ha at R = {:.2}, {}  \
         (exact {:+.6e}, MBE2 {:+.6e})",
        g2_gap, g2w.r, g2w.orient.label(), g2w.e_int_exact, g2w.e_int_mbe2
    );
    println!(
        "# G2: {}",
        if g2_holds { "HOLDS — the pair level over-binds" } else { "KILLED — the pair level is faithful" }
    );

    // ------------------------------------------------------------------ G3
    println!("\n# ================================ G3 — does the three-body term correct it?");
    println!("# staked: |MBE3 - exact| < |MBE2 - exact| at G2's worst point. No band on the size.");
    let r2 = (g2w.e_int_mbe2 - g2w.e_int_exact).abs();
    let r3 = (g2w.e_int_mbe3 - g2w.e_int_exact).abs();
    let g3_holds = r3 < r2;
    println!(
        "# at R = {:.2}, {}:  |MBE2 residual| {:.6e}  ->  |MBE3 residual| {:.6e}   ({:.1}x)",
        g2w.r,
        g2w.orient.label(),
        r2,
        r3,
        if r3 > 0.0 { r2 / r3 } else { f64::INFINITY }
    );
    println!(
        "# G3: {}",
        if g3_holds { "HOLDS — MBE3 is closer" } else { "KILLED — MBE3 is further" }
    );

    // ------------------------------------------------------------------ P1
    println!("\n# ================================ P1 — the sign plant (carrier: intermolecular channel)");
    let carrier: Vec<&Reading> = all
        .iter()
        .filter(|d| d.r >= G2_BAND.0 && d.r <= G2_BAND.1 && d.e_int_exact.abs() > 1.0e-4)
        .collect();
    println!(
        "# carrier NONZERO IN the intermolecular channel: {} of {} points in R in [{}, {}] \
         have |E_int| > 1.0e-4 Ha",
        carrier.len(),
        all.iter().filter(|d| d.r >= G2_BAND.0 && d.r <= G2_BAND.1).count(),
        G2_BAND.0,
        G2_BAND.1
    );
    // The plant: negate E_int and re-run G1's finder over the same points.
    let planted_min = all
        .iter()
        .filter(|d| d.r >= G1_FROM)
        .map(|d| -d.e_int_exact)
        .fold(f64::INFINITY, f64::min);
    let p1_fires = planted_min < -P1_BAR;
    println!(
        "# planted deepest E_int: {:+.6e} Ha (bar -{:.1e})   work: {} points negated",
        planted_min,
        P1_BAR,
        all.iter().filter(|d| d.r >= G1_FROM).count()
    );
    println!("# P1: {}", if p1_fires { "FIRES" } else { "DID NOT FIRE — G1 is VOID" });

    // ------------------------------------------------------------------ P2
    println!("\n# ================================ P2 — the three-body deletion plant (carrier: 3-body channel)");
    println!(
        "# carrier NONZERO IN the three-body channel at G2's worst point: sum dE3 = {:+.6e} Ha",
        g2w.sum_de3
    );
    // The plant presents MBE2 as if it were MBE3. The reported residual must move.
    let planted_r3 = r2;
    let p2_ratio = if r3 > 0.0 { planted_r3 / r3 } else { f64::INFINITY };
    let p2_fires = p2_ratio >= P2_FACTOR || (1.0 / p2_ratio) >= P2_FACTOR;
    println!(
        "# honest MBE3 residual {:.6e}  ->  planted (dE3 deleted) {:.6e}   ratio {:.1}x (bar {:.0}x)",
        r3, planted_r3, p2_ratio, P2_FACTOR
    );
    println!("# P2: {}", if p2_fires { "FIRES" } else { "DID NOT FIRE — G3 is VOID" });

    // ------------------------------------------- the localization clause and the price
    println!("\n# ================================ localization clause");
    let mut worst_de4: &Reading = &all[0];
    for rd in all.iter() {
        if rd.de4.abs() > worst_de4.de4.abs() {
            worst_de4 = rd;
        }
    }
    println!(
        "# the expansion's error is largest at R = {:.2}, {} — dE4 = {:+.6e} Ha, \
         shortest cross distance {:.3} bohr",
        worst_de4.r, worst_de4.orient.label(), worst_de4.de4, worst_de4.r_min_cross
    );
    let bad_total: usize = all.iter().map(|d| d.unconverged_subclusters).sum();
    let quad_bad = all.iter().filter(|d| !d.quad.converged()).count();
    println!(
        "# convergence: {} sub-cluster solves not converged; {} of {} four-atom solves not converged",
        bad_total,
        quad_bad,
        all.len()
    );
    println!("# WORK: {} FCI solves, largest 36 determinants (staked cost model: ~429)", solves + 2);
}
