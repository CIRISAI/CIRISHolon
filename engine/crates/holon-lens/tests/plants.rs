//! THE PLANTS, exactly as `conformance/water_observatory/CENSUS_PREREG.md` §5 and §7
//! staked them, before the instrument existed.
//!
//! Each plant names the sector it acts on and is checked to FIRE on this instrument
//! (M-PLANT-OBS: observability is instrument-relative — a plant re-used from another
//! instrument's write-up is a belief, not a check). Where a plant is a REFUSAL, the test
//! also shows the instrument accepting the same carrier once the planted defect is
//! removed, so that "refused" is a fact about the plant and not about the fixture.

use holon_lens::census::{self, BlockVerdict, Census, Stakes};
use holon_lens::classifier::{self, Labelled, Phase};
use holon_lens::partition::Mask;
use holon_lens::synthetic::{self, Spec};

/// Four oxygens then eight hydrogens: the mixed arm's own composition, so a plant's
/// blocks have the same formulas the real census will be reading.
fn z12() -> Vec<u32> {
    vec![8, 8, 8, 8, 1, 1, 1, 1, 1, 1, 1, 1]
}

fn spec(seed: u64, n_frames: usize) -> Spec {
    let mut s = Spec::quench_like(n_frames, z12());
    s.seed = seed;
    s
}

/// Arena indices 0 (an O), 4 and 5 (two H): the OH2 the census is being built to judge.
const OH2: Mask = 0b0000_0011_0001;

fn report(t: &holon_lens::traj::Trajectory) -> census::CensusReport {
    match census::run(t, &Stakes::default()) {
        Census::Report(r) => r,
        Census::Refused { gate, reason } => panic!("unexpected refusal at {gate}: {reason}"),
    }
}

fn row<'a>(r: &'a census::CensusReport, m: Mask) -> &'a census::BlockReport {
    r.blocks
        .iter()
        .find(|b| b.block == m)
        .unwrap_or_else(|| panic!("block {m:#014b} never appeared"))
}

// ============================================================ C-1  must CERTIFY

/// Carrier: three atoms bonded to each other and to nothing else in every frame, with
/// thermal internal motion. Sector: MEMBERSHIP — the planted signal is nonzero in the
/// block-view series `v_B`, which reads 1 in every frame.
#[test]
fn c1_a_permanently_held_moving_block_is_certified_strict() {
    let t = synthetic::vibrating_block(spec(11, 1200), OH2, 0.4, |_| true);
    let r = report(&t);
    assert_eq!(r.window_frames, 1000, "the staked 834 fs window at this dt");
    let b = row(&r, OH2);
    assert_eq!(b.formula, "OH2");
    assert!(
        matches!(b.verdict, BlockVerdict::CertifiedStrict { .. }),
        "a census that cannot certify this cannot certify anything: {:?}",
        b.verdict
    );
    assert!(b.rms_internal >= census::PREREG_MIN_RMS_BOHR, "rms {}", b.rms_internal);
    assert!(b.max_sep_variation >= census::PREREG_MIN_SEP_VAR_BOHR);
    assert_eq!(b.control_rate, Some(0.0), "no peer block is ever a component here");
    assert!(b.control_pool > 0, "and the eligible pool is not empty");
}

// ============================================================ C-2  must REJECT

/// Carrier: the same block, bonded for half the window and then permanently split.
/// Sector: MEMBERSHIP — `v_B` transitions 1 to 0 and never returns.
#[test]
fn c2_a_dissociating_block_is_transient() {
    let t = synthetic::vibrating_block(spec(12, 1200), OH2, 0.4, |f| f < 600);
    let r = report(&t);
    let b = row(&r, OH2);
    assert!(
        matches!(b.verdict, BlockVerdict::Transient { .. }),
        "{:?}",
        b.verdict
    );
    assert_eq!(b.longest_run, 600);
}

// ============================================================ C-3  must REJECT

