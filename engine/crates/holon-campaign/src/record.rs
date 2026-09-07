//! Records: built, validated, and refused before they reach the disk.
//!
//! # What it declares
//!
//! **A record must parse as its format.** [`RecordWriter::write`] parses what it is about
//! to write with the strict parser in this module and refuses on the first thing that is
//! not JSON, with the byte offset and the text around it. The records in this repo are
//! assembled line by line by their runners — that is deliberate, it keeps every number's
//! formatting under the runner's own eye — and the cost of it is that a missing comma or a
//! `NaN` token produces a file that looks right and cannot be read. This is the gate on
//! that cost.
//!
//! **No `{:+` anywhere near a record.** Rust's `+` flag prints `+1.0`, which is not a JSON
//! number: JSON allows a leading `-` and no leading `+`. [`RecordWriter::write`] refuses any
//! `+` immediately before a digit that is not an exponent sign, in a value or inside a
//! string, and names the offset. The strict parser would catch it in a value; the explicit
//! scan catches it in a `detail` string too, where it is still wrong and would still be
//! copied into somebody's table.
//!
//! The string half is deliberately STRICTER than JSON: `"+5"` inside prose is legal JSON and
//! is refused here anyway, because a signed number in a record's own text is either the flag
//! or a number a reader will copy as one. The escape is to write the prose without the sign
//! ("5 higher", "a rise of 5"), which is what a reader wanted in the first place.
//!
//! **A screen is labelled, and the label is in every file it writes.**
//! [`RecordWriter::screen`] stamps `"dry": true` and `"screen": "<label>"` on every record;
//! a plain writer stamps `"dry": false`. [`Reading::count`] refuses a dry record, so
//! nothing a screen writes can be counted by a reader that went through this crate. That is
//! GANTT2's screen law as a type rather than as a promise.
//!
//! **A `.done` marker means the phase finished.** [`RecordWriter::done`] writes it and
//! [`is_done`] asks; a `read` phase that runs against a `run.json` with no `run.done` beside
//! it is reading a run that did not finish, and should say so.
//!
//! # What it refuses
//!
//! * Output that is not JSON, with the offset ([`WriteRefusal::NotJson`]).
//! * Output carrying a `+` before a digit ([`WriteRefusal::PlusFlag`]).
//! * A record carrying a typed stake that is not labelled `kill_from_experiment`
//!   ([`WriteRefusal::UnlabelledTypedStake`]) — see [`crate::stake`]. The writer refuses
//!   what it is ASKED to write: a stake that never went through [`Record::stake`] is not in
//!   the record and is not the writer's to catch.
//! * Counting a dry record ([`Reading::count`]).

use crate::stake::{ReadRefusal, Stake};
use std::path::{Path, PathBuf};

/// This repo's number format for a record: nine significant digits in exponent form, and
/// `null` for anything not finite. Never a `NaN` token, which is not JSON.
pub fn num(x: f64) -> String {
    if x.is_finite() {
        format!("{:.9e}", x)
    } else {
        "null".to_string()
    }
}

// ------------------------------------------------------------------ the strict parser

/// Where a record stopped being JSON.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonRefusal {
    pub at: usize,
    pub why: String,
    /// The bytes around the offset, so the refusal can be read without the file.
    pub excerpt: String,
}

