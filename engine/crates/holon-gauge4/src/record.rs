//! The stored record: gauge-invariant fields of ONE configuration on one three-dimensional
//! slice at fixed Euclidean time, at a ladder of flow times, in a small binary file.
//!
//! Thermalising takes minutes; a page may not spend 200 ms. So the configuration is made
//! ahead of time (`examples/export.rs`) and only what is drawn is kept: the action density,
//! the topological charge density and the Polyakov loop, each QUANTISED TO 16 BITS against
//! that snapshot's own largest value (the scales are in the file as f32). The links are not
//! stored. The provenance line in the file says how it was made.
//!
//! Layout, little-endian: `HG4F`, version u16 (1), the four sides u16, the time slice u16,
//! the snapshot count u16, beta f64, the provenance's length u32 and its UTF-8 bytes; then
//! per snapshot: flow time f64, the whole lattice's topological charge f64 and mean action
//! density f64, the slice's largest action density f32 and largest |charge density| f32,
//! then the action density u16 per site, the charge density i16 per site, and the Polyakov
//! loop's real and imaginary parts i16 per site (full scale 1).
use crate::lattice::Lattice;

/// The fields on one time slice at one flow time; site `x0 + n0 (x1 + n1 x2)`.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    /// The flow time, in units of the lattice spacing squared.
    pub t: f64,
    /// The topological charge and the mean clover action density of the WHOLE lattice.
    pub q_total: f64,
    pub e_mean: f64,
    pub action: Vec<f32>,
    pub charge: Vec<f32>,
    pub polyakov: Vec<[f32; 2]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub dims: [usize; 4],
    pub beta: f64,
    /// Which Euclidean time slice the three-dimensional fields are of.
    pub tau: usize,
    pub provenance: String,
    pub snapshots: Vec<Snapshot>,
}

impl Snapshot {
    /// Measure a configuration (already flowed to time `t`) on time slice `tau`.
    pub fn measure(l: &Lattice, t: f64, tau: usize) -> Snapshot {
        let v3 = l.dims[0] * l.dims[1] * l.dims[2];
        let (e, q) = l.densities();
        let slice = tau * v3..(tau + 1) * v3;
        Snapshot {
            t,
            q_total: q.iter().sum(),
            e_mean: e.iter().sum::<f64>() / e.len() as f64,
            action: e[slice.clone()].iter().map(|&x| x as f32).collect(),
            charge: q[slice].iter().map(|&x| x as f32).collect(),
            polyakov: l.polyakov_loops().iter().map(|p| [p.re as f32, p.im as f32]).collect(),
        }
    }
}

impl Record {
    pub fn volume3(&self) -> usize {
        self.dims[0] * self.dims[1] * self.dims[2]
    }
    /// The file's bytes. Lossy: 16 bits a value.
    pub fn encode(&self) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(b"HG4F");
        b.extend_from_slice(&1u16.to_le_bytes());
        for d in self.dims {
            b.extend_from_slice(&(d as u16).to_le_bytes());
        }
        b.extend_from_slice(&(self.tau as u16).to_le_bytes());
        b.extend_from_slice(&(self.snapshots.len() as u16).to_le_bytes());
        b.extend_from_slice(&self.beta.to_le_bytes());
        b.extend_from_slice(&(self.provenance.len() as u32).to_le_bytes());
        b.extend_from_slice(self.provenance.as_bytes());
        for s in &self.snapshots {
            assert_eq!(s.action.len(), self.volume3());
            let amax = s.action.iter().fold(0.0f32, |m, x| m.max(*x)).max(f32::MIN_POSITIVE);
            let qmax = s.charge.iter().fold(0.0f32, |m, x| m.max(x.abs())).max(f32::MIN_POSITIVE);
            b.extend_from_slice(&s.t.to_le_bytes());
            b.extend_from_slice(&s.q_total.to_le_bytes());
            b.extend_from_slice(&s.e_mean.to_le_bytes());
            b.extend_from_slice(&amax.to_le_bytes());
            b.extend_from_slice(&qmax.to_le_bytes());
            for x in &s.action {
                b.extend_from_slice(&((x.max(0.0) / amax * 65535.0).round() as u16).to_le_bytes());
            }
            for x in &s.charge {
                b.extend_from_slice(&((x / qmax * 32767.0).round() as i16).to_le_bytes());
            }
            for p in &s.polyakov {
                for x in p {
                    b.extend_from_slice(&((x.clamp(-1.0, 1.0) * 32767.0).round() as i16).to_le_bytes());
                }
            }
        }
        b
    }
    /// Read a file's bytes back. `None` if they are not a record of this version and length.
    pub fn decode(b: &[u8]) -> Option<Record> {
        let mut at = 0usize;
        let mut take = |n: usize| -> Option<&[u8]> {
            let s = b.get(at..at + n)?;
            at += n;
            Some(s)
        };
        let u16_ = |s: &[u8]| u16::from_le_bytes([s[0], s[1]]);
        let f64_ = |s: &[u8]| f64::from_le_bytes(s.try_into().unwrap());
        let f32_ = |s: &[u8]| f32::from_le_bytes(s.try_into().unwrap());
        if take(4)? != b"HG4F" || u16_(take(2)?) != 1 {
            return None;
        }
        let mut dims = [0usize; 4];
        for d in dims.iter_mut() {
            *d = u16_(take(2)?) as usize;
        }
        let tau = u16_(take(2)?) as usize;
        let n = u16_(take(2)?) as usize;
        let beta = f64_(take(8)?);
        let plen = u32::from_le_bytes(take(4)?.try_into().unwrap()) as usize;
        let provenance = String::from_utf8(take(plen)?.to_vec()).ok()?;
        let v3 = dims[0] * dims[1] * dims[2];
        let mut snapshots = Vec::with_capacity(n);
        for _ in 0..n {
            let (t, q_total, e_mean) = (f64_(take(8)?), f64_(take(8)?), f64_(take(8)?));
            let (amax, qmax) = (f32_(take(4)?), f32_(take(4)?));
            let action = take(2 * v3)?.chunks_exact(2).map(|c| u16_(c) as f32 / 65535.0 * amax).collect();
            let charge = take(2 * v3)?.chunks_exact(2).map(|c| u16_(c) as i16 as f32 / 32767.0 * qmax).collect();
            let polyakov = take(4 * v3)?.chunks_exact(4).map(|c| [u16_(&c[..2]) as i16 as f32 / 32767.0, u16_(&c[2..]) as i16 as f32 / 32767.0]).collect();
            snapshots.push(Snapshot { t, q_total, e_mean, action, charge, polyakov });
        }
        (at == b.len()).then_some(Record { dims, beta, tau, provenance, snapshots })
    }
    /// The record as a reader of the file sees it: every field through its 16 bits.
    pub fn quantised(&self) -> Record {
        Record::decode(&self.encode()).expect("a record decodes its own bytes")
    }
    /// Flow a copy of `l` through `times` (increasing, each a whole number of steps of at
    /// most `eps` from the last) and measure a snapshot at each.
    pub fn flowed(l: &Lattice, beta: f64, tau: usize, times: &[f64], eps: f64, provenance: String) -> Record {
        let mut w = l.clone();
        let mut now = 0.0;
        let mut snapshots = Vec::with_capacity(times.len());
        for &t in times {
            assert!(t >= now);
            let steps = ((t - now) / eps - 1e-9).ceil().max(0.0) as usize;
            if steps > 0 {
                w.flow(t - now, steps);
            }
            now = t;
            snapshots.push(Snapshot::measure(&w, t, tau));
        }
        Record { dims: l.dims, beta, tau, provenance, snapshots }
    }
}
