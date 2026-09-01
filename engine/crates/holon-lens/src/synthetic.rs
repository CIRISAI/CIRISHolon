//! SYNTHETIC TRAJECTORIES: the substrate the plants are planted in.
//!
//! A plant is only worth trusting if it is known to FIRE on the instrument it is aimed at
//! (M-PLANT-OBS: observability is instrument-relative). These builders exist so a plant
//! can be constructed with its answer known by hand — a block that is held in every
//! frame, a block that dissociates at a stated frame, a lattice that is a lattice — and
//! the instrument's verdict compared against the construction rather than against a
//! belief about the physics.
//!
//! Nothing here is physics. These are fixtures, and they are never used as evidence about
//! water; they are used as evidence about the instrument.

use crate::partition::Mask;
use crate::traj::{pair_index, Frame, Header, Trajectory};

#[derive(Clone, Debug)]
pub struct Spec {
    pub seed: u64,
    pub n_atoms: usize,
    pub dims: u32,
    pub n_frames: usize,
    pub dt: f64,
    pub substeps: u32,
    pub box_w: f64,
    pub box_h: f64,
    pub box_d: f64,
    pub z: Vec<u32>,
}

impl Spec {
    /// A twelve-atom, two-dimensional box on the quench protocol's own numbers, so a
    /// synthetic trajectory converts frames to femtoseconds exactly as a real one does.
    pub fn quench_like(n_frames: usize, z: Vec<u32>) -> Self {
        Self {
            seed: 0,
            n_atoms: z.len(),
            dims: 2,
            n_frames,
            dt: 0.5386,
            substeps: 64,
            box_w: 34.6,
            box_h: 20.8,
            box_d: 24.0,
            z,
        }
    }
}

/// Deterministic jitter. The same generator shape `waterquench` uses, so a synthetic
/// scene's randomness is a function of a stated seed and nothing else.
pub struct Lcg(pub u64);

impl Lcg {
    pub fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 11) as f64) / ((1u64 << 53) as f64)
    }
    /// Uniform in `[-a, a]`.
    pub fn jitter(&mut self, a: f64) -> f64 {
        a * (2.0 * self.next() - 1.0)
    }
}

/// Every intra-block pair bonded, and nothing else. This is how a PARTITION is planted:
/// the census reads components, so a fixture states the components it wants and this
/// turns them into the edge bits the format carries.
pub fn bonds_from_blocks(n: usize, blocks: &[Mask]) -> u128 {
    let mut bits = 0u128;
    for &b in blocks {
        for i in 0..n {
            if b >> i & 1 == 0 {
                continue;
            }
            for j in (i + 1)..n {
                if b >> j & 1 == 1 {
                    bits |= 1u128 << pair_index(n, i, j);
                }
            }
        }
    }
    bits
}

/// Build a trajectory frame by frame. The closure fills positions and velocities and
/// returns the frame's bond bits.
pub fn build<F>(spec: Spec, mut f: F) -> Trajectory
where
    F: FnMut(usize, &mut [[f64; 3]], &mut [[f64; 3]]) -> u128,
{
    let header = Header {
        seed: spec.seed,
        n_atoms: spec.n_atoms,
        dims: spec.dims,
        substeps: spec.substeps,
        n_frames: spec.n_frames,
        dt: spec.dt,
        box_w: spec.box_w,
        box_h: spec.box_h,
        box_d: spec.box_d,
        z: spec.z.clone(),
    };
    let frame_fs = header.frame_fs();
    let mut frames = Vec::with_capacity(spec.n_frames);
    let mut pos = vec![[0.0f64; 3]; spec.n_atoms];
    let mut vel = vec![[0.0f64; 3]; spec.n_atoms];
    for t in 0..spec.n_frames {
        let bonded = f(t, &mut pos, &mut vel);
        frames.push(Frame {
            index: t as u64,
            time: t as f64 * frame_fs,
            temperature: 300.0,
            bonded,
            pos: pos.clone(),
            vel: vel.clone(),
        });
    }
    Trajectory { header, frames }
}

/// A molecule of `block` vibrating about a fixed geometry, with every other atom free.
///
/// `held(t)` decides whether the block is a component at frame `t`; when it is not, the
/// block's atoms are all free. This one builder covers plants C-1, C-2 and C-3: a block
/// held always, a block that dissociates, and a block with a breach run of stated length.
pub fn vibrating_block<H>(spec: Spec, block: Mask, amp: f64, mut held: H) -> Trajectory
where
    H: FnMut(usize) -> bool,
{
    let n = spec.n_atoms;
    let members: Vec<usize> = (0..n).filter(|i| block >> i & 1 == 1).collect();
    let mut rng = Lcg(spec.seed ^ 0x9E37_79B9_7F4A_7C15);
    // A fixed reference geometry: members on a small ring, everyone else spread out.
    let base: Vec<[f64; 3]> = (0..n)
        .map(|i| match members.iter().position(|&m| m == i) {
            Some(k) => {
                let a = std::f64::consts::TAU * k as f64 / members.len().max(1) as f64;
                [3.0 + 1.8 * a.cos(), 3.0 + 1.8 * a.sin(), 0.0]
            }
            None => [12.0 + 3.0 * (i % 5) as f64, 12.0 + 3.0 * (i / 5) as f64, 0.0],
        })
        .collect();
    let phase: Vec<f64> = (0..n).map(|_| rng.next() * std::f64::consts::TAU).collect();
    build(spec, move |t, pos, vel| {
        for i in 0..n {
            // A real vibration, not a random walk: the carrier must MOVE (the census's
            // G5) while the geometry stays a molecule's geometry.
            let w = 0.7 + 0.13 * (i % 3) as f64;
            let s = (w * t as f64 * 0.05 + phase[i]).sin();
            pos[i] = [base[i][0] + amp * s, base[i][1] + amp * 0.6 * s, 0.0];
            vel[i] = [amp * w * 0.05 * s.cos(), 0.0, 0.0];
        }
        if held(t) {
            bonds_from_blocks(n, &[block])
        } else {
            0
        }
    })
}

