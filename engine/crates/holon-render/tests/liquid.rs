//! LIQUID-1's gates (`conformance/water_observatory/LIQUID1_PREREG.md` §2 L0, §6): the box
//! builder, and the boundary door on the liquid cell.
//!
//! The door gate is L0's static half stated as an EXACT identity rather than an inequality:
//! the tables' reach (20 bohr) refuses a 29.6-bohr cell, the seam-aware reach admits it, and
//! `legality_radius` under the seam IS `max(INTRA_UNIT_REACH, model.reach(budget))` to the
//! bit. The per-pass half of L0 lives in the runner, where a unit that dissolves mid-arm
//! voids the arm at the frame it dissolved on.
//!
//! The seam model here is DECLARED (FIELD-9's harvested coefficients, restated in the source
//! so this gate does not depend on a record being on disk). Nothing below is a reading of the
//! law: every assertion is a property of the image rule that holds for any coefficients whose
//! reach fits the cell, and the declared numbers are simply the ones the campaign will run.

use holon_lens::lens::{diffusion, hbonds, hbonds_periodic, rdf_oo};
use holon_lens::traj::{BondSet, Frame, Header, Trajectory, AU_TIME_FS};
use holon_render::field::{WATER_PIN_R_BOHR, WATER_PIN_THETA_RAD};
use holon_render::seam::{SeamModel, FREE};
use holon_render::sim::{Boundary, BoundaryRefusal, INTRA_UNIT_REACH, SEAM_REACH_BUDGET};
use holon_render::waterbox::{bcc_sites, bohr2_per_fs_to_cm2_per_s, cell_edge_bohr, first_peak, liquid_box};

#[path = "common/field2_scenes.rs"]
mod field2_scenes;
use field2_scenes::scene;

/// The freeze's state point (§0) and seed.
const DENSITY_G_CM3: f64 = 0.997;
const N_CELLS: usize = 4;
const SEED: u64 = 0x4c49_5155_4944;

/// FIELD-9's harvested law, DECLARED here (`conformance/water_observatory/field9/wall9.json`).
fn declared_law() -> SeamModel {
    SeamModel {
        a: 948.0,
        b: 2.4,
        p: 16.26,
        c: 2.06,
        a_oh: 22.59,
        b_oh: 2.2,
        a_hh: 1.525,
        b_hh: 1.75,
        ..SeamModel::NO_WALL
    }
}

fn min_image(a: [f64; 3], b: [f64; 3], l: f64) -> f64 {
    let mut s = 0.0;
    for k in 0..3 {
        let mut d = b[k] - a[k];
        d -= l * (d / l).round();
        s += d * d;
    }
    s.sqrt()
}