impl core::fmt::Display for JsonRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "not JSON at byte {}: {} — near {:?}", self.at, self.why, self.excerpt)
    }
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    fn refuse(&self, why: &str) -> JsonRefusal {
        let lo = self.i.saturating_sub(24);
        let hi = (self.i + 24).min(self.b.len());
        JsonRefusal {
            at: self.i,
            why: why.to_string(),
            excerpt: String::from_utf8_lossy(&self.b[lo..hi]).to_string(),
        }
    }

    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn byte(&self) -> Option<u8> {
        self.b.get(self.i).copied()
    }

    fn expect(&mut self, c: u8) -> Result<(), JsonRefusal> {
        if self.byte() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            Err(self.refuse(&format!("expected {:?}", c as char)))
        }
    }

    fn literal(&mut self, word: &str) -> Result<(), JsonRefusal> {
        if self.b[self.i..].starts_with(word.as_bytes()) {
            self.i += word.len();
            Ok(())
        } else {
            Err(self.refuse(&format!("expected {word}")))
        }
    }

    fn string(&mut self) -> Result<(), JsonRefusal> {
        self.expect(b'"')?;
        loop {
            match self.byte() {
                None => return Err(self.refuse("the string never closes")),
                Some(b'"') => {
                    self.i += 1;
                    return Ok(());
                }
                Some(b'\\') => {
                    self.i += 1;
                    match self.byte() {
                        Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => self.i += 1,
                        Some(b'u') => {
                            self.i += 1;
                            for _ in 0..4 {
                                match self.byte() {
                                    Some(c) if c.is_ascii_hexdigit() => self.i += 1,
                                    _ => return Err(self.refuse("\\u needs four hex digits")),
                                }
                            }
                        }
                        _ => {
                            return Err(self.refuse(
                                "unknown escape (Rust's {:?} writes \\u{..} for a control byte, \
                                 which JSON does not accept)",
                            ))
                        }
                    }
                }
                Some(c) if c < 0x20 => {
                    return Err(self.refuse("a raw control byte inside a string"))
                }
                Some(_) => self.i += 1,
            }
        }
    }

    fn number(&mut self) -> Result<(), JsonRefusal> {
        let start = self.i;
        if self.byte() == Some(b'-') {
            self.i += 1;
        }
        if self.byte() == Some(b'+') {
            return Err(self.refuse("a JSON number has no leading '+' (the `{:+` flag wrote this)"));
        }
        match self.byte() {
            Some(b'0') => self.i += 1,
            Some(c) if c.is_ascii_digit() => {
                while matches!(self.byte(), Some(c) if c.is_ascii_digit()) {
                    self.i += 1;
                }
            }
            _ => return Err(self.refuse("expected a digit")),
        }
        if self.byte() == Some(b'.') {
            self.i += 1;
            if !matches!(self.byte(), Some(c) if c.is_ascii_digit()) {
                return Err(self.refuse("a decimal point needs a digit after it"));
            }
            while matches!(self.byte(), Some(c) if c.is_ascii_digit()) {
                self.i += 1;
            }
        }
        if matches!(self.byte(), Some(b'e' | b'E')) {
            self.i += 1;
            if matches!(self.byte(), Some(b'+' | b'-')) {
                self.i += 1;
            }
            if !matches!(self.byte(), Some(c) if c.is_ascii_digit()) {
                return Err(self.refuse("an exponent needs a digit"));
            }
            while matches!(self.byte(), Some(c) if c.is_ascii_digit()) {
                self.i += 1;
            }
        }
        debug_assert!(self.i > start);
        Ok(())
    }

    fn value(&mut self) -> Result<(), JsonRefusal> {
        self.ws();
        match self.byte() {
            None => Err(self.refuse("the record ends where a value was expected")),
            Some(b'{') => {
                self.i += 1;
                self.ws();
                if self.byte() == Some(b'}') {
                    self.i += 1;
                    return Ok(());
                }
                loop {
                    self.ws();
                    self.string()?;
                    self.ws();
                    self.expect(b':')?;
                    self.value()?;
                    self.ws();
                    match self.byte() {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(());
                        }
                        _ => return Err(self.refuse("expected ',' or '}' in an object")),
                    }
                }
            }
            Some(b'[') => {
                self.i += 1;
                self.ws();
                if self.byte() == Some(b']') {
                    self.i += 1;
                    return Ok(());
                }
                loop {
                    self.value()?;
                    self.ws();
                    match self.byte() {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Ok(());
                        }
                        _ => return Err(self.refuse("expected ',' or ']' in an array")),
                    }
                }
            }
            Some(b'"') => self.string(),
            Some(b't') => self.literal("true"),
            Some(b'f') => self.literal("false"),
            Some(b'n') => self.literal("null"),
            Some(_) => self.number(),
        }
    }
}

