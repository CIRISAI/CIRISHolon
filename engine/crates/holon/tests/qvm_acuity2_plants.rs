//! QVM-ACUITY-2's plants, PQ-1 … PQ-5 (`conformance/qasm/QVM_ACUITY2_PREREG.md` §4), each of
//! which must fire before any instance is read. Run with `--nocapture` for the numbers.
#![allow(clippy::needless_range_loop)]

use holon::affine::Gate;
use holon::sector::{self, Observable};
use holon::views::{self, Mps, MpsStop, ProbeMode, Query, View, Z};

fn marginal_of(y: &[bool]) -> Observable {
    Observable::Marginal { qubits: vec![0, 1, 2, 3], bits: y[..4].to_vec() }
}

/// PQ-1 — a Clifford-only circuit presented with a T-gate the light cone removes: TABLEAU
/// selected, `t_eff = 0` printed. Carrier: a random Clifford circuit on wires 0‥7 and a disjoint
/// block on wires 8‥11 that carries two T-gates, read through the marginal on wires 0‥3.
#[test]
fn pq1_a_removable_t_leaves_the_tableau_selected() {
    let k = views::calibrate();
    for seed in 1..=5u64 {
        let mut gates = sector::random_instance(8, 0, seed).gates;
        gates.extend([Gate::H(11), Gate::T(11), Gate::H(11), Gate::Cx(10, 11), Gate::T(10), Gate::H(9), Gate::Cx(9, 8)]);
        let y = sector::argmax_bitstring(12, &gates);
        let q = Query { n: 12, gates: gates.clone(), obs: marginal_of(&y), eps: 0.1 / 16.0, label: None };
        let s = views::select(&q, &k, ProbeMode::Cadence);
        println!("PQ-1 seed {seed}: {}  |  {}", s.sector_line, s.line);
        assert!(s.sector_line.contains("hard=0"), "t_eff must print as 0: {}", s.sector_line);
        assert_eq!(s.t_eff, 0);
        assert!(s.sector_line.contains("removed=2"), "both T-gates removed: {}", s.sector_line);
        assert_eq!(s.selected, View::Tableau, "{}", s.line);
        let run = views::run_view(&q, View::Tableau, q.eps, None, MpsStop::Never);
        let referee = sector::referee::value(12, &gates, &q.obs);
        let e = run.value.unwrap().abs_diff(referee);
        assert!(e < 1e-12, "tableau vs referee {e}");
    }
}

/// PQ-2 — family L with the brickwork raised to depth `20n`: the probe's discarded weight exceeds
/// the acuity's share at every `χ ≤ 16` and MPS is NOT selected. Read under the AMENDED probe
/// (cadence through the whole circuit), which is the reading the campaign runs.
#[test]
fn pq2_saturated_brickwork_opens_the_mps_view_under_the_cadence_probe() {
    let k = views::calibrate();
    for (i, n) in [12usize, 16, 20].into_iter().enumerate() {
        let c = views::brickwork(n, 20 * n, 40 + i, 77 + i as u64);
        let y = sector::argmax_bitstring(n, &c.gates);
        for (obs, scale) in [
            (Observable::Amplitude(y.clone()), (2f64).powf(-(n as f64) / 2.0)),
            (marginal_of(&y), 1.0 / 16.0),
        ] {
            for eps_rel in [1e-1, 1e-2] {
                let q = Query { n, gates: c.gates.clone(), obs: obs.clone(), eps: eps_rel * scale, label: None };
                let s = views::select(&q, &k, ProbeMode::Cadence);
                let proj: Vec<String> = s.probe.at.iter().map(|a| format!("chi={} {:.2e}", a.chi, a.projected)).collect();
                println!("PQ-2 n={n} {} eps={:.1e}: probe [{}] -> {}", obs.describe().chars().take(9).collect::<String>(), q.eps, proj.join(", "), s.line);
                assert_eq!(s.probe.at.len(), 4, "every chi <= 16 must be probed");
                for a in &s.probe.at {
                    assert!(a.projected > q.eps, "chi={} closed on a saturated brickwork", a.chi);
                }
                assert!(s.probe.chi.is_none());
                assert_ne!(s.selected, View::Mps);
            }
        }
    }
}

