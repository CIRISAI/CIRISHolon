//! ION TABLES: the eleven gates of `conformance/atomworld/ION_TABLES_PREREG.md`.
//!
//! Every number here is MEASURED in STO-3G full CI at STAKED geometries, and none is
//! quoted from anywhere. The freeze is `conformance/atomworld/ION_TABLES_PREREG.md`,
//! committed alone at `cce77f6` before `src/ion_table.rs` existed; the gate ids below are
//! its gate ids and the plant ids are its plant ids.
//!
//! # The one sentence
//!
//! A table row that does not name its charge and its spin sector is unusable, and must be
//! impossible to construct. G2 is the type-level half of that (the compiler is the gate);
//! G8 is the artifact half.
//!
//! # What "the gate is green" is allowed to mean
//!
//! Each gate that could pass by doing nothing asserts its own WORK COUNT — how many knots
//! were compared, how many held-out points were sampled, how many fields were checked —
//! because a verifier that reports success for work it did not do is the failure shape
//! `MISFITS.md` registers as `M-VACUOUS-SUCCESS`, and it has fired in this tree three
//! times.

use holon_chem::dual::D2;
use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::fci::{SolveExit, SolverRoute};
use holon_chem::ion_table::{
    charged_route, generate_ion_table, generate_ion_table_planted, interpolant_error, parse_spec,
    Channel, Fragment, IonKey, IonTable, Plant, StretchCut, TableRefusal, FENCE_I5,
};
use holon_chem::ions::{solve_geometry_charged, ChargeRefusal};
use holon_chem::pair::{generate_pair_table, solve_geometry};

// ---------------------------------------------------------------- the staked geometry
//
// Bohr and degrees, TAKEN UNCHANGED from `tests/ion_core.rs`'s staked pyramid so the cut
// passes through the certified point. They are re-declared rather than imported because an
// integration test cannot see another integration test's constants; G9 is the gate that
// makes the duplication safe by measuring that the two agree.

/// O–H separation in H3O+, and the point on the cut where node C certified the species.
const R_OH_H3O: f64 = 1.85;
/// H–O–H angle in H3O+, degrees.
const ANGLE_H3O_DEG: f64 = 113.0;
/// O–H separation in H2O.
const R_OH_H2O: f64 = 1.81;
/// H–O–H angle in H2O, degrees.
const ANGLE_H2O_DEG: f64 = 104.5;

/// The staked domain of the H3O+ table, bohr.
const Q_MIN: f64 = 1.0;
const Q_MAX: f64 = 12.0;
/// Knots on the staked domain.
const KNOTS: usize = 96;
/// The staked held-out sample: these grid intervals, at these fractions inside each.
const HELD_OUT_INTERVALS: [usize; 8] = [0, 12, 24, 36, 48, 60, 72, 84];

/// The staked interior offsets. DERIVED, not convenient: the cubic Hermite VALUE error
/// peaks near the midpoint while its SLOPE error peaks at `1/2 ± 1/(2√3)`, and a campaign
/// in this tree read its own bound 1.5% low by sampling `{1/4, 1/2, 3/4}`.
fn held_out_positions() -> [f64; 3] {
    let d = 1.0 / (2.0 * 3.0f64.sqrt());
    [0.5 - d, 0.5, 0.5 + d]
}

/// The staked kills, from the freeze.
const G5_ASYMPTOTE_BOUND: f64 = 1e-3;
const G6_INTERPOLANT_KILL: f64 = 1e-4;
const G7_BOUNDARY_KILL: f64 = 1e-3;
const G7_DECAY_FACTOR: f64 = 4.0;
const G9_ANCHOR_BOUND: f64 = 1e-12;
/// `ion_core.rs`'s pinned reading, which G9 must reproduce through the table's own path.
const PROTON_AFFINITY_MEASURED: f64 = 0.379432332077;

// -------------------------------------------------------------------- the cuts

/// The C3v cone's polar angle, derived from the staked H–O–H angle exactly as
/// `ion_core.rs` derives it: two unit vectors at polar angle `t` with azimuths 120° apart
/// have dot product `(3cos²t − 1)/2`.
fn cone() -> (f64, f64) {
    let cos_hoh = ANGLE_H3O_DEG.to_radians().cos();
    let cos_t = ((2.0 * cos_hoh + 1.0) / 3.0).sqrt();
    (cos_t, (1.0 - cos_t * cos_t).sqrt())
}

fn cone_dir(k: usize) -> [f64; 3] {
    let (cos_t, sin_t) = cone();
    let phi = 2.0 * std::f64::consts::PI * (k as f64) / 3.0;
    [sin_t * phi.cos(), sin_t * phi.sin(), cos_t]
}

