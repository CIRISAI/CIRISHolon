//! Emit the shipped, referee-pinnable pair tables the browser cannot afford to solve.
//!
//! ```text
//! cargo run --release -p holon-chem --example emit_pair_tables -- <out_dir> [PAIR ...]
//! ```
//!
//! # Why these exist
//!
//! MIXTURES-1 splits the pair bank in two: light pairs are solved IN THE BROWSER at load,
//! exactly as H2 has always been, and heavy pairs arrive as files. The split's thresholds
//! are declared in `holon_render::bank` and were measured rather than guessed — Cl2 has
//! only 324 determinants but eighteen basis functions and costs a minute and a half, which
//! is not a page load.
//!
//! Every file carries its producer, its solver ROUTE, its grid RULE and one declared
//! uncertainty, because the provenance gate reads those and refuses a table that does not
//! state them. A file whose uncertainty is absent is refused rather than read as perfect.
//!
//! The manifest lists what was emitted and what each file is, so a host can discover the
//! bank without a hardcoded list.

use holon_chem::elements::{by_symbol, Species};
use holon_chem::pair::{automatic_route, generate_pair_table};
use std::time::Instant;

/// Knots per shipped curve. Denser than the P1 run's 96 because a shipped file is paid for
/// ONCE, here, and then read by every page load for free.
const KNOTS: usize = 192;

/// The pairs shipped by default: the ones MIXTURES-1's product needs and the browser
/// cannot solve. H-H is deliberately NOT here — it is light, and the whole point of the
/// sandbox is that it solves the curve it is showing.
const DEFAULT: [&str; 2] = ["HCl", "Cl2"];

fn split(name: &str) -> (Species, Species) {
    if let Some(stem) = name.strip_suffix('2') {
        let sp = by_symbol(stem).unwrap_or_else(|| panic!("unknown element {stem}"));
        return (sp, sp);
    }
    let at = name
        .char_indices()
        .skip(1)
        .find(|(_, c)| c.is_uppercase())
        .map(|(i, _)| i)
        .unwrap_or_else(|| panic!("cannot split {name}"));
    (
        by_symbol(&name[..at]).unwrap_or_else(|| panic!("unknown element {}", &name[..at])),
        by_symbol(&name[at..]).unwrap_or_else(|| panic!("unknown element {}", &name[at..])),
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "docs/atoms/tables".to_string());
    let names: Vec<String> = if args.len() > 2 {
        args[2..].to_vec()
    } else {
        DEFAULT.iter().map(|s| s.to_string()).collect()
    };
    std::fs::create_dir_all(&out).expect("output directory");

    let mut manifest = String::from("{\n  \"schema\": \"MIXTURES1/pair-table/v1\",\n");
    manifest.push_str(
        "  \"note\": \"Shipped pair curves for the pairs the browser cannot afford to \
         solve at load. Each file declares its producer, its solver route, its grid rule \
         and one uncertainty; the engine's provenance gate refuses a table that does not. \
         Light pairs are absent on purpose: the sandbox solves those itself.\",\n",
    );
    manifest.push_str(&format!("  \"knots\": {KNOTS},\n"));
    manifest.push_str("  \"pairs\": [\n");

    for (k, name) in names.iter().enumerate() {
        let (a, b) = split(name);
        let f = automatic_route(a, b);
        assert!(
            f.exists(),
            "{name}: the automatic router has no route for this curve ({} determinants, \
             {} orbitals). The determinant route can still reach it -- see \
             pair::AutomaticRoute -- but this emitter does not choose routes by hand.",
            f.n_det(),
            f.n_orb()
        );
        let t0 = Instant::now();
        let pt = generate_pair_table(a, b, KNOTS);
        let secs = t0.elapsed().as_secs_f64();
        let path = format!("{out}/{name}.json");
        std::fs::write(&path, pt.to_json()).expect("write");
        let m = &pt.meta;
        println!(
            "{name:>6}: n_basis {:>2}  n_det {:>7}  route {:?}  knots {}  \
             uncertainty {:.3e} Ha  R in [{:.4}, {:.4}]  well {:?}  {:.1} s  -> {path}",
            m.n_basis,
            m.n_det,
            m.route,
            pt.r.len(),
            m.worst_residual,
            m.r_min,
            m.r_max,
            m.well.map(|w| (w.r_e, w.d_e)),
            secs
        );
        manifest.push_str(&format!(
            "    {{\"pair\": \"{name}\", \"file\": \"{name}.json\", \"Z\": [{}, {}], \
             \"n_basis\": {}, \"n_determinants\": {}, \"solver_route\": \"{}\", \
             \"exact_in_model\": {}, \"uncertainty_hartree\": {:?}, \"n_grid\": {}}}{}\n",
            m.z_a,
            m.z_b,
            m.n_basis,
            m.n_det,
            match m.route {
                holon_chem::fci::SolverRoute::Determinant => "determinant",
                holon_chem::fci::SolverRoute::Dmrg => "DMRG",
            },
            m.route.is_exact_in_model(),
            m.worst_residual,
            pt.r.len(),
            if k + 1 == names.len() { "" } else { "," }
        ));
    }
    manifest.push_str("  ]\n}\n");
    let mpath = format!("{out}/manifest.json");
    std::fs::write(&mpath, manifest).expect("write manifest");
    println!("manifest -> {mpath}");
}
