//! The removability gate on matrices read from a file — a front end for carriers that are not
//! this engine's own (a language model's layer states, for one: `conformance/reasoning/`).
//!
//! ```text
//! cargo run --release -p holon-closure --example admit_file -- SPEC.txt
//! ```
//!
//! It prints the MARGINAL admission (each candidate against the kept set plus every other
//! candidate) and the ORDERED one (backward elimination with a joint price), with the price
//! certificate of every call, then one line of JSON per candidate. Nothing is written.
//!
//! The file is whitespace-separated tokens; `#` starts a comment to the end of the line.
//!
//! ```text
//! budget 0.02
//! folds units 5            # or: folds timeblocks 5
//! lag 0                    # rows are items: the target row is the same row
//! horizon future           # or: increment
//! ridge order1             # or: hbond
//! null shift               # optional, any of: shift | permute SEED | derange SHIFT
//! units 2
//! unit kept R C  <R*C numbers>  target R C  <R*C numbers>      # once per unit
//! candidate NAME  block R C <numbers>  block R C <numbers>      # one block per unit
//! ```
//!
//! What it does NOT say: an increment is a held-out regression, an observation. A candidate
//! the gate prices at zero is one the target can be predicted without; a candidate it carries
//! is one the target cannot be predicted without FROM THIS DICTIONARY. Neither is a statement
//! about what an intervention would do. The price is a prediction to be realised by one.

use holon_closure::removable::{Admission, Candidate, Folds, Horizon, Mat, Nulls, Price, Question, Ridge, Swap, Unit};

struct Tokens {
    t: Vec<String>,
    i: usize,
}

impl Tokens {
    fn new(text: &str) -> Tokens {
        let t = text.lines().map(|l| l.split('#').next().unwrap_or("")).flat_map(|l| l.split_whitespace()).map(str::to_string).collect();
        Tokens { t, i: 0 }
    }
    fn peek(&self) -> Option<&str> {
        self.t.get(self.i).map(String::as_str)
    }
    fn next(&mut self, what: &str) -> Result<String, String> {
        let v = self.t.get(self.i).cloned().ok_or_else(|| format!("the file ends where {what} was expected"))?;
        self.i += 1;
        Ok(v)
    }
    fn word(&mut self, w: &str) -> Result<(), String> {
        let v = self.next(w)?;
        if v == w {
            Ok(())
        } else {
            Err(format!("expected `{w}`, found `{v}` (token {})", self.i))
        }
    }
    fn num<T: std::str::FromStr>(&mut self, what: &str) -> Result<T, String> {
        let v = self.next(what)?;
        v.parse().map_err(|_| format!("{what}: `{v}` is not a number (token {})", self.i))
    }
    fn mat(&mut self, what: &str) -> Result<Mat, String> {
        let (r, c): (usize, usize) = (self.num(what)?, self.num(what)?);
        let mut m = Mat::zeros(r, c);
        for i in 0..r {
            for j in 0..c {
                let v: f64 = self.num(what)?;
                if !v.is_finite() {
                    return Err(format!("{what}: a value that is not finite at row {i}, column {j}"));
                }
                m.set(i, j, v);
            }
        }
        Ok(m)
    }
}

/// A parsed file: the question, the nulls, the budget, the units and the dictionary.
pub struct Spec {
    pub budget: f64,
    pub q: Question,
    pub nulls: Nulls,
    pub units: Vec<Unit>,
    pub candidates: Vec<Candidate>,
}

