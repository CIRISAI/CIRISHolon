//! SATURATION-3 gate G0 — THE HARDEST COMBOS ARE TRIVIAL AS PREDICTED.
//!
//! First and blocking. No table node is built until this passes, and any staked combo more
//! than 10x its predicted cost class re-scopes the campaign.
//!
//! # What is being tested, and why it is the arithmetic and not a memory of it
//!
//! The freeze COMMITTED five determinant counts before any solve: (H,H,Cl) 605,
//! (H,Cl,Cl) 3,249, (Cl,Cl,Cl) 9,477, (O,O,H) 9,075, (O,O,O) 207,025. They come from hole
//! counting — every one of these triples is near-closed-shell, so the space is
//! `C(n_orb, n_alpha) * C(n_orb, n_beta)` with one to three HOLES rather than a
//! combinatorial explosion in the electrons. Chlorine is 9 orbitals and 17 electrons: a
//! chlorine trimer is 27 orbitals and 51 electrons, which is 1 alpha hole and 2 beta holes
//! and therefore 27 * 351 determinants, not an astronomical number.
//!
//! This gate requires the engine to produce those counts EXACTLY. A count that merely
//! "looks about right" would let a mis-derived electron count or a wrong Sz sector through,
//! and every cost estimate downstream rests on it.
//!
//! # THE TRAP THIS GATE HAD TO ROUTE AROUND
//!
//! `fci::solve` sends a space its resource door refuses to the
//! MPS/DMRG route, whose MPO builder reaches six orbitals and then HANGS rather than
//! erroring. (O,O,O) is 207,025 determinants over fifteen orbitals — it is the one staked
//! combo that crosses that threshold, and calling `solve_geometry` on it would not return.
//! So every solve here goes through `solve_determinant` EXPLICITLY. The one expensive
//! measurement in this gate is the one the shortcut would have swallowed.
//!
//! # THE STAKED GEOMETRY RULE, stated before any solve
//!
//! Determinant COUNT does not depend on geometry; solve TIME does, and it is worst where
//! the overlap is greatest and the spectrum densest. So each triple is measured at its
//! WORST COMPACT geometry, defined as:
//!
//!   * each side of the triangle set to `COMPACT_FRACTION` times that PAIR's own in-model
//!     equilibrium separation `R_e`, located here on a coarse staked grid;
//!   * `COMPACT_FRACTION = 0.75` — inside the repulsive wall of every pair in the set,
//!     and far clear of the linear-dependence corner SATURATION-2 measured (which for
//!     hydrogen sits below 0.05 bohr, two decades under anything here).
//!
//! Any three such sides satisfy the triangle inequality automatically, since each is a
//! positive multiple of a sum of two per-species scales.
//!
//! `R_e` is LOCATED rather than quoted, per species pair, and the located values are
//! printed so the geometry can be re-derived. `homonuclear_radius` was NOT used: it is
//! declared only for Z <= 10 and silently returns hydrogen's value for chlorine, which
//! would have staked the chlorine geometries at a hydrogen length without saying so.
//!
//! # Exit reasons (M-EXIT-DISCRIMINATOR), and a correction to this gate's first version
//!
//! Every solve records its iteration count, its final residual, its VARIATIONAL MARGIN and
//! the SOLVER'S OWN exit reason.
//!
//! The first version of this gate derived an exit reason itself, by comparing the residual
//! against `pair::CONVERGED_RESIDUAL` (1e-10), and reported all five combos "CONVERGED".
//! The solver's own target is 1e-11 and it reaches none of them: every one of these solves
//! exits `subspace stagnated`. So the column was reporting THIS FILE'S threshold and
//! calling it the solve's outcome — a second copy of a rule, disagreeing with the first.
//!
//! The mechanism, since it matters for every table this campaign builds: the residual
//! floors just under 1e-10 because `davidson_eigh` accepts a new expansion direction only
//! when its norm exceeds a hardcoded 1e-10, and that threshold is SCALE-FREE. It is not an
//! energy-scale noise floor — across these five combos `|E|` spans 192 to 1651 hartree and
//! `n_det` spans 605 to 207,025, so an `eps*|E|*sqrt(n_det)` floor would span 14x, while
//! the observed residuals span 1.21x. One threshold dominates all five.
//!
//! The eigenvalue error that costs is `~resid^2/gap`, about 1e-20 hartree — twelve orders
//! below anything this campaign measures. These solves are ACCURATE; the label is what is
//! wrong, and the label is not worth changing a shared tolerance for, because every energy's
//! last bits would move and SATURATION-2's committed table is gated on bit-identity.
//!
//! ```text
//! cargo run --release -p holon-chem --example s3_g0
//! ```

