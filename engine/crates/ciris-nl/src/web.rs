//! Browser/wasm backend: rten, loading `.onnx` **directly** — no `rten-convert` step.
//!
//! Two rten facts shape this file, both established by execution rather than reading:
//!
//! 1. `Model::load_file` auto-detects format and the `contrib` feature (in `default`)
//!    registers the `com.microsoft` ops these exports use. So the published/self-produced
//!    `model_q4f16.onnx` loads as-is. **Never set `default-features = false` on rten** to
//!    trim wasm size — that drops `contrib` and the model stops loading entirely.
//!
//! 2. **rten 0.25 cannot rewind a KV cache.** `Generator` exposes `with_prompt`,
//!    `append_prompt` and `clear_prompt`, and `clear_prompt`'s own documentation says it
//!    "does not 'rewind' the conversation". There is no truncate-to-length. So the
//!    native path's trick — keep the system prefix resident, drop everything after it —
//!    has no equivalent here. Independent calls must rebuild the `Generator`, which
//!    re-prefills the system prompt every time. That cost is real and is the browser
//!    path's main structural disadvantage against native.
//!
//! Constrained decoding is done WITHOUT llguidance. The output language is four fixed
//! strings, so an exact token-level constraint is a few lines and needs no grammar
//! engine — which also keeps llguidance off the wasm dependency graph, where it is
//! unverified.

use crate::{grammar, BridgeError, NlBridge, Result, Surface};
use rten::Model;
use rten_generate::filter::LogitsFilter;
use rten_generate::Logits;
use rten_generate::sampler::ArgMax;
use rten_generate::Generator;
use tokenizers::Tokenizer;
use std::path::Path;

fn err<E: std::fmt::Display>(e: E) -> BridgeError { BridgeError::Infer(e.to_string()) }

/// Permits only tokens that continue one of the candidate strings.
///
/// The filter is stateless per rten's contract, so the prompt length is captured at
/// construction and the generated suffix is recovered from `prev_tokens`.
struct ClosedSetFilter {
    /// One token sequence per admissible output string.
    seqs: Vec<Vec<u32>>,
    prompt_len: usize,
}

impl LogitsFilter for ClosedSetFilter {
    fn filter(&self, logits: Logits, prev_tokens: &[u32]) -> Logits {
        let gen = &prev_tokens[self.prompt_len.min(prev_tokens.len())..];
        let mut allowed: Vec<u32> = self
            .seqs
            .iter()
            .filter(|s| s.len() > gen.len() && s.starts_with(gen))
            .map(|s| s[gen.len()])
            .collect();
        allowed.sort_unstable();
        allowed.dedup();
        if allowed.is_empty() {
            // Every candidate is complete; leave the logits alone and let the caller stop.
            return logits;
        }
        let (vals, idx) = logits.into_logits_indices();
        let mut keep_v = Vec::with_capacity(allowed.len());
        let mut keep_i = Vec::with_capacity(allowed.len());
        for (t, v) in idx.into_iter().zip(vals) {
            if allowed.binary_search(&t).is_ok() {
                keep_i.push(t);
                keep_v.push(v);
            }
        }
        if keep_i.is_empty() { return Logits::default(); }
        Logits::sparse(keep_v, keep_i)
    }
}

/// Resident model. Build once per process; loading is ~0.6s for the 569.8MB q4f16.
pub struct WebEngine {
    model: Model,
    tokenizer: Tokenizer,
}

impl WebEngine {
    pub fn load(model_path: impl AsRef<Path>, tokenizer_path: impl AsRef<Path>) -> Result<Self> {
        let model = Model::load_file(model_path.as_ref())
            .map_err(|e| BridgeError::Load(e.to_string()))?;
        // MUST be `tokenizers`, not `rten-text`. rten-text's `encode` silently drops
        // newline characters — `encode("\n")` returns an EMPTY token list — so every
        // multi-line prompt reaches the model as a run-on string. It is a silent
        // corruption, not a load error, and it measurably wrecks output quality.
        // (rten-text additionally cannot load the SmolLM2 family's tokenizer at all.)
        let tokenizer = Tokenizer::from_file(tokenizer_path.as_ref())
            .map_err(|e| BridgeError::Load(format!("{e}")))?;
        Ok(WebEngine { model, tokenizer })
    }

