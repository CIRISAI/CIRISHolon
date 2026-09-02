//! RUNG-1 PLANTS — every one checked to FIRE before a real network reading is trusted.
//!
//! `conformance/water_observatory/RUNG1_PREREG.md` §6 names nine plants, each with its
//! carrier and **the sector it must be nonzero in** (M-PLANT-SECTOR). P-9 is the negative
//! control on real data and lives in the runner, because its carrier is eight banked
//! trajectories; the eight here are the ones a fixture can carry.
//!
//! Six of the eight are planted as GEOMETRY, not as key series: the fixture places
//! oxygens and hydrogens, `lens::hbonds` computes the edge set from those positions under
//! the Luzar–Chandler criterion, and the chart is read from that. So the plant exercises
//! the whole chain — positions → edge set → chart → key → defect — rather than only the
//! arithmetic at the end. P-3 is the exception and says so: the gate it aims at is a
//! property of the key series itself.
//!
//! The scene: four oxygens on a square of side 5.5 bohr. That number is chosen against the
//! criterion's own constants and against nothing else — 5.5 < `HB_R_OO_BOHR` = 6.6140 so a
//! side pair CAN carry an edge, while the diagonal 7.78 > 6.6140 so a diagonal pair never
//! can. A hydrogen at 1.8 bohr from its oxygen either points AT the clockwise neighbour
//! (angle 0°, O···H 3.7 bohr — the edge fires) or outward from the square (angle ≥ 110°,
//! O···H ≥ 6.3 bohr — it fails both the angle and the distance clause, so the absence is
//! doubly determined and not balanced on one threshold).

use holon_lens::census::{self, Stakes};
use holon_lens::lens;
use holon_lens::network::{self, Chart, Rung1};
use holon_lens::synthetic::{self, Spec};
use holon_lens::partition::Mask;
use holon_lens::traj::{BondSet, Trajectory};

const SIDE: f64 = 5.5;
const OH: f64 = 1.8;
/// Four oxygens, each followed by its two hydrogens: O at 0, 3, 6, 9.
fn z12() -> Vec<u32> {
    let mut z = Vec::new();
    for _ in 0..4 {
        z.extend_from_slice(&[8, 1, 1]);
    }
    z
}

fn corner(i: usize) -> [f64; 3] {
    match i {
        0 => [0.0, 0.0, 0.0],
        1 => [SIDE, 0.0, 0.0],
        2 => [SIDE, SIDE, 0.0],
        _ => [0.0, SIDE, 0.0],
    }
}

/// Unit vector from the square's centre through corner `i` — "outward".
fn outward(i: usize) -> (f64, f64) {
    let c = corner(i);
    let (dx, dy) = (c[0] - SIDE / 2.0, c[1] - SIDE / 2.0);
    let n = (dx * dx + dy * dy).sqrt();
    (dx / n, dy / n)
}

/// Place the square scene for one edge configuration: bit `i` set means oxygen `i` donates
/// to oxygen `(i+1) % 4`.
fn place_square(cfg: u8, pos: &mut [[f64; 3]]) {
    for i in 0..4 {
        let o = corner(i);
        pos[3 * i] = o;
        let (ox, oy) = outward(i);
        // Hydrogen A: at the neighbour when the bit is set, outward otherwise.
        let (ax, ay) = if cfg >> i & 1 == 1 {
            let nb = corner((i + 1) % 4);
            let (dx, dy) = (nb[0] - o[0], nb[1] - o[1]);
            let n = (dx * dx + dy * dy).sqrt();
            (dx / n, dy / n)
        } else {
            (ox, oy)
        };
        pos[3 * i + 1] = [o[0] + OH * ax, o[1] + OH * ay, 0.0];
        // Hydrogen B: always outward, rotated 25° so the two hydrogens never coincide.
        let th = oy.atan2(ox) + 25.0_f64.to_radians();
        pos[3 * i + 2] = [o[0] + OH * th.cos(), o[1] + OH * th.sin(), 0.0];
    }
}

