//! THE CHANNEL LEDGER — OBJECT.md design rule 10, as DECLARATIONS.
//!
//! *The interaction is a ledger of channels, each with a derived rate, and precision is an
//! allocator over them.* This module writes that sentence down as data the engine can be
//! checked against. It evaluates NOTHING. Every energy the force law computes is still
//! computed by the sector loops in `sim.rs`, in the order they always ran, and the gate on
//! this module is bit-identity: a `Sim` built before it and one built after it agree in
//! every byte (`tests/channel_ledger.rs`).
//!
//! # Sectors are not channels
//!
//! The force law is organised by SECTOR — the near pair table, the far pair tail, the
//! three-body tables, the many-body clusters, the field — and each sector has its own
//! ledger row (`Row`). The physics is organised by CHANNEL — five pushes, each with a
//! derived decay law (`Channel`). These are two different partitions of one energy, and
//! they map many-to-many: the pair table carries pair dispersion AND exchange folded into
//! one number; the field row carries channel 1 at molecule level only. `Row::carries`
//! states the map honestly, per row, with the carriage marked, so that a reader of the
//! ledger knows which channels a row's number is made of and which channels have no row
//! of their own yet (induction has none — FIELD-2 is named, not built).
//!
//! # The five, with their rates DERIVED and not fitted
//!
//! | channel | kind (the taxonomy's word) | arity | rate | shape | prior art |
//! |---|---|---|---|---|---|
//! | the field | Circumstances | 2 | `R⁻¹` | a sum | the multipole series (Stone) |
//! | induction | Structure | 2 | `R⁻⁴` | a fixed point | the multipole series (Stone) |
//! | pair dispersion | Process | 2 | `R⁻⁶` | a sum | London 1930 |
//! | three-body dispersion | Rules | 3 | `R⁻⁹` | a sum | Axilrod–Teller–Muto 1943 |
//! | exchange | Identity | 2 | exponential | a solve | Pauli; the exact core |
//!
//! The rates are the leading orders of standard intermolecular theory (SAPT: Jeziorski,
//! Moszynski, Szalewicz 1994) and they are what a residual is ASSIGNED by — EMBED-2's
//! measured `9.34` named channel 4 before anyone looked for it. So a tail whose measured
//! exponent disagrees with its channel's assigned one is a finding, and `ExponentReading`
//! reports it beside the fit rather than letting the fit stand in for the law. The
//! refusal built on that reading (`FarSector::require_assigned_exponent`) is OPT-IN: no
//! banked scene changes a bit because this module exists.
//!
//! # One allocator
//!
//! "How far does this channel reach at this budget" was answered in three dialects — a
//! closed form on a pure power (`TailModel::radius_for_budget`), a bisection on an
//! interpolant (`Sim::derive_pair_cutoff`), and a declared measured reach per class and
//! order (`SurfaceRegistry`). `reach_for_budget` is the one function the first two now
//! call, with the third as its `Declared` arm, and its arithmetic is the arithmetic those
//! callers had — so every radius is identical to the bit and the gate can say so.
//!
//! # What this module is NOT
//!
//! Not channel 4's far side (the harvested `−C/R⁹` past the three-body table's reach), not
//! channel 2's fixed point (FIELD-2), not a change to any sum's order. Those are physics
//! and live behind their own freezes. This is the notation, and the notation's receipt.

use crate::longrange::TailFit;

/// The taxonomy's word for what a channel changes about the thing it acts on
/// (`CIRISOntology/Core/WrongKind.lean`, plain names; the five kinds ABOUT THE THING,
/// `PHILOLOGY_BACKPASS.md` Backpass II/III).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// What is around the thing changed; the thing did not.
    Circumstances,
    /// How the thing is built changed, in answer to what is around it.
    Structure,
    /// What the thing does together with one partner.
    Process,
    /// What holds three of them in line — no pair can see it.
    Rules,
    /// What the thing is; which of two is which.
    Identity,
}

/// How a channel is evaluated. Three shapes, and no record collapses them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Kernel × tuple, summed. Channels 1, 3, 4.
    Sum,
    /// Iterated to self-consistency. Channel 2 (FIELD-2, not built).
    FixedPoint,
    /// The exact core, inside the channel's reach. Channel 5.
    Solve,
}

/// A channel's DERIVED decay law — the leading order of its standard expansion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rate {
    /// `C / R^p` with `p` the leading power.
    Power(f64),
    /// Overlap decay: gone by ~3 Å for closed-shell fragments.
    Exponential,
}

