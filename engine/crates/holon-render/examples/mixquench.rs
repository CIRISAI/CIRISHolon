//! MIXTURES-1 gate P1: the mixed quench, and its two single-species controls.
//!
//! ```text
//! cargo run --release -p holon-render --example mixquench -- <arm> [seed ...]
//!   arm = mixed | hydrogen | chlorine | cost
//! ```
//!
//! With no seeds listed it runs the eight staked ones. `cost` measures the two inputs the
//! schedule is frozen against — how the derived timestep depends on knot density, and each
//! arm's per-boundary rate — so `FRAMES` and `CURVE_KNOTS` rest on measurements rather than
//! on guesses. It scores nothing.
//!
//! THE PROTOCOL IS FROZEN in `conformance/atomworld/MIXTURES1_RESULTS.md`, committed
//! before the mixed arm was run or its output looked at, as the prereg requires. Every
//! number it needs is a `const` in this file rather than a flag, so a reported run re-runs
//! byte for byte — and so that a run whose parameters were overridden cannot be reported
//! as one whose parameters were staked.
//!
//! # What P1 asks
//!
//! > the frozen quench protocol on an 8 H + 8 Cl gas ends with HCl as the modal molecule
//! > (branch a), the H-only and Cl-only controls reproducing their own banked behaviours.
//! > Branch (b) reported and investigated. VOID if a control fails.
//!
//! # The fence, displayed rather than assumed
//!
//! The three-body term is H3-ONLY: `Sim::accumulate_three_body` skips any triple
//! containing a non-hydrogen atom. So the mixed arm runs MBE2-exact over all three pair
//! types plus MBE3 over the hydrogen triples only, and no reading here is
//! beyond-pair-complete for a triple containing chlorine. Every run prints the fence.

use holon_chem::elements::{Species, CHLORINE, HYDROGEN};
use holon_render::sim::{Boundary, Dims, Sim, K_B, MAX_ATOMS};
use holon_render::{generate_trimer_table, load_pair_table, TABLE_OK};
use std::io::Write;
use std::time::Instant;

macro_rules! say {
    ($($t:tt)*) => {{
        println!($($t)*);
        let _ = std::io::stdout().flush();
    }};
}

// ------------------------------------------------------------------ THE FROZEN PROTOCOL

/// The eight staked seeds. Written here, not generated: a seed a program chose is a seed
/// nobody staked. Distinct from SATURATION-1's set so the two campaigns' runs cannot be
/// confused for one another.
const SEEDS: [u64; 8] = [
    0x0000_0000_4d49_5801,
    0x0000_0000_4d49_5802,
    0x0000_0000_4d49_5803,
    0x0000_0000_4d49_5804,
    0x0000_0000_4d49_5805,
    0x0000_0000_4d49_5806,
    0x0000_0000_4d49_5807,
    0x0000_0000_4d49_5808,
];

const N_ATOMS: usize = 16;

/// Box, bohr. SATURATION-1's box kept, so the mixed quench happens in the scene the
/// hydrogen quench was measured in and the H-only control is comparable to its bank.
const BOX_W: f64 = 40.0;
const BOX_H: f64 = 24.0;

/// Initial and target kinetic temperature, kelvin, and the Berendsen coupling time in
/// atomic time units. SATURATION-1's values kept, for the same reason as the box.
///
/// The three wells in play are H2 at 0.204 Ha, HCl at 0.148 and Cl2 at 0.0646 — 64,500 K,
/// 46,800 K and 20,400 K in temperature units. `T_INIT` is well below the shallowest of
/// them, so the opening scene is a gas that can condense rather than one that cannot bind.
const T_INIT: f64 = 3000.0;
const T_TARGET: f64 = 300.0;
const TAU: f64 = 2000.0;

