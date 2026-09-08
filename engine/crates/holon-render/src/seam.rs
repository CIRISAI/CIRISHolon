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
    /// CT-3: the transfer term is served from the TABLE (`Sim::ct_table`) rather than from
    /// `p_ct`, `c_ct` and the angular factors. `false` in every record written before CT-3, so
    /// [`SeamModel::ct_mode`] reads `Pair` or `Angular` on all of them, bit for bit.
    pub ct_table_on: bool,
}

impl SeamModel {
    /// The seam rule with no cross-unit term at all.
    pub const NO_WALL: SeamModel = SeamModel { a: 0.0, b: 0.0, p: 0.0, c: 0.0, c6: 0.0, a_oh: 0.0, b_oh: 0.0, a_hh: 0.0, b_hh: 0.0, p_hh: 0.0, c_hh: 0.0, p_ct: 0.0, c_ct: 0.0, m_ct: 0, k_ct: 0, lambda_ct: 0.0, r_cut: 0.0, ct_table_on: false };

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

    /// Is the transfer term the ANGULAR FAMILY's (CT-2) rather than CT-1's pair exponential.
    /// This is the family's own predicate and it says nothing about CT-3's table; the selector
    /// across all three shapes is [`SeamModel::ct_mode`].
    #[inline]
    pub fn ct_is_angular(&self) -> bool {
        self.m_ct != 0 || self.k_ct != 0
    }

    /// WHICH SHAPE CHANNEL 6 IS SERVED AS (CT-3). The table wins when it is switched on,
    /// because it replaces the family rather than multiplying it; otherwise the exponents
    /// decide, exactly as they did before this field existed.
    #[inline]
    pub fn ct_mode(&self) -> CtMode {
        if self.ct_table_on {
            CtMode::Table
        } else if self.ct_is_angular() {
            CtMode::Angular
        } else {
            CtMode::Pair
        }
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
        self.bounded_ct(q_h, r_min, kt, &|r| self.charge_transfer(r))
    }

