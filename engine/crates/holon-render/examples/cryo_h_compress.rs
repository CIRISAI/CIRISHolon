//! CRYO-H-O ARM 3 — THE METALLIC-HYDROGEN FENCE. Where does the fragment-local picture
//! stop being able to speak?
//!
//! Frozen by `conformance/atomworld/CRYO_HO_PREREG.md` (2026-09-02, commit fc7b6a0).
//! Gates G9 and G10, plant P2, void conditions V4 and V5.
//!
//! **THIS ARM CLAIMS NO PHASE.** Metallization is electron delocalization across many
//! centres, and this engine's whole picture is fragment-local: energies assembled from
//! clusters of two, three and four atoms, each solved with its own electrons. A metal is
//! the exact breakdown of that picture and this engine cannot exhibit one. What it can do
//! is measure WHERE its own picture fails, in its own units, and name the exit. Any
//! sentence anywhere that reads this file's output as a claim about metallic hydrogen is
//! a defect in that sentence.
//!
//! Every level of the expansion is built from EXACT sub-cluster FCI solves — no table, no
//! interpolant, no fitted surface — so a residual here belongs to the many-body expansion
//! and nothing else.
//!
//! ```text
//! cargo run --release -p holon-render --example cryo_h_compress
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{Species, HYDROGEN};
use holon_chem::pair::{generate_pair_table, solve_geometry};
use holon_render::bank::Host;
use holon_render::sim::{Boundary, Dims, Sim};
use holon_render::{load_pair_table, TABLE_OK};
use std::time::Instant;

// ============================================================= THE FROZEN PROTOCOL

const R_E: f64 = 1.388_694_018_017_776_3;
const N: usize = 8;

/// Nearest-neighbour molecular centre separation, bohr. Staked in the freeze, stepped
/// DOWN so the run reaches the loose end before the basis gives out at the tight end.
const LADDER: [f64; 10] = [8.0, 6.5, 5.5, 4.5, 4.0, 3.5, 3.0, 2.6, 2.2, 1.9];

/// G9's bar, hartree per atom. Below chemical accuracy (1.6e-3 Ha) on purpose: the fence
/// belongs where the model stops being usable, not where it becomes absurd.
const G9_BAR: f64 = 1.0e-3;
/// The centred difference's step, as a fraction of `a`.
const DIFF_H: f64 = 0.02;
/// P2 must move the reported MBE3 residual by this factor at the LOOSEST rung.
const P2_FACTOR: f64 = 10.0;

/// Four H2 molecules on a 2 x 2 planar lattice: centres at (0,0), (a,0), (0,a), (a,a),
/// every molecular axis along x, every bond at the referee's `R_E`. Coplanar in `z = 0`,
/// the campaign's 2D standing fence.
fn scene(a: f64) -> [[f64; 3]; N] {
    let h = 0.5 * R_E;
    let c = [[0.0, 0.0], [a, 0.0], [0.0, a], [a, a]];
    let mut out = [[0.0f64; 3]; N];
    for (m, ctr) in c.iter().enumerate() {
        out[2 * m] = [ctr[0] - h, ctr[1], 0.0];
        out[2 * m + 1] = [ctr[0] + h, ctr[1], 0.0];
    }
    out
}

struct Sol {
    e: f64,
    exit: String,
    converged: bool,
    residual: f64,
    iters: usize,
    margin: Option<f64>,
    scf: bool,
    s_min: f64,
    n_det: usize,
    device: String,
}

/// One FCI solve, with the basis-conditioning refusal (V5) caught rather than allowed to
/// abort the ladder. `cholesky_orthonormaliser` panics on an overlap matrix that has
/// stopped being positive definite, and that refusal is a legitimate terminus of this
/// ladder — a DIFFERENT fence from G9's, and one that must never be presented as G9's.
fn fci(pos: &[[f64; 3]]) -> Option<Sol> {
    let sp: Vec<Species> = vec![HYDROGEN; pos.len()];
    let c: Vec<[D2; 3]> = pos.iter().map(|p| [D2::c(p[0]), D2::c(p[1]), D2::c(p[2])]).collect();
    let r = std::panic::catch_unwind(move || solve_geometry(&sp, c));
    r.ok().map(|s| Sol {
        e: s.e.v,
        exit: format!("{:?}", s.exit),
        converged: s.exit.is_converged() && s.scf_converged,
        residual: s.residual,
        iters: s.davidson_iters,
        margin: None,
        scf: s.scf_converged,
        s_min: s.s_min_eigenvalue,
        n_det: s.n_det,
        device: format!("{:?}", s.device),
    })
}

