//! THE TRAJECTORY ARTIFACT, v2: arbitrary N, forces and the intervention ledger optional,
//! and the scene's dimensionality MEASURED rather than believed.
//!
//! v1 ([`crate::traj`]) is not deprecated and is not rewritten. It is READ here, and the
//! reading is held to byte identity: `Trajectory2::read` on a v1 file, re-serialised by
//! [`Trajectory2::write_as_v1`], reproduces the original file's bytes. That is the whole
//! discipline of this module — a format change that touches banked artifacts is exactly
//! where silent reinterpretation lives, so the compatibility claim is a digest, not a
//! promise.
//!
//! ## The three exits this format pays
//!
//! **The 16-atom cap.** v1's `u128` pair bitset holds `C(16,2) = 120` bits and the writer
//! refuses past it. `RUNG2_RESULTS.md`'s successor bar is written at a hundred atoms per
//! cell, so the cap is not a nuisance, it is a wall in front of the next campaign. v2
//! carries the bond set as a SPARSE ASCENDING LIST of pair indices: the bonded set is a
//! subset of the neighbour list, which is `O(N)` at a declared cutoff, so the list is
//! `O(N)` where the bitset is `O(N²)`. A wider fixed bitset would have been the wrong kind
//! of answer — `Boundaries.no_fixed_width_carrier` is that argument's shape, and moving a
//! refusal from 128 bits to 4096 does not remove it. This format's refusal is at the
//! `u32` pair index, [`MAX_ATOMS`], and it is named rather than hidden.
//!
//! **No forces and no intervention ledger.** `RUNG2_RESULTS.md` §5.3 marks G9b and G9c
//! UNDISCHARGED — "not computable from this artifact" — because the dump carried positions,
//! velocities, bond bits, time and temperature and nothing else. The engine already tracks
//! everything the two ledgers need (`Sim::p0`, `Sim::j_ext`, `Sim::w_ext`, `Sim::l0`); it
//! was only ever the artifact that dropped them. [`Ledger`] carries them, so a reader that
//! never constructs a `Sim` can close both.
//!
//! **A declared dimensionality nobody measured.** `CENSUS_RESULTS.md` §14.4: seventeen of
//! eighteen banked trajectories hold `z` at placement BIT-EXACTLY for 20,000 frames while
//! declaring `dims = 2`, and the one arm that left the plane was therefore not comparable
//! to them. [`Header2::dims_declared`] records what the caller asserted and is the one
//! field this format labels untrustworthy; [`Trajectory2::measure`] computes the answer
//! from the frames and every reader is expected to use that instead.
//!
//! ## Layout, little-endian throughout
//!
//! ```text
//! magic         8 bytes   b"HLNTRAJ2"
//! version       u32       2
//! content       u32       bitmask: 1 = forces, 2 = ledger; FIXED for the whole file
//! n_atoms       u32
//! dims_declared u32       recorded, never trusted
//! substeps      u32
//! seed          u64
//! n_frames      u64       what the writer INTENDED; the reader counts what is there
//! dt            f64       the PLACEMENT timestep — nothing may derive a duration from it
//! box_w/h/d     f64 x3    bohr
//! z             u32 x n_atoms
//! then n_frames frames, each:
//!   index       u64
//!   time        f64       atomic units, exactly as `Sim::time` carries it
//!   temperature f64       kelvin
//!   n_bonds     u32
//!   bonds       u32 x n_bonds     ASCENDING pair indices, the engine's own enumeration
//!   atoms       f64 x 6 x n_atoms x, y, z, vx, vy, vz
//!   [forces]    f64 x 3 x n_atoms present iff content & 1
//!   [ledger]    f64 x 8           present iff content & 2
//! ```
//!
//! `content` is a HEADER field and not a per-frame flag on purpose. A run either recorded
//! forces or it did not; a per-frame flag would let one file mean two things halfway
//! through, which is the silent reinterpretation the version tag exists to prevent.

use crate::traj::{BondSet, self, pair_index, Header, Trajectory, TrajWriter};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

pub const MAGIC: [u8; 8] = *b"HLNTRAJ2";
pub const VERSION: u32 = 2;

/// Per-frame forces are recorded.
pub const CONTENT_FORCES: u32 = 1;
/// Per-frame external-impulse and external-work columns are recorded.
pub const CONTENT_LEDGER: u32 = 2;
/// Every bit this version of the format knows. A file setting anything else is refused
/// rather than read with the unknown sections skipped, because a section whose length
/// this reader cannot compute makes every byte after it a guess.
pub const CONTENT_KNOWN: u32 = CONTENT_FORCES | CONTENT_LEDGER;

/// The largest scene whose pair indices fit a `u32`.
///
/// `C(92682, 2) = 4_294_930_821 < 2^32` and `C(92683, 2) = 4_295_023_503` does not. This
/// is an envelope and it is stated rather than hidden: `Boundaries.no_fixed_width_carrier`
/// proves every fixed-width carrier has one, so the honest move is to name the location.
/// It is 231x the largest scene the carrier campaign produces.
pub const MAX_ATOMS: usize = 92_682;

/// One atomic time unit in femtoseconds — v1's constant, re-exported rather than re-typed.
pub use crate::traj::AU_TIME_FS;

/// Why a trajectory could not be read or written.
///
/// Every variant carries what was FOUND, not just that something was wrong, and
/// [`TrajError::exit_code`] gives each a distinct process exit status. A single exit
/// code 1 for everything cannot tell a caller which failure it hit
/// (`M-EXIT-DISCRIMINATOR`), and a refusal that does not name what it found cannot be
/// distinguished from a refusal that guessed.
#[derive(Debug)]
pub enum TrajError {
    Io(std::io::Error),
    /// The first eight bytes are not a trajectory magic this reader knows.
    BadMagic { found: [u8; 8] },
    /// The magic was known but the version word was not.
    BadVersion { found: u32, expected: u32 },
    /// A v1 file reached a path that speaks only v2, or the reverse. Never reinterpreted.
    WrongVersion { found: u32, wanted: u32 },
    /// The scene is larger than the pair-index envelope.
    Envelope { n_atoms: usize, max: usize },
    /// The value cannot be written in v1 without losing something.
    NotRepresentableAsV1(String),
    /// The file's own fields contradict each other.
    Malformed(String),
}

impl TrajError {
    /// The exit status a launcher should carry out of this refusal. Fixed in
    /// `CARRIER_V2_PREREG.md` §6 and not to be changed without changing that document.
    pub fn exit_code(&self) -> i32 {
        match self {
            TrajError::Io(_) => 3,
            TrajError::BadMagic { .. }
            | TrajError::BadVersion { .. }
            | TrajError::WrongVersion { .. }
            | TrajError::NotRepresentableAsV1(_)
            | TrajError::Malformed(_) => 4,
            TrajError::Envelope { .. } => 7,
        }
    }
}

impl std::fmt::Display for TrajError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrajError::Io(e) => write!(f, "io: {e}"),
            TrajError::BadMagic { found } => write!(
                f,
                "not a trajectory: magic {:?} (v1 is {:?}, v2 is {:?})",
                String::from_utf8_lossy(found),
                String::from_utf8_lossy(&traj::MAGIC),
                String::from_utf8_lossy(&MAGIC),
            ),
            TrajError::BadVersion { found, expected } => {
                write!(f, "trajectory version {found}, this reader speaks {expected}")
            }
            TrajError::WrongVersion { found, wanted } => write!(
                f,
                "this path speaks trajectory v{wanted} only and the file is v{found}; \
                 refusing rather than reinterpreting it"
            ),
            TrajError::Envelope { n_atoms, max } => write!(
                f,
                "{n_atoms} atoms exceeds the pair-index envelope of {max}: pair indices \
                 are u32 and C({max},2) is the last one that fits"
            ),
            TrajError::NotRepresentableAsV1(why) => {
                write!(f, "cannot be written as v1 without loss: {why}")
            }
            TrajError::Malformed(why) => write!(f, "malformed trajectory: {why}"),
        }
    }
}

impl std::error::Error for TrajError {}

impl From<std::io::Error> for TrajError {
    fn from(e: std::io::Error) -> Self {
        TrajError::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, TrajError>;

/// The intervention ledger's per-frame row — what the engine already knew and the v1
/// artifact dropped.
///
/// `RUNG2_RESULTS.md` G9b wants the momentum change accounted by the wall term and G9c
/// wants the energy draw accounted by the thermostat. Both are subtractions this row makes
/// possible without a `Sim`:
///
/// * momentum: `|P(t) - P(0) - J_ext(t)|`, and `P` comes from the frame's own velocities
///   and the header's `z` (nuclear charge determines mass), so `p0` need not be stored;
/// * energy: `|ledger(t) - l0 - w_ext(t)|`, all four of which are here.
///
/// The three work columns are kept SEPARATE rather than summed, for the reason
/// `ExternalWork` gives in the engine: a run that took energy from the hand and gave the
/// same amount back to the thermostat has a total of zero and two large receipts, and it
/// is not the same run as one nobody touched.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ledger {
    /// Accumulated external impulse since the ledger's origin: `Sim::j_ext`.
    pub j_ext: [f64; 3],
    /// `ExternalWork::hand`.
    pub w_hand: f64,
    /// `ExternalWork::thermostat`.
    pub w_thermostat: f64,
    /// `ExternalWork::barostat`.
    pub w_barostat: f64,
    /// `Sim::ledger()` — the scene's total energy as the ledger sums it.
    pub total: f64,
    /// `Sim::l0` — the ledger's value at its origin. The invariant is
    /// `total - w_ext == l0`, forever.
    pub l0: f64,
}

impl Ledger {
    /// The work columns' sum, in the engine's fixed order so the total is reproducible.
    pub fn w_ext(&self) -> f64 {
        (self.w_hand + self.w_thermostat) + self.w_barostat
    }

