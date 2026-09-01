//! DETERMINISM: checkpoints are exact, and a restored scene replays bit for bit.
//!
//! FSD-W1 WB-5.4. The claim is narrow on purpose and the tests are written to its width:
//! ON ONE BUILD, a checkpoint round-trips exactly, a restored trajectory is
//! indistinguishable from an uninterrupted one down to the last bit, and a checkpoint
//! taken against different physics is REFUSED rather than continued.
//!
//! The device-class half of WB-5.4 is not tested here and cannot be: two machines whose
//! `sqrt` or `exp` round differently are two charts, and no serializer can bridge them.
//! `holon-resource`'s D0 says the class belongs to the ARTIFACT — so what this file gates
//! is the part that is this crate's to keep.
//!
//! Plants:
//!
//! | plant | what it breaks | what must fire |
//! |---|---|---|
//! | P-R1 | one byte of the checkpoint | `CheckpointError::Corrupt` |
//! | P-R2 | the physics under a checkpoint | `CheckpointError::PhysicsMismatch` |
//! | P-R3 | a state field dropped from the snapshot | the replay-identity comparison |

use holon_render::checkpoint::CheckpointError;
use holon_render::sim::{Boundary, Dims, Sim};

fn potential_source() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/viewer/h2_potential.json");
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("cannot read {path}: {e}. Run: cargo run -p holon-render --example make_placeholder")
    })
}

fn loaded_sim() -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    holon_render::json::load_into(s.table_mut(), &potential_source()).expect("table loads");
    s.adopt_table_timescale();
    s
}

/// A scene with something actually going on in it: three dimensions, walls (so the impulse
/// ledger is non-trivial), a thermostat (so a receipt column moves), and enough atoms that
/// bonds form and dissolve.
fn busy_scene() -> Box<Sim> {
    let mut s = loaded_sim();
    s.dims = Dims::Three;
    s.boundary = Boundary::Walls;
    s.reset(24);
    s.thermostat_on = true;
    s.target_temperature = 450.0;
    s
}

/// Everything about a scene that a replay must reproduce. Compared BY BITS: a comparison
/// with a tolerance would pass on a scene that had drifted and call it identical.
fn fingerprint(s: &Sim) -> Vec<u64> {
    let mut out = Vec::new();
    for i in 0..s.n {
        let a = &s.atoms[i];
        for v in [a.x, a.y, a.z, a.vx, a.vy, a.vz] {
            out.push(v.to_bits());
        }
        out.push(a.species.z as u64);
    }
    for v in [
        s.e_kin,
        s.e_pair,
        s.e_three,
        s.e_four,
        s.e_wall,
        s.e_spring,
        s.e_grav,
        s.g,
        s.w_ext,
        s.work.hand,
        s.work.thermostat,
        s.work.barostat,
        s.l0,
        s.time,
        s.drift_peak,
        s.momentum_residual_peak,
        s.e_ref,
        s.j_ext.0,
        s.j_ext.1,
        s.j_ext.2,
        s.dt(),
    ] {
        out.push(v.to_bits());
    }
    out.push(s.steps);
    out.push(s.frame);
    out.push(s.holons.molecule_count() as u64);
    out.push(s.holons.census.formations);
    out.push(s.holons.census.dissolutions);
    out.push(s.bonded_count() as u64);
    out
}

/// A checkpoint of a scene, restored into a fresh scene, is the same scene.
#[test]
fn a_checkpoint_round_trips_exactly() {
    let mut s = busy_scene();
    for _ in 0..30 {
        s.step_frame(32);
    }
    let ck = s.checkpoint();
    let before = fingerprint(&s);

    let mut t = loaded_sim();
    t.restore(&ck).expect("the restore was refused");
    let after = fingerprint(&t);
    assert_eq!(
        before, after,
        "the restored scene is not the scene that was checkpointed"
    );

    // And the checkpoint of the restored scene is byte-identical to the original, which is
    // the stronger statement: not merely that the fields survived, but that nothing the
    // snapshot carries was reconstructed differently.
    let ck2 = t.checkpoint();
    assert_eq!(
        ck.bytes, ck2.bytes,
        "checkpoint -> restore -> checkpoint is not a fixed point"
    );
}

