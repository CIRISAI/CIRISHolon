//! Stakes, and the read inputs they are derived from.
//!
//! # What it declares
//!
//! **A stake carries where it came from.** [`Stake::derived`] takes the [`ReadInput`]s it
//! was computed from and the ARITHMETIC that computed it, both as data, so a band printed
//! in a log can be checked against the records it came out of without reading the source.
//! FLUID-1's runner does this by hand — every value printed with its path and its field —
//! and this is that discipline as a type.
//!
//! **A read input keeps its path and its field** (M-STALE-INSTRUMENT). A number lifted out
//! of a record and carried around as a bare `f64` is a number nobody can follow back; the
//! whole misfit is about citations that cannot be resolved from outside the session that
//! wrote them.
//!
//! **A typed stake must say it is a kill from experiment.** There is one legitimate reason
//! to type a number into an instrument rather than derive it: it came from outside this
//! engine and it is what would falsify the claim — water's Schmidt number, water's first
//! peak position, the diffusion constant. [`Stake::typed`] produces a stake the record
//! writer REFUSES; [`Stake::typed_kill_from_experiment`] produces one it accepts, and the
//! label is in the record where a reader will see it.
//!
//! # What it refuses
//!
//! * A record containing an unlabelled typed stake ([`crate::record::RecordWriter::write`]
//!   returns [`crate::record::WriteRefusal::UnlabelledTypedStake`]).
//! * A read whose file, key path or field is absent, or whose field is not a number — each
//!   with the file it was absent from, never a default. A default here is how a campaign
//!   runs on a number nobody wrote.

use crate::record::num;

/// A number read out of a JSON record, with the citation retained.
#[derive(Clone, Debug, PartialEq)]
pub struct ReadInput {
    /// The path the value was read from, exactly as the caller gave it.
    pub path: String,
    /// The literal keys that had to be found first, in order, to reach the field. Empty for
    /// a flat record.
    pub keys: Vec<String>,
    pub field: String,
    pub value: f64,
}

impl ReadInput {
    /// The citation, printable beside the number wherever it is used.
    pub fn cite(&self) -> String {
        if self.keys.is_empty() {
            format!("{}:{} = {:.9e}", self.path, self.field, self.value)
        } else {
            format!("{}:{}.{} = {:.9e}", self.path, self.keys.join("."), self.field, self.value)
        }
    }

    pub fn json(&self) -> String {
        format!(
            "{{\"path\": {:?}, \"keys\": [{}], \"field\": {:?}, \"value\": {}}}",
            self.path,
            self.keys.iter().map(|k| format!("{k:?}")).collect::<Vec<_>>().join(", "),
            self.field,
            num(self.value)
        )
    }
}

/// Why a record could not be read.
#[derive(Clone, Debug, PartialEq)]
pub enum ReadRefusal {
    Unreadable { path: String, why: String },
    NoKey { path: String, key: String, at: usize },
    NoField { path: String, field: String },
    NotANumber { path: String, field: String, text: String },
}

impl core::fmt::Display for ReadRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ReadRefusal::Unreadable { path, why } => write!(f, "cannot read the record {path}: {why}"),
            ReadRefusal::NoKey { path, key, at } => {
                write!(f, "{path}: no {key:?} at or after byte {at}")
            }
            ReadRefusal::NoField { path, field } => write!(f, "{path}: no field {field:?}"),
            ReadRefusal::NotANumber { path, field, text } => {
                write!(f, "{path}: field {field:?} is not a number: {text:?}")
            }
        }
    }
}

/// The first number written under `field` in a JSON record, with its citation kept.
pub fn read_input(path: &str, field: &str) -> Result<ReadInput, ReadRefusal> {
    read_input_after(path, &[], field)
}

