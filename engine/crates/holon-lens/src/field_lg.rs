//! THE LATTICE-GAS CHART: the molecular scene read as FHP-6 mode occupancy.
//!
//! Rung 2, amendment A2. `conformance/water_observatory/RUNG2_PREREG_A2.md` is the freeze
//! (`e5bd812`) and every threshold below is one of its constants.
//!
//! # Why this chart and not a cell-field scheme
//!
//! The operator's instruction: this programme did fluid dynamics BEFORE water, and the
//! fluid chart must build on that machinery rather than beside it. The `(N, P)` sector
//! decomposition IS the conserved-fields chart — density and momentum — and it arrives with
//! the isotropy warrant and the sector census already paid for.
//!
//! **FHP-6, because this carrier is two-dimensional.** `engine/MESH_DESIGN.md` §2.1 makes
//! the standing 3D choice FCHC-24, on the ground that cubic point symmetry cannot carry an
//! isotropic fourth-rank momentum-flux tensor and the face-centred *hyper*-cubic 24 can.
//! These scenes are `dims = 2`, so FHP-6 is the like-for-like chart; nothing here adopts a
//! cheaper mode set and nothing here touches the 3D choice.
//!
//! > **CREDIT**, per the convergence rule — a hit, not a strike. FHP-6: Frisch, Hasslacher
//! > & Pomeau, *Lattice-gas automata for the Navier–Stokes equation*, Phys. Rev. Lett. **56**
//! > (1986) 1505 — the hexagonal lattice's fourth-order isotropy is theirs, and it is the
//! > whole warrant of the founding 64-state object. FCHC-24 (3D, not exercised here):
//! > d'Humières, Lallemand & Frisch, *Lattice gas models for 3D hydrodynamics*, Europhys.
//! > Lett. **2** (1986) 291.
//!
//! # THE SECTOR LABEL IS NOT COMPUTED HERE, AND THAT IS DELIBERATE
//!
//! `ciris-sim-core::regplus::sector` is the one implementation of the `(N, P)` label, and
//! its own test reproduces `Core/Lattice.lean`'s 53 sectors with histogram 44/7/2 in-tree.
//! Two implementations of one label is how the two of them come to disagree — the rule
//! `sim.rs` states about the bond reading and the census obeyed.
//!
//! This crate has ZERO dependencies by design (see its `Cargo.toml`: it is read by the
//! census and written by an example in `holon-render`, and it has been tested on a tree
//! where `holon-render` did not compile). So it cannot import `regplus`. The resolution is
//! that this module builds the six-bit LOCAL WORD and stops: the caller supplies the
//! labeller, and the A2 runner supplies `regplus::sector`. The label therefore has exactly
//! one implementation and this crate keeps its isolation.

use crate::field::{Grid, Kind, Reading, Refusal};
use crate::traj::Trajectory;

/// The six FHP directions in the axial integer coordinates
/// `ciris_sim_core::regplus::DIRECTIONS` uses.
///
/// A pinned copy, for the isolation reason in this module's header. It is cross-checked
/// against `regplus::DIRECTIONS` by an integration test in `holon-mesh`, which CAN see both
/// — the same pattern `field.rs` uses for the element masses.
pub const DIRECTIONS_AXIAL: [[i64; 2]; 6] =
    [[1, 0], [0, 1], [-1, 1], [-1, 0], [0, -1], [1, -1]];

/// The mode count. FHP-6 has no rest particle, and A2 §2.2 refuses to invent one.
pub const MODES: usize = 6;

/// Axial `[p, q]` to Cartesian: the hexagonal unit vectors at 60° spacing,
/// `[p, q] ↦ (p + q/2, q·√3/2)`.
pub fn cartesian(d: usize) -> [f64; 2] {
    let (p, q) = (DIRECTIONS_AXIAL[d][0] as f64, DIRECTIONS_AXIAL[d][1] as f64);
    [p + q * 0.5, q * (0.75f64).sqrt()]
}