/// Grain boundaries per run, and substeps per boundary.
///
/// SATURATION-1's 20,000 x 64 kept. The cost was measured before this was frozen (`cost`
/// arm) rather than inherited on faith: the mixed arm's force loop is the same 120 pairs
/// and FEWER triples, because the H3-only fence skips every triple containing a chlorine.
const FRAMES: usize = 20000;
const SUBSTEPS: u32 = 64;

/// Jitter on the opening lattice, bohr.
const JITTER: f64 = 0.8;

/// Knots per pair curve.
///
/// MEASURED before it was frozen, by the `cost` arm's knot sweep. `R_e`, `D_e` and `k_e`
/// do not depend on it at all — they come from `locate_well`'s own Newton solve on the
/// solver rather than from the interpolant — and the DERIVED TIMESTEP moves by 0.25% from
/// 24 knots to 384 (1.079664 against 1.076997 on the opened hydrogen scene). So the choice
/// is about the interpolant's accuracy BETWEEN knots and about cost, not about the clock.
///
/// 96 is four times the test fixtures' 24 and about a fifth of the H2 viewer curve's 492.
/// Cl2 is what prices it: 18 basis functions, about 97 s at 48 knots on the campaign
/// machine, and the curves are generated once per process rather than once per seed.
const CURVE_KNOTS: usize = 96;

/// THE MIXED ARM'S COMPOSITION, frozen: which lattice cells hold chlorine.
///
/// A CHECKERBOARD on the 4 x 4 lattice — cell `(col + row)` odd is chlorine — which puts
/// eight of each and gives every chlorine four hydrogen nearest neighbours and vice versa.
/// Stated as a rule rather than a list so it cannot be quietly re-drawn, and chosen before
/// any arm ran: an opening that clustered the chlorines on one side of the box would be an
/// opening that decided the answer.
fn is_chlorine_cell(i: usize) -> bool {
    let (col, row) = (i % 4, i / 4);
    (col + row) % 2 == 1
}

// ------------------------------------------------------------------ deterministic setup

fn lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}

fn gauss(state: &mut u64) -> f64 {
    let u1 = lcg(state).max(1e-12);
    let u2 = lcg(state);
    (-2.0 * u1.ln()).sqrt() * (core::f64::consts::TAU * u2).cos()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Mixed,
    Hydrogen,
    Chlorine,
}

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Arm::Mixed => "mixed (8 H + 8 Cl)",
            Arm::Hydrogen => "control: hydrogen only (16 H)",
            Arm::Chlorine => "control: chlorine only (16 Cl)",
        }
    }

    /// The species of lattice cell `i` under this arm.
    fn species_at(self, i: usize) -> Species {
        match self {
            Arm::Hydrogen => HYDROGEN,
            Arm::Chlorine => CHLORINE,
            Arm::Mixed => {
                if is_chlorine_cell(i) {
                    CHLORINE
                } else {
                    HYDROGEN
                }
            }
        }
    }

    /// The pair types this arm needs banked.
    fn pair_types(self) -> Vec<(Species, Species)> {
        match self {
            Arm::Hydrogen => vec![(HYDROGEN, HYDROGEN)],
            Arm::Chlorine => vec![(CHLORINE, CHLORINE)],
            Arm::Mixed => vec![
                (HYDROGEN, HYDROGEN),
                (HYDROGEN, CHLORINE),
                (CHLORINE, CHLORINE),
            ],
        }
    }
}

