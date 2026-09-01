//! Write a SYNTHETIC `DE4TABLE/quaternary-table/v1` artifact, in exactly the production
//! format, on the real 13/11 grid.
//!
//! # Why this exists
//!
//! `examples/de4_certify.rs` runs `DE4_TABLE_PREREG.md`'s acceptance gates. The production
//! table takes hours to generate, and a gate that has never run is not a gate — it is a
//! block of code with an unmeasured claim attached. This writes a table the certifier can
//! actually be run against TODAY, so the four gates that are properties of the HARNESS and
//! the INTERPOLANT rather than of the physics (C1, S3, B1, and the loader's grid-line
//! refusal) are demonstrated firing-or-passing before the real artifact exists.
//!
//! # What the surface is, stated so it cannot be mistaken for physics
//!
//! ```text
//! dE4_synth(R, u) = A * s(R1) s(R2) s(R3) * (1/6) * SUM_ab u_ab * (1 + m_ab)
//!     s(R)  = exp(-DECAY * (R - R_LO))          the far-field decay B1 measures
//!     m_ab  = ( tau(R_a) + tau(R_b) ) / 2       tau = (R - R_LO)/(R_HI - R_LO)
//!     ab    ranges over the three pairs (1,2), (2,3), (3,1)
//! ```
//!
//! Two properties are deliberate and both are load-bearing:
//!
//! * It is invariant under the DIAGONAL `S3` — the simultaneous permutation of the three
//!   `R` axes and the three `u` axes — and NOT under `S3 x S3`. That matters: a surface
//!   symmetric under independent permutation of the cosines would pass gate S3 even if the
//!   orbit map coupled `R` to the wrong pair, which is exactly the `sort_ohhh_internals`
//!   defect (M2) this table exists to repair. `m_ab` is what couples them.
//! * It decays, so gate B1 measures something. At `R = R_HI` the factor is
//!   `exp(-1.7 * 5.1) = 1.72e-4`, so the boundary shell carries a small NONZERO value
//!   rather than an identically-zero one a gate cannot see.
//!
//! It is NOT a four-body term, it is not FCI, and its `provenance` field says so in words
//! `de4_certify` checks: the certifier REFUSES to run the physics gates against it, and
//! refuses to skip them for a production artifact. The flag is cross-checked, not trusted.
//!
//! Run:
//! ```text
//! cargo run --release -p holon-chem --example de4_synth_artifact -- <out.synth.json>
//! ```

use holon_chem::quaternary_table as qt;
use std::io::{BufWriter, Write};

/// The label a synthetic artifact carries INSTEAD of `qt::QUATERNARY_PROVENANCE`. The
/// certifier keys its refusals on this being different, so the two must never converge.
const SYNTH_PROVENANCE: &str =
    "SYNTHETIC closed-form S3-symmetric surface for harness certification -- NOT physics, \
     NOT FCI, NOT a four-body term";

const AMPL: f64 = 0.20;
const DECAY: f64 = 1.7;

/// The closed form. Diagonal-`S3` symmetric by construction: every term pairs `u_ab` with
/// the two radii of the SAME pair, so a relabelling moves both together.
fn synth(r: [f64; 3], u: [f64; 3]) -> f64 {
    let s = |x: f64| (-DECAY * (x - qt::R_LO)).exp();
    let tau = |x: f64| (x - qt::R_LO) / (qt::R_HI - qt::R_LO);
    let m = |a: usize, b: usize| 0.5 * (tau(r[a]) + tau(r[b]));
    let bracket =
        (u[0] * (1.0 + m(0, 1)) + u[1] * (1.0 + m(1, 2)) + u[2] * (1.0 + m(2, 0))) / 6.0;
    AMPL * s(r[0]) * s(r[1]) * s(r[2]) * bracket
}

