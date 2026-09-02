//! THE dE5 TRUNCATION AUDIT — does the many-body ladder terminate at four?
//!
//! GANTT node H: **MEASURE, never build.** This example does not contain a five-body
//! term and no branch of it writes one. It measures
//!
//! ```text
//! dE5(S) = E_FCI(S) - E_MBE4(S),   |S| = 5
//! ```
//!
//! on compact five-atom clusters drawn by a staked rule from the parked mixed-arm
//! trajectories, and reports the distribution against a bound frozen before the
//! instrument existed.
//!
//! Every constant below is a quotation from `conformance/water_observatory/DE5_PREREG.md`,
//! which is committed AHEAD of this file — `git log --oneline` is the proof of order.
//! Nothing here may be tuned; a value that needs to change needs the freeze amended
//! first, in its own commit, saying so.
//!
//! # The estimator, and its subtraction basis
//!
//! ```text
//! E_MBE4(S) = SUM_i E({a_i}) + SUM_{|P|=2} dE2(P) + SUM_{|T|=3} dE3(T) + SUM_{|Q|=4} dE4(Q)
//! ```
//!
//! with each rung defined as the previous ladder's closing residual. `subtraction_basis`
//! is **`fci_live`**: every rung at every arity from the determinant route, exact in
//! model, with no served surface anywhere in the assembly. At the PAIR rung this is
//! `pair_point_exact`, which is `quaternary::ohhh_mbe3_energy`'s own convention, matched
//! by calling the same three functions (`pair_point`, `atom_energy_o`, `atom_energy_h`)
//! and checked for bit equality at startup rather than assumed.
//!
//! At the triple and quadruple rungs the basis DIFFERS from the runtime path on purpose:
//! `WaterTable`'s and `TrimerTable`'s measured held-out errors (3.3e-4 and 6.3e-5 Ha) are
//! at and above the 5e-5 Ha bound this audit tests, so a served assembly would put its
//! own grid error where the five-body term should be. The freeze's section 1.1 carries
//! the full argument; gate 9 here prints the two bases side by side on every `OHHH`
//! quadruple so the difference is measured rather than asserted.
//!
//! # Why every solve goes through `solve_determinant`
//!
//! `fci::solve` routes past `MPS_ROUTE_THRESHOLD = 50,000` determinants into DMRG, whose
//! energy is a variational upper bound and not exact in model. The `O2H3` pentamer is
//! 204,490 determinants and the `O2H2` quadruple is 48,400 — a 3.2% margin, far too
//! close to assume — so this file reads `Solution::route` on EVERY solve and refuses a
//! `Dmrg` route rather than trusting a size it believes it knows.
//!
//! # Usage
//!
//! ```text
//! de5_audit --traj-dir DIR --manifest FILE [--out CSV] [--plants] [--audit]
//!           [--no-crosscheck] [--limit N]
//! ```
//!
//! Unknown arguments are REFUSED, never ignored: a CLI that silently drops an argument
//! makes one command mean two different things in two different trees, and the tell is
//! bit-identical output where the operator expected a change. Every parameter that shaped
//! a reading is echoed into the CSV header and onto stdout.

use holon_chem::dual::D2;
use holon_chem::elements::{Species, HYDROGEN, OXYGEN};
use holon_chem::fci::{self, SolveExit, SolverRoute, MPS_ROUTE_THRESHOLD};
use holon_chem::pair::{
    geometry_problem, pair_point, solve_geometry, CONVERGED_RESIDUAL,
};
use holon_chem::quaternary::{atom_energy_h, atom_energy_o, de4_ohhh_fci, R_CUT};
use holon_chem::trimer::{self, TrimerTable};
use holon_chem::water::{self, WaterTable};
use holon_lens::traj::Trajectory;
use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

// ---------------------------------------------------------------- the staked constants
//
// Every one of these is quoted from DE5_PREREG.md by section number. They are `const`
// rather than CLI knobs so that changing one is a source edit visible in a diff.

/// Freeze section 3.1. The cluster DIAMETER fence, in bohr, equal to `quaternary::R_CUT`
/// — the measured radius past which dE4 itself falls below the tolerance this audit uses
/// as its bound. Sampling inside it means every scored cluster is one where the previous
/// rung is still live.
const R_COMPACT: f64 = R_CUT;

/// Freeze section 3.2. Frames between samples.
const STRIDE: usize = 250;

/// Freeze section 3.3. A given (seed, five-set of arena indices) contributes at most one
/// candidate per block of this many frames — 1.67 ps, more than three H2O rotations.
const DEDUPE_BLOCK: u64 = 2000;

/// Freeze section 3.4. Per-composition quota, target draw, and the shortfall priority.
const QUOTA: usize = 8;
const N_TARGET: usize = 24;
/// Shortfall redistribution order, fixed in the freeze and nowhere else: `O2H3` first
/// because it is the composition the `fenced` seed 0x53415422 actually formed.
const SHORTFALL_PRIORITY: [(usize, usize); 3] = [(2, 3), (1, 4), (0, 5)];

/// Freeze section 2.1. The determinant fence. `O2H3` at 204,490 is the largest admitted;
/// `O3H2` at 5,664,400 is the smallest refused; nothing lies in the 27.7x gap between
/// them, so no threshold inside it can move a verdict.
const FCI_DET_MAX: usize = 250_000;

/// Freeze section 4. `|dE5|` below this reads TERMINATES for that config.
const BOUND: f64 = 5.0e-5;

/// Freeze section 3.5. A config is LIVE only if some quadruple term reaches the same bar.
const LIVE_BAR: f64 = 5.0e-5;

/// Freeze section 8, plant P-1: a plant smaller than this could not be seen.
const PLANT_MIN: f64 = 1.0e-6;
/// P-1's identity tolerance.
const P1_TOL: f64 = 1.0e-12;
/// P-2's separation, bohr, and the zero it must read.
const P2_SEP: f64 = 40.0;
const P2_TOL: f64 = 1.0e-8;

/// Freeze section 5, gate G10. Per-config budget in seconds.
const WALL_BUDGET_S: f64 = 3600.0;

/// Freeze section 2.1. The three admitted compositions, as `(n_oxygen, n_hydrogen)`.
const IN_SCOPE: [(usize, usize); 3] = [(0, 5), (1, 4), (2, 3)];

/// Freeze section 1.4.
const DEVICE_CLASS: &str = "cpu";
const SUBTRACTION_BASIS: &str = "fci_live";

// -------------------------------------------------------------------------- sha256
//
// Written here rather than pulled in, because `holon-chem`'s dependency rule forbids a
// hashing crate and the trajectory pin is worth more than the twenty lines. It is
// SELF-TESTED against the two NIST vectors at startup: an unchecked hash that is subtly
// wrong would refuse every trajectory or accept every trajectory, and both failures look
// like a configuration problem rather than a broken instrument.

const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
    0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
    0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
    0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
    0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
    0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
    0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
    0xc67178f2,
];

