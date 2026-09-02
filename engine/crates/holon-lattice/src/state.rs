//! The local state space, its fibers, and the collision laws that live on them.
//!
//! # The census is a classification, not a control
//!
//! A collision is local and must fix the conserved label. It therefore permutes within
//! `(N,P)` fibers, which means `Core/Lattice.lean`'s `sector_dims` is a statement about the
//! DYNAMICS: a law is the identity on all 44 fibers of dimension 1, free on the two of
//! dimension 3 and the seven of dimension 2, and nothing else is available. The group is
//! `S₃ × (S₂)⁷ × S₃`, order 4608, and [`Model::collision_laws`] enumerates it.
//!
//! # Two implementations of the label, on purpose
//!
//! [`Model::label`] is generic over a direction set because HPP-4 needs one too. For FHP-6
//! it is checked against `ciris_sim_core::regplus::sector` on all 64 states
//! ([`tests::fhp_label_agrees_with_the_pinned_runtime_object`]) — that crate's table is the
//! one `Core/Lattice.lean` pins, so the check ties this crate to the machine-checked object
//! rather than to a retyped copy of it.

use ciris_sim_core::regplus::DIRECTIONS as FHP_DIRECTIONS;

/// The order of the full sector-preserving collision group on FHP-6: `3! · (2!)⁷ · 3!`.
/// Derived by [`Model::collision_group_order`] rather than typed, and pinned here so a
/// change to the fiber structure cannot pass silently.
pub const COLLISION_GROUP_ORDER: u64 = 4608;

/// A lattice-gas model: a direction set, and everything the fiber structure forces.
///
/// The four HPP directions are `±x, ±y` on the square lattice. The six FHP directions are
/// `regplus::DIRECTIONS`, in the axial integer coordinates `Core/Lattice.lean` uses; the
/// Euclidean embedding needed for [`crate::isotropy`] is applied there and only there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Model {
    pub name: &'static str,
    pub dirs: Vec<[i64; 2]>,
}

/// The conserved label of one local state: occupancy and the two momentum components.
pub type Label = (u32, i64, i64);

impl Model {
    /// FHP-6 on the triangular lattice, directions taken from `regplus` and never retyped.
    pub fn fhp6() -> Self {
        Self { name: "FHP-6", dirs: FHP_DIRECTIONS.to_vec() }
    }

    /// HPP-4 on the square lattice (Hardy–Pomeau–de Pazzis 1973). Present as a CONTROL: its
    /// per-line momentum invariant is an exactly held non-global chart, and its fourth-rank
    /// tensor is anisotropic. An instrument that has only ever seen FHP has not been shown
    /// able to return either of those answers.
    pub fn hpp4() -> Self {
        Self { name: "HPP-4", dirs: vec![[1, 0], [0, 1], [-1, 0], [0, -1]] }
    }

    #[inline]
    pub fn n_dirs(&self) -> usize {
        self.dirs.len()
    }

    /// Number of local states, `2^n_dirs`.
    #[inline]
    pub fn n_states(&self) -> usize {
        1 << self.n_dirs()
    }

    /// The direction opposite `d`. Both direction sets are centrally symmetric with the
    /// opposite listed half a turn later, which [`tests::opposites_are_opposite`] checks
    /// by arithmetic on the vectors rather than by trusting the ordering.
    #[inline]
    pub fn opposite(&self, d: usize) -> usize {
        (d + self.n_dirs() / 2) % self.n_dirs()
    }

    /// The conserved `(N, Px, Py)` label — `Core/Lattice.lean`'s `np`.
    pub fn label(&self, s: u8) -> Label {
        let (mut n, mut px, mut py) = (0, 0, 0);
        for d in 0..self.n_dirs() {
            if s >> d & 1 == 1 {
                n += 1;
                px += self.dirs[d][0];
                py += self.dirs[d][1];
            }
        }
        (n, px, py)
    }

