//! **The GPU-VRAM lease plants** — RESOURCE_DESIGN D3b and D9, on a real card.
//!
//! Every plant here asserts its CARRIER before it is scored (M-PLANT-SECTOR: a plant on an
//! empty sector VOIDs rather than passes), and the VOID branch is a `panic!` with the word VOID
//! in it rather than an early `return` — a plant that quietly skips itself on a busy machine is
//! a plant that has never fired and looks like one that has.
//!
//! The yank is REAL: a competitor allocation takes the free VRAM out from under a live lease on
//! the actual device. It is also BOUNDED — it leaves a declared reserve and gives the memory
//! back immediately, because the browser's GPU process is on this card and a plant that
//! demonstrated the lease layer by wedging the machine would have proved the wrong thing.

use holon_chem::dual::D2;
use holon_chem::elements::by_symbol;
use holon_chem::fci::{ci_ints, CiInts, FciSpace, MoIntegrals, Order};
use holon_chem::pair::geometry_problem;
use holon_chem::sigma_op::SigmaOp;
use holon_gpu::lease::{LeasedGpuError, LeasedGpuProvider, VramCompetitor};
use holon_resource::probe::ResourceKind;
use holon_resource::{Arena, LeaseState};

/// Left free for everything else on this card while the yank is held. The browser's GPU process
/// lives here; a plant is not a licence to corner a shared device.
const RESERVE_MIB: u64 = 2048;

/// A space big enough that its device footprint exceeds [`RESERVE_MIB`], built WITHOUT an SCF.
///
/// `FciSpace::new` builds the occupation strings and their excitation lists and nothing else, and
/// `GpuFciSigma::new` does its VRAM arithmetic BEFORE it touches an integral — so the yank plant
/// gets a production-sized footprint for the cost of some combinatorics. The integrals are zeros
/// because this plant never computes a sigma; it is about whether the memory is there. A test
/// that scored a NUMBER from these would be scoring zeros, which is why it does not.
///
/// 13 orbitals, 6 electrons per spin: 1716 strings per spin, T alone is 1716 x 169 x 1716 x 8 =
/// 3.98 GB. Comfortably past the reserve, comfortably inside an idle card.
fn oversized_space() -> (FciSpace, CiInts) {
    let n = 13;
    let space = FciSpace::new(n, 6, 6);
    let ci = CiInts {
        n,
        k: vec![0.0; n * n],
        g: vec![0.0; n * n * n * n],
    };
    (space, ci)
}

fn at(x: f64, y: f64, z: f64) -> [D2; 3] {
    [D2::c(x), D2::c(y), D2::c(z)]
}

fn water_problem() -> (FciSpace, MoIntegrals) {
    let o = by_symbol("O").unwrap();
    let h = by_symbol("H").unwrap();
    let (space, mo, _) = geometry_problem(
        &[o, h, h],
        vec![at(0.0, 0.0, 0.0), at(1.81, 0.0, 0.0), at(-0.46, 1.75, 0.0)],
    );
    (space, mo)
}

/// **The lease works, and its boundary is a computed quantity rather than a guess (D3b).**
///
/// The carrier: the operator really is built, really computes, and the books really balance
/// afterwards — without which the refusal plants below would be satisfied by a layer that
/// refuses everything.
#[test]
fn a_vram_lease_states_its_boundary_and_the_books_balance() {
    let (space, mo) = water_problem();
    let ci = ci_ints(&mo, Order::Value);
    let mut arena = Arena::new();
    let provider = LeasedGpuProvider::new(0, &mut arena, None).expect("no CUDA device");

    let declared = LeasedGpuProvider::mib_for(&space);
    assert!(declared > 0, "a lease boundary of zero bounds nothing");

    let mut held = provider
        .lease_and_build(&space, &ci)
        .unwrap_or_else(|e| panic!("the lease was refused on an idle card: {e}"));
    assert_eq!(held.mib, declared, "the lease was taken for a different amount than declared");

    // it really computes
    let c = vec![0.125f64; space.n_det];
    let mut sigma = vec![0.0f64; space.n_det];
    held.apply(&c, &mut sigma);
    assert!(sigma.iter().any(|x| *x != 0.0), "the leased operator produced nothing");

    println!(
        "leased {} MiB for {} determinants; ledger {:?}",
        held.mib,
        space.n_det,
        provider.ledger()
    );
    provider.release(held).expect("release");
    assert!(provider.balances(), "the lease books do not balance after a clean release");
    let l = provider.ledger();
    assert_eq!((l.opened, l.released, l.convicted), (1, 1, 0));
}

