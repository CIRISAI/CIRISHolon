//! The device arm's conformance gates: **agreement, determinism, and the D0 stamp**.
//!
//! These need a CUDA device. The crate is outside the workspace and `ci-gates.sh` cannot reach
//! it, which is the point — a gate that needs hardware CI does not have must not be able to
//! silently not-run inside a green workspace build.
//!
//! Every reference here is `holon-chem`'s OWN answer on the SAME problem — `sigma_direct` and
//! the host Davidson — never a re-derivation. A device kernel checked against a second
//! implementation of the same algebra tests the algebra; checked against the production path it
//! tests what adoption would actually change.

use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::{
    ci_ints, sigma_direct, solve_determinant_with, CiInts, FciSpace, MoIntegrals, Order,
};
use holon_chem::pair::geometry_problem;
use holon_chem::sigma_op::{
    bit_identity_over_runs, CpuProvider, DeviceClass, SigmaOp, SigmaProvider,
};
use holon_gpu::{GpuSigmaProvider, VramProbe};
use holon_resource::probe::{Probe, ProbeVerdict, ResourceKind};

fn at(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

/// Water at a bent geometry: 441 determinants, dense sigma, real integrals from the same
/// `geometry_problem` call the tables make. Small enough to run in a test, real enough that the
/// gather patterns are the production ones — a synthetic index structure of the right SHAPE
/// would still be a toy, because the memory behaviour comes from the lists' contents.
fn water_problem() -> (FciSpace, MoIntegrals) {
    let o = by_symbol("O").unwrap();
    let h = by_symbol("H").unwrap();
    let (space, mo, _) = geometry_problem(
        &[o, h, h],
        vec![at(0.0, 0.0, 0.0), at(1.81, 0.0, 0.0), at(-0.46, 1.75, 0.0)],
    );
    (space, mo)
}

fn probe_vector(n: usize) -> Vec<f64> {
    let mut c = vec![0.0f64; n];
    let mut seed = 0x243f_6a88_85a3_08d3u64;
    for x in c.iter_mut() {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *x = ((seed >> 33) as f64 / (1u64 << 31) as f64) - 0.5;
    }
    c
}

fn cpu_reference(space: &FciSpace, ci: &CiInts, c: &[f64]) -> Vec<f64> {
    let mut s = vec![0.0f64; space.n_det];
    sigma_direct(space, ci, c, &mut s);
    s
}

/// **The agreement gate, and the measurement D0 rests on.**
///
/// The device answer must reproduce `sigma_direct` to a declared relative bound — and the test
/// also RECORDS how many entries differ bitwise, because that number is the whole content of
/// D0: two correct answers that are not the same artifact.
#[test]
fn the_device_sigma_reproduces_sigma_direct_and_differs_bitwise() {
    let (space, mo) = water_problem();
    let ci = ci_ints(&mo, Order::Value);
    let c = probe_vector(space.n_det);
    let want = cpu_reference(&space, &ci, &c);

    // THE CARRIER, before anything is scored: a reference that was mostly zeros would let a
    // broken kernel agree with it.
    let nz = want.iter().filter(|x| **x != 0.0).count();
    assert!(
        nz * 2 > space.n_det,
        "the CPU reference has {nz} of {} entries nonzero; that is a degenerate carrier",
        space.n_det
    );

    let provider = GpuSigmaProvider::new(0).expect("no CUDA device");
    let mut op = provider.build(&space, &ci).expect("could not build the device operator");
    assert_eq!(op.device(), DeviceClass::Gpu);
    assert_eq!(op.n_det(), space.n_det);

    let mut got = vec![0.0f64; space.n_det];
    op.apply(&c, &mut got);

    let scale = want.iter().fold(0.0f64, |m, x| m.max(x.abs()));
    let max_abs = want
        .iter()
        .zip(got.iter())
        .fold(0.0f64, |m, (a, b)| m.max((a - b).abs()));
    let rel = max_abs / scale;
    let bitdiff = want
        .iter()
        .zip(got.iter())
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count();

    println!(
        "n_det {}  max|d| {:.3e}  relative {:.3e}  bitwise-differing {}/{} ({:.1}%)",
        space.n_det,
        max_abs,
        rel,
        bitdiff,
        space.n_det,
        100.0 * bitdiff as f64 / space.n_det as f64
    );
    assert!(
        rel < 1e-12,
        "the device sigma does not reproduce sigma_direct: relative {rel:.3e}. No timing is \
         worth reporting for a fast wrong answer."
    );
    // NOT an assertion that they DO differ — on a small space they might not, and a test that
    // required disagreement would be requiring a defect. The number is printed because it is
    // the quantity D0 is about, and the (O,O,O) benchmark is where it was measured at 91%.
}

/// **The determinism gate: five applications, BITWISE identical.**
///
/// This is the adoption condition, not a nicety. An artifact that is not reproducible against
/// itself has no class to declare.
#[test]
fn the_device_sigma_is_bit_identical_over_five_runs() {
    let (space, mo) = water_problem();
    let ci = ci_ints(&mo, Order::Value);
    let c = probe_vector(space.n_det);
    let provider = GpuSigmaProvider::new(0).expect("no CUDA device");
    let mut op = provider.build(&space, &ci).expect("could not build the device operator");
    bit_identity_over_runs(&mut op, &c, 5)
        .expect("the device sigma is not run-to-run bit-identical");
}

/// **The whole solve on the device class, and the artifact stamped with it.**
///
/// Both providers drive the SAME host Davidson over the same space. The energies must agree to
/// chemical irrelevance and the two `Solution`s must be stamped with different classes — which
/// is exactly the situation D0 exists to keep straight: the same physics, two artifacts.
#[test]
fn the_full_determinant_solve_runs_on_the_device_and_declares_it() {
    let (space, mo) = water_problem();

    let host = solve_determinant_with(&space, &mo, None, &CpuProvider).expect("host solve");
    let provider = GpuSigmaProvider::new(0).expect("no CUDA device");
    let dev = solve_determinant_with(&space, &mo, None, &provider).expect("device solve");

    println!(
        "host  E = {:.15} Ha, {} Davidson iters, residual {:.3e}, class {}\n\
         dev   E = {:.15} Ha, {} Davidson iters, residual {:.3e}, class {}\n\
         dE    = {:.3e} Ha",
        host.e.v,
        host.davidson_iters,
        host.residual,
        host.device,
        dev.e.v,
        dev.davidson_iters,
        dev.residual,
        dev.device,
        (host.e.v - dev.e.v).abs()
    );

    assert_eq!(host.device, DeviceClass::Cpu);
    assert_eq!(dev.device, DeviceClass::Gpu);
    assert!(
        (host.e.v - dev.e.v).abs() < 1e-9,
        "the two device classes disagree by {:.3e} Ha, which is far past the arithmetic \
         difference between them; something other than the last bits differs",
        (host.e.v - dev.e.v).abs()
    );
    // The derivatives too: the whole solve ran on one class, not just the eigensolve.
    assert!((host.e.d - dev.e.d).abs() < 1e-6, "first derivatives disagree");
}

/// **PLANT D0 — a provider that hands back an operator of another class is REFUSED.**
///
/// The failure this guards against is silent: a `Solution` stamped `Gpu` whose curvature a CPU
/// computed. Nothing downstream could tell, and a bit-gated table built from such solves would
/// be a mixture wearing one label.
///
/// The carrier is asserted first: the honest provider must SUCCEED on the same problem, or
/// "everything is refused" would pass this.
#[test]
fn plant_d0_a_mixed_class_provider_is_refused() {
    struct Liar;
    impl SigmaProvider for Liar {
        fn device(&self) -> DeviceClass {
            // claims the device
            DeviceClass::Gpu
        }
        fn op_for<'a>(
            &self,
            space: &'a FciSpace,
            ci: &'a CiInts,
        ) -> Result<Box<dyn SigmaOp<f64> + 'a>, String> {
            // ...and hands back the host
            CpuProvider.op_for(space, ci)
        }
    }
    let (space, mo) = water_problem();

    // THE CARRIER: an honest provider gets through on this exact problem.
    assert!(solve_determinant_with(&space, &mo, None, &CpuProvider).is_ok());

    // `Solution` is not `Debug` (it carries a whole CI vector), so the error is unwrapped by
    // hand rather than through `expect_err`.
    let err = match solve_determinant_with(&space, &mo, None, &Liar) {
        Err(e) => e,
        Ok(s) => panic!(
            "a provider that mis-declared its class produced a Solution stamped {} — the \
             mixed-class artifact D0 exists to prevent",
            s.device
        ),
    };
    assert!(err.contains("MIXED artifact"), "{err}");
}

