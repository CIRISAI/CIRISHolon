//! THE CLOSURE CENSUS: is this a molecule, or a component whose formula reads like one?
//!
//! Frozen in `conformance/water_observatory/CENSUS_PREREG.md` before this file existed.
//! Every threshold below is a `PREREG_*` constant, declared once, so that the stake and
//! the instrument cannot drift apart — and so that a reader can grep one prefix and see
//! the whole of what was promised.
//!
//! Two legs, because "Closed" and "persistent" are different claims:
//!
//! * **Leg A, HELD.** For a block `B` of arena indices, the view `v_B(x) = 1` iff `B` is
//!   EXACTLY a block of the bonded partition. `Held v_B T` over a window is `v_B ≡ 1`
//!   across it. This is the persistence question, and it is the one the road asks.
//! * **Leg B, CLOSED.** For the full partition view, `closed_iff_fiber_invariant` says
//!   closure IS `∀ x y, v x = v y → v (T x) = v (T y)`. On a trajectory we can test the
//!   fibers we observed: two frames reading the same partition whose NEXT frames read
//!   different partitions are a witness pair in the exact sense of
//!   `nonfactoring_iff_not_closed`. Leg B can exhibit those; it can never prove closure,
//!   and it never says it has.
//!
//! Both legs carry an anti-vacuity gate, because both are the kind of test that passes
//! loudest when it is measuring nothing (M-VACUOUS-SUCCESS, M-FIXED-POINT-TRAJECTORY).

use crate::partition::{self, Mask};
use crate::traj::Trajectory;
use std::collections::HashMap;

// ============================================================ THE STAKES, from the prereg

/// The window, in femtoseconds. 91 O–H stretch periods, 40 bends, 1.6 free rotations at
/// 300 K. Staked in TIME, not frames, because `dt` is derived per scene.
pub const PREREG_WINDOW_FS: f64 = 834.0;
/// At most 2% of a window's frames may read not-held.
pub const PREREG_BETA: f64 = 0.02;
/// No single breach may exceed one O–H stretch period. This is the clause that stops the
/// 2% from being an escape hatch: 2% of 1000 frames is 20 consecutive frames, and 20
/// consecutive frames of dissociation is a dissociation.
pub const PREREG_FLICKER_FS: f64 = 8.4;
/// A held block must be a MOVING carrier, or the reading is vacuous.
pub const PREREG_MIN_RMS_BOHR: f64 = 0.1;
pub const PREREG_MIN_SEP_VAR_BOHR: f64 = 0.05;
/// Leg B's work count. Below this the functionality test constrains nothing.
pub const PREREG_MIN_INFORMATIVE: usize = 200;
/// The control floor: if more than 5% of same-composition blocks also pass, the census is
/// not discriminating on this trajectory.
pub const PREREG_CONTROL_MAX_RATE: f64 = 0.05;
/// Sampled only when the eligible pool is larger than this; at twelve atoms it never is,
/// so the base rate below is EXACT rather than estimated.
pub const PREREG_CONTROL_DRAWS: usize = 200;
/// OBJECT.md rule 1's non-expansive leak, in its discrete form.
pub const PREREG_NONEXPANSION: f64 = 1.05;

#[derive(Clone, Copy, Debug)]
pub struct Stakes {
    pub window_fs: f64,
    pub beta: f64,
    pub flicker_fs: f64,
    pub min_rms_bohr: f64,
    pub min_sep_var_bohr: f64,
    pub min_informative: usize,
    pub control_max_rate: f64,
    pub nonexpansion: f64,
}

impl Default for Stakes {
    fn default() -> Self {
        Self {
            window_fs: PREREG_WINDOW_FS,
            beta: PREREG_BETA,
            flicker_fs: PREREG_FLICKER_FS,
            min_rms_bohr: PREREG_MIN_RMS_BOHR,
            min_sep_var_bohr: PREREG_MIN_SEP_VAR_BOHR,
            min_informative: PREREG_MIN_INFORMATIVE,
            control_max_rate: PREREG_CONTROL_MAX_RATE,
            nonexpansion: PREREG_NONEXPANSION,
        }
    }
}

