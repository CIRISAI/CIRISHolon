//! **Arithmetic precision as a leased resource: the ladder of rungs, and the routing rule.**
//!
//! RESOURCE_DESIGN §2.3 / D3b. A tier of arithmetic guarantees residuals down to its own
//! declared boundary and no further. A request past that boundary is **not a retry** and **not a
//! constant to edit** — it is an OVERFLOW, and it leases the next rung.
//!
//! # Why a boundary is not a constant, which is the whole design of this module
//!
//! The obvious implementation gives each rung one number. The measurements say that is wrong,
//! and by a lot. `holon-chem`'s `DD_EXPANSION_FLOOR` ships at `1e-24` — declared provisional in
//! its own doc comment, with a calibration protocol attached — and the live calibration reads:
//!
//! | atom | determinants | where the residual actually pins | vs the declared `1e-24` |
//! |---|---|---|---|
//! | Sb | 9,477 | **3.37e-15** | 9 orders short |
//! | Te | 729 | **2.97e-13** | 11 orders short |
//!
//! Two readings matter here and only the second is about the constant.
//!
//! 1. The declared floor is out by nine to eleven orders. That is calibration, not a defect —
//!    the constant's own comment says it moves when measurement contradicts it.
//! 2. **The two atoms differ from EACH OTHER by two orders, on the same rung.** Te has a
//!    degenerate ground term and pins two orders worse than Sb. So no single number is the
//!    honest boundary of that rung: the reach is a function of the rung AND the problem class.
//!
//! A lease claiming `1e-24` on a rung that delivers `2.97e-13` is a promise missed by eleven
//! orders, which is precisely what D3 forbids — a lease is a receipt for rent paid, not a
//! promise about the future. So [`Boundary`] is per-class and carries its own [`Provenance`],
//! and this module has **no floor constant in it at all**. The numbers live where they are
//! measured; the ladder only routes.
//!
//! # D8 discipline
//!
//! Reaches are integer base-10 exponents (`reach_exp: u32`, where 13 means `1e-13`), because
//! this module's outputs land in ledger entries and the ledger takes no floats. Conversion from
//! a measured residual is CONSERVATIVE and explicit: see [`Boundary::from_measured_residual`].