/// **PLANT D2 — the reporting probe and the attempting probe disagree exactly where it
/// matters.**
///
/// The carrier is asserted first: both probes must PASS on a request the card can honour, or a
/// probe that refuses everything would satisfy the second half. Then both are asked for more
/// VRAM than the card physically has: the reporting probe is right here too (it is right
/// whenever the number is), so the plant is the ATTEMPT — the reading that stays authoritative
/// when the number stops being.
#[test]
fn plant_d2_the_vram_probe_attempts_rather_than_reports() {
    let mut attempt = VramProbe::new(0).expect("no CUDA device");
    let (free, total) = attempt.mem_info_mib().expect("could not read device memory");
    println!("device reports {free} MiB free of {total} MiB");

    // THE CARRIER: a modest request really is available and really is granted.
    assert!(free > 64, "this card has {free} MiB free; the plant has no room to run");
    assert!(
        attempt.probe(ResourceKind::Vram, 64).passed(),
        "the attempting probe refused 64 MiB on a card reporting {free} MiB free"
    );
    assert_eq!(attempt.allocations, 1, "the probe passed WITHOUT allocating");

    // A request past the card's physical size: refused, and refused by having tried.
    let impossible = total * 4;
    let v = attempt.probe(ResourceKind::Vram, impossible);
    assert!(!v.passed(), "the probe granted {impossible} MiB on a {total} MiB card");

    // And it refuses what it did not test rather than passing on it.
    match attempt.probe(ResourceKind::Disk, 1) {
        ProbeVerdict::Fail(why) => assert!(why.contains("VRAM"), "{why}"),
        ProbeVerdict::Pass(w) => panic!("the VRAM probe passed a DISK question: {w}"),
    }
}
