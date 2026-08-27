//! Native CPU backend: llama.cpp via `llama-cpp-2`, pinned at 0.1.154.
//!
//! Two structures, and the split is the architecture:
//!   `Engine`  owns the backend + model. Built ONCE per process. Cold start lives here.
//!   `Session` owns the context and the primed system prefix. Built once, reused per call.
//!
//! The model is mmap'd by llama.cpp, so `Engine::load` is cheap (~250ms for a 0.6B Q4);
//! the expensive part of a cold start is the first prefill, which is why the process must
//! stay resident and why the system prompt is primed once into the KV cache.

use crate::{grammar, BridgeError, NlBridge, Result, Surface};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llguidance::api::TopLevelGrammar;
use llguidance::{Matcher, ParserFactory};
use std::num::NonZeroU32;
use std::path::Path;
use std::pin::pin;

fn err<E: std::fmt::Display>(e: E) -> BridgeError { BridgeError::Infer(e.to_string()) }

/// True once `s` contains a complete, brace-balanced JSON object.
fn depth_closed(s: &str) -> bool {
    let (mut depth, mut seen, mut in_str, mut esc) = (0i32, false, false, false);
    for c in s.chars() {
        if in_str {
            if esc { esc = false; } else if c == '\\' { esc = true; } else if c == '"' { in_str = false; }
            continue;
        }
        match c {
            '"' => in_str = true,
            '{' => { depth += 1; seen = true; }
            '}' => { depth -= 1; if seen && depth == 0 { return true; } }
            _ => {}
        }
    }
    false
}

#[derive(Debug, Clone)]
pub struct Config {
    pub n_ctx: u32,
    /// Threads for DECODE. Measured: best at the P-core count; spilling onto E-cores
    /// made decode monotonically worse (400ms -> 486ms going 8 -> 24 on a 13900HX).
    pub n_threads: i32,
    /// Threads for PREFILL. Compute-bound, so it tolerates more threads than decode.
    /// Kept separate because the two scale in opposite directions.
    pub n_threads_batch: i32,
    pub n_batch: u32,
    pub n_ubatch: u32,
    /// Cap on generated tokens. The grammar makes outputs terse, so this is a safety rail.
    pub max_tokens: usize,
}

impl Default for Config {
    fn default() -> Self {
        let p = crate::p_core_count() as i32;
        Config {
            n_ctx: 4096,
            n_threads: p,
            // measured: ~3x the P-core count was the prefill optimum, with a mild penalty
            // beyond it; clamped so we never exceed the logical CPU count.
            n_threads_batch: (p * 3).min(
                std::thread::available_parallelism().map(|n| n.get() as i32).unwrap_or(p),
            ),
            n_batch: 2048,
            n_ubatch: 128,
            max_tokens: 24,
        }
    }
}

/// Resident, process-lifetime. Build once.
pub struct Engine {
    backend: LlamaBackend,
    model: LlamaModel,
    cfg: Config,
}

impl Engine {
    pub fn load(model_path: impl AsRef<Path>, cfg: Config) -> Result<Self> {
        let backend = LlamaBackend::init().map_err(|e| BridgeError::Load(e.to_string()))?;
        let params = pin!(LlamaModelParams::default());
        let model = LlamaModel::load_from_file(&backend, model_path.as_ref(), &params)
            .map_err(|e| BridgeError::Load(e.to_string()))?;
        Ok(Engine { backend, model, cfg })
    }

    pub fn config(&self) -> &Config { &self.cfg }

    /// Open a session and prime `system` as a reusable KV prefix.
    ///
    /// Priming is the whole point: measured 6.83x on a 5-token output, because it deletes
    /// the prefill that otherwise dominates every call.
    pub fn session(&self, system: &str) -> Result<Session<'_>> {
        let params = LlamaContextParams::default()
            .with_n_ctx(Some(NonZeroU32::new(self.cfg.n_ctx).unwrap()))
            .with_n_batch(self.cfg.n_batch)
            .with_n_ubatch(self.cfg.n_ubatch)
            .with_n_threads(self.cfg.n_threads)
            .with_n_threads_batch(self.cfg.n_threads_batch);
        let mut ctx = self.model.new_context(&self.backend, params).map_err(err)?;

