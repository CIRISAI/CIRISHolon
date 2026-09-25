//! QVM-RANK-1's plants and the instrument's own gates (prereg §4; `conformance/rank/`).
//!
//! PR-1 exact path load-bearing; PR-2 the exact stabilizer-ness test on the three-qubit
//! dictionary and on rejects; PR-3 the symmetry-reduced enumeration against brute and
//! Burnside counts at m = 3, 4; PR-4 each filter under its own premise keeps every term of
//! every known witness. PR-5 lives with the device (`holon-gpu/tests/rank1_twin.rs`).

use holon::ledger::Cyc;
use holon::stabrank::*;

fn known() -> Vec<KnownDec> {
    let text = include_str!("../../../../conformance/rank/known/qubit_H_known.txt");
    parse_known(text)
}

#[test]
fn dictionary_counts_and_roundtrips() {
    for n in 1..=4 {
        let d = enumerate_all(n);
        assert_eq!(d.len() as u128, count_states(n), "n = {n}");
        let set: std::collections::HashSet<&Stab> = d.iter().collect();
        assert_eq!(set.len(), d.len(), "canonical forms distinct at n = {n}");
        for s in &d {
            assert_eq!(Stab::from_phases(n, &s.phases()).as_ref(), Some(s));
        }
    }
    assert_eq!(count_states(3), 1080);
    assert_eq!(count_states(4), 36720);
    assert_eq!(count_states(7), 81_284_860_800);
    let per_k: u128 = (0..=7).map(|k| count_with_dim(7, k)).sum();
    assert_eq!(per_k, count_states(7));
}

#[test]
fn tableau_roundtrip_every_three_qubit_state() {
    for s in enumerate_all(3) {
        let t = s.to_tableau();
        assert_eq!(Stab::from_tableau(&t), s);
    }
    let mut rng = Rng::new(7);
    for _ in 0..200 {
        let s = Stab::random(7, &mut rng);
        assert_eq!(Stab::from_tableau(&s.to_tableau()), s);
    }
}

/// PR-2.
#[test]
fn pr2_exact_stabilizer_test() {
    let d = enumerate_all(3);
    assert_eq!(d.len(), 1080);
    let mut accepted = 0;
    for s in &d {
        // a nontrivial common scalar: (1 + ω)·s
        let c = Cyc { c: [1, 1, 0, 0], m: 0 };
        let v: Vec<Cyc> = s.cyc_vec().iter().map(|e| e.mul(c)).collect();
        assert_eq!(Stab::recognize_cyc(3, &v).as_ref(), Some(s));
        accepted += 1;
    }
    assert_eq!(accepted, 1080);
    assert!(Stab::recognize_cyc(3, &target_cyc(3)).is_none(), "|H>^3 rejected");
    let mut rng = Rng::new(20260924);
    let mut rejected = 0;
    let mut kinds = [0usize; 4];
    let mut tries = 0;
    while rejected < 1000 {
        tries += 1;
        let s = &d[rng.below(d.len())];
        let mut v = s.cyc_vec();
        let supp: Vec<usize> = (0..8).filter(|&x| v[x] != Cyc::ZERO).collect();
        let x = supp[rng.below(supp.len())];
        let kind = rng.below(4);
        match kind {
            0 => v[x] = v[x].mul(Cyc { c: [0, 1, 0, 0], m: 0 }), // an eighth-root phase
            1 => v[x] = v[x].mul(Cyc { c: [2, 0, 0, 0], m: 0 }), // a modulus
            2 => v[x] = Cyc::ZERO,                              // a support hole
            _ => {
                for e in v.iter_mut() {
                    let mut r = || rng.below(3) as i128 - 1;
                    *e = Cyc { c: [r(), r(), r(), r()], m: 0 };
                }
            }
        }
        // A plant can land on a genuine stabilizer ray by chance (a hole in a 2-point support,
        // a random vector that happens to be one). Those are not rejects: decide membership
        // independently, by proportionality to a dictionary state, and skip them.
        let is_dict = d.iter().any(|t| {
            let tv = t.cyc_vec();
            let fst = (0..8).find(|&i| tv[i] != Cyc::ZERO).unwrap();
            if holon::affine::cyc_is_zero(v[fst]) {
                return false;
            }
            (0..8).all(|i| holon::affine::cyc_eq(v[i].mul(tv[fst]), tv[i].mul(v[fst])))
        });
        if is_dict || v.iter().all(|e| holon::affine::cyc_is_zero(*e)) {
            continue;
        }
        assert!(Stab::recognize_cyc(3, &v).is_none(), "non-stabilizer accepted: {v:?}");
        rejected += 1;
        kinds[kind] += 1;
    }
    eprintln!("PR-2: 1080 accepted; |H>^3 rejected; 1000 rejects by kind (phase, modulus, hole, random) = {kinds:?} in {tries} draws");
    assert!(kinds.iter().all(|&c| c > 100));
}

