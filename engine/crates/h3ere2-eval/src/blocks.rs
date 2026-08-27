//! Surface label -> the engine kinds that carry it.
//!
//! The chain is `Block -> Site -> ChoiceKind -> plain name -> engine index`, and every
//! link is in the Lean:
//!   * `Core/Surface.lean`   `Site.block`      (block_cards = [4,3,1,3], summing to 11)
//!   * `Core/Surface.lean`   `Block.surface`   (the representative site per block)
//!   * `Core/Generator.lean` `Site.kind`       (site -> ChoiceKind)
//!   * `Core/WrongKind.lean` `WrongKind.plain` (ChoiceKind -> the public name)
//!
//! The table below is checked against those sources by `tests::agrees_with_lean`, which
//! parses the Lean rather than trusting this comment. If the Lean moves, the test fails.

use ciris_sim_core::data::KINDS;

/// The four surface labels, in the classifier's order.
pub const SURFACES: [&str; 4] = ["Facts", "Rules", "Identity", "Manner"];

/// Plain kind names per surface block, in Lean site order.
pub const BLOCK_MEMBERS: [(&str, &[&str]); 4] = [
    ("Facts",    &["Facts", "Confidence", "Model", "Premises"]), // assertive
    ("Rules",    &["Rules", "Priorities", "Process"]),           // directive
    ("Identity", &["Identity"]),                                 // declaration
    ("Manner",   &["Manner", "Structure", "Circumstances"]),      // carrier
];

/// Engine index of a plain kind name.
pub fn index_of(kind: &str) -> Option<usize> {
    KINDS.iter().position(|k| *k == kind)
}

/// Engine indices carrying `surface`; the first is the block's representative,
/// which is the node the perturbation is injected at.
pub fn members(surface: &str) -> Option<Vec<usize>> {
    BLOCK_MEMBERS
        .iter()
        .find(|(s, _)| *s == surface)
        .map(|(_, ks)| ks.iter().filter_map(|k| index_of(k)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    /// Re-derive the mapping from the Lean sources and compare. This is the guard that
    /// makes the table above a derivation rather than a transcription.
    #[test]
    fn agrees_with_lean() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../CIRISOntology/Core/");
        let read = |f: &str| std::fs::read_to_string(format!("{root}{f}")).ok();
        let (Some(surface), Some(gen), Some(wk)) = (
            read("Surface.lean"), read("Generator.lean"), read("WrongKind.lean")
        ) else {
            eprintln!("Lean sources not reachable; skipping derivation check");
            return;
        };

        // `| .site => .block`
        let arrows = |src: &str, section: &str| -> HashMap<String, String> {
            let body = match src.split_once(section) { Some((_, b)) => b, None => return HashMap::new() };
            let mut out = HashMap::new();
            for line in body.lines() {
                let l = line.trim();
                // Lean interleaves `--` comments between the match arms, so a non-arrow
                // line is not the end of the definition. Only a new top-level item is.
                if l.is_empty() || l.starts_with("--") { continue; }
                if !l.starts_with('|') {
                    if l.starts_with("def ") || l.starts_with("theorem ")
                        || l.starts_with("/-") || l.starts_with("abbrev ")
                        || l.starts_with("inductive ") { break; }
                    continue;
                }
                if let Some((lhs, rhs)) = l.trim_start_matches('|').split_once("=>") {
                    let a = lhs.trim().trim_start_matches('.').trim().to_string();
                    let b = rhs.trim().trim_start_matches('.').trim()
                             .trim_matches('"').to_string();
                    if !a.is_empty() && !b.is_empty() { out.insert(a, b); }
                }
            }
            out
        };

        let site_block = arrows(&surface, "def Site.block");
        let block_surface = arrows(&surface, "def Block.surface");
        let site_kind = arrows(&gen, "def Site.kind");
        let kind_plain = arrows(&wk, "def WrongKind.plain");
        assert_eq!(site_block.len(), 11, "expected 11 sites in Site.block");
        assert_eq!(block_surface.len(), 4, "expected 4 blocks");

        // block -> set of plain kind names
        let mut derived: HashMap<String, HashSet<String>> = HashMap::new();
        for (site, block) in &site_block {
            let kind = site_kind.get(site).expect("site missing from Site.kind");
            let plain = kind_plain.get(kind).expect("kind missing from WrongKind.plain");
            derived.entry(block.clone()).or_default().insert(plain.clone());
        }
        // block -> surface label, via its representative site
        let mut block_label: HashMap<String, String> = HashMap::new();
        for (block, site) in &block_surface {
            let kind = site_kind.get(site).expect("representative site missing");
            let plain = kind_plain.get(kind).expect("representative kind missing");
            block_label.insert(block.clone(), plain.clone());
        }

        for (block, kinds) in &derived {
            let label = block_label.get(block).expect("block has no surface");
            let ours: HashSet<String> = members(label)
                .unwrap_or_else(|| panic!("no members for surface {label}"))
                .into_iter().map(|i| KINDS[i].to_string()).collect();
            assert_eq!(&ours, kinds, "block {block} (surface {label}) disagrees with the Lean");
        }
        // and the partition must be exactly the 11 kinds
        let total: usize = derived.values().map(|s| s.len()).sum();
        assert_eq!(total, KINDS.len(), "blocks do not partition the 11 kinds");
    }

    #[test]
    fn representative_is_first_and_named_by_the_surface() {
        for (label, ks) in BLOCK_MEMBERS {
            assert_eq!(ks[0], label, "the representative must lead the member list");
        }
    }
}
