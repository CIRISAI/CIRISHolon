//! Write a synthetic trajectory THROUGH THE REAL FORMAT, so the census runner is exercised
//! end to end — file writer, file reader, census, lenses, classifier — before it is pointed
//! at a real run.
//!
//! The plants test the census against in-memory fixtures. This tests the path the real data
//! takes, which is a different path: a bug in the writer or the reader would leave every
//! plant green and every real verdict wrong.
//!
//! ```text
//! cargo run --release -p holon-lens --example fixture -- <out-dir> [held|breaks|shuffle|frozen]
//! ```

use holon_lens::partition::Mask;
use holon_lens::synthetic::{self, Spec};
use holon_lens::traj::TrajWriter;
use std::path::PathBuf;

const OH2: Mask = 0b0000_0011_0001;

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = PathBuf::from(args.next().expect("out dir"));
    let kind = args.next().unwrap_or_else(|| "held".into());
    std::fs::create_dir_all(&dir).expect("out dir");

    let z = vec![8, 8, 8, 8, 1, 1, 1, 1, 1, 1, 1, 1];
    let mut spec = Spec::quench_like(1200, z);
    spec.seed = 0x5341_5400;
    let n = spec.n_atoms;

    let traj = match kind.as_str() {
        "held" => synthetic::vibrating_block(spec, OH2, 0.4, |_| true),
        "breaks" => synthetic::vibrating_block(spec, OH2, 0.4, |f| f < 600),
        "frozen" => synthetic::frozen_block(spec, OH2),
        "shuffle" => synthetic::build(spec, move |f, pos, vel| {
            let k = (f / 3) % 7;
            for i in 0..n {
                let a = 0.4 * ((0.31 * f as f64) + i as f64).sin();
                pos[i] = [3.0 + 2.0 * (i as f64).cos() + a, 3.0 + 2.0 * (i as f64).sin(), 0.0];
                vel[i] = [a, 0.0, 0.0];
            }
            let b: Mask = (1 << 0) | (1 << (4 + k)) | (1 << (5 + k));
            synthetic::bonds_from_blocks(n, &[b])
        }),
        other => panic!("unknown fixture {other}"),
    };

    let path = dir.join(format!("fixture_{kind}.traj"));
    let mut w = TrajWriter::create(&path, &traj.header).expect("open");
    for f in &traj.frames {
        w.push(f.index, f.time, f.temperature, f.bonded, &f.pos, &f.vel)
            .expect("write");
    }
    let n_written = w.finish().expect("close");
    println!(
        "# wrote {} frames to {} ({:.2} MB)",
        n_written,
        path.display(),
        std::fs::metadata(&path).map(|m| m.len() as f64 / 1e6).unwrap_or(0.0)
    );
}
