//! A ledger of edits to a model's weights, checked: replay chain, a conservation ring, cells
//! nobody measured, and admissibility against every row at a budget fixed first.
//!
//! ```text
//! cargo run --release -p holon-closure --example edit_ledger -- LEDGER.txt
//! ```
//!
//! Written for the kindscope series (`RATCHET/experiments/kindscope`): "what did this edit
//! change elsewhere?" The rules are this engine's ledger rules, carried over:
//!
//! 1. **A ring closes.** A RING is a partition of the output vocabulary at a fixed set of
//!    positions (classes of the next byte, or of the next token, plus an explicit "everything
//!    else"). Its cells hold the change in mean probability mass. They sum to zero by
//!    construction, so a sum beyond `ring_tolerance` is an accounting error and the ledger
//!    is REFUSED before anything is read from it.
//! 2. **A cell nobody measured is not a zero** (`closure::LedgerRow`'s served flag). A ring
//!    with an unserved cell is not summed, and an edit with an unserved cell off its target
//!    is UNRESOLVED, never admissible.
//! 3. **A separate cell is never summed.** A category that is not a class of a one-step
//!    distribution (a word class at byte level, whose probability spans several steps) is a
//!    `cell`: measured apart, judged against the budget, kept out of every ring.
//! 4. **Replay.** Each edit names the digest of the file it started from and the one it
//!    made. The first starts at the origin; each next starts where the last ended. A broken
//!    chain is REFUSED. (This program checks the chain of digests it is given. Recomputing a
//!    digest from a file is the caller's.)
//! 5. **Admissibility is against every row.** With the budget fixed before the edits (it
//!    must be the file's first statement), an edit is INADMISSIBLE if some cell off its
//!    target moved by more than the budget beyond two standard errors, ADMISSIBLE if every
//!    cell off its target is within the budget by two standard errors, UNRESOLVED otherwise.
//! 6. **The ledger telescopes.** If a `total` block gives a ring measured directly from the
//!    origin to the last file, the edits' cells must add up to it within `ring_tolerance`
//!    per edit; if not, some row was measured on a different file or position set: REFUSED.
//!
//! The file is whitespace-separated tokens; `#` starts a comment.
//!
//! ```text
//! budget 0.01                      # first, always
//! ring_tolerance 1e-6              # optional
//! origin  sha256:aaaa
//! edit E1 from sha256:aaaa to sha256:bbbb
//!   target adj                     # one or more; a ring class or a cell
//!   ring first_byte  adj 0.031 0.002  noun -0.020 0.002  other -0.011 0.002
//!   cell ADJ_word 0.050 0.004
//!   cell NOUN_word notserved
//! total ring first_byte  adj 0.040 0.002  noun -0.027 0.002  other -0.013 0.002
//! ```

use holon_closure::closure::{Channel, LedgerRow};
use std::collections::BTreeMap;
use std::fmt;

/// One cell: a change and its standard error, or nobody measured it.
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub name: String,
    row: LedgerRow,
    pub se: f64,
}