impl Rate {
    /// The leading power, or `None` for an exponential.
    pub fn power(&self) -> Option<f64> {
        match self {
            Rate::Power(p) => Some(*p),
            Rate::Exponential => None,
        }
    }
}

/// The five channels of the ledger, in rate order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelId {
    /// The standing multipoles of everything else, counted once (FIELD-1).
    Field,
    /// The field's fixed point: the fragment reshaped in answer (FIELD-2, named).
    Induction,
    /// Correlated fluctuation of two, invisible to either alone.
    PairDispersion,
    /// The triple's correlation no pair sees — the founding parity shape in matter.
    ThreeBody,
    /// Which electron is which, and the sign of saying so.
    Exchange,
}

/// One channel of the ledger — a record, not a computation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Channel {
    pub id: ChannelId,
    pub kind: Kind,
    /// How many fragments must be present for the channel to exist.
    pub arity: u8,
    pub rate: Rate,
    pub shape: Shape,
    /// The receipt column a transfer on this channel posts to, where one exists. The
    /// field's charge-assignment transitions post to `work.field`; the others are
    /// conservative inside the force law and post nothing (their energy is held by the
    /// potential, which is the ledger's rule for every conservative term).
    pub receipt: Option<&'static str>,
    /// Credited before the sentence, per house rule.
    pub prior_art: &'static str,
}

/// THE FIVE. Order is rate order, which is also the order of reach at any budget.
pub const CHANNELS: [Channel; 5] = [
    Channel {
        id: ChannelId::Field,
        kind: Kind::Circumstances,
        arity: 2,
        rate: Rate::Power(1.0),
        shape: Shape::Sum,
        receipt: Some("work.field"),
        prior_art: "the multipole series (Stone, The Theory of Intermolecular Forces); FIELD-1",
    },
    Channel {
        id: ChannelId::Induction,
        kind: Kind::Structure,
        arity: 2,
        rate: Rate::Power(4.0),
        shape: Shape::FixedPoint,
        receipt: None,
        prior_art: "the multipole series (Stone); the self-consistent fixed point of EMBED-1 G4; FIELD-2 (named, not built)",
    },
    Channel {
        id: ChannelId::PairDispersion,
        kind: Kind::Process,
        arity: 2,
        rate: Rate::Power(6.0),
        shape: Shape::Sum,
        receipt: None,
        prior_art: "London 1930; inside the exact pair curve here (holon-chem::pair)",
    },
    Channel {
        id: ChannelId::ThreeBody,
        kind: Kind::Rules,
        arity: 3,
        rate: Rate::Power(9.0),
        shape: Shape::Sum,
        receipt: None,
        prior_art: "Axilrod–Teller–Muto 1943; harvested from exact solves, EMBED2_RESULTS.md (C = 8.52 Ha·bohr⁹, linear HF triple)",
    },
    Channel {
        id: ChannelId::Exchange,
        kind: Kind::Identity,
        arity: 2,
        rate: Rate::Exponential,
        shape: Shape::Solve,
        receipt: None,
        prior_art: "Pauli; the exact determinant core (holon-chem::lanes, the k = 2 case); CIRISOntology Core/ExchangeSign.lean",
    },
];

impl ChannelId {
    pub fn record(self) -> &'static Channel {
        &CHANNELS[self as usize]
    }
}

/// How a ledger row carries a channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Carriage {
    /// The row IS the channel's number, at the level named.
    Whole,
    /// The channel is inside the row's number, FOLDED with others — the row cannot say
    /// how much of it is this channel.
    Folded,
}

/// The ledger's rows — every term `Sim::energy` sums, IN THE ORDER IT SUMS THEM.
///
/// The order is load-bearing: floating-point addition is not associative, and
/// `Sim::energy` is the number every drift gate and every replay fingerprint was banked
/// against. `Row::ALL` is that order, and `Sim::energy` folds over it left to right with
/// exactly the operations the hand-written chain performed, so deriving the sum from
/// this table changes no bit. Adding a row to the physics means adding it HERE, at the
/// end, and re-banking under one cause line — never inserting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    /// Kinetic energy. Not a channel: the substance's motion, not an interaction.
    Kin,
    /// The near pair sector: tabulated exact atom-pair curves inside their support.
    Pair,
    /// The three-body sector: tabulated exact three-body residuals inside their reach.
    Three,
    /// The many-body sector: exact connected cluster terms at the declared order.
    Many,
    /// The far pair sector: the tail past the switch radius, as a power law (B2).
    Far,
    /// The embedding field: fixed derived charges on census rows, counted once (FIELD-1).
    Field,
    /// The walls. The container, not the substance.
    Wall,
    /// The user's spring. The hand, not the substance.
    Spring,
    /// The uniform gravitational field. Conservative; an acceleration, not a channel.
    Grav,
    /// The seam's wall: channel 5 between units, whole, harvested (FIELD-3). Appended at
    /// the end under FIELD-3's cause line; exact `0.0` whenever the seam is off.
    Seam,
}