/// PR-3.
#[test]
fn pr3_orbit_counts_m3_m4() {
    let d3 = enumerate_all(3);
    let g3 = symmetry_group(3);
    assert_eq!(g3.len(), 48);
    let a3 = action_table(&d3, &g3);
    for k in 1..=3 {
        let reduced = canonical_sets(&a3, d3.len(), k).len() as u128;
        let burn = burnside_subsets(&a3, k);
        eprintln!("PR-3 m=3 k={k}: reduced {reduced}, burnside {burn}");
        assert_eq!(reduced, burn);
        if k <= 2 {
            let brute = brute_orbit_count(&a3, d3.len(), k) as u128;
            eprintln!("PR-3 m=3 k={k}: brute {brute}");
            assert_eq!(reduced, brute);
        }
    }
    let d4 = enumerate_all(4);
    let g4 = symmetry_group(4);
    assert_eq!(g4.len(), 384);
    let a4 = action_table(&d4, &g4);
    let r1 = canonical_sets(&a4, d4.len(), 1).len() as u128;
    let b1 = burnside_subsets(&a4, 1);
    let x1 = brute_orbit_count(&a4, d4.len(), 1) as u128;
    eprintln!("PR-3 m=4 k=1: reduced {r1}, burnside {b1}, brute {x1}");
    assert_eq!(r1, b1);
    assert_eq!(r1, x1);
    let r2 = canonical_sets(&a4, d4.len(), 2).len() as u128;
    let b2 = burnside_subsets(&a4, 2);
    eprintln!("PR-3 m=4 k=2: reduced {r2}, burnside {b2}");
    assert_eq!(r2, b2);
}

/// Every known witness is an exact decomposition under OUR exact test (the reader is right
/// and the ring is right), and PR-4: each filter under its own premise keeps every term.
#[test]
fn known_witnesses_exact_and_pr4_filters_keep_them() {
    let ks = known();
    assert!(ks.len() >= 33);
    for (m, r, src, terms) in &ks {
        let w = exact_solve(terms).unwrap_or_else(|| panic!("{src}: not an exact decomposition"));
        assert!(w.accept_cyc(), "{src}: Cyc acceptance");
        for t in terms {
            assert!(passes_term_filter(t, *m, *r), "{src}: term filter removed a known term");
        }
        assert!(passes_tuple_slice_filter(terms, *m), "{src}: tuple slice filter");
        for drop in 0..*r {
            let sub: Vec<Stab> =
                terms.iter().enumerate().filter(|(i, _)| *i != drop).map(|(_, t)| t.clone()).collect();
            assert!(passes_galois_subset(&sub, *r), "{src}: Galois subset filter");
        }
    }
    // the filter's premise: at (7, 6) it is dual distance >= 3; at (6, 6) full along single
    // qubits only; at (4, 4) nothing per term — and the m = 6 witness violates P, as it may
    assert_eq!(full_slice_order(7, 6), 2);
    // (6, 5): the board's counting gives single-qubit fullness only; P at (6, 5) is PR 87's
    // table result, not a counting consequence
    assert_eq!(full_slice_order(6, 5), 1);
    assert_eq!(full_slice_order(6, 6), 1);
    assert_eq!(full_slice_order(4, 4), 0);
    let m6 = ks.iter().find(|k| k.0 == 6).unwrap();
    assert!(m6.3.iter().any(|t| t.dual_distance() < 3), "the QPG m=6 witness violates P, as it may");
}

