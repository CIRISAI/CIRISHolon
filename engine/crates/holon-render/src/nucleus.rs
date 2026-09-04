//! The bottom of the ladder: the ATOM band and the NUCLEUS band of the workbench (FSD-W3
//! §11.2, WB-10.1 / WB-10.2), as ABI doors.
//!
//! Every number served here is one of two kinds, and the door says which:
//!
//! * COMPUTED by the engine — the picked atom's (or its molecule's) STO-3G FCI on the lane
//!   engine, the same arithmetic as the native referee under the reduction law, so the page's
//!   value can be pinned EQUAL TO THE BIT against native (`holon_law_probe`, `tests/wasm_law.rs`,
//!   `docs/workbench/law_probe.json`); and the nucleus's thermal de Broglie wavelength at the
//!   scene's measured temperature, a closed form on `Sim::temperature`.
//! * DECLARED, MEASURED INPUTS — nuclear spin, charge radius, atomic mass — served from
//!   `holon_chem::elements::NUCLEI` / `Species`, which name their sources; the page tags them
//!   DECLARED (WB-1.7). A door that cannot serve a declared value returns the sentinel the page
//!   fences on (`u32::MAX`, `0.0`), never a plausible number.
//!
//! "In a molecule or not" is the census's word (`holon_atom_in_molecule`): membership of a live
//! `HolonRow`, never a distance heuristic.

use std::sync::Mutex;

use holon_chem::dual::D2;
use holon_chem::elements::{by_z, nuclear, OXYGEN};
use holon_chem::fci::SolveExit;
use holon_chem::pair::{atom_energy, solve_geometry};

use crate::sim::K_B;

// ------------------------------------------------------------------ the nucleus, declared

#[no_mangle]
pub extern "C" fn holon_nucleus_spin2(z: u32) -> u32 {
    nuclear(z).map_or(u32::MAX, |n| u32::from(n.spin_2))
}

#[no_mangle]
pub extern "C" fn holon_nucleus_charge_radius_fm(z: u32) -> f64 {
    nuclear(z).map_or(0.0, |n| n.charge_radius_fm)
}

#[no_mangle]
pub extern "C" fn holon_nucleus_mass_u(z: u32) -> f64 {
    by_z(z).map_or(0.0, |s| s.mass_u)
}

/// The thermal de Broglie wavelength `λ = sqrt(2π ħ² / (m k T))` in bohr, atomic units
/// (`ħ = 1`), for a nucleus of mass `mass_me` (electron masses) at `t_kelvin`. Zero when the
/// temperature is not positive: a wavelength at absolute zero is not a number this door
/// pretends to have.
pub fn thermal_wavelength_bohr(mass_me: f64, t_kelvin: f64) -> f64 {
    if !(t_kelvin > 0.0) || !(mass_me > 0.0) {
        return 0.0;
    }
    (2.0 * std::f64::consts::PI / (mass_me * K_B * t_kelvin)).sqrt()
}

/// Atom `i`'s NUCLEUS at the scene's measured temperature: the atomic mass less `Z` electron
/// masses (the electrons ride with the atom in the dynamics; the wavelength is the nucleus's
/// own). Zero for an absent atom or an undefined temperature.
#[no_mangle]
pub extern "C" fn holon_nucleus_thermal_wavelength_bohr(i: u32) -> f64 {
    let s = crate::sim();
    let i = i as usize;
    if i >= s.n {
        return 0.0;
    }
    let sp = s.atoms[i].species;
    let m_nucleus = sp.mass_me() - f64::from(sp.z);
    thermal_wavelength_bohr(m_nucleus, s.temperature())
}

// ------------------------------------------------------------------ in a molecule or not

/// `0` when atom `i` belongs to no live census molecule row; otherwise `1 + row index`.
#[no_mangle]
pub extern "C" fn holon_atom_in_molecule(i: u32) -> u32 {
    let s = crate::sim();
    if (i as usize) >= s.n {
        return 0;
    }
    for (r, row) in s.holons.rows.iter().enumerate() {
        if row.alive && row.members[..row.member_count as usize].contains(&i) {
            return r as u32 + 1;
        }
    }
    0
}

// ------------------------------------------------------------------ the atom band