/// Does this text parse as one JSON document, with nothing after it?
pub fn validate_json(text: &str) -> Result<(), JsonRefusal> {
    let mut p = Parser { b: text.as_bytes(), i: 0 };
    p.value()?;
    p.ws();
    if p.i != p.b.len() {
        return Err(p.refuse("trailing bytes after the record's one value"));
    }
    Ok(())
}

/// A `+` immediately before a digit that is not an exponent's sign — the `{:+` flag's
/// signature, in a value or inside a string.
pub fn find_plus_flag(text: &str) -> Option<(usize, String)> {
    let b = text.as_bytes();
    for i in 0..b.len() {
        if b[i] != b'+' {
            continue;
        }
        if b.get(i + 1).is_none_or(|c| !c.is_ascii_digit()) {
            continue;
        }
        if i > 0 && matches!(b[i - 1], b'e' | b'E') {
            continue; // a legal JSON exponent sign
        }
        let lo = i.saturating_sub(24);
        let hi = (i + 24).min(b.len());
        return Some((i, String::from_utf8_lossy(&b[lo..hi]).to_string()));
    }
    None
}

// ------------------------------------------------------------------ building a record

/// One record, built field by field.
#[derive(Clone, Debug, Default)]
pub struct Record {
    phase: String,
    fields: Vec<(String, String)>,
    stakes: Vec<Stake>,
}

impl Record {
    /// A record of one phase: `gate`, `run`, `read`, `price`, whatever the campaign calls
    /// it. The phase is the first field of the file.
    pub fn new(phase: impl Into<String>) -> Record {
        Record { phase: phase.into(), fields: Vec::new(), stakes: Vec::new() }
    }

    fn push(&mut self, key: impl Into<String>, value: String) {
        self.fields.push((key.into(), value));
    }

    pub fn number(mut self, key: impl Into<String>, v: f64) -> Record {
        self.push(key, num(v));
        self
    }

    pub fn int(mut self, key: impl Into<String>, v: i64) -> Record {
        self.push(key, v.to_string());
        self
    }

    pub fn flag(mut self, key: impl Into<String>, v: bool) -> Record {
        self.push(key, v.to_string());
        self
    }

    pub fn text(mut self, key: impl Into<String>, v: &str) -> Record {
        self.push(key, format!("{v:?}"));
        self
    }

    /// A field whose value is already a JSON fragment — a gate's `json()`, an array of
    /// them, a nested object a runner assembled itself.
    pub fn raw(mut self, key: impl Into<String>, json_value: impl Into<String>) -> Record {
        self.push(key, json_value.into());
        self
    }

    /// Record a stake, with its provenance. The writer validates every stake added this
    /// way and refuses an unlabelled typed one.
    pub fn stake(mut self, s: &Stake) -> Record {
        self.push(format!("stake_{}", s.name.replace(' ', "_")), s.json());
        self.stakes.push(s.clone());
        self
    }

    /// Record a plant: its carrier, its analytic reach, its stake and its verdict, and its
    /// stake goes through the same validation every other stake does.
    pub fn plant(mut self, p: &crate::plant::Plant) -> Record {
        self.push(format!("plant_{}", p.name.replace(' ', "_")), p.json());
        self.stakes.push(p.stake().clone());
        self
    }

    /// Record a gate under its own id.
    pub fn gate(mut self, g: &crate::gate::Gate) -> Record {
        self.push(format!("gate_{}", g.id.replace(' ', "_")), g.json());
        self
    }

    pub fn stakes(&self) -> &[Stake] {
        &self.stakes
    }

