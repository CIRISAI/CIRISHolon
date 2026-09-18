//! THE CONTINUUM CHART: coarse fields over cells, and whether the fine motion closes them.
//!
//! This is rung 2 of GANTT node G. `conformance/water_observatory/RUNG2_PREREG.md` is the
//! freeze and every threshold below is one of its constants; nothing here chose a number.
//!
//! # What it reads and what it refuses
//!
//! The chart is `n_x × n_y` Eulerian cells over the box, carrying three fields per cell:
//! occupancy (per species, exactly integral), momentum (binned) and KINETIC energy
//! (binned). The potential energy is REFUSED as a field and the prereg §2.2 says why: a
//! pair straddling a face has no share of its interaction energy that the dynamics forces
//! onto one side, so a cell-local potential would be a free parameter living inside the
//! chart.
//!
//! # The one thing this module exists to not do
//!
//! `lean/CIRISHolon/Tiers.lean::exists_closed_view` proves that every step closes SOME
//! view — itself. The one-cell chart's fields ARE the motion's invariants, so it is Closed
//! for free and measures nothing. [`Gates::vacuity`] is the fence, and the `(1,1)` grid is
//! run in every arm as a control that MUST read VOID.
//!
//! The second trap is `ClosureLadder.lean::refinement_removes_collisions`: refinement can
//! only remove collisions, for ANY views whatsoever. So [`ladder_monotone`] is an
//! instrument SELF-CHECK — a violation convicts this file — and is never a finding.
//!
//! # Units
//!
//! Hartree atomic units throughout, as the engine carries them: bohr, hartree, electron
//! masses. The two masses and `k_B` are duplicated from `holon-chem` rather than imported,
//! because this crate has zero dependencies by design (see its `Cargo.toml`); they are
//! pinned by [`tests::masses_match_the_element_table`] against the values that crate
//! declares, so a drift in either place fires.

use crate::traj::Trajectory;
use std::collections::HashMap;

// ------------------------------------------------------------------ pinned constants

/// Electron masses per unified atomic mass unit. `holon-chem::elements::M_E_PER_U`.
pub const M_E_PER_U: f64 = 1822.888486;
/// `¹H` atomic mass in u. `holon-chem::elements::HYDROGEN.mass_u`.
pub const H_MASS_U: f64 = 1.00782503207;
/// `¹⁶O` atomic mass in u. `holon-chem::elements::OXYGEN.mass_u`.
pub const O_MASS_U: f64 = 15.9949146196;
/// Boltzmann's constant, hartree per kelvin. `holon-render::sim::K_B`.
pub const K_B: f64 = 3.166811563e-6;
/// The thermostat target of the frozen `waterquench` protocol, kelvin.
pub const T_TARGET: f64 = 300.0;

/// Mass in electron masses for a nuclear charge the quench protocol can carry.
///
/// REFUSES any other charge rather than guessing: a scene with a third element is a
/// different protocol and this chart has not been staked for it.
pub fn mass_me(z: u32) -> Result<f64, Refusal> {
    match z {
        1 => Ok(H_MASS_U * M_E_PER_U),
        8 => Ok(O_MASS_U * M_E_PER_U),
        other => Err(Refusal::UnknownSpecies(other)),
    }
}

/// The momentum bin width: the thermal momentum of a hydrogen atom at the thermostat's
/// own target, `√(m_H · k_B · T_target)`. PREREG §2.3 — derived from protocol constants,
/// never fitted, so no number in the chart was chosen after seeing data.
pub fn dp_au() -> f64 {
    (H_MASS_U * M_E_PER_U * K_B * T_TARGET).sqrt()
}

/// The energy bin width: one thermal quantum, `k_B · T_target`. PREREG §2.3.
pub fn de_ha() -> f64 {
    K_B * T_TARGET
}

/// PREREG §2.3's stated values, to the precision the freeze prints them.
pub const DP_AU_FROZEN: f64 = 1.3211;
pub const DE_HA_FROZEN: f64 = 9.500435e-4;

// ------------------------------------------------------------------------- refusals

/// Object rule 9: a reading the scene cannot carry REFUSES and names the gate whose
/// passing would lift the refusal. It never returns a number that looks like a
/// measurement.
#[derive(Clone, Debug, PartialEq)]
pub enum Refusal {
    /// R1 — an atom outside the box on some frame. Never clamped into an edge cell: a
    /// clamp would manufacture occupancy at the boundary.
    AtomOutsideBox {
        frame: usize,
        atom: usize,
        x: f64,
        y: f64,
    },
    /// R2 — fewer frames than any closure statement could rest on.
    TooFewFrames { have: usize, need: usize },
    /// A species the frozen protocol does not carry.
    UnknownSpecies(u32),
    /// A grid with no cells.
    EmptyGrid,
}

// ---------------------------------------------------------------------------- charts

/// A cell grid over the box. PREREG §2.5 freezes the list that is tested.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    pub nx: usize,
    pub ny: usize,
}

impl Grid {
    pub fn cells(&self) -> usize {
        self.nx * self.ny
    }
}

/// PREREG §2.5's frozen grid list: mean occupancies 12, 6, 3, 1.5, 0.5 at N = 12.
pub const FROZEN_GRIDS: [Grid; 5] = [
    Grid { nx: 1, ny: 1 },
    Grid { nx: 2, ny: 1 },
    Grid { nx: 2, ny: 2 },
    Grid { nx: 4, ny: 2 },
    Grid { nx: 6, ny: 4 },
];

// ------------------------------------------------------------- RUNG2_AMENDMENT_1.md
//
// Three changes to the freeze's letter, each derived from the freeze's own constants and
// declared in `conformance/water_observatory/RUNG2_AMENDMENT_1.md` before any read. The
// frozen path above and below is UNCHANGED: [`Grid`], [`cell_series`] and [`readings`] keep
// their signatures and are now thin wrappers over the general forms with `n_z = 1` and
// `Density::Exact`, so the freeze's chart is reproduced bit for bit (plant PA-4).

/// A2 — a cell grid with its third axis. `n_z = 1` on a `dims = 2` carrier IS the freeze's
/// grid; on a `dims = 3` carrier a cell is a box, not a column through the box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid3 {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
}

impl Grid3 {
    pub fn cells(&self) -> usize {
        self.nx * self.ny * self.nz
    }
}

impl From<Grid> for Grid3 {
    fn from(g: Grid) -> Grid3 {
        Grid3 { nx: g.nx, ny: g.ny, nz: 1 }
    }
}

/// A1 — how the density field is read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Density {
    /// The freeze: exact integer occupancy. Right at `N = 12`, where occupancies `0–12`
    /// repeat; empty of collisions by counting at the occupancy G2 demands.
    Exact,
    /// Amendment 1: occupancy binned at `Δn = √⟨n⟩`, one Poisson standard deviation —
    /// the resolution G2 itself defines a fluid element by (`1/√N ≤ 0.10`). Parameter-free
    /// (`⟨n⟩ = N_species / cells` is arithmetic), and parallel to `Δp` and `Δe`. The
    /// momentum and energy bins stay the freeze's per-atom ones.
    Poisson,
    /// Amendment 2 (`RUNG2_AMENDMENT_2.md`): Amendment 1's rule applied to every field —
    /// `Δ_cell = √⟨n⟩ · Δ_atom` with `Δ_atom` the freeze's own `1`, `Δp`, `Δe`. A cell's
    /// field is known to within `√⟨n⟩` of the resolution the freeze gave one atom's. At the
    /// density rung this IS `Poisson` (plant PB-4); at the momentum and energy rungs it is
    /// what lets two frames share a reading at all (plant PB-1). **Superseded by `Derived`:
    /// its scale was calibrated on an unrepresentative box (the correction at the head of
    /// `RUNG2_AMENDMENT_2.md`) and is kept as the printed control.**
    CellScale,
    /// Amendment 3 (`RUNG2_AMENDMENT_3.md`): the two continuous bins DERIVED as one standard
    /// deviation of the cell's field at equilibrium for `⟨n⟩` independent thermal atoms of
    /// the carrier's own mean mass — `Δp = √(⟨n⟩ m̄ k_B T (1 − ⟨n⟩/N))` per component, with
    /// the finite-population factor of a conserved total momentum, and `Δe = √(3⟨n⟩/2) k_B T`.
    /// Density keeps Amendment 1's `√⟨n⟩`. Used as the nearest integer multiple of the
    /// freeze's per-atom bins so the freeze's chart refines this one exactly (PC-3).
    Derived,
    /// Amendment 4, route (i): density at a LIQUID's fluctuation, `√(S(0) ⟨n⟩ (1 − ⟨n⟩/N))`
    /// with water's structure factor at zero wavevector as a declared external constant
    /// ([`WATER_S0`]); momentum and energy as `Derived`.
    External,
    /// Amendment 4, route (ii): density at `σ(n)` MEASURED on held-out seeds of the same
    /// liquid at the same grid (never on the file being graded — the driver refuses that);
    /// momentum and energy as `Derived`. The value is the pooled `σ` for the whole carrier;
    /// per-species it scales as `√(N_s / N)`.
    Calibrated(f64),
}

/// Water's structure factor at zero wavevector, `S(0) = ρ k_B T κ_T`, from CRC constants at
/// 25 °C: `κ_T = 45.24e-11 Pa⁻¹`, `ρ = 997.05 kg m⁻³`, `T = 298.15 K`. An external protocol
/// constant, entering as `T_TARGET` does (RUNG2_AMENDMENT_4.md, route (i)).
pub const WATER_S0: f64 = 0.0621;

/// Amendment 3's two continuous bins as multiples of the freeze's, for a cell of `n_bar`
/// atoms on a carrier of `n_total` atoms of mean mass `m_bar` (electron masses). Public so
/// a record can print the derivation beside the reading.
pub fn derived_multiples(n_bar: f64, n_total: f64, m_bar: f64) -> (usize, usize) {
    let var_p = n_bar * m_bar * K_B * T_TARGET * (1.0 - n_bar / n_total).max(0.0);
    let dp_cell = var_p.sqrt();
    let de_cell = (1.5 * n_bar).sqrt() * K_B * T_TARGET;
    ((dp_cell / dp_au()).round().max(1.0) as usize, (de_cell / de_ha()).round().max(1.0) as usize)
}

/// A3 — the grid ladder derived from the carrier: `2^k` cells while `⟨n⟩ ≥ 1`, each doubling
/// splitting the longest remaining axis. Reproduces the freeze's first four grids at
/// `N = 12`, `dims = 2` (plant PA-6). `2^0` is G3's vacuity control, as `(1,1)` was.
pub fn doubling_ladder(n_atoms: usize, dims: u32) -> Vec<Grid3> {
    let mut out = Vec::new();
    let mut g = Grid3 { nx: 1, ny: 1, nz: 1 };
    loop {
        if g.cells() == 0 || (n_atoms as f64) / (g.cells() as f64) < 1.0 {
            break;
        }
        out.push(g);
        // split the longest axis; ties go x, then y, then z — a stated order, not a choice
        let axes: [(usize, usize); 3] = [(g.nx, 0), (g.ny, 1), (g.nz, 2)];
        let live = if dims >= 3 { 3 } else { 2 };
        let (_, which) = axes[..live].iter().copied().fold((usize::MAX, 0), |best, (n, i)| if n < best.0 { (n, i) } else { best });
        match which {
            0 => g.nx *= 2,
            1 => g.ny *= 2,
            _ => g.nz *= 2,
        }
    }
    out
}

