//! Diagnostics on the comparison harness itself, before any number from it is believed.
use ciris_sim_core::linalg::{jacobi_eigen, laplacian, JACOBI_MAX_SWEEPS};
#[path = "../scene.rs"] mod scene;
use scene::Scene;

fn check<const N: usize>(s: &Scene) {
    let flat = s.coupling();
    let mut c = Box::new([[0.0f64; N]; N]);
    for i in 0..N { for j in 0..N { c[i][j] = flat[i * N + j]; } }
    let l = Box::new(laplacian(&c));
    let t0 = std::time::Instant::now();
    let e = jacobi_eigen(&l);
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    let mut worst = 0.0f64;
    for i in 0..N.min(64) {
        for j in 0..N.min(64) {
            let mut acc = 0.0;
            for m in 0..N { acc += e.values[m] * e.vectors[m][i] * e.vectors[m][j]; }
            worst = worst.max((acc - l[i][j]).abs());
        }
    }
    println!(
        "{:<26} N={:<5} sweeps={:<4} converged={:<6} cap={} recon_resid={:.3e}  {:.1} ms",
        s.name, N, e.sweeps, e.converged, JACOBI_MAX_SWEEPS, worst, ms
    );
}

fn main() {
    std::thread::Builder::new().stack_size(512*1024*1024).spawn(|| {
        check::<11>(&Scene::k11());
        check::<11>(&Scene::complete(11));
        check::<32>(&Scene::complete(32));
        check::<64>(&Scene::complete(64));
        check::<128>(&Scene::complete(128));
        check::<256>(&Scene::complete(256));
        check::<27>(&Scene::lattice(3));
        check::<64>(&Scene::lattice(4));
        check::<125>(&Scene::lattice(5));
        check::<216>(&Scene::lattice(6));
        check::<512>(&Scene::lattice(8));
    }).unwrap().join().unwrap();
}
