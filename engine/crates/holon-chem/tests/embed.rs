//! EMBED-1's gates on System 1 (`conformance/water_observatory/EMBED_PREREG.md` §3, §5):
//! G2 (the density is exact), G3 (Hellmann–Feynman), G4 (one fixed point), and the three
//! plants with their carriers asserted in the sector each acts on.

use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::*;
use holon_chem::pair::atomic_rms_radius;

fn sp(s: &str) -> Species {
    by_symbol(s).expect("species")
}

fn max_abs(v: impl Iterator<Item = f64>) -> f64 {
    v.fold(0.0, |m, x| m.max(x.abs()))
}

/// G2 on the free atoms: traces, symmetry, neutrality, and `sqrt(<r²>/N)` equal to
/// `atomic_rms_radius` — the same object by two routes.
#[test]
fn g2_density_free_atoms() {
    for sym in ["H", "O", "F"] {
        let s = sp(sym);
        let f = Fragment::new(vec![s], vec![[0.0; 3]], vec![0.0]);
        let m = monomer(&f, &[], ChargeModel::Mulliken);
        let n = m.solve.basis.n;
        let ne = s.n_electrons() as f64;
        assert!((trace(&m.gamma, n) - ne).abs() <= 1e-12, "{sym}: tr γ = {}", trace(&m.gamma, n));
        assert!((trace_product(&m.p, &m.mom.s, n) - ne).abs() <= 1e-12, "{sym}: tr PS");
        let asym = max_abs((0..n).flat_map(|p| (0..n).map(move |q| (p, q))).map(|(p, q)| m.gamma[p * n + q] - m.gamma[q * n + p]));
        assert!(asym <= 1e-12, "{sym}: γ asymmetric by {asym}");
        assert!(m.charges.iter().sum::<f64>().abs() <= 1e-12, "{sym}: Mulliken not neutral");
        let rms_density = (r2_expectation(&m.p, &m.mom) / ne).sqrt();
        let rms_record = atomic_rms_radius(s);
        let rel = ((rms_density - rms_record) / rms_record).abs();
        assert!(rel <= 1e-10, "{sym}: density route {rms_density} vs record {rms_record} (rel {rel:.2e})");
    }
}

/// G2 on the HF monomer, isolated and inside a test charge: traces and neutrality for
/// both charge models; the dipole-exact charges reproduce the dipole they were fit to.
#[test]
fn g2_density_hf_monomer() {
    let (r, _) = pin_hf(sp("F"), sp("H"));
    let f = Fragment::new(vec![sp("F"), sp("H")], vec![[0.0; 3], [0.0, 0.0, r]], vec![-1.0, 1.0]);
    for ext in [vec![], vec![External { r: [0.0, 0.0, r + 4.0], q: 0.3 }]] {
        let m = monomer(&f, &ext, ChargeModel::DipoleExact);
        let n = m.solve.basis.n;
        assert!((trace(&m.gamma, n) - 10.0).abs() <= 1e-12);
        assert!((trace_product(&m.p, &m.mom.s, n) - 10.0).abs() <= 1e-12);
        assert!(m.charges.iter().sum::<f64>().abs() <= 1e-12);
        assert!(m.charges_control.iter().sum::<f64>().abs() <= 1e-12);
        // the model's dipole is the density's, by construction, along the bond
        let model_mu_z = m.charges[1] * r;
        assert!((model_mu_z - m.dipole[2]).abs() <= 1e-12, "dipole-exact charges miss the dipole");
    }
}

/// G3: Hellmann–Feynman at six staked points around the HF monomer, and plants (i) and (iii).
#[test]
fn g3_hellmann_feynman_and_plants_i_iii() {
    let (r, grad) = pin_hf(sp("F"), sp("H"));
    assert!(grad.abs() <= 1e-6, "G1: HF pin gradient {grad}");
    let species = [sp("F"), sp("H")];
    let centers = [[0.0; 3], [0.0, 0.0, r]];
    let m = monomer(&Fragment::new(species.to_vec(), centers.to_vec(), vec![-1.0, 1.0]), &[], ChargeModel::DipoleExact);
    let gamma_plant = rdm1_plant_unsigned(&m.solve.gp.space, &m.solve.sol.vector);
    let n = m.solve.basis.n;
    let p_plant = ao_density(&gamma_plant, &m.solve.gp.orbitals, n);
    // plant (iii)'s trace must NOT fire: the trace is sign-blind
    assert!((trace(&gamma_plant, n) - 10.0).abs() <= 1e-12, "plant (iii) trace fired, which it must not");
    let points = [
        [0.0, 0.0, r + 3.0], [0.0, 0.0, r + 6.0], [0.0, 0.0, r + 12.0],
        [3.0, 0.0, 0.0], [6.0, 0.0, 0.0], [12.0, 0.0, 0.0],
    ];
    let h = 1e-4;
    let mut worst = 0.0f64;
    let mut carrier_v = 0.0f64;
    let mut carrier_iii = f64::INFINITY;
    for (k, pt) in points.iter().enumerate() {
        let v = potential_at(&m.p, &species, &centers, *pt);
        let ep = solve_embedded(&species, &centers, &[External { r: *pt, q: h }]).e_total;
        let em = solve_embedded(&species, &centers, &[External { r: *pt, q: -h }]).e_total;
        let fd = (ep - em) / (2.0 * h);
        worst = worst.max((fd - v).abs());
        assert!((fd - v).abs() <= 1e-7, "G3 point {k}: dE/dq {fd} vs V {v}");
        if k == 0 {
            carrier_v = v.abs();
            // plant (i): the field's sign flipped — the derivative disagrees by 2|V|
            let fd_plant = (em - ep) / (2.0 * h);
            assert!((fd_plant - v).abs() >= 2.0 * carrier_v - 1e-6, "plant (i) did not fire");
        }
        // plant (iii): the unsigned density's potential differs at every point
        let v_plant = potential_at(&p_plant, &species, &centers, *pt);
        carrier_iii = carrier_iii.min((v_plant - v).abs());
    }
    assert!(carrier_v >= 1e-3, "plant (i) carrier: |V| at the nearest point is {carrier_v}, not nonzero in the sector");
    assert!(carrier_iii >= 1e-6, "plant (iii) carrier: off-diagonal contribution {carrier_iii} not nonzero in the sector");
    eprintln!("G3 worst |dE/dq − V| = {worst:.3e}; plant (i) carrier |V| = {carrier_v:.4e}; plant (iii) carrier = {carrier_iii:.3e}");
}

