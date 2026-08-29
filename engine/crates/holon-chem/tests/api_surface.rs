//! Every public item the library itself never references must be a DECISION.
//!
//! # The shape this exists to catch
//!
//! A computation or a guard that only a test reaches looks alive and is on no path that
//! runs. This crate shipped two: `homonuclear_size`, reachable only from an example, whose
//! contact-radius branch nothing checked; and `PairMeta::converged`, computed, serialised,
//! and never consulted by the production path — added an hour earlier in response to the
//! sibling lane finding exactly this shape in its own code, and reproducing it.
//!
//! # Why `dead_code` is not enough, and what covers the rest
//!
//! `cargo build --lib` with `-D dead_code -D unreachable_pub` catches the crate-internal
//! half and this crate is clean under it. It cannot catch the other half: `dead_code` never
//! fires on genuine `pub` API, and every item below IS public. So the reference count is
//! taken by hand across `src/`, with `#[cfg(test)]` modules stripped, and anything with no
//! library-internal reference must be named in `tests/data/api_surface.txt` with a reason.
//!
//! The allowlist is a TEXT file outside `src/` on purpose. Put it in a `.rs` file under the
//! scanned tree and naming an item there would count as referencing it — an allowlist that
//! launders its own entries out of the bucket it exempts them from is not an allowlist.
//! The referee lane hit precisely that; this is the same fix applied in advance.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// Remove `#[cfg(test)]` blocks, brace-matched, so a name used only by an inline unit test
/// does not read as a library reference.
fn strip_test_modules(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(at) = rest.find("#[cfg(test)]") {
        out.push_str(&rest[..at]);
        let after = &rest[at..];
        let Some(open) = after.find('{') else { break };
        let mut depth = 0usize;
        let mut end = None;
        for (i, c) in after[open..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(open + i + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        match end {
            Some(e) => rest = &after[e..],
            None => break,
        }
    }
    out.push_str(rest);
    out
}

fn word_count(haystack: &str, word: &str) -> usize {
    let mut n = 0;
    let bytes = haystack.as_bytes();
    let mut from = 0;
    while let Some(at) = haystack[from..].find(word) {
        let s = from + at;
        let e = s + word.len();
        let before_ok = s == 0 || !(bytes[s - 1] as char).is_alphanumeric() && bytes[s - 1] != b'_';
        let after_ok = e >= bytes.len()
            || !(bytes[e] as char).is_alphanumeric() && bytes[e] != b'_';
        if before_ok && after_ok {
            n += 1;
        }
        from = e;
    }
    n
}

#[test]
fn every_unreferenced_public_item_is_a_decision() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut sources = Vec::new();
    for entry in std::fs::read_dir(root.join("src")).expect("src/").flatten() {
        let p = entry.path();
        if p.extension().is_some_and(|e| e == "rs") {
            sources.push(strip_test_modules(&std::fs::read_to_string(&p).unwrap()));
        }
    }
    assert!(sources.len() >= 8, "expected the whole module set, found {}", sources.len());
    let body = sources.join("\n");

    // `pub fn NAME`, `pub const NAME`, `pub static NAME`.
    let mut names: BTreeSet<String> = BTreeSet::new();
    for kw in ["pub fn ", "pub const ", "pub static "] {
        let mut from = 0;
        while let Some(at) = body[from..].find(kw) {
            let s = from + at + kw.len();
            let end = body[s..]
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .map(|i| s + i)
                .unwrap_or(body.len());
            if end > s {
                names.insert(body[s..end].to_string());
            }
            from = s;
        }
    }
    assert!(names.len() > 100, "only {} public items found; the scan is broken", names.len());

    let mut unreferenced: Vec<String> = Vec::new();
    for name in names.iter() {
        let uses = word_count(&body, name);
        let defs = ["pub fn ", "pub const ", "pub static "]
            .iter()
            .map(|kw| word_count(&body, &format!("{kw}{name}")))
            .sum::<usize>();
        if uses.saturating_sub(defs) == 0 {
            unreferenced.push(name.clone());
        }
    }

    let list = std::fs::read_to_string(root.join("tests/data/api_surface.txt"))
        .expect("tests/data/api_surface.txt");
    let classified: BTreeSet<&str> = list
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_whitespace().next())
        .collect();

    let unclassified: Vec<&String> = unreferenced
        .iter()
        .filter(|n| !classified.contains(n.as_str()))
        .collect();
    println!(
        "  {} public items, {} referenced nowhere in the library, {} classified, {} not",
        names.len(),
        unreferenced.len(),
        classified.len(),
        unclassified.len()
    );
    assert!(
        unclassified.is_empty(),
        "these public items are referenced NOWHERE inside the library, and are not \
         classified in tests/data/api_surface.txt: {unclassified:?}\n\nEach is either real \
         API with a named consumer, or a computation only a test reaches — which is the \
         defect shape this audit exists for. Name it with a reason; do not delete the line \
         to make this pass."
    );

    // The other direction: a classification whose item no longer exists is stale, and a
    // stale allowlist quietly grants exemptions to whatever later takes the name.
    let stale: Vec<&&str> = classified
        .iter()
        .filter(|c| !names.contains(**c))
        .collect();
    assert!(stale.is_empty(), "api_surface.txt classifies items that no longer exist: {stale:?}");
}