/// §0's box: 128 waters, 384 atoms, the edge from the DENSITY, the lattice from the edge,
/// and every monomer EMBED-1's pin exactly.
#[test]
fn the_liquid_box_is_the_state_points_box() {
    let (sp, pos, l) = liquid_box(N_CELLS, DENSITY_G_CM3, SEED);
    assert_eq!(sp.len(), 384, "384 atoms");
    assert_eq!(pos.len(), 384);
    let oxy: Vec<usize> = (0..384).filter(|&i| sp[i].z == 8).collect();
    assert_eq!(oxy.len(), 128, "128 oxygens");
    assert_eq!((0..384).filter(|&i| sp[i].z == 1).count(), 256, "256 hydrogens");

    // the edge is the density's, to the freeze's printed precision
    assert!((l - 29.60).abs() <= 0.01, "cell edge {l:.4} bohr against the freeze's 29.60");
    assert_eq!(l, cell_edge_bohr(128, DENSITY_G_CM3), "the builder's edge IS cell_edge_bohr");

    // the lattice: nearest BCC sites are corner-to-body-centre, (sqrt(3)/2)·(L/4) = L·sqrt(3)/8
    let sites = bcc_sites(N_CELLS, l);
    assert_eq!(sites.len(), 128);
    let expect = l * 3.0f64.sqrt() / 8.0;
    for (k, &o) in oxy.iter().enumerate() {
        // the oxygen IS its site
        assert!(min_image(pos[o], sites[k], l) < 1e-12, "oxygen {k} is off its site");
        let mut nearest = f64::INFINITY;
        for (m, &p) in oxy.iter().enumerate() {
            if m != k {
                nearest = nearest.min(min_image(pos[o], pos[p], l));
            }
        }
        assert!((nearest - expect).abs() < 1e-9, "site {k}: nearest O–O {nearest:.12} against L·sqrt(3)/8 = {expect:.12}");
    }
    assert!((expect - 6.41).abs() < 0.01, "the freeze's 6.41 bohr: {expect:.4}");

    // every monomer is the pin, internally
    let r_oh = WATER_PIN_R_BOHR;
    let r_hh = 2.0 * WATER_PIN_R_BOHR * (0.5 * WATER_PIN_THETA_RAD).sin();
    for w in 0..128 {
        let (o, h1, h2) = (3 * w, 3 * w + 1, 3 * w + 2);
        assert_eq!(sp[o].z, 8);
        assert_eq!(sp[h1].z, 1);
        assert_eq!(sp[h2].z, 1);
        let d = |a: usize, b: usize| {
            let v = [pos[a][0] - pos[b][0], pos[a][1] - pos[b][1], pos[a][2] - pos[b][2]];
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
        };
        assert!((d(o, h1) - r_oh).abs() < 1e-12, "water {w}: O–H {:.15}", d(o, h1));
        assert!((d(o, h2) - r_oh).abs() < 1e-12, "water {w}: O–H {:.15}", d(o, h2));
        assert!((d(h1, h2) - r_hh).abs() < 1e-12, "water {w}: H–H {:.15} against {r_hh:.15}", d(h1, h2));
    }

    // one seed, one box (M-FIXED-POINT-TRAJECTORY), and the orientations really do vary
    let (_, again, l2) = liquid_box(N_CELLS, DENSITY_G_CM3, SEED);
    assert_eq!(pos, again);
    assert_eq!(l, l2);
    let (_, other, _) = liquid_box(N_CELLS, DENSITY_G_CM3, SEED ^ 1);
    assert_ne!(pos, other, "a different seed is a different box");
    assert!(
        (1..128).any(|w| (pos[3 * w + 1][0] - pos[3 * w][0] - (pos[1][0] - pos[0][0])).abs() > 1e-6),
        "the orientations are drawn, not shared"
    );
}

