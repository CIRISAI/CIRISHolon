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
#[derive(Debug)]
pub struct BlockPlan {
    chi_l: usize,
    chi_r: usize,
    /// Left-bond indices grouped by label, each group ascending.
    l_blocks: Vec<Vec<usize>>,
    /// Right-bond indices grouped by label, each group ascending.
    r_blocks: Vec<Vec<usize>>,
    /// For each left block and each `(a, b)`: the right block carrying `q_l + a·e₁ + b·e₂`, or NONE.
    cut_r_block: Vec<[usize; 4]>,
    /// Per live LEFT channel: `(channel, for each l_out block → the l_in block at label − Δ, or NONE)`.
    left: Vec<(usize, Vec<usize>)>,
    /// Per live RIGHT channel: `(channel, for each r_in block → the r_out block at label + Δ, or NONE)`.
    right: Vec<(usize, Vec<usize>)>,
    /// E14 item 5a, LOCALITY: the environments' live sub-matrices copied out ONCE per plan as
    /// contiguous tiles, so the hundreds of matvecs of a local eigensolve read dense rows
    /// instead of gathering through index lists. `left_tiles[i][lb_out]` is the tile of live
    /// left channel `i` with rows `l_blocks[lb_out]` and columns `l_blocks[map[lb_out]]`
    /// (row-major, `|out| × |in|`); `right_tiles[i][rb_in]` has rows `r_blocks[map[rb_in]]`
    /// and columns `r_blocks[rb_in]`. Empty where the map is NONE. Memory: the live entries
    /// of the environments, a few percent of `d·χ²`. The layout the device kernel consumes.
    left_tiles: Vec<Vec<Vec<f64>>>,
    right_tiles: Vec<Vec<Vec<f64>>>,
    /// The three intermediates `t1`, `t2`, `t3`, allocated ONCE per plan and never cleared:
    /// every stage writes a slot's first contribution by assignment and later ones by
    /// accumulation (`0.0 + x == x` exactly), and every stage reads only the slots the one
    /// before it wrote. Measured before this existed: zero-filling three dense `d·4χ²`
    /// intermediates per matvec — 135 MB at χ = 256 — was 22.7 of a 32.0 ms matvec.
    scratch: std::sync::Mutex<Scratch>,
}

#[derive(Debug, Clone, Default)]
struct Scratch {
    t1: Vec<f64>,
    t2: Vec<f64>,
    t3: Vec<f64>,
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
        let (l_blocks, _, _) = group(q_l);
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
        let chi_l = q_l.len();
        let chi_r = q_r.len();
        let left_tiles = left_plan
            .iter()
            .map(|(c, map)| {
                let m = &left[*c];
                l_blocks
                    .iter()
                    .enumerate()
                    .map(|(lb_out, rows)| {
                        let lb_in = map[lb_out];
                        if lb_in == NONE {
                            return Vec::new();
                        }
                        let cols = &l_blocks[lb_in];
                        let mut t = Vec::with_capacity(rows.len() * cols.len());
                        for &i in rows {
                            for &j in cols {
                                t.push(m[i * chi_l + j]);
                            }
                        }
                        t
                    })
                    .collect()
            })
            .collect();
        let right_tiles = right_plan
            .iter()
            .map(|(c, map)| {
                let m = &right[*c];
                r_blocks
                    .iter()
                    .enumerate()
                    .map(|(rb_in, cols)| {
                        let rb_out = map[rb_in];
                        if rb_out == NONE {
                            return Vec::new();
                        }
                        let rows = &r_blocks[rb_out];
                        let mut t = Vec::with_capacity(rows.len() * cols.len());
                        for &i in rows {
                            for &j in cols {
                                t.push(m[i * chi_r + j]);
                            }
                        }
                        t
                    })
                    .collect()
            })
            .collect();
        Ok(BlockPlan { chi_l, chi_r, l_blocks, r_blocks, cut_r_block, left: left_plan, right: right_plan, left_tiles, right_tiles, scratch: std::sync::Mutex::new(Scratch::default()) })
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