/// The chart ladder of PREREG §2.4. Each rung REFINES the one before, which is what makes
/// `refinement_removes_collisions` applicable and [`ladder_monotone`] meaningful.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rung {
    /// v1 — occupancy per species per cell. Exactly discrete: no binning, no choice.
    Occ,
    /// v2 — v1 plus binned momentum.
    Mom,
    /// v3 — v2 plus binned kinetic energy.
    Ene,
}

pub const LADDER: [Rung; 3] = [Rung::Occ, Rung::Mom, Rung::Ene];

/// How an atom is assigned to a cell.
///
/// The three non-spatial kinds are controls, and they are NOT interchangeable — PREREG
/// §3.5 and §7 stake different jobs for them:
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// The real chart: cell from position.
    Spatial,
    /// **The control as the freeze literally wrote it** — cell from a fixed permutation of
    /// ARENA INDEX. Membership is then constant in time, so this chart has ZERO transport
    /// and the vacuity fence takes it. That is a defect in the freeze and it is reported
    /// as one rather than repaired in place; it is kept and run so the finding has its
    /// witness.
    BlindIndex,
    /// **The control that actually discriminates.** Cell is `π_i(spatial cell of i)`, with
    /// `π_i` a fixed permutation of the cell labels drawn PER ARENA INDEX. Each atom's
    /// cell series is a relabelling of its true one — identical transition times, identical
    /// dwell distribution, identical transport rate — while the aggregate field loses the
    /// spatial coherence BETWEEN atoms. Same field arity, same occupancy total, no spatial
    /// meaning.
    BlindLabel,
    /// P-7's paired negative: ONE permutation of cell labels shared by every atom. That is
    /// a pure relabelling of the chart, so every reading must be a relabelling and every
    /// gate reading must be bit-identical to `Spatial`. If this moves a number, the
    /// instrument is reading presentation rather than structure (M-PRESENTATION-VERDICT).
    GlobalRelabel,
}

/// A chart reading at one frame: the fields, flattened in a fixed order, as integers.
///
/// Integers throughout — occupancy is already integral and both continuous fields are
/// binned — so equality of readings is EXACT and a collision is a fact, never a tolerance.
pub type Reading = Vec<i64>;

/// A per-atom permutation of cell labels, deterministic from a stated seed.
///
/// Deterministic and stated so the control is reproducible: the same seed gives the same
/// permutations on any machine, and no permutation is drawn after seeing a defect.
fn label_perms(n_atoms: usize, cells: usize, seed: u64, per_atom: bool) -> Vec<Vec<usize>> {
    let mut s = seed;
    let next = |s: &mut u64| {
        *s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (*s >> 33) as usize
    };
    let mut shared: Vec<usize> = (0..cells).collect();
    for i in (1..cells).rev() {
        let j = next(&mut s) % (i + 1);
        shared.swap(i, j);
    }
    if !per_atom {
        return vec![shared; n_atoms];
    }
    (0..n_atoms)
        .map(|_| {
            let mut p: Vec<usize> = (0..cells).collect();
            for i in (1..cells).rev() {
                let j = next(&mut s) % (i + 1);
                p.swap(i, j);
            }
            p
        })
        .collect()
}

/// The seed the controls are drawn from. Stated once, here, so it is a constant of the
/// instrument and not a knob.
pub const CONTROL_SEED: u64 = 0x5255_4e47_3200_0002;

/// The cell of each atom on each frame, under the chosen assignment.
///
/// This is separated from [`readings`] because the transport gate needs the cell series
/// and nothing else, and because a chart that refuses should refuse before any field is
/// summed.
pub fn cell_series(
    traj: &Trajectory,
    grid: Grid,
    kind: Kind,
) -> Result<Vec<Vec<usize>>, Refusal> {
    cell_series3(traj, grid.into(), kind)
}

/// The general form (RUNG2_AMENDMENT_1 A2). With `n_z = 1` this is [`cell_series`] to the
/// bit: the third coordinate is neither read nor checked, so a `dims = 2` carrier whose `z`
/// is anything at all reads exactly as the freeze read it.
pub fn cell_series3(
    traj: &Trajectory,
    grid: Grid3,
    kind: Kind,
) -> Result<Vec<Vec<usize>>, Refusal> {
    if grid.cells() == 0 {
        return Err(Refusal::EmptyGrid);
    }
    let n = traj.header.n_atoms;
    let (w, h, d) = (traj.header.box_w, traj.header.box_h, traj.header.box_d);
    let (cw, ch, cd) = (w / grid.nx as f64, h / grid.ny as f64, d / grid.nz as f64);
    let perms = match kind {
        Kind::Spatial => None,
        Kind::BlindIndex => None,
        Kind::BlindLabel => Some(label_perms(n, grid.cells(), CONTROL_SEED, true)),
        Kind::GlobalRelabel => Some(label_perms(n, grid.cells(), CONTROL_SEED, false)),
    };
    let mut out = Vec::with_capacity(traj.frames.len());
    for (fi, f) in traj.frames.iter().enumerate() {
        let mut row = Vec::with_capacity(n);
        for a in 0..n {
            let cell = match kind {
                // The freeze's literal control: index only, so membership never changes.
                Kind::BlindIndex => {
                    let mut s = CONTROL_SEED ^ (a as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
                    s ^= s >> 29;
                    (s as usize) % grid.cells()
                }
                _ => {
                    let (x, y) = (f.pos[a][0], f.pos[a][1]);
                    // R1: refuse rather than clamp. A clamp manufactures edge occupancy.
                    if !(x >= 0.0 && x <= w && y >= 0.0 && y <= h) {
                        return Err(Refusal::AtomOutsideBox { frame: fi, atom: a, x, y });
                    }
                    let ix = ((x / cw) as usize).min(grid.nx - 1);
                    let iy = ((y / ch) as usize).min(grid.ny - 1);
                    let iz = if grid.nz > 1 {
                        let z = f.pos[a][2];
                        if !(z >= 0.0 && z <= d) {
                            return Err(Refusal::AtomOutsideBox { frame: fi, atom: a, x: z, y: f64::NAN });
                        }
                        ((z / cd) as usize).min(grid.nz - 1)
                    } else {
                        0
                    };
                    let c = (iz * grid.ny + iy) * grid.nx + ix;
                    match &perms {
                        Some(p) => p[a][c],
                        None => c,
                    }
                }
            };
            row.push(cell);
        }
        out.push(row);
    }
    Ok(out)
}

/// The chart's readings, one per frame.
///
/// Field order is fixed and documented so a reimplementation is bit-identical: for each
/// cell in index order, the occupancy of each species in ASCENDING nuclear charge; then,
/// at `Mom` and above, each cell's two binned momentum components; then, at `Ene`, each
/// cell's binned kinetic energy.
pub fn readings(
    traj: &Trajectory,
    grid: Grid,
    rung: Rung,
    kind: Kind,
) -> Result<Vec<Reading>, Refusal> {
    readings3(traj, grid.into(), rung, kind, Density::Exact)
}

/// The general form (RUNG2_AMENDMENT_1 A1 + A2). With `n_z = 1` and `Density::Exact` this
/// is [`readings`] to the bit. Under `Density::Poisson` each species' occupancy is binned at
/// `floor(n / √(N_species / cells))`; the momentum and energy fields are binned exactly as
/// the freeze binned them, unchanged.
pub fn readings3(
    traj: &Trajectory,
    grid: Grid3,
    rung: Rung,
    kind: Kind,
    density: Density,
) -> Result<Vec<Reading>, Refusal> {
    let cells = cell_series3(traj, grid, kind)?;
    let n = traj.header.n_atoms;
    let nc = grid.cells();
    let mut species: Vec<u32> = traj.header.z.clone();
    species.sort_unstable();
    species.dedup();
    // A1: one Poisson scale per species, from arithmetic the header fixes. Amendment 4's
    // two routes replace the Poisson variance by a liquid's: `S(0)⟨n⟩(1−⟨n⟩/N)` (External)
    // or a held-out measured `σ²` scaled to the species (Calibrated). No bin is below 1:
    // a fraction of an atom is not a resolution.
    let dn: Vec<f64> = species
        .iter()
        .map(|z| {
            let n_s = traj.header.z.iter().filter(|q| *q == z).count() as f64;
            let n_bar = n_s / (nc as f64);
            match density {
                Density::External => (WATER_S0 * n_bar * (1.0 - n_bar / n_s).max(0.0)).sqrt().max(1.0),
                Density::Calibrated(sigma) => (sigma * (n_s / n as f64).sqrt()).max(1.0),
                _ => n_bar.sqrt().max(1.0),
            }
        })
        .collect();
    let masses: Vec<f64> = traj
        .header
        .z
        .iter()
        .map(|z| mass_me(*z))
        .collect::<Result<_, _>>()?;
    // RUNG2_AMENDMENT_2: the per-cell momentum and energy scales. `⟨n⟩` here is the cell's
    // total occupancy over all species, because those two fields are sums over every atom
    // in the cell. ROUNDED to an integer multiple of the freeze's bin, and the reason is a
    // theorem about `floor`: a coarse bin is a union of fine bins only when the bin ratio
    // is an integer, so `refines(Poisson, CellScale)` — plant PB-2, the self-check — holds
    // exactly only then. (The density field needs no rounding: its underlying value is an
    // integer, and `floor(n / Δ)` is a function of `n` for any `Δ`, which is why PA-5 held
    // at unrounded `√⟨n⟩` and why A1's density rule is kept as written.)
    let (kp, ke) = match density {
        Density::CellScale => {
            let k = ((n as f64) / (nc as f64)).sqrt().round().max(1.0) as usize;
            (k, k)
        }
        Density::Derived | Density::External | Density::Calibrated(_) => {
            let m_bar = masses.iter().sum::<f64>() / (n as f64);
            derived_multiples((n as f64) / (nc as f64), n as f64, m_bar)
        }
        _ => (1, 1),
    };
    let (dp, de) = (dp_au() * kp as f64, de_ha() * ke as f64);

    let mut out = Vec::with_capacity(traj.frames.len());
    for (fi, f) in traj.frames.iter().enumerate() {
        let mut occ = vec![0i64; nc * species.len()];
        let mut px = vec![0.0f64; nc];
        let mut py = vec![0.0f64; nc];
        let mut ek = vec![0.0f64; nc];
        for a in 0..n {
            let c = cells[fi][a];
            let si = species.iter().position(|z| *z == traj.header.z[a]).unwrap();
            occ[c * species.len() + si] += 1;
            if rung >= Rung::Mom {
                let m = masses[a];
                px[c] += m * f.vel[a][0];
                py[c] += m * f.vel[a][1];
                if rung >= Rung::Ene {
                    let v2 = f.vel[a][0] * f.vel[a][0]
                        + f.vel[a][1] * f.vel[a][1]
                        + f.vel[a][2] * f.vel[a][2];
                    ek[c] += 0.5 * m * v2;
                }
            }
        }
        let mut r: Reading = match density {
            Density::Exact => occ,
            Density::Poisson | Density::CellScale | Density::Derived | Density::External | Density::Calibrated(_) => occ
                .iter()
                .enumerate()
                .map(|(k, o)| ((*o as f64) / dn[k % species.len()]).floor() as i64)
                .collect(),
        };
        if rung >= Rung::Mom {
            for c in 0..nc {
                r.push((px[c] / dp).floor() as i64);
                r.push((py[c] / dp).floor() as i64);
            }
        }
        if rung >= Rung::Ene {
            for c in 0..nc {
                r.push((ek[c] / de).floor() as i64);
            }
        }
        out.push(r);
    }
    Ok(out)
}

// ------------------------------------------------------------------------- the legs

/// LEG A — the collision form (PREREG G5).
///
/// Chosen over a fitted-model residual deliberately: `M-ONE-MODEL-DELTA` says a defect
/// against one chosen model earns only "worse than that model", while the collision form
/// earns "best memoryless" — which is what `Closed` needs, since its `h` is quantified
/// existentially.
#[derive(Clone, Debug, Default)]
pub struct LegA {
    /// Pairs of frames with equal readings. Counted in closed form, never enumerated: a
    /// coarse chart can carry 10^8 of them and the count must still be exact.
    pub collisions: u128,
    /// Collisions whose successors differ — `ClosureLadder.lean::Firing`.
    pub firing: u128,
    /// Transitions departing from a reading visited at least twice (PREREG G4).
    pub informative: usize,
    /// Distinct readings seen.
    pub distinct: usize,
    /// A bounded exhibition of firing pairs by frame index. The COUNT above is exact and
    /// complete; this listing is truncated, and the results document says so.
    pub witnesses: Vec<(usize, usize)>,
}

impl LegA {
    /// `D_A` — the firing fraction. `None` when there are no collisions at all, which is
    /// a different fact from a defect of zero (M-EXIT-DISCRIMINATOR: a record that cannot
    /// tell "no work" from "no failures" has deleted the distinction).
    pub fn defect(&self) -> Option<f64> {
        if self.collisions == 0 {
            None
        } else {
            Some(self.firing as f64 / self.collisions as f64)
        }
    }
}

/// The exhibition cap. The firing COUNT is exact; only the listing is bounded.
pub const WITNESS_CAP: usize = 10;

pub fn leg_a(readings: &[Reading]) -> LegA {
    let mut groups: HashMap<&Reading, Vec<usize>> = HashMap::new();
    if readings.len() < 2 {
        return LegA::default();
    }
    // The last frame has no successor, so it can be a collision partner but never
    // contributes a transition. It is excluded from the grouping entirely rather than
    // special-cased later.
    for i in 0..readings.len() - 1 {
        groups.entry(&readings[i]).or_default().push(i);
    }
    let mut out = LegA {
        distinct: groups.len(),
        ..Default::default()
    };
    for (_, idx) in groups.iter() {
        let s = idx.len() as u128;
        if s < 2 {
            continue;
        }
        out.informative += idx.len();
        out.collisions += s * (s - 1) / 2;
        // Partition the group by SUCCESSOR reading; agreeing pairs are the within-class
        // pairs, so firing = C(S,2) − Σ C(s_k,2). Closed form, so a group of 10^4 frames
        // costs 10^4 and not 10^8.
        let mut by_succ: HashMap<&Reading, Vec<usize>> = HashMap::new();
        for &i in idx {
            by_succ.entry(&readings[i + 1]).or_default().push(i);
        }
        let mut agree: u128 = 0;
        for (_, sub) in by_succ.iter() {
            let k = sub.len() as u128;
            agree += k * (k - 1) / 2;
        }
        out.firing += s * (s - 1) / 2 - agree;
        if out.witnesses.len() < WITNESS_CAP && by_succ.len() > 1 {
            // Sorted before pairing: `HashMap` iteration order is not stable, and a
            // witness list that varies between runs of the same instrument on the same
            // bytes is not a record. The COUNTS above never depended on order; this
            // listing did.
            let mut classes: Vec<&Vec<usize>> = by_succ.values().collect();
            classes.sort_unstable_by_key(|c| c[0]);
            'w: for a in 0..classes.len() {
                for b in (a + 1)..classes.len() {
                    out.witnesses.push((classes[a][0], classes[b][0]));
                    if out.witnesses.len() >= WITNESS_CAP {
                        break 'w;
                    }
                }
            }
        }
    }
    out.witnesses.sort_unstable();
    out
}

