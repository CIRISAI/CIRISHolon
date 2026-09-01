//! **The dispatch plants: D5, D12, D0 and M-CACHE-KIND** — the last of RESOURCE_DESIGN §8's set.
//!
//! Every plant asserts its carrier before it is scored, and every "must refuse" plant carries the
//! control that would otherwise let "refuse everything" pass.
//!
//! The registered numbers are REAL MEASUREMENTS of the `(O,O,O)` sigma kernel at 207,025
//! determinants, so these are the registry's actual citizens rather than invented ones.
//!
//! **Re-registered 2026-09-01** against the production arm (`holon-gpu/examples/fci_bench.rs`,
//! the in-crate kernel that `holon-chem`'s solve path now uses) rather than the prototype. A
//! registration is a MEMORY of a measurement and calibrations are rented, so it is refreshed
//! when the thing it remembers is replaced. The prototype's own numbers (65.7 sigma/s, 318.4
//! GFLOP/s) are not lost: they are banked in `conformance/BENCHMARKS.md`, where the three
//! independent readings of this kernel are compared side by side.

use holon_resource::probe::ProbeVerdict;
use holon_resource::registry::{
    CheckMode, Determinism, DeviceClass, Dispatch, Entry, Registry, SpotCheck, Step, Workload,
    WorkloadKey,
};
use holon_resource::{Arena, ScriptedProbe};

const SIZE: u64 = 207_025;

fn gpu_entry() -> Entry {
    Entry {
        key: WorkloadKey {
            workload: "fci_sigma",
            size: SIZE,
            device: DeviceClass::Gpu,
        },
        // THE ROUND TRIP, not the kernel — the quantity the CALLER receives.
        //
        // Every application in the production Davidson is `SigmaOp::apply` -> htod + sigma +
        // dtoh + synchronize; `time_kernel_only` is called from the benchmark and nowhere else.
        // An entry holding the device-internal rate makes the entry and its spot-check agree
        // with each other while both overstate what dispatch delivers: internally coherent,
        // externally wrong, and invisible to every plant in this file. That is why this is the
        // round-trip figure even though the kernel figure is prettier.
        //
        // MEASURED IN THE REGIME THE SPOT-CHECK RUNS IN: twelve SEPARATE invocations, separate
        // processes and CUDA contexts, loadavg 78-110
        // (`conformance/atomworld/gpu_fci/spread_both.txt`). Both laws at once — the right
        // quantity, and its spread taken where it will be checked.
        //
        // The kernel-only figure over the same twelve is 67.864 +/- 0.358 (0.53%) and is kept
        // in BENCHMARKS.md as a device-internal number for device-class comparisons. It is not
        // registered, because dispatch does not deliver it.
        mean: 61.619,
        spread: 4.927,
        k: 3.0,
        instrument: "holon-gpu spread_both.txt, (O,O,O) ROUND TRIP, n=12 invocations, loadavg 78-110, 2026-09-01",
        // Atomics-free by construction, cuBLAS pinned to pedantic math with a fixed workspace,
        // five repeat applications bit-identical ON THE OPERATOR THAT RUNS.
        determinism: Determinism::FixedOrder { runs_checked: 5 },
    }
}

fn cpu_entry() -> Entry {
    Entry {
        key: WorkloadKey {
            workload: "fci_sigma",
            size: SIZE,
            device: DeviceClass::Cpu,
        },
        // 17.2-20.8 over two runs at loadavg 18 and 32; the spread is the machine. This is the
        // 32-thread AGGREGATE and it is deliberately NOT refreshed: the machine has been
        // carrying an ozone tabulation on 27 cores since 2026-08-31, and a 32-thread arm
        // measured against that would be measuring the neighbours rather than the kernel.
        // The single-thread pinned arm WAS re-measured (0.95-1.11 sigma/s on a P-core,
        // CPU-time, loadavg 66) and is not registered here because dispatch does not place
        // one thread — it places the mesh.
        mean: 20.8,
        spread: 1.8,
        k: 3.0,
        instrument: "SATURATION-3 G2, s3_sigma_cost.rs (32-thread aggregate, not refreshed)",
        determinism: Determinism::FixedOrder { runs_checked: 2 },
    }
}

