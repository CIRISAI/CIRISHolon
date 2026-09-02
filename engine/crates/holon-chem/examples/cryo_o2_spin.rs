//! CRYO-H-O ARM 2 — THE DISCLOSURE GATE. Which spin sector did the banked O–O curve
//! actually solve?
//!
//! Frozen by `conformance/atomworld/CRYO_HO_PREREG.md` (2026-09-02, commit fc7b6a0).
//! Gates G5 and G6, plant P3.
//!
//! Physical O2 is a triplet ground state and liquid O2 is paramagnetic. This engine
//! solves every even-electron system in `sz2_sector(16) = 0`, and `elements.rs` justifies
//! that with an argument rather than a measurement: a multiplet of total spin S has a
//! component in every sector with `|S_z| <= S`, so the minimal sector contains every
//! state and the solve cannot miss the ground state whatever its spin. **That is a claim,
//! and this file measures it instead of repeating it.**
//!
//! Every number here carries the same inherited disclosure the banked curve does: this
//! solve is 2025 determinants in 10 basis functions under a 5000-iteration budget, which
//! is where `IterationCap` came from in `B1B_RESULTS.md`.
//!
//! ```text
//! cargo run --release -p holon-chem --example cryo_o2_spin
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{OXYGEN, HYDROGEN};
use holon_chem::fci::{multiplicity, s_squared, solve_determinant};
use holon_chem::pair::{build_basis, geometry_problem};

/// The banked O–O curve's equilibrium, `p2_de4_full/*.log`.
const R_OO: f64 = 2.4421;
/// The banked H–H curve's equilibrium, for the control that a KNOWN closed-shell case
/// reads S = 0 through the same instrument.
const R_HH: f64 = 1.388_694_018_017_776_3;
/// G6's bar on the quintet gap, twenty times the inherited 4.809e-6 residual.
const G6_QUINTET_BAR: f64 = 1.0e-4;
/// G6's bar on the M_s degeneracy of one multiplet.
const G6_DEGEN_BAR: f64 = 1.0e-6;
/// `multiplicity`'s tolerance on reading S back from <S^2>.
const MULT_TOL: f64 = 1.0e-6;

struct Sector {
    n_alpha: usize,
    n_beta: usize,
    sz: f64,
    e: f64,
    s_sq: f64,
    mult: Option<(f64, usize)>,
    exit: String,
    residual: f64,
    iters: usize,
    n_det: usize,
    margin: Option<f64>,
    device: String,
}

/// One geometry in one EXPLICIT `(n_alpha, n_beta)` sector. `solve_geometry` would pick
/// the sector for us from `sz2_sector`; this goes around it deliberately, because the
/// whole question is what that choice does.
fn solve_in(z: &[holon_chem::elements::Species], pos: &[[f64; 3]], na: usize, nb: usize) -> Sector {
    let centers: Vec<[D2; 3]> = pos
        .iter()
        .map(|p| [D2::c(p[0]), D2::c(p[1]), D2::c(p[2])])
        .collect();
    // `geometry_problem` builds the space from `electron_counts`, which imposes the
    // minimal sector. Rebuild the space at the requested partition from the same
    // integrals so the two differ ONLY in the sector.
    let (_, mo, e_nuc) = geometry_problem(z, centers.clone());
    let basis = build_basis(z, centers);
    let space = holon_chem::fci::FciSpace::new(basis.n, na, nb);
    let sol = solve_determinant(&space, &mo);
    let s_sq = s_squared(&space, &sol.vector);
    Sector {
        n_alpha: na,
        n_beta: nb,
        sz: 0.5 * (na as f64 - nb as f64),
        e: sol.e.v + e_nuc.v,
        s_sq,
        mult: multiplicity(s_sq, na + nb, MULT_TOL),
        exit: format!("{:?}", sol.exit),
        residual: sol.residual,
        iters: sol.davidson_iters,
        n_det: space.n_det,
        margin: sol.variational_margin,
        device: format!("{:?}", sol.device),
    }
}