/// LEG B — held out (PREREG G6).
///
/// `h` is built on the first half (each reading's MODAL successor) and applied to the
/// second. This is a ONE-MODEL delta by construction and carries that scope: it earns
/// "worse than this `h`", never "best memoryless". It exists because a low `D_A` can be
/// produced by a chart whose fibers are all visited inside one short stretch, and
/// generalisation is what that cannot fake.
#[derive(Clone, Debug, Default)]
pub struct LegB {
    /// Second-half transitions whose reading was seen in the first half.
    pub predicted: usize,
    /// Of those, the ones `h` got wrong.
    pub mismatched: usize,
    /// Second-half transitions in total — `predicted / attempted` is the coverage, and it
    /// is reported beside the defect because a defect over 1% of the frames is not a
    /// statement about the run.
    pub attempted: usize,
}

impl LegB {
    pub fn defect(&self) -> Option<f64> {
        if self.predicted == 0 {
            None
        } else {
            Some(self.mismatched as f64 / self.predicted as f64)
        }
    }
    pub fn coverage(&self) -> f64 {
        if self.attempted == 0 {
            0.0
        } else {
            self.predicted as f64 / self.attempted as f64
        }
    }
}

pub fn leg_b(readings: &[Reading]) -> LegB {
    if readings.len() < 4 {
        return LegB::default();
    }
    let last = readings.len() - 1; // transitions are 0..last
    let mid = last / 2;
    let mut tally: HashMap<&Reading, HashMap<&Reading, usize>> = HashMap::new();
    for i in 0..mid {
        *tally
            .entry(&readings[i])
            .or_default()
            .entry(&readings[i + 1])
            .or_default() += 1;
    }
    // The modal successor, with ties broken by the reading itself so the law is a function
    // of the data and not of hash order — two runs must build the same `h`.
    let law: HashMap<&Reading, &Reading> = tally
        .into_iter()
        .map(|(k, succ)| {
            let best = succ
                .into_iter()
                .max_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(b.0)))
                .map(|(r, _)| r)
                .unwrap();
            (k, best)
        })
        .collect();
    let mut out = LegB::default();
    for i in mid..last {
        out.attempted += 1;
        if let Some(pred) = law.get(&readings[i]) {
            out.predicted += 1;
            if **pred != readings[i + 1] {
                out.mismatched += 1;
            }
        }
    }
    out
}

// -------------------------------------------------------------------------- the gates

/// PREREG's staked constants. Named so a reader can check each against the freeze.
pub mod prereg {
    /// G2 — mean occupancy a fluid element needs, from `1/√N ≤ 0.10`.
    pub const ADMISSIBLE_OCCUPANCY: f64 = 100.0;
    /// G2 — cells a fluid-element chart needs.
    pub const ADMISSIBLE_CELLS: usize = 4;
    /// G2 — the relative density fluctuation a fluid element may carry.
    pub const ADMISSIBLE_FLUCTUATION: f64 = 0.10;
    /// G3 — the vacuity fence: cells, and the fraction of boundaries carrying a crossing.
    pub const MIN_CELLS: usize = 2;
    pub const MIN_TRANSPORT: f64 = 0.05;
    /// G4 — informative transitions, carried over unchanged from `CENSUS_PREREG.md` G6.
    pub const MIN_INFORMATIVE: usize = 200;
    /// G5/G6 — the budget, the census's own β carried over unchanged.
    pub const BETA: f64 = 0.02;
    /// G7 — the separation the position-blind control must lose by.
    pub const MIN_SEPARATION: f64 = 0.05;
}

/// The fraction of grain boundaries carrying at least one atom across a cell face
/// (PREREG G3). This is `M-FIXED-POINT-TRAJECTORY` in the field chart's clothes: a
/// closure gate is vacuous on a carrier the motion does not move.
pub fn transport_fraction(cells: &[Vec<usize>]) -> f64 {
    if cells.len() < 2 {
        return 0.0;
    }
    let moved = (0..cells.len() - 1)
        .filter(|&i| cells[i].iter().zip(&cells[i + 1]).any(|(a, b)| a != b))
        .count();
    moved as f64 / (cells.len() - 1) as f64
}

/// Mean occupancy and relative density fluctuation over cells and frames (PREREG G2).
///
/// `σ/⟨n⟩` is taken over the pooled cell-frame population, which is the quantity the
/// fluid-element argument is about: how much a single cell's count wanders relative to its
/// own mean.
pub fn occupancy_stats(cells: &[Vec<usize>], ncells: usize) -> (f64, f64) {
    if cells.is_empty() || ncells == 0 {
        return (0.0, 0.0);
    }
    let mut sum = 0.0f64;
    let mut sumsq = 0.0f64;
    let mut count = 0usize;
    for row in cells {
        let mut occ = vec![0usize; ncells];
        for &c in row {
            occ[c] += 1;
        }
        for &o in &occ {
            sum += o as f64;
            sumsq += (o * o) as f64;
            count += 1;
        }
    }
    let mean = sum / count as f64;
    let var = (sumsq / count as f64 - mean * mean).max(0.0);
    let rel = if mean > 0.0 { var.sqrt() / mean } else { 0.0 };
    (mean, rel)
}

/// The pooled standard deviation of cell occupancy over cells and frames — the quantity
/// Amendment 4's route (ii) calibrates on, measured on a trajectory OTHER than the one graded.
pub fn occupancy_sigma(cells: &[Vec<usize>], ncells: usize) -> f64 {
    let (mean, rel) = occupancy_stats(cells, ncells);
    mean * rel
}

/// Species totals are constant across every frame (PREREG G9a) — the ONE field of the
/// three whose conservation the dynamics actually constrains. Walls break momentum and
/// the thermostat breaks energy, so no null is staked on those (M-NULL-MISSTAKE); this
/// one is exact and a violation is a refusal.
pub fn species_conserved(traj: &Trajectory) -> bool {
    if traj.frames.is_empty() {
        return true;
    }
    // Nothing in the format lets nuclei change species, so this is a check that the
    // ARTIFACT is what it claims: a frame with a different atom count would fail it.
    traj.frames
        .iter()
        .all(|f| f.pos.len() == traj.header.n_atoms && f.vel.len() == traj.header.n_atoms)
}

/// The measured drift of the two fields whose conservation the protocol BREAKS
/// (PREREG G9b, G9c), reported as numbers rather than as a gate.
///
/// The ledger legs those gates ask for are NOT COMPUTABLE from this artifact: the dump
/// carries positions, velocities, bond bits, time and temperature, and no forces and no
/// intervention ledger. "Not computable" and "computed and failed" are different facts
/// (M-EXIT-DISCRIMINATOR), so the raw drift is what this returns and the ledger leg is
/// reported UNDISCHARGED.
pub struct Drift {
    pub p_first: [f64; 2],
    pub p_last: [f64; 2],
    pub ek_first: f64,
    pub ek_last: f64,
}