    /// The fibers of the label, each sorted, ordered by first member. This is the census.
    pub fn fibers(&self) -> Vec<Vec<u8>> {
        let mut labels: Vec<Label> = Vec::new();
        let mut out: Vec<Vec<u8>> = Vec::new();
        for s in 0..self.n_states() as u8 {
            let l = self.label(s);
            match labels.iter().position(|&k| k == l) {
                Some(i) => out[i].push(s),
                None => {
                    labels.push(l);
                    out.push(vec![s]);
                }
            }
        }
        out
    }

    /// `(sector count, histogram of fiber dimensions)`. For FHP-6 this must be
    /// `(53, [(1,44), (2,7), (3,2)])` — the instrument control, reproducing
    /// `Core/Lattice.lean`, `regplus.rs` and `holon-mesh::fchc` alike.
    pub fn census(&self) -> (usize, Vec<(usize, usize)>) {
        let f = self.fibers();
        let mut hist: Vec<(usize, usize)> = Vec::new();
        for fiber in &f {
            match hist.iter_mut().find(|(d, _)| *d == fiber.len()) {
                Some((_, c)) => *c += 1,
                None => hist.push((fiber.len(), 1)),
            }
        }
        hist.sort_unstable();
        (f.len(), hist)
    }

    /// `|S_{d₁} × … × S_{dₖ}|` over the fiber dimensions: the size of the space of REG+
    /// collision laws this model admits. 4608 for FHP-6.
    pub fn collision_group_order(&self) -> u64 {
        self.fibers()
            .iter()
            .map(|f| (1..=f.len() as u64).product::<u64>())
            .product()
    }

    /// The states a fiber move can touch: those lying in a fiber of dimension ≥ 2. Twenty
    /// of the 64 for FHP-6.
    pub fn movable(&self) -> Vec<u8> {
        let mut out: Vec<u8> =
            self.fibers().into_iter().filter(|f| f.len() > 1).flatten().collect();
        out.sort_unstable();
        out
    }

    /// The cyclic successor of `s` within its own fiber, or `None` for a fiber of
    /// dimension 1. **This is the fiber move**: it leaves `(N,P)` — hence every block chart
    /// simultaneously — exactly unchanged, which is what makes one perturbation serve the
    /// whole chart family without a confound.
    pub fn fiber_successor(&self, s: u8) -> Option<u8> {
        let f = self.fibers().into_iter().find(|f| f.contains(&s))?;
        if f.len() < 2 {
            return None;
        }
        let i = f.iter().position(|&t| t == s).unwrap();
        Some(f[(i + 1) % f.len()])
    }

    /// A state in a DIFFERENT fiber, or `None` if the model has only one. The probe's
    /// POSITIVE control: this perturbation changes the chart by construction, so a probe
    /// that does not fire on it is not measuring anything.
    pub fn other_fiber_state(&self, s: u8) -> Option<u8> {
        let l = self.label(s);
        (0..self.n_states() as u8).find(|&t| self.label(t) != l)
    }

    /// FHP-I: the 3-cycle on the head-on fiber `{9,18,36}` (the Lean's `three_route_sector`)
    /// and the swap on the three-body fiber `{21,42}`. `chirality` selects the sense of the
    /// 60° rotation.
    ///
    /// The identity on the other 50 states is FORCED, not chosen: 44 of them are alone in
    /// their fiber, and the remaining six lie in fibers this law leaves fixed.
    pub fn fhp_i(&self, chirality: bool) -> Vec<u8> {
        assert_eq!(self.n_dirs(), 6, "FHP-I is defined on the six-direction chart");
        let mut c: Vec<u8> = (0..64).collect();
        if chirality {
            c[9] = 18;
            c[18] = 36;
            c[36] = 9;
        } else {
            c[9] = 36;
            c[36] = 18;
            c[18] = 9;
        }
        c[21] = 42;
        c[42] = 21;
        c
    }

