//! DETERMINISTIC CHECKPOINT AND REPLAY: the scene's state as bytes, and the promise that
//! restoring them puts the trajectory back on the same rails, bit for bit.
//!
//! FSD-W1 WB-5.4: "seeded scenes replay bit-identically per device class (D0: the class is
//! part of the artifact); checkpoints are exact." This module is the second half of that
//! sentence. The first half — the device class — is not something a serializer can
//! promise, and this file does not pretend to: a checkpoint restored on a machine whose
//! libm rounds a `sqrt` differently is a checkpoint restored into a different chart. What
//! IS promised, and gated, is that on ONE build, the same checkpoint replays to the same
//! bits, and that a checkpoint carries enough state that a restore-and-continue is
//! indistinguishable from never having stopped.
//!
//! # Every float is written as its bits
//!
//! `to_le_bytes` on `f64::to_bits`, never a decimal rendering. A decimal round-trip is
//! correct to 17 significant digits and that is not the same as correct; the whole point
//! of the exercise is the last bit, and a checkpoint that loses it would make the replay
//! gate a measurement of the formatter.
//!
//! # What is NOT in the file, and why that is a refusal rather than an omission
//!
//! The POTENTIAL TABLES are not serialized. They are the artifact the run was performed
//! against — hundreds of kilobytes of measured surface — and a checkpoint is a statement
//! about a trajectory, not a copy of the physics it ran on. But a checkpoint restored
//! against a DIFFERENT curve is a different experiment wearing the same file name, so the
//! checkpoint carries a FUNCTIONAL DIGEST of the physics ([`physics_digest`]) and a
//! restore whose digest disagrees is REFUSED.
//!
//! The digest is functional rather than structural on purpose: it hashes what the tables
//! COMPUTE at a fixed ladder of probe geometries, not how they are stored. Two builds that
//! lay the same surface out differently agree; two surfaces that differ anywhere the probe
//! looks do not. It is also the only form of the check that survives a storage refactor of
//! the table types, which is not a hypothetical — one was in flight in this crate while
//! this file was written.

use crate::sim::{Atom, Boundary, Dims, Sim};

/// Format version. Bumped whenever the field list changes; a mismatch is a refusal, never
/// a best-effort read of a layout that has moved.
///
/// **2 (2026-09-01):** the gravitational field `Sim::g` joined the field list, written
/// after `thermostat_tau`. A v1 checkpoint read under the v2 layout would take the first
/// eight bytes of `w_ext` as `g` and then read every subsequent field one slot early --
/// a scene that restores, recomputes a self-consistent ledger, and is wrong throughout,
/// with no gate able to see it because every invariant would close around the shifted
/// values. That is the exact failure this constant exists to make impossible, so bumping
/// it is not bookkeeping.
/// **3 (2026-09-01, WB-2.4c):** the field became a VECTOR, so one `f64` in that slot
/// became three. A v2 file read under v3 shears by two slots from `w_ext` onward. Note
/// what a version check buys that a length check would not: v2 and v3 differ by sixteen
/// bytes, so a truncated-or-padded v2 could pass a size test and still restore a scene
/// whose field points somewhere nobody chose.
/// v4 (2026-09-02): the `acuity` receipt column joins the ledger block after `barostat`.
/// v5 (2026-09-02): the four-body pair `(de4_eval_count: u64, de4_enabled: bool)` becomes
/// the many-body triple `(many_body_evals: u64, many_body_order: u64, many_body_unserved:
/// u64)` — the sector is any-order now and its declared order is state a restore must carry.
/// v6 (2026-09-04, FIELD-1): the `field` receipt column joins the ledger block after `acuity`,
/// and the field's charge `field_q` (0.0 = off) follows `frame`; a restore re-derives the
/// per-atom assignment from the live rows.
/// v7 (2026-09-05, FIELD-3): the `seam` receipt column joins the ledger block after `field`,
/// and the seam's state (`seam_on: u32`, `seam_a`, `seam_b`) follows `field_q`; a restore
/// re-derives the unit assignment at its first force pass without posting a transition.
/// v8 (2026-09-05, FIELD-4): the seam's three further coefficients (`p`, `c`, `c6`) follow
/// `seam_b`.
pub const CHECKPOINT_VERSION: u32 = 8;

