//! THE SEAM (FIELD-3, `conformance/water_observatory/FIELD3_PREREG.md`): the unit as a
//! CLOSURE reading, the closure surfaces served only within the closure, and channel 5's
//! wall between closures.
//!
//! Three statements, each the engine's own:
//!
//! 1. **The unit.** Each hydrogen belongs to the oxygen it is MOST BOUND to by the engine's
//!    O–H pair curve (`u(r)` lowest, inside the curve's reach); a water unit is an oxygen
//!    with exactly two such hydrogens. Every other atom is FREE. FIELD-2 showed why the
//!    pair VERDICT (`E_rel < 0` inside the turning point) cannot be the identity: it bonds
//!    the donor hydrogen to the acceptor oxygen across a hydrogen bond, and the FIELD-1
//!    rule then assigns nothing at exactly the configuration under test.
//! 2. **The seam rule.** With the seam on, a pair table serves a pair only when both atoms
//!    are in one unit or either is free; a three-body surface serves a triple only when all
//!    three are in one unit or any is free. The (O,H,H) surface is the water molecule's own
//!    residual and the O–H curve is the radical's; between two molecules FIELD-2 measured
//!    them at +42 mHa and −21 mHa of the wrong thing.
//! 3. **The wall.** Between units the ledger serves the contact: channel 1 (the field,
//!    FIELD-1) and channel 5 — `A·exp(−b·r)` on every cross-unit O–O pair, the shape the
//!    ledger declares for exchange (`channel::CHANNELS`, `Kind::Identity`), the coefficients
//!    TRANSFERRED from the exact dimer's residual over the field (`examples/field3_harvest.rs`)
//!    and never chosen. `SeamModel { a: 0.0, b: 0.0 }` is a legitimate state: the seam rule
//!    with no wall (gate G-B4).
//!
//! A change of unit membership at fixed positions is a transition: the closure sector's
//! energy under the old assignment against the new, posted to `w_ext` and to the `seam`
//! receipt column, exactly as the field's transitions are.

/// The unit id of an atom in no unit.
pub const FREE: u32 = u32::MAX;

/// The seam's cross-unit terms, each a DECLARED shape with TRANSFERRED coefficients
/// (FIELD-3, FIELD-4): the wall `A·exp(−b·r)` on cross-unit oxygen–oxygen pairs (channel 5);
/// the penetration-and-induction term `−P·exp(−c·r)` on cross-unit hydrogen–oxygen pairs
/// (channels 1 and 2 at the contact, FIELD-4); dispersion `−C₆/r⁶` on cross-unit
/// oxygen–oxygen pairs (channel 3, FIELD-4). Every coefficient at `0.0` switches its term
/// off exactly, so FIELD-3's engine is the identity when the FIELD-4 terms are zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeamModel {
    pub a: f64,
    pub b: f64,
    pub p: f64,
    pub c: f64,
    pub c6: f64,
    /// FIELD-7: the wall on cross-unit hydrogen–oxygen pairs, `a_oh·exp(−b_oh·r)`.
    pub a_oh: f64,
    pub b_oh: f64,
    /// FIELD-7: the wall on cross-unit hydrogen–hydrogen pairs, `a_hh·exp(−b_hh·r)`.
    pub a_hh: f64,
    pub b_hh: f64,
    /// FIELD-8: the contact term on cross-unit hydrogen–hydrogen pairs, `−p_hh·exp(−c_hh·r)`.
    pub p_hh: f64,
    pub c_hh: f64,
    /// CT-1: the charge-transfer term on cross-unit hydrogen–oxygen pairs, `−p_ct·exp(−c_ct·r)`
    /// (channel 6, whole) — the donor's O–H against the acceptor's lone pair is the contact.
    pub p_ct: f64,
    pub c_ct: f64,
}

impl SeamModel {
    /// The seam rule with no cross-unit term at all.
    pub const NO_WALL: SeamModel = SeamModel { a: 0.0, b: 0.0, p: 0.0, c: 0.0, c6: 0.0, a_oh: 0.0, b_oh: 0.0, a_hh: 0.0, b_hh: 0.0, p_hh: 0.0, c_hh: 0.0, p_ct: 0.0, c_ct: 0.0 };

    /// The charge-transfer term's energy at a cross-unit H–O separation `r` (CT-1).
    #[inline]
    pub fn charge_transfer(&self, r: f64) -> f64 {
        -self.p_ct * (-self.c_ct * r).exp()
    }

    /// The H–H contact term's energy at a cross-unit H–H separation `r` (FIELD-8).
    #[inline]
    pub fn contact_hh(&self, r: f64) -> f64 {
        -self.p_hh * (-self.c_hh * r).exp()
    }

