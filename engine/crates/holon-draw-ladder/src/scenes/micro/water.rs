//! Liquid water as molecules: the surface of the drop, a slab at liquid density.
use super::*;
use holon_draw::ladder::Status;

/// Ångströms.
pub const UNIT: i32 = -10;
/// The rigid water molecule this repository's fluid was measured on: O-H in Å, H-O-H in degrees.
pub const OH: f32 = 0.9572;
pub const HOH_DEG: f32 = 104.52;
/// Molecules per cubic Ångström in the liquid (33.4 per nm³).
pub const DENSITY: f64 = 0.0334;
/// Radii the atoms are DRAWN at (ball and stick), in Å.
pub const R_O: f32 = 0.50;
pub const R_H: f32 = 0.30;
const R_BOND: f32 = 0.11;
/// Sites along x and y, and layers down.
const SIDE: i32 = 19;
const LAYERS: i32 = 3;
/// The nearest molecules move; the rest are still.
const MOVING: usize = 300;
/// Molecules are dealt into tiles by distance from the point the zoom goes into (and the
/// still ones by quadrant too), so that the allocator can draw each tile's balls on the
/// backend's full mesh or its cheap one by what a pixel resolves THERE (the core prices a
/// ball on the cheap mesh at its outline's shortfall, 4.9% of its size). These are the ranks
/// at which a new ring of tiles starts.
const RINGS: [usize; 6] = [0, 60, 150, MOVING, 500, 750];
const C_O: [u8; 4] = [204, 58, 50, 255];
const C_H: [u8; 4] = [236, 236, 232, 255];
const C_BOND: [u8; 4] = [150, 150, 146, 255];

struct Mol {
    o: [f32; 3],
    h1: [f32; 3],
    h2: [f32; 3],
    axis: [f32; 3],
    /// Libration amplitude (radians), its rate (radians a second) and phase.
    amp: f32,
    rate: f64,
    phase: f64,
    /// How far the molecule wanders (Å).
    wander: f32,
}

pub struct Water {
    mols: Vec<Mol>,
    floor: Tile,
    /// The still molecules' tiles, built once.
    still: Vec<Tile>,
    /// The moving molecules' tiles: the ranks in each.
    moving: Vec<std::ops::Range<usize>>,
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
/// `v` turned about the unit vector `k` by the angle whose sine and cosine are given.
fn turn(v: [f32; 3], k: [f32; 3], s: f32, c: f32) -> [f32; 3] {
    let kv = cross(k, v);
    let kd = dot(k, v) * (1.0 - c);
    [v[0] * c + kv[0] * s + k[0] * kd, v[1] * c + kv[1] * s + k[1] * kd, v[2] * c + kv[2] * s + k[2] * kd]
}

impl Water {
    /// The lattice spacing that gives the liquid's density (Å).
    pub fn spacing() -> f64 {
        (1.0 / DENSITY).cbrt()
    }
    pub fn molecules(&self) -> usize {
        self.mols.len()
    }

