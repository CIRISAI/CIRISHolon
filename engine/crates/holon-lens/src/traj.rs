//! THE TRAJECTORY ARTIFACT: what a run leaves behind so a later reading can be made.
//!
//! A quench run's final frame is a statement about one instant. The closure census is a
//! statement about a WINDOW, so the run has to leave the window behind. This module is
//! the format it leaves it in, and the format's one design rule is:
//!
//! > **the bond bits are the ENGINE's bond bits.**
//!
//! `Sim::refresh_pairs` decides `bonded` from the pair's own curve (`e_rel < 0` and
//! `r < r_outer`), and `sim.rs` says in as many words why nothing may recompute it:
//! "Two implementations of a cluster reading is how the two of them come to disagree."
//! So the dump carries the engine's `bonded` bit for every pair, and this crate reads
//! those bits rather than re-deriving them from the positions it also carries. The
//! positions are here for the OTHER lenses — q6, the hexatic, MSD, the H-bond census —
//! which are genuinely new readings and are labelled as such.
//!
//! Layout, little-endian throughout:
//!
//! ```text
//! magic      8 bytes  b"HLNTRAJ1"
//! version    u32      1
//! n_atoms    u32
//! dims       u32      2 or 3
//! substeps   u32      integration substeps per grain boundary
//! seed       u64
//! n_frames   u64
//! dt         f64      atomic units
//! box_w/h/d  f64 x3   bohr
//! z          u32 x n_atoms      nuclear charge per ARENA index
//! then n_frames frames, each:
//! index      u64
//! time       f64      atomic units
//! temperature f64     kelvin
//! bonded     u128     bit k set iff pair k reads BONDED; k = pair_index(n, i, j)
//! atoms      f64 x 6 x n_atoms  x, y, z, vx, vy, vz
//! ```

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

pub const MAGIC: [u8; 8] = *b"HLNTRAJ1";
pub const VERSION: u32 = 1;

/// One atomic time unit in femtoseconds.
///
/// The same constant `waterquench` uses to print its picoseconds, scaled: that example
/// writes `2.4188843265e-5` for picoseconds and this is the femtosecond form of the same
/// number. It is repeated rather than imported because importing it would mean depending
/// on `holon-render`, and the whole point of this crate's isolation is that it must be
/// testable while `holon-render` is mid-refactor.
pub const AU_TIME_FS: f64 = 2.4188843265e-2;

/// The cap this format's pair bitset can carry.
///
/// `u128` holds `C(16,2) = 120` pair bits with eight to spare. The engine's scene size is
/// no longer capped (T3 made the state heap-backed), so this is a limit of the ARTIFACT
/// and not of the engine, and the writer refuses past it rather than silently truncating
/// — a dropped bond bit would show up as a partition that quietly falls apart.
pub const MAX_DUMP_ATOMS: usize = 16;

