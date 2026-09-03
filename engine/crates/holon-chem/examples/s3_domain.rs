//! SATURATION-3 domain derivation: where each trimer surface may be truncated.
//!
//! # The rule, and why it is the same rule as SATURATION-2's
//!
//! AMENDMENT A1 truncates on the SECOND-SMALLEST side, because `dE3` vanishes exactly
//! when some atom is far from BOTH of the others — which is the statement that two of the
//! three sides are long. That reasoning is about geometry, not species, so it transfers to
//! every triple unchanged, and this is `s2_domain`'s sweep made species-general rather
//! than a second transcription of it.
//!
//! The box's two axes are the two sides meeting at the APEX atom — the one the table's
//! symmetry does not exchange — and the third side is set by the angle between them. So
//! `x, y <= R_HI` bounds both apex sides and leaves the opposite side free, and the
//! question this sweep asks is what zeroing the surface at `R_HI` costs: over the whole
//! shell `max(apex side) = b`, how big does `dE3` get?
//!
//! `x_lo` is NOT swept for: it is `pair::derive_range`'s own inner end for the apex pair,
//! the separation at which that pair sits one hartree above its asymptote. Using the
//! crate's existing rule rather than a new one means the table's inner wall and the pair
//! curve's inner wall are the same claim.
//!
//! ```text
//! cargo run --release -p holon-chem --example s3_domain -- <APEX> <B> <C> [threads] [nx] [nc]
//! ```
//!
//! `APEX` is the atom the two axis sides meet at: `Cl H H` for the (H,H,Cl) table,
//! `H Cl Cl` for (H,Cl,Cl), `H O O` for (O,O,H), `Cl Cl Cl` for the S3 table.

use holon_chem::dual::D2;
use holon_chem::elements::{by_symbol, Species};
use holon_chem::pair::{atom_energy, derive_range, pair_point, solve_geometry};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

const SQRT2: f64 = std::f64::consts::SQRT_2;