    /// THE BOUNDEDNESS WALK (FIELD-9 G-B0, the discharge of M-EXTRAPOLATED-HOLE as FIELD-8 read
    /// it): each cross-unit class potential — O–O, H–O, H–H, in that order of `r_min` — walked
    /// from its own `r_min` (the shortest distance of that class the wall was FIT on) inward to
    /// 0.5 bohr on a 0.05 grid. A physical contact potential may dip below its fit range (a
    /// hydrogen bond's H···O well is real); what it may not do is fall without bound or end
    /// attractive at contact. Refused, by name: a value along the walk lower than the value at
    /// `r_min` by more than `kt`, or a value at 0.5 bohr that is not positive. `None` is a law
    /// the dynamics cannot fall through.
    pub fn bounded(&self, q_h: f64, r_min: [f64; 3], kt: f64) -> Option<String> {
        let q_o = -2.0 * q_h;
        let classes: [(&str, &dyn Fn(f64) -> f64); 3] = [
            ("O–O", &|r: f64| self.wall(r) + self.dispersion(r) + q_o * q_o / r),
            ("H–O", &|r: f64| self.penetration(r) + self.charge_transfer(r) + self.wall_oh(r) + q_h * q_o / r),
            ("H–H", &|r: f64| self.contact_hh(r) + self.wall_hh(r) + q_h * q_h / r),
        ];
        for (k, (name, u)) in classes.iter().enumerate() {
            let r0 = r_min[k];
            let floor = u(r0) - kt;
            let mut r = r0;
            while r > 0.5 + 1e-12 {
                r = (r - 0.05).max(0.5);
                let v = u(r);
                if v < floor {
                    return Some(format!("{name} potential falls to {v:+.4e} at r = {r:.2} bohr, more than kT below its value {:+.4e} at its fit floor r_min = {r0:.3}", u(r0)));
                }
            }
            let at_contact = u(0.5);
            if !(at_contact > 0.0) {
                return Some(format!("{name} potential is not positive at contact: {at_contact:+.4e} at 0.5 bohr"));
            }
        }
        None
    }

    /// THE NO-HOLE WALK (FIELD-8 G-N0, the discharge of M-EXTRAPOLATED-HOLE): each cross-unit
    /// class potential — H–O: contact + wall + charges; O–O: wall + dispersion + charges;
    /// H–H: contact + wall + charges, with the pin charge `q_h` on hydrogen and `−2 q_h` on
    /// oxygen — walked from 3.0 bohr inward to 0.5 on a 0.05 grid; the first FALL inward is
    /// named, with its class and radius. `None` is a law that rises to contact everywhere.
    pub fn hole(&self, q_h: f64) -> Option<String> {
        let q_o = -2.0 * q_h;
        let classes: [(&str, &dyn Fn(f64) -> f64); 3] = [
            ("H–O", &|r: f64| self.penetration(r) + self.charge_transfer(r) + self.wall_oh(r) + q_h * q_o / r),
            ("O–O", &|r: f64| self.wall(r) + self.dispersion(r) + q_o * q_o / r),
            ("H–H", &|r: f64| self.contact_hh(r) + self.wall_hh(r) + q_h * q_h / r),
        ];
        for (name, u) in classes.iter() {
            let mut k = 0usize;
            let mut prev = u(3.0);
            loop {
                k += 1;
                let r = 3.0 - 0.05 * k as f64;
                if r < 0.5 - 1e-12 {
                    break;
                }
                let v = u(r);
                if v < prev - 1e-12 {
                    return Some(format!("{name} potential falls inward at r = {r:.2} bohr ({v:+.4e} < {prev:+.4e})"));
                }
                prev = v;
            }
        }
        None
    }

    /// FIELD-3's wall alone.
    pub const fn wall_only(a: f64, b: f64) -> SeamModel {
        SeamModel { a, b, ..SeamModel::NO_WALL }
    }

    /// The H–O wall's energy at a cross-unit H–O separation `r` (FIELD-7).
    #[inline]
    pub fn wall_oh(&self, r: f64) -> f64 {
        self.a_oh * (-self.b_oh * r).exp()
    }

    /// The H–H wall's energy at a cross-unit H–H separation `r` (FIELD-7).
    #[inline]
    pub fn wall_hh(&self, r: f64) -> f64 {
        self.a_hh * (-self.b_hh * r).exp()
    }

    /// The wall's energy at separation `r`.
    #[inline]
    pub fn wall(&self, r: f64) -> f64 {
        self.a * (-self.b * r).exp()
    }

    /// The penetration term's energy at a cross-unit H–O separation `r` (negative for `P > 0`).
    #[inline]
    pub fn penetration(&self, r: f64) -> f64 {
        -self.p * (-self.c * r).exp()
    }

