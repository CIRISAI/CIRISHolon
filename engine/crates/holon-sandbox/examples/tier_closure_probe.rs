//! TIER CLOSURE — is a coarse (tier) view of this engine CLOSED?
//!
//! A tier view is Closed if its readings are a function of the tier's own variables:
//! two micro-states that a coarse view cannot tell apart must stay indistinguishable to
//! that view as the scene evolves. This instrument builds exactly that test.
//!
//! Twin sessions, MACRO-MATCHED and MICRO-DIFFERENT. Session B is session A with a set
//! of VELOCITY SWAPS applied between grains that are (a) in the same coarse cell and
//! (b) of EXACTLY equal mass (bitwise `f64`). Such a swap is a permutation of identical
//! summands, so every declared per-cell aggregate — total mass, both momentum
//! components, kinetic energy, mass-weighted mean x — is invariant by construction, up
//! to the order in which the floating-point sum is accumulated. The two scenes are the
//! same scene to the coarse view and different scenes underneath.
//!
//! Then both are evolved with NO further intervention: the micro-difference IS the
//! intervention. Per frame we emit
//!   * `micro_div` — the full-resolution divergence, id-joined (see `idjoin_probe.rs`:
//!     `nodes.holon` is stable because the arena is append-only);
//!   * four DECLARED coarse-view divergences, one per tier aggregate;
//!   * 64 RANDOM coarse-view divergences — random unit directions in the space of
//!     per-cell SYMMETRIC velocity moments (see `NMOM`), standardized by frozen
//!     frame-0 scene scales.
//!
//! The random views are the FAIR control on the declared ones. An earlier draft used
//! per-grain random coefficients; those are not permutation invariant, so they read the
//! swap itself at frame 0 while the declared views start at ULP-zero — the two classes
//! had incomparable geometry and the comparison would have convicted the declared views
//! by construction. Every view is now a symmetric function of the per-cell velocity
//! multiset: all are exactly blind to the swap at frame 0, all can only climb, and the
//! declared aggregates are simply three particular directions (the conserved-quantity
//! ones) in the same standardized moment space the random directions sample.
//!
//! A SHAM pair (A against an identically built A', no swaps) is emitted alongside. Every
//! column of it must be exactly 0: that is the zero baseline, and it is what separates a
//! reading about the SCENE from a reading about the instrument.
//!
//! ---------------------------------------------------------------------------------
//! WARM-UP, AND WHY THIS INSTRUMENT HAS ONE
//!
//! A freshly built session has EVERY node velocity exactly `[0.0, 0.0]` and an awake set
//! of size ZERO — `Session::new` settles the scene, and `throw` adds a projectile, not
//! grain motion. Swapping velocities "before any stepping" is therefore a no-op on
//! 117,760 free grains: it would pass a naive pair count with ~54,000 swaps and leave
//! the two sessions bit-identical, so the instrument would report perfect closure while
//! measuring nothing. Worse, a velocity written onto an ASLEEP cell is never integrated
//! (only `awake_list` is stepped) and the sleep pass zeroes it, so such a write shows up
//! in the coarse readout as a difference the dynamics never realizes — a phantom.
//!
//! So swaps are restricted to grains that are FREE and AWAKE, and `--warmup` frames are
//! stepped IDENTICALLY on both sessions before the swap. Warm-up frames are not part of
//! the measurement; frame 0 of the CSV is the first frame after the swap.
//!
//! `warmup = 0` is the default, and it preserves the literal specification: with no
//! warm-up there is no awake set, the effective-pair count is 0, and the run refuses
//! with `INSUFFICIENT PAIRS`. That refusal is the honest reading of the literal design,
//! not a failure of the code.
//!
//! Usage: `tier_closure_probe <out_dir> <frames> [warmup]`

use holon_sandbox::sim::Session;
use holon_sandbox::tier::TierId;
use std::collections::HashMap;
use std::fmt::Write as _;

const DT: f64 = 1.0 / 60.0;
/// Coarse grid: 8 columns in x by 4 rows in y over the frame-0 free-grain bounding box.
const NX: usize = 8;
const NY: usize = 4;
const CELLS: usize = NX * NY;
/// Random control views.
const VIEWS: usize = 64;
/// Below this many EFFECTIVE (velocity-changing) swaps the instrument refuses.
const MIN_PAIRS: usize = 100;

/// Symmetric per-cell velocity moments, all mass-weighted:
/// m·vx, m·vy, ½m|v|², m·vx², m·vy², m·vx·vy, m|v|³, m·vx³, m·vy³.
const NMOM: usize = 9;