/// H3O+ with slot 3 leaving along its own bond direction. Slots: O, H, H, H.
fn h3o_cut() -> StretchCut<4> {
    let (cos_t, sin_t) = cone();
    let frozen = |k: usize| {
        let phi = 2.0 * std::f64::consts::PI * (k as f64) / 3.0;
        [
            R_OH_H3O * sin_t * phi.cos(),
            R_OH_H3O * sin_t * phi.sin(),
            R_OH_H3O * cos_t,
        ]
    };
    StretchCut::new(
        [OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN],
        [[0.0, 0.0, 0.0], frozen(0), frozen(1), [0.0; 3]],
        3,
        [0.0, 0.0, 0.0],
        cone_dir(2),
        "r(O-H3), bohr: one O-H bond of the staked C3v pyramid, the other two frozen",
    )
}

/// The two channels of that cut, each stating its fragments' charges. Slot bit 3 is the
/// leaving hydrogen.
const H3O_CHANNEL_A: [Fragment; 2] = [
    Fragment { slots: 0b0111, charge: 0, label: "H2O" },
    Fragment { slots: 0b1000, charge: 1, label: "H+" },
];
const H3O_CHANNEL_B: [Fragment; 2] = [
    Fragment { slots: 0b0111, charge: 1, label: "H2O+" },
    Fragment { slots: 0b1000, charge: 0, label: "H" },
];

fn h3o_channels() -> [Channel<'static>; 2] {
    [
        Channel { label: "H2O + H+", fragments: &H3O_CHANNEL_A },
        Channel { label: "H2O+ + H", fragments: &H3O_CHANNEL_B },
    ]
}

/// A neutral diatomic cut: slot 0 at the origin, slot 1 out along `+z`. The SAME cut type
/// the cation uses, with different data.
fn diatomic_cut(a: holon_chem::elements::Species, b: holon_chem::elements::Species) -> StretchCut<2> {
    StretchCut::new(
        [a, b],
        [[0.0; 3]; 2],
        1,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        "separation, bohr",
    )
}

const DIATOMIC_ATOMS: [Fragment; 2] = [
    Fragment { slots: 0b01, charge: 0, label: "A" },
    Fragment { slots: 0b10, charge: 0, label: "B" },
];

// ============================================================ G1 + P1 + P2 + P5

/// **G1 — the neutral path does not move by one bit**, and three plants that prove the
/// gate can see a defect.
///
/// The charge-0 instance of the ion generator is run on the grid `generate_pair_table`
/// derives for itself, and every knot of every column is compared on raw `f64` bits. Two
/// species: H2 (even electron count, singlet) and OH (odd count, doublet — the branch of
/// the parity rule node C's charge-0 regression never covered).
///
/// A tolerance here would pass against a re-implementation that agreed to fifteen digits
/// and then diverged the first time either side changed.
#[test]
fn g1_the_neutral_path_does_not_move_by_one_bit() {
    let n = 24usize;
    let mut compared = 0usize;
    for (label, a, b) in [("H2", HYDROGEN, HYDROGEN), ("OH", OXYGEN, HYDROGEN)] {
        let pt = generate_pair_table(a, b, n);
        let cut = diatomic_cut(a, b);
        let ch = [Channel { label: "two atoms", fragments: &DIATOMIC_ATOMS }];
        let it = generate_ion_table(&cut, 0, pt.meta.r_min, pt.meta.r_max, n, &ch)
            .expect("a neutral diatomic is a stateable charge on this door");
        println!(
            "  {label:<3} knots {n}  pair route {:?} exit {:?}  ion route {:?} exit {:?}  \
             pair asym {:.12}  ion asym {:.12} ({})",
            pt.meta.route,
            pt.meta.exit,
            it.meta.route,
            it.meta.exit,
            pt.meta.e_asymptote,
            it.meta.e_asymptote,
            it.meta.asymptote_channel
        );
        assert_eq!(it.q.len(), n, "{label}: the ion table has the wrong knot count");
        for i in 0..n {
            assert_eq!(it.q[i].to_bits(), pt.r[i].to_bits(), "{label}: grid knot {i}");
            assert_eq!(
                it.e[i].to_bits(),
                pt.e[i].to_bits(),
                "{label}: ENERGY at knot {i} — the ion door has become a second \
                 implementation of the neutral path. Delete it; do not repair it."
            );
            assert_eq!(it.f[i].to_bits(), pt.f[i].to_bits(), "{label}: force at knot {i}");
            assert_eq!(it.e2[i].to_bits(), pt.e2[i].to_bits(), "{label}: curvature at knot {i}");
            compared += 4;
        }
        // The channel machinery reproduces the pair path's asymptote too — computed from
        // two stated charge-0 fragments rather than from a hardcoded sum of two atoms.
        assert!(
            (it.meta.e_asymptote - pt.meta.e_asymptote).abs() < 1e-14,
            "{label}: the stated-fragment asymptote {:.15} differs from the pair path's \
             {:.15}",
            it.meta.e_asymptote,
            pt.meta.e_asymptote
        );

        // --- the plants. Each must MOVE the energy column; a silent plant means the
        // mutation is unobservable, and then neither it nor the gate may be reported.
        for plant in [Plant::EnergyUlp, Plant::ChargeOffByOne, Plant::SectorShift] {
            let planted =
                generate_ion_table_planted(&cut, 0, pt.meta.r_min, pt.meta.r_max, n, &ch, plant)
                    .expect("every plant here is a defect in the numbers, not in the door");
            let moved = (0..n).filter(|&i| planted.e[i].to_bits() != pt.e[i].to_bits()).count();
            let worst = (0..n)
                .map(|i| (planted.e[i] - pt.e[i]).abs())
                .fold(0.0f64, f64::max);
            println!("    plant {plant:?}: {moved}/{n} knots moved, worst |dE| {worst:.3e} Ha");
            assert!(
                moved > 0,
                "{label}: plant {plant:?} left every knot bit-identical. The MUTATION is \
                 unobservable on this instrument, not the gate — suspect the plant first."
            );
        }
    }
    assert_eq!(compared, 2 * 4 * n, "G1 compared {compared} values, not {}", 2 * 4 * n);
    println!("  G1: {compared} bit comparisons over 2 species, all equal; 6 plant runs all fired");
}