// ============================================================ verdicts

#[derive(Clone, Debug, PartialEq)]
pub enum BlockVerdict {
    /// `v_B ≡ 1` across a whole window, and the carrier moves.
    CertifiedStrict { window_start: usize },
    /// Held within the budget, with both clauses met, and the carrier moves.
    CertifiedBudgeted {
        window_start: usize,
        breach_frames: usize,
        max_breach_run: usize,
    },
    /// Neither reading reaches the window.
    Transient { longest_run: usize },
    /// Held, but on a carrier that is not moving: the gate cannot mean what it says.
    VoidFrozenCarrier { rms: f64, sep_var: f64 },
    /// Held, but so are its peers: the criterion is not separating on this trajectory.
    VoidNoSeparation { control_rate: f64 },
}

impl BlockVerdict {
    pub fn is_certified(&self) -> bool {
        matches!(
            self,
            BlockVerdict::CertifiedStrict { .. } | BlockVerdict::CertifiedBudgeted { .. }
        )
    }
    pub fn tag(&self) -> &'static str {
        match self {
            BlockVerdict::CertifiedStrict { .. } => "CERTIFIED-STRICT",
            BlockVerdict::CertifiedBudgeted { .. } => "CERTIFIED-BUDGETED",
            BlockVerdict::Transient { .. } => "TRANSIENT",
            BlockVerdict::VoidFrozenCarrier { .. } => "VOID(frozen carrier)",
            BlockVerdict::VoidNoSeparation { .. } => "VOID(no separation)",
        }
    }
}

#[derive(Clone, Debug)]
pub struct BlockReport {
    pub block: Mask,
    pub formula: String,
    /// Frames in which `B` was exactly a block of the partition.
    pub frames_present: usize,
    /// The longest STRICT held run, in frames — reported for every block, so the distance
    /// to the bar is visible even when the verdict is TRANSIENT.
    pub longest_run: usize,
    pub longest_run_fs: f64,
    pub verdict: BlockVerdict,
    /// Internal RMS displacement in the block's own centroid frame, over the certified
    /// window (or over the longest run, when there is no certified window).
    pub rms_internal: f64,
    pub max_sep_variation: f64,
    /// Fraction of the SAME-COMPOSITION eligible pool, excluding `B` itself, that also
    /// reaches the window. The base rate M-BASE-RATE-OMITTED demands, computed exactly.
    pub control_rate: Option<f64>,
    pub control_pool: usize,
    /// True when `B` is a component of the FINAL frame — i.e. when connected-component
    /// naming would have printed it as a molecule.
    pub named_at_final_frame: bool,
}

/// Leg B: the empirical fiber-invariance test on the full partition view.
#[derive(Clone, Debug)]
pub struct ClosureLeg {
    pub distinct_readings: usize,
    /// Transitions departing a reading that was visited at least twice. The WORK COUNT.
    pub informative_transitions: usize,
    pub total_transitions: usize,
    /// Fraction of informative transitions that violate functionality.
    pub defect: f64,
    pub defect_first_half: f64,
    pub defect_second_half: f64,
    /// Exhibited witness pairs `(s, t)`: `P_s = P_t` but `P_{s+1} ≠ P_{t+1}`. Capped for
    /// printing; `witness_pair_count` is the true total.
    pub witness_pairs: Vec<(usize, usize)>,
    pub witness_pair_count: usize,
    pub void: bool,
    pub nonexpansion_ok: bool,
}

