//! THE BLIND PHASE CLASSIFIER, and the two plants that keep it blind.
//!
//! **Blindness is enforced by the signature, not by discipline.** `classify` takes a
//! `&Trajectory`, and `Trajectory` has no launch label in it — there is no field to read,
//! so no amount of carelessness inside `classify` can read one. The label lives in
//! [`Labelled`], a separate type that `classify` is never handed. That is the whole of
//! P-1's mechanism, and it is the mechanism M-TAG-AS-PROPERTY asks for: a verdict computed
//! from construction metadata is a lookup wearing a measurement's clothes, and the way to
//! prove it is not one is to make the metadata unreachable.
//!
//! The staked criteria, in full, so the classifier is reproducible:
//!
//! | quantity | definition | used for |
//! |---|---|---|
//! | `free_fraction` | time-averaged fraction of atoms in singleton components | VAPOR above 0.50 |
//! | `order` | time-averaged `ψ6` (2D) or `q6` (3D) over each atom's six (twelve) nearest, FLOOR-CORRECTED (below) | ICE at or above 0.45 |
//! | `mobility` | MSD at a lag of one tenth of the run, over the squared mean nearest-neighbour distance | ICE below 0.10 |
//!
//! ## What P-5 did to this classifier, in order
//!
//! P-5 FIRED three times during construction, before any real trajectory was read, and
//! each firing named a different defect. The record matters more than the final number:
//! every fix below is derived from the failure it repairs, and none of them moved the
//! 0.45 stake.
//!
//! **1. 4.0% (8/200) — no finite-N floor.** For `N` neighbours at random orientations the
//! bond-orientational parameter has a nonzero expectation purely from sampling,
//!
//! ```text
//! E[psi6^2] = E[q_l^2] = 1 / N        (independent random directions)
//! ```
//!
//! because `q_l^2 = 4π/(2l+1) · Σ_m |q_lm|^2`, each `|q_lm|^2` is the squared modulus of a
//! mean over `N` samples with `∫|Y_lm|^2 dΩ = 1`, and the `2l+1` terms sum to
//! `(2l+1)/(4πN)`. At six neighbours that floor is `1/√6 = 0.408`, just under a 0.45 bar,
//! so a twelve-atom gas clears it by luck. Fixed by reporting the floor-corrected,
//! unit-normalised form `sqrt(max(0, raw² − 1/N)) / sqrt(1 − 1/N)`, which is 0 in
//! expectation on a disordered scene and exactly 1 on a perfect lattice.
//!
//! **2. 1.6% (16/1000) — six neighbours is not a first shell.** An edge atom's outer
//! "neighbours" are the next shell, and a chance cluster of gas atoms can pass a
//! nearest-to-farthest ratio test while being ragged. Fixed by requiring the shell to be
//! COMPLETE (`STAKE_SHELL_RATIO`) and TIGHT (`STAKE_SHELL_SPREAD`), and by averaging the
//! order over interior atoms only.
//!
//! **3. 0.2% (4/2000) — one atom is not a bulk.** Every remaining firing had exactly ONE
//! distinct interior atom, its environment counted across dozens of correlated frames:
//! `interior_samples` read in the hundreds while the evidence was a single local hexagon
//! that persisted because the scene barely moves. This is the autocorrelation trap in
//! miniature — correlated samples counted as independent evidence. Fixed by
//! `STAKE_MIN_INTERIOR_ATOMS`, and by reporting `interior_atoms` beside
//! `interior_samples` so the difference is visible.
//!
//! The published number is therefore a MEASURED rate and not an inferred one: 0.2%
//! (4/2000) for the criterion, 0 of 2000 for the verdict, against the prereg's staked
//! upper bound of 1.5%. The gas fixture that exposed all three was left SLOW on purpose —
//! its mobility passes the ICE mobility clause — so the order clause has to refuse it
//! unaided.
//!
//! Branch order is VAPOR, then ICE, then LIQUID. That ordering would make P-5 vacuous on
//! its own — a vapor caught by the first branch never reaches the ICE test, so "vapor
//! never reads ICE" would be true by control flow and would measure nothing
//! (M-VACUOUS-SUCCESS). So the report also carries `ice_criterion_fired`, evaluated
//! unconditionally, and P-5 is staked on THAT: the question is whether the ICE criterion
//! itself ever fires on a gas, not whether an earlier branch got there first.

use crate::lens;
use crate::partition;
use crate::traj::Trajectory;