/// PR-1.
#[test]
fn pr1_exact_path_is_load_bearing() {
    let ks = known();
    let mut done = 0;
    for (_, _, src, terms) in ks.iter().filter(|k| k.0 == 4 || k.0 == 6 || k.0 == 7) {
        let w = exact_solve(terms).unwrap();
        let alpha: Vec<Cyc> = (0..terms.len()).map(|j| w.alpha_cyc(j)).collect();
        assert!(w.accept_cyc_with(&alpha));
        assert!(w.float_residual_with(&alpha) < 1e-12);
        let mut bad = alpha.clone();
        // one ring unit, 2^{-50}, added to one coefficient
        bad[0] = bad[0].add(Cyc { c: [1, 0, 0, 0], m: 100 });
        let fr = w.float_residual_with(&bad);
        assert!(fr < 1e-12, "{src}: float check accepts the perturbed witness ({fr:e})");
        assert!(!w.accept_cyc_with(&bad), "{src}: exact check rejects it");
        done += 1;
    }
    eprintln!("PR-1: {done} known witnesses perturbed by 2^-50 in one coefficient: float accepts, Cyc rejects");
    assert!(done >= 32);
}

#[test]
fn completion_finds_known_terms() {
    // A pivot set P completes iff rank(P, a, b) = r (the quotient-basis condition); every
    // witness has such a P (the images of its terms span span/V). Check both facts.
    let ks = known();
    let mut good = 0;
    let mut degenerate = 0;
    for (m, r, src, terms) in ks.iter().filter(|k| k.0 == 4 || k.0 == 6) {
        let n = *m;
        let (a, b) = target_ab(n);
        let mut any = false;
        let idx: Vec<usize> = (0..*r).collect();
        for drop_i in 0..*r {
            for drop_j in drop_i + 1..*r {
                let piv: Vec<Stab> = idx.iter().filter(|&&t| t != drop_i && t != drop_j).map(|&t| terms[t].clone()).collect();
                let mut cols: Vec<Vec<Gi>> = piv.iter().map(|s| s.gi_vec()).collect();
                cols.push(a.iter().map(|&x| Gi::new(x, 0)).collect());
                cols.push(b.iter().map(|&x| Gi::new(x, 0)).collect());
                if bareiss_rank(&cols).0 < *r {
                    degenerate += 1;
                    continue;
                }
                let comp = complete_pivot(&piv);
                assert!(comp.contains(&terms[drop_i]) && comp.contains(&terms[drop_j]), "{src}: completion missed a term");
                any = true;
                good += 1;
            }
        }
        assert!(any, "{src}: no pivot subset spans a complement of V");
    }
    eprintln!("completion: {good} full-rank pivot subsets completed, {degenerate} degenerate skipped");
}

/// The slice lift is complete on a real target: the QPG rank-6 witness of |H>^6, sliced on
/// its top qubit, gives a rank-6 decomposition of |H>^5 whose lifts include the witness.
#[test]
fn lift_recovers_the_qpg_witness_from_its_slice() {
    let ks = known();
    let (_, _, _, terms) = ks.iter().find(|k| k.0 == 6).unwrap();
    for q in 0..6usize {
        // restrict every term to x_q = 0, re-indexing the other five qubits
        let slice: Vec<Stab> = terms
            .iter()
            .map(|s| {
                let ph = s.phases();
                let mut out = vec![ABSENT; 32];
                for (x, &e) in ph.iter().enumerate() {
                    if (x >> q) & 1 == 0 {
                        let lo = x & ((1 << q) - 1);
                        let hi = (x >> (q + 1)) << q;
                        out[lo | hi] = e;
                    }
                }
                Stab::from_phases(5, &out).expect("slice of a full-along-q term")
            })
            .collect();
        let base = exact_solve(&slice).expect("the slice is a rank-6 decomposition of |H>^5");
        let (lifts, st) = lift(&base, 11);
        // the original, with qubit q moved to the top, must be among the lifts
        let perm: Vec<u8> = (0..6u8).map(|c| if (c as usize) < q { c } else if c as usize == q { 5 } else { c - 1 }).collect();
        let mut want: Vec<Stab> = terms.iter().map(|s| s.permute(&perm)).collect();
        want.sort();
        let hit = lifts.iter().any(|w| {
            let mut t = w.terms.clone();
            t.sort();
            t == want
        });
        eprintln!("lift of the QPG slice on qubit {q}: {} lifts, {:?}", lifts.len(), st);
        assert!(hit, "the lift missed the witness it was sliced from (qubit {q})");
    }
}

