//! **Arithmetic precision as a leased resource: the ladder of rungs, and the routing rule.**
//!
//! RESOURCE_DESIGN §2.3 / D3b. A tier of arithmetic guarantees residuals down to its own
//! declared boundary and no further. A request past that boundary is **not a retry** and **not a
//! constant to edit** — it is an OVERFLOW, and it leases the next rung.
//!
//! # Why a boundary is not a constant, which is the whole design of this module
//!
//! The obvious implementation gives each rung one number. The 2026-08-30 DD calibration says
//! that is wrong, and the reason is not the one this module first wrote down.
//!
//! | atom | ground term | determinants | reached | exit |
//! |---|---|---|---|---|
//! | Sb | ⁴S (nondegenerate, refines deep) | 9,477 | **3.37e-15** | `IterationCap`, still descending |
//! | Te | ³P (degenerate, crawls) | 729 | **2.97e-13** | `IterationCap`, still descending |
//!
//! **Two orders apart ON THE SAME RUNG**, by ground-term degeneracy. So no single number is
//! that rung's honest boundary: the reach is a function of the rung AND the problem class,
//! which is why [`Boundary`] is class-keyed and this module holds **no floor constant at all**.
//!
//! # Two corrections, and this lane needed both
//!
//! **First**, the numbers are not `holon-chem`'s `DD_EXPANSION_FLOOR`. That constant is the
//! **mechanism** — the internal Gram-Schmidt acceptance guard — and its move trigger is a DD
//! solve exiting `Stagnated` above it, which has not fired. What a calibration measures is the
//! **promise**: what the rung DELIVERS. Reading one as the other conflates a floor reading with
//! a budget reading, and this lane reported exactly that conflation to the lead before the DD
//! lane corrected it.
//!
//! **Second, and it invalidated this module's first numbers: a CHECKPOINT IS NOT AN EXIT.** The
//! readings originally recorded here — `Sb 3.37e-15`, `Te 2.97e-13`, two orders apart and read
//! as a class boundary — were samples taken from a *running* sequence, and the Davidson residual
//! is **non-monotone across thick restarts**. Te's own trace makes it plain:
//!
//! | iteration | Te residual |
//! |---|---|
//! | 15 | 5.75e-13 — crossed the ask, CONVERGED |
//! | 400 | 5.53e-12 — wandered back UP |
//! | 4000 | 2.97e-13 |
//!
//! A number picked off that sequence mid-flight says where the walk happened to be, not what the
//! rung delivers. **A boundary is measured AT EXIT, under an ask that means something** — which
//! is this module's own [`Limit`] discipline extended one step, and the step it had missed.
//!
//! At the reachable ask (`1e-12`), the deliverable turns out to be **CLASS-INDEPENDENT**: both
//! degeneracy classes converge in ~15 iterations to ~6e-13 from an f64 warm start. The deep-tail
//! split (⁴S reaching 1.99e-15 at 4000 where ³P did not) is real data and **not** a calibrated
//! boundary — it may be class physics or restart-wander luck — so it carries
//! [`Provenance::Provisional`] until something measures it at exit under a deep ask.
//!
//! [`Limit`] carries the difference, because it changes the ANSWER a caller gets: a
//! budget-limited boundary moves if you buy more budget, and a floor-limited one never does —
//! only the next rung does. A lease that could not tell those apart would refuse work that a
//! larger budget would have finished.
//!
//! # D8 discipline
//!
//! Reaches are integer base-10 exponents (`reach_exp: u32`, where 13 means `1e-13`), because
//! this module's outputs land in ledger entries and the ledger takes no floats. Conversion from
//! a measured residual is CONSERVATIVE and explicit: see [`Boundary::from_measured_residual`].

/// How well a rung's reach is known for a class. The distinction is load-bearing: a provisional
/// boundary relied on as if measured is a lease making a promise nobody checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limit {
    /// The solve exhausted its declared BUDGET while still descending. The boundary is a
    /// contract under that budget, and buying more budget moves it.
    Budget,
    /// The solve STAGNATED at the tier's own arithmetic floor. More budget buys nothing here;
    /// only the next rung does.
    Floor,
    /// The solve **CONVERGED to what was asked** and stopped because it was satisfied.
    ///
    /// This is neither of the above and the distinction is not pedantry: it is a **LOWER BOUND**
    /// on the rung's reach, not its boundary. The rung reached at least this far; where it would
    /// have stopped was never tested, because nobody asked it to go further.
    ///
    /// Treating an ask-limited reading as a ceiling is the error the DD calibration nearly
    /// bequeathed to this module — both classes converge at `1e-12` in ~15 iterations, and
    /// recording that as "the Dd rung reaches 1e-12 and no further" would invent a wall out of
    /// a satisfied request.
    Ask,
}