/// VAPOR above this free fraction.
pub const STAKE_FREE_FRACTION: f64 = 0.50;
/// ICE at or above this bond-orientational order.
pub const STAKE_ORDER: f64 = 0.45;
/// ICE below this scaled mobility.
pub const STAKE_MOBILITY: f64 = 0.10;
/// An atom's first neighbour shell is COMPLETE when its `want`-th nearest neighbour lies
/// within this multiple of its nearest. On a 2D triangular lattice an interior atom has
/// all six at `a` while an edge atom's sixth is at `sqrt(3) a = 1.73 a`; in FCC the next
/// shell is at `sqrt(2) a = 1.41 a`. 1.35 separates both, and it is a geometric fact
/// about those lattices rather than a fitted number.
pub const STAKE_SHELL_RATIO: f64 = 1.35;
/// And the shell must be a SHELL: the spread of the neighbour distances, as a fraction of
/// their mean, must not exceed this. A lattice shell is tight (exactly 0 for a perfect
/// one); a chance cluster of six gas atoms that happens to pass the ratio test above is
/// still ragged. This is the discriminator between "six neighbours" and "a first shell".
pub const STAKE_SHELL_SPREAD: f64 = 0.25;
/// And a bulk is more than one atom. Time samples of one atom's environment are the SAME
/// configuration counted many times — the trajectory moves slowly compared with the
/// sampling stride — so `interior_samples` can read in the hundreds while the evidence is
/// a single local hexagon. Measured: every false-crystal firing in a 1000-draw gas sweep
/// had exactly one distinct interior atom. This is the gate that separates them.
pub const STAKE_MIN_INTERIOR_ATOMS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Vapor,
    Liquid,
    Ice,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    Phase(Phase),
    Refused { gate: &'static str, reason: String },
}