/// How well a rung's reach is known for a class. The distinction is load-bearing: a provisional
/// boundary relied on as if measured is a lease making a promise nobody checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    /// Observed, on this class, by a named instrument.
    Measured { instrument: &'static str },
    /// Declared and not calibrated. Must travel with the claim rather than being flattened
    /// away — §7 of the design lists exactly this for the `Dd` rung.
    Provisional { why: &'static str },
}

impl Provenance {
    pub fn is_measured(&self) -> bool {
        matches!(self, Provenance::Measured { .. })
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
    pub fn from_measured_residual(residual: f64, instrument: &'static str) -> Option<Boundary> {
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
            provenance: Provenance::Measured { instrument },
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
                    Provenance::Measured { instrument } => format!("measured by {instrument}"),
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
    fn engine_ladder() -> Ladder {
        // f64: every heavy solve pins just under 1e-10 (the scale-free expansion-floor
        // acceptance test), on both classes.
        let f64_reach = Boundary::from_measured_residual(9.8e-11, "SATURATION-3 G0").unwrap();
        // Dd: the live calibration. Te has a degenerate ground term and pins two orders worse
        // than Sb — which is the entire reason a boundary is per-class.
        let sb = Boundary::from_measured_residual(3.37e-15, "DD calibration, Sb").unwrap();
        let te = Boundary::from_measured_residual(2.97e-13, "DD calibration, Te").unwrap();
        Ladder::new(vec![
            Rung::new("f64", Some("Dd"))
                .with_boundary("Sb", f64_reach)
                .with_boundary("Te", f64_reach),
            Rung::new("Dd", None)
                .with_boundary("Sb", sb)
                .with_boundary("Te", te),
        ])
    }

    /// A measured residual becomes a CONSERVATIVE integer reach: 2.97e-13 reached 1e-12 and did
    /// not reach 1e-13.
    #[test]
    fn a_measured_residual_never_claims_a_decade_it_did_not_reach() {
        let te = Boundary::from_measured_residual(2.97e-13, "x").unwrap();
        assert_eq!(te.reach_exp, 12, "2.97e-13 claimed 1e-13, which it did not reach");
        assert!(te.covers(12) && !te.covers(13));

        let sb = Boundary::from_measured_residual(3.37e-15, "x").unwrap();
        assert_eq!(sb.reach_exp, 14);
        assert!(sb.covers(14) && !sb.covers(15));

        // An exact power of ten is reached.
        assert_eq!(
            Boundary::from_measured_residual(1e-10, "x").unwrap().reach_exp,
            10
        );
        assert!(Boundary::from_measured_residual(0.0, "x").is_none());
        assert!(Boundary::from_measured_residual(f64::NAN, "x").is_none());
    }

    /// **PLANT D3b — a request past the tier's boundary OVERFLOWS to the next rung.**
    ///
    /// Carrier asserted first: the f64 rung must genuinely be AT its boundary, or "it escalated"
    /// would be indistinguishable from a router that escalates everything.
    #[test]
    fn plant_d3b_a_request_past_the_floor_overflows_rather_than_retrying() {
        let l = engine_ladder();

        // THE CARRIER: f64 really does cover 1e-10 and really does stop there.
        match l.route("f64", "Sb", 10) {
            Routing::Satisfied { rung, boundary } => {
                assert_eq!(rung, "f64");
                assert_eq!(boundary.reach_exp, 10);
            }
            other => panic!("f64 does not reach its own measured floor: {other:?}"),
        }

        // THE PLANT: one decade past it.
        let r = l.route("f64", "Sb", 12);
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
            l.route("Dd", "Sb", 12),
            Routing::Satisfied { rung: "Dd", .. }
        ));
    }

    /// **THE CONTROL.** A request the current rung covers must NOT escalate. Without this,
    /// "overflow always" passes the plant above while destroying the cheap path.
    #[test]
    fn a_request_within_reach_is_not_escalated() {
        let l = engine_ladder();
        for asked in [1u32, 5, 9, 10] {
            match l.route("f64", "Te", asked) {
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

        let sb = l.route("Dd", "Sb", asked);
        let te = l.route("Dd", "Te", asked);

        // Sb's class reaches 1e-14, so Dd covers it.
        assert!(
            matches!(sb, Routing::Satisfied { rung: "Dd", .. }),
            "Dd should cover 1e-13 for Sb (it pinned at 3.37e-15): {sb:?}"
        );
        // Te's degenerate ground term pins at 2.97e-13, so Dd does NOT cover 1e-13 — and there
        // is no rung above, so it is refused rather than silently promised.
        assert!(
            matches!(te, Routing::Exhausted { last: "Dd", .. }),
            "Dd claimed 1e-13 for Te, which measurement says it does not reach: {te:?}"
        );
        assert_ne!(
            sb, te,
            "the two classes routed identically, so the per-class boundary is not being read — a \
             single constant would produce exactly this, and it is wrong by two orders"
        );
        assert!(te.message().contains("no rung above"));
    }

    /// A rung that makes no calibrated claim about a class REFUSES rather than borrowing another
    /// class's number.
    #[test]
    fn an_unregistered_class_is_refused_not_guessed() {
        let l = engine_ladder();
        let r = l.route("Dd", "SomeUncalibratedAtom", 13);
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

        let l = Ladder::new(vec![Rung::new("Dd", None).with_boundary("Sb", declared)]);
        let r = l.route("Dd", "Sb", 20);
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
        let measured_sb = Boundary::from_measured_residual(3.37e-15, "Sb").unwrap();
        assert_eq!(declared.reach_exp - measured_sb.reach_exp, 10);
        assert!(declared.covers(20) && !measured_sb.covers(20));
    }
}