/// Build the arm's scene: the bank filled with exactly the pair types it needs, the atoms
/// on the lattice, velocities Maxwellian at `T_INIT` with the net drift removed.
///
/// The velocity width is PER SPECIES: `sigma = sqrt(k_B T / m)`, so a chlorine opens 5.9
/// times slower than a hydrogen at the same temperature. Drawing one width for the whole
/// box would open the mixed arm at two different temperatures and call it one.
fn build(arm: Arm, seed: u64, curves: &Curves) -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    s.boundary = Boundary::Walls;
    s.dims = Dims::Two;
    s.width = BOX_W;
    s.height = BOX_H;
    s.reset(N_ATOMS);
    for i in 0..N_ATOMS {
        assert!(
            s.set_species(i, arm.species_at(i)),
            "the bank refused a species the arm needs"
        );
    }
    for (a, b) in arm.pair_types() {
        let status = load_pair_table(&mut s, curves.get(a, b), holon_render::bank::Host::Native);
        assert_eq!(
            status, TABLE_OK,
            "{}{} did not load into the bank (status {status})",
            a.symbol, b.symbol
        );
    }
    // The three-body surface. H3-only by construction; see the module header's fence.
    generate_trimer_table(&mut s);

    let mut st = seed;
    let mut vs = [(0.0f64, 0.0f64); MAX_ATOMS];
    let mut px = 0.0;
    let mut py = 0.0;
    #[allow(clippy::needless_range_loop)]
    for i in 0..N_ATOMS {
        let (col, row) = (i % 4, i / 4);
        let x = BOX_W * (col as f64 + 0.5) / 4.0 + JITTER * (2.0 * lcg(&mut st) - 1.0);
        let y = BOX_H * (row as f64 + 0.5) / 4.0 + JITTER * (2.0 * lcg(&mut st) - 1.0);
        s.set_position(i, x, y);
        let sigma = (K_B * T_INIT / s.atoms[i].mass()).sqrt();
        let (vx, vy) = (sigma * gauss(&mut st), sigma * gauss(&mut st));
        vs[i] = (vx, vy);
        // MOMENTUM, not velocity: the drift that has to be removed is the box's total
        // momentum, and in a mixed box that is not the mean velocity.
        px += s.atoms[i].mass() * vx;
        py += s.atoms[i].mass() * vy;
    }
    let total_mass: f64 = (0..N_ATOMS).map(|i| s.atoms[i].mass()).sum();
    #[allow(clippy::needless_range_loop)]
    for i in 0..N_ATOMS {
        s.set_velocity(i, vs[i].0 - px / total_mass, vs[i].1 - py / total_mass);
    }
    s.adopt_table_timescale();
    s.rebase();
    s.thermostat_on = true;
    s.target_temperature = T_TARGET;
    s.thermostat_tau = TAU;
    s
}

// ------------------------------------------------------------------ THE MEASUREMENT RULE

/// A component's chemical formula: nuclear charges and how many of each, sorted by `Z`.
///
/// Keyed by `Z` and not by the bank's species index on purpose — the index depends on
/// registration order, so a formula built from it would depend on which atom happened to
/// be placed first.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Formula(Vec<(u32, usize)>);

impl Formula {
    fn size(&self) -> usize {
        self.0.iter().map(|(_, n)| n).sum()
    }

    fn max_z(&self) -> u32 {
        self.0.iter().map(|(z, _)| *z).max().unwrap_or(0)
    }

    fn text(&self) -> String {
        let mut s = String::new();
        for (z, n) in self.0.iter() {
            let sym = holon_chem::elements::by_z(*z).map_or("?", |sp| sp.symbol);
            s.push_str(sym);
            if *n > 1 {
                s.push_str(&n.to_string());
            }
        }
        s
    }
}

/// Every component of size two or more, as a formula. Free atoms are excluded: a component
/// of one atom is a free atom, not a molecule.
fn formulae(s: &Sim) -> Vec<Formula> {
    let counts = s.cluster_species_counts();
    let sizes = s.cluster_sizes();
    let mut out = Vec::new();
    for i in 0..s.n {
        if sizes[i] < 2 {
            continue;
        }
        let mut f: Vec<(u32, usize)> = counts[i]
            .iter()
            .copied()
            .filter(|(z, n)| *z != 0 && *n > 0)
            .collect();
        f.sort();
        out.push(Formula(f));
    }
    out
}