#[derive(Clone, Debug)]
pub enum Census {
    /// Object rule 9: outside its certified scope, the instrument refuses and names the
    /// gate whose passing would lift the refusal.
    Refused { gate: &'static str, reason: String },
    Report(CensusReport),
}

#[derive(Clone, Debug)]
pub struct CensusReport {
    pub seed: u64,
    pub n_atoms: usize,
    pub n_frames: usize,
    pub complete: bool,
    pub window_frames: usize,
    pub flicker_frames: usize,
    pub frame_fs: f64,
    pub blocks: Vec<BlockReport>,
    pub closure: ClosureLeg,
    /// What connected-component naming reports: the final frame's components of size ≥ 2.
    pub final_frame_molecules: Vec<(Mask, String)>,
}

impl CensusReport {
    pub fn certified(&self) -> impl Iterator<Item = &BlockReport> {
        self.blocks.iter().filter(|b| b.verdict.is_certified())
    }
}

// ============================================================ the run

pub fn run(traj: &Trajectory, st: &Stakes) -> Census {
    let n = traj.header.n_atoms;
    let frame_fs = traj.header.frame_fs();
    let window = traj.header.frames_in(st.window_fs);
    let flicker = (st.flicker_fs / frame_fs).floor().max(1.0) as usize;
    let nf = traj.frames.len();

    if nf < window {
        return Census::Refused {
            gate: "G3/G4 window length",
            reason: format!(
                "the trajectory is {nf} frames ({:.1} fs) and the staked window is {window} \
                 frames ({:.1} fs); no held reading is possible. The gate whose passing \
                 would lift this refusal is a trajectory of at least {window} frames.",
                nf as f64 * frame_fs,
                st.window_fs
            ),
        };
    }

    // ---- the membership view, frame by frame -------------------------------------
    let labels: Vec<Vec<u8>> = traj
        .frames
        .iter()
        .map(|f| partition::labels_from_bonds(n, f.bonded))
        .collect();
    let blocks_at: Vec<Vec<Mask>> = labels.iter().map(partition::blocks).collect();
    let keys: Vec<u64> = labels.iter().map(partition::key).collect();

    // Every block of two or more atoms that is EVER exactly a component. A block present
    // in fewer frames than the window cannot reach it, so the count is an exact
    // pre-filter and not a heuristic shortcut.
    let mut present: HashMap<Mask, usize> = HashMap::new();
    for bs in &blocks_at {
        for &m in bs {
            if partition::popcount(m) >= 2 {
                *present.entry(m).or_insert(0) += 1;
            }
        }
    }

    let final_blocks = blocks_at.last().cloned().unwrap_or_default();
    let final_frame_molecules: Vec<(Mask, String)> = final_blocks
        .iter()
        .copied()
        .filter(|&m| partition::popcount(m) >= 2)
        .map(|m| (m, partition::formula(m, &traj.header.z)))
        .collect();

    let mut candidates: Vec<Mask> = present.keys().copied().collect();
    candidates.sort_unstable();

    let mut reports = Vec::new();
    for m in candidates {
        let frames_present = present[&m];
        let series = series_for(&blocks_at, m);
        let longest_run = longest_true_run(&series);
        // Cheap exact pre-filter, and the bar is the BUDGETED one. A block present for
        // fewer than `window` frames in total can still pass the budgeted reading -- that
        // reading only asks for `(1-beta)*W` present frames INSIDE the window -- so
        // filtering on `window` would silently reject exactly the blocks the budget
        // exists for. The correct floor is the budget's own.
        let need = ((1.0 - st.beta) * window as f64).ceil() as usize;
        let hit = if frames_present < need {
            None
        } else {
            strict_window(&series, window)
                .map(|a| (a, 0usize, 0usize, true))
                .or_else(|| {
                    budgeted_window(&series, window, st.beta, flicker)
                        .map(|(a, breach, run)| (a, breach, run, false))
                })
        };

        let (verdict, rms, sep_var, control) = match hit {
            None => {
                // Still report the motion over the longest run, so a TRANSIENT row is not
                // silently also a frozen one.
                let (rms, sv) = match first_run_of(&series, longest_run) {
                    Some(a) if longest_run >= 2 => carrier_motion(traj, m, a, a + longest_run),
                    _ => (0.0, 0.0),
                };
                (BlockVerdict::Transient { longest_run }, rms, sv, None)
            }
            Some((a, breach, breach_run, strict)) => {
                let (rms, sv) = carrier_motion(traj, m, a, a + window);
                if rms < st.min_rms_bohr || sv < st.min_sep_var_bohr {
                    (
                        BlockVerdict::VoidFrozenCarrier { rms, sep_var: sv },
                        rms,
                        sv,
                        None,
                    )
                } else {
                    let (rate, pool) = control_rate(traj, &blocks_at, m, window, st, flicker);
                    if rate > st.control_max_rate {
                        (
                            BlockVerdict::VoidNoSeparation { control_rate: rate },
                            rms,
                            sv,
                            Some((rate, pool)),
                        )
                    } else if strict {
                        (
                            BlockVerdict::CertifiedStrict { window_start: a },
                            rms,
                            sv,
                            Some((rate, pool)),
                        )
                    } else {
                        (
                            BlockVerdict::CertifiedBudgeted {
                                window_start: a,
                                breach_frames: breach,
                                max_breach_run: breach_run,
                            },
                            rms,
                            sv,
                            Some((rate, pool)),
                        )
                    }
                }
            }
        };

        reports.push(BlockReport {
            block: m,
            formula: partition::formula(m, &traj.header.z),
            frames_present,
            longest_run,
            longest_run_fs: longest_run as f64 * frame_fs,
            verdict,
            rms_internal: rms,
            max_sep_variation: sep_var,
            control_rate: control.map(|(r, _)| r),
            control_pool: control.map(|(_, p)| p).unwrap_or(0),
            named_at_final_frame: final_blocks.contains(&m),
        });
    }

    // Longest-held first: the interesting rows are at the top whatever the verdict.
    reports.sort_by(|a, b| {
        b.longest_run
            .cmp(&a.longest_run)
            .then(a.block.cmp(&b.block))
    });

    Census::Report(CensusReport {
        seed: traj.header.seed,
        n_atoms: n,
        n_frames: nf,
        complete: traj.is_complete(),
        window_frames: window,
        flicker_frames: flicker,
        frame_fs,
        blocks: reports,
        closure: closure_leg(&keys, st),
        final_frame_molecules,
    })
}

// ============================================================ Leg A internals

fn series_for(blocks_at: &[Vec<Mask>], m: Mask) -> Vec<bool> {
    blocks_at.iter().map(|bs| bs.contains(&m)).collect()
}

pub fn longest_true_run(s: &[bool]) -> usize {
    let (mut best, mut cur) = (0usize, 0usize);
    for &v in s {
        cur = if v { cur + 1 } else { 0 };
        best = best.max(cur);
    }
    best
}

fn first_run_of(s: &[bool], len: usize) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let mut cur = 0usize;
    for (i, &v) in s.iter().enumerate() {
        cur = if v { cur + 1 } else { 0 };
        if cur == len {
            return Some(i + 1 - len);
        }
    }
    None
}

