//! The BLOCK-SPARSE two-site operator for the labelled sweep — E14 item 1.
//!
//! The dense operator (`mps::apply_effective_h_mpo_live`) computes every entry of the
//! two-site tensor and the sweep zeroes the label-inconsistent ones AFTERWARDS. Measured
//! at the middle bond of N = 8 (`examples/sym_sparsity.rs`, 2026-09-03): the wavefunction
//! is 0.8–2.6 % live, every environment channel is 0.7–2.5 % live, and every one of the 70
//! live channels carries exactly ONE charge shift — `L[c][l_out, l_in] ≠ 0` only where
//! `q(l_out) = q(l_in) + Δ_c`. So the two heavy stages (the right-environment and the
//! left-environment contractions, each `~4·|live channels|·χ³`) spend 39×–125× their
//! useful flops on structural zeros, and the waste grows with volume.
//!
//! This operator does the same arithmetic on the live blocks only. BIT-IDENTICAL to the
//! dense operator by construction and by gate: every product it omits is an exact IEEE
//! zero (one factor structurally zero), every product it keeps is accumulated in the same
//! ascending index order the dense loops use, and `x + 0.0 == x` exactly. The plan is built
//! once per local eigensolve from the labels and the environments; the channel shifts are
//! READ OFF the environments and verified on every nonzero entry, so an MPO that does not
//! conserve these labels is refused by name rather than contracted wrongly.
//!
//! Stages 2 and 3 (the two MPO sites) are shared with the dense path unchanged: their cost
//! runs over channels, not bonds, and is a fraction of a percent of the whole.

use crate::mpo::MpoSite;
use crate::mps::{self, Env};
use crate::symmetric::{charge_add, charge_sub, Charge};
use std::collections::HashMap;

const NONE: usize = usize::MAX;

/// Why a plan could not be built: the environment is not block-diagonal in the labels it
/// was handed, which means the operator does not conserve them.
#[derive(Debug, Clone, PartialEq)]
pub struct NotBlockDiagonal {
    pub side: &'static str,
    pub channel: usize,
    pub shift_seen: Charge,
    pub shift_expected: Charge,
}

impl std::fmt::Display for NotBlockDiagonal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} environment channel {} carries two charge shifts ({:?} and {:?}): the operator does not \
             conserve these labels, and the block-sparse contraction will not pretend it does",
            self.side, self.channel, self.shift_expected, self.shift_seen
        )
    }
}

/// The label structure of one two-site update, precomputed.
#[derive(Debug, Clone)]
pub struct BlockPlan {
    chi_l: usize,
    chi_r: usize,
    /// Left-bond indices grouped by label, each group ascending.
    l_blocks: Vec<Vec<usize>>,
    /// Right-bond indices grouped by label, each group ascending.
    r_blocks: Vec<Vec<usize>>,
    /// Left index → its block.
    l_block_of: Vec<usize>,
    /// For each left block and each `(a, b)`: the right block carrying `q_l + a·e₁ + b·e₂`, or NONE.
    cut_r_block: Vec<[usize; 4]>,
    /// Per live LEFT channel: `(channel, for each l_out block → the l_in block at label − Δ, or NONE)`.
    left: Vec<(usize, Vec<usize>)>,
    /// Per live RIGHT channel: `(channel, for each r_in block → the r_out block at label + Δ, or NONE)`.
    right: Vec<(usize, Vec<usize>)>,
}

fn group(q: &[Charge]) -> (Vec<Vec<usize>>, Vec<usize>, HashMap<Charge, usize>) {
    let mut id: HashMap<Charge, usize> = HashMap::new();
    let mut blocks: Vec<Vec<usize>> = Vec::new();
    let mut of = Vec::with_capacity(q.len());
    for (i, &c) in q.iter().enumerate() {
        let b = *id.entry(c).or_insert_with(|| {
            blocks.push(Vec::new());
            blocks.len() - 1
        });
        blocks[b].push(i); // i ascending, so every group is ascending
        of.push(b);
    }
    (blocks, of, id)
}