fn registry() -> Registry {
    let mut r = Registry::new();
    r.register(gpu_entry());
    r.register(cpu_entry());
    r
}

/// **PLANT D12 — a deliberately mis-registered throughput entry is CONVICTED by a runtime
/// spot-check.**
///
/// Carrier asserted first: the two devices' throughputs must differ by more than the spot-check
/// tolerance, or the plant sits in an empty sector and a conviction would prove nothing.
#[test]
fn plant_d12_a_mis_registered_entry_is_convicted() {
    // THE CARRIER.
    let (gpu, cpu) = (gpu_entry(), cpu_entry());
    let tolerance = gpu.k * gpu.spread;
    assert!(
        (gpu.mean - cpu.mean).abs() > tolerance,
        "the two devices are within the spot-check tolerance, so this plant has an empty sector: \
         a lie could not be distinguished from the honest other device"
    );

    // THE CONTROL: an honest entry, observed at its measured rate, is NOT convicted — including
    // at the low tail this workload really produces. 48.227 is a MEASURED reading from the
    // twelve-invocation series (a descheduled host at loadavg 110), and an entry whose
    // tolerance could not survive its own observed tail would convict the machine on the
    // machine's ordinary behaviour.
    let r = registry();
    let key = gpu.key;
    assert!(matches!(
        r.spot_check(&key, 61.619),
        Some(SpotCheck::Consistent { .. })
    ));
    assert!(
        matches!(r.spot_check(&key, 48.227), Some(SpotCheck::Consistent { .. })),
        "the measured low tail of this workload was convicted; the tolerance does not cover \
         the spread its own registration was built from"
    );

    // THE PLANT: register the GPU at 10x its measured throughput, then observe the real rate.
    let mut lying = Registry::new();
    let mut liar = gpu_entry();
    liar.mean = 616.19;
    lying.register(liar);
    match lying.spot_check(&key, 61.619) {
        Some(SpotCheck::Convicted {
            observed,
            mean,
            tolerance,
        }) => {
            assert_eq!((observed, mean), (61.619, 616.19));
            assert!(tolerance < 554.5);
        }
        other => panic!(
            "a 10x mis-registration survived the spot-check: {other:?}. The registry is trusted \
             about itself, which is the thing D12 exists to prevent."
        ),
    }
}

/// **THE DETECTION FLOOR, demonstrated rather than asserted.**
///
/// Registering the quantity the caller actually receives cost this entry its precision, and a
/// cost that is only described is a cost nobody has checked. The round-trip spread is 8.00%
/// over twelve invocations, so with `k = 3` the tolerance is 14.78 sigma/s and a
/// mis-registration is convicted only if it is **>= 1.24x high or <= 0.76x low**.
///
/// This test pins both sides of that boundary: a 1.2x lie SURVIVES and a 1.3x lie is CAUGHT.
/// The surviving lie is not a defect being tolerated — it is the honest reach of a detector
/// calibrated on a quantity that genuinely varies by 8% in the regime it is checked in.
///
/// **Why this is better than the tighter alternative.** The kernel-only figure over the same
/// twelve invocations is 67.864 +/- 0.358, which with the same `k` would have promised a
/// **1.6%** floor. That precision is not real: it is a promise about a device-internal rate no
/// caller receives, made by a detector that re-times the same unreceived rate. The choice here
/// is between a 24% floor that means what it says and a 1.6% floor that does not.
#[test]
fn the_detection_floor_is_where_the_measured_spread_puts_it() {
    let r = registry();
    let key = gpu_entry().key;
    let mean = gpu_entry().mean;
    let tolerance = gpu_entry().k * gpu_entry().spread;

    // THE CARRIER: the tolerance is the one the registration was built from, not a number
    // chosen to make this test come out.
    assert!((tolerance - 14.781).abs() < 0.01, "tolerance {tolerance} is not 3 x the registered spread");

    // A 1.2x mis-registration SURVIVES. Stated as a fact about reach, not hidden as a gap.
    let mut small = Registry::new();
    let mut e = gpu_entry();
    e.mean = mean * 1.2;
    small.register(e);
    assert!(
        matches!(small.spot_check(&key, mean), Some(SpotCheck::Consistent { .. })),
        "a 1.2x mis-registration was convicted; then the floor is tighter than the measured \
         spread allows, which means the spread is not the one being applied"
    );

    // A 1.3x mis-registration is CAUGHT. Without this half, "it never convicts" would pass.
    let mut big = Registry::new();
    let mut e = gpu_entry();
    e.mean = mean * 1.3;
    big.register(e);
    assert!(
        big.spot_check(&key, mean).unwrap().convicted(),
        "a 1.3x mis-registration survived; the entry is past the floor its own spread implies"
    );
}