        let sys_tokens = self.model.str_to_token(system, AddBos::Always).map_err(err)?;
        let n_sys = i32::try_from(sys_tokens.len())
            .map_err(|_| BridgeError::Load("system prompt too long".into()))?;

        let mut batch = LlamaBatch::new(self.cfg.n_batch as usize, 1);
        let last = sys_tokens.len() - 1;
        for (i, t) in sys_tokens.iter().enumerate() {
            batch.add(*t, i as i32, &[0], i == last).map_err(err)?;
        }
        ctx.decode(&mut batch).map_err(err)?;

        let schema = grammar::surface_schema();
        // Built ONCE. `llguidance_tok_env` walks the whole vocabulary to build a token
        // trie; on Qwen3's 151k vocab that measured 259ms, which is half a call's budget.
        // It depends only on the model, so it must never be rebuilt per request.
        let tok_env = LlamaSampler::llguidance_tok_env(&self.model);
        let factory = ParserFactory::new_simple(&tok_env).map_err(err)?;
        Ok(Session { engine: self, ctx, n_sys, schema, factory })
    }
}

/// One live context with the system prefix already resident in its KV cache.
pub struct Session<'a> {
    engine: &'a Engine,
    ctx: llama_cpp_2::context::LlamaContext<'a>,
    n_sys: i32,
    schema: String,
    factory: ParserFactory,
}

impl<'a> Session<'a> {
    pub fn system_tokens(&self) -> i32 { self.n_sys }

    /// Run one constrained completion over `suffix`, reusing the primed prefix.
    fn complete(&mut self, suffix: &str) -> Result<String> {
        let cfg = &self.engine.cfg;
        let model = &self.engine.model;

        // Drop everything after the system prefix; positions 0..n_sys survive untouched.
        self.ctx
            .clear_kv_cache_seq(Some(0), Some(self.n_sys as u32), None)
            .map_err(err)?;

        let toks = model.str_to_token(suffix, AddBos::Never).map_err(err)?;
        let mut batch = LlamaBatch::new(cfg.n_batch as usize, 1);
        let last = toks.len().saturating_sub(1);
        for (i, t) in toks.iter().enumerate() {
            batch.add(*t, self.n_sys + i as i32, &[0], i == last).map_err(err)?;
        }
        self.ctx.decode(&mut batch).map_err(err)?;

        let t_g = std::time::Instant::now();
        // Only the Matcher is per-call: it carries parse state. The factory is hoisted.
        let grammar = TopLevelGrammar::from_tagged_str("json", &self.schema)
            .map_err(|e| BridgeError::Load(format!("invalid grammar: {e}")))?;
        let parser = self.factory.create_parser(grammar).map_err(err)?;
        let mut sampler =
            LlamaSampler::chain_simple([LlamaSampler::from(Matcher::new(Ok(parser))), LlamaSampler::greedy()]);

        let g_ms = t_g.elapsed().as_secs_f64() * 1e3;
        let t_d = std::time::Instant::now();
        let mut n_gen = 0usize;
        let mut n_cur = self.n_sys + toks.len() as i32;
        let mut out = String::new();
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        for _ in 0..cfg.max_tokens {
            let token = sampler.sample(&self.ctx, batch.n_tokens() - 1);
            sampler.accept(token);
            if model.is_eog_token(token) { break; }
            n_gen += 1;
            out.push_str(&model.token_to_piece(token, &mut decoder, true, None).map_err(err)?);
            // The json grammar stops CONSTRAINING once the object is complete, but nothing
            // stops GENERATING. Without this the model runs out the whole token budget
            // emitting commentary after a valid answer (measured: 24 tokens for an 8-token
            // answer). Halt on balanced braces.
            if depth_closed(&out) { break; }
            batch.clear();
            batch.add(token, n_cur, &[0], true).map_err(err)?;
            n_cur += 1;
            self.ctx.decode(&mut batch).map_err(err)?;
        }
        if std::env::var("CIRIS_NL_TRACE").is_ok() {
            eprintln!("    trace: grammar_setup={:.1}ms decode={:.1}ms tokens={} out={:?}",
                      g_ms, t_d.elapsed().as_secs_f64()*1e3, n_gen, out);
        }
        Ok(out)
    }