/// **PLANT D3b — a lease past the card's quantitative boundary is REFUSED, not attempted.**
///
/// The carrier is asserted first: a lease the card CAN honour is granted on the same provider,
/// so "refuses everything" cannot pass this.
#[test]
fn plant_d3b_a_lease_past_the_boundary_is_refused() {
    let (space, mo) = water_problem();
    let ci = ci_ints(&mo, Order::Value);
    let mut arena = Arena::new();
    let provider = LeasedGpuProvider::new(0, &mut arena, None).expect("no CUDA device");

    // THE CARRIER: an honourable ask succeeds.
    let held = provider.lease_and_build(&space, &ci).expect("carrier lease refused");
    provider.release(held).expect("release");

    // The overflow: ask the probe directly for more than the card has. This is the lease's
    // quantitative boundary being tested, not the operator's — D3b says the boundary is
    // DECLARED and the escalation is leased rather than improvised, so the refusal has to
    // happen at the lease and not inside a kernel that OOMs.
    let mut probe = holon_gpu::VramProbe::new(0).expect("no CUDA device");
    let (_free, total) = probe.mem_info_mib().expect("device memory unreadable");
    let over = total * 4;
    let e = arena
        .lease(&mut probe, None, ResourceKind::Vram, over)
        .expect_err("a lease for four times the card's memory was granted");
    match e {
        holon_resource::LeaseError::Refused { kind, amount, .. } => {
            assert_eq!(kind, ResourceKind::Vram);
            assert_eq!(amount, over);
        }
        other => panic!("expected REFUSED, got {other:?}"),
    }
    // A refusal leaves no entry: it is not a lease that failed, it is a lease that never was.
    // One entry exists, and it is the carrier's.
    assert_eq!(
        arena.ledger().opened,
        1,
        "a refusal opened a ledger entry; only the carrier's lease should be in the books"
    );
}