/// **PLANT D5 — half-visible hardware REFUSES and does not fall back.**
///
/// Driver present, CUDA broken. The carrier is the part that makes this a real choice: a WORKING
/// CPU entry exists and would have been a perfectly good fallback, so refusing is a decision
/// rather than an absence of alternatives.
#[test]
fn plant_d5_a_broken_device_refuses_rather_than_falling_back() {
    let r = registry();
    let mut arena = Arena::new();

    // THE CARRIER: the CPU path is registered and viable.
    assert!(r
        .get(&WorkloadKey {
            workload: "fci_sigma",
            size: SIZE,
            device: DeviceClass::Cpu
        })
        .is_some());

    // THE PLANT: the workload declares Gpu (bit-gated, so D0 pins it there) and the device probe
    // fails the way a present-driver/broken-CUDA host fails.
    let work = Workload {
        name: "fci_sigma",
        size: SIZE,
        bit_gated: true,
        declared_class: Some(DeviceClass::Gpu),
    };
    let mut broken = ScriptedProbe::always_fail("driver present, CUDA context creation failed");
    let d = r.dispatch(&work, &mut arena, &mut broken);

    match &d {
        Dispatch::Refused { step, why } => {
            assert_eq!(*step, Step::ProbeDevice);
            assert!(why.contains("CUDA context creation failed"));
            assert!(
                why.contains("REFUSED rather than falling back"),
                "the refusal did not say it declined to fall back: {why}"
            );
        }
        Dispatch::Use { device, .. } => panic!(
            "a broken GPU dispatched to {device:?}. If that is Cpu it is a SILENT FALLBACK — the \
             run would complete, the number would look fine, and nothing would record that the \
             registered path was never taken."
        ),
    }
    assert_eq!(d.used(), None);
    // Nothing was leased on a failed probe.
    assert_eq!(arena.ledger().opened, 0);
    assert!(arena.balances());

    // THE CONTROL: the same workload on a healthy device dispatches. Without it, "refuse always"
    // would pass the plant above.
    let mut healthy = ScriptedProbe::always_pass();
    let ok = r.dispatch(&work, &mut arena, &mut healthy);
    assert_eq!(
        ok.used(),
        Some(DeviceClass::Gpu),
        "the healthy control did not dispatch: {}",
        ok.message()
    );
}