    /// THE SAME WALK with the transfer row supplied by the caller (CT-3). The three shapes of
    /// channel 6 are not the same function of one separation: CT-1's and CT-2's are (CT-2's at
    /// its linear value, the deepest the family can be), and CT-3's table is not a pair term at
    /// all, so the walk takes the deepest reading the table can return at each `r`
    /// ([`CtTable::deepest`]) instead. Passing `charge_transfer` reproduces `bounded` exactly,
    /// and every caller of record does.
    pub fn bounded_ct(&self, q_h: f64, r_min: [f64; 3], kt: f64, ct: &dyn Fn(f64) -> f64) -> Option<String> {
        let q_o = -2.0 * q_h;
        let classes: [(&str, &dyn Fn(f64) -> f64); 3] = [
            ("O–O", &|r: f64| self.wall(r) + self.dispersion(r) + q_o * q_o / r),
            ("H–O", &|r: f64| self.penetration(r) + ct(r) + self.wall_oh(r) + q_h * q_o / r),
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
        self.hole_ct(q_h, &|r| self.charge_transfer(r))
    }

    /// The monotone walk with the transfer row supplied by the caller (CT-3), for the same
    /// reason [`SeamModel::bounded_ct`] takes one.
    pub fn hole_ct(&self, q_h: f64, ct: &dyn Fn(f64) -> f64) -> Option<String> {
        let q_o = -2.0 * q_h;
        let classes: [(&str, &dyn Fn(f64) -> f64); 3] = [
            ("H–O", &|r: f64| self.penetration(r) + ct(r) + self.wall_oh(r) + q_h * q_o / r),
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

// ---------------------------------------------------------------- CT-3: the term as a table

/// Which shape the seam serves for channel 6 (charge transfer).
///
/// `Pair` is CT-1's exponential on cross-unit H–O distances; `Angular` is CT-2's declared
/// family with the bond's alignment on both sides; `Table` is CT-3's — the 64-node map served
/// directly, with no family in between. The three are exclusive and the selector is
/// [`SeamModel::ct_mode`]; every record written before CT-3 reads `Pair` or `Angular`, bit for
/// bit, because `ct_table_on` is `false` in all of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CtMode {
    /// CT-1: `−P·e^{−c r}` on every cross-unit H–O pair.
    #[default]
    Pair,
    /// CT-2: the same, times the donor and acceptor angular factors.
    Angular,
    /// CT-3: the map itself, one reading per ordered pair of units at its contact.
    Table,
}

/// WHICH SERVING RULE THE TRANSFER TABLE IS READ THROUGH — the argmin the map was harvested
/// under, or the smooth partition of unity that replaces it.
///
/// CT-3 serves ONE reading per unordered pair of units, at the pair's SHORTEST cross-unit
/// H···O contact. That is an ARGMIN, and an argmin is discontinuous where it ties. CT-3
/// measured the jump on ONE dimer at `1.418e-5` hartree and fenced it; LIQUID-2's labelled
/// screen then measured what a 128-water box does with it
/// (`conformance/water_observatory/liquid2/DRIFT_NOTE.md`): 3,803 handovers in 2,000 frames,
/// the running extremum of their SIGNED jump sum `6.361e-3` hartree against a measured drift
/// peak of `6.585e-3` — **96.6 % of the drift**, three orders above the same box with channel
/// 6 switched off. The values of the table are not what failed; the argmin is.
///
/// `Blend` is the declared replacement, and it is a rule and not a smoothing knob:
///
/// ```text
/// E_pair = Σ_k w_k · S(r_k) · E_table(coords_k)          over the FOUR cross-unit H···O contacts
/// w_k    = e^{−β r_k} / Σ_j e^{−β r_j}                    a partition of unity on the contact distances
/// ```
///
/// with the FULL force — both terms — on every atom of every contact,
///
/// ```text
/// −∇E = −Σ_k w_k ∇Ẽ_k − Σ_k Ẽ_k ∇w_k,   ∇w_k = w_k(−β ∇r_k + β Σ_j w_j ∇r_j)
/// ```
///
/// (the second term is the one the first specification of this rule omitted, and the second
/// external review caught). `β` is DERIVED from the map's own records and never typed — see
/// [`CtTable::set_blend`] and `examples/ct3_smooth.rs` — and the switch applies PER CONTACT,
/// so a contact past `r_cut` contributes an exact zero to the energy and to both force terms
/// while still carrying its own weight.
///
/// `Argmin` is the default and every record written before this rule existed reads it, bit
/// for bit: [`CtTable::empty`] loads `Argmin` and only [`CtTable::set_blend`] changes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CtServe {
    /// CT-3's own: the pair's shortest cross-unit H···O contact, and nothing else about the
    /// pair enters. Discontinuous where two contacts tie.
    #[default]
    Argmin,
    /// The partition of unity over all four cross-unit H···O contacts at inverse length `β`.
    Blend,
}

/// The number of cross-unit H···O contacts an unordered pair of water units presents: each
/// unit's two hydrogens against the other unit's oxygen. The argmin ranks these four and the
/// blend weights them.
pub const CT_CONTACTS: usize = 4;

/// The number of knots the transfer table can hold. CT-3's map has 60 distinct sites; the
/// bound is the next power of two above it, and `finish` refuses more.
pub const MAX_CT_KNOTS: usize = 128;

/// The table's coordinate count: `(r, cos θ_d, u·b̂, q)`. See [`ct_coords`].
pub const CT_DIM: usize = 4;

/// Why a transfer table did not load.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtLoad {
    Empty,
    Ok,
    TooManyKnots,
    /// Fewer knots than the interpolant's own polynomial tail has coefficients.
    TooFewKnots,
    NotFinite,
    /// An axis whose knots all carry one value: the scaling is undefined and the table would
    /// be a function of three coordinates wearing four.
    DegenerateAxis,
    /// Two knots at the same coordinates. CT-3's map has four such pairs — both poles of the
    /// acceptor's azimuth — and the LOADER must merge them by the freeze's own rule before
    /// pushing, because a table cannot hold two values at one site and pretending otherwise
    /// is how a coordinate degeneracy gets laundered into an interpolation.
    DuplicateKnot,
    /// The interpolation matrix is singular to working precision.
    Singular,
}

/// THE COORDINATES OF ONE TRANSFER CONTACT, and their gradients on the five atoms that carry
/// them — `[H, O_a, O_d, h₁, h₂]`, the same five, in the same order, as [`SeamModel::ct_angular`].
///
/// ```text
/// r  = |H − O_a|                                        the contact separation, bohr
/// c_d = cos θ_d = (O_d−H)·(O_a−H)/(|O_d−H||O_a−H|)      the donor's alignment
/// p  = u·b̂                                              the acceptor's polar alignment
/// q  = 2(u·n̂)² + p² − 1                                 the acceptor's azimuth
/// ```
///
/// with `u = (H − O_a)/r`, `b̂ = unit(h₁ + h₂ − 2O_a)` the acceptor's bisector and
/// `n̂ = unit((h₁−O_a) × (h₂−O_a))` its plane normal — every one of them a definition
/// [`SeamModel::ct_angular`] already uses, and none of them new.
///
/// **Why `cos θ_d` and `p` rather than the angles, and why `q` rather than an azimuth.** The
/// map puts fifty of its sixty-four nodes at `θ_d = 180°` exactly and every tilt node at an
/// exact multiple of 30°; `dθ/d cos θ` is singular at `cos θ = ±1`, so a table in the ANGLES
/// has an infinite force at the geometries the map is densest at. An azimuth `atan2(|u·n̂|,
/// |u·t̂|)` is worse: it is folded by the acceptor's own symmetry, so it carries absolute
/// values whose kinks sit exactly on the two sheets the map samples (the tilt family at
/// `u·t̂ = 0`, the twist family at `u·n̂ = 0`). `q` is the same information with neither
/// defect: it is smooth everywhere, invariant under both of the acceptor's mirrors and under
/// relabelling its hydrogens, and it vanishes at both poles by construction — which is where
/// the azimuth is genuinely undefined and the map's four duplicate sites live.
/// `q = +1` is the donor on the acceptor's plane NORMAL (CT-2's finding), `q = −1` the donor
/// in the acceptor's own plane, `q = 0` on the bisector at either end.
pub fn ct_coords(
    xh: [f64; 3],
    xa: [f64; 3],
    xd: [f64; 3],
    h1: [f64; 3],
    h2: [f64; 3],
) -> ([f64; CT_DIM], [[[f64; 3]; 5]; CT_DIM]) {
    fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
    fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
    fn sc(a: [f64; 3], k: f64) -> [f64; 3] { [a[0] * k, a[1] * k, a[2] * k] }
    fn dot(a: [f64; 3], b: [f64; 3]) -> f64 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
    fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
    fn norm(a: [f64; 3]) -> f64 { dot(a, a).sqrt() }

    let mut g = [[[0.0f64; 3]; 5]; CT_DIM];

    // (0) the separation
    let d = sub(xh, xa);
    let r = norm(d).max(1e-9);
    let u = sc(d, 1.0 / r);
    g[0][0] = u;
    g[0][1] = sc(u, -1.0);

    // (1) the donor's alignment
    let a = sub(xd, xh);
    let b = sub(xa, xh);
    let (na, nb) = (norm(a).max(1e-9), norm(b).max(1e-9));
    let cd = (dot(a, b) / (na * nb)).clamp(-1.0, 1.0);
    let dca = sub(sc(b, 1.0 / (na * nb)), sc(a, cd / (na * na)));
    let dcb = sub(sc(a, 1.0 / (na * nb)), sc(b, cd / (nb * nb)));
    g[1][2] = dca;
    g[1][1] = dcb;
    g[1][0] = sc(add(dca, dcb), -1.0);

    // the acceptor's frame
    let v1 = sub(h1, xa);
    let v2 = sub(h2, xa);
    let w = add(v1, v2);
    let nw = norm(w).max(1e-9);
    let bh = sc(w, 1.0 / nw);
    let wn = cross(v1, v2);
    let nn = norm(wn).max(1e-9);
    let nh = sc(wn, 1.0 / nn);
    let p = dot(u, bh);
    let un = dot(u, nh);

    // (2) the acceptor's polar alignment `p = u·b̂`
    // through u: ∂(u·ê)/∂x_H = (ê − (u·ê)u)/r, and the opposite on O_a
    let du_b = sc(sub(bh, sc(u, p)), 1.0 / r);
    g[2][0] = du_b;
    g[2][1] = sc(du_b, -1.0);
    // through b̂: ∂(u·b̂)/∂w = (u − p b̂)/|w|, with w = h₁ + h₂ − 2 O_a
    let dw_b = sc(sub(u, sc(bh, p)), 1.0 / nw);
    g[2][3] = dw_b;
    g[2][4] = dw_b;
    g[2][1] = sub(g[2][1], sc(dw_b, 2.0));

    // `un = u·n̂`, the same two routes, with the normal's own chain rule through v₁ × v₂
    let du_n = sc(sub(nh, sc(u, un)), 1.0 / r);
    let mut gn = [[0.0f64; 3]; 5];
    gn[0] = du_n;
    gn[1] = sc(du_n, -1.0);
    let qn = sc(sub(u, sc(nh, un)), 1.0 / nn);
    let dv1 = cross(v2, qn);
    let dv2 = cross(qn, v1);
    gn[3] = add(gn[3], dv1);
    gn[4] = add(gn[4], dv2);
    gn[1] = sub(gn[1], add(dv1, dv2));

    // (3) the azimuth `q = 2 un² + p² − 1`
    let qc = 2.0 * un * un + p * p - 1.0;
    for i in 0..5 {
        for c in 0..3 {
            g[3][i][c] = 4.0 * un * gn[i][c] + 2.0 * p * g[2][i][c];
        }
    }

    ([r, cd, p, qc], g)
}

/// THE FOUR CROSS-UNIT H···O CONTACTS OF ONE UNORDERED PAIR OF WATER UNITS, in
/// [`ct_coords`]' own five-atom order `[H, O_a, O_d, h₁, h₂]` — unit A's two hydrogens
/// donating to unit B's oxygen first, then unit B's two to unit A's.
///
/// This is the list the argmin ranks and the list the blend weights, written ONCE so the
/// engine's `accumulate_seam`, the campaign runner and the gates cannot disagree about which
/// four they are or which way round each one is. Positions are given relative to any common
/// origin; the caller passes minimum-image deltas.
pub fn ct_pair_contacts(
    o_a: [f64; 3],
    h_a: [[f64; 3]; 2],
    o_b: [f64; 3],
    h_b: [[f64; 3]; 2],
) -> [[[f64; 3]; 5]; CT_CONTACTS] {
    [
        [h_a[0], o_b, o_a, h_b[0], h_b[1]],
        [h_a[1], o_b, o_a, h_b[0], h_b[1]],
        [h_b[0], o_a, o_b, h_a[0], h_a[1]],
        [h_b[1], o_a, o_b, h_a[0], h_a[1]],
    ]
}

/// THE TRANSFER TERM AS A TABLE (CT-3): the 64-node exact-minus-closed-sector map of CT-2,
/// served as the charge-transfer term itself.
///
/// # The contract
///
/// Knots arrive through `begin` / `knot` / `finish`, the way [`crate::table::PotentialTable`]'s
/// do, so the loader — the harvest runner natively, the JSON door in the browser — is the only
/// thing that reads a file and the interpolant is built in exactly one place. Each knot is one
/// node of the map: its four coordinates by [`ct_coords`], and its value the node's own `E_CT`
/// (the exact total minus the closed-sector total, the records' own rule). Nothing is fitted
/// and no parameter is chosen: **the interpolant IS the term**, as the Hermite spline IS the
/// pair potential.
///
/// # The rule, declared
///
/// The served energy is
///
/// ```text
/// E(y) = −S(ỹ) · exp(−c₀ · r)
/// S(ỹ) = Σ_i w_i ‖ỹ − x̃_i‖³ + w_n + Σ_j w_{n+1+j} ỹ_j       (the cubic polyharmonic spline)
/// ỹ_j  = (y_j − lo_j)/(hi_j − lo_j)                          (each axis on its own knot range)
/// ```
///
/// — the **cubic polyharmonic spline with a linear tail**, on the four coordinates scaled to
/// the box the knots themselves span. Three reasons, each one a thing that was measured rather
/// than assumed:
///
/// * **`ρ³` has no shape parameter.** Every Gaussian, multiquadric or inverse-multiquadric
///   kernel carries a width that would have to be chosen, and this programme does not type a
///   number a record does not carry. `ρ³` carries none, and it was also the best of the six
///   kernels tried on the map's own leave-one-out.
/// * **It is C² everywhere, knots included.** `‖x‖³` has a continuous Hessian at the origin,
///   so the force is continuous and differentiable AT the data — where a piecewise-linear
///   simplex interpolant would put a facet and a plain Shepard weighting would put a flat spot.
/// * **The exponential prefactor carries the decay.** `c₀` is a knot of the loader's, not the
///   table's: CT-2's own fitted `c_ct` from `wall_ct2.json`. Dividing it out leaves a shape
///   function of order one, which is what makes a scattered interpolant honest here; tabling
///   `E_CT` directly was measured at three times the leave-one-out error, and it also makes
///   the far field a polynomial rather than a decay.
///
/// # What it does not do
///
/// It does not extrapolate gracefully and does not pretend to: past the knot box the cubic
/// grows and only the prefactor and the seam's own switch hold it down. The reach, the
/// boundedness walk and the switch are the fences, and each is measured rather than assumed.
#[derive(Clone)]
pub struct CtTable {
    x: [[f64; CT_DIM]; MAX_CT_KNOTS],
    v: [f64; MAX_CT_KNOTS],
    s: [f64; MAX_CT_KNOTS],
    w: [f64; MAX_CT_KNOTS + CT_DIM + 1],
    lo: [f64; CT_DIM],
    rng: [f64; CT_DIM],
    n: usize,
    filling: usize,
    /// The exponent divided out before interpolation, per bohr — a record value.
    pub c0: f64,
    /// THE DECLARED INWARD FENCE (CT-3 §2 G-B0), in bohr; `0.0` is no fence, which is what
    /// every table loads as. When set, the SHAPE is read at `max(r, r_clamp)` while the
    /// prefactor `exp(−c₀ r)` keeps running at the true `r` — the spline's own behaviour below
    /// its innermost knot is replaced by that knot's shape, held. It is a stated fence and not
    /// a fit: nothing is chosen, the value held is the interpolant's own at the knot floor, and
    /// it caps the shape exactly the way `reach` caps at `r_cut`. Its price is a force
    /// discontinuity at `r_clamp` — the energy is continuous, `dS/dr` is not — and the freeze
    /// measures that jump rather than asserting it small.
    pub r_clamp: f64,
    pub status: CtLoad,
    /// The worst `|E(x_i) − v_i|` over the knots, measured by `finish`. The interpolant is an
    /// interpolant, so this is a statement about the LINEAR SOLVE's arithmetic and nothing else.
    pub worst_knot_miss: f64,
    /// The largest knot shape value. `−deepest_shape·exp(−c₀ r)` is the deepest reading the
    /// table can return at separation `r` on its own data, and it is what the boundedness walk
    /// takes for the transfer row rather than the linear value (which is NOT the deepest here,
    /// as it was for CT-2's family).
    pub deepest_shape: f64,
    /// WHICH SERVING RULE (see [`CtServe`]). `Argmin` on every table that has ever loaded;
    /// only [`CtTable::set_blend`] changes it, and it is deliberately not settable by a bare
    /// field write, so that the inverse length can never be installed without the rule or the
    /// rule without a derived inverse length.
    serve_mode: CtServe,
    /// The blend's inverse length, per bohr — DERIVED from the map's own records, never typed.
    /// `0.0` under `Argmin`.
    beta: f64,
}

impl CtTable {
    pub const fn empty() -> Self {
        Self {
            x: [[0.0; CT_DIM]; MAX_CT_KNOTS],
            v: [0.0; MAX_CT_KNOTS],
            s: [0.0; MAX_CT_KNOTS],
            w: [0.0; MAX_CT_KNOTS + CT_DIM + 1],
            lo: [0.0; CT_DIM],
            rng: [0.0; CT_DIM],
            n: 0,
            filling: 0,
            c0: 0.0,
            r_clamp: 0.0,
            status: CtLoad::Empty,
            worst_knot_miss: 0.0,
            deepest_shape: 0.0,
            serve_mode: CtServe::Argmin,
            beta: 0.0,
        }
    }

