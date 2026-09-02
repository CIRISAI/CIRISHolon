//! The MAGIC tier: Clifford+T by stabilizer branch-sum, priced by T-count.
//!
//! T = ((1+ω)/2)·I + ((1−ω)/2)·Z with ω = e^{iπ/4}, so a circuit with t
//! T-gates is a sum of 2^t phase-tracked stabilizer branches — cost
//! 2^t · poly(n), priced by MAGIC, not by qubits. The branch states are kept
//! in the affine form (Dehaene–De Moor 2003; Van den Nest 2010, credited):
//! amplitude(x) = γ · i^{Σ d_a u_a} · (−1)^{Σ_{a<b} J_{ab} u_a u_b} on the
//! affine subspace x = R u ⊕ h, with γ EXACT in Z[ω]·2^{−m/2} — no floating
//! point until the final conversion, so "the probabilities sum to exactly 1"
//! is an arithmetic invariant, not a tolerance.
//!
//! The naive branch-sum is exactly 2^t; the Bravyi–Gosset stabilizer-rank
//! reduction (2^{~0.48t}) is the named next improvement, not claimed.

use crate::{Circuit, Gate};
use std::collections::BTreeMap;

// ---------------------------------------------------------------- exact scalars

/// (c0 + c1 ω + c2 ω² + c3 ω³) · 2^{−m/2},  ω = e^{iπ/4},  ω⁴ = −1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cyc {
    pub c: [i128; 4],
    pub m: i32,
}

impl Cyc {
    pub const ONE: Cyc = Cyc { c: [1, 0, 0, 0], m: 0 };
    pub const ZERO: Cyc = Cyc { c: [0, 0, 0, 0], m: 0 };

    pub fn i_pow(k: u8) -> Cyc {
        // i = ω²
        let mut c = [0i128; 4];
        match k % 4 {
            0 => c[0] = 1,
            1 => c[2] = 1,
            2 => c[0] = -1,
            _ => c[2] = -1,
        }
        Cyc { c, m: 0 }
    }

    fn normalize(mut self) -> Cyc {
        while self.m >= 2 && self.c.iter().all(|&x| x % 2 == 0) &&
              self.c.iter().any(|&x| x != 0) {
            for x in &mut self.c {
                *x /= 2;
            }
            self.m -= 2;
        }
        if self.c.iter().all(|&x| x == 0) {
            self.m = 0;
        }
        self
    }

    /// The exactness envelope, enforced: refuse (panic) on i128 overflow
    /// rather than wrap — a wrapped coefficient is a silently wrong exact
    /// value, the one output the referee may never produce. Coefficients
    /// grow like 2^{O(n+t)} (Quist–Coopmans–Laarman, arXiv:2602.17775),
    /// so the envelope is reachable and the refusal is load-bearing.
    #[inline]
    fn refuse() -> ! {
        panic!(
            "Cyc: coefficient overflow — the i128 exactness envelope is exceeded; \
             refusing rather than wrapping"
        )
    }

    #[inline]
    fn cadd(a: i128, b: i128) -> i128 {
        match a.checked_add(b) {
            Some(v) => v,
            None => Self::refuse(),
        }
    }

    #[inline]
    fn cmul(a: i128, b: i128) -> i128 {
        match a.checked_mul(b) {
            Some(v) => v,
            None => Self::refuse(),
        }
    }

    #[inline]
    fn csub(a: i128, b: i128) -> i128 {
        match a.checked_sub(b) {
            Some(v) => v,
            None => Self::refuse(),
        }
    }

    pub fn mul(self, o: Cyc) -> Cyc {
        let mut c = [0i128; 8];
        for a in 0..4 {
            for b in 0..4 {
                c[a + b] = Self::cadd(c[a + b], Self::cmul(self.c[a], o.c[b]));
            }
        }
        let mut out = [0i128; 4];
        for a in 0..4 {
            out[a] = Self::csub(c[a], c[a + 4]); // ω⁴ = −1
        }
        Cyc { c: out, m: self.m + o.m }.normalize()
    }

    /// Multiply by √2 = ω − ω³ (exact in the ring).
    pub fn mul_sqrt2(self) -> Cyc {
        self.mul(Cyc { c: [0, 1, 0, -1], m: 0 })
    }