/// FNV-1a over the value bits, little-endian. This is THIS FILE's digest, not
/// `holon-tables`' `Digest` — a synthetic artifact must not be able to present a digest
/// that looks like a certified one.
fn fnv1a(vals: &[f64]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in vals {
        for b in v.to_bits().to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x1000_0000_01b3);
        }
    }
    h
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = match args.get(1) {
        Some(p) if p.ends_with(".synth.json") => p.clone(),
        Some(p) => {
            eprintln!(
                "de4_synth_artifact: refusing to write {p}: the path must end .synth.json so a \
                 synthetic surface can never be mistaken for the campaign's table"
            );
            std::process::exit(2);
        }
        None => {
            eprintln!("usage: de4_synth_artifact <out.synth.json>");
            std::process::exit(2);
        }
    };

    println!("=== de4_synth_artifact: a SYNTHETIC table in the production format ===");
    println!("  grid_line       {}", qt::grid_line());
    println!("  box nodes       {}", qt::N_NODES);
    println!("  surface         A={AMPL} exp(-{DECAY}(R-R_LO))^3 * mean_ab u_ab (1 + m_ab)");
    println!("  provenance      {SYNTH_PROVENANCE}");

    // Fill exactly the way the production generator does: solve the canonical
    // representative, then write its whole relabelling orbit through `set_orbit`. Filling
    // every box node directly would be simpler AND would stop exercising the orbit map,
    // which is the addressing gate S3 exists to convict.
    let mut t = qt::QuaternaryTable::begin();
    let mut reps = 0usize;
    let mut solved = 0usize;
    let mut continued_reps = 0usize;
    let mut peak = 0.0f64;
    for i0 in 0..qt::NR {
        for i1 in 0..qt::NR {
            for i2 in 0..qt::NR {
                for k0 in 0..qt::NU {
                    for k1 in 0..qt::NU {
                        for k2 in 0..qt::NU {
                            let i = [i0, i1, i2];
                            let k = [k0, k1, k2];
                            if qt::canonical_index(i, k) != (i, k) {
                                continue;
                            }
                            reps += 1;
                            let r = [qt::node_r(i0), qt::node_r(i1), qt::node_r(i2)];
                            let u0 = [qt::node_u(k0), qt::node_u(k1), qt::node_u(k2)];
                            // The frozen continuation rule: a node outside the elliptope is
                            // not a geometry, and is filled by the S3-equivariant radial
                            // scaling and MARKED. Applied here so the synthetic artifact has
                            // the same structure the real one will have.
                            let real = qt::gram_det(u0) >= 0.0;
                            let (u, _s) = qt::elliptope_scale(u0);
                            if real {
                                solved += 1;
                            } else {
                                continued_reps += 1;
                            }
                            let v = synth(r, u);
                            if v.abs() > peak {
                                peak = v.abs();
                            }
                            t.set_orbit(i, k, v, real);
                        }
                    }
                }
            }
        }
    }
    let mut meta = qt::QuaternaryMeta::empty();
    meta.solves = solved;
    meta.continued = continued_reps;
    assert!(t.finish(meta), "the synthetic table failed its own finish()");
    assert_eq!(
        t.filled(),
        qt::N_NODES,
        "the orbit fill left {} box nodes unwritten",
        qt::N_NODES - t.filled()
    );

    // The per-node counts the artifact reports. `voided` in the production writer is the
    // count of records that are not OK, which for this grid is the not-a-geometry set, so
    // it is counted over the BOX rather than over the representatives.
    let mut box_continued = 0usize;
    for k0 in 0..qt::NU {
        for k1 in 0..qt::NU {
            for k2 in 0..qt::NU {
                if qt::gram_det([qt::node_u(k0), qt::node_u(k1), qt::node_u(k2)]) < 0.0 {
                    box_continued += qt::NR * qt::NR * qt::NR;
                }
            }
        }
    }
    let box_solved = qt::N_NODES - box_continued;

    println!("\n--- filled ---");
    println!("  canonical reps  {reps}");
    println!("  of which real   {solved}   continued {continued_reps}");
    println!("  box real        {box_solved}   box continued {box_continued}");
    println!("  peak |dE4|      {peak:.6e} Ha   (the real table's banked max is 2.284e-1)");

    // Read the box back out in canonical `node_index` order, which is the order
    // `de4_table` writes `values_hex` in (its records are indexed by the NdGrid's linear
    // id, axis 0 slowest, which IS `node_index` for these six axes).
    let mut vals: Vec<f64> = Vec::with_capacity(qt::N_NODES);
    for i0 in 0..qt::NR {
        for i1 in 0..qt::NR {
            for i2 in 0..qt::NR {
                for k0 in 0..qt::NU {
                    for k1 in 0..qt::NU {
                        for k2 in 0..qt::NU {
                            vals.push(t.node([i0, i1, i2], [k0, k1, k2]));
                        }
                    }
                }
            }
        }
    }
    assert_eq!(vals.len(), qt::N_NODES);
    let digest = fnv1a(&vals);

    // Boundary reading, printed here so the artifact's own writer states what B1 will find
    // and a disagreement between the two is visible rather than silent.
    let mut shell = 0.0f64;
    for i1 in 0..qt::NR {
        for i2 in 0..qt::NR {
            for k0 in 0..qt::NU {
                for k1 in 0..qt::NU {
                    for k2 in 0..qt::NU {
                        let v = t.node([qt::NR - 1, i1, i2], [k0, k1, k2]).abs();
                        if v > shell {
                            shell = v;
                        }
                    }
                }
            }
        }
    }
    println!("  R1 = R_HI shell max |dE4|  {shell:.4e} Ha   (B1's band is 1.0e-4)");

    let f = std::fs::File::create(&out).unwrap_or_else(|e| {
        eprintln!("de4_synth_artifact: cannot write {out}: {e}");
        std::process::exit(2);
    });
    let mut f = BufWriter::with_capacity(1 << 22, f);
    let _ = writeln!(f, "{{");
    let _ = writeln!(f, "  \"schema\": \"DE4TABLE/quaternary-table/v1\",");
    let _ = writeln!(f, "  \"provenance\": \"{SYNTH_PROVENANCE}\",");
    let _ = writeln!(f, "  \"synthetic\": true,");
    let _ = writeln!(f, "  \"solver_route\": \"closed-form\",");
    let _ = writeln!(f, "  \"exact_in_model\": false,");
    let _ = writeln!(f, "  \"model\": \"NONE -- a closed-form stand-in for harness certification\",");
    let _ = writeln!(f, "  \"stored\": \"dE4_synth, hartree-shaped but not a physical quantity\",");
    let _ = writeln!(f, "  \"grid_line\": \"{}\",", qt::grid_line().replace('"', "'"));
    let _ = writeln!(
        f,
        "  \"grid\": {{\"nr\": {}, \"nu\": {}, \"region\": [0,0,0,0,0,0], \"n_nodes\": {}, \"orbits\": {}}},",
        qt::NR, qt::NU, qt::N_NODES, reps
    );
    let _ = writeln!(f, "  \"warm_policy\": \"NotApplicable\",");
    let _ = writeln!(f, "  \"serpentine\": \"NotApplicable\",");
    let _ = writeln!(f, "  \"digest\": \"{digest:016x}\",");
    let _ = writeln!(f, "  \"digest_covers\": [\"energy\"],");
    let _ = writeln!(
        f,
        "  \"solved\": {}, \"mirrored\": {}, \"voided\": {},",
        box_solved,
        qt::N_NODES - reps,
        box_continued
    );
    let _ = writeln!(
        f,
        "  \"generation\": {{\"workers\": 1, \"wall_s\": 0.0, \"cold_solves\": 0, \"warm_solves\": 0, \"davidson_iters\": 0, \"core_s_per_solved_node\": 0.0}},"
    );
    let _ = writeln!(
        f,
        "  \"seams\": {{\"scanned\": false, \"instrument\": \"none\", \"loci\": [], \"accepted_floor\": {{\"hartree\": 0.0, \"why\": \"synthetic surface: no seam scan was run and none is claimed\", \"located_by\": \"none\"}}}},"
    );
    let _ = writeln!(f, "  \"units\": \"Hartree atomic units: R in bohr, u dimensionless, dE4 in hartree\",");
    let _ = writeln!(f, "  \"values_hex\": [");
    for (n, v) in vals.iter().enumerate() {
        let comma = if n + 1 == vals.len() { "" } else { "," };
        let _ = writeln!(f, "    \"{:016x}\"{}", v.to_bits(), comma);
    }
    let _ = writeln!(f, "  ]");
    let _ = writeln!(f, "}}");
    let _ = f.flush();
    drop(f);

    let sz = std::fs::metadata(&out).map(|m| m.len()).unwrap_or(0);
    println!("\nwrote {out}  ({:.1} MB)", sz as f64 / 1.048576e6);
    println!("digest (FNV-1a over the value bits, NOT holon-tables' Digest): {digest:016x}");
}