    /// The serving rule this table is read through. `Argmin` unless [`CtTable::set_blend`] has
    /// been called, which is what keeps every record written before the smooth rule existed
    /// bit-identical.
    #[inline]
    pub fn serve_mode(&self) -> CtServe {
        self.serve_mode
    }

    /// The blend's inverse length per bohr, `0.0` under `Argmin`.
    #[inline]
    pub fn beta(&self) -> f64 {
        self.beta
    }

    /// INSTALL THE SMOOTH SERVING RULE at a derived inverse length. Refuses a `β` that is not
    /// finite and strictly positive, because a zero or negative one is not a partition of
    /// unity concentrated on the contact — it is the flat mean, or the LONGEST contact.
    ///
    /// `β` is not a parameter of this engine and is not chosen here: it comes from the map's
    /// own shortest-to-second-shortest contact separations against the table's own resolution
    /// floor (`examples/ct3_smooth.rs`, which prints the arithmetic and every input's path).
    pub fn set_blend(&mut self, beta: f64) -> bool {
        if !beta.is_finite() || beta <= 0.0 {
            return false;
        }
        self.serve_mode = CtServe::Blend;
        self.beta = beta;
        true
    }

    /// Back to CT-3's own argmin, and the inverse length with it.
    pub fn set_argmin(&mut self) {
        self.serve_mode = CtServe::Argmin;
        self.beta = 0.0;
    }