    /// The dispersion term's energy at a cross-unit O–O separation `r`.
    #[inline]
    pub fn dispersion(&self, r: f64) -> f64 {
        if self.c6 == 0.0 {
            0.0
        } else {
            let r2 = r * r;
            -self.c6 / (r2 * r2 * r2)
        }
    }
}

/// Why the seam cannot be switched on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeamRefusal {
    /// An acuity frame is installed. The seam's transition pass re-runs the pair loop, and
    /// the pair loop posts the frame's own transitions — the two ledgers would double-post.
    /// Not combined; refused by name rather than guarded by a comment.
    AcuityFrameSet,
    /// A far sector (B2) is declared. The seam rule fences the pair TABLES; the far tail of a
    /// cross-unit pair would still be served, and that is a scope the freeze does not cover.
    FarSectorDeclared,
    /// The many-body sector is on. A cluster spanning two units would still be served.
    ManyBodySectorOn,
}

impl core::fmt::Display for SeamRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SeamRefusal::AcuityFrameSet => write!(f, "the seam is refused while an acuity frame is installed (the pair loop would post the frame's transitions twice)"),
            SeamRefusal::FarSectorDeclared => write!(f, "the seam is refused while a far sector is declared (the cross-unit tail would still be served)"),
            SeamRefusal::ManyBodySectorOn => write!(f, "the seam is refused while the many-body sector is on (a cross-unit cluster would still be served)"),
        }
    }
}

/// The freeze's plants (§5), each acting on one sector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeamPlant {
    #[default]
    None,
    /// (i) `A → −A`.
    FlipSign,
    /// (ii) the seam rule for pairs only; three-body surfaces served across the seam.
    TriplesAcross,
    /// (iii) the reaction dropped on the wall.
    DropReaction,
    /// FIELD-4 plant (ii): the reaction dropped on the penetration and dispersion terms.
    DropReactionNew,
    /// FIELD-4 plant (i): `P → −P`.
    FlipPenetration,
    /// CT-1 plant (ii): `P_CT → −P_CT`.
    FlipChargeTransfer,
}

/// The seam's work counters for the last force pass, and its transitions to date.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SeamWork {
    /// Cross-unit pairs the tables did not serve, in the last force pass.
    pub pairs_dropped: u64,
    /// Cross-unit triples the surfaces did not serve, in the last force pass.
    pub triples_dropped: u64,
    /// The same two, summed over every force pass since the seam was switched on — the
    /// counts a vacuity check reads (a last pass can be dissociated; a life cannot hide).
    pub pairs_dropped_total: u64,
    pub triples_dropped_total: u64,
    /// Cross-unit oxygen–oxygen pairs the wall served.
    pub oo_pairs: u64,
    /// Cross-unit hydrogen–oxygen pairs the penetration term served (FIELD-4).
    pub ho_pairs: u64,
    /// Water units in the last assignment.
    pub units: u64,
    /// Membership transitions posted (including the enabling and disabling ones).
    pub transitions: u64,
}

/// The unit assignment from each hydrogen's best oxygen: `best_o[h]` is the index of the
/// oxygen hydrogen `h` is most bound to, or `FREE`; entries for non-hydrogens are ignored.
/// Returns the unit id per atom — the oxygen's own index for the members of a unit, `FREE`
/// otherwise.
pub fn units_from_best(z: &[u32], best_o: &[u32]) -> Vec<u32> {
    let n = z.len();
    let mut count = vec![0u32; n];
    for h in 0..n {
        if z[h] == 1 && best_o[h] != FREE {
            count[best_o[h] as usize] += 1;
        }
    }
    let mut unit = vec![FREE; n];
    for o in 0..n {
        if z[o] == 8 && count[o] == 2 {
            unit[o] = o as u32;
        }
    }
    for h in 0..n {
        if z[h] == 1 && best_o[h] != FREE && unit[best_o[h] as usize] != FREE {
            unit[h] = best_o[h];
        }
    }
    unit
}

