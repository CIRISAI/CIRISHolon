//! The device arm on a chemistry space (water, two spin lanes): the SAME BITS as the host
//! kernel, run-to-run identical, the full determinant solve landing on the device and
//! declaring it, and the two plants that guard the class stamp (D0) and the probe (D2).

use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::{ci_ints, sigma_direct, solve_determinant_with, CiInts, FciSpace, MoIntegrals, Order};
use holon_chem::pair::geometry_problem;
use holon_chem::sigma_op::{bit_identity_over_runs, CpuProvider, DeviceClass, SigmaOp, SigmaProvider};
use holon_gpu::{GpuSigmaProvider, VramProbe};
use holon_resource::probe::{Probe, ProbeVerdict, ResourceKind};

fn at(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

fn water_problem() -> (FciSpace, MoIntegrals) {
    let o = by_symbol("O").unwrap();
    let h = by_symbol("H").unwrap();
    let (space, mo, _) = geometry_problem(&[o, h, h], vec![at(0.0, 0.0, 0.0), at(1.81, 0.0, 0.0), at(-0.46, 1.75, 0.0)]);
    (space, mo)
}

fn probe_vector(n: usize) -> Vec<f64> {
    let mut c = vec![0.0f64; n];
    let mut seed = 0x243f_6a88_85a3_08d3u64;
    for x in c.iter_mut() {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *x = ((seed >> 33) as f64 / (1u64 << 31) as f64) - 0.5;
    }
    c
}

fn cpu_reference(space: &FciSpace, ci: &CiInts, c: &[f64]) -> Vec<f64> {
    let mut s = vec![0.0f64; space.n_det];
    sigma_direct(space, ci, c, &mut s);
    s
}

#[test]
fn the_device_sigma_is_the_host_sigma_to_the_bit() {
    let (space, mo) = water_problem();
    let ci = ci_ints(&mo, Order::Value);
    let c = probe_vector(space.n_det);
    let want = cpu_reference(&space, &ci, &c);

    let nz = want.iter().filter(|x| **x != 0.0).count();
    assert!(nz * 2 > space.n_det, "the CPU reference has {nz} of {} entries nonzero; that is a degenerate carrier", space.n_det);

    let provider = GpuSigmaProvider::new(0).expect("no CUDA device");
    let mut op = provider.build(&space, &ci).expect("could not build the device operator");
    assert_eq!(op.device(), DeviceClass::Gpu);
    assert_eq!(op.n_det(), space.n_det);

    let mut got = vec![0.0f64; space.n_det];
    op.apply(&c, &mut got);

    let max_abs = want.iter().zip(got.iter()).fold(0.0f64, |m, (a, b)| m.max((a - b).abs()));
    let bitdiff = want.iter().zip(got.iter()).filter(|(a, b)| a.to_bits() != b.to_bits()).count();
    println!("n_det {}  max|d| {:.3e}  bitwise-differing {}/{}", space.n_det, max_abs, bitdiff, space.n_det);
    assert_eq!(
        bitdiff, 0,
        "the device sigma differs from the host kernel on {bitdiff} of {} entries (max |d| {max_abs:.3e}); \
         the two walk the same tables in the same order without FMA, so any difference is a defect",
        space.n_det
    );
}

#[test]
fn the_device_sigma_is_bit_identical_over_five_runs() {
    let (space, mo) = water_problem();
    let ci = ci_ints(&mo, Order::Value);
    let c = probe_vector(space.n_det);
    let provider = GpuSigmaProvider::new(0).expect("no CUDA device");
    let mut op = provider.build(&space, &ci).expect("could not build the device operator");
    bit_identity_over_runs(&mut op, &c, 5).expect("the device sigma is not run-to-run bit-identical");
}

#[test]
fn the_full_determinant_solve_runs_on_the_device_and_declares_it() {
    let (space, mo) = water_problem();

    let host = solve_determinant_with(&space, &mo, None, &CpuProvider).expect("host solve");
    let provider = GpuSigmaProvider::new(0).expect("no CUDA device");
    let dev = solve_determinant_with(&space, &mo, None, &provider).expect("device solve");

    println!(
        "host  E = {:.15} Ha, {} Davidson iters, residual {:.3e}, class {}\n\
         dev   E = {:.15} Ha, {} Davidson iters, residual {:.3e}, class {}",
        host.e.v, host.davidson_iters, host.residual, host.device, dev.e.v, dev.davidson_iters, dev.residual, dev.device
    );

    assert_eq!(host.device, DeviceClass::Cpu);
    assert_eq!(dev.device, DeviceClass::Gpu);
    // every sigma is the same bits, so every Davidson step is, so the energy and its
    // derivatives are — the class is a fact about where it ran, not about the number
    assert_eq!(host.e.v.to_bits(), dev.e.v.to_bits(), "host {:.17} vs device {:.17}", host.e.v, dev.e.v);
    assert_eq!(host.e.d.to_bits(), dev.e.d.to_bits(), "first derivatives differ");
    assert_eq!(host.davidson_iters, dev.davidson_iters);
}

#[test]
fn plant_d0_a_mixed_class_provider_is_refused() {
    struct Liar;
    impl SigmaProvider for Liar {
        fn device(&self) -> DeviceClass {
            DeviceClass::Gpu
        }
        fn op_for<'a>(&self, space: &'a FciSpace, ci: &'a CiInts) -> Result<Box<dyn SigmaOp<f64> + 'a>, String> {
            CpuProvider.op_for(space, ci)
        }
    }
    let (space, mo) = water_problem();

    assert!(solve_determinant_with(&space, &mo, None, &CpuProvider).is_ok());

    let err = match solve_determinant_with(&space, &mo, None, &Liar) {
        Err(e) => e,
        Ok(s) => panic!(
            "a provider that mis-declared its class produced a Solution stamped {} — the mixed-class artifact D0 exists to prevent",
            s.device
        ),
    };
    assert!(err.contains("MIXED artifact"), "{err}");
}

#[test]
fn plant_d2_the_vram_probe_attempts_rather_than_reports() {
    let mut attempt = VramProbe::new(0).expect("no CUDA device");
    let (free, total) = attempt.mem_info_mib().expect("could not read device memory");
    println!("device reports {free} MiB free of {total} MiB");

    assert!(free > 64, "this card has {free} MiB free; the plant has no room to run");
    assert!(attempt.probe(ResourceKind::Vram, 64).passed(), "the attempting probe refused 64 MiB on a card reporting {free} MiB free");
    assert_eq!(attempt.allocations, 1, "the probe passed WITHOUT allocating");

    let impossible = total * 4;
    let v = attempt.probe(ResourceKind::Vram, impossible);
    assert!(!v.passed(), "the probe granted {impossible} MiB on a {total} MiB card");

    match attempt.probe(ResourceKind::Disk, 1) {
        ProbeVerdict::Fail(why) => assert!(why.contains("VRAM"), "{why}"),
        ProbeVerdict::Pass(w) => panic!("the VRAM probe passed a DISK question: {w}"),
    }
}