    /// HPP's collision: the single head-on pair rotates a quarter turn. `{5, 10}` are the
    /// two-particle head-on states on the square lattice.
    pub fn hpp_collision(&self) -> Vec<u8> {
        assert_eq!(self.n_dirs(), 4, "HPP's collision is defined on the four-direction chart");
        let mut c: Vec<u8> = (0..16).collect();
        c[5] = 10;
        c[10] = 5;
        c
    }

    /// The identity collision. Present so that "the defect survives removing the collision"
    /// is a run and not an argument.
    pub fn identity_collision(&self) -> Vec<u8> {
        (0..self.n_states() as u8).collect()
    }

    /// Is `c` a permutation of the local states? M-NONBIJECTIVE-STEP: any map called
    /// dynamics must be VERIFIED bijective, never assumed from how it was built.
    pub fn is_bijection(&self, c: &[u8]) -> bool {
        let n = self.n_states();
        if c.len() != n {
            return false;
        }
        let mut seen = vec![false; n];
        for &t in c {
            if t as usize >= n || seen[t as usize] {
                return false;
            }
            seen[t as usize] = true;
        }
        true
    }

    /// Does `c` fix the conserved label on every state?
    pub fn is_sector_preserving(&self, c: &[u8]) -> bool {
        (0..self.n_states() as u8).all(|s| self.label(c[s as usize]) == self.label(s))
    }

    /// Every sector-preserving collision law, in a deterministic order. 4608 of them for
    /// FHP-6. Used to show that the closure defect belongs to the LATTICE and not to any
    /// one chosen law (M-ONE-MODEL-DELTA).
    pub fn collision_laws(&self) -> Vec<Vec<u8>> {
        let nontrivial: Vec<Vec<u8>> =
            self.fibers().into_iter().filter(|f| f.len() > 1).collect();
        let per: Vec<Vec<Vec<u8>>> = nontrivial.iter().map(|f| permutations(f)).collect();
        let mut out = Vec::new();
        let mut idx = vec![0usize; per.len()];
        loop {
            let mut c: Vec<u8> = (0..self.n_states() as u8).collect();
            for (k, fiber) in nontrivial.iter().enumerate() {
                for (a, b) in fiber.iter().zip(per[k][idx[k]].iter()) {
                    c[*a as usize] = *b;
                }
            }
            out.push(c);
            let mut k = per.len();
            loop {
                if k == 0 {
                    return out;
                }
                k -= 1;
                idx[k] += 1;
                if idx[k] < per[k].len() {
                    break;
                }
                idx[k] = 0;
            }
        }
    }
}

