//! Gates: a verdict, every failing leg, and the work that was done to reach it.
//!
//! # What it declares
//!
//! **A gate names EVERY failing leg** (M-FIRST-VIOLATION-ONLY). CT-1's boundedness gate
//! walked three classes in a fixed order, returned at the first violation, and reported a
//! one-kT dip while a class further down the list ended at −20.7 hartree. A [`Gate`] here
//! cannot do that: legs are pushed, never returned from, and [`Gate::verdict`] is computed
//! over the whole list. [`Gate::failing_legs`] is the whole list of failures and
//! [`Gate::worst_first`] puts the worst value beside the first, which is the other half of
//! that misfit's rule.
//!
//! **Zero work is VOID and never PASS** (M-VACUOUS-SUCCESS). A gate that checked nothing
//! passes trivially, and a suite of them is green for the wrong reason.
//!
//! **A BRANCH is an answer, not a failure.** A freeze that pre-commits branches (a), (b),
//! (c) reads one of them; the gate carries the letter and still prints every failing leg,
//! because which legs failed is how the reader checks that the branch is the right one.
//!
//! # What it refuses
//!
//! * A gate with zero work is VOID whatever its legs say.
//! * A gate voided by name stays VOID; a later passing leg cannot lift it.
//! * `Gate::json` writes `null` for a non-finite leg value rather than the token `NaN`,
//!   which is not JSON and would make the record unparseable by the thing that reads it.

use crate::record::num;

/// What a gate concluded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    /// The freeze's pre-committed branch that was read.
    Branch(char),
    /// The gate could not be run, or the sector it acts on was empty.
    Void,
}

impl Verdict {
    pub fn name(&self) -> String {
        match self {
            Verdict::Pass => "PASS".to_string(),
            Verdict::Fail => "FAIL".to_string(),
            Verdict::Branch(c) => format!("BRANCH({c})"),
            Verdict::Void => "VOID".to_string(),
        }
    }

    /// Does this verdict admit whatever it gates? Only PASS and a branch do; a reader that
    /// treats VOID as a soft pass is the failure this method exists to prevent.
    pub fn admits(&self) -> bool {
        matches!(self, Verdict::Pass | Verdict::Branch(_))
    }
}

/// One leg of a gate: what was checked, whether it held, and the value it held at.
#[derive(Clone, Debug, PartialEq)]
pub struct Leg {
    pub name: String,
    pub pass: bool,
    /// The number the leg was decided on, where it has one. Carried so the worst violation
    /// can be reported beside the first (M-FIRST-VIOLATION-ONLY).
    pub value: Option<f64>,
}

/// One gate.
#[derive(Clone, Debug, PartialEq)]
pub struct Gate {
    pub id: String,
    pub legs: Vec<Leg>,
    /// How many checks were actually performed. Zero is VOID.
    pub work: u64,
    pub detail: String,
    branch: Option<char>,
    voided: Option<String>,
}

impl Gate {
    pub fn new(id: impl Into<String>) -> Gate {
        Gate {
            id: id.into(),
            legs: Vec::new(),
            work: 0,
            detail: String::new(),
            branch: None,
            voided: None,
        }
    }

    /// A leg with no number behind it.
    pub fn leg(mut self, name: impl Into<String>, pass: bool) -> Gate {
        self.legs.push(Leg { name: name.into(), pass, value: None });
        self
    }

    /// A leg decided on a number, which is carried so the worst can be found later.
    pub fn leg_at(mut self, name: impl Into<String>, pass: bool, value: f64) -> Gate {
        self.legs.push(Leg { name: name.into(), pass, value: Some(value) });
        self
    }

    /// How many checks were done. A gate that never sets this is VOID.
    pub fn work(mut self, n: u64) -> Gate {
        self.work = n;
        self
    }

    pub fn detail(mut self, d: impl Into<String>) -> Gate {
        self.detail = d.into();
        self
    }

