//! RUNG 1 — THE H-BOND NETWORK TIER: is the network a Closed view of the molecular tier?
//!
//! Frozen in `conformance/water_observatory/RUNG1_PREREG.md` before this file existed.
//! Every threshold below is either a `PREREG_*` constant INHERITED from
//! [`crate::census`] — the census is the referee and its window is not re-chosen at the
//! tier above — or a `RUNG1_*` constant this campaign staked itself and declared as a
//! judgment in the freeze.
//!
//! Three legs, because the rung carries three different claims and the programme has run
//! them together before:
//!
//! * **Leg A-N, HELD.** For a structure `C` of oxygen arena indices, `v_C(x) = 1` iff `C`
//!   is EXACTLY a component of the undirected H-bond graph. `Held v_C T` over a window.
//!   By `held_imp_closed` a Held view is a Closed one, so this is the only leg that can
//!   return a positive with a theorem behind it — and it is the same leg, unchanged, that
//!   certified the molecular tier.
//! * **Leg B-N, CLOSED.** Fiber-invariance of a network chart under one grain boundary,
//!   via [`crate::census::closure_leg`] — literally the census's own function, so a
//!   defect measured here and a defect in `CENSUS_RESULTS.md` are the same arithmetic.
//! * **Leg F, FACTORS.** Does the network chart factor through the MOLECULAR chart? This
//!   leg exists because `closed_comp` — the theorem that makes a tower a tower — requires
//!   the upper chart to be a function of the lower one, and the H-bond criterion is
//!   GEOMETRIC: it reads angles the bonded partition has already thrown away. A pair of
//!   frames agreeing on the molecular chart and disagreeing on the network chart is a
//!   factorisation witness, exhibited by index.
//!
//! ## The edge set is the LENS's, and that is a declared difference from the census
//!
//! The census's rule 1 was "the edge set is the ENGINE's" — its bits came from
//! `Sim::refresh_pairs` and nothing recomputed them. The H-bond edge set has no engine
//! counterpart, so it is computed by [`crate::lens::hbonds`], called UNMODIFIED. No
//! second implementation of it exists in this module, and the one place a network chart
//! touches the engine's own bits — the molecular partition, and the covalent-contamination
//! test — reads them from the dump exactly as the census does.
//!
//! ## Why contamination is a gate and not a footnote
//!
//! In liquid water there are no covalent O–O bonds, so every Luzar–Chandler edge is
//! between distinct molecules by construction. In THESE scenes the O–O curve has
//! `R_e = 2.4421 bohr` and oxygens aggregate into covalently bonded clusters whose O–O
//! separations sit well within the criterion's `6.6140 bohr` O···O cut. So the criterion
//! CAN fire on a covalently bonded pair, and a chart built from those edges would be a
//! relabelled covalent skeleton wearing a hydrogen bond's name. That is a lens that does
//! not contain the variable it claims to measure, and the freeze gates it at 0.20.

use crate::census::{self, Stakes};
use crate::lens;
use crate::partition::{self, Mask};
use crate::traj::{pair_index, Trajectory, AU_TIME_FS};
use std::collections::HashMap;

// ======================================================= THE STAKES, from RUNG1_PREREG

/// The closure budget: at most 1% of informative transitions may violate functionality.
///
/// A JUDGMENT, not a derivation, and the freeze says so. `holon-render/src/holon.rs`
/// stakes `CLOSURE_DEFECT_MAX = 1e-2` as the engine's own bar for calling a composite
/// view autonomous; that constant is in energy-fraction units and this one is in
/// functional units, and transposing it is an act of judgment made before any data.
pub const RUNG1_DELTA_STAR: f64 = 0.01;

/// At most one H-bond edge in five may sit on a covalently bonded oxygen pair before the
/// chart is refused as not containing its own variable.
pub const RUNG1_MAX_CONTAMINATION: f64 = 0.20;

/// Anti-vacuity, second clause: the chart must actually CHANGE. A constant view is Closed
/// by `h = id` and has said nothing — the census's own H-bond-count row read defect 0.000
/// on the hydrogen arm and was labelled VACUOUS, and this constant is that finding turned
/// into a gate. Matches the work count's own number rather than inventing a second.
pub const RUNG1_MIN_CHANGES: usize = 200;

/// Anti-vacuity, third clause: the smallest reading count at which a chart is not a coin
/// flip. A judgment, declared.
pub const RUNG1_MIN_DISTINCT: usize = 4;

/// Surrogates for the shuffle control floor. `CENSUS_RESULTS.md` §4 convicted its own
/// flat 5% pool rate and staked this repair for "the next freeze"; this is that freeze.
pub const RUNG1_SURROGATES: usize = 200;

/// The surrogate floor's own bar, inherited from the census's control ceiling so the two
/// floors are read on one scale.
pub const RUNG1_MAX_Q: f64 = 0.05;

/// The RNG salt, so the shuffle floor reproduces exactly from the trajectory's seed.
pub const RUNG1_RNG_SALT: u64 = 0x52554E473100;

// ======================================================= the charts