pub fn drift(traj: &Trajectory) -> Result<Drift, Refusal> {
    let masses: Vec<f64> = traj
        .header
        .z
        .iter()
        .map(|z| mass_me(*z))
        .collect::<Result<_, _>>()?;
    let totals = |k: usize| -> ([f64; 2], f64) {
        let f = &traj.frames[k];
        let mut p = [0.0f64; 2];
        let mut ek = 0.0f64;
        for a in 0..traj.header.n_atoms {
            p[0] += masses[a] * f.vel[a][0];
            p[1] += masses[a] * f.vel[a][1];
            ek += 0.5
                * masses[a]
                * (f.vel[a][0] * f.vel[a][0]
                    + f.vel[a][1] * f.vel[a][1]
                    + f.vel[a][2] * f.vel[a][2]);
        }
        (p, ek)
    };
    if traj.frames.is_empty() {
        return Err(Refusal::TooFewFrames { have: 0, need: 1 });
    }
    let (p0, e0) = totals(0);
    let (p1, e1) = totals(traj.frames.len() - 1);
    Ok(Drift {
        p_first: p0,
        p_last: p1,
        ek_first: e0,
        ek_last: e1,
    })
}

/// G8 — the ladder self-check. Collision counts must be non-increasing up the ladder,
/// because `refinement_removes_collisions` proves it for ANY views whatsoever.
///
/// **This is never a finding.** `M-CONJUNCTION-MONOTONE` is precisely the error of reading
/// a holds-for-any-predicate monotonicity as evidence. A violation convicts this file.
pub fn ladder_monotone(counts: &[u128]) -> bool {
    counts.windows(2).all(|w| w[1] <= w[0])
}

/// G8, THE STRONG FORM: does `fine` actually REFINE `coarse` on this trajectory?
///
/// `refinement_removes_collisions` has a hypothesis — `w = f ∘ v'`, the coarse view
/// factors through the fine one — and monotone collision counts do NOT establish it.
/// P-6 found that out the hard way: a mutated v2 that dropped the occupancy fields
/// entirely still had FEWER collisions than v1 (41,407 against 179,101), so the frozen
/// monotonicity check stayed silent on a chart that was not a refinement at all.
///
/// This is the check that has the hypothesis in it: every pair of frames the fine view
/// identifies must be identified by the coarse view too. O(F), by grouping.
///
/// The weak form is kept and still run, because it is what the freeze staked; this is
/// reported beside it as the repair, not as a substitution.
pub fn refines(fine: &[Reading], coarse: &[Reading]) -> bool {
    if fine.len() != coarse.len() {
        return false;
    }
    let mut seen: HashMap<&Reading, &Reading> = HashMap::new();
    for i in 0..fine.len() {
        match seen.get(&fine[i]) {
            Some(c) if **c != coarse[i] => return false,
            Some(_) => {}
            None => {
                seen.insert(&fine[i], &coarse[i]);
            }
        }
    }
    true
}

/// The verdict a single (arm, grid, chart) cell earns. VOID is a first-class answer and is
/// printed as loudly as a pass (PREREG G3, G4, G7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    CertifiedStrict,
    CertifiedBudgeted,
    NotClosed,
    VoidVacuous(&'static str),
    VoidWorkCount(usize),
    VoidNoCollisions,
}

