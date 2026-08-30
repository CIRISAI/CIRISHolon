//! SELECTOR-5 Landscape Sweep module across finite groups |G| <= 128.
//!
//! Evaluates the frozen SELECTOR-4 bootstrap criteria on finite groups:
//!   - Gate C1: Commutator Defect delta_comm > 0 (Non-abelian gauge curvature)
//!   - Gate C2: Order Spectrum Entropy H_ord > 0 (Timescale diversity)
//!   - Gate C3: Orientation Index O(G) > 0 (Non-ambivalent chiral holonomy)
//!   - Gate C4: Schur Multiplier / Spin Cover (|M(G)| > 1 or |Z| >= 2 & |M(G/Z)| > 1)
//!   - Full SELECTOR-4 Gauntlet: C1 & C2 & C3 & C4

use std::collections::{HashMap, HashSet};

/// Finite group representation with Cayley table.
#[derive(Clone, Debug)]
pub struct FiniteGroup {
    pub name: String,
    pub family: String,
    pub order: usize,
    pub mul: Vec<Vec<usize>>,
    pub inv: Vec<usize>,
    pub identity: usize,
    pub is_lie_type: bool,
    pub notes: String,
    pub aliases: Vec<String>,
}

impl FiniteGroup {
    pub fn new(
        name: impl Into<String>,
        family: impl Into<String>,
        order: usize,
        mul: Vec<Vec<usize>>,
        inv: Vec<usize>,
        identity: usize,
        is_lie_type: bool,
        notes: impl Into<String>,
    ) -> Self {
        assert_eq!(mul.len(), order);
        assert_eq!(inv.len(), order);
        Self {
            name: name.into(),
            family: family.into(),
            order,
            mul,
            inv,
            identity,
            is_lie_type,
            notes: notes.into(),
            aliases: Vec::new(),
        }
    }

    /// Conjugacy classes of the group.
    pub fn classes(&self) -> Vec<HashSet<usize>> {
        let n = self.order;
        let mut cls = vec![None; n];
        let mut classes = Vec::new();

        for g in 0..n {
            if cls[g].is_some() {
                continue;
            }
            let mut orbit = HashSet::new();
            for x in 0..n {
                let x_g = self.mul[x][g];
                let conj = self.mul[x_g][self.inv[x]];
                orbit.insert(conj);
            }
            let cid = classes.len();
            for &h in &orbit {
                cls[h] = Some(cid);
            }
            classes.push(orbit);
        }
        classes
    }

    pub fn num_classes(&self) -> usize {
        self.classes().len()
    }

    /// Commutator subgroup [G, G].
    pub fn commutator_subgroup(&self) -> HashSet<usize> {
        let n = self.order;
        let mut comm_set = HashSet::new();
        for a in 0..n {
            for b in 0..n {
                let ab = self.mul[a][b];
                let inv_a_inv_b = self.mul[self.inv[a]][self.inv[b]];
                let comm = self.mul[ab][inv_a_inv_b];
                comm_set.insert(comm);
            }
        }
        let mut comm_grp = comm_set.clone();
        let mut changed = true;
        while changed {
            changed = false;
            let mut new_elems = Vec::new();
            for &x in &comm_grp {
                for &y in &comm_grp {
                    let xy = self.mul[x][y];
                    if !comm_grp.contains(&xy) {
                        new_elems.push(xy);
                        changed = true;
                    }
                }
            }
            for e in new_elems {
                comm_grp.insert(e);
            }
        }
        comm_grp
    }

    /// Center Z(G).
    pub fn center(&self) -> HashSet<usize> {
        let n = self.order;
        let mut cent = HashSet::new();
        for g in 0..n {
            let mut in_center = true;
            for x in 0..n {
                if self.mul[g][x] != self.mul[x][g] {
                    in_center = false;
                    break;
                }
            }
            if in_center {
                cent.insert(g);
            }
        }
        cent
    }

    /// Commutator defect delta_comm = 1 - k(G)/|G|.
    pub fn commutator_defect(&self) -> f64 {
        1.0 - (self.num_classes() as f64 / self.order as f64)
    }

    /// Element orders.
    pub fn element_orders(&self) -> Vec<usize> {
        let n = self.order;
        let mut orders = Vec::with_capacity(n);
        for g in 0..n {
            let mut ord = 1;
            let mut cur = g;
            while cur != self.identity {
                cur = self.mul[cur][g];
                ord += 1;
            }
            orders.push(ord);
        }
        orders
    }

    /// Order spectrum entropy in bits: - sum p(d) log2 p(d).
    pub fn order_entropy(&self) -> f64 {
        let orders = self.element_orders();
        let mut counts = HashMap::new();
        for o in orders {
            *counts.entry(o).or_insert(0) += 1;
        }
        let n = self.order as f64;
        let mut ent = 0.0;
        for &c in counts.values() {
            let p = c as f64 / n;
            if p > 0.0 {
                ent -= p * p.log2();
            }
        }
        ent
    }