/// The seven charts, fixed by the freeze's declared population rule. No chart is added
/// after a reading is taken; a chart invented later is a new prereg, not a rescue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chart {
    /// C1H — the `(donor_o, hydrogen, acceptor_o)` triples, sorted.
    HbFull,
    /// C1 — C1H with the mediating hydrogen projected out.
    HbAdj,
    /// C2 — the partition of oxygens under the undirected H-bond graph.
    HbPart,
    /// C3 — the number of H-bond records, counted per mediating hydrogen.
    HbCount,
    /// C4 — the network of molecules, arena identity kept.
    MolNetId,
    /// C5 — C4 with arena identity forgotten: formulas only.
    MolNetFormula,
    /// C6 — the molecular chart itself. The tier BELOW; reference and negative control.
    MolPart,
}

impl Chart {
    pub const ALL: [Chart; 7] = [
        Chart::HbFull,
        Chart::HbAdj,
        Chart::HbPart,
        Chart::HbCount,
        Chart::MolNetId,
        Chart::MolNetFormula,
        Chart::MolPart,
    ];
    /// The candidate network charts — everything except the tier below.
    pub const CANDIDATES: [Chart; 6] = [
        Chart::HbFull,
        Chart::HbAdj,
        Chart::HbPart,
        Chart::HbCount,
        Chart::MolNetId,
        Chart::MolNetFormula,
    ];
    pub fn tag(&self) -> &'static str {
        match self {
            Chart::HbFull => "C1H/HB-FULL",
            Chart::HbAdj => "C1/HB-ADJ",
            Chart::HbPart => "C2/HB-PART",
            Chart::HbCount => "C3/HB-COUNT",
            Chart::MolNetId => "C4/MOL-NET-ID",
            Chart::MolNetFormula => "C5/MOL-NET-FORMULA",
            Chart::MolPart => "C6/MOL-PART",
        }
    }
}

/// The canonical byte encoding of one chart reading at one frame.
///
/// Bytes rather than a hash, so two readings are equal exactly when the freeze says they
/// are. The `u64` keys the closure engine compares are dense ids assigned by [`Keyer`],
/// which is injective BY CONSTRUCTION — no hash collision can merge two distinct readings
/// and manufacture a witness pair (G-N0).
pub fn reading(
    chart: Chart,
    n: usize,
    // Positions are not read by any chart in the frozen set: every one of the seven is a
    // function of the EDGE SETS (the lens's H-bond records and the engine's bond bits).
    // The parameter stays because a chart added by a future prereg would need it, and
    // removing it would make that a signature change rather than a body change.
    _pos: &[[f64; 3]],
    z: &[u32],
    bonded: u128,
    hb: &[lens::HBond],
) -> Vec<u8> {
    match chart {
        Chart::HbFull => {
            let mut v: Vec<(u8, u8, u8)> = hb
                .iter()
                .map(|b| (b.donor_o as u8, b.hydrogen as u8, b.acceptor_o as u8))
                .collect();
            v.sort_unstable();
            v.dedup();
            v.into_iter().flat_map(|(a, b, c)| [a, b, c]).collect()
        }
        Chart::HbAdj => {
            let mut v: Vec<(u8, u8)> = hb
                .iter()
                .map(|b| (b.donor_o as u8, b.acceptor_o as u8))
                .collect();
            v.sort_unstable();
            v.dedup();
            v.into_iter().flat_map(|(a, b)| [a, b]).collect()
        }
        Chart::HbPart => hb_oxygen_labels(z, hb),
        Chart::HbCount => {
            let mut v: Vec<(u8, u8, u8)> = hb
                .iter()
                .map(|b| (b.donor_o as u8, b.hydrogen as u8, b.acceptor_o as u8))
                .collect();
            v.sort_unstable();
            v.dedup();
            (v.len() as u32).to_le_bytes().to_vec()
        }
        Chart::MolNetId => {
            let labels = partition::labels_from_bonds(n, bonded);
            let mut out: Vec<u8> = Vec::new();
            for m in partition::blocks(&labels) {
                out.extend_from_slice(&m.to_le_bytes());
            }
            out.push(0xFF);
            for (a, b) in inter_block_pairs(&labels, hb) {
                out.push(a as u8);
                out.push(b as u8);
            }
            out
        }
        Chart::MolNetFormula => {
            let labels = partition::labels_from_bonds(n, bonded);
            let mut forms: Vec<String> = partition::blocks(&labels)
                .into_iter()
                .map(|m| partition::formula(m, z))
                .collect();
            forms.sort();
            let mut pairs: Vec<(String, String)> = inter_block_pairs(&labels, hb)
                .into_iter()
                .map(|(a, b)| {
                    let fa = partition::formula(block_of(&labels, a), z);
                    let fb = partition::formula(block_of(&labels, b), z);
                    if fa <= fb {
                        (fa, fb)
                    } else {
                        (fb, fa)
                    }
                })
                .collect();
            pairs.sort();
            let mut s = forms.join(";");
            s.push('|');
            s.push_str(
                &pairs
                    .into_iter()
                    .map(|(a, b)| format!("{a}-{b}"))
                    .collect::<Vec<_>>()
                    .join(";"),
            );
            s.into_bytes()
        }
        Chart::MolPart => partition::key(&partition::labels_from_bonds(n, bonded))
            .to_le_bytes()
            .to_vec(),
    }
}

/// The oxygen arena indices, ascending. The chart's index space.
pub fn oxygens(z: &[u32]) -> Vec<usize> {
    (0..z.len()).filter(|&i| z[i] == 8).collect()
}

