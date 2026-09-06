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
    /// CT-2: the bond's alignment on both sides of the transfer term. `m_ct` is the donor
    /// factor's exponent (`((1 − cos θ_d)/2)^m`, θ_d the O_d–H···O_a angle), `k_ct` the acceptor
    /// factor's (the two lone-pair alignments `((1 + u·l±)/2)^k`, normalised to 1 on the linear
    /// dimer), `lambda_ct` the lone-pair angle from the reversed bisector, in RADIANS. At
    /// `m_ct = k_ct = 0` the term is CT-1's pair exponential, bit for bit.
    pub m_ct: u8,
    pub k_ct: u8,
    pub lambda_ct: f64,
    /// LIQUID-1 Amendment 2: the cutoff radius of the C² switch on every cross-unit term
    /// (bohr); `0.0` means no switch — every record before the amendment is that state, bit
    /// for bit. The switch is 1 up to `r_cut − 2`, the quintic step down to 0 at `r_cut`.
    pub r_cut: f64,
}

impl SeamModel {
    /// The seam rule with no cross-unit term at all.
    pub const NO_WALL: SeamModel = SeamModel { a: 0.0, b: 0.0, p: 0.0, c: 0.0, c6: 0.0, a_oh: 0.0, b_oh: 0.0, a_hh: 0.0, b_hh: 0.0, p_hh: 0.0, c_hh: 0.0, p_ct: 0.0, c_ct: 0.0, m_ct: 0, k_ct: 0, lambda_ct: 0.0, r_cut: 0.0 };

    /// THE SWITCH (LIQUID-1 Amendment 2): `(S, dS/dr)` at `r` — 1 and 0 with no cutoff or
    /// below `r_on = r_cut − 2`; the quintic C² step `1 − 10x³ + 15x⁴ − 6x⁵`, `x = (r − r_on)/2`,
    /// between; 0 and 0 at and past `r_cut`.
    #[inline]
    pub fn switch(&self, r: f64) -> (f64, f64) {
        if self.r_cut <= 0.0 {
            return (1.0, 0.0);
        }
        let r_on = self.r_cut - 2.0;
        if r <= r_on {
            (1.0, 0.0)
        } else if r >= self.r_cut {
            (0.0, 0.0)
        } else {
            let x = (r - r_on) / 2.0;
            let (x2, x3) = (x * x, x * x * x);
            (1.0 - 10.0 * x3 + 15.0 * x3 * x - 6.0 * x3 * x2, (-30.0 * x2 + 60.0 * x3 - 30.0 * x3 * x) / 2.0)
        }
    }

    /// Is the transfer term angular (CT-2) or CT-1's pair exponential.
    #[inline]
    pub fn ct_is_angular(&self) -> bool {
        self.m_ct != 0 || self.k_ct != 0
    }