/// Which map from molecular state to mode. The three non-real kinds are A2 §2.6's plants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapKind {
    /// THE MAP (A2 §2.2): the mode maximising `u · e_d`, ties to the lowest index.
    Velocity,
    /// **MAP-1, must REJECT.** The mode taken from the atom's ARENA INDEX instead of its
    /// velocity. The velocity-direction sector is exactly zero here; if this scores as well
    /// as `Velocity`, the chart is not reading lattice-gas structure at all.
    ByIndex,
    /// **MAP-2, must NOT fire.** The six directions cyclically relabelled by one position —
    /// a 60° rotation of the mode SET. The partition of velocities into modes is identical,
    /// so the collision structure must be bit-identical; only the labels move.
    Rotated,
}

/// The mode an atom's planar velocity occupies, or `None` for an atom with exactly zero
/// planar velocity — FHP-6 has no rest mode, so such an atom occupies none and is COUNTED
/// (A2 §2.2). It is never absorbed by inventing a seventh mode.
pub fn mode_of(vx: f64, vy: f64) -> Option<usize> {
    if vx == 0.0 && vy == 0.0 {
        return None;
    }
    let mut best = 0usize;
    let mut best_dot = f64::NEG_INFINITY;
    for d in 0..MODES {
        let e = cartesian(d);
        let dot = vx * e[0] + vy * e[1];
        // Strictly greater: ties break to the LOWEST index, as the freeze states.
        if dot > best_dot {
            best_dot = dot;
            best = d;
        }
    }
    Some(best)
}

/// What the map cost, printed for every grid whatever the verdict (A2 §2.2, gate A2g).
///
/// `Core/ModeChart.lean`'s fence is the warrant: Boolean occupancy is exact only for
/// DETERMINATE states, and over mixtures the exact invariant is the CAP — mean occupancy in
/// `[0,1]`. FHP is an exclusion automaton, so two atoms in one mode are LOST to the Boolean
/// word, and how often that happens bounds what the chart can be claimed to be a view of.
#[derive(Clone, Debug, Default)]
pub struct MapStats {
    /// (cell, frame) pairs in which some mode carried ≥ 2 atoms.
    pub saturated_cellframes: u64,
    /// (cell, frame) pairs that held at least one atom.
    pub occupied_cellframes: u64,
    /// Atoms dropped by the Boolean word: `Σ_modes max(0, count − 1)`.
    pub lost_atoms: u64,
    /// Atoms placed into some mode.
    pub placed_atoms: u64,
    /// Atoms with exactly zero planar velocity, which occupy no mode.
    pub zero_velocity_atoms: u64,
    /// Distinct six-bit local words seen anywhere (A2 gate A1 reads this).
    pub distinct_words: usize,
}

impl MapStats {
    /// The saturation rate: of the cell-frames that held anything, the fraction where the
    /// exclusion cap actually bit.
    pub fn saturation(&self) -> f64 {
        if self.occupied_cellframes == 0 {
            0.0
        } else {
            self.saturated_cellframes as f64 / self.occupied_cellframes as f64
        }
    }
    /// The fraction of placed atoms the Boolean word could not carry.
    pub fn lost_fraction(&self) -> f64 {
        if self.placed_atoms == 0 {
            0.0
        } else {
            self.lost_atoms as f64 / self.placed_atoms as f64
        }
    }
}

