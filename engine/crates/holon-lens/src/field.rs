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
    /// Amendment 5, A5.3: every field binned at its own HELD-OUT fluctuation of the
    /// WINDOW-AVERAGED field — absolute per-cell bins for occupancy, momentum (per
    /// component) and kinetic energy, measured on other seeds at the same grid and window.
    /// Only meaningful through [`readings_windowed`]; under [`readings3`] it is refused.
    Calibrated3 { n: f64, p: f64, e: f64 },
}

/// Water's speed of sound at 25 °C, CRC: `1497 m s⁻¹`, in bohr per femtosecond. An external
/// protocol constant (RUNG2_AMENDMENT_5.md A5.1), entering as `T_TARGET` and `WATER_S0` do.
pub const WATER_CS_BOHR_PER_FS: f64 = 1497.0 * 1e-15 / 5.29177210903e-11;

/// A5.1 — the cadence of a chart: its smallest cell edge over the sound speed, in fs.
pub fn cadence_fs(traj: &Trajectory, grid: Grid3) -> f64 {
    let h = &traj.header;
    let mut a = h.box_w / grid.nx as f64;
    a = a.min(h.box_h / grid.ny as f64);
    if grid.nz > 1 {
        a = a.min(h.box_d / grid.nz as f64);
    }
    a / WATER_CS_BOHR_PER_FS
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

/// One frame's fields over the cells, before any binning (RUNG2_AMENDMENT_5.md A5.2): the
/// quantities the chart averages over its window. Occupancy is `f64` because an average of
/// integers is not one.
#[derive(Clone, Debug, PartialEq)]
pub struct CellFields {
    /// `cells × species`, species in ascending nuclear charge.
    pub occ: Vec<f64>,
    /// per cell, three components; the freeze read two and a `dims = 2` chart still does.
    pub p: Vec<[f64; 3]>,
    /// per cell, kinetic only (PREREG §2.2 refuses the potential as a field).
    pub ek: Vec<f64>,
}

/// The instantaneous fields of every frame.
pub fn fields3(traj: &Trajectory, grid: Grid3, kind: Kind) -> Result<Vec<CellFields>, Refusal> {
    let cells = cell_series3(traj, grid, kind)?;
    let n = traj.header.n_atoms;
    let nc = grid.cells();
    let mut species: Vec<u32> = traj.header.z.clone();
    species.sort_unstable();
    species.dedup();
    let masses: Vec<f64> = traj.header.z.iter().map(|z| mass_me(*z)).collect::<Result<_, _>>()?;
    let mut out = Vec::with_capacity(traj.frames.len());
    for (fi, f) in traj.frames.iter().enumerate() {
        let mut occ = vec![0.0f64; nc * species.len()];
        let mut p = vec![[0.0f64; 3]; nc];
        let mut ek = vec![0.0f64; nc];
        for a in 0..n {
            let c = cells[fi][a];
            let si = species.iter().position(|z| *z == traj.header.z[a]).unwrap();
            occ[c * species.len() + si] += 1.0;
            let m = masses[a];
            for k in 0..3 {
                p[c][k] += m * f.vel[a][k];
            }
            ek[c] += 0.5 * m * (f.vel[a][0] * f.vel[a][0] + f.vel[a][1] * f.vel[a][1] + f.vel[a][2] * f.vel[a][2]);
        }
        out.push(CellFields { occ, p, ek });
    }
    Ok(out)
}

/// A5.2 — non-overlapping windows of `w` frames, each field averaged; a trailing partial
/// window is dropped, never padded.
pub fn window_mean(fields: &[CellFields], w: usize) -> Vec<CellFields> {
    let w = w.max(1);
    let mut out = Vec::with_capacity(fields.len() / w);
    for block in fields.chunks_exact(w) {
        let mut acc = CellFields { occ: vec![0.0; block[0].occ.len()], p: vec![[0.0; 3]; block[0].p.len()], ek: vec![0.0; block[0].ek.len()] };
        for f in block {
            for (a, b) in acc.occ.iter_mut().zip(&f.occ) { *a += b; }
            for (a, b) in acc.p.iter_mut().zip(&f.p) { for k in 0..3 { a[k] += b[k]; } }
            for (a, b) in acc.ek.iter_mut().zip(&f.ek) { *a += b; }
        }
        let inv = 1.0 / w as f64;
        for a in acc.occ.iter_mut() { *a *= inv; }
        for a in acc.p.iter_mut() { for k in 0..3 { a[k] *= inv; } }
        for a in acc.ek.iter_mut() { *a *= inv; }
        out.push(acc);
    }
    out
}

/// The pooled standard deviations of the three fields over cells and frames — what A5.3
/// calibrates on, measured on trajectories OTHER than the one graded. Momentum is pooled
/// over the components a `dims`-dimensional chart reads.
pub fn field_sigmas(fields: &[CellFields], components: usize) -> (f64, f64, f64) {
    let sd = |v: &[f64]| -> f64 {
        if v.len() < 2 { return 0.0; }
        let m = v.iter().sum::<f64>() / v.len() as f64;
        (v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / v.len() as f64).sqrt()
    };
    // occupancy: deviation from each cell-species' own mean, pooled
    let ns = fields.first().map(|f| f.occ.len()).unwrap_or(0);
    let mut dn = Vec::new();
    for k in 0..ns {
        let col: Vec<f64> = fields.iter().map(|f| f.occ[k]).collect();
        let m = col.iter().sum::<f64>() / col.len().max(1) as f64;
        dn.extend(col.iter().map(|x| x - m));
    }
    let p: Vec<f64> = fields.iter().flat_map(|f| f.p.iter().flat_map(move |c| c[..components].to_vec())).collect();
    let nc = fields.first().map(|f| f.ek.len()).unwrap_or(0);
    let mut de = Vec::new();
    for c in 0..nc {
        let col: Vec<f64> = fields.iter().map(|f| f.ek[c]).collect();
        let m = col.iter().sum::<f64>() / col.len().max(1) as f64;
        de.extend(col.iter().map(|x| x - m));
    }
    (sd(&dn), sd(&p), sd(&de))
}

/// A5 — the chart read at a cadence: fields averaged over `window` frames, then binned.
/// `window = 1` with a two-component grid is [`readings3`] bit for bit (plant PE-1); on
/// `n_z > 1` the third momentum component is read (A5.4). Bins: `Calibrated3` uses its
/// absolute held-out σ per field; every other `Density` uses the same bins `readings3`
/// would — which for an averaged field are too wide by `~√window`, so the driver prints
/// them only as the instantaneous control.
pub fn readings_windowed(
    traj: &Trajectory,
    grid: Grid3,
    rung: Rung,
    kind: Kind,
    density: Density,
    window: usize,
) -> Result<Vec<Reading>, Refusal> {
    let fields = window_mean(&fields3(traj, grid, kind)?, window);
    let n = traj.header.n_atoms;
    let nc = grid.cells();
    let mut species: Vec<u32> = traj.header.z.clone();
    species.sort_unstable();
    species.dedup();
    let masses: Vec<f64> = traj.header.z.iter().map(|z| mass_me(*z)).collect::<Result<_, _>>()?;
    let comps = if grid.nz > 1 { 3 } else { 2 };
    let (dn, dp, de): (Vec<f64>, f64, f64) = match density {
        Density::Calibrated3 { n: sn, p: sp, e: se } => (vec![sn.max(1e-12); species.len()], sp.max(1e-12), se.max(1e-12)),
        _ => {
            let dn: Vec<f64> = species
                .iter()
                .map(|z| {
                    let n_s = traj.header.z.iter().filter(|q| *q == z).count() as f64;
                    let n_bar = n_s / (nc as f64);
                    match density {
                        Density::Exact => 1.0,
                        Density::External => (WATER_S0 * n_bar * (1.0 - n_bar / n_s).max(0.0)).sqrt().max(1.0),
                        Density::Calibrated(sigma) => (sigma * (n_s / n as f64).sqrt()).max(1.0),
                        _ => n_bar.sqrt().max(1.0),
                    }
                })
                .collect();
            let (kp, ke) = match density {
                Density::CellScale => { let k = ((n as f64) / (nc as f64)).sqrt().round().max(1.0) as usize; (k, k) }
                Density::Derived | Density::External | Density::Calibrated(_) => {
                    let m_bar = masses.iter().sum::<f64>() / (n as f64);
                    derived_multiples((n as f64) / (nc as f64), n as f64, m_bar)
                }
                _ => (1, 1),
            };
            (dn, dp_au() * kp as f64, de_ha() * ke as f64)
        }
    };
    let mut out = Vec::with_capacity(fields.len());
    for f in &fields {
        let mut r: Reading = f.occ.iter().enumerate().map(|(k, o)| (o / dn[k % species.len()]).floor() as i64).collect();
        if rung >= Rung::Mom {
            for c in 0..nc {
                for k in 0..comps {
                    r.push((f.p[c][k] / dp).floor() as i64);
                }
            }
        }
        if rung >= Rung::Ene {
            for c in 0..nc {
                r.push((f.ek[c] / de).floor() as i64);
            }
        }
        out.push(r);
    }
    Ok(out)
}

/// RUNG2_AMENDMENT_6.md — the continuity leg. For consecutive window-averaged fields `k`
/// and `k+1` of a chart of `grid` cells on a periodic box, the finite-volume prediction of
/// each cell's occupancy change from the momentum crossing its faces, against the observed
/// change, as an RMS ratio: `0` means the momentum field accounts for every particle that
/// changed cells, `1` means it accounts for none.
///
/// `Δn_c^pred = −(τ/m̄) Σ_faces (P̄_face · n̂) / a_axis`, the face momentum the mean of the two
/// adjacent cells' momenta at the two windows' midpoint (`½(P(k) + P(k+1))`, then `½` across
/// the face) — the standard finite-volume closure, no parameter. Faces exist only on axes
/// with more than one cell (a `dims = 2` chart sums four, PF-4); periodic faces wrap.
pub struct Continuity {
    pub windows_compared: usize,
    pub rms_observed: f64,
    pub rms_residual: f64,
}

impl Continuity {
    pub fn defect(&self) -> Option<f64> {
        if self.rms_observed > 0.0 { Some(self.rms_residual / self.rms_observed) } else { None }
    }
}

/// Whether the finite-volume closure can carry information on this grid. On an axis of
/// exactly TWO periodic cells the `+` and `−` faces border the same neighbour and a central
/// face flux cancels identically — the leg reads `D_cont = 1` on every carrier, including
/// exact advection (plant PF-1 found it). The leg needs at least three cells on every split
/// axis, and a grid that does not have them is REFUSED rather than read.
pub fn continuity_admits(grid: Grid3) -> Result<(), &'static str> {
    for n in [grid.nx, grid.ny, grid.nz] {
        if n == 2 { return Err("an axis of two periodic cells: opposite faces border the same cell and the central flux cancels identically"); }
    }
    if [grid.nx, grid.ny, grid.nz].iter().all(|&n| n < 2) { return Err("no split axis: no faces"); }
    Ok(())
}