/// **PLANT D0 — a bit-gated workload is never moved across device classes by dispatch.**
///
/// This is the rule G2's 91%-bitwise measurement forced. The carrier: the GPU is FASTER on the
/// registered numbers, so a throughput-maximising dispatcher would want to move the work, and
/// declining to is a real cost rather than a free choice.
#[test]
fn plant_d0_a_bit_gated_workload_stays_in_its_declared_class() {
    let r = registry();
    let mut arena = Arena::new();
    let mut probe = ScriptedProbe::always_pass();

    // THE CARRIER: the GPU really is the faster registered entry.
    let gpu = r.get(&gpu_entry().key).unwrap().mean;
    let cpu = r.get(&cpu_entry().key).unwrap().mean;
    assert!(gpu > cpu, "the GPU is not faster, so staying on CPU costs nothing here");

    // A bit-gated workload declaring CPU must STAY on CPU even though the GPU wins.
    let pinned = Workload {
        name: "fci_sigma",
        size: SIZE,
        bit_gated: true,
        declared_class: Some(DeviceClass::Cpu),
    };
    assert_eq!(
        r.dispatch(&pinned, &mut arena, &mut probe).used(),
        Some(DeviceClass::Cpu),
        "dispatch moved a bit-gated workload off its declared class to chase throughput; the \
         table's trailing bits are now a function of the schedule"
    );

    // And a bit-gated workload with NO declared class is refused outright: there is no safe
    // choice to make.
    let undeclared = Workload {
        name: "fci_sigma",
        size: SIZE,
        bit_gated: true,
        declared_class: None,
    };
    match r.dispatch(&undeclared, &mut arena, &mut probe) {
        Dispatch::Refused { step, why } => {
            assert_eq!(step, Step::Consult);
            assert!(why.contains("declares no device class"), "{why}");
        }
        other => panic!("an undeclared bit-gated workload was dispatched: {other:?}"),
    }

    // THE CONTROL: work that is NOT bit-gated may be placed freely, and takes the faster device.
    let free = Workload {
        name: "fci_sigma",
        size: SIZE,
        bit_gated: false,
        declared_class: None,
    };
    assert_eq!(
        r.dispatch(&free, &mut arena, &mut probe).used(),
        Some(DeviceClass::Gpu),
        "non-bit-gated work did not take the faster device; D0 has over-reached into work it \
         does not govern"
    );
}

/// **PLANT M-CACHE-KIND — a small-size entry must not admit a large dispatch.**
///
/// Crossovers are per-size facts. A registry that answered with the nearest entry would let a
/// measurement taken at 605 determinants place work at 207,025, where the whole ordering may be
/// reversed.
#[test]
fn plant_cache_kind_a_size_miss_is_not_a_hit() {
    let r = registry();
    let mut arena = Arena::new();
    let mut probe = ScriptedProbe::always_pass();

    // THE CARRIER: the registry does answer at the size it was measured at.
    let at_size = Workload {
        name: "fci_sigma",
        size: SIZE,
        bit_gated: false,
        declared_class: None,
    };
    assert!(r.dispatch(&at_size, &mut arena, &mut probe).used().is_some());

    // THE PLANT: the same workload at a DIFFERENT size has no entry, and must be refused rather
    // than served from the neighbouring one.
    let other_size = Workload {
        name: "fci_sigma",
        size: 605,
        bit_gated: false,
        declared_class: None,
    };
    match r.dispatch(&other_size, &mut arena, &mut probe) {
        Dispatch::Refused { step, why } => {
            assert_eq!(step, Step::Consult);
            assert!(why.contains("AT SIZE 605"), "{why}");
            assert!(why.contains("per-size facts"), "{why}");
        }
        Dispatch::Use { cited_mean, .. } => panic!(
            "a dispatch at size 605 was served by an entry measured at {SIZE}, citing \
             {cited_mean} /s. Existence stood in for certification."
        ),
    }

    // Direct lookup agrees: an exact key only.
    assert!(r
        .get(&WorkloadKey {
            workload: "fci_sigma",
            size: 605,
            device: DeviceClass::Gpu
        })
        .is_none());
}