/// The six-bit local word per cell per frame, plus what the map cost.
///
/// Bit `d` of a cell's word is set iff at least one atom in that cell maps to mode `d`.
pub fn local_words(
    traj: &Trajectory,
    grid: Grid,
    kind: Kind,
    map: MapKind,
) -> Result<(Vec<Vec<u8>>, MapStats), Refusal> {
    let cells = crate::field::cell_series(traj, grid, kind)?;
    let n = traj.header.n_atoms;
    let nc = grid.cells();
    let mut stats = MapStats::default();
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::with_capacity(traj.frames.len());

    for (fi, f) in traj.frames.iter().enumerate() {
        // Per-mode counts, so saturation is measured rather than assumed away.
        let mut counts = vec![[0u32; MODES]; nc];
        for a in 0..n {
            let c = cells[fi][a];
            let m = match map {
                MapKind::Velocity => mode_of(f.vel[a][0], f.vel[a][1]),
                MapKind::Rotated => mode_of(f.vel[a][0], f.vel[a][1]).map(|d| (d + 1) % MODES),
                // The velocity-direction sector is exactly zero here, by construction.
                MapKind::ByIndex => Some(a % MODES),
            };
            match m {
                Some(d) => {
                    counts[c][d] += 1;
                    stats.placed_atoms += 1;
                }
                None => stats.zero_velocity_atoms += 1,
            }
        }
        let mut row = Vec::with_capacity(nc);
        for c in 0..nc {
            let mut word = 0u8;
            let mut held = false;
            let mut saturated = false;
            for d in 0..MODES {
                let k = counts[c][d];
                if k > 0 {
                    word |= 1 << d;
                    held = true;
                    if k > 1 {
                        saturated = true;
                        stats.lost_atoms += (k - 1) as u64;
                    }
                }
            }
            if held {
                stats.occupied_cellframes += 1;
                if saturated {
                    stats.saturated_cellframes += 1;
                }
            }
            seen.insert(word);
            row.push(word);
        }
        out.push(row);
    }
    stats.distinct_words = seen.len();
    Ok((out, stats))
}

/// The A2 chart ladder (A2 §1.1). Each rung is a refinement of the one before, and the
/// ladder is the machinery's own structure: the 64-state local word, its 53-sector
/// quotient, and the occupancy marginal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LgRung {
    /// w1 — `N`, the number of OCCUPIED MODES (0…6). **Not** `field.rs`'s `v1`, which
    /// counts ATOMS (0…12). Different fields; never conflate them.
    W1,
    /// w2 — the full `(N, P)` sector label. THE operator's chart.
    W2,
    /// w3 — the six-bit local word, pre-quotient.
    W3,
}

pub const LG_LADDER: [LgRung; 3] = [LgRung::W1, LgRung::W2, LgRung::W3];

/// A `(N, P)` labeller. The A2 runner passes `regplus::sector`; this crate never
/// implements one (see the module header).
pub type Labeller = fn(u8) -> (u8, [i8; 2]);

/// Turn local words into chart readings at one ladder rung.
pub fn readings_from_words(words: &[Vec<u8>], rung: LgRung, label: Labeller) -> Vec<Reading> {
    words
        .iter()
        .map(|row| {
            let mut r: Reading = Vec::with_capacity(row.len() * 3);
            for &w in row {
                match rung {
                    LgRung::W1 => r.push(label(w).0 as i64),
                    LgRung::W2 => {
                        let (nn, p) = label(w);
                        r.push(nn as i64);
                        r.push(p[0] as i64);
                        r.push(p[1] as i64);
                    }
                    LgRung::W3 => r.push(w as i64),
                }
            }
            r
        })
        .collect()
}

/// A2 gate A1 — map non-degeneracy: ≥ 8 distinct local words, else VOID (degenerate map).
/// A chart whose word never changes is `exists_closed_view` again, one level down.
pub const MIN_DISTINCT_WORDS: usize = 8;

/// A2 gate A3 — the phase-resolved defect, for door (c).
///
/// For a period `p` and residue `r`, the Leg-A defect restricted to collisions whose BOTH
/// frames satisfy `i ≡ r (mod p)`. A grain boundary exists iff some `(p, r)` reaches
/// `D_A = 0` EXACT with its work count met.
///
/// `grain.rs`'s fence governs the reading: a period belongs to the coupling that measured
/// it, never to nature and never to the engine. This function MEASURES; it does not
/// construct a `Grain`, and a `(p, r)` below the work count contributes nothing.
pub fn phase_defect(readings: &[Reading], p: usize, r: usize) -> crate::field::LegA {
    let sub: Vec<usize> = (0..readings.len().saturating_sub(1))
        .filter(|i| i % p == r)
        .collect();
    // Re-group only the selected frames, keeping each one's TRUE successor: the phase
    // selects which collisions are counted, never which frame follows which.
    let mut groups: std::collections::HashMap<&Reading, Vec<usize>> =
        std::collections::HashMap::new();
    for &i in &sub {
        groups.entry(&readings[i]).or_default().push(i);
    }
    let mut out = crate::field::LegA {
        distinct: groups.len(),
        ..Default::default()
    };
    for idx in groups.values() {
        let s = idx.len() as u128;
        if s < 2 {
            continue;
        }
        out.informative += idx.len();
        out.collisions += s * (s - 1) / 2;
        let mut by_succ: std::collections::HashMap<&Reading, u128> =
            std::collections::HashMap::new();
        for &i in idx {
            *by_succ.entry(&readings[i + 1]).or_default() += 1;
        }
        let agree: u128 = by_succ.values().map(|k| k * (k - 1) / 2).sum();
        out.firing += s * (s - 1) / 2 - agree;
    }
    out
}