/// Grade one cell against the frozen bars. The order matters and is the freeze's: vacuity
/// first, then work count, then the defect — a chart that fails the fence never reaches a
/// defect comparison at all.
pub fn grade(cells_ok: bool, transport: f64, leg: &LegA) -> Verdict {
    if !cells_ok {
        return Verdict::VoidVacuous("fewer than 2 cells: the fields are the invariants");
    }
    if transport < prereg::MIN_TRANSPORT {
        return Verdict::VoidVacuous("no transport: the chart is frozen");
    }
    if leg.informative < prereg::MIN_INFORMATIVE {
        return Verdict::VoidWorkCount(leg.informative);
    }
    match leg.defect() {
        None => Verdict::VoidNoCollisions,
        Some(d) if d == 0.0 => Verdict::CertifiedStrict,
        Some(d) if d <= prereg::BETA => Verdict::CertifiedBudgeted,
        Some(_) => Verdict::NotClosed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        Frame {
            index: i,
            time: i as f64,
            temperature: 300.0,
            bonds: crate::traj::BondSet::empty(),
            pos,
            vel,
        }
    }

    /// The two masses and `k_B` are duplicated from `holon-chem` and `holon-render`
    /// because this crate has zero dependencies. They are pinned here so a drift in
    /// either place fires rather than silently rescaling every momentum bin.
    #[test]
    fn masses_match_the_element_table() {
        assert_eq!(H_MASS_U, 1.00782503207);
        assert_eq!(O_MASS_U, 15.9949146196);
        assert_eq!(M_E_PER_U, 1822.888486);
        assert_eq!(K_B, 3.166811563e-6);
        assert!(mass_me(6).is_err(), "a species the protocol lacks must refuse");
    }

    /// The bin widths are DERIVED from protocol constants, and the freeze printed them.
    /// If either drifts from what `RUNG2_PREREG.md` §2.3 states, the chart is no longer
    /// the chart that was staked.
    #[test]
    fn bin_widths_match_the_freeze() {
        assert!(
            (dp_au() - DP_AU_FROZEN).abs() < 5e-5,
            "dp = {} but the freeze printed {DP_AU_FROZEN}",
            dp_au()
        );
        assert!(
            (de_ha() - DE_HA_FROZEN).abs() < 1e-9,
            "de = {} but the freeze printed {DE_HA_FROZEN}",
            de_ha()
        );
    }

    // ---------------------------------------------------------------- P-1: must VOID

    /// P-1. Carrier: atoms that never leave their starting cells. Sector the plant acts
    /// on: the TRANSPORT sector, exactly zero by construction, while the OCCUPANCY sector
    /// is nonzero (a nontrivial two-cell reading exists). Expected: no firing collision on
    /// any rung AND VOID at the vacuity fence.
    ///
    /// This is the plant that proves the fence fires. Without it, `exists_closed_view`
    /// gets reported as a result.
    #[test]
    fn p1_frozen_chart_voids_rather_than_certifies() {
        let n = 4;
        let pos: Vec<[f64; 3]> = vec![
            [5.0, 5.0, 0.0],
            [6.0, 5.0, 0.0],
            [25.0, 5.0, 0.0],
            [26.0, 5.0, 0.0],
        ];
        let vel = vec![[0.0; 3]; n];
        let traj = Trajectory {
            header: header(n, vec![1; n]),
            frames: (0..500)
                .map(|i| frame(i, pos.clone(), vel.clone()))
                .collect(),
        };
        let grid = Grid { nx: 2, ny: 1 };
        let cs = cell_series(&traj, grid, Kind::Spatial).unwrap();
        let t = transport_fraction(&cs);
        assert_eq!(t, 0.0, "the plant's transport sector must be exactly zero");
        let r = readings(&traj, grid, Rung::Occ, Kind::Spatial).unwrap();
        let a = leg_a(&r);
        assert_eq!(a.firing, 0, "a frozen chart has no firing collision");
        assert_eq!(
            grade(true, t, &a),
            Verdict::VoidVacuous("no transport: the chart is frozen"),
            "a chart the motion does not move must VOID, never certify"
        );
    }

    /// The one-cell chart is the same trap wearing the fields' clothes: its readings ARE
    /// the invariants. It must be caught by the cell count before anything else runs.
    #[test]
    fn p1b_one_cell_chart_voids() {
        let n = 3;
        let traj = Trajectory {
            header: header(n, vec![1; n]),
            frames: (0..300)
                .map(|i| {
                    let x = 5.0 + (i % 20) as f64;
                    frame(
                        i,
                        vec![[x, 5.0, 0.0], [x + 1.0, 6.0, 0.0], [x + 2.0, 7.0, 0.0]],
                        vec![[0.1, 0.0, 0.0]; n],
                    )
                })
                .collect(),
        };
        let grid = Grid { nx: 1, ny: 1 };
        let cs = cell_series(&traj, grid, Kind::Spatial).unwrap();
        let r = readings(&traj, grid, Rung::Occ, Kind::Spatial).unwrap();
        let a = leg_a(&r);
        assert_eq!(a.firing, 0, "one cell: occupancy is the invariant, never split");
        assert!(matches!(
            grade(grid.cells() >= prereg::MIN_CELLS, transport_fraction(&cs), &a),
            Verdict::VoidVacuous(_)
        ));
    }

    // -------------------------------------------------------------- P-2: must CERTIFY

    /// P-2. Carrier: a deterministic cellular rule in which each atom's next cell is a
    /// function of the chart alone, so the chart is Closed BY CONSTRUCTION. Sector: the
    /// TRANSPORT sector is nonzero (atoms cross faces every frame). Expected:
    /// CERTIFIED-STRICT. An instrument that cannot certify a chart built to be closed
    /// cannot certify anything.
    #[test]
    fn p2_a_chart_built_closed_certifies_strict() {
        let n = 4;
        let ncell = 4;
        let cw = 34.6 / ncell as f64;
        // Each atom marches one cell to the right per frame, wrapping. The occupancy
        // reading is a deterministic function of the previous occupancy reading.
        let frames: Vec<Frame> = (0..600)
            .map(|i| {
                let pos: Vec<[f64; 3]> = (0..n)
                    .map(|a| {
                        let c = (a + i as usize) % ncell;
                        [(c as f64 + 0.5) * cw, 10.0, 0.0]
                    })
                    .collect();
                frame(i as u64, pos, vec![[0.0; 3]; n])
            })
            .collect();
        let traj = Trajectory {
            header: header(n, vec![1; n]),
            frames,
        };
        let grid = Grid { nx: 4, ny: 1 };
        let cs = cell_series(&traj, grid, Kind::Spatial).unwrap();
        let t = transport_fraction(&cs);
        assert!(t > prereg::MIN_TRANSPORT, "the plant must actually transport");
        let r = readings(&traj, grid, Rung::Occ, Kind::Spatial).unwrap();
        let a = leg_a(&r);
        assert!(a.informative >= prereg::MIN_INFORMATIVE, "work count {}", a.informative);
        assert_eq!(
            grade(true, t, &a),
            Verdict::CertifiedStrict,
            "a chart closed by construction must certify"
        );
        let b = leg_b(&r);
        assert_eq!(b.mismatched, 0, "and its coarse law must generalise");
    }

    // --------------------------------------------------------------- P-3: must REJECT

    /// P-3. Carrier: P-2's motion with one hidden bit per atom deciding its move, invisible
    /// to every field of the chart. Sector: the HIDDEN sector is nonzero while the
    /// chart-visible sectors carry the same marginals. Expected: firing collisions — and
    /// still firing at the top of the ladder, because refinement cannot recover a variable
    /// the chart does not carry. This is the defect the rung exists to detect.
    #[test]
    fn p3_a_hidden_variable_fires_at_every_rung() {
        let n = 4;
        let ncell = 4;
        let cw = 34.6 / ncell as f64;
        let mut cells: Vec<usize> = (0..n).collect();
        let mut s: u64 = 0xDEAD_BEEF;
        let mut frames = Vec::new();
        for i in 0..1200u64 {
            let pos: Vec<[f64; 3]> = cells
                .iter()
                .map(|&c| [(c as f64 + 0.5) * cw, 10.0, 0.0])
                .collect();
            frames.push(frame(i, pos, vec![[0.0; 3]; n]));
            // The hidden bit: left or right, drawn from a stream the chart cannot see.
            for c in cells.iter_mut() {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                *c = if (s >> 60) & 1 == 1 {
                    (*c + 1) % ncell
                } else {
                    (*c + ncell - 1) % ncell
                };
            }
        }
        let traj = Trajectory {
            header: header(n, vec![1; n]),
            frames,
        };
        let grid = Grid { nx: 4, ny: 1 };
        let cs = cell_series(&traj, grid, Kind::Spatial).unwrap();
        let t = transport_fraction(&cs);
        let mut counts = Vec::new();
        for rung in LADDER {
            let r = readings(&traj, grid, rung, Kind::Spatial).unwrap();
            let a = leg_a(&r);
            counts.push(a.collisions);
            assert!(
                a.firing > 0,
                "the hidden variable must fire at rung {rung:?}; refinement cannot \
                 recover what the chart does not carry"
            );
            assert_eq!(grade(true, t, &a), Verdict::NotClosed);
            assert!(!a.witnesses.is_empty(), "a firing must exhibit its witness pair");
        }
        assert!(ladder_monotone(&counts), "G8: collisions must not grow up the ladder");
    }

    // ------------------------------------------- RUNG2_AMENDMENT_1.md's plants, PA-1..PA-6

    /// A 3D header for the amendment's carriers: a cube, so a cell can be a box.
    fn header3(n: usize, z: Vec<u32>, edge: f64) -> Header {
        Header { seed: 7, n_atoms: n, dims: 3, substeps: 64, n_frames: 0, dt: 1.0, box_w: edge, box_h: edge, box_d: edge, z }
    }

    /// A random walk of `n` atoms in a cube — a carrier with no structure to close on,
    /// whose only job is to POPULATE the grid at a stated occupancy.
    fn random_walk3(n: usize, edge: f64, frames: usize, seed: u64) -> Trajectory {
        let mut s = seed;
        let mut next = move || {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((s >> 11) as f64) / ((1u64 << 53) as f64)
        };
        let mut pos: Vec<[f64; 3]> = (0..n).map(|_| [next() * edge, next() * edge, next() * edge]).collect();
        let mut out = Vec::with_capacity(frames);
        for i in 0..frames {
            for p in pos.iter_mut() {
                for c in 0..3 {
                    // a step of ~3% of the edge, reflected at the walls so R1 never fires
                    let mut v = p[c] + (next() - 0.5) * 0.06 * edge;
                    if v < 0.0 { v = -v; }
                    if v > edge { v = 2.0 * edge - v; }
                    p[c] = v.clamp(0.0, edge);
                }
            }
            out.push(frame(i as u64, pos.clone(), vec![[0.0; 3]; n]));
        }
        Trajectory { header: header3(n, vec![8; n], edge), frames: out }
    }

    /// PA-1 — the fault, exhibited, and the repair, exhibited beside it. 400 atoms in a
    /// 2×2×1 grid is exactly G2's admissibility bar (100 per cell, 4 cells). On the
    /// freeze's EXACT chart the readings never repeat and the verdict is
    /// `VoidNoCollisions` by counting; on the amendment's Poisson chart the same frames
    /// collide and the work count is met. Nothing about the carrier changed between the
    /// two lines — only how its density was read.
    #[test]
    fn pa1_exact_occupancy_has_no_collisions_at_the_bar_and_the_poisson_bin_does() {
        let traj = random_walk3(400, 40.0, 600, 0x5041_3031);
        let grid = Grid3 { nx: 2, ny: 2, nz: 1 };
        let cs = cell_series3(&traj, grid, Kind::Spatial).unwrap();
        let (occ, fluct) = occupancy_stats(&cs, grid.cells());
        assert!(occ >= prereg::ADMISSIBLE_OCCUPANCY - 1e-9, "the plant must sit AT the bar, occ {occ}");
        assert!(fluct <= 0.2, "a random walk's fluctuation is Poisson-ish, got {fluct}");
        let t = transport_fraction(&cs);
        assert!(t > prereg::MIN_TRANSPORT, "the plant must transport, got {t}");
        let exact = leg_a(&readings3(&traj, grid, Rung::Occ, Kind::Spatial, Density::Exact).unwrap());
        // The fault, stated exactly: at admissible occupancy the exact chart is VOID — no
        // collisions, or too few to meet G4. A correlated walk repeats a few readings
        // frame-to-frame (73 here), so "none" would overstate it; "below the work count by
        // counting" is the claim, and it is the grade the real 2×1 chart got.
        assert!(
            matches!(grade(true, t, &exact), Verdict::VoidNoCollisions | Verdict::VoidWorkCount(_)),
            "the freeze's exact chart at admissible occupancy must be VOID by counting — \
             the fault the amendment exists for (collisions {}, informative {})",
            exact.collisions,
            exact.informative
        );
        let binned = leg_a(&readings3(&traj, grid, Rung::Occ, Kind::Spatial, Density::Poisson).unwrap());
        assert!(binned.collisions > 0, "the Poisson-binned chart must collide");
        assert!(
            binned.informative >= prereg::MIN_INFORMATIVE,
            "and meet G4 on the same frames the exact chart could not: {}",
            binned.informative
        );
        // A random walk has no structure to close on, so this is NOT a certification —
        // and the plant says so by asserting only that a verdict other than VOID exists.
        assert!(matches!(grade(true, t, &binned), Verdict::NotClosed | Verdict::CertifiedBudgeted | Verdict::CertifiedStrict));
    }

    /// PA-2 — binning does not MANUFACTURE a defect: P-2's chart, closed by construction,
    /// certifies strict on the Poisson chart exactly as it does on the exact one.
    #[test]
    fn pa2_a_chart_closed_by_construction_still_certifies_under_the_poisson_bin() {
        let n = 4;
        let ncell = 4;
        let cw = 34.6 / ncell as f64;
        let frames: Vec<Frame> = (0..600)
            .map(|i| {
                let pos: Vec<[f64; 3]> = (0..n).map(|a| { let c = (a + i as usize) % ncell; [(c as f64 + 0.5) * cw, 10.0, 0.0] }).collect();
                frame(i as u64, pos, vec![[0.0; 3]; n])
            })
            .collect();
        let traj = Trajectory { header: header(n, vec![1; n]), frames };
        let grid = Grid3 { nx: 4, ny: 1, nz: 1 };
        let t = transport_fraction(&cell_series3(&traj, grid, Kind::Spatial).unwrap());
        let a = leg_a(&readings3(&traj, grid, Rung::Occ, Kind::Spatial, Density::Poisson).unwrap());
        assert!(a.informative >= prereg::MIN_INFORMATIVE);
        assert_eq!(grade(true, t, &a), Verdict::CertifiedStrict, "binning must not manufacture a defect");
    }

    /// PA-3 — binning does not HIDE a defect: P-3's hidden variable still fires on the
    /// Poisson chart at every rung.
    #[test]
    fn pa3_a_hidden_variable_still_fires_under_the_poisson_bin() {
        let n = 4;
        let ncell = 4;
        let cw = 34.6 / ncell as f64;
        let mut cells: Vec<usize> = (0..n).collect();
        let mut s: u64 = 0xDEAD_BEEF;
        let mut frames = Vec::new();
        for i in 0..1200u64 {
            let pos: Vec<[f64; 3]> = cells.iter().map(|&c| [(c as f64 + 0.5) * cw, 10.0, 0.0]).collect();
            frames.push(frame(i, pos, vec![[0.0; 3]; n]));
            for c in cells.iter_mut() {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                *c = if (s >> 60) & 1 == 1 { (*c + 1) % ncell } else { (*c + ncell - 1) % ncell };
            }
        }
        let traj = Trajectory { header: header(n, vec![1; n]), frames };
        let grid = Grid3 { nx: 4, ny: 1, nz: 1 };
        let t = transport_fraction(&cell_series3(&traj, grid, Kind::Spatial).unwrap());
        for rung in LADDER {
            let a = leg_a(&readings3(&traj, grid, rung, Kind::Spatial, Density::Poisson).unwrap());
            assert!(a.firing > 0, "the hidden variable must still fire at {rung:?} under the bin");
            assert_eq!(grade(true, t, &a), Verdict::NotClosed);
        }
    }

    /// PA-4 — backward compatibility, EXACT: on a `dims = 2` carrier the `(n_x, n_y, 1)`
    /// chart under `Density::Exact` is the freeze's chart bit for bit, for every frozen
    /// grid, every rung and every control kind. The wrappers guarantee it by construction;
    /// this is the test that would catch the construction changing.
    #[test]
    fn pa4_the_frozen_chart_is_reproduced_bit_for_bit() {
        let n = 12;
        let mut s: u64 = 0xA4;
        let mut next = move || { s = s.wrapping_mul(6364136223846793005).wrapping_add(1); ((s >> 11) as f64) / ((1u64 << 53) as f64) };
        let frames: Vec<Frame> = (0..300).map(|i| {
            let pos: Vec<[f64; 3]> = (0..n).map(|_| [next() * 34.6, next() * 20.8, next() * 5.0]).collect();
            let vel: Vec<[f64; 3]> = (0..n).map(|_| [(next() - 0.5) * 4.0, (next() - 0.5) * 4.0, (next() - 0.5) * 4.0]).collect();
            frame(i as u64, pos, vel)
        }).collect();
        let traj = Trajectory { header: header(n, vec![1; n]), frames };
        for grid in FROZEN_GRIDS {
            for rung in LADDER {
                for kind in [Kind::Spatial, Kind::BlindLabel, Kind::BlindIndex, Kind::GlobalRelabel] {
                    let frozen = readings(&traj, grid, rung, kind).unwrap();
                    let amended = readings3(&traj, grid.into(), rung, kind, Density::Exact).unwrap();
                    assert_eq!(frozen, amended, "grid {grid:?} rung {rung:?} kind {kind:?}: the freeze's chart moved");
                }
            }
        }
    }

    /// PA-5 — the self-check: the exact chart REFINES the binned one on every trajectory.
    /// A violation convicts the binning, never the trajectory.
    #[test]
    fn pa5_exact_refines_poisson() {
        let traj = random_walk3(200, 30.0, 400, 0x5041_3035);
        for grid in doubling_ladder(200, 3) {
            let exact = readings3(&traj, grid, Rung::Occ, Kind::Spatial, Density::Exact).unwrap();
            let binned = readings3(&traj, grid, Rung::Occ, Kind::Spatial, Density::Poisson).unwrap();
            assert!(refines(&exact, &binned), "grid {grid:?}: exact occupancy must refine its own bin");
        }
    }

    /// PA-6 — the derived ladder's first four grids are the freeze's first four, cell for
    /// cell, at the freeze's own `N = 12`, `dims = 2`.
    #[test]
    fn pa6_the_doubling_ladder_reproduces_the_freeze() {
        let ladder = doubling_ladder(12, 2);
        let frozen: Vec<Grid3> = FROZEN_GRIDS[..4].iter().map(|g| (*g).into()).collect();
        assert_eq!(ladder, frozen, "the first four grids must be the freeze's; the fifth (6×4) is not on a doubling ladder and is kept in the frozen mode");
        // and in 3D it makes boxes, not columns
        let l3 = doubling_ladder(128, 3);
        assert_eq!(l3.len(), 8, "128 waters: 2^0 .. 2^7 cells");
        assert_eq!(l3[3], Grid3 { nx: 2, ny: 2, nz: 2 }, "the third doubling splits z");
        assert!(l3.iter().all(|g| g.nz >= 1) && l3[7].cells() == 128);
    }

    // ------------------------------------------- RUNG2_AMENDMENT_2.md's plants, PB-1..PB-4

    /// A random walk whose atoms also carry THERMAL velocities at `T_target`, so the
    /// momentum and energy rungs read something. The walk's positions and its velocities
    /// are independent streams — a carrier with no structure to close on, whose only job
    /// is to populate every field of the chart at a stated occupancy.
    ///
    /// The atoms are HYDROGEN, deliberately. The freeze's `Δp` is one hydrogen's thermal
    /// momentum, so for independent hydrogens a cell's momentum spread is `√⟨n⟩ · Δp` by
    /// the plain statistics of a sum — the scale Amendment 2 declares, with nothing else
    /// assumed. (For independent OXYGENS it would be `4×` wider; that real water's oxygen
    /// cells nonetheless fluctuate at `√⟨n⟩ · Δp` is a property of the liquid — momenta
    /// anticorrelated by conservation and carried collectively — which the FIELD reading
    /// checks and this plant does not. The first draft of this plant used independent
    /// oxygens, reached 2 informative transitions at the momentum rung, and taught that
    /// distinction the hard way.)
    fn thermal_walk3(n: usize, edge: f64, frames: usize, seed: u64) -> Trajectory {
        let mut s = seed;
        let mut next = move || {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((s >> 11) as f64) / ((1u64 << 53) as f64)
        };
        // Box–Muller for a thermal velocity component of a HYDROGEN at T_target
        let sigma_v = (K_B * T_TARGET / (H_MASS_U * M_E_PER_U)).sqrt();
        let mut gauss = move || {
            let (u1, u2) = (next().max(1e-12), next());
            (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        };
        let mut pos: Vec<[f64; 3]> = (0..n).map(|_| [gauss().abs() % 1.0 * edge, gauss().abs() % 1.0 * edge, gauss().abs() % 1.0 * edge]).collect();
        let mut out = Vec::with_capacity(frames);
        for i in 0..frames {
            let vel: Vec<[f64; 3]> = (0..n).map(|_| [gauss() * sigma_v, gauss() * sigma_v, gauss() * sigma_v]).collect();
            for p in pos.iter_mut() {
                for c in 0..3 {
                    let mut v = p[c] + gauss() * 0.02 * edge;
                    if v < 0.0 { v = -v; }
                    if v > edge { v = 2.0 * edge - v; }
                    p[c] = v.clamp(0.0, edge);
                }
            }
            out.push(frame(i as u64, pos.clone(), vel));
        }
        Trajectory { header: header3(n, vec![1; n], edge), frames: out }
    }

    /// PB-1 — the fault and the repair at the momentum rung. Under Amendment 1 alone the
    /// momentum field is binned at one atom's thermal momentum and a cell of a hundred
    /// oxygens almost never repeats a reading; under Amendment 2 the same frames collide
    /// and meet G4.
    #[test]
    fn pb1_the_momentum_rung_is_void_under_a1_and_reads_under_a2() {
        let traj = thermal_walk3(200, 30.0, 600, 0x5042_3031);
        let grid = Grid3 { nx: 2, ny: 1, nz: 1 };
        let t = transport_fraction(&cell_series3(&traj, grid, Kind::Spatial).unwrap());
        assert!(t > prereg::MIN_TRANSPORT);
        let a1 = leg_a(&readings3(&traj, grid, Rung::Mom, Kind::Spatial, Density::Poisson).unwrap());
        assert!(
            matches!(grade(true, t, &a1), Verdict::VoidNoCollisions | Verdict::VoidWorkCount(_)),
            "under A1 alone the momentum rung must be VOID by counting (collisions {}, informative {})",
            a1.collisions, a1.informative
        );
        let a2 = leg_a(&readings3(&traj, grid, Rung::Mom, Kind::Spatial, Density::CellScale).unwrap());
        assert!(a2.collisions > 0, "under A2 the momentum rung must collide");
        assert!(a2.informative >= prereg::MIN_INFORMATIVE, "and meet G4: {}", a2.informative);
        // The energy rung is one field finer and the amendment stakes nothing about its
        // work count on this carrier — only that it collides, where under A1 it did not.
        let e1 = leg_a(&readings3(&traj, grid, Rung::Ene, Kind::Spatial, Density::Poisson).unwrap());
        let e2 = leg_a(&readings3(&traj, grid, Rung::Ene, Kind::Spatial, Density::CellScale).unwrap());
        assert!(e2.collisions > e1.collisions, "the energy rung must collide more under A2 ({} vs {})", e2.collisions, e1.collisions);
    }

    /// PB-2 — the self-check: the finer chart refines the coarser at every rung. A
    /// violation convicts the scaling, never the trajectory.
    #[test]
    fn pb2_poisson_refines_cell_scale_at_every_rung() {
        let traj = thermal_walk3(200, 30.0, 400, 0x5042_3032);
        for grid in doubling_ladder(200, 3) {
            for rung in LADDER {
                let fine = readings3(&traj, grid, rung, Kind::Spatial, Density::Poisson).unwrap();
                let coarse = readings3(&traj, grid, rung, Kind::Spatial, Density::CellScale).unwrap();
                assert!(refines(&fine, &coarse), "grid {grid:?} rung {rung:?}: A1's chart must refine A2's");
            }
        }
    }

    /// PB-3 — the wider bin hides nothing: P-3's hidden variable, its atoms given thermal
    /// velocities, still fires at every rung under `CellScale`.
    #[test]
    fn pb3_a_hidden_variable_still_fires_under_cell_scale() {
        let n = 4;
        let ncell = 4;
        let cw = 34.6 / ncell as f64;
        let mut cells: Vec<usize> = (0..n).collect();
        let mut s: u64 = 0xDEAD_BEEF;
        let sigma_v = (K_B * T_TARGET / (H_MASS_U * M_E_PER_U)).sqrt();
        let mut frames = Vec::new();
        for i in 0..1200u64 {
            let pos: Vec<[f64; 3]> = cells.iter().map(|&c| [(c as f64 + 0.5) * cw, 10.0, 0.0]).collect();
            // velocities: a deterministic function of the cell, so the chart's momentum
            // field carries no more than its density field does and the hidden bit stays hidden
            let vel: Vec<[f64; 3]> = cells.iter().map(|&c| [sigma_v * (c as f64 - 1.5), 0.0, 0.0]).collect();
            frames.push(frame(i, pos, vel));
            for c in cells.iter_mut() {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                *c = if (s >> 60) & 1 == 1 { (*c + 1) % ncell } else { (*c + ncell - 1) % ncell };
            }
        }
        let traj = Trajectory { header: header(n, vec![1; n]), frames };
        let grid = Grid3 { nx: 4, ny: 1, nz: 1 };
        let t = transport_fraction(&cell_series3(&traj, grid, Kind::Spatial).unwrap());
        for rung in LADDER {
            let a = leg_a(&readings3(&traj, grid, rung, Kind::Spatial, Density::CellScale).unwrap());
            assert!(a.firing > 0, "the hidden variable must still fire at {rung:?} under CellScale");
            assert_eq!(grade(true, t, &a), Verdict::NotClosed);
        }
    }

    /// PB-4 — at the density rung the two amendments are the same rule: `CellScale` and
    /// `Poisson` readings are equal exactly.
    #[test]
    fn pb4_cell_scale_equals_poisson_at_the_density_rung() {
        let traj = thermal_walk3(128, 29.6, 300, 0x5042_3034);
        for grid in doubling_ladder(128, 3) {
            for kind in [Kind::Spatial, Kind::BlindLabel, Kind::GlobalRelabel] {
                let a1 = readings3(&traj, grid, Rung::Occ, kind, Density::Poisson).unwrap();
                let a2 = readings3(&traj, grid, Rung::Occ, kind, Density::CellScale).unwrap();
                assert_eq!(a1, a2, "grid {grid:?} kind {kind:?}: at the Occ rung A2 must equal A1");
            }
        }
    }

    // ------------------------------------------- RUNG2_AMENDMENT_3.md's plants, PC-1..PC-5

    /// `N` independent thermal OXYGENS at `T_target`, walking, with the total momentum
    /// removed every frame — the zero-sum thermal sample the formula is the statistics of.
    /// Amendment 2's plant used hydrogens to dodge a mass it had got wrong; this one uses
    /// the carrier's own species because the formula now carries the mass.
    fn zero_sum_thermal_walk3(n: usize, edge: f64, frames: usize, seed: u64) -> Trajectory {
        let mut s = seed;
        let mut next = move || {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((s >> 11) as f64) / ((1u64 << 53) as f64)
        };
        let m = O_MASS_U * M_E_PER_U;
        let sigma_v = (K_B * T_TARGET / m).sqrt();
        let mut gauss = move || {
            let (u1, u2) = (next().max(1e-12), next());
            (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
        };
        // UNIFORM initial positions, from their own stream. The first draft used
        // `|gauss| mod 1`, which is not uniform: the atoms started clumped, the walk is slow,
        // and the half-box count drifted for the whole run — plant PD-2 read a spread 2.3×
        // binomial and caught it. A carrier used to check a fluctuation formula has to be
        // stationary from frame zero.
        let mut u = seed ^ 0x9E37_79B9_7F4A_7C15;
        let mut uniform = move || {
            u = u.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((u >> 11) as f64) / ((1u64 << 53) as f64)
        };
        let mut pos: Vec<[f64; 3]> = (0..n).map(|_| [uniform() * edge, uniform() * edge, uniform() * edge]).collect();
        let mut out = Vec::with_capacity(frames);
        for i in 0..frames {
            let mut vel: Vec<[f64; 3]> = (0..n).map(|_| [gauss() * sigma_v, gauss() * sigma_v, gauss() * sigma_v]).collect();
            for c in 0..3 {
                let mean = vel.iter().map(|v| v[c]).sum::<f64>() / n as f64;
                for v in vel.iter_mut() { v[c] -= mean; }
            }
            for p in pos.iter_mut() {
                for c in 0..3 {
                    let mut v = p[c] + gauss() * 0.02 * edge;
                    if v < 0.0 { v = -v; }
                    if v > edge { v = 2.0 * edge - v; }
                    p[c] = v.clamp(0.0, edge);
                }
            }
            out.push(frame(i as u64, pos.clone(), vel));
        }
        Trajectory { header: header3(n, vec![8; n], edge), frames: out }
    }

    /// PC-1 — the formula is the physics of its own carrier: the measured spread of the
    /// cell momentum and energy on a zero-sum thermal sample lands within 15 % of the
    /// derived bins. This is the check Amendment 2 did against the wrong box, made a test.
    ///
    /// The energy spread is measured CONDITIONAL ON OCCUPANCY — with the occupancy's mean
    /// contribution `(3/2) k_B T · n_cell` removed — because occupancy is its own field of
    /// the chart and the continuous fields are resolved at their fluctuation given it. The
    /// first draft of this plant measured the raw spread and read `1.28×`: on a Poisson
    /// carrier the occupancy's wandering adds `(3/2 k_B T)² · Var(n)` to the cell energy's
    /// variance, a third of the thermal term at `⟨n⟩ = 64`. Momentum has no such term
    /// (its per-atom mean is zero), which is why it passed unconditioned.
    #[test]
    fn pc1_the_derived_bins_match_their_own_carriers_measured_spread() {
        let n = 128;
        let traj = zero_sum_thermal_walk3(n, 29.6, 2000, 0x5043_3031);
        let m = O_MASS_U * M_E_PER_U;
        let grid = Grid3 { nx: 2, ny: 1, nz: 1 };
        let cells = cell_series3(&traj, grid, Kind::Spatial).unwrap();
        let (mut px, mut ek) = (Vec::new(), Vec::new());
        for (fi, f) in traj.frames.iter().enumerate() {
            let (mut p, mut e, mut occ) = (0.0, 0.0, 0usize);
            for a in 0..n {
                if cells[fi][a] == 0 {
                    p += m * f.vel[a][0];
                    e += 0.5 * m * (f.vel[a][0].powi(2) + f.vel[a][1].powi(2) + f.vel[a][2].powi(2));
                    occ += 1;
                }
            }
            px.push(p);
            ek.push(e - 1.5 * K_B * T_TARGET * occ as f64);   // conditional on occupancy
        }
        let sd = |v: &[f64]| { let mu = v.iter().sum::<f64>() / v.len() as f64; (v.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / v.len() as f64).sqrt() };
        let (kp, ke) = derived_multiples(n as f64 / 2.0, n as f64, m);
        let (rp, re) = (sd(&px) / (kp as f64 * dp_au()), sd(&ek) / (ke as f64 * de_ha()));
        assert!((rp - 1.0).abs() < 0.15, "momentum: measured/derived = {rp:.3} (k = {kp})");
        assert!((re - 1.0).abs() < 0.15, "energy: measured/derived = {re:.3} (k = {ke})");
    }

    /// PC-2 — VOID by counting under the freeze's per-atom bin; collides and meets G4 under
    /// `Derived`, on the same frames.
    #[test]
    fn pc2_the_momentum_rung_reads_under_derived() {
        let traj = zero_sum_thermal_walk3(128, 29.6, 600, 0x5043_3032);
        let grid = Grid3 { nx: 2, ny: 1, nz: 1 };
        let t = transport_fraction(&cell_series3(&traj, grid, Kind::Spatial).unwrap());
        assert!(t > prereg::MIN_TRANSPORT);
        let a1 = leg_a(&readings3(&traj, grid, Rung::Mom, Kind::Spatial, Density::Poisson).unwrap());
        assert!(matches!(grade(true, t, &a1), Verdict::VoidNoCollisions | Verdict::VoidWorkCount(_)), "under the per-atom bin: VOID (informative {})", a1.informative);
        let a3 = leg_a(&readings3(&traj, grid, Rung::Mom, Kind::Spatial, Density::Derived).unwrap());
        assert!(a3.collisions > 0 && a3.informative >= prereg::MIN_INFORMATIVE, "under Derived: collisions {} informative {}", a3.collisions, a3.informative);
    }

    /// PC-3 — the freeze's chart refines this one at every rung and grid: the self-check.
    #[test]
    fn pc3_exact_refines_derived_at_every_rung() {
        let traj = zero_sum_thermal_walk3(128, 29.6, 400, 0x5043_3033);
        for grid in doubling_ladder(128, 3) {
            for rung in LADDER {
                let fine = readings3(&traj, grid, rung, Kind::Spatial, Density::Exact).unwrap();
                let coarse = readings3(&traj, grid, rung, Kind::Spatial, Density::Derived).unwrap();
                assert!(refines(&fine, &coarse), "grid {grid:?} rung {rung:?}: the freeze's chart must refine Amendment 3's");
            }
        }
    }

    /// PC-4 — the wider bin hides nothing: the hidden variable still fires under `Derived`.
    #[test]
    fn pc4_a_hidden_variable_still_fires_under_derived() {
        let n = 4;
        let ncell = 4;
        let cw = 34.6 / ncell as f64;
        let mut cells: Vec<usize> = (0..n).collect();
        let mut s: u64 = 0xDEAD_BEEF;
        let sigma_v = (K_B * T_TARGET / (H_MASS_U * M_E_PER_U)).sqrt();
        let mut frames = Vec::new();
        for i in 0..1200u64 {
            let pos: Vec<[f64; 3]> = cells.iter().map(|&c| [(c as f64 + 0.5) * cw, 10.0, 0.0]).collect();
            let vel: Vec<[f64; 3]> = cells.iter().map(|&c| [sigma_v * (c as f64 - 1.5), 0.0, 0.0]).collect();
            frames.push(frame(i, pos, vel));
            for c in cells.iter_mut() {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                *c = if (s >> 60) & 1 == 1 { (*c + 1) % ncell } else { (*c + ncell - 1) % ncell };
            }
        }
        let traj = Trajectory { header: header(n, vec![1; n]), frames };
        let grid = Grid3 { nx: 4, ny: 1, nz: 1 };
        let t = transport_fraction(&cell_series3(&traj, grid, Kind::Spatial).unwrap());
        for rung in LADDER {
            let a = leg_a(&readings3(&traj, grid, rung, Kind::Spatial, Density::Derived).unwrap());
            assert!(a.firing > 0 && grade(true, t, &a) == Verdict::NotClosed, "{rung:?} must still fire under Derived");
        }
    }

    /// PC-5 — at the density rung Amendment 3 IS Amendment 1.
    #[test]
    fn pc5_derived_equals_poisson_at_the_density_rung() {
        let traj = zero_sum_thermal_walk3(128, 29.6, 300, 0x5043_3035);
        for grid in doubling_ladder(128, 3) {
            let a = readings3(&traj, grid, Rung::Occ, Kind::Spatial, Density::Poisson).unwrap();
            let b = readings3(&traj, grid, Rung::Occ, Kind::Spatial, Density::Derived).unwrap();
            assert_eq!(a, b, "grid {grid:?}");
        }
    }

    // ------------------------------------------- RUNG2_AMENDMENT_4.md's plants, PD-1..PD-4

    /// PD-1 — the routes are A1 at a different scale and nothing else: with `σ = √⟨n⟩`,
    /// `Calibrated` equals `Poisson` at the density rung exactly.
    #[test]
    fn pd1_calibrated_at_the_poisson_scale_is_poisson() {
        let traj = zero_sum_thermal_walk3(128, 29.6, 300, 0x5044_3031);
        for grid in doubling_ladder(128, 3) {
            let sigma = (128.0 / grid.cells() as f64).sqrt();
            let a = readings3(&traj, grid, Rung::Occ, Kind::Spatial, Density::Poisson).unwrap();
            let b = readings3(&traj, grid, Rung::Occ, Kind::Spatial, Density::Calibrated(sigma)).unwrap();
            assert_eq!(a, b, "grid {grid:?}");
        }
    }

    /// PD-2 — on a zero-sum thermal carrier the measured `σ(n)` IS the finite-population
    /// Poisson value, so `Calibrated` at that σ reproduces `External` at `S(0) = 1`: the
    /// two routes coincide on the carrier where the liquid correction is absent.
    #[test]
    fn pd2_the_routes_coincide_where_the_liquid_correction_is_absent() {
        let traj = zero_sum_thermal_walk3(128, 29.6, 1500, 0x5044_3032);
        let grid = Grid3 { nx: 2, ny: 1, nz: 1 };
        let cs = cell_series3(&traj, grid, Kind::Spatial).unwrap();
        let sigma = occupancy_sigma(&cs, grid.cells());
        let expect = (64.0f64 * 0.5).sqrt();   // S(0) = 1 with the finite-population factor
        assert!((sigma / expect - 1.0).abs() < 0.15, "measured {sigma:.2} vs binomial {expect:.2}");
        let a = readings3(&traj, grid, Rung::Mom, Kind::Spatial, Density::Calibrated(sigma)).unwrap();
        let b = readings3(&traj, grid, Rung::Mom, Kind::Spatial, Density::Derived).unwrap();
        // the momentum and energy fields are Derived under every Amendment-4 route
        assert_eq!(a.iter().map(|r| &r[2..]).collect::<Vec<_>>(), b.iter().map(|r| &r[2..]).collect::<Vec<_>>(),
                   "momentum/energy fields must be Amendment 3's under route (ii)");
    }

    /// PD-3 — the self-check under both routes.
    #[test]
    fn pd3_exact_refines_both_routes() {
        let traj = zero_sum_thermal_walk3(128, 29.6, 400, 0x5044_3033);
        for grid in doubling_ladder(128, 3) {
            for rung in LADDER {
                let fine = readings3(&traj, grid, rung, Kind::Spatial, Density::Exact).unwrap();
                for d in [Density::External, Density::Calibrated(2.3)] {
                    let coarse = readings3(&traj, grid, rung, Kind::Spatial, d).unwrap();
                    assert!(refines(&fine, &coarse), "grid {grid:?} rung {rung:?} {d:?}");
                }
            }
        }
    }

    // ------------------------------------------------------- P-6 / P-7: the pair

    /// P-7 — must NOT fire. A GLOBAL relabelling of cell labels is a re-presentation of
    /// the same partition, so every reading must be a relabelling and every count must be
    /// bit-identical. M-PRESENTATION-VERDICT: a criterion on the chart has to be invariant
    /// under re-presentation, demonstrated on a re-presented instance.
    ///
    /// P-6 and P-7 are only meaningful as a pair: one must fire and one must not.
    #[test]
    fn p7_a_global_relabelling_moves_nothing() {
        let n = 6;
        let mut s: u64 = 7;
        let frames: Vec<Frame> = (0..800)
            .map(|i| {
                let pos: Vec<[f64; 3]> = (0..n)
                    .map(|_| {
                        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                        let u = (((s >> 32) as u32) as f64) / (u32::MAX as f64);
                        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                        let v = (((s >> 32) as u32) as f64) / (u32::MAX as f64);
                        [u * 34.6, v * 20.8, 0.0]
                    })
                    .collect();
                frame(i, pos, vec![[0.0; 3]; n])
            })
            .collect();
        let traj = Trajectory {
            header: header(n, vec![1; n]),
            frames,
        };
        let grid = Grid { nx: 2, ny: 2 };
        let a = leg_a(&readings(&traj, grid, Rung::Occ, Kind::Spatial).unwrap());
        let b = leg_a(&readings(&traj, grid, Rung::Occ, Kind::GlobalRelabel).unwrap());
        assert_eq!(a.collisions, b.collisions, "a relabelling changed the collisions");
        assert_eq!(a.firing, b.firing, "a relabelling changed the defect");
        assert_eq!(a.informative, b.informative);
        assert_eq!(a.distinct, b.distinct);
    }

    /// The freeze's LITERAL control (`Kind::BlindIndex`) assigns cells from arena index
    /// alone, so membership never changes and the chart has zero transport. That makes it
    /// unusable as a defect comparison — it VOIDs at the fence instead of scoring.
    ///
    /// This test is the witness for that finding. The stake is NOT repaired in place; the
    /// degenerate control is kept and run, and `Kind::BlindLabel` is reported beside it as
    /// an addition.
    #[test]
    fn the_frozen_index_control_is_degenerate_and_voids() {
        let n = 6;
        let mut s: u64 = 11;
        let frames: Vec<Frame> = (0..400)
            .map(|i| {
                let pos: Vec<[f64; 3]> = (0..n)
                    .map(|_| {
                        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                        let u = (((s >> 32) as u32) as f64) / (u32::MAX as f64);
                        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                        let v = (((s >> 32) as u32) as f64) / (u32::MAX as f64);
                        [u * 34.6, v * 20.8, 0.0]
                    })
                    .collect();
                frame(i, pos, vec![[0.0; 3]; n])
            })
            .collect();
        let traj = Trajectory {
            header: header(n, vec![1; n]),
            frames,
        };
        let grid = Grid { nx: 2, ny: 2 };
        let cs = cell_series(&traj, grid, Kind::BlindIndex).unwrap();
        assert_eq!(
            transport_fraction(&cs),
            0.0,
            "index-only membership cannot move, which is the defect in the freeze"
        );
        let a = leg_a(&readings(&traj, grid, Rung::Occ, Kind::BlindIndex).unwrap());
        assert!(matches!(
            grade(true, transport_fraction(&cs), &a),
            Verdict::VoidVacuous(_)
        ));
        // The repair carries the same transport as the real chart, which is what makes it
        // a control rather than a different experiment.
        let real = cell_series(&traj, grid, Kind::Spatial).unwrap();
        let blind = cell_series(&traj, grid, Kind::BlindLabel).unwrap();
        assert_eq!(
            transport_fraction(&real),
            transport_fraction(&blind),
            "the label control must preserve the transport rate exactly"
        );
    }

    /// P-6 — must FIRE the self-check. The mutation: a v2 reading that carries ONLY the
    /// momentum fields, dropping the occupancy the real v2 keeps. That is a plausible
    /// "optimisation" — momentum looks like it implies occupancy — and it breaks the
    /// refinement relation, so the ladder is no longer a ladder and G8 must catch it.
    ///
    /// Sector the plant acts on: the QUANTISATION/CHART-CONTENT sector, nonzero by
    /// construction. A planted defect that stays silent is a defect in the plant, so this
    /// is checked to fire before G8 is trusted anywhere.
    fn mutated_v2_dropping_occupancy(traj: &Trajectory, grid: Grid) -> Vec<Reading> {
        let cells = cell_series(traj, grid, Kind::Spatial).unwrap();
        let masses: Vec<f64> = traj.header.z.iter().map(|z| mass_me(*z).unwrap()).collect();
        let dp = dp_au();
        traj.frames
            .iter()
            .enumerate()
            .map(|(fi, f)| {
                let nc = grid.cells();
                let (mut px, mut py) = (vec![0.0; nc], vec![0.0; nc]);
                for a in 0..traj.header.n_atoms {
                    let c = cells[fi][a];
                    px[c] += masses[a] * f.vel[a][0];
                    py[c] += masses[a] * f.vel[a][1];
                }
                let mut r: Reading = Vec::new();
                for c in 0..nc {
                    r.push((px[c] / dp).floor() as i64);
                    r.push((py[c] / dp).floor() as i64);
                }
                r
            })
            .collect()
    }

    #[test]
    fn p6_a_ladder_that_stops_refining_fires_g8() {
        let n = 6;
        let mut s: u64 = 99;
        let frames: Vec<Frame> = (0..600)
            .map(|i| {
                let mut pos = Vec::new();
                let mut vel = Vec::new();
                for _ in 0..n {
                    s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let u = (((s >> 32) as u32) as f64) / (u32::MAX as f64);
                    s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let v = (((s >> 32) as u32) as f64) / (u32::MAX as f64);
                    pos.push([u * 34.6, v * 20.8, 0.0]);
                    // Velocities of EQUAL magnITUDE and opposite sign, sized so one atom
                    // is 2.5 momentum bins. A cell holding {+,−} then reads the same
                    // momentum as an EMPTY cell — momentum aliases occupancy, which is
                    // exactly the coincidence the mutation has to be able to exploit.
                    // Without it `refines` is vacuously true (no two frames share a
                    // momentum reading) and the plant stays silent for a reason that has
                    // nothing to do with the gate.
                    let speed = 2.5 * dp_au() / (H_MASS_U * M_E_PER_U);
                    // The sign comes from its OWN draw. Taking it from `v` made it a
                    // function of the y-coordinate, hence of the cell row, so every atom
                    // in a cell shared a sign and momentum determined occupancy exactly —
                    // the mutated chart really WAS a refinement and the plant was right to
                    // stay silent. The defect was in the carrier, not in the gate.
                    s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                    let sign = if (s >> 60) & 1 == 1 { 1.0 } else { -1.0 };
                    vel.push([sign * speed, 0.0, 0.0]);
                }
                frame(i, pos, vel)
            })
            .collect();
        let traj = Trajectory {
            header: header(n, vec![1; n]),
            frames,
        };
        let grid = Grid { nx: 2, ny: 2 };
        let r1 = readings(&traj, grid, Rung::Occ, Kind::Spatial).unwrap();
        let r2 = readings(&traj, grid, Rung::Mom, Kind::Spatial).unwrap();
        let v1 = leg_a(&r1);
        let v2 = leg_a(&r2);
        assert!(
            ladder_monotone(&[v1.collisions, v2.collisions]),
            "the UNmutated ladder must be monotone — refinement_removes_collisions"
        );
        assert!(refines(&r2, &r1), "and v2 must genuinely refine v1");

        let bad = mutated_v2_dropping_occupancy(&traj, grid);
        let bad_a = leg_a(&bad);

        // THE FINDING, kept as an assertion so it cannot quietly stop being true: the
        // frozen weak form does NOT catch this mutation. The mutated chart is not a
        // refinement of v1 at all, yet it has fewer collisions, so monotonicity holds.
        assert!(
            ladder_monotone(&[v1.collisions, bad_a.collisions]),
            "the weak form was expected to stay silent here ({} -> {})",
            v1.collisions,
            bad_a.collisions
        );
        // And the strong form fires, which is what makes G8 a gate rather than a hope.
        assert!(
            !refines(&bad, &r1),
            "PLANT SILENT: v2 with the occupancy dropped is not a refinement of v1, and \
             the strong form of G8 failed to say so"
        );
    }

    // ------------------------------------------------------------- P-5: must REFUSE

    /// P-5. Carrier: a trajectory truncated below the work count. Sector: the SAMPLE-SIZE
    /// sector, nonzero by construction. Expected: VOID naming the work count — not a pass
    /// and not a fail.
    #[test]
    fn p5_a_short_trajectory_voids_on_work_count() {
        let n = 4;
        let ncell = 4;
        let cw = 34.6 / ncell as f64;
        let frames: Vec<Frame> = (0..20)
            .map(|i| {
                let pos: Vec<[f64; 3]> = (0..n)
                    .map(|a| [((a + i as usize) % ncell) as f64 * cw + 1.0, 10.0, 0.0])
                    .collect();
                frame(i as u64, pos, vec![[0.0; 3]; n])
            })
            .collect();
        let traj = Trajectory {
            header: header(n, vec![1; n]),
            frames,
        };
        let grid = Grid { nx: 4, ny: 1 };
        let cs = cell_series(&traj, grid, Kind::Spatial).unwrap();
        let a = leg_a(&readings(&traj, grid, Rung::Occ, Kind::Spatial).unwrap());
        assert!(matches!(
            grade(true, transport_fraction(&cs), &a),
            Verdict::VoidWorkCount(_)
        ));
    }

    /// R1 — an atom outside the box refuses by frame and atom, and is never clamped into
    /// an edge cell. A clamp would manufacture occupancy exactly where the walls are.
    #[test]
    fn r1_an_atom_outside_the_box_refuses() {
        let n = 2;
        let traj = Trajectory {
            header: header(n, vec![1; n]),
            frames: vec![
                frame(0, vec![[1.0, 1.0, 0.0], [2.0, 2.0, 0.0]], vec![[0.0; 3]; n]),
                frame(1, vec![[1.0, 1.0, 0.0], [99.0, 2.0, 0.0]], vec![[0.0; 3]; n]),
            ],
        };
        match cell_series(&traj, Grid { nx: 2, ny: 2 }, Kind::Spatial) {
            Err(Refusal::AtomOutsideBox { frame, atom, .. }) => {
                assert_eq!((frame, atom), (1, 1));
            }
            other => panic!("expected a refusal naming the frame and atom, got {other:?}"),
        }
    }

    /// THE CARRIER GUARD, and it is here because the carriers were wrong.
    ///
    /// Three plants above draw positions from `s >> 33`, which on a `u64` is a 31-bit
    /// value; divided by `u32::MAX` it never exceeds 0.5, so every atom sat in cell 0 and
    /// the "random" scenes had exactly ONE distinct occupancy reading. P-6 stayed silent
    /// for that reason and not for any reason about the gate, and P-7 and the degenerate-
    /// control test were passing on a scene that could not have failed them.
    ///
    /// An asserted zero has to be a fact about the SCENE, not about the instrument's
    /// coverage. This test makes the carriers assert their own coverage.
    #[test]
    fn the_random_carriers_actually_populate_the_grid() {
        let n = 6;
        let mut s: u64 = 11;
        let frames: Vec<Frame> = (0..400)
            .map(|i| {
                let pos: Vec<[f64; 3]> = (0..n)
                    .map(|_| {
                        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                        let u = (((s >> 32) as u32) as f64) / (u32::MAX as f64);
                        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                        let v = (((s >> 32) as u32) as f64) / (u32::MAX as f64);
                        [u * 34.6, v * 20.8, 0.0]
                    })
                    .collect();
                frame(i, pos, vec![[0.0; 3]; n])
            })
            .collect();
        let traj = Trajectory {
            header: header(n, vec![1; n]),
            frames,
        };
        let grid = Grid { nx: 2, ny: 2 };
        let cs = cell_series(&traj, grid, Kind::Spatial).unwrap();
        let mut hit = vec![false; grid.cells()];
        for row in &cs {
            for &c in row {
                hit[c] = true;
            }
        }
        assert!(hit.iter().all(|&h| h), "a carrier must reach every cell: {hit:?}");
        let r = readings(&traj, grid, Rung::Occ, Kind::Spatial).unwrap();
        let distinct = r.iter().collect::<std::collections::HashSet<_>>().len();
        assert!(
            distinct > 1,
            "a carrier with ONE distinct occupancy reading cannot fail any plant"
        );
    }

    /// The closed-form collision count must equal the brute-force one. This is the
    /// arithmetic the whole instrument rests on — a coarse chart carries 10^8 collisions
    /// and they are counted, never enumerated — so it is checked against enumeration on a
    /// small case where enumeration is possible.
    #[test]
    fn closed_form_collision_count_matches_enumeration() {
        let r: Vec<Reading> = vec![
            vec![1], vec![2], vec![1], vec![1], vec![2], vec![3], vec![1], vec![2],
        ];
        let a = leg_a(&r);
        let last = r.len() - 1;
        let (mut coll, mut fire) = (0u128, 0u128);
        for i in 0..last {
            for j in (i + 1)..last {
                if r[i] == r[j] {
                    coll += 1;
                    if r[i + 1] != r[j + 1] {
                        fire += 1;
                    }
                }
            }
        }
        assert_eq!(a.collisions, coll, "closed-form collisions");
        assert_eq!(a.firing, fire, "closed-form firings");
    }

    /// Leg B must not be able to score itself: a law built on the first half and applied
    /// to it would read zero by construction. The split is checked to be a real one.
    #[test]
    fn leg_b_is_held_out() {
        // First half: A→B always. Second half: A→C always. A law fitted on the first half
        // must be WRONG on the second, and a leg that scored in-sample would read zero.
        let mut r: Vec<Reading> = Vec::new();
        for _ in 0..100 {
            r.push(vec![0]);
            r.push(vec![1]);
        }
        for _ in 0..100 {
            r.push(vec![0]);
            r.push(vec![2]);
        }
        let b = leg_b(&r);
        assert!(b.predicted > 0, "the second half must be covered");
        assert!(
            b.defect().unwrap() > 0.4,
            "a law fitted on the first half must fail on a second half that changed; got {:?}",
            b.defect()
        );
    }
}