/// A frozen block: held in every frame, with NOTHING moving. Plant C-6's carrier — the
/// vacuity G5 exists to catch.
pub fn frozen_block(spec: Spec, block: Mask) -> Trajectory {
    let n = spec.n_atoms;
    let members: Vec<usize> = (0..n).filter(|i| block >> i & 1 == 1).collect();
    let base: Vec<[f64; 3]> = (0..n)
        .map(|i| match members.iter().position(|&m| m == i) {
            Some(k) => {
                let a = std::f64::consts::TAU * k as f64 / members.len().max(1) as f64;
                [3.0 + 1.8 * a.cos(), 3.0 + 1.8 * a.sin(), 0.0]
            }
            None => [12.0 + 3.0 * (i % 5) as f64, 12.0 + 3.0 * (i / 5) as f64, 0.0],
        })
        .collect();
    build(spec, move |_t, pos, vel| {
        pos.copy_from_slice(&base);
        for v in vel.iter_mut() {
            *v = [0.0; 3];
        }
        bonds_from_blocks(n, &[block])
    })
}

// ------------------------------------------------------------------ phase fixtures

/// A dilute gas: every atom far from every other, no bonds at any frame.
pub fn vapor(spec: Spec) -> Trajectory {
    let n = spec.n_atoms;
    let (w, h) = (spec.box_w, spec.box_h);
    let mut rng = Lcg(spec.seed ^ 0xD1B5_4A32_D192_ED03);
    let start: Vec<[f64; 3]> = (0..n)
        .map(|_| [rng.next() * w, rng.next() * h, 0.0])
        .collect();
    let vv: Vec<[f64; 3]> = (0..n)
        .map(|_| [rng.jitter(0.02), rng.jitter(0.02), 0.0])
        .collect();
    build(spec, move |t, pos, vel| {
        for i in 0..n {
            // Reflect off the walls so the scene stays a gas in a box.
            let mut x = start[i][0] + vv[i][0] * t as f64;
            let mut y = start[i][1] + vv[i][1] * t as f64;
            x = fold(x, w);
            y = fold(y, h);
            pos[i] = [x, y, 0.0];
            vel[i] = vv[i];
        }
        0
    })
}

fn fold(v: f64, l: f64) -> f64 {
    let p = 2.0 * l;
    let m = v.rem_euclid(p);
    if m <= l {
        m
    } else {
        p - m
    }
}

/// A triangular lattice with small thermal jitter and every neighbour pair bonded: the
/// scene an "ice" label is supposed to describe.
pub fn crystal(spec: Spec, amp: f64) -> Trajectory {
    let n = spec.n_atoms;
    let a = 2.6f64;
    let cols = 4usize;
    let base: Vec<[f64; 3]> = (0..n)
        .map(|i| {
            let (c, r) = (i % cols, i / cols);
            [
                4.0 + a * (c as f64 + 0.5 * (r % 2) as f64),
                4.0 + a * 0.8660254037844386 * r as f64,
                0.0,
            ]
        })
        .collect();
    let mut rng = Lcg(spec.seed ^ 0x2545_F491_4F6C_DD1D);
    let phase: Vec<f64> = (0..n).map(|_| rng.next() * std::f64::consts::TAU).collect();
    let all: Mask = if n >= 16 { Mask::MAX } else { ((1u32 << n) - 1) as Mask };
    build(spec, move |t, pos, vel| {
        for i in 0..n {
            let s = (0.9 * t as f64 * 0.05 + phase[i]).sin();
            pos[i] = [base[i][0] + amp * s, base[i][1] - amp * s, 0.0];
            vel[i] = [amp * 0.045 * s.cos(), 0.0, 0.0];
        }
        bonds_from_blocks(n, &[all])
    })
}

/// A bonded but MOBILE scene: one component, atoms exchanging places, displacement
/// growing with time. The scene a "liquid" label describes.
pub fn liquid(spec: Spec) -> Trajectory {
    let n = spec.n_atoms;
    let (w, h) = (spec.box_w, spec.box_h);
    let mut rng = Lcg(spec.seed ^ 0x1234_5678_9ABC_DEF0);
    let mut cur: Vec<[f64; 3]> = (0..n)
        .map(|_| [6.0 + rng.next() * 8.0, 6.0 + rng.next() * 6.0, 0.0])
        .collect();
    let all: Mask = if n >= 16 { Mask::MAX } else { ((1u32 << n) - 1) as Mask };
    build(spec, move |_t, pos, vel| {
        for i in 0..n {
            let dx = rng.jitter(0.12);
            let dy = rng.jitter(0.12);
            cur[i][0] = fold(cur[i][0] + dx, w);
            cur[i][1] = fold(cur[i][1] + dy, h);
            pos[i] = cur[i];
            vel[i] = [dx, dy, 0.0];
        }
        bonds_from_blocks(n, &[all])
    })
}
