//! THE FILE PATH, end to end: writer, reader, census.
//!
//! The plants in `plants.rs` build trajectories in memory and hand them straight to the
//! census. That leaves one whole layer untested — the one the real data actually travels
//! through. A bug in the writer's field order, in the reader's, or in the pair-bit
//! packing would leave every plant green and every verdict on a real run wrong, and it
//! would do so silently, because a permuted bond graph is still a bond graph.
//!
//! So these tests write real files, read them back, and check that the verdict on the
//! round-tripped trajectory is the SAME verdict as on the in-memory one.

use holon_lens::census::{self, BlockVerdict, Census, Stakes};
use holon_lens::partition::Mask;
use holon_lens::synthetic::{self, Spec};
use holon_lens::traj::{Trajectory, TrajWriter};
use std::path::PathBuf;

fn oh2() -> Mask {
    Mask::from_bits(0b0000_0011_0001)
}

fn tmpdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("hlens-fp-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn write_and_read(t: &Trajectory, tag: &str) -> Trajectory {
    let dir = tmpdir(tag);
    let path = dir.join("t.traj");
    let mut w = TrajWriter::create(&path, &t.header).unwrap();
    for f in &t.frames {
        w.push(f.index, f.time, f.temperature, &f.bonds, &f.pos, &f.vel)
            .unwrap();
    }
    assert_eq!(w.finish().unwrap() as usize, t.frames.len());
    let back = Trajectory::read(&path).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    back
}

fn verdict_of(t: &Trajectory, m: Mask) -> BlockVerdict {
    match census::run(t, &Stakes::default()) {
        Census::Report(r) => r
            .blocks
            .iter()
            .find(|b| b.block == m)
            .map(|b| b.verdict.clone())
            .unwrap_or_else(|| panic!("block missing")),
        Census::Refused { gate, .. } => panic!("refused at {gate}"),
    }
}

/// The round trip must not change a single reading the census depends on.
#[test]
fn the_file_round_trip_preserves_every_frame_exactly() {
    let mem = synthetic::vibrating_block(
        {
            let mut s = Spec::quench_like(400, vec![8, 8, 8, 8, 1, 1, 1, 1, 1, 1, 1, 1]);
            s.seed = 0x5341_5499;
            s
        },
        oh2(),
        0.4,
        |f| f % 97 != 0,
    );
    let disk = write_and_read(&mem, "exact");
    assert_eq!(disk.header, mem.header);
    assert_eq!(disk.frames.len(), mem.frames.len());
    for (a, b) in mem.frames.iter().zip(disk.frames.iter()) {
        assert_eq!(a.index, b.index);
        assert_eq!(a.bonds, b.bonds, "bond bits at frame {}", a.index);
        assert_eq!(a.time.to_bits(), b.time.to_bits(), "time is bit-exact");
        assert_eq!(a.temperature.to_bits(), b.temperature.to_bits());
        for i in 0..mem.header.n_atoms {
            for c in 0..3 {
                assert_eq!(a.pos[i][c].to_bits(), b.pos[i][c].to_bits());
                assert_eq!(a.vel[i][c].to_bits(), b.vel[i][c].to_bits());
            }
        }
    }
}

/// And the VERDICT survives the trip, which is the thing that actually matters.
#[test]
fn the_census_verdict_is_the_same_on_disk_as_in_memory() {
    let cases: [(&str, Box<dyn Fn(usize) -> bool>); 2] = [
        ("held", Box::new(|_| true)),
        ("breaks", Box::new(|f: usize| f < 600)),
    ];
    for (tag, held) in cases {
        let mut s = Spec::quench_like(1200, vec![8, 8, 8, 8, 1, 1, 1, 1, 1, 1, 1, 1]);
        s.seed = 0x5341_5500;
        let mem = synthetic::vibrating_block(s, oh2(), 0.4, held);
        let disk = write_and_read(&mem, tag);
        assert_eq!(
            verdict_of(&mem, oh2()),
            verdict_of(&disk, oh2()),
            "{tag}: the file layer changed the verdict"
        );
    }
}

/// The bond bits must survive as a GRAPH, not merely as a bit count.
///
/// A permutation of the pair index would preserve the popcount and destroy the partition,
/// and the census would go on reporting confident verdicts about the wrong molecules. So
/// the check is on the induced partition, atom by atom.
#[test]
fn the_bond_graph_survives_the_trip_as_a_graph() {
    let mut s = Spec::quench_like(200, vec![8, 8, 8, 8, 1, 1, 1, 1, 1, 1, 1, 1]);
    s.seed = 0x5341_5501;
    let n = s.n_atoms;
    // A different, ASYMMETRIC partition every frame, so a permuted index cannot agree by
    // accident: {0,4,5} and {1,6,7,8} leave 2,3,9,10,11 free, and no relabelling of the
    // pair enumeration maps that partition to itself.
    let mem = synthetic::build(s, move |t, pos, vel| {
        for i in 0..n {
            pos[i] = [i as f64 * 1.7, (t % 3) as f64, 0.0];
            vel[i] = [0.0; 3];
        }
        synthetic::bonds_from_blocks(n, &[Mask::from_bits(0b0000_0011_0001), Mask::from_bits(0b0001_1100_0010)])
    });
    let disk = write_and_read(&mem, "graph");
    for (a, b) in mem.frames.iter().zip(disk.frames.iter()) {
        let la = holon_lens::partition::labels_from_bonds(n, &a.bonds);
        let lb = holon_lens::partition::labels_from_bonds(n, &b.bonds);
        assert_eq!(la, lb);
        assert_eq!(
            holon_lens::partition::blocks(&la),
            [0b0000_0011_0001u128, 0b0001_1100_0010, 4, 8, 512, 1024, 2048].map(Mask::from_bits).to_vec()
        );
    }
}