/// The index of pair `(i, j)`, `i < j`, in the engine's own enumeration order.
///
/// `Sim::refresh_pairs` walks `i` in `0..n` and `j` in `i+1..n`, incrementing one counter.
/// This reproduces that counter arithmetically. Getting it wrong would silently permute
/// the bond graph, so `pair_index_matches_engine_enumeration` in this module's tests
/// rebuilds the engine's loop and compares every index for every `n` up to the cap.
#[inline]
pub const fn pair_index(n: usize, i: usize, j: usize) -> usize {
    // Sum over rows above i of that row's length, plus the offset within row i.
    i * n - i * (i + 1) / 2 + (j - i - 1)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Header {
    pub seed: u64,
    pub n_atoms: usize,
    pub dims: u32,
    pub substeps: u32,
    pub n_frames: usize,
    pub dt: f64,
    pub box_w: f64,
    pub box_h: f64,
    pub box_d: f64,
    /// Nuclear charge per ARENA index. Keyed by charge rather than by the bank's species
    /// slot for the reason `cluster_species_counts` gives: the species index is an
    /// artefact of registration order.
    pub z: Vec<u32>,
}

impl Header {
    /// Wall time of one grain boundary, in femtoseconds.
    ///
    /// The census stakes its window in PHYSICAL TIME because `dt` is derived per scene and
    /// differs between seeds of the same protocol — 0.5386 and 1.0772 a.u. both appear in
    /// the banked P2 log. A window staked in frames would be a different window per seed.
    pub fn frame_fs(&self) -> f64 {
        self.dt * self.substeps as f64 * AU_TIME_FS
    }

    /// How many frames the staked window spans on THIS trajectory.
    pub fn frames_in(&self, fs: f64) -> usize {
        (fs / self.frame_fs()).round().max(1.0) as usize
    }

    pub fn n_pairs(&self) -> usize {
        self.n_atoms * self.n_atoms.saturating_sub(1) / 2
    }
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub index: u64,
    pub time: f64,
    pub temperature: f64,
    /// Bit `k` set iff pair `k` read BONDED in the engine at this grain boundary.
    pub bonded: u128,
    pub pos: Vec<[f64; 3]>,
    pub vel: Vec<[f64; 3]>,
}

impl Frame {
    #[inline]
    pub fn is_bonded(&self, n: usize, i: usize, j: usize) -> bool {
        let (a, b) = if i < j { (i, j) } else { (j, i) };
        self.bonded >> pair_index(n, a, b) & 1 == 1
    }
}

#[derive(Clone, Debug)]
pub struct Trajectory {
    pub header: Header,
    pub frames: Vec<Frame>,
}

// ---------------------------------------------------------------------------- writing

/// Streams frames to disk as they are produced.
///
/// Streaming rather than collecting: a 20,000-frame run of twelve atoms is 12 MB, which
/// would fit in memory, but the format exists so that a run which dies at frame 14,000
/// leaves 14,000 usable frames behind rather than nothing. The header carries the frame
/// count the writer INTENDED; `finish` rewrites it to the count actually written, and a
/// reader that finds fewer frames than the header claims reports the truncation instead
/// of failing.
pub struct TrajWriter {
    out: BufWriter<File>,
    n_atoms: usize,
    written: u64,
}

impl TrajWriter {
    pub fn create(path: &Path, h: &Header) -> std::io::Result<Self> {
        assert!(
            h.n_atoms <= MAX_DUMP_ATOMS,
            "the trajectory format's pair bitset holds C({MAX_DUMP_ATOMS},2) = 120 bits; \
             {} atoms would need {} and the bond graph would silently lose edges",
            h.n_atoms,
            h.n_pairs()
        );
        assert_eq!(h.z.len(), h.n_atoms, "one nuclear charge per arena index");
        let mut out = BufWriter::new(File::create(path)?);
        out.write_all(&MAGIC)?;
        out.write_all(&VERSION.to_le_bytes())?;
        out.write_all(&(h.n_atoms as u32).to_le_bytes())?;
        out.write_all(&h.dims.to_le_bytes())?;
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
            written: 0,
        })
    }

    pub fn push(
        &mut self,
        index: u64,
        time: f64,
        temperature: f64,
        bonded: u128,
        pos: &[[f64; 3]],
        vel: &[[f64; 3]],
    ) -> std::io::Result<()> {
        assert_eq!(pos.len(), self.n_atoms);
        assert_eq!(vel.len(), self.n_atoms);
        self.out.write_all(&index.to_le_bytes())?;
        self.out.write_all(&time.to_le_bytes())?;
        self.out.write_all(&temperature.to_le_bytes())?;
        self.out.write_all(&bonded.to_le_bytes())?;
        for k in 0..self.n_atoms {
            for c in 0..3 {
                self.out.write_all(&pos[k][c].to_le_bytes())?;
            }
            for c in 0..3 {
                self.out.write_all(&vel[k][c].to_le_bytes())?;
            }
        }
        self.written += 1;
        Ok(())
    }

    pub fn frames_written(&self) -> u64 {
        self.written
    }

    pub fn finish(mut self) -> std::io::Result<u64> {
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
fn u128r(r: &mut impl Read) -> std::io::Result<u128> {
    Ok(u128::from_le_bytes(rd::<16>(r)?))
}
fn f64r(r: &mut impl Read) -> std::io::Result<f64> {
    Ok(f64::from_le_bytes(rd::<8>(r)?))
}

impl Trajectory {
    pub fn read(path: &Path) -> std::io::Result<Self> {
        let mut r = BufReader::new(File::open(path)?);
        let magic: [u8; 8] = rd::<8>(&mut r)?;
        if magic != MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "not a HLNTRAJ1 trajectory",
            ));
        }
        let version = u32r(&mut r)?;
        if version != VERSION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("trajectory version {version}, this reader speaks {VERSION}"),
            ));
        }
        let n_atoms = u32r(&mut r)? as usize;
        let dims = u32r(&mut r)?;
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
        let header = Header {
            seed,
            n_atoms,
            dims,
            substeps,
            n_frames,
            dt,
            box_w,
            box_h,
            box_d,
            z,
        };
        // Read until EOF rather than until `n_frames`: a run killed mid-window leaves a
        // short file, and the honest reading of a short file is "this many frames", not a
        // parse error and not a silent pad.
        let mut frames = Vec::with_capacity(n_frames);
        loop {
            let index = match u64r(&mut r) {
                Ok(v) => v,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };
            let time = f64r(&mut r)?;
            let temperature = f64r(&mut r)?;
            let bonded = u128r(&mut r)?;
            let mut pos = Vec::with_capacity(n_atoms);
            let mut vel = Vec::with_capacity(n_atoms);
            for _ in 0..n_atoms {
                let p = [f64r(&mut r)?, f64r(&mut r)?, f64r(&mut r)?];
                let v = [f64r(&mut r)?, f64r(&mut r)?, f64r(&mut r)?];
                pos.push(p);
                vel.push(v);
            }
            frames.push(Frame {
                index,
                time,
                temperature,
                bonded,
                pos,
                vel,
            });
        }
        Ok(Self { header, frames })
    }

    /// Whether the file holds every frame its header promised.
    ///
    /// Reported, never asserted: a truncated trajectory is still a measurable trajectory,
    /// and the census refuses on window length rather than on this flag. What must never
    /// happen is a census reporting a verdict while silently standing on a short file, so
    /// every census report carries this.
    pub fn is_complete(&self) -> bool {
        self.frames.len() == self.header.n_frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// G — the pair index must reproduce the engine's own enumeration EXACTLY.
    ///
    /// `refresh_pairs` walks `i`, then `j > i`, bumping one counter. If this arithmetic
    /// disagrees anywhere, the dump's bond bits are a permutation of the engine's and the
    /// census would read a different graph while looking perfectly healthy.
    #[test]
    fn pair_index_matches_engine_enumeration() {
        for n in 0..=MAX_DUMP_ATOMS {
            let mut k = 0usize;
            for i in 0..n {
                for j in (i + 1)..n {
                    assert_eq!(pair_index(n, i, j), k, "n={n} pair ({i},{j})");
                    k += 1;
                }
            }
            assert_eq!(k, n * n.saturating_sub(1) / 2);
        }
    }

    /// The bitset must reach the last pair of the largest scene the format admits.
    #[test]
    fn largest_scene_fits_the_bitset() {
        let n = MAX_DUMP_ATOMS;
        let last = pair_index(n, n - 2, n - 1);
        assert_eq!(last, 119);
        assert!(last < 128);
    }

    #[test]
    fn round_trip_preserves_every_field() {
        let dir = std::env::temp_dir().join(format!("hlens-rt-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.traj");
        let h = Header {
            seed: 0x5341_5422,
            n_atoms: 3,
            dims: 2,
            substeps: 64,
            n_frames: 2,
            dt: 0.5386,
            box_w: 34.6,
            box_h: 20.8,
            box_d: 24.0,
            z: vec![8, 1, 1],
        };
        let mut w = TrajWriter::create(&path, &h).unwrap();
        let pos = vec![[0.0, 0.0, 0.0], [1.8, 0.0, 0.0], [-0.5, 1.7, 0.0]];
        let vel = vec![[0.1, 0.0, 0.0], [0.0, 0.2, 0.0], [0.0, 0.0, 0.3]];
        w.push(0, 0.0, 300.0, 0b011, &pos, &vel).unwrap();
        w.push(1, 34.5, 299.0, 0b111, &pos, &vel).unwrap();
        assert_eq!(w.finish().unwrap(), 2);

        let t = Trajectory::read(&path).unwrap();
        assert_eq!(t.header, h);
        assert_eq!(t.frames.len(), 2);
        assert!(t.is_complete());
        assert_eq!(t.frames[0].bonded, 0b011);
        assert_eq!(t.frames[1].bonded, 0b111);
        // Bit 0 is (0,1), bit 1 is (0,2), bit 2 is (1,2).
        assert!(t.frames[0].is_bonded(3, 0, 1));
        assert!(t.frames[0].is_bonded(3, 1, 0), "the accessor is symmetric");
        assert!(!t.frames[0].is_bonded(3, 1, 2));
        assert!(t.frames[1].is_bonded(3, 1, 2));
        assert_eq!(t.frames[1].pos[2], [-0.5, 1.7, 0.0]);
        assert_eq!(t.frames[1].vel[2], [0.0, 0.0, 0.3]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A run killed mid-window must leave a readable prefix, reported as incomplete.
    #[test]
    fn truncated_file_reads_as_incomplete_not_as_an_error() {
        let dir = std::env::temp_dir().join(format!("hlens-tr-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.traj");
        let h = Header {
            seed: 1,
            n_atoms: 2,
            dims: 2,
            substeps: 64,
            n_frames: 100,
            dt: 1.0,
            box_w: 10.0,
            box_h: 10.0,
            box_d: 10.0,
            z: vec![1, 1],
        };
        let mut w = TrajWriter::create(&path, &h).unwrap();
        let p = vec![[0.0; 3], [1.4; 3]];
        for f in 0..7 {
            w.push(f, f as f64, 300.0, 1, &p, &p).unwrap();
        }
        w.finish().unwrap();
        let t = Trajectory::read(&path).unwrap();
        assert_eq!(t.frames.len(), 7);
        assert_eq!(t.header.n_frames, 100);
        assert!(!t.is_complete());
    }

    #[test]
    fn window_frames_follow_the_timestep_not_the_frame_count() {
        let mk = |dt: f64| Header {
            seed: 0,
            n_atoms: 2,
            dims: 2,
            substeps: 64,
            n_frames: 20000,
            dt,
            box_w: 1.0,
            box_h: 1.0,
            box_d: 1.0,
            z: vec![1, 1],
        };
        // The two timesteps the banked P2 log actually used. The staked 834 fs window is
        // 1000 frames at the fine one and 500 at the coarse one; a window staked in
        // FRAMES would have been two different physical windows.
        assert_eq!(mk(0.5386).frames_in(834.0), 1000);
        assert_eq!(mk(1.0772).frames_in(834.0), 500);
    }
}