pub fn continuity(fields: &[CellFields], grid: Grid3, box_edges: [f64; 3], m_bar: f64, tau_au: f64) -> Continuity {
    let nc = grid.cells();
    let dims = [grid.nx, grid.ny, grid.nz];
    let edge = [box_edges[0] / grid.nx as f64, box_edges[1] / grid.ny as f64, box_edges[2] / grid.nz as f64];
    let idx = |ix: usize, iy: usize, iz: usize| (iz * grid.ny + iy) * grid.nx + ix;
    let coords = |c: usize| (c % grid.nx, (c / grid.nx) % grid.ny, c / (grid.nx * grid.ny));
    let (mut so, mut sr, mut n) = (0.0f64, 0.0f64, 0usize);
    for k in 0..fields.len().saturating_sub(1) {
        let (a, b) = (&fields[k], &fields[k + 1]);
        let ns = a.occ.len() / nc.max(1);
        for c in 0..nc {
            // total occupancy over species: continuity is about particles, not species
            let obs: f64 = (0..ns).map(|s| b.occ[c * ns + s] - a.occ[c * ns + s]).sum();
            let (ix, iy, iz) = coords(c);
            let mut flux = 0.0;
            for axis in 0..3 {
                if dims[axis] < 2 { continue; }
                for dir in [-1i64, 1i64] {
                    let mut q = [ix as i64, iy as i64, iz as i64];
                    q[axis] = (q[axis] + dir).rem_euclid(dims[axis] as i64);
                    let d = idx(q[0] as usize, q[1] as usize, q[2] as usize);
                    let p_mid_c = 0.5 * (a.p[c][axis] + b.p[c][axis]);
                    let p_mid_d = 0.5 * (a.p[d][axis] + b.p[d][axis]);
                    let p_face = 0.5 * (p_mid_c + p_mid_d);
                    flux += (dir as f64) * p_face / edge[axis];
                }
            }
            let pred = -(tau_au / m_bar) * flux;
            so += obs * obs;
            sr += (obs - pred) * (obs - pred);
            n += 1;
        }
    }
    let nf = n.max(1) as f64;
    Continuity { windows_compared: fields.len().saturating_sub(1), rms_observed: (so / nf).sqrt(), rms_residual: (sr / nf).sqrt() }
}

// ------------------------------------------------------------ RESPONSE1_PREREG.md's modes
//
// The density and transverse-current modes at `k = 2π/L` along x, one number per readout,
// read from the oxygens' positions and velocities; and the fit of a mode's time series to
// a relaxation, classified overdamped or underdamped by the data and not by assumption.

/// `ρ_k(t) = Σ_j cos(k x_j)` and `j_k(t) = Σ_j v_{axis,j} cos(k x_j)` per frame, for the
/// chart's atoms (or, with `perm`, a position-blind relabelling — plant R4).
pub fn modes(traj: &Trajectory, axis: usize, kind: Kind) -> Result<(Vec<f64>, Vec<f64>), Refusal> {
    let l = traj.header.box_w;
    let k = 2.0 * std::f64::consts::PI / l;
    let n = traj.header.n_atoms;
    // BlindLabel: the same per-atom scramble the charts use, here as a per-atom sign so
    // the mode is read on a partition with no spatial meaning
    let signs: Vec<f64> = match kind {
        Kind::Spatial => vec![1.0; n],
        _ => {
            let perms = label_perms(n, 2, CONTROL_SEED, true);
            (0..n).map(|a| if perms[a][0] == 0 { 1.0 } else { -1.0 }).collect()
        }
    };
    let mut rho = Vec::with_capacity(traj.frames.len());
    let mut cur = Vec::with_capacity(traj.frames.len());
    for f in &traj.frames {
        let (mut r, mut c) = (0.0, 0.0);
        for a in 0..n {
            let x = f.pos[a][0].rem_euclid(l);
            let w = signs[a] * (k * x).cos();
            r += w;
            c += w * f.vel[a][axis];
        }
        rho.push(r);
        cur.push(c);
    }
    Ok((rho, cur))
}

/// Both quadratures of both modes at `k = 2π/L` along x (RESPONSE1_AMENDMENT_1 A1): the
/// kick of §2 is a `sin(k x)` velocity mode, so the DRIVEN current mode is `cur_sin` and the
/// density's response to it is `rho_cos` (`∂_t ρ = −ρ₀ ∂_x u`); `cur_cos` and `rho_sin` are
/// the undriven quadratures, read as the null R4′. [`modes`] is the cos pair of this.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Modes {
    pub rho_cos: Vec<f64>,
    pub rho_sin: Vec<f64>,
    pub cur_cos: Vec<f64>,
    pub cur_sin: Vec<f64>,
}

/// Every quadrature of the two modes per frame; see [`Modes`].
pub fn modes_both(traj: &Trajectory, axis: usize, kind: Kind) -> Result<Modes, Refusal> {
    let l = traj.header.box_w;
    let k = 2.0 * std::f64::consts::PI / l;
    let n = traj.header.n_atoms;
    let signs: Vec<f64> = match kind {
        Kind::Spatial => vec![1.0; n],
        _ => {
            let perms = label_perms(n, 2, CONTROL_SEED, true);
            (0..n).map(|a| if perms[a][0] == 0 { 1.0 } else { -1.0 }).collect()
        }
    };
    let mut m = Modes::default();
    for f in &traj.frames {
        let (mut rc, mut rs, mut cc, mut cs) = (0.0, 0.0, 0.0, 0.0);
        for a in 0..n {
            let x = f.pos[a][0].rem_euclid(l);
            let (sn, co) = (k * x).sin_cos();
            let v = f.vel[a][axis];
            rc += signs[a] * co; rs += signs[a] * sn;
            cc += signs[a] * co * v; cs += signs[a] * sn * v;
        }
        m.rho_cos.push(rc); m.rho_sin.push(rs); m.cur_cos.push(cc); m.cur_sin.push(cs);
    }
    Ok(m)
}

/// The sign-aligned average of a mode over a run's kick cycles (RESPONSE1_AMENDMENT_1 A2).
#[derive(Clone, Debug, PartialEq)]
pub struct Aligned {
    /// the aligned mean, one value per readout of a cycle, baseline the tail mean
    pub mean: Vec<f64>,
    /// each cycle's sign-aligned, baseline-subtracted segment (the same length)
    pub per_cycle: Vec<Vec<f64>>,
    /// the aligned mean's noise: the SD of its relaxed tail (the last quarter)
    pub noise: f64,
    /// the mean of the per-cycle tail SDs — a single cycle's noise
    pub noise_per_cycle: f64,
    /// each cycle's relaxed-tail mean in units of its standard error: a cycle whose tail
    /// mean is beyond 3 has NOT relaxed by its end (the smoke's 600-fs cycles at 128 waters
    /// read −0.3 of the kick in the tail, a rebound), and the reader says so
    pub tail_sigma: Vec<f64>,
    /// the aligned mean's tail mean in units of its standard error
    pub tail_sigma_mean: f64,
}

/// Align `cycles` cycles of `relax` readouts of `series`, the first cycle's first readout at
/// index `first` (the walk's row 0 is the pre-kick state, so `first = 1`), cycle `c` carrying
/// the sign `(−1)^c` of §2's sign flip. The baseline is the PHYSICAL zero — both modes have
/// zero mean at equilibrium (uniform density; zero total momentum with velocities
/// uncorrelated with positions) — and each cycle's relaxed-tail mean is reported in units
/// of its standard error as the relaxation check, not subtracted (Amendment 1's correction
/// on building: the smoke's tails were NOT relaxed, and subtracting them inflated the kick
/// amplitude by 40 %). Cycles that do not fit in the series are dropped.
pub fn align_cycles(series: &[f64], cycles: usize, relax: usize, first: usize) -> Aligned {
    let mut per_cycle: Vec<Vec<f64>> = Vec::new();
    let mut sds = Vec::new();
    let mut tail_sigma = Vec::new();
    let tail_stats = |tail: &[f64]| -> (f64, f64) {
        let m = tail.iter().sum::<f64>() / tail.len() as f64;
        let sd = (tail.iter().map(|v| (v - m).powi(2)).sum::<f64>() / tail.len() as f64).sqrt();
        (m, sd)
    };
    for c in 0..cycles {
        let start = first + c * relax;
        let end = start + relax;
        if end > series.len() || relax < 8 { break; }
        let sign = if c % 2 == 0 { 1.0 } else { -1.0 };
        let (m, sd) = tail_stats(&series[end - relax / 4..end]);
        sds.push(sd);
        tail_sigma.push(m.abs() / (sd / ((relax / 4) as f64).sqrt()).max(1e-300));
        per_cycle.push(series[start..end].iter().map(|v| sign * v).collect());
    }
    let n = per_cycle.len();
    let mean: Vec<f64> = if n == 0 { Vec::new() } else { (0..relax).map(|i| per_cycle.iter().map(|c| c[i]).sum::<f64>() / n as f64).collect() };
    let (noise, tail_sigma_mean) = if n == 0 { (f64::NAN, f64::NAN) } else {
        let (m, sd) = tail_stats(&mean[relax - relax / 4..]);
        (sd, m.abs() / (sd / ((relax / 4) as f64).sqrt()).max(1e-300))
    };
    let noise_per_cycle = if n == 0 { f64::NAN } else { sds.iter().sum::<f64>() / n as f64 };
    Aligned { mean, per_cycle, noise, noise_per_cycle, tail_sigma, tail_sigma_mean }
}