fn sha256(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bitlen = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    let mut w = [0u32; 64];
    for chunk in msg.chunks_exact(64) {
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d) = (h[0], h[1], h[2], h[3]);
        let (mut e, mut f, mut g, mut hh) = (h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K256[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, v) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *slot = slot.wrapping_add(v);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

/// The hash's own plant. Both vectors are NIST's; a hash that fails either is refused
/// before it can refuse a trajectory.
fn sha256_self_test() -> Result<(), String> {
    let empty = sha256(b"");
    let abc = sha256(b"abc");
    if empty != "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" {
        return Err(format!("sha256(\"\") = {empty}"));
    }
    if abc != "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" {
        return Err(format!("sha256(\"abc\") = {abc}"));
    }
    Ok(())
}

// ------------------------------------------------------------------ subsystem solving

/// One subsystem's reading, with every field a gate reads.
#[derive(Clone, Debug)]
struct Sub {
    /// Total energy — electronic plus nuclear repulsion — hartree.
    e: f64,
    n_det: usize,
    residual: f64,
    exit: SolveExit,
    route: SolverRoute,
    /// `None` where the entry point that guarantees the exact route does not report it.
    /// Recorded as unobservable rather than defaulted to true (freeze correction C-2).
    scf: Option<bool>,
    davidson_iters: usize,
}

impl Sub {
    /// Freeze G1 + G2. Returns the VOID reason, or `None` if the solve is admissible.
    fn void_reason(&self) -> Option<String> {
        if self.route != SolverRoute::Determinant {
            return Some(format!("route {} (G1: exactness held)", self.route.label()));
        }
        if !matches!(self.exit, SolveExit::Converged | SolveExit::Trivial) {
            return Some(format!("exit {}", self.exit.label()));
        }
        if self.residual > CONVERGED_RESIDUAL {
            return Some(format!(
                "residual {:.3e} > {:.1e}",
                self.residual, CONVERGED_RESIDUAL
            ));
        }
        None
    }

    /// The clause AMENDMENT A-1 removed from `void_reason`, kept as its own function so
    /// the STRICT reading — G2 exactly as frozen — is still computable on every config
    /// and can be published beside the amended one. A post-data amendment that deleted
    /// its own predecessor would leave nothing to compare against.
    fn strict_scf_void(&self) -> bool {
        self.scf == Some(false)
    }
}

fn species_of(z: u32) -> Species {
    match z {
        1 => HYDROGEN,
        8 => OXYGEN,
        other => panic!("this audit's scope is O and H only; the trajectory carries Z = {other}"),
    }
}

fn centres(pos: &[[f64; 3]]) -> Vec<[D2; 3]> {
    pos.iter()
        .map(|p| [D2::c(p[0]), D2::c(p[1]), D2::c(p[2])])
        .collect()
}

/// Solve one subsystem on the DETERMINANT route, whatever its size.
///
/// Two entry points, one route, and the choice is forced rather than stylistic:
///
/// * at or below `MPS_ROUTE_THRESHOLD`, `solve_geometry` IS the determinant route (its
///   `solve` call cannot branch there) and it additionally reports `scf_converged`, so it
///   is used and the route is CHECKED from `Solution::route` rather than inferred;
/// * above it, `solve_geometry` would silently become DMRG, so the only lawful path is
///   `geometry_problem` + `solve_determinant`, which does not carry the SCF flag. That is
///   the freeze's correction C-2, and the flag is recorded as unobservable there.
fn solve_sub(species: &[Species], pos: &[[f64; 3]]) -> Result<Sub, String> {
    let (space, mo, nuc) = geometry_problem(species, centres(pos));
    let n_det = space.n_det;
    if n_det > FCI_DET_MAX {
        return Err(format!("{n_det} determinants exceeds FCI_DET_MAX {FCI_DET_MAX}"));
    }
    if n_det <= MPS_ROUTE_THRESHOLD {
        let sol = solve_geometry(species, centres(pos));
        return Ok(Sub {
            e: sol.e.v,
            n_det: sol.n_det,
            residual: sol.residual,
            exit: sol.exit,
            route: sol.route,
            scf: Some(sol.scf_converged),
            davidson_iters: sol.davidson_iters,
        });
    }
    let sol = fci::solve_determinant(&space, &mo);
    Ok(Sub {
        e: (sol.e + nuc).v,
        n_det,
        residual: sol.residual,
        exit: sol.exit,
        route: sol.route,
        scf: None,
        davidson_iters: sol.davidson_iters,
    })
}

/// The isolated-atom energy, from `quaternary.rs`'s own cached constants — the same two
/// numbers `ohhh_mbe3_energy` subtracts.
fn atom_e(sp: Species) -> f64 {
    if sp.z == OXYGEN.z {
        atom_energy_o()
    } else {
        atom_energy_h()
    }
}

// ------------------------------------------------------------------------ the assembly

/// Everything one five-cluster produced.
#[derive(Clone, Debug)]
struct Assembly {
    e_fci: f64,
    e_mbe4: f64,
    de5: f64,
    /// The five quadruple terms, in the order "drop atom i".
    de4: [f64; 5],
    de3: Vec<f64>,
    de2: Vec<f64>,
    n_det_max: usize,
    worst_residual: f64,
    residual_sum: f64,
    n_solves: usize,
    davidson_iters: usize,
    scf_unobservable: usize,
    /// Solves that reported `scf_converged = false`. Recorded, not refused (A-1); this is
    /// the count that decides the STRICT reading's VOID for this config.
    scf_not_converged: usize,
    /// The first subsystem that would have VOIDed under G2 as frozen, for the strict
    /// reading's own VOID structure.
    strict_void_at: Option<String>,
    seconds: f64,
}

impl Assembly {
    fn max_abs_de4(&self) -> f64 {
        self.de4.iter().fold(0.0f64, |m, v| m.max(v.abs()))
    }
    fn is_live(&self) -> bool {
        self.max_abs_de4() >= LIVE_BAR
    }
}

/// `dE5` and every rung beneath it, for one five-atom cluster.
///
/// `drop_quadruple` is plant P-1's only lever: `Some(k)` deletes the quadruple term that
/// omits atom `k` from the `E_MBE4` sum and touches nothing else, so the difference
/// between a planted and an unplanted run is that one term and cannot be anything else.
fn assemble(
    species: &[Species; 5],
    pos: &[[f64; 3]; 5],
    drop_quadruple: Option<usize>,
) -> Result<Assembly, String> {
    let t0 = Instant::now();
    let mut worst = 0.0f64;
    let mut rsum = 0.0f64;
    let mut n_solves = 0usize;
    let mut iters = 0usize;
    let mut n_det_max = 0usize;
    let mut scf_unobs = 0usize;
    let mut scf_bad = 0usize;
    let mut strict_at: Option<String> = None;

    // A VOID reason that does not say WHICH subsystem failed is a reason nobody can act
    // on: a config has 26 solves and "scf did not converge" names none of them. The label
    // carries arity, the arena slots, the composition, the determinant count and the
    // subsystem's own minimum interatomic separation, because a failed solve on a pair at
    // 0.1 bohr and a failed solve on a well-separated pentamer are different facts.
    let mut take = |s: &Sub, what: &str| -> Result<(), String> {
        if let Some(r) = s.void_reason() {
            return Err(format!("{what}: {r}"));
        }
        worst = worst.max(s.residual);
        rsum += s.residual;
        n_solves += 1;
        iters += s.davidson_iters;
        n_det_max = n_det_max.max(s.n_det);
        if s.scf.is_none() {
            scf_unobs += 1;
        }
        if s.strict_scf_void() {
            scf_bad += 1;
            if strict_at.is_none() {
                strict_at = Some(format!("{what}: scf did not converge"));
            }
        }
        Ok(())
    };

    // Rung 1 — the five atom terms. `atom_energy` is the same solve `atom_energy_o` and
    // `atom_energy_h` cache; it is re-run here ONLY to read its diagnostics, and the
    // value used below is the cached one, so the two cannot drift.
    let mut e_atoms = 0.0;
    for sp in species.iter() {
        let s = solve_sub(&[*sp], &[[0.0, 0.0, 0.0]])?;
        take(&s, &format!("atom Z{} (n_det {})", sp.z, s.n_det))?;
        e_atoms += atom_e(*sp);
    }

    // Rung 2 — ten pair terms, built exactly as `ohhh_mbe3_energy` builds its six:
    // `pair_point(A, B, r).e - E(A) - E(B)`, with the same cached atom energies.
    let mut de2 = Vec::with_capacity(10);
    let mut pair_of: BTreeMap<(usize, usize), f64> = BTreeMap::new();
    for i in 0..5 {
        for j in (i + 1)..5 {
            let r = dist(&pos[i], &pos[j]);
            let s = solve_sub(&[species[i], species[j]], &[pos[i], pos[j]])?;
            take(
                &s,
                &format!(
                    "pair ({i},{j}) Z{}Z{} r={r:.4} bohr (n_det {})",
                    species[i].z, species[j].z, s.n_det
                ),
            )?;
            // The VALUE comes from the house function, so the bit-equality claim in the
            // freeze is made by calling it rather than by reimplementing it.
            let v = pair_point(species[i], species[j], r).e - atom_e(species[i]) - atom_e(species[j]);
            pair_of.insert((i, j), v);
            de2.push(v);
        }
    }

    // Rung 3 — ten triple terms.
    let mut de3 = Vec::with_capacity(10);
    let mut triple_of: BTreeMap<(usize, usize, usize), f64> = BTreeMap::new();
    for i in 0..5 {
        for j in (i + 1)..5 {
            for k in (j + 1)..5 {
                let sp = [species[i], species[j], species[k]];
                let ps = [pos[i], pos[j], pos[k]];
                let s = solve_sub(&sp, &ps)?;
                take(
                    &s,
                    &format!(
                        "triple ({i},{j},{k}) Z{}Z{}Z{} r_min={:.4} bohr (n_det {})",
                        sp[0].z,
                        sp[1].z,
                        sp[2].z,
                        min_sep(&ps),
                        s.n_det
                    ),
                )?;
                let v = s.e
                    - atom_e(sp[0])
                    - atom_e(sp[1])
                    - atom_e(sp[2])
                    - pair_of[&(i, j)]
                    - pair_of[&(i, k)]
                    - pair_of[&(j, k)];
                triple_of.insert((i, j, k), v);
                de3.push(v);
            }
        }
    }

    // Rung 4 — five quadruple terms, indexed by the atom each one omits.
    let mut de4 = [0.0f64; 5];
    for drop in 0..5 {
        let idx: Vec<usize> = (0..5).filter(|&x| x != drop).collect();
        let sp: Vec<Species> = idx.iter().map(|&x| species[x]).collect();
        let ps: Vec<[f64; 3]> = idx.iter().map(|&x| pos[x]).collect();
        let s = solve_sub(&sp, &ps)?;
        take(
            &s,
            &format!(
                "quadruple (drop {drop}) idx {idx:?} r_min={:.4} bohr (n_det {})",
                min_sep(&ps),
                s.n_det
            ),
        )?;
        let mut v = s.e;
        for &a in &idx {
            v -= atom_e(species[a]);
        }
        for a in 0..4 {
            for b in (a + 1)..4 {
                v -= pair_of[&(idx[a].min(idx[b]), idx[a].max(idx[b]))];
            }
        }
        for a in 0..4 {
            for b in (a + 1)..4 {
                for c in (b + 1)..4 {
                    let mut t = [idx[a], idx[b], idx[c]];
                    t.sort_unstable();
                    v -= triple_of[&(t[0], t[1], t[2])];
                }
            }
        }
        de4[drop] = v;
    }

    // Rung 5 — the pentamer, and the residue.
    let s5 = solve_sub(species, pos)?;
    take(
        &s5,
        &format!(
            "pentamer r_min={:.4} bohr (n_det {}, scf {})",
            min_sep(pos),
            s5.n_det,
            match s5.scf {
                Some(true) => "converged",
                Some(false) => "NOT converged",
                None => "unobservable (freeze correction C-2)",
            }
        ),
    )?;
    let e_fci = s5.e;

    let mut e_mbe4 = e_atoms;
    for v in &de2 {
        e_mbe4 += v;
    }
    for v in &de3 {
        e_mbe4 += v;
    }
    for (k, v) in de4.iter().enumerate() {
        if drop_quadruple == Some(k) {
            continue;
        }
        e_mbe4 += v;
    }

    let seconds = t0.elapsed().as_secs_f64();
    if seconds > WALL_BUDGET_S {
        return Err(format!(
            "wall clock {seconds:.1} s exceeds the G10 budget of {WALL_BUDGET_S:.0} s"
        ));
    }

    Ok(Assembly {
        e_fci,
        e_mbe4,
        de5: e_fci - e_mbe4,
        de4,
        de3,
        de2,
        n_det_max,
        worst_residual: worst,
        residual_sum: rsum,
        n_solves,
        davidson_iters: iters,
        scf_unobservable: scf_unobs,
        scf_not_converged: scf_bad,
        strict_void_at: strict_at,
        seconds,
    })
}

/// The smallest interatomic separation in a subsystem, bohr. A failed solve at 0.1 bohr
/// and a failed solve at 2 bohr are different facts and the VOID reason must say which.
fn min_sep(ps: &[[f64; 3]]) -> f64 {
    let mut m = f64::INFINITY;
    for i in 0..ps.len() {
        for j in (i + 1)..ps.len() {
            m = m.min(dist(&ps[i], &ps[j]));
        }
    }
    m
}

#[inline]
fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

// -------------------------------------------------------------------------- the sample

#[derive(Clone, Debug)]
struct Candidate {
    seed: u64,
    frame: u64,
    idx: [usize; 5],
    comp: (usize, usize),
    diameter: f64,
    pos: [[f64; 3]; 5],
    z: [u32; 5],
}

fn composition(z: &[u32; 5]) -> (usize, usize) {
    let n_o = z.iter().filter(|&&x| x == 8).count();
    let n_h = z.iter().filter(|&&x| x == 1).count();
    (n_o, n_h)
}

fn comp_name(c: (usize, usize)) -> String {
    match c {
        (0, h) => format!("H{h}"),
        (o, 0) => format!("O{o}"),
        (o, h) => format!("O{o}H{h}"),
    }
}

/// The determinant count of a composition, from the same arithmetic the freeze tabulates:
/// `n_orb = 5*nO + nH`, `n_elec = 8*nO + nH`, minimal `Sz` sector.
fn det_count(c: (usize, usize)) -> u128 {
    let n_orb = 5 * c.0 + c.1;
    let n_elec = 8 * c.0 + c.1;
    let na = n_elec.div_ceil(2);
    let nb = n_elec / 2;
    binom(n_orb, na) * binom(n_orb, nb)
}

fn binom(n: usize, k: usize) -> u128 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut r: u128 = 1;
    for i in 0..k {
        r = r * (n - i) as u128 / (i as u128 + 1);
    }
    r
}

/// Freeze section 3: enumerate every candidate in a fully determined order, applying the
/// stride, the compactness fence and the near-duplicate rule. Composition scope is NOT
/// applied here — the out-of-scope rows are the landscape base rate section 3.6 demands.
fn enumerate(traj: &Trajectory, seed: u64) -> Vec<Candidate> {
    let n = traj.header.n_atoms;
    let mut out = Vec::new();
    let mut seen: BTreeMap<([usize; 5], u64), ()> = BTreeMap::new();
    let mut f = 0usize;
    while f < traj.frames.len() {
        let fr = &traj.frames[f];
        let mut d = vec![0.0f64; n * n];
        for i in 0..n {
            for j in (i + 1)..n {
                let v = dist(&fr.pos[i], &fr.pos[j]);
                d[i * n + j] = v;
                d[j * n + i] = v;
            }
        }
        for a in 0..n {
            for b in (a + 1)..n {
                for c in (b + 1)..n {
                    for e in (c + 1)..n {
                        for g in (e + 1)..n {
                            let idx = [a, b, c, e, g];
                            let mut diam: f64 = 0.0;
                            for p in 0..5 {
                                for q in (p + 1)..5 {
                                    diam = diam.max(d[idx[p] * n + idx[q]]);
                                }
                            }
                            if diam >= R_COMPACT {
                                continue;
                            }
                            let block = fr.index / DEDUPE_BLOCK;
                            if seen.insert((idx, block), ()).is_some() {
                                continue;
                            }
                            let z: [u32; 5] = core::array::from_fn(|k| traj.header.z[idx[k]]);
                            let pos: [[f64; 3]; 5] = core::array::from_fn(|k| fr.pos[idx[k]]);
                            out.push(Candidate {
                                seed,
                                frame: fr.index,
                                idx,
                                comp: composition(&z),
                                diameter: diam,
                                pos,
                                z,
                            });
                        }
                    }
                }
            }
        }
        f += STRIDE;
    }
    out
}

/// Freeze section 3.4: per-composition systematic draw, then shortfall redistribution in
/// the fixed priority order, capped at `N_TARGET`. Returns the draw and, per composition,
/// `(enumerated, drawn)` so the excess is printed rather than swallowed.
fn draw(
    all: &[Candidate],
) -> (Vec<Candidate>, BTreeMap<(usize, usize), (usize, usize)>) {
    let mut by: BTreeMap<(usize, usize), Vec<&Candidate>> = BTreeMap::new();
    for c in all {
        by.entry(c.comp).or_default().push(c);
    }
    let mut picked: Vec<Candidate> = Vec::new();
    let mut stats: BTreeMap<(usize, usize), (usize, usize)> = BTreeMap::new();
    for (comp, list) in &by {
        stats.insert(*comp, (list.len(), 0));
    }
    // THE ALLOCATION IS INTEGER ARITHMETIC ON COUNTS, DECIDED BEFORE ANY CANDIDATE IS
    // TOUCHED, and only then is a sample drawn to fill it.
    //
    // The first version interleaved the two — draw the quota, then draw more, skipping
    // what the first pass took — and it OVERRAN THE DECLARED CAP: with 31 candidates in
    // one composition it drew 31 against an N_TARGET of 24. A silent cap is exactly what
    // the freeze's section 3.4 forbids ("the cap is DECLARED, not silent"), and the defect
    // was invisible in the totals until the counts were printed beside the target. Doing
    // the arithmetic first makes the cap structural: the draw cannot exceed what the
    // allocation says, because the allocation is what it draws to.
    let mut alloc: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    for comp in IN_SCOPE {
        let m = by.get(&comp).map(|v| v.len()).unwrap_or(0);
        alloc.insert(comp, QUOTA.min(m));
    }
    let mut remaining = N_TARGET.saturating_sub(alloc.values().sum::<usize>());
    for comp in SHORTFALL_PRIORITY {
        if remaining == 0 {
            break;
        }
        let m = by.get(&comp).map(|v| v.len()).unwrap_or(0);
        let have = *alloc.get(&comp).unwrap_or(&0);
        let add = remaining.min(m.saturating_sub(have));
        alloc.insert(comp, have + add);
        remaining -= add;
    }
    debug_assert!(alloc.values().sum::<usize>() <= N_TARGET);

    // A systematic sample of exactly `q` of `m`, spanning the whole list: index
    // `floor(i * m / q)` for `i` in `0..q`. Strictly increasing whenever `q <= m`, so it
    // returns `q` DISTINCT candidates spread across the run rather than its first minutes.
    let systematic = |list: &[&Candidate], q: usize| -> Vec<Candidate> {
        let m = list.len();
        if m == 0 || q == 0 {
            return Vec::new();
        }
        let q = q.min(m);
        (0..q).map(|i| list[i * m / q].clone()).collect()
    };
    for comp in IN_SCOPE {
        let list = by.get(&comp).cloned().unwrap_or_default();
        let got = systematic(&list, *alloc.get(&comp).unwrap_or(&0));
        stats.entry(comp).or_insert((list.len(), 0)).1 = got.len();
        picked.extend(got);
    }
    assert!(
        picked.len() <= N_TARGET,
        "the draw must never exceed its declared cap: {} > {N_TARGET}",
        picked.len()
    );
    (picked, stats)
}

// ------------------------------------------------------------------------------- main

struct Args {
    traj_dir: PathBuf,
    manifest: PathBuf,
    out: PathBuf,
    plants: bool,
    audit: bool,
    scf_probe: bool,
    crosscheck: bool,
    limit: Option<usize>,
}

fn parse_args() -> Args {
    let mut a = Args {
        traj_dir: PathBuf::from("/home/emoore/holon-artifacts/census-traj"),
        manifest: PathBuf::new(),
        out: PathBuf::from("de5_audit.csv"),
        plants: false,
        audit: false,
        scf_probe: false,
        crosscheck: true,
        limit: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(k) = it.next() {
        match k.as_str() {
            "--traj-dir" => a.traj_dir = PathBuf::from(it.next().expect("--traj-dir needs a path")),
            "--manifest" => a.manifest = PathBuf::from(it.next().expect("--manifest needs a path")),
            "--out" => a.out = PathBuf::from(it.next().expect("--out needs a path")),
            "--plants" => a.plants = true,
            "--scf-probe" => a.scf_probe = true,
            "--audit" => a.audit = true,
            "--no-crosscheck" => a.crosscheck = false,
            "--limit" => {
                a.limit = Some(it.next().expect("--limit needs a number").parse().expect("integer"))
            }
            other => panic!(
                "unknown argument {other:?}. This CLI REFUSES what it does not understand: an \
                 ignored flag makes one command mean two things in two trees, and the tell is \
                 output that did not change when the operator expected it to."
            ),
        }
    }
    if a.manifest.as_os_str().is_empty() {
        panic!("--manifest is required: the trajectory pin is not optional (freeze G4)");
    }
    a
}

fn main() {
    let args = parse_args();
    println!("=== dE5 TRUNCATION AUDIT — GANTT node H ===");
    println!("freeze     conformance/water_observatory/DE5_PREREG.md (committed ahead of this file)");
    println!("device_class      {DEVICE_CLASS}");
    println!("solver_budget     {}", fci::davidson_budget());
    println!("subtraction_basis {SUBTRACTION_BASIS}");
    println!(
        "staked     R_COMPACT {R_COMPACT} bohr  STRIDE {STRIDE}  DEDUPE_BLOCK {DEDUPE_BLOCK} \
         QUOTA {QUOTA}  N_TARGET {N_TARGET}"
    );
    println!(
        "staked     FCI_DET_MAX {FCI_DET_MAX}  BOUND {BOUND:.1e} Ha  LIVE_BAR {LIVE_BAR:.1e} Ha  \
         WALL_BUDGET {WALL_BUDGET_S:.0} s"
    );
    println!("args       traj-dir {:?}  manifest {:?}  out {:?}  plants {}  audit {}  crosscheck {}  limit {:?}",
             args.traj_dir, args.manifest, args.out, args.plants, args.audit, args.crosscheck, args.limit);

    match sha256_self_test() {
        Ok(()) => println!("sha256     self-test PASS (NIST vectors for \"\" and \"abc\")"),
        Err(e) => panic!("sha256 self-test FAILED: {e}"),
    }

    // ---- the pair-rung bit-equality check (freeze section 1.1), run once, printed.
    pair_rung_identity_check();

    // ---- G4: the pin.
    let pins = read_manifest(&args.manifest);
    let mut trajs: Vec<(u64, PathBuf)> = Vec::new();
    for (rel, want) in &pins {
        if !rel.starts_with("fenced/") {
            continue;
        }
        let p = args.traj_dir.join(rel);
        let bytes = std::fs::read(&p).unwrap_or_else(|e| panic!("cannot read {p:?}: {e}"));
        let got = sha256(&bytes);
        if &got != want {
            panic!("G4 REFUSES {p:?}\n  manifest {want}\n  measured {got}");
        }
        let t = Trajectory::read(&p).expect("HLNTRAJ1");
        println!(
            "pin OK     {rel}  sha256 {}  n_atoms {}  dims {}  frames {}",
            &got[..16],
            t.header.n_atoms,
            t.header.dims,
            t.frames.len()
        );
        assert_eq!(t.header.dims, 2, "freeze section 2.3: the source is planar");
        trajs.push((t.header.seed, p));
    }
    assert_eq!(trajs.len(), 8, "the freeze pins eight fenced seeds");

    if args.audit {
        // Pin-only mode: G4 and the self-tests, nothing solved.
        println!("\n--audit: pin and self-tests only, no solves.");
        return;
    }

    // ---- enumerate, then draw.
    let mut all: Vec<Candidate> = Vec::new();
    for (_seed, p) in &trajs {
        let t = Trajectory::read(p).expect("HLNTRAJ1");
        let mut v = enumerate(&t, t.header.seed);
        all.append(&mut v);
    }
    let mut landscape: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    for c in &all {
        *landscape.entry(c.comp).or_insert(0) += 1;
    }
    println!("\n=== THE LANDSCAPE (freeze 3.6: every compact candidate, in scope or not) ===");
    println!("  {:<8} {:>10} {:>16} {:>10}", "comp", "candidates", "determinants", "in scope");
    let mut total = 0usize;
    for (c, n) in &landscape {
        total += n;
        let d = det_count(*c);
        println!(
            "  {:<8} {:>10} {:>16} {:>10}",
            comp_name(*c),
            n,
            d,
            if IN_SCOPE.contains(c) { "yes" } else { "NO" }
        );
    }
    println!("  {:<8} {:>10}", "TOTAL", total);

    let (picked, stats) = draw(&all);
    println!("\n=== THE DRAW (freeze 3.4; the cap is declared and the excess counted) ===");
    println!("  {:<8} {:>10} {:>8} {:>10}", "comp", "enumerated", "drawn", "excess");
    for comp in IN_SCOPE {
        let (m, q) = stats.get(&comp).copied().unwrap_or((0, 0));
        println!("  {:<8} {:>10} {:>8} {:>10}", comp_name(comp), m, q, m - q);
    }
    println!("  drawn total {} (N_TARGET {N_TARGET})", picked.len());

    if args.scf_probe {
        scf_probe(&picked);
        return;
    }

    if args.plants {
        run_plants(&picked);
        return;
    }

    // ---- score.
    let tables = if args.crosscheck { load_tables() } else { None };
    let mut csv = std::fs::File::create(&args.out).expect("csv");
    writeln!(
        csv,
        "# dE5 audit — freeze conformance/water_observatory/DE5_PREREG.md\n\
         # device_class={DEVICE_CLASS} solver_budget={} subtraction_basis={SUBTRACTION_BASIS}\n\
         # R_COMPACT={R_COMPACT} STRIDE={STRIDE} DEDUPE_BLOCK={DEDUPE_BLOCK} QUOTA={QUOTA} \
         N_TARGET={N_TARGET} FCI_DET_MAX={FCI_DET_MAX} BOUND={BOUND:.3e} LIVE_BAR={LIVE_BAR:.3e}\n\
         seed,frame,idx,comp,diameter,status,void_reason,e_fci,e_mbe4,de5,abs_de5,max_abs_de4,\
         ratio,live,under_bound,n_det_max,worst_residual,residual_sum,n_solves,davidson_iters,\
         scf_unobservable,scf_not_converged,strict_void,seconds",
        fci::davidson_budget()
    )
    .unwrap();

    let n = args.limit.unwrap_or(picked.len()).min(picked.len());
    let mut scored = 0usize;
    let mut live = 0usize;
    let mut vacuous = 0usize;
    let mut voids: Vec<(String, String)> = Vec::new();
    let mut strict_voids: Vec<(String, String)> = Vec::new();
    let mut scf_bad_total = 0usize;
    let mut worst: Option<(f64, Candidate, Assembly)> = None;
    println!("\n=== SCORING {n} configs ===");
    for c in picked.iter().take(n) {
        let sp: [Species; 5] = core::array::from_fn(|k| species_of(c.z[k]));
        let t0 = Instant::now();
        let res = assemble(&sp, &c.pos, None);
        match res {
            Err(why) => {
                voids.push((comp_name(c.comp), why.clone()));
                println!(
                    "  VOID   {} seed {:#x} frame {:>6} idx {:?}  {}",
                    comp_name(c.comp),
                    c.seed,
                    c.frame,
                    c.idx,
                    why
                );
                writeln!(
                    csv,
                    "{:#x},{},{},{},{:.6},VOID,\"{}\",,,,,,,,,,,,,,,,,{:.1}",
                    c.seed,
                    c.frame,
                    c.idx.map(|x| x.to_string()).join("+"),
                    comp_name(c.comp),
                    c.diameter,
                    why,
                    t0.elapsed().as_secs_f64()
                )
                .unwrap();
            }
            Ok(a) => {
                scored += 1;
                let is_live = a.is_live();
                if is_live {
                    live += 1;
                } else {
                    vacuous += 1;
                }
                let ratio = if a.max_abs_de4() > 0.0 {
                    a.de5.abs() / a.max_abs_de4()
                } else {
                    f64::NAN
                };
                let status = if is_live { "LIVE" } else { "VACUOUS" };
                println!(
                    "  {status:<7} {} seed {:#x} frame {:>6}  diam {:.3}  dE5 {:+.6e}  max|dE4| {:.6e}  \
                     ratio {:.4}  {}  [{} solves, {:.1} s]",
                    comp_name(c.comp),
                    c.seed,
                    c.frame,
                    c.diameter,
                    a.de5,
                    a.max_abs_de4(),
                    ratio,
                    if a.de5.abs() < BOUND { "under bound" } else { "OVER BOUND" },
                    a.n_solves,
                    a.seconds
                );
                writeln!(
                    csv,
                    "{:#x},{},{},{},{:.6},{},,{:.12},{:.12},{:.6e},{:.6e},{:.6e},{:.6e},{},{},{},{:.3e},{:.3e},{},{},{},{},{},{:.1}",
                    c.seed,
                    c.frame,
                    c.idx.map(|x| x.to_string()).join("+"),
                    comp_name(c.comp),
                    c.diameter,
                    status,
                    a.e_fci,
                    a.e_mbe4,
                    a.de5,
                    a.de5.abs(),
                    a.max_abs_de4(),
                    ratio,
                    is_live,
                    a.de5.abs() < BOUND,
                    a.n_det_max,
                    a.worst_residual,
                    a.residual_sum,
                    a.n_solves,
                    a.davidson_iters,
                    a.scf_unobservable,
                    a.scf_not_converged,
                    a.strict_void_at.is_some(),
                    a.seconds
                )
                .unwrap();
                scf_bad_total += a.scf_not_converged;
                if let Some(w) = &a.strict_void_at {
                    strict_voids.push((comp_name(c.comp), w.clone()));
                }
                if is_live && worst.as_ref().map(|w| a.de5.abs() > w.0).unwrap_or(true) {
                    worst = Some((a.de5.abs(), c.clone(), a.clone()));
                }
                if let Some((w, tr)) = tables.as_ref() {
                    crosscheck_oh3(c, &sp, &a, w, tr);
                }
            }
        }
    }

    // ---- the verdict.
    println!("\n=== VOID STRUCTURE (freeze section 6) ===");
    if voids.is_empty() {
        println!("  none");
    } else {
        let mut by: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (c, r) in &voids {
            by.entry(c.clone()).or_default().push(r.clone());
        }
        for (c, rs) in &by {
            println!("  {c}: {} VOID", rs.len());
            for r in rs {
                println!("      {r}");
            }
        }
    }
    println!("\n=== THE STRICT READING (G2 exactly as frozen, before amendment A-1) ===");
    println!(
        "  configs VOIDed by the scf_converged clause alone: {} of {n}",
        strict_voids.len()
    );
    println!("  solves reporting scf_converged = false: {scf_bad_total}");
    println!(
        "  STRICT VERDICT: {}",
        if n.saturating_sub(voids.len() + strict_voids.len()) < 20 {
            "BRANCH (d) VOID — no verdict about the ladder"
        } else {
            "the strict clause does not change the branch"
        }
    );
    println!("  (published beside the amended reading every time — freeze section 5b)");

    println!("\n=== THE READING (amended, A-1) ===");
    println!("  scored {scored}   LIVE {live}   VACUOUS {vacuous}   VOID {}", voids.len());
    println!("  (the maximum below is a bound on the SCORED set only — M-MAX-OVER-SUCCESSES)");
    if let Some((w, c, a)) = &worst {
        println!("  worst LIVE |dE5| = {w:.6e} Ha   bound {BOUND:.1e}   {}",
                 if *w < BOUND { "UNDER" } else { "OVER" });
        println!("  worst config: {} seed {:#x} frame {} idx {:?} diameter {:.6} bohr",
                 comp_name(c.comp), c.seed, c.frame, c.idx, c.diameter);
        for k in 0..5 {
            println!("    atom {} Z={} at [{:.12}, {:.12}, {:.12}]",
                     c.idx[k], c.z[k], c.pos[k][0], c.pos[k][1], c.pos[k][2]);
        }
        println!("    dE4 terms: {:?}", a.de4);
        // The rungs beneath, so the ladder's own decay is visible on the worst case
        // rather than only its top. A dE5 that is large where dE4 and dE3 are ALSO large
        // is a converging ladder read at a hard geometry; one that is large where the
        // rungs beneath it are small is a different finding entirely.
        let s2: f64 = a.de2.iter().sum();
        let s3: f64 = a.de3.iter().sum();
        let s4: f64 = a.de4.iter().sum();
        let m2 = a.de2.iter().fold(0.0f64, |m, v| m.max(v.abs()));
        let m3 = a.de3.iter().fold(0.0f64, |m, v| m.max(v.abs()));
        println!(
            "    rung sums:  dE2 {s2:+.9e} (max |{m2:.3e}|)   dE3 {s3:+.9e} (max |{m3:.3e}|)   \
             dE4 {s4:+.9e} (max |{:.3e}|)",
            a.max_abs_de4()
        );
        println!("    E_FCI {:.12}   E_MBE4 {:.12}   dE5 {:+.9e}", a.e_fci, a.e_mbe4, a.de5);
    } else {
        println!("  no LIVE config scored — branch (d)");
    }
    let verdict = if live < 20 {
        "BRANCH (d) VOID — fewer than 20 LIVE configs; no verdict about the ladder"
    } else if worst.map(|w| w.0 < BOUND).unwrap_or(false) {
        "BRANCH (a) TERMINATES — planar, STO-3G, {H5,OH4,O2H3}, diameter < 6.0 bohr"
    } else {
        "BRANCH (b) DOES NOT TERMINATE — the DMRG-cluster seam requirement fires (GANTT MPS node)"
    };
    println!("\n  VERDICT: {verdict}");
    println!("\n  csv written to {:?}", args.out);
}

/// Freeze section 1.1's pair-rung claim, made by MEASUREMENT rather than by assertion.
///
/// The claim has two halves and each needs a check that could fail:
///
/// 1. the VALUE this audit's pair rung uses is the one `quaternary::ohhh_mbe3_energy`
///    uses — established by calling the same `pair_point` and the same cached atom
///    energies, so there is one expression and not two;
/// 2. the DIAGNOSTICS this audit reads (`route`, `exit`, `n_det`) belong to that same
///    solve. `pair_point` returns no diagnostics, so the audit reads them from
///    `solve_geometry` on the identical dual-variable placement. That is a second entry
///    point, and whether the two agree is a measurement, not an assumption: the check
///    below compares them BIT FOR BIT and would fire if `pair_point` ever grew a
///    different placement, a warm start, or its own budget.
///
/// The first version of this function compared `mine` against `house` where both sides
/// were the same expression — a check that establishes one direction and reports two, and
/// could not have failed. It is replaced rather than kept, and named here so the shape is
/// on the record.
fn pair_rung_identity_check() {
    let cases = [
        (OXYGEN, HYDROGEN, 1.9909),
        (HYDROGEN, HYDROGEN, 1.3887),
        (OXYGEN, OXYGEN, 2.4421),
    ];
    println!("pair rung  two entry points, one number (bit equality), and the route CHECKED:");
    for (a, b, r) in cases {
        let via_pair_point = pair_point(a, b, r).e;
        let sol = solve_geometry(
            &[a, b],
            vec![
                [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
                [D2::c(0.0), D2::c(0.0), D2::var(r)],
            ],
        );
        let via_solve_geometry = sol.e.v;
        assert_eq!(
            via_pair_point.to_bits(),
            via_solve_geometry.to_bits(),
            "the value path and the diagnostic path must be the same solve: \
             pair_point gave {via_pair_point:.17e}, solve_geometry gave {via_solve_geometry:.17e}"
        );
        let de2 = via_pair_point - atom_e(a) - atom_e(b);
        println!(
            "  Z{}-Z{} at r={r}: E {:.12}  dE2 {:.12}  n_det {} (<= {MPS_ROUTE_THRESHOLD}, so \
             solve IS solve_determinant)  route {}  exit {}",
            a.z,
            b.z,
            via_pair_point,
            de2,
            sol.n_det,
            sol.route.label(),
            sol.exit.label()
        );
        assert!(sol.n_det <= MPS_ROUTE_THRESHOLD);
        assert_eq!(sol.route, SolverRoute::Determinant);
    }
}

fn read_manifest(p: &Path) -> Vec<(String, String)> {
    let t = std::fs::read_to_string(p).unwrap_or_else(|e| panic!("cannot read manifest {p:?}: {e}"));
    let mut v = Vec::new();
    for line in t.lines() {
        let mut it = line.split_whitespace();
        if let (Some(h), Some(f)) = (it.next(), it.next()) {
            if h.len() == 64 {
                v.push((f.to_string(), h.to_string()));
            }
        }
    }
    v
}

fn load_tables() -> Option<(WaterTable, TrimerTable)> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/s2/s2_water_table.txt");
    let src = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            println!("G9 SKIP: the committed (O,H,H) table did not load: {e}");
            return None;
        }
    };
    let w = water::from_text(&src)?;
    println!("G9: (O,H,H) table loaded; building the (H,H,H) table...");
    let tri = trimer::generate()?;
    println!("G9: both served surfaces ready");
    Some((w, tri))
}

