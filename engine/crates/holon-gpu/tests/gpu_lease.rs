//! The GPU as a leased resource: the boundary is the operator's own footprint on the tables it
//! will upload; a lease past the card is REFUSED; a card that goes away under a valid lease
//! CONVICTS the lease; and the worker bound is derived from free VRAM and that footprint, in
//! both directions.
//!
//! The D9 plant relies on `GpuLaneSigma::new` doing its VRAM arithmetic BEFORE it touches the
//! device — so the yank between lease and build is visible as a footprint check that fails,
//! and the lease ends Convicted rather than the process ending in a driver error.

use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::{ci_ints, CiInts, FciSpace, MoIntegrals, Order};
use holon_chem::lanes::LaneTables;
use holon_chem::pair::geometry_problem;
use holon_chem::sigma_op::SigmaOp;
use holon_gpu::lease::{LeasedGpuError, LeasedGpuProvider, VramCompetitor};
use holon_gpu::GpuLaneSigma;
use holon_resource::probe::ResourceKind;
use holon_resource::{Arena, LeaseState};

const RESERVE_MIB: u64 = 2048;

/// A space whose two vectors alone are past the card: 18 orbitals, 9/9, 2.36e9 determinants.
fn oversized() -> (FciSpace, CiInts) {
    let n = 18;
    let space = FciSpace::new(n, 9, 9);
    let ci = CiInts { n, k: vec![0.0; n * n], g: vec![0.0; n * n * n * n] };
    (space, ci)
}

/// A space the card holds with room to spare whose two vectors are LARGER than the reserve the
/// D9 competitor leaves behind, so the yank can take the lease's footprint away: 16 orbitals,
/// 8/8, 165.6 million determinants, 2.65 GB of vectors against a 2 GiB reserve.
fn mid_sized() -> (FciSpace, CiInts) {
    let n = 16;
    let space = FciSpace::new(n, 8, 8);
    let ci = CiInts { n, k: vec![0.0; n * n], g: vec![0.0; n * n * n * n] };
    (space, ci)
}