const MAGIC: [u8; 8] = *b"HOLONCK1";

/// Why a restore was refused. Every variant is a case where continuing would produce a
/// number rather than an error, which is the failure mode worth spending an enum on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CheckpointError {
    /// Not a checkpoint, or truncated.
    Malformed,
    /// Written by a different format version.
    Version(u32),
    /// The trailing checksum does not match the body: the bytes were altered in transit.
    Corrupt,
    /// The scene's physics is not the physics this checkpoint was taken against — a
    /// different curve, a table loaded or unloaded, a three-body surface swapped.
    PhysicsMismatch { expected: u64, found: u64 },
}

impl CheckpointError {
    pub fn plain(self) -> &'static str {
        match self {
            CheckpointError::Malformed => {
                "this is not a checkpoint, or it is shorter than the state it claims to hold"
            }
            CheckpointError::Version(_) => {
                "this checkpoint was written by a different version of the format; the field \
                 layout has moved and reading it as this one would silently misinterpret it"
            }
            CheckpointError::Corrupt => {
                "the checkpoint's checksum does not match its body, so some byte of it \
                 changed between writing and reading"
            }
            CheckpointError::PhysicsMismatch { .. } => {
                "this checkpoint was taken against a different potential than the one now \
                 loaded; restoring it would continue a trajectory on a curve it never ran on"
            }
        }
    }
}

/// A byte-exact snapshot of the scene.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Checkpoint {
    pub bytes: Vec<u8>,
}

impl Checkpoint {
    /// The checkpoint's own content hash — the identity two checkpoints are compared by.
    pub fn digest(&self) -> u64 {
        fnv(&self.bytes)
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

/// FNV-1a over bytes. Not a cryptographic hash and not claimed to be one: it detects a
/// changed byte, which is what a checkpoint checksum is for. An adversary editing a
/// checkpoint is not in this threat model, and a hash that pretended otherwise would be
/// making a promise the rest of the file cannot keep.
fn fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

struct Writer {
    out: Vec<u8>,
}

impl Writer {
    fn f64(&mut self, v: f64) {
        self.out.extend_from_slice(&v.to_bits().to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.out.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.out.extend_from_slice(&v.to_le_bytes());
    }
    fn u8(&mut self, v: u8) {
        self.out.push(v);
    }
    fn bool(&mut self, v: bool) {
        self.out.push(u8::from(v));
    }
    fn opt_f64_pair(&mut self, v: Option<(f64, f64)>) {
        match v {
            None => {
                self.bool(false);
                self.f64(0.0);
                self.f64(0.0);
            }
            Some((a, b)) => {
                self.bool(true);
                self.f64(a);
                self.f64(b);
            }
        }
    }
}

struct Reader<'a> {
    src: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], CheckpointError> {
        if self.at + n > self.src.len() {
            return Err(CheckpointError::Malformed);
        }
        let s = &self.src[self.at..self.at + n];
        self.at += n;
        Ok(s)
    }
    fn f64(&mut self) -> Result<f64, CheckpointError> {
        let b = self.take(8)?;
        Ok(f64::from_bits(u64::from_le_bytes(b.try_into().unwrap())))
    }
    fn u64(&mut self) -> Result<u64, CheckpointError> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes(b.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, CheckpointError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes(b.try_into().unwrap()))
    }
    fn u8(&mut self) -> Result<u8, CheckpointError> {
        Ok(self.take(1)?[0])
    }
    fn bool(&mut self) -> Result<bool, CheckpointError> {
        Ok(self.u8()? != 0)
    }
    fn opt_f64_pair(&mut self) -> Result<Option<(f64, f64)>, CheckpointError> {
        let present = self.bool()?;
        let a = self.f64()?;
        let b = self.f64()?;
        Ok(if present { Some((a, b)) } else { None })
    }
}

