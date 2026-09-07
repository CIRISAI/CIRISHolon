//! The closure grammar: three productions, and the conditions supplied by the tier.
//!
//! # What it is
//!
//! `GANTT2.md`'s compiler-for-matter paragraph: terminals are atoms; a PRODUCTION rewrites
//! closures when their ledger satisfies a condition, and the merged form is real when the
//! dynamics never splits it (`Core/Closure.lean`) and its binding pays its rent
//! (`Core/Maintenance.lean`). Three productions are in play and this module is all three:
//!
//! * **unit** — members → a closure, by a condition on the CARRIER. The molecular tier's is
//!   `assign_units`: an oxygen and the two hydrogens its O–H curve holds lowest.
//! * **bond** — two closures → a pair, by a condition on the LEDGER. The molecular tier's
//!   is the seam ledger's condition, held by the rent clause rather than by a threshold.
//! * **rewrite** — a closure changes composition, by a BOUNDARY crossed. The molecular
//!   tier's is the proton transfer at H···O 1.95 bohr.
//!
//! # What it declares
//!
//! The conditions are the tier's, not this crate's: a [`Production`] holds a function the
//! tier supplies, and a [`Grammar`] is a list of them with the tier stamped on it. This
//! crate contributes the SHAPE and the refusals, and contributes no physics at all.
//!
//! # What it refuses
//!
//! **An ambiguous grammar reads as ambiguous, never as its first hit.** If two unit
//! productions admit the same members, [`Grammar::unit`] returns [`UnitReading::Ambiguous`]
//! naming every one of them rather than silently taking the first — the same rule
//! M-FIRST-VIOLATION-ONLY states for gates, applied where it belongs to a grammar: which
//! production fired must not depend on the order somebody pushed them.

use crate::closure::{Closure, Ledger, TierId};

/// A boundary the tier says a closure crossed, and where.
///
/// `at` is in the tier's own coordinate — bohr at the molecular tier, a chart at the fluid
/// tier — and the tier names the coordinate in `boundary`, because a crossing quoted
/// without its coordinate is a number nobody can check.
#[derive(Clone, Debug, PartialEq)]
pub struct Crossing {
    /// What was crossed, named: "H···O 1.95 bohr, the proton hands over" and the like.
    pub boundary: &'static str,
    /// Where, in the units the name states.
    pub at: f64,
}

/// Which of the three a production is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductionKind {
    Unit,
    Bond,
    Rewrite,
}

/// One production of the grammar. `C` is the tier's carrier — whatever object the tier's
/// condition needs to look at, and this crate never inspects it.
pub enum Production<C> {
    /// Members → a closure, by a condition on the carrier.
    Unit {
        name: &'static str,
        #[allow(clippy::type_complexity)]
        admits: Box<dyn Fn(&C, &[u32]) -> bool>,
    },
    /// Two closures → a pair, by a condition on the ledger between them.
    Bond {
        name: &'static str,
        #[allow(clippy::type_complexity)]
        admits: Box<dyn Fn(&Ledger) -> bool>,
    },
    /// A closure changes composition, by a boundary crossed.
    Rewrite {
        name: &'static str,
        #[allow(clippy::type_complexity)]
        crossed: Box<dyn Fn(&C, &Closure) -> Option<Crossing>>,
    },
}

impl<C> Production<C> {
    pub fn name(&self) -> &'static str {
        match self {
            Production::Unit { name, .. }
            | Production::Bond { name, .. }
            | Production::Rewrite { name, .. } => name,
        }
    }

    pub fn kind(&self) -> ProductionKind {
        match self {
            Production::Unit { .. } => ProductionKind::Unit,
            Production::Bond { .. } => ProductionKind::Bond,
            Production::Rewrite { .. } => ProductionKind::Rewrite,
        }
    }

    /// A unit production from a plain function.
    pub fn unit(name: &'static str, admits: impl Fn(&C, &[u32]) -> bool + 'static) -> Production<C> {
        Production::Unit { name, admits: Box::new(admits) }
    }

    /// A bond production from a plain function on the ledger.
    pub fn bond(name: &'static str, admits: impl Fn(&Ledger) -> bool + 'static) -> Production<C> {
        Production::Bond { name, admits: Box::new(admits) }
    }

    /// A rewrite production from a plain function on the carrier and the closure.
    pub fn rewrite(
        name: &'static str,
        crossed: impl Fn(&C, &Closure) -> Option<Crossing> + 'static,
    ) -> Production<C> {
        Production::Rewrite { name, crossed: Box::new(crossed) }
    }
}