/// How well a rung's reach is known for a class. The distinction is load-bearing: a provisional
/// boundary relied on as if measured is a lease making a promise nobody checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// Observed, on this class, by a named instrument — and WHY the observation stopped where it
    /// did, because that decides whether the boundary can be improved at all.
    Measured {
        instrument: &'static str,
        limit: Limit,
    },
    /// Declared and not calibrated. Must travel with the claim rather than being flattened
    /// away — §7 of the design lists exactly this for the `Dd` rung.
    Provisional { why: &'static str },
}

impl Provenance {
    pub fn is_measured(&self) -> bool {
        matches!(self, Provenance::Measured { .. })
    }

    /// Whether a bigger budget could move this boundary. `false` for a floor-limited
    /// measurement and for an uncalibrated declaration — in both cases spending more is not the
    /// answer, though for different reasons.
    pub fn budget_would_help(&self) -> bool {
        matches!(
            self,
            Provenance::Measured {
                limit: Limit::Budget,
                ..
            }
        )
    }

    /// Whether this reading is a LOWER BOUND rather than a boundary — the rung reached at least
    /// this far and was never asked for more. A router must not read it as a ceiling.
    pub fn is_lower_bound_only(&self) -> bool {
        matches!(
            self,
            Provenance::Measured {
                limit: Limit::Ask,
                ..
            }
        )
    }
}

/// What one rung claims it can reach for one class of problem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Boundary {
    /// The rung reaches `1e-reach_exp` for this class, and not the next decade down.
    pub reach_exp: u32,
    pub provenance: Provenance,
}

impl Boundary {
    /// Turn a MEASURED residual into a conservative integer reach.
    ///
    /// A rung that pinned at `2.97e-13` reached `1e-12` and did NOT reach `1e-13`, so its honest
    /// reach is 12. Rounding the other way would claim a decade the instrument never showed —
    /// the same "round it and hope" move `Receipt::from_measurement` refuses for rent.
    pub fn from_measured_residual(
        residual: f64,
        instrument: &'static str,
        limit: Limit,
    ) -> Option<Boundary> {
        if !(residual.is_finite() && residual > 0.0) {
            return None;
        }
        // largest k with residual <= 1e-k  =>  k = floor(-log10(residual))
        let k = (-residual.log10()).floor();
        if !(k.is_finite() && k >= 0.0) {
            return None;
        }
        Some(Boundary {
            reach_exp: k as u32,
            provenance: Provenance::Measured { instrument, limit },
        })
    }

    /// Whether this boundary covers a request for `1e-asked_exp`.
    pub fn covers(&self, asked_exp: u32) -> bool {
        self.reach_exp >= asked_exp
    }
}

/// One rung of the arithmetic ladder.
///
/// Deliberately holds NO default floor. A rung with no boundary registered for a class makes no
/// claim about that class, which is different from claiming zero and different from claiming the
/// rung's behaviour on some other class.
#[derive(Clone, Debug)]
pub struct Rung {
    pub name: &'static str,
    pub boundaries: Vec<(&'static str, Boundary)>,
    /// The rung to overflow into. `None` is the top of the ladder.
    pub next: Option<&'static str>,
}

impl Rung {
    pub fn new(name: &'static str, next: Option<&'static str>) -> Rung {
        Rung {
            name,
            boundaries: Vec::new(),
            next,
        }
    }

    pub fn with_boundary(mut self, class: &'static str, b: Boundary) -> Rung {
        self.boundaries.push((class, b));
        self
    }

