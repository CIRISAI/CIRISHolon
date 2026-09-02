//! The reduction law across the wasm build and the native referee (FSD-W3 WB-10.2): the
//! native engine computes the law probe and pins its BITS in `docs/workbench/law_probe.json`;
//! the page's smoke gate reads the same file and compares it to what the shipped wasm serves.
//! Two gates, one file: the constant cannot be edited to make either pass without the other
//! noticing.
//!
//! Set `HOLON_PIN_LAW=1` to (re)write the pin from this build — the only way it changes, and
//! the reason a regime change shows up here as a red test rather than a silent drift.

use std::path::PathBuf;

use holon_render::nucleus::holon_law_probe;

fn pin_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../docs/workbench/law_probe.json")
}

#[test]
fn the_law_probe_is_pinned_to_the_bit() {
    let e = holon_law_probe();
    let bits = format!("{:016x}", e.to_bits());
    let path = pin_path();
    let rewrite = std::env::var("HOLON_PIN_LAW").map_or(false, |v| v == "1");
    if rewrite || !path.exists() {
        let text = format!(
            "{{\n  \"probe\": \"the oxygen atom's STO-3G FCI energy on the lane engine (holon_law_probe)\",\n  \"energy_bits_hex\": \"{bits}\",\n  \"energy\": {e:.17e},\n  \"pinned_by\": \"engine/crates/holon-render/tests/wasm_law.rs (native, HostSpace under the reduction law)\"\n}}\n"
        );
        std::fs::write(&path, text).expect("write the law pin");
        println!("pinned {bits} ({e:.17e}) to {}", path.display());
        return;
    }
    let text = std::fs::read_to_string(&path).expect("read the law pin");
    let pinned = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("\"energy_bits_hex\": \"").map(|s| s.trim_end_matches("\",").trim_end_matches('"').to_string()))
        .expect("law_probe.json carries energy_bits_hex");
    assert_eq!(
        pinned, bits,
        "the native law probe ({e:.17e}, bits {bits}) is not the pinned {pinned}. The reduction law moved, \
         or the oxygen solve did; re-pin with HOLON_PIN_LAW=1 only after saying which in the record"
    );
}
