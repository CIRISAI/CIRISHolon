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
//! # The correction that matters, and this lane got it wrong first
//!
//! These numbers are NOT `holon-chem`'s `DD_EXPANSION_FLOOR`, and reading them as a
//! contradiction of it — as this lane initially reported to the lead — conflates two different
//! facts:
//!
//! * `DD_EXPANSION_FLOOR` is the **mechanism**: the internal Gram-Schmidt acceptance guard. Its
//!   move trigger is a DD solve exiting **`Stagnated`** with its residual above the floor, and
//!   that has **NOT fired**.
//! * `3.37e-15` / `2.97e-13` are the **promise**: what the rung DELIVERS per class *under a
//!   declared budget*. Both solves exited `IterationCap` **while still descending** — they ran
//!   out of iterations, not of arithmetic.
//!
//! Capped-while-descending is a budget reading; stagnated-at-the-floor is a floor reading. They
//! are different facts, and the distinction is the one `SolveExit` exists to carry — the same
//! discriminator G1 relied on when every heavy solve exited `Stagnated`. The deliverable
//! boundary is a LEASE CONTRACT and lives here, per-class with provenance; the mechanism guard
//! stays in `scalar.rs` where the arithmetic is.
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

    /// The ladder as the engine actually measures it, built from residuals rather than from
    /// declared constants. NOTHING here is hard-coded as a floor.
    ///
    /// **The class key is the GROUND TERM, not the element.** That is the lead's calibration
    /// finding stated precisely: Sb reached 3.37e-15 on a `⁴S` ground term (degeneracy 4) and Te
    /// pins at 2.97e-13 on a `³P` (degeneracy 9), at the same budget. The discriminator is the
    /// degeneracy of the ground term, so keying by element would need a fresh calibration for
    /// every atom while keying by term lets a newly-solved atom inherit a boundary already
    /// measured on its class — and, more importantly, stops a `³P` atom silently borrowing a
    /// `⁴S` number.
    fn engine_ladder() -> Ladder {
        // f64: every heavy solve pins just under 1e-10 (the scale-free expansion-floor
        // acceptance test), on both classes.
        // f64 is FLOOR-limited: every heavy solve exits `Stagnated` at the scale-free
        // expansion floor, so more budget buys nothing and only the next rung does.
        let f64_reach =
            Boundary::from_measured_residual(9.8e-11, "SATURATION-3 G0", Limit::Floor).unwrap();
        // Dd: the live calibration, keyed by ground term.
        // Dd is BUDGET-limited: both calibration solves exited `IterationCap` while STILL
        // DESCENDING, so these are contracts under a declared budget rather than the rung's
        // arithmetic floor — buying more budget moves them.
        let s4 = Boundary::from_measured_residual(3.37e-15, "DD calibration, Sb (4S)", Limit::Budget)
            .unwrap();
        let p3 = Boundary::from_measured_residual(2.97e-13, "DD calibration, Te (3P)", Limit::Budget)
            .unwrap();
        Ladder::new(vec![
            Rung::new("f64", Some("Dd"))
                .with_boundary("4S", f64_reach)
                .with_boundary("3P", f64_reach),
            Rung::new("Dd", None)
                .with_boundary("4S", s4)
                .with_boundary("3P", p3),
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

        // And the next rung takes it.
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

    /// **PLANT — THE BOUNDARY IS PER-CLASS, so one constant cannot serve the rung.**
    ///
    /// The lead's calibration note as a test: the SAME request routes differently for Te and Sb
    /// on the SAME rung. A hard-coded `DD_EXPANSION_FLOOR` cannot produce both answers, which is
    /// why this module has no floor constant in it.
    #[test]
    fn plant_the_same_request_routes_differently_for_two_classes() {
        let l = engine_ladder();
        let asked = 13; // 1e-13

        let sb = l.route("Dd", "4S", asked);
        let te = l.route("Dd", "3P", asked);

        // Sb's class reaches 1e-14, so Dd covers it.
        assert!(
            matches!(sb, Routing::Satisfied { rung: "Dd", .. }),
            "Dd should cover 1e-13 for the 4S class (Sb pinned at 3.37e-15): {sb:?}"
        );
        // Te's degenerate ground term pins at 2.97e-13, so Dd does NOT cover 1e-13 — and there
        // is no rung above, so it is refused rather than silently promised.
        assert!(
            matches!(te, Routing::Exhausted { last: "Dd", .. }),
            "Dd claimed 1e-13 for the 3P class, which measurement says it does not reach: {te:?}"
        );
        assert_ne!(
            sb, te,
            "the two classes routed identically, so the per-class boundary is not being read — a \
             single constant would produce exactly this, and it is wrong by two orders"
        );
        assert!(te.message().contains("no rung above"));
    }

    /// Keying by GROUND TERM rather than by element is what makes a calibration reusable: a
    /// newly-solved atom sharing a class inherits the boundary already measured on it, and — the
    /// half that matters — an atom of a DIFFERENT class cannot borrow it.
    #[test]
    fn a_boundary_is_shared_by_class_and_never_across_classes() {
        let l = engine_ladder();
        // Two different atoms, same ⁴S class: one calibration serves both, no new entry needed.
        let for_sb = l.route("Dd", "4S", 14);
        let for_another_4s_atom = l.route("Dd", "4S", 14);
        assert_eq!(for_sb, for_another_4s_atom);
        assert!(matches!(for_sb, Routing::Satisfied { rung: "Dd", .. }));

        // The ³P class does NOT inherit it, which is the two-order error being prevented.
        assert!(
            !matches!(l.route("Dd", "3P", 14), Routing::Satisfied { .. }),
            "a 3P request was satisfied by a boundary measured on 4S"
        );
    }

    /// **The correction this lane got wrong first**, as a test: a BUDGET-limited boundary and a
    /// FLOOR-limited one are different promises, and a caller's next move differs.
    ///
    /// Both DD calibration solves exited `IterationCap` while still descending, so their reach
    /// is a contract under a budget — buy more and it moves. f64's reach is stagnation at the
    /// arithmetic floor — buy all the budget in the world and it does not.
    #[test]
    fn a_budget_limited_boundary_is_a_different_promise_from_a_floor_limited_one() {
        let l = engine_ladder();

        let f64_b = l.rung("f64").unwrap().boundary_for("3P").unwrap();
        assert!(!f64_b.provenance.budget_would_help(),
            "f64's floor was reported as improvable by budget; it stagnates, it does not run out");

        let dd_b = l.rung("Dd").unwrap().boundary_for("3P").unwrap();
        assert!(
            dd_b.provenance.budget_would_help(),
            "the Dd boundary was reported as a hard floor. Both calibration solves exited \
             IterationCap WHILE STILL DESCENDING — treating that as the rung's arithmetic limit \
             would refuse work a larger budget would have finished"
        );

        // And the message says which, because it changes what the caller should do next.
        let m = l.route("Dd", "3P", 12).message();
        assert!(m.contains("BUDGET-limited") && m.contains("larger budget moves this"), "{m}");
        let m2 = l.route("f64", "3P", 10).message();
        assert!(m2.contains("FLOOR-limited") && m2.contains("more budget buys nothing"), "{m2}");
    }

    /// A rung that makes no calibrated claim about a class REFUSES rather than borrowing another
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
