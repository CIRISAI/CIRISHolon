//! QVM-GPUFOLD-1 — `conformance/qasm/QVM_GPUFOLD1_PREREG.md`: G1, G2, the three
//! plants, and G3's bench (`#[ignore]`d, run by hand).
//!
//! The budgeted sum (`holon::acuity`) decides the prefix on the host; the
//! backend folds it. The reference on the CPU side is always `holon::mesh` —
//! through `acuity::fold_prefix_on(.., FoldBackend::CpuMesh { shards: 8 })` —
//! never this crate's own twin, which is what PG-1 USES and so cannot also be
//! the referee of G1.
//!
//! Needs CUDA device 0. `ci-gates.sh` cannot reach this crate (its empty
//! `[workspace]` table), so the suite is run by hand:
//!
//! ```text
//! taskset -c 21-27 cargo test --release --manifest-path crates/holon-gpu/Cargo.toml \
//!     --test qvm_gpufold_plants -- --nocapture --test-threads 1
//! taskset -c 21-27 cargo test --release --manifest-path crates/holon-gpu/Cargo.toml \
//!     --test qvm_gpufold_plants -- --ignored --nocapture g3_
//! ```

use holon::acuity::{
    self, budgeted_amplitude_on, fold_prefix_on, single_class, BudgetPlan, Bounded,
    DeviceClass, FoldBackend, SignFlip,
};
use holon::ledger::Cyc;
use holon::magic::{Circuit, Gate};
use holon::magic5::{expected_branches, Magic5Source};
use holon::mesh;
use holon::BranchSource;
use holon_gpu::acuity::{CpuTwinFold, GpuBranchFold};
use holon_gpu::desc::pack_y;
use holon_gpu::{GpuFolder, Shape};

// ---------------------------------------------------------------- instances
// ACUITY-1's family, copied from `holon/tests/qvm_acuity_hard.rs` (tests cannot
// share code across crates); the generator is the same function, not a lookalike.

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn random_circuit(rng: &mut Rng, n: usize, n_clifford: usize, t: usize) -> Circuit {
    let mut gates = Vec::with_capacity(n_clifford + t);
    let (mut t_left, mut c_left) = (t, n_clifford);
    while t_left + c_left > 0 {
        let take_t = if t_left == 0 {
            false
        } else if c_left == 0 {
            true
        } else {
            rng.below((t_left + c_left) as u64) < t_left as u64
        };
        let q = rng.below(n as u64) as usize;
        if take_t {
            t_left -= 1;
            gates.push(if rng.below(2) == 0 { Gate::T(q) } else { Gate::Tdg(q) });
        } else {
            c_left -= 1;
            gates.push(match rng.below(6) {
                0 => Gate::H(q),
                1 => Gate::S(q),
                2 => Gate::Sdg(q),
                3 => Gate::X(q),
                4 => Gate::Z(q),
                _ => {
                    let mut t2 = rng.below(n as u64) as usize;
                    if t2 == q {
                        t2 = (q + 1) % n;
                    }
                    Gate::Cx(q, t2)
                }
            });
        }
    }
    Circuit { n_qubits: n, gates }
}

fn random_y(rng: &mut Rng, n: usize) -> Vec<bool> {
    (0..n).map(|_| rng.below(2) == 1).collect()
}

fn y_with_signal(rng: &mut Rng, n: usize, src: &Magic5Source) -> Vec<bool> {
    for _ in 0..256 {
        let y = random_y(rng, n);
        // S = 7 only for speed: the mesh fold is the same struct at every S.
        let (re, im) = mesh::fold_amplitude(src, &y, 7).to_complex();
        if re.hypot(im) > 1e-6 {
            return y;
        }
    }
    panic!("no y with a live amplitude in 256 draws — the carrier is the problem");
}

/// One instance: the certified source (the bound that truncates), its plan, `y`.
#[allow(dead_code)]
struct Instance {
    n: usize,
    t: usize,
    circuit: Circuit,
    src: Bounded<Magic5Source>,
    plan: BudgetPlan,
    y: Vec<bool>,
}