// ================================================================= G3 and G4

/// **G3 — an unstated charge REFUSES**, and **G4 — anions are REFUSED under fence I-5**.
///
/// Both are two-legged on purpose: a refusal that fires because the whole door is broken
/// proves nothing, so each refusing input is paired with a served one.
#[test]
fn g3_g4_the_two_refusal_doors() {
    // --- G3, at the spec door. P4 is the refused form.
    assert_eq!(
        parse_spec("O H H H").unwrap_err(),
        TableRefusal::UnstatedCharge { spec: "O H H H".to_string() },
        "a spec ending in an element has NOT stated a charge and must never be read as \
         neutral"
    );
    let (sp, charge) = parse_spec("O H H H +1").expect("the stated form must be served");
    assert_eq!((sp.len(), charge), (4, 1));
    let key: IonKey<4> = IonKey::state(&[sp[0], sp[1], sp[2], sp[3]], charge).expect("H3O+ keys");
    assert_eq!((key.charge(), key.n_electrons(), key.sz2()), (1, 10, 0));
    println!("  G3: 'O H H H' REFUSED (UnstatedCharge); 'O H H H +1' served, key {key:?}");

    // --- G4, at the table door. The anion is refused; the neutral and the cation are not.
    let cut = diatomic_cut(OXYGEN, HYDROGEN);
    let anion_channel = [
        Fragment { slots: 0b01, charge: -1, label: "O-" },
        Fragment { slots: 0b10, charge: 0, label: "H" },
    ];
    let refused = generate_ion_table(
        &cut,
        -1,
        1.5,
        6.0,
        8,
        &[Channel { label: "O- + H", fragments: &anion_channel }],
    );
    match refused {
        Err(TableRefusal::AnionFenced { fence, charge, cause }) => {
            println!("  G4: OH- table REFUSED at fence {fence}, charge {charge}");
            println!("      cause: {cause}");
            assert_eq!(fence, FENCE_I5, "the refusal must name the fence that fired");
            assert_eq!(charge, -1);
            assert!(
                cause.contains("diffuse") && cause.contains("STO-3G"),
                "the refusal must name the CAUSE (no diffuse functions in STO-3G), not just \
                 the symptom"
            );
            assert!(
                cause.contains("I-5"),
                "the refusal must name its exit so the fence is a debt and not architecture"
            );
        }
        other => panic!(
            "an anion table was NOT refused under fence I-5. The fired gate has been \
             quietly lifted and this model's anion energies are being published as \
             chemistry. Got: {other:?}"
        ),
    }
    // The sanity legs: the same door, the same species, at 0 and at +1.
    let neutral_ch = [Channel { label: "O + H", fragments: &DIATOMIC_ATOMS }];
    let neutral = generate_ion_table(&cut, 0, 1.5, 6.0, 8, &neutral_ch)
        .expect("SANITY: neutral OH must be served, or the anion refusal proves nothing");
    let cation_frag = [
        Fragment { slots: 0b01, charge: 1, label: "O+" },
        Fragment { slots: 0b10, charge: 0, label: "H" },
    ];
    let cation = generate_ion_table(
        &cut,
        1,
        1.5,
        6.0,
        8,
        &[Channel { label: "O+ + H", fragments: &cation_frag }],
    )
    .expect("SANITY: the cation must be served — the fence is about anions, not about charge");
    println!(
        "      sanity: OH  served (asym {:.9}, {} knots), OH+ served (asym {:.9}, {} knots)",
        neutral.meta.e_asymptote,
        neutral.q.len(),
        cation.meta.e_asymptote,
        cation.q.len()
    );
    assert_eq!(neutral.meta.key.charge(), 0);
    assert_eq!(cation.meta.key.charge(), 1);
    // And the arithmetic refusals still come back under their own names rather than being
    // swallowed by the fence: an anion beyond the nuclear charge is not a system at all.
    assert_eq!(
        IonKey::state(&[HYDROGEN], -2).unwrap_err(),
        ChargeRefusal::ChargeTooLarge { total_z: 1, charge: -2 }
    );
}