/// Carrier: held except for one breach run of `L_flick + 1` frames, whose TOTAL breach
/// fraction is inside the 2% budget. Sector: MEMBERSHIP, and the breach run IS the plant.
///
/// This is the plant that proves the budget is not an escape hatch. It fires, and the
/// second half of the test shows the instrument accepting the identical carrier once the
/// run cap alone is loosened — so the refusal is the run clause and nothing else.
#[test]
fn c3_a_breach_run_past_the_cap_is_refused_though_the_fraction_passes() {
    let flicker = 10usize; // floor(8.4 fs / 0.8338 fs)
    let breach = flicker + 1;
    let t = synthetic::vibrating_block(spec(13, 1200), OH2, 0.4, move |f| {
        !(500..500 + breach).contains(&f)
    });
    let r = report(&t);
    assert_eq!(r.flicker_frames, flicker);
    let b = row(&r, OH2);
    assert!(
        (breach as f64) / (r.window_frames as f64) < census::PREREG_BETA,
        "the plant's whole point: {breach} frames is inside the 2% budget"
    );
    assert!(
        matches!(b.verdict, BlockVerdict::Transient { .. }),
        "the run cap must refuse what the fraction admits: {:?}",
        b.verdict
    );

    // FIRES: loosen only the run cap and the same carrier certifies.
    let loose = Stakes {
        flicker_fs: 8.4 * 2.0,
        ..Stakes::default()
    };
    match census::run(&t, &loose) {
        Census::Report(rr) => {
            let bb = row(&rr, OH2);
            assert!(
                matches!(bb.verdict, BlockVerdict::CertifiedBudgeted { .. }),
                "{:?}",
                bb.verdict
            );
        }
        Census::Refused { gate, .. } => panic!("unexpected refusal at {gate}"),
    }
}

// ============================================================ C-4  must REJECT

/// THE NAMING ARTIFACT ITSELF. Carrier: a trajectory whose FINAL frame holds a
/// (1 O, 2 H) component while the membership of that component is reshuffled every three
/// frames. Sector: MEMBERSHIP — the COMPOSITION series is constant at OH2 throughout
/// while the BLOCK series flickers, so the plant acts on membership and not on formula.
///
/// The formula reader must say OH2 and the census must say TRANSIENT. This is the defect
/// the whole census exists to catch, and if it does not fire here nothing else in this
/// file matters.
#[test]
fn c4_a_reshuffling_oh2_is_named_by_formula_and_refused_by_closure() {
    let n = 12usize;
    let s = spec(14, 1200);
    // The oxygen stays; the two hydrogens step along every three frames.
    let t = synthetic::build(s, move |f, pos, vel| {
        let k = (f / 3) % 7;
        let (h1, h2) = (4 + k, 5 + k);
        for i in 0..n {
            let a = 0.4 * ((0.31 * f as f64) + i as f64).sin();
            pos[i] = [3.0 + 2.0 * (i as f64).cos() + a, 3.0 + 2.0 * (i as f64).sin(), 0.0];
            vel[i] = [a, 0.0, 0.0];
        }
        let block: Mask = (1 << 0) | (1 << h1) | (1 << h2);
        synthetic::bonds_from_blocks(n, &[block])
    });
    let r = report(&t);

    // The formula reader — what `waterquench` prints — sees a water molecule.
    assert!(
        r.final_frame_molecules.iter().any(|(_, f)| f == "OH2"),
        "connected-component naming must report OH2 here: {:?}",
        r.final_frame_molecules
    );

    // The census sees nothing that holds. Every OH2-shaped block is transient.
    let oh2_rows: Vec<_> = r.blocks.iter().filter(|b| b.formula == "OH2").collect();
    assert!(!oh2_rows.is_empty());
    for b in &oh2_rows {
        assert!(
            matches!(b.verdict, BlockVerdict::Transient { .. }),
            "block {:#014b} certified when its membership turns over every 3 frames: {:?}",
            b.block,
            b.verdict
        );
        assert!(b.longest_run <= 6, "longest run {}", b.longest_run);
    }
    assert_eq!(r.certified().count(), 0);
}

// ============================================================ C-5  must REFUSE