/// The water units of an assignment: `(oxygen, [h1, h2])`, in oxygen order.
pub fn water_units(unit_of: &[u32], z: &[u32]) -> Vec<(usize, [usize; 2])> {
    let mut out = Vec::new();
    for o in 0..unit_of.len() {
        if z[o] != 8 || unit_of[o] != o as u32 {
            continue;
        }
        let hs: Vec<usize> = (0..unit_of.len()).filter(|&h| z[h] == 1 && unit_of[h] == o as u32).collect();
        if hs.len() == 2 {
            out.push((o, [hs[0], hs[1]]));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_contended_hydrogen_belongs_to_one_oxygen_and_units_need_exactly_two() {
        // O0 H1 H2 | O3 H4 H5, with H1 best-bound to O0 (its own) — two units
        let z = [8, 1, 1, 8, 1, 1];
        let best = [FREE, 0, 0, FREE, 3, 3];
        assert_eq!(units_from_best(&z, &best), vec![0, 0, 0, 3, 3, 3]);
        // H1 best-bound to O3 instead: O0 has one, O3 has three — no unit anywhere
        let best = [FREE, 3, 0, FREE, 3, 3];
        assert_eq!(units_from_best(&z, &best), vec![FREE; 6]);
        // a free hydrogen beside one unit
        let z = [8, 1, 1, 1];
        let best = [FREE, 0, 0, FREE];
        assert_eq!(units_from_best(&z, &best), vec![0, 0, 0, FREE]);
        assert_eq!(water_units(&[0, 0, 0, FREE], &z), vec![(0, [1, 2])]);
    }

    #[test]
    fn the_wall_is_the_declared_shape() {
        let m = SeamModel::wall_only(2.0, 0.5);
        assert_eq!(m.wall(0.0), 2.0);
        assert!((m.wall(2.0) - 2.0 * (-1.0f64).exp()).abs() < 1e-15);
        assert_eq!(SeamModel::NO_WALL.wall(3.0), 0.0);
        assert_eq!(SeamModel::NO_WALL.penetration(3.0), 0.0);
        assert_eq!(SeamModel::NO_WALL.dispersion(3.0), 0.0);
        let f = SeamModel { p: 1.0, c: 1.0, c6: 64.0, ..SeamModel::NO_WALL };
        assert!((f.penetration(1.0) + (-1.0f64).exp()).abs() < 1e-15);
        assert_eq!(f.dispersion(2.0), -1.0);
        assert!((SeamModel { p_hh: 2.0, c_hh: 1.0, ..SeamModel::NO_WALL }.contact_hh(1.0) + 2.0 * (-1.0f64).exp()).abs() < 1e-15);
        assert!((SeamModel { p_ct: 3.0, c_ct: 2.0, ..SeamModel::NO_WALL }.charge_transfer(0.5) + 3.0 * (-1.0f64).exp()).abs() < 1e-15);
        assert_eq!(SeamModel::NO_WALL.charge_transfer(2.0), 0.0);
    }

    #[test]
    fn the_no_hole_walk_names_a_fall_and_admits_a_law_that_rises() {
        let q = 0.231380372;
        // FIELD-7's harvest: the H–O contact decays slower than its wall — a hole
        let f7 = SeamModel { a: 1623.675, b: 2.20, p: 8.971, c: 1.83, a_oh: 8.669, b_oh: 2.30, a_hh: 2.652, b_hh: 1.90, ..SeamModel::NO_WALL };
        let h = f7.hole(q);
        assert!(h.as_deref().map_or(false, |m| m.starts_with("H–O")), "{h:?}");
        // the same law with an H–O wall that wins inward — no hole
        let ok = SeamModel { a_oh: 40.0, b_oh: 2.30, ..f7 };
        assert_eq!(ok.hole(q), None);
        // no seam terms at all: the charges alone attract on H–O — a hole, named
        assert!(SeamModel::NO_WALL.hole(q).is_some());
        // FIELD-9's boundedness: FIELD-7's law falls without bound (refused); FIELD-8's law dips
        // by a fraction of kT and rises to +1.3 Ha at contact (admitted); no terms at all ends
        // attractive at contact (refused)
        let kt = 9.28e-4;
        let r_min = [4.724, 2.78, 3.5];
        assert!(f7.bounded(q, r_min, kt).is_some());
        // FIELD-8's law: its H–O well dips 4.5 mHa below the value at r_min = 2.78 (about 5 kT, the
        // minimum at 2.28 bohr)
        // and rises to +1.3 Ha at contact — refused at a kT depth, admitted at 5 mHa; the monotone
        // walk names the dip either way. The depth is the rule; the test exercises both sides.
        let f8 = SeamModel { a: 387.87, b: 2.20, p: 11.737, c: 1.98, a_oh: 17.302, b_oh: 2.15, a_hh: 1.309, b_hh: 1.70, p_hh: 1.16e-4, c_hh: 0.5, ..SeamModel::NO_WALL };
        assert!(f8.bounded(q, r_min, kt).as_deref().map_or(false, |m| m.starts_with("H–O potential falls")));
        assert_eq!(f8.bounded(q, r_min, 5.0e-3), None, "{:?}", f8.bounded(q, r_min, 5.0e-3));
        assert!(f8.hole(q).is_some(), "the monotone walk still names FIELD-8's dip");
        assert!(SeamModel::NO_WALL.bounded(q, r_min, kt).is_some());
    }
}