struct Rung {
    a: f64,
    /// Number density, atoms per bohr^2, over the 4-molecule cell `A = 4 a^2`.
    n_dens: f64,
    e_exact: f64,
    e_mbe2: f64,
    e_mbe3: f64,
    e_mbe4: f64,
    /// `|E_exact - E_MBEk| / N`, hartree per atom.
    r2: f64,
    r3: f64,
    r4: f64,
    sum_de3: f64,
    sum_de4: f64,
    /// The model's own 2D pressure, `-dE/dA`, Ha/bohr^2.
    p2d: f64,
    exit: String,
    residual: f64,
    iters: usize,
    scf: bool,
    s_min: f64,
    sub_unconverged: usize,
    /// The sub-cluster non-convergence broken out BY REASON. A bare count makes
    /// iteration-cap, subspace stagnation and an unconverged SCF indistinguishable, and
    /// they are three different facts: only the first is a budget problem, and the third
    /// does not touch the energy at all because a FULL CI is invariant under any unitary
    /// rotation of its orbitals.
    sub_scf_only: usize,
    sub_stagnated: usize,
    sub_itercap: usize,
    /// The localization clause: which sub-cluster class carries the largest single term.
    worst_class: &'static str,
    worst_term: f64,
    worst_sep: f64,
}

fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    (dx * dx + dy * dy).sqrt()
}