    /// The energy ledger's residual: `|total - l0 - w_ext|`. G9c's quantity, computed
    /// from the artifact alone.
    pub fn energy_residual(&self) -> f64 {
        (self.total - self.l0 - self.w_ext()).abs()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Header2 {
    pub seed: u64,
    pub n_atoms: usize,
    /// WHAT THE CALLER ASSERTED, and the one field in this format that must never be
    /// believed. `CENSUS_RESULTS.md` §14.4 is the record of what believing it cost.
    /// [`Trajectory2::measure`] is the answer; this is the claim.
    pub dims_declared: u32,
    pub substeps: u32,
    pub n_frames: usize,
    /// The timestep AT PLACEMENT. v1's warning carries over verbatim and is not softened:
    /// the engine halves its timestep when the scene stiffens, so this describes the setup
    /// and not the run. Nothing that measures a duration may read it; read `Frame2::time`.
    pub dt: f64,
    pub box_w: f64,
    pub box_h: f64,
    pub box_d: f64,
    /// Nuclear charge per ARENA index — keyed by charge rather than by the bank's species
    /// slot, because the species index is an artefact of registration order.
    pub z: Vec<u32>,
    /// Which optional sections every frame in this file carries. Fixed for the file.
    pub content: u32,
}

impl Header2 {
    pub fn n_pairs(&self) -> usize {
        self.n_atoms * self.n_atoms.saturating_sub(1) / 2
    }
    pub fn has_forces(&self) -> bool {
        self.content & CONTENT_FORCES != 0
    }
    pub fn has_ledger(&self) -> bool {
        self.content & CONTENT_LEDGER != 0
    }
}

#[derive(Clone, Debug)]
pub struct Frame2 {
    pub index: u64,
    /// Simulated time in ATOMIC UNITS, exactly as `Sim::time` carries it.
    pub time: f64,
    pub temperature: f64,
    /// The pair indices that read BONDED at this grain boundary, ASCENDING. The engine's
    /// own enumeration via [`crate::traj::pair_index`]; nothing here re-derives the bond
    /// criterion.
    pub bonds: Vec<u32>,
    pub pos: Vec<[f64; 3]>,
    pub vel: Vec<[f64; 3]>,
    /// Present iff the header says so.
    pub forces: Option<Vec<[f64; 3]>>,
    /// Present iff the header says so.
    pub ledger: Option<Ledger>,
}

impl Frame2 {
    #[inline]
    pub fn is_bonded(&self, n: usize, i: usize, j: usize) -> bool {
        let (a, b) = if i < j { (i, j) } else { (j, i) };
        let k = pair_index(n, a, b) as u32;
        self.bonds.binary_search(&k).is_ok()
    }

    /// The v1 bitset this frame's bond set would have been. Refuses above v1's cap rather
    /// than dropping edges — a dropped bond bit shows up as a partition that quietly falls
    /// apart, which is the defect v1's own writer refused to commit.
    pub fn bonded_u128(&self, n_atoms: usize) -> Result<u128> {
        if n_atoms > traj::MAX_DUMP_ATOMS {
            return Err(TrajError::NotRepresentableAsV1(format!(
                "{n_atoms} atoms need {} pair bits and v1's bitset holds 128",
                n_atoms * n_atoms.saturating_sub(1) / 2
            )));
        }
        let mut bits = 0u128;
        for &k in &self.bonds {
            bits |= 1u128 << k;
        }
        Ok(bits)
    }
}

/// What the DATA says about the scene's dimensionality — never what the header says.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measured {
    /// `max - min` per box axis, over every atom of every frame, in bohr. Reported because
    /// `CENSUS_RESULTS.md` §14.4 states its finding in `max |z - z0|` and this campaign
    /// must be able to reproduce that number in its own units.
    pub span: [f64; 3],
    /// The square roots of the position covariance's eigenvalues, DESCENDING. The
    /// covariance route rather than the spans alone, because a scene locked to a plane
    /// that is not axis-aligned has three nonzero spans and rank two.
    pub rms: [f64; 3],
    /// How many principal axes carry extent: the count of `k` with `rms[k]/rms[0] > TOL`.
    pub dims: u32,
    /// `rms[2]/rms[0]`, printed on every read whatever the verdict, so no reader has to
    /// take [`Measured::TOL`]'s word for it.
    pub flatness: f64,
    /// How many (frame, atom) position samples the reading is over.
    pub samples: usize,
}

impl Measured {
    /// The threshold separating a live principal axis from a symmetry-locked one.
    ///
    /// Chosen against the gap it has to resolve rather than as a round number:
    /// `CENSUS_RESULTS.md` §14.4's seventeen planar trajectories hold z bit-exactly, so
    /// their flatness is EXACTLY zero, and `de4_on` reaches 11.49 bohr against a box half
    /// depth of 12.0. The measured quantities sit at 0 and at order 1; anything in between
    /// would be reported by [`Measured::flatness`] and adjudicated by a human, which is
    /// why the ratio is always printed.
    ///
    /// **The margin is two decades, not eight, and that is measured rather than assumed.**
    /// An AXIS-ALIGNED lock reads exactly zero because `z - z0` is exactly zero for every
    /// sample. A TILTED plane does not: its null direction is a cancellation in the
    /// covariance, so the smallest eigenvalue lands near `eps` and its square root — the
    /// flatness — near `sqrt(eps) ~ 1.5e-8`. `a_tilted_plane_has_three_spans_and_rank_two`
    /// measures it at 8.0e-9 on an exactly planar fixture. So this threshold sits about
    /// 100x above the instrument's own floor, and a scene whose genuine out-of-plane
    /// extent is below 1e-6 of its in-plane extent is BEYOND this instrument's reach, not
    /// measured flat by it.
    pub const TOL: f64 = 1e-6;
}

#[derive(Clone, Debug)]
pub struct Trajectory2 {
    pub header: Header2,
    pub frames: Vec<Frame2>,
    /// Which on-disk version this came from: 1 or 2. Carried so a caller can refuse a
    /// lifted v1 file where only a genuine v2 file will do, without re-opening it.
    pub source_version: u32,
    /// The file ended PART WAY THROUGH a frame, and that partial frame was dropped.
    ///
    /// Distinguished from "fewer frames than the header promised" because they are
    /// different facts: a run killed at a grain boundary leaves whole frames, and a run
    /// killed mid-write leaves a fragment. Both are readable prefixes and neither is an
    /// error, but a reader that cannot tell them apart cannot say whether the writer was
    /// interrupted or the disk filled.
    ///
    /// **v1 files cannot report this.** `crate::traj::Trajectory::read` fails on a
    /// mid-frame cut, and it is a banked instrument this module does not edit — every
    /// digest in `census_traj_manifest.sha256` was taken through it. A truncated v1 file
    /// therefore still errors on the lift path, and that asymmetry is recorded rather
    /// than smoothed over.
    pub truncated_frame: bool,
}

// ---------------------------------------------------------------------------- writing

/// Streams v2 frames to disk as they are produced.
///
/// Streaming for v1's reason: the format exists so that a run which dies at frame 14,000
/// leaves 14,000 usable frames behind rather than nothing. The header carries the frame
/// count the writer INTENDED and a reader that finds fewer reports the truncation.
pub struct TrajWriter2 {
    out: BufWriter<File>,
    n_atoms: usize,
    content: u32,
    written: u64,
}

impl TrajWriter2 {
    pub fn create(path: &Path, h: &Header2) -> Result<Self> {
        if h.n_atoms > MAX_ATOMS {
            return Err(TrajError::Envelope {
                n_atoms: h.n_atoms,
                max: MAX_ATOMS,
            });
        }
        if h.content & !CONTENT_KNOWN != 0 {
            return Err(TrajError::Malformed(format!(
                "content bits {:#x} include bits this version does not define",
                h.content
            )));
        }
        if h.z.len() != h.n_atoms {
            return Err(TrajError::Malformed(format!(
                "{} nuclear charges for {} atoms; one per arena index",
                h.z.len(),
                h.n_atoms
            )));
        }
        let mut out = BufWriter::new(File::create(path)?);
        out.write_all(&MAGIC)?;
        out.write_all(&VERSION.to_le_bytes())?;
        out.write_all(&h.content.to_le_bytes())?;
        out.write_all(&(h.n_atoms as u32).to_le_bytes())?;
        out.write_all(&h.dims_declared.to_le_bytes())?;
        out.write_all(&h.substeps.to_le_bytes())?;
        out.write_all(&h.seed.to_le_bytes())?;
        out.write_all(&(h.n_frames as u64).to_le_bytes())?;
        for v in [h.dt, h.box_w, h.box_h, h.box_d] {
            out.write_all(&v.to_le_bytes())?;
        }
        for z in &h.z {
            out.write_all(&z.to_le_bytes())?;
        }
        Ok(Self {
            out,
            n_atoms: h.n_atoms,
            content: h.content,
            written: 0,
        })
    }

