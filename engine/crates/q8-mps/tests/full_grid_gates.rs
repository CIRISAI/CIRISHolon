//! G2, G3-secondary, G6, G0-2, G7 — `Q8_MPS_PREREG.md` §4. Grid `N ∈ {8,10}` per Amendment 2
//! (2026-08-24): `N=12` was demoted mid-run on a resource decision, not a scientific one — see
//! the amendment for the probe numbers and the reasoning. `#[ignore]`d: this is a genuinely
//! long-running validation (minutes per configuration), run explicitly with
//! `cargo test --release -- --ignored --nocapture`, not on every default `cargo test`.
//!
//! One DMRG run per configuration backs every gate below — they are reported and asserted
//! together per configuration (one gate per invariant, per house rule, but sharing the run that
//! produces them all is not the same as conflating what they test).

mod common;

use common::{CachedExact, U_GRID};
use q8_mps::dmrg::{self, Params};
use q8_mps::observables;

const CHI_MAX: usize = 256;
const MAX_SWEEPS: usize = 20;
const SWEEP_TOL: f64 = 1e-10;

const ENERGY_TOL_GAPPED: f64 = 1e-8; // relative, U/t in {1,4,16}
const ENERGY_TOL_GAPLESS: f64 = 1e-6; // relative, U=0
const DENSITY_TOL: f64 = 1e-6;
const DOCC_TOL: f64 = 1e-6;
const MI_TOL: f64 = 1e-6; // G6
const SECTOR_TOL: f64 = 1e-6; // G0-2: |<N_tot>-N|, |<Sz>|, <Sz^2>
const FLOOR_SLACK: f64 = 1e-9; // G3-secondary