    /// One UNCONSTRAINED completion over `content`, reusing the primed system prefix.
    /// Returns the text and the number of tokens generated.
    ///
    /// This is deliberately NOT `complete`. `complete` exists to emit one label and is
    /// llguidance-JSON-constrained, so it cannot produce prose: it masks the vocabulary to
    /// the schema and halts on brace balance. Free-text callers need the opposite on both
    /// counts — the whole vocabulary, and a halt on end-of-generation. The two share only
    /// the KV-cache discipline, which is the part that must not diverge, so it is written
    /// the same way here rather than factored out: the prefix at positions `0..n_sys`
    /// survives untouched and every call clears exactly what follows it.
    ///
    /// `content` is the user-turn CONTENT, not a templated string. This method applies
    /// Qwen3's chat template itself via [`crate::chat::user_turn`] — see that module for
    /// why a missing template is a silent quality defect rather than an error. Callers
    /// must therefore pass raw text; templating it first would apply the template twice.
    ///
    /// Sampling is GREEDY, so output is deterministic for a given prefix and content.
    pub fn generate(&mut self, content: &str, max_tokens: usize) -> Result<(String, usize)> {
        let cfg = &self.engine.cfg;
        let model = &self.engine.model;

        self.ctx
            .clear_kv_cache_seq(Some(0), Some(self.n_sys as u32), None)
            .map_err(err)?;

        let suffix = crate::chat::user_turn(content);
        let toks = model.str_to_token(&suffix, AddBos::Never).map_err(err)?;
        let mut batch = LlamaBatch::new(cfg.n_batch as usize, 1);
        let last = toks.len().saturating_sub(1);
        for (i, t) in toks.iter().enumerate() {
            batch.add(*t, self.n_sys + i as i32, &[0], i == last).map_err(err)?;
        }
        self.ctx.decode(&mut batch).map_err(err)?;

        let mut sampler = LlamaSampler::chain_simple([LlamaSampler::greedy()]);
        let t_d = std::time::Instant::now();
        let mut n_gen = 0usize;
        let mut n_cur = self.n_sys + toks.len() as i32;
        let mut out = String::new();
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        for _ in 0..max_tokens {
            let token = sampler.sample(&self.ctx, batch.n_tokens() - 1);
            sampler.accept(token);
            // Unconstrained prose has no structural stop the way a JSON object does, so
            // end-of-generation is the ONLY halt besides the caller's budget.
            if model.is_eog_token(token) { break; }
            n_gen += 1;
            out.push_str(&model.token_to_piece(token, &mut decoder, true, None).map_err(err)?);
            batch.clear();
            batch.add(token, n_cur, &[0], true).map_err(err)?;
            n_cur += 1;
            self.ctx.decode(&mut batch).map_err(err)?;
        }
        if std::env::var("CIRIS_NL_TRACE").is_ok() {
            eprintln!("    trace: decode={:.1}ms tokens={} out={:?}",
                      t_d.elapsed().as_secs_f64() * 1e3, n_gen, out);
        }
        Ok((out, n_gen))
    }
}

impl<'a> NlBridge for Session<'a> {
    fn decompose(&mut self, input: &str) -> Result<Surface> {
        let suffix = format!("\nChange: {input}\nLabel: ");
        let raw = self.complete(&suffix)?;
        grammar::extract_label(&raw).ok_or(BridgeError::Ungrammatical(raw))
    }

    /// Rendering is deliberately NOT a model call. The label set is closed and its
    /// glosses are fixed, so generating them would add latency and a failure mode
    /// in exchange for nothing. Kept behind the trait so a model-backed renderer can
    /// replace it without touching callers.
    fn render(&mut self, label: Surface, subject: &str) -> Result<String> {
        Ok(format!("{subject} is a change to {} ({}).", label, label.gloss()))
    }
}