    /// Append one frame.
    ///
    /// The optional sections are checked against the HEADER's `content` rather than
    /// against whether the caller happened to pass them: a file whose sections come and go
    /// mid-run has a length no reader can compute, and refusing here is what makes
    /// `content` a fact about the file.
    #[allow(clippy::too_many_arguments)]
    pub fn push(
        &mut self,
        index: u64,
        time: f64,
        temperature: f64,
        bonds: &[u32],
        pos: &[[f64; 3]],
        vel: &[[f64; 3]],
        forces: Option<&[[f64; 3]]>,
        ledger: Option<&Ledger>,
    ) -> Result<()> {
        if pos.len() != self.n_atoms || vel.len() != self.n_atoms {
            return Err(TrajError::Malformed(format!(
                "frame carries {} positions and {} velocities for {} atoms",
                pos.len(),
                vel.len(),
                self.n_atoms
            )));
        }
        if bonds.windows(2).any(|w| w[0] >= w[1]) {
            return Err(TrajError::Malformed(
                "bond indices must be strictly ascending and distinct".into(),
            ));
        }
        let want_f = self.content & CONTENT_FORCES != 0;
        if want_f != forces.is_some() {
            return Err(TrajError::Malformed(format!(
                "header says forces={want_f} and this frame says {}",
                forces.is_some()
            )));
        }
        let want_l = self.content & CONTENT_LEDGER != 0;
        if want_l != ledger.is_some() {
            return Err(TrajError::Malformed(format!(
                "header says ledger={want_l} and this frame says {}",
                ledger.is_some()
            )));
        }
        if let Some(f) = forces {
            if f.len() != self.n_atoms {
                return Err(TrajError::Malformed(format!(
                    "frame carries {} forces for {} atoms",
                    f.len(),
                    self.n_atoms
                )));
            }
        }
        self.out.write_all(&index.to_le_bytes())?;
        self.out.write_all(&time.to_le_bytes())?;
        self.out.write_all(&temperature.to_le_bytes())?;
        self.out.write_all(&(bonds.len() as u32).to_le_bytes())?;
        for k in bonds {
            self.out.write_all(&k.to_le_bytes())?;
        }
        for k in 0..self.n_atoms {
            for c in 0..3 {
                self.out.write_all(&pos[k][c].to_le_bytes())?;
            }
            for c in 0..3 {
                self.out.write_all(&vel[k][c].to_le_bytes())?;
            }
        }
        if let Some(f) = forces {
            for k in 0..self.n_atoms {
                for c in 0..3 {
                    self.out.write_all(&f[k][c].to_le_bytes())?;
                }
            }
        }
        if let Some(l) = ledger {
            for v in [
                l.j_ext[0],
                l.j_ext[1],
                l.j_ext[2],
                l.w_hand,
                l.w_thermostat,
                l.w_barostat,
                l.total,
                l.l0,
            ] {
                self.out.write_all(&v.to_le_bytes())?;
            }
        }
        self.written += 1;
        Ok(())
    }

    pub fn frames_written(&self) -> u64 {
        self.written
    }

    pub fn finish(mut self) -> Result<u64> {
        self.out.flush()?;
        Ok(self.written)
    }
}

// ---------------------------------------------------------------------------- reading

fn rd<const N: usize>(r: &mut impl Read) -> std::io::Result<[u8; N]> {
    let mut b = [0u8; N];
    r.read_exact(&mut b)?;
    Ok(b)
}
fn u32r(r: &mut impl Read) -> std::io::Result<u32> {
    Ok(u32::from_le_bytes(rd::<4>(r)?))
}
fn u64r(r: &mut impl Read) -> std::io::Result<u64> {
    Ok(u64::from_le_bytes(rd::<8>(r)?))
}
fn f64r(r: &mut impl Read) -> std::io::Result<f64> {
    Ok(f64::from_le_bytes(rd::<8>(r)?))
}

/// The on-disk version of `path`, read from its first twelve bytes and nothing else.
///
/// Exposed so a caller can DECIDE rather than discover: a launcher that wants only v2
/// asks first and refuses with [`TrajError::WrongVersion`], which is a different exit
/// status from "this is not a trajectory at all".
pub fn peek_version(path: &Path) -> Result<u32> {
    let mut r = BufReader::new(File::open(path)?);
    // A FILE TOO SHORT TO HAVE A VERSION IS A FORMAT REFUSAL, NOT A PATH FAILURE.
    //
    // The raw `?` here mapped `UnexpectedEof` onto `TrajError::Io` and therefore onto exit
    // code 3, which the freeze defines as "a path did not resolve" — so a caller handed a
    // truncated or foreign file would be told to check its paths. That is the exact
    // failure `M-EXIT-DISCRIMINATOR` names, in the code whose job is to discriminate.
    let head = |e: std::io::Error| -> TrajError {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            TrajError::Malformed(format!(
                "{size} bytes is too short to be a trajectory: the magic and version alone \
                 need 12"
            ))
        } else {
            TrajError::Io(e)
        }
    };
    let magic: [u8; 8] = rd::<8>(&mut r).map_err(head)?;
    let version = u32r(&mut r).map_err(head)?;
    if magic == traj::MAGIC {
        if version != traj::VERSION {
            return Err(TrajError::BadVersion {
                found: version,
                expected: traj::VERSION,
            });
        }
        return Ok(traj::VERSION);
    }
    if magic == MAGIC {
        if version != VERSION {
            return Err(TrajError::BadVersion {
                found: version,
                expected: VERSION,
            });
        }
        return Ok(VERSION);
    }
    Err(TrajError::BadMagic { found: magic })
}

impl Trajectory2 {
    /// Read a trajectory of EITHER version, lifting v1 into the v2 shape.
    ///
    /// The lift adds nothing: `content` is 0, so a lifted v1 file has no forces and no
    /// ledger and says so, rather than carrying zeros that would read as "measured and
    /// found zero". `source_version` records where it came from.
    pub fn read(path: &Path) -> Result<Self> {
        match peek_version(path)? {
            v if v == traj::VERSION => {
                let t = Trajectory::read(path)?;
                Self::from_v1(&t)
            }
            _ => Self::read_v2(path),
        }
    }

    /// Read a v2 file, refusing a v1 one by name rather than reinterpreting it.
    pub fn read_strict(path: &Path) -> Result<Self> {
        let v = peek_version(path)?;
        if v != VERSION {
            return Err(TrajError::WrongVersion {
                found: v,
                wanted: VERSION,
            });
        }
        Self::read_v2(path)
    }