fn instance(n: usize, t: usize, salt: u64) -> Instance {
    let mut rng = Rng(0x6F01_D000 ^ salt ^ ((n as u64) << 32) ^ (t as u64));
    let circuit = random_circuit(&mut rng, n, 20 * n, t);
    let raw = Magic5Source::new(&circuit);
    let y = y_with_signal(&mut rng, n, &raw);
    let bounds = raw.scalar_bounds();
    let src = Bounded::new(raw, bounds);
    let plan = BudgetPlan::of(&src);
    Instance { n, t, circuit, src, plan, y }
}

fn folder() -> GpuFolder {
    GpuFolder::new(0).expect("no CUDA device 0 — this suite needs one (declared, not detected)")
}

fn is_zero(x: Cyc) -> bool {
    x.c.iter().all(|&v| v == 0)
}

/// The five launch shapes of `tests/determinism.rs`, verbatim.
const SHAPES: [Shape; 5] = [
    Shape { block: 32, grid: 1 },
    Shape { block: 64, grid: 7 },
    Shape { block: 256, grid: 512 },
    Shape { block: 512, grid: 133 },
    Shape { block: 1024, grid: 4096 },
];

/// A per-`y` cache of the source's own per-branch amplitudes, so the CPU mesh
/// can be folded at EVERY prefix length without re-evaluating `N²/2` affine
/// amplitudes. `mesh::fold_amplitude` over this folds the SAME values in the
/// SAME order with the SAME law as over the source itself; the eps ladder
/// below checks the claim on the source directly.
struct Cached {
    amps: Vec<Cyc>,
    n: usize,
}

impl BranchSource for Cached {
    fn n_branches(&self) -> u64 {
        self.amps.len() as u64
    }
    fn amplitude_of(&self, b: u64, _y: &[bool]) -> Cyc {
        self.amps[b as usize]
    }
    fn n_qubits(&self) -> usize {
        self.n
    }
}

/// The eps ladder: absolute and relative (`ε · 2^{−n/2}`) rungs.
fn eps_ladder(n: usize) -> Vec<f64> {
    let rel = 2f64.powf(-(n as f64) / 2.0);
    let mut v = vec![0.0, 1e-1, 1e-2, 1e-3, 1e-4, 1e-6];
    v.extend([1e-1, 1e-2, 1e-3].iter().map(|e| e * rel));
    v
}

// ======================================================================= G1

