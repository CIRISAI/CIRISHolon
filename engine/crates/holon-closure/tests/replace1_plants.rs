//! REPLACE-1's plants and small-data stakes (`conformance/replace1/REPLACE1_PREREG.md`).
//!
//! Run in CI (no data outside the repository):
//! * PR-1..PR-4 on a SYNTHETIC hydrodynamic carrier (OU density and current modes at the walks'
//!   cadence) — the plants' logic, re-derived for `removable`, on a carrier CI can build;
//! * G4 at n = 12 and 16 — the light-cone removal through `removable` gives the removed set the
//!   cone's own loop gives, and every removed gate moves the marginal by `≤ 10⁻¹²` on the referee;
//! * G5 on the synthetic carrier — refusal by name and price, and the defect rising by the price.
//!
//! `#[ignore]`d, the WALK-SIZED reads, whose record is the driver's output
//! (`conformance/replace1/replace1_read.txt`, from `examples/replace1`): PR-1..PR-4 on the
//! carrier of record (`slow1/T293_seed0`), PR-5 and G1 on the RESPONSE-1 arms, G2's ORDER-1
//! reproduction on the three 293 K walks, G4 at n = 20. The walks live in the main checkout's
//! `conformance/water_observatory/replace0/` (not in git); set `REPLACE1_WALKS` to point
//! elsewhere. Run by hand:
//! `cargo test --release -p holon-closure --test replace1_plants -- --ignored`.

use holon::sector::{self, referee, Observable};
use holon_closure::removable::{
    self, admit, refuse_drop, Candidate, Folds, Horizon, Mat, Nulls, NumpyPcg64, Question, Ridge, Swap, Verdict,
};
use holon_lens::staggered::{self, FluidChart};
use holon_lens::walk::{self, RigidWalk, C_LS, C_Q, C_STRUCT};
use std::path::PathBuf;

const BETA: f64 = 0.02;

fn fluid_q(lag: usize) -> Question {
    Question { lag, horizon: Horizon::Future, folds: Folds::TimeBlocks { k: 5 }, ridge: Ridge::ORDER1 }
}

/// A synthetic hydrodynamic sector at the SLOW-1 cadence (20 fs, 2501 readouts): each density
/// mode a damped oscillator with a sub-picosecond memory, each current an OU of 0.3 ps, and an
/// unrelated "structural" field of 1 ps; columns in `fourier_features`' layout.
fn synthetic_carrier(seed: u64) -> (Vec<[f64; 24]>, Vec<[f64; 6]>) {
    let t_len = 2501;
    let mut rng = NumpyPcg64::new(seed);
    let rho = removable::ou_field(t_len, 6, 25.0, &mut rng); // 0.5 ps
    let cur = removable::ou_field(t_len, 18, 15.0, &mut rng); // 0.3 ps
    let q = removable::ou_field(t_len, 6, 50.0, &mut rng); // 1 ps
    let mut h = vec![[0.0f64; 24]; t_len];
    let others: Vec<usize> = (0..24).filter(|c| !walk::FOURIER_DENSITY.contains(c)).collect();
    for (t, row) in h.iter_mut().enumerate() {
        for (j, &c) in walk::FOURIER_DENSITY.iter().enumerate() {
            row[c] = 3.0 * rho.at(t, j);
        }
        for (j, &c) in others.iter().enumerate() {
            row[c] = 1e-3 * cur.at(t, j);
        }
    }
    let mut qq: Vec<[f64; 6]> = (0..t_len).map(|t| [0, 1, 2, 3, 4, 5].map(|j| 0.8 * q.at(t, j))).collect();
    walk::centre(&mut h);
    walk::centre(&mut qq);
    (h, qq)
}