/// Each oxygen with its own two hydrogens is one block, and no O–O pair is bonded — so
/// every H-bond edge crosses a molecular boundary and the contamination sector is ZERO
/// by construction on every square fixture.
fn square_bonds() -> BondSet {
    synthetic::bonds_from_blocks(12, &[Mask::from_bits(0b000_000_000_111), Mask::from_bits(0b000_000_111_000), Mask::from_bits(0b000_111_000_000), Mask::from_bits(0b111_000_000_000)])
}

fn square_traj(configs: &[u8], n_frames: usize) -> Trajectory {
    let spec = Spec::quench_like(n_frames, z12());
    synthetic::build(spec, |t, pos, vel| {
        place_square(configs[t % configs.len()], pos);
        for v in vel.iter_mut() {
            *v = [0.0; 3];
        }
        square_bonds()
    })
}

/// Configs chosen so all seven give DISTINCT directed edge sets. That is what makes the
/// period-7 walk deterministic on the C1 reading, and it is asserted rather than assumed.
const CYCLE: [u8; 7] = [0b0001, 0b0011, 0b0111, 0b1111, 0b1110, 0b1100, 0b1000];

fn report(t: &Trajectory, surrogates: usize) -> Box<holon_lens::network::Rung1Report> {
    match network::run(t, &Stakes::default(), surrogates) {
        Rung1::Report(r) => r,
        Rung1::Refused { gate, reason } => panic!("unexpected refusal at {gate}: {reason}"),
    }
}

fn chart<'a>(
    r: &'a holon_lens::network::Rung1Report,
    c: Chart,
) -> &'a holon_lens::network::ChartReport {
    r.charts.iter().find(|x| x.chart == c).expect("chart present")
}

/// The fixture's own gate: the geometry must actually produce the edge set the config
/// names, or every plant below is planted in the wrong soil. Checked in BOTH directions —
/// the bit set produces exactly that edge, the bit clear produces none.
#[test]
fn p0_the_fixture_produces_the_edges_it_claims() {
    let z = z12();
    let mut pos = vec![[0.0f64; 3]; 12];

    place_square(0b0000, &mut pos);
    assert!(
        lens::hbonds(&pos, &z).unwrap().is_empty(),
        "with no bit set the square must carry no H-bond at all"
    );

    place_square(0b0001, &mut pos);
    let hb = lens::hbonds(&pos, &z).unwrap();
    assert_eq!(hb.len(), 1, "one bit, one edge; got {hb:?}");
    assert_eq!((hb[0].donor_o, hb[0].acceptor_o), (0, 3), "bit 0 is O0 -> O1 (arena 0 -> 3)");

    place_square(0b1111, &mut pos);
    let hb = lens::hbonds(&pos, &z).unwrap();
    assert_eq!(hb.len(), 4, "four bits, four edges; got {hb:?}");

    // All seven cycle members must read differently on C1, or the walk is not period-7.
    let mut seen = std::collections::HashSet::new();
    for c in CYCLE {
        place_square(c, &mut pos);
        let hb = lens::hbonds(&pos, &z).unwrap();
        assert!(
            seen.insert(network::reading(Chart::HbAdj, 12, &pos, &z, &BondSet::empty(), &hb)),
            "config {c:04b} repeats an earlier C1 reading"
        );
    }
    assert_eq!(seen.len(), 7);
}

/// **P-1 — must read defect EXACTLY 0.** Carrier: the period-7 square walk. Sector: the
/// CHART sector is nonzero (seven distinct readings, changing every frame) while the
/// DEFECT sector is exactly zero by construction. An instrument that cannot read zero
/// here cannot read a defect.
#[test]
fn p1_a_deterministic_cycle_reads_defect_exactly_zero() {
    let t = square_traj(&CYCLE, 1400);
    let r = report(&t, 0);
    for c in [Chart::HbAdj, Chart::HbFull] {
        let cr = chart(&r, c);
        assert_eq!(cr.distinct, 7, "{}: seven configs, seven readings", c.tag());
        assert_eq!(cr.changes, 1399, "{}: the chart must change every frame", c.tag());
        assert!(cr.closure.informative_transitions >= 200, "{}: work count", c.tag());
        assert_eq!(cr.closure.defect, 0.0, "{}: a deterministic cycle has no defect", c.tag());
        assert_eq!(cr.closure.witness_pair_count, 0, "{}: and no witness pair", c.tag());
        assert!(!cr.vacuous, "{}: seven readings changing every frame is not vacuous", c.tag());
    }
}