/// THE MODAL MOLECULE, with its tie-break stated in advance.
///
/// The most common formula among components of size two or more. Ties are broken toward
/// the SMALLER component, and then toward the LOWER maximum `Z`, and then
/// lexicographically — a total order, so the answer cannot depend on iteration order.
/// Returns `None` when the run ended with no component of two or more atoms at all, which
/// is a reading and not a failure.
fn modal(fs: &[Formula]) -> Option<(Formula, usize)> {
    let mut tally: Vec<(Formula, usize)> = Vec::new();
    for f in fs {
        match tally.iter_mut().find(|(g, _)| g == f) {
            Some(e) => e.1 += 1,
            None => tally.push((f.clone(), 1)),
        }
    }
    tally.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then(a.0.size().cmp(&b.0.size()))
            .then(a.0.max_z().cmp(&b.0.max_z()))
            .then(a.0.cmp(&b.0))
    });
    tally.first().cloned()
}

// ------------------------------------------------------------------ the curve bank

/// The arm's curves, generated once per process. Cl2 costs about a minute and a half at
/// 48 knots; regenerating it per seed would be eight times that for no new information.
struct Curves {
    entries: Vec<((u32, u32), holon_chem::pair::PairTable)>,
}

impl Curves {
    fn build(arm: Arm) -> Self {
        let mut entries = Vec::new();
        for (a, b) in arm.pair_types() {
            let t0 = Instant::now();
            let pt = holon_chem::pair::generate_pair_table(a, b, CURVE_KNOTS);
            let key = if a.z <= b.z { (a.z, b.z) } else { (b.z, a.z) };
            say!(
                "  curve {}{}: {} knots, n_det {}, route {:?}, {:.1} s, well {:?}",
                a.symbol,
                b.symbol,
                pt.r.len(),
                pt.meta.n_det,
                pt.meta.route,
                t0.elapsed().as_secs_f64(),
                pt.meta.well.map(|w| (w.r_e, w.d_e))
            );
            entries.push((key, pt));
        }
        Curves { entries }
    }

    fn get(&self, a: Species, b: Species) -> &holon_chem::pair::PairTable {
        let key = if a.z <= b.z { (a.z, b.z) } else { (b.z, a.z) };
        &self
            .entries
            .iter()
            .find(|(k, _)| *k == key)
            .unwrap_or_else(|| panic!("no banked curve for {}{}", a.symbol, b.symbol))
            .1
    }
}

// ------------------------------------------------------------------ the run

struct Outcome {
    seed: u64,
    modal: Option<(Formula, usize)>,
    formulae: Vec<Formula>,
    free: usize,
    drift: f64,
    bound: f64,
    momentum: f64,
    momentum_bound: f64,
    temperature: f64,
    e_three: f64,
    dt: f64,
    seconds: f64,
}

fn run_one(arm: Arm, seed: u64, curves: &Curves) -> Outcome {
    let t0 = Instant::now();
    let mut s = build(arm, seed, curves);
    for _ in 0..FRAMES {
        s.step_frame(SUBSTEPS);
    }
    let fs = formulae(&s);
    let sizes = s.cluster_sizes();
    let free = sizes[..s.n].iter().filter(|&&x| x == 1).count();
    Outcome {
        seed,
        modal: modal(&fs),
        formulae: fs,
        free,
        drift: s.drift_peak,
        bound: s.drift_bound(),
        momentum: s.momentum_residual_peak,
        momentum_bound: s.momentum_bound(),
        temperature: s.temperature(),
        e_three: s.e_three,
        dt: s.dt(),
        seconds: t0.elapsed().as_secs_f64(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arm_name = args.get(1).map(String::as_str).unwrap_or("cost");
    let seeds: Vec<u64> = if args.len() > 2 {
        args[2..]
            .iter()
            .map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).expect("seed"))
            .collect()
    } else {
        SEEDS.to_vec()
    };

    if arm_name == "cost" {
        cost();
        return;
    }
    if arm_name == "diagnose" {
        diagnose(seeds[0]);
        return;
    }
    let arm = match arm_name {
        "mixed" => Arm::Mixed,
        "hydrogen" => Arm::Hydrogen,
        "chlorine" => Arm::Chlorine,
        other => panic!(
            "unknown arm {other}; expected mixed | hydrogen | chlorine | cost | diagnose"
        ),
    };

    say!("# MIXTURES-1 P1 — {}", arm.label());
    say!("#   protocol   conformance/atomworld/MIXTURES1_RESULTS.md, frozen before this ran");
    say!(
        "#   scene      {N_ATOMS} atoms, {BOX_W} x {BOX_H} bohr, walls, 2D; 4x4 lattice, \
         jitter +-{JITTER} bohr"
    );
    say!("#   thermostat Berendsen, T_init {T_INIT} K -> T_target {T_TARGET} K, tau {TAU} a.u.");
    say!("#   run        {FRAMES} boundaries x {SUBSTEPS} substeps");
    say!("#   curves     {CURVE_KNOTS} knots, engine-computed STO-3G FCI");
    say!("#   FENCE      3-body: H3 only. Triples containing a non-hydrogen atom contribute exactly zero.");
    say!("#   composition rule: lattice cell (col+row) odd is chlorine (checkerboard)");
    let curves = Curves::build(arm);

    let mut outcomes = Vec::new();
    for seed in seeds.iter().copied() {
        let o = run_one(arm, seed, &curves);
        report(&o);
        outcomes.push(o);
    }
    summarise(arm, &outcomes);
}