/// L0's static half: the cell the tables refuse is the cell the seam admits, and under the
/// seam the legality radius IS the seam-aware rule to the bit.
#[test]
fn l0_the_128_water_cell_is_refused_by_the_tables_and_admitted_under_the_seam() {
    let (sp, pos, l) = liquid_box(N_CELLS, DENSITY_G_CM3, SEED);
    let model = declared_law();
    let seam_reach = INTRA_UNIT_REACH.max(model.reach(SEAM_REACH_BUDGET));
    let mut s = scene(&sp, &pos, l, 293.0);
    let bare = s.legality_radius();
    eprintln!(
        "L0 (DECLARED law, FIELD-9's coefficients): tables' reach {bare:.4} bohr, seam reach {seam_reach:.4} (= max({INTRA_UNIT_REACH}, law {:.4})), half-edge {:.4}",
        model.reach(SEAM_REACH_BUDGET),
        0.5 * l
    );
    assert!(bare > 0.5 * l, "the tables must refuse this cell for the gate to have two sides");
    match s.set_boundary(Boundary::Periodic) {
        Err(BoundaryRefusal::BreaksPeriodicImages { reach, half_edge }) => {
            assert_eq!(reach.to_bits(), bare.to_bits());
            assert_eq!(half_edge.to_bits(), (0.5 * l).to_bits());
        }
        other => panic!("without the seam the tables' reach must refuse this cell: {other:?}"),
    }

    s.set_field(true, None).expect("the open box admits the field");
    s.set_seam(Some(model)).expect("no acuity frame");
    let units = s.units_reading();
    assert!(units[..s.n].iter().all(|&u| u != FREE), "every atom of the box is inside a unit");
    assert_eq!(
        units[..s.n].iter().enumerate().filter(|(i, &u)| u == *i as u32).count(),
        128,
        "128 units"
    );
    assert_eq!(
        s.legality_radius().to_bits(),
        seam_reach.to_bits(),
        "under the seam the legality radius IS max(INTRA_UNIT_REACH, model.reach(SEAM_REACH_BUDGET)), to the bit"
    );
    assert!(seam_reach <= 0.5 * l, "and it fits the cell: {seam_reach:.4} against {:.4}", 0.5 * l);
    s.set_boundary(Boundary::Periodic).expect("the liquid cell is admitted under the seam");
    assert!(s.pbc_ok(), "and stays legal per pass");
    assert_eq!(s.seam_work.units, 128, "the engine's own posted count agrees with the pure reading");

    // the readouts run on the box the door admitted
    let z: Vec<u32> = (0..s.n).map(|i| s.atoms[i].species.z).collect();
    let p: Vec<[f64; 3]> = (0..s.n).map(|i| [s.atoms[i].x, s.atoms[i].y, s.atoms[i].z]).collect();
    let hb = hbonds_periodic(&p, &z, [l, l, l]).expect("the box has oxygens and hydrogens");
    let rdf = rdf_oo(&p, &z, [l, l, l], 0.1, 0.5 * l).expect("the box has 128 oxygens");
    assert_eq!(rdf.n_o, 128);
    assert!((rdf.rho_o - 128.0 / (l * l * l)).abs() < 1e-15);
    // the BCC start's first shell is at L·sqrt(3)/8, outside the H-bond criterion's O···O
    // window is FALSE — 6.41 < 6.614 — so the lattice does read some bonds; what it may not
    // do is read a shell where there is none
    let first = rdf.r.iter().zip(rdf.g.iter()).find(|(_, &g)| g > 0.0).map(|(r, g)| (*r, *g));
    eprintln!("L0 start: {} periodic H-bonds, first non-empty g_OO bin at {:?}", hb.len(), first);
    let (r0, _) = first.expect("the lattice has a first shell");
    assert!((r0 - l * 3.0f64.sqrt() / 8.0).abs() <= 0.1, "the first shell of the START is the lattice's own: {r0:.3}");
}