/// **P-2 — must FIRE.** Carrier: P-1's walk with exactly 13 visits to node 0 diverted to
/// node 3 instead of node 1. Sector: the DEFECT sector, nonzero by construction. The
/// planted count is EXACT, so this plant fails if the instrument over-counts as loudly as
/// if it under-counts.
#[test]
fn p2_thirteen_planted_diversions_are_counted_exactly_thirteen() {
    // The walk, built by hand so the expected violation count is arithmetic and not a
    // second run of the instrument: node i -> i+1, except 13 designated visits to node 0.
    let n_frames = 1400usize;
    let mut nodes: Vec<usize> = Vec::with_capacity(n_frames);
    let mut node = 0usize;
    let mut diverted = 0usize;
    let mut planted: Vec<usize> = Vec::new();
    for f in 0..n_frames {
        nodes.push(node);
        // Divert on the first 13 visits to node 0 that fall at least 40 frames apart, so
        // the plants are separated and none can be masked by another.
        let far_enough = planted.last().map(|&p| f >= p + 40).unwrap_or(true);
        if node == 0 && diverted < 13 && far_enough && f > 0 {
            planted.push(f);
            diverted += 1;
            node = 3;
        } else {
            node = (node + 1) % 7;
        }
    }
    assert_eq!(diverted, 13, "the fixture must plant exactly 13");
    let visits0 = nodes.iter().filter(|&&n| n == 0).count();
    assert!(visits0 - 13 > 13, "the majority successor must stay the majority: {visits0} visits");

    let spec = Spec::quench_like(n_frames, z12());
    let t = synthetic::build(spec, |f, pos, vel| {
        place_square(CYCLE[nodes[f]], pos);
        for v in vel.iter_mut() {
            *v = [0.0; 3];
        }
        square_bonds()
    });
    let r = report(&t, 0);
    let cr = chart(&r, Chart::HbAdj);
    let violations = (cr.closure.defect * cr.closure.informative_transitions as f64).round() as usize;
    assert_eq!(violations, 13, "exactly the planted count, no more and no fewer");
    assert_eq!(cr.closure.witness_pair_count, 1, "one reading acquired one extra successor");
    assert!(!cr.closure.witness_pairs.is_empty(), "and it must be exhibited by index");
}

/// **P-3 — must VOID by work count.** Carrier: a key series in which every frame is a
/// distinct reading. Sector: the READING-MULTIPLICITY sector — zero repeats by
/// construction. **This plant is at the key-series level and not geometric, deliberately:
/// the gate it aims at is a property of the key series, and a four-oxygen square can only
/// spell sixteen readings.** Expected VOID, and specifically not "closed".
#[test]
fn p3_a_never_repeating_reading_voids_on_work_count() {
    let keys: Vec<u64> = (0..1400u64).collect();
    let leg = census::closure_leg(&keys, &Stakes::default());
    assert_eq!(leg.informative_transitions, 0, "no reading is ever revisited");
    assert!(leg.void, "and that is VOID, not a pass");
    assert_eq!(leg.defect, 0.0, "the zero is real arithmetic and must not be read as closure");
}

/// **P-4 — must VOID by dynamism.** Carrier: a square whose H-bond graph never changes.
/// Sector: the CHANGE sector — exactly zero changes by construction, while the chart
/// reads a legal value at every frame. A constant view is Closed by `h = id` and has said
/// nothing; the instrument must call that VOID and name the gate.
#[test]
fn p4_a_frozen_chart_voids_and_is_not_closed() {
    let t = square_traj(&[0b0101], 1400);
    let r = report(&t, 0);
    let cr = chart(&r, Chart::HbAdj);
    assert_eq!(cr.distinct, 1, "one configuration, one reading");
    assert_eq!(cr.changes, 0, "and it never changes");
    assert_eq!(cr.closure.defect, 0.0, "which reads as a zero defect");
    assert!(cr.vacuous, "and the zero must be VOID, not a certification");
    let why = cr.void_reason.as_ref().expect("G-N13: a VOID prints its cause");
    assert!(why.contains("G-N7"), "and the cause names the gate: {why}");
}

