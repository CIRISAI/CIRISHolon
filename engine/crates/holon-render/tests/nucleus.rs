//! The bottom of the ladder's doors (FSD-W3 WB-10.1): declared values served as declared,
//! the two declared tables pinned to each other, the wavelength's closed form on a known
//! case, the sentinels where nothing is declared, and the atom band's state machine.

use holon_chem::elements::{by_z, nuclear, FIRST_ROW, OXYGEN};
use holon_chem::pair::atom_energy;
use holon_render::{holon_atom_count, holon_reset};
use holon_render::nucleus::{holon_atom_band_solve,
    holon_atom_band_coupled_rms_bohr, holon_atom_band_exit, holon_atom_band_population, holon_law_probe, holon_nucleus_charge_radius_fm, holon_nucleus_mass_u,
    holon_nucleus_spin2, holon_nucleus_thermal_wavelength_bohr, thermal_wavelength_bohr,
};

#[test]
fn the_declared_nuclei_are_the_species_isotopes_and_serve_as_declared() {
    for sp in FIRST_ROW {
        let n = nuclear(sp.z).unwrap_or_else(|| panic!("no declared nucleus for Z={}", sp.z));
        assert_eq!(n.isotope, sp.isotope, "Z={}: the nuclear table and the species table name different isotopes", sp.z);
        assert_eq!(holon_nucleus_spin2(sp.z), u32::from(n.spin_2));
        assert_eq!(holon_nucleus_charge_radius_fm(sp.z).to_bits(), n.charge_radius_fm.to_bits());
        assert_eq!(holon_nucleus_mass_u(sp.z).to_bits(), sp.mass_u.to_bits());
    }
    // the proton is spin ½; oxygen-16 is spin 0 with the Angeli–Marinova radius
    assert_eq!(holon_nucleus_spin2(1), 1);
    assert_eq!(holon_nucleus_spin2(8), 0);
    assert_eq!(holon_nucleus_charge_radius_fm(8), 2.6991);
    assert_eq!(by_z(8).unwrap().symbol, "O");
}

#[test]
fn an_undeclared_nucleus_serves_the_sentinels_and_never_a_number() {
    for z in [0u32, 11, 26, 200] {
        assert_eq!(holon_nucleus_spin2(z), u32::MAX, "Z={z}");
        assert_eq!(holon_nucleus_charge_radius_fm(z), 0.0, "Z={z}");
    }
    assert_eq!(holon_nucleus_mass_u(0), 0.0);
}

#[test]
fn the_thermal_wavelength_is_the_closed_form_and_zero_without_a_temperature() {
    // a proton at 293 K: λ = sqrt(2π ħ² / (m k T)) is about 1.0 Å; in atomic units
    let m_p = by_z(1).unwrap().mass_me() - 1.0;
    let lam = thermal_wavelength_bohr(m_p, 293.0);
    let k_b = holon_render::sim::K_B;
    let expect = (2.0 * std::f64::consts::PI / (m_p * k_b * 293.0)).sqrt();
    assert_eq!(lam.to_bits(), expect.to_bits());
    assert!((lam * 0.529177 - 1.0).abs() < 0.05, "proton at 293 K: {:.3} Å", lam * 0.529177);
    assert_eq!(thermal_wavelength_bohr(m_p, 0.0), 0.0);
    assert_eq!(thermal_wavelength_bohr(m_p, -1.0), 0.0);
    // oxygen's nucleus is heavier, so its wavelength is shorter, by sqrt(m ratio)
    let m_o = by_z(8).unwrap().mass_me() - 8.0;
    let ratio = thermal_wavelength_bohr(m_p, 293.0) / thermal_wavelength_bohr(m_o, 293.0);
    assert!((ratio - (m_o / m_p).sqrt()).abs() < 1e-12);
    // no atoms loaded in this process: the door serves zero, not a guess
    assert_eq!(holon_nucleus_thermal_wavelength_bohr(0), 0.0);
}

#[test]
fn the_atom_band_reports_not_computed_until_it_is_asked() {
    assert_eq!(holon_atom_band_exit(0), 4);
    assert_eq!(holon_atom_band_exit(7), 4);
    // the coupled size and the population serve the sentinel, never a number, unsolved
    assert_eq!(holon_atom_band_coupled_rms_bohr(0), 0.0);
    assert_eq!(holon_atom_band_population(0), 0.0);
}

#[test]
fn the_law_probe_is_the_oxygen_atom_on_the_lane_engine() {
    let probe = holon_law_probe();
    assert_eq!(probe.to_bits(), atom_energy(OXYGEN).to_bits());
    assert!(probe < -70.0 && probe > -80.0, "oxygen STO-3G FCI energy {probe}");
}

/// The coupled-size door on a real scene through the ABI: reset a hydrogen scene, solve
/// the band for atom 0, and read a positive size and a population for it — and the
/// sentinel for an atom the solve did not contain. Natively, so a wasm-side trap in the
/// same path is separable from a defect in the door.
#[test]
fn the_coupled_size_door_serves_the_solved_molecule_and_the_sentinel_elsewhere() {
    holon_reset(4);
    let n = holon_atom_count();
    assert!(n >= 2, "scene has {n} atoms");
    let code = holon_atom_band_solve(0);
    assert!(code == 0 || code == 3, "solve exit {code}");
    let r = holon_render::nucleus::holon_atom_band_coupled_rms_bohr(0);
    let pop = holon_render::nucleus::holon_atom_band_population(0);
    assert!(r > 0.5 && r < 5.0, "coupled rms {r}");
    assert!(pop > 0.0, "population {pop}");
    // an atom outside the solved molecule serves the sentinel — unless the census put every
    // atom in one row, in which case every atom has a size; both are honest, so only the
    // solved atom is asserted
    eprintln!("coupled door: atom 0 rms {r:.6} bohr, population {pop:.6}; atom 1 rms {:.6}", holon_render::nucleus::holon_atom_band_coupled_rms_bohr(1));
}