    /// The freeze's pre-committed branch this gate read.
    pub fn branch(mut self, letter: char) -> Gate {
        self.branch = Some(letter);
        self
    }

    /// Void the gate by name — the sector was empty, the instrument refused, the arm did
    /// not run. Nothing lifts it afterwards.
    pub fn void(mut self, why: impl Into<String>) -> Gate {
        self.voided = Some(why.into());
        self
    }

    pub fn verdict(&self) -> Verdict {
        if self.voided.is_some() || self.work == 0 {
            return Verdict::Void;
        }
        if let Some(c) = self.branch {
            return Verdict::Branch(c);
        }
        if self.legs.iter().any(|l| !l.pass) {
            Verdict::Fail
        } else {
            Verdict::Pass
        }
    }

    /// Why the gate was voided by name, if it was. `None` when a zero work count voided it,
    /// which the work column already says.
    pub fn void_reason(&self) -> Option<&str> {
        self.voided.as_deref()
    }

    /// Every failing leg, in the order they were pushed.
    pub fn failing_legs(&self) -> Vec<&Leg> {
        self.legs.iter().filter(|l| !l.pass).collect()
    }

    /// Every failing leg, worst value first, then the ones with no value in push order.
    /// M-FIRST-VIOLATION-ONLY's second half: the worst value beside the first.
    pub fn worst_first(&self) -> Vec<&Leg> {
        let mut f = self.failing_legs();
        f.sort_by(|a, b| {
            let (x, y) = (a.value.map(f64::abs), b.value.map(f64::abs));
            match (x, y) {
                (Some(p), Some(q)) => q.partial_cmp(&p).unwrap_or(core::cmp::Ordering::Equal),
                (Some(_), None) => core::cmp::Ordering::Less,
                (None, Some(_)) => core::cmp::Ordering::Greater,
                (None, None) => core::cmp::Ordering::Equal,
            }
        });
        f
    }

    /// The console line: the verdict, the work, and EVERY failing leg with its value.
    pub fn line(&self) -> String {
        let mut s = format!(
            "{:<10} {:<10} [{} checks, {} legs]  {}",
            self.id,
            self.verdict().name(),
            self.work,
            self.legs.len(),
            self.detail
        );
        if let Some(w) = &self.voided {
            s.push_str(&format!("\n           VOID: {w}"));
        }
        let f = self.worst_first();
        if !f.is_empty() {
            s.push_str(&format!("\n           FAILING LEGS ({}), worst first:", f.len()));
            for l in f {
                match l.value {
                    Some(v) => s.push_str(&format!("\n             {} = {v:.6e}", l.name)),
                    None => s.push_str(&format!("\n             {}", l.name)),
                }
            }
        }
        s
    }