/// As [`read_input`], after every literal in `keys` has been found in order.
///
/// Pretty-printed and nested records are read the same way as flat ones. This is
/// deliberately the same textual walk `fluid1_lattice.rs` uses rather than a parse into a
/// tree: the records in this repo are assembled line by line, the walk is what has been
/// reading them, and a second reading strategy is how one record comes to have two values.
pub fn read_input_after(path: &str, keys: &[&str], field: &str) -> Result<ReadInput, ReadRefusal> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| ReadRefusal::Unreadable { path: path.to_string(), why: e.to_string() })?;
    let mut at = 0usize;
    for needle in keys {
        let i = text[at..].find(needle).ok_or_else(|| ReadRefusal::NoKey {
            path: path.to_string(),
            key: (*needle).to_string(),
            at,
        })?;
        at = at + i + needle.len();
    }
    let pat = format!("\"{field}\":");
    let i = text[at..]
        .find(&pat)
        .ok_or_else(|| ReadRefusal::NoField { path: path.to_string(), field: field.to_string() })?;
    let start = at + i + pat.len();
    let rest = &text[start..];
    let end = rest.find([',', '}', '\n']).unwrap_or(rest.len());
    let raw = rest[..end].trim().trim_matches('"');
    let value = raw.parse::<f64>().map_err(|_| ReadRefusal::NotANumber {
        path: path.to_string(),
        field: field.to_string(),
        text: raw.to_string(),
    })?;
    Ok(ReadInput {
        path: path.to_string(),
        keys: keys.iter().map(|k| (*k).to_string()).collect(),
        field: field.to_string(),
        value,
    })
}

/// Where a stake's number came from.
#[derive(Clone, Debug, PartialEq)]
pub enum Provenance {
    /// Computed from records this engine wrote, by the stated arithmetic.
    Derived { inputs: Vec<ReadInput>, arithmetic: String },
    /// Typed in from outside. Admissible only when it is the kill and says so.
    Typed { source: String, kill_from_experiment: bool },
}

/// A number a freeze commits to before it sees a result.
#[derive(Clone, Debug, PartialEq)]
pub struct Stake {
    pub name: String,
    pub value: f64,
    pub provenance: Provenance,
}

impl Stake {
    /// A stake computed from records, carrying the inputs and the printed arithmetic.
    pub fn derived(
        name: impl Into<String>,
        value: f64,
        from: &[ReadInput],
        arithmetic: &str,
    ) -> Stake {
        Stake {
            name: name.into(),
            value,
            provenance: Provenance::Derived {
                inputs: from.to_vec(),
                arithmetic: arithmetic.to_string(),
            },
        }
    }

    /// A number typed in. The record writer REFUSES this one: use
    /// [`Stake::typed_kill_from_experiment`] where the number is the kill, or derive it.
    pub fn typed(name: impl Into<String>, value: f64, source: &str) -> Stake {
        Stake {
            name: name.into(),
            value,
            provenance: Provenance::Typed { source: source.to_string(), kill_from_experiment: false },
        }
    }

    /// A number typed in because it comes from EXPERIMENT and is what would falsify the
    /// claim. `source` is the citation — the paper, the measurement, the year.
    pub fn typed_kill_from_experiment(name: impl Into<String>, value: f64, source: &str) -> Stake {
        Stake {
            name: name.into(),
            value,
            provenance: Provenance::Typed { source: source.to_string(), kill_from_experiment: true },
        }
    }

    /// May a record carry this stake?
    pub fn is_writable(&self) -> bool {
        match &self.provenance {
            Provenance::Derived { .. } => true,
            Provenance::Typed { kill_from_experiment, .. } => *kill_from_experiment,
        }
    }

    /// The stake with its provenance, for the console.
    pub fn print(&self) -> String {
        match &self.provenance {
            Provenance::Derived { inputs, arithmetic } => {
                let mut s = format!("{} = {:.9e}   DERIVED: {arithmetic}", self.name, self.value);
                for i in inputs {
                    s.push_str(&format!("\n    from {}", i.cite()));
                }
                s
            }
            Provenance::Typed { source, kill_from_experiment: true } => {
                format!("{} = {:.9e}   TYPED, and it is the KILL: {source}", self.name, self.value)
            }
            Provenance::Typed { source, kill_from_experiment: false } => format!(
                "{} = {:.9e}   TYPED and UNLABELLED: {source} — a record writer refuses this",
                self.name, self.value
            ),
        }
    }