/// **PLANT D9/D3 — the GPU is YANKED between the lease and the use, and the outcome is
/// CONVICTED, not an error.**
///
/// The ruled shape, and the order is the whole of it:
///
/// 1. the VRAM probe runs on an idle card and **passes** — it really allocates the footprint
///    and gives it back, so the lease is genuinely valid at lease time;
/// 2. a competitor takes the free VRAM out from under it — a real allocation on the real card,
///    bounded to leave [`RESERVE_MIB`];
/// 3. the holder's **USE** — building the operator it leased the memory for — fails.
///
/// D3(2) says that failing use is the authoritative reading, more authoritative than the probe
/// that preceded it. D9 says the outcome must SURFACE in the parent's books rather than vanish
/// into an error return. So the lease ends `Convicted`, the ledger's `convicted` count moves,
/// and the books still balance.
///
/// **This is the gap a lease exists to be honest about**, and it is not hypothetical: it is the
/// disk-full afternoon of 2026-08-30 in the device's vocabulary — a probe that passed and was
/// false milliseconds later.
#[test]
fn plant_d9_a_yanked_gpu_convicts_the_lease_rather_than_erroring() {
    let (space, ci) = oversized_space();

    // ---- CARRIER 1: with nothing yanked, the same lease-and-build SUCCEEDS. Without this,
    // "everything fails on this card" would satisfy the plant.
    {
        let mut arena = Arena::new();
        let provider = LeasedGpuProvider::new(0, &mut arena, None).expect("no CUDA device");
        let held = provider.lease_and_build(&space, &ci).unwrap_or_else(|e| {
            panic!(
                "PLANT VOID (empty sector): the unyanked control failed, so a later failure \
                 would say nothing about the yank. ({e})"
            )
        });
        provider.release(held).expect("release");
        assert!(provider.balances());
    }

    let mut arena = Arena::new();
    let provider = LeasedGpuProvider::new(0, &mut arena, None).expect("no CUDA device");
    let declared = LeasedGpuProvider::mib_for(&space);

    // ---- STEP 1: the probe passes and the lease is granted, on an idle card.
    let lease = provider
        .take_lease(&space)
        .unwrap_or_else(|e| panic!("the lease was refused before any yank: {}", e.message()));
    let lease_id = lease.id;
    assert_eq!(lease.mib, declared);
    assert_eq!(
        provider.lease_state(lease_id),
        Some(LeaseState::Leased),
        "the lease is not live, so there is nothing to yank it out from under"
    );
    println!("leased {declared} MiB (probe passed: it allocated that much and freed it)");

    // ---- STEP 2: THE YANK. Bounded, and given back at the end of the test.
    let Some(competitor) = VramCompetitor::take_all_but(provider.context(), RESERVE_MIB)
        .expect("could not query the device")
    else {
        panic!(
            "PLANT VOID (empty sector): the card has less than {RESERVE_MIB} MiB free, so there \
             is nothing to yank and this test would score a machine state rather than the lease \
             layer. Re-run on an idle card."
        );
    };

    // ---- CARRIER 2: the yank really took the card, and really left less than the lease covers.
    let mut check = holon_gpu::VramProbe::on(provider.context().clone());
    let (free_after, total) = check.mem_info_mib().expect("device memory unreadable");
    println!(
        "yank: competitor took {} MiB; {free_after} MiB free of {total} MiB against a lease for \
         {declared} MiB",
        competitor.took_mib
    );
    assert!(
        free_after < declared,
        "PLANT VOID (empty sector): {free_after} MiB is still enough for a {declared} MiB \
         lease, so the resource did not go away and there is nothing to convict"
    );

    // ---- STEP 3: THE USE.
    let outcome = provider.build_on(lease, &space, &ci);
    drop(competitor);

    match outcome {
        Err(LeasedGpuError::ConvictedOnBuild { lease, why }) => {
            println!("CONVICTED lease {lease}: {why}");
            assert_eq!(lease, lease_id);
            assert_eq!(
                provider.lease_state(lease_id),
                Some(LeaseState::Convicted),
                "the lease did not end Convicted"
            );
            let l = provider.ledger();
            assert_eq!(l.convicted, 1, "the conviction did not reach the ledger");
            assert_eq!(l.released, 0, "a convicted lease was also counted as released");
            assert!(provider.balances(), "the books do not balance after a conviction");
        }
        Err(LeasedGpuError::Lease(e)) => panic!(
            "PLANT VOID (wrong branch): this came back REFUSED rather than CONVICTED, so the \
             resource never went away under a valid lease — the lease step must run before the \
             yank for this plant to mean anything. ({})",
            e.message()
        ),
        Ok(_) => panic!(
            "the operator was built with only {free_after} MiB free against a {declared} MiB \
             footprint — either the yank did not take, or the footprint check is not checking"
        ),
    }
}