/// The four fluid plants on a carrier: `(h, q)` and the lags in readouts. Returns
/// `(pr1, pr2, pr3, pr4, pr1_increment)`.
fn fluid_plants(h: &[[f64; 24]], q: &[[f64; 6]], lags: &[usize], plant_seed: u64) -> (bool, bool, bool, bool, f64) {
    let (hp, qp) = walk::plant_po4(h, q, 250.0, 50, 0.05, 1.0, plant_seed);
    let ch = walk::order_chains(&hp, &[qp, q.to_vec()]); // [H(8) | planted(2) | Q(2)]
    let planted = [8usize, 9];
    let (mut pr1, mut pr2, mut pr3, mut pr4) = (false, true, true, true);
    let mut best = f64::NEG_INFINITY;
    let perm = NumpyPcg64::new(11).permutation(ch[0].rows);
    let sh: Vec<Mat> = ch.iter().map(|c| c.permute_rows(&perm)).collect();
    for &l in lags {
        let us = walk::units(&ch, &C_LS, &C_LS);
        // PR-1 — admitted, with the joint pass region (carried at beta, refused at 0)
        let adm = admit(&us, &[Candidate::new("planted", walk::cand(&ch, &planted))], &fluid_q(l), &Nulls { shift: true, swap: Some(Swap::Derange { shift: 2 }) }, BETA);
        let p = &adm.prices[0];
        best = best.max(p.increment);
        if p.increment >= 0.05 && p.null_max().unwrap() <= 0.01 && p.verdict == Verdict::Carried && refuse_drop(&adm, "planted", 0.0).is_err() {
            pr1 = true;
        }
        // PR-2 — the same field from another wavevector's chain
        let sw: Vec<Mat> = (0..ch.len()).map(|u| ch[(u + 2) % ch.len()].select_cols(&planted)).collect();
        let a2 = admit(&us, &[Candidate::new("swapped", sw)], &fluid_q(l), &Nulls { shift: false, swap: None }, BETA);
        pr2 &= a2.prices[0].verdict == Verdict::Dropped && a2.prices[0].increment <= 0.01;
        // PR-3 — an affine copy of kept columns
        let cp: Vec<Mat> = ch
            .iter()
            .map(|c| Mat::from_rows(&(0..c.rows).map(|t| vec![2.0 * c.at(t, 0) - 3.0, 0.5 * c.at(t, 1) - 2.0 * c.at(t, 3) + 1.0]).collect::<Vec<_>>()))
            .collect();
        let a3 = admit(&us, &[Candidate::new("copy", cp)], &fluid_q(l), &Nulls { shift: false, swap: None }, BETA);
        pr3 &= a3.prices[0].verdict == Verdict::Dropped && (0.0..=1e-10).contains(&a3.prices[0].increment);
        // PR-4 — the time-shuffled dictionary admits nothing
        let ush = walk::units(&sh, &C_LS, &C_LS);
        for cols in [planted.to_vec(), vec![10, 11]] {
            let a4 = admit(&ush, &[Candidate::new("shuffled", walk::cand(&sh, &cols))], &fluid_q(l), &Nulls { shift: true, swap: None }, BETA);
            pr4 &= a4.carried.is_empty() && a4.prices[0].increment <= 0.01;
        }
    }
    (pr1, pr2, pr3, pr4, best)
}

#[test]
fn plants_fire_on_a_synthetic_carrier() {
    let (h, q) = synthetic_carrier(21);
    let (pr1, pr2, pr3, pr4, inc) = fluid_plants(&h, &q, &[50, 250], 5);
    assert!(pr1, "PR-1: the planted carried field was not admitted (best increment {inc:+.4})");
    assert!(pr2, "PR-2: the partner-swapped field was not dropped");
    assert!(pr3, "PR-3: an affine copy of a kept column was not dropped at exactly zero");
    assert!(pr4, "PR-4: the time-shuffled dictionary admitted something");
}