// ================================================================= G10 and G11

/// **G10 — the feasibility door refuses with NUMBERS**, and **G11 — a channel that does
/// not conserve charge is refused** — both BEFORE anything is spent.
///
/// "Before anything is spent" is not asserted by a clock, which would be a measurement of
/// the machine's load rather than of the code. It is asserted STRUCTURALLY: every cut here
/// puts its centres at ONE POINT, a geometry whose overlap matrix is singular and whose
/// solve panics inside the orthogonaliser. A refusal that comes back at all is a refusal
/// that came back before the solve.
#[test]
fn g10_g11_the_door_refuses_before_it_spends() {
    // (H3O+ · H2O): I-2's headline ionic pair. Two oxygens and five hydrogens, +1.
    let species = [OXYGEN, OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN, HYDROGEN, HYDROGEN];
    let key: IonKey<7> = IonKey::state(&species, 1).expect("O2H5+ is a stateable charge");
    // Routing is admission by price (2026-09-02): whether THIS machine admits the pair's
    // 9M-determinant working set is a machine fact, printed below. The gate's claim — the
    // door refuses WITH NUMBERS before anything is spent — is exercised under a PLANTED
    // refusing door, which is the only way to make it fire on every machine.
    {
        let (d, o, real) = charged_route(&species, &key);
        println!("      this machine's door: n_det {d}, n_orb {o}, automatic route exists = {real}");
    }
    holon_chem::budget::set_admission(Some(Box::new(
        holon_resource::probe::ScriptedProbe::always_fail("planted: the door refuses"),
    )));
    let (n_det, n_orb, exists) = charged_route(&species, &key);
    println!(
        "  G10: (H3O+ . H2O) priced: {n_orb} orbitals, {n_det} determinants, \
         {} electrons as ({}, {}), automatic route exists = {exists}",
        key.n_electrons(),
        key.partition().0,
        key.partition().1
    );
    assert_eq!(key.n_electrons(), 20);
    assert!(n_orb >= 15, "seven first-row atoms carry at least 15 STO-3G orbitals");
    assert!(n_det > 1_000_000, "the pair is a multi-million-determinant space: {n_det}");
    assert!(!exists, "under a refusing door the automatic router has no route for {n_det} determinants");

    let singular = StretchCut::new(
        species,
        [[0.0; 3]; 7],
        6,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        "a geometry no solve survives",
    );
    let frags = [
        Fragment { slots: 0b0011111, charge: 1, label: "H3O+" },
        Fragment { slots: 0b1100000, charge: 0, label: "H2O" },
    ];
    let refusal = generate_ion_table(
        &singular,
        1,
        1.0,
        6.0,
        4,
        &[Channel { label: "H3O+ + H2O", fragments: &frags }],
    );
    match refusal {
        Err(TableRefusal::PastAutomaticRoute { n_det: d, n_orb: o, det_price_bytes, mpo_price_bytes }) => {
            println!(
                "      REFUSED with numbers: n_det {d} (working set {det_price_bytes} bytes), n_orb {o} \
                 (MPO ~{mpo_price_bytes} bytes)"
            );
            assert_eq!((d, o), (n_det, n_orb), "the refusal must carry the price it computed");
        }
        other => panic!("the priced-out species was not refused with its numbers: {other:?}"),
    }

    holon_chem::budget::set_admission(None);
    // --- G11, on an affordable species so the refusal cannot be the price in disguise.
    let cut = h3o_cut();
    let bad = [
        Fragment { slots: 0b0111, charge: 0, label: "H2O" },
        Fragment { slots: 0b1000, charge: 0, label: "H" },
    ];
    let got = generate_ion_table(
        &cut,
        1,
        Q_MIN,
        Q_MAX,
        4,
        &[Channel { label: "does not conserve", fragments: &bad }],
    );
    assert_eq!(
        got.unwrap_err(),
        TableRefusal::ChannelChargeNotConserved {
            channel: "does not conserve",
            declared_total: 1,
            fragment_sum: 0
        },
        "G11: fragment charges that do not sum to the cluster's must refuse, by INTEGER \
         identity and never by tolerance"
    );
    let overlapping = [
        Fragment { slots: 0b0111, charge: 0, label: "H2O" },
        Fragment { slots: 0b1100, charge: 1, label: "H+" },
    ];
    assert!(
        matches!(
            generate_ion_table(
                &cut,
                1,
                Q_MIN,
                Q_MAX,
                4,
                &[Channel { label: "overlaps", fragments: &overlapping }]
            ),
            Err(TableRefusal::ChannelSlotsNotAPartition { .. })
        ),
        "G11: fragments that share a slot are not a partition"
    );
    println!("  G11: both structural channel refusals fired on an affordable species");
}