    pub fn is_loaded(&self) -> bool {
        self.status == CtLoad::Ok && self.n >= CT_DIM + 2
    }

    pub fn knots(&self) -> usize {
        self.n
    }

    pub fn knot_x(&self, i: usize) -> [f64; CT_DIM] {
        if i < self.n { self.x[i] } else { [0.0; CT_DIM] }
    }

    pub fn knot_v(&self, i: usize) -> f64 {
        if i < self.n { self.v[i] } else { 0.0 }
    }

    /// The knot box, per axis, as `finish` measured it.
    pub fn axis_range(&self, j: usize) -> (f64, f64) {
        if j < CT_DIM { (self.lo[j], self.lo[j] + self.rng[j]) } else { (0.0, 0.0) }
    }

    pub fn begin(&mut self, count: usize, c0: f64) -> bool {
        if count > MAX_CT_KNOTS {
            self.status = CtLoad::TooManyKnots;
            return false;
        }
        if count < CT_DIM + 2 {
            self.status = CtLoad::TooFewKnots;
            return false;
        }
        if !c0.is_finite() || c0 <= 0.0 {
            self.status = CtLoad::NotFinite;
            return false;
        }
        self.n = 0;
        self.filling = count;
        self.c0 = c0;
        self.worst_knot_miss = 0.0;
        self.deepest_shape = 0.0;
        self.status = CtLoad::Empty;
        true
    }

    /// One knot: its four coordinates and its `E_CT` in hartree (negative for an attraction).
    pub fn knot(&mut self, index: usize, y: [f64; CT_DIM], value: f64) -> bool {
        if index >= self.filling {
            return false;
        }
        if !value.is_finite() || y.iter().any(|c| !c.is_finite()) {
            self.status = CtLoad::NotFinite;
            return false;
        }
        self.x[index] = y;
        self.v[index] = value;
        if index + 1 > self.n {
            self.n = index + 1;
        }
        true
    }