/// The first window of `w` consecutive frames all reading held.
pub fn strict_window(s: &[bool], w: usize) -> Option<usize> {
    first_run_of(s, w)
}

/// The first window meeting BOTH budget clauses, with its breach statistics.
///
/// The window is required to read held at BOTH ENDPOINTS. That is not decoration: with
/// the endpoints held, every breach run overlapping the window is entirely INSIDE it, so
/// "the longest breach in the window" is unambiguous and no window can be placed to clip
/// a long dissociation down to a passing length. It also matches what the claim means —
/// a window in which the molecule exists is one where it exists at the start and at the
/// end.
pub fn budgeted_window(
    s: &[bool],
    w: usize,
    beta: f64,
    flicker: usize,
) -> Option<(usize, usize, usize)> {
    let n = s.len();
    if w == 0 || n < w {
        return None;
    }
    let mut prefix = vec![0usize; n + 1];
    for i in 0..n {
        prefix[i + 1] = prefix[i] + s[i] as usize;
    }
    // Zero-runs, as (start, len), ascending by start.
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut i = 0usize;
    while i < n {
        if s[i] {
            i += 1;
            continue;
        }
        let start = i;
        while i < n && !s[i] {
            i += 1;
        }
        runs.push((start, i - start));
    }
    let starts: Vec<usize> = runs.iter().map(|r| r.0).collect();
    let lens: Vec<usize> = runs.iter().map(|r| r.1).collect();
    let table = SparseMax::build(&lens);

    let need = ((1.0 - beta) * w as f64).ceil() as usize;
    for a in 0..=(n - w) {
        if !s[a] || !s[a + w - 1] {
            continue;
        }
        let ones = prefix[a + w] - prefix[a];
        if ones < need {
            continue;
        }
        // Endpoints held, so runs overlapping [a, a+w) are contained in it.
        let lo = starts.partition_point(|&x| x < a);
        let hi = starts.partition_point(|&x| x < a + w);
        let worst = if lo < hi { table.max(lo, hi) } else { 0 };
        if worst <= flicker {
            return Some((a, w - ones, worst));
        }
    }
    None
}