// ====================================================================== G9 + P2

/// **G9 — the anchor**: at `q = 1.85` bohr the cut IS node C's staked C3v pyramid, so the
/// proton affinity recomputed through the table's own path must reproduce `ion_core.rs`'s
/// pinned reading.
///
/// This is the gate that ties a new artifact to an old receipt. It is a tolerance and not
/// a bit comparison because the two paths associate the same geometry's products in a
/// different order — `q * (sin t cos φ)` here against `(R sin t) * cos φ` there.
#[test]
fn g9_the_certified_core_point_is_on_the_cut() {
    let cut = h3o_cut();
    let e_h3o = solve_geometry_charged(&cut.species(), cut.centers(D2::var(R_OH_H3O)), 1)
        .expect("H3O+ solves on the cut")
        .e
        .v;

    // H2O at ion_core's own staked geometry, written with ion_core's own association.
    let a = ANGLE_H2O_DEG.to_radians();
    let h2o = vec![
        [D2::c(0.0), D2::c(0.0), D2::c(0.0)],
        [D2::c(R_OH_H2O), D2::c(0.0), D2::c(0.0)],
        [D2::c(R_OH_H2O * a.cos()), D2::c(R_OH_H2O * a.sin()), D2::c(0.0)],
    ];
    let e_h2o = solve_geometry(&[OXYGEN, HYDROGEN, HYDROGEN], h2o).e.v;

    let affinity = e_h2o - e_h3o;
    println!(
        "  G9: E(H3O+) on the cut at q = {R_OH_H3O} bohr = {e_h3o:.12} Ha\n      \
         E(H2O) at the staked geometry              = {e_h2o:.12} Ha\n      \
         proton affinity = {affinity:+.12} Ha, pinned {PROTON_AFFINITY_MEASURED:+.12}, \
         delta {:.3e}",
        (affinity - PROTON_AFFINITY_MEASURED).abs()
    );
    assert!(
        (affinity - PROTON_AFFINITY_MEASURED).abs() < G9_ANCHOR_BOUND,
        "the cut does not pass through the certified point: affinity {affinity:+.12} against \
         the pinned {PROTON_AFFINITY_MEASURED:+.12} Ha. Either the cut is not the staked \
         pyramid or the charged seam has moved."
    );

    // P2, on this gate: the same computation one electron off must NOT reproduce the pin.
    let planted = solve_geometry_charged(&cut.species(), cut.centers(D2::var(R_OH_H3O)), 2)
        .expect("H3O2+ is stateable")
        .e
        .v;
    let planted_affinity = e_h2o - planted;
    println!(
        "      P2 (charge off by one): affinity would read {planted_affinity:+.12} Ha, \
         {:.3} Ha from the pin",
        (planted_affinity - PROTON_AFFINITY_MEASURED).abs()
    );
    assert!(
        (planted_affinity - PROTON_AFFINITY_MEASURED).abs() > 1e-3,
        "P2 is unobservable at this gate: solving one charge off reproduces the pinned \
         affinity. Suspect the plant first."
    );
}

// ============================================== G2 + G5 + G6 + G7 + G8, on the table

