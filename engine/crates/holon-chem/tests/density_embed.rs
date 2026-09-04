//! EMBED-2's gates that need no trimer (`EMBED2_PREREG.md` §3, §5): G1 (Hellmann–Feynman on
//! the density field), G2 (one fixed point of the densities), G3 (the reduction is exact),
//! and both plants' carriers, on the HF chain at 5.0 and 3.0 Å.

use holon_chem::density_embed::*;
use holon_chem::elements::{by_symbol, Species};
use holon_chem::embed::*;
use holon_chem::seam::{bare_pa, hf_chain};

fn sp(s: &str) -> Species {
    by_symbol(s).expect("species")
}
const R_HF: f64 = 1.879437929774;

#[test]
fn g1_hellmann_feynman_on_the_density_field_and_plant_i() {
    for r_ang in [5.0, 3.0] {
        let frags = hf_chain(sp("F"), sp("H"), R_HF, r_ang * ANGSTROM_TO_BOHR, 2);
        let pb = solve_in_densities(&frags[1], &[]).p;
        let at = |lam: f64, plant: bool| {
            let scaled: Vec<f64> = pb.iter().map(|x| x * lam).collect();
            solve_in_densities_with(&frags[0], &[Partner { frag: &frags[1], p: &scaled }], plant, false)
        };
        let h = 1e-4;
        let base = at(1.0, false);
        let fd = (at(1.0 + h, false).e_total - at(1.0 - h, false).e_total) / (2.0 * h);
        let hf = base.e_ee - base.e_ne;
        assert!((fd - hf).abs() <= 1e-7, "G1 at {r_ang} Å: dE/dλ {fd} vs ⟨ρ_A|J⟩ − Σ Z V {hf}");
        // plant (i): J attractive — the derivative disagrees by 2⟨ρ_A|J⟩
        let fd_p = (at(1.0 + h, true).e_total - at(1.0 - h, true).e_total) / (2.0 * h);
        assert!(base.e_ee >= 1e-3, "plant (i) carrier ⟨ρ_A|J⟩ = {} not nonzero in the sector", base.e_ee);
        assert!((fd_p - hf).abs() >= 2.0 * base.e_ee - 1e-3, "plant (i) did not fire: {fd_p} vs {hf}");
        eprintln!("G1 {r_ang} Å: |dE/dλ − HF| = {:.2e}; e_ee = {:.4}, e_ne = {:.4}; plant (i) mismatch {:.4}", (fd - hf).abs(), base.e_ee, base.e_ne, (fd_p - hf).abs());
    }
}

#[test]
fn g2_one_fixed_point_and_g3_exact_reduction() {
    for r_ang in [5.0, 3.0] {
        let frags = hf_chain(sp("F"), sp("H"), R_HF, r_ang * ANGSTROM_TO_BOHR, 3);
        let z = embed_densities(&frags, DensityStart::Zero);
        let i = embed_densities(&frags, DensityStart::Isolated);
        assert!(z.converged && i.converged, "G2 at {r_ang} Å: {} / {} sweeps", z.sweeps, i.sweeps);
        let dp = z.densities.iter().flatten().zip(i.densities.iter().flatten()).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
        assert!(dp <= 1e-8, "G2 at {r_ang} Å: densities differ by {dp}");
        let ez = rho_pa(&frags, &z.densities);
        let ei = rho_pa(&frags, &i.densities);
        assert!((ez.total - ei.total).abs() <= 1e-10, "G2 at {r_ang} Å: E_ρ-PA differs by {}", (ez.total - ei.total).abs());
        // G3: no partners anywhere — every solve bare — equals the bare pairwise sum
        let bare = bare_pa(&frags);
        let mut mono = 0.0;
        for f in &frags {
            mono += solve_in_densities(f, &[]).e_total;
        }
        let mut dim = 0.0;
        for a in 0..3 {
            for b in (a + 1)..3 {
                let (s, c) = holon_chem::seam::joined(&frags[a], &frags[b]);
                let d = Fragment::new(s, c, vec![-1.0, 1.0, -1.0, 1.0]);
                dim += solve_in_densities(&d, &[]).e_total;
            }
        }
        assert!(((dim - mono) - bare.total).abs() <= 1e-12, "G3 at {r_ang} Å: {} vs {}", dim - mono, bare.total);
        eprintln!("G2 {r_ang} Å: {} / {} sweeps, Δρ {dp:.1e}, ΔE {:.1e}; G3 |Δ| = {:.1e}", z.sweeps, i.sweeps, (ez.total - ei.total).abs(), ((dim - mono) - bare.total).abs());
    }
}

#[test]
fn plant_ii_carrier_is_nonzero_in_the_sector() {
    let frags = hf_chain(sp("F"), sp("H"), R_HF, 5.0 * ANGSTROM_TO_BOHR, 3);
    let z = embed_densities(&frags, DensityStart::Zero);
    let honest = rho_pa(&frags, &z.densities);
    let dropped = rho_pa_with(&frags, &z.densities, false, true);
    let nn = nn_between(&frags[0], &frags[1]);
    assert!(nn >= 1.0, "plant (ii) carrier E_nn(A, Z_b) = {nn} not nonzero in the sector");
    // the dropped nuclear terms cancel PAIRWISE between the dimer and monomer sums (each
    // fragment's nuclei-in-partner-field enters once in each), so the SUM moves only by the
    // polarisation the unbalanced field induces — 2.4e-2 Ha here — which is still four
    // orders above the far sector's three-body term (4.5e-6 at 5 Å): the freeze's κ > 1
    assert!((dropped.total - honest.total).abs() >= 1e-4, "plant (ii) moved the sum by only {}", (dropped.total - honest.total).abs());
    eprintln!("plant (ii): E_nn(A,B) = {nn:.4} Ha; the sum moved by {:.4} Ha", (dropped.total - honest.total).abs());
}