impl Cell {
    /// The change, or `None` where the cell is unserved. There is no other way out.
    pub fn read(&self) -> Option<f64> {
        self.row.read()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ring {
    pub name: String,
    pub cells: Vec<Cell>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Edit {
    pub id: String,
    pub from: String,
    pub to: String,
    pub targets: Vec<String>,
    pub rings: Vec<Ring>,
    pub cells: Vec<Cell>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ledger {
    pub budget: f64,
    pub ring_tolerance: f64,
    pub origin: String,
    pub edits: Vec<Edit>,
    pub totals: Vec<Ring>,
}

/// Why a ledger is refused. A refused ledger says nothing about any edit.
#[derive(Clone, Debug, PartialEq)]
pub enum Refusal {
    Malformed(String),
    RingOpen { edit: String, ring: String, sum: f64 },
    ChainBroken { edit: String, expected: String, found: String },
    DoesNotTelescope { ring: String, class: String, edits_sum: f64, total: f64 },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::Malformed(m) => write!(f, "malformed: {m}"),
            Refusal::RingOpen { edit, ring, sum } => write!(f, "{edit}: ring {ring} does not close (sum {sum:+.3e}); an accounting error, nothing is read"),
            Refusal::ChainBroken { edit, expected, found } => write!(f, "{edit}: starts from {found}, the chain stands at {expected}"),
            Refusal::DoesNotTelescope { ring, class, edits_sum, total } => {
                write!(f, "ring {ring}, class {class}: the edits add to {edits_sum:+.6}, the total reads {total:+.6}; a row was measured on another file or position set")
            }
        }
    }
}

/// What the ledger says of one edit.
#[derive(Clone, Debug, PartialEq)]
pub enum Standing {
    Admissible { largest_off_target: Option<(String, f64)> },
    Inadmissible { cell: String, change: f64, se: f64 },
    Unresolved { reason: String },
}

struct Tokens {
    t: Vec<String>,
    i: usize,
}

impl Tokens {
    fn peek(&self) -> Option<&str> {
        self.t.get(self.i).map(String::as_str)
    }
    fn next(&mut self, what: &str) -> Result<String, Refusal> {
        let v = self.t.get(self.i).cloned().ok_or_else(|| Refusal::Malformed(format!("the file ends where {what} was expected")))?;
        self.i += 1;
        Ok(v)
    }
    fn num(&mut self, what: &str) -> Result<f64, Refusal> {
        let v = self.next(what)?;
        match v.parse::<f64>() {
            Ok(x) if x.is_finite() => Ok(x),
            _ => Err(Refusal::Malformed(format!("{what}: `{v}` is not a finite number"))),
        }
    }
    fn word(&mut self, w: &str) -> Result<(), Refusal> {
        let v = self.next(w)?;
        if v == w {
            Ok(())
        } else {
            Err(Refusal::Malformed(format!("expected `{w}`, found `{v}`")))
        }
    }
}

const KEYWORDS: [&str; 8] = ["budget", "ring_tolerance", "origin", "edit", "target", "ring", "cell", "total"];

fn cell(t: &mut Tokens, name: String) -> Result<Cell, Refusal> {
    if t.peek() == Some("notserved") {
        t.i += 1;
        return Ok(Cell { name, row: LedgerRow::unserved(Channel::Presence), se: f64::NAN });
    }
    let v = t.num("a cell's change")?;
    let se = t.num("a cell's standard error")?;
    if se < 0.0 {
        return Err(Refusal::Malformed(format!("cell {name}: a negative standard error")));
    }
    Ok(Cell { name, row: LedgerRow::served(Channel::Presence, v).expect("finite"), se })
}

fn ring(t: &mut Tokens) -> Result<Ring, Refusal> {
    let name = t.next("a ring's name")?;
    let mut cells = Vec::new();
    while let Some(k) = t.peek() {
        if KEYWORDS.contains(&k) {
            break;
        }
        let n = t.next("a class")?;
        if cells.iter().any(|c: &Cell| c.name == n) {
            return Err(Refusal::Malformed(format!("ring {name}: class {n} twice")));
        }
        cells.push(cell(t, n)?);
    }
    if cells.len() < 2 {
        return Err(Refusal::Malformed(format!("ring {name}: a partition has at least two classes")));
    }
    Ok(Ring { name, cells })
}

pub fn parse(text: &str) -> Result<Ledger, Refusal> {
    let toks = text.lines().map(|l| l.split('#').next().unwrap_or("")).flat_map(|l| l.split_whitespace()).map(str::to_string).collect();
    let mut t = Tokens { t: toks, i: 0 };
    t.word("budget").map_err(|_| Refusal::Malformed("the budget is the file's first statement: it is fixed before the edits".into()))?;
    let budget = t.num("budget")?;
    if budget <= 0.0 {
        return Err(Refusal::Malformed("a budget above zero".into()));
    }
    let mut l = Ledger { budget, ring_tolerance: 1e-6, origin: String::new(), edits: Vec::new(), totals: Vec::new() };
    while let Some(k) = t.peek().map(str::to_string) {
        t.i += 1;
        match k.as_str() {
            "budget" => return Err(Refusal::Malformed("the budget is stated once".into())),
            "ring_tolerance" => l.ring_tolerance = t.num("ring_tolerance")?,
            "origin" => l.origin = t.next("the origin's digest")?,
            "edit" => {
                let id = t.next("an edit's id")?;
                t.word("from")?;
                let from = t.next("a digest")?;
                t.word("to")?;
                let to = t.next("a digest")?;
                if l.edits.iter().any(|e| e.id == id) {
                    return Err(Refusal::Malformed(format!("edit {id} twice")));
                }
                l.edits.push(Edit { id, from, to, targets: Vec::new(), rings: Vec::new(), cells: Vec::new() });
            }
            "target" | "ring" | "cell" => {
                let e = l.edits.last_mut().ok_or_else(|| Refusal::Malformed(format!("`{k}` before any edit")))?;
                match k.as_str() {
                    "target" => e.targets.push(t.next("a target")?),
                    "ring" => e.rings.push(ring(&mut t)?),
                    _ => {
                        let n = t.next("a cell's name")?;
                        e.cells.push(cell(&mut t, n)?);
                    }
                }
            }
            "total" => {
                t.word("ring")?;
                l.totals.push(ring(&mut t)?);
            }
            o => return Err(Refusal::Malformed(format!("unknown keyword `{o}`"))),
        }
    }
    if l.origin.is_empty() {
        return Err(Refusal::Malformed("no origin".into()));
    }
    for e in &l.edits {
        if e.targets.is_empty() {
            return Err(Refusal::Malformed(format!("edit {}: no target; every edit says what it was for", e.id)));
        }
        let names: Vec<&String> = e.rings.iter().flat_map(|r| r.cells.iter().map(|c| &c.name)).chain(e.cells.iter().map(|c| &c.name)).collect();
        for tg in &e.targets {
            if !names.contains(&tg) {
                return Err(Refusal::Malformed(format!("edit {}: target {tg} is no cell of this edit", e.id)));
            }
        }
    }
    Ok(l)
}

/// Check the ledger. `Err` refuses it whole; `Ok` gives each edit's standing, in order.
pub fn check(l: &Ledger) -> Result<Vec<(String, Standing)>, Refusal> {
    // 4. replay
    let mut at = l.origin.clone();
    for e in &l.edits {
        if e.from != at {
            return Err(Refusal::ChainBroken { edit: e.id.clone(), expected: at, found: e.from.clone() });
        }
        at = e.to.clone();
    }
    // 1. rings close (a ring with an unserved cell is not summed)
    for e in &l.edits {
        for r in &e.rings {
            if let Some(vals) = r.cells.iter().map(Cell::read).collect::<Option<Vec<f64>>>() {
                let sum: f64 = vals.iter().sum();
                if sum.abs() > l.ring_tolerance {
                    return Err(Refusal::RingOpen { edit: e.id.clone(), ring: r.name.clone(), sum });
                }
            }
        }
    }
    for r in &l.totals {
        if let Some(vals) = r.cells.iter().map(Cell::read).collect::<Option<Vec<f64>>>() {
            let sum: f64 = vals.iter().sum();
            if sum.abs() > l.ring_tolerance {
                return Err(Refusal::RingOpen { edit: "total".into(), ring: r.name.clone(), sum });
            }
        }
    }
    // 6. telescoping
    for tot in &l.totals {
        for tc in &tot.cells {
            let Some(total) = tc.read() else { continue };
            let mut sum = Some(0.0);
            for e in &l.edits {
                let c = e.rings.iter().find(|r| r.name == tot.name).and_then(|r| r.cells.iter().find(|c| c.name == tc.name));
                sum = match (sum, c.and_then(Cell::read)) {
                    (Some(s), Some(v)) => Some(s + v),
                    _ => None,
                };
            }
            if let Some(s) = sum {
                if (s - total).abs() > l.ring_tolerance * l.edits.len().max(1) as f64 {
                    return Err(Refusal::DoesNotTelescope { ring: tot.name.clone(), class: tc.name.clone(), edits_sum: s, total });
                }
            }
        }
    }
    // 5. admissibility against every row
    let mut out = Vec::new();
    for e in &l.edits {
        let off: Vec<&Cell> = e.rings.iter().flat_map(|r| r.cells.iter()).chain(e.cells.iter()).filter(|c| !e.targets.contains(&c.name)).collect();
        let mut worst: BTreeMap<&str, (f64, f64)> = BTreeMap::new();
        let mut unserved = Vec::new();
        for c in &off {
            match c.read() {
                Some(v) => {
                    worst.insert(&c.name, (v, c.se));
                }
                None => unserved.push(c.name.clone()),
            }
        }
        let over = worst.iter().filter(|(_, (v, se))| v.abs() - 2.0 * se > l.budget).max_by(|a, b| a.1 .0.abs().total_cmp(&b.1 .0.abs()));
        let standing = if let Some((n, (v, se))) = over {
            Standing::Inadmissible { cell: n.to_string(), change: *v, se: *se }
        } else if !unserved.is_empty() {
            Standing::Unresolved { reason: format!("not served: {}", unserved.join(", ")) }
        } else if let Some((n, (v, se))) = worst.iter().find(|(_, (v, se))| v.abs() + 2.0 * se > l.budget) {
            Standing::Unresolved { reason: format!("{n} moved {v:+.4} ± {se:.4}: neither within the budget nor beyond it by two standard errors") }
        } else {
            Standing::Admissible { largest_off_target: worst.iter().max_by(|a, b| a.1 .0.abs().total_cmp(&b.1 .0.abs())).map(|(n, (v, _))| (n.to_string(), *v)) }
        };
        out.push((e.id.clone(), standing));
    }
    Ok(out)
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: edit_ledger LEDGER.txt");
        std::process::exit(2)
    });
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("{path}: {e}");
        std::process::exit(2)
    });
    let ledger = match parse(&text) {
        Ok(l) => l,
        Err(r) => {
            println!("REFUSED  {r}");
            std::process::exit(1)
        }
    };
    match check(&ledger) {
        Err(r) => {
            println!("REFUSED  {r}");
            std::process::exit(1)
        }
        Ok(st) => {
            println!("budget {}  ring tolerance {}  edits {}  chain closes at {}", ledger.budget, ledger.ring_tolerance, ledger.edits.len(), ledger.edits.last().map_or(&ledger.origin, |e| &e.to));
            for (id, s) in &st {
                match s {
                    Standing::Admissible { largest_off_target: Some((n, v)) } => println!("ADMISSIBLE    {id}: largest movement off target {n} {v:+.4}"),
                    Standing::Admissible { largest_off_target: None } => println!("ADMISSIBLE    {id}: no cell off target"),
                    Standing::Inadmissible { cell, change, se } => println!("INADMISSIBLE  {id}: {cell} moved {change:+.4} ± {se:.4}"),
                    Standing::Unresolved { reason } => println!("UNRESOLVED    {id}: {reason}"),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = "budget 0.01\norigin o\n\
        edit E1 from o to a  target adj target ADJ ring fb adj 0.031 0.002 noun -0.004 0.001 other -0.027 0.002  cell ADJ 0.05 0.004\n\
        edit E2 from a to b  target noun ring fb adj 0.002 0.001 noun 0.004 0.001 other -0.006 0.001\n\
        total ring fb adj 0.033 0.002 noun 0.0 0.001 other -0.033 0.002\n";

    fn standing(text: &str, id: &str) -> Standing {
        check(&parse(text).unwrap()).unwrap().into_iter().find(|(i, _)| i == id).unwrap().1
    }

    #[test]
    fn a_closed_ledger_is_read_edit_by_edit() {
        // E1 moved "other" by 0.027 against a budget of 0.01: the side effect nobody chose to watch
        assert!(matches!(standing(GOOD, "E1"), Standing::Inadmissible { ref cell, .. } if cell == "other"));
        assert!(matches!(standing(GOOD, "E2"), Standing::Admissible { .. }));
    }

    #[test]
    fn the_target_is_not_a_side_effect_and_a_separate_cell_is_never_summed() {
        let t = "budget 0.05\norigin o\nedit E from o to a target adj target ADJ ring fb adj 0.03 0.002 noun -0.01 0.002 other -0.02 0.002 cell ADJ 9.0 0.1\n";
        // the ring closes without ADJ, and ADJ, a target, is not judged against the budget
        assert!(matches!(standing(t, "E"), Standing::Admissible { .. }));
    }

    #[test]
    fn a_ring_that_does_not_close_refuses_the_ledger() {
        let t = GOOD.replace("other -0.027", "other -0.020");
        assert!(matches!(check(&parse(&t).unwrap()), Err(Refusal::RingOpen { ref edit, .. }) if edit == "E1"));
    }

    #[test]
    fn unmeasured_is_not_zero() {
        let t = "budget 0.05\norigin o\nedit E from o to a target adj ring fb adj 0.03 0.002 noun notserved other -0.02 0.002 cell NOUN notserved\n";
        // the ring is not summed (0.03 - 0.02 would not close), and the edit cannot be admitted
        assert!(matches!(standing(t, "E"), Standing::Unresolved { ref reason } if reason.contains("noun") && reason.contains("NOUN")));
    }

    #[test]
    fn a_broken_chain_and_a_ledger_that_does_not_telescope_are_refused() {
        let t = GOOD.replace("edit E2 from a", "edit E2 from x");
        assert!(matches!(check(&parse(&t).unwrap()), Err(Refusal::ChainBroken { ref edit, .. }) if edit == "E2"));
        let t = GOOD.replace("total ring fb adj 0.033 0.002 noun 0.0 0.001 other -0.033", "total ring fb adj 0.040 0.002 noun 0.0 0.001 other -0.040");
        assert!(matches!(check(&parse(&t).unwrap()), Err(Refusal::DoesNotTelescope { ref class, .. }) if class == "adj"));
    }

    #[test]
    fn between_the_budget_and_two_standard_errors_is_unresolved() {
        let t = "budget 0.01\norigin o\nedit E from o to a target adj ring fb adj 0.012 0.002 noun -0.012 0.004 other 0.0 0.001\n";
        assert!(matches!(standing(t, "E"), Standing::Unresolved { ref reason } if reason.contains("noun")));
    }

    #[test]
    fn the_budget_comes_first_and_every_edit_names_its_target() {
        assert!(matches!(parse("origin o budget 0.01"), Err(Refusal::Malformed(ref m)) if m.contains("first")));
        assert!(matches!(parse("budget 0.01 origin o edit E from o to a ring fb a 0.0 0.0 b 0.0 0.0"), Err(Refusal::Malformed(ref m)) if m.contains("target")));
        assert!(matches!(parse("budget 0.01 origin o edit E from o to a target zz ring fb a 0.0 0.0 b 0.0 0.0"), Err(Refusal::Malformed(ref m)) if m.contains("zz")));
    }
}