impl Row {
    /// THE ORDER OF `Sim::energy`. See the type's doc for why it may only grow at the end.
    pub const ALL: [Row; 10] = [
        Row::Kin,
        Row::Pair,
        Row::Three,
        Row::Many,
        Row::Far,
        Row::Field,
        Row::Wall,
        Row::Spring,
        Row::Grav,
        Row::Seam,
    ];

    /// Which channels this row's number is made of, and how. Container rows and the
    /// kinetic row carry none: they are not interactions between fragments.
    pub fn carries(self) -> &'static [(ChannelId, Carriage)] {
        match self {
            Row::Kin | Row::Wall | Row::Spring | Row::Grav => &[],
            // An exact atom-pair curve is everything two atoms do inside its support:
            // exchange at contact, pair dispersion beyond it, and the atom-level parts of
            // the two electrostatic channels — one number, four channels, none separable
            // from the table alone.
            Row::Pair => &[
                (ChannelId::Exchange, Carriage::Folded),
                (ChannelId::PairDispersion, Carriage::Folded),
                (ChannelId::Field, Carriage::Folded),
                (ChannelId::Induction, Carriage::Folded),
            ],
            // The three-body residual of the pair view: channel 4, plus the three-body
            // part of induction, folded.
            Row::Three => &[
                (ChannelId::ThreeBody, Carriage::Folded),
                (ChannelId::Induction, Carriage::Folded),
            ],
            // Whatever the sectors below missed, at the declared order — every channel's
            // higher-order remainder, folded.
            Row::Many => &[
                (ChannelId::ThreeBody, Carriage::Folded),
                (ChannelId::Induction, Carriage::Folded),
                (ChannelId::PairDispersion, Carriage::Folded),
                (ChannelId::Exchange, Carriage::Folded),
            ],
            // The far tail is one power law per curve: pair dispersion, whole, past R_s —
            // on a force law with no charge, which is the reason its exponent is checked
            // against channel 3's and not channel 1's.
            Row::Far => &[(ChannelId::PairDispersion, Carriage::Whole)],
            // Channel 1 at molecule level, whole, counted once.
            Row::Field => &[(ChannelId::Field, Carriage::Whole)],
            // Channel 5 between units, whole: the exponential the ledger declares, its
            // coefficients transferred from the exact dimer's residual (FIELD-3).
            Row::Seam => &[(ChannelId::Exchange, Carriage::Whole)],
        }
    }

    /// Does this row carry the channel at all.
    pub fn carries_channel(self, id: ChannelId) -> bool {
        self.carries().iter().any(|(c, _)| *c == id)
    }

    pub fn name(self) -> &'static str {
        match self {
            Row::Kin => "e_kin",
            Row::Pair => "e_pair",
            Row::Three => "e_three",
            Row::Many => "e_many",
            Row::Far => "e_far",
            Row::Field => "e_field",
            Row::Wall => "e_wall",
            Row::Spring => "e_spring",
            Row::Grav => "e_grav",
            Row::Seam => "e_seam",
        }
    }
}

/// Every row that carries a given channel, in ledger order.
pub fn rows_carrying(id: ChannelId) -> Vec<Row> {
    Row::ALL.iter().copied().filter(|r| r.carries_channel(id)).collect()
}

// ------------------------------------------------------------------ the one allocator

/// What "how far does this reach at this budget" is asked of.
pub enum Kernel<'a> {
    /// A pure power `−c · r^(−p)`: the reach is closed-form, floored at `r_min` (the radius
    /// the kernel is valid from — a far sector's `R_s`).
    Power { c: f64, p: f64, r_min: f64 },
    /// A sampled, monotone tail `u(r)` valid from `base` outward (a table's last knot and
    /// its exponential extrapolation past it): the reach is found by bisection.
    Sampled { u: &'a dyn Fn(f64) -> f64, base: f64 },
    /// A reach that was MEASURED and declared, per class and order (the surface registry's
    /// body reach). The allocator returns it as given; there is nothing to compute.
    Declared { reach: f64 },
}