/// Range-max over a fixed array, so the budget scan stays linear in the number of windows
/// rather than quadratic in the window length.
struct SparseMax {
    levels: Vec<Vec<usize>>,
}

impl SparseMax {
    fn build(v: &[usize]) -> Self {
        let mut levels = vec![v.to_vec()];
        let mut k = 1usize;
        while 2 * k <= v.len() {
            let prev = levels.last().unwrap();
            let len = v.len() + 1 - 2 * k;
            let mut cur = Vec::with_capacity(len);
            for i in 0..len {
                cur.push(prev[i].max(prev[i + k]));
            }
            levels.push(cur);
            k *= 2;
        }
        Self { levels }
    }
    /// Max over `[lo, hi)`; `lo < hi` required.
    fn max(&self, lo: usize, hi: usize) -> usize {
        let len = hi - lo;
        let k = usize::BITS as usize - 1 - len.leading_zeros() as usize;
        let lvl = &self.levels[k];
        lvl[lo].max(lvl[hi - (1 << k)])
    }
}

/// Internal motion of a block over `[a, b)`: RMS displacement about each atom's own mean
/// position in the block's centroid frame, and the largest excursion of any intra-block
/// separation.
///
/// The centroid is GEOMETRIC, not mass-weighted, because the trajectory format carries
/// nuclear charge rather than mass. The choice only shifts how much of the motion is
/// booked as translation rather than internal, and it is stated rather than hidden; the
/// second quantity, the separation excursion, is centroid-free and is the one that
/// settles a frozen carrier on its own.
fn carrier_motion(traj: &Trajectory, m: Mask, a: usize, b: usize) -> (f64, f64) {
    let idx: Vec<usize> = (0..traj.header.n_atoms).filter(|i| m >> i & 1 == 1).collect();
    let w = b - a;
    if idx.is_empty() || w < 2 {
        return (0.0, 0.0);
    }
    let mut rel: Vec<Vec<[f64; 3]>> = vec![Vec::with_capacity(w); idx.len()];
    for t in a..b {
        let f = &traj.frames[t];
        let mut c = [0.0f64; 3];
        for &i in &idx {
            for k in 0..3 {
                c[k] += f.pos[i][k];
            }
        }
        for k in 0..3 {
            c[k] /= idx.len() as f64;
        }
        for (s, &i) in idx.iter().enumerate() {
            rel[s].push([
                f.pos[i][0] - c[0],
                f.pos[i][1] - c[1],
                f.pos[i][2] - c[2],
            ]);
        }
    }
    let mut acc = 0.0f64;
    for series in &rel {
        let mut mean = [0.0f64; 3];
        for p in series {
            for k in 0..3 {
                mean[k] += p[k];
            }
        }
        for k in 0..3 {
            mean[k] /= w as f64;
        }
        for p in series {
            for k in 0..3 {
                let d = p[k] - mean[k];
                acc += d * d;
            }
        }
    }
    let rms = (acc / (idx.len() * w) as f64).sqrt();

    let mut sep_var = 0.0f64;
    for x in 0..idx.len() {
        for y in (x + 1)..idx.len() {
            let (mut lo, mut hi) = (f64::INFINITY, 0.0f64);
            for t in 0..w {
                let p = rel[x][t];
                let q = rel[y][t];
                let d = ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2))
                    .sqrt();
                lo = lo.min(d);
                hi = hi.max(d);
            }
            sep_var = sep_var.max(hi - lo);
        }
    }
    (rms, sep_var)
}