/// The sign-aligned average of the FIELDS over a run's kick cycles (RESPONSE1_AMENDMENT_2
/// A2): each cell's occupancy departure from its time mean and its momentum, multiplied
/// by the cycle's sign `(−1)^c` and averaged over the cycles that fit; the time-mean
/// occupancy is added back so the result is a physical occupancy series of one cycle's
/// length. Continuity is linear, so the averaged fields obey it iff each cycle's do, and
/// the shot noise falls as `1/√C`. `first` is the index of the first cycle's first readout.
pub fn align_fields(fields: &[CellFields], cycles: usize, relax: usize, first: usize) -> (Vec<CellFields>, usize) {
    let nc = fields.first().map(|f| f.occ.len()).unwrap_or(0);
    let np = fields.first().map(|f| f.p.len()).unwrap_or(0);
    let mut mean_occ = vec![0.0f64; nc];
    for f in fields { for (m, o) in mean_occ.iter_mut().zip(&f.occ) { *m += o; } }
    for m in mean_occ.iter_mut() { *m /= fields.len().max(1) as f64; }
    let mut out: Vec<CellFields> = (0..relax).map(|_| CellFields { occ: vec![0.0; nc], p: vec![[0.0; 3]; np], ek: vec![0.0; np] }).collect();
    let mut used = 0usize;
    for c in 0..cycles {
        let start = first + c * relax;
        if start + relax > fields.len() { break; }
        let sign = if c % 2 == 0 { 1.0 } else { -1.0 };
        for i in 0..relax {
            let f = &fields[start + i];
            for (a, (o, m)) in out[i].occ.iter_mut().zip(f.occ.iter().zip(&mean_occ)) { *a += sign * (o - m); }
            for (a, b) in out[i].p.iter_mut().zip(&f.p) { for k in 0..3 { a[k] += sign * b[k]; } }
            for (a, b) in out[i].ek.iter_mut().zip(&f.ek) { *a += b; }
        }
        used += 1;
    }
    let inv = 1.0 / used.max(1) as f64;
    for o in out.iter_mut() {
        for (a, m) in o.occ.iter_mut().zip(&mean_occ) { *a = *a * inv + m; }
        for a in o.p.iter_mut() { for k in 0..3 { a[k] *= inv; } }
        for a in o.ek.iter_mut() { *a *= inv; }
    }
    (out, used)
}

/// The driven continuity read's signal-to-noise and floor (RESPONSE1_AMENDMENT_2 A2), on
/// the INSTANTANEOUS fields with the integral form's own differencing: the RMS over cells
/// of the occupancy change across each of the first `lead` windows of `w` readouts (signal
/// plus noise) against the same over the last `lead` windows (relaxed: noise), and
/// `s = √(lead²/tail² − 1)`, `D_floor = 1/√(1 + s²)`. Returns `(s, floor)`; NaN with fewer
/// than `2·lead + 1` windows.
pub fn driven_floor(fields: &[CellFields], w: usize, lead: usize) -> (f64, f64) {
    let w = w.max(1);
    let windows = fields.len().saturating_sub(1) / w;
    if windows < 2 * lead + 1 || lead == 0 { return (f64::NAN, f64::NAN); }
    let rms = |range: std::ops::Range<usize>| -> f64 {
        let (mut s, mut n) = (0.0, 0usize);
        for k in range {
            let (a, b) = (&fields[k * w], &fields[(k + 1) * w]);
            let nc = a.p.len().max(1);
            let per = a.occ.len() / nc;
            for c in 0..nc {
                let d: f64 = (0..per).map(|sp| b.occ[c * per + sp] - a.occ[c * per + sp]).sum();
                s += d * d; n += 1;
            }
        }
        (s / n.max(1) as f64).sqrt()
    };
    let sig = rms(0..lead);
    let noise = rms((windows - lead)..windows).max(1e-300);
    let s = (sig * sig / (noise * noise) - 1.0).max(0.0).sqrt();
    (s, 1.0 / (1.0 + s * s).sqrt())
}

/// The driven read's signal-to-noise from the PLACEBO (RESPONSE1_AMENDMENT_2 A2, as read on
/// the arms): the position-blind partition sees the same molecules with scrambled labels, so
/// its observed occupancy changes over the same windows are the shot noise with no coherent
/// part; `s = √(rms_obs(spatial)² / rms_obs(blind)² − 1)`. Independent of whether the cycle's
/// tail has relaxed, which the eight-window cycles' has not (16 % of the peak remains).
pub fn driven_floor_from_blind(spatial: &Continuity, blind: &Continuity) -> (f64, f64) {
    let r = spatial.rms_observed / blind.rms_observed.max(1e-300);
    let s = (r * r - 1.0).max(0.0).sqrt();
    (s, 1.0 / (1.0 + s * s).sqrt())
}

/// The continuity leg in its INTEGRAL form (RESPONSE1_AMENDMENT_2 A5): the occupancy
/// difference between two readouts `w` apart against the time-INTEGRATED face flux over
/// every readout between them (trapezoid in time, the same midpoint face interpolation in
/// space). The window-mean form of [`continuity`] assumes the fields vary slowly over a
/// window; a driven density transient faster than the cadence (the overdamped rise,
/// `~40` fs against `τ = 392` fs at 432 waters) breaks that assumption and reads `D > 1`
/// on an EXACT fluid (plant PR-12). The integral form has no temporal discretisation error
/// beyond the readout spacing and keeps the spatial floor `1 − sinc(π/n) cos(π/n)`.
pub fn continuity_integral(fields: &[CellFields], grid: Grid3, box_edges: [f64; 3], m_bar: f64, dt_au: f64, w: usize) -> Continuity {
    let nc = grid.cells();
    let dims = [grid.nx, grid.ny, grid.nz];
    let edge = [box_edges[0] / grid.nx as f64, box_edges[1] / grid.ny as f64, box_edges[2] / grid.nz as f64];
    let idx = |ix: usize, iy: usize, iz: usize| (iz * grid.ny + iy) * grid.nx + ix;
    let coords = |c: usize| (c % grid.nx, (c / grid.nx) % grid.ny, c / (grid.nx * grid.ny));
    let w = w.max(1);
    let (mut so, mut sr, mut n, mut windows) = (0.0f64, 0.0f64, 0usize, 0usize);
    let mut k = 0;
    while k + w < fields.len() {
        let (a, b) = (&fields[k], &fields[k + w]);
        let ns = a.occ.len() / nc.max(1);
        for c in 0..nc {
            let obs: f64 = (0..ns).map(|s| b.occ[c * ns + s] - a.occ[c * ns + s]).sum();
            let (ix, iy, iz) = coords(c);
            let mut integral = 0.0;
            for i in k..k + w {
                let (f0, f1) = (&fields[i], &fields[i + 1]);
                for axis in 0..3 {
                    if dims[axis] < 2 { continue; }
                    for dir in [-1i64, 1i64] {
                        let mut q = [ix as i64, iy as i64, iz as i64];
                        q[axis] = (q[axis] + dir).rem_euclid(dims[axis] as i64);
                        let d = idx(q[0] as usize, q[1] as usize, q[2] as usize);
                        let p_face_0 = 0.5 * (f0.p[c][axis] + f0.p[d][axis]);
                        let p_face_1 = 0.5 * (f1.p[c][axis] + f1.p[d][axis]);
                        integral += (dir as f64) * 0.5 * (p_face_0 + p_face_1) / edge[axis];
                    }
                }
            }
            let pred = -(dt_au / m_bar) * integral;
            so += obs * obs;
            sr += (obs - pred) * (obs - pred);
            n += 1;
        }
        windows += 1;
        k += w;
    }
    let nf = n.max(1) as f64;
    Continuity { windows_compared: windows, rms_observed: (so / nf).sqrt(), rms_residual: (sr / nf).sqrt() }
}

/// The midpoint continuity law's own floor on a single mode at `k = 2π/L` read on `n` cells
/// along the wave (RESPONSE1_AMENDMENT_2 A4): the cell average of the mode is the mode times
/// `sinc(π/n)`, the face value interpolated as the mean of two neighbours is the face's
/// value times `cos(π/n)`, so the predicted flux is `sinc(π/n) cos(π/n)` of the true one and
/// `D_cont = 1 − sinc(π/n) cos(π/n)` at zero noise: `0.363` at 4 cells, `0.100` at 8, `0.026`
/// at 16. Plant PR-12 reads it back on the exact continuum wave.
pub fn continuity_spatial_floor(n_cells_along_wave: usize) -> f64 {
    let a = std::f64::consts::PI / n_cells_along_wave.max(1) as f64;
    1.0 - (a.sin() / a) * a.cos()
}