/// G9 — the two-basis comparison on every `OHHH` quadruple, printed, never averaged.
fn crosscheck_oh3(
    c: &Candidate,
    sp: &[Species; 5],
    a: &Assembly,
    w: &WaterTable,
    tri: &TrimerTable,
) {
    for drop in 0..5 {
        let idx: Vec<usize> = (0..5).filter(|&x| x != drop).collect();
        let zs: Vec<u32> = idx.iter().map(|&x| sp[x].z).collect();
        if zs.iter().filter(|&&z| z == 8).count() != 1 || zs.len() != 4 {
            continue;
        }
        // quaternary.rs's slot order is [O, H, H, H].
        let o = idx[zs.iter().position(|&z| z == 8).unwrap()];
        let hs: Vec<usize> = idx.iter().copied().filter(|&x| sp[x].z == 1).collect();
        let centres = [c.pos[o], c.pos[hs[0]], c.pos[hs[1]], c.pos[hs[2]]];
        let served = de4_ohhh_fci(&centres, w, tri);
        let live = a.de4[drop];
        println!(
            "    G9 OHHH(drop {drop}): live {live:+.6e}  served {served:+.6e}  diff {:+.6e} \
             {}",
            live - served,
            if (live - served).abs() > 1.0e-3 { "<-- FINDING, above the served-table budget" } else { "" }
        );
    }
}