/// **P-5 — must VOID by contamination.** Carrier: four oxygens at 2.4 bohr — the O–O
/// curve's own `R_e` — every O–O pair covalently bonded in the engine's bit set, and
/// hydrogens placed so the Luzar–Chandler criterion fires anyway. Sector: the
/// CONTAMINATION sector, nonzero by construction: every edge sits on a covalent pair.
/// This plant is why §3.5's gate is not decoration.
#[test]
fn p5_a_covalent_oxygen_skeleton_reads_fully_contaminated() {
    let r_oo = 2.4_f64; // the O–O curve's R_e, to one decimal
    let spec = Spec::quench_like(1400, z12());
    let t = synthetic::build(spec, |_, pos, vel| {
        for i in 0..4 {
            let (x, y) = match i {
                0 => (0.0, 0.0),
                1 => (r_oo, 0.0),
                2 => (r_oo, r_oo),
                _ => (0.0, r_oo),
            };
            pos[3 * i] = [x, y, 0.0];
            // Hydrogens at 1.0 bohr toward the clockwise neighbour: nearest oxygen is
            // still their own (1.0 against 1.4), and the edge fires at angle 0.
            let nb = match i {
                0 => (r_oo, 0.0),
                1 => (r_oo, r_oo),
                2 => (0.0, r_oo),
                _ => (0.0, 0.0),
            };
            let (dx, dy) = (nb.0 - x, nb.1 - y);
            let n = (dx * dx + dy * dy).sqrt();
            pos[3 * i + 1] = [x + dx / n, y + dy / n, 0.0];
            pos[3 * i + 2] = [x + 0.7 * dx / n, y + 0.7 * dy / n, 0.0];
        }
        for v in vel.iter_mut() {
            *v = [0.0; 3];
        }
        // ONE block: every oxygen covalently bonded to every other.
        synthetic::bonds_from_blocks(12, &[Mask::from_bits(0b111_111_111_111)])
    });
    let r = report(&t, 0);
    assert!(r.hb_records > 0, "the plant must produce edges to contaminate");
    assert_eq!(
        r.contamination, 1.0,
        "every edge sits on a covalently bonded O–O pair: {}/{}",
        r.hb_contaminated, r.hb_records
    );
    assert!(
        r.contamination > network::RUNG1_MAX_CONTAMINATION,
        "and that must exceed the staked ceiling"
    );
    assert_eq!(
        r.frames_with_intermolecular, 0,
        "a single covalent block has no INTER-molecular edge at all"
    );
}

/// **P-6 — must REFUSE.** Carrier: the real hydrogen-arm scene shape, twelve hydrogens and
/// no oxygen. Sector: the SPECIES sector — zero oxygens by construction. Expected a
/// refusal naming the gate, not a pass and not a fail (OBJECT.md rule 9).
#[test]
fn p6_a_scene_with_no_oxygen_is_refused_by_name() {
    let spec = Spec::quench_like(1400, vec![1u32; 12]);
    let t = synthetic::build(spec, |_, _, _| BondSet::empty());
    match network::run(&t, &Stakes::default(), 0) {
        Rung1::Refused { gate, reason } => {
            assert!(gate.contains("G-N12"), "the gate is named: {gate}");
            assert!(reason.contains("oxygen"), "and the cause is named: {reason}");
        }
        Rung1::Report(_) => panic!("a scene with no oxygen must be refused, not reported"),
    }
}