/// **G1 — bit-identity at every prefix and every ε.** The GPU fold of
/// `order[..k]` is the same `Cyc` STRUCT as the S = 8 mesh, for every `k ∈ [0, N]`
/// on the 16-instance grid; and the budgeted result on each backend carries the
/// same `(value, remainder, k, N, order)` at every ε. No canonicalisation.
#[test]
fn g1_the_gpu_prefix_fold_is_the_mesh_struct_at_every_k_and_eps() {
    let f = folder();
    println!("# G1 on {} — struct equality GPU vs CpuMesh{{8}}", f.name());
    println!(
        "{:>3} {:>3} {:>6} {:>8} {:>8} {:>8} {:>6} {:>6} {:>9}",
        "n", "t", "N", "prefixes", "nonzero", "struct≠", "parity", "expon", "eps rows"
    );
    let (mut compared, mut nonzero_total, mut mismatches, mut want) = (0u64, 0u64, 0u64, 0u64);
    let mut value_only = 0u64;
    let mut eps_rows = 0u64;
    for &n in &[12usize, 16, 20, 24] {
        for &t in &[16usize, 20, 24, 28] {
            let inst = instance(n, t, 0x61);
            let nb = inst.plan.n_branches() as usize;
            assert_eq!(nb as u64, expected_branches(t));
            want += nb as u64 + 1;
            let gpu = GpuBranchFold::new(&f, &inst.src).expect("descriptors");
            let cached = Cached {
                amps: (0..nb as u64).map(|b| inst.src.amplitude_of(b, &inst.y)).collect(),
                n,
            };
            let (mut nz, mut bad) = (0u64, 0u64);
            for k in 0..=nb {
                let cpu = fold_prefix_on(&cached, &inst.plan, k, &inst.y, FoldBackend::CpuMesh { shards: 8 })
                    .expect("the mesh does not fail");
                let dev = fold_prefix_on(&inst.src, &inst.plan, k, &inst.y, FoldBackend::Device(&gpu))
                    .expect("device fold");
                compared += 1;
                if !is_zero(cpu) {
                    nz += 1;
                }
                if cpu != dev {
                    bad += 1;
                    if mesh::canonicalize(cpu) == mesh::canonicalize(dev) {
                        value_only += 1;
                    }
                    if bad <= 3 {
                        println!("#   n={n} t={t} k={k}: CPU {cpu:?} GPU {dev:?}");
                    }
                }
            }
            // The flags at the full prefix, where the batch is largest.
            let info = gpu.ensure_resident(&inst.plan.order).expect("resident");
            // The eps ladder, on the SOURCE itself (not the cache).
            for eps in eps_ladder(n) {
                let c = budgeted_amplitude_on(&inst.src, &inst.plan, &inst.y, eps, FoldBackend::CpuMesh { shards: 8 })
                    .expect("mesh");
                let g = budgeted_amplitude_on(&inst.src, &inst.plan, &inst.y, eps, FoldBackend::Device(&gpu))
                    .expect("device");
                assert_eq!(c.value, g.value, "G1 KILLED: n={n} t={t} eps={eps:e} value struct");
                assert_eq!(c.remainder.to_bits(), g.remainder.to_bits(), "remainder moved with the backend");
                assert_eq!((c.evaluated, c.total, c.order), (g.evaluated, g.total, g.order));
                assert_eq!(c.device, DeviceClass::Cpu);
                assert_eq!(g.device, DeviceClass::Gpu);
                eps_rows += 1;
            }
            println!(
                "{n:>3} {t:>3} {nb:>6} {:>8} {nz:>8} {bad:>8} {:>6} {:>6} {:>9}",
                nb + 1,
                info.parity_uniform,
                info.exponent_uniform,
                eps_ladder(n).len()
            );
            nonzero_total += nz;
            mismatches += bad;
        }
    }
    println!(
        "# G1: {compared} prefix comparisons (want {want}), {nonzero_total} with a nonzero fold, \
         {mismatches} struct mismatches ({value_only} of them value-equal after canonicalize); \
         {eps_rows} eps rows"
    );
    // M-VACUOUS-SUCCESS: the work count, asserted.
    assert_eq!(compared, want, "G1 did not reach every prefix");
    assert!(nonzero_total > compared / 2, "G1's carrier is mostly zero folds");
    assert_eq!(mismatches, 0, "G1 KILLED: {mismatches} prefixes with unequal structs");
}

// ======================================================================= G2

