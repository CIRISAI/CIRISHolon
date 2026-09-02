//! **GANTT node F's receipts: the device class is part of the table's identity, and mixing is
//! refused rather than detected.**
//!
//! RESOURCE_DESIGN **D0**. The warrant is SATURATION-3 G2: the two classes agree to
//! `3.033e-15` relative and **91.0% of 207,025 entries differ BITWISE**. Both correct, not the
//! same artifact. So a table declares its class, the declaration travels into every manifest
//! beside the other two identity axes, and a node that came back from the wrong class stops
//! the run instead of being averaged into a digest.
//!
//! The three axes are one identity, not three diagnostics:
//!
//! | axis | what it fixes | how it was learned |
//! |---|---|---|
//! | device class | which arithmetic produced the bits | G2's 91% bitwise split |
//! | solver budget | which regime the solves ran under | a silent 1200 → 4000 default change |
//! | subtraction basis | what the stored number is a residual OF | a 4-body residual read as a total |
//!
//! Each was learned the same way — a number moved and nothing recorded which regime made it.

use holon_chem::sigma_op::DeviceClass;
use holon_tables::generate::{generate, GenSpec, WarmPolicy};
use holon_tables::grid::TableGrid;
use holon_tables::{DistanceTetramer, Surface, TrimerSurface};
use holon_chem::elements::by_symbol;

fn tiny_grid() -> TableGrid {
    // Deliberately small: this file is about identity plumbing, not about physics, and a
    // gate that took minutes would not be run.
    TableGrid::new(2, 2, 2, [1, 1, 1], (1.4, 1.8), (1.4, 1.8), (-0.2, 0.2))
}

fn h3() -> [holon_chem::elements::Species; 3] {
    let h = by_symbol("H").unwrap();
    [h, h, h]
}

/// **The default is CPU, and it is a DECLARATION rather than a fallback.**
///
/// Every committed table declares `cpu`; the point of the default is that it is the same class
/// those tables were built in, not that it is what you get when something else fails. There is
/// no silent fallback across classes anywhere in this crate (D4).
#[test]
fn a_spec_declares_its_device_class_and_defaults_to_cpu() {
    let spec = GenSpec::new(h3(), tiny_grid());
    assert_eq!(spec.device, DeviceClass::Cpu);
    assert_eq!(
        spec.with_device(DeviceClass::Gpu).device,
        DeviceClass::Gpu,
        "the declared class must be settable, or it is not an axis of the artifact"
    );
}

/// **Same-table bit-identity WITHIN a class** — node F's first receipt.
///
/// Two runs of the same CPU-class spec at different worker counts must agree BIT for BIT, and
/// the digest must be the same digest. This is G1's property restated under the new axis: the
/// class is part of the identity, and the schedule still is not.
#[test]
fn the_same_class_reproduces_bit_for_bit_across_worker_counts() {
    let spec = GenSpec::new(h3(), tiny_grid()).with_warm(WarmPolicy::CanonicalChain);

    let one = generate(&spec, 1);
    let four = generate(&spec, 4);

    // **THE SECOND CARRIER, and it is de4-table's lesson applied to me.** They found their
    // checkpoint tests non-vacuous on the statistic and VACUOUS ON THE SCENE: 4x4x3 grids
    // where every region finished in milliseconds could not catch a granularity defect that
    // put the first commit 21 wall-hours away. The same trap is here — if this grid produced
    // fewer regions than workers, the 4-worker run would leave workers idle and "agrees across
    // worker counts" would be a statement about a run that never sharded.
    // The question is whether there is enough work for four workers to divide, and
    // `shard_digests.len()` does NOT answer it — that is one digest per worker by
    // construction, present even for a worker that got nothing. Asking it would have been a
    // carrier as vacuous as the thing it guards against, which is what I first wrote. The
    // real question is the REGION COUNT.
    let regions = tiny_grid().n_regions();
    assert!(
        regions >= 4,
        "this grid has {regions} regions and the run claims to exercise 4 workers, so at least \
         one worker got no work and 'agrees across worker counts' is a statement about a run \
         that never sharded"
    );

    // THE CARRIER, asserted before the comparison is scored: an empty or all-VOID table would
    // agree with itself for the most useless of reasons.
    let solved = one
        .records
        .iter()
        .filter(|r| r.energy_bits != 0)
        .count();
    assert!(
        solved > 0,
        "no node in this table carries an energy, so bit-identity here is a statement about \
         nothing"
    );

    assert_eq!(
        one.records.len(),
        four.records.len(),
        "the two runs produced different node counts"
    );
    let differing = one
        .records
        .iter()
        .zip(four.records.iter())
        .filter(|(a, b)| a.energy_bits != b.energy_bits || a.node != b.node)
        .count();
    assert_eq!(
        differing, 0,
        "{differing} nodes differ BITWISE between a 1-worker and a 4-worker run of the SAME \
         device class; the schedule has reached the numbers"
    );
    assert_eq!(
        one.digest().hex(),
        four.digest().hex(),
        "the digests differ across worker counts within one class"
    );
}

