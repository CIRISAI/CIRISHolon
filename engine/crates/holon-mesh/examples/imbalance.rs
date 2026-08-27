use holon_mesh::{Grid, Mesh, MeshSpec};
fn main() {
    for (tag, grid, n) in [
        ("flat 128x128", Grid::new(128, 128), (8usize, 8usize, 1usize)),
        ("flat 256x256", Grid::new(256, 256), (8, 8, 1)),
        ("cube 24^3", Grid::new_3d(24, 24, 24), (4, 4, 4)),
        ("cube 32^3", Grid::new_3d(32, 32, 32), (4, 4, 4)),
        ("slab 64x64x8", Grid::new_3d(64, 64, 8), (4, 4, 2)),
        ("thin 64x64x4", Grid::new_3d(64, 64, 4), (4, 4, 4)),
    ] {
        let m = Mesh::new(MeshSpec::new_3d(grid, n.0, n.1, n.2)).expect("built");
        let w: Vec<usize> = m.shards().iter().map(|s| s.work_per_round()).collect();
        let max = *w.iter().max().unwrap() as f64;
        let mean = w.iter().sum::<usize>() as f64 / w.len() as f64;
        println!("{tag:>16} shards={:>3} max/mean={:.3}", w.len(), max / mean);
    }
}
