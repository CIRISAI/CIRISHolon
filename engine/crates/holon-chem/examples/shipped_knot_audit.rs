//! Read a SHIPPED table's convergence at the density it ships at, knot by knot.
//!
//! ```text
//! cargo run --release -p holon-chem --example shipped_knot_audit -- <table.json> <Za> <Zb>
//! ```
//!
//! # The question, routed from the water lane through the lead
//!
//! `worst_residual` is a MAXIMUM over knots, so a coarser grid is optimistic and a denser
//! one can only find worse. On O-O the two readings differ by 8x (1.6353e-5 at 24 knots
//! against 1.321e-4 at 96). The nine-curve table that fed the bar's re-derivation was
//! taken at diagnostic density, so the question is whether the SHIPPED tables were ever
//! read at 192.
//!
//! By code they were: `emit_pair_tables` calls `generate_pair_table(a, b, 192)` and the
//! meta's `worst_residual` is the running max over exactly those knots. This walks them
//! anyway, because a code reading and a measurement are different evidence and the
//! declared number either reproduces or it does not.
//!
//! # And the second question, which the code cannot answer
//!
//! A knot that exits `IterationCap` reports a residual that is not a bound in either
//! direction — the water lane traced one whose value oscillates 1.08e-4, 1.33e-5,
//! 8.87e-5, 1.32e-4, 1.10e-6 across a ladder of caps before settling at 9.53e-11. The
//! shipped JSON carries no exit field, so a file cannot say that a knot gave up, and the
//! only way to find out is to solve the grid and look.
use holon_chem::dual::D2;
use holon_chem::elements::by_z;
use holon_chem::fci::SolveExit;
use holon_chem::pair::{solve_geometry, CONVERGED_RESIDUAL};

fn grid_of(src: &str) -> Vec<f64> {
    let at = src.find("\"R_grid_bohr\"").expect("no R_grid_bohr");
    let open = src[at..].find('[').unwrap() + at + 1;
    let close = src[open..].find(']').unwrap() + open;
    src[open..close]
        .split(',')
        .map(|t| t.trim().parse::<f64>().expect("bad knot"))
        .collect()
}

fn declared(src: &str, key: &str) -> Option<f64> {
    let at = src.find(&format!("\"{key}\""))?;
    let rest = &src[at + key.len() + 2..];
    let colon = rest.find(':')? + 1;
    let tail = rest[colon..].trim_start();
    let end = tail
        .find(|c: char| !(c.is_ascii_digit() || "+-.eE".contains(c)))
        .unwrap_or(tail.len());
    tail[..end].parse().ok()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let (path, za, zb) = (
        a[1].clone(),
        a[2].parse::<u32>().unwrap(),
        a[3].parse::<u32>().unwrap(),
    );
    let src = std::fs::read_to_string(&path).expect("cannot read the table");
    let grid = grid_of(&src);
    let (sa, sb) = (by_z(za).unwrap(), by_z(zb).unwrap());
    println!("# {path}: {}{} over {} knots", sa.symbol, sb.symbol, grid.len());
    println!("# bar CONVERGED_RESIDUAL = {CONVERGED_RESIDUAL:.3e}");

    let mut worst = (0usize, 0.0f64, 0.0f64);
    let mut capped: Vec<(usize, f64, f64, SolveExit)> = Vec::new();
    let mut n_conv = 0usize;
    for (i, r) in grid.iter().copied().enumerate() {
        let sol = solve_geometry(
            &[sa, sb],
            vec![
                [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                [D2::c(0.0), D2::c(0.0), D2::var(r)],
            ],
        );
        if sol.exit == SolveExit::Converged || sol.exit == SolveExit::Trivial {
            n_conv += 1;
        } else {
            capped.push((i, r, sol.residual, sol.exit));
        }
        if sol.residual > worst.1 {
            worst = (i, sol.residual, r);
        }
    }

    println!(
        "\nWORST DAVIDSON RESIDUAL: {:.6e} at knot {} (R = {:.6} bohr) — {:.1}% of the bar",
        worst.1,
        worst.0,
        worst.2,
        100.0 * worst.1 / CONVERGED_RESIDUAL
    );
    match declared(&src, "worst_davidson_residual") {
        Some(d) => println!(
            "  declared in the file: {d:.6e}  — {}",
            if (d - worst.1).abs() <= 1e-18 { "REPRODUCES EXACTLY" } else { "DOES NOT REPRODUCE" }
        ),
        None => println!("  the file declares no worst_davidson_residual"),
    }
    println!(
        "\nEXITS: {n_conv}/{} converged or trivial, {} did not",
        grid.len(),
        capped.len()
    );
    for (i, r, res, ex) in &capped {
        println!(
            "  knot {i:>3} R = {r:>9.5}  residual {res:.4e}  exit {ex:?}  \
             — {} the bar",
            if *res <= CONVERGED_RESIDUAL { "UNDER" } else { "over" }
        );
    }
    if capped.is_empty() {
        println!("  no knot gave up: every residual in this file is a solve that finished.");
    }
}