impl Verdict {
    pub fn phase(&self) -> Option<Phase> {
        match self {
            Verdict::Phase(p) => Some(*p),
            Verdict::Refused { .. } => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Report {
    pub verdict: Verdict,
    pub free_fraction: f64,
    pub order: f64,
    pub mobility: f64,
    /// The ICE criterion evaluated UNCONDITIONALLY, whatever branch the verdict took.
    /// P-5 is staked on this, so that branch ordering cannot pass the plant for it.
    pub ice_criterion_fired: bool,
    /// How many frames were actually read. A classifier reporting a phase without saying
    /// how much trajectory it looked at is the shape M-VACUOUS-SUCCESS names.
    pub frames_read: usize,
    /// (atom, frame) samples whose first neighbour shell was COMPLETE — the only samples
    /// an order parameter means anything on. Zero of these is why the ICE branch refuses.
    pub interior_samples: usize,
    /// DISTINCT atoms that ever presented a complete shell. Samples are time-correlated,
    /// so a large `interior_samples` over two atoms is one configuration counted many
    /// times; this is the count that says how much of the scene is bulk.
    pub interior_atoms: usize,
}

/// A trajectory beside the preset it was launched under.
///
/// The label is PRIVATE and there is no accessor that hands it to a classifier: the only
/// way out is [`Labelled::declared`], which exists for a test to assert what the plant
/// planted. `trajectory()` is what a classifier is given, and its type carries no label.
pub struct Labelled {
    label: String,
    traj: Trajectory,
}

impl Labelled {
    pub fn new(label: impl Into<String>, traj: Trajectory) -> Self {
        Self {
            label: label.into(),
            traj,
        }
    }
    /// What a classifier receives. Note the type: there is no label in it.
    pub fn trajectory(&self) -> &Trajectory {
        &self.traj
    }
    /// What the LAUNCH claimed. For a plant to state its own construction, never for a
    /// verdict to consult.
    pub fn declared(&self) -> &str {
        &self.label
    }
}

/// Strip the sampling floor from a bond-orientational reading.
///
/// `E[raw²] = 1/n` for `n` neighbours at random orientations, so the corrected quantity is
/// zero in expectation on a disordered scene and one on a perfect lattice. See the module
/// header for the derivation and for the plant that forced it.
fn floor_corrected(raw: f64, n: usize) -> f64 {
    if n < 2 {
        return 0.0;
    }
    let floor = 1.0 / n as f64;
    ((raw * raw - floor).max(0.0) / (1.0 - floor)).sqrt()
}

/// The classifier. Its whole input is the trajectory.
pub fn classify(traj: &Trajectory) -> Report {
    let n = traj.header.n_atoms;
    let nf = traj.frames.len();
    if nf < 2 || n < 4 {
        return Report {
            verdict: Verdict::Refused {
                gate: "n_frames >= 2 and n_atoms >= 4",
                reason: format!(
                    "{nf} frames and {n} atoms; neither an order parameter nor a mobility \
                     is defined on this scene"
                ),
            },
            free_fraction: 0.0,
            order: 0.0,
            mobility: 0.0,
            ice_criterion_fired: false,
            frames_read: nf,
            interior_samples: 0,
            interior_atoms: 0,
        };
    }

    // --- free fraction: atoms in singleton components, averaged over frames -----------
    let mut free = 0usize;
    for f in &traj.frames {
        let labels = partition::labels_from_bonds(n, &f.bonds);
        for b in partition::blocks(&labels) {
            if b.popcount() == 1 {
                free += 1;
            }
        }
    }
    let free_fraction = free as f64 / (n * nf) as f64;

    // --- order: the dimension-appropriate bond-orientational parameter ---------------
    // Sampled on a stride so a 20,000-frame run costs the same as a 200-frame one; the
    // stride is stated rather than tuned.
    let stride = (nf / 200).max(1);
    let want = if traj.header.dims == 2 { 6 } else { 12 };
    let (mut osum, mut ocount) = (0.0f64, 0usize);
    let mut interior_seen = vec![false; n];
    for f in traj.frames.iter().step_by(stride) {
        for i in 0..n {
            let nb_idx = lens::k_nearest(&f.pos, i, want);
            if nb_idx.len() < want {
                continue;
            }
            let nb: Vec<[f64; 3]> = nb_idx.iter().map(|&j| f.pos[j]).collect();
            // A bond-orientational parameter is a statement about a COMPLETE shell. On an
            // edge atom the outer "neighbours" are the next shell, and the reading is a
            // fact about the boundary rather than about the phase — the same shape as
            // M-MAINTENANCE-LENS, one dimension down. Such atoms are skipped, and the
            // count of what survived is reported.
            let d: Vec<f64> = nb.iter().map(|&q| dist(f.pos[i], q)).collect();
            let (d_near, d_far) = (d[0], d[want - 1]);
            if d_near <= 0.0 || d_far > STAKE_SHELL_RATIO * d_near {
                continue;
            }
            let mean = d.iter().sum::<f64>() / want as f64;
            if mean <= 0.0 || (d_far - d_near) / mean > STAKE_SHELL_SPREAD {
                continue;
            }
            let r = if traj.header.dims == 2 {
                lens::hexatic_psi6(2, f.pos[i], &nb)
            } else {
                lens::steinhardt_q(6, 3, f.pos[i], &nb)
            };
            if let Ok(v) = r {
                osum += floor_corrected(v, want);
                ocount += 1;
                interior_seen[i] = true;
            }
        }
    }
    let order = if ocount == 0 { 0.0 } else { osum / ocount as f64 };
    let interior_samples = ocount;
    let interior_atoms = interior_seen.iter().filter(|b| **b).count();

    // --- mobility: displacement over a lag, in units of the neighbour spacing ---------
    let lag = (nf / 10).max(1);
    let d2 = lens::msd(traj, lag);
    let mut spacing = 0.0f64;
    let f0 = &traj.frames[0];
    for i in 0..n {
        if let Some(&j) = lens::k_nearest(&f0.pos, i, 1).first() {
            let d = [
                f0.pos[j][0] - f0.pos[i][0],
                f0.pos[j][1] - f0.pos[i][1],
                f0.pos[j][2] - f0.pos[i][2],
            ];
            spacing += (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        }
    }
    spacing /= n as f64;
    let mobility = if spacing <= 0.0 {
        0.0
    } else {
        d2 / (spacing * spacing)
    };

    // The ICE criterion, evaluated whatever the branch order does with it.
    let has_bulk = interior_atoms >= STAKE_MIN_INTERIOR_ATOMS;
    let ice_criterion_fired = has_bulk && order >= STAKE_ORDER && mobility < STAKE_MOBILITY;

    // A scene with NO complete neighbour shell anywhere cannot be asked whether it is a
    // crystal. It can still be asked whether it is a gas (a free fraction needs no shell)
    // and whether it is flowing (a mobility needs no shell) — so only the case that turns
    // on order REFUSES, and it names the gate that would lift the refusal.
    let verdict = if free_fraction > STAKE_FREE_FRACTION {
        Verdict::Phase(Phase::Vapor)
    } else if !has_bulk && mobility < STAKE_MOBILITY {
        Verdict::Refused {
            gate: "at least two atoms with a complete first neighbour shell",
            reason: format!(
                "{n} atoms, of which {interior_atoms} ever closed a first shell of {want} \
                 neighbours (needed: {STAKE_MIN_INTERIOR_ATOMS}). Below that the order \
                 parameter is one atom's environment counted across correlated frames, not \
                 a statement about the scene, and it cannot separate ICE from LIQUID here. \
                 The gate that lifts this is a scene with a bulk."
            ),
        }
    } else if ice_criterion_fired {
        Verdict::Phase(Phase::Ice)
    } else {
        Verdict::Phase(Phase::Liquid)
    };

    Report {
        verdict,
        free_fraction,
        order,
        mobility,
        ice_criterion_fired,
        frames_read: nf,
        interior_samples,
        interior_atoms,
    }
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synthetic::{self, Spec};

    fn spec(seed: u64, n_frames: usize) -> Spec {
        let mut s = Spec::quench_like(n_frames, vec![8, 8, 8, 8, 1, 1, 1, 1, 1, 1, 1, 1]);
        s.seed = seed;
        s
    }

    /// A crystal with a BULK reads ICE. Sixteen atoms in a 4x4 triangular patch is the
    /// smallest 2D scene with an interior atom, and the trajectory format's 16-atom cap is
    /// exactly that size — so this is the only crystal this format can certify at all.
    #[test]
    fn a_crystal_with_a_bulk_reads_ice() {
        let mut s = Spec::quench_like(400, vec![8; 16]);
        s.seed = 1;
        let t = synthetic::crystal(s, 0.02);
        let r = classify(&t);
        assert!(r.interior_samples > 0, "the fixture must have a bulk: {r:?}");
        assert_eq!(r.verdict.phase(), Some(Phase::Ice), "{r:?}");
    }

    /// Twelve atoms in a 4x3 triangular patch DO have a bulk — the middle row's two
    /// interior atoms — and so the scene is decidable. This test was first written the
    /// other way round, asserting a refusal, and the instrument corrected it: three rows
    /// is enough for a complete shell even though four columns is not enough for two.
    #[test]
    fn a_twelve_atom_crystal_has_a_thin_bulk_and_is_decidable() {
        let t = synthetic::crystal(spec(1, 400), 0.02);
        let r = classify(&t);
        assert!(r.interior_samples > 0, "the middle row is interior");
        assert_eq!(r.verdict.phase(), Some(Phase::Ice), "{r:?}");
    }

    /// A scene with NO complete shell anywhere is refused rather than guessed. A bonded
    /// chain is the clean case: a chain atom's sixth-nearest neighbour is three spacings
    /// away, so no shell closes, and an order parameter would be a reading about the
    /// chain's ends.
    #[test]
    fn a_scene_with_no_complete_shell_is_refused() {
        let mut sp = Spec::quench_like(400, vec![8; 12]);
        sp.seed = 7;
        let n = 12usize;
        let all = crate::partition::Mask::all(n);
        let t = synthetic::build(sp, move |_t, pos, vel| {
            for i in 0..n {
                pos[i] = [2.0 + 2.6 * i as f64, 5.0, 0.0];
                vel[i] = [0.0; 3];
            }
            synthetic::bonds_from_blocks(n, std::slice::from_ref(&all))
        });
        let r = classify(&t);
        assert_eq!(r.interior_samples, 0, "a chain closes no shell");
        match &r.verdict {
            Verdict::Refused { gate, .. } => {
                assert_eq!(*gate, "at least two atoms with a complete first neighbour shell")
            }
            v => panic!("guessing {v:?} on a scene with no bulk is the error"),
        }
    }

    #[test]
    fn a_gas_reads_vapor() {
        let t = synthetic::vapor(spec(2, 400));
        assert_eq!(classify(&t).verdict.phase(), Some(Phase::Vapor));
    }

    #[test]
    fn a_bonded_mobile_scene_reads_liquid() {
        let t = synthetic::liquid(spec(3, 400));
        let r = classify(&t);
        assert_eq!(r.verdict.phase(), Some(Phase::Liquid), "{r:?}");
    }

    #[test]
    fn a_scene_too_small_to_read_is_refused() {
        let t = synthetic::vapor(Spec::quench_like(1, vec![1, 1]));
        assert!(matches!(classify(&t).verdict, Verdict::Refused { .. }));
    }
}
