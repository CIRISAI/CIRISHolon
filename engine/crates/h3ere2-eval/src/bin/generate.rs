//! h3ere2 stages 2-4: engine reasoning + articulation, for all three arms.
//!
//! Arms (PREREG.md, commit 1dac9a0):
//!   A base       one call, no engine
//!   B scrambled  full pipeline, coupling scrambled -- THE PLACEBO, 10 independent draws
//!   C real       full pipeline, real coupling
//!
//! B and C share the renderer, its system prompt, its token budget and its path FORMAT.
//! The only thing that differs is which kinds the path names. That is the isolation the
//! prereg is built around, so nothing here may special-case one arm over the other.

use ciris_nl::native::{Config, Engine};
use ciris_sim_core::data::COUPLING;
use h3ere2_eval::{blocks, path, scramble};
use std::io::Write;

const N_SCRAMBLES: u64 = 10;
const MAX_TOKENS: usize = 160;

const SYS_BASE: &str = "You advise on changes to documents. Given a change, write a short \
recommendation of two or three sentences: what it affects and what the reader should watch. \
Be concrete and do not repeat the change back. Never use placeholders or square brackets.";

const SYS_PATH: &str = "You advise on changes to documents. You are \
given a change, the aspect it primarily lands on, and the order in which its effects reach \
other aspects. Write a short recommendation of two or three sentences: say what the change \
does and what the reader should check next, letting the propagation order decide the ORDER \
in which you raise concerns. Write about this document's actual content -- name the real \
things involved. Do NOT list or name the aspect words themselves. Never use placeholders or \
square brackets.";

fn trunc(s: &str, n: usize) -> String {
    if s.chars().count() <= n { s.to_string() }
    else { s.chars().take(n).collect::<String>() + "..." }
}

fn change_block(o: &serde_json::Value) -> String {
    format!("BEFORE: {}\nAFTER: {}\nWHAT CHANGED: {}",
        trunc(o["before"].as_str().unwrap_or(""), 700),
        trunc(o["after"].as_str().unwrap_or(""), 700),
        o["variation_site"].as_str().unwrap_or(""))
}

fn path_text(p: &[path::Arrival]) -> (String, String) {
    let primary: Vec<&str> = p.iter().filter(|a| a.seeded).map(|a| a.kind).collect();
    let rest: Vec<&str> = p.iter().filter(|a| !a.seeded).map(|a| a.kind).collect();
    (primary.join(", "), rest.join(" then "))
}