/// PQ-2 as FROZEN — the prefix probe (the first `2n` gates) on the same saturated brickwork. This
/// pins the finding that sent the probe to Amendment 1: `2n` gates of an `n`-wide brickwork is
/// barely one layer, the prefix discards nothing, and the frozen probe CLOSES at `χ = 2` — the
/// plant fails under the frozen reading. Asserted so a change to it is seen.
#[test]
fn pq2_as_frozen_the_prefix_probe_is_blind_to_depth() {
    for n in [12usize, 16, 20] {
        let c = views::brickwork(n, 20 * n, 40, 5);
        let y = sector::argmax_bitstring(n, &c.gates);
        let q = Query { n, gates: c.gates.clone(), obs: Observable::Amplitude(y), eps: 0.1 * (2f64).powf(-(n as f64) / 2.0), label: None };
        let p = views::probe(&q, ProbeMode::Prefix, None);
        println!(
            "PQ-2 frozen n={n}: prefix probe closed at chi={:?}, measured {:.2e} over {} gates",
            p.chi, p.at[0].measured, p.at[0].gates_probed
        );
        assert_eq!(p.chi, Some(2), "the frozen prefix probe was expected to close at chi = 2");
        let full = views::mps_run(&q, 2, q.gates.len(), MpsStop::Never, None);
        println!("   the same chi = 2 over the whole circuit: certificate {:.3e} against eps {:.1e}", full.certificate, q.eps);
        assert!(full.certificate > q.eps);
    }
}

/// PQ-3 — the MPS certificate on a state with a KNOWN Schmidt spectrum. Carrier: `n = 8`, four
/// pairs, pair `j` prepared on adjacent wires as `CX · (HTH ⊗ 1)|00⟩` (Schmidt weights
/// `cos²(π/8), sin²(π/8)`) or `CX · (H ⊗ 1)` (`½, ½`), then pulled apart by a SWAP network (each
/// SWAP three nearest-neighbour CX) until every pair straddles the middle cut, which then carries
/// the product spectrum of all four. (Routed non-adjacent CX would NOT do: the routing carries the
/// qubit map, so a partner is walked over while still a product state and nothing truncates —
/// measured, the first carrier this plant was given.) Truncations happen inside the network at
/// several bonds. At every cap `χ ∈ {1,2,4,8}` the certificate `Σ√w` is never
/// below the true 2-norm error (the dense referee) and within `4×` of it; at the middle cut the
/// untruncated spectrum is checked against the analytic one.
#[test]
fn pq3_the_certificate_bounds_the_true_error_on_a_known_spectrum() {
    let n = 8;
    let biased = [true, false, true, true];
    let mut gates = Vec::new();
    // pairs made ADJACENT, on wires (2j, 2j+1) — no truncation can happen here …
    for (j, &b) in biased.iter().enumerate() {
        let a = 2 * j;
        if b {
            gates.extend([Gate::H(a), Gate::T(a), Gate::H(a)]);
        } else {
            gates.push(Gate::H(a));
        }
        gates.push(Gate::Cx(a, a + 1));
    }
    // … then pulled apart by nearest-neighbour SWAPs spelled as three CX each, so every pair ends
    // up straddling the middle cut: [a0 b0 a1 b1 a2 b2 a3 b3] -> [a0 a1 a2 a3 b0 b1 b2 b3]
    for w in [1usize, 3, 2, 5, 4, 3] {
        gates.extend([Gate::Cx(w, w + 1), Gate::Cx(w + 1, w), Gate::Cx(w, w + 1)]);
    }
    let exact = sector::referee::statevector(n, &gates);
    // the analytic middle-cut spectrum
    let pb = (std::f64::consts::PI / 8.0).cos().powi(2);
    let mut spec: Vec<f64> = vec![1.0];
    for &b in &biased {
        let (a, c) = if b { (pb, 1.0 - pb) } else { (0.5, 0.5) };
        spec = spec.iter().flat_map(|&x| [x * a, x * c]).collect();
    }
    spec.sort_by(|a, b| b.total_cmp(a));
    let mut worst_ratio: f64 = 0.0;
    for chi in [1usize, 2, 4, 8, 16] {
        let (ops, _) = views::route(n, &gates);
        let mut m = Mps::zero_state(n, chi);
        for op in ops {
            m.apply_op(op);
        }
        let d = m.to_dense();
        let err = exact
            .iter()
            .zip(&d)
            .map(|(a, b)| (Z::new(a.0, a.1) - *b).norm2())
            .sum::<f64>()
            .sqrt();
        let analytic_single_cut: f64 = spec[chi.min(16)..].iter().sum::<f64>().sqrt();
        println!(
            "PQ-3 chi={chi:2}: certificate {:.6e}  true 2-norm error {:.6e}  ratio {:.3}  truncations {}  (one ideal cut at the middle would cost {:.6e})",
            m.delta,
            err,
            if err > 0.0 { m.delta / err } else { f64::NAN },
            m.discarded.len(),
            analytic_single_cut
        );
        assert!(m.delta + 1e-12 >= err, "chi={chi}: the certificate {:.3e} is BELOW the true error {:.3e}", m.delta, err);
        if chi < 16 {
            assert!(err > 1e-6, "the plant must truncate at chi={chi}");
            let r = m.delta / err;
            worst_ratio = worst_ratio.max(r);
            assert!(r <= 4.0, "chi={chi}: certificate {:.3e} is more than 4x the true error {:.3e}", m.delta, err);
        } else {
            assert!(err < 1e-12 && m.delta < 1e-12, "chi = 16 holds the whole state");
        }
    }
    // the untruncated state's middle-cut Schmidt spectrum IS the analytic one
    let mut psi = Mps::zero_state(n, 64);
    for op in views::route(n, &gates).0 {
        psi.apply_op(op);
    }
    let mut mat = views::Mat::zeros(16, 16);
    for x in 0..256usize {
        let (l, r) = (x & 15, x >> 4);
        let a = exact[x];
        mat.a[l * 16 + r] = Z::new(a.0, a.1);
    }
    let (_, s, _) = views::svd(&mat);
    for (j, (&sv, &w)) in s.iter().zip(&spec).enumerate() {
        assert!((sv * sv - w).abs() < 1e-12, "Schmidt weight {j}: {} vs analytic {w}", sv * sv);
    }
    println!("PQ-3 worst certificate/true-error ratio over chi in {{1,2,4,8}}: {worst_ratio:.3}");
}