/// The eligible-pool pass rate: how often a block of the SAME COMPOSITION, other than
/// this one, also reaches the window.
///
/// At twelve atoms the pool of (1 O, 2 H) blocks is 4·C(8,2) = 112, far below the sampling
/// threshold, so this is an EXACT enumeration and not an estimate. `B` itself is excluded
/// from the numerator and the denominator: the question the floor asks is how often a
/// non-target passes, and including the target in its own base rate would understate the
/// separation for a pool of one and overstate it for a large one.
fn control_rate(
    traj: &Trajectory,
    blocks_at: &[Vec<Mask>],
    m: Mask,
    window: usize,
    st: &Stakes,
    flicker: usize,
) -> (f64, usize) {
    let n = traj.header.n_atoms;
    let target = partition::composition(m, &traj.header.z);
    let mut pool: Vec<Mask> = Vec::new();
    for cand in 1u32..(1u32 << n) {
        let cm = cand as Mask;
        if cm == m || partition::popcount(cm) != partition::popcount(m) {
            continue;
        }
        if partition::composition(cm, &traj.header.z) == target {
            pool.push(cm);
        }
    }
    if pool.is_empty() {
        return (0.0, 0);
    }
    let mut pass = 0usize;
    for &cm in &pool {
        let s = series_for(blocks_at, cm);
        let hit = strict_window(&s, window).is_some()
            || budgeted_window(&s, window, st.beta, flicker).is_some();
        if hit {
            pass += 1;
        }
    }
    (pass as f64 / pool.len() as f64, pool.len())
}

// ============================================================ Leg B