/// The last atom-band solve: the picked atom alone, or the census molecule it belongs to,
/// solved as a geometry of the engine's own species at the atoms' current positions.
struct AtomBand {
    atom: u32,
    energy: f64,
    n_electrons: u32,
    residual: f64,
    exit: u32,
}

static BAND: Mutex<Option<AtomBand>> = Mutex::new(None);

fn exit_code(e: SolveExit) -> u32 {
    match e {
        SolveExit::Converged => 0,
        SolveExit::IterationCap => 1,
        SolveExit::Stagnated => 2,
        SolveExit::Trivial => 3,
    }
}

/// Solve the atom band for atom `i` — the free atom, or the whole molecule the census says it
/// is in — and keep the result for the getters. Returns the exit code (`4` = no such atom).
/// The page calls this on pick and at its own cadence, not per frame: a molecule's FCI is
/// milliseconds, and its value changes with every position.
#[no_mangle]
pub extern "C" fn holon_atom_band_solve(i: u32) -> u32 {
    let (species, centres) = {
        let s = crate::sim();
        if (i as usize) >= s.n {
            return 4;
        }
        let members: Vec<u32> = match s.holons.rows.iter().find(|row| row.alive && row.members[..row.member_count as usize].contains(&i)) {
            Some(row) => row.members[..row.member_count as usize].to_vec(),
            None => vec![i],
        };
        let species = members.iter().map(|&m| s.atoms[m as usize].species).collect::<Vec<_>>();
        let centres = members
            .iter()
            .map(|&m| {
                let a = &s.atoms[m as usize];
                [D2::c(a.x), D2::c(a.y), D2::c(a.z)]
            })
            .collect::<Vec<_>>();
        (species, centres)
    };
    let n_electrons = species.iter().map(|sp| sp.z).sum();
    let sol = solve_geometry(&species, centres);
    let code = exit_code(sol.exit);
    *BAND.lock().expect("atom band") = Some(AtomBand { atom: i, energy: sol.e.v, n_electrons, residual: sol.residual, exit: code });
    code
}

fn band_for(i: u32) -> Option<(f64, u32, f64, u32)> {
    BAND.lock().expect("atom band").as_ref().filter(|b| b.atom == i).map(|b| (b.energy, b.n_electrons, b.residual, b.exit))
}

#[no_mangle]
pub extern "C" fn holon_atom_band_energy(i: u32) -> f64 {
    band_for(i).map_or(0.0, |b| b.0)
}

#[no_mangle]
pub extern "C" fn holon_atom_band_n_electrons(i: u32) -> u32 {
    band_for(i).map_or(0, |b| b.1)
}

#[no_mangle]
pub extern "C" fn holon_atom_band_residual(i: u32) -> f64 {
    band_for(i).map_or(0.0, |b| b.2)
}

/// `0` converged, `1` iteration cap, `2` stagnated, `3` trivial, `4` not computed for this atom.
#[no_mangle]
pub extern "C" fn holon_atom_band_exit(i: u32) -> u32 {
    band_for(i).map_or(4, |b| b.3)
}

/// The picked atom's electronic SIZE — the root-mean-square distance of an electron from
/// its nucleus in the free atom, in bohr, from the same STO-3G solve the band runs
/// (`holon_chem::pair::atomic_rms_radius`). What the atom band DRAWS: a sphere of this
/// radius at the band's own scale, so the picture is a measured size and not an icon.
/// `0.0` for no such atom.
#[no_mangle]
pub extern "C" fn holon_atom_band_rms_radius_bohr(i: u32) -> f64 {
    let sp = {
        let s = crate::sim();
        if (i as usize) >= s.n {
            return 0.0;
        }
        s.atoms[i as usize].species
    };
    holon_chem::pair::atomic_rms_radius(sp)
}

// ------------------------------------------------------------------ the law probe

/// A fixed reference solve — the oxygen atom's STO-3G FCI energy on the lane engine — whose
/// bits are pinned natively in `docs/workbench/law_probe.json` (`tests/wasm_law.rs`). The page
/// shows this value beside the pin: EQUAL TO THE BIT is the reduction law holding across the
/// wasm build and the native referee, and anything else is a defect on screen.
#[no_mangle]
pub extern "C" fn holon_law_probe() -> f64 {
    atom_energy(OXYGEN)
}