    pub fn add_fixed(self, o: Cyc) -> Cyc {
        if self.c.iter().all(|&x| x == 0) {
            return o;
        }
        if o.c.iter().all(|&x| x == 0) {
            return self;
        }
        // Align denominators in ONE shot: scale the smaller-m side's
        // coefficients by (√2)^Δ (2^{Δ/2} for even Δ, plus one ring-multiply
        // by √2 = ω − ω³ for odd Δ). A loop of mul_sqrt2 calls would fight
        // normalize() and cycle (measured: hang at Δ ≥ 3, 2026-08-27).
        let (mut a, mut b) = (self, o);
        if a.m > b.m {
            std::mem::swap(&mut a, &mut b);
        }
        let delta = (b.m - a.m) as u32;
        let half = delta / 2;
        if half >= 127 {
            Self::refuse();
        }
        let pow = 1i128 << half;
        for x in &mut a.c {
            *x = Self::cmul(*x, pow);
        }
        if delta % 2 == 1 {
            // one ring-multiply by √2 = ω − ω³
            let t = a.c;
            let mut acc = [0i128; 8];
            for p in 0..4 {
                acc[p + 1] = Self::cadd(acc[p + 1], t[p]);
                acc[p + 3] = Self::csub(acc[p + 3], t[p]);
            }
            for p in 0..4 {
                a.c[p] = Self::csub(acc[p], acc[p + 4]);
            }
        }
        a.m = b.m;
        Cyc {
            c: [
                Self::cadd(a.c[0], b.c[0]),
                Self::cadd(a.c[1], b.c[1]),
                Self::cadd(a.c[2], b.c[2]),
                Self::cadd(a.c[3], b.c[3]),
            ],
            m: a.m,
        }
        .normalize()
    }

    pub fn to_complex(self) -> (f64, f64) {
        let s = std::f64::consts::FRAC_1_SQRT_2;
        // ω^k as complex
        let w = [(1.0, 0.0), (s, s), (0.0, 1.0), (-s, s)];
        let mut re = 0.0;
        let mut im = 0.0;
        for k in 0..4 {
            re += self.c[k] as f64 * w[k].0;
            im += self.c[k] as f64 * w[k].1;
        }
        let scale = (2.0f64).powf(-(self.m as f64) / 2.0);
        (re * scale, im * scale)
    }
}

// ---------------------------------------------------------------- affine state

#[derive(Clone)]
pub struct Affine {
    n: usize,
    /// R: n rows × k columns, x = R u ⊕ h.
    r: Vec<Vec<bool>>,
    h: Vec<bool>,
    /// d_a mod 4 (i-power linear part), one per column.
    d: Vec<u8>,
    /// J_{ab} (symmetric, diagonal unused): (−1)^{J u_a u_b}.
    j: Vec<Vec<bool>>,
    gamma: Cyc,
    zero: bool,
    /// Planted mutations for the gauge.
    mut_drop_s_cross: bool,
    mut_wrong_gauss: bool,
}

impl Affine {
    pub fn new(n: usize) -> Self {
        Affine {
            n,
            r: vec![Vec::new(); n],
            h: vec![false; n],
            d: Vec::new(),
            j: Vec::new(),
            gamma: Cyc::ONE,
            zero: false,
            mut_drop_s_cross: false,
            mut_wrong_gauss: false,
        }
    }

    pub fn with_mutation(n: usize, drop_s_cross: bool, wrong_gauss: bool) -> Self {
        let mut a = Self::new(n);
        a.mut_drop_s_cross = drop_s_cross;
        a.mut_wrong_gauss = wrong_gauss;
        a
    }

    fn k(&self) -> usize {
        self.d.len()
    }

