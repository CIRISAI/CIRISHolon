//! THE CARRIER-V2 PLANTS: P-1, P-2 and P-3 of `CARRIER_V2_PREREG.md` §3.
//!
//! G1 and G2 are this campaign's whole compatibility claim, and a bit-identity check that
//! has never been seen to fail is a fence rather than a gate. These three plants
//! manufacture the exact silent reinterpretation the version tag exists to prevent and
//! require the campaign's OWN instruments — `Trajectory2::write_as_v1` for G1 and
//! `Trajectory2::diff_against_v1` for G2, not paraphrases of them — to convict it.
//!
//! Each plant names its carrier and the sector it must be nonzero in, and each asserts
//! not only that the gate fires but WHERE, because a gate that fires in the wrong sector
//! is a defect in the instrument wearing a pass.
//!
//! P-4 through P-9 live in `src/traj2.rs`'s unit tests, beside the code they exercise.

use holon_lens::traj::{BondSet, Header, TrajWriter, Trajectory};
use holon_lens::traj2::Trajectory2;
use std::path::{Path, PathBuf};

fn tmp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("carrier-plant-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A v1 file shaped like the banked ones: twelve atoms, four oxygens first, a bond set
/// that moves, and a timestep that HALVES part way through — the real trajectories' shape,
/// because a fixture with a constant timestep is the fixture that let a half-length window
/// pass as a full one (`traj.rs`'s own note).
fn banked_shape(path: &Path, n_frames: usize) -> Header {
    let n_atoms = 12;
    let h = Header {
        seed: 0x0000_0000_5341_5422,
        n_atoms,
        dims: 2,
        substeps: 64,
        n_frames,
        dt: 1.0772,
        box_w: 34.6,
        box_h: 20.8,
        box_d: 24.0,
        z: (0..n_atoms).map(|i| if i < 4 { 8 } else { 1 }).collect(),
    };
    let mut w = TrajWriter::create(path, &h).unwrap();
    let mut t = 0.0f64;
    for f in 0..n_frames {
        let pos: Vec<[f64; 3]> = (0..n_atoms)
            .map(|i| {
                [
                    (i % 4) as f64 * 8.65 + f as f64 * 0.013,
                    (i / 4) as f64 * 6.93,
                    12.0, // the plane the seventeen banked trajectories never leave
                ]
            })
            .collect();
        let vel: Vec<[f64; 3]> = (0..n_atoms)
            .map(|i| [0.0011 * i as f64, -0.0007 * (i % 3) as f64, 0.0])
            .collect();
        // A bond set that changes frame to frame, including the HIGHEST LEGAL pair
        // index for this scene: `pair_index(12, 10, 11) = 65`. Not 119 — a twelve-atom
        // scene has 66 pairs, and a bit above them is a file contradicting itself, which
        // `from_v1` now refuses. The first draft of this fixture set bit 119 and that is
        // how the refusal came to exist.
        let bits: u128 = 0b1011 | (1u128 << (f % 40)) | (1u128 << 65);
        w.push(f as u64, t, 3000.0 - 2.0 * f as f64, &BondSet::from_bits(bits), &pos, &vel)
            .unwrap();
        t += if f < n_frames / 2 { 68.9414 } else { 34.4707 };
    }
    w.finish().unwrap();
    h
}

/// **P-1 — one flipped byte.**
///
/// Carrier: a COPY of a v1 file. Sector it must be nonzero in: the file digest.
///
/// The mutation is applied to a byte deep inside a frame payload, where a reader that
/// round-tripped structurally but reinterpreted a float would still produce a file of the
/// right LENGTH. Length equality is what a careless digest check degrades into, so the
/// plant is placed exactly where length cannot catch it.
#[test]
fn p1_one_flipped_byte_is_convicted_by_the_digest() {
    let d = tmp("p1");
    let good = d.join("good.traj");
    let bad = d.join("bad.traj");
    banked_shape(&good, 12);

    let mut bytes = std::fs::read(&good).unwrap();
    // Deep inside a frame, and at a KNOWN field: the low mantissa byte of frame 3, atom
    // 5's x. Picked by arithmetic rather than by `len / 2`, so the sector this plant is
    // supposed to fire in is the sector it is actually placed in.
    let n_atoms = 12usize;
    let header_len = 8 + 4 + 4 + 4 + 4 + 8 + 8 + 8 * 4 + 4 * n_atoms;
    let frame_len = 8 + 8 + 8 + 16 + 8 * 6 * n_atoms;
    let at = header_len + 3 * frame_len + 8 + 8 + 8 + 16 + 5 * 6 * 8;
    let before = bytes[at];
    bytes[at] ^= 0x01;
    assert_ne!(bytes[at], before);
    std::fs::write(&bad, &bytes).unwrap();
    assert_eq!(
        std::fs::metadata(&good).unwrap().len(),
        std::fs::metadata(&bad).unwrap().len(),
        "the plant must be invisible to a length check, or it tests the wrong thing"
    );

    // G1's instrument, on both. The clean file reproduces its bytes; the planted one
    // reproduces ITS bytes and therefore differs from the clean file's.
    let rt = |src: &Path, dst: &Path| {
        Trajectory2::read(src).unwrap().write_as_v1(dst).unwrap();
        std::fs::read(dst).unwrap()
    };
    let good_rt = rt(&good, &d.join("g.rt"));
    let bad_rt = rt(&bad, &d.join("b.rt"));
    assert_eq!(good_rt, std::fs::read(&good).unwrap(), "G1 on the clean file");
    assert_eq!(bad_rt, bytes, "the reader is faithful to whatever it is given");
    assert_ne!(
        good_rt, bad_rt,
        "P-1 DID NOT FIRE: one flipped byte survived the round trip"
    );

    // And the sector: the disagreement is in a POSITION, not in the header or the bonds.
    let one = Trajectory::read(&bad).unwrap();
    let two = Trajectory2::read(&good).unwrap();
    let diff = two.diff_against_v1(&one);
    assert!(!diff.is_empty(), "P-1 DID NOT FIRE in the comparator");
    assert!(
        diff.iter().all(|s| s.contains("pos") || s.contains("vel")),
        "P-1 fired in the wrong sector: {diff:?}"
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **P-2 — the header's two `u32`s transposed.**
///
/// Carrier: a mutant reader that takes `dims` and `substeps` in the opposite order.
/// Sector it must be nonzero in: the two header integers.
///
/// This is the plant this freeze cares most about. `dims = 2, substeps = 64` and
/// `dims = 64, substeps = 2` are both perfectly well-formed headers of the right length,
/// and every downstream number — every position, every bond bit, every timestamp — is
/// IDENTICAL under the transposition. Nothing but a field-by-field comparison can see it,
/// which is why G2 exists beside G1 rather than being folded into it.
#[test]
fn p2_a_transposed_header_is_convicted_and_both_fields_are_named() {
    let d = tmp("p2");
    let path = d.join("t.traj");
    banked_shape(&path, 6);

    let one = Trajectory::read(&path).unwrap();
    let honest = Trajectory2::read(&path).unwrap();
    assert!(
        honest.diff_against_v1(&one).is_empty(),
        "the honest reader must agree before the mutant can be shown to disagree"
    );

    // THE MUTANT READER: the same lift with the two header words swapped, which is
    // exactly what reading them in the opposite order produces.
    let mut mutant = Trajectory2::read(&path).unwrap();
    std::mem::swap(&mut mutant.header.dims_declared, &mut mutant.header.substeps);
    assert_ne!(
        mutant.header.dims_declared, honest.header.dims_declared,
        "the transposition must actually change the header, or the plant is inert"
    );

    let diff = mutant.diff_against_v1(&one);
    assert!(!diff.is_empty(), "P-2 DID NOT FIRE: a transposed header passed G2");
    assert!(
        diff.iter().any(|s| s.starts_with("dims:")),
        "P-2 fired without naming `dims`: {diff:?}"
    );
    assert!(
        diff.iter().any(|s| s.starts_with("substeps:")),
        "P-2 fired without naming `substeps`: {diff:?}"
    );
    // The sector, stated as a negative too: nothing downstream moved, and a comparator
    // that reported frame differences here would be reporting noise.
    assert!(
        diff.iter().all(|s| s.starts_with("dims:") || s.starts_with("substeps:")),
        "P-2 fired outside the header sector: {diff:?}"
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **P-3 — one bond dropped from one frame.**
///
/// Carrier: a copy of a v1 file with one frame's bitset losing its highest set bit.
/// Sector it must be nonzero in: the bond set of exactly one frame.
///
/// The highest bit, because that is the one a narrower bitset drops — the failure mode
/// v1's own cap exists to refuse, planted here so the v2 path is shown to catch it. A
/// dropped bond bit shows up downstream as a partition that quietly falls apart, which is
/// a defect no census would attribute to its carrier.
#[test]
fn p3_a_dropped_bond_fires_on_exactly_one_frame() {
    let d = tmp("p3");
    let good = d.join("good.traj");
    let bad = d.join("bad.traj");
    banked_shape(&good, 9);

    // Locate frame 4's `bonded` word and clear bit 119.
    let n_atoms = 12usize;
    let header_len = 8 + 4 + 4 + 4 + 4 + 8 + 8 + 8 * 4 + 4 * n_atoms;
    let frame_len = 8 + 8 + 8 + 16 + 8 * 6 * n_atoms;
    let victim = 4usize;
    let at = header_len + victim * frame_len + 8 + 8 + 8;
    let mut bytes = std::fs::read(&good).unwrap();
    let mut word = u128::from_le_bytes(bytes[at..at + 16].try_into().unwrap());
    assert!(word >> 65 & 1 == 1, "the fixture must have the bit to drop");
    word &= !(1u128 << 65);
    bytes[at..at + 16].copy_from_slice(&word.to_le_bytes());
    std::fs::write(&bad, &bytes).unwrap();

    let one = Trajectory::read(&bad).unwrap();
    let two = Trajectory2::read(&good).unwrap();
    let diff = two.diff_against_v1(&one);
    assert!(!diff.is_empty(), "P-3 DID NOT FIRE: a dropped bond passed G2");
    assert_eq!(
        diff.len(),
        1,
        "P-3 must fire ONCE; it fired {} times: {diff:?}",
        diff.len()
    );
    assert!(
        diff[0].contains(&format!("frame {victim} bond set")),
        "P-3 fired on the wrong frame or the wrong sector: {diff:?}"
    );

    // And the pair the drop removed is exactly (10, 11), the last one.
    let clean = Trajectory2::read(&good).unwrap();
    let planted = Trajectory2::read(&bad).unwrap();
    assert!(clean.frames[victim].is_bonded(n_atoms, 10, 11));
    assert!(!planted.frames[victim].is_bonded(n_atoms, 10, 11));
    for f in 0..9 {
        if f != victim {
            assert_eq!(
                clean.frames[f].bonds, planted.frames[f].bonds,
                "the plant leaked into frame {f}"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// The plants are only worth their run time if the instrument they convict is CLEAN on the
/// unplanted carrier. Three negative controls, one per plant.
#[test]
fn the_unplanted_carrier_passes_all_three_gates() {
    let d = tmp("clean");
    let path = d.join("c.traj");
    banked_shape(&path, 20);
    let one = Trajectory::read(&path).unwrap();
    let two = Trajectory2::read(&path).unwrap();
    assert!(two.diff_against_v1(&one).is_empty(), "G2 on a clean file");
    let back = d.join("back.traj");
    two.write_as_v1(&back).unwrap();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        std::fs::read(&back).unwrap(),
        "G1 on a clean file"
    );
    // G5's shape: the fixture is planar by construction and must read EXACTLY so.
    let m = two.measure();
    assert_eq!(m.span[2], 0.0);
    assert_eq!(m.flatness, 0.0);
    assert_eq!(m.dims, 2);
    assert_eq!(m.samples, 20 * 12);
    let _ = std::fs::remove_dir_all(&d);
}