#[test]
#[ignore]
fn full_grid_g2_g3_g6_g0_2_g7() {
    let mut failures: Vec<String> = Vec::new();
    let mut void_count = 0usize;
    let grid_size = [8usize, 10].len() * U_GRID.len();
    let cache = common::load_exact_cache();
    // Rotates which single config is forced live each run (A3.2 fix — a fixed "always the
    // first" pin would spot-check only the cheapest entry forever, N=10/U=16 never re-validated
    // once cached).
    let forced_index = common::next_rotation_index(grid_size);
    let mut grid_idx = 0usize;

    for &sites in &[8usize, 10] {
        for &u in &U_GRID {
            eprintln!("=== N={sites} U={u} — running DMRG (chi={CHI_MAX}) ===");
            let p =
                Params { sites, t: 1.0, u, chi_max: CHI_MAX, max_sweeps: MAX_SWEEPS, sweep_tol: SWEEP_TOL };
            let r = match dmrg::run(&p, dmrg::RefusalPolicy::Typed) {
                Ok(r) => r,
                Err(refusal) => {
                    failures.push(format!(
                        "N={sites} U={u}: engine REFUSED at chi={CHI_MAX} — bond={} weight={:e} \
                         (unexpected: chi=256 is meant to be generous, not forcing)",
                        refusal.bond, refusal.weight
                    ));
                    continue;
                }
            };

            let n_target = sites as f64;
            let mu = u / 2.0;
            let unshifted = r.energy_shifted + mu * n_target;

            eprintln!(
                "N={sites} U={u}: E={unshifted:.10} sweeps={} converged={} discarded_max={:e}",
                r.sweeps_used,
                r.converged,
                r.discarded_weight.iter().cloned().fold(0.0, f64::max)
            );

            // G7 — fixed schedule, no post-hoc extension. A non-convergence is recorded, not
            // silently retried with a bigger cap.
            eprintln!(
                "N={sites} U={u}: G7 sweeps_used={}/{MAX_SWEEPS} converged={} (band: converge by sweep {MAX_SWEEPS})",
                r.sweeps_used, r.converged
            );
            if !r.converged {
                void_count += 1;
                failures.push(format!("N={sites} U={u}: G7 VOID — did not converge in {MAX_SWEEPS} sweeps"));
            }

            // Cached q-seam exact reference (declared deviation from §9's "called live for
            // every exact comparison" — cache.rs's doc comment, research-manager verification):
            // deterministic under q-seam's pinned Lanczos policy, so re-deriving it on every
            // re-run during iterative fixing is pure waste. Exactly ONE config per run is
            // forced live regardless of cache state, ROTATING across runs (A3.2 fix — the naive
            // always-first pin spot-checks only the cheapest entry forever), and checked against
            // the cache entry if one exists — a stale cache is a finding, not silently trusted.
            let force_live = grid_idx == forced_index;
            grid_idx += 1;
            let cached = cache.get(&(sites, u.to_bits()));
            let need_live = force_live || cached.is_none();
            let exact_live = if need_live {
                eprintln!(
                    "N={sites} U={u}: calling q-seam exact reference (dev-dependency, live{})...",
                    if force_live { ", forced (per-run spot-check)" } else { "" }
                );
                let h = q_seam::hubbard::Hubbard::new(sites, 1.0, u);
                let exact = match q_seam::lanczos::ground_state(&h) {
                    Some(g) => g,
                    None => {
                        failures.push(format!("N={sites} U={u}: q-seam Lanczos failed to converge — VOID"));
                        continue;
                    }
                };
                let obs = q_seam::observables::ExactObservables::measure(&h, &exact.vector);
                let e = CachedExact {
                    energy: exact.energy,
                    s_squared: obs.s_squared,
                    density: obs.density.clone(),
                    magnetization: obs.magnetization.clone(),
                    double_occ: obs.double_occ.clone(),
                };
                if cached.is_none() {
                    common::append_exact_cache(sites, u, &e);
                }
                Some(e)
            } else {
                eprintln!("N={sites} U={u}: using cached q-seam exact reference");
                None
            };

            if let (Some(live), Some(c)) = (&exact_live, cached) {
                let de = (live.energy - c.energy).abs();
                if de > 1e-12 {
                    panic!(
                        "N={sites} U={u}: CACHE STALE — live energy {} vs cached {} (Δ={de:e}); \
                         determinism is broken somewhere, this outranks every other finding",
                        live.energy, c.energy
                    );
                }
            }
            let exact_cached = exact_live.as_ref().unwrap_or_else(|| cached.unwrap());
            let exact_energy = exact_cached.energy;
            let exact_density = &exact_cached.density;
            let exact_double_occ = &exact_cached.double_occ;

            // G3-secondary: the variational floor, unshifted (valid once G0-2 below confirms
            // sector lock — reported together, not conflated).
            let g3_secondary_margin = unshifted - exact_energy;
            eprintln!(
                "N={sites} U={u}: G3-secondary margin={g3_secondary_margin:e} (band: >= -{FLOOR_SLACK:e})"
            );
            if unshifted < exact_energy - FLOOR_SLACK {
                failures.push(format!(
                    "N={sites} U={u}: G3-secondary FAILED — DMRG {unshifted} BELOW exact {} by {:e}",
                    exact_energy,
                    exact_energy - unshifted
                ));
            }

            // G3-primary (research-manager's Defect 2: previously UNCHECKED — no per-sweep
            // history existed to check it against). Two clauses on the SHIFTED trajectory:
            // floor at every sweep, and monotone non-increasing across sweeps.
            let exact_shifted = exact_energy - mu * n_target;
            let hist = &r.energy_history;
            let worst_floor_margin = hist.iter().map(|&e| e - exact_shifted).fold(f64::INFINITY, f64::min);
            eprintln!(
                "N={sites} U={u}: G3-primary floor worst_margin={worst_floor_margin:e} over {} sweeps (band: >= -{FLOOR_SLACK:e})",
                hist.len()
            );
            let floor_violation = hist
                .iter()
                .enumerate()
                .find(|(_, &e)| e < exact_shifted - FLOOR_SLACK);
            if let Some((k, &e)) = floor_violation {
                failures.push(format!(
                    "N={sites} U={u}: G3-primary FLOOR FAILED — sweep {k} energy {e} BELOW shifted exact {exact_shifted} by {:e}",
                    exact_shifted - e
                ));
            }
            let worst_rise = hist.windows(2).map(|w| w[1] - w[0]).fold(f64::NEG_INFINITY, f64::max);
            eprintln!(
                "N={sites} U={u}: G3-primary monotonicity worst_rise={worst_rise:e} (band: <= {FLOOR_SLACK:e})"
            );
            let mut non_monotone = None;
            for w in hist.windows(2) {
                if w[1] > w[0] + FLOOR_SLACK {
                    non_monotone = Some((w[0], w[1]));
                    break;
                }
            }
            if let Some((prev, next)) = non_monotone {
                failures.push(format!(
                    "N={sites} U={u}: G3-primary MONOTONICITY FAILED — energy rose {prev} -> {next}"
                ));
            }

            // G2 — energy.
            let etol = if u == 0.0 { ENERGY_TOL_GAPLESS } else { ENERGY_TOL_GAPPED };
            let e_rel = (unshifted - exact_energy).abs() / exact_energy.abs().max(1.0);
            eprintln!("N={sites} U={u}: G2 energy rel_err={e_rel:e} (band: <= {etol:e})");
            if e_rel > etol {
                failures.push(format!(
                    "N={sites} U={u}: G2 energy FAILED — rel err {e_rel:e} > {etol:e} (mps={unshifted}, exact={})",
                    exact_energy
                ));
            }

            // G2 — density profile (n_i = up+down per chain site).
            let mps_occ = observables::occupation_profile(&r.tensors, sites);
            let exact_occ = &exact_density;
            let occ_err = mps_occ
                .iter()
                .zip(exact_occ.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f64::max);
            eprintln!("N={sites} U={u}: G2 density max|Δn_i|={occ_err:e} (band: <= {DENSITY_TOL:e})");
            if occ_err > DENSITY_TOL {
                failures.push(format!("N={sites} U={u}: G2 density FAILED — max|Δn_i|={occ_err:e} > {DENSITY_TOL:e}"));
            }

            // G2 — double occupancy.
            let mps_docc = observables::double_occupancy_profile(&r.tensors, sites);
            let exact_docc = &exact_double_occ;
            let docc_err = mps_docc
                .iter()
                .zip(exact_docc.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f64::max);
            eprintln!("N={sites} U={u}: G2 double-occ max|Δd_i|={docc_err:e} (band: <= {DOCC_TOL:e})");
            if docc_err > DOCC_TOL {
                failures.push(format!("N={sites} U={u}: G2 double-occ FAILED — max|Δd_i|={docc_err:e} > {DOCC_TOL:e}"));
            }

            // G6 — m_i theorem anchor (=0 exactly; Q7 §2.2 spin-independence + Sz-sector
            // uniqueness, not Lieb). Needs no exact reference.
            let mps_mag = observables::magnetization_profile(&r.tensors, sites);
            let mag_err = mps_mag.iter().cloned().map(f64::abs).fold(0.0, f64::max);
            eprintln!("N={sites} U={u}: G6 max|m_i|={mag_err:e} (band: <= {MI_TOL:e})");
            if mag_err > MI_TOL {
                failures.push(format!("N={sites} U={u}: G6 m_i FAILED — max|m_i|={mag_err:e} > {MI_TOL:e}"));
            }

            // G0-2 — standing sector-lock anchor. Needs no exact reference either.
            let n_tot = observables::total_number(&r.tensors);
            let sz = observables::total_sz(&r.tensors);
            let sz_sq = observables::total_sz_squared(&r.tensors);
            eprintln!(
                "N={sites} U={u}: G0-2 |N_tot-{n_target}|={:e} |Sz|={:e} Sz^2={:e} (band: <= {SECTOR_TOL:e} each)",
                (n_tot - n_target).abs(), sz.abs(), sz_sq.abs()
            );
            if (n_tot - n_target).abs() > SECTOR_TOL {
                failures.push(format!("N={sites} U={u}: G0-2 N_tot FAILED — |{n_tot}-{n_target}| > {SECTOR_TOL:e}"));
            }
            if sz.abs() > SECTOR_TOL {
                failures.push(format!("N={sites} U={u}: G0-2 Sz FAILED — |Sz|={} > {SECTOR_TOL:e}", sz.abs()));
            }
            if sz_sq.abs() > SECTOR_TOL {
                failures.push(format!("N={sites} U={u}: G0-2 Sz^2 FAILED — Sz^2={sz_sq} > {SECTOR_TOL:e}"));
            }

            // Particle-hole anchor (theorem-pinned <n_j_sigma>=1/2 PER SPIN-ORBITAL,
            // Q8_MPS_PREREG.md §1.1(iii) — the governing text; §3's shorthand dropped the sigma
            // and an earlier version of this check applied it to the per-site TOTAL, which
            // particle-hole pins to 1, not 1/2 (occupation_profile sums both spins per chain
            // site, matching q-seam's own `density` convention). Fixed per research-manager's
            // Defect 1: checked per spin-orbital directly, not via the per-site total.
            let norm = observables::norm_squared(&r.tensors);
            let ph_err = (0..2 * sites)
                .map(|j| (observables::expectation(&r.tensors, &[(j, q8_mps::ops::N2)]) / norm - 0.5).abs())
                .fold(0.0, f64::max);
            eprintln!("N={sites} U={u}: particle-hole max|<n_j_sigma>-0.5|={ph_err:e} (band: <= {DENSITY_TOL:e})");
            if ph_err > DENSITY_TOL {
                failures.push(format!(
                    "N={sites} U={u}: particle-hole anchor FAILED — max|<n_j_sigma>-0.5|={ph_err:e}"
                ));
            }
        }
    }

    // SWEEP KILL (§7): "fires iff more than 2 of the grid's configurations VOID under G7."
    // Amendment 2 shrank the grid from 12 to 8 without touching that absolute count — an open
    // researcher degree of freedom in front of live data, ruled here under BOTH readings per
    // A3/R2 discipline (research-manager verification): absolute (>2 of the grid, whatever its
    // size) and proportional (>2/12 of the ORIGINAL grid, i.e. >1.33 of 8, so >=2). Both fire on
    // this data, so nothing is left unadjudicated and the amendment decided neither.
    let sweep_kill_absolute = void_count > 2;
    let sweep_kill_proportional = (void_count as f64) > (2.0 / 12.0) * (grid_size as f64);
    let sweep_kill_fires = sweep_kill_absolute && sweep_kill_proportional;
    println!(
        "\n=== SWEEP KILL: {} of {grid_size} configurations VOID under G7 — absolute reading (>2) {}, \
         proportional reading (>2/12 of grid) {} — {} ===",
        void_count,
        if sweep_kill_absolute { "FIRES" } else { "holds" },
        if sweep_kill_proportional { "FIRES" } else { "holds" },
        if sweep_kill_fires { "SWEEP KILL FIRES" } else { "SWEEP KILL DOES NOT FIRE" }
    );

    if !failures.is_empty() {
        panic!(
            "{} gate failure(s) (includes {void_count} G7 VOID(s), SWEEP KILL {}):\n{}",
            failures.len(),
            if sweep_kill_fires { "FIRES" } else { "does not fire" },
            failures.join("\n")
        );
    }
    assert!(!sweep_kill_fires, "SWEEP KILL fires ({void_count}/{grid_size} VOID) even though no other gate failed");
}