    /// Orientation index: fraction of non-ambivalent conjugacy classes.
    pub fn orientation_index(&self) -> f64 {
        let classes = self.classes();
        let mut non_amb = 0;
        for cl in &classes {
            let inv_cl: HashSet<usize> = cl.iter().map(|&x| self.inv[x]).collect();
            if inv_cl != *cl {
                non_amb += 1;
            }
        }
        non_amb as f64 / classes.len() as f64
    }

    pub fn num_non_ambivalent_classes(&self) -> usize {
        let classes = self.classes();
        let mut non_amb = 0;
        for cl in &classes {
            let inv_cl: HashSet<usize> = cl.iter().map(|&x| self.inv[x]).collect();
            if inv_cl != *cl {
                non_amb += 1;
            }
        }
        non_amb
    }

    /// Schur multiplier invariant factors M(G).
    pub fn schur_multiplier(&self) -> Vec<usize> {
        let n = self.order;
        let fam = self.family.as_str();

        if fam == "Cyclic" || (self.commutator_subgroup().len() == 1 && self.element_orders().iter().any(|&o| o == n)) {
            return Vec::new();
        }
        if self.commutator_subgroup().len() == 1 {
            // Abelian product
            let cent = self.center();
            if cent.len() == n {
                // Approximate for cyclic factors
                return Vec::new();
            }
        }
        if fam == "Dihedral" {
            let m = n / 2;
            if m % 2 == 0 && m >= 4 {
                return vec![2];
            }
            return Vec::new();
        }
        if fam == "Dicyclic" || fam == "BinaryPolyhedral" || fam == "Frobenius" {
            return Vec::new();
        }
        if fam == "Alternating" {
            if n == 12 || n == 60 {
                return vec![2];
            }
            return Vec::new();
        }
        if fam == "Symmetric" {
            if n == 24 || n == 120 {
                return vec![2];
            }
            return Vec::new();
        }
        if fam == "Delta3n2" {
            if n == 27 || n == 108 {
                return vec![3, 3];
            }
            if n == 48 {
                return vec![2];
            }
        }
        if fam == "Heisenberg" {
            if n == 8 {
                return vec![2];
            }
            if n == 27 {
                return vec![3, 3];
            }
        }
        Vec::new()
    }

    /// Check if group is a binary spin cover.
    pub fn is_spin_cover(&self) -> bool {
        if self.family == "BinaryPolyhedral" || self.family == "Dicyclic" {
            return true;
        }
        if self.name == "2T" || self.name == "2O" || self.name == "2I" || self.name == "GL(2,3)" {
            return true;
        }
        if self.center().len() >= 2 {
            let q = self.order / 2;
            if q == 12 || q == 24 || q == 60 {
                return true;
            }
        }
        false
    }

    // SELECTOR-4 Gates
    pub fn gate_c1(&self) -> bool {
        self.commutator_subgroup().len() > 1 && self.commutator_defect() > 0.0
    }

    pub fn gate_c2(&self) -> bool {
        self.order_entropy() > 0.0
    }

    pub fn gate_c3(&self) -> bool {
        self.orientation_index() > 0.0
    }

    pub fn gate_c4(&self) -> bool {
        let schur = self.schur_multiplier();
        let has_obstruction = !schur.is_empty() && schur.iter().product::<usize>() > 1;
        has_obstruction || self.is_spin_cover()
    }

    pub fn full_selector_pass(&self) -> bool {
        self.gate_c1() && self.gate_c2() && self.gate_c3() && self.gate_c4()
    }

    pub fn structural_signature(&self) -> (usize, Vec<usize>, Vec<usize>, usize, usize, Vec<usize>, usize) {
        let mut orders = self.element_orders();
        orders.sort_unstable();
        let mut class_sizes: Vec<usize> = self.classes().iter().map(|c| c.len()).collect();
        class_sizes.sort_unstable();
        let comm_size = self.commutator_subgroup().len();
        let cent_size = self.center().len();
        let schur = self.schur_multiplier();
        let non_amb = self.num_non_ambivalent_classes();
        (self.order, orders, class_sizes, comm_size, cent_size, schur, non_amb)
    }
}

// -----------------------------------------------------------------------------
// Builders
// -----------------------------------------------------------------------------

pub fn build_cyclic(n: usize) -> FiniteGroup {
    let mut mul = vec![vec![0; n]; n];
    let mut inv = vec![0; n];
    for i in 0..n {
        for j in 0..n {
            mul[i][j] = (i + j) % n;
        }
        inv[i] = (n - (i % n)) % n;
    }
    FiniteGroup::new(format!("Z_{n}"), "Cyclic", n, mul, inv, 0, true, "Abelian subgroup of U(1)")
}