    /// THE ANGULAR TRANSFER TERM (CT-2 §0) on one cross-unit H–O pair, with its gradient on
    /// the five atoms it depends on: `[H, O_a, O_d, h₁, h₂]` — the hydrogen, the acceptor
    /// oxygen, the hydrogen's own oxygen, and the acceptor unit's two hydrogens (its frame).
    /// Positions are given RELATIVE to any common origin (the caller passes minimum-image
    /// deltas from `O_a`, so the term reduces to the minimum image like every other seam term).
    ///
    /// ```text
    /// E = −P·e^{−c r}·f_d(θ_d)·g_a(u)
    /// f_d = ((1 − cos θ_d)/2)^m,   cos θ_d = (O_d−H)·(O_a−H)/(|O_d−H||O_a−H|)
    /// g_a = [((1+u·l₊)/2)^k + ((1+u·l₋)/2)^k] / [2((1+cos λ)/2)^k],   u = (H−O_a)/r
    /// l± = −cos λ·b̂ ± sin λ·n̂,   b̂ = unit(h₁+h₂−2O_a),   n̂ = unit((h₁−O_a)×(h₂−O_a))
    /// ```
    ///
    /// The gradient is analytic, by the chain rule through `r`, `cos θ_d`, `u`, `b̂` and `n̂`;
    /// the seam suite's G-B3 (a central difference on every atom) is the check.
    pub fn ct_angular(&self, xh: [f64; 3], xa: [f64; 3], xd: [f64; 3], h1: [f64; 3], h2: [f64; 3]) -> (f64, [[f64; 3]; 5]) {
        fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
        fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
        fn sc(a: [f64; 3], k: f64) -> [f64; 3] { [a[0] * k, a[1] * k, a[2] * k] }
        fn dot(a: [f64; 3], b: [f64; 3]) -> f64 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
        fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
        fn norm(a: [f64; 3]) -> f64 { dot(a, a).sqrt() }
        let (m, k) = (self.m_ct as i32, self.k_ct as i32);
        let (cl, sl) = (self.lambda_ct.cos(), self.lambda_ct.sin());
        let mut gr = [[0.0f64; 3]; 5];
        // the pair part
        let d = sub(xh, xa);
        let r = norm(d).max(1e-9);
        let u = sc(d, 1.0 / r);
        let base = -self.p_ct * (-self.c_ct * r).exp();
        // the donor factor
        let a = sub(xd, xh);
        let b = sub(xa, xh);
        let (na, nb) = (norm(a).max(1e-9), norm(b).max(1e-9));
        let cth = (dot(a, b) / (na * nb)).clamp(-1.0, 1.0);
        let (f, df) = if m == 0 {
            (1.0, 0.0)
        } else {
            let t = 0.5 * (1.0 - cth);
            (t.powi(m), -0.5 * m as f64 * t.powi(m - 1))
        };
        // the acceptor frame and factor
        let v1 = sub(h1, xa);
        let v2 = sub(h2, xa);
        let w = add(v1, v2);
        let nw = norm(w).max(1e-9);
        let bh = sc(w, 1.0 / nw);
        let wn = cross(v1, v2);
        let nn = norm(wn).max(1e-9);
        let nh = sc(wn, 1.0 / nn);
        let (ub, un) = (dot(u, bh), dot(u, nh));
        let (sp, sm) = (-cl * ub + sl * un, -cl * ub - sl * un);
        let (g, dgp, dgm) = if k == 0 {
            (1.0, 0.0, 0.0)
        } else {
            let norm_g = 2.0 * (0.5 * (1.0 + cl)).powi(k);
            let (tp, tm) = (0.5 * (1.0 + sp), 0.5 * (1.0 + sm));
            ((tp.powi(k) + tm.powi(k)) / norm_g, 0.5 * k as f64 * tp.powi(k - 1) / norm_g, 0.5 * k as f64 * tm.powi(k - 1) / norm_g)
        };
        let e = base * f * g;
        // dE through r: d(base)/dr = −c·base
        let kr = -self.c_ct * base * f * g;
        gr[0] = add(gr[0], sc(u, kr));
        gr[1] = sub(gr[1], sc(u, kr));
        // dE through cos θ_d
        if m != 0 {
            let coef = base * g * df;
            let dca = sub(sc(b, 1.0 / (na * nb)), sc(a, cth / (na * na)));
            let dcb = sub(sc(a, 1.0 / (na * nb)), sc(b, cth / (nb * nb)));
            gr[2] = add(gr[2], sc(dca, coef));
            gr[1] = add(gr[1], sc(dcb, coef));
            gr[0] = sub(gr[0], sc(add(dca, dcb), coef));
        }
        // dE through u, b̂ and n̂
        if k != 0 {
            let (cp, cm) = (base * f * dgp, base * f * dgm);
            let aa = -cl * (cp + cm);
            let bb = sl * (cp - cm);
            let dvu = add(sc(bh, aa), sc(nh, bb));
            let proj = sc(sub(dvu, sc(u, dot(dvu, u))), 1.0 / r);
            gr[0] = add(gr[0], proj);
            gr[1] = sub(gr[1], proj);
            let dw = sc(sub(u, sc(bh, ub)), aa / nw);
            gr[3] = add(gr[3], dw);
            gr[4] = add(gr[4], dw);
            gr[1] = sub(gr[1], sc(dw, 2.0));
            let q = sc(sub(u, sc(nh, un)), bb / nn);
            let dv1 = cross(v2, q);
            let dv2 = cross(q, v1);
            gr[3] = add(gr[3], dv1);
            gr[4] = add(gr[4], dv2);
            gr[1] = sub(gr[1], add(dv1, dv2));
        }
        (e, gr)
    }

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
    ///
    /// EVERY violating class and BOTH legs are named, joined by `; ` (CT-1's correction): the
    /// first-violation return this walk shipped with under FIELD-9 reported the H–O class's
    /// one-kT dip at 2.58 bohr and said nothing of the same class ending at −0.41 hartree at
    /// contact or of the H–H class's −20.7 hartree — a detector that stopped at the mildest
    /// failure and hid the worst (M-FIRST-VIOLATION-ONLY).
    pub fn bounded(&self, q_h: f64, r_min: [f64; 3], kt: f64) -> Option<String> {
        let q_o = -2.0 * q_h;
        let classes: [(&str, &dyn Fn(f64) -> f64); 3] = [
            ("O–O", &|r: f64| self.wall(r) + self.dispersion(r) + q_o * q_o / r),
            ("H–O", &|r: f64| self.penetration(r) + self.charge_transfer(r) + self.wall_oh(r) + q_h * q_o / r),
            ("H–H", &|r: f64| self.contact_hh(r) + self.wall_hh(r) + q_h * q_h / r),
        ];
        let mut named: Vec<String> = Vec::new();
        for (k, (name, u)) in classes.iter().enumerate() {
            let r0 = r_min[k];
            let floor = u(r0) - kt;
            let mut r = r0;
            // the first fall below the floor, and the walk's minimum, both named
            let (mut first_fall, mut min_v, mut min_r) = (None, u(r0), r0);
            while r > 0.5 + 1e-12 {
                r = (r - 0.05).max(0.5);
                let v = u(r);
                if v < min_v {
                    min_v = v;
                    min_r = r;
                }
                if v < floor && first_fall.is_none() {
                    first_fall = Some((r, v));
                }
            }
            if let Some((rf, vf)) = first_fall {
                named.push(format!("{name} potential falls to {vf:+.4e} at r = {rf:.2} bohr, more than kT below its value {:+.4e} at its fit floor r_min = {r0:.3} (walk minimum {min_v:+.4e} at {min_r:.2} bohr, {:.1} kT below the floor)", u(r0), (u(r0) - min_v) / kt));
            }
            let at_contact = u(0.5);
            if !(at_contact > 0.0) {
                named.push(format!("{name} potential is not positive at contact: {at_contact:+.4e} at 0.5 bohr"));
            }
        }
        if named.is_empty() { None } else { Some(named.join("; ")) }
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

    /// HOW FAR THE SEAM LAW REACHES (LIQUID-1): the radius past which every cross-unit term
    /// is under `budget` in magnitude, from the law's own coefficients — an exponential
    /// `A·e^{−b r}` is under the budget past `ln(A/budget)/b`, the dispersion past
    /// `(C₆/budget)^{1/6}`. A term at an exact `0.0` reaches nowhere. Under a wrapping
    /// boundary this is the radius the minimum image must be unique out to for the seam's
    /// terms, and `Sim::legality_radius` takes it in place of the tables' reach when every
    /// atom is inside a unit.
    pub fn reach(&self, budget: f64) -> f64 {
        let exp_reach = |amp: f64, rate: f64| -> f64 {
            if amp.abs() > budget && rate > 0.0 {
                (amp.abs() / budget).ln() / rate
            } else {
                0.0
            }
        };
        let disp = if self.c6.abs() > budget { (self.c6.abs() / budget).powf(1.0 / 6.0) } else { 0.0 };
        let r = [
            exp_reach(self.a, self.b),
            exp_reach(self.p, self.c),
            exp_reach(self.a_oh, self.b_oh),
            exp_reach(self.a_hh, self.b_hh),
            exp_reach(self.p_hh, self.c_hh),
            exp_reach(self.p_ct, self.c_ct),
            disp,
        ]
        .into_iter()
        .fold(0.0, f64::max);
        // under a declared switch every term IS zero past r_cut (LIQUID-1 Amendment 2)
        if self.r_cut > 0.0 { r.min(self.r_cut) } else { r }
    }

    /// The unswitched reach, for the door's report beside the switched one.
    pub fn reach_unswitched(&self, budget: f64) -> f64 {
        SeamModel { r_cut: 0.0, ..*self }.reach(budget)
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
        // the reach: no term reaches anywhere at zero; one wall reaches ln(A/budget)/b; the
        // dispersion's sixth root; the law's reach is the largest of its terms'
        assert_eq!(SeamModel::NO_WALL.reach(1e-10), 0.0);
        // LIQUID-1 Amendment 2: the switch is 1 below r_on, 0 at r_cut, C² in between, and
        // its derivative matches a central difference; the reach is capped at r_cut
        let sw = SeamModel { a: 948.0, b: 2.4, p_hh: 0.017, c_hh: 1.02, r_cut: 14.0, ..SeamModel::NO_WALL };
        assert_eq!(sw.switch(11.99), (1.0, 0.0));
        assert_eq!(sw.switch(14.0), (0.0, 0.0));
        assert_eq!(sw.switch(15.0), (0.0, 0.0));
        let (mid, _) = sw.switch(13.0);
        assert!((mid - 0.5).abs() < 1e-15, "the quintic step's midpoint is one half: {mid}");
        for r in [12.0001, 12.3, 12.7, 13.0, 13.4, 13.9, 13.9999] {
            let (_, ds) = sw.switch(r);
            let h = 1e-6;
            let fd = (sw.switch(r + h).0 - sw.switch(r - h).0) / (2.0 * h);
            assert!((ds - fd).abs() < 1e-8, "dS/dr at {r}: {ds} vs {fd}");
        }
        // C² at both ends: S' and S'' vanish (S'' by the difference of S' across the joint)
        assert!(sw.switch(12.0 + 1e-9).1.abs() < 1e-15 && sw.switch(14.0 - 1e-9).1.abs() < 1e-15);
        assert!((sw.reach_unswitched(1e-10) - (0.017f64 / 1e-10).ln() / 1.02).abs() < 1e-9);
        assert_eq!(sw.reach(1e-10), 14.0, "the switched reach is r_cut");
        assert_eq!(SeamModel::NO_WALL.switch(3.0), (1.0, 0.0), "no cutoff: the identity");
        // CT-2: the angular term at m = k = 0 IS the pair term, on any frame
        let pair = SeamModel { p_ct: 1.47488, c_ct: 1.46, ..SeamModel::NO_WALL };
        let (xh, xa, xd, h1, h2) = ([0.3, -0.2, 1.9], [0.0, 0.0, 5.4], [0.1, 0.4, 0.0], [1.4, 0.2, 6.6], [-1.3, -0.3, 6.7]);
        let (e0, g0) = pair.ct_angular(xh, xa, xd, h1, h2);
        let r = ((xh[0] - xa[0]).powi(2) + (xh[1] - xa[1]).powi(2) + (xh[2] - xa[2]).powi(2)).sqrt();
        assert_eq!(e0, pair.charge_transfer(r));
        assert!(g0[2] == [0.0; 3] && g0[3] == [0.0; 3] && g0[4] == [0.0; 3], "no frame force at m = k = 0");
        // on the linear dimer of record f_d = g_a = 1 exactly: the angular term equals the pair term
        let ang = SeamModel { p_ct: 1.47488, c_ct: 1.46, m_ct: 2, k_ct: 2, lambda_ct: 55.0f64.to_radians(), ..SeamModel::NO_WALL };
        let (r_oh, th) = (1.9435738400f64, 1.6887434037f64);
        let (s2, c2) = ((0.5 * th).sin(), (0.5 * th).cos());
        let lin = ([0.0, 0.0, r_oh], [0.0, 0.0, 5.48], [0.0, 0.0, 0.0], [r_oh * s2, 0.0, 5.48 + r_oh * c2], [-r_oh * s2, 0.0, 5.48 + r_oh * c2]);
        let (e_lin, _) = ang.ct_angular(lin.0, lin.1, lin.2, lin.3, lin.4);
        assert!((e_lin - ang.charge_transfer(5.48 - r_oh)).abs() < 1e-14, "linear: {e_lin} vs {}", ang.charge_transfer(5.48 - r_oh));
        // the analytic gradient against a central difference on every coordinate of every atom
        for (m, k) in [(1u8, 0u8), (0, 1), (2, 2), (4, 4), (1, 4)] {
            let mm = SeamModel { m_ct: m, k_ct: k, ..ang };
            let pts = [xh, xa, xd, h1, h2];
            let (_, g) = mm.ct_angular(pts[0], pts[1], pts[2], pts[3], pts[4]);
            let h = 1e-6;
            for i in 0..5 {
                for c in 0..3 {
                    let mut pp = pts;
                    pp[i][c] += h;
                    let (ep, _) = mm.ct_angular(pp[0], pp[1], pp[2], pp[3], pp[4]);
                    pp[i][c] -= 2.0 * h;
                    let (em, _) = mm.ct_angular(pp[0], pp[1], pp[2], pp[3], pp[4]);
                    let fd = (ep - em) / (2.0 * h);
                    assert!((g[i][c] - fd).abs() <= 1e-9 * (1.0 + fd.abs()), "m {m} k {k} atom {i} coord {c}: analytic {} vs fd {fd}", g[i][c]);
                }
            }
            // translation invariance: the gradients sum to zero
            for c in 0..3 {
                let sum: f64 = (0..5).map(|i| g[i][c]).sum();
                assert!(sum.abs() < 1e-12, "m {m} k {k}: gradients sum to {sum}");
            }
        }
        let w = SeamModel::wall_only(948.0, 2.4);
        assert!((w.reach(1e-10) - (948.0f64 / 1e-10).ln() / 2.4).abs() < 1e-12);
        assert!(w.wall(w.reach(1e-10)) <= 1e-10 * (1.0 + 1e-9));
        let d = SeamModel { c6: 64.0, ..SeamModel::NO_WALL };
        assert!((d.reach(1e-10) - (64.0f64 / 1e-10).powf(1.0 / 6.0)).abs() < 1e-9);
        let both = SeamModel { c6: 64.0, ..w };
        assert_eq!(both.reach(1e-10), w.reach(1e-10).max(d.reach(1e-10)));
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
        // CT-1's law: the H–O class dips one kT at 2.58 bohr AND ends negative at contact, and
        // the H–H class is an abyss — every class and both legs are named, not only the first
        let ct1 = SeamModel { a: 948.048736, b: 2.40, p: 23.704848, c: 2.44, a_oh: 22.586054, b_oh: 2.20, a_hh: 1.525046, b_hh: 1.75, p_hh: 158.7891, c_hh: 4.00, p_ct: 1.47488, c_ct: 1.46, ..SeamModel::NO_WALL };
        let why = ct1.bounded(q, [4.724315, 2.780741, 1.314606], kt).expect("CT-1's law is refused");
        assert!(why.contains("H–O potential falls to"), "{why}");
        assert!(why.contains("H–O potential is not positive at contact"), "{why}");
        assert!(why.contains("H–H potential falls to"), "{why}");
        assert!(why.contains("H–H potential is not positive at contact"), "{why}");
        assert!(!why.contains("O–O potential"), "the O–O class is bounded: {why}");
        assert_eq!(why.matches("; ").count(), 3, "four violations, three separators: {why}");
        eprintln!("CT-1's law, every class named: {why}");
    }
}