/// Carrier: a trajectory shorter than the window. Expected: a REFUSAL naming the gate
/// (Object rule 9), not a pass and not a fail.
#[test]
fn c5_a_trajectory_shorter_than_the_window_is_refused() {
    let t = synthetic::vibrating_block(spec(15, 100), OH2, 0.4, |_| true);
    match census::run(&t, &Stakes::default()) {
        Census::Refused { gate, reason } => {
            assert_eq!(gate, "G3/G4 window length");
            assert!(reason.contains("1000"), "the refusal names the window: {reason}");
        }
        Census::Report(_) => panic!("a 100-frame trajectory must not produce a verdict"),
    }
}

// ============================================================ C-6  must VOID

/// Carrier: a block held perfectly with every velocity zero and no geometry change.
/// Sector: nonzero in MEMBERSHIP (`v_B` reads 1 in every frame) and exactly ZERO in
/// MOTION — which is the vacuity G5 exists to catch (M-FIXED-POINT-TRAJECTORY).
#[test]
fn c6_a_frozen_carrier_voids_rather_than_certifies() {
    let t = synthetic::frozen_block(spec(16, 1200), OH2);
    let r = report(&t);
    let b = row(&r, OH2);
    assert!(
        matches!(b.verdict, BlockVerdict::VoidFrozenCarrier { .. }),
        "held forever on a carrier that never moves is VOID, not CERTIFIED: {:?}",
        b.verdict
    );
    assert_eq!(b.longest_run, 1200, "and it was held in every single frame");

    // FIRES: the same membership series on a MOVING carrier certifies, so the void is
    // about the motion sector and not about the fixture.
    let moving = synthetic::vibrating_block(spec(16, 1200), OH2, 0.4, |_| true);
    let rm = report(&moving);
    assert!(row(&rm, OH2).verdict.is_certified());
}

// ============================================================ Leg B on a real fixture

/// The closure leg must assert its WORK COUNT, and on a real fixture it exhibits the
/// witness pair the Object contract's normal form predicts.
///
/// This test was WRITTEN expecting the dissociating fixture to be uninformative, and it
/// was wrong in an instructive direction: the fixture is the most informative case there
/// is. While the block is held, the partition reading is constant, so the reading
/// `P = {block, singletons}` is visited 600 times — and at frame 599 its successor is a
/// DIFFERENT partition from the one it had the other 598 times. Those two frames are a
/// witness pair in the exact sense of `nonfactoring_iff_not_closed`: same reading, split
/// futures. The membership view cannot say from itself when the molecule will break, and
/// the instrument exhibits the frames where it fails to.
#[test]
fn leg_b_exhibits_the_witness_pair_a_dissociation_creates() {
    let held = synthetic::vibrating_block(spec(17, 1200), OH2, 0.4, |_| true);
    let r = report(&held);
    // One partition for the whole run: every transition informative, no witness pair, and
    // the honest reading is "none found at this resolution", never "Closed".
    assert_eq!(r.closure.distinct_readings, 1);
    assert_eq!(r.closure.informative_transitions, 1199);
    assert_eq!(r.closure.witness_pair_count, 0);
    assert_eq!(r.closure.defect, 0.0);
    assert!(!r.closure.void, "1199 transitions is well above the staked 200");

    let once = synthetic::vibrating_block(spec(18, 1200), OH2, 0.4, |f| f < 600);
    let r2 = report(&once);
    assert_eq!(r2.closure.distinct_readings, 2);
    assert!(!r2.closure.void);
    assert!(
        r2.closure.witness_pair_count >= 1,
        "a dissociation IS a closure failure of the membership view"
    );
    assert!(r2.closure.defect > 0.0);
    let (a, b) = r2.closure.witness_pairs[0];
    let key_of = |f: usize| {
        holon_lens::partition::key(&holon_lens::partition::labels_from_bonds(
            12,
            once.frames[f].bonded,
        ))
    };
    assert_eq!(key_of(a), key_of(b), "a witness pair agrees on the reading");
    assert_ne!(
        key_of(a + 1),
        key_of(b + 1),
        "and its two futures disagree — the square does not commute"
    );
}

