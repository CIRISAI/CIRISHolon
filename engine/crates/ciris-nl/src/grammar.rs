//! The decode grammar. Output is constrained to exactly one of the four surface labels,
//! so a malformed or invented label is structurally impossible rather than merely unlikely.
//!
//! What this does NOT protect against is semantic misassignment — right shape, wrong slot.
//! That failure passes every structural check silently and needs a labelled eval set.

use crate::Surface;

/// JSON Schema pinning the output to `{"label": <one of four>}`.
pub fn surface_schema() -> String {
    let labels = Surface::ALL
        .iter()
        .map(|k| format!("\"{}\"", k.as_str()))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"{{
  "type": "object",
  "properties": {{
    "label": {{ "type": "string", "enum": [{labels}] }}
  }},
  "required": ["label"],
  "additionalProperties": false
}}"#
    )
}

/// Pull the label out of the constrained JSON. The grammar guarantees the shape, so this
/// is a scan rather than a parser, and any miss is a grammar bug worth surfacing loudly.
pub fn extract_label(s: &str) -> Option<Surface> {
    let after = s.split("\"label\"").nth(1)?;
    let start = after.find(':').map(|i| i + 1)?;
    let rest = &after[start..];
    let open = rest.find('"')? + 1;
    let close = rest[open..].find('"')? + open;
    Surface::parse(&rest[open..close])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn schema_names_all_four() {
        let s = surface_schema();
        for k in Surface::ALL { assert!(s.contains(k.as_str()), "{k} missing from schema"); }
    }
    #[test]
    fn extracts_each_label() {
        for k in Surface::ALL {
            let json = format!("{{\"label\": \"{}\"}}", k.as_str());
            assert_eq!(extract_label(&json), Some(k));
        }
    }
    #[test]
    fn rejects_unknown_label() {
        assert_eq!(extract_label(r#"{"label": "Premises"}"#), None);
    }
}