pub fn parse(text: &str) -> Result<Spec, String> {
    let mut t = Tokens::new(text);
    let (mut budget, mut folds, mut lag, mut horizon, mut ridge) = (None, None, None, Horizon::Future, Ridge::ORDER1);
    let mut nulls = Nulls { shift: false, swap: None };
    let mut units: Vec<Unit> = Vec::new();
    let mut declared = None;
    let mut candidates = Vec::new();
    while let Some(k) = t.peek().map(str::to_string) {
        t.i += 1;
        match k.as_str() {
            "budget" => budget = Some(t.num::<f64>("budget")?),
            "lag" => lag = Some(t.num::<usize>("lag")?),
            "folds" => {
                let kind = t.next("folds")?;
                let k: usize = t.num("folds")?;
                folds = Some(match kind.as_str() {
                    "units" => Folds::Units { k },
                    "timeblocks" => Folds::TimeBlocks { k },
                    o => return Err(format!("folds: `{o}` is neither `units` nor `timeblocks`")),
                });
            }
            "horizon" => {
                horizon = match t.next("horizon")?.as_str() {
                    "future" => Horizon::Future,
                    "increment" => Horizon::Increment,
                    o => return Err(format!("horizon: `{o}` is neither `future` nor `increment`")),
                }
            }
            "ridge" => {
                ridge = match t.next("ridge")?.as_str() {
                    "order1" => Ridge::ORDER1,
                    "hbond" => Ridge::HBOND,
                    o => return Err(format!("ridge: `{o}` is neither `order1` nor `hbond`")),
                }
            }
            "null" => match t.next("null")?.as_str() {
                "shift" => nulls.shift = true,
                "permute" => nulls.swap = Some(Swap::Permute { seed: t.num("null permute")? }),
                "derange" => nulls.swap = Some(Swap::Derange { shift: t.num("null derange")? }),
                o => return Err(format!("null: `{o}` is none of `shift`, `permute`, `derange`")),
            },
            "units" => declared = Some(t.num::<usize>("units")?),
            "unit" => {
                t.word("kept")?;
                let kept = t.mat("kept")?;
                t.word("target")?;
                let target = t.mat("target")?;
                if kept.rows != target.rows {
                    return Err(format!("unit {}: kept has {} rows and target {}", units.len(), kept.rows, target.rows));
                }
                units.push(Unit { kept, target });
            }
            "candidate" => {
                let name = t.next("a candidate's name")?;
                let mut cols = Vec::new();
                while t.peek() == Some("block") {
                    t.i += 1;
                    cols.push(t.mat("block")?);
                }
                if cols.len() != units.len() {
                    return Err(format!("candidate {name}: {} blocks for {} units (declare the units first)", cols.len(), units.len()));
                }
                for (u, c) in cols.iter().enumerate() {
                    if c.rows != units[u].kept.rows {
                        return Err(format!("candidate {name}, unit {u}: {} rows against the unit's {}", c.rows, units[u].kept.rows));
                    }
                }
                if candidates.iter().any(|c: &Candidate| c.name == name) {
                    return Err(format!("candidate {name} is declared twice"));
                }
                candidates.push(Candidate::new(&name, cols));
            }
            o => return Err(format!("unknown keyword `{o}` (token {})", t.i)),
        }
    }
    if let Some(n) = declared {
        if n != units.len() {
            return Err(format!("`units {n}` declared, {} given", units.len()));
        }
    }
    let folds = folds.ok_or("no `folds` line")?;
    if folds.k() < 2 {
        return Err("folds: at least 2".into());
    }
    if units.is_empty() || candidates.is_empty() {
        return Err("no unit, or no candidate".into());
    }
    let q = Question { lag: lag.ok_or("no `lag` line")?, horizon, folds, ridge };
    Ok(Spec { budget: budget.ok_or("no `budget` line")?, q, nulls, units, candidates })
}

fn json(kind: &str, p: &Price) -> String {
    let n = |v: Option<f64>| v.map_or("null".to_string(), |x| format!("{x:.6}"));
    format!(
        "{{\"admission\":\"{kind}\",\"name\":\"{}\",\"increment\":{:.6},\"se\":{:.6},\"r2_kept\":{:.6},\"r2_with\":{:.6},\"null_shift\":{},\"null_swap\":{},\"redundant_cols\":{:?},\"verdict\":\"{:?}\"}}",
        p.name, p.increment, p.se, p.r2_kept, p.r2_with, n(p.null_shift), n(p.null_swap), p.redundant, p.verdict
    )
}