/// **The GPU-class worker bound is DERIVED from VRAM, and it really is small.**
///
/// F.2's shape depends on this number, so it is measured on the real card rather than
/// asserted from the design doc. Each GPU worker needs its own device operator — `GpuFciSigma`
/// owns device buffers a second thread cannot share — so the bound is VRAM per operator, not
/// cores.
///
/// Both directions are pinned, because a bound that only ever says "lots" or only ever says
/// "none" is not a bound:
///
/// * a SMALL space fits many workers (the reserve, not the footprint, is what limits it);
/// * a space whose single operator exceeds the card returns **0**, which a caller must read as
///   a refusal of GPU-class generation rather than a licence to fall back to the host.
#[test]
fn the_gpu_worker_bound_is_derived_from_vram_and_bounds_in_both_directions() {
    let gp = holon_gpu::GpuSigmaProvider::new(0).expect("no CUDA device");

    // A small space: many workers fit, and the answer must be positive or the bound is
    // refusing work the card can obviously do.
    let (small, _ci) = {
        let o = holon_chem::elements::by_symbol("O").unwrap();
        let h = holon_chem::elements::by_symbol("H").unwrap();
        let (space, mo, _) = holon_chem::pair::geometry_problem(
            &[o, h, h],
            vec![at(0.0, 0.0, 0.0), at(1.81, 0.0, 0.0), at(-0.46, 1.75, 0.0)],
        );
        let ci = holon_chem::fci::ci_ints(&mo, holon_chem::fci::Order::Value);
        (space, ci)
    };
    let many = gp.max_workers_for(&small, 1024).expect("could not derive the bound");
    println!("water ({} det): {many} GPU workers fit with a 1 GiB reserve", small.n_det);
    assert!(
        many > 1,
        "a 441-determinant space fits {many} workers on a 16 GB card; the bound is refusing \
         work the card can plainly do"
    );

    // THE (O,O,O) CASE, pinned, because a number quoted into two lanes' inboxes was wrong by
    // an order of magnitude before this line existed. 15 orbitals, 12 electrons per spin is
    // C(15,12) = 455 strings per spin — the production table's exact shape.
    let ooo = holon_chem::fci::FciSpace::new(15, 12, 12);
    let per_mib = holon_gpu::fci::vram_bytes_for(&ooo).unwrap() as f64 / (1u64 << 20) as f64;
    let ooo_workers = gp.max_workers_for(&ooo, 1024).expect("could not derive the bound");
    println!("(O,O,O) ({} det, {per_mib:.1} MiB/worker): {ooo_workers} GPU workers fit", ooo.n_det);
    assert_eq!(ooo.n_det, 207_025, "this is not the production table's shape");
    assert!(
        ooo_workers >= 20,
        "the (O,O,O) footprint admits only {ooo_workers} workers, against {per_mib:.1} MiB \
         each on a card with over 15 GB free. If this is genuinely low the footprint grew; if \
         it is not, the bound is wrong. Either way the claim that VRAM is what stops GPU-class \
         table generation needs re-deriving — it was ALREADY wrong once, by an order of \
         magnitude, in the direction that made the recommendation sound better supported."
    );

    // A space no single operator fits: the answer is 0, and 0 means REFUSE.
    let huge = holon_chem::fci::FciSpace::new(14, 7, 7);
    let none = gp.max_workers_for(&huge, 1024).expect("could not derive the bound");
    println!(
        "14-orbital 7/7 ({} det, {:.1} GB/worker): {none} GPU workers fit",
        huge.n_det,
        holon_gpu::fci::vram_bytes_for(&huge).unwrap() as f64 / 1e9
    );
    assert_eq!(
        none, 0,
        "a space needing more VRAM than the card has reported {none} workers; a bound that \
         cannot say `none` cannot refuse, and D4 forbids the fallback that would follow"
    );

    // The reserve is real: asking to hold back more than the card has leaves nothing.
    let (_free, total) = gp.mem_info().expect("device memory unreadable");
    let starved = gp
        .max_workers_for(&small, (total as u64 / (1 << 20)) * 2)
        .expect("could not derive the bound");
    assert_eq!(
        starved, 0,
        "a reserve larger than the card still admitted {starved} workers, so the reserve is \
         not being held back"
    );
}