#[test]
fn g5_refuses_by_name_and_the_defect_rises_by_the_price() {
    let (h, q) = synthetic_carrier(22);
    let (hp, qp) = walk::plant_po4(&h, &q, 250.0, 50, 0.05, 1.0, 6);
    let ch = walk::order_chains(&hp, &[qp]);
    let us = walk::units(&ch, &[8, 9], &C_LS);
    let adm = admit(&us, &[Candidate::new("hydrodynamic block", walk::cand(&ch, &C_LS))], &fluid_q(50), &Nulls { shift: true, swap: None }, BETA);
    let p = adm.prices[0].clone();
    assert!(p.increment > 0.0);
    let r = refuse_drop(&adm, "hydrodynamic block", 0.0).expect_err("budget 0 must refuse");
    assert_eq!(r.name, "hydrodynamic block");
    assert_eq!(r.price, p.increment);
    assert!(format!("{r}").starts_with("REFUSED: dropping 'hydrodynamic block'"));
    let receipt = refuse_drop(&adm, "hydrodynamic block", p.increment + 0.01).expect("above the price it drops");
    assert_eq!(receipt.price, p.increment);
    let full = walk::units(&ch, &[8, 9, 0, 1, 2, 3], &C_LS);
    let rise = removable::closure_r2(&full, &fluid_q(50)) - removable::closure_r2(&us, &fluid_q(50));
    assert!((rise - p.increment).abs() < 1e-12, "defect rise {rise} vs price {}", p.increment);
}

fn reference_removed(circuit: &[holon::affine::Gate], obs: &Observable) -> Vec<usize> {
    let n = sector::width(circuit, obs);
    let mut cone = vec![false; n];
    for q in obs.support(n) {
        cone[q] = true;
    }
    let mut keep = vec![false; circuit.len()];
    for i in (0..circuit.len()).rev() {
        let s = sector::support(circuit[i]);
        if s.iter().any(|&q| cone[q]) {
            for q in s {
                cone[q] = true;
            }
            keep[i] = true;
        }
    }
    (0..circuit.len()).filter(|&i| !sector::closure_of(circuit[i]).is_closed() && !keep[i]).collect()
}

/// G4 on one width of ACUITY-1's grid: returns (removed total, worst change).
fn g4_width(n: usize) -> (usize, f64) {
    let (mut total, mut worst) = (0usize, 0.0f64);
    let obs = Observable::Marginal { qubits: vec![0, 1, 2, 3], bits: vec![false; 4] };
    for t in [8usize, 12, 16] {
        for seed in [1u64, 2, 3] {
            let gates = sector::random_instance_depth(n, t, seed, 20 * n).gates;
            let got: Vec<usize> = sector::locate(&gates, &obs).removed.iter().map(|(i, _)| *i).collect();
            assert_eq!(got, reference_removed(&gates, &obs), "G4 KILL: removed sets differ at n={n} t={t} seed={seed}");
            let adm = sector::light_cone_admission(&gates, &obs);
            assert_eq!(adm.dropped.iter().map(|s| s.parse().unwrap()).collect::<Vec<usize>>(), got);
            let want = referee::marginal(&referee::statevector(n, &gates), &[0, 1, 2, 3], &[false; 4]);
            for &k in &got {
                let mut cut = gates.clone();
                cut.remove(k);
                let d = (referee::marginal(&referee::statevector(n, &cut), &[0, 1, 2, 3], &[false; 4]) - want).abs();
                assert!(d <= sector::REMOVAL_BUDGET, "a removed gate moved the marginal by {d}");
                worst = worst.max(d);
            }
            total += got.len();
        }
    }
    (total, worst)
}

#[test]
fn g4_the_light_cone_removal_is_the_gate_n12_n16() {
    let (a, wa) = g4_width(12);
    let (b, wb) = g4_width(16);
    // measured by the driver on this grid: 21 at n = 12, 29 at n = 16 (26 more at n = 20: 76)
    assert_eq!((a, b), (21, 29));
    assert!(wa.max(wb) <= 1e-12);
}

// ------------------------------------------------------------------ walk-sized reads (ignored)

fn walks() -> PathBuf {
    let p = std::env::var("REPLACE1_WALKS").unwrap_or_else(|_| "/home/emoore/CIRISHolon/conformance/water_observatory/replace0".into());
    let p = PathBuf::from(p);
    assert!(p.join("slow1").exists(), "the walks are not at {} (set REPLACE1_WALKS)", p.display());
    p
}