// ============================================================ P-1  preset blindness

/// Carrier: a LIQUID trajectory whose launch label says `ice`. The plant is nonzero in
/// the LABEL sector (the label contradicts the truth) and exactly zero in the TRAJECTORY
/// sector (the coordinates are an unmodified liquid's).
///
/// Blindness here is structural: `classify` takes a `&Trajectory`, and `Trajectory` has
/// no label field, so the label is not merely ignored — it is unreachable.
#[test]
fn p1_a_liquid_launched_as_ice_classifies_liquid() {
    let truth = synthetic::liquid(spec(21, 600));
    let mislabelled = Labelled::new("ice", truth);
    assert_eq!(mislabelled.declared(), "ice", "the plant is nonzero in the label sector");
    let r = classifier::classify(mislabelled.trajectory());
    assert_eq!(
        r.verdict.phase(),
        Some(Phase::Liquid),
        "the classifier read its label: {r:?}"
    );
    assert!(!r.ice_criterion_fired);

    // FIRES: the same classifier on a scene that really is a crystal says ICE, so the
    // LIQUID verdict above is a reading and not a constant.
    let crystal = synthetic::crystal(spec(21, 600), 0.02);
    assert_eq!(
        classifier::classify(&crystal).verdict.phase(),
        Some(Phase::Ice)
    );
}

// ============================================================ P-5  vapor is never ice

/// Carrier: dilute-gas trajectories. The plant acts on the DENSITY sector.
///
/// Staked in the prereg before the classifier existed: no vapor trajectory may classify
/// ICE, and the published false-crystal rate must sit under the 1.5% bound that 0/200
/// would have justified.
///
/// It FIRED THREE TIMES during construction — 4.0%, then 1.6%, then 0.2% — and each
/// firing named a different defect in the order parameter (no finite-N floor; six
/// neighbours mistaken for a first shell; one atom's environment counted across
/// correlated frames as if it were a bulk). The classifier module header carries the
/// derivations. What is asserted here is the MEASURED rate, not a zero: at 0.2% a
/// 1000-draw test expects about two firings, and a test asserting zero would be flaky
/// while claiming to be strict.
///
/// The verdict-level assertion alone would be vacuous — VAPOR is the first branch, so a
/// gas caught there never reaches the ICE test (M-VACUOUS-SUCCESS). So the rate is taken
/// on `ice_criterion_fired`, which is evaluated unconditionally.
#[test]
fn p5_the_false_crystal_rate_sits_under_its_staked_bound() {
    const N: usize = 1000;
    const STAKED_BOUND: f64 = 0.015; // what 0/200 would have justified at 95%
    let mut ice_verdicts = 0usize;
    let mut criterion_fires = 0usize;
    let mut vapor_verdicts = 0usize;
    for s in 0..N as u64 {
        let t = synthetic::vapor(spec(1000 + s, 300));
        let r = classifier::classify(&t);
        match r.verdict.phase() {
            Some(Phase::Ice) => ice_verdicts += 1,
            Some(Phase::Vapor) => vapor_verdicts += 1,
            _ => {}
        }
        if r.ice_criterion_fired {
            criterion_fires += 1;
        }
    }
    assert_eq!(vapor_verdicts, N, "the work count: every trajectory was read and classified");
    assert_eq!(ice_verdicts, 0, "a gas classified as a crystal");
    let rate = criterion_fires as f64 / N as f64;
    assert!(
        rate <= STAKED_BOUND,
        "false-crystal rate {rate:.4} ({criterion_fires}/{N}) exceeds the staked {STAKED_BOUND}"
    );

    // FIRES: the same criterion says yes on a real crystal, so the rate above is a rate
    // and not a criterion that can never fire at all (M-PLANT-OBS).
    let mut cs = Spec::quench_like(300, vec![8; 16]);
    cs.seed = 5;
    let crystal = synthetic::crystal(cs, 0.02);
    assert!(
        classifier::classify(&crystal).ice_criterion_fired,
        "the ICE criterion must be able to fire, or 0/N means nothing"
    );
}
