//! SEAM-1's gates that need no trimer solve (`SEAM_PREREG.md` §3, §5): G2 (one fixed point
//! for three), G3 (zero charges reduce the embedded machinery to the bare one exactly), and
//! both plants' carriers, at a far and a near node of the HF chain.

use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::*;
use holon_chem::seam::*;

fn sp(s: &str) -> Species {
    by_symbol(s).expect("species")
}

const R_HF: f64 = 1.879437929774;

#[test]
fn g1_the_pin_is_embed1s() {
    let (r, g) = pin_hf(sp("F"), sp("H"));
    assert!((r - R_HF).abs() <= 1e-8, "pin moved: {r}");
    assert!(g.abs() <= 1e-6);
}

#[test]
fn g2_one_fixed_point_for_three_and_g3_the_identity() {
    for r_ang in [5.0, 3.0] {
        let frags = hf_chain(sp("F"), sp("H"), R_HF, r_ang * ANGSTROM_TO_BOHR, 3);
        let z = embed_many(&frags, ChargeModel::DipoleExact, Start::Zero);
        let i = embed_many(&frags, ChargeModel::DipoleExact, Start::Isolated);
        assert!(z.converged && i.converged, "G2 at {r_ang} Å: {} / {} sweeps", z.iterations, i.iterations);
        let dq = z.charges.iter().flatten().zip(i.charges.iter().flatten()).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
        assert!(dq <= 1e-8, "G2 at {r_ang} Å: charges differ by {dq}");
        let ez = ee_pa(&frags, &z.charges);
        let ei = ee_pa(&frags, &i.charges);
        assert!((ez.total - ei.total).abs() <= 1e-10, "G2 at {r_ang} Å: E_EE-PA differs by {}", (ez.total - ei.total).abs());
        // G3: every charge zero — the embedded sum IS the bare sum
        let zeros: Vec<Vec<f64>> = frags.iter().map(|f| vec![0.0; f.species.len()]).collect();
        let e0 = ee_pa(&frags, &zeros);
        let bare = bare_pa(&frags);
        assert!((e0.total - bare.total).abs() <= 1e-12, "G3 at {r_ang} Å: {} vs {}", e0.total, bare.total);
        eprintln!("G2 {r_ang} Å: {} / {} sweeps, q = {:?}; G3 |Δ| = {:.2e}", z.iterations, i.iterations,
            z.charges.iter().map(|q| q[1]).collect::<Vec<_>>(), (e0.total - bare.total).abs());
    }
}

#[test]
fn plants_i_and_ii_carriers_are_nonzero_in_their_sector() {
    for r_ang in [5.0, 6.0, 8.0] {
        let frags = hf_chain(sp("F"), sp("H"), R_HF, r_ang * ANGSTROM_TO_BOHR, 3);
        let z = embed_many(&frags, ChargeModel::DipoleExact, Start::Zero);
        // (i): the field's effect on the pair AB (the third's charges on the first two)
        let with = ee_pa(&frags, &z.charges);
        let without = ee_pa_with(&frags, &z.charges, false);
        let d_ab = (with.e_dimer[0].2 - without.e_dimer[0].2).abs();
        assert!(d_ab >= 1e-6, "plant (i) carrier at {r_ang} Å: |E_AB[q_C] − E_AB| = {d_ab:.3e}");
        // (ii): the two models' dipoles on the embedded monomers differ by ≥ 0.05 a.u.
        let m = embed_many(&frags, ChargeModel::Mulliken, Start::Zero);
        let mu_primary = z.monomers[1].dipole[2];
        let mu_from_mulliken = m.charges[1][1] * R_HF; // the control's charges' own dipole
        assert!((mu_primary - mu_from_mulliken).abs() >= 0.05, "plant (ii) carrier at {r_ang} Å: dipoles {mu_primary} vs {mu_from_mulliken}");
        eprintln!("carriers at {r_ang} Å: (i) {d_ab:.3e} Ha, (ii) {:.3} a.u.", (mu_primary - mu_from_mulliken).abs());
    }
}