/// THE REPLAY IDENTITY. Stopping and resuming is not an event: a run checkpointed at
/// frame 30 and continued to frame 60 must land on exactly the bits an uninterrupted run
/// to frame 60 lands on.
#[test]
fn a_restored_run_replays_bit_for_bit() {
    let mut straight = busy_scene();
    for _ in 0..30 {
        straight.step_frame(32);
    }
    let ck = straight.checkpoint();
    for _ in 0..30 {
        straight.step_frame(32);
    }
    let straight_end = fingerprint(&straight);

    let mut resumed = loaded_sim();
    resumed.restore(&ck).expect("the restore was refused");
    for _ in 0..30 {
        resumed.step_frame(32);
    }
    let resumed_end = fingerprint(&resumed);

    assert_eq!(
        straight_end.len(),
        resumed_end.len(),
        "the resumed scene has a different shape"
    );
    let first_difference = straight_end
        .iter()
        .zip(resumed_end.iter())
        .position(|(a, b)| a != b);
    assert!(
        first_difference.is_none(),
        "the resumed trajectory diverged from the uninterrupted one at word {}",
        first_difference.unwrap()
    );
}

/// The same seed and the same scene give the same trajectory twice. There is no RNG in
/// this engine at all — the openers are deterministic by construction — so this gate is
/// really the statement that nothing ELSE (iteration order over a map, an address, a
/// timing) has leaked into the numbers. The cell list is the new candidate for exactly
/// that, which is why this test runs a scene big enough to engage it.
#[test]
fn the_same_scene_twice_is_the_same_trajectory() {
    let run = || {
        let mut s = loaded_sim();
        s.dims = Dims::Three;
        s.boundary = Boundary::Periodic;
        s.width = 48.0;
        s.height = 48.0;
        s.depth = 48.0;
        s.resize_storage(216);
        for i in 0..216 {
            let (ix, iy, iz) = (i % 6, (i / 6) % 6, i / 36);
            s.atoms[i].x = (ix as f64 + 0.5) * 8.0;
            s.atoms[i].y = (iy as f64 + 0.5) * 8.0;
            s.atoms[i].z = (iz as f64 + 0.5) * 8.0;
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            s.atoms[i].vx = sign * 0.003;
            s.atoms[i].vy = sign * 0.001;
            s.atoms[i].vz = sign * 0.002;
        }
        s.sync_species();
        s.rebase();
        assert!(s.set_pair_cutoff(1e-6));
        assert_eq!(
            s.route(),
            holon_render::cells::Route::Cells,
            "the cell list did not engage, so this test is not exercising it"
        );
        for _ in 0..20 {
            s.step_frame(16);
        }
        fingerprint(&s)
    };
    assert_eq!(run(), run(), "two identical runs produced different bits");
}

/// PLANT P-R1: a single flipped byte. The checksum is the only thing standing between a
/// corrupt file and a trajectory that continues from a state no run produced.
#[test]
fn p_r1_a_corrupted_checkpoint_is_refused() {
    let mut s = busy_scene();
    for _ in 0..10 {
        s.step_frame(32);
    }
    let ck = s.checkpoint();

    // Control: intact, it restores.
    let mut t = loaded_sim();
    assert!(t.restore(&ck).is_ok(), "the control checkpoint was refused");

    // Every byte of the body, one at a time, would be too slow; a spread across the header,
    // the atoms and the ledger is enough to show the checksum covers the whole file.
    let n = ck.bytes.len();
    for at in [12usize, 40, n / 3, n / 2, (2 * n) / 3, n - 12] {
        let mut bad = ck.clone();
        bad.bytes[at] ^= 0x01;
        let mut t = loaded_sim();
        let before = fingerprint(&t);
        assert_eq!(
            t.restore(&bad),
            Err(CheckpointError::Corrupt),
            "a flipped byte at offset {at} of {n} was accepted"
        );
        assert_eq!(
            fingerprint(&t),
            before,
            "the refused restore modified the scene anyway — a refusal must leave the scene \
             it refused to overwrite"
        );
    }
}

