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
//!
//! ## The window is a TIME window, and that is not a detail
//!
//! The first version of this file converted the staked window into a frame count once, up
//! front, from the header's `dt`. That was wrong on real data and it was wrong in the
//! permissive direction. The engine's timestep is DERIVED from the scene and it ADAPTS:
//! on hydrogen seed `0x53415421` the header records `dt = 1.0772` at placement, the
//! timestep halves after eleven frames, and 19,988 of the 20,000 frames run at 0.5386.
//! A window computed from the header `dt` was therefore 500 frames of 1.6676 fs each on
//! paper and 500 frames of 0.8338 fs each in fact — 417 fs of simulated time against a
//! staked 834 fs, making certification twice as easy as it was staked to be.
//!
//! The dump carries `time` per frame, so the information was always there. Every window in
//! this file is now measured against those timestamps and never against `dt`. The
//! `Header::dt` field is the placement value only; nothing here reads it.

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
        max_breach_fs: f64,
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
    /// The longest STRICT held run, in frames and in simulated time. Reported for every
    /// block, so the distance to the bar is visible even when the verdict is TRANSIENT.
    /// The FS column is the one to read: frames are not a unit here.
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
    /// The staked window in simulated femtoseconds — the quantity that is actually
    /// enforced. There is no single frame equivalent, because the timestep adapts.
    pub window_fs: f64,
    pub flicker_fs: f64,
    /// The MEDIAN frame duration, for reading the frame columns as times.
    pub median_frame_fs: f64,
    /// How many distinct frame durations the run actually used, and the extremes. A run
    /// with more than one is a run where "frames" is not a unit, and the first real
    /// trajectory this instrument read had two.
    pub distinct_frame_durations: usize,
    pub min_frame_fs: f64,
    pub max_frame_fs: f64,
    /// The dimensionality the trajectory DECLARED. Recorded rather than inferred: a
    /// declaration and the measurement of whether it held are two different facts, and an
    /// accessor that derives one from the other collapses them.
    pub dims_declared: u32,
    /// Largest departure of any atom from its own placement `z`, in bohr.
    ///
    /// A scene declaring `dims = 2` must hold this at EXACTLY zero: a planar configuration
    /// under in-plane forces stays planar by symmetry, and seventeen of eighteen banked
    /// trajectories hold it bit-exactly across 20,000 frames. The eighteenth reached 11.49
    /// bohr against a 12.0 box half-depth while still declaring `dims = 2`, which made
    /// every dimension-keyed lens refusal in this crate a decision taken on a false
    /// premise. The declaration is now CHECKED rather than trusted.
    pub max_z_excursion: f64,
    /// True when the scene's motion contradicts its declared dimensionality.
    pub dims_declaration_violated: bool,
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
    let nf = traj.frames.len();

    // TIMESTAMPS, not `dt`. The engine's timestep is derived from the scene and adapts
    // during a run; the header's value is the one at placement. See the module header for
    // what reading `dt` instead cost on the first real trajectory.
    let times: Vec<f64> = traj
        .frames
        .iter()
        .map(|f| f.time * crate::traj::AU_TIME_FS)
        .collect();
    if nf < 2 {
        return Census::Refused {
            gate: "at least 2 frames",
            reason: format!("{nf} frames; no window and no motion exists"),
        };
    }
    let mut deltas: Vec<f64> = (1..nf).map(|i| times[i] - times[i - 1]).collect();
    deltas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_frame_fs = deltas[deltas.len() / 2];
    let min_frame_fs = deltas[0];
    let max_frame_fs = deltas[deltas.len() - 1];
    let distinct = {
        let mut d = deltas.clone();
        d.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        d.len()
    };
    // The declared dimensionality, MEASURED. Every lens refusal keyed on `dims` is only as
    // good as this, and a header is a claim rather than an observation.
    let mut max_z_excursion = 0.0f64;
    if let Some(f0) = traj.frames.first() {
        for f in &traj.frames {
            for i in 0..n {
                let d = (f.pos[i][2] - f0.pos[i][2]).abs();
                if d > max_z_excursion {
                    max_z_excursion = d;
                }
            }
        }
    }
    let dims_declaration_violated = traj.header.dims == 2 && max_z_excursion > 0.0;

    let span_fs = times[nf - 1] - times[0];
    if span_fs < st.window_fs {
        return Census::Refused {
            gate: "G3/G4 window length",
            reason: format!(
                "the trajectory spans {span_fs:.1} fs of simulated time and the staked \
                 window is {:.1} fs; no held reading is possible. The gate whose passing \
                 would lift this refusal is a longer trajectory.",
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
    // Per-frame durations, for the held-time pre-filter.
    let dur: Vec<f64> = (0..nf)
        .map(|i| if i + 1 < nf { times[i + 1] - times[i] } else { median_frame_fs })
        .collect();

    for m in candidates {
        let frames_present = present[&m];
        let series = series_for(&blocks_at, m);
        let longest_run = longest_true_run(&series);
        let (longest_fs, longest_at) = longest_true_run_fs(&series, &times);
        // Cheap exact pre-filter, and the bar is the BUDGETED one: the budgeted reading
        // only asks for (1-beta) of the window's frames to be held, so filtering on the
        // full window would silently reject exactly the blocks the budget exists for.
        // Measured in TIME, because frames are not a unit.
        let held_fs: f64 = series
            .iter()
            .enumerate()
            .filter(|(_, v)| **v)
            .map(|(i, _)| dur[i])
            .sum();
        let hit = if held_fs < (1.0 - st.beta) * st.window_fs {
            None
        } else {
            strict_window(&series, &times, st.window_fs)
                .map(|(a, b)| (a, b, 0usize, 0.0f64, true))
                .or_else(|| {
                    budgeted_window(&series, &times, st.window_fs, st.beta, st.flicker_fs)
                        .map(|(a, b, breach, worst)| (a, b, breach, worst, false))
                })
        };

        let (verdict, rms, sep_var, control) = match hit {
            None => {
                // Report the motion over the longest run anyway, so a TRANSIENT row is not
                // silently also a frozen one.
                let (rms, sv) = if longest_run >= 2 {
                    carrier_motion(traj, m, longest_at, longest_at + longest_run)
                } else {
                    (0.0, 0.0)
                };
                (BlockVerdict::Transient { longest_run }, rms, sv, None)
            }
            Some((a, b, breach, worst, strict)) => {
                let (rms, sv) = carrier_motion(traj, m, a, b);
                if rms < st.min_rms_bohr || sv < st.min_sep_var_bohr {
                    (
                        BlockVerdict::VoidFrozenCarrier { rms, sep_var: sv },
                        rms,
                        sv,
                        None,
                    )
                } else {
                    let (rate, pool) = control_rate(traj, &blocks_at, &times, m, st);
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
                                max_breach_fs: worst,
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
            longest_run_fs: longest_fs,
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
        window_fs: st.window_fs,
        flicker_fs: st.flicker_fs,
        median_frame_fs,
        distinct_frame_durations: distinct,
        min_frame_fs,
        max_frame_fs,
        dims_declared: traj.header.dims,
        max_z_excursion,
        dims_declaration_violated,
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

/// The longest held run measured in SIMULATED TIME, and where it starts.
///
/// Frames are not interchangeable units: the engine's timestep adapts, so "the longest run
/// of 3951 frames" is not a duration until the timestamps say so.
pub fn longest_true_run_fs(s: &[bool], t: &[f64]) -> (f64, usize) {
    let (mut best, mut best_at) = (0.0f64, 0usize);
    let mut i = 0usize;
    while i < s.len() {
        if !s[i] {
            i += 1;
            continue;
        }
        let start = i;
        while i < s.len() && s[i] {
            i += 1;
        }
        let span = t[i - 1] - t[start];
        if span > best {
            best = span;
            best_at = start;
        }
    }
    (best, best_at)
}

/// Zero-runs as `(start, end_exclusive, duration_fs)`.
///
/// A breach's duration runs from its first breached frame to the next HELD frame, which is
/// the elapsed time the block was not a block. Where the run reaches the end of the
/// trajectory there is no next held frame and the last timestamp is used instead.
fn zero_runs(s: &[bool], t: &[f64]) -> Vec<(usize, usize, f64)> {
    let n = s.len();
    let mut out = Vec::new();
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
        let end_t = if i < n { t[i] } else { t[n - 1] };
        out.push((start, i, end_t - t[start]));
        i += 1 - 1;
        if i == start {
            i += 1;
        }
    }
    out
}

/// The first window of `w_fs` SIMULATED FEMTOSECONDS in which the block is held at every
/// frame. Returns `(start, end_exclusive)` for the minimal such window.
pub fn strict_window(s: &[bool], t: &[f64], w_fs: f64) -> Option<(usize, usize)> {
    let n = s.len();
    let mut i = 0usize;
    while i < n {
        if !s[i] {
            i += 1;
            continue;
        }
        let start = i;
        while i < n && s[i] {
            i += 1;
        }
        // The run is [start, i). Does any prefix of it span the window?
        if t[i - 1] - t[start] >= w_fs {
            let mut b = start;
            while t[b] - t[start] < w_fs {
                b += 1;
            }
            return Some((start, b + 1));
        }
    }
    None
}

/// The first window of `w_fs` simulated femtoseconds meeting BOTH budget clauses.
///
/// Returns `(start, end_exclusive, breach_frames, longest_breach_fs)`.
///
/// The window is required to read held at BOTH ENDPOINTS. With the endpoints held, every
/// breach overlapping the window lies entirely inside it, so "the longest breach in the
/// window" is unambiguous and no window can be placed so that a long dissociation is
/// clipped by its edge down to a passing length.
pub fn budgeted_window(
    s: &[bool],
    t: &[f64],
    w_fs: f64,
    beta: f64,
    flicker_fs: f64,
) -> Option<(usize, usize, usize, f64)> {
    let n = s.len();
    if n == 0 {
        return None;
    }
    let mut prefix = vec![0usize; n + 1];
    for i in 0..n {
        prefix[i + 1] = prefix[i] + s[i] as usize;
    }
    let runs = zero_runs(s, t);
    let starts: Vec<usize> = runs.iter().map(|r| r.0).collect();
    let durs: Vec<f64> = runs.iter().map(|r| r.2).collect();
    let table = SparseMaxF::build(&durs);

    let mut b = 0usize;
    for a in 0..n {
        if !s[a] {
            continue;
        }
        if b < a {
            b = a;
        }
        while b < n && t[b] - t[a] < w_fs {
            b += 1;
        }
        if b >= n {
            return None; // no later start can reach the window either
        }
        if !s[b] {
            continue;
        }
        let len = b - a + 1;
        let ones = prefix[b + 1] - prefix[a];
        let need = ((1.0 - beta) * len as f64).ceil() as usize;
        if ones < need {
            continue;
        }
        let lo = starts.partition_point(|&x| x < a);
        let hi = starts.partition_point(|&x| x <= b);
        let worst = if lo < hi { table.max(lo, hi) } else { 0.0 };
        if worst <= flicker_fs {
            return Some((a, b + 1, len - ones, worst));
        }
    }
    None
}

/// Range-max over a fixed array, so the budget scan stays linear in the number of windows
/// rather than quadratic in the window length.
struct SparseMaxF {
    levels: Vec<Vec<f64>>,
}

impl SparseMaxF {
    fn build(v: &[f64]) -> Self {
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
    fn max(&self, lo: usize, hi: usize) -> f64 {
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
pub fn carrier_motion(traj: &Trajectory, m: Mask, a: usize, b: usize) -> (f64, f64) {
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
    times: &[f64],
    m: Mask,
    st: &Stakes,
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
        let hit = strict_window(&s, times, st.window_fs).is_some()
            || budgeted_window(&s, times, st.window_fs, st.beta, st.flicker_fs).is_some();
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

    /// THE PLANARITY GATE, both directions. A declared-2D scene that stays planar must not
    /// trip it, and one that leaves the plane must -- with the excursion reported, since
    /// "violated" without a magnitude is as unusable as a control rate without its pool.
    #[test]
    fn a_declared_planar_scene_that_leaves_the_plane_is_flagged() {
        use crate::synthetic::{self, Spec};
        let z = vec![8, 8, 8, 8, 1, 1, 1, 1, 1, 1, 1, 1];
        let n = 12usize;

        // Planar: z never moves. This is what seventeen of eighteen banked trajectories do.
        let flat = synthetic::vibrating_block(Spec::quench_like(1200, z.clone()), 0b0011_0001, 0.4, |_| true);
        let r = match run(&flat, &Stakes::default()) {
            Census::Report(r) => r,
            _ => panic!("refused"),
        };
        assert_eq!(r.dims_declared, 2);
        assert_eq!(r.max_z_excursion, 0.0);
        assert!(!r.dims_declaration_violated);

        // The same scene with one atom drifting out of plane, declaring dims = 2 anyway.
        let mut sp = Spec::quench_like(1200, z);
        sp.seed = 99;
        let escaped = synthetic::build(sp, move |t, pos, vel| {
            for i in 0..n {
                pos[i] = [3.0 + (i as f64), 3.0, if i == 5 { t as f64 * 0.001 } else { 0.0 }];
                vel[i] = [0.0; 3];
            }
            synthetic::bonds_from_blocks(n, &[0b0011_0001])
        });
        let r2 = match run(&escaped, &Stakes::default()) {
            Census::Report(r) => r,
            _ => panic!("refused"),
        };
        assert_eq!(r2.dims_declared, 2, "it still CLAIMS to be planar");
        assert!(r2.max_z_excursion > 1.0, "excursion {}", r2.max_z_excursion);
        assert!(
            r2.dims_declaration_violated,
            "a scene that leaves its declared plane must be flagged"
        );
    }

    #[test]
    fn longest_run_counts_consecutive_only() {
        assert_eq!(longest_true_run(&[true, true, false, true]), 2);
        assert_eq!(longest_true_run(&[false, false]), 0);
        assert_eq!(longest_true_run(&[true; 5]), 5);
    }

    /// A uniform clock, so a frame count and a duration are interchangeable in these
    /// unit tests and only in these unit tests.
    fn clock(n: usize, step_fs: f64) -> Vec<f64> {
        (0..n).map(|i| i as f64 * step_fs).collect()
    }

    #[test]
    fn strict_window_finds_the_first_full_run() {
        let s = [true, false, true, true, true, true];
        let t = clock(6, 1.0);
        // Frames 2..5 span t=2 to t=5, three femtoseconds.
        assert_eq!(strict_window(&s, &t, 3.0), Some((2, 6)));
        assert_eq!(strict_window(&s, &t, 4.0), None);
    }

    /// THE BUG THIS API EXISTS FOR: a window staked in TIME must not shrink when the
    /// engine halves its timestep.
    ///
    /// Two clocks over the same held series — one at 1.6676 fs a frame, one at 0.8338 —
    /// are the two timesteps hydrogen seed 0x53415421 actually ran at. A window computed
    /// once from the header `dt` would have accepted 500 frames on both, which is 834 fs
    /// on the first clock and 417 fs on the second. Windowing on the timestamps gives the
    /// same DURATION on both, which is what was staked.
    #[test]
    fn the_window_is_a_duration_and_survives_a_timestep_change() {
        let held = vec![true; 2000];
        let coarse = clock(2000, 1.6676);
        let fine = clock(2000, 0.8338);
        let (a1, b1) = strict_window(&held, &coarse, 834.0).unwrap();
        let (a2, b2) = strict_window(&held, &fine, 834.0).unwrap();
        assert_eq!(a1, 0);
        assert_eq!(a2, 0);
        assert_eq!(b1, 502, "501 coarse frames reach the window");
        assert_eq!(b2, 1002, "and it takes 1001 fine ones");
        // The DURATIONS agree; the frame counts do not, which is the whole point.
        let d1 = coarse[b1 - 1] - coarse[a1];
        let d2 = fine[b2 - 1] - fine[a2];
        assert!(d1 >= 834.0 && d1 < 836.0, "coarse span {d1}");
        assert!(d2 >= 834.0 && d2 < 835.0, "fine span {d2}");
        assert!((d1 - d2).abs() < 1.7, "the two clocks staked the same duration");
    }

    /// And the same on a MIXED clock: eleven coarse frames then fine ones, exactly the
    /// shape the real trajectory had.
    #[test]
    fn a_clock_that_changes_mid_run_is_windowed_by_time() {
        let n = 3000;
        let mut t = Vec::with_capacity(n);
        let mut acc = 0.0;
        for i in 0..n {
            t.push(acc);
            acc += if i < 11 { 1.6676 } else { 0.8338 };
        }
        let held = vec![true; n];
        let (a, b) = strict_window(&held, &t, 834.0).unwrap();
        assert_eq!(a, 0);
        let span = t[b - 1] - t[a];
        assert!(span >= 834.0 && span < 835.0, "span {span}");
        // 11 coarse frames carry 18.3 fs; the rest of the window is fine frames.
        assert!(b > 990 && b < 1000, "b = {b}");
    }

    /// The budget admits a short flicker.
    #[test]
    fn budget_admits_one_short_breach() {
        let mut s = vec![true; 200];
        s[100] = false;
        let t = clock(200, 0.8338);
        assert!(strict_window(&s, &t, 50.0).is_some(), "runs either side clear 50 fs");
        assert_eq!(strict_window(&s, &t, 100.0), None, "neither clears 100 fs");
        // A 150 fs window has to contain the breach, and the budget admits it.
        let hit = budgeted_window(&s, &t, 150.0, 0.02, 8.4).unwrap();
        assert!(hit.2 >= 1, "the breach is counted: {hit:?}");
        assert!(hit.3 > 0.0 && hit.3 <= 8.4, "and its duration is inside the cap");
    }

    /// C-3, in unit form: a breach LONGER than the flicker cap must be refused even when
    /// the total breach fraction is inside the 2% budget. This is the test that stops the
    /// budget from being an escape hatch.
    #[test]
    fn budget_refuses_a_long_breach_inside_the_fraction() {
        // 1300 frames is 1084 fs: long enough for one 834 fs window, short enough that no
        // CLEAN one exists after the breach. That is what makes the breach unavoidable.
        let n = 1300;
        let t = clock(n, 0.8338);
        let mut s = vec![true; n];
        for i in 500..515 {
            s[i] = false; // 15 frames = 12.5 fs, past an 8.4 fs cap
        }
        assert_eq!(strict_window(&s, &t, 834.0), None, "no clean window exists at all");
        assert!(15.0 / 1000.0 < PREREG_BETA, "the fraction is inside beta");
        assert_eq!(budgeted_window(&s, &t, 834.0, 0.02, 8.4), None);
        // Loosen ONLY the run cap and it passes, so the run cap is what refused.
        assert!(budgeted_window(&s, &t, 834.0, 0.02, 20.0).is_some());
    }

    /// A window may not be placed so that a long breach is clipped by its edge.
    #[test]
    fn a_window_cannot_clip_a_dissociation_at_its_edge() {
        // 250 frames at 1 fs, with 60 fs of dissociation in the middle. Every 100 fs
        // window either straddles the breach or runs off the end, so the only way to pass
        // would be to clip the breach at an edge -- which the held-endpoints rule forbids.
        let n = 250;
        let t = clock(n, 1.0);
        let mut s = vec![true; n];
        for i in 100..160 {
            s[i] = false;
        }
        assert_eq!(budgeted_window(&s, &t, 100.0, 0.5, 5.0), None);
    }

    /// The pre-filter must use the BUDGET's floor, not the window's.
    #[test]
    fn a_block_short_of_the_window_can_still_pass_the_budget() {
        // Ten single-frame flickers spread across the WHOLE run, so no clean stretch is
        // long enough on its own.
        let n = 1200;
        let t = clock(n, 0.8338);
        let mut s = vec![true; n];
        for k in 0..10 {
            s[50 + 100 * k] = false;
        }
        assert_eq!(strict_window(&s, &t, 834.0), None, "no strict window exists");
        assert!(
            budgeted_window(&s, &t, 834.0, 0.02, 8.4).is_some(),
            "but a budgeted one does"
        );
    }

    #[test]
    fn longest_run_in_time_reads_the_clock_not_the_frame_count() {
        let s = [true, true, true, false, true, true];
        // A slow clock at the start, a fast one at the end: the THREE-frame run is longer
        // in time than the two-frame one, and would be either way; the point is that the
        // answer is a duration.
        let t = vec![0.0, 10.0, 20.0, 30.0, 31.0, 32.0];
        let (fs, at) = longest_true_run_fs(&s, &t);
        assert_eq!(at, 0);
        assert!((fs - 20.0).abs() < 1e-12);
    }

    #[test]
    fn sparse_max_matches_a_naive_scan() {
        let v: Vec<f64> = (0..64).map(|i| ((i * 37 + 11) % 29) as f64).collect();
        let t = SparseMaxF::build(&v);
        for lo in 0..v.len() {
            for hi in (lo + 1)..=v.len() {
                let naive = v[lo..hi].iter().copied().fold(f64::MIN, f64::max);
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