fn permutations(items: &[u8]) -> Vec<Vec<u8>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut out = Vec::new();
    for i in 0..items.len() {
        let mut rest = items.to_vec();
        let head = rest.remove(i);
        for mut p in permutations(&rest) {
            p.insert(0, head);
            out.push(p);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The instrument control, and the reason it is one: this crate's own label routine is
    /// checked against the runtime table that `Core/Lattice.lean` pins, on every state. A
    /// census reproduced only by the code that defines it is not a control.
    #[test]
    fn fhp_label_agrees_with_the_pinned_runtime_object() {
        let m = Model::fhp6();
        for s in 0..64u8 {
            let pinned = ciris_sim_core::regplus::sector(s);
            assert_eq!(
                m.label(s),
                (pinned.occupancy as u32, pinned.momentum[0] as i64, pinned.momentum[1] as i64),
                "state {s}"
            );
        }
    }

    #[test]
    fn the_census_reproduces_the_lean_theorem() {
        let (sectors, hist) = Model::fhp6().census();
        assert_eq!(sectors, 53);
        assert_eq!(hist, vec![(1, 44), (2, 7), (3, 2)]);
    }

    /// The classification: the census says where a collision law may act, and how many
    /// there are. `three_route_sector` is the dimension-3 fiber FHP-I turns.
    #[test]
    fn the_census_classifies_the_collision_laws() {
        let m = Model::fhp6();
        assert_eq!(m.collision_group_order(), COLLISION_GROUP_ORDER);
        let dim3: Vec<Vec<u8>> =
            m.fibers().into_iter().filter(|f| f.len() == 3).collect();
        assert_eq!(dim3, vec![vec![9, 18, 36], vec![27, 45, 54]]);
        assert_eq!(m.label(9), (2, 0, 0));
        assert_eq!(m.label(27), (4, 0, 0));
        assert_eq!(m.label(21), (3, 0, 0));
        assert_eq!(m.movable().len(), 20);
        assert_eq!(m.collision_laws().len(), COLLISION_GROUP_ORDER as usize);
    }

    /// Every law in the enumerated group is a bijection AND conserves; and the enumeration
    /// has no duplicates, which is what makes its cardinality mean something.
    #[test]
    fn every_enumerated_law_is_a_conserving_bijection() {
        let m = Model::fhp6();
        let laws = m.collision_laws();
        for c in &laws {
            assert!(m.is_bijection(c));
            assert!(m.is_sector_preserving(c));
        }
        let mut sorted = laws.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), laws.len(), "the enumeration repeats a law");
    }

    #[test]
    fn fhp_i_and_hpp_are_conserving_bijections_and_hpp_is_not_fhp() {
        let f = Model::fhp6();
        for chirality in [true, false] {
            let c = f.fhp_i(chirality);
            assert!(f.is_bijection(&c) && f.is_sector_preserving(&c));
            let acting: Vec<u8> = (0..64u8).filter(|&s| c[s as usize] != s).collect();
            assert_eq!(acting, vec![9, 18, 21, 36, 42]);
        }
        let h = Model::hpp4();
        let c = h.hpp_collision();
        assert!(h.is_bijection(&c) && h.is_sector_preserving(&c));
        // HPP's census is the sharper half of the contrast: 15 sectors, and exactly ONE of
        // dimension above 1 — the head-on pair {5,10}. So HPP admits a collision group of
        // order 2 where FHP admits 4608, which is the classification saying in advance that
        // the square lattice has almost no dynamics to choose from.
        assert_eq!(h.census(), (15, vec![(1, 14), (2, 1)]));
        assert_eq!(h.collision_group_order(), 2);
        assert_eq!(h.movable(), vec![5, 10]);
    }

    /// M-NONBIJECTIVE-STEP's plant, pre-checked to fire here rather than in a postmortem:
    /// a table that sends two states to one is caught, and the check is not satisfied by
    /// conservation (the plant conserves and still fails).
    #[test]
    fn the_bijectivity_check_fires_on_a_planted_collision() {
        let m = Model::fhp6();
        let mut c = m.fhp_i(true);
        c[18] = 9;
        assert!(!m.is_bijection(&c), "the non-injective plant was not caught");
        assert!(m.is_sector_preserving(&c), "the plant should still conserve, isolating the gate");
    }

    #[test]
    fn opposites_are_opposite() {
        for m in [Model::fhp6(), Model::hpp4()] {
            for d in 0..m.n_dirs() {
                let o = m.opposite(d);
                assert_eq!(m.dirs[d][0] + m.dirs[o][0], 0, "{} dir {d}", m.name);
                assert_eq!(m.dirs[d][1] + m.dirs[o][1], 0, "{} dir {d}", m.name);
            }
        }
    }

    /// The fiber move preserves the label exactly — the property the whole closure probe
    /// rests on — and the positive control's move does not.
    #[test]
    fn the_fiber_move_preserves_the_label_and_the_control_move_does_not() {
        let m = Model::fhp6();
        let mut moved = 0;
        for s in 0..64u8 {
            match m.fiber_successor(s) {
                Some(t) => {
                    assert_ne!(t, s);
                    assert_eq!(m.label(t), m.label(s), "fiber move changed the label at {s}");
                    moved += 1;
                }
                None => assert_eq!(m.fibers().iter().find(|f| f.contains(&s)).unwrap().len(), 1),
            }
            let o = m.other_fiber_state(s).unwrap();
            assert_ne!(m.label(o), m.label(s));
        }
        assert_eq!(moved, 20, "the fiber move must be available on exactly 20 states");
    }
}