/// A density mode that starts at zero and rises before it decays (RESPONSE1_AMENDMENT_1 A3).
#[derive(Clone, Debug, PartialEq)]
pub struct RiseDecay {
    /// index of the peak of `|y|`, where the slow fit begins
    pub peak: usize,
    /// the fit from the peak, the prereg's exponential, `lambda` = `λ₁`
    pub slow: Relaxation,
    /// `λ₂` from the two-exponential form `A (e^{−λ₁t} − e^{−λ₂t})`, when the rise is
    /// resolved (three or more readouts before the peak) and the fit converged
    pub fast: Option<f64>,
}

/// Fit `y` from its peak (A3). The classifier of [`fit_relaxation`] is not applied to the
/// rise: the segment handed to it begins at the peak. With the rise resolved, Gauss–Newton on
/// `A (e^{−λ₁t} − e^{−λ₂t})` over the whole segment, seeded from the peak fit, gives `λ₂`.
pub fn fit_rise_decay(y: &[f64], dt: f64, noise: f64) -> RiseDecay {
    let half = (y.len() / 2).max(1);
    let peak = (0..half).fold(0usize, |b, i| if y[i].abs() > y[b].abs() { i } else { b });
    let slow = fit_relaxation(&y[peak..], dt, noise);
    let fast = match (&slow, peak >= 3) {
        (Relaxation::Overdamped { lambda, amplitude, .. }, true) => {
            let (mut l1, mut l2, mut a) = (*lambda, 1.0 / (peak as f64 * dt), *amplitude);
            let ts: Vec<f64> = (0..y.len()).map(|i| i as f64 * dt).collect();
            let mut ok = false;
            for _ in 0..50 {
                let (mut jtj, mut jtr) = ([[0.0f64; 3]; 3], [0.0f64; 3]);
                for (i, &t) in ts.iter().enumerate() {
                    let (e1, e2) = ((-l1 * t).exp(), (-l2 * t).exp());
                    let r = y[i] - a * (e1 - e2);
                    let j = [e1 - e2, -a * t * e1, a * t * e2];
                    for p in 0..3 { jtr[p] += j[p] * r; for q in 0..3 { jtj[p][q] += j[p] * j[q]; } }
                }
                let Some(d) = solve3(jtj, jtr) else { break };
                a += d[0]; l1 += d[1]; l2 += d[2];
                if d.iter().zip([a, l1, l2]).all(|(x, v)| x.abs() < 1e-10 * v.abs().max(1e-300)) { ok = true; break; }
            }
            if ok && l2 > l1 && l1 > 0.0 { Some(l2) } else { None }
        }
        _ => None,
    };
    RiseDecay { peak, slow, fast }
}

fn solve3(m: [[f64; 3]; 3], b: [f64; 3]) -> Option<[f64; 3]> {
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if det.abs() < 1e-300 || !det.is_finite() { return None; }
    let mut x = [0.0; 3];
    for c in 0..3 {
        let mut mc = m;
        for r in 0..3 { mc[r][c] = b[r]; }
        let dc = mc[0][0] * (mc[1][1] * mc[2][2] - mc[1][2] * mc[2][1]) - mc[0][1] * (mc[1][0] * mc[2][2] - mc[1][2] * mc[2][0]) + mc[0][2] * (mc[1][0] * mc[2][1] - mc[1][1] * mc[2][0]);
        x[c] = dc / det;
    }
    Some(x)
}

/// How a driven mode came back: its fit and its class.
#[derive(Clone, Debug, PartialEq)]
pub enum Relaxation {
    /// `A e^{−λ t}`: the rate, the amplitude, the residual RMS over the fit, points used.
    Overdamped { lambda: f64, amplitude: f64, residual: f64, points: usize },
    /// `A e^{−Γ t} cos(ω t + φ)`: rate, angular frequency, amplitude, residual, points.
    Underdamped { gamma: f64, omega: f64, amplitude: f64, residual: f64, points: usize },
    /// Nothing to fit: the initial amplitude is under the stated noise (PR-3's refusal).
    Refused { amplitude: f64, noise: f64 },
}

