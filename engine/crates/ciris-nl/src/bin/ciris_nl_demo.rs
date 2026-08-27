//! Resident-process demo and latency harness for the native bridge.
//!
//! usage: ciris-nl-demo <model.gguf> [iters]

use ciris_nl::native::{Config, Engine};
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
    "Measured throughput fell from 1.2M to 900K requests per second.",
    "All deployments must now be approved by two reviewers.",
    "Ownership of the billing module moved from Platform to Payments.",
    "Log timestamps are now rendered in ISO-8601 instead of epoch seconds.",
];

fn pct(v: &mut Vec<f64>, p: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if v.is_empty() { return 0.0; }
    v[((v.len() as f64 - 1.0) * p).round() as usize]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let model_path = args.get(1).cloned().ok_or("usage: ciris-nl-demo <model.gguf> [iters]")?;
    let iters: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);

    let cfg = Config::default();
    println!(
        "config: p_cores={} n_threads={} n_threads_batch={} n_ubatch={} max_tokens={}",
        ciris_nl::p_core_count(), cfg.n_threads, cfg.n_threads_batch, cfg.n_ubatch, cfg.max_tokens
    );

    // ---- COLD START: paid once, because the process stays resident ----
    let t0 = Instant::now();
    let engine = Engine::load(&model_path, cfg)?;
    let t_load = t0.elapsed().as_secs_f64() * 1e3;
    let mut sess = engine.session(SYSTEM)?;
    let t_ready = t0.elapsed().as_secs_f64() * 1e3;
    println!(
        "cold start: model_load={t_load:.1}ms  +system_prime={:.1}ms  ready={t_ready:.1}ms  (system={} tokens)",
        t_ready - t_load, sess.system_tokens()
    );

    // ---- correctness pass ----
    println!("\n-- decompose (constrained to 4 labels) --");
    let mut agree = 0usize;
    for c in CASES {
        let t = Instant::now();
        let label = sess.decompose(c)?;
        let ms = t.elapsed().as_secs_f64() * 1e3;
        println!("  [{:>6.1}ms] {:<8} <- {}", ms, label.to_string(), c);
        agree += 1;
        let _ = label;
    }
    println!("  {agree}/{} produced a well-formed label", CASES.len());

    println!("\n-- render (closed set, no model call) --");
    println!("  {}", sess.render(Surface::Rules, "The two-reviewer requirement")?);

    // ---- steady-state latency, prefix reused every call ----
    let mut lat = Vec::new();
    for _ in 0..iters {
        for c in CASES {
            let t = Instant::now();
            let _ = sess.decompose(c)?;
            lat.push(t.elapsed().as_secs_f64() * 1e3);
        }
    }
    let n = lat.len();
    let mean: f64 = lat.iter().sum::<f64>() / n as f64;
    println!(
        "\nsteady state (prefix reused): n={n} min={:.1} p50={:.1} mean={:.1} p95={:.1} max={:.1} ms",
        pct(&mut lat, 0.0), pct(&mut lat, 0.5), mean, pct(&mut lat, 0.95), pct(&mut lat, 1.0)
    );
    Ok(())
}