    /// u_a := u_a ⊕ u_b  (fold a's dependence into b): R_{·,b} ^= R_{·,a},
    /// d_b += d_a, J_{ab} ^= d_a&1, J_{bc} ^= J_{ac}, d_b += 2·J_{ab_old}.
    fn fold(&mut self, a: usize, b: usize) {
        assert_ne!(a, b);
        for row in 0..self.n {
            let ra = self.r[row][a];
            self.r[row][b] ^= ra;
        }
        let da = self.d[a];
        let jab_old = self.j[a][b];
        let ja_row: Vec<bool> = self.j[a].clone();
        self.d[b] = (self.d[b] + da) % 4;
        self.j[a][b] ^= da & 1 == 1;
        self.j[b][a] = self.j[a][b];
        for c in 0..self.k() {
            if c != a && c != b && ja_row[c] {
                self.j[b][c] = !self.j[b][c];
                self.j[c][b] = self.j[b][c];
            }
        }
        self.d[b] = (self.d[b] + if jab_old { 2 } else { 0 }) % 4;
    }

    /// Remove column a with u_a pinned to val.
    fn pin_remove(&mut self, a: usize, val: bool) {
        if val {
            for row in 0..self.n {
                if self.r[row][a] {
                    self.h[row] = !self.h[row];
                }
            }
            self.gamma = self.gamma.mul(Cyc::i_pow(self.d[a]));
            for c in 0..self.k() {
                if c != a && self.j[a][c] {
                    self.d[c] = (self.d[c] + 2) % 4;
                }
            }
        }
        self.remove_col(a);
    }

    fn remove_col(&mut self, a: usize) {
        for row in 0..self.n {
            self.r[row].remove(a);
        }
        self.d.remove(a);
        self.j.remove(a);
        for jr in &mut self.j {
            jr.remove(a);
        }
    }

    /// Sum out phase-only column a (its R column is all-zero):
    /// Σ_w i^{δw} (−1)^{w·Λ},  Λ = Σ_{b∈L} u_b,  L = J-neighbours of a.
    fn gauss_sum_out(&mut self, a: usize) {
        let delta = self.d[a];
        let l: Vec<usize> = (0..self.k()).filter(|&b| b != a && self.j[a][b]).collect();
        match delta % 4 {
            0 | 2 => {
                let eps = delta == 2; // constraint Λ ≡ eps
                if l.is_empty() {
                    if eps {
                        self.zero = true;
                        self.remove_col(a);
                        return;
                    }
                    self.gamma.m -= 2; // ×2
                    self.remove_col(a);
                } else {
                    // impose Σ_L u = eps: fold onto c = l[0], pin it.
                    let c = l[0];
                    for &b in &l[1..] {
                        self.fold(c, b);
                    }
                    self.gamma.m -= 2;
                    self.remove_col(a);
                    let c_adj = if c > a { c - 1 } else { c };
                    self.pin_remove(c_adj, eps);
                }
            }
            _ => {
                // δ odd: ×(1+i^δ), and each b∈L gets d_b += δ+2.
                let phase = if self.mut_wrong_gauss {
                    Cyc::i_pow(delta) // PLANTED WRONG: drops the 1+ structure
                } else {
                    Cyc::ONE.add_fixed(Cyc::i_pow(delta))
                };
                self.gamma = self.gamma.mul(phase);
                // (−i^δ)^{Λ mod 2} with Λ = Σ_L u: the XOR expansion — the
                // identity (1+i^δ(−1)^Λ) = (1+i^δ)(−i^δ)^Λ holds only for
                // Λ ∈ {0,1}, and Λ here is an integer sum, so the factorized
                // form needs i^{(δ+2)(⊕_L u)}: per-variable phases PLUS
                // pairwise (−1)^{u_a u_b} flips across L (measured: sign error
                // on amp(1,1) of h,cx,h,s,h — the entangled HSH sandwich).
                for &b in &l {
                    self.d[b] = (self.d[b] + delta + 2) % 4;
                }
                for i1 in 0..l.len() {
                    for i2 in i1 + 1..l.len() {
                        let (b1, b2) = (l[i1], l[i2]);
                        self.j[b1][b2] = !self.j[b1][b2];
                        self.j[b2][b1] = self.j[b1][b2];
                    }
                }
                self.remove_col(a);
            }
        }
    }

    pub fn x(&mut self, q: usize) {
        self.h[q] = !self.h[q];
    }