    /// Stage 1 on tiles: the right environment applied on the live blocks only, every tile
    /// row a contiguous dot with the gathered `ψ` block. The accumulation over `r_in` runs in
    /// ascending index order exactly as the dense stage does, so the bits are the dense bits.
    fn step1(&self, psi: &[f64], d_r: usize, t1: &mut [f64]) {
        let (chi_l, chi_r) = (self.chi_l, self.chi_r);
        let block = chi_l * 4 * chi_r;
        debug_assert_eq!(t1.len(), d_r * block);
        let nthreads = mps::threads();
        let step = |ci: usize, t1c: &mut [f64]| {
            let map = &self.right[ci].1;
            let tiles = &self.right_tiles[ci];
            let mut pvec: Vec<f64> = Vec::new();
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
                        let n_in = r_ins.len();
                        pvec.clear();
                        pvec.extend(r_ins.iter().map(|&r| psi[base + r]));
                        let tile = &tiles[rb_in];
                        for (row, &r_out) in self.r_blocks[rb_out].iter().enumerate() {
                            let trow = &tile[row * n_in..row * n_in + n_in];
                            let mut acc = 0.0;
                            for k in 0..n_in {
                                acc += trow[k] * pvec[k];
                            }
                            t1c[base + r_out] = acc;
                        }
                    }
                }
            }
        };
        let mut jobs: Vec<(usize, &mut [f64])> = Vec::new();
        let chunks = t1.chunks_mut(block).enumerate();
        let mut ci = 0usize;
        for (c2, t1c) in chunks {
            if ci < self.right.len() && self.right[ci].0 == c2 {
                jobs.push((ci, t1c));
                ci += 1;
            }
        }
        if nthreads <= 1 || jobs.len() < 2 {
            for (ci, t1c) in jobs.iter_mut() {
                step(*ci, t1c);
            }
        } else {
            let per = jobs.len().div_ceil(nthreads).max(1);
            std::thread::scope(|sc| {
                for chunk in jobs.chunks_mut(per) {
                    let step = &step;
                    sc.spawn(move || {
                        for (ci, t1c) in chunk.iter_mut() {
                            step(*ci, t1c);
                        }
                    });
                }
            });
        }
    }

    /// Stage 4 on tiles: the left environment applied on the live blocks only, the work
    /// split over DISJOINT `l_out` blocks. For every output element the channels are visited
    /// in the dense order and the `l_in` sum inside a channel runs in ascending index order,
    /// accumulated apart and then added — the dense stage's arithmetic, on contiguous rows.
    fn step4(&self, _left: &Env, t3: &[f64]) -> Vec<f64> {
        let (chi_l, chi_r) = (self.chi_l, self.chi_r);
        let row = 4 * chi_r;
        let mut out = vec![0.0; chi_l * row];
        let nthreads = mps::threads();
        // one job per l_out block: its rows of `out`, contiguous when the block's indices are
        // consecutive, gathered otherwise — the job writes only its own rows
        let step = |lb_out: usize, out: &mut [f64]| {
            let rows_out = &self.l_blocks[lb_out];
            let mut tvec: Vec<f64> = Vec::new();
            for (ci, (c1, map)) in self.left.iter().enumerate() {
                let lb_in = map[lb_out];
                if lb_in == NONE {
                    continue;
                }
                let tile = &self.left_tiles[ci][lb_out];
                let l_ins = &self.l_blocks[lb_in];
                let n_in = l_ins.len();
                for st in 0..4 {
                    let rb = self.cut_r_block[lb_out][st];
                    if rb == NONE {
                        continue;
                    }
                    let r_outs = &self.r_blocks[rb];
                    for &r_out in r_outs {
                        // gather t3[c1][l_in, st, r_out] over the l_in block once per (st, r_out)
                        tvec.clear();
                        tvec.extend(l_ins.iter().map(|&l_in| t3[((c1 * chi_l + l_in) * 4 + st) * chi_r + r_out]));
                        for (ri, &l_out) in rows_out.iter().enumerate() {
                            let trow = &tile[ri * n_in..ri * n_in + n_in];
                            let mut acc = 0.0;
                            for k in 0..n_in {
                                acc += trow[k] * tvec[k];
                            }
                            out[l_out * row + st * chi_r + r_out] += acc;
                        }
                    }
                }
            }
        };
        let nblocks = self.l_blocks.len();
        if nthreads <= 1 || nblocks < 2 {
            for lb in 0..nblocks {
                step(lb, &mut out);
            }
        } else {
            // disjoint rows per block, but blocks interleave in `out`: each thread accumulates
            // its blocks into its own full-size buffer and the buffers are summed per element
            // in block order — every element receives exactly one nonzero contribution, so the
            // sum adds zeros to it and the bits are unchanged
            let per = nblocks.div_ceil(nthreads).max(1);
            let partials: Vec<Vec<f64>> = std::thread::scope(|sc| {
                let handles: Vec<_> = (0..nblocks)
                    .collect::<Vec<_>>()
                    .chunks(per)
                    .map(|chunk| {
                        let chunk = chunk.to_vec();
                        let step = &step;
                        sc.spawn(move || {
                            let mut buf = vec![0.0; chi_l * row];
                            for lb in chunk {
                                step(lb, &mut buf);
                            }
                            buf
                        })
                    })
                    .collect();
                handles.into_iter().map(|h| h.join().expect("stage-4 worker")).collect()
            });
            for p in partials {
                for (o, v) in out.iter_mut().zip(p) {
                    if v != 0.0 {
                        *o = v;
                    }
                }
            }
        }
        out
    }

    /// Stages 2 and 3 on the live blocks: the two MPO sites applied to `t1`, walking only the
    /// `r_out` block each `(channel, l, a, b)` slot can be nonzero in. Measured before this
    /// existed (examples/apply_bench.rs, N = 8, B = 2, χ = 256): with stages 1 and 4 sparse the
    /// dense MPO stages were 30.6 of a 40.4 ms matvec — 75 % — walking 4·χ² entries per nonzero
    /// MPO element at 3 % occupancy. The loop nest keeps the dense order `(c1', c2, t, b, l, a,
    /// r_out)` and `(c1, c1', s, a, l, t, r_out)`, so every output element receives the same
    /// nonzero contributions in the same order: the dense bits.
    fn apply_mpo_pair_blocks(&self, w1: &MpoSite, w2: &MpoSite, t1: &[f64], t2: &mut [f64], t3: &mut [f64]) {
        let (chi_l, chi_r) = (self.chi_l, self.chi_r);
        let (d_l, d_mid, d_r) = (w1.d_l, w1.d_r, w2.d_r);
        debug_assert_eq!(d_mid, w2.d_l);
        let block = chi_l * 4 * chi_r;
        debug_assert_eq!(t2.len(), d_mid * block);
        debug_assert_eq!(t3.len(), d_l * block);
        let nblk = self.l_blocks.len();
        let right_index: Vec<usize> = (0..d_r).map(|c2| self.right.iter().position(|(c, _)| *c == c2).unwrap_or(NONE)).collect();
        let nthreads = mps::threads();
        // Stage 2: t2[c1'][l, a, t][r] = Σ_{b, c2} t1[c2][l, a, b][r] · W2[c1', c2, t, b], one
        // output channel per job, its slots' r blocks recorded in mid_block for stage 3
        let mut mid_block = vec![vec![[NONE; 4]; nblk]; d_mid];
        let stage2 = |c1p: usize, t2c: &mut [f64], mid: &mut Vec<[usize; 4]>| {
            for c2 in 0..d_r {
                if right_index[c2] == NONE {
                    continue;
                }
                let map = &self.right[right_index[c2]].1;
                for t in 0..2 {
                    for b in 0..2 {
                        let wv = w2.get(c1p, c2, t, b);
                        if wv == 0.0 {
                            continue;
                        }
                        for (lb, lidx) in self.l_blocks.iter().enumerate() {
                            for a in 0..2 {
                                let rb_in = self.cut_r_block[lb][a * 2 + b];
                                if rb_in == NONE {
                                    continue;
                                }
                                let rb = map[rb_in];
                                if rb == NONE {
                                    continue;
                                }
                                let slot = &mut mid[lb][a * 2 + t];
                                let first = *slot == NONE;
                                debug_assert!(first || *slot == rb, "a t2 slot is live in two r blocks: the MPO does not conserve the labels");
                                *slot = rb;
                                let r_outs = &self.r_blocks[rb];
                                for &l_in in lidx {
                                    let src = (((c2 * chi_l + l_in) * 2 + a) * 2 + b) * chi_r;
                                    let dst = ((l_in * 2 + a) * 2 + t) * chi_r;
                                    if first {
                                        for &r_out in r_outs {
                                            t2c[dst + r_out] = wv * t1[src + r_out];
                                        }
                                    } else {
                                        for &r_out in r_outs {
                                            t2c[dst + r_out] += wv * t1[src + r_out];
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
        {
            let mut jobs: Vec<(usize, &mut [f64], &mut Vec<[usize; 4]>)> =
                t2.chunks_mut(block).zip(mid_block.iter_mut()).enumerate().map(|(c, (t, m))| (c, t, m)).collect();
            if nthreads <= 1 {
                for (c1p, t2c, mid) in jobs.iter_mut() {
                    stage2(*c1p, t2c, mid);
                }
            } else {
                let per = jobs.len().div_ceil(nthreads).max(1);
                std::thread::scope(|sc| {
                    for chunk in jobs.chunks_mut(per) {
                        let stage2 = &stage2;
                        sc.spawn(move || {
                            for (c1p, t2c, mid) in chunk.iter_mut() {
                                stage2(*c1p, t2c, mid);
                            }
                        });
                    }
                });
            }
        }
        // Stage 3: t3[c1][l, s, t][r] = Σ_{a, c1'} t2[c1'][l, a, t][r] · W1[c1, c1', s, a]
        let mid_block = &mid_block;
        let t2: &[f64] = t2;
        let stage3 = |c1: usize, t3c: &mut [f64]| {
            let mut seen = vec![[false; 4]; nblk];
            for c1p in 0..d_mid {
                for sb in 0..2 {
                    for a in 0..2 {
                        let wv = w1.get(c1, c1p, sb, a);
                        if wv == 0.0 {
                            continue;
                        }
                        for (lb, lidx) in self.l_blocks.iter().enumerate() {
                            for t in 0..2 {
                                let rb = mid_block[c1p][lb][a * 2 + t];
                                if rb == NONE {
                                    continue;
                                }
                                let first = !seen[lb][sb * 2 + t];
                                seen[lb][sb * 2 + t] = true;
                                let r_outs = &self.r_blocks[rb];
                                for &l_in in lidx {
                                    let src = (((c1p * chi_l + l_in) * 2 + a) * 2 + t) * chi_r;
                                    let dst = ((l_in * 2 + sb) * 2 + t) * chi_r;
                                    if first {
                                        for &r_out in r_outs {
                                            t3c[dst + r_out] = wv * t2[src + r_out];
                                        }
                                    } else {
                                        for &r_out in r_outs {
                                            t3c[dst + r_out] += wv * t2[src + r_out];
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        };
        let mut jobs: Vec<(usize, &mut [f64])> = t3.chunks_mut(block).enumerate().collect();
        if nthreads <= 1 {
            for (c1, t3c) in jobs.iter_mut() {
                stage3(*c1, t3c);
            }
        } else {
            let per = jobs.len().div_ceil(nthreads).max(1);
            std::thread::scope(|sc| {
                for chunk in jobs.chunks_mut(per) {
                    let stage3 = &stage3;
                    sc.spawn(move || {
                        for (c1, t3c) in chunk.iter_mut() {
                            stage3(*c1, t3c);
                        }
                    });
                }
            });
        }
    }

    /// The two-site effective Hamiltonian on `psi`, live blocks only. Bit-identical to
    /// `mps::apply_effective_h_mpo_live` on any label-consistent `psi`.
    pub fn apply(&self, left: &Env, w1: &MpoSite, w2: &MpoSite, right: &Env, psi: &[f64]) -> Vec<f64> {
        self.apply_timed(left, w1, w2, right, psi).0
    }

    /// [`apply`](Self::apply) with the three stages' seconds: `(stage 1, stages 2+3, stage 4)`.
    pub fn apply_timed(&self, left: &Env, w1: &MpoSite, w2: &MpoSite, right: &Env, psi: &[f64]) -> (Vec<f64>, [f64; 3]) {
        debug_assert_eq!(psi.len(), self.chi_l * 4 * self.chi_r);
        let block = self.chi_l * 4 * self.chi_r;
        let mut sc = self.scratch.lock().unwrap_or_else(|p| p.into_inner());
        if sc.t1.len() != w2.d_r * block {
            sc.t1 = vec![0.0; w2.d_r * block];
            sc.t2 = vec![0.0; w1.d_r * block];
            sc.t3 = vec![0.0; w1.d_l * block];
        }
        let Scratch { t1, t2, t3 } = &mut *sc;
        let t = std::time::Instant::now();
        self.step1(psi, w2.d_r, t1);
        let s1 = t.elapsed().as_secs_f64();
        let t = std::time::Instant::now();
        self.apply_mpo_pair_blocks(w1, w2, t1, t2, t3);
        let s23 = t.elapsed().as_secs_f64();
        let t = std::time::Instant::now();
        let out = self.step4(left, t3);
        let _ = right;
        (out, [s1, s23, t.elapsed().as_secs_f64()])
    }
}
