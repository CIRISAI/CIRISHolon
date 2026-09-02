//! The device lane sigma against the host body: the SAME BITS, by construction and by
//! measurement — one host shard, many host shards, and the kernel; run-to-run on the device;
//! the diagonal; and the Davidson through the device operator landing on the frozen referees.

use holon_chem::budget::DAVIDSON_SUBSPACE_MAX;
use holon_chem::fci::{davidson_budget, SolveExit};
use holon_chem::lanes::{solve_lanes_with, LaneSigma, LaneTables};
use holon_chem::qcd2::Qcd2;
use holon_chem::sigma_op::{bit_identity_over_runs, DeviceClass, SigmaOp};
use holon_gpu::{GpuLaneSigma, GpuSigmaProvider};

const REFEREE: [(i32, f64); 2] = [(0, -24.5391166860), (1, -17.5847761876)];

fn lcg_vector(n: usize, mut st: u64) -> Vec<f64> {
    (0..n)
        .map(|_| {
            st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((st >> 11) as f64) / ((1u64 << 53) as f64) - 0.5
        })
        .collect()
}

fn bit_mismatches(a: &[f64], b: &[f64]) -> (usize, f64) {
    let n = a.iter().zip(b).filter(|(x, y)| x.to_bits() != y.to_bits()).count();
    let d = a.iter().zip(b).map(|(x, y)| (x - y).abs()).fold(0.0, f64::max);
    (n, d)
}

#[test]
fn the_device_sigma_is_bit_identical_to_the_host_shards() {
    let provider = GpuSigmaProvider::new(0).expect("no CUDA device");
    for (n, x, b) in [(4usize, 4.0f64, 0i32), (6, 9.0, 0), (6, 4.0, 1)] {
        let q = Qcd2::new(n, x);
        let space = q.lane_space(b);
        let tables = LaneTables::build(&space, &q.lane_hamiltonian());
        let c = lcg_vector(space.n_det, 5 + n as u64);
        let mut host = vec![0.0; space.n_det];
        LaneSigma::new(&space, &q.lane_hamiltonian(), 8).apply(&c, &mut host);
        let mut op = GpuLaneSigma::new(provider.context(), &tables, 512).expect("device operator");
        assert_eq!(op.device(), DeviceClass::Gpu);
        let mut dev = vec![0.0; space.n_det];
        op.apply(&c, &mut dev);
        let (mism, d) = bit_mismatches(&host, &dev);
        assert_eq!(mism, 0, "N={n} x={x} B={b}: device differs from host on {mism}/{} entries, max |Δ| {d:e}", space.n_det);
        // the diagonal, same walk
        let host_diag = LaneSigma::new(&space, &q.lane_hamiltonian(), 1).diagonal();
        let dev_diag = op.diagonal().expect("diagonal");
        let (mism, d) = bit_mismatches(&host_diag, &dev_diag);
        assert_eq!(mism, 0, "N={n} B={b}: diagonal differs on {mism} entries, max |Δ| {d:e}");
        // run to run on the device
        bit_identity_over_runs(&mut op, &c, 3).expect("device sigma is not reproducible against itself");
    }
}

#[test]
fn four_lanes_with_two_empty_run_the_device_decode_path_to_the_bit() {
    // The k ≥ 4 register-array path on the device, against the host on the same tables and
    // against the two-lane operator on the same index order (an empty lane has one string
    // and no singles, so the two spaces are the same operator term for term).
    let provider = GpuSigmaProvider::new(0).expect("no CUDA device");
    let q = Qcd2::new(4, 4.0);
    let two = q.lane_space(0);
    let ham2 = q.lane_hamiltonian();
    let four = holon_chem::lanes::LaneSpace::uniform(4, &[2, 2, 2, 0]);
    let mut ham4 = holon_chem::lanes::LaneHamiltonian::<f64>::new(&[4; 4]);
    for l in 0..3 {
        ham4.h[l].copy_from_slice(&ham2.h[l]);
        for m in l..3 {
            let (src, dst) = (ham2.pair(l, m), ham4.pair(l, m));
            ham4.g[dst].copy_from_slice(&ham2.g[src]);
        }
    }
    assert_eq!(four.n_det, two.n_det);
    let c = lcg_vector(two.n_det, 44);
    let mut ref2 = vec![0.0; two.n_det];
    LaneSigma::new(&two, &ham2, 4).apply(&c, &mut ref2);
    let t4 = LaneTables::build(&four, &ham4);
    let mut host4 = vec![0.0; four.n_det];
    LaneSigma { tables: LaneTables::build(&four, &ham4), threads: 4 }.apply(&c, &mut host4);
    let mut op = GpuLaneSigma::new(provider.context(), &t4, 512).expect("device operator");
    let mut dev4 = vec![0.0; four.n_det];
    op.apply(&c, &mut dev4);
    let (m1, _) = bit_mismatches(&ref2, &host4);
    let (m2, d) = bit_mismatches(&host4, &dev4);
    assert_eq!(m1, 0, "four host lanes differ from two on {m1} entries");
    assert_eq!(m2, 0, "device four-lane sigma differs from host on {m2} entries, max |Δ| {d:e}");
    let (m3, _) = bit_mismatches(&LaneSigma { tables: LaneTables::build(&four, &ham4), threads: 1 }.diagonal(), &op.diagonal().expect("diag"));
    assert_eq!(m3, 0, "four-lane diagonal differs on {m3} entries");
}

#[test]
fn the_davidson_through_the_device_lands_on_the_referees() {
    let provider = GpuSigmaProvider::new(0).expect("no CUDA device");
    let q = Qcd2::new(4, 4.0);
    for (b, e_ref) in REFEREE {
        let space = q.lane_space(b);
        let tables = LaneTables::build(&space, &q.lane_hamiltonian());
        let mut op = GpuLaneSigma::new(provider.context(), &tables, 512).expect("device operator");
        let diag = op.diagonal().expect("diagonal");
        let sol = solve_lanes_with(&mut op, &diag, None, davidson_budget(), DAVIDSON_SUBSPACE_MAX);
        assert_eq!(sol.device, DeviceClass::Gpu);
        assert_eq!(sol.exit, SolveExit::Converged, "B={b}: {:?}", sol.exit);
        assert!((sol.energy - e_ref).abs() <= 1e-8, "B={b}: device {:.10} vs referee {e_ref:.10}", sol.energy);
        // and the same bits as the host solve, since every sigma is the same bits
        let host = q.ground_lanes(b, 4);
        assert_eq!(host.energy.to_bits(), sol.energy.to_bits(), "B={b}: host {:.15} vs device {:.15}", host.energy, sol.energy);
    }
}