    pub fn z(&mut self, q: usize) {
        if self.h[q] {
            self.gamma = self.gamma.mul(Cyc::i_pow(2));
        }
        for a in 0..self.k() {
            if self.r[q][a] {
                self.d[a] = (self.d[a] + 2) % 4;
            }
        }
    }

    pub fn s(&mut self, q: usize) {
        // i^{x_q}: γ·i^{h}, d_a += 1+2h for a∈A, J_{ab} ^= 1 for a<b∈A.
        let a_set: Vec<usize> = (0..self.k()).filter(|&a| self.r[q][a]).collect();
        if self.h[q] {
            self.gamma = self.gamma.mul(Cyc::i_pow(1));
        }
        let bump = if self.h[q] { 3 } else { 1 };
        for &a in &a_set {
            self.d[a] = (self.d[a] + bump) % 4;
        }
        if !self.mut_drop_s_cross {
            for i in 0..a_set.len() {
                for jj in i + 1..a_set.len() {
                    let (a, b) = (a_set[i], a_set[jj]);
                    self.j[a][b] = !self.j[a][b];
                    self.j[b][a] = self.j[a][b];
                }
            }
        }
    }

    pub fn cx(&mut self, c: usize, t: usize) {
        for a in 0..self.k() {
            let rc = self.r[c][a];
            self.r[t][a] ^= rc;
        }
        let hc = self.h[c];
        self.h[t] ^= hc;
    }

    pub fn h_gate(&mut self, q: usize) {
        // Reduce row q to at most one supporting column a*.
        let support: Vec<usize> = (0..self.k()).filter(|&a| self.r[q][a]).collect();
        let a_star = if support.is_empty() {
            None
        } else {
            let a = support[0];
            for &b in &support[1..] {
                self.fold(a, b);
            }
            Some(a)
        };
        // New variable v; phase (−1)^{(u_{a*} ⊕ h_q)·v}.
        let v = self.k();
        for row in 0..self.n {
            self.r[row].push(false);
        }
        self.d.push(if self.h[q] { 2 } else { 0 });
        for jr in &mut self.j {
            jr.push(false);
        }
        self.j.push(vec![false; v + 1]);
        if let Some(a) = a_star {
            self.j[a][v] = true;
            self.j[v][a] = true;
        }
        for a in 0..self.k() {
            self.r[q][a] = false;
        }
        self.r[q][v] = true;
        self.h[q] = false;
        self.gamma.m += 1;
        // Row-clearing can break column independence: col a* may now equal an
        // XOR of other columns (two columns that differed only at row q
        // collide once the row is cleared — measured: h,h,h,h,cx,h err 0.375).
        // If so, fold that subset into a* until it is all-zero, then Gauss-sum
        // it out; the amplitude query REQUIRES independent columns.
        if let Some(a) = a_star {
            if !(0..self.n).all(|row| !self.r[row][a]) {
                if let Some(subset) = self.dependent_subset(a) {
                    for b in subset {
                        self.fold(b, a);
                    }
                }
            }
            if (0..self.n).all(|row| !self.r[row][a]) {
                self.gauss_sum_out(a);
            }
        }
    }

    /// If column a is an XOR of other columns, return that subset.
    fn dependent_subset(&self, a: usize) -> Option<Vec<usize>> {
        let k = self.k();
        let others: Vec<usize> = (0..k).filter(|&b| b != a).collect();
        // Solve [cols others] x = col a over F2.
        let mut rows: Vec<(Vec<bool>, bool)> = (0..self.n)
            .map(|r| (others.iter().map(|&b| self.r[r][b]).collect(), self.r[r][a]))
            .collect();
        let m = others.len();
        let mut piv = vec![usize::MAX; m];
        let mut rr = 0;
        for col in 0..m {
            if let Some(p) = (rr..self.n).find(|&p| rows[p].0[col]) {
                rows.swap(rr, p);
                for p2 in 0..self.n {
                    if p2 != rr && rows[p2].0[col] {
                        let src = rows[rr].clone();
                        rows[p2].0.iter_mut().zip(&src.0).for_each(|(x, y)| *x ^= *y);
                        rows[p2].1 ^= src.1;
                    }
                }
                piv[col] = rr;
                rr += 1;
            }
        }
        if rows[rr..].iter().any(|r| r.1) {
            return None; // independent
        }
        let mut subset = Vec::new();
        for col in 0..m {
            if piv[col] != usize::MAX && rows[piv[col]].1 {
                subset.push(others[col]);
            }
        }
        Some(subset)
    }