    /// The record as JSON, with `dry` and the screen label stamped by the writer.
    fn json_with(&self, dry: bool, screen: Option<&str>) -> String {
        let mut s = format!("{{\n  \"phase\": {:?},\n  \"dry\": {dry}", self.phase);
        if let Some(label) = screen {
            s.push_str(&format!(
                ",\n  \"screen\": {label:?},\n  \"DIAGNOSTIC\": \"a screen, not a reading\""
            ));
        }
        for (k, v) in &self.fields {
            s.push_str(&format!(",\n  {k:?}: {v}"));
        }
        s.push_str("\n}\n");
        s
    }

    /// The record as it would be written by a plain (non-screen) writer. For a caller that
    /// wants the text without the disk.
    pub fn json(&self) -> String {
        self.json_with(false, None)
    }
}

// ------------------------------------------------------------------ writing

/// Why a record was not written.
#[derive(Clone, Debug, PartialEq)]
pub enum WriteRefusal {
    /// A `+` before a digit: the `{:+` flag's output, which is not a JSON number.
    PlusFlag { at: usize, excerpt: String },
    NotJson(JsonRefusal),
    /// A typed stake that does not say it is the kill from experiment.
    UnlabelledTypedStake { name: String, source: String },
    Io { path: String, why: String },
}

impl core::fmt::Display for WriteRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WriteRefusal::PlusFlag { at, excerpt } => write!(
                f,
                "a '+' before a digit at byte {at} — the `{{:+` flag writes numbers JSON \
                 cannot hold. Near {excerpt:?}"
            ),
            WriteRefusal::NotJson(j) => write!(f, "{j}"),
            WriteRefusal::UnlabelledTypedStake { name, source } => write!(
                f,
                "the stake {name:?} was typed in from {source:?} and is not labelled \
                 kill_from_experiment: derive it from a record, or say it is the kill"
            ),
            WriteRefusal::Io { path, why } => write!(f, "cannot write {path}: {why}"),
        }
    }
}

/// Writes a campaign's records into one directory, and refuses what must not be written.
#[derive(Clone, Debug)]
pub struct RecordWriter {
    dir: PathBuf,
    screen: Option<String>,
}

impl RecordWriter {
    /// A writer for a counted campaign. Every record it writes carries `"dry": false`.
    pub fn new(dir: impl Into<PathBuf>) -> RecordWriter {
        RecordWriter { dir: dir.into(), screen: None }
    }

    /// A writer for a labelled screen. Every record it writes carries `"dry": true` and the
    /// label, and [`Reading::count`] refuses all of them.
    pub fn screen(dir: impl Into<PathBuf>, label: impl Into<String>) -> RecordWriter {
        RecordWriter { dir: dir.into(), screen: Some(label.into()) }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn is_dry(&self) -> bool {
        self.screen.is_some()
    }

    pub fn screen_label(&self) -> Option<&str> {
        self.screen.as_deref()
    }

    /// Validate and write. Nothing reaches the disk unless every check passes.
    pub fn write(&self, name: &str, record: &Record) -> Result<PathBuf, WriteRefusal> {
        for s in record.stakes() {
            if !s.is_writable() {
                let source = match &s.provenance {
                    crate::stake::Provenance::Typed { source, .. } => source.clone(),
                    crate::stake::Provenance::Derived { .. } => String::new(),
                };
                return Err(WriteRefusal::UnlabelledTypedStake { name: s.name.clone(), source });
            }
        }
        let text = record.json_with(self.is_dry(), self.screen_label());
        self.write_text(name, &text)
    }

    /// Validate and write an already-assembled body. The escape hatch for a runner that
    /// builds its JSON by hand, and it goes through exactly the same two checks.
    pub fn write_text(&self, name: &str, text: &str) -> Result<PathBuf, WriteRefusal> {
        if let Some((at, excerpt)) = find_plus_flag(text) {
            return Err(WriteRefusal::PlusFlag { at, excerpt });
        }
        validate_json(text).map_err(WriteRefusal::NotJson)?;
        let path = self.dir.join(name);
        std::fs::create_dir_all(&self.dir).map_err(|e| WriteRefusal::Io {
            path: self.dir.to_string_lossy().to_string(),
            why: e.to_string(),
        })?;
        std::fs::write(&path, text).map_err(|e| WriteRefusal::Io {
            path: path.to_string_lossy().to_string(),
            why: e.to_string(),
        })?;
        Ok(path)
    }

    /// The phase's `.done` marker. Written LAST, so its presence means the phase finished.
    /// The note is free text and is not JSON, which is why it does not go through the
    /// validator.
    pub fn done(&self, name: &str, note: &str) -> Result<PathBuf, WriteRefusal> {
        let path = self.dir.join(name);
        std::fs::create_dir_all(&self.dir).map_err(|e| WriteRefusal::Io {
            path: self.dir.to_string_lossy().to_string(),
            why: e.to_string(),
        })?;
        std::fs::write(&path, format!("{note}\n")).map_err(|e| WriteRefusal::Io {
            path: path.to_string_lossy().to_string(),
            why: e.to_string(),
        })?;
        Ok(path)
    }
}

/// Is the phase's `.done` marker there?
pub fn is_done(dir: impl AsRef<Path>, name: &str) -> bool {
    dir.as_ref().join(name).exists()
}

// ------------------------------------------------------------------ reading back

/// A record read back, with the two facts a reader must not skip.
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    pub path: String,
    pub text: String,
    pub dry: bool,
    pub screen: Option<String>,
}