    pub fn finish(&mut self) -> CtLoad {
        let n = self.n;
        if n != self.filling || n < CT_DIM + 2 {
            self.status = CtLoad::TooFewKnots;
            return self.status;
        }
        for j in 0..CT_DIM {
            let mut lo = f64::INFINITY;
            let mut hi = f64::NEG_INFINITY;
            for i in 0..n {
                lo = lo.min(self.x[i][j]);
                hi = hi.max(self.x[i][j]);
            }
            if !(hi - lo > 0.0) {
                self.status = CtLoad::DegenerateAxis;
                return self.status;
            }
            self.lo[j] = lo;
            self.rng[j] = hi - lo;
        }
        for i in 0..n {
            self.s[i] = -self.v[i] * (self.c0 * self.x[i][0]).exp();
        }
        self.deepest_shape = (0..n).fold(f64::NEG_INFINITY, |m, i| m.max(self.s[i]));

        // the saddle system: [A P; Pᵀ 0][w; λ] = [s; 0], A_ij = ‖x̃_i − x̃_j‖³, P = [1 x̃]
        let m = n + CT_DIM + 1;
        let mut a = vec![0.0f64; m * m];
        let mut rhs = vec![0.0f64; m];
        for i in 0..n {
            let xi = self.scaled(self.x[i]);
            for j in 0..n {
                let xj = self.scaled(self.x[j]);
                let d2: f64 = (0..CT_DIM).map(|k| (xi[k] - xj[k]) * (xi[k] - xj[k])).sum();
                if i != j && d2 <= 1e-24 {
                    self.status = CtLoad::DuplicateKnot;
                    return self.status;
                }
                a[i * m + j] = d2.sqrt() * d2;
            }
            a[i * m + n] = 1.0;
            a[n * m + i] = 1.0;
            for k in 0..CT_DIM {
                a[i * m + n + 1 + k] = xi[k];
                a[(n + 1 + k) * m + i] = xi[k];
            }
            rhs[i] = self.s[i];
        }
        // Gaussian elimination with partial pivoting; the block is indefinite, so no Cholesky.
        for c in 0..m {
            let mut piv = c;
            for r in (c + 1)..m {
                if a[r * m + c].abs() > a[piv * m + c].abs() {
                    piv = r;
                }
            }
            if !(a[piv * m + c].abs() > 1e-300) {
                self.status = CtLoad::Singular;
                return self.status;
            }
            if piv != c {
                for k in 0..m {
                    a.swap(c * m + k, piv * m + k);
                }
                rhs.swap(c, piv);
            }
            let d = a[c * m + c];
            for r in (c + 1)..m {
                let f = a[r * m + c] / d;
                if f == 0.0 {
                    continue;
                }
                for k in c..m {
                    a[r * m + k] -= f * a[c * m + k];
                }
                rhs[r] -= f * rhs[c];
            }
        }
        for c in (0..m).rev() {
            let mut acc = rhs[c];
            for k in (c + 1)..m {
                acc -= a[c * m + k] * self.w[k];
            }
            self.w[c] = acc / a[c * m + c];
        }
        if self.w[..m].iter().any(|x| !x.is_finite()) {
            self.status = CtLoad::Singular;
            return self.status;
        }
        self.status = CtLoad::Ok;
        let mut worst = 0.0f64;
        for i in 0..n {
            worst = worst.max((self.eval(self.x[i]) - self.v[i]).abs());
        }
        self.worst_knot_miss = worst;
        self.status
    }

    #[inline]
    fn scaled(&self, y: [f64; CT_DIM]) -> [f64; CT_DIM] {
        let mut o = [0.0; CT_DIM];
        for j in 0..CT_DIM {
            o[j] = (y[j] - self.lo[j]) / self.rng[j];
        }
        // the declared inward fence, on the SHAPE's argument only — the prefactor is applied
        // outside this and keeps running at the true separation
        if self.r_clamp > 0.0 && y[0] < self.r_clamp {
            o[0] = (self.r_clamp - self.lo[0]) / self.rng[0];
        }
        o
    }

    /// The innermost knot's separation, in bohr — the value `r_clamp` takes when the fence is
    /// declared, read off the knots and never chosen.
    pub fn r_knot_floor(&self) -> f64 {
        self.lo[0]
    }

    /// The table's energy at one contact's coordinates.
    pub fn eval(&self, y: [f64; CT_DIM]) -> f64 {
        self.eval_grad(y).0
    }

    /// The energy and its gradient with respect to the four COORDINATES.
    pub fn eval_grad(&self, y: [f64; CT_DIM]) -> (f64, [f64; CT_DIM]) {
        if !self.is_loaded() {
            return (0.0, [0.0; CT_DIM]);
        }
        let n = self.n;
        let ys = self.scaled(y);
        let mut sv = self.w[n];
        let mut ds = [0.0f64; CT_DIM];
        for j in 0..CT_DIM {
            sv += self.w[n + 1 + j] * ys[j];
            ds[j] = self.w[n + 1 + j];
        }
        for i in 0..n {
            let xi = self.scaled(self.x[i]);
            let mut d2 = 0.0;
            let mut dd = [0.0f64; CT_DIM];
            for j in 0..CT_DIM {
                dd[j] = ys[j] - xi[j];
                d2 += dd[j] * dd[j];
            }
            let rho = d2.sqrt();
            sv += self.w[i] * rho * d2;
            // ∇‖x‖³ = 3‖x‖ x — continuous at the knot, and zero there
            let k = 3.0 * self.w[i] * rho;
            for j in 0..CT_DIM {
                ds[j] += k * dd[j];
            }
        }
        let ex = (-self.c0 * y[0]).exp();
        let e = -sv * ex;
        let mut ge = [0.0f64; CT_DIM];
        for j in 0..CT_DIM {
            ge[j] = -(ds[j] / self.rng[j]) * ex;
        }
        // below the declared fence the shape does not move with `r`, so neither does its
        // derivative — the jump this leaves in the force at `r_clamp` is the fence's price
        if self.r_clamp > 0.0 && y[0] < self.r_clamp {
            ge[0] = 0.0;
        }
        ge[0] += self.c0 * sv * ex;
        (e, ge)
    }

    /// The whole term on one contact: its energy and the gradient on `[H, O_a, O_d, h₁, h₂]`.
    pub fn serve(&self, xh: [f64; 3], xa: [f64; 3], xd: [f64; 3], h1: [f64; 3], h2: [f64; 3]) -> (f64, [[f64; 3]; 5]) {
        let (y, gy) = ct_coords(xh, xa, xd, h1, h2);
        let (e, ge) = self.eval_grad(y);
        let mut g = [[0.0f64; 3]; 5];
        for j in 0..CT_DIM {
            for i in 0..5 {
                for c in 0..3 {
                    g[i][c] += ge[j] * gy[j][i][c];
                }
            }
        }
        (e, g)
    }