    fn read_v2(path: &Path) -> Result<Self> {
        let mut r = BufReader::new(File::open(path)?);
        let magic: [u8; 8] = rd::<8>(&mut r)?;
        if magic != MAGIC {
            return Err(TrajError::BadMagic { found: magic });
        }
        let version = u32r(&mut r)?;
        if version != VERSION {
            return Err(TrajError::BadVersion {
                found: version,
                expected: VERSION,
            });
        }
        let content = u32r(&mut r)?;
        if content & !CONTENT_KNOWN != 0 {
            return Err(TrajError::Malformed(format!(
                "content bits {content:#x} include bits this reader does not define; \
                 refusing rather than skipping a section whose length is unknown"
            )));
        }
        let n_atoms = u32r(&mut r)? as usize;
        if n_atoms > MAX_ATOMS {
            return Err(TrajError::Envelope {
                n_atoms,
                max: MAX_ATOMS,
            });
        }
        let dims_declared = u32r(&mut r)?;
        let substeps = u32r(&mut r)?;
        let seed = u64r(&mut r)?;
        let n_frames = u64r(&mut r)? as usize;
        let dt = f64r(&mut r)?;
        let box_w = f64r(&mut r)?;
        let box_h = f64r(&mut r)?;
        let box_d = f64r(&mut r)?;
        let mut z = Vec::with_capacity(n_atoms);
        for _ in 0..n_atoms {
            z.push(u32r(&mut r)?);
        }
        let header = Header2 {
            seed,
            n_atoms,
            dims_declared,
            substeps,
            n_frames,
            dt,
            box_w,
            box_h,
            box_d,
            z,
            content,
        };
        let n_pairs = header.n_pairs();
        // Read to EOF rather than to `n_frames`: a run killed mid-window leaves a short
        // file, and the honest reading of a short file is "this many frames", not a parse
        // error and not a silent pad.
        let mut frames = Vec::new();
        let mut truncated_frame = false;
        loop {
            let index = match u64r(&mut r) {
                Ok(v) => v,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            };
            // The body is read through one fallible closure so that an EOF ANYWHERE
            // inside a frame drops that fragment and stops, rather than propagating as a
            // parse error. A writer killed mid-frame leaves a fragment; the honest
            // reading of a fragment is "the frames before it", and this loop's first
            // version got that right only for a cut that happened to land on a frame
            // boundary. The plant that found it is `truncation_and_the_degenerate_scenes`.
            let body = (|| -> std::io::Result<(f64, f64, usize, Vec<u32>)> {
                let time = f64r(&mut r)?;
                let temperature = f64r(&mut r)?;
                let n_bonds = u32r(&mut r)? as usize;
                Ok((time, temperature, n_bonds, Vec::new()))
            })();
            let (time, temperature, n_bonds, mut bonds) = match body {
                Ok(v) => v,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    truncated_frame = true;
                    break;
                }
                Err(e) => return Err(e.into()),
            };
            if n_bonds > n_pairs {
                return Err(TrajError::Malformed(format!(
                    "frame {index} claims {n_bonds} bonds and the scene has {n_pairs} pairs"
                )));
            }
            let rest = (|| -> std::io::Result<(Vec<[f64; 3]>, Vec<[f64; 3]>, Option<Vec<[f64; 3]>>, Option<Ledger>)> {
                bonds.reserve(n_bonds);
                for _ in 0..n_bonds {
                    bonds.push(u32r(&mut r)?);
                }
                let mut pos = Vec::with_capacity(n_atoms);
                let mut vel = Vec::with_capacity(n_atoms);
                for _ in 0..n_atoms {
                    pos.push([f64r(&mut r)?, f64r(&mut r)?, f64r(&mut r)?]);
                    vel.push([f64r(&mut r)?, f64r(&mut r)?, f64r(&mut r)?]);
                }
                let forces = if header.has_forces() {
                    let mut f = Vec::with_capacity(n_atoms);
                    for _ in 0..n_atoms {
                        f.push([f64r(&mut r)?, f64r(&mut r)?, f64r(&mut r)?]);
                    }
                    Some(f)
                } else {
                    None
                };
                let ledger = if header.has_ledger() {
                    Some(Ledger {
                        j_ext: [f64r(&mut r)?, f64r(&mut r)?, f64r(&mut r)?],
                        w_hand: f64r(&mut r)?,
                        w_thermostat: f64r(&mut r)?,
                        w_barostat: f64r(&mut r)?,
                        total: f64r(&mut r)?,
                        l0: f64r(&mut r)?,
                    })
                } else {
                    None
                };
                Ok((pos, vel, forces, ledger))
            })();
            let (pos, vel, forces, ledger) = match rest {
                Ok(v) => v,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    truncated_frame = true;
                    break;
                }
                Err(e) => return Err(e.into()),
            };
            if bonds.windows(2).any(|w| w[0] >= w[1]) {
                return Err(TrajError::Malformed(format!(
                    "frame {index}'s bond indices are not strictly ascending"
                )));
            }
            frames.push(Frame2 {
                index,
                time,
                temperature,
                bonds,
                pos,
                vel,
                forces,
                ledger,
            });
        }
        Ok(Self {
            header,
            frames,
            source_version: VERSION,
            truncated_frame,
        })
    }

    /// Lift a v1 trajectory into the v2 shape, losing nothing and inventing nothing.
    ///
    /// FALLIBLE, and the reason is a hole this campaign's own plant fixture fell into. v1
    /// stores 128 bond bits whatever the scene's size, so a twelve-atom file has 66 real
    /// pair bits and 62 that cannot be pair indices at all. The first version of this lift
    /// filtered the bit set to `0..n_pairs` — which reads a self-contradicting file
    /// SILENTLY, drops the impossible bits, and produces a v2 value whose re-serialisation
    /// is a different file. That is precisely the silent reinterpretation G1 exists to
    /// catch, sitting inside the code G1 runs through.
    ///
    /// A bit at or above `n_pairs` is therefore REFUSED and the offending bit is named.
    /// Every banked trajectory is written by `bond_bits`, which only ever sets indices
    /// below `n_pairs`, so no artifact of record is affected — but "no artifact we have
    /// hits it" is not the same fact as "the reader cannot be fooled by it".
    pub fn from_v1(t: &Trajectory) -> Result<Self> {
        let n = t.header.n_atoms;
        let header = Header2 {
            seed: t.header.seed,
            n_atoms: n,
            dims_declared: t.header.dims,
            substeps: t.header.substeps,
            n_frames: t.header.n_frames,
            dt: t.header.dt,
            box_w: t.header.box_w,
            box_h: t.header.box_h,
            box_d: t.header.box_d,
            z: t.header.z.clone(),
            content: 0,
        };
        let n_pairs = header.n_pairs();
        let mut frames = Vec::with_capacity(t.frames.len());
        for f in &t.frames {
            // `n_pairs >= 128` means every bit of the word is a legal pair index and the
            // shift below would be undefined; a v1 file can DECLARE such a scene even
            // though v1's writer refuses to produce one, so the guard is a real branch.
            if let Some(bit) = f.bonds.iter().find(|&k| k as usize >= n_pairs) {
                return Err(TrajError::Malformed(format!(
                    "frame {} has bond bit {bit} set and a {n}-atom scene has only \
                     {n_pairs} pairs; refusing rather than dropping it",
                    f.index
                )));
            }
            frames.push(Frame2 {
                index: f.index,
                time: f.time,
                temperature: f.temperature,
                bonds: f.bonds.pairs().to_vec(),
                pos: f.pos.clone(),
                vel: f.vel.clone(),
                forces: None,
                ledger: None,
            });
        }
        Ok(Self {
            header,
            frames,
            source_version: traj::VERSION,
            truncated_frame: false,
        })
    }

    /// Re-serialise as a v1 file, byte for byte.
    ///
    /// THE BIT-IDENTITY PLANT'S OTHER HALF. `CARRIER_V2_PREREG.md` G1 requires that every
    /// banked v1 trajectory, read through this module and written back through here,
    /// reproduces its manifest digest. Anything that cannot be represented in v1 is
    /// REFUSED with what would have been lost, never dropped: a writer that silently
    /// discarded the ledger would make the digest check pass on a file that is no longer
    /// the same artifact.
    pub fn write_as_v1(&self, path: &Path) -> Result<u64> {
        if self.header.content != 0 {
            return Err(TrajError::NotRepresentableAsV1(format!(
                "content {:#x} carries sections v1 has no place for",
                self.header.content
            )));
        }
        if self.header.n_atoms > traj::MAX_DUMP_ATOMS {
            return Err(TrajError::NotRepresentableAsV1(format!(
                "{} atoms; v1's pair bitset holds C({},2) = 120 bits",
                self.header.n_atoms,
                traj::MAX_DUMP_ATOMS
            )));
        }
        let h = Header {
            seed: self.header.seed,
            n_atoms: self.header.n_atoms,
            dims: self.header.dims_declared,
            substeps: self.header.substeps,
            n_frames: self.header.n_frames,
            dt: self.header.dt,
            box_w: self.header.box_w,
            box_h: self.header.box_h,
            box_d: self.header.box_d,
            z: self.header.z.clone(),
        };
        let mut w = TrajWriter::create(path, &h)?;
        for f in &self.frames {
            // Refused by name past v1's cap, before the writer's own `to_bits` could.
            let bits = f.bonded_u128(self.header.n_atoms)?;
            let bonds = BondSet::from_bits(bits);
            w.push(f.index, f.time, f.temperature, &bonds, &f.pos, &f.vel)?;
        }
        Ok(w.finish()?)
    }

    /// Whether the file holds every frame its header promised. Reported, never asserted.
    pub fn is_complete(&self) -> bool {
        self.frames.len() == self.header.n_frames
    }

    /// G2's COMPARATOR: every disagreement with what the v1 reader made of the same file,
    /// each named by the field it is in.
    ///
    /// One implementation, used by the bank instrument and by the plants that convict it.
    /// A comparator the plants exercise and the campaign does not is a different
    /// comparator, and the campaign's green result would say nothing about it.
    ///
    /// Float comparison is on BITS, not on a tolerance. This is a re-read of the same
    /// bytes, so any difference at all is a difference in interpretation, and a tolerance
    /// here would be a place for one to hide.
    pub fn diff_against_v1(&self, one: &Trajectory) -> Vec<String> {
        let mut out = Vec::new();
        macro_rules! cmp {
            ($name:literal, $a:expr, $b:expr) => {
                if $a != $b {
                    out.push(format!("{}: v2 reads {:?}, v1 reads {:?}", $name, $a, $b));
                }
            };
        }
        macro_rules! cmpf {
            ($name:literal, $a:expr, $b:expr) => {
                if $a.to_bits() != $b.to_bits() {
                    out.push(format!("{}: v2 reads {:?}, v1 reads {:?}", $name, $a, $b));
                }
            };
        }
        cmp!("seed", self.header.seed, one.header.seed);
        cmp!("n_atoms", self.header.n_atoms, one.header.n_atoms);
        cmp!("dims", self.header.dims_declared, one.header.dims);
        cmp!("substeps", self.header.substeps, one.header.substeps);
        cmp!("n_frames", self.header.n_frames, one.header.n_frames);
        cmpf!("dt", self.header.dt, one.header.dt);
        cmpf!("box_w", self.header.box_w, one.header.box_w);
        cmpf!("box_h", self.header.box_h, one.header.box_h);
        cmpf!("box_d", self.header.box_d, one.header.box_d);
        cmp!("z", self.header.z, one.header.z);
        cmp!("frame count", self.frames.len(), one.frames.len());
        if !out.is_empty() {
            // A header disagreement makes every frame comparison meaningless, and a
            // hundred derived complaints would bury the one that matters.
            return out;
        }
        let n = self.header.n_atoms;
        for (f2, f1) in self.frames.iter().zip(one.frames.iter()) {
            let at = f1.index;
            cmp!("frame index", f2.index, f1.index);
            cmpf!("time", f2.time, f1.time);
            cmpf!("temperature", f2.temperature, f1.temperature);
            match f2.bonded_u128(n) {
                Ok(bits) if BondSet::from_bits(bits) != f1.bonds => out.push(format!(
                    "frame {at} bond set: v2 reads {bits:#x}, v1 reads {:#x}",
                    f1.bonds.to_bits().unwrap_or(u128::MAX)
                )),
                Err(e) => out.push(format!("frame {at} bond set: {e}")),
                _ => {}
            }
            for i in 0..n {
                for c in 0..3 {
                    if f2.pos[i][c].to_bits() != f1.pos[i][c].to_bits() {
                        out.push(format!("frame {at} atom {i} pos[{c}]"));
                    }
                    if f2.vel[i][c].to_bits() != f1.vel[i][c].to_bits() {
                        out.push(format!("frame {at} atom {i} vel[{c}]"));
                    }
                }
            }
            if out.len() > 32 {
                out.push("... further disagreements not listed".into());
                break;
            }
        }
        out
    }

    /// THE MEASUREMENT the header's `dims_declared` may not substitute for.
    ///
    /// Two passes and a deliberate origin shift. The covariance is accumulated about the
    /// FIRST sample's position rather than about the running mean, and that is not a
    /// performance choice: if every `z` is bit-identical, `z - z0` is exactly zero for
    /// every sample, so the covariance's third row and column are exactly zero and the
    /// third eigenvalue is exactly zero. Accumulating about a computed mean would leave
    /// rounding in that row and report a symmetry-locked scene as faintly three
    /// dimensional. `CENSUS_RESULTS.md` §14.4's seventeen trajectories are the fixture
    /// this has to read as EXACTLY planar.
    pub fn measure(&self) -> Measured {
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        let mut samples = 0usize;
        let origin = self
            .frames
            .first()
            .and_then(|f| f.pos.first())
            .copied()
            .unwrap_or([0.0; 3]);
        // Second moments about `origin`, and the mean about `origin`.
        let mut m = [[0.0f64; 3]; 3];
        let mut mu = [0.0f64; 3];
        for f in &self.frames {
            for p in &f.pos {
                let d = [p[0] - origin[0], p[1] - origin[1], p[2] - origin[2]];
                for k in 0..3 {
                    if p[k] < lo[k] {
                        lo[k] = p[k];
                    }
                    if p[k] > hi[k] {
                        hi[k] = p[k];
                    }
                    mu[k] += d[k];
                    for l in 0..3 {
                        m[k][l] += d[k] * d[l];
                    }
                }
                samples += 1;
            }
        }
        if samples == 0 {
            return Measured {
                span: [0.0; 3],
                rms: [0.0; 3],
                dims: 0,
                flatness: 0.0,
                samples: 0,
            };
        }
        let n = samples as f64;
        let mut cov = [[0.0f64; 3]; 3];
        for k in 0..3 {
            mu[k] /= n;
        }
        for k in 0..3 {
            for l in 0..3 {
                cov[k][l] = m[k][l] / n - mu[k] * mu[l];
            }
        }
        let eig = jacobi_eigenvalues_3x3(cov);
        let rms = [
            eig[0].max(0.0).sqrt(),
            eig[1].max(0.0).sqrt(),
            eig[2].max(0.0).sqrt(),
        ];
        let flatness = if rms[0] > 0.0 { rms[2] / rms[0] } else { 0.0 };
        let dims = if rms[0] <= 0.0 {
            0
        } else {
            rms.iter().filter(|r| **r / rms[0] > Measured::TOL).count() as u32
        };
        Measured {
            span: [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]],
            rms,
            dims,
            flatness,
            samples,
        }
    }

    /// G9b's QUANTITY, computed from the artifact alone: `|P(t) - P(0) - J_ext(t)|`.
    ///
    /// `RUNG2_RESULTS.md` §5.3 marked this UNDISCHARGED — "not computable from this
    /// artifact" — because v1 carried no intervention ledger. It is computable now, and
    /// this function is deliberately the whole of what that took: the velocities were
    /// always there, `j_ext` is the new part, and `P(0)` is this trajectory's own first
    /// frame.
    ///
    /// **The caller supplies the mass law.** `holon-lens` has no element table and will
    /// not grow one: a second statement of "what does nucleus Z weigh" is how the two of
    /// them come to disagree, and the engine's is in `holon-chem`. Pass
    /// `|z| sim_species_mass(z)` from a crate that has it. This is `OBJECT.md`'s generator
    /// discipline one tier down — the caller supplies the physics and the layer refuses to
    /// invent it.
    ///
    /// `None` when the file carries no ledger, which is the honest answer for every v1
    /// file and is not the same answer as zero.
    pub fn momentum_residual(&self, frame: usize, mass_of: impl Fn(u32) -> f64) -> Option<f64> {
        let f = self.frames.get(frame)?;
        let l = f.ledger?;
        let first = self.frames.first()?;
        let mut d = [0.0f64; 3];
        for k in 0..3 {
            let mut p = 0.0;
            let mut p0 = 0.0;
            for (i, z) in self.header.z.iter().enumerate() {
                let m = mass_of(*z);
                p += m * f.vel[i][k];
                p0 += m * first.vel[i][k];
            }
            d[k] = p - p0 - l.j_ext[k];
        }
        Some((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
    }

    /// PER-CELL FRACTIONAL MEAN OCCUPANCY over a grid the CALLER declares.
    ///
    /// `RUNG2_PREREG_A2`'s fluid-element representation, derived rather than stored, and
    /// the choice is deliberate. Occupancy is a function of the positions and A GRID, and
    /// a grid is a reading, not a fact about the run. Storing it in the artifact would
    /// (i) bake one grid into a file every later campaign has to live with — the same
    /// trusted-declaration failure `dims_declared` exists to close — and (ii) create a
    /// second implementation of a reading that already has one. So the format carries the
    /// positions and this is the only implementation.
    ///
    /// Returns the grid in `x + nx*(y + ny*z)` order, and — separately, never folded in —
    /// how many (frame, atom) samples fell OUTSIDE the declared box. The walls in this
    /// engine are soft, so atoms genuinely leave; clamping them into the edge cells would
    /// manufacture density exactly where a fluid-element reading is most fragile, and
    /// dropping them silently would make the grid's total disagree with `N`.
    pub fn mean_cell_occupancy(&self, n: [usize; 3]) -> Option<(Vec<f64>, usize)> {
        if n.iter().any(|c| *c == 0) || self.frames.is_empty() {
            return None;
        }
        let extent = [self.header.box_w, self.header.box_h, self.header.box_d];
        if extent.iter().any(|e| !(*e > 0.0) || !e.is_finite()) {
            return None;
        }
        let cells = n[0] * n[1] * n[2];
        let mut count = vec![0u64; cells];
        let mut outside = 0usize;
        for f in &self.frames {
            for p in &f.pos {
                let mut idx = [0usize; 3];
                let mut out = false;
                for k in 0..3 {
                    let t = p[k] / extent[k];
                    // The upper face is INSIDE, matching `cells.rs`, which adds "a hair of
                    // margin so an atom exactly on the upper face lands in the last cell
                    // rather than one past it". Two conventions for which cell owns a face
                    // is how the fluid chart and the neighbour list come to disagree about
                    // where an atom is, and only one of them would be reported.
                    if !(0.0..=1.0).contains(&t) {
                        out = true;
                        break;
                    }
                    idx[k] = ((t * n[k] as f64) as usize).min(n[k] - 1);
                }
                if out {
                    outside += 1;
                    continue;
                }
                count[idx[0] + n[0] * (idx[1] + n[1] * idx[2])] += 1;
            }
        }
        let frames = self.frames.len() as f64;
        Some((count.iter().map(|c| *c as f64 / frames).collect(), outside))
    }

    /// Whether the header's claim and the data's answer agree. Reported by every reader;
    /// NEVER repaired in place, because correcting the header would put the lie one layer
    /// down where the next campaign would inherit it.
    pub fn dims_agree(&self) -> bool {
        self.measure().dims == self.header.dims_declared
    }
}

/// The eigenvalues of a symmetric 3x3, DESCENDING, by cyclic Jacobi.
///
/// A fixed sweep budget and no external solver: the reading has to be identical on every
/// machine that computes it, and a library eigensolver is a device-class dependence
/// (`M-DEVICE-CLASS`) hiding inside a number that looks like geometry. Jacobi is chosen
/// over the closed-form trigonometric root because the closed form loses its accuracy
/// exactly where this campaign lives — a nearly degenerate spectrum with one eigenvalue at
/// or near zero.
fn jacobi_eigenvalues_3x3(mut a: [[f64; 3]; 3]) -> [f64; 3] {
    for _ in 0..64 {
        // The largest off-diagonal magnitude, and its position.
        let mut p = 0usize;
        let mut q = 1usize;
        let mut best = a[0][1].abs();
        if a[0][2].abs() > best {
            best = a[0][2].abs();
            p = 0;
            q = 2;
        }
        if a[1][2].abs() > best {
            best = a[1][2].abs();
            p = 1;
            q = 2;
        }
        // Scaled against the diagonal so the exit is a relative one; an absolute epsilon
        // would loop forever on a large matrix and quit early on a tiny one.
        let scale = a[0][0].abs().max(a[1][1].abs()).max(a[2][2].abs());
        if best <= scale * f64::EPSILON || best == 0.0 {
            break;
        }
        let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
        let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
        let c = 1.0 / (t * t + 1.0).sqrt();
        let s = t * c;
        let mut b = a;
        for k in 0..3 {
            b[k][p] = c * a[k][p] - s * a[k][q];
            b[k][q] = s * a[k][p] + c * a[k][q];
        }
        let mut d = b;
        for k in 0..3 {
            d[p][k] = c * b[p][k] - s * b[q][k];
            d[q][k] = s * b[p][k] + c * b[q][k];
        }
        d[p][q] = 0.0;
        d[q][p] = 0.0;
        a = d;
    }
    let mut e = [a[0][0], a[1][1], a[2][2]];
    e.sort_by(|x, y| y.partial_cmp(x).unwrap_or(std::cmp::Ordering::Equal));
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("hlens2-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn v1_fixture(path: &Path, n_atoms: usize, n_frames: usize) -> Header {
        let h = Header {
            seed: 0x5341_5422,
            n_atoms,
            dims: 2,
            substeps: 64,
            n_frames,
            dt: 0.5386,
            box_w: 34.6,
            box_h: 20.8,
            box_d: 24.0,
            z: (0..n_atoms).map(|i| if i < 4 { 8 } else { 1 }).collect(),
        };
        let mut w = TrajWriter::create(path, &h).unwrap();
        for f in 0..n_frames {
            let pos: Vec<[f64; 3]> = (0..n_atoms)
                .map(|i| [i as f64 * 1.7 + f as f64 * 0.01, i as f64 * 0.3, 12.0])
                .collect();
            let vel: Vec<[f64; 3]> = (0..n_atoms)
                .map(|i| [0.001 * i as f64, -0.002, 0.0])
                .collect();
            let bits = (1u128 << (f % 7)) | 0b1011;
            w.push(f as u64, f as f64 * 34.4, 300.0 - f as f64, &BondSet::from_bits(bits), &pos, &vel)
                .unwrap();
        }
        w.finish().unwrap();
        h
    }

    /// G1's shape on a synthetic carrier: read a v1 file through v2 and write it back,
    /// and the BYTES must be the same bytes. The campaign runs this on the real bank; this
    /// is the version that runs in CI without one.
    #[test]
    fn v1_round_trips_through_v2_byte_for_byte() {
        let d = tmp("bit");
        let a = d.join("a.traj");
        let b = d.join("b.traj");
        v1_fixture(&a, 12, 9);
        let t = Trajectory2::read(&a).unwrap();
        assert_eq!(t.source_version, 1);
        assert_eq!(t.header.content, 0, "a lifted v1 file invents no sections");
        t.write_as_v1(&b).unwrap();
        assert_eq!(
            std::fs::read(&a).unwrap(),
            std::fs::read(&b).unwrap(),
            "the v2 reader reinterpreted a v1 file"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    /// A v1 file whose bitset carries a bit no pair of its scene could own is REFUSED,
    /// never quietly filtered.
    ///
    /// This test exists because the first version of `from_v1` did quietly filter, and a
    /// plant fixture that set bit 119 on a twelve-atom scene (66 pairs) round-tripped to a
    /// DIFFERENT file while every gate stayed green. The bit is named in the refusal so a
    /// caller can see whether it hit a corrupt file or a foreign one.
    #[test]
    fn a_bond_bit_no_pair_could_own_is_refused_by_name() {
        let d = tmp("stray");
        let path = d.join("s.traj");
        let h = Header {
            seed: 1,
            n_atoms: 12,
            dims: 2,
            substeps: 8,
            n_frames: 2,
            dt: 1.0,
            box_w: 10.0,
            box_h: 10.0,
            box_d: 10.0,
            z: vec![1; 12],
        };
        let mut w = TrajWriter::create(&path, &h).unwrap();
        let p = vec![[0.0; 3]; 12];
        // 12 atoms have C(12,2) = 66 pairs, so 65 is the last legal index.
        assert_eq!(pair_index(12, 10, 11), 65);
        w.push(0, 0.0, 300.0, &BondSet::from_bits(1u128 << 65), &p, &p).unwrap();
        w.push(1, 1.0, 300.0, &BondSet::from_bits(1u128 << 119), &p, &p).unwrap();
        w.finish().unwrap();
        let e = match Trajectory2::read(&path) {
            Ok(_) => panic!("a bit 62 places past the last pair was accepted"),
            Err(e) => e,
        };
        let msg = e.to_string();
        assert!(msg.contains("bond bit 119"), "{msg}");
        assert!(msg.contains("frame 1"), "{msg}");
        assert!(msg.contains("66 pairs"), "{msg}");
        // And the legal top bit on its own is fine.
        let ok = d.join("ok.traj");
        let mut w = TrajWriter::create(&ok, &h).unwrap();
        w.push(0, 0.0, 300.0, &BondSet::from_bits(1u128 << 65), &p, &p).unwrap();
        w.finish().unwrap();
        let t = Trajectory2::read(&ok).unwrap();
        assert_eq!(t.frames[0].bonds, vec![65]);
        assert!(t.frames[0].is_bonded(12, 10, 11));
        let _ = std::fs::remove_dir_all(&d);
    }

    /// G2: every field the v1 reader produces, the v2 reader produces. Bit equality on
    /// floats, set equality on bonds.
    #[test]
    fn v1_fields_are_identical_through_both_readers() {
        let d = tmp("field");
        let a = d.join("a.traj");
        v1_fixture(&a, 12, 9);
        let one = Trajectory::read(&a).unwrap();
        let two = Trajectory2::read(&a).unwrap();
        assert_eq!(two.header.seed, one.header.seed);
        assert_eq!(two.header.n_atoms, one.header.n_atoms);
        assert_eq!(two.header.dims_declared, one.header.dims);
        assert_eq!(two.header.substeps, one.header.substeps);
        assert_eq!(two.header.n_frames, one.header.n_frames);
        assert_eq!(two.header.dt.to_bits(), one.header.dt.to_bits());
        assert_eq!(two.header.box_w.to_bits(), one.header.box_w.to_bits());
        assert_eq!(two.header.box_h.to_bits(), one.header.box_h.to_bits());
        assert_eq!(two.header.box_d.to_bits(), one.header.box_d.to_bits());
        assert_eq!(two.header.z, one.header.z);
        assert_eq!(two.frames.len(), one.frames.len());
        for (f2, f1) in two.frames.iter().zip(one.frames.iter()) {
            assert_eq!(f2.index, f1.index);
            assert_eq!(f2.time.to_bits(), f1.time.to_bits());
            assert_eq!(f2.temperature.to_bits(), f1.temperature.to_bits());
            assert_eq!(BondSet::from_bits(f2.bonded_u128(12).unwrap()), f1.bonds);
            for i in 0..12 {
                for c in 0..3 {
                    assert_eq!(f2.pos[i][c].to_bits(), f1.pos[i][c].to_bits());
                    assert_eq!(f2.vel[i][c].to_bits(), f1.vel[i][c].to_bits());
                }
            }
            // And the accessor agrees with v1's, pair by pair.
            for i in 0..12 {
                for j in (i + 1)..12 {
                    assert_eq!(f2.is_bonded(12, i, j), f1.is_bonded(12, i, j));
                    assert_eq!(f2.is_bonded(12, j, i), f1.is_bonded(12, i, j));
                }
            }
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    /// G3, both directions. Neither reader may guess.
    #[test]
    fn the_two_versions_refuse_each_other_by_name() {
        let d = tmp("ver");
        let one = d.join("one.traj");
        let two = d.join("two.traj");
        v1_fixture(&one, 4, 3);
        let h = Header2 {
            seed: 1,
            n_atoms: 4,
            dims_declared: 3,
            substeps: 8,
            n_frames: 1,
            dt: 1.0,
            box_w: 10.0,
            box_h: 10.0,
            box_d: 10.0,
            z: vec![1, 1, 1, 1],
            content: 0,
        };
        let mut w = TrajWriter2::create(&two, &h).unwrap();
        let p = vec![[0.0; 3]; 4];
        w.push(0, 0.0, 300.0, &[0, 2], &p, &p, None, None).unwrap();
        w.finish().unwrap();

        // A v2 file into the v1 reader: refused, and the message names the magic.
        let e = Trajectory::read(&two).unwrap_err();
        assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
        assert!(e.to_string().contains("HLNTRAJ1"), "{e}");
        // A v1 file into the strict v2 path: refused, naming the version FOUND.
        let e = Trajectory2::read_strict(&one).unwrap_err();
        assert!(matches!(
            e,
            TrajError::WrongVersion { found: 1, wanted: 2 }
        ));
        assert_eq!(e.exit_code(), 4);
        // And the tolerant path reads both.
        assert_eq!(Trajectory2::read(&one).unwrap().source_version, 1);
        assert_eq!(Trajectory2::read(&two).unwrap().source_version, 2);
        let _ = std::fs::remove_dir_all(&d);
    }

    /// G4: the cap is gone, and the envelope is exactly where the header says.
    #[test]
    fn no_atom_cap_below_the_stated_envelope() {
        let d = tmp("big");
        let path = d.join("big.traj");
        let n = 402usize;
        let h = Header2 {
            seed: 7,
            n_atoms: n,
            dims_declared: 3,
            substeps: 16,
            n_frames: 2,
            dt: 0.5,
            box_w: 60.0,
            box_h: 60.0,
            box_d: 60.0,
            z: (0..n).map(|i| if i % 3 == 0 { 8 } else { 1 }).collect(),
            content: 0,
        };
        let mut w = TrajWriter2::create(&path, &h).unwrap();
        // A bond set that reaches the LAST pair index, which is the one a bitset drops.
        let last = pair_index(n, n - 2, n - 1) as u32;
        assert_eq!(last, (n * (n - 1) / 2 - 1) as u32);
        let bonds: Vec<u32> = vec![0, 1, 119, 120, last - 1, last];
        let pos: Vec<[f64; 3]> = (0..n)
            .map(|i| [i as f64 * 0.13, (i % 7) as f64, (i % 11) as f64])
            .collect();
        for f in 0..2 {
            w.push(f, f as f64, 300.0, &bonds, &pos, &pos, None, None)
                .unwrap();
        }
        w.finish().unwrap();
        let t = Trajectory2::read_strict(&path).unwrap();
        assert_eq!(t.header.n_atoms, n);
        assert_eq!(t.frames[0].bonds, bonds);
        assert!(t.frames[0].is_bonded(n, n - 2, n - 1), "the last pair reads");
        assert!(t.frames[0].is_bonded(n, 0, 1));
        assert!(!t.frames[0].is_bonded(n, 0, 3));
        // It is NOT representable as v1, and the refusal says why rather than truncating.
        let e = t.write_as_v1(&d.join("nope.traj")).unwrap_err();
        assert!(matches!(e, TrajError::NotRepresentableAsV1(_)), "{e}");

        // And the envelope refuses where §1.4 says it does.
        let too_big = Header2 {
            n_atoms: MAX_ATOMS + 1,
            z: Vec::new(),
            ..h.clone()
        };
        let e = match TrajWriter2::create(&d.join("huge.traj"), &too_big) {
            Ok(_) => panic!("the envelope did not refuse"),
            Err(e) => e,
        };
        assert!(matches!(e, TrajError::Envelope { .. }), "{e}");
        assert_eq!(e.exit_code(), 7);
        // The envelope is the LAST N whose pair count fits a u32, not one before it.
        assert!((MAX_ATOMS * (MAX_ATOMS - 1) / 2) < u32::MAX as usize);
        assert!(((MAX_ATOMS + 1) * MAX_ATOMS / 2) > u32::MAX as usize);
        let _ = std::fs::remove_dir_all(&d);
    }

    /// P-5 and P-4: a truly planar scene reads EXACTLY zero, and a staked span reads back
    /// to the bit.
    #[test]
    fn planarity_is_measured_exactly_and_a_staked_span_reads_back() {
        let d = tmp("dims");
        let n = 8usize;
        let mk = |path: &Path, zs: &dyn Fn(usize) -> f64, declared: u32| {
            let h = Header2 {
                seed: 3,
                n_atoms: n,
                dims_declared: declared,
                substeps: 8,
                n_frames: 5,
                dt: 1.0,
                box_w: 20.0,
                box_h: 20.0,
                box_d: 20.0,
                z: vec![1; n],
                content: 0,
            };
            let mut w = TrajWriter2::create(path, &h).unwrap();
            for f in 0..5 {
                let pos: Vec<[f64; 3]> = (0..n)
                    .map(|i| [i as f64 + f as f64 * 0.5, (i * i % 5) as f64, zs(i)])
                    .collect();
                w.push(f as u64, f as f64, 300.0, &[], &pos, &pos, None, None)
                    .unwrap();
            }
            w.finish().unwrap();
        };

        // P-5: every z identical. The plane is exact and the reading must be too.
        let flat = d.join("flat.traj");
        mk(&flat, &|_| 10.0, 2);
        let m = Trajectory2::read(&flat).unwrap().measure();
        assert_eq!(m.span[2], 0.0, "a locked plane has EXACTLY zero z span");
        assert_eq!(m.flatness, 0.0, "and exactly zero flatness");
        assert_eq!(m.dims, 2);
        assert_eq!(m.samples, 40);
        assert!(Trajectory2::read(&flat).unwrap().dims_agree());

        // P-4: a staked span of exactly 7.5 bohr.
        let deep = d.join("deep.traj");
        mk(&deep, &|i| if i == 0 { 2.5 } else { 10.0 }, 2);
        let t = Trajectory2::read(&deep).unwrap();
        let m = t.measure();
        assert_eq!(m.span[2], 7.5, "the staked span reads back exactly");
        assert_eq!(m.dims, 3);
        // P-6: the declaration lies, and the reader reports the DISAGREEMENT.
        assert!(!t.dims_agree(), "declared 2, measured 3, and it must say so");
        assert_eq!(t.header.dims_declared, 2, "and it does not repair the header");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// The plane need not be axis-aligned for the rank reading to be right — which is why
    /// the spans alone are not the instrument.
    #[test]
    fn a_tilted_plane_has_three_spans_and_rank_two() {
        let d = tmp("tilt");
        let path = d.join("tilt.traj");
        let n = 16usize;
        let h = Header2 {
            seed: 5,
            n_atoms: n,
            dims_declared: 3,
            substeps: 8,
            n_frames: 3,
            dt: 1.0,
            box_w: 40.0,
            box_h: 40.0,
            box_d: 40.0,
            z: vec![1; n],
            content: 0,
        };
        let mut w = TrajWriter2::create(&path, &h).unwrap();
        for f in 0..3 {
            // z = x + 2y exactly: three nonzero spans, and rank two.
            let pos: Vec<[f64; 3]> = (0..n)
                .map(|i| {
                    let x = (i % 4) as f64 + f as f64 * 0.25;
                    let y = (i / 4) as f64;
                    [x, y, x + 2.0 * y]
                })
                .collect();
            w.push(f as u64, f as f64, 300.0, &[], &pos, &pos, None, None)
                .unwrap();
        }
        w.finish().unwrap();
        let m = Trajectory2::read(&path).unwrap().measure();
        assert!(m.span.iter().all(|s| *s > 0.5), "all three spans move: {m:?}");
        assert_eq!(m.dims, 2, "and the rank is still two: {m:?}");
        // AND THE INSTRUMENT'S OWN FLOOR, gauged rather than asserted away. A tilted
        // plane's null direction is a cancellation, so its flatness lands near sqrt(eps)
        // and NOT at zero — 8.0e-9 here, against `TOL` at 1e-6. Two decades of margin, not
        // eight. Bracketed on both sides so a change to the accumulation that moved this
        // floor toward the threshold would fire.
        assert!(
            m.flatness > 1e-12 && m.flatness < 1e-7,
            "the tilted-plane floor moved: {}",
            m.flatness
        );
        assert!(m.flatness < Measured::TOL / 100.0, "margin shrank");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// G6 and P-7: the two ledgers become computable from the artifact, and the planted
    /// impulse is nonzero so a closing ledger is not a ledger that never moved.
    #[test]
    fn the_ledger_rows_close_from_the_artifact_alone() {
        let d = tmp("ledger");
        let path = d.join("l.traj");
        let n = 3usize;
        let h = Header2 {
            seed: 9,
            n_atoms: n,
            dims_declared: 3,
            substeps: 4,
            n_frames: 4,
            dt: 1.0,
            box_w: 10.0,
            box_h: 10.0,
            box_d: 10.0,
            z: vec![8, 1, 1],
            content: CONTENT_FORCES | CONTENT_LEDGER,
        };
        let mut w = TrajWriter2::create(&path, &h).unwrap();
        let pos = vec![[1.0, 2.0, 3.0], [2.0, 2.0, 3.0], [1.0, 3.0, 3.0]];
        let vel = vec![[0.1, 0.0, 0.0], [0.0, 0.2, 0.0], [0.0, 0.0, 0.3]];
        for f in 0..4 {
            let planted = 0.25 * (f as f64 + 1.0);
            let led = Ledger {
                j_ext: [planted, 0.0, 0.0],
                w_hand: 0.0,
                w_thermostat: -planted * 0.5,
                w_barostat: 0.0,
                total: 1.5 - planted * 0.5,
                l0: 1.5,
            };
            let forces = vec![[0.01, 0.02, 0.03]; n];
            w.push(f, f as f64, 300.0, &[0], &pos, &vel, Some(&forces), Some(&led))
                .unwrap();
        }
        w.finish().unwrap();

        let t = Trajectory2::read_strict(&path).unwrap();
        assert!(t.header.has_forces() && t.header.has_ledger());
        for f in &t.frames {
            let l = f.ledger.unwrap();
            // G9c's quantity, from the artifact and no `Sim`.
            assert!(l.energy_residual() < 1e-15, "{l:?}");
            // P-7's control: the ledger CLOSED and it also MOVED. A run where nothing
            // happened would pass the first and fail this.
            assert!(l.j_ext[0] > 0.0 && l.w_thermostat < 0.0, "{l:?}");
            assert_eq!(f.forces.as_ref().unwrap().len(), n);
        }
        // A frame's total momentum, from the artifact: velocities are here and the header
        // says which nucleus each index is.
        let f0 = &t.frames[0];
        assert_eq!(f0.vel.len(), n);
        assert_eq!(t.header.z, vec![8, 1, 1]);
        let _ = std::fs::remove_dir_all(&d);
    }

    /// G9b, the gate `RUNG2_RESULTS.md` §5.3 could not compute, computed from the file.
    ///
    /// Planted rather than merely exercised: the impulse is a KNOWN 0.5 in x, the
    /// velocities move by exactly that impulse over the oxygen's mass, and the residual
    /// must come back at zero. A ledger that closed because nothing moved would pass a
    /// bare `residual < eps` and fail the second assertion.
    #[test]
    fn the_momentum_ledger_closes_from_the_artifact_and_the_impulse_is_real() {
        let d = tmp("mom");
        let path = d.join("m.traj");
        // A mass law the CALLER supplies, as the API demands. These are the engine's
        // electron-mass values for H and O to four figures; the test needs a law, not the
        // element table, and inventing one here is exactly what the signature prevents
        // from happening inside the crate.
        let mass_of = |z: u32| match z {
            1 => 1837.15,
            8 => 29165.1,
            _ => panic!("the fixture has no nucleus {z}"),
        };
        let h = Header2 {
            seed: 11,
            n_atoms: 3,
            dims_declared: 3,
            substeps: 1,
            n_frames: 3,
            dt: 1.0,
            box_w: 12.0,
            box_h: 12.0,
            box_d: 12.0,
            z: vec![8, 1, 1],
            content: CONTENT_LEDGER,
        };
        let mut w = TrajWriter2::create(&path, &h).unwrap();
        let pos = vec![[1.0, 2.0, 3.0], [2.5, 2.0, 3.0], [1.0, 3.5, 3.0]];
        let impulse = 0.5f64;
        for f in 0..3u64 {
            // The whole impulse lands on the oxygen, which is what a wall on one face
            // would do. dv = J / m.
            let dv = impulse * f as f64 / mass_of(8);
            let vel = vec![[0.01 + dv, 0.0, 0.0], [-0.02, 0.03, 0.0], [0.0, -0.03, 0.04]];
            let led = Ledger {
                j_ext: [impulse * f as f64, 0.0, 0.0],
                ..Ledger::default()
            };
            w.push(f, f as f64, 300.0, &[], &pos, &vel, None, Some(&led))
                .unwrap();
        }
        w.finish().unwrap();

        let t = Trajectory2::read_strict(&path).unwrap();
        for f in 0..3 {
            let r = t.momentum_residual(f, mass_of).expect("the ledger is present");
            assert!(r < 1e-9, "frame {f} residual {r}");
        }
        // AND THE CONTROL: the momentum genuinely moved, so the closure above is a
        // measurement and not a description of a scene where nothing happened.
        let p = |f: usize| -> f64 {
            t.header
                .z
                .iter()
                .enumerate()
                .map(|(i, z)| mass_of(*z) * t.frames[f].vel[i][0])
                .sum()
        };
        assert!(
            (p(2) - p(0) - 2.0 * impulse).abs() < 1e-9,
            "the planted impulse must be visible in the raw momentum: {} vs {}",
            p(2) - p(0),
            2.0 * impulse
        );
        assert!((p(2) - p(0)).abs() > 0.9, "and it must be large: {}", p(2) - p(0));
        // A v1 file has no ledger, and that is NOT the same answer as a residual of zero.
        let v1 = d.join("v1.traj");
        let hv1 = Header {
            seed: 1,
            n_atoms: 2,
            dims: 2,
            substeps: 1,
            n_frames: 1,
            dt: 1.0,
            box_w: 5.0,
            box_h: 5.0,
            box_d: 5.0,
            z: vec![1, 1],
        };
        let mut wv = TrajWriter::create(&v1, &hv1).unwrap();
        let pp = vec![[0.0; 3], [1.4; 3]];
        wv.push(0, 0.0, 300.0, &BondSet::empty(), &pp, &pp).unwrap();
        wv.finish().unwrap();
        assert!(
            Trajectory2::read(&v1).unwrap().momentum_residual(0, mass_of).is_none(),
            "a file with no ledger must answer None, never 0.0"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    /// The fluid-element reading is DERIVED, and it counts every atom or says where they
    /// went.
    #[test]
    fn cell_occupancy_is_fractional_and_escapees_are_reported_not_clamped() {
        let d = tmp("occ");
        let path = d.join("o.traj");
        let n = 8usize;
        let h = Header2 {
            seed: 2,
            n_atoms: n,
            dims_declared: 3,
            substeps: 1,
            n_frames: 4,
            dt: 1.0,
            box_w: 10.0,
            box_h: 10.0,
            box_d: 10.0,
            z: vec![1; n],
            content: 0,
        };
        let mut w = TrajWriter2::create(&path, &h).unwrap();
        for f in 0..4usize {
            // Every atom in the low-x half; on the last frame ONE of them leaves the box
            // entirely, which soft walls permit and a fluid reading must not hide.
            let pos: Vec<[f64; 3]> = (0..n)
                .map(|i| {
                    if f == 3 && i == 0 {
                        [-1.0, 5.0, 5.0]
                    } else {
                        [2.0, 5.0, 5.0]
                    }
                })
                .collect();
            w.push(f as u64, f as f64, 300.0, &[], &pos, &pos, None, None)
                .unwrap();
        }
        w.finish().unwrap();
        let t = Trajectory2::read_strict(&path).unwrap();
        let (occ, outside) = t.mean_cell_occupancy([2, 1, 1]).unwrap();
        assert_eq!(outside, 1, "the escapee is counted, once");
        // 8 atoms x 3 frames + 7 atoms x 1 frame = 31 samples, all in the low-x cell,
        // over 4 frames.
        assert_eq!(occ.len(), 2);
        assert!((occ[0] - 31.0 / 4.0).abs() < 1e-12, "{occ:?}");
        assert_eq!(occ[1], 0.0);
        // FRACTIONAL, which is the whole point: 7.75 atoms per cell is not an atom count.
        assert!(occ[0].fract() > 0.0);
        // The grid total plus the escapees accounts for every sample; nothing is clamped
        // into an edge cell and nothing vanishes.
        let total: f64 = occ.iter().sum();
        assert!(
            (total * 4.0 + outside as f64 - (n * 4) as f64).abs() < 1e-9,
            "the books do not balance: {total} x 4 + {outside} != {}",
            n * 4
        );
        assert!(t.mean_cell_occupancy([0, 1, 1]).is_none(), "a zero axis refuses");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// The box's faces belong to the cells `cells.rs` puts them in, and a file too short to
    /// have a version says so as a FORMAT refusal rather than as a path failure.
    #[test]
    fn the_upper_face_is_inside_and_a_stub_file_is_a_format_refusal() {
        let d = tmp("edges");
        let path = d.join("f.traj");
        let h = Header2 {
            seed: 4,
            n_atoms: 4,
            dims_declared: 3,
            substeps: 1,
            n_frames: 1,
            dt: 1.0,
            box_w: 10.0,
            box_h: 10.0,
            box_d: 10.0,
            z: vec![1; 4],
            content: 0,
        };
        let mut w = TrajWriter2::create(&path, &h).unwrap();
        // Exactly on the lower face, exactly on the upper face, just inside, just outside.
        let pos = vec![
            [0.0, 5.0, 5.0],
            [10.0, 5.0, 5.0],
            [9.999, 5.0, 5.0],
            [10.001, 5.0, 5.0],
        ];
        w.push(0, 0.0, 300.0, &[], &pos, &pos, None, None).unwrap();
        w.finish().unwrap();
        let t = Trajectory2::read_strict(&path).unwrap();
        let (occ, outside) = t.mean_cell_occupancy([2, 1, 1]).unwrap();
        assert_eq!(outside, 1, "only the atom PAST the face is outside");
        assert_eq!(occ[0], 1.0, "the lower face belongs to the first cell");
        assert_eq!(occ[1], 2.0, "the upper face belongs to the LAST cell");

        // A stub too short to carry a version: exit 4 (format), never exit 3 (path).
        let stub = d.join("stub.traj");
        std::fs::write(&stub, b"HLNTRA").unwrap();
        let e = match Trajectory2::read(&stub) {
            Ok(_) => panic!("a six-byte file was accepted as a trajectory"),
            Err(e) => e,
        };
        assert_eq!(e.exit_code(), 4, "{e}");
        assert!(e.to_string().contains("6 bytes"), "{e}");
        // An absent file IS a path failure, and must stay one.
        let e = match Trajectory2::read(&d.join("nothing-here.traj")) {
            Ok(_) => panic!("a missing file was read"),
            Err(e) => e,
        };
        assert_eq!(e.exit_code(), 3, "{e}");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// The writer refuses a file whose sections come and go — the thing that would make
    /// every byte after them a guess.
    #[test]
    fn sections_are_fixed_for_the_file() {
        let d = tmp("sec");
        let path = d.join("s.traj");
        let h = Header2 {
            seed: 1,
            n_atoms: 2,
            dims_declared: 3,
            substeps: 1,
            n_frames: 2,
            dt: 1.0,
            box_w: 1.0,
            box_h: 1.0,
            box_d: 1.0,
            z: vec![1, 1],
            content: CONTENT_LEDGER,
        };
        let mut w = TrajWriter2::create(&path, &h).unwrap();
        let p = vec![[0.0; 3]; 2];
        let e = w.push(0, 0.0, 300.0, &[], &p, &p, None, None).unwrap_err();
        assert!(matches!(e, TrajError::Malformed(_)), "{e}");
        w.push(0, 0.0, 300.0, &[], &p, &p, None, Some(&Ledger::default()))
            .unwrap();
        // Unascending bonds are refused too: the reader binary-searches them.
        let e = w
            .push(1, 1.0, 300.0, &[1, 0], &p, &p, None, Some(&Ledger::default()))
            .unwrap_err();
        assert!(matches!(e, TrajError::Malformed(_)), "{e}");
        // And an undefined content bit is refused rather than skipped.
        let bad = Header2 { content: 0x40, ..h };
        assert!(TrajWriter2::create(&d.join("bad.traj"), &bad).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    /// P-8 and P-9: a truncated file reads as a short prefix, and the degenerate scenes
    /// round-trip rather than panicking.
    #[test]
    fn truncation_and_the_degenerate_scenes() {
        let d = tmp("edge");
        let path = d.join("t.traj");
        let h = Header2 {
            seed: 1,
            n_atoms: 2,
            dims_declared: 3,
            substeps: 1,
            n_frames: 100,
            dt: 1.0,
            box_w: 5.0,
            box_h: 5.0,
            box_d: 5.0,
            z: vec![1, 1],
            content: 0,
        };
        let mut w = TrajWriter2::create(&path, &h).unwrap();
        let p = vec![[0.0; 3], [1.4; 3]];
        for f in 0..7 {
            w.push(f, f as f64, 300.0, &[0], &p, &p, None, None).unwrap();
        }
        w.finish().unwrap();
        // Cut the file mid-frame and it still reads the whole frames.
        let bytes = std::fs::read(&path).unwrap();
        std::fs::write(&path, &bytes[..bytes.len() - 13]).unwrap();
        let t = Trajectory2::read_strict(&path).unwrap();
        assert_eq!(t.frames.len(), 6, "the whole frames before the cut, and no error");
        assert_eq!(t.header.n_frames, 100);
        assert!(!t.is_complete());
        assert!(t.truncated_frame, "and it says the last frame was a fragment");
        // A cut ON a frame boundary is the OTHER fact, and must not be reported as this one.
        std::fs::write(&path, &bytes[..bytes.len() - 128]).unwrap();
        let t = Trajectory2::read_strict(&path).unwrap();
        assert_eq!(t.frames.len(), 6);
        assert!(!t.truncated_frame, "a clean boundary cut left no fragment");

        for n in [0usize, 1] {
            let path = d.join(format!("n{n}.traj"));
            let h = Header2 {
                n_atoms: n,
                z: vec![1; n],
                n_frames: 1,
                ..h.clone()
            };
            let mut w = TrajWriter2::create(&path, &h).unwrap();
            let p = vec![[0.0; 3]; n];
            w.push(0, 0.0, 300.0, &[], &p, &p, None, None).unwrap();
            w.finish().unwrap();
            let t = Trajectory2::read_strict(&path).unwrap();
            assert_eq!(t.header.n_atoms, n);
            assert_eq!(t.header.n_pairs(), 0);
            assert!(t.frames[0].bonds.is_empty());
            assert!(t.is_complete());
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    /// The eigensolver on the two cases the campaign actually meets.
    #[test]
    fn jacobi_reads_the_degenerate_spectra() {
        // A rank-2 matrix: third eigenvalue EXACTLY zero.
        let e = jacobi_eigenvalues_3x3([[4.0, 1.0, 0.0], [1.0, 3.0, 0.0], [0.0, 0.0, 0.0]]);
        assert_eq!(e[2], 0.0);
        assert!((e[0] - 4.618033988749895).abs() < 1e-12, "{e:?}");
        assert!((e[1] - 2.381966011250105).abs() < 1e-12, "{e:?}");
        // Already diagonal: no rotation, descending order.
        let e = jacobi_eigenvalues_3x3([[1.0, 0.0, 0.0], [0.0, 9.0, 0.0], [0.0, 0.0, 4.0]]);
        assert_eq!(e, [9.0, 4.0, 1.0]);
        // The zero matrix.
        assert_eq!(jacobi_eigenvalues_3x3([[0.0; 3]; 3]), [0.0; 3]);
    }
}