    fn encode(&self, text: &str) -> Result<Vec<u32>> {
        Ok(self
            .tokenizer
            .encode(text, false)
            .map_err(|e| BridgeError::Infer(format!("{e}")))?
            .get_ids()
            .to_vec())
    }

    /// Open a session over a fixed system prompt.
    pub fn session(&self, system: &str) -> Result<WebSession<'_>> {
        let system_ids = self.encode(system)?;
        // Precompute the admissible completions once.
        let mut seqs = Vec::with_capacity(Surface::ALL.len());
        for k in Surface::ALL {
            seqs.push(self.encode(&format!("{{\"label\": \"{}\"}}", k.as_str()))?);
        }
        Ok(WebSession { engine: self, system_ids, seqs })
    }
}

pub struct WebSession<'a> {
    engine: &'a WebEngine,
    system_ids: Vec<u32>,
    seqs: Vec<Vec<u32>>,
}

impl<'a> WebSession<'a> {
    pub fn system_tokens(&self) -> usize { self.system_ids.len() }

    fn complete(&mut self, suffix: &str) -> Result<String> {
        let mut prompt = self.system_ids.clone();
        prompt.extend(self.engine.encode(suffix)?);
        let prompt_len = prompt.len();
        let longest = self.seqs.iter().map(|s| s.len()).max().unwrap_or(16);

        // Rebuilt per call: see the module note — rten cannot rewind, so a fresh
        // Generator is the only way to get an independent call, and it re-prefills.
        let filter = ClosedSetFilter { seqs: self.seqs.clone(), prompt_len };
        let unconstrained = std::env::var("CIRIS_NL_NOFILTER").is_ok();
        let mut gen = Generator::from_model(&self.engine.model)
            .map_err(err)?
            .with_prompt(&prompt)
            .with_sampler(ArgMax::new());
        if !unconstrained { gen = gen.with_logits_filter(filter); }

        let mut out = Vec::new();
        let budget = if std::env::var("CIRIS_NL_NOFILTER").is_ok() { 16 } else { longest };
        for _ in 0..budget {
            match gen.next() {
                Some(Ok(t)) => {
                    out.push(t);
                    if self.seqs.iter().any(|s| s.as_slice() == out.as_slice()) { break; }
                }
                Some(Err(e)) => return Err(err(e)),
                None => break,
            }
        }
        self.engine
            .tokenizer
            .decode(&out, false)
            .map_err(|e| BridgeError::Infer(format!("{e}")))
    }
}

impl<'a> NlBridge for WebSession<'a> {
    fn decompose(&mut self, input: &str) -> Result<Surface> {
        let raw = self.complete(&format!("\nChange: {input}\nLabel: "))?;
        grammar::extract_label(&raw).ok_or(BridgeError::Ungrammatical(raw))
    }
    fn render(&mut self, label: Surface, subject: &str) -> Result<String> {
        Ok(format!("{subject} is a change to {} ({}).", label, label.gloss()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filt(seqs: Vec<Vec<u32>>, prompt_len: usize) -> ClosedSetFilter {
        ClosedSetFilter { seqs, prompt_len }
    }

    /// The filter must permit exactly the tokens that continue some candidate.
    #[test]
    fn permits_only_live_continuations() {
        // two candidates sharing a 1-token prefix, diverging at index 1
        let f = filt(vec![vec![9, 1, 5], vec![9, 2, 5]], 1);
        let dense = Logits::dense(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);

        // nothing generated yet -> only the shared first token is live
        let out = f.filter(dense.clone(), &[42]);
        assert_eq!(out.indices(), &[9]);

        // after the shared prefix -> both branches are live
        let out = f.filter(dense.clone(), &[42, 9]);
        assert_eq!(out.indices(), &[1, 2]);

        // committed to the first branch -> only its continuation survives
        let out = f.filter(dense, &[42, 9, 1]);
        assert_eq!(out.indices(), &[5]);
    }

    /// Once every candidate is exhausted the filter must not blank the logits,
    /// otherwise the sampler panics on an empty set.
    #[test]
    fn passes_through_when_all_candidates_complete() {
        let f = filt(vec![vec![9]], 1);
        let dense = Logits::dense(vec![0.0, 1.0]);
        let out = f.filter(dense, &[42, 9]);
        assert_eq!(out.len(), 2);
    }
}