/// **G2 — the device class is part of the artifact.**
#[test]
fn g2_the_class_is_declared_and_a_cpu_run_is_refused_as_gpu() {
    let f = folder();
    let inst = instance(12, 16, 0x62);
    let gpu = GpuBranchFold::new(&f, &inst.src).expect("descriptors");
    let twin = CpuTwinFold::new(&inst.src, 4).expect("descriptors");

    let on_cpu = budgeted_amplitude_on(&inst.src, &inst.plan, &inst.y, 0.0, FoldBackend::CpuMesh { shards: 8 })
        .unwrap();
    let on_gpu = budgeted_amplitude_on(&inst.src, &inst.plan, &inst.y, 0.0, FoldBackend::Device(&gpu)).unwrap();
    let on_twin = budgeted_amplitude_on(&inst.src, &inst.plan, &inst.y, 0.0, FoldBackend::Device(&twin)).unwrap();
    // The legacy entry points are the CPU mesh and say so.
    let legacy = acuity::budgeted_amplitude_with(&inst.src, &inst.plan, &inst.y, 0.0, 8);

    assert_eq!(on_cpu.device, DeviceClass::Cpu);
    assert_eq!(legacy.device, DeviceClass::Cpu);
    assert_eq!(on_gpu.device, DeviceClass::Gpu);
    // The twin goes through the SAME `Device` door and still declares Cpu.
    assert_eq!(on_twin.device, DeviceClass::Cpu);

    // Same number, different artifacts: the refusal does not care that they agree.
    assert_eq!(on_cpu.value, on_gpu.value);
    for (name, r) in [("CpuMesh", &on_cpu), ("legacy", &legacy), ("CpuTwin", &on_twin)] {
        let refused = r.require_class(DeviceClass::Gpu);
        assert!(refused.is_err(), "G2 KILLED: a {name} result was admitted as Gpu");
        println!("# G2 {name}: {}", refused.unwrap_err());
    }
    assert!(on_gpu.require_class(DeviceClass::Gpu).is_ok());
    assert!(on_gpu.require_class(DeviceClass::Cpu).is_err());

    // A table that mixes classes is refused; one class is admitted.
    assert!(single_class(&[on_gpu.clone(), on_cpu.clone()]).is_err(), "a mixed table was admitted");
    assert_eq!(single_class(&[on_gpu.clone(), on_gpu.clone()]).unwrap(), Some(DeviceClass::Gpu));
    assert_eq!(single_class(&[on_cpu.clone(), legacy.clone(), on_twin.clone()]).unwrap(), Some(DeviceClass::Cpu));

    // No fallback, two ways. A device that does not exist is an ERROR at the
    // door, not a host fold; and a register the descriptor cannot hold (n + t > 64)
    // is refused at construction rather than routed to the host.
    assert!(GpuFolder::new(7).is_err(), "device ordinal 7 opened on a one-GPU box");
    let mut rng = Rng(0x6F01_D062);
    let wide = random_circuit(&mut rng, 40, 80, 28); // 40 + 28 = 68 wires
    let wide_src = acuity::source_for(&wide, &vec![false; 40]);
    assert!(GpuBranchFold::new(&f, &wide_src).is_err(), "a 68-wire register was accepted");
    println!("# G2: 68-wire register refused at construction; ordinal 7 refused at the door");
}

// ======================================================================= PG-1

/// **PG-1 — a corrupted lane on the device, convicted by the CPU twin.**
#[test]
fn pg1_a_corrupted_device_lane_is_convicted_by_the_cpu_twin() {
    let f = folder();
    let inst = instance(16, 20, 0x71);
    let gpu = GpuBranchFold::new(&f, &inst.src).expect("descriptors");
    let prefix = inst.plan.order.clone();
    let yp = pack_y(&inst.y);

    assert!(gpu.audit(&prefix, &inst.y).unwrap().is_none(), "a clean batch was convicted");

    // The carrier: the first prefix position whose branch is LIVE at y — a dead
    // branch's base is never summed, so corrupting it would plant nothing.
    let pos = prefix
        .iter()
        .position(|&b| !is_zero(gpu.descs()[b as usize].amplitude(yp)))
        .expect("PG-1's carrier is empty: no live branch");
    let branch = prefix[pos];
    for limb in [0usize, 5] {
        gpu.evict();
        gpu.plant_lane(&prefix, pos, limb, 1 << 3).expect("plant");
        let conv = gpu.audit(&prefix, &inst.y).unwrap().expect("PG-1 FAILED: the corrupted lane was not convicted");
        println!("# PG-1 limb {limb}: {conv}");
        assert_eq!(conv.lanes, vec![(pos, branch, limb)], "the wrong lane was named");
        // Through the budgeted door the plant is visible too: the GPU result
        // now disagrees with the mesh.
        let g = budgeted_amplitude_on(&inst.src, &inst.plan, &inst.y, 0.0, FoldBackend::Device(&gpu)).unwrap();
        let c = budgeted_amplitude_on(&inst.src, &inst.plan, &inst.y, 0.0, FoldBackend::CpuMesh { shards: 8 }).unwrap();
        if limb % 2 == 0 {
            // a low-limb flip moves a live lane by 8 units of 2^{-m/2}
            assert_ne!(g.value, c.value, "PG-1: the fold did not move");
            assert_ne!(conv.device, conv.twin);
        }
        // A high-limb flip adds 2^67 to one lane: it too must move the fold.
        assert_ne!(conv.device, conv.twin, "PG-1: the fold did not see limb {limb}");
    }
    // Re-upload heals: the conviction was of the card's lane, not of the source.
    gpu.evict();
    assert!(gpu.audit(&prefix, &inst.y).unwrap().is_none(), "a fresh upload was convicted");
}

