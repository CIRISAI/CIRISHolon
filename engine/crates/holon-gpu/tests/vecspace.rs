//! The device space against the host space: every row program and the reduction law the same
//! bits, and the device-resident Davidson landing on the host's energies to the bit — on the
//! colour lanes and on a chemistry space.

use holon_chem::budget::DAVIDSON_SUBSPACE_MAX;
use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::{ci_ints, davidson_budget, Order, SolveExit};
use holon_chem::lanes::{solve_lanes_with, LaneSigma, LaneTables};
use holon_chem::pair::geometry_problem;
use holon_chem::qcd2::Qcd2;
use holon_chem::sigma_op::DeviceClass;
use holon_chem::vecspace::{HostSpace, VectorSpace, DOT_BLOCK};
use holon_gpu::{solve_lanes_on_device, DeviceSpace, GpuSigmaProvider};

fn lcg(n: usize, mut st: u64) -> Vec<f64> {
    (0..n)
        .map(|_| {
            st = st.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((st >> 11) as f64) / ((1u64 << 53) as f64) - 0.5
        })
        .collect()
}

fn mism(a: &[f64], b: &[f64]) -> usize {
    a.iter().zip(b).filter(|(x, y)| x.to_bits() != y.to_bits()).count()
}

#[test]
fn the_row_programs_are_the_same_bits_on_the_device() {
    let provider = GpuSigmaProvider::new(0).expect("no CUDA device");
    let dev = DeviceSpace::new(provider.context()).expect("device space");
    let host = HostSpace::with_threads(8);
    for n in [1usize, 255, 257, 5 * DOT_BLOCK * 64 + 3] {
        let m = 5;
        let basis: Vec<Vec<f64>> = (0..m).map(|j| lcg(n, 10 + j as u64)).collect();
        let hbasis: Vec<Vec<f64>> = (0..m).map(|j| lcg(n, 20 + j as u64)).collect();
        let diag = lcg(n, 30);
        let y = vec![0.3, -0.2, 0.0, 0.7, 0.1];
        let theta = -1.25;
        let d_basis: Vec<_> = basis.iter().map(|b| dev.upload(b)).collect();
        let d_hbasis: Vec<_> = hbasis.iter().map(|b| dev.upload(b)).collect();
        let d_diag = dev.upload(&diag);

        // dot
        let hd = VectorSpace::<f64>::dot(&host, &basis[0], &basis[1]);
        let dd = dev.dot(&d_basis[0], &d_basis[1]);
        assert_eq!(hd.to_bits(), dd.to_bits(), "n={n}: dot {hd:.17e} vs {dd:.17e}");

        // ritz
        let (mut hx, mut hr, mut hc) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
        let (hrr, hbc) = host.ritz(&basis, &hbasis, &y, theta, &diag, &mut hx, &mut hr, &mut hc);
        let (mut dx, mut dr, mut dc) = (dev.zeros(n), dev.zeros(n), dev.zeros(n));
        let (drr, dbc) = dev.ritz(&d_basis, &d_hbasis, &y, theta, &d_diag, &mut dx, &mut dr, &mut dc);
        assert_eq!(hrr.to_bits(), drr.to_bits(), "n={n}: ‖r‖²");
        assert_eq!(mism(&hbc, &dbc), 0, "n={n}: Bᵀcorr");
        assert_eq!(mism(&hx, &dev.download(&dx)), 0, "n={n}: x");
        assert_eq!(mism(&hr, &dev.download(&dr)), 0, "n={n}: r");
        assert_eq!(mism(&hc, &dev.download(&dc)), 0, "n={n}: corr");

        // deflate, deflate_norm, gram_row
        let mut hw = hc.clone();
        let hbw = host.deflate(&basis, &hbc, &mut hw);
        let mut dw = dev.copy(&dc);
        let dbw = dev.deflate(&d_basis, &hbc, &mut dw);
        assert_eq!(mism(&hbw, &dbw), 0, "n={n}: deflate reductions");
        assert_eq!(mism(&hw, &dev.download(&dw)), 0, "n={n}: deflate vector");
        let hnn = host.deflate_norm(&basis, &hbw, &mut hw);
        let dnn = dev.deflate_norm(&d_basis, &hbw, &mut dw);
        assert_eq!(hnn.to_bits(), dnn.to_bits(), "n={n}: ‖w‖²");
        assert_eq!(mism(&hw, &dev.download(&dw)), 0, "n={n}: deflate_norm vector");
        let hg = host.gram_row(&basis, &hc);
        let dg = dev.gram_row(&d_basis, &dc);
        assert_eq!(mism(&hg, &dg), 0, "n={n}: gram row");

        // scale, axpy
        let mut hs = hc.clone();
        host.scale(0.5, &mut hs);
        let mut ds = dev.copy(&dc);
        dev.scale(0.5, &mut ds);
        assert_eq!(mism(&hs, &dev.download(&ds)), 0, "n={n}: scale");
        host.axpy(-0.25, &hr, &mut hs);
        dev.axpy(-0.25, &dr, &mut ds);
        assert_eq!(mism(&hs, &dev.download(&ds)), 0, "n={n}: axpy");
    }
}