fn report(o: &Outcome) {
    let mut hist: Vec<(String, usize)> = Vec::new();
    for f in o.formulae.iter() {
        let t = f.text();
        match hist.iter_mut().find(|(s, _)| *s == t) {
            Some(e) => e.1 += 1,
            None => hist.push((t, 1)),
        }
    }
    hist.sort();
    let hist_s: Vec<String> = hist.iter().map(|(s, n)| format!("{s}x{n}")).collect();
    say!(
        "seed {:#018x}  modal {:>6}  molecules {:>2}  free {:>2}  [{}]",
        o.seed,
        o.modal.as_ref().map_or("-".to_string(), |(f, _)| f.text()),
        o.formulae.len(),
        o.free,
        hist_s.join(" ")
    );
    say!(
        "    gates: |dE|_peak {:.4e} / bound {:.4e} = {:.4}   |dP|_peak {:.3e} / bound \
         {:.3e} = {:.4}   T {:.1} K   E_three {:+.4e}   dt {:.4}   {:.1} s",
        o.drift,
        o.bound,
        o.drift / o.bound,
        o.momentum,
        o.momentum_bound,
        o.momentum / o.momentum_bound,
        o.temperature,
        o.e_three,
        o.dt,
        o.seconds
    );
}

fn summarise(arm: Arm, os: &[Outcome]) {
    // Pooled formula histogram across every seed.
    let mut pooled: Vec<(String, usize)> = Vec::new();
    for o in os {
        for f in o.formulae.iter() {
            let t = f.text();
            match pooled.iter_mut().find(|(s, _)| *s == t) {
                Some(e) => e.1 += 1,
                None => pooled.push((t, 1)),
            }
        }
    }
    pooled.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    say!("# POOLED over {} seeds: {:?}", os.len(), pooled);

    let mut modal_tally: Vec<(String, usize)> = Vec::new();
    for o in os {
        let t = o.modal.as_ref().map_or("-".to_string(), |(f, _)| f.text());
        match modal_tally.iter_mut().find(|(s, _)| *s == t) {
            Some(e) => e.1 += 1,
            None => modal_tally.push((t, 1)),
        }
    }
    modal_tally.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    say!("# MODAL PER SEED: {:?}", modal_tally);

    let energy_ok = os.iter().all(|o| o.drift <= o.bound);
    let momentum_ok = os.iter().all(|o| o.momentum <= o.momentum_bound);
    say!("# C1 energy gate over every seed: {}", if energy_ok { "HOLDS" } else { "FIRED" });
    say!("# C1 momentum gate over every seed: {}", if momentum_ok { "HOLDS" } else { "FIRED" });

    let hcl = modal_tally
        .iter()
        .find(|(s, _)| s == "HCl")
        .map_or(0, |(_, n)| *n);
    match arm {
        Arm::Mixed => say!(
            "# P1 READING: HCl is the modal molecule in {hcl} of {} seeds. Branch (a) needs \
             6 of 8.",
            os.len()
        ),
        _ => say!(
            "# CONTROL READING: this arm is single-species; its formulae must contain only \
             its own element, and it must produce molecules at all."
        ),
    }
}