/// A bit-gated workload may not be placed on a kernel whose determinism gate is merely a bounded
/// agreement — the distinction G2 turns on.
#[test]
fn a_bounded_agreement_does_not_admit_bit_gated_work() {
    let mut r = Registry::new();
    let mut weak = gpu_entry();
    weak.determinism = Determinism::BoundedAgreement {
        relative: 3.033e-15,
        bitwise_differing_fraction: 0.910,
    };
    r.register(weak);
    let mut arena = Arena::new();
    let mut probe = ScriptedProbe::always_pass();

    let work = Workload {
        name: "fci_sigma",
        size: SIZE,
        bit_gated: true,
        declared_class: Some(DeviceClass::Gpu),
    };
    match r.dispatch(&work, &mut arena, &mut probe) {
        Dispatch::Refused { step, why } => {
            assert_eq!(step, Step::Consult);
            assert!(why.contains("not a fixed reduction order"), "{why}");
        }
        other => panic!("bit-gated work was admitted on a bounded-agreement gate: {other:?}"),
    }

    // The control: the same entry serves work that is not bit-gated.
    let free = Workload {
        bit_gated: false,
        ..work
    };
    assert_eq!(
        r.dispatch(&free, &mut arena, &mut probe).used(),
        Some(DeviceClass::Gpu)
    );
}

/// Each of the three steps can refuse, and the refusal names WHICH — so a wrong placement is
/// traceable to the step that made it rather than to a mood.
#[test]
fn every_step_of_the_dispatch_can_refuse_and_says_which() {
    let r = registry();
    let work = Workload {
        name: "fci_sigma",
        size: SIZE,
        bit_gated: false,
        declared_class: Some(DeviceClass::Gpu),
    };

    // Consult refuses: no entry at this size.
    let mut arena = Arena::new();
    let mut p = ScriptedProbe::always_pass();
    let missing = Workload { size: 1, ..work };
    assert!(matches!(
        r.dispatch(&missing, &mut arena, &mut p),
        Dispatch::Refused { step: Step::Consult, .. }
    ));

    // ProbeDevice refuses.
    let mut p2 = ScriptedProbe::always_fail("no device");
    assert!(matches!(
        r.dispatch(&work, &mut arena, &mut p2),
        Dispatch::Refused { step: Step::ProbeDevice, .. }
    ));

    // Lease refuses: the device probe passes, then the lease probe says no.
    let mut p3 = ScriptedProbe {
        answers: vec![ProbeVerdict::Pass("device present")],
        calls: 0,
        default: ProbeVerdict::Fail("out of VRAM by the time we asked"),
    };
    match r.dispatch(&work, &mut arena, &mut p3) {
        Dispatch::Refused { step, why } => {
            assert_eq!(step, Step::Lease);
            assert!(why.contains("out of VRAM"), "{why}");
        }
        other => panic!("the lease step could not refuse: {other:?}"),
    }
    // And nothing was leased.
    assert!(arena.balances());
    assert_eq!(arena.live_count(), 0);
}

// ---------------------------------------------------------------------------------------
// D12b — THE RE-READ RUNG. Convict only what REPRODUCES.
//
// FOUNDING CASE, measured 2026-09-01 and not invented for the test: a neighbour's honestly
// registered GPU entry was convicted at its own rate on a box at loadavg 72 — observed 58.9
// against a registered mean with a 3-sigma tolerance. Nothing was wrong with the entry. The
// same quantity across twelve separate invocations read 68.25 +/- 0.37.
//
// The check could not distinguish "the registration is wrong" from "the host was
// descheduled", and a check that cannot distinguish two causes is a detector wearing a
// verdict's clothes. One re-read IS the discriminator: a descheduled reading does not
// reproduce, a bad registration does.
// ---------------------------------------------------------------------------------------