/// The Leg A-N fixture: one H-bonded oxygen pair with a genuinely vibrating separation,
/// and two spectator oxygens far enough away to carry no edge. `held(t)` decides whether
/// the pair is bonded at frame `t`.
fn dimer_traj<H>(n_frames: usize, mut held: H) -> Trajectory
where
    H: FnMut(usize) -> bool,
{
    let spec = Spec::quench_like(n_frames, z12());
    let mut rng = synthetic::Lcg(0xA5A5);
    synthetic::build(spec, move |t, pos, vel| {
        let d = SIDE + rng.jitter(0.5); // O–O separation, 5.0–6.0 bohr
        let (o0, o1) = if held(t) {
            ([0.0, 0.0, 0.0], [d, 0.0, 0.0])
        } else {
            // Broken: O1 swung far off, out of range of every other oxygen.
            ([0.0, 0.0, 0.0], [SIDE, 12.0, 0.0])
        };
        pos[0] = o0;
        pos[1] = [OH, 0.0, 0.0]; // the mediating hydrogen, donor O0 -> acceptor O1
        pos[2] = [-OH, 0.0, 0.0]; // pointing away: angle 180°, no edge
        pos[3] = o1;
        pos[4] = [o1[0] + OH, o1[1], 0.0];
        pos[5] = [o1[0], o1[1] + OH, 0.0];
        // Two spectators, far from everything and from each other.
        for (k, x) in [20.0f64, 28.0].iter().enumerate() {
            let i = 6 + 3 * k;
            pos[i] = [*x, 0.0, 0.0];
            pos[i + 1] = [x + OH, 0.0, 0.0];
            pos[i + 2] = [*x, OH, 0.0];
        }
        for v in vel.iter_mut() {
            *v = [0.0; 3];
        }
        square_bonds()
    })
}

/// **P-7 — must CERTIFY.** Carrier: a two-oxygen H-bond cluster present in every frame,
/// with an O–O separation that genuinely vibrates. Sector: the MEMBERSHIP sector —
/// `v_C ≡ 1` by construction. A Leg A-N that cannot certify this cannot certify anything.
#[test]
fn p7_a_permanently_held_hbond_pair_certifies_strict() {
    let t = dimer_traj(1400, |_| true);
    let r = report(&t, 8);
    let s = r
        .structures
        .iter()
        .find(|s| s.structure == Mask::from_bits(0b0000_0000_1001))
        .expect("the {O0, O1} structure must be a candidate");
    assert_eq!(s.size, 2);
    assert!(
        matches!(s.verdict, network::StructVerdict::CertifiedStrict { .. }),
        "expected CERTIFIED-STRICT, got {} (held {:.1} fs against {:.1})",
        s.verdict.tag(),
        s.longest_run_fs,
        r.window_fs
    );
    assert!(s.rms_internal >= 0.1, "the carrier must move: rms {}", s.rms_internal);
    assert!(s.max_sep_variation >= 0.05, "and vibrate: {}", s.max_sep_variation);
}

/// **P-8 — must REJECT (budget abuse).** Carrier: P-7 with ONE breach run of eleven
/// frames — 9.17 fs, past the 8.4 fs flicker clause — placed mid-window, with a total
/// breach fraction of 1.1%, comfortably inside the 2% budget. Sector: MEMBERSHIP; the
/// breach run IS the plant. This plant exists solely to prove the budget is not an escape
/// hatch, and it must fail BOTH clauses.
#[test]
fn p8_one_over_long_breach_fails_despite_passing_the_two_percent() {
    let t = dimer_traj(1400, |f| !(600..611).contains(&f));
    let r = report(&t, 8);
    let s = r
        .structures
        .iter()
        .find(|s| s.structure == Mask::from_bits(0b0000_0000_1001))
        .expect("the structure is still a candidate, it just does not certify");
    assert!(
        matches!(s.verdict, network::StructVerdict::Transient { .. }),
        "an 11-frame breach is 9.17 fs against a staked 8.4 fs flicker clause, so both \
         the strict and the budgeted reading must fail; got {}",
        s.verdict.tag()
    );
    // And the reason must be the flicker clause rather than the fraction: the longest
    // unbroken run is 789 frames and the window needs 1002, while the 11 breached frames
    // are 1.1% of a window's 1002 — inside the 2% budget. If the fraction had been the
    // binding clause this assertion would be the one that fails.
    assert!(
        s.longest_run < 1002,
        "no unbroken window exists either: longest run {} frames",
        s.longest_run
    );
}