/// The periods A2 §3's gate A3 sweeps.
pub const PHASE_PERIODS: [usize; 6] = [1, 2, 3, 4, 6, 8];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{leg_a, Grid, Kind};
    use crate::traj::{Frame, Header};

    fn header(n: usize, z: Vec<u32>) -> Header {
        Header {
            seed: 1,
            n_atoms: n,
            dims: 2,
            substeps: 64,
            n_frames: 0,
            dt: 1.0,
            box_w: 34.6,
            box_h: 20.8,
            box_d: 0.0,
            z,
        }
    }
    fn frame(i: u64, pos: Vec<[f64; 3]>, vel: Vec<[f64; 3]>) -> Frame {
        Frame { index: i, time: i as f64, temperature: 300.0, bonds: crate::traj::BondSet::empty(), pos, vel }
    }
    /// A labeller for the PLUMBING tests only. It is the same arithmetic `regplus::sector`
    /// performs, and the cross-check that the two agree is an integration test in
    /// `holon-mesh`, which can see both crates. Nothing in the campaign's reported numbers
    /// comes from this function: the runner passes `regplus::sector`.
    fn test_label(w: u8) -> (u8, [i8; 2]) {
        let (mut n, mut p) = (0u8, [0i8; 2]);
        for d in 0..MODES {
            if w >> d & 1 == 1 {
                n += 1;
                p[0] += DIRECTIONS_AXIAL[d][0] as i8;
                p[1] += DIRECTIONS_AXIAL[d][1] as i8;
            }
        }
        (n, p)
    }

    /// The six directions must be unit vectors 60° apart, or the mode assignment is not the
    /// hexagonal one and the isotropy warrant does not transfer.
    #[test]
    fn the_six_directions_are_the_hexagonal_unit_vectors() {
        for d in 0..MODES {
            let e = cartesian(d);
            let norm = (e[0] * e[0] + e[1] * e[1]).sqrt();
            assert!((norm - 1.0).abs() < 1e-12, "mode {d} has length {norm}");
        }
        for d in 0..MODES {
            let (a, b) = (cartesian(d), cartesian((d + 1) % MODES));
            let dot = a[0] * b[0] + a[1] * b[1];
            assert!((dot - 0.5).abs() < 1e-12, "adjacent modes are not 60° apart: {dot}");
        }
    }

    /// A velocity pointing exactly along a mode must select that mode, and the map must be
    /// onto: every mode is reachable. A map that can never emit some mode would silently
    /// shrink the chart.
    #[test]
    fn the_map_is_onto_and_fixes_its_own_directions() {
        for d in 0..MODES {
            let e = cartesian(d);
            assert_eq!(mode_of(e[0], e[1]), Some(d), "mode {d} is not its own image");
        }
        assert_eq!(mode_of(0.0, 0.0), None, "FHP-6 has no rest mode");
    }

    /// MAP-5 — must fire the exclusion fence. Carrier: six atoms in one cell all pointing
    /// along one direction. Sector: the SATURATION sector, nonzero by construction.
    /// Expected: saturation 1.0 and 5 of 6 atoms lost, both printed rather than absorbed.
    /// Without this, gate A2g could always read zero and nobody would know.
    #[test]
    fn map5_the_exclusion_fence_reports_a_real_loss() {
        let n = 6;
        let e = cartesian(0);
        let traj = Trajectory {
            header: header(n, vec![1; n]),
            frames: (0..10)
                .map(|i| {
                    frame(
                        i,
                        (0..n).map(|k| [2.0 + k as f64 * 0.1, 3.0, 0.0]).collect(),
                        vec![[e[0], e[1], 0.0]; n],
                    )
                })
                .collect(),
        };
        let (_, st) = local_words(&traj, Grid { nx: 1, ny: 1 }, Kind::Spatial, MapKind::Velocity)
            .unwrap();
        assert_eq!(st.saturation(), 1.0, "every occupied cell-frame is saturated here");
        assert!(
            (st.lost_fraction() - 5.0 / 6.0).abs() < 1e-12,
            "5 of 6 atoms must be lost to the Boolean word, got {}",
            st.lost_fraction()
        );
    }

    /// MAP-4 — must VOID. Carrier: every atom moving in one direction forever, so exactly
    /// one mode is ever occupied. Sector: the DIRECTION-DIVERSITY sector, exactly zero.
    #[test]
    fn map4_a_degenerate_map_voids_on_distinct_words() {
        let n = 6;
        let e = cartesian(2);
        let mut s: u64 = 5;
        let traj = Trajectory {
            header: header(n, vec![1; n]),
            frames: (0..400)
                .map(|i| {
                    let pos = (0..n)
                        .map(|_| {
                            s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                            let u = (((s >> 32) as u32) as f64) / (u32::MAX as f64);
                            s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                            let v = (((s >> 32) as u32) as f64) / (u32::MAX as f64);
                            [u * 34.6, v * 20.8, 0.0]
                        })
                        .collect();
                    frame(i, pos, vec![[e[0], e[1], 0.0]; n])
                })
                .collect(),
        };
        let (_, st) =
            local_words(&traj, Grid { nx: 2, ny: 2 }, Kind::Spatial, MapKind::Velocity).unwrap();
        assert!(
            st.distinct_words < MIN_DISTINCT_WORDS,
            "one direction can produce only the empty and the single-mode word, got {}",
            st.distinct_words
        );
    }

    /// MAP-2 — must NOT fire. A cyclic relabelling of the mode set is a 60° rotation: the
    /// partition of velocities into modes is identical, so the collision structure must be
    /// bit-identical at every rung. Only the `(N, P)` labels move with the frame.
    ///
    /// MAP-1 and MAP-2 are only meaningful as a pair, and the pair is asserted together in
    /// the next test.
    #[test]
    fn map2_rotating_the_mode_set_moves_nothing() {
        let traj = scene(8, 600);
        let grid = Grid { nx: 2, ny: 2 };
        let (wa, sa) = local_words(&traj, grid, Kind::Spatial, MapKind::Velocity).unwrap();
        let (wb, sb) = local_words(&traj, grid, Kind::Spatial, MapKind::Rotated).unwrap();
        assert_eq!(sa.distinct_words, sb.distinct_words);
        assert_eq!(sa.lost_atoms, sb.lost_atoms, "a rotation cannot change what is lost");
        for rung in LG_LADDER {
            let a = leg_a(&readings_from_words(&wa, rung, test_label));
            let b = leg_a(&readings_from_words(&wb, rung, test_label));
            assert_eq!(
                (a.collisions, a.firing, a.informative, a.distinct),
                (b.collisions, b.firing, b.informative, b.distinct),
                "rotating the mode set moved a reading at {rung:?}"
            );
        }
    }

    /// MAP-1 — must REJECT. Taking the mode from the arena index reads the
    /// velocity-direction sector as exactly zero. It must NOT reproduce the real map's
    /// readings; if it did, the chart would not be reading lattice-gas structure at all.
    #[test]
    fn map1_a_map_that_ignores_velocity_does_not_reproduce_the_real_one() {
        let traj = scene(8, 600);
        let grid = Grid { nx: 2, ny: 2 };
        let (wr, _) = local_words(&traj, grid, Kind::Spatial, MapKind::Velocity).unwrap();
        let (wi, _) = local_words(&traj, grid, Kind::Spatial, MapKind::ByIndex).unwrap();
        let real = leg_a(&readings_from_words(&wr, LgRung::W2, test_label));
        let idx = leg_a(&readings_from_words(&wi, LgRung::W2, test_label));
        assert_ne!(
            (real.collisions, real.firing),
            (idx.collisions, idx.firing),
            "PLANT SILENT: a map that never reads velocity produced the real map's readings"
        );
    }

    /// The A2 ladder must genuinely refine: w3 → w2 → w1. This is `sector` being a function
    /// of the word and `N` a function of the label — checked, not assumed, because
    /// `RUNG2_RESULTS.md` §5.2 showed monotone counts do not establish it.
    #[test]
    fn the_lattice_gas_ladder_refines() {
        let traj = scene(8, 600);
        let grid = Grid { nx: 2, ny: 2 };
        let (w, _) = local_words(&traj, grid, Kind::Spatial, MapKind::Velocity).unwrap();
        let r1 = readings_from_words(&w, LgRung::W1, test_label);
        let r2 = readings_from_words(&w, LgRung::W2, test_label);
        let r3 = readings_from_words(&w, LgRung::W3, test_label);
        assert!(crate::field::refines(&r2, &r1), "w2 must refine w1");
        assert!(crate::field::refines(&r3, &r2), "w3 must refine w2");
        let c: Vec<u128> = [&r1, &r2, &r3]
            .iter()
            .map(|r| leg_a(r).collisions)
            .collect();
        assert!(crate::field::ladder_monotone(&c), "collisions must not grow up the ladder");
    }

    /// A3's phase restriction must select frames without disturbing successors: at `p = 1`
    /// it must reproduce the unrestricted Leg A exactly. A phase sweep whose `p = 1` case
    /// disagreed with the main instrument would be measuring something else.
    #[test]
    fn phase_period_one_reproduces_the_unrestricted_leg() {
        let traj = scene(8, 600);
        let (w, _) =
            local_words(&traj, Grid { nx: 2, ny: 2 }, Kind::Spatial, MapKind::Velocity).unwrap();
        let r = readings_from_words(&w, LgRung::W2, test_label);
        let full = leg_a(&r);
        let p1 = phase_defect(&r, 1, 0);
        assert_eq!((full.collisions, full.firing), (p1.collisions, p1.firing));
        // And a real period must partition the work: the phases' informative counts sum to
        // no more than the whole, or the restriction is double-counting frames.
        let s: usize = (0..4).map(|r_| phase_defect(&r, 4, r_).informative).sum();
        assert!(s <= full.informative, "phase restriction double-counted: {s} > {}", full.informative);
    }

    /// A moving scene with varied velocity directions — the carrier the map plants need.
    /// It asserts its own coverage, because `RUNG2_RESULTS.md` §5.2's carrier defect was a
    /// scene that could not have failed the plants placed on it.
    fn scene(n: usize, frames: usize) -> Trajectory {
        let mut s: u64 = 20260902;
        let fs: Vec<Frame> = (0..frames)
            .map(|i| {
                let mut pos = Vec::new();
                let mut vel = Vec::new();
                for _ in 0..n {
                    s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let u = (((s >> 32) as u32) as f64) / (u32::MAX as f64);
                    s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let v = (((s >> 32) as u32) as f64) / (u32::MAX as f64);
                    s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let th = (((s >> 32) as u32) as f64) / (u32::MAX as f64)
                        * std::f64::consts::TAU;
                    pos.push([u * 34.6, v * 20.8, 0.0]);
                    vel.push([th.cos() * 1e-3, th.sin() * 1e-3, 0.0]);
                }
                frame(i as u64, pos, vel)
            })
            .collect();
        let t = Trajectory { header: header(n, vec![1; n]), frames: fs };
        let (w, st) =
            local_words(&t, Grid { nx: 2, ny: 2 }, Kind::Spatial, MapKind::Velocity).unwrap();
        assert!(
            st.distinct_words >= MIN_DISTINCT_WORDS,
            "carrier reaches only {} words; the plants on it could not fail",
            st.distinct_words
        );
        let mut hit = [false; MODES];
        for row in &w {
            for &word in row {
                for d in 0..MODES {
                    if word >> d & 1 == 1 {
                        hit[d] = true;
                    }
                }
            }
        }
        assert!(hit.iter().all(|&h| h), "carrier never reaches every mode: {hit:?}");
        t
    }
}
