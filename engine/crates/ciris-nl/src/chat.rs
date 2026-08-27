//! Qwen3 chat formatting.
//!
//! **Why this module exists.** An instruct model fed a bare string runs as a raw
//! completion, and the failure is silent: structural gates still pass (an enum mask
//! guarantees well-formed output whatever the prompt looks like) and latency is
//! unaffected, so only answer QUALITY degrades. Measured cost of exactly this defect
//! elsewhere in the project: **0.402 against 0.783**, from a chat template that went
//! missing without an error.
//!
//! It becomes an outright correctness defect the moment fine-tuned weights are served,
//! because the fine-tune was trained WITH the template — serving without it is
//! train/serve skew and breaks the learned output format.
//!
//! The split between [`system_turn`] and [`user_turn`] is load-bearing for performance,
//! not just tidiness: the system turn must stay inside the KV-cache prefix that is primed
//! once and reused, which is worth ~6.8x on short outputs. Only the user turn may vary
//! per call.

/// The primed, reusable prefix: the system turn only.
pub fn system_turn(system: &str) -> String {
    format!("<|im_start|>system\n{}<|im_end|>\n", system.trim_end())
}

/// The per-call suffix: one user turn plus the assistant opener.
///
/// `enable_thinking = false` is expressed the way Qwen3's own template expresses it —
/// a pre-closed, empty think block — so the model does not open one of its own.
pub fn user_turn(content: &str) -> String {
    format!(
        "<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n<think>\n\n</think>\n\n",
        content.trim()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_turn_is_a_closed_prefix() {
        let s = system_turn("You classify changes.");
        assert!(s.starts_with("<|im_start|>system\n"));
        assert!(s.ends_with("<|im_end|>\n"));
        // It must NOT open the user or assistant turn: everything after this point has to
        // stay variable, or it cannot be the reusable primed segment.
        assert!(!s.contains("<|im_start|>user"));
        assert!(!s.contains("<|im_start|>assistant"));
    }

    #[test]
    fn user_turn_opens_the_assistant_and_closes_thinking() {
        let u = user_turn("BEFORE: a\nAFTER: b");
        assert!(u.starts_with("<|im_start|>user\n"));
        assert!(u.contains("<|im_end|>\n<|im_start|>assistant\n"));
        assert!(u.ends_with("<think>\n\n</think>\n\n"));
    }

    /// Concatenation must reproduce Qwen3's conversation format exactly once.
    #[test]
    fn concatenation_has_exactly_one_of_each_turn() {
        let full = format!("{}{}", system_turn("S"), user_turn("U"));
        assert_eq!(full.matches("<|im_start|>system").count(), 1);
        assert_eq!(full.matches("<|im_start|>user").count(), 1);
        assert_eq!(full.matches("<|im_start|>assistant").count(), 1);
        assert_eq!(full.matches("<|im_end|>").count(), 2);
    }
}
