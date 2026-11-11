//! The checker for sums a consumer emits.
//!
//! A consumer that generates a world (the CEWP twin is the first) writes one JSON object per
//! line, one line per holon, seed and kind:
//!
//! ```text
//! {"id": 422212465065985, "level": 3, "seed": 7, "kind": "households",
//!  "parent": 412, "children": [103, 99, 110, 100]}
//! {"id": 9, "level": 3, "seed": 7, "kind": "crossings", "side": 2, "peer": 10,
//!  "parent": 5, "children": [2, 3]}
//! ```
//!
//! and this module checks, for EVERY line, that the children sum to the parent (the section
//! law, `ScenePopulation.pop` on one parent), and, for crossings that name a `peer`, that the
//! two holons sharing a side report the same count for the same seed. Every violation is
//! returned, not the first.
//!
//! A line may carry `"bin": k` (a time bin). It is then one bin of a rhythm row, and the
//! check is the same: in that bin the children sum to the parent.
//!
//! Integers only: a number with a fraction, an exponent or a sign is a parse error, because a
//! count that is not an integer is not a count. The parser is hand-written for this one flat
//! shape so that the crate keeps its zero dependencies.

use std::collections::HashMap;

/// One emitted line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub id: u64,
    pub level: u64,
    pub seed: u64,
    pub kind: String,
    pub parent: u64,
    pub children: Vec<u64>,
    pub side: Option<u64>,
    pub peer: Option<u64>,
}

/// Why a line or a file failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Violation {
    /// Line `line` (1-based) is not an object of the agreed shape.
    Parse { line: usize, why: String },
    /// The children do not sum to the parent.
    Section { line: usize, id: u64, seed: u64, kind: String, parent: u64, children: u128 },
    /// Two holons sharing a side report different counts.
    Crossing { line: usize, id: u64, peer: u64, seed: u64, here: u64, there: u64 },
    /// A crossing names a peer that never reports back.
    Unanswered { line: usize, id: u64, peer: u64, seed: u64 },
}

enum Val {
    Int(u64),
    Str(String),
    Arr(Vec<u64>),
}

struct P<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> P<'a> {
    fn ws(&mut self) {
        while self.i < self.s.len() && (self.s[self.i] as char).is_ascii_whitespace() {
            self.i += 1;
        }
    }
    fn eat(&mut self, c: u8) -> Result<(), String> {
        self.ws();
        if self.i < self.s.len() && self.s[self.i] == c {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("expected '{}' at byte {}", c as char, self.i))
        }
    }
    fn peek(&mut self) -> Option<u8> {
        self.ws();
        self.s.get(self.i).copied()
    }
    fn string(&mut self) -> Result<String, String> {
        self.eat(b'"')?;
        let start = self.i;
        while self.i < self.s.len() && self.s[self.i] != b'"' {
            if self.s[self.i] == b'\\' {
                return Err(format!("escape in a string at byte {}", self.i));
            }
            self.i += 1;
        }
        if self.i >= self.s.len() {
            return Err("unterminated string".into());
        }
        let out = String::from_utf8(self.s[start..self.i].to_vec()).map_err(|_| "not utf-8".to_string())?;
        self.i += 1;
        Ok(out)
    }
    fn int(&mut self) -> Result<u64, String> {
        self.ws();
        let start = self.i;
        while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
            self.i += 1;
        }
        if start == self.i {
            return Err(format!("expected a non-negative integer at byte {}", start));
        }
        if let Some(&c) = self.s.get(self.i) {
            if c == b'.' || c == b'e' || c == b'E' {
                return Err(format!("a count must be an integer (byte {})", self.i));
            }
        }
        std::str::from_utf8(&self.s[start..self.i]).unwrap().parse::<u64>().map_err(|e| e.to_string())
    }
    fn value(&mut self) -> Result<Val, String> {
        match self.peek() {
            Some(b'"') => Ok(Val::Str(self.string()?)),
            Some(b'[') => {
                self.eat(b'[')?;
                let mut v = Vec::new();
                if self.peek() == Some(b']') {
                    self.i += 1;
                    return Ok(Val::Arr(v));
                }
                loop {
                    v.push(self.int()?);
                    match self.peek() {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Ok(Val::Arr(v));
                        }
                        _ => return Err(format!("expected ',' or ']' at byte {}", self.i)),
                    }
                }
            }
            _ => Ok(Val::Int(self.int()?)),
        }
    }
}

