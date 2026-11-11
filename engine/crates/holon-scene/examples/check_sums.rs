//! `cargo run -p holon-scene --example check_sums -- <file.jsonl>`
//!
//! Reads sums emitted by a consumer and checks the section law on every line. Exit 0 when
//! there is no violation, 1 when there is one, 2 when the file cannot be read.

use holon_scene::sums::{check, Violation};

fn main() {
    let path = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: check_sums <file.jsonl>");
            std::process::exit(2);
        }
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {path}: {e}");
            std::process::exit(2);
        }
    };
    let rep = check(&text);
    println!("{} lines read from {path}", rep.lines);
    for (kind, n) in &rep.by_kind {
        println!("  {kind}: {n}");
    }
    println!("  crossing pairs checked from both sides: {}", rep.crossings_paired);
    for v in &rep.violations {
        match v {
            Violation::Parse { line, why } => println!("PARSE     line {line}: {why}"),
            Violation::Section { line, id, seed, kind, parent, children } => println!(
                "SECTION   line {line}: holon {id} seed {seed} {kind}: parent {parent}, children sum to {children}"
            ),
            Violation::Crossing { line, id, peer, seed, here, there } => println!(
                "CROSSING  line {line}: holon {id} reports {here} toward {peer}, which reports {there} (seed {seed})"
            ),
            Violation::Unanswered { line, id, peer, seed } => println!(
                "UNANSWERED line {line}: holon {id} names peer {peer}, which reports nothing back (seed {seed})"
            ),
        }
    }
    if rep.violations.is_empty() {
        println!("section law holds on every line");
    } else {
        println!("{} violations", rep.violations.len());
        std::process::exit(1);
    }
}
