//! The torus, the motion, and the ledger.
//!
//! `T = S ∘ C`: collide in place, then stream. One application is one step and it is the
//! tier's OWN clock — there is no molecular clock anywhere in this crate.
//!
//! # The ledger has no epsilon
//!
//! Mass and both momentum components are integers and are gated by IDENTITY across steps,
//! never by a residual under a tolerance. That is the tier's structural difference from a
//! floating-point molecular tier and it is the reason "exact" is used literally below.
//!
//! # The wall is a term in the ledger, never a tolerance
//!
//! Bounce-back reverses a particle, so it changes the fluid's momentum by `−2·DIR[d]`. A
//! channel present in the dynamics and absent from the ledger reads as unexplained loss, so
//! the impulse is ACCUMULATED and the gate is `P(t) = P(0) + impulse(t)`, exactly. Mass is
//! untouched by the wall and is gated separately (one gate per conservation law).

use crate::state::Model;

/// The scene's conserved totals at one instant, plus the cumulative wall impulse.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ledger {
    pub mass: i64,
    pub momentum: [i64; 2],
    /// Cumulative change in fluid momentum caused by bounce-back, so that
    /// `momentum(t) == momentum(0) + wall_impulse(t)` is an integer identity.
    pub wall_impulse: [i64; 2],
}

/// What one application of the motion did, beyond moving the state.
///
/// The impulse is here and not in a tolerance: bounce-back reverses a particle, so it
/// changes the fluid's momentum by `−2·DIR[d]`, and a channel present in the dynamics and
/// absent from the ledger reads as unexplained loss.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepStats {
    pub collisions_fired: u64,
    pub wall_impulse: [i64; 2],
}

/// An `L × L` periodic lattice of local states, with an optional solid mask.
#[derive(Clone, Debug)]
pub struct Lattice {
    pub model: Model,
    pub l: usize,
    pub cells: Vec<u8>,
    /// Solid cells reverse every particle that reaches them. Empty means no obstacle.
    pub solid: Vec<bool>,
    collision: Vec<u8>,
    /// When set, the `±60°` sense is chosen per cell and step by a counter hash, so the run
    /// is reproducible bitwise and depends on no traversal order and no sequential state.
    /// Both tables are built once; rebuilding one per cell would make the step allocate.
    chirality_pair: Option<(u64, [Vec<u8>; 2])>,
    step_index: u64,
    scratch: Vec<u8>,
    /// `neighbour[c * n_dirs + d]` — the cell a particle at `c` moving in direction `d`
    /// reaches. Precomputed because the torus wrap makes it a non-uniform offset, and
    /// recomputing it per cell per step is most of the step's cost.
    neighbour: Vec<u32>,
    /// The conserved label per local state, precomputed for the same reason.
    labels: Vec<(u32, i64, i64)>,
    /// Cells whose local state was actually changed by the collision, this step. A gate that
    /// reports PASS on zero work has not passed (M-VACUOUS-SUCCESS).
    pub collisions_fired: u64,
    pub wall_impulse: [i64; 2],
    /// Hoisted out of the step's inner loop: with no wall and no chirality hash the
    /// collision is one table lookup per cell, and checking for a wall that is not there
    /// on every cell of every step is most of what the general path costs.
    has_wall: bool,
}

