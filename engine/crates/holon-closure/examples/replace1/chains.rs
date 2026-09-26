//! The reasoning carrier: `accord_traces.jsonl` read into parent→child chains exactly as
//! `conformance/reasoning/reason_search0b.py`'s `load` builds them, with a minimal JSON scanner
//! (this crate and its examples take no dependency).

use std::collections::HashMap;

/// A JSON value, as far as the reader needs one.
#[derive(Clone, Debug, PartialEq)]
pub enum J {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Obj(Vec<(String, J)>),
    Other,
}

impl J {
    pub fn get(&self, k: &str) -> Option<&J> {
        match self {
            J::Obj(v) => v.iter().rev().find(|(kk, _)| kk == k).map(|(_, v)| v),
            _ => None,
        }
    }
    /// A number that is not a bool (Python's `isinstance(v, (int, float)) and not bool`).
    pub fn num(&self) -> Option<f64> {
        match self {
            J::Num(x) => Some(*x),
            _ => None,
        }
    }
    pub fn str(&self) -> Option<&str> {
        match self {
            J::Str(s) => Some(s),
            _ => None,
        }
    }
}

struct P<'a> {
    b: &'a [u8],
    i: usize,
}

impl P<'_> {
    fn ws(&mut self) {
        while self.i < self.b.len() && (self.b[self.i] as char).is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn string(&mut self) -> String {
        // at the opening quote
        self.i += 1;
        let mut out: Vec<u8> = Vec::new();
        while self.i < self.b.len() {
            let c = self.b[self.i];
            match c {
                b'"' => {
                    self.i += 1;
                    break;
                }
                b'\\' => {
                    let e = self.b[self.i + 1];
                    self.i += 2;
                    match e {
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'u' => {
                            let h = std::str::from_utf8(&self.b[self.i..self.i + 4]).unwrap();
                            let cp = u32::from_str_radix(h, 16).unwrap_or(0xfffd);
                            self.i += 4;
                            let ch = char::from_u32(cp).unwrap_or('\u{fffd}');
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        other => out.push(other),
                    }
                }
                _ => {
                    out.push(c);
                    self.i += 1;
                }
            }
        }
        String::from_utf8_lossy(&out).into_owned()
    }
    /// Parse a value; objects nested deeper than `depth` are skipped (returned as `Other`).
    fn value(&mut self, depth: usize) -> J {
        self.ws();
        match self.b[self.i] {
            b'{' => {
                self.i += 1;
                let mut v = Vec::new();
                loop {
                    self.ws();
                    if self.b[self.i] == b'}' {
                        self.i += 1;
                        break;
                    }
                    let k = self.string();
                    self.ws();
                    self.i += 1; // ':'
                    let val = if depth == 0 { self.skip() } else { self.value(depth - 1) };
                    v.push((k, val));
                    self.ws();
                    if self.b[self.i] == b',' {
                        self.i += 1;
                    }
                }
                J::Obj(v)
            }
            b'[' => self.skip(),
            b'"' => J::Str(self.string()),
            b't' => {
                self.i += 4;
                J::Bool(true)
            }
            b'f' => {
                self.i += 5;
                J::Bool(false)
            }
            b'n' => {
                self.i += 4;
                J::Null
            }
            _ => {
                let s = self.i;
                while self.i < self.b.len() && (self.b[self.i] == b'-' || self.b[self.i] == b'+' || self.b[self.i] == b'.' || self.b[self.i] == b'e' || self.b[self.i] == b'E' || self.b[self.i].is_ascii_digit()) {
                    self.i += 1;
                }
                J::Num(std::str::from_utf8(&self.b[s..self.i]).unwrap().parse().unwrap_or(f64::NAN))
            }
        }
    }
    fn skip(&mut self) -> J {
        self.ws();
        match self.b[self.i] {
            b'"' => {
                return J::Str(self.string());
            }
            b'{' | b'[' => {
                let mut depth = 0usize;
                while self.i < self.b.len() {
                    match self.b[self.i] {
                        b'"' => {
                            self.string();
                            continue;
                        }
                        b'{' | b'[' => depth += 1,
                        b'}' | b']' => {
                            depth -= 1;
                            if depth == 0 {
                                self.i += 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    self.i += 1;
                }
            }
            _ => {
                return self.value(0);
            }
        }
        J::Other
    }
}

/// Parse one JSONL line: the top-level object with its first-level objects parsed one level
/// deep (enough for `action_result.follow_up_thought_id`).
pub fn parse_line(line: &str) -> J {
    let mut p = P { b: line.as_bytes(), i: 0 };
    p.value(1)
}

/// `reason_search0b.py`'s dictionary: `[plaus, align, k_eff, corr_risk, depth, log_tok,
/// act_SPEAK]`, or `None` where the row is unusable.
fn feat(r: &J) -> Option<Vec<f64>> {
    let mut d = Vec::with_capacity(7);
    for k in ["csdma_plausibility_score", "dsdma_domain_alignment", "idma_k_eff", "idma_correlation_risk"] {
        d.push(r.get(k)?.num()?);
    }
    let a = r.get("selected_action")?.str()?;
    if a != "SPEAK" && a != "PONDER" {
        return None;
    }
    d.push(r.get("thought_depth").and_then(|v| v.num()).expect("thought_depth"));
    let tok = r.get("tokens_output").and_then(|v| v.num()).filter(|&x| x != 0.0).unwrap_or(1.0);
    d.push(tok.max(1.0).ln());
    d.push(if a == "SPEAK" { 1.0 } else { 0.0 });
    Some(d)
}

/// The chains (each `len × 7`, rows in parent→child order), in `reason_search0b.py`'s order.
pub fn load(path: &str) -> Result<Vec<Vec<Vec<f64>>>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    let recs: Vec<J> = text.lines().filter(|l| !l.trim().is_empty()).map(parse_line).collect();
    // by = {thought_id: r} — the last record of an id wins
    let mut by: HashMap<String, usize> = HashMap::new();
    for (i, r) in recs.iter().enumerate() {
        if let Some(t) = r.get("thought_id").and_then(|v| v.str()) {
            by.insert(t.to_string(), i);
        }
    }
    // child = {r.thought_id: follow_up for r in R if follow_up in by} — insertion order of the
    // key's first appearance, the value of its last qualifying record
    let mut order: Vec<String> = Vec::new();
    let mut child: HashMap<String, String> = HashMap::new();
    for r in &recs {
        let fu = r.get("action_result").and_then(|a| a.get("follow_up_thought_id")).and_then(|v| v.str());
        let Some(fu) = fu else { continue };
        if !by.contains_key(fu) {
            continue;
        }
        let t = r.get("thought_id").and_then(|v| v.str()).expect("thought_id").to_string();
        if !child.contains_key(&t) {
            order.push(t.clone());
        }
        child.insert(t, fu.to_string());
    }
    let parents: std::collections::HashSet<&String> = child.values().collect();
    let mut chains = Vec::new();
    for root in order.iter().filter(|t| !parents.contains(t)) {
        let mut seq = vec![root.clone()];
        let mut t = root.clone();
        while let Some(c) = child.get(&t) {
            t = c.clone();
            seq.push(t.clone());
        }
        let mut rows = Vec::new();
        for s in &seq {
            match feat(&recs[by[s]]) {
                Some(x) => rows.push(x),
                None => break,
            }
        }
        if rows.len() >= 2 {
            chains.push(rows);
        }
    }
    Ok(chains)
}