/// **The GPU class is REFUSED here, loudly, and the refusal names its exit.**
///
/// `holon-tables` cannot construct a GPU provider — it does not link CUDA, deliberately, since
/// it sits inside the workspace whose isolation gates exist to keep CUDA out. D4 says a path
/// that cannot be taken produces a loud refusal naming what was asked and what to do instead,
/// never a quiet run on the other class. A table stamped `gpu` that a CPU produced would pass
/// every gate in this repository and be wrong.
///
/// The carrier is asserted first: the SAME spec on the CPU class really does generate, so this
/// is a refusal of one class rather than a generator that refuses everything.
#[test]
fn a_gpu_class_spec_is_refused_rather_than_run_on_the_host() {
    // THE CARRIER.
    let cpu = GenSpec::new(h3(), tiny_grid());
    let ok = generate(&cpu, 1);
    assert!(!ok.records.is_empty(), "the CPU-class control produced nothing");

    let gpu = GenSpec::new(h3(), tiny_grid()).with_device(DeviceClass::Gpu);
    let panic = std::panic::catch_unwind(|| generate(&gpu, 1));
    let err = panic.expect_err(
        "a GPU-class spec generated a table inside holon-tables, which cannot construct a GPU \
         provider — so the numbers came from the host under a `gpu` declaration",
    );
    let msg = err
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        msg.contains("holon-gpu"),
        "the refusal must name the exit — where a GPU-class table IS generated. Got: {msg}"
    );
    assert!(
        msg.contains("REFUSED"),
        "the refusal must be loud and say so. Got: {msg}"
    );
}

/// **Every surface names its subtraction basis, and they do not all say the same thing.**
///
/// The method is required rather than defaulted precisely so a surface that DOES subtract
/// cannot inherit a manifest line saying it does not. This checks the axis actually
/// discriminates — if every surface returned the same string the axis would be decoration.
#[test]
fn the_subtraction_basis_discriminates_between_surfaces() {
    let o = by_symbol("O").unwrap();
    let h = by_symbol("H").unwrap();

    let trimer = TrimerSurface::new([o, h, h]);
    let tetramer = DistanceTetramer::new([o, h, h, h]);
    // The four-body basis is read as a CONSTANT rather than by constructing the surface,
    // which samples two pair curves — a gate that expensive is a gate nobody runs.
    let ohhh_basis = holon_tables::ohhh::OHHH_BASIS;

    for (what, basis) in [
        ("trimer", trimer.basis()),
        ("tetramer", tetramer.basis()),
        ("ohhh", ohhh_basis),
    ] {
        assert!(!basis.is_empty(), "{what} declares an empty subtraction basis");
    }

    // The one that subtracts must not read like the ones that do not.
    assert!(
        ohhh_basis.contains("MBE3"),
        "the four-body residual surface does not name what it subtracts: {ohhh_basis}"
    );
    assert_ne!(
        ohhh_basis,
        trimer.basis(),
        "a surface that subtracts and one that stores the total report the SAME basis; the \
         axis is decoration rather than identity"
    );
}