/// One channel's shift, read off its environment matrix and checked on every nonzero entry.
fn channel_shift(side: &'static str, c: usize, m: &[f64], q: &[Charge]) -> Result<Option<Charge>, NotBlockDiagonal> {
    let chi = q.len();
    let mut shift: Option<Charge> = None;
    for i in 0..chi {
        for j in 0..chi {
            if m[i * chi + j] == 0.0 {
                continue;
            }
            let d = charge_sub(q[i], q[j]);
            match shift {
                None => shift = Some(d),
                Some(s) if s != d => {
                    return Err(NotBlockDiagonal { side, channel: c, shift_seen: d, shift_expected: s })
                }
                _ => {}
            }
        }
    }
    Ok(shift)
}

impl BlockPlan {
    /// Build the plan for the update at a bond with labels `q_l` (left bond) and `q_r` (right
    /// bond), site charges `e1`, `e2`, and the two environments the eigensolve will use.
    pub fn build(q_l: &[Charge], q_r: &[Charge], e1: Charge, e2: Charge, left: &Env, right: &Env) -> Result<BlockPlan, NotBlockDiagonal> {
        let (l_blocks, l_block_of, _) = group(q_l);
        let (r_blocks, _, r_id) = group(q_r);
        let mut cut_r_block = Vec::with_capacity(l_blocks.len());
        for lb in &l_blocks {
            let ql = q_l[lb[0]];
            let mut row = [NONE; 4];
            for a in 0..2 {
                let qa = if a == 1 { charge_add(ql, e1) } else { ql };
                for b in 0..2 {
                    let qab = if b == 1 { charge_add(qa, e2) } else { qa };
                    row[a * 2 + b] = r_id.get(&qab).copied().unwrap_or(NONE);
                }
            }
            cut_r_block.push(row);
        }
        let mut left_plan = Vec::new();
        for (c, m) in left.iter().enumerate() {
            let Some(shift) = channel_shift("left", c, m, q_l)? else { continue };
            let l_id: HashMap<Charge, usize> = l_blocks.iter().enumerate().map(|(i, b)| (q_l[b[0]], i)).collect();
            let map: Vec<usize> = l_blocks
                .iter()
                .map(|b| l_id.get(&charge_sub(q_l[b[0]], shift)).copied().unwrap_or(NONE))
                .collect();
            left_plan.push((c, map));
        }
        let mut right_plan = Vec::new();
        for (c, m) in right.iter().enumerate() {
            let Some(shift) = channel_shift("right", c, m, q_r)? else { continue };
            let map: Vec<usize> = r_blocks
                .iter()
                .map(|b| r_id.get(&charge_add(q_r[b[0]], shift)).copied().unwrap_or(NONE))
                .collect();
            right_plan.push((c, map));
        }
        Ok(BlockPlan { chi_l: q_l.len(), chi_r: q_r.len(), l_blocks, r_blocks, l_block_of, cut_r_block, left: left_plan, right: right_plan })
    }

    /// The live channels this plan found, in the order the dense operator visits them.
    pub fn live_left(&self) -> Vec<usize> {
        self.left.iter().map(|p| p.0).collect()
    }
    pub fn live_right(&self) -> Vec<usize> {
        self.right.iter().map(|p| p.0).collect()
    }

    /// Structurally live entries of the two-site wavefunction under this plan.
    pub fn live_entries(&self) -> usize {
        let mut n = 0;
        for (lb, row) in self.cut_r_block.iter().enumerate() {
            let nl = self.l_blocks[lb].len();
            for rb in row {
                if *rb != NONE {
                    n += nl * self.r_blocks[*rb].len();
                }
            }
        }
        n
    }