fn boundary_code(b: Boundary) -> u8 {
    match b {
        Boundary::Walls => 0,
        Boundary::Open => 1,
        Boundary::Periodic => 2,
    }
}

fn boundary_of(c: u8) -> Result<Boundary, CheckpointError> {
    Ok(match c {
        0 => Boundary::Walls,
        1 => Boundary::Open,
        2 => Boundary::Periodic,
        _ => return Err(CheckpointError::Malformed),
    })
}

/// THE FUNCTIONAL DIGEST of the physics a scene is running on.
///
/// Every filled pair curve is evaluated at a fixed radial ladder and every loaded
/// many-body surface at a fixed geometry ladder; the resulting bits are hashed. Two scenes
/// agreeing here compute the same forces at every probed configuration, which is the
/// property a restore actually needs — not that the tables are stored identically.
///
/// The ladders are deliberately coarse (a few dozen points) because the digest runs once
/// per checkpoint, not per step, and its job is to catch A DIFFERENT TABLE rather than to
/// certify equality. A surface that differs only between probe points would pass, and that
/// is a stated limit rather than a hidden one.
pub fn physics_digest(s: &Sim) -> u64 {
    let mut w = Writer { out: Vec::new() };
    // The pair bank: every slot, filled or not, so a table appearing or disappearing moves
    // the digest as surely as one changing.
    for slot in 0..crate::bank::MAX_TABLES {
        let t = s.bank.table_slot(slot);
        w.bool(t.is_loaded());
        if !t.is_loaded() {
            continue;
        }
        for k in 0..48 {
            let r = 0.5 + 0.5 * k as f64;
            let (v, d, c) = t.eval(r);
            w.f64(v);
            w.f64(d);
            w.f64(c);
        }
    }
    // The many-body surfaces, on a ladder of triangles that stays inside every domain.
    w.bool(s.trimer.loaded);
    w.bool(s.water.loaded);
    w.bool(s.ooh.loaded);
    w.bool(s.ozone.loaded);
    w.u32(s.trimers.len() as u32);
    for k in 0..16 {
        let a = 1.5 + 0.25 * k as f64;
        let b = 2.0 + 0.30 * k as f64;
        let c = 2.5 + 0.20 * k as f64;
        let (v, g) = s.trimer.eval([a, b, c]);
        w.f64(v);
        for x in g {
            w.f64(x);
        }
        let (v, g) = s.water.eval(a, b, c);
        w.f64(v);
        for x in g {
            w.f64(x);
        }
        let (v, g) = s.ooh.eval(a, b, c);
        w.f64(v);
        for x in g {
            w.f64(x);
        }
        let (v, g) = s.ozone.eval(a, b, c);
        w.f64(v);
        for x in g {
            w.f64(x);
        }
    }
    // THE LONG-RANGE SECTOR (node B2), and it belongs in the DIGEST rather than in the
    // file for the reason the module header gives about the tables: a far sector is
    // physics the run was performed against, and a checkpoint restored without it is a
    // different experiment wearing the same file name. The digest already refuses that
    // for a changed curve; this makes it refuse it for a changed far sector too.
    //
    // NOTHING is written when there is none, so every checkpoint that existed before B2
    // hashes exactly the bytes it hashed before and every banked replay fingerprint in the
    // tree stays valid. That is why the `if let` guards the whole block rather than
    // writing a `false` flag.
    //
    // What this does NOT do, stated so it is a fence and not an omission: it does not
    // SERIALIZE the far sector, so restoring a far-sector checkpoint still requires the
    // caller to declare the same one before restoring. The exit is a checkpoint format
    // version that carries `R_s`, the budget and the shell count; that is a format change
    // touching every lane's banked files, so it is named here rather than taken.
    if let Some(f) = &s.far {
        w.f64(f.r_s());
        w.f64(f.r_f());
        w.f64(f.budget());
        w.u32(f.shells() as u32);
        w.bool(f.is_fenced());
        // The tail models themselves, probed the way the tables are: what they COMPUTE,
        // not how they are stored.
        for slot in 0..crate::bank::MAX_TABLES {
            match f.model(slot) {
                Some(m) => {
                    w.bool(true);
                    for k in 0..8 {
                        let r = f.r_s() + 0.25 * (k + 1) as f64;
                        let (v, d) = m.eval(r);
                        w.f64(v);
                        w.f64(d);
                    }
                }
                None => w.bool(false),
            }
        }
    }
    fnv(&w.out)
}