/// THE SCF PROBE — a diagnostic, and the evidence base for any amendment to G2.
///
/// The first four scored configs all VOIDed on the SAME clause and the SAME rung: an
/// `(O,H,H)` triple at an ordinary water geometry (r_min ~ 2.0-2.2 bohr) reporting
/// `scf_converged = false`. A gate that refuses every config is measuring itself, so
/// before anything is amended the question has to be answered by measurement: does that
/// flag say anything about the ENERGY?
///
/// This probe answers it against an INDEPENDENT reference — the committed `(O,H,H)`
/// table, generated by another campaign on another day through `examples/s2_table.rs` —
/// rather than by re-deriving the same number down the same path. For every `(O,H,H)`
/// triple in the drawn sample it prints the SCF flag, this audit's live `dE3`, the served
/// table's `dE3`, and the difference. If the live values agree with the table across BOTH
/// values of the flag, the flag is not about the energy; if the non-converged ones are
/// the ones that disagree, it is, and the gate stands exactly as staked.
fn scf_probe(picked: &[Candidate]) {
    println!("\n=== SCF PROBE (a diagnostic; no verdict is computed here) ===");
    let Some((w, _tri)) = load_tables() else {
        println!("  the committed (O,H,H) table did not load; the probe cannot run");
        return;
    };
    println!(
        "  {:<9} {:>7} {:>8} {:>8} {:>8} {:>6} {:>17} {:>17} {:>12}",
        "triple", "n_det", "r_OH1", "r_OH2", "r_HH", "scf", "dE3 live", "dE3 served", "diff"
    );
    let mut n_true = 0usize;
    let mut n_false = 0usize;
    let mut worst_true = 0.0f64;
    let mut worst_false = 0.0f64;
    let mut arity_scf: BTreeMap<usize, (usize, usize)> = BTreeMap::new();
    for c in picked {
        let sp: [Species; 5] = core::array::from_fn(|k| species_of(c.z[k]));
        let mut pair_of: BTreeMap<(usize, usize), f64> = BTreeMap::new();
        for i in 0..5 {
            for j in (i + 1)..5 {
                let s = solve_geometry(&[sp[i], sp[j]], centres(&[c.pos[i], c.pos[j]]));
                let e = arity_scf.entry(2).or_insert((0, 0));
                if s.scf_converged {
                    e.0 += 1
                } else {
                    e.1 += 1
                }
                pair_of.insert((i, j), s.e.v - atom_e(sp[i]) - atom_e(sp[j]));
            }
        }
        for i in 0..5 {
            for j in (i + 1)..5 {
                for k in (j + 1)..5 {
                    let zs = [sp[i].z, sp[j].z, sp[k].z];
                    let ps = [c.pos[i], c.pos[j], c.pos[k]];
                    let s = solve_geometry(&[sp[i], sp[j], sp[k]], centres(&ps));
                    let e = arity_scf.entry(3).or_insert((0, 0));
                    if s.scf_converged {
                        e.0 += 1
                    } else {
                        e.1 += 1
                    }
                    if zs.iter().filter(|&&z| z == 8).count() != 1 {
                        continue;
                    }
                    let live = s.e.v
                        - atom_e(sp[i])
                        - atom_e(sp[j])
                        - atom_e(sp[k])
                        - pair_of[&(i, j)]
                        - pair_of[&(i, k)]
                        - pair_of[&(j, k)];
                    let idx = [i, j, k];
                    let o = idx[zs.iter().position(|&z| z == 8).unwrap()];
                    let hs: Vec<usize> = idx.iter().copied().filter(|&x| sp[x].z == 1).collect();
                    let (r1, r2, r12) = (
                        dist(&c.pos[o], &c.pos[hs[0]]),
                        dist(&c.pos[o], &c.pos[hs[1]]),
                        dist(&c.pos[hs[0]], &c.pos[hs[1]]),
                    );
                    let served = w.eval(r1, r2, r12).0;
                    let d = live - served;
                    if s.scf_converged {
                        n_true += 1;
                        worst_true = worst_true.max(d.abs());
                    } else {
                        n_false += 1;
                        worst_false = worst_false.max(d.abs());
                    }
                    println!(
                        "  {:<9} {:>7} {:>8.4} {:>8.4} {:>8.4} {:>6} {:>+17.9e} {:>+17.9e} {:>+12.3e}",
                        format!("({i},{j},{k})"),
                        s.n_det,
                        r1,
                        r2,
                        r12,
                        s.scf_converged,
                        live,
                        served,
                        d
                    );
                }
            }
        }
    }
    println!("\n  SCF flag by arity (converged / NOT converged):");
    for (a, (t_, f_)) in &arity_scf {
        println!("    arity {a}: {t_} converged, {f_} NOT");
    }
    println!("\n  (O,H,H) live-vs-served agreement, split BY THE FLAG:");
    println!("    scf converged     n = {n_true:>4}   worst |live - served| = {worst_true:.3e} Ha");
    println!("    scf NOT converged n = {n_false:>4}   worst |live - served| = {worst_false:.3e} Ha");
    println!("    the served table's own held-out maximum is 3.3e-4 Ha (water.rs: at most a third");
    println!("    of T1's 1e-3 kill), and that is the yardstick both rows are read against.");
}