/// G4: one fixed point from two starts, on a far and a near node; plant (ii)'s carrier.
#[test]
fn g4_one_fixed_point_and_plant_ii() {
    let (r, _) = pin_hf(sp("F"), sp("H"));
    for r_ff_a in [5.0, 2.8] {
        let (a, b) = hf_dimer(sp("F"), sp("H"), r, r_ff_a * ANGSTROM_TO_BOHR);
        let z = embed_pair(&a, &b, ChargeModel::DipoleExact, Start::Zero, false);
        let i = embed_pair(&a, &b, ChargeModel::DipoleExact, Start::Isolated, false);
        assert!(z.converged && i.converged, "G4 at {r_ff_a} Å: not converged ({} / {} iterations)", z.iterations, i.iterations);
        let dq = max_abs(z.q_a.iter().zip(i.q_a.iter()).chain(z.q_b.iter().zip(i.q_b.iter())).map(|(x, y)| x - y));
        assert!(dq <= 1e-8, "G4 at {r_ff_a} Å: charges differ by {dq}");
        assert!((z.e_emb - i.e_emb).abs() <= 1e-10, "G4 at {r_ff_a} Å: E_emb differs by {}", (z.e_emb - i.e_emb).abs());
        // plant (ii): the double count — differs from the honest energy by exactly e_qq
        let pl = embed_pair(&a, &b, ChargeModel::DipoleExact, Start::Zero, true);
        assert!(z.e_qq.abs() >= 1e-6, "plant (ii) carrier at {r_ff_a} Å: |e_qq| = {} not nonzero in the sector", z.e_qq);
        assert!(((pl.e_emb - z.e_emb) - z.e_qq).abs() <= 1e-12, "plant (ii) did not add back exactly e_qq");
        eprintln!("G4 {r_ff_a} Å: {} / {} iterations, q_a = {:?}, e_qq = {:.6e}", z.iterations, i.iterations, z.q_a, z.e_qq);
    }
}

/// The coupled size: on a free atom it IS `atomic_rms_radius`; on H₂ at its bond length
/// the populations sum to the electron count and the partitioned size differs from the
/// free atom's — the density responds to the bond, which is the whole point of the door.
#[test]
fn partitioned_sizes_are_the_record_on_an_atom_and_move_on_a_molecule() {
    for sym in ["H", "O", "F"] {
        let s = sp(sym);
        let f = Fragment::new(vec![s], vec![[0.0; 3]], vec![0.0]);
        let m = monomer(&f, &[], ChargeModel::Mulliken);
        let sizes = partitioned_sizes(&m.p, &m.solve.basis, &f.species, &f.centers);
        assert!((sizes[0].0 - s.n_electrons() as f64).abs() <= 1e-12, "{sym}: population");
        let rel = ((sizes[0].1 - atomic_rms_radius(s)) / atomic_rms_radius(s)).abs();
        assert!(rel <= 1e-10, "{sym}: partitioned size {} vs record {} (rel {rel:.2e})", sizes[0].1, atomic_rms_radius(s));
    }
    let h = sp("H");
    let f = Fragment::new(vec![h, h], vec![[0.0; 3], [0.0, 0.0, 1.4]], vec![-1.0, 1.0]);
    let m = monomer(&f, &[], ChargeModel::Mulliken);
    let sizes = partitioned_sizes(&m.p, &m.solve.basis, &f.species, &f.centers);
    let pop: f64 = sizes.iter().map(|x| x.0).sum();
    assert!((pop - 2.0).abs() <= 1e-12, "H2 populations sum to {pop}");
    assert!((sizes[0].0 - sizes[1].0).abs() <= 1e-12, "H2 is symmetric");
    let free = atomic_rms_radius(h);
    assert!((sizes[0].1 - free).abs() > 1e-3, "H2's partitioned size {} did not move from the free atom's {free}", sizes[0].1);
    eprintln!("coupled size: H free {free:.6} bohr, H in H2 at 1.4 bohr {:.6} bohr (population {:.6})", sizes[0].1, sizes[0].0);
}