    pub fn json(&self) -> String {
        match &self.provenance {
            Provenance::Derived { inputs, arithmetic } => format!(
                "{{\"name\": {:?}, \"value\": {}, \"provenance\": \"derived\", \
                 \"arithmetic\": {:?}, \"inputs\": [{}]}}",
                self.name,
                num(self.value),
                arithmetic,
                inputs.iter().map(|i| i.json()).collect::<Vec<_>>().join(", ")
            ),
            Provenance::Typed { source, kill_from_experiment } => format!(
                "{{\"name\": {:?}, \"value\": {}, \"provenance\": \"typed\", \
                 \"kill_from_experiment\": {}, \"source\": {:?}}}",
                self.name,
                num(self.value),
                kill_from_experiment,
                source
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmp(name: &str, body: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!("holon-campaign-{}-{}", std::process::id(), name));
        let mut f = std::fs::File::create(&p).expect("create the temp record");
        f.write_all(body.as_bytes()).expect("write the temp record");
        p.to_string_lossy().to_string()
    }

    #[test]
    fn a_read_input_keeps_its_path_and_its_field() {
        let p = tmp("flat.json", "{\n  \"units\": 128, \"cell_edge_bohr\": 2.959363131e1\n}\n");
        let r = read_input(&p, "cell_edge_bohr").unwrap();
        assert!((r.value - 29.59363131).abs() < 1e-9);
        assert!(r.cite().contains("cell_edge_bohr"));
        assert!(r.cite().contains(&p));
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn a_nested_record_is_read_through_its_key_path() {
        let p = tmp(
            "nested.json",
            "{\n  \"r1\": {\"peak\": 5.75}, \"r2\": {\"peak\": 1.184}\n}\n",
        );
        let a = read_input_after(&p, &["\"r1\""], "peak").unwrap();
        let b = read_input_after(&p, &["\"r2\""], "peak").unwrap();
        assert!((a.value - 5.75).abs() < 1e-12);
        assert!((b.value - 1.184).abs() < 1e-12);
        assert!(b.cite().contains("\"r2\".peak"));
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn a_missing_file_key_or_field_is_a_refusal_and_never_a_default() {
        let p = tmp("thin.json", "{\"a\": 1}\n");
        assert!(matches!(read_input("/nowhere/at/all.json", "a"), Err(ReadRefusal::Unreadable { .. })));
        assert!(matches!(read_input(&p, "b"), Err(ReadRefusal::NoField { .. })));
        assert!(matches!(
            read_input_after(&p, &["\"zzz\""], "a"),
            Err(ReadRefusal::NoKey { .. })
        ));
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn a_non_numeric_field_is_a_refusal() {
        let p = tmp("text.json", "{\"branch\": \"c\"}\n");
        assert!(matches!(read_input(&p, "branch"), Err(ReadRefusal::NotANumber { .. })));
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn a_derived_stake_carries_its_inputs_and_its_arithmetic() {
        let i = ReadInput {
            path: "mesh/fluid0/gate.json".into(),
            keys: vec![],
            field: "sc".into(),
            value: 0.107,
        };
        let s = Stake::derived(
            "Sc branch (b)",
            10.0,
            std::slice::from_ref(&i),
            "the freeze's branch, 10x the census ceiling",
        );
        assert!(s.is_writable());
        assert!(s.print().contains("mesh/fluid0/gate.json"));
        assert!(s.json().contains("\"provenance\": \"derived\""));
    }

    #[test]
    fn a_typed_stake_is_unwritable_unless_it_says_it_is_the_kill() {
        let bad = Stake::typed("Sc water", 434.8, "Kestin 1978");
        assert!(!bad.is_writable());
        assert!(bad.print().contains("UNLABELLED"));
        let good = Stake::typed_kill_from_experiment(
            "Sc water",
            434.8,
            "Kestin, Sokolov & Wakeham 1978; Krynicki, Green & Sawyer 1978",
        );
        assert!(good.is_writable());
        assert!(good.json().contains("\"kill_from_experiment\": true"));
    }
}