// ------------------------------------------------------------------------- the plants

fn run_plants(picked: &[Candidate]) {
    println!("\n=== PLANTS (freeze section 8) ===");
    // Pick the first drawn config that can carry P-1: a quadruple term above PLANT_MIN.
    let mut chosen: Option<(Candidate, Assembly, usize)> = None;
    for c in picked {
        let sp: [Species; 5] = core::array::from_fn(|k| species_of(c.z[k]));
        let Ok(a) = assemble(&sp, &c.pos, None) else { continue };
        let (k, v) = a
            .de4
            .iter()
            .enumerate()
            .max_by(|x, y| x.1.abs().partial_cmp(&y.1.abs()).unwrap())
            .map(|(k, v)| (k, *v))
            .unwrap();
        if v.abs() < PLANT_MIN {
            println!(
                "  P-1 REFUSES {} seed {:#x} frame {}: largest |dE4| = {:.3e} < {PLANT_MIN:.1e}; a \
                 plant smaller than the arithmetic could not be seen",
                comp_name(c.comp), c.seed, c.frame, v.abs()
            );
            continue;
        }
        chosen = Some((c.clone(), a, k));
        break;
    }
    let Some((c, base, k)) = chosen else {
        println!("  no drawn config carries a plantable quadruple; plants NOT run");
        return;
    };
    let sp: [Species; 5] = core::array::from_fn(|x| species_of(c.z[x]));
    println!(
        "  carrier: {} seed {:#x} frame {} idx {:?}   quadruple #{k} (drops atom {})",
        comp_name(c.comp), c.seed, c.frame, c.idx, c.idx[k]
    );

    // ---- P-1: drop that quadruple, and only that.
    let planted = assemble(&sp, &c.pos, Some(k)).expect("the planted assembly solves");
    let shift = planted.de5 - base.de5;
    let want = base.de4[k];
    println!("\n  P-1  the dropped quadruple — must shift EXACTLY");
    println!("    sector the plant acts on: the FOUR-BODY sector; carrier dE4(Q*) = {want:+.12e} Ha");
    println!("    (nonzero in that sector: |dE4(Q*)| = {:.3e} >= PLANT_MIN {PLANT_MIN:.1e})", want.abs());
    println!("    dE5 unplanted  {:+.12e}", base.de5);
    println!("    dE5 planted    {:+.12e}", planted.de5);
    println!("    shift          {:+.12e}", shift);
    println!("    expected       {:+.12e}   (= +dE4(Q*))", want);
    println!("    |shift - dE4(Q*)| = {:.3e}   tolerance {P1_TOL:.1e}", (shift - want).abs());
    println!(
        "    P-1 {}",
        if (shift - want).abs() < P1_TOL { "FIRES — the instrument sees what it measures" } else { "FAILED" }
    );

    // ---- P-2: separate one atom; the five-body term must vanish identically.
    let mut far = c.pos;
    let cen = {
        let mut s = [0.0f64; 3];
        for p in c.pos.iter() {
            for x in 0..3 {
                s[x] += p[x] / 5.0;
            }
        }
        s
    };
    far[4] = [cen[0] + P2_SEP, cen[1], cen[2]];
    println!("\n  P-2  the separated atom — must read ZERO");
    println!("    sector the plant acts on: the GEOMETRY sector, nonzero there by {P2_SEP} bohr,");
    println!("    while the FIVE-BODY sector must read exactly zero (the MBE is size-consistent,");
    println!("    so a four-cluster plus a distant atom has an identically vanishing dE5).");
    match assemble(&sp, &far, None) {
        Ok(a) => println!(
            "    dE5 = {:+.6e} Ha   tolerance {P2_TOL:.1e}   P-2 {}",
            a.de5,
            if a.de5.abs() < P2_TOL { "FIRES" } else { "FAILED" }
        ),
        Err(e) => println!("    P-2 VOID: {e}"),
    }

    // ---- P-3: exhaust the budget; the config must VOID and never be scored.
    println!("\n  P-3  the exhausted budget — must VOID");
    println!("    sector the plant acts on: the SOLVER-EXIT sector, where it must be nonzero");
    println!("    (IterationCap where the unplanted run reports Converged).");
    let saved = fci::davidson_budget();
    fci::DAVIDSON_MAX_ITER.store(2, std::sync::atomic::Ordering::Relaxed);
    let r = assemble(&sp, &c.pos, None);
    fci::DAVIDSON_MAX_ITER.store(saved, std::sync::atomic::Ordering::Relaxed);
    match r {
        Err(why) => println!("    VOID as staked: {why}\n    P-3 FIRES — the exhausted case is refused, not scored"),
        Ok(a) => println!("    P-3 FAILED: a budget of 2 iterations produced a SCORED dE5 = {:+.6e}", a.de5),
    }
    println!("\n  budget restored to {}", fci::davidson_budget());
}