    pub fn apply(&mut self, g: Gate) {
        if self.zero {
            return;
        }
        match g {
            Gate::X(q) => self.x(q),
            Gate::Z(q) => self.z(q),
            Gate::S(q) => self.s(q),
            Gate::Sdg(q) => {
                self.s(q);
                self.s(q);
                self.s(q);
            }
            Gate::H(q) => self.h_gate(q),
            Gate::Cx(c, t) => self.cx(c, t),
            _ => panic!("magic tier branches must be Clifford"),
        }
    }

    /// Exact amplitude of basis state y (bit i = qubit i).
    pub fn amplitude(&self, y: &[bool]) -> Cyc {
        if self.zero {
            return Cyc::ZERO;
        }
        // Solve R u = y ⊕ h.
        let k = self.k();
        let mut aug: Vec<(Vec<bool>, bool)> = (0..self.n)
            .map(|row| (self.r[row].clone(), y[row] ^ self.h[row]))
            .collect();
        let mut u = vec![false; k];
        let mut pivot_row = vec![usize::MAX; k];
        let mut rr = 0;
        for col in 0..k {
            if let Some(p) = (rr..self.n).find(|&p| aug[p].0[col]) {
                aug.swap(rr, p);
                for p2 in 0..self.n {
                    if p2 != rr && aug[p2].0[col] {
                        let (head, tail) = if p2 < rr {
                            let (a, b) = aug.split_at_mut(rr);
                            (&mut a[p2], &mut b[0])
                        } else {
                            let (a, b) = aug.split_at_mut(p2);
                            (&mut b[0], &mut a[rr])
                        };
                        for cc in 0..k {
                            head.0[cc] ^= tail.0[cc];
                        }
                        head.1 ^= tail.1;
                    }
                }
                pivot_row[col] = rr;
                rr += 1;
            }
        }
        for row in rr..self.n {
            if aug[row].1 {
                return Cyc::ZERO; // y not in the affine subspace
            }
        }
        assert!(
            (0..k).all(|col| pivot_row[col] != usize::MAX),
            "affine invariant broken: R has dependent columns (rank < k)"
        );
        for col in 0..k {
            u[col] = aug[pivot_row[col]].1;
        }
        let mut ip: u8 = 0;
        let mut sign = false;
        for a in 0..k {
            if u[a] {
                ip = (ip + self.d[a]) % 4;
                for b in a + 1..k {
                    if u[b] && self.j[a][b] {
                        sign = !sign;
                    }
                }
            }
        }
        let mut amp = self.gamma.mul(Cyc::i_pow(ip));
        if sign {
            amp = amp.mul(Cyc::i_pow(2));
        }
        amp
    }
}

// ---------------------------------------------------------------- branch sum

/// `run_magic`, refusing by name instead of panicking. The distribution path
/// pays BOTH budgets — 2^t branches into a 2^n accumulator — so both are named.
pub fn try_run_magic(
    c: &Circuit,
    drop_s_cross: bool,
    wrong_gauss: bool,
) -> Result<BTreeMap<String, f64>, String> {
    crate::check_magic_distribution_budget(c)?;
    Ok(run_magic(c, drop_s_cross, wrong_gauss))
}