/// The staked H3O+ table, generated once and put through the five gates that need it.
///
/// One generation, because it is the campaign's whole compute: 96 knots plus the well
/// bisection plus the channels plus 24 held-out solves.
#[test]
fn g2_g5_g6_g7_g8_the_h3o_plus_table() {
    let cut = h3o_cut();
    let channels = h3o_channels();
    let t0 = std::time::Instant::now();
    let table = generate_ion_table(&cut, 1, Q_MIN, Q_MAX, KNOTS, &channels)
        .expect("the staked H3O+ table generates");
    let gen_s = t0.elapsed().as_secs_f64();
    let m = &table.meta;
    println!(
        "  H3O+ table: {} knots on [{}, {}] bohr, {} basis, {} determinants, route {:?}, \
         exit {:?}, device {}, budget {}, worst residual {:.3e}, {gen_s:.1} s",
        m.n_knots,
        m.q_min,
        m.q_max,
        m.n_basis,
        m.n_det,
        m.route,
        m.exit,
        m.device,
        m.solver_budget_iterations,
        m.worst_residual
    );

    // ---------------------------------------------------------------- G2: the key
    assert_eq!(m.key.charge(), 1, "G2: the key must carry the charge");
    assert_eq!(m.key.sz2(), 0, "G2: the key must carry the sector");
    assert_eq!(m.key.n_electrons(), 10);
    assert_eq!(m.key.partition(), (5, 5));
    assert_eq!(m.key.class().zs(), [1, 1, 1, 8], "G2: the key must carry the composition");
    assert_eq!(m.exit, SolveExit::Converged);
    assert_eq!(m.route, SolverRoute::Determinant);
    assert!(m.scf_converged_everywhere);

    // ------------------------------------------------- G5: the asymptote is the MINIMUM
    println!("  G5: channels as enumerated —");
    for c in &m.channels {
        let terms: Vec<String> = c
            .fragments
            .iter()
            .map(|(l, ch, e)| format!("{l}({ch:+}) = {e:.12}"))
            .collect();
        println!("      {:<10} sum {:.12} Ha   [{}]", c.label, c.sum, terms.join(" + "));
    }
    assert_eq!(m.channels.len(), 2, "G5 needs every enumerated channel measured");
    let lowest = m
        .channels
        .iter()
        .fold(f64::INFINITY, |b, c| if c.sum < b { c.sum } else { b });
    assert_eq!(
        m.e_asymptote, lowest,
        "G5: the declared asymptote is not the minimum over the enumerated channels"
    );
    let gap = m.channels[0].sum - m.channels[1].sum;
    let residual = table.e[KNOTS - 1] - m.e_asymptote;
    println!(
        "      asymptote {:.12} Ha from channel '{}'; the other channel is {gap:+.6} Ha away.\n      \
         E(q_max) - asymptote = {residual:+.6e} Ha",
        m.e_asymptote, m.asymptote_channel
    );
    assert!(
        residual.abs() <= G5_ASYMPTOTE_BOUND,
        "G5 FIRED: the curve sits {residual:+.6e} Ha from its declared asymptote, past the \
         staked {G5_ASYMPTOTE_BOUND:.0e}. A curve significantly BELOW its asymptote means \
         the channel enumeration is incomplete."
    );
    assert!(
        residual != 0.0,
        "G5 VOID: an exact zero means the curve and the channel sum are the same \
         computation, and the check tested nothing"
    );

    // ------------------------------------------------ G6: held-out interpolant fidelity
    let positions = held_out_positions();
    let (worst_e, worst_f, points) =
        interpolant_error(&table, &cut, &HELD_OUT_INTERVALS, &positions);
    println!(
        "  G6: {points} held-out points ({} intervals x {} offsets, none on a node): worst \
         |dE| {worst_e:.4e} Ha, worst |dF| {worst_f:.4e} Ha/bohr",
        HELD_OUT_INTERVALS.len(),
        positions.len()
    );
    assert_eq!(
        points,
        HELD_OUT_INTERVALS.len() * positions.len(),
        "G6 sampled {points} points, not the staked {}",
        HELD_OUT_INTERVALS.len() * positions.len()
    );
    assert!(
        worst_e > 0.0,
        "G6 VOID: an exact zero means the draw hit grid nodes and the check tested nothing"
    );
    assert!(
        worst_e <= G6_INTERPOLANT_KILL,
        "G6 FIRED: the interpolant departs from the model by {worst_e:.4e} Ha, past the \
         staked {G6_INTERPOLANT_KILL:.0e}. The grid is too coarse for the declared domain; \
         move the knot count, NOT the domain."
    );

    // ------------------------------------------------- G7: the boundary systematic
    let mid = table
        .q
        .iter()
        .enumerate()
        .min_by(|a, b| (a.1 - 6.0).abs().partial_cmp(&(b.1 - 6.0).abs()).unwrap())
        .map(|(i, _)| i)
        .expect("a non-empty grid");
    let at_mid = (table.e[mid] - m.e_asymptote).abs();
    println!(
        "  G7: boundary systematic |E(q_max) - asym| = {:.4e} Ha at q = {:.3}; at the knot \
         nearest 6 bohr (q = {:.3}) it is {at_mid:.4e} Ha — a factor {:.3e}",
        m.boundary_systematic,
        m.q_max,
        table.q[mid],
        at_mid / m.boundary_systematic.max(f64::MIN_POSITIVE)
    );
    assert!(
        m.boundary_systematic <= G7_BOUNDARY_KILL,
        "G7 FIRED: the domain truncation discards {:.4e} Ha. That is a finding for GANTT \
         node B2 and is reported as one; nothing here adds a long-range term to rescue it.",
        m.boundary_systematic
    );
    assert!(
        m.boundary_systematic * G7_DECAY_FACTOR <= at_mid,
        "G7 FIRED on its decay leg: the residual at the boundary ({:.4e}) is not at most a \
         quarter of the residual at 6 bohr ({at_mid:.4e}). A tail that does not fall off is \
         exactly what an unhandled long-range term looks like.",
        m.boundary_systematic
    );

    // ------------------------------------------------------------- G8: the disclosure
    let json = table
        .clone()
        .to_json()
        .expect_err("G8: a table whose interpolant error was never measured must REFUSE");
    assert_eq!(
        json,
        TableRefusal::InterpolantErrorNotMeasured,
        "G8: the refusal must name the missing measurement"
    );
    let published = table.clone().with_interpolant_uncertainty(worst_e);
    let text = published.to_json().expect("a measured table serialises");
    let required = [
        "\"charge\":",
        "\"sz2\":",
        "\"n_electrons\":",
        "\"class_z\":",
        "\"solver_exit\":",
        "\"uncertainty_hartree\":",
        "\"solver_budget_iterations\":",
        "\"interpolant_uncertainty_hartree\":",
        "\"solver_route\":",
        "\"device_class\":",
        "\"asymptote_channel\":",
        "\"boundary_systematic_hartree\":",
        "\"coordinate\":",
        "\"sector_rule\":",
    ];
    for field in required {
        assert!(
            text.contains(field),
            "G8: the emitted table does not carry {field} — a row source missing any of \
             these is a number whose meaning is missing"
        );
    }
    assert_eq!(
        required.len(),
        14,
        "G8 checked {} fields; the staked set is 14",
        required.len()
    );
    println!("  G8: {} required fields present on the emitted row source", required.len());

    // The three disclosure fields must be TOGETHER — a residual quoted without its budget
    // is a number whose meaning is missing, and separation in the file is how that happens.
    let exit_at = text.find("\"solver_exit\":").expect("solver_exit");
    let unc_at = text.find("\"uncertainty_hartree\":").expect("uncertainty");
    let bud_at = text.find("\"solver_budget_iterations\":").expect("budget");
    assert!(
        exit_at < unc_at && unc_at < bud_at && bud_at - exit_at < 400,
        "G8: solver_exit, uncertainty_hartree and solver_budget_iterations must sit \
         together on the row source (found at {exit_at}, {unc_at}, {bud_at})"
    );

    report_well(&published);
}