/// POST-HOC, AND LABELLED AS SUCH. Not part of the frozen protocol and scoring nothing.
///
/// The mixed arm came back BRANCH (b): one sixteen-atom component on every seed rather
/// than HCl dimers. The freeze requires branch (b) to be investigated, and the first
/// question is whether that component is a CONDENSED PHASE — atoms sitting at their pairs'
/// own equilibria — or an artefact of the measurement rule, which defines an edge by
/// BOUNDNESS and therefore counts a pair whose interaction is a thousandth of `kT` deep
/// exactly as it counts a chemical bond.
///
/// So this prints, for the final configuration: every bonded edge with its separation, its
/// pair type's own `R_e`, its potential depth, and whether that depth is above or below
/// `k_B T_target`. It also prints the CENSUS reading — the closure-based molecule count the
/// holon layer maintains — because a cluster is a statement about boundness and a census
/// row is a statement about closure, and how far the two disagree is the fence the code
/// already documents made visible.
///
/// Nothing here is a criterion. The gate's reading is the frozen one.
fn diagnose(seed: u64) {
    let arm = Arm::Mixed;
    say!("# POST-HOC DIAGNOSTIC, not the gate. Arm: {}, seed {seed:#018x}", arm.label());
    let curves = Curves::build(arm);
    let mut s = build(arm, seed, &curves);
    for _ in 0..FRAMES {
        s.step_frame(SUBSTEPS);
    }
    let kt = K_B * T_TARGET;
    say!("# k_B T_target = {kt:.6e} Ha at T_target = {T_TARGET} K; measured T = {:.1} K", s.temperature());
    say!("#   R_e per pair type, from the curves themselves:");
    for (a, b) in arm.pair_types() {
        let pt = curves.get(a, b);
        say!(
            "#     {}{}: R_e {:?}  D_e {:?}",
            a.symbol,
            b.symbol,
            pt.meta.well.map(|w| w.r_e),
            pt.meta.well.map(|w| w.d_e)
        );
    }
    say!("i\tj\tpair\tr_bohr\tR_e\tu_Ha\te_rel_Ha\tdepth/kT\tbonded");
    let mut bonded = 0usize;
    let mut tail_bonded = 0usize;
    let mut near_re = 0usize;
    for k in 0..s.pair_count {
        let p = s.pairs[k];
        if !p.bonded {
            continue;
        }
        bonded += 1;
        let (za, zb) = (s.atoms[p.i].species, s.atoms[p.j].species);
        let pt = curves.get(za, zb);
        let slot = s.bank.slot_of_z(za.z, zb.z).unwrap();
        let u = s.bank.table_slot(slot).u(p.r);
        let r_e = pt.meta.well.map(|w| w.r_e).unwrap_or(f64::NAN);
        let ratio = u.abs() / kt;
        if ratio < 1.0 {
            tail_bonded += 1;
        }
        if (p.r - r_e).abs() < 0.5 {
            near_re += 1;
        }
        say!(
            "{}\t{}\t{}{}\t{:.4}\t{:.4}\t{:+.4e}\t{:+.4e}\t{:.3}\t{}",
            p.i, p.j, za.symbol, zb.symbol, p.r, r_e, u, p.e_rel, ratio, p.bonded
        );
    }
    say!(
        "# {bonded} bonded edges. {tail_bonded} of them are shallower than k_B T_target          ({:.1}%), and {near_re} sit within 0.5 bohr of their own pair's R_e ({:.1}%).",
        100.0 * tail_bonded as f64 / bonded.max(1) as f64,
        100.0 * near_re as f64 / bonded.max(1) as f64
    );
    let sizes = s.cluster_sizes();
    let largest = sizes[..s.n].iter().copied().max().unwrap_or(0);
    say!(
        "# CLUSTER reading (boundness): largest component {largest} of {} atoms.",
        s.n
    );
    say!(
        "# CENSUS reading (closure): {} live molecule rows, bond-sector energy {:+.6e} Ha.",
        s.holons.molecule_count(),
        s.holons.bond_sector_energy()
    );
    say!(
        "# The two answer different questions and are expected to differ. Neither is the \
         gate's reading, which is the frozen composition rule."
    );
}