/// THE READOUT CHAIN, end to end, on the box the door admits — and the reason it is a GATE
/// and not only a dry run.
///
/// The dry run of `examples/liquid1.rs` on FIELD-9's record VOIDs in settling (that law's
/// boundedness walk already refused it, and its units dissolve at frame 82), so the arm's
/// counted-frame readouts never see data there. They are exercised here instead, on
/// DISPLACED copies of the box rather than on a trajectory: the arithmetic under test is the
/// readouts', not the dynamics', and a fixture that needs no integrator cannot be voided by
/// one. Every piece the arm uses appears below — the accumulated `g_OO`, the declared first
/// peak, both plants of §5, the carrier they act through, and the diffusion reading with its
/// conversion out of atomic units.
#[test]
fn the_readout_chain_runs_end_to_end_on_the_liquid_box() {
    let (sp, pos0, l) = liquid_box(N_CELLS, DENSITY_G_CM3, SEED);
    let cell = [l, l, l];
    let z: Vec<u32> = sp.iter().map(|s| s.z).collect();
    let oxy: Vec<usize> = (0..sp.len()).filter(|&i| z[i] == 8).collect();
    assert_eq!(oxy.len(), 128);

    // A DECLARED disorder: each molecule displaced as ONE OBJECT by an LCG draw of up to
    // `AMP` bohr per axis, then everything folded into the cell. Small on purpose — the
    // fixture has to keep a first shell for the readouts to read, and a lattice shaken by
    // half its spacing has no shell and overlapping molecules. What it does have is what the
    // readouts must survive: a populated first shell whose pairs straddle faces.
    const AMP: f64 = 0.7;
    let mut st: u64 = 0x4c49_5155_4944_5254;
    let mut lcg = || {
        st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((st >> 11) as f64) / ((1u64 << 53) as f64)
    };
    let amp = AMP;
    let mut frames_pos: Vec<Vec<[f64; 3]>> = Vec::new();
    for _ in 0..3 {
        let mut p = pos0.clone();
        for w in 0..128 {
            let d = [amp * (2.0 * lcg() - 1.0), amp * (2.0 * lcg() - 1.0), amp * (2.0 * lcg() - 1.0)];
            for a in 3 * w..3 * w + 3 {
                for c in 0..3 {
                    p[a][c] = p[a][c] + d[c];
                    p[a][c] -= l * (p[a][c] / l).floor();
                }
            }
        }
        frames_pos.push(p);
    }

    // ---- the accumulated g_OO, and PLANT (i): the same lens with the periodicity removed,
    // renormalised from the plant cell's density to the real one (a scalar rescale, so the
    // plant is the histogram's change and nothing else)
    const PLANT_CELL: f64 = 1.0e6;
    let dr = 0.1;
    let r_max = 0.5 * l;
    let mut g_sum: Vec<f64> = Vec::new();
    let mut g_raw_sum: Vec<f64> = Vec::new();
    let mut r_bins: Vec<f64> = Vec::new();
    for p in frames_pos.iter() {
        let t = rdf_oo(p, &z, cell, dr, r_max).expect("128 oxygens");
        let raw = rdf_oo(p, &z, [PLANT_CELL; 3], dr, r_max).expect("128 oxygens");
        let rescale = raw.rho_o / t.rho_o;
        if g_sum.is_empty() {
            g_sum = vec![0.0; t.g.len()];
            g_raw_sum = vec![0.0; t.g.len()];
            r_bins = t.r.clone();
        }
        for k in 0..g_sum.len() {
            g_sum[k] += t.g[k];
            g_raw_sum[k] += raw.g[k] * rescale;
        }
    }
    let f = frames_pos.len() as f64;
    let g: Vec<f64> = g_sum.iter().map(|x| x / f).collect();
    let g_raw: Vec<f64> = g_raw_sum.iter().map(|x| x / f).collect();
    // g -> 1 far from the origin is the normalisation telling the truth about itself
    let tail: f64 = g.iter().zip(r_bins.iter()).filter(|(_, &r)| r > 10.0).map(|(x, _)| *x).sum::<f64>()
        / g.iter().zip(r_bins.iter()).filter(|(_, &r)| r > 10.0).count() as f64;
    assert!((tail - 1.0).abs() < 0.1, "the ideal-gas normalisation must send g to 1 in the tail: {tail:.4}");
    let (r1, h1) = first_peak(&r_bins, &g).expect("the displaced box has a first peak");
    eprintln!("readout chain: first peak at {r1:.2} bohr, height {h1:.3} (bin {dr}); tail g = {tail:.4}");
    assert!(r1 > 0.0 && h1 > 1.0);

    // PLANT (i) fires: without the minimum image the first peak's height must move by > 20 %
    let raw_peak = first_peak(&r_bins, &g_raw);
    let move_i = match raw_peak {
        Some((_, h)) => (h - h1).abs() / h1,
        None => f64::INFINITY,
    };
    eprintln!("plant (i): raw-difference peak {raw_peak:?} against ({r1:.2}, {h1:.3}), moved {move_i:.3} (the ARM's threshold is 0.20)");
    assert!(move_i > 0.2, "plant (i) must move the first peak by more than 20 %: {move_i:.4}");

    // ---- PLANT (ii): the open-box lens on the same wrapped positions must LOSE bonds.
    //
    // The ARM's criterion is a magnitude — the count per molecule falls by more than 20 % —
    // and that number is about WATER. This fixture is a shaken lattice, so the magnitude is
    // reported here and the assertion is the exact MECHANISM instead: a bond whose O···O
    // vector crosses a face has a RAW separation over half a box edge, which is past the
    // criterion's own `HB_R_OO_BOHR`, so the open-box lens cannot report that donor-acceptor
    // pair at all. Every such bond must be missing from the open list, and there must be some.
    let mut hb = 0.0f64;
    let mut hb_open = 0.0f64;
    let mut crossing_bonds = 0usize;
    for p in frames_pos.iter() {
        let per_frame = hbonds_periodic(p, &z, cell).expect("O and H present");
        let open_frame = hbonds(p, &z).expect("O and H present");
        for b in per_frame.iter() {
            let crossed = (0..3).any(|c| ((p[b.acceptor_o][c] - p[b.donor_o][c]) / l).round() != 0.0);
            if crossed {
                crossing_bonds += 1;
                assert!(
                    !open_frame.iter().any(|o| o.donor_o == b.donor_o && o.hydrogen == b.hydrogen && o.acceptor_o == b.acceptor_o),
                    "a bond across a face cannot be in the open-box lens's list: {b:?}"
                );
            }
        }
        hb += per_frame.len() as f64;
        hb_open += open_frame.len() as f64;
    }
    let (per, per_open) = (hb / f / 128.0, hb_open / f / 128.0);
    let fall = (per - per_open) / per;
    eprintln!("plant (ii): {per:.4} H-bonds/molecule under the minimum image, {per_open:.4} open-box, fell {fall:.3} (the ARM's threshold is 0.20, on water; this fixture is a shaken lattice); {crossing_bonds} bonds cross a face");
    assert!(per > 0.0, "the fixture must have hydrogen bonds for the plant to remove any");
    assert!(crossing_bonds > 0, "the plant acts on face-crossing bonds and the fixture must have some");
    assert!(per_open < per, "the open-box lens must strictly lose bonds: {per_open:.4} against {per:.4}");

    // ---- the CARRIER (§5): first-shell O–O pairs whose minimum-image vector crosses a face.
    // The arm takes the shell from the FREEZE (3.2 angstrom, water's); this fixture is a
    // shaken lattice and not water, so it takes the shell from its OWN measured first peak —
    // the same arithmetic at the radius the fixture actually has one.
    let first_shell = r1 + 0.5;
    eprintln!("carrier shell: {first_shell:.3} bohr (the fixture's own first peak + 0.5; the ARM uses the freeze's 3.2 A = {:.3} bohr)", 3.2 / holon_render::waterbox::BOHR_ANGSTROM);
    let (mut pairs, mut crossing) = (0u64, 0u64);
    for p in frames_pos.iter() {
        for (ii, &i) in oxy.iter().enumerate() {
            for &j in oxy[ii + 1..].iter() {
                let (mut d2, mut crossed) = (0.0, false);
                for c in 0..3 {
                    let raw = p[j][c] - p[i][c];
                    let shift = (raw / l).round();
                    if shift != 0.0 {
                        crossed = true;
                    }
                    let d = raw - l * shift;
                    d2 += d * d;
                }
                if d2 < first_shell * first_shell {
                    pairs += 1;
                    if crossed {
                        crossing += 1;
                    }
                }
            }
        }
    }
    let carrier = crossing as f64 / pairs as f64;
    eprintln!("carrier: {crossing}/{pairs} first-shell O–O pairs cross a face = {carrier:.4} (floor 0.05)");
    assert!(carrier >= 0.05, "the sector both plants act on must be nonzero: {carrier:.4}");

    // ---- R3's path: the trajectory, the lens, and the ONE conversion out of atomic units.
    // A PLANTED walk with a known constant: per-frame displacement uniform on [−h, h] per
    // axis gives MSD(lag) = lag·h² and therefore D = h²/(6·dt_fs) bohr²/fs.
    let h = 0.5f64;
    let dt_au = 100.0f64;
    let nf = 40usize;
    let mut walk: Vec<[f64; 3]> = oxy.iter().map(|&i| pos0[i]).collect();
    let mut frames: Vec<Frame> = Vec::new();
    for k in 0..nf {
        if k > 0 {
            for p in walk.iter_mut() {
                for c in 0..3 {
                    p[c] += h * (2.0 * lcg() - 1.0);
                }
            }
        }
        frames.push(Frame {
            index: k as u64,
            time: k as f64 * dt_au,
            temperature: 293.0,
            bonds: BondSet::empty(),
            pos: walk.clone(),
            vel: vec![[0.0; 3]; walk.len()],
        });
    }
    let traj = Trajectory {
        header: Header { seed: SEED, n_atoms: oxy.len(), dims: 3, substeps: 1, n_frames: nf, dt: dt_au, box_w: l, box_h: l, box_d: l, z: vec![8; oxy.len()] },
        frames,
    };
    let d_bohr2_fs = diffusion(&traj, nf / 4).expect("a clean random walk is diffusive");
    let conv = bohr2_per_fs_to_cm2_per_s();
    let d_cm2_s = d_bohr2_fs * conv;
    let planted = h * h / (6.0 * dt_au * AU_TIME_FS);
    eprintln!("R3 path: D = {d_bohr2_fs:.6e} bohr²/fs = {d_cm2_s:.4e} cm²/s; planted {planted:.6e} bohr²/fs; conversion {conv:.9e}");
    assert!((d_bohr2_fs - planted).abs() / planted < 0.1, "the lens must recover the planted walk: {d_bohr2_fs:.6e} against {planted:.6e}");
    // the conversion is the engine's own bohr, squared, over a femtosecond — not a typed number
    let bohr_cm = holon_render::sim::BOHR_M * 100.0;
    assert_eq!(conv, bohr_cm * bohr_cm / 1.0e-15);
    // and the two constants for the atomic time unit agree to the lens's precision
    let au_s_from_lens = AU_TIME_FS * 1.0e-15;
    assert!(((au_s_from_lens - holon_render::sim::AU_TIME_S) / holon_render::sim::AU_TIME_S).abs() < 1e-9);
}