    /// Stage 1, block-sparse: the right environment applied on the live blocks only.
    fn step1(&self, right: &Env, psi: &[f64], d_r: usize) -> Vec<f64> {
        let (chi_l, chi_r) = (self.chi_l, self.chi_r);
        let block = chi_l * 4 * chi_r;
        let mut t1 = vec![0.0; d_r * block];
        let nthreads = mps::threads();
        let step = |c2: usize, map: &[usize], t1c: &mut [f64]| {
            let rmat = &right[c2];
            for (lb, lidx) in self.l_blocks.iter().enumerate() {
                for &l_in in lidx {
                    for ab in 0..4 {
                        let rb_in = self.cut_r_block[lb][ab];
                        if rb_in == NONE {
                            continue;
                        }
                        let rb_out = map[rb_in];
                        if rb_out == NONE {
                            continue;
                        }
                        let base = (l_in * 4 + ab) * chi_r;
                        let r_ins = &self.r_blocks[rb_in];
                        for &r_out in &self.r_blocks[rb_out] {
                            let rrow = r_out * chi_r;
                            let mut acc = 0.0;
                            for &r_in in r_ins {
                                acc += rmat[rrow + r_in] * psi[base + r_in];
                            }
                            t1c[base + r_out] = acc;
                        }
                    }
                }
            }
        };
        let mut jobs: Vec<(usize, &[usize], &mut [f64])> = Vec::new();
        let mut chunks = t1.chunks_mut(block).enumerate();
        let mut it = self.right.iter().peekable();
        while let Some((c2, t1c)) = chunks.next() {
            if let Some((c, map)) = it.peek() {
                if *c == c2 {
                    jobs.push((c2, map.as_slice(), t1c));
                    it.next();
                }
            }
        }
        if nthreads <= 1 || jobs.len() < 2 {
            for (c2, map, t1c) in jobs.iter_mut() {
                step(*c2, map, t1c);
            }
        } else {
            let per = jobs.len().div_ceil(nthreads).max(1);
            std::thread::scope(|sc| {
                for chunk in jobs.chunks_mut(per) {
                    let step = &step;
                    sc.spawn(move || {
                        for (c2, map, t1c) in chunk.iter_mut() {
                            step(*c2, map, t1c);
                        }
                    });
                }
            });
        }
        t1
    }

    /// Stage 4, block-sparse: the left environment applied on the live blocks only, threaded
    /// over disjoint `l_out` rows with the channel and `l_in` order kept serial inside a row.
    fn step4(&self, left: &Env, t3: &[f64]) -> Vec<f64> {
        let (chi_l, chi_r) = (self.chi_l, self.chi_r);
        let row = 4 * chi_r;
        let mut out = vec![0.0; chi_l * row];
        let nthreads = mps::threads();
        let step = |l_out: usize, outrow: &mut [f64]| {
            let lb_out = self.l_block_of[l_out];
            for (c1, map) in &self.left {
                let lb_in = map[lb_out];
                if lb_in == NONE {
                    continue;
                }
                let lmat = &left[*c1];
                let lrow = l_out * chi_l;
                let l_ins = &self.l_blocks[lb_in];
                for st in 0..4 {
                    let rb = self.cut_r_block[lb_out][st];
                    if rb == NONE {
                        continue;
                    }
                    let out_base = st * chi_r;
                    for &r_out in &self.r_blocks[rb] {
                        let mut acc = 0.0;
                        for &l_in in l_ins {
                            let src = ((c1 * chi_l + l_in) * 4 + st) * chi_r;
                            acc += lmat[lrow + l_in] * t3[src + r_out];
                        }
                        outrow[out_base + r_out] += acc;
                    }
                }
            }
        };
        if nthreads <= 1 || chi_l < 2 {
            for (l_out, outrow) in out.chunks_mut(row).enumerate() {
                step(l_out, outrow);
            }
        } else {
            let per = chi_l.div_ceil(nthreads).max(1);
            std::thread::scope(|sc| {
                for (ci, chunk) in out.chunks_mut(per * row).enumerate() {
                    let step = &step;
                    sc.spawn(move || {
                        for (i, outrow) in chunk.chunks_mut(row).enumerate() {
                            step(ci * per + i, outrow);
                        }
                    });
                }
            });
        }
        out
    }

    /// The two-site effective Hamiltonian on `psi`, live blocks only. Bit-identical to
    /// `mps::apply_effective_h_mpo_live` on any label-consistent `psi`.
    pub fn apply(&self, left: &Env, w1: &MpoSite, w2: &MpoSite, right: &Env, psi: &[f64]) -> Vec<f64> {
        debug_assert_eq!(psi.len(), self.chi_l * 4 * self.chi_r);
        let t1 = self.step1(right, psi, w2.d_r);
        let t3 = mps::apply_mpo_pair(w1, w2, &t1, self.chi_l, self.chi_r);
        self.step4(left, &t3)
    }
}