fn report_well(t: &IonTable<4>) {
    match t.meta.well {
        Some(w) => println!(
            "  well: q_e {:.6} bohr, D_e {:.6} Ha against '{}', E(q_e) {:.12}, k_e {:.6}\n  \
             (fragments FROZEN at their in-complex geometry — D_e OVERSTATES the relaxed depth)",
            w.r_e, w.d_e, t.meta.asymptote_channel, w.e_at_r_e, w.k_e
        ),
        None => println!("  well: none deeper than the schema's floor below the declared asymptote"),
    }
}

// ========================================================================= P3

/// **P3 — dropping the lowest channel fires G5.**
///
/// The plant runs on a REDUCED grid (12 knots) rather than the staked 96, and that is a
/// stated argument rather than a shortcut: the quantity the plant acts on is the channel
/// ENUMERATION, which is computed after the knot loop and does not depend on the knot
/// count. The test proves that rather than asserting it — the reduced run's channel sums
/// are compared bit for bit against the staked table's, and only then is the plant's
/// verdict read off the reduced grid.
#[test]
fn p3_dropping_the_lowest_channel_fires_g5() {
    let cut = h3o_cut();
    let channels = h3o_channels();
    let n = 12usize;

    let honest = generate_ion_table(&cut, 1, Q_MIN, Q_MAX, n, &channels)
        .expect("the reduced-grid table generates");
    let planted =
        generate_ion_table_planted(&cut, 1, Q_MIN, Q_MAX, n, &channels, Plant::DropLowestChannel)
            .expect("the planted table generates");

    // The knot count does not move the channel sums: bit-identical, both channels.
    assert_eq!(honest.meta.channels.len(), 2);
    assert_eq!(planted.meta.channels.len(), 1, "the plant must remove exactly one channel");
    let survivor = &planted.meta.channels[0];
    let matching = honest
        .meta
        .channels
        .iter()
        .find(|c| c.label == survivor.label)
        .expect("the surviving channel is one of the two");
    assert_eq!(
        survivor.sum.to_bits(),
        matching.sum.to_bits(),
        "the plant must not perturb the channel it leaves behind"
    );
    assert!(
        honest.meta.e_asymptote < planted.meta.e_asymptote,
        "the plant must have removed the LOWEST channel"
    );

    let honest_residual = (honest.e[n - 1] - honest.meta.e_asymptote).abs();
    let planted_residual = (planted.e[n - 1] - planted.meta.e_asymptote).abs();
    println!(
        "  P3: honest asymptote {:.12} ('{}') -> residual {honest_residual:.3e} Ha\n      \
         planted asymptote {:.12} ('{}') -> residual {planted_residual:.3e} Ha, \
         {:.0}x the staked bound",
        honest.meta.e_asymptote,
        honest.meta.asymptote_channel,
        planted.meta.e_asymptote,
        planted.meta.asymptote_channel,
        planted_residual / G5_ASYMPTOTE_BOUND
    );
    assert!(
        honest_residual <= G5_ASYMPTOTE_BOUND,
        "the honest run must PASS G5, or the plant's firing proves nothing"
    );
    assert!(
        planted_residual > 100.0 * G5_ASYMPTOTE_BOUND,
        "P3 is unobservable: dropping the lowest channel moved G5's residual to \
         {planted_residual:.3e} Ha, not past 100x the staked {G5_ASYMPTOTE_BOUND:.0e}. \
         Suspect the plant first."
    );
    // And the well the plant would have published, which is the damage the gate prevents.
    if let (Some(h), Some(p)) = (honest.meta.well, planted.meta.well) {
        println!(
            "      the well the plant would have published: D_e {:.6} Ha against the true \
             {:.6} Ha — {:+.4} Ha of manufactured depth",
            p.d_e,
            h.d_e,
            p.d_e - h.d_e
        );
    }
}