/// LIQUID-1 (2026-09-06): THE THREE-BODY FORCE UNDER THE MINIMUM IMAGE. A water molecule
/// straddling a face of the periodic cell must feel the same forces as the same molecule at
/// the centre, and must not heat from rest. Before the fix `push_side` took the raw
/// coordinate difference for a triple's force direction while the energy took the folded
/// separation: a straddling molecule at rest heated to 9,000 K in twenty steps, and the
/// 128-water box lost a unit at settling frame 82 on two different laws. The door is
/// bypassed on purpose here (the tables' reach exceeds the half-edge; the test is about
/// the intra-unit triple, which the seam rule does not touch).
#[test]
fn a_water_straddling_a_face_feels_the_centre_molecules_forces_and_does_not_heat() {
    use holon_chem::elements::by_symbol;
    use holon_chem::embed::water_centers;
    use holon_render::channel::Row;
    let (o, h) = (by_symbol("O").unwrap(), by_symbol("H").unwrap());
    let l = 29.59363131;
    let c = water_centers(1.9435738400, 1.6887434037);
    let build = |shift: [f64; 3]| {
        let pos: Vec<[f64; 3]> = c.iter().map(|p| [p[0] + shift[0], p[1] + shift[1], p[2] + shift[2]]).collect();
        let mut s = scene(&[o, h, h], &pos, l, 293.0);
        for i in 0..s.n {
            s.atoms[i].vx = 0.0;
            s.atoms[i].vy = 0.0;
            s.atoms[i].vz = 0.0;
        }
        s.thermostat_on = false;
        s.boundary = Boundary::Periodic;
        s.compute_forces();
        s
    };
    let mut centre = build([0.5 * l; 3]);
    let fc: Vec<(f64, f64, f64)> = (0..3).map(|i| centre.internal_force(i)).collect();
    for shift in [[0.3, 0.2, 0.1], [0.3, 0.5 * l, 0.5 * l], [0.5 * l, 0.5 * l, l - 0.4]] {
        let mut s = build(shift);
        // the wrapped image of the same molecule: fold every atom into the cell first
        s.step();
        s.step();
        let mut w = build(shift);
        for i in 0..w.n {
            let (x, y, z) = w.geom().wrap((w.atoms[i].x, w.atoms[i].y, w.atoms[i].z));
            w.atoms[i].x = x;
            w.atoms[i].y = y;
            w.atoms[i].z = z;
        }
        w.compute_forces();
        assert!((w.row(Row::Pair) - centre.row(Row::Pair)).abs() < 1e-12 && (w.row(Row::Three) - centre.row(Row::Three)).abs() < 1e-12, "energy rows under the minimum image");
        for i in 0..3 {
            let f = w.internal_force(i);
            for (a, b) in [(f.0, fc[i].0), (f.1, fc[i].1), (f.2, fc[i].2)] {
                assert!((a - b).abs() < 1e-12, "atom {i}: force {a:.6e} vs the centre molecule's {b:.6e} (shift {shift:?})");
            }
        }
        // twenty free steps: the straddler's temperature equals the centre molecule's (the
        // pin geometry is not the tables' exact minimum, so both move a little; before the
        // fix the straddler read 9,246 K here and the centre 0.000)
        let mut ref_c = build([0.5 * l; 3]);
        for _ in 0..20 {
            w.step();
            ref_c.step();
        }
        let (tw, tc) = (w.temperature(), ref_c.temperature());
        assert!((tw - tc).abs() <= 1e-9 * tc.max(1e-12) + 1e-12, "a straddling molecule at rest read {tw:.6e} K after 20 steps against the centre molecule's {tc:.6e} (shift {shift:?})");
        assert!(tw < 0.1, "a molecule at rest heated to {tw:.3} K in 20 steps (shift {shift:?})");
    }
    let _ = centre.temperature();
}