fn show(tag: &str, s: &Sector) {
    println!(
        "  {tag:<26} n_a {:>2} n_b {:>2}  S_z {:+.1}  E {:+.9} Ha  <S^2> {:.6}  \
         mult {:<12}  exit {:<11} resid {:.2e}  iters {:>4}  n_det {:>6}  var_margin {}  device {}",
        s.n_alpha,
        s.n_beta,
        s.sz,
        s.e,
        s.s_sq,
        match s.mult {
            Some((spin, m)) => format!("S={spin:.1} 2S+1={m}"),
            None => "UNRESOLVED".to_string(),
        },
        s.exit,
        s.residual,
        s.iters,
        s.n_det,
        match s.margin {
            Some(m) => format!("{m:+.3e}"),
            None => "none".to_string(),
        },
        s.device
    );
}

fn main() {
    println!("# CRYO-H-O ARM 2 — the spin-sector disclosure gate");
    println!("# prereg conformance/atomworld/CRYO_HO_PREREG.md, frozen fc7b6a0");
    println!("# STANDING FENCES: 2D scene | classical nuclei (no NQE) | STO-3G minimal basis");
    println!("# INHERITED DISCLOSURE: the banked O-O curve exits IterationCap at budget 5000,");
    println!("#   worst_residual 4.809e-6 Ha, n_det 2025, n_basis 10 (B1B_RESULTS.md W1).");
    println!("# The engine's rule under test: sz2_sector(n) = n % 2, elements.rs:2649.");

    let o2 = [[0.0, 0.0, 0.0], [0.0, 0.0, R_OO]];
    let sp = [OXYGEN, OXYGEN];

    println!("\n# ---- O2 at the banked R_e = {R_OO} bohr, three S_z sectors ----");
    let s0 = solve_in(&sp, &o2, 8, 8);
    let s1 = solve_in(&sp, &o2, 9, 7);
    let s2 = solve_in(&sp, &o2, 10, 6);
    show("S_z = 0  (the engine's)", &s0);
    show("S_z = 1", &s1);
    show("S_z = 2  (P3's plant)", &s2);

    // ------------------------------------------------------------------ G5
    println!("\n# ================================ G5 — which sector did the banked curve return?");
    println!("# staked: <S^2> = 2.000 +/- 0.01, multiplicity 3 (a triplet)");
    println!("# KILL:   <S^2> = 0 (multiplicity 1) -> a named MODEL FENCE on every ARM 2 number");
    println!("# third pre-committed branch: neither, reported UNRESOLVED with the value printed");
    let g5 = if (s0.s_sq - 2.0).abs() <= 0.01 && s0.mult.map(|m| m.1) == Some(3) {
        "HOLDS — the banked O-O curve IS a triplet curve; no paramagnetic fence is owed"
    } else if s0.s_sq.abs() <= 0.01 && s0.mult.map(|m| m.1) == Some(1) {
        "KILLED — the banked O-O curve is an S=0 curve; MODEL FENCE owed on every ARM 2 number"
    } else {
        "UNRESOLVED — the third branch; ARM 2 carries the weaker fence"
    };
    println!("# measured <S^2> = {:.9}, multiplicity {:?}", s0.s_sq, s0.mult);
    println!("# G5: {g5}");

    // ------------------------------------------------------------------ G6
    println!("\n# ================================ G6 — is the sector plumbing two-sided?");
    let quintet_gap = s2.e - s0.e;
    let ms_gap = (s1.e - s0.e).abs();
    println!(
        "# staked (a): E(S_z=2) - E(S_z=0) > {:.1e} Ha    measured {:+.6e}",
        G6_QUINTET_BAR, quintet_gap
    );
    let g6a = quintet_gap > G6_QUINTET_BAR;
    let is_triplet = s0.mult.map(|m| m.1) == Some(3);
    println!(
        "# staked (b), IF G5 returns a triplet: |E(S_z=1) - E(S_z=0)| < {:.1e} Ha    measured {:.6e}  [{}]",
        G6_DEGEN_BAR,
        ms_gap,
        if is_triplet { "in force" } else { "not in force — G5 did not return a triplet" }
    );
    let g6b = !is_triplet || ms_gap < G6_DEGEN_BAR;
    println!(
        "# G6: {}",
        if g6a && g6b {
            "HOLDS — the sector machinery does what the code says"
        } else {
            "KILLED — every energy that came through this seam is in question"
        }
    );

    // ------------------------------------------------------------------ P3
    println!("\n# ================================ P3 — the spin-sector plant (carrier: S_z = 2 block)");
    println!(
        "# carrier NONZERO IN the S_z = 2 sector: {} determinants, and it holds no M_s = 0 \
         component at all, so the plant's space is disjoint from the honest solve's by construction",
        s2.n_det
    );
    println!(
        "# planted E(S_z=2) sits {:+.6e} Ha above the honest E(S_z=0) (bar {:.1e})",
        quintet_gap, G6_QUINTET_BAR
    );
    println!(
        "# P3: {}",
        if g6a { "FIRES" } else { "DID NOT FIRE — G5 and G6 are VOID" }
    );

    // -------------------------------------------- the closed-shell control, same instrument
    println!("\n# ---- CONTROL: H2 at R_e, a case whose answer is known independently ----");
    println!("# The same instrument on a molecule that must read S = 0. A gate that only ever");
    println!("# returns 'triplet' would pass G5 on an O2 that was not one.");
    let hh = [[0.0, 0.0, 0.0], [0.0, 0.0, R_HH]];
    let h0 = solve_in(&[HYDROGEN, HYDROGEN], &hh, 1, 1);
    let h1 = solve_in(&[HYDROGEN, HYDROGEN], &hh, 2, 0);
    show("H2  S_z = 0", &h0);
    show("H2  S_z = 1", &h1);
    println!(
        "# control: H2 reads <S^2> = {:.6} (multiplicity {:?}); the instrument distinguishes \
         the two molecules",
        h0.s_sq, h0.mult
    );

    // ------------------------------------------------ the scope G5 does NOT cover, measured
    //
    // POST-DATA scope extension. G5 is staked at R_e and holds there, but the banked curve
    // is 96 knots over a RANGE and the multiplicity is a property of a geometry, not of a
    // molecule. At dissociation two ground-state oxygens are each S = 1, and S = 0, 1 and 2
    // are degenerate there — so a curve that is a triplet at R_e need not be one everywhere,
    // and a gate that measured one point and spoke about a curve would be overreaching.
    println!("\n# ---- POST-DATA: <S^2> along the curve, the scope G5's single point does not cover ----");
    println!("#     R      E(S_z=0)        <S^2>     mult        E(S_z=1)-E(S_z=0)   exit         resid");
    for r in [1.8f64, 2.2, R_OO, 2.8, 3.5, 4.5, 6.0, 8.0] {
        let a = solve_in(&sp, &[[0.0, 0.0, 0.0], [0.0, 0.0, r]], 8, 8);
        let b = solve_in(&sp, &[[0.0, 0.0, 0.0], [0.0, 0.0, r]], 9, 7);
        println!(
            "  {:6.3}  {:+.9}  {:8.6}  {:<11} {:+.6e}       {:<11}  {:.1e}",
            r,
            a.e,
            a.s_sq,
            match a.mult {
                Some((s, m)) => format!("S={s:.1} 2S+1={m}"),
                None => "UNRESOLVED".to_string(),
            },
            b.e - a.e,
            a.exit,
            a.residual
        );
    }
    println!(
        "# READ: G5's verdict is about R_e = {R_OO} bohr, which is where the banked curve's well is. \
         Any row above whose multiplicity differs is a place the curve is a different spin state, \
         and is scope G5 did not claim."
    );

    println!("\n# WORK: 5 determinant solves, largest {} determinants.", s0.n_det.max(s2.n_det));
}