    pub fn boundary_for(&self, class: &str) -> Option<Boundary> {
        self.boundaries
            .iter()
            .find(|(c, _)| *c == class)
            .map(|(_, b)| *b)
    }
}

/// Where a request should go.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Routing {
    /// This rung's claim covers the request. `provenance` travels with it so a caller can see
    /// whether it is leaning on a measurement or on a declaration.
    Satisfied {
        rung: &'static str,
        boundary: Boundary,
    },
    /// OVERFLOW — the defining case of D3b. The current rung is at its boundary and the request
    /// is past it, so the next rung is leased. Not a retry.
    Overflow {
        from: &'static str,
        to: &'static str,
        asked_exp: u32,
        from_reach_exp: u32,
    },
    /// A rung exists but makes NO claim about this class. Refused rather than guessed: the
    /// silent alternative is to reuse another class's number, which is what the Te/Sb split
    /// shows is wrong by two orders.
    NoClaim {
        rung: &'static str,
        class: &'static str,
    },
    /// The ladder ran out. Refused loudly — there is no rung above this one to lease.
    Exhausted {
        last: &'static str,
        asked_exp: u32,
    },
    /// The ask is past what this rung has been MEASURED to reach, but that measurement was a
    /// lower bound (the solve converged to what it was asked and stopped). The rung may well do
    /// it; nobody has tried. Refusing would give up on a rung that might deliver, and satisfying
    /// would invent a guarantee — so the honest answer is neither, and it names the measurement
    /// that is owed.
    Unmeasured {
        rung: &'static str,
        asked_exp: u32,
        known_at_least_exp: u32,
    },
}

impl Routing {
    pub fn message(&self) -> String {
        match self {
            Routing::Satisfied { rung, boundary } => format!(
                "{rung} covers 1e-{} ({}){}",
                boundary.reach_exp,
                match boundary.provenance {
                    Provenance::Measured {
                        instrument,
                        limit: Limit::Floor,
                    } => format!("measured by {instrument}, FLOOR-limited — more budget buys nothing"),
                    Provenance::Measured {
                        instrument,
                        limit: Limit::Budget,
                    } => format!(
                        "measured by {instrument}, BUDGET-limited — still descending when it \
                         stopped, so a larger budget moves this"
                    ),
                    Provenance::Measured {
                        instrument,
                        limit: Limit::Ask,
                    } => format!(
                        "measured by {instrument}, ASK-limited — it converged to what it was \
                         asked, so this is a LOWER BOUND and not the rung's ceiling"
                    ),
                    Provenance::Provisional { why } => format!("PROVISIONAL: {why}"),
                },
                if boundary.provenance.is_measured() {
                    ""
                } else {
                    " — this lease rests on an uncalibrated claim and its ledger entry must say so"
                }
            ),
            Routing::Overflow {
                from,
                to,
                asked_exp,
                from_reach_exp,
            } => format!(
                "OVERFLOW: 1e-{asked_exp} is past {from}'s reach of 1e-{from_reach_exp}; leasing \
                 {to}. This is not a retry and not a constant to edit."
            ),
            Routing::NoClaim { rung, class } => format!(
                "REFUSED: {rung} makes no calibrated claim about class '{class}'. Reusing another \
                 class's number is the error the Te/Sb split measures at two orders."
            ),
            Routing::Exhausted { last, asked_exp } => format!(
                "REFUSED: 1e-{asked_exp} is past the top of the ladder ({last}); there is no rung \
                 above it to lease."
            ),
            Routing::Unmeasured {
                rung,
                asked_exp,
                known_at_least_exp,
            } => format!(
                "UNMEASURED: {rung} is known to reach at least 1e-{known_at_least_exp}, but that \
                 reading is a converged-to-the-ask LOWER BOUND and nobody has asked it for \
                 1e-{asked_exp}. Neither refuse nor promise — measure it at exit under this ask."
            ),
        }
    }

    pub fn is_overflow(&self) -> bool {
        matches!(self, Routing::Overflow { .. })
    }
}

/// The ladder: rungs in ascending order of precision.
#[derive(Clone, Debug, Default)]
pub struct Ladder {
    pub rungs: Vec<Rung>,
}

impl Ladder {
    pub fn new(rungs: Vec<Rung>) -> Ladder {
        Ladder { rungs }
    }

    pub fn rung(&self, name: &str) -> Option<&Rung> {
        self.rungs.iter().find(|r| r.name == name)
    }