pub fn run_magic(
    c: &Circuit,
    drop_s_cross: bool,
    wrong_gauss: bool,
) -> BTreeMap<String, f64> {
    if let Err(reason) = crate::check_magic_distribution_budget(c) {
        panic!("{reason}");
    }
    let t_count = crate::t_count(c);
    let n_branches = 1usize << t_count;
    // Amplitude accumulators per basis state (exact).
    let dim = 1usize << c.n_qubits;
    let mut acc = vec![Cyc::ZERO; dim];
    for branch in 0..n_branches {
        let mut st = Affine::with_mutation(c.n_qubits, drop_s_cross, wrong_gauss);
        let mut coeff = Cyc::ONE;
        let mut ti = 0;
        for g in &c.gates {
            match *g {
                Gate::T(q) | Gate::Tdg(q) => {
                    let dag = matches!(g, Gate::Tdg(_));
                    let z_branch = branch >> ti & 1 == 1;
                    ti += 1;
                    // T = (1+ω)/2 · I + (1−ω)/2 · Z ; Tdg with ω → ω⁻¹ = −ω³.
                    let (ci, cz) = if !dag {
                        (Cyc { c: [1, 1, 0, 0], m: 2 }, Cyc { c: [1, -1, 0, 0], m: 2 })
                    } else {
                        (Cyc { c: [1, 0, 0, -1], m: 2 }, Cyc { c: [1, 0, 0, 1], m: 2 })
                    };
                    if z_branch {
                        coeff = coeff.mul(cz);
                        st.apply(Gate::Z(q));
                    } else {
                        coeff = coeff.mul(ci);
                    }
                }
                g => st.apply(g),
            }
        }
        if st.zero {
            continue;
        }
        let mut y = vec![false; c.n_qubits];
        for idx in 0..dim {
            for q in 0..c.n_qubits {
                y[q] = idx >> q & 1 == 1;
            }
            let a = st.amplitude(&y);
            if a != Cyc::ZERO {
                acc[idx] = acc[idx].add_fixed(coeff.mul(a));
            }
        }
    }
    let mut out = BTreeMap::new();
    for idx in 0..dim {
        let (re, im) = acc[idx].to_complex();
        let p = re * re + im * im;
        if p < 1e-14 {
            continue;
        }
        let mut bits = vec![false; c.n_clbits];
        for &(q, cl) in &c.measures {
            bits[cl] = idx >> q & 1 == 1;
        }
        let key: String = (0..c.n_clbits)
            .rev()
            .map(|i| if bits[i] { '1' } else { '0' })
            .collect();
        *out.entry(key).or_insert(0.0) += p;
    }
    out
}

/// `magic_amplitude`, refusing by name instead of panicking. This path has no
/// 2^n, so the T-count is the whole of its price and the whole of its wall.
pub fn try_magic_amplitude(
    c: &Circuit,
    y: &[bool],
    drop_s_cross: bool,
    wrong_gauss: bool,
) -> Result<(f64, f64), String> {
    crate::check_t_budget(c)?;
    Ok(magic_amplitude(c, y, drop_s_cross, wrong_gauss))
}

/// Exact amplitude of one basis state via the branch sum: 2^t · poly(n) work,
/// no 2^n anywhere — the object the T-count prices.
pub fn magic_amplitude(
    c: &Circuit,
    y: &[bool],
    drop_s_cross: bool,
    wrong_gauss: bool,
) -> (f64, f64) {
    if let Err(reason) = crate::check_t_budget(c) {
        panic!("{reason}");
    }
    let t_count = crate::t_count(c);
    let mut acc = Cyc::ZERO;
    for branch in 0..(1usize << t_count) {
        let mut st = Affine::with_mutation(c.n_qubits, drop_s_cross, wrong_gauss);
        let mut coeff = Cyc::ONE;
        let mut ti = 0;
        for g in &c.gates {
            match *g {
                Gate::T(q) | Gate::Tdg(q) => {
                    let dag = matches!(g, Gate::Tdg(_));
                    let z_branch = branch >> ti & 1 == 1;
                    ti += 1;
                    let (ci, cz) = if !dag {
                        (Cyc { c: [1, 1, 0, 0], m: 2 }, Cyc { c: [1, -1, 0, 0], m: 2 })
                    } else {
                        (Cyc { c: [1, 0, 0, -1], m: 2 }, Cyc { c: [1, 0, 0, 1], m: 2 })
                    };
                    if z_branch {
                        coeff = coeff.mul(cz);
                        st.apply(Gate::Z(q));
                    } else {
                        coeff = coeff.mul(ci);
                    }
                }
                g => st.apply(g),
            }
        }
        if st.zero {
            continue;
        }
        let a = st.amplitude(y);
        if a != Cyc::ZERO {
            acc = acc.add_fixed(coeff.mul(a));
        }
    }
    acc.to_complex()
}