/// The empirical fiber-invariance test.
///
/// `closed_iff_fiber_invariant` makes closure the statement `∀ x y, v x = v y →
/// v (T x) = v (T y)`. Here `v` is the partition and `T` is one grain boundary, so the
/// test is whether the observed map from reading to next-reading is a FUNCTION. A pair
/// of frames that breaks it is a witness pair, exhibited by index.
///
/// The count of INFORMATIVE transitions is the work count, and it is why this function
/// can return `void`: over readings each visited once, the map is a function by
/// construction and the defect is zero while nothing has been tested.
pub fn closure_leg(keys: &[u64], st: &Stakes) -> ClosureLeg {
    let n = keys.len();
    let mut succ: HashMap<u64, HashMap<u64, Vec<usize>>> = HashMap::new();
    for t in 0..n.saturating_sub(1) {
        succ.entry(keys[t])
            .or_default()
            .entry(keys[t + 1])
            .or_default()
            .push(t);
    }
    let half = n / 2;
    let mut informative = 0usize;
    let mut violations = 0usize;
    let (mut inf_a, mut vio_a, mut inf_b, mut vio_b) = (0usize, 0usize, 0usize, 0usize);
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    let mut pair_count = 0usize;

    for (_, outs) in succ.iter() {
        let m: usize = outs.values().map(|v| v.len()).sum();
        if m < 2 {
            continue;
        }
        informative += m;
        let top = outs.values().map(|v| v.len()).max().unwrap_or(0);
        violations += m - top;
        // Halves, for the non-expansion reading.
        for (_, ts) in outs.iter() {
            for &t in ts {
                if t < half {
                    inf_a += 1;
                } else {
                    inf_b += 1;
                }
            }
        }
        if outs.len() > 1 {
            // Exhibit one concrete witness pair for this reading: the earliest frame of
            // the majority successor against the earliest frame of any other.
            let mut branches: Vec<(u64, usize)> =
                outs.iter().map(|(k, v)| (*k, v[0])).collect();
            branches.sort_by_key(|(_, t)| *t);
            pair_count += outs.len() - 1;
            if pairs.len() < 16 {
                pairs.push((branches[0].1, branches[1].1));
            }
            // Per-half violation bookkeeping: a reading whose successors differ
            // contributes its non-majority frames to whichever half they fall in.
            let maj = outs
                .iter()
                .max_by_key(|(_, v)| v.len())
                .map(|(k, _)| *k)
                .unwrap();
            for (k, ts) in outs.iter() {
                if *k == maj {
                    continue;
                }
                for &t in ts {
                    if t < half {
                        vio_a += 1;
                    } else {
                        vio_b += 1;
                    }
                }
            }
        }
    }

    let rate = |v: usize, i: usize| if i == 0 { 0.0 } else { v as f64 / i as f64 };
    let d_a = rate(vio_a, inf_a);
    let d_b = rate(vio_b, inf_b);
    ClosureLeg {
        distinct_readings: succ.len(),
        informative_transitions: informative,
        total_transitions: n.saturating_sub(1),
        defect: rate(violations, informative),
        defect_first_half: d_a,
        defect_second_half: d_b,
        witness_pairs: pairs,
        witness_pair_count: pair_count,
        void: informative < st.min_informative,
        nonexpansion_ok: d_b <= st.nonexpansion * d_a || d_a == 0.0 && d_b == 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_run_counts_consecutive_only() {
        assert_eq!(longest_true_run(&[true, true, false, true]), 2);
        assert_eq!(longest_true_run(&[false, false]), 0);
        assert_eq!(longest_true_run(&[true; 5]), 5);
    }

    #[test]
    fn strict_window_finds_the_first_full_run() {
        let s = [true, false, true, true, true, true];
        assert_eq!(strict_window(&s, 4), Some(2));
        assert_eq!(strict_window(&s, 5), None);
    }

    /// The budget admits a short flicker.
    #[test]
    fn budget_admits_one_short_breach() {
        let mut s = vec![true; 100];
        s[50] = false;
        let hit = budgeted_window(&s, 100, 0.02, 10).unwrap();
        assert_eq!(hit, (0, 1, 1));
        assert_eq!(strict_window(&s, 100), None, "and the strict reading refuses it");
    }

    /// C-3, in unit form: a breach run longer than the flicker cap must be REFUSED even
    /// when the total breach fraction is inside the 2% budget. This is the test that
    /// stops the budget from being an escape hatch.
    #[test]
    fn budget_refuses_a_long_breach_inside_the_fraction() {
        let mut s = vec![true; 1000];
        for t in 500..515 {
            s[t] = false; // 15 frames: 1.5% of the window, but longer than a 10-frame cap
        }
        assert!(1.5 < 2.0, "the breach fraction is inside beta");
        assert_eq!(budgeted_window(&s, 1000, 0.02, 10), None);
        // Loosen only the run cap and it passes, which proves the run cap is what refused.
        assert!(budgeted_window(&s, 1000, 0.02, 15).is_some());
    }

    /// A window may not be placed so that a long breach is clipped by its edge.
    #[test]
    fn a_window_cannot_clip_a_dissociation_at_its_edge() {
        // 40 held, 30 broken, 40 held. No 50-frame window should pass with a 5-frame cap.
        let mut s = vec![true; 110];
        for t in 40..70 {
            s[t] = false;
        }
        assert_eq!(budgeted_window(&s, 50, 0.5, 5), None);
    }

    /// The pre-filter must use the BUDGET's floor, not the window's.
    ///
    /// A block present for 990 of 1200 frames never fills a 1000-frame window strictly,
    /// but it can fill one within a 2% budget. Filtering candidates on `frames_present <
    /// window` would drop it before the budgeted reading ever ran -- rejecting precisely
    /// the blocks the budget was written for, and doing it invisibly.
    #[test]
    fn a_block_short_of_the_window_can_still_pass_the_budget() {
        let mut s = vec![true; 1200];
        // Ten scattered single-frame breaches inside the first window, plus a long tail
        // outside it: 1190 present in all, and no 1000-frame strict window anywhere.
        for k in 0..10 {
            s[50 + 90 * k] = false;
        }
        let present = s.iter().filter(|b| **b).count();
        assert_eq!(present, 1190);
        assert_eq!(strict_window(&s, 1000), None, "no strict window exists");
        let hit = budgeted_window(&s, 1000, 0.02, 10);
        assert!(hit.is_some(), "but a budgeted one does: {hit:?}");
        let need = ((1.0 - 0.02) * 1000.0f64).ceil() as usize;
        assert_eq!(need, 980);
        assert!(present >= need, "and the pre-filter's floor is {need}, not 1000");
    }

    #[test]
    fn sparse_max_matches_a_naive_scan() {
        let v: Vec<usize> = (0..64).map(|i| (i * 37 + 11) % 29).collect();
        let t = SparseMax::build(&v);
        for lo in 0..v.len() {
            for hi in (lo + 1)..=v.len() {
                let naive = v[lo..hi].iter().copied().max().unwrap();
                assert_eq!(t.max(lo, hi), naive, "range [{lo},{hi})");
            }
        }
    }

    /// Leg B on a reading visited once has NOTHING to say, and says so.
    #[test]
    fn closure_leg_is_void_on_readings_visited_once() {
        let keys: Vec<u64> = (0..500).collect(); // every reading distinct
        let leg = closure_leg(&keys, &Stakes::default());
        assert_eq!(leg.informative_transitions, 0);
        assert_eq!(leg.defect, 0.0);
        assert!(leg.void, "zero defect over zero work is VOID, not a pass");
    }

    /// A deterministic coarse dynamics has no witness pair, and the leg is not void.
    #[test]
    fn closure_leg_finds_no_witness_on_a_functional_map() {
        // Reading cycles 0 -> 1 -> 2 -> 0, deterministically, 900 times.
        let keys: Vec<u64> = (0..900).map(|i| (i % 3) as u64).collect();
        let leg = closure_leg(&keys, &Stakes::default());
        assert!(!leg.void);
        assert_eq!(leg.witness_pair_count, 0);
        assert_eq!(leg.defect, 0.0);
    }

    /// A reading whose successor depends on hidden micro state IS a witness pair, and the
    /// leg exhibits it by frame index.
    #[test]
    fn closure_leg_exhibits_a_witness_pair() {
        // 0 -> 1 the first 400 times, 0 -> 2 thereafter: the coarse view cannot be a
        // function of itself.
        let mut keys = Vec::new();
        for i in 0..800 {
            keys.push(0u64);
            keys.push(if i < 400 { 1 } else { 2 });
        }
        let leg = closure_leg(&keys, &Stakes::default());
        assert!(!leg.void);
        assert!(leg.witness_pair_count >= 1);
        assert!(leg.defect > 0.0);
        let (s, t) = leg.witness_pairs[0];
        assert_eq!(keys[s], keys[t], "a witness pair agrees on the reading");
        assert_ne!(keys[s + 1], keys[t + 1], "and splits on the next one");
    }
}