    /// Route a request for `1e-asked_exp` on `class`, starting at `from`.
    ///
    /// One step only — the caller leases what it is told to and asks again if that rung also
    /// overflows. That is deliberate: each rung is a separate lease with its own probe and its
    /// own ledger entry (D3), so collapsing the walk into one call would hide the leases.
    pub fn route(&self, from: &str, class: &str, asked_exp: u32) -> Routing {
        let Some(rung) = self.rung(from) else {
            return Routing::NoClaim {
                rung: "<unknown rung>",
                class: "<unknown>",
            };
        };
        let Some(b) = rung.boundary_for(class) else {
            return Routing::NoClaim {
                rung: rung.name,
                class: rung
                    .boundaries
                    .iter()
                    .find(|(c, _)| *c == class)
                    .map(|(c, _)| *c)
                    .unwrap_or("unregistered"),
            };
        };
        if b.covers(asked_exp) {
            return Routing::Satisfied {
                rung: rung.name,
                boundary: b,
            };
        }
        // A lower-bound reading is not a ceiling: the rung was never asked for more, so
        // overflowing off it would be giving up on a rung that has not refused.
        if b.provenance.is_lower_bound_only() {
            return Routing::Unmeasured {
                rung: rung.name,
                asked_exp,
                known_at_least_exp: b.reach_exp,
            };
        }
        match rung.next {
            Some(to) => Routing::Overflow {
                from: rung.name,
                to,
                asked_exp,
                from_reach_exp: b.reach_exp,
            },
            None => Routing::Exhausted {
                last: rung.name,
                asked_exp,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ladder as the engine actually measures it, built from EXIT readings rather than from
    /// declared constants or mid-flight checkpoints. NOTHING here is hard-coded as a floor.
    ///
    /// The Dd rung's entry is **class-independent** and **ask-limited**: at the reachable ask of
    /// `1e-12` both degeneracy classes converged in ~15 iterations to ~6e-13 from an f64 warm
    /// start. That is a LOWER bound — the rung reached at least there and was never asked for
    /// more — so `Limit::Ask`, and a request past it routes to `Unmeasured` rather than
    /// pretending either way.
    ///
    /// The deep-tail split is registered SEPARATELY and as `Provisional`, because it was read
    /// off a running sequence: ⁴S reached 1.99e-15 at iteration 4000 where ³P did not, and the
    /// residual is non-monotone across thick restarts, so that may be class physics or it may be
    /// restart-wander luck. Nothing routes on it until it is measured at exit under a deep ask.
    fn engine_ladder() -> Ladder {
        // f64 is FLOOR-limited: every heavy solve exits `Stagnated` at the scale-free expansion
        // floor, so more budget buys nothing and only the next rung does.
        let f64_reach =
            Boundary::from_measured_residual(9.8e-11, "SATURATION-3 G0", Limit::Floor).unwrap();
        // Dd, at the reachable ask, measured AT EXIT and identical for both classes.
        let dd = Boundary::from_measured_residual(
            6.19e-13,
            "DD confirmation at the 1e-12 ask, converged ~15 iters, both classes",
            Limit::Ask,
        )
        .unwrap();
        Ladder::new(vec![
            Rung::new("f64", Some("Dd"))
                .with_boundary("4S", f64_reach)
                .with_boundary("3P", f64_reach),
            Rung::new("Dd", None)
                .with_boundary("4S", dd)
                .with_boundary("3P", dd),
        ])
    }

    /// A measured residual becomes a CONSERVATIVE integer reach: 2.97e-13 reached 1e-12 and did
    /// not reach 1e-13.
    #[test]
    fn a_measured_residual_never_claims_a_decade_it_did_not_reach() {
        let te = Boundary::from_measured_residual(2.97e-13, "x", Limit::Budget).unwrap();
        assert_eq!(te.reach_exp, 12, "2.97e-13 claimed 1e-13, which it did not reach");
        assert!(te.covers(12) && !te.covers(13));

        let sb = Boundary::from_measured_residual(3.37e-15, "x", Limit::Budget).unwrap();
        assert_eq!(sb.reach_exp, 14);
        assert!(sb.covers(14) && !sb.covers(15));

        // An exact power of ten is reached.
        assert_eq!(
            Boundary::from_measured_residual(1e-10, "x", Limit::Floor)
                .unwrap()
                .reach_exp,
            10
        );
        assert!(Boundary::from_measured_residual(0.0, "x", Limit::Floor).is_none());
        assert!(Boundary::from_measured_residual(f64::NAN, "x", Limit::Floor).is_none());
    }

    /// **PLANT D3b — a request past the tier's boundary OVERFLOWS to the next rung.**
    ///
    /// Carrier asserted first: the f64 rung must genuinely be AT its boundary, or "it escalated"
    /// would be indistinguishable from a router that escalates everything.
    #[test]
    fn plant_d3b_a_request_past_the_floor_overflows_rather_than_retrying() {
        let l = engine_ladder();

        // THE CARRIER: f64 really does cover 1e-10 and really does stop there.
        match l.route("f64", "4S", 10) {
            Routing::Satisfied { rung, boundary } => {
                assert_eq!(rung, "f64");
                assert_eq!(boundary.reach_exp, 10);
            }
            other => panic!("f64 does not reach its own measured floor: {other:?}"),
        }

        // THE PLANT: one decade past it.
        let r = l.route("f64", "4S", 12);
        match &r {
            Routing::Overflow {
                from,
                to,
                asked_exp,
                from_reach_exp,
            } => {
                assert_eq!((*from, *to, *asked_exp, *from_reach_exp), ("f64", "Dd", 12, 10));
            }
            other => panic!(
                "a request past f64's floor did not overflow: {other:?}. Looping on f64 here is \
                 the retry-forever D3(2) forbids, and editing the constant is the other wrong \
                 answer."
            ),
        }
        assert!(r.message().contains("not a retry"));

        // And the next rung takes it — measured at exit, at the ask that means something.
        assert!(matches!(
            l.route("Dd", "4S", 12),
            Routing::Satisfied { rung: "Dd", .. }
        ));
    }

    /// **THE CONTROL.** A request the current rung covers must NOT escalate. Without this,
    /// "overflow always" passes the plant above while destroying the cheap path.
    #[test]
    fn a_request_within_reach_is_not_escalated() {
        let l = engine_ladder();
        for asked in [1u32, 5, 9, 10] {
            match l.route("f64", "3P", asked) {
                Routing::Satisfied { rung: "f64", .. } => {}
                other => panic!("1e-{asked} was escalated off f64 needlessly: {other:?}"),
            }
        }
    }

    /// **The per-class machinery still works, and is now exercised on a HYPOTHETICAL split**
    /// rather than on the DD numbers — because at the reachable ask the two classes are the
    /// same, and the deep-tail split is not calibrated.
    ///
    /// The mechanism is what is under test: two classes on one rung must be able to carry
    /// different boundaries and must never borrow each other's. What changed is that the DD
    /// calibration no longer supplies the example.
    #[test]
    fn plant_two_classes_on_one_rung_never_borrow_each_others_boundary() {
        let deep = Boundary::from_measured_residual(1.0e-15, "deep ask, at exit", Limit::Floor)
            .unwrap();
        let shallow = Boundary::from_measured_residual(1.0e-13, "deep ask, at exit", Limit::Floor)
            .unwrap();
        let l = Ladder::new(vec![Rung::new("Dd", None)
            .with_boundary("4S", deep)
            .with_boundary("3P", shallow)]);

        let a = l.route("Dd", "4S", 14);
        let b = l.route("Dd", "3P", 14);
        assert!(matches!(a, Routing::Satisfied { rung: "Dd", .. }), "{a:?}");
        assert!(matches!(b, Routing::Exhausted { last: "Dd", .. }), "{b:?}");
        assert_ne!(
            a, b,
            "the two classes routed identically, so the per-class boundary is not being read — \
             a single constant would produce exactly this"
        );
    }

    /// **THE CORRECTION, as a test: an ask-limited reading is a LOWER BOUND, never a ceiling.**
    ///
    /// The DD confirmation converged to what it was asked and stopped. Recording that as "the
    /// rung reaches 1e-12 and no further" would invent a wall out of a satisfied request, and
    /// routing a deeper ask off the rung would give up on one that has not refused.
    #[test]
    fn an_ask_limited_reading_routes_to_unmeasured_rather_than_overflowing() {
        let l = engine_ladder();

        // Within the measured reach: satisfied, and flagged as a lower bound.
        match l.route("Dd", "3P", 12) {
            Routing::Satisfied { boundary, .. } => {
                assert!(boundary.provenance.is_lower_bound_only());
                assert!(!boundary.provenance.budget_would_help());
            }
            other => panic!("{other:?}"),
        }

        // PAST it: neither promised nor refused.
        let r = l.route("Dd", "4S", 15);
        match &r {
            Routing::Unmeasured {
                rung,
                asked_exp,
                known_at_least_exp,
            } => {
                assert_eq!((*rung, *asked_exp, *known_at_least_exp), ("Dd", 15, 12));
            }
            other => panic!(
                "a request past an ask-limited reading was resolved as {other:?}. Exhausted \
                 would give up on a rung that never refused; Satisfied would invent a guarantee."
            ),
        }
        assert!(r.message().contains("LOWER BOUND") && r.message().contains("measure it at exit"));
    }

    /// The deep-tail split is real data and NOT a calibrated boundary, so it stays Provisional
    /// and says so wherever it is cited.
    #[test]
    fn the_deep_tail_split_is_provisional_until_measured_at_exit() {
        let split = Boundary {
            reach_exp: 14, // 4S reached 1.99e-15 at iteration 4000
            provenance: Provenance::Provisional {
                why: "read at an iteration CHECKPOINT of a non-monotone sequence, not at exit; \
                      may be class physics or restart-wander luck",
            },
        };
        assert!(!split.provenance.is_measured());
        assert!(!split.provenance.is_lower_bound_only());
        let l = Ladder::new(vec![Rung::new("Dd", None).with_boundary("4S", split)]);
        let m = l.route("Dd", "4S", 13).message();
        assert!(m.contains("PROVISIONAL") && m.contains("ledger entry must say so"), "{m}");
    }

    /// Keying by GROUND TERM rather than by element is what makes a calibration reusable: a
    /// newly-solved atom sharing a class inherits the boundary already measured on it.
    ///
    /// At the reachable ask the two classes agree, so this checks the SHARING half here and
    /// leaves the never-borrow-across-classes half to
    /// [`plant_two_classes_on_one_rung_never_borrow_each_others_boundary`], which supplies a
    /// split the DD calibration no longer does.
    #[test]
    fn a_boundary_is_shared_by_class() {
        let l = engine_ladder();
        // Within the measured reach, both classes are satisfied and identically so — one
        // calibration serves every atom of either class, no new entry needed.
        let a = l.route("Dd", "4S", 12);
        let b = l.route("Dd", "3P", 12);
        assert!(matches!(a, Routing::Satisfied { rung: "Dd", .. }), "{a:?}");
        assert_eq!(a, b, "the two classes disagreed at the reachable ask, where they were \
                          measured to agree");
    }

    /// A rung that makes no calibrated claim about a class REFUSES rather than borrowing another    /// A rung that makes no calibrated claim about a class REFUSES rather than borrowing another
    /// class's number.
    #[test]
    fn an_unregistered_class_is_refused_not_guessed() {
        let l = engine_ladder();
        let r = l.route("Dd", "2D", 13);
        assert!(matches!(r, Routing::NoClaim { rung: "Dd", .. }), "{r:?}");
        assert!(r.message().contains("no calibrated claim"));
    }

    /// **A PROVISIONAL boundary stays visibly provisional.** §7 of the design records that `Dd`'s
    /// declared floor is uncalibrated; a lease resting on it must say so rather than reading it
    /// as measured.
    #[test]
    fn a_provisional_boundary_cannot_pass_as_measured() {
        let declared = Boundary {
            reach_exp: 24, // the shipped DD_EXPANSION_FLOOR, 1e-24
            provenance: Provenance::Provisional {
                why: "declared from an eps argument; calibration protocol not yet run",
            },
        };
        assert!(!declared.provenance.is_measured());

        let l = Ladder::new(vec![Rung::new("Dd", None).with_boundary("4S", declared)]);
        let r = l.route("Dd", "4S", 20);
        match &r {
            Routing::Satisfied { boundary, .. } => {
                assert!(!boundary.provenance.is_measured());
                assert!(
                    r.message().contains("PROVISIONAL")
                        && r.message().contains("ledger entry must say so"),
                    "a provisional claim was reported as if measured: {}",
                    r.message()
                );
            }
            other => panic!("unexpected: {other:?}"),
        }

        // And the finding that motivates all of this: the declared 1e-24 is contradicted by
        // measurement by NINE orders on Sb and ELEVEN on Te. A lease promising 1e-24 on a rung
        // that delivers 2.97e-13 is exactly the promise-about-the-future D3 forbids.
        let measured_sb =
            Boundary::from_measured_residual(3.37e-15, "Sb (4S)", Limit::Budget).unwrap();
        assert_eq!(declared.reach_exp - measured_sb.reach_exp, 10);
        assert!(declared.covers(20) && !measured_sb.covers(20));
    }
}