    pub fn new() -> Water {
        let mut r = Rng::new(0x77_6174_6572);
        let a = Self::spacing() as f32;
        // one molecule per site of a cubic lattice at the liquid's density, each thrown off
        // its site, then pushed apart from any neighbour closer than 2.65 Å (three passes)
        let n = (SIDE * SIDE * LAYERS) as usize;
        let at = |i: i32, j: i32, k: i32| ((k * SIDE + j) * SIDE + i) as usize;
        let mut p: Vec<[f32; 3]> = Vec::with_capacity(n);
        for k in 0..LAYERS {
            for j in 0..SIDE {
                for i in 0..SIDE {
                    let h = 0.5 * (SIDE - 1) as f32;
                    p.push([(i as f32 - h + 0.3 * r.s()) * a, (j as f32 - h + 0.3 * r.s()) * a, -(k as f32 + 0.3 * r.s()) * a]);
                }
            }
        }
        let near = [(1, 0, 0), (0, 1, 0), (0, 0, 1), (1, 1, 0), (1, -1, 0), (1, 0, 1), (1, 0, -1), (0, 1, 1), (0, 1, -1), (1, 1, 1), (1, 1, -1), (1, -1, 1), (1, -1, -1)];
        for _ in 0..3 {
            for k in 0..LAYERS {
                for j in 0..SIDE {
                    for i in 0..SIDE {
                        for (di, dj, dk) in near {
                            let (i2, j2, k2) = (i + di, j + dj, k + dk);
                            if i2 >= SIDE || !(0..SIDE).contains(&j2) || !(0..LAYERS).contains(&k2) {
                                continue;
                            }
                            let (u, v) = (at(i, j, k), at(i2, j2, k2));
                            let d = [p[v][0] - p[u][0], p[v][1] - p[u][1], p[v][2] - p[u][2]];
                            let l = dot(d, d).sqrt();
                            if l < 2.65 && l > 1e-3 {
                                let f = 0.5 * (2.65 - l) / l;
                                for c in 0..3 {
                                    p[u][c] -= f * d[c];
                                    p[v][c] += f * d[c];
                                }
                            }
                        }
                    }
                }
            }
        }
        let half = (0.5 * HOH_DEG).to_radians();
        let (lh1, lh2) = ([OH * half.sin(), 0.0, OH * half.cos()], [-OH * half.sin(), 0.0, OH * half.cos()]);
        let mut mols: Vec<Mol> = p
            .iter()
            .map(|&o| {
                // a random orientation: turn the molecule's own frame about a random axis
                let (k, ang) = (r.dir(), r.f() * std::f32::consts::TAU);
                let (s, c) = ang.sin_cos();
                // and about a second, so the pair is uniform enough over orientations to look it
                let (k2, ang2) = (r.dir(), r.f() * std::f32::consts::TAU);
                let (s2, c2) = ang2.sin_cos();
                Mol { o, h1: turn(turn(lh1, k, s, c), k2, s2, c2), h2: turn(turn(lh2, k, s, c), k2, s2, c2), axis: r.dir(), amp: r.range(0.15, 0.45), rate: r.range(0.5, 1.4) as f64, phase: r.f() as f64 * std::f64::consts::TAU, wander: r.range(0.1, 0.28) }
            })
            .collect();
        // The molecule the zoom goes into: the one at the middle of the top layer, turned so
        // that one O-H points straight up out of the surface, with that hydrogen AT the origin.
        let c = at(SIDE / 2, SIDE / 2, 0);
        mols.swap(0, c);
        mols[0].h1 = [0.0, 0.0, OH];
        let t = HOH_DEG.to_radians();
        mols[0].h2 = [OH * t.sin(), 0.0, OH * t.cos()];
        mols[0].amp = 0.0;
        mols[0].wander = 0.0;
        let shift = add(mols[0].o, mols[0].h1);
        for m in mols.iter_mut() {
            m.o = [m.o[0] - shift[0], m.o[1] - shift[1], m.o[2] - shift[2]];
        }
        mols[0].o = [0.0, 0.0, -OH];
        // nearest first: the first MOVING move, the rest are built once
        mols[1..].sort_by(|x, y| dot(x.o, x.o).total_cmp(&dot(y.o, y.o)));
        let floor = tile(3, 0, vec![disc([0.0, 0.0, -(LAYERS as f32 + 0.3) * a - shift[2]], 6000.0, shade(WATER, 1.05))]);
        // the tiles: rings of rank; beyond the moving set, each ring in four quadrants
        let mut moving = Vec::new();
        let mut still = Vec::new();
        for (k, &from) in RINGS.iter().enumerate() {
            let to = RINGS.get(k + 1).copied().unwrap_or(n).min(n);
            if from < MOVING {
                moving.push(from..to);
                continue;
            }
            for q in 0..4 {
                let members: Vec<usize> = (from..to).filter(|&i| (mols[i].o[0] >= 0.0) as usize + 2 * (mols[i].o[1] >= 0.0) as usize == q).collect();
                let (mut balls, mut sticks) = (Vec::with_capacity(3 * members.len()), Vec::with_capacity(2 * members.len()));
                for &i in &members {
                    let m = &mols[i];
                    molecule(&mut balls, &mut sticks, m.o, add(m.o, m.h1), add(m.o, m.h2));
                }
                still.push(tile(100 + 4 * k as u64 + q as u64, 1, balls));
                still.push(tile(200 + 4 * k as u64 + q as u64, 1, sticks));
            }
        }
        Water { mols, floor, still, moving }
    }