// ======================================================================= PG-2

/// **PG-2 — a scrambled schedule leaves the struct unchanged**: five launch
/// shapes, and the prefix uploaded in permuted orders.
#[test]
fn pg2_a_scrambled_schedule_leaves_the_struct_unchanged() {
    let f = folder();
    for &(n, t) in &[(12usize, 20usize), (20, 28)] {
        let inst = instance(n, t, 0x72);
        let gpu = GpuBranchFold::new(&f, &inst.src).expect("descriptors");
        // The full prefix and a TRUNCATED one: ε = R_j for the first j ≥ N/2
        // whose budgeted prefix folds to NONZERO (M-PLANT-SECTOR — a prefix
        // that folds to zero is a carrier on which a scramble could hide), so
        // the scramble is exercised on a prefix the budget actually cut.
        let cut = (inst.plan.order.len() / 2..inst.plan.order.len())
            // ascending from the middle
            .map(|j| inst.plan.remainder(j))
            .find(|&e| {
                let k = inst.plan.stop_at(e);
                k > 1
                    && !is_zero(
                        fold_prefix_on(&inst.src, &inst.plan, k, &inst.y, FoldBackend::CpuMesh { shards: 1 })
                            .unwrap(),
                    )
            })
            .expect("PG-2: no truncated prefix with a nonzero fold");
        for (ei, eps) in [0.0, cut].into_iter().enumerate() {
            let k = inst.plan.stop_at(eps);
            if ei == 1 {
                assert!(k < inst.plan.order.len() && k > 1, "PG-2: the cut rung did not truncate (k={k})");
            }
            let prefix: Vec<u64> = inst.plan.order[..k].to_vec();
            let reference =
                fold_prefix_on(&inst.src, &inst.plan, k, &inst.y, FoldBackend::CpuMesh { shards: 8 }).unwrap();
            assert!(!is_zero(reference), "PG-2's carrier is a zero fold at n={n} t={t} k={k}");
            let mut orders: Vec<Vec<u64>> = vec![prefix.clone()];
            let mut rev = prefix.clone();
            rev.reverse();
            orders.push(rev);
            let mut rot = prefix.clone();
            rot.rotate_left(k / 3);
            orders.push(rot);
            let mut shuf = prefix.clone();
            let mut rng = Rng(0x5C4A_3B1E ^ k as u64);
            for i in (1..shuf.len()).rev() {
                shuf.swap(i, rng.below(i as u64 + 1) as usize);
            }
            orders.push(shuf);
            let mut folds = 0;
            for (oi, ord) in orders.iter().enumerate() {
                for s in SHAPES {
                    gpu.set_shape(Some(s));
                    let v = holon::acuity::DeviceFold::fold_prefix(&gpu, ord, &inst.y).unwrap();
                    assert_eq!(v, reference, "PG-2 FAILED: n={n} t={t} k={k} order {oi} shape {s:?}");
                    folds += 1;
                }
            }
            gpu.set_shape(None);
            println!("# PG-2 n={n} t={t} k={k}: {folds} scrambled folds, one struct {reference:?}");
        }
    }
}

// ======================================================================= PG-3