fn at(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

fn water_problem() -> (FciSpace, MoIntegrals) {
    let o = by_symbol("O").unwrap();
    let h = by_symbol("H").unwrap();
    let (space, mo, _) = geometry_problem(&[o, h, h], vec![at(0.0, 0.0, 0.0), at(1.81, 0.0, 0.0), at(-0.46, 1.75, 0.0)]);
    (space, mo)
}

#[test]
fn a_vram_lease_states_its_boundary_and_the_books_balance() {
    let (space, mo) = water_problem();
    let ci = ci_ints(&mo, Order::Value);
    let mut arena = Arena::new();
    let provider = LeasedGpuProvider::new(0, &mut arena, None).expect("no CUDA device");

    let tables = LeasedGpuProvider::tables_for(&space, &ci);
    let declared = LeasedGpuProvider::mib_for(&tables);
    assert!(declared > 0, "a lease boundary of zero bounds nothing");

    let mut held = provider.lease_and_build(&space, &ci).unwrap_or_else(|e| panic!("the lease was refused on an idle card: {e}"));
    assert_eq!(held.mib, declared, "the lease was taken for a different amount than declared");

    let c = vec![0.125f64; space.n_det];
    let mut sigma = vec![0.0f64; space.n_det];
    held.apply(&c, &mut sigma);
    assert!(sigma.iter().any(|x| *x != 0.0), "the leased operator produced nothing");

    println!("leased {} MiB for {} determinants; ledger {:?}", held.mib, space.n_det, provider.ledger());
    provider.release(held).expect("release");
    assert!(provider.balances(), "the lease books do not balance after a clean release");
    let l = provider.ledger();
    assert_eq!((l.opened, l.released, l.convicted), (1, 1, 0));
}

#[test]
fn plant_d3b_a_lease_past_the_boundary_is_refused() {
    let (space, mo) = water_problem();
    let ci = ci_ints(&mo, Order::Value);
    let mut arena = Arena::new();
    let provider = LeasedGpuProvider::new(0, &mut arena, None).expect("no CUDA device");

    let held = provider.lease_and_build(&space, &ci).expect("carrier lease refused");
    provider.release(held).expect("release");

    let mut probe = holon_gpu::VramProbe::new(0).expect("no CUDA device");
    let (_free, total) = probe.mem_info_mib().expect("device memory unreadable");
    let over = total * 4;
    let e = arena.lease(&mut probe, None, ResourceKind::Vram, over).expect_err("a lease for four times the card's memory was granted");
    match e {
        holon_resource::LeaseError::Refused { kind, amount, .. } => {
            assert_eq!(kind, ResourceKind::Vram);
            assert_eq!(amount, over);
        }
        other => panic!("expected REFUSED, got {other:?}"),
    }
    assert_eq!(arena.ledger().opened, 1, "a refusal opened a ledger entry; only the carrier's lease should be in the books");
}

#[test]
fn plant_d9_a_yanked_gpu_convicts_the_lease_rather_than_erroring() {
    let (space, ci) = mid_sized();
    let tables = LeasedGpuProvider::tables_for(&space, &ci);

    {
        let mut arena = Arena::new();
        let provider = LeasedGpuProvider::new(0, &mut arena, None).expect("no CUDA device");
        let held = provider
            .lease_and_build(&space, &ci)
            .unwrap_or_else(|e| panic!("PLANT VOID (empty sector): the unyanked control failed, so a later failure would say nothing about the yank. ({e})"));
        provider.release(held).expect("release");
        assert!(provider.balances());
    }

    let mut arena = Arena::new();
    let provider = LeasedGpuProvider::new(0, &mut arena, None).expect("no CUDA device");
    let declared = LeasedGpuProvider::mib_for(&tables);

    let lease = provider.take_lease(&tables).unwrap_or_else(|e| panic!("the lease was refused before any yank: {}", e.message()));
    let lease_id = lease.id;
    assert_eq!(lease.mib, declared);
    assert_eq!(provider.lease_state(lease_id), Some(LeaseState::Leased), "the lease is not live, so there is nothing to yank it out from under");
    println!("leased {declared} MiB (probe passed: it allocated that much and freed it)");

    let Some(competitor) = VramCompetitor::take_all_but(provider.context(), RESERVE_MIB).expect("could not query the device") else {
        panic!(
            "PLANT VOID (empty sector): the card has less than {RESERVE_MIB} MiB free, so there is nothing to yank and this test would score a machine state rather than the lease layer. Re-run on an idle card."
        );
    };

    let mut check = holon_gpu::VramProbe::on(provider.context().clone());
    let (free_after, total) = check.mem_info_mib().expect("device memory unreadable");
    println!("yank: competitor took {} MiB; {free_after} MiB free of {total} MiB against a lease for {declared} MiB", competitor.took_mib);
    assert!(
        free_after < declared,
        "PLANT VOID (empty sector): {free_after} MiB is still enough for a {declared} MiB lease, so the resource did not go away and there is nothing to convict"
    );

    let outcome = provider.build_on(lease, &tables);
    drop(competitor);

    match outcome {
        Err(LeasedGpuError::ConvictedOnBuild { lease, why }) => {
            println!("CONVICTED lease {lease}: {why}");
            assert_eq!(lease, lease_id);
            assert_eq!(provider.lease_state(lease_id), Some(LeaseState::Convicted), "the lease did not end Convicted");
            let l = provider.ledger();
            assert_eq!(l.convicted, 1, "the conviction did not reach the ledger");
            assert_eq!(l.released, 0, "a convicted lease was also counted as released");
            assert!(provider.balances(), "the books do not balance after a conviction");
        }
        Err(LeasedGpuError::Lease(e)) => panic!(
            "PLANT VOID (wrong branch): this came back REFUSED rather than CONVICTED, so the resource never went away under a valid lease — the lease step must run before the yank for this plant to mean anything. ({})",
            e.message()
        ),
        Ok(_) => panic!(
            "the operator was built with only {free_after} MiB free against a {declared} MiB footprint — either the yank did not take, or the footprint check is not checking"
        ),
    }
}

#[test]
fn the_gpu_worker_bound_is_derived_from_vram_and_bounds_in_both_directions() {
    let gp = holon_gpu::GpuSigmaProvider::new(0).expect("no CUDA device");

    let (small, ci) = {
        let (space, mo) = water_problem();
        let ci = ci_ints(&mo, Order::Value);
        (space, ci)
    };
    let small_tables = LaneTables::for_ci(&small, &ci);
    let many = gp.max_workers_for(&small_tables, 1024).expect("could not derive the bound");
    println!("water ({} det, {:.2} MiB/worker): {many} GPU workers fit with a 1 GiB reserve", small.n_det, GpuLaneSigma::bytes_for(&small_tables) as f64 / (1u64 << 20) as f64);
    assert!(many > 1, "a {}-determinant space fits {many} workers on this card; the bound is refusing work the card can plainly do", small.n_det);

    // The bound is the free VRAM over the footprint, read live and printed rather than
    // asserted against a remembered number: the footprint is the operator's own arithmetic
    // and the free figure is the machine's.
    let (free, _tot) = gp.mem_info().expect("device memory unreadable");
    for (label, n_orb, n_elec) in [("0.25M", 12usize, 4usize), ("1M", 14, 4), ("4M", 14, 5), ("9M", 14, 6), ("64M", 16, 7)] {
        let sp = FciSpace::new(n_orb, n_elec, n_elec);
        let cz = CiInts { n: n_orb, k: vec![0.0; n_orb * n_orb], g: vec![0.0; n_orb * n_orb * n_orb * n_orb] };
        let t = LaneTables::for_ci(&sp, &cz);
        let b = GpuLaneSigma::bytes_for(&t);
        let workers = gp.max_workers_for(&t, 1024).expect("bound");
        let fits = b <= free as u64;
        println!(
            "  ladder {label:>5}: {n_orb}orb/{n_elec}e  {:>10} det  {:>7.3} GiB/op  {}  workers {workers}",
            sp.n_det,
            b as f64 / (1u64 << 30) as f64,
            if fits { "FITS" } else { "DOES NOT FIT" }
        );
        assert_eq!(workers > 0, b + (1024 << 20) <= free as u64, "the worker bound and the footprint disagree about whether one operator fits");
    }

    let (huge, hci) = oversized();
    let huge_tables = LaneTables::for_ci(&huge, &hci);
    let none = gp.max_workers_for(&huge_tables, 1024).expect("could not derive the bound");
    println!("18-orbital 9/9 ({} det, {:.1} GB/worker): {none} GPU workers fit", huge.n_det, GpuLaneSigma::bytes_for(&huge_tables) as f64 / 1e9);
    assert_eq!(none, 0, "a space needing more VRAM than the card has reported {none} workers; a bound that cannot say `none` cannot refuse, and D4 forbids the fallback that would follow");

    let (_free, total) = gp.mem_info().expect("device memory unreadable");
    let starved = gp.max_workers_for(&small_tables, (total as u64 / (1 << 20)) * 2).expect("could not derive the bound");
    assert_eq!(starved, 0, "a reserve larger than the card still admitted {starved} workers, so the reserve is not being held back");
}
