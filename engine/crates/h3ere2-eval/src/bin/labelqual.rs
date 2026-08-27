//! Label quality for the bridge's own decompose path, on the frozen labelled split.
//!
//! This number has been owed since the bridge landed and was never meaningful before:
//! the crate applied no chat template, so an instruct model was being run as a raw
//! completion. Structural validity (the enum mask) hid it and latency did not care.
//! With the template correct, this measures something.
//!
//! usage: labelqual <model.gguf> <test_split.jsonl> <surface_map.json>

use ciris_nl::native::{Config, Engine};
use ciris_nl::{NlBridge, Surface};
use std::collections::HashMap;

const SYS: &str = "You classify what FAMILY of change was made to a document. Answer with \
exactly one label from this list:\n\
- Facts: the assertive family: what is claimed, how strongly, under what rule, on what premise\n\
- Rules: the directive family: what is required, in what preference order, in what step order\n\
- Identity: the declarative family: what counts as what\n\
- Manner: the force-neutral carrier family: how it is encoded, how it is presented or registered, which instance it is\n\
Pick the single family the change belongs to.";

fn trunc(s: &str, n: usize) -> String {
    if s.chars().count() <= n { s.to_string() } else { s.chars().take(n).collect::<String>() + "\n[...truncated]" }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let (model, split, map) = (&a[1], &a[2], &a[3]);

    let m: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(map)?)?;
    let k2b: HashMap<String, String> = serde_json::from_value(m["kind2block"].clone())?;
    let surf: HashMap<String, String> = serde_json::from_value(m["surface_plain"].clone())?;

    let items: Vec<serde_json::Value> = std::fs::read_to_string(split)?
        .lines().filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .filter(|o| k2b.contains_key(o["kind_target"].as_str().unwrap_or("")))
        .collect();
    eprintln!("labelled items: {}", items.len());

    let engine = Engine::load(model, Config::default())?;
    let mut sess = engine.session(SYS)?;

    let (mut ok, mut n) = (0usize, 0usize);
    let mut confusion: HashMap<(String, String), usize> = HashMap::new();
    let t0 = std::time::Instant::now();
    for o in &items {
        let gold = surf[&k2b[o["kind_target"].as_str().unwrap()]].clone();
        let content = format!(
            "BEFORE:\n{}\n\nAFTER:\n{}\n\nWHAT CHANGED: {}",
            trunc(o["before"].as_str().unwrap_or(""), 1400),
            trunc(o["after"].as_str().unwrap_or(""), 1400),
            o["variation_site"].as_str().unwrap_or(""));
        let pred: Surface = sess.decompose(&content)?;
        n += 1;
        if pred.as_str() == gold { ok += 1; }
        *confusion.entry((gold, pred.as_str().to_string())).or_default() += 1;
    }
    println!("4-way accuracy = {:.3}  ({ok}/{n})  in {:.0}s", ok as f64 / n as f64, t0.elapsed().as_secs_f64());
    let mut rows: Vec<_> = confusion.into_iter().collect();
    rows.sort_by_key(|((g, p), _)| (g.clone(), p.clone()));
    println!("gold -> pred (count):");
    for ((g, p), c) in rows { println!("  {:<9} -> {:<9} {c}", g, p); }
    Ok(())
}