/// The undirected H-bond graph over the oxygen sub-index space, as pair bits in the
/// engine's own `pair_index` enumeration.
pub fn hb_oxygen_bits(z: &[u32], hb: &[lens::HBond]) -> (Vec<usize>, u128) {
    let ox = oxygens(z);
    let m = ox.len();
    let mut bits = 0u128;
    for b in hb {
        let (Some(p), Some(q)) = (
            ox.iter().position(|&i| i == b.donor_o),
            ox.iter().position(|&i| i == b.acceptor_o),
        ) else {
            continue;
        };
        if p == q {
            continue;
        }
        let (lo, hi) = if p < q { (p, q) } else { (q, p) };
        bits |= 1u128 << pair_index(m, lo, hi);
    }
    (ox, bits)
}

fn hb_oxygen_labels(z: &[u32], hb: &[lens::HBond]) -> Vec<u8> {
    let (ox, bits) = hb_oxygen_bits(z, hb);
    partition::labels_from_bonds(ox.len(), bits)
}

/// The components of the undirected H-bond graph, as masks over ARENA indices. Only
/// structures of two or more oxygens: a lone oxygen is not a network.
pub fn hb_components(z: &[u32], hb: &[lens::HBond]) -> Vec<Mask> {
    let (ox, bits) = hb_oxygen_bits(z, hb);
    let labels = partition::labels_from_bonds(ox.len(), bits);
    partition::blocks(&labels)
        .into_iter()
        .filter(|m| partition::popcount(*m) >= 2)
        .map(|m| {
            let mut arena: Mask = 0;
            for (p, &i) in ox.iter().enumerate() {
                if m >> p & 1 == 1 {
                    arena |= 1 << i;
                }
            }
            arena
        })
        .collect()
}

/// An ARENA mask over oxygens, re-expressed in the oxygen POSITION index space the floor
/// and the surrogate work in. Arena indices stay the identity (Object rule 6); positions
/// are only the compact coordinates the pool enumeration runs over.
pub fn ox_positions_of(ox: &[usize], arena: Mask) -> u16 {
    let mut m = 0u16;
    for (p, &i) in ox.iter().enumerate() {
        if arena >> i & 1 == 1 {
            m |= 1 << p;
        }
    }
    m
}

fn block_of(labels: &[u8], i: usize) -> Mask {
    let lead = labels[i];
    let mut m: Mask = 0;
    for (k, &l) in labels.iter().enumerate() {
        if l == lead {
            m |= 1 << k;
        }
    }
    m
}