// ==================================================== the artifact against the stakes

/// The EMITTED artifact was produced under the inputs the gates above verified.
///
/// A results artifact whose instrument's stakes are not its own is the shape `MISFITS.md`
/// registers as `M-STALE-INSTRUMENT`, and it is invisible from either side alone: the
/// gates pass on a table generated in-process, the file passes any schema check, and
/// nothing compares the two. This does.
///
/// A missing file FAILS rather than skips. The bank is a committed receipt, and a skip is
/// a silent pass — which is the whole failure this file exists to refuse.
#[test]
fn the_emitted_bank_was_produced_under_the_staked_inputs() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../docs/atoms/tables/ions");
    let manifest = std::fs::read_to_string(root.join("manifest.json")).unwrap_or_else(|e| {
        panic!(
            "the emitted bank is missing ({e}). Run:\n    cargo run --release -p holon-chem \
             --example emit_ion_tables\nThe bank is a committed receipt; its absence is not \
             a reason to skip this check."
        )
    });
    let curve = std::fs::read_to_string(root.join("H3O_plus.json")).expect("the H3O+ curve");

    let mut checked = 0usize;
    let mut require = |text: &str, needle: String| {
        assert!(
            text.contains(&needle),
            "the emitted bank does not carry {needle} — the artifact was produced under \
             inputs the gates never verified"
        );
        checked += 1;
    };
    // The staked inputs, in both files.
    require(&manifest, format!("\"q_min_bohr\": {Q_MIN:?}"));
    require(&manifest, format!("\"q_max_bohr\": {Q_MAX:?}"));
    require(&manifest, format!("\"n_grid\": {KNOTS}"));
    require(&manifest, "\"charge\": 1".to_string());
    require(&manifest, "\"sz2\": 0".to_string());
    require(&manifest, "\"n_electrons\": 10".to_string());
    require(&manifest, "\"class_z\": [1, 1, 1, 8]".to_string());
    require(&curve, format!("\"n_grid\": {KNOTS}"));
    require(&curve, "\"charge\": 1".to_string());
    require(&curve, "\"sz2\": 0".to_string());
    // And the refusals, which are part of the published bank rather than absences in it.
    require(&manifest, "\"refusal\": \"AnionFenced\"".to_string());
    require(&manifest, format!("\"fence\": \"{FENCE_I5}\""));
    require(&manifest, "\"refusal\": \"PastAutomaticRoute\"".to_string());
    require(&manifest, "\"species\": \"OH-\"".to_string());
    assert_eq!(checked, 14, "this check made {checked} assertions, not the 14 it names");
    println!("  bank: {checked} staked inputs and refusals present in the emitted artifact");
}
