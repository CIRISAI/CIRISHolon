//! Natural-language bridge: decompose input into a typed surface label, and render
//! a chosen label back into language. The reasoning between the two is symbolic and
//! lives elsewhere; this crate only crosses the language boundary.
//!
//! Architecture is fixed by measurement, not preference (see `docs` in the repo report):
//!   * the engine is RESIDENT — cold start is 0.5-0.9s and prefill-dominated
//!   * the system prompt is a REUSED KV PREFIX — worth 6.8x at short output lengths
//!   * decoding is CONSTRAINED to the four surface labels, so output cannot be malformed
//!   * threads default to the P-core count, with prefill and decode set separately

pub mod chat;
pub mod grammar;
#[cfg(feature = "native")]
pub mod native;
#[cfg(feature = "web")]
pub mod web;

use core::fmt;

/// The four surface kinds. `Core/Surface.lean` mechanizes 11 = 4 + 7; these are the four
/// that carry ~91% of wild change-traffic, and the level at which small-model grip exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Surface {
    Facts,
    Rules,
    Identity,
    Manner,
}

impl Surface {
    pub const ALL: [Surface; 4] = [Surface::Facts, Surface::Rules, Surface::Identity, Surface::Manner];

    pub const fn as_str(self) -> &'static str {
        match self {
            Surface::Facts => "Facts",
            Surface::Rules => "Rules",
            Surface::Identity => "Identity",
            Surface::Manner => "Manner",
        }
    }

    /// Parse a label. Exact match only — the grammar guarantees one of these four,
    /// so a miss here is a bug in the grammar, not user input to be forgiven.
    pub fn parse(s: &str) -> Option<Surface> {
        Surface::ALL.into_iter().find(|k| k.as_str() == s)
    }

    /// One-line gloss used when rendering a label back into language.
    pub const fn gloss(self) -> &'static str {
        match self {
            Surface::Facts => "what is the case",
            Surface::Rules => "what is permitted or required",
            Surface::Identity => "who or what something is",
            Surface::Manner => "how something is done",
        }
    }
}

impl fmt::Display for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.as_str()) }
}

#[derive(Debug)]
pub enum BridgeError {
    Load(String),
    Infer(String),
    /// The model emitted something the grammar should have made impossible.
    Ungrammatical(String),
}

impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BridgeError::Load(m) => write!(f, "load: {m}"),
            BridgeError::Infer(m) => write!(f, "inference: {m}"),
            BridgeError::Ungrammatical(m) => write!(f, "ungrammatical output (grammar bug): {m}"),
        }
    }
}
impl std::error::Error for BridgeError {}

pub type Result<T> = core::result::Result<T, BridgeError>;

/// Both directions across the language boundary.
pub trait NlBridge {
    /// Natural language in, typed label out.
    fn decompose(&mut self, input: &str) -> Result<Surface>;
    /// Typed label in, natural language out.
    fn render(&mut self, label: Surface, subject: &str) -> Result<String>;
}

/// Performance-core count.
///
/// On hybrid Intel parts the P-cores are listed directly in sysfs; prefer that over any
/// frequency heuristic. A max-frequency comparison is actively wrong here: Turbo Boost Max
/// 3.0 gives two "favored" cores a higher ceiling than their siblings, so matching on the
/// maximum finds 2 cores on a part that has 8.
pub fn p_core_count() -> usize {
    // 1. Intel hybrid: the kernel enumerates performance cores for us.
    if let Ok(list) = std::fs::read_to_string("/sys/devices/cpu_core/cpus") {
        let n = parse_cpu_list(list.trim());
        if n > 0 { return physical_of(&cpu_list_vec(list.trim())).max(1); }
    }
    // 2. Homogeneous machine: distinct physical cores.
    let mut ids: Vec<u64> = Vec::new();
    for cpu in 0..1024u32 {
        let base = format!("/sys/devices/system/cpu/cpu{cpu}");
        if !std::path::Path::new(&base).exists() { break; }
        if let Ok(v) = std::fs::read_to_string(format!("{base}/topology/core_id")) {
            if let Ok(id) = v.trim().parse::<u64>() { ids.push(id); }
        }
    }
    if !ids.is_empty() { ids.sort_unstable(); ids.dedup(); return ids.len(); }
    // 3. Give up: assume SMT.
    std::thread::available_parallelism().map(|n| (n.get() / 2).max(1)).unwrap_or(4)
}

fn cpu_list_vec(s: &str) -> Vec<u32> {
    let mut out = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() { continue; }
        match part.split_once('-') {
            Some((a, b)) => {
                if let (Ok(a), Ok(b)) = (a.parse::<u32>(), b.parse::<u32>()) {
                    out.extend(a..=b);
                }
            }
            None => { if let Ok(a) = part.parse::<u32>() { out.push(a); } }
        }
    }
    out
}

fn parse_cpu_list(s: &str) -> usize { cpu_list_vec(s).len() }

/// Collapse a logical-CPU list to distinct physical cores (SMT siblings share a core_id).
fn physical_of(cpus: &[u32]) -> usize {
    let mut ids: Vec<(u64, u64)> = Vec::new();
    for &c in cpus {
        let base = format!("/sys/devices/system/cpu/cpu{c}");
        let core = std::fs::read_to_string(format!("{base}/topology/core_id"))
            .ok().and_then(|v| v.trim().parse::<u64>().ok());
        let pkg = std::fs::read_to_string(format!("{base}/topology/physical_package_id"))
            .ok().and_then(|v| v.trim().parse::<u64>().ok()).unwrap_or(0);
        if let Some(core) = core { ids.push((pkg, core)); }
    }
    ids.sort_unstable(); ids.dedup();
    ids.len()
}