pub fn run(spec: &Spec) -> (Admission, Admission) {
    (
        Admission::marginal(&spec.units, &spec.candidates, &spec.q, &spec.nulls, spec.budget),
        Admission::ordered(&spec.units, &spec.candidates, &spec.q, &spec.nulls, spec.budget),
    )
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: admit_file SPEC.txt");
        std::process::exit(2)
    });
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("{path}: {e}");
        std::process::exit(2)
    });
    let spec = parse(&text).unwrap_or_else(|e| {
        eprintln!("{path}: {e}");
        std::process::exit(2)
    });
    let (marginal, ordered) = run(&spec);
    println!("budget {}  units {}  candidates {}", spec.budget, spec.units.len(), spec.candidates.len());
    println!("MARGINAL (each against the kept set plus every other candidate)");
    for p in &marginal.prices {
        println!("  {p}");
    }
    println!("ORDERED (backward elimination, joint price)");
    for (i, s) in ordered.order.iter().enumerate() {
        println!("  removed {}: {}", i + 1, s.price);
    }
    println!("  carried: {:?}", ordered.carried);
    for p in &ordered.prices {
        println!("  {p}");
    }
    for p in &marginal.prices {
        println!("{}", json("marginal", p));
    }
    for p in &ordered.prices {
        println!("{}", json("ordered", p));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use holon_closure::removable::{NumpyPcg64, Verdict};

    /// A carrier in the shape of the question the reasoning work asks: rows are items. A
    /// hidden WRITING coordinate `w` decides the target. `read` is a noisy reading of `w`
    /// (what a probe finds), `copy` an affine copy of `w` plus a little noise (a second
    /// direction carrying the same thing), `idle` is unrelated.
    fn carrier(seed: u64) -> String {
        let mut rng = NumpyPcg64::new(seed);
        let (units, rows) = (10, 40);
        let mut s = String::from("budget 0.02\nfolds units 5\nlag 0\nhorizon future\nridge order1\nnull permute 11\n");
        let mut blocks: [String; 4] = Default::default();
        for _ in 0..units {
            let (mut kept, mut target) = (format!("unit kept {rows} 1"), format!(" target {rows} 2"));
            let mut b: [String; 4] = [format!(" block {rows} 1"), format!(" block {rows} 1"), format!(" block {rows} 1"), format!(" block {rows} 1")];
            for _ in 0..rows {
                let (base, w, idle) = (rng.normal(), rng.normal(), rng.normal());
                kept += &format!(" {base}");
                target += &format!(" {} {}", 0.5 * base + w + 0.3 * rng.normal(), -w + 0.3 * rng.normal());
                b[0] += &format!(" {w}");
                b[1] += &format!(" {}", 2.0 * w + 1.0 + 0.05 * rng.normal());
                b[2] += &format!(" {}", w + 1.0 * rng.normal());
                b[3] += &format!(" {idle}");
            }
            s += &kept;
            s += &target;
            s.push('\n');
            for i in 0..4 {
                blocks[i] += &b[i];
            }
        }
        for (n, b) in ["write", "copy", "read", "idle"].iter().zip(blocks.iter()) {
            s += &format!("candidate {n}{b}\n");
        }
        s
    }

    #[test]
    fn two_carriers_of_one_thing_and_a_reading_of_it() {
        let spec = parse(&carrier(5)).unwrap();
        let (marginal, ordered) = run(&spec);
        // one at a time, each of the two copies looks free to remove: the other still holds it
        for n in ["write", "copy"] {
            assert_ne!(marginal.price(n).unwrap().verdict, Verdict::Carried, "{n} marginal");
        }
        // jointly, exactly one of the two survives, and it is priced at what the pair is worth
        let kept: Vec<&String> = ordered.carried.iter().filter(|n| *n == "write" || *n == "copy").collect();
        assert_eq!(kept.len(), 1, "carried {:?}", ordered.carried);
        assert!(ordered.price(kept[0]).unwrap().increment > 0.3);
        // the noisy reading and the unrelated column are removable once a carrier stands
        assert!(!ordered.carried.contains(&"read".to_string()));
        assert!(!ordered.carried.contains(&"idle".to_string()));
        // ... but the reading ALONE is carried: readable is not the same as free to remove
        let alone = Spec { candidates: spec.candidates.iter().filter(|c| c.name == "read" || c.name == "idle").cloned().collect(), ..spec };
        let (_, o2) = run(&alone);
        assert_eq!(o2.carried, vec!["read".to_string()]);
        assert!(o2.price("read").unwrap().null_swap.unwrap().abs() < 0.02, "the re-paired null stays within the budget");
    }

    #[test]
    fn the_answer_does_not_depend_on_declared_order() {
        let a = parse(&carrier(9)).unwrap();
        let mut b = parse(&carrier(9)).unwrap();
        b.candidates.reverse();
        let (_, oa) = run(&a);
        let (_, ob) = run(&b);
        let (mut ca, mut cb) = (oa.carried.clone(), ob.carried.clone());
        ca.sort();
        cb.sort();
        // the two copies are priced within rounding of each other; which one is named may differ
        let norm = |v: Vec<String>| v.into_iter().map(|n| if n == "copy" { "write".to_string() } else { n }).collect::<Vec<_>>();
        assert_eq!(norm(ca), norm(cb));
    }

    #[test]
    fn a_malformed_file_is_refused_with_its_reason() {
        for (text, needle) in [
            ("budget 0.02 folds units 5 lag 0 unit kept 2 1 1 2 target 3 1 1 2 3", "rows"),
            ("budget 0.02 folds units 5 lag 0 unit kept 1 1 1 target 1 1 nan candidate a block 1 1 0", "finite"),
            ("budget 0.02 folds units 5 lag 0 unit kept 1 1 1 target 1 1 1 candidate a", "blocks"),
            ("folds units 5 lag 0 unit kept 1 1 1 target 1 1 1 candidate a block 1 1 0", "budget"),
            ("budget 0.02 folds units 5 lag 0 wibble", "unknown keyword"),
        ] {
            let e = parse(text).err().unwrap_or_else(|| panic!("accepted: {text}"));
            assert!(e.contains(needle), "{e}");
        }
    }
}