/// `⟨y|C|0^n⟩` by dense statevector — shares no code with magic5/affine/mesh.
/// Copied from `holon/tests/qvm_acuity_hard.rs`.
fn statevector_amplitude(c: &Circuit, y: &[bool]) -> (f64, f64) {
    let n = c.n_qubits;
    let dim = 1usize << n;
    let mut re = vec![0.0f64; dim];
    let mut im = vec![0.0f64; dim];
    re[0] = 1.0;
    let s = std::f64::consts::FRAC_1_SQRT_2;
    for g in &c.gates {
        match *g {
            Gate::X(q) => {
                for x in 0..dim {
                    if x >> q & 1 == 0 {
                        let z = x | (1 << q);
                        re.swap(x, z);
                        im.swap(x, z);
                    }
                }
            }
            Gate::Z(q) => {
                for x in 0..dim {
                    if x >> q & 1 == 1 {
                        re[x] = -re[x];
                        im[x] = -im[x];
                    }
                }
            }
            Gate::S(q) | Gate::Sdg(q) => {
                let sgn = if matches!(*g, Gate::S(_)) { 1.0 } else { -1.0 };
                for x in 0..dim {
                    if x >> q & 1 == 1 {
                        let (a, b) = (re[x], im[x]);
                        re[x] = -sgn * b;
                        im[x] = sgn * a;
                    }
                }
            }
            Gate::T(q) | Gate::Tdg(q) => {
                let sgn = if matches!(*g, Gate::T(_)) { 1.0 } else { -1.0 };
                let (wr, wi) = (s, sgn * s);
                for x in 0..dim {
                    if x >> q & 1 == 1 {
                        let (a, b) = (re[x], im[x]);
                        re[x] = a * wr - b * wi;
                        im[x] = a * wi + b * wr;
                    }
                }
            }
            Gate::H(q) => {
                for x in 0..dim {
                    if x >> q & 1 == 0 {
                        let z = x | (1 << q);
                        let (a0, b0, a1, b1) = (re[x], im[x], re[z], im[z]);
                        re[x] = (a0 + a1) * s;
                        im[x] = (b0 + b1) * s;
                        re[z] = (a0 - a1) * s;
                        im[z] = (b0 - b1) * s;
                    }
                }
            }
            Gate::Cx(ctrl, tgt) => {
                for x in 0..dim {
                    if x >> ctrl & 1 == 1 && x >> tgt & 1 == 0 {
                        let z = x | (1 << tgt);
                        re.swap(x, z);
                        im.swap(x, z);
                    }
                }
            }
            #[allow(unreachable_patterns)]
            other => panic!("statevector referee: gate {other:?} not in the family"),
        }
    }
    let idx: usize = y.iter().enumerate().fold(0, |a, (i, &b)| a | ((b as usize) << i));
    (re[idx], im[idx])
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// **PG-3 — a planted wrong branch breaks the certificate, on the device**,
/// exactly as ACUITY-1's PQ-4: the sign-flipped term's bound is unchanged.
#[test]
fn pg3_a_planted_sign_flip_breaks_the_certificate_on_the_device() {
    let f = folder();
    for &t in &[8usize, 12] {
        // PQ-4's own instances: n = 10, 20n Clifford gates, its seeds.
        let mut rng = Rng(0xACE1_7400 + t as u64);
        let n = 10;
        let c = random_circuit(&mut rng, n, 20 * n, t);
        let raw = Magic5Source::new(&c);
        let y = y_with_signal(&mut rng, n, &raw);
        let src = Bounded::new(acuity::source_for(&c, &y), raw.scalar_bounds());
        let plan = BudgetPlan::of(&src);
        let state = statevector_amplitude(&c, &y);

        let clean_gpu = GpuBranchFold::new(&f, &src).expect("descriptors");
        let clean = budgeted_amplitude_on(&src, &plan, &y, 0.0, FoldBackend::Device(&clean_gpu)).unwrap();
        assert!(dist(clean.value_f64, state) <= 1e-9, "the clean GPU sum must meet the referee first");

        let (pos, planted) = plan
            .order
            .iter()
            .copied()
            .enumerate()
            .find(|&(_, b)| !is_zero(src.amplitude_of(b, &y)))
            .expect("PG-3's carrier is empty: no branch live at y");
        let (tr, ti) = src.amplitude_of(planted, &y).to_complex();
        let flipped = SignFlip { inner: &src, branch: planted };
        let gpu = GpuBranchFold::new(&f, &flipped).expect("descriptors");

        let out = budgeted_amplitude_on(&flipped, &plan, &y, 0.0, FoldBackend::Device(&gpu)).unwrap();
        assert_eq!(out.device, DeviceClass::Gpu);
        let d = dist(out.value_f64, state);
        println!(
            "# PG-3 t={t} branch={planted} |2·term|={:.6e} disagreement={d:.6e} R_k={}",
            2.0 * tr.hypot(ti),
            out.remainder
        );
        assert_eq!(out.remainder, 0.0);
        assert!(d > out.remainder, "PG-3 FAILED: the planted branch did not break the device sum");
        assert!((d - 2.0 * tr.hypot(ti)).abs() < 1e-9, "the plant moved the sum by something else");
        // The device sum of the planted source is the mesh's sum of it, struct for struct.
        let mesh_flip =
            budgeted_amplitude_on(&flipped, &plan, &y, 0.0, FoldBackend::CpuMesh { shards: 8 }).unwrap();
        assert_eq!(out.value, mesh_flip.value, "the planted source folds differently on the two backends");

        let mut live = 0;
        for &eps in &[1e-2f64, 1e-3, 1e-4] {
            let k = plan.stop_at(eps);
            if pos >= k {
                continue;
            }
            let o = budgeted_amplitude_on(&flipped, &plan, &y, eps, FoldBackend::Device(&gpu)).unwrap();
            let dd = dist(o.value_f64, state);
            println!("# PG-3 t={t} eps={eps:.0e} k={k} disagreement={dd:.6e} R_k={:.6e}", o.remainder);
            assert!(dd > o.remainder, "PG-3 FAILED at eps={eps}: {dd} ≤ R_k {}", o.remainder);
            live += 1;
        }
        println!("# PG-3 t={t}: {live} live-budget rungs contained the plant");
    }
}

// ======================================================================= G3

fn stats(mut v: Vec<f64>) -> (f64, f64, f64) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (v[0], v[v.len() / 2], v[v.len() - 1])
}