/// The QPG rank-6 witness splits into two triples whose spans meet V = span(a, b) in two
/// independent RATIONAL lines (a Galois-stable span meets V in a σ-stable subspace; a line
/// is σ-stable iff its direction is Q(i)-rational). This is the shape `vsplit` searches.
#[test]
fn qpg_witness_is_a_v_split() {
    let ks = known();
    let (_, _, _, terms) = ks.iter().find(|k| k.0 == 6).unwrap();
    let mut dirs = Vec::new();
    for i in 0..6 {
        for j in i + 1..6 {
            for k in j + 1..6 {
                let t = vec![terms[i].clone(), terms[j].clone(), terms[k].clone()];
                if meets_v_exact(&t) {
                    let d = vmember_direction(&t);
                    eprintln!("QPG triple ({i},{j},{k}) meets V along {d:?}");
                    dirs.push(d);
                }
            }
        }
    }
    assert!(dirs.len() >= 2);
}

/// stabrank's stored list of 30 rank-4 decompositions of |H>^4 falls into the classes the
/// exhaustive pivot completion counts (23, PR 88's number) under S4 x H^(subset).
#[test]
fn stored_rank4_list_classes_under_the_full_group() {
    let ks = known();
    let g4 = symmetry_group(4);
    let mut keys = std::collections::HashSet::new();
    let mut n = 0;
    for (_, _, src, terms) in ks.iter().filter(|k| k.0 == 4 && k.1 == 4 && k.2.contains("m4_rank4.json")) {
        let _ = src;
        keys.insert(canonical_set_key(terms, &g4));
        n += 1;
    }
    eprintln!("stored rank-4 list: {n} decompositions, {} classes under S4 x H^(subset)", keys.len());
    assert_eq!(n, 30);
    assert_eq!(keys.len(), 23, "the exhaustive pivot completion (qvm_rank1 exhaust4) also finds 23");
}

/// The member census at m = 4 is planted on the QPG directions: its two member lines at m = 6,
/// 5a+7b and 7a+10b, and their slices T(d) = (2,3), (3,4), are rank <= 3 member directions two
/// and one qubits down, as the chain argument requires of any all-visible member.
#[test]
fn census_m4_contains_the_qpg_chain() {
    let dict = enumerate_all(4);
    let reps = orbit_reps(&dict, 4);
    assert_eq!(reps.len(), 246, "orbit reps = PR-3's orbit count");
    let hits = member_census(4, &dict, &reps, 1);
    let mut dirs = std::collections::HashSet::new();
    for h in &hits {
        assert!(meets_v_exact(&h.terms));
        dirs.insert(h.dir);
        dirs.insert(odd_h_dir(h.dir));
    }
    let g = |a: i128, b: i128| norm_dir((Gi::new(a, 0), Gi::new(b, 0)));
    for d in [g(5, 7), g(7, 10), g(2, 3), g(3, 4)] {
        assert!(dirs.contains(&d), "missing {d:?}");
    }
    assert_eq!(slice_t(g(5, 7)), g(2, 3));
    assert_eq!(slice_t(g(7, 10)), g(3, 4));
}

/// `lift_member` is complete on a real member: each QPG triple (a rank-3 member of V_6),
/// sliced on its top qubit, lifts back to itself along its own direction.
#[test]
fn lift_member_recovers_the_qpg_triples() {
    let ks = known();
    let (_, _, _, terms) = ks.iter().find(|k| k.0 == 6).unwrap();
    for tri in [[0usize, 1, 2], [3, 4, 5]] {
        let t: Vec<Stab> = tri.iter().map(|&i| terms[i].clone()).collect();
        let d = norm_dir(vmember_direction(&t));
        let slice: Vec<Stab> = t
            .iter()
            .map(|s| {
                let ph = s.phases();
                Stab::from_phases(5, &ph[..32]).expect("full along qubit 5")
            })
            .collect();
        assert!(meets_v_exact(&slice));
        assert_eq!(norm_dir(vmember_direction(&slice)), d, "the 0-slice keeps the direction");
        let lifts = lift_member(&slice, d);
        let mut want = t.clone();
        want.sort();
        let hit = lifts.iter().any(|l| {
            let mut x = l.clone();
            x.sort();
            x == want
        });
        eprintln!("QPG triple {tri:?} direction {d:?}: {} lifts of its slice", lifts.len());
        assert!(hit);
    }
}