impl Sim {
    /// Serialize the scene's DYNAMICAL STATE — everything a continuation needs and nothing
    /// that can be recomputed from it.
    pub fn checkpoint(&self) -> Checkpoint {
        let mut w = Writer { out: Vec::new() };
        w.out.extend_from_slice(&MAGIC);
        w.u32(CHECKPOINT_VERSION);
        w.u64(physics_digest(self));

        // --- the box and its rules ---
        w.f64(self.width);
        w.f64(self.height);
        w.f64(self.depth);
        w.f64(self.wall_inset);
        w.u8(boundary_code(self.boundary));
        w.u8(match self.dims {
            Dims::Two => 0,
            Dims::Three => 1,
        });
        w.opt_f64_pair(self.pair_switch());
        w.f64(self.truncation_floor());

        // --- the scene ---
        w.u64(self.n as u64);
        for a in self.atoms.iter() {
            w.f64(a.x);
            w.f64(a.y);
            w.f64(a.z);
            w.f64(a.vx);
            w.f64(a.vy);
            w.f64(a.vz);
            w.u32(a.species.z);
        }

        // --- the hand and the thermostat ---
        w.u64(self.grabbed.map(|g| g as u64 + 1).unwrap_or(0));
        w.f64(self.anchor.0);
        w.f64(self.anchor.1);
        w.f64(self.anchor.2);
        w.bool(self.thermostat_on);
        w.f64(self.target_temperature);
        w.f64(self.thermostat_tau);
        // The gravitational field is a SETTING and belongs here; `e_grav` is derived and
        // does not, for the same reason no other energy term is stored. A checkpoint that
        // dropped `g` would restore a scene that looks identical and falls differently,
        // and `recompute()` below would rebuild a consistent-but-wrong ledger around it.
        w.f64(self.g_vec.0);
        w.f64(self.g_vec.1);
        w.f64(self.g_vec.2);

        // --- the ledger, including the receipt columns ---
        w.f64(self.w_ext);
        w.f64(self.work.hand);
        w.f64(self.work.thermostat);
        w.f64(self.work.barostat);
        w.f64(self.work.acuity);
        w.f64(self.work.field);
        w.f64(self.work.seam);
        w.f64(self.l0);
        w.f64(self.p0.0);
        w.f64(self.p0.1);
        w.f64(self.p0.2);
        w.f64(self.j_ext.0);
        w.f64(self.j_ext.1);
        w.f64(self.j_ext.2);
        w.f64(self.time);
        w.u64(self.steps);
        w.u64(self.frame);
        w.f64(self.field.map_or(0.0, |m| m.q_h));
        w.u32(u32::from(self.seam.is_some()));
        w.f64(self.seam.map_or(0.0, |m| m.a));
        w.f64(self.seam.map_or(0.0, |m| m.b));
        w.f64(self.seam.map_or(0.0, |m| m.p));
        w.f64(self.seam.map_or(0.0, |m| m.c));
        w.f64(self.seam.map_or(0.0, |m| m.c6));
        w.f64(self.e_ref);
        w.f64(self.drift_peak);
        w.f64(self.momentum_residual_peak);
        w.f64(self.e_rel_max);
        w.u64(self.many_body_evals);
        w.u64(self.many_body_order as u64);
        w.u64(self.many_body_unserved);

        // --- the envelope state the drift bound is built from ---
        for v in self.envelope_state() {
            w.f64(v);
        }
        for v in self.clock_state() {
            w.f64(v);
        }
        w.bool(self.timescale.calibrated);
        w.bool(self.timescale.allow_dt_growth);

        let sum = fnv(&w.out);
        w.u64(sum);
        Checkpoint { bytes: w.out }
    }