/// Parse one line of the agreed shape.
pub fn parse_line(text: &str) -> Result<Line, String> {
    let mut p = P { s: text.as_bytes(), i: 0 };
    p.eat(b'{')?;
    let mut ints: HashMap<String, u64> = HashMap::new();
    let mut kind = None;
    let mut children = None;
    if p.peek() != Some(b'}') {
        loop {
            let key = p.string()?;
            p.eat(b':')?;
            match (key.as_str(), p.value()?) {
                ("kind", Val::Str(s)) => kind = Some(s),
                ("children", Val::Arr(a)) => children = Some(a),
                ("id" | "level" | "seed" | "parent" | "side" | "peer" | "bin", Val::Int(n)) => {
                    if ints.insert(key.clone(), n).is_some() {
                        return Err(format!("field \"{key}\" appears twice"));
                    }
                }
                (k, _) => return Err(format!("field \"{k}\" is unknown or has the wrong type")),
            }
            match p.peek() {
                Some(b',') => p.i += 1,
                Some(b'}') => break,
                _ => return Err(format!("expected ',' or '}}' at byte {}", p.i)),
            }
        }
    }
    p.eat(b'}')?;
    if p.peek().is_some() {
        return Err("text after the object".into());
    }
    let need = |k: &str| ints.get(k).copied().ok_or_else(|| format!("missing field \"{k}\""));
    Ok(Line {
        id: need("id")?,
        level: need("level")?,
        seed: need("seed")?,
        parent: need("parent")?,
        kind: kind.ok_or("missing field \"kind\"")?,
        children: children.ok_or("missing field \"children\"")?,
        side: ints.get("side").copied(),
        peer: ints.get("peer").copied(),
    })
}

/// What a file of lines amounts to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub lines: usize,
    pub by_kind: Vec<(String, usize)>,
    pub crossings_paired: usize,
    pub violations: Vec<Violation>,
}