fn c3(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

/// The located equilibrium of a pair, by golden-free coarse scan then refinement — the
/// same quantity G0 staked its compact geometries against, recomputed here rather than
/// copied, so the domain cites a measurement and not a number in a document.
fn locate_r_e(a: Species, b: Species) -> f64 {
    let (mut lo, mut hi, mut best) = (0.5f64, 12.0f64, (f64::INFINITY, 1.0f64));
    for _ in 0..3 {
        let n = 41;
        best = (f64::INFINITY, lo);
        for i in 0..n {
            let r = lo + (hi - lo) * i as f64 / (n - 1) as f64;
            let e = pair_point(a, b, r).e;
            if e < best.0 {
                best = (e, r);
            }
        }
        let h = (hi - lo) / (n - 1) as f64;
        lo = (best.1 - h).max(0.4);
        hi = best.1 + h;
    }
    best.1
}

struct Ctx {
    apex: Species,
    b1: Species,
    b2: Species,
    e_atoms: f64,
}

impl Ctx {
    /// `dE3` at the triangle with apex sides `x` (to `b1`) and `y` (to `b2`), `u` the
    /// cosine between them. Every pair energy goes through the same route as the triple.
    fn de3(&self, x: f64, y: f64, u: f64) -> (f64, f64) {
        let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();
        let s = (1.0 - u * u).max(0.0).sqrt();
        let e3 = solve_geometry(
            &[self.apex, self.b1, self.b2],
            vec![c3(0.0, 0.0, 0.0), c3(x, 0.0, 0.0), c3(y * u, y * s, 0.0)],
        )
        .e
        .v;
        let d = e3 + self.e_atoms
            - pair_point(self.apex, self.b1, x).e
            - pair_point(self.apex, self.b2, y).e
            - pair_point(self.b1, self.b2, z).e;
        (d, z)
    }
}

#[derive(Clone, Copy, Default)]
struct Worst {
    d: f64,
    x: f64,
    theta: f64,
    z: f64,
    /// The second-smallest of the three sides at the worst point — the quantity A1
    /// truncates on, reported so the box rule can be read against it.
    s2: f64,
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let name = |i: usize| -> Species {
        by_symbol(&a[i]).unwrap_or_else(|| panic!("unknown element {}", a[i]))
    };
    let (apex, b1, b2) = (name(0), name(1), name(2));
    let threads: usize = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(8);
    // this producer owns the machine and runs its own pool: split the cores between
    // the pool and the lane kernel beneath it, or the two multiply (scheduling only)
    holon_chem::lanes::set_lane_threads_for_pool(threads);
    let nx: usize = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(13);
    let nc: usize = a.get(5).and_then(|s| s.parse().ok()).unwrap_or(17);
    // THE ANGLE WINDOW'S CLOSED END, and it is an argument because it turned out to be
    // load-bearing. Every row of the first (Cl,H,H) sweep put its worst |dE3| at
    // theta = 4.05 deg, which is exactly arccos(1 - 0.05^2) — the c = 0.05 endpoint. A
    // maximum attained ON a window edge is evidence the supremum is OUTSIDE the window, so
    // the sweep cannot be read as bracketing anything until this is pushed down and the
    // maximum is shown to be interior. Found by saturation3-mesh reading the column rather
    // than the number.
    let c_min: f64 = a.get(6).and_then(|s| s.parse().ok()).unwrap_or(0.05);
    // Sweep ONE shell, named as a multiple of the apex R_e, when the question is about the
    // angle axis rather than the truncation radius.
    let only: Option<f64> = a.get(7).and_then(|s| s.parse().ok());

    let t0 = Instant::now();
    let e_atoms = atom_energy(apex) + atom_energy(b1) + atom_energy(b2);
    let ctx = Ctx { apex, b1, b2, e_atoms };

    // IDENTIFY BEFORE COMPUTING. Everything below this point does electronic-structure
    // work -- two `derive_range` calls and three `locate_r_e` scans, each of which is 123
    // pair solves -- and for the oxygen tables those include O-O at radii near the spin
    // degeneracy, where a solve costs orders more than the compact one G0 timed. Printing
    // the header AFTER them left the job silent through a startup that is itself minutes
    // long, so a reader could not tell a live run from a wedged one. Adding progress to
    // the sweep loop alone did not fix that: the sweep had not started yet. A job has to be
    // legible from its FIRST second, not from its first result.
    println!("# DOMAIN SWEEP starting: ({}, {}, {}), apex {}, threads = {threads}",
        apex.symbol, b1.symbol, b2.symbol, apex.symbol);
    println!("#   deriving inner ends and equilibria (5 scans, each real solves)...");
    let asym1 = atom_energy(apex) + atom_energy(b1);
    let asym2 = atom_energy(apex) + atom_energy(b2);
    let (x_lo, _) = derive_range(apex, b1, asym1);
    println!("#   x_lo = {x_lo:.4} ({:.0} s)", t0.elapsed().as_secs_f64());
    let (y_lo, _) = derive_range(apex, b2, asym2);
    println!("#   y_lo = {y_lo:.4} ({:.0} s)", t0.elapsed().as_secs_f64());
    let re1 = locate_r_e(apex, b1);
    println!("#   R_e(apex-{}) = {re1:.4} ({:.0} s)", b1.symbol, t0.elapsed().as_secs_f64());
    let re2 = locate_r_e(apex, b2);
    println!("#   R_e(apex-{}) = {re2:.4} ({:.0} s)", b2.symbol, t0.elapsed().as_secs_f64());
    let re_opp = locate_r_e(b1, b2);
    println!("#   R_e({}-{}) = {re_opp:.4} ({:.0} s)", b1.symbol, b2.symbol, t0.elapsed().as_secs_f64());

    println!(
        "# DOMAIN SWEEP for the ({}, {}, {}) table — apex {}, axes {}-{} and {}-{}",
        apex.symbol, b1.symbol, b2.symbol, apex.symbol, apex.symbol, b1.symbol, apex.symbol, b2.symbol
    );
    println!("# threads = {threads}, nx = {nx}, nc = {nc}");
    println!("# E(atoms) = {e_atoms:.12} Ha");
    println!("# located R_e: apex-{} {re1:.4}, apex-{} {re2:.4}, opposite {}-{} {re_opp:.4} bohr",
        b1.symbol, b2.symbol, b1.symbol, b2.symbol);
    println!("# inner ends from pair::derive_range (one hartree above asymptote):");
    println!("#   x_lo = {x_lo:.4}, y_lo = {y_lo:.4} bohr\n");

    // Shells in units of the LARGER apex equilibrium, so the ladder means the same thing
    // for chlorine as for hydrogen instead of being a set of bohr that happens to suit one.
    let re_max = re1.max(re2);
    let mults: Vec<f64> = match only {
        Some(m) => vec![m],
        // RE-SIZED. The ladder stopped at 8x R_e and the last two rungs were the most
        // expensive by far -- cost grows with b because a larger shell is a more
        // DISSOCIATED geometry, and dissociated oxygen solves sit near the spin degeneracy
        // where Davidson grinds. They were also pure confirmation: (Cl,H,H) crossed the
        // 1e-5 truncation stake between 5.0 and 6.0, and 7.0/8.0 only restated a decay
        // already established. Dropping them and spending the saving on half-steps where
        // the crossing actually lives is strictly more information for strictly less time.
        None => vec![1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.5],
    };
    let bs: Vec<f64> = mults.iter().map(|m| m * re_max).collect();
    println!("## truncation shell: worst |dE3| anywhere with max(apex side) = b");
    println!("##   x swept {nx} ways over [{x_lo:.3}, b], angle swept {nc} ways over");
    println!("##   c = sqrt(1-cos theta) in [{c_min}, {SQRT2:.4}]  (theta from {:.3} deg)\n",
        (1.0f64 - c_min * c_min).clamp(-1.0, 1.0).acos().to_degrees());

    let jobs: Vec<(usize, usize)> = (0..bs.len())
        .flat_map(|i| (0..nx).map(move |j| (i, j)))
        .collect();
    let next = AtomicUsize::new(0);
    let progress = AtomicUsize::new(0);
    let acc: Vec<Mutex<Worst>> = bs.iter().map(|_| Mutex::new(Worst::default())).collect();
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let t = next.fetch_add(1, Ordering::SeqCst);
                if t >= jobs.len() {
                    break;
                }
                let (i, j) = jobs[t];
                let b = bs[i];
                let x = x_lo + (b - x_lo) * j as f64 / (nx - 1) as f64;
                let mut local = Worst::default();
                for k in 0..nc {
                    let cc = c_min + (SQRT2 - c_min) * k as f64 / (nc - 1) as f64;
                    let u: f64 = 1.0 - cc * cc;
                    let (d, z) = ctx.de3(x, b, u);
                    if d.abs() > local.d.abs() {
                        let mut s = [x, b, z];
                        s.sort_by(|p, q| p.partial_cmp(q).unwrap());
                        local = Worst {
                            d,
                            x,
                            theta: u.clamp(-1.0, 1.0).acos().to_degrees(),
                            z,
                            s2: s[1],
                        };
                    }
                }
                let mut slot = acc[i].lock().unwrap();
                if local.d.abs() > slot.d.abs() {
                    *slot = local;
                }
                drop(slot);
                // PROGRESS, PER JOB. Without it this sweep is untriageable: it prints its
                // table only at the end, so a run that is 1/3 done and one that is wedged
                // look identical from outside, and the only way to tell them apart is
                // /proc accounting. That cost a real decision -- an (H,O,O) sweep ran 18.8
                // hours at 79% CPU efficiency before anyone could say whether it was
                // progressing. s2_build prints per row for exactly this reason.
                let done = progress.fetch_add(1, Ordering::SeqCst) + 1;
                println!(
                    "#   job {done}/{} done ({:.0} s)",
                    jobs.len(),
                    t0.elapsed().as_secs_f64()
                );
            });
        }
    });
    println!("   b/R_e      b     worst |dE3|    at x     theta      z(opp)      s2");
    for (i, b) in bs.iter().enumerate() {
        let w = *acc[i].lock().unwrap();
        println!(
            "   {:5.1}  {b:6.3}   {:.4e}   {:6.3}  {:7.2}   {:8.3}  {:6.3}",
            mults[i], w.d.abs(), w.x, w.theta, w.z, w.s2
        );
    }
    println!("\n# swept in {:.0} s", t0.elapsed().as_secs_f64());
    println!(
        "# READ THIS AS: R_HI is the smallest b whose row is under the table's truncation\n\
         # stake, RE-SWEPT FINELY before it is trusted — a grid maximum understates its own\n\
         # supremum, and SATURATION-2's first sweep cleared its stake by three percent and\n\
         # then failed the fine re-sweep."
    );
}