/// The user-turn CONTENT. `ciris-nl` applies Qwen3's chat template itself, so anything
/// templated here would be applied twice.
fn user_turn(change: &str, path: Option<(&str, &str)>) -> String {
    match path {
        None => change.to_string(),
        Some((prim, rest)) =>
            format!("{change}\nPRIMARY ASPECT: {prim}\nEFFECTS REACH, IN ORDER: {rest}"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().collect();
    // AMENDMENT_A2: soft encoding is the DEFAULT; --hard restores the argmax seeding.
    let hard = args.iter().any(|a| a == "--hard");
    args.retain(|a| a != "--hard");
    let model = args.get(1).ok_or("usage: generate [--hard] <model.gguf> <encoded.jsonl> <out.jsonl> [limit]")?;
    let encoded = args.get(2).ok_or("missing encoded.jsonl")?;
    let outpath = args.get(3).ok_or("missing out.jsonl")?;
    let limit: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(usize::MAX);

    let items: Vec<serde_json::Value> = std::fs::read_to_string(encoded)?
        .lines().filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap()).take(limit).collect();
    eprintln!("items: {}", items.len());

    let engine = Engine::load(model, Config::default())?;
    let mut base = engine.session(&ciris_nl::chat::system_turn(SYS_BASE))?;   // arm A
    let mut pathed = engine.session(&ciris_nl::chat::system_turn(SYS_PATH))?; // arms B and C, one renderer

    // Precompute structures ONCE. `Structure::from_coupling` runs an O(N^3) eigensolve
    // that this pipeline never reads, so rebuilding it per item would be pure waste.
    let real_st = path::structure_for(&COUPLING);
    let scramble_sts: Vec<_> = (0..N_SCRAMBLES)
        .map(|s| path::structure_for(&scramble::scramble(s)))
        .collect();
    let pp = path::PathParams::default();

    // Paths depend only on (surface, coupling), and the encoder yields very few distinct
    // surfaces, so computing one per item would repeat the same integration hundreds of
    // times. The finer dt makes that expensive; memoise instead.
    let mut memo: std::collections::HashMap<(String, i64), Vec<path::Arrival>> =
        std::collections::HashMap::new();

    let mut out = std::fs::File::create(outpath)?;
    let t_all = std::time::Instant::now();

    for (n, o) in items.iter().enumerate() {
        let id = o["id"].as_str().unwrap_or("?").to_string();
        let surface = o["surface"].as_str().unwrap_or("Facts").to_string();
        let seeds = blocks::members(&surface).ok_or("unknown surface")?;
        let ch = change_block(o);

        // A2 soft seeding: the classifier's 4-way softmax, in SURFACES order. Soft is
        // the default; absence of "probs" in the input falls back to hard so the gold
        // (A1.2) encoding, which has no distribution, runs unchanged.
        let mass: Option<[f64; 4]> = if hard { None } else {
            o.get("probs").and_then(|p| {
                let mut m = [0.0f64; 4];
                for (i, s) in blocks::SURFACES.iter().enumerate() {
                    m[i] = p.get(*s)?.as_f64()?;
                }
                Some(m)
            })
        };

        let mut emit = |arm: &str, seed: Option<u64>, text: String, toks: usize,
                        ms: f64, p: Option<&Vec<path::Arrival>>| {
            let path_str = p.map(|v| v.iter()
                .map(|a| format!("{}{}", a.kind, if a.seeded { "*" } else { "" }))
                .collect::<Vec<_>>().join(" > "));
            let rec = serde_json::json!({
                "id": id, "arm": arm,
                "scramble_id": seed, "scramble_seed": seed,   // both namings
                "surface": surface,
                "soft": mass.is_some(),
                "entropy": o.get("entropy"),
                "gold_surface": o.get("gold_surface"),
                "response": text.trim(),
                "path": path_str,
                "gen_tokens": toks,
                "wall_s": ms / 1000.0, "gen_ms": ms,
                "path_len": p.map(|v| v.len()).unwrap_or(0),
                "resp_chars": text.trim().chars().count(),
            });
            writeln!(out, "{}", rec).unwrap();
        };

        // ---- arm A: base, no engine ----
        let t = std::time::Instant::now();
        let (a, at) = base.generate(&user_turn(&ch, None), MAX_TOKENS)?;
        emit("A", None, a, at, t.elapsed().as_secs_f64() * 1e3, None);

        // ---- arm C: real coupling ----
        let pc = match &mass {
            Some(m) => path::relax_soft(&real_st, m, &pp),
            None => memo.entry((surface.clone(), -1))
                .or_insert_with(|| path::relax(&real_st, &seeds, &pp)).clone(),
        };
        let (prim, rest) = path_text(&pc);
        let t = std::time::Instant::now();
        let (c, ct) = pathed.generate(&user_turn(&ch, Some((&prim, &rest))), MAX_TOKENS)?;
        emit("C", None, c, ct, t.elapsed().as_secs_f64() * 1e3, Some(&pc));

        // ---- arm B: the placebo, one response per independent scramble ----
        for (s, st) in scramble_sts.iter().enumerate() {
            let pb = match &mass {
                Some(m) => path::relax_soft(st, m, &pp),
                None => memo.entry((surface.clone(), s as i64))
                    .or_insert_with(|| path::relax(st, &seeds, &pp)).clone(),
            };
            let (prim, rest) = path_text(&pb);
            let t = std::time::Instant::now();
            let (b, bt) = pathed.generate(&user_turn(&ch, Some((&prim, &rest))), MAX_TOKENS)?;
            emit("B", Some(s as u64), b, bt, t.elapsed().as_secs_f64() * 1e3, Some(&pb));
        }

        if (n + 1) % 5 == 0 {
            eprintln!("  {}/{} items  {:.0}s", n + 1, items.len(), t_all.elapsed().as_secs_f64());
        }
    }
    eprintln!("done in {:.0}s -> {outpath}", t_all.elapsed().as_secs_f64());
    Ok(())
}