/// PQ-4 — a wrong price planted on one view (its predicted cost halved): the selection flips on
/// exactly the instances where that view was second and the halving crosses the first price, and
/// on no other; the walls of the flipped selections are measured for S2.
#[test]
fn pq4_a_halved_price_flips_the_runner_up() {
    let k = views::calibrate();
    let mut insts = Vec::new();
    for f in ['C', 'T', 'L', 'D'] {
        for i in [0usize, 1, 3, 4] {
            insts.push(views::family_instance(f, i, 11));
        }
    }
    let sels: Vec<_> = insts.iter().map(|x| views::select(&x.query(), &k, ProbeMode::Cadence)).collect();
    // the planted view: the most frequent runner-up
    let mut counts = [0usize; 4];
    for s in &sels {
        if let Some(v) = s.ranking().get(1) {
            counts[v.index()] += 1;
        }
    }
    let planted = View::ALL[(0..4).max_by_key(|&i| counts[i]).unwrap()];
    let mut scale = [1.0; 4];
    scale[planted.index()] = 0.5;
    let mut flips = 0;
    for (x, s) in insts.iter().zip(&sels) {
        let s2 = views::select_scaled(&x.query(), &k, ProbeMode::Cadence, scale);
        let rank = s.ranking();
        let should_flip = rank.len() > 1 && rank[1] == planted && 0.5 * s.price_of(planted) < s.price_of(rank[0]);
        let expected = if should_flip { planted } else { s.selected };
        assert_eq!(s2.selected, expected, "{}{}: planted {:?}", x.family, x.index, planted);
        if should_flip {
            flips += 1;
            let w_new = views::run_view(&x.query(), planted, x.eps, s2.probe.chi, MpsStop::Never).wall;
            let w_old = views::run_view(&x.query(), s.selected, x.eps, s.probe.chi, MpsStop::Never).wall;
            println!(
                "PQ-4 {}{} n={}: {} -> {} (price ratio {:.2}); wall {:.3e} s against {:.3e} s, ratio {:.2}",
                x.family,
                x.index,
                x.n,
                s.selected.name(),
                planted.name(),
                s.price_of(planted) / s.price_of(s.selected),
                w_new,
                w_old,
                w_new / w_old
            );
        }
    }
    println!("PQ-4 planted on {}: {flips} flips of {}", planted.name(), insts.len());
    assert!(flips > 0, "a plant that flips nothing tests nothing");
}

/// PQ-5 — the selection with the family label REVEALED equals the selection without it, on every
/// instance, to the bit of every price.
#[test]
fn pq5_the_family_label_is_never_read() {
    let k = views::calibrate();
    let mut n = 0;
    for f in ['C', 'T', 'L', 'D'] {
        for i in 0..6 {
            let x = views::family_instance(f, i, 3);
            let a = views::select(&x.query(), &k, ProbeMode::Cadence);
            let b = views::select(&x.query_labelled(), &k, ProbeMode::Cadence);
            assert_eq!(a.selected, b.selected);
            assert_eq!(a.probe.chi, b.probe.chi);
            for (p, r) in a.priced.iter().zip(&b.priced) {
                assert_eq!(p.closed, r.closed);
                assert_eq!(p.price.to_bits(), r.price.to_bits());
            }
            n += 1;
        }
    }
    println!("PQ-5: {n} instances, the revealed label changed nothing");
}