// ------------------------------------------------------------ THE CLOSURE ADAPTER'S GATE
//
// `holon-render::closure` hands this crate's unit reading to `holon-closure`, so that a
// water unit here, a bonded pair in `holon-lattice` and an H-bond component in `holon-lens`
// are ONE type. The adapter must reproduce the reading BIT FOR BIT or one rule has become
// two, and the box to reproduce it on is L0's — the 128-water start box `door.json` records.
//
// The test lives HERE rather than in a file of its own for a reason worth stating: this
// binary already pays for `banked()`, which generates the pair curves and the trimer table,
// and a second binary that builds this box would pay it again. It also means the state
// point, the seed and the declared law are declared ONCE in this file instead of twice in
// two, which is the same fence the adapter itself exists to keep.

/// 128 units on the LIQUID-1 start box, and the same units the existing reading gives.
#[test]
fn the_closures_are_the_units_reading_on_the_liquid1_start_box() {
    use holon_render::closure::{free_atoms, unit_closures, units_from_closures, TIER};

    let (sp, pos, l) = liquid_box(N_CELLS, DENSITY_G_CM3, SEED);
    let mut s = scene(&sp, &pos, l, 293.0);
    s.set_field(true, None).expect("the open box admits the field");
    s.set_seam(Some(declared_law())).expect("no acuity frame");

    let units = s.units_reading();
    let cs = unit_closures(&s);

    assert_eq!(cs.len(), 128, "128 units on the start box, as door.json records");
    assert_eq!(TIER, holon_closure::TierId::MOLECULAR);
    assert!(cs.iter().all(|(_, c)| c.tier == TIER));
    assert!(cs.iter().all(|(_, c)| c.len() == 3), "a water unit is an oxygen and two hydrogens");
    assert!(free_atoms(&units[..s.n]).is_empty(), "every atom of the box is inside a unit");

    // BIT FOR BIT: the reading rebuilt from the closures IS the reading.
    assert_eq!(units_from_closures(s.n, &cs), units[..s.n], "the closures are a different assignment");

    // and each closure's root is the oxygen the reading already names the unit by
    for (root, c) in &cs {
        assert_eq!(units[*root as usize], *root, "a root that is not its own unit's root");
        assert_eq!(sp[*root as usize].z, 8, "a unit rooted on something that is not an oxygen");
        assert_eq!(c.members().iter().filter(|&&m| sp[m as usize].z == 1).count(), 2);
        for &m in c.members() {
            assert_eq!(units[m as usize], *root);
        }
    }

    // a closure the adapter built carries no ledger and no rent: the molecular tier's six
    // channels were measured on the DIMER, and a row filled in here would be a claim nobody
    // made (`holon_closure::LedgerRow` keeps "measured zero" and "not served" apart)
    assert!(cs[0].1.ledger.is_empty());
    assert_eq!(cs[0].1.rent, None);
}

/// The free-atom case the start box does not exercise, and the one a dissolving unit
/// produces: a free hydrogen is a unit that came apart, never a closure of one.
#[test]
fn a_free_atom_is_reported_and_never_folded_in_as_a_closure_of_one() {
    use holon_render::closure::{closures_from_units, free_atoms, units_from_closures};
    let units: Vec<u32> = vec![0, 0, 0, FREE, 4, 4, 4];
    let cs = closures_from_units(&units);
    assert_eq!(cs.len(), 2);
    assert_eq!(cs[0].1.members(), &[0, 1, 2]);
    assert_eq!(cs[1].1.members(), &[4, 5, 6]);
    assert_eq!(free_atoms(&units), vec![3]);
    assert_eq!(units_from_closures(units.len(), &cs), units);
}