    /// THE SMOOTH SERVING RULE ([`CtServe::Blend`]), on ONE unordered pair of units, with the
    /// FULL analytic force on every atom of every contact.
    ///
    /// `pts[k]` is contact `k`'s five atoms in [`ct_coords`]' own order `[H, O_a, O_d, h₁, h₂]`,
    /// every position given relative to ONE common origin for the whole pair (the caller
    /// passes minimum-image deltas, so the term reduces to the minimum image the way every
    /// other seam term does). The four contacts are the pair's own: each unit's two hydrogens
    /// against the other unit's oxygen.
    ///
    /// ```text
    /// Ẽ_k = S(r_k) · E_table(coords_k)                the switch applies PER CONTACT
    /// w_k = e^{−β r_k} / Σ_j e^{−β r_j}               a partition of unity on the contact distances
    /// E   = Σ_k w_k Ẽ_k
    /// ∇E  = Σ_k w_k ∇Ẽ_k + Σ_k Ẽ_k ∇w_k
    ///     = Σ_k w_k ∇Ẽ_k − β Σ_k w_k (Ẽ_k − E) ∇r_k
    /// ```
    ///
    /// — the second line is `∇w_k = w_k(−β ∇r_k + β Σ_j w_j ∇r_j)` summed and collected, and
    /// it is the term the rule's first specification omitted. `∇r_k` lives on contact `k`'s
    /// own two atoms: `+u_k` on its hydrogen and `−u_k` on its acceptor oxygen, with
    /// `u_k = (H_k − O_{a,k})/r_k`.
    ///
    /// Returns the pair's energy and `∇E` — the GRADIENT, not the force — laid out per contact
    /// and per atom, so the caller adds each contact's five contributions into its own atom
    /// slots (an atom appears in several contacts and the contributions add). The weights are
    /// formed against the shortest contact so the exponentials cannot overflow, and a contact
    /// at or past `r_cut` contributes an exact zero to `Ẽ_k` and to `∇Ẽ_k` while still carrying
    /// its weight.
    pub fn serve_blend(
        &self,
        model: &SeamModel,
        pts: &[[[f64; 3]; 5]; CT_CONTACTS],
    ) -> (f64, [[[f64; 3]; 5]; CT_CONTACTS]) {
        let mut grad = [[[0.0f64; 3]; 5]; CT_CONTACTS];
        if !self.is_loaded() || self.beta <= 0.0 {
            return (0.0, grad);
        }
        // the contact separations and their unit vectors, `∇r_k`'s own two atoms
        let mut r = [0.0f64; CT_CONTACTS];
        let mut u = [[0.0f64; 3]; CT_CONTACTS];
        let mut rmin = f64::INFINITY;
        for k in 0..CT_CONTACTS {
            let d = [pts[k][0][0] - pts[k][1][0], pts[k][0][1] - pts[k][1][1], pts[k][0][2] - pts[k][1][2]];
            let rk = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-9);
            r[k] = rk;
            u[k] = [d[0] / rk, d[1] / rk, d[2] / rk];
            if rk < rmin {
                rmin = rk;
            }
        }
        // the partition of unity, formed against the shortest contact
        let mut w = [0.0f64; CT_CONTACTS];
        let mut z = 0.0f64;
        for k in 0..CT_CONTACTS {
            w[k] = (-self.beta * (r[k] - rmin)).exp();
            z += w[k];
        }
        for k in 0..CT_CONTACTS {
            w[k] /= z;
        }
        // each contact's switched energy and its own gradient
        let mut et = [0.0f64; CT_CONTACTS];
        let mut g = [[[0.0f64; 3]; 5]; CT_CONTACTS];
        for k in 0..CT_CONTACTS {
            let (sw, dsw) = model.switch(r[k]);
            if sw == 0.0 {
                continue;
            }
            let (e, ge) = self.serve(pts[k][0], pts[k][1], pts[k][2], pts[k][3], pts[k][4]);
            et[k] = sw * e;
            for i in 0..5 {
                for c in 0..3 {
                    g[k][i][c] = sw * ge[i][c];
                }
            }
            if dsw != 0.0 {
                for c in 0..3 {
                    let d = dsw * e * u[k][c];
                    g[k][0][c] += d;
                    g[k][1][c] -= d;
                }
            }
        }
        let e_pair: f64 = (0..CT_CONTACTS).map(|k| w[k] * et[k]).sum();
        for k in 0..CT_CONTACTS {
            // the first term, −Σ w_k ∇Ẽ_k, and the second collected on `∇r_k`
            let s = -self.beta * w[k] * (et[k] - e_pair);
            for i in 0..5 {
                for c in 0..3 {
                    grad[k][i][c] = w[k] * g[k][i][c];
                }
            }
            for c in 0..3 {
                grad[k][0][c] += s * u[k][c];
                grad[k][1][c] -= s * u[k][c];
            }
        }
        (e_pair, grad)
    }

    /// The deepest reading the table can return at separation `r`, from its own knots. This is
    /// what the boundedness walk takes for the transfer row: `−max_i s_i · exp(−c₀ r)`.
    #[inline]
    pub fn deepest(&self, r: f64) -> f64 {
        if !self.is_loaded() { 0.0 } else { -self.deepest_shape * (-self.c0 * r).exp() }
    }