/// Fit one cycle's mode series `y(t)` from its kick. Classification is by the data: if the
/// series changes sign within the first two e-folds of its envelope it is underdamped and
/// fitted as a damped cosine; otherwise it is fitted as a pure exponential on the log of
/// the absolute value over the first two e-folds. `noise` is the series' RMS before the
/// kick (or on the relaxed tail), and a kick under `3 noise` is REFUSED rather than fitted.
pub fn fit_relaxation(y: &[f64], dt: f64, noise: f64) -> Relaxation {
    let a0 = y.first().copied().unwrap_or(0.0);
    if a0.abs() < 3.0 * noise || y.len() < 4 {
        return Relaxation::Refused { amplitude: a0.abs(), noise };
    }
    // The window: up to the LAST index at which |y| is still above the noise (so a cosine's
    // dips through zero do not end it — the first draft cut at the first dip and read every
    // underdamped mode as overdamped), capped at the first two e-folds of the RUNNING
    // MAXIMUM of |y| read backwards.
    // The window ends where the signal stays dead: the first index after which |y| is under
    // 2 noise for five consecutive points (a stray noise excursion does not reopen it).
    let dead = 2.0 * noise;
    let mut cut = y.len();
    for i in 3..y.len().saturating_sub(4) {
        if y[i..i + 5].iter().all(|v| v.abs() < dead) { cut = i; break; }
    }
    let cut = cut.max(4).min(y.len());
    // Underdamped means a SIGNIFICANT rebound of the opposite sign — beyond 4 noise and a
    // tenth of the kick — not a zero crossing in the tail, which noise makes freely.
    let rebound = y[..cut].iter().any(|v| v.signum() != a0.signum() && v.abs() > (4.0 * noise).max(0.1 * a0.abs()));
    if !rebound {
        // Gauss–Newton on y = A e^{-λ t} directly (the log-linear fit is biased shallow
        // where the tail meets the noise); seeded from the log fit on points above 3 noise.
        let seed_pts: Vec<(f64, f64)> = y[..cut].iter().enumerate().filter(|(_, v)| v.abs() > 3.0 * noise).map(|(i, v)| (i as f64 * dt, v.abs().ln())).collect();
        if seed_pts.len() < 3 { return Relaxation::Refused { amplitude: a0.abs(), noise }; }
        let n = seed_pts.len() as f64;
        let (sx, sy) = (seed_pts.iter().map(|p| p.0).sum::<f64>(), seed_pts.iter().map(|p| p.1).sum::<f64>());
        let (sxx, sxy) = (seed_pts.iter().map(|p| p.0 * p.0).sum::<f64>(), seed_pts.iter().map(|p| p.0 * p.1).sum::<f64>());
        let mut lam = -(n * sxy - sx * sy) / (n * sxx - sx * sx);
        let mut amp = ((sy - (-lam) * sx) / n).exp() * a0.signum();
        let ts: Vec<f64> = (0..cut).map(|i| i as f64 * dt).collect();
        for _ in 0..30 {
            // residual r_i = y_i − A e^{−λ t_i}; J = [e^{−λt}, −A t e^{−λt}]
            let (mut jtj, mut jtr) = ([[0.0f64; 2]; 2], [0.0f64; 2]);
            for (i, &t) in ts.iter().enumerate() {
                let e = (-lam * t).exp(); let r = y[i] - amp * e;
                let j = [e, -amp * t * e];
                for p in 0..2 { jtr[p] += j[p] * r; for q in 0..2 { jtj[p][q] += j[p] * j[q]; } }
            }
            let det = jtj[0][0] * jtj[1][1] - jtj[0][1] * jtj[1][0];
            if det.abs() < 1e-300 { break; }
            let da = (jtr[0] * jtj[1][1] - jtr[1] * jtj[0][1]) / det;
            let dl = (jtj[0][0] * jtr[1] - jtj[1][0] * jtr[0]) / det;
            amp += da; lam += dl;
            if da.abs() < 1e-12 * amp.abs().max(1e-300) && dl.abs() < 1e-12 * lam.abs().max(1e-300) { break; }
        }
        let resid = (ts.iter().enumerate().map(|(i, &t)| (y[i] - amp * (-lam * t).exp()).powi(2)).sum::<f64>() / cut as f64).sqrt();
        Relaxation::Overdamped { lambda: lam, amplitude: amp, residual: resid, points: cut }
    } else {
        let zeros: Vec<f64> = y[..cut].windows(2).enumerate().filter(|(_, w)| w[0] * w[1] < 0.0)
            .map(|(i, w)| (i as f64 + w[0] / (w[0] - w[1])) * dt).collect();
        let omega = if zeros.len() >= 2 { std::f64::consts::PI / ((zeros[zeros.len() - 1] - zeros[0]) / (zeros.len() - 1) as f64) } else { std::f64::consts::PI / (2.0 * zeros[0]) };
        // ONE envelope point per half-period: the maximum of |y| between consecutive zero
        // crossings. (Every local maximum was the first draft, and at late times noise makes
        // local maxima all over the cosine's flanks, below the envelope — Γ read 15 % high.)
        let mut bounds: Vec<usize> = vec![0];
        bounds.extend(y[..cut].windows(2).enumerate().filter(|(_, w)| w[0] * w[1] < 0.0).map(|(i, _)| i + 1));
        bounds.push(cut);
        let mut ext: Vec<(f64, f64)> = Vec::new();
        for seg in bounds.windows(2) {
            if seg[1] <= seg[0] { continue; }
            let (i_max, v_max) = (seg[0]..seg[1]).map(|i| (i, y[i].abs())).fold((seg[0], 0.0), |b, x| if x.1 > b.1 { x } else { b });
            if v_max > 3.0 * noise { ext.push((i_max as f64 * dt, v_max.ln())); }
        }
        let n = ext.len() as f64;
        let (gamma, resid) = if ext.len() >= 2 {
            let (sx, sy) = (ext.iter().map(|p| p.0).sum::<f64>(), ext.iter().map(|p| p.1).sum::<f64>());
            let (sxx, sxy) = (ext.iter().map(|p| p.0 * p.0).sum::<f64>(), ext.iter().map(|p| p.0 * p.1).sum::<f64>());
            let g = (n * sxy - sx * sy) / (n * sxx - sx * sx); let b = (sy - g * sx) / n;
            (-g, (ext.iter().map(|p| (p.1 - (b + g * p.0)).powi(2)).sum::<f64>() / n).sqrt())
        } else { (f64::NAN, f64::NAN) };
        Relaxation::Underdamped { gamma, omega, amplitude: a0, residual: resid, points: ext.len() }
    }
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
    if let Density::Calibrated3 { .. } = density {
        // an averaged-field bin has no meaning on an instantaneous chart
        return Err(Refusal::TooFewFrames { have: 0, need: 1 });
    }
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
            Density::Poisson | Density::CellScale | Density::Derived | Density::External | Density::Calibrated(_) | Density::Calibrated3 { .. } => occ
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

    // ------------------------------------------- RUNG2_AMENDMENT_5.md's plants, PE-1..PE-5

    /// PE-1 — at window 1 the averaged chart IS the instantaneous one: bit for bit on every
    /// two-component grid, and on a 3D grid identical once the third momentum component
    /// A5.4 adds is stripped.
    #[test]
    fn pe1_window_one_is_the_instantaneous_chart() {
        let t2 = {
            let n = 12; let mut s: u64 = 0xE1;
            let mut next = move || { s = s.wrapping_mul(6364136223846793005).wrapping_add(1); ((s >> 11) as f64) / ((1u64 << 53) as f64) };
            let frames: Vec<Frame> = (0..200).map(|i| frame(i as u64,
                (0..n).map(|_| [next() * 34.6, next() * 20.8, 0.0]).collect(),
                (0..n).map(|_| [(next() - 0.5) * 4.0, (next() - 0.5) * 4.0, (next() - 0.5) * 4.0]).collect())).collect();
            Trajectory { header: header(n, vec![1; n]), frames }
        };
        for grid in FROZEN_GRIDS {
            for rung in LADDER {
                for d in [Density::Exact, Density::Poisson, Density::Derived] {
                    let a = readings3(&t2, grid.into(), rung, Kind::Spatial, d).unwrap();
                    let b = readings_windowed(&t2, grid.into(), rung, Kind::Spatial, d, 1).unwrap();
                    assert_eq!(a, b, "2D grid {grid:?} {rung:?} {d:?}");
                }
            }
        }
        let t3 = zero_sum_thermal_walk3(128, 29.6, 200, 0x5045_3031);
        let g = Grid3 { nx: 2, ny: 2, nz: 2 };
        let a = readings3(&t3, g, Rung::Mom, Kind::Spatial, Density::Derived).unwrap();
        let b = readings_windowed(&t3, g, Rung::Mom, Kind::Spatial, Density::Derived, 1).unwrap();
        let nc = g.cells();
        for (ra, rb) in a.iter().zip(&b) {
            assert_eq!(rb.len(), ra.len() + nc, "A5.4 adds exactly one component per cell");
            assert_eq!(&ra[..nc], &rb[..nc], "occupancy identical");
            for c in 0..nc { assert_eq!(&ra[nc + 2 * c..nc + 2 * c + 2], &rb[nc + 3 * c..nc + 3 * c + 2], "x,y identical in cell {c}"); }
        }
    }

    /// PE-2 — P-2's closed-by-construction chart, averaged over a window dividing its
    /// period (4), still certifies strict: averaging manufactures no defect.
    #[test]
    fn pe2_averaging_a_closed_chart_manufactures_no_defect() {
        let n = 4; let ncell = 4; let cw = 34.6 / ncell as f64;
        let frames: Vec<Frame> = (0..2400).map(|i| {
            let pos: Vec<[f64; 3]> = (0..n).map(|a| { let c = (a + i as usize) % ncell; [(c as f64 + 0.5) * cw, 10.0, 0.0] }).collect();
            frame(i as u64, pos, vec![[0.0; 3]; n])
        }).collect();
        let traj = Trajectory { header: header(n, vec![1; n]), frames };
        let grid = Grid3 { nx: 4, ny: 1, nz: 1 };
        let t = transport_fraction(&cell_series3(&traj, grid, Kind::Spatial).unwrap());
        for w in [1usize, 2] {
            let r = readings_windowed(&traj, grid, Rung::Occ, Kind::Spatial, Density::Calibrated3 { n: 0.5, p: 1.0, e: 1.0 }, w).unwrap();
            let a = leg_a(&r);
            assert!(a.informative >= prereg::MIN_INFORMATIVE, "w={w}: informative {}", a.informative);
            assert_eq!(grade(true, t, &a), Verdict::CertifiedStrict, "w={w}");
        }
    }

    /// PE-3 — what a window does and does not do to a hidden variable. A variable flipping
    /// FASTER than the window is BLURRED (its averaged chart has more collisions — the
    /// spread falls) and STILL FIRES (the collision form measures unpredictability, which
    /// averaging does not remove — the variable is still hidden). One flipping slower still
    /// fires too. The first draft asserted the fast variable's DEFECT falls under the window;
    /// it does not (0.91 against 0.86 instantaneous), and the amendment's PE-3 row was
    /// corrected to say what this test says: variance falls, firing does not.
    #[test]
    fn pe3_the_window_blurs_a_fast_hidden_variable_and_hides_none() {
        let n = 4; let ncell = 4; let cw = 34.6 / ncell as f64;
        let make = |flip_every: u64| -> Trajectory {
            let mut cells: Vec<usize> = (0..n).collect();
            let mut s: u64 = 0xDEAD_BEEF; let mut frames = Vec::new();
            for i in 0..4000u64 {
                let pos: Vec<[f64; 3]> = cells.iter().map(|&c| [(c as f64 + 0.5) * cw, 10.0, 0.0]).collect();
                frames.push(frame(i, pos, vec![[0.0; 3]; n]));
                if (i + 1) % flip_every == 0 {
                    for c in cells.iter_mut() {
                        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
                        *c = if (s >> 60) & 1 == 1 { (*c + 1) % ncell } else { (*c + ncell - 1) % ncell };
                    }
                }
            }
            Trajectory { header: header(n, vec![1; n]), frames }
        };
        let grid = Grid3 { nx: 4, ny: 1, nz: 1 };
        let w = 8usize;
        let d = Density::Calibrated3 { n: 0.5, p: 1.0, e: 1.0 };
        let fast_t = make(1);
        let f_inst = fields3(&fast_t, grid, Kind::Spatial).unwrap();
        let (s_inst, _, _) = field_sigmas(&f_inst, 2);
        let (s_avg, _, _) = field_sigmas(&window_mean(&f_inst, w), 2);
        assert!(s_avg < 0.5 * s_inst, "the window must blur a fast variable: σ {s_avg:.3} vs {s_inst:.3}");
        let fast = leg_a(&readings_windowed(&fast_t, grid, Rung::Occ, Kind::Spatial, d, w).unwrap());
        assert!(fast.firing > 0, "and the fast hidden variable must STILL fire — averaging removes variance, not unpredictability");
        let slow = leg_a(&readings_windowed(&make(64), grid, Rung::Occ, Kind::Spatial, d, w).unwrap());
        assert!(slow.firing > 0, "a variable flipping every 64 frames must still fire through an 8-frame window");
    }

    /// PE-4 — the averaging is doing what A5.2 says: on a zero-sum thermal carrier the
    /// averaged momentum's spread falls as `1/√w`.
    #[test]
    fn pe4_averaged_momentum_spread_falls_as_root_window() {
        let traj = zero_sum_thermal_walk3(128, 29.6, 2000, 0x5045_3034);
        let grid = Grid3 { nx: 2, ny: 1, nz: 1 };
        let f = fields3(&traj, grid, Kind::Spatial).unwrap();
        let (_, s1, _) = field_sigmas(&f, 2);
        let (_, s16, _) = field_sigmas(&window_mean(&f, 16), 2);
        let ratio = s1 / s16;
        assert!((ratio / 4.0 - 1.0).abs() < 0.2, "σ(w=1)/σ(w=16) = {ratio:.2}, expected ≈ 4");
    }

    /// PE-5 — `refines` across cadences is NOT a self-check: an average is not a coarsening
    /// of a frame. Recorded as a test that the relation FAILS, so nobody adds the assertion.
    #[test]
    fn pe5_refines_across_cadences_is_not_asserted() {
        let traj = zero_sum_thermal_walk3(128, 29.6, 400, 0x5045_3035);
        let grid = Grid3 { nx: 2, ny: 1, nz: 1 };
        let inst = readings3(&traj, grid, Rung::Occ, Kind::Spatial, Density::Exact).unwrap();
        let avg = readings_windowed(&traj, grid, Rung::Occ, Kind::Spatial, Density::Calibrated3 { n: 1.0, p: 1.0, e: 1.0 }, 4).unwrap();
        assert_ne!(inst.len(), avg.len(), "different cadences have different lengths; refines() is not even defined across them");
    }

    // ------------------------------------------- RUNG2_AMENDMENT_6.md's plants, PF-1..PF-4

    /// Particles carried by a small-amplitude STANDING WAVE, `v_x = (ξ ω) sin(kx) cos(ωt)` with
    /// `k = 2π/L`, displacement amplitude `ξ` a fraction of a cell, stepped exactly with the
    /// velocity recorded. Continuity holds by construction, the flux has a signal (the
    /// density oscillates at the nodes), and — the point — the density stays SMOOTH for ever,
    /// so the finite-volume closure's error is a discretisation error that converges.
    ///
    /// Two carriers came before this one and both taught something: a divergence-free flow
    /// at uniform density (continuity predicts zero; the reading of 1.0 was correct), and a
    /// compressive flow run long enough to form caustics (`k A t ≈ 16`), whose sub-cell density
    /// structure no closure resolves at any grid — `D_cont` sat at 0.3 for 4, 8 and 16 cells.
    fn standing_wave3(n: usize, edge: f64, frames: usize, xi: f64, period_frames: usize, dt: f64, seed: u64) -> Trajectory {
        let mut u = seed;
        let mut next = move || { u = u.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((u >> 11) as f64) / ((1u64 << 53) as f64) };
        let mut pos: Vec<[f64; 3]> = (0..n).map(|_| [next() * edge, next() * edge, next() * edge]).collect();
        let k = 2.0 * std::f64::consts::PI / edge;
        let omega = 2.0 * std::f64::consts::PI / (period_frames as f64 * dt);
        let mut out = Vec::with_capacity(frames);
        for i in 0..frames {
            let t = i as f64 * dt;
            let vel: Vec<[f64; 3]> = pos.iter().map(|p| [xi * omega * (k * p[0]).sin() * (omega * t).cos(), 0.0, 0.0]).collect();
            out.push(frame(i as u64, pos.clone(), vel.clone()));
            for (p, v) in pos.iter_mut().zip(&vel) { p[0] = (p[0] + v[0] * dt).rem_euclid(edge); }
        }
        Trajectory { header: header3(n, vec![8; n], edge), frames: out }
    }

    /// PF-1 — a standing wave: the momentum field accounts for the density change, and the
    /// closure CONVERGES with the grid where it can. "Where it can" is the third thing this
    /// plant taught: on ANY particle carrier the crossings through a face in a window are a
    /// count, so `D_cont` has a shot-noise floor of about `1/√(crossings)`, and refining the
    /// grid RAISES it (at 16,000 particles the reading went 0.42 → 0.46 → 0.69 with the grid).
    /// A million particles put the floor at a few percent on the coarse grids, and there the
    /// discretisation error shows and falls. The same floor bounds the leg's power on a
    /// liquid, and the amendment says so.
    #[test]
    fn pf1_a_standing_wave_converges_under_continuity() {
        let (n, edge, dt) = (1_000_000, 40.0, 1.0);
        let m = O_MASS_U * M_E_PER_U;
        let traj = standing_wave3(n, edge, 160, 1.0, 80, dt, 0x5046_3031);
        let w = 8usize;
        let mut ds = Vec::new();
        for nx in [4usize, 8] {
            let grid = Grid3 { nx, ny: 3, nz: 3 };
            let f = fields3(&traj, grid, Kind::Spatial).unwrap();
            ds.push(continuity(&window_mean(&f, w), grid, [edge; 3], m, w as f64 * dt).defect().unwrap());
        }
        eprintln!("PF-1 D_cont at 4/8 cells per wavelength, 1e6 particles: {:.3} / {:.3}", ds[0], ds[1]);
        assert!(ds[1] < ds[0], "the closure must improve from 4 to 8 cells per wavelength above the shot-noise floor: {ds:?}");
        assert!(ds[1] < 0.15, "and be under 0.15 at eight cells per wavelength: {:.3}", ds[1]);
    }

    /// PF-2 — random walkers whose velocities are uncorrelated with their displacement: the
    /// momentum field explains nothing, and the blind chart does no worse.
    #[test]
    fn pf2_random_walkers_fail_continuity_and_do_not_separate() {
        let (n, edge) = (2000, 40.0);
        let m = O_MASS_U * M_E_PER_U;
        let traj = zero_sum_thermal_walk3(n, edge, 400, 0x5046_3032);
        let grid = Grid3 { nx: 4, ny: 4, nz: 4 };
        let w = 4usize;
        let tau = w as f64 * 826.0;
        let fs = fields3(&traj, grid, Kind::Spatial).unwrap();
        let fb = fields3(&traj, grid, Kind::BlindLabel).unwrap();
        let ds = continuity(&window_mean(&fs, w), grid, [edge; 3], m, tau).defect().unwrap();
        let db = continuity(&window_mean(&fb, w), grid, [edge; 3], m, tau).defect().unwrap();
        assert!(ds > 0.7, "walkers: D_cont = {ds:.3}, expected near 1");
        assert!((db - ds).abs() < 0.3, "and no separation from the blind chart: {ds:.3} vs {db:.3}");
    }

    /// PF-3 — the control loses on the carrier the real chart passes.
    #[test]
    fn pf3_the_scrambled_control_fails_where_advection_passes() {
        let (n, edge, _dt) = (200_000, 40.0, 1.0);
        let m = O_MASS_U * M_E_PER_U;
        let traj = standing_wave3(n, edge, 160, 1.0, 80, 1.0, 0x5046_3033);
        let grid = Grid3 { nx: 8, ny: 3, nz: 3 };
        let w = 8usize;
        let dt = 1.0;
        let ds = continuity(&window_mean(&fields3(&traj, grid, Kind::Spatial).unwrap(), w), grid, [edge; 3], m, w as f64 * dt).defect().unwrap();
        let db = continuity(&window_mean(&fields3(&traj, grid, Kind::BlindLabel).unwrap(), w), grid, [edge; 3], m, w as f64 * dt).defect().unwrap();
        assert!(db - ds >= prereg::MIN_SEPARATION, "G7-form separation on advection: spatial {ds:.3} blind {db:.3}");
        assert!(db > 0.7, "the scrambled control must fail: {db:.3}");
    }

    /// PF-4 — a `dims = 2` chart sums four faces: with `n_z = 1` the z-faces contribute
    /// nothing, exactly, whatever the z-momentum.
    #[test]
    fn pf4_a_2d_chart_sums_four_faces() {
        let (n, edge, dt) = (1000, 40.0, 5.0);
        let m = O_MASS_U * M_E_PER_U;
        let traj = standing_wave3(n, edge, 100, 1.0, 80, dt, 0x5046_3034);
        let grid = Grid3 { nx: 4, ny: 4, nz: 1 };
        let f = window_mean(&fields3(&traj, grid, Kind::Spatial).unwrap(), 4);
        let a = continuity(&f, grid, [edge; 3], m, 20.0);
        let mut g = f.clone();
        for w in g.iter_mut() { for c in w.p.iter_mut() { c[2] += 1e6; } }
        let b = continuity(&g, grid, [edge; 3], m, 20.0);
        assert_eq!(a.rms_residual.to_bits(), b.rms_residual.to_bits(), "z-momentum must not enter a 2D chart's continuity");
    }

    // ------------------------------------------- RESPONSE1_PREREG.md's plants, PR-1..PR-5

    /// A synthetic mode series on random positions: `N` atoms placed so that `ρ_k(t)`
    /// follows a given law exactly (a fraction of the atoms are displaced along the wave),
    /// with thermal velocities of stated scale for the current mode's noise.
    fn synth_series(law: impl Fn(f64) -> f64, frames: usize, dt: f64, noise: f64, seed: u64) -> Vec<f64> {
        let mut s = seed;
        let mut next = move || { s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((s >> 11) as f64) / ((1u64 << 53) as f64) - 0.5 };
        (0..frames).map(|i| law(i as f64 * dt) + noise * 3.46 * next()).collect()   // uniform(-0.5,0.5)*3.46 has sd 1
    }

    /// PR-1 — an overdamped mode is read back to 2 %.
    #[test]
    fn pr1_an_overdamped_mode_reads_its_rate() {
        let (a, lam, dt) = (20.0, 1.0 / 1716.0, 10.0);
        let y = synth_series(|t| a * (-lam * t).exp(), 600, dt, 0.2, 0x5052_3031);
        match fit_relaxation(&y, dt, 0.2) {
            Relaxation::Overdamped { lambda, amplitude, .. } => {
                assert!((lambda / lam - 1.0).abs() < 0.02, "λ {lambda:.3e} vs {lam:.3e}");
                assert!((amplitude / a - 1.0).abs() < 0.05);
            }
            other => panic!("expected overdamped, got {other:?}"),
        }
    }

    /// PR-2 — an underdamped mode: period to 2 %, Γ to 5 %.
    #[test]
    fn pr2_an_underdamped_mode_reads_period_and_envelope() {
        let (a, g, w, dt) = (20.0, 1.0 / 2000.0, 2.0 * std::f64::consts::PI / 500.0, 5.0);
        let y = synth_series(|t| a * (-g * t).exp() * (w * t).cos(), 1200, dt, 0.05, 0x5052_3032);
        match fit_relaxation(&y, dt, 0.05) {
            Relaxation::Underdamped { gamma, omega, .. } => {
                assert!((omega / w - 1.0).abs() < 0.02, "ω {omega:.4e} vs {w:.4e}");
                assert!((gamma / g - 1.0).abs() < 0.05, "Γ {gamma:.3e} vs {g:.3e}");
            }
            other => panic!("expected underdamped, got {other:?}"),
        }
    }

    /// PR-3 — a transverse current mode on thermal noise: read to 5 % when the kick is
    /// three times the noise; REFUSED with the reason when it is under the noise.
    #[test]
    fn pr3_a_current_mode_reads_or_refuses_by_its_noise() {
        let (g, dt, noise) = (1.0 / 140.0, 10.0, 1.0);
        let strong = synth_series(|t| 6.0 * (-g * t).exp(), 200, dt, noise, 0x5052_3033);
        match fit_relaxation(&strong, dt, noise) {
            Relaxation::Overdamped { lambda, .. } => assert!((lambda / g - 1.0).abs() < 0.05, "Γ_s {lambda:.3e} vs {g:.3e}"),
            other => panic!("expected a read, got {other:?}"),
        }
        let weak = synth_series(|t| 0.5 * (-g * t).exp(), 200, dt, noise, 0x5052_3034);
        assert!(matches!(fit_relaxation(&weak, dt, noise), Relaxation::Refused { .. }), "a kick under the noise must refuse");
    }

    /// PR-5 — the scrambled partition of a coherent wave carries no mode: the blind ρ_k is
    /// under a tenth of the spatial one.
    #[test]
    fn pr5_the_scrambled_partition_has_no_mode() {
        let (n, edge) = (2000, 40.0);
        let k = 2.0 * std::f64::consts::PI / edge;
        // atoms displaced along a standing wave of the density: x = u + A sin(k u)
        let mut s: u64 = 0x5052_3035;
        let mut next = move || { s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((s >> 11) as f64) / ((1u64 << 53) as f64) };
        let pos: Vec<[f64; 3]> = (0..n).map(|_| { let u = next() * edge; [(u - 3.0 * (k * u).sin()).rem_euclid(edge), next() * edge, next() * edge] }).collect();
        let traj = Trajectory { header: header3(n, vec![8; n], edge), frames: vec![frame(0, pos, vec![[0.0; 3]; n])] };
        let (rs, _) = modes(&traj, 0, Kind::Spatial).unwrap();
        let (rb, _) = modes(&traj, 0, Kind::BlindLabel).unwrap();
        assert!(rs[0].abs() > 100.0, "the spatial mode must see the wave: {}", rs[0]);
        assert!(rb[0].abs() < 0.1 * rs[0].abs(), "the scrambled partition must not: blind {} vs spatial {}", rb[0], rs[0]);
    }

    /// PR-6 (Amendment 1) — thermal velocities on random positions, kicked by §2's rule:
    /// the sin reader returns the kick amplitude to 2 %; the cos reader, the undriven
    /// quadrature, returns under 3σ. (The reader as first built read cos against a sin
    /// kick and fitted noise for one run.)
    #[test]
    fn pr6_the_driven_quadrature_carries_the_kick_and_the_other_does_not() {
        let (n, edge) = (432usize, 44.39);
        let k = 2.0 * std::f64::consts::PI / edge;
        let mut s: u64 = 0x5052_3036;
        let mut next = move || { s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((s >> 11) as f64) / ((1u64 << 53) as f64) };
        let pos: Vec<[f64; 3]> = (0..n).map(|_| [next() * edge, next() * edge, next() * edge]).collect();
        // thermal velocities of scale σ_v; the kick sized so v_d N/2 = 4 σ with σ = σ_v √(N/2)
        let sigma_v = 1.7e-4;
        let sigma = sigma_v * (n as f64 / 2.0).sqrt();
        let v_d = 4.0 * sigma / (n as f64 / 2.0);
        let mean_sin = pos.iter().map(|p| (k * p[0]).sin()).sum::<f64>() / n as f64;
        let mut vel: Vec<[f64; 3]> = (0..n).map(|_| [0.0, sigma_v * 3.46 * (next() - 0.5), 0.0]).collect();
        let thermal: Vec<f64> = vel.iter().map(|v| v[1]).collect();
        for (i, p) in pos.iter().enumerate() { vel[i][1] += v_d * ((k * p[0]).sin() - mean_sin); }
        let traj = Trajectory { header: header3(n, vec![8; n], edge), frames: vec![frame(0, pos.clone(), vel)] };
        let m = modes_both(&traj, 1, Kind::Spatial).unwrap();
        let expected: f64 = pos.iter().map(|p| { let sn = (k * p[0]).sin(); v_d * (sn - mean_sin) * sn }).sum();
        let thermal_sin: f64 = pos.iter().zip(&thermal).map(|(p, v)| v * (k * p[0]).sin()).sum();
        assert!(((m.cur_sin[0] - thermal_sin) / expected - 1.0).abs() < 1e-9, "sin reader {} vs kick {expected} (+ thermal {thermal_sin})", m.cur_sin[0]);
        assert!((expected / (v_d * n as f64 / 2.0) - 1.0).abs() < 0.02, "the kick's own projection is v_d N/2 to 2 %: {expected} vs {}", v_d * n as f64 / 2.0);
        assert!(m.cur_cos[0].abs() < 3.0 * sigma, "the cos quadrature must carry no kick: {} vs 3σ {}", m.cur_cos[0], 3.0 * sigma);
        // the kick's own projection is 4σ by construction; the thermal draw on this seed is
        // −0.5σ, so the noisy total is what a single cycle at SNR 4 looks like (A2)
        assert!((m.cur_sin[0] - thermal_sin).abs() > 3.0 * sigma, "the sin quadrature must carry it: {} vs 3σ {}", m.cur_sin[0] - thermal_sin, 3.0 * sigma);
    }

    /// PR-7 (Amendment 1) — twelve cycles of alternating sign at SNR 2 each: the aligned
    /// average reads λ to 5 % while at least nine of the twelve per-cycle fits refuse.
    #[test]
    fn pr7_the_aligned_average_reads_what_single_cycles_refuse() {
        let (lam, dt, relax, cycles, noise) = (1.0 / 140.0, 10.0, 314usize, 12usize, 1.0);
        let a = 2.0 * noise;   // SNR 2 per cycle
        let mut series = vec![0.0];   // the pre-kick row
        let mut s: u64 = 0x5052_3037;
        let mut next = move || { s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((s >> 11) as f64) / ((1u64 << 53) as f64) - 0.5 };
        for c in 0..cycles {
            let sign = if c % 2 == 0 { 1.0 } else { -1.0 };
            for i in 0..relax { series.push(sign * a * (-lam * i as f64 * dt).exp() + noise * 3.46 * next()); }
        }
        let al = align_cycles(&series, cycles, relax, 1);
        assert_eq!(al.per_cycle.len(), cycles);
        let refused = al.per_cycle.iter().filter(|c| matches!(fit_relaxation(c, dt, al.noise_per_cycle), Relaxation::Refused { .. })).count();
        assert!(refused >= 9, "at SNR 2 most cycles must refuse: {refused} of {cycles}");
        match fit_relaxation(&al.mean, dt, al.noise) {
            Relaxation::Overdamped { lambda, .. } => assert!((lambda / lam - 1.0).abs() < 0.05, "aligned λ {lambda:.3e} vs {lam:.3e} (noise {:.3})", al.noise),
            other => panic!("the aligned average must read: {other:?} (noise {:.3})", al.noise),
        }
    }

    /// PR-8 (Amendment 1) — a density mode that rises then decays, `e^{−λ₁t} − e^{−λ₂t}`
    /// with `λ₂ = 40 λ₁` and noise a tenth of the peak: λ₁ to 5 % from the peak, λ₂ from
    /// the two-exponential form to 10 %, and the series is NOT classified underdamped.
    #[test]
    fn pr8_a_rising_density_mode_reads_its_slow_rate_from_the_peak() {
        let (l1, dt): (f64, f64) = (1.0 / 1700.0, 10.0);
        let l2: f64 = 40.0 * l1;
        let a: f64 = 10.0;
        let peak = a * ((-l1 * 40.0).exp() - (-l2 * 40.0).exp());
        let y = synth_series(|t| a * ((-l1 * t).exp() - (-l2 * t).exp()), 314, dt, 0.1 * peak, 0x5052_3038);
        let r = fit_rise_decay(&y, dt, 0.1 * peak);
        match r.slow {
            Relaxation::Overdamped { lambda, .. } => assert!((lambda / l1 - 1.0).abs() < 0.05, "λ₁ {lambda:.3e} vs {l1:.3e} from peak {}", r.peak),
            ref other => panic!("expected overdamped from the peak, got {other:?} (peak {})", r.peak),
        }
        let fast = r.fast.expect("the rise is resolved (peak >= 3 readouts) so λ₂ must be read");
        assert!((fast / l2 - 1.0).abs() < 0.10, "λ₂ {fast:.3e} vs {l2:.3e}");
    }

    /// The exact continuum carrier for PR-9/10/12 (RESPONSE1_AMENDMENT_2): the linearised
    /// hydrodynamic response to a velocity kick, overdamped — `u(x,t) = u0 sin(kx) ·
    /// (λ₂ e^{−λ₂ t} − λ₁ e^{−λ₁ t})/(λ₂ − λ₁)` whose net displacement is zero, so the density
    /// continuity gives it, `δn = −n̄ k u0 cos(kx) (e^{−λ₁ t} − e^{−λ₂ t})/(λ₂ − λ₁)`, rises and
    /// returns (Amendment 1 A3's form). Cycles alternate sign and SUPERPOSE (the response is
    /// linear; nothing is reset at a cycle boundary), cell-averaged on `nx` cells per
    /// readout; `λ` in inverse readouts. `sigma` is an occupancy shot noise per WINDOW-cell
    /// (constant within a window of `w` readouts, so window averaging does not thin it) and
    /// `sigma_p` the same on the momentum side. `m̄ = 1`, `edge = L/nx`.
    #[allow(clippy::too_many_arguments)]
    fn continuum_carrier(nx: usize, l: f64, n_bar: f64, u0: f64, lam1: f64, lam2: f64, cycles: usize, relax: usize, w: usize, sigma: f64, sigma_p: f64, seed: u64) -> Vec<CellFields> {
        let mut s = seed;
        let mut next = move || { s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); ((s >> 11) as f64) / ((1u64 << 53) as f64) - 0.5 };
        let k = 2.0 * std::f64::consts::PI / l;
        let edge = l / nx as f64;
        let mut out = Vec::new();
        let mut noise_occ = vec![0.0; nx]; let mut noise_p = vec![0.0; nx];
        for c in 0..cycles {
            for i in 0..relax {
                if i % w == 0 { for cidx in 0..nx { noise_occ[cidx] = sigma * 3.46 * next(); noise_p[cidx] = sigma_p * 3.46 * next(); } }
                // superpose every cycle so far
                let (mut vel, mut dis) = (0.0, 0.0);   // the mode's velocity and displacement amplitudes
                for cp in 0..=c {
                    let sign = if cp % 2 == 0 { 1.0 } else { -1.0 };
                    let t = ((c - cp) * relax + i) as f64;
                    vel += sign * (lam2 * (-lam2 * t).exp() - lam1 * (-lam1 * t).exp()) / (lam2 - lam1);
                    dis += sign * ((-lam1 * t).exp() - (-lam2 * t).exp()) / (lam2 - lam1);
                }
                let mut occ = vec![0.0; nx]; let mut p = vec![[0.0; 3]; nx];
                for cidx in 0..nx {
                    let (a, b) = (cidx as f64 * edge, (cidx as f64 + 1.0) * edge);
                    // ∫_cell n dx = n̄ edge − n̄ u0 dis ∫ k cos(kx) dx ; ∫_cell n̄ u dx = n̄ u0 vel ∫ sin(kx) dx
                    occ[cidx] = n_bar * edge - n_bar * u0 * dis * ((k * b).sin() - (k * a).sin()) + noise_occ[cidx];
                    p[cidx][0] = n_bar * u0 * vel * (-((k * b).cos() - (k * a).cos()) / k) + noise_p[cidx];
                }
                out.push(CellFields { occ, p, ek: vec![0.0; nx] });
            }
        }
        out
    }

    /// PR-12 (Amendment 2) — the midpoint law's spatial floor: the exact continuum wave at
    /// zero noise reads `D_cont = 1 − sinc(π/n) cos(π/n)` to 1 % on 4, 8 and 16 cells.
    #[test]
    fn pr12_the_midpoint_laws_spatial_floor_is_read_back_on_the_exact_wave() {
        let (relax, w) = (320usize, 40usize);
        for nx in [4usize, 8, 16] {
            let graded = nx <= 8;   // the grids R1 is read on; 16 is printed (the formula's residual grows as the floor shrinks)
            // slow decay (λ₁ = 1/(200 windows)) isolates the spatial floor; the realistic cadence
            // (λ₁ = 1/(4.3 windows), λ₂ = 40 λ₁) is printed beside it
            let slow = continuum_carrier(nx, 4.0, 100.0, 1e-3, 1.0 / (200.0 * w as f64), 40.0 / (200.0 * w as f64), 1, relax, w, 0.0, 0.0, 1);
            let d = continuity(&window_mean(&slow, w), Grid3 { nx, ny: 1, nz: 1 }, [4.0, 1.0, 1.0], 1.0, w as f64).defect().unwrap();
            let real = continuum_carrier(nx, 4.0, 100.0, 1e-3, 1.0 / (4.3 * w as f64), 40.0 / (4.3 * w as f64), 1, relax, w, 0.0, 0.0, 1);
            let d_real = continuity(&window_mean(&real, w), Grid3 { nx, ny: 1, nz: 1 }, [4.0, 1.0, 1.0], 1.0, w as f64).defect().unwrap();
            let d_int = continuity_integral(&real, Grid3 { nx, ny: 1, nz: 1 }, [4.0, 1.0, 1.0], 1.0, 1.0, w).defect().unwrap();
            let floor = continuity_spatial_floor(nx);
            eprintln!("PR-12 n={nx}: window-mean law D {d:.4} (slow decay) vs derived spatial floor {floor:.4}; at the realistic cadence {d_real:.4} (window-mean) and {d_int:.4} (integral form)");
            if !graded { continue; }
            assert!((d - floor).abs() < 0.05 * floor, "n={nx}: D {d:.4} vs derived floor {floor:.4}");
            assert!(d_real > 1.0, "the window-mean law must FAIL on the fast transient at the cell cadence: {d_real:.3}");
            assert!((d_int - floor).abs() < 0.05 * floor, "integral form at the realistic cadence: D {d_int:.4} vs spatial floor {floor:.4}");
        }
    }

    /// PR-9 (Amendment 2) — `D_cont` on raw pooled windows does NOT fall with the window
    /// count and sits at `√((D_disc² s² + 1)/(s² + 1))`; on the cycle-aligned average `s`
    /// becomes `√C s`, and `D` falls toward the spatial floor.
    #[test]
    fn pr9_the_continuity_defect_falls_with_aligned_cycles_not_with_pooled_windows() {
        // sixteen windows a cycle so the tail the noise is read from is relaxed (e^{−14/4.3} = 4 %
        // of the peak); the arms' eight-window cycles read their noise from the blind partition
        let (nx, cycles, relax, w) = (8usize, 12usize, 640usize, 40usize);
        let grid = Grid3 { nx, ny: 1, nz: 1 };
        let boxe = [4.0, 1.0, 1.0];
        let (lam1, lam2) = (1.0 / (4.3 * w as f64), 40.0 / (4.3 * w as f64));
        // size the noise from the quiet carrier: the RMS window-to-window occupancy change over
        // the first two windows, divided by the target s = 1.2
        let quiet = continuum_carrier(nx, 4.0, 100.0, 1e-3, lam1, lam2, 1, relax, w, 0.0, 0.0, 1);
        let sig_rms = { let (mut s2, mut n) = (0.0, 0); for k in 0..2 { for c in 0..nx { let d = quiet[(k + 1) * w].occ[c] - quiet[k * w].occ[c]; s2 += d * d; n += 1; } } (s2 / n as f64).sqrt() };
        let sigma = sig_rms / 1.2 / 2f64.sqrt();   // a window-to-window difference of two independent noises has √2 the per-window SD
        let noisy = continuum_carrier(nx, 4.0, 100.0, 1e-3, lam1, lam2, cycles, relax, w, sigma, 0.0, 0x5052_3039);
        let d_disc = continuity_spatial_floor(nx);
        // raw pooled windows: one cycle and twelve cycles must read the SAME D within noise
        // the driven read is over the LEAD windows of a cycle (the prereg's "first two"): pooling
        // relaxed, noise-only windows into an RMS ratio can only raise it
        let lead = 2usize;
        let d_one = continuity_integral(&noisy[..=lead * w], grid, boxe, 1.0, 1.0, w).defect().unwrap();
        // twelve cycles' lead windows pooled (equal counts per cycle, so the ratio of summed squares)
        let (mut so, mut sr) = (0.0, 0.0);
        for c in 0..cycles { let k = continuity_integral(&noisy[c * relax..=c * relax + lead * w], grid, boxe, 1.0, 1.0, w); so += k.rms_observed * k.rms_observed; sr += k.rms_residual * k.rms_residual; }
        let d_raw = (sr / so).sqrt();
        let (s_one, _) = driven_floor(&noisy[..relax], w, lead);
        let floor_raw = ((d_disc * d_disc * s_one * s_one + 1.0) / (s_one * s_one + 1.0)).sqrt();
        let (aligned, used) = align_fields(&noisy, cycles, relax, 0);
        assert_eq!(used, cycles);
        let d_al = continuity_integral(&aligned[..=lead * w], grid, boxe, 1.0, 1.0, w).defect().unwrap();
        let (s_al, _) = driven_floor(&aligned, w, lead);
        let floor_al = ((d_disc * d_disc * s_al * s_al + 1.0) / (s_al * s_al + 1.0)).sqrt();
        eprintln!("PR-9 (n={nx}, D_disc {d_disc:.3}): one cycle D {d_one:.3} (s {s_one:.2}, floor {floor_raw:.3}); twelve cycles' lead windows pooled D {d_raw:.3}; aligned D {d_al:.3} (s {s_al:.2}, floor {floor_al:.3})");
        assert!(s_one > 0.6 && s_one < 2.5, "the carrier must sit near s ≈ 1.2 per window-cell: {s_one:.2}");
        assert!((d_raw - d_one).abs() < 0.15, "pooling windows must not lower D: one cycle {d_one:.3}, twelve {d_raw:.3}");
        assert!((d_raw - floor_raw).abs() < 0.1, "raw D {d_raw:.3} vs its floor {floor_raw:.3}");
        assert!((d_al - floor_al).abs() < 0.1, "aligned D {d_al:.3} vs its floor {floor_al:.3}");
        assert!(d_al < 0.6 * d_raw, "alignment must lower D: {d_al:.3} vs {d_raw:.3}");
        assert!(s_al > 2.0 * s_one, "alignment must raise s by about √12: {s_al:.2} vs {s_one:.2}");
    }

    /// PR-10 (Amendment 2) — no drive: the aligned `D_cont` stays at the floor (≥ 0.8); the
    /// alignment manufactures no closure.
    #[test]
    fn pr10_alignment_manufactures_no_closure_without_a_drive() {
        let (cycles, relax, window) = (12usize, 320usize, 40usize);
        let fields = continuum_carrier(4, 4.0, 100.0, 0.0, 1.0 / (4.3 * window as f64), 40.0 / (4.3 * window as f64), cycles, relax, window, 2.0, 2.0, 0x5052_303a);
        let grid = Grid3 { nx: 4, ny: 1, nz: 1 };
        let (aligned, _) = align_fields(&fields, cycles, relax, 0);
        let d = continuity_integral(&aligned[..=2 * window], grid, [4.0, 1.0, 1.0], 1.0, 1.0, window).defect().unwrap();
        assert!(d >= 0.8, "undriven aligned D must sit at its floor: {d:.3}");
    }

    /// PR-11 (Amendment 2) — the prereg's `2×2×1` is refused for the continuity leg by name.
    #[test]
    fn pr11_the_preregs_grid_is_refused_by_name() {
        assert!(continuity_admits(Grid3 { nx: 2, ny: 2, nz: 1 }).is_err());
        assert!(continuity_admits(Grid3 { nx: 4, ny: 1, nz: 1 }).is_ok());
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