/// The control floor must be DETERMINISTIC from the trajectory's seed, or it is a
/// different number every time it is quoted. Checked by running it twice.
///
/// It also pins the DEGENERACY this campaign found on its own positive control before any
/// real data: the fixture's single H-bond edge is present in 1400 of 1400 frames, and a
/// circular shift of an all-true series is that series. So the surrogate IS the data, the
/// null has no degrees of freedom, and `max_edge_occupancy` is the field that says so.
#[test]
fn the_control_floor_reproduces_and_reports_its_own_power() {
    let t = dimer_traj(1400, |_| true);
    let hbs: Vec<Vec<lens::HBond>> = t
        .frames
        .iter()
        .map(|f| lens::hbonds(&f.pos, &t.header.z).unwrap())
        .collect();
    let times: Vec<f64> = t
        .frames
        .iter()
        .map(|f| f.time * holon_lens::traj::AU_TIME_FS)
        .collect();
    let st = Stakes::default();
    let ox = network::oxygens(&t.header.z);
    let target = network::ox_positions_of(&ox, &Mask::from_bits(0b0000_0000_1001));
    let a = network::control_floor(&t, &hbs, &times, &target, &st, 16);
    let b = network::control_floor(&t, &hbs, &times, &target, &st, 16);
    assert_eq!(a.p_data, b.p_data, "the same seed must give the same pool rate");
    assert_eq!(a.p_null_p95, b.p_null_p95, "and the same surrogate percentile");
    assert_eq!(
        a.max_edge_occupancy, 1.0,
        "the fixture's edge is permanent, so the shift cannot move it"
    );
    assert_eq!(
        a.q_any, 1.0,
        "which is why the any-structure diagnostic reads 1.0 and cannot discriminate"
    );
    assert_eq!(
        a.p_data, 0.0,
        "while the POOL rate — the census's own statistic — separates cleanly: none of \
         the five other oxygen pairs ever forms"
    );
}

/// G-N2's own shape, on a fixture: the charts really are nested, so the coarser one can
/// never have FEWER collisions. `refinement_removes_collisions` forces it, and a
/// violation would convict the instrument rather than the physics.
#[test]
fn g_n2_the_ladder_is_monotone_on_a_fixture() {
    let t = square_traj(&CYCLE, 1400);
    let r = report(&t, 0);
    let full = chart(&r, Chart::HbFull).collisions;
    let adj = chart(&r, Chart::HbAdj).collisions;
    let part = chart(&r, Chart::HbPart).collisions;
    assert!(full <= adj, "C1H refines C1: {full} <= {adj}");
    assert!(adj <= part, "C1 refines C2: {adj} <= {part}");
    let id = chart(&r, Chart::MolNetId).collisions;
    let form = chart(&r, Chart::MolNetFormula).collisions;
    assert!(id <= form, "C4 refines C5: {id} <= {form}");
}

/// Leg F's own zero and its own one, on a fixture. A chart factors through itself
/// exactly; and a chart that is a strict function of geometry the base chart discarded
/// must exhibit witnesses against a base that never moves.
#[test]
fn leg_f_reads_zero_on_itself_and_fires_on_a_geometric_chart() {
    let t = square_traj(&CYCLE, 1400);
    let r = report(&t, 0);
    // C6 is constant on this fixture (the molecular partition never changes), so every
    // frame is one fiber and the H-bond chart's seven readings must all be witnesses.
    let mol = chart(&r, Chart::MolPart);
    assert_eq!(mol.distinct, 1, "the fixture's molecular partition is constant");
    let adj = chart(&r, Chart::HbAdj);
    assert!(
        adj.factor.witness_count > 0,
        "a geometric chart cannot factor through a constant molecular chart"
    );
    assert_eq!(
        mol.factor.witness_count, 0,
        "while the molecular chart factors through itself exactly"
    );
    assert_eq!(mol.factor.defect, 0.0);
}