pub fn build_dihedral(n: usize) -> FiniteGroup {
    let order = 2 * n;
    let mut mul = vec![vec![0; order]; order];
    for i1 in 0..2 {
        for j1 in 0..n {
            let idx1 = i1 * n + j1;
            for i2 in 0..2 {
                for j2 in 0..n {
                    let idx2 = i2 * n + j2;
                    let i3 = (i1 + i2) % 2;
                    let j3 = if i2 == 0 {
                        (j1 + j2) % n
                    } else {
                        (n - (j1 % n) + j2) % n
                    };
                    let idx3 = i3 * n + j3;
                    mul[idx1][idx2] = idx3;
                }
            }
        }
    }
    let mut inv = vec![0; order];
    for i in 0..order {
        inv[i] = (0..order).find(|&j| mul[i][j] == 0).unwrap();
    }
    FiniteGroup::new(format!("D_{}", 2 * n), "Dihedral", order, mul, inv, 0, true, "Dihedral subgroup of SO(3)")
}

pub fn build_dicyclic(n: usize) -> FiniteGroup {
    let order = 4 * n;
    let mut mul = vec![vec![0; order]; order];
    for k1 in 0..2 {
        for j1 in 0..(2 * n) {
            let idx1 = k1 * 2 * n + j1;
            for k2 in 0..2 {
                for j2 in 0..(2 * n) {
                    let idx2 = k2 * 2 * n + j2;
                    let (j3, k3) = match (k1, k2) {
                        (0, 0) => ((j1 + j2) % (2 * n), 0),
                        (0, 1) => ((j1 + j2) % (2 * n), 1),
                        (1, 0) => ((2 * n + j1 - (j2 % (2 * n))) % (2 * n), 1),
                        _ => ((2 * n + j1 - (j2 % (2 * n)) + n) % (2 * n), 0),
                    };
                    mul[idx1][idx2] = k3 * 2 * n + j3;
                }
            }
        }
    }
    let mut inv = vec![0; order];
    for i in 0..order {
        inv[i] = (0..order).find(|&j| mul[i][j] == 0).unwrap();
    }
    let name = if n == 2 { "Q_8".to_string() } else { format!("Dic_{n}") };
    FiniteGroup::new(name, "Dicyclic", order, mul, inv, 0, true, "Binary dihedral in SU(2)")
}

pub fn build_binary_tetrahedral() -> FiniteGroup {
    // 2T quaternions
    fn qmul(p: [i32; 4], q: [i32; 4]) -> [i32; 4] {
        [
            (p[0]*q[0] - p[1]*q[1] - p[2]*q[2] - p[3]*q[3]) / 2,
            (p[0]*q[1] + p[1]*q[0] + p[2]*q[3] - p[3]*q[2]) / 2,
            (p[0]*q[2] - p[1]*q[3] + p[2]*q[0] + p[3]*q[1]) / 2,
            (p[0]*q[3] + p[1]*q[2] - p[2]*q[1] + p[3]*q[0]) / 2,
        ]
    }

    let mut elems = Vec::new();
    for pos in 0..4 {
        for &s in &[2, -2] {
            let mut c = [0; 4];
            c[pos] = s;
            elems.push(c);
        }
    }
    for &s0 in &[1, -1] {
        for &s1 in &[1, -1] {
            for &s2 in &[1, -1] {
                for &s3 in &[1, -1] {
                    elems.push([s0, s1, s2, s3]);
                }
            }
        }
    }
    assert_eq!(elems.len(), 24);
    let mut mul = vec![vec![0; 24]; 24];
    for i in 0..24 {
        for j in 0..24 {
            let p = qmul(elems[i], elems[j]);
            let k = elems.iter().position(|&x| x == p).unwrap();
            mul[i][j] = k;
        }
    }
    let ide = elems.iter().position(|&x| x == [2, 0, 0, 0]).unwrap();
    let mut inv = vec![0; 24];
    for i in 0..24 {
        inv[i] = (0..24).find(|&j| mul[i][j] == ide).unwrap();
    }
    FiniteGroup::new("2T", "BinaryPolyhedral", 24, mul, inv, ide, true, "Binary tetrahedral in SU(2)")
}