/// Why a reading may not be counted.
#[derive(Clone, Debug, PartialEq)]
pub enum CountRefusal {
    /// The record is a screen's. Nothing a screen writes enters a gate or a claim.
    Screen { path: String, label: String },
    /// The record is dry with no label: an instrument check, not a reading.
    Dry { path: String },
}

impl core::fmt::Display for CountRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CountRefusal::Screen { path, label } => write!(
                f,
                "{path} was written by the screen {label:?}: a screen may turn any knob, and \
                 nothing it writes enters a gate or a claim"
            ),
            CountRefusal::Dry { path } => {
                write!(f, "{path} is a dry run: an instrument check, not a reading")
            }
        }
    }
}

/// Read a record back, keeping its path and its dry flag.
pub fn read_record(path: impl AsRef<Path>) -> Result<Reading, ReadRefusal> {
    let p = path.as_ref().to_string_lossy().to_string();
    let text = std::fs::read_to_string(path.as_ref())
        .map_err(|e| ReadRefusal::Unreadable { path: p.clone(), why: e.to_string() })?;
    let dry = text.contains("\"dry\": true") || text.contains("\"dry\":true");
    let screen = text.find("\"screen\":").and_then(|i| {
        let rest = &text[i + 9..];
        let a = rest.find('"')?;
        let b = rest[a + 1..].find('"')?;
        Some(rest[a + 1..a + 1 + b].to_string())
    });
    Ok(Reading { path: p, text, dry, screen })
}

impl Reading {
    /// The reading, if it may be counted. Refuses a screen and refuses a dry run.
    pub fn count(&self) -> Result<&Reading, CountRefusal> {
        match (&self.screen, self.dry) {
            (Some(label), _) => {
                Err(CountRefusal::Screen { path: self.path.clone(), label: label.clone() })
            }
            (None, true) => Err(CountRefusal::Dry { path: self.path.clone() }),
            (None, false) => Ok(self),
        }
    }