/// Check a whole file. Blank lines are skipped.
pub fn check(text: &str) -> Report {
    let mut rep = Report::default();
    let mut kinds: HashMap<String, usize> = HashMap::new();
    // (id, peer, seed) -> (line, count) for crossings that name a peer
    let mut cross: HashMap<(u64, u64, u64), (usize, u64)> = HashMap::new();
    for (k, raw) in text.lines().enumerate() {
        let n = k + 1;
        if raw.trim().is_empty() {
            continue;
        }
        let line = match parse_line(raw) {
            Ok(l) => l,
            Err(why) => {
                rep.violations.push(Violation::Parse { line: n, why });
                continue;
            }
        };
        rep.lines += 1;
        *kinds.entry(line.kind.clone()).or_insert(0) += 1;
        let s: u128 = line.children.iter().map(|&c| c as u128).sum();
        if s != line.parent as u128 {
            rep.violations.push(Violation::Section {
                line: n, id: line.id, seed: line.seed, kind: line.kind.clone(), parent: line.parent, children: s,
            });
        }
        if let Some(peer) = line.peer {
            cross.insert((line.id, peer, line.seed), (n, line.parent));
        }
    }
    let mut keys: Vec<_> = cross.keys().copied().collect();
    keys.sort_unstable();
    for (id, peer, seed) in keys {
        let (n, here) = cross[&(id, peer, seed)];
        match cross.get(&(peer, id, seed)) {
            None => rep.violations.push(Violation::Unanswered { line: n, id, peer, seed }),
            Some(&(_, there)) => {
                if id < peer {
                    rep.crossings_paired += 1;
                }
                if here != there {
                    rep.violations.push(Violation::Crossing { line: n, id, peer, seed, here, there });
                }
            }
        }
    }
    rep.by_kind = kinds.into_iter().collect();
    rep.by_kind.sort();
    rep
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
{"id": 422212465065985, "level": 3, "seed": 7, "kind": "households", "parent": 412, "children": [103, 99, 110, 100]}
{"id": 422212465065985, "level": 3, "seed": 7, "kind": "people", "parent": 1030, "children": [258, 247, 275, 250]}
{"id": 9, "level": 3, "seed": 7, "kind": "crossings", "side": 2, "peer": 10, "parent": 5, "children": [2, 3]}
{"id": 10, "level": 3, "seed": 7, "kind": "crossings", "side": 0, "peer": 9, "parent": 5, "children": [1, 4]}
{"id": 11, "level": 4, "seed": 7, "kind": "households", "parent": 0, "children": []}
"#;

    #[test]
    fn a_clean_file_has_no_violation() {
        let r = check(GOOD);
        assert_eq!(r.violations, vec![]);
        assert_eq!(r.lines, 5);
        assert_eq!(r.crossings_paired, 1);
        assert_eq!(r.by_kind, vec![("crossings".into(), 2), ("households".into(), 2), ("people".into(), 1)]);
    }

    #[test]
    fn every_broken_sum_is_reported_not_the_first() {
        let bad = GOOD.replace("[103, 99, 110, 100]", "[103, 99, 110, 99]").replace("[258, 247, 275, 250]", "[258, 247, 275, 251]");
        let r = check(&bad);
        assert_eq!(r.violations.len(), 2);
        assert!(matches!(r.violations[0], Violation::Section { parent: 412, children: 411, .. }));
        assert!(matches!(r.violations[1], Violation::Section { parent: 1030, children: 1031, .. }));
    }

    #[test]
    fn two_sides_that_disagree_are_caught_from_both() {
        let bad = GOOD.replace(r#""peer": 9, "parent": 5, "children": [1, 4]"#, r#""peer": 9, "parent": 6, "children": [2, 4]"#);
        let r = check(&bad);
        assert_eq!(r.violations.len(), 2);
        assert!(r.violations.iter().all(|v| matches!(v, Violation::Crossing { .. })));
    }

    #[test]
    fn a_peer_that_never_reports_is_caught() {
        let bad: String = GOOD.lines().filter(|l| !l.contains(r#""id": 10"#)).collect::<Vec<_>>().join("\n");
        let r = check(&bad);
        assert_eq!(r.violations, vec![Violation::Unanswered { line: 4, id: 9, peer: 10, seed: 7 }]);
    }

    #[test]
    fn a_fractional_count_is_refused() {
        let r = check(r#"{"id": 1, "level": 0, "seed": 0, "kind": "people", "parent": 5, "children": [2.5, 2.5]}"#);
        assert!(matches!(&r.violations[0], Violation::Parse { why, .. } if why.contains("integer")));
    }

    #[test]
    fn a_missing_field_and_an_unknown_field_are_refused() {
        let r = check("{\"id\": 1, \"level\": 0, \"kind\": \"people\", \"parent\": 5, \"children\": [5]}\n{\"id\": 1, \"level\": 0, \"seed\": 0, \"kind\": \"people\", \"parent\": 5, \"children\": [5], \"note\": 3}");
        assert_eq!(r.violations.len(), 2);
        assert!(r.violations.iter().all(|v| matches!(v, Violation::Parse { .. })));
    }

    #[test]
    fn a_positive_parent_with_no_children_breaks_the_law() {
        let r = check(r#"{"id": 1, "level": 0, "seed": 0, "kind": "people", "parent": 5, "children": []}"#);
        assert!(matches!(r.violations[0], Violation::Section { parent: 5, children: 0, .. }));
    }

    #[test]
    fn a_rhythm_line_is_checked_in_its_bin() {
        let good = r#"{"id": 7, "level": 1, "seed": 2, "bin": 33, "kind": "presence.people", "parent": 900, "children": [610, 120, 90, 80]}"#;
        assert_eq!(check(good).violations, vec![]);
        let bad = good.replace("610", "611");
        assert!(matches!(check(&bad).violations[0], Violation::Section { parent: 900, children: 901, .. }));
    }

    #[test]
    fn ids_above_two_to_the_fifty_three_survive() {
        let l = parse_line(r#"{"id": 18446744073709551615, "level": 12, "seed": 1, "kind": "devices", "parent": 1, "children": [1]}"#).unwrap();
        assert_eq!(l.id, u64::MAX);
    }
}