pub fn build_frobenius(p: usize, k: usize) -> Option<FiniteGroup> {
    if (p - 1) % k != 0 || k <= 1 {
        return None;
    }
    let order = p * k;
    if order > 128 {
        return None;
    }
    let mut u = None;
    for cand in 2..p {
        let mut pow_val = 1;
        let mut is_k = true;
        for d in 1..=k {
            pow_val = (pow_val * cand) % p;
            if d < k && pow_val == 1 {
                is_k = false;
                break;
            }
        }
        if is_k && pow_val == 1 {
            u = Some(cand);
            break;
        }
    }
    let u = u?;
    let mut pow_u = vec![1; k];
    for j in 1..k {
        pow_u[j] = (pow_u[j - 1] * u) % p;
    }

    let mut mul = vec![vec![0; order]; order];
    for j1 in 0..k {
        for i1 in 0..p {
            let idx1 = j1 * p + i1;
            for j2 in 0..k {
                for i2 in 0..p {
                    let idx2 = j2 * p + i2;
                    let i3 = (i1 + i2 * pow_u[j1]) % p;
                    let j3 = (j1 + j2) % k;
                    let idx3 = j3 * p + i3;
                    mul[idx1][idx2] = idx3;
                }
            }
        }
    }
    let mut inv = vec![0; order];
    for i in 0..order {
        inv[i] = (0..order).find(|&j| mul[i][j] == 0).unwrap();
    }
    Some(FiniteGroup::new(format!("F_{}", p * k), "Frobenius", order, mul, inv, 0, (p, k) == (3, 2), format!("Frobenius group Z_{p}:Z_{k}")))
}

pub fn build_delta_3n2(n: usize) -> Option<FiniteGroup> {
    let order = 3 * n * n;
    if order > 128 {
        return None;
    }
    let mut elems = Vec::new();
    for z in 0..3 {
        for x in 0..n {
            for y in 0..n {
                elems.push((x, y, z));
            }
        }
    }
    let mut mul = vec![vec![0; order]; order];
    for (i, &(x1, y1, z1)) in elems.iter().enumerate() {
        for (j, &(x2, y2, z2)) in elems.iter().enumerate() {
            let (ax, ay) = match z1 {
                0 => (x2, y2),
                1 => ((n * 2 - (x2 + y2) % n) % n, x2),
                _ => (y2, (n * 2 - (x2 + y2) % n) % n),
            };
            let x3 = (x1 + ax) % n;
            let y3 = (y1 + ay) % n;
            let z3 = (z1 + z2) % 3;
            let idx3 = elems.iter().position(|&e| e == (x3, y3, z3)).unwrap();
            mul[i][j] = idx3;
        }
    }
    let ide = elems.iter().position(|&e| e == (0, 0, 0)).unwrap();
    let mut inv = vec![0; order];
    for i in 0..order {
        inv[i] = (0..order).find(|&j| mul[i][j] == ide).unwrap();
    }
    Some(FiniteGroup::new(format!("Delta({order})"), "Delta3n2", order, mul, inv, ide, true, format!("SU(3) subgroup Delta(3*{n}^2)")))
}

pub fn build_direct_product(g1: &FiniteGroup, g2: &FiniteGroup) -> FiniteGroup {
    let n1 = g1.order;
    let n2 = g2.order;
    let order = n1 * n2;
    let mut mul = vec![vec![0; order]; order];
    for i1 in 0..n1 {
        for j1 in 0..n2 {
            let idx1 = i1 * n2 + j1;
            for i2 in 0..n1 {
                for j2 in 0..n2 {
                    let idx2 = i2 * n2 + j2;
                    let i3 = g1.mul[i1][i2];
                    let j3 = g2.mul[j1][j2];
                    let idx3 = i3 * n2 + j3;
                    mul[idx1][idx2] = idx3;
                }
            }
        }
    }
    let ide = g1.identity * n2 + g2.identity;
    let mut inv = vec![0; order];
    for i1 in 0..n1 {
        for j1 in 0..n2 {
            let idx1 = i1 * n2 + j1;
            inv[idx1] = g1.inv[i1] * n2 + g2.inv[j1];
        }
    }
    FiniteGroup::new(
        format!("{}x{}", g1.name, g2.name),
        "DirectProduct",
        order,
        mul,
        inv,
        ide,
        g1.is_lie_type && g2.is_lie_type,
        format!("Direct product of {} and {}", g1.name, g2.name),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_2t_passes_selector() {
        let g2t = build_binary_tetrahedral();
        assert_eq!(g2t.order, 24);
        assert_eq!(g2t.num_classes(), 7);
        assert!(g2t.gate_c1());
        assert!(g2t.gate_c2());
        assert!(g2t.gate_c3());
        assert!(g2t.gate_c4());
        assert!(g2t.full_selector_pass());
    }

    #[test]
    fn test_dihedral_fails_c3() {
        let d8 = build_dihedral(4);
        assert!(d8.gate_c1());
        assert!(!d8.gate_c3());
        assert!(!d8.full_selector_pass());
    }

    #[test]
    fn test_cyclic_fails_c1() {
        let z12 = build_cyclic(12);
        assert!(!z12.gate_c1());
        assert!(!z12.full_selector_pass());
    }
}