/// PLANT P-R2: the same trajectory, a different curve underneath it. This is the
/// dangerous one, because it produces numbers rather than errors: the state restores
/// cleanly and every subsequent force is computed on physics the run never saw.
#[test]
fn p_r2_a_checkpoint_from_different_physics_is_refused() {
    let mut s = busy_scene();
    for _ in 0..10 {
        s.step_frame(32);
    }
    let ck = s.checkpoint();

    // A scene with NO curve loaded: a different physics in the most obvious way.
    let mut bare = Box::new(Sim::empty());
    match bare.restore(&ck) {
        Err(CheckpointError::PhysicsMismatch { expected, found }) => {
            assert_ne!(expected, found);
        }
        other => panic!("an empty bank accepted a checkpoint taken against a loaded one: {other:?}"),
    }

    // And a subtler one: the same curve with a single knot moved. The digest is functional,
    // so it must notice a change in what the table COMPUTES rather than in how it is
    // stored.
    let mut nudged = loaded_sim();
    let d_e_before = nudged.table().d_e;
    {
        // Rewrite the SAME curve with one interior energy knot moved by a part in 10^6 —
        // far too small to see in any trajectory over ten frames, and exactly the kind of
        // change that must not be silently continued across. Pushed through the ordinary
        // knot door rather than by reaching into the table's fields, so the resulting
        // curve is one the loader would have accepted from a file.
        let reference = loaded_sim();
        let src = reference.table();
        let count = src.knots();
        let mid = count / 2;
        let rows: Vec<(f64, f64, f64)> = (0..count)
            .map(|i| (src.knot_r(i), src.knot_u(i), -src.knot_d(i)))
            .collect();
        let (r_e, d_e, asym) = (src.r_e, src.d_e, 0.0);
        let t = nudged.table_mut();
        assert!(t.begin(count));
        for (i, &(r, e, f)) in rows.iter().enumerate() {
            let e = if i == mid { e * (1.0 + 1e-6) } else { e };
            assert!(t.knot(i, r, e, f), "knot {i} was refused");
        }
        let status = t.finish(r_e, d_e, asym);
        assert!(
            nudged.table().is_loaded(),
            "the rewritten curve did not load: {status:?}"
        );
    }
    assert_eq!(
        nudged.table().d_e,
        d_e_before,
        "the harness moved more than the one knot it meant to"
    );
    assert!(
        matches!(
            nudged.restore(&ck),
            Err(CheckpointError::PhysicsMismatch { .. })
        ),
        "a curve with a knot moved by 1e-6 was accepted as the same physics"
    );
}

/// A checkpoint written by a different format version is refused rather than read as this
/// one. The field layout is what a version names, and reading a moved layout produces a
/// plausible scene made of the wrong fields.
#[test]
fn a_checkpoint_from_another_version_is_refused() {
    let mut s = busy_scene();
    s.step_frame(32);
    let ck = s.checkpoint();
    let mut bad = ck.clone();
    // Bump the version word (bytes 8..12), then repair the checksum so that VERSION is what
    // is being tested rather than corruption.
    let v = holon_render::checkpoint::CHECKPOINT_VERSION + 1;
    bad.bytes[8..12].copy_from_slice(&v.to_le_bytes());
    let body_len = bad.bytes.len() - 8;
    let sum = {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in &bad.bytes[..body_len] {
            h ^= b as u64;
            h = h.wrapping_mul(0x1000_0000_01b3);
        }
        h
    };
    bad.bytes[body_len..].copy_from_slice(&sum.to_le_bytes());

    let mut t = loaded_sim();
    assert_eq!(t.restore(&bad), Err(CheckpointError::Version(v)));
}

/// PLANT P-R3: a field that stops being carried. Demonstrated on the ledger's origin
/// `l0`, because dropping it is the shape of the mistake — the scene restores, the
/// trajectory continues correctly, and only the DRIFT reading is wrong, which is the one
/// number the run exists to report.
///
/// The plant is applied to the restored scene rather than to the serializer, because a
/// serializer that has been edited to drop a field is a serializer whose test no longer
/// compiles. What is demonstrated is that the fingerprint comparison the replay gate uses
/// can SEE such a loss — which is the property that makes that gate a gate.
#[test]
fn p_r3_a_dropped_state_field_is_visible_to_the_replay_gate() {
    let mut s = busy_scene();
    for _ in 0..10 {
        s.step_frame(32);
    }
    let ck = s.checkpoint();
    let reference = fingerprint(&s);

    let mut t = loaded_sim();
    t.restore(&ck).expect("restore");
    assert_eq!(fingerprint(&t), reference, "the control must match");

    // The plant: the ledger origin is lost, as it would be if `l0` had been left out of
    // the field list. Nothing about the atoms changes.
    t.l0 = 0.0;
    assert_ne!(
        fingerprint(&t),
        reference,
        "the replay fingerprint cannot see a lost ledger origin, so it would not have \
         caught the serializer dropping it"
    );
    assert!(
        s.drift() != t.drift(),
        "the drift reading did not move when its origin was lost"
    );
}