/// The `cost` arm: the schedule's inputs, measured. Scores nothing; it exists so `FRAMES`
/// and `CURVE_KNOTS` are frozen against measurements rather than against guesses.
///
/// It reports two things. First, how the DERIVED TIMESTEP depends on knot density — which
/// it does, even though `R_e`, `D_e` and `k_e` do not, because those come from
/// `locate_well`'s own Newton solve on the solver while `dt` comes from the INTERPOLANT's
/// curvature envelope, and a cubic Hermite's second derivative is knot-dependent on the
/// repulsive wall. A coarse grid therefore reads a stiffer envelope than the curve has and
/// buys a smaller timestep with it. Second, the per-boundary cost of each arm.
fn cost() {
    say!("# knot density against the derived timestep (hydrogen arm, seed 0):");
    for n in [24usize, 48, 96, 192, 384] {
        let t0 = Instant::now();
        let pt = holon_chem::pair::generate_pair_table(HYDROGEN, HYDROGEN, n);
        let mut s = Box::new(Sim::empty());
        s.boundary = Boundary::Walls;
        s.dims = Dims::Two;
        s.width = BOX_W;
        s.height = BOX_H;
        s.reset(N_ATOMS);
        assert_eq!(
            load_pair_table(&mut s, &pt, holon_render::bank::Host::Native),
            TABLE_OK
        );
        s.adopt_table_timescale();
        let dt_curve = s.dt();
        let k_curve = s.timescale.k_env;
        // and again after the opening scene has seeded the envelope
        let curves = Curves {
            entries: vec![((1, 1), pt)],
        };
        let scene = build(Arm::Hydrogen, SEEDS[0], &curves);
        say!(
            "  {n:>4} knots  curve {:>6.1} s  dt(curve) {:.6}  k_env(curve) {:.6e}               dt(opened) {:.6}  k_env(opened) {:.6e}",
            t0.elapsed().as_secs_f64(),
            dt_curve,
            k_curve,
            scene.dt(),
            scene.timescale.k_env
        );
    }
    say!("# per-arm cost:");
    for arm in [Arm::Hydrogen, Arm::Chlorine, Arm::Mixed] {
        let t0 = Instant::now();
        let curves = Curves::build(arm);
        let curve_s = t0.elapsed().as_secs_f64();
        let mut s = build(arm, SEEDS[0], &curves);
        let t1 = Instant::now();
        const PROBE: usize = 100;
        for _ in 0..PROBE {
            s.step_frame(SUBSTEPS);
        }
        let per_frame = t1.elapsed().as_secs_f64() / PROBE as f64;
        say!(
            "{:<32} curves {:>7.1} s   {:.4} s/boundary   {FRAMES} boundaries = {:>7.1} s/seed \
             = {:>6.2} h for 8 seeds   dt {:.4} a.u. -> {:.2} ps of sim time",
            arm.label(),
            curve_s,
            per_frame,
            per_frame * FRAMES as f64,
            per_frame * FRAMES as f64 * 8.0 / 3600.0,
            s.dt(),
            s.dt() * SUBSTEPS as f64 * FRAMES as f64 * holon_render::clock::AU_TO_FS / 1000.0
        );
    }
}