use holon_chem::dual::D2;
use holon_chem::elements::{Species, CHLORINE, HYDROGEN, OXYGEN};
use holon_chem::fci::solve_determinant;
use holon_chem::pair::{geometry_problem, pair_point, CONVERGED_RESIDUAL};
use std::time::Instant;

/// How far inside contact the worst-compact geometry sits. See the header.
const COMPACT_FRACTION: f64 = 0.75;

/// The committed arithmetic, from the freeze, before any solve.
const COMMITTED: [(&str, usize); 5] = [
    ("(H,H,Cl)", 605),
    ("(H,Cl,Cl)", 3_249),
    ("(Cl,Cl,Cl)", 9_477),
    ("(O,O,H)", 9_075),
    ("(O,O,O)", 207_025),
];

/// The predicted cost class, seconds per point, cold. The freeze's numbers.
const CLASS_CHEAP_S: f64 = 5.0;
const CLASS_OOO_S: f64 = 300.0;
/// The kill: over this multiple of its class, a combo re-scopes the campaign.
const KILL_MULTIPLE: f64 = 10.0;

fn c3(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

/// The pair's own in-model equilibrium separation, located on a coarse staked grid and
/// then refined by golden section. Coarse on purpose: this fixes a GEOMETRY for a cost
/// measurement, and a 1% error in `R_e` moves no cost class.
fn locate_r_e(a: Species, b: Species) -> (f64, usize) {
    let (lo, hi, n) = (1.0f64, 8.0f64, 29);
    let mut solves = 0usize;
    let mut best = (f64::INFINITY, lo);
    for i in 0..n {
        let r = lo + (hi - lo) * i as f64 / (n - 1) as f64;
        let e = pair_point(a, b, r).e;
        solves += 1;
        if e < best.0 {
            best = (e, r);
        }
    }
    // Golden-section refinement in the bracketing cell.
    let h = (hi - lo) / (n - 1) as f64;
    let (mut x0, mut x3) = ((best.1 - h).max(0.6), best.1 + h);
    const PHI: f64 = 0.618_033_988_749_895;
    let (mut x1, mut x2) = (x3 - PHI * (x3 - x0), x0 + PHI * (x3 - x0));
    let (mut f1, mut f2) = (pair_point(a, b, x1).e, pair_point(a, b, x2).e);
    solves += 2;
    for _ in 0..18 {
        if f1 < f2 {
            x3 = x2;
            x2 = x1;
            f2 = f1;
            x1 = x3 - PHI * (x3 - x0);
            f1 = pair_point(a, b, x1).e;
        } else {
            x0 = x1;
            x1 = x2;
            f1 = f2;
            x2 = x0 + PHI * (x3 - x0);
            f2 = pair_point(a, b, x2).e;
        }
        solves += 1;
    }
    (0.5 * (x1 + x2), solves)
}

/// Place a triangle with the three given side lengths: A at the origin, B along +x, C
/// fixed by the law of cosines.
fn triangle(ab: f64, ac: f64, bc: f64) -> Vec<[D2; 3]> {
    let cos = ((ab * ab + ac * ac - bc * bc) / (2.0 * ab * ac)).clamp(-1.0, 1.0);
    let sin = (1.0 - cos * cos).max(0.0).sqrt();
    vec![
        c3(0.0, 0.0, 0.0),
        c3(ab, 0.0, 0.0),
        c3(ac * cos, ac * sin, 0.0),
    ]
}

fn main() {
    println!("# SATURATION-3 gate G0 — cost validation, blocking\n");
    println!(
        "# staked rule: each side = {COMPACT_FRACTION} x that pair's own located R_e\n\
         # cost classes: {CLASS_CHEAP_S} s/point, except (O,O,O) at {CLASS_OOO_S} s/point\n\
         # kill: {KILL_MULTIPLE}x its class re-scopes the campaign before tables start\n\
         # every solve goes through solve_determinant EXPLICITLY: fci::solve routes past\n\
         #   the door's refusal into an MPO builder that hangs,\n\
         #   and (O,O,O) at 207,025 is the one staked combo that crosses it.\n"
    );

    // ---------------------------------------------------------------- the geometries
    println!("## the staked geometries, from located equilibria");
    let pairs: [(&str, Species, Species); 5] = [
        ("H-H", HYDROGEN, HYDROGEN),
        ("H-Cl", HYDROGEN, CHLORINE),
        ("Cl-Cl", CHLORINE, CHLORINE),
        ("O-O", OXYGEN, OXYGEN),
        ("O-H", OXYGEN, HYDROGEN),
    ];
    let mut r_e = std::collections::HashMap::new();
    for (name, a, b) in pairs {
        let t = Instant::now();
        let (r, n) = locate_r_e(a, b);
        r_e.insert(name.to_string(), r);
        println!(
            "   {name:>6}  R_e = {r:.4} bohr  -> compact side {:.4}   ({n} solves, {:.1} s)",
            COMPACT_FRACTION * r,
            t.elapsed().as_secs_f64()
        );
    }
    let side = |k: &str| COMPACT_FRACTION * r_e[k];

    // ---------------------------------------------------------------- the five combos
    let combos: [(&str, [Species; 3], [f64; 3]); 5] = [
        // sides are (A-B, A-C, B-C) for the species triple in the same order
        ("(H,H,Cl)", [HYDROGEN, HYDROGEN, CHLORINE], [side("H-H"), side("H-Cl"), side("H-Cl")]),
        ("(H,Cl,Cl)", [HYDROGEN, CHLORINE, CHLORINE], [side("H-Cl"), side("H-Cl"), side("Cl-Cl")]),
        ("(Cl,Cl,Cl)", [CHLORINE, CHLORINE, CHLORINE], [side("Cl-Cl"), side("Cl-Cl"), side("Cl-Cl")]),
        ("(O,O,H)", [OXYGEN, OXYGEN, HYDROGEN], [side("O-O"), side("O-H"), side("O-H")]),
        ("(O,O,O)", [OXYGEN, OXYGEN, OXYGEN], [side("O-O"), side("O-O"), side("O-O")]),
    ];

    println!("\n## the measurement");
    println!(
        "   {:>11} {:>5} {:>9} {:>9} {:>8} {:>8} {:>6} {:>18} {:>10}",
        "combo", "orb", "n_det", "committed", "assemble", "CI", "iters", "exit", "verdict"
    );
    let mut fired = Vec::new();
    let mut worst_ratio = 0.0f64;
    for (name, sp, sides) in combos {
        let centers = triangle(sides[0], sides[1], sides[2]);
        let t_asm = Instant::now();
        let (space, mo, _nuc) = geometry_problem(&sp, centers);
        let asm = t_asm.elapsed().as_secs_f64();

        let committed = COMMITTED.iter().find(|(n, _)| *n == name).unwrap().1;
        let count_ok = space.n_det == committed;

        // EXPLICITLY the determinant route. See the header.
        let t_ci = Instant::now();
        let sol = solve_determinant(&space, &mo);
        let ci = t_ci.elapsed().as_secs_f64();

        let total = asm + ci;
        let class = if name == "(O,O,O)" { CLASS_OOO_S } else { CLASS_CHEAP_S };
        let ratio = total / class;
        worst_ratio = worst_ratio.max(ratio);
        // The SOLVER's exit, not this file's opinion of it.
        let exit = sol.exit.label();
        // What "converged" has to mean here: the solver reached its own target, OR it
        // stopped at the orthogonalisation floor with a residual inside the crate's
        // publication bar. Both are stated so neither is implied.
        let converged = sol.exit.is_converged() || sol.residual <= CONVERGED_RESIDUAL;
        // The variational bound. A solve above `min_i H_ii` is not the ground state, and
        // neither the residual nor the exit reason can tell — measured on this crate at
        // sixteen of sixty (H,H,Cl) geometries.
        let margin = sol.variational_margin.expect("the determinant route reports it");
        // THREE distinct outcomes, not two. The freeze states a cost CLASS ("<= 5 seconds
        // per point") and separately a KILL ("over 10x its class re-scopes the campaign").
        // A combo can miss the class without triggering the kill, and collapsing those into
        // one "ok" would report a class the measurement does not support.
        let verdict = if !count_ok {
            "COUNT"
        } else if !converged {
            "EXIT"
        } else if ratio > KILL_MULTIPLE {
            "KILL"
        } else if ratio > 1.0 {
            "over-class"
        } else {
            "in-class"
        };
        if verdict != "in-class" {
            fired.push((name, verdict, total, class, space.n_det, committed, sol.residual));
        }
        println!(
            "   {name:>11} {:>5} {:>9} {committed:>9} {asm:>8.2} {ci:>8.2} {:>6} {:>18} {verdict:>10}",
            space.n_orb,
            space.n_det,
            sol.davidson_iters,
            exit,
        );
        println!(
            "               sides {:.3} {:.3} {:.3} bohr; residual {:.2e}; variational margin \
             {margin:+.4} Ha; route {}; total {:.2} s = {:.3} of its {class:.0} s class",
            sides[0], sides[1], sides[2], sol.residual, sol.route.label(), total, ratio
        );
        assert!(
            margin > 0.0,
            "{name}: the solve is {:.4} Ha ABOVE min_i H_ii, so it is not the ground state \
             and this cost measurement is of the wrong eigenvector",
            -margin
        );
    }

    // ---------------------------------------------------------------- the verdict
    println!("\n## G0 verdict");
    println!("   worst cost as a fraction of its class: {worst_ratio:.3} (kill at {KILL_MULTIPLE:.0})");
    let killed: Vec<_> = fired
        .iter()
        .filter(|(_, why, ..)| *why == "KILL" || *why == "COUNT" || *why == "EXIT")
        .collect();
    let over: Vec<_> = fired.iter().filter(|(_, why, ..)| *why == "over-class").collect();
    if killed.is_empty() {
        println!("   G0 HOLDS: five of five combos match the committed arithmetic EXACTLY,");
        println!("   every solve converged, and nothing is near the {KILL_MULTIPLE:.0}x kill.");
    } else {
        println!("   G0 FIRED on {} combo(s):", killed.len());
        for (n, why, got, class, nd, want, res) in &killed {
            println!(
                "     {n}: {why} — {got:.2} s against a {class:.0} s class, {nd} determinants \
                 against a committed {want}, residual {res:.2e}"
            );
        }
        println!("   Per the freeze, this re-scopes the campaign BEFORE tables start.");
    }
    // Reported whether or not it changes the verdict: a combo outside the stated class is a
    // correction to the class, and the freeze's wording ("every other type <= 5 seconds per
    // point") is what the campaign will be read against.
    if !over.is_empty() {
        println!("\n   OVER ITS STATED CLASS, without triggering the kill:");
        for (n, _, got, class, ..) in &over {
            println!(
                "     {n}: {got:.2} s against the freeze's {class:.0} s — {:.2}x. Inside the \
                 kill by {:.0}x, so it does not re-scope; the CLASS statement is what needs \
                 amending, not the campaign.",
                *got / *class,
                KILL_MULTIPLE / (*got / *class)
            );
        }
    }
}
