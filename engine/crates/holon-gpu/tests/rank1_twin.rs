//! QVM-RANK-1 G4 and PR-5: the device path evaluates every exact residual batch the campaign
//! produces — `Σ_j α_j s_j` at every basis state `y`, `α_j` the exact ledger-ring
//! coefficients of a decomposition — bit-identically to the CPU twin (`cpu::fold_packed`),
//! and the value equals `den · v(y)` (the rescaled target). A corrupted lane is convicted by
//! position (PR-5).
//!
//! The batch is one decomposition: its terms are the branches, `2^m` folds, one per `y`. The
//! prereg's search itself (float least squares, pivot-completion scans) is not a branch fold
//! and does not run here; what the device carries is the acceptance arithmetic, which is the
//! one place a wrong bit would matter. Device class: `Gpu` (RTX 4090 Laptop, CUDA via
//! `holon-gpu`'s fold kernel), against `Cpu` (`fold_packed`, one shard and seven).
//!
//! Inputs: every known decomposition (`conformance/rank/known/qubit_H_known.txt`) and every
//! decomposition the campaign wrote under `conformance/rank/g2/` and
//! `conformance/rank/shot/` in the same flat format (`*.dec.txt`).

use holon::affine::{cyc_eq, Affine};
use holon::ledger::Cyc;
use holon::stabrank::{exact_solve, parse_known, target_cyc, KnownDec, Stab, Witness};
use holon_gpu::desc::AffineDesc;
use holon_gpu::{cpu, GpuBatch, GpuFolder, Shape};

const SHAPES: [Shape; 3] = [
    Shape { block: 32, grid: 1 },
    Shape { block: 256, grid: 7 },
    Shape { block: 1024, grid: 64 },
];

fn decompositions() -> Vec<KnownDec> {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../conformance/rank");
    let mut out = parse_known(&std::fs::read_to_string(format!("{root}/known/qubit_H_known.txt")).unwrap());
    for sub in ["g2", "shot"] {
        let Ok(rd) = std::fs::read_dir(format!("{root}/{sub}")) else { continue };
        let mut paths: Vec<_> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            if p.to_string_lossy().ends_with(".dec.txt") {
                out.extend(parse_known(&std::fs::read_to_string(&p).unwrap()));
            }
        }
    }
    out
}

/// One term as an engine `Affine` (unnormalised: amplitude `i^{…}` on the flat), canonicalised,
/// with the scalar canonicalisation pulled out.
fn affine_of(s: &Stab) -> (Affine, Cyc) {
    let n = s.n as usize;
    let k = s.k as usize;
    let mut a = Affine::new(n);
    let qubits: Vec<usize> = (0..n).collect();
    let mut j = vec![vec![false; k]; k];
    for (i, row) in j.iter_mut().enumerate() {
        for (jj, cell) in row.iter_mut().enumerate() {
            let (lo, hi) = if i < jj { (i, jj) } else { (jj, i) };
            *cell = lo != hi && (s.q[lo] >> hi) & 1 == 1;
        }
    }
    a.attach(&qubits, &s.w, s.x0, &s.l, &j);
    let g = a.canonicalize();
    (a, g)
}

fn descs_of(w: &Witness) -> Vec<AffineDesc> {
    w.terms
        .iter()
        .enumerate()
        .map(|(j, s)| {
            let (a, g) = affine_of(s);
            let d = AffineDesc::from_branch(w.alpha_cyc(j).mul(g), &a).expect("descriptor");
            // the descriptor IS the term times its coefficient, entry by entry
            let ph = s.phases();
            for (y, &e) in ph.iter().enumerate() {
                let want = if e == holon::stabrank::ABSENT { Cyc::ZERO } else { w.alpha_cyc(j).mul_i_pow(e) };
                assert!(cyc_eq(d.amplitude(y as u64), want), "descriptor of term {j} wrong at y = {y}");
            }
            d
        })
        .collect()
}

/// G4: every decomposition's residual batch, GPU struct == CPU struct at every y and shape,
/// and the value is exactly den·v(y).
#[test]
fn g4_residual_batches_bit_identical_to_the_cpu_twin() {
    let f = GpuFolder::new(0).expect("no CUDA device 0 — this suite needs one (declared, not detected)");
    let decs = decompositions();
    let (mut batches, mut folds, mut struct_eq) = (0u64, 0u64, 0u64);
    for (m, r, src, terms) in &decs {
        let w = exact_solve(terms).unwrap_or_else(|| panic!("{src}: not exact"));
        assert_eq!(w.terms.len(), *r);
        let descs = descs_of(&w);
        let batch = GpuBatch::upload(&f, &descs).expect("upload");
        let v = target_cyc(*m);
        let den = Cyc { c: [w.den, 0, 0, 0], m: 0 };
        for y in 0..(1u64 << m) {
            let c1 = cpu::fold_packed(&descs, y, 1);
            let c7 = cpu::fold_packed(&descs, y, 7);
            assert!(cyc_eq(c1, den.mul(v[y as usize])), "{src}: CPU fold is not den·v at y = {y}");
            assert!(cyc_eq(c1, c7));
            for sh in SHAPES {
                let g = batch.fold(&f, y, sh).expect("fold");
                folds += 1;
                assert!(cyc_eq(g, c1), "{src}: GPU VALUE differs at y = {y}");
                if g == c1 {
                    struct_eq += 1;
                }
            }
        }
        assert_eq!(batch.read_base(&f).unwrap(), batch.expected_base(&descs), "{src}: resident lanes");
        batches += 1;
    }
    println!(
        "G4 on {}: {batches} residual batches, {folds} GPU folds, struct-equal to the CPU twin on {struct_eq}, value-equal on all",
        f.name()
    );
    assert_eq!(struct_eq, folds, "struct equality (bit identity) on every fold");
}

/// PR-5: one resident limb flipped on the device is convicted by position, and the fold it
/// corrupts disagrees with the twin.
#[test]
fn pr5_flipped_lane_convicted_by_position() {
    let f = GpuFolder::new(0).expect("no CUDA device 0");
    let decs = decompositions();
    let (_, _, src, terms) = decs.iter().find(|d| d.0 == 6).unwrap();
    let w = exact_solve(terms).unwrap();
    let descs = descs_of(&w);
    let mut batch = GpuBatch::upload(&f, &descs).unwrap();
    let b = descs.len();
    for (branch, limb) in [(2usize, 0usize), (4, 5), (0, 3)] {
        let mut clean = GpuBatch::upload(&f, &descs).unwrap();
        let want = clean.expected_base(&descs);
        let orig = want[limb * b + branch];
        clean.plant_base_limb(&f, branch, limb, orig ^ 1).unwrap();
        let got = clean.read_base(&f).unwrap();
        let bad: Vec<usize> = (0..got.len()).filter(|&i| got[i] != want[i]).collect();
        assert_eq!(bad, vec![limb * b + branch], "{src}: conviction by position");
        let convicted = (bad[0] % b, bad[0] / b);
        assert_eq!(convicted, (branch, limb));
        // and the value moves somewhere
        let moved = (0..(1u64 << 6)).any(|y| {
            let g = clean.fold(&f, y, SHAPES[1]).unwrap();
            !cyc_eq(g, cpu::fold_packed(&descs, y, 1))
        });
        assert!(moved, "the planted limb must change some fold");
        println!("PR-5 {src}: flipped limb {limb} of branch {branch} convicted at position {} -> (branch {}, limb {})", bad[0], convicted.0, convicted.1);
    }
    let _ = &mut batch;
}