/// One rung, all four levels of the expansion from exact sub-cluster solves.
fn ladder_rung(a: f64, e_h: f64) -> Option<Rung> {
    let pos = scene(a);
    let ex = fci(&pos)?;
    let mut bad = 0usize;
    let (mut scf_only, mut stag, mut cap) = (0usize, 0usize, 0usize);
    let mut tally = |s: &Sol| {
        if s.converged {
            return;
        }
        if s.exit == "Stagnated" {
            stag += 1;
        } else if s.exit == "IterationCap" {
            cap += 1;
        } else if !s.scf {
            scf_only += 1;
        }
    };

    let mut v2 = [[0.0f64; N]; N];
    let mut sum_v2 = 0.0f64;
    let (mut worst_class, mut worst_term, mut worst_sep) = ("pair", 0.0f64, 0.0f64);
    for i in 0..N {
        for j in (i + 1)..N {
            let s = fci(&[pos[i], pos[j]])?;
            if !s.converged {
                bad += 1;
            }
            tally(&s);
            let e = s.e - 2.0 * e_h;
            v2[i][j] = e;
            v2[j][i] = e;
            sum_v2 += e;
            if e.abs() > worst_term.abs() {
                worst_class = "pair";
                worst_term = e;
                worst_sep = dist(&pos[i], &pos[j]);
            }
        }
    }

    let mut de3 = vec![0.0f64; 0];
    let mut idx3: Vec<(usize, usize, usize)> = Vec::new();
    let mut sum_de3 = 0.0f64;
    for i in 0..N {
        for j in (i + 1)..N {
            for k in (j + 1)..N {
                let s = fci(&[pos[i], pos[j], pos[k]])?;
                if !s.converged {
                    bad += 1;
                }
                tally(&s);
                let t = s.e - 3.0 * e_h - v2[i][j] - v2[i][k] - v2[j][k];
                de3.push(t);
                idx3.push((i, j, k));
                sum_de3 += t;
                if t.abs() > worst_term.abs() {
                    worst_class = "triple";
                    worst_term = t;
                    worst_sep = dist(&pos[i], &pos[j])
                        .max(dist(&pos[i], &pos[k]))
                        .max(dist(&pos[j], &pos[k]));
                }
            }
        }
    }
    let de3_of = |i: usize, j: usize, k: usize| -> f64 {
        let p = idx3.iter().position(|&t| t == (i, j, k)).expect("every triple is enumerated");
        de3[p]
    };

    let mut sum_de4 = 0.0f64;
    for i in 0..N {
        for j in (i + 1)..N {
            for k in (j + 1)..N {
                for l in (k + 1)..N {
                    let s = fci(&[pos[i], pos[j], pos[k], pos[l]])?;
                    if !s.converged {
                        bad += 1;
                    }
                    tally(&s);
                    let pairs = v2[i][j] + v2[i][k] + v2[i][l] + v2[j][k] + v2[j][l] + v2[k][l];
                    let trips = de3_of(i, j, k) + de3_of(i, j, l) + de3_of(i, k, l) + de3_of(j, k, l);
                    let q = s.e - 4.0 * e_h - pairs - trips;
                    sum_de4 += q;
                    if q.abs() > worst_term.abs() {
                        worst_class = "quadruple";
                        worst_term = q;
                        worst_sep = dist(&pos[i], &pos[l]);
                    }
                }
            }
        }
    }

    let e_mbe2 = N as f64 * e_h + sum_v2;
    let e_mbe3 = e_mbe2 + sum_de3;
    let e_mbe4 = e_mbe3 + sum_de4;

    // The model's own 2D pressure by centred difference on the cell area `A = 4 a^2`.
    // A cluster analogue of a pressure, not a bulk one, and labelled as such.
    let h = DIFF_H * a;
    let (ep, em) = (fci(&scene(a + h))?.e, fci(&scene(a - h))?.e);
    let de_da = (ep - em) / (2.0 * h);
    let da_da = 8.0 * a; // A = 4 a^2

    Some(Rung {
        a,
        n_dens: N as f64 / (4.0 * a * a),
        e_exact: ex.e,
        e_mbe2,
        e_mbe3,
        e_mbe4,
        r2: (ex.e - e_mbe2).abs() / N as f64,
        r3: (ex.e - e_mbe3).abs() / N as f64,
        r4: (ex.e - e_mbe4).abs() / N as f64,
        sum_de3,
        sum_de4,
        p2d: -de_da / da_da,
        exit: ex.exit.clone(),
        residual: ex.residual,
        iters: ex.iters,
        scf: ex.scf,
        s_min: ex.s_min,
        sub_unconverged: bad,
        sub_scf_only: scf_only,
        sub_stagnated: stag,
        sub_itercap: cap,
        worst_class,
        worst_term,
        worst_sep,
    })
}