/// Unordered oxygen pairs that are H-bonded AND lie in different blocks of the molecular
/// partition — the INTER-MOLECULAR edges, which are the only edges a network of molecules
/// has. An H-bond between two atoms of the same molecule is not an edge of that network.
fn inter_block_pairs(labels: &[u8], hb: &[lens::HBond]) -> Vec<(usize, usize)> {
    let mut v: Vec<(usize, usize)> = hb
        .iter()
        .filter(|b| labels[b.donor_o] != labels[b.acceptor_o])
        .map(|b| {
            if b.donor_o < b.acceptor_o {
                (b.donor_o, b.acceptor_o)
            } else {
                (b.acceptor_o, b.donor_o)
            }
        })
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// Dense sequential ids in order of first appearance. Injective by construction (G-N0):
/// distinct readings never share a key, so no witness pair can be an artefact of hashing.
/// The id VALUES carry no meaning; the closure test uses only equality.
#[derive(Default)]
pub struct Keyer {
    map: HashMap<Vec<u8>, u64>,
}

impl Keyer {
    pub fn key(&mut self, r: Vec<u8>) -> u64 {
        let next = self.map.len() as u64;
        *self.map.entry(r).or_insert(next)
    }
    pub fn distinct(&self) -> usize {
        self.map.len()
    }
}

// ======================================================= Leg F — factorisation

/// Does the fine reading factor through the base reading?
///
/// Same normal form as the closure test, with the successor replaced by the same-frame
/// image: group frames by `base`, and count the frames whose `fine` reading is not the
/// group's majority. A group whose members disagree carries a FACTORISATION WITNESS, and
/// `closed_comp` does not apply to a chart that has one — the tower does not stack
/// through it.
///
/// The arithmetic deliberately mirrors [`census::closure_leg`]: only groups of size ≥ 2
/// are informative, and the defect is non-majority over informative.
#[derive(Clone, Debug)]
pub struct FactorLeg {
    pub informative: usize,
    pub violations: usize,
    pub defect: f64,
    pub witnesses: Vec<(usize, usize)>,
    pub witness_count: usize,
    pub void: bool,
}

pub fn factor_leg(base: &[u64], fine: &[u64], min_informative: usize) -> FactorLeg {
    let mut groups: HashMap<u64, HashMap<u64, Vec<usize>>> = HashMap::new();
    for (t, (&b, &f)) in base.iter().zip(fine.iter()).enumerate() {
        groups.entry(b).or_default().entry(f).or_default().push(t);
    }
    let (mut informative, mut violations, mut witness_count) = (0usize, 0usize, 0usize);
    let mut witnesses: Vec<(usize, usize)> = Vec::new();
    for outs in groups.values() {
        let m: usize = outs.values().map(|v| v.len()).sum();
        if m < 2 {
            continue;
        }
        informative += m;
        let top = outs.values().map(|v| v.len()).max().unwrap_or(0);
        violations += m - top;
        if outs.len() > 1 {
            witness_count += outs.len() - 1;
            if witnesses.len() < 16 {
                let mut br: Vec<usize> = outs.values().map(|v| v[0]).collect();
                br.sort_unstable();
                witnesses.push((br[0], br[1]));
            }
        }
    }
    FactorLeg {
        informative,
        violations,
        defect: if informative == 0 {
            0.0
        } else {
            violations as f64 / informative as f64
        },
        witnesses,
        witness_count,
        void: informative < min_informative,
    }
}

// ======================================================= the shuffle control floor

/// splitmix64 — a deterministic stream from a stated seed, so the floor reproduces.
pub struct SplitMix(pub u64);

impl SplitMix {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
}

/// Per-directed-oxygen-pair boolean occupancy, one series per ordered pair `(p, q)` of
/// oxygen POSITIONS. The substrate the surrogate shuffles.
pub fn directed_edge_series(traj: &Trajectory, hbs: &[Vec<lens::HBond>]) -> (Vec<usize>, Vec<Vec<bool>>) {
    let ox = oxygens(&traj.header.z);
    let m = ox.len();
    let nf = hbs.len();
    let mut series = vec![vec![false; nf]; m * m];
    for (t, hb) in hbs.iter().enumerate() {
        for b in hb {
            let (Some(p), Some(q)) = (
                ox.iter().position(|&i| i == b.donor_o),
                ox.iter().position(|&i| i == b.acceptor_o),
            ) else {
                continue;
            };
            if p != q {
                series[p * m + q][t] = true;
            }
        }
    }
    (ox, series)
}

/// Components of size `k` from a shuffled directed-edge state, as position masks.
fn components_from_edges(m: usize, present: &[bool]) -> Vec<u16> {
    let mut bits = 0u128;
    for p in 0..m {
        for q in 0..m {
            if p != q && present[p * m + q] {
                let (lo, hi) = if p < q { (p, q) } else { (q, p) };
                bits |= 1u128 << pair_index(m, lo, hi);
            }
        }
    }
    let labels = partition::labels_from_bonds(m, bits);
    partition::blocks(&labels)
        .into_iter()
        .filter(|x| partition::popcount(*x) >= 2)
        .collect()
}

/// The eligible pool for a structure of `size` oxygens: EVERY `size`-subset of the oxygen
/// position set. An exact enumeration, not a sample — at four oxygens the pool is 6, 4 and
/// 1 for sizes 2, 3 and 4.
pub fn pool_subsets(m: usize, size: usize) -> Vec<u16> {
    (1u32..(1u32 << m))
        .map(|c| c as u16)
        .filter(|c| partition::popcount(*c) == size)
        .collect()
}

/// What the control floor measured, with every term kept so a reader can see WHICH
/// statistic bound the verdict (G-N13).
#[derive(Clone, Copy, Debug)]
pub struct Floor {
    /// The pool pass rate in the DATA: same-size structures other than the target that
    /// also reach the window. The census's own statistic, exactly enumerated.
    pub p_data: f64,
    /// The 95th percentile of that same pool rate over the surrogates. This is the
    /// census's staked repair: a flat ceiling is replaced by what the scene's own edge
    /// marginals produce by chance.
    pub p_null_p95: f64,
    /// The bar actually applied: `max(p_null_p95, RUNG1_MAX_Q)`. The staked 0.05 is kept
    /// as a FLOOR on the bar, never as a ceiling that a chemically busier scene fails
    /// merely for being busier — which is precisely what CENSUS_RESULTS.md §4 convicted.
    pub bar: f64,
    /// DIAGNOSTIC, not binding: the fraction of surrogates in which ANY same-size
    /// structure reaches the window.
    pub q_any: f64,
    /// DIAGNOSTIC, not binding: the largest per-edge occupancy among the target's edges.
    /// **This is the number that says whether the surrogate had any power at all.** A
    /// circular shift of an all-true series is that series, so as occupancy approaches 1
    /// the null loses its degrees of freedom and stops being able to reject anything.
    pub max_edge_occupancy: f64,
    pub pool: usize,
}

/// **The control floor** — `CENSUS_RESULTS.md` §4's staked repair, built here.
///
/// The census convicted its own floor: a flat 5% ceiling is satisfiable only by a scene
/// holding at most four structures of the composition under test, so it gave H₂ opposite
/// verdicts in two arms of identical physics. Its staked repair, quoted in the freeze, was
/// to compare *"the pool pass rate ... against the pass rate among same-composition blocks
/// in a surrogate whose bond graph is time-shuffled within each pair"* — and that is what
/// this computes.
///
/// Each directed oxygen-pair edge series is independently rotated by a uniform circular
/// shift, which preserves that edge's marginal occupancy and its own run-length
/// distribution EXACTLY while destroying the inter-edge coincidence that makes a
/// particular oxygen set a persistent cluster. The pool rate is recomputed on the shifted
/// edges, and its 95th percentile becomes the bar — raised above the flat 0.05 where the
/// scene's own marginals would produce peer passes by chance, never lowered below it.
///
/// **The surrogate's power is reported, not assumed** (`max_edge_occupancy`). A shift
/// cannot move an all-true series, so a structure whose edges are permanently occupied has
/// a null with no degrees of freedom. That is a fact about the null and it is printed
/// beside every reading rather than left for a reader to infer from a verdict.
pub fn control_floor(
    traj: &Trajectory,
    hbs: &[Vec<lens::HBond>],
    times: &[f64],
    target: u16,
    st: &Stakes,
    surrogates: usize,
) -> Floor {
    let (ox, series) = directed_edge_series(traj, hbs);
    let m = ox.len();
    let nf = times.len();
    let size = partition::popcount(target);
    let subsets = pool_subsets(m, size);
    let ti = subsets.iter().position(|&s| s == target);

    // ---- the target's own edge occupancy, the null's power diagnostic ----------------
    let mut max_occ = 0.0f64;
    for p in 0..m {
        for q in 0..m {
            if p == q || target >> p & 1 == 0 || target >> q & 1 == 0 {
                continue;
            }
            let s = &series[p * m + q];
            let occ = s.iter().filter(|x| **x).count() as f64 / nf as f64;
            if occ > max_occ {
                max_occ = occ;
            }
        }
    }

    // ---- the pool rate in the DATA ---------------------------------------------------
    let comps: Vec<Vec<u16>> = hbs
        .iter()
        .map(|hb| {
            let (_, bits) = hb_oxygen_bits(&traj.header.z, hb);
            partition::blocks(&partition::labels_from_bonds(m, bits))
                .into_iter()
                .filter(|c| partition::popcount(*c) >= 2)
                .collect()
        })
        .collect();
    let data_pass: Vec<bool> = subsets
        .iter()
        .map(|&c| {
            let s: Vec<bool> = comps.iter().map(|cs| cs.contains(&c)).collect();
            reaches_window(&s, times, st)
        })
        .collect();
    let total_data = data_pass.iter().filter(|x| **x).count();
    let pool = subsets.len().saturating_sub(1);
    let p_data = if pool == 0 {
        0.0
    } else {
        let own = ti.map(|i| data_pass[i] as usize).unwrap_or(0);
        (total_data - own) as f64 / pool as f64
    };

    if surrogates == 0 || pool == 0 || m < 2 || nf < 2 {
        let bar = RUNG1_MAX_Q;
        return Floor {
            p_data,
            p_null_p95: 0.0,
            bar,
            q_any: 0.0,
            max_edge_occupancy: max_occ,
            pool,
        };
    }

    // ---- the same rate under the surrogate -------------------------------------------
    let mut rng = SplitMix(traj.header.seed ^ RUNG1_RNG_SALT);
    let mut rates: Vec<f64> = Vec::with_capacity(surrogates);
    let mut any_hits = 0usize;
    let mut present = vec![false; m * m];
    for _ in 0..surrogates {
        let shifts: Vec<usize> = (0..m * m)
            .map(|_| (rng.next_u64() % (nf as u64 - 1)) as usize + 1)
            .collect();
        let mut occ: Vec<Vec<bool>> = vec![vec![false; nf]; subsets.len()];
        for t in 0..nf {
            for e in 0..m * m {
                present[e] = series[e][(t + shifts[e]) % nf];
            }
            for c in components_from_edges(m, &present) {
                if let Some(i) = subsets.iter().position(|&s| s == c) {
                    occ[i][t] = true;
                }
            }
        }
        let pass: Vec<bool> = occ.iter().map(|s| reaches_window(s, times, st)).collect();
        let total: usize = pass.iter().filter(|x| **x).count();
        if total > 0 {
            any_hits += 1;
        }
        let own = ti.map(|i| pass[i] as usize).unwrap_or(0);
        rates.push((total - own) as f64 / pool as f64);
    }
    rates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    // The 95th percentile, taken as the ceiling-indexed order statistic so a small
    // surrogate count cannot silently read as the maximum.
    let idx = (((surrogates as f64) * 0.95).ceil() as usize).min(surrogates) - 1;
    let p_null_p95 = rates[idx];
    Floor {
        p_data,
        p_null_p95,
        bar: p_null_p95.max(RUNG1_MAX_Q),
        q_any: any_hits as f64 / surrogates as f64,
        max_edge_occupancy: max_occ,
        pool,
    }
}

/// Strict-or-budgeted window, with the census's own cheap exact pre-filter in front of
/// the expensive budgeted scan. Same two functions the molecular tier was certified with.
pub fn reaches_window(s: &[bool], times: &[f64], st: &Stakes) -> bool {
    if census::strict_window(s, times, st.window_fs).is_some() {
        return true;
    }
    let nf = times.len();
    let held: f64 = s
        .iter()
        .enumerate()
        .filter(|(_, v)| **v)
        .map(|(i, _)| {
            if i + 1 < nf {
                times[i + 1] - times[i]
            } else {
                0.0
            }
        })
        .sum();
    if held < (1.0 - st.beta) * st.window_fs {
        return false;
    }
    census::budgeted_window(s, times, st.window_fs, st.beta, st.flicker_fs).is_some()
}

// ======================================================= the report

#[derive(Clone, Debug)]
pub struct ChartReport {
    pub chart: Chart,
    pub distinct: usize,
    pub changes: usize,
    pub closure: census::ClosureLeg,
    pub factor: FactorLeg,
    /// Collisions — informative transitions — used by G-N2's monotonicity check.
    pub collisions: usize,
    pub vacuous: bool,
    /// The gate that produced a VOID or REFUSAL, always printed (G-N13).
    pub void_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StructVerdict {
    CertifiedStrict { window_start: usize },
    CertifiedBudgeted { window_start: usize, breach_frames: usize, max_breach_fs: f64 },
    Transient { longest_run: usize },
    VoidFrozenCarrier { rms: f64, sep_var: f64 },
    VoidNoSeparation { p_data: f64, bar: f64 },
}

impl StructVerdict {
    pub fn tag(&self) -> &'static str {
        match self {
            StructVerdict::CertifiedStrict { .. } => "CERTIFIED-STRICT",
            StructVerdict::CertifiedBudgeted { .. } => "CERTIFIED-BUDGETED",
            StructVerdict::Transient { .. } => "TRANSIENT",
            StructVerdict::VoidFrozenCarrier { .. } => "VOID(frozen carrier)",
            StructVerdict::VoidNoSeparation { .. } => "VOID(no separation)",
        }
    }
}

#[derive(Clone, Debug)]
pub struct StructReport {
    pub structure: Mask,
    pub size: usize,
    pub frames_present: usize,
    pub longest_run: usize,
    pub longest_run_fs: f64,
    pub verdict: StructVerdict,
    pub rms_internal: f64,
    pub max_sep_variation: f64,
    /// The control floor, with every term kept: the data's pool rate, the surrogate's
    /// 95th percentile, the bar actually applied, and the two diagnostics that say
    /// whether the surrogate had any power (G-N13).
    pub floor: Option<Floor>,
}

#[derive(Clone, Debug)]
pub enum Rung1 {
    Refused { gate: &'static str, reason: String },
    Report(Box<Rung1Report>),
}

#[derive(Clone, Debug)]
pub struct Rung1Report {
    pub seed: u64,
    pub n_atoms: usize,
    pub n_oxygens: usize,
    pub n_frames: usize,
    pub span_fs: f64,
    pub window_fs: f64,
    /// Σ contaminated records / Σ records, with both terms kept.
    pub contamination: f64,
    pub hb_records: usize,
    pub hb_contaminated: usize,
    /// Frames carrying at least one INTER-MOLECULAR H-bond — branch (E)'s own quantity.
    pub frames_with_intermolecular: usize,
    pub charts: Vec<ChartReport>,
    pub structures: Vec<StructReport>,
}

// ======================================================= the run

pub fn run(traj: &Trajectory, st: &Stakes, surrogates: usize) -> Rung1 {
    let n = traj.header.n_atoms;
    let nf = traj.frames.len();
    let z = &traj.header.z;
    let ox = oxygens(z);

    if ox.len() < 2 {
        return Rung1::Refused {
            gate: "G-N12 scene carries the network variable",
            reason: format!(
                "{} oxygens; a hydrogen-bond NETWORK needs at least two, and a zero here \
                 would read as a measured absence rather than an inapplicable lens. The \
                 gate whose passing would lift this refusal is a scene with 2+ oxygens.",
                ox.len()
            ),
        };
    }
    if nf < 2 {
        return Rung1::Refused {
            gate: "at least 2 frames",
            reason: format!("{nf} frames; no window and no motion exists"),
        };
    }

    let times: Vec<f64> = traj.frames.iter().map(|f| f.time * AU_TIME_FS).collect();
    let span_fs = times[nf - 1] - times[0];
    if span_fs < st.window_fs {
        return Rung1::Refused {
            gate: "G-N8/G-N9 window length",
            reason: format!(
                "the trajectory spans {span_fs:.1} fs and the staked window is {:.1} fs; \
                 no held reading is possible.",
                st.window_fs
            ),
        };
    }

    // ---- the H-bond edge set, once, from the committed lens ------------------------
    let hbs: Vec<Vec<lens::HBond>> = traj
        .frames
        .iter()
        .map(|f| lens::hbonds(&f.pos, z).expect("oxygen and hydrogen both present"))
        .collect();

    // ---- contamination: does the criterion contain its own variable? ---------------
    let (mut records, mut contaminated) = (0usize, 0usize);
    let mut frames_with_inter = 0usize;
    for (t, hb) in hbs.iter().enumerate() {
        let f = &traj.frames[t];
        let labels = partition::labels_from_bonds(n, f.bonded);
        let mut seen: Vec<(usize, usize, usize)> = hb
            .iter()
            .map(|b| (b.donor_o, b.hydrogen, b.acceptor_o))
            .collect();
        seen.sort_unstable();
        seen.dedup();
        records += seen.len();
        for (d, _, a) in &seen {
            if f.is_bonded(n, *d, *a) {
                contaminated += 1;
            }
        }
        if !inter_block_pairs(&labels, hb).is_empty() {
            frames_with_inter += 1;
        }
    }
    let contamination = if records == 0 {
        0.0
    } else {
        contaminated as f64 / records as f64
    };

    // ---- the charts ---------------------------------------------------------------
    let mut keyed: HashMap<&'static str, (Vec<u64>, usize)> = HashMap::new();
    for chart in Chart::ALL {
        let mut keyer = Keyer::default();
        let keys: Vec<u64> = (0..nf)
            .map(|t| {
                let f = &traj.frames[t];
                keyer.key(reading(chart, n, &f.pos, z, f.bonded, &hbs[t]))
            })
            .collect();
        keyed.insert(chart.tag(), (keys, keyer.distinct()));
    }
    let base = keyed[Chart::MolPart.tag()].0.clone();

    let mut charts = Vec::new();
    for chart in Chart::ALL {
        let (keys, distinct) = &keyed[chart.tag()];
        // G-N0: dense sequential ids are injective by construction; assert it holds.
        assert_eq!(
            *distinct,
            keys.iter().collect::<std::collections::HashSet<_>>().len(),
            "G-N0: key map is not injective on {}",
            chart.tag()
        );
        let changes = (1..nf).filter(|&t| keys[t] != keys[t - 1]).count();
        let closure = census::closure_leg(keys, st);
        let factor = factor_leg(&base, keys, st.min_informative);
        let vacuous = closure.void || changes < RUNG1_MIN_CHANGES || *distinct < RUNG1_MIN_DISTINCT;
        let void_reason = if vacuous {
            Some(format!(
                "G-N7 anti-vacuity: informative {} (>= {}), changes {} (>= {}), distinct {} (>= {})",
                closure.informative_transitions,
                st.min_informative,
                changes,
                RUNG1_MIN_CHANGES,
                distinct,
                RUNG1_MIN_DISTINCT
            ))
        } else {
            None
        };
        charts.push(ChartReport {
            chart,
            distinct: *distinct,
            changes,
            collisions: closure.informative_transitions,
            closure,
            factor,
            vacuous,
            void_reason,
        });
    }

    // ---- Leg A-N: persistent network structures ------------------------------------
    let comps_at: Vec<Vec<Mask>> = hbs.iter().map(|hb| hb_components(z, hb)).collect();
    let mut present: HashMap<Mask, usize> = HashMap::new();
    for cs in &comps_at {
        for &m in cs {
            *present.entry(m).or_insert(0) += 1;
        }
    }
    let mut cands: Vec<Mask> = present.keys().copied().collect();
    cands.sort_unstable();

    let dur: Vec<f64> = (0..nf)
        .map(|i| if i + 1 < nf { times[i + 1] - times[i] } else { 0.0 })
        .collect();

    // The floor is cached per TARGET, because the census's pool rate excludes the target
    // from its own base rate and therefore differs between two structures of one size.
    let mut floor_cache: HashMap<u16, Floor> = HashMap::new();

    let mut structures = Vec::new();
    for m in cands {
        let series: Vec<bool> = comps_at.iter().map(|cs| cs.contains(&m)).collect();
        let longest_run = census::longest_true_run(&series);
        let (longest_fs, longest_at) = census::longest_true_run_fs(&series, &times);
        let size = partition::popcount(m);
        let held_fs: f64 = series
            .iter()
            .enumerate()
            .filter(|(_, v)| **v)
            .map(|(i, _)| dur[i])
            .sum();
        let hit = if held_fs < (1.0 - st.beta) * st.window_fs {
            None
        } else {
            census::strict_window(&series, &times, st.window_fs)
                .map(|(a, b)| (a, b, 0usize, 0.0f64, true))
                .or_else(|| {
                    census::budgeted_window(&series, &times, st.window_fs, st.beta, st.flicker_fs)
                        .map(|(a, b, br, w)| (a, b, br, w, false))
                })
        };

        let (verdict, rms, sv, floor) = match hit {
            None => {
                let (rms, sv) = if longest_run >= 2 {
                    census::carrier_motion(traj, m, longest_at, longest_at + longest_run)
                } else {
                    (0.0, 0.0)
                };
                (StructVerdict::Transient { longest_run }, rms, sv, None)
            }
            Some((a, b, breach, worst, strict)) => {
                let (rms, sv) = census::carrier_motion(traj, m, a, b);
                if rms < st.min_rms_bohr || sv < st.min_sep_var_bohr {
                    (StructVerdict::VoidFrozenCarrier { rms, sep_var: sv }, rms, sv, None)
                } else {
                    let pm = ox_positions_of(&ox, m);
                    let fl = *floor_cache
                        .entry(pm)
                        .or_insert_with(|| control_floor(traj, &hbs, &times, pm, st, surrogates));
                    if fl.p_data > fl.bar {
                        (
                            StructVerdict::VoidNoSeparation { p_data: fl.p_data, bar: fl.bar },
                            rms,
                            sv,
                            Some(fl),
                        )
                    } else if strict {
                        (StructVerdict::CertifiedStrict { window_start: a }, rms, sv, Some(fl))
                    } else {
                        (
                            StructVerdict::CertifiedBudgeted {
                                window_start: a,
                                breach_frames: breach,
                                max_breach_fs: worst,
                            },
                            rms,
                            sv,
                            Some(fl),
                        )
                    }
                }
            }
        };

        structures.push(StructReport {
            structure: m,
            size,
            frames_present: present[&m],
            longest_run,
            longest_run_fs: longest_fs,
            verdict,
            rms_internal: rms,
            max_sep_variation: sv,
            floor,
        });
    }
    structures.sort_by(|a, b| {
        b.longest_run_fs
            .partial_cmp(&a.longest_run_fs)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.structure.cmp(&b.structure))
    });

    Rung1::Report(Box::new(Rung1Report {
        seed: traj.header.seed,
        n_atoms: n,
        n_oxygens: ox.len(),
        n_frames: nf,
        span_fs,
        window_fs: st.window_fs,
        contamination,
        hb_records: records,
        hb_contaminated: contaminated,
        frames_with_intermolecular: frames_with_inter,
        charts,
        structures,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lens::HBond;

    fn hb(d: usize, h: usize, a: usize) -> HBond {
        HBond { donor_o: d, hydrogen: h, acceptor_o: a }
    }

    /// Z for a 4-oxygen, 8-hydrogen scene laid out as the mixed arm's: oxygens first.
    fn mixed_z() -> Vec<u32> {
        let mut z = vec![8u32; 4];
        z.extend(std::iter::repeat(1u32).take(8));
        z
    }

    #[test]
    fn hb_components_are_arena_masks_of_two_or_more() {
        let z = mixed_z();
        // 0—1 bonded, 2 and 3 isolated.
        let c = hb_components(&z, &[hb(0, 4, 1)]);
        assert_eq!(c, vec![0b0011]);
    }

    #[test]
    fn direction_does_not_split_a_component() {
        let z = mixed_z();
        let a = hb_components(&z, &[hb(0, 4, 1)]);
        let b = hb_components(&z, &[hb(1, 5, 0)]);
        assert_eq!(a, b, "the undirected projection must not depend on donor order");
    }

    /// C2 is a function of C1 and C1 is a function of C1H, so a chart reading that
    /// differs at a coarse rung must differ at every finer one. This is the ladder
    /// `refinement_removes_collisions` is checked against on real data (G-N2).
    #[test]
    fn the_ladder_is_actually_nested() {
        let z = mixed_z();
        let n = 12;
        let p = vec![[0.0f64; 3]; n];
        // Two H-bond sets that agree on C1 (same donor→acceptor pairs) but differ on
        // C1H (different mediating hydrogen). Coarser must agree, finer must differ.
        let x = vec![hb(0, 4, 1)];
        let y = vec![hb(0, 5, 1)];
        assert_ne!(
            reading(Chart::HbFull, n, &p, &z, 0, &x),
            reading(Chart::HbFull, n, &p, &z, 0, &y)
        );
        assert_eq!(
            reading(Chart::HbAdj, n, &p, &z, 0, &x),
            reading(Chart::HbAdj, n, &p, &z, 0, &y)
        );
        assert_eq!(
            reading(Chart::HbPart, n, &p, &z, 0, &x),
            reading(Chart::HbPart, n, &p, &z, 0, &y)
        );
    }

    /// The network of MOLECULES has no edge inside a molecule. If the engine's own bond
    /// graph puts the two oxygens in one block, the H-bond between them is not a network
    /// edge, and C4/C5 must not carry it.
    #[test]
    fn intra_molecular_hbonds_are_not_network_edges() {
        let z = mixed_z();
        let n = 12;
        let p = vec![[0.0f64; 3]; n];
        let bonds = crate::synthetic::bonds_from_blocks(n, &[0b0011]); // atoms 0,1 one block
        let x = vec![hb(0, 4, 1)];
        let with = reading(Chart::MolNetId, n, &p, &z, bonds, &x);
        let without = reading(Chart::MolNetId, n, &p, &z, bonds, &[]);
        assert_eq!(with, without, "an intra-block H-bond must not appear as a network edge");
        // The same H-bond across a partition boundary IS an edge.
        let split = reading(Chart::MolNetId, n, &p, &z, 0, &x);
        assert_ne!(split, reading(Chart::MolNetId, n, &p, &z, 0, &[]));
    }

    /// A chart factors through itself, exactly. The instrument's own zero.
    #[test]
    fn a_chart_factors_through_itself() {
        let keys = vec![1u64, 2, 1, 3, 2, 1, 3, 3];
        let f = factor_leg(&keys, &keys, 0);
        assert_eq!(f.violations, 0);
        assert_eq!(f.defect, 0.0);
        assert_eq!(f.witness_count, 0);
    }

    /// Against a CONSTANT base every frame is one group, so the defect is exactly the
    /// non-majority fraction of the fine reading. Two known ends, so the arithmetic is
    /// pinned from both sides.
    #[test]
    fn a_constant_base_measures_the_fine_reading_spread() {
        let base = vec![7u64; 8];
        let fine = vec![1u64, 1, 1, 1, 1, 2, 2, 3];
        let f = factor_leg(&base, &fine, 0);
        assert_eq!(f.informative, 8);
        assert_eq!(f.violations, 3); // 8 minus the majority's 5
        assert_eq!(f.witness_count, 2); // three distinct images
    }

    #[test]
    fn splitmix_is_deterministic_from_its_seed() {
        let mut a = SplitMix(12345);
        let mut b = SplitMix(12345);
        for _ in 0..8 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        assert_ne!(SplitMix(1).next_u64(), SplitMix(2).next_u64());
    }

    #[test]
    fn keyer_is_injective_and_dense() {
        let mut k = Keyer::default();
        assert_eq!(k.key(vec![1, 2]), 0);
        assert_eq!(k.key(vec![3]), 1);
        assert_eq!(k.key(vec![1, 2]), 0);
        assert_eq!(k.distinct(), 2);
    }

    /// A scene with fewer than two oxygens has no network variable, and the instrument
    /// must say so by name rather than return a zero (OBJECT.md rule 9).
    #[test]
    fn a_scene_without_two_oxygens_is_refused() {
        let spec = crate::synthetic::Spec::quench_like(1400, vec![1u32; 12]);
        let t = crate::synthetic::build(spec, |_, _, _| 0);
        match run(&t, &Stakes::default(), 4) {
            Rung1::Refused { gate, .. } => assert!(gate.contains("G-N12")),
            Rung1::Report(_) => panic!("a 12-hydrogen scene must be refused, not reported"),
        }
    }
}