    /// Molecule `i`'s oxygen and two hydrogens at `clock`.
    fn pose(&self, i: usize, clock: f64) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let m = &self.mols[i];
        if i >= MOVING || (m.amp == 0.0 && m.wander == 0.0) {
            return (m.o, add(m.o, m.h1), add(m.o, m.h2));
        }
        // the motion dies away toward the edge of the moving set, so there is no seam
        let taper = 1.0 - (i as f32 / MOVING as f32).powi(2);
        let ph = m.rate * clock + m.phase;
        let (s, c) = ((m.amp * taper) * (ph.sin() as f32)).sin_cos();
        let w = m.wander * taper;
        let o = [m.o[0] + w * ((0.83 * ph).sin() as f32), m.o[1] + w * ((0.71 * ph + 1.3).sin() as f32), m.o[2] + w * ((0.59 * ph + 2.1).sin() as f32)];
        (o, add(o, turn(m.h1, m.axis, s, c)), add(o, turn(m.h2, m.axis, s, c)))
    }
}

impl Default for Water {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for Water {
    fn name(&self) -> &'static str {
        "liquid water, molecule by molecule"
    }
    fn unit(&self) -> i32 {
        UNIT
    }
    fn status(&self) -> Status {
        Status::Illustration
    }
    fn source(&self) -> &'static str {
        "A picture of the model this repository MEASURED its water on (TIERS.md, the molecule and fluid rows; MOLSEARCH1_RESULTS.md): rigid molecules, O-H 0.9572 Å, H-O-H 104.52°, at the liquid's 33.4 molecules per nm³, at the drop's surface. The positions, orientations and slow motion are drawn from a seed, NOT read from the measured trajectories; atoms are drawn as balls of 0.5 and 0.3 Å (on a coarser mesh where a pixel cannot tell), and the surface beyond the 6 nm patch as the drop rung draws it."
    }
    fn span(&self) -> (f64, f64) {
        (-9.0 - VIEW_DECADES, -7.5 - VIEW_DECADES)
    }
    fn tiles(&mut self, z: f64, clock: f64, _alpha: f64) -> Vec<Tile> {
        // a stick is 0.22 Å across: drawn while a pixel resolves it
        let sticks = 2.0e-10 * R_BOND as f64 / 10f64.powf(z) >= 1.0;
        let z = z + VIEW_DECADES; // the view's height, as the rest of this function has it
        let mut out = vec![self.floor.clone()];
        for (k, ranks) in self.moving.iter().enumerate() {
            let (mut balls, mut rods) = (Vec::with_capacity(ranks.len() * 3), Vec::with_capacity(ranks.len() * 2));
            for i in ranks.clone() {
                let (o, h1, h2) = self.pose(i, clock);
                molecule(&mut balls, &mut rods, o, h1, h2);
            }
            out.push(tile(10 + k as u64, 0, balls));
            if sticks {
                out.push(tile(20 + k as u64, 0, rods));
            }
        }
        // the far molecules come in as a pixel passes 0.063 Å, where their balls are small
        // enough for the cheap mesh (an oxygen's shortfall is under a pixel from 0.049 Å);
        // closer, they are beyond the horizon of the near ones
        let far = ramp(z, -8.2, -8.05);
        if far > 0.0 {
            for t in self.still.iter().filter(|t| sticks || t.id < 200) {
                out.push(faded(t.clone(), far));
            }
        }
        out
    }
}

/// One molecule's marks: three balls into one tile's list and two sticks into another's.
/// They are in separate tiles because the core prices a rod on the cheap mesh by its LENGTH
/// (7.6% of the bond, not of the stick's width), which would hold a tile of balls on the full
/// mesh half a decade longer than the balls need.
fn molecule(balls: &mut Vec<Instance>, sticks: &mut Vec<Instance>, o: [f32; 3], h1: [f32; 3], h2: [f32; 3]) {
    balls.push(sphere(o, R_O, C_O));
    balls.push(sphere(h1, R_H, C_H));
    balls.push(sphere(h2, R_H, C_H));
    sticks.push(rod(o, h1, R_BOND, C_BOND));
    sticks.push(rod(o, h2, R_BOND, C_BOND));
}

#[cfg(test)]
pub(super) fn geometry(w: &Water, i: usize, clock: f64) -> (f32, f32, f32) {
    let (o, h1, h2) = w.pose(i, clock);
    let (a, b) = ([h1[0] - o[0], h1[1] - o[1], h1[2] - o[2]], [h2[0] - o[0], h2[1] - o[1], h2[2] - o[2]]);
    let (la, lb) = (dot(a, a).sqrt(), dot(b, b).sqrt());
    (la, lb, (dot(a, b) / (la * lb)).acos().to_degrees())
}
#[cfg(test)]
pub(super) fn oxygen(w: &Water, i: usize) -> [f32; 3] {
    w.mols[i].o
}