#[test]
fn the_device_resident_davidson_lands_on_the_host_energies_to_the_bit() {
    let provider = GpuSigmaProvider::new(0).expect("no CUDA device");
    let referee = [(4usize, 0i32, -24.5391166860f64), (4, 1, -17.5847761876), (6, 0, -38.2128396646)];
    for (n, b, e_ref) in referee {
        let q = Qcd2::new(n, 4.0);
        let host = q.ground_lanes(b, 4);
        let tables = LaneTables::build(&q.lane_space(b), &q.lane_hamiltonian());
        let dev = solve_lanes_on_device(provider.context(), &tables, 512, None, davidson_budget(), DAVIDSON_SUBSPACE_MAX).expect("device solve");
        assert_eq!(dev.device, DeviceClass::Gpu);
        assert_eq!(dev.exit, SolveExit::Converged, "N={n} B={b}: {:?}", dev.exit);
        assert!((dev.energy - e_ref).abs() <= 1e-8, "N={n} B={b}: device {:.10} vs referee {e_ref:.10}", dev.energy);
        assert_eq!(host.energy.to_bits(), dev.energy.to_bits(), "N={n} B={b}: host {:.17} vs device {:.17}", host.energy, dev.energy);
        assert_eq!(host.iters, dev.iters, "N={n} B={b}: iteration counts differ");
        assert_eq!(mism(&host.vector, &dev.vector), 0, "N={n} B={b}: vectors differ");
    }
}

#[test]
fn a_chemistry_space_solves_on_the_device_to_the_host_bits() {
    let provider = GpuSigmaProvider::new(0).expect("no CUDA device");
    let o = by_symbol("O").unwrap();
    let h = by_symbol("H").unwrap();
    let at = |x: f64, y: f64| [D2::c(x), D2::c(y), D2::c(0.0)];
    let (space, mo, _) = geometry_problem(&[o, h, h], vec![at(0.0, 0.0), at(1.81, 0.0), at(-0.46, 1.75)]);
    let ci = ci_ints(&mo, Order::Value);
    let mut host_op = LaneSigma::for_ci(&space, &ci);
    let diag = host_op.diagonal();
    let host = solve_lanes_with(&mut host_op, &diag, None, davidson_budget(), DAVIDSON_SUBSPACE_MAX);
    let tables = LaneTables::for_ci(&space, &ci);
    let dev = solve_lanes_on_device(provider.context(), &tables, 512, None, davidson_budget(), DAVIDSON_SUBSPACE_MAX).expect("device solve");
    assert_eq!(host.energy.to_bits(), dev.energy.to_bits(), "water: host {:.17} vs device {:.17}", host.energy, dev.energy);
    assert_eq!(host.exit, dev.exit);
}
