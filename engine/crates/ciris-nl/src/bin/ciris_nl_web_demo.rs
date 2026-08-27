//! Browser-backend demo: exercises the rten path on the real q4f16 export.
//! usage: ciris_nl_web_demo <model.onnx> <tokenizer.json> [iters]

use ciris_nl::web::WebEngine;
use ciris_nl::{NlBridge, Surface};
use std::time::Instant;

const SYSTEM: &str = "You classify changes to a document into exactly one of four kinds.\n\
Facts: a change to what is the case — data, values, observations, measurements.\n\
Rules: a change to what is permitted or required — policy, constraints, obligations.\n\
Identity: a change to who or what something is — names, roles, ownership, categories.\n\
Manner: a change to how something is done — style, procedure, tone, formatting.\n\
Answer with JSON: {\"label\": \"<one of Facts, Rules, Identity, Manner>\"}.\n";

const CASES: &[&str] = &[
    "The retry limit was raised from 3 to 5.",
    "Contractors are now forbidden from accessing the production database.",
    "The service formerly called Aurora is now called Beacon.",
    "Error messages should be written in sentence case rather than title case.",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<String> = std::env::args().collect();
    let model = a.get(1).ok_or("usage: <model.onnx> <tokenizer.json> [iters]")?;
    let tok = a.get(2).ok_or("usage: <model.onnx> <tokenizer.json> [iters]")?;
    let iters: usize = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(2);

    let t0 = Instant::now();
    let engine = WebEngine::load(model, tok)?;
    let t_load = t0.elapsed().as_secs_f64() * 1e3;
    let mut sess = engine.session(SYSTEM)?;
    println!("web cold start: model_load={t_load:.1}ms  system={} tokens", sess.system_tokens());

    let mut lat = Vec::new();
    for i in 0..iters {
        for c in CASES {
            let t = Instant::now();
            let label = sess.decompose(c)?;
            let ms = t.elapsed().as_secs_f64() * 1e3;
            lat.push(ms);
            if i == 0 { println!("  [{ms:>7.1}ms] {:<8} <- {c}", label.to_string()); }
        }
    }
    lat.sort_by(|x, y| x.partial_cmp(y).unwrap());
    println!("\nweb steady state: n={} min={:.1} p50={:.1} max={:.1} ms",
             lat.len(), lat[0], lat[lat.len()/2], lat[lat.len()-1]);
    println!("  {}", sess.render(Surface::Manner, "The sentence-case rule")?);
    Ok(())
}