/// **G3 — the device pays, round trip included.** `#[ignore]`d: timing, by hand.
///
/// `taskset -c 21-27 cargo test --release --manifest-path crates/holon-gpu/Cargo.toml \
///     --test qvm_gpufold_plants -- --ignored --nocapture g3_`
#[test]
#[ignore = "timing: run by hand, taskset -c 21-27, GPU idle"]
fn g3_gpu_fold_against_the_s8_mesh() {
    let f = folder();
    let reps = 25;
    println!("# G3 on {}; ms; min / median / max over {reps}; eps = 0 (k = N)", f.name());
    println!(
        "{:>3} {:>3} {:>6} | {:>26} | {:>26} | {:>26} | {:>26} | {:>9} {:>9} {:>9} | {:>7} {:>7}",
        "n", "t", "N", "CPU S=8 wall", "GPU warm wall (round trip)", "GPU kernel (events)",
        "GPU cold (upload+query)", "ratio min", "ratio med", "cold med", "decode", "load"
    );
    for &t in &[24usize, 28, 32] {
        let n = 24;
        let t_build = std::time::Instant::now();
        let inst = instance(n, t, 0x63);
        let build_s = t_build.elapsed().as_secs_f64();
        let nb = inst.plan.n_branches();
        assert_eq!(nb, expected_branches(t));
        let t_dec = std::time::Instant::now();
        let gpu = GpuBranchFold::new(&f, &inst.src).expect("descriptors");
        let decode_ms = t_dec.elapsed().as_secs_f64() * 1e3;
        let load0 = holon_gpu::loadavg();

        // CPU arm: the S = 8 mesh through the budgeted door.
        let mut cpu = Vec::new();
        let mut cpu_val = None;
        for _ in 0..reps {
            let t0 = std::time::Instant::now();
            let r = budgeted_amplitude_on(&inst.src, &inst.plan, &inst.y, 0.0, FoldBackend::CpuMesh { shards: 8 })
                .unwrap();
            cpu.push(t0.elapsed().as_secs_f64() * 1e3);
            cpu_val = Some(r.value);
        }
        // GPU warm: the batch resident, the budgeted door, the round trip in the wall.
        gpu.ensure_resident(&inst.plan.order).expect("upload");
        let mut warm = Vec::new();
        let mut kern = Vec::new();
        for _ in 0..reps {
            let t0 = std::time::Instant::now();
            let r = budgeted_amplitude_on(&inst.src, &inst.plan, &inst.y, 0.0, FoldBackend::Device(&gpu)).unwrap();
            warm.push(t0.elapsed().as_secs_f64() * 1e3);
            assert_eq!(Some(r.value), cpu_val, "G3: the arms disagree — nothing is read");
            let (v, ms) = gpu.fold_resident_timed(&inst.plan.order, &inst.y).unwrap();
            assert_eq!(Some(v), cpu_val);
            kern.push(ms as f64);
        }
        // GPU cold: evict, then the budgeted door pays the gather + upload.
        let mut cold = Vec::new();
        for _ in 0..reps {
            gpu.evict();
            let t0 = std::time::Instant::now();
            let r = budgeted_amplitude_on(&inst.src, &inst.plan, &inst.y, 0.0, FoldBackend::Device(&gpu)).unwrap();
            cold.push(t0.elapsed().as_secs_f64() * 1e3);
            assert_eq!(Some(r.value), cpu_val);
        }
        // The PREREG's cold (§2 G3): descriptor decode + upload + query, i.e. a
        // fresh backend built from the source for one amplitude.
        let mut cold_full = Vec::new();
        for _ in 0..reps {
            let t0 = std::time::Instant::now();
            let g = GpuBranchFold::new(&f, &inst.src).expect("descriptors");
            let r = budgeted_amplitude_on(&inst.src, &inst.plan, &inst.y, 0.0, FoldBackend::Device(&g)).unwrap();
            cold_full.push(t0.elapsed().as_secs_f64() * 1e3);
            assert_eq!(Some(r.value), cpu_val);
        }
        // CONTEXT, not staked: the tightest honest CPU arm (`cpu.rs`'s header) —
        // the SAME packed descriptors folded on the host at S = 8. It separates
        // what the descriptor packing buys from what the device buys; the stake
        // is against the mesh arm ACUITY-1 measured, and stays there.
        let twin = CpuTwinFold::from_descs(gpu.descs().to_vec(), n, 8);
        let mut packed = Vec::new();
        for _ in 0..reps {
            let t0 = std::time::Instant::now();
            let r = budgeted_amplitude_on(&inst.src, &inst.plan, &inst.y, 0.0, FoldBackend::Device(&twin)).unwrap();
            packed.push(t0.elapsed().as_secs_f64() * 1e3);
            assert_eq!(Some(r.value), cpu_val, "the packed twin disagrees with the mesh");
        }
        let (p0, p1, p2) = stats(packed);
        println!(
            "#   t={t}: CPU packed twin S=8 (context) min {p0:.4} med {p1:.4} max {p2:.4} ms; \
             GPU warm / packed twin med {:.4}",
            stats(warm.clone()).1 / p1
        );
        let (f0, f1, f2) = stats(cold_full);
        println!(
            "#   t={t}: prereg cold (decode+upload+query) min {f0:.4} med {f1:.4} max {f2:.4} ms, \
             ratio to CPU S=8 med {:.4}; first decode {decode_ms:.1} ms",
            f1 / stats(cpu.clone()).1
        );
        let load1 = holon_gpu::loadavg();
        let (c0, c1, c2) = stats(cpu);
        let (w0, w1, w2) = stats(warm);
        let (k0, k1, k2) = stats(kern);
        let (o0, o1, o2) = stats(cold);
        println!(
            "{n:>3} {t:>3} {nb:>6} | {c0:>8.4} {c1:>8.4} {c2:>8.4} | {w0:>8.4} {w1:>8.4} {w2:>8.4} | \
             {k0:>8.4} {k1:>8.4} {k2:>8.4} | {o0:>8.4} {o1:>8.4} {o2:>8.4} | {:>9.4} {:>9.4} {:>9.4} | \
             {decode_ms:>7.1} {:>3.1}-{:<3.1}",
            w0 / c0,
            w1 / c1,
            o1 / c1,
            load0,
            load1
        );
        println!(
            "#   t={t}: source+plan build {build_s:.2} s; spread max/min CPU {:.2}x warm {:.2}x cold {:.2}x; \
             round trip (warm − kernel) median {:.4} ms",
            c2 / c0,
            w2 / w0,
            o2 / o0,
            w1 - k1
        );
    }
}