    /// The gate as one JSON object, ready to be a value in a record.
    pub fn json(&self) -> String {
        let legs = self
            .legs
            .iter()
            .map(|l| {
                format!(
                    "{{\"leg\": {:?}, \"pass\": {}, \"value\": {}}}",
                    l.name,
                    l.pass,
                    l.value.map(num).unwrap_or_else(|| "null".to_string())
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let failing = self
            .worst_first()
            .iter()
            .map(|l| format!("{:?}", l.name))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{{\"verdict\": {:?}, \"work\": {}, \"branch\": {}, \"void_reason\": {}, \
             \"legs\": [{legs}], \"failing_legs_worst_first\": [{failing}], \"detail\": {:?}}}",
            self.verdict().name(),
            self.work,
            self.branch.map(|c| format!("{:?}", c.to_string())).unwrap_or_else(|| "null".to_string()),
            self.voided.as_ref().map(|w| format!("{w:?}")).unwrap_or_else(|| "null".to_string()),
            self.detail
        )
    }
}

/// Every gate of one phase, and whether the phase admits what follows it.
#[derive(Clone, Debug, Default)]
pub struct Report {
    gates: Vec<Gate>,
}

impl Report {
    pub fn new() -> Report {
        Report { gates: Vec::new() }
    }

    /// Record a gate and print its line. Returns the verdict so a caller can branch on it
    /// without asking twice.
    pub fn gate(&mut self, g: Gate) -> Verdict {
        let v = g.verdict();
        println!("{}", g.line());
        self.gates.push(g);
        v
    }

    pub fn gates(&self) -> &[Gate] {
        &self.gates
    }

    /// Every gate that does not admit — FAIL and VOID alike, named with its failing legs.
    pub fn refusals(&self) -> Vec<String> {
        self.gates
            .iter()
            .filter(|g| !g.verdict().admits())
            .map(|g| {
                let legs: Vec<String> = g.worst_first().iter().map(|l| l.name.clone()).collect();
                format!(
                    "{}: {} — {}{}",
                    g.id,
                    g.verdict().name(),
                    g.void_reason().unwrap_or(&g.detail),
                    if legs.is_empty() { String::new() } else { format!(" [{}]", legs.join(" | ")) }
                )
            })
            .collect()
    }

    pub fn admits(&self) -> bool {
        self.gates.iter().all(|g| g.verdict().admits())
    }

    /// The gates as one JSON object keyed by gate id.
    pub fn json(&self) -> String {
        format!(
            "{{{}}}",
            self.gates
                .iter()
                .map(|g| format!("{:?}: {}", g.id, g.json()))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gate_with_zero_work_is_void_and_never_pass() {
        let g = Gate::new("G0").leg("a leg that held", true);
        assert_eq!(g.verdict(), Verdict::Void);
        assert!(!g.verdict().admits());
    }

    /// M-FIRST-VIOLATION-ONLY: every failing leg, not the first one met.
    #[test]
    fn a_gate_names_every_failing_leg_and_puts_the_worst_first() {
        let g = Gate::new("G1")
            .work(3)
            .leg_at("O-O bounded", true, 0.0)
            .leg_at("H-O bounded", false, -0.41)
            .leg_at("H-H bounded", false, -20.7);
        assert_eq!(g.verdict(), Verdict::Fail);
        assert_eq!(g.failing_legs().len(), 2, "both, not the first");
        let worst = g.worst_first();
        assert_eq!(worst[0].name, "H-H bounded", "the worst value comes first");
        assert_eq!(worst[1].name, "H-O bounded");
    }

    #[test]
    fn a_branch_is_an_answer_and_still_prints_its_failing_legs() {
        let g = Gate::new("S")
            .work(1)
            .branch('c')
            .leg_at("Sc >= 10", false, 0.20)
            .detail("no closure of a pair reaches Sc >= 10 either");
        assert_eq!(g.verdict(), Verdict::Branch('c'));
        assert!(g.verdict().admits(), "a branch is a reading the freeze pre-committed");
        assert_eq!(g.failing_legs().len(), 1);
        assert!(g.line().contains("Sc >= 10"));
    }

    #[test]
    fn a_named_void_is_not_lifted_by_a_passing_leg() {
        let g = Gate::new("W")
            .work(100)
            .leg("everything held", true)
            .void("no closure existed in the sector the gate acts on");
        assert_eq!(g.verdict(), Verdict::Void);
        assert_eq!(g.void_reason(), Some("no closure existed in the sector the gate acts on"));
    }

    #[test]
    fn the_report_names_every_refusal_and_admits_only_when_all_do() {
        let mut r = Report::new();
        r.gate(Gate::new("G0").work(1).leg("held", true));
        assert!(r.admits());
        r.gate(Gate::new("G1").work(1).leg_at("did not hold", false, 3.0));
        assert!(!r.admits());
        assert_eq!(r.refusals().len(), 1);
        assert!(r.refusals()[0].contains("did not hold"));
    }

    #[test]
    fn a_non_finite_leg_value_writes_null_and_not_nan() {
        let g = Gate::new("G").work(1).leg_at("infinite", false, f64::NAN);
        assert!(g.json().contains("\"value\": null"), "{}", g.json());
        assert!(!g.json().contains("NaN"));
    }
}