    /// THE DEEPEST READING THE SERVED TERM CAN RETURN AT SEPARATION `r` — the interpolant
    /// ITSELF, evaluated at that separation over every angular geometry a real contact can
    /// present, together with the coordinates that reach it.
    ///
    /// This is what the boundedness walk takes, and it is not [`CtTable::deepest`]. That one
    /// carries the largest knot SHAPE inward under `exp(−c₀ r)`, and the largest shape of this
    /// map sits at 4.5 bohr where the twist family decays more slowly than the divided-out
    /// exponent; carrying it to contact describes a term the law does not serve. This one sets
    /// `r` to the walk's own radius and asks the spline, so the table's behaviour below its
    /// innermost knot — its linear tail in the scaled box, or the declared fence if `r_clamp`
    /// is set — is what is walked.
    ///
    /// The angular scan respects what a real contact can present, which is NOT the knot box:
    /// `u` is a unit vector in the acceptor's orthonormal frame, so `(u·n̂)² ≤ 1 − p²` and
    /// therefore `|q| ≤ 1 − p²`. The box's corners at `|p| = |q| = 1` are unreachable and are
    /// not walked. `cos θ_d` is scanned over its full physical `[−1, 1]`, which is WIDER than
    /// the knot box, because a real donor can point anywhere.
    ///
    /// A MINIMUM OVER A GRID IS AN UPPER BOUND ON THE TRUE MINIMUM, so this is refined: a
    /// coarse pass at `n` points per axis, then a fine pass over the coarse cell around the
    /// best point. The freeze reports both resolutions, because a walk that reads its depth off
    /// a grid must say how fine the grid was.
    pub fn deepest_served(&self, r: f64, n: usize) -> (f64, [f64; CT_DIM]) {
        if !self.is_loaded() {
            return (0.0, [r, 0.0, 0.0, 0.0]);
        }
        let n = n.max(3);
        let scan = |cd_lo: f64, cd_hi: f64, p_lo: f64, p_hi: f64, s_lo: f64, s_hi: f64| -> (f64, [f64; CT_DIM]) {
            let mut best = f64::INFINITY;
            let mut arg = [r, cd_lo, p_lo, 0.0];
            for i in 0..n {
                let cd = cd_lo + (cd_hi - cd_lo) * (i as f64) / ((n - 1) as f64);
                for j in 0..n {
                    let p = p_lo + (p_hi - p_lo) * (j as f64) / ((n - 1) as f64);
                    // `s` is (u·n̂)² as a fraction of what the polar angle leaves for it
                    let room = (1.0 - p * p).max(0.0);
                    for k in 0..n {
                        let s = s_lo + (s_hi - s_lo) * (k as f64) / ((n - 1) as f64);
                        let q = 2.0 * s.clamp(0.0, 1.0) * room + p * p - 1.0;
                        let y = [r, cd, p, q];
                        let e = self.eval(y);
                        if e < best {
                            best = e;
                            arg = y;
                        }
                    }
                }
            }
            (best, arg)
        };
        let (_, a0) = scan(-1.0, 1.0, -1.0, 1.0, 0.0, 1.0);
        // the fine pass, one coarse cell either side of the coarse winner
        let d = 2.0 / ((n - 1) as f64);
        let s0 = {
            let room = (1.0 - a0[2] * a0[2]).max(1e-12);
            ((a0[3] - a0[2] * a0[2] + 1.0) / (2.0 * room)).clamp(0.0, 1.0)
        };
        let ds = 1.0 / ((n - 1) as f64);
        scan(
            (a0[1] - d).max(-1.0),
            (a0[1] + d).min(1.0),
            (a0[2] - d).max(-1.0),
            (a0[2] + d).min(1.0),
            (s0 - ds).max(0.0),
            (s0 + ds).min(1.0),
        )
    }