impl Lattice {
    /// Seed from a counter hash of the global cell index, so the scene is reproducible and
    /// depends on nothing but `(l, seed)`. `density` in `[0,1]` is the per-direction
    /// occupancy probability.
    pub fn seeded(model: Model, l: usize, seed: u64, density: f64, collision: Vec<u8>) -> Self {
        assert!(l >= 2 && (0.0..=1.0).contains(&density));
        assert_eq!(collision.len(), model.n_states());
        let n = model.n_dirs();
        let cells = (0..l * l)
            .map(|c| {
                let mut s = 0u8;
                for d in 0..n {
                    let h = mix64(seed ^ (c as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ ((d as u64) << 56));
                    if (h >> 11) as f64 / ((1u64 << 53) as f64) < density {
                        s |= 1 << d;
                    }
                }
                s
            })
            .collect();
        Self::from_cells(model, l, cells, collision)
    }

    /// The tables every lattice needs, around a state that has already been seeded. Split out
    /// of [`Lattice::seeded`] verbatim so the shear seeding below builds the SAME object;
    /// `seeded`'s behaviour is unchanged, byte for byte.
    fn from_cells(model: Model, l: usize, cells: Vec<u8>, collision: Vec<u8>) -> Self {
        assert!(l >= 2);
        assert_eq!(collision.len(), model.n_states());
        assert_eq!(cells.len(), l * l);
        let n = model.n_dirs();
        let mut neighbour = vec![0u32; l * l * n];
        for i in 0..l {
            for j in 0..l {
                for d in 0..n {
                    let ii = (i as i64 + model.dirs[d][0]).rem_euclid(l as i64) as usize;
                    let jj = (j as i64 + model.dirs[d][1]).rem_euclid(l as i64) as usize;
                    neighbour[(i * l + j) * n + d] = (ii * l + jj) as u32;
                }
            }
        }
        let labels = (0..model.n_states() as u8).map(|s| model.label(s)).collect();
        Self {
            l,
            cells,
            neighbour,
            labels,
            solid: vec![false; l * l],
            collision,
            chirality_pair: None,
            step_index: 0,
            scratch: vec![0; l * l],
            collisions_fired: 0,
            wall_impulse: [0, 0],
            has_wall: false,
            model,
        }
    }

    /// Choose the head-on rotation's sense per cell and step. Deterministic FHP-I breaks
    /// chirality; the certificate is read on the randomized variant and the deterministic
    /// one's chirality defect is reported as a named artifact, never as physics.
    pub fn randomize_chirality(&mut self, seed: u64) -> &mut Self {
        self.chirality_pair = Some((seed, [self.model.fhp_i(false), self.model.fhp_i(true)]));
        self
    }

    /// A lattice seeded with a transverse momentum wave — FLUID-0's shear initial condition.
    ///
    /// Each of a cell's directions is seeded independently with occupation probability
    /// `p_dir = clamp(density + amplitude · c_dir·x̂ · sin(2π k_index j / l), 0, 1)`, where
    /// `c_dir` is the Euclidean embedding of direction `dir` ([`crate::isotropy::embed`]) and
    /// `j = c % l` is the cell's row index — the SAME index [`Lattice::line_momenta`] sums the
    /// x-momenta over. Because the hexagon has `Σ_d c_dx c_dx = 3` and `Σ_d c_dx c_dy = 0`,
    /// the seeded momentum density is exactly `3·A·sin(...)` along `x̂` and zero along `ŷ`:
    /// a pure shear wave, transverse to a wavevector that lies along `ŷ`.
    ///
    /// The per-direction hash is [`Lattice::seeded`]'s, with `p_dir` in place of `density`.
    pub fn seeded_shear(
        model: Model,
        l: usize,
        seed: u64,
        density: f64,
        amplitude: f64,
        k_index: usize,
        collision: Vec<u8>,
    ) -> Self {
        Self::seeded_shear_drive(model, l, seed, density, amplitude, k_index, collision, [1.0, 0.0], false)
    }

    /// [`Lattice::seeded_shear`] with the momentum direction and the modulated axis named.
    ///
    /// `drive` is the Euclidean direction the seeded momentum points along; the modulated
    /// index is the row `j = c % l` when `along_columns` is false and the column `i = c / l`
    /// when it is true. The row wave has its wavevector along `ŷ`, the column wave has its
    /// wavevector along `(√3/2, −1/2)`; both have `|k| = 2π·k_index / (l·√3/2)`, because the
    /// perpendicular spacing of the constant-`i` lines is `√3/2` links exactly as the
    /// constant-`j` lines' is (the two families are a 60° rotation apart).
    #[allow(clippy::too_many_arguments)]
    pub fn seeded_shear_drive(
        model: Model,
        l: usize,
        seed: u64,
        density: f64,
        amplitude: f64,
        k_index: usize,
        collision: Vec<u8>,
        drive: [f64; 2],
        along_columns: bool,
    ) -> Self {
        assert!(l >= 2 && (0.0..=1.0).contains(&density));
        assert_eq!(collision.len(), model.n_states());
        let n = model.n_dirs();
        let proj: Vec<f64> = crate::isotropy::embed(&model)
            .iter()
            .map(|v| v[0] * drive[0] + v[1] * drive[1])
            .collect();
        let two_pi_k = 2.0 * core::f64::consts::PI * k_index as f64 / l as f64;
        let wave: Vec<f64> = (0..l).map(|m| (two_pi_k * m as f64).sin()).collect();
        let cells = (0..l * l)
            .map(|c| {
                let m = if along_columns { c / l } else { c % l };
                let w = wave[m];
                let mut s = 0u8;
                for (d, &pd) in proj.iter().enumerate().take(n) {
                    let p = (density + amplitude * pd * w).clamp(0.0, 1.0);
                    let h = mix64(seed ^ (c as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ ((d as u64) << 56));
                    if (h >> 11) as f64 / ((1u64 << 53) as f64) < p {
                        s |= 1 << d;
                    }
                }
                s
            })
            .collect();
        Self::from_cells(model, l, cells, collision)
    }

    /// A colour bit-plane over this lattice's occupation: bit `d` set means the particle in
    /// direction `d` is red. Each occupied particle is red with probability
    /// `½ + (amplitude/2)·sin(2π k_index j / l)` — a colour wave on a fluid at rest, with the
    /// same row index `j = c % l` the momentum readout uses.
    ///
    /// The hash is keyed away from [`Lattice::seeded`]'s so the colour draw is independent of
    /// the occupation draw at the same cell and direction.
    pub fn seed_colour_wave(&self, seed: u64, amplitude: f64, k_index: usize) -> Vec<u8> {
        let n = self.model.n_dirs();
        let two_pi_k = 2.0 * core::f64::consts::PI * k_index as f64 / self.l as f64;
        let wave: Vec<f64> = (0..self.l).map(|m| (two_pi_k * m as f64).sin()).collect();
        self.cells
            .iter()
            .enumerate()
            .map(|(c, &s)| {
                let q = (0.5 + 0.5 * amplitude * wave[c % self.l]).clamp(0.0, 1.0);
                let mut r = 0u8;
                for d in 0..n {
                    if s >> d & 1 == 0 {
                        continue;
                    }
                    let h = mix64(
                        COLOUR_SEED_KEY
                            ^ seed
                            ^ (c as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                            ^ ((d as u64) << 56),
                    );
                    if (h >> 11) as f64 / ((1u64 << 53) as f64) < q {
                        r |= 1 << d;
                    }
                }
                r
            })
            .collect()
    }

    /// Mark a straight run of `len` cells starting at `(i0, j0)` as solid — the structurally
    /// inhomogeneous graph M-HOMOG demands, since a periodic torus is homogeneous and a
    /// locality-shaped result on one may hold for a homogeneity reason instead.
    pub fn add_wall(&mut self, i0: usize, j0: usize, len: usize) -> &mut Self {
        for k in 0..len {
            self.solid[((i0 + k) % self.l) * self.l + j0 % self.l] = true;
        }
        self.has_wall = true;
        self
    }

    #[inline]
    fn idx(&self, i: usize, j: usize) -> usize {
        i * self.l + j
    }

    /// The conserved totals right now. Solid cells hold particles and are counted: a
    /// bounced particle still exists, and pretending otherwise would make the wall look
    /// like a mass sink.
    pub fn ledger(&self) -> Ledger {
        self.ledger_of(&self.cells)
    }

    /// [`Lattice::ledger`] on a SUPPLIED state, for a caller that advances borrowed buffers
    /// rather than `self.cells` (the transport readouts do). One implementation of the sum.
    pub fn ledger_of(&self, cells: &[u8]) -> Ledger {
        let (mut mass, mut px, mut py) = (0i64, 0i64, 0i64);
        for &s in cells {
            let (n, x, y) = self.labels[s as usize];
            mass += n as i64;
            px += x;
            py += y;
        }
        Ledger { mass, momentum: [px, py], wall_impulse: self.wall_impulse }
    }

    /// One step of the tier's own motion — collide, then stream — on a SUPPLIED state.
    ///
    /// `cells` is collided in place and the streamed result is written to `out`; the caller
    /// swaps. Borrowing rather than owning is what lets the closure probe advance two
    /// micro-states without cloning the lattice's tables 160,000 times.
    pub fn advance(&self, cells: &mut [u8], out: &mut [u8], step_index: u64) -> StepStats {
        let n = self.model.n_dirs();
        let mut stats = StepStats::default();
        let nb = self.neighbour.as_slice();

        // --- C, in place.
        match (&self.chirality_pair, self.has_wall) {
            (None, false) => {
                let col = self.collision.as_slice();
                let mut fired = 0u64;
                for s in cells.iter_mut() {
                    let t = col[*s as usize];
                    fired += u64::from(t != *s);
                    *s = t;
                }
                stats.collisions_fired = fired;
            }
            _ => {
                for c in 0..cells.len() {
                    let s = cells[c];
                    if self.solid[c] {
                        let mut r = 0u8;
                        for d in 0..n {
                            if s >> d & 1 == 1 {
                                let o = self.model.opposite(d);
                                r |= 1 << o;
                                stats.wall_impulse[0] +=
                                    self.model.dirs[o][0] - self.model.dirs[d][0];
                                stats.wall_impulse[1] +=
                                    self.model.dirs[o][1] - self.model.dirs[d][1];
                            }
                        }
                        cells[c] = r;
                        continue;
                    }
                    let t = match &self.chirality_pair {
                        None => self.collision[s as usize],
                        Some((k, alt)) => {
                            let h = mix64(
                                k ^ (c as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                                    ^ step_index.rotate_left(32),
                            );
                            alt[(h & 1) as usize][s as usize]
                        }
                    };
                    stats.collisions_fired += u64::from(t != s);
                    cells[c] = t;
                }
            }
        }

        // --- S. Each particle moves exactly one cell in its own direction.
        out.fill(0);
        for (c, &s) in cells.iter().enumerate() {
            if s == 0 {
                continue;
            }
            let base = c * n;
            for d in 0..n {
                if s >> d & 1 == 1 {
                    out[nb[base + d] as usize] |= 1 << d;
                }
            }
        }
        stats
    }

    /// One step of this lattice's own state, folding the step's stats into the run's.
    pub fn step(&mut self) {
        let mut cells = core::mem::take(&mut self.cells);
        let mut scratch = core::mem::take(&mut self.scratch);
        let stats = self.advance(&mut cells, &mut scratch, self.step_index);
        core::mem::swap(&mut cells, &mut scratch);
        self.cells = cells;
        self.scratch = scratch;
        self.collisions_fired = stats.collisions_fired;
        self.wall_impulse[0] += stats.wall_impulse[0];
        self.wall_impulse[1] += stats.wall_impulse[1];
        self.step_index += 1;
    }

    /// One step carrying a COLOUR bit-plane beside the occupation — the two-species step
    /// FLUID-0's tracer readout needs, with the colour-blind rule of d'Humières, Lallemand &
    /// Searby (1987).
    ///
    /// The occupation half is [`Lattice::advance`] verbatim: the same table, the same
    /// chirality hash, the same wall. With an all-zero colour plane `out` is bit-identical to
    /// what `advance` writes, and [`tests::the_colour_step_reduces_to_advance`] measures that
    /// rather than asserting it. The colour half:
    ///
    /// * a collision that leaves the state unchanged leaves the colour unchanged;
    /// * a FIRING collision conserves the number of red particles and reassigns the red bits
    ///   uniformly at random among the outgoing occupied directions, drawn from a counter hash
    ///   of `(cell, step)` alone — so the result depends on no traversal order and is
    ///   reproducible bitwise;
    /// * streaming moves a colour bit with its occupation bit;
    /// * a solid cell reverses both.
    ///
    /// Colour bits are always a subset of occupation bits, and stay one.
    pub fn advance_with_colour(
        &self,
        cells: &mut [u8],
        colour: &mut [u8],
        out: &mut [u8],
        out_colour: &mut [u8],
        step_index: u64,
    ) -> StepStats {
        self.advance_with_colour_rule(cells, colour, out, out_colour, step_index, ColourRule::Blind)
    }

    /// [`Lattice::advance_with_colour`] with the reassignment rule named — the plant handle.
    pub fn advance_with_colour_rule(
        &self,
        cells: &mut [u8],
        colour: &mut [u8],
        out: &mut [u8],
        out_colour: &mut [u8],
        step_index: u64,
        rule: ColourRule,
    ) -> StepStats {
        let n = self.model.n_dirs();
        let mut stats = StepStats::default();
        let nb = self.neighbour.as_slice();

        // --- C, in place, on both planes.
        for c in 0..cells.len() {
            let s = cells[c];
            if self.has_wall && self.solid[c] {
                let (mut r, mut rq) = (0u8, 0u8);
                let q = colour[c];
                for d in 0..n {
                    if s >> d & 1 == 1 {
                        let o = self.model.opposite(d);
                        r |= 1 << o;
                        if q >> d & 1 == 1 {
                            rq |= 1 << o;
                        }
                        stats.wall_impulse[0] += self.model.dirs[o][0] - self.model.dirs[d][0];
                        stats.wall_impulse[1] += self.model.dirs[o][1] - self.model.dirs[d][1];
                    }
                }
                cells[c] = r;
                colour[c] = rq;
                continue;
            }
            let t = match &self.chirality_pair {
                None => self.collision[s as usize],
                Some((k, alt)) => {
                    let h = mix64(
                        k ^ (c as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                            ^ step_index.rotate_left(32),
                    );
                    alt[(h & 1) as usize][s as usize]
                }
            };
            if t != s {
                stats.collisions_fired += 1;
                colour[c] = recolour(s, t, colour[c], n, rule, c, step_index);
            }
            cells[c] = t;
        }

        // --- S. Each particle moves one cell in its own direction, carrying its colour.
        out.fill(0);
        out_colour.fill(0);
        for (c, &s) in cells.iter().enumerate() {
            if s == 0 {
                continue;
            }
            let q = colour[c];
            let base = c * n;
            for d in 0..n {
                if s >> d & 1 == 1 {
                    let dst = nb[base + d] as usize;
                    out[dst] |= 1 << d;
                    if q >> d & 1 == 1 {
                        out_colour[dst] |= 1 << d;
                    }
                }
            }
        }
        stats
    }

    /// Stream backwards. Present so that "the motion is a bijection" is a measurement on the
    /// full micro-state rather than an inference from the collision table alone.
    pub fn unstream(&self, cells: &[u8]) -> Vec<u8> {
        let n = self.model.n_dirs();
        let mut out = vec![0u8; cells.len()];
        for i in 0..self.l {
            for j in 0..self.l {
                let s = cells[self.idx(i, j)];
                for d in 0..n {
                    if s >> d & 1 == 1 {
                        let ii = (i as i64 - self.model.dirs[d][0]).rem_euclid(self.l as i64) as usize;
                        let jj = (j as i64 - self.model.dirs[d][1]).rem_euclid(self.l as i64) as usize;
                        out[self.idx(ii, jj)] |= 1 << d;
                    }
                }
            }
        }
        out
    }

    /// Momentum summed along each lattice line, a chart `L` times finer than global. HPP-4
    /// holds it EXACTLY — its spurious invariant, and historically the whole reason FHP
    /// exists; FHP-6 does not. The Leg-A gauge lives here.
    ///
    /// **The line a component is summed along is the one that component's movers do not
    /// leave.** An `x`-mover changes `i` and keeps `j`, so `Px` is summed over constant `j`;
    /// `Py` over constant `i`. Summing each component along its OWN axis instead reads a
    /// quantity nothing conserves, and does so on HPP as loudly as on FHP — the gauge would
    /// then have no sides at all. (It did, until this line was written the other way round.)
    pub fn line_momenta(&self) -> Vec<i64> {
        self.line_momenta_of(&self.cells)
    }

    /// [`Lattice::line_momenta`] on a SUPPLIED state. `out[j]` for `j < l` is the x-momentum
    /// summed over the row `j`; `out[l + i]` is the y-momentum summed over the column `i`.
    pub fn line_momenta_of(&self, cells: &[u8]) -> Vec<i64> {
        let mut out = vec![0i64; 2 * self.l];
        for i in 0..self.l {
            for j in 0..self.l {
                let (_, x, y) = self.labels[cells[self.idx(i, j)] as usize];
                out[j] += x;
                out[self.l + i] += y;
            }
        }
        out
    }

    /// Hamming distance in occupied directions between two micro-states — the
    /// carrier-motion counter that keeps a closure reading from being taken on a fixed
    /// point (M-FIXED-POINT-TRAJECTORY).
    pub fn occupancy_distance(a: &[u8], b: &[u8]) -> u64 {
        a.iter().zip(b).map(|(&x, &y)| (x ^ y).count_ones() as u64).sum()
    }

    /// The cell a particle at `cell` moving in direction `dir` reaches — the precomputed
    /// torus wrap, READ from the same table [`Lattice::advance`] streams on.
    ///
    /// Added for `crate::orientation`, which carries a second state beside this one and must
    /// stream on the SAME wrap rather than a second copy of the arithmetic (a second copy is
    /// how two objects that must agree come to disagree). Additive: no existing path changes.
    #[inline]
    pub fn neighbour_of(&self, cell: usize, dir: usize) -> usize {
        self.neighbour[cell * self.model.n_dirs() + dir] as usize
    }

    /// The image `[`Lattice::advance`]`'s collision phase would write at `cell`, for the
    /// state `s` at `step_index` — the same table and the SAME chirality hash, in one
    /// expression, so a state beside this one collides identically.
    ///
    /// Solid cells are not handled here: this is the collision, not the wall.
    /// `crate::orientation` refuses a walled lattice at construction, and
    /// `orientation::tests::the_no_bond_path_is_advance_with_colour` measures the agreement
    /// over 500 steps rather than asserting it. Additive: no existing path changes.
    #[inline]
    pub fn collision_image(&self, cell: usize, s: u8, step_index: u64) -> u8 {
        match &self.chirality_pair {
            None => self.collision[s as usize],
            Some((k, alt)) => {
                let h = mix64(
                    k ^ (cell as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                        ^ step_index.rotate_left(32),
                );
                alt[(h & 1) as usize][s as usize]
            }
        }
    }
}

/// How the colour plane is reassigned at a FIRING collision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColourRule {
    /// The colour-blind rule (d'Humières, Lallemand & Searby 1987; Rothman & Zaleski 1997):
    /// the red bits are redistributed uniformly at random among the outgoing occupied
    /// directions, their number conserved. This is the physics FLUID-0 measures.
    Blind,
    /// `FLUID0_PREREG` §5 plant (i), the colour-blind rule REMOVED: the `k`-th occupied
    /// outgoing direction takes the colour of the `k`-th occupied incoming direction, both in
    /// ascending direction-index order. Deterministic — the colour stays with its direction
    /// index — and the red count is still conserved exactly, so the plant moves the tracer's
    /// mixing and nothing else. Never physics; only the plant uses it.
    Ordinal,
}

/// Keys that hold the three counter-hash streams apart: occupation seeding uses the bare
/// seed, colour seeding `COLOUR_SEED_KEY`, and the collision's colour draw `COLOUR_MIX_KEY`.
/// Sharing one stream would correlate a particle's colour with its own existence.
const COLOUR_SEED_KEY: u64 = 0x436F_6C6F_7572_5364;
const COLOUR_MIX_KEY: u64 = 0x436F_6C6F_7572_4D78;

/// The colour plane's image of one firing collision `s → t`, red count conserved.
///
/// `pub` so `crate::orientation` carries the colour by the SAME rule rather than a second
/// copy of it — FLUID-1's no-bond control must be bit-identical to FLUID-0's colour step on
/// BOTH planes, and two spellings of one rule is how that identity would be lost. Making a
/// private function public changes no existing behaviour.
pub fn recolour(s: u8, t: u8, q: u8, n: usize, rule: ColourRule, c: usize, step_index: u64) -> u8 {
    let red = q.count_ones() as usize;
    if red == 0 {
        return 0;
    }
    let mut outs = [0u8; 8];
    let mut m = 0usize;
    for d in 0..n {
        if t >> d & 1 == 1 {
            outs[m] = d as u8;
            m += 1;
        }
    }
    // A sector-preserving law conserves occupancy, so `red <= m`; `red == m` colours the lot.
    if red >= m {
        return t;
    }
    match rule {
        ColourRule::Ordinal => {
            let mut ins = [0u8; 8];
            let mut p = 0usize;
            for d in 0..n {
                if s >> d & 1 == 1 {
                    ins[p] = d as u8;
                    p += 1;
                }
            }
            let mut r = 0u8;
            for k in 0..p.min(m) {
                if q >> ins[k] & 1 == 1 {
                    r |= 1 << outs[k];
                }
            }
            r
        }
        ColourRule::Blind => {
            let base = mix64(
                COLOUR_MIX_KEY
                    ^ (c as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                    ^ step_index.rotate_left(32),
            );
            // Partial Fisher–Yates over the outgoing occupied directions: the first `red`
            // entries of a uniformly shuffled list are a uniform `red`-subset.
            let mut perm = outs;
            for k in 0..red {
                let h = mix64(base ^ ((k as u64) << 40) ^ 0x1234_5678_9ABC_DEF0);
                let j = k + (h % (m - k) as u64) as usize;
                perm.swap(k, j);
            }
            perm[..red].iter().fold(0u8, |r, &d| r | 1 << d)
        }
    }
}

/// SplitMix64. Local rather than imported: this crate's dependency profile is one crate, and
/// the constant is a published one, not a shared implementation whose drift could matter.
#[inline]
pub fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fhp(l: usize) -> Lattice {
        let m = Model::fhp6();
        let c = m.fhp_i(true);
        Lattice::seeded(m, l, 0xC1A5, 0.35, c)
    }

    /// G1–G3: each conservation law gated SEPARATELY, as an integer identity across steps —
    /// per-carrier invariance, never equality between two different carriers
    /// (M-NULL-MISSTAKE). And the work count is asserted: a run whose collisions never fire
    /// conserves trivially (M-VACUOUS-SUCCESS).
    #[test]
    fn mass_and_each_momentum_component_are_exactly_invariant() {
        let mut g = fhp(48);
        let l0 = g.ledger();
        let mut fired = 0u64;
        for t in 0..400 {
            g.step();
            fired += g.collisions_fired;
            let l = g.ledger();
            assert_eq!(l.mass, l0.mass, "mass moved at step {t}");
            assert_eq!(l.momentum[0], l0.momentum[0], "momentum-x moved at step {t}");
            assert_eq!(l.momentum[1], l0.momentum[1], "momentum-y moved at step {t}");
        }
        assert!(fired > 0, "no collision ever fired: the gates passed on work not done");
    }

    /// The three isolating plants of LG_PREREG §7. Each moves exactly ONE conserved
    /// quantity, so each fires exactly one gate; a plant that fires two would itself be a
    /// finding. All three act on the carrier state 9, whose population is asserted nonzero
    /// in the (N=2, P=0) sector the plants act on before any of them is read.
    #[test]
    fn each_conservation_plant_fires_its_own_gate_and_no_other() {
        let m = Model::fhp6();
        let carrier = fhp(48).cells.iter().filter(|&&s| s == 9).count();
        assert!(carrier > 0, "plant carrier 9 is absent from the sector the plants act on");

        for (target, moves) in [(0u8, [true, false, false]), (34, [false, true, false]), (5, [false, false, true])] {
            let mut c = m.fhp_i(true);
            c[9] = target;
            let mut g = Lattice::seeded(m.clone(), 48, 0xC1A5, 0.35, c);
            let l0 = g.ledger();
            let (mut dm, mut dx, mut dy) = (false, false, false);
            for _ in 0..40 {
                g.step();
                let l = g.ledger();
                dm |= l.mass != l0.mass;
                dx |= l.momentum[0] != l0.momentum[0];
                dy |= l.momentum[1] != l0.momentum[1];
            }
            assert_eq!([dm, dx, dy], moves, "plant 9->{target} moved the wrong gates");
        }
    }

    /// The motion is a bijection on the FULL micro-state, not only on the 64-state table:
    /// stream and unstream round-trip, and the collision inverts.
    #[test]
    fn the_motion_is_a_bijection_on_the_whole_micro_state() {
        let mut g = fhp(32);
        for _ in 0..7 {
            g.step();
        }
        let before = g.cells.clone();
        g.step();
        let unstreamed = g.unstream(&g.cells);
        let m = &g.model;
        let c = m.fhp_i(true);
        let mut inverse = vec![0u8; 64];
        for s in 0..64u8 {
            inverse[c[s as usize] as usize] = s;
        }
        let recovered: Vec<u8> = unstreamed.iter().map(|&s| inverse[s as usize]).collect();
        assert_eq!(recovered, before, "T is not invertible on the micro-state");
    }

    /// G6's two-sided Leg-A gauge. HPP-4 holds its per-line momenta EXACTLY; FHP-6 does
    /// not. One instrument, one chart granularity, both answers.
    #[test]
    fn hpp_holds_its_spurious_line_chart_exactly_and_fhp_does_not() {
        let hm = Model::hpp4();
        let hc = hm.hpp_collision();
        let mut h = Lattice::seeded(hm, 32, 0x4899, 0.35, hc);
        let h0 = h.line_momenta();
        for t in 0..300 {
            h.step();
            assert_eq!(h.line_momenta(), h0, "HPP-4's line chart moved at step {t}");
        }
        let mut f = fhp(32);
        let f0 = f.line_momenta();
        let mut broke = None;
        for t in 0..100 {
            f.step();
            if f.line_momenta() != f0 {
                broke = Some(t);
                break;
            }
        }
        assert!(broke.is_some(), "FHP-6's line chart held: the gauge has only one side");
    }

    /// G4: with a wall the momentum ledger balances only when the impulse is a TERM in it.
    /// Mass stays exact and is gated separately; P7 plants the dropped impulse.
    #[test]
    fn the_wall_impulse_closes_the_momentum_ledger_exactly() {
        let m = Model::fhp6();
        let c = m.fhp_i(true);
        let mut g = Lattice::seeded(m, 40, 0x7A11, 0.35, c);
        g.add_wall(8, 20, 16);
        let l0 = g.ledger();
        let mut bounced = false;
        for t in 0..200 {
            g.step();
            let l = g.ledger();
            assert_eq!(l.mass, l0.mass, "the wall lost mass at step {t}");
            assert_eq!(
                [l.momentum[0], l.momentum[1]],
                [l0.momentum[0] + l.wall_impulse[0], l0.momentum[1] + l.wall_impulse[1]],
                "momentum ledger did not balance at step {t}"
            );
            bounced |= l.wall_impulse != [0, 0];
        }
        assert!(bounced, "no particle ever hit the wall: the ledger gate did no work");
        // P7: the same gate with the impulse dropped from the ledger must FAIL, which it
        // can only do if the wall actually moved momentum by the end of the run.
        let l = g.ledger();
        assert_ne!(l.wall_impulse, [0, 0], "P7 cannot fire: the run ends with zero impulse");
        assert_ne!(
            [l.momentum[0], l.momentum[1]], [l0.momentum[0], l0.momentum[1]],
            "P7 did not fire: the wall channel is invisible to the ledger gate"
        );
    }

    /// The colour step's occupation half IS `advance`, measured rather than asserted: with an
    /// all-zero colour plane the two write bit-identical states, step after step, with and
    /// without a wall and with and without the chirality hash.
    #[test]
    fn the_colour_step_reduces_to_advance() {
        let m = Model::fhp6();
        for (walled, chiral) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut a = Lattice::seeded(m.clone(), 32, 0xC1A5, 0.35, m.fhp_i(true));
            if walled {
                a.add_wall(5, 11, 9);
            }
            if chiral {
                a.randomize_chirality(0x5EED);
            }
            let mut plain = a.cells.clone();
            let mut plain_out = vec![0u8; 32 * 32];
            let mut col = a.cells.clone();
            let mut colour = vec![0u8; 32 * 32];
            let mut col_out = vec![0u8; 32 * 32];
            let mut col_colour_out = vec![0u8; 32 * 32];
            for t in 0..120u64 {
                let sa = a.advance(&mut plain, &mut plain_out, t);
                let sb = a.advance_with_colour(&mut col, &mut colour, &mut col_out, &mut col_colour_out, t);
                core::mem::swap(&mut plain, &mut plain_out);
                core::mem::swap(&mut col, &mut col_out);
                core::mem::swap(&mut colour, &mut col_colour_out);
                assert_eq!(plain, col, "wall={walled} chiral={chiral}: states diverged at step {t}");
                assert_eq!(sa, sb, "wall={walled} chiral={chiral}: stats diverged at step {t}");
                assert!(colour.iter().all(|&q| q == 0), "colour appeared from nowhere at step {t}");
            }
        }
    }

    /// G13: the carrier moves. A closure reading on a fixed point is vacuous and this is
    /// the counter that refuses one (M-FIXED-POINT-TRAJECTORY).
    #[test]
    fn the_carrier_provably_moves() {
        let mut g = fhp(64);
        let start = g.cells.clone();
        for _ in 0..100 {
            g.step();
        }
        let d = Lattice::occupancy_distance(&start, &g.cells);
        assert!(d as f64 > 0.30 * (64 * 64) as f64, "carrier barely moved: distance {d}");
    }
}