    /// A number out of this record, with its citation kept (see [`crate::stake::ReadInput`]).
    pub fn field(&self, field: &str) -> Result<crate::stake::ReadInput, ReadRefusal> {
        crate::stake::read_input(&self.path, field)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate::Gate;
    use crate::stake::Stake;

    fn dir(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("holon-campaign-rec-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    #[test]
    fn the_parser_accepts_this_repos_own_record_shape() {
        let text = "{\n  \"phase\": \"gate\",\n  \"units\": 128, \"edge\": 2.959363131e1,\n  \
                    \"r3\": {\"branch\": \"VOID\", \"value\": null, \"ok\": false},\n  \
                    \"rows\": [1, -2.5e-7, {\"a\": true}]\n}\n";
        assert_eq!(validate_json(text), Ok(()));
    }

    #[test]
    fn the_parser_refuses_a_nan_token_and_a_trailing_comma() {
        assert!(validate_json("{\"a\": NaN}").is_err());
        assert!(validate_json("{\"a\": 1,}").is_err());
        assert!(validate_json("{\"a\": 1} trailing").is_err());
        assert!(validate_json("{\"a\": 1").is_err());
    }

    /// The `{:+` flag's output, in a value and inside a string.
    #[test]
    fn a_plus_before_a_digit_is_refused_wherever_it_appears() {
        assert!(find_plus_flag("{\"a\": +1.0}").is_some());
        assert!(find_plus_flag("{\"detail\": \"moved by +0.31\"}").is_some());
        assert!(find_plus_flag("{\"a\": 1.0e+9}").is_none(), "an exponent sign is legal JSON");
        assert!(find_plus_flag("{\"a\": -1.0}").is_none());
        // and the parser catches the value case on its own
        assert!(validate_json("{\"a\": +1.0}").is_err());
    }

    #[test]
    fn a_record_writes_validates_and_reads_back() {
        let d = dir("write");
        let w = RecordWriter::new(&d);
        let g = Gate::new("G0").work(2).leg_at("held", true, 1.5);
        let r = Record::new("gate").int("units", 128).number("edge_bohr", 29.59363131).gate(&g);
        let path = w.write("gate.json", &r).expect("the record is written");
        let back = read_record(&path).expect("the record reads back");
        assert!(!back.dry);
        assert!(back.count().is_ok());
        assert_eq!(validate_json(&back.text), Ok(()));
        assert!((back.field("edge_bohr").unwrap().value - 29.59363131).abs() < 1e-9);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn a_screen_stamps_every_file_and_the_reader_refuses_to_count_it() {
        let d = dir("screen");
        let w = RecordWriter::screen(&d, "warmer, 320 K");
        assert!(w.is_dry());
        let path = w.write("arm.json", &Record::new("run").int("frames", 10_000)).unwrap();
        let back = read_record(&path).unwrap();
        assert!(back.dry);
        assert_eq!(back.screen.as_deref(), Some("warmer, 320 K"));
        match back.count() {
            Err(CountRefusal::Screen { label, .. }) => assert_eq!(label, "warmer, 320 K"),
            other => panic!("{other:?}"),
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn the_writer_refuses_an_unlabelled_typed_stake() {
        let d = dir("stake");
        let w = RecordWriter::new(&d);
        let bad = Stake::typed("Sc water", 434.8, "Kestin 1978");
        let r = Record::new("gate").stake(&bad);
        match w.write("gate.json", &r) {
            Err(WriteRefusal::UnlabelledTypedStake { name, .. }) => assert_eq!(name, "Sc water"),
            other => panic!("{other:?}"),
        }
        assert!(!d.join("gate.json").exists(), "nothing reached the disk");
        let good = Stake::typed_kill_from_experiment("Sc water", 434.8, "Kestin 1978");
        w.write("gate.json", &Record::new("gate").stake(&good)).expect("the labelled kill writes");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn the_writer_refuses_a_plus_flag_before_it_reaches_the_disk() {
        let d = dir("plus");
        let w = RecordWriter::new(&d);
        match w.write_text("bad.json", "{\"a\": +1.0}\n") {
            Err(WriteRefusal::PlusFlag { .. }) => {}
            other => panic!("{other:?}"),
        }
        assert!(!d.join("bad.json").exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn a_done_marker_is_written_and_asked_for() {
        let d = dir("done");
        let w = RecordWriter::new(&d);
        assert!(!is_done(&d, "run.done"));
        w.write("run.json", &Record::new("run").int("steps", 1)).unwrap();
        w.done("run.done", "500 steps, L = 64").unwrap();
        assert!(is_done(&d, "run.done"));
        let _ = std::fs::remove_dir_all(&d);
    }
}