    /// How far the table reaches at `budget`, from its own deepest shape and `c₀` — the same
    /// arithmetic [`SeamModel::reach`] does for an exponential, on the table's own numbers.
    pub fn reach(&self, budget: f64) -> f64 {
        if !self.is_loaded() || !(self.deepest_shape.abs() > budget) || !(self.c0 > 0.0) {
            0.0
        } else {
            (self.deepest_shape.abs() / budget).ln() / self.c0
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
    fn the_transfer_table_interpolates_its_knots_and_its_force_is_its_derivative() {
        // five water-dimer-shaped frames, each giving one contact and so one knot. The VALUES
        // here are a unit test's, not a record's: what is under test is the interpolant and its
        // gradient, and both are indifferent to what the numbers mean.
        let (r_oh, th) = (1.9435738400f64, 1.6887434037f64);
        let (s2, c2) = ((0.5 * th).sin(), (0.5 * th).cos());
        // `bend` swings the donor's O–H off the axis, so `cos θ_d` is not one value on every
        // knot — an axis whose knots all agree is refused by `finish`, and rightly
        let frame = |roo: f64, tilt: f64, twist: f64, bend: f64| -> ([f64; 3], [f64; 3], [f64; 3], [f64; 3], [f64; 3]) {
            let (st, ct) = (tilt.sin(), tilt.cos());
            let (sw, cw) = (twist.sin(), twist.cos());
            let turn = |p: [f64; 3]| -> [f64; 3] {
                let (x1, y1, z1) = (p[0] * cw - p[1] * sw, p[0] * sw + p[1] * cw, p[2]);
                [x1, y1 * ct - z1 * st, roo + y1 * st + z1 * ct]
            };
            ([r_oh * bend.sin(), 0.0, r_oh * bend.cos()], [0.0, 0.0, roo], [0.0, 0.0, 0.0],
             turn([r_oh * s2, 0.0, r_oh * c2]), turn([-r_oh * s2, 0.0, r_oh * c2]))
        };
        let geoms = [
            frame(5.48, 0.0, 0.0, 0.0),
            frame(5.48, 1.0, 0.0, 0.0),
            frame(5.48, 1.0, 1.2, 0.25),
            frame(5.86, 0.4, 0.7, 0.0),
            frame(6.30, 2.2, 0.3, 0.55),
            frame(5.10, 1.6, 1.5, 0.15),
            frame(6.60, 0.8, 1.1, 0.40),
        ];
        let vals = [-9.3e-3, -1.24e-2, -1.11e-2, -5.4e-3, -2.7e-3, -1.85e-2, -1.1e-3];
        let mut tbl = CtTable::empty();
        assert!(tbl.begin(geoms.len(), 1.40));
        for (i, g) in geoms.iter().enumerate() {
            let (y, _) = ct_coords(g.0, g.1, g.2, g.3, g.4);
            assert!(tbl.knot(i, y, vals[i]), "knot {i}");
        }
        assert_eq!(tbl.finish(), CtLoad::Ok, "the table loads");
        assert!(tbl.is_loaded() && tbl.knots() == geoms.len());
        // THE INTERPOLANT IS THE TERM: it reproduces every knot it was built on
        assert!(tbl.worst_knot_miss < 1e-14, "knot reproduction {:.3e}", tbl.worst_knot_miss);
        for (i, g) in geoms.iter().enumerate() {
            let (e, _) = tbl.serve(g.0, g.1, g.2, g.3, g.4);
            assert!((e - vals[i]).abs() < 1e-14, "knot {i}: {e} vs {}", vals[i]);
        }
        // the deepest reading is the deepest knot's shape, and the reach is that shape's
        assert!(tbl.deepest_shape > 0.0 && tbl.deepest(3.0) < 0.0);
        assert!((tbl.reach(1e-10) - (tbl.deepest_shape / 1e-10).ln() / 1.40).abs() < 1e-9);
        // the analytic gradient against a central difference, on every coordinate of every atom,
        // at geometries BETWEEN the knots as well as at them
        for probe in [frame(5.48, 0.0, 0.0, 0.0), frame(5.63, 0.9, 0.55, 0.20), frame(6.05, 1.75, 1.35, 0.45)] {
            let pts = [probe.0, probe.1, probe.2, probe.3, probe.4];
            let (_, g) = tbl.serve(pts[0], pts[1], pts[2], pts[3], pts[4]);
            let h = 1e-6;
            for i in 0..5 {
                for c in 0..3 {
                    let mut pp = pts;
                    pp[i][c] += h;
                    let (ep, _) = tbl.serve(pp[0], pp[1], pp[2], pp[3], pp[4]);
                    pp[i][c] -= 2.0 * h;
                    let (em, _) = tbl.serve(pp[0], pp[1], pp[2], pp[3], pp[4]);
                    let fd = (ep - em) / (2.0 * h);
                    assert!((g[i][c] - fd).abs() <= 1e-8 * (1.0 + fd.abs()), "atom {i} coord {c}: analytic {} vs fd {fd}", g[i][c]);
                }
            }
            // translation invariance: the gradients sum to zero, so the term posts no net force
            for c in 0..3 {
                let sum: f64 = (0..5).map(|i| g[i][c]).sum();
                assert!(sum.abs() < 1e-10, "gradients sum to {sum}");
            }
        }
        // ROTATION INVARIANCE: the energy is a function of four scalars, so turning the whole
        // frame cannot move it. (A table on `u·n̂` rather than `q` would fail this under a
        // relabelling of the acceptor's hydrogens; `q` is invariant under that too, below.)
        let g = frame(5.63, 0.9, 0.55, 0.20);
        let (e0, _) = tbl.serve(g.0, g.1, g.2, g.3, g.4);
        let rot = |p: [f64; 3]| -> [f64; 3] {
            let (s, c) = (0.7f64.sin(), 0.7f64.cos());
            let (x, y, z) = (p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c);
            [x + 3.0, y * c - z * s - 1.0, y * s + z * c + 2.0]
        };
        let (e1, _) = tbl.serve(rot(g.0), rot(g.1), rot(g.2), rot(g.3), rot(g.4));
        assert!((e0 - e1).abs() < 1e-12, "rotation moved the table: {e0} vs {e1}");
        let (e2, _) = tbl.serve(g.0, g.1, g.2, g.4, g.3);
        assert!((e0 - e2).abs() < 1e-12, "swapping the acceptor's hydrogens moved the table: {e0} vs {e2}");
        // the refusals, each by name
        let mut bad = CtTable::empty();
        assert!(!bad.begin(MAX_CT_KNOTS + 1, 1.4) && bad.status == CtLoad::TooManyKnots);
        assert!(!bad.begin(3, 1.4) && bad.status == CtLoad::TooFewKnots);
        assert!(!bad.begin(7, 0.0) && bad.status == CtLoad::NotFinite);
        let mut dup = CtTable::empty();
        assert!(dup.begin(geoms.len(), 1.40));
        for i in 0..geoms.len() {
            let g = geoms[if i == 1 { 0 } else { i }];
            let (y, _) = ct_coords(g.0, g.1, g.2, g.3, g.4);
            dup.knot(i, y, vals[i]);
        }
        assert_eq!(dup.finish(), CtLoad::DuplicateKnot, "two knots at one site are refused, not averaged");
        assert_eq!(dup.eval([3.5, -1.0, -1.0, 0.0]), 0.0, "a table that did not load serves nothing");
    }

    #[test]
    fn the_mode_selects_one_shape_and_the_walk_takes_the_one_it_serves() {
        let q = 0.231380372;
        let kt = 9.28e-4;
        let r_min = [4.724315, 2.780741, 1.314606];
        // the selector: three shapes, exclusive, and every record before CT-3 reads one of two
        let pair = SeamModel { p_ct: 1.47488, c_ct: 1.46, ..SeamModel::NO_WALL };
        assert_eq!(pair.ct_mode(), CtMode::Pair);
        let ang = SeamModel { m_ct: 1, ..pair };
        assert_eq!(ang.ct_mode(), CtMode::Angular);
        assert_eq!(SeamModel { ct_table_on: true, ..ang }.ct_mode(), CtMode::Table, "the table replaces the family, it does not multiply it");
        assert_eq!(SeamModel { ct_table_on: true, ..pair }.ct_mode(), CtMode::Table);
        // `bounded_ct` with the pair term IS `bounded`, on CT-2's own admitted law and on CT-1's
        for m in [
            SeamModel { a: 948.048736, b: 2.40, p: 22.174044, c: 2.44, a_oh: 22.586054, b_oh: 2.20, a_hh: 1.525046, b_hh: 1.75, p_hh: 0.016971, c_hh: 1.02, p_ct: 1.371043, c_ct: 1.40, m_ct: 1, ..SeamModel::NO_WALL },
            SeamModel { a: 948.048736, b: 2.40, p: 23.704848, c: 2.44, a_oh: 22.586054, b_oh: 2.20, a_hh: 1.525046, b_hh: 1.75, p_hh: 158.7891, c_hh: 4.00, p_ct: 1.47488, c_ct: 1.46, ..SeamModel::NO_WALL },
        ] {
            assert_eq!(m.bounded(q, r_min, kt), m.bounded_ct(q, r_min, kt, &|r| m.charge_transfer(r)));
            assert_eq!(m.hole(q), m.hole_ct(q, &|r| m.charge_transfer(r)));
        }
        // CT-2's law is the admitted one, and a DEEPER transfer row breaks it — which is the
        // whole reason the walk takes the table's deepest reading rather than its linear one
        let ct2 = SeamModel { a: 948.048736, b: 2.40, p: 22.174044, c: 2.44, a_oh: 22.586054, b_oh: 2.20, a_hh: 1.525046, b_hh: 1.75, p_hh: 0.016971, c_hh: 1.02, p_ct: 1.371043, c_ct: 1.40, m_ct: 1, ..SeamModel::NO_WALL };
        assert_eq!(ct2.bounded(q, r_min, kt), None, "CT-2's law is admitted");
        let deep = ct2.bounded_ct(q, r_min, kt, &|r| -2.92 * (-1.40 * r).exp());
        assert!(deep.as_deref().map_or(false, |m| m.contains("H–O potential is not positive at contact")), "{deep:?}");
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