// ---------------------------------------------------------------------------------
// Random-view directions: splitmix64 keyed by (view, moment), each row normalized to a
// unit vector in standardized moment space. Global — no per-grain state.

fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A `u64` to a double in [-1, 1), using the 53 bits a `f64` can hold exactly.
fn unit(h: u64) -> f64 {
    ((h >> 11) as f64) / ((1u64 << 53) as f64) * 2.0 - 1.0
}

/// The 64 random unit directions, generated once.
fn combo_table() -> Vec<f64> {
    let mut t = vec![0.0; VIEWS * NMOM];
    for r in 0..VIEWS {
        let mut norm = 0.0;
        for j in 0..NMOM {
            let key = (r as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ (j as u64).wrapping_mul(0xD1B5_4A32_D192_ED03);
            let c = unit(splitmix64(key));
            t[r * NMOM + j] = c;
            norm += c * c;
        }
        let inv = 1.0 / norm.sqrt();
        for j in 0..NMOM {
            t[r * NMOM + j] *= inv;
        }
    }
    t
}

// ---------------------------------------------------------------------------------
// The coarse grid.

/// The frame-0 partition: a fixed box, never recomputed. Grains are binned by their
/// CURRENT position each frame, and anything that has left the box lands in the clamped
/// edge cell rather than being dropped.
#[derive(Clone, Copy)]
struct Grid {
    x0: f64,
    y0: f64,
    inv_w: f64,
    inv_h: f64,
}

impl Grid {
    fn over(session: &Session) -> Self {
        let n = session.nodes();
        let (mut x0, mut x1) = (f64::MAX, f64::MIN);
        let (mut y0, mut y1) = (f64::MAX, f64::MIN);
        for i in 0..n.holon.len() {
            if n.anchored[i] {
                continue;
            }
            x0 = x0.min(n.position[i][0]);
            x1 = x1.max(n.position[i][0]);
            y0 = y0.min(n.position[i][1]);
            y1 = y1.max(n.position[i][1]);
        }
        // A degenerate extent would put every grain in one column; widen rather than
        // divide by zero, and say so if it ever happens.
        let w = if x1 > x0 { x1 - x0 } else { 1.0 };
        let h = if y1 > y0 { y1 - y0 } else { 1.0 };
        Self { x0, y0, inv_w: NX as f64 / w, inv_h: NY as f64 / h }
    }

    fn cell(&self, p: [f64; 2]) -> usize {
        let cx = (((p[0] - self.x0) * self.inv_w) as isize).clamp(0, NX as isize - 1) as usize;
        let cy = (((p[1] - self.y0) * self.inv_h) as isize).clamp(0, NY as isize - 1) as usize;
        cy * NX + cx
    }
}

// ---------------------------------------------------------------------------------
// One frame's coarse readings.

struct Coarse {
    mass: [f64; CELLS],
    momx: [f64; CELLS],
    momy: [f64; CELLS],
    ke: [f64; CELLS],
    /// Mass-weighted sum of x; divided down to the mean at the end.
    mx: [f64; CELLS],
    comx: [f64; CELLS],
    mom: Vec<f64>,
    /// Aligned (absolute-value) moments, same layout: the denominator of coherence.
    mom_abs: Vec<f64>,
}

impl Coarse {
    fn new() -> Self {
        Self {
            mass: [0.0; CELLS],
            momx: [0.0; CELLS],
            momy: [0.0; CELLS],
            ke: [0.0; CELLS],
            mx: [0.0; CELLS],
            comx: [0.0; CELLS],
            mom: vec![0.0; NMOM * CELLS],
            mom_abs: vec![0.0; NMOM * CELLS],
        }
    }

    /// Read every coarse view off one session. Free grains only: the anchored ring is
    /// boundary, its velocity is clamped to zero every substep, and it is the same set
    /// of grains in both twins.
    ///
    /// Grains at exactly zero velocity are skipped past the velocity-dependent views.
    /// That is not an approximation: their contribution to every one of those sums is
    /// exactly `+0.0`, so the skip is bitwise identical to including them, and it turns
    /// a 118k-grain inner loop over 64 views into one over the few thousand awake.
    fn read(&mut self, session: &Session, grid: &Grid) {
        self.mass = [0.0; CELLS];
        self.momx = [0.0; CELLS];
        self.momy = [0.0; CELLS];
        self.ke = [0.0; CELLS];
        self.mx = [0.0; CELLS];
        self.mom.iter_mut().for_each(|v| *v = 0.0);
        self.mom_abs.iter_mut().for_each(|v| *v = 0.0);

        let n = session.nodes();
        for i in 0..n.holon.len() {
            if n.anchored[i] {
                continue;
            }
            let c = grid.cell(n.position[i]);
            let m = n.mass_kg[i];
            self.mass[c] += m;
            self.mx[c] += m * n.position[i][0];
            let v = n.velocity[i];
            if v[0] == 0.0 && v[1] == 0.0 {
                continue;
            }
            self.momx[c] += m * v[0];
            self.momy[c] += m * v[1];
            self.ke[c] += 0.5 * m * (v[0] * v[0] + v[1] * v[1]);
            let (vx, vy) = (v[0], v[1]);
            let v2 = vx * vx + vy * vy;
            let vn = v2.sqrt();
            let mom = [m * vx, m * vy, 0.5 * m * v2, m * vx * vx, m * vy * vy,
                       m * vx * vy, m * v2 * vn, m * vx * vx * vx, m * vy * vy * vy];
            for (j, val) in mom.iter().enumerate() {
                self.mom[j * CELLS + c] += val;
                self.mom_abs[j * CELLS + c] += val.abs();
            }
        }
        for c in 0..CELLS {
            self.comx[c] = if self.mass[c] > 0.0 { self.mx[c] / self.mass[c] } else { 0.0 };
        }
    }
}

/// L2 across cells of a per-cell difference.
fn l2(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum::<f64>().sqrt()
}

// ---------------------------------------------------------------------------------
// The id-join, cached. The arena is append-only, so the index -> holon map is stable and
// the join only has to be rebuilt when either session's resident count changes.

struct Join {
    pairs: Vec<(usize, usize)>,
    len_a: usize,
    len_b: usize,
    only_a: usize,
    only_b: usize,
}

impl Join {
    fn new() -> Self {
        Self { pairs: Vec::new(), len_a: usize::MAX, len_b: usize::MAX, only_a: 0, only_b: 0 }
    }

    fn refresh(&mut self, a: &Session, b: &Session) {
        let (na, nb) = (a.nodes(), b.nodes());
        if na.holon.len() == self.len_a && nb.holon.len() == self.len_b {
            return;
        }
        let mut index_b = HashMap::with_capacity(nb.holon.len());
        for j in 0..nb.holon.len() {
            index_b.insert(nb.holon[j], j);
        }
        self.pairs.clear();
        for i in 0..na.holon.len() {
            if let Some(&j) = index_b.get(&na.holon[i]) {
                self.pairs.push((i, j));
            }
        }
        self.len_a = na.holon.len();
        self.len_b = nb.holon.len();
        self.only_a = self.len_a - self.pairs.len();
        self.only_b = self.len_b - self.pairs.len();
    }
}

/// Full-resolution divergence: an L2 in a length-scaled phase space, velocity carried
/// into position units by one frame's `DT` so the two halves are commensurable.
fn micro_div(a: &Session, b: &Session, join: &Join) -> f64 {
    let (na, nb) = (a.nodes(), b.nodes());
    let mut acc = 0.0;
    for &(i, j) in &join.pairs {
        let dp = [na.position[i][0] - nb.position[j][0], na.position[i][1] - nb.position[j][1]];
        let dv = [na.velocity[i][0] - nb.velocity[j][0], na.velocity[i][1] - nb.velocity[j][1]];
        acc += dp[0] * dp[0] + dp[1] * dp[1] + DT * DT * (dv[0] * dv[0] + dv[1] * dv[1]);
    }
    acc.sqrt()
}

// ---------------------------------------------------------------------------------

/// Apply the macro-preserving micro-perturbation to `b`.
///
/// Within each coarse cell, grains that are free, awake, and of bitwise-identical mass
/// are formed into disjoint pairs and have their velocity vectors exchanged. Returns
/// (pairs formed, pairs that actually changed a velocity).
fn swap_velocities(b: &mut Session, grid: &Grid) -> (usize, usize) {
    // (cell, mass bits) -> the grains waiting for a partner.
    let mut bucket: HashMap<(usize, u64), Vec<usize>> = HashMap::new();
    {
        let n = b.nodes();
        for i in 0..n.holon.len() {
            if n.anchored[i] || !n.awake[i] {
                continue;
            }
            let key = (grid.cell(n.position[i]), n.mass_kg[i].to_bits());
            bucket.entry(key).or_default().push(i);
        }
    }
    // Deterministic order: a HashMap iterates arbitrarily, and the swap set must not
    // depend on the hasher's mood.
    let mut keys: Vec<(usize, u64)> = bucket.keys().copied().collect();
    keys.sort_unstable();

    let (mut formed, mut effective) = (0usize, 0usize);
    let nodes = b.nodes_mut();
    for key in keys {
        let members = &bucket[&key];
        for pair in members.chunks_exact(2) {
            let (i, j) = (pair[0], pair[1]);
            formed += 1;
            let (vi, vj) = (nodes.velocity[i], nodes.velocity[j]);
            if vi[0].to_bits() != vj[0].to_bits() || vi[1].to_bits() != vj[1].to_bits() {
                effective += 1;
            }
            nodes.velocity[i] = vj;
            nodes.velocity[j] = vi;
        }
    }
    (formed, effective)
}

/// Evolve a twin pair and write its CSV. `label` names the file.
fn run_pair(
    out: &str,
    label: &str,
    a: &mut Session,
    b: &mut Session,
    grid: &Grid,
    frames: usize,
) {
    let combos = combo_table();
    let (mut ca, mut cb) = (Coarse::new(), Coarse::new());
    let mut join = Join::new();
    // Frozen per-moment scene scales: RMS across cells of session A's moments at the
    // first measured frame. Written alongside the CSV; every random-view divergence is
    // computed in these standardized units so the 64 directions are commensurable.
    let mut scales = [1.0_f64; NMOM];
    let mut scales_set = false;

    let mut csv = String::from("frame,micro_div,d_momx,d_momy,d_ke,d_comx");
    for r in 0..VIEWS {
        write!(csv, ",rand_{r:02}").unwrap();
    }
    // Join diagnostics, appended after every specified column so a positional reader of
    // the named ones is unaffected.
    csv.push_str(",join,only_a,only_b\n");

    for f in 0..frames {
        a.step(DT);
        b.step(DT);
        join.refresh(a, b);
        ca.read(a, grid);
        cb.read(b, grid);
        if !scales_set {
            for j in 0..NMOM {
                let ms: f64 = ca.mom[j * CELLS..(j + 1) * CELLS].iter().map(|x| x * x).sum();
                let rms = (ms / CELLS as f64).sqrt();
                scales[j] = if rms > 0.0 { rms } else { 1.0 };
            }
            scales_set = true;
            let mut sf = String::new();
            for j in 0..NMOM {
                writeln!(sf, "scale_{j} = {:.9e}", scales[j]).unwrap();
            }
            std::fs::write(format!("{out}/{label}_scales.txt"), sf).unwrap();
        }
        write!(
            csv,
            "{f},{:.9e},{:.9e},{:.9e},{:.9e},{:.9e}",
            micro_div(a, b, &join),
            l2(&ca.momx, &cb.momx),
            l2(&ca.momy, &cb.momy),
            l2(&ca.ke, &cb.ke),
            l2(&ca.comx, &cb.comx),
        )
        .unwrap();
        if f == 0 || f == 300 || f == 1200 {
            let mut co = String::new();
            for j in 0..NMOM {
                let rs: f64 = (ca.mom[j * CELLS..(j + 1) * CELLS].iter().map(|x| x * x).sum::<f64>() / CELLS as f64).sqrt();
                let ra: f64 = (ca.mom_abs[j * CELLS..(j + 1) * CELLS].iter().map(|x| x * x).sum::<f64>() / CELLS as f64).sqrt();
                writeln!(co, "{f},{j},{rs:.9e},{ra:.9e}").unwrap();
            }
            let path = format!("{out}/{label}_coherence.csv");
            let prev = std::fs::read_to_string(&path).unwrap_or_else(|_| "frame,moment,rms_signed,rms_abs\n".into());
            std::fs::write(&path, prev + &co).unwrap();
        }
        for r in 0..VIEWS {
            let mut acc = 0.0;
            for c in 0..CELLS {
                let mut d = 0.0;
                for j in 0..NMOM {
                    d += combos[r * NMOM + j]
                        * (ca.mom[j * CELLS + c] - cb.mom[j * CELLS + c])
                        / scales[j];
                }
                acc += d * d;
            }
            write!(csv, ",{:.9e}", acc.sqrt()).unwrap();
        }
        writeln!(csv, ",{},{},{}", join.pairs.len(), join.only_a, join.only_b).unwrap();
    }
    std::fs::write(format!("{out}/{label}.csv"), csv).unwrap();
    eprintln!("{label}.csv written ({frames} frames)");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args.get(1).cloned().unwrap_or_else(|| "../../../scratchpad/tier_closure".into());
    let frames: usize = args.get(2).map(|s| s.parse().expect("frames")).unwrap_or(2400);
    // 0 preserves the literal specification: swap before any stepping. See the header
    // for why that configuration refuses.
    let warmup: usize = args.get(3).map(|s| s.parse().expect("warmup")).unwrap_or(0);
    std::fs::create_dir_all(&out).unwrap();

    let tier = TierId::Sandbox;

    // Scene construction, exactly as `idjoin_probe.rs` does it: a settled session plus
    // the one throw both twins share. The throw is the scene's initial condition, not
    // the probe — this instrument applies no probe at all.
    let seed = Session::new(tier);
    let xs: Vec<f64> = seed.nodes().position.iter().map(|p| p[0]).collect();
    let xmin = xs.iter().cloned().fold(f64::MAX, f64::min);
    let xmax = xs.iter().cloned().fold(f64::MIN, f64::max);
    let mid = 0.5 * (xmin + xmax);
    drop(seed);

    let build = || {
        let mut s = Session::new(tier);
        s.throw(mid, 0.8, 0.5);
        for _ in 0..warmup {
            s.step(DT);
        }
        s
    };

    let mut a = build();
    let mut b = build();
    let grid = Grid::over(&a);

    let (formed, effective) = swap_velocities(&mut b, &grid);
    eprintln!("swaps: formed={formed} effective={effective} (warmup={warmup} frames)");
    if effective < MIN_PAIRS {
        println!("INSUFFICIENT PAIRS n={effective}");
        eprintln!(
            "refusing: {effective} velocity-changing swaps is under the {MIN_PAIRS} floor. \
             {formed} pairs were formed, so the shortfall is that the paired grains carry \
             the SAME velocity — at warmup=0 every free grain is asleep at exactly zero."
        );
        std::process::exit(2);
    }

    // Frame-0 verification. The swap permutes identical-mass summands, so each declared
    // aggregate is invariant up to summation order; whatever is left is reported as it
    // reads. Mass-weighted mean x touches no velocity and must be exactly 0.
    let (mut ca, mut cb) = (Coarse::new(), Coarse::new());
    ca.read(&a, &grid);
    cb.read(&b, &grid);
    let named: [(&str, &[f64; CELLS], &[f64; CELLS]); 5] = [
        ("sum_m", &ca.mass, &cb.mass),
        ("sum_m_vx", &ca.momx, &cb.momx),
        ("sum_m_vy", &ca.momy, &cb.momy),
        ("sum_ke", &ca.ke, &cb.ke),
        ("mean_x", &ca.comx, &cb.comx),
    ];
    let mut worst = 0.0f64;
    let mut meta = String::new();
    for (name, x, y) in named {
        let d = x.iter().zip(y.iter()).map(|(p, q)| (p - q).abs()).fold(0.0f64, f64::max);
        worst = worst.max(d);
        writeln!(meta, "frame0 max|A-B| {name} = {d:.9e}").unwrap();
    }
    // Random views are symmetric in the per-cell velocity multiset, so the swap is
    // invisible to them too: their frame-0 divergence is the same ULP dust as the
    // declared aggregates'. Verified on the raw (unstandardized) moments.
    let rand0 = (0..NMOM)
        .map(|j| {
            let s = j * CELLS;
            l2(&ca.mom[s..s + CELLS], &cb.mom[s..s + CELLS])
        })
        .fold(0.0f64, f64::max);
    println!("frame0 max declared aggregate diff = {worst:.9e}");
    println!("frame0 max raw-moment L2           = {rand0:.9e}");
    print!("{meta}");
    writeln!(meta, "frame0 max declared aggregate diff = {worst:.9e}").unwrap();
    writeln!(meta, "frame0 max raw-moment L2 = {rand0:.9e}").unwrap();
    writeln!(meta, "swaps formed = {formed}, effective = {effective}").unwrap();
    writeln!(meta, "warmup = {warmup}, frames = {frames}").unwrap();
    writeln!(meta, "grid = {NX}x{NY} over free-grain bbox at the swap frame").unwrap();
    std::fs::write(format!("{out}/meta.txt"), &meta).unwrap();

    run_pair(&out, "swap", &mut a, &mut b, &grid, frames);

    // The sham: A against an identically built A', no swaps. Every column must be 0.
    let mut sa = build();
    let mut sb = build();
    run_pair(&out, "sham", &mut sa, &mut sb, &grid, frames);
}