impl<C> core::fmt::Debug for Production<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Production")
            .field("kind", &self.kind())
            .field("name", &self.name())
            .finish()
    }
}

/// What the grammar's unit productions say about a candidate member set.
#[derive(Clone, Debug, PartialEq)]
pub enum UnitReading {
    /// No unit production admits these members.
    NoProduction,
    /// Exactly one admits, and it is named.
    Admitted { by: &'static str, closure: Closure },
    /// More than one admits. The grammar is ambiguous HERE and the reading says which
    /// productions, rather than taking whichever was pushed first.
    Ambiguous(Vec<&'static str>),
    /// A production admitted but the members could not make a closure (empty, or repeated).
    Refused(crate::closure::ClosureRefusal),
}

/// What the grammar's bond productions say about one ledger.
#[derive(Clone, Debug, PartialEq)]
pub enum BondReading {
    NoProduction,
    Admitted { by: &'static str },
    Ambiguous(Vec<&'static str>),
}

/// What the grammar's rewrite productions say about one closure.
#[derive(Clone, Debug, PartialEq)]
pub enum RewriteReading {
    NoProduction,
    Crossed { by: &'static str, crossing: Crossing },
    Ambiguous(Vec<&'static str>),
}

/// A tier's grammar: its productions, and the rung they are written for.
pub struct Grammar<C> {
    pub tier: TierId,
    productions: Vec<Production<C>>,
}

impl<C> core::fmt::Debug for Grammar<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Grammar")
            .field("tier", &self.tier)
            .field("productions", &self.productions)
            .finish()
    }
}

impl<C> Grammar<C> {
    pub fn new(tier: TierId) -> Grammar<C> {
        Grammar { tier, productions: Vec::new() }
    }

    /// Add a production. Builder form so a tier's grammar reads as a list.
    pub fn with(mut self, p: Production<C>) -> Grammar<C> {
        self.productions.push(p);
        self
    }

    pub fn productions(&self) -> &[Production<C>] {
        &self.productions
    }

    /// How many productions of each kind, in the order unit / bond / rewrite. A tier whose
    /// row in `GANTT2.md` is complete has at least one of each; this counts, it does not
    /// judge.
    pub fn census(&self) -> [usize; 3] {
        let mut n = [0usize; 3];
        for p in &self.productions {
            n[match p.kind() {
                ProductionKind::Unit => 0,
                ProductionKind::Bond => 1,
                ProductionKind::Rewrite => 2,
            }] += 1;
        }
        n
    }

    /// Members → a closure, by every unit production that admits them.
    pub fn unit(&self, carrier: &C, members: &[u32]) -> UnitReading {
        let hits: Vec<&'static str> = self
            .productions
            .iter()
            .filter_map(|p| match p {
                Production::Unit { name, admits } if admits(carrier, members) => Some(*name),
                _ => None,
            })
            .collect();
        match hits.len() {
            0 => UnitReading::NoProduction,
            1 => match Closure::new(self.tier, members.to_vec()) {
                Ok(closure) => UnitReading::Admitted { by: hits[0], closure },
                Err(e) => UnitReading::Refused(e),
            },
            _ => UnitReading::Ambiguous(hits),
        }
    }

    /// Two closures → a pair, by every bond production whose condition the ledger meets.
    pub fn bond(&self, rows: &Ledger) -> BondReading {
        let hits: Vec<&'static str> = self
            .productions
            .iter()
            .filter_map(|p| match p {
                Production::Bond { name, admits } if admits(rows) => Some(*name),
                _ => None,
            })
            .collect();
        match hits.len() {
            0 => BondReading::NoProduction,
            1 => BondReading::Admitted { by: hits[0] },
            _ => BondReading::Ambiguous(hits),
        }
    }

    /// A closure changes composition, by every rewrite production that reads a crossing.
    pub fn rewrite(&self, carrier: &C, c: &Closure) -> RewriteReading {
        let hits: Vec<(&'static str, Crossing)> = self
            .productions
            .iter()
            .filter_map(|p| match p {
                Production::Rewrite { name, crossed } => crossed(carrier, c).map(|x| (*name, x)),
                _ => None,
            })
            .collect();
        match hits.len() {
            0 => RewriteReading::NoProduction,
            1 => {
                let (by, crossing) = hits.into_iter().next().expect("one hit");
                RewriteReading::Crossed { by, crossing }
            }
            _ => RewriteReading::Ambiguous(hits.into_iter().map(|(n, _)| n).collect()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::closure::{Channel, Ledger, TierId};

    /// A toy carrier: species by index, the way the molecular tier's is `z`.
    struct Z(Vec<u32>);

    fn water_grammar() -> Grammar<Z> {
        Grammar::new(TierId::MOLECULAR)
            .with(Production::unit("one oxygen and two hydrogens", |z: &Z, m: &[u32]| {
                let o = m.iter().filter(|&&i| z.0[i as usize] == 8).count();
                let h = m.iter().filter(|&&i| z.0[i as usize] == 1).count();
                o == 1 && h == 2 && m.len() == 3
            }))
            .with(Production::bond("sharing served and attractive", |l: &Ledger| {
                matches!(l.read(Channel::Sharing), Some(v) if v < 0.0)
            }))
            .with(Production::rewrite("the proton hands over", |_z: &Z, c: &Closure| {
                (c.len() != 3).then_some(Crossing { boundary: "H...O 1.95 bohr", at: 1.95 })
            }))
    }

    #[test]
    fn the_grammar_carries_one_of_each_production() {
        assert_eq!(water_grammar().census(), [1, 1, 1]);
    }

    #[test]
    fn the_unit_production_admits_a_water_and_refuses_a_pair() {
        let z = Z(vec![8, 1, 1, 8, 1, 1]);
        let g = water_grammar();
        match g.unit(&z, &[0, 1, 2]) {
            UnitReading::Admitted { by, closure } => {
                assert_eq!(by, "one oxygen and two hydrogens");
                assert_eq!(closure.members(), &[0, 1, 2]);
                assert_eq!(closure.tier, TierId::MOLECULAR);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(g.unit(&z, &[0, 1]), UnitReading::NoProduction);
    }

    #[test]
    fn the_bond_production_reads_the_ledger_and_not_a_threshold() {
        let g = water_grammar();
        let mut l = Ledger::unserved();
        assert_eq!(g.bond(&l), BondReading::NoProduction, "an unserved row admits nothing");
        assert!(l.serve(Channel::Sharing, 0.5));
        assert_eq!(g.bond(&l), BondReading::NoProduction);
        assert!(l.serve(Channel::Sharing, -0.5));
        assert_eq!(g.bond(&l), BondReading::Admitted { by: "sharing served and attractive" });
    }

    #[test]
    fn the_rewrite_production_names_the_boundary_it_crossed() {
        let z = Z(vec![8, 1, 1, 1]);
        let g = water_grammar();
        let three = Closure::new(TierId::MOLECULAR, vec![0, 1, 2]).unwrap();
        assert_eq!(g.rewrite(&z, &three), RewriteReading::NoProduction);
        let four = Closure::new(TierId::MOLECULAR, vec![0, 1, 2, 3]).unwrap();
        match g.rewrite(&z, &four) {
            RewriteReading::Crossed { by, crossing } => {
                assert_eq!(by, "the proton hands over");
                assert_eq!(crossing.at, 1.95);
            }
            other => panic!("{other:?}"),
        }
    }

    /// Which production fired must not depend on push order.
    #[test]
    fn two_productions_admitting_one_thing_read_ambiguous_and_name_both() {
        let z = Z(vec![8, 1, 1]);
        let g: Grammar<Z> = Grammar::new(TierId::MOLECULAR)
            .with(Production::unit("first", |_z, _m| true))
            .with(Production::unit("second", |_z, _m| true));
        assert_eq!(g.unit(&z, &[0, 1, 2]), UnitReading::Ambiguous(vec!["first", "second"]));
        let h: Grammar<Z> = Grammar::new(TierId::MOLECULAR)
            .with(Production::unit("second", |_z, _m| true))
            .with(Production::unit("first", |_z, _m| true));
        // the SET is the same; only the printing order follows the list
        let (a, b) = (g.unit(&z, &[0, 1, 2]), h.unit(&z, &[0, 1, 2]));
        let names = |r: UnitReading| match r {
            UnitReading::Ambiguous(mut n) => {
                n.sort_unstable();
                n
            }
            other => panic!("{other:?}"),
        };
        assert_eq!(names(a), names(b));
    }

    #[test]
    fn a_unit_production_admitting_an_empty_member_set_is_refused_not_constructed() {
        let z = Z(vec![]);
        let g: Grammar<Z> =
            Grammar::new(TierId::MOLECULAR).with(Production::unit("admits anything", |_z, _m| true));
        assert_eq!(
            g.unit(&z, &[]),
            UnitReading::Refused(crate::closure::ClosureRefusal::NoMembers)
        );
    }
}