/// Doublings the sampled walk may take past `base + 1` before the budget is refused.
pub const SAMPLED_WALK_GUARD: usize = 64;
/// Halvings the sampled bisection performs inside its bracket.
pub const SAMPLED_BISECT_STEPS: usize = 80;

/// THE ALLOCATOR. The radius past which one contribution on this kernel is under
/// `budget` hartree, or `None` where no radius meets it.
///
/// The arithmetic is, arm for arm, the arithmetic its two callers performed before this
/// function existed (`TailModel::radius_for_budget`, `Sim::derive_pair_cutoff`), so every
/// radius is bit-identical — which is what lets `tests/channel_ledger.rs` gate the
/// collapse of three dialects into one on identity rather than on a tolerance.
pub fn reach_for_budget(kernel: Kernel<'_>, budget: f64) -> Option<f64> {
    match kernel {
        Kernel::Power { c, p, r_min } => {
            if !(budget > 0.0) || c == 0.0 {
                return Some(r_min);
            }
            Some((c.abs() / budget).powf(1.0 / p).max(r_min))
        }
        Kernel::Sampled { u, base } => {
            if !(budget > 0.0) {
                return None;
            }
            if u(base).abs() <= budget {
                // Already under budget at the last knot: the tail is all that is left.
                return Some(base);
            }
            // The tail is a decaying exponential past the last knot, so `|u|` is monotone
            // there and bisection is exact to the bracket. Walk out in doublings until the
            // budget is met, then halve in.
            let mut hi = base + 1.0;
            let mut guard = 0;
            while u(hi).abs() > budget && guard < SAMPLED_WALK_GUARD {
                hi = base + (hi - base) * 2.0;
                guard += 1;
            }
            if u(hi).abs() > budget {
                return None;
            }
            let mut lo = base;
            for _ in 0..SAMPLED_BISECT_STEPS {
                let mid = 0.5 * (lo + hi);
                if u(mid).abs() > budget {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            Some(hi)
        }
        Kernel::Declared { reach } => Some(reach),
    }
}

// ------------------------------------------------ the exponent: assigned vs measured

/// Slack, as a ratio, between a tail's measured exponent and its channel's assigned one
/// before `ExponentReading::agrees` reads false. Wide, deliberately: the curves are FCI in
/// a minimal basis (`longrange.rs` header), which distorts the asymptote, so this is a
/// reading on the MODEL's tail against the channel's LAW, and a disagreement is a
/// finding about the curve, never silently a fit. B2's adopting band `[5, 7]` around `6`
/// is the same slack stated as an interval.
pub const EXPONENT_SLACK: f64 = 1.0 / 6.0;

/// One curve's measured tail exponent read against the channel it is booked to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExponentReading {
    pub slot: usize,
    pub channel: ChannelId,
    /// The channel's DERIVED leading power.
    pub assigned: f64,
    /// The curve's least-squares exponent over its last knots (`TailFit::p_fit`).
    pub measured: f64,
    /// `|measured − assigned| / assigned`.
    pub deviation: f64,
    /// `deviation ≤ EXPONENT_SLACK`.
    pub agrees: bool,
}

impl ExponentReading {
    /// Read a fit against a channel. `None` if the channel has no power (exchange).
    pub fn of(slot: usize, channel: ChannelId, fit: &TailFit) -> Option<Self> {
        let assigned = channel.record().rate.power()?;
        let deviation = (fit.p_fit - assigned).abs() / assigned;
        Some(Self {
            slot,
            channel,
            assigned,
            measured: fit.p_fit,
            deviation,
            agrees: deviation <= EXPONENT_SLACK,
        })
    }
}

// ------------------------------------------------------------- the report, per channel

/// How far a channel reaches in a scene right now, and by which dialect the reach was
/// set. Reported, never used to evaluate anything.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Reach {
    /// The whole scene: no cutoff, the sum runs over every tuple.
    Scene,
    /// A radius, bohr, and the name of the mechanism that set it.
    Radius { r: f64, by: &'static str },
    /// The channel has no row in this engine; nothing reaches.
    Absent,
}

/// One channel's standing in a scene: its record, the rows that carry it, the sum of
/// those rows AS THEY STAND (folded numbers included, so this is a bound on the channel's
/// contribution and never its value where the carriage is `Folded`), and its reach.
#[derive(Clone, Debug)]
pub struct ChannelStanding {
    pub channel: &'static Channel,
    pub rows: Vec<(Row, Carriage, f64)>,
    pub reach: Reach,
}

impl ChannelStanding {
    /// Is any row carrying this channel `Whole` — i.e. does the engine have the channel's
    /// own number anywhere.
    pub fn has_own_row(&self) -> bool {
        self.rows.iter().any(|(_, c, _)| *c == Carriage::Whole)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_five_are_in_rate_order_and_each_has_a_kind() {
        let kinds: Vec<Kind> = CHANNELS.iter().map(|c| c.kind).collect();
        assert_eq!(
            kinds,
            [Kind::Circumstances, Kind::Structure, Kind::Process, Kind::Rules, Kind::Identity]
        );
        let mut last = 0.0;
        for c in CHANNELS.iter() {
            if let Rate::Power(p) = c.rate {
                assert!(p > last, "rate order");
                last = p;
            }
        }
        assert_eq!(CHANNELS[4].rate, Rate::Exponential);
        for (i, c) in CHANNELS.iter().enumerate() {
            assert_eq!(c.id as usize, i);
            assert_eq!(c.id.record(), c);
        }
    }

    #[test]
    fn every_channel_but_induction_has_a_row_and_every_interaction_row_has_a_channel() {
        for c in CHANNELS.iter() {
            let rows = rows_carrying(c.id);
            assert!(!rows.is_empty(), "{:?} is carried by no row", c.id);
        }
        // The two whole rows: the far tail (dispersion) and the field (channel 1).
        assert!(Row::Far.carries().iter().any(|(id, c)| *id == ChannelId::PairDispersion && *c == Carriage::Whole));
        assert!(Row::Field.carries().iter().any(|(id, c)| *id == ChannelId::Field && *c == Carriage::Whole));
        // Induction has NO whole row: FIELD-2 is named, not built.
        assert!(!rows_carrying(ChannelId::Induction)
            .iter()
            .any(|r| r.carries().iter().any(|(id, c)| *id == ChannelId::Induction && *c == Carriage::Whole)));
        for r in Row::ALL.iter() {
            let interaction = !matches!(r, Row::Kin | Row::Wall | Row::Spring | Row::Grav);
            assert_eq!(interaction, !r.carries().is_empty(), "{:?}", r);
        }
    }

    #[test]
    fn the_power_arm_is_the_closed_form() {
        let (c, p, r_s) = (0.37, 6.0, 20.0);
        for &b in &[1.0e-9, 1.0e-6, 1.0e-3] {
            let got = reach_for_budget(Kernel::Power { c, p, r_min: r_s }, b).unwrap();
            let want = (c.abs() / b).powf(1.0 / p).max(r_s);
            assert_eq!(got.to_bits(), want.to_bits());
        }
        assert_eq!(reach_for_budget(Kernel::Power { c, p, r_min: r_s }, 0.0), Some(r_s));
        assert_eq!(reach_for_budget(Kernel::Power { c: 0.0, p, r_min: r_s }, 1e-9), Some(r_s));
    }

    #[test]
    fn the_sampled_arm_refuses_a_budget_no_radius_meets() {
        let flat = |_r: f64| 1.0;
        assert_eq!(reach_for_budget(Kernel::Sampled { u: &flat, base: 10.0 }, 0.5), None);
        let under = |_r: f64| 1.0e-12;
        assert_eq!(reach_for_budget(Kernel::Sampled { u: &under, base: 10.0 }, 0.5), Some(10.0));
        assert_eq!(reach_for_budget(Kernel::Sampled { u: &flat, base: 10.0 }, 0.0), None);
        assert_eq!(reach_for_budget(Kernel::Declared { reach: 7.5 }, 1.0), Some(7.5));
    }

    #[test]
    fn an_exponent_reading_is_a_deviation_from_the_assigned_power() {
        let fit = |p: f64| TailFit {
            p_fit: p,
            residual: 0.0,
            exp_index: p,
            band: crate::longrange::TailBand::Adopting,
            r_max: 20.0,
            u_at_max: 1e-9,
            knots_fitted: 4,
        };
        let r = ExponentReading::of(0, ChannelId::PairDispersion, &fit(6.0)).unwrap();
        assert!(r.agrees && r.deviation == 0.0);
        let r = ExponentReading::of(0, ChannelId::PairDispersion, &fit(7.5)).unwrap();
        assert!(!r.agrees);
        assert!(ExponentReading::of(0, ChannelId::Exchange, &fit(6.0)).is_none());
    }
}