    /// Restore a checkpoint onto this scene, or refuse and leave the scene untouched.
    ///
    /// "Untouched" is load-bearing: every refusal is decided BEFORE the first field is
    /// written, so a scene that refused a checkpoint is still the scene it was and can
    /// carry on. A half-restored scene would be a state no run ever produced.
    pub fn restore(&mut self, c: &Checkpoint) -> Result<(), CheckpointError> {
        if c.bytes.len() < MAGIC.len() + 4 + 8 + 8 {
            return Err(CheckpointError::Malformed);
        }
        let body = &c.bytes[..c.bytes.len() - 8];
        let stated = u64::from_le_bytes(c.bytes[c.bytes.len() - 8..].try_into().unwrap());
        if fnv(body) != stated {
            return Err(CheckpointError::Corrupt);
        }
        let mut r = Reader { src: body, at: 0 };
        if r.take(MAGIC.len())? != MAGIC {
            return Err(CheckpointError::Malformed);
        }
        let version = r.u32()?;
        if version != CHECKPOINT_VERSION {
            return Err(CheckpointError::Version(version));
        }
        let expected = r.u64()?;
        let found = physics_digest(self);
        if expected != found {
            return Err(CheckpointError::PhysicsMismatch { expected, found });
        }

        // Everything below this line writes. Nothing above it did.
        let width = r.f64()?;
        let height = r.f64()?;
        let depth = r.f64()?;
        let wall_inset = r.f64()?;
        let boundary = boundary_of(r.u8()?)?;
        let dims = match r.u8()? {
            0 => Dims::Two,
            1 => Dims::Three,
            _ => return Err(CheckpointError::Malformed),
        };
        let pair_switch = r.opt_f64_pair()?;
        let pair_floor = r.f64()?;

        let n = r.u64()? as usize;
        // A length field is an instruction to allocate, and a corrupt one is an
        // instruction to allocate absurdly. The checksum has already passed, so this is
        // belt and braces rather than the primary defence — but the primary defence
        // against a hostile file is not to have one, and saying so is better than
        // implying otherwise.
        if n > (body.len() - r.at) / 52 {
            return Err(CheckpointError::Malformed);
        }
        let mut atoms: Vec<Atom> = Vec::with_capacity(n);
        for _ in 0..n {
            let x = r.f64()?;
            let y = r.f64()?;
            let z = r.f64()?;
            let vx = r.f64()?;
            let vy = r.f64()?;
            let vz = r.f64()?;
            let zz = r.u32()?;
            let species =
                holon_chem::elements::by_z(zz).ok_or(CheckpointError::Malformed)?;
            atoms.push(Atom {
                x,
                y,
                z,
                vx,
                vy,
                vz,
                species,
            });
        }

        let grabbed_raw = r.u64()?;
        let anchor = (r.f64()?, r.f64()?, r.f64()?);
        let thermostat_on = r.bool()?;
        let target_temperature = r.f64()?;
        let thermostat_tau = r.f64()?;
        let g_vec = (r.f64()?, r.f64()?, r.f64()?);

        let w_ext = r.f64()?;
        let hand = r.f64()?;
        let thermostat = r.f64()?;
        let barostat = r.f64()?;
        let acuity = r.f64()?;
        let field_col = r.f64()?;
        let seam_col = r.f64()?;
        let l0 = r.f64()?;
        let p0 = (r.f64()?, r.f64()?, r.f64()?);
        let j_ext = (r.f64()?, r.f64()?, r.f64()?);
        let time = r.f64()?;
        let steps = r.u64()?;
        let frame = r.u64()?;
        let field_q = r.f64()?;
        let seam_on = r.u32()?;
        let seam_a = r.f64()?;
        let seam_b = r.f64()?;
        let seam_p = r.f64()?;
        let seam_c = r.f64()?;
        let seam_c6 = r.f64()?;
        let e_ref = r.f64()?;
        let drift_peak = r.f64()?;
        let momentum_residual_peak = r.f64()?;
        let e_rel_max = r.f64()?;
        let many_body_evals = r.u64()?;
        let many_body_order = r.u64()? as usize;
        let many_body_unserved = r.u64()?;

        let mut envelope = [0.0f64; ENVELOPE_WORDS];
        for v in envelope.iter_mut() {
            *v = r.f64()?;
        }
        let mut clock = [0.0f64; CLOCK_WORDS];
        for v in clock.iter_mut() {
            *v = r.f64()?;
        }
        let calibrated = r.bool()?;
        let allow_dt_growth = r.bool()?;

        // --- commit ---
        self.width = width;
        self.height = height;
        self.depth = depth;
        self.wall_inset = wall_inset;
        self.boundary = boundary;
        self.dims = dims;
        self.set_pair_switch_raw(pair_switch, pair_floor);
        self.resize_storage(n);
        self.atoms.copy_from_slice(&atoms);
        self.sync_species();
        self.grabbed = if grabbed_raw == 0 {
            None
        } else {
            Some(grabbed_raw as usize - 1)
        };
        self.anchor = anchor;
        self.thermostat_on = thermostat_on;
        self.target_temperature = target_temperature;
        self.thermostat_tau = thermostat_tau;
        self.g_vec = g_vec;
        self.w_ext = w_ext;
        self.work.hand = hand;
        self.work.thermostat = thermostat;
        self.work.barostat = barostat;
        self.work.acuity = acuity;
        self.work.field = field_col;
        self.work.seam = seam_col;
        self.field = if field_q != 0.0 { Some(crate::field::FieldModel { q_h: field_q }) } else { None };
        self.seam = if seam_on != 0 { Some(crate::seam::SeamModel { a: seam_a, b: seam_b, p: seam_p, c: seam_c, c6: seam_c6 }) } else { None };
        self.seam_assigned = false;
        self.l0 = l0;
        self.p0 = p0;
        self.j_ext = j_ext;
        self.time = time;
        self.steps = steps;
        self.frame = frame;
        self.e_ref = e_ref;
        self.drift_peak = drift_peak;
        self.momentum_residual_peak = momentum_residual_peak;
        self.e_rel_max = e_rel_max;
        self.many_body_evals = many_body_evals;
        self.many_body_order = many_body_order;
        self.many_body_unserved = many_body_unserved;
        self.set_envelope_state(envelope);
        self.set_clock_state(clock);
        self.timescale.calibrated = calibrated;
        self.timescale.allow_dt_growth = allow_dt_growth;

        // The derived state — forces, energies, pair readings, the cell list — is
        // RECOMPUTED rather than stored. It is a function of what was just written, so
        // storing it would be storing the same fact twice and inviting the two copies to
        // disagree; and recomputing it is the same call the step loop makes.
        self.recompute();
        self.refresh_pairs();
        Ok(())
    }
}

/// How many `f64` the envelope snapshot carries. Named so the writer and the reader cannot
/// drift apart by one.
pub const ENVELOPE_WORDS: usize = 4;
/// How many `f64` the clock snapshot carries.
pub const CLOCK_WORDS: usize = 14;