fn main() {
    // The panic hook is silenced only around the caught solves; V5's refusal is a
    // legitimate terminus and its backtrace is noise, not information.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    println!("# CRYO-H-O ARM 3 — the metallic-hydrogen FENCE. NO PHASE IS CLAIMED.");
    println!("# prereg conformance/atomworld/CRYO_HO_PREREG.md, frozen fc7b6a0");
    println!("# STANDING FENCES: 2D scene | classical nuclei (no NQE) | STO-3G minimal basis");
    println!("# scene: 4 H2 on a 2x2 planar lattice, bond frozen at {R_E:.10} bohr, 8 atoms");

    let e_h = fci(&[[0.0, 0.0, 0.0]]).expect("the atom solves").e;
    println!("# E(H) = {e_h:.12} Ha");
    println!("#");
    println!(
        "#    a    n(bohr^-2)   E_exact        |dR2|/at    |dR3|/at    |dR4|/at   \
         sum dE3     sum dE4     P_2D(Ha/b^2)  exit       resid    s_min    bad"
    );

    let t0 = Instant::now();
    let mut rungs: Vec<Rung> = Vec::new();
    let mut v5_at: Option<f64> = None;
    for &a in LADDER.iter() {
        match ladder_rung(a, e_h) {
            None => {
                println!(
                    "  {a:5.2}   V5 REFUSED — the overlap matrix has stopped being positive \
                     definite; the BASIS's own limit, a different fence from G9's"
                );
                v5_at = Some(a);
                break;
            }
            Some(r) => {
                println!(
                    "  {:5.2}  {:9.5}  {:+.8}  {:.4e}  {:.4e}  {:.4e}  {:+.3e}  {:+.3e}  \
                     {:+.5e}  {:<10} {:.1e}  {:.1e}  {}",
                    r.a, r.n_dens, r.e_exact, r.r2, r.r3, r.r4, r.sum_de3, r.sum_de4,
                    r.p2d, r.exit, r.residual, r.s_min, r.sub_unconverged
                );
                rungs.push(r);
            }
        }
    }
    std::panic::set_hook(default_hook);
    let secs = t0.elapsed().as_secs_f64();

    // ------------------------------------------------------------------ V4
    println!("\n# ================================ V4 — solver-exit voids");
    let void4: Vec<&Rung> = rungs.iter().filter(|r| r.exit != "Converged" || !r.scf).collect();
    println!(
        "# {} of {} rungs VOID for exit or SCF: {:?}",
        void4.len(),
        rungs.len(),
        void4.iter().map(|r| (r.a, r.exit.clone(), r.scf)).collect::<Vec<_>>()
    );
    println!(
        "# sub-cluster solves not converged, summed over the ladder: {} \
         (SCF-only {}, Stagnated {}, IterationCap {})",
        rungs.iter().map(|r| r.sub_unconverged).sum::<usize>(),
        rungs.iter().map(|r| r.sub_scf_only).sum::<usize>(),
        rungs.iter().map(|r| r.sub_stagnated).sum::<usize>(),
        rungs.iter().map(|r| r.sub_itercap).sum::<usize>()
    );
    for r in rungs.iter().filter(|r| r.sub_unconverged > 0) {
        println!(
            "#   a {:5.2}: {} of 154 sub-clusters — SCF-only {}, Stagnated {}, IterationCap {}",
            r.a, r.sub_unconverged, r.sub_scf_only, r.sub_stagnated, r.sub_itercap
        );
    }
    println!(
        "# An SCF-only non-convergence does NOT move the energy: a full CI is invariant \
         under any unitary rotation of its orbitals, so the SCF here only chooses the basis \
         the Davidson runs in. IterationCap and Stagnated are different facts and are counted \
         separately for exactly that reason."
    );
    let live: Vec<&Rung> = rungs.iter().filter(|r| r.exit == "Converged" && r.scf).collect();

    // ------------------------------------------------------------------ V5
    println!("\n# ================================ V5 — basis linear dependence");
    match v5_at {
        Some(a) => println!(
            "# the ladder terminated at a = {a} bohr on the overlap matrix, NOT on G9. \
             The last live rung's smallest overlap eigenvalue was {:.3e}.",
            live.last().map(|r| r.s_min).unwrap_or(f64::NAN)
        ),
        None => println!("# the ladder ran to its end; the basis never refused"),
    }

    // ------------------------------------------------------------------ G9
    println!("\n# ================================ G9 — does the expansion's error grow, and where does it cross?");
    println!("# staked (i)   MBE3 error < {G9_BAR:.1e} Ha/atom at the loosest rung a = {}", LADDER[0]);
    println!("# staked (ii)  that error is monotone non-decreasing as a falls");
    println!("# staked (iii) it crosses {G9_BAR:.1e} Ha/atom somewhere on this ladder");
    let i_ok = live.first().map(|r| r.r3 < G9_BAR).unwrap_or(false);
    println!(
        "# (i):   MBE3 error at a = {:.2} is {:.4e} Ha/atom -> {}",
        live.first().map(|r| r.a).unwrap_or(f64::NAN),
        live.first().map(|r| r.r3).unwrap_or(f64::NAN),
        if i_ok { "HOLDS" } else { "KILLED — no baseline" }
    );
    let mut ii_ok = true;
    for w in live.windows(2) {
        if w[1].r3 < w[0].r3 {
            println!(
                "#   BREACH: MBE3 error FALLS {:.4e} -> {:.4e} between a = {:.2} and a = {:.2}",
                w[0].r3, w[1].r3, w[0].a, w[1].a
            );
            ii_ok = false;
        }
    }
    println!(
        "# (ii):  {}",
        if ii_ok { "HOLDS — monotone non-decreasing under compression" } else { "KILLED — not monotone" }
    );
    let cross = live.iter().find(|r| r.r3 >= G9_BAR);
    match cross {
        Some(r) => {
            println!(
                "# (iii): HOLDS — THE FENCE IS AT a = {:.2} bohr, n = {:.5} atoms/bohr^2, \
                 P_2D = {:+.5e} Ha/bohr^2   (MBE3 error {:.4e} Ha/atom)",
                r.a, r.n_dens, r.p2d, r.r3
            );
        }
        None => println!(
            "# (iii): KILLED — the expansion survives the whole ladder. The fence is BEYOND \
             a = {:.2} bohr, reported as a BOUND and not as an absence.",
            live.last().map(|r| r.a).unwrap_or(f64::NAN)
        ),
    }

    // ------------------------------------------------------------------ G10
    println!("\n# ================================ G10 — the ladder's own non-convergence signature");
    println!("# converging at a rung iff |dR4| < |dR3| < |dR2|. No band is staked on where it breaks.");
    let mut broke: Option<&&Rung> = None;
    for r in live.iter() {
        let ok = r.r4 < r.r3 && r.r3 < r.r2;
        println!(
            "#   a {:5.2}:  |dR2| {:.4e}  |dR3| {:.4e}  |dR4| {:.4e}   {}",
            r.a, r.r2, r.r3, r.r4,
            if ok { "converging" } else { "NOT converging" }
        );
        if !ok && broke.is_none() {
            broke = Some(r);
        }
    }
    match (broke, cross) {
        (Some(b), Some(c)) => println!(
            "# G10: the chain first breaks at a = {:.2}; G9's crossing is at a = {:.2}. {}",
            b.a, c.a,
            if (b.a - c.a).abs() < 1e-9 { "They agree." } else { "THEY DISAGREE — and the disagreement is the finding." }
        ),
        (Some(b), None) => println!("# G10: the chain first breaks at a = {:.2}; G9 never crossed.", b.a),
        (None, Some(c)) => println!(
            "# G10: the chain never breaks on this ladder, though G9 crossed at a = {:.2}. \
             THEY DISAGREE — and the disagreement is the finding.",
            c.a
        ),
        (None, None) => println!("# G10: the chain never breaks and G9 never crossed."),
    }

    // ------------------------------------------------------------------ P2
    println!("\n# ================================ P2 — the three-body deletion plant (carrier: 3-body channel)");
    match live.first() {
        None => println!("# P2: NOT RUN — no live rung"),
        Some(r) => {
            println!(
                "# carrier NONZERO IN the three-body channel at the loosest rung a = {:.2}: \
                 sum dE3 = {:+.6e} Ha",
                r.a, r.sum_de3
            );
            // The plant presents E_MBE2 as E_MBE3. The reported residual becomes |dR2|.
            let ratio = if r.r3 > 0.0 { r.r2 / r.r3 } else { f64::INFINITY };
            let fires = ratio >= P2_FACTOR;
            println!(
                "# honest MBE3 residual {:.6e} Ha/atom  ->  planted (dE3 deleted) {:.6e}   \
                 ratio {:.2}x (bar {:.0}x)",
                r.r3, r.r2, ratio, P2_FACTOR
            );
            println!(
                "# P2: {}",
                if fires { "FIRES" } else { "DID NOT FIRE — G9 is VOID per V6" }
            );
        }
    }

    // ---------------------------------------------- the engine's own pressure readout
    println!("\n# ================================ THE UNIT FENCE — the engine's virial pressure");
    println!("# The freeze demands density be mapped to pressure through the engine's own");
    println!("# virial readout, and both quoted. This section measures whether the engine HAS one.");
    // THE TABLE MUST BE LOADED FIRST. Read on an empty `Sim` this section returns
    // `list_cutoff = 0.000` and `pbc_ok = true` at every density — a pass reported for a
    // scene with no interactions in it, which is M-VACUOUS-SUCCESS exactly. The first run
    // of this file did that.
    //
    // Loading it did NOT change the reading, and that is the finding rather than the fix.
    // `Sim::list_cutoff` is `max(three_body_cutoff, four_body_cutoff, far.r_s,
    // pair_switch.r_cut)` — the PAIR TABLE's own support is not among them, because an
    // undeclared pair sector is a COMPLETE sum with no cutoff to report. So on a scene
    // with no three-body surface, no far sector and no declared truncation window,
    // `list_cutoff()` is exactly zero and `Sim::pbc_ok` — which is
    // `list_cutoff() <= half the shortest edge` — is true for EVERY box size, including
    // boxes far smaller than the pair curve's support. The row below prints the pair
    // table's `r_max` beside the half-edge so the comparison the guard does not make is
    // visible.
    let pt = generate_pair_table(HYDROGEN, HYDROGEN, 96);
    println!(
        "#   the H-H pair table's own support: r_min {:.3} to r_max {:.3} bohr",
        pt.meta.r_min, pt.meta.r_max
    );
    for &a in [LADDER[0], 4.5, LADDER[LADDER.len() - 1]].iter() {
        let mut s = Box::new(Sim::empty());
        s.dims = Dims::Two;
        s.width = 2.0 * a;
        s.height = 2.0 * a;
        s.boundary = Boundary::Walls;
        assert_eq!(load_pair_table(&mut s, &pt, Host::Native), TABLE_OK);
        let pos = scene(a);
        s.reset(N);
        for i in 0..N {
            assert!(s.set_species(i, HYDROGEN));
            s.set_position(i, pos[i][0], pos[i][1]);
            s.set_velocity(i, 0.0, 0.0);
        }
        s.rebase();
        s.recompute();
        let walls_defined = s.pressure_defined();
        let p_walls = s.pressure();
        s.boundary = Boundary::Periodic;
        s.recompute();
        let (cut, half_edge) = s.pbc_margin();
        let ok = s.pbc_ok();
        println!(
            "#   a {:5.2}: box {:.2} x {:.2} x depth {:.1} | WALLS pressure_defined = {} \
             (the number it would print: {:+.4e} Ha/bohr^3) | PERIODIC list_cutoff {:.3} vs \
             half-edge {:.3} -> pbc_ok = {}{}",
            a, s.width, s.height, s.depth, walls_defined, p_walls, cut, half_edge, ok,
            if ok {
                format!(
                    "  -> P = {:+.4e} Ha/bohr^3 = {:+.4e} Pa   [table r_max {:.2} vs half-edge \
                     {:.2}: the guard does {} the comparison that matters]",
                    s.pressure(),
                    s.pressure() * holon_render::barostat::AU_PRESSURE_PA,
                    pt.meta.r_max,
                    half_edge,
                    if pt.meta.r_max <= half_edge { "pass" } else { "NOT make" }
                )
            } else {
                "  -> REFUSED".to_string()
            }
        );
    }
    println!("# UNIT HONESTY, stated in the freeze and confirmed here: Sim::pressure computes");
    println!("#   (2K - virial) / 3V with V = width*height*depth and depth defaulting to 24 bohr");
    println!("#   on a 2D scene. So the engine's pascal number for a 2D scene is a THREE-");
    println!("#   dimensional pressure on a slab of assumed thickness, with the 3D virial");
    println!("#   factor 3 where a 2D scene wants 2. P_2D above is the primary reading and is");
    println!("#   quoted in Ha/bohr^2. No GPa comparison is performed: it would be a number");
    println!("#   with an invented thickness in it.");

    // ------------------------------------------------- localization clause and the price
    println!("\n# ================================ localization clause");
    for r in live.iter() {
        println!(
            "#   a {:5.2}: largest single expansion term is a {} worth {:+.4e} Ha at a \
             separation of {:.3} bohr",
            r.a, r.worst_class, r.worst_term, r.worst_sep
        );
    }
    println!("\n# THE EXIT, as the fence law requires: past this fence the engine stops being");
    println!("# able to speak, and the exit is DELOCALIZED / PERIODIC ELECTRONIC STRUCTURE —");
    println!("# a band or plane-wave solver with k-point sampling, a different solver class,");
    println!("# out of scope for this campaign and for this crate.");
    println!(
        "\n# WORK: {} rungs x (1 exact + 28 pair + 56 triple + 70 quad + 2 difference) = {} \
         FCI solves, largest 4900 determinants. [{:.0} s of a 60-load box, NOT a cost]",
        rungs.len(),
        rungs.len() * 157,
        secs
    );
}