// ---------------------------------------------------------------------------------------
// RE-CARRIED 2026-09-01 by the gpu-production lane, and the reason is a RESULT rather than a
// maintenance edit.
//
// These plants were staked on the founding case: observed 58.9 against a registered
// 69.4 +/- 3.0, a conviction of an honest entry. That entry has since been corrected on BOTH
// registration-side laws — it now holds the ROUND-TRIP rate (the quantity a caller actually
// receives) with the spread measured across twelve separate invocations at loadavg 78-110:
// 61.619 +/- 4.927, tolerance 14.78.
//
// Under the corrected entry, 58.9 IS CONSISTENT. It is 2.7 sigma/s from the mean against a
// tolerance of 14.8, so the founding false conviction would never have happened — the
// registration fix alone prevents it, with no re-read needed.
//
// THAT DOES NOT MAKE THE RUNG REDUNDANT and the distinction matters. The registration fix
// widens the band to cover the machine's ORDINARY variation; the re-read rung is what
// separates a genuine outlier BEYOND that band from a wrong entry. The two answer different
// questions and the second is still unanswerable from a single reading. So the plants keep
// their semantics and get carriers that are outliers under the CURRENT entry — 40.0 is
// outside the corrected tolerance the way 58.9 was outside the old one.
//
// The founding numbers are kept in the prose above rather than deleted: a plant re-carried
// without saying what stopped firing hides the finding that made the re-carry necessary.
// ---------------------------------------------------------------------------------------

/// A descheduled outlier is NOT a conviction: the re-read does not reproduce it.
#[test]
fn d12b_live_does_not_convict_an_outlier_that_does_not_reproduce() {
    let mut reg = Registry::default();
    reg.register(gpu_entry());
    let key = gpu_entry().key;

    // THE CARRIER: 40.0 must actually be outside the corrected tolerance, or this plant is
    // scoring a reading that was never going to convict.
    let e = gpu_entry();
    assert!(
        (40.0f64 - e.mean).abs() > e.k * e.spread,
        "40.0 is inside the corrected tolerance; this plant has no outlier to decline"
    );

    let mut calls = 0;
    // 40.0 is an outlier under the corrected entry; 61.6 is its twelve-invocation mean.
    let v = reg
        .spot_check_mode(&key, 40.0, CheckMode::Live, &mut || {
            calls += 1;
            61.6
        })
        .expect("registered");

    assert!(!v.convicted(), "a non-reproducing outlier must not convict: {v:?}");
    assert!(
        matches!(v, SpotCheck::Unreproduced { first, .. } if first == 40.0),
        "the verdict must name the outlier it declined to convict on: {v:?}"
    );
    assert_eq!(calls, 1, "exactly one re-read, never more");
}

/// A genuinely wrong registration still convicts — the rung must not be an amnesty.
#[test]
fn d12b_live_still_convicts_what_reproduces() {
    let mut reg = Registry::default();
    reg.register(gpu_entry());
    let key = gpu_entry().key;

    let mut calls = 0;
    let v = reg
        .spot_check_mode(&key, 30.0, CheckMode::Live, &mut || {
            calls += 1;
            30.2
        })
        .expect("registered");

    assert!(v.convicted(), "a reproducing outlier IS the entry being wrong: {v:?}");
    assert_eq!(calls, 1);
}

/// GAUGING never re-reads. A plant that needs a second opinion is not firing.
#[test]
fn d12b_gauging_is_blunt_and_never_retimes() {
    let mut reg = Registry::default();
    reg.register(gpu_entry());
    let key = gpu_entry().key;

    let mut calls = 0;
    let v = reg
        .spot_check_mode(&key, 40.0, CheckMode::Gauging, &mut || {
            calls += 1;
            61.6
        })
        .expect("registered");

    assert!(v.convicted(), "gauging convicts on the first reading: {v:?}");
    assert_eq!(calls, 0, "GAUGING must never call retime — bluntness is the point");
}

/// The consistent path costs nothing: a hot-path caller pays for no timing it did not need.
#[test]
fn d12b_consistent_reading_never_retimes() {
    let mut reg = Registry::default();
    reg.register(gpu_entry());
    let key = gpu_entry().key;

    let mut calls = 0;
    let v = reg
        .spot_check_mode(&key, 68.5, CheckMode::Live, &mut || {
            calls += 1;
            0.0
        })
        .expect("registered");

    assert!(matches!(v, SpotCheck::Consistent { .. }), "{v:?}");
    assert_eq!(calls, 0, "no re-read on the consistent path");
}