#[test]
#[ignore = "walk-sized: G4 at n = 20 (2^20 statevectors); the record is the driver's output"]
fn g4_n20() {
    let (c, w) = g4_width(20);
    assert_eq!(c, 26);
    assert!(w <= 1e-12);
}

#[test]
#[ignore = "walk-sized: PR-5 and G1 on the RESPONSE-1 arms (data outside git); the record is the driver's output"]
fn pr5_and_g1_on_the_response1_arms() {
    let root = walks();
    let mut arms = Vec::new();
    for k in 0..3 {
        let w = RigidWalk::load(&root.join(format!("response1_L200_seed{k}")), None).unwrap();
        arms.push([4usize, 8, 16].map(|nx| staggered::closure_read(&w, nx, &FluidChart::r1_rows(), 12, 314, 2)));
    }
    let mine: String = arms[0].iter().map(|g| format!("\n{}", staggered::format_grid(g, None))).collect();
    let banked = std::fs::read_to_string(root.join("response1_L200_seed0/r1_closure_test.txt")).unwrap();
    let banked: String = banked.lines().skip(1).map(|l| format!("{l}\n")).collect();
    assert_eq!(mine, banked, "PR-5: the per-arm table differs from r1_closure_test.txt");
    let pooled = staggered::pool(&[&arms[0][1], &arms[1][1], &arms[2][1]]);
    let row = pooled.rows.iter().find(|r| r.chart == FluidChart::OF_RECORD).unwrap();
    let (se, _) = staggered::jackknife(row);
    assert!((row.d_lead - 0.180).abs() <= 0.01, "G1: D = {}", row.d_lead);
    assert!(row.d_lead <= 0.2 && (0.8..=1.25).contains(&row.alpha));
    assert!((se - 0.058).abs() < 0.0005);
}

#[test]
#[ignore = "walk-sized: G2's ORDER-1 reproduction and PR-1..PR-4 on the carrier of record; the record is the driver's output"]
fn g2_and_plants_on_the_slow1_walks() {
    let root = walks();
    let banked = [[-0.0032, -0.0126], [-0.0007, -0.0049], [-0.0033, -0.0089]];
    let feats: Vec<_> = (0..3)
        .map(|k| walk::order_features(&RigidWalk::load(&root.join(format!("slow1/T293_seed{k}")), None).unwrap()))
        .collect();
    for (k, f) in feats.iter().enumerate() {
        let ch = walk::order_chains(&f.h, &f.fields);
        for (li, l) in [50usize, 250].iter().enumerate() {
            let us = walk::units(&ch, &C_LS, &C_LS);
            let donor = walk::order_chains(&feats[(k + 1) % 3].h, &feats[(k + 1) % 3].fields);
            let cands = [Candidate::new("Q", walk::cand(&ch, &C_Q)), Candidate::new("structural block", walk::cand(&ch, &C_STRUCT))];
            for c in &cands {
                let cols: Vec<usize> = if c.name == "Q" { C_Q.to_vec() } else { C_STRUCT.to_vec() };
                let adm = admit(&us, std::slice::from_ref(c), &fluid_q(*l), &Nulls { shift: true, swap: Some(Swap::Donor(walk::cand(&donor, &cols))) }, BETA);
                let p = &adm.prices[0];
                assert_eq!(p.verdict, Verdict::Dropped, "G2 KILL: {p}");
                assert!(p.increment <= 0.01 && p.null_max().unwrap() <= 0.01, "{p}");
                if c.name == "Q" {
                    assert!((p.increment - banked[k][li]).abs() <= 0.005, "G2: {p} against banked {}", banked[k][li]);
                }
            }
        }
    }
    let (pr1, pr2, pr3, pr4, inc) = fluid_plants(&feats[0].h, &feats[0].fields[0], &[50, 250], 5);
    assert!(pr1 && pr2 && pr3 && pr4, "plants on the carrier of record: {pr1} {pr2} {pr3} {pr4} (PR-1 {inc:+.4})");
}
